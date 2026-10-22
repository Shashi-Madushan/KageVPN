//! Data caps per app. Each capped app has its own running tally per period, counted from the
//! connection poll by the app's bundle path, so the usage history (and clearing it) never changes
//! a cap. The pure parts (periods, levels, tallies, rule edits) are separate from the parts that
//! touch the app, so they can be tested alone. See
//! `aidlc-docs/construction/u2-per-app-caps/functional-design/` for the rules.

use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Datelike, Duration, Local, NaiveDate};
use rusqlite::Row;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use super::query::midnight;
use crate::error::{AppError, Result};
use crate::notify;
use crate::session::{self, State as Conn};
use crate::settings::{Action, AppCap, AtLimit, CapPeriod, RouteMode, RoutingSettings, Rule, RuleKind, RuleOrigin};
use crate::state::AppState;

pub const MIN_LIMIT: u64 = 1024 * 1024;
pub const MAX_LIMIT: u64 = 10 * 1024 * 1024 * 1024 * 1024;

// ── Periods and levels ──────────────────────────────────────────────────────

fn first_day(today: NaiveDate, period: CapPeriod) -> NaiveDate {
    match period {
        CapPeriod::Day => today,
        CapPeriod::Week => today - Duration::days(i64::from(today.weekday().num_days_from_monday())),
        CapPeriod::Month => NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today),
    }
}

/// When the period containing `now` began: local midnight, on a Monday for weeks and on the 1st
/// for months.
pub fn period_start(now: DateTime<Local>, period: CapPeriod) -> i64 {
    midnight(first_day(now.date_naive(), period))
}

/// When the next period begins.
pub fn period_end(now: DateTime<Local>, period: CapPeriod) -> i64 {
    let first = first_day(now.date_naive(), period);
    let next = match period {
        CapPeriod::Day => first + Duration::days(1),
        CapPeriod::Week => first + Duration::days(7),
        CapPeriod::Month => {
            let (year, month) = if first.month() == 12 { (first.year() + 1, 1) } else { (first.year(), first.month() + 1) };
            NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(first + Duration::days(31))
        }
    };
    midnight(next)
}

/// 100 at or over the limit, 80 from four fifths of it, otherwise 0.
pub fn level(used: u64, limit: u64) -> u8 {
    if limit == 0 {
        return 0;
    }
    if used >= limit {
        100
    } else if u128::from(used) * 10 >= u128::from(limit) * 8 {
        80
    } else {
        0
    }
}

// ── Tally ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Offer {
    #[default]
    None,
    Pending,
    Taken,
    Dismissed,
}

