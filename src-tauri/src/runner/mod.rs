use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use crate::config::Mode;
use crate::error::{AppError, Result};
use crate::paths::AppPaths;

const SUPERVISOR: &str = include_str!("supervisor.sh");
const HELPER: &str = include_str!("helper.sh");
const USER_CANCELED: &str = "-128";
const RUN_START_WAIT: Duration = Duration::from_secs(15);
const HELPER_QUIT_WAIT: Duration = Duration::from_secs(4);

/// The root helper for this app launch and the sing-box binary it was started for.
static ROOT_HELPER: Mutex<Option<(u32, PathBuf)>> = Mutex::new(None);

/// Starts sing-box and returns the pid whose lifetime is the connection's.
///
/// Proxy mode runs a plain per-connection supervisor. TUN mode needs root, so a helper is started
/// through one administrator prompt per app launch and reused for every later connect; stopping
/// is done with a stop file, so neither disconnecting nor reconnecting prompts again.
pub async fn start(paths: &AppPaths, binary: &Path, mode: Mode, kill_switch_ips: Option<String>) -> Result<u32> {
    match mode {
        Mode::Tun => start_with_root_helper(paths, binary, kill_switch_ips).await,
        Mode::Proxy => start_plain(paths, binary),
    }
}

async fn start_with_root_helper(paths: &AppPaths, binary: &Path, kill_switch_ips: Option<String>) -> Result<u32> {
    ensure_helper(paths, binary).await?;

    let _ = std::fs::remove_file(paths.child_pid_file());
    let command = match kill_switch_ips {
        None => "none".to_string(),
        Some(ips) if ips.is_empty() => "-".to_string(),
        Some(ips) => ips,
    };
    let pending = paths.helper_start_file().with_extension("tmp");
    std::fs::write(&pending, command)?;
    std::fs::rename(&pending, paths.helper_start_file())?;

    let waits = RUN_START_WAIT.as_millis() / 100;
    for _ in 0..waits {
        if let Some(pid) = std::fs::read_to_string(paths.child_pid_file()).ok().and_then(|t| t.trim().parse().ok()) {
            return Ok(pid);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(AppError::Launch("the privileged helper did not start sing-box".into()))
}

/// Makes sure a root helper for `binary` is running, asking for the administrator password only
/// when there is none yet (first connect after launch, or the engine changed).
async fn ensure_helper(paths: &AppPaths, binary: &Path) -> Result<()> {
    let current = ROOT_HELPER.lock().unwrap().clone();
    if let Some((pid, running_binary)) = current {
        if is_alive(pid) && running_binary == binary {
            return Ok(());
        }
        if is_alive(pid) {
            std::fs::write(paths.helper_quit_file(), b"")?;
            let waits = HELPER_QUIT_WAIT.as_millis() / 100;
            for _ in 0..waits {
                if !is_alive(pid) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        let _ = std::fs::remove_file(paths.helper_quit_file());
    }

    let script = paths.helper_script();
    std::fs::write(&script, HELPER)?;
    let args = vec![
        script.display().to_string(),
        binary.display().to_string(),
        paths.config_file().display().to_string(),
        paths.data_dir().display().to_string(),
        std::process::id().to_string(),
    ];
    let pid = spawn_privileged(&args).await?;
    *ROOT_HELPER.lock().unwrap() = Some((pid, binary.to_path_buf()));
    Ok(())
}

fn start_plain(paths: &AppPaths, binary: &Path) -> Result<u32> {
    let script = paths.supervisor_script();
    std::fs::write(&script, SUPERVISOR)?;

    let args = vec![
        script.display().to_string(),
        binary.display().to_string(),
        paths.config_file().display().to_string(),
        paths.stop_file().display().to_string(),
        std::process::id().to_string(),
    ];
    spawn_plain(&args)
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
