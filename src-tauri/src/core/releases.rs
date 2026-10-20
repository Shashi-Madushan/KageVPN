use serde::{Deserialize, Serialize};

use super::Channel;
use crate::app_config::CoreSpec;
use crate::error::{AppError, Result};

const MAX_RESULTS: usize = 12;
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

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

pub fn asset_name(spec: &CoreSpec, version: &str) -> String {
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        _ => "amd64",
    };
    (spec.asset_name)(version, arch)
}

pub async fn fetch(spec: &CoreSpec, channel: Channel) -> Result<Vec<Release>> {
    let response = reqwest::Client::builder()
        .user_agent(crate::app_config::user_agent())
        .timeout(TIMEOUT)
        .build()?
        .get(spec.releases_api)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::FORBIDDEN
        || response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
    {
        return Err(AppError::Launch("GitHub rate limit reached, try again later".into()));
    }
    let releases: Vec<GithubRelease> = response.error_for_status()?.json().await?;
    Ok(select(spec, releases, channel))
}

fn select(spec: &CoreSpec, releases: Vec<GithubRelease>, channel: Channel) -> Vec<Release> {
    releases
        .into_iter()
        .filter(|r| channel == Channel::Prerelease || !r.prerelease)
        .filter_map(|release| {
            let version = release.tag_name.trim_start_matches('v').to_string();
            let wanted = asset_name(spec, &version);
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
#[path = "../tests/core_releases.rs"]
mod tests;
