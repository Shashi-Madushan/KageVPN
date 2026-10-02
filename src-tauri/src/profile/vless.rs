use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD, Engine};
use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use url::Url;
use uuid::Uuid;

use super::{Profile, Reality, Security, Transport, TransportKind};
use crate::error::{AppError, Result};

pub fn parse_vless(link: &str) -> Result<Profile> {
    let url = Url::parse(link.trim()).map_err(|e| invalid(format!("not a valid link: {e}")))?;
    if url.scheme() != "vless" {
        return Err(invalid("only vless:// links are supported"));
    }

    let user_id = Uuid::parse_str(url.username()).map_err(|_| invalid("link has no valid UUID"))?;
    let server = url
        .host_str()
        .map(|h| h.trim_matches(['[', ']']))
        .filter(|h| !h.is_empty())
        .ok_or_else(|| invalid("link has no server"))?;
    let params: HashMap<_, _> = url.query_pairs().collect();
    let param = |key: &str| params.get(key).filter(|v| !v.is_empty()).map(|v| v.to_string());

    let security = match param("security").as_deref() {
        Some("tls") => Security::Tls,
        Some("reality") => Security::Reality,
        _ => return Err(invalid("only security=tls or reality is supported")),
    };
    let kind = match param("type").as_deref() {
        None | Some("tcp") => TransportKind::Tcp,
        Some("ws") => TransportKind::Ws,
        Some("grpc") => TransportKind::Grpc,
        Some("httpupgrade") => TransportKind::Httpupgrade,
        Some(other) => return Err(invalid(format!("transport '{other}' is not supported"))),
    };

    let reality = match security {
        Security::Reality => Some(Reality {
            public_key: param("pbk").ok_or_else(|| invalid("Reality link has no public key (pbk)"))?,
            short_id: param("sid").unwrap_or_default(),
        }),
        Security::Tls => None,
    };

    let name = url
        .fragment()
        .map(|f| percent_decode_str(f).decode_utf8_lossy().into_owned())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| server.to_string());

    Ok(Profile {
        id: Uuid::new_v4().to_string(),
        name,
        server: server.to_string(),
        port: url.port().unwrap_or(443),
        uuid: user_id.to_string(),
        security,
        sni: param("sni"),
        fingerprint: param("fp").or_else(|| (security == Security::Reality).then(|| "chrome".into())),
        alpn: param("alpn").map(|a| a.split(',').map(str::to_string).collect()).unwrap_or_default(),
        flow: param("flow"),
        pinned_key: match security {
            Security::Tls => param("pcs").and_then(|hex| hex_to_base64(&hex)),
            Security::Reality => None,
        },
        transport: Transport {
            kind,
            path: param("path"),
            host: param("host"),
            service_name: param("serviceName"),
        },
        reality,
        packet_encoding: param("packetEncoding").or_else(|| param("pe")),
        group: None,
        favorite: false,
        subscription_id: None,
    })
}

/// Parses every `vless://` token in a block of text, returning profiles and the errors.
pub fn parse_many(text: &str) -> (Vec<Profile>, Vec<String>) {
    let mut profiles = Vec::new();
    let mut errors = Vec::new();
    for token in text.split_whitespace().filter(|t| t.starts_with("vless://")) {
        match parse_vless(token) {
            Ok(profile) => profiles.push(profile),
            Err(error) => errors.push(error.to_string()),
        }
    }
    (profiles, errors)
}

pub fn to_link(profile: &Profile) -> String {
    let mut query = url::form_urlencoded::Serializer::new(String::new());
    query.append_pair("encryption", "none");
    query.append_pair("security", if profile.security == Security::Reality { "reality" } else { "tls" });
    let mut optional = |key: &str, value: &Option<String>| {
        if let Some(value) = value.as_deref().filter(|v| !v.is_empty()) {
            query.append_pair(key, value);
        }
    };
    optional("sni", &profile.sni);
    optional("fp", &profile.fingerprint);
    optional("flow", &profile.flow);
    optional("packetEncoding", &profile.packet_encoding);
    optional("path", &profile.transport.path);
    optional("host", &profile.transport.host);
    optional("serviceName", &profile.transport.service_name);
    if !profile.alpn.is_empty() {
        query.append_pair("alpn", &profile.alpn.join(","));
    }
    query.append_pair(
        "type",
        match profile.transport.kind {
            TransportKind::Tcp => "tcp",
            TransportKind::Ws => "ws",
            TransportKind::Grpc => "grpc",
            TransportKind::Httpupgrade => "httpupgrade",
        },
    );
    if let Some(reality) = &profile.reality {
        query.append_pair("pbk", &reality.public_key);
        if !reality.short_id.is_empty() {
            query.append_pair("sid", &reality.short_id);
        }
    }

    let host = if profile.server.contains(':') { format!("[{}]", profile.server) } else { profile.server.clone() };
    format!(
        "vless://{}@{}:{}?{}#{}",
        profile.uuid,
        host,
        profile.port,
        query.finish(),
        utf8_percent_encode(&profile.name, NON_ALPHANUMERIC)
    )
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

fn invalid(message: impl Into<String>) -> AppError {
    AppError::Invalid(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINK: &str = "vless://11111111-2222-4333-8444-555555555555@vpn.example.com:443?encryption=none&security=tls&sni=sni.example.com&fp=chrome&alpn=h2%2Chttp%2F1.1&pcs=0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF&type=tcp&headerType=none#demo";

    #[test]
    fn parses_full_link() {
        let p = parse_vless(LINK).unwrap();
        assert_eq!(p.name, "demo");
        assert_eq!(p.server, "vpn.example.com");
        assert_eq!(p.sni.as_deref(), Some("sni.example.com"));
        assert_eq!(p.alpn, ["h2", "http/1.1"]);
    }

    #[test]
    fn reality_link_round_trips() {
        let link = "vless://11111111-2222-4333-8444-555555555555@example.com:443?security=reality&sni=www.microsoft.com&fp=chrome&pbk=abc&sid=12&flow=xtls-rprx-vision&type=grpc&serviceName=svc#R";
        let p = parse_vless(link).unwrap();
        assert_eq!(p.security, Security::Reality);
        let again = parse_vless(&to_link(&p)).unwrap();
        assert_eq!(again.reality.unwrap().public_key, "abc");
        assert_eq!(again.transport.service_name.as_deref(), Some("svc"));
    }
}
