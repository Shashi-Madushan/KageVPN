//! The Windows system proxy as data: the per-user WinINet values Kage saves, sets and puts back.
//! Kept free of Windows calls so the rules are tested on every OS; `sysproxy_windows.rs` reads and
//! writes the registry with them.

use serde::{Deserialize, Serialize};

const LOCALHOST: &str = "127.0.0.1";
/// Windows' own entry for "bypass the proxy for plain host names without dots".
const LOCAL_NAMES: &str = "<local>";
/// An address range wider than this many `a.b.c.*` patterns is widened to the next whole octet.
const MAX_EXPANSION: u32 = 32;

/// The values under `HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings` that Kage
/// changes. `None` means the value did not exist, so putting it back deletes it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Values {
    pub proxy_enable: Option<u32>,
    pub proxy_server: Option<String>,
    pub proxy_override: Option<String>,
    pub auto_config_url: Option<String>,
}

/// What Kage saved before its first change, and the port it set.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Saved {
    pub port: u16,
    pub before: Values,
}

/// The values that point Windows at the local proxy. An automatic configuration script would win
/// over the manual proxy, so it is removed while connected (and put back after).
pub fn kage_values(port: u16, bypass: &[String]) -> Values {
    Values {
        proxy_enable: Some(1),
        proxy_server: Some(format!("{LOCALHOST}:{port}")),
        proxy_override: Some(override_list(bypass)),
        auto_config_url: None,
    }
}

/// Whether the current values still point at Kage on `port`. If the user pointed Windows somewhere
/// else while connected, Kage leaves their choice alone.
pub fn still_kage(current: &Values, port: u16) -> bool {
    current.proxy_enable == Some(1) && current.proxy_server.as_deref() == Some(format!("{LOCALHOST}:{port}").as_str())
}

/// The bypass list in WinINet's form: `;`-separated patterns where `*` is a wildcard and address
/// ranges are written as `10.*`. IPv6 ranges cannot be written that way and are left out; plain
/// IPv6 addresses are bracketed. `<local>` is always added.
pub fn override_list(bypass: &[String]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for entry in bypass {
        for pattern in patterns(entry) {
            if !parts.iter().any(|p| p.eq_ignore_ascii_case(&pattern)) {
                parts.push(pattern);
            }
        }
    }
    parts.push(LOCAL_NAMES.into());
    parts.join(";")
}

fn patterns(entry: &str) -> Vec<String> {
    let entry = entry.trim();
    if let Some((address, prefix)) = entry.split_once('/') {
        return match (address.parse::<std::net::Ipv4Addr>(), prefix.parse::<u32>()) {
            (Ok(ip), Ok(prefix)) if prefix <= 32 => ipv4_range(ip, prefix),
            _ => Vec::new(),
        };
    }
    match entry.parse::<std::net::Ipv6Addr>() {
        Ok(ip) => vec![format!("[{ip}]")],
        Err(_) => vec![entry.to_string()],
    }
}

/// `10.0.0.0/8` → `10.*`; `172.16.0.0/12` → `172.16.*` to `172.31.*`. A prefix that would need more
/// than `MAX_EXPANSION` patterns is widened to the whole octet above it.
fn ipv4_range(ip: std::net::Ipv4Addr, prefix: u32) -> Vec<String> {
    if prefix == 0 {
        return vec!["*".into()];
    }
    let octets = ip.octets();
    let whole = (prefix / 8) as usize;
    let spare = prefix % 8;
    let head = |n: usize| octets[..n].iter().map(u8::to_string).collect::<Vec<_>>();
    let pattern = |mut fixed: Vec<String>| {
        if fixed.len() < 4 {
            fixed.push("*".into());
        }
        fixed.join(".")
    };
    if spare == 0 || whole == 4 {
        return vec![pattern(head(whole.min(4)))];
    }
    let count = 1u32 << (8 - spare);
    if count > MAX_EXPANSION {
        return vec![pattern(head(whole))];
    }
    let start = u32::from(octets[whole]) & !(count - 1);
    (start..start + count)
        .map(|value| {
            let mut fixed = head(whole);
            fixed.push(value.to_string());
            pattern(fixed)
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/wininet.rs"]
mod tests;
