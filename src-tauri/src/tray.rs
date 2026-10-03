use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Listener, Manager, PhysicalPosition, Rect, Window, WindowEvent};

use crate::session::{self, State, Status};
use crate::{notify, speed_icon};
use crate::state::AppState;
use crate::traffic::Traffic;

const TRAY_ID: &str = "main";
const POPOVER: &str = "tray";
const OPEN: &str = "open";
const TOGGLE: &str = "toggle";
const QUIT: &str = "quit";
const POPOVER_GAP: f64 = 6.0;
const SCREEN_MARGIN: f64 = 8.0;
const REOPEN_GUARD_MS: u64 = 250;

static LAST_BLUR_MS: AtomicU64 = AtomicU64::new(0);

/// Smoothing factor for the menu bar rates: higher reacts faster, lower is calmer.
const RATE_SMOOTHING: f64 = 0.45;
/// Below this the displayed rate snaps to zero instead of trailing off ("3B", "1B", ...).
const RATE_FLOOR: f64 = 512.0;

/// What the menu bar icon currently shows, so identical frames are never redrawn.
#[derive(PartialEq)]
enum Frame {
    Plain(State),
    Speed { up: String, down: String },
}

static SHOWN: Mutex<Option<Frame>> = Mutex::new(None);
static SMOOTHED: Mutex<Option<(f64, f64)>> = Mutex::new(None);

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, OPEN, "Open SingBox GUI", true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, TOGGLE, "Connect", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&open, &toggle, &PredefinedMenuItem::separator(app)?, &quit],
    )?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon(State::Disconnected)?)
        .icon_as_template(true)
        .show_menu_on_left_click(false)
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            OPEN => show_main_window(app),
            TOGGLE => toggle_connection(app),
            QUIT => quit_app(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                toggle_popover(tray.app_handle(), rect);
            }
        })
        .build(app)?;

    let handle = app.clone();
    app.listen("status", move |event| {
        let Ok(status) = serde_json::from_str::<Status>(event.payload()) else { return };
        apply_status(&handle, &toggle, status.state);
    });
    Ok(())
}

fn apply_status(app: &AppHandle, toggle: &MenuItem<tauri::Wry>, state: State) {
    let label = match state {
        State::Disconnected => "Connect",
        State::Connecting => "Connecting…",
        State::Connected => "Disconnect",
    };
    let _ = toggle.set_text(label);
    let _ = toggle.set_enabled(state != State::Connecting);

    refresh(app, state, None);
}

fn icon(state: State) -> tauri::Result<Image<'static>> {
    let bytes: &[u8] = match state {
        State::Disconnected => include_bytes!("../icons/tray/idle.png"),
        State::Connecting => include_bytes!("../icons/tray/connecting.png"),
        State::Connected => include_bytes!("../icons/tray/connected.png"),
    };
    Image::from_bytes(bytes)
}

pub fn set_speed(app: &AppHandle, traffic: Option<Traffic>) {
    refresh(app, state_of(app), traffic);
}

/// One smoothing step: eases the shown rates toward the new sample and snaps tiny tails to zero.
fn smooth_step(previous: Option<(f64, f64)>, up: u64, down: u64) -> (f64, f64) {
    let ease = |shown: f64, raw: u64| {
        let value = shown + (raw as f64 - shown) * RATE_SMOOTHING;
        if raw == 0 && value < RATE_FLOOR { 0.0 } else { value }
    };
    match previous {
        Some((shown_up, shown_down)) => (ease(shown_up, up), ease(shown_down, down)),
        None => (up as f64, down as f64),
    }
}

/// Smooths the raw per-second rates so the digits do not jitter.
fn smooth(traffic: Option<Traffic>) -> Option<(u64, u64)> {
    let mut previous = SMOOTHED.lock().unwrap();
    let Some(t) = traffic else {
        *previous = None;
        return None;
    };
    let next = smooth_step(*previous, t.up, t.down);
    *previous = Some(next);
    Some((next.0 as u64, next.1 as u64))
}

