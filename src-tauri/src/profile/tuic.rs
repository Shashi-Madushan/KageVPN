use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use url::Url;
use uuid::Uuid;

use super::common::{self, invalid, Params, Tls};
use super::form::{Field, Section};
use super::{Profile, ProtocolDef, ProtocolId, Secret};
use crate::error::Result;

pub const ID: ProtocolId = ProtocolId("tuic");

const CONGESTION: &[(&str, &str)] = &[("bbr", "BBR"), ("cubic", "CUBIC"), ("new_reno", "NewReno")];
const RELAY_MODES: &[(&str, &str)] = &[("native", "Native"), ("quic", "QUIC")];
const DEFAULT_CONGESTION: &str = "bbr";
const DEFAULT_RELAY_MODE: &str = "native";
/// QUIC needs an ALPN and TUIC servers almost always expect `h3`, so a link without one gets it.
const DEFAULT_ALPN: &str = "h3";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TuicSettings {
    pub uuid: String,
    pub password: String,
    pub congestion_control: String,
    pub udp_relay_mode: String,
    pub zero_rtt: bool,
    pub tls: Tls,
}

impl Default for TuicSettings {
    fn default() -> Self {
        Self {
            uuid: String::new(),
            password: String::new(),
            congestion_control: DEFAULT_CONGESTION.into(),
            udp_relay_mode: DEFAULT_RELAY_MODE.into(),
            zero_rtt: false,
            tls: Tls { alpn: vec![DEFAULT_ALPN.into()], ..Tls::default() },
        }
    }
}

pub struct Tuic;

impl ProtocolDef for Tuic {
    fn id(&self) -> &'static str {
        ID.as_str()
    }

    fn label(&self) -> &'static str {
        "TUIC"
    }

    fn schemes(&self) -> &'static [&'static str] {
        &["tuic"]
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
        let settings: TuicSettings = profile.settings_as()?;
        if Uuid::parse_str(&settings.uuid).is_err() {
            return Err(invalid("UUID is not valid"));
        }
        common::check_password(&settings.password)?;
        check_choice(&settings.congestion_control, CONGESTION, "congestion control")?;
        check_choice(&settings.udp_relay_mode, RELAY_MODES, "UDP relay mode")?;
        Ok(())
    }

    fn outbound(&self, profile: &Profile) -> Value {
        // QUIC carries its own transport, and browser fingerprints and Reality do not apply to it.
        let settings: TuicSettings = profile.settings_as().unwrap_or_default();
        let mut outbound = json!({
            "type": "tuic",
            "server": profile.server,
            "server_port": profile.port,
            "uuid": settings.uuid,
            "password": settings.password,
            "congestion_control": settings.congestion_control,
            "udp_relay_mode": settings.udp_relay_mode,
            "tls": common::tls_base(&profile.server, &settings.tls),
        });
        if settings.zero_rtt {
            outbound["zero_rtt_handshake"] = json!(true);
        }
        outbound
    }

    fn identity(&self, profile: &Profile) -> String {
        let settings: TuicSettings = profile.settings_as().unwrap_or_default();
        format!("{}|{}|{}", profile.server, profile.port, settings.uuid)
    }

    fn secrets(&self, profile: &Profile) -> Vec<Secret> {
        let settings: TuicSettings = profile.settings_as().unwrap_or_default();
        common::secrets(&[(&settings.uuid, "[uuid]"), (&settings.password, "[password]")], &settings.tls, None)
    }

    fn summary(&self, profile: &Profile) -> String {
        let settings: TuicSettings = profile.settings_as().unwrap_or_default();
        let congestion = CONGESTION.iter().find(|(value, _)| *value == settings.congestion_control).map_or("TUIC", |(_, label)| label);
        format!("TUIC · {congestion}")
    }

    fn sample(&self) -> &'static str {
        "tuic://00000000-0000-4000-8000-000000000001:sample-password@tuic.example.com:443/?sni=tuic.example.com&alpn=h3&congestion_control=bbr&udp_relay_mode=native#Sample%20TUIC"
    }

    fn form(&self) -> Vec<Field> {
        vec![
            Field::text("uuid", "User ID (UUID)", Section::Connection)
                .mono()
                .placeholder("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx")
                .pattern("^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$", "This is not a valid UUID."),
            Field::secret("password", "Password", Section::Connection),
            common::sni_field(),
            common::insecure_field(),
            Field::select("congestionControl", "Congestion control", Section::Advanced, CONGESTION),
            Field::segmented("udpRelayMode", "UDP relay", Section::Advanced, RELAY_MODES),
            Field::toggle("zeroRtt", "0-RTT handshake", Section::Advanced)
                .warning("Faster to connect, but a recorded handshake can be replayed. Leave it off unless the server asks for it."),
            common::alpn_field(),
        ]
    }

    fn blank(&self) -> Profile {
        Profile::build(ID, String::new(), String::new(), common::DEFAULT_PORT, &TuicSettings::default())
    }

    fn normalize(&self, profile: &mut Profile) {
        let Ok(mut settings) = profile.settings_as::<TuicSettings>() else { return };
        settings.uuid = settings.uuid.trim().to_lowercase();
        settings.congestion_control = choice_or(&settings.congestion_control, DEFAULT_CONGESTION);
        settings.udp_relay_mode = choice_or(&settings.udp_relay_mode, DEFAULT_RELAY_MODE);
        if settings.tls.alpn.is_empty() {
            settings.tls.alpn = vec![DEFAULT_ALPN.into()];
        }
        profile.put_settings(&settings);
    }
}

