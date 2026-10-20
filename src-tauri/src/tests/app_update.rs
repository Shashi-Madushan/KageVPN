use super::*;

#[test]
fn updates_are_off_until_both_the_feed_and_the_key_are_set() {
    // The shipped config leaves both unset, so the app never contacts an update server.
    assert_eq!(configured().is_some(), APP_UPDATE_FEED.is_some() && APP_UPDATE_PUBKEY.is_some());
}

#[test]
fn settings_default_to_asking_and_old_files_load() {
    let settings: AppUpdateSettings = serde_json::from_str("{}").unwrap();
    assert_eq!(settings.policy, UpdatePolicy::Ask);
    assert_eq!(settings.last_check, 0);
    let saved: AppUpdateSettings = serde_json::from_str(r#"{"policy":"auto","lastCheck":42}"#).unwrap();
    assert_eq!(saved.policy, UpdatePolicy::Auto);
}

#[test]
fn the_policy_names_match_what_the_ui_sends() {
    for (name, policy) in [("ask", UpdatePolicy::Ask), ("auto", UpdatePolicy::Auto), ("off", UpdatePolicy::Off)] {
        assert_eq!(serde_json::from_str::<UpdatePolicy>(&format!("\"{name}\"")).unwrap(), policy);
    }
}

// ── Against a local update feed ─────────────────────────────────────────────
//
// `fixtures/test-updater.pub` is a throwaway public key made only for these tests, and
// `update-payload.bin.sig` is the payload signed with its private half (not kept).

use std::io::{Read, Write};
use std::net::TcpListener;

const TEST_KEY: &str = include_str!("fixtures/test-updater.pub");
const PAYLOAD: &[u8] = include_bytes!("fixtures/update-payload.bin");
const SIGNATURE: &str = include_str!("fixtures/update-payload.bin.sig");

/// Serves `/latest.json` and `/payload` from a local port for exactly `connections` requests.
fn feed(version: &str, payload: &'static [u8], connections: usize) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let platform = |name: &str| format!(r#""{name}":{{"signature":"{}","url":"{base}/payload"}}"#, SIGNATURE.trim());
    let manifest = format!(
        r#"{{"version":"{version}","notes":"Faster tray icon","pub_date":"2030-01-01T00:00:00Z","platforms":{{{},{}}}}}"#,
        platform("darwin-aarch64"),
        platform("darwin-x86_64"),
    );
    let handle = std::thread::spawn(move || {
        for _ in 0..connections {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]).to_string();
            let (kind, body): (&str, Vec<u8>) = if request.starts_with("GET /payload") {
                ("application/octet-stream", payload.to_vec())
            } else {
                ("application/json", manifest.clone().into_bytes())
            };
            let head = format!("HTTP/1.1 200 OK\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(&body);
        }
    });
    (format!("{base}/latest.json"), handle)
}

fn mock_app() -> tauri::App<tauri::test::MockRuntime> {
    // The plugin insists on a `plugins.updater` entry in tauri.conf.json; the real key is set in
    // code from app_config.rs, so the file only carries an empty placeholder. Mirror that here.
    let mut context = tauri::test::mock_context(tauri::test::noop_assets());
    context.config_mut().plugins.0.insert("updater".into(), serde_json::json!({ "pubkey": "" }));
    tauri::test::mock_builder().plugin(tauri_plugin_updater::Builder::new().build()).build(context).unwrap()
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap()
}

#[test]
fn a_newer_version_in_the_feed_is_found_with_its_notes() {
    let app = mock_app();
    let (url, server) = feed("999.0.0", PAYLOAD, 1);
    let updater = build_updater(app.handle(), &url, TEST_KEY.trim()).unwrap();
    let found = runtime().block_on(updater.check()).unwrap().expect("an update");
    server.join().unwrap();
    assert_eq!(found.version, "999.0.0");
    assert_eq!(found.body.as_deref(), Some("Faster tray icon"));
}

#[test]
fn a_feed_that_is_not_newer_means_no_update() {
    let app = mock_app();
    let (url, server) = feed("0.0.1", PAYLOAD, 1);
    let updater = build_updater(app.handle(), &url, TEST_KEY.trim()).unwrap();
    assert!(runtime().block_on(updater.check()).unwrap().is_none());
    server.join().unwrap();
}

#[test]
fn a_correctly_signed_download_is_accepted() {
    let app = mock_app();
    let (url, server) = feed("999.0.0", PAYLOAD, 2);
    let updater = build_updater(app.handle(), &url, TEST_KEY.trim()).unwrap();
    let rt = runtime();
    let update = rt.block_on(updater.check()).unwrap().unwrap();
    let mut chunks = 0usize;
    let bytes = rt.block_on(update.download(|n, _| chunks += n, || {})).unwrap();
    server.join().unwrap();
    assert_eq!(bytes, PAYLOAD);
    assert_eq!(chunks, PAYLOAD.len(), "progress adds up to the whole file");
}

#[test]
fn a_download_that_was_tampered_with_is_rejected() {
    let app = mock_app();
    static TAMPERED: &[u8] = b"pretend this is a malicious bundle";
    let (url, server) = feed("999.0.0", TAMPERED, 2);
    let updater = build_updater(app.handle(), &url, TEST_KEY.trim()).unwrap();
    let rt = runtime();
    let update = rt.block_on(updater.check()).unwrap().unwrap();
    let result = rt.block_on(update.download(|_, _| {}, || {}));
    server.join().unwrap();
    assert!(result.is_err(), "an unsigned or altered download must never be installed");
}

#[test]
fn an_unreachable_feed_is_an_error_not_a_crash() {
    let app = mock_app();
    let updater = build_updater(app.handle(), "http://127.0.0.1:1/latest.json", TEST_KEY.trim()).unwrap();
    assert!(runtime().block_on(updater.check()).is_err());
    assert!(build_updater(app.handle(), "not a url", TEST_KEY.trim()).is_err());
}

#[test]
fn the_updater_can_be_pointed_at_the_tunnel_proxy() {
    let app = mock_app();
    // A dead proxy must make the check fail (so it really is used), and a bad proxy string is ignored.
    let via = build_updater_via(app.handle(), "http://127.0.0.1:1/latest.json", TEST_KEY.trim(), Some("http://127.0.0.1:1")).unwrap();
    assert!(runtime().block_on(via.check()).is_err());
    assert!(build_updater_via(app.handle(), "http://127.0.0.1:1/latest.json", TEST_KEY.trim(), Some("not a url")).is_ok());
}