impl Offer {
    pub fn as_str(self) -> &'static str {
        match self {
            Offer::None => "none",
            Offer::Pending => "pending",
            Offer::Taken => "taken",
            Offer::Dismissed => "dismissed",
        }
    }

    pub fn parse(text: &str) -> Self {
        match text {
            "pending" => Offer::Pending,
            "taken" => Offer::Taken,
            "dismissed" => Offer::Dismissed,
            _ => Offer::None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tally {
    pub path: String,
    pub period_start: i64,
    pub up: u64,
    pub down: u64,
    pub alert_level: u8,
    pub offer: Offer,
}

impl Tally {
    pub fn new(path: &str, period_start: i64) -> Self {
        Self { path: path.to_string(), period_start, up: 0, down: 0, alert_level: 0, offer: Offer::None }
    }

    pub fn used(&self) -> u64 {
        self.up.saturating_add(self.down)
    }

    fn restart(&mut self, period_start: i64) {
        *self = Tally::new(&self.path.clone(), period_start);
    }
}

pub fn read_tally(row: &Row) -> rusqlite::Result<Tally> {
    Ok(Tally {
        path: row.get(0)?,
        period_start: row.get(1)?,
        up: row.get::<_, i64>(2)?.max(0) as u64,
        down: row.get::<_, i64>(3)?.max(0) as u64,
        alert_level: row.get::<_, i64>(4)?.clamp(0, 100) as u8,
        offer: Offer::parse(&row.get::<_, String>(5)?),
    })
}

/// What a tally update did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Change {
    pub rolled_over: bool,
    pub new_alert: Option<u8>,
    pub offer_raised: bool,
}

impl Change {
    pub fn any(&self) -> bool {
        self.rolled_over || self.new_alert.is_some() || self.offer_raised
    }
}

/// Starts the tally over when its period has ended. Returns whether it did.
pub fn roll_over(tally: &mut Tally, period_start: i64) -> bool {
    if tally.period_start == period_start {
        return false;
    }
    tally.restart(period_start);
    true
}

/// Adds bytes to an app's tally and works out the alert and the offer. The alert level only goes
/// up inside a period, so raising the limit never repeats an alert.
pub fn accumulate(tally: &mut Tally, cap: &AppCap, period_start: i64, bytes: (u64, u64)) -> Change {
    let mut change = Change { rolled_over: roll_over(tally, period_start), ..Change::default() };
    tally.up = tally.up.saturating_add(bytes.0);
    tally.down = tally.down.saturating_add(bytes.1);
    let now_level = level(tally.used(), cap.limit_bytes);
    if now_level > tally.alert_level {
        tally.alert_level = now_level;
        change.new_alert = Some(now_level);
    }
    if now_level == 100 && cap.at_limit == AtLimit::OfferBlock && tally.offer == Offer::None {
        tally.offer = Offer::Pending;
        change.offer_raised = true;
    }
    change
}

// ── Validation and rule edits ───────────────────────────────────────────────

pub fn app_name(path: &str) -> String {
    path.trim_end_matches('/').rsplit('/').next().unwrap_or(path).trim_end_matches(".app").to_string()
}

pub fn validate_cap(cap: &AppCap) -> Result<()> {
    if !cap.path.starts_with('/') || !cap.path.trim_end_matches('/').ends_with(".app") {
        return Err(AppError::Invalid("a data cap needs an installed app".into()));
    }
    if cap.limit_bytes < MIN_LIMIT || cap.limit_bytes > MAX_LIMIT {
        return Err(AppError::Invalid("the limit must be between 1 MB and 10 TB".into()));
    }
    let probe = Rule { id: String::new(), enabled: true, kind: RuleKind::App, value: cap.path.clone(), action: Action::Block, origin: RuleOrigin::Manual };
    crate::config::validate_rules(&[probe])
        .map_err(|_| AppError::Invalid(format!("{} cannot have a data cap: this app and its engine need the network for the VPN itself", app_name(&cap.path))))
}

fn is_cap_rule(rule: &Rule, path: &str) -> bool {
    rule.origin == RuleOrigin::Cap && rule.kind == RuleKind::App && rule.value == path
}

pub fn has_cap_rule(rules: &[Rule], path: &str) -> bool {
    rules.iter().any(|r| is_cap_rule(r, path))
}

/// Puts the block for `path` at the top of the list, so it wins over the user's own rule for the
/// same app, which stays underneath.
pub fn apply_cap_rule(rules: &mut Vec<Rule>, path: &str) {
    rules.retain(|r| !is_cap_rule(r, path));
    rules.insert(0, Rule { id: format!("cap:{path}"), enabled: true, kind: RuleKind::App, value: path.to_string(), action: Action::Block, origin: RuleOrigin::Cap });
}

/// Removes the cap's block. Returns whether there was one.
pub fn lift_cap_rule(rules: &mut Vec<Rule>, path: &str) -> bool {
    let before = rules.len();
    rules.retain(|r| !is_cap_rule(r, path));
    rules.len() != before
}

/// Saving the Routing page must not remove or create cap blocks, whatever its draft holds: the
/// stored ones are kept (on top) and any in the incoming list are dropped.
pub fn keep_cap_rules(stored: &[Rule], incoming: Vec<Rule>) -> Vec<Rule> {
    let mut rules: Vec<Rule> = stored.iter().filter(|r| r.origin == RuleOrigin::Cap).cloned().collect();
    rules.extend(incoming.into_iter().filter(|r| r.origin != RuleOrigin::Cap));
    rules
}

/// A per-app block only means something in Rules mode; the old mode becomes the default action.
/// Returns how many saved rules and lists start applying with the switch: the other modes ignore
/// them, so they may have been left there long ago.
pub fn ensure_rules_mode(routing: &mut RoutingSettings) -> (usize, usize) {
    if routing.mode == RouteMode::Rule {
        return (0, 0);
    }
    routing.final_action = if routing.mode == RouteMode::Direct { Action::Direct } else { Action::Proxy };
    routing.mode = RouteMode::Rule;
    (routing.rules.iter().filter(|r| r.enabled).count(), routing.rule_sets.iter().filter(|s| s.enabled).count())
}

/// The notice for rules and lists that a switch to Rules mode turned on, or `None` when there are none.
/// The Apps page words it the same way (`nowActiveMessage` in `appRules.ts`).
pub fn now_active_message(rules: usize, lists: usize) -> Option<String> {
    let part = |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
    let what = match (rules, lists) {
        (0, 0) => return None,
        (r, 0) => part(r, "saved rule", "saved rules"),
        (0, l) => part(l, "saved list", "saved lists"),
        (r, l) => format!("{} and {}", part(r, "saved rule", "saved rules"), part(l, "list", "lists")),
    };
    Some(format!("Per-app choices need Rules mode, so {what} from Routing now apply too. Check them under Routing."))
}

// ── Status for the UI ───────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CapState {
    Ok,
    Warn80,
    Over,
    Blocked,
    Dismissed,
    NotCounting,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapStatus {
    pub path: String,
    pub name: String,
    pub installed: bool,
    pub limit_bytes: u64,
    pub period: CapPeriod,
    pub at_limit: AtLimit,
    pub used: u64,
    pub up: u64,
    pub down: u64,
    pub percent: f64,
    pub level: u8,
    pub state: CapState,
    pub offer: Offer,
    pub period_start: i64,
    pub period_end: i64,
}

pub fn state_of(counting: bool, level: u8, offer: Offer, blocked: bool) -> CapState {
    if blocked {
        CapState::Blocked
    } else if !counting {
        CapState::NotCounting
    } else if level >= 100 {
        if offer == Offer::Dismissed { CapState::Dismissed } else { CapState::Over }
    } else if level >= 80 {
        CapState::Warn80
    } else {
        CapState::Ok
    }
}

pub fn status_of(cap: &AppCap, tally: Option<&Tally>, counting: bool, blocked: bool, installed: bool, now: DateTime<Local>) -> CapStatus {
    let start = period_start(now, cap.period);
    // A tally from an earlier period counts as empty until the next tick rolls it over.
    let current = tally.filter(|t| t.period_start == start);
    let (up, down, offer) = current.map(|t| (t.up, t.down, t.offer)).unwrap_or((0, 0, Offer::None));
    let used = up.saturating_add(down);
    let lvl = level(used, cap.limit_bytes);
    CapStatus {
        path: cap.path.clone(),
        name: app_name(&cap.path),
        installed,
        limit_bytes: cap.limit_bytes,
        period: cap.period,
        at_limit: cap.at_limit,
        used,
        up,
        down,
        percent: if cap.limit_bytes == 0 { 0.0 } else { used as f64 / cap.limit_bytes as f64 * 100.0 },
        level: lvl,
        state: state_of(counting, lvl, offer, blocked),
        offer,
        period_start: start,
        period_end: period_end(now, cap.period),
    }
}

fn human(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let (mut value, mut unit) = (bytes as f64, 0);
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 || value >= 10.0 { format!("{value:.0} {}", UNITS[unit]) } else { format!("{value:.1} {}", UNITS[unit]) }
}

fn period_word(period: CapPeriod) -> &'static str {
    match period {
        CapPeriod::Day => "daily",
        CapPeriod::Week => "weekly",
        CapPeriod::Month => "monthly",
    }
}

