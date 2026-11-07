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
# Ca chuyển đổi lỗi (trigger chặn ghi `meta`): quay lui, chỉ đọc, kho giống hệt từng dòng/từng ảnh, tệp sao lưu mở lại
#   được và nhập vào kho trống cho đúng trạng thái kỳ vọng.
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
# `adb exec-in` có thể trả về trước khi bên máy ghi xong, nên chờ tới khi kích thước khớp (không thì ứng dụng có thể mở một
# kho chép dở).
put() {
  local want got
  want="$(wc -c < "$1" | tr -d ' ')"
  adb exec-in "run-as $PKG sh -c 'mkdir -p \$(dirname $2) && cat > $2'" < "$1"
  for _ in $(seq 1 60); do
    got="$(adb shell run-as "$PKG" sh -c "'wc -c < $2'" 2>/dev/null | tr -d '\r ')"
    [ "$got" = "$want" ] && return 0
    sleep 0.5
  done
  fail "chép $1 → $2: $got/$want byte"
}
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
# Lõi chỉ có tùy chọn buộc lỗi ở API Rust (OpenOptions::force_migration_failure, không qua FFI). Như Windows (upgrade.rs):
# thêm trigger chặn ghi bảng `meta` → bước chuyển 1 → 2 lỗi thật ở câu SQL cuối giao dịch, dữ liệu cũ vẫn hợp lệ.
BAD="$(mktemp -d)"; cp -r "$FIX/store/." "$BAD/"
sqlite3 "$BAD/lvhv.sqlite" "CREATE TRIGGER kiem_dinh_loi_chuyen_doi BEFORE INSERT ON meta BEGIN SELECT RAISE(ABORT, 'R14 kiem dinh: loi chuyen doi'); END;" \
  || fail "không sửa được bản sao bộ mẫu"
# Nội dung từng dòng mọi bảng + phiên bản (như db_content của Windows) và băm từng ảnh — trước khi mở.
content() {
  local db="$1"
  for t in settings photo person event_type event event_person meta; do echo "## $t"; sqlite3 "$db" "SELECT * FROM $t ORDER BY 1;"; done
  echo "user_version=$(sqlite3 "$db" 'PRAGMA user_version;')"
}
content "$BAD/lvhv.sqlite" > "$BAD/before.txt"
(cd "$BAD/photos" && sha256sum * | sort) > "$BAD/photos-before.txt"
seed_store "$BAD"
put "$EXPECTED" files/r14-expected.json
R="$(self_check verify-readonly)"; echo "$R"
case "$R" in OK*) ;; *) fail "chỉ đọc: $R" ;; esac
run_main
R="$(self_check verify-readonly)"; echo "lần mở thứ hai: $R"
case "$R" in OK*) ;; *) fail "chỉ đọc (lần mở thứ hai): $R" ;; esac
# Kho cũ trên máy giống hệt trước khi mở: từng dòng mọi bảng, từng ảnh; không có tệp đính kèm mồ côi.
mkdir -p "$BAD/after"
adb exec-out run-as "$PKG" cat files/lvhv.sqlite > "$BAD/after/lvhv.sqlite"
adb exec-out "run-as $PKG sh -c 'cat files/lvhv.sqlite-wal 2>/dev/null'" > "$BAD/after/lvhv.sqlite-wal"
[ -s "$BAD/after/lvhv.sqlite-wal" ] || rm -f "$BAD/after/lvhv.sqlite-wal"
content "$BAD/after/lvhv.sqlite" > "$BAD/after.txt"
if diff -u "$BAD/before.txt" "$BAD/after.txt"; then echo "kho cũ: $(wc -l < "$BAD/before.txt") dòng giống hệt"; else fail "chỉ đọc: nội dung kho đổi sau khi mở"; fi
adb exec-out "run-as $PKG sh -c 'cd files/photos && sha256sum *'" | tr -d '\r' | sort > "$BAD/photos-after.txt"
cat "$BAD/photos-after.txt"
[ "$(wc -l < "$BAD/photos-after.txt")" = "$(ls "$BAD/photos" | wc -l)" ] || fail "chỉ đọc: không đọc được danh sách ảnh trên máy"
diff -u "$BAD/photos-before.txt" "$BAD/photos-after.txt" || fail "chỉ đọc: ảnh đổi hoặc mất"
adb shell run-as "$PKG" ls -la files files/attachments || true
# Giải mã thêm tệp sao lưu máy xuất bằng công cụ của lõi trên máy dựng (cùng mã nguồn) — nhập vào kho trống.
adb exec-out run-as "$PKG" cat files/r14-export.lvhv > "$BAD/r14-export.lvhv" || true
if [ -s "$BAD/r14-export.lvhv" ]; then
  OUTJ="$(cargo run -q -p lvhv-ffi --bin lvhv-store-tool -- import "$BAD/r14-export.lvhv" "mật khẩu thử nghiệm" --replace --into "$BAD/import")"
  echo "$OUTJ" | cut -c1-400
  echo "$OUTJ" | grep -q '"ok":true' || fail "chỉ đọc: công cụ của lõi không nhập được tệp sao lưu máy xuất"
