use super::*;

fn entries(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items.iter().map(|(n, a)| (n.to_string(), a.to_string())).collect()
}

#[test]
fn test_fingerprint_ignores_tunnel_and_loopback_interfaces() {
    let print = fingerprint(entries(&[("lo0", "127.0.0.1"), ("utun4", "172.19.0.1"), ("en0", "192.168.1.5")]));
    assert_eq!(print, vec!["en0=192.168.1.5".to_string()]);
}

#[test]
fn test_fingerprint_is_order_independent_and_deduplicated() {
    let a = fingerprint(entries(&[("en0", "10.0.0.2"), ("en1", "10.1.0.2"), ("en0", "10.0.0.2")]));
    let b = fingerprint(entries(&[("en1", "10.1.0.2"), ("en0", "10.0.0.2")]));
    assert_eq!(a, b);
    assert_eq!(a.len(), 2);
}

#[test]
fn test_slept_needs_a_clear_gap_between_clocks() {
    assert!(!slept(Duration::from_secs(3), Duration::from_secs(3)));
    assert!(!slept(Duration::from_secs(8), Duration::from_secs(3)));
    assert!(slept(Duration::from_secs(600), Duration::from_secs(3)));
}

#[test]
fn test_detect_prefers_wake_over_network_change() {
    let before = vec!["en0=10.0.0.2".to_string()];
    let after = vec!["en0=10.0.0.9".to_string()];
    let tick = Duration::from_secs(3);
    assert_eq!(detect(Duration::from_secs(900), tick, &before, &after), Some(Change::Woke));
    assert_eq!(detect(tick, tick, &before, &after), Some(Change::Network));
    assert_eq!(detect(tick, tick, &before, &before), None);
}
