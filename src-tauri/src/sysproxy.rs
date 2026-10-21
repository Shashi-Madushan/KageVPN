//! The macOS system proxy for Proxy Only mode: apps that follow the system settings use the local
//! proxy without being set up one by one.
//!
//! Every change goes through `/usr/sbin/networksetup`, which macOS lets change network settings
//! without an administrator password (it is entitled for that right). Before the first change the
//! previous settings of every service are saved to a file, with a restore script beside it that
//! the supervisor runs if the app dies, so a crash never leaves a proxy pointing at nothing.
//! A restore only touches services that still point at Kage, so a change the user made while
//! connected is kept.

use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};
use crate::paths::AppPaths;

const NETWORKSETUP: &str = "/usr/sbin/networksetup";
const LOCALHOST: &str = "127.0.0.1";

/// Always skipped: this Mac, the local network and link-local addresses.
pub const BUILT_IN_BYPASS: &[&str] =
    &["localhost", "127.0.0.1", "::1", "*.local", "169.254.0.0/16", "10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16"];
const MAX_BYPASS: usize = 200;

// ── What macOS stores per service ───────────────────────────────────────────

/// One of the Web, Secure Web and SOCKS proxies of a service.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proxy {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    /// Uses a login macOS keeps in the Keychain, which `networksetup` cannot read back.
    pub authenticated: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceProxy {
    pub service: String,
    pub web: Proxy,
    pub secure_web: Proxy,
    pub socks: Proxy,
    /// The automatic proxy (PAC) URL, if one is stored, and whether it is on.
    pub auto_url: Option<String>,
    pub auto_enabled: bool,
    /// Automatic proxy discovery (WPAD).
    pub auto_discovery: bool,
    pub bypass: Vec<String>,
}

#[derive(Clone, Copy)]
enum Kind {
    Web,
    SecureWeb,
    Socks,
}

impl Kind {
    const ALL: [Kind; 3] = [Kind::Web, Kind::SecureWeb, Kind::Socks];

    fn get(self) -> &'static str {
        match self {
            Kind::Web => "-getwebproxy",
            Kind::SecureWeb => "-getsecurewebproxy",
            Kind::Socks => "-getsocksfirewallproxy",
        }
    }

    fn set(self) -> &'static str {
        match self {
            Kind::Web => "-setwebproxy",
            Kind::SecureWeb => "-setsecurewebproxy",
            Kind::Socks => "-setsocksfirewallproxy",
        }
    }

    fn set_state(self) -> &'static str {
        match self {
            Kind::Web => "-setwebproxystate",
            Kind::SecureWeb => "-setsecurewebproxystate",
            Kind::Socks => "-setsocksfirewallproxystate",
        }
    }

    fn of(self, service: &ServiceProxy) -> &Proxy {
        match self {
            Kind::Web => &service.web,
            Kind::SecureWeb => &service.secure_web,
            Kind::Socks => &service.socks,
        }
    }
}

// ── Reading networksetup output ─────────────────────────────────────────────

fn field<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    text.lines().find_map(|line| line.strip_prefix(name)?.strip_prefix(':')).map(str::trim)
}

/// `-listallnetworkservices`: the enabled services; disabled ones start with `*` and the first line
/// is an explanation.
pub fn parse_services(text: &str) -> Vec<String> {
    text.lines()
        .skip_while(|line| line.starts_with("An asterisk"))
        .map(str::trim_end)
        .filter(|line| !line.is_empty() && !line.starts_with('*'))
        .map(str::to_string)
        .collect()
}

/// `-getwebproxy` and friends.
pub fn parse_proxy(text: &str) -> Proxy {
    Proxy {
        enabled: field(text, "Enabled") == Some("Yes"),
        host: field(text, "Server").unwrap_or_default().to_string(),
        port: field(text, "Port").and_then(|p| p.parse().ok()).unwrap_or(0),
        authenticated: field(text, "Authenticated Proxy Enabled") == Some("1"),
    }
}

/// `-getautoproxyurl`: the URL (`(null)` when none) and whether it is on.
pub fn parse_auto_url(text: &str) -> (Option<String>, bool) {
    let url = field(text, "URL").filter(|u| !u.is_empty() && *u != "(null)").map(str::to_string);
    (url, field(text, "Enabled") == Some("Yes"))
}

