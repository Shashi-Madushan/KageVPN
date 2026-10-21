//! Hysteria (version 1). Superseded by Hysteria2 upstream, but providers still hand it out and
//! sing-box still runs it.

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use url::Url;

use super::common::{self, invalid, Params, Tls};
use super::form::{Field, Section};
use super::{Profile, ProtocolDef, ProtocolId, Secret};
use crate::error::Result;

pub const ID: ProtocolId = ProtocolId("hysteria");

const OBFS_XPLUS: &str = "xplus";
/// Hysteria v1 clients send this ALPN when a link names none.
const DEFAULT_ALPN: &str = "hysteria";
/// sing-box needs both speeds; many links leave them out, so these fill the gap (Mbps).
pub const DEFAULT_UP_MBPS: u32 = 10;
pub const DEFAULT_DOWN_MBPS: u32 = 50;
const MAX_MBPS: u32 = 100_000;
const SPEED_PATTERN: &str = r"^\s*\d+(\.\d+)?\s*(m|mbps|g|gbps)?\s*$";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HysteriaSettings {
    /// Optional in Hysteria v1: a server may accept anyone.
    #[serde(deserialize_with = "common::null_as_empty")]
    pub auth: String,
    /// The editor sends text, saved profiles hold numbers; both are read.
    #[serde(deserialize_with = "speed")]
    pub up_mbps: u32,
    #[serde(deserialize_with = "speed")]
    pub down_mbps: u32,
    /// The `xplus` obfuscation password.
    pub obfs: Option<String>,
    pub tls: Tls,
}

impl Default for HysteriaSettings {
    fn default() -> Self {
        Self {
            auth: String::new(),
            up_mbps: DEFAULT_UP_MBPS,
            down_mbps: DEFAULT_DOWN_MBPS,
            obfs: None,
            tls: Tls { alpn: vec![DEFAULT_ALPN.into()], ..Tls::default() },
        }
    }
}

/// A speed written as a number or as text such as "100", "100 Mbps" or "1 Gbps". Anything
/// unreadable becomes 0, which `validate` rejects with a message.
fn speed<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<u32, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::Number(n) => n.as_u64().map_or(0, |n| u32::try_from(n).unwrap_or(u32::MAX)),
        Value::String(s) => parse_speed(&s).unwrap_or(0),
        _ => 0,
    })
}

/// Mbps from "100", "100 Mbps", "100m", "1.5 Gbps" or "2g". `None` for anything else.
pub fn parse_speed(text: &str) -> Option<u32> {
    let text = text.trim().to_lowercase();
    let (number, factor) = if let Some(n) = text.strip_suffix("gbps").or_else(|| text.strip_suffix('g')) {
        (n, 1000.0)
    } else {
        (text.strip_suffix("mbps").or_else(|| text.strip_suffix('m')).unwrap_or(&text), 1.0)
    };
    let mbps = number.trim().parse::<f64>().ok()? * factor;
    (mbps.is_finite() && mbps >= 1.0 && mbps <= f64::from(MAX_MBPS)).then(|| mbps.round() as u32)
}

pub struct Hysteria;

