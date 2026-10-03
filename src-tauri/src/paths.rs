use std::fs::{self, File};
use std::io;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppPaths {
    data_dir: PathBuf,
}

impl AppPaths {
    pub fn new(data_dir: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&data_dir)?;
        Ok(Self { data_dir })
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

    pub fn cache_file(&self) -> PathBuf {
        self.data_dir.join("cache.db")
    }

    pub fn config_file(&self) -> PathBuf {
        self.data_dir.join("config.json")
    }

    pub fn log_file(&self) -> PathBuf {
        self.data_dir.join("sing-box.log")
    }

    pub fn stop_file(&self) -> PathBuf {
        self.data_dir.join("stop")
    }

    pub fn supervisor_script(&self) -> PathBuf {
        self.data_dir.join("supervisor.sh")
    }

    /// The log file is created by the user so a root-owned sing-box can append to it
    /// without leaving a file the app can no longer truncate.
    pub fn reset_runtime_files(&self) -> io::Result<()> {
        File::create(self.log_file())?;
        match fs::remove_file(self.stop_file()) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }
}
