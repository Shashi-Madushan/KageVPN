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
        log_level: LogLevel::default(),
        mixed_port: DEFAULT_MIXED_PORT,
        clash_port: DEFAULT_CLASH_PORT,
        rule_sets_dir: Path::new("/tmp/kage-vpn-no-rule-sets"),
        secret: "secret",
        proxy_password: None,
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
    DnsSettings { provider: DnsProvider::Custom, custom: custom.into(), ..Default::default() }
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
    assert!(validate_dns(&DnsSettings { provider: DnsProvider::Cloudflare, custom: "junk".into(), ..Default::default() }).is_ok());
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
        routing.rules.push(Rule { id: value.into(), enabled: true, kind, value: value.into(), action, ..Default::default() });
    }
    let cfg = build(&input(&profiles, &routing, Mode::Proxy)).unwrap();
    let rules = cfg["route"]["rules"].as_array().unwrap();
    assert!(rules.iter().any(|r| r["process_path_regex"] == serde_json::json!([r"^/Applications/zoom\.us\.app/"]) && r["outbound"] == "direct"));
    let path = std::env::temp_dir().join(format!("kage-vpn-test-app-rules-{}.json", std::process::id()));
    std::fs::write(&path, serde_json::to_vec(&cfg).unwrap()).unwrap();
    crate::singbox::check(&binary, &path).unwrap();
    let _ = std::fs::remove_file(path);
}

#[test]
fn the_log_level_and_ports_come_from_the_input() {
    let profiles = vec![profile()];
    let routing = RoutingSettings::default();
    let mut config_input = input(&profiles, &routing, Mode::Tun);
    config_input.log_level = LogLevel::Errors;
    config_input.mixed_port = 2185;
    config_input.clash_port = 19191;
    let cfg = build(&config_input).unwrap();
    assert_eq!(cfg["log"]["level"], "warn");
    assert_eq!(cfg["inbounds"][0]["listen_port"], 2185);
    assert_eq!(cfg["experimental"]["clash_api"]["external_controller"], "127.0.0.1:19191");

    for (level, name) in [(LogLevel::Errors, "warn"), (LogLevel::Normal, "info"), (LogLevel::Detailed, "debug")] {
        config_input.log_level = level;
        assert_eq!(build(&config_input).unwrap()["log"]["level"], name);
    }
}

#[test]
fn a_config_with_other_ports_and_a_quiet_log_passes_singbox_check() {
    let Ok(binary) = crate::core::locate(&crate::app_config::SING_BOX) else { return };
    let profiles = vec![profile()];
    let routing = RoutingSettings::default();
    let mut config_input = input(&profiles, &routing, Mode::Proxy);
    config_input.log_level = LogLevel::Errors;
    config_input.mixed_port = 2185;
    config_input.clash_port = 19191;
    let path = std::env::temp_dir().join("kage-vpn-test-ports.json");
    std::fs::write(&path, serde_json::to_vec(&build(&config_input).unwrap()).unwrap()).unwrap();
    crate::singbox::check(&binary, &path).unwrap();
    let _ = std::fs::remove_file(path);
}

#[test]
fn the_local_proxy_asks_for_a_login_only_when_a_password_is_set() {
    let profiles = vec![profile()];
    let routing = RoutingSettings::default();
    let mut config_input = input(&profiles, &routing, Mode::Proxy);
    assert!(build(&config_input).unwrap()["inbounds"][0].get("users").is_none());
    config_input.proxy_password = Some("pw");
    let cfg = build(&config_input).unwrap();
    assert_eq!(cfg["inbounds"][0]["users"][0], serde_json::json!({ "username": PROXY_USER, "password": "pw" }));
    let Ok(binary) = crate::core::locate(&crate::app_config::SING_BOX) else { return };
    let path = std::env::temp_dir().join("kage-vpn-test-proxy-login.json");
    std::fs::write(&path, serde_json::to_vec(&cfg).unwrap()).unwrap();
    crate::singbox::check(&binary, &path).unwrap();
    let _ = std::fs::remove_file(path);
}

