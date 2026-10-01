use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::profile::Profile;

pub const MIXED_PORT: u16 = 2080;
pub const CLASH_PORT: u16 = 19090;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Tun,
    Proxy,
}

pub fn build(profile: &Profile, mode: Mode, log_path: &Path, api_secret: &str) -> Value {
    let mut inbounds = vec![json!({
        "type": "mixed",
        "tag": "mixed-in",
        "listen": "127.0.0.1",
        "listen_port": MIXED_PORT,
    })];
    if mode == Mode::Tun {
        inbounds.push(json!({
            "type": "tun",
            "tag": "tun-in",
            "address": ["172.19.0.1/30"],
            "auto_route": true,
        }));
    }

    json!({
        "log": {
            "level": "info",
            "timestamp": true,
            "output": log_path,
        },
        "dns": {
            "servers": [
                { "type": "https", "tag": "remote", "server": "1.1.1.1", "detour": "proxy" },
                { "type": "local", "tag": "local" },
            ],
            "final": "remote",
        },
        "experimental": {
            "clash_api": {
                "external_controller": format!("127.0.0.1:{CLASH_PORT}"),
                "secret": api_secret,
            },
        },
        "inbounds": inbounds,
        "outbounds": [outbound(profile), { "type": "direct", "tag": "direct" }],
        "route": {
            "auto_detect_interface": true,
            "default_domain_resolver": "local",
            "rules": [
                { "action": "sniff" },
                { "protocol": "dns", "action": "hijack-dns" },
                { "ip_is_private": true, "outbound": "direct" },
            ],
            "final": "proxy",
        },
    })
}

fn outbound(profile: &Profile) -> Value {
    let mut tls = json!({
        "enabled": true,
        "server_name": profile.sni.as_deref().unwrap_or(&profile.server),
        "alpn": profile.alpn,
    });
    if let Some(fingerprint) = &profile.fingerprint {
        tls["utls"] = json!({ "enabled": true, "fingerprint": fingerprint });
    }
    if let Some(key) = &profile.pinned_key {
        tls["certificate_public_key_sha256"] = json!([key]);
    }

    let mut outbound = json!({
        "type": "vless",
        "tag": "proxy",
        "server": profile.server,
        "server_port": profile.port,
        "uuid": profile.uuid,
        "tls": tls,
    });
    if let Some(flow) = &profile.flow {
        outbound["flow"] = json!(flow);
    }
    outbound
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
            flow: None,
            pinned_key: Some(PIN.into()),
        }
    }

    #[test]
    fn tun_mode_adds_tun_inbound() {
        let cfg = build(&profile(), Mode::Tun, Path::new("/tmp/log"), "secret");
        assert_eq!(cfg["inbounds"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn proxy_mode_has_only_mixed_inbound() {
        let cfg = build(&profile(), Mode::Proxy, Path::new("/tmp/log"), "secret");
        assert_eq!(cfg["inbounds"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn outbound_carries_tls_settings() {
        let cfg = build(&profile(), Mode::Proxy, Path::new("/tmp/log"), "secret");
        let tls = &cfg["outbounds"][0]["tls"];
        assert_eq!(tls["server_name"], "aka.ms");
        assert_eq!(tls["certificate_public_key_sha256"][0], PIN);
    }

    #[test]
    fn generated_configs_pass_singbox_check() {
        let Ok(binary) = crate::singbox::locate() else { return };
        for mode in [Mode::Tun, Mode::Proxy] {
            let path = std::env::temp_dir().join(format!("singbox-gui-test-{mode:?}.json"));
            let cfg = build(&profile(), mode, Path::new("/tmp/singbox-gui-test.log"), "secret");
            std::fs::write(&path, serde_json::to_vec(&cfg).unwrap()).unwrap();
            crate::singbox::check(&binary, &path).unwrap();
        }
    }
}
