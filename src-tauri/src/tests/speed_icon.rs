use super::*;

#[test]
fn atlas_matches_glyph_order() {
    assert_eq!(ATLAS.len(), ORDER.chars().count() * GLYPH_W * GLYPH_H);
}

#[test]
fn composed_image_is_wide_enough_for_text() {
    let base = Image::new_owned(vec![0; 36 * 36 * 4], 36, 36);
    let image = compose(&base, "1.5M", "900B");
    assert_eq!(image.width() as usize, 36 + GAP + TEXT_CELLS * GLYPH_W);
    assert_eq!(image.height(), 36);
    assert!(image.rgba().iter().skip(3).step_by(4).any(|&a| a > 0));
}
