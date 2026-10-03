//! One place that runs every download the app does (cores today, the app's own updates
//! next): a shared lock so only one runs at a time, one cancel flag, one pair of events for
//! progress and completion, and one notification path. A thing that can be downloaded
//! implements [`DownloadProvider`]; everything else here is generic over it.

use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::error::{AppError, Result};
use crate::notify;
use crate::state::AppState;

/// What to do once a newer version is found.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdatePolicy {
    /// Notify and let the user decide.
    #[default]
    Ask,
    /// Download in the background and switch over while disconnected.
    Auto,
    /// Never check.
    Off,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    App,
    Core,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Download,
    Verify,
    Extract,
    Check,
    Install,
    Done,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub id: String,
    pub kind: Kind,
    pub title: String,
    pub version: String,
    pub stage: Stage,
    pub received: u64,
    pub total: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finished {
    pub id: String,
    pub kind: Kind,
    pub title: String,
    pub version: Option<String>,
    pub error: Option<String>,
}

/// Something the app can fetch and install. Providers differ in where releases come from and
/// how they are installed; the lock, events and notifications are shared.
pub trait DownloadProvider: Send + Sync + 'static {
    type Release: Send + Sync;

    /// Stable key, such as `core:sing-box` or `app`.
    fn id(&self) -> String;
    fn kind(&self) -> Kind;
    /// Display name, such as `sing-box` or the app's own name.
    fn name(&self) -> String;
    fn version_of(&self, release: &Self::Release) -> String;
    /// Text for the notification shown when an install succeeds.
    fn success_message(&self, version: &str) -> String;
    /// The given version, or the newest suitable one when `None`.
    fn resolve(&self, version: Option<&str>) -> impl Future<Output = Result<Self::Release>> + Send;
    fn install(
        &self,
        app: &AppHandle,
        release: &Self::Release,
        reporter: &Reporter,
    ) -> impl Future<Output = Result<()>> + Send;
}

/// Shared state for the one-at-a-time rule and cancellation.
#[derive(Default)]
pub struct Downloads {
    busy: AtomicBool,
    cancel: AtomicBool,
}

impl Downloads {
    pub fn is_busy(&self) -> bool {
        self.busy.load(Ordering::SeqCst)
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }
}

/// Holds the "one download at a time" flag for as long as it lives.
pub struct Guard(AppHandle);

impl Guard {
    pub fn acquire(app: &AppHandle) -> Result<Self> {
        let downloads = &app.state::<AppState>().downloads;
        if downloads.busy.swap(true, Ordering::SeqCst) {
            return Err(AppError::Invalid("another download is already in progress".into()));
        }
        downloads.cancel.store(false, Ordering::SeqCst);
        Ok(Self(app.clone()))
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        self.0.state::<AppState>().downloads.busy.store(false, Ordering::SeqCst);
    }
}

/// Hands a provider one place to report progress and to learn the user cancelled.
#[derive(Clone)]
pub struct Reporter {
    app: AppHandle,
    id: String,
    kind: Kind,
    title: String,
    version: String,
}

impl Reporter {
    pub fn report(&self, stage: Stage, received: u64, total: u64) {
        let progress = Progress {
            id: self.id.clone(),
            kind: self.kind,
            title: self.title.clone(),
            version: self.version.clone(),
            stage,
            received,
            total,
        };
        let _ = self.app.emit("download-progress", progress);
    }

    pub fn cancelled(&self) -> bool {
        self.app.state::<AppState>().downloads.is_cancelled()
    }
}

/// Starts a download on a background task and returns at once. Progress arrives as
/// `download-progress` events and the outcome as `download-finished`.
pub fn spawn_install<P: DownloadProvider>(app: AppHandle, provider: P, version: Option<String>) -> Result<()> {
    let guard = Guard::acquire(&app)?;
    tauri::async_runtime::spawn(async move {
        run_guarded(&app, &provider, version.as_deref(), guard).await;
    });
    Ok(())
}

/// Runs one install while holding the lock, then reports the outcome to the UI and as a
/// notification. For callers that already hold the lock (background update checks).
pub async fn run_guarded<P: DownloadProvider>(app: &AppHandle, provider: &P, version: Option<&str>, guard: Guard) {
    let outcome = async {
        let release = provider.resolve(version).await?;
        let version = provider.version_of(&release);
        let reporter = Reporter {
            app: app.clone(),
            id: provider.id(),
            kind: provider.kind(),
            title: provider.name(),
            version: version.clone(),
        };
        provider.install(app, &release, &reporter).await.map(|_| version)
    }
    .await;
    drop(guard);
    announce(app, provider, outcome);
}

fn announce<P: DownloadProvider>(app: &AppHandle, provider: &P, outcome: Result<String>) {
    let name = provider.name();
    let (notice, finished) = match outcome {
        Ok(version) => (
            (format!("{name} ready"), provider.success_message(&version)),
            Finished { id: provider.id(), kind: provider.kind(), title: name, version: Some(version), error: None },
        ),
        Err(e) => (
            (format!("{name} download failed"), e.to_string()),
            Finished { id: provider.id(), kind: provider.kind(), title: name, version: None, error: Some(e.to_string()) },
        ),
    };
    let cancelled = finished.error.as_deref().is_some_and(|e| e.contains("cancelled"));
    let _ = app.emit("download-finished", finished);
    if !cancelled {
        notify::send(app, &notice.0, &notice.1);
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAvailable {
    pub id: String,
    pub kind: Kind,
    pub name: String,
    pub version: String,
}

/// Tells the UI a newer version exists (the `update-available` event) and notifies the user.
pub fn announce_available<P: DownloadProvider>(app: &AppHandle, provider: &P, version: &str) {
    let name = provider.name();
    let _ = app.emit(
        "update-available",
        UpdateAvailable { id: provider.id(), kind: provider.kind(), name: name.clone(), version: version.to_string() },
    );
    notify::send(app, &format!("{name} update available"), &format!("Version {version} is ready to install. Open Home to update."));
}

#[tauri::command]
pub fn download_cancel(state: State<AppState>) {
    state.downloads.cancel();
}

#[cfg(test)]
#[path = "tests/downloads.rs"]
mod tests;
