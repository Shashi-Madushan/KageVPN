use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{AppError, Result};
use crate::profile::{Profile, Security, TransportKind};
use crate::settings::{Action, DnsProvider, DnsSettings, RouteMode, RoutingSettings, Rule, RuleKind, RuleSetKind};

pub const MIXED_PORT: u16 = 2080;
pub const CLASH_PORT: u16 = 19090;
const MAX_NODES: usize = 150;
const PROBE_URL: &str = "http://cp.cloudflare.com/generate_204";
const GEOIP_URL: &str = "https://raw.githubusercontent.com/SagerNet/sing-geoip/rule-set/geoip-";
const GEOSITE_URL: &str = "https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-";

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
    pub cache_path: &'a Path,
    pub secret: &'a str,
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

    let mut outbounds: Vec<Value> = nodes.iter().map(|p| outbound(p)).collect();
    outbounds.push(if input.auto {
        json!({
            "type": "urltest", "tag": "proxy", "outbounds": tags,
            "url": PROBE_URL, "interval": "3m", "tolerance": 50,
        })
    } else {
        json!({ "type": "selector", "tag": "proxy", "outbounds": tags, "default": default_tag })
    });
    outbounds.push(json!({ "type": "direct", "tag": "direct" }));

    let mut inbounds = vec![json!({
        "type": "mixed", "tag": "mixed-in", "listen": "127.0.0.1", "listen_port": MIXED_PORT,
    })];
    if input.mode == Mode::Tun {
        inbounds.push(json!({
            "type": "tun", "tag": "tun-in", "address": ["172.19.0.1/30"], "auto_route": true,
        }));
    }

    let mut route = json!({
        "auto_detect_interface": true,
        "find_process": true,
        "default_domain_resolver": "local",
        "rules": route_rules(input.routing),
        "final": final_outbound(input.routing),
    });
    let rule_sets = rule_set_definitions(input.routing);
    let uses_rule_sets = !rule_sets.is_empty();
    if uses_rule_sets {
        route["rule_set"] = Value::Array(rule_sets);
    }

    let mut experimental = json!({
        "clash_api": {
            "external_controller": format!("127.0.0.1:{CLASH_PORT}"),
            "secret": input.secret,
        },
    });
    if uses_rule_sets {
        experimental["cache_file"] = json!({ "enabled": true, "path": input.cache_path });
    }

    Ok(json!({
        "log": { "level": "info", "timestamp": true, "output": input.log_path },
        "dns": dns(&input.routing.dns),
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

fn outbound(profile: &Profile) -> Value {
    let mut tls = json!({
        "enabled": true,
        "server_name": profile.sni.as_deref().unwrap_or(&profile.server),
    });
    if !profile.alpn.is_empty() {
        tls["alpn"] = json!(profile.alpn);
    }
    if let Some(fingerprint) = &profile.fingerprint {
        tls["utls"] = json!({ "enabled": true, "fingerprint": fingerprint });
    }
    match (profile.security, &profile.reality) {
        (Security::Reality, Some(reality)) => {
            tls["reality"] = json!({
                "enabled": true,
                "public_key": reality.public_key,
                "short_id": reality.short_id,
            });
        }
        _ => {
            if let Some(key) = &profile.pinned_key {
                tls["certificate_public_key_sha256"] = json!([key]);
            }
        }
    }

    let mut outbound = json!({
        "type": "vless",
        "tag": node_tag(&profile.id),
        "server": profile.server,
        "server_port": profile.port,
        "uuid": profile.uuid,
        "tls": tls,
    });
    if let Some(flow) = profile.flow.as_deref().filter(|f| !f.is_empty()) {
        outbound["flow"] = json!(flow);
    }
    if let Some(encoding) = profile.packet_encoding.as_deref().filter(|e| !e.is_empty()) {
        outbound["packet_encoding"] = json!(encoding);
    }
    if let Some(transport) = transport(profile) {
        outbound["transport"] = transport;
    }
    outbound
}

fn transport(profile: &Profile) -> Option<Value> {
    let t = &profile.transport;
    let path = t.path.as_deref().filter(|p| !p.is_empty());
    let host = t.host.as_deref().filter(|h| !h.is_empty());
    match t.kind {
        TransportKind::Tcp => None,
        TransportKind::Ws => {
            let mut value = json!({ "type": "ws", "path": path.unwrap_or("/") });
            if let Some(host) = host {
                value["headers"] = json!({ "Host": host });
            }
            Some(value)
        }
        TransportKind::Grpc => Some(json!({
            "type": "grpc",
            "service_name": t.service_name.as_deref().unwrap_or_default(),
        })),
        TransportKind::Httpupgrade => {
            let mut value = json!({ "type": "httpupgrade", "path": path.unwrap_or("/") });
            if let Some(host) = host {
                value["host"] = json!(host);
            }
            Some(value)
        }
    }
}

fn route_rules(routing: &RoutingSettings) -> Vec<Value> {
    let mut rules = vec![json!({ "action": "sniff" }), json!({ "protocol": "dns", "action": "hijack-dns" })];
    if routing.bypass_lan {
        rules.push(json!({ "ip_is_private": true, "outbound": "direct" }));
    }
    if routing.mode == RouteMode::Rule {
        rules.extend(routing.rules.iter().filter_map(rule_json));
        rules.extend(routing.rule_sets.iter().filter(|s| s.enabled).filter_map(|set| {
            let tag = rule_set_tag(set.kind, &set.country)?;
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
    let key = match rule.kind {
        RuleKind::Domain => "domain",
        RuleKind::DomainSuffix => "domain_suffix",
        RuleKind::DomainKeyword => "domain_keyword",
        RuleKind::IpCidr => "ip_cidr",
        RuleKind::ProcessName => "process_name",
    };
    let mut json = json!({ key: [value] });
    apply_action(&mut json, rule.action);
    Some(json)
}

fn apply_action(rule: &mut Value, action: Action) {
    match action {
        Action::Proxy => rule["outbound"] = json!("proxy"),
        Action::Direct => rule["outbound"] = json!("direct"),
        Action::Block => rule["action"] = json!("reject"),
    }
}

fn rule_set_tag(kind: RuleSetKind, country: &str) -> Option<String> {
    let country = country.trim().to_lowercase();
    let valid = !country.is_empty() && country.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    valid.then(|| match kind {
        RuleSetKind::Geoip => format!("geoip-{country}"),
        RuleSetKind::Geosite => format!("geosite-{country}"),
    })
}

fn rule_set_definitions(routing: &RoutingSettings) -> Vec<Value> {
    if routing.mode != RouteMode::Rule {
        return Vec::new();
    }
    routing
        .rule_sets
        .iter()
        .filter(|s| s.enabled)
        .filter_map(|set| {
            let tag = rule_set_tag(set.kind, &set.country)?;
            let base = if set.kind == RuleSetKind::Geoip { GEOIP_URL } else { GEOSITE_URL };
            let name = tag.split_once('-').map(|(_, rest)| rest).unwrap_or_default();
            Some(json!({
                "type": "remote", "tag": tag, "format": "binary",
                "url": format!("{base}{name}.srs"), "download_detour": "direct",
            }))
        })
        .collect()
}

fn dns(settings: &DnsSettings) -> Value {
    let remote = match settings.provider {
        DnsProvider::Cloudflare => Some("1.1.1.1".to_string()),
        DnsProvider::Google => Some("8.8.8.8".to_string()),
        DnsProvider::Quad9 => Some("9.9.9.9".to_string()),
        DnsProvider::Custom => {
            let host = settings.custom.trim().trim_start_matches("https://");
            let host = host.split('/').next().unwrap_or_default();
            (!host.is_empty()).then(|| host.to_string())
        }
        DnsProvider::System => None,
    };
    match remote {
        Some(server) => json!({
            "servers": [
                { "type": "https", "tag": "remote", "server": server, "detour": "proxy" },
                { "type": "local", "tag": "local" },
            ],
            "final": "remote",
        }),
        None => json!({ "servers": [{ "type": "local", "tag": "local" }], "final": "local" }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PIN: &str = "4AJ7t5dYu7LHkpjhv6iBfFJQohf2f2+8QvpDJ3S8nBI=";

    fn profile() -> Profile {
        Profile {
            id: "1".into(),
            name: "test".into(),
            server: "example.com".into(),
            port: 443,
            uuid: "11111111-2222-4333-8444-555555555555".into(),
            sni: Some("aka.ms".into()),
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
}
