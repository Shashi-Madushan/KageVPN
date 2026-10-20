//! The apps installed on this Mac, for picking one in a routing rule instead of typing the name of
//! its executable.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{AppError, Result};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApp {
    pub name: String,
    /// The bundle, such as `/Applications/Slack.app`. Routing rules store this.
    pub path: String,
    pub bundle_id: Option<String>,
}

/// Where apps live. Folders inside them that are not apps themselves (`/Applications/Setapp`,
/// `/Applications/Utilities`) are looked into one level deep.
fn roots() -> Vec<PathBuf> {
    let mut roots = vec![PathBuf::from("/Applications"), PathBuf::from("/System/Applications")];
    if let Some(home) = std::env::var_os("HOME") {
        roots.push(Path::new(&home).join("Applications"));
    }
    roots
}

pub async fn list() -> Result<Vec<InstalledApp>> {
    tauri::async_runtime::spawn_blocking(|| scan(&roots()))
        .await
        .map_err(|e| AppError::Launch(format!("could not list the installed apps: {e}")))
}

pub fn scan(roots: &[PathBuf]) -> Vec<InstalledApp> {
    let mut seen = HashSet::new();
    let mut apps = Vec::new();
    for root in roots {
        for entry in children(root) {
            if is_bundle(&entry) {
                push(&mut apps, &mut seen, &entry);
            } else if entry.is_dir() {
                children(&entry).iter().filter(|p| is_bundle(p)).for_each(|p| push(&mut apps, &mut seen, p));
            }
        }
    }
    apps.sort_by_cached_key(|app| app.name.to_lowercase());
    apps
}

fn children(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir).map(|entries| entries.flatten().map(|e| e.path()).collect()).unwrap_or_default()
}

fn is_bundle(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("app")) && path.is_dir()
}

fn push(apps: &mut Vec<InstalledApp>, seen: &mut HashSet<String>, bundle: &Path) {
    let path = bundle.to_string_lossy().into_owned();
    if seen.insert(path.clone()) {
        apps.push(read(bundle, path));
    }
}

fn read(bundle: &Path, path: String) -> InstalledApp {
    let info = plist::Value::from_file(bundle.join("Contents/Info.plist")).ok();
    let text = |key: &str| {
        info.as_ref()
            .and_then(|v| v.as_dictionary())
            .and_then(|d| d.get(key))
            .and_then(|v| v.as_string())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
    };
    let name = text("CFBundleDisplayName").or_else(|| text("CFBundleName")).unwrap_or_else(|| stem(bundle));
    InstalledApp { name, path, bundle_id: text("CFBundleIdentifier") }
}

fn stem(bundle: &Path) -> String {
    bundle.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

/// The icon file name an app's `Info.plist` names, with the `.icns` ending macOS leaves out.
pub fn icon_file_name(info: &plist::Dictionary) -> Option<String> {
    let name = info.get("CFBundleIconFile").and_then(|v| v.as_string()).map(str::trim).filter(|n| !n.is_empty())?;
    Some(if name.to_lowercase().ends_with(".icns") { name.to_string() } else { format!("{name}.icns") })
}

/// A file name that is safe to create for any bundle path.
fn cache_name(bundle: &Path) -> String {
    let flat: String = bundle.to_string_lossy().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    format!("{flat}.png")
}

/// The app's icon as a small `data:` PNG, made once with `sips` and kept in `cache`. `None` when the
/// app has no icon or the conversion fails; the caller shows the app's initial instead.
pub fn icon_data_url(bundle: &Path, cache: &Path) -> Option<String> {
    if !is_bundle(bundle) {
        return None;
    }
    let cached = cache.join(cache_name(bundle));
    if !cached.is_file() {
        let info = plist::Value::from_file(bundle.join("Contents/Info.plist")).ok()?;
        let icns = bundle.join("Contents/Resources").join(icon_file_name(info.as_dictionary()?)?);
        std::fs::create_dir_all(cache).ok()?;
        let made = std::process::Command::new("/usr/bin/sips")
            .args(["-s", "format", "png", "-Z", "48"])
            .arg(&icns)
            .arg("--out")
            .arg(&cached)
            .output()
            .ok()?;
        if !made.status.success() || !cached.is_file() {
            let _ = std::fs::remove_file(&cached);
            return None;
        }
    }
    use base64::Engine;
    let bytes = std::fs::read(&cached).ok()?;
    Some(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}

/// The app a running process belongs to: the outermost `.app` in its path, so a helper deep inside
/// (`.../Google Chrome.app/Contents/Frameworks/.../Google Chrome Helper.app/...`) counts as the app.
pub fn bundle_of(process_path: &str) -> Option<&str> {
    let end = process_path.find(".app/").map(|i| i + ".app".len())?;
    Some(&process_path[..end]).filter(|bundle| bundle.starts_with('/'))
}

#[cfg(test)]
#[path = "tests/apps.rs"]
mod tests;
