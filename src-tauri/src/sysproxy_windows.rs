//! The Windows system proxy for Proxy Only mode: the per-user WinINet settings, which Edge, Chrome
//! and most apps follow. No administrator rights are needed.
//!
//! Before the first change the previous values are saved to a file. They are put back on
//! disconnect and quit; if sing-box dies on its own the runner's watcher puts them back; if the app
//! itself dies a small guard process (this same program, started with `GUARD_FLAG`) does; and
//! anything still left is put back at the next launch. A restore only happens while the values
//! still point at Kage, so a change the user made while connected is kept.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
use winreg::RegKey;

use crate::error::{AppError, Result};
use crate::paths::AppPaths;
use crate::wininet::{kage_values, still_kage, Saved, Values};

#[path = "sysproxy_common.rs"]
mod common;
pub use common::*;

const SETTINGS_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";
const SERVICE_NAME: &str = "Windows proxy settings";
/// Starts the guard: `<exe> --restore-proxy-after <app pid> <saved-settings file>`.
pub const GUARD_FLAG: &str = "--restore-proxy-after";
const DETACHED_PROCESS: u32 = 0x0000_0008;
const GUARD_POLL: Duration = Duration::from_secs(1);

// ── The registry ────────────────────────────────────────────────────────────

fn settings_key() -> Result<RegKey> {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(SETTINGS_KEY, KEY_READ | KEY_WRITE)
        .map_err(|e| AppError::Launch(format!("Windows refused to open the proxy settings ({e})")))
}

fn read_values(key: &RegKey) -> Values {
    Values {
        proxy_enable: key.get_value::<u32, _>("ProxyEnable").ok(),
        proxy_server: key.get_value::<String, _>("ProxyServer").ok(),
        proxy_override: key.get_value::<String, _>("ProxyOverride").ok(),
        auto_config_url: key.get_value::<String, _>("AutoConfigURL").ok(),
    }
}

fn write_values(key: &RegKey, values: &Values) -> Result<()> {
    let fail = |e: std::io::Error| AppError::Launch(format!("Windows refused to change the proxy settings ({e})"));
    match values.proxy_enable {
        Some(v) => key.set_value("ProxyEnable", &v).map_err(fail)?,
        None => delete(key, "ProxyEnable").map_err(fail)?,
    }
    for (name, value) in [("ProxyServer", &values.proxy_server), ("ProxyOverride", &values.proxy_override), ("AutoConfigURL", &values.auto_config_url)] {
        match value {
            Some(v) => key.set_value(name, v).map_err(fail)?,
            None => delete(key, name).map_err(fail)?,
        }
    }
    refresh();
    Ok(())
}

