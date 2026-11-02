//! The Windows helper service that runs sing-box for TUN mode. It comes in phase 2; until then it
//! is never installed and installing it says why.

use std::path::Path;
use std::sync::atomic::AtomicBool;

use crate::error::{AppError, Result};
use crate::paths::AppPaths;

pub fn installed() -> bool {
    false
}

pub async fn install(_paths: &AppPaths, _binary: &Path, _cancel: Option<&AtomicBool>) -> Result<()> {
    Err(AppError::Launch(crate::platform::TUN_UNAVAILABLE.into()))
}

pub async fn uninstall() -> Result<()> {
    Ok(())
}
