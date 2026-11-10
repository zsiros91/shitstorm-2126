#!/usr/bin/env python3
"""Native-resolution pixel-art generator for SHITSTORM 2126.

Everything is drawn procedurally at 1:1 pixel resolution: hard edges,
limited palette, ordered dithering, no resampling, no vector smoothing.
Run with:  python3 tools/gen_assets.py
Requires Pillow (pip install pillow).
"""

from __future__ import annotations

import math
import random
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "sprites"
OUT.mkdir(parents=True, exist_ok=True)

# ---------------------------------------------------------------------------
# palette -- night desert: bleached sand, rust, faded paint, deep ink outlines
# ---------------------------------------------------------------------------
INK = (18, 16, 20)
WHITE = (255, 248, 236)

SAND_1 = (180, 160, 118)
SAND_2 = (162, 143, 103)
SAND_3 = (142, 124, 88)
SAND_4 = (118, 101, 71)

DIRT_1 = (130, 102, 74)
DIRT_2 = (112, 87, 62)
DIRT_3 = (94, 73, 52)

GRAVEL_1 = (134, 132, 126)
GRAVEL_2 = (114, 112, 106)
GRAVEL_3 = (92, 91, 86)

DRYGRASS_1 = (140, 140, 86)
DRYGRASS_2 = (116, 118, 70)
DRYGRASS_3 = (96, 98, 58)

ASPHALT_1 = (76, 77, 82)
ASPHALT_2 = (66, 67, 72)
ASPHALT_3 = (86, 87, 91)
ASPHALT_4 = (50, 51, 55)

CONCRETE_1 = (126, 125, 119)
CONCRETE_2 = (110, 109, 104)
CONCRETE_3 = (88, 87, 83)

BRICK_1 = (130, 88, 68)
BRICK_2 = (110, 74, 58)
BRICK_3 = (92, 62, 50)
MORTAR = (136, 130, 120)

RUST_1 = (94, 60, 40)
RUST_2 = (128, 82, 48)
RUST_3 = (160, 108, 60)

OLIVE_1 = (72, 78, 56)
OLIVE_2 = (90, 98, 68)
OLIVE_3 = (56, 62, 46)

LEATHER_1 = (98, 72, 50)
LEATHER_2 = (78, 57, 40)
LEATHER_3 = (120, 90, 62)

SKIN_1 = (214, 172, 132)
SKIN_2 = (186, 146, 110)
SKIN_3 = (152, 116, 86)

MUTANT_1 = (152, 160, 114)
MUTANT_2 = (122, 132, 92)
MUTANT_3 = (96, 106, 72)

HAIR_1 = (58, 46, 36)
HAIR_2 = (94, 72, 44)

YELLOW = (228, 192, 76)
ORANGE = (212, 130, 52)
RED = (178, 60, 44)
RED_D = (112, 36, 30)
TEAL = (88, 152, 156)
TEAL_L = (140, 200, 200)
BLUE_R = (74, 94, 124)
GREEN_R = (94, 124, 76)
WOOD_1 = (146, 114, 74)
WOOD_2 = (120, 92, 58)
WOOD_3 = (94, 72, 44)


def clamp(v, lo=0, hi=255):
    return lo if v < lo else hi if v > hi else v


def mix(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(min(len(a), len(b))))


def rgba(c, a=255):
    return (c[0], c[1], c[2], a)


def new(w, h):
    return Image.new("RGBA", (w, h), (0, 0, 0, 0))


def save(im, name):
    im.save(OUT / name)
    return name


def noise(im, amount, seed):
    """Jitter RGB of opaque pixels for a hand-dithered texture."""
    rng = random.Random(seed)
    px = im.load()
    w, h = im.size
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if a == 0:
                continue
            n = rng.randint(-amount, amount)
            px[x, y] = (clamp(r + n), clamp(g + n), clamp(b + n), a)


def patch(im, x0, y0, pw, ph, col, density, seed, mode="checker"):
    """Blend `col` over a rect of opaque pixels.

    mode="checker": ordered 50% dither (density = blend strength)
    mode="rand":    stochastic scatter (density = probability)
    mode="solid":   full blend (density = blend strength)
    """
    rng = random.Random(seed)
    px = im.load()
    w, h = im.size
    for y in range(max(0, y0), min(h, y0 + ph)):
        for x in range(max(0, x0), min(w, x0 + pw)):
            if px[x, y][3] == 0:
                continue
            if mode == "checker":
                on = ((x + y) & 1) == 0
            elif mode == "rand":
                on = rng.random() < density
            else:
                on = True
            if not on:
                continue
            r, g, b, a = px[x, y]
            t = density if mode != "checker" else min(1.0, density * 1.7)
            px[x, y] = (
                clamp(int(r + (col[0] - r) * t)),
                clamp(int(g + (col[1] - g) * t)),
                clamp(int(b + (col[2] - b) * t)),
                a,
            )


def speck(im, count, seed, colors, smin=1, smax=2, strength=(60, 150)):
    """Scatter 1-2px flecks (stones, grit, paint chips) over opaque pixels."""
    rng = random.Random(seed)
    px = im.load()
    w, h = im.size
    for _ in range(count):
        x = rng.randrange(w)
        y = rng.randrange(h)
        if px[x, y][3] == 0:
            continue
        col = rng.choice(colors)
        s = rng.randint(smin, smax)
        t = rng.randint(*strength) / 255.0
        for dy in range(s):
            for dx in range(s):
                xx, yy = x + dx, y + dy
                if 0 <= xx < w and 0 <= yy < h and px[xx, yy][3] > 0:
                    r, g, b, a = px[xx, yy]
                    px[xx, yy] = (
                        clamp(int(r + (col[0] - r) * t)),
                        clamp(int(g + (col[1] - g) * t)),
                        clamp(int(b + (col[2] - b) * t)),
                        a,
                    )


def crack(im, count, seed, color, reach=7, length=(3, 6)):
    """Hairline cracks for asphalt / concrete / brick."""
    rng = random.Random(seed)
    d = ImageDraw.Draw(im)
    w, h = im.size
    for _ in range(count):
        x = rng.randrange(w)
        y = rng.randrange(h)
        pts = [(x, y)]
        for _ in range(rng.randint(*length)):
            x = clamp(x + rng.randint(-reach, reach), 0, w - 1)
            y = clamp(y + rng.randint(-reach, reach), 0, h - 1)
            pts.append((x, y))
        d.line(pts, fill=rgba(color, 255), width=1)


def outline(im, color=(16, 14, 18, 255)):
    """Add a 1px dark contour around every opaque blob (needs 1px margin)."""
    px = im.load()
    w, h = im.size
    add = []
    for y in range(h):
        for x in range(w):
            if px[x, y][3] != 0:
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < w and 0 <= ny < h and px[nx, ny][3] > 0:
                    add.append((x, y))
                    break
    for x, y in add:
        px[x, y] = color


def bevel(d, box, fill, light=None, dark=None, r=2):
    """Hard-edged bevelled rectangle: light top-left, dark bottom-right."""
    x0, y0, x1, y1 = box
    light = light or mix(fill, WHITE, 0.32)
    dark = dark or mix(fill, INK, 0.45)
    d.rectangle(box, fill=rgba(fill))
    d.line([(x0 + r, y0), (x1 - r, y0)], fill=rgba(light))
    d.line([(x0, y0 + r), (x0, y1 - r)], fill=rgba(light))
    d.line([(x0 + r, y1 - 1), (x1 - r, y1 - 1)], fill=rgba(dark))
    d.line([(x1 - 1, y0 + r), (x1 - 1, y1 - r)], fill=rgba(dark))


def blob(d, cx, cy, rx, ry, color, seed, alpha=255, steps=16):
    """Irregular organic patch (stain, scorch, leaf clump)."""
    rng = random.Random(seed)
    pts = []
    for i in range(steps):
        a = i / steps * math.tau
        j = 0.7 + rng.random() * 0.5
        pts.append((cx + math.cos(a) * rx * j, cy + math.sin(a) * ry * j))
    d.polygon(pts, fill=rgba(color, alpha))


def banded_glow(name, size, color, peak, power=2.2, steps=6):
    """Radial glow with quantised alpha bands -- reads as pixel art, not blur."""
    im = new(size, size)
    px = im.load()
    c = size / 2.0
    for y in range(size):
        for x in range(size):
            dist = math.hypot(x + 0.5 - c, y + 0.5 - c) / c
            if dist > 1.0:
                continue
            t = (1.0 - dist) ** power
            band = int(t * peak) // 32 * 32
            if band <= 0:
                continue
            px[x, y] = (color[0], color[1], color[2], min(255, band))
    return save(im, name)

