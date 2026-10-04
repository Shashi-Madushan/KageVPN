//! Server profiles and the protocols that can run them.
//!
//! Every protocol lives in its own module and implements [`ProtocolDef`]: how to read and write
//! its links, check a profile, build the sing-box outbound, say which fields identify a server
//! and which are secret. To add one: create the module and list it in [`REGISTRY`]. Nothing
//! else in the backend needs to know about it.

mod common;
mod hysteria2;
mod links;
mod shadowsocks;
mod trojan;
mod vless;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::error::{AppError, Result};

pub use links::{contains_link, parse_many, to_link};

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
    #[allow(dead_code)] // read by the editor once it is built from these definitions
    fn label(&self) -> &'static str;
    /// URL schemes its links use, lowercase.
    fn schemes(&self) -> &'static [&'static str];
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
    #[allow(dead_code)] // read by the server list once it is built from these definitions
    fn summary(&self, profile: &Profile) -> String;
}

pub static REGISTRY: &[&dyn ProtocolDef] = &[&vless::Vless, &trojan::Trojan, &shadowsocks::Shadowsocks, &hysteria2::Hysteria2];

pub fn def(protocol: ProtocolId) -> &'static dyn ProtocolDef {
    *REGISTRY.iter().find(|d| d.id() == protocol.0).expect("every ProtocolId comes from REGISTRY")
}

pub fn shadowsocks_methods() -> &'static [&'static str] {
    shadowsocks::methods()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Security {
    #[default]
    Tls,
    Reality,
    /// No TLS layer, as with Shadowsocks.
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportKind {
    #[default]
    Tcp,
    Ws,
    Grpc,
    Httpupgrade,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Transport {
    pub kind: TransportKind,
    pub path: Option<String>,
    pub host: Option<String>,
    pub service_name: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Reality {
    pub public_key: String,
    pub short_id: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Obfs {
    pub kind: String,
    pub password: String,
}

/// One server. Fields that only some protocols use say which, and stay empty for the rest;
/// old saved profiles have none of them and load as VLESS.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile {
    pub id: String,
    pub protocol: ProtocolId,
    pub name: String,
    pub server: String,
    pub port: u16,
    /// VLESS user id.
    pub uuid: String,
    /// Trojan, Shadowsocks and Hysteria2 password.
    pub password: String,
    /// Shadowsocks cipher.
    pub method: Option<String>,
    /// Hysteria2 traffic obfuscation.
    pub obfs: Option<Obfs>,
    /// Skip certificate verification. Only set when a link or the user asks for it.
    pub allow_insecure: bool,
    pub security: Security,
    pub sni: Option<String>,
    pub fingerprint: Option<String>,
    pub alpn: Vec<String>,
    pub flow: Option<String>,
    pub pinned_key: Option<String>,
    pub transport: Transport,
    pub reality: Option<Reality>,
    pub packet_encoding: Option<String>,
    pub group: Option<String>,
    pub favorite: bool,
    pub subscription_id: Option<String>,
}

impl Profile {
    /// True when both profiles would connect to the same place the same way, ignoring name,
    /// group and favorite. Used to avoid importing the same link twice.
    pub fn same_endpoint(&self, other: &Profile) -> bool {
        let same_reality = self.reality.as_ref().map(|r| (&r.public_key, &r.short_id))
            == other.reality.as_ref().map(|r| (&r.public_key, &r.short_id));
        self.protocol == other.protocol
            && self.server.eq_ignore_ascii_case(&other.server)
            && self.port == other.port
            && self.uuid.eq_ignore_ascii_case(&other.uuid)
            && self.password == other.password
            && self.method == other.method
            && self.obfs == other.obfs
            && self.allow_insecure == other.allow_insecure
            && self.security == other.security
            && self.sni == other.sni
            && self.flow == other.flow
            && self.transport.kind == other.transport.kind
            && self.transport.path == other.transport.path
            && self.transport.host == other.transport.host
            && self.transport.service_name == other.transport.service_name
            && same_reality
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

    /// The sing-box outbound for this server, tagged for the selector.
    pub fn outbound(&self, tag: &str) -> Value {
        let mut outbound = def(self.protocol).outbound(self);
        outbound["tag"] = Value::String(tag.to_string());
        outbound
    }
}
