mod vless;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

pub use vless::{parse_many, to_link};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Security {
    #[default]
    Tls,
    Reality,
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub server: String,
    pub port: u16,
    pub uuid: String,
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
        self.server.eq_ignore_ascii_case(&other.server)
            && self.port == other.port
            && self.uuid.eq_ignore_ascii_case(&other.uuid)
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
        if uuid::Uuid::parse_str(&self.uuid).is_err() {
            return invalid("UUID is not valid");
        }
        if self.security == Security::Reality {
            let reality = self.reality.as_ref();
            if reality.is_none_or(|r| r.public_key.trim().is_empty()) {
                return invalid("Reality needs a public key");
            }
        }
        if self.transport.kind == TransportKind::Grpc
            && self.transport.service_name.as_deref().is_none_or(str::is_empty)
        {
            return invalid("gRPC needs a service name");
        }
        Ok(())
    }
}
