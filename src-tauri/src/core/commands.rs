use std::process::Command;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use super::releases::{self, Release};
use super::version::Version;
use super::{installer, resolve, Channel, CoreInfo, CoreSettings, Installed, Source};
use crate::config::{self, ConfigInput, Mode};
use crate::error::{AppError, Result};
use crate::state::AppState;
use crate::{notify, singbox};

const CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;
const BREW_PATHS: [&str; 2] = ["/opt/homebrew/bin/brew", "/usr/local/bin/brew"];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreStatus {
    active: Option<CoreInfo>,
    error: Option<String>,
    system: Option<CoreInfo>,
    installed: Vec<Installed>,
    settings: CoreSettings,
    busy: bool,
}

#[tauri::command]
pub fn core_status(state: State<AppState>) -> CoreStatus {
    let settings = state.store.lock().unwrap().core_settings();
    let system = singbox::locate().ok().and_then(|path| super::inspect(&path, Source::System).ok());
    let (active, error) = match resolve(&state.paths, &settings) {
        Ok((path, source)) => match super::inspect(&path, source) {
            Ok(info) => (Some(info), None),
            Err(e) => (None, Some(e.to_string())),
        },
        Err(e) => (None, Some(e.to_string())),
    };
    CoreStatus {
        active,
        error,
        system,
        installed: super::installed(&state.paths),
        settings,
        busy: state.core_busy.load(Ordering::SeqCst),
    }
}

#[tauri::command]
pub async fn core_check(state: State<'_, AppState>) -> Result<Vec<Release>> {
    let channel = state.store.lock().unwrap().core_settings().channel;
    let releases = releases::fetch(channel).await?;
    let mut store = state.store.lock().unwrap();
    let mut settings = store.core_settings();
    settings.last_check = now_secs();
    store.set_core_settings(settings)?;
    Ok(releases)
}

#[tauri::command]
pub async fn core_install(app: AppHandle, version: String, allow_unverified: bool) -> Result<()> {
    let state = app.state::<AppState>();
    if state.core_busy.swap(true, Ordering::SeqCst) {
        return Err(AppError::Invalid("another core operation is in progress".into()));
    }
    let result = install_version(&app, &version, allow_unverified).await;
    state.core_busy.store(false, Ordering::SeqCst);
    result
}

async fn install_version(app: &AppHandle, version: &str, allow_unverified: bool) -> Result<()> {
    let state = app.state::<AppState>();
    let release = releases::fetch(Channel::Prerelease)
        .await?
        .into_iter()
        .find(|r| r.version == version)
        .ok_or_else(|| AppError::Invalid(format!("version {version} was not found")))?;

    let check_config = {
        let store = state.store.lock().unwrap();
        let profiles = store.profiles().to_vec();
        let routing = store.routing();
        (!profiles.is_empty())
            .then(|| {
                config::build(&ConfigInput {
                    profiles: &profiles,
                    active_id: store.active_id(),
                    auto: false,
                    mode: Mode::Proxy,
                    routing: &routing,
                    log_path: &state.paths.log_file(),
                    cache_path: &state.paths.cache_file(),
                    secret: "check",
                })
                .ok()
            })
            .flatten()
    };

    installer::install(app, &state.paths, &release, allow_unverified, check_config).await?;

    let mut store = state.store.lock().unwrap();
    let mut settings = store.core_settings();
    settings.managed_version = Some(release.version);
    store.set_core_settings(settings)
}

#[tauri::command]
pub fn core_use(state: State<AppState>, version: Option<String>) -> Result<()> {
    if let Some(version) = &version {
        if !super::managed_binary(&state.paths, version).is_file() {
            return Err(AppError::Invalid(format!("version {version} is not installed")));
        }
    }
    let mut store = state.store.lock().unwrap();
    let mut settings = store.core_settings();
    settings.managed_version = version;
    store.set_core_settings(settings)
}

#[tauri::command]
pub fn core_remove(state: State<AppState>, version: String) -> Result<()> {
    let active = state.store.lock().unwrap().core_settings().managed_version;
    if active.as_deref() == Some(version.as_str()) {
        return Err(AppError::Invalid("switch to another core before removing this one".into()));
    }
    if Version::parse(&version).is_none() {
        return Err(AppError::Invalid("invalid version".into()));
    }
    std::fs::remove_dir_all(state.paths.core_dir().join(version))?;
    Ok(())
}

#[tauri::command]
pub fn update_core_settings(state: State<AppState>, channel: Channel, auto_check: bool) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    let mut settings = store.core_settings();
    settings.channel = channel;
    settings.auto_check = auto_check;
    store.set_core_settings(settings)
}

#[tauri::command]
pub async fn core_brew_upgrade() -> Result<String> {
    let brew = BREW_PATHS
        .iter()
        .find(|path| std::path::Path::new(path).is_file())
        .ok_or_else(|| AppError::Invalid("Homebrew was not found".into()))?;
    let output = tauri::async_runtime::spawn_blocking(move || {
        Command::new(brew).args(["upgrade", "sing-box"]).output()
    })
    .await
    .map_err(|e| AppError::Launch(e.to_string()))??;

    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let tail: Vec<&str> = text.lines().rev().take(6).collect();
    let tail = tail.into_iter().rev().collect::<Vec<_>>().join("\n");
    if output.status.success() {
        Ok(tail)
    } else {
        Err(AppError::Launch(tail))
    }
}

pub fn spawn_background_check(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let settings = state.store.lock().unwrap().core_settings();
        if !settings.auto_check || now_secs().saturating_sub(settings.last_check) < CHECK_INTERVAL_SECS {
            return;
        }
        let Ok(releases) = releases::fetch(settings.channel).await else { return };

        let current = resolve(&state.paths, &settings)
            .ok()
            .and_then(|(path, source)| super::inspect(&path, source).ok())
            .and_then(|info| Version::parse(&info.version));
        let newest = releases.iter().find_map(|r| Version::parse(&r.version).map(|v| (v, r.version.clone())));

        if let (Some(current), Some((latest, label))) = (current, newest) {
            if latest > current {
                notify::send(&app, "sing-box update available", &format!("Version {label} is ready to install."));
            }
        }
        let mut store = state.store.lock().unwrap();
        let mut updated = store.core_settings();
        updated.last_check = now_secs();
        let _ = store.set_core_settings(updated);
    });
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}
