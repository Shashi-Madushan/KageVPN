//! Server lists written as configuration files instead of share links: sing-box JSON (its
//! `outbounds`) and Clash or Clash Meta YAML (its `proxies`). Providers hand these out as
//! subscriptions and people drop them into the import box. Each server is rewritten as the share
//! link its protocol already reads, so parsing, checks and defaults stay in one place per protocol.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use serde_json::{json, Map, Value};

use super::common::format_host;

/// Everything but the characters a URL never needs to escape, so a UUID stays readable.
const USERINFO: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'.').remove(b'_').remove(b'~');

/// One entry of a configuration file, in file order: a server as `(name, share link)`, or why it
/// was left out.
pub type Found = std::result::Result<(String, String), String>;

/// The servers in `text` when it is a sing-box or Clash configuration file, `None` otherwise.
pub fn extract(text: &str) -> Option<Vec<Found>> {
    let text = text.trim_start_matches('\u{feff}').trim();
    if text.starts_with('{') {
        let value: Value = serde_json::from_str(text).ok()?;
        // WireGuard sits in `endpoints` since sing-box 1.11.
        let list = |key: &str| value.get(key).and_then(Value::as_array).cloned();
        let (outbounds, endpoints) = (list("outbounds"), list("endpoints"));
        if outbounds.is_none() && endpoints.is_none() {
            return None;
        }
        let entries = outbounds.into_iter().flatten().chain(endpoints.into_iter().flatten());
        return Some(collect(entries, singbox));
    }
    if text.lines().any(|line| line.starts_with("proxies:")) {
        let mut document: serde_yaml_ng::Value = serde_yaml_ng::from_str(text).ok()?;
        // Clash files often share settings between servers with `<<: *anchor`.
        document.apply_merge().ok()?;
        let proxies = document.get("proxies")?.as_sequence()?;
        // A server whose fields do not map onto JSON (a key that is not text) is skipped by `collect`.
        let entries = proxies.iter().map(|proxy| serde_json::to_value(proxy).unwrap_or(Value::Null));
        return Some(collect(entries, clash));
    }
    None
}

enum Entry {
    Link(String, String),
    Skip(String),
    /// Not a server: a group, a direct or block route, a DNS entry.
    Ignore,
}

fn collect(entries: impl Iterator<Item = Value>, convert: fn(&Map<String, Value>) -> Entry) -> Vec<Found> {
    entries
        .filter_map(|entry| match entry.as_object().map(convert) {
            None => Some(Err("an entry is not a server".into())),
            Some(Entry::Link(name, link)) => Some(Ok((name, link))),
            Some(Entry::Skip(reason)) => Some(Err(reason)),
            Some(Entry::Ignore) => None,
        })
        .collect()
}

// ── Reading fields ──────────────────────────────────────────────────────────

fn text(object: &Map<String, Value>, key: &str) -> Option<String> {
    let value = object.get(key)?;
    let text = match value {
        Value::String(s) => s.trim().to_string(),
        Value::Number(n) => n.to_string(),
        _ => return None,
    };
    (!text.is_empty()).then_some(text)
}

fn flag(object: &Map<String, Value>, key: &str) -> bool {
    match object.get(key) {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => matches!(s.trim(), "true" | "1"),
        Some(Value::Number(n)) => n.as_u64() == Some(1),
        _ => false,
    }
}

fn object<'a>(object: &'a Map<String, Value>, key: &str) -> Option<&'a Map<String, Value>> {
    object.get(key)?.as_object()
}

