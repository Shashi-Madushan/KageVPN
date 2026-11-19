//! `kagevpn://` links. Any web page can open one, and a browser may remember "always allow" for the
//! scheme, so a link must never be able to do something the person would not want without being
//! asked. Turning the VPN on is harmless. Turning it off exposes the traffic, so it needs a yes in a
//! dialog the page cannot click. Importing a server only fills the import box; the person still
//! presses Import.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

use crate::session::{self, State};
use crate::state::AppState;
use crate::{notify, reliability, tray};

/// How long the question waits before it counts as "no".
const ASK_SECONDS: u32 = 30;
/// A link cannot start the same action again this soon. A page that fires links in a loop gets one
/// answer, not a stack of dialogs or a flood of connection attempts.
const COOL_DOWN: Duration = Duration::from_secs(5);
/// The longest import text a link may hand to the import box.
const MAX_IMPORT_CHARS: usize = 200_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Connect,
    Disconnect,
    Toggle,
    Import,
}

/// What the link asks for, from its host part; `None` for anything else.
pub fn action_of(url: &url::Url) -> Option<Action> {
    if url.scheme() != "kagevpn" {
        return None;
    }
    match url.host_str().unwrap_or_default() {
        "connect" => Some(Action::Connect),
        "disconnect" => Some(Action::Disconnect),
        "toggle" => Some(Action::Toggle),
        "import" => Some(Action::Import),
        _ => None,
    }
}

/// Whether the action would end or cancel a connection, which is what needs the person's yes.
pub fn needs_confirmation(action: Action, connection: State) -> bool {
    match action {
        Action::Disconnect => connection != State::Disconnected,
        Action::Toggle => connection != State::Disconnected,
        Action::Connect | Action::Import => false,
    }
}

/// Whether an action asked for at `now` is too soon after the last one of the same kind.
pub fn cooling(last: Option<Instant>, now: Instant) -> bool {
    last.is_some_and(|at| now.saturating_duration_since(at) < COOL_DOWN)
}

static LAST: Mutex<[Option<Instant>; 3]> = Mutex::new([None; 3]);
static ASKING: AtomicBool = AtomicBool::new(false);

fn slot(action: Action) -> Option<usize> {
    match action {
        Action::Connect => Some(0),
        Action::Disconnect | Action::Toggle => Some(1),
        Action::Import => Some(2),
    }
}

/// Records the action and says whether it may go ahead, or is too soon after the last of its kind.
fn admit(action: Action) -> bool {
    let Some(index) = slot(action) else { return true };
    let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
    let now = Instant::now();
    if cooling(last[index], now) {
        return false;
    }
    last[index] = Some(now);
    true
}

pub fn handle(app: &AppHandle, raw: &str) {
    let Ok(url) = url::Url::parse(raw) else { return };
    let Some(action) = action_of(&url) else { return };
    if !admit(action) {
        return;
    }
    let app = app.clone();
    match action {
        Action::Connect => {
            tauri::async_runtime::spawn(async move {
                reliability::user_took_over(&app);
                if let Err(error) = session::connect(&app).await {
                    if !error.is_cancelled() {
                        notify::send(&app, "Connection failed", &error.to_string());
                    }
                }
            });
        }
        Action::Disconnect | Action::Toggle => {
            let connection = app.state::<AppState>().status.lock().unwrap().state;
            if !needs_confirmation(action, connection) {
                // Toggle while disconnected is a connect.
                tray::toggle_connection(&app);
                return;
            }
            tauri::async_runtime::spawn(async move {
                if !confirm_turning_off().await {
                    return;
                }
                reliability::user_took_over(&app);
                let _ = session::disconnect(&app).await;
            });
        }
        Action::Import => {
            if let Some((_, value)) = url.query_pairs().find(|(key, _)| key == "url") {
                if value.chars().count() > MAX_IMPORT_CHARS {
                    return;
                }
                tray::show_main_window(&app);
                let _ = app.emit("import-request", value.into_owned());
            }
        }
    }
}

/// Asks in a system dialog whether the VPN may be turned off. One question at a time; no answer
/// within `ASK_SECONDS` counts as no.
async fn confirm_turning_off() -> bool {
    if ASKING.swap(true, Ordering::SeqCst) {
        return false;
    }
    let answer = tokio::task::spawn_blocking(ask).await.unwrap_or(false);
    ASKING.store(false, Ordering::SeqCst);
    answer
}

fn ask() -> bool {
    let script = format!(
        "display dialog \"A link asked Kage to turn the VPN off. Your traffic would no longer be protected. Turn it off?\" with title \"Kage\" buttons {{\"Keep VPN on\", \"Turn off\"}} default button \"Keep VPN on\" cancel button \"Keep VPN on\" with icon caution giving up after {ASK_SECONDS}"
    );
    std::process::Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
        .is_some_and(|text| text.contains("button returned:Turn off") && !text.contains("gave up:true"))
}

#[cfg(test)]
#[path = "tests/deeplink.rs"]
mod tests;
