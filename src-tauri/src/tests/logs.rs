use super::*;

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
