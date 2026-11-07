#!/usr/bin/env bash
# R14 (docs/v0.2/04 Bổ sung B5) — nâng cấp tại chỗ trên iPhone mô phỏng, không mất dữ liệu.
#   Ca A: bản 0.1 (ứng dụng trên `main`, chưa có Người thân) → bản mới: cỡ chữ, sáng/tối, "Để sau" giữ nguyên.
#   Ca B: bản 0.2-dev cũ (commit $OLD2_REF của nhánh này) tạo dữ liệu mẫu (5 hồ sơ, 1 sự kiện, 2 ảnh, thông tin báo tử)
#         → `xcodebuild test` cài đè bản mới (cùng bundle id) → PeopleUpgradeTests so dữ liệu với bản chép trước khi
#         cài đè, đọc lại ảnh, kiểm lịch nhắc được đặt lại.
# Cần: XCFramework và Generated/ đã dựng ở apps/apple (build-xcframework.sh), xcodegen, RUNNER_TEMP.
set -euo pipefail
ROOT="$(pwd)"
A="$ROOT/apps/apple"
T="$RUNNER_TEMP/r14"
SHOTS="$RUNNER_TEMP/shots-upgrade"
OLD1_REF="${OLD1_REF:-c4d3128}"   # main trước nhánh này: ứng dụng Apple 0.1 (chưa có Người thân)
OLD2_REF="${OLD2_REF:-89dedb3}"
BID="vn.clevix.lichviet"
mkdir -p "$T" "$SHOTS"

UDID=$(python3 "$A/ci/pick_sim.py" iPhone "iPhone 16" "iPhone 16 Pro" "iPhone 15")
echo "Máy mô phỏng: $UDID"
xcrun simctl boot "$UDID" || true
xcrun simctl status_bar "$UDID" override --time 9:41 --batteryState charged --batteryLevel 100 || true

# Lõi CŨ (trước core2): bản cũ phải chạy với đúng lõi của nó, nếu không kho tạo ra đã ở lược đồ mới và ca B/C
# không thử được chuyển đổi. Dựng một lát mô phỏng arm64 từ $OLD2_REF (lõi giống ở $OLD1_REF).
build_old_core() {
  local dir="$T/core-old"
  git worktree add -f "$dir" "$OLD2_REF" >/dev/null
  export IPHONEOS_DEPLOYMENT_TARGET=16.0
  (cd "$dir" && CARGO_TARGET_DIR="$T/target-old" cargo build -q -p lvhv-ffi --lib --release --target aarch64-apple-ios-sim \
    && CARGO_TARGET_DIR="$T/target-old" cargo build -q -p lvhv-ffi --features bindgen-cli --bin uniffi-bindgen) \
    > "$T/core-old.log" 2>&1 || { tail -40 "$T/core-old.log"; exit 1; }
  local lib="$T/target-old/aarch64-apple-ios-sim/release/liblvhv_ffi.a"
  rm -rf "$T/gen-old" "$T/hdr-old" "$T/old-core.xcframework"; mkdir -p "$T/gen-old" "$T/hdr-old"
  "$T/target-old/debug/uniffi-bindgen" generate --library "$lib" --language swift --out-dir "$T/gen-old" \
    --config "$dir/ffi/uniffi.toml" >/dev/null
  cp "$T/gen-old/lvhv_ffiFFI.h" "$T/hdr-old/"
  cp "$T/gen-old/lvhv_ffiFFI.modulemap" "$T/hdr-old/module.modulemap"
  xcodebuild -create-xcframework -library "$lib" -headers "$T/hdr-old" -output "$T/old-core.xcframework" >/dev/null
}

build_old() { # ref tên cấu-hình
  local ref=$1 name=$2 cfg=$3
  local dir="$T/$name"
  git worktree add -f "$dir" "$ref" >/dev/null
  mkdir -p "$dir/apps/apple/build" "$dir/apps/apple/Generated"
  cp -R "$T/old-core.xcframework" "$dir/apps/apple/build/LvhvCore.xcframework"
  cp "$T/gen-old/LvhvCore.swift" "$dir/apps/apple/Generated/LvhvCore.swift"
  cp "$A/Generated/Version.xcconfig" "$dir/apps/apple/Generated/Version.xcconfig"
  (cd "$dir/apps/apple" && xcodegen generate >/dev/null && \
    xcodebuild -project LichVanHoaViet.xcodeproj -scheme LichViet-iOS -configuration "$cfg" \
      -sdk iphonesimulator -destination 'generic/platform=iOS Simulator' -derivedDataPath "$T/dd-$name" \
      ARCHS=arm64 ONLY_ACTIVE_ARCH=NO CODE_SIGNING_ALLOWED=NO CODE_SIGNING_REQUIRED=NO build > "$T/$name.log" 2>&1) \
    || { tail -50 "$T/$name.log"; exit 1; }
  echo "$T/dd-$name/Build/Products/$cfg-iphonesimulator/LichViet.app"
}

