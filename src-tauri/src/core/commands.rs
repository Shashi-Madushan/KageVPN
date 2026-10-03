use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use super::provider::{newest, Activation, CoreProvider};
use super::releases::{self, Release};
use super::version::Version;
use super::{resolve, should_auto_install, Channel, CoreInfo, CoreSettings, Installed, Source, UpdatePolicy};
use crate::app_config::{core_spec, CoreSpec, CORES};
use crate::downloads::{self, Guard};
use crate::error::{AppError, Result};
use crate::session::State as Conn;
use crate::state::AppState;

const CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;
const BREW_PATHS: [&str; 2] = ["/opt/homebrew/bin/brew", "/usr/local/bin/brew"];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreRef {
    id: &'static str,
    name: &'static str,
    runnable: bool,
    min_version: &'static str,
    required_features: &'static [&'static str],
}

impl From<&CoreSpec> for CoreRef {
    fn from(spec: &CoreSpec) -> Self {
        Self {
            id: spec.id,
            name: spec.name,
            runnable: spec.runnable,
            min_version: spec.min_version,
            required_features: spec.required_features,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreStatus {
    core: CoreRef,
    cores: Vec<CoreRef>,
    active: Option<CoreInfo>,
    error: Option<String>,
    system: Option<CoreInfo>,
    installed: Vec<Installed>,
    settings: CoreSettings,
    busy: bool,
}

/// The core that connections use, with its settings; commands act on this one.
fn active_core(state: &AppState) -> (&'static CoreSpec, CoreSettings) {
    let store = state.store.lock().unwrap();
    (store.active_core(), store.core_settings())
}

#[tauri::command]
pub fn core_status(state: State<AppState>) -> CoreStatus {
    let (spec, settings) = active_core(&state);
    let system = super::locate(spec).ok().and_then(|path| super::inspect(spec, &path, Source::System).ok());
    let (active, error) = match resolve(&state.paths, spec, &settings) {
        Ok((path, source)) => match super::inspect(spec, &path, source) {
            Ok(info) => (Some(info), None),
            Err(e) => (None, Some(e.to_string())),
        },
        Err(e) => (None, Some(e.to_string())),
    };
    CoreStatus {
        core: spec.into(),
        cores: CORES.iter().map(CoreRef::from).collect(),
        active,
        error,
        system,
        installed: super::installed(&state.paths, spec),
        settings,
        busy: state.downloads.is_busy(),
    }
}

#[tauri::command]
pub async fn core_check(state: State<'_, AppState>) -> Result<Vec<Release>> {
    let (spec, settings) = active_core(&state);
    let releases = releases::fetch(spec, settings.channel).await?;
    let mut store = state.store.lock().unwrap();
    let mut settings = store.core_settings_for(spec.id);
    settings.last_check = now_secs();
    store.set_core_settings_for(spec.id, settings)?;
    Ok(releases)
}

/// Starts an install on a background task and returns at once. Progress arrives as
/// `download-progress` events and the outcome as `download-finished`.
#[tauri::command]
pub fn core_install_background(app: AppHandle, version: Option<String>, allow_unverified: Option<bool>) -> Result<()> {
    let spec = app.state::<AppState>().store.lock().unwrap().active_core();
    let provider = CoreProvider { spec, allow_unverified: allow_unverified.unwrap_or(false), activation: Activation::Now };
    downloads::spawn_install(app, provider, version)
}

#[tauri::command]
pub fn core_use(state: State<AppState>, version: Option<String>) -> Result<()> {
    let (spec, _) = active_core(&state);
    if let Some(version) = &version {
        if !super::managed_binary(&state.paths, spec, version).is_file() {
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
    let (spec, settings) = active_core(&state);
    let active = settings.managed_version;
    if active.as_deref() == Some(version.as_str()) {
        return Err(AppError::Invalid("switch to another core before removing this one".into()));
    }
    if Version::parse(&version).is_none() {
        return Err(AppError::Invalid("invalid version".into()));
    }
    std::fs::remove_dir_all(state.paths.core_dir_for(spec.id).join(version))?;
    Ok(())
}

/// Chooses which core runs connections. Only cores with a runtime adapter can be chosen.
#[tauri::command]
pub fn core_set_active(state: State<AppState>, id: String) -> Result<()> {
    let spec = core_spec(&id).ok_or_else(|| AppError::Invalid(format!("unknown core {id}")))?;
    if !spec.runnable {
        return Err(AppError::Invalid(format!("{} can be downloaded but cannot run connections yet", spec.name)));
    }
    if state.status.lock().unwrap().state != Conn::Disconnected {
        return Err(AppError::Invalid("disconnect before switching cores".into()));
    }
    state.store.lock().unwrap().set_active_core(spec.id)
}

#[tauri::command]
pub fn set_core_channel(state: State<AppState>, channel: Channel) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    let mut settings = store.core_settings();
    settings.channel = channel;
    store.set_core_settings(settings)
}

#[tauri::command]
pub fn set_core_update_policy(state: State<AppState>, policy: UpdatePolicy) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    let mut settings = store.core_settings();
    settings.update_policy = policy;
    settings.auto_check = policy != UpdatePolicy::Off;
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
        let (spec, settings) = active_core(&state);
        if !settings.auto_check
            || settings.update_policy == UpdatePolicy::Off
            || now_secs().saturating_sub(settings.last_check) < CHECK_INTERVAL_SECS
        {
            return;
        }
        let Ok(releases) = releases::fetch(spec, settings.channel).await else { return };

        let resolved = resolve(&state.paths, spec, &settings)
            .ok()
            .and_then(|(path, source)| super::inspect(spec, &path, source).ok().map(|info| (info, source)));
        let latest = newest(releases);

        if let (Some((info, source)), Some(latest)) = (resolved, latest) {
            if Version::parse(&latest.version) > Version::parse(&info.version) {
                if should_auto_install(settings.update_policy, settings.channel, source) {
                    if let Ok(guard) = Guard::acquire(&app) {
                        let provider = CoreProvider { spec, allow_unverified: false, activation: Activation::WhenIdle };
                        downloads::run_guarded(&app, &provider, Some(&latest.version), guard).await;
                    }
                } else {
                    let provider = CoreProvider { spec, allow_unverified: false, activation: Activation::Now };
                    downloads::announce_available(&app, &provider, &latest.version);
                }
            }
        }
        let mut store = state.store.lock().unwrap();
        let mut updated = store.core_settings_for(spec.id);
        updated.last_check = now_secs();
        let _ = store.set_core_settings_for(spec.id, updated);
    });
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}
