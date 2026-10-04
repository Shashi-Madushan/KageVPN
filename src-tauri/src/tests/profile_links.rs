use super::*;
use crate::profile::spec;

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
    let kinds: Vec<_> = profiles.iter().map(|p| p.protocol).collect();
    assert_eq!(kinds, [Protocol::Vless, Protocol::Trojan, Protocol::Shadowsocks, Protocol::Hysteria2]);
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
    for spec in REGISTRY {
        for scheme in spec.schemes {
            assert!(seen.insert(*scheme), "scheme {scheme} is registered twice");
        }
    }
}

#[test]
fn every_protocol_is_in_the_registry() {
    for protocol in [Protocol::Vless, Protocol::Trojan, Protocol::Shadowsocks, Protocol::Hysteria2] {
        assert_eq!(spec(protocol).protocol, protocol);
    }
}

#[test]
fn old_saved_profiles_load_as_vless() {
    let old = format!(r#"{{"id":"1","name":"n","server":"s","port":443,"uuid":"{ID}","security":"tls"}}"#);
    let profile: Profile = serde_json::from_str(&old).unwrap();
    assert_eq!(profile.protocol, Protocol::Vless);
}

#[test]
fn different_protocols_at_one_address_are_not_duplicates() {
    let a = parse_link("trojan://secret@h.example.com:443").unwrap();
    let b = parse_link("hysteria2://secret@h.example.com:443").unwrap();
    assert!(!a.same_endpoint(&b));
}
