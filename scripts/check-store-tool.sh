#!/usr/bin/env bash
# Chạy công cụ hộp đen lvhv-store-tool (docs/v0.2/10-CONG-CU-KIEM-DINH.md) — bản Rust và bản TypeScript/PWA — trên
# vector trong docs/v0.2/schema/test-vectors/, kiểm kết quả với tệp kỳ vọng đã commit và so hai bản với nhau.
#   - open: bộ mẫu đóng băng released/0.2-dev-before-attachments (chuyển 1 → 2, mở lại, --force-migration-failure)
#   - import: backup.lvhv của bộ mẫu (Thay toàn bộ, Gộp vào kho đã chuyển), example-v1.lvhv, notice-v1.lvhv,
#     sai mật khẩu, 21 tệp corrupt/
#   - save-person: viết hoa khi lưu (danh_xung, quan_he), gioi_tinh null → bỏ khóa
#   - suggestions, display-capitalize
#   - check-media: mọi ca của media/manifest.json (B6)
# Cần: cargo, node ≥ 20, python3, pwa/node_modules (npm ci). Không sửa tệp nào trong kho mã (làm trên bản sao tạm).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
TV="$ROOT/docs/v0.2/schema/test-vectors"
FIX="$TV/released/0.2-dev-before-attachments"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/lvhv-store-tool-ci.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

echo "[store-tool] dựng bản Rust"
cargo build --locked -q -p lvhv-ffi --bin lvhv-store-tool
BIN="${CARGO_TARGET_DIR:-$ROOT/target}/debug/lvhv-store-tool"
TS=(node "$ROOT/pwa/test/tools/lvhv-store-tool.mjs")
PW="mật khẩu thử nghiệm"

# run <tên> <mã-thoát-kỳ-vọng> <lệnh…>: ghi stdout vào $WORK/<tên>.json (stderr: .err), kiểm mã thoát.
run() {
  local name="$1" want="$2"; shift 2
  local rc=0
  "$@" > "$WORK/$name.json" 2> "$WORK/$name.err" || rc=$?
  if [ "$rc" != "$want" ]; then
    echo "[store-tool] LỖI: $name thoát $rc, cần $want" >&2; cat "$WORK/$name.json" "$WORK/$name.err" >&2; exit 1
  fi
}

# Ca của media/manifest.json: "<số thứ tự> <kind> <mime> <tệp> [width height]" mỗi dòng.
python3 - "$TV/media/manifest.json" > "$WORK/media-cases.txt" <<'EOF'
import json, sys
for i, c in enumerate(json.load(open(sys.argv[1], encoding="utf-8"))["cases"]):
    dims = f" {c['width']} {c['height']}" if c.get("width") is not None else ""
    print(f"{i} {c['kind']} {c['mime']} {c['file']}{dims}")
EOF
media() { # media <tiền tố> <lệnh…>
  local pre="$1"; shift
  local i kind mime file dims
  while read -r i kind mime file dims; do
    # shellcheck disable=SC2086
    run "$pre-media-$i" 0 "$@" check-media "$kind" "$mime" "$TV/media/$file" $dims
  done < "$WORK/media-cases.txt"
}

