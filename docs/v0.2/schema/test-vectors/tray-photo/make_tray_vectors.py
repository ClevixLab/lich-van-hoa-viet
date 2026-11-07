#!/usr/bin/env python3
"""Tệp ảnh cho kịch bản `lvhv-store-tool scenario tray_photo` (YC-2026-10; 01 Bổ sung 01-YC10-a, MA10-02).

`tray-1601x1200.jpg`: ảnh JPEG thật (Pillow, chất lượng 85, không siêu dữ liệu), cạnh dài 1601 px — hợp lệ với mọi
kiểm tra B6 của `Photo` (≤ 2048 px, ≤ 1 MiB) nhưng quá giới hạn 1600 px của ảnh mâm lễ, nên lõi phải từ chối khi gắn làm
`memorial.tray_photo_id` (lưu và nhập). Sinh từ ảnh mẫu `photo-…5e09.jpg`; sha256 ghi ở `manifest.json`.

    python3 docs/v0.2/schema/test-vectors/tray-photo/make_tray_vectors.py
"""
import hashlib
import io
import json
import os

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
SAMPLE = os.path.join(HERE, "..", "photo-0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e09.jpg")


def main():
    im = Image.open(SAMPLE).convert("RGB").resize((1601, 1200))
    out = io.BytesIO()
    im.save(out, "JPEG", quality=85, optimize=False, progressive=False)
    b = out.getvalue()
    assert len(b) <= 1_048_576, len(b)
    open(os.path.join(HERE, "tray-1601x1200.jpg"), "wb").write(b)
    m = {
        "description": "YC-2026-10: ảnh cho kịch bản tray_photo; expected-scenario.json là kết quả chuẩn hóa mà bản Rust và bản TS của lvhv-store-tool phải in.",
        "files": {"tray-1601x1200.jpg": {"sha256": hashlib.sha256(b).hexdigest(), "width": 1601, "height": 1200, "mime": "image/jpeg"}},
    }
    open(os.path.join(HERE, "manifest.json"), "w", encoding="utf-8").write(json.dumps(m, ensure_ascii=False, indent=1) + "\n")
    print(len(b), m["files"]["tray-1601x1200.jpg"]["sha256"])


if __name__ == "__main__":
    main()
