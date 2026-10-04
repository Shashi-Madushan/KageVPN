use serde_json::Value;

use super::*;
use crate::profile::{blank, def, parse_link, views, Security};

#[test]
fn every_field_key_starts_at_a_real_profile_property() {
    for view in views() {
        let json = serde_json::to_value(blank(view.id).unwrap()).unwrap();
        for field in &view.fields {
            let root = field.key.split('.').next().unwrap();
            assert!(json.get(root).is_some(), "{}: field '{}' has no matching profile property", view.id, field.key);
        }
    }
}

#[test]
fn every_show_when_points_at_a_field_of_the_same_protocol() {
    for view in views() {
        let keys: Vec<_> = view.fields.iter().map(|f| f.key).collect();
        for field in &view.fields {
            if let Some(when) = &field.show_when {
                assert!(keys.contains(&when.key), "{}: '{}' depends on missing '{}'", view.id, field.key, when.key);
            }
        }
    }
}

#[test]
fn selects_and_segments_offer_choices_and_keys_are_unique() {
    for view in views() {
        let mut seen = std::collections::HashSet::new();
        for field in &view.fields {
            assert!(seen.insert(field.key), "{}: duplicate field '{}'", view.id, field.key);
            if matches!(field.kind, Kind::Select | Kind::Segmented) {
                assert!(!field.choices.is_empty(), "{}: '{}' has no choices", view.id, field.key);
            }
        }
    }
}

#[test]
fn blank_profiles_belong_to_their_protocol_and_unknown_ids_fail() {
    for view in views() {
        assert_eq!(blank(view.id).unwrap().protocol, view.id);
    }
    assert!(blank("nope").is_err());
}

#[test]
fn vless_normalize_drops_reality_for_tls_and_fills_the_flow_for_reality() {
    let mut p = parse_link("vless://11111111-2222-4333-8444-555555555555@h.example.com:443?security=reality&pbk=abc&sid=1").unwrap();
    p.flow = None;
    def(p.protocol).normalize(&mut p);
    assert_eq!(p.flow.as_deref(), Some("xtls-rprx-vision"));

    p.security = Security::Tls;
    def(p.protocol).normalize(&mut p);
    assert!(p.reality.is_none());
}

#[test]
fn hysteria2_normalize_drops_an_empty_obfuscation_password() {
    let mut p = parse_link("hysteria2://pw@h.example.com:443/?obfs=salamander&obfs-password=x").unwrap();
    p.obfs.as_mut().unwrap().password.clear();
    def(p.protocol).normalize(&mut p);
    assert!(p.obfs.is_none());
}

#[test]
fn the_serialized_view_adds_a_summary_without_changing_saved_fields() {
    let p = parse_link("trojan://pw@h.example.com:443?type=ws#t").unwrap();
    let json: Value = serde_json::to_value(p.view()).unwrap();
    assert_eq!(json["summary"], "Trojan · ws");
    assert_eq!(json["server"], "h.example.com");
}

/// The screenshot script feeds the interface a copy of the real protocol descriptions. This
/// keeps that copy honest: when a protocol changes, run
/// `UPDATE_FIXTURES=1 cargo test demo_protocols_fixture` and commit the result.
#[test]
fn demo_protocols_fixture_is_current() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts/screenshots/protocols.json");
    let current = serde_json::to_string_pretty(&views()).unwrap() + "\n";
    if std::env::var_os("UPDATE_FIXTURES").is_some() {
        std::fs::write(&path, &current).unwrap();
    }
    let saved = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(saved, current, "scripts/screenshots/protocols.json is out of date: run UPDATE_FIXTURES=1 cargo test demo_protocols_fixture");
}
