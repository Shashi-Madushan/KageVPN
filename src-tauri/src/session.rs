use std::sync::atomic::Ordering;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
/// A port still busy within this long of the app stopping sing-box is most likely that same
/// process finishing up, so the next connect waits for it instead of failing.
const PORT_WAIT_WINDOW: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_millis(250);
const DROP_GRACE: Duration = Duration::from_millis(1500);

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
    /// The port of the local proxy while connected; people point apps at it in Proxy Only mode.
    #[serde(default)]
    pub proxy_port: Option<u16>,
    /// The kill switch is blocking traffic outside the tunnel (while reconnecting, or when the
    /// tunnel is down). Set from the root helper's marker; see `reliability::spawn_block_watch`.
    #[serde(default)]
    pub blocked: bool,
}

impl Status {
    pub fn disconnected(mode: Mode) -> Self {
        Self { state: State::Disconnected, mode, since: None, error: None, health: Health::Idle, reason: None, latency_ms: None, failure: None, proxy_port: None, blocked: false }
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
                s.proxy_port = Some(config::mixed_port());
            });
            applog::info(format!("connect: up, supervisor pid {pid}"));
            watch(app.clone(), pid);
            traffic::spawn(app.clone(), pid, secret);
            spawn_connection_poll(app.clone(), pid);
            health::spawn_monitor(app.clone(), pid);
            if mode == Mode::Tun && state.store.lock().unwrap().reliability().kill_switch {
                spawn_allow_list_refresh(app.clone(), pid);
            }
            // Cancelled in the last moments, after the final checkpoint: take it down the usual way.
            if state.cancel_connect.swap(false, Ordering::SeqCst) {
                let _ = disconnect(app).await;
                return Err(AppError::Cancelled);
            }
            Ok(())
        }
        Err(error) if error.is_cancelled() => {
            applog::info("connect: cancelled by the user");
            *state.clash_secret.lock().unwrap() = None;
            note_stop(&state);
            let _ = runner::request_stop(&state.paths);
            update(app, |s| *s = Status::disconnected(mode));
            Err(error)
        }
        Err(error) => {
            applog::warn(format!("connect: failed: {error}"));
            *state.clash_secret.lock().unwrap() = None;
            note_stop(&state);
            let _ = end_run(app, mode, Ending::Failed);
            update(app, |s| *s = Status { error: Some(error.to_string()), failure: Some(error.info()), ..Status::disconnected(mode) });
            Err(error)
        }
    }
}

/// What a request to disconnect should do right now.
#[derive(Debug, PartialEq, Eq)]
enum Plan {
    /// The connection is still starting: tell it to stop at its next step.
    Cancel,
    /// Stop the running connection.
    Stop(u32),
    Nothing,
}

fn disconnect_plan(state: State, pid: Option<u32>) -> Plan {
    match (state, pid) {
        (State::Connecting, _) => Plan::Cancel,
        (_, Some(pid)) => Plan::Stop(pid),
        (_, None) => Plan::Nothing,
    }
}

/// Notes that sing-box was just asked to stop, so the next connect can wait for its port.
fn note_stop(state: &AppState) {
    *state.stop_requested_at.lock().unwrap() = Some(Instant::now());
}

/// Ends the connection if the user cancelled it; called between the steps of starting one.
fn check_cancelled(app: &AppHandle) -> Result<()> {
    if app.state::<AppState>().cancel_connect.load(Ordering::SeqCst) {
        return Err(AppError::Cancelled);
    }
    Ok(())
}

/// Passes the tunnel's health on to the kill switch: a verified tunnel takes down a block that
/// was held while it came up, a dead one puts the block up even though sing-box is still running.
pub fn signal_tunnel(app: &AppHandle, healthy: bool) {
    let state = app.state::<AppState>();
    let kill_switch = state.store.lock().unwrap().reliability().kill_switch;
    let mode = state.status.lock().unwrap().mode;
    if !kill_switch || mode != Mode::Tun {
        return;
    }
    let _ = if healthy { runner::signal_ready(&state.paths) } else { runner::signal_block(&state.paths) };
}

/// Keeps the kill switch's list of server addresses fresh while connected, so a server name that
/// starts resolving elsewhere is already allowed when the block has to go up.
fn spawn_allow_list_refresh(app: AppHandle, pid: u32) {
    tauri::async_runtime::spawn(async move {
        loop {
            sleep(crate::allowlist::REFRESH_EVERY).await;
            let state = app.state::<AppState>();
            if *state.pid.lock().unwrap() != Some(pid) {
                return;
            }
            crate::allowlist::refresh(&state.paths, None, !state.blocked.load(Ordering::SeqCst)).await;
        }
    });
}

/// Why a run is being ended, which decides whether the kill-switch block stays up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    /// The user disconnected, cancelled or quit: traffic is released.
    UserStop,
    /// A connect attempt failed.
    Failed,
    /// The tunnel is being restarted because it died or stopped carrying traffic.
    Restart,
}

/// Whether ending a run should keep the kill-switch block up. A failed first connect releases
/// it, so a wrong password or a bad config never cuts the user off; a failed *reconnect* and a
/// restart keep it, because the user was connected a moment ago and expects no leaks.
pub fn holds_block(ending: Ending, kill_switch: bool, mode: Mode, reconnecting: bool) -> bool {
    kill_switch
        && mode == Mode::Tun
        && match ending {
            Ending::UserStop => false,
            Ending::Failed => reconnecting,
            Ending::Restart => true,
        }
}

