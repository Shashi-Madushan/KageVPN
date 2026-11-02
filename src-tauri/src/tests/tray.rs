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

#[test]
fn the_menu_button_follows_the_connection_and_is_off_while_it_is_being_stopped() {
    let mut status = Status::disconnected(Default::default());
    assert_eq!(toggle_item(&status), ("Connect", true));
    status.state = State::Connecting;
    assert_eq!(toggle_item(&status), ("Cancel", true));
    status.state = State::Connected;
    assert_eq!(toggle_item(&status), ("Disconnect", true));
    status.disconnecting = true;
    assert_eq!(toggle_item(&status), ("Disconnecting…", false), "still Connected underneath, but nothing to press");
}

#[test]
fn the_status_line_says_disconnecting_rather_than_protected() {
    let mut status = Status::disconnected(Default::default());
    status.state = State::Connected;
    status.health = Health::Healthy;
    assert_eq!(status_text(&status), "Protected");
    status.disconnecting = true;
    assert_eq!(status_text(&status), "Disconnecting…");
}

#[test]
fn template_icons_are_repainted_keeping_their_shape() {
    let rgba = [0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 128];
    assert_eq!(tint(&rgba, [255, 255, 255]), [255, 255, 255, 255, 255, 255, 255, 0, 255, 255, 255, 128]);
}

#[test]
fn the_popover_hangs_below_a_menu_bar_icon() {
    // macOS: a 1440x900 screen, icon at the top right.
    let (x, y) = popover_origin((1300.0, 0.0, 24.0, 24.0), (272.0, 450.0), Some((0.0, 0.0, 1440.0, 900.0)), 1.0);
    assert_eq!(y, 24.0 + POPOVER_GAP);
    assert_eq!(x, 1440.0 - 272.0 - SCREEN_MARGIN, "kept inside the screen");
}

#[test]
fn the_popover_sits_above_a_taskbar_icon() {
    // Windows: a 1920x1080 screen at 150%, icon in the taskbar at the bottom right.
    let (x, y) = popover_origin((1700.0, 1032.0, 48.0, 48.0), (408.0, 675.0), Some((0.0, 0.0, 1920.0, 1080.0)), 1.5);
    assert_eq!(y, 1032.0 - 675.0 - POPOVER_GAP * 1.5);
    assert!(x + 408.0 <= 1920.0 - SCREEN_MARGIN * 1.5);
}
