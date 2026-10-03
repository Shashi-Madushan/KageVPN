use super::*;

const PIN: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

fn profile() -> Profile {
    Profile {
        id: "1".into(),
        name: "test".into(),
        server: "example.com".into(),
        port: 443,
        uuid: "11111111-2222-4333-8444-555555555555".into(),
        sni: Some("sni.example.com".into()),
        fingerprint: Some("chrome".into()),
        alpn: vec!["h2".into()],
        pinned_key: Some(PIN.into()),
        ..Profile::default()
    }
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

#[test]
fn generated_configs_pass_singbox_check() {
    let Ok(binary) = crate::singbox::locate() else { return };
    let profiles = [profile()];
    let routing = RoutingSettings::default();
    for mode in [Mode::Tun, Mode::Proxy] {
        let path = std::env::temp_dir().join(format!("singbox-gui-test-{mode:?}.json"));
        let cfg = build(&input(&profiles, &routing, mode)).unwrap();
        std::fs::write(&path, serde_json::to_vec(&cfg).unwrap()).unwrap();
        crate::singbox::check(&binary, &path).unwrap();
    }
}
