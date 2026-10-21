//! Server profiles and the protocols that can run them.
//!
//! Every protocol lives in its own module and implements [`ProtocolDef`]: how to read and write
//! its links, check a profile, build the sing-box outbound, say which fields identify a server
//! and which are secret. To add one: create the module and list it in [`REGISTRY`]. Nothing
//! else in the backend needs to know about it.

mod common;
#[cfg(test)]
#[path = "../tests/profile_conformance.rs"]
mod conformance;
mod form;
mod formats;
mod hysteria;
mod hysteria2;
mod links;
mod shadowsocks;
mod ssh;
mod trojan;
mod tuic;
mod vless;
mod vmess;
mod wireguard;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

use crate::error::{AppError, Result};

pub use form::{Field, ProtocolView};
pub use links::{contains_link, parse_all_named, parse_many, to_link};
#[cfg(test)]
pub use links::parse_link;

/// Which protocol a profile uses: the id of an entry in [`REGISTRY`], stored as text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProtocolId(&'static str);

impl ProtocolId {
    pub fn as_str(self) -> &'static str {
        self.0
    }

    /// The registered protocol with this id.
    pub fn lookup(id: &str) -> Option<Self> {
        REGISTRY.iter().find(|def| def.id() == id).map(|def| Self(def.id()))
    }
}

impl Default for ProtocolId {
    fn default() -> Self {
        vless::ID
    }
}

impl PartialEq<&str> for ProtocolId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl Serialize for ProtocolId {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(self.0)
    }
}

impl<'de> Deserialize<'de> for ProtocolId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let id = String::deserialize(deserializer)?;
        Self::lookup(&id).ok_or_else(|| serde::de::Error::custom(format!("unknown protocol '{id}'")))
    }
}

/// Where a server goes in the sing-box config: `outbounds`, or `endpoints` (WireGuard, since
/// sing-box 1.11). Either way the selector and routing refer to it by its tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerKind {
    Outbound,
    Endpoint,
}

/// A WireGuard-shaped key for tests: 32 copies of one byte, made when the test runs, so no
/// key-like value sits in the source for secret scanners to report.
#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) struct TestKey(pub u8);

#[cfg(test)]
impl TestKey {
    /// Percent-encoded, for links.
    pub fn url(self) -> String {
        common::encode_userinfo(&self.to_string())
    }
}

#[cfg(test)]
impl std::fmt::Display for TestKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use base64::Engine;
        f.write_str(&base64::engine::general_purpose::STANDARD.encode([self.0; 32]))
    }
}

/// Private-key-shaped text for tests, assembled when the test runs (no key header in the source).
#[cfg(test)]
pub(crate) fn fake_private_key(body: &str, newline: &str) -> String {
    let kind = ["OPENSSH", "PRIVATE", "KEY"].join(" ");
    format!("-----BEGIN {kind}-----{newline}{body}{newline}-----END {kind}-----{newline}")
}

/// A value that must never reach a feedback report, and the label that replaces it.
pub struct Secret {
    pub value: String,
    pub label: &'static str,
}

impl Secret {
    pub fn new(value: impl Into<String>, label: &'static str) -> Self {
        Self { value: value.into(), label }
    }
}

