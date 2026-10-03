use super::*;
use crate::profile::{Profile, Security};

fn profile(id: &str) -> Profile {
    Profile {
        id: id.into(),
        name: id.into(),
        server: "h.example.com".into(),
        port: 443,
        uuid: "11111111-2222-4333-8444-555555555555".into(),
        security: Security::Tls,
        ..Profile::default()
    }
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
    other_transport.transport.path = Some("/ws".into());

    let (added, skipped) = store.import_unique(vec![profile("a"), again.clone(), other_transport]).unwrap();
    assert_eq!((added, skipped), (2, 1), "duplicate inside one paste");
    let (added, skipped) = store.import_unique(vec![again]).unwrap();
    assert_eq!((added, skipped), (0, 1), "duplicate of a saved profile");
    assert_eq!(store.profiles().len(), 2);
    let _ = fs::remove_dir_all(path.parent().unwrap());
}
