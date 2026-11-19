//! What the system proxy does on every OS: the bypass list, the state shown in the app, and when
//! the proxy applies. The macOS and Windows files each include this.

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// Always skipped: this computer, the local network and link-local addresses.
pub const BUILT_IN_BYPASS: &[&str] =
    &["localhost", "127.0.0.1", "::1", "*.local", "169.254.0.0/16", "10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16"];
const MAX_BYPASS: usize = 200;

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

/// Whether the system proxy should be set for a run in `mode` with these settings.
pub fn applies(mode: crate::config::Mode, settings: &crate::settings::ReliabilitySettings) -> bool {
    mode == crate::config::Mode::Proxy && settings.system_proxy && !settings.proxy_auth
}
