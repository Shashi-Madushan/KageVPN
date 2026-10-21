use std::time::Duration;

use super::*;
use crate::profile::parse_many;

const DAY: u64 = 24 * 3600;

#[test]
fn a_subscription_without_an_expiry_date_never_warns() {
    assert_eq!(due_expiry_notice(0, 1_000, (0, 0)), None);
}

#[test]
fn the_warnings_come_at_seven_three_and_one_days_then_at_the_end_each_once() {
    let expire = 100 * DAY;
    assert_eq!(due_expiry_notice(expire, 50 * DAY, (0, 0)), None, "weeks away");
    assert_eq!(due_expiry_notice(expire, expire - 8 * DAY, (0, 0)), None);
    assert_eq!(due_expiry_notice(expire, expire - 7 * DAY, (0, 0)), Some(Expiry::Week));
    assert_eq!(due_expiry_notice(expire, expire - 5 * DAY, (expire, 1)), None, "the week warning is not repeated");
    assert_eq!(due_expiry_notice(expire, expire - 3 * DAY, (expire, 1)), Some(Expiry::Soon));
    assert_eq!(due_expiry_notice(expire, expire - 2 * DAY, (expire, 2)), None);
    assert_eq!(due_expiry_notice(expire, expire - DAY, (expire, 2)), Some(Expiry::LastDay));
    assert_eq!(due_expiry_notice(expire, expire - 3600, (expire, 3)), None);
    // Past the date: say so once, then stay quiet. This used to repeat on every refresh, forever.
    assert_eq!(due_expiry_notice(expire, expire + 1, (expire, 3)), Some(Expiry::Expired));
    assert_eq!(due_expiry_notice(expire, expire + 12 * 3600, (expire, 4)), None);
    assert_eq!(due_expiry_notice(expire, expire + 30 * DAY, (expire, 4)), None);
}

#[test]
fn a_plan_first_seen_close_to_the_end_announces_only_the_latest_stage() {
    let expire = 100 * DAY;
    assert_eq!(due_expiry_notice(expire, expire - 2 * DAY, (0, 0)), Some(Expiry::Soon));
    assert_eq!(due_expiry_notice(expire, expire - 3600, (0, 0)), Some(Expiry::LastDay));
}

#[test]
fn renewing_starts_the_warnings_over() {
    let (old, renewed) = (100 * DAY, 130 * DAY);
    assert_eq!(due_expiry_notice(renewed, renewed - DAY, (old, 4)), Some(Expiry::LastDay), "a new date is a new subscription period");
    assert_eq!(due_expiry_notice(renewed, renewed - 6 * DAY, (old, 4)), Some(Expiry::Week));
}

#[test]
fn the_stored_dates_are_checked_without_a_refresh() {
    let mut sub = subscription("s");
    sub.user_info = Some(UserInfo { expire: 100 * DAY, ..UserInfo::default() });
    // Refreshes keep failing, but the date is known and close.
    sub.failures = 5;
    let notice = expiry_pass(&mut sub, 100 * DAY - 2 * DAY);
    assert_eq!(notice, Some(Notice::Expiry { name: "Provider".into(), expire: 100 * DAY, stage: Expiry::Soon }));
    assert_eq!((sub.notified_expire, sub.notified_stage), (100 * DAY, 2));
    assert_eq!(expiry_pass(&mut sub, 100 * DAY - 2 * DAY + 1800), None, "the next tick says nothing");
    assert!(expiry_pass(&mut subscription("none"), 5).is_none(), "no figures, no warning");
    let message = Notice::Expiry { name: "P".into(), expire: 100 * DAY, stage: Expiry::LastDay }.message(100 * DAY - 3600);
    assert_eq!(message, "P expires in a day.");
}

#[test]
fn older_saved_stage_numbers_cause_at_most_one_repeat() {
    // Before the week and last-day stages existed, 1 meant "soon" and 2 meant "expired".
    let expire = 100 * DAY;
    assert_eq!(due_expiry_notice(expire, expire - 2 * DAY, (expire, 1)), Some(Expiry::Soon), "one repeat");
    assert_eq!(due_expiry_notice(expire, expire + 1, (expire, 2)), Some(Expiry::Expired), "one repeat");
}

#[test]
fn a_subscription_first_seen_already_expired_is_announced_directly() {
    assert_eq!(due_expiry_notice(10, 20, (0, 0)), Some(Expiry::Expired));
}

#[test]
fn days_are_rounded_up_in_plain_words() {
    assert_eq!(days_left(100 * DAY, 100 * DAY - 3600), "a day");
    assert_eq!(days_left(100 * DAY, 100 * DAY - 2 * DAY + 60), "2 days");
    assert_eq!(days_left(100 * DAY, 100 * DAY - 3 * DAY), "3 days");
}

