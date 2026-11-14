#!/usr/bin/env python3
"""Generates megamap's GUI art procedurally (no image editor, no third-party art).

Writes PNGs to assets/megamap/textures/gui/ with only the standard library (zlib + struct):
  wp_<icon>.png   16x16 white glyphs with a dark outline, coloured by the waypoint diamond drawn under them
  frame_round.png 128x128 ring for the round minimap
  arrow.png       32x32 player arrow
  compass.png     32x32 north rose
  icon.png        64x64 mod icon
  tb_<button>.png 16x16 world-map toolbar icons (white, dark outline, so they read on dark and light panels)

Glyphs are signed-distance shapes sampled 4x4 per pixel, so edges are antialiased. Run from anywhere:
  python3 tools/gen_art.py
"""

import math
import os
import struct
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "assets", "megamap", "textures", "gui")


def png(path, w, h, px):
    """px: list of (r, g, b, a) rows-major."""
    raw = bytearray()
    for y in range(h):
        raw.append(0)
        for x in range(w):
            raw.extend(px[y * w + x])

    def chunk(tag, data):
        c = struct.pack(">I", len(data)) + tag + data
        return c + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    data = b"\x89PNG\r\n\x1a\n"
    data += chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
    data += chunk(b"IDAT", zlib.compress(bytes(raw), 9))
    data += chunk(b"IEND", b"")
    with open(path, "wb") as f:
        f.write(data)


# ---- signed distance helpers, in a -1..1 square, negative inside ----

def sd_circle(x, y, cx, cy, r):
    return math.hypot(x - cx, y - cy) - r


def sd_box(x, y, cx, cy, hw, hh):
    dx, dy = abs(x - cx) - hw, abs(y - cy) - hh
    return math.hypot(max(dx, 0), max(dy, 0)) + min(max(dx, dy), 0)


def sd_poly(x, y, pts):
    d = float("inf")
    inside = False
    n = len(pts)
    for i in range(n):
        ax, ay = pts[i]
        bx, by = pts[(i + 1) % n]
        ex, ey = bx - ax, by - ay
        wx, wy = x - ax, y - ay
        t = max(0.0, min(1.0, (wx * ex + wy * ey) / (ex * ex + ey * ey)))
        d = min(d, math.hypot(wx - ex * t, wy - ey * t))
        if (ay > y) != (by > y) and x < ax + (y - ay) * ex / ey:
            inside = not inside
    return -d if inside else d


def star(n, r_out, r_in, rot=-math.pi / 2):
    pts = []
    for i in range(n * 2):
        r = r_out if i % 2 == 0 else r_in
        a = rot + i * math.pi / n
        pts.append((math.cos(a) * r, math.sin(a) * r))
    return pts


GLYPHS = {
    "diamond": lambda x, y: sd_poly(x, y, [(0, -0.8), (0.6, 0), (0, 0.8), (-0.6, 0)]),
    "star": lambda x, y: sd_poly(x, y, star(5, 0.85, 0.36)),
    "home": lambda x, y: min(
        sd_poly(x, y, [(0, -0.85), (0.8, -0.1), (-0.8, -0.1)]),
        max(sd_box(x, y, 0, 0.35, 0.55, 0.45), -sd_box(x, y, 0, 0.5, 0.17, 0.32)),
    ),
    "skull": lambda x, y: max(
        min(sd_circle(x, y, 0, -0.15, 0.62), sd_box(x, y, 0, 0.5, 0.32, 0.25)),
        -min(sd_circle(x, y, -0.25, -0.15, 0.17), sd_circle(x, y, 0.25, -0.15, 0.17)),
    ),
    "flag": lambda x, y: min(
        sd_box(x, y, -0.55, 0, 0.09, 0.85),
        sd_poly(x, y, [(-0.46, -0.8), (0.8, -0.45), (-0.46, -0.1)]),
    ),
    "cross": lambda x, y: min(sd_box(x, y, 0, 0, 0.18, 0.8), sd_box(x, y, 0, -0.2, 0.6, 0.17)),
}


def ring(x, y, cx, cy, r, w):
    return abs(sd_circle(x, y, cx, cy, r)) - w


