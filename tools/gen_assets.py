#!/usr/bin/env python3
"""Generate the original StarCraft-inspired sprite set for SHITSTORM 2126.

Everything is drawn procedurally so the game ships with original art only.
Run with:  python3 tools/gen_assets.py
Requires Pillow (pip install pillow).
"""

from __future__ import annotations

import math
import random
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "sprites"
FONTS = ROOT / "assets" / "fonts"
OUT.mkdir(parents=True, exist_ok=True)

SS = 4  # supersampling factor

# Pillow's ellipse requires ordered coordinates; a lot of the procedural
# sprites mirror shapes with a negative sign.
_orig_ellipse = ImageDraw.ImageDraw.ellipse


def _safe_ellipse(self, xy, *args, **kwargs):
    x0, y0, x1, y1 = xy
    if x0 > x1:
        x0, x1 = x1, x0
    if y0 > y1:
        y0, y1 = y1, y0
    return _orig_ellipse(self, (x0, y0, x1, y1), *args, **kwargs)


ImageDraw.ImageDraw.ellipse = _safe_ellipse

# ---------------------------------------------------------------------------
# palette
# ---------------------------------------------------------------------------
STEEL_0 = (18, 22, 27)
STEEL_1 = (34, 41, 49)
STEEL_2 = (51, 61, 71)
STEEL_3 = (74, 87, 99)
STEEL_4 = (108, 124, 137)
STEEL_5 = (154, 171, 181)

RUST_1 = (72, 48, 34)
RUST_2 = (105, 70, 44)
RUST_3 = (141, 96, 56)
RUST_4 = (176, 128, 74)

AMBER = (231, 165, 51)
GOLD = (212, 177, 94)
CYAN = (70, 220, 255)
CYAN_D = (22, 105, 138)
CYAN_L = (180, 247, 255)
BLUE = (58, 125, 205)

PURPLE_0 = (35, 13, 48)
PURPLE_1 = (64, 26, 82)
PURPLE_2 = (98, 43, 120)
PURPLE_3 = (139, 70, 158)
PURPLE_4 = (188, 112, 200)
ZERG_GREEN = (113, 196, 88)

RED = (207, 61, 45)
RED_D = (112, 28, 25)
GREEN = (90, 200, 120)
BONE = (214, 207, 178)
WHITE = (255, 255, 255)


def rgba(c, a=None):
    if a is not None:
        return (c[0], c[1], c[2], a)
    if len(c) == 4:
        return c
    return (c[0], c[1], c[2], 255)


def lerp(a, b, t):
    n = min(len(a), len(b))
    out = [int(a[i] + (b[i] - a[i]) * t) for i in range(n)]
    if n == 3:
        return tuple(out)
    return tuple(out)


def mix(a, b, t):
    return lerp(a, b, max(0.0, min(1.0, t)))


def canvas(w, h, scale=SS):
    return Image.new("RGBA", (int(w * scale), int(h * scale)), (0, 0, 0, 0))


def S(v, scale=SS):
    return int(round(v * scale))


def finish(img, w, h, name, scale=SS):
    img = img.resize((int(w), int(h)), Image.LANCZOS)
    img.save(OUT / name)


def save(img, name):
    img.save(OUT / name)


