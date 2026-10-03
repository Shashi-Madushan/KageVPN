use std::path::Path;
use std::process::Command;

use crate::error::{AppError, Result};
use crate::logs::strip_ansi;

pub fn check(binary: &Path, config: &Path) -> Result<()> {
    let output = Command::new(binary).arg("check").arg("-c").arg(config).output()?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = strip_ansi(&String::from_utf8_lossy(&output.stderr));
    Err(AppError::ConfigCheck(stderr.trim().to_string()))
}
