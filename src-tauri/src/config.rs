use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use url::Url;

use crate::error::{AppError, Result};
use crate::profile::Profile;
use crate::settings::{Action, DnsProvider, DnsSettings, LogLevel, RouteMode, RoutingSettings, Rule, RuleKind, RuleSetKind};

/// The ports the app asks for first. When another program holds one, connecting picks a free one
/// instead (see `net::choose_ports`) and `set_ports` records the choice for everything that talks
/// to sing-box afterwards.
pub const DEFAULT_MIXED_PORT: u16 = 2080;
pub const DEFAULT_CLASH_PORT: u16 = 19090;
static MIXED_PORT: AtomicU16 = AtomicU16::new(DEFAULT_MIXED_PORT);
static CLASH_PORT: AtomicU16 = AtomicU16::new(DEFAULT_CLASH_PORT);

/// The local proxy port of the current (or last) connection.
pub fn mixed_port() -> u16 {
    MIXED_PORT.load(Ordering::SeqCst)
}

/// The sing-box controller port of the current (or last) connection.
pub fn clash_port() -> u16 {
    CLASH_PORT.load(Ordering::SeqCst)
}

/// User name of the optional local proxy login; the password is a setting.
pub const PROXY_USER: &str = "kage";
static PROXY_PASSWORD: Mutex<Option<String>> = Mutex::new(None);

/// Sets the local proxy password of the current connection (`None` = open proxy).
pub fn set_proxy_password(password: Option<String>) {
    *PROXY_PASSWORD.lock().unwrap() = password;
}

/// The local proxy address, with the login when one is set. Every in-app request through the
/// tunnel must use this, or a protected proxy would refuse it.
pub fn proxy_url() -> String {
    match PROXY_PASSWORD.lock().unwrap().as_deref() {
        Some(password) => format!("socks5h://{PROXY_USER}:{password}@127.0.0.1:{}", mixed_port()),
        None => format!("socks5h://127.0.0.1:{}", mixed_port()),
    }
}

pub fn set_ports(mixed: u16, clash: u16) {
    MIXED_PORT.store(mixed, Ordering::SeqCst);
    CLASH_PORT.store(clash, Ordering::SeqCst);
}
const TUN_ADDRESS_V4: &str = "172.19.0.1/30";
const TUN_ADDRESS_V6: &str = "fdfe:dcba:9876::1/126";
const MAX_NODES: usize = 150;
use crate::app_config::urls::CONNECTION_PROBE as PROBE_URL;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Tun,
    Proxy,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Tun => "tun",
            Mode::Proxy => "proxy",
        }
    }
}

pub struct ConfigInput<'a> {
    pub profiles: &'a [Profile],
    pub active_id: Option<&'a str>,
    pub auto: bool,
    pub mode: Mode,
    pub routing: &'a RoutingSettings,
    pub log_path: &'a Path,
    pub log_level: LogLevel,
    pub mixed_port: u16,
    pub clash_port: u16,
    /// Where the country and category lists are kept (see `rulesets`).
    pub rule_sets_dir: &'a Path,
    pub secret: &'a str,
    /// Password for the local proxy; `None` leaves it open.
    pub proxy_password: Option<&'a str>,
}

pub fn node_tag(profile_id: &str) -> String {
    format!("node-{profile_id}")
}

