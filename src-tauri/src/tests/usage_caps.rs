use super::*;
use chrono::TimeZone;

const GB: u64 = 1024 * 1024 * 1024;
const SLACK: &str = "/Applications/Slack.app";

fn at(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Local> {
    Local.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
}

fn cap(limit: u64, at_limit: AtLimit) -> AppCap {
    AppCap { path: SLACK.into(), limit_bytes: limit, period: CapPeriod::Day, at_limit }
}

fn date(y: i32, m: u32, d: u32) -> i64 {
    midnight(NaiveDate::from_ymd_opt(y, m, d).unwrap())
}

// ── Periods ─────────────────────────────────────────────────────────────────

#[test]
fn a_day_starts_at_local_midnight() {
    assert_eq!(period_start(at(2026, 10, 20, 15, 30), CapPeriod::Day), date(2026, 10, 20));
    assert_eq!(period_start(at(2026, 10, 20, 0, 0), CapPeriod::Day), date(2026, 10, 20));
    assert_eq!(period_end(at(2026, 10, 20, 23, 59), CapPeriod::Day), date(2026, 10, 21));
}

#[test]
fn a_week_starts_on_monday() {
    // 20 Oct 2026 is a Tuesday.
    assert_eq!(period_start(at(2026, 10, 20, 9, 0), CapPeriod::Week), date(2026, 10, 19));
    assert_eq!(period_start(at(2026, 10, 25, 23, 59), CapPeriod::Week), date(2026, 10, 19));
    assert_eq!(period_start(at(2026, 10, 26, 0, 0), CapPeriod::Week), date(2026, 10, 26));
    assert_eq!(period_end(at(2026, 10, 20, 9, 0), CapPeriod::Week), date(2026, 10, 26));
}

#[test]
fn a_month_starts_on_the_first_and_ends_at_the_next_one() {
    assert_eq!(period_start(at(2026, 10, 31, 23, 0), CapPeriod::Month), date(2026, 10, 1));
    assert_eq!(period_end(at(2026, 10, 31, 23, 0), CapPeriod::Month), date(2026, 11, 1));
    assert_eq!(period_end(at(2026, 12, 15, 8, 0), CapPeriod::Month), date(2027, 1, 1));
    assert_eq!(period_end(at(2028, 2, 10, 8, 0), CapPeriod::Month), date(2028, 3, 1));
}

#[test]
fn levels_switch_at_eighty_and_one_hundred_percent() {
    assert_eq!(level(0, 100), 0);
    assert_eq!(level(79, 100), 0);
    assert_eq!(level(80, 100), 80);
    assert_eq!(level(99, 100), 80);
    assert_eq!(level(100, 100), 100);
    assert_eq!(level(500, 100), 100);
    assert_eq!(level(5, 0), 0);
    // Big numbers do not overflow.
    assert_eq!(level(u64::MAX, u64::MAX), 100);
    assert_eq!(level(u64::MAX / 10 * 8 + 10, u64::MAX), 80);
}

// ── Tally ───────────────────────────────────────────────────────────────────

#[test]
fn bytes_raise_each_alert_once_per_period() {
    let c = cap(100, AtLimit::Notify);
    let mut t = Tally::new(SLACK, 1000);
    assert!(!accumulate(&mut t, &c, 1000, (10, 10)).any());
    let change = accumulate(&mut t, &c, 1000, (30, 30));
    assert_eq!(change.new_alert, Some(80));
    assert!(!accumulate(&mut t, &c, 1000, (1, 0)).any(), "no repeat at 80");
    assert_eq!(accumulate(&mut t, &c, 1000, (20, 0)).new_alert, Some(100));
    assert!(!accumulate(&mut t, &c, 1000, (50, 50)).any(), "no repeat at 100");
    assert_eq!(t.offer, Offer::None, "notify-only caps never offer a block");
}

#[test]
fn jumping_straight_past_the_limit_sends_only_the_hundred_alert() {
    let mut t = Tally::new(SLACK, 1000);
    let change = accumulate(&mut t, &cap(100, AtLimit::Notify), 1000, (0, 150));
    assert_eq!(change.new_alert, Some(100));
}

#[test]
fn a_cap_that_offers_a_block_raises_the_offer_once() {
    let c = cap(100, AtLimit::OfferBlock);
    let mut t = Tally::new(SLACK, 1000);
    let change = accumulate(&mut t, &c, 1000, (60, 60));
    assert!(change.offer_raised);
    assert_eq!(t.offer, Offer::Pending);
    assert!(!accumulate(&mut t, &c, 1000, (1, 1)).offer_raised);
    t.offer = Offer::Dismissed;
    assert!(!accumulate(&mut t, &c, 1000, (1, 1)).offer_raised, "not again after Not now");
}

#[test]
fn changing_to_offer_block_while_over_the_limit_raises_the_offer() {
    let mut t = Tally::new(SLACK, 1000);
    accumulate(&mut t, &cap(100, AtLimit::Notify), 1000, (200, 0));
    assert_eq!(t.offer, Offer::None);
    assert!(accumulate(&mut t, &cap(100, AtLimit::OfferBlock), 1000, (0, 0)).offer_raised);
}

#[test]
fn a_new_period_starts_over() {
    let c = cap(100, AtLimit::OfferBlock);
    let mut t = Tally::new(SLACK, 1000);
    accumulate(&mut t, &c, 1000, (100, 100));
    assert_eq!(t.offer, Offer::Pending);
    let change = accumulate(&mut t, &c, 2000, (5, 5));
    assert!(change.rolled_over);
    assert_eq!((t.period_start, t.up, t.down, t.alert_level, t.offer), (2000, 5, 5, 0, Offer::None));
}

#[test]
fn raising_the_limit_never_repeats_an_alert() {
    let mut t = Tally::new(SLACK, 1000);
    accumulate(&mut t, &cap(100, AtLimit::Notify), 1000, (90, 0));
    assert_eq!(t.alert_level, 80);
    // The limit doubles: usage is now below 80 percent, then crosses it again.
    assert!(!accumulate(&mut t, &cap(200, AtLimit::Notify), 1000, (80, 0)).any());
}

// ── Validation ──────────────────────────────────────────────────────────────

#[test]
fn caps_need_an_app_and_a_sensible_limit() {
    assert!(validate_cap(&cap(GB, AtLimit::Notify)).is_ok());
    assert!(validate_cap(&cap(MIN_LIMIT - 1, AtLimit::Notify)).is_err());
    assert!(validate_cap(&cap(MAX_LIMIT + 1, AtLimit::Notify)).is_err());
    let mut c = cap(GB, AtLimit::Notify);
    c.path = "/usr/bin/curl".into();
    assert!(validate_cap(&c).is_err());
    c.path = "Slack.app".into();
    assert!(validate_cap(&c).is_err());
}

#[test]
fn this_app_and_the_engine_cannot_have_a_cap() {
    let mut c = cap(GB, AtLimit::OfferBlock);
    c.path = "/Applications/Kage VPN.app".into();
    let err = validate_cap(&c).unwrap_err().to_string();
    assert!(err.contains("cannot have a data cap"), "{err}");
}

// ── Rules ───────────────────────────────────────────────────────────────────

fn manual(value: &str, action: Action) -> Rule {
    Rule { id: value.into(), enabled: true, kind: RuleKind::App, value: value.into(), action, origin: RuleOrigin::Manual }
}

#[test]
fn a_cap_block_goes_on_top_and_leaves_the_users_rule_underneath() {
    let mut rules = vec![manual(SLACK, Action::Direct), manual("/Applications/Zoom.app", Action::Proxy)];
    apply_cap_rule(&mut rules, SLACK);
    apply_cap_rule(&mut rules, SLACK);
    assert_eq!(rules.len(), 3, "applying twice adds one");
    assert_eq!((rules[0].origin, rules[0].action), (RuleOrigin::Cap, Action::Block));
    assert!(has_cap_rule(&rules, SLACK));
    assert!(lift_cap_rule(&mut rules, SLACK));
    assert!(!lift_cap_rule(&mut rules, SLACK));
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].action, Action::Direct);
}

