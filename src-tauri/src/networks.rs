//! Auto-connect by network. A network is identified by its gateway's hardware address, which needs
//! no permission (the Wi-Fi name needs Location access). What to do on each network is a setting;
//! `decide` is the whole policy and the rest only reads the network and acts on the answer.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::error::{AppError, Result};
use crate::network_watch::is_virtual;
use crate::session::{self, State as Conn};
use crate::settings::{KnownNetwork, NetworkAction, NetworkSettings};
use crate::state::AppState;
use crate::{applog, notify};

const MAX_NAME: usize = 40;
/// How long the question about a new network waits before it counts as "Ignore".
const ASK_SECONDS: u32 = 60;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    Nothing,
    /// A network seen for the first time: remember it and ask what to do.
    Ask,
    Connect,
    Disconnect,
}

/// What to do on the network `id`. `reconnect_active` is true while the reconnect loop is running.
pub fn decide(settings: &NetworkSettings, id: &str, state: Conn, has_servers: bool, reconnect_active: bool) -> Decision {
    if !settings.enabled || id.is_empty() {
        return Decision::Nothing;
    }
    let Some(known) = settings.networks.iter().find(|n| n.id == id) else { return Decision::Ask };
    match known.action {
        NetworkAction::Connect if state == Conn::Disconnected && has_servers && !reconnect_active => Decision::Connect,
        NetworkAction::Disconnect if state != Conn::Disconnected => Decision::Disconnect,
        _ => Decision::Nothing,
    }
}

/// The default gateway of the first physical interface, from `netstat -rn -f inet`. While the tunnel
/// is up the plain default route points into it, but the interface-scoped one still names the
/// real gateway.
pub fn parse_gateway(netstat: &str) -> Option<(String, String)> {
    netstat.lines().find_map(|line| {
        let mut columns = line.split_whitespace();
        (columns.next()? == "default").then_some(())?;
        let gateway = columns.next()?;
        let interface = columns.nth(1)?;
        (gateway.parse::<std::net::Ipv4Addr>().is_ok() && !is_virtual(interface)).then(|| (gateway.to_string(), interface.to_string()))
    })
}

/// A hardware address from `arp -n <ip>` output, lower case with two digits per part.
pub fn parse_mac(arp: &str) -> Option<String> {
    let token = arp.split_whitespace().skip_while(|t| *t != "at").nth(1)?;
    let parts: Vec<&str> = token.split(':').collect();
    (parts.len() == 6 && parts.iter().all(|p| (1..=2).contains(&p.len()) && p.chars().all(|c| c.is_ascii_hexdigit())))
        .then(|| parts.iter().map(|p| format!("{:0>2}", p.to_ascii_lowercase())).collect::<Vec<_>>().join(":"))
}

/// "Network de56": the last four characters of the hardware address.
pub fn default_name(id: &str) -> String {
    let compact: String = id.chars().filter(|c| *c != ':').collect();
    format!("Network {}", &compact[compact.len().saturating_sub(4)..])
}

/// Names are trimmed and 1 to 40 characters.
pub fn clean_name(name: &str) -> Option<String> {
    let name = name.trim();
    (!name.is_empty() && name.chars().count() <= MAX_NAME).then(|| name.to_string())
}

