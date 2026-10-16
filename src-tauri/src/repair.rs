//! One-click repair of the VPN service: clears the leftovers of an interrupted run and installs the
//! root service again. For a service that an OS update unloaded, or stale files that confuse a start.

use std::path::PathBuf;

use tauri::State;

use crate::applog;
use crate::core;
use crate::error::{AppError, Result};
use crate::paths::AppPaths;
use crate::runner::{self, daemon};
use crate::session::State as Conn;
use crate::state::AppState;

/// The request and signal files the app leaves in its data folder that are still there.
pub fn stale_files(paths: &AppPaths) -> Vec<PathBuf> {
    [
        paths.stop_file(),
        paths.down_file(),
        paths.ready_file(),
        paths.block_file(),
        paths.helper_start_file(),
        paths.helper_quit_file(),
        paths.child_pid_file(),
    ]
    .into_iter()
    .filter(|file| file.exists())
    .collect()
}

/// Deletes the stale files; one that cannot be removed (root-owned) is skipped. Returns how many went.
pub fn clear_stale(paths: &AppPaths) -> usize {
    stale_files(paths).into_iter().filter(|file| std::fs::remove_file(file).is_ok()).count()
}

#[tauri::command]
pub async fn repair_service(state: State<'_, AppState>) -> Result<String> {
    if state.status.lock().unwrap().state != Conn::Disconnected {
        return Err(AppError::Invalid("disconnect before repairing the service".into()));
    }
    let cleared = clear_stale(&state.paths);
    runner::forget_repair_attempts();
    let (spec, settings) = {
        let store = state.store.lock().unwrap();
        (store.active_core(), store.core_settings())
    };
    let (binary, _) = core::resolve(&state.paths, spec, &settings)?;
    applog::info(format!("repair: cleared {cleared} stale file(s), installing the service again"));
    daemon::install(&state.paths, &binary).await?;
    if !daemon::installed() {
        return Err(AppError::Launch("the service files are still missing after the repair".into()));
    }
    applog::info("repair: the service is installed");
    Ok(format!("The VPN service was installed again{}.", if cleared > 0 { format!(" and {cleared} leftover file(s) were cleared") } else { String::new() }))
}

#[cfg(test)]
#[path = "tests/repair.rs"]
mod tests;
