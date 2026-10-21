use super::*;
use crate::profile::TestKey;

fn cfg(profile: &Profile) -> WireGuardSettings {
    profile.settings_as().unwrap()
}

// Keys made when the tests run; they only need the right shape. These bytes give `+` and `/`.
const PRIVATE: TestKey = TestKey(0xfb);
const PUBLIC: TestKey = TestKey(0xfe);
const PSK: TestKey = TestKey(0x01);

fn enc(key: TestKey) -> String {
    key.url()
}

fn link(extra: &str) -> String {
    format!("wireguard://{}@wg.example.com:51821?publickey={}&address=10.7.0.2%2F32%2Cfd07%3A%3A2{extra}#WG", enc(PRIVATE), enc(PUBLIC))
}

#[test]
fn parses_a_link_with_defaults() {
    let p = parse(&link("")).unwrap();
    assert_eq!((p.protocol.as_str(), p.port, p.name.as_str()), ("wireguard", 51821, "WG"));
    let s = cfg(&p);
    assert_eq!((s.private_key.clone(), s.peer_public_key.clone()), (PRIVATE.to_string(), PUBLIC.to_string()));
    assert_eq!(s.addresses, ["10.7.0.2/32", "fd07::2/128"]);
    assert_eq!((s.mtu, s.keepalive, s.reserved, s.pre_shared_key.as_str()), (DEFAULT_MTU, 0, None, ""));
}

#[test]
fn reads_the_spellings_clients_use_and_unescaped_keys() {
    // Raw `+`, `/` and `=` before the host, a `+` turned into a space in the query.
    let raw = format!(
        "wg://{PRIVATE}@[2001:db8::9]:2408?publicKey={}&ip=172.16.0.2&mtu=1280&reserved=AQID&psk={}&persistentKeepalive=25",
        PUBLIC.to_string().replace('+', " "),
        enc(PSK)
    );
    let p = parse(&raw).unwrap();
    let s = cfg(&p);
    assert_eq!((p.server.as_str(), p.port), ("2001:db8::9", 2408));
    assert_eq!((s.private_key.clone(), s.peer_public_key.clone(), s.pre_shared_key.clone()), (PRIVATE.to_string(), PUBLIC.to_string(), PSK.to_string()));
    assert_eq!((s.addresses.as_slice(), s.mtu, s.keepalive, s.reserved), (["172.16.0.2/32".to_string()].as_slice(), 1280, 25, Some([1, 2, 3])));
}

#[test]
fn the_private_key_may_come_as_a_parameter() {
    let p = parse(&format!("wireguard://wg.example.com:51820?privatekey={}&publickey={}&address=10.0.0.2", enc(PRIVATE), enc(PUBLIC))).unwrap();
    assert_eq!(cfg(&p).private_key, PRIVATE.to_string());
}

#[test]
fn reserved_bytes_are_read_in_every_spelling() {
    use serde_json::json;
    for value in [json!([1, 2, 3]), json!("1,2,3"), json!("[1, 2, 3]"), json!("AQID"), json!(["1", "2", "3"])] {
        assert_eq!(reserved_of(&value), Ok(Some([1, 2, 3])), "{value}");
    }
    assert_eq!(reserved_of(&json!(null)), Ok(None));
    assert_eq!(reserved_of(&json!("")), Ok(None));
    for bad in [json!([1, 2]), json!("1,2,256"), json!("AQIDBA=="), json!("n?pe"), json!(true)] {
        assert_eq!(reserved_of(&bad), Err("reserved must be three numbers from 0 to 255".into()), "{bad}");
    }
}

#[test]
fn round_trips_through_its_own_link() {
    let first = parse(&link(&format!("&presharedkey={}&keepalive=15&reserved=9%2C8%2C7&mtu=1420", enc(PSK)))).unwrap();
    let again = parse(&to_link(&first)).unwrap();
    assert!(first.same_endpoint(&again));
    assert_eq!(cfg(&first), cfg(&again));
}

