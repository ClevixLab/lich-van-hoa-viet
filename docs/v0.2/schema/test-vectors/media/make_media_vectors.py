#!/usr/bin/env python3
"""Bộ tệp kiểm tra chặt ảnh / tệp đính kèm (04 Bổ sung B6, rà soát bảo mật BM2-01; S9, S12).

Sinh tất định các tệp hợp lệ và độc hại cạnh tệp này, cùng `manifest.json` ghi mã kết quả kỳ vọng. Bản Rust
(`store/tests/media.rs`) và bản TypeScript (`pwa/test/store/run.mjs`) phải cho đúng mã đó với mọi ca.

    python3 docs/v0.2/schema/test-vectors/media/make_media_vectors.py
"""
import json
import os
import struct
import zlib

HERE = os.path.dirname(os.path.abspath(__file__))
SAMPLE = os.path.join(HERE, "..", "photo-0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e09.jpg")  # JPEG sạch 768×1024


def jpeg_with_segment(base: bytes, marker: int, payload: bytes) -> bytes:
    seg = bytes([0xFF, marker]) + struct.pack(">H", len(payload) + 2) + payload
    return base[:2] + seg + base[2:]


def jpeg_with_dims(base: bytes, w: int, h: int) -> bytes:
    b = bytearray(base)
    i = 2
    while b[i] == 0xFF:
        m, ln = b[i + 1], (b[i + 2] << 8) | b[i + 3]
        if 0xC0 <= m <= 0xCF and m not in (0xC4, 0xC8, 0xCC):
            b[i + 5:i + 7] = struct.pack(">H", h)
            b[i + 7:i + 9] = struct.pack(">H", w)
            return bytes(b)
        i += 2 + ln
    raise SystemExit("không thấy SOF")


def chunk(tag: bytes, data: bytes) -> bytes:
    return tag + struct.pack("<I", len(data)) + data + (b"\0" if len(data) & 1 else b"")


def riff(chunks: bytes) -> bytes:
    return b"RIFF" + struct.pack("<I", 4 + len(chunks)) + b"WEBP" + chunks


def vp8l(w: int, h: int) -> bytes:
    v = (w - 1) | ((h - 1) << 14)
    return chunk(b"VP8L", b"\x2f" + struct.pack("<I", v) + b"\x00" * 8)


def vp8x(w: int, h: int, flags: int) -> bytes:
    return chunk(b"VP8X", bytes([flags, 0, 0, 0]) + struct.pack("<I", w - 1)[:3] + struct.pack("<I", h - 1)[:3])