/// A list of text, written either as a list or as one comma-separated value.
fn list(object: &Map<String, Value>, key: &str) -> Vec<String> {
    match object.get(key) {
        Some(Value::Array(items)) => items.iter().filter_map(|v| v.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        Some(Value::String(s)) => s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        _ => Vec::new(),
    }
}

fn port(object: &Map<String, Value>, key: &str) -> Option<u16> {
    text(object, key)?.parse::<u16>().ok().filter(|p| *p != 0)
}

// ── Writing links ───────────────────────────────────────────────────────────

/// The parts every server shares, read once.
struct Server {
    name: String,
    host: String,
    port: u16,
}

impl Server {
    fn read(object: &Map<String, Value>, name_key: &str, port_key: &str) -> Result<Self, String> {
        let name = text(object, name_key).unwrap_or_default();
        let label = if name.is_empty() { "a server".to_string() } else { name.clone() };
        let host = text(object, "server").ok_or_else(|| format!("{label}: no server address"))?;
        let port = port(object, port_key).ok_or_else(|| format!("{label}: no valid port"))?;
        Ok(Self { name, host, port })
    }

    fn label(&self) -> &str {
        if self.name.is_empty() { &self.host } else { &self.name }
    }

    fn link(&self, scheme: &str, userinfo: &str, query: Query) -> Entry {
        self.link_encoded(scheme, &utf8_percent_encode(userinfo, USERINFO).to_string(), query)
    }

    /// `link` for a userinfo whose parts are already escaped, such as `user:password`.
    fn link_encoded(&self, scheme: &str, userinfo: &str, mut query: Query) -> Entry {
        let query = query.finish();
        let query = if query.is_empty() { String::new() } else { format!("?{query}") };
        let fragment = if self.name.is_empty() { String::new() } else { format!("#{}", utf8_percent_encode(&self.name, NON_ALPHANUMERIC)) };
        Entry::Link(self.label().to_string(), format!("{scheme}://{userinfo}@{}:{}{query}{fragment}", format_host(&self.host), self.port))
    }

    fn skip(&self, why: &str) -> Entry {
        Entry::Skip(format!("{}: {why}", self.label()))
    }
}

type Query = url::form_urlencoded::Serializer<'static, String>;

fn query() -> Query {
    url::form_urlencoded::Serializer::new(String::new())
}

fn put(query: &mut Query, key: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|v| !v.is_empty()) {
        query.append_pair(key, value);
    }
}

/// TLS settings shared by the TLS protocols, in the form both config formats can be read into.
#[derive(Default)]
struct Tls {
    enabled: bool,
    sni: Option<String>,
    insecure: bool,
    alpn: Vec<String>,
    fingerprint: Option<String>,
    reality: Option<(String, String)>,
}

impl Tls {
    fn security(&self) -> Option<&'static str> {
        match (self.enabled, &self.reality) {
            (_, Some(_)) => Some("reality"),
            (true, None) => Some("tls"),
            (false, None) => None,
        }
    }

    fn write(&self, query: &mut Query) {
        put(query, "sni", self.sni.as_deref());
        put(query, "fp", self.fingerprint.as_deref());
        if !self.alpn.is_empty() {
            query.append_pair("alpn", &self.alpn.join(","));
        }
        if self.insecure {
            query.append_pair("allowInsecure", "1");
        }
        if let Some((key, id)) = &self.reality {
            query.append_pair("pbk", key);
            put(query, "sid", Some(id));
        }
    }
}

/// A transport in the link vocabulary: `type`, `path`, `host`, `serviceName`.
#[derive(Default)]
struct Transport {
    kind: &'static str,
    path: Option<String>,
    host: Option<String>,
    service_name: Option<String>,
}

impl Transport {
    fn write(&self, query: &mut Query) {
        query.append_pair("type", if self.kind.is_empty() { "tcp" } else { self.kind });
        put(query, "path", self.path.as_deref());
        put(query, "host", self.host.as_deref());
        put(query, "serviceName", self.service_name.as_deref());
    }
}

