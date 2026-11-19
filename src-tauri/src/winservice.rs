//! The parts of the Windows helper service that are plain logic: the pipe messages, the check the
//! service makes on a configuration before it runs sing-box as SYSTEM, the pipe and service access
//! descriptors, and the PowerShell the app runs (elevated, once) to install or remove the service.
//! None of it touches Windows APIs, so it is tested on every OS.

#![cfg_attr(not(windows), allow(dead_code))]

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SERVICE_NAME: &str = "KageHelper";
pub const PIPE_NAME: &str = r"\\.\pipe\com.apzur.kage.helper";
/// The service's configuration may not be larger than this.
pub const MAX_CONFIG_BYTES: usize = 4 * 1024 * 1024;

/// What the app asks of the service. One JSON object per line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Request {
    Start { config: Value },
    Stop,
}

/// What the service answers. One JSON object per line.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Reply {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Reply {
    pub fn fail(message: impl Into<String>) -> Self {
        Reply { ok: false, error: Some(message.into()), ..Reply::default() }
    }
    pub fn ok() -> Self {
        Reply { ok: true, ..Reply::default() }
    }
}

pub fn parse_request(line: &str) -> Result<Request, String> {
    serde_json::from_str(line.trim()).map_err(|error| format!("unreadable request: {error}"))
}

/// Keys that would make SYSTEM read or write files of the caller's choosing, download a web UI, or
/// load keys; the same list the macOS helper refuses.
const FORBIDDEN_KEYS: [&str; 14] = [
    "external_ui",
    "external_ui_download_url",
    "external_ui_download_detour",
    "cache_file",
    "v2ray_api",
    "certificate_path",
    "certificate_directory_path",
    "key_path",
    "private_key_path",
    "config_path",
    "state_directory",
    "plugin",
    "plugin_opts",
    "ntp",
];

/// Checks a configuration before the service runs sing-box on it. It never changes the
/// configuration, except that the caller's `log.output` is replaced by the service afterwards.
pub fn validate_config(config: &Value) -> Result<(), String> {
    let object = config.as_object().ok_or("the configuration is not a JSON object")?;
    if config.to_string().len() > MAX_CONFIG_BYTES {
        return Err("the configuration is too large".into());
    }
    let has_tun = object.get("inbounds").and_then(Value::as_array).is_some_and(|list| list.iter().any(|i| i.get("type").and_then(Value::as_str) == Some("tun")));
    if !has_tun {
        return Err("the configuration has no tun inbound".into());
    }
    if let Some(key) = find_forbidden_key(config) {
        return Err(format!("the configuration uses \"{key}\", which the helper does not allow"));
    }
    if let Some(sets) = object.get("route").and_then(|r| r.get("rule_set")).and_then(Value::as_array) {
        for set in sets {
            match set.get("type").and_then(Value::as_str) {
                Some("local") => {
                    let path = set.get("path").and_then(Value::as_str).unwrap_or_default();
                    if !is_absolute_windows_path(path) || !path.to_ascii_lowercase().ends_with(".srs") || path.contains("..") {
                        return Err(format!("a rule set path is not allowed: {path}"));
                    }
                }
                _ => return Err("only local rule sets are allowed".into()),
            }
        }
    }
    Ok(())
}

fn find_forbidden_key(value: &Value) -> Option<&'static str> {
    match value {
        Value::Object(map) => {
            for (key, inner) in map {
                if let Some(bad) = FORBIDDEN_KEYS.iter().find(|bad| *bad == key) {
                    return Some(bad);
                }
                if let Some(bad) = find_forbidden_key(inner) {
                    return Some(bad);
                }
            }
            None
        }
        Value::Array(list) => list.iter().find_map(find_forbidden_key),
        _ => None,
    }
}

fn is_absolute_windows_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() > 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/')
}

/// The configuration with `log.output` pointing at the service's own log, which is what runs.
pub fn with_service_log(mut config: Value, log_path: &str) -> Value {
    config["log"]["output"] = Value::String(log_path.to_string());
    config
}

/// Whether `sid` is shaped like a Windows SID, so it can go into a descriptor without surprises.
pub fn is_sid(sid: &str) -> bool {
    let mut parts = sid.split('-');
    parts.next() == Some("S")
        && parts.next() == Some("1")
        && {
            let rest: Vec<&str> = parts.collect();
            rest.len() >= 2 && rest.len() <= 16 && rest.iter().all(|p| !p.is_empty() && p.len() <= 12 && p.bytes().all(|b| b.is_ascii_digit()))
        }
}

