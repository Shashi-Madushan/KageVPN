use super::*;

#[test]
fn errors_that_will_not_fix_themselves_are_not_retried() {
    // Retrying a cancelled password prompt would bring the dialog back every few seconds.
    for error in [
        AppError::PermissionDenied,
        AppError::SingBoxMissing,
        AppError::ConfigCheck("bad".into()),
        AppError::Invalid("add a profile first".into()),
        AppError::ProfileNotFound,
        AppError::Cancelled,
    ] {
        assert!(!worth_retrying(&error), "{error}");
    }
}

#[test]
fn network_and_startup_trouble_is_worth_another_try() {
    assert!(worth_retrying(&AppError::ServerUnreachable("down".into())));
    assert!(worth_retrying(&AppError::Launch("sing-box did not start in time".into())));
    assert!(worth_retrying(&AppError::Io(std::io::Error::other("busy"))));
}

#[test]
fn a_reconnect_that_gave_up_resumes_only_when_nothing_else_is_going_on() {
    assert!(should_resume(true, Conn::Disconnected, true, false));
    assert!(!should_resume(false, Conn::Disconnected, true, false), "it never gave up");
    assert!(!should_resume(true, Conn::Connected, true, false), "already back");
    assert!(!should_resume(true, Conn::Connecting, true, false), "a connect is in progress");
    assert!(!should_resume(true, Conn::Disconnected, false, false), "the user turned auto-reconnect off");
    assert!(!should_resume(true, Conn::Disconnected, true, true), "a loop is already running");
}

#[test]
fn a_block_is_lifted_once_the_kill_switch_is_off() {
    assert_eq!(unwanted_block(true, false, Conn::Disconnected), Some(Lift::Stop), "held between runs");
    assert_eq!(unwanted_block(true, false, Conn::Connected), Some(Lift::Ready), "a run is up");
    assert_eq!(unwanted_block(true, false, Conn::Connecting), Some(Lift::Ready), "a run is starting");
}

#[test]
fn a_wanted_block_or_no_block_is_left_alone() {
    for state in [Conn::Disconnected, Conn::Connecting, Conn::Connected] {
        assert_eq!(unwanted_block(true, true, state), None, "the kill switch is on");
        assert_eq!(unwanted_block(false, false, state), None, "nothing is blocked");
    }
}

#[test]
fn turning_the_kill_switch_on_mid_session_waits_for_the_next_tun_connect() {
    assert!(kill_switch_waits_for_reconnect(false, true, Conn::Connected, Mode::Tun));
    assert!(kill_switch_waits_for_reconnect(false, true, Conn::Connecting, Mode::Tun));
    assert!(!kill_switch_waits_for_reconnect(false, true, Conn::Disconnected, Mode::Tun), "the next connect has it anyway");
    assert!(!kill_switch_waits_for_reconnect(false, true, Conn::Connected, Mode::Proxy), "it never applies to Proxy Only");
    assert!(!kill_switch_waits_for_reconnect(true, true, Conn::Connected, Mode::Tun), "it was already on");
    assert!(!kill_switch_waits_for_reconnect(true, false, Conn::Connected, Mode::Tun), "turned off");
}
