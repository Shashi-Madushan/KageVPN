use tauri::AppHandle;

#[cfg(target_os = "macos")]
const ICON_ICNS: &[u8] = include_bytes!("../icons/icon.icns");

pub fn setup(_app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        ensure_macos_bundle(_app);
    }
}

pub fn send(app: &AppHandle, title: &str, body: &str) {
    #[cfg(target_os = "macos")]
    {
        send_macos(app, title, body);
    }

    #[cfg(not(target_os = "macos"))]
    {
        use tauri_plugin_notification::NotificationExt;
        let _ = app.notification().builder().title(title).body(body).show();
    }
}

#[cfg(target_os = "macos")]
fn ensure_macos_bundle(app: &AppHandle) {
    use tauri::Manager;
    let Ok(data_dir) = app.path().app_data_dir() else { return };
    let bundle_dir = data_dir.join("SingBox GUI.app");
    let contents_dir = bundle_dir.join("Contents");
    let resources_dir = contents_dir.join("Resources");
    let icon_file = resources_dir.join("icon.icns");
    let plist_file = contents_dir.join("Info.plist");

    let needs_register = !icon_file.exists() || !plist_file.exists();
    if needs_register {
        let _ = std::fs::create_dir_all(&resources_dir);
        let _ = std::fs::write(&icon_file, ICON_ICNS);
        let plist_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.shashi.singboxgui</string>
    <key>CFBundleName</key>
    <string>SingBox GUI</string>
    <key>CFBundleDisplayName</key>
    <string>SingBox GUI</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
</dict>
</plist>
"#;
        let _ = std::fs::write(&plist_file, plist_content);
        let lsregister = "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister";
        let _ = std::process::Command::new(lsregister)
            .arg("-f")
            .arg(&bundle_dir)
            .output();
    }

    let _ = mac_notification_sys::set_application("com.shashi.singboxgui");
}

#[cfg(target_os = "macos")]
fn send_macos(app: &AppHandle, title: &str, body: &str) {
    ensure_macos_bundle(app);

    let mut notif = mac_notification_sys::Notification::new();
    notif.sound(mac_notification_sys::Sound::Default);

    let res = mac_notification_sys::send_notification(title, None, body, Some(&notif));
    if res.is_err() {
        send_osascript_fallback(title, body);
    }
}

#[cfg(target_os = "macos")]
fn send_osascript_fallback(title: &str, body: &str) {
    let title_clean = applescript_escape(title);
    let body_clean = applescript_escape(body);
    let script = format!(
        "display notification \"{}\" with title \"{}\" sound name \"default\"",
        body_clean, title_clean
    );

    tauri::async_runtime::spawn_blocking(move || {
        let _ = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output();
    });
}

#[cfg(target_os = "macos")]
fn applescript_escape(value: &str) -> String {
    value
        .replace('\\', r"\\")
        .replace('"', r#"\""#)
        .replace('\n', " ")
        .replace('\r', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_applescript_escape() {
        let text = "Error: \"file not found\"\nPath: C:\\test";
        assert_eq!(
            applescript_escape(text),
            "Error: \\\"file not found\\\" Path: C:\\\\test"
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_mac_notification_sys() {
        let _ = mac_notification_sys::set_application("com.shashi.singboxgui");
        let mut notif = mac_notification_sys::Notification::new();
        notif.sound(mac_notification_sys::Sound::Default);
        let res = mac_notification_sys::send_notification("SingBox GUI", None, "SingBox icon notification", Some(&notif));
        assert!(res.is_ok());
    }
}