/// The v2rayN `vmess://` form: base64 of a JSON description.
fn vmess_link(server: &Server, uuid: &str, alter_id: &str, cipher: &str, tls: &Tls, transport: &Transport) -> Entry {
    let path = if transport.kind == "grpc" { &transport.service_name } else { &transport.path };
    let description = json!({
        "v": "2",
        "ps": server.name,
        "add": server.host,
        "port": server.port.to_string(),
        "id": uuid,
        "aid": alter_id,
        "scy": cipher,
        "net": if transport.kind.is_empty() { "tcp" } else { transport.kind },
        "type": "none",
        "host": transport.host.clone().unwrap_or_default(),
        "path": path.clone().unwrap_or_default(),
        "tls": if tls.enabled { "tls" } else { "" },
        "sni": tls.sni.clone().unwrap_or_default(),
        "alpn": tls.alpn.join(","),
        "fp": tls.fingerprint.clone().unwrap_or_default(),
    });
    Entry::Link(server.label().to_string(), format!("vmess://{}", STANDARD.encode(description.to_string())))
}

// ── sing-box ────────────────────────────────────────────────────────────────

fn singbox(outbound: &Map<String, Value>) -> Entry {
    let Some(kind) = text(outbound, "type") else {
        // Xray writes `protocol` where sing-box writes `type`.
        return if outbound.contains_key("protocol") { Entry::Skip("Xray configuration files are not supported yet".into()) } else { Entry::Ignore };
    };
    if matches!(kind.as_str(), "selector" | "urltest" | "direct" | "block" | "dns") {
        return Entry::Ignore;
    }
    if kind == "wireguard" {
        return singbox_wireguard(outbound);
    }
    // Name the type before the address check: an endpoint such as Tailscale has no server field.
    if !matches!(kind.as_str(), "vless" | "vmess" | "trojan" | "shadowsocks" | "hysteria2" | "hysteria" | "tuic" | "ssh") {
        return Entry::Skip(format!("{}: {kind} servers are not supported yet", text(outbound, "tag").unwrap_or_default()));
    }
    let server = match Server::read(outbound, "tag", "server_port") {
        Ok(server) => server,
        Err(_) if matches!(kind.as_str(), "hysteria2" | "hysteria") && outbound.contains_key("server_ports") => {
            let label = if kind == "hysteria" { "Hysteria" } else { "Hysteria2" };
            return Entry::Skip(format!("{}: {label} port hopping is not supported, use a single port", text(outbound, "tag").unwrap_or_default()));
        }
        Err(reason) => return Entry::Skip(reason),
    };
    if outbound.contains_key("detour") {
        return server.skip("servers that connect through another server are not supported");
    }
    let tls = singbox_tls(object(outbound, "tls"));
    let transport = match singbox_transport(object(outbound, "transport")) {
        Ok(transport) => transport,
        Err(why) => return server.skip(&why),
    };
    match kind.as_str() {
        "vless" => {
            let mut q = query();
            q.append_pair("encryption", "none");
            put(&mut q, "security", tls.security());
            put(&mut q, "flow", text(outbound, "flow").as_deref());
            put(&mut q, "packetEncoding", text(outbound, "packet_encoding").as_deref());
            tls.write(&mut q);
            transport.write(&mut q);
            server.link("vless", &text(outbound, "uuid").unwrap_or_default(), q)
        }
        "vmess" => {
            let alter_id = text(outbound, "alter_id").unwrap_or_else(|| "0".into());
            let cipher = text(outbound, "security").unwrap_or_else(|| "auto".into());
            vmess_link(&server, &text(outbound, "uuid").unwrap_or_default(), &alter_id, &cipher, &tls, &transport)
        }
        "trojan" => {
            if !tls.enabled {
                return server.skip("Trojan without TLS is not supported");
            }
            let mut q = query();
            q.append_pair("security", "tls");
            tls.write(&mut q);
            transport.write(&mut q);
            server.link("trojan", &text(outbound, "password").unwrap_or_default(), q)
        }
        "shadowsocks" => {
            if text(outbound, "plugin").is_some() {
                return server.skip("Shadowsocks plugins are not supported");
            }
            let method = text(outbound, "method").unwrap_or_default();
            let password = text(outbound, "password").unwrap_or_default();
            shadowsocks_link(&server, &method, &password)
        }
        "hysteria2" => {
            let obfs = object(outbound, "obfs").map(|o| (text(o, "type").unwrap_or_default(), text(o, "password")));
            hysteria2_link(&server, &text(outbound, "password").unwrap_or_default(), &tls, obfs)
        }
        "hysteria" => {
            if text(outbound, "auth").is_some() && text(outbound, "auth_str").is_none() {
                return server.skip("Hysteria with a binary (base64) auth key is not supported, use auth_str");
            }
            hysteria_link(
                &server,
                &Hysteria {
                    auth: text(outbound, "auth_str"),
                    up: text(outbound, "up_mbps").or_else(|| text(outbound, "up")),
                    down: text(outbound, "down_mbps").or_else(|| text(outbound, "down")),
                    obfs: text(outbound, "obfs"),
                    protocol: None,
                },
                &tls,
            )
        }
        "ssh" => {
            if text(outbound, "private_key_path").is_some() {
                return server.skip(SSH_KEY_FILE);
            }
            // sing-box takes the key as one text or as a list of its lines.
            let private_key = match outbound.get("private_key") {
                Some(Value::Array(lines)) => Some(lines.iter().filter_map(|l| l.as_str()).collect::<Vec<_>>().join("\n")),
                _ => text(outbound, "private_key"),
            };
            ssh_link(
                &server,
                &Ssh {
                    user: text(outbound, "user").unwrap_or_default(),
                    password: text(outbound, "password"),
                    private_key,
                    passphrase: text(outbound, "private_key_passphrase"),
                    host_keys: list(outbound, "host_key"),
                },
            )
        }
        "tuic" => tuic_link(
            &server,
            &Tuic {
                uuid: text(outbound, "uuid").unwrap_or_default(),
                password: text(outbound, "password").unwrap_or_default(),
                congestion: text(outbound, "congestion_control"),
                relay_mode: text(outbound, "udp_relay_mode"),
                zero_rtt: flag(outbound, "zero_rtt_handshake"),
            },
            &tls,
        ),
        other => server.skip(&format!("{other} servers are not supported yet")),
    }
}

