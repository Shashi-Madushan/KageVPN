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
