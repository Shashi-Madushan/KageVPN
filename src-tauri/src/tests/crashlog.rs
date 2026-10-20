use super::*;

#[test]
fn reads_the_last_exit_status() {
    let text = "panic stuff\n2026-10-08 09:20:16 sing-box exited with status 0 (stop requested: no)\n2026-10-08 10:00:00 sing-box exited with status 2 (stop requested: no)";
    assert_eq!(exit_status(text).as_deref(), Some("2 (stop requested: no)"));
    assert_eq!(exit_status("nothing here"), None);
}

#[test]
fn summary_combines_status_and_last_error() {
    let text = summary(Some("2 (stop requested: no)"), Some("+0530 2026-10-08 ERROR start service: bind failed"));
    assert_eq!(text, "sing-box exited with status 2: start service: bind failed");
    assert_eq!(summary(None, None), "sing-box stopped unexpectedly");
}

#[test]
fn record_appends_and_trims() {
    let dir = std::env::temp_dir().join(format!("kage-crash-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    for i in 0..3 {
        record(&dir, "exit", &format!("reason {i}"), "details");
    }
    let text = fs::read_to_string(dir.join(FILE)).unwrap();
    assert_eq!(text.matches("==== ").count(), 3);
    assert!(text.contains("reason 2"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn read_returns_entries_and_clear_removes_them() {
    let dir = std::env::temp_dir().join(format!("kage-crash-read-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    assert_eq!(read(&dir), "");
    record(&dir, "exit", "boom", "details");
    assert!(read(&dir).contains("boom"));
    clear(&dir);
    assert_eq!(read(&dir), "");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn trimming_a_big_log_with_non_ascii_text_does_not_crash() {
    // The midpoint of this file falls inside a three-byte character. Cutting there used to panic,
    // and the crash log is written exactly when something else has already gone wrong.
    let dir = std::env::temp_dir().join(format!("kage-crash-utf8-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let big = format!("==== old entry\n{}\n\n", "€".repeat(70_000));
    assert!(big.len() as u64 > MAX_BYTES && (big.len() / 2) % 3 != 0);
    fs::write(dir.join(FILE), &big).unwrap();

    record(&dir, "exit", "fresh", "details");

    let text = fs::read_to_string(dir.join(FILE)).unwrap();
    assert!(text.contains("fresh"));
    assert!((text.len() as u64) < MAX_BYTES);
    let _ = fs::remove_dir_all(&dir);
}
