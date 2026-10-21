//! SSH as a proxy: sing-box opens an SSH session and forwards connections through it.

use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use url::Url;

use super::common::{self, invalid, Params};
use super::form::{Field, Section};
use super::{Profile, ProtocolDef, ProtocolId, Secret, ServerKind};
use crate::error::Result;

pub const ID: ProtocolId = ProtocolId("ssh");

pub const DEFAULT_PORT: u16 = 22;
const HOST_KEY_WARNING: &str =
    "Without a host key, anyone who can intercept your connection could pretend to be this server. Ask your provider for the server's public host key.";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SshSettings {
    pub user: String,
    #[serde(deserialize_with = "common::null_as_empty")]
    pub password: String,
    /// The key text (PEM or OpenSSH), pasted in; key files on disk are not used.
    #[serde(deserialize_with = "common::null_as_empty")]
    pub private_key: String,
    #[serde(deserialize_with = "common::null_as_empty")]
    pub passphrase: String,
    /// Public host keys, each as `algorithm base64`. Empty means the server is not checked.
    pub host_keys: Vec<String>,
}

pub struct Ssh;

impl ProtocolDef for Ssh {
    fn id(&self) -> &'static str {
        ID.as_str()
    }

    fn label(&self) -> &'static str {
        "SSH"
    }

    fn schemes(&self) -> &'static [&'static str] {
        &["ssh"]
    }

    fn over_udp(&self) -> bool {
        false
    }

    fn kind(&self) -> ServerKind {
        ServerKind::Outbound
    }

    fn feature(&self) -> Option<&'static str> {
        None
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
        let settings: SshSettings = profile.settings_as().unwrap_or_default();
        let mut outbound = json!({
            "type": "ssh",
            "server": profile.server,
            "server_port": profile.port,
            "user": settings.user,
        });
        if !settings.password.is_empty() {
            outbound["password"] = json!(settings.password);
        }
        if !settings.private_key.is_empty() {
            outbound["private_key"] = json!(settings.private_key);
            if !settings.passphrase.is_empty() {
                outbound["private_key_passphrase"] = json!(settings.passphrase);
            }
        }
        if !settings.host_keys.is_empty() {
            outbound["host_key"] = json!(settings.host_keys);
        }
        outbound
    }

    fn identity(&self, profile: &Profile) -> String {
        let settings: SshSettings = profile.settings_as().unwrap_or_default();
        format!("{}|{}|{}", profile.server, profile.port, settings.user)
    }

    fn secrets(&self, profile: &Profile) -> Vec<Secret> {
        // Host keys are public; the user name is not a credential on its own.
        let settings: SshSettings = profile.settings_as().unwrap_or_default();
        [(settings.password, "[password]"), (settings.private_key, "[key]"), (settings.passphrase, "[password]")]
            .into_iter()
            .filter(|(value, _)| !value.is_empty())
            .map(|(value, label)| Secret::new(value, label))
            .collect()
    }

    fn summary(&self, profile: &Profile) -> String {
        let settings: SshSettings = profile.settings_as().unwrap_or_default();
        let login = if settings.private_key.is_empty() { "password" } else { "key" };
        let checked = if settings.host_keys.is_empty() { " · host not checked" } else { "" };
        format!("SSH · {login}{checked}")
    }

    fn sample(&self) -> &'static str {
        "ssh://root:sample-password@ssh.example.com:22?host_key=ssh-ed25519%20AAAAC3NzaC1lZDI1NTE5AAAAIGE58h0urP4jPA6D6Ee%2BUnCVCbe6FbFtJbHa8Nh7CW%2Fr#Sample%20SSH"
    }

    fn form(&self) -> Vec<Field> {
        vec![
            Field::text("user", "User name", Section::Connection).mono(),
            Field::secret("password", "Password", Section::Connection).optional(),
            Field::multiline("privateKey", "Private key", Section::Security)
                .optional()
                .placeholder("-----BEGIN OPENSSH PRIVATE KEY-----"),
            Field::list("hostKeys", "Server host key", Section::Security)
                .optional()
                .placeholder("ssh-ed25519 AAAA…")
                .warning_when_empty(HOST_KEY_WARNING),
            Field::secret("passphrase", "Key passphrase", Section::Advanced).optional(),
        ]
    }

    fn blank(&self) -> Profile {
        Profile::build(ID, String::new(), String::new(), DEFAULT_PORT, &SshSettings::default())
    }

    fn normalize(&self, profile: &mut Profile) {
        let Ok(mut settings) = profile.settings_as::<SshSettings>() else { return };
        settings.user = settings.user.trim().to_string();
        settings.private_key = tidy_key(&settings.private_key);
        settings.host_keys = settings.host_keys.iter().map(|k| k.split_whitespace().collect::<Vec<_>>().join(" ")).filter(|k| !k.is_empty()).collect();
        if settings.private_key.is_empty() {
            settings.passphrase.clear();
        }
        profile.put_settings(&settings);
    }
}