def seg(x, y, ax, ay, bx, by, w):
    ex, ey = bx - ax, by - ay
    t = max(0.0, min(1.0, ((x - ax) * ex + (y - ay) * ey) / (ex * ex + ey * ey)))
    return math.hypot(x - ax - ex * t, y - ay - ey * t) - w


def u(*ds):
    return min(ds)


TOOLBAR = {
    # Magnifier with + / -.
    "zoom_in": lambda x, y: u(
        ring(x, y, -0.15, -0.15, 0.5, 0.11),
        seg(x, y, 0.22, 0.22, 0.75, 0.75, 0.14),
        sd_box(x, y, -0.15, -0.15, 0.27, 0.07),
        sd_box(x, y, -0.15, -0.15, 0.07, 0.27),
    ),
    "zoom_out": lambda x, y: u(
        ring(x, y, -0.15, -0.15, 0.5, 0.11),
        seg(x, y, 0.22, 0.22, 0.75, 0.75, 0.14),
        sd_box(x, y, -0.15, -0.15, 0.27, 0.07),
    ),
    # Crosshair on the player.
    "center": lambda x, y: u(
        ring(x, y, 0, 0, 0.5, 0.09),
        sd_circle(x, y, 0, 0, 0.17),
        sd_box(x, y, 0, -0.68, 0.07, 0.2),
        sd_box(x, y, 0, 0.68, 0.07, 0.2),
        sd_box(x, y, -0.68, 0, 0.2, 0.07),
        sd_box(x, y, 0.68, 0, 0.2, 0.07),
    ),
    # Bulleted list.
    "list": lambda x, y: u(*[
        d for row in (-0.5, 0.0, 0.5)
        for d in (sd_circle(x, y, -0.6, row, 0.12), sd_box(x, y, 0.15, row, 0.55, 0.08))
    ]),
    # Cave arch.
    "cave": lambda x, y: max(
        sd_box(x, y, 0, 0.2, 0.8, 0.6),
        -u(sd_circle(x, y, 0, 0.25, 0.42), sd_box(x, y, 0, 0.6, 0.42, 0.35)),
        sd_circle(x, y, 0, 0.6, 1.05),
    ),
    # Dotted footsteps path.
    "trail": lambda x, y: u(
        sd_circle(x, y, -0.6, 0.6, 0.14), sd_circle(x, y, -0.25, 0.3, 0.13),
        sd_circle(x, y, 0.05, -0.05, 0.12), sd_circle(x, y, 0.3, -0.35, 0.12),
        sd_poly(x, y, [(0.35, -0.85), (0.85, -0.6), (0.45, -0.4)]),
    ),
    # Cumulus.
    "clouds": lambda x, y: u(
        sd_circle(x, y, -0.4, 0.15, 0.32), sd_circle(x, y, 0.05, -0.12, 0.45),
        sd_circle(x, y, 0.45, 0.15, 0.33), sd_box(x, y, 0.02, 0.3, 0.7, 0.2),
    ),
    # Two mountain peaks.
    "relief": lambda x, y: u(
        sd_poly(x, y, [(-0.85, 0.65), (-0.25, -0.55), (0.25, 0.65)]),
        sd_poly(x, y, [(-0.1, 0.65), (0.4, -0.2), (0.88, 0.65)]),
    ),
    # 3x3 grid.
    "grid": lambda x, y: u(
        sd_box(x, y, -0.72, 0, 0.07, 0.79),
        sd_box(x, y, -0.24, 0, 0.07, 0.79), sd_box(x, y, 0.24, 0, 0.07, 0.79), sd_box(x, y, 0.72, 0, 0.07, 0.79),
        sd_box(x, y, 0, -0.72, 0.79, 0.07), sd_box(x, y, 0, -0.24, 0.79, 0.07),
        sd_box(x, y, 0, 0.24, 0.79, 0.07), sd_box(x, y, 0, 0.72, 0.79, 0.07),
    ),
}


def raster(w, h, shade, ss=4):
    """shade(x, y) -> (r, g, b, a) floats 0..1 with premultiplication done here per subsample."""
    out = []
    for py in range(h):
        for px in range(w):
            acc = [0.0, 0.0, 0.0, 0.0]
            for sy in range(ss):
                for sx in range(ss):
                    x = ((px + (sx + 0.5) / ss) / w) * 2 - 1
                    y = ((py + (sy + 0.5) / ss) / h) * 2 - 1
                    r, g, b, a = shade(x, y)
                    acc[0] += r * a
                    acc[1] += g * a
                    acc[2] += b * a
                    acc[3] += a
            n = ss * ss
            a = acc[3] / n
            if a <= 0:
                out.append((0, 0, 0, 0))
            else:
                out.append(tuple(int(round(min(1, c / acc[3]) * 255)) for c in acc[:3]) + (int(round(a * 255)),))
    return out


