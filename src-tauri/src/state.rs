use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Mutex;

use crate::downloads::Downloads;
use crate::logs::LogBuffer;
use crate::paths::AppPaths;
use crate::session::Status;
use crate::store::Store;
use crate::traffic::TrafficState;
use crate::usage::collector::UsageState;

pub struct AppState {
    pub paths: AppPaths,
    pub store: Mutex<Store>,
    pub status: Mutex<Status>,
    pub pid: Mutex<Option<u32>>,
    pub logs: LogBuffer,
    pub traffic: Mutex<TrafficState>,
    pub downloads: Downloads,
    pub stopping: AtomicBool,
    /// Set when the user disconnects while the connection is still starting; `launch` stops at its
    /// next checkpoint.
    pub cancel_connect: AtomicBool,
    /// When the app last asked sing-box to stop, so the next connect can wait for the port to free.
    pub stop_requested_at: Mutex<Option<std::time::Instant>>,
    /// Whether the kill switch is blocking traffic right now, as reported by the root helper.
    pub blocked: AtomicBool,
    pub reconnecting: AtomicBool,
    /// The reconnect loop gave up while the network was down; it starts again when the network returns.
    pub reconnect_waiting: AtomicBool,
    /// Counts the times the user connected or disconnected by hand. A reconnect in flight compares
    /// it with the value it started under and stands down when it changed.
    pub manual_actions: AtomicU64,
    pub clash_secret: Mutex<Option<String>>,
    pub usage: UsageState,
    /// What Kage last did to the macOS proxy settings; see `sysproxy`.
    pub system_proxy: Mutex<crate::sysproxy::SystemProxyState>,
}

impl AppState {
    pub fn new(paths: AppPaths) -> Self {
        let paths_for_usage = paths.clone();
        let store = Store::load(paths.store_file());
        let status = Status::disconnected(store.mode());
        Self {
            paths,
            store: Mutex::new(store),
            status: Mutex::new(status),
            pid: Mutex::new(None),
            logs: LogBuffer::default(),
            traffic: Mutex::new(TrafficState::default()),
            downloads: Downloads::default(),
            stopping: AtomicBool::new(false),
            cancel_connect: AtomicBool::new(false),
            stop_requested_at: Mutex::new(None),
            blocked: AtomicBool::new(false),
            reconnecting: AtomicBool::new(false),
            reconnect_waiting: AtomicBool::new(false),
            manual_actions: AtomicU64::new(0),
            clash_secret: Mutex::new(None),
            usage: UsageState::open(&paths_for_usage),
            system_proxy: Mutex::new(Default::default()),
        }
    }
}