// ── Working with the app ────────────────────────────────────────────────────

fn recording(app: &AppHandle) -> bool {
    let settings = app.state::<AppState>().store.lock().unwrap().usage_settings();
    settings.record_apps && !settings.paused
}

fn with_db<T>(state: &AppState, action: impl FnOnce(&super::store::Db) -> Result<T>) -> Result<T> {
    let guard = state.usage.db.lock().unwrap();
    let db = guard.as_ref().ok_or_else(|| AppError::Invalid("the usage database could not be opened".into()))?;
    action(db)
}

fn changed(app: &AppHandle) {
    let _ = app.emit("caps-changed", ());
}

/// Reconnects once so a routing change takes effect. Does nothing while the VPN is off.
async fn reapply(app: &AppHandle) {
    let connected = app.state::<AppState>().status.lock().unwrap().state == Conn::Connected;
    if !connected {
        return;
    }
    let _ = session::disconnect_for_restart(app).await;
    if let Err(e) = session::connect(app).await {
        notify::send(app, "Could not reconnect", &e.to_string());
    }
}

/// Removes a cap's block from the stored routing. Returns whether there was one.
fn lift_stored_block(app: &AppHandle, path: &str) -> Result<bool> {
    let state = app.state::<AppState>();
    let mut store = state.store.lock().unwrap();
    let mut routing = store.routing();
    if !lift_cap_rule(&mut routing.rules, path) {
        return Ok(false);
    }
    store.set_routing(routing)?;
    Ok(true)
}

