#!/usr/bin/env bash
# Chặn trôi dạt Rust ↔ TypeScript (L7, L12, RS-12, DUYET-MA §4) và kiểm L13.
#
# 1. Build lvhv-dump (release) từ lõi HIỆN TẠI.
# 2. Sinh MỚI vào thư mục tạm (không đọc tệp vàng core/out/ đã commit), cho cả ba phương pháp (hnd, precise, jpl):
#      dayinfo-jsonl 1900-01-01 … 3000-12-31, tet 1900 … 1999 và 2000 … 3000,
#      gooddays mẫu: mọi việc × 2026-01-01 … 2026-12-31 × (không tuổi, năm âm 1976).
# 3. Build tsport và so từng byte (tsport/test/compare.mjs với LVHV_GOLDEN_DIR) + test đầu vào xấu.
# 4. Sinh lại tsport/src/ngochap_tables.ts từ bảng Rust và so byte với tệp đang có.
# 5. L13: phần 2000-01-01 … 3000-12-31 của dayinfo mới phải giống hệt từng byte tệp chuẩn
#    core/out/baseline-2000-3000/{hnd,precise}.jsonl.gz (lưu trước khi mở rộng sang 1900).
# 6. v0.2 (lvhv-memorial): lvhv-memorial-dump (Rust) ↔ tsport/test/memorial_dump.mjs (TypeScript), so
#    từng byte mọi đầu ra kiểm định (docs/v0.2/02 Phụ lục A): gio-dates solar 1900–3000 / lunar 1900–2999
#    / unknown 1900–3000 (R1, cả --gio-het 3 cho một mẫu), reminders configs + reminders đầy đủ R3,
#    upcoming 2026–2036 (R11), r10 a–d, r12 --steps, ics (bộ R3 và tệp ví dụ), derive; + test đơn vị TS.
# Thoát khác 0 nếu có bất kỳ sai lệch nào.
#
# Cách dùng (từ bất kỳ đâu):  scripts/check-parity.sh            # cả ba phương pháp
#                             scripts/check-parity.sh --method hnd
#                             KEEP=1 scripts/check-parity.sh     # giữ thư mục tạm để xem
#                             scripts/check-parity.sh --memorial-only   # chỉ phần 6 (~6 phút)
# Cần: cargo, node ≥ 18, gzip; pwa/node_modules đã cài (tsport dùng tsc/esbuild của pwa/).
# Thời gian trên máy 2 vCPU: ~4–5 phút.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

METHODS=(hnd precise jpl)
MEMORIAL_ONLY=""
if [[ "${1:-}" == "--method" ]]; then METHODS=("${2:?thiếu tên phương pháp}"); fi
if [[ "${1:-}" == "--memorial-only" ]]; then MEMORIAL_ONLY=1; fi

TMP="$(mktemp -d "${TMPDIR:-/tmp}/lvhv-parity.XXXXXX")"
cleanup() { if [[ -z "${KEEP:-}" ]]; then rm -rf "$TMP"; else echo "[parity] giữ $TMP"; fi; }
trap cleanup EXIT

log() { echo "[parity] $*"; }

if [[ ! -x pwa/node_modules/.bin/esbuild || ! -x pwa/node_modules/.bin/tsc ]]; then
  echo "[parity] LỖI: thiếu pwa/node_modules (chạy: cd pwa && npm ci --ignore-scripts && node node_modules/esbuild/install.js)" >&2
  exit 2
fi

# ---- 6. v0.2 lvhv-memorial: Rust ↔ TypeScript
memorial_parity() {
  log "build lvhv-memorial-dump (release) + tsport"
  cargo build --release --locked -p lvhv-memorial --bin lvhv-memorial-dump --quiet
  (cd tsport && npm run -s typecheck && npm run -s build >/dev/null)
  node tsport/test/memorial_unit.mjs || status=1
  local R="$ROOT/target/release/lvhv-memorial-dump" T="node $ROOT/tsport/test/memorial_dump.mjs"
  local DS="$ROOT/memorial/testdata/r3-dataset.json" EX="$ROOT/docs/v0.2/schema/example-backup-v1.json"
  local cmds=(
    "gio-dates solar 1900-01-01 3000-12-31 10"
    "gio-dates lunar 1900 2999 10"
    "gio-dates unknown 1900 3000"
    "--gio-het 3 gio-dates solar 2000-01-01 2030-12-31 10"
    "--method hnd gio-dates solar 2020-01-01 2030-12-31 10"
    "--method precise gio-dates solar 2070-01-01 2090-12-31 10"
    "reminders configs"
    "reminders $DS"
    "upcoming $DS 2026-01-01 2036-12-31"
    "r10 a" "r10 b" "r10 c" "r10 d"
    "r12 $DS --steps"
    "ics $DS 2026-11-06 20261106T031500Z"
    "ics $EX 2026-07-01 20261106T031500Z"
    "derive $EX"
  )
  local c a b
  for c in "${cmds[@]}"; do
    # shellcheck disable=SC2086
    a=$($R $c | sha256sum | cut -d' ' -f1)
    # shellcheck disable=SC2086
    b=$($T $c | sha256sum | cut -d' ' -f1)
    if [[ "$a" == "$b" ]]; then log "memorial [$c]: giống hệt ($a)"
    else echo "[parity] LỖI: memorial [$c]: Rust $a ≠ TS $b" >&2; status=1; fi
  done
  # Tệp dẫn xuất của bộ R3 phải đúng bằng lệnh derive hiện hành.
  python3 "$ROOT/memorial/testdata/make_r3_dataset.py" > "$TMP/r3-in.json"
  if ! "$R" derive "$TMP/r3-in.json" | cmp -s - "$DS"; then
    echo "[parity] LỖI: memorial/testdata/r3-dataset.json khác bản sinh lại (make_r3_dataset.py | derive)" >&2; status=1
  fi
}

