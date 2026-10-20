use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use url::Url;
use uuid::Uuid;

use super::common::{self, invalid, Params, Reality, Tls, Transport};
use super::form::{Field, Section};
use super::{Profile, ProtocolDef, ProtocolId, Secret};
use crate::error::Result;

pub const ID: ProtocolId = ProtocolId("vless");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Security {
    #[default]
    Tls,
    Reality,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VlessSettings {
    pub uuid: String,
    pub security: Security,
    pub flow: Option<String>,
    pub packet_encoding: Option<String>,
    pub tls: Tls,
    pub transport: Transport,
}

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
        let settings: VlessSettings = profile.settings_as()?;
        if Uuid::parse_str(&settings.uuid).is_err() {
            return Err(invalid("UUID is not valid"));
        }
        if settings.security == Security::Reality && settings.tls.reality.as_ref().is_none_or(|r| r.public_key.trim().is_empty()) {
            return Err(invalid("Reality needs a public key"));
        }
        common::check_transport(&settings.transport)
    }

    fn outbound(&self, profile: &Profile) -> Value {
        let settings: VlessSettings = profile.settings_as().unwrap_or_default();
        let mut outbound = common::base_outbound("vless", profile, &settings.tls, &settings.transport);
        outbound["uuid"] = json!(settings.uuid);
        if let Some(flow) = settings.flow.as_deref().filter(|f| !f.is_empty()) {
            outbound["flow"] = json!(flow);
        }
        if let Some(encoding) = settings.packet_encoding.as_deref().filter(|e| !e.is_empty()) {
            outbound["packet_encoding"] = json!(encoding);
        }
        outbound
    }

    fn identity(&self, profile: &Profile) -> String {
        let settings: VlessSettings = profile.settings_as().unwrap_or_default();
        format!("{}|{}|{}", profile.server, profile.port, settings.uuid)
    }

    fn secrets(&self, profile: &Profile) -> Vec<Secret> {
        let settings: VlessSettings = profile.settings_as().unwrap_or_default();
        common::secrets(&[(&settings.uuid, "[uuid]")], &settings.tls, Some(&settings.transport))
    }

    fn summary(&self, profile: &Profile) -> String {
        let settings: VlessSettings = profile.settings_as().unwrap_or_default();
        let security = if settings.security == Security::Reality { "Reality" } else { "TLS" };
        format!("{security} · {}", common::transport_name(settings.transport.kind))
    }

    fn sample(&self) -> &'static str {
        "vless://11111111-2222-4333-8444-555555555555@vless.example.com:443?encryption=none&security=reality&sni=www.example.org&fp=chrome&pbk=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&sid=a1b2c3d4&flow=xtls-rprx-vision&type=grpc&serviceName=svc#Sample%20VLESS"
    }

    fn form(&self) -> Vec<Field> {
        let mut fields = vec![
            Field::text("uuid", "User ID (UUID)", Section::Connection)
                .mono()
                .placeholder("00000000-0000-0000-0000-000000000000")
                .pattern("^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$", "This is not a valid UUID."),
            Field::segmented("security", "Security", Section::Security, &[("tls", "TLS"), ("reality", "Reality")]),
            common::sni_field(),
            Field::text("tls.reality.publicKey", "Public key", Section::Security).mono().show_when("security", &["reality"]),
            Field::text("tls.reality.shortId", "Short ID", Section::Security).optional().mono().show_when("security", &["reality"]),
            common::insecure_field().show_when("security", &["tls"]),
        ];
        fields.extend(common::transport_fields());
        fields.push(common::fingerprint_field());
        fields.push(Field::select("flow", "Flow", Section::Advanced, &[("", "None"), ("xtls-rprx-vision", "xtls-rprx-vision")]).optional());
        fields.push(common::alpn_field().show_when("security", &["tls"]));
        fields.push(
            Field::select("packetEncoding", "Packet encoding", Section::Advanced, &[("", "Default"), ("xudp", "xudp"), ("packetaddr", "packetaddr")]).optional(),
        );
        fields
    }

    fn blank(&self) -> Profile {
        let settings = VlessSettings { tls: Tls { fingerprint: Some("chrome".into()), ..Tls::default() }, ..VlessSettings::default() };
        Profile::build(ID, String::new(), String::new(), common::DEFAULT_PORT, &settings)
    }

    fn normalize(&self, profile: &mut Profile) {
        let Ok(mut settings) = profile.settings_as::<VlessSettings>() else { return };
        match settings.security {
            Security::Reality => {
                settings.tls.reality.get_or_insert_with(Reality::default);
                settings.tls.allow_insecure = false;
                settings.tls.pinned_key = None;
                if settings.flow.as_deref().is_none_or(str::is_empty) {
                    settings.flow = Some("xtls-rprx-vision".into());
                }
            }
            Security::Tls => settings.tls.reality = None,
        }
        profile.put_settings(&settings);
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

    let mut tls = common::tls_of(&params);
    match security {
        Security::Reality => {
            tls.reality = Some(Reality {
                public_key: params.get("pbk").ok_or_else(|| invalid("Reality link has no public key (pbk)"))?,
                short_id: params.get("sid").unwrap_or_default(),
            });
            tls.fingerprint.get_or_insert_with(|| "chrome".into());
        }
        Security::Tls => tls.pinned_key = params.get("pcs").and_then(|hex| hex_to_base64(&hex)),
    }

    let settings = VlessSettings {
        uuid: user_id.to_string(),
        security,
        flow: params.get("flow"),
        packet_encoding: params.first(&["packetEncoding", "pe"]),
        tls,
        transport: common::transport_of(&params)?,
    };
    Ok(Profile::build(
        ID,
        common::name_of(url.fragment(), &server),
        server,
        url.port().unwrap_or(common::DEFAULT_PORT),
        &settings,
    ))
}

pub fn to_link(profile: &Profile) -> String {
    let settings: VlessSettings = profile.settings_as().unwrap_or_default();
    let mut query = common::query();
    query.append_pair("encryption", "none");
    query.append_pair("security", if settings.security == Security::Reality { "reality" } else { "tls" });
    common::push(&mut query, "flow", &settings.flow);
    common::push(&mut query, "packetEncoding", &settings.packet_encoding);
    common::push_tls_and_transport(&mut query, &settings.tls, &settings.transport);
    if let Some(reality) = &settings.tls.reality {
        query.append_pair("pbk", &reality.public_key);
        if !reality.short_id.is_empty() {
            query.append_pair("sid", &reality.short_id);
        }
    }
    common::build_link("vless", &settings.uuid, profile, &query.finish())
}

/// `pcs` is a certificate hash while sing-box pins public keys, so this value is a
/// placeholder: the first connection reports the server's real key to trust.
fn hex_to_base64(hex: &str) -> Option<String> {
    // Only ASCII hex digits get past this check, so the two-byte slices below always fall on
    // character boundaries; slicing arbitrary text here used to panic and end the whole app.
    if !hex.len().is_multiple_of(2) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
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
