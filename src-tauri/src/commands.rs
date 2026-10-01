use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

use crate::config::Mode;
use crate::error::{AppError, Result};
use crate::logs::LogLine;
use crate::profile::{parse_vless, Profile};
use crate::session::{self, Status};
use crate::state::AppState;
use crate::store::Settings;
use crate::traffic::{Sample, Traffic};
use crate::{net, tray};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    profiles: Vec<Profile>,
    active_id: Option<String>,
    mode: Mode,
    status: Status,
    logs: Vec<LogLine>,
    traffic: Traffic,
    history: Vec<Sample>,
    settings: Settings,
    launch_at_login: bool,
}

#[tauri::command]
pub fn snapshot(app: AppHandle, state: State<AppState>) -> Snapshot {
    let store = state.store.lock().unwrap();
    let (traffic, history) = state.traffic.lock().unwrap().snapshot();
    Snapshot {
        profiles: store.profiles().to_vec(),
        active_id: store.active_id().map(str::to_string),
        mode: store.mode(),
        status: state.status.lock().unwrap().clone(),
        logs: state.logs.snapshot(),
        traffic,
        history,
        settings: store.settings(),
        launch_at_login: app.autolaunch().is_enabled().unwrap_or(false),
    }
}

#[tauri::command]
pub fn import_profile(state: State<AppState>, link: String) -> Result<Profile> {
    let profile = parse_vless(&link)?;
    state.store.lock().unwrap().add(profile.clone())?;
    Ok(profile)
}

#[tauri::command]
pub fn delete_profile(state: State<AppState>, id: String) -> Result<()> {
    state.store.lock().unwrap().remove(&id)
}

#[tauri::command]
pub fn select_profile(state: State<AppState>, id: String) -> Result<()> {
    state.store.lock().unwrap().select(&id)
}

#[tauri::command]
pub fn set_mode(state: State<AppState>, mode: Mode) -> Result<()> {
    state.store.lock().unwrap().set_mode(mode)
}

#[tauri::command]
pub fn trust_key(state: State<AppState>, id: String, key: String) -> Result<()> {
    state.store.lock().unwrap().trust_key(&id, key)
}

#[tauri::command]
pub fn clear_logs(state: State<AppState>) {
    state.logs.clear();
}

#[tauri::command]
pub async fn connect(app: AppHandle) -> Result<()> {
    session::connect(&app).await
}

#[tauri::command]
pub async fn disconnect(app: AppHandle) -> Result<()> {
    session::disconnect(&app).await
}

#[tauri::command]
pub async fn ping_profile(state: State<'_, AppState>, id: String) -> Result<u64> {
    let (server, port) = {
        let store = state.store.lock().unwrap();
        let profile = store.get(&id).ok_or(AppError::ProfileNotFound)?;
        (profile.server.clone(), profile.port)
    };
    net::tcp_latency(&server, port).await
}

#[tauri::command]
pub async fn test_connection() -> Result<u64> {
    net::proxy_delay().await
}

#[tauri::command]
pub async fn exit_ip() -> Result<String> {
    net::exit_ip().await
}

#[tauri::command]
pub fn rename_profile(state: State<AppState>, id: String, name: String) -> Result<()> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Invalid("name cannot be empty".into()));
    }
    state.store.lock().unwrap().rename(&id, name)
}

#[tauri::command]
pub fn update_settings(app: AppHandle, state: State<AppState>, settings: Settings) -> Result<()> {
    state.store.lock().unwrap().set_settings(settings)?;
    if !settings.show_speed {
        tray::set_speed(&app, None);
    }
    Ok(())
}

#[tauri::command]
pub fn set_launch_at_login(app: AppHandle, enabled: bool) -> Result<()> {
    let autolaunch = app.autolaunch();
    let result = if enabled { autolaunch.enable() } else { autolaunch.disable() };
    result.map_err(|e| AppError::Launch(e.to_string()))
}

#[tauri::command]
pub fn open_main_window(app: AppHandle) {
    tray::show_main_window(&app);
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    tray::quit_app(&app);
}
