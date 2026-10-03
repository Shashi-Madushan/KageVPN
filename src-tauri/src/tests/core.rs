use super::*;

#[test]
fn parses_version_and_tags() {
    let text = "sing-box version 1.14.2\n\nEnvironment: go1.26 darwin/arm64\nTags: with_clash_api,with_utls\n";
    let (version, tags) = parse_version_output(text).unwrap();
    assert_eq!(version, "1.14.2");
    assert_eq!(tags, ["with_clash_api", "with_utls"]);
}

#[test]
fn rejects_garbage_output() {
    assert!(parse_version_output("hello world").is_none());
}

fn release(version: &str) -> releases::Release {
    releases::Release {
        version: version.into(),
        prerelease: false,
        published_at: None,
        notes: String::new(),
        size: 0,
        url: String::new(),
        sha256: None,
    }
}

#[test]
fn newest_release_is_the_highest_version_not_the_first() {
    let picked = commands::newest(vec![release("1.9.7"), release("1.12.10"), release("1.12.9")]).unwrap();
    assert_eq!(picked.version, "1.12.10");
    assert!(commands::newest(vec![]).is_none());
}

#[test]
fn auto_install_only_replaces_a_managed_engine_on_the_stable_channel() {
    use UpdatePolicy::*;
    assert!(should_auto_install(Auto, Channel::Stable, Source::Managed));
    assert!(!should_auto_install(Auto, Channel::Stable, Source::System), "never touch a Homebrew engine");
    assert!(!should_auto_install(Auto, Channel::Prerelease, Source::Managed));
    assert!(!should_auto_install(Ask, Channel::Stable, Source::Managed));
    assert!(!should_auto_install(Off, Channel::Stable, Source::Managed));
}

#[test]
fn update_policy_defaults_to_ask_and_old_settings_still_load() {
    let settings: CoreSettings = serde_json::from_str(r#"{"managedVersion":"1.12.0","autoCheck":true}"#).unwrap();
    assert_eq!(settings.update_policy, UpdatePolicy::Ask);
    assert_eq!(settings.pending_version, None);
}

#[test]
fn pending_version_becomes_active_only_if_it_is_on_disk() {
    let dir = std::env::temp_dir().join(format!("sbgui-pending-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let paths = AppPaths::new(dir.clone()).unwrap();
    let binary = managed_binary(&paths, "1.13.0");
    std::fs::create_dir_all(binary.parent().unwrap()).unwrap();
    std::fs::write(&binary, b"x").unwrap();

    let mut settings = CoreSettings { pending_version: Some("1.13.0".into()), ..Default::default() };
    assert!(apply_pending(&mut settings, &paths));
    assert_eq!(settings.managed_version.as_deref(), Some("1.13.0"));
    assert_eq!(settings.pending_version, None);

    let mut missing = CoreSettings { pending_version: Some("9.9.9".into()), managed_version: Some("1.13.0".into()), ..Default::default() };
    assert!(apply_pending(&mut missing, &paths));
    assert_eq!(missing.managed_version.as_deref(), Some("1.13.0"), "keeps the working engine");
    assert!(!apply_pending(&mut missing, &paths));
    let _ = std::fs::remove_dir_all(dir);
}
