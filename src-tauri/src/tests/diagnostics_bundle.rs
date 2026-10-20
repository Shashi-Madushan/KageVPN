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

/// Reads the archive back with macOS's own `unzip`, which also proves the bytes form a valid zip.
fn unzip(bytes: Vec<u8>) -> Vec<(String, String)> {
    let dir = std::env::temp_dir().join(format!("kage-bundle-{}-{}", std::process::id(), bytes.len()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let archive = dir.join("bundle.zip");
    std::fs::write(&archive, bytes).unwrap();
    let test = std::process::Command::new("/usr/bin/unzip").arg("-tq").arg(&archive).output().unwrap();
    assert!(test.status.success(), "unzip rejected the archive: {}", String::from_utf8_lossy(&test.stdout));
    let extracted = dir.join("out");
    let status = std::process::Command::new("/usr/bin/unzip").arg("-q").arg(&archive).arg("-d").arg(&extracted).status().unwrap();
    assert!(status.success());
    let mut files: Vec<(String, String)> = std::fs::read_dir(&extracted)
        .unwrap()
        .map(|e| {
            let e = e.unwrap();
            (e.file_name().to_string_lossy().into_owned(), std::fs::read_to_string(e.path()).unwrap())
        })
        .collect();
    files.sort();
    let _ = std::fs::remove_dir_all(dir);
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
