//! A resource sample for `app.log` every 30 seconds: memory and CPU of the app and sing-box,
//! open file descriptors against the limit, and the number of live connections. If the app
//! restarts under heavy traffic, the last samples before the restart show what was running out.

use std::process::Command;
use std::time::Duration;

use tauri::{AppHandle, Manager};
use tokio::time::sleep;

use crate::state::AppState;
use crate::{applog, clash};

const INTERVAL: Duration = Duration::from_secs(30);
/// macOS apps start with a soft limit of 256 open files, which heavy traffic can exhaust.
const WANTED_FD_LIMIT: u64 = 10_240;

/// Raises the soft file-descriptor limit as far as the system allows, up to `WANTED_FD_LIMIT`.
pub fn raise_fd_limit() {
    // SAFETY: getrlimit/setrlimit only read and write the `rlimit` struct passed to them.
    unsafe {
        let mut limit = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        if libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) != 0 {
            return;
        }
        let before = limit.rlim_cur;
        limit.rlim_cur = WANTED_FD_LIMIT.min(limit.rlim_max);
        if limit.rlim_cur > before && libc::setrlimit(libc::RLIMIT_NOFILE, &limit) == 0 {
            applog::info(format!("open-file limit raised from {before} to {}", limit.rlim_cur));
        }
    }
}

fn open_fds(pid: u32) -> Option<usize> {
    // SAFETY: with a null buffer proc_pidinfo only reports how many bytes the list would need.
    let bytes = unsafe { libc::proc_pidinfo(pid as i32, libc::PROC_PIDLISTFDS, 0, std::ptr::null_mut(), 0) };
    (bytes > 0).then(|| bytes as usize / std::mem::size_of::<libc::proc_fdinfo>())
}

fn fd_limit() -> u64 {
    let mut limit = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
    // SAFETY: getrlimit only writes to the struct passed to it.
    unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) };
    limit.rlim_cur
}

/// `"rss=12MB cpu=0.4%"` for one process, from `ps`.
fn usage_of(pid: u32) -> String {
    let output = Command::new("ps").args(["-o", "rss=,%cpu=", "-p", &pid.to_string()]).output();
    let text = output.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    let mut fields = text.split_whitespace();
    match (fields.next().and_then(|v| v.parse::<u64>().ok()), fields.next()) {
        (Some(kb), Some(cpu)) => format!("rss={}MB cpu={cpu}%", kb / 1024),
        _ => "gone".into(),
    }
}

pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            sleep(INTERVAL).await;
            let own = std::process::id();
            let core = *app.state::<AppState>().pid.lock().unwrap();
            let mut line = format!(
                "sample app[{}] fds={}/{}",
                usage_of(own),
                open_fds(own).map_or("?".into(), |n| n.to_string()),
                fd_limit()
            );
            if let Some(pid) = core {
                line.push_str(&format!(" supervisor[{}]", usage_of(pid)));
                if let Ok(connections) = clash::connections(&app).await {
                    line.push_str(&format!(" connections={}", connections.len()));
                }
            }
            applog::info(line);
        }
    });
}
