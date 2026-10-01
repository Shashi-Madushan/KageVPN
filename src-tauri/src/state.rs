use std::sync::Mutex;

use crate::logs::LogBuffer;
use crate::paths::AppPaths;
use crate::session::Status;
use crate::store::Store;
use crate::traffic::TrafficState;

pub struct AppState {
    pub paths: AppPaths,
    pub store: Mutex<Store>,
    pub status: Mutex<Status>,
    pub pid: Mutex<Option<u32>>,
    pub logs: LogBuffer,
    pub traffic: Mutex<TrafficState>,
}

impl AppState {
    pub fn new(paths: AppPaths) -> Self {
        let store = Store::load(paths.store_file());
        let status = Status::disconnected(store.mode());
        Self {
            paths,
            store: Mutex::new(store),
            status: Mutex::new(status),
            pid: Mutex::new(None),
            logs: LogBuffer::default(),
            traffic: Mutex::new(TrafficState::default()),
        }
    }
}
