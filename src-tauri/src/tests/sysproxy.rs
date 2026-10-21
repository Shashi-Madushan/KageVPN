use std::cell::RefCell;
use std::collections::HashMap;

use super::*;

const SERVICES: &str = "An asterisk (*) denotes that a network service is disabled.\nWi-Fi\nUSB 10/100 LAN\n*Thunderbolt Bridge\nBob's Dock\n";
const OFF: &str = "Enabled: No\nServer: \nPort: 0\nAuthenticated Proxy Enabled: 0\n";

fn on(host: &str, port: u16) -> String {
    format!("Enabled: Yes\nServer: {host}\nPort: {port}\nAuthenticated Proxy Enabled: 0\n")
}

/// A pretend macOS: answers `-get...` from its own settings and applies `-set...` to them.
#[derive(Default)]
struct FakeMac {
    services: RefCell<Vec<String>>,
    settings: RefCell<HashMap<String, ServiceProxy>>,
    calls: RefCell<Vec<Vec<String>>>,
    /// A command that fails once, to test what happens half-way.
    fail_on: RefCell<Option<(String, String)>>,
}

impl FakeMac {
    fn new(services: &[ServiceProxy]) -> Self {
        let mac = Self::default();
        for s in services {
            mac.services.borrow_mut().push(s.service.clone());
            mac.settings.borrow_mut().insert(s.service.clone(), s.clone());
        }
        mac
    }

    fn get(&self, service: &str) -> ServiceProxy {
        self.settings.borrow()[service].clone()
    }

    fn sets(&self) -> usize {
        self.calls.borrow().iter().filter(|c| c[0].starts_with("-set")).count()
    }
}

fn text(p: &Proxy) -> String {
    let enabled = if p.enabled { "Yes" } else { "No" };
    format!("Enabled: {enabled}\nServer: {}\nPort: {}\nAuthenticated Proxy Enabled: {}\n", p.host, p.port, u8::from(p.authenticated))
}

impl Runner for FakeMac {
    fn run(&self, args: &[String]) -> crate::error::Result<String> {
        self.calls.borrow_mut().push(args.to_vec());
        // Fails once, like a passing hiccup, so a rollback after it can succeed.
        let failing = self.fail_on.borrow().as_ref().is_some_and(|(command, service)| &args[0] == command && args.get(1) == Some(service));
        if failing {
            self.fail_on.borrow_mut().take();
            return Err(AppError::Launch("refused".into()));
        }
        if args[0] == "-listallnetworkservices" {
            let mut out = "An asterisk (*) denotes that a network service is disabled.\n".to_string();
            for s in self.services.borrow().iter() {
                out.push_str(s);
                out.push('\n');
            }
            return Ok(out);
        }
        let service = args[1].clone();
        let mut all = self.settings.borrow_mut();
        let s = all.get_mut(&service).expect("known service");
        let proxy = |s: &mut ServiceProxy, which: &str| -> *mut Proxy {
            match which {
                "web" => &mut s.web,
                "secureweb" => &mut s.secure_web,
                _ => &mut s.socks,
            }
        };
        let which = |cmd: &str| if cmd.contains("secureweb") { "secureweb" } else if cmd.contains("socks") { "socks" } else { "web" };
        Ok(match args[0].as_str() {
            "-getwebproxy" => text(&s.web),
            "-getsecurewebproxy" => text(&s.secure_web),
            "-getsocksfirewallproxy" => text(&s.socks),
            "-getautoproxyurl" => format!("URL: {}\nEnabled: {}\n", s.auto_url.clone().unwrap_or("(null)".into()), if s.auto_enabled { "Yes" } else { "No" }),
            "-getproxyautodiscovery" => format!("Auto Proxy Discovery: {}\n", if s.auto_discovery { "On" } else { "Off" }),
            "-getproxybypassdomains" if s.bypass.is_empty() => format!("There aren't any bypass domains set on {service}.\n"),
            "-getproxybypassdomains" => s.bypass.join("\n") + "\n",
            "-setwebproxy" | "-setsecurewebproxy" | "-setsocksfirewallproxy" => {
                // SAFETY: the pointer is into `s`, borrowed mutably for this block only.
                let p = unsafe { &mut *proxy(s, which(&args[0])) };
                *p = Proxy { enabled: true, host: args[2].clone(), port: args[3].parse().unwrap(), authenticated: false };
                String::new()
            }
            "-setwebproxystate" | "-setsecurewebproxystate" | "-setsocksfirewallproxystate" => {
                let p = unsafe { &mut *proxy(s, which(&args[0])) };
                p.enabled = args[2] == "on";
                String::new()
            }
            "-setautoproxyurl" => {
                s.auto_url = Some(args[2].clone());
                s.auto_enabled = true;
                String::new()
            }
            "-setautoproxystate" => {
                s.auto_enabled = args[2] == "on";
                String::new()
            }
            "-setproxyautodiscovery" => {
                s.auto_discovery = args[2] == "on";
                String::new()
            }
            "-setproxybypassdomains" => {
                s.bypass = if args[2] == "Empty" { Vec::new() } else { args[2..].to_vec() };
                String::new()
            }
            other => panic!("unexpected {other}"),
        })
    }
}