impl ProtocolDef for Hysteria {
    fn id(&self) -> &'static str {
        ID.as_str()
    }

    fn label(&self) -> &'static str {
        "Hysteria"
    }

    fn schemes(&self) -> &'static [&'static str] {
        &["hysteria"]
    }

    fn over_udp(&self) -> bool {
        true
    }

    fn parse(&self, link: &str) -> Result<Profile> {
        parse(link)
    }

    fn to_link(&self, profile: &Profile) -> String {
        to_link(profile)
    }

    fn validate(&self, profile: &Profile) -> Result<()> {
        let settings: HysteriaSettings = profile.settings_as()?;
        for (mbps, what) in [(settings.up_mbps, "upload"), (settings.down_mbps, "download")] {
            if !(1..=MAX_MBPS).contains(&mbps) {
                return Err(invalid(format!("{what} speed must be between 1 and {MAX_MBPS} Mbps")));
            }
        }
        if settings.obfs.as_deref().is_some_and(|o| o.trim().is_empty()) {
            return Err(invalid("obfuscation needs a password"));
        }
        Ok(())
    }

    fn outbound(&self, profile: &Profile) -> Value {
        // QUIC carries its own transport, and browser fingerprints and Reality do not apply to it.
        let settings: HysteriaSettings = profile.settings_as().unwrap_or_default();
        let mut outbound = json!({
            "type": "hysteria",
            "server": profile.server,
            "server_port": profile.port,
            "up_mbps": settings.up_mbps,
            "down_mbps": settings.down_mbps,
            "tls": common::tls_base(&profile.server, &settings.tls),
        });
        if !settings.auth.is_empty() {
            outbound["auth_str"] = json!(settings.auth);
        }
        if let Some(obfs) = &settings.obfs {
            outbound["obfs"] = json!(obfs);
        }
        outbound
    }

    fn identity(&self, profile: &Profile) -> String {
        let settings: HysteriaSettings = profile.settings_as().unwrap_or_default();
        format!("{}|{}|{}", profile.server, profile.port, settings.auth)
    }

    fn secrets(&self, profile: &Profile) -> Vec<Secret> {
        let settings: HysteriaSettings = profile.settings_as().unwrap_or_default();
        let mut secrets = common::secrets(&[], &settings.tls, None);
        if !settings.auth.is_empty() {
            secrets.push(Secret::new(settings.auth.clone(), "[password]"));
        }
        if let Some(obfs) = &settings.obfs {
            secrets.push(Secret::new(obfs.clone(), "[password]"));
        }
        secrets
    }

    fn summary(&self, profile: &Profile) -> String {
        let settings: HysteriaSettings = profile.settings_as().unwrap_or_default();
        format!("Hysteria · {}/{} Mbps", settings.up_mbps, settings.down_mbps)
    }

    fn sample(&self) -> &'static str {
        "hysteria://hy1.example.com:443?protocol=udp&auth=sample-auth&peer=hy1.example.com&upmbps=20&downmbps=100&alpn=hysteria&obfs=xplus&obfsParam=sample-mask#Sample%20Hysteria"
    }

    fn form(&self) -> Vec<Field> {
        vec![
            Field::secret("auth", "Password", Section::Connection).optional().placeholder("None"),
            Field::text("upMbps", "Upload speed (Mbps)", Section::Connection).mono().pattern(SPEED_PATTERN, "Enter a speed in Mbps, such as 20."),
            Field::text("downMbps", "Download speed (Mbps)", Section::Connection).mono().pattern(SPEED_PATTERN, "Enter a speed in Mbps, such as 100."),
            common::sni_field(),
            Field::secret("obfs", "Obfuscation password (xplus)", Section::Security).optional(),
            common::insecure_field(),
            common::alpn_field(),
        ]
    }

    fn blank(&self) -> Profile {
        Profile::build(ID, String::new(), String::new(), common::DEFAULT_PORT, &HysteriaSettings::default())
    }

    fn normalize(&self, profile: &mut Profile) {
        let Ok(mut settings) = profile.settings_as::<HysteriaSettings>() else { return };
        settings.obfs = settings.obfs.take().filter(|o| !o.is_empty());
        if settings.tls.alpn.is_empty() {
            settings.tls.alpn = vec![DEFAULT_ALPN.into()];
        }
        profile.put_settings(&settings);
    }

    fn import_note(&self, link: &str) -> Option<String> {
        let params = Params::of(&Url::parse(link.trim()).ok()?);
        let up = params.first(&["upmbps", "up"]).is_none();
        let down = params.first(&["downmbps", "down"]).is_none();
        let filled = match (up, down) {
            (true, true) => format!("no speeds in the link, using {DEFAULT_UP_MBPS} Mbps up and {DEFAULT_DOWN_MBPS} Mbps down"),
            (true, false) => format!("no upload speed in the link, using {DEFAULT_UP_MBPS} Mbps"),
            (false, true) => format!("no download speed in the link, using {DEFAULT_DOWN_MBPS} Mbps"),
            (false, false) => return None,
        };
        Some(format!("{filled}; change it in the server editor if that is wrong"))
    }
}