/// `-getproxyautodiscovery`.
pub fn parse_auto_discovery(text: &str) -> bool {
    field(text, "Auto Proxy Discovery") == Some("On")
}

/// `-getproxybypassdomains`: one entry per line, or a sentence when there are none.
pub fn parse_bypass(text: &str) -> Vec<String> {
    if text.contains("There aren't any bypass domains") {
        return Vec::new();
    }
    text.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect()
}

// ── Deciding ────────────────────────────────────────────────────────────────

/// Whether this service still points at Kage's local proxy on `port`.
pub fn still_kage(current: &ServiceProxy, port: u16) -> bool {
    Kind::ALL.iter().any(|kind| {
        let proxy = kind.of(current);
        proxy.enabled && proxy.host == LOCALHOST && proxy.port == port
    })
}

/// Whether a service has a proxy with a login. Its password lives in the Keychain and cannot be
/// put back, so Kage leaves such a service alone.
pub fn has_login(service: &ServiceProxy) -> bool {
    Kind::ALL.iter().any(|kind| kind.of(service).enabled && kind.of(service).authenticated)
}

/// The built-in list followed by the user's entries, trimmed and without repeats.
pub fn bypass_list(user: &[String]) -> Result<Vec<String>> {
    let mut list: Vec<String> = BUILT_IN_BYPASS.iter().map(|s| s.to_string()).collect();
    for entry in user.iter().map(|e| e.trim()).filter(|e| !e.is_empty()) {
        if !valid_bypass(entry) {
            return Err(AppError::Invalid(format!("'{entry}' is not a host name or address range")));
        }
        if !list.iter().any(|e| e.eq_ignore_ascii_case(entry)) {
            list.push(entry.to_string());
        }
    }
    if list.len() > BUILT_IN_BYPASS.len() + MAX_BYPASS {
        return Err(AppError::Invalid(format!("at most {MAX_BYPASS} extra entries")));
    }
    Ok(list)
}

