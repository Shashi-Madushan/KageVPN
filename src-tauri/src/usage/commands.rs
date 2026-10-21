use std::collections::HashMap;

use tauri::{AppHandle, Manager, State};

use super::query::{self, Report, ReportArgs, TopArgs, TopRow};
use super::quota::{self, QuotaStatus};
use super::store::Db;
use crate::error::{AppError, Result};
use crate::settings::UsageSettings;
use crate::state::AppState;

fn with_db<T>(state: &AppState, action: impl FnOnce(&Db) -> Result<T>) -> Result<T> {
    let guard = state.usage.db.lock().unwrap();
    let db = guard.as_ref().ok_or_else(|| AppError::Invalid("the usage database could not be opened".into()))?;
    action(db)
}

#[tauri::command]
pub fn usage_report(state: State<AppState>, args: ReportArgs) -> Result<Report> {
    state.usage.flush();
    with_db(&state, |db| query::report(db, &args))
}

#[tauri::command]
pub fn usage_top(state: State<AppState>, args: TopArgs) -> Result<Vec<TopRow>> {
    state.usage.flush();
    with_db(&state, |db| query::top(db, &args))
}

#[tauri::command]
pub fn usage_export(app: AppHandle, state: State<AppState>, args: ReportArgs) -> Result<String> {
    state.usage.flush();
    let csv = with_db(&state, |db| query::export_csv(db, &args))?;
    let directory = app.path().download_dir().map_err(|e| AppError::Launch(e.to_string()))?;
    let name = format!(
        "kage-usage-{}-{}.csv",
        query::local_time(args.from).format("%Y%m%d"),
        query::local_time(args.to - 1).format("%Y%m%d")
    );
    let path = directory.join(name);
    std::fs::write(&path, csv)?;
    Ok(path.display().to_string())
}

#[tauri::command]
pub fn usage_get_settings(state: State<AppState>) -> UsageSettings {
    state.store.lock().unwrap().usage_settings()
}

/// What gets stored when the Usage page saves: the new settings, with the alert reset when the
/// cap changes. App data caps and the record of which warning was shown are kept by the app, so
/// the stored ones win over the page's copy, which may be older than a warning that came since.
pub fn merge_usage_settings(current: &UsageSettings, settings: UsageSettings) -> UsageSettings {
    let mut next = settings;
    next.alert_cycle = current.alert_cycle;
    next.alert_level = current.alert_level;
    if next.quota_bytes != current.quota_bytes || next.cycle_day != current.cycle_day {
        next.alert_level = 0;
    }
    next.cycle_day = next.cycle_day.clamp(1, 31);
    next.app_caps = current.app_caps.clone();
    next
}

#[tauri::command]
pub fn usage_set_settings(state: State<AppState>, settings: UsageSettings) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    let next = merge_usage_settings(&store.usage_settings(), settings);
    store.set_usage_settings(next)
}

#[tauri::command]
pub fn usage_quota(app: AppHandle, state: State<AppState>) -> QuotaStatus {
    state.usage.flush();
    quota::status(&app)
}

#[tauri::command]
pub fn usage_reset(state: State<AppState>) -> Result<()> {
    state.usage.flush();
    with_db(&state, |db| db.clear_all())
}

#[tauri::command]
pub fn usage_clear_details(state: State<AppState>) -> Result<()> {
    state.usage.flush();
    with_db(&state, |db| db.clear_details())
}

#[tauri::command]
pub fn latency_history(state: State<AppState>) -> Result<HashMap<String, Vec<Option<u64>>>> {
    with_db(&state, |db| Ok(db.latency_history()?.into_iter().collect()))
}
