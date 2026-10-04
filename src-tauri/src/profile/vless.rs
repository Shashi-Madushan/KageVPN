use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use url::Url;
use uuid::Uuid;

use super::common::{self, invalid, Params};
use super::{Profile, ProtocolDef, ProtocolId, Reality, Secret, Security};
use crate::error::Result;

pub const ID: ProtocolId = ProtocolId("vless");

pub struct Vless;

impl ProtocolDef for Vless {
    fn id(&self) -> &'static str {
        ID.as_str()
    }

    fn label(&self) -> &'static str {
        "VLESS"
    }

    fn schemes(&self) -> &'static [&'static str] {
        &["vless"]
    }

    fn parse(&self, link: &str) -> Result<Profile> {
        parse_vless(link)
    }

    fn to_link(&self, profile: &Profile) -> String {
        to_link(profile)
    }

    fn validate(&self, profile: &Profile) -> Result<()> {
        validate(profile)
    }

    fn outbound(&self, profile: &Profile) -> Value {
        outbound(profile)
    }

    fn identity(&self, profile: &Profile) -> String {
        format!("{}|{}|{}", profile.server, profile.port, profile.uuid)
    }

    fn secrets(&self, profile: &Profile) -> Vec<Secret> {
        common::secrets(profile, &[(&profile.uuid, "[uuid]")])
    }

    fn summary(&self, profile: &Profile) -> String {
        format!("{} · {}", if profile.security == Security::Reality { "Reality" } else { "TLS" }, common::transport_name(profile.transport.kind))
    }
}

pub fn parse_vless(link: &str) -> Result<Profile> {
    let url = Url::parse(link.trim()).map_err(|e| invalid(format!("not a valid link: {e}")))?;
    if url.scheme() != "vless" {
        return Err(invalid("only vless:// links are supported"));
    }

    let user_id = Uuid::parse_str(url.username()).map_err(|_| invalid("link has no valid UUID"))?;
    let server = common::server_of(&url)?;
    let params = Params::of(&url);

    if let Some(encryption) = params.get("encryption").filter(|e| e != "none") {
        let shown: String = encryption.chars().take(24).collect();
        return Err(invalid(format!("VLESS encryption '{shown}' is not supported, only encryption=none")));
    }
    let security = match params.get("security").as_deref() {
        Some("tls") => Security::Tls,
        Some("reality") => Security::Reality,
        _ => return Err(invalid("only security=tls or reality is supported")),
    };
    let reality = match security {
        Security::Reality => Some(Reality {
            public_key: params.get("pbk").ok_or_else(|| invalid("Reality link has no public key (pbk)"))?,
            short_id: params.get("sid").unwrap_or_default(),
        }),
        _ => None,
    };

    Ok(Profile {
        id: Uuid::new_v4().to_string(),
        protocol: ID,
        name: common::name_of(url.fragment(), &server),
        server,
        port: url.port().unwrap_or(common::DEFAULT_PORT),
        uuid: user_id.to_string(),
        security,
        sni: params.get("sni"),
        fingerprint: params.get("fp").or_else(|| (security == Security::Reality).then(|| "chrome".into())),
        alpn: common::alpn_of(&params),
        flow: params.get("flow"),
        pinned_key: match security {
            Security::Tls => params.get("pcs").and_then(|hex| hex_to_base64(&hex)),
            _ => None,
        },
        transport: common::transport_of(&params)?,
        reality,
        packet_encoding: params.first(&["packetEncoding", "pe"]),
        allow_insecure: params.flag(&["allowInsecure", "insecure"]),
        ..Profile::default()
    })
}

pub fn to_link(profile: &Profile) -> String {
    let mut query = common::query();
    query.append_pair("encryption", "none");
    query.append_pair("security", if profile.security == Security::Reality { "reality" } else { "tls" });
    common::push(&mut query, "flow", &profile.flow);
    common::push(&mut query, "packetEncoding", &profile.packet_encoding);
    common::push_tls_and_transport(&mut query, profile);
    if let Some(reality) = &profile.reality {
        query.append_pair("pbk", &reality.public_key);
        if !reality.short_id.is_empty() {
            query.append_pair("sid", &reality.short_id);
        }
    }
    common::build_link("vless", &profile.uuid, profile, &query.finish())
}

fn validate(profile: &Profile) -> Result<()> {
    if Uuid::parse_str(&profile.uuid).is_err() {
        return Err(invalid("UUID is not valid"));
    }
    if profile.security == Security::Reality && profile.reality.as_ref().is_none_or(|r| r.public_key.trim().is_empty()) {
        return Err(invalid("Reality needs a public key"));
    }
    common::check_transport(profile)
}

fn outbound(profile: &Profile) -> Value {
    let mut outbound = common::base_outbound("vless", profile);
    outbound["uuid"] = json!(profile.uuid);
    if let Some(flow) = profile.flow.as_deref().filter(|f| !f.is_empty()) {
        outbound["flow"] = json!(flow);
    }
    if let Some(encoding) = profile.packet_encoding.as_deref().filter(|e| !e.is_empty()) {
        outbound["packet_encoding"] = json!(encoding);
    }
    outbound
}

/// `pcs` is a certificate hash while sing-box pins public keys, so this value is a
/// placeholder: the first connection reports the server's real key to trust.
fn hex_to_base64(hex: &str) -> Option<String> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    let bytes: Option<Vec<u8>> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect();
    bytes.map(|b| STANDARD.encode(b))
}

#[cfg(test)]
#[path = "../tests/profile_vless.rs"]
mod tests;
