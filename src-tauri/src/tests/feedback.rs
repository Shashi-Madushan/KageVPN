use std::io::{Read, Write};
use std::net::TcpListener;

use super::*;
use crate::profile::{parse_many, Profile};

const UUID: &str = "11111111-2222-4333-8444-555555555555";

fn profile() -> Profile {
    let link = format!("vless://{UUID}@vpn.example.com:443?security=reality&sni=front.example.org&pbk=PUBKEYPUBKEYPUBKEY&sid=abcd1234&flow=xtls-rprx-vision");
    let mut profile = parse_many(&link).0.remove(0);
    profile.id = "p1".into();
    profile.name = "Test".into();
    profile
}

fn context(logs: Vec<&str>) -> Context {
    Context {
        app: AppRef { name: "App".into(), version: "1.2.3".into() },
        diagnostics: DiagnosticsInfo {
            macos: "15.0".into(),
            arch: "aarch64".into(),
            core: CoreRef { id: "sing-box".into(), version: Some("1.12.9".into()) },
            mode: "tun".into(),
            state: "connected".into(),
            logs: logs.into_iter().map(String::from).collect(),
        },
        profiles: vec![profile()],
        home: Some("/Users/someone".into()),
    }
}

fn input(message: &str, diagnostics: bool) -> FeedbackInput {
    FeedbackInput { kind: FeedbackKind::Bug, message: message.into(), contact: None, include_diagnostics: diagnostics }
}

#[test]
fn nothing_identifying_survives_redaction() {
    let log = format!(
        "[INFO] dial tcp VPN.Example.com:443 as {UUID} via 203.0.113.10 and 2001:db8::1 key PUBKEYPUBKEYPUBKEY sid ABCD1234 sni front.example.org file /Users/someone/Library/x"
    );
    let out = redact(&log, &[profile()], Some("/Users/someone"));
    for leaked in ["example.com", UUID, "203.0.113.10", "2001:db8", "PUBKEY", "abcd1234", "front.example", "someone"] {
        assert!(!out.to_lowercase().contains(&leaked.to_lowercase()), "{leaked} leaked: {out}");
    }
    assert!(out.contains("[server]") && out.contains("[uuid]") && out.contains("[ip]"), "{out}");
}

#[test]
fn leftover_uuids_and_ips_are_masked_even_without_a_matching_profile() {
    let out = redact("peer 198.51.100.7 id 99999999-aaaa-4bbb-8ccc-dddddddddddd", &[], None);
    assert_eq!(out, "peer [ip] id [uuid]");
}

#[test]
fn versions_and_clock_times_are_left_alone() {
    let line = "12:30:05 sing-box 1.12.9 started in 0.5s";
    assert_eq!(redact(line, &[], None), line);
}

#[test]
fn a_trailing_full_stop_after_an_address_is_kept() {
    assert_eq!(redact("connected to 203.0.113.10.", &[], None), "connected to [ip].");
}

#[test]
fn diagnostics_are_null_unless_the_user_opted_in() {
    let off = build_report(context(vec!["[INFO] hi"]), input("It crashes", false), "r1".into(), 1).unwrap();
    assert!(off.diagnostics.is_none());
    let json = serde_json::to_value(&off).unwrap();
    assert!(json["diagnostics"].is_null());
    assert_eq!(json["reportId"], "r1");
    assert_eq!(json["kind"], "bug");

    let on = build_report(context(vec![&format!("[INFO] {UUID}")]), input("It crashes", true), "r2".into(), 1).unwrap();
    let logs = on.diagnostics.unwrap().logs;
    assert_eq!(logs, ["[INFO] [uuid]"]);
}

#[test]
fn the_message_and_contact_are_checked() {
    assert!(build_report(context(vec![]), input("   ", false), "r".into(), 1).is_err());
    assert!(build_report(context(vec![]), input(&"x".repeat(MAX_MESSAGE + 1), false), "r".into(), 1).is_err());
    let mut long_contact = input("ok", false);
    long_contact.contact = Some("c".repeat(MAX_CONTACT + 1));
    assert!(build_report(context(vec![]), long_contact, "r".into(), 1).is_err());

    let mut blank = input("  hello  ", false);
    blank.contact = Some("   ".into());
    let report = build_report(context(vec![]), blank, "r".into(), 1).unwrap();
    assert_eq!(report.message, "hello");
    assert!(report.contact.is_none());
}

/// A throwaway HTTP server that answers once and hands back the raw request it received.
fn serve_once(status_line: &'static str, body: &'static str) -> (String, std::thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/feedback", listener.local_addr().unwrap());
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut raw = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            let n = stream.read(&mut buf).unwrap_or(0);
            raw.extend_from_slice(&buf[..n]);
            let text = String::from_utf8_lossy(&raw).to_string();
            let done = text.split_once("\r\n\r\n").is_some_and(|(head, rest)| {
                let length = head
                    .lines()
                    .find_map(|l| l.to_ascii_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse::<usize>().unwrap_or(0)))
                    .unwrap_or(0);
                rest.len() >= length
            });
            if n == 0 || done {
                break;
            }
        }
        let reply = format!("HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        stream.write_all(reply.as_bytes()).unwrap();
        String::from_utf8_lossy(&raw).to_string()
    });
    (url, handle)
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap()
}

#[test]
fn a_report_is_posted_as_json_with_the_bearer_token_and_the_reference_id_is_returned() {
    let (url, server) = serve_once("201 Created", r#"{"id":"FB-1042"}"#);
    let report = build_report(context(vec![]), input("Please add dark mode", false), "r9".into(), 5).unwrap();

    let result = runtime().block_on(send_report(&url, Some("ingest-token"), &report)).unwrap();
    assert_eq!(result.id.as_deref(), Some("FB-1042"));

    let request = server.join().unwrap();
    assert!(request.starts_with("POST /feedback"), "{request}");
    assert!(request.to_lowercase().contains("authorization: bearer ingest-token"), "{request}");
    assert!(request.to_lowercase().contains("content-type: application/json"), "{request}");
    assert!(request.contains(r#""message":"Please add dark mode""#), "{request}");
    assert!(request.contains(r#""reportId":"r9""#), "{request}");
}

#[test]
fn a_success_without_a_body_is_still_accepted() {
    let (url, server) = serve_once("204 No Content", "");
    let report = build_report(context(vec![]), input("hi", false), "r".into(), 1).unwrap();
    let result = runtime().block_on(send_report(&url, None, &report)).unwrap();
    assert!(result.id.is_none());
    assert!(!server.join().unwrap().to_lowercase().contains("authorization"));
}

#[test]
fn a_server_error_keeps_the_draft_by_failing_with_a_readable_message() {
    let (url, server) = serve_once("500 Internal Server Error", "oops");
    let report = build_report(context(vec![]), input("hi", false), "r".into(), 1).unwrap();
    let err = runtime().block_on(send_report(&url, None, &report)).unwrap_err().to_string();
    server.join().unwrap();
    assert!(err.contains("rejected") && err.contains("500"), "{err}");
}

#[test]
fn an_unreachable_server_fails_without_leaking_internals() {
    let report = build_report(context(vec![]), input("hi", false), "r".into(), 1).unwrap();
    let err = runtime().block_on(send_report("http://127.0.0.1:1/x", None, &report)).unwrap_err().to_string();
    assert!(err.contains("could not reach"), "{err}");
}
