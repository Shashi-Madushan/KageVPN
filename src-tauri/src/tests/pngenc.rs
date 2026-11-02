use super::*;

#[test]
fn the_checksum_matches_the_standard_one() {
    let mut crc = Crc::new();
    crc.update(b"123456789");
    assert_eq!(crc.finish(), 0xcbf4_3926);
}

#[test]
fn an_icon_round_trips_through_a_png_reader() {
    // A 2x2 image: red, green, blue and a half-transparent white.
    let pixels = [255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 128];
    let png = encode_rgba(2, 2, &pixels).unwrap();
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    // Tauri's own image loader (the `png` crate) reads it back unchanged.
    let image = tauri::image::Image::from_bytes(&png).unwrap();
    assert_eq!((image.width(), image.height()), (2, 2));
    assert_eq!(image.rgba(), &pixels);
}

#[test]
fn a_size_that_does_not_match_the_pixels_is_refused() {
    assert!(encode_rgba(2, 2, &[0; 12]).is_none());
    assert!(encode_rgba(0, 0, &[]).is_none());
}