/// Everything the rest of the app needs to know about one protocol. Every method is required,
/// so adding a protocol and forgetting one (say, which values are secret) does not compile.
pub trait ProtocolDef: Sync {
    /// Stable id, written into saved profiles.
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    /// URL schemes its links use, lowercase.
    fn schemes(&self) -> &'static [&'static str];
    /// Whether it reaches the server over UDP (QUIC and the like), so a TCP connect says
    /// nothing about whether the server is up.
    fn over_udp(&self) -> bool;
    /// Which list of the sing-box config its servers go in.
    fn kind(&self) -> ServerKind;
    /// The sing-box build feature it needs beyond the engine's required ones (`with_quic`,
    /// `with_wireguard`), so a server the engine cannot run is left out with a clear reason.
    fn feature(&self) -> Option<&'static str>;
    fn parse(&self, link: &str) -> Result<Profile>;
    fn to_link(&self, profile: &Profile) -> String;
    /// Protocol-specific checks; the name, address and port are checked for every protocol.
    fn validate(&self, profile: &Profile) -> Result<()>;
    /// The sing-box outbound, without its `tag`.
    fn outbound(&self, profile: &Profile) -> Value;
    /// What makes this the same server after a subscription refresh.
    fn identity(&self, profile: &Profile) -> String;
    /// Credentials and keys, so feedback reports can hide them.
    fn secrets(&self, profile: &Profile) -> Vec<Secret>;
    /// A short description for lists, such as "Reality · grpc".
    fn summary(&self, profile: &Profile) -> String;
    /// An example link, made of documentation-range values. The conformance tests run every
    /// protocol through it, so a protocol that cannot round-trip its own sample fails them.
    #[cfg_attr(not(test), allow(dead_code))] // read by the tests only
    fn sample(&self) -> &'static str;
    /// The fields the editor shows for this protocol, in order.
    fn form(&self) -> Vec<Field>;
    /// A new server of this protocol, with sensible defaults.
    fn blank(&self) -> Profile;
    /// Tidies a profile before it is saved: clears settings that no longer apply and fills
    /// required defaults. Runs before `validate`.
    fn normalize(&self, _profile: &mut Profile) {}
    /// Something the user should know about a server this link imported, such as a default
    /// filled in for a value the link left out. Shown with the import result.
    fn import_note(&self, _link: &str) -> Option<String> {
        None
    }
}

pub static REGISTRY: &[&dyn ProtocolDef] = &[&vless::Vless, &trojan::Trojan, &shadowsocks::Shadowsocks, &hysteria2::Hysteria2, &tuic::Tuic, &hysteria::Hysteria, &vmess::Vmess, &ssh::Ssh, &wireguard::WireGuard];

pub fn def(protocol: ProtocolId) -> &'static dyn ProtocolDef {
    *REGISTRY.iter().find(|d| d.id() == protocol.0).expect("every ProtocolId comes from REGISTRY")
}

pub fn views() -> Vec<ProtocolView> {
    REGISTRY.iter().map(|d| ProtocolView { id: d.id(), label: d.label(), schemes: d.schemes(), fields: d.form() }).collect()
}

/// A profile as the interface sees it: the saved fields plus the list description.
#[derive(Serialize)]
pub struct ProfileView {
    #[serde(flatten)]
    pub profile: Profile,
    pub summary: String,
}

/// The build feature this server needs that an engine with these `tags` lacks. `None` when it
/// has everything, or when the tags are unknown (then `sing-box check` decides).
pub fn missing_feature(profile: &Profile, tags: Option<&[String]>) -> Option<&'static str> {
    let feature = def(profile.protocol).feature()?;
    let tags = tags?;
    (!tags.iter().any(|t| t == feature)).then_some(feature)
}

/// Why a server cannot run on this engine, for the connect error.
pub fn missing_feature_message(profile: &Profile, feature: &str) -> String {
    let what = match feature {
        "with_wireguard" => "WireGuard",
        "with_quic" => "QUIC",
        other => other,
    };
    format!(
        "This sing-box was built without {what} support, which {} servers need. Install the official build on the Engine page.",
        def(profile.protocol).label()
    )
}

/// Splits servers into those the engine can run and those it cannot, keeping their order.
pub fn usable_nodes(nodes: &[Profile], tags: Option<&[String]>) -> (Vec<Profile>, Vec<Profile>) {
    nodes.iter().cloned().partition(|p| missing_feature(p, tags).is_none())
}