fn parse(link: &str) -> Result<Profile> {
    let link = link.trim();
    if common::has_port_list(link) {
        return Err(invalid("Hysteria port hopping is not supported, use a single port"));
    }
    let url = Url::parse(link).map_err(|e| invalid(format!("not a valid link: {e}")))?;
    let server = common::server_of(&url)?;
    let params = Params::of(&url);
    if params.get("mport").is_some() {
        return Err(invalid("Hysteria port hopping is not supported, use a single port"));
    }
    if let Some(protocol) = params.get("protocol").filter(|p| p != "udp") {
        return Err(invalid(format!("Hysteria over {protocol} is not supported, only udp")));
    }
    let obfs = match params.get("obfs").as_deref() {
        None | Some("none") => None,
        Some(OBFS_XPLUS) => Some(params.first(&["obfsParam", "obfs-password"]).ok_or_else(|| invalid("obfuscation has no password (obfsParam)"))?),
        Some(other) => return Err(invalid(format!("obfuscation '{other}' is not supported"))),
    };
    let speed_of = |keys: &[&str], default: u32, what: &str| match params.first(keys) {
        None => Ok(default),
        Some(text) => parse_speed(&text).ok_or_else(|| invalid(format!("{what} speed '{text}' is not a number of Mbps"))),
    };
    let up_mbps = speed_of(&["upmbps", "up"], DEFAULT_UP_MBPS, "upload")?;
    let down_mbps = speed_of(&["downmbps", "down"], DEFAULT_DOWN_MBPS, "download")?;

    // QUIC has no browser fingerprint, so only the name, ALPN and insecure flag carry over.
    let mut tls = Tls { fingerprint: None, ..common::tls_of(&params) };
    if tls.alpn.is_empty() {
        tls.alpn = vec![DEFAULT_ALPN.into()];
    }
    let auth = params.get("auth").unwrap_or_else(|| common::credential_of(&url));
    let settings = HysteriaSettings { auth, up_mbps, down_mbps, obfs, tls };
    Ok(Profile::build(
        ID,
        common::name_of(url.fragment(), &server),
        server,
        url.port().unwrap_or(common::DEFAULT_PORT),
        &settings,
    ))
}

/// The official form: everything in the query, nothing before the host.
fn to_link(profile: &Profile) -> String {
    let settings: HysteriaSettings = profile.settings_as().unwrap_or_default();
    let mut query = common::query();
    query.append_pair("protocol", "udp");
    if !settings.auth.is_empty() {
        query.append_pair("auth", &settings.auth);
    }
    common::push(&mut query, "peer", &settings.tls.sni);
    if settings.tls.allow_insecure {
        query.append_pair("insecure", "1");
    }
    query.append_pair("upmbps", &settings.up_mbps.to_string());
    query.append_pair("downmbps", &settings.down_mbps.to_string());
    if !settings.tls.alpn.is_empty() {
        query.append_pair("alpn", &settings.tls.alpn.join(","));
    }
    if let Some(obfs) = &settings.obfs {
        query.append_pair("obfs", OBFS_XPLUS);
        query.append_pair("obfsParam", obfs);
    }
    format!(
        "hysteria://{}:{}?{}#{}",
        common::format_host(&profile.server),
        profile.port,
        query.finish(),
        utf8_percent_encode(&profile.name, NON_ALPHANUMERIC)
    )
}

#[cfg(test)]
#[path = "../tests/profile_hysteria.rs"]
mod tests;
