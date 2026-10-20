use super::*;
use crate::settings::{Action, RuleSetRef};
use std::io::{Read, Write};
use std::net::TcpListener;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kage-rulesets-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn set(kind: RuleSetKind, country: &str, enabled: bool) -> RuleSetRef {
    RuleSetRef { id: country.into(), enabled, kind, country: country.into(), action: Action::Direct }
}

fn routing(mode: RouteMode, sets: Vec<RuleSetRef>) -> RoutingSettings {
    RoutingSettings { mode, rule_sets: sets, ..RoutingSettings::default() }
}

fn item(tag_kind: RuleSetKind, name: &str) -> Wanted {
    Wanted { tag: rule_set_tag(tag_kind, name).unwrap(), kind: tag_kind, name: name.into() }
}

const SRS: &[u8] = b"SRS\x01 a pretend compiled list";

/// A local web server answering every request with `status` and `body`.
fn site(status: &'static str, body: &'static [u8]) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/geoip-cn.srs", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        while let Ok((mut stream, _)) = listener.accept() {
            let _ = stream.read(&mut [0u8; 2048]);
            let _ = stream.write_all(format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).as_bytes());
            let _ = stream.write_all(body);
        }
    });
    url
}

fn dead_site() -> String {
    format!("http://{}/geoip-cn.srs", TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap())
}

fn run<T>(future: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(future)
}

#[test]
fn only_rules_mode_asks_for_lists_and_each_list_counts_once() {
    let sets = vec![
        set(RuleSetKind::Geoip, "CN", true),
        set(RuleSetKind::Geoip, "cn", true),
        set(RuleSetKind::Geosite, "category-ads-all", true),
        set(RuleSetKind::Geoip, "ir", false),
        set(RuleSetKind::Geoip, "../etc", true),
        set(RuleSetKind::Geoip, "  ", true),
    ];
    let tags: Vec<String> = wanted(&routing(RouteMode::Rule, sets.clone())).into_iter().map(|w| w.tag).collect();
    assert_eq!(tags, ["geoip-cn", "geosite-category-ads-all"]);
    assert!(wanted(&routing(RouteMode::Global, sets.clone())).is_empty());
    assert!(wanted(&routing(RouteMode::Direct, sets)).is_empty());
}

fn default_provider() -> RuleProvider {
    provider_of(&RoutingSettings::default())
}

#[test]
fn a_list_is_fetched_from_github_first_and_a_cdn_second() {
    let urls = sources(&item(RuleSetKind::Geoip, "cn"), &default_provider());
    assert_eq!(urls.len(), 2);
    assert!(urls[0].starts_with("https://raw.githubusercontent.com/") && urls[0].ends_with("/geoip-cn.srs"));
    assert!(urls[1].starts_with("https://cdn.jsdelivr.net/") && urls[1].ends_with("/geoip-cn.srs"));
    assert!(sources(&item(RuleSetKind::Geosite, "category-ads-all"), &default_provider())[0].ends_with("/geosite-category-ads-all.srs"));
}

#[test]
fn the_cdn_copy_of_a_github_address_is_worked_out_from_it() {
    assert_eq!(
        jsdelivr_mirror("https://raw.githubusercontent.com/SagerNet/sing-geoip/rule-set/geoip-cn.srs").as_deref(),
        Some("https://cdn.jsdelivr.net/gh/SagerNet/sing-geoip@rule-set/geoip-cn.srs")
    );
    assert_eq!(jsdelivr_mirror("https://example.com/a/b/c.srs"), None);
    assert_eq!(jsdelivr_mirror("https://raw.githubusercontent.com/o/r/branch"), None);
}

