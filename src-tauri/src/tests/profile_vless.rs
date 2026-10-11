use super::*;

fn cfg(profile: &Profile) -> VlessSettings {
    profile.settings_as().unwrap()
}
use crate::profile::parse_many;

const LINK: &str = "vless://11111111-2222-4333-8444-555555555555@vpn.example.com:443?encryption=none&security=tls&sni=sni.example.com&fp=chrome&alpn=h2%2Chttp%2F1.1&pcs=0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF&type=tcp&headerType=none#demo";

#[test]
fn parses_full_link() {
    let p = parse_vless(LINK).unwrap();
    assert_eq!(p.name, "demo");
    assert_eq!(p.server, "vpn.example.com");
    assert_eq!(cfg(&p).tls.sni.as_deref(), Some("sni.example.com"));
    assert_eq!(cfg(&p).tls.alpn, ["h2", "http/1.1"]);
}

#[test]
fn reality_link_round_trips() {
    let link = "vless://11111111-2222-4333-8444-555555555555@example.com:443?security=reality&sni=www.microsoft.com&fp=chrome&pbk=abc&sid=12&flow=xtls-rprx-vision&type=grpc&serviceName=svc#R";
    let p = parse_vless(link).unwrap();
    assert_eq!(cfg(&p).security, Security::Reality);
    let again = parse_vless(&to_link(&p)).unwrap();
    assert_eq!(cfg(&again).tls.reality.unwrap().public_key, "abc");
    assert_eq!(cfg(&again).transport.service_name.as_deref(), Some("svc"));
}

const ID: &str = "11111111-2222-4333-8444-555555555555";

#[test]
fn websocket_link_round_trips_path_alpn_and_name() {
    let link = format!("vless://{ID}@h.example.com:8443?security=tls&type=ws&path=%2Fws%3Fed%3D2048&host=cdn.example.com&alpn=h2%2Chttp%2F1.1#WS%20Node");
    let p = parse_vless(&link).unwrap();
    assert_eq!((p.port, p.name.as_str()), (8443, "WS Node"));
    let again = parse_vless(&to_link(&p)).unwrap();
    assert_eq!(cfg(&again).transport.path.as_deref(), Some("/ws?ed=2048"));
    assert_eq!(cfg(&again).transport.host.as_deref(), Some("cdn.example.com"));
    assert_eq!(cfg(&again).tls.alpn, ["h2", "http/1.1"]);
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

#[test]
fn a_non_ascii_certificate_hash_is_ignored_instead_of_crashing_the_app() {
    // `pcs` is hex. Slicing it by byte offsets used to panic in the middle of a multi-byte
    // character, and a panic ends the whole app (and the VPN with it).
    let link = format!("vless://{ID}@h.example.com:443?security=tls&pcs=a%E2%82%AC#odd");
    let p = parse_vless(&link).expect("the link is still importable");
    assert_eq!(cfg(&p).tls.pinned_key, None);
}
