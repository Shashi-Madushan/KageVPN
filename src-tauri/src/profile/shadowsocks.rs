use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use percent_encoding::percent_decode_str;
use serde_json::{json, Value};
use uuid::Uuid;

use super::common::{self, invalid};
use super::{Profile, Protocol, ProtocolSpec, Security};
use crate::error::Result;

pub static SPEC: ProtocolSpec = ProtocolSpec {
    protocol: Protocol::Shadowsocks,
    schemes: &["ss"],
    parse,
    to_link,
    validate,
    outbound,
};

/// Ciphers sing-box can run. The 2022 family is the modern one.
const METHODS: &[&str] = &[
    "2022-blake3-aes-128-gcm",
    "2022-blake3-aes-256-gcm",
    "2022-blake3-chacha20-poly1305",
    "aes-128-gcm",
    "aes-192-gcm",
    "aes-256-gcm",
    "chacha20-ietf-poly1305",
    "xchacha20-ietf-poly1305",
    "aes-128-ctr",
    "aes-192-ctr",
    "aes-256-ctr",
    "aes-128-cfb",
    "aes-192-cfb",
    "aes-256-cfb",
    "chacha20-ietf",
    "xchacha20",
    "rc4-md5",
    "none",
];

/// Accepts both link styles:
/// - SIP002: `ss://BASE64(method:password)@host:port/?plugin=…#name` (or the userinfo unencoded)
/// - legacy: `ss://BASE64(method:password@host:port)#name`
fn parse(link: &str) -> Result<Profile> {
    let body = link.trim().get(5..).filter(|_| link.trim().get(..5).is_some_and(|s| s.eq_ignore_ascii_case("ss://")));
    let body = body.ok_or_else(|| invalid("not an ss:// link"))?;
    let (body, fragment) = body.split_once('#').unwrap_or((body, ""));
    let (main, query) = body.split_once('?').unwrap_or((body, ""));
    let main = main.trim_end_matches('/');

    if url::form_urlencoded::parse(query.as_bytes()).any(|(key, _)| key == "plugin") {
        return Err(invalid("Shadowsocks plugins are not supported"));
    }

    let (userinfo, address) = match main.rsplit_once('@') {
        Some((userinfo, address)) => (decode_userinfo(userinfo)?, address.to_string()),
        None => {
            let decoded = decode_base64(main)?;
            let (userinfo, address) = decoded.rsplit_once('@').ok_or_else(|| invalid("link has no server"))?;
            (userinfo.to_string(), address.to_string())
        }
    };
    let (method, password) = userinfo.split_once(':').ok_or_else(|| invalid("link has no method and password"))?;
    let (server, port) = split_address(&address)?;

    Ok(Profile {
        id: Uuid::new_v4().to_string(),
        protocol: Protocol::Shadowsocks,
        name: common::name_of((!fragment.is_empty()).then_some(fragment), &server),
        server,
        port,
        method: Some(method.to_string()),
        password: password.to_string(),
        security: Security::None,
        fingerprint: None,
        ..Profile::default()
    })
}

fn to_link(profile: &Profile) -> String {
    let method = profile.method.as_deref().unwrap_or_default();
    let credentials = format!("{method}:{}", profile.password);
    // 2022 ciphers use keys that are not safe to base64-wrap in every client, so write them plainly.
    let userinfo = if method.starts_with("2022-") {
        common::encode_userinfo(method) + ":" + &common::encode_userinfo(&profile.password)
    } else {
        URL_SAFE_NO_PAD.encode(credentials)
    };
    common::build_link("ss", &userinfo, profile, "")
}

fn validate(profile: &Profile) -> Result<()> {
    common::check_password(profile)?;
    match profile.method.as_deref() {
        Some(method) if METHODS.contains(&method) => Ok(()),
        Some(method) => Err(invalid(format!("cipher '{method}' is not supported"))),
        None => Err(invalid("choose an encryption method")),
    }
}

fn outbound(profile: &Profile) -> Value {
    json!({
        "type": "shadowsocks",
        "server": profile.server,
        "server_port": profile.port,
        "method": profile.method,
        "password": profile.password,
    })
}

pub fn methods() -> &'static [&'static str] {
    METHODS
}

fn decode_userinfo(userinfo: &str) -> Result<String> {
    let plain = percent_decode_str(userinfo).decode_utf8_lossy().into_owned();
    if plain.contains(':') {
        return Ok(plain);
    }
    decode_base64(&plain)
}

fn decode_base64(text: &str) -> Result<String> {
    let compact: String = text.split_whitespace().collect();
    [&URL_SAFE_NO_PAD, &STANDARD_NO_PAD, &URL_SAFE, &STANDARD]
        .iter()
        .find_map(|engine| engine.decode(compact.trim_end_matches('=')).ok().or_else(|| engine.decode(&compact).ok()))
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .ok_or_else(|| invalid("link is not valid Shadowsocks"))
}

/// `host:port` or `[v6]:port`.
fn split_address(address: &str) -> Result<(String, u16)> {
    let address = address.trim();
    let (host, port) = address.rsplit_once(':').ok_or_else(|| invalid("link has no port"))?;
    let host = host.trim_matches(['[', ']']);
    let port: u16 = port.parse().map_err(|_| invalid("link has an invalid port"))?;
    if host.is_empty() {
        return Err(invalid("link has no server"));
    }
    Ok((host.to_string(), port))
}

#[cfg(test)]
#[path = "../tests/profile_shadowsocks.rs"]
mod tests;
