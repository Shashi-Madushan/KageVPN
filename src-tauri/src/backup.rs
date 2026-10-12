//! Backups of everything a person set up: servers, subscriptions, routing and settings, in one
//! file to keep or to carry to another Mac. Installed engines, usage history and the subscription
//! device ID stay behind; they belong to this Mac, not to the setup.

use serde::Serialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Manager, State};

use crate::error::{AppError, Result};
use crate::session::State as ConnState;
use crate::state::AppState;

const KIND: &str = "kage-vpn-backup";
const VERSION: u64 = 1;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreSummary {
    pub servers: usize,
    pub subscriptions: usize,
}

/// The backup file's contents: a labelled envelope around the data file's own layout.
pub fn envelope(data: Value, created: u64) -> Value {
    json!({
        "kind": KIND,
        "version": VERSION,
        "appVersion": crate::app_config::APP_VERSION,
        "created": created,
        "data": data,
    })
}

/// The setup inside a backup. A plain data file copied from another Mac is accepted too.
pub fn unwrap(text: &str) -> Result<Value> {
    let not_backup = || AppError::Invalid("this file is not a Kage VPN backup".into());
    let mut value: Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|_| not_backup())?;
    match value.get("kind").and_then(Value::as_str) {
        Some(KIND) => {
            if value.get("version").and_then(Value::as_u64).is_some_and(|v| v > VERSION) {
                return Err(AppError::Invalid("this backup was made by a newer version of the app. Update the app, then restore it".into()));
            }
            value.get_mut("data").map(Value::take).filter(Value::is_object).ok_or_else(not_backup)
        }
        Some(_) => Err(not_backup()),
        None if value.get("schemaVersion").is_some() && value.get("profiles").is_some() => Ok(value),
        None => Err(not_backup()),
    }
}

#[tauri::command]
pub fn export_backup(app: AppHandle, state: State<AppState>) -> Result<String> {
    let data = state.store.lock().unwrap().backup_value()?;
    let file = envelope(data, crate::subscriptions::now_secs());
    let directory = app.path().download_dir().map_err(|e| AppError::Launch(e.to_string()))?;
    let name = format!("kage-vpn-backup-{}.json", chrono::Local::now().format("%Y-%m-%d-%H%M"));
    let path = directory.join(name);
    // The file holds passwords and keys, so only the owner can read it.
    crate::fsutil::write_private_atomic(&path, &serde_json::to_vec_pretty(&file)?)?;
    Ok(path.display().to_string())
}

#[tauri::command]
pub fn restore_backup(app: AppHandle, state: State<AppState>, text: String) -> Result<RestoreSummary> {
    if state.status.lock().unwrap().state != ConnState::Disconnected {
        return Err(AppError::Invalid("disconnect before restoring a backup".into()));
    }
    let data = unwrap(&text)?;
    let (servers, subscriptions) = state.store.lock().unwrap().restore(data)?;
    crate::reliability::apply_hotkey(&app);
    Ok(RestoreSummary { servers, subscriptions })
}

#[cfg(test)]
#[path = "tests/backup.rs"]
mod tests;
