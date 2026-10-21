//! WireGuard. Unlike the other protocols it is a sing-box *endpoint* (since 1.11), not an
//! outbound; the config builder puts it in `endpoints` and the selector refers to it by tag.
//! Servers come from `wireguard://` links, wg-quick `.conf` files and subscriptions.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use url::Url;

use super::common::{self, invalid, Params};
use super::form::{Field, Section};
use super::{Profile, ProtocolDef, ProtocolId, Secret, ServerKind};
use crate::error::Result;

pub const ID: ProtocolId = ProtocolId("wireguard");

pub const DEFAULT_PORT: u16 = 51820;
/// sing-box's own default.
pub const DEFAULT_MTU: u32 = 1408;
const MTU_RANGE: std::ops::RangeInclusive<u32> = 1280..=1500;
const KEY_PATTERN: &str = r"^\s*[a-z0-9+/]{43}=\s*$";
/// AmneziaWG adds these to wg-quick files; plain WireGuard (and sing-box) cannot run them.
const AMNEZIA_KEYS: &[&str] = &["jc", "jmin", "jmax", "s1", "s2", "h1", "h2", "h3", "h4"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WireGuardSettings {
    pub private_key: String,
    pub peer_public_key: String,
    #[serde(deserialize_with = "common::null_as_empty")]
    pub pre_shared_key: String,
    /// This device's addresses inside the tunnel, with prefixes (`172.16.0.2/32`).
    pub addresses: Vec<String>,
    /// The editor sends numbers as text; both are read.
    #[serde(deserialize_with = "number")]
    pub mtu: u32,
    /// Seconds between keepalive packets; 0 is off, saved as null so the editor shows it empty.
    #[serde(deserialize_with = "number", serialize_with = "zero_as_null")]
    pub keepalive: u32,
    /// The three reserved header bytes Cloudflare WARP uses.
    #[serde(deserialize_with = "reserved")]
    pub reserved: Option<[u8; 3]>,
}

impl Default for WireGuardSettings {
    fn default() -> Self {
        Self {
            private_key: String::new(),
            peer_public_key: String::new(),
            pre_shared_key: String::new(),
            addresses: Vec::new(),
            mtu: DEFAULT_MTU,
            keepalive: 0,
            reserved: None,
        }
    }
}

/// A number written as a number or as text; blank or null is 0 and unreadable text is
/// `u32::MAX`, which `validate` rejects with a message.
fn number<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<u32, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::Number(n) => n.as_u64().map_or(u32::MAX, |n| u32::try_from(n).unwrap_or(u32::MAX)),
        Value::String(s) if s.trim().is_empty() => 0,
        Value::String(s) => s.trim().parse().unwrap_or(u32::MAX),
        _ => 0,
    })
}

fn zero_as_null<S: serde::Serializer>(value: &u32, serializer: S) -> std::result::Result<S::Ok, S::Error> {
    if *value == 0 { serializer.serialize_none() } else { serializer.serialize_u32(*value) }
}

/// Reserved bytes as a list of numbers, text such as "1,2,3", or base64 of three bytes; anything
/// else fails to read, with the reason.
fn reserved<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Option<[u8; 3]>, D::Error> {
    let value = Value::deserialize(deserializer)?;
    reserved_of(&value).map_err(serde::de::Error::custom)
}

pub fn reserved_of(value: &Value) -> std::result::Result<Option<[u8; 3]>, String> {
    const WRONG: &str = "reserved must be three numbers from 0 to 255";
    let numbers: Vec<u64> = match value {
        Value::Null => return Ok(None),
        Value::Array(items) => items.iter().map(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))).collect::<Option<_>>().ok_or(WRONG)?,
        Value::String(s) if s.trim().is_empty() => return Ok(None),
        Value::String(s) => {
            let text = s.trim().trim_matches(['[', ']']);
            if text.contains(',') {
                text.split(',').map(|p| p.trim().parse().ok()).collect::<Option<_>>().ok_or(WRONG)?
            } else {
                STANDARD.decode(text).map_err(|_| WRONG)?.into_iter().map(u64::from).collect()
            }
        }
        _ => return Err(WRONG.into()),
    };
    match numbers.as_slice() {
        [a, b, c] if [a, b, c].iter().all(|n| **n <= 255) => Ok(Some([*a as u8, *b as u8, *c as u8])),
        _ => Err(WRONG.into()),
    }
}

