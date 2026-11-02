//! Running sing-box on Windows. Proxy Only mode runs it as a hidden child of the app inside a Job
//! Object that kills it when the app's last handle closes, so a crashed app never leaves a proxy
//! running. A watcher thread does what `supervisor.sh` does on macOS: it ends the run when the
//! stop file appears, notes how sing-box exited, and puts the system proxy back if sing-box died
//! on its own. TUN mode needs the helper service (phase 2) and is refused until then.

pub mod daemon;

use std::fs::OpenOptions;
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::Duration;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, STILL_ACTIVE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows_sys::Win32::System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

use crate::applog;
use crate::config::Mode;
use crate::error::{AppError, Result};
use crate::paths::AppPaths;

/// No console window for sing-box.
pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const POLL: Duration = Duration::from_millis(500);
const MAX_STDERR_LOG: u64 = 1_000_000;
pub(crate) const TUN_UNAVAILABLE: &str = "TUN mode is not available on Windows yet. Use Proxy Only mode.";

/// The Job Object of the current run. Kept open for the run's lifetime: when the app exits, Windows
/// closes it and the job kills sing-box.
struct Job(HANDLE);
// SAFETY: a job handle is a kernel object handle, usable from any thread; it is only closed in Drop.
unsafe impl Send for Job {}
impl Drop for Job {
    fn drop(&mut self) {
        // SAFETY: the handle came from CreateJobObjectW and is closed exactly once.
        unsafe { CloseHandle(self.0) };
    }
}

static JOB: Mutex<Option<Job>> = Mutex::new(None);

/// Nothing to forget: Windows has no once-per-launch service repairs yet.
pub fn forget_repair_attempts() {}

pub async fn start(paths: &AppPaths, binary: &Path, mode: Mode, _kill_switch_ips: Option<String>, _cancel: &AtomicBool) -> Result<u32> {
    match mode {
        Mode::Tun => Err(AppError::Launch(TUN_UNAVAILABLE.into())),
        Mode::Proxy => start_plain(paths, binary),
    }
}

fn start_plain(paths: &AppPaths, binary: &Path) -> Result<u32> {
    let _ = std::fs::remove_file(paths.stop_file());
    let errlog = paths.config_file().with_file_name("sing-box.stderr.log");
    if std::fs::metadata(&errlog).is_ok_and(|m| m.len() > MAX_STDERR_LOG) {
        let _ = std::fs::remove_file(&errlog);
    }
    let out = OpenOptions::new().create(true).append(true).open(&errlog)?;
    let child = Command::new(binary)
        .args(["run", "-c"])
        .arg(paths.config_file())
        .stdin(Stdio::null())
        .stdout(out.try_clone()?)
        .stderr(out)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;
    let pid = child.id();
    *JOB.lock().unwrap() = Some(job_for(&child)?);
    let (stop, errlog, restore) = (paths.stop_file(), errlog.clone(), paths.clone());
    std::thread::spawn(move || watch(child, &stop, &errlog, &restore));
    Ok(pid)
}

/// A job that kills its processes when its last handle closes, with `child` in it.
fn job_for(child: &Child) -> Result<Job> {
    use std::os::windows::io::AsRawHandle;
    // SAFETY: plain Win32 calls on a job handle this function owns and the child's live process
    // handle; the limit struct is zeroed and then filled in before it is passed by pointer.
    unsafe {
        let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if job.is_null() {
            return Err(AppError::Launch("could not create a job object for sing-box".into()));
        }
        let job = Job(job);
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let size = std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32;
        if SetInformationJobObject(job.0, JobObjectExtendedLimitInformation, (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(), size) == 0
            || AssignProcessToJobObject(job.0, child.as_raw_handle() as HANDLE) == 0
        {
            return Err(AppError::Launch("could not tie sing-box to the app".into()));
        }
        Ok(job)
    }
}

/// Waits for the run to end, by the stop file or by sing-box exiting, and tidies up after it.
fn watch(mut child: Child, stop: &Path, errlog: &Path, paths: &AppPaths) {
    let status = loop {
        if let Ok(Some(status)) = child.try_wait() {
            break Some(status);
        }
        if stop.exists() {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(POLL);
    };
    if let Some(status) = status {
        let stopped = stop.exists();
        let line = format!("{} sing-box exited with {status} (stop requested: {})\n", chrono::Local::now().format("%F %T"), if stopped { "yes" } else { "no" });
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(errlog) {
            let _ = file.write_all(line.as_bytes());
        }
        if !stopped {
            applog::warn(line.trim_end());
            crate::sysproxy::restore_after_crash(paths);
        }
    }
    JOB.lock().unwrap().take();
    let _ = std::fs::remove_file(stop);
}

pub fn request_stop(paths: &AppPaths) -> Result<()> {
    std::fs::write(paths.stop_file(), b"")?;
    Ok(())
}

/// There is no kill-switch block to keep on Windows yet, so this is a plain stop.
pub fn request_down(paths: &AppPaths) -> Result<()> {
    request_stop(paths)
}

/// Only the TUN helper holds a kill-switch block; there is none on Windows yet.
pub fn signal_ready(_paths: &AppPaths) -> Result<()> {
    Ok(())
}

pub fn signal_block(_paths: &AppPaths) -> Result<()> {
    Ok(())
}

pub fn is_alive(pid: u32) -> bool {
    // SAFETY: the handle is checked and closed; GetExitCodeProcess only writes `code`.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return false;
        }
        let mut code = 0u32;
        let ok = GetExitCodeProcess(handle, &mut code) != 0;
        CloseHandle(handle);
        ok && code == STILL_ACTIVE as u32
    }
}

/// There is no administrator prompt for shell commands on Windows; the helper service (phase 2)
/// takes over what needs one.
pub(crate) async fn run_admin_script(_command: &str, _prompt: &str) -> Result<String> {
    Err(AppError::Launch("this needs the Kage helper service, which is not available on Windows yet".into()))
}
