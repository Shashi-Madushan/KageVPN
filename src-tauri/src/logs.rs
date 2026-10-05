use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::applog;
use crate::state::AppState;

const CAPACITY: usize = 2000;
const POLL_INTERVAL: Duration = Duration::from_millis(400);
/// At most this many lines go to the UI per poll; a traffic burst can log thousands.
const MAX_LINES_PER_POLL: usize = 300;
const LEVELS: [(&str, Level); 5] = [
    ("FATAL", Level::Error),
    ("ERROR", Level::Error),
    ("WARN", Level::Warn),
    ("INFO", Level::Info),
    ("DEBUG", Level::Debug),
];

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Hint {
    KeyMismatch { key: String },
    Unresolved,
}

#[derive(Clone, Serialize)]
pub struct LogLine {
    pub level: Level,
    pub text: String,
    pub hint: Option<Hint>,
}

pub fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        if chars.next() == Some('[') {
            for end in chars.by_ref() {
                if end.is_ascii_alphabetic() {
                    break;
                }
            }
        }
    }
    out
}

impl LogLine {
    pub fn parse(raw: &str) -> Self {
        let text = strip_ansi(raw);
        let text = text.as_str();
        let level = text
            .split_whitespace()
            .take(5)
            .find_map(|token| LEVELS.iter().find(|(name, _)| *name == token).map(|(_, l)| *l))
            .unwrap_or(Level::Info);
        Self { level, text: text.to_string(), hint: detect_hint(text) }
    }
}

fn detect_hint(text: &str) -> Option<Hint> {
    const KEY_MARKER: &str = "unrecognized remote public key: ";
    if let Some(index) = text.find(KEY_MARKER) {
        let key = text[index + KEY_MARKER.len()..].split_whitespace().next()?;
        return Some(Hint::KeyMismatch { key: key.to_string() });
    }
    if text.contains("NXDOMAIN") || text.contains("no such host") {
        return Some(Hint::Unresolved);
    }
    None
}

#[derive(Default)]
pub struct LogBuffer {
    lines: Mutex<VecDeque<LogLine>>,
}

impl LogBuffer {
    pub fn push(&self, new_lines: &[LogLine]) {
        let mut lines = self.lines.lock().unwrap();
        lines.extend(new_lines.iter().cloned());
        let overflow = lines.len().saturating_sub(CAPACITY);
        lines.drain(..overflow);
    }

    pub fn snapshot(&self) -> Vec<LogLine> {
        self.lines.lock().unwrap().iter().cloned().collect()
    }

    pub fn clear(&self) {
        self.lines.lock().unwrap().clear();
    }

    pub fn last_error(&self) -> Option<String> {
        let lines = self.lines.lock().unwrap();
        lines.iter().rev().find(|l| l.level == Level::Error).map(|l| l.text.clone())
    }
}

pub fn spawn_tailer(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut offset = 0u64;
        let mut pending = String::new();

        loop {
            tokio::time::sleep(POLL_INTERVAL).await;
            let state = app.state::<AppState>();
            let lines = read_new_lines(&state.paths.log_file(), &mut offset, &mut pending);
            if lines.is_empty() {
                continue;
            }
            let (lines, dropped) = cap_lines(lines, MAX_LINES_PER_POLL);
            if dropped > 0 {
                applog::warn(format!("sing-box log burst: {dropped} lines not sent to the UI"));
            }
            state.logs.push(&lines);
            let _ = app.emit("logs", lines);
        }
    });
}

/// Keeps every warning and error plus the newest lines, up to `cap` of the rest; returns how many
/// lines were dropped. A burst of per-connection INFO lines would otherwise flood the UI.
fn cap_lines(lines: Vec<LogLine>, cap: usize) -> (Vec<LogLine>, usize) {
    if lines.len() <= cap {
        return (lines, 0);
    }
    let first_kept = lines.len() - cap;
    let before = lines.len();
    let kept: Vec<LogLine> = lines
        .into_iter()
        .enumerate()
        .filter(|(index, line)| *index >= first_kept || matches!(line.level, Level::Warn | Level::Error))
        .map(|(_, line)| line)
        .collect();
    let dropped = before - kept.len();
    (kept, dropped)
}

fn read_new_lines(path: &std::path::Path, offset: &mut u64, pending: &mut String) -> Vec<LogLine> {
    let Ok(mut file) = File::open(path) else {
        *offset = 0;
        return Vec::new();
    };
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    if len < *offset {
        *offset = 0;
        pending.clear();
    }
    if len == *offset || file.seek(SeekFrom::Start(*offset)).is_err() {
        return Vec::new();
    }

    let mut bytes = Vec::new();
    if file.read_to_end(&mut bytes).is_err() {
        return Vec::new();
    }
    *offset += bytes.len() as u64;
    pending.push_str(&String::from_utf8_lossy(&bytes));

    let mut lines = Vec::new();
    while let Some(end) = pending.find('\n') {
        let line: String = pending.drain(..=end).collect();
        let line = line.trim_end();
        if !line.is_empty() {
            lines.push(LogLine::parse(line));
        }
    }
    lines
}

#[cfg(test)]
#[path = "tests/logs.rs"]
mod tests;