def glyph(sd, outline=0.16):
    def shade(x, y):
        d = sd(x, y)
        if d < 0:
            return (1, 1, 1, 1)
        if d < outline:
            return (0.08, 0.08, 0.1, 0.9)
        return (0, 0, 0, 0)

    return shade


def frame_round(x, y):
    r = math.hypot(x, y)
    # 128 px: the map sits inside r < 0.955. A bevelled rim (lit from the top left), a dark edge, a soft inner
    # shadow over the map and a faint drop shadow outside.
    if 0.955 <= r < 0.985:
        light = 0.5 + 0.5 * (-(x + y) / (r * math.sqrt(2)))
        v = 0.62 + 0.33 * light
        return (v, v + 0.01, v + 0.04, 1)
    if 0.985 <= r <= 1.0:
        return (0.05, 0.05, 0.07, 0.95)
    if 0.90 <= r < 0.955:
        return (0.0, 0.0, 0.02, 0.55 * (r - 0.90) / 0.055)
    return (0, 0, 0, 0)


ARROW = [(0, -0.9), (0.62, 0.75), (0, 0.38), (-0.62, 0.75)]


def arrow(x, y):
    d = sd_poly(x, y, ARROW)
    if d < 0:
        return (1, 1, 1, 1) if x < 0.0 else (0.82, 0.85, 0.9, 1)
    if d < 0.12:
        return (0, 0, 0, 1)
    return (0, 0, 0, 0)


def compass(x, y):
    # A four-point rose: red north, white south, grey east / west, a ring and a hub.
    n = sd_poly(x, y, [(0, -0.95), (0.22, -0.05), (-0.22, -0.05)])
    s_ = sd_poly(x, y, [(0, 0.95), (0.22, 0.05), (-0.22, 0.05)])
    ew = min(sd_poly(x, y, [(-0.8, 0), (0, -0.16), (0, 0.16)]), sd_poly(x, y, [(0.8, 0), (0, -0.16), (0, 0.16)]))
    rim = ring(x, y, 0, 0, 0.62, 0.035)
    if sd_circle(x, y, 0, 0, 0.1) < 0:
        return (0.15, 0.15, 0.18, 1)
    if n < 0:
        return (0.92, 0.25, 0.22, 1) if x < 0 else (0.72, 0.15, 0.13, 1)
    if s_ < 0:
        return (0.95, 0.95, 0.97, 1) if x > 0 else (0.78, 0.8, 0.84, 1)
    if ew < 0:
        return (0.62, 0.65, 0.7, 1)
    if min(n, s_, ew) < 0.08 or rim < 0:
        return (0.05, 0.05, 0.07, 0.9)
    return (0, 0, 0, 0)


def icon(x, y):
    # A rounded tile of terrain bands, a ring and an arrow: the mod at a glance.
    box = sd_box(x, y, 0, 0, 0.82, 0.82) - 0.12
    if box > 0:
        return (0, 0, 0, 0)
    if box > -0.06:
        return (0.07, 0.08, 0.1, 1)
    a = arrow(x * 1.8, y * 1.8 + 0.05)
    if a[3] > 0:
        return a
    h = math.sin(x * 3.1 + 0.6) * 0.25 + math.cos(y * 4.3 - x * 1.7) * 0.2 + y * 0.4
    if h < -0.2:
        return (0.25, 0.46, 0.85, 1)
    if h < -0.1:
        return (0.85, 0.8, 0.55, 1)
    if h < 0.35:
        return (0.42, 0.66, 0.3, 1)
    return (0.33, 0.5, 0.24, 1)


# ---- line icons, emitted as const bitmaps into src/engine/icons.rs ----
# The RT .hnmod carries no PNGs, so the toolbar / settings icons live in code: one u16 bitmask per row. They are
# hand-placed pixel art on the 16 px grid (1 px lines, 2 px where a stroke must survive GUI scale 1): an SDF
# thresholded to 1 bit gives ragged 1/2 px strokes at this size.