fn singbox_tls(block: Option<&Map<String, Value>>) -> Tls {
    let Some(block) = block else { return Tls::default() };
    let reality = object(block, "reality")
        .filter(|r| flag(r, "enabled"))
        .map(|r| (text(r, "public_key").unwrap_or_default(), text(r, "short_id").unwrap_or_default()));
    Tls {
        enabled: flag(block, "enabled"),
        sni: text(block, "server_name"),
        insecure: flag(block, "insecure"),
        alpn: list(block, "alpn"),
        fingerprint: object(block, "utls").filter(|u| flag(u, "enabled")).and_then(|u| text(u, "fingerprint")),
        reality,
    }
}

fn singbox_transport(block: Option<&Map<String, Value>>) -> Result<Transport, String> {
    let Some(block) = block else { return Ok(Transport::default()) };
    let kind = text(block, "type").unwrap_or_default();
    match kind.as_str() {
        "ws" => Ok(Transport {
            kind: "ws",
            path: text(block, "path"),
            host: object(block, "headers").and_then(|h| text(h, "Host").or_else(|| text(h, "host"))),
            service_name: None,
        }),
        "httpupgrade" => Ok(Transport { kind: "httpupgrade", path: text(block, "path"), host: text(block, "host"), service_name: None }),
        "grpc" => Ok(Transport { kind: "grpc", service_name: text(block, "service_name"), ..Transport::default() }),
        other => Err(format!("transport '{other}' is not supported")),
    }
}

// ── Clash and Clash Meta ────────────────────────────────────────────────────