# ---------------------------------------------------------------------------
# terrain tiles (32x32)
# ---------------------------------------------------------------------------
def tile_sand(name, seed, base=SAND_1):
    rng = random.Random(seed)
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 31, 31), fill=rgba(base))
    # wind-swept dune bands (checker dithered, no gradients)
    for i in range(3):
        ox = rng.randrange(0, 24)
        oy = rng.randrange(0, 26)
        pw = rng.randint(8, 18)
        ph = rng.randint(4, 8)
        col = SAND_3 if i % 2 == 0 else SAND_2
        patch(im, ox, oy, pw, ph, col, 0.5, seed + i, mode="checker")
    speck(im, 70, seed + 11, [SAND_4, SAND_3, DIRT_3], 1, 2, (50, 130))
    speck(im, 12, seed + 12, [(96, 84, 60), (76, 66, 48)], 1, 1, (90, 170))
    noise(im, 5, seed)
    return save(im, name)


def tile_dirt(name, seed):
    rng = random.Random(seed)
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 31, 31), fill=rgba(DIRT_1))
    for i in range(4):
        blob(d, rng.randrange(32), rng.randrange(32), rng.randint(5, 11),
             rng.randint(4, 8), DIRT_2 if i % 2 else DIRT_3, seed + i, alpha=255)
    speck(im, 90, seed + 21, [DIRT_3, SAND_3, (86, 68, 50)], 1, 2, (60, 150))
    speck(im, 14, seed + 22, [(70, 56, 40), (58, 46, 34)], 1, 1, (90, 170))
    noise(im, 6, seed)
    return save(im, name)


def tile_gravel(name, seed):
    rng = random.Random(seed)
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 31, 31), fill=rgba(GRAVEL_2))
    for _ in range(46):
        x = rng.randrange(32)
        y = rng.randrange(32)
        r = rng.randint(1, 2)
        col = rng.choice([GRAVEL_1, (150, 148, 140), GRAVEL_3, (124, 120, 112)])
        d.rectangle((x, y, x + r, y + r), fill=rgba(col))
        d.point((x, y), fill=rgba(mix(col, WHITE, 0.5)))
    speck(im, 40, seed + 31, [GRAVEL_3, DIRT_3], 1, 1, (60, 140))
    noise(im, 6, seed)
    return save(im, name)


def tile_drygrass(name, seed):
    rng = random.Random(seed)
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 31, 31), fill=rgba(SAND_2))
    for _ in range(5):
        blob(d, rng.randrange(32), rng.randrange(32), rng.randint(6, 12),
             rng.randint(4, 9), DRYGRASS_1, seed, alpha=210)
    patch(im, 0, 0, 32, 32, DRYGRASS_2, 0.45, seed + 1, mode="checker")
    # tufts
    for _ in range(22):
        x = rng.randrange(30)
        y = rng.randrange(30)
        h = rng.randint(2, 4)
        col = rng.choice([DRYGRASS_3, DRYGRASS_2, DRYGRASS_1])
        d.line([(x, y + h), (x + rng.randint(-1, 1), y)], fill=rgba(col))
    noise(im, 6, seed)
    return save(im, name)


def tile_asphalt(name, seed):
    rng = random.Random(seed)
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 31, 31), fill=rgba(ASPHALT_1))
    # patchwork repairs
    for _ in range(2):
        x0 = rng.randrange(24)
        y0 = rng.randrange(24)
        d.rectangle((x0, y0, x0 + rng.randint(6, 14), y0 + rng.randint(5, 12)),
                    fill=rgba(ASPHALT_2 if rng.random() < 0.6 else ASPHALT_3))
    speck(im, 90, seed + 41, [(96, 98, 102), (56, 57, 62), (108, 108, 110)],
          1, 2, (50, 130))
    crack(im, 2, seed + 42, ASPHALT_4, reach=6, length=(3, 5))
    noise(im, 5, seed)
    return save(im, name)


def _road_base(seed):
    name = tile_asphalt(f"_tmp_{seed}.png", seed)
    im = Image.open(OUT / name).copy()
    (OUT / name).unlink()
    return im


def tile_road(name, seed, mark="double_h"):
    im = _road_base(seed)
    d = ImageDraw.Draw(im)
    if mark == "double_h":
        d.rectangle((0, 13, 31, 14), fill=rgba(YELLOW))
        d.rectangle((0, 17, 31, 18), fill=rgba(YELLOW))
        # worn paint
        for x in range(0, 32, 3):
            if (x * 7 + seed) % 5 == 0:
                d.rectangle((x, 13, x, 18), fill=rgba(ASPHALT_1))
    elif mark == "dash_h":
        d.rectangle((0, 14, 15, 16), fill=rgba((226, 222, 210)))
        for x in range(0, 16, 4):
            if (x + seed) % 3 == 0:
                d.rectangle((x, 14, x, 16), fill=rgba(ASPHALT_2))
    elif mark == "edge_h":
        d.rectangle((0, 3, 31, 5), fill=rgba((222, 218, 206)))
        patch(im, 0, 3, 32, 3, ASPHALT_2, 0.35, seed + 3, mode="rand")
    elif mark == "cross_h":
        # zebra stripes running across the horizontal road
        for x in range(2, 32, 7):
            d.rectangle((x, 2, x + 4, 29), fill=rgba((224, 220, 208)))
            patch(im, x, 2, 5, 28, ASPHALT_2, 0.3, seed + x, mode="rand")
    noise(im, 4, seed + 1)
    return save(im, name)


def tile_sidewalk(name, seed):
    rng = random.Random(seed)
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 31, 31), fill=rgba(CONCRETE_1))
    # slab joints
    d.line([(0, 0), (31, 0)], fill=rgba(CONCRETE_3))
    d.line([(0, 0), (0, 31)], fill=rgba(CONCRETE_3))
    d.line([(0, 1), (31, 1)], fill=rgba(mix(CONCRETE_1, WHITE, 0.35)))
    d.line([(1, 0), (1, 31)], fill=rgba(mix(CONCRETE_1, WHITE, 0.35)))
    patch(im, 3, 3, 28, 28, CONCRETE_2, 0.4, seed, mode="checker")
    speck(im, 40, seed + 51, [CONCRETE_3, (140, 138, 130)], 1, 2, (50, 130))
    crack(im, 2, seed + 52, CONCRETE_3, reach=5, length=(2, 4))
    noise(im, 5, seed)
    return save(im, name)


def tile_checker(name, seed):
    rng = random.Random(seed)
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    light = (178, 170, 152)
    dark = (96, 94, 92)
    d.rectangle((0, 0, 31, 31), fill=rgba(light))
    for y in range(0, 32, 8):
        for x in range(0, 32, 8):
            if ((x // 8) + (y // 8)) % 2 == 0:
                d.rectangle((x, y, x + 7, y + 7), fill=rgba(dark))
    # grime, scuffs, cracked tiles
    patch(im, 0, 0, 32, 32, (70, 62, 52), 0.3, seed + 1, mode="rand")
    speck(im, 30, seed + 61, [(64, 58, 50), (150, 142, 126)], 1, 2, (40, 120))
    crack(im, 2, seed + 62, (60, 56, 52), reach=6, length=(3, 5))
    noise(im, 5, seed)
    return save(im, name)


def tile_wood(name, seed):
    rng = random.Random(seed)
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 31, 31), fill=rgba(WOOD_1))
    for y in range(0, 32, 6):
        col = WOOD_1 if (y // 6) % 2 == 0 else mix(WOOD_1, WOOD_2, 0.7)
        d.rectangle((0, y, 31, y + 5), fill=rgba(col))
        d.line([(0, y + 5), (31, y + 5)], fill=rgba(WOOD_3))
        d.line([(0, y), (31, y)], fill=rgba(mix(col, WHITE, 0.18)))
        # grain
        for _ in range(3):
            gx = rng.randrange(32)
            d.line([(gx, y + 1), (clamp(gx + rng.randint(-6, 6), 0, 31), y + 4)],
                   fill=rgba(WOOD_2))
    speck(im, 24, seed + 71, [WOOD_3, (76, 58, 36)], 1, 1, (50, 130))
    noise(im, 5, seed)
    return save(im, name)


def tile_concrete(name, seed):
    rng = random.Random(seed)
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 31, 31), fill=rgba(CONCRETE_2))
    d.line([(0, 16), (31, 16)], fill=rgba(CONCRETE_3))
    d.line([(16, 0), (16, 31)], fill=rgba(CONCRETE_3))
    patch(im, 0, 0, 32, 16, CONCRETE_1, 0.35, seed, mode="checker")
    speck(im, 60, seed + 81, [CONCRETE_3, (140, 138, 130), (76, 74, 70)],
          1, 2, (50, 140))
    crack(im, 3, seed + 82, CONCRETE_3, reach=7, length=(3, 6))
    noise(im, 6, seed)
    return save(im, name)


