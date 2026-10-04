use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use super::common::{self, invalid, Tls, Transport, TransportKind};
use super::form::{Field, Section};
use super::{Profile, ProtocolDef, ProtocolId, Secret};
use crate::error::Result;

pub const ID: ProtocolId = ProtocolId("vmess");

/// Encryption methods sing-box accepts for VMess.
const CIPHERS: &[&str] = &["auto", "aes-128-gcm", "chacha20-poly1305", "none", "zero"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VmessSettings {
    pub uuid: String,
    pub security: String,
    pub tls_enabled: bool,
    pub tls: Tls,
    pub transport: Transport,
}

impl Default for VmessSettings {
    fn default() -> Self {
        Self { uuid: String::new(), security: "auto".into(), tls_enabled: false, tls: Tls::default(), transport: Transport::default() }
    }
}

pub struct Vmess;

impl ProtocolDef for Vmess {
    fn id(&self) -> &'static str {
        ID.as_str()
    }

    fn label(&self) -> &'static str {
        "VMess"
    }

    fn schemes(&self) -> &'static [&'static str] {
        &["vmess"]
    }

    fn parse(&self, link: &str) -> Result<Profile> {
        parse(link)
    }

    fn to_link(&self, profile: &Profile) -> String {
        to_link(profile)
    }

    fn validate(&self, profile: &Profile) -> Result<()> {
        let settings: VmessSettings = profile.settings_as()?;
        if Uuid::parse_str(&settings.uuid).is_err() {
            return Err(invalid("UUID is not valid"));
        }
        if !CIPHERS.contains(&settings.security.as_str()) {
            return Err(invalid(format!("encryption '{}' is not supported", settings.security)));
        }
        common::check_transport(&settings.transport)
    }

    fn outbound(&self, profile: &Profile) -> Value {
        let settings: VmessSettings = profile.settings_as().unwrap_or_default();
        let mut outbound = json!({
            "type": "vmess",
            "server": profile.server,
            "server_port": profile.port,
            "uuid": settings.uuid,
            "security": settings.security,
            "alter_id": 0,
        });
        if settings.tls_enabled {
            outbound["tls"] = common::tls_json(&profile.server, &settings.tls);
        }
        if let Some(transport) = common::transport_json(&settings.transport) {
            outbound["transport"] = transport;
        }
        outbound
    }

    fn identity(&self, profile: &Profile) -> String {
        let settings: VmessSettings = profile.settings_as().unwrap_or_default();
        format!("{}|{}|{}", profile.server, profile.port, settings.uuid)
    }

    fn secrets(&self, profile: &Profile) -> Vec<Secret> {
        let settings: VmessSettings = profile.settings_as().unwrap_or_default();
        common::secrets(&[(&settings.uuid, "[uuid]")], &settings.tls, Some(&settings.transport))
    }

    fn summary(&self, profile: &Profile) -> String {
        let settings: VmessSettings = profile.settings_as().unwrap_or_default();
        let tls = if settings.tls_enabled { "TLS" } else { "No TLS" };
        format!("VMess · {tls} · {}", common::transport_name(settings.transport.kind))
    }

    fn sample(&self) -> &'static str {
        "vmess://eyJ2IjoiMiIsInBzIjoiU2FtcGxlIFZNZXNzIiwiYWRkIjoidm1lc3MuZXhhbXBsZS5jb20iLCJwb3J0IjoiNDQzIiwiaWQiOiIxMTExMTExMS0yMjIyLTQzMzMtODQ0NC01NTU1NTU1NTU1NTUiLCJhaWQiOiIwIiwic2N5IjoiYXV0byIsIm5ldCI6IndzIiwidHlwZSI6Im5vbmUiLCJob3N0IjoiY2RuLmV4YW1wbGUuY29tIiwicGF0aCI6Ii93cyIsInRscyI6InRscyIsInNuaSI6InZtZXNzLmV4YW1wbGUuY29tIiwiYWxwbiI6ImgyIiwiZnAiOiJjaHJvbWUifQ=="
    }

    fn form(&self) -> Vec<Field> {
        let ciphers: Vec<(&str, &str)> = CIPHERS.iter().map(|c| (*c, *c)).collect();
        let tls = |field: Field| field.show_when("tlsEnabled", &["true"]);
        let mut fields = vec![
            Field::text("uuid", "User ID (UUID)", Section::Connection)
                .mono()
                .placeholder("00000000-0000-0000-0000-000000000000")
                .pattern("^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$", "This is not a valid UUID."),
            Field::select("security", "Encryption", Section::Connection, &ciphers),
            Field::toggle("tlsEnabled", "Use TLS", Section::Security),
            tls(common::sni_field()),
            tls(common::insecure_field()),
        ];
        fields.extend(common::transport_fields());
        fields.push(tls(common::fingerprint_field()));
        fields.push(tls(common::alpn_field()));
        fields
    }

    fn blank(&self) -> Profile {
        let settings = VmessSettings { tls: Tls { fingerprint: Some("chrome".into()), ..Tls::default() }, ..VmessSettings::default() };
        Profile::build(ID, String::new(), String::new(), common::DEFAULT_PORT, &settings)
    }
}

