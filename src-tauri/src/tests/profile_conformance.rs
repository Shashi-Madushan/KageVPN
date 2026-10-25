//! Everything every protocol must satisfy, run against each entry in the registry through its
//! own sample link. A new protocol gets all of this by providing `sample()`.

use serde_json::Value;

use super::*;

fn parsed(def: &dyn ProtocolDef) -> Profile {
    def.parse(def.sample()).unwrap_or_else(|e| panic!("{}: its own sample does not parse: {e}", def.id()))
}

#[test]
fn the_sample_parses_validates_and_is_recognised_by_its_scheme() {
    for def in REGISTRY {
        let profile = parsed(*def);
        profile.validate().unwrap_or_else(|e| panic!("{}: its own sample is not valid: {e}", def.id()));
        assert_eq!(profile.protocol, def.id());
        let scheme = def.sample().split("://").next().unwrap();
        assert!(def.schemes().contains(&scheme), "{}: the sample uses an unlisted scheme '{scheme}'", def.id());
        assert!(links::parse_link(def.sample()).is_ok(), "{}: the shared entry point rejects its sample", def.id());
    }
}

#[test]
fn writing_a_link_and_reading_it_back_gives_the_same_server() {
    for def in REGISTRY {
        let first = parsed(*def);
        let again = def.parse(&def.to_link(&first)).unwrap();
        assert!(first.same_endpoint(&again), "{}: the link did not round-trip", def.id());
        assert_eq!(first.name, again.name, "{}: the name did not round-trip", def.id());
        assert_eq!(def.identity(&first), def.identity(&again), "{}: identity changed across a round-trip", def.id());
    }
}

#[test]
fn identity_ignores_the_name_but_not_the_server() {
    for def in REGISTRY {
        let mut renamed = parsed(*def);
        let original = def.identity(&renamed);
        renamed.name = "another name".into();
        assert_eq!(def.identity(&renamed), original, "{}: renaming changed the identity", def.id());
        renamed.server = "other.example.net".into();
        assert_ne!(def.identity(&renamed), original, "{}: the identity ignores the server", def.id());
    }
}

#[test]
fn the_outbound_has_an_address_and_no_tag() {
    for def in REGISTRY {
        let outbound = def.outbound(&parsed(*def));
        assert!(outbound["type"].is_string(), "{}: no outbound type", def.id());
        // An endpoint (WireGuard) names its server in its peer.
        let (server, port) = match def.kind() {
            ServerKind::Outbound => (&outbound["server"], &outbound["server_port"]),
            ServerKind::Endpoint => (&outbound["peers"][0]["address"], &outbound["peers"][0]["port"]),
        };
        assert_eq!(*port, parsed(*def).port, "{}: wrong port", def.id());
        assert!(server.is_string(), "{}: no server", def.id());
        assert!(outbound.get("tag").is_none(), "{}: the tag is added by the caller", def.id());
    }
}

#[test]
fn every_credential_is_hidden_from_feedback_reports() {
    for def in REGISTRY {
        let profile = parsed(*def);
        let secrets = def.secrets(&profile);
        assert!(!secrets.is_empty(), "{}: it reports no secrets", def.id());
        let text = secrets.iter().map(|s| s.value.as_str()).collect::<Vec<_>>().join(" | ");
        let redacted = crate::feedback::redact(&text, &[profile], None);
        for secret in secrets.iter().filter(|s| s.value.trim().len() >= 4) {
            assert!(!redacted.contains(&secret.value), "{}: '{}' survived redaction: {redacted}", def.id(), secret.label);
        }
    }
}

#[test]
fn the_editor_fields_point_at_real_settings() {
    for def in REGISTRY {
        let profile = parsed(*def);
        for field in def.form() {
            let mut value = Some(Value::Object(profile.settings.clone()));
            for key in field.key.split('.') {
                value = match value {
                    // A null parent (such as no Reality block) is fine: the field is hidden then.
                    Some(Value::Null) => break,
                    Some(parent) => parent.get(key).cloned(),
                    None => break,
                };
                assert!(value.is_some(), "{}: field '{}' points at nothing in a saved profile", def.id(), field.key);
            }
        }
    }
}

#[test]
fn a_blank_server_of_this_protocol_needs_its_credentials() {
    // Hysteria v1 servers may accept anyone, so its password is optional.
    const OPTIONAL_CREDENTIALS: &[&str] = &["hysteria"];
    for def in REGISTRY.iter().filter(|d| !OPTIONAL_CREDENTIALS.contains(&d.id())) {
        let mut blank = def.blank();
        assert_eq!(blank.protocol, def.id());
        blank.name = "n".into();
        blank.server = "h.example.com".into();
        assert!(blank.validate().is_err(), "{}: validate accepts a server with no credentials", def.id());
    }
}

#[test]
fn normalizing_twice_changes_nothing_and_keeps_the_sample_valid() {
    for def in REGISTRY {
        let mut once = parsed(*def);
        once.normalize();
        let mut twice = once.clone();
        twice.normalize();
        assert_eq!(once.settings, twice.settings, "{}: normalize is not idempotent", def.id());
        once.validate().unwrap_or_else(|e| panic!("{}: normalize broke the sample: {e}", def.id()));
    }
}

#[test]
fn lists_get_a_description() {
    for def in REGISTRY {
        assert!(!def.summary(&parsed(*def)).is_empty(), "{}: empty summary", def.id());
        assert!(!def.label().is_empty());
    }
}

#[test]
fn only_udp_protocols_need_quic_and_endpoints_are_wireguard() {
    for def in REGISTRY {
        if def.feature() == Some("with_quic") {
            assert!(def.over_udp(), "{}: QUIC is UDP", def.id());
        }
        if def.kind() == ServerKind::Endpoint {
            assert_eq!(def.feature(), Some("with_wireguard"), "{}", def.id());
        }
    }
}