# ---------------------------------------------------------------------------
# walls (32x32) -- top-down block with a lit north cap and shadowed south cap
# ---------------------------------------------------------------------------
def wall_tile(name, kind, seed, broken=False):
    rng = random.Random(seed)
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    if kind == "brick":
        base, light, dark = BRICK_2, BRICK_1, BRICK_3
    elif kind == "concrete":
        base, light, dark = CONCRETE_2, CONCRETE_1, CONCRETE_3
    else:
        base, light, dark = RUST_2, RUST_3, RUST_1
    d.rectangle((0, 0, 31, 31), fill=rgba(base))
    d.rectangle((0, 0, 31, 5), fill=rgba(light))
    d.line([(0, 5), (31, 5)], fill=rgba(mix(light, WHITE, 0.45)))
    d.rectangle((0, 26, 31, 31), fill=rgba(dark))
    d.line([(0, 26), (31, 26)], fill=rgba(mix(dark, INK, 0.5)))

    if kind == "brick":
        for row in range(6):
            y = 7 + row * 4
            d.line([(0, y), (31, y)], fill=rgba(MORTAR))
            off = 0 if row % 2 == 0 else 8
            for x in range(off, 34, 16):
                d.line([(x, y), (x, y + 3)], fill=rgba(MORTAR))
        patch(im, 0, 6, 32, 20, BRICK_3, 0.3, seed + 1, mode="rand")
    elif kind == "concrete":
        for x in range(0, 32, 4):
            d.line([(x, 6), (x, 25)], fill=rgba(mix(base, INK, 0.25)))
            d.line([(x + 1, 6), (x + 1, 25)], fill=rgba(mix(base, WHITE, 0.12)))
        for _ in range(5):
            cx, cy = rng.randrange(32), rng.randint(8, 24)
            r = rng.randint(2, 5)
            d.ellipse((cx - r, cy - r, cx + r, cy + r),
                      fill=rgba(mix(base, INK, 0.35)))
    else:
        for x in range(0, 32, 6):
            d.line([(x, 6), (x, 25)], fill=rgba(mix(base, INK, 0.35)))
            d.line([(x + 2, 6), (x + 2, 25)], fill=rgba(mix(base, WHITE, 0.18)))
        for _ in range(7):
            cx, cy = rng.randrange(32), rng.randint(6, 26)
            r = rng.randint(1, 4)
            d.ellipse((cx - r, cy - r, cx + r, cy + r), fill=rgba(RUST_1))
    speck(im, 26, seed + 91, [mix(base, INK, 0.4), mix(base, WHITE, 0.25)],
          1, 2, (60, 140))

    if broken:
        # a collapsed chunk: erase an irregular hole, spill rubble below
        pts = [(rng.randrange(6, 26), 31)]
        cx = rng.randrange(10, 24)
        for i in range(7):
            a = math.pi + i / 6 * math.pi
            pts.append((clamp(int(cx + math.cos(a) * rng.randint(7, 13)), 0, 31),
                        clamp(int(31 + math.sin(a) * rng.randint(8, 16)), 0, 31)))
        d.polygon(pts, fill=(0, 0, 0, 0))
        for _ in range(20):
            x = rng.randrange(32)
            y = rng.randrange(20, 32)
            if im.getpixel((x, y))[3] == 0 and rng.random() < 0.7:
                r = rng.randint(1, 2)
                col = rng.choice([light, base, dark, MORTAR])
                d.rectangle((x, y, x + r, y + r), fill=rgba(col))
    noise(im, 5, seed)
    return save(im, name)


# ---------------------------------------------------------------------------
# ground decals (32x32 / 48x48, transparent)
# ---------------------------------------------------------------------------
def decal_leaf(name, seed):
    rng = random.Random(seed)
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    for _ in range(30):
        x = rng.randrange(30)
        y = rng.randrange(30)
        w = rng.randint(2, 4)
        h = rng.randint(1, 3)
        col = rng.choice([YELLOW, ORANGE, DRYGRASS_1, (196, 158, 66), DRYGRASS_2])
        d.rectangle((x, y, x + w, y + h), fill=rgba(col, rng.randint(170, 245)))
        d.point((x, y), fill=rgba(mix(col, WHITE, 0.4)))
    return save(im, name)


def decal_grass(name, seed):
    rng = random.Random(seed)
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    for _ in range(26):
        x = rng.randrange(32)
        y = rng.randrange(28)
        h = rng.randint(3, 6)
        col = rng.choice([DRYGRASS_1, DRYGRASS_2, DRYGRASS_3])
        d.line([(x, y + h), (clamp(x + rng.randint(-2, 2), 0, 31), y)],
               fill=rgba(col, 230))
    return save(im, name)


def decal_oil(name, seed):
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    blob(d, 16, 16, 13, 10, (36, 34, 36), seed, alpha=215)
    blob(d, 14, 18, 7, 5, (24, 22, 24), seed + 1, alpha=230)
    d.ellipse((9, 10, 21, 17), fill=rgba((58, 66, 78), 70))
    return save(im, name)


def decal_blood(name, seed):
    rng = random.Random(seed)
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    blob(d, 16, 16, 11, 9, (92, 22, 20), seed, alpha=235)
    blob(d, 15, 15, 6, 5, (124, 30, 26), seed + 1, alpha=245)
    for _ in range(12):
        a = rng.random() * math.tau
        dist = rng.uniform(7, 15)
        x = int(16 + math.cos(a) * dist)
        y = int(16 + math.sin(a) * dist)
        r = rng.randint(1, 2)
        d.rectangle((x - r, y - r, x + r, y + r), fill=rgba((110, 26, 22), 230))
    return save(im, name)


def decal_rubble(name, seed):
    rng = random.Random(seed)
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    for _ in range(26):
        x = rng.randrange(30)
        y = rng.randrange(30)
        r = rng.randint(1, 3)
        col = rng.choice([CONCRETE_2, CONCRETE_3, BRICK_2, (150, 140, 120),
                          (86, 84, 80), RUST_2])
        d.rectangle((x, y, x + r, y + r), fill=rgba(col, rng.randint(200, 255)))
        d.point((x, y), fill=rgba(mix(col, WHITE, 0.35)))
    return save(im, name)


def decal_crack(name, seed):
    im = new(32, 32)
    crack(im, 4, seed, (52, 52, 56), reach=7, length=(4, 7))
    return save(im, name)


def decal_tire(name, seed):
    rng = random.Random(seed)
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    for y in (4, 18):
        d.rectangle((0, y, 31, y + 6), fill=rgba((42, 40, 42), 150))
        for x in range(0, 32, 4):
            d.line([(x, y), (x, y + 6)], fill=rgba((30, 28, 30), 190))
    return save(im, name)


def decal_scorch(name, seed):
    im = new(32, 32)
    d = ImageDraw.Draw(im)
    blob(d, 16, 16, 14, 12, (42, 36, 32), seed, alpha=200)
    blob(d, 16, 16, 8, 7, (24, 21, 20), seed + 1, alpha=230)
    speck(im, 26, seed + 2, [(66, 58, 50), (98, 86, 72)], 1, 2, (70, 150))
    return save(im, name)

# ---------------------------------------------------------------------------
# props -- top-down, 1px ink outline, native sizes
# ---------------------------------------------------------------------------
def car_wreck(name, seed, body=RUST_2, flip=False):
    rng = random.Random(seed)
    im = new(60, 34)
    d = ImageDraw.Draw(im)
    w, h = 60, 34
    # wheels peeking out at the corners
    for x in (8, w - 20):
        d.rectangle((x, 1, x + 11, 6), fill=rgba((30, 30, 34)))
        d.rectangle((x, h - 7, x + 11, h - 2), fill=rgba((30, 30, 34)))
    # hull
    bevel(d, (3, 5, w - 4, h - 6), body, r=5)
    # roof cabin
    bevel(d, (18, 8, w - 16, h - 9), mix(body, INK, 0.3), r=3)
    # windscreen / rear glass
    d.rectangle((19, 10, 27, h - 11), fill=rgba((92, 116, 124)))
    d.rectangle((w - 27, 10, w - 19, h - 11), fill=rgba((80, 102, 110)))
    d.line([(20, 10), (20, h - 11)], fill=rgba((150, 176, 182)))
    # hood scorch + rust bloom
    d.rectangle((4, 9, 17, h - 10), fill=rgba(mix(body, RUST_1, 0.55)))
    for _ in range(8):
        x = rng.randint(4, w - 8)
        y = rng.randint(7, h - 9)
        r = rng.randint(1, 3)
        d.ellipse((x - r, y - r, x + r, y + r), fill=rgba(RUST_1, 200))
    # shattered glass specks
    for _ in range(6):
        x = rng.randint(19, w - 21)
        y = rng.randint(10, h - 12)
        d.point((x, y), fill=rgba((196, 214, 218)))
    # door seam
    d.line([(30, 8), (30, h - 9)], fill=rgba(mix(body, INK, 0.5)))
    outline(im)
    if flip:
        im = im.transpose(Image.FLIP_LEFT_RIGHT)
    return save(im, name)


