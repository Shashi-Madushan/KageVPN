use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{AppError, Result};
use crate::logs::strip_ansi;

const KNOWN_LOCATIONS: [&str; 2] = ["/opt/homebrew/bin/sing-box", "/usr/local/bin/sing-box"];

pub fn locate() -> Result<PathBuf> {
    let on_path: Vec<PathBuf> = env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).map(|dir| dir.join("sing-box")).collect())
        .unwrap_or_default();

    KNOWN_LOCATIONS
        .iter()
        .map(PathBuf::from)
        .chain(on_path)
        .find(|candidate| candidate.is_file())
        .ok_or(AppError::SingBoxMissing)
}

pub fn check(binary: &Path, config: &Path) -> Result<()> {
    let output = Command::new(binary).arg("check").arg("-c").arg(config).output()?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = strip_ansi(&String::from_utf8_lossy(&output.stderr));
    Err(AppError::ConfigCheck(stderr.trim().to_string()))
}
