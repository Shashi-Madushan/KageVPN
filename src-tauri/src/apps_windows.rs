//! The apps installed on this PC, for picking one in a routing rule. They come from the uninstall
//! entries Windows keeps for every installed program (per machine, per user, and 32-bit ones). An
//! app is matched by the folder its program is in, so its helpers in the same folder count too.
//! Microsoft Store apps keep no such entry and are not listed; a process-name rule covers them.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::RegKey;

use crate::error::{AppError, Result};
use crate::winapps::{app_folder, exe_from_display_icon, is_helper};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApp {
    pub name: String,
    /// The program's folder, such as `C:\Program Files\Slack`. Routing rules store this.
    pub path: String,
    /// The program's file name, such as `slack.exe`.
    pub bundle_id: Option<String>,
}

const UNINSTALL: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";
const UNINSTALL_32: &str = r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall";

/// Each listed app's folder and program, so its icon can be found from the folder alone.
static PROGRAMS: Mutex<Option<HashMap<String, PathBuf>>> = Mutex::new(None);

pub async fn list() -> Result<Vec<InstalledApp>> {
    tauri::async_runtime::spawn_blocking(scan).await.map_err(|e| AppError::Launch(format!("could not list the installed apps: {e}")))
}

fn scan() -> Vec<InstalledApp> {
    let mut seen = HashSet::new();
    let mut programs = HashMap::new();
    let mut apps = Vec::new();
    let roots = [(HKEY_LOCAL_MACHINE, UNINSTALL), (HKEY_LOCAL_MACHINE, UNINSTALL_32), (HKEY_CURRENT_USER, UNINSTALL)];
    for (hive, path) in roots {
        let Ok(list) = RegKey::predef(hive).open_subkey(path) else { continue };
        for entry in list.enum_keys().flatten().filter_map(|name| list.open_subkey(name).ok()) {
            let Some((name, exe)) = read_entry(&entry) else { continue };
            let Some(folder) = app_folder(&exe) else { continue };
            if !seen.insert(folder.to_lowercase()) {
                continue;
            }
            let file = exe.file_name().map(|f| f.to_string_lossy().into_owned());
            programs.insert(folder.to_lowercase(), exe);
            apps.push(InstalledApp { name, path: folder, bundle_id: file });
        }
    }
    *PROGRAMS.lock().unwrap_or_else(|e| e.into_inner()) = Some(programs);
    apps.sort_by_cached_key(|app| app.name.to_lowercase());
    apps
}

/// An entry's name and program, or `None` for system parts, updates and entries without a program.
fn read_entry(entry: &RegKey) -> Option<(String, PathBuf)> {
    let text = |name: &str| entry.get_value::<String, _>(name).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
    if entry.get_value::<u32, _>("SystemComponent").is_ok_and(|v| v == 1) || text("ParentKeyName").is_some() || text("ReleaseType").is_some() {
        return None;
    }
    let name = text("DisplayName")?;
    let exe = text("DisplayIcon")
        .and_then(|icon| exe_from_display_icon(&icon))
        .filter(|exe| !is_helper(exe) && exe.is_file())
        .or_else(|| text("InstallLocation").and_then(|folder| program_in(Path::new(folder.trim_matches('"')), &name)))?;
    Some((name, exe))
}

/// The program in an install folder: the one named like the app, or the only one there.
fn program_in(folder: &Path, app_name: &str) -> Option<PathBuf> {
    let exes: Vec<PathBuf> = std::fs::read_dir(folder)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("exe")) && !is_helper(p))
        .collect();
    let wanted = app_name.to_lowercase().replace(' ', "");
    exes.iter()
        .find(|p| p.file_stem().is_some_and(|s| wanted.contains(&s.to_string_lossy().to_lowercase().replace(' ', ""))))
        .cloned()
        .or_else(|| (exes.len() == 1).then(|| exes[0].clone()))
}