/// Called every poll. `bytes` is what each capped app moved since the last one, by bundle path;
/// when `counting` is false nothing is added and nothing alerts, but periods still roll over.
pub fn tick(app: &AppHandle, bytes: &HashMap<String, (u64, u64)>, counting: bool) {
    let state = app.state::<AppState>();
    let caps = state.store.lock().unwrap().usage_settings().app_caps;
    if caps.is_empty() {
        return;
    }
    let now = Local::now();
    let mut dirty = false;
    let mut lifted: Vec<String> = Vec::new();
    for cap in &caps {
        let start = period_start(now, cap.period);
        let result = with_db(&state, |db| {
            let mut tally = db.tally(&cap.path)?.unwrap_or_else(|| Tally::new(&cap.path, start));
            let change = if counting {
                accumulate(&mut tally, cap, start, bytes.get(&cap.path).copied().unwrap_or((0, 0)))
            } else {
                Change { rolled_over: roll_over(&mut tally, start), ..Change::default() }
            };
            let touched = counting && bytes.get(&cap.path).is_some_and(|b| b.0 + b.1 > 0);
            if change.any() || touched {
                db.save_tally(&tally)?;
            }
            Ok((tally, change))
        });
        let Ok((tally, change)) = result else { continue };
        if change.rolled_over {
            lifted.push(cap.path.clone());
        }
        let name = app_name(&cap.path);
        let limit = human(cap.limit_bytes);
        let offer_hint = if change.offer_raised { " Open Kage to block it." } else { "" };
        if let Some(lvl) = change.new_alert {
            let body = if lvl >= 100 {
                format!("{name} reached its {limit} {} data cap.{offer_hint}", period_word(cap.period))
            } else {
                format!("{name} used 80% of its {limit} {} data cap.", period_word(cap.period))
            };
            notify::send(app, "Data cap", &body);
        } else if change.offer_raised {
            notify::send(app, "Data cap", &format!("{name} is over its {limit} {} data cap.{offer_hint}", period_word(cap.period)));
        }
        dirty |= change.any();
        let _ = tally;
    }
    // A period ended: lift that app's block (one reconnect for all of them).
    let mut reconnect = false;
    let mut names = Vec::new();
    for path in lifted {
        if matches!(lift_stored_block(app, &path), Ok(true)) {
            reconnect = true;
            names.push(app_name(&path));
        }
    }
    if reconnect {
        notify::send(app, "Data cap reset", &format!("{} can use the VPN again.", names.join(", ")));
        let app = app.clone();
        tauri::async_runtime::spawn(async move { reapply(&app).await });
    }
    if dirty || reconnect {
        changed(app);
    }
}

/// At app start and before each connect: ends the period of every cap whose period is over and
/// lifts its block, without reconnecting (a connect is about to build the configuration).
pub fn rollover_all(app: &AppHandle) {
    let state = app.state::<AppState>();
    let caps = state.store.lock().unwrap().usage_settings().app_caps;
    let now = Local::now();
    for cap in &caps {
        let start = period_start(now, cap.period);
        let rolled = with_db(&state, |db| {
            let Some(mut tally) = db.tally(&cap.path)? else { return Ok(false) };
            if !roll_over(&mut tally, start) {
                return Ok(false);
            }
            db.save_tally(&tally)?;
            Ok(true)
        })
        .unwrap_or(false);
        if rolled {
            let _ = lift_stored_block(app, &cap.path);
        }
    }
}

fn load_status(app: &AppHandle) -> Vec<CapStatus> {
    let state = app.state::<AppState>();
    let caps = state.store.lock().unwrap().usage_settings().app_caps;
    let rules = state.store.lock().unwrap().routing().rules;
    let counting = recording(app);
    let now = Local::now();
    caps.iter()
        .map(|cap| {
            let tally = with_db(&state, |db| db.tally(&cap.path)).ok().flatten();
            status_of(cap, tally.as_ref(), counting, has_cap_rule(&rules, &cap.path), Path::new(&cap.path).is_dir(), now)
        })
        .collect()
}

// ── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn caps_list(app: AppHandle) -> Vec<CapStatus> {
    load_status(&app)
}

#[tauri::command]
pub async fn cap_set(app: AppHandle, cap: AppCap) -> Result<()> {
    validate_cap(&cap)?;
    let state = app.state::<AppState>();
    let mut lifted = false;
    {
        let mut store = state.store.lock().unwrap();
        let mut settings = store.usage_settings();
        let previous = settings.app_caps.iter().find(|c| c.path == cap.path).cloned();
        match settings.app_caps.iter_mut().find(|c| c.path == cap.path) {
            Some(existing) => *existing = cap.clone(),
            None => settings.app_caps.push(cap.clone()),
        }
        store.set_usage_settings(settings)?;
        drop(store);
        let period_changed = previous.as_ref().is_some_and(|p| p.period != cap.period);
        let start = period_start(Local::now(), cap.period);
        let _ = with_db(&state, |db| {
            if period_changed {
                db.delete_tally(&cap.path)?;
            } else if let Some(mut tally) = db.tally(&cap.path)? {
                // A higher limit can take a pending offer back; a lower one is picked up by the next tick.
                if tally.period_start == start && tally.offer == Offer::Pending && level(tally.used(), cap.limit_bytes) < 100 {
                    tally.offer = Offer::None;
                    db.save_tally(&tally)?;
                }
            }
            Ok(())
        });
    }
    if cap.at_limit == AtLimit::Notify {
        lifted = lift_stored_block(&app, &cap.path)?;
    }
    changed(&app);
    if lifted {
        reapply(&app).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn cap_remove(app: AppHandle, path: String) -> Result<()> {
    let state = app.state::<AppState>();
    {
        let mut store = state.store.lock().unwrap();
        let mut settings = store.usage_settings();
        settings.app_caps.retain(|c| c.path != path);
        store.set_usage_settings(settings)?;
    }
    let _ = with_db(&state, |db| db.delete_tally(&path));
    let lifted = lift_stored_block(&app, &path)?;
    changed(&app);
    if lifted {
        reapply(&app).await;
    }
    Ok(())
}

fn pending_tally(state: &State<AppState>, path: &str) -> Result<Tally> {
    let tally = with_db(state, |db| db.tally(path))?;
    match tally {
        Some(t) if t.offer == Offer::Pending => Ok(t),
        _ => Err(AppError::Invalid("this app has not reached its data cap".into())),
    }
}

#[tauri::command]
pub async fn cap_block(app: AppHandle, state: State<'_, AppState>, path: String) -> Result<()> {
    let mut tally = pending_tally(&state, &path)?;
    let now_active = {
        let mut store = state.store.lock().unwrap();
        let mut routing = store.routing();
        let (rules, lists) = ensure_rules_mode(&mut routing);
        apply_cap_rule(&mut routing.rules, &path);
        crate::config::validate_rules(&routing.rules)?;
        store.set_routing(routing)?;
        now_active_message(rules, lists)
    };
    tally.offer = Offer::Taken;
    with_db(&state, |db| db.save_tally(&tally))?;
    changed(&app);
    reapply(&app).await;
    if let Some(message) = now_active {
        notify::send(&app, "Routing switched to Rules mode", &message);
    }
    Ok(())
}

#[tauri::command]
pub fn cap_not_now(app: AppHandle, state: State<AppState>, path: String) -> Result<()> {
    let mut tally = pending_tally(&state, &path)?;
    tally.offer = Offer::Dismissed;
    with_db(&state, |db| db.save_tally(&tally))?;
    changed(&app);
    Ok(())
}

#[tauri::command]
pub async fn cap_unblock(app: AppHandle, state: State<'_, AppState>, path: String) -> Result<()> {
    let lifted = lift_stored_block(&app, &path)?;
    let _ = with_db(&state, |db| {
        if let Some(mut tally) = db.tally(&path)? {
            tally.offer = Offer::Dismissed;
            db.save_tally(&tally)?;
        }
        Ok(())
    });
    changed(&app);
    if lifted {
        reapply(&app).await;
    }
    Ok(())
}

#[cfg(test)]
#[path = "../tests/usage_caps.rs"]
mod tests;
