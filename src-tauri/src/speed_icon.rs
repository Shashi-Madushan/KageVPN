use tauri::image::Image;

const ORDER: &str = "0123456789.BKMG↑↓";
const ATLAS: &[u8] = include_bytes!("../icons/tray/glyphs.bin");
const GLYPH_W: usize = 10;
const GLYPH_H: usize = 17;
const NUMBER_CELLS: usize = 5;
const TEXT_CELLS: usize = NUMBER_CELLS + 1;
const GAP: usize = 6;

/// Draws the base icon with two stacked rows of text to its right:
/// an up arrow with the upload rate over a down arrow with the download rate.
pub fn compose(base: &Image<'_>, up: &str, down: &str) -> Image<'static> {
    let (base_w, height) = (base.width() as usize, base.height() as usize);
    let width = base_w + GAP + TEXT_CELLS * GLYPH_W;
    let mut rgba = vec![0u8; width * height * 4];

    for y in 0..height {
        let src = y * base_w * 4;
        let dst = y * width * 4;
        rgba[dst..dst + base_w * 4].copy_from_slice(&base.rgba()[src..src + base_w * 4]);
    }

    let top = height.saturating_sub(GLYPH_H * 2) / 2;
    let origin_x = base_w + GAP;
    draw_row(&mut rgba, width, origin_x, top, '↑', up);
    draw_row(&mut rgba, width, origin_x, top + GLYPH_H, '↓', down);
    Image::new_owned(rgba, width as u32, height as u32)
}

fn draw_row(rgba: &mut [u8], width: usize, x: usize, y: usize, arrow: char, number: &str) {
    draw_glyph(rgba, width, x, y, arrow);
    let number: Vec<char> = number.chars().collect();
    let start = NUMBER_CELLS.saturating_sub(number.len());
    for (i, c) in number.iter().take(NUMBER_CELLS).enumerate() {
        draw_glyph(rgba, width, x + (1 + start + i) * GLYPH_W, y, *c);
    }
}

fn draw_glyph(rgba: &mut [u8], width: usize, x: usize, y: usize, c: char) {
    let Some(index) = ORDER.chars().position(|known| known == c) else { return };
    let glyph = &ATLAS[index * GLYPH_W * GLYPH_H..(index + 1) * GLYPH_W * GLYPH_H];
    for row in 0..GLYPH_H {
        for col in 0..GLYPH_W {
            let alpha = glyph[row * GLYPH_W + col];
            let pixel = ((y + row) * width + x + col) * 4;
            rgba[pixel + 3] = rgba[pixel + 3].max(alpha);
        }
    }
}

#[cfg(test)]
mod tests {
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
}
