use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::sleep;

use crate::config::{self, ConfigInput, Mode};
use crate::error::{AppError, Result};
use crate::profile::Profile;
use crate::state::AppState;
use crate::usage::collector::spawn_connection_poll;
use crate::{applog, clash, core, net, notify, reliability, runner, traffic};

const READY_ATTEMPTS: u32 = 60;
const STOP_ATTEMPTS: u32 = 40;
const POLL_INTERVAL: Duration = Duration::from_millis(250);
const DROP_GRACE: Duration = Duration::from_millis(1500);
const MAX_RESOLVED_IPS: usize = 64;

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
    let mode = {
        let store = state.store.lock().unwrap();
        if store.profiles().is_empty() {
            return Err(AppError::Invalid("add a profile first".into()));
        }
        store.mode()
    };

    begin(app, mode)?;
    applog::info(format!("connect: starting ({mode:?})"));
    let secret = uuid::Uuid::new_v4().to_string();
    *state.clash_secret.lock().unwrap() = Some(secret.clone());

    match launch(app, mode, &secret).await {
        Ok(pid) => {
            *state.pid.lock().unwrap() = Some(pid);
            update(app, |s| {
                s.state = State::Connected;
                s.since = Some(now_secs());
            });
            applog::info(format!("connect: up, supervisor pid {pid}"));
            watch(app.clone(), pid);
            traffic::spawn(app.clone(), pid, secret);
            spawn_connection_poll(app.clone(), pid);
            reliability::on_connected(app, pid);
            Ok(())
        }
        Err(error) => {
            applog::warn(format!("connect: failed: {error}"));
            *state.clash_secret.lock().unwrap() = None;
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

    applog::info(format!("disconnect: stopping supervisor pid {pid}"));
    state.stopping.store(true, Ordering::SeqCst);
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

/// Called when sing-box stopped answering while its supervisor is still alive.
pub async fn tunnel_lost(app: &AppHandle, pid: u32) {
    sleep(DROP_GRACE).await;
    let state = app.state::<AppState>();
    if *state.pid.lock().unwrap() != Some(pid) || state.stopping.load(Ordering::SeqCst) {
        return;
    }
    let reconnect = state.store.lock().unwrap().reliability().auto_reconnect;
    applog::warn(format!("tunnel lost (pid {pid}): disconnecting, auto-reconnect {reconnect}"));
    notify::send(app, "VPN tunnel dropped", if reconnect { "Reconnecting…" } else { "The tunnel stopped." });
    let _ = disconnect(app).await;
    if reconnect {
        reliability::spawn_reconnect(app.clone());
    }
}

fn begin(app: &AppHandle, mode: Mode) -> Result<()> {
    let state = app.state::<AppState>();
    let mut status = state.status.lock().unwrap();
    if status.state != State::Disconnected {
        return Err(AppError::Busy);
    }
    state.stopping.store(false, Ordering::SeqCst);
    *status = Status { state: State::Connecting, ..Status::disconnected(mode) };
    let _ = app.emit("status", status.clone());
    Ok(())
}

async fn launch(app: &AppHandle, mode: Mode, secret: &str) -> Result<u32> {
    let state = app.state::<AppState>();
    let paths = &state.paths;

    let (spec, core_settings) = {
        let mut store = state.store.lock().unwrap();
        let spec = store.active_core();
        let mut settings = store.core_settings();
        if core::apply_pending(&mut settings, paths, spec) {
            let _ = store.set_core_settings(settings.clone());
        }
        (spec, settings)
    };
    let (binary, _) = core::resolve(paths, spec, &core_settings)?;
    if net::proxy_port_open().await {
        return Err(AppError::Launch(format!(
            "port {} is already in use, is another sing-box running?",
            config::MIXED_PORT
        )));
    }
    state.logs.clear();
    paths.reset_runtime_files()?;

    let (profiles, active_id, auto_group, auto, routing, reliability) = {
        let store = state.store.lock().unwrap();
        let auto = store.auto();
        (
            store.profiles().to_vec(),
            store.active_id().map(str::to_string),
            auto.group.clone(),
            auto.enabled,
            store.routing(),
            store.reliability(),
        )
    };
    let candidates: Vec<Profile> = match (&auto_group, auto) {
        (Some(group), true) => profiles.iter().filter(|p| p.group.as_deref() == Some(group.as_str())).cloned().collect(),
        _ => profiles.clone(),
    };
    let nodes = if candidates.is_empty() { &profiles } else { &candidates };

    let config = config::build(&ConfigInput {
        profiles: nodes,
        active_id: active_id.as_deref(),
        auto,
        mode,
        routing: &routing,
        log_path: &paths.log_file(),
        cache_path: &paths.cache_file(),
        secret,
    })?;
    crate::fsutil::write_private_atomic(&paths.config_file(), &serde_json::to_vec_pretty(&config)?)?;
    crate::singbox::check(&binary, &paths.config_file())?;

    let kill_switch_ips = if reliability.kill_switch && mode == Mode::Tun {
        Some(resolve_servers(nodes).await)
    } else {
        None
    };
    let pid = runner::start(paths, &binary, mode, kill_switch_ips).await?;
    wait_until_ready(app, pid).await?;

    if !auto {
        if let Some(id) = &active_id {
            let _ = clash::select_profile(app, id).await;
        }
    }
    Ok(pid)
}

async fn resolve_servers(profiles: &[Profile]) -> String {
    let mut ips: Vec<String> = Vec::new();
    for profile in profiles {
        if let Ok(addresses) = tokio::net::lookup_host((profile.server.as_str(), profile.port)).await {
            ips.extend(addresses.map(|a| a.ip().to_string()));
        }
        if ips.len() >= MAX_RESOLVED_IPS {
            break;
        }
    }
    ips.sort();
    ips.dedup();
    ips.join(",")
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
    *state.clash_secret.lock().unwrap() = None;

    let stopped_by_user = state.stopping.load(Ordering::SeqCst);
    let error = if stopped_by_user { None } else { state.logs.last_error() };
    applog::info(format!(
        "supervisor pid {pid} gone ({}){}",
        if stopped_by_user { "stopped by app" } else { "UNEXPECTED" },
        error.as_deref().map(|e| format!("; last sing-box error: {e}")).unwrap_or_default()
    ));
    if !stopped_by_user {
        applog::warn(format!("sing-box stderr tail:\n{}", tail_of(&state.paths.stderr_log(), 15)));
    }
    let mode = state.store.lock().unwrap().mode();
    update(app, |s| *s = Status { error, ..Status::disconnected(mode) });
    traffic::reset(app);

    if !stopped_by_user {
        notify::send(app, "VPN connection lost", "sing-box stopped unexpectedly.");
        reliability::on_unexpected_exit(app);
    }
}

/// The last `count` lines of a text file, or a note when it cannot be read.
fn tail_of(path: &std::path::Path, count: usize) -> String {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let lines: Vec<&str> = text.lines().collect();
            lines[lines.len().saturating_sub(count)..].join("\n")
        }
        Err(_) => "(no stderr log)".into(),
    }
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