def rv_wreck(name, seed):
    rng = random.Random(seed)
    im = new(88, 44)
    d = ImageDraw.Draw(im)
    w, h = 88, 44
    # wheels
    for x in (12, w - 26):
        d.rectangle((x, 1, x + 13, 7), fill=rgba((30, 30, 34)))
        d.rectangle((x, h - 8, x + 13, h - 2), fill=rgba((30, 30, 34)))
    # boxy body
    bevel(d, (3, 5, w - 4, h - 6), (188, 182, 164), r=4)
    # cab at the left
    bevel(d, (4, 8, 24, h - 9), (150, 146, 132), r=3)
    d.rectangle((5, 12, 15, h - 13), fill=rgba((86, 108, 116)))  # windshield
    # faded stripe
    d.rectangle((24, 18, w - 6, 23), fill=rgba((178, 96, 60)))
    d.rectangle((24, 24, w - 6, 26), fill=rgba((196, 168, 96)))
    # roof furniture: AC unit, vent, solar panel
    bevel(d, (34, 10, 50, 22), (146, 146, 140), r=2)
    bevel(d, (58, 12, 68, 20), (128, 130, 126), r=1)
    d.rectangle((72, 10, 84, 22), fill=rgba((52, 66, 88)))
    d.line([(72, 16), (84, 16)], fill=rgba((72, 92, 118)))
    # grime + rust streaks
    patch(im, 24, 6, 62, 34, (122, 104, 78), 0.3, seed, mode="rand")
    for _ in range(7):
        x = rng.randint(26, w - 8)
        y = rng.randint(8, h - 10)
        r = rng.randint(1, 3)
        d.ellipse((x - r, y - r, x + r, y + r), fill=rgba(RUST_2, 190))
    outline(im)
    return save(im, name)


def dumpster(name, seed, col=(78, 104, 76)):
    rng = random.Random(seed)
    im = new(44, 26)
    d = ImageDraw.Draw(im)
    bevel(d, (2, 4, 41, 23), col, r=3)
    d.rectangle((4, 6, 40, 13), fill=rgba(mix(col, WHITE, 0.2)))   # lid
    d.line([(4, 13), (40, 13)], fill=rgba(mix(col, INK, 0.55)))
    d.line([(22, 5), (22, 22)], fill=rgba(mix(col, INK, 0.4)))      # lid split
    for _ in range(9):
        x = rng.randint(4, 38)
        y = rng.randint(6, 21)
        r = rng.randint(1, 3)
        d.ellipse((x - r, y - r, x + r, y + r), fill=rgba(RUST_2, 180))
    speck(im, 14, seed + 1, [RUST_1, mix(col, INK, 0.4)], 1, 2, (70, 150))
    outline(im)
    return save(im, name)


def sofa(name, seed, col=(122, 84, 62)):
    im = new(36, 24)
    d = ImageDraw.Draw(im)
    bevel(d, (2, 3, 33, 20), col, r=3)
    d.rectangle((5, 5, 30, 12), fill=rgba(mix(col, WHITE, 0.22)))   # back cushions
    d.line([(17, 5), (17, 12)], fill=rgba(mix(col, INK, 0.4)))
    d.rectangle((5, 13, 30, 18), fill=rgba(mix(col, INK, 0.18)))    # seat
    patch(im, 4, 4, 28, 16, mix(col, INK, 0.4), 0.3, seed, mode="rand")
    outline(im)
    return save(im, name)


def table(name, seed):
    im = new(26, 22)
    d = ImageDraw.Draw(im)
    bevel(d, (3, 3, 22, 18), WOOD_2, r=2)
    d.rectangle((6, 6, 19, 15), fill=rgba(mix(WOOD_2, WHITE, 0.15)))
    for y in (8, 12):
        d.line([(6, y), (19, y)], fill=rgba(WOOD_3))
    outline(im)
    return save(im, name)


def chair(name, seed, col=(94, 70, 52)):
    im = new(14, 16)
    d = ImageDraw.Draw(im)
    bevel(d, (2, 4, 11, 13), col, r=1)
    d.rectangle((3, 5, 10, 8), fill=rgba(mix(col, INK, 0.35)))
    outline(im)
    return save(im, name)


def shelf(name, seed):
    rng = random.Random(seed)
    im = new(44, 16)
    d = ImageDraw.Draw(im)
    bevel(d, (1, 2, 42, 13), (116, 118, 122), r=1)
    d.rectangle((3, 4, 40, 11), fill=rgba((86, 88, 92)))
    # scattered goods
    for _ in range(7):
        x = rng.randint(4, 36)
        y = rng.randint(5, 9)
        col = rng.choice([RED, YELLOW, TEAL, GREEN_R, ORANGE])
        d.rectangle((x, y, x + 3, y + 3), fill=rgba(mix(col, INK, 0.25)))
    outline(im)
    return save(im, name)


def counter(name, seed):
    im = new(48, 16)
    d = ImageDraw.Draw(im)
    bevel(d, (1, 2, 46, 13), WOOD_1, r=2)
    d.rectangle((3, 4, 44, 7), fill=rgba(mix(WOOD_1, WHITE, 0.2)))
    d.rectangle((34, 4, 44, 10), fill=rgba((64, 68, 72)))   # register
    d.rectangle((36, 5, 41, 7), fill=rgba((150, 190, 160)))
    outline(im)
    return save(im, name)


def pole(name, seed):
    """Telephone pole seen from above: post + crossarm + insulators."""
    im = new(28, 34)
    d = ImageDraw.Draw(im)
    # crossarm runs east-west
    d.rectangle((2, 14, 25, 18), fill=rgba(WOOD_2))
    d.line([(2, 14), (25, 14)], fill=rgba(mix(WOOD_2, WHITE, 0.3)))
    d.line([(2, 18), (25, 18)], fill=rgba(WOOD_3))
    for x in (5, 13, 21):
        d.rectangle((x, 11, x + 2, 14), fill=rgba((70, 78, 86)))   # insulators
        d.point((x, 11), fill=rgba((130, 160, 168)))
    # post (round, seen from above) with lit cap
    d.ellipse((10, 12, 17, 21), fill=rgba(WOOD_2))
    d.ellipse((11, 13, 16, 18), fill=rgba(mix(WOOD_1, WHITE, 0.25)))
    d.ellipse((12, 16, 16, 21), fill=rgba(WOOD_3))
    outline(im)
    return save(im, name)


def wire(name):
    im = new(12, 3)
    d = ImageDraw.Draw(im)
    d.line([(0, 1), (11, 1)], fill=rgba((26, 24, 28, 255)))
    d.point((0, 0), fill=rgba((46, 44, 48, 200)))
    return save(im, name)


def street_lamp(name, seed):
    im = new(16, 30)
    d = ImageDraw.Draw(im)
    d.rectangle((7, 10, 9, 28), fill=rgba((78, 80, 84)))          # pole
    d.line([(7, 10), (7, 28)], fill=rgba((112, 114, 118)))
    d.rectangle((6, 27, 10, 29), fill=rgba((60, 62, 66)))         # base
    # lamp head with a glowing lens
    bevel(d, (3, 2, 13, 12), (96, 98, 102), r=2)
    d.rectangle((5, 5, 11, 10), fill=rgba((255, 214, 140)))
    d.rectangle((6, 6, 10, 8), fill=rgba((255, 240, 200)))
    outline(im)
    return save(im, name)


def traffic_light(name, seed):
    im = new(14, 26)
    d = ImageDraw.Draw(im)
    d.rectangle((6, 16, 8, 24), fill=rgba((72, 74, 78)))
    bevel(d, (2, 3, 12, 17), (60, 62, 60), r=2)
    # three lenses: only the top (red) still glows
    d.ellipse((4, 4, 10, 9), fill=rgba(RED))
    d.point((6, 5), fill=rgba((255, 170, 150)))
    d.ellipse((4, 9, 10, 13), fill=rgba((96, 76, 40)))
    d.ellipse((4, 13, 10, 16), fill=rgba((46, 74, 52)))
    outline(im)
    return save(im, name)


def road_sign(name, seed):
    im = new(26, 34)
    d = ImageDraw.Draw(im)
    d.rectangle((11, 20, 14, 32), fill=rgba((116, 114, 108)))     # post
    bevel(d, (1, 2, 24, 21), (74, 118, 84), r=2)
    d.rectangle((3, 4, 22, 19), fill=rgba((60, 100, 70)))
    d.line([(5, 15), (19, 7)], fill=rgba((216, 214, 206)))        # worn text
    d.line([(5, 10), (15, 10)], fill=rgba((200, 198, 190)))
    d.line([(5, 13), (13, 13)], fill=rgba((188, 186, 178)))
    patch(im, 2, 3, 22, 18, (52, 64, 54), 0.3, seed, mode="rand")
    outline(im)
    return save(im, name)


def gas_pump(name, seed):
    rng = random.Random(seed)
    im = new(20, 30)
    d = ImageDraw.Draw(im)
    bevel(d, (3, 3, 16, 26), (156, 152, 142), r=2)
    d.rectangle((5, 6, 14, 13), fill=rgba((44, 50, 54)))          # display
    d.rectangle((6, 7, 13, 11), fill=rgba((168, 196, 168)))
    d.rectangle((4, 17, 15, 21), fill=rgba(RED))                   # branding band
    d.line([(16, 14), (19, 18), (16, 24)], fill=rgba((36, 36, 38)))  # hose
    for _ in range(5):
        x, y = rng.randint(4, 15), rng.randint(4, 25)
        r = rng.randint(1, 2)
        d.ellipse((x - r, y - r, x + r, y + r), fill=rgba(RUST_2, 170))
    outline(im)
    return save(im, name)


