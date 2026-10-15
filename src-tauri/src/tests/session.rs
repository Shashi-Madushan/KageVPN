use super::*;

#[test]
fn disconnecting_while_still_connecting_cancels_instead_of_doing_nothing() {
    // It used to return at once because there is no process to stop yet.
    assert_eq!(disconnect_plan(State::Connecting, None), Plan::Cancel);
    assert_eq!(disconnect_plan(State::Connecting, Some(42)), Plan::Cancel, "even once the process exists but the connect is not finished");
}

#[test]
fn a_running_connection_is_stopped_and_an_idle_one_is_left_alone() {
    assert_eq!(disconnect_plan(State::Connected, Some(42)), Plan::Stop(42));
    assert_eq!(disconnect_plan(State::Connected, None), Plan::Nothing);
    assert_eq!(disconnect_plan(State::Disconnected, None), Plan::Nothing);
}

#[test]
fn the_kill_switch_block_is_held_only_when_a_reconnect_is_coming() {
    use Ending::*;
    // A user stop always releases, and so does anything without the kill switch or outside TUN mode.
    assert!(!holds_block(UserStop, true, Mode::Tun, true));
    assert!(!holds_block(Restart, false, Mode::Tun, true));
    assert!(!holds_block(Restart, true, Mode::Proxy, true));
    // A restart keeps the block; a failed attempt keeps it only inside a reconnect loop.
    assert!(holds_block(Restart, true, Mode::Tun, false));
    assert!(holds_block(Failed, true, Mode::Tun, true));
    assert!(!holds_block(Failed, true, Mode::Tun, false), "a failed first connect must not cut the user off");
}
