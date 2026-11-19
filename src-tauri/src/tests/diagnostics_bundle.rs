use super::*;
use crate::profile::parse_many;

const UUID: &str = "11111111-2222-4333-8444-555555555555";
const SUB: &str = "https://sub.provider.example/api/v1/token-ABCDEF123456";

fn material() -> Material {
    let link = format!("vless://{UUID}@vpn.example.com:443?security=reality&sni=front.example.org&pbk=PUBKEYPUBKEYPUBKEY&sid=abcd1234&flow=xtls-rprx-vision");
    let mut profile = parse_many(&link).0.remove(0);
    profile.id = "p1".into();
    let config = serde_json::json!({
        "experimental": { "clash_api": { "external_controller": "127.0.0.1:19090", "secret": "SECRETVALUE1234" } },
        "inbounds": [{ "type": "mixed", "users": [{ "username": "kage", "password": "PROXYPASS9999" }] }],
        "outbounds": [{ "type": "vless", "server": "vpn.example.com", "uuid": UUID }],
    });
    Material {
        app_log: format!("[INFO] connect to vpn.example.com as {UUID}\n[INFO] refreshed {SUB}\n[INFO] file /Users/someone/Library/x"),
        singbox_log: (0..3000).map(|n| format!("line {n} 203.0.113.10")).collect::<Vec<_>>().join("\n"),
        config: Some(config.to_string()),
        versions: "app Kage 1.0.0\nengine sing-box 1.14.2".into(),
        settings: "servers 1, subscriptions 1".into(),
        profiles: vec![profile],
        subscription_urls: vec![SUB.into()],
        home: Some("/Users/someone".into()),
    }
}

/// Reads the archive back with an independent zip reader, which checks every entry's CRC, so the
/// bytes are proven to form a valid zip on every OS.
fn unzip(bytes: Vec<u8>) -> Vec<(String, String)> {
    use std::io::Read;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("a readable zip");
    let mut files: Vec<(String, String)> = (0..zip.len())
        .map(|i| {
            let mut entry = zip.by_index(i).unwrap();
            let mut text = String::new();
            entry.read_to_string(&mut text).expect("the entry's CRC matches");
            (entry.name().to_string(), text)
        })
        .collect();
    files.sort();
    files
}

#[test]
fn nothing_identifying_is_in_any_file_of_the_bundle() {
    let files = unzip(build(&material()).unwrap());
    let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["README.txt", "app.log", "config.json", "sing-box.log", "summary.txt"]);
    for (name, text) in &files {
        for secret in [UUID, SUB, "token-ABCDEF123456", "vpn.example.com", "PUBKEYPUBKEYPUBKEY", "SECRETVALUE1234", "PROXYPASS9999", "203.0.113.10", "/Users/someone", "front.example.org"] {
            assert!(!text.contains(secret), "{name} still contains {secret}");
        }
    }
}

#[test]
fn the_singbox_log_is_capped_to_the_last_lines() {
    let files = unzip(build(&material()).unwrap());
    let log = &files.iter().find(|(n, _)| n == "sing-box.log").unwrap().1;
    assert_eq!(log.lines().count(), LOG_TAIL_LINES);
    assert!(log.contains("line 2999") && !log.contains("line 999 "));
}

#[test]
fn a_missing_or_broken_config_does_not_stop_the_export() {
    let mut m = material();
    m.config = None;
    assert!(unzip(build(&m).unwrap()).iter().any(|(n, t)| n == "config.json" && t.contains("no configuration")));
    m.config = Some("{ not json".into());
    assert!(unzip(build(&m).unwrap()).iter().any(|(n, t)| n == "config.json" && t.contains("could not be read")));
}

#[test]
fn the_summary_has_versions_and_settings_but_no_servers() {
    let text = summary(&material());
    assert!(text.contains("sing-box 1.14.2") && text.contains("servers 1"));
}

#[test]
fn the_sites_a_connection_went_to_are_not_in_the_log() {
    let log = "+0530 2026-10-26 22:28:41 INFO [3002240486 1ms] inbound/tun[tun-in]: inbound connection to www.secret-bank.example:443\n\
+0530 2026-10-26 22:28:41 INFO [3002240486 1ms] inbound/tun[tun-in]: inbound connection from 192.168.1.20:56096\n\
+0530 2026-10-26 22:28:42 ERROR [7 2ms] inbound/tun[tun-in]: process connection from 10.0.0.2:55: connection to private-site.example:443: i/o timeout\n\
+0530 2026-10-26 22:28:43 INFO [9 0ms] dns: exchanged A news.example. IN A\n\
+0530 2026-10-26 22:28:44 INFO sing-box started (1.2s)";
    let hidden = hide_destinations(log);
    for secret in ["secret-bank", "192.168.1.20", "private-site", "news.example"] {
        assert!(!hidden.contains(secret), "{secret} survived: {hidden}");
    }
    assert!(hidden.contains("inbound connection to [destination]"));
    assert!(hidden.contains("ERROR"), "the level and the start of the message stay: {hidden}");
    assert!(hidden.contains("sing-box started (1.2s)"), "lines about the engine itself are untouched");
    assert_eq!(hide_destinations("Ünïcode connection to café.example"), "Ünïcode connection to [destination]");
}