pub fn build(input: &ConfigInput) -> Result<Value> {
    let nodes = select_nodes(input);
    if nodes.is_empty() {
        return Err(AppError::Invalid("add a profile first".into()));
    }
    let tags: Vec<String> = nodes.iter().map(|p| node_tag(&p.id)).collect();
    let default_tag = input
        .active_id
        .filter(|id| nodes.iter().any(|p| p.id == *id))
        .map(node_tag)
        .unwrap_or_else(|| tags[0].clone());

    let mut outbounds: Vec<Value> = nodes.iter().map(|p| p.outbound(&node_tag(&p.id))).collect();
    outbounds.push(if input.auto {
        json!({
            "type": "urltest", "tag": "proxy", "outbounds": tags,
            "url": PROBE_URL, "interval": "3m", "tolerance": 50,
        })
    } else {
        json!({ "type": "selector", "tag": "proxy", "outbounds": tags, "default": default_tag })
    });
    outbounds.push(json!({ "type": "direct", "tag": "direct" }));

    let mut mixed = json!({
        "type": "mixed", "tag": "mixed-in", "listen": "127.0.0.1", "listen_port": input.mixed_port,
    });
    if let Some(password) = input.proxy_password {
        mixed["users"] = json!([{ "username": PROXY_USER, "password": password }]);
    }
    let mut inbounds = vec![mixed];
    if input.mode == Mode::Tun {
        // sing-box only routes an address family into the tunnel when the interface has an address
        // of that family. With IPv4 alone, IPv6 traffic would skip the VPN on any network that has
        // IPv6 and show the user's real address.
        inbounds.push(json!({
            "type": "tun", "tag": "tun-in", "address": [TUN_ADDRESS_V4, TUN_ADDRESS_V6], "auto_route": true,
        }));
    }

    // Lists are local files, so a missing one is left out rather than failing the connection.
    let rule_sets = available_rule_sets(input.routing, input.rule_sets_dir);
    let mut route = json!({
        "auto_detect_interface": true,
        "find_process": true,
        "default_domain_resolver": "local",
        "rules": route_rules(input.routing, &rule_sets),
        "final": final_outbound(input.routing),
    });
    if !rule_sets.is_empty() {
        route["rule_set"] = Value::Array(rule_set_definitions(&rule_sets));
    }

    let experimental = json!({
        "clash_api": {
            "external_controller": format!("127.0.0.1:{}", input.clash_port),
            "secret": input.secret,
        },
    });

    Ok(json!({
        "log": { "level": input.log_level.as_str(), "timestamp": true, "output": input.log_path },
        "dns": dns(input.routing, input.mode),
        "experimental": experimental,
        "inbounds": inbounds,
        "outbounds": outbounds,
        "route": route,
    }))
}

fn select_nodes<'a>(input: &ConfigInput<'a>) -> Vec<&'a Profile> {
    let mut nodes: Vec<&Profile> = input.profiles.iter().take(MAX_NODES).collect();
    if let Some(active) = input.active_id.and_then(|id| input.profiles.iter().find(|p| p.id == id)) {
        if !nodes.iter().any(|p| p.id == active.id) {
            nodes.insert(0, active);
        }
    }
    nodes
}

fn route_rules(routing: &RoutingSettings, available: &[(String, PathBuf)]) -> Vec<Value> {
    let mut rules = vec![json!({ "action": "sniff" }), json!({ "protocol": "dns", "action": "hijack-dns" })];
    if !routing.ipv6 {
        rules.push(json!({ "ip_version": 6, "action": "reject" }));
    }
    if routing.bypass_lan {
        rules.push(json!({ "ip_is_private": true, "outbound": "direct" }));
    }
    if routing.mode == RouteMode::Rule {
        rules.extend(routing.rules.iter().filter_map(rule_json));
        rules.extend(routing.rule_sets.iter().filter(|s| s.enabled).filter_map(|set| {
            let tag = rule_set_tag(set.kind, &set.country)?;
            if !available.iter().any(|(known, _)| *known == tag) {
                return None;
            }
            let mut rule = json!({ "rule_set": [tag] });
            apply_action(&mut rule, set.action);
            Some(rule)
        }));
    }
    rules
}

fn final_outbound(routing: &RoutingSettings) -> &'static str {
    match routing.mode {
        RouteMode::Global => "proxy",
        RouteMode::Direct => "direct",
        RouteMode::Rule => if routing.final_action == Action::Direct { "direct" } else { "proxy" },
    }
}

