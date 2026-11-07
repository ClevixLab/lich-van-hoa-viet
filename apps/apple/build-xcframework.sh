#!/usr/bin/env bash
# Dựng lvhv-ffi thành LvhvCore.xcframework (iOS thiết bị, iOS mô phỏng arm64+x86_64, macOS
# arm64+x86_64) và sinh mã Swift UniFFI vào Generated/.
#
# Cần: macOS + Xcode 15/16 (xcodebuild, lipo), Rust qua rustup. Script tự thêm các target Rust.
# Chạy:  apps/apple/build-xcframework.sh          (bản release)
#        apps/apple/build-xcframework.sh --debug  (nhanh hơn, cho lúc phát triển)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
PROFILE="release"
[[ "${1:-}" == "--debug" ]] && PROFILE="debug"
PFLAG=()
[[ "$PROFILE" == "release" ]] && PFLAG=(--release)

TARGETS=(aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios aarch64-apple-darwin x86_64-apple-darwin)
rustup target add "${TARGETS[@]}"

export IPHONEOS_DEPLOYMENT_TARGET=16.0
export MACOSX_DEPLOYMENT_TARGET=13.0

cd "$ROOT"
for t in "${TARGETS[@]}"; do
  echo "== cargo build --target $t ($PROFILE)"
  cargo build -p lvhv-ffi --lib --target "$t" "${PFLAG[@]}"
done

# Mã Swift + tiêu đề C + modulemap, sinh từ thư viện macOS (metadata giống mọi target).
cargo build -p lvhv-ffi --features bindgen-cli --bin uniffi-bindgen
GEN="$HERE/build/uniffi"
rm -rf "$GEN"
mkdir -p "$GEN"
"$ROOT/target/debug/uniffi-bindgen" generate \
  --library "$ROOT/target/aarch64-apple-darwin/$PROFILE/liblvhv_ffi.a" \
  --language swift --out-dir "$GEN" --config "$ROOT/ffi/uniffi.toml"

mkdir -p "$HERE/Generated"
cp "$GEN/LvhvCore.swift" "$HERE/Generated/LvhvCore.swift"

# Thư mục headers dùng chung cho mọi lát (slice): tiêu đề + module.modulemap (tên bắt buộc).
HDR="$HERE/build/headers"
rm -rf "$HDR"
mkdir -p "$HDR"
cp "$GEN/lvhv_ffiFFI.h" "$HDR/"
cp "$GEN/lvhv_ffiFFI.modulemap" "$HDR/module.modulemap"

# Gộp các kiến trúc cùng nền tảng bằng lipo.
LIBS="$HERE/build/libs"
rm -rf "$LIBS"
mkdir -p "$LIBS/ios" "$LIBS/ios-sim" "$LIBS/macos"
cp "$ROOT/target/aarch64-apple-ios/$PROFILE/liblvhv_ffi.a" "$LIBS/ios/liblvhv_ffi.a"
lipo -create \
  "$ROOT/target/aarch64-apple-ios-sim/$PROFILE/liblvhv_ffi.a" \
  "$ROOT/target/x86_64-apple-ios/$PROFILE/liblvhv_ffi.a" \
  -output "$LIBS/ios-sim/liblvhv_ffi.a"
lipo -create \
  "$ROOT/target/aarch64-apple-darwin/$PROFILE/liblvhv_ffi.a" \
  "$ROOT/target/x86_64-apple-darwin/$PROFILE/liblvhv_ffi.a" \
  -output "$LIBS/macos/liblvhv_ffi.a"

XCF="$HERE/build/LvhvCore.xcframework"
rm -rf "$XCF"
xcodebuild -create-xcframework \
  -library "$LIBS/ios/liblvhv_ffi.a" -headers "$HDR" \
  -library "$LIBS/ios-sim/liblvhv_ffi.a" -headers "$HDR" \
  -library "$LIBS/macos/liblvhv_ffi.a" -headers "$HDR" \
  -output "$XCF"

"$HERE/gen-version.sh"
"$HERE/gen-trays.sh"

echo "Xong: $XCF"
echo "      $HERE/Generated/LvhvCore.swift"
echo "      $HERE/Generated/Version.xcconfig"