def dead_tree(name, seed):
    """Bare tree seen from above: trunk rosette of branches."""
    rng = random.Random(seed)
    im = new(46, 46)
    d = ImageDraw.Draw(im)
    cx, cy = 23, 23
    trunk = (92, 72, 54)
    for i in range(9):
        a = i / 9 * math.tau + rng.uniform(-0.15, 0.15)
        ln = rng.randint(9, 19)
        x1 = int(cx + math.cos(a) * ln)
        y1 = int(cy + math.sin(a) * ln)
        d.line([(cx, cy), (x1, y1)], fill=rgba(trunk), width=2)
        for _ in range(2):
            a2 = a + rng.uniform(-0.7, 0.7)
            ln2 = rng.randint(5, 10)
            d.line([(x1, y1), (int(x1 + math.cos(a2) * ln2),
                               int(y1 + math.sin(a2) * ln2))],
                   fill=rgba(mix(trunk, INK, 0.25)), width=1)
    d.ellipse((cx - 4, cy - 4, cx + 4, cy + 4), fill=rgba(mix(trunk, WHITE, 0.22)))
    for _ in range(6):   # stubborn dead leaves
        x = rng.randint(6, 39)
        y = rng.randint(6, 39)
        if im.getpixel((x, y))[3] == 0:
            continue
        d.point((x, y), fill=rgba((146, 116, 70)))
    outline(im)
    return save(im, name)


def yucca(name, seed, palm=False):
    rng = random.Random(seed)
    im = new(40, 40)
    d = ImageDraw.Draw(im)
    cx, cy = 20, 20
    count = 11 if palm else 13
    for i in range(count):
        a = i / count * math.tau + rng.uniform(-0.1, 0.1)
        ln = rng.randint(11, 17)
        x1 = cx + math.cos(a) * ln
        y1 = cy + math.sin(a) * ln
        col = OLIVE_2 if i % 2 else DRYGRASS_1
        if palm:
            col = GREEN_R if i % 2 else OLIVE_2
        d.line([(cx, cy), (int(x1), int(y1))], fill=rgba(col), width=2)
        # barbs
        d.point((int(cx + math.cos(a) * (ln - 3) + 1),
                 int(cy + math.sin(a) * (ln - 3))), fill=rgba(mix(col, WHITE, 0.3)))
    d.ellipse((cx - 3, cy - 3, cx + 3, cy + 3), fill=rgba(mix(OLIVE_3, INK, 0.3)))
    outline(im, (22, 24, 18, 255))
    return save(im, name)


def shrub(name, seed):
    rng = random.Random(seed)
    im = new(22, 18)
    d = ImageDraw.Draw(im)
    blob(d, 11, 9, 9, 7, OLIVE_2, seed, alpha=255, steps=10)
    blob(d, 9, 8, 6, 5, mix(OLIVE_2, DRYGRASS_1, 0.5), seed + 1, alpha=255, steps=8)
    patch(im, 2, 2, 18, 14, OLIVE_3, 0.45, seed + 2, mode="checker")
    speck(im, 8, seed + 3, [DRYGRASS_1, OLIVE_1], 1, 1, (80, 160))
    outline(im, (24, 26, 20, 255))
    return save(im, name)


def rock(name, seed, size=(26, 22)):
    rng = random.Random(seed)
    w, h = size
    im = new(w, h)
    d = ImageDraw.Draw(im)
    cx, cy = w // 2, h // 2
    pts = []
    for i in range(8):
        a = i / 8 * math.tau
        r = 0.6 + rng.random() * 0.35
        pts.append((int(cx + math.cos(a) * (w / 2 - 2) * r),
                    int(cy + math.sin(a) * (h / 2 - 2) * r)))
    d.polygon(pts, fill=rgba((124, 120, 112)))
    top = [(x, y - 2) for x, y in pts[:5]]
    d.polygon(top, fill=rgba((152, 148, 138)))
    for _ in range(8):
        x, y = rng.randint(3, w - 4), rng.randint(3, h - 4)
        if im.getpixel((x, y))[3] > 0:
            d.point((x, y), fill=rgba((94, 90, 84)))
    d.line([pts[4] + [pts[0]]][0] if False else pts + [pts[0]], fill=rgba((82, 78, 72)))
    outline(im)
    return save(im, name)


def barrel(name, seed, col=RUST_2, band=YELLOW):
    rng = random.Random(seed)
    im = new(20, 22)
    d = ImageDraw.Draw(im)
    d.ellipse((2, 2, 17, 19), fill=rgba(col))
    d.ellipse((3, 3, 16, 18), fill=rgba(mix(col, WHITE, 0.18)))
    d.ellipse((5, 5, 14, 16), fill=rgba(mix(col, INK, 0.2)))
    d.ellipse((7, 7, 12, 14), fill=rgba(mix(col, INK, 0.45)))   # bung hole
    d.arc((2, 2, 17, 19), 200, 340, fill=rgba(mix(col, WHITE, 0.4)))
    d.rectangle((3, 10, 16, 13), fill=rgba(band))               # hazard band
    for _ in range(5):
        x, y = rng.randint(4, 15), rng.randint(4, 17)
        if im.getpixel((x, y))[3] > 0:
            d.point((x, y), fill=rgba(RUST_1))
    outline(im)
    return save(im, name)


def crate(name, seed, wood=False):
    rng = random.Random(seed)
    im = new(24, 24)
    d = ImageDraw.Draw(im)
    if wood:
        bevel(d, (2, 2, 21, 21), WOOD_1, r=1)
        d.line([(3, 3), (20, 20)], fill=rgba(WOOD_3))
        d.line([(20, 3), (3, 20)], fill=rgba(WOOD_3))
        d.rectangle((2, 10, 21, 13), fill=rgba(mix(WOOD_1, WHITE, 0.2)))
    else:
        bevel(d, (2, 2, 21, 21), (96, 106, 114), r=1)
        d.line([(4, 4), (19, 19)], fill=rgba((70, 78, 86)))
        d.line([(19, 4), (4, 19)], fill=rgba((70, 78, 86)))
        d.rectangle((7, 10, 16, 14), fill=rgba(YELLOW))
        d.point((8, 11), fill=rgba(mix(YELLOW, WHITE, 0.5)))
    for _ in range(5):
        x, y = rng.randint(3, 19), rng.randint(3, 19)
        d.point((x, y), fill=rgba(RUST_1 if not wood else WOOD_3))
    outline(im)
    return save(im, name)


def tire_pile(name, seed):
    im = new(24, 20)
    d = ImageDraw.Draw(im)
    for i, (dx, dy) in enumerate(((0, 0), (5, 3), (-3, 6))):
        x, y = 12 + dx, 10 + dy
        r = 8 - i
        d.ellipse((x - r, y - r, x + r, y + r), fill=rgba((36, 36, 40)))
        d.ellipse((x - r + 2, y - r + 2, x + r - 2, y + r - 2),
                  fill=rgba((56, 56, 60)))
        d.ellipse((x - 3, y - 3, x + 3, y + 3), fill=rgba((24, 24, 28)))
        d.arc((x - r, y - r, x + r, y + r), 180, 320, fill=rgba((78, 78, 84)))
    outline(im)
    return save(im, name)


def debris_pile(name, seed):
    rng = random.Random(seed)
    im = new(34, 28)
    d = ImageDraw.Draw(im)
    blob(d, 17, 15, 15, 11, (104, 96, 84), seed, alpha=255, steps=12)
    for _ in range(30):
        x, y = rng.randint(3, 30), rng.randint(4, 25)
        r = rng.randint(1, 3)
        col = rng.choice([CONCRETE_2, RUST_2, (154, 142, 118), (86, 84, 80),
                          BRICK_2, OLIVE_2])
        d.rectangle((x, y, x + r, y + r), fill=rgba(col))
        d.point((x, y), fill=rgba(mix(col, WHITE, 0.3)))
    for _ in range(4):   # twisted rebar
        x, y = rng.randint(6, 28), rng.randint(6, 22)
        d.line([(x, y), (x + rng.randint(-8, 8), y + rng.randint(-6, 6))],
               fill=rgba(RUST_3), width=1)
    outline(im)
    return save(im, name)


def barrier(name, seed):
    im = new(36, 14)
    d = ImageDraw.Draw(im)
    bevel(d, (1, 3, 34, 12), CONCRETE_2, r=2)
    d.rectangle((2, 4, 33, 6), fill=rgba(CONCRETE_1))
    x = 3
    while x < 32:
        d.polygon([(x, 11), (x + 4, 11), (x + 7, 5), (x + 3, 5)],
                  fill=rgba(YELLOW))
        d.polygon([(x + 4, 11), (x + 8, 11), (x + 11, 5), (x + 7, 5)],
                  fill=rgba((52, 50, 48)))
        x += 8
    outline(im)
    return save(im, name)