fn end_run(app: &AppHandle, mode: Mode, ending: Ending) -> Result<()> {
    let state = app.state::<AppState>();
    let kill_switch = state.store.lock().unwrap().reliability().kill_switch;
    if holds_block(ending, kill_switch, mode, state.reconnecting.load(Ordering::SeqCst)) {
        runner::request_down(&state.paths)
    } else {
        runner::request_stop(&state.paths)
    }
}

pub async fn disconnect(app: &AppHandle) -> Result<()> {
    stop_with(app, Ending::UserStop).await
}

/// Ends the connection for a restart: the kill-switch block, if on, stays up until the next run.
pub async fn disconnect_for_restart(app: &AppHandle) -> Result<()> {
    stop_with(app, Ending::Restart).await
}

async fn stop_with(app: &AppHandle, ending: Ending) -> Result<()> {
    let state = app.state::<AppState>();
    let connection = state.status.lock().unwrap().state;
    let pid = match disconnect_plan(connection, *state.pid.lock().unwrap()) {
        Plan::Nothing => return Ok(()),
        Plan::Cancel => {
            applog::info("disconnect: cancelling a connection that is still starting");
            state.cancel_connect.store(true, Ordering::SeqCst);
            return Ok(());
        }
        Plan::Stop(pid) => pid,
    };

    applog::info(format!("disconnect: stopping supervisor pid {pid}"));
    state.stopping.store(true, Ordering::SeqCst);
    note_stop(&state);
    let mode = state.status.lock().unwrap().mode;
    end_run(app, mode, ending)?;
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
    // With a reconnect coming, the kill-switch block stays up in between.
    let _ = if reconnect { disconnect_for_restart(app).await } else { disconnect(app).await };
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
    state.cancel_connect.store(false, Ordering::SeqCst);
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
    check_cancelled(app)?;
    // A sing-box this app stopped a moment ago can take a few seconds to let go of its port, so
    // that one is waited for. Any other program holding a port is moved around.
    let just_stopped = state.stop_requested_at.lock().unwrap().is_some_and(|at| at.elapsed() < PORT_WAIT_WINDOW);
    let ports = net::choose_ports(just_stopped).await?;
    if !ports.is_default() {
        applog::warn(format!("default ports are busy: using proxy port {} and controller port {}", ports.mixed, ports.clash));
        notify::send(app, "Using another local port", &format!("Port {} was busy, so the local proxy listens on {}.", config::DEFAULT_MIXED_PORT, ports.mixed));
    }
    config::set_ports(ports.mixed, ports.clash);
    // With the kill switch holding a block, DNS is blocked too: names cannot be looked up and the
    // downloads would only time out, so those steps use what is already known and on disk.
    let behind_block = state.blocked.load(Ordering::SeqCst);
    if !state.store.lock().unwrap().auto().enabled {
        let active = state.store.lock().unwrap().active_profile().cloned();
        if let Some(profile) = active {
            health::preflight(&profile, behind_block).await?;
        }
    }
    check_cancelled(app)?;
    state.logs.clear();
    paths.reset_runtime_files()?;

    let (profiles, active_id, auto_group, auto, routing, reliability, log_level) = {
        let store = state.store.lock().unwrap();
        let auto = store.auto();
        (
            store.profiles().to_vec(),
            store.active_id().map(str::to_string),
            auto.group.clone(),
            auto.enabled,
            store.routing(),
            store.reliability(),
            store.settings().log_level,
        )
    };
    let proxy_password = Some(reliability.proxy_password.clone()).filter(|p| reliability.proxy_auth && !p.is_empty());
    config::set_proxy_password(proxy_password.clone());
    if behind_block {
        applog::info("connect: the kill switch is blocking, so rule lists are not downloaded now");
    } else {
        prepare_rule_sets(app, &routing).await;
    }
    check_cancelled(app)?;

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
        log_level,
        mixed_port: ports.mixed,
        clash_port: ports.clash,
        rule_sets_dir: &paths.rule_sets_dir(),
        secret,
        proxy_password: proxy_password.as_deref(),
    })?;
    crate::fsutil::write_private_atomic(&paths.config_file(), &serde_json::to_vec_pretty(&config)?)?;
    crate::singbox::check(&binary, &paths.config_file())?;

    let kill_switch_ips = if reliability.kill_switch && mode == Mode::Tun {
        let targets = nodes.iter().map(|p| (p.server.clone(), p.port)).collect();
        Some(crate::allowlist::refresh(paths, Some(targets), !behind_block).await)
    } else {
        None
    };
    check_cancelled(app)?;
    let pid = runner::start(paths, &binary, mode, kill_switch_ips).await?;
    wait_until_ready(app, pid).await?;

    if !auto {
        if let Some(id) = &active_id {
            let _ = clash::select_profile(app, id).await;
        }
    }
    check_cancelled(app)?;
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

async fn wait_until_ready(app: &AppHandle, pid: u32) -> Result<()> {
    let state = app.state::<AppState>();
    for _ in 0..READY_ATTEMPTS {
        check_cancelled(app)?;
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
    if state.store.lock().unwrap().settings().clear_logs {
        state.logs.clear();
        crate::logs::erase_files(&state.paths.log_file());
        applog::info("clear logs on disconnect: sing-box log files emptied");
    }

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
    status.blocked = state.blocked.load(Ordering::SeqCst);
    let _ = app.emit("status", status.clone());
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[cfg(test)]
#[path = "tests/session.rs"]
mod tests;
