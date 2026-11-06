#!/usr/bin/env bash
# Lưu trữ + sao lưu v0.2 (docs/v0.2/03, 04 R6): vector Rust tất định, kiểm chéo Rust ↔ PWA qua tệp thật.
#   1. Sinh lại vector (docs/v0.2/schema/test-vectors/) và so từng byte với bản đã commit (git status).
#   2. Rust xuất bộ R6 → pwa/test/store/out/r6-from-rust.lvhv
#   3. PWA (Node + fake-indexeddb): mọi kiểm thử R6 phía JS, mở tệp của Rust, xuất r6-from-pwa.lvhv
#   4. Rust mở tệp của PWA → state_hash giống hệt
# Cần: cargo, node ≥ 20, pwa/node_modules (npm ci). Đối số: --quick (bỏ lật bit 65 536 biến thể phía JS).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
TV=docs/v0.2/schema/test-vectors
[ -z "$(git status --porcelain -- "$TV")" ] || { echo "[store] LỖI: $TV có sửa chưa commit — commit trước khi kiểm" >&2; exit 2; }
echo "[store] 1/4 sinh lại vector và so với bản đã commit"
cargo run --release --locked -q -p lvhv-store --example gen_test_vectors
if [ -n "$(git status --porcelain -- "$TV")" ]; then
  git status --porcelain -- "$TV"
  echo "[store] LỖI: vector sinh lại khác bản đã commit" >&2; exit 1
fi
echo "[store] 2/4 Rust xuất bộ R6"
cargo run --release --locked -q -p lvhv-store --example cross_platform -- write
echo "[store] 3/4 PWA (Node)"
(cd pwa && node test/store/run.mjs "$@")
echo "[store] 4/4 Rust mở tệp của PWA"
cargo run --release --locked -q -p lvhv-store --example cross_platform -- check
echo "[store] ĐẠT"