else
  fail "chỉ đọc: không lấy được tệp sao lưu máy xuất"
fi
N_ATT="$(adb shell "run-as $PKG sh -c 'ls files/attachments | wc -l'" | tr -d '\r ')"
echo "tệp trong files/attachments: $N_ATT"
[ "${N_ATT:-0}" = "0" ] || fail "chỉ đọc: còn $N_ATT tệp đính kèm mồ côi sau chuyển đổi lỗi"
end

# docs/05 §L: mở thẻ "Giới thiệu" trên ứng dụng thật. Bản 0.1.4 đã phát hành (d9650c7) sập vì painterResource không vẽ
# được biểu tượng adaptive-icon — ghi lại để đối chiếu (không làm đỏ CI); bản mới KHÔNG được sập.
about_check() {
  adb uninstall "$PKG" >/dev/null 2>&1 || true
  adb install "$1" >/dev/null || { echo "INSTALL-FAIL"; return; }
  adb logcat -c
  adb shell am start -W -n "$MAIN" >/dev/null 2>&1
  sleep 6
  adb shell uiautomator dump /sdcard/ui.xml >/dev/null 2>&1
  local b
  b="$(adb shell cat /sdcard/ui.xml | tr '>' '\n' | grep -E '(text|content-desc)="Giới thiệu"' | head -1 \
    | sed -n 's/.*bounds="\[\([0-9]*\),\([0-9]*\)\]\[\([0-9]*\),\([0-9]*\)\]".*/\1 \2 \3 \4/p')"
  if [ -z "$b" ]; then echo "NO-TAB"; return; fi
  set -- $b
  adb shell input tap $(( ($1 + $3) / 2 )) $(( ($2 + $4) / 2 ))
  sleep 5
  if adb logcat -d -s AndroidRuntime:E | grep -q "FATAL EXCEPTION"; then
    adb logcat -d -s AndroidRuntime:E | grep -m3 -E "Exception|at .*AboutScreen" >&2
    echo "CRASH"
  else
    echo "OK"
  fi
}
say "Thẻ Giới thiệu không sập (docs/05 §L)"
OLD014="$(for a in "$@"; do case "$a" in *v0.1.4*) echo "$a";; esac; done | head -1)"
if [ -n "$OLD014" ]; then echo "bản 0.1.4 đã phát hành: $(about_check "$OLD014") (kỳ vọng CRASH — lỗi đã phát hành)"; fi
R="$(about_check "$NEW")"; echo "bản mới: $R"
[ "$R" = "OK" ] || fail "thẻ Giới thiệu của bản mới: $R"
end

adb logcat -d -s LvhvR14:I LvhvNhac:W AndroidRuntime:E | tail -50
if [ "$FAILS" -gt 0 ]; then echo "R14: KHÔNG ĐẠT ($FAILS lỗi)"; exit 1; fi
echo "R14 (Android): ĐẠT — mọi ca"
