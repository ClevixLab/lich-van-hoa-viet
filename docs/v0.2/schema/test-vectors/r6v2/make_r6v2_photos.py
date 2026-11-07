#!/usr/bin/env python3
"""Ảnh thật của bộ R6v2 (04 Bổ sung B7) — 20 ảnh, cùng số lượng, kiểu và kích thước khai báo như bộ R6 cũ:
ảnh i (0..19) có kích thước (300 + i) × (400 + i); i chẵn là JPEG, i lẻ là WebP. Nội dung là dải màu tất định,
mã hóa bằng Pillow với tham số cố định, không siêu dữ liệu (JPEG chỉ có JFIF APP0; WebP dạng `VP8 ` đơn giản, không
VP8X/EXIF/XMP).

Ảnh đã tạo được COMMIT và KHÓA bằng sha256 (manifest.json của thư mục này, 03 §2.5). Chạy lại tập lệnh với một phiên
bản Pillow/libjpeg/libwebp khác có thể cho byte khác — khi đó KHÔNG thay tệp đã khóa; tập lệnh chỉ ghi lại cách tạo.

    python3 docs/v0.2/schema/test-vectors/r6v2/make_r6v2_photos.py
"""
import os
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))


def name(i: int) -> str:
    return f"photo-{i:02d}.{'jpg' if i % 2 == 0 else 'webp'}"


def image(i: int) -> Image.Image:
    w, h = 300 + i, 400 + i
    im = Image.new("RGB", (w, h))
    px = im.load()
    for y in range(h):
        for x in range(w):
            px[x, y] = ((x * 255 // w + i * 13) % 256, (y * 255 // h + i * 29) % 256, ((x + y) * 3 + i * 7) % 256)
    return im


def main():
    for i in range(20):
        path = os.path.join(HERE, name(i))
        if os.path.exists(path) and "--force" not in sys.argv:
            continue  # tệp đã khóa: không ghi đè
        im = image(i)
        if i % 2 == 0:
            im.save(path, format="JPEG", quality=85, optimize=False, progressive=False, subsampling=2)
        else:
            im.save(path, format="WEBP", quality=80, method=4, lossless=False)
    print("ok")


if __name__ == "__main__":
    main()
