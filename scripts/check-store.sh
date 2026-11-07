#!/usr/bin/env bash
# Lưu trữ + sao lưu v0.2 (docs/v0.2/03, 04 R6): vector Rust tất định, kiểm chéo Rust ↔ PWA qua tệp thật.
#   1. Sinh lại vector (docs/v0.2/schema/test-vectors/) và so từng byte với bản đã commit (git status).
#   2. Rust xuất bộ R6 → pwa/test/store/out/r6-from-rust.lvhv
#   3. PWA (Node + fake-indexeddb): mọi kiểm thử R6 phía JS, mở tệp của Rust, xuất r6-from-pwa.lvhv
#   4. Rust mở tệp của PWA → state_hash giống hệt
#   5. YC-2026-10 (04 Bổ sung B8, R15 (g)): kịch bản `tray_photo` của lvhv-store-tool, Rust ≡ TS = tệp kỳ vọng
# Cần: cargo, node ≥ 20, pwa/node_modules (npm ci). Đối số: --quick (bỏ lật bit 65 536 biến thể phía JS).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
TV=docs/v0.2/schema/test-vectors
[ -z "$(git status --porcelain -- "$TV")" ] || { echo "[store] LỖI: $TV có sửa chưa commit — commit trước khi kiểm" >&2; exit 2; }
echo "[store] 1/5 sinh lại vector và so với bản đã commit"
cargo run --release --locked -q -p lvhv-store --example gen_test_vectors
cargo run --release --locked -q -p lvhv-store --example gen_notice_vector >/dev/null
if [ -n "$(git status --porcelain -- "$TV")" ]; then
  git status --porcelain -- "$TV"
  echo "[store] LỖI: vector sinh lại khác bản đã commit" >&2; exit 1
fi
echo "[store] 2/5 Rust xuất bộ R6"
cargo run --release --locked -q -p lvhv-store --example cross_platform -- write
echo "[store] 3/5 PWA (Node)"
(cd pwa && node test/store/run.mjs "$@")
echo "[store] 4/5 Rust mở tệp của PWA"
cargo run --release --locked -q -p lvhv-store --example cross_platform -- check
echo "[store] 5/5 ảnh mâm lễ: kịch bản tray_photo, Rust ≡ PWA"
TRAYW="$(mktemp -d "${TMPDIR:-/tmp}/lvhv-tray.XXXXXX")"
trap 'rm -rf "$TRAYW"' EXIT
cargo run --release --locked -q -p lvhv-ffi --bin lvhv-store-tool -- scenario tray_photo "$TRAYW/rs-work" > "$TRAYW/rs.json"
(cd pwa && node test/tools/lvhv-store-tool.mjs scenario tray_photo) > "$TRAYW/ts.json"
python3 - "$TRAYW/rs.json" "$TRAYW/ts.json" "$TV/tray-photo/expected-scenario.json" <<'EOF'
import json, sys
rs, ts, exp = (json.load(open(f, encoding="utf-8")) for f in sys.argv[1:])
if not (rs == exp and ts == exp):
    print(f"[store] LỖI: tray_photo — Rust = kỳ vọng: {rs == exp}; TS = kỳ vọng: {ts == exp}", file=sys.stderr)
    sys.exit(1)
print("ok: tray_photo — Rust ≡ TS = tray-photo/expected-scenario.json")
EOF
echo "[store] ĐẠT"
