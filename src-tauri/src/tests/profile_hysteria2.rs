use super::*;

fn cfg(profile: &Profile) -> Hysteria2Settings {
    profile.settings_as().unwrap()
}

const LINK: &str = "hysteria2://auth%20pass@h.example.com:8443/?sni=front.example.com&obfs=salamander&obfs-password=mask&insecure=1#HY2%20Node";

#[test]
fn parses_auth_obfuscation_and_flags() {
    let p = parse(LINK).unwrap();
    assert_eq!((p.protocol.as_str(), p.port, p.name.as_str()), ("hysteria2", 8443, "HY2 Node"));
    assert_eq!(cfg(&p).password, "auth pass");
    assert_eq!(cfg(&p).obfs, Some(Obfs { kind: "salamander".into(), password: "mask".into() }));
    assert!(cfg(&p).tls.allow_insecure);
}

#[test]
fn accepts_the_hy2_scheme_and_user_pass_auth() {
    let p = parse("hy2://user:pw@h.example.com/#short").unwrap();
    assert_eq!((cfg(&p).password.as_str(), p.port), ("user:pw", 443));
}

#[test]
fn round_trips_through_its_own_link() {
    let first = parse(LINK).unwrap();
    assert!(first.same_endpoint(&parse(&to_link(&first)).unwrap()));
}

#[test]
fn rejects_what_it_cannot_run() {
    for link in [
        "hysteria2://h.example.com:443",
        "hysteria2://pw@h.example.com:443,5000-6000/",
        "hysteria2://pw@h.example.com:443/?pinSHA256=AB:CD",
        "hysteria2://pw@h.example.com:443/?obfs=salamander",
        "hysteria2://pw@h.example.com:443/?obfs=quux&obfs-password=x",
    ] {
        assert!(parse(link).is_err(), "{link}");
    }
}

#[test]
fn port_lists_are_told_apart_from_dashes_in_host_names() {
    assert!(has_port_list("hysteria2://pw@h.example.com:5000-6000/"));
    assert!(!has_port_list("hysteria2://pw@my-host.example.com:443/?x=1-2"));
    assert!(!has_port_list("hysteria2://pw@[2001:db8::1]:443/"));
}

#[test]
fn outbound_leaves_out_tcp_only_tls_options() {
    let out = Hysteria2.outbound(&parse(LINK).unwrap());
    assert_eq!(out["type"], "hysteria2");
    assert_eq!(out["obfs"]["type"], "salamander");
    assert_eq!(out["tls"]["insecure"], true);
    assert!(out["tls"].get("utls").is_none() && out["tls"].get("reality").is_none());
}
