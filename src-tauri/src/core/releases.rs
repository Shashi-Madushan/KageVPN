use serde::{Deserialize, Serialize};

use super::Channel;
use crate::error::{AppError, Result};

const RELEASES_URL: &str = "https://api.github.com/repos/SagerNet/sing-box/releases?per_page=30";
const MAX_RESULTS: usize = 12;

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    prerelease: bool,
    published_at: Option<String>,
    body: Option<String>,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    size: u64,
    browser_download_url: String,
    digest: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub version: String,
    pub prerelease: bool,
    pub published_at: Option<String>,
    pub notes: String,
    pub size: u64,
    pub url: String,
    pub sha256: Option<String>,
}

pub fn asset_name(version: &str) -> String {
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        _ => "amd64",
    };
    format!("sing-box-{version}-darwin-{arch}.tar.gz")
}

pub async fn fetch(channel: Channel) -> Result<Vec<Release>> {
    let response = reqwest::Client::builder()
        .user_agent(concat!("singbox-gui/", env!("CARGO_PKG_VERSION")))
        .build()?
        .get(RELEASES_URL)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::FORBIDDEN
        || response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
    {
        return Err(AppError::Launch("GitHub rate limit reached, try again later".into()));
    }
    let releases: Vec<GithubRelease> = response.error_for_status()?.json().await?;
    Ok(select(releases, channel))
}

fn select(releases: Vec<GithubRelease>, channel: Channel) -> Vec<Release> {
    releases
        .into_iter()
        .filter(|r| channel == Channel::Prerelease || !r.prerelease)
        .filter_map(|release| {
            let version = release.tag_name.trim_start_matches('v').to_string();
            let wanted = asset_name(&version);
            let asset = release.assets.into_iter().find(|a| a.name == wanted)?;
            Some(Release {
                version,
                prerelease: release.prerelease,
                published_at: release.published_at,
                notes: release.body.unwrap_or_default(),
                size: asset.size,
                url: asset.browser_download_url,
                sha256: asset.digest.and_then(|d| d.strip_prefix("sha256:").map(str::to_string)),
            })
        })
        .take(MAX_RESULTS)
        .collect()
}

#[cfg(test)]
mod tests {
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
}
