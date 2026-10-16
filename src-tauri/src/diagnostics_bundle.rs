//! A zip of everything needed to understand a problem, with anything that identifies the user's
//! servers, subscriptions or machine removed. Saved in Downloads; nothing is sent anywhere.

use std::path::PathBuf;

use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::error::{AppError, Result};
use crate::feedback::redact;
use crate::profile::Profile;
use crate::state::AppState;

/// How many lines of the sing-box log go into the bundle.
const LOG_TAIL_LINES: usize = 2000;
/// Config keys whose values are credentials, removed before the general redaction runs.
const SECRET_KEYS: [&str; 9] = ["secret", "password", "uuid", "private_key", "public_key", "pre_shared_key", "psk", "username", "auth_str"];

/// What the bundle is built from, gathered once so building is a pure function.
pub struct Material {
    pub app_log: String,
    pub singbox_log: String,
    pub config: Option<String>,
    pub versions: String,
    pub settings: String,
    pub profiles: Vec<Profile>,
    pub subscription_urls: Vec<String>,
    pub home: Option<String>,
}

/// The last `max` lines of `text`.
pub fn tail(text: &str, max: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(max)..].join("\n")
}

fn scrub_json(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, entry) in map.iter_mut() {
                if SECRET_KEYS.contains(&key.as_str()) && entry.is_string() {
                    *entry = Value::String("[removed]".into());
                } else {
                    scrub_json(entry);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(scrub_json),
        _ => {}
    }
}

fn hide_subscriptions(text: &str, urls: &[String]) -> String {
    let mut out = text.to_string();
    for url in urls.iter().filter(|u| u.trim().len() >= 8) {
        out = out.replace(url.trim(), "[subscription]");
    }
    out
}

fn clean(text: &str, material: &Material) -> String {
    redact(&hide_subscriptions(text, &material.subscription_urls), &material.profiles, material.home.as_deref())
}

fn clean_config(config: &str, material: &Material) -> String {
    match serde_json::from_str::<Value>(config) {
        Ok(mut value) => {
            scrub_json(&mut value);
            clean(&serde_json::to_string_pretty(&value).unwrap_or_default(), material)
        }
        Err(_) => "[the configuration could not be read]".into(),
    }
}

/// The short text copied to the clipboard.
pub fn summary(material: &Material) -> String {
    clean(&material.versions, material) + "\n" + &clean(&material.settings, material)
}

/// The zip's bytes: `README.txt`, `summary.txt`, `app.log`, `sing-box.log`, `config.json`.
pub fn build(material: &Material) -> Result<Vec<u8>> {
    let config = material.config.as_deref().map(|c| clean_config(c, material)).unwrap_or_else(|| "[no configuration yet]".into());
    let files: [(&str, String); 5] = [
        ("README.txt", "Kage VPN diagnostics. Server addresses, passwords, keys, user IDs, subscription links and your home folder are replaced with placeholders. Nothing was sent anywhere; this file only exists on your Mac.\n".into()),
        ("summary.txt", summary(material)),
        ("app.log", clean(&material.app_log, material)),
        ("sing-box.log", clean(&tail(&material.singbox_log, LOG_TAIL_LINES), material)),
        ("config.json", config),
    ];
    stored_zip(&files.iter().map(|(name, text)| (*name, text.as_bytes())).collect::<Vec<_>>())
}

