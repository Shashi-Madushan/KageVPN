#!/usr/bin/env python3
"""Regenerates glyphs.bin, the alpha atlas used by src/speed_icon.rs.

Each glyph is a GLYPH_W x GLYPH_H block of 8-bit alpha values, stored in ORDER.
Keep these constants in sync with speed_icon.rs. Requires Pillow and macOS's San Francisco system font.
"""
from PIL import Image, ImageDraw, ImageFont

ORDER = "0123456789.BKMG↑↓"
GLYPH_W, GLYPH_H = 12, 17
CAP = 14           # ink height of digits/letters, in pixels
BASELINE = 15      # row just below the baseline
SCALE = 8          # supersampling for smooth edges
FONT = "/System/Library/Fonts/SFNS.ttf"  # San Francisco, Semibold
WEIGHT = b"Semibold"
ARROW_W, ARROW_H, SHAFT_W, HEAD_H = 9, 13, 3.0, 5.5


def font_for_cap(cap_px: int) -> ImageFont.FreeTypeFont:
    size = 10
    while True:
        f = ImageFont.truetype(FONT, size * SCALE)
        f.set_variation_by_name(WEIGHT)
        box = f.getbbox("0")
        if (box[3] - box[1]) / SCALE >= cap_px:
            return f
        size += 0.25


def arrow(draw, up: bool):
    """Crisp arrow drawn as a polygon, centred on the cap height."""
    cx = GLYPH_W * SCALE / 2
    top = (BASELINE - CAP / 2 - ARROW_H / 2) * SCALE
    bottom = top + ARROW_H * SCALE
    half, shaft, head = ARROW_W * SCALE / 2, SHAFT_W * SCALE / 2, HEAD_H * SCALE
    if up:
        pts = [(cx, top), (cx + half, top + head), (cx + shaft, top + head), (cx + shaft, bottom),
               (cx - shaft, bottom), (cx - shaft, top + head), (cx - half, top + head)]
    else:
        pts = [(cx, bottom), (cx + half, bottom - head), (cx + shaft, bottom - head), (cx + shaft, top),
               (cx - shaft, top), (cx - shaft, bottom - head), (cx - half, bottom - head)]
    draw.polygon(pts, fill=255)


def render(ch: str, font) -> bytes:
    canvas = Image.new("L", (GLYPH_W * SCALE, GLYPH_H * SCALE), 0)
    draw = ImageDraw.Draw(canvas)
    if ch in "↑↓":
        arrow(draw, ch == "↑")
    else:
        # Draw on a wide canvas, then squeeze horizontally if it would touch its neighbours.
        wide = Image.new("L", (GLYPH_W * SCALE * 2, GLYPH_H * SCALE), 0)
        ImageDraw.Draw(wide).text((GLYPH_W * SCALE * 0.5, BASELINE * SCALE), ch, font=font, fill=255, anchor="ls")
        ink = wide.getbbox()
        glyph = wide.crop(ink)
        limit = int((GLYPH_W - 1.5) * SCALE)
        if glyph.width > limit:
            glyph = glyph.resize((limit, glyph.height), Image.LANCZOS)
        canvas.paste(glyph, ((GLYPH_W * SCALE - glyph.width) // 2, ink[1]))
    small = canvas.resize((GLYPH_W, GLYPH_H), Image.LANCZOS)
    return bytes(small.tobytes())


font = font_for_cap(CAP)
atlas = b"".join(render(c, font) for c in ORDER)
assert len(atlas) == len(ORDER) * GLYPH_W * GLYPH_H
open("glyphs.bin", "wb").write(atlas)
print("wrote glyphs.bin", len(atlas))
