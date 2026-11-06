"""Tạo logo Clevix nền trong suốt, cắt sát, thu nhỏ cho chân trang PWA.
Dùng: python3 scripts/make-clevix-logo.py <ảnh gốc> <thư mục ra>"""
import sys
from PIL import Image
import numpy as np

src, out = sys.argv[1], sys.argv[2]
im = np.asarray(Image.open(src).convert("RGB")).astype(np.float64)
h, w, _ = im.shape
# màu nền: trung vị các điểm ở 4 góc
corners = np.concatenate([im[:20, :20].reshape(-1, 3), im[:20, -20:].reshape(-1, 3),
                          im[-20:, :20].reshape(-1, 3), im[-20:, -20:].reshape(-1, 3)])
bg = np.median(corners, axis=0)
dist = np.sqrt(((im - bg) ** 2).sum(axis=2))
a = np.clip((dist - 12) / 70, 0, 1)            # mềm hoá viền, bỏ nhiễu nền
safe = np.maximum(a, 1e-6)[..., None]
rgb = np.clip((im - bg * (1 - a[..., None])) / safe, 0, 255)  # khử pha trộn với nền
rgba = np.dstack([rgb, a * 255]).astype(np.uint8)
ys, xs = np.where(a > 0.05)
pad = 6
box = (max(xs.min() - pad, 0), max(ys.min() - pad, 0), min(xs.max() + pad, w), min(ys.max() + pad, h))
img = Image.fromarray(rgba, "RGBA").crop(box)
print("bg", bg.round(1), "box", box, "size", img.size)
for hh in (72, 144, 216):  # 24px và 48px hiển thị ở mật độ 3x
    r = img.resize((round(img.width * hh / img.height), hh), Image.LANCZOS)
    r = r.quantize(colors=64, method=Image.Quantize.FASTOCTREE, dither=Image.Dither.NONE)
    r.save(f"{out}/clevix-logo-{hh}.png", optimize=True)
    print(hh, r.size)
