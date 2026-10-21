pub mod daemon;

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use crate::applog;
use crate::config::Mode;
use crate::error::{AppError, Result};
use crate::paths::AppPaths;

const SUPERVISOR: &str = include_str!("supervisor.sh");
const HELPER: &str = include_str!("helper.sh");
const USER_CANCELED: &str = "-128";
const RUN_START_WAIT: Duration = Duration::from_secs(15);
const HELPER_QUIT_WAIT: Duration = Duration::from_secs(4);

/// Set once the app has tried to refresh an out-of-date root service script in this launch, so a
/// declined password prompt is not repeated on every connect.
static SCRIPT_UPDATE_TRIED: AtomicBool = AtomicBool::new(false);

/// Set once the app has reinstalled a root service that did not answer, for the same reason.
static SERVICE_REPAIR_TRIED: AtomicBool = AtomicBool::new(false);

/// Lets the next connect try the once-per-launch service updates and reinstalls again.
pub fn forget_repair_attempts() {
    SCRIPT_UPDATE_TRIED.store(false, Ordering::SeqCst);
    SERVICE_REPAIR_TRIED.store(false, Ordering::SeqCst);
}

/// The root helper for this app launch and the sing-box binary it was started for.
static ROOT_HELPER: Mutex<Option<(u32, PathBuf)>> = Mutex::new(None);

/// Starts sing-box and returns the pid whose lifetime is the connection's.
///
/// Proxy mode runs a plain per-connection supervisor. TUN mode needs root, so a helper is started
/// through one administrator prompt per app launch and reused for every later connect; stopping
/// is done with a stop file, so neither disconnecting nor reconnecting prompts again.
///
/// `cancel` is the user's "stop connecting": the administrator prompt and the wait for the helper
/// both end as soon as it is set, with `Cancelled`.
pub async fn start(paths: &AppPaths, binary: &Path, mode: Mode, kill_switch_ips: Option<String>, cancel: &AtomicBool) -> Result<u32> {
    match mode {
        Mode::Tun => start_with_root_helper(paths, binary, kill_switch_ips, cancel).await,
        Mode::Proxy => start_plain(paths, binary),
    }
}

async fn start_with_root_helper(paths: &AppPaths, binary: &Path, kill_switch_ips: Option<String>, cancel: &AtomicBool) -> Result<u32> {
    let service = daemon::installed();
    if service {
        if !daemon::binary_current(binary) {
            daemon::install(paths, binary, Some(cancel)).await?;
        } else if !daemon::script_current() && !SCRIPT_UPDATE_TRIED.swap(true, Ordering::SeqCst) {
            // The service that is running still works, so a declined or failed update must not stop
            // the connection; it is tried again next launch.
            applog::info("root service script is out of date: asking to update it");
            if let Err(error) = daemon::install(paths, binary, Some(cancel)).await {
                if error.is_cancelled() {
                    return Err(error);
                }
                applog::warn(format!("could not update the root service script: {error}"));
            }
        }
    } else {
        ensure_helper(paths, binary, cancel).await?;
    }

    let command = match kill_switch_ips {
        None => "none".to_string(),
        Some(ips) if ips.is_empty() => "-".to_string(),
        Some(ips) => ips,
    };
    if let Some(pid) = request_start(paths, &command, RUN_START_WAIT, cancel).await? {
        return Ok(pid);
    }
    // The service's files are there but nothing answered, which is what a service that was
    // unloaded (an OS update can do it) looks like. Installing it again starts it. This is tried
    // once per launch, so a service that cannot be revived costs one extra password prompt.
    if service && !SERVICE_REPAIR_TRIED.swap(true, Ordering::SeqCst) {
        applog::warn("the root service did not answer: installing it again");
        daemon::install(paths, binary, Some(cancel)).await?;
        if let Some(pid) = request_start(paths, &command, RUN_START_WAIT, cancel).await? {
            return Ok(pid);
        }
    }
    // Give up cleanly: a helper that wakes up later must not find this request and start sing-box
    // as root while the app thinks it is disconnected.
    let _ = std::fs::remove_file(paths.helper_start_file());
    Err(AppError::Launch("the privileged helper did not start sing-box".into()))
}