def png(w: int, h: int) -> bytes:
    def c(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + c(b"IHDR", ihdr) + c(b"IDAT", zlib.compress(b"\0" * 64)) + c(b"IEND", b"")


def main():
    base = open(SAMPLE, "rb").read()
    pdf_ok = b"%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[]/Count 0>>endobj\ntrailer<</Root 1 0 R>>\n%%EOF\n"
    files = {
        "jpeg-ok.jpg": base,
        "jpeg-exif-gps.jpg": jpeg_with_segment(base, 0xE1, b"Exif\0\0MM\0*\0\0\0\x08\0\x01\x88\x25\0\x04\0\0\0\x01\0\0\0\x1a"),
        "jpeg-xmp.jpg": jpeg_with_segment(base, 0xE1, b"http://ns.adobe.com/xap/1.0/\0<x:xmpmeta/>"),
        "jpeg-iptc.jpg": jpeg_with_segment(base, 0xED, b"Photoshop 3.0\0" + b"8BIM" + b"\0" * 8),
        "jpeg-4000x3000.jpg": jpeg_with_dims(base, 4000, 3000),
        "jpeg-truncated.jpg": base[:200],
        "png-20000x20000-bomb.png": png(20000, 20000),
        "html-as-image.html": b"<!doctype html><html><body><script>alert(1)</script></body></html>\n",
        "pdf-ok.pdf": pdf_ok,
        "pdf-fake-no-eof.pdf": b"%PDF-1.4\n<html><script>alert(1)</script></html>\n",
        "webp-ok-16x16.webp": riff(vp8l(16, 16)),
        "webp-exif.webp": riff(vp8x(16, 16, 0x08) + vp8l(16, 16) + chunk(b"EXIF", b"MM\0*\0\0\0\x08")),
        "webp-xmp-chunk.webp": riff(vp8l(16, 16) + chunk(b"XMP ", b"<x:xmpmeta/>")),
        "webp-3000x10.webp": riff(vp8l(3000, 10)),
    }
    cases = [
        # (tệp, mime, loại, width, height, kỳ vọng)
        ("jpeg-ok.jpg", "image/jpeg", "photo", 768, 1024, "ok"),
        ("jpeg-ok.jpg", "image/jpeg", "attachment", None, None, "ok"),
        ("jpeg-ok.jpg", "image/jpeg", "photo", 800, 1024, "dimensions_mismatch"),
        ("jpeg-ok.jpg", "image/webp", "photo", 768, 1024, "mime_mismatch"),
        ("jpeg-exif-gps.jpg", "image/jpeg", "photo", 768, 1024, "metadata"),
        ("jpeg-exif-gps.jpg", "image/jpeg", "attachment", None, None, "metadata"),
        ("jpeg-xmp.jpg", "image/jpeg", "attachment", None, None, "metadata"),
        ("jpeg-iptc.jpg", "image/jpeg", "photo", 768, 1024, "metadata"),
        ("jpeg-4000x3000.jpg", "image/jpeg", "photo", 4000, 3000, "too_large"),
        ("jpeg-4000x3000.jpg", "image/jpeg", "attachment", None, None, "too_large"),
        ("jpeg-truncated.jpg", "image/jpeg", "attachment", None, None, "bad_header"),
        ("png-20000x20000-bomb.png", "image/jpeg", "photo", 768, 1024, "mime_mismatch"),
        ("png-20000x20000-bomb.png", "image/webp", "attachment", None, None, "mime_mismatch"),
        ("html-as-image.html", "image/jpeg", "attachment", None, None, "mime_mismatch"),
        ("html-as-image.html", "application/pdf", "attachment", None, None, "mime_mismatch"),
        ("pdf-ok.pdf", "application/pdf", "attachment", None, None, "ok"),
        ("pdf-ok.pdf", "image/jpeg", "attachment", None, None, "mime_mismatch"),
        ("pdf-fake-no-eof.pdf", "application/pdf", "attachment", None, None, "pdf_no_eof"),
        ("webp-ok-16x16.webp", "image/webp", "attachment", None, None, "ok"),
        ("webp-ok-16x16.webp", "image/webp", "photo", 16, 16, "ok"),
        ("webp-ok-16x16.webp", "image/jpeg", "photo", 16, 16, "mime_mismatch"),
        ("webp-exif.webp", "image/webp", "attachment", None, None, "metadata"),
        ("webp-xmp-chunk.webp", "image/webp", "photo", 16, 16, "metadata"),
        ("webp-3000x10.webp", "image/webp", "attachment", None, None, "too_large"),
        ("pdf-ok.pdf", "application/zip", "attachment", None, None, "bad_mime"),
    ]
    import hashlib
    for f, b in files.items():
        open(os.path.join(HERE, f), "wb").write(b)
    m = {
        "description": "04 B6 / BM2-01: ảnh và tệp đính kèm hợp lệ và độc hại; kỳ vọng = mã lỗi của media::check_photo / check_attachment (Rust) và checkPhoto / checkAttachment (TS).",
        "files": {f: hashlib.sha256(b).hexdigest() for f, b in sorted(files.items())},
        "cases": [{"file": f, "mime": mi, "kind": k, "width": w, "height": h, "expect": e} for f, mi, k, w, h, e in cases],
    }
    open(os.path.join(HERE, "manifest.json"), "w", encoding="utf-8").write(json.dumps(m, ensure_ascii=False, indent=1) + "\n")
    print(f"{len(files)} tệp, {len(cases)} ca")


if __name__ == "__main__":
    main()
