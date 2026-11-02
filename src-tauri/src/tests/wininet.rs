use super::*;

fn list(entries: &[&str]) -> Vec<String> {
    entries.iter().map(|s| s.to_string()).collect()
}

#[test]
fn the_built_in_bypass_list_becomes_windows_patterns() {
    let built_in: Vec<String> = crate::sysproxy::BUILT_IN_BYPASS.iter().map(|s| s.to_string()).collect();
    let text = override_list(&built_in);
    let parts: Vec<&str> = text.split(';').collect();
    for expected in ["localhost", "127.0.0.1", "[::1]", "*.local", "169.254.*", "10.*", "172.16.*", "172.31.*", "192.168.*", "<local>"] {
        assert!(parts.contains(&expected), "{expected} missing from {text}");
    }
    assert!(!parts.contains(&"172.32.*") && !parts.contains(&"172.15.*"), "{text}");
    assert_eq!(parts.last(), Some(&"<local>"));
}

#[test]
fn address_ranges_follow_the_prefix() {
    assert_eq!(override_list(&list(&["10.20.0.0/16"])), "10.20.*;<local>");
    assert_eq!(override_list(&list(&["192.168.1.0/24"])), "192.168.1.*;<local>");
    assert_eq!(override_list(&list(&["192.168.1.7/32"])), "192.168.1.7;<local>");
    assert_eq!(override_list(&list(&["10.0.0.0/30"])), "10.0.0.0;10.0.0.1;10.0.0.2;10.0.0.3;<local>");
    // /9 would need 128 patterns, so it is widened to the octet above it.
    assert_eq!(override_list(&list(&["10.0.0.0/9"])), "10.*;<local>");
    assert_eq!(override_list(&list(&["0.0.0.0/0"])), "*;<local>");
}

#[test]
fn ipv6_ranges_are_left_out_and_repeats_are_dropped() {
    assert_eq!(override_list(&list(&["fd00::/8", "*.corp.example", "*.CORP.example"])), "*.corp.example;<local>");
}

#[test]
fn kage_points_windows_at_the_local_proxy_and_knows_its_own_values() {
    let values = kage_values(2081, &list(&["localhost"]));
    assert_eq!(values.proxy_enable, Some(1));
    assert_eq!(values.proxy_server.as_deref(), Some("127.0.0.1:2081"));
    assert_eq!(values.proxy_override.as_deref(), Some("localhost;<local>"));
    assert_eq!(values.auto_config_url, None);
    assert!(still_kage(&values, 2081));
    assert!(!still_kage(&values, 2080));
    let changed = Values { proxy_server: Some("proxy.corp.example:8080".into()), ..values.clone() };
    assert!(!still_kage(&changed, 2081));
    let off = Values { proxy_enable: Some(0), ..values };
    assert!(!still_kage(&off, 2081));
}

#[test]
fn saved_values_survive_a_round_trip_with_missing_entries() {
    let saved = Saved { port: 2080, before: Values { proxy_enable: Some(0), proxy_server: None, proxy_override: None, auto_config_url: Some("http://wpad/pac".into()) } };
    let json = serde_json::to_string(&saved).unwrap();
    assert_eq!(serde_json::from_str::<Saved>(&json).unwrap(), saved);
}