/// Asks the root helper for a run and waits up to `wait` for the pid it reports, or `None` when it
/// stays silent. Cancelling takes the request back, so a helper that wakes up later does not start
/// sing-box for a connection nobody wants any more.
async fn request_start(paths: &AppPaths, command: &str, wait: Duration, cancel: &AtomicBool) -> Result<Option<u32>> {
    let _ = std::fs::remove_file(paths.child_pid_file());
    let pending = paths.helper_start_file().with_extension("tmp");
    // Line 2 names this app, so the root service stops sing-box if the app disappears.
    std::fs::write(&pending, format!("{command}\n{}\n", std::process::id()))?;
    std::fs::rename(&pending, paths.helper_start_file())?;

    let waits = wait.as_millis() / 100;
    for _ in 0..waits {
        if let Some(pid) = std::fs::read_to_string(paths.child_pid_file()).ok().and_then(|t| t.trim().parse().ok()) {
            return Ok(Some(pid));
        }
        if cancel.load(Ordering::SeqCst) {
            let _ = std::fs::remove_file(paths.helper_start_file());
            return Err(AppError::Cancelled);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Ok(None)
}

/// Makes sure a root helper for `binary` is running, asking for the administrator password only
/// when there is none yet (first connect after launch, or the engine changed).
async fn ensure_helper(paths: &AppPaths, binary: &Path, cancel: &AtomicBool) -> Result<()> {
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
    let pid = spawn_privileged(&args, cancel).await?;
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

/// Ends the run but keeps the kill-switch block up, for a run that failed or is being replaced
/// while the user still wants no traffic outside the tunnel. Only the root helper (TUN mode) holds
/// a block; anywhere else this is a plain stop.
pub fn request_down(paths: &AppPaths) -> Result<()> {
    std::fs::write(paths.down_file(), b"")?;
    Ok(())
}

/// Tells the root helper the run is verified, so it takes down a kill-switch block it was holding.
pub fn signal_ready(paths: &AppPaths) -> Result<()> {
    std::fs::write(paths.ready_file(), b"")?;
    Ok(())
}

/// Tells the root helper the tunnel is not carrying traffic, so it puts the kill-switch block up.
pub fn signal_block(paths: &AppPaths) -> Result<()> {
    std::fs::write(paths.block_file(), b"")?;
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

const ADMIN_PROMPT: &str = "Kage needs administrator access to create the VPN tunnel. Install the VPN service in Settings to avoid being asked each time you open the app.";

async fn spawn_privileged(args: &[String], cancel: &AtomicBool) -> Result<u32> {
    let command = format!(
        "/bin/sh {} >/dev/null 2>&1 </dev/null & echo $!",
        args.iter().map(|a| shell_quote(a)).collect::<Vec<_>>().join(" ")
    );
    run_admin_script_until(&command, ADMIN_PROMPT, Some(cancel))
        .await?
        .trim()
        .parse()
        .map_err(|_| AppError::Launch("could not read the started process id".into()))
}

/// Runs a shell command as root through the macOS administrator prompt, showing `prompt` as the
/// reason, and returns what it printed.
pub(crate) async fn run_admin_script(command: &str, prompt: &str) -> Result<String> {
    run_admin_script_until(command, prompt, None).await
}

/// Like `run_admin_script`, but gives up with `Cancelled` as soon as `cancel` is set. The prompt
/// is dismissed by ending `osascript`, so a cancelled connect never leaves a password dialog
/// behind that would start a root helper nobody is waiting for.
pub(crate) async fn run_admin_script_until(command: &str, prompt: &str, cancel: Option<&AtomicBool>) -> Result<String> {
    let script = format!(
        "do shell script \"{}\" with prompt \"{}\" with administrator privileges",
        applescript_escape(command),
        applescript_escape(prompt)
    );

    let child = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = wait_cancellable(child, cancel).await?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(if stderr.contains(USER_CANCELED) {
            AppError::PermissionDenied
        } else {
            AppError::Launch(stderr.trim().to_string())
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Waits for `child` to finish without blocking the runtime, and kills it when `cancel` is set.
async fn wait_cancellable(mut child: std::process::Child, cancel: Option<&AtomicBool>) -> Result<std::process::Output> {
    fn drain(pipe: Option<impl Read + Send + 'static>) -> std::thread::JoinHandle<Vec<u8>> {
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            if let Some(mut pipe) = pipe {
                let _ = pipe.read_to_end(&mut bytes);
            }
            bytes
        })
    }
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(std::process::Output { status, stdout: stdout.join().unwrap_or_default(), stderr: stderr.join().unwrap_or_default() });
        }
        if cancel.is_some_and(|flag| flag.load(Ordering::SeqCst)) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(AppError::Cancelled);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
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