fn service(name: &str) -> ServiceProxy {
    ServiceProxy { service: name.into(), ..ServiceProxy::default() }
}

/// A service with a work PAC file, an HTTP proxy and its own bypass list.
fn work_wifi() -> ServiceProxy {
    ServiceProxy {
        service: "Wi-Fi".into(),
        web: Proxy { enabled: true, host: "proxy.corp.example".into(), port: 3128, authenticated: false },
        auto_url: Some("http://wpad.corp.example/proxy.pac".into()),
        auto_enabled: true,
        auto_discovery: true,
        bypass: vec!["*.corp.example".into()],
        ..service("Wi-Fi")
    }
}

/// A fresh folder per test, removed when the guard goes.
struct Dir(std::path::PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn paths() -> (Dir, AppPaths) {
    let dir = std::env::temp_dir().join(format!("kage-sysproxy-{}", uuid::Uuid::new_v4()));
    let paths = AppPaths::new(dir.clone()).unwrap();
    (Dir(dir), paths)
}

#[test]
fn reads_networksetup_output() {
    assert_eq!(parse_services(SERVICES), ["Wi-Fi", "USB 10/100 LAN", "Bob's Dock"]);
    assert_eq!(parse_proxy(&on("127.0.0.1", 2080)), Proxy { enabled: true, host: "127.0.0.1".into(), port: 2080, authenticated: false });
    assert_eq!(parse_proxy(OFF), Proxy::default());
    assert!(parse_proxy("Enabled: Yes\nServer: p\nPort: 1\nAuthenticated Proxy Enabled: 1\n").authenticated);
    assert_eq!(parse_auto_url("URL: (null)\nEnabled: No\n"), (None, false));
    assert_eq!(parse_auto_url("URL: http://w/p.pac\nEnabled: Yes\n"), (Some("http://w/p.pac".into()), true));
    assert!(parse_auto_discovery("Auto Proxy Discovery: On\n"));
    assert!(parse_bypass("There aren't any bypass domains set on Wi-Fi.\n").is_empty());
    assert_eq!(parse_bypass("*.local\n169.254/16\n"), ["*.local", "169.254/16"]);
}

#[test]
fn the_bypass_list_puts_the_built_in_entries_first_and_checks_the_rest() {
    let list = bypass_list(&[" *.corp.example ".into(), "localhost".into(), "".into(), "10.20.0.0/16".into(), "fd00::/8".into()]).unwrap();
    assert_eq!(&list[..BUILT_IN_BYPASS.len()], BUILT_IN_BYPASS);
    assert_eq!(&list[BUILT_IN_BYPASS.len()..], ["*.corp.example", "10.20.0.0/16", "fd00::/8"]);
    for bad in ["http://x", "a b", "10.0.0.0/33", "host/8", "-bad.example", "*.", "a..b"] {
        assert_eq!(bypass_list(&[bad.into()]).unwrap_err().to_string(), format!("'{bad}' is not a host name or address range"), "{bad}");
    }
    assert!(bypass_list(&vec!["a.example".to_string(); 1].into_iter().chain((0..201).map(|i| format!("h{i}.example"))).collect::<Vec<_>>()).is_err());
}

#[test]
fn kage_owns_a_service_only_while_it_points_at_its_port() {
    let mut s = service("Wi-Fi");
    assert!(!still_kage(&s, 2080));
    s.socks = Proxy { enabled: true, host: "127.0.0.1".into(), port: 2080, authenticated: false };
    assert!(still_kage(&s, 2080));
    assert!(!still_kage(&s, 2081));
    s.socks.enabled = false;
    assert!(!still_kage(&s, 2080));
}

#[test]
fn set_then_restore_puts_every_value_back() {
    let (_dir, paths) = paths();
    let mac = FakeMac::new(&[work_wifi(), service("USB 10/100 LAN")]);
    let state = apply(&paths, &mac, 21080, &["*.example.net".into()]);
    assert_eq!(state, SystemProxyState::Set { port: 21080, services: vec!["Wi-Fi".into(), "USB 10/100 LAN".into()] });

    let wifi = mac.get("Wi-Fi");
    for p in [&wifi.web, &wifi.secure_web, &wifi.socks] {
        assert_eq!((p.enabled, p.host.as_str(), p.port), (true, "127.0.0.1", 21080));
    }
    assert!(!wifi.auto_enabled && !wifi.auto_discovery, "PAC and discovery would win over the manual proxies");
    assert_eq!(wifi.bypass.last().map(String::as_str), Some("*.example.net"));
    assert!(has_snapshot(&paths) && paths.system_proxy_script().exists());

    assert_eq!(restore(&paths, &mac), SystemProxyState::Off);
    let back = mac.get("Wi-Fi");
    assert_eq!((back.web.clone(), back.auto_url.clone(), back.auto_enabled, back.auto_discovery, back.bypass.clone()), (work_wifi().web, work_wifi().auto_url, true, true, work_wifi().bypass));
    assert_eq!((back.secure_web.clone(), back.socks.clone()), (Proxy::default(), Proxy::default()), "off, and their old (empty) addresses back");
    assert!(!mac.get("USB 10/100 LAN").web.enabled && mac.get("USB 10/100 LAN").bypass.is_empty());
    assert!(!has_snapshot(&paths) && !paths.system_proxy_script().exists());
}

#[test]
fn a_service_the_user_changed_is_left_alone() {
    let (_dir, paths) = paths();
    let mac = FakeMac::new(&[work_wifi(), service("USB 10/100 LAN")]);
    apply(&paths, &mac, 21080, &[]);
    // The user switches Wi-Fi's proxies off by hand while connected.
    {
        let mut all = mac.settings.borrow_mut();
        let wifi = all.get_mut("Wi-Fi").unwrap();
        wifi.web.enabled = false;
        wifi.secure_web.enabled = false;
        wifi.socks.enabled = false;
    }
    assert_eq!(restore(&paths, &mac), SystemProxyState::Off);
    assert!(!mac.get("Wi-Fi").web.enabled, "not overwritten with the saved work proxy");
    assert!(!mac.get("USB 10/100 LAN").socks.enabled, "the other service is still restored");
}

#[test]
fn a_failure_half_way_puts_back_what_was_changed() {
    let (_dir, paths) = paths();
    let mac = FakeMac::new(&[work_wifi(), service("USB 10/100 LAN")]);
    *mac.fail_on.borrow_mut() = Some(("-setsocksfirewallproxy".into(), "USB 10/100 LAN".into()));
    let state = apply(&paths, &mac, 21080, &[]);
    assert!(matches!(state, SystemProxyState::Failed { action: Action::Set, .. }), "{state:?}");
    *mac.fail_on.borrow_mut() = None;
    assert_eq!(mac.get("Wi-Fi").web, work_wifi().web);
    assert!(!mac.get("USB 10/100 LAN").web.enabled);
    assert!(!has_snapshot(&paths));
}

#[test]
fn a_new_service_and_a_new_port_are_picked_up_without_losing_the_saved_settings() {
    let (_dir, paths) = paths();
    let mac = FakeMac::new(&[work_wifi()]);
    apply(&paths, &mac, 21080, &[]);
    // Ethernet is plugged in, and the next run uses another port.
    mac.services.borrow_mut().push("USB 10/100 LAN".into());
    mac.settings.borrow_mut().insert("USB 10/100 LAN".into(), service("USB 10/100 LAN"));
    let state = apply(&paths, &mac, 21081, &[]);
    assert_eq!(state, SystemProxyState::Set { port: 21081, services: vec!["Wi-Fi".into(), "USB 10/100 LAN".into()] });
    assert_eq!(mac.get("Wi-Fi").web.port, 21081);
    restore(&paths, &mac);
    assert_eq!(mac.get("Wi-Fi").web, work_wifi().web, "the first saved settings, not Kage's own");
    assert!(!mac.get("USB 10/100 LAN").web.enabled);
}

#[test]
fn a_proxy_with_a_login_is_left_alone() {
    let (_dir, paths) = paths();
    let mut locked = service("Wi-Fi");
    locked.web = Proxy { enabled: true, host: "p.example".into(), port: 8080, authenticated: true };
    let mac = FakeMac::new(&[locked.clone(), service("USB 10/100 LAN")]);
    let state = apply(&paths, &mac, 21080, &[]);
    assert_eq!(state, SystemProxyState::Set { port: 21080, services: vec!["USB 10/100 LAN".into()] });
    assert_eq!(mac.get("Wi-Fi"), locked);
}

#[test]
fn nothing_saved_means_nothing_to_restore() {
    let (_dir, paths) = paths();
    let mac = FakeMac::new(&[work_wifi()]);
    assert_eq!(restore(&paths, &mac), SystemProxyState::Off);
    assert_eq!(mac.sets(), 0);
}

#[test]
fn a_failed_restore_keeps_the_saved_settings_for_repair() {
    let (_dir, paths) = paths();
    let mac = FakeMac::new(&[work_wifi()]);
    apply(&paths, &mac, 21080, &[]);
    *mac.fail_on.borrow_mut() = Some(("-setwebproxy".into(), "Wi-Fi".into()));
    assert!(matches!(restore(&paths, &mac), SystemProxyState::Failed { action: Action::Restore, .. }));
    assert!(has_snapshot(&paths));
    *mac.fail_on.borrow_mut() = None;
    assert_eq!(restore(&paths, &mac), SystemProxyState::Off);
    assert_eq!(mac.get("Wi-Fi").web, work_wifi().web);
}

#[test]
fn the_restore_script_quotes_names_and_guards_each_service() {
    let snapshot = Snapshot { port: 21080, services: vec![work_wifi(), service("Bob's Dock")] };
    let script = restore_script(&snapshot);
    assert!(script.starts_with("#!/bin/sh\n"));
    assert!(script.contains("grep -qx 'Port: 21080'"));
    assert!(script.contains("if owned 'Wi-Fi'; then\n  $NS '-setwebproxy' 'Wi-Fi' 'proxy.corp.example' '3128'\n"));
    assert!(script.contains("if owned 'Bob'\\''s Dock'; then"));
    assert!(script.contains("$NS '-setproxybypassdomains' 'Bob'\\''s Dock' 'Empty'"));
    assert!(script.contains("$NS '-setautoproxyurl' 'Wi-Fi' 'http://wpad.corp.example/proxy.pac'"));
    // Every networksetup call is guarded: nothing runs outside an `if owned` block.
    for line in script.lines().filter(|l| l.contains("$NS '-set")) {
        assert!(line.starts_with("  "), "{line}");
    }
}

#[test]
fn it_applies_only_in_proxy_mode_without_a_login() {
    use crate::config::Mode;
    use crate::settings::ReliabilitySettings;
    let on = ReliabilitySettings { system_proxy: true, ..ReliabilitySettings::default() };
    assert!(applies(Mode::Proxy, &on));
    assert!(!applies(Mode::Tun, &on));
    assert!(!applies(Mode::Proxy, &ReliabilitySettings::default()));
    assert!(!applies(Mode::Proxy, &ReliabilitySettings { proxy_auth: true, ..on }));
}

#[test]
fn old_settings_load_with_the_system_proxy_off() {
    let old: crate::settings::ReliabilitySettings = serde_json::from_str(r#"{"autoReconnect":true,"proxyAuth":false}"#).unwrap();
    assert!(!old.system_proxy && old.bypass.is_empty());
}

/// Real macOS, but only the one named service is visible, so nothing else can be touched.
struct OnlyService(&'static str);

impl Runner for OnlyService {
    fn run(&self, args: &[String]) -> crate::error::Result<String> {
        if args[0] == "-listallnetworkservices" {
            return Ok(format!("An asterisk (*) denotes that a network service is disabled.\n{}\n", self.0));
        }
        assert_eq!(args.get(1).map(String::as_str), Some(self.0), "refusing to touch another service: {args:?}");
        NetworkSetup.run(args)
    }
}

/// Changes a real network service on this Mac and puts it back. Run by hand only:
/// `cargo test --lib live_system_proxy_round_trip -- --ignored --nocapture`
#[test]
#[ignore]
fn live_system_proxy_round_trip() {
    const SERVICE: &str = "iPhone USB";
    const PORT: u16 = 21080;
    let runner = OnlyService(SERVICE);
    let (_dir, paths) = paths();
    let before = read_service(&runner, SERVICE).unwrap();
    println!("before: {before:?}");

    let state = apply(&paths, &runner, PORT, &["*.corp.example".into()]);
    assert_eq!(state, SystemProxyState::Set { port: PORT, services: vec![SERVICE.into()] });
    let during = read_service(&runner, SERVICE).unwrap();
    println!("during: {during:?}");
    assert!(still_kage(&during, PORT));
    for p in [&during.web, &during.secure_web, &during.socks] {
        assert_eq!((p.enabled, p.host.as_str(), p.port), (true, "127.0.0.1", PORT));
    }
    assert!(!during.auto_enabled && !during.auto_discovery);
    assert!(during.bypass.iter().any(|b| b == "*.corp.example"));

    // The supervisor's path: run the restore script the app wrote, as the supervisor would.
    let script = paths.system_proxy_script();
    let status = std::process::Command::new("/bin/sh").arg(&script).status().unwrap();
    assert!(status.success());
    let after_script = read_service(&runner, SERVICE).unwrap();
    println!("after the restore script: {after_script:?}");
    assert!(!still_kage(&after_script, PORT), "the script put the service back");

    // And the app's own restore (a no-op now, since the service no longer points at Kage).
    assert_eq!(restore(&paths, &runner), SystemProxyState::Off);
    let after = read_service(&runner, SERVICE).unwrap();
    println!("after: {after:?}");
    assert_eq!(after, before, "every setting is back, including the addresses of proxies that were off");
}