fn run(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentNetwork {
    pub id: String,
    pub name: String,
}

/// The network the Mac is on, or `None` without a gateway or hardware address.
pub fn current() -> Option<CurrentNetwork> {
    let (gateway, interface) = parse_gateway(&run("/usr/sbin/netstat", &["-rn", "-f", "inet"])?)?;
    let id = parse_mac(&run("/usr/sbin/arp", &["-n", &gateway])?)?;
    // The Wi-Fi name, when macOS still hands it out; newer versions often answer with a placeholder.
    let wifi = run("/usr/sbin/networksetup", &["-getairportnetwork", &interface])
        .and_then(|text| text.split_once(": ").map(|(_, name)| name.trim().to_string()))
        .filter(|name| !name.is_empty() && !name.contains("redacted") && !name.contains("not associated"));
    let name = wifi.and_then(|n| clean_name(&n)).unwrap_or_else(|| default_name(&id));
    Some(CurrentNetwork { id, name })
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Looks at the current network and does what the settings say. `manual_actions` is the counter's
/// value when the change was noticed: a manual connect or disconnect since then wins.
pub async fn apply(app: &AppHandle, manual_actions: u64) {
    let state = app.state::<AppState>();
    let Some(network) = tokio::task::spawn_blocking(current).await.ok().flatten() else { return };
    if state.manual_actions.load(std::sync::atomic::Ordering::SeqCst) != manual_actions {
        applog::info("networks: the user chose by hand, leaving the connection alone");
        return;
    }
    let (settings, has_servers) = {
        let mut store = state.store.lock().unwrap();
        let mut settings = store.networks();
        // Keep "last seen" fresh for a known network.
        if let Some(known) = settings.networks.iter_mut().find(|n| n.id == network.id) {
            known.last_seen = now_secs();
            let _ = store.set_networks(settings.clone());
        }
        (settings, store.active_profile().is_some())
    };
    let connection = state.status.lock().unwrap().state;
    let reconnecting = state.reconnecting.load(std::sync::atomic::Ordering::SeqCst) || state.reconnect_waiting.load(std::sync::atomic::Ordering::SeqCst);
    match decide(&settings, &network.id, connection, has_servers, reconnecting) {
        Decision::Nothing => {}
        Decision::Connect => {
            applog::info(format!("networks: connecting on {}", network.name));
            if let Err(error) = session::connect(app).await {
                if !error.is_cancelled() {
                    notify::send(app, "Connection failed", &error.to_string());
                }
            }
        }
        Decision::Disconnect => {
            applog::info(format!("networks: disconnecting on {}", network.name));
            let _ = session::disconnect(app).await;
        }
        Decision::Ask => ask(app, network),
    }
}

/// Remembers a new network (as Ignore) and asks once what to do on it.
fn ask(app: &AppHandle, network: CurrentNetwork) {
    {
        let state = app.state::<AppState>();
        let mut store = state.store.lock().unwrap();
        let mut settings = store.networks();
        settings.networks.push(KnownNetwork { id: network.id.clone(), name: network.name.clone(), action: NetworkAction::Ignore, last_seen: now_secs() });
        let _ = store.set_networks(settings);
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let name = network.name.clone();
        let answer = tokio::task::spawn_blocking(move || question(&name)).await.ok().flatten();
        let action = if answer.as_deref() == Some("Connect") { NetworkAction::Connect } else { NetworkAction::Ignore };
        applog::info(format!("networks: {} set to {action:?}", network.name));
        let _ = set_action(&app, &network.id, action);
        if action == NetworkAction::Connect {
            let _ = session::connect(&app).await;
        }
    });
}

/// A small macOS dialog; `None` when it timed out or was closed.
fn question(name: &str) -> Option<String> {
    let name: String = name.chars().filter(|c| !matches!(c, '"' | '\\')).collect();
    let script = format!(
        "display dialog \"New network: {name}. Connect the VPN whenever you are on it? You can change this in Settings, Networks.\" with title \"Kage\" buttons {{\"Ignore\", \"Connect\"}} default button \"Connect\" giving up after {ASK_SECONDS}"
    );
    let text = run("/usr/bin/osascript", &["-e", &script])?;
    // Output looks like: button returned:Connect, gave up:false
    let returned = text.split("button returned:").nth(1)?;
    let gave_up = returned.contains("gave up:true");
    (!gave_up).then(|| returned.split(',').next().unwrap_or("").trim().to_string())
}

fn set_action(app: &AppHandle, id: &str, action: NetworkAction) -> Result<()> {
    let state = app.state::<AppState>();
    let mut store = state.store.lock().unwrap();
    let mut settings = store.networks();
    if let Some(known) = settings.networks.iter_mut().find(|n| n.id == id) {
        known.action = action;
    }
    store.set_networks(settings)
}

#[tauri::command]
pub fn set_network_settings(state: State<AppState>, mut settings: NetworkSettings) -> Result<()> {
    for network in &mut settings.networks {
        network.name = clean_name(&network.name).ok_or_else(|| AppError::Invalid(format!("a network name must be 1 to {MAX_NAME} characters")))?;
    }
    let mut seen = std::collections::HashSet::new();
    settings.networks.retain(|n| seen.insert(n.id.clone()));
    state.store.lock().unwrap().set_networks(settings)
}

#[tauri::command(async)]
pub fn current_network() -> Option<CurrentNetwork> {
    current()
}

#[cfg(test)]
#[path = "tests/networks.rs"]
mod tests;
