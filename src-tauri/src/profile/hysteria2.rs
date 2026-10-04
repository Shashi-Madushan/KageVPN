use serde_json::{json, Value};
use url::Url;
use uuid::Uuid;

use super::common::{self, invalid, Params};
use super::{Obfs, Profile, Protocol, ProtocolSpec, Security};
use crate::error::Result;

pub static SPEC: ProtocolSpec = ProtocolSpec {
    protocol: Protocol::Hysteria2,
    schemes: &["hysteria2", "hy2"],
    parse,
    to_link,
    validate,
    outbound,
};

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
        protocol: Protocol::Hysteria2,
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
