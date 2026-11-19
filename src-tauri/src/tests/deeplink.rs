use super::*;

fn action(link: &str) -> Option<Action> {
    action_of(&url::Url::parse(link).unwrap())
}

#[test]
fn only_the_four_known_links_do_anything() {
    assert_eq!(action("kagevpn://connect"), Some(Action::Connect));
    assert_eq!(action("kagevpn://disconnect"), Some(Action::Disconnect));
    assert_eq!(action("kagevpn://toggle"), Some(Action::Toggle));
    assert_eq!(action("kagevpn://import?url=x"), Some(Action::Import));
    assert_eq!(action("kagevpn://format"), None);
    assert_eq!(action("kagevpn://"), None);
    assert_eq!(action("https://connect"), None, "another scheme is not ours");
}

#[test]
fn turning_the_vpn_off_always_asks_and_turning_it_on_never_does() {
    for state in [State::Connected, State::Connecting] {
        assert!(needs_confirmation(Action::Disconnect, state));
        assert!(needs_confirmation(Action::Toggle, state), "a toggle of a live connection is a disconnect");
    }
    assert!(!needs_confirmation(Action::Toggle, State::Disconnected), "toggling an idle VPN connects");
    assert!(!needs_confirmation(Action::Disconnect, State::Disconnected), "nothing to turn off");
    assert!(!needs_confirmation(Action::Connect, State::Connected));
    assert!(!needs_confirmation(Action::Import, State::Connected));
}

#[test]
fn a_link_fired_in_a_loop_is_answered_once() {
    let start = Instant::now();
    assert!(!cooling(None, start));
    assert!(cooling(Some(start), start + Duration::from_secs(1)));
    assert!(!cooling(Some(start), start + COOL_DOWN));
}
