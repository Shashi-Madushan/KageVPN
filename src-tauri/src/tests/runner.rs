use super::*;

#[test]
fn quotes_paths_with_spaces_and_apostrophes() {
    assert_eq!(shell_quote("/a b/it's"), r"'/a b/it'\''s'");
}

#[test]
fn escapes_applescript_specials() {
    assert_eq!(applescript_escape(r#"a"b\c"#), r#"a\"b\\c"#);
}
