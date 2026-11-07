# Dựng ứng dụng native — Android, iOS/macOS, Windows

Trạng thái: bản 0.1 (A6/A7/A8/A11), 2026-11-06. Áp dụng cho `ffi/`, `apps/android/`, `apps/apple/`,
`apps/windows/`, `.github/workflows/`.

> **Đọc mục 8 trước.** Môi trường soạn mã (Linux) không có Android SDK, Xcode hay công cụ Windows; ứng dụng
> Android, Apple và gói Windows chỉ được dựng trên CI (xanh từ 2026-11-06, mục 8.1). Chưa chạy thử trên thiết bị thật.

## 1. Kiến trúc

```
core/ (lvhv-core) + ngochap/ (lvhv-ngochap)       lõi Rust, không unsafe
      │
ffi/ (lvhv-ffi, UniFFI 0.29.4)                     JSON + vài kiểu đơn giản, lỗi tiếng Việt
      ├── cdylib liblvhv_ffi.so ──► Android: JNA 5.17.0 + mã Kotlin UniFFI + Jetpack Compose
      ├── staticlib liblvhv_ffi.a ─► iOS/macOS: LvhvCore.xcframework + mã Swift UniFFI + SwiftUI
      └── crate Rust ──────────────► Windows: Slint 1.8.0 (Rust thuần, không qua FFI C)
```

Năm sinh ở mục Chọn ngày tốt là **năm âm lịch** (năm tuổi); màn hình ghi rõ "sinh trước Tết thì lấy năm
trước". FFI có `lunar_year_of_birth` nếu sau này muốn nhập ngày sinh đầy đủ.

Mọi nền tảng gọi cùng một lõi, nên số liệu (ngày âm, Can Chi, sao, điểm, ca dao) giống hệt nhau. Màn
hình và chữ chép theo PWA: Tờ lịch (vuốt/phím ← → để lật ngày), bảng tháng "Chọn ngày", Chọn ngày tốt,
Đổi ngày, Giới thiệu (kèm Nguồn quy tắc và ghi công ca dao CC BY-SA 4.0, phông OFL).

Không ứng dụng nào dùng mạng: Android không xin quyền nào (kể cả `INTERNET`); macOS chạy trong App
Sandbox không có quyền mạng; iOS không gọi mạng; Windows không có mã mạng.

## 2. Phiên bản được ghim

| Thành phần | Phiên bản | Ghi chú |
|---|---|---|
| Rust | stable (CI: `dtolnay/rust-toolchain@stable`) | máy soạn mã: rustc 1.97.0 |
| UniFFI | `=0.29.4` | kiểm tra tồn tại bằng `cargo info` |
| serde / serde_json | 1.0.228 / 1.0.145 trở lên | |
| Android Gradle Plugin | 8.7.3 | cần Gradle ≥ 8.9 |
| Gradle | 8.11.1 | qua `gradle/actions/setup-gradle@v4` |
| Kotlin + Compose compiler plugin | 2.0.21 | |
| Compose BOM | 2024.12.01 | Material3 1.3.x |
| activity-compose / core-ktx / lifecycle | 1.9.3 / 1.15.0 / 2.8.7 | |
| JNA | 5.17.0 (`@aar`) | từ 5.16.0, `libjnidispatch.so` căn trang 16 KB (đã kiểm: 5.15.0 = 64 KB, 5.16/5.17 = 16 KB) |
| Android NDK | r27c | qua `nttld/setup-ndk@v1` |
| cargo-ndk | 3.5.4 | |
| JDK | Temurin 17 | |
| compileSdk / targetSdk / minSdk | 35 / 35 / 26 | |
| Xcode | 16.x trên `macos-15` | iOS 16+, macOS 13+ |
| XcodeGen | 2.42.0 | tải bản phát hành; dự phòng `brew install xcodegen` |
| Slint | `=1.8.0` (style fluent, backend winit, renderer **Skia** + software) | Skia: Direct3D 12 trên Windows, tự lùi về Skia phần mềm; thư viện dựng sẵn tải từ rust-skia/skia-binaries lúc biên dịch |
| winresource | `=0.1.19` | nhúng biểu tượng/thông tin vào .exe |
| cargo-wix / WiX | 0.3.9 / 3.14 | WiX 3.14 có sẵn trên `windows-2022` |
| GitHub Actions | checkout@v4, setup-java@v4, setup-node@v4, upload-artifact@v4, setup-gradle@v4, setup-android@v3, setup-ndk@v1, rust-cache@v2, install-action@v2, cargo-deny-action@v2, configure-pages@v5, upload-pages-artifact@v3, deploy-pages@v4 | đã kiểm tra mỗi thẻ có `action.yml` |

Sinh mã UniFFI: trong `ffi/README.md`.

## 3. Dựng trên máy

### 3.1 Lõi + FFI (mọi hệ điều hành)

```bash
cargo test --workspace            # core, ngochap, ffi
cargo build -p lvhv-ffi --release
```

### 3.2 Android

