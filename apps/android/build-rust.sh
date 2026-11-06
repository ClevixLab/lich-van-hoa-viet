#!/usr/bin/env bash
# Dựng thư viện Rust (lvhv-ffi) cho Android và sinh mã Kotlin UniFFI.
#
# Cần: Rust (rustup), Android NDK (ANDROID_NDK_HOME hoặc ANDROID_NDK_ROOT), cargo-ndk 3.5.4.
#   rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
#   cargo install cargo-ndk --version 3.5.4 --locked
# Chạy từ bất kỳ đâu:  apps/android/build-rust.sh   (thêm --debug để dựng bản debug của Rust)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
APP="$HERE/app"
PROFILE="release"
[[ "${1:-}" == "--debug" ]] && PROFILE="debug"

NDK="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}"
if [[ -z "$NDK" || ! -d "$NDK" ]]; then
  echo "Lỗi: chưa đặt ANDROID_NDK_HOME (thư mục NDK, ví dụ \$ANDROID_HOME/ndk/27.2.12479018)." >&2
  exit 1
fi
export ANDROID_NDK_HOME="$NDK"

cd "$ROOT"
PFLAG=()
[[ "$PROFILE" == "release" ]] && PFLAG=(--release)

# 1) Thư viện .so cho 3 ABI. --platform 26 = minSdk. Căn trang 16 KB cho Android 15+.
RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-Wl,-z,max-page-size=16384" \
cargo ndk --platform 26 \
  -t arm64-v8a -t armeabi-v7a -t x86_64 \
  -o "$APP/rustJniLibs" \
  build -p lvhv-ffi --lib "${PFLAG[@]}"

# 2) Mã Kotlin UniFFI (chế độ --library, đọc metadata UNIFFI_META_* trong bảng ký hiệu).
# KHÔNG đọc từ .so Android: cargo-ndk strip thư viện nên mất metadata, uniffi-bindgen khi đó không
# sinh tệp nào mà vẫn thoát 0 (lỗi CI lần đầu). Dùng thư viện host (chưa strip) dựng từ cùng mã nguồn
# và cùng profile — giống ci.yml — nên checksum API khớp với .so Android.
cargo build -p lvhv-ffi --lib "${PFLAG[@]}"
cargo build -p lvhv-ffi --features bindgen-cli --bin uniffi-bindgen
HOSTLIB="$ROOT/target/$PROFILE/liblvhv_ffi.so"
[[ "$(uname -s)" == "Darwin" ]] && HOSTLIB="$ROOT/target/$PROFILE/liblvhv_ffi.dylib"
OUT="$APP/build/generated/uniffi/kotlin"
rm -rf "$OUT"
mkdir -p "$OUT"
"$ROOT/target/debug/uniffi-bindgen" generate \
  --library "$HOSTLIB" \
  --language kotlin --out-dir "$OUT" --config "$ROOT/ffi/uniffi.toml" --no-format
KT="$OUT/vn/clevix/lichviet/core/lvhv_ffi.kt"
if [[ ! -s "$KT" ]]; then
  echo "Lỗi: uniffi-bindgen không sinh $KT (thư viện $HOSTLIB không có metadata UniFFI?)." >&2
  exit 1
fi

echo "Xong: $(find "$APP/rustJniLibs" -name liblvhv_ffi.so | wc -l) thư viện .so; mã Kotlin ở $OUT"
