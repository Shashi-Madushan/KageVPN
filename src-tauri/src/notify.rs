use tauri::AppHandle;

pub fn send(_app: &AppHandle, title: &str, body: &str) {
    #[cfg(target_os = "macos")]
    {
        send_macos(title, body);
    }

    #[cfg(not(target_os = "macos"))]
    {
        use tauri_plugin_notification::NotificationExt;
        let _ = _app.notification().builder().title(title).body(body).show();
    }
}

#[cfg(target_os = "macos")]
fn send_macos(title: &str, body: &str) {
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
}