#[test]
fn every_built_in_provider_has_a_unique_id_and_working_templates() {
    let all = builtin_providers();
    assert_eq!(all[0].id, DEFAULT_PROVIDER);
    let mut ids: Vec<&str> = all.iter().map(|p| p.id.as_str()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), all.len());
    for provider in &all {
        assert!(!provider.geoip.is_empty() || !provider.geosite.is_empty(), "{}", provider.name);
        for template in [&provider.geoip, &provider.geosite].into_iter().filter(|t| !t.is_empty()) {
            assert!(template.starts_with("https://") && template.contains("{name}"), "{}: {template}", provider.name);
        }
    }
    // Same file, different home: the name is filled in per provider.
    let meta = all.iter().find(|p| p.id == "metacubex").unwrap();
    assert_eq!(
        sources(&item(RuleSetKind::Geosite, "cn"), meta)[0],
        "https://raw.githubusercontent.com/MetaCubeX/meta-rules-dat/sing/geo/geosite/cn.srs"
    );
}

#[test]
fn a_provider_without_site_lists_offers_none_and_says_why_when_asked_to_download() {
    let loyalsoldier = builtin_providers().into_iter().find(|p| p.id == "loyalsoldier").unwrap();
    let site_list = item(RuleSetKind::Geosite, "cn");
    assert!(sources(&site_list, &loyalsoldier).is_empty());
    let dir = scratch("no-site-lists");
    let error = run(download(&dir, &site_list, &[], None)).unwrap_err().to_string();
    assert!(error.contains("no address"), "{error}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_selected_provider_is_found_among_built_in_and_custom_ones_with_a_fallback() {
    let custom = RuleProvider { id: "mine".into(), name: "Mine".into(), geoip: "https://example.com/{name}.srs".into(), ..RuleProvider::default() };
    let mut routing = RoutingSettings { custom_providers: vec![custom], ..RoutingSettings::default() };
    assert_eq!(provider_of(&routing).id, DEFAULT_PROVIDER);
    routing.rule_provider = "mine".into();
    assert_eq!(provider_of(&routing).name, "Mine");
    routing.rule_provider = "metacubex".into();
    assert_eq!(provider_of(&routing).id, "metacubex");
    routing.rule_provider = "removed-long-ago".into();
    assert_eq!(provider_of(&routing).id, DEFAULT_PROVIDER, "an unknown id never leaves the app without a source");
}

#[test]
fn custom_providers_are_checked_when_saved() {
    let ok = RuleProvider { id: "a".into(), name: "A".into(), geoip: "https://example.com/geoip-{name}.srs".into(), ..RuleProvider::default() };
    assert!(validate_providers(std::slice::from_ref(&ok)).is_ok());
    let bad = |change: fn(&mut RuleProvider)| {
        let mut provider = ok.clone();
        change(&mut provider);
        validate_providers(&[provider]).is_err()
    };
    assert!(bad(|p| p.name = "  ".into()), "needs a name");
    assert!(bad(|p| p.geoip = String::new()), "needs at least one address");
    assert!(bad(|p| p.geoip = "http://example.com/{name}.srs".into()), "https only");
    assert!(bad(|p| p.geoip = "https://example.com/cn.srs".into()), "needs {{name}}");
    assert!(bad(|p| p.geoip = "https://example.com/a b/{name}.srs".into()), "no spaces");
    assert!(bad(|p| p.id = "sagernet".into()), "cannot reuse a built-in id");
    assert!(validate_providers(&[ok.clone(), ok]).is_err(), "ids are unique");
}

#[test]
fn missing_and_old_lists_are_told_apart() {
    let dir = scratch("plan");
    let (fresh, old, absent, empty) = (item(RuleSetKind::Geoip, "cn"), item(RuleSetKind::Geoip, "ir"), item(RuleSetKind::Geoip, "ru"), item(RuleSetKind::Geoip, "us"));
    std::fs::write(fresh.path(&dir), SRS).unwrap();
    std::fs::write(old.path(&dir), SRS).unwrap();
    std::fs::write(empty.path(&dir), b"").unwrap();
    let long_ago = SystemTime::now() - MAX_AGE - Duration::from_secs(3600);
    std::fs::OpenOptions::new().write(true).open(old.path(&dir)).unwrap().set_modified(long_ago).unwrap();

    let (missing, stale) = plan(&dir, &[fresh, old.clone(), absent.clone(), empty.clone()], MAX_AGE);
    assert_eq!(missing, [absent, empty], "no file, or an empty one, cannot be used");
    assert_eq!(stale, [old], "usable, but due a refresh");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn only_real_rule_set_files_are_accepted() {
    assert!(is_rule_set(SRS));
    assert!(!is_rule_set(b"<html>Sign in to the hotel Wi-Fi</html>"));
    assert!(!is_rule_set(b""));
    assert!(!is_rule_set(b"SRS"));
}

#[test]
fn a_blocked_first_source_falls_back_to_the_mirror() {
    let dir = scratch("mirror");
    let cn = item(RuleSetKind::Geoip, "cn");
    run(download(&dir, &cn, &[dead_site(), site("200 OK", SRS)], None)).unwrap();
    assert_eq!(std::fs::read(cn.path(&dir)).unwrap(), SRS);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_login_page_is_never_saved_as_a_list() {
    // Hotel and airport Wi-Fi answer every request with a sign-in page. Saved as a list, it would
    // stop sing-box from starting, on every connect, until someone deleted the file.
    let dir = scratch("portal");
    let cn = item(RuleSetKind::Geoip, "cn");
    let portal = site("200 OK", b"<html><body>Please sign in</body></html>");
    assert!(run(download(&dir, &cn, &[portal], None)).is_err());
    assert!(!cn.path(&dir).exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_failed_refresh_keeps_the_copy_already_there() {
    let dir = scratch("keep");
    let cn = item(RuleSetKind::Geoip, "cn");
    std::fs::write(cn.path(&dir), SRS).unwrap();
    assert!(run(download(&dir, &cn, &[dead_site(), site("503 Service Unavailable", b"")], None)).is_err());
    assert_eq!(std::fs::read(cn.path(&dir)).unwrap(), SRS);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fetching_many_lists_reports_exactly_the_ones_that_failed() {
    let dir = scratch("many");
    let (good, bad) = (item(RuleSetKind::Geoip, "cn"), item(RuleSetKind::Geosite, "ads"));
    let (good_url, bad_url) = (site("200 OK", SRS), dead_site());
    let sources_of: SourcesOf = {
        let good_tag = good.tag.clone();
        Arc::new(move |w: &Wanted| vec![if w.tag == good_tag { good_url.clone() } else { bad_url.clone() }])
    };
    let failed = run(fetch_all_from(&dir, vec![good.clone(), bad.clone()], sources_of, None));
    assert_eq!(failed.iter().map(|(w, _)| w.tag.as_str()).collect::<Vec<_>>(), [bad.tag.as_str()]);
    assert!(good.path(&dir).is_file() && !bad.path(&dir).exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_notification_names_the_lists_in_plain_words() {
    assert!(skipped_message(&["geoip-cn".into()]).starts_with("geoip-cn could not be downloaded, so it is not applied."));
    assert!(skipped_message(&["geoip-cn".into(), "geosite-ads".into()]).contains("geoip-cn, geosite-ads could not be downloaded, so they are not applied"));
}

#[test]
fn a_list_that_just_failed_is_not_waited_for_again_but_is_retried_later() {
    let start = Instant::now();
    let mut cooldown = Cooldown::default();
    assert!(!cooldown.cooling("geoip-cn", start), "never failed");

    cooldown.failed("geoip-cn", start);
    assert!(cooldown.cooling("geoip-cn", start + Duration::from_secs(30)), "a reconnect attempt a moment later skips it");
    assert!(!cooldown.cooling("geoip-ir", start + Duration::from_secs(30)), "other lists are unaffected");
    assert!(!cooldown.cooling("geoip-cn", start + RETRY_AFTER + Duration::from_secs(1)), "a later attempt tries again");

    cooldown.failed("geoip-cn", start);
    cooldown.succeeded("geoip-cn");
    assert!(!cooldown.cooling("geoip-cn", start), "success clears it");
}

#[test]
fn failures_seen_by_fetch_all_put_the_list_on_cooldown() {
    let dir = scratch("cooldown");
    let blocked = item(RuleSetKind::Geoip, "cooldown-test-list");
    let dead = dead_site();
    let sources_of: SourcesOf = Arc::new(move |_| vec![dead.clone()]);
    assert!(skip_recent_failures(vec![blocked.clone()]).len() == 1);
    assert_eq!(run(fetch_all_from(&dir, vec![blocked.clone()], sources_of, None)).len(), 1);
    assert!(skip_recent_failures(vec![blocked]).is_empty(), "the next attempt does not wait for it again");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn through_the_tunnel_comes_first_and_direct_access_is_only_the_fallback() {
    // A plain-HTTP proxy receives the whole address, so a local web server stands in for the tunnel.
    let dir = scratch("tunnel-first");
    let cn = item(RuleSetKind::Geoip, "cn");
    let direct_hits = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let direct = {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/geoip-cn.srs", listener.local_addr().unwrap());
        let hits = direct_hits.clone();
        std::thread::spawn(move || {
            while let Ok((mut stream, _)) = listener.accept() {
                hits.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let _ = stream.read(&mut [0u8; 2048]);
                let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            }
        });
        url
    };
    let tunnel = site("200 OK", SRS);
    let proxy = tunnel.trim_end_matches("/geoip-cn.srs").to_string();
    run(download(&dir, &cn, std::slice::from_ref(&direct), Some(&proxy))).unwrap();
    assert_eq!(std::fs::read(cn.path(&dir)).unwrap(), SRS);
    assert_eq!(direct_hits.load(std::sync::atomic::Ordering::SeqCst), 0, "the tunnel answered, so the blocked network was never asked");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_dead_tunnel_falls_back_to_direct_access() {
    let dir = scratch("tunnel-dead");
    let cn = item(RuleSetKind::Geoip, "cn");
    let dead_proxy = dead_site().trim_end_matches("/geoip-cn.srs").to_string();
    run(download(&dir, &cn, &[site("200 OK", SRS)], Some(&dead_proxy))).unwrap();
    assert_eq!(std::fs::read(cn.path(&dir)).unwrap(), SRS);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_manager_lists_what_is_on_disk_and_deletes_it() {
    let dir = scratch("manage");
    let (cn, ads) = (item(RuleSetKind::Geoip, "cn"), item(RuleSetKind::Geosite, "category-ads-all"));
    let routing = routing(RouteMode::Global, vec![set(RuleSetKind::Geoip, "cn", true), set(RuleSetKind::Geosite, "category-ads-all", true), set(RuleSetKind::Geoip, "ru", false)]);
    std::fs::write(cn.path(&dir), SRS).unwrap();

    // Listed in any routing mode, disabled lists left out, present and missing told apart.
    let listed = status(&dir, &routing);
    assert_eq!(listed.iter().map(|l| (l.tag.as_str(), l.present)).collect::<Vec<_>>(), [("geoip-cn", true), ("geosite-category-ads-all", false)]);
    assert_eq!(listed[0].bytes, SRS.len() as u64);
    assert!(listed[0].modified.is_some() && listed[1].modified.is_none());

    std::fs::write(ads.path(&dir), SRS).unwrap();
    remember_provider(&dir, "metacubex");
    assert_eq!(downloaded_from(&dir).as_deref(), Some("metacubex"));
    remove(&dir, Some("geoip-cn")).unwrap();
    assert!(!cn.path(&dir).exists() && ads.path(&dir).exists());
    assert!(remove(&dir, Some("../escape")).is_err(), "only list names are accepted");
    std::fs::write(dir.join("notes.txt"), "keep").unwrap();
    remove(&dir, None).unwrap();
    assert!(!ads.path(&dir).exists() && downloaded_from(&dir).is_none());
    assert!(dir.join("notes.txt").exists(), "files that are not lists are left alone");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn settings_saved_before_list_sources_existed_use_the_default_source() {
    let old: RoutingSettings = serde_json::from_str(r#"{"mode":"rule","ruleSets":[{"id":"1","enabled":true,"kind":"geoip","country":"cn","action":"direct"}]}"#).unwrap();
    assert_eq!(old.rule_provider, DEFAULT_PROVIDER);
    assert!(old.custom_providers.is_empty());
    assert_eq!(provider_of(&old).id, DEFAULT_PROVIDER);
    assert_eq!(wanted(&old).len(), 1);
}
