use std::process::Command;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use super::releases::{self, Release};
use super::version::Version;
use super::{installer, resolve, should_auto_install, Channel, CoreInfo, CoreSettings, Installed, Source, UpdatePolicy};
use crate::app_config::{core_spec, CoreSpec, CORES};
use crate::config::{self, ConfigInput, Mode};
use crate::error::{AppError, Result};
use crate::session::State as Conn;
use crate::state::AppState;
use crate::notify;

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
        busy: state.core_busy.load(Ordering::SeqCst),
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

/// Holds the "one core operation at a time" flag for as long as it lives.
struct Busy(AppHandle);

impl Busy {
    fn acquire(app: &AppHandle) -> Result<Self> {
        let state = app.state::<AppState>();
        if state.core_busy.swap(true, Ordering::SeqCst) {
            return Err(AppError::Invalid("another core operation is in progress".into()));
        }
        state.core_cancel.store(false, Ordering::SeqCst);
        Ok(Self(app.clone()))
    }
}

impl Drop for Busy {
    fn drop(&mut self) {
        self.0.state::<AppState>().core_busy.store(false, Ordering::SeqCst);
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Finished {
    core: String,
    version: Option<String>,
    error: Option<String>,
}

/// Starts an install on a background task and returns at once. Progress arrives as
/// `core-progress` events and the outcome as `core-finished`, so the UI can be left meanwhile.
#[tauri::command]
pub fn core_install_background(app: AppHandle, version: Option<String>, allow_unverified: Option<bool>) -> Result<()> {
    let busy = Busy::acquire(&app)?;
    let spec = app.state::<AppState>().store.lock().unwrap().active_core();
    tauri::async_runtime::spawn(async move {
        let outcome = async {
            let release = match &version {
                Some(version) => find_release(spec, version).await?,
                None => latest_stable(spec).await?,
            };
            install_release(&app, spec, &release, allow_unverified.unwrap_or(false), Activation::Now).await.map(|_| release.version)
        }
        .await;
        drop(busy);
        report(&app, spec, outcome);
    });
    Ok(())
}

#[tauri::command]
pub fn core_cancel(state: State<AppState>) {
    state.core_cancel.store(true, Ordering::SeqCst);
}

fn report(app: &AppHandle, spec: &CoreSpec, outcome: Result<String>) {
    let core = spec.name.to_string();
    let (title, body, finished) = match outcome {
        Ok(version) => (
            format!("{} engine ready", spec.name),
            format!("Version {version} is installed."),
            Finished { core, version: Some(version), error: None },
        ),
        Err(e) => (format!("{} download failed", spec.name), e.to_string(), Finished { core, version: None, error: Some(e.to_string()) }),
    };
    let _ = app.emit("core-finished", finished);
    notify::send(app, &title, &body);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Activation {
    /// Make it the active engine immediately.
    Now,
    /// Make it active now if idle, otherwise on the next connect.
    WhenIdle,
}

async fn find_release(spec: &CoreSpec, version: &str) -> Result<Release> {
    releases::fetch(spec, Channel::Prerelease)
        .await?
        .into_iter()
        .find(|r| r.version == version)
        .ok_or_else(|| AppError::Invalid(format!("version {version} was not found")))
}

async fn latest_stable(spec: &CoreSpec) -> Result<Release> {
    let found = releases::fetch(spec, Channel::Stable).await?;
    newest(found).ok_or_else(|| AppError::Invalid(format!("no stable {} release was found for this Mac", spec.name)))
}

pub(crate) fn newest(releases: Vec<Release>) -> Option<Release> {
    releases
        .into_iter()
        .filter_map(|r| Version::parse(&r.version).map(|v| (v, r)))
        .max_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, r)| r)
}

async fn install_release(app: &AppHandle, spec: &CoreSpec, release: &Release, allow_unverified: bool, activation: Activation) -> Result<()> {
    let state = app.state::<AppState>();

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

    installer::install(app, &state.paths, spec, release, allow_unverified, check_config).await?;

    let idle = state.status.lock().unwrap().state == Conn::Disconnected;
    let mut store = state.store.lock().unwrap();
    let mut settings = store.core_settings_for(spec.id);
    if activation == Activation::Now || idle {
        settings.managed_version = Some(release.version.clone());
        settings.pending_version = None;
    } else {
        settings.pending_version = Some(release.version.clone());
    }
    store.set_core_settings_for(spec.id, settings)
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
                if should_auto_install(settings.update_policy, settings.channel, source) && !state.core_busy.load(Ordering::SeqCst) {
                    if let Ok(busy) = Busy::acquire(&app) {
                        let outcome = install_release(&app, spec, &latest, false, Activation::WhenIdle).await.map(|_| latest.version.clone());
                        drop(busy);
                        report(&app, spec, outcome);
                    }
                } else {
                    let _ = app.emit("core-update", latest.version.clone());
                    notify::send(&app, "sing-box update available", &format!("Version {} is ready to install. Open Home to update.", latest.version));
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
