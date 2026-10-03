use std::process::Command;
use std::sync::atomic::Ordering;
use std::time::Duration;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tokio::time::sleep;

use crate::error::{AppError, Result};
use crate::state::AppState;
use crate::{net, notify, session};

const RECONNECT_DELAYS: [u64; 4] = [2, 5, 15, 30];
const WATCHDOG_INTERVAL: Duration = Duration::from_secs(30);
const WATCHDOG_FAILURES: u32 = 3;
const HOTKEY: &str = "CommandOrControl+Alt+V";
const PF_ANCHOR: &str = "com.apple/250.OspreyVPN";
/// Anchor used before the rename; still flushed so an old kill switch can never strand the network.
const LEGACY_PF_ANCHOR: &str = "com.apple/250.SingBoxGUI";

pub fn on_connected(app: &AppHandle, pid: u32) {
    if app.state::<AppState>().store.lock().unwrap().reliability().watchdog {
        spawn_watchdog(app.clone(), pid);
    }
}

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
            match session::connect(&app).await {
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

fn spawn_watchdog(app: AppHandle, pid: u32) {
    tauri::async_runtime::spawn(async move {
        let mut failures = 0;
        loop {
            sleep(WATCHDOG_INTERVAL).await;
            if *app.state::<AppState>().pid.lock().unwrap() != Some(pid) {
                return;
            }
            match net::proxy_delay().await {
                Ok(_) => failures = 0,
                Err(_) => failures += 1,
            }
            if failures >= WATCHDOG_FAILURES {
                session::tunnel_lost(&app, pid).await;
                return;
            }
        }
    });
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
    let script = format!(
        "do shell script \"pfctl -a '{PF_ANCHOR}' -F all; pfctl -a '{LEGACY_PF_ANCHOR}' -F all\" with administrator privileges"
    );
    let output = tauri::async_runtime::spawn_blocking(move || Command::new("osascript").arg("-e").arg(script).output())
        .await
        .map_err(|e| AppError::Launch(e.to_string()))??;
    if output.status.success() {
        Ok(())
    } else {
        Err(AppError::PermissionDenied)
    }
}