fn clash(proxy: &Map<String, Value>) -> Entry {
    let kind = text(proxy, "type").unwrap_or_default().to_lowercase();
    if matches!(kind.as_str(), "direct" | "reject" | "dns") {
        return Entry::Ignore;
    }
    if kind == "wireguard" {
        return clash_wireguard(proxy);
    }
    let server = match Server::read(proxy, "name", "port") {
        Ok(server) => server,
        Err(reason) => return Entry::Skip(reason),
    };
    if text(proxy, "dialer-proxy").is_some() {
        return server.skip("servers that connect through another server are not supported");
    }
    let tls = clash_tls(proxy, &kind);
    let transport = match clash_transport(proxy) {
        Ok(transport) => transport,
        Err(why) => return server.skip(&why),
    };
    match kind.as_str() {
        "vless" => {
            if let Some(encryption) = text(proxy, "encryption").filter(|e| e != "none") {
                let shown: String = encryption.chars().take(24).collect();
                return server.skip(&format!("VLESS encryption '{shown}' is not supported"));
            }
            let mut q = query();
            q.append_pair("encryption", "none");
            put(&mut q, "security", tls.security());
            put(&mut q, "flow", text(proxy, "flow").as_deref());
            put(&mut q, "packetEncoding", text(proxy, "packet-encoding").as_deref());
            tls.write(&mut q);
            transport.write(&mut q);
            server.link("vless", &text(proxy, "uuid").unwrap_or_default(), q)
        }
        "vmess" => {
            let alter_id = text(proxy, "alterId").unwrap_or_else(|| "0".into());
            let cipher = text(proxy, "cipher").unwrap_or_else(|| "auto".into());
            vmess_link(&server, &text(proxy, "uuid").unwrap_or_default(), &alter_id, &cipher, &tls, &transport)
        }
        "trojan" => {
            let mut q = query();
            q.append_pair("security", "tls");
            tls.write(&mut q);
            transport.write(&mut q);
            server.link("trojan", &text(proxy, "password").unwrap_or_default(), q)
        }
        "ss" => {
            if text(proxy, "plugin").is_some() {
                return server.skip("Shadowsocks plugins are not supported");
            }
            shadowsocks_link(&server, &text(proxy, "cipher").unwrap_or_default(), &text(proxy, "password").unwrap_or_default())
        }
        "hysteria2" | "hy2" => {
            if text(proxy, "ports").is_some_and(|p| p.contains(',') || p.contains('-')) {
                return server.skip("Hysteria2 port hopping is not supported, use a single port");
            }
            let obfs = text(proxy, "obfs").map(|kind| (kind, text(proxy, "obfs-password")));
            hysteria2_link(&server, &text(proxy, "password").unwrap_or_default(), &tls, obfs)
        }
        "hysteria" => {
            if text(proxy, "ports").is_some_and(|p| p.contains(',') || p.contains('-')) {
                return server.skip("Hysteria port hopping is not supported, use a single port");
            }
            hysteria_link(
                &server,
                &Hysteria {
                    auth: text(proxy, "auth-str").or_else(|| text(proxy, "auth_str")),
                    up: text(proxy, "up"),
                    down: text(proxy, "down"),
                    obfs: text(proxy, "obfs"),
                    protocol: text(proxy, "protocol"),
                },
                &tls,
            )
        }
        "ssh" => {
            let private_key = text(proxy, "private-key");
            if private_key.as_deref().is_some_and(|k| !k.starts_with("-----BEGIN")) {
                return server.skip(SSH_KEY_FILE);
            }
            ssh_link(
                &server,
                &Ssh {
                    user: text(proxy, "username").unwrap_or_default(),
                    password: text(proxy, "password"),
                    private_key,
                    passphrase: text(proxy, "private-key-passphrase"),
                    host_keys: list(proxy, "host-key"),
                },
            )
        }
        "tuic" => {
            if text(proxy, "token").is_some() {
                return server.skip("TUIC v4 is not supported, ask for a v5 server (UUID and password)");
            }
            if flag(proxy, "disable-sni") {
                return server.skip("TUIC without SNI (disable-sni) is not supported");
            }
            tuic_link(
                &server,
                &Tuic {
                    uuid: text(proxy, "uuid").unwrap_or_default(),
                    password: text(proxy, "password").unwrap_or_default(),
                    congestion: text(proxy, "congestion-controller"),
                    relay_mode: text(proxy, "udp-relay-mode"),
                    zero_rtt: flag(proxy, "reduce-rtt"),
                },
                &tls,
            )
        }
        "" => server.skip("the entry has no type"),
        other => server.skip(&format!("{other} servers are not supported yet")),
    }
}