pub struct WireGuard;

impl ProtocolDef for WireGuard {
    fn id(&self) -> &'static str {
        ID.as_str()
    }

    fn label(&self) -> &'static str {
        "WireGuard"
    }

    fn schemes(&self) -> &'static [&'static str] {
        &["wireguard", "wg"]
    }

    fn over_udp(&self) -> bool {
        true
    }

    fn kind(&self) -> ServerKind {
        ServerKind::Endpoint
    }

    fn feature(&self) -> Option<&'static str> {
        Some("with_wireguard")
    }

    fn parse(&self, link: &str) -> Result<Profile> {
        parse(link)
    }

    fn to_link(&self, profile: &Profile) -> String {
        to_link(profile)
    }

    fn validate(&self, profile: &Profile) -> Result<()> {
        check(&profile.settings_as()?)
    }

    fn outbound(&self, profile: &Profile) -> Value {
        let settings: WireGuardSettings = profile.settings_as().unwrap_or_default();
        // What goes through the tunnel is decided by Kage's routing, which picks this server;
        // WireGuard's own filter would only drop traffic.
        let mut peer = json!({
            "address": profile.server,
            "port": profile.port,
            "public_key": settings.peer_public_key,
            "allowed_ips": ["0.0.0.0/0", "::/0"],
        });
        if !settings.pre_shared_key.is_empty() {
            peer["pre_shared_key"] = json!(settings.pre_shared_key);
        }
        if settings.keepalive > 0 {
            peer["persistent_keepalive_interval"] = json!(settings.keepalive);
        }
        if let Some(reserved) = settings.reserved {
            peer["reserved"] = json!(reserved);
        }
        json!({
            "type": "wireguard",
            // In-process, so Proxy Only mode needs no privileges for it.
            "system": false,
            "mtu": settings.mtu,
            "address": settings.addresses,
            "private_key": settings.private_key,
            "peers": [peer],
        })
    }

    fn identity(&self, profile: &Profile) -> String {
        // Not the local address: some providers hand out a new one on every refresh.
        let settings: WireGuardSettings = profile.settings_as().unwrap_or_default();
        format!("{}|{}|{}", profile.server, profile.port, settings.peer_public_key)
    }

    fn secrets(&self, profile: &Profile) -> Vec<Secret> {
        let settings: WireGuardSettings = profile.settings_as().unwrap_or_default();
        [(settings.private_key, "[key]"), (settings.pre_shared_key, "[key]")]
            .into_iter()
            .filter(|(value, _)| !value.is_empty())
            .map(|(value, label)| Secret::new(value, label))
            .collect()
    }

    fn summary(&self, profile: &Profile) -> String {
        let settings: WireGuardSettings = profile.settings_as().unwrap_or_default();
        if settings.reserved.is_some() { "WireGuard · WARP".into() } else { "WireGuard".into() }
    }

    fn sample(&self) -> &'static str {
        "wireguard://JqrQV%2FCpFkpb3qIND9EboiuSWxrbxrxvG4i1L%2BJ1r1I%3D@wg.example.com:51820?publickey=cfd8nh5vBUAu37mGNeL%2F9tJ7DSpFLLvxSTIUGXBF%2B3g%3D&address=10.7.0.2%2F32%2Cfd07%3A%3A2%2F128&mtu=1420#Sample%20WireGuard"
    }

    fn form(&self) -> Vec<Field> {
        let key = "This is not a WireGuard key (44 characters ending in =).";
        vec![
            Field::secret("privateKey", "Private key", Section::Connection).pattern(KEY_PATTERN, key),
            Field::text("peerPublicKey", "Server public key", Section::Connection).mono().pattern(KEY_PATTERN, key),
            Field::list("addresses", "Local addresses", Section::Connection).placeholder("172.16.0.2/32, fd01::2/128"),
            Field::secret("preSharedKey", "Pre-shared key", Section::Security).optional().pattern(KEY_PATTERN, key),
            Field::text("mtu", "MTU", Section::Advanced).mono().placeholder("1408").pattern(r"^\s*\d{4}\s*$", "Enter an MTU from 1280 to 1500."),
            Field::text("keepalive", "Keepalive (seconds)", Section::Advanced).optional().mono().placeholder("Off").pattern(r"^\s*\d{1,5}\s*$", "Enter a number of seconds."),
            Field::text("reserved", "Reserved bytes", Section::Advanced)
                .optional()
                .mono()
                .placeholder("WARP only: 12,34,56")
                .pattern(r"^\s*\d{1,3}\s*,\s*\d{1,3}\s*,\s*\d{1,3}\s*$", "Enter three numbers, such as 12,34,56."),
        ]
    }

    fn blank(&self) -> Profile {
        Profile::build(ID, String::new(), String::new(), DEFAULT_PORT, &WireGuardSettings::default())
    }

    fn normalize(&self, profile: &mut Profile) {
        let Ok(mut settings) = profile.settings_as::<WireGuardSettings>() else { return };
        settings.private_key = settings.private_key.trim().to_string();
        settings.peer_public_key = settings.peer_public_key.trim().to_string();
        settings.pre_shared_key = settings.pre_shared_key.trim().to_string();
        settings.addresses = settings.addresses.iter().map(|a| with_prefix(a)).filter(|a| !a.is_empty()).collect();
        if settings.mtu == 0 {
            settings.mtu = DEFAULT_MTU;
        }
        profile.put_settings(&settings);
    }
}