status=0
if [[ -z "$MEMORIAL_ONLY" ]]; then
log "build lvhv-dump (release)"
cargo build --release --locked -p lvhv-core --bin lvhv-dump --quiet
DUMP="$ROOT/target/release/lvhv-dump"

gen() {
  local method="$1" out="$2"
  mkdir -p "$out"
  "$DUMP" --method "$method" dayinfo-jsonl 1900-01-01 3000-12-31 | gzip -1 -n > "$out/dayinfo_1900_3000.jsonl.gz"
  "$DUMP" --method "$method" tet 1900 1999 > "$out/tet_1900_1999.csv"
  "$DUMP" --method "$method" tet 2000 3000 > "$out/tet.csv"
  {
    "$DUMP" --method "$method" gooddays all 2026-01-01 2026-12-31 none
    "$DUMP" --method "$method" gooddays all 2026-01-01 2026-12-31 1976
  } > "$out/gooddays_sample.jsonl"
}

pids=()
for m in "${METHODS[@]}"; do
  case "$m" in
    hnd) dir="$TMP" ;;
    precise) dir="$TMP/precise" ;;
    jpl) dir="$TMP/jpl" ;;
    *) echo "[parity] phương pháp không hợp lệ: $m" >&2; exit 2 ;;
  esac
  log "sinh dữ liệu Rust ($m) → $dir"
  gen "$m" "$dir" &
  pids+=($!)
done
for p in "${pids[@]}"; do wait "$p"; done

log "build + typecheck tsport"
(cd tsport && npm run -s typecheck && npm run -s build >/dev/null)

log "tsport: test đầu vào xấu"
node tsport/test/bad_input.mjs

for m in "${METHODS[@]}"; do
  log "so byte Rust ↔ tsport ($m)"
  if ! LVHV_GOLDEN_DIR="$TMP" node tsport/test/compare.mjs --method "$m"; then status=1; fi
done

log "bảng ngày Jpl của tsport phải được sinh từ Rust (lvhv-dump jpl-days-ts)"
if ! "$DUMP" jpl-days-ts | cmp -s - tsport/src/jpl_days.ts; then
  echo "[parity] LỖI: tsport/src/jpl_days.ts khác bản sinh từ lõi Rust (lvhv-dump jpl-days-ts)" >&2
  status=1
fi

log "bảng Ngọc Hạp TS phải được sinh từ Rust (emit_ts_tables)"
cp tsport/src/ngochap_tables.ts "$TMP/ngochap_tables.before.ts"
cargo test --release --locked -p lvhv-ngochap --quiet emit_ts_tables -- --ignored >/dev/null
if ! cmp -s "$TMP/ngochap_tables.before.ts" tsport/src/ngochap_tables.ts; then
  echo "[parity] LỖI: tsport/src/ngochap_tables.ts khác bản sinh từ Rust (đã ghi bản mới; xem git diff)" >&2
  status=1
fi

# L13: số dòng của 1900-01-01 … 1999-12-31 = JDN(2000-01-01) − JDN(1900-01-01) = 36524.
PRE2000_LINES=36524
for m in "${METHODS[@]}"; do
  # L13 không áp dụng cho Jpl (docs/02 A4 mục 4; L16 thay thế — docs/reports/BAO-CAO-JPL-L16.md).
  case "$m" in hnd) dir="$TMP" ;; precise) dir="$TMP/precise" ;; jpl) continue ;; esac
  base="core/out/baseline-2000-3000/$m.jsonl.gz"
  log "L13: 2000–3000 ($m) so với $base"
  if [[ ! -f "$base" ]]; then
    echo "[parity] LỖI: thiếu tệp chuẩn $base" >&2; status=1; continue
  fi
  if cmp -s <(gzip -dc "$dir/dayinfo_1900_3000.jsonl.gz" | tail -n +$((PRE2000_LINES + 1))) <(gzip -dc "$base"); then
    log "[$m] L13: giống hệt từng byte"
  else
    echo "[parity] LỖI: [$m] L13 — kết quả 2000–3000 khác tệp chuẩn" >&2; status=1
  fi
done

fi
memorial_parity

if [[ $status -eq 0 ]]; then log "ĐẠT: 0 sai lệch"; else log "KHÔNG ĐẠT"; fi
exit $status
