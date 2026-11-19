//! Installing, checking and removing the Kage helper service. Installing and removing need
//! administrator rights, so each is one PowerShell script run elevated (one UAC prompt). After the
//! install the signed-in user may start the service without a prompt, so connecting never asks.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use windows_service::service::{ServiceAccess, ServiceState};
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

use super::service::{install_dir, sha256_file};
use super::CREATE_NO_WINDOW;
use crate::error::{AppError, Result};
use crate::paths::AppPaths;
use crate::winservice::{self, SERVICE_NAME};

pub fn installed() -> bool {
    open_service(ServiceAccess::QUERY_STATUS).is_ok()
}

/// Whether the installed service runs this app's copy of `binary` (the engine) and this exe. If
/// not, the service is installed again, which also updates it after an app update.
pub fn binary_current(binary: &Path) -> bool {
    let dir = install_dir();
    let recorded = std::fs::read_to_string(dir.join("engine").join("sing-box.sha256")).unwrap_or_default();
    let same_engine = sha256_file(binary).is_ok_and(|hash| hash == recorded.trim());
    let same_exe = match (std::env::current_exe().and_then(std::fs::metadata), std::fs::metadata(dir.join("kage-vpn.exe"))) {
        (Ok(ours), Ok(installed)) => ours.len() == installed.len(),
        _ => false,
    };
    same_engine && same_exe
}

/// Installs or updates the service, asking Windows for permission once.
pub async fn install(_paths: &AppPaths, binary: &Path, cancel: Option<&AtomicBool>) -> Result<()> {
    let sha = sha256_file(binary).map_err(AppError::Launch)?;
    let exe = std::env::current_exe()?;
    let sid = user_sid()?;
    let script = winservice::install_script(&exe.to_string_lossy(), &binary.to_string_lossy(), &sha, &sid).map_err(AppError::Launch)?;
    run_elevated(&script, cancel).await?;
    if installed() {
        Ok(())
    } else {
        Err(AppError::Launch("the helper service is not there after the install".into()))
    }
}

pub async fn uninstall() -> Result<()> {
    if !installed() {
        return Ok(());
    }
    run_elevated(&winservice::uninstall_script(), None).await
}

/// Starts the service if it is stopped. The install lets the signed-in user do this.
pub fn ensure_running() -> Result<()> {
    let service = open_service(ServiceAccess::QUERY_STATUS | ServiceAccess::START).map_err(|e| AppError::Launch(format!("the Kage helper service is not available: {}", describe(&e))))?;
    let state = |s: &windows_service::service::Service| s.query_status().map(|status| status.current_state).unwrap_or(ServiceState::Stopped);
    if state(&service) == ServiceState::Stopped {
        service.start::<&str>(&[]).map_err(|e| AppError::Launch(format!("the Kage helper service did not start: {}", describe(&e))))?;
    }
    let started = Instant::now();
    while state(&service) != ServiceState::Running {
        if started.elapsed() > Duration::from_secs(10) {
            return Err(AppError::Launch("the Kage helper service did not start; try Repair in Diagnostics".into()));
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    Ok(())
}

/// The error with the Windows reason behind it: `windows-service` itself only says "IO error in
/// winapi call", and the code and text after it are what say what went wrong.
fn describe(error: &windows_service::Error) -> String {
    match std::error::Error::source(error) {
        Some(source) => format!("{error}: {source}"),
        None => error.to_string(),
    }
}

fn open_service(access: ServiceAccess) -> windows_service::Result<windows_service::service::Service> {
    ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?.open_service(SERVICE_NAME, access)
}

/// The signed-in user's SID, which the service lets onto its pipe.
fn user_sid() -> Result<String> {
    let out = Command::new("whoami").args(["/user", "/fo", "csv", "/nh"]).creation_flags(CREATE_NO_WINDOW).output()?;
    winservice::parse_whoami_sid(&String::from_utf8_lossy(&out.stdout)).ok_or_else(|| AppError::Launch("could not tell which Windows account this is".into()))
}

/// Runs `script` as administrator and waits. Declining the Windows prompt is `Cancelled`.
async fn run_elevated(script: &str, cancel: Option<&AtomicBool>) -> Result<()> {
    let launcher = format!(
        "$p = Start-Process powershell.exe -Verb RunAs -Wait -PassThru -WindowStyle Hidden -ArgumentList '-NoProfile','-EncodedCommand','{}'; exit $p.ExitCode",
        winservice::encode_script(script)
    );
    let mut child = Command::new("powershell.exe")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &launcher])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;
    loop {
        if let Some(status) = child.try_wait()? {
            if status.success() {
                return Ok(());
            }
            let mut text = String::new();
            if let Some(mut err) = child.stderr.take() {
                let _ = std::io::Read::read_to_string(&mut err, &mut text);
            }
            return Err(if text.to_lowercase().contains("canceled") || text.to_lowercase().contains("cancelled") {
                AppError::Cancelled
            } else {
                AppError::Launch(format!("installing the Kage helper service failed ({status})"))
            });
        }
        if cancel.is_some_and(|c| c.load(Ordering::SeqCst)) {
            let _ = child.kill();
            return Err(AppError::Cancelled);
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
