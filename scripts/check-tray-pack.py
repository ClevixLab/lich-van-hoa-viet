#!/usr/bin/env python3
"""Kiểm gói ảnh mâm lễ của Clevix trước khi nhúng vào bản dựng (YC-2026-10; 04 B8 / R15 (b); 08 §16.3).

Chỉ dùng thư viện chuẩn và kiểm trên byte (không giải mã ảnh). Với `content/memorial/trays/manifest.json`:
- kiểm cấu trúc theo `manifest.schema.json` (cài lại bằng tay, không cần thư viện ngoài);
- chỉ mục `trang_thai = "duyet"` được đóng gói; mục `"cho"` bị bỏ qua (chỉ kiểm cấu trúc);
- với mỗi mục đã duyệt, **thất bại** nếu: thiếu tệp; `sha256` sai; chữ ký tệp không phải JPEG/WebP hoặc không khớp
  `mime`; JPEG có đoạn APP1 (EXIF/XMP) hay APP13 (IPTC), WebP có khối EXIF hay XMP; có dữ liệu thừa sau cuối ảnh
  (JPEG sau EOI, WebP ngoài độ dài RIFF); kích thước điểm ảnh trong phần đầu tệp khác `width`/`height`; tệp > 400 KB;
- **thất bại** nếu tổng các tệp đã duyệt > 2 MB;
- in danh sách mục đã duyệt (JSON); `--out <tệp>` ghi danh sách đó cho bước dựng của từng nền tảng.

Gói rỗng là hợp lệ: không có mâm nào được hiện (R15 (a)).

Dùng (ở gốc kho mã):
  python3 scripts/check-tray-pack.py                       # kiểm gói thật
  python3 scripts/check-tray-pack.py --manifest <đường dẫn> [--out <tệp>]
  python3 scripts/check-tray-pack.py --self-test           # chạy bộ ca kiểm ở content/memorial/trays/test/
Mã thoát: 0 = đạt; 1 = gói không đạt; 2 = lỗi cách dùng.
"""
import argparse
import hashlib
import json
import re
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MANIFEST = ROOT / "content" / "memorial" / "trays" / "manifest.json"
TEST_DIR = ROOT / "content" / "memorial" / "trays" / "test"

MAX_ITEM = 400 * 1024
MAX_TOTAL = 2 * 1024 * 1024
ITEM_KEYS = ["id", "tray_style", "file", "mime", "width", "height", "sha256", "nguon", "giay_phep", "tac_gia",
             "ghi_cong", "duyet_boi", "ngay_duyet", "trang_thai"]
STYLES = {"truyen_thong", "phat_giao", "cong_giao"}
MIMES = {"image/jpeg", "image/webp"}


class PackError(Exception):
    pass


# ---------------------------------------------------------------- cấu trúc manifest

