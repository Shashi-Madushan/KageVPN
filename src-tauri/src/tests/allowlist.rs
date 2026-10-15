use super::*;

fn servers(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| n.to_string()).collect()
}

fn v(addresses: &[&str]) -> Vec<String> {
    addresses.iter().map(|a| a.to_string()).collect()
}

#[test]
fn addresses_from_several_lookups_are_all_kept() {
    let mut history = History::default();
    history.record("cdn.example", v(&["192.0.2.1"]));
    history.record("cdn.example", v(&["192.0.2.2", "2001:db8::1"]));
    assert_eq!(history.addresses(&servers(&["cdn.example"])), v(&["192.0.2.1", "192.0.2.2", "2001:db8::1"]));
}

#[test]
fn an_address_that_stopped_appearing_is_dropped_after_a_few_lookups() {
    let mut history = History::default();
    history.record("s", v(&["192.0.2.1"]));
    for last in ["192.0.2.2", "192.0.2.3", "192.0.2.4"] {
        history.record("s", v(&[last]));
    }
    let known = history.addresses(&servers(&["s"]));
    assert!(!known.contains(&"192.0.2.1".to_string()), "{known:?}");
    assert_eq!(known.len(), REMEMBERED_LOOKUPS);
}

#[test]
fn a_failed_lookup_changes_nothing() {
    let mut history = History::default();
    history.record("s", v(&["192.0.2.1"]));
    for _ in 0..10 {
        history.record("s", Vec::new());
    }
    assert_eq!(history.addresses(&servers(&["s"])), v(&["192.0.2.1"]));
}

#[test]
fn the_same_answer_twice_does_not_push_older_ones_out() {
    let mut history = History::default();
    history.record("s", v(&["192.0.2.1"]));
    history.record("s", v(&["192.0.2.2"]));
    for _ in 0..10 {
        history.record("s", v(&["192.0.2.2"]));
    }
    assert_eq!(history.addresses(&servers(&["s"])), v(&["192.0.2.1", "192.0.2.2"]));
}

#[test]
fn servers_no_longer_in_use_are_forgotten_and_the_list_is_capped() {
    let mut history = History::default();
    history.record("old", v(&["192.0.2.9"]));
    history.record("new", (0..100).map(|i| format!("198.51.100.{i}")).collect());
    history.keep_only(&servers(&["new"]));
    let known = history.addresses(&servers(&["old", "new"]));
    assert!(!known.contains(&"192.0.2.9".to_string()));
    assert_eq!(known.len(), MAX_ADDRESSES);
}

#[test]
fn refreshing_writes_the_list_the_helper_reads_and_keeps_old_answers_when_a_lookup_fails() {
    let dir = std::env::temp_dir().join(format!("kage-allow-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let paths = crate::paths::AppPaths::new(dir.clone()).unwrap();
    let run = |future| tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(future);

    // IP literals resolve without a network.
    let first = run(refresh(&paths, Some(vec![("192.0.2.7".into(), 443), ("2001:db8::7".into(), 443)])));
    assert_eq!(first, "192.0.2.7,2001:db8::7");
    assert_eq!(std::fs::read_to_string(paths.allow_file()).unwrap(), first);

    // A server that cannot be resolved keeps what was known, and a later refresh needs no targets.
    let again = run(refresh(&paths, None));
    assert_eq!(again, first);
    let _ = std::fs::remove_dir_all(dir);
}
