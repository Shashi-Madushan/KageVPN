//! The app's side of the helper pipe. The connection stays open for the whole TUN run: closing it,
//! by disconnecting, quitting or crashing, is what makes the service stop sing-box.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::error::{AppError, Result};
use crate::winservice::{Reply, Request, PIPE_NAME};

const CONNECT_WAIT: Duration = Duration::from_secs(10);

static PIPE: Mutex<Option<File>> = Mutex::new(None);

/// Whether a TUN run is held open through the service.
pub fn is_connected() -> bool {
    PIPE.lock().unwrap().is_some()
}

/// Sends the configuration to the service and returns the pid of the sing-box it started.
pub fn start(config: serde_json::Value) -> Result<u32> {
    let mut pipe = connect()?;
    let reply = ask(&mut pipe, &Request::Start { config })?;
    let pid = reply.pid.ok_or_else(|| AppError::Launch("the helper did not report sing-box's pid".into()))?;
    *PIPE.lock().unwrap() = Some(pipe);
    Ok(pid)
}

/// Stops the run. Closing the pipe would do it; the request just makes it prompt and explicit.
pub fn stop() {
    if let Some(mut pipe) = PIPE.lock().unwrap().take() {
        let _ = ask(&mut pipe, &Request::Stop);
    }
}

/// The service may still be starting, so the pipe is retried for a few seconds.
fn connect() -> Result<File> {
    let started = Instant::now();
    loop {
        match OpenOptions::new().read(true).write(true).open(PIPE_NAME) {
            Ok(pipe) => return Ok(pipe),
            Err(error) if started.elapsed() < CONNECT_WAIT => {
                let _ = error;
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(error) => return Err(AppError::Launch(format!("could not reach the Kage helper service: {error}"))),
        }
    }
}

fn ask(pipe: &mut File, request: &Request) -> Result<Reply> {
    let mut line = serde_json::to_string(request).map_err(|e| AppError::Launch(e.to_string()))?;
    line.push('\n');
    pipe.write_all(line.as_bytes())?;
    let mut answer = String::new();
    BufReader::new(&*pipe).read_line(&mut answer)?;
    let reply: Reply = serde_json::from_str(answer.trim()).map_err(|_| AppError::Launch("the Kage helper service sent an unreadable answer".into()))?;
    if reply.ok {
        Ok(reply)
    } else {
        Err(AppError::Launch(reply.error.unwrap_or_else(|| "the Kage helper service refused the request".into())))
    }
}
