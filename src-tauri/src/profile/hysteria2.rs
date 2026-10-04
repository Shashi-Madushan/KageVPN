use serde_json::{json, Value};
use url::Url;
use uuid::Uuid;

use super::common::{self, invalid, Params};
use super::form::{Field, Section};
use super::{Obfs, Profile, ProtocolDef, ProtocolId, Secret, Security};
use crate::error::Result;

pub const ID: ProtocolId = ProtocolId("hysteria2");

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
        validate(profile)
    }

    fn outbound(&self, profile: &Profile) -> Value {
        outbound(profile)
    }

    fn identity(&self, profile: &Profile) -> String {
        format!("{}|{}|{}", profile.server, profile.port, profile.password)
    }

    fn secrets(&self, profile: &Profile) -> Vec<Secret> {
        let mut secrets = common::secrets(profile, &[(&profile.password, "[password]")]);
        if let Some(obfs) = &profile.obfs {
            secrets.push(Secret::new(obfs.password.clone(), "[password]"));
        }
        secrets
    }

    fn summary(&self, _profile: &Profile) -> String {
        "Hysteria2".to_string()
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
        Profile { protocol: ID, port: common::DEFAULT_PORT, fingerprint: None, ..Profile::default() }
    }

    fn normalize(&self, profile: &mut Profile) {
        profile.obfs = profile.obfs.take().filter(|o| !o.password.is_empty()).map(|o| Obfs { kind: OBFS_SALAMANDER.into(), password: o.password });
    }
}

const OBFS_SALAMANDER: &str = "salamander";

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

    Ok(Profile {
        id: Uuid::new_v4().to_string(),
        protocol: ID,
        name: common::name_of(url.fragment(), &server),
        server,
        port: url.port().unwrap_or(common::DEFAULT_PORT),
        password,
        security: Security::Tls,
        sni: params.first(&["sni", "peer"]),
        alpn: common::alpn_of(&params),
        allow_insecure: params.flag(&["insecure", "allowInsecure"]),
        obfs,
        ..Profile::default()
    })
}

fn to_link(profile: &Profile) -> String {
    let mut query = common::query();
    common::push(&mut query, "sni", &profile.sni);
    if let Some(obfs) = &profile.obfs {
        query.append_pair("obfs", &obfs.kind);
        query.append_pair("obfs-password", &obfs.password);
    }
    if profile.allow_insecure {
        query.append_pair("insecure", "1");
    }
    common::build_link("hysteria2", &common::encode_userinfo(&profile.password), profile, &query.finish())
}

fn validate(profile: &Profile) -> Result<()> {
    common::check_password(profile)?;
    if profile.obfs.as_ref().is_some_and(|o| o.password.trim().is_empty()) {
        return Err(invalid("obfuscation needs a password"));
    }
    Ok(())
}

fn outbound(profile: &Profile) -> Value {
    // QUIC carries its own transport, and browser fingerprints and Reality do not apply to it.
    let mut outbound = json!({
        "type": "hysteria2",
        "server": profile.server,
        "server_port": profile.port,
        "password": profile.password,
        "tls": common::tls_base(profile),
    });
    if let Some(obfs) = &profile.obfs {
        outbound["obfs"] = json!({ "type": obfs.kind, "password": obfs.password });
    }
    outbound
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
