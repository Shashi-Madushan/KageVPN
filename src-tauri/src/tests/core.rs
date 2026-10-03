use super::*;

#[test]
fn parses_version_and_tags() {
    let text = "sing-box version 1.14.2\n\nEnvironment: go1.26 darwin/arm64\nTags: with_clash_api,with_utls\n";
    let (version, tags) = parse_version_output(text).unwrap();
    assert_eq!(version, "1.14.2");
    assert_eq!(tags, ["with_clash_api", "with_utls"]);
}

#[test]
fn rejects_garbage_output() {
    assert!(parse_version_output("hello world").is_none());
}
