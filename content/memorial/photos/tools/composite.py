"""Step 2: cut out the real photos, export WebP 1x/2x/3x within budget, and build one preview PNG.

Only editing operations are used: crop, background removal (rembg segmentation of the real photo), resize,
colour balance. Nothing is generated or painted in. The backdrop of the preview is the memorial screen's CSS
gradient (UI colour), and contact shadows are blurred ellipses (compositing), as allowed by docs/v0.2/09 §3.3.
"""
import io
import json

from PIL import Image, ImageDraw, ImageEnhance, ImageFilter

from common import ROOT, load

CUT = ROOT / "cutouts"
WEBP = ROOT / "webp"
PREV = ROOT / "preview"
for d in (CUT, WEBP, PREV):
    d.mkdir(exist_ok=True)

BUDGET = 600 * 1024
CANVAS = (360, 270)


def cutout(ledger_entry, layer, session):
    from rembg import remove  # imported lazily: heavy

    src = Image.open(ROOT / "originals" / ledger_entry["file"]).convert("RGB")
    if layer.get("crop"):  # [x0, y0, x1, y1] as fractions of the source image
        w, h = src.size
        x0, y0, x1, y1 = layer["crop"]
        src = src.crop((int(x0 * w), int(y0 * h), int(x1 * w), int(y1 * h)))
    if layer.get("warmth") or layer.get("brightness"):
        src = ImageEnhance.Brightness(src).enhance(layer.get("brightness", 1.0))
        r, g, b = src.split()
        k = layer.get("warmth", 0.0)
        r = r.point(lambda v: min(255, int(v * (1 + k))))
        b = b.point(lambda v: int(v * (1 - k)))
        src = Image.merge("RGB", (r, g, b))
    rgba = remove(src, session=session, post_process_mask=True)
    bbox = rgba.getchannel("A").point(lambda a: 255 if a > 16 else 0).getbbox()
    if bbox:
        rgba = rgba.crop(bbox)
    return rgba


def export(layer_id, rgba, width_css, quality):
    sizes = {}
    for k in (1, 2, 3):
        w = max(1, round(width_css * k))
        h = max(1, round(rgba.height * w / rgba.width))
        im = rgba.resize((w, h), Image.LANCZOS)
        p = WEBP / f"mam-tt-{layer_id}@{k}x.webp"
        im.save(p, "WEBP", quality=quality, alpha_quality=90, method=6)
        sizes[k] = p.stat().st_size
    return sizes


def gradient_bg(size):
    w, h = size
    bg = Image.new("RGB", size, "#1d0f0c")
    d = ImageDraw.Draw(bg)
    stops = [(0.0, (0x5a, 0x2a, 0x1e)), (0.5, (0x3a, 0x1a, 0x14)), (1.0, (0x1d, 0x0f, 0x0c))]
    cx, cy, R = w / 2, h * 0.1, max(w, h) * 0.95
    for r in range(int(R), 0, -4):
        t = r / R
        for (t0, c0), (t1, c1) in zip(stops, stops[1:]):
            if t0 <= t <= t1:
                f = (t - t0) / (t1 - t0)
                col = tuple(int(c0[i] + (c1[i] - c0[i]) * f) for i in range(3))
        d.ellipse((cx - r, cy - r, cx + r, cy + r), fill=col)
    return bg


def main():
    from rembg import new_session

    ledger = {e["id"]: e for e in load("ledger.json")["photos"]}
    layout = load("layout.json")
    session = new_session("isnet-general-use")
    cut = {}
    for layer in layout["layers"]:
        e = ledger[layer["photo"]]
        rgba = cutout(e, layer, session)
        rgba.save(CUT / f"{layer['id']}.png")
        cut[layer["id"]] = rgba

    quality = 80
    while True:
        report = {lid: export(lid, cut[lid], next(l["width"] for l in layout["layers"] if l["id"] == lid), quality)
                  for lid in cut}
        total = sum(sum(s.values()) for s in report.values())
        if total <= BUDGET or quality <= 40:
            break
        quality -= 5
    (ROOT / "webp" / "budget.json").write_text(json.dumps(
        {"quality": quality, "total_bytes": total, "budget_bytes": BUDGET, "per_layer": report},
        indent=2) + "\n", encoding="utf-8")
    print("total", total, "quality", quality)

    S = 3
    W, H = CANVAS[0] * S, CANVAS[1] * S
    bg = gradient_bg((W, H)).convert("RGBA")
    for layer in sorted(layout["layers"], key=lambda l: l["z"]):
        im = Image.open(WEBP / f"mam-tt-{layer['id']}@3x.webp").convert("RGBA")
        x = int(layer["cx"] * S - im.width / 2)
        y = int(layer["bottom"] * S - im.height)
        if layer.get("shadow", True):
            sh = Image.new("RGBA", (W, H), (0, 0, 0, 0))
            sd = ImageDraw.Draw(sh)
            sw = im.width * 0.9
            sd.ellipse((x + im.width / 2 - sw / 2 + 6 * S, y + im.height - 4 * S,
                        x + im.width / 2 + sw / 2 + 6 * S, y + im.height + 5 * S), fill=(0, 0, 0, 90))
            bg = Image.alpha_composite(bg, sh.filter(ImageFilter.GaussianBlur(4 * S)))
        bg.alpha_composite(im, (x, y))
    out = PREV / "mam-anh-that-preview.png"
    bg.convert("RGB").save(out, optimize=True)
    print("preview", out)


if __name__ == "__main__":
    main()