/// The SID in the output of `whoami /user /fo csv /nh`: `"DOMAIN\user","S-1-5-21-..."`.
pub fn parse_whoami_sid(output: &str) -> Option<String> {
    let sid = output.trim().rsplit(',').next()?.trim().trim_matches('"').to_string();
    is_sid(&sid).then_some(sid)
}

/// The pipe's access list: SYSTEM and Administrators fully, the installing user read and write.
pub fn pipe_sddl(user_sid: &str) -> Option<String> {
    is_sid(user_sid).then(|| format!("D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GRGW;;;{user_sid})"))
}

/// The service's access list: Windows' default plus the installing user may start it and read its
/// state, so connecting never needs an administrator prompt after the install.
pub fn service_sddl(user_sid: &str) -> Option<String> {
    is_sid(user_sid).then(|| {
        format!("D:(A;;CCLCSWRPWPDTLOCRRC;;;SY)(A;;CCDCLCSWRPWPDTLOCRSDRCWDWO;;;BA)(A;;CCLCSWLOCRRC;;;IU)(A;;CCLCSWLOCRRC;;;SU)(A;;RPLCRC;;;{user_sid})")
    })
}

fn ps_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "''"))
}

/// The script that installs or updates the service. It runs elevated, once. It copies the app and
/// the engine into `%ProgramData%\Kage` (admin-writable only), checks the engine's hash after the
/// copy, and registers the service.
pub fn install_script(app_exe: &str, engine_exe: &str, engine_sha256: &str, user_sid: &str) -> Result<String, String> {
    if !is_sid(user_sid) {
        return Err("the user account could not be identified".into());
    }
    let service_sddl = service_sddl(user_sid).ok_or("bad SID")?;
    Ok(format!(
        r#"$ErrorActionPreference = 'Stop'
$d = Join-Path $env:ProgramData 'Kage'
$name = '{name}'
if (Get-Service -Name $name -ErrorAction SilentlyContinue) {{
  & sc.exe stop $name | Out-Null
  for ($i = 0; $i -lt 40; $i++) {{ if ((Get-Service -Name $name).Status -eq 'Stopped') {{ break }}; Start-Sleep -Milliseconds 250 }}
  & sc.exe delete $name | Out-Null
  Start-Sleep -Milliseconds 500
}}
New-Item -ItemType Directory -Force -Path $d, (Join-Path $d 'engine'), (Join-Path $d 'run') | Out-Null
& icacls.exe $d /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' '*S-1-5-32-545:(OI)(CI)RX' | Out-Null
Copy-Item -LiteralPath {app} -Destination (Join-Path $d 'kage-vpn.exe') -Force
Copy-Item -LiteralPath {engine} -Destination (Join-Path $d 'engine\sing-box.exe') -Force
$hash = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $d 'engine\sing-box.exe')).Hash.ToLower()
if ($hash -ne {sha}) {{ throw 'the engine changed while it was being copied' }}
Set-Content -LiteralPath (Join-Path $d 'engine\sing-box.sha256') -Value $hash -NoNewline
$bin = '"' + (Join-Path $d 'kage-vpn.exe') + '" --service --user-sid {sid}'
New-Service -Name $name -BinaryPathName $bin -DisplayName 'Kage Helper' -StartupType Manual | Out-Null
& sc.exe description $name 'Runs the Kage VPN tunnel for the Kage app.' | Out-Null
& sc.exe sdset $name {sddl} | Out-Null
"#,
        name = SERVICE_NAME,
        app = ps_quote(app_exe),
        engine = ps_quote(engine_exe),
        sha = ps_quote(&engine_sha256.to_ascii_lowercase()),
        sid = user_sid,
        sddl = ps_quote(&service_sddl),
    ))
}

/// The script that removes the service and its folder.
pub fn uninstall_script() -> String {
    format!(
        r#"$ErrorActionPreference = 'SilentlyContinue'
$d = Join-Path $env:ProgramData 'Kage'
$name = '{name}'
& sc.exe stop $name | Out-Null
for ($i = 0; $i -lt 40; $i++) {{ if ((Get-Service -Name $name).Status -eq 'Stopped') {{ break }}; Start-Sleep -Milliseconds 250 }}
& sc.exe delete $name | Out-Null
Start-Sleep -Milliseconds 500
Remove-Item -LiteralPath $d -Recurse -Force
"#,
        name = SERVICE_NAME
    )
}

/// `-EncodedCommand` text for a script: base64 of its UTF-16LE bytes. The script travels on the
/// command line, so there is no file a user-level process could swap before it runs elevated.
pub fn encode_script(script: &str) -> String {
    use base64::Engine;
    let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

#[cfg(test)]
#[path = "tests/winservice.rs"]
mod tests;
