"""Subset Lora (SIL OFL 1.1) to Latin + Vietnamese glyphs, output WOFF2, and RENAME the family.

OFL 1.1 §3: a Modified Version must not use the Reserved Font Name ("Lora"). Subsetting is a
modification, so the name table of the output is rewritten to "LichViet Serif" (RS-20) and every
"Lora" string is removed from nameIDs 1/2/3/4/6/16/17/21/22 (family, style, unique ID, full name,
PostScript name, typographic family/subfamily, WWS). Copyright (0), licence (13, 14) are kept.

Run from pwa/:  python3 scripts/subset-font.py
Requires fontTools + brotli. Output is committed in public/fonts/ so the build itself does not need
Python. The script then re-opens each output and fails if "Lora" still appears in a name record.
"""
import os
import sys
from fontTools import subset
from fontTools.ttLib import TTFont

FAMILY = "LichViet Serif"
PS_FAMILY = "LichVietSerif"
SRC = [
    ("/usr/share/fonts/truetype/google-fonts/Lora-Variable.ttf", "public/fonts/lichviet-serif.woff2", "Regular", False),
    ("/usr/share/fonts/truetype/google-fonts/Lora-Italic-Variable.ttf", "public/fonts/lichviet-serif-italic.woff2", "Italic", True),
]
OLD = ["public/fonts/lora-vi.woff2", "public/fonts/lora-vi-italic.woff2"]
ranges = [(0x20, 0x7E), (0xA0, 0xFF), (0x102, 0x103), (0x110, 0x111), (0x128, 0x129),
          (0x168, 0x169), (0x1A0, 0x1A1), (0x1AF, 0x1B0), (0x300, 0x301), (0x303, 0x303),
          (0x309, 0x309), (0x323, 0x323), (0x1EA0, 0x1EF9), (0x2013, 0x2014), (0x2018, 0x201D),
          (0x2026, 0x2026), (0x20AB, 0x20AB)]
unicodes = [c for a, b in ranges for c in range(a, b + 1)]
KEEP_IDS = [0, 1, 2, 3, 4, 5, 6, 13, 14]


def rename(font: TTFont, style: str) -> None:
    name = font["name"]
    version = (name.getDebugName(5) or "Version 1.0").split(";")[0]
    new = {
        1: FAMILY,
        2: style,
        3: f"{version};CLEVIX;{PS_FAMILY}-{style}",
        4: f"{FAMILY} {style}" if style != "Regular" else FAMILY,
        6: f"{PS_FAMILY}-{style}",
    }
    for rec in list(name.names):
        if rec.nameID in (16, 17, 21, 22, 25):
            name.removeNames(nameID=rec.nameID)
    for nid, val in new.items():
        name.removeNames(nameID=nid)
        name.setName(val, nid, 3, 1, 0x409)  # Windows, Unicode BMP, en-US
        name.setName(val, nid, 1, 0, 0)      # Mac Roman, English
    # Named instances / STAT of the variable font reference family names via nameIDs ≥ 256;
    # rewrite any record that still mentions the reserved name.
    for rec in name.names:
        s = rec.toUnicode()
        if "Lora" in s and rec.nameID not in (0, 13, 14):
            rec.string = s.replace("Lora", FAMILY)
    if "CFF " in font:
        font["CFF "].cff.fontNames = [f"{PS_FAMILY}-{style}"]


def check(path: str) -> None:
    f = TTFont(path)
    bad = [(r.nameID, r.toUnicode()) for r in f["name"].names
           if "Lora" in r.toUnicode() and r.nameID not in (0, 13, 14)]
    if bad:
        sys.exit(f"{path}: Reserved Font Name still present in name table: {bad}")
    fam = f["name"].getDebugName(1)
    if fam != FAMILY:
        sys.exit(f"{path}: family is {fam!r}, expected {FAMILY!r}")


for src, out, style, _italic in SRC:
    opts = subset.Options()
    opts.flavor = "woff2"
    opts.layout_features = ["kern", "liga", "lnum", "tnum", "pnum", "ccmp", "mark", "mkmk", "locl"]
    opts.name_IDs = ["*"]  # keep all, then rewrite (named instances use IDs ≥ 256)
    opts.notdef_outline = True
    font = subset.load_font(src, opts)
    s = subset.Subsetter(opts)
    s.populate(unicodes=unicodes)
    s.subset(font)
    rename(font, style)
    os.makedirs(os.path.dirname(out), exist_ok=True)
    subset.save_font(font, out, opts)
    check(out)
    print(out, os.path.getsize(out), "bytes, family =", FAMILY)

for p in OLD:
    if os.path.exists(p):
        os.remove(p)
        print("removed", p)
