use std::sync::atomic::Ordering;
use std::time::Duration;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tokio::time::sleep;

use crate::error::{AppError, Result};
use crate::config::Mode;
use crate::session::State as Conn;
use crate::state::AppState;
use crate::{applog, notify, session};

const RECONNECT_DELAYS: [u64; 4] = [2, 5, 15, 30];
const HOTKEY: &str = "CommandOrControl+Alt+V";
const PF_ANCHOR: &str = "com.apple/250.KageVPN";
/// Anchor used before the rename; still flushed so an old kill switch can never strand the network.
const LEGACY_PF_ANCHORS: [&str; 3] = ["com.apple/250.ApzurVPN", "com.apple/250.OspreyVPN", "com.apple/250.SingBoxGUI"];

pub fn on_unexpected_exit(app: &AppHandle) {
    if app.state::<AppState>().store.lock().unwrap().reliability().auto_reconnect {
        spawn_reconnect(app.clone());
    }
}

/// Whether trying again could change the outcome. A cancelled password prompt, a missing engine or
/// a rejected configuration will fail the same way every time, and retrying the first would put
/// the macOS password dialog back on the screen over and over.
pub fn worth_retrying(error: &AppError) -> bool {
    !matches!(
        error,
        AppError::PermissionDenied
            | AppError::SingBoxMissing
            | AppError::ConfigCheck(_)
            | AppError::Invalid(_)
            | AppError::ProfileNotFound
            | AppError::Cancelled
    )
}

/// Whether a reconnect that gave up earlier should start again now that the network changed.
pub fn should_resume(waiting: bool, state: Conn, auto_reconnect: bool, loop_running: bool) -> bool {
    waiting && state == Conn::Disconnected && auto_reconnect && !loop_running
}

/// How a kill-switch block that is no longer wanted is lifted.
#[derive(Debug, PartialEq, Eq)]
pub enum Lift {
    /// Nothing is running: the helper is holding the block between runs and reads the stop file.
    Stop,
    /// A run is up or starting: the helper reads the ready file while sing-box runs, and a run that
    /// starts without the kill switch drops the block on its own.
    Ready,
}

/// Whether a block that is up should be lifted now. The user can turn the kill switch off while it
/// is blocking (or while a run that has it is still going, which blocks if sing-box dies), and
/// nothing else would take that block down.
pub fn unwanted_block(blocked: bool, kill_switch: bool, state: Conn) -> Option<Lift> {
    match (blocked && !kill_switch, state) {
        (false, _) => None,
        (true, Conn::Disconnected) => Some(Lift::Stop),
        (true, _) => Some(Lift::Ready),
    }
}

/// Whether turning the kill switch on only takes effect at the next connect. The helper of a TUN
/// run that started without it has no server list to let through, so it cannot block.
pub fn kill_switch_waits_for_reconnect(was: bool, now: bool, state: Conn, mode: Mode) -> bool {
    !was && now && state != Conn::Disconnected && mode == Mode::Tun
}

/// Records that the user connected or disconnected by hand. A reconnect in flight stands down, and
/// nothing resumes on its own afterwards.
pub fn user_took_over(app: &AppHandle) {
    let state = app.state::<AppState>();
    state.manual_actions.fetch_add(1, Ordering::SeqCst);
    state.reconnect_waiting.store(false, Ordering::SeqCst);
}

/// True (once) when the network came back after a reconnect gave up and a new attempt is due.
pub fn take_resume(app: &AppHandle) -> bool {
    let state = app.state::<AppState>();
    let connection = state.status.lock().unwrap().state;
    let auto = state.store.lock().unwrap().reliability().auto_reconnect;
    let due = should_resume(state.reconnect_waiting.load(Ordering::SeqCst), connection, auto, state.reconnecting.load(Ordering::SeqCst));
    if due {
        state.reconnect_waiting.store(false, Ordering::SeqCst);
    }
    due
}

enum Outcome {
    Reconnected,
    /// The user connected or disconnected by hand meanwhile, so this loop has nothing left to do.
    UserTookOver,
    /// An error that retrying cannot fix.
    Failed(AppError),
    /// Every attempt failed, most likely because the network or the server is down.
    GaveUp,
}