#[test]
fn saving_the_routing_page_keeps_stored_cap_blocks_and_ignores_incoming_ones() {
    let mut stored = vec![manual("/Applications/Zoom.app", Action::Proxy)];
    apply_cap_rule(&mut stored, SLACK);
    // A stale draft without the cap block, plus one that tries to invent a cap block.
    let mut incoming = vec![manual("/Applications/Zoom.app", Action::Direct)];
    apply_cap_rule(&mut incoming, "/Applications/Other.app");
    let kept = keep_cap_rules(&stored, incoming);
    assert!(has_cap_rule(&kept, SLACK));
    assert!(!has_cap_rule(&kept, "/Applications/Other.app"));
    assert_eq!(kept.len(), 2);
    assert_eq!(kept[1].action, Action::Direct, "the user's edit still applies");
}

#[test]
fn a_block_switches_global_and_direct_routing_to_rules_mode() {
    let mut routing = RoutingSettings { mode: RouteMode::Global, ..RoutingSettings::default() };
    ensure_rules_mode(&mut routing);
    assert_eq!((routing.mode, routing.final_action), (RouteMode::Rule, Action::Proxy));
    let mut routing = RoutingSettings { mode: RouteMode::Direct, ..RoutingSettings::default() };
    ensure_rules_mode(&mut routing);
    assert_eq!((routing.mode, routing.final_action), (RouteMode::Rule, Action::Direct));
    let mut routing = RoutingSettings { mode: RouteMode::Rule, final_action: Action::Direct, ..RoutingSettings::default() };
    ensure_rules_mode(&mut routing);
    assert_eq!(routing.final_action, Action::Direct, "rules mode is left alone");
}

