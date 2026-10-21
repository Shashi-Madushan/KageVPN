//! Whether a connected tunnel actually carries traffic.
//!
//! sing-box opens its local port as soon as it starts, whether or not the server works, so
//! "connected" alone proves nothing. After connecting, a staged probe checks the active node and
//! then a real request through the tunnel, and says which stage failed and why.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;
use tokio::time::sleep;

use crate::error::AppError;
use crate::profile::Profile;
use crate::state::AppState;
use crate::{applog, clash, net, session};

/// The tag of the outbound group that carries user traffic (see `config::build`).
const GROUP_TAG: &str = "proxy";
const VERIFY_ATTEMPTS: u32 = 6;
const VERIFY_PAUSE: Duration = Duration::from_millis(1500);
const HEALTHY_EVERY: Duration = Duration::from_secs(15);
const CONFIRM_AFTER: Duration = Duration::from_secs(3);
const DOWN_EVERY_MAX: Duration = Duration::from_secs(60);
const FAILURES_BEFORE_DOWN: u32 = 2;
const FAILURES_BEFORE_RESTART: u32 = 4;

static RECHECK: Notify = Notify::const_new();

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Health {
    /// Not connected, so there is nothing to judge.
    #[default]
    Idle,
    /// Connected; the first checks have not finished.
    Verifying,
    Healthy,
    /// The checks failed, so the tunnel is up but traffic is not getting through.
    Down,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Cause {
    /// The server did not answer at all (offline, blocked, wrong address or port).
    ServerUnreachable,
    /// The server answered but refused the session (credentials, TLS, protocol settings).
    Handshake,
    /// The server is fine but no traffic passes through it.
    NoTraffic,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Reason {
    pub cause: Cause,
    pub message: String,
}

/// The outcome of one probe round.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Verdict {
    Healthy { latency_ms: u64 },
    Down(Reason),
}

/// Decides what a probe round means. `node` is the delay of the active outbound measured by
/// sing-box itself; `tunnel` is a request made through the local proxy; `log_error` is the most
/// recent sing-box error line, used to tell an unreachable server from a rejecting one.
pub fn classify(node: &Result<u64, String>, tunnel: &Result<u64, String>, log_error: Option<&str>) -> Verdict {
    if let Ok(latency_ms) = tunnel {
        return Verdict::Healthy { latency_ms: *latency_ms };
    }
    if node.is_ok() {
        return Verdict::Down(Reason {
            cause: Cause::NoTraffic,
            message: "The server answers, but no traffic passes through it. Check its DNS and routing.".into(),
        });
    }
    let cause = log_error.map_or(Cause::ServerUnreachable, cause_from_log);
    let message = match cause {
        Cause::Handshake => "The server rejected the connection. Check the credentials, TLS and protocol settings.",
        _ => "The server did not answer. It may be offline, blocked, or the address or port is wrong.",
    };
    Verdict::Down(Reason { cause, message: message.into() })
}

/// Reads a sing-box error line: handshake wording means the server was reached but said no.
pub fn cause_from_log(line: &str) -> Cause {
    let line = line.to_lowercase();
    const REJECTED: [&str; 8] =
        ["tls:", "handshake", "authentication", "unauthorized", "reality", "invalid user", "bad certificate", "forbidden"];
    if REJECTED.iter().any(|marker| line.contains(marker)) {
        Cause::Handshake
    } else {
        Cause::ServerUnreachable
    }
}

/// One probe round against the running tunnel.
pub async fn probe(app: &AppHandle) -> Verdict {
    let node = clash::delay_by_tag(app, GROUP_TAG).await.map_err(|e| e.to_string());
    let tunnel = net::proxy_delay().await.map_err(|e| e.to_string());
    let log_error = app.state::<AppState>().logs.last_error();
    classify(&node, &tunnel, log_error.as_deref())
}

/// UDP-based protocols cannot be judged by a TCP connect.
pub fn needs_tcp_check(protocol: &str) -> bool {
    crate::profile::ProtocolId::lookup(protocol).is_none_or(|id| !crate::profile::def(id).over_udp())
}

/// Cheap checks before sing-box starts, so an obviously unusable server fails in seconds with a
/// plain explanation instead of connecting and then showing no internet.
///
/// `behind_block`: the kill switch is blocking DNS, so the server's remembered addresses are
/// checked instead of looking its name up.
pub async fn preflight(profile: &Profile, behind_block: bool) -> crate::error::Result<()> {
    if behind_block {
        let known = crate::allowlist::known(&profile.server);
        if !known.is_empty() {
            if !needs_tcp_check(profile.protocol.as_str()) {
                return Ok(());
            }
            for address in &known {
                if net::tcp_latency(address, profile.port).await.is_ok() {
                    return Ok(());
                }
            }
            return Err(AppError::ServerUnreachable(format!(
                "{} did not accept a connection on port {}. It may be offline, or the internet is not back yet.",
                profile.server, profile.port
            )));
        }
    }
    let target = (profile.server.as_str(), profile.port);
    let resolved = tokio::net::lookup_host(target).await.map(|mut a| a.next().is_some()).unwrap_or(false);
    if !resolved {
        return Err(AppError::ServerUnreachable(format!(
            "The address '{}' does not resolve. Check the server name and your own internet connection.",
            profile.server
        )));
    }
    if needs_tcp_check(profile.protocol.as_str()) {
        if let Err(error) = net::tcp_latency(&profile.server, profile.port).await {
            return Err(AppError::ServerUnreachable(format!(
                "{} did not accept a connection on port {} ({error}). It may be offline or blocked.",
                profile.server, profile.port
            )));
        }
    }
    Ok(())
}

/// Wakes the monitor early, for example after the Mac woke or the network changed.
pub fn recheck() {
    RECHECK.notify_one();
}

/// Watches one connection until it ends: first verifies it, then keeps checking cheaply.
///
/// A healthy tunnel costs one small request every `HEALTHY_EVERY`. Failures are re-checked
/// quickly to confirm, then the full staged probe explains them; while down the pace backs off.
/// With "restart when down" on, a tunnel that stays down is restarted.
pub fn spawn_monitor(app: AppHandle, pid: u32) {
    tauri::async_runtime::spawn(async move {
        if !verify_first(&app, pid).await {
            return;
        }
        let mut failures = 0u32;
        loop {
            let wait = match failures {
                0 => HEALTHY_EVERY,
                1 => CONFIRM_AFTER,
                n => (CONFIRM_AFTER * 2u32.pow((n - 1).min(4))).min(DOWN_EVERY_MAX),
            };
            tokio::select! {
                _ = sleep(wait) => {}
                _ = RECHECK.notified() => {}
            }
            if !is_current(&app, pid) {
                return;
            }
            match net::proxy_delay().await {
                Ok(latency_ms) => {
                    let recovered = failures > 0;
                    failures = 0;
                    apply(&app, &Verdict::Healthy { latency_ms });
                    if recovered {
                        session::signal_tunnel(&app, true);
                    }
                }
                Err(error) => {
                    failures += 1;
                    applog::warn(format!("health probe failed ({failures}): {error}"));
                    if failures >= FAILURES_BEFORE_DOWN {
                        let verdict = probe(&app).await;
                        if !is_current(&app, pid) {
                            return;
                        }
                        apply(&app, &verdict);
                        if matches!(verdict, Verdict::Down(_)) {
                            session::signal_tunnel(&app, false);
                        }
                    }
                    let restart = app.state::<AppState>().store.lock().unwrap().reliability().watchdog;
                    if restart && failures >= FAILURES_BEFORE_RESTART {
                        session::tunnel_lost(&app, pid).await;
                        return;
                    }
                }
            }
        }
    });
}

fn is_current(app: &AppHandle, pid: u32) -> bool {
    *app.state::<AppState>().pid.lock().unwrap() == Some(pid)
}

/// Checks a fresh connection until it proves healthy or the attempts run out.
/// Returns false when the connection ended meanwhile.
async fn verify_first(app: &AppHandle, pid: u32) -> bool {
    let mut verdict = Verdict::Down(Reason { cause: Cause::NoTraffic, message: String::new() });
    for attempt in 0..VERIFY_ATTEMPTS {
        if attempt > 0 {
            sleep(VERIFY_PAUSE).await;
        }
        if !is_current(app, pid) {
            return false;
        }
        verdict = probe(app).await;
        if matches!(verdict, Verdict::Healthy { .. }) {
            break;
        }
    }
    if !is_current(app, pid) {
        return false;
    }
    apply(app, &verdict);
    if matches!(verdict, Verdict::Healthy { .. }) {
        session::signal_tunnel(app, true);
        mark_engine_good(app);
        session::spawn_rule_set_refresh(app.clone(), pid);
    }
    true
}

/// A verified tunnel proves the engine in use, so a rollback can return to it later.
fn mark_engine_good(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut store = state.store.lock().unwrap();
    let mut settings = store.core_settings();
    if settings.mark_good() {
        let _ = store.set_core_settings(settings);
    }
}

/// Publishes a verdict in the connection status.
pub fn apply(app: &AppHandle, verdict: &Verdict) {
    let current = app.state::<AppState>().status.lock().unwrap().clone();
    let unchanged = match verdict {
        Verdict::Healthy { .. } => current.health == Health::Healthy,
        Verdict::Down(reason) => current.health == Health::Down && current.reason.as_ref() == Some(reason),
    };
    if unchanged {
        return;
    }
    match verdict {
        Verdict::Healthy { latency_ms } => {
            applog::info(format!("health: tunnel verified ({latency_ms} ms)"));
            session::update(app, |s| {
                s.health = Health::Healthy;
                s.reason = None;
                s.latency_ms = Some(*latency_ms);
            });
        }
        Verdict::Down(reason) => {
            applog::warn(format!("health: tunnel is up but traffic does not flow ({:?}): {}", reason.cause, reason.message));
            session::update(app, |s| {
                s.health = Health::Down;
                s.reason = Some(reason.clone());
                s.latency_ms = None;
            });
        }
    }
}

#[cfg(test)]
#[path = "tests/health.rs"]
mod tests;