def draw_noise(img, amount=10, seed=0, tint=(255, 255, 255, 255)):
    rng = random.Random(seed)
    px = img.load()
    w, h = img.size
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if a == 0:
                continue
            n = rng.randint(-amount, amount)
            px[x, y] = (
                max(0, min(255, r + n + (tint[0] - 255) // 8)),
                max(0, min(255, g + n + (tint[1] - 255) // 8)),
                max(0, min(255, b + n + (tint[2] - 255) // 8)),
                a,
            )


def glow(img, radius, color, strength=1.0):
    """Add a soft glow derived from the bright parts of ``img``."""
    a = img.getchannel("A")
    mask = ImageChops.multiply(a, img.convert("L"))
    g = mask.filter(ImageFilter.GaussianBlur(radius))
    g = Image.eval(g, lambda v: int(v * strength))
    layer = Image.new("RGBA", img.size, rgba(color, 0))
    layer.putalpha(g)
    return Image.alpha_composite(layer, img)


def bevel_rect(d, box, fill, light=None, dark=None, radius=3, width=1, outline=None):
    x0, y0, x1, y1 = box
    light = light or mix(fill, WHITE, 0.34)
    dark = dark or mix(fill, (0, 0, 0, 255), 0.42)
    if len(fill) == 4 and fill[3] == 0:
        return
    d.rounded_rectangle(box, radius=radius, fill=fill, outline=outline)
    # top / left highlights
    d.line([(x0 + radius, y0 + width), (x1 - radius, y0 + width)], fill=light, width=width)
    d.line([(x0 + width, y0 + radius), (x0 + width, y1 - radius)], fill=light, width=width)
    # bottom / right shadows
    d.line([(x0 + radius, y1 - width), (x1 - radius, y1 - width)], fill=dark, width=width)
    d.line([(x1 - width, y0 + radius), (x1 - width, y1 - radius)], fill=dark, width=width)


def bolts(d, box, color=STEEL_4, r=1.2, inset=3.0, scale=SS):
    x0, y0, x1, y1 = box
    for x in (x0 + S(inset, scale), x1 - S(inset, scale)):
        for y in (y0 + S(inset, scale), y1 - S(inset, scale)):
            rr = S(r, scale)
            d.ellipse((x - rr, y - rr, x + rr, y + rr), fill=color)
            d.ellipse((x - rr + S(0.5, scale), y - rr + S(0.5, scale), x + rr, y + rr), fill=mix(color, WHITE, 0.3))


def polygon(d, pts, fill, outline=None, width=1):
    d.polygon(pts, fill=fill)
    if outline:
        d.line(pts + [pts[0]], fill=outline, width=width, joint="curve")


def glow_dot(d, pos, radius, color, core=WHITE, scale=SS):
    x, y = pos
    r = S(radius, scale)
    d.ellipse((x - r, y - r, x + r, y + r), fill=color)
    rr = max(1, int(r * 0.45))
    d.ellipse((x - rr, y - rr, x + rr, y + rr), fill=core)


def radial_gradient(size, inner, outer, power=2.0):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    px = img.load()
    c = size / 2
    for y in range(size):
        for x in range(size):
            dist = math.hypot(x - c + 0.5, y - c + 0.5) / c
            if dist > 1.0:
                continue
            t = max(0.0, 1.0 - dist) ** power
            col = lerp(outer, inner, t)
            px[x, y] = (col[0], col[1], col[2], int(col[3] * min(1.0, t * 2.2)))
    return img


# ---------------------------------------------------------------------------
# terrain
# ---------------------------------------------------------------------------
def terrain_metal(name, seed, rust=0.18, hazard=False, base=STEEL_2):
    rng = random.Random(seed)
    img = canvas(64, 64)
    d = ImageDraw.Draw(img)
    w = S(64)
    d.rectangle((0, 0, w, w), fill=base)
    # large plates
    for _ in range(3):
        x = rng.randint(-10, 50)
        y = rng.randint(-10, 50)
        pw = rng.randint(22, 42)
        ph = rng.randint(18, 38)
        box = (S(x), S(y), S(x + pw), S(y + ph))
        c = mix(base, STEEL_3 if rng.random() > 0.5 else STEEL_1, rng.uniform(0.15, 0.55))
        bevel_rect(d, box, c, radius=S(1), width=max(1, S(0.35)))
    # seams
    for x in (0, 32):
        d.line([(S(x), 0), (S(x), w)], fill=STEEL_0, width=max(1, S(0.6)))
        d.line([(S(x) + S(0.7), 0), (S(x) + S(0.7), w)], fill=STEEL_4, width=max(1, S(0.25)))
    for y in (0, 32):
        d.line([(0, S(y)), (w, S(y))], fill=STEEL_0, width=max(1, S(0.6)))
        d.line([(0, S(y) + S(0.7)), (w, S(y) + S(0.7))], fill=STEEL_4, width=max(1, S(0.25)))
    # rivets
    for x in (5, 27, 37, 59):
        for y in (5, 27, 37, 59):
            bolts(d, (S(x) - 1, S(y) - 1, S(x) + 1, S(y) + 1), r=0.5, inset=0.2)
    # scratches
    for _ in range(rng.randint(4, 10)):
        x = rng.uniform(2, 62)
        y = rng.uniform(2, 62)
        ln = rng.uniform(1.5, 6)
        a = rng.uniform(0, math.tau)
        d.line(
            [(S(x), S(y)), (S(x + math.cos(a) * ln), S(y + math.sin(a) * ln))],
            fill=mix(STEEL_4, WHITE, 0.2) + (60,),
            width=max(1, S(0.18)),
        )
    # rust patches
    if rust > 0:
        for _ in range(int(rust * 20)):
            x = rng.uniform(0, 64)
            y = rng.uniform(0, 64)
            rr = rng.uniform(1.5, 7)
            c = mix(RUST_2, RUST_3, rng.random()) + (rng.randint(50, 130),)
            d.ellipse((S(x - rr), S(y - rr), S(x + rr), S(y + rr)), fill=c)
    if hazard:
        for i in range(-2, 8):
            d.polygon(
                [
                    (S(i * 12), S(64)),
                    (S(i * 12 + 7), S(64)),
                    (S(i * 12 + 25), S(52)),
                    (S(i * 12 + 18), S(52)),
                ],
                fill=AMBER + (150,),
            )
    draw_noise(img, 13, seed)
    finish(img, 64, 64, name)


def terrain_grate():
    rng = random.Random(7)
    img = canvas(64, 64)
    d = ImageDraw.Draw(img)
    d.rectangle((0, 0, S(64), S(64)), fill=STEEL_1)
    # under-glow
    d.rectangle((0, 0, S(64), S(64)), fill=CYAN_D + (90,))
    for i in range(8):
        for j in range(8):
            x = i * 8
            y = j * 8
            d.rectangle((S(x + 1), S(y + 1), S(x + 7), S(y + 7)), fill=(0, 0, 0, 0))
            bevel_rect(
                d,
                (S(x + 0.5), S(y + 0.5), S(x + 7.5), S(y + 7.5)),
                STEEL_2 + (230,),
                radius=S(0.8),
                width=max(1, S(0.3)),
            )
            d.rectangle((S(x + 2), S(y + 2), S(x + 6), S(y + 6)), fill=STEEL_0 + (235,))
    # cyan strips seen through the grate
    d.line([(0, S(12)), (S(64), S(12))], fill=CYAN + (100,), width=S(1.2))
    d.line([(0, S(44)), (S(64), S(44))], fill=CYAN + (80,), width=S(1.0))
    draw_noise(img, 8, 11)
    finish(img, 64, 64, "tile_grate.png")


def terrain_rock(name, seed):
    rng = random.Random(seed)
    img = canvas(64, 64)
    d = ImageDraw.Draw(img)
    d.rectangle((0, 0, S(64), S(64)), fill=(20, 21, 22, 255))
    for _ in range(16):
        x = rng.uniform(0, 64)
        y = rng.uniform(0, 64)
        rr = rng.uniform(5, 18)
        pts = []
        for k in range(7):
            a = k / 7 * math.tau
            rad = rr * rng.uniform(0.65, 1.0)
            pts.append((S(x + math.cos(a) * rad), S(y + math.sin(a) * rad)))
        c = mix((45, 45, 44, 255), (92, 88, 80, 255), rng.random())
        polygon(d, pts, c, outline=(12, 13, 14, 255), width=S(0.6))
    for _ in range(8):
        x = rng.uniform(0, 64)
        y = rng.uniform(0, 64)
        d.line(
            [(S(x), S(y)), (S(x + rng.uniform(-12, 12)), S(y + rng.uniform(-12, 12)))],
            fill=(150, 140, 120, 45),
            width=S(0.35),
        )
    draw_noise(img, 15, seed)
    finish(img, 64, 64, name)


def terrain_creep(name, seed):
    rng = random.Random(seed)
    img = canvas(64, 64)
    d = ImageDraw.Draw(img)
    d.rectangle((0, 0, S(64), S(64)), fill=PURPLE_1)
    for _ in range(22):
        x = rng.uniform(0, 64)
        y = rng.uniform(0, 64)
        rr = rng.uniform(4, 16)
        d.ellipse(
            (S(x - rr), S(y - rr), S(x + rr), S(y + rr)),
            fill=mix(PURPLE_2, PURPLE_3, rng.random()) + (rng.randint(70, 140),),
        )
    # veins
    for _ in range(7):
        x = rng.uniform(0, 64)
        y = rng.uniform(0, 64)
        pts = [(S(x), S(y))]
        for _ in range(4):
            x += rng.uniform(-12, 12)
            y += rng.uniform(-12, 12)
            pts.append((S(x), S(y)))
        d.line(pts, fill=PURPLE_4 + (180,), width=S(0.9), joint="curve")
        d.line(pts, fill=ZERG_GREEN + (70,), width=S(0.35), joint="curve")
    for _ in range(5):
        glow_dot(d, (S(rng.uniform(4, 60)), S(rng.uniform(4, 60))), rng.uniform(0.8, 2.0), ZERG_GREEN + (220,))
    draw_noise(img, 12, seed)
    finish(img, 64, 64, name)


def base_pad():
    img = canvas(512, 512)
    d = ImageDraw.Draw(img)
    c = S(256)
    # outer hazard ring
    d.ellipse((c - S(250), c - S(250), c + S(250), c + S(250)), fill=STEEL_0 + (210,))
    d.ellipse((c - S(244), c - S(244), c + S(244), c + S(244)), fill=STEEL_2 + (235,))
    for i in range(48):
        a0 = i / 48 * math.tau
        a1 = (i + 0.48) / 48 * math.tau
        if i % 2 == 0:
            pts = [
                (c + math.cos(a0) * S(236), c + math.sin(a0) * S(236)),
                (c + math.cos(a0) * S(250), c + math.sin(a0) * S(250)),
                (c + math.cos(a1) * S(250), c + math.sin(a1) * S(250)),
                (c + math.cos(a1) * S(236), c + math.sin(a1) * S(236)),
            ]
            d.polygon(pts, fill=AMBER + (190,))
    # inner plates
    d.ellipse((c - S(230), c - S(230), c + S(230), c + S(230)), fill=STEEL_1 + (245,))
    d.ellipse((c - S(222), c - S(222), c + S(222), c + S(222)), fill=STEEL_2 + (245,))
    for k in range(12):
        a = k / 12 * math.tau
        x = c + math.cos(a) * S(190)
        y = c + math.sin(a) * S(190)
        d.line([(c, c), (x, y)], fill=STEEL_0 + (90,), width=S(2.0))
    d.ellipse((c - S(160), c - S(160), c + S(160), c + S(160)), fill=STEEL_1 + (255,), outline=STEEL_4 + (255,), width=S(1.5))
    d.ellipse((c - S(152), c - S(152), c + S(152), c + S(152)), fill=STEEL_0 + (255,))
    # landing circle
    d.ellipse((c - S(126), c - S(126), c + S(126), c + S(126)), fill=STEEL_2 + (255,), outline=CYAN_D + (255,), width=S(2.0))
    d.ellipse((c - S(110), c - S(110), c + S(110), c + S(110)), fill=STEEL_1 + (255,))
    # H marking
    d.rectangle((c - S(54), c - S(32), c - S(34), c + S(32)), fill=GOLD + (220,))
    d.rectangle((c + S(34), c - S(32), c + S(54), c + S(32)), fill=GOLD + (220,))
    d.rectangle((c - S(34), c - S(8), c + S(34), c + S(8)), fill=GOLD + (220,))
    # corner lights
    for k in range(8):
        a = k / 8 * math.tau + math.pi / 8
        x = c + math.cos(a) * S(210)
        y = c + math.sin(a) * S(210)
        glow_dot(d, (x, y), 6, CYAN + (255,))
    for _ in range(90):
        x = random.uniform(0, 512)
        y = random.uniform(0, 512)
        if math.hypot(x - 256, y - 256) < 238:
            ln = random.uniform(2, 15)
            a = random.uniform(0, math.tau)
            d.line(
                [(S(x), S(y)), (S(x + math.cos(a) * ln), S(y + math.sin(a) * ln))],
                fill=(180, 195, 200, 40),
                width=S(0.3),
            )
    draw_noise(img, 9, 21)
    finish(img, 512, 512, "base_pad.png")


# ---------------------------------------------------------------------------
# structures
# ---------------------------------------------------------------------------
def command_center():
    img = canvas(256, 256)
    d = ImageDraw.Draw(img)
    cx = cy = S(128)
    # raised octagonal platform
    pts = []
    for k in range(8):
        a = k / 8 * math.tau + math.pi / 8
        pts.append((cx + math.cos(a) * S(106), cy + math.sin(a) * S(106)))
    d.polygon([(x + S(4), y + S(5)) for x, y in pts], fill=(0, 0, 0, 120))
    polygon(d, pts, STEEL_1, outline=STEEL_0, width=S(1.4))
    for k in range(8):
        a = k / 8 * math.tau + math.pi / 8
        x = cx + math.cos(a) * S(96)
        y = cy + math.sin(a) * S(96)
        a2 = a + math.pi / 8
        x2 = cx + math.cos(a2) * S(96)
        y2 = cy + math.sin(a2) * S(96)
        d.line([(x, y), (x2, y2)], fill=STEEL_3, width=S(1.2))
        glow_dot(d, (x, y), 3.0, CYAN + (230,))
    # panel ring
    for k in range(8):
        a = k / 8 * math.tau + math.pi / 8
        x = cx + math.cos(a) * S(64)
        y = cy + math.sin(a) * S(64)
        d.line([(cx, cy), (x, y)], fill=STEEL_0, width=S(1.2))
    # central command tower
    bevel_rect(
        d,
        (cx - S(54), cy - S(54), cx + S(54), cy + S(54)),
        STEEL_2,
        radius=S(6),
        width=S(1.4),
    )
    bevel_rect(
        d,
        (cx - S(44), cy - S(44), cx + S(44), cy + S(44)),
        STEEL_3,
        radius=S(5),
        width=S(1.0),
    )
    # roof panels
    d.polygon(
        [
            (cx - S(38), cy - S(26)),
            (cx - S(20), cy - S(40)),
            (cx + S(20), cy - S(40)),
            (cx + S(38), cy - S(26)),
            (cx + S(38), cy + S(26)),
            (cx + S(20), cy + S(40)),
            (cx - S(20), cy + S(40)),
            (cx - S(38), cy + S(26)),
        ],
        fill=STEEL_2,
        outline=STEEL_0,
        width=S(0.8),
    )
    # landing hatch
    d.ellipse((cx - S(22), cy - S(22), cx + S(22), cy + S(22)), fill=STEEL_0, outline=CYAN_D, width=S(1.4))
    d.ellipse((cx - S(17), cy - S(17), cx + S(17), cy + S(17)), fill=STEEL_1)
    d.line([(cx - S(10), cy), (cx + S(10), cy)], fill=CYAN + (220,), width=S(1.0))
    d.line([(cx, cy - S(10)), (cx, cy + S(10))], fill=CYAN + (220,), width=S(1.0))
    # radar dish
    dx, dy = cx + S(56), cy - S(56)
    d.ellipse((dx - S(20), dy - S(20), dx + S(20), dy + S(20)), fill=STEEL_0, outline=STEEL_4, width=S(1.2))
    d.arc((dx - S(18), dy - S(18), dx + S(18), dy + S(18)), 200, 340, fill=GOLD, width=S(1.6))
    d.line([(dx, dy), (dx + S(13), dy - S(13))], fill=STEEL_4, width=S(1.3))
    glow_dot(d, (dx + S(13), dy - S(13)), 2.4, CYAN + (255,))
    # antennae
    for ox, oy in ((-52, -58), (52, 58), (-56, 52)):
        x, y = cx + S(ox), cy + S(oy)
        d.line([(x, y), (x + S(3), y - S(20))], fill=STEEL_4, width=S(1.0))
        glow_dot(d, (x + S(3), y - S(20)), 2.0, RED + (230,))
    # corner turret pods
    for ox, oy in ((-78, -78), (78, -78), (-78, 78), (78, 78)):
        x, y = cx + S(ox), cy + S(oy)
        d.ellipse((x - S(12), y - S(12), x + S(12), y + S(12)), fill=STEEL_0)
        d.ellipse((x - S(9), y - S(9), x + S(9), y + S(9)), fill=STEEL_3)
        d.line([(x, y), (x, y - S(14))], fill=STEEL_5, width=S(2.2))
        glow_dot(d, (x, y), 2.2, AMBER + (255,))
    # pipes / grime
    for _ in range(16):
        x = random.uniform(cx - 90, cx + 90)
        y = random.uniform(cy - 90, cy + 90)
        d.line([(S(x), S(y)), (S(x + random.uniform(-18, 18)), S(y + random.uniform(-4, 4)))], fill=RUST_2 + (50,), width=S(0.7))
    draw_noise(img, 8, 22)
    finish(img, 256, 256, "hq_command.png")


def tower_base(name, accent=CYAN):
    img = canvas(72, 72)
    d = ImageDraw.Draw(img)
    c = S(36)
    # shadow ring
    d.ellipse((c - S(33), c - S(33), c + S(33), c + S(33)), fill=(0, 0, 0, 80))
    pts = []
    for k in range(8):
        a = k / 8 * math.tau + math.pi / 8
        pts.append((c + math.cos(a) * S(30), c + math.sin(a) * S(30)))
    polygon(d, pts, STEEL_1, outline=STEEL_0, width=S(1.0))
    pts2 = []
    for k in range(8):
        a = k / 8 * math.tau + math.pi / 8
        pts2.append((c + math.cos(a) * S(23), c + math.sin(a) * S(23)))
    polygon(d, pts2, STEEL_2, outline=STEEL_3, width=S(0.6))
    d.ellipse((c - S(14), c - S(14), c + S(14), c + S(14)), fill=STEEL_0, outline=accent + (180,), width=S(1.2))
    for k in range(4):
        a = k / 4 * math.tau + math.pi / 4
        x = c + math.cos(a) * S(26)
        y = c + math.sin(a) * S(26)
        glow_dot(d, (x, y), 2.3, accent + (220,))
    for ox, oy in ((-28, -28), (28, -28), (-28, 28), (28, 28)):
        glow_dot(d, (c + S(ox), c + S(oy)), 1.2, STEEL_4)
    draw_noise(img, 7, 30)
    finish(img, 72, 72, name)


def gauss_head():
    img = canvas(64, 64)
    d = ImageDraw.Draw(img)
    c = S(32)
    # mounting ring
    d.ellipse((c - S(20), c - S(20), c + S(20), c + S(20)), fill=STEEL_0)
    d.ellipse((c - S(17), c - S(17), c + S(17), c + S(17)), fill=STEEL_3)
    # body
    bevel_rect(d, (c - S(14), c - S(20), c + S(14), c + S(14)), STEEL_2, radius=S(4), width=S(0.8))
    # twin barrels
    for ox in (-8, 8):
        d.rounded_rectangle(
            (c + S(ox - 3), c - S(31), c + S(ox + 3), c - S(4)),
            radius=S(1.5),
            fill=STEEL_4,
            outline=STEEL_0,
            width=S(0.6),
        )
        d.line([(c + S(ox - 1), c - S(30)), (c + S(ox - 1), c - S(8))], fill=STEEL_5, width=S(0.5))
        glow_dot(d, (c + S(ox), c - S(30)), 1.4, CYAN + (255,))
    # sensor eye
    glow_dot(d, (c, c + S(2)), 4.2, CYAN + (255,))
    d.rectangle((c - S(10), c + S(10), c + S(10), c + S(13)), fill=STEEL_0)
    draw_noise(img, 5, 31)
    finish(img, 64, 64, "tower_gauss_head.png")


def missile_head():
    img = canvas(72, 72)
    d = ImageDraw.Draw(img)
    c = S(36)
    d.ellipse((c - S(22), c - S(22), c + S(22), c + S(22)), fill=STEEL_0)
    d.ellipse((c - S(19), c - S(19), c + S(19), c + S(19)), fill=STEEL_3)
    bevel_rect(d, (c - S(19), c - S(22), c + S(19), c + S(16)), STEEL_2, radius=S(4), width=S(0.8))
    # missile pod 2x2
    for i in range(2):
        for j in range(2):
            x = c - S(12) + i * S(13)
            y = c - S(18) + j * S(11)
            d.rounded_rectangle((x, y, x + S(10), y + S(9)), radius=S(2), fill=STEEL_0, outline=STEEL_4, width=S(0.5))
            d.ellipse((x + S(2), y + S(1), x + S(8), y + S(7)), fill=(40, 38, 34, 255), outline=AMBER + (200,), width=S(0.5))
            glow_dot(d, (x + S(5), y + S(4)), 1.2, AMBER + (255,))
    # side vents
    for ox in (-22, 22):
        d.rectangle((c + S(ox) - S(3), c - S(8), c + S(ox) + S(3), c + S(8)), fill=STEEL_0)
        for k in range(4):
            d.line([(c + S(ox) - S(2), c - S(6) + k * S(3)), (c + S(ox) + S(2), c - S(6) + k * S(3))], fill=STEEL_4, width=S(0.5))
    glow_dot(d, (c, c + S(3)), 2.6, AMBER + (255,))
    draw_noise(img, 5, 32)
    finish(img, 72, 72, "tower_missile_head.png")


def arc_head():
    img = canvas(64, 64)
    d = ImageDraw.Draw(img)
    c = S(32)
    d.ellipse((c - S(20), c - S(20), c + S(20), c + S(20)), fill=STEEL_0)
    d.ellipse((c - S(17), c - S(17), c + S(17), c + S(17)), fill=STEEL_3)
    # prongs
    for a in (0, 2.1, 4.2):
        x = c + math.cos(a) * S(12)
        y = c + math.sin(a) * S(12)
        d.line([(c, c), (x, y)], fill=STEEL_4, width=S(3.0))
        glow_dot(d, (x, y), 3.0, CYAN + (255,))
    # orb
    glow_dot(d, (c, c), 9.0, CYAN + (255,), core=CYAN_L)
    d.ellipse((c - S(4), c - S(4), c + S(4), c + S(4)), fill=CYAN_L)
    for _ in range(10):
        a = random.uniform(0, math.tau)
        r0 = random.uniform(8, 13)
        r1 = random.uniform(15, 21)
        d.line(
            [(c + math.cos(a) * S(r0), c + math.sin(a) * S(r0)), (c + math.cos(a + 0.4) * S(r1), c + math.sin(a + 0.4) * S(r1))],
            fill=CYAN + (150,),
            width=S(0.6),
        )
    draw_noise(img, 4, 33)
    finish(img, 64, 64, "tower_arc_head.png")


def relay():
    img = canvas(64, 64)
    d = ImageDraw.Draw(img)
    c = S(32)
    d.ellipse((c - S(27), c - S(27), c + S(27), c + S(27)), fill=(0, 0, 0, 80))
    pts = []
    for k in range(6):
        a = k / 6 * math.tau + math.pi / 6
        pts.append((c + math.cos(a) * S(24), c + math.sin(a) * S(24)))
    polygon(d, pts, STEEL_1, outline=STEEL_0, width=S(0.9))
    d.ellipse((c - S(16), c - S(16), c + S(16), c + S(16)), fill=STEEL_2, outline=STEEL_4, width=S(0.8))
    # pylon
    d.polygon(
        [
            (c - S(8), c + S(6)),
            (c - S(4), c - S(18)),
            (c + S(4), c - S(18)),
            (c + S(8), c + S(6)),
        ],
        fill=STEEL_3,
        outline=STEEL_0,
        width=S(0.6),
    )
    glow_dot(d, (c, c - S(16)), 6.0, CYAN + (255,), core=CYAN_L)
    for k in range(3):
        a = -math.pi / 2 + (k - 1) * 2.1
        x = c + math.cos(a) * S(17)
        y = c + math.sin(a) * S(17)
        glow_dot(d, (x, y), 2.4, CYAN + (220,))
    draw_noise(img, 5, 34)
    finish(img, 64, 64, "relay.png")


# ---------------------------------------------------------------------------
# units
# ---------------------------------------------------------------------------
def goliath_legs(frame):
    img = canvas(80, 80)
    d = ImageDraw.Draw(img)
    c = S(40)
    swing = S(4) if frame == 0 else -S(4)
    # shadow
    d.ellipse((c - S(29), c - S(24), c + S(29), c + S(24)), fill=(0, 0, 0, 70))
    # legs: side pods, alternating
    for side, sign in ((0, -1), (1, 1)):
        ox = sign * S(23)
        oy = swing if side == 0 else -swing
        # upper leg
        d.rounded_rectangle(
            (c + ox - S(9), c - S(20) + oy, c + ox + S(9), c + S(18) - oy),
            radius=S(4),
            fill=STEEL_2,
            outline=STEEL_0,
            width=S(1.0),
        )
        # knee / foot
        d.ellipse((c + ox - S(10), c + S(11) - oy, c + ox + S(10), c + S(30) - oy), fill=STEEL_1, outline=STEEL_0, width=S(0.8))
        d.rounded_rectangle(
            (c + ox - S(8), c + S(20) - oy, c + ox + S(8), c + S(30) - oy),
            radius=S(3),
            fill=STEEL_3,
            outline=STEEL_0,
            width=S(0.6),
        )
        d.line([(c + ox, c - S(17) + oy), (c + ox, c + S(12) - oy)], fill=STEEL_4, width=S(0.8))
        glow_dot(d, (c + ox, c + S(26) - oy), 2.0, CYAN + (220,))
    # hip / lower chassis
    d.ellipse((c - S(18), c - S(19), c + S(18), c + S(19)), fill=STEEL_1, outline=STEEL_0, width=S(1.0))
    d.ellipse((c - S(14), c - S(15), c + S(14), c + S(15)), fill=STEEL_3)
    # rear thruster
    d.rectangle((c - S(7), c + S(13), c + S(7), c + S(20)), fill=STEEL_0)
    for k in range(3):
        d.line([(c - S(5), c + S(15) + k * S(2)), (c + S(5), c + S(15) + k * S(2))], fill=CYAN + (140,), width=S(0.5))
    draw_noise(img, 6, 40 + frame)
    finish(img, 80, 80, f"goliath_legs_{frame + 1}.png")


def goliath_torso():
    img = canvas(80, 80)
    d = ImageDraw.Draw(img)
    c = S(40)
    # shoulder armor
    for sign in (-1, 1):
        x = c + sign * S(22)
        d.ellipse((x - S(13), c - S(14), x + S(13), c + S(18)), fill=STEEL_1, outline=STEEL_0, width=S(1.0))
        d.ellipse((x - S(10), c - S(11), x + S(10), c + S(15)), fill=STEEL_3)
        d.arc((x - S(10), c - S(11), x + S(10), c + S(15)), 200, 340, fill=STEEL_5, width=S(0.8))
    # cockpit
    d.ellipse((c - S(18), c - S(20), c + S(18), c + S(16)), fill=STEEL_2, outline=STEEL_0, width=S(1.1))
    # visor
    d.rounded_rectangle((c - S(11), c - S(11), c + S(11), c - S(2)), radius=S(3), fill=(8, 16, 22, 255), outline=CYAN_D, width=S(0.7))
    d.rounded_rectangle((c - S(9), c - S(9), c + S(9), c - S(4)), radius=S(2), fill=CYAN + (210,))
    d.line([(c - S(8), c - S(7)), (c + S(8), c - S(7))], fill=WHITE + (200,), width=S(0.6))
    # twin cannons
    for ox in (-12, 12):
        d.rounded_rectangle(
            (c + S(ox - 4), c - S(32), c + S(ox + 4), c - S(7)),
            radius=S(2),
            fill=STEEL_3,
            outline=STEEL_0,
            width=S(0.7),
        )
        d.rectangle((c + S(ox - 2), c - S(32), c + S(ox + 2), c - S(26)), fill=STEEL_5)
        d.line([(c + S(ox - 2), c - S(28)), (c + S(ox + 2), c - S(28))], fill=(0, 0, 0, 180), width=S(0.5))
    # team stripe
    d.rectangle((c - S(15), c + S(11), c + S(15), c + S(15)), fill=BLUE + (230,))
    # antenna
    d.line([(c + S(16), c - S(4)), (c + S(19), c - S(22))], fill=STEEL_5, width=S(0.7))
    glow_dot(d, (c + S(19), c - S(22)), 1.4, CYAN + (255,))
    draw_noise(img, 6, 41)
    finish(img, 80, 80, "goliath_torso.png")


def zergling():
    img = canvas(56, 56)
    d = ImageDraw.Draw(img)
    c = S(28)
    # legs
    for sign in (-1, 1):
        for k in range(2):
            x = c + sign * S(10) + sign * S(k * 6)
            y = c + (k - 0.5) * S(15)
            d.line([(c, c), (x + sign * S(11), y + S(4))], fill=PURPLE_0, width=S(4.0))
            d.line([(c, c), (x + sign * S(11), y + S(4))], fill=PURPLE_2, width=S(2.2))
            d.ellipse((x + sign * S(8), y + S(1), x + sign * S(15), y + S(8)), fill=PURPLE_0, outline=PURPLE_3, width=S(0.5))
    # body
    d.ellipse((c - S(15), c - S(12), c + S(15), c + S(12)), fill=PURPLE_1, outline=PURPLE_0, width=S(1.0))
    d.ellipse((c - S(11), c - S(8), c + S(11), c + S(8)), fill=PURPLE_2)
    # carapace ridges
    for k in range(4):
        x = c - S(8) + k * S(5)
        d.arc((x - S(3), c - S(8), x + S(3), c + S(8)), 90, 270, fill=PURPLE_4, width=S(0.8))
    # head / jaws
    d.polygon(
        [
            (c + S(13), c - S(7)),
            (c + S(25), c - S(3)),
            (c + S(27), c),
            (c + S(25), c + S(3)),
            (c + S(13), c + S(7)),
        ],
        fill=PURPLE_1,
        outline=PURPLE_0,
        width=S(0.8),
    )
    for oy in (-1, 1):
        d.line([(c + S(20), c + oy * S(2)), (c + S(29), c + oy * S(5))], fill=BONE + (220,), width=S(1.2))
    glow_dot(d, (c + S(19), c - S(2)), 2.0, ZERG_GREEN + (255,))
    glow_dot(d, (c + S(19), c + S(2)), 2.0, ZERG_GREEN + (255,))
    draw_noise(img, 8, 50)
    finish(img, 56, 56, "enemy_zergling.png")


def hydralisk():
    img = canvas(72, 72)
    d = ImageDraw.Draw(img)
    c = S(36)
    # tail coils
    for k in range(3):
        a = k * 0.9
        d.arc(
            (c - S(22 + k * 3), c - S(14 + k * 2), c + S(10 + k * 3), c + S(24 + k * 2)),
            40,
            320,
            fill=PURPLE_1,
            width=S(4.0 - k * 0.5),
        )
    # body
    d.ellipse((c - S(18), c - S(14), c + S(16), c + S(14)), fill=PURPLE_1, outline=PURPLE_0, width=S(1.0))
    d.ellipse((c - S(13), c - S(10), c + S(11), c + S(10)), fill=PURPLE_2)
    # hood
    d.polygon(
        [
            (c + S(8), c - S(12)),
            (c + S(28), c - S(18)),
            (c + S(31), c - S(5)),
            (c + S(27), c + S(5)),
            (c + S(8), c + S(12)),
        ],
        fill=PURPLE_1,
        outline=PURPLE_0,
        width=S(0.9),
    )
    for k in range(4):
        d.line(
            [(c + S(11), c - S(8) + k * S(5)), (c + S(27), c - S(12) + k * S(5))],
            fill=PURPLE_3 + (220,),
            width=S(0.8),
        )
    # scythes
    for sign in (-1, 1):
        d.line(
            [(c + S(6), c + sign * S(8)), (c + S(20), c + sign * S(22))],
            fill=PURPLE_0,
            width=S(3.2),
        )
        d.line(
            [(c + S(20), c + sign * S(22)), (c + S(32), c + sign * S(30))],
            fill=BONE + (230,),
            width=S(1.8),
        )
        d.line(
            [(c + S(20), c + sign * S(22)), (c + S(33), c + sign * S(24))],
            fill=BONE + (200,),
            width=S(1.2),
        )
    glow_dot(d, (c + S(21), c - S(4)), 2.4, ZERG_GREEN + (255,))
    glow_dot(d, (c + S(21), c + S(4)), 2.4, ZERG_GREEN + (255,))
    draw_noise(img, 8, 51)
    finish(img, 72, 72, "enemy_hydralisk.png")


def ultralisk():
    img = canvas(104, 104)
    d = ImageDraw.Draw(img)
    c = S(52)
    # legs
    for sign in (-1, 1):
        for k in range(2):
            x = c + sign * S(22)
            y = c + (k - 0.5) * S(28)
            d.line([(c, c), (x + sign * S(18), y + S(8))], fill=PURPLE_0, width=S(8.0))
            d.line([(c, c), (x + sign * S(18), y + S(8))], fill=PURPLE_2, width=S(4.0))
            d.ellipse(
                (x + sign * S(12), y + S(2), x + sign * S(24), y + S(14)),
                fill=PURPLE_0,
                outline=PURPLE_3,
                width=S(0.7),
            )
    # huge carapace
    d.ellipse((c - S(34), c - S(28), c + S(30), c + S(28)), fill=PURPLE_1, outline=PURPLE_0, width=S(1.4))
    d.ellipse((c - S(28), c - S(22), c + S(24), c + S(22)), fill=PURPLE_2)
    for k in range(5):
        d.arc(
            (c - S(24) + k * S(8), c - S(20), c - S(4) + k * S(8), c + S(20)),
            100,
            260,
            fill=PURPLE_3,
            width=S(1.4),
        )
    # head
    d.polygon(
        [
            (c + S(22), c - S(18)),
            (c + S(47), c - S(10)),
            (c + S(50), c),
            (c + S(47), c + S(10)),
            (c + S(22), c + S(18)),
        ],
        fill=PURPLE_1,
        outline=PURPLE_0,
        width=S(1.0),
    )
    # tusks
    for sign in (-1, 1):
        d.line(
            [(c + S(37), c + sign * S(8)), (c + S(54), c + sign * S(23))],
            fill=BONE + (235,),
            width=S(2.6),
        )
        d.line(
            [(c + S(42), c + sign * S(3)), (c + S(55), c + sign * S(10))],
            fill=BONE + (220,),
            width=S(1.8),
        )
    glow_dot(d, (c + S(34), c - S(5)), 3.0, ZERG_GREEN + (255,))
    glow_dot(d, (c + S(34), c + S(5)), 3.0, ZERG_GREEN + (255,))
    draw_noise(img, 10, 52)
    finish(img, 104, 104, "enemy_ultralisk.png")


# ---------------------------------------------------------------------------
# effects
# ---------------------------------------------------------------------------
def bullet(name, w, h, col, core=WHITE):
    img = canvas(w, h)
    d = ImageDraw.Draw(img)
    d.rounded_rectangle((0, 0, S(w), S(h)), radius=S(w / 2), fill=col)
    d.rounded_rectangle((S(w * 0.25), S(h * 0.1), S(w * 0.75), S(h * 0.9)), radius=S(w * 0.2), fill=core)
    img = glow(img, S(2), col, 0.8)
    finish(img, w, h, name)


def muzzle_flash():
    img = canvas(48, 48)
    d = ImageDraw.Draw(img)
    c = S(24)
    # bursts
    for k in range(8):
        a = k / 8 * math.tau
        r0 = S(3)
        r1 = S(13 + (k % 2) * 6)
        d.polygon(
            [
                (c + math.cos(a - 0.22) * r0, c + math.sin(a - 0.22) * r0),
                (c + math.cos(a) * r1, c + math.sin(a) * r1),
                (c + math.cos(a + 0.22) * r0, c + math.sin(a + 0.22) * r0),
            ],
            fill=AMBER + (220,),
        )
    d.ellipse((c - S(8), c - S(8), c + S(8), c + S(8)), fill=(255, 238, 180, 255))
    d.ellipse((c - S(4), c - S(4), c + S(4), c + S(4)), fill=WHITE)
    img = glow(img, S(3), AMBER, 0.9)
    finish(img, 48, 48, "muzzle_flash.png")


def tracer():
    img = canvas(64, 8)
    d = ImageDraw.Draw(img)
    for y in range(S(8)):
        t = 1.0 - abs(y / S(8) - 0.5) * 2.0
        a = int(80 + t * 175)
        d.line([(0, y), (S(64), y)], fill=CYAN + (a,))
    d.line([(0, S(4)), (S(64), S(4))], fill=CYAN_L, width=S(1.2))
    finish(img, 64, 8, "tracer.png")


def explosion_frames():
    for f in range(6):
        img = canvas(96, 96)
        d = ImageDraw.Draw(img)
        c = S(48)
        t = f / 5
        # smoke
        for k in range(9):
            a = random.uniform(0, math.tau)
            dist = random.uniform(0, 18 + f * 5)
            rr = random.uniform(12, 26) + f * 2
            x = c + math.cos(a) * S(dist)
            y = c + math.sin(a) * S(dist)
            col = mix((70, 64, 58, 200), (28, 27, 26, 120), t)
            d.ellipse((x - S(rr), y - S(rr), x + S(rr), y + S(rr)), fill=col)
        # fireball
        radius = (30 - f * 3.4) * (1.0 + f * 0.15)
        if f < 3:
            d.ellipse(
                (c - S(radius), c - S(radius), c + S(radius), c + S(radius)),
                fill=(255, 196, 74, 230 - f * 40),
            )
            d.ellipse(
                (c - S(radius * 0.65), c - S(radius * 0.65), c + S(radius * 0.65), c + S(radius * 0.65)),
                fill=(255, 240, 190, 240),
            )
        # sparks
        for k in range(14):
            a = random.uniform(0, math.tau)
            dist = random.uniform(6, 20 + f * 7)
            x = c + math.cos(a) * S(dist)
            y = c + math.sin(a) * S(dist)
            d.ellipse((x - S(1.2), y - S(1.2), x + S(1.2), y + S(1.2)), fill=AMBER + (200,))
        img = glow(img, S(4), (255, 150, 45, 255), 0.6)
        draw_noise(img, 8, 60 + f)
        finish(img, 96, 96, f"explosion_{f + 1}.png")


def smoke():
    img = canvas(64, 64)
    d = ImageDraw.Draw(img)
    c = S(32)
    for k in range(12):
        a = random.uniform(0, math.tau)
        dist = random.uniform(0, 14)
        rr = random.uniform(8, 18)
        x = c + math.cos(a) * S(dist)
        y = c + math.sin(a) * S(dist)
        d.ellipse((x - S(rr), y - S(rr), x + S(rr), y + S(rr)), fill=(56, 56, 58, 120))
    img = img.filter(ImageFilter.GaussianBlur(S(2)))
    finish(img, 64, 64, "smoke.png")


def spark():
    img = canvas(8, 8)
    d = ImageDraw.Draw(img)
    d.ellipse((0, 0, S(8), S(8)), fill=AMBER + (255,))
    d.ellipse((S(2), S(2), S(6), S(6)), fill=WHITE)
    img = glow(img, S(2), AMBER, 0.8)
    finish(img, 8, 8, "spark.png")


def ring(name, col):
    img = canvas(128, 128)
    d = ImageDraw.Draw(img)
    c = S(64)
    d.ellipse((c - S(50), c - S(50), c + S(50), c + S(50)), outline=col + (230,), width=S(3.0))
    d.ellipse((c - S(42), c - S(42), c + S(42), c + S(42)), outline=col + (100,), width=S(1.2))
    for k in range(4):
        a = k / 4 * math.tau
        x = c + math.cos(a) * S(50)
        y = c + math.sin(a) * S(50)
        glow_dot(d, (x, y), 2.4, col + (255,), core=WHITE)
    img = glow(img, S(4), col, 0.8)
    finish(img, 128, 128, name)


def beam():
    img = canvas(16, 128)
    d = ImageDraw.Draw(img)
    for x in range(S(16)):
        t = 1.0 - abs(x / S(16) - 0.5) * 2.0
        a = int(60 + 195 * t)
        d.line([(x, 0), (x, S(128))], fill=CYAN + (a,))
    d.line([(S(8), 0), (S(8), S(128))], fill=WHITE + (240,), width=S(1.0))
    img = glow(img, S(3), CYAN, 0.9)
    finish(img, 16, 128, "beam.png")


def shadow():
    img = canvas(96, 96)
    d = ImageDraw.Draw(img)
    d.ellipse((S(6), S(18), S(90), S(78)), fill=(0, 0, 0, 150))
    img = img.filter(ImageFilter.GaussianBlur(S(4)))
    finish(img, 96, 96, "shadow.png")


def decal_scorch():
    img = canvas(128, 128)
    d = ImageDraw.Draw(img)
    c = S(64)
    for k in range(20):
        a = random.uniform(0, math.tau)
        dist = random.uniform(0, 20)
        rr = random.uniform(12, 34)
        x = c + math.cos(a) * S(dist)
        y = c + math.sin(a) * S(dist)
        d.ellipse((x - S(rr), y - S(rr), x + S(rr), y + S(rr)), fill=(10, 10, 10, random.randint(30, 80)))
    for k in range(10):
        a = random.uniform(0, math.tau)
        dist = random.uniform(20, 50)
        x = c + math.cos(a) * S(dist)
        y = c + math.sin(a) * S(dist)
        d.ellipse((x - S(2), y - S(2), x + S(2), y + S(2)), fill=(20, 18, 16, 100))
    img = img.filter(ImageFilter.GaussianBlur(S(2)))
    finish(img, 128, 128, "decal_scorch.png")


def decal_creep():
    img = canvas(160, 160)
    d = ImageDraw.Draw(img)
    c = S(80)
    for k in range(24):
        a = random.uniform(0, math.tau)
        dist = random.uniform(0, 28)
        rr = random.uniform(20, 48)
        x = c + math.cos(a) * S(dist)
        y = c + math.sin(a) * S(dist)
        d.ellipse((x - S(rr), y - S(rr), x + S(rr), y + S(rr)), fill=PURPLE_1 + (random.randint(40, 90),))
    for k in range(14):
        a = random.uniform(0, math.tau)
        x0 = c + math.cos(a) * S(15)
        y0 = c + math.sin(a) * S(15)
        x1 = c + math.cos(a + 0.4) * S(random.uniform(45, 75))
        y1 = c + math.sin(a + 0.4) * S(random.uniform(45, 75))
        d.line([(x0, y0), ((x0 + x1) / 2 + random.uniform(-10, 10) * SS, (y0 + y1) / 2), (x1, y1)], fill=PURPLE_4 + (120,), width=S(1.2), joint="curve")
    img = img.filter(ImageFilter.GaussianBlur(S(2)))
    finish(img, 160, 160, "decal_creep.png")


# ---------------------------------------------------------------------------
# props
# ---------------------------------------------------------------------------
def crate(name, metal=True):
    img = canvas(56, 56)
    d = ImageDraw.Draw(img)
    base = STEEL_2 if metal else RUST_2
    bevel_rect(d, (S(5), S(6), S(51), S(51)), base, radius=S(3), width=S(1.2))
    d.rectangle((S(10), S(11), S(46), S(46)), outline=STEEL_0, width=S(1.0))
    d.line([(S(10), S(11)), (S(46), S(46))], fill=STEEL_0, width=S(1.6))
    d.line([(S(46), S(11)), (S(10), S(46))], fill=STEEL_0, width=S(1.6))
    if metal:
        d.rectangle((S(20), S(20), S(36), S(36)), fill=STEEL_1, outline=STEEL_4, width=S(0.8))
        glow_dot(d, (S(28), S(28)), 2.0, AMBER + (220,))
    else:
        for _ in range(10):
            x = random.uniform(10, 46)
            d.line([(S(x), S(12)), (S(x + random.uniform(-3, 3)), S(44))], fill=RUST_1 + (100,), width=S(0.8))
    draw_noise(img, 9, 70)
    finish(img, 56, 56, name)


def barrel(name, col):
    img = canvas(40, 40)
    d = ImageDraw.Draw(img)
    c = S(20)
    d.ellipse((c - S(17), c - S(17), c + S(17), c + S(17)), fill=(0, 0, 0, 80))
    d.ellipse((c - S(15), c - S(15), c + S(15), c + S(15)), fill=STEEL_1, outline=STEEL_0, width=S(1.0))
    d.ellipse((c - S(12), c - S(12), c + S(12), c + S(12)), fill=col, outline=mix(col, WHITE, 0.25), width=S(0.8))
    d.ellipse((c - S(7), c - S(7), c + S(7), c + S(7)), fill=STEEL_0, outline=STEEL_4, width=S(0.8))
    d.ellipse((c - S(3), c - S(3), c + S(3), c + S(3)), fill=STEEL_3)
    d.arc((c - S(15), c - S(15), c + S(15), c + S(15)), 200, 340, fill=mix(col, WHITE, 0.4), width=S(1.0))
    draw_noise(img, 7, 71)
    finish(img, 40, 40, name)


def pipe():
    img = canvas(128, 48)
    d = ImageDraw.Draw(img)
    for y in (S(10), S(28)):
        d.rounded_rectangle((0, y, S(128), y + S(10)), radius=S(4), fill=STEEL_1, outline=STEEL_0, width=S(0.7))
        d.line([(0, y + S(2)), (S(128), y + S(2))], fill=STEEL_4, width=S(0.8))
        for x in range(0, 128, 32):
            d.rectangle((S(x + 2), y - S(1), S(x + 6), y + S(11)), fill=STEEL_3, outline=STEEL_0, width=S(0.4))
    draw_noise(img, 8, 72)
    finish(img, 128, 48, "pipe.png")


def sandbag():
    img = canvas(72, 32)
    d = ImageDraw.Draw(img)
    for i in range(4):
        x = S(4 + i * 17)
        d.rounded_rectangle((x, S(6), x + S(20), S(26)), radius=S(6), fill=(96, 84, 58, 255), outline=(42, 38, 28, 255), width=S(0.8))
        d.arc((x, S(6), x + S(20), S(26)), 200, 340, fill=(150, 132, 92, 255), width=S(1.0))
    draw_noise(img, 8, 73)
    finish(img, 72, 32, "sandbag.png")


def rock():
    img = canvas(96, 96)
    d = ImageDraw.Draw(img)
    c = S(48)
    d.ellipse((S(8), S(50), S(88), S(84)), fill=(0, 0, 0, 100))
    pts = []
    for k in range(9):
        a = k / 9 * math.tau
        r = S(30) * (0.7 + random.random() * 0.4)
        pts.append((c + math.cos(a) * r * 1.05, c + math.sin(a) * r * 0.8))
    polygon(d, pts, (62, 58, 52, 255), outline=(24, 24, 24, 255), width=S(1.0))
    for _ in range(6):
        x = random.uniform(20, 76)
        y = random.uniform(30, 70)
        d.line([(S(x), S(y)), (S(x + random.uniform(-16, 16)), S(y + random.uniform(-10, 10)))], fill=(110, 102, 88, 130), width=S(0.8))
    draw_noise(img, 12, 74)
    finish(img, 96, 96, "rock.png")


# ---------------------------------------------------------------------------
# UI
# ---------------------------------------------------------------------------
def panel():
    img = canvas(64, 64)
    d = ImageDraw.Draw(img)
    bevel_rect(d, (0, 0, S(64), S(64)), (17, 23, 30, 245), radius=S(2), width=S(1.0))
    d.rectangle((S(3), S(3), S(61), S(61)), outline=(54, 68, 80, 255), width=S(0.7))
    d.line([(S(4), S(4)), (S(60), S(4))], fill=GOLD + (130,), width=S(0.7))
    for x in (S(7), S(57)):
        for y in (S(7), S(57)):
            glow_dot(d, (x, y), 1.0, STEEL_4)
    finish(img, 64, 64, "ui_panel.png")


def button(name, fill, border):
    img = canvas(64, 64)
    d = ImageDraw.Draw(img)
    bevel_rect(d, (S(1), S(1), S(63), S(63)), fill, radius=S(3), width=S(1.2))
    d.rectangle((S(5), S(5), S(59), S(59)), outline=border, width=S(0.8))
    d.line([(S(6), S(6)), (S(58), S(6))], fill=mix(fill, WHITE, 0.4), width=S(0.6))
    for x in (S(8), S(56)):
        for y in (S(8), S(56)):
            glow_dot(d, (x, y), 0.9, border)
    finish(img, 64, 64, name)


def icon_mineral():
    img = canvas(32, 32)
    d = ImageDraw.Draw(img)
    pts = [(S(16), S(3)), (S(28), S(11)), (S(25), S(27)), (S(7), S(27)), (S(4), S(11))]
    polygon(d, pts, (88, 160, 220, 255), outline=(20, 50, 80, 255), width=S(0.8))
    d.polygon([(S(16), S(3)), (S(28), S(11)), (S(16), S(16)), (S(4), S(11))], fill=(150, 220, 255, 255))
    d.line([(S(16), S(3)), (S(16), S(27))], fill=(40, 90, 130, 255), width=S(0.8))
    img = glow(img, S(2), CYAN, 0.5)
    finish(img, 32, 32, "icon_mineral.png")


def icon_gas():
    img = canvas(32, 32)
    d = ImageDraw.Draw(img)
    d.rectangle((S(9), S(15), S(23), S(29)), fill=STEEL_1, outline=STEEL_4, width=S(0.7))
    d.ellipse((S(11), S(3), S(21), S(17)), fill=(80, 200, 100, 220), outline=(30, 110, 50, 255), width=S(0.6))
    d.ellipse((S(14), S(6), S(18), S(12)), fill=(180, 255, 180, 230))
    img = glow(img, S(2), GREEN, 0.6)
    finish(img, 32, 32, "icon_gas.png")


def icon_tower(name, kind):
    img = canvas(40, 40)
    d = ImageDraw.Draw(img)
    c = S(20)
    if kind == "gauss":
        d.ellipse((c - S(15), c - S(15), c + S(15), c + S(15)), fill=STEEL_1, outline=STEEL_4, width=S(0.8))
        for ox in (-5, 5):
            d.rectangle((c + S(ox - 2), c - S(14), c + S(ox + 2), c + S(2)), fill=STEEL_4)
        glow_dot(d, (c, c + S(3)), 2.2, CYAN)
    elif kind == "missile":
        d.ellipse((c - S(15), c - S(15), c + S(15), c + S(15)), fill=STEEL_1, outline=STEEL_4, width=S(0.8))
        for i in range(2):
            for j in range(2):
                x = c - S(9) + i * S(9)
                y = c - S(11) + j * S(9)
                d.rectangle((x, y, x + S(7), y + S(7)), fill=STEEL_0, outline=AMBER, width=S(0.6))
        glow_dot(d, (c, c), 2.0, AMBER)
    else:
        d.ellipse((c - S(15), c - S(15), c + S(15), c + S(15)), fill=STEEL_1, outline=STEEL_4, width=S(0.8))
        for k in range(3):
            a = k / 3 * math.tau
            d.line([(c, c), (c + math.cos(a) * S(11), c + math.sin(a) * S(11))], fill=STEEL_4, width=S(2.0))
        glow_dot(d, (c, c), 5.0, CYAN, core=CYAN_L)
    finish(img, 40, 40, name)


def icon_relay():
    img = canvas(40, 40)
    d = ImageDraw.Draw(img)
    c = S(20)
    d.ellipse((c - S(15), c - S(15), c + S(15), c + S(15)), fill=STEEL_1, outline=STEEL_4, width=S(0.8))
    d.polygon([(c - S(6), c + S(10)), (c - S(3), c - S(8)), (c + S(3), c - S(8)), (c + S(6), c + S(10))], fill=STEEL_3, outline=STEEL_0)
    glow_dot(d, (c, c - S(8)), 4.0, CYAN, core=CYAN_L)
    finish(img, 40, 40, "icon_relay.png")


def icon_repair():
    img = canvas(40, 40)
    d = ImageDraw.Draw(img)
    c = S(20)
    d.ellipse((c - S(15), c - S(15), c + S(15), c + S(15)), fill=STEEL_1, outline=STEEL_4, width=S(0.8))
    d.rectangle((c - S(3), c - S(11), c + S(3), c + S(11)), fill=GREEN)
    d.rectangle((c - S(11), c - S(3), c + S(11), c + S(3)), fill=GREEN)
    img = glow(img, S(2), GREEN, 0.5)
    finish(img, 40, 40, "icon_repair.png")


def portrait():
    img = canvas(96, 96)
    d = ImageDraw.Draw(img)
    # background
    for y in range(S(96)):
        t = y / S(96)
        d.line([(0, y), (S(96), y)], fill=mix((20, 40, 54, 255), (8, 12, 18, 255), t))
    # frame
    d.rectangle((S(4), S(4), S(92), S(92)), outline=GOLD + (220,), width=S(1.2))
    d.rectangle((S(8), S(8), S(88), S(88)), outline=STEEL_4 + (160,), width=S(0.7))
    c = S(48)
    # goliath bust
    d.ellipse((c - S(25), c - S(18), c + S(25), c + S(32)), fill=STEEL_1, outline=STEEL_0, width=S(1.0))
    d.ellipse((c - S(19), c - S(13), c + S(19), c + S(24)), fill=STEEL_2)
    d.rounded_rectangle((c - S(14), c - S(10), c + S(14), c + S(2)), radius=S(4), fill=(6, 12, 16, 255), outline=CYAN_D, width=S(0.8))
    d.rounded_rectangle((c - S(11), c - S(8), c + S(11), c - S(2)), radius=S(2), fill=CYAN + (230,))
    for ox in (-18, 18):
        d.ellipse((c + S(ox) - S(9), c - S(6), c + S(ox) + S(9), c + S(18)), fill=STEEL_1, outline=STEEL_0, width=S(0.8))
        d.ellipse((c + S(ox) - S(6), c - S(3), c + S(ox) + S(6), c + S(15)), fill=STEEL_3)
    # scanner lines
    for y in range(S(12), S(86), S(4)):
        d.line([(S(10), y), (S(86), y)], fill=CYAN + (16,))
    finish(img, 96, 96, "portrait_goliath.png")


def minimap_bg():
    img = canvas(192, 128)
    d = ImageDraw.Draw(img)
    d.rectangle((0, 0, S(192), S(128)), fill=(8, 14, 19, 235), outline=STEEL_3, width=S(1.0))
    for x in range(0, 192, 16):
        d.line([(S(x), 0), (S(x), S(128))], fill=(40, 80, 96, 50), width=S(0.4))
    for y in range(0, 128, 16):
        d.line([(0, S(y)), (S(192), S(y))], fill=(40, 80, 96, 50), width=S(0.4))
    d.rectangle((S(2), S(2), S(190), S(126)), outline=CYAN_D + (150,), width=S(0.5))
    finish(img, 192, 128, "ui_minimap.png")


def main():
    random.seed(2126)
    terrain_metal("tile_metal_1.png", 1, rust=0.1)
    terrain_metal("tile_metal_2.png", 2, rust=0.25)
    terrain_metal("tile_metal_3.png", 3, rust=0.18, hazard=True)
    terrain_grate()
    terrain_metal("tile_rust_1.png", 4, rust=0.7, base=RUST_1)
    terrain_rock("tile_rock_1.png", 5)
    terrain_rock("tile_rock_2.png", 6)
    terrain_creep("tile_creep_1.png", 8)
    terrain_creep("tile_creep_2.png", 9)
    base_pad()

    command_center()
    tower_base("tower_gauss_base.png", CYAN)
    gauss_head()
    tower_base("tower_missile_base.png", AMBER)
    missile_head()
    tower_base("tower_arc_base.png", CYAN)
    arc_head()
    relay()

    goliath_legs(0)
    goliath_legs(1)
    goliath_torso()
    zergling()
    hydralisk()
    ultralisk()

    bullet("bullet_gauss.png", 8, 24, CYAN)
    bullet("bullet_missile.png", 14, 30, AMBER, core=(255, 240, 200, 255))
    muzzle_flash()
    tracer()
    explosion_frames()
    smoke()
    spark()
    ring("ring_cyan.png", CYAN)
    ring("selection_ring.png", GREEN)
    beam()
    shadow()
    decal_scorch()
    decal_creep()

    crate("crate_metal.png", True)
    crate("crate_wood.png", False)
    barrel("barrel_green.png", (70, 120, 70, 255))
    barrel("barrel_red.png", (150, 52, 44, 255))
    pipe()
    sandbag()
    rock()

    panel()
    button("ui_button.png", (27, 38, 47, 245), (74, 94, 108, 255))
    button("ui_button_hover.png", (43, 70, 78, 250), GOLD + (255,))
    icon_mineral()
    icon_gas()
    icon_tower("icon_gauss.png", "gauss")
    icon_tower("icon_missile.png", "missile")
    icon_tower("icon_arc.png", "arc")
    icon_relay()
    icon_repair()
    portrait()
    minimap_bg()

    print("generated assets in", OUT)


if __name__ == "__main__":
    main()