def sandbags(name, seed):
    rng = random.Random(seed)
    im = new(42, 18)
    d = ImageDraw.Draw(im)
    for row, y in enumerate((3, 9)):
        x = 2 + (3 if row else 0)
        while x < 38:
            col = rng.choice([OLIVE_1, OLIVE_2, OLIVE_3, (104, 110, 80)])
            d.rectangle((x, y, x + 11, y + 7), fill=rgba(col))
            d.line([(x + 1, y + 1), (x + 10, y + 1)],
                   fill=rgba(mix(col, WHITE, 0.3)))
            d.line([(x + 1, y + 6), (x + 10, y + 6)],
                   fill=rgba(mix(col, INK, 0.45)))
            x += 11
    outline(im)
    return save(im, name)

# ---------------------------------------------------------------------------
# characters -- 24x36, 3/4 top-down: 0=down, 1=left, 2=up, 3=right
# ---------------------------------------------------------------------------
def _legs(d, facing, frame, pant, boot):
    """Walk cycle: frame 0 left leg leads, frame 1 right leg leads."""
    if facing == "side":
        back0 = 26 if frame == 0 else 28
        front0 = 28 if frame == 0 else 26
        d.rectangle((7, back0, 11, back0 + 6), fill=rgba(pant))
        d.rectangle((7, back0 + 5, 11, back0 + 6), fill=rgba(boot))
        d.rectangle((13, front0, 17, front0 + 6), fill=rgba(pant))
        d.rectangle((13, front0 + 5, 17, front0 + 6), fill=rgba(boot))
    else:
        lead = 28 if frame == 0 else 26
        trail = 26 if frame == 0 else 28
        d.rectangle((7, trail, 11, trail + 6), fill=rgba(pant))
        d.rectangle((7, trail + 5, 11, trail + 6), fill=rgba(boot))
        d.rectangle((13, lead, 17, lead + 6), fill=rgba(pant))
        d.rectangle((13, lead + 5, 17, lead + 6), fill=rgba(boot))


def draw_survivor(im, facing, frame):
    d = ImageDraw.Draw(im)
    _legs(d, facing, frame, LEATHER_2, (56, 44, 36))
    if facing == "side":
        # duster seen in profile
        bevel(d, (7, 15, 17, 29), LEATHER_1, r=3)
        d.rectangle((14, 17, 16, 27), fill=rgba(LEATHER_2))
        d.rectangle((15, 16, 21, 19), fill=rgba(LEATHER_1))       # forward arm
        d.rectangle((19, 16, 21, 18), fill=rgba(SKIN_2))          # hand
        d.rectangle((7, 17, 9, 25), fill=rgba(LEATHER_2))         # back arm
        # head + wide brim, face to the right
        d.ellipse((3, 4, 21, 15), fill=rgba(LEATHER_2))           # brim
        d.ellipse((4, 5, 20, 14), fill=rgba((110, 84, 56)))
        d.rectangle((15, 9, 19, 14), fill=rgba(SKIN_1))           # face
        d.point((18, 11), fill=rgba(INK))
        d.ellipse((8, 5, 17, 12), fill=rgba((96, 74, 50)))        # crown
        d.line([(9, 6), (16, 6)], fill=rgba((126, 98, 66)))
    else:
        back = facing == "up"
        bevel(d, (5, 15, 18, 29), LEATHER_1, r=4)                 # duster
        d.rectangle((11, 16, 12, 28), fill=rgba(LEATHER_2))       # coat seam
        d.rectangle((3, 16, 6, 25), fill=rgba(LEATHER_1))         # arms
        d.rectangle((17, 16, 20, 25), fill=rgba(LEATHER_1))
        d.rectangle((3, 24, 6, 25), fill=rgba(SKIN_2))            # hands
        d.rectangle((17, 24, 20, 25), fill=rgba(SKIN_2))
        if back:
            bevel(d, (8, 17, 15, 25), OLIVE_1, r=2)               # pack
            d.line([(8, 21), (15, 21)], fill=rgba(OLIVE_3))
            d.rectangle((7, 16, 16, 18), fill=rgba(LEATHER_2))    # collar
        else:
            d.rectangle((9, 16, 14, 20), fill=rgba((62, 54, 46)))  # shirt
        # wide-brim hat
        d.ellipse((3, 3, 20, 16), fill=rgba(LEATHER_2))           # brim
        d.ellipse((4, 4, 19, 15), fill=rgba((110, 84, 56)))
        d.ellipse((7, 4, 16, 13), fill=rgba((96, 74, 50)))        # crown
        d.line([(8, 5), (15, 5)], fill=rgba((126, 98, 66)))
        if not back:
            d.rectangle((9, 12, 14, 16), fill=rgba(SKIN_1))       # face
            d.point((10, 13), fill=rgba(INK))
            d.point((13, 13), fill=rgba(INK))
    noise(im, 4, 7)


def draw_raider(im, facing, frame):
    d = ImageDraw.Draw(im)
    _legs(d, facing, frame, (62, 64, 70), (38, 38, 42))
    if facing == "side":
        bevel(d, (7, 15, 17, 29), (86, 92, 96), r=3)              # vest
        d.rectangle((7, 17, 17, 20), fill=rgba((62, 66, 70)))
        d.rectangle((14, 16, 21, 19), fill=rgba(SKIN_2))          # forward arm
        d.rectangle((7, 17, 9, 25), fill=rgba(LEATHER_2))         # back arm
        d.ellipse((10, 5, 19, 15), fill=rgba(SKIN_2))             # head
        d.ellipse((10, 5, 19, 10), fill=rgba(HAIR_2))             # hair
        d.rectangle((11, 10, 19, 12), fill=rgba(RED_D))           # headband
        d.point((17, 13), fill=rgba(INK))                         # eye
        d.line([(14, 14), (18, 14)], fill=rgba(SKIN_3))           # jaw
    else:
        back = facing == "up"
        bevel(d, (5, 15, 18, 29), (86, 92, 96), r=4)              # vest
        d.rectangle((6, 18, 17, 26), fill=rgba((60, 64, 68)))     # plate
        d.line([(6, 22), (17, 22)], fill=rgba((44, 46, 50)))
        d.rectangle((3, 15, 7, 22), fill=rgba(LEATHER_2))         # pads
        d.rectangle((16, 15, 20, 22), fill=rgba(LEATHER_2))
        d.rectangle((3, 22, 6, 26), fill=rgba(SKIN_2))            # arms
        d.rectangle((17, 22, 20, 26), fill=rgba(SKIN_2))
        # head
        d.ellipse((7, 5, 16, 17), fill=rgba(SKIN_2))
        d.ellipse((7, 5, 16, 10), fill=rgba(HAIR_2))
        d.rectangle((7, 10, 16, 12), fill=rgba(RED_D))            # headband
        if back:
            d.ellipse((8, 6, 15, 12), fill=rgba(HAIR_2))
        else:
            d.point((10, 14), fill=rgba(INK))
            d.point((13, 14), fill=rgba(INK))
            d.line([(10, 16), (13, 16)], fill=rgba(SKIN_3))
    noise(im, 4, 13)


def draw_infected(im, facing, frame):
    d = ImageDraw.Draw(im)
    _legs(d, facing, frame, MUTANT_3, (74, 84, 58))
    if facing == "side":
        # hunched silhouette, skull thrust forward
        bevel(d, (5, 14, 16, 29), MUTANT_2, r=3)
        for y in (17, 20, 23, 26):
            d.line([(7, y), (14, y)], fill=rgba(MUTANT_3))
        d.rectangle((13, 14, 21, 18), fill=rgba(MUTANT_2))        # reaching arm
        d.rectangle((19, 14, 21, 16), fill=rgba((206, 202, 176))) # claws
        d.ellipse((11, 5, 21, 15), fill=rgba(MUTANT_1))           # skull
        d.rectangle((17, 8, 21, 12), fill=rgba(MUTANT_2))
        d.point((19, 9), fill=rgba(YELLOW))                       # eye
        d.line([(17, 13), (21, 13)], fill=rgba((60, 66, 44)))     # jaw
    else:
        back = facing == "up"
        bevel(d, (5, 14, 18, 29), MUTANT_2, r=4)
        for y in (17, 20, 23, 26):
            d.line([(7, y), (16, y)], fill=rgba(MUTANT_3))        # ribs
        d.rectangle((2, 15, 6, 27), fill=rgba(MUTANT_2))          # arms
        d.rectangle((17, 15, 21, 27), fill=rgba(MUTANT_2))
        d.rectangle((2, 26, 5, 27), fill=rgba((206, 202, 176)))   # claws
        d.rectangle((18, 26, 21, 27), fill=rgba((206, 202, 176)))
        d.ellipse((7, 5, 16, 17), fill=rgba(MUTANT_1))
        d.ellipse((8, 6, 15, 12), fill=rgba(MUTANT_2))
        if back:
            d.line([(9, 8), (14, 8)], fill=rgba(MUTANT_3))
        else:
            d.point((9, 12), fill=rgba(YELLOW))
            d.point((14, 12), fill=rgba(YELLOW))
            d.rectangle((9, 14, 14, 16), fill=rgba((52, 58, 40)))
    noise(im, 5, 23)