LINE = {
"zoom_in": """
................
....#####.......
...#.....#......
..#.......#.....
..#...#...#.....
..#...#...#.....
..#.#####.#.....
..#...#...#.....
..#...#...#.....
...#.....##.....
....#####.##....
..........###...
...........###..
............###.
.............##.
................""",
"zoom_out": """
................
....#####.......
...#.....#......
..#.......#.....
..#.......#.....
..#.......#.....
..#.#####.#.....
..#.......#.....
..#.......#.....
...#.....##.....
....#####.##....
..........###...
...........###..
............###.
.............##.
................""",
"center": """
.......#........
.......#........
.....#####......
....#..#..#.....
...#.......#....
..#.........#...
..#....#....#...
######.#.######.
..#....#....#...
..#.........#...
...#.......#....
....#..#..#.....
.....#####......
.......#........
.......#........
................""",
"list": """
................
................
.##.##########..
.##.............
................
................
................
.##.##########..
.##.............
................
................
................
.##.##########..
.##.............
................
................""",
"cave": """
................
................
......####......
....##....##....
...#........#...
..#..........#..
..#..........#..
.#............#.
.#....####....#.
.#...#....#...#.
.#..#......#..#.
.#..#......#..#.
.#..#......#..#.
################
................
................""",
"trail": """
...........####.
...........#..#.
...........#..#.
...........####.
............#...
................
.........##.....
.........##.....
................
.....##.........
.....##.........
................
.##.............
.##.............
................
................""",
"clouds": """
................
................
................
......####......
.....#....#.....
....#......#....
..###.......##..
.#............#.
#..............#
#..............#
.#............#.
..############..
................
................
................
................""",
"relief": """
................
................
................
.....#..........
....#.#.........
....#.#.........
...#...#..#.....
...#...#.#.#....
..#.....#...#...
..#.........#...
.#...........#..
.#...........#..
#.............#.
################
................
................""",
"grid": """
................
.##############.
.#....#....#..#.
.#....#....#..#.
.#....#....#..#.
.#....#....#..#.
.##############.
.#....#....#..#.
.#....#....#..#.
.#....#....#..#.
.#....#....#..#.
.##############.
.#....#....#..#.
.#....#....#..#.
.##############.
................""",
"settings": """
................
.......##.......
...##..##..##...
...###....###...
....#......#....
...#...##...#...
.##...#..#...##.
.##..#....#..##.
.....#....#.....
.##...#..#...##.
...#...##...#...
....#......#....
...###....###...
...##..##..##...
.......##.......
................""",
"entities": """
................
......####......
.....#....#.....
.....#....#.....
.....#....#.....
.....#....#.....
......####......
................
....########....
...#........#...
..#..........#..
..#..........#..
..#..........#..
..############..
................
................""",
"dimension": """
................
...##########...
...#........#...
...#.######.#...
...#.#....#.#...
...#.#..#.#.#...
...#.#.#..#.#...
...#.#..#.#.#...
...#.#.#..#.#...
...#.#..#.#.#...
...#.#.#..#.#...
...#.#....#.#...
...#.######.#...
...#........#...
...##########...
................""",
"eye": """
................
................
................
................
.....######.....
...##......##...
..#....##....#..
.#....####....#.
.#....####....#.
..#....##....#..
...##......##...
.....######.....
................
................
................
................""",
"info": """
................
.....######.....
....#......#....
...#...##...#...
..#....##....#..
.#............#.
.#.....##.....#.
.#.....##.....#.
.#.....##.....#.
.#.....##.....#.
..#....##....#..
...#........#...
....#......#....
.....######.....
................
................""",
"autozoom": """
###..........###
#..............#
#.....####.....#
.....#....#.....
....#......#....
....#......#....
....#......#....
....#......#....
.....#....##....
......####.##...
............##..
.............##.
#..............#
#..............#
###..........###
................""",
"perf": """
................
................
................
.....######.....
....#......#....
...#........#...
..#.......#..#..
.#.......#....#.
.#......#.....#.
.#.....##.....#.
.#.....##.....#.
.#............#.
.##############.
................
................
................""",
"flag": """
................
..##............
..#####.........
..#....####.....
..#........###..
..#.......##....
..#.....##......
..######........
..#.............
..#.............
..#.............
..#.............
..#.............
..#.............
.###............
................""",
"data": """
................
..############..
..#..........#..
..#.##.......#..
..############..
................
..############..
..#..........#..
..#.##.......#..
..############..
................
..############..
..#..........#..
..#.##.......#..
..############..
................""",
"window": """
................
................
.##############.
.#.##.........#.
.##############.
.#............#.
.#............#.
.#............#.
.#............#.
.#............#.
.#............#.
.#............#.
.#............#.
.##############.
................
................""",
"reset": """
................
................
...#.####.......
...##....##.....
...###.....#....
...####.....#...
...........#....
.............#..
.#...........#..
.#...........#..
.#...........#..
..#.........#...
...#.......#....
....##...##.....
......###.......
................""",
"search": """
................
................
.....####.......
....#....#......
...#......#.....
...#......#.....
...#......#.....
...#......#.....
....#....##.....
.....####.##....
..........###...
...........###..
............##..
................
................
................""",
}


