use std::collections::VecDeque;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::sleep;

use crate::config::CLASH_PORT;
use crate::error::Result;
use crate::state::AppState;
use crate::tray;

const HISTORY_LEN: usize = 60;
const CONNECT_ATTEMPTS: u32 = 10;
const RETRY_DELAY: Duration = Duration::from_millis(300);

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
pub struct Sample {
    pub up: u64,
    pub down: u64,
}

#[derive(Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Traffic {
    pub up: u64,
    pub down: u64,
    pub total_up: u64,
    pub total_down: u64,
}

#[derive(Default)]
pub struct TrafficState {
    current: Traffic,
    history: VecDeque<Sample>,
}

impl TrafficState {
    fn record(&mut self, sample: Sample) -> Traffic {
        self.current = Traffic {
            up: sample.up,
            down: sample.down,
            total_up: self.current.total_up + sample.up,
            total_down: self.current.total_down + sample.down,
        };
        self.history.push_back(sample);
        if self.history.len() > HISTORY_LEN {
            self.history.pop_front();
        }
        self.current
    }

    pub fn snapshot(&self) -> (Traffic, Vec<Sample>) {
        (self.current, self.history.iter().copied().collect())
    }
}

pub fn spawn(app: AppHandle, pid: u32, secret: String) {
    tauri::async_runtime::spawn(async move {
        let _ = stream(&app, pid, &secret).await;
        if app.state::<AppState>().pid.lock().unwrap().is_none_or(|current| current != pid) {
            reset(&app);
        }
    });
}

pub fn reset(app: &AppHandle) {
    let state = app.state::<AppState>();
    *state.traffic.lock().unwrap() = TrafficState::default();
    let _ = app.emit("traffic", Traffic::default());
    tray::set_speed(app, None);
}

async fn stream(app: &AppHandle, pid: u32, secret: &str) -> Result<()> {
    let client = reqwest::Client::builder().no_proxy().build()?;
    let url = format!("http://127.0.0.1:{CLASH_PORT}/traffic");

    let mut response = None;
    for _ in 0..CONNECT_ATTEMPTS {
        match client.get(&url).bearer_auth(secret).send().await {
            Ok(r) if r.status().is_success() => {
                response = Some(r);
                break;
            }
            _ => sleep(RETRY_DELAY).await,
        }
    }
    let Some(mut response) = response else { return Ok(()) };

    let state = app.state::<AppState>();
    let mut buffer = String::new();
    while let Some(chunk) = response.chunk().await? {
        if *state.pid.lock().unwrap() != Some(pid) {
            break;
        }
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(end) = buffer.find('\n') {
            let line: String = buffer.drain(..=end).collect();
            if let Ok(sample) = serde_json::from_str::<Sample>(line.trim()) {
                publish(app, sample);
            }
        }
    }
    Ok(())
}

fn publish(app: &AppHandle, sample: Sample) {
    let state = app.state::<AppState>();
    let traffic = state.traffic.lock().unwrap().record(sample);
    let _ = app.emit("traffic", traffic);
    tray::set_speed(app, Some(traffic));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulates_totals_and_caps_history() {
        let mut state = TrafficState::default();
        for _ in 0..HISTORY_LEN + 5 {
            state.record(Sample { up: 1, down: 2 });
        }
        let (current, history) = state.snapshot();
        assert_eq!(current.total_up, (HISTORY_LEN + 5) as u64);
        assert_eq!(current.total_down, 2 * (HISTORY_LEN + 5) as u64);
        assert_eq!(history.len(), HISTORY_LEN);
    }
}
