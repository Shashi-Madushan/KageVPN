use super::*;
use crate::profile::parse_many;

const DAY: u64 = 24 * 3600;

#[test]
fn a_subscription_without_an_expiry_date_never_warns() {
    assert_eq!(due_expiry_notice(0, 1_000, (0, 0)), None);
}

#[test]
fn a_far_off_expiry_is_quiet_then_warns_once_then_says_it_ended_once() {
    let expire = 100 * DAY;
    // Weeks away: nothing to say.
    assert_eq!(due_expiry_notice(expire, 50 * DAY, (0, 0)), None);
    // Within three days: warn once.
    assert_eq!(due_expiry_notice(expire, expire - 2 * DAY, (0, 0)), Some(Expiry::Soon));
    assert_eq!(due_expiry_notice(expire, expire - DAY, (expire, 1)), None, "already warned for this date");
    // Past the date: say so once, then stay quiet. This used to repeat on every refresh, forever.
    assert_eq!(due_expiry_notice(expire, expire + 1, (expire, 1)), Some(Expiry::Expired));
    assert_eq!(due_expiry_notice(expire, expire + 12 * 3600, (expire, 2)), None);
    assert_eq!(due_expiry_notice(expire, expire + 30 * DAY, (expire, 2)), None);
}

#[test]
fn renewing_starts_the_warnings_over() {
    let (old, renewed) = (100 * DAY, 130 * DAY);
    assert_eq!(due_expiry_notice(renewed, renewed - DAY, (old, 2)), Some(Expiry::Soon), "a new date is a new subscription period");
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

    let (result, notice) = record_refresh(&mut store, "s", Ok((servers(&["a.example.com"]), None, vec![])), 5_000);
    assert!(result.is_err() && notice.is_none());
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
    assert_eq!(first.map(|(name, _, notice)| (name, notice)), Some(("Provider".to_string(), Expiry::Expired)));
    let (_, again) = record_refresh(&mut store, "s", Ok((servers(&["a.example.com"]), info(), vec![])), 40_000);
    assert!(again.is_none(), "the same expiry is not announced again");
    let _ = std::fs::remove_dir_all(dir);
}
