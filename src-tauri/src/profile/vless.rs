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

    if let Some(encryption) = param("encryption").filter(|e| e != "none") {
        let shown: String = encryption.chars().take(24).collect();
        return Err(invalid(format!("VLESS encryption '{shown}' is not supported, only encryption=none")));
    }
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
    for token in text.split_whitespace().filter(|t| is_vless_link(t)) {
        match parse_vless(token) {
            Ok(profile) => profiles.push(profile),
            Err(error) => errors.push(error.to_string()),
        }
    }
    (profiles, errors)
}

/// Schemes are case-insensitive, so `VLESS://` counts too.
pub fn is_vless_link(token: &str) -> bool {
    token.get(..8).is_some_and(|scheme| scheme.eq_ignore_ascii_case("vless://"))
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

    const ID: &str = "11111111-2222-4333-8444-555555555555";

    #[test]
    fn websocket_link_round_trips_path_alpn_and_name() {
        let link = format!("vless://{ID}@h.example.com:8443?security=tls&type=ws&path=%2Fws%3Fed%3D2048&host=cdn.example.com&alpn=h2%2Chttp%2F1.1#WS%20Node");
        let p = parse_vless(&link).unwrap();
        assert_eq!((p.port, p.name.as_str()), (8443, "WS Node"));
        let again = parse_vless(&to_link(&p)).unwrap();
        assert_eq!(again.transport.path.as_deref(), Some("/ws?ed=2048"));
        assert_eq!(again.transport.host.as_deref(), Some("cdn.example.com"));
        assert_eq!(again.alpn, ["h2", "http/1.1"]);
    }

    #[test]
    fn handles_ipv6_default_port_emoji_and_missing_name() {
        let v6 = parse_vless(&format!("vless://{ID}@[2001:db8::1]:443?security=tls#v6")).unwrap();
        assert_eq!(v6.server, "2001:db8::1");
        assert!(to_link(&v6).contains("@[2001:db8::1]:443"));

        let plain = parse_vless(&format!("vless://{ID}@h.example.com?security=tls")).unwrap();
        assert_eq!((plain.port, plain.name.as_str()), (443, "h.example.com"));

        let flag = parse_vless(&format!("vless://{ID}@h.example.com:443?security=tls#%F0%9F%87%A9%F0%9F%87%AA%20Frankfurt")).unwrap();
        assert_eq!(flag.name, "🇩🇪 Frankfurt");
    }

    #[test]
    fn rejects_links_it_cannot_run() {
        for link in [
            format!("vless://{ID}@h.example.com:443?security=none&type=tcp"),
            format!("vless://{ID}@h.example.com:443?type=tcp"),
            format!("vless://{ID}@h.example.com:443?security=tls&type=xhttp"),
            format!("vless://{ID}@h.example.com:443?security=reality"),
            "vless://not-a-uuid@h.example.com:443?security=tls".to_string(),
        ] {
            assert!(parse_vless(&link).is_err(), "{link}");
        }
    }

    #[test]
    fn rejects_vless_encryption_it_would_ignore() {
        let link = format!("vless://{ID}@h.example.com:443?security=tls&encryption=mlkem768x25519plus.native.0rtt.abc");
        let error = parse_vless(&link).unwrap_err().to_string();
        assert!(error.contains("encryption"), "{error}");
        assert!(parse_vless(&format!("vless://{ID}@h.example.com:443?security=tls&encryption=none")).is_ok());
    }

    #[test]
    fn bulk_import_accepts_an_uppercase_scheme() {
        let (profiles, errors) = parse_many(&format!("VLESS://{ID}@a.example.com:443?security=tls#A vless://{ID}@b.example.com:443?security=tls#B"));
        assert_eq!((profiles.len(), errors.len()), (2, 0));
    }
}
