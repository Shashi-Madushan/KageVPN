use serde_json::{json, Value};
use url::Url;
use uuid::Uuid;

use super::common::{self, invalid, Params};
use super::{Profile, ProtocolDef, ProtocolId, Secret, Security};
use crate::error::Result;

pub const ID: ProtocolId = ProtocolId("trojan");

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
        validate(profile)
    }

    fn outbound(&self, profile: &Profile) -> Value {
        outbound(profile)
    }

    fn identity(&self, profile: &Profile) -> String {
        format!("{}|{}|{}", profile.server, profile.port, profile.password)
    }

    fn secrets(&self, profile: &Profile) -> Vec<Secret> {
        common::secrets(profile, &[(&profile.password, "[password]")])
    }

    fn summary(&self, profile: &Profile) -> String {
        format!("Trojan · {}", common::transport_name(profile.transport.kind))
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

    Ok(Profile {
        id: Uuid::new_v4().to_string(),
        protocol: ID,
        name: common::name_of(url.fragment(), &server),
        server,
        port: url.port().unwrap_or(common::DEFAULT_PORT),
        password,
        security: Security::Tls,
        sni: params.first(&["sni", "peer"]),
        fingerprint: params.get("fp"),
        alpn: common::alpn_of(&params),
        transport: common::transport_of(&params)?,
        allow_insecure: params.flag(&["allowInsecure", "insecure"]),
        ..Profile::default()
    })
}

fn to_link(profile: &Profile) -> String {
    let mut query = common::query();
    query.append_pair("security", "tls");
    common::push_tls_and_transport(&mut query, profile);
    common::build_link("trojan", &common::encode_userinfo(&profile.password), profile, &query.finish())
}

fn validate(profile: &Profile) -> Result<()> {
    common::check_password(profile)?;
    common::check_transport(profile)
}

fn outbound(profile: &Profile) -> Value {
    let mut outbound = common::base_outbound("trojan", profile);
    outbound["password"] = json!(profile.password);
    outbound
}

#[cfg(test)]
#[path = "../tests/profile_trojan.rs"]
mod tests;