fn clash_tls(proxy: &Map<String, Value>, kind: &str) -> Tls {
    // Trojan, Hysteria, Hysteria2 and TUIC always use TLS and call the server name `sni`; VLESS and VMess
    // have a `tls` switch and call it `servername`.
    let always = matches!(kind, "trojan" | "hysteria2" | "hy2" | "hysteria" | "tuic");
    let reality = object(proxy, "reality-opts").map(|r| (text(r, "public-key").unwrap_or_default(), text(r, "short-id").unwrap_or_default()));
    Tls {
        enabled: always || flag(proxy, "tls"),
        sni: text(proxy, "servername").or_else(|| text(proxy, "sni")),
        insecure: flag(proxy, "skip-cert-verify"),
        alpn: list(proxy, "alpn"),
        fingerprint: text(proxy, "client-fingerprint"),
        reality,
    }
}

fn clash_transport(proxy: &Map<String, Value>) -> Result<Transport, String> {
    let network = text(proxy, "network").unwrap_or_else(|| "tcp".into());
    match network.as_str() {
        "tcp" => Ok(Transport::default()),
        "ws" => {
            let opts = object(proxy, "ws-opts");
            let path = opts.and_then(|o| text(o, "path"));
            let host = opts.and_then(|o| object(o, "headers")).and_then(|h| text(h, "Host").or_else(|| text(h, "host")));
            let upgrade = opts.is_some_and(|o| flag(o, "v2ray-http-upgrade"));
            Ok(Transport { kind: if upgrade { "httpupgrade" } else { "ws" }, path, host, service_name: None })
        }
        "grpc" => Ok(Transport {
            kind: "grpc",
            service_name: object(proxy, "grpc-opts").and_then(|o| text(o, "grpc-service-name")),
            ..Transport::default()
        }),
        other => Err(format!("transport '{other}' is not supported")),
    }
}

// ── Shared by both formats ──────────────────────────────────────────────────

fn shadowsocks_link(server: &Server, method: &str, password: &str) -> Entry {
    if method.is_empty() || password.is_empty() {
        return server.skip("no method or password");
    }
    // `method:password` written plainly (each part escaped), which the Shadowsocks reader accepts.
    let userinfo = format!("{}:{}", utf8_percent_encode(method, USERINFO), utf8_percent_encode(password, USERINFO));
    let fragment = if server.name.is_empty() { String::new() } else { format!("#{}", utf8_percent_encode(&server.name, NON_ALPHANUMERIC)) };
    Entry::Link(server.label().to_string(), format!("ss://{userinfo}@{}:{}{fragment}", format_host(&server.host), server.port))
}

fn hysteria2_link(server: &Server, password: &str, tls: &Tls, obfs: Option<(String, Option<String>)>) -> Entry {
    let mut q = query();
    put(&mut q, "sni", tls.sni.as_deref());
    if !tls.alpn.is_empty() {
        q.append_pair("alpn", &tls.alpn.join(","));
    }
    if tls.insecure {
        q.append_pair("insecure", "1");
    }
    if let Some((kind, password)) = obfs.filter(|(kind, _)| !kind.is_empty() && kind != "none") {
        q.append_pair("obfs", &kind);
        put(&mut q, "obfs-password", password.as_deref());
    }
    server.link("hysteria2", password, q)
}

/// The WireGuard values every format carries, under their own names.
#[derive(Default)]
struct WireGuard {
    private_key: Option<String>,
    public_key: Option<String>,
    pre_shared_key: Option<String>,
    addresses: Vec<String>,
    mtu: Option<String>,
    keepalive: Option<String>,
    reserved: Option<Value>,
}

const MULTI_PEER: &str = "WireGuard configs with more than one peer are not supported";