FACINGS = ("down", "side", "up", "side")   # index 0=down 1=left 2=up 3=right


def character_frames(prefix, draw_fn):
    names = []
    for dir_index in range(4):
        for frame in (0, 1):
            im = new(24, 36)
            draw_fn(im, FACINGS[dir_index], frame)
            if dir_index == 1:     # left = mirrored right-facing body
                im = im.transpose(Image.FLIP_LEFT_RIGHT)
            outline(im)
            names.append(save(im, f"{prefix}_{dir_index}_{frame}.png"))
    return names


def gun_overlay(name, sleeve, metal=(62, 62, 68), stock=(86, 62, 40)):
    """Rifle drawn pointing east; pivot = sprite centre (16, 6)."""
    im = new(32, 12)
    d = ImageDraw.Draw(im)
    d.polygon([(3, 5), (13, 5), (13, 9), (3, 8)], fill=rgba(stock))     # stock
    d.line([(4, 5), (12, 5)], fill=rgba(mix(stock, WHITE, 0.3)))
    bevel(d, (12, 4, 23, 9), metal, r=1)                                 # receiver
    d.rectangle((17, 9, 21, 11), fill=rgba((44, 44, 48)))                # magazine
    d.rectangle((14, 8, 17, 11), fill=rgba((50, 50, 54)))                # grip
    d.rectangle((23, 5, 30, 7), fill=rgba((42, 42, 46)))                 # barrel
    d.rectangle((29, 4, 31, 8), fill=rgba((36, 36, 40)))                 # muzzle
    d.rectangle((18, 2, 20, 4), fill=rgba((48, 48, 52)))                 # sight
    d.line([(13, 4), (22, 4)], fill=rgba(mix(metal, WHITE, 0.35)))
    d.rectangle((12, 6, 17, 10), fill=rgba(sleeve))                      # hands
    d.point((13, 7), fill=rgba(mix(sleeve, WHITE, 0.25)))
    outline(im)
    return save(im, name)

# ---------------------------------------------------------------------------
# fx
# ---------------------------------------------------------------------------
def jagged(d, cx, cy, r, color, seed, points=14, jitter=0.45):
    rng = random.Random(seed)
    pts = []
    for i in range(points):
        a = i / points * math.tau
        rr = r * (1.0 - jitter / 2 + rng.random() * jitter)
        pts.append((cx + math.cos(a) * rr, cy + math.sin(a) * rr))
    d.polygon(pts, fill=rgba(color))


def bullet(name):
    im = new(10, 4)
    d = ImageDraw.Draw(im)
    d.rectangle((1, 1, 8, 2), fill=rgba((255, 226, 150)))
    d.rectangle((3, 1, 7, 1), fill=rgba(WHITE))
    d.point((9, 1), fill=rgba((255, 244, 200)))
    return save(im, name)


def tracer(name):
    im = new(16, 2)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 15, 1), fill=rgba(WHITE))
    return save(im, name)


def muzzle_flash(name):
    im = new(24, 24)
    d = ImageDraw.Draw(im)
    c = 12
    jagged(d, c, c, 10, (255, 196, 96), 5, points=8, jitter=0.7)
    jagged(d, c, c, 6, (255, 240, 190), 6, points=8, jitter=0.6)
    d.rectangle((c + 2, c - 1, c + 11, c + 1), fill=rgba((255, 226, 150)))
    d.point((c, c), fill=rgba(WHITE))
    return save(im, name)


def casing(name):
    im = new(5, 3)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 4, 2), fill=rgba((214, 176, 92)))
    d.point((1, 0), fill=rgba((244, 214, 130)))
    return save(im, name)


def soft_puff(name, size, color, peak=160, power=2.4):
    return banded_glow(name, size, color, peak, power=power)


def spark(name):
    im = new(3, 3)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 2, 2), fill=rgba((255, 224, 150)))
    d.point((1, 1), fill=rgba(WHITE))
    return save(im, name)


def shadow(name, w=40, h=24, peak=150):
    im = new(w, h)
    px = im.load()
    for y in range(h):
        for x in range(w):
            dx = (x + 0.5 - w / 2) / (w / 2)
            dy = (y + 0.5 - h / 2) / (h / 2)
            dist = math.hypot(dx, dy)
            if dist > 1.0:
                continue
            band = int((1.0 - dist) * peak) // 32 * 32
            if band > 0:
                px[x, y] = (8, 8, 16, min(255, band))
    return save(im, name)


def explosion_frames():
    names = []
    for i in range(4):
        im = new(48, 48)
        d = ImageDraw.Draw(im)
        c = 24
        t = i / 3
        if i < 3:
            r = 14 + i * 5
            jagged(d, c, c, r, (255, 214, 120), 40 + i, points=13, jitter=0.5)
            jagged(d, c, c, int(r * 0.7), (255, 244, 206) if i == 0
                   else (240, 150, 60), 50 + i, points=11, jitter=0.55)
            if i > 0:
                jagged(d, c, c, int(r * 0.45), (150, 70, 44), 60 + i,
                       points=10, jitter=0.6)
        else:
            for k in range(7):
                a = k / 7 * math.tau
                blob(d, int(c + math.cos(a) * 12), int(c + math.sin(a) * 11),
                     7, 6, (72, 66, 64), 70 + k, alpha=230, steps=9)
            blob(d, c, c, 9, 8, (52, 48, 48), 77, alpha=240, steps=9)
        names.append(save(im, f"explosion_{i + 1}.png"))
    return names


def crosshair(name):
    im = new(13, 13)
    d = ImageDraw.Draw(im)
    c = 6
    col = (238, 244, 236)
    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        for step in (4, 5, 6):
            d.point((c + dx * step, c + dy * step), fill=rgba(col))
    d.point((c, c), fill=rgba(col))
    d.point((c + 1, c), fill=rgba((120, 130, 124)))
    return save(im, name)


def hit_marker(name):
    im = new(11, 11)
    d = ImageDraw.Draw(im)
    for s in (-1, 1):
        for k in range(3, 6):
            d.point((5 + s * k, 5 + s * k), fill=rgba((255, 92, 74)))
            d.point((5 + s * k, 5 - s * k), fill=rgba((255, 92, 74)))
    return save(im, name)


# ---------------------------------------------------------------------------
# lighting + vignette
# ---------------------------------------------------------------------------
def vignette(name):
    im = new(160, 90)
    px = im.load()
    for y in range(90):
        for x in range(160):
            dx = abs(x - 80) / 80.0
            dy = abs(y - 45) / 45.0
            edge = max(dx, dy)
            t = max(0.0, edge - 0.55) / 0.45
            band = int(t * t * 190) // 32 * 32
            if band > 0:
                px[x, y] = (4, 5, 12, min(255, band))
    return save(im, name)


# ---------------------------------------------------------------------------
# hud
# ---------------------------------------------------------------------------
def ui_panel(name):
    im = new(16, 16)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 15, 15), fill=rgba((14, 20, 26, 236)))
    d.rectangle((0, 0, 15, 15), outline=rgba((168, 132, 64, 255)))
    d.rectangle((2, 2, 13, 13), outline=rgba((74, 62, 42, 255)))
    for x, y in ((1, 1), (14, 1), (1, 14), (14, 14)):
        d.point((x, y), fill=rgba((226, 190, 110, 255)))
    return save(im, name)


def ui_slot(name, fill=(22, 30, 36, 230), border=(96, 104, 100, 255)):
    im = new(14, 14)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 13, 13), fill=fill)
    d.rectangle((0, 0, 13, 13), outline=border)
    d.line([(2, 2), (11, 2)], fill=rgba(mix(border[:3], WHITE, 0.3)))
    return save(im, name)


def icon(name, kind):
    im = new(12, 12)
    d = ImageDraw.Draw(im)
    if kind == "health":
        d.rectangle((4, 2, 7, 9), fill=rgba(RED))
        d.rectangle((2, 4, 9, 7), fill=rgba(RED))
        d.point((4, 3), fill=rgba((222, 110, 92)))
        d.point((3, 5), fill=rgba((222, 110, 92)))
    elif kind == "stamina":
        d.polygon([(7, 1), (3, 6), (5, 6), (4, 11), (9, 5), (6, 5)],
                  fill=rgba(YELLOW))
        d.point((6, 3), fill=rgba((255, 234, 160)))
    elif kind == "ammo":
        d.rectangle((4, 3, 7, 9), fill=rgba((226, 208, 152)))
        d.rectangle((4, 8, 7, 10), fill=rgba((176, 158, 112)))
        d.polygon([(4, 3), (7, 3), (5, 1)], fill=rgba((196, 176, 128)))
        d.point((5, 4), fill=rgba((250, 240, 200)))
    elif kind == "reload":
        d.arc((2, 2, 9, 9), 40, 320, fill=rgba(TEAL_L), width=2)
        d.polygon([(7, 1), (11, 4), (6, 5)], fill=rgba(TEAL_L))
    return save(im, name)