echo "== Dựng lõi cũ ($OLD2_REF)"
build_old_core
echo "== Dựng bản cũ 0.1 ($OLD1_REF)"
OLD1_APP=$(build_old "$OLD1_REF" old01 Release | tail -1)
echo "== Dựng bản cũ 0.2-dev ($OLD2_REF)"
OLD2_APP=$(build_old "$OLD2_REF" old02 Debug | tail -1)
echo "== Dựng bản mới (HEAD) + kiểm thử"
(cd "$A" && xcodebuild -project LichVanHoaViet.xcodeproj -scheme LichViet-iOS -configuration Debug \
  -destination "id=$UDID" -derivedDataPath "$T/dd-new" CODE_SIGNING_ALLOWED=NO CODE_SIGNING_REQUIRED=NO \
  build-for-testing > "$T/new.log" 2>&1) || { tail -50 "$T/new.log"; exit 1; }
NEW_APP="$T/dd-new/Build/Products/Debug-iphonesimulator/LichViet.app"
ls -d "$OLD1_APP" "$OLD2_APP" "$NEW_APP"

container() { xcrun simctl get_app_container "$UDID" "$BID" data; }

echo "== Ca A: 0.1 → mới (tùy chọn giao diện)"
xcrun simctl uninstall "$UDID" "$BID" || true
xcrun simctl install "$UDID" "$OLD1_APP"
DATA=$(container)
PL="$DATA/Library/Preferences/$BID.plist"
mkdir -p "$(dirname "$PL")"
# Ghi thẳng tệp (không qua cfprefsd của máy chủ, tránh đọc nhầm bộ đệm).
printf '%s\n' '<?xml version="1.0" encoding="UTF-8"?>' \
  '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">' \
  '<plist version="1.0"><dict><key>theme</key><integer>2</integer><key>size</key><integer>3</integer>' \
  '<key>updateSnoozedUntil</key><real>1893456000</real></dict></plist>' > "$PL"
plutil -convert binary1 "$PL"
xcrun simctl launch "$UDID" "$BID"; sleep 8
xcrun simctl io "$UDID" screenshot "$SHOTS/upgrade-a-v01-old.png"
xcrun simctl terminate "$UDID" "$BID" || true
xcrun simctl install "$UDID" "$NEW_APP"
[ "$(container)" = "$DATA" ] || echo "Thư mục dữ liệu đổi đường dẫn (bình thường với iOS); kiểm bằng nội dung"
DATA=$(container); PL="$DATA/Library/Preferences/$BID.plist"
xcrun simctl launch "$UDID" "$BID"; sleep 8
xcrun simctl io "$UDID" screenshot "$SHOTS/upgrade-a-v01-new.png"
xcrun simctl terminate "$UDID" "$BID" || true
TH=$(plutil -extract theme raw -o - "$PL"); SZ=$(plutil -extract size raw -o - "$PL")
SN=$(plutil -extract updateSnoozedUntil raw -o - "$PL")
echo "Sau nâng cấp: theme=$TH size=$SZ updateSnoozedUntil=$SN"
[ "$TH" = "2" ] && [ "$SZ" = "3" ] && [ "${SN%%.*}" = "1893456000" ] || { echo "::error::R14 ca A: mất tùy chọn giao diện"; exit 1; }

echo "== Ca B: 0.2-dev → mới (dữ liệu Người thân)"
xcrun simctl uninstall "$UDID" "$BID" || true
xcrun simctl install "$UDID" "$OLD2_APP"
xcrun simctl launch "$UDID" "$BID" -lvhvSeed demo -lvhvToday 2026-11-07 -lvhvNoNotif 1 -lvhvScreen profiles
sleep 20
xcrun simctl io "$UDID" screenshot "$SHOTS/upgrade-b-02dev-old.png"
xcrun simctl terminate "$UDID" "$BID" || true
AS="$(container)/Library/Application Support"
test -f "$AS/LichVanHoaViet/lvhv.sqlite" || { echo "::error::bản cũ không tạo được kho"; ls -laR "$AS" || true; exit 1; }
rm -rf "$AS/LichVanHoaViet-before"
cp -R "$AS/LichVanHoaViet" "$AS/LichVanHoaViet-before"
ls -la "$AS/LichVanHoaViet" "$AS/LichVanHoaViet/photos"
set -o pipefail
(cd "$A" && TEST_RUNNER_LVHV_UPGRADE=1 TEST_RUNNER_SCREENSHOT_DIR="$SHOTS" \
  xcodebuild -project LichVanHoaViet.xcodeproj -scheme LichViet-iOS -configuration Debug \
    -destination "id=$UDID" -derivedDataPath "$T/dd-new" -resultBundlePath "$T/upgrade.xcresult" \
    -only-testing:LichVietUITests-iOS/PeopleUpgradeTests CODE_SIGNING_ALLOWED=NO CODE_SIGNING_REQUIRED=NO \
    test-without-building) 2>&1 | tee "$T/upgrade-test.log" | grep -E "R14|persons=|pending=|same_as_before|migration=|Test Case|error:|Executed" || true