fn wireguard_link(server: &Server, wg: &WireGuard) -> Entry {
    let reserved = match wg.reserved.as_ref().map(super::wireguard::reserved_of).transpose() {
        Ok(reserved) => reserved.flatten(),
        Err(why) => return server.skip(&why),
    };
    let mut q = query();
    put(&mut q, "publickey", wg.public_key.as_deref());
    if !wg.addresses.is_empty() {
        q.append_pair("address", &wg.addresses.join(","));
    }
    put(&mut q, "mtu", wg.mtu.as_deref());
    put(&mut q, "presharedkey", wg.pre_shared_key.as_deref());
    put(&mut q, "keepalive", wg.keepalive.as_deref());
    if let Some([a, b, c]) = reserved {
        q.append_pair("reserved", &format!("{a},{b},{c}"));
    }
    let key = wg.private_key.as_deref().unwrap_or_default();
    server.link_encoded("wireguard", &utf8_percent_encode(key, NON_ALPHANUMERIC).to_string(), q)
}

/// A sing-box WireGuard entry: an endpoint (1.11 and later, peers listed) or the old outbound
/// (one peer, written flat).
fn singbox_wireguard(entry: &Map<String, Value>) -> Entry {
    let tag = text(entry, "tag").unwrap_or_default();
    let label = if tag.is_empty() { "a server".to_string() } else { tag.clone() };
    let peers = entry.get("peers").and_then(Value::as_array);
    let (peer, server) = match peers.map(Vec::as_slice) {
        Some([peer]) => {
            let peer = peer.as_object().cloned().unwrap_or_default();
            let Some(host) = text(&peer, "address").or_else(|| text(&peer, "server")) else { return Entry::Skip(format!("{label}: no server address")) };
            let Some(port) = port(&peer, "port").or_else(|| port(&peer, "server_port")) else { return Entry::Skip(format!("{label}: no valid port")) };
            (peer, Server { name: tag, host, port })
        }
        Some([]) | None => match Server::read(entry, "tag", "server_port") {
            Ok(server) => (entry.clone(), server),
            Err(reason) => return Entry::Skip(reason),
        },
        Some(_) => return Entry::Skip(format!("{label}: {MULTI_PEER}")),
    };
    let mut addresses = list(entry, "address");
    addresses.extend(list(entry, "local_address"));
    wireguard_link(
        &server,
        &WireGuard {
            private_key: text(entry, "private_key"),
            public_key: text(&peer, "public_key").or_else(|| text(&peer, "peer_public_key")),
            pre_shared_key: text(&peer, "pre_shared_key"),
            addresses,
            mtu: text(entry, "mtu"),
            keepalive: text(&peer, "persistent_keepalive_interval"),
            reserved: peer.get("reserved").or_else(|| entry.get("reserved")).cloned(),
        },
    )
}

/// A Clash WireGuard proxy: flat (one peer) or with a `peers` list.
fn clash_wireguard(proxy: &Map<String, Value>) -> Entry {
    let name = text(proxy, "name").unwrap_or_default();
    let label = if name.is_empty() { "a server".to_string() } else { name.clone() };
    if proxy.contains_key("amnezia-wg-option") {
        return Entry::Skip(format!("{label}: AmneziaWG options are not supported; sing-box runs plain WireGuard"));
    }
    let peers = proxy.get("peers").and_then(Value::as_array);
    let peer = match peers.map(Vec::as_slice) {
        Some([peer]) => peer.as_object().cloned().unwrap_or_default(),
        Some([]) | None => proxy.clone(),
        Some(_) => return Entry::Skip(format!("{label}: {MULTI_PEER}")),
    };
    let server = match Server::read(&Map::from_iter(peer.clone().into_iter().chain([("name".to_string(), Value::String(name))])), "name", "port") {
        Ok(server) => server,
        Err(reason) => return Entry::Skip(reason),
    };
    let addresses = [text(proxy, "ip"), text(proxy, "ipv6")].into_iter().flatten().collect();
    wireguard_link(
        &server,
        &WireGuard {
            private_key: text(proxy, "private-key"),
            public_key: text(&peer, "public-key"),
            pre_shared_key: text(&peer, "pre-shared-key"),
            addresses,
            mtu: text(proxy, "mtu"),
            keepalive: text(proxy, "persistent-keepalive"),
            reserved: peer.get("reserved").or_else(|| proxy.get("reserved")).cloned(),
        },
    )
}

