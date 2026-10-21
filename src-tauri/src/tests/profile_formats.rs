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
            "Tokyo TUIC: TUIC needs a UUID and a password (v4 servers are not supported)",
            "Chained: servers that connect through another server are not supported",
            "No port: no valid port",
        ]
    );
}

#[test]
fn singbox_tuic_outbounds_import_with_their_options() {
    let config = json!({ "outbounds": [{
        "type": "tuic", "tag": "TUIC", "server": "tuic.example.com", "server_port": 443,
        "uuid": "11111111-2222-4333-8444-555555555555", "password": "secret",
        "congestion_control": "new_reno", "udp_relay_mode": "quic", "zero_rtt_handshake": true,
        "tls": { "enabled": true, "server_name": "front.example.com", "alpn": ["h3"] }
    }] });
    let out = outbound(&only(&config.to_string()));
    assert_eq!((out["congestion_control"].as_str(), out["udp_relay_mode"].as_str()), (Some("new_reno"), Some("quic")));
    assert_eq!(out["zero_rtt_handshake"], true);
    assert_eq!(out["tls"]["server_name"], "front.example.com");
}

#[test]
fn singbox_hysteria_outbounds_import_and_explain_what_they_cannot() {
    let config = json!({ "outbounds": [
        { "type": "hysteria", "tag": "HY", "server": "hy.example.com", "server_port": 443, "up": "20 Mbps", "down_mbps": 80, "auth_str": "pw", "tls": { "enabled": true, "alpn": ["hysteria"] } },
        { "type": "hysteria", "tag": "Binary", "server": "b.example.com", "server_port": 443, "up_mbps": 1, "down_mbps": 1, "auth": "c2VjcmV0", "tls": { "enabled": true } },
        { "type": "hysteria", "tag": "Hop", "server": "h.example.com", "server_ports": ["1000:2000"], "up_mbps": 1, "down_mbps": 1, "tls": { "enabled": true } }
    ] });
    let (profiles, errors) = parse_many(&config.to_string());
    assert_eq!(profiles.len(), 1);
    let out = outbound(&profiles[0]);
    assert_eq!((out["up_mbps"].as_u64(), out["down_mbps"].as_u64(), out["auth_str"].as_str()), (Some(20), Some(80), Some("pw")));
    assert_eq!(
        errors,
        [
            "Binary: Hysteria with a binary (base64) auth key is not supported, use auth_str",
            "Hop: Hysteria port hopping is not supported, use a single port",
        ]
    );
}

#[test]
fn singbox_ssh_outbounds_take_the_key_as_text_or_lines_but_not_a_file() {
    let config = json!({ "outbounds": [
        { "type": "ssh", "tag": "Lines", "server": "a.example.com", "server_port": 22, "user": "root",
          "private_key": ["-----BEGIN OPENSSH PRIVATE KEY-----", "AAAA", "-----END OPENSSH PRIVATE KEY-----"] },
        { "type": "ssh", "tag": "File", "server": "b.example.com", "server_port": 22, "user": "root", "private_key_path": "/root/.ssh/id" }
    ] });
    let (profiles, errors) = parse_many(&config.to_string());
    assert_eq!(profiles.len(), 1);
    assert_eq!(outbound(&profiles[0])["private_key"], "-----BEGIN OPENSSH PRIVATE KEY-----\nAAAA\n-----END OPENSSH PRIVATE KEY-----\n");
    assert_eq!(errors, ["File: SSH key files on disk are not supported, paste the key into the server editor"]);
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
  - { name: TUIC, type: tuic, server: tuic.example.com, port: 8443, uuid: 11111111-2222-4333-8444-555555555555, password: "p@ss", sni: tuic.example.com, alpn: [h3], congestion-controller: cubic, udp-relay-mode: quic, reduce-rtt: true, skip-cert-verify: true }
  - { name: TUIC v4, type: tuic, server: v4.example.com, port: 443, token: abc }
  - { name: HY1, type: hysteria, server: hy1.example.com, port: 443, auth-str: secret, up: "30 Mbps", down: 200, obfs: mask, sni: hy1.example.com, alpn: [hysteria] }
  - { name: SSH, type: ssh, server: ssh.example.com, port: 2222, username: deploy, password: pw, host-key: ["ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGE58h0urP4jPA6D6Ee+UnCVCbe6FbFtJbHa8Nh7CW/r"] }
  - { name: SSH file, type: ssh, server: ssh.example.com, port: 22, username: root, private-key: ~/.ssh/id_ed25519 }
  - { name: HY1 faketcp, type: hysteria, server: fake.example.com, port: 443, protocol: faketcp, up: 10, down: 10 }
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
    assert_eq!(names, ["🇺🇸 VLESS Reality", "VMess WS", "Trojan gRPC", "Upgrade", "SS 2022", "Hy2", "TUIC", "HY1", "SSH"]);
    assert_eq!(
        errors,
        [
            "SS obfs: Shadowsocks plugins are not supported",
            "Hopping: Hysteria2 port hopping is not supported, use a single port",
            "Plain VLESS: only security=tls or reality is supported",
            "TUIC v4: TUIC v4 is not supported, ask for a v5 server (UUID and password)",
            "SSH file: SSH key files on disk are not supported, paste the key into the server editor",
            "HY1 faketcp: Hysteria over faketcp is not supported, only udp",
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

    let tuic = by_name("TUIC");
    assert_eq!((tuic["server_port"].as_u64(), tuic["password"].as_str()), (Some(8443), Some("p@ss")));
    assert_eq!((tuic["congestion_control"].as_str(), tuic["udp_relay_mode"].as_str()), (Some("cubic"), Some("quic")));
    assert_eq!(tuic["zero_rtt_handshake"], true);
    assert_eq!(tuic["tls"]["insecure"], true);

    let hy1 = by_name("HY1");
    assert_eq!((hy1["up_mbps"].as_u64(), hy1["down_mbps"].as_u64()), (Some(30), Some(200)));
    assert_eq!((hy1["auth_str"].as_str(), hy1["obfs"].as_str()), (Some("secret"), Some("mask")));

    let ssh = by_name("SSH");
    assert_eq!((ssh["server_port"].as_u64(), ssh["user"].as_str(), ssh["password"].as_str()), (Some(2222), Some("deploy"), Some("pw")));
    assert!(ssh["host_key"][0].as_str().unwrap().starts_with("ssh-ed25519 AAAA"));
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
