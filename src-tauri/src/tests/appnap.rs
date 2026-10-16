use super::*;

// One test: the assertion is process-wide state, so separate tests would race.
#[test]
fn the_assertion_is_held_once_and_released_safely() {
    release();
    assert!(!is_held(), "releasing when nothing is held is harmless");
    hold();
    hold();
    assert!(is_held());
    release();
    assert!(!is_held(), "one release is enough however many holds");
    release();
    assert!(!is_held());
}