/// Unix line endings, no stray blank lines around it, and the final newline SSH tools expect.
pub fn tidy_key(key: &str) -> String {
    let key = key.replace("\r\n", "\n").replace('\r', "\n");
    let key = key.trim();
    if key.is_empty() { String::new() } else { format!("{key}\n") }
}

fn check(settings: &SshSettings) -> Result<()> {
    if settings.user.trim().is_empty() {
        return Err(invalid("SSH needs a user name"));
    }
    if settings.password.is_empty() && settings.private_key.trim().is_empty() {
        return Err(invalid("SSH needs a password or a private key"));
    }
    if !settings.private_key.trim().is_empty() && !looks_like_private_key(&settings.private_key) {
        return Err(invalid("the private key must be the whole key text, from -----BEGIN to -----END"));
    }
    for key in &settings.host_keys {
        check_host_key(key)?;
    }
    Ok(())
}

fn looks_like_private_key(key: &str) -> bool {
    let key = key.trim();
    key.starts_with("-----BEGIN ") && key.contains("PRIVATE KEY-----") && key.ends_with("-----")
}

fn check_host_key(key: &str) -> Result<()> {
    if key.trim_start().to_ascii_uppercase().starts_with("SHA256:") || key.trim_start().to_ascii_uppercase().starts_with("MD5:") {
        return Err(invalid("host key must be the full public key line, not a fingerprint"));
    }
    let mut parts = key.split_whitespace();
    let algorithm = parts.next().unwrap_or_default();
    let body = parts.next().unwrap_or_default();
    let known = algorithm.starts_with("ssh-") || algorithm.starts_with("ecdsa-") || algorithm.starts_with("sk-");
    let base64 = !body.is_empty() && body.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '='));
    if !known || !base64 {
        return Err(invalid(format!("host key '{}' is not a public key line such as 'ssh-ed25519 AAAA…'", shorten(key))));
    }
    Ok(())
}

fn shorten(text: &str) -> String {
    let short: String = text.chars().take(24).collect();
    if short.len() < text.len() { format!("{short}…") } else { short }
}

fn decoded(text: &str) -> String {
    percent_decode_str(text).decode_utf8_lossy().into_owned()
}

fn parse(link: &str) -> Result<Profile> {
    let url = Url::parse(link.trim()).map_err(|e| invalid(format!("not a valid link: {e}")))?;
    let server = common::server_of(&url)?;
    let params = Params::of(&url);
    let settings = SshSettings {
        user: decoded(url.username()),
        password: url.password().map(decoded).unwrap_or_default(),
        private_key: tidy_key(&params.first(&["private_key", "pk"]).unwrap_or_default()),
        passphrase: params.first(&["passphrase", "pkp"]).unwrap_or_default(),
        host_keys: params.first(&["host_key", "hk"]).map(|keys| keys.split(',').map(|k| k.trim().to_string()).filter(|k| !k.is_empty()).collect()).unwrap_or_default(),
    };
    check(&settings)?;
    Ok(Profile::build(ID, common::name_of(url.fragment(), &server), server, url.port().unwrap_or(DEFAULT_PORT), &settings))
}

fn to_link(profile: &Profile) -> String {
    let settings: SshSettings = profile.settings_as().unwrap_or_default();
    let mut query = common::query();
    if !settings.private_key.is_empty() {
        query.append_pair("private_key", &settings.private_key);
    }
    if !settings.passphrase.is_empty() {
        query.append_pair("passphrase", &settings.passphrase);
    }
    if !settings.host_keys.is_empty() {
        query.append_pair("host_key", &settings.host_keys.join(","));
    }
    let mut userinfo = utf8_percent_encode(&settings.user, NON_ALPHANUMERIC).to_string();
    if !settings.password.is_empty() {
        userinfo = format!("{userinfo}:{}", common::encode_userinfo(&settings.password));
    }
    common::build_link("ssh", &userinfo, profile, &query.finish())
}

#[cfg(test)]
#[path = "../tests/profile_ssh.rs"]
mod tests;