fn rule_json(rule: &Rule) -> Option<Value> {
    let value = rule.value.trim();
    if !rule.enabled || value.is_empty() {
        return None;
    }
    let mut json = match rule.kind {
        RuleKind::Domain => json!({ "domain": [value] }),
        RuleKind::DomainSuffix => json!({ "domain_suffix": [value] }),
        RuleKind::DomainKeyword => json!({ "domain_keyword": [value] }),
        RuleKind::IpCidr => json!({ "ip_cidr": [value] }),
        RuleKind::ProcessName => json!({ "process_name": [value] }),
        RuleKind::App => json!({ "process_path_regex": [app_path_pattern(value)] }),
    };
    apply_action(&mut json, rule.action);
    Some(json)
}

/// A pattern for every process inside an app bundle. Browsers and chat apps send most of their
/// traffic from helper processes deeper in the bundle, so matching the main executable's name
/// alone would leave that traffic on the default route.
pub(crate) fn app_path_pattern(bundle: &str) -> String {
    format!("^{}/", quote_meta(bundle.trim_end_matches('/')))
}

/// Escapes text for sing-box's regular expressions (Go's RE2 syntax), like Go's `regexp.QuoteMeta`.
fn quote_meta(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len());
    for c in text.chars() {
        if r"\.+*?()|[]{}^$".contains(c) {
            quoted.push('\\');
        }
        quoted.push(c);
    }
    quoted
}

fn apply_action(rule: &mut Value, action: Action) {
    match action {
        Action::Proxy => rule["outbound"] = json!("proxy"),
        Action::Direct => rule["outbound"] = json!("direct"),
        Action::Block => rule["action"] = json!("reject"),
    }
}

pub(crate) fn rule_set_tag(kind: RuleSetKind, country: &str) -> Option<String> {
    let country = country.trim().to_lowercase();
    let valid = !country.is_empty() && country.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    valid.then(|| match kind {
        RuleSetKind::Geoip => format!("geoip-{country}"),
        RuleSetKind::Geosite => format!("geosite-{country}"),
    })
}

/// The enabled lists whose file is on disk, as `(tag, file)`, each once.
fn available_rule_sets(routing: &RoutingSettings, dir: &Path) -> Vec<(String, PathBuf)> {
    crate::rulesets::wanted(routing)
        .into_iter()
        .map(|item| {
            let path = item.path(dir);
            (item.tag, path)
        })
        .filter(|(_, path)| path.is_file())
        .collect()
}

fn rule_set_definitions(available: &[(String, PathBuf)]) -> Vec<Value> {
    available
        .iter()
        .map(|(tag, path)| json!({ "type": "local", "tag": tag, "format": "binary", "path": path }))
        .collect()
}

const FAKE_IP_V4: &str = "198.18.0.0/15";
const FAKE_IP_V6: &str = "fc00::/18";

fn doh(server: &str) -> Value {
    json!({ "type": "https", "tag": "remote", "server": server, "detour": "proxy" })
}

/// The DNS server for what a person typed in Settings: a bare host (read as DNS over HTTPS) or a
/// link such as `https://dns.nextdns.io/abc123`, `tls://dns.google`, `quic://dns.adguard-dns.com`,
/// `udp://8.8.8.8` or `tcp://8.8.8.8`, with an optional port. A DoH path is kept, because personal
/// endpoints (NextDNS, AdGuard) carry the account in it and without it the wrong resolver answers.
pub fn custom_dns_server(text: &str) -> Option<Value> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let url = Url::parse(&if text.contains("://") { text.to_string() } else { format!("https://{text}") }).ok()?;
    let kind = match url.scheme() {
        "https" => "https",
        "tls" => "tls",
        "quic" => "quic",
        "udp" => "udp",
        "tcp" => "tcp",
        _ => return None,
    };
    let host = url.host_str()?.trim_matches(['[', ']']);
    if host.is_empty() {
        return None;
    }
    let mut server = doh(host);
    server["type"] = json!(kind);
    if let Some(port) = url.port() {
        server["server_port"] = json!(port);
    }
    if kind == "https" && url.path() != "/" {
        server["path"] = json!(url.path());
    }
    Some(server)
}