fn choice_or(value: &str, default: &str) -> String {
    let value = value.trim().to_lowercase();
    if value.is_empty() { default.to_string() } else { value }
}

fn check_choice(value: &str, choices: &[(&str, &str)], what: &str) -> Result<()> {
    if choices.iter().any(|(v, _)| *v == value) {
        Ok(())
    } else {
        Err(invalid(format!("{what} '{value}' is not supported")))
    }
}

fn parse(link: &str) -> Result<Profile> {
    let link = link.trim();
    if common::has_port_list(link) {
        return Err(invalid("TUIC port hopping is not supported, use a single port"));
    }
    let url = Url::parse(link).map_err(|e| invalid(format!("not a valid link: {e}")))?;
    let params = Params::of(&url);
    let password = url.password().map(|p| percent_decode_str(p).decode_utf8_lossy().into_owned()).unwrap_or_default();
    if params.get("token").is_some() || params.get("version").as_deref() == Some("4") || password.is_empty() {
        return Err(invalid("TUIC v4 is not supported, ask for a v5 link (UUID and password)"));
    }
    let uuid = Uuid::parse_str(&percent_decode_str(url.username()).decode_utf8_lossy()).map_err(|_| invalid("link has no valid UUID"))?;
    let server = common::server_of(&url)?;
    if params.flag(&["disable_sni"]) {
        return Err(invalid("TUIC without SNI (disable_sni) is not supported"));
    }
    let congestion_control = choice_or(&params.first(&["congestion_control", "congestion_controller"]).unwrap_or_default(), DEFAULT_CONGESTION);
    check_choice(&congestion_control, CONGESTION, "congestion control")?;
    let udp_relay_mode = choice_or(&params.get("udp_relay_mode").unwrap_or_default(), DEFAULT_RELAY_MODE);
    check_choice(&udp_relay_mode, RELAY_MODES, "UDP relay mode")?;

    // QUIC has no browser fingerprint, so only the name, ALPN and insecure flag carry over.
    let mut tls = Tls { fingerprint: None, ..common::tls_of(&params) };
    tls.allow_insecure |= params.flag(&["allow_insecure"]);
    if tls.alpn.is_empty() {
        tls.alpn = vec![DEFAULT_ALPN.into()];
    }
    let settings = TuicSettings {
        uuid: uuid.to_string(),
        password,
        congestion_control,
        udp_relay_mode,
        zero_rtt: params.flag(&["zero_rtt_handshake", "reduce_rtt"]),
        tls,
    };
    Ok(Profile::build(
        ID,
        common::name_of(url.fragment(), &server),
        server,
        url.port().unwrap_or(common::DEFAULT_PORT),
        &settings,
    ))
}

fn to_link(profile: &Profile) -> String {
    let settings: TuicSettings = profile.settings_as().unwrap_or_default();
    let mut query = common::query();
    common::push(&mut query, "sni", &settings.tls.sni);
    if !settings.tls.alpn.is_empty() {
        query.append_pair("alpn", &settings.tls.alpn.join(","));
    }
    query.append_pair("congestion_control", &settings.congestion_control);
    query.append_pair("udp_relay_mode", &settings.udp_relay_mode);
    if settings.zero_rtt {
        query.append_pair("zero_rtt_handshake", "1");
    }
    if settings.tls.allow_insecure {
        query.append_pair("allow_insecure", "1");
    }
    let userinfo = format!("{}:{}", settings.uuid, common::encode_userinfo(&settings.password));
    common::build_link("tuic", &userinfo, profile, &query.finish())
}

#[cfg(test)]
#[path = "../tests/profile_tuic.rs"]
mod tests;
