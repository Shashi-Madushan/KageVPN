//! What this build can do on the OS it runs on, so the interface shows only what works here.

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Platform {
    /// `"macos"` or `"windows"`.
    pub os: &'static str,
    /// TUN mode (the whole-system tunnel).
    pub tun: bool,
    /// The firewall kill switch.
    pub kill_switch: bool,
    /// A background service that runs the tunnel without asking for a password each time.
    pub root_service: bool,
    pub system_proxy: bool,
    /// macOS App Nap, which the app holds off while connected.
    pub app_nap: bool,
    /// The live speed next to the tray icon; Windows tray icons are square, with no room for it.
    pub speed_in_tray: bool,
}

#[cfg(target_os = "macos")]
pub const CURRENT: Platform = Platform { os: "macos", tun: true, kill_switch: true, root_service: true, system_proxy: true, app_nap: true, speed_in_tray: true };

/// Windows: Proxy Only, the system proxy, and TUN through the helper service. The kill switch
/// comes in phase 3; until then TUN uses sing-box's `strict_route`.
#[cfg(windows)]
pub const CURRENT: Platform = Platform { os: "windows", tun: true, kill_switch: false, root_service: true, system_proxy: true, app_nap: false, speed_in_tray: false };

#[cfg(not(any(target_os = "macos", windows)))]
pub const CURRENT: Platform = Platform { os: "other", tun: false, kill_switch: false, root_service: false, system_proxy: false, app_nap: false, speed_in_tray: false };

/// Why TUN mode is refused on an OS that does not have it yet.
pub const TUN_UNAVAILABLE: &str = "TUN mode is not available on Windows yet. Use Proxy Only mode.";

#[tauri::command]
pub fn platform() -> Platform {
    CURRENT
}

#[cfg(test)]
#[path = "tests/platform.rs"]
mod tests;
