use super::*;

fn cfg(profile: &Profile) -> SshSettings {
    profile.settings_as().unwrap()
}

const HOST_KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGE58h0urP4jPA6D6Ee+UnCVCbe6FbFtJbHa8Nh7CW/r";
/// Key-shaped text made when the test runs; it is never parsed as a real key.
fn key() -> String {
    crate::profile::fake_private_key("AAAAB3Nz
C1yc2EAAAADAQAB", "
")
}

fn key_link() -> String {
    let mut query = common::query();
    query.append_pair("private_key", &key());
    query.append_pair("passphrase", "open sesame");
    query.append_pair("host_key", &format!("{HOST_KEY},ecdsa-sha2-nistp256 AAAAE2VjZHNh"));
    format!("ssh://deploy@[2001:db8::7]:2222?{}#Key%20Box", query.finish())
}

#[test]
fn parses_a_password_link_with_the_default_port() {
    let p = parse("ssh://root:p%40ss@ssh.example.com#Box").unwrap();
    assert_eq!((p.protocol.as_str(), p.port, p.name.as_str()), ("ssh", 22, "Box"));
    let s = cfg(&p);
    assert_eq!((s.user.as_str(), s.password.as_str()), ("root", "p@ss"));
    assert!(s.private_key.is_empty() && s.host_keys.is_empty());
}

#[test]
fn parses_a_key_passphrase_and_host_keys_from_the_query() {
    let p = parse(&key_link()).unwrap();
    assert_eq!((p.server.as_str(), p.port), ("2001:db8::7", 2222));
    let s = cfg(&p);
    assert_eq!((s.user.as_str(), s.password.as_str(), s.passphrase.as_str()), ("deploy", "", "open sesame"));
    assert_eq!(s.private_key, key(), "line breaks survive");
    assert_eq!(s.host_keys, [HOST_KEY, "ecdsa-sha2-nistp256 AAAAE2VjZHNh"]);
}

#[test]
fn round_trips_through_its_own_link() {
    for link in [key_link(), "ssh://root:pw@ssh.example.com:22#P".to_string()] {
        let first = parse(&link).unwrap();
        let again = parse(&to_link(&first)).unwrap();
        assert!(first.same_endpoint(&again), "{link}");
        assert_eq!(cfg(&first), cfg(&again), "{link}");
    }
}

#[test]
fn rejects_what_it_cannot_run_and_says_why() {
    for (link, why) in [
        ("ssh://ssh.example.com:22", "SSH needs a user name"),
        ("ssh://root@ssh.example.com:22", "SSH needs a password or a private key"),
        ("ssh://root@ssh.example.com:22?pk=AAAAB3NzaC1yc2E", "the private key must be the whole key text, from -----BEGIN to -----END"),
        ("ssh://root:pw@ssh.example.com:22?hk=SHA256:abcdef", "host key must be the full public key line, not a fingerprint"),
        ("ssh://root:pw@ssh.example.com:22?hk=AAAAC3NzaC1lZDI1NTE5", "host key 'AAAAC3NzaC1lZDI1NTE5' is not a public key line such as 'ssh-ed25519 AAAA…'"),
    ] {
        assert_eq!(parse(link).unwrap_err().to_string(), why, "{link}");
    }
}

#[test]
fn the_list_warns_when_the_host_is_not_checked() {
    assert_eq!(Ssh.summary(&parse("ssh://root:pw@ssh.example.com#P").unwrap()), "SSH · password · host not checked");
    assert_eq!(Ssh.summary(&parse(&key_link()).unwrap()), "SSH · key");
}

#[test]
fn outbound_has_only_what_is_set() {
    let password = Ssh.outbound(&parse("ssh://root:pw@ssh.example.com#P").unwrap());
    assert_eq!((password["type"].as_str(), password["user"].as_str(), password["password"].as_str()), (Some("ssh"), Some("root"), Some("pw")));
    assert!(password.get("private_key").is_none() && password.get("host_key").is_none() && password.get("tls").is_none());

    let key = Ssh.outbound(&parse(&key_link()).unwrap());
    assert_eq!(key["private_key"], self::key());
    assert_eq!(key["private_key_passphrase"], "open sesame");
    assert_eq!(key["host_key"][0], HOST_KEY);
    assert!(key.get("password").is_none());
}

#[test]
fn normalize_tidies_pasted_keys_and_empty_editor_fields() {
    let mut p = parse("ssh://root:pw@ssh.example.com#P").unwrap();
    p.settings.insert("privateKey".into(), serde_json::json!(format!("\r\n{}\r\n", crate::profile::fake_private_key("AAAA", "\r\n"))));
    p.settings.insert("hostKeys".into(), serde_json::json!(["  ssh-ed25519   AAAAC3Nz  ", ""]));
    p.settings.insert("passphrase".into(), serde_json::Value::Null);
    Ssh.normalize(&mut p);
    let s = cfg(&p);
    assert_eq!(s.private_key, crate::profile::fake_private_key("AAAA", "\n"));
    assert_eq!(s.host_keys, ["ssh-ed25519 AAAAC3Nz"]);

    // The editor sends null for optional fields left empty, and a passphrase without a key goes.
    p.settings.insert("privateKey".into(), serde_json::Value::Null);
    p.settings.insert("passphrase".into(), serde_json::json!("left over"));
    Ssh.normalize(&mut p);
    assert_eq!((cfg(&p).private_key.as_str(), cfg(&p).passphrase.as_str()), ("", ""));
    assert!(p.validate().is_ok());
}

#[test]
fn secrets_cover_the_password_key_and_passphrase_but_not_host_keys() {
    let labels: Vec<_> = Ssh.secrets(&parse(&key_link()).unwrap()).into_iter().map(|s| s.value).collect();
    assert_eq!(labels, [key(), "open sesame".to_string()]);
}