echo "[store-tool] Rust"
cp -r "$FIX/store" "$WORK/rs-a"
cp -r "$FIX/store" "$WORK/rs-b"
run rs-open 0 "$BIN" open "$WORK/rs-a"
run rs-reopen 0 "$BIN" open "$WORK/rs-a"
run rs-fail 0 "$BIN" open "$WORK/rs-b" --force-migration-failure
run rs-import-old 0 "$BIN" import "$FIX/backup.lvhv" "$PW" --replace
run rs-import-merge 0 "$BIN" import "$FIX/backup.lvhv" "$PW" --merge --into "$WORK/rs-a"
run rs-import-ex 0 "$BIN" import "$TV/example-v1.lvhv" "$PW" --replace
run rs-import-notice 0 "$BIN" import "$TV/supplement-yc05/notice-v1.lvhv" "$PW"
run rs-import-wrongpw 1 "$BIN" import "$TV/example-v1.lvhv" "sai mật khẩu 1"
for f in "$TV"/corrupt/*.lvhv; do
  run "rs-corrupt-$(basename "$f" .lvhv)" 1 "$BIN" import "$f" "$PW"
done
python3 - "$TV/example-v1.json" "$WORK/person-in.json" <<'EOF'
import json, sys
p = json.load(open(sys.argv[1], encoding="utf-8"))["persons"][2]   # người đang sống, không ảnh
p.update(id="", danh_xung="chị", quan_he="chị gái", gioi_tinh=None)
json.dump(p, open(sys.argv[2], "w", encoding="utf-8"), ensure_ascii=False)
EOF
run rs-save 0 "$BIN" save-person "$WORK/rs-new" "$WORK/person-in.json"
for k in deceased living; do
  run "rs-sugg-$k" 0 "$BIN" suggestions "$k"
  for g in nam nu; do run "rs-sugg-$k-$g" 0 "$BIN" suggestions "$k" "$g"; done
done
run rs-disp 0 "$BIN" display-capitalize "bà cô tổ họ"
run rs-usage 2 "$BIN" suggestions dead
media rs "$BIN"

echo "[store-tool] TypeScript/PWA"
run ts-open 0 "${TS[@]}" open "$FIX"
run ts-fail 0 "${TS[@]}" open "$FIX" --force-migration-failure
run ts-import-old 0 "${TS[@]}" import "$FIX/backup.lvhv" "$PW" --replace
run ts-import-merge 0 "${TS[@]}" import "$FIX/backup.lvhv" "$PW" --merge --into "$FIX"
run ts-import-ex 0 "${TS[@]}" import "$TV/example-v1.lvhv" "$PW" --replace
run ts-import-notice 0 "${TS[@]}" import "$TV/supplement-yc05/notice-v1.lvhv" "$PW"
run ts-import-wrongpw 1 "${TS[@]}" import "$TV/example-v1.lvhv" "sai mật khẩu 1"
for c in magic file_version-2 ciphertext truncated-1; do
  run "ts-corrupt-$c" 1 "${TS[@]}" import "$TV/corrupt/$c.lvhv" "$PW"
done
run ts-save 0 "${TS[@]}" save-person - "$WORK/person-in.json"
for k in deceased living; do
  run "ts-sugg-$k" 0 "${TS[@]}" suggestions "$k"
  for g in nam nu; do run "ts-sugg-$k-$g" 0 "${TS[@]}" suggestions "$k" "$g"; done
done
run ts-disp 0 "${TS[@]}" display-capitalize "bà cô tổ họ"
run ts-usage 2 "${TS[@]}" suggestions dead
media ts "${TS[@]}"

echo "[store-tool] kiểm kết quả"
python3 - "$WORK" "$TV" <<'EOF'
import json, sys
W, TV = sys.argv[1:]
fails = 0
def ok(c, m):
    global fails
    print(("  ok  " if c else "FAIL  ") + m)
    if not c: fails += 1
J = lambda n: json.load(open(f"{W}/{n}.json", encoding="utf-8"))
exp = json.load(open(f"{TV}/released/expected/0.2-dev-before-attachments.json", encoding="utf-8"))
ex_m = json.load(open(f"{TV}/manifest.json", encoding="utf-8"))
yc = json.load(open(f"{TV}/supplement-yc05/manifest.json", encoding="utf-8"))
ex_doc = json.load(open(f"{TV}/example-v1.json", encoding="utf-8"))
strip = lambda d: {k: v for k, v in d.items() if k not in ("app", "exported_at")}

for side, ver in (("rs", 2), ("ts", 3)):
    o = J(f"{side}-open")
    ms = o["migration_status"]
    ok((ms["state"], ms["from_version"], ms["to_version"]) == ("migrated", 1, ver), f"{side} open: chuyển 1 → {ver}")
    ok(o["store"]["document"] == exp["state"] and o["store"]["state_sha256"] == exp["state_sha256"],
       f"{side} open: kho sau chuyển = released/expected (state_sha256 {exp['state_sha256'][:12]}…)")
    f = J(f"{side}-fail")
    ms = f["migration_status"]
    ok((ms["state"], ms["to_version"]) == ("read_only", 1) and ms["error"], f"{side} --force-migration-failure: chỉ đọc, phiên bản 1")
    pr = f["read_only_probe"]
    ok(all(not pr[k]["ok"] for k in ("put_settings", "put_person", "delete_person")) and pr["export_backup"]["ok"],
       f"{side} chỉ đọc: ghi bị từ chối, xuất sao lưu vẫn được")
    ok(not f["store"]["attachment_bytes"] and any("certificate_photo_ids" in (p.get("death") or {}).get("notice", {}) for p in f["store"]["document"]["persons"]),
       f"{side} chỉ đọc: dữ liệu cũ (certificate_photo_ids) còn nguyên")
    ok(J(f"{side}-import-old")["store"]["state_sha256"] == exp["state_sha256"], f"{side} import backup.lvhv cũ --replace = released/expected")
    ok(J(f"{side}-import-merge")["ok"], f"{side} import --merge vào kho đã chuyển")
    ie = J(f"{side}-import-ex")
    ok(strip(ie["store"]["document"]) == strip(ex_doc), f"{side} import example-v1.lvhv: kho = example-v1.json")
    ok(J(f"{side}-import-notice")["store"]["state_sha256"] == yc["state_sha256"], f"{side} import notice-v1.lvhv: state_sha256 = manifest")
    ok(J(f"{side}-import-wrongpw")["stage"] == "open_backup", f"{side} sai mật khẩu → từ chối ở open_backup")
    s = J(f"{side}-save")["saved"]
    ok(s["danh_xung"] == "Chị" and s["quan_he"] == "Chị gái" and "gioi_tinh" not in s and s["rev"] == 1 and len(s["id"]) == 36,
       f"{side} save-person: viết hoa khi lưu, gioi_tinh null → bỏ khóa, rev 1")
    ok(len(J(f"{side}-sugg-deceased")["danh_xung"]) == 22 and len(J(f"{side}-sugg-living")["danh_xung"]) == 17,
       f"{side} suggestions: 22 cách gọi người mất, 17 người sống")
    ok(J(f"{side}-disp")["output"] == "Bà cô tổ họ", f"{side} display-capitalize")

import glob, os
n = 0
for c in ex_m["corrupt"]:
    r = J("rs-corrupt-" + os.path.basename(c["file"])[:-5])
    n += r["stage"] == "open_backup" and r["error"]["kind"] == "Backup"
ok(n == len(ex_m["corrupt"]) == 21, f"rs import: {n}/21 tệp corrupt/ bị từ chối (Backup)")
ok(all(J(f"ts-corrupt-{c}")["stage"] == "open_backup" for c in ("magic", "file_version-2", "ciphertext", "truncated-1")), "ts import: 4 tệp corrupt/ bị từ chối")

mc = json.load(open(f"{TV}/media/manifest.json", encoding="utf-8"))["cases"]
for side in ("rs", "ts"):
    bad = [f"{c['file']}/{c['kind']}: {J(f'{side}-media-{i}')['verdict']} != {c['expect']}" for i, c in enumerate(mc) if J(f"{side}-media-{i}")["verdict"] != c["expect"]]
    ok(not bad, f"{side} check-media: {len(mc) - len(bad)}/{len(mc)} ca media/manifest.json" + (f" — {bad}" if bad else ""))

# Rust ≡ TypeScript
for n in ("sugg-deceased", "sugg-living", "sugg-deceased-nam", "sugg-deceased-nu", "sugg-living-nam", "sugg-living-nu", "disp"):
    ok(J(f"rs-{n}") == J(f"ts-{n}"), f"Rust ≡ TS: {n}")
for n in ("open", "import-old", "import-ex", "import-notice"):
    a, b = J(f"rs-{n}")["store"], J(f"ts-{n}")["store"]
    ok(a["document"] == b["document"] and a["state_sha256"] == b["state_sha256"] and a["photo_bytes"] == b["photo_bytes"]
       and a["attachment_bytes"] == b["attachment_bytes"], f"Rust ≡ TS: {n} (tài liệu kho, byte ảnh, tệp đính kèm)")
vol = ("id", "created_at", "updated_at")
a, b = J("rs-save")["saved"], J("ts-save")["saved"]
ok({k: v for k, v in a.items() if k not in vol} == {k: v for k, v in b.items() if k not in vol}, "Rust ≡ TS: save-person (trừ id, thời điểm)")
print("ĐẠT" if not fails else f"KHÔNG ĐẠT: {fails} lỗi")
sys.exit(1 if fails else 0)
EOF
echo "[store-tool] ĐẠT"
