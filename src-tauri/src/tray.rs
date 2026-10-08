use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{
    AppHandle, Listener, Manager, PhysicalPosition, Rect, WebviewWindow, WebviewWindowBuilder, Window,
    WindowEvent,
};

use crate::health::Health;
use crate::session::{self, State, Status};
use crate::{net, notify, speed_icon};
use crate::state::AppState;
use crate::traffic::Traffic;

const TRAY_ID: &str = "main";
const POPOVER: &str = "tray";
const STATUS: &str = "status";
const OPEN: &str = "open";
const TOGGLE: &str = "toggle";
const PING: &str = "ping";
const QUIT: &str = "quit";
const POPOVER_GAP: f64 = 6.0;
const SCREEN_MARGIN: f64 = 8.0;
const REOPEN_GUARD_MS: u64 = 250;
/// A hidden popover keeps its WebView for this long, then is destroyed to free memory.
const POPOVER_IDLE_MS: u64 = 180_000;

static LAST_BLUR_MS: AtomicU64 = AtomicU64::new(0);
static POPOVER_GEN: AtomicU64 = AtomicU64::new(0);
/// Set from the status event. `refresh` must not lock the status: events are emitted while it is held.
static TUNNEL_DOWN: AtomicBool = AtomicBool::new(false);

/// Smoothing factor for the menu bar rates: higher reacts faster, lower is calmer.
const RATE_SMOOTHING: f64 = 0.45;
/// Below this the displayed rate snaps to zero instead of trailing off ("3B", "1B", ...).
const RATE_FLOOR: f64 = 512.0;

/// What the menu bar icon currently shows, so identical frames are never redrawn.
#[derive(PartialEq)]
enum Frame {
    Plain(State),
    /// Connected, but the tunnel carries no traffic.
    Warning,
    Speed { up: String, down: String },
}

static SHOWN: Mutex<Option<Frame>> = Mutex::new(None);
static SMOOTHED: Mutex<Option<(f64, f64)>> = Mutex::new(None);

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let status_line = MenuItem::with_id(app, STATUS, status_text(&Status::disconnected(Default::default())), false, None::<&str>)?;
    let open = MenuItem::with_id(app, OPEN, format!("Open {}", crate::app_config::APP_NAME), true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, TOGGLE, "Connect", true, None::<&str>)?;
    let ping = MenuItem::with_id(app, PING, "Ping Server", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&status_line, &PredefinedMenuItem::separator(app)?, &open, &toggle, &ping, &PredefinedMenuItem::separator(app)?, &quit],
    )?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon(State::Disconnected)?)
        .icon_as_template(true)
        .show_menu_on_left_click(false)
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            OPEN => show_main_window(app),
            TOGGLE => toggle_connection(app),
            PING => ping_server(app),
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
        apply_status(&handle, &toggle, &status_line, &status);
    });
    Ok(())
}

/// The one-line summary at the top of the menu.
fn status_text(status: &Status) -> String {
    match (status.state, status.health) {
        (State::Disconnected, _) => "Not protected".into(),
        (State::Connecting, _) => "Connecting…".into(),
        (State::Connected, Health::Healthy) => match status.latency_ms {
            Some(ms) => format!("Protected · {ms} ms"),
            None => "Protected".into(),
        },
        (State::Connected, Health::Down) => "Connected, but no internet".into(),
        (State::Connected, _) => "Checking connection…".into(),
    }
}

fn apply_status(app: &AppHandle, toggle: &MenuItem<tauri::Wry>, status_line: &MenuItem<tauri::Wry>, status: &Status) {
    let state = status.state;
    TUNNEL_DOWN.store(status.health == Health::Down, Ordering::Relaxed);
    let _ = status_line.set_text(status_text(status));
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
    let down = state == State::Connected && TUNNEL_DOWN.load(Ordering::Relaxed);
    let show = app.state::<AppState>().store.lock().unwrap().settings().show_speed;
    let rates = smooth(traffic.filter(|_| show && state == State::Connected));

    let (frame, image) = if down {
        (Frame::Warning, Image::from_bytes(include_bytes!("../icons/tray/warning.png")))
    } else if show && state == State::Connected {
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

/// The window with this label, building it from `tauri.conf.json` when it does not exist.
///
/// Windows are declared with `create: false` and destroyed when hidden, so the WebView (tens of
/// megabytes of WebKit helper processes) only exists while the user can see it.
fn window_or_create(app: &AppHandle, label: &str) -> Option<WebviewWindow> {
    if let Some(window) = app.get_webview_window(label) {
        return Some(window);
    }
    let config = app.config().app.windows.iter().find(|c| c.label == label)?.clone();
    WebviewWindowBuilder::from_config(app, &config).ok()?.build().ok()
}

pub fn show_main_window(app: &AppHandle) {
    hide_popover(app);
    if let Some(window) = window_or_create(app, "main") {
        // The app is menu-bar only (LSUIElement); it gets a Dock icon while its window is open.
        #[cfg(target_os = "macos")]
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn hide_popover(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(POPOVER) {
        let _ = window.hide();
        schedule_popover_destroy(app);
    }
}

/// Destroys the hidden popover after `POPOVER_IDLE_MS` unless it was shown again meanwhile.
fn schedule_popover_destroy(app: &AppHandle) {
    let generation = POPOVER_GEN.fetch_add(1, Ordering::Relaxed) + 1;
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(POPOVER_IDLE_MS)).await;
        if POPOVER_GEN.load(Ordering::Relaxed) != generation {
            return;
        }
        if let Some(window) = app.get_webview_window(POPOVER) {
            if !window.is_visible().unwrap_or(false) {
                let _ = window.destroy();
            }
        }
    });
}

pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    #[cfg(target_os = "macos")]
    if window.label() == "main" && matches!(event, WindowEvent::Destroyed) {
        let _ = window.app_handle().set_activation_policy(tauri::ActivationPolicy::Accessory);
    }
    if window.label() == POPOVER && matches!(event, WindowEvent::Focused(false)) {
        LAST_BLUR_MS.store(now_ms(), Ordering::Relaxed);
        let _ = window.hide();
        schedule_popover_destroy(window.app_handle());
    }
}

fn toggle_popover(app: &AppHandle, anchor: Rect) {
    if let Some(window) = app.get_webview_window(POPOVER) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
            schedule_popover_destroy(app);
            return;
        }
    }
    if now_ms().saturating_sub(LAST_BLUR_MS.load(Ordering::Relaxed)) < REOPEN_GUARD_MS {
        return;
    }
    POPOVER_GEN.fetch_add(1, Ordering::Relaxed);
    let Some(window) = window_or_create(app, POPOVER) else { return };
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

/// Checks that the server answers and says so in a notification. While connected the check goes
/// through the tunnel (so it proves traffic flows); otherwise it is a TCP connect to the server.
pub fn ping_server(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let connected = state.status.lock().unwrap().state == State::Connected;
        let target = state.store.lock().unwrap().active_profile().map(|p| (p.name.clone(), p.server.clone(), p.port));
        let Some((name, server, port)) = target else {
            notify::send(&app, "Ping", "Add a server first.");
            return;
        };
        let (result, how) = if connected {
            (net::proxy_delay().await, "through the tunnel")
        } else {
            (net::tcp_latency(&server, port).await, "direct to the server")
        };
        match result {
            Ok(ms) => notify::send(&app, "Server is working", &format!("{name}: {ms} ms {how}.")),
            Err(error) => notify::send(&app, "Server did not answer", &format!("{name}: {error}")),
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