/// `10.0.0.2` becomes `10.0.0.2/32`, `fd00::2` becomes `fd00::2/128`.
fn with_prefix(address: &str) -> String {
    let address = address.trim();
    if address.is_empty() || address.contains('/') {
        address.to_string()
    } else if address.contains(':') {
        format!("{address}/128")
    } else {
        format!("{address}/32")
    }
}

fn check_key(key: &str, which: &str) -> Result<()> {
    match STANDARD.decode(key.trim()) {
        Ok(bytes) if bytes.len() == 32 => Ok(()),
        _ => Err(invalid(format!("{which} key is not a WireGuard key"))),
    }
}

fn check(settings: &WireGuardSettings) -> Result<()> {
    check_key(&settings.private_key, "private")?;
    check_key(&settings.peer_public_key, "public")?;
    if !settings.pre_shared_key.is_empty() {
        check_key(&settings.pre_shared_key, "pre-shared")?;
    }
    if settings.addresses.is_empty() {
        return Err(invalid("WireGuard needs this device's address (Address)"));
    }
    for address in &settings.addresses {
        let (ip, prefix) = address.split_once('/').unwrap_or((address, ""));
        let parsed: std::result::Result<std::net::IpAddr, _> = ip.trim().parse();
        let max = if ip.contains(':') { 128 } else { 32 };
        if parsed.is_err() || prefix.parse::<u8>().map_or(!prefix.is_empty(), |p| p > max) {
            return Err(invalid(format!("'{address}' is not an address such as 172.16.0.2/32")));
        }
    }
    if !MTU_RANGE.contains(&settings.mtu) {
        return Err(invalid("MTU must be between 1280 and 1500"));
    }
    if settings.keepalive > 65535 {
        return Err(invalid("keepalive must be a number of seconds up to 65535"));
    }
    Ok(())
}

/// Keys hold `/`, `+` and `=`. Links that leave them unescaped before the `@` would read the
/// `/` as the start of a path, so the part before the host is escaped first.
fn escape_userinfo(link: &str) -> String {
    let Some((scheme, rest)) = link.split_once("://") else { return link.to_string() };
    let Some((user, host)) = rest.split_once('@') else { return link.to_string() };
    if !user.contains(['/', '+', '=']) {
        return link.to_string();
    }
    let user = user.replace('+', "%2B").replace('/', "%2F").replace('=', "%3D");
    format!("{scheme}://{user}@{host}")
}

/// Query values decode `+` to a space, which a base64 key never holds.
fn key_param(params: &Params, keys: &[&str]) -> Option<String> {
    params.first(keys).map(|k| k.trim().replace(' ', "+"))
}

