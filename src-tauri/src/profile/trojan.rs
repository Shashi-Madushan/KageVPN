use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use url::Url;

use super::common::{self, invalid, Params, Tls, Transport};
use super::form::{Field, Section};
use super::{Profile, ProtocolDef, ProtocolId, Secret};
use crate::error::Result;

pub const ID: ProtocolId = ProtocolId("trojan");

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TrojanSettings {
    pub password: String,
    pub tls: Tls,
    pub transport: Transport,
}

pub struct Trojan;

impl ProtocolDef for Trojan {
    fn id(&self) -> &'static str {
        ID.as_str()
    }

    fn label(&self) -> &'static str {
        "Trojan"
    }

    fn schemes(&self) -> &'static [&'static str] {
        &["trojan"]
    }

    fn parse(&self, link: &str) -> Result<Profile> {
        parse(link)
    }

    fn to_link(&self, profile: &Profile) -> String {
        to_link(profile)
    }

    fn validate(&self, profile: &Profile) -> Result<()> {
        let settings: TrojanSettings = profile.settings_as()?;
        common::check_password(&settings.password)?;
        common::check_transport(&settings.transport)
    }

    fn outbound(&self, profile: &Profile) -> Value {
        let settings: TrojanSettings = profile.settings_as().unwrap_or_default();
        let mut outbound = common::base_outbound("trojan", profile, &settings.tls, &settings.transport);
        outbound["password"] = json!(settings.password);
        outbound
    }

    fn identity(&self, profile: &Profile) -> String {
        let settings: TrojanSettings = profile.settings_as().unwrap_or_default();
        format!("{}|{}|{}", profile.server, profile.port, settings.password)
    }

    fn secrets(&self, profile: &Profile) -> Vec<Secret> {
        let settings: TrojanSettings = profile.settings_as().unwrap_or_default();
        common::secrets(&[(&settings.password, "[password]")], &settings.tls, Some(&settings.transport))
    }

    fn summary(&self, profile: &Profile) -> String {
        let settings: TrojanSettings = profile.settings_as().unwrap_or_default();
        format!("Trojan · {}", common::transport_name(settings.transport.kind))
    }

    fn form(&self) -> Vec<Field> {
        let mut fields = vec![Field::secret("password", "Password", Section::Connection), common::sni_field(), common::insecure_field()];
        fields.extend(common::transport_fields());
        fields.push(common::fingerprint_field());
        fields.push(common::alpn_field());
        fields
    }

    fn blank(&self) -> Profile {
        let settings = TrojanSettings { tls: Tls { fingerprint: Some("chrome".into()), ..Tls::default() }, ..TrojanSettings::default() };
        Profile::build(ID, String::new(), String::new(), common::DEFAULT_PORT, &settings)
    }
}

fn parse(link: &str) -> Result<Profile> {
    let url = Url::parse(link.trim()).map_err(|e| invalid(format!("not a valid link: {e}")))?;
    let password = common::credential_of(&url);
    if password.is_empty() {
        return Err(invalid("link has no password"));
    }
    let server = common::server_of(&url)?;
    let params = Params::of(&url);
    match params.get("security").as_deref() {
        None | Some("tls") => {}
        Some(other) => return Err(invalid(format!("Trojan security '{other}' is not supported, only tls"))),
    }

    let settings = TrojanSettings { password, tls: common::tls_of(&params), transport: common::transport_of(&params)? };
    Ok(Profile::build(
        ID,
        common::name_of(url.fragment(), &server),
        server,
        url.port().unwrap_or(common::DEFAULT_PORT),
        &settings,
    ))
}

fn to_link(profile: &Profile) -> String {
    let settings: TrojanSettings = profile.settings_as().unwrap_or_default();
    let mut query = common::query();
    query.append_pair("security", "tls");
    common::push_tls_and_transport(&mut query, &settings.tls, &settings.transport);
    common::build_link("trojan", &common::encode_userinfo(&settings.password), profile, &query.finish())
}

#[cfg(test)]
#[path = "../tests/profile_trojan.rs"]
mod tests;
