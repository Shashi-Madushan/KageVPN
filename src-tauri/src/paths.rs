use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

/// Moves the data folder an earlier bundle identifier used to `new_dir`, once, so renaming the
/// app keeps saved servers, settings, usage history and downloaded cores. Does nothing if
/// `new_dir` already has content or no legacy folder exists. Returns whether anything moved.
pub fn adopt_legacy_dir(new_dir: &Path, legacy_dirs: &[PathBuf]) -> io::Result<bool> {
    let in_use = fs::read_dir(new_dir).map(|mut entries| entries.next().is_some()).unwrap_or(false);
    if in_use {
        return Ok(false);
    }
    let Some(legacy) = legacy_dirs.iter().find(|dir| dir.is_dir() && dir.as_path() != new_dir) else {
        return Ok(false);
    };
    if let Some(parent) = new_dir.parent() {
        fs::create_dir_all(parent)?;
    }
    let _ = fs::remove_dir(new_dir); // an empty folder would block the move
    fs::rename(legacy, new_dir)?;
    Ok(true)
}

#[derive(Clone)]
pub struct AppPaths {
    data_dir: PathBuf,
}

impl AppPaths {
    pub fn new(data_dir: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&data_dir)?;
        Ok(Self { data_dir })
    }

    pub fn data_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }

    pub fn store_file(&self) -> PathBuf {
        self.data_dir.join("data.json")
    }

    pub fn core_dir(&self) -> PathBuf {
        self.data_dir.join("core")
    }

    /// Where one core's downloaded versions live: `core/<id>/<version>/`.
    pub fn core_dir_for(&self, core_id: &str) -> PathBuf {
        self.core_dir().join(core_id)
    }

    pub fn usage_db(&self) -> PathBuf {
        self.data_dir.join("usage.db")
    }

    /// Where the country and category lists live, one `.srs` file per list.
    pub fn rule_sets_dir(&self) -> PathBuf {
        self.data_dir.join("rule-sets")
    }

    pub fn config_file(&self) -> PathBuf {
        self.data_dir.join("config.json")
    }

    pub fn log_file(&self) -> PathBuf {
        self.data_dir.join("sing-box.log")
    }

    /// sing-box's stderr and exit status, written by the supervisor or helper script.
    pub fn stderr_log(&self) -> PathBuf {
        self.data_dir.join("sing-box.stderr.log")
    }

    pub fn stop_file(&self) -> PathBuf {
        self.data_dir.join("stop")
    }

    /// Like `stop_file`, but the root helper keeps the kill-switch block up; see `runner/helper.sh`.
    pub fn down_file(&self) -> PathBuf {
        self.data_dir.join("down")
    }

    /// The server addresses the kill switch lets through, refreshed by the app; see `allowlist`.
    pub fn allow_file(&self) -> PathBuf {
        self.data_dir.join("allow")
    }

    pub fn helper_script(&self) -> PathBuf {
        self.data_dir.join("helper.sh")
    }

    /// Written by the app to ask the root helper for a run; see `runner/helper.sh`.
    pub fn helper_start_file(&self) -> PathBuf {
        self.data_dir.join("helper.start")
    }

    pub fn helper_quit_file(&self) -> PathBuf {
        self.data_dir.join("helper.quit")
    }

    /// Written by the root helper with the pid of the sing-box it started.
    pub fn child_pid_file(&self) -> PathBuf {
        self.data_dir.join("child.pid")
    }

    pub fn supervisor_script(&self) -> PathBuf {
        self.data_dir.join("supervisor.sh")
    }

    /// The log file is created by the user so a root-owned sing-box can append to it
    /// without leaving a file the app can no longer truncate.
    pub fn reset_runtime_files(&self) -> io::Result<()> {
        // Keep the last run's log as `sing-box.log.prev`: it is the evidence when a run ended badly.
        let log = self.log_file();
        if fs::metadata(&log).map(|m| m.len() > 0).unwrap_or(false) {
            let _ = fs::rename(&log, log.with_extension("log.prev"));
        }
        File::create(log)?;
        match fs::remove_file(self.stop_file()) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
#[path = "tests/paths.rs"]
mod tests;