pub fn spawn_reconnect(app: AppHandle) {
    let state = app.state::<AppState>();
    if state.reconnecting.swap(true, Ordering::SeqCst) {
        return;
    }
    state.reconnect_waiting.store(false, Ordering::SeqCst);
    let started_under = state.manual_actions.load(Ordering::SeqCst);
    tauri::async_runtime::spawn(async move {
        let taken_over = |app: &AppHandle| app.state::<AppState>().manual_actions.load(Ordering::SeqCst) != started_under;
        let mut outcome = Outcome::GaveUp;
        for delay in RECONNECT_DELAYS {
            sleep(Duration::from_secs(delay)).await;
            if taken_over(&app) {
                outcome = Outcome::UserTookOver;
                break;
            }
            let result = session::connect(&app).await;
            applog::info(format!("reconnect after {delay}s: {}", result.as_ref().map_or_else(|e| e.to_string(), |_| "ok".into())));
            match result {
                // Busy means a connection is already starting or up, which is what we wanted.
                Ok(()) | Err(AppError::Busy) => {
                    outcome = Outcome::Reconnected;
                    break;
                }
                // The user cancelled it by hand, so there is nothing left for this loop to do.
                Err(AppError::Cancelled) => {
                    outcome = Outcome::UserTookOver;
                    break;
                }
                Err(error) if !worth_retrying(&error) => {
                    outcome = Outcome::Failed(error);
                    break;
                }
                Err(_) => {}
            }
        }
        if taken_over(&app) {
            outcome = Outcome::UserTookOver;
        }
        let state = app.state::<AppState>();
        state.reconnecting.store(false, Ordering::SeqCst);
        // A kill-switch block may still be up from the failed attempts. It stays only while this loop
        // (or its resume) is still going to try; otherwise nothing would ever release it.
        if matches!(outcome, Outcome::UserTookOver | Outcome::Failed(_)) && state.status.lock().unwrap().state == Conn::Disconnected {
            let _ = crate::runner::request_stop(&state.paths);
        }
        if matches!(outcome, Outcome::GaveUp | Outcome::Failed(_)) {
            crate::sysproxy::restore_now(&app).await;
        }
        match outcome {
            Outcome::Reconnected => notify::send(&app, "VPN reconnected", "The tunnel is back up."),
            Outcome::UserTookOver => {}
            Outcome::Failed(error) => notify::send(&app, "Could not reconnect", &format!("{error}. Open the app to connect manually.")),
            Outcome::GaveUp => {
                state.reconnect_waiting.store(true, Ordering::SeqCst);
                applog::info("reconnect gave up; it will start again when the network changes or the server answers");
                let blocked = state.store.lock().unwrap().reliability().kill_switch;
                let note = if blocked { " The kill switch keeps other traffic blocked until then; Settings can restore internet access." } else { "" };
                notify::send(&app, "Could not reconnect", &format!("It will try again when your internet is back, or open the app to connect now.{note}"));
            }
        }
    });
}

/// Stops a tunnel that no longer carries traffic and brings it back up with the usual retries.
pub async fn restart_tunnel(app: &AppHandle, pid: u32, why: &str) {
    let state = app.state::<AppState>();
    if *state.pid.lock().unwrap() != Some(pid) || state.stopping.load(Ordering::SeqCst) {
        return;
    }
    applog::warn(format!("restart tunnel (pid {pid}): {why} and the tunnel stopped responding"));
    notify::send(app, "Reconnecting VPN", &format!("{why} and the tunnel stopped responding."));
    let _ = session::disconnect_for_restart(app).await;
    spawn_reconnect(app.clone());
}

pub fn apply_hotkey(app: &AppHandle) {
    let enabled = app.state::<AppState>().store.lock().unwrap().reliability().hotkey;
    let Ok(shortcut) = HOTKEY.parse::<Shortcut>() else { return };
    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister(shortcut);
    if enabled {
        let _ = shortcuts.register(shortcut);
    }
}

pub fn hotkey_label() -> &'static str {
    HOTKEY
}

/// Follows the root helper's `blocked` marker and publishes it in the status, so Home can say why
/// the internet is blocked, and lifts a block left over after the kill switch was turned off.
/// Costs one `stat` a second.
pub fn spawn_block_watch(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            let state = app.state::<AppState>();
            let shown = state.blocked.load(Ordering::SeqCst);
            let kill_switch = state.store.lock().unwrap().reliability().kill_switch;
            let blocked = state.paths.blocked_file().exists();
            let connection = state.status.lock().unwrap().state;
            let lifted = match unwanted_block(blocked, kill_switch, connection) {
                Some(Lift::Stop) => crate::runner::request_stop(&state.paths),
                Some(Lift::Ready) => crate::runner::signal_ready(&state.paths),
                None => Ok(()),
            };
            if let Err(e) = lifted {
                applog::warn(format!("kill switch is off but a block is up, and it could not be lifted: {e}"));
            }
            if blocked != shown {
                state.blocked.store(blocked, Ordering::SeqCst);
                applog::info(format!("kill switch: traffic is {}", if blocked { "blocked" } else { "released" }));
                session::update(&app, |_| {});
            }
        }
    });
}

/// Ends a kill-switch block the user does not want any more, without a password: the root helper
/// reads the stop file. Also stops a reconnect in progress, so nothing puts the block back.
pub async fn unblock(app: &AppHandle) -> Result<()> {
    user_took_over(app);
    let state = app.state::<AppState>();
    if state.status.lock().unwrap().state == Conn::Disconnected {
        crate::runner::request_stop(&state.paths)
    } else {
        session::disconnect(app).await
    }
}

/// Removes the kill switch firewall rules. Needs one administrator prompt.
pub async fn restore_network() -> Result<()> {
    let flush = std::iter::once(PF_ANCHOR)
        .chain(LEGACY_PF_ANCHORS)
        .map(|anchor| format!("pfctl -a '{anchor}' -F all"))
        .collect::<Vec<_>>()
        .join("; ");
    crate::runner::run_admin_script(&flush, "Kage wants to remove the kill switch firewall rules so your internet works again.").await.map(|_| ())
}

#[cfg(test)]
#[path = "tests/reliability.rs"]
mod tests;
