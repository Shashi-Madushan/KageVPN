use tauri::AppHandle;

pub fn setup(_app: &AppHandle) {}

/// A bundled macOS app has its own notification identity; a dev binary does not, so the
/// system silently drops its notifications and we go through osascript instead.
#[cfg(target_os = "macos")]
fn is_bundled() -> bool {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().contains(".app/Contents/"))
        .unwrap_or(false)
}

/// The same failure is reported from several places (each webview, the backend), so drop
/// an identical notification that follows another within a few seconds.
fn is_duplicate(title: &str, body: &str) -> bool {
    use std::sync::Mutex;
    use std::time::{Duration, Instant};
    static LAST: Mutex<Option<(String, String, Instant)>> = Mutex::new(None);

    let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
    let now = Instant::now();
    if let Some((t, b, at)) = last.as_ref() {
        if t == title && b == body && now.duration_since(*at) < Duration::from_secs(5) {
            return true;
        }
    }
    *last = Some((title.to_owned(), body.to_owned(), now));
    false
}

pub fn send(app: &AppHandle, title: &str, body: &str) {
    if is_duplicate(title, body) {
        return;
    }
    #[cfg(target_os = "macos")]
    if !is_bundled() {
        send_osascript_fallback(title, body);
        return;
    }

    use tauri_plugin_notification::NotificationExt;
    let _ = app.notification().builder().title(title).body(body).show();
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
}
