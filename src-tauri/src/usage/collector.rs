use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager};
use tokio::time::sleep;

use super::store::Db;
use crate::clash;
use crate::paths::AppPaths;
use crate::state::AppState;

const FLUSH_EVERY: Duration = Duration::from_secs(10);
const POLL_EVERY: Duration = Duration::from_secs(3);
const MAINTAIN_EVERY: Duration = Duration::from_secs(6 * 3600);
const KIND_APP: &str = "app";
const KIND_DOMAIN: &str = "domain";

struct Accumulator {
    minute: i64,
    profile: String,
    mode: String,
    up: u64,
    down: u64,
    dirty: bool,
    last_flush: Instant,
    dims: HashMap<(i64, String, &'static str, String), (u64, u64)>,
}

pub struct UsageState {
    pub db: Mutex<Option<Db>>,
    acc: Mutex<Accumulator>,
    seen: Mutex<HashMap<String, (u64, u64)>>,
}

impl UsageState {
    pub fn open(paths: &AppPaths) -> Self {
        Self {
            db: Mutex::new(Db::open(&paths.usage_db()).ok()),
            acc: Mutex::new(Accumulator {
                minute: 0,
                profile: String::new(),
                mode: String::new(),
                up: 0,
                down: 0,
                dirty: false,
                last_flush: Instant::now(),
                dims: HashMap::new(),
            }),
            seen: Mutex::new(HashMap::new()),
        }
    }

    pub fn record_sample(&self, profile: &str, mode: &str, up: u64, down: u64) {
        let minute = now_secs() - now_secs().rem_euclid(60);
        let mut acc = self.acc.lock().unwrap();
        if acc.minute != minute || acc.profile != profile || acc.mode != mode {
            self.write(&mut acc);
            acc.minute = minute;
            acc.profile = profile.to_string();
            acc.mode = mode.to_string();
        }
        acc.up += up;
        acc.down += down;
        acc.dirty = true;
    }

    fn record_dim(&self, profile: &str, kind: &'static str, key: String, up: u64, down: u64) {
        let hour = now_secs() - now_secs().rem_euclid(3600);
        let mut acc = self.acc.lock().unwrap();
        let entry = acc.dims.entry((hour, profile.to_string(), kind, key)).or_default();
        entry.0 += up;
        entry.1 += down;
        acc.dirty = true;
    }

    fn write(&self, acc: &mut Accumulator) {
        if !acc.dirty {
            return;
        }
        if let Some(db) = self.db.lock().unwrap().as_ref() {
            if acc.minute > 0 {
                let _ = db.upsert_minute(acc.minute, &acc.profile, &acc.mode, acc.up, acc.down);
            }
            for ((hour, profile, kind, key), (up, down)) in acc.dims.drain() {
                let _ = db.upsert_dim(hour, &profile, kind, &key, up, down);
            }
        }
        acc.up = 0;
        acc.down = 0;
        acc.dims.clear();
        acc.dirty = false;
        acc.last_flush = Instant::now();
    }

    pub fn flush(&self) {
        let mut acc = self.acc.lock().unwrap();
        self.write(&mut acc);
    }

    pub fn flush_due(&self) -> bool {
        self.acc.lock().unwrap().last_flush.elapsed() >= FLUSH_EVERY
    }
}

/// Polls the connection table and attributes byte deltas to apps and domains.
/// These figures are estimates: bytes of connections that closed between polls are missed.
pub fn spawn_connection_poll(app: AppHandle, pid: u32) {
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        state.usage.seen.lock().unwrap().clear();

        loop {
            sleep(POLL_EVERY).await;
            if *state.pid.lock().unwrap() != Some(pid) {
                break;
            }
            let settings = state.store.lock().unwrap().usage_settings();
            if settings.paused || !(settings.record_apps || settings.record_domains) {
                continue;
            }
            let Ok(connections) = clash::connections(&app).await else { continue };
            let profile = current_profile(&app);

            let mut seen = state.usage.seen.lock().unwrap();
            let mut live = HashMap::new();
            for connection in connections {
                let previous = seen.get(&connection.id).copied().unwrap_or((0, 0));
                let up = connection.up.saturating_sub(previous.0);
                let down = connection.down.saturating_sub(previous.1);
                live.insert(connection.id.clone(), (connection.up, connection.down));
                if up == 0 && down == 0 {
                    continue;
                }
                if settings.record_apps && !connection.process.is_empty() {
                    state.usage.record_dim(&profile, KIND_APP, connection.process.clone(), up, down);
                }
                if settings.record_domains {
                    let host = if connection.host.is_empty() { connection.destination_ip.clone() } else { connection.host.to_lowercase() };
                    if !host.is_empty() {
                        state.usage.record_dim(&profile, KIND_DOMAIN, host, up, down);
                    }
                }
            }
            *seen = live;
        }
    });
}

pub fn spawn_maintenance(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let state = app.state::<AppState>();
            let retention = state.store.lock().unwrap().usage_settings().detail_retention_days;
            if let Some(db) = state.usage.db.lock().unwrap().as_ref() {
                let _ = db.maintain(now_secs(), retention);
            }
            sleep(MAINTAIN_EVERY).await;
        }
    });
}

/// Profile id the usage rows are filed under; the auto group is recorded as "auto".
pub fn current_profile(app: &AppHandle) -> String {
    let state = app.state::<AppState>();
    let store = state.store.lock().unwrap();
    if store.auto().enabled {
        "auto".to_string()
    } else {
        store.active_id().unwrap_or("unknown").to_string()
    }
}

pub fn now_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}
