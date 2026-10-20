use super::*;
use crate::profile::parse_many;
use crate::settings::{RouteMode, Subscription};
use crate::store::Store;

fn store(label: &str) -> (Store, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("kage-backup-{label}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    (Store::load(dir.join("data.json")), dir)
}

fn setup(store: &mut Store) {
    let links = "trojan://secret@a.example.com:443#Alpha\nhysteria2://pw@b.example.com:443#Beta";
    store.add_many(parse_many(links).0).unwrap();
    let mut routing = store.routing();
    routing.mode = RouteMode::Rule;
    store.set_routing(routing).unwrap();
    store
        .add_subscription(Subscription { id: "s".into(), name: "Provider".into(), url: "https://example.com/sub".into(), send_device_id: true, ..Subscription::default() })
        .unwrap();
}

fn export(store: &Store) -> String {
    envelope(store.backup_value().unwrap(), 1_700_000_000).to_string()
}

#[test]
fn a_backup_restores_the_whole_setup_on_another_mac() {
    let (mut old_mac, old_dir) = store("from");
    setup(&mut old_mac);
    let old_device = old_mac.device_id().unwrap();
    let file = export(&old_mac);
    assert!(!file.contains(&old_device), "the device ID stays on the Mac it was made for");
    assert!(!file.contains("\"cores\""), "installed engines are not part of a backup");

    let (mut new_mac, new_dir) = store("to");
    let new_device = new_mac.device_id().unwrap();
    new_mac.add_many(parse_many("trojan://x@old.example.com:443#Replaced").0).unwrap();

    let (servers, subscriptions) = new_mac.restore(unwrap(&file).unwrap()).unwrap();
    assert_eq!((servers, subscriptions), (2, 1));
    let names: Vec<&str> = new_mac.profiles().iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["Alpha", "Beta"]);
    assert_eq!(new_mac.active_id(), old_mac.active_id(), "server ids survive, so the selection does too");
    assert_eq!(new_mac.routing().mode, RouteMode::Rule);
    assert!(new_mac.subscriptions()[0].send_device_id);
    assert_eq!(new_mac.device_id().unwrap(), new_device, "this Mac keeps its own device ID");

    // What was there before is kept next to the data file, and the restore is on disk.
    assert!(new_dir.join("data.before-restore.json").is_file());
    let reloaded = Store::load(new_dir.join("data.json"));
    assert_eq!(reloaded.profiles().len(), 2);

    let _ = std::fs::remove_dir_all(old_dir);
    let _ = std::fs::remove_dir_all(new_dir);
}

#[test]
fn files_that_are_not_backups_are_refused_without_touching_anything() {
    let (mut target, dir) = store("refuse");
    setup(&mut target);
    for text in ["", "not json", "[]", "{}", r#"{ "kind": "something-else", "data": {} }"#, r#"{ "kind": "kage-vpn-backup" }"#, r#"{ "profiles": [] }"#] {
        assert!(unwrap(text).is_err(), "{text:?} was taken for a backup");
    }
    // An envelope around something without a schema version would read as "no servers".
    let empty = unwrap(r#"{ "kind": "kage-vpn-backup", "version": 1, "data": {} }"#).unwrap();
    assert!(target.restore(empty).is_err());
    assert_eq!(target.profiles().len(), 2, "a refused restore leaves the setup alone");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_backup_from_a_newer_app_is_refused_with_a_reason() {
    let newer_envelope = r#"{ "kind": "kage-vpn-backup", "version": 99, "data": { "schemaVersion": 4, "profiles": [] } }"#;
    assert!(unwrap(newer_envelope).unwrap_err().to_string().contains("newer version"));

    let (mut target, dir) = store("newer");
    let newer_data = serde_json::json!({ "schemaVersion": 99, "profiles": [] });
    assert!(target.restore(newer_data).unwrap_err().to_string().contains("newer version"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_data_file_copied_from_another_mac_is_accepted() {
    let (mut source, source_dir) = store("copied");
    setup(&mut source);
    let raw = std::fs::read_to_string(source_dir.join("data.json")).unwrap();

    let (mut target, dir) = store("copied-to");
    assert_eq!(target.restore(unwrap(&raw).unwrap()).unwrap(), (2, 1));
    let _ = std::fs::remove_dir_all(source_dir);
    let _ = std::fs::remove_dir_all(dir);
}