/// Deletes a value; one that is not there is already deleted.
fn delete(key: &RegKey, name: &str) -> std::io::Result<()> {
    match key.delete_value(name) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

/// Tells running programs the settings changed, so they do not wait for a restart to notice.
fn refresh() {
    use windows_sys::Win32::Networking::WinInet::{InternetSetOptionW, INTERNET_OPTION_REFRESH, INTERNET_OPTION_SETTINGS_CHANGED};
    // SAFETY: both options take no buffer; a null handle means "all sessions".
    unsafe {
        InternetSetOptionW(std::ptr::null(), INTERNET_OPTION_SETTINGS_CHANGED, std::ptr::null(), 0);
        InternetSetOptionW(std::ptr::null(), INTERNET_OPTION_REFRESH, std::ptr::null(), 0);
    }
}

// ── Saved settings ──────────────────────────────────────────────────────────

fn load(snapshot: &Path) -> Option<Saved> {
    serde_json::from_slice(&std::fs::read(snapshot).ok()?).ok()
}

fn save(paths: &AppPaths, saved: &Saved) -> Result<()> {
    crate::fsutil::write_private_atomic(&paths.system_proxy_snapshot(), &serde_json::to_vec_pretty(saved)?)?;
    Ok(())
}

/// Whether Kage has saved settings it has not put back yet.
pub fn has_snapshot(paths: &AppPaths) -> bool {
    paths.system_proxy_snapshot().exists()
}

/// Points Windows at the local proxy on `port`, saving the previous values first (once per run of
/// changes; a reconnect on a new port only updates the saved port).
pub fn apply(paths: &AppPaths, port: u16, bypass: &[String]) -> SystemProxyState {
    let failed = |reason: String| SystemProxyState::Failed { action: Action::Set, reason };
    let bypass = match bypass_list(bypass) {
        Ok(list) => list,
        Err(e) => return failed(e.to_string()),
    };
    let key = match settings_key() {
        Ok(key) => key,
        Err(e) => return failed(e.to_string()),
    };
    let saved = match load(&paths.system_proxy_snapshot()) {
        Some(previous) => Saved { port, ..previous },
        None => Saved { port, before: read_values(&key) },
    };
    if let Err(e) = save(paths, &saved) {
        return failed(format!("could not save the current settings first: {e}"));
    }
    if let Err(e) = write_values(&key, &kage_values(port, &bypass)) {
        let _ = write_values(&key, &saved.before);
        let _ = std::fs::remove_file(paths.system_proxy_snapshot());
        return failed(e.to_string());
    }
    start_guard(paths);
    SystemProxyState::Set { port, services: vec![SERVICE_NAME.into()] }
}

/// Puts the saved values back if Windows still points at Kage, then forgets them.
pub fn restore(paths: &AppPaths) -> SystemProxyState {
    restore_file(&paths.system_proxy_snapshot())
}

fn restore_file(snapshot: &Path) -> SystemProxyState {
    let Some(saved) = load(snapshot) else {
        let _ = std::fs::remove_file(snapshot);
        return SystemProxyState::Off;
    };
    let result = settings_key().and_then(|key| {
        if still_kage(&read_values(&key), saved.port) {
            write_values(&key, &saved.before)
        } else {
            crate::applog::info("system proxy: the settings were changed while connected; keeping them");
            Ok(())
        }
    });
    match result {
        Ok(()) => {
            let _ = std::fs::remove_file(snapshot);
            SystemProxyState::Off
        }
        Err(e) => SystemProxyState::Failed { action: Action::Restore, reason: e.to_string() },
    }
}

/// For the runner's watcher when sing-box died on its own: nothing would answer on the port.
pub fn restore_after_crash(paths: &AppPaths) {
    if has_snapshot(paths) {
        let _one = BUSY.lock().unwrap_or_else(|e| e.into_inner());
        match restore(paths) {
            SystemProxyState::Off => crate::applog::info("system proxy: put back after sing-box stopped"),
            other => crate::applog::warn(format!("system proxy: could not put back after sing-box stopped: {other:?}")),
        }
    }
}

// ── The guard process ───────────────────────────────────────────────────────

static GUARD: Mutex<Option<Child>> = Mutex::new(None);

/// Starts the guard unless one is already watching this app.
fn start_guard(paths: &AppPaths) {
    let mut guard = GUARD.lock().unwrap_or_else(|e| e.into_inner());
    if guard.as_mut().is_some_and(|child| matches!(child.try_wait(), Ok(None))) {
        return;
    }
    let Ok(exe) = std::env::current_exe() else { return };
    let started = Command::new(exe)
        .arg(GUARD_FLAG)
        .arg(std::process::id().to_string())
        .arg(paths.system_proxy_snapshot())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(crate::runner::CREATE_NO_WINDOW | DETACHED_PROCESS)
        .spawn();
    match started {
        Ok(child) => *guard = Some(child),
        Err(e) => crate::applog::warn(format!("system proxy: could not start the guard ({e}); a crash is repaired at the next launch")),
    }
}

/// Run from `main` before anything else: when this program was started as the guard, waits for the
/// app to exit and puts the settings back if it left them, then returns `true` (the process ends).
/// The guard also ends as soon as the app has put the settings back itself.
pub fn run_guard_if_asked() -> bool {
    let args: Vec<String> = std::env::args().collect();
    let Some(at) = args.iter().position(|a| a == GUARD_FLAG) else { return false };
    let (Some(pid), Some(snapshot)) = (args.get(at + 1).and_then(|p| p.parse::<u32>().ok()), args.get(at + 2)) else { return true };
    let snapshot = Path::new(snapshot);
    loop {
        if !snapshot.exists() {
            return true;
        }
        if !crate::runner::is_alive(pid) {
            restore_file(snapshot);
            return true;
        }
        std::thread::sleep(GUARD_POLL);
    }
}

// ── In the app ──────────────────────────────────────────────────────────────

/// One change at a time: a connect, a stop and a network change can all ask at once.
static BUSY: Mutex<()> = Mutex::new(());

fn record(app: &tauri::AppHandle, new: SystemProxyState) {
    use tauri::Manager;
    match &new {
        SystemProxyState::Set { port, .. } => crate::applog::info(format!("system proxy: set to 127.0.0.1:{port}")),
        SystemProxyState::Off => crate::applog::info("system proxy: settings put back"),
        SystemProxyState::Failed { action, reason } => crate::applog::warn(format!("system proxy: {action:?} failed: {reason}")),
    }
    *app.state::<crate::state::AppState>().system_proxy.lock().unwrap() = new;
    crate::session::update(app, |_| {});
}

/// Brings the Windows settings in line with the current run, like the macOS version.
pub fn sync(app: &tauri::AppHandle) {
    use tauri::Manager;
    let _one = BUSY.lock().unwrap_or_else(|e| e.into_inner());
    let state = app.state::<crate::state::AppState>();
    let (connected, mode) = {
        let status = state.status.lock().unwrap();
        (status.state == crate::session::State::Connected, status.mode)
    };
    let settings = state.store.lock().unwrap().reliability();
    if connected && applies(mode, &settings) {
        record(app, apply(&state.paths, crate::config::mixed_port(), &settings.bypass));
    } else if has_snapshot(&state.paths) {
        record(app, restore(&state.paths));
    }
}

pub fn spawn_sync(app: tauri::AppHandle) {
    tauri::async_runtime::spawn_blocking(move || sync(&app));
}

/// Puts the saved settings back now, whatever the run; for a stop, a quit and Repair.
pub async fn restore_now(app: &tauri::AppHandle) -> SystemProxyState {
    use tauri::Manager;
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _one = BUSY.lock().unwrap_or_else(|e| e.into_inner());
        let state = app.state::<crate::state::AppState>();
        if !has_snapshot(&state.paths) {
            return SystemProxyState::Off;
        }
        let result = restore(&state.paths);
        record(&app, result.clone());
        result
    })
    .await
    .unwrap_or(SystemProxyState::Off)
}

/// At launch: settings left behind by a run that ended without putting them back are put back.
pub fn restore_at_launch(paths: &AppPaths) {
    if has_snapshot(paths) {
        match restore(paths) {
            SystemProxyState::Off => crate::applog::info("system proxy: put back settings left by the last run"),
            other => crate::applog::warn(format!("system proxy: could not put back settings left by the last run: {other:?}")),
        }
    }
}

/// The state to show: a `Set` whose saved settings are gone was put back by the watcher or guard.
pub fn current(state: &crate::state::AppState) -> SystemProxyState {
    let mut current = state.system_proxy.lock().unwrap();
    if matches!(*current, SystemProxyState::Set { .. }) && !has_snapshot(&state.paths) {
        *current = SystemProxyState::Off;
    }
    current.clone()
}
