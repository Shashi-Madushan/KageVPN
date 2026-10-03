use std::sync::atomic::AtomicBool;
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
    pub reconnecting: AtomicBool,
    pub clash_secret: Mutex<Option<String>>,
    pub usage: UsageState,
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
            reconnecting: AtomicBool::new(false),
            clash_secret: Mutex::new(None),
            usage: UsageState::open(&paths_for_usage),
        }
    }
}
