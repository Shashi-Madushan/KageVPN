use std::path::Path;
use std::process::{Command, Stdio};

use crate::config::Mode;
use crate::error::{AppError, Result};
use crate::paths::AppPaths;

const SUPERVISOR: &str = include_str!("supervisor.sh");
const USER_CANCELED: &str = "-128";

/// Starts the supervisor script and returns its pid. TUN mode needs root, so the
/// script is launched through one administrator prompt; stopping is done with a
/// stop file so disconnecting never prompts again.
pub async fn start(paths: &AppPaths, binary: &Path, mode: Mode, kill_switch_ips: Option<String>) -> Result<u32> {
    let script = paths.supervisor_script();
    std::fs::write(&script, SUPERVISOR)?;

    let mut args = vec![
        script.display().to_string(),
        binary.display().to_string(),
        paths.config_file().display().to_string(),
        paths.stop_file().display().to_string(),
        std::process::id().to_string(),
    ];
    if let (Mode::Tun, Some(ips)) = (mode, kill_switch_ips) {
        args.push(if ips.is_empty() { "-".to_string() } else { ips });
    }

    match mode {
        Mode::Proxy => spawn_plain(&args),
        Mode::Tun => spawn_privileged(&args).await,
    }
}

pub fn request_stop(paths: &AppPaths) -> Result<()> {
    std::fs::write(paths.stop_file(), b"")?;
    Ok(())
}

pub fn is_alive(pid: u32) -> bool {
    Command::new("ps")
        .args(["-o", "stat=", "-p", &pid.to_string()])
        .output()
        .map(|out| {
            let stat = String::from_utf8_lossy(&out.stdout);
            let stat = stat.trim();
            !stat.is_empty() && !stat.starts_with('Z')
        })
        .unwrap_or(false)
}

fn spawn_plain(args: &[String]) -> Result<u32> {
    let child = Command::new("/bin/sh")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(child.id())
}

async fn spawn_privileged(args: &[String]) -> Result<u32> {
    let command = format!(
        "/bin/sh {} >/dev/null 2>&1 </dev/null & echo $!",
        args.iter().map(|a| shell_quote(a)).collect::<Vec<_>>().join(" ")
    );
    let script = format!(
        "do shell script \"{}\" with administrator privileges",
        applescript_escape(&command)
    );

    let output = tauri::async_runtime::spawn_blocking(move || {
        Command::new("osascript").arg("-e").arg(script).output()
    })
    .await
    .map_err(|e| AppError::Launch(e.to_string()))??;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(if stderr.contains(USER_CANCELED) {
            AppError::PermissionDenied
        } else {
            AppError::Launch(stderr.trim().to_string())
        });
    }
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .map_err(|_| AppError::Launch("could not read the started process id".into()))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

fn applescript_escape(value: &str) -> String {
    value.replace('\\', r"\\").replace('"', r#"\""#)
}

#[cfg(test)]
#[path = "../tests/runner.rs"]
mod tests;
