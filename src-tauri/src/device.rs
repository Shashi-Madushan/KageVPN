//! What this Mac is, for the places that need to say: feedback reports, and subscriptions from
//! providers that limit how many devices may use them.

use std::sync::OnceLock;

#[cfg(not(windows))]
fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(program).args(args).output().ok()?;
    let text = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!text.is_empty()).then_some(text)
}

/// The OS's name for reports and providers.
pub const OS_NAME: &str = if cfg!(windows) { "Windows" } else { "macOS" };

/// Such as `15.4`, or on Windows `23H2 (build 22631)`.
pub fn os_version() -> String {
    static VERSION: OnceLock<String> = OnceLock::new();
    VERSION.get_or_init(|| read_version().unwrap_or_else(|| "unknown".into())).clone()
}

/// The hardware model, such as `Mac14,2` or a PC's product name. It names a kind of computer, not a
/// particular one.
pub fn model() -> String {
    static MODEL: OnceLock<String> = OnceLock::new();
    MODEL.get_or_init(|| read_model().unwrap_or_else(|| if cfg!(windows) { "PC" } else { "Mac" }.into())).clone()
}

#[cfg(not(windows))]
fn read_version() -> Option<String> {
    command_output("sw_vers", &["-productVersion"])
}

#[cfg(not(windows))]
fn read_model() -> Option<String> {
    command_output("sysctl", &["-n", "hw.model"])
}

#[cfg(windows)]
fn local_machine(path: &str, name: &str) -> Option<String> {
    let key = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE).open_subkey(path).ok()?;
    key.get_value::<String, _>(name).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

#[cfg(windows)]
fn read_version() -> Option<String> {
    const KEY: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion";
    let build = local_machine(KEY, "CurrentBuild")?;
    Some(match local_machine(KEY, "DisplayVersion") {
        Some(release) => format!("{release} (build {build})"),
        None => format!("build {build}"),
    })
}

#[cfg(windows)]
fn read_model() -> Option<String> {
    local_machine(r"HARDWARE\DESCRIPTION\System\BIOS", "SystemProductName")
}

/// The headers device-limited subscription panels (Remnawave and the clients that follow Happ's
/// convention) read. `id` is a random identifier made for this installation, not a hardware serial.
pub fn subscription_headers(id: &str) -> Vec<(&'static str, String)> {
    vec![
        ("x-hwid", id.to_string()),
        ("x-device-os", OS_NAME.to_string()),
        ("x-ver-os", os_version()),
        ("x-device-model", model()),
    ]
}

/// A new device ID. Panels accept 10 to 64 letters, digits, `=` and `-`, and silently ignore
/// anything else, so the shape matters.
pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn is_valid_id(id: &str) -> bool {
    (10..=64).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'=' || b == b'-')
}
