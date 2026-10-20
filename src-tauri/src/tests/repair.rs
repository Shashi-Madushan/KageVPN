use super::*;

#[test]
fn only_leftover_request_files_are_listed_and_cleared() {
    let dir = std::env::temp_dir().join(format!("kage-repair-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let paths = AppPaths::new(dir.clone()).unwrap();
    assert!(stale_files(&paths).is_empty());

    for file in [paths.stop_file(), paths.helper_start_file(), paths.child_pid_file()] {
        std::fs::write(file, b"x").unwrap();
    }
    std::fs::write(paths.allow_file(), b"1.2.3.4").unwrap();
    std::fs::write(paths.store_file(), b"{}").unwrap();
    assert_eq!(stale_files(&paths).len(), 3);
    assert_eq!(clear_stale(&paths), 3);
    assert!(stale_files(&paths).is_empty());
    assert!(paths.allow_file().exists(), "the server allow-list is not a leftover");
    assert!(paths.store_file().exists(), "saved data is never touched");
    let _ = std::fs::remove_dir_all(dir);
}
