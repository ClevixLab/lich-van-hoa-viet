#!/usr/bin/env bash
# Sinh Generated/Version.xcconfig từ tệp VERSION ở gốc kho (nguồn duy nhất của số phiên bản).
# project.yml dùng tệp này (configFiles) cho MARKETING_VERSION, CURRENT_PROJECT_VERSION,
# LVHV_BUILD_DATE và APP_STORE_ID. Chạy trước `xcodegen generate` (build-xcframework.sh tự gọi).
#
# Biến môi trường (tùy chọn):
#   LVHV_BUILD_NUMBER  số bản dựng → CURRENT_PROJECT_VERSION (CI: github.run_number; mặc định 1)
#   LVHV_BUILD_DATE    ngày dựng YYYY-MM-DD (mặc định: hôm nay, UTC)
#   APP_STORE_ID       mã số ứng dụng trên App Store (chỉ chữ số). TRỐNG cho tới khi có bản ghi App
#                      Store thật; khi trống, nút cập nhật mở trang tải về của dự án.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
V="$(tr -d '[:space:]' < "$ROOT/VERSION")"
[[ "$V" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "VERSION phải có dạng X.Y.Z, đang là '$V'" >&2; exit 1; }
B="${LVHV_BUILD_NUMBER:-1}"
[[ "$B" =~ ^[0-9]+$ ]] || { echo "LVHV_BUILD_NUMBER phải là số" >&2; exit 1; }
D="${LVHV_BUILD_DATE:-$(date -u +%Y-%m-%d)}"
[[ "$D" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || { echo "LVHV_BUILD_DATE phải có dạng YYYY-MM-DD" >&2; exit 1; }
S="${APP_STORE_ID:-}"
[[ -z "$S" || "$S" =~ ^[0-9]+$ ]] || { echo "APP_STORE_ID chỉ gồm chữ số (hoặc để trống)" >&2; exit 1; }
mkdir -p "$HERE/Generated"
cat > "$HERE/Generated/Version.xcconfig" <<XC
// Sinh bởi gen-version.sh từ VERSION — không sửa tay, không commit.
MARKETING_VERSION = $V
CURRENT_PROJECT_VERSION = $B
LVHV_BUILD_DATE = $D
APP_STORE_ID = $S
XC
echo "Version.xcconfig: $V (bản dựng $B, ngày $D, APP_STORE_ID='${S}')"