#[test]
fn the_switch_to_rules_mode_counts_the_saved_rules_and_lists_it_turns_on() {
    let rule = |enabled| Rule { id: "r".into(), enabled, kind: RuleKind::DomainSuffix, value: "example.com".into(), ..Rule::default() };
    let list = |enabled| crate::settings::RuleSetRef { enabled, country: "cn".into(), ..Default::default() };
    let mut routing = RoutingSettings { mode: RouteMode::Global, rules: vec![rule(true), rule(true), rule(false)], rule_sets: vec![list(true), list(false)], ..RoutingSettings::default() };
    assert_eq!(ensure_rules_mode(&mut routing), (2, 1), "switched off ones stay off");
    assert_eq!(ensure_rules_mode(&mut routing), (0, 0), "already in Rules mode: nothing new applies");
}

#[test]
fn the_notice_names_what_now_applies_and_is_skipped_when_nothing_does() {
    assert_eq!(now_active_message(0, 0), None);
    let one = now_active_message(1, 0).unwrap();
    assert!(one.contains("1 saved rule from Routing"), "{one}");
    let both = now_active_message(3, 2).unwrap();
    assert!(both.contains("3 saved rules and 2 lists"), "{both}");
    assert!(now_active_message(0, 1).unwrap().contains("1 saved list from"));
}

// ── Status ──────────────────────────────────────────────────────────────────

#[test]
fn the_state_follows_the_level_the_offer_and_whether_usage_is_counted() {
    assert_eq!(state_of(true, 0, Offer::None, false), CapState::Ok);
    assert_eq!(state_of(true, 80, Offer::None, false), CapState::Warn80);
    assert_eq!(state_of(true, 100, Offer::Pending, false), CapState::Over);
    assert_eq!(state_of(true, 100, Offer::Dismissed, false), CapState::Dismissed);
    assert_eq!(state_of(true, 100, Offer::Taken, true), CapState::Blocked);
    assert_eq!(state_of(false, 100, Offer::Pending, false), CapState::NotCounting);
    assert_eq!(state_of(false, 0, Offer::None, true), CapState::Blocked, "a block shows even when not counting");
}

