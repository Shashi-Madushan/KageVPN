//! Server profiles and the protocols that can run them.
//!
//! Every protocol lives in its own module and describes itself with a [`ProtocolSpec`]:
//! how to read its links, write them back, check a profile, and build the sing-box outbound.
//! To add one: create the module, add a [`Protocol`] variant, and list its `SPEC` in
//! [`REGISTRY`]. Nothing else in the backend needs to know about it.

mod common;
mod hysteria2;
mod links;
mod shadowsocks;
mod trojan;
mod vless;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, Result};

pub use links::{contains_link, parse_many, to_link};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    #[default]
    Vless,
    Trojan,
    Shadowsocks,
    Hysteria2,
}

/// Everything the rest of the app needs to know about one protocol.
pub struct ProtocolSpec {
    pub protocol: Protocol,
    /// URL schemes its links use, lowercase.
    pub schemes: &'static [&'static str],
    pub parse: fn(&str) -> Result<Profile>,
    pub to_link: fn(&Profile) -> String,
    /// Protocol-specific checks; the name, address and port are checked for every protocol.
    pub validate: fn(&Profile) -> Result<()>,
    /// The sing-box outbound, without its `tag`.
    pub outbound: fn(&Profile) -> Value,
}

pub static REGISTRY: &[&ProtocolSpec] = &[&vless::SPEC, &trojan::SPEC, &shadowsocks::SPEC, &hysteria2::SPEC];

pub fn spec(protocol: Protocol) -> &'static ProtocolSpec {
    REGISTRY.iter().find(|s| s.protocol == protocol).expect("every protocol is listed in REGISTRY")
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
    pub protocol: Protocol,
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
        (spec(self.protocol).validate)(self)
    }

    /// The sing-box outbound for this server, tagged for the selector.
    pub fn outbound(&self, tag: &str) -> Value {
        let mut outbound = (spec(self.protocol).outbound)(self);
        outbound["tag"] = Value::String(tag.to_string());
        outbound
    }
}