/// Reads the v2rayN share format: `vmess://` followed by base64 of a JSON object. Numbers
/// are often written as text, so both forms are accepted.
fn parse(link: &str) -> Result<Profile> {
    let body = link.trim().get(8..).filter(|_| link.trim().get(..8).is_some_and(|s| s.eq_ignore_ascii_case("vmess://")));
    let body = body.ok_or_else(|| invalid("not a vmess:// link"))?;
    let json = decode(body.split('#').next().unwrap_or_default())?;
    let text = |key: &str| json.get(key).and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty()).map(str::to_string);
    let number = |key: &str| {
        json.get(key).and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())))
    };

    let server = text("add").ok_or_else(|| invalid("link has no server"))?;
    let port = number("port").and_then(|p| u16::try_from(p).ok()).filter(|p| *p != 0).ok_or_else(|| invalid("link has an invalid port"))?;
    let uuid = Uuid::parse_str(&text("id").unwrap_or_default()).map_err(|_| invalid("link has no valid UUID"))?;
    if number("aid").unwrap_or(0) != 0 {
        return Err(invalid("legacy VMess (alterId above 0) is not supported"));
    }

    let net = text("net").unwrap_or_else(|| "tcp".into());
    let kind = match net.as_str() {
        "tcp" => TransportKind::Tcp,
        "ws" => TransportKind::Ws,
        "grpc" => TransportKind::Grpc,
        "httpupgrade" => TransportKind::Httpupgrade,
        other => return Err(invalid(format!("transport '{other}' is not supported"))),
    };
    let path = text("path");
    let transport = match kind {
        // v2rayN keeps a gRPC service name in the path field.
        TransportKind::Grpc => Transport { kind, service_name: path, ..Transport::default() },
        _ => Transport { kind, path, host: text("host"), service_name: None },
    };
    let tls_enabled = matches!(text("tls").as_deref(), Some("tls"));
    let tls = Tls {
        sni: text("sni"),
        fingerprint: text("fp"),
        alpn: text("alpn").map(|a| a.split(',').map(str::to_string).collect()).unwrap_or_default(),
        ..Tls::default()
    };

    let settings = VmessSettings { uuid: uuid.to_string(), security: text("scy").unwrap_or_else(|| "auto".into()), tls_enabled, tls, transport };
    let name = common::name_of(text("ps").as_deref().map(|n| n.replace('%', "%25")).as_deref(), &server);
    Ok(Profile::build(ID, name, server, port, &settings))
}

fn to_link(profile: &Profile) -> String {
    let settings: VmessSettings = profile.settings_as().unwrap_or_default();
    let t = &settings.transport;
    let path = if t.kind == TransportKind::Grpc { &t.service_name } else { &t.path };
    let json = json!({
        "v": "2",
        "ps": profile.name,
        "add": profile.server,
        "port": profile.port.to_string(),
        "id": settings.uuid,
        "aid": "0",
        "scy": settings.security,
        "net": common::transport_name(t.kind),
        "type": "none",
        "host": t.host.clone().unwrap_or_default(),
        "path": path.clone().unwrap_or_default(),
        "tls": if settings.tls_enabled { "tls" } else { "" },
        "sni": settings.tls.sni.clone().unwrap_or_default(),
        "alpn": settings.tls.alpn.join(","),
        "fp": settings.tls.fingerprint.clone().unwrap_or_default(),
    });
    format!("vmess://{}", STANDARD.encode(json.to_string()))
}

fn decode(text: &str) -> Result<Value> {
    let compact: String = text.split_whitespace().collect();
    let bytes = [&STANDARD, &STANDARD_NO_PAD, &URL_SAFE, &URL_SAFE_NO_PAD]
        .iter()
        .find_map(|engine| engine.decode(compact.trim_end_matches('=')).ok().or_else(|| engine.decode(&compact).ok()))
        .ok_or_else(|| invalid("link is not valid VMess"))?;
    serde_json::from_slice(&bytes).map_err(|_| invalid("link is not valid VMess"))
}

#[cfg(test)]
#[path = "../tests/profile_vmess.rs"]
mod tests;
