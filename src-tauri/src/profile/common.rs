//! Pieces every protocol shares: query parameters, names, TLS and transport settings.
//! A protocol module only adds what is specific to it.

use std::collections::HashMap;

use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use serde_json::{json, Value};
use url::Url;

use super::{Profile, Security, Transport, TransportKind};
use crate::error::{AppError, Result};

pub const DEFAULT_PORT: u16 = 443;

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

pub fn alpn_of(params: &Params) -> Vec<String> {
    params.get("alpn").map(|a| a.split(',').map(str::to_string).collect()).unwrap_or_default()
}

/// `type`, `path`, `host` and `serviceName`, as written by v2rayN-style links.
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

pub fn check_transport(profile: &Profile) -> Result<()> {
    if profile.transport.kind == TransportKind::Grpc && profile.transport.service_name.as_deref().is_none_or(str::is_empty) {
        return Err(invalid("gRPC needs a service name"));
    }
    Ok(())
}

pub fn check_password(profile: &Profile) -> Result<()> {
    if profile.password.trim().is_empty() {
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
pub fn push_tls_and_transport(query: &mut Query, profile: &Profile) {
    push(query, "sni", &profile.sni);
    push(query, "fp", &profile.fingerprint);
    push(query, "path", &profile.transport.path);
    push(query, "host", &profile.transport.host);
    push(query, "serviceName", &profile.transport.service_name);
    if !profile.alpn.is_empty() {
        query.append_pair("alpn", &profile.alpn.join(","));
    }
    query.append_pair("type", transport_name(profile.transport.kind));
    if profile.allow_insecure {
        query.append_pair("allowInsecure", "1");
    }
}

/// The part of sing-box's `tls` block every TLS protocol needs.
pub fn tls_base(profile: &Profile) -> Value {
    let mut tls = json!({
        "enabled": true,
        "server_name": profile.sni.as_deref().unwrap_or(&profile.server),
    });
    if !profile.alpn.is_empty() {
        tls["alpn"] = json!(profile.alpn);
    }
    if profile.allow_insecure {
        tls["insecure"] = json!(true);
    }
    tls
}

/// `tls_base` plus the browser fingerprint, Reality and certificate pinning that TCP-based
/// protocols can use.
pub fn tls_json(profile: &Profile) -> Value {
    let mut tls = tls_base(profile);
    if let Some(fingerprint) = &profile.fingerprint {
        tls["utls"] = json!({ "enabled": true, "fingerprint": fingerprint });
    }
    match (profile.security, &profile.reality) {
        (Security::Reality, Some(reality)) => {
            tls["reality"] = json!({
                "enabled": true,
                "public_key": reality.public_key,
                "short_id": reality.short_id,
            });
        }
        _ => {
            if let Some(key) = &profile.pinned_key {
                tls["certificate_public_key_sha256"] = json!([key]);
            }
        }
    }
    tls
}

pub fn transport_json(profile: &Profile) -> Option<Value> {
    let t = &profile.transport;
    let path = t.path.as_deref().filter(|p| !p.is_empty());
    let host = t.host.as_deref().filter(|h| !h.is_empty());
    match t.kind {
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
            "service_name": t.service_name.as_deref().unwrap_or_default(),
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
pub fn base_outbound(kind: &str, profile: &Profile) -> Value {
    let mut outbound = json!({
        "type": kind,
        "server": profile.server,
        "server_port": profile.port,
        "tls": tls_json(profile),
    });
    if let Some(transport) = transport_json(profile) {
        outbound["transport"] = transport;
    }
    outbound
}
