use super::*;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("osprey-paths-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn an_old_data_folder_is_moved_to_the_new_identifier_with_its_contents() {
    let root = scratch("move");
    let (old, new) = (root.join("com.old.app"), root.join("com.new.app"));
    fs::create_dir_all(old.join("core/sing-box/1.12.0")).unwrap();
    fs::write(old.join("data.json"), br#"{"profiles":[]}"#).unwrap();
    fs::write(old.join("core/sing-box/1.12.0/sing-box"), b"bin").unwrap();

    assert!(adopt_legacy_dir(&new, std::slice::from_ref(&old)).unwrap());

    assert_eq!(fs::read(new.join("data.json")).unwrap(), br#"{"profiles":[]}"#);
    assert!(new.join("core/sing-box/1.12.0/sing-box").is_file());
    assert!(!old.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn nothing_is_overwritten_when_the_new_folder_already_has_data() {
    let root = scratch("keep");
    let (old, new) = (root.join("old"), root.join("new"));
    fs::create_dir_all(&old).unwrap();
    fs::write(old.join("data.json"), b"old").unwrap();
    fs::create_dir_all(&new).unwrap();
    fs::write(new.join("data.json"), b"new").unwrap();

    assert!(!adopt_legacy_dir(&new, std::slice::from_ref(&old)).unwrap());
    assert_eq!(fs::read(new.join("data.json")).unwrap(), b"new");
    assert!(old.join("data.json").is_file(), "the old copy is left alone");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_empty_new_folder_does_not_block_the_move_and_a_missing_old_one_is_fine() {
    let root = scratch("empty");
    let (old, new) = (root.join("old"), root.join("new"));
    assert!(!adopt_legacy_dir(&new, std::slice::from_ref(&old)).unwrap(), "no old folder");

    fs::create_dir_all(&old).unwrap();
    fs::write(old.join("data.json"), b"x").unwrap();
    fs::create_dir_all(&new).unwrap();
    assert!(adopt_legacy_dir(&new, std::slice::from_ref(&old)).unwrap());
    assert!(new.join("data.json").is_file());
    let _ = fs::remove_dir_all(root);
}
