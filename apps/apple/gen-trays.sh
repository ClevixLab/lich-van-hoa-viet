#!/usr/bin/env bash
# YC-2026-10 (08 §16.3, 04 R15 (b)): nhúng gói ảnh mâm lễ Clevix vào app. Chỉ các mục đã duyệt mà
# scripts/check-tray-pack.py chấp nhận (có tệp, đúng sha256, JPEG/WebP, không siêu dữ liệu, đúng kích thước, trong giới hạn).
# Kết quả: apps/apple/build/TrayPack/{manifest.json, <tệp ảnh>} — project.yml thêm thư mục này vào tài nguyên của app.
# Không có gói hay kịch bản kiểm (nhánh cũ) → gói rỗng. Kiểm không đạt → dừng dựng.
# build-xcframework.sh tự gọi; chạy trước `xcodegen generate`.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
SRC="$ROOT/content/memorial/trays"
OUT="$HERE/build/TrayPack"
rm -rf "$OUT"
mkdir -p "$OUT"
if [[ -f "$SRC/manifest.json" && -f "$ROOT/scripts/check-tray-pack.py" ]]; then
  python3 "$ROOT/scripts/check-tray-pack.py" --manifest "$SRC/manifest.json" --out "$OUT/approved.json" >/dev/null
  python3 - "$SRC" "$OUT" <<'PY'
import json, shutil, sys
from pathlib import Path
src, out = Path(sys.argv[1]), Path(sys.argv[2])
approved = json.loads((out / "approved.json").read_text(encoding="utf-8"))["approved"]
items = []
for it in approved:
    name = it["file"]
    if "/" in name or name.startswith("."):
        sys.exit(f"tên tệp không hợp lệ: {name}")
    shutil.copyfile(src / name, out / name)
    items.append({**it, "trang_thai": "duyet"})
(out / "manifest.json").write_text(json.dumps({"pack_version": 1, "items": items}, ensure_ascii=False, indent=2) + "\n",
                                   encoding="utf-8")
(out / "approved.json").unlink()
print(f"gói ảnh mâm lễ: {len(items)} ảnh đã duyệt")
PY
else
  printf '{"pack_version": 1, "items": []}\n' > "$OUT/manifest.json"
  echo "gói ảnh mâm lễ: chưa có trên nhánh này — gói rỗng"
fi