#[test]
fn rejects_what_it_cannot_run_and_says_why() {
    let no_address = format!("wireguard://{}@wg.example.com:51820?publickey={}", enc(PRIVATE), enc(PUBLIC));
    for (link, why) in [
        (no_address, "WireGuard needs this device's address (Address)".to_string()),
        (link("").replace(&enc(PRIVATE), "short"), "private key is not a WireGuard key".into()),
        (link("").replace(&enc(PUBLIC), "short"), "public key is not a WireGuard key".into()),
        (link("&presharedkey=short"), "pre-shared key is not a WireGuard key".into()),
        (link("&mtu=9000"), "MTU must be between 1280 and 1500".into()),
        (link("&reserved=1,2"), "reserved must be three numbers from 0 to 255".into()),
        (link("").replace("10.7.0.2%2F32", "10.7.0.999"), "'10.7.0.999/32' is not an address such as 172.16.0.2/32".into()),
    ] {
        assert_eq!(parse(&link).unwrap_err().to_string(), why, "{link}");
    }
}

#[test]
fn the_endpoint_entry_is_what_sing_box_expects() {
    let p = parse(&link(&format!("&presharedkey={}&keepalive=15&reserved=1,2,3", enc(PSK)))).unwrap();
    let entry = WireGuard.outbound(&p);
    assert_eq!((entry["type"].as_str(), entry["system"].as_bool(), entry["mtu"].as_u64()), (Some("wireguard"), Some(false), Some(1408)));
    assert_eq!(entry["address"], serde_json::json!(["10.7.0.2/32", "fd07::2/128"]));
    let peer = &entry["peers"][0];
    assert_eq!((peer["address"].as_str(), peer["port"].as_u64()), (Some("wg.example.com"), Some(51821)));
    assert_eq!((peer["public_key"].as_str(), peer["pre_shared_key"].as_str()), (Some(PUBLIC.to_string().as_str()), Some(PSK.to_string().as_str())));
    assert_eq!(peer["allowed_ips"], serde_json::json!(["0.0.0.0/0", "::/0"]));
    assert_eq!((peer["persistent_keepalive_interval"].as_u64(), &peer["reserved"]), (Some(15), &serde_json::json!([1, 2, 3])));

    let plain = WireGuard.outbound(&parse(&link("")).unwrap());
    let peer = &plain["peers"][0];
    assert!(peer.get("pre_shared_key").is_none() && peer.get("reserved").is_none() && peer.get("persistent_keepalive_interval").is_none());
}

#[test]
fn identity_ignores_the_local_address() {
    let a = parse(&link("")).unwrap();
    let b = parse(&link("").replace("10.7.0.2", "10.7.0.99")).unwrap();
    assert_eq!(WireGuard.identity(&a), WireGuard.identity(&b));
}

#[test]
fn the_editor_sends_text_and_nulls() {
    let mut p = parse(&link("")).unwrap();
    for (key, value) in [("mtu", serde_json::json!("1380")), ("keepalive", serde_json::json!(null)), ("reserved", serde_json::json!("4,5,6")), ("preSharedKey", serde_json::json!(null))] {
        p.settings.insert(key.into(), value);
    }
    p.settings.insert("addresses".into(), serde_json::json!([" 10.9.0.2 ", ""]));
    WireGuard.normalize(&mut p);
    let s = cfg(&p);
    assert_eq!((s.mtu, s.keepalive, s.reserved, s.addresses.as_slice()), (1380, 0, Some([4, 5, 6]), ["10.9.0.2/32".to_string()].as_slice()));
    assert!(p.validate().is_ok());
    p.settings.insert("mtu".into(), serde_json::json!("big"));
    assert_eq!(WireGuard.validate(&p).unwrap_err().to_string(), "MTU must be between 1280 and 1500");
}

#[test]
fn warp_shows_in_the_list() {
    assert_eq!(WireGuard.summary(&parse(&link("&reserved=1,2,3")).unwrap()), "WireGuard · WARP");
    assert_eq!(WireGuard.summary(&parse(&link("")).unwrap()), "WireGuard");
}

// ── wg-quick files ──

fn conf(interface_extra: &str, peer_extra: &str) -> String {
    format!(
        "# Name = Tokyo exit\n[Interface]\nPrivateKey = {PRIVATE}\nAddress = 10.66.0.2/32, fd66::2/128\n{interface_extra}\n[Peer]\nPublicKey = {PUBLIC}\nAllowedIPs = 0.0.0.0/0, ::/0\nEndpoint = vpn.example.com:51820\n{peer_extra}\n"
    )
}

