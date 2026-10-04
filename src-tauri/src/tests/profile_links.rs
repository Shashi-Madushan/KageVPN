use super::*;
use crate::profile::def;

const ID: &str = "11111111-2222-4333-8444-555555555555";

fn sample_links() -> Vec<String> {
    vec![
        format!("vless://{ID}@a.example.com:443?security=tls&type=tcp#vless"),
        "trojan://secret@b.example.com:443?security=tls&type=tcp#trojan".into(),
        "ss://YWVzLTI1Ni1nY206cGE1NXdvcmQ@c.example.com:8388#ss".into(),
        "hysteria2://auth@d.example.com:443/?sni=d.example.com#hy2".into(),
    ]
}

#[test]
fn one_paste_can_mix_every_protocol() {
    let (profiles, errors) = parse_many(&sample_links().join("\n"));
    assert!(errors.is_empty(), "{errors:?}");
    let kinds: Vec<_> = profiles.iter().map(|p| p.protocol.as_str()).collect();
    assert_eq!(kinds, ["vless", "trojan", "shadowsocks", "hysteria2"]);
}

#[test]
fn every_registered_protocol_round_trips_through_the_shared_entry_points() {
    for link in sample_links() {
        let first = parse_link(&link).unwrap();
        first.validate().unwrap();
        let again = parse_link(&to_link(&first)).unwrap();
        assert!(first.same_endpoint(&again), "{link}");
    }
}

#[test]
fn schemes_are_case_insensitive_and_unknown_ones_are_ignored() {
    let (profiles, errors) = parse_many("HY2://pw@h.example.com/#up https://example.com wireguard://x");
    assert_eq!((profiles.len(), errors.len()), (1, 0));
    assert!(contains_link("hello TROJAN://x@y.example.com"));
    assert!(!contains_link("hello world"));
}

#[test]
fn each_registered_scheme_belongs_to_exactly_one_protocol() {
    let mut seen = std::collections::HashSet::new();
    for def in REGISTRY {
        for scheme in def.schemes() {
            assert!(seen.insert(*scheme), "scheme {scheme} is registered twice");
        }
    }
}

#[test]
fn registry_ids_are_unique_and_resolve_back_to_themselves() {
    let mut seen = std::collections::HashSet::new();
    for def in REGISTRY {
        assert!(seen.insert(def.id()), "protocol {} is registered twice", def.id());
        assert_eq!(ProtocolId::lookup(def.id()).map(ProtocolId::as_str), Some(def.id()));
    }
    assert!(ProtocolId::lookup("nope").is_none());
}

#[test]
fn old_saved_profiles_load_as_vless() {
    let old = format!(r#"{{"id":"1","name":"n","server":"s","port":443,"uuid":"{ID}","security":"tls"}}"#);
    let profile: Profile = serde_json::from_str(&old).unwrap();
    assert_eq!(profile.protocol, "vless");
}

#[test]
fn different_protocols_at_one_address_are_not_duplicates() {
    let a = parse_link("trojan://secret@h.example.com:443").unwrap();
    let b = parse_link("hysteria2://secret@h.example.com:443").unwrap();
    assert!(!a.same_endpoint(&b));
}

#[test]
fn an_unknown_protocol_in_saved_data_is_an_error_not_a_silent_vless() {
    let saved = r#"{"id":"1","protocol":"wireguard","name":"n","server":"s","port":1}"#;
    assert!(serde_json::from_str::<Profile>(saved).is_err());
}

#[test]
fn identity_ignores_name_and_group_but_not_the_server_or_credential() {
    let a = parse_link("trojan://secret@h.example.com:443#one").unwrap();
    let b = parse_link("trojan://secret@h.example.com:443#two").unwrap();
    let other = parse_link("trojan://different@h.example.com:443#one").unwrap();
    let id = |p: &Profile| def(p.protocol).identity(p);
    assert_eq!(id(&a), id(&b));
    assert_ne!(id(&a), id(&other));
}

#[test]
fn every_protocol_reports_its_credentials_as_secrets() {
    for link in sample_links() {
        let profile = parse_link(&link).unwrap();
        assert!(!def(profile.protocol).secrets(&profile).is_empty(), "{link}");
        assert!(!def(profile.protocol).summary(&profile).is_empty(), "{link}");
    }
}
