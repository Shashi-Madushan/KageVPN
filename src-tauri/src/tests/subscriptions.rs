use super::*;

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
