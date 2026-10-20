use super::*;
use crate::app_config::SING_BOX;

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
    let picked = provider::newest(vec![release("1.9.7"), release("1.12.10"), release("1.12.9")]).unwrap();
    assert_eq!(picked.version, "1.12.10");
    assert!(provider::newest(vec![]).is_none());
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
    let binary = managed_binary(&paths, &SING_BOX, "1.13.0");
    std::fs::create_dir_all(binary.parent().unwrap()).unwrap();
    std::fs::write(&binary, b"x").unwrap();

    let mut settings = CoreSettings { pending_version: Some("1.13.0".into()), ..Default::default() };
    assert!(apply_pending(&mut settings, &paths, &SING_BOX));
    assert_eq!(settings.managed_version.as_deref(), Some("1.13.0"));
    assert_eq!(settings.pending_version, None);

    let mut missing = CoreSettings { pending_version: Some("9.9.9".into()), managed_version: Some("1.13.0".into()), ..Default::default() };
    assert!(apply_pending(&mut missing, &paths, &SING_BOX));
    assert_eq!(missing.managed_version.as_deref(), Some("1.13.0"), "keeps the working engine");
    assert!(!apply_pending(&mut missing, &paths, &SING_BOX));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn old_download_folders_move_under_the_core_id_and_keep_working() {
    let dir = std::env::temp_dir().join(format!("sbgui-layout-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let paths = AppPaths::new(dir.clone()).unwrap();
    let old = paths.core_dir().join("1.12.4");
    std::fs::create_dir_all(&old).unwrap();
    std::fs::write(old.join("sing-box"), b"x").unwrap();

    migrate_layout(&paths, &SING_BOX);

    assert!(managed_binary(&paths, &SING_BOX, "1.12.4").is_file());
    assert!(!old.exists());
    assert_eq!(installed(&paths, &SING_BOX).len(), 1);
    migrate_layout(&paths, &SING_BOX); // running again changes nothing
    assert_eq!(installed(&paths, &SING_BOX).len(), 1);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn every_core_spec_is_well_formed() {
    use crate::app_config::{core_spec, default_core, CORES};
    let mut ids = std::collections::HashSet::new();
    for spec in CORES {
        assert!(ids.insert(spec.id), "duplicate core id {}", spec.id);
        assert!(Version::parse(spec.min_version).is_some(), "{} has a bad min_version", spec.id);
        assert!(spec.releases_api.starts_with("https://"));
        let asset = (spec.asset_name)("1.2.3", "arm64");
        assert!(asset.contains("1.2.3") && asset.contains("arm64"), "{asset}");
        assert_eq!(core_spec(spec.id).unwrap().id, spec.id);
    }
    assert!(core_spec("missing").is_none());
    assert_eq!(default_core().id, CORES[0].id);
}

fn fake_versions(name: &str, versions: &[&str]) -> (AppPaths, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("sbgui-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let paths = AppPaths::new(dir.clone()).unwrap();
    for version in versions {
        let binary = managed_binary(&paths, &SING_BOX, version);
        std::fs::create_dir_all(binary.parent().unwrap()).unwrap();
        std::fs::write(&binary, b"x").unwrap();
    }
    (paths, dir)
}

#[test]
fn activating_a_version_remembers_the_one_it_replaces() {
    let mut settings = CoreSettings::default();
    settings.activate(Some("1.12.0".into()));
    assert_eq!(settings.previous_version, None, "nothing to remember the first time");
    settings.activate(Some("1.13.0".into()));
    assert_eq!(settings.previous_version.as_deref(), Some("1.12.0"));
    settings.activate(Some("1.13.0".into()));
    assert_eq!(settings.previous_version.as_deref(), Some("1.12.0"), "re-activating the same one changes nothing");
}

#[test]
fn good_is_marked_only_for_the_active_managed_version() {
    let mut settings = CoreSettings::default();
    assert!(!settings.mark_good(), "a system engine is not managed");
    settings.activate(Some("1.12.0".into()));
    assert!(settings.mark_good());
    assert!(!settings.mark_good(), "already marked");
    assert_eq!(settings.good_version.as_deref(), Some("1.12.0"));
}

#[test]
fn rollback_prefers_the_good_version_and_needs_it_on_disk() {
    let mut settings = CoreSettings::default();
    settings.activate(Some("1.12.0".into()));
    settings.mark_good();
    settings.activate(Some("1.13.0".into()));
    let both = vec!["1.12.0".to_string(), "1.13.0".to_string()];
    assert_eq!(settings.rollback_target(&both).as_deref(), Some("1.12.0"));
    assert_eq!(settings.rollback_target(&["1.13.0".to_string()]), None, "the target was deleted");
    // Good is the active one: fall back to the one used before.
    settings.activate(Some("1.14.0".into()));
    settings.mark_good();
    let all = vec!["1.12.0".to_string(), "1.13.0".to_string(), "1.14.0".to_string()];
    assert_eq!(settings.rollback_target(&all).as_deref(), Some("1.13.0"));
}

#[test]
fn removing_old_versions_keeps_the_active_one_and_the_rollback_target() {
    let (paths, dir) = fake_versions("removeold", &["1.11.0", "1.12.0", "1.13.0"]);
    let mut settings = CoreSettings::default();
    settings.activate(Some("1.12.0".into()));
    settings.mark_good();
    settings.activate(Some("1.13.0".into()));
    let removed = remove_old(&paths, &SING_BOX, &settings).unwrap();
    assert_eq!(removed, vec!["1.11.0".to_string()]);
    let left: Vec<String> = installed(&paths, &SING_BOX).into_iter().map(|i| i.version).collect();
    assert_eq!(left, vec!["1.13.0".to_string(), "1.12.0".to_string()]);
    let _ = std::fs::remove_dir_all(dir);
}
