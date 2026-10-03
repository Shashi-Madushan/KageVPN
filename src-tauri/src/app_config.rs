//! Everything an operator would change when shipping this app lives in this one file:
//! the app's identity, the cores it can download, where updates and feedback go, and the
//! outside services it talks to. Plain constants, no environment files.
//!
//! An `Option` set to `None` means the feature is off; nothing is ever sent to a placeholder.
//! `tauri.conf.json` (window title, bundle identifier) cannot import Rust and is edited there.

// ── Identity ────────────────────────────────────────────────────────────────

pub const APP_NAME: &str = "SingBox GUI";
pub const APP_SLUG: &str = "singbox-gui";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn user_agent() -> String {
    format!("{APP_SLUG}/{APP_VERSION}")
}

// ── App updates (the GUI itself) ────────────────────────────────────────────

/// URL of the update manifest. `None` disables app updates.
pub const APP_UPDATE_FEED: Option<&str> = None;
/// Public key the update manifest signature is checked against.
pub const APP_UPDATE_PUBKEY: Option<&str> = None;

// ── Feedback ────────────────────────────────────────────────────────────────

/// Backend that receives feedback reports. `None` hides the feedback form.
pub const FEEDBACK_ENDPOINT: Option<&str> = None;
/// Sent as `Authorization: Bearer …` when set. Not a secret in a desktop app, so the server
/// must rate-limit and validate regardless.
#[allow(dead_code)] // read by the feedback sender once it exists
pub const FEEDBACK_TOKEN: Option<&str> = None;

// ── Cores ───────────────────────────────────────────────────────────────────

/// Describes a proxy core the app can download and manage. Adding a core to download is one
/// entry here; running it also needs a runtime adapter, which is why `runnable` exists.
pub struct CoreSpec {
    /// Stable key used in saved settings, folder names and events.
    pub id: &'static str,
    pub name: &'static str,
    /// GitHub-style releases API listing the core's versions.
    pub releases_api: &'static str,
    /// Archive file name for a version on a CPU architecture (`arm64` or `amd64`).
    pub asset_name: fn(version: &str, arch: &str) -> String,
    pub binary_name: &'static str,
    /// Arguments that make the binary print its version and build features.
    pub version_args: &'static [&'static str],
    pub min_version: &'static str,
    /// Build features the binary must report, or the app cannot use it.
    pub required_features: &'static [&'static str],
    /// Places outside the app's own folder where the core may already be installed.
    pub system_paths: &'static [&'static str],
    /// Whether the app can start this core for connections (needs a runtime adapter).
    pub runnable: bool,
}

pub const SING_BOX: CoreSpec = CoreSpec {
    id: "sing-box",
    name: "sing-box",
    releases_api: "https://api.github.com/repos/SagerNet/sing-box/releases?per_page=30",
    asset_name: |version, arch| format!("sing-box-{version}-darwin-{arch}.tar.gz"),
    binary_name: "sing-box",
    version_args: &["version"],
    min_version: "1.12.0",
    required_features: &["with_clash_api", "with_utls"],
    system_paths: &["/opt/homebrew/bin/sing-box", "/usr/local/bin/sing-box"],
    runnable: true,
};

/// Every core the app knows about. The first one is the default.
pub const CORES: &[CoreSpec] = &[SING_BOX];

pub fn core_spec(id: &str) -> Option<&'static CoreSpec> {
    CORES.iter().find(|spec| spec.id == id)
}

pub fn default_core() -> &'static CoreSpec {
    &CORES[0]
}

// ── Outside services ────────────────────────────────────────────────────────

pub mod urls {
    /// Tiny endpoint used to time a connection and confirm it works.
    pub const CONNECTION_PROBE: &str = "http://cp.cloudflare.com/generate_204";
    /// Returns the caller's public IP as plain text.
    pub const EXIT_IP: &str = "https://api.ipify.org";
    /// Returns IP, country and network details as JSON (diagnostics).
    pub const IP_DETAILS: &str = "https://ipwho.is/";
    /// Shows which DNS resolver answered, for leak checks.
    pub const DNS_LEAK: &str = "http://edns.ip-api.com/json";
    pub const SPEED_DOWNLOAD: &str = "https://speed.cloudflare.com/__down?bytes=100000000";
    pub const SPEED_UPLOAD: &str = "https://speed.cloudflare.com/__up";
    /// Prefixes for per-country and per-category rule sets; the file name is appended.
    pub const GEOIP_RULE_SETS: &str = "https://raw.githubusercontent.com/SagerNet/sing-geoip/rule-set/geoip-";
    pub const GEOSITE_RULE_SETS: &str = "https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-";
}
