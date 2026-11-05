#!/usr/bin/env bash
# Chặn trôi dạt Rust ↔ TypeScript (L7, RS-12, DUYET-MA §4).
#
# 1. Build lvhv-dump (release) từ lõi HIỆN TẠI.
# 2. Sinh MỚI vào thư mục tạm (không đọc tệp vàng core/out/ đã commit), cho cả hai phương pháp:
#      dayinfo-jsonl 2000-01-01 … 3000-12-31, tet 2000 … 3000,
#      gooddays mẫu: mọi việc × 2026-01-01 … 2026-12-31 × (không tuổi, năm âm 1976).
# 3. Build tsport và so từng byte (tsport/test/compare.mjs với LVHV_GOLDEN_DIR) + test đầu vào xấu.
# 4. Sinh lại tsport/src/ngochap_tables.ts từ bảng Rust và so byte với tệp đang có.
# Thoát khác 0 nếu có bất kỳ sai lệch nào.
#
# Cách dùng (từ bất kỳ đâu):  scripts/check-parity.sh            # cả hai phương pháp
#                             scripts/check-parity.sh --method hnd
#                             KEEP=1 scripts/check-parity.sh     # giữ thư mục tạm để xem
# Cần: cargo, node ≥ 18, gzip; pwa/node_modules đã cài (tsport dùng tsc/esbuild của pwa/).
# Thời gian trên máy 2 vCPU: ~3–4 phút.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

METHODS=(hnd precise)
if [[ "${1:-}" == "--method" ]]; then METHODS=("${2:?thiếu tên phương pháp}"); fi

TMP="$(mktemp -d "${TMPDIR:-/tmp}/lvhv-parity.XXXXXX")"
cleanup() { if [[ -z "${KEEP:-}" ]]; then rm -rf "$TMP"; else echo "[parity] giữ $TMP"; fi; }
trap cleanup EXIT

log() { echo "[parity] $*"; }

if [[ ! -x pwa/node_modules/.bin/esbuild || ! -x pwa/node_modules/.bin/tsc ]]; then
  echo "[parity] LỖI: thiếu pwa/node_modules (chạy: cd pwa && npm ci --ignore-scripts && node node_modules/esbuild/install.js)" >&2
  exit 2
fi

log "build lvhv-dump (release)"
cargo build --release --locked -p lvhv-core --bin lvhv-dump --quiet
DUMP="$ROOT/target/release/lvhv-dump"

gen() {
  local method="$1" out="$2"
  mkdir -p "$out"
  "$DUMP" --method "$method" dayinfo-jsonl 2000-01-01 3000-12-31 | gzip -1 -n > "$out/dayinfo_2000_3000.jsonl.gz"
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

status=0
for m in "${METHODS[@]}"; do
  log "so byte Rust ↔ tsport ($m)"
  if ! LVHV_GOLDEN_DIR="$TMP" node tsport/test/compare.mjs --method "$m"; then status=1; fi
done

log "bảng Ngọc Hạp TS phải được sinh từ Rust (emit_ts_tables)"
cp tsport/src/ngochap_tables.ts "$TMP/ngochap_tables.before.ts"
cargo test --release --locked -p lvhv-ngochap --quiet emit_ts_tables -- --ignored >/dev/null
if ! cmp -s "$TMP/ngochap_tables.before.ts" tsport/src/ngochap_tables.ts; then
  echo "[parity] LỖI: tsport/src/ngochap_tables.ts khác bản sinh từ Rust (đã ghi bản mới; xem git diff)" >&2
  status=1
fi

if [[ $status -eq 0 ]]; then log "ĐẠT: 0 sai lệch"; else log "KHÔNG ĐẠT"; fi
exit $status