#[test]
fn the_default_log_level_is_normal_and_reads_from_old_settings() {
    assert_eq!(LogLevel::default(), LogLevel::Normal);
    let old: crate::store::Settings = serde_json::from_str(r#"{"showSpeed":false,"autoConnect":true}"#).unwrap();
    assert_eq!((old.log_level, old.clear_logs), (LogLevel::Normal, false));
    let new: crate::store::Settings = serde_json::from_str(r#"{"logLevel":"errors","clearLogs":true}"#).unwrap();
    assert_eq!((new.log_level, new.clear_logs), (LogLevel::Errors, true));
}

#[test]
fn ipv6_off_blocks_ipv6_and_passes_singbox_check() {
    let profiles = every_protocol();
    let routing = RoutingSettings { ipv6: false, ..RoutingSettings::default() };
    let cfg = build(&input(&profiles, &routing, Mode::Tun)).unwrap();
    assert_eq!(cfg["dns"]["strategy"], "ipv4_only");
    assert!(cfg["route"]["rules"].as_array().unwrap().iter().any(|r| r["ip_version"] == 6 && r["action"] == "reject"));
    // Still dual-stack, so v6 is captured by the tunnel and rejected rather than leaking.
    assert_eq!(cfg["inbounds"][1]["address"].as_array().unwrap().len(), 2);
    let on = build(&input(&profiles, &RoutingSettings::default(), Mode::Tun)).unwrap();
    assert!(on["dns"].get("strategy").is_none());
    if let Ok(binary) = crate::core::locate(&crate::app_config::SING_BOX) {
        let path = std::env::temp_dir().join("kage-vpn-test-ipv6off.json");
        std::fs::write(&path, serde_json::to_vec(&cfg).unwrap()).unwrap();
        crate::singbox::check(&binary, &path).unwrap();
    }
}

fn fake_ip_config(mode: Mode, route: RouteMode, ipv6: bool) -> serde_json::Value {
    let profiles = every_protocol();
    let mut routing = RoutingSettings::default();
    routing.dns.fake_ip = true;
    routing.mode = route;
    routing.ipv6 = ipv6;
    build(&input(&profiles, &routing, mode)).unwrap()["dns"].clone()
}

#[test]
fn fake_ip_adds_a_server_and_keeps_local_names_real() {
    let dns = fake_ip_config(Mode::Tun, RouteMode::Global, true);
    let servers = dns["servers"].as_array().unwrap();
    let fake = servers.iter().find(|s| s["type"] == "fakeip").expect("fakeip server");
    assert_eq!((fake["tag"].as_str(), fake["inet4_range"].as_str(), fake["inet6_range"].as_str()), (Some("fakeip"), Some("198.18.0.0/15"), Some("fc00::/18")));
    let rules = dns["rules"].as_array().unwrap();
    assert_eq!(rules[0]["server"], "local");
    assert_eq!(rules[1]["server"], "fakeip");
}

#[test]
fn fake_ip_is_left_out_when_it_cannot_help() {
    for dns in [fake_ip_config(Mode::Proxy, RouteMode::Global, true), fake_ip_config(Mode::Tun, RouteMode::Direct, true)] {
        assert!(dns.get("rules").is_none());
        assert!(!dns["servers"].as_array().unwrap().iter().any(|s| s["type"] == "fakeip"));
    }
    let no_v6 = fake_ip_config(Mode::Tun, RouteMode::Global, false);
    assert!(no_v6["servers"].as_array().unwrap().iter().find(|s| s["type"] == "fakeip").unwrap().get("inet6_range").is_none());
}

#[test]
fn fake_ip_config_passes_singbox_check() {
    let Ok(binary) = crate::core::locate(&crate::app_config::SING_BOX) else { return };
    let profiles = every_protocol();
    let mut routing = RoutingSettings::default();
    routing.dns.fake_ip = true;
    let path = std::env::temp_dir().join("kage-vpn-test-fakeip.json");
    std::fs::write(&path, serde_json::to_vec(&build(&input(&profiles, &routing, Mode::Tun)).unwrap()).unwrap()).unwrap();
    crate::singbox::check(&binary, &path).unwrap();
}

// ── Per-app rules ───────────────────────────────────────────────────────────

fn rule(kind: RuleKind, value: &str, action: Action, enabled: bool) -> Rule {
    Rule { id: value.into(), enabled, kind, value: value.into(), action, ..Default::default() }
}

#[test]
fn blocking_this_app_or_the_engine_is_refused() {
    for (kind, value) in [
        (RuleKind::App, "/Applications/Kage VPN.app"),
        (RuleKind::App, "/Users/me/Applications/Kage VPN.app/"),
        (RuleKind::ProcessName, "sing-box"),
        (RuleKind::ProcessName, "Kage VPN"),
    ] {
        let err = validate_rules(&[rule(kind, value, Action::Block, true)]).unwrap_err().to_string();
        assert!(err.contains("cannot be blocked"), "{value}: {err}");
    }
}

#[test]
fn other_apps_and_other_actions_are_accepted() {
    assert!(validate_rules(&[rule(RuleKind::App, "/Applications/Slack.app", Action::Block, true)]).is_ok());
    // Sending this app direct is harmless, and a disabled block does nothing.
    assert!(validate_rules(&[rule(RuleKind::App, "/Applications/Kage VPN.app", Action::Direct, true)]).is_ok());
    assert!(validate_rules(&[rule(RuleKind::ProcessName, "sing-box", Action::Block, false)]).is_ok());
    // A domain that merely contains the name is not an app rule.
    assert!(validate_rules(&[rule(RuleKind::Domain, "kage", Action::Block, true)]).is_ok());
}

#[test]
fn a_rule_saved_before_origin_existed_loads_as_manual() {
    let old: Rule = serde_json::from_str(r#"{"id":"1","enabled":true,"kind":"app","value":"/Applications/Slack.app","action":"direct"}"#).unwrap();
    assert_eq!(old.origin, crate::settings::RuleOrigin::Manual);
    let cap: Rule = serde_json::from_str(r#"{"id":"2","origin":"cap","kind":"app","value":"/x.app","action":"block"}"#).unwrap();
    assert_eq!(cap.origin, crate::settings::RuleOrigin::Cap);
    assert!(serde_json::to_string(&cap).unwrap().contains("\"origin\":\"cap\""));
}
