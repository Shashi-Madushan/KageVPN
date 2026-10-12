use super::*;

const PIN: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

fn profile() -> Profile {
    let link = "vless://11111111-2222-4333-8444-555555555555@example.com:443?security=tls&sni=sni.example.com&fp=chrome&alpn=h2&pcs=00";
    let mut profile = crate::profile::parse_many(link).0.remove(0);
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
        rule_sets_dir: Path::new("/tmp/kage-vpn-no-rule-sets"),
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

fn dns_with(custom: &str) -> DnsSettings {
    DnsSettings { provider: DnsProvider::Custom, custom: custom.into() }
}

#[test]
fn a_custom_dns_link_keeps_its_scheme_port_and_path() {
    let server = |text: &str| custom_dns_server(text).unwrap_or_else(|| panic!("{text} should parse"));

    // Personal DoH endpoints carry the account in the path; dropping it queries the wrong resolver.
    let nextdns = server("https://dns.nextdns.io/abc123");
    assert_eq!((nextdns["type"].as_str(), nextdns["server"].as_str(), nextdns["path"].as_str()), (Some("https"), Some("dns.nextdns.io"), Some("/abc123")));

    // A bare host is DNS over HTTPS on the default path, as before.
    let bare = server("dns.example.com");
    assert_eq!((bare["type"].as_str(), bare["server"].as_str()), (Some("https"), Some("dns.example.com")));
    assert!(bare.get("path").is_none() && bare.get("server_port").is_none());
    assert!(server("https://dns.example.com/").get("path").is_none());

    assert_eq!(server("tls://dns.google")["type"], "tls");
    assert_eq!(server("quic://dns.adguard-dns.com")["type"], "quic");
    let udp = server("udp://8.8.8.8:5353");
    assert_eq!((udp["type"].as_str(), udp["server_port"].as_u64()), (Some("udp"), Some(5353)));
    assert_eq!(server("https://[2606:4700:4700::1111]/dns-query")["server"], "2606:4700:4700::1111");
    assert_eq!(server("  https://dns.example.com:8443/dns-query  ")["server_port"], 8443);
}

#[test]
fn a_custom_dns_entry_that_cannot_work_is_refused_not_ignored() {
    for bad in ["", "   ", "ftp://dns.example.com", "https://", "not a host"] {
        assert!(custom_dns_server(bad).is_none(), "{bad:?}");
        assert!(validate_dns(&dns_with(bad)).is_err(), "{bad:?}");
    }
    assert!(validate_dns(&dns_with("tls://dns.google")).is_ok());
    // Only the custom provider reads the text.
    assert!(validate_dns(&DnsSettings { provider: DnsProvider::Cloudflare, custom: "junk".into() }).is_ok());
}

#[test]
fn every_custom_dns_shape_passes_singbox_check() {
    let Ok(binary) = crate::core::locate(&crate::app_config::SING_BOX) else { return };
    let profiles = vec![profile()];
    for text in [
        "dns.example.com",
        "https://dns.nextdns.io/abc123",
        "https://dns.example.com:8443/dns-query",
        "tls://dns.google",
        "quic://dns.adguard-dns.com",
        "udp://8.8.8.8",
        "tcp://8.8.8.8:5353",
        "https://[2606:4700:4700::1111]/dns-query",
    ] {
        let routing = RoutingSettings { dns: dns_with(text), ..RoutingSettings::default() };
        let path = std::env::temp_dir().join(format!("kage-vpn-dns-{}.json", text.replace(['/', ':', '[', ']', '.'], "_")));
        std::fs::write(&path, serde_json::to_vec(&build(&input(&profiles, &routing, Mode::Proxy)).unwrap()).unwrap()).unwrap();
        crate::singbox::check(&binary, &path).unwrap_or_else(|e| panic!("{text}: {e}"));
        let _ = std::fs::remove_file(path);
    }
}

#[test]
fn the_tunnel_carries_both_address_families_so_ipv6_cannot_skip_the_vpn() {
    let profiles = vec![profile()];
    let routing = RoutingSettings::default();
    let config = build(&input(&profiles, &routing, Mode::Tun)).unwrap();
    let tun = config["inbounds"].as_array().unwrap().iter().find(|i| i["type"] == "tun").expect("a tun inbound");
    let addresses: Vec<&str> = tun["address"].as_array().unwrap().iter().filter_map(|a| a.as_str()).collect();
    assert!(addresses.iter().any(|a| a.contains('.')), "IPv4 prefix: {addresses:?}");
    assert!(addresses.iter().any(|a| a.contains(':')), "IPv6 prefix: {addresses:?}");

    let proxy_only = build(&input(&profiles, &routing, Mode::Proxy)).unwrap();
    assert!(proxy_only["inbounds"].as_array().unwrap().iter().all(|i| i["type"] != "tun"));
}

/// Compiles a tiny real rule-set with the installed sing-box, or `None` when there is none.
fn compile_list(dir: &Path, name: &str) -> Option<std::path::PathBuf> {
    let binary = crate::core::locate(&crate::app_config::SING_BOX).ok()?;
    let source = dir.join(format!("{name}.json"));
    std::fs::write(&source, r#"{"version":3,"rules":[{"ip_cidr":["10.0.0.0/8"]}]}"#).unwrap();
    let output = dir.join(format!("{name}.srs"));
    let status = std::process::Command::new(binary).args(["rule-set", "compile"]).arg(&source).arg("-o").arg(&output).status().ok()?;
    status.success().then_some(output)
}

fn rule_mode(sets: Vec<crate::settings::RuleSetRef>) -> RoutingSettings {
    RoutingSettings { mode: crate::settings::RouteMode::Rule, rule_sets: sets, ..RoutingSettings::default() }
}

fn country(kind: crate::settings::RuleSetKind, code: &str, action: crate::settings::Action) -> crate::settings::RuleSetRef {
    crate::settings::RuleSetRef { id: code.into(), enabled: true, kind, country: code.into(), action }
}

#[test]
fn country_lists_become_local_files_so_startup_never_needs_the_network() {
    use crate::settings::{Action, RuleSetKind};
    let dir = std::env::temp_dir().join(format!("kage-config-lists-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("geoip-cn.srs"), b"SRS\x01 stand-in").unwrap();

    let profiles = vec![profile()];
    let routing = rule_mode(vec![country(RuleSetKind::Geoip, "cn", Action::Direct)]);
    let mut config_input = input(&profiles, &routing, Mode::Proxy);
    config_input.rule_sets_dir = &dir;
    let config = build(&config_input).unwrap();

    let definition = &config["route"]["rule_set"][0];
    assert_eq!((definition["type"].as_str(), definition["tag"].as_str()), (Some("local"), Some("geoip-cn")));
    assert_eq!(definition["path"].as_str(), Some(dir.join("geoip-cn.srs").to_str().unwrap()));
    assert!(definition.get("url").is_none() && definition.get("download_detour").is_none(), "nothing for sing-box to download");
    assert!(config["experimental"].get("cache_file").is_none());
    assert!(config["route"]["rules"].as_array().unwrap().iter().any(|r| r["rule_set"] == serde_json::json!(["geoip-cn"])));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_list_that_is_not_on_disk_is_left_out_instead_of_breaking_the_config() {
    use crate::settings::{Action, RuleSetKind};
    let dir = std::env::temp_dir().join(format!("kage-config-nolists-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("geoip-cn.srs"), b"SRS\x01 stand-in").unwrap();

    let profiles = vec![profile()];
    let routing = rule_mode(vec![country(RuleSetKind::Geoip, "cn", Action::Direct), country(RuleSetKind::Geosite, "category-ads-all", Action::Block)]);
    let mut config_input = input(&profiles, &routing, Mode::Proxy);
    config_input.rule_sets_dir = &dir;
    let config = build(&config_input).unwrap();

    let tags: Vec<&str> = config["route"]["rule_set"].as_array().unwrap().iter().filter_map(|d| d["tag"].as_str()).collect();
    assert_eq!(tags, ["geoip-cn"], "only the list that exists is declared");
    let rules = serde_json::to_string(&config["route"]["rules"]).unwrap();
    assert!(rules.contains("geoip-cn") && !rules.contains("category-ads-all"), "and no rule refers to one that is not: {rules}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn no_lists_means_no_rule_set_section_at_all() {
    let profiles = vec![profile()];
    let config = build(&input(&profiles, &RoutingSettings::default(), Mode::Proxy)).unwrap();
    assert!(config["route"].get("rule_set").is_none());
}

#[test]
fn real_compiled_lists_pass_singbox_check() {
    use crate::settings::{Action, RuleSetKind};
    let dir = std::env::temp_dir().join(format!("kage-config-real-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (Some(_), Some(_)) = (compile_list(&dir, "geoip-cn"), compile_list(&dir, "geosite-category-ads-all")) else { return };
    let Ok(binary) = crate::core::locate(&crate::app_config::SING_BOX) else { return };

    let profiles = vec![profile()];
    let routing = rule_mode(vec![country(RuleSetKind::Geoip, "cn", Action::Direct), country(RuleSetKind::Geosite, "category-ads-all", Action::Block)]);
    for mode in [Mode::Tun, Mode::Proxy] {
        let mut config_input = input(&profiles, &routing, mode);
        config_input.rule_sets_dir = &dir;
        let path = dir.join(format!("config-{mode:?}.json"));
        std::fs::write(&path, serde_json::to_vec(&build(&config_input).unwrap()).unwrap()).unwrap();
        crate::singbox::check(&binary, &path).unwrap();
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_app_rule_matches_every_process_inside_the_bundle() {
    assert_eq!(app_path_pattern("/Applications/Slack.app"), r"^/Applications/Slack\.app/");
    // Characters that mean something in a pattern are taken literally.
    assert_eq!(app_path_pattern("/Applications/C++ (Beta) [x].app/"), r"^/Applications/C\+\+ \(Beta\) \[x\]\.app/");
    assert_eq!(quote_meta(r"a\b$^{}|?*"), r"a\\b\$\^\{\}\|\?\*");
}

#[test]
fn app_rules_pass_singbox_check() {
    let Ok(binary) = crate::core::locate(&crate::app_config::SING_BOX) else { return };
    let profiles = vec![profile()];
    let mut routing = RoutingSettings { mode: RouteMode::Rule, ..RoutingSettings::default() };
    for (kind, value, action) in [
        (RuleKind::App, "/Applications/zoom.us.app", Action::Direct),
        (RuleKind::App, "/Applications/Google Chrome.app", Action::Proxy),
        (RuleKind::ProcessName, "curl", Action::Block),
    ] {
        routing.rules.push(Rule { id: value.into(), enabled: true, kind, value: value.into(), action });
    }
    let cfg = build(&input(&profiles, &routing, Mode::Proxy)).unwrap();
    let rules = cfg["route"]["rules"].as_array().unwrap();
    assert!(rules.iter().any(|r| r["process_path_regex"] == serde_json::json!([r"^/Applications/zoom\.us\.app/"]) && r["outbound"] == "direct"));
    let path = std::env::temp_dir().join(format!("kage-vpn-test-app-rules-{}.json", std::process::id()));
    std::fs::write(&path, serde_json::to_vec(&cfg).unwrap()).unwrap();
    crate::singbox::check(&binary, &path).unwrap();
    let _ = std::fs::remove_file(path);
}
