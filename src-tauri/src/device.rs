//! What this Mac is, for the places that need to say: feedback reports, and subscriptions from
//! providers that limit how many devices may use them.

use std::process::Command;
use std::sync::OnceLock;

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    let text = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!text.is_empty()).then_some(text)
}

/// Such as `15.4`.
pub fn macos_version() -> String {
    static VERSION: OnceLock<String> = OnceLock::new();
    VERSION.get_or_init(|| command_output("sw_vers", &["-productVersion"]).unwrap_or_else(|| "unknown".into())).clone()
}

/// The hardware model identifier, such as `Mac14,2`. It names a kind of Mac, not a particular one.
pub fn model() -> String {
    static MODEL: OnceLock<String> = OnceLock::new();
    MODEL.get_or_init(|| command_output("sysctl", &["-n", "hw.model"]).unwrap_or_else(|| "Mac".into())).clone()
}

/// The headers device-limited subscription panels (Remnawave and the clients that follow Happ's
/// convention) read. `id` is a random identifier made for this installation, not a hardware serial.
pub fn subscription_headers(id: &str) -> Vec<(&'static str, String)> {
    vec![
        ("x-hwid", id.to_string()),
        ("x-device-os", "macOS".to_string()),
        ("x-ver-os", macos_version()),
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
