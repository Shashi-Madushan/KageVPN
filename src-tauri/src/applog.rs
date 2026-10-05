//! The app's own diagnostic log (`app.log` in the data folder; sing-box keeps its own).
//!
//! It records the moments that explain a restart or a dropped tunnel: start and exit, panics,
//! connect/disconnect decisions, network changes, and a resource sample every 30 seconds. Lines
//! are rare by design (never one per connection), the file is capped, and a failure to write
//! is ignored so logging can never take the app down.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MAX_BYTES: u64 = 1_000_000;
const FILE: &str = "app.log";
const MARKER: &str = "running.pid";

struct Sink {
    path: PathBuf,
    file: File,
    written: u64,
}

static SINK: Mutex<Option<Sink>> = Mutex::new(None);

/// Opens the log, notes whether the previous run ended without a clean exit, and installs the
/// panic hook. Call once, first thing at startup.
pub fn init(dir: &Path) {
    let path = dir.join(FILE);
    let written = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    if let Ok(file) = OpenOptions::new().create(true).append(true).open(&path) {
        *lock() = Some(Sink { path, file, written });
    }

    let marker = dir.join(MARKER);
    if let Ok(previous) = fs::read_to_string(&marker) {
        warn(format!("previous run (pid {}) did not exit cleanly: crash, panic, or killed", previous.trim()));
    }
    let _ = fs::write(&marker, std::process::id().to_string());

    info(format!(
        "start v{} pid {} args {:?}",
        env!("CARGO_PKG_VERSION"),
        std::process::id(),
        std::env::args().skip(1).collect::<Vec<_>>()
    ));
    install_panic_hook();
}

/// Marks the run as ended cleanly so the next start does not report a crash.
pub fn clean_exit(dir: &Path) {
    info("exit: clean");
    let _ = fs::remove_file(dir.join(MARKER));
}

pub fn info(message: impl AsRef<str>) {
    write("INFO", message.as_ref());
}

pub fn warn(message: impl AsRef<str>) {
    write("WARN", message.as_ref());
}

pub fn error(message: impl AsRef<str>) {
    write("ERROR", message.as_ref());
}

fn lock() -> std::sync::MutexGuard<'static, Option<Sink>> {
    SINK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn write(level: &str, message: &str) {
    let mut guard = lock();
    let Some(sink) = guard.as_mut() else { return };
    let line = format!("{} {level} {message}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"));
    if sink.written + line.len() as u64 > MAX_BYTES {
        rotate(sink);
    }
    if sink.file.write_all(line.as_bytes()).is_ok() {
        sink.written += line.len() as u64;
    }
}

/// Keeps the current and one previous file, so the log never grows past about 2 MB.
fn rotate(sink: &mut Sink) {
    let old = sink.path.with_extension("log.1");
    let _ = fs::rename(&sink.path, &old);
    if let Ok(file) = OpenOptions::new().create(true).append(true).open(&sink.path) {
        sink.file = file;
        sink.written = 0;
    }
}

/// With `panic = "abort"` a panic anywhere ends the whole app, so the message and location are
/// written before the process goes.
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let thread = std::thread::current();
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "(no message)".into());
        let location = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
        error(format!(
            "PANIC in thread '{}' at {location}: {message}\n{}",
            thread.name().unwrap_or("?"),
            std::backtrace::Backtrace::force_capture()
        ));
        previous(info);
    }));
}

#[cfg(test)]
#[path = "tests/applog.rs"]
mod tests;
