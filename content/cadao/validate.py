#!/usr/bin/env python3
"""Kiểm tra content/cadao/cadao.json (tiêu chí C1 và quy tắc chọn câu hằng ngày).

Chạy:  python3 content/cadao/validate.py   (thoát mã 1 nếu có lỗi)
"""
import collections
import json
import os
import re
import sys
import unicodedata

HERE = os.path.dirname(os.path.abspath(__file__))
PATH = os.path.join(HERE, "cadao.json")

TYPES = {"ca dao", "tục ngữ", "thành ngữ"}
THEMES = {"gia_dinh", "hieu_thao", "hoc_tap", "can_cu", "ban_be", "biet_on",
          "kien_nhan", "thien_nhien", "nong_nghiep", "nhan_ai", "que_huong", "dao_duc"}
SEED = 0x4C564E31  # xem select.md
EPOCH_JDN = 2451545


def norm(s):
    """Khóa chống trùng: bỏ dấu, chữ thường, bỏ dấu câu, gộp khoảng trắng."""
    s = unicodedata.normalize("NFD", s.lower()).replace("đ", "d")
    s = "".join(c for c in s if unicodedata.category(c) != "Mn")
    return re.sub(r"\s+", " ", re.sub(r"[^\w\s]", " ", s)).strip()


def xorshift32(x):
    x ^= (x << 13) & 0xFFFFFFFF
    x ^= x >> 17
    x ^= (x << 5) & 0xFFFFFFFF
    return x & 0xFFFFFFFF


def expected_order(n):
    perm = list(range(n))
    x = SEED
    for i in range(n - 1, 0, -1):
        x = xorshift32(x)
        j = x % (i + 1)
        perm[i], perm[j] = perm[j], perm[i]
    order = [0] * n
    for pos, idx in enumerate(perm):
        order[idx] = pos
    return order


def main():
    data = json.load(open(PATH, encoding="utf-8"))
    errors = []
    seen = {}
    n = len(data)
    for i, e in enumerate(data):
        eid = e.get("id", f"#{i}")
        if eid != f"cd{i + 1:04d}":
            errors.append(f"{eid}: id không liên tục (mong đợi cd{i + 1:04d})")
        if e.get("type") not in TYPES:
            errors.append(f"{eid}: type không hợp lệ: {e.get('type')!r}")
        if e.get("theme") not in THEMES:
            errors.append(f"{eid}: theme không hợp lệ: {e.get('theme')!r}")
        text = e.get("text", "")
        if not text.strip():
            errors.append(f"{eid}: text rỗng")
        if text != unicodedata.normalize("NFC", text):
            errors.append(f"{eid}: text chưa chuẩn hóa NFC")
        lines = e.get("lines") or []
        if not lines or any(not l.strip() for l in lines):
            errors.append(f"{eid}: lines rỗng")
        elif " / ".join(lines) != text:
            errors.append(f"{eid}: lines không khớp text")
        if e.get("type") == "ca dao" and len(lines) < 2:
            errors.append(f"{eid}: ca dao phải có ≥ 2 dòng")
        if e.get("type") in ("tục ngữ", "thành ngữ") and len(lines) != 1:
            errors.append(f"{eid}: tục ngữ/thành ngữ phải một dòng")
        meaning = e.get("meaning", None)
        if meaning is None:
            errors.append(f"{eid}: thiếu trường meaning")
        elif len(meaning.split()) > 30:
            errors.append(f"{eid}: meaning quá dài ({len(meaning.split())} từ)")
        srcs = e.get("sources") or []
        if not srcs:
            errors.append(f"{eid}: C1 — không có nguồn")
        for s in srcs:
            if not (s.get("title") and s.get("url", "").startswith("https://") and s.get("license")):
                errors.append(f"{eid}: nguồn thiếu title/url/license")
        k = norm(text)
        if k in seen:
            errors.append(f"{eid}: trùng với {seen[k]}")
        seen[k] = eid

    orders = [e.get("order") for e in data]
    if sorted(o for o in orders if isinstance(o, int)) != list(range(n)):
        errors.append("order không phải hoán vị 0..N-1")
    elif orders != expected_order(n):
        errors.append("order không khớp thuật toán trong select.md")

    tc = collections.Counter(e.get("type") for e in data)
    th = collections.Counter(e.get("theme") for e in data)
    print(f"Tổng số câu: {n}")
    print("Theo loại:", ", ".join(f"{k}={v}" for k, v in tc.most_common()))
    print("Theo chủ đề:", ", ".join(f"{k}={v}" for k, v in sorted(th.items())))
    print("Có ý nghĩa (meaning) điền sẵn:", sum(1 for e in data if e.get("meaning")))
    with_free = sum(1 for e in data if any(not s["license"].startswith("Chỉ dùng") for s in e["sources"]))
    print("Câu có ít nhất một nguồn giấy phép mở/phạm vi công cộng:", with_free)
    # Ví dụ chọn câu hôm nay theo select.md (2026-11-06 = JDN 2461351)
    jdn = 2461351
    pos = (jdn - EPOCH_JDN) % n
    pick = next(e for e in data if e["order"] == pos)
    print(f"Ví dụ JDN {jdn}: vị trí {pos} -> {pick['id']}: {pick['text']}")
    if errors:
        print(f"\nLỖI ({len(errors)}):")
        for x in errors:
            print(" -", x)
        sys.exit(1)
    print("\nOK: không có lỗi.")


if __name__ == "__main__":
    main()