#[test]
fn a_tally_from_an_earlier_period_reads_as_empty() {
    let now = at(2026, 10, 20, 12, 0);
    let c = cap(GB, AtLimit::OfferBlock);
    let mut old = Tally::new(SLACK, date(2026, 10, 19));
    old.up = 2 * GB;
    let status = status_of(&c, Some(&old), true, false, true, now);
    assert_eq!((status.used, status.level), (0, 0));
    let mut current = Tally::new(SLACK, date(2026, 10, 20));
    current.down = GB / 2;
    current.offer = Offer::None;
    let status = status_of(&c, Some(&current), true, false, true, now);
    assert_eq!((status.used, status.level, status.state), (GB / 2, 0, CapState::Ok));
    assert_eq!(status.period_end, date(2026, 10, 21));
    assert_eq!(status.name, "Slack");
}

// ── Store and settings ──────────────────────────────────────────────────────

fn temp_db(name: &str) -> (super::super::store::Db, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("kage-caps-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    (super::super::store::Db::open(&dir.join("usage.db")).unwrap(), dir)
}

#[test]
fn tallies_are_saved_loaded_and_survive_clearing_the_usage_history() {
    let (db, dir) = temp_db("tally");
    assert_eq!(db.tally(SLACK).unwrap(), None);
    let mut t = Tally::new(SLACK, 1000);
    t.up = 5;
    t.down = 7;
    t.alert_level = 80;
    t.offer = Offer::Pending;
    db.save_tally(&t).unwrap();
    assert_eq!(db.tally(SLACK).unwrap(), Some(t.clone()));
    t.up = 9;
    db.save_tally(&t).unwrap();
    assert_eq!(db.tally(SLACK).unwrap().unwrap().up, 9, "saving again replaces");
    db.clear_all().unwrap();
    db.clear_details().unwrap();
    assert_eq!(db.tally(SLACK).unwrap().unwrap().up, 9, "history clears leave caps alone");
    db.delete_tally(SLACK).unwrap();
    assert_eq!(db.tally(SLACK).unwrap(), None);
    drop(db);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn old_usage_settings_load_without_caps_and_caps_round_trip() {
    let old: crate::settings::UsageSettings = serde_json::from_str(r#"{"paused":false,"recordApps":true}"#).unwrap();
    assert!(old.app_caps.is_empty());
    let mut settings = crate::settings::UsageSettings::default();
    settings.app_caps.push(cap(GB, AtLimit::OfferBlock));
    let text = serde_json::to_string(&settings).unwrap();
    assert!(text.contains(r#""atLimit":"offerBlock""#) && text.contains(r#""period":"day""#), "{text}");
    let back: crate::settings::UsageSettings = serde_json::from_str(&text).unwrap();
    assert_eq!(back.app_caps, settings.app_caps);
}

#[test]
fn saving_usage_settings_keeps_the_stored_caps_and_resets_the_alert_when_the_quota_changes() {
    use crate::usage::commands::merge_usage_settings;
    let mut current = crate::settings::UsageSettings::default();
    current.app_caps.push(cap(GB, AtLimit::Notify));
    current.quota_bytes = Some(100);
    current.alert_level = 80;

    // A page that loaded before the cap existed sends none, and a wrong cycle day.
    let sent = crate::settings::UsageSettings { quota_bytes: Some(200), alert_level: 80, cycle_day: 99, ..Default::default() };
    let saved = merge_usage_settings(&current, sent);
    assert_eq!(saved.app_caps, current.app_caps);
    assert_eq!((saved.alert_level, saved.cycle_day), (0, 31));

    let mut same = current.clone();
    same.app_caps.clear();
    assert_eq!(merge_usage_settings(&current, same).alert_level, 80, "an unchanged quota keeps its alert");
}

#[test]
fn a_page_loaded_before_a_warning_does_not_bring_the_warning_back() {
    use crate::usage::commands::merge_usage_settings;
    let mut current = crate::settings::UsageSettings { quota_bytes: Some(100), ..Default::default() };
    let page = current.clone();
    // The 80% warning is shown after the page loaded its copy.
    (current.alert_cycle, current.alert_level) = (1_700_000_000, 80);

    let sent = crate::settings::UsageSettings { record_domains: true, ..page };
    let saved = merge_usage_settings(&current, sent);
    assert_eq!((saved.alert_cycle, saved.alert_level), (1_700_000_000, 80));
    assert!(saved.record_domains, "the change itself is saved");
}
