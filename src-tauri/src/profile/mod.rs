mod vless;

use serde::{Deserialize, Serialize};

pub use vless::parse_vless;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub server: String,
    pub port: u16,
    pub uuid: String,
    pub sni: Option<String>,
    pub fingerprint: Option<String>,
    pub alpn: Vec<String>,
    pub flow: Option<String>,
    pub pinned_key: Option<String>,
}