const SSH_KEY_FILE: &str = "SSH key files on disk are not supported, paste the key into the server editor";

/// The SSH values both formats carry, under their own names.
struct Ssh {
    user: String,
    password: Option<String>,
    private_key: Option<String>,
    passphrase: Option<String>,
    host_keys: Vec<String>,
}

fn ssh_link(server: &Server, ssh: &Ssh) -> Entry {
    let mut q = query();
    put(&mut q, "private_key", ssh.private_key.as_deref());
    put(&mut q, "passphrase", ssh.passphrase.as_deref());
    if !ssh.host_keys.is_empty() {
        q.append_pair("host_key", &ssh.host_keys.join(","));
    }
    let mut userinfo = utf8_percent_encode(&ssh.user, USERINFO).to_string();
    if let Some(password) = ssh.password.as_deref() {
        userinfo = format!("{userinfo}:{}", utf8_percent_encode(password, USERINFO));
    }
    server.link_encoded("ssh", &userinfo, q)
}

/// The Hysteria v1 values both formats carry, under their own names. Speeds stay as written
/// ("100", "100 Mbps"); the protocol reads them.
struct Hysteria {
    auth: Option<String>,
    up: Option<String>,
    down: Option<String>,
    obfs: Option<String>,
    protocol: Option<String>,
}

fn hysteria_link(server: &Server, hysteria: &Hysteria, tls: &Tls) -> Entry {
    let mut q = query();
    put(&mut q, "protocol", hysteria.protocol.as_deref());
    put(&mut q, "auth", hysteria.auth.as_deref());
    put(&mut q, "peer", tls.sni.as_deref());
    if tls.insecure {
        q.append_pair("insecure", "1");
    }
    put(&mut q, "upmbps", hysteria.up.as_deref());
    put(&mut q, "downmbps", hysteria.down.as_deref());
    if !tls.alpn.is_empty() {
        q.append_pair("alpn", &tls.alpn.join(","));
    }
    if let Some(obfs) = hysteria.obfs.as_deref() {
        q.append_pair("obfs", "xplus");
        q.append_pair("obfsParam", obfs);
    }
    let query = q.finish();
    let fragment = if server.name.is_empty() { String::new() } else { format!("#{}", utf8_percent_encode(&server.name, NON_ALPHANUMERIC)) };
    Entry::Link(server.label().to_string(), format!("hysteria://{}:{}?{query}{fragment}", format_host(&server.host), server.port))
}

/// The TUIC v5 values both formats carry, under their own names.
struct Tuic {
    uuid: String,
    password: String,
    congestion: Option<String>,
    relay_mode: Option<String>,
    zero_rtt: bool,
}

fn tuic_link(server: &Server, tuic: &Tuic, tls: &Tls) -> Entry {
    if tuic.uuid.is_empty() || tuic.password.is_empty() {
        return server.skip("TUIC needs a UUID and a password (v4 servers are not supported)");
    }
    let mut q = query();
    put(&mut q, "sni", tls.sni.as_deref());
    if !tls.alpn.is_empty() {
        q.append_pair("alpn", &tls.alpn.join(","));
    }
    put(&mut q, "congestion_control", tuic.congestion.as_deref());
    put(&mut q, "udp_relay_mode", tuic.relay_mode.as_deref());
    if tuic.zero_rtt {
        q.append_pair("zero_rtt_handshake", "1");
    }
    if tls.insecure {
        q.append_pair("allow_insecure", "1");
    }
    let userinfo = format!("{}:{}", utf8_percent_encode(&tuic.uuid, USERINFO), utf8_percent_encode(&tuic.password, USERINFO));
    server.link_encoded("tuic", &userinfo, q)
}

#[cfg(test)]
#[path = "../tests/profile_formats.rs"]
mod tests;