#[test]
fn the_userinfo_header_is_read_and_junk_is_ignored() {
    let info = parse_user_info("upload=10; download=20; total=1000; expire=1700000000; bogus; x=y");
    assert_eq!((info.upload, info.download, info.total, info.expire), (10, 20, 1000, 1_700_000_000));
}

// ── Placeholders, back-off and refreshing a changed list ────────────────────

const ID: &str = "11111111-2222-4333-8444-555555555555";

fn servers(hosts: &[&str]) -> Vec<Profile> {
    let text: Vec<String> = hosts.iter().map(|h| format!("vless://{ID}@{h}:443?security=tls#Name of {h}")).collect();
    parse_many(&text.join("\n")).0
}

fn fresh_store(label: &str) -> (Store, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("kage-subs-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    (Store::load(dir.join("data.json")), dir)
}

fn subscription(id: &str) -> Subscription {
    Subscription { id: id.into(), name: "Provider".into(), url: "https://example.com/sub".into(), ..Subscription::default() }
}

#[test]
fn entries_that_are_not_servers_are_dropped_with_a_reason() {
    // Providers add rows like "Remaining traffic: 80 GB" that point at the local machine.
    let all = servers(&["a.example.com", "127.0.0.1", "0.0.0.0", "localhost", "[::1]", "b.example.com"]);
    assert_eq!(all.len(), 6);
    let (real, reasons) = drop_placeholders(all);
    assert_eq!(real.iter().map(|p| p.server.as_str()).collect::<Vec<_>>(), ["a.example.com", "b.example.com"]);
    assert_eq!(reasons.len(), 4);
    assert!(reasons[0].starts_with("not a server (127.0.0.1): Name of 127.0.0.1"), "{reasons:?}");
}

#[test]
fn a_private_lan_address_is_still_a_real_server() {
    // Someone's own server on the local network is legitimate; only loopback and 0.0.0.0 are not.
    let (real, reasons) = drop_placeholders(servers(&["192.168.1.10", "10.0.0.5"]));
    assert_eq!((real.len(), reasons.len()), (2, 0));
}

#[test]
fn a_failing_subscription_is_retried_sooner_then_less_and_less_often() {
    let base = Subscription { interval_hours: 12, last_updated: 1_000_000, last_attempt: 1_000_000, ..Subscription::default() };
    assert_eq!(next_due(&base), 1_000_000 + 12 * 3600, "a healthy one waits its interval");

    let wait = |failures: u32| next_due(&Subscription { failures, ..base.clone() }) - 1_000_000;
    assert_eq!(wait(1), 30 * 60, "a hiccup is retried in half an hour");
    assert_eq!(wait(2), 60 * 60);
    assert_eq!(wait(3), 2 * 3600);
    assert_eq!(wait(4), 4 * 3600);
    assert_eq!(wait(5), 8 * 3600);
    assert_eq!(wait(6), 12 * 3600, "never longer than the normal interval");
    assert_eq!(wait(500), 12 * 3600);
}

#[test]
fn a_brand_new_subscription_is_due_at_once() {
    assert!(next_due(&Subscription::default()) <= 12 * 3600, "never refreshed, so the scheduler picks it up on its first pass");
}

#[test]
fn a_change_made_while_the_download_ran_is_not_overwritten() {
    let (mut store, dir) = fresh_store("lost-update");
    store.add_subscription(subscription("s")).unwrap();
    // The user renames it while the download is in flight.
    store.update_subscription(Subscription { name: "Renamed".into(), ..subscription("s") }).unwrap();

    let (result, _) = record_refresh(&mut store, "s", Ok((servers(&["a.example.com"]), None, vec![])), 5_000);
    assert_eq!(result.unwrap().added, 1);
    let saved = &store.subscriptions()[0];
    assert_eq!((saved.name.as_str(), saved.last_updated, saved.failures), ("Renamed", 5_000, 0));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_subscription_removed_during_the_download_does_not_leave_orphan_servers() {
    let (mut store, dir) = fresh_store("removed");
    store.add_subscription(subscription("s")).unwrap();
    store.remove_subscription("s").unwrap();

    let (result, notices) = record_refresh(&mut store, "s", Ok((servers(&["a.example.com"]), None, vec![])), 5_000);
    assert!(result.is_err() && notices.is_empty());
    assert!(store.profiles().is_empty(), "no servers belonging to a subscription that no longer exists");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn failures_are_counted_and_a_success_clears_them() {
    let (mut store, dir) = fresh_store("failures");
    store.add_subscription(subscription("s")).unwrap();
    let down = || Err(AppError::ServerUnreachable("down".into()));

    record_refresh(&mut store, "s", down(), 1_000).0.unwrap_err();
    record_refresh(&mut store, "s", down(), 2_000).0.unwrap_err();
    let failed = store.subscriptions()[0].clone();
    assert_eq!((failed.failures, failed.last_attempt, failed.last_updated), (2, 2_000, 0));
    assert_eq!(failed.last_error.as_deref(), Some("down"));

    record_refresh(&mut store, "s", Ok((servers(&["a.example.com"]), None, vec![])), 3_000).0.unwrap();
    let healed = &store.subscriptions()[0];
    assert_eq!((healed.failures, healed.last_updated, healed.last_error.clone()), (0, 3_000, None));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_expiry_notice_comes_out_of_a_refresh_once() {
    let (mut store, dir) = fresh_store("expiry");
    store.add_subscription(subscription("s")).unwrap();
    let info = || Some(UserInfo { expire: 10_000, ..UserInfo::default() });

    let (_, first) = record_refresh(&mut store, "s", Ok((servers(&["a.example.com"]), info(), vec![])), 20_000);
    assert_eq!(first, [Notice::Expiry { name: "Provider".into(), expire: 10_000, stage: Expiry::Expired }]);
    let (_, again) = record_refresh(&mut store, "s", Ok((servers(&["a.example.com"]), info(), vec![])), 40_000);
    assert!(again.is_empty(), "the same expiry is not announced again");
    let _ = std::fs::remove_dir_all(dir);
}

// ── Data warnings, client names and device-limited providers ────────────────

const GB: u64 = 1024 * 1024 * 1024;

fn usage(used: u64, total: u64) -> UserInfo {
    UserInfo { upload: used / 4, download: used - used / 4, total, expire: 0 }
}

#[test]
fn an_unlimited_plan_never_warns_about_data() {
    // `total=0` means no limit. One client read it as "used up" and stopped connecting.
    assert_eq!(due_data_notice(&usage(500 * GB, 0), 0), (None, 0));
}

#[test]
fn data_warnings_come_once_at_ninety_percent_and_once_when_used_up() {
    assert_eq!(due_data_notice(&usage(80 * GB, 100 * GB), 0), (None, 0));
    assert_eq!(due_data_notice(&usage(90 * GB, 100 * GB), 0), (Some(DataStage::Low), 1));
    assert_eq!(due_data_notice(&usage(95 * GB, 100 * GB), 1), (None, 1), "already warned");
    assert_eq!(due_data_notice(&usage(100 * GB, 100 * GB), 1), (Some(DataStage::UsedUp), 2));
    assert_eq!(due_data_notice(&usage(120 * GB, 100 * GB), 2), (None, 2));
    // A new month (or a top-up) starts over, so the next period warns again.
    assert_eq!(due_data_notice(&usage(GB, 100 * GB), 2), (None, 0));
    assert_eq!(due_data_notice(&usage(91 * GB, 100 * GB), 0), (Some(DataStage::Low), 1));
    // First seen already used up: one notice, not two.
    assert_eq!(due_data_notice(&usage(100 * GB, 100 * GB), 0), (Some(DataStage::UsedUp), 2));
    // Huge figures do not overflow.
    assert_eq!(due_data_notice(&UserInfo { upload: u64::MAX, download: u64::MAX, total: u64::MAX, expire: 0 }, 0).1, 2);
}

#[test]
fn the_data_notice_comes_out_of_a_refresh_once_with_what_is_left() {
    let (mut store, dir) = fresh_store("data");
    store.add_subscription(subscription("s")).unwrap();
    let refresh = |store: &mut Store, used: u64| record_refresh(store, "s", Ok((servers(&["a.example.com"]), Some(usage(used, 100 * GB)), vec![])), 5_000).1;

    let first = refresh(&mut store, 92 * GB);
    assert_eq!(first, [Notice::Data { name: "Provider".into(), stage: DataStage::Low, left: 8 * GB }]);
    assert_eq!(first[0].message(0), "Provider: 90% of your data is used, 8.0 GB left.");
    assert!(refresh(&mut store, 93 * GB).is_empty());
    assert_eq!(refresh(&mut store, 100 * GB)[0].message(0), "Provider: your data allowance is used up.");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_typed_client_name_is_tidied_or_refused() {
    assert_eq!(clean_user_agent(None).unwrap(), None);
    assert_eq!(clean_user_agent(Some("   ")).unwrap(), None);
    assert_eq!(clean_user_agent(Some(" v2rayN/7.10.0 ")).unwrap().as_deref(), Some("v2rayN/7.10.0"));
    assert_eq!(clean_user_agent(Some("SFM/1.14.2 (sing-box)")).unwrap().as_deref(), Some("SFM/1.14.2 (sing-box)"));
    // A header value cannot hold line breaks or other control characters.
    assert!(clean_user_agent(Some("evil\r\nX-Other: 1")).is_err());
    assert!(clean_user_agent(Some("naïve")).is_err());
    assert!(clean_user_agent(Some(&"a".repeat(201))).is_err());
}

#[test]
fn refusals_say_what_to_do_and_never_quote_the_link() {
    let none = |_: &str| false;
    let with = |names: &'static [&'static str]| move |name: &str| names.contains(&name);

    assert_eq!(refusal(200, none, false), None);
    // A device-limited panel answers 404 without the device ID; say why instead of "not found".
    let needs_id = refusal(404, with(&["x-hwid-active", "x-hwid-not-supported"]), false).unwrap();
    assert!(needs_id.contains("Send device ID"), "{needs_id}");
    // The limit itself is reached: turning the switch on would not help.
    for header in ["x-hwid-max-devices-reached", "x-hwid-limit"] {
        let full = refusal(200, with(if header == "x-hwid-limit" { &["x-hwid-limit"] } else { &["x-hwid-max-devices-reached"] }), true).unwrap();
        assert!(full.contains("device limit"), "{full}");
    }
    // A plain 404 still mentions the device switch as one possibility, but only if it is off.
    assert!(refusal(404, none, false).unwrap().contains("Send device ID"));
    assert!(!refusal(404, none, true).unwrap().contains("Send device ID"));
    assert!(refusal(403, none, false).unwrap().contains("fresh one"));
    assert_eq!(refusal(502, none, false).unwrap(), "The subscription server replied 502. The problem is on the provider's side; try again later.");
}

/// A throwaway HTTP server that answers once and hands back the request it received.
fn serve_once(head: &'static str, body: &'static str) -> (String, std::thread::JoinHandle<String>) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/sub/secret-token", listener.local_addr().unwrap());
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut raw = Vec::new();
        let mut buf = [0u8; 4096];
        while !String::from_utf8_lossy(&raw).contains("\r\n\r\n") {
            let n = stream.read(&mut buf).unwrap_or(0);
            if n == 0 {
                break;
            }
            raw.extend_from_slice(&buf[..n]);
        }
        let reply = format!("HTTP/1.1 {head}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        stream.write_all(reply.as_bytes()).unwrap();
        String::from_utf8_lossy(&raw).to_lowercase()
    });
    (url, handle)
}

fn run<T>(future: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(future)
}

#[test]
fn the_device_id_and_client_name_are_sent_only_when_asked_for() {
    let body = "proxies:\n  - { name: Clash server, type: trojan, server: trojan.example.com, port: 443, password: secret }\n";
    let (url, server) = serve_once("200 OK\r\nsubscription-userinfo: upload=1; download=2; total=0", body);
    let options = FetchOptions { user_agent: Some("SFM/1.14.2".into()), device_id: Some("11111111-2222-4333-8444-555555555555".into()) };
    let (profiles, info, _) = run(fetch(&url, &options)).unwrap();
    let request = server.join().unwrap();
    assert!(request.contains("user-agent: sfm/1.14.2"), "{request}");
    assert!(request.contains("x-hwid: 11111111-2222-4333-8444-555555555555"), "{request}");
    assert!(request.contains("x-device-os: macos"), "{request}");
    // A Clash list in a subscription is read like a list of links.
    assert_eq!(profiles[0].name, "Clash server");
    assert_eq!(info.map(|i| i.total), Some(0));

    let (url, server) = serve_once("200 OK", "trojan://secret@trojan.example.com:443#Plain");
    run(fetch(&url, &FetchOptions::default())).unwrap();
    let request = server.join().unwrap();
    assert!(!request.contains("x-hwid"), "nothing identifying is sent unless the user turns it on");
    assert!(request.contains(&format!("user-agent: {}", crate::app_config::user_agent().to_lowercase())));
}

#[test]
fn a_device_limited_provider_gets_a_clear_message_without_the_token() {
    let (url, server) = serve_once("404 Not Found\r\nx-hwid-active: true\r\nx-hwid-not-supported: true", "");
    let error = run(fetch(&url, &FetchOptions::default())).unwrap_err().to_string();
    server.join().unwrap();
    assert!(error.contains("Send device ID"), "{error}");
    assert!(!error.contains("secret-token"), "the link's token must not end up in messages: {error}");
}
