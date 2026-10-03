use super::*;

#[test]
fn test_applescript_escape() {
    let text = "Error: \"file not found\"\nPath: C:\\test";
    assert_eq!(
        applescript_escape(text),
        "Error: \\\"file not found\\\" Path: C:\\\\test"
    );
}