/// A host name (an optional leading `*.`), an address, or an address with a `/prefix`.
fn valid_bypass(entry: &str) -> bool {
    if let Some((address, prefix)) = entry.split_once('/') {
        return match address.parse::<std::net::IpAddr>() {
            Ok(ip) => prefix.parse::<u8>().is_ok_and(|p| p <= if ip.is_ipv4() { 32 } else { 128 }),
            Err(_) => false,
        };
    }
    if entry.parse::<std::net::IpAddr>().is_ok() {
        return true;
    }
    let host = entry.strip_prefix("*.").unwrap_or(entry);
    !host.is_empty()
        && host.len() <= 253
        && host.split('.').all(|label| {
            !label.is_empty() && label.len() <= 63 && !label.starts_with('-') && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
}

/// The `networksetup` calls that point `service` at the local proxy.
pub fn set_commands(service: &str, port: u16, bypass: &[String]) -> Vec<Vec<String>> {
    let s = service.to_string();
    let mut commands: Vec<Vec<String>> = Kind::ALL.iter().map(|k| vec![k.set().into(), s.clone(), LOCALHOST.into(), port.to_string()]).collect();
    // An automatic proxy or discovery would take precedence over the manual proxies.
    commands.push(vec!["-setautoproxystate".into(), s.clone(), "off".into()]);
    commands.push(vec!["-setproxyautodiscovery".into(), s.clone(), "off".into()]);
    commands.push(bypass_command(&s, bypass));
    commands
}

/// The `networksetup` calls that put `saved` back as it was.
pub fn restore_commands(saved: &ServiceProxy) -> Vec<Vec<String>> {
    let s = saved.service.clone();
    let mut commands = Vec::new();
    for kind in Kind::ALL {
        // Setting the address turns the proxy on; a proxy that was off gets its old address
        // (often empty, which networksetup accepts) and is turned off again.
        let proxy = kind.of(saved);
        commands.push(vec![kind.set().into(), s.clone(), proxy.host.clone(), proxy.port.to_string()]);
        if !proxy.enabled {
            commands.push(vec![kind.set_state().into(), s.clone(), "off".into()]);
        }
    }
    match &saved.auto_url {
        Some(url) => {
            commands.push(vec!["-setautoproxyurl".into(), s.clone(), url.clone()]);
            if !saved.auto_enabled {
                commands.push(vec!["-setautoproxystate".into(), s.clone(), "off".into()]);
            }
        }
        None => commands.push(vec!["-setautoproxystate".into(), s.clone(), "off".into()]),
    }
    commands.push(vec!["-setproxyautodiscovery".into(), s.clone(), if saved.auto_discovery { "on" } else { "off" }.into()]);
    commands.push(bypass_command(&s, &saved.bypass));
    commands
}

fn bypass_command(service: &str, bypass: &[String]) -> Vec<String> {
    let mut command = vec!["-setproxybypassdomains".to_string(), service.to_string()];
    if bypass.is_empty() {
        command.push("Empty".into());
    } else {
        command.extend(bypass.iter().cloned());
    }
    command
}

fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

/// A shell script the supervisor runs when the app dies, putting back every saved service that
/// still points at Kage on `port`. Only `networksetup` and `grep` run; all values are quoted.
pub fn restore_script(snapshot: &Snapshot) -> String {
    let mut script = format!(
        "#!/bin/sh\n# Written by Kage: puts back the proxy settings it changed.\nNS={NETWORKSETUP}\n\
         owned() {{\n  for get in -getwebproxy -getsecurewebproxy -getsocksfirewallproxy; do\n    \
         out=$($NS \"$get\" \"$1\")\n    \
         echo \"$out\" | grep -qx 'Enabled: Yes' && echo \"$out\" | grep -qx 'Server: {LOCALHOST}' && echo \"$out\" | grep -qx 'Port: {}' && return 0\n  \
         done\n  return 1\n}}\n",
        snapshot.port
    );
    for saved in &snapshot.services {
        script.push_str(&format!("if owned {}; then\n", quote(&saved.service)));
        for command in restore_commands(saved) {
            let args: Vec<String> = command.iter().map(|a| quote(a)).collect();
            script.push_str(&format!("  $NS {}\n", args.join(" ")));
        }
        script.push_str("fi\n");
    }
    script
}

// ── State ───────────────────────────────────────────────────────────────────

/// The settings Kage saved before changing anything, and the port it set.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub port: u16,
    pub services: Vec<ServiceProxy>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum SystemProxyState {
    /// Kage has not changed anything.
    #[default]
    Off,
    Set { port: u16, services: Vec<String> },
    Failed { action: Action, reason: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Set,
    Restore,
}

// ── Running commands ────────────────────────────────────────────────────────

/// Runs `networksetup`; tests use a fake that records the calls.
pub trait Runner {
    fn run(&self, args: &[String]) -> Result<String>;
}

pub struct NetworkSetup;

impl Runner for NetworkSetup {
    fn run(&self, args: &[String]) -> Result<String> {
        let output = Command::new(NETWORKSETUP).args(args).output()?;
        let text = String::from_utf8_lossy(&output.stdout).into_owned() + &String::from_utf8_lossy(&output.stderr);
        // networksetup often exits 0 after an error and says so in its output.
        if !output.status.success() || text.contains("** Error") || text.contains("requires admin") || text.contains("must run this tool as root") {
            let first = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("no output").trim().to_string();
            return Err(AppError::Launch(format!("macOS refused to change the proxy settings ({first})")));
        }
        Ok(text)
    }
}

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// The current settings of `service`.
pub fn read_service(runner: &dyn Runner, service: &str) -> Result<ServiceProxy> {
    let get = |command: &str| runner.run(&args(&[command, service]));
    let (auto_url, auto_enabled) = parse_auto_url(&get("-getautoproxyurl")?);
    Ok(ServiceProxy {
        service: service.to_string(),
        web: parse_proxy(&get(Kind::Web.get())?),
        secure_web: parse_proxy(&get(Kind::SecureWeb.get())?),
        socks: parse_proxy(&get(Kind::Socks.get())?),
        auto_url,
        auto_enabled,
        auto_discovery: parse_auto_discovery(&get("-getproxyautodiscovery")?),
        bypass: parse_bypass(&get("-getproxybypassdomains")?),
    })
}

fn load_snapshot(paths: &AppPaths) -> Option<Snapshot> {
    let bytes = std::fs::read(paths.system_proxy_snapshot()).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn save(paths: &AppPaths, snapshot: &Snapshot) -> Result<()> {
    crate::fsutil::write_private_atomic(&paths.system_proxy_snapshot(), &serde_json::to_vec_pretty(snapshot)?)?;
    crate::fsutil::write_private_atomic(&paths.system_proxy_script(), restore_script(snapshot).as_bytes())?;
    Ok(())
}

fn forget(paths: &AppPaths) {
    let _ = std::fs::remove_file(paths.system_proxy_script());
    let _ = std::fs::remove_file(paths.system_proxy_snapshot());
}

/// Whether Kage has saved settings it has not put back yet.
pub fn has_snapshot(paths: &AppPaths) -> bool {
    paths.system_proxy_snapshot().exists()
}

/// Points every enabled service at the local proxy on `port`. Services seen for the first time
/// are saved before they change; a run on a new port rewrites the saved port. On a failure the
/// services already changed are put back.
pub fn apply(paths: &AppPaths, runner: &dyn Runner, port: u16, bypass: &[String]) -> SystemProxyState {
    let failed = |reason: String| SystemProxyState::Failed { action: Action::Set, reason };
    let bypass = match bypass_list(bypass) {
        Ok(list) => list,
        Err(e) => return failed(e.to_string()),
    };
    let services = match runner.run(&args(&["-listallnetworkservices"])) {
        Ok(text) => parse_services(&text),
        Err(e) => return failed(e.to_string()),
    };
    let mut snapshot = load_snapshot(paths).unwrap_or_default();
    let mut skipped = Vec::new();
    for service in &services {
        if snapshot.services.iter().any(|s| &s.service == service) {
            continue;
        }
        match read_service(runner, service) {
            // A proxy with a login cannot be put back (its password is in the Keychain).
            Ok(current) if has_login(&current) => skipped.push(service.clone()),
            Ok(current) => snapshot.services.push(current),
            Err(e) => return failed(e.to_string()),
        }
    }
    let old_port = snapshot.port;
    snapshot.port = port;
    // Saved first: if anything below fails or the app dies, the restore knows what to put back.
    if let Err(e) = save(paths, &snapshot) {
        return failed(format!("could not save the current settings: {e}"));
    }
    let targets: Vec<&ServiceProxy> = snapshot.services.iter().filter(|s| services.contains(&s.service)).collect();
    for saved in &targets {
        for command in set_commands(&saved.service, port, &bypass) {
            if let Err(e) = runner.run(&command) {
                // Put back what was changed; services on the old port count as Kage's too.
                let _ = restore_with(paths, runner, &[port, old_port]);
                return failed(e.to_string());
            }
        }
    }
    if !skipped.is_empty() {
        crate::applog::warn(format!("system proxy: left {} alone (its proxy has a login Kage could not put back)", skipped.join(", ")));
    }
    SystemProxyState::Set { port, services: targets.iter().map(|s| s.service.clone()).collect() }
}

/// Puts back every saved service that still points at Kage, then forgets the saved settings.
pub fn restore(paths: &AppPaths, runner: &dyn Runner) -> SystemProxyState {
    let port = load_snapshot(paths).map(|s| s.port).unwrap_or(0);
    restore_with(paths, runner, &[port])
}

fn restore_with(paths: &AppPaths, runner: &dyn Runner, ports: &[u16]) -> SystemProxyState {
    let Some(snapshot) = load_snapshot(paths) else {
        forget(paths);
        return SystemProxyState::Off;
    };
    let present = runner.run(&args(&["-listallnetworkservices"])).map(|t| parse_services(&t)).unwrap_or_default();
    let mut problems = Vec::new();
    for saved in snapshot.services.iter().filter(|s| present.contains(&s.service)) {
        let owned = match read_service(runner, &saved.service) {
            Ok(current) => ports.iter().any(|p| *p != 0 && still_kage(&current, *p)),
            Err(e) => {
                problems.push(format!("{}: {e}", saved.service));
                continue;
            }
        };
        if !owned {
            continue;
        }
        if let Some(e) = restore_commands(saved).iter().find_map(|c| runner.run(c).err()) {
            problems.push(format!("{}: {e}", saved.service));
        }
    }
    if problems.is_empty() {
        forget(paths);
        SystemProxyState::Off
    } else {
        SystemProxyState::Failed { action: Action::Restore, reason: problems.join("; ") }
    }
}

/// Whether the system proxy should be set for a run in `mode` with these settings.
pub fn applies(mode: crate::config::Mode, settings: &crate::settings::ReliabilitySettings) -> bool {
    mode == crate::config::Mode::Proxy && settings.system_proxy && !settings.proxy_auth
}

// ── In the app ──────────────────────────────────────────────────────────────

/// One change at a time: a connect, a stop and a network change can all ask at once.
static BUSY: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn record(app: &tauri::AppHandle, new: SystemProxyState) {
    use tauri::Manager;
    match &new {
        SystemProxyState::Set { port, services } => crate::applog::info(format!("system proxy: set to 127.0.0.1:{port} on {}", services.join(", "))),
        SystemProxyState::Off => crate::applog::info("system proxy: settings put back"),
        SystemProxyState::Failed { action, reason } => crate::applog::warn(format!("system proxy: {action:?} failed: {reason}")),
    }
    *app.state::<crate::state::AppState>().system_proxy.lock().unwrap() = new;
    crate::session::update(app, |_| {});
}

/// Brings the macOS settings in line with the current run: set while connected in Proxy Only mode
/// with the option on (on the run's port), put back otherwise. Runs `networksetup`, so call it off
/// the async runtime (`spawn_sync`).
pub fn sync(app: &tauri::AppHandle) {
    use tauri::Manager;
    let _one = BUSY.lock().unwrap_or_else(|e| e.into_inner());
    let state = app.state::<crate::state::AppState>();
    let (connected, mode) = {
        let status = state.status.lock().unwrap();
        (status.state == crate::session::State::Connected, status.mode)
    };
    let settings = state.store.lock().unwrap().reliability();
    if connected && applies(mode, &settings) {
        record(app, apply(&state.paths, &NetworkSetup, crate::config::mixed_port(), &settings.bypass));
    } else if has_snapshot(&state.paths) {
        record(app, restore(&state.paths, &NetworkSetup));
    }
}

pub fn spawn_sync(app: tauri::AppHandle) {
    tauri::async_runtime::spawn_blocking(move || sync(&app));
}

/// Puts the saved settings back now, whatever the run; for a stop, a quit and Repair.
pub async fn restore_now(app: &tauri::AppHandle) -> SystemProxyState {
    use tauri::Manager;
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _one = BUSY.lock().unwrap_or_else(|e| e.into_inner());
        let state = app.state::<crate::state::AppState>();
        if !has_snapshot(&state.paths) {
            return SystemProxyState::Off;
        }
        let result = restore(&state.paths, &NetworkSetup);
        record(&app, result.clone());
        result
    })
    .await
    .unwrap_or(SystemProxyState::Off)
}

/// At launch: settings left behind by a run that ended without putting them back (power loss,
/// a killed supervisor) are put back before anything connects.
pub fn restore_at_launch(paths: &AppPaths) {
    if has_snapshot(paths) {
        match restore(paths, &NetworkSetup) {
            SystemProxyState::Off => crate::applog::info("system proxy: put back settings left by the last run"),
            other => crate::applog::warn(format!("system proxy: could not put back settings left by the last run: {other:?}")),
        }
    }
}

/// The state to show: a `Set` whose saved settings are gone was put back by the supervisor.
pub fn current(state: &crate::state::AppState) -> SystemProxyState {
    let mut current = state.system_proxy.lock().unwrap();
    if matches!(*current, SystemProxyState::Set { .. }) && !has_snapshot(&state.paths) {
        *current = SystemProxyState::Off;
    }
    current.clone()
}

#[cfg(test)]
#[path = "tests/sysproxy.rs"]
mod tests;