/// A blank server for the protocol with this id.
pub fn blank(id: &str) -> Result<Profile> {
    let protocol = ProtocolId::lookup(id).ok_or_else(|| AppError::Invalid(format!("unknown protocol '{id}'")))?;
    Ok(def(protocol).blank())
}

/// One server. The fields every protocol has live here; everything specific to the protocol
/// (credentials, TLS, transport, ...) is in `settings`, which the protocol's own module reads
/// and writes as a typed struct.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile {
    pub id: String,
    pub protocol: ProtocolId,
    pub name: String,
    pub server: String,
    pub port: u16,
    pub settings: Map<String, Value>,
    pub group: Option<String>,
    pub favorite: bool,
    pub subscription_id: Option<String>,
}

impl Profile {
    /// A new profile of `protocol` holding `settings`.
    pub fn build<T: Serialize>(protocol: ProtocolId, name: String, server: String, port: u16, settings: &T) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            protocol,
            name,
            server,
            port,
            settings: to_map(settings),
            ..Self::default()
        }
    }

    /// The protocol's own settings struct, read from the saved block.
    pub fn settings_as<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_value(Value::Object(self.settings.clone()))
            .map_err(|e| AppError::Invalid(format!("the saved settings are not valid: {e}")))
    }

    pub fn put_settings<T: Serialize>(&mut self, settings: &T) {
        self.settings = to_map(settings);
    }

    /// One setting by path, such as `tls.sni`.
    pub fn setting(&self, path: &str) -> Option<&Value> {
        let mut keys = path.split('.');
        let mut value = self.settings.get(keys.next()?)?;
        for key in keys {
            value = value.get(key)?;
        }
        Some(value)
    }

    /// Writes one setting by path, creating the objects along the way.
    pub fn set_setting(&mut self, path: &str, value: Value) {
        let keys: Vec<&str> = path.split('.').collect();
        let (last, parents) = keys.split_last().expect("a path has at least one key");
        let mut node = &mut self.settings;
        for key in parents {
            let entry = node.entry(key.to_string()).or_insert_with(|| Value::Object(Map::new()));
            if !entry.is_object() {
                *entry = Value::Object(Map::new());
            }
            node = entry.as_object_mut().expect("just made an object");
        }
        node.insert(last.to_string(), value);
    }

    /// True when both profiles would connect to the same place the same way, ignoring name,
    /// group and favorite. Used to avoid importing the same link twice.
    pub fn same_endpoint(&self, other: &Profile) -> bool {
        self.protocol == other.protocol
            && self.server.eq_ignore_ascii_case(&other.server)
            && self.port == other.port
            && self.settings == other.settings
    }

    pub fn view(&self) -> ProfileView {
        ProfileView { profile: self.clone(), summary: def(self.protocol).summary(self) }
    }

    pub fn normalize(&mut self) {
        self.name = self.name.trim().to_string();
        self.server = self.server.trim().to_string();
        def(self.protocol).normalize(self);
    }

    pub fn validate(&self) -> Result<()> {
        let invalid = |message: &str| Err(AppError::Invalid(message.into()));
        if self.name.trim().is_empty() {
            return invalid("name cannot be empty");
        }
        if self.server.trim().is_empty() {
            return invalid("server cannot be empty");
        }
        if self.port == 0 {
            return invalid("port must be between 1 and 65535");
        }
        def(self.protocol).validate(self)
    }

    pub fn kind(&self) -> ServerKind {
        def(self.protocol).kind()
    }

    /// The sing-box entry for this server (an outbound, or an endpoint for `ServerKind::Endpoint`),
    /// tagged for the selector.
    pub fn outbound(&self, tag: &str) -> Value {
        let mut outbound = def(self.protocol).outbound(self);
        outbound["tag"] = Value::String(tag.to_string());
        outbound
    }
}

fn to_map<T: Serialize>(settings: &T) -> Map<String, Value> {
    match serde_json::to_value(settings) {
        Ok(Value::Object(map)) => map,
        _ => Map::new(),
    }
}