Cần: Android Studio Ladybug (2024.2) trở lên hoặc SDK dòng lệnh; NDK r27c; JDK 17; Rust.

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo install cargo-ndk --version 3.5.4 --locked
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/27.2.12479018
apps/android/build-rust.sh                  # .so cho 3 ABI + mã Kotlin vào app/build/generated/uniffi
cd apps/android
gradle wrapper --gradle-version 8.11.1      # chỉ lần đầu: sinh gradlew + gradle-wrapper.jar
./gradlew :app:assembleDebug                # hoặc mở apps/android trong Android Studio
```

Kho mã **không có** `gradle-wrapper.jar` (không tải được trong môi trường soạn mã); chỉ có
`gradle/wrapper/gradle-wrapper.properties`. CI cài Gradle 8.11.1 bằng `setup-gradle`. Chạy
`gradle wrapper` như trên một lần rồi commit `gradlew`, `gradlew.bat`, `gradle/wrapper/gradle-wrapper.jar`
nếu muốn dùng `./gradlew`.

Ký bản phát hành trên máy: tạo `apps/android/keystore.properties` (đã nằm trong `.gitignore`):

```
storeFile=C:/Users/<tên>/ClevixKeys/lichviet-upload.jks
storePassword=...
keyAlias=lichviet-upload
keyPassword=...
```

hoặc đặt biến môi trường `LVHV_KEYSTORE_FILE`, `LVHV_KEYSTORE_PASSWORD`, `LVHV_KEY_ALIAS`,
`LVHV_KEY_PASSWORD`. Thiếu một giá trị thì bản release **không ký** (vẫn dựng được, không cài lên máy
thật được).

### 3.3 iOS / macOS (cần máy Mac)

Cần: macOS 14+, Xcode 16, Rust, XcodeGen (`brew install xcodegen`).

```bash
apps/apple/build-xcframework.sh     # build/LvhvCore.xcframework + Generated/LvhvCore.swift
cd apps/apple
export LVHV_TEAM_ID=ABCDE12345      # Team ID Apple Developer (để ký tự động)
xcodegen generate                   # sinh LichVanHoaViet.xcodeproj (không commit)
open LichVanHoaViet.xcodeproj
```

`DEVELOPMENT_TEAM` lấy từ `$(LVHV_TEAM_ID)`; Xcode đọc biến môi trường lúc dựng. Muốn cố định thì đặt
Team trong Signing & Capabilities của Xcode sau khi `xcodegen generate` (sẽ mất khi sinh lại).

### 3.4 Windows

Cần: Windows 10/11 x64, Rust (toolchain MSVC), Visual Studio Build Tools 2022 (mục "Desktop development
with C++"). Bộ cài MSI cần thêm WiX 3.14 (`winget install WiXToolset.WiXToolset`).

```powershell
powershell -ExecutionPolicy Bypass -File apps\windows\scripts\Build-Windows.ps1
```

Kết quả: `apps\windows\dist\LichViet-<VERSION>-x64.zip` (chạy thẳng `LichViet.exe`) và `.msi` nếu có WiX.
MSI cài theo người dùng (không cần quyền quản trị) vào `%LOCALAPPDATA%\Programs\LichVanHoaViet`, tạo lối
tắt trong Start Menu.

`lvhv-windows` có bảng `[workspace]` riêng (không nằm trong workspace gốc) để `cargo test` của lõi không
phải dựng Slint (~300 crate, vài phút). Crate này **dựng được cả trên Linux** (đã kiểm, mục 8); trên
Linux cần `libfontconfig1-dev libxkbcommon-dev libwayland-dev libx11-dev libgl1-mesa-dev`.

**Phông chữ và renderer (từ 0.1.3).** Mọi phông được nhúng vào `.exe` (`ui/fonts/`, khai báo bằng
`import "fonts/…ttf"` trong `app.slint`, Slint đăng ký lúc chạy): "LVHV Serif" (Lora rút gọn, Regular/
SemiBold/Bold/Italic) cho tiêu đề và số, "LVHV Sans" (Be Vietnam Pro rút gọn, Regular/SemiBold/Bold) cho
chữ thường — đặt bằng `default-font-family` của cửa sổ nên mọi chữ, kể cả ô nhập và hộp chọn, dùng cùng một
phông có đủ dấu tiếng Việt. Biểu tượng là SVG (`ui/icons/`), không dùng ký tự ‹ › ✕ ▦ ▲ ▼ (không có trong
phông nên máy phải mượn phông khác). Renderer là Skia (dựng chữ bằng HarfBuzz, làm mịn như trình duyệt):
bản 0.1.2 dùng FemtoVG (OpenGL) — chữ mảnh, nhòe ở tỉ lệ màn hình 125/150%, và chữ thường rơi về phông hệ
thống mặc định (không khai báo phông sans).

**Kiểm thử giao diện và chụp màn hình.** `cargo test` chạy cả `src/ui_tests.rs` (backend kiểm thử của
Slint, bấm chuột giả lập vào từng nút). `LichViet.exe --chup-man-hinh <thư mục>` mở lần lượt 11 màn hình,
chụp bằng renderer của ứng dụng rồi thoát; `windows.yml` chạy bước này và đính kèm ảnh (artifact
`windows-screens`). Trên Linux: `xvfb-run target/release/LichViet --chup-man-hinh /tmp/anh`, thêm
`SLINT_SCALE_FACTOR=2` để chụp ở 2×.

Vì sao Slint mà không dùng WebView2: một tệp `.exe` duy nhất (~12 MB, đo trên Linux), không phụ thuộc
WebView2 runtime, khởi động nhanh, đọc màn hình (UI Automation) qua AccessKit. RAM đo trên Linux/Xvfb
(mục 8): ~52 MB với renderer phần mềm, ~170 MB với femtovg/OpenGL qua llvmpipe (Mesa phần mềm — trên máy
có GPU thật thường thấp hơn). Một vỏ WebView2 (wry/Tauri) thường tốn 80–150 MB vì có thêm tiến trình
Edge. Chưa đo trên Windows thật: `windows.yml` in số đo Working Set vào `mem.txt`.

#### 3.4a Mô-đun "Người thân" trên Windows (v0.2 đợt 3, T8)

Đặc tả chung: `docs/v0.2/08-DAC-TA-GIAO-DIEN-NGUOI-THAN.md`. Mã: `apps/windows/ui/people.slint` (giao diện),
`src/people.rs` (nối giao diện với lõi), `src/platform.rs` (thư mục dữ liệu, múi giờ, hộp chọn tệp, thông báo),
`src/photo.rs` (ảnh). Ứng dụng gọi thẳng hàm Rust của `lvhv-ffi` (phần `v02.rs`, cũng là lớp mà Android/Apple dùng
qua UniFFI) — mọi tính toán (ngày giỗ, nhắc, xoay vòng 64, danh sách theo kỳ, .ics, sao lưu, kiểm dữ liệu) nằm
trong `lvhv-memorial`, `lvhv-store`, `lvhv-backup`. Giao diện chỉ hiển thị và kiểm độ dài/ô trống.

- **Dữ liệu:** `%LOCALAPPDATA%\ClevixLab\LichVanHoaViet\` (`lvhv.sqlite`, `photos\`). Không tài khoản, không mạng.
  Kho chỉ được tạo khi người dùng mở mục Người thân lần đầu; khi đã có kho, ứng dụng mở kho lúc khởi động để đặt
  lại thông báo. Tệp nhỏ `ui-restored-from.txt` ghi ngày của bản sao lưu vừa "Thay toàn bộ" (TK-16, §12h).
- **Ảnh (S9):** ảnh chân dung và ảnh giấy báo tử được giải mã, xoay theo EXIF, thu cạnh dài ≤ 1024 px rồi **mã hóa
  lại JPEG** (`image` 0.24.9, đã có trong cây phụ thuộc của Slint) — byte lưu không còn APP1/APP13 (kiểm thử
  `photo::tests`, `ui_tests_people`).
- **Thông báo đặt lịch (02 §7.5, R-WIN-1):** `ScheduledToastNotification` (WinRT) với AppUserModelID
  `ClevixLab.LichVanHoaViet`. Bộ cài `.msi` tạo lối tắt Start Menu mang thuộc tính `System.AppUserModel.ID`
  (`wix/main.wxs`); ứng dụng gọi `SetCurrentProcessExplicitAppUserModelID` lúc mở. Mỗi lần mở ứng dụng, lưu dữ
  liệu, đổi cài đặt hay khôi phục, ứng dụng đọc danh sách chờ của Windows, gọi `LvhvStore::reschedule_json` của lõi
  (giới hạn 64, thông báo nhắc mở ứng dụng) rồi gỡ/thêm/thay đúng theo kế hoạch. Thông báo đã đặt do Windows giữ và
  hiện kể cả khi ứng dụng đã đóng; **không** có tiến trình nền. Mặc định thông báo không ghi tên (O-9); tùy chọn
  "Hiện tên người trong thông báo" lưu theo máy (`lock_screen_names`), không vào sao lưu.
- **Bấm vào thông báo** mở đúng hồ sơ: thông báo kích hoạt URI `lichviet:nguoi/<id>?…` (chỉ UUID; id và nội dung của
  lõi cũng nằm trong URI để đối chiếu khi xoay vòng — MA3N-25-01); bộ cài đăng ký giao thức `lichviet` trong
  `HKCU\Software\Classes`. Ứng dụng đang mở thì Windows mở thêm một cửa sổ ở đúng hồ sơ.
- **Múi giờ:** độ lệch hiện tại + các lần chuyển giờ mùa hè trong 401 ngày tới (`GetTimeZoneInformationForYear`), như
  Android/Apple (MA3N-25-05).
- **Việc nặng** (Argon2id khi tạo/mở sao lưu, khôi phục, giải mã ảnh) chạy ở luồng phụ; ảnh > 25 MB bị từ chối trước khi
  đọc tệp (MA3N-25-06, -07).
- **Nâng cấp không mất dữ liệu (R14, 04 v0.2 Bổ sung B5):** dữ liệu không thuộc bộ cài — sổ Người thân ở
  `%LOCALAPPDATA%\ClevixLab\LichVanHoaViet`, tệp "Để sau" của bản 0.1.x ở `%LOCALAPPDATA%\Clevix\LichVanHoaViet`;
  MSI không có thành phần nào trỏ tới hai thư mục này nên cài đè, nâng cấp, gỡ cài đặt đều không xóa. `MajorUpgrade`
  có `AllowSameVersionUpgrades` (cùng số phiên bản vẫn là nâng cấp, không cài song song). Mở kho lỗi (chuyển đổi lược
  đồ hỏng — lõi lùi giao dịch, giữ bản sao trước chuyển đổi; hoặc kho của bản mới hơn) → mục Người thân hiện khung báo
  chỉ xem "Dữ liệu cũ của bạn vẫn còn nguyên…" với nút "Mở thư mục dữ liệu" để chép dự phòng; không thử tạo kho mới, không
  thử lại trong lần chạy đó. CI (bước R14 của `windows.yml`): cài `LichViet-0.1.4-x64.msi` từ GitHub Releases, ghi "Để
  sau", tạo sổ Người thân mẫu (`--tao-du-lieu-mau`, từ `.zip` của bản dựng này), cài MSI mới đè lên, chạy bản đã cài
  `--kiem-nang-cap` (so hồ sơ, sự kiện, loại sự kiện, cài đặt, hiện tên, băm SHA-256 từng ảnh, "Để sau"; đặt lại thông
  báo), gỡ cài đặt rồi so băm từng tệp trong thư mục dữ liệu; kết quả ở `apps/windows/screenshots/nang-cap.txt`.
  Bản Windows 0.1.4 không lưu cỡ chữ và sáng/tối giữa các lần mở, nên thứ duy nhất cần giữ từ 0.1.4 là "Để sau".
- **Giới hạn đã biết:**
  - bản `.zip` chạy không cài không có lối tắt mang AUMID → Windows có thể không hiện thông báo dù đặt được; giao diện ghi
    "Windows chưa xác nhận…" và khuyên cài `.msi`;
  - gỡ ứng dụng không xóa sổ Người thân (`%LOCALAPPDATA%\ClevixLab\LichVanHoaViet`) và không gỡ thông báo đang chờ — trang
    Giới thiệu ghi rõ cách xóa;
  - R5 Windows (máy thật, khởi động lại) **chưa thử** — cần chủ dự án thử trên Windows 11 (04 R5);
  - YC-2026-07: cách gọi / quan hệ viết hoa chữ đầu và gợi ý cách gọi theo người sống/mất **đã có**; ô Giới tính chỉ hiện
    khi lõi nhận `gioi_tinh` (hiện **chưa**); "Tệp đính kèm" (ảnh/PDF, 01 §3.8a) **chưa làm** — chờ lõi.
- **Kiểm thử:** `src/ui_tests_people.rs` bấm qua mọi luồng: L2 (ngày âm, tháng nhuận, các lỗi), nhắc (mốc, giờ 24
  giờ, tiên thường), ảnh, L11 tưởng niệm (dừng/thắp lại khói), L12 sửa (`rev`), L3 người đang sống + sinh nhật âm
  (TK-29), L4 sự kiện, US-M-07 đang sống → đã mất với thông tin báo tử (cùng `id`, B4), B4 biểu mẫu sự kiện, L5
  bốn loại kỳ × ba bộ lọc so với `events_in_period` của lõi, ‹ › Hôm nay, cài đặt O-9, L8 .ics (không chứa thông
  tin báo tử), L9/L10 sao lưu → xóa → khôi phục "Thay toàn bộ" giống hệt, sai mật khẩu, L7 thẻ nhắc hôm nay.
- **Ảnh chụp:** `LichViet.exe --chup-man-hinh <thư mục>` dùng kho tạm với dữ liệu mẫu; tên tệp
  `<màn hình>-<sang|toi>-<1x|2x>.png`. CI chụp ở 1× và 2× rồi commit vào `apps/windows/screenshots/` của nhánh PR
  (kèm `bo-nho.txt`, `thong-bao.txt`). `--do-bo-nho` mở Người thân + tưởng niệm với dữ liệu mẫu để đo Working Set;
  `--kiem-thong-bao` tự kiểm đặt/đọc/gỡ một thông báo thử (CI chạy sau khi cài `.msi`).

**Giấy phép Slint:** Slint phát hành theo GPL-3.0 **hoặc** "Slint Royalty-free License 2.0" **hoặc**
giấy phép thương mại. Bản Royalty-free miễn phí cho ứng dụng desktop, với điều kiện hiển thị ghi công
"Made with Slint" (màn hình Giới thiệu đã có). Chủ dự án nên đọc
https://slint.dev/terms-and-conditions trước khi phát hành để xác nhận điều kiện áp dụng.

### 3.5 Tạo khóa ký Android (Windows)

```powershell
powershell -ExecutionPolicy Bypass -File apps\windows\scripts\New-AndroidUploadKeystore.ps1
```

Không cần sửa script. Script tìm `keytool` (JDK hoặc Android Studio), hỏi mật khẩu (ẩn), tạo
`%USERPROFILE%\ClevixKeys\lichviet-upload.jks` (RSA 4096, 10 000 ngày, PKCS12), ghi tệp Base64, chép
vào clipboard, in 4 secret cần đặt và dấu vân tay SHA-256. Script không ghi đè khóa cũ. **Sao lưu tệp
.jks và mật khẩu.**

## 4. CI (GitHub Actions)

| Workflow | Khi chạy | Làm gì | Sản phẩm (Artifacts) |
|---|---|---|---|
| `ci.yml` | mọi push / PR | `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace --release`, kiểm S1 (không `unsafe` trong core), sinh lại mã UniFFI và so với `ffi/generated`, `cargo-deny check advisories`, sinh lại `core/out/*` bằng `lvhv-dump` rồi chạy `tsport` L7 (`npm test`), typecheck + build PWA, kiểm thử khói Playwright (Chromium) | `ffi-bindings`, `pwa-dist`, `pwa-smoke` |
| `android.yml` | push `main`, thẻ `v*`, PR | NDK r27c + cargo-ndk → `.so` 3 ABI; Gradle 8.11.1: unit test, lint, `assembleRelease` + `bundleRelease` (R8); kiểm APK không có `uses-permission` và có đủ `.so` | `android-release` (APK, AAB, mapping.txt), `android-lint-report` |
| `apple.yml` | push `main`, thẻ `v*`, PR | XCFramework 5 target, XcodeGen, `xcodebuild` iOS Simulator + iOS thiết bị + macOS (không ký), kiểm PrivacyInfo/phông/quyền mạng, chạy thử trên Simulator và chụp màn hình | `apple-unsigned` (zip .app iOS Simulator, zip .app macOS, ảnh) |
| `windows.yml` | push `main`, thẻ `v*`, PR | `cargo test` (gồm kiểm thử giao diện), `cargo build --release`, đo RAM, chụp 11 màn hình (`--chup-man-hinh`), ký Authenticode nếu có chứng chỉ, gói zip + MSI | `windows-x64`, `windows-screens` |
| `pwa-pages.yml` | thẻ `v*`, chạy tay | build PWA, đăng `pwa/dist` lên GitHub Pages | trang Pages |

Kho riêng tư (private): GitHub cấp phút chạy miễn phí có hạn; runner macOS tính **gấp 10 lần** phút
Linux. Mỗi lần `apple.yml` chạy khoảng 15–25 phút. Có thể đổi `on:` của `apple.yml` chỉ còn
`workflow_dispatch` và thẻ `v*` để tiết kiệm.

Bật Pages một lần: Settings → Pages → Source = **GitHub Actions**.

### 4.1 Số phiên bản và cập nhật (không dùng mạng)

**Nguồn duy nhất:** tệp `VERSION` ở gốc kho (dạng `X.Y.Z`, hiện `0.1.1`). Đổi phiên bản = sửa tệp này
(và dòng `version` trong `apps/windows/Cargo.toml`, vì Cargo không đọc được tệp ngoài — `build.rs` dừng
dựng nếu hai nơi khác nhau).

| Nền tảng | Đọc VERSION ở đâu | Số bản dựng | CI kiểm khớp |
|---|---|---|---|
| Android | `app/build.gradle.kts` đọc `../../VERSION` → `versionName`; `versionCode = X*10000 + Y*100 + Z` (Y, Z < 100). Không còn biến ghi đè `LVHV_VERSION_NAME/CODE` | `BuildConfig.BUILD_NUMBER` = `LVHV_BUILD_NUMBER` (CI: `github.run_number`), `BuildConfig.BUILD_DATE` = `LVHV_BUILD_DATE` | `android.yml`: `aapt2 dump badging` của APK phải có đúng `versionName` và `versionCode` |
| iOS/macOS | `gen-version.sh` (do `build-xcframework.sh` gọi) sinh `Generated/Version.xcconfig`; `project.yml` dùng làm `configFiles` → `MARKETING_VERSION`, `CURRENT_PROJECT_VERSION`, `LVHV_BUILD_DATE`, `APP_STORE_ID`; Info.plist: `CFBundleShortVersionString`, `CFBundleVersion`, `LVHVBuildDate`, `LVHVAppStoreID` | `CURRENT_PROJECT_VERSION` = `LVHV_BUILD_NUMBER` (CI: `github.run_number`, mặc định 1) | `apple.yml`: PlistBuddy đọc Info.plist của cả app iOS và macOS đã dựng |
| Windows | `build.rs` đọc `../../VERSION`, so với `CARGO_PKG_VERSION`; `winresource` ghi FileVersion/ProductVersion của `.exe`; cargo-wix lấy `Version` của MSI từ Cargo.toml (X.Y.Z số, đúng yêu cầu WiX) | `LVHV_BUILD_NUMBER` (CI: `github.run_number`) | `windows.yml`: so Cargo.toml với VERSION trước khi dựng; sau khi gói, đọc `ProductVersion` của `.exe` và bảng Property của MSI |

**Hiển thị:** màn hình Giới thiệu có mục "Phiên bản và cập nhật" với dòng "Phiên bản đang dùng: X.Y.Z"
(chỉ số phiên bản; số bản dựng và ngày dựng vẫn nằm trong tệp cài/Info.plist/versionCode và `version.json`
của PWA để dùng nội bộ, không hiển thị cho người dùng), nút **Kiểm tra cập nhật / Cập nhật** và câu giải thích "Ứng dụng không tự kết nối mạng để
bảo vệ quyền riêng tư…"; cuối trang có hai dòng "© 2026 Clevix Lab" (serif, © nhỏ và nhẹ hơn) và "Phiên bản X.Y.Z" (nhỏ, nhạt) (cùng
cách viết với chân trang PWA; ứng dụng native không có chân trang cố định nên dòng này nằm cuối Giới thiệu).

**Nút cập nhật** chỉ giao một địa chỉ cho hệ điều hành mở; ứng dụng không có quyền/mã mạng:

| Nền tảng | Địa chỉ mở | Cách mở |
|---|---|---|
| Android | `market://details?id=vn.clevix.lichviet` → nếu không có ứng dụng nào xử lý: `https://play.google.com/store/apps/details?id=vn.clevix.lichviet` → nếu vẫn không: `https://clevixlab.github.io/lich-van-hoa-viet/` | `Intent.ACTION_VIEW` (`update/Updates.kt`). Không cần khai báo `<queries>` vì gọi thẳng `startActivity` và bắt `ActivityNotFoundException` |
| iOS/macOS | Khi build setting `APP_STORE_ID` (chỉ chữ số) khác trống: `itms-apps://apps.apple.com/app/id<ID>`; khi trống (mặc định, vì chưa có bản ghi App Store): trang tải về | `UIApplication.open` / `NSWorkspace.open` (`Shared/Update.swift`); nếu mở `itms-apps` thất bại thì mở trang tải về. CI lấy `APP_STORE_ID` từ biến kho (Settings → Variables) `LVHV_APP_STORE_ID`, trống nếu chưa đặt |
| Windows | Khi `[package.metadata.lvhv] ms-store-id` trong `apps/windows/Cargo.toml` khác trống: `ms-windows-store://pdp/?productid=<ID>`; khi trống (mặc định): trang tải về | `rundll32 url.dll,FileProtocolHandler` (`src/update.rs`), tức trình duyệt/Store mặc định |

**Thẻ nhắc cập nhật** (ngoại tuyến): ứng dụng không biết bản mới nhất là bản nào, chỉ biết **ngày dựng**
của chính nó (ghi lúc dựng: `LVHV_BUILD_DATE`, mặc định ngày UTC hôm nay). Khi hôm nay cách ngày dựng
**hơn 60 ngày**, đầu màn hình Tờ lịch hiện thẻ "Phiên bản này đã phát hành hơn 2 tháng. Hãy kiểm tra cập
nhật để có dữ liệu và tính năng mới nhất." với [Kiểm tra cập nhật] [Để sau]. "Để sau" ẩn thẻ 30 ngày. Chỉ
lưu đúng một cờ giao diện: ngày hết hạn "Để sau" (Android: SharedPreferences `hien_thi`/`nhac_cap_nhat_den`;
Apple: UserDefaults `updateSnoozedUntil`, cùng lý do CA92.1; Windows: tệp
`%LOCALAPPDATA%\Clevix\LichVanHoaViet\nhac-cap-nhat.txt` chứa một ngày — trước đây bản Windows không
ghi gì). Hệ quả cần biết: người đang dùng bản mới nhất nhưng bản đó đã dựng hơn 60 ngày vẫn thấy thẻ nhắc
(mỗi 30 ngày nếu bấm "Để sau"); muốn tránh thì phát hành bản dựng mới ít nhất mỗi 2 tháng.

Android cài từ Google Play: mục Giới thiệu thêm câu "Bản cài từ Google Play cũng được cửa hàng tự cập
nhật…" khi `PackageManager.getInstallSourceInfo` (API 30+; API 26–29 dùng `getInstallerPackageName`) trả
`com.android.vending`; mọi lỗi → coi như không phải bản Play (không hiện câu này).

**Đã kiểm chứng (2026-11-06):** CI xanh ở commit `e8af91c` — android
[42179919068](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42179919068) (APK `versionName='0.1.1'`,
`versionCode='101'`; vẫn chỉ có quyền signature của androidx.core, không `INTERNET`; `UpdatePolicyTest` chạy
trong `testReleaseUnitTest`), apple [42179919050](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42179919050)
(Info.plist iOS + macOS: `0.1.1 (run_number)`, `LVHVBuildDate` đúng dạng; entitlements không có quyền mạng;
`APP_STORE_ID` trống), windows [42179919107](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42179919107)
(`.exe` ProductVersion/FileVersion `0.1.1`, MSI ProductVersion `0.1.1`). Trên Linux: `cargo test` của
`lvhv-windows` 6/6 ĐẠT (gồm 3 test `update.rs`), `clippy -D warnings` sạch; `UpdatePolicy.kt` biên dịch bằng
kotlinc 2.0.21 và chạy kiểm tra trên JVM. **CHƯA KIỂM:** bấm nút thật trên thiết bị (Intent `market://`
trên Android có/không có Play, `itms-apps`/Safari trên iOS, `NSWorkspace` trong App Sandbox macOS,
`rundll32 url.dll` trên Windows), giao diện thẻ nhắc trên màn hình thật (chưa có bản dựng > 60 ngày; muốn
xem thì dựng với `LVHV_BUILD_DATE` lùi hơn 60 ngày), đường `getInstallSourceInfo` với bản cài từ Play.

## 5. Chủ dự án cần cung cấp

| Nền tảng | Cần có | Ở đâu |
|---|---|---|
| Android | Tài khoản Google Play Console (phí một lần 25 USD); khóa tải lên (script mục 3.5); bật **Play App Signing** (Google giữ khóa ký ứng dụng) | play.google.com/console |
| iOS/macOS | Apple Developer Program (99 USD/năm) cho Clevix LLC (cần số D-U-N-S của Clevix LLC); Team ID; chứng chỉ "Apple Distribution"; App ID `vn.clevix.lichviet`; bản ghi ứng dụng trên App Store Connect | developer.apple.com, appstoreconnect.apple.com |
| Windows | Tùy chọn A (khuyên dùng): Microsoft Store (tài khoản nhà phát triển cho công ty; Store tự ký gói MSIX). Tùy chọn B: chứng chỉ ký mã Authenticode (OV/EV, .pfx) để ký `.exe`/`.msi` phát hành ngoài Store, tránh cảnh báo SmartScreen | partner.microsoft.com; nhà cấp chứng chỉ |

### 5.1 GitHub Secrets (Settings → Secrets and variables → Actions)

| Secret | Dùng bởi | Nội dung |
|---|---|---|
| `LVHV_ANDROID_KEYSTORE_BASE64` | android.yml | Base64 của `lichviet-upload.jks` (script mục 3.5 in sẵn) |
| `LVHV_ANDROID_KEYSTORE_PASSWORD` | android.yml | mật khẩu kho khóa |
| `LVHV_ANDROID_KEY_ALIAS` | android.yml | `lichviet-upload` |
| `LVHV_ANDROID_KEY_PASSWORD` | android.yml | mật khẩu khóa (script dùng cùng mật khẩu kho khóa) |
| `LVHV_WIN_CERT_PFX_BASE64` | windows.yml | Base64 của chứng chỉ ký mã `.pfx` (tùy chọn) |
| `LVHV_WIN_CERT_PASSWORD` | windows.yml | mật khẩu `.pfx` |

Không có secret nào → mọi workflow vẫn chạy và tạo bản **không ký** để kiểm tra.

Chưa có trong CI (cố ý, cần chứng chỉ của chủ dự án): ký + xuất `.ipa`/`.pkg` và tải lên App Store
Connect. Khi có chứng chỉ, nên làm trên máy Mac bằng Xcode (Product → Archive → Distribute App), hoặc bổ
sung bước `xcodebuild archive` + `-exportArchive` với các secret `APPLE_CERT_P12_BASE64`,
`APPLE_CERT_PASSWORD`, `APPLE_TEAM_ID`, `APP_STORE_CONNECT_API_KEY_*` (chưa viết).

## 6. Danh sách nộp kho ứng dụng

### 6.1 Google Play Console

- [ ] Tạo ứng dụng "Lịch và Văn hóa Việt", ngôn ngữ mặc định tiếng Việt (vi-VN), loại Ứng dụng, Miễn phí.
- [ ] Bật Play App Signing; tải lên AAB đã ký bằng khóa tải lên (artifact `android-release` khi có secret).
- [ ] `versionName` = tệp `VERSION`; `versionCode` = X·10000 + Y·100 + Z (mục 4.1). Mỗi lần tải lên Play phải **tăng `VERSION`** trước (Play từ chối `versionCode` trùng).
- [ ] Data safety: "Không thu thập dữ liệu", "Không chia sẻ dữ liệu". Đúng với mã: không có quyền nào, không có thư viện phân tích.
- [ ] Chính sách quyền riêng tư: Play **bắt buộc** có URL công khai, kể cả khi không thu thập dữ liệu. Có thể đăng trang trên GitHub Pages của PWA.
- [ ] Phân loại nội dung (IARC): Tham khảo / không bạo lực → Mọi lứa tuổi. Đối tượng: không nhắm trẻ em (tránh yêu cầu chính sách Gia đình).
- [ ] Quảng cáo: Không. Truy cập ứng dụng: không cần đăng nhập.
- [ ] Ảnh: biểu tượng 512×512 (`pwa/public/icons/icon-512.png`), ảnh nổi bật 1024×500, ít nhất 2 ảnh chụp điện thoại.
- [ ] Mô tả: nêu rõ phần xem ngày là văn hóa dân gian, chỉ để tham khảo (như màn hình Giới thiệu).
- [ ] Kiểm thử nội bộ (Internal testing) trước; tài khoản cá nhân mới phải qua kiểm thử kín 12 người trong 14 ngày trước khi phát hành chính thức — tài khoản tổ chức (Clevix LLC) không bị yêu cầu này.

### 6.2 App Store Connect (iOS + macOS)

- [ ] Đăng ký App ID `vn.clevix.lichviet` (một Bundle ID dùng cho cả iOS và macOS — App Store hỗ trợ "Universal Purchase").
- [ ] Tạo ứng dụng: tên "Lịch và Văn hóa Việt", ngôn ngữ chính Vietnamese, SKU `LICHVIET01`.
- [ ] App Privacy: "Data Not Collected". `PrivacyInfo.xcprivacy` đã khai báo: không theo dõi, không thu thập; API UserDefaults lý do CA92.1 (lưu cỡ chữ, sáng/tối).
- [ ] Mã hóa xuất khẩu: `ITSAppUsesNonExemptEncryption = NO` đã đặt trong Info.plist.
- [ ] Phân loại tuổi: 4+. Danh mục: Reference (Tham khảo) hoặc Lifestyle.
- [ ] URL chính sách quyền riêng tư và URL hỗ trợ (bắt buộc).
- [ ] Ảnh chụp: iPhone 6.9" (1320×2868), iPad 13" nếu hỗ trợ iPad (project.yml bật iPad: `TARGETED_DEVICE_FAMILY 1,2`), Mac 1280×800 trở lên.
- [ ] Archive trong Xcode với Team của Clevix LLC → Distribute → App Store Connect; kiểm thử qua TestFlight.
- [ ] Ghi chú cho người duyệt: ứng dụng chạy hoàn toàn ngoại tuyến, không cần tài khoản.
- [ ] Guideline 4.2 (Minimum Functionality): ứng dụng có nội dung riêng (lịch âm, chọn ngày, ca dao có nguồn) — nêu trong ghi chú.

### 6.3 Microsoft Store (MSIX) — chưa có gói MSIX trong CI

- [ ] Đăng ký tài khoản nhà phát triển Microsoft (công ty: Clevix LLC); đặt tên ứng dụng "Lịch và Văn hóa Việt".
- [ ] Lấy Package/Identity/Name, Publisher (CN=…) và PublisherDisplayName từ Partner Center.
- [ ] Đóng gói MSIX từ `LichViet.exe`: dùng MSIX Packaging Tool, hoặc `makeappx pack` với `AppxManifest.xml` (Capabilities: chỉ `runFullTrust`, **không** `internetClient`). Biểu tượng: 44×44, 150×150, 310×150 từ `apps/windows/assets/icon-256.png`.
- [ ] Store tự ký gói khi phát hành; không cần chứng chỉ Authenticode riêng cho Store.
- [ ] Phương án thay thế không cần MSIX: Partner Center cho nộp ứng dụng Win32 dạng **MSI/EXE** qua URL tải xuống — nhưng tệp phải được ký Authenticode (secret `LVHV_WIN_CERT_*`).
- [ ] Khai báo quyền riêng tư: không thu thập dữ liệu; URL chính sách quyền riêng tư.
- [ ] Phân loại tuổi (IARC): 3+.

## 7. Gỡ lỗi thường gặp

| Triệu chứng | Nguyên nhân / cách xử lý |
|---|---|
| Gradle: "Thiếu thư viện Rust hoặc mã UniFFI" | Chưa chạy `apps/android/build-rust.sh` |
| `UnsatisfiedLinkError: liblvhv_ffi.so` khi chạy | `.so` không có trong APK (kiểm `jniLibs.srcDirs("rustJniLibs")`) hoặc R8 xóa lớp JNA (đã giữ trong `proguard-rules.pro`) |
| Kotlin: "Checksum mismatch" từ UniFFI | Mã Kotlin sinh từ bản `.so` khác bản đóng gói; chạy lại `build-rust.sh` (sinh cả hai cùng lúc) |
| Xcode: "No such module 'lvhv_ffiFFI'" | Chưa chạy `build-xcframework.sh` trước `xcodegen generate`, hoặc thư mục headers thiếu `module.modulemap` |
| Xcode: lỗi ký | Dựng thử thì thêm `CODE_SIGNING_ALLOWED=NO`; dựng thật thì đặt `LVHV_TEAM_ID` |
| Windows: `link.exe not found` | Chưa cài Visual Studio Build Tools (C++) |
| MSI: `candle.exe` không có | Cài WiX 3.14 (không phải WiX 4/5) |

## 8. Trạng thái kiểm chứng (2026-11-06, cập nhật sau lần chạy CI thật)

Kết luận theo quy ước: **ĐÃ CHẠY** = đã biên dịch/chạy trong môi trường soạn mã (Linux x86_64);
**CHƯA KIỂM** = chưa từng được biên dịch, không có bằng chứng là chạy được.

| Hạng mục | Trạng thái | Bằng chứng / ghi chú |
|---|---|---|
| `lvhv-ffi` dựng (debug + release, Linux) | ĐÃ CHẠY | `cargo build -p lvhv-ffi [--release]` |
| `lvhv-ffi` test (API lõi mới, `Result<_, DateError>`) | ĐÃ CHẠY | 20 test ĐẠT: L8 2026-11-05; `day_info_json` và `find_good_days_json` giống lõi từng byte; ánh xạ đủ 6 loại `DateError`; lỗi không panic (kể cả tháng âm, năm `i32::MAX`); năm âm của ngày sinh trước/sau Tết; khứ hồi âm↔dương; ca dao select.md + không lặp trong N ngày; ghi công khớp SOURCES.md; ngưỡng nhãn giống PWA |
| `cargo test --workspace --release` (core + ngochap + ffi) | ĐÃ CHẠY | mọi test ĐẠT |
| `cargo fmt --all --check` | ĐÃ CHẠY (đạt) | đã sửa trước lần CI đầu; `ci.yml` xanh: [42164601448](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42164601448) |
| `clippy -D warnings`, `rustfmt` cho ffi | ĐÃ CHẠY | sạch |
| Sinh mã Kotlin + Swift bằng uniffi-bindgen 0.29.4 | ĐÃ CHẠY | `ffi/generated/` |
| Mã Kotlin sinh ra **biên dịch và gọi thật** vào `liblvhv_ffi.so` trên JVM | ĐÃ CHẠY | kotlinc 2.0.21 (bản nhúng trong Gradle 8.14.3 của máy soạn mã) + JNA 5.15.0 (jar desktop), Java 21: mọi hàm, cả ném `LvhvException.InvalidDate/UnknownActivity` |
| Lớp dữ liệu Android (`data/Json.kt`, `data/Lich.kt`) + `JsonReaderTest` | ĐÃ CHẠY trên JVM | biên dịch cùng mã UniFFI, gọi thư viện thật; test chạy bằng bộ giả JUnit tối giản (không phải JUnit thật) |
| Giao diện Android (Compose: 7 tệp `ui/*.kt`, `MainActivity.kt`) | ĐÃ DỰNG trên CI | biên dịch không lỗi, lint đạt: [42164601463](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42164601463). **Chưa chạy trên máy/emulator Android** |
| Gradle Android (AGP 8.7.3, R8, lint, ký) | ĐÃ CHẠY trên CI (chưa ký) | `testReleaseUnitTest`, `lintRelease`, `assembleRelease`, `bundleRelease` đạt; `app-release-unsigned.apk` 4,8 MB, `app-release.aab` 4,3 MB; không quyền hệ thống nào (chỉ quyền signature tự định nghĩa của androidx.core). Ký bằng khóa: CHƯA KIỂM (chưa có secret) |
| cargo-ndk, `.so` Android 3 ABI, căn trang 16 KB | ĐÃ DỰNG (3 ABI); căn 16 KB **CHƯA KIỂM** | APK có `lib/{arm64-v8a,armeabi-v7a,x86_64}/liblvhv_ffi.so` + `libjnidispatch.so`. Chưa có bước `zipalign -c -P 16` / `readelf` kiểm căn trang |
| Mã Swift sinh ra | ĐÃ DỰNG trên CI | Xcode 16.4 (macos-15): [42163747556](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42163747556) |
| Giao diện SwiftUI (9 tệp `Shared/*.swift`) | ĐÃ DỰNG trên CI; khởi động được trên iOS Simulator | biên dịch iOS Simulator, iOS thiết bị arm64 (không ký), macOS; app chạy trên Simulator và chụp được `ios-sim.png` (chưa xem ảnh, chưa chạy app macOS, chưa thử trên thiết bị thật) |
| `build-xcframework.sh`, `project.yml` (XcodeGen), xcodebuild | ĐÃ CHẠY trên CI | XCFramework 5 target, XcodeGen 2.42.0; tên macOS có dấu không gây lỗi. Ký + Archive + nộp: CHƯA KIỂM |
| `PrivacyInfo.xcprivacy`, entitlements, plist | Cú pháp ĐÃ CHẠY | `plistlib`/XML parse; nội dung chưa được Apple duyệt |
| Biểu tượng iOS/macOS/Windows | ĐÃ TẠO | rsvg-convert từ `pwa/public/icons/*.svg` |
| `lvhv-windows` (Slint 1.8.0) dựng **trên Linux** (debug + release) | ĐÃ CHẠY | `cargo build`, `cargo test` (3 test ĐẠT), `clippy -D warnings` sạch |
| Giao diện Slint hiển thị đúng | ĐÃ CHẠY trên Linux/Xvfb | chụp màn hình tờ lịch + bảng tháng ở 1280×900 |
| RAM ứng dụng Slint | ĐO trên Linux/Xvfb | ~52 MB (renderer phần mềm), ~170 MB (femtovg qua llvmpipe). Chưa đo trên Windows |
| `lvhv-windows` dựng trên **Windows/MSVC**, nhúng tài nguyên, MSI (WiX) | ĐÃ CHẠY trên CI | windows-2022: `cargo test`, `cargo build --release`, zip + MSI (WiX 3.14.1, cảnh báo ICE91 vô hại cho bộ cài theo người dùng): [42164601439](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42164601439). RAM sau 8 giây: Working set 56,2 MB, Private 39,5 MB. **Chưa cài thử MSI**, chưa kiểm tra giao diện trên Windows |
| Script PowerShell `New-AndroidUploadKeystore.ps1` | ĐÃ CHẠY trên pwsh 7.4.6 (Linux) | tạo khóa thật bằng keytool, Base64 khứ hồi đúng, in SHA-256, từ chối ghi đè. `Read-Host`/`Set-Clipboard` được giả lập khi chạy thử. Chưa chạy trên Windows PowerShell 5.1 |
| Script `Build-Windows.ps1` | Cú pháp ĐÃ CHẠY | trình phân tích của pwsh 7.4.6; chưa chạy thật |
| Workflow YAML | ĐÃ CHẠY | `yaml.safe_load` + **actionlint 1.7.7** + shellcheck 0.10.0: sạch |
| Ký Authenticode (Windows) | **CHƯA KIỂM** | chưa có secret `LVHV_WIN_CERT_*` |
| Các workflow chạy thật trên GitHub | ĐÃ CHẠY, xanh ở commit `84d0dc8` | android [42164601463](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42164601463) · windows [42164601439](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42164601439) · ci [42164601448](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42164601448) · apple [42164601457](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42164601457) (cũng xanh ở `c5a168e`: [42163747556](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42163747556)) |

### 8.1 Nhật ký build CI (2026-11-06)

Lần chạy đầu (commit `2ddb4ec`): android [42160816767](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42160816767) đỏ, apple [42160816591](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42160816591) đỏ,
windows [42160816589](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42160816589) bị hủy (một lần push mới hơn đã thay nó).

| # | Nền tảng | Lỗi | Nguyên nhân | Sửa (commit) |
|---|---|---|---|---|
| 1 | apple | bước "Kiểm tra": `Có quyền mạng trong entitlements` | `grep -R "com.apple.security.network"` khớp với **chú thích XML** trong `LichViet.entitlements` ("KHÔNG có quyền mạng (com.apple.security.network.client/server)"). Không có quyền mạng thật | grep chỉ bắt `<key>com.apple.security.network…` (`f71d2db`) |
| 2 | android | `:app:checkRustArtifacts` báo thiếu `lvhv_ffi.kt` (dù `build-rust.sh` báo "Xong") | cargo-ndk **strip** `.so` Android nên mất ký hiệu `UNIFFI_META_*`; `uniffi-bindgen --library` khi đó không sinh tệp nào mà vẫn thoát 0 (tái hiện được trên Linux bằng `strip`) | `build-rust.sh` sinh mã Kotlin từ thư viện host chưa strip, cùng profile (giống `ci.yml`; tệp sinh ra giống hệt `ffi/generated`); script báo lỗi nếu không có `.kt`; Gradle in đường dẫn tệp thiếu (`f71d2db`, `c5a168e`) |
| 3 | cả ba | run trên `main` bị hủy giữa chừng ("Canceling since a higher priority waiting request…") | `concurrency.cancel-in-progress: true` cùng nhóm theo ref, trong khi commit PWA được push liên tục lên `main` | nhóm concurrency theo commit trên main/thẻ; chỉ hủy run cũ của PR (`c5a168e`) |
| 4 | windows | `candle` lỗi `CNDL0230` ở `wix/main.wxs` dòng 48, 54 | `Guid='*'` không dùng được cho thành phần có KeyPath là giá trị sổ đăng ký (bắt buộc với MSI cài theo người dùng) | GUID cố định cho 3 thành phần, không được đổi giữa các phiên bản (`277c68b`) |
| 5 | android | bước kiểm quyền: `uses-permission: vn.clevix.lichviet.DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION` | androidx.core ≥ 1.9 tự thêm quyền mức signature do chính ứng dụng định nghĩa (bảo vệ receiver nội bộ). Không phải quyền hệ thống; không có `INTERNET` | chỉ ngoại lệ đúng dòng đó; mọi `uses-permission` khác vẫn làm đỏ (`84d0dc8`) |

Kết quả cuối: android ✓ [42164601463](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42164601463) (artifact `android-release`: `app-release-unsigned.apk`,
`app-release.aab`, `mapping.txt`; `android-lint-report`) · windows ✓ [42164601439](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42164601439) (artifact `windows-x64`:
`LichViet-0.1.0-x64.zip`, `LichViet-0.1.0-x64.msi`, `mem.txt`) · apple ✓ [42164601457](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42164601457) (artifact
`apple-unsigned`: `LichViet-iOS-simulator.zip`, `LichViet-macOS-unsigned.zip`, `ios-sim.png`) · ci ✓ [42164601448](https://github.com/ClevixLab/lich-van-hoa-viet/actions/runs/42164601448).
Không cần sửa mã Compose, SwiftUI hay Slint nào. Không đổi `core/`, `ngochap/`, `tsport/`, `pwa/`, `content/`.

Còn mở: cảnh báo GitHub "Node.js 20 is deprecated" cho các action v4 (chưa lỗi); chưa kiểm căn trang 16 KB của
`.so`; chưa cài/chạy thử APK, MSI, app macOS trên máy thật.

Rủi ro lớn nhất khi chạy lần đầu (theo thứ tự): (1) lỗi biên dịch Compose/SwiftUI nhỏ (tên API, thiếu
import) — sửa theo log CI; (2) `apple.yml`: đường dẫn sản phẩm `.app` có dấu tiếng Việt trong tên
(`PRODUCT_NAME` của macOS) — nếu công cụ nào lỗi vì tên có dấu, đổi `PRODUCT_NAME` sang `LichViet` và
giữ `CFBundleDisplayName` có dấu; (3) WiX: mã trang 1252 với chuỗi tiếng Việt (đã viết không dấu trong
MSI để tránh).

## 9. Đối chiếu tiêu chí

- **S1** áp dụng cho `lvhv-core` (CI kiểm `#![forbid(unsafe_code)]` và không có từ `unsafe` trong
  `core/src`). `lvhv-ffi` không có `unsafe` viết tay; macro UniFFI sinh `extern "C"` có `unsafe` bên trong.
  `lvhv-windows` dùng `#![deny(unsafe_code)]`; chỉ mô-đun `ui` (mã Slint sinh ra) được miễn.
- **S2** (lõi không phụ thuộc thư viện ngoài): không bị ảnh hưởng — UniFFI, serde, Slint chỉ là phụ
  thuộc của `ffi/` và `apps/windows/`, không phải của `core/`.
- **S3/S4 cho native**: không có quyền/mã mạng; chỉ lưu cỡ chữ + sáng/tối (Android SharedPreferences,
  Apple UserDefaults; Windows không lưu) và ngày hết hạn "Để sau" của thẻ nhắc cập nhật (mục 4.1). Năm sinh không bao giờ được lưu (Android: `remember`, không
  `rememberSaveable`).
- **P1/P2** đo cho lõi native ở docs/03 §13; lớp FFI thêm chi phí tuần tự hóa JSON (chưa đo riêng trên
  thiết bị).
