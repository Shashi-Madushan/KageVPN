use super::*;

#[test]
fn formats_rates_compactly() {
    assert_eq!(format_rate(0), "0B");
    assert_eq!(format_rate(900), "900B");
    assert_eq!(format_rate(2048), "2K");
    assert_eq!(format_rate(1_572_864), "1.5M");
}

#[test]
fn first_sample_is_shown_as_is() {
    assert_eq!(smooth_step(None, 4000, 9000), (4000.0, 9000.0));
}

#[test]
fn rates_ease_toward_new_samples() {
    let (up, down) = smooth_step(Some((1000.0, 1000.0)), 3000, 0);
    assert!(up > 1000.0 && up < 3000.0);
    assert!(down < 1000.0 && down > 0.0);
}

#[test]
fn small_tails_snap_to_zero() {
    let (up, _) = smooth_step(Some((300.0, 0.0)), 0, 0);
    assert_eq!(up, 0.0);
}

#[test]
fn rate_text_always_fits_the_readout() {
    for value in [0, 900, 1023, 1024, 999 * 1024, 5 * 1024 * 1024, 99 * 1024 * 1024, 100 * 1024 * 1024, 125 * 1024 * 1024, 999 * 1024 * 1024] {
        assert!(format_rate(value).chars().count() <= 5, "{value}");
    }
}

#[test]
fn fast_links_keep_their_unit() {
    // "120.0M" is six characters and used to be cut to "120.0", dropping the unit.
    assert_eq!(format_rate(120 * 1024 * 1024), "120M");
    assert_eq!(format_rate(12 * 1024 * 1024), "12M");
    assert_eq!(format_rate(9_437_184), "9.0M");
}
