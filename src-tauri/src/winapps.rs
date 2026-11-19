//! Reading the Windows uninstall entries, kept free of Windows calls so it is tested everywhere;
//! `apps_windows.rs` feeds it from the registry.

use std::path::{Path, PathBuf};

/// The program a `DisplayIcon` value points at: `"C:\App\app.exe",0` or `C:\App\app.exe`. `None`
/// for an icon file or anything that is not an `.exe`.
pub fn exe_from_display_icon(value: &str) -> Option<PathBuf> {
    let value = value.trim();
    let path = match value.strip_prefix('"') {
        Some(rest) => rest.split('"').next()?,
        None => value.rsplit_once(',').filter(|(_, index)| index.trim().parse::<i32>().is_ok()).map_or(value, |(path, _)| path),
    };
    let path = path.trim();
    path.to_ascii_lowercase().ends_with(".exe").then(|| PathBuf::from(path))
}

/// Uninstallers and updaters are not the app the user means.
pub fn is_helper(exe: &Path) -> bool {
    // Split by hand: Windows paths are tested on macOS too, where `\` is not a separator.
    let text = exe.to_string_lossy().to_ascii_lowercase();
    let name = text.rsplit(['\\', '/']).next().unwrap_or_default();
    ["unins", "uninst", "uninstall", "setup", "update", "updater", "installer"].iter().any(|word| name.starts_with(word))
}

/// The folder an app is matched by: the folder its program is in.
pub fn app_folder(exe: &Path) -> Option<String> {
    let text = exe.to_string_lossy();
    let (folder, _) = text.rsplit_once(['\\', '/'])?;
    (!folder.is_empty()).then(|| folder.to_string())
}

/// The app a running process belongs to: the folder of its program, when the path is a Windows one.
pub fn folder_of_process(process_path: &str) -> Option<&str> {
    let (folder, file) = process_path.rsplit_once('\\')?;
    (file.to_ascii_lowercase().ends_with(".exe") && folder.len() > 2).then_some(folder)
}

#[cfg(test)]
#[path = "tests/winapps.rs"]
mod tests;
