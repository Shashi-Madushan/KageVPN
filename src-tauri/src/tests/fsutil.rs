use super::*;

fn temp_dir(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("sbg-fsutil-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn replaces_the_file_and_leaves_no_temp_behind() {
    let dir = temp_dir("replace");
    let path = dir.join("data.json");
    write_private_atomic(&path, b"first").unwrap();
    write_private_atomic(&path, b"second").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"second");
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
    let _ = fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn file_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let dir = temp_dir("mode");
    let path = dir.join("data.json");
    write_private_atomic(&path, b"x").unwrap();
    assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn failed_write_keeps_the_original() {
    let dir = temp_dir("fail");
    let path = dir.join("data.json");
    write_private_atomic(&path, b"keep me").unwrap();
    // A directory in the temp file's place makes the write fail.
    fs::create_dir(dir.join("data.json.tmp")).unwrap();
    assert!(write_private_atomic(&path, b"new").is_err());
    assert_eq!(fs::read(&path).unwrap(), b"keep me");
    let _ = fs::remove_dir_all(dir);
}
