use super::*;

fn settings(enabled: bool, action: NetworkAction) -> NetworkSettings {
    NetworkSettings { enabled, networks: vec![KnownNetwork { id: "aa:bb:cc:dd:ee:ff".into(), name: "Home".into(), action, last_seen: 0 }] }
}
const HOME: &str = "aa:bb:cc:dd:ee:ff";

#[test]
fn nothing_happens_while_the_feature_is_off_or_there_is_no_network() {
    assert_eq!(decide(&settings(false, NetworkAction::Connect), HOME, Conn::Disconnected, true, false), Decision::Nothing);
    assert_eq!(decide(&settings(true, NetworkAction::Connect), "", Conn::Disconnected, true, false), Decision::Nothing);
}

#[test]
fn an_unknown_network_is_asked_about_once_known() {
    assert_eq!(decide(&settings(true, NetworkAction::Connect), "11:22:33:44:55:66", Conn::Disconnected, true, false), Decision::Ask);
    assert_eq!(decide(&NetworkSettings { enabled: true, networks: vec![] }, HOME, Conn::Connected, true, false), Decision::Ask);
}

#[test]
fn connect_only_when_disconnected_with_servers_and_no_reconnect_running() {
    let s = settings(true, NetworkAction::Connect);
    assert_eq!(decide(&s, HOME, Conn::Disconnected, true, false), Decision::Connect);
    assert_eq!(decide(&s, HOME, Conn::Connected, true, false), Decision::Nothing);
    assert_eq!(decide(&s, HOME, Conn::Connecting, true, false), Decision::Nothing);
    assert_eq!(decide(&s, HOME, Conn::Disconnected, false, false), Decision::Nothing, "no servers");
    assert_eq!(decide(&s, HOME, Conn::Disconnected, true, true), Decision::Nothing, "reconnect loop active");
}

#[test]
fn disconnect_when_connected_or_connecting_and_ignore_never_acts() {
    let s = settings(true, NetworkAction::Disconnect);
    assert_eq!(decide(&s, HOME, Conn::Connected, true, false), Decision::Disconnect);
    assert_eq!(decide(&s, HOME, Conn::Connecting, true, false), Decision::Disconnect);
    assert_eq!(decide(&s, HOME, Conn::Disconnected, true, false), Decision::Nothing);
    for state in [Conn::Disconnected, Conn::Connecting, Conn::Connected] {
        assert_eq!(decide(&settings(true, NetworkAction::Ignore), HOME, state, true, false), Decision::Nothing);
    }
}

#[test]
fn the_gateway_is_the_default_route_of_a_physical_interface() {
    let table = "Routing tables\n\nInternet:\nDestination        Gateway            Flags               Netif Expire\ndefault            172.19.0.2         UGScg               utun4\ndefault            192.168.8.1        UGScIg                en0\n1                  172.19.0.1         UGSc                utun4\n";
    assert_eq!(parse_gateway(table), Some(("192.168.8.1".into(), "en0".into())));
    assert_eq!(parse_gateway("default            link#17            UCSIg               en0\n"), None, "no address to look up");
    assert_eq!(parse_gateway(""), None);
}

#[test]
fn the_mac_is_normalised_and_an_incomplete_entry_is_rejected() {
    assert_eq!(parse_mac("? (192.168.8.1) at d8:d8:6:46:DE:56 on en0 ifscope [ethernet]"), Some("d8:d8:06:46:de:56".into()));
    assert_eq!(parse_mac("? (192.168.8.1) at (incomplete) on en0 ifscope [ethernet]"), None);
    assert_eq!(parse_mac("no entry"), None);
}

#[test]
fn names_are_defaulted_and_validated() {
    assert_eq!(default_name("d8:d8:06:46:de:56"), "Network de56");
    assert_eq!(clean_name("  Home  "), Some("Home".into()));
    assert_eq!(clean_name("   "), None);
    assert_eq!(clean_name(&"x".repeat(41)), None);
}

#[test]
fn an_old_data_file_without_networks_loads_with_the_feature_off() {
    let settings: NetworkSettings = serde_json::from_str("{}").unwrap();
    assert!(!settings.enabled && settings.networks.is_empty());
    let known: KnownNetwork = serde_json::from_str(r#"{"id":"aa"}"#).unwrap();
    assert_eq!(known.action, NetworkAction::Ignore);
}