/// The app's icon as a small `data:` PNG, made once and kept in `cache`. `None` when it cannot be read;
/// the caller shows the app's initial instead.
pub fn icon_data_url(folder: &Path, cache: &Path) -> Option<String> {
    let key = folder.to_string_lossy().to_lowercase();
    let cached = cache.join(format!("{:016x}.png", fnv(&key)));
    if std::fs::metadata(&cached).map_or(true, |m| m.len() == 0) {
        let exe = PROGRAMS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|p| p.get(&key).cloned()).or_else(|| program_in(folder, ""))?;
        let (width, height, rgba) = icon_pixels(&exe)?;
        let png = crate::pngenc::encode_rgba(width, height, &rgba)?;
        std::fs::create_dir_all(cache).ok()?;
        std::fs::write(&cached, png).ok()?;
    }
    use base64::Engine;
    let bytes = std::fs::read(&cached).ok()?;
    Some(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}

/// A stable short name for a cache file.
fn fnv(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3))
}

/// The program's first icon as RGBA rows. Old icons without an alpha channel get theirs from the mask.
fn icon_pixels(exe: &Path) -> Option<(u32, u32, Vec<u8>)> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC,
    };
    use windows_sys::Win32::UI::Shell::ExtractIconExW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};

    /// The bitmap's pixels as top-down BGRA.
    ///
    /// # Safety
    /// `dc` is a live device context and `bitmap` a live bitmap of `width` x `height`.
    unsafe fn pixels(dc: HDC, bitmap: HBITMAP, width: u32, height: u32) -> Option<Vec<u8>> {
        let mut info: BITMAPINFO = unsafe { std::mem::zeroed() };
        info.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            ..unsafe { std::mem::zeroed() }
        };
        let mut buffer = vec![0u8; (width * height * 4) as usize];
        // SAFETY: the buffer holds exactly `height` rows of 32-bit pixels, as the header describes.
        let rows = unsafe { GetDIBits(dc, bitmap, 0, height, buffer.as_mut_ptr().cast(), &mut info, DIB_RGB_COLORS) };
        (rows as u32 == height).then_some(buffer)
    }

    let wide: Vec<u16> = exe.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    // SAFETY: every handle below is checked before use and released exactly once at the end; the
    // structs passed by pointer are sized and zeroed first.
    unsafe {
        let mut icon: HICON = std::ptr::null_mut();
        if ExtractIconExW(wide.as_ptr(), 0, &mut icon, std::ptr::null_mut(), 1) == 0 || icon.is_null() {
            return None;
        }
        let mut info: ICONINFO = std::mem::zeroed();
        if GetIconInfo(icon, &mut info) == 0 {
            DestroyIcon(icon);
            return None;
        }
        let mut bitmap: BITMAP = std::mem::zeroed();
        let size_ok = !info.hbmColor.is_null() && GetObjectW(info.hbmColor, std::mem::size_of::<BITMAP>() as i32, (&mut bitmap as *mut BITMAP).cast()) != 0;
        let (width, height) = (bitmap.bmWidth.unsigned_abs(), bitmap.bmHeight.unsigned_abs());
        let dc = CreateCompatibleDC(std::ptr::null_mut());
        let result = if size_ok && !dc.is_null() && width > 0 && width <= 256 && height > 0 && height <= 256 {
            pixels(dc, info.hbmColor, width, height).map(|mut bgra| {
                let has_alpha = bgra.as_chunks::<4>().0.iter().any(|px| px[3] != 0);
                let mask = if has_alpha || info.hbmMask.is_null() { None } else { pixels(dc, info.hbmMask, width, height) };
                for (i, px) in bgra.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                    px.swap(0, 2);
                    if !has_alpha {
                        // A black mask pixel is part of the icon, a white one is see-through.
                        px[3] = match &mask {
                            Some(m) if m[i * 4] != 0 => 0,
                            _ => 255,
                        };
                    }
                }
                (width, height, bgra)
            })
        } else {
            None
        };
        if !dc.is_null() {
            DeleteDC(dc);
        }
        if !info.hbmColor.is_null() {
            DeleteObject(info.hbmColor);
        }
        if !info.hbmMask.is_null() {
            DeleteObject(info.hbmMask);
        }
        DestroyIcon(icon);
        result
    }
}

/// The app a running process belongs to: the folder its program is in.
pub fn bundle_of(process_path: &str) -> Option<&str> {
    crate::winapps::folder_of_process(process_path)
}