def check_structure(doc):
    errs = []
    if not isinstance(doc, dict) or set(doc) != {"pack_version", "items"}:
        return ["manifest phải là đối tượng có đúng hai khóa pack_version, items"]
    if doc["pack_version"] != 1 or isinstance(doc["pack_version"], bool):
        errs.append("pack_version phải là 1")
    if not isinstance(doc["items"], list):
        return errs + ["items phải là mảng"]
    seen = set()
    for i, it in enumerate(doc["items"]):
        w = f"items[{i}]"
        if not isinstance(it, dict):
            errs.append(f"{w}: phải là đối tượng")
            continue
        keys = set(it)
        if keys != set(ITEM_KEYS):
            miss, extra = sorted(set(ITEM_KEYS) - keys), sorted(keys - set(ITEM_KEYS))
            errs.append(f"{w}: thiếu khóa {miss} / khóa lạ {extra}")
            continue
        s = lambda k: isinstance(it[k], str) and len(it[k]) > 0  # noqa: E731
        if not (s("id") and re.fullmatch(r"[a-z0-9][a-z0-9-]{0,63}", it["id"])):
            errs.append(f"{w}: id không hợp lệ")
        elif it["id"] in seen:
            errs.append(f"{w}: id trùng '{it['id']}'")
        else:
            seen.add(it["id"])
        if it["tray_style"] not in STYLES:
            errs.append(f"{w}: tray_style phải là một trong {sorted(STYLES)}")
        if not (s("file") and re.fullmatch(r"[a-z0-9][a-z0-9/_-]*\.(jpg|jpeg|webp)", it["file"])) or ".." in it["file"]:
            errs.append(f"{w}: file không hợp lệ (đường dẫn tương đối, chữ thường không dấu, đuôi .jpg/.jpeg/.webp)")
        if it["mime"] not in MIMES:
            errs.append(f"{w}: mime phải là image/jpeg hoặc image/webp")
        for k in ("width", "height"):
            v = it[k]
            if not (isinstance(v, int) and not isinstance(v, bool) and 1 <= v <= 4096):
                errs.append(f"{w}: {k} phải là số nguyên 1–4096")
        if not (s("sha256") and re.fullmatch(r"[0-9a-f]{64}", it["sha256"])):
            errs.append(f"{w}: sha256 phải là 64 chữ hex thường")
        for k in ("nguon", "giay_phep", "tac_gia", "duyet_boi"):
            if not s(k):
                errs.append(f"{w}: {k} không được trống")
        if not (it["ghi_cong"] is None or s("ghi_cong")):
            errs.append(f"{w}: ghi_cong phải là chuỗi khác rỗng hoặc null")
        if s("giay_phep") and re.search(r"cc[ -]by", it["giay_phep"], re.I) and not s("ghi_cong"):
            errs.append(f"{w}: giấy phép CC BY / CC BY-SA bắt buộc có ghi_cong")
        if not (s("ngay_duyet") and re.fullmatch(r"\d{4}-\d{2}-\d{2}", it["ngay_duyet"])):
            errs.append(f"{w}: ngay_duyet phải dạng YYYY-MM-DD")
        if it["trang_thai"] not in ("duyet", "cho"):
            errs.append(f"{w}: trang_thai phải là duyet hoặc cho")
    return errs


# ---------------------------------------------------------------- JPEG

SOF = {0xC0, 0xC1, 0xC2, 0xC3, 0xC5, 0xC6, 0xC7, 0xC9, 0xCA, 0xCB, 0xCD, 0xCE, 0xCF}


def inspect_jpeg(b):
    """Trả về (width, height). Ném PackError nếu có APP1/APP13, thiếu SOF/EOI, hay có byte thừa sau EOI."""
    if b[:3] != b"\xff\xd8\xff":
        raise PackError("không có chữ ký JPEG (FF D8 FF)")
    n, i, dims = len(b), 2, None
    while True:
        if i >= n or b[i] != 0xFF:
            raise PackError(f"cấu trúc JPEG hỏng ở byte {i}")
        while i < n and b[i] == 0xFF:  # byte đệm FF
            i += 1
        if i >= n:
            raise PackError("JPEG kết thúc giữa chừng")
        m = b[i]
        i += 1
        if m == 0xD9:  # EOI
            if i != n:
                raise PackError(f"có {n - i} byte thừa sau EOI")
            if dims is None:
                raise PackError("không có đoạn SOF (kích thước)")
            return dims
        if 0xD0 <= m <= 0xD7 or m == 0x01:
            continue
        if i + 2 > n:
            raise PackError("JPEG kết thúc giữa chừng")
        seg_len = struct.unpack(">H", b[i:i + 2])[0]
        if seg_len < 2 or i + seg_len > n:
            raise PackError(f"độ dài đoạn FF{m:02X} không hợp lệ")
        if m == 0xE1:
            raise PackError("có đoạn APP1 (EXIF/XMP)")
        if m == 0xED:
            raise PackError("có đoạn APP13 (IPTC)")
        if m in SOF:
            if seg_len < 7:
                raise PackError("đoạn SOF quá ngắn")
            h, w = struct.unpack(">HH", b[i + 3:i + 7])
            dims = (w, h)
        i += seg_len
        if m == 0xDA:  # SOS: bỏ qua dữ liệu nén tới marker kế tiếp
            while True:
                j = b.find(b"\xff", i)
                if j < 0 or j + 1 >= n:
                    raise PackError("JPEG thiếu EOI")
                nxt = b[j + 1]
                if nxt == 0x00 or 0xD0 <= nxt <= 0xD7 or nxt == 0xFF:
                    i = j + 1 if nxt == 0xFF else j + 2
                    continue
                i = j
                break


