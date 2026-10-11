use super::*;

#[test]
fn progress_events_use_the_names_the_ui_listens_for() {
    let json = serde_json::to_value(Progress {
        id: "core:sing-box".into(),
        kind: Kind::Core,
        title: "sing-box".into(),
        version: "1.12.9".into(),
        stage: Stage::Verify,
        received: 5,
        total: 10,
    })
    .unwrap();
    assert_eq!(json["id"], "core:sing-box");
    assert_eq!(json["kind"], "core");
    assert_eq!(json["stage"], "verify");
    assert_eq!(json["received"], 5);
}

#[test]
fn finished_events_carry_either_a_version_or_an_error() {
    let ok = serde_json::to_value(Finished {
        id: "app".into(),
        kind: Kind::App,
        title: "App".into(),
        version: Some("2.0.0".into()),
        error: None,
    })
    .unwrap();
    assert_eq!(ok["kind"], "app");
    assert!(ok["error"].is_null());
}

#[test]
fn cancel_is_a_flag_the_next_download_clears() {
    let downloads = Downloads::default();
    assert!(!downloads.is_cancelled() && !downloads.is_busy());
    downloads.cancel();
    assert!(downloads.is_cancelled());
}

// ── Reading a body that stops, or that the user cancels ─────────────────────

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;

/// Answers one request with `declared` bytes promised, sends `parts` (pausing after each), then
/// holds the connection open for `hold` before closing it.
fn serve(declared: usize, parts: &'static [&'static str], pause: Duration, hold: Duration) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/file", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else { return };
        // The request has to be read first, or the client sees an answer to a question it is still asking.
        let _ = stream.read(&mut [0u8; 4096]);
        let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {declared}\r\nConnection: close\r\n\r\n").as_bytes());
        for part in parts {
            let _ = stream.write_all(part.as_bytes());
            let _ = stream.flush();
            std::thread::sleep(pause);
        }
        std::thread::sleep(hold);
    });
    url
}

fn run<T>(future: impl Future<Output = T>) -> T {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(async { tokio::time::timeout(Duration::from_secs(10), future).await.expect("the download hung") })
}

async fn fetch(url: &str) -> reqwest::Response {
    reqwest::Client::new().get(url).send().await.unwrap()
}

#[test]
fn a_body_is_read_to_the_end() {
    let url = serve(11, &["hello ", "world"], Duration::from_millis(30), Duration::ZERO);
    let mut got = Vec::new();
    run(async { read_body(fetch(&url).await, Duration::from_secs(5), || false, |chunk| {
        got.extend_from_slice(chunk);
        Ok(())
    }).await }).unwrap();
    assert_eq!(got, b"hello world");
}

#[test]
fn a_connection_that_goes_silent_ends_in_an_error_instead_of_hanging() {
    // 100 bytes are promised, 5 arrive, then nothing: what a dropped Wi-Fi link looks like.
    let url = serve(100, &["hello"], Duration::ZERO, Duration::from_secs(20));
    let started = std::time::Instant::now();
    let error = run(async { read_body(fetch(&url).await, Duration::from_millis(600), || false, |_| Ok(())).await }).unwrap_err();
    assert!(error.to_string().contains("stalled"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(5), "gave up promptly");
}

#[test]
fn cancelling_stops_even_while_the_connection_is_silent() {
    let url = serve(100, &["hello"], Duration::ZERO, Duration::from_secs(20));
    let cancel = AtomicBool::new(false);
    let error = run(async {
        read_body(
            fetch(&url).await,
            Duration::from_secs(30),
            || cancel.load(Ordering::SeqCst),
            |_| {
                cancel.store(true, Ordering::SeqCst);
                Ok(())
            },
        )
        .await
    })
    .unwrap_err();
    assert!(error.to_string().contains("cancelled"), "{error}");
}
