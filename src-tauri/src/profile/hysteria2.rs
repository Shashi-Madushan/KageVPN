use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use url::Url;

use super::common::{self, invalid, Obfs, Params, Tls};
use super::form::{Field, Section};
use super::{Profile, ProtocolDef, ProtocolId, Secret};
use crate::error::Result;

pub const ID: ProtocolId = ProtocolId("hysteria2");

const OBFS_SALAMANDER: &str = "salamander";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Hysteria2Settings {
    pub password: String,
    pub tls: Tls,
    pub obfs: Option<Obfs>,
}

pub struct Hysteria2;

impl ProtocolDef for Hysteria2 {
    fn id(&self) -> &'static str {
        ID.as_str()
    }

    fn label(&self) -> &'static str {
        "Hysteria2"
    }

    fn schemes(&self) -> &'static [&'static str] {
        &["hysteria2", "hy2"]
    }

    fn parse(&self, link: &str) -> Result<Profile> {
        parse(link)
    }

    fn to_link(&self, profile: &Profile) -> String {
        to_link(profile)
    }

    fn validate(&self, profile: &Profile) -> Result<()> {
        let settings: Hysteria2Settings = profile.settings_as()?;
        common::check_password(&settings.password)?;
        if settings.obfs.as_ref().is_some_and(|o| o.password.trim().is_empty()) {
            return Err(invalid("obfuscation needs a password"));
        }
        Ok(())
    }

    fn outbound(&self, profile: &Profile) -> Value {
        // QUIC carries its own transport, and browser fingerprints and Reality do not apply to it.
        let settings: Hysteria2Settings = profile.settings_as().unwrap_or_default();
        let mut outbound = json!({
            "type": "hysteria2",
            "server": profile.server,
            "server_port": profile.port,
            "password": settings.password,
            "tls": common::tls_base(&profile.server, &settings.tls),
        });
        if let Some(obfs) = &settings.obfs {
            outbound["obfs"] = json!({ "type": obfs.kind, "password": obfs.password });
        }
        outbound
    }

    fn identity(&self, profile: &Profile) -> String {
        let settings: Hysteria2Settings = profile.settings_as().unwrap_or_default();
        format!("{}|{}|{}", profile.server, profile.port, settings.password)
    }

    fn secrets(&self, profile: &Profile) -> Vec<Secret> {
        let settings: Hysteria2Settings = profile.settings_as().unwrap_or_default();
        let mut secrets = common::secrets(&[(&settings.password, "[password]")], &settings.tls, None);
        if let Some(obfs) = &settings.obfs {
            secrets.push(Secret::new(obfs.password.clone(), "[password]"));
        }
        secrets
    }

    fn summary(&self, _profile: &Profile) -> String {
        "Hysteria2".to_string()
    }

    fn sample(&self) -> &'static str {
        "hysteria2://sample-password@hy.example.com:443/?sni=hy.example.com&obfs=salamander&obfs-password=sample-mask#Sample%20Hysteria2"
    }

    fn form(&self) -> Vec<Field> {
        vec![
            Field::secret("password", "Password", Section::Connection),
            common::sni_field(),
            Field::secret("obfs.password", "Obfuscation password", Section::Security).optional(),
            common::insecure_field(),
            common::alpn_field(),
        ]
    }

    fn blank(&self) -> Profile {
        Profile::build(ID, String::new(), String::new(), common::DEFAULT_PORT, &Hysteria2Settings::default())
    }

    fn normalize(&self, profile: &mut Profile) {
        let Ok(mut settings) = profile.settings_as::<Hysteria2Settings>() else { return };
        settings.obfs = settings.obfs.take().filter(|o| !o.password.is_empty()).map(|o| Obfs { kind: OBFS_SALAMANDER.into(), password: o.password });
        profile.put_settings(&settings);
    }
}

fn parse(link: &str) -> Result<Profile> {
    let link = link.trim();
    if has_port_list(link) {
        return Err(invalid("Hysteria2 port hopping is not supported, use a single port"));
    }
    let url = Url::parse(link).map_err(|e| invalid(format!("not a valid link: {e}")))?;
    let password = common::credential_of(&url);
    if password.is_empty() {
        return Err(invalid("link has no password"));
    }
    let server = common::server_of(&url)?;
    let params = Params::of(&url);
    if params.get("pinSHA256").is_some() {
        return Err(invalid("certificate pinning (pinSHA256) is not supported"));
    }
    let obfs = match params.get("obfs").as_deref() {
        None | Some("none") => None,
        Some(OBFS_SALAMANDER) => Some(Obfs {
            kind: OBFS_SALAMANDER.into(),
            password: params.get("obfs-password").ok_or_else(|| invalid("obfuscation has no password (obfs-password)"))?,
        }),
        Some(other) => return Err(invalid(format!("obfuscation '{other}' is not supported"))),
    };

    // QUIC has no browser fingerprint, so only the name, ALPN and insecure flag carry over.
    let tls = Tls { fingerprint: None, ..common::tls_of(&params) };
    let settings = Hysteria2Settings { password, tls, obfs };
    Ok(Profile::build(
        ID,
        common::name_of(url.fragment(), &server),
        server,
        url.port().unwrap_or(common::DEFAULT_PORT),
        &settings,
    ))
}

fn to_link(profile: &Profile) -> String {
    let settings: Hysteria2Settings = profile.settings_as().unwrap_or_default();
    let mut query = common::query();
    common::push(&mut query, "sni", &settings.tls.sni);
    if let Some(obfs) = &settings.obfs {
        query.append_pair("obfs", &obfs.kind);
        query.append_pair("obfs-password", &obfs.password);
    }
    if settings.tls.allow_insecure {
        query.append_pair("insecure", "1");
    }
    common::build_link("hysteria2", &common::encode_userinfo(&settings.password), profile, &query.finish())
}

/// `host:443,5000-6000` style ports: the authority's port is not a single number.
fn has_port_list(link: &str) -> bool {
    let after_scheme = link.split_once("://").map_or(link, |(_, rest)| rest);
    let authority = after_scheme.split(['/', '?', '#']).next().unwrap_or_default();
    let host_and_port = authority.rsplit_once('@').map_or(authority, |(_, rest)| rest);
    let port = host_and_port.rsplit_once(':').filter(|(host, _)| !host.ends_with(']') || host.contains('[')).map(|(_, p)| p);
    port.is_some_and(|p| p.contains(',') || p.contains('-'))
}

#[cfg(test)]
#[path = "../tests/profile_hysteria2.rs"]
mod tests;