grep -q "Test Case '-\[LichVietUITests_iOS.PeopleUpgradeTests testDataSurvivesUpgrade\]' passed" "$T/upgrade-test.log" \
  || { echo "::error::R14 ca B KHÔNG ĐẠT"; tail -80 "$T/upgrade-test.log"; exit 1; }
AS="$(container)/Library/Application Support"
test -f "$AS/LichVanHoaViet/lvhv.sqlite" || { echo "::error::kho mất sau nâng cấp"; exit 1; }
ls -la "$AS/LichVanHoaViet"

echo "== Ca C: 0.2-dev có ảnh giấy báo tử hỏng → mới: chuyển đổi lỗi, chỉ đọc, không mất dữ liệu"
xcrun simctl uninstall "$UDID" "$BID" || true
xcrun simctl install "$UDID" "$OLD2_APP"
xcrun simctl launch "$UDID" "$BID" -lvhvSeed demo -lvhvToday 2026-11-07 -lvhvNoNotif 1 -lvhvScreen profiles
sleep 20
xcrun simctl terminate "$UDID" "$BID" || true
AS="$(container)/Library/Application Support"
# Làm hỏng byte mọi ảnh (ảnh giấy báo tử phải qua kiểm B6 khi chuyển 1 → 2).
for f in "$AS/LichVanHoaViet/photos/"*; do printf '<html>khong phai anh</html>' > "$f"; done
NPH=$(ls "$AS/LichVanHoaViet/photos" | wc -l | tr -d ' ')
SUM_BEFORE=$(shasum "$AS/LichVanHoaViet/lvhv.sqlite" | cut -d' ' -f1)
(cd "$A" && TEST_RUNNER_LVHV_UPGRADE_FAIL=1 TEST_RUNNER_SCREENSHOT_DIR="$SHOTS" \
  xcodebuild -project LichVanHoaViet.xcodeproj -scheme LichViet-iOS -configuration Debug \
    -destination "id=$UDID" -derivedDataPath "$T/dd-new" -resultBundlePath "$T/upgrade-fail.xcresult" \
    -only-testing:LichVietUITests-iOS/PeopleUpgradeTests/testFailedMigrationIsReadOnly \
    CODE_SIGNING_ALLOWED=NO CODE_SIGNING_REQUIRED=NO test-without-building) 2>&1 | tee "$T/upgrade-fail.log" \
  | grep -E "R14|persons=|migration=|Test Case|error:|Executed" || true
grep -q "testFailedMigrationIsReadOnly\]' passed" "$T/upgrade-fail.log" \
  || { echo "::error::R14 ca C KHÔNG ĐẠT"; tail -80 "$T/upgrade-fail.log"; exit 1; }
AS="$(container)/Library/Application Support"
ls -la "$AS/LichVanHoaViet" "$AS/LichVanHoaViet/photos"
test -f "$AS/LichVanHoaViet/lvhv.sqlite" || { echo "::error::ca C: kho mất"; exit 1; }
[ "$(ls "$AS/LichVanHoaViet/photos" | wc -l | tr -d ' ')" = "$NPH" ] || { echo "::error::ca C: tệp ảnh bị xóa"; exit 1; }
ls "$AS/LichVanHoaViet" | grep -q "pre-v2.bak" && echo "Có bản sao trước chuyển đổi" || echo "::warning::không thấy lvhv.sqlite.pre-v2.bak"
SUM_AFTER=$(shasum "$AS/LichVanHoaViet/lvhv.sqlite" | cut -d' ' -f1)
echo "sha1 kho trước/sau ca lỗi: $SUM_BEFORE / $SUM_AFTER"
echo "R14 (Apple, mô phỏng): ĐẠT ca A, B và C"
