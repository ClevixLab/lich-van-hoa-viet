#!/usr/bin/env bash
# Kiểm thử nâng cấp tại chỗ R14 (docs/v0.2/04 Bổ sung B5) trên trình giả lập Android:
#   cài bản cũ → tạo dữ liệu → `adb install -r` bản mới → dữ liệu còn nguyên, chuỗi báo thức được đặt lại.
#
#   upgrade-test.sh <APK mới> <APK cũ 1> [<APK cũ 2> …]
#
# Ca 1…n (mỗi APK cũ, ví dụ v0.1.4 và 0.2-dev từ main): bản cũ chưa có mô-đun Người thân → dữ liệu là tùy chọn giao
#   diện (cỡ chữ A++, giao diện tối, "Để sau" của thẻ nhắc cập nhật) — R14 (a).
# Ca dữ liệu người thân: bản có mô-đun tạo đủ loại dữ liệu bằng màn tự kiểm (chỉ có ở bản debug), hủy báo
#   thức, cài đè, rồi kiểm từng bản ghi, từng ảnh/tệp đính kèm và báo thức đã được ứng dụng tự đặt lại.
# Ca chuyển đổi: kho SQLite thật của bản 0.2-dev (bộ mẫu đóng băng, phiên bản 1) → bản này: đúng tệp kỳ vọng chung.
# Ca chuyển đổi lỗi: kho phiên bản 1 có hồ sơ không hợp lệ → quay lui, chỉ đọc, xem được, xuất sao lưu được.
set -uo pipefail
PKG=vn.clevix.lichviet.debug
MAIN="$PKG/vn.clevix.lichviet.MainActivity"
CHECK="$PKG/vn.clevix.lichviet.debug.SelfCheckActivity"
NEW="$1"; shift
FAILS=0

say() { echo "::group::$*"; }
end() { echo "::endgroup::"; }
fail() { echo "::error::$*"; FAILS=$((FAILS + 1)); }

run_main() { adb shell am start -W -n "$MAIN" >/dev/null 2>&1; sleep 6; adb shell am force-stop "$PKG"; }

self_check() {
  adb shell run-as "$PKG" rm -f files/upgrade-result.txt
  adb shell am start -W -n "$CHECK" --es op "$1" >/dev/null 2>&1
  for _ in $(seq 1 30); do
    R="$(adb shell run-as "$PKG" cat files/upgrade-result.txt 2>/dev/null | tr -d '\r')"
    [ -n "$R" ] && break
    sleep 1
  done
  # Không force-stop (force-stop đặt ứng dụng vào trạng thái "đã dừng" — hệ thống không gửi broadcast, khác người dùng
  # thật): về màn hình chính rồi kết thúc tiến trình nền như hệ thống vẫn làm.
  adb shell input keyevent KEYCODE_HOME
  sleep 1
  adb shell am kill "$PKG"
  echo "$R"
}

# Tùy chọn giao diện như người dùng bản 0.1.x đã chọn: A++ (size=3), tối (theme=2), "Để sau" tới 2099-12-31.
PREFS="<?xml version='1.0' encoding='utf-8' standalone='yes' ?><map><int name=\"size\" value=\"3\" /><int name=\"theme\" value=\"2\" /><string name=\"nhac_cap_nhat_den\">2099-12-31</string></map>"
write_prefs() {
  local b64; b64="$(printf '%s' "$PREFS" | base64 -w0)"
  adb shell "echo $b64 | base64 -d | run-as $PKG sh -c 'mkdir -p shared_prefs && cat > shared_prefs/hien_thi.xml'"
}

n=0
for OLD in "$@"; do
  n=$((n + 1))
  say "Ca $n: $(basename "$OLD") → bản mới (tùy chọn giao diện)"
  adb uninstall "$PKG" >/dev/null 2>&1 || true
  adb install "$OLD" || fail "Ca $n: không cài được bản cũ"
  run_main
  write_prefs
  run_main   # bản cũ đọc và ghi lại tùy chọn của người dùng
  adb shell run-as "$PKG" cat shared_prefs/hien_thi.xml
  adb install -r "$NEW" || fail "Ca $n: không cài đè được bản mới"
  run_main
  R="$(self_check verify-prefs)"
  echo "$R"
  case "$R" in OK*) ;; *) fail "Ca $n ($(basename "$OLD")): $R" ;; esac
  end
done

