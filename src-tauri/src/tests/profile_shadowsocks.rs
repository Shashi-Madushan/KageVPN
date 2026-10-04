use super::*;

// "aes-256-gcm:pa55word" in base64url
const SIP002: &str = "ss://YWVzLTI1Ni1nY206cGE1NXdvcmQ@s.example.com:8388/?#SS%20Node";

#[test]
fn parses_sip002_links() {
    let p = parse(SIP002).unwrap();
    assert_eq!((p.method.as_deref(), p.password.as_str()), (Some("aes-256-gcm"), "pa55word"));
    assert_eq!((p.server.as_str(), p.port, p.name.as_str()), ("s.example.com", 8388, "SS Node"));
    assert_eq!(p.security, Security::None);
}

#[test]
fn parses_the_legacy_whole_link_encoding() {
    // "chacha20-ietf-poly1305:secret@legacy.example.com:443"
    let p = parse("ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpzZWNyZXRAbGVnYWN5LmV4YW1wbGUuY29tOjQ0Mw#Legacy").unwrap();
    assert_eq!((p.server.as_str(), p.port, p.name.as_str()), ("legacy.example.com", 443, "Legacy"));
    assert_eq!(p.method.as_deref(), Some("chacha20-ietf-poly1305"));
}

#[test]
fn parses_plain_2022_userinfo_and_ipv6() {
    let p = parse("ss://2022-blake3-aes-128-gcm:YWJj%2FZGVm%3D@[2001:db8::5]:9000#v6").unwrap();
    assert_eq!((p.method.as_deref(), p.password.as_str()), (Some("2022-blake3-aes-128-gcm"), "YWJj/ZGVm="));
    assert_eq!((p.server.as_str(), p.port), ("2001:db8::5", 9000));
}

#[test]
fn round_trips_both_link_styles() {
    for link in [SIP002, "ss://2022-blake3-aes-128-gcm:YWJj%2FZGVm%3D@s.example.com:9000#x"] {
        let first = parse(link).unwrap();
        let again = parse(&to_link(&first)).unwrap();
        assert!(first.same_endpoint(&again), "{link}");
    }
}

#[test]
fn rejects_plugins_and_broken_links() {
    for link in [
        "ss://YWVzLTI1Ni1nY206cGE1NXdvcmQ@s.example.com:8388/?plugin=v2ray-plugin#x",
        "ss://not base64 at all",
        "ss://YWVzLTI1Ni1nY206cGE1NXdvcmQ@s.example.com#noport",
    ] {
        assert!(parse(link).is_err(), "{link}");
    }
}

#[test]
fn validate_checks_the_cipher() {
    let mut p = parse(SIP002).unwrap();
    assert!(validate(&p).is_ok());
    p.method = Some("rot13".into());
    assert!(validate(&p).is_err());
}

#[test]
fn outbound_names_the_cipher() {
    let out = outbound(&parse(SIP002).unwrap());
    assert_eq!((out["type"].as_str(), out["method"].as_str()), (Some("shadowsocks"), Some("aes-256-gcm")));
    assert!(out.get("tls").is_none());
}
