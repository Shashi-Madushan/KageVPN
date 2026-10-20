use super::*;

#[test]
fn disconnecting_while_still_connecting_cancels_instead_of_doing_nothing() {
    // It used to return at once because there is no process to stop yet.
    assert_eq!(disconnect_plan(State::Connecting, None, false), Plan::Cancel);
    assert_eq!(disconnect_plan(State::Connecting, Some(42), false), Plan::Cancel, "even once the process exists but the connect is not finished");
}

#[test]
fn a_running_connection_is_stopped_and_an_idle_one_is_left_alone() {
    assert_eq!(disconnect_plan(State::Connected, Some(42), false), Plan::Stop(42));
    assert_eq!(disconnect_plan(State::Connected, None, false), Plan::Nothing);
    assert_eq!(disconnect_plan(State::Disconnected, None, false), Plan::Nothing);
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

#[test]
fn a_second_disconnect_waits_for_the_first_instead_of_starting_another() {
    // The state stays Connected until sing-box has exited, so "already stopping" has its own flag.
    assert_eq!(disconnect_plan(State::Connected, Some(42), true), Plan::WaitForStop);
    assert_eq!(disconnect_plan(State::Connecting, Some(42), true), Plan::Cancel, "cancelling is unaffected");
    assert_eq!(disconnect_plan(State::Disconnected, None, true), Plan::Nothing);
}

#[test]
fn a_new_status_is_not_disconnecting_and_an_old_one_without_the_field_still_loads() {
    assert!(!Status::disconnected(Mode::Tun).disconnecting);
    let mut json = serde_json::to_value(Status::disconnected(Mode::Tun)).unwrap();
    assert_eq!(json["disconnecting"], serde_json::Value::Bool(false));
    json.as_object_mut().unwrap().remove("disconnecting");
    let status: Status = serde_json::from_value(json).unwrap();
    assert!(!status.disconnecting);
}

#[tokio::test]
async fn slow_work_is_abandoned_as_soon_as_the_connection_is_cancelled() {
    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = cancel.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        flag.store(true, Ordering::SeqCst);
    });
    let started = Instant::now();
    let result = until_cancelled(&cancel, sleep(Duration::from_secs(30))).await;
    assert!(matches!(result, Err(AppError::Cancelled)));
    assert!(started.elapsed() < Duration::from_secs(3));
}

#[tokio::test]
async fn work_that_finishes_before_a_cancel_gives_its_value() {
    let cancel = std::sync::atomic::AtomicBool::new(false);
    assert_eq!(until_cancelled(&cancel, async { 7 }).await.unwrap(), 7);
}

#[tokio::test]
async fn a_connection_cancelled_before_the_work_starts_never_runs_it() {
    let cancel = std::sync::atomic::AtomicBool::new(true);
    let ran = std::sync::atomic::AtomicBool::new(false);
    let result = until_cancelled(&cancel, async {
        sleep(Duration::from_secs(5)).await;
        ran.store(true, Ordering::SeqCst);
    })
    .await;
    assert!(matches!(result, Err(AppError::Cancelled)));
    assert!(!ran.load(Ordering::SeqCst));
}