def portrait(name):
    """Small survivor bust for menu / game-over."""
    im = new(48, 48)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 47, 47), fill=rgba((30, 34, 38)))
    d.rectangle((8, 34, 39, 47), fill=rgba(LEATHER_1))          # coat
    d.rectangle((20, 30, 27, 38), fill=rgba(SKIN_2))            # neck
    d.ellipse((14, 10, 33, 32), fill=rgba(SKIN_1))              # head
    d.rectangle((17, 20, 30, 23), fill=rgba((44, 48, 52)))      # goggles band
    d.rectangle((18, 21, 23, 23), fill=rgba(TEAL))
    d.rectangle((25, 21, 30, 23), fill=rgba(TEAL))
    d.pieslice((12, 5, 35, 26), 180, 360, fill=rgba((96, 74, 50)))  # hat crown
    d.rectangle((6, 16, 41, 20), fill=rgba(LEATHER_2))          # brim
    d.rectangle((14, 36, 33, 41), fill=rgba((128, 74, 52)))     # scarf
    d.rectangle((0, 0, 47, 47), outline=rgba((168, 132, 64, 255)))
    noise(im, 6, 3)
    return save(im, name)

# ---------------------------------------------------------------------------
# manifest + generation
# ---------------------------------------------------------------------------
def _v_variant(name_h):
    """Rotate a horizontal road tile 90 degrees for vertical roads."""
    name_v = name_h.replace("_h.png", "_v.png")
    im = Image.open(OUT / name_h)
    im.transpose(Image.ROTATE_90).save(OUT / name_v)
    return name_v


def main():
    names = []

    # terrain ------------------------------------------------------------
    for i in (1, 2, 3):
        names.append(tile_sand(f"tile_sand_{i}.png", 10 + i,
                               (SAND_1, SAND_2, SAND_3)[i - 1]))
    for i in (1, 2, 3):
        names.append(tile_dirt(f"tile_dirt_{i}.png", 20 + i))
    for i in (1, 2):
        names.append(tile_gravel(f"tile_gravel_{i}.png", 30 + i))
    for i in (1, 2):
        names.append(tile_drygrass(f"tile_drygrass_{i}.png", 40 + i))
    for i in (1, 2, 3):
        names.append(tile_asphalt(f"tile_asphalt_{i}.png", 50 + i))
    for i, mark in enumerate(("double_h", "dash_h", "edge_h", "cross_h"), 1):
        h = tile_road(f"tile_road_{mark}.png", 60 + i, mark=mark)
        names.append(h)
        names.append(_v_variant(h))
    for i in (1, 2):
        names.append(tile_sidewalk(f"tile_sidewalk_{i}.png", 70 + i))
    for i in (1, 2):
        names.append(tile_checker(f"tile_checker_{i}.png", 80 + i))
    for i in (1, 2):
        names.append(tile_wood(f"tile_wood_{i}.png", 90 + i))
    for i in (1, 2):
        names.append(tile_concrete(f"tile_concrete_{i}.png", 100 + i))

    # walls --------------------------------------------------------------
    for i in (1, 2):
        names.append(wall_tile(f"wall_brick_{i}.png", "brick", 110 + i))
    for i in (1, 2):
        names.append(wall_tile(f"wall_brick_broken_{i}.png", "brick", 120 + i,
                               broken=True))
    for i in (1, 2):
        names.append(wall_tile(f"wall_concrete_{i}.png", "concrete", 130 + i))
    names.append(wall_tile("wall_concrete_broken.png", "concrete", 140,
                           broken=True))
    for i in (1, 2):
        names.append(wall_tile(f"wall_metal_{i}.png", "metal", 150 + i))

    # decals -------------------------------------------------------------
    for i in (1, 2):
        names.append(decal_leaf(f"decal_leaf_{i}.png", 160 + i))
    for i in (1, 2):
        names.append(decal_grass(f"decal_grass_{i}.png", 170 + i))
    names.append(decal_oil("decal_oil.png", 180))
    for i in (1, 2):
        names.append(decal_blood(f"decal_blood_{i}.png", 190 + i))
    for i in (1, 2):
        names.append(decal_rubble(f"decal_rubble_{i}.png", 200 + i))
    for i in (1, 2):
        names.append(decal_crack(f"decal_crack_{i}.png", 210 + i))
    for i in (1, 2):
        names.append(decal_tire(f"decal_tire_{i}.png", 220 + i))
    names.append(decal_scorch("decal_scorch.png", 230))

    # props --------------------------------------------------------------
    names.append(car_wreck("car_wreck_1.png", 301, RUST_2))
    names.append(car_wreck("car_wreck_2.png", 302, (96, 110, 124), flip=True))
    names.append(car_wreck("car_wreck_3.png", 303, (140, 118, 78)))
    names.append(rv_wreck("rv_wreck.png", 310))
    names.append(dumpster("dumpster_1.png", 320, (78, 104, 76)))
    names.append(dumpster("dumpster_2.png", 321, (112, 88, 64)))
    names.append(sofa("sofa.png", 330))
    names.append(table("table.png", 331))
    names.append(chair("chair.png", 332))
    names.append(shelf("shelf.png", 333))
    names.append(counter("counter.png", 334))
    names.append(pole("pole.png", 340))
    names.append(wire("wire.png"))
    names.append(street_lamp("street_lamp.png", 341))
    names.append(traffic_light("traffic_light.png", 342))
    names.append(road_sign("road_sign.png", 343))
    names.append(gas_pump("gas_pump.png", 344))
    names.append(dead_tree("dead_tree_1.png", 350))
    names.append(dead_tree("dead_tree_2.png", 351))
    for i in (1, 2):
        names.append(yucca(f"yucca_{i}.png", 360 + i))
    names.append(yucca("palm.png", 370, palm=True))
    for i in (1, 2):
        names.append(shrub(f"shrub_{i}.png", 380 + i))
    names.append(rock("rock_1.png", 390, (26, 22)))
    names.append(rock("rock_2.png", 391, (20, 17)))
    names.append(rock("rock_3.png", 392, (15, 13)))
    names.append(barrel("barrel_rust.png", 400, RUST_2, YELLOW))
    names.append(barrel("barrel_blue.png", 401, (72, 94, 124),
                        (222, 222, 210)))
    names.append(crate("crate_metal.png", 410, wood=False))
    names.append(crate("crate_wood.png", 411, wood=True))
    for i in (1, 2):
        names.append(tire_pile(f"tire_pile_{i}.png", 420 + i))
    for i in (1, 2):
        names.append(debris_pile(f"debris_pile_{i}.png", 430 + i))
    names.append(barrier("barrier.png", 440))
    names.append(sandbags("sandbags.png", 441))

    # characters ---------------------------------------------------------
    names += character_frames("player_body", draw_survivor)
    names.append(gun_overlay("player_gun.png", LEATHER_1))
    names += character_frames("raider_body", draw_raider)
    names.append(gun_overlay("raider_gun.png", (86, 92, 96),
                             metal=(52, 52, 56), stock=(76, 62, 44)))
    names += character_frames("infected_body", draw_infected)

    # fx -----------------------------------------------------------------
    names.append(bullet("bullet.png"))
    names.append(tracer("tracer.png"))
    names.append(muzzle_flash("muzzle_flash.png"))
    names.append(casing("casing.png"))
    names.append(soft_puff("dust_puff.png", 16, (198, 182, 150), peak=160))
    names.append(soft_puff("smoke.png", 32, (116, 114, 112), peak=176,
                           power=2.0))
    names.append(spark("spark.png"))
    names.append(shadow("shadow.png", 40, 24))
    names.append(shadow("shadow_small.png", 24, 16, peak=140))
    names.append(decal_blood("blood_splat.png", 450))
    names += explosion_frames()
    names.append(crosshair("crosshair.png"))
    names.append(hit_marker("hit_marker.png"))

    # lighting -----------------------------------------------------------
    names.append(banded_glow("glow_warm.png", 64, (255, 196, 118), 176,
                             power=2.0))
    names.append(banded_glow("glow_cool.png", 64, (176, 200, 232), 128,
                             power=2.2))
    names.append(banded_glow("glow_fire.png", 48, (255, 150, 60), 208,
                             power=1.8))
    names.append(vignette("vignette.png"))

    # hud ----------------------------------------------------------------
    names.append(ui_panel("ui_panel.png"))
    names.append(ui_slot("ui_slot.png"))
    names.append(ui_slot("ui_slot_hover.png", (34, 46, 52, 240),
                         (168, 176, 160, 255)))
    names.append(icon("icon_health.png", "health"))
    names.append(icon("icon_stamina.png", "stamina"))
    names.append(icon("icon_ammo.png", "ammo"))
    names.append(icon("icon_reload.png", "reload"))
    names.append(portrait("portrait.png"))

    # prune sprites from older passes ------------------------------------
    keep = set(names)
    for path in sorted(OUT.glob("*.png")):
        if path.name not in keep:
            path.unlink()

    print(f"generated {len(names)} sprites into {OUT}")


if __name__ == "__main__":
    main()
