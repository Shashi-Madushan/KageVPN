use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::sleep;

use crate::config::{self, Mode};
use crate::error::{AppError, Result};
use crate::profile::Profile;
use crate::state::AppState;
use crate::{net, runner, singbox, traffic};

const READY_ATTEMPTS: u32 = 60;
const STOP_ATTEMPTS: u32 = 40;
const POLL_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum State {
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub state: State,
    pub mode: Mode,
    pub since: Option<u64>,
    pub error: Option<String>,
}

impl Status {
    pub fn disconnected(mode: Mode) -> Self {
        Self { state: State::Disconnected, mode, since: None, error: None }
    }
}

pub async fn connect(app: &AppHandle) -> Result<()> {
    let state = app.state::<AppState>();
    let (profile, mode) = {
        let store = state.store.lock().unwrap();
        let profile = store
            .active_profile()
            .cloned()
            .ok_or_else(|| AppError::Invalid("select a profile first".into()))?;
        (profile, store.mode())
    };

    begin(app, mode)?;
    let secret = uuid::Uuid::new_v4().to_string();
    match launch(app, &profile, mode, &secret).await {
        Ok(pid) => {
            *state.pid.lock().unwrap() = Some(pid);
            update(app, |s| {
                s.state = State::Connected;
                s.since = Some(now_secs());
            });
            watch(app.clone(), pid);
            traffic::spawn(app.clone(), pid, secret);
            Ok(())
        }
        Err(error) => {
            let _ = runner::request_stop(&state.paths);
            update(app, |s| *s = Status { error: Some(error.to_string()), ..Status::disconnected(mode) });
            Err(error)
        }
    }
}

pub async fn disconnect(app: &AppHandle) -> Result<()> {
    let state = app.state::<AppState>();
    let Some(pid) = *state.pid.lock().unwrap() else {
        return Ok(());
    };

    runner::request_stop(&state.paths)?;
    for _ in 0..STOP_ATTEMPTS {
        if !runner::is_alive(pid) {
            break;
        }
        sleep(POLL_INTERVAL).await;
    }
    finish(app, pid);
    Ok(())
}

fn begin(app: &AppHandle, mode: Mode) -> Result<()> {
    let state = app.state::<AppState>();
    let mut status = state.status.lock().unwrap();
    if status.state != State::Disconnected {
        return Err(AppError::Busy);
    }
    *status = Status { state: State::Connecting, ..Status::disconnected(mode) };
    let _ = app.emit("status", status.clone());
    Ok(())
}

async fn launch(app: &AppHandle, profile: &Profile, mode: Mode, secret: &str) -> Result<u32> {
    let state = app.state::<AppState>();
    let paths = &state.paths;

    let binary = singbox::locate()?;
    if net::proxy_port_open().await {
        return Err(AppError::Launch(format!(
            "port {} is already in use, is another sing-box running?",
            config::MIXED_PORT
        )));
    }
    state.logs.clear();
    paths.reset_runtime_files()?;

    let config = config::build(profile, mode, &paths.log_file(), secret);
    std::fs::write(paths.config_file(), serde_json::to_vec_pretty(&config)?)?;
    singbox::check(&binary, &paths.config_file())?;

    let pid = runner::start(paths, &binary, mode).await?;
    wait_until_ready(app, pid).await?;
    Ok(pid)
}

async fn wait_until_ready(app: &AppHandle, pid: u32) -> Result<()> {
    let state = app.state::<AppState>();
    for _ in 0..READY_ATTEMPTS {
        if !runner::is_alive(pid) {
            let reason = state.logs.last_error().unwrap_or_else(|| "sing-box exited unexpectedly".into());
            return Err(AppError::Launch(reason));
        }
        if net::proxy_port_open().await {
            return Ok(());
        }
        sleep(POLL_INTERVAL).await;
    }
    Err(AppError::Launch("sing-box did not start in time".into()))
}

fn watch(app: AppHandle, pid: u32) {
    tauri::async_runtime::spawn(async move {
        while runner::is_alive(pid) {
            sleep(Duration::from_secs(1)).await;
        }
        finish(&app, pid);
    });
}

fn finish(app: &AppHandle, pid: u32) {
    let state = app.state::<AppState>();
    {
        let mut current = state.pid.lock().unwrap();
        if *current != Some(pid) {
            return;
        }
        *current = None;
    }

    let stopped_by_user = state.paths.stop_file().exists() || !state.paths.log_file().exists();
    let error = if stopped_by_user { None } else { state.logs.last_error() };
    let mode = state.store.lock().unwrap().mode();
    update(app, |s| *s = Status { error, ..Status::disconnected(mode) });
    traffic::reset(app);
}

fn update(app: &AppHandle, change: impl FnOnce(&mut Status)) {
    let state = app.state::<AppState>();
    let mut status = state.status.lock().unwrap();
    change(&mut status);
    let _ = app.emit("status", status.clone());
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}
