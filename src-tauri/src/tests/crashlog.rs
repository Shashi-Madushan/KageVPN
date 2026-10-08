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
