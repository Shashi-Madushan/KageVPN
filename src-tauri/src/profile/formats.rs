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
        let outbounds = value.get("outbounds")?.as_array()?;
        return Some(collect(outbounds.iter().cloned(), singbox));
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

    fn link(&self, scheme: &str, userinfo: &str, mut query: Query) -> Entry {
        let query = query.finish();
        let query = if query.is_empty() { String::new() } else { format!("?{query}") };
        let fragment = if self.name.is_empty() { String::new() } else { format!("#{}", utf8_percent_encode(&self.name, NON_ALPHANUMERIC)) };
        let userinfo = utf8_percent_encode(userinfo, USERINFO);
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
    let server = match Server::read(outbound, "tag", "server_port") {
        Ok(server) => server,
        Err(_) if kind == "hysteria2" && outbound.contains_key("server_ports") => {
            return Entry::Skip(format!("{}: Hysteria2 port hopping is not supported, use a single port", text(outbound, "tag").unwrap_or_default()));
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
        "" => server.skip("the entry has no type"),
        other => server.skip(&format!("{other} servers are not supported yet")),
    }
}

fn clash_tls(proxy: &Map<String, Value>, kind: &str) -> Tls {
    // Trojan and Hysteria2 always use TLS and call the server name `sni`; VLESS and VMess have a
    // `tls` switch and call it `servername`.
    let always = matches!(kind, "trojan" | "hysteria2" | "hy2");
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

#[cfg(test)]
#[path = "../tests/profile_formats.rs"]
mod tests;
