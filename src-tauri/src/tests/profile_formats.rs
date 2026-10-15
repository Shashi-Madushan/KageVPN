use serde_json::json;

use super::*;
use crate::profile::{parse_many, Profile, REGISTRY};

/// The sing-box outbound a profile produces, without the tag, for comparing servers.
fn outbound(profile: &Profile) -> Value {
    let mut outbound = profile.outbound("x");
    outbound.as_object_mut().unwrap().remove("tag");
    outbound
}

fn only(text: &str) -> Profile {
    let (mut profiles, errors) = parse_many(text);
    assert!(errors.is_empty(), "unexpected skips: {errors:?}");
    assert_eq!(profiles.len(), 1);
    profiles.remove(0)
}

#[test]
fn every_protocol_round_trips_through_a_singbox_config() {
    // A server exported as a sing-box outbound and imported again must connect the same way.
    // Covers each registered protocol's sample, so a new protocol is checked without editing this.
    for def in REGISTRY {
        let original = def.parse(def.sample()).unwrap();
        let mut exported = original.outbound(&original.name);
        exported["tag"] = json!(original.name);
        let config = json!({ "outbounds": [exported, { "type": "direct", "tag": "direct" }] }).to_string();

        let imported = only(&config);
        assert_eq!(imported.protocol, original.protocol, "{}", def.id());
        assert_eq!(imported.name, original.name, "{}", def.id());
        assert_eq!(outbound(&imported), outbound(&original), "{} changed on the way through", def.id());
    }
}

#[test]
fn singbox_configs_skip_groups_quietly_and_explain_the_rest() {
    let config = json!({
        "log": { "level": "info" },
        "outbounds": [
            { "type": "selector", "tag": "proxy", "outbounds": ["a", "b"] },
            { "type": "urltest", "tag": "auto", "outbounds": ["a"] },
            {
                "type": "vless", "tag": "🇩🇪 Frankfurt", "server": "203.0.113.10", "server_port": 443,
                "uuid": "11111111-2222-4333-8444-555555555555", "flow": "xtls-rprx-vision",
                "tls": {
                    "enabled": true, "server_name": "www.example.com",
                    "utls": { "enabled": true, "fingerprint": "chrome" },
                    "reality": { "enabled": true, "public_key": "PUBLICKEYPUBLICKEYPUBLICKEYPUBLICKEYPUBLIC", "short_id": "abcd" }
                }
            },
            { "type": "tuic", "tag": "Tokyo TUIC", "server": "tuic.example.com", "server_port": 443 },
            { "type": "trojan", "tag": "Chained", "server": "chain.example.com", "server_port": 443, "password": "p", "detour": "proxy", "tls": { "enabled": true } },
            { "type": "vless", "tag": "No port", "server": "noport.example.com", "uuid": "11111111-2222-4333-8444-555555555555" },
            { "type": "direct", "tag": "direct" },
            { "type": "block", "tag": "block" }
        ]
    });
    let (profiles, errors) = parse_many(&config.to_string());
    assert_eq!(profiles.len(), 1);
    let vless = &profiles[0];
    assert_eq!((vless.name.as_str(), vless.server.as_str(), vless.port), ("🇩🇪 Frankfurt", "203.0.113.10", 443));
    let out = outbound(vless);
    assert_eq!(out["flow"], "xtls-rprx-vision");
    assert_eq!(out["tls"]["reality"]["short_id"], "abcd");
    assert_eq!(out["tls"]["server_name"], "www.example.com");
    assert_eq!(
        errors,
        [
            "Tokyo TUIC: tuic servers are not supported yet",
            "Chained: servers that connect through another server are not supported",
            "No port: no valid port",
        ]
    );
}

#[test]
fn xray_configs_are_named_rather_than_ignored() {
    let config = r#"{ "outbounds": [ { "protocol": "vless", "settings": {} }, { "protocol": "freedom" } ] }"#;
    let (profiles, errors) = parse_many(config);
    assert!(profiles.is_empty());
    assert_eq!(errors[0], "Xray configuration files are not supported yet");
}

const CLASH: &str = r#"
mixed-port: 7890
allow-lan: false
x-common: &tls-common
  tls: true
  skip-cert-verify: false
  client-fingerprint: chrome
proxies:
  - name: "🇺🇸 VLESS Reality"
    type: vless
    server: 198.51.100.7
    port: 443
    uuid: 11111111-2222-4333-8444-555555555555
    network: tcp
    flow: xtls-rprx-vision
    servername: www.example.com
    <<: *tls-common
    reality-opts:
      public-key: PUBLICKEYPUBLICKEYPUBLICKEYPUBLICKEYPUBLIC
      short-id: 0123abcd
  - { name: VMess WS, type: vmess, server: vmess.example.com, port: "8443", uuid: 11111111-2222-4333-8444-555555555555, alterId: 0, cipher: auto, tls: true, servername: cdn.example.com, network: ws, ws-opts: { path: /ws, headers: { Host: cdn.example.com } } }
  - name: Trojan gRPC
    type: trojan
    server: trojan.example.com
    port: 443
    password: "p@ss:word"
    sni: trojan.example.com
    alpn: [h2]
    network: grpc
    grpc-opts:
      grpc-service-name: tunnel
  - name: Upgrade
    type: vless
    server: up.example.com
    port: 443
    uuid: 11111111-2222-4333-8444-555555555555
    tls: true
    network: ws
    ws-opts: { path: /up, v2ray-http-upgrade: true }
  - { name: SS 2022, type: ss, server: "2001:db8::1", port: 8388, cipher: 2022-blake3-aes-128-gcm, password: "c2VjcmV0c2VjcmV0c2VjcmV0" }
  - { name: SS obfs, type: ss, server: ss.example.com, port: 8388, cipher: aes-128-gcm, password: x, plugin: obfs }
  - { name: Hy2, type: hysteria2, server: hy.example.com, port: 443, password: secret, sni: hy.example.com, obfs: salamander, obfs-password: salt, skip-cert-verify: true }
  - { name: Hopping, type: hysteria2, server: hop.example.com, port: 443, ports: "20000-30000", password: secret }
  - { name: Plain VLESS, type: vless, server: plain.example.com, port: 80, uuid: 11111111-2222-4333-8444-555555555555 }
  - { name: WireGuard, type: wireguard, server: wg.example.com, port: 51820 }
  - { name: Direct, type: direct }