fn parse(link: &str) -> Result<Profile> {
    let url = Url::parse(&escape_userinfo(link.trim())).map_err(|e| invalid(format!("not a valid link: {e}")))?;
    let server = common::server_of(&url)?;
    let params = Params::of(&url);
    let user = percent_encoding::percent_decode_str(url.username()).decode_utf8_lossy().into_owned();
    let private_key = if user.is_empty() { key_param(&params, &["privatekey", "private_key", "privateKey"]).unwrap_or_default() } else { user };
    let number = |keys: &[&str]| params.first(keys).map(|v| v.trim().parse::<u32>().unwrap_or(u32::MAX));
    let settings = WireGuardSettings {
        private_key,
        peer_public_key: key_param(&params, &["publickey", "publicKey", "public_key", "peer_public_key"]).unwrap_or_default(),
        pre_shared_key: key_param(&params, &["presharedkey", "preSharedKey", "pre_shared_key", "psk"]).unwrap_or_default(),
        addresses: params
            .first(&["address", "ip", "local_address"])
            .map(|a| a.split(',').map(with_prefix).filter(|a| !a.is_empty()).collect())
            .unwrap_or_default(),
        mtu: number(&["mtu"]).unwrap_or(DEFAULT_MTU),
        keepalive: number(&["keepalive", "persistentKeepalive"]).unwrap_or(0),
        reserved: reserved_of(&params.get("reserved").map(|r| Value::String(r.replace(' ', "+"))).unwrap_or(Value::Null)).map_err(invalid)?,
    };
    check(&settings)?;
    Ok(Profile::build(ID, common::name_of(url.fragment(), &server), server, url.port().unwrap_or(DEFAULT_PORT), &settings))
}

fn to_link(profile: &Profile) -> String {
    let settings: WireGuardSettings = profile.settings_as().unwrap_or_default();
    let mut query = common::query();
    query.append_pair("publickey", &settings.peer_public_key);
    query.append_pair("address", &settings.addresses.join(","));
    query.append_pair("mtu", &settings.mtu.to_string());
    if !settings.pre_shared_key.is_empty() {
        query.append_pair("presharedkey", &settings.pre_shared_key);
    }
    if settings.keepalive > 0 {
        query.append_pair("keepalive", &settings.keepalive.to_string());
    }
    if let Some([a, b, c]) = settings.reserved {
        query.append_pair("reserved", &format!("{a},{b},{c}"));
    }
    common::build_link("wireguard", &common::encode_userinfo(&settings.private_key), profile, &query.finish())
}

// ── wg-quick files ──────────────────────────────────────────────────────────

/// Whether this text is a wg-quick file: its first section is `[Interface]` and it has a peer.
pub fn is_wg_quick(text: &str) -> bool {
    let first = text.lines().map(str::trim).find(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with(';'));
    first.is_some_and(|l| l.eq_ignore_ascii_case("[interface]")) && text.lines().any(|l| l.trim().eq_ignore_ascii_case("[peer]"))
}

/// One `[Interface]` or `[Peer]` block, keys lowercased.
#[derive(Default)]
struct ConfSection {
    values: Vec<(String, String)>,
}

impl ConfSection {
    fn all(&self, key: &str) -> Vec<String> {
        self.values.iter().filter(|(k, _)| k == key).flat_map(|(_, v)| v.split(',').map(|p| p.trim().to_string())).filter(|v| !v.is_empty()).collect()
    }

    fn one(&self, key: &str) -> Option<String> {
        self.values.iter().find(|(k, _)| k == key).map(|(_, v)| v.trim().to_string()).filter(|v| !v.is_empty())
    }
}

const INTERFACE_KEYS: &[&str] = &["privatekey", "address", "dns", "mtu"];
const PEER_KEYS: &[&str] = &["publickey", "presharedkey", "endpoint", "allowedips", "persistentkeepalive"];

