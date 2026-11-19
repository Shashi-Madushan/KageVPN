use super::*;

fn v(text: &str) -> Version {
    Version::parse(text).unwrap()
}

#[test]
fn parses_with_and_without_prefix() {
    assert_eq!(v("v1.14.2"), v("1.14.2"));
    assert!(Version::parse("1.14").is_none());
    assert!(Version::parse("abc").is_none());
}

#[test]
fn orders_numerically_not_lexically() {
    assert!(v("1.14.2") > v("1.9.10"));
    assert!(v("1.15.0") > v("1.14.9"));
}

#[test]
fn prerelease_sorts_before_its_release() {
    assert!(v("1.15.0-alpha.9") < v("1.15.0"));
    assert!(v("1.15.0-alpha.10") > v("1.15.0-alpha.9"));
    assert!(v("1.15.0-alpha.1") > v("1.14.2"));
}

#[test]
fn a_version_cannot_be_a_path() {
    for bad in ["1.0.0-../../x", "1.0.0-a/b", "1.0.0./etc", "1.2.3.4", "1.0.0-", "1.0.0-a b", "1.0.0-.."] {
        assert!(Version::parse(bad).is_none(), "{bad} parsed");
    }
    assert!(Version::parse("1.15.0-alpha.9").is_some());
    assert!(Version::parse("1.15.0-rc-2").is_some());
}