proxy-groups:
  - { name: Proxy, type: select, proxies: ["🇺🇸 VLESS Reality", VMess WS] }
rules:
  - MATCH,Proxy
"#;

#[test]
fn clash_configs_import_every_supported_server() {
    let (profiles, errors) = parse_many(CLASH);
    let names: Vec<&str> = profiles.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["🇺🇸 VLESS Reality", "VMess WS", "Trojan gRPC", "Upgrade", "SS 2022", "Hy2"]);
    assert_eq!(
        errors,
        [
            "SS obfs: Shadowsocks plugins are not supported",
            "Hopping: Hysteria2 port hopping is not supported, use a single port",
            "Plain VLESS: only security=tls or reality is supported",
            "WireGuard: wireguard servers are not supported yet",
        ]
    );

    let by_name = |name: &str| outbound(profiles.iter().find(|p| p.name == name).unwrap());

    // The merge key brought in TLS and the fingerprint; Reality comes from its own block.
    let reality = by_name("🇺🇸 VLESS Reality");
    assert_eq!(reality["tls"]["utls"]["fingerprint"], "chrome");
    assert_eq!(reality["tls"]["reality"]["public_key"], "PUBLICKEYPUBLICKEYPUBLICKEYPUBLICKEYPUBLIC");
    assert_eq!(reality["tls"]["server_name"], "www.example.com");

    let vmess = by_name("VMess WS");
    assert_eq!((vmess["server_port"].as_u64(), vmess["transport"]["path"].as_str()), (Some(8443), Some("/ws")));
    assert_eq!(vmess["transport"]["headers"]["Host"], "cdn.example.com");
    assert_eq!(vmess["tls"]["server_name"], "cdn.example.com");

    let trojan = by_name("Trojan gRPC");
    assert_eq!(trojan["password"], "p@ss:word");
    assert_eq!(trojan["transport"]["service_name"], "tunnel");
    assert_eq!(trojan["tls"]["alpn"], json!(["h2"]));

    assert_eq!(by_name("Upgrade")["transport"]["type"], "httpupgrade");

    let ss = by_name("SS 2022");
    assert_eq!((ss["server"].as_str(), ss["method"].as_str()), (Some("2001:db8::1"), Some("2022-blake3-aes-128-gcm")));
    assert_eq!(ss["password"], "c2VjcmV0c2VjcmV0c2VjcmV0");

    let hy2 = by_name("Hy2");
    assert_eq!(hy2["obfs"], json!({ "type": "salamander", "password": "salt" }));
    assert_eq!(hy2["tls"]["insecure"], true);
}

#[test]
fn a_clash_server_matches_the_same_server_given_as_a_link() {
    let link = "trojan://secret@trojan.example.com:443?security=tls&sni=front.example.com&type=ws&path=%2Fws&host=cdn.example.com#Same";
    let yaml = "proxies:\n  - { name: Same, type: trojan, server: trojan.example.com, port: 443, password: secret, sni: front.example.com, network: ws, ws-opts: { path: /ws, headers: { Host: cdn.example.com } } }\n";
    assert_eq!(outbound(&only(yaml)), outbound(&only(link)));
}

#[test]
fn text_that_is_not_a_config_file_falls_through_to_links() {
    assert!(extract("vless://11111111-2222-4333-8444-555555555555@example.com:443?security=tls").is_none());
    assert!(extract("{ not json").is_none());
    assert!(extract(r#"{ "servers": [] }"#).is_none());
    assert!(extract("name: x\nport: 1\n").is_none());
    // Damaged YAML does not panic and is not treated as a server list.
    assert!(extract("proxies:\n  - [unclosed\n").is_none());
    // A config with no servers is a config: it reports nothing instead of "no links found".
    assert!(extract("proxies: []\n").unwrap().is_empty());
}

#[test]
fn damaged_entries_never_panic() {
    // Wrong types everywhere a field is read; the parsers run with `panic = "abort"` in release.
    let values = [json!(null), json!(true), json!(-1), json!(70000), json!("é"), json!([1, "x"]), json!({ "a": 1 })];
    for kind in ["vless", "vmess", "trojan", "shadowsocks", "hysteria2", "ss"] {
        for value in &values {
            for key in ["server", "server_port", "port", "uuid", "password", "tls", "transport", "network", "ws-opts", "reality-opts", "alpn", "obfs", "tag", "name"] {
                let mut entry = json!({ "type": kind, "tag": "t", "name": "t", "server": "example.com", "server_port": 443, "port": 443 });
                entry[key] = value.clone();
                let _ = parse_many(&json!({ "outbounds": [entry.clone()] }).to_string());
                let _ = parse_many(&format!("proxies:\n  - {entry}\n"));
            }
        }
    }
}