# ---------------------------------------------------------------- WebP

def inspect_webp(b):
    if len(b) < 20 or b[:4] != b"RIFF" or b[8:12] != b"WEBP":
        raise PackError("không có chữ ký WebP (RIFF....WEBP)")
    riff = struct.unpack("<I", b[4:8])[0]
    if riff + 8 != len(b):
        raise PackError(f"độ dài RIFF {riff + 8} khác độ dài tệp {len(b)} (dữ liệu thừa hoặc thiếu)")
    i, dims, first = 12, None, True
    while i < len(b):
        if i + 8 > len(b):
            raise PackError("khối WebP bị cắt")
        fourcc, size = b[i:i + 4], struct.unpack("<I", b[i + 4:i + 8])[0]
        p = b[i + 8:i + 8 + size]
        if len(p) != size:
            raise PackError("khối WebP bị cắt")
        if fourcc in (b"EXIF", b"XMP "):
            raise PackError(f"có khối {fourcc.decode().strip()}")
        if first:
            if fourcc == b"VP8X" and size >= 10:
                w = 1 + (p[4] | p[5] << 8 | p[6] << 16)
                h = 1 + (p[7] | p[8] << 8 | p[9] << 16)
                dims = (w, h)
            elif fourcc == b"VP8 " and size >= 10 and p[3:6] == b"\x9d\x01\x2a":
                w, h = struct.unpack("<HH", p[6:10])
                dims = (w & 0x3FFF, h & 0x3FFF)
            elif fourcc == b"VP8L" and size >= 5 and p[0] == 0x2F:
                v = struct.unpack("<I", p[1:5])[0]
                dims = ((v & 0x3FFF) + 1, ((v >> 14) & 0x3FFF) + 1)
            else:
                raise PackError(f"khối đầu WebP không hợp lệ: {fourcc!r}")
            first = False
        i += 8 + size + (size & 1)
    if i != len(b):
        raise PackError("dữ liệu thừa sau khối cuối")
    if dims is None:
        raise PackError("không đọc được kích thước WebP")
    return dims


# ---------------------------------------------------------------- gói