/// A zip with every file stored without compression. Text this size does not need squeezing, and
/// the format is small enough to write here instead of adding a dependency.
pub fn stored_zip(files: &[(&str, &[u8])]) -> Result<Vec<u8>> {
    fn put16(out: &mut Vec<u8>, v: u16) {
        out.extend_from_slice(&v.to_le_bytes());
    }
    fn put32(out: &mut Vec<u8>, v: u32) {
        out.extend_from_slice(&v.to_le_bytes());
    }
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in files {
        let size = u32::try_from(data.len()).map_err(|_| AppError::Invalid("a diagnostics file is too large".into()))?;
        let mut crc = flate2::Crc::new();
        crc.update(data);
        let offset = u32::try_from(out.len()).map_err(|_| AppError::Invalid("the diagnostics bundle is too large".into()))?;
        // Local file header: version 20, flags 0 (UTF-8 names are ASCII here), stored, DOS date 1980-01-01.
        for (width, value) in [(32, 0x0403_4b50u32), (16, 20), (16, 0), (16, 0), (16, 0), (16, 0x21)] {
            if width == 32 { put32(&mut out, value) } else { put16(&mut out, value as u16) }
        }
        put32(&mut out, crc.sum());
        put32(&mut out, size);
        put32(&mut out, size);
        put16(&mut out, name.len() as u16);
        put16(&mut out, 0);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(data);

        for (width, value) in [(32, 0x0201_4b50u32), (16, 20), (16, 20), (16, 0), (16, 0), (16, 0), (16, 0x21)] {
            if width == 32 { put32(&mut central, value) } else { put16(&mut central, value as u16) }
        }
        put32(&mut central, crc.sum());
        put32(&mut central, size);
        put32(&mut central, size);
        put16(&mut central, name.len() as u16);
        for _ in 0..4 {
            put16(&mut central, 0); // extra, comment, disk, internal attributes
        }
        put32(&mut central, 0); // external attributes
        put32(&mut central, offset);
        central.extend_from_slice(name.as_bytes());
    }
    let central_offset = out.len() as u32;
    let central_size = central.len() as u32;
    out.extend_from_slice(&central);
    put32(&mut out, 0x0605_4b50);
    put16(&mut out, 0);
    put16(&mut out, 0);
    put16(&mut out, files.len() as u16);
    put16(&mut out, files.len() as u16);
    put32(&mut out, central_size);
    put32(&mut out, central_offset);
    put16(&mut out, 0);
    Ok(out)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Exported {
    pub path: String,
    pub summary: String,
}

fn read(path: PathBuf) -> String {
    String::from_utf8_lossy(&std::fs::read(path).unwrap_or_default()).into_owned()
}

fn gather(state: &AppState) -> Material {
    let store = state.store.lock().unwrap();
    let spec = store.active_core();
    let core_version = crate::core::resolve(&state.paths, spec, &store.core_settings())
        .ok()
        .and_then(|(path, source)| crate::core::inspect(spec, &path, source).ok())
        .map(|info| info.version)
        .unwrap_or_else(|| "none".into());
    let routing = store.routing();
    let reliability = store.reliability();
    let status = state.status.lock().unwrap().clone();
    let versions = format!(
        "app {} {}\nmacOS {} ({})\nengine {} {}\nstate {:?}, mode {:?}",
        crate::app_config::APP_NAME,
        crate::app_config::APP_VERSION,
        crate::device::macos_version(),
        std::env::consts::ARCH,
        spec.id,
        core_version,
        status.state,
        status.mode,
    );
    let settings = format!(
        "servers {}, subscriptions {}\nrouting {:?}, {} rule(s), {} list(s), ipv6 {}, bypass lan {}\nkill switch {}, auto reconnect {}, watchdog {}",
        store.profiles().len(),
        store.subscriptions().len(),
        routing.mode,
        routing.rules.len(),
        routing.rule_sets.len(),
        routing.ipv6,
        routing.bypass_lan,
        reliability.kill_switch,
        reliability.auto_reconnect,
        reliability.watchdog,
    );
    Material {
        app_log: read(state.paths.data_dir().join("app.log")),
        singbox_log: read(state.paths.log_file()),
        config: std::fs::read_to_string(state.paths.config_file()).ok(),
        versions,
        settings,
        profiles: store.profiles().to_vec(),
        subscription_urls: store.subscriptions().iter().map(|s| s.url.clone()).collect(),
        home: std::env::var("HOME").ok(),
    }
}

/// Writes the bundle to `~/Downloads/kage-diagnostics-<time>.zip`.
#[tauri::command(async)]
pub fn diagnostics_export(state: State<AppState>) -> Result<Exported> {
    let material = gather(&state);
    let bytes = build(&material)?;
    let home = std::env::var("HOME").map_err(|_| AppError::Invalid("cannot find your home folder".into()))?;
    let dir = PathBuf::from(home).join("Downloads");
    std::fs::create_dir_all(&dir)?;
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let path = dir.join(format!("kage-diagnostics-{stamp}.zip"));
    std::fs::write(&path, bytes)?;
    // Show it in Finder; failing to is harmless.
    let _ = std::process::Command::new("/usr/bin/open").arg("-R").arg(&path).spawn();
    Ok(Exported { path: path.display().to_string(), summary: summary(&material) })
}

#[cfg(test)]
#[path = "tests/diagnostics_bundle.rs"]
mod tests;
