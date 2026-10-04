//! Pieces every protocol shares: query parameters, names, and the TLS and transport settings.
//! A protocol module only adds what is specific to it.

use std::collections::HashMap;

use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use url::Url;

use super::form::{Field, Section};
use super::{Profile, Secret};
use crate::error::{AppError, Result};

pub const DEFAULT_PORT: u16 = 443;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportKind {
    #[default]
    Tcp,
    Ws,
    Grpc,
    Httpupgrade,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Transport {
    pub kind: TransportKind,
    pub path: Option<String>,
    pub host: Option<String>,
    pub service_name: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Reality {
    pub public_key: String,
    pub short_id: String,
}

/// The TLS layer, for the protocols that have one.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Tls {
    pub sni: Option<String>,
    pub fingerprint: Option<String>,
    pub alpn: Vec<String>,
    /// Skip certificate verification. Only set when a link or the user asks for it.
    pub allow_insecure: bool,
    pub pinned_key: Option<String>,
    pub reality: Option<Reality>,
}

/// Hysteria2 traffic obfuscation.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Obfs {
    pub kind: String,
    pub password: String,
}

pub fn invalid(message: impl Into<String>) -> AppError {
    AppError::Invalid(message.into())
}

/// A link's query string, with empty values dropped.
pub struct Params(HashMap<String, String>);

impl Params {
    pub fn of(url: &Url) -> Self {
        Self(url.query_pairs().filter(|(_, v)| !v.is_empty()).map(|(k, v)| (k.into_owned(), v.into_owned())).collect())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    pub fn first(&self, keys: &[&str]) -> Option<String> {
        keys.iter().find_map(|key| self.get(key))
    }

    pub fn flag(&self, keys: &[&str]) -> bool {
        self.first(keys).is_some_and(|v| matches!(v.as_str(), "1" | "true"))
    }
}

pub fn server_of(url: &Url) -> Result<String> {
    url.host_str()
        .map(|h| h.trim_matches(['[', ']']))
        .filter(|h| !h.is_empty())
        .map(str::to_string)
        .ok_or_else(|| invalid("link has no server"))
}

/// The label after `#`, or the server address when there is none.
pub fn name_of(fragment: Option<&str>, server: &str) -> String {
    fragment
        .map(|f| percent_decode_str(f).decode_utf8_lossy().into_owned())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| server.to_string())
}

/// The text before `@`, percent-decoded: a password or an id.
pub fn credential_of(url: &Url) -> String {
    let user = percent_decode_str(url.username()).decode_utf8_lossy().into_owned();
    match url.password() {
        Some(password) => format!("{user}:{}", percent_decode_str(password).decode_utf8_lossy()),
        None => user,
    }
}

/// `sni`, `fp`, `alpn` and the insecure flag, as written by v2rayN-style links.
pub fn tls_of(params: &Params) -> Tls {
    Tls {
        sni: params.first(&["sni", "peer"]),
        fingerprint: params.get("fp"),
        alpn: params.get("alpn").map(|a| a.split(',').map(str::to_string).collect()).unwrap_or_default(),
        allow_insecure: params.flag(&["allowInsecure", "insecure"]),
        ..Tls::default()
    }
}

/// `type`, `path`, `host` and `serviceName`.
pub fn transport_of(params: &Params) -> Result<Transport> {
    let kind = match params.get("type").as_deref() {
        None | Some("tcp") => TransportKind::Tcp,
        Some("ws") => TransportKind::Ws,
        Some("grpc") => TransportKind::Grpc,
        Some("httpupgrade") => TransportKind::Httpupgrade,
        Some(other) => return Err(invalid(format!("transport '{other}' is not supported"))),
    };
    Ok(Transport { kind, path: params.get("path"), host: params.get("host"), service_name: params.get("serviceName") })
}

pub fn transport_name(kind: TransportKind) -> &'static str {
    match kind {
        TransportKind::Tcp => "tcp",
        TransportKind::Ws => "ws",
        TransportKind::Grpc => "grpc",
        TransportKind::Httpupgrade => "httpupgrade",
    }
}

pub fn check_transport(transport: &Transport) -> Result<()> {
    if transport.kind == TransportKind::Grpc && transport.service_name.as_deref().is_none_or(str::is_empty) {
        return Err(invalid("gRPC needs a service name"));
    }
    Ok(())
}

pub fn check_password(password: &str) -> Result<()> {
    if password.trim().is_empty() {
        return Err(invalid("password cannot be empty"));
    }
    Ok(())
}

pub fn format_host(server: &str) -> String {
    if server.contains(':') { format!("[{server}]") } else { server.to_string() }
}

/// `scheme://userinfo@host:port?query#name`, with `userinfo` already encoded.
pub fn build_link(scheme: &str, userinfo: &str, profile: &Profile, query: &str) -> String {
    let query = if query.is_empty() { String::new() } else { format!("?{query}") };
    format!(
        "{scheme}://{userinfo}@{}:{}{query}#{}",
        format_host(&profile.server),
        profile.port,
        utf8_percent_encode(&profile.name, NON_ALPHANUMERIC)
    )
}

pub fn encode_userinfo(text: &str) -> String {
    utf8_percent_encode(text, NON_ALPHANUMERIC).to_string()
}

pub type Query = url::form_urlencoded::Serializer<'static, String>;

pub fn query() -> Query {
    url::form_urlencoded::Serializer::new(String::new())
}

pub fn push(query: &mut Query, key: &str, value: &Option<String>) {
    if let Some(value) = value.as_deref().filter(|v| !v.is_empty()) {
        query.append_pair(key, value);
    }
}