/// Reads a wg-quick file into one server, and a note naming the lines it does not use.
/// `name_hint` is the dropped file's name without `.conf`.
pub fn parse_wg_quick(text: &str, name_hint: Option<&str>) -> Result<(Profile, Option<String>)> {
    let mut interface = ConfSection::default();
    let mut peers: Vec<ConfSection> = Vec::new();
    let mut in_peer = None::<bool>;
    let mut name_comment = None;
    // Original spellings of the keys, in file order, for the note.
    let mut unused: Vec<String> = Vec::new();
    for line in text.lines().map(str::trim) {
        if let Some(comment) = line.strip_prefix('#').or_else(|| line.strip_prefix(';')) {
            if let Some((key, value)) = comment.split_once('=') {
                if key.trim().eq_ignore_ascii_case("name") && !value.trim().is_empty() {
                    name_comment = Some(value.trim().to_string());
                }
            }
            continue;
        }
        if line.is_empty() {
            continue;
        }
        if line.eq_ignore_ascii_case("[interface]") {
            in_peer = Some(false);
            continue;
        }
        if line.eq_ignore_ascii_case("[peer]") {
            in_peer = Some(true);
            peers.push(ConfSection::default());
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let (original, key) = (key.trim(), key.trim().to_ascii_lowercase());
        if AMNEZIA_KEYS.contains(&key.as_str()) {
            return Err(invalid(format!("AmneziaWG options ({original}, ...) are not supported; sing-box runs plain WireGuard")));
        }
        let section = match in_peer {
            Some(true) => peers.last_mut().expect("a [Peer] line came first"),
            Some(false) => &mut interface,
            None => continue,
        };
        let known = if in_peer == Some(true) { PEER_KEYS } else { INTERFACE_KEYS };
        if !known.contains(&key.as_str()) && !unused.iter().any(|u| u.eq_ignore_ascii_case(original)) {
            unused.push(original.to_string());
        }
        section.values.push((key, value.trim().to_string()));
    }
    let peer = match peers.as_slice() {
        [] => return Err(invalid("the file has no [Peer] section")),
        [peer] => peer,
        _ => return Err(invalid("WireGuard configs with more than one peer are not supported")),
    };
    let endpoint = peer.one("endpoint").ok_or_else(|| invalid("the file has no Endpoint for its peer"))?;
    let (host, port) = split_endpoint(&endpoint)?;

    let mut ignored = Vec::new();
    if !interface.all("dns").is_empty() {
        ignored.push("DNS".to_string());
    }
    let allowed = peer.all("allowedips");
    if !allowed.is_empty() && !allowed.iter().any(|a| a == "0.0.0.0/0") {
        ignored.push("AllowedIPs".to_string());
    }
    ignored.extend(unused);

    let settings = WireGuardSettings {
        private_key: interface.one("privatekey").unwrap_or_default(),
        peer_public_key: peer.one("publickey").unwrap_or_default(),
        pre_shared_key: peer.one("presharedkey").unwrap_or_default(),
        addresses: interface.all("address").iter().map(|a| with_prefix(a)).collect(),
        mtu: interface.one("mtu").map_or(DEFAULT_MTU, |m| m.parse().unwrap_or(u32::MAX)),
        keepalive: peer.one("persistentkeepalive").map_or(0, |k| k.parse().unwrap_or(u32::MAX)),
        reserved: None,
    };
    check(&settings)?;
    let name = name_hint.map(str::trim).filter(|n| !n.is_empty()).map(str::to_string).or(name_comment).unwrap_or_else(|| host.clone());
    let note = (!ignored.is_empty()).then(|| {
        let list = join_names(&ignored);
        let verb = if ignored.len() == 1 { "is" } else { "are" };
        format!("{list} from the file {verb} not used; Kage's DNS and routing settings decide")
    });
    Ok((Profile::build(ID, name, host, port, &settings), note))
}

/// `host:port`, `[v6]:port`.
fn split_endpoint(endpoint: &str) -> Result<(String, u16)> {
    let wrong = || invalid(format!("Endpoint '{endpoint}' is not host:port"));
    let (host, port) = endpoint.rsplit_once(':').ok_or_else(wrong)?;
    let port = port.parse::<u16>().ok().filter(|p| *p != 0).ok_or_else(wrong)?;
    let host = host.trim_matches(['[', ']']);
    if host.is_empty() {
        return Err(wrong());
    }
    Ok((host.to_string(), port))
}

/// "A", "A and B", "A, B and C".
fn join_names(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

#[cfg(test)]
#[path = "../tests/profile_wireguard.rs"]
mod tests;