def check_pack(manifest_path):
    """Trả về danh sách mục đã duyệt. Ném PackError nếu gói không đạt."""
    manifest_path = Path(manifest_path)
    try:
        doc = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        raise PackError(f"không đọc được manifest: {e}") from e
    errs = check_structure(doc)
    if errs:
        raise PackError("manifest sai cấu trúc:\n  " + "\n  ".join(errs))
    base = manifest_path.parent.resolve()
    approved, total, problems = [], 0, []
    for it in doc["items"]:
        if it["trang_thai"] != "duyet":
            continue
        f = (base / it["file"]).resolve()
        tag = f"{it['id']} ({it['file']})"
        if base not in f.parents:
            problems.append(f"{tag}: tệp nằm ngoài thư mục gói")
            continue
        if not f.is_file():
            problems.append(f"{tag}: thiếu tệp")
            continue
        b = f.read_bytes()
        if len(b) > MAX_ITEM:
            problems.append(f"{tag}: {len(b)} byte > {MAX_ITEM} (400 KB)")
        if hashlib.sha256(b).hexdigest() != it["sha256"]:
            problems.append(f"{tag}: sha256 không khớp")
        try:
            if b[:3] == b"\xff\xd8\xff":
                kind, (w, h) = "image/jpeg", inspect_jpeg(b)
            elif b[:4] == b"RIFF" and b[8:12] == b"WEBP":
                kind, (w, h) = "image/webp", inspect_webp(b)
            else:
                raise PackError("không phải JPEG hay WebP")
            if kind != it["mime"]:
                problems.append(f"{tag}: tệp là {kind} nhưng mime ghi {it['mime']}")
            if (w, h) != (it["width"], it["height"]):
                problems.append(f"{tag}: kích thước trong tệp {w}×{h} khác manifest {it['width']}×{it['height']}")
        except PackError as e:
            problems.append(f"{tag}: {e}")
        total += len(b)
        approved.append(it)
    if total > MAX_TOTAL:
        problems.append(f"tổng {total} byte > {MAX_TOTAL} (2 MB)")
    if problems:
        raise PackError("gói không đạt:\n  " + "\n  ".join(problems))
    return approved


def self_test():
    """Mỗi thư mục con của test/ có manifest.json và expect.json {"ok": bool, "approved": n, "contains": "..."}."""
    cases = sorted(p for p in TEST_DIR.iterdir() if (p / "manifest.json").is_file())
    if not cases:
        print("không có ca kiểm nào", file=sys.stderr)
        return 1
    global MAX_ITEM, MAX_TOTAL
    failed = 0
    for c in cases:
        exp = json.loads((c / "expect.json").read_text(encoding="utf-8"))
        # Ca kiểm giới hạn dung lượng dùng ngưỡng thu nhỏ (khóa "limits") để không phải lưu tệp lớn trong kho.
        saved = (MAX_ITEM, MAX_TOTAL)
        MAX_ITEM = exp.get("limits", {}).get("item", MAX_ITEM)
        MAX_TOTAL = exp.get("limits", {}).get("total", MAX_TOTAL)
        try:
            got_ok, got_n, msg = True, len(check_pack(c / "manifest.json")), ""
        except PackError as e:
            got_ok, got_n, msg = False, None, str(e)
        finally:
            MAX_ITEM, MAX_TOTAL = saved
        good = got_ok == exp["ok"]
        if good and exp["ok"]:
            good = got_n == exp["approved"]
        if good and not exp["ok"]:
            good = exp["contains"] in msg
        print(f"{'ĐẠT ' if good else 'SAI '} {c.name}: mong {exp}, được ok={got_ok} approved={got_n}"
              + (f" lỗi=«{msg.splitlines()[-1].strip()}»" if msg else ""))
        failed += not good
    print(f"{len(cases) - failed}/{len(cases)} ca đạt")
    return 1 if failed else 0


def main(argv=None):
    ap = argparse.ArgumentParser(description="Kiểm gói ảnh mâm lễ (R15 (b)).")
    ap.add_argument("--manifest", default=str(DEFAULT_MANIFEST))
    ap.add_argument("--out", help="ghi danh sách mục đã duyệt (JSON) ra tệp này")
    ap.add_argument("--self-test", action="store_true")
    a = ap.parse_args(argv)
    if a.self_test:
        return self_test()
    try:
        approved = check_pack(a.manifest)
    except PackError as e:
        print(f"KHÔNG ĐẠT: {e}", file=sys.stderr)
        return 1
    out = {"pack_version": 1, "approved": approved}
    text = json.dumps(out, ensure_ascii=False, indent=2) + "\n"
    if a.out:
        Path(a.out).write_text(text, encoding="utf-8")
    sys.stdout.write(text)
    print(f"ĐẠT: {len(approved)} ảnh đã duyệt" + (" — gói rỗng, không hiện mâm (R15 (a))" if not approved else ""),
          file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
