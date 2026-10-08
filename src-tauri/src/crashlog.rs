//! A record of every time the VPN stopped without being asked to (`crashes.log` in the data
//! folder): when, why, sing-box's exit status, and the last errors it printed. Entries are rare,
//! and the file is cut down to its newest half when it grows past `MAX_BYTES`.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

const FILE: &str = "crashes.log";
const MAX_BYTES: u64 = 200_000;

/// The exit-status line the supervisor writes when sing-box dies, from its stderr log.
pub fn exit_status(stderr: &str) -> Option<String> {
    stderr
        .lines()
        .rev()
        .find_map(|line| line.split_once("sing-box exited with status ").map(|(_, rest)| rest.trim().to_string()))
}

/// What to show people: the exit status and the last error, in one sentence.
pub fn summary(status: Option<&str>, last_error: Option<&str>) -> String {
    let status = status.map(|s| s.split_whitespace().next().unwrap_or(s));
    let mut text = match status {
        Some(code) => format!("sing-box exited with status {code}"),
        None => "sing-box stopped unexpectedly".to_string(),
    };
    if let Some(error) = last_error {
        let error = error.split_once(" ERROR ").map_or(error, |(_, rest)| rest);
        text.push_str(": ");
        text.push_str(error.trim());
    }
    text
}

pub fn format_entry(kind: &str, reason: &str, details: &str, time: &str) -> String {
    format!("==== {time} · {kind}\n{reason}\n{}\n\n", details.trim_end())
}

pub fn record(dir: &Path, kind: &str, reason: &str, details: &str) {
    let path = dir.join(FILE);
    let entry = format_entry(kind, reason, details, &chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    if fs::metadata(&path).map(|m| m.len()).unwrap_or(0) > MAX_BYTES {
        if let Ok(text) = fs::read_to_string(&path) {
            let keep = text.len() / 2;
            let cut = text[keep..].find("==== ").map_or(text.len(), |i| keep + i);
            let _ = fs::write(&path, &text[cut..]);
        }
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(entry.as_bytes());
    }
}

#[cfg(test)]
#[path = "tests/crashlog.rs"]
mod tests;
