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

/// The pinned VLESS profile plus every registered protocol's sample, so the real `sing-box check`
/// vets each outbound shape and a new protocol is covered without editing this test.
fn every_protocol() -> Vec<Profile> {
    let mut profiles = vec![profile()];
    for (index, def) in crate::profile::REGISTRY.iter().enumerate() {
        let mut sample = def.parse(def.sample()).unwrap();
        sample.id = format!("sample-{index}");
        profiles.push(sample);
    }
    profiles
}

#[test]
fn generated_configs_pass_singbox_check() {
    let Ok(binary) = crate::core::locate(&crate::app_config::SING_BOX) else { return };
    let profiles = every_protocol();
    let routing = RoutingSettings::default();
    for mode in [Mode::Tun, Mode::Proxy] {
        let path = std::env::temp_dir().join(format!("kage-vpn-test-{mode:?}.json"));
        let cfg = build(&input(&profiles, &routing, mode)).unwrap();
        std::fs::write(&path, serde_json::to_vec(&cfg).unwrap()).unwrap();
        crate::singbox::check(&binary, &path).unwrap();
    }
}
