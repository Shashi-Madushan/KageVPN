use super::*;

#[test]
fn accumulates_totals_and_caps_history() {
    let mut state = TrafficState::default();
    for _ in 0..HISTORY_LEN + 5 {
        state.record(Sample { up: 1, down: 2 });
    }
    let (current, history) = state.snapshot();
    assert_eq!(current.total_up, (HISTORY_LEN + 5) as u64);
    assert_eq!(current.total_down, 2 * (HISTORY_LEN + 5) as u64);
    assert_eq!(history.len(), HISTORY_LEN);
}
