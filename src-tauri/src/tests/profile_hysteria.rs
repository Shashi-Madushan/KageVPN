use super::*;

fn cfg(profile: &Profile) -> HysteriaSettings {
    profile.settings_as().unwrap()
}

const LINK: &str = "hysteria://h.example.com:8443?protocol=udp&auth=secret%20pass&peer=front.example.com&insecure=1&upmbps=20&downmbps=100&alpn=hysteria&obfs=xplus&obfsParam=mask#HY%20Node";

#[test]
fn parses_the_official_link_form() {
    let p = parse(LINK).unwrap();
    assert_eq!((p.protocol.as_str(), p.port, p.name.as_str()), ("hysteria", 8443, "HY Node"));
    let s = cfg(&p);
    assert_eq!((s.auth.as_str(), s.up_mbps, s.down_mbps), ("secret pass", 20, 100));
    assert_eq!(s.obfs.as_deref(), Some("mask"));
    assert_eq!(s.tls.sni.as_deref(), Some("front.example.com"));
    assert!(s.tls.allow_insecure);
    assert_eq!(s.tls.alpn, ["hysteria"]);
}

#[test]
fn reads_auth_before_the_host_and_the_other_names_clients_use() {
    let p = parse("hysteria://pw@h.example.com:443?sni=s.example.com&up=30%20Mbps&down=1%20Gbps&obfs=xplus&obfs-password=m&allowInsecure=1").unwrap();
    let s = cfg(&p);
    assert_eq!((s.auth.as_str(), s.up_mbps, s.down_mbps), ("pw", 30, 1000));
    assert_eq!((s.obfs.as_deref(), s.tls.sni.as_deref()), (Some("m"), Some("s.example.com")));
    assert!(s.tls.allow_insecure);
}

#[test]
fn speeds_are_read_in_the_usual_spellings() {
    assert_eq!(parse_speed("100"), Some(100));
    assert_eq!(parse_speed(" 100 Mbps "), Some(100));
    assert_eq!(parse_speed("50m"), Some(50));
    assert_eq!(parse_speed("1.5 Gbps"), Some(1500));
    assert_eq!(parse_speed("2g"), Some(2000));
    for bad in ["", "0", "fast", "-5", "100 kbps", "1000000"] {
        assert_eq!(parse_speed(bad), None, "{bad}");
    }
}

#[test]
fn missing_speeds_get_defaults_and_a_note() {
    let link = "hysteria://h.example.com:443?auth=pw";
    let s = cfg(&parse(link).unwrap());
    assert_eq!((s.up_mbps, s.down_mbps), (DEFAULT_UP_MBPS, DEFAULT_DOWN_MBPS));
    assert_eq!(
        Hysteria.import_note(link).unwrap(),
        "no speeds in the link, using 10 Mbps up and 50 Mbps down; change it in the server editor if that is wrong"
    );
    assert!(Hysteria.import_note("hysteria://h.example.com:443?upmbps=5").unwrap().starts_with("no download speed in the link, using 50 Mbps"));
    assert!(Hysteria.import_note("hysteria://h.example.com:443?down=5").unwrap().starts_with("no upload speed in the link, using 10 Mbps"));
    assert_eq!(Hysteria.import_note(LINK), None);
}

#[test]
fn a_server_without_a_password_is_allowed() {
    let p = parse("hysteria://open.example.com:443?upmbps=10&downmbps=10").unwrap();
    assert!(p.validate().is_ok());
    assert!(Hysteria.outbound(&p).get("auth_str").is_none());
}

#[test]
fn an_empty_password_from_the_editor_saves() {
    // The editor sends null for an optional field left empty.
    let mut p = parse(LINK).unwrap();
    p.settings.insert("auth".into(), serde_json::Value::Null);
    assert_eq!(cfg(&p).auth, "");
    assert!(Hysteria.validate(&p).is_ok());
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
    let hopping = "Hysteria port hopping is not supported, use a single port";
    for (link, why) in [
        ("hysteria://h.example.com:443?protocol=faketcp", "Hysteria over faketcp is not supported, only udp"),
        ("hysteria://h.example.com:443?protocol=wechat-video", "Hysteria over wechat-video is not supported, only udp"),
        ("hysteria://h.example.com:443?obfs=xplus", "obfuscation has no password (obfsParam)"),
        ("hysteria://h.example.com:443?obfs=salamander&obfsParam=x", "obfuscation 'salamander' is not supported"),
        ("hysteria://h.example.com:443?mport=1000-2000", hopping),
        ("hysteria://h.example.com:443,5000-6000", hopping),
        ("hysteria://h.example.com:443?upmbps=fast", "upload speed 'fast' is not a number of Mbps"),
        ("hysteria://h.example.com:443?downmbps=0", "download speed '0' is not a number of Mbps"),
    ] {
        assert_eq!(parse(link).unwrap_err().to_string(), why, "{link}");
    }
}

#[test]
fn the_editor_can_send_speeds_as_text() {
    let mut p = parse(LINK).unwrap();
    p.settings.insert("upMbps".into(), serde_json::json!("25"));
    p.settings.insert("downMbps".into(), serde_json::json!("1 Gbps"));
    assert_eq!((cfg(&p).up_mbps, cfg(&p).down_mbps), (25, 1000));
    p.settings.insert("upMbps".into(), serde_json::json!("lots"));
    assert_eq!(Hysteria.validate(&p).unwrap_err().to_string(), "upload speed must be between 1 and 100000 Mbps");
}

#[test]
fn outbound_carries_speeds_auth_and_obfuscation() {
    let out = Hysteria.outbound(&parse(LINK).unwrap());
    assert_eq!(out["type"], "hysteria");
    assert_eq!((out["up_mbps"].as_u64(), out["down_mbps"].as_u64()), (Some(20), Some(100)));
    assert_eq!((out["auth_str"].as_str(), out["obfs"].as_str()), (Some("secret pass"), Some("mask")));
    assert_eq!(out["tls"]["alpn"], serde_json::json!(["hysteria"]));
    assert!(out["tls"].get("utls").is_none() && out["tls"].get("reality").is_none());
}

#[test]
fn the_list_shows_the_speeds() {
    assert_eq!(Hysteria.summary(&parse(LINK).unwrap()), "Hysteria · 20/100 Mbps");
}
