//! Updates for the app itself. The feed URL and signing key come from `app_config.rs`; with
//! either unset nothing here does anything. Updates download through the shared download
//! manager, are signature-checked by the updater plugin, and never restart the app on their
//! own: the user chooses when, because a restart ends a live VPN connection.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime, State};
use tauri_plugin_updater::{Update, Updater, UpdaterExt};
use tokio::time::sleep;

use crate::app_config::{self, APP_UPDATE_FEED, APP_UPDATE_PUBKEY};
use crate::downloads::{self, DownloadProvider, Guard, Kind, Reporter, Stage, UpdatePolicy};
use crate::error::{AppError, Result};
use crate::session::State as Conn;
use crate::state::AppState;

const CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;
const CANCEL_POLL: Duration = Duration::from_millis(150);

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppUpdateSettings {
    pub policy: UpdatePolicy,
    pub last_check: u64,
}

impl Default for AppUpdateSettings {
    fn default() -> Self {
        Self { policy: UpdatePolicy::Ask, last_check: 0 }
    }
}

/// The feed and key, or `None` when this build has app updates switched off.
fn configured() -> Option<(&'static str, &'static str)> {
    Some((APP_UPDATE_FEED?, APP_UPDATE_PUBKEY?))
}

fn failure(e: tauri_plugin_updater::Error) -> AppError {
    AppError::Launch(format!("update check failed: {e}"))
}

fn build_updater<R: Runtime>(app: &AppHandle<R>, feed: &str, key: &str) -> Result<Updater> {
    let feed = feed.parse().map_err(|_| AppError::Invalid("the update feed address is not valid".into()))?;
    app.updater_builder()
        .endpoints(vec![feed])
        .map_err(failure)?
        .pubkey(key)
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(failure)
}

async fn check_for_update(app: &AppHandle) -> Result<Option<Update>> {
    let (feed, key) = configured().ok_or_else(|| AppError::Invalid("app updates are not set up in this build".into()))?;
    build_updater(app, feed, key)?.check().await.map_err(failure)
}

/// Downloads and installs the app's own new version.
pub struct AppProvider {
    pub app: AppHandle,
}

impl DownloadProvider for AppProvider {
    type Release = Update;

    fn id(&self) -> String {
        "app".into()
    }

    fn kind(&self) -> Kind {
        Kind::App
    }

    fn name(&self) -> String {
        app_config::APP_NAME.into()
    }

    fn version_of(&self, release: &Update) -> String {
        release.version.clone()
    }

    fn success_message(&self, version: &str) -> String {
        format!("Version {version} is installed. Restart to start using it.")
    }

    async fn resolve(&self, _version: Option<&str>) -> Result<Update> {
        check_for_update(&self.app).await?.ok_or_else(|| AppError::Invalid("you already have the latest version".into()))
    }

    async fn install(&self, _app: &AppHandle, release: &Update, reporter: &Reporter) -> Result<()> {
        let mut received = 0u64;
        let download = release.download(
            |chunk, total| {
                received += chunk as u64;
                reporter.report(Stage::Download, received, total.unwrap_or(0));
            },
            || reporter.report(Stage::Verify, 0, 0),
        );
        let cancelled = async {
            while !reporter.cancelled() {
                sleep(CANCEL_POLL).await;
            }
        };
        // Dropping the download future when the user cancels stops the transfer.
        let bytes = tokio::select! {
            result = download => result.map_err(|e| AppError::Launch(format!("update download failed: {e}")))?,
            _ = cancelled => return Err(AppError::Launch("download cancelled".into())),
        };

        reporter.report(Stage::Install, 0, 0);
        release.install(bytes).map_err(|e| AppError::Launch(format!("could not install the update: {e}")))?;
        reporter.report(Stage::Done, 0, 0);
        Ok(())
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn stamp_check(state: &AppState) {
    let mut store = state.store.lock().unwrap();
    let mut settings = store.app_update();
    settings.last_check = now_secs();
    let _ = store.set_app_update(settings);
}

/// Checks once a day at launch. "Ask" announces; "Automatic" downloads and installs, leaving
/// the restart to the user.
pub fn spawn_background_check(app: AppHandle) {
    if configured().is_none() {
        return;
    }
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let settings = state.store.lock().unwrap().app_update();
        if settings.policy == UpdatePolicy::Off || now_secs().saturating_sub(settings.last_check) < CHECK_INTERVAL_SECS {
            return;
        }
        let Ok(Some(update)) = check_for_update(&app).await else {
            stamp_check(&state);
            return;
        };
        let provider = AppProvider { app: app.clone() };
        match settings.policy {
            UpdatePolicy::Auto => {
                if let Ok(guard) = Guard::acquire(&app) {
                    downloads::run_guarded(&app, &provider, Some(&update.version), guard).await;
                }
            }
            _ => downloads::announce_available(&app, &provider, &update.version),
        }
        stamp_check(&state);
    });
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateInfo {
    version: String,
    notes: Option<String>,
}

#[tauri::command]
pub async fn app_update_check(state: State<'_, AppState>, app: AppHandle) -> Result<Option<AppUpdateInfo>> {
    let found = check_for_update(&app).await?;
    stamp_check(&state);
    Ok(found.map(|u| AppUpdateInfo { version: u.version, notes: u.body }))
}

#[tauri::command]
pub fn app_update_install(app: AppHandle) -> Result<()> {
    downloads::spawn_install(app.clone(), AppProvider { app }, None)
}

#[tauri::command]
pub fn app_update_settings(state: State<AppState>) -> AppUpdateSettings {
    state.store.lock().unwrap().app_update()
}

#[tauri::command]
pub fn set_app_update_policy(state: State<AppState>, policy: UpdatePolicy) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    let mut settings = store.app_update();
    settings.policy = policy;
    store.set_app_update(settings)
}

/// Restarts into the installed update. A live connection is only ended when `force` says the
/// user agreed, because the VPN stops with the app.
#[tauri::command]
pub fn app_restart(app: AppHandle, state: State<AppState>, force: bool) -> Result<()> {
    if !force && state.status.lock().unwrap().state != Conn::Disconnected {
        return Err(AppError::Invalid("restarting will disconnect the VPN".into()));
    }
    tauri::async_runtime::spawn(async move {
        let _ = crate::session::disconnect(&app).await;
        app.restart();
    });
    Ok(())
}

#[cfg(test)]
#[path = "tests/app_update.rs"]
mod tests;
