use super::*;

fn cfg(profile: &Profile) -> TuicSettings {
    profile.settings_as().unwrap()
}

const UUID: &str = "11111111-2222-4333-8444-555555555555";
const LINK: &str = "tuic://11111111-2222-4333-8444-555555555555:p%40ss@t.example.com:8443/?sni=front.example.com&alpn=h3&congestion_control=cubic&udp_relay_mode=quic&zero_rtt_handshake=1&allow_insecure=1#TUIC%20Node";

#[test]
fn parses_credentials_choices_and_flags() {
    let p = parse(LINK).unwrap();
    assert_eq!((p.protocol.as_str(), p.port, p.name.as_str()), ("tuic", 8443, "TUIC Node"));
    let s = cfg(&p);
    assert_eq!((s.uuid.as_str(), s.password.as_str()), (UUID, "p@ss"));
    assert_eq!((s.congestion_control.as_str(), s.udp_relay_mode.as_str()), ("cubic", "quic"));
    assert!(s.zero_rtt && s.tls.allow_insecure);
    assert_eq!(s.tls.sni.as_deref(), Some("front.example.com"));
}

#[test]
fn missing_options_get_the_usual_defaults() {
    let s = cfg(&parse(&format!("tuic://{UUID}:pw@t.example.com")).unwrap());
    assert_eq!((s.congestion_control.as_str(), s.udp_relay_mode.as_str()), ("bbr", "native"));
    assert_eq!(s.tls.alpn, ["h3"]);
    assert!(!s.zero_rtt && !s.tls.allow_insecure);
}

#[test]
fn reads_the_other_names_clients_use() {
    let link = format!("tuic://{UUID}:pw@t.example.com:443?congestion_controller=new_reno&reduce_rtt=1&insecure=1&alpn=h3,h3-29");
    let s = cfg(&parse(&link).unwrap());
    assert_eq!(s.congestion_control, "new_reno");
    assert!(s.zero_rtt && s.tls.allow_insecure);
    assert_eq!(s.tls.alpn, ["h3", "h3-29"]);
}

#[test]
fn round_trips_through_its_own_link() {
    let first = parse(LINK).unwrap();
    let again = parse(&to_link(&first)).unwrap();
    assert!(first.same_endpoint(&again));
    assert_eq!(cfg(&first), cfg(&again));
}

#[test]
fn rejects_what_it_cannot_run_and_says_why() {
    let v4 = "TUIC v4 is not supported, ask for a v5 link (UUID and password)";
    for (link, why) in [
        ("tuic://sometoken@t.example.com:443".to_string(), v4),
        (format!("tuic://{UUID}:pw@t.example.com:443?token=abc"), v4),
        (format!("tuic://{UUID}:pw@t.example.com:443?version=4"), v4),
        ("tuic://not-a-uuid:pw@t.example.com:443".to_string(), "link has no valid UUID"),
        (format!("tuic://{UUID}:pw@t.example.com:443?congestion_control=hybla"), "congestion control 'hybla' is not supported"),
        (format!("tuic://{UUID}:pw@t.example.com:443?udp_relay_mode=stream"), "UDP relay mode 'stream' is not supported"),
        (format!("tuic://{UUID}:pw@t.example.com:443?disable_sni=1"), "TUIC without SNI (disable_sni) is not supported"),
        (format!("tuic://{UUID}:pw@t.example.com:443,5000-6000"), "TUIC port hopping is not supported, use a single port"),
    ] {
        assert_eq!(parse(&link).unwrap_err().to_string(), why, "{link}");
    }
}

#[test]
fn validate_checks_what_the_editor_can_break() {
    let mut p = parse(LINK).unwrap();
    let mut s = cfg(&p);
    s.uuid = "nope".into();
    p.put_settings(&s);
    assert!(Tuic.validate(&p).is_err());
    s.uuid = UUID.into();
    s.udp_relay_mode = "stream".into();
    p.put_settings(&s);
    assert!(Tuic.validate(&p).is_err());
}

#[test]
fn normalize_tidies_editor_input() {
    let mut p = parse(LINK).unwrap();
    let mut s = cfg(&p);
    s.uuid = format!(" {} ", UUID.to_uppercase());
    s.congestion_control = "BBR".into();
    s.udp_relay_mode = String::new();
    s.tls.alpn.clear();
    p.put_settings(&s);
    Tuic.normalize(&mut p);
    let s = cfg(&p);
    assert_eq!((s.uuid.as_str(), s.congestion_control.as_str(), s.udp_relay_mode.as_str()), (UUID, "bbr", "native"));
    assert_eq!(s.tls.alpn, ["h3"]);
}

#[test]
fn outbound_carries_the_choices_and_no_tcp_only_tls_options() {
    let out = Tuic.outbound(&parse(LINK).unwrap());
    assert_eq!(out["type"], "tuic");
    assert_eq!((out["uuid"].as_str(), out["password"].as_str()), (Some(UUID), Some("p@ss")));
    assert_eq!((out["congestion_control"].as_str(), out["udp_relay_mode"].as_str()), (Some("cubic"), Some("quic")));
    assert_eq!(out["zero_rtt_handshake"], true);
    assert_eq!(out["tls"]["alpn"], serde_json::json!(["h3"]));
    assert!(out["tls"].get("utls").is_none() && out["tls"].get("reality").is_none());
    let plain = Tuic.outbound(&parse(&format!("tuic://{UUID}:pw@t.example.com")).unwrap());
    assert!(plain.get("zero_rtt_handshake").is_none());
}

#[test]
fn the_list_names_the_congestion_control() {
    assert_eq!(Tuic.summary(&parse(LINK).unwrap()), "TUIC · CUBIC");
}
