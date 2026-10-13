use super::*;
use std::io::Write;

#[test]
fn parses_level_from_timestamped_line() {
    let line = LogLine::parse("+0530 2026-09-30 22:30:01 WARN something happened");
    assert!(line.level == Level::Warn);
}

#[test]
fn detects_key_mismatch_hint() {
    let line = LogLine::parse(
        "+0530 2026-09-30 22:31:06 ERROR open connection: unrecognized remote public key: abc123=",
    );
    assert!(matches!(line.hint, Some(Hint::KeyMismatch { key }) if key == "abc123="));
}

#[test]
fn strips_ansi_sequences() {
    assert_eq!(strip_ansi("\u{1b}[31mFATAL\u{1b}[0m[0000] boom"), "FATAL[0000] boom");
}

#[test]
fn detects_unresolved_hint() {
    let line = LogLine::parse("+0530 2026-09-30 22:28:40 ERROR lookup host: NXDOMAIN");
    assert!(matches!(line.hint, Some(Hint::Unresolved)));
}

#[test]
fn a_burst_of_info_lines_is_cut_but_errors_survive() {
    let mut lines: Vec<LogLine> = (0..1000).map(|i| LogLine::parse(&format!("INFO connection {i}"))).collect();
    lines[3] = LogLine::parse("ERROR early failure");

    let (kept, dropped) = cap_lines(lines, 300);

    assert_eq!(dropped, 699);
    assert_eq!(kept.len(), 301);
    assert!(kept.iter().any(|l| l.text == "ERROR early failure"));
    assert!(kept.last().unwrap().text.ends_with("999"), "the newest line is kept");
}

#[test]
fn a_quiet_poll_is_passed_through_untouched() {
    let lines = vec![LogLine::parse("INFO a"), LogLine::parse("INFO b")];
    let (kept, dropped) = cap_lines(lines, 300);
    assert_eq!((kept.len(), dropped), (2, 0));
}

#[test]
fn an_oversized_log_is_set_aside_and_restarted() {
    let dir = std::env::temp_dir().join(format!("kage-logtrim-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let log = dir.join("sing-box.log");

    std::fs::write(&log, "small\n").unwrap();
    assert!(!trim_if_large(&log, 1024), "a small log is left alone");
    assert!(!trim_if_large(&dir.join("missing.log"), 1024), "no file, nothing to do");

    std::fs::write(&log, "x".repeat(2048)).unwrap();
    assert!(trim_if_large(&log, 1024));
    assert_eq!(std::fs::metadata(&log).unwrap().len(), 0, "the live log starts over");
    assert_eq!(std::fs::metadata(dir.join("sing-box.log.prev")).unwrap().len(), 2048, "the old one is kept");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_tailer_starts_again_from_the_top_after_the_log_is_trimmed() {
    let dir = std::env::temp_dir().join(format!("kage-logtail-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let log = dir.join("sing-box.log");
    std::fs::write(&log, format!("{}\n", "INFO old line ".repeat(200))).unwrap();

    let (mut offset, mut pending) = (0u64, String::new());
    assert_eq!(read_new_lines(&log, &mut offset, &mut pending).len(), 1);
    assert!(trim_if_large(&log, 100));
    std::fs::OpenOptions::new().append(true).open(&log).unwrap().write_all(b"INFO fresh\n").unwrap();

    let lines = read_new_lines(&log, &mut offset, &mut pending);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].text.contains("fresh"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn erasing_the_logs_empties_the_log_and_removes_the_previous_one() {
    let dir = std::env::temp_dir().join(format!("kage-logerase-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let log = dir.join("sing-box.log");
    std::fs::write(&log, "INFO inbound connection to example.com:443\n").unwrap();
    std::fs::write(dir.join("sing-box.log.prev"), "older\n").unwrap();

    erase_files(&log);
    assert_eq!(std::fs::metadata(&log).unwrap().len(), 0, "the live log stays, empty");
    assert!(!dir.join("sing-box.log.prev").exists());

    erase_files(&dir.join("missing.log"));
    assert!(!dir.join("missing.log").exists(), "a missing log is not created");
    let _ = std::fs::remove_dir_all(&dir);
}
