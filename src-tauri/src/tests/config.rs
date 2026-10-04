use super::*;

const PIN: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

fn profile() -> Profile {
    let link = format!("vless://11111111-2222-4333-8444-555555555555@example.com:443?security=tls&sni=sni.example.com&fp=chrome&alpn=h2&pcs=00");
    let mut profile = crate::profile::parse_many(&link).0.remove(0);
    profile.id = "1".into();
    profile.name = "test".into();
    profile.set_setting("tls.pinnedKey", serde_json::json!(PIN));
    profile
}

fn input<'a>(profiles: &'a [Profile], routing: &'a RoutingSettings, mode: Mode) -> ConfigInput<'a> {
    ConfigInput {
        profiles,
        active_id: Some("1"),
        auto: false,
        mode,
        routing,
        log_path: Path::new("/tmp/log"),
        cache_path: Path::new("/tmp/cache.db"),
        secret: "secret",
    }
}

/// One profile per protocol, so the real `sing-box check` vets every outbound shape.
fn every_protocol() -> Vec<Profile> {
    let links = [
        "trojan://secret@t.example.com:443?security=tls&type=ws&path=%2Fws&host=cdn.example.com&alpn=h2",
        "ss://YWVzLTI1Ni1nY206cGE1NXdvcmQ@s.example.com:8388",
        "ss://2022-blake3-aes-128-gcm:YWJjZGVmZ2hpamtsbW5vcA%3D%3D@s.example.com:8389",
        "hysteria2://auth@h.example.com:443/?sni=h.example.com&obfs=salamander&obfs-password=mask&insecure=1",
    ];
    let mut profiles = vec![profile()];
    profiles.extend(links.iter().map(|link| crate::profile::parse_many(link).0.remove(0)));
    profiles
}

#[test]
fn generated_configs_pass_singbox_check() {
    let Ok(binary) = crate::core::locate(&crate::app_config::SING_BOX) else { return };
    let profiles = every_protocol();
    let routing = RoutingSettings::default();
    for mode in [Mode::Tun, Mode::Proxy] {
        let path = std::env::temp_dir().join(format!("apzur-vpn-test-{mode:?}.json"));
        let cfg = build(&input(&profiles, &routing, mode)).unwrap();
        std::fs::write(&path, serde_json::to_vec(&cfg).unwrap()).unwrap();
        crate::singbox::check(&binary, &path).unwrap();
    }
}
