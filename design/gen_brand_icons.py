#!/usr/bin/env python3
"""Draws the Kage "moon ninja" mark and writes the master app icon plus the menu bar glyphs.

  design/app-icon.png                       1024px master (feed it to `tauri icon`, see design/README.md)
  src-tauri/icons/tray/{idle,connecting,connected}.png   36px template glyphs (alpha only)

Requires Pillow and numpy. Re-run after changing a shape, then regenerate the icon set.
"""
from pathlib import Path

import numpy as np
from PIL import Image, ImageChops, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parent.parent

# ── Palette ─────────────────────────────────────────────────────────────────
TILE_TOP, TILE_BOTTOM = (0x1d, 0x18, 0x3d), (0x0a, 0x09, 0x15)
MOON = (0xe9, 0xe3, 0xff)
SHADOW = (0x0b, 0x0a, 0x16)
VIOLET = (0x8b, 0x6d, 0xff)
GLOW = (0x7c, 0x5c, 0xff)


def quad(p0, p1, p2, steps=24):
    """Points along a quadratic Bezier."""
    return [
        ((1 - t) ** 2 * p0[0] + 2 * (1 - t) * t * p1[0] + t * t * p2[0],
         (1 - t) ** 2 * p0[1] + 2 * (1 - t) * t * p1[1] + t * t * p2[1])
        for t in (i / steps for i in range(steps + 1))
    ]


def body(left, shoulder_l, apex, shoulder_r, right):
    """A hooded torso: straight sides, curved shoulders."""
    return [left, shoulder_l, *quad(shoulder_l, apex, shoulder_r), right]


def mask(size, ss, draw_fn):
    img = Image.new("L", (size * ss, size * ss), 0)
    draw_fn(ImageDraw.Draw(img), ss)
    return img


def finish(img, size):
    return img.resize((size, size), Image.LANCZOS)


# ── App icon ────────────────────────────────────────────────────────────────

def app_icon(size=1024, ss=4):
    S = size * ss
    tile = size * 824 / 1024          # Apple icon grid: 824px tile on a 1024px canvas
    off = (size - tile) / 2
    k = tile / 256                    # the 256-unit design space is the tile

    def p(x, y):
        return ((off + x * k) * ss, (off + y * k) * ss)

    def circle(d, cx, cy, r, fill):
        (x, y), rr = p(cx, cy), r * k * ss
        d.ellipse([x - rr, y - rr, x + rr, y + rr], fill=fill)

    # Tile with a soft vertical gradient.
    tile_mask = Image.new("L", (S, S), 0)
    ImageDraw.Draw(tile_mask).rounded_rectangle(
        [off * ss, off * ss, (off + tile) * ss - 1, (off + tile) * ss - 1], radius=tile * 0.2237 * ss, fill=255)
    t = np.linspace(0, 1, S, dtype=np.float32)[:, None, None]
    top, bottom = np.array(TILE_TOP, np.float32), np.array(TILE_BOTTOM, np.float32)
    grad = (top * (1 - t) + bottom * t).astype(np.uint8)
    base = Image.fromarray(np.broadcast_to(grad, (S, S, 3)).copy(), "RGB").convert("RGBA")
    base.putalpha(tile_mask)

    # Violet halo behind the moon, kept inside the tile.
    halo = Image.new("L", (S, S), 0)
    circle(ImageDraw.Draw(halo), 128, 116, 104, 255)
    halo = halo.resize((size, size), Image.LANCZOS).filter(ImageFilter.GaussianBlur(size * 0.045))
    img = base.resize((size, size), Image.LANCZOS)
    halo_alpha = ImageChops.multiply(halo.point(lambda v: int(v * 0.28)), img.getchannel("A"))
    glow = Image.new("RGBA", (size, size), GLOW + (0,))
    glow.putalpha(halo_alpha)
    img = Image.alpha_composite(img, glow)

    # Moon, ninja and ground line, drawn at supersampled size then reduced.
    art = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(art)
    circle(d, 128, 116, 72, MOON + (255,))
    circle(d, 128, 108, 18, SHADOW + (255,))
    d.polygon([p(*pt) for pt in body((86, 204), (96, 144), (128, 122), (160, 144), (170, 204))], fill=SHADOW + (255,))
    d.rectangle([*p(117, 104), *p(139, 109)], fill=MOON + (255,))
    d.rounded_rectangle([*p(64, 204), *p(192, 209)], radius=2.5 * k * ss, fill=VIOLET + (255,))
    return Image.alpha_composite(img, art.resize((size, size), Image.LANCZOS))


# ── Menu bar glyphs ─────────────────────────────────────────────────────────
# 36px canvas (18pt @2x). Black with alpha only: macOS recolours template images.

def tray_glyphs(size=36, ss=16):
    cx, cy, R, ring = 18, 18, 15.0, 2.4
    floor = 26.0

    def sc(v):
        return v * ss

    def disc(d, r, fill=255):
        d.ellipse([sc(cx - r), sc(cy - r), sc(cx + r), sc(cy + r)], fill=fill)

    def ninja(d, fill=255):
        d.ellipse([sc(18 - 3.3), sc(11.6 - 3.3), sc(18 + 3.3), sc(11.6 + 3.3)], fill=fill)
        d.polygon([(sc(x), sc(y)) for x, y in body((12.2, floor), (14.2, 17.8), (18, 15.6), (21.8, 17.8), (23.8, floor))], fill=fill)

    def slit(d, fill):
        d.rectangle([sc(15.9), sc(10.9), sc(20.1), sc(12.2)], fill=fill)

    def idle(d, _):
        disc(d, R)
        disc(d, R - ring, 0)
        ninja(d)
        slit(d, 0)

    def connected(d, _):
        disc(d, R)
        ninja(d, 0)
        slit(d, 255)

    idle_m = finish(mask(size, ss, idle), size)
    connected_m = finish(mask(size, ss, connected), size)
    connecting_m = idle_m.point(lambda v: int(v * 0.5))

    out = {}
    for name, m in (("idle", idle_m), ("connecting", connecting_m), ("connected", connected_m)):
        img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        img.putalpha(m)
        out[name] = img
    return out


if __name__ == "__main__":
    app_icon().save(ROOT / "design" / "app-icon.png")
    for name, img in tray_glyphs().items():
        img.save(ROOT / "src-tauri" / "icons" / "tray" / f"{name}.png")
    print("wrote design/app-icon.png and src-tauri/icons/tray/{idle,connecting,connected}.png")
