#!/usr/bin/env python3
"""Cloud detail metric: the references in reference/clouds/ against megamap renders.

Every sample is a grey crop scaled to a 300 px short side, contrast-normalised (divided by its own standard
deviation). Two numbers:
  hf   - high-frequency energy ratio: energy of I - blur(I, 1.5 px) over energy of I - blur(I, 15 px). The share
         of the fine band (detail a few pixels wide) in everything finer than the cloud masses.
  g90  - 90th percentile of the gradient magnitude / its median: how heavy-tailed the edges are (crisp billow
         rims and grain against smooth gradients).
Usage: cloud_detail.py RENDER_DIR [RENDER_DIR...]   (dirs of PPMs from `cargo run --example detail`)
"""
import sys, os, math
from PIL import Image, ImageFilter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REFS = [
    ("ref1 sky", "cloud-ref-1.png", (0, 0, 1090, 300)),
    ("ref2 a", "cloud-ref-2.png", (0, 0, 800, 320)),
    ("ref2 b", "cloud-ref-2.png", (800, 0, 1600, 320)),
    ("ref2 c", "cloud-ref-2.png", (0, 435, 800, 740)),
    ("ref2 d", "cloud-ref-2.png", (800, 435, 1600, 740)),
    ("ref3", "cloud-ref-3.png", (300, 40, 1750, 1000)),
]
SIDE = 300


def prep(img, box=None):
    g = img.convert("L")
    if box:
        g = g.crop(box)
    s = SIDE / min(g.size)
    return g.resize((round(g.size[0] * s), round(g.size[1] * s)), Image.LANCZOS)


def energy(a, b):
    pa, pb = a.tobytes(), b.tobytes()
    return sum((x - y) ** 2 for x, y in zip(pa, pb)) / len(pa)


def stats(g):
    px = g.tobytes()
    n = len(px)
    mean = sum(px) / n
    sd = math.sqrt(sum((p - mean) ** 2 for p in px) / n) or 1.0
    fine = energy(g, g.filter(ImageFilter.GaussianBlur(1.5)))
    coarse = energy(g, g.filter(ImageFilter.GaussianBlur(15)))
    w, h = g.size
    mags = []
    for y in range(1, h - 1):
        r0, r1, r2 = px[(y - 1) * w:y * w], px[y * w:(y + 1) * w], px[(y + 1) * w:(y + 2) * w]
        for x in range(1, w - 1, 2):
            gx = r1[x + 1] - r1[x - 1]
            gy = r2[x] - r0[x]
            mags.append(math.hypot(gx, gy) / sd)
    mags.sort()
    med = mags[len(mags) // 2] or 1e-3
    return fine / max(coarse, 1e-9), mags[int(len(mags) * 0.9)] / med, mags[len(mags) // 2]


def main():
    rows = []
    for name, f, box in REFS:
        rows.append((name, *stats(prep(Image.open(os.path.join(ROOT, "reference/clouds", f)), box))))
    ref_hf = sum(r[1] for r in rows) / len(rows)
    ref_g = sum(r[2] for r in rows) / len(rows)
    for name, hf, g90, med in rows:
        print(f"{name:14s} hf {hf:.4f}  g90/med {g90:.2f}  med|g|/sd {med:.3f}")
    print(f"{'ref mean':14s} hf {ref_hf:.4f}  g90/med {ref_g:.2f}")
    for d in sys.argv[1:]:
        for f in sorted(os.listdir(d)):
            if f.endswith(".ppm"):
                hf, g90, med = stats(prep(Image.open(os.path.join(d, f))))
                print(f"{os.path.basename(d.rstrip('/'))}/{f:12s} hf {hf:.4f} ({hf / ref_hf:4.2f}x ref)  g90/med {g90:.2f}  med|g|/sd {med:.3f}")


if __name__ == "__main__":
    main()
