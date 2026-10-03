use super::*;

fn release(tag: &str, prerelease: bool, with_asset: bool) -> GithubRelease {
    let assets = if with_asset {
        vec![GithubAsset {
            name: asset_name(tag.trim_start_matches('v')),
            size: 1,
            browser_download_url: "https://example.test/a".into(),
            digest: Some("sha256:abc".into()),
        }]
    } else {
        vec![]
    };
    GithubRelease { tag_name: tag.into(), prerelease, published_at: None, body: None, assets }
}

#[test]
fn stable_channel_skips_prereleases_and_missing_assets() {
    let list = vec![release("v1.15.0-alpha.1", true, true), release("v1.14.2", false, true), release("v1.14.1", false, false)];
    let picked = select(list, Channel::Stable);
    assert_eq!(picked.len(), 1);
    assert_eq!(picked[0].version, "1.14.2");
    assert_eq!(picked[0].sha256.as_deref(), Some("abc"));
}

#[test]
fn prerelease_channel_includes_both() {
    let list = vec![release("v1.15.0-alpha.1", true, true), release("v1.14.2", false, true)];
    assert_eq!(select(list, Channel::Prerelease).len(), 2);
}
