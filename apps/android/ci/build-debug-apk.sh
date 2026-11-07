#!/usr/bin/env bash
# Dựng APK debug (chỉ ABI x86_64 cho trình giả lập) của một bản mã nguồn bất kỳ — dùng cho kiểm thử nâng cấp R14
# (docs/v0.2/04 Bổ sung B5). Mọi bản dựng trên cùng máy CI dùng chung khóa debug (~/.android/debug.keystore), nên
# `adb install -r` bản mới đè lên bản cũ được như người dùng thật cập nhật.
#
#   build-debug-apk.sh <thư mục gốc kho của bản cần dựng> <tệp APK ra>
#
# Cần: ANDROID_NDK_HOME, cargo-ndk, gradle, Rust có target x86_64-linux-android.
set -euo pipefail
SRC="$(cd "$1" && pwd)"
OUT="$2"
APP="$SRC/apps/android/app"
cd "$SRC"

# 1) Thư viện Rust cho trình giả lập (x86_64). Bản cũ kiểm có .so arm64-v8a trước khi dựng (checkRustArtifacts):
#    chép tạm bản x86_64 vào chỗ đó — APK này chỉ chạy trên trình giả lập x86_64, không bao giờ nạp tệp arm64.
RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-Wl,-z,max-page-size=16384" \
  cargo ndk --platform 26 -t x86_64 -o "$APP/rustJniLibs" build -p lvhv-ffi --lib --release
mkdir -p "$APP/rustJniLibs/arm64-v8a" "$APP/rustJniLibs/armeabi-v7a"
cp -f "$APP/rustJniLibs/x86_64/liblvhv_ffi.so" "$APP/rustJniLibs/arm64-v8a/liblvhv_ffi.so"
cp -f "$APP/rustJniLibs/x86_64/liblvhv_ffi.so" "$APP/rustJniLibs/armeabi-v7a/liblvhv_ffi.so"

# 2) Mã Kotlin UniFFI từ thư viện máy dựng của CÙNG bản mã nguồn.
cargo build -p lvhv-ffi --lib --release
cargo build -p lvhv-ffi --features bindgen-cli --bin uniffi-bindgen
TGT="${CARGO_TARGET_DIR:-$SRC/target}"
GEN="$APP/build/generated/uniffi/kotlin"
rm -rf "$GEN" && mkdir -p "$GEN"
"$TGT/debug/uniffi-bindgen" generate --library "$TGT/release/liblvhv_ffi.so" \
  --language kotlin --out-dir "$GEN" --config "$SRC/ffi/uniffi.toml" --no-format
test -s "$GEN/vn/clevix/lichviet/core/lvhv_ffi.kt"

# 3) APK debug.
gradle --no-daemon -p "$SRC/apps/android" :app:assembleDebug
APK=$(find "$APP/build/outputs/apk/debug" -name "*.apk" | sort | head -1)
cp -f "$APK" "$OUT"
echo "Đã dựng $OUT từ $(git -C "$SRC" rev-parse --short HEAD)"
