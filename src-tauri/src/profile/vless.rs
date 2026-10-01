use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD, Engine};
use percent_encoding::percent_decode_str;
use url::Url;
use uuid::Uuid;

use super::Profile;
use crate::error::{AppError, Result};

pub fn parse_vless(link: &str) -> Result<Profile> {
    let url = Url::parse(link.trim()).map_err(|e| invalid(format!("not a valid link: {e}")))?;
    if url.scheme() != "vless" {
        return Err(invalid("only vless:// links are supported"));
    }

    let user_id = Uuid::parse_str(url.username()).map_err(|_| invalid("link has no valid UUID"))?;
    let server = url.host_str().ok_or_else(|| invalid("link has no server"))?;
    let params: HashMap<_, _> = url.query_pairs().collect();
    let param = |key: &str| params.get(key).filter(|v| !v.is_empty()).map(|v| v.to_string());

    if param("security").as_deref() != Some("tls") {
        return Err(invalid("only security=tls is supported"));
    }
    if let Some(transport) = param("type").filter(|t| t != "tcp") {
        return Err(invalid(format!("transport '{transport}' is not supported yet")));
    }

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
        sni: param("sni"),
        fingerprint: param("fp"),
        alpn: param("alpn")
            .map(|a| a.split(',').map(str::to_string).collect())
            .unwrap_or_default(),
        flow: param("flow"),
        pinned_key: param("pcs").and_then(|hex| hex_to_base64(&hex)),
    })
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

    const LINK: &str = "vless://11111111-2222-4333-8444-555555555555@vpn.example.com:443?encryption=none&security=tls&sni=aka.ms&fp=chrome&alpn=h2%2Chttp%2F1.1&pcs=0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF&type=tcp&headerType=none#me-me";

    #[test]
    fn parses_full_link() {
        let p = parse_vless(LINK).unwrap();
        assert_eq!(p.name, "me-me");
        assert_eq!(p.server, "vpn.example.com");
        assert_eq!(p.port, 443);
        assert_eq!(p.sni.as_deref(), Some("aka.ms"));
        assert_eq!(p.fingerprint.as_deref(), Some("chrome"));
        assert_eq!(p.alpn, ["h2", "http/1.1"]);
        assert_eq!(
            p.pinned_key.as_deref(),
            Some("4AJ7t5dYu7LHkpjhv6iBfFJQohf2f2+8QvpDJ3S8nBI=")
        );
    }

    #[test]
    fn rejects_other_schemes() {
        assert!(parse_vless("vmess://abc").is_err());
    }

    #[test]
    fn rejects_missing_uuid() {
        assert!(parse_vless("vless://host:443?security=tls").is_err());
    }

    #[test]
    fn rejects_unsupported_transport() {
        let link = LINK.replace("type=tcp", "type=ws");
        assert!(parse_vless(&link).is_err());
    }
}
