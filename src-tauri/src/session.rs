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
use crate::error::ErrorInfo;
use crate::health::{self, Health, Reason};
use crate::{applog, clash, core, crashlog, net, notify, reliability, runner, traffic};

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
    /// Whether the tunnel carries traffic; only meaningful while connected.
    #[serde(default)]
    pub health: Health,
    #[serde(default)]
    pub reason: Option<Reason>,
    #[serde(default)]
    pub latency_ms: Option<u64>,
    /// Why the last attempt to connect failed, described for people.
    #[serde(default)]
    pub failure: Option<ErrorInfo>,
}

impl Status {
    pub fn disconnected(mode: Mode) -> Self {
        Self { state: State::Disconnected, mode, since: None, error: None, health: Health::Idle, reason: None, latency_ms: None, failure: None }
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
                s.health = Health::Verifying;
            });
            applog::info(format!("connect: up, supervisor pid {pid}"));
            watch(app.clone(), pid);
            traffic::spawn(app.clone(), pid, secret);
            spawn_connection_poll(app.clone(), pid);
            health::spawn_monitor(app.clone(), pid);
            Ok(())
        }
        Err(error) => {
            applog::warn(format!("connect: failed: {error}"));
            *state.clash_secret.lock().unwrap() = None;
            let _ = runner::request_stop(&state.paths);
            update(app, |s| *s = Status { error: Some(error.to_string()), failure: Some(error.info()), ..Status::disconnected(mode) });
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
    let reason = "The tunnel stopped answering while sing-box was still running.";
    crashlog::record(&state.paths.data_dir(), "tunnel lost", reason, &format!("auto-reconnect: {reconnect}\nlast error: {}", state.logs.last_error().unwrap_or_default()));
    notify::send(app, "VPN tunnel dropped", &format!("{reason} {}", if reconnect { "Reconnecting…" } else { "It was disconnected." }));
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
    if !state.store.lock().unwrap().auto().enabled {
        let active = state.store.lock().unwrap().active_profile().cloned();
        if let Some(profile) = active {
            health::preflight(&profile).await?;
        }
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
    prepare_rule_sets(app, &routing).await;

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
        rule_sets_dir: &paths.rule_sets_dir(),
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

/// Makes sure the country and category lists the routing uses are on disk. A list that is missing
/// is fetched now (a connection cannot use it otherwise); one that is merely old is refreshed in
/// the background. A list that cannot be fetched is left out and the user is told, but it never
/// stops the connection.
async fn prepare_rule_sets(app: &AppHandle, routing: &crate::settings::RoutingSettings) {
    let dir = app.state::<AppState>().paths.rule_sets_dir();
    let (missing, stale) = crate::rulesets::plan(&dir, &crate::rulesets::wanted(routing), crate::rulesets::MAX_AGE);
    // A list that just failed is not waited for again: reconnect attempts follow each other fast.
    let missing = crate::rulesets::skip_recent_failures(missing);
    if !missing.is_empty() {
        applog::info(format!("downloading {} rule list(s) before connecting", missing.len()));
        let failed = crate::rulesets::fetch_all(&dir, missing).await;
        for (item, error) in &failed {
            applog::warn(format!("rule list {} left out: {error}", item.tag));
        }
        if !failed.is_empty() {
            let tags: Vec<String> = failed.into_iter().map(|(item, _)| item.tag).collect();
            notify::send(app, "Some lists were left out", &crate::rulesets::skipped_message(&tags));
        }
    }
    if !stale.is_empty() {
        tauri::async_runtime::spawn(async move {
            let _ = crate::rulesets::fetch_all(&dir, stale).await;
        });
    }
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
    let mut error = if stopped_by_user { None } else { state.logs.last_error() };
    let mut failure = None;
    applog::info(format!(
        "supervisor pid {pid} gone ({}){}",
        if stopped_by_user { "stopped by app" } else { "UNEXPECTED" },
        error.as_deref().map(|e| format!("; last sing-box error: {e}")).unwrap_or_default()
    ));
    let mut reason = String::new();
    if !stopped_by_user {
        let stderr = tail_of(&state.paths.stderr_log(), 15);
        applog::warn(format!("sing-box stderr tail:\n{stderr}"));
        reason = crashlog::summary(crashlog::exit_status(&stderr).as_deref(), error.as_deref());
        let recent: Vec<String> = state.logs.snapshot().iter().rev().filter(|l| l.level == crate::logs::Level::Error).take(5).map(|l| l.text.clone()).collect();
        crashlog::record(&state.paths.data_dir(), "sing-box stopped", &reason, &format!(
                "Last errors:\n{}\n\nstderr:\n{stderr}\n\nLast sing-box log lines:\n{}",
                recent.join("\n"),
                tail_of(&state.paths.log_file(), 40)
            ),
        );
        error = Some(reason.clone());
        failure = Some(ErrorInfo {
            code: "engineStopped".into(),
            title: "sing-box stopped unexpectedly".into(),
            detail: format!("{reason}. Details were saved to crashes.log."),
            action: Some("retry".into()),
        });
    }
    let mode = state.store.lock().unwrap().mode();
    update(app, |s| *s = Status { error, failure, ..Status::disconnected(mode) });
    traffic::reset(app);

    if !stopped_by_user {
        notify::send(app, "VPN stopped unexpectedly", &reason);
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

pub(crate) fn update(app: &AppHandle, change: impl FnOnce(&mut Status)) {
    let state = app.state::<AppState>();
    let mut status = state.status.lock().unwrap();
    change(&mut status);
    let _ = app.emit("status", status.clone());
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}
