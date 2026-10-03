pub mod commands;
pub mod installer;
pub mod provider;
pub mod releases;
pub mod version;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::app_config::CoreSpec;
use crate::error::{AppError, Result};
use crate::logs::strip_ansi;
use crate::paths::AppPaths;
use version::Version;

pub const MANIFEST_NAME: &str = "manifest.json";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    #[default]
    Stable,
    Prerelease,
}

pub use crate::downloads::UpdatePolicy;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CoreSettings {
    pub managed_version: Option<String>,
    pub channel: Channel,
    pub auto_check: bool,
    pub last_check: u64,
    pub update_policy: UpdatePolicy,
    /// Installed in the background while connected; becomes active on the next connect.
    pub pending_version: Option<String>,
}

impl Default for CoreSettings {
    fn default() -> Self {
        Self {
            managed_version: None,
            channel: Channel::Stable,
            auto_check: true,
            last_check: 0,
            update_policy: UpdatePolicy::Ask,
            pending_version: None,
        }
    }
}

/// Which core runs connections, and each core's own settings (versions, channel, policy).
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CoreSet {
    pub active: String,
    pub entries: std::collections::BTreeMap<String, CoreSettings>,
}

impl Default for CoreSet {
    fn default() -> Self {
        Self { active: crate::app_config::default_core().id.to_string(), entries: Default::default() }
    }
}

/// Makes a version installed in the background the active one, if it is still on disk.
pub fn apply_pending(settings: &mut CoreSettings, paths: &AppPaths, spec: &CoreSpec) -> bool {
    let Some(version) = settings.pending_version.take() else { return false };
    if managed_binary(paths, spec, &version).is_file() {
        settings.managed_version = Some(version);
    }
    true
}

/// Background updates only ever replace an engine this app manages itself.
pub fn should_auto_install(policy: UpdatePolicy, channel: Channel, source: Source) -> bool {
    policy == UpdatePolicy::Auto && channel == Channel::Stable && source == Source::Managed
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

pub fn managed_binary(paths: &AppPaths, spec: &CoreSpec, version: &str) -> PathBuf {
    paths.core_dir_for(spec.id).join(version).join(spec.binary_name)
}

pub fn inspect(spec: &CoreSpec, path: &Path, source: Source) -> Result<CoreInfo> {
    let output = Command::new(path).args(spec.version_args).output()?;
    if !output.status.success() {
        return Err(AppError::Launch("the binary did not report a version".into()));
    }
    let text = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    let (version, tags) = parse_version_output(&text)
        .ok_or_else(|| AppError::Launch("unrecognised `sing-box version` output".into()))?;

    let missing_tags = spec
        .required_features
        .iter()
        .filter(|tag| !tags.iter().any(|t| t == *tag))
        .map(|t| t.to_string())
        .collect();
    let minimum = Version::parse(spec.min_version).expect("valid constant");
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

pub fn installed(paths: &AppPaths, spec: &CoreSpec) -> Vec<Installed> {
    let Ok(entries) = fs::read_dir(paths.core_dir_for(spec.id)) else { return Vec::new() };
    let mut found: Vec<(Version, Installed)> = entries
        .flatten()
        .filter_map(|entry| {
            let version = entry.file_name().to_string_lossy().to_string();
            let parsed = Version::parse(&version)?;
            let path = entry.path().join(spec.binary_name);
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

pub fn resolve(paths: &AppPaths, spec: &CoreSpec, settings: &CoreSettings) -> Result<(PathBuf, Source)> {
    if let Some(version) = &settings.managed_version {
        let binary = managed_binary(paths, spec, version);
        if binary.is_file() {
            return Ok((binary, Source::Managed));
        }
    }
    locate(spec).map(|path| (path, Source::System))
}

/// Finds a copy of the core installed outside the app: the usual prefixes, then `PATH`.
pub fn locate(spec: &CoreSpec) -> Result<PathBuf> {
    let on_path: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).map(|dir| dir.join(spec.binary_name)).collect())
        .unwrap_or_default();

    spec.system_paths
        .iter()
        .map(PathBuf::from)
        .chain(on_path)
        .find(|candidate| candidate.is_file())
        .ok_or(AppError::SingBoxMissing)
}

/// Earlier versions kept downloads directly in `core/<version>/`; move them under the core's id.
pub fn migrate_layout(paths: &AppPaths, spec: &CoreSpec) {
    let Ok(entries) = fs::read_dir(paths.core_dir()) else { return };
    let target = paths.core_dir_for(spec.id);
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if Version::parse(&name).is_some() && entry.path().is_dir() {
            let _ = fs::create_dir_all(&target);
            let _ = fs::rename(entry.path(), target.join(&name));
        }
    }
}

#[cfg(test)]
#[path = "../tests/core.rs"]
mod tests;