/// Whether a rule names this app or the engine. Blocking either would cut the app's own health
/// checks, downloads and local proxy traffic, so it is refused rather than saved.
fn names_the_app_itself(rule: &Rule) -> bool {
    let value = rule.value.trim().trim_end_matches('/');
    let name = value.rsplit('/').next().unwrap_or(value);
    let name = name.strip_suffix(".app").unwrap_or(name).to_lowercase();
    let own = [crate::app_config::APP_NAME.to_lowercase(), "kage vpn".into(), "kage-vpn".into(), "kage".into(), "sing-box".into()];
    match rule.kind {
        RuleKind::ProcessName => own.contains(&name),
        RuleKind::App => {
            own.contains(&name)
                || std::env::current_exe().ok().and_then(|exe| crate::apps::bundle_of(&exe.to_string_lossy()).map(str::to_string)).is_some_and(|bundle| bundle == value)
        }
        _ => false,
    }
}

/// Refuses an enabled block rule for this app or the engine.
pub fn validate_rules(rules: &[Rule]) -> Result<()> {
    if let Some(rule) = rules.iter().find(|r| r.enabled && r.action == Action::Block && names_the_app_itself(r)) {
        let label = rule.value.rsplit('/').next().unwrap_or(&rule.value).trim_end_matches(".app");
        return Err(AppError::Invalid(format!(
            "{label} cannot be blocked: this app and its engine need the network for the VPN itself, health checks and downloads"
        )));
    }
    Ok(())
}

/// Refuses a custom DNS entry that would silently turn into "use the system resolver".
pub fn validate_dns(settings: &DnsSettings) -> Result<()> {
    if settings.provider == DnsProvider::Custom && custom_dns_server(&settings.custom).is_none() {
        return Err(AppError::Invalid(
            "that DNS server is not valid. Use a host such as dns.example.com, or a link such as https://dns.example.com/your-id or tls://dns.example.com".into(),
        ));
    }
    Ok(())
}

fn dns(routing: &RoutingSettings, mode: Mode) -> Value {
    let mut config = dns_servers(&routing.dns);
    if routing.dns.fake_ip && mode == Mode::Tun && routing.mode != RouteMode::Direct {
        add_fake_ip(&mut config, routing.ipv6);
    }
    if !routing.ipv6 {
        config["strategy"] = json!("ipv4_only");
    }
    config
}

/// Answers address lookups with made-up addresses that the tunnel maps back to the domain, so the
/// route and the server see the name and not an address resolved on this Mac. Local names keep
/// their real answers, and the server's own host is resolved through `default_domain_resolver`.
fn add_fake_ip(config: &mut Value, ipv6: bool) {
    let mut server = json!({ "type": "fakeip", "tag": "fakeip", "inet4_range": FAKE_IP_V4 });
    if ipv6 {
        server["inet6_range"] = json!(FAKE_IP_V6);
    }
    if let Some(servers) = config["servers"].as_array_mut() {
        servers.push(server);
    }
    config["rules"] = json!([
        { "domain_suffix": ["local", "lan", "localdomain", "home.arpa"], "server": "local" },
        { "query_type": ["A", "AAAA"], "server": "fakeip" },
    ]);
}

fn dns_servers(settings: &DnsSettings) -> Value {
    let remote = match settings.provider {
        DnsProvider::Cloudflare => Some(doh("1.1.1.1")),
        DnsProvider::Google => Some(doh("8.8.8.8")),
        DnsProvider::Quad9 => Some(doh("9.9.9.9")),
        DnsProvider::Custom => custom_dns_server(&settings.custom),
        DnsProvider::System => None,
    };
    match remote {
        Some(server) => json!({
            "servers": [server, { "type": "local", "tag": "local" }],
            "final": "remote",
        }),
        None => json!({ "servers": [{ "type": "local", "tag": "local" }], "final": "local" }),
    }
}

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;
