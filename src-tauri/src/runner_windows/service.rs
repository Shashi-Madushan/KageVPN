//! The Kage helper service (`kage-vpn.exe --service`), which runs sing-box as SYSTEM for TUN mode.
//! It listens on one named pipe that only SYSTEM, Administrators and the installing user can open.
//! The app sends the configuration; the service checks it, runs only its own copy of the engine
//! from the admin-only `%ProgramData%\Kage` folder, and stops sing-box when the app disconnects.

use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::windows::io::{FromRawHandle, RawHandle};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use sha2::{Digest, Sha256};
use windows_service::service::{ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::{define_windows_service, service_dispatcher};
use windows_sys::Win32::Foundation::{LocalFree, ERROR_PIPE_CONNECTED, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Security::Authorization::{ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1};
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
use windows_sys::Win32::System::Pipes::{ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE, PIPE_WAIT};

use super::{job_for, Job, CREATE_NO_WINDOW};
use crate::winservice::{self, Reply, Request, PIPE_NAME, SERVICE_NAME};

define_windows_service!(ffi_service_main, service_main);

/// The sing-box run, if there is one. Dropping the job kills the process.
static RUN: Mutex<Option<(Child, Job)>> = Mutex::new(None);

/// Where the service keeps its files: `%ProgramData%\Kage`.
pub fn install_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("ProgramData").unwrap_or_else(|| r"C:\ProgramData".into())).join("Kage")
}

/// Called from `main` when Windows starts the service.
pub fn run() -> Result<(), String> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main).map_err(|e| e.to_string())
}

fn service_main(_args: Vec<OsString>) {
    let handler = service_control_handler::register(SERVICE_NAME, |control| match control {
        ServiceControl::Stop | ServiceControl::Shutdown => {
            // Windows closes the job with the process, which stops sing-box.
            std::process::exit(0);
        }
        ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
        _ => ServiceControlHandlerResult::NotImplemented,
    });
    let Ok(handler) = handler else { return };
    let _ = handler.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::ZERO,
        process_id: None,
    });
    let _ = serve_forever();
}

fn user_sid() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    let sid = args.iter().position(|a| a == "--user-sid").and_then(|i| args.get(i + 1))?;
    winservice::is_sid(sid).then(|| sid.clone())
}

fn serve_forever() -> Result<(), String> {
    let sddl = winservice::pipe_sddl(&user_sid().ok_or("no user SID")?).ok_or("bad SID")?;
    loop {
        let pipe = create_pipe(&sddl)?;
        handle_client(pipe);
        stop_run();
    }
}

/// One pipe instance with the access list applied, waiting for the app to connect.
fn create_pipe(sddl: &str) -> Result<File, String> {
    let wide = |s: &str| -> Vec<u16> { s.encode_utf16().chain(Some(0)).collect() };
    // SAFETY: the descriptor is allocated by Windows, used for this one CreateNamedPipeW call and
    // freed after it; every pointer passed is to a live local.
    unsafe {
        let mut descriptor = std::ptr::null_mut();
        if ConvertStringSecurityDescriptorToSecurityDescriptorW(wide(sddl).as_ptr(), SDDL_REVISION_1, &mut descriptor, std::ptr::null_mut()) == 0 {
            return Err("could not build the pipe's access list".into());
        }
        let attributes = SECURITY_ATTRIBUTES { nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32, lpSecurityDescriptor: descriptor, bInheritHandle: 0 };
        let handle = CreateNamedPipeW(
            wide(PIPE_NAME).as_ptr(),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            1,
            64 * 1024,
            64 * 1024,
            0,
            &attributes,
        );
        LocalFree(descriptor);
        if handle == INVALID_HANDLE_VALUE {
            return Err("could not create the pipe".into());
        }
        // A client that connected before this call is reported as an error that is really success.
        if ConnectNamedPipe(handle, std::ptr::null_mut()) == 0 && windows_sys::Win32::Foundation::GetLastError() != ERROR_PIPE_CONNECTED {
            drop(File::from_raw_handle(handle as RawHandle));
            return Err("could not wait for the app".into());
        }
        Ok(File::from_raw_handle(handle as RawHandle))
    }
}

/// Answers requests until the app disconnects.
fn handle_client(pipe: File) {
    let mut reader = BufReader::new(&pipe);
    let mut writer = &pipe;
    let mut line = String::new();
    loop {
        line.clear();
        // Reading ends (0 bytes or an error) when the app quits or crashes.
        if !matches!(reader.read_line(&mut line), Ok(n) if n > 0) {
            break;
        }
        let reply = match winservice::parse_request(&line) {
            Ok(request) => handle(request),
            Err(error) => Reply::fail(error),
        };
        let Ok(mut text) = serde_json::to_string(&reply) else { break };
        text.push('\n');
        if writer.write_all(text.as_bytes()).is_err() {
            break;
        }
    }
    // SAFETY: the handle is the pipe's own; disconnecting it before it is closed is the normal end.
    unsafe { DisconnectNamedPipe(std::os::windows::io::AsRawHandle::as_raw_handle(&pipe) as _) };
}

fn handle(request: Request) -> Reply {
    match request {
        Request::Stop => {
            stop_run();
            Reply::ok()
        }
        Request::Start { config } => match start_run(&config) {
            Ok(pid) => Reply { ok: true, pid: Some(pid), ..Reply::default() },
            Err(error) => Reply::fail(error),
        },
    }
}

fn stop_run() {
    if let Some((mut child, _job)) = RUN.lock().unwrap().take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn start_run(config: &serde_json::Value) -> Result<u32, String> {
    stop_run();
    winservice::validate_config(config)?;
    let dir = install_dir();
    let engine = dir.join("engine").join("sing-box.exe");
    let recorded = std::fs::read_to_string(dir.join("engine").join("sing-box.sha256")).map_err(|_| "the helper's engine is not installed")?;
    if sha256_file(&engine)? != recorded.trim() {
        return Err("the helper's engine does not match the recorded hash; use Repair in Diagnostics".into());
    }
    let run_dir = dir.join("run");
    let log = run_dir.join("sing-box.log");
    let config = winservice::with_service_log(config.clone(), &log.to_string_lossy());
    let config_file = run_dir.join("config.json");
    std::fs::write(&config_file, serde_json::to_vec(&config).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let out = OpenOptions::new().create(true).append(true).open(run_dir.join("sing-box.stderr.log")).map_err(|e| e.to_string())?;
    let mut child = Command::new(&engine)
        .args(["run", "-c"])
        .arg(&config_file)
        .current_dir(&run_dir)
        .stdin(Stdio::null())
        .stdout(out.try_clone().map_err(|e| e.to_string())?)
        .stderr(out)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("could not start sing-box: {e}"))?;
    let job = job_for(&child).map_err(|e| e.to_string())?;
    // A configuration sing-box rejects makes it exit at once; say so instead of reporting a pid.
    std::thread::sleep(Duration::from_secs(1));
    if let Ok(Some(status)) = child.try_wait() {
        return Err(format!("sing-box exited at once ({status}); see {}", run_dir.join("sing-box.stderr.log").display()));
    }
    let pid = child.id();
    *RUN.lock().unwrap() = Some((child, job));
    Ok(pid)
}

pub fn sha256_file(path: &std::path::Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;
    Ok(hasher.finalize().iter().map(|b| format!("{b:02x}")).collect())
}
