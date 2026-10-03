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