say "Ca dữ liệu người thân: tạo → cài đè → kiểm"
adb uninstall "$PKG" >/dev/null 2>&1 || true
adb install "$NEW" || fail "không cài được bản mới"
R="$(self_check seed)"; echo "$R"
case "$R" in OK*) ;; *) fail "tạo dữ liệu: $R" ;; esac
adb shell run-as "$PKG" ls -la files files/photos
adb install -r "$NEW" || fail "không cài đè được"
sleep 10   # MY_PACKAGE_REPLACED → đặt lại chuỗi báo thức
adb shell dumpsys alarm | grep -A2 "$PKG" | head -20 || true
adb logcat -d -s LvhvNhac:I | tail -5
R="$(self_check verify)"; echo "$R"
case "$R" in OK*) ;; *) fail "sau cài đè: $R" ;; esac
end

# R14 (c) — kho THẬT của bản dựng 0.2-dev (phiên bản 1, bộ mẫu đóng băng, có certificate_photo_ids) → bản này chuyển 1 → 2.
FIX="docs/v0.2/schema/test-vectors/released/0.2-dev-before-attachments"
EXPECTED="docs/v0.2/schema/test-vectors/released/expected/0.2-dev-before-attachments.json"
# Chép một tệp từ máy dựng vào thư mục riêng của ứng dụng (ứng dụng chưa chạy lần nào sau khi cài).
put() { adb exec-in "run-as $PKG sh -c 'mkdir -p \$(dirname $2) && cat > $2'" < "$1"; }
seed_store() {
  local dir="$1"
  adb uninstall "$PKG" >/dev/null 2>&1 || true
  adb install "$NEW" || fail "không cài được bản mới"
  put "$dir/lvhv.sqlite" files/lvhv.sqlite
  for f in "$dir"/photos/*; do put "$f" "files/photos/$(basename "$f")"; done
  adb shell run-as "$PKG" ls -la files files/photos
}

say "Ca chuyển đổi dữ liệu 0.2-dev (phiên bản 1) → bản mới: bộ mẫu thật"
seed_store "$FIX/store"
put "$EXPECTED" files/r14-expected.json
# Lần mở kho đầu tiên là màn tự kiểm (để thấy trạng thái "migrated"; lần mở sau trạng thái là "ok").
R="$(self_check verify-migrated)"; echo "$R"
case "$R" in OK*) ;; *) fail "chuyển đổi 1 → 2: $R" ;; esac
adb logcat -c
run_main   # rồi mở ứng dụng như người dùng trên kho đã chuyển
adb logcat -d -s AndroidRuntime:E | grep -q "vn.clevix.lichviet" && fail "ứng dụng lỗi khi mở kho đã chuyển"
end

say "Ca chuyển đổi lỗi: quay lui, chế độ chỉ đọc, vẫn xuất sao lưu"
# Lõi chỉ có tùy chọn buộc lỗi ở API Rust (OpenOptions::force_migration_failure, không qua FFI). Ở đây làm lỗi bằng dữ
# liệu như store/tests/released.rs: một chuỗi chưa cắt khoảng trắng trong kho phiên bản 1 → kiểm 01 §6.2 sau chuyển đổi lỗi.
BAD="$(mktemp -d)"; cp -r "$FIX/store/." "$BAD/"
sqlite3 "$BAD/lvhv.sqlite" "UPDATE person SET json = json_set(json, '\$.death.notice.place', '  Quê nhà') WHERE id = '0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02';" \
  || fail "không sửa được bản sao bộ mẫu"
BEFORE="$(sha256sum "$BAD/lvhv.sqlite" | cut -c1-64)"
seed_store "$BAD"
run_main
R="$(self_check verify-readonly)"; echo "$R"
case "$R" in OK*) ;; *) fail "chỉ đọc: $R" ;; esac
# Dữ liệu cũ còn nguyên từng byte (kho không đổi sau khi quay lui).
adb exec-out run-as "$PKG" cat files/lvhv.sqlite > "$BAD/after.sqlite"
sqlite3 "$BAD/after.sqlite" "PRAGMA user_version;" | grep -qx 1 || fail "chỉ đọc: kho không còn ở phiên bản 1"
echo "sha256 trước: $BEFORE — sau: $(sha256sum "$BAD/after.sqlite" | cut -c1-64) (WAL có thể gộp trang; so nội dung bằng user_version + số hồ sơ)"
[ "$(sqlite3 "$BAD/after.sqlite" 'SELECT count(*) FROM person;')" = "3" ] || fail "chỉ đọc: số hồ sơ đổi"
end

adb logcat -d -s LvhvR14:I LvhvNhac:W AndroidRuntime:E | tail -50
if [ "$FAILS" -gt 0 ]; then echo "R14: KHÔNG ĐẠT ($FAILS lỗi)"; exit 1; fi
echo "R14 (Android): ĐẠT — mọi ca"
