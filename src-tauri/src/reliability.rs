use std::process::Command;
use std::sync::atomic::Ordering;
use std::time::Duration;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tokio::time::sleep;

use crate::error::{AppError, Result};
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

pub fn spawn_reconnect(app: AppHandle) {
    let state = app.state::<AppState>();
    if state.reconnecting.swap(true, Ordering::SeqCst) {
        return;
    }
    tauri::async_runtime::spawn(async move {
        let mut reconnected = false;
        for delay in RECONNECT_DELAYS {
            sleep(Duration::from_secs(delay)).await;
            let result = session::connect(&app).await;
            applog::info(format!("reconnect after {delay}s: {}", result.as_ref().map_or_else(|e| e.to_string(), |_| "ok".into())));
            match result {
                Ok(()) => {
                    reconnected = true;
                    break;
                }
                Err(AppError::Busy) => {
                    reconnected = true;
                    break;
                }
                Err(_) => {}
            }
        }
        app.state::<AppState>().reconnecting.store(false, Ordering::SeqCst);
        notify::send(
            &app,
            if reconnected { "VPN reconnected" } else { "Could not reconnect" },
            if reconnected { "The tunnel is back up." } else { "Open the app to connect manually." },
        );
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
    let _ = session::disconnect(app).await;
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

/// Removes the kill switch firewall rules. Needs one administrator prompt.
pub async fn restore_network() -> Result<()> {
    let flush = std::iter::once(PF_ANCHOR)
        .chain(LEGACY_PF_ANCHORS)
        .map(|anchor| format!("pfctl -a '{anchor}' -F all"))
        .collect::<Vec<_>>()
        .join("; ");
    let script = format!("do shell script \"{flush}\" with administrator privileges");
    let output = tauri::async_runtime::spawn_blocking(move || Command::new("osascript").arg("-e").arg(script).output())
        .await
        .map_err(|e| AppError::Launch(e.to_string()))??;
    if output.status.success() {
        Ok(())
    } else {
        Err(AppError::PermissionDenied)
    }
}
