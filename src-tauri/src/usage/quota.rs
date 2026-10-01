use chrono::{DateTime, Datelike, Local, NaiveDate};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use super::query::midnight;
use crate::notify;
use crate::session;
use crate::settings::QuotaAction;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaStatus {
    pub limit: Option<u64>,
    pub used: u64,
    pub up: u64,
    pub down: u64,
    pub cycle_start: i64,
    pub cycle_end: i64,
    pub percent: f64,
}

pub fn cycle_bounds(now: DateTime<Local>, day: u8) -> (i64, i64) {
    let day = u32::from(day.clamp(1, 31));
    let this = start_of(now.year(), now.month(), day);
    let (start_year, start_month) = if now.timestamp() >= this {
        (now.year(), now.month())
    } else {
        shift(now.year(), now.month(), -1)
    };
    let (end_year, end_month) = shift(start_year, start_month, 1);
    (start_of(start_year, start_month, day), start_of(end_year, end_month, day))
}

fn shift(year: i32, month: u32, delta: i32) -> (i32, u32) {
    let index = year * 12 + (month as i32 - 1) + delta;
    (index.div_euclid(12), (index.rem_euclid(12) + 1) as u32)
}

fn start_of(year: i32, month: u32, day: u32) -> i64 {
    let (next_year, next_month) = shift(year, month, 1);
    let last = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(28);
    midnight(NaiveDate::from_ymd_opt(year, month, day.min(last)).unwrap_or_default())
}

pub fn status(app: &AppHandle) -> QuotaStatus {
    let state = app.state::<AppState>();
    let settings = state.store.lock().unwrap().usage_settings();
    let (cycle_start, cycle_end) = cycle_bounds(Local::now(), settings.cycle_day);
    let (up, down) = state
        .usage
        .db
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|db| db.sum_since(cycle_start, cycle_end).ok())
        .unwrap_or_default();
    let used = up + down;
    let limit = settings.quota_bytes.filter(|l| *l > 0);
    let percent = limit.map(|l| used as f64 / l as f64 * 100.0).unwrap_or(0.0);
    QuotaStatus { limit, used, up, down, cycle_start, cycle_end, percent }
}

/// Raises each threshold (80%, 100%) once per billing cycle.
pub fn check(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut settings = state.store.lock().unwrap().usage_settings();
    let quota = status(app);
    let Some(limit) = quota.limit else { return };

    let mut changed = false;
    if settings.alert_cycle != quota.cycle_start {
        settings.alert_cycle = quota.cycle_start;
        settings.alert_level = 0;
        changed = true;
    }
    let level = if quota.used >= limit { 100 } else if quota.used * 10 >= limit * 8 { 80 } else { 0 };
    if level > settings.alert_level {
        settings.alert_level = level;
        changed = true;
        let body = if level == 100 { "You reached your data cap for this cycle." } else { "You used 80% of your data cap." };
        notify::send(app, "Data usage", body);
        if level == 100 && settings.quota_action == QuotaAction::Disconnect {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let _ = session::disconnect(&app).await;
            });
        }
    }
    if changed {
        let _ = state.store.lock().unwrap().set_usage_settings(settings);
    }
}
