use super::*;
use crate::profile::{parse_many, Profile};
use serde_json::json;

fn profile(id: &str) -> Profile {
    let mut profile = parse_many("vless://11111111-2222-4333-8444-555555555555@h.example.com:443?security=tls").0.remove(0);
    profile.id = id.into();
    profile.name = id.into();
    profile
}

fn temp_path(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sbg-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir.join("data.json")
}

#[test]
fn profiles_and_selection_survive_a_restart() {
    let path = temp_path("restart");
    let mut store = Store::load(path.clone());
    store.add(profile("a")).unwrap();
    store.add(profile("b")).unwrap();
    store.select("b").unwrap();
    store.set_mode(Mode::Proxy).unwrap();

    let reloaded = Store::load(path.clone());
    assert_eq!(reloaded.profiles().len(), 2);
    assert_eq!(reloaded.active_id(), Some("b"));
    assert_eq!(reloaded.mode(), Mode::Proxy);
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn removing_the_active_profile_selects_another() {
    let path = temp_path("remove");
    let mut store = Store::load(path.clone());
    store.add(profile("a")).unwrap();
    store.add(profile("b")).unwrap();
    store.remove("a").unwrap();
    assert_eq!(store.active_id(), Some("b"));
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn unreadable_file_is_kept_aside_not_overwritten() {
    let path = temp_path("corrupt");
    let mut store = Store::load(path.clone());
    store.add(profile("a")).unwrap();
    let good = fs::read(&path).unwrap();
    fs::write(&path, &good[..good.len() / 2]).unwrap();

    let mut broken = Store::load(path.clone());
    assert!(broken.profiles().is_empty());
    let saved = broken.take_recovery().expect("recovery path");
    assert_eq!(fs::read(&saved).unwrap(), &good[..good.len() / 2]);
    assert!(broken.take_recovery().is_none(), "reported once");

    // The next corruption must not clobber the first saved copy.
    broken.add(profile("b")).unwrap();
    fs::write(&path, b"{ nope").unwrap();
    let mut again = Store::load(path.clone());
    let second = again.take_recovery().unwrap();
    assert_ne!(second, saved);
    assert!(saved.exists());
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn missing_file_starts_empty_without_a_warning() {
    let path = temp_path("missing");
    let mut store = Store::load(path.clone());
    assert!(store.profiles().is_empty());
    assert!(store.take_recovery().is_none());
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[cfg(unix)]
#[test]
fn data_file_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let path = temp_path("mode");
    Store::load(path.clone()).add(profile("a")).unwrap();
    assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn import_skips_profiles_that_are_already_saved() {
    let path = temp_path("dupes");
    let mut store = Store::load(path.clone());
    let mut again = profile("x");
    again.name = "same server, other name".into();
    let mut other_transport = profile("y");
    other_transport.set_setting("transport.path", json!("/ws"));

    let (added, skipped) = store.import_unique(vec![profile("a"), again.clone(), other_transport]).unwrap();
    assert_eq!((added, skipped), (2, 1), "duplicate inside one paste");
    let (added, skipped) = store.import_unique(vec![again]).unwrap();
    assert_eq!((added, skipped), (0, 1), "duplicate of a saved profile");
    assert_eq!(store.profiles().len(), 2);
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn single_core_settings_from_schema_two_move_under_the_default_core() {
    let path = temp_path("core-migrate");
    fs::write(
        &path,
        r#"{"schemaVersion":2,"core":{"managedVersion":"1.12.0","channel":"prerelease","autoCheck":false}}"#,
    )
    .unwrap();

    let store = Store::load(path.clone());
    let settings = store.core_settings();
    assert_eq!(settings.managed_version.as_deref(), Some("1.12.0"));
    assert_eq!(settings.channel, crate::core::Channel::Prerelease);
    assert_eq!(store.active_core().id, "sing-box");

    let saved = fs::read_to_string(&path).unwrap();
    assert!(!saved.contains("\"core\":"), "the legacy key is dropped once migrated: {saved}");
    assert!(saved.contains("\"cores\""));
}

#[test]
fn unknown_core_ids_are_rejected_and_a_stale_saved_id_falls_back() {
    let path = temp_path("core-unknown");
    let mut store = Store::load(path.clone());
    assert!(store.set_active_core("nope").is_err());
    drop(store);

    fs::write(&path, r#"{"schemaVersion":3,"cores":{"active":"removed-core"}}"#).unwrap();
    assert_eq!(Store::load(path).active_core().id, "sing-box");
}

#[test]
fn servers_saved_in_the_old_layout_are_set_aside_not_loaded_half_working() {
    let path = temp_path("v3");
    let old = serde_json::json!({
        "schemaVersion": 3,
        "profiles": [{ "id": "old", "name": "old", "server": "h.example.com", "port": 443, "uuid": "11111111-2222-4333-8444-555555555555", "security": "tls" }],
        "activeId": "old"
    });
    fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();

    let store = Store::load(path.clone());
    assert!(store.profiles().is_empty(), "an old-layout server has no settings to run");
    assert_eq!(store.active_id(), None);
    assert!(path.with_file_name("data.v3.json").is_file(), "the old file is kept as a backup");
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn servers_in_the_current_layout_survive_the_upgrade_check() {
    let path = temp_path("v4");
    let mut store = Store::load(path.clone());
    store.add(profile("a")).unwrap();
    assert_eq!(Store::load(path.clone()).profiles().len(), 1);
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[cfg(unix)]
#[test]
fn a_file_that_cannot_be_read_is_moved_aside_not_replaced_by_an_empty_store() {
    use std::os::unix::fs::PermissionsExt;
    let path = temp_path("unreadable");
    let mut store = Store::load(path.clone());
    store.add(profile("a")).unwrap();
    let good = fs::read(&path).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();

    let mut reloaded = Store::load(path.clone());
    assert!(reloaded.profiles().is_empty());
    let saved = reloaded.take_recovery().expect("the unreadable file is kept, not forgotten");
    fs::set_permissions(&saved, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(fs::read(&saved).unwrap(), good, "the user's servers are still on disk");
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[cfg(unix)]
#[test]
fn a_file_that_can_be_neither_read_nor_moved_is_never_overwritten() {
    use std::os::unix::fs::PermissionsExt;
    let path = temp_path("locked");
    let dir = path.parent().unwrap().to_path_buf();
    let mut store = Store::load(path.clone());
    store.add(profile("a")).unwrap();
    let good = fs::read(&path).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o500)).unwrap();

    let mut locked = Store::load(path.clone());
    let refused = locked.add(profile("b")).is_err();

    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(refused, "saving over an unreadable file must be refused");
    assert_eq!(fs::read(&path).unwrap(), good);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_file_from_a_newer_version_is_backed_up_before_this_one_saves_over_it() {
    let path = temp_path("newer");
    let newer = format!(r#"{{"schemaVersion":{},"futureSetting":{{"kept":true}}}}"#, SCHEMA_VERSION + 1);
    fs::write(&path, &newer).unwrap();

    let mut store = Store::load(path.clone());
    store.set_mode(Mode::Proxy).unwrap();

    let backup = path.with_file_name(format!("data.v{}.json", SCHEMA_VERSION + 1));
    assert_eq!(fs::read_to_string(backup).unwrap(), newer);
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

fn data_with_an_unknown_protocol(path: &std::path::Path) -> serde_json::Value {
    let mut store = Store::load(path.to_path_buf());
    store.add(profile("a")).unwrap();
    store.set_mode(Mode::Proxy).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let mut future = value["profiles"][0].clone();
    future["id"] = json!("future");
    future["protocol"] = json!("tuic-from-a-newer-version");
    value["profiles"].as_array_mut().unwrap().push(future.clone());
    fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    future
}

#[test]
fn one_server_this_version_cannot_read_does_not_hide_everything_else() {
    let path = temp_path("unknown-protocol");
    let future = data_with_an_unknown_protocol(&path);

    let mut store = Store::load(path.clone());
    assert!(store.take_recovery().is_none(), "the file is not treated as damaged");
    assert_eq!(store.profiles().len(), 1, "the readable server loads");
    assert_eq!(store.mode(), Mode::Proxy, "and so do the settings");
    assert_eq!(store.unreadable_count(), 1);

    // Saving must not lose the server it cannot read.
    store.add(profile("b")).unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["unreadableProfiles"], json!([future]));
    assert_eq!(saved["profiles"].as_array().unwrap().len(), 2);
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn a_server_kept_aside_returns_once_it_becomes_readable() {
    let path = temp_path("unknown-protocol-returns");
    data_with_an_unknown_protocol(&path);
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    // The newer build is installed again: the same entry now names a protocol that exists.
    value["profiles"][1]["protocol"] = json!("vless");
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(Store::load(path.clone()).profiles().len(), 2);

    // And a server saved under "unreadableProfiles" earlier is picked up the same way.
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let entry = value["profiles"].as_array_mut().unwrap().remove(1);
    value["unreadableProfiles"] = json!([entry]);
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    let store = Store::load(path.clone());
    assert_eq!((store.profiles().len(), store.unreadable_count()), (2, 0));
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn a_file_that_is_not_a_data_file_at_all_is_still_set_aside() {
    let path = temp_path("not-json");
    fs::write(&path, b"{ nope").unwrap();
    let mut store = Store::load(path.clone());
    assert!(store.take_recovery().is_some());
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn removing_several_servers_keeps_a_valid_selection() {
    let dir = std::env::temp_dir().join(format!("kage-store-remove-many-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut store = Store::load(dir.join("data.json"));
    store.add_many(crate::profile::parse_many("trojan://a@a.example.com:443#A\ntrojan://b@b.example.com:443#B\ntrojan://c@c.example.com:443#C").0).unwrap();
    let ids: Vec<String> = store.profiles().iter().map(|p| p.id.clone()).collect();
    store.select(&ids[0]).unwrap();

    assert_eq!(store.remove_many(&[ids[0].clone(), ids[1].clone(), "unknown".into()]).unwrap(), 2);
    assert_eq!(store.profiles().len(), 1);
    assert_eq!(store.active_id(), Some(ids[2].as_str()), "the selection moves to a server that still exists");
    assert_eq!(store.remove_many(&[]).unwrap(), 0);
    let _ = std::fs::remove_dir_all(dir);
}
