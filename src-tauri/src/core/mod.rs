pub mod commands;
pub mod installer;
pub mod releases;
pub mod version;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};
use crate::logs::strip_ansi;
use crate::paths::AppPaths;
use crate::singbox;
use version::Version;

pub const REQUIRED_TAGS: [&str; 2] = ["with_clash_api", "with_utls"];
const MIN_VERSION: &str = "1.12.0";
pub const BINARY_NAME: &str = "sing-box";
pub const MANIFEST_NAME: &str = "manifest.json";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    #[default]
    Stable,
    Prerelease,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CoreSettings {
    pub managed_version: Option<String>,
    pub channel: Channel,
    pub auto_check: bool,
    pub last_check: u64,
}

impl Default for CoreSettings {
    fn default() -> Self {
        Self { managed_version: None, channel: Channel::Stable, auto_check: true, last_check: 0 }
    }
}

#[derive(Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    System,
    Managed,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreInfo {
    pub path: String,
    pub version: String,
    pub tags: Vec<String>,
    pub source: Source,
    pub missing_tags: Vec<String>,
    pub outdated: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub version: String,
    pub archive_sha256: String,
    pub binary_sha256: String,
    pub installed_at: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Installed {
    pub version: String,
    pub path: String,
    pub installed_at: Option<u64>,
}

pub fn managed_binary(paths: &AppPaths, version: &str) -> PathBuf {
    paths.core_dir().join(version).join(BINARY_NAME)
}

pub fn inspect(path: &Path, source: Source) -> Result<CoreInfo> {
    let output = Command::new(path).arg("version").output()?;
    if !output.status.success() {
        return Err(AppError::Launch("the binary did not report a version".into()));
    }
    let text = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    let (version, tags) = parse_version_output(&text)
        .ok_or_else(|| AppError::Launch("unrecognised `sing-box version` output".into()))?;

    let missing_tags = REQUIRED_TAGS
        .iter()
        .filter(|tag| !tags.iter().any(|t| t == *tag))
        .map(|t| t.to_string())
        .collect();
    let minimum = Version::parse(MIN_VERSION).expect("valid constant");
    let outdated = Version::parse(&version).is_some_and(|v| v < minimum);

    Ok(CoreInfo { path: path.display().to_string(), version, tags, source, missing_tags, outdated })
}

fn parse_version_output(text: &str) -> Option<(String, Vec<String>)> {
    let version = text.lines().next()?.split_whitespace().last()?.to_string();
    let tags = text
        .lines()
        .find_map(|line| line.strip_prefix("Tags:"))
        .map(|rest| rest.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect())
        .unwrap_or_default();
    Version::parse(&version).map(|_| (version, tags))
}

pub fn installed(paths: &AppPaths) -> Vec<Installed> {
    let Ok(entries) = fs::read_dir(paths.core_dir()) else { return Vec::new() };
    let mut found: Vec<(Version, Installed)> = entries
        .flatten()
        .filter_map(|entry| {
            let version = entry.file_name().to_string_lossy().to_string();
            let parsed = Version::parse(&version)?;
            let path = entry.path().join(BINARY_NAME);
            path.is_file().then(|| {
                let installed_at = read_manifest(&entry.path()).map(|m| m.installed_at);
                (parsed, Installed { version, path: path.display().to_string(), installed_at })
            })
        })
        .collect();
    found.sort_by(|a, b| b.0.cmp(&a.0));
    found.into_iter().map(|(_, installed)| installed).collect()
}

pub fn read_manifest(dir: &Path) -> Option<Manifest> {
    serde_json::from_slice(&fs::read(dir.join(MANIFEST_NAME)).ok()?).ok()
}

pub fn resolve(paths: &AppPaths, settings: &CoreSettings) -> Result<(PathBuf, Source)> {
    if let Some(version) = &settings.managed_version {
        let binary = managed_binary(paths, version);
        if binary.is_file() {
            return Ok((binary, Source::Managed));
        }
    }
    singbox::locate().map(|path| (path, Source::System))
}

#[cfg(test)]
mod tests {
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
}