/// Draws the menu bar icon for `state`, adding the speed readout while connected.
///
/// The icon and its template flag are applied in one call (`set_icon` followed by
/// `set_icon_as_template` renders twice on macOS, which shows up as flicker), and a frame
/// identical to the one on screen is skipped. While connected with the readout enabled the
/// icon always has the readout's full width, so the menu bar never shifts.
fn refresh(app: &AppHandle, state: State, traffic: Option<Traffic>) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else { return };
    let show = app.state::<AppState>().store.lock().unwrap().settings().show_speed;
    let rates = smooth(traffic.filter(|_| show && state == State::Connected));

    let (frame, image) = if show && state == State::Connected {
        let (up, down) = rates.unwrap_or((0, 0));
        let (up, down) = (format_rate(up), format_rate(down));
        let image = icon(State::Connected)
            .map(|base| speed_icon::compose(&base, &up, &down));
        (Frame::Speed { up, down }, image)
    } else {
        (Frame::Plain(state), icon(state))
    };

    let mut shown = SHOWN.lock().unwrap();
    if shown.as_ref() == Some(&frame) {
        return;
    }
    if let Ok(image) = image {
        if tray.set_icon_with_as_template(Some(image), true).is_ok() {
            *shown = Some(frame);
        }
    }
}

fn state_of(app: &AppHandle) -> State {
    app.state::<AppState>().status.lock().unwrap().state
}

fn format_rate(bytes_per_second: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    let value = bytes_per_second as f64;
    if value < KIB {
        format!("{bytes_per_second}B")
    } else if value < MIB {
        format!("{:.0}K", value / KIB)
    } else {
        format!("{:.1}M", value / MIB)
    }
}

pub fn show_main_window(app: &AppHandle) {
    hide_popover(app);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn hide_popover(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(POPOVER) {
        let _ = window.hide();
    }
}

pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    if window.label() == POPOVER && matches!(event, WindowEvent::Focused(false)) {
        LAST_BLUR_MS.store(now_ms(), Ordering::Relaxed);
        let _ = window.hide();
    }
}

fn toggle_popover(app: &AppHandle, anchor: Rect) {
    let Some(window) = app.get_webview_window(POPOVER) else { return };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }
    if now_ms().saturating_sub(LAST_BLUR_MS.load(Ordering::Relaxed)) < REOPEN_GUARD_MS {
        return;
    }
    place_below(app, &window, anchor);
    let _ = window.show();
    let _ = window.set_focus();
}

fn place_below(app: &AppHandle, window: &tauri::WebviewWindow, anchor: Rect) {
    let scale = window.scale_factor().unwrap_or(1.0);
    let position = anchor.position.to_physical::<f64>(scale);
    let size = anchor.size.to_physical::<f64>(scale);
    let Ok(window_size) = window.outer_size() else { return };

    let center_x = position.x + size.width / 2.0;
    let mut x = center_x - f64::from(window_size.width) / 2.0;
    let y = position.y + size.height + POPOVER_GAP * scale;

    if let Ok(Some(monitor)) = app.monitor_from_point(center_x, position.y) {
        let margin = SCREEN_MARGIN * scale;
        let left = f64::from(monitor.position().x) + margin;
        let right = f64::from(monitor.position().x)
            + f64::from(monitor.size().width)
            - f64::from(window_size.width)
            - margin;
        x = x.clamp(left, right.max(left));
    }
    let _ = window.set_position(PhysicalPosition::new(x as i32, y as i32));
}

pub fn toggle_connection(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>().status.lock().unwrap().state;
        let result = match state {
            State::Disconnected => session::connect(&app).await,
            State::Connected => session::disconnect(&app).await,
            State::Connecting => Ok(()),
        };
        if let Err(error) = result {
            notify::send(&app, "Connection failed", &error.to_string());
            show_main_window(&app);
        }
    });
}

pub fn quit_app(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = session::disconnect(&app).await;
        app.exit(0);
    });
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

#[cfg(test)]
#[path = "tests/tray.rs"]
mod tests;