def icon_rows(art):
    lines = [l for l in art.split("\n") if l]
    assert len(lines) == 16 and all(len(l) == 16 for l in lines), art
    return [sum(1 << (15 - i) for i, c in enumerate(l) if c == "#") for l in lines]


def write_icons(path):
    out = ["//! Generated by tools/gen_art.py: monochrome 16x16 line icons, one u16 per row (MSB = left column).",
           "//! Drawn with run-length fills in the button's icon colour, so they need no textures in the .hnmod.", ""]
    for name, art in LINE.items():
        rows = icon_rows(art)
        out.append(f"pub const {name.upper()}: [u16; 16] = [")
        out.append("    " + ", ".join(f"0x{r:04X}" for r in rows[:8]) + ",")
        out.append("    " + ", ".join(f"0x{r:04X}" for r in rows[8:]) + ",")
        out.append("];")
    out.append("")
    out.append("/// Toolbar icons in `ui::BUTTONS` order (+ - P W C T F R G S E D).")
    out.append("pub const TOOLBAR: [&[u16; 16]; 12] =")
    out.append("    [&ZOOM_IN, &ZOOM_OUT, &CENTER, &LIST, &CAVE, &TRAIL, &CLOUDS, &RELIEF, &GRID, &SETTINGS, &ENTITIES, &DIMENSION];")
    out.append("")
    out.append("""/// Horizontal runs `(x, y, len)` of the set pixels, for fill-based drawing.
pub fn runs(icon: &[u16; 16], mut f: impl FnMut(u32, u32, u32)) {
    for (y, &row) in icon.iter().enumerate() {
        let mut x = 0u32;
        while x < 16 {
            if row & (0x8000 >> x) != 0 {
                let s = x;
                while x < 16 && row & (0x8000 >> x) != 0 {
                    x += 1;
                }
                f(s, y as u32, x - s);
            } else {
                x += 1;
            }
        }
    }
}

/// Number of set pixels (tests: every icon is visible and not a filled blob).
pub fn ink(icon: &[u16; 16]) -> u32 {
    icon.iter().map(|r| r.count_ones()).sum()
}
""")
    with open(path, "w") as f:
        f.write("\n".join(out))


def main():
    os.makedirs(OUT, exist_ok=True)
    for name, sd in GLYPHS.items():
        png(os.path.join(OUT, f"wp_{name}.png"), 16, 16, raster(16, 16, glyph(sd)))
    for name, sd in TOOLBAR.items():
        png(os.path.join(OUT, f"tb_{name}.png"), 16, 16, raster(16, 16, glyph(sd, 0.18)))
    png(os.path.join(OUT, "frame_round.png"), 128, 128, raster(128, 128, frame_round, 2))
    png(os.path.join(OUT, "arrow.png"), 32, 32, raster(32, 32, arrow))
    png(os.path.join(OUT, "compass.png"), 32, 32, raster(32, 32, compass))
    png(os.path.join(OUT, "icon.png"), 64, 64, raster(64, 64, icon))
    write_icons(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "src", "engine", "icons.rs"))
    print("wrote", len(GLYPHS) + len(TOOLBAR) + 4, "images to", OUT)


if __name__ == "__main__":
    main()