#[test]
fn a_plain_conf_imports_without_a_note_and_takes_its_name_comment() {
    let (p, note) = parse_wg_quick(&conf("MTU = 1380", "PersistentKeepalive = 25"), None).unwrap();
    assert_eq!((p.name.as_str(), p.server.as_str(), p.port), ("Tokyo exit", "vpn.example.com", 51820));
    let s = cfg(&p);
    assert_eq!((s.addresses.len(), s.mtu, s.keepalive), (2, 1380, 25));
    assert_eq!(note, None);
}

#[test]
fn the_file_name_wins_and_the_host_is_the_last_resort() {
    assert_eq!(parse_wg_quick(&conf("", ""), Some("warp-tokyo")).unwrap().0.name, "warp-tokyo");
    let unnamed = conf("", "").replace("# Name = Tokyo exit\n", "");
    assert_eq!(parse_wg_quick(&unnamed, None).unwrap().0.name, "vpn.example.com");
}

#[test]
fn lines_it_does_not_use_are_named_in_a_note() {
    let (_, note) = parse_wg_quick(&conf("DNS = 1.1.1.1\nTable = off\nPostUp = iptables -A x", &format!("PresharedKey = {PSK}")).replace("AllowedIPs = 0.0.0.0/0, ::/0", "AllowedIPs = 10.0.0.0/8"), None).unwrap();
    assert_eq!(note.unwrap(), "DNS, AllowedIPs, Table and PostUp from the file are not used; Kage's DNS and routing settings decide");
    let (_, note) = parse_wg_quick(&conf("DNS = 1.1.1.1", ""), None).unwrap();
    assert_eq!(note.unwrap(), "DNS from the file is not used; Kage's DNS and routing settings decide");
}

#[test]
fn a_conf_it_cannot_run_is_refused_with_the_reason() {
    let two_peers = format!("{}[Peer]\nPublicKey = {PSK}\nEndpoint = b.example.com:51820\n", conf("", ""));
    for (text, why) in [
        (two_peers, "WireGuard configs with more than one peer are not supported"),
        (conf("Jc = 4\nJmin = 40", ""), "AmneziaWG options (Jc, ...) are not supported; sing-box runs plain WireGuard"),
        (conf("", "").replace("Endpoint = vpn.example.com:51820\n", ""), "the file has no Endpoint for its peer"),
        (conf("", "").replace("Address = 10.66.0.2/32, fd66::2/128\n", ""), "WireGuard needs this device's address (Address)"),
        (format!("[Interface]\nPrivateKey = {PRIVATE}\nAddress = 10.0.0.2/32\n"), "the file has no [Peer] section"),
        (conf("", "").replace("vpn.example.com:51820", "vpn.example.com"), "Endpoint 'vpn.example.com' is not host:port"),
    ] {
        assert_eq!(parse_wg_quick(&text, None).unwrap_err().to_string(), why, "{text}");
    }
}

#[test]
fn ipv6_endpoints_crlf_lines_and_repeated_address_lines_are_read() {
    let text = format!("[Interface]\r\nPrivateKey = {PRIVATE}\r\nAddress = 10.0.0.2/32\r\nAddress = fd00::2/128\r\n\r\n[Peer]\r\nPublicKey = {PUBLIC}\r\nEndpoint = [2001:db8::1]:443\r\n");
    assert!(is_wg_quick(&text));
    let (p, _) = parse_wg_quick(&text, None).unwrap();
    assert_eq!((p.server.as_str(), p.port), ("2001:db8::1", 443));
    assert_eq!(cfg(&p).addresses, ["10.0.0.2/32", "fd00::2/128"]);
}

#[test]
fn only_interface_first_text_with_a_peer_is_a_conf() {
    assert!(is_wg_quick(&conf("", "")));
    assert!(!is_wg_quick("[Peer]\nEndpoint = h:1\n[Interface]\n"));
    assert!(!is_wg_quick(&format!("[Interface]\nPrivateKey = {PRIVATE}\n")));
    assert!(!is_wg_quick("vless://x@h:443"));
}