/// The shared `sni`, `fp`, `alpn`, `type`, `path`, `host` and `serviceName` parameters.
pub fn push_tls_and_transport(query: &mut Query, tls: &Tls, transport: &Transport) {
    push(query, "sni", &tls.sni);
    push(query, "fp", &tls.fingerprint);
    push(query, "path", &transport.path);
    push(query, "host", &transport.host);
    push(query, "serviceName", &transport.service_name);
    if !tls.alpn.is_empty() {
        query.append_pair("alpn", &tls.alpn.join(","));
    }
    query.append_pair("type", transport_name(transport.kind));
    if tls.allow_insecure {
        query.append_pair("allowInsecure", "1");
    }
}

/// The part of sing-box's `tls` block every TLS protocol needs.
pub fn tls_base(server: &str, tls: &Tls) -> Value {
    let mut block = json!({
        "enabled": true,
        "server_name": tls.sni.as_deref().unwrap_or(server),
    });
    if !tls.alpn.is_empty() {
        block["alpn"] = json!(tls.alpn);
    }
    if tls.allow_insecure {
        block["insecure"] = json!(true);
    }
    block
}

/// `tls_base` plus the browser fingerprint, Reality and certificate pinning that TCP-based
/// protocols can use.
pub fn tls_json(server: &str, tls: &Tls) -> Value {
    let mut block = tls_base(server, tls);
    if let Some(fingerprint) = &tls.fingerprint {
        block["utls"] = json!({ "enabled": true, "fingerprint": fingerprint });
    }
    match &tls.reality {
        Some(reality) => {
            block["reality"] = json!({
                "enabled": true,
                "public_key": reality.public_key,
                "short_id": reality.short_id,
            });
        }
        None => {
            if let Some(key) = &tls.pinned_key {
                block["certificate_public_key_sha256"] = json!([key]);
            }
        }
    }
    block
}

pub fn transport_json(transport: &Transport) -> Option<Value> {
    let path = transport.path.as_deref().filter(|p| !p.is_empty());
    let host = transport.host.as_deref().filter(|h| !h.is_empty());
    match transport.kind {
        TransportKind::Tcp => None,
        TransportKind::Ws => {
            let mut value = json!({ "type": "ws", "path": path.unwrap_or("/") });
            if let Some(host) = host {
                value["headers"] = json!({ "Host": host });
            }
            Some(value)
        }
        TransportKind::Grpc => Some(json!({
            "type": "grpc",
            "service_name": transport.service_name.as_deref().unwrap_or_default(),
        })),
        TransportKind::Httpupgrade => {
            let mut value = json!({ "type": "httpupgrade", "path": path.unwrap_or("/") });
            if let Some(host) = host {
                value["host"] = json!(host);
            }
            Some(value)
        }
    }
}

/// Fields of a typical TLS-over-TCP outbound: address, TLS block and transport.
pub fn base_outbound(kind: &str, profile: &Profile, tls: &Tls, transport: &Transport) -> Value {
    let mut outbound = json!({
        "type": kind,
        "server": profile.server,
        "server_port": profile.port,
        "tls": tls_json(&profile.server, tls),
    });
    if let Some(transport) = transport_json(transport) {
        outbound["transport"] = transport;
    }
    outbound
}

/// The values a TLS profile exposes besides its credential: the front domain, certificate and
/// Reality keys, and the transport host. `credentials` are the protocol's own secrets.
pub fn secrets(credentials: &[(&str, &'static str)], tls: &Tls, transport: Option<&Transport>) -> Vec<Secret> {
    let mut secrets: Vec<Secret> = credentials.iter().map(|(value, label)| Secret::new(*value, label)).collect();
    let optional = [
        (&tls.sni, "[sni]"),
        (&tls.pinned_key, "[key]"),
        (&transport.and_then(|t| t.host.clone()), "[host]"),
    ];
    secrets.extend(optional.into_iter().filter_map(|(value, label)| value.as_deref().map(|v| Secret::new(v, label))));
    if let Some(reality) = &tls.reality {
        secrets.push(Secret::new(reality.public_key.clone(), "[key]"));
        secrets.push(Secret::new(reality.short_id.clone(), "[id]"));
    }
    secrets
}

const FINGERPRINTS: &[(&str, &str)] =
    &[("chrome", "chrome"), ("firefox", "firefox"), ("safari", "safari"), ("ios", "ios"), ("android", "android"), ("edge", "edge"), ("random", "random")];

pub fn sni_field() -> Field {
    Field::text("tls.sni", "Server name (SNI)", Section::Security).optional().placeholder("Same as the address")
}

pub fn insecure_field() -> Field {
    Field::toggle("tls.allowInsecure", "Skip certificate check", Section::Security)
        .warning("Anyone on the network could impersonate this server. Only use this for a server you run yourself.")
}

pub fn fingerprint_field() -> Field {
    Field::select("tls.fingerprint", "Fingerprint", Section::Advanced, FINGERPRINTS)
}

pub fn alpn_field() -> Field {
    Field::list("tls.alpn", "ALPN", Section::Advanced).optional().placeholder("h2,http/1.1")
}

/// Transport choice and the settings each transport needs.
pub fn transport_fields() -> Vec<Field> {
    vec![
        Field::segmented(
            "transport.kind",
            "Transport",
            Section::Transport,
            &[("tcp", "TCP"), ("ws", "WebSocket"), ("grpc", "gRPC"), ("httpupgrade", "HTTPUpgrade")],
        ),
        Field::text("transport.path", "Path", Section::Transport).optional().mono().placeholder("/").show_when("transport.kind", &["ws", "httpupgrade"]),
        Field::text("transport.host", "Host header", Section::Transport).optional().mono().show_when("transport.kind", &["ws", "httpupgrade"]),
        Field::text("transport.serviceName", "Service name", Section::Transport).mono().show_when("transport.kind", &["grpc"]),
    ]
}
