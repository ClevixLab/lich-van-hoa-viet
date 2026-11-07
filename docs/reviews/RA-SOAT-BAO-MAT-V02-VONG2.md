# Rà soát bảo mật và quyền riêng tư — v0.2 đợt 3, vòng 2 (kiểm lại)

Trạng thái: **HOÀN TẤT — chờ các vai sửa và chủ dự án xem.**

Vai: rà soát bảo mật và quyền riêng tư (T10), cùng bên đã làm vòng 1 — bên không tham gia xây dựng đặc tả và không
viết mã đang duyệt. Ngày: 2026-11-07. Không sửa mã đang duyệt; mọi chương trình thử nằm ngoài kho.

Vòng 1: `docs/reviews/RA-SOAT-BAO-MAT-V02.md` (PR #34, mã phát hiện BM2-01 … BM2-24, mô hình đe dọa v0.2, yêu cầu
đăng ký trước §5.3). Tài liệu này chỉ ghi phần **kiểm lại** và phát hiện mới (mã BM3-xx); mô hình đe dọa của vòng 1
vẫn dùng, với các khoảng trống T-D2, T-D4, T-I3, T-I4, T-I5 nay đã được đóng hoặc thu hẹp như dưới đây.

## 0. Phạm vi

| Nhánh | Commit đã rà | Ghi chú |
|---|---|---|
| `main` | `8835cd7` | đã gộp #28 (core2), #35 (B6), #36 (`lvhv-store-tool`, cargo-deny mọi tính năng), #37 (B7), #38 (core3) |
| PR #30 PWA vòng 2 | `9e590d8` | CI xanh (ci, android, apple, windows) |
| PR #24 Android | `ab83b9b` | CI xanh (android: build, ui, upgrade, screenshots) |
| PR #25 Windows | `5f4dfcc` | CI xanh; đầu nhánh hiện là `8b86892` = chỉ thêm ảnh chụp `[skip ci]`, mã như `5f4dfcc` |
| PR #23 Apple | `e3e1ddf` (mã); đầu nhánh `85850c4` chỉ thêm ảnh chụp | **CI chưa xanh**: job `build` của workflow `apple` trên `e3e1ddf` còn `in_progress` lúc rà (run 42349887604; `upgrade` đã `success`); các run của `85850c4` ở trạng thái `action_required`. Rà mã vẫn làm; kết luận #23 có điều kiện |

## 1. Tổng hợp

**Ba mục chặn của vòng 1 (BM2-01, BM2-02, BM2-03) đã đóng.** Không còn mục mức chặn nào.

| Mức | Còn mở từ vòng 1 | Mới ở vòng 2 |
|---|---|---|
| Chặn | 0 | 0 |
| Nên sửa | 1 (BM2-11) | 3 (BM3-01, BM3-02, BM3-03) |
| Góp ý | 5 (BM2-06 phần còn lại, BM2-15, BM2-21, BM2-22, BM2-24) | 4 (BM3-04 … BM3-07) |

| Mã | Mức | Nền tảng / PR | Tiêu đề |
|---|---|---|---|
| BM3-01 | nên sửa | main (lõi Rust) | Sau chuyển đổi kho 1 → 2, tệp ảnh giấy chứng tử gốc `photos/<id>.img` thành tệp mồ côi: không bị xóa sau 7 ngày, không bị xóa khi xóa hồ sơ (PoC §3.2) |
| BM3-02 | nên sửa | #25 Windows | Mở PDF bằng ứng dụng ngoài không qua hộp xác nhận (S10, §5.3 mục 3) |
| BM3-03 | nên sửa | #23 Apple | Xem tệp đính kèm bằng Quick Look có sẵn nút Chia sẻ/Lưu của hệ điều hành, không qua hộp xác nhận (S10) |
| BM3-04 | góp ý | main (lõi Rust + TS) | Kiểm ảnh B6 nhận byte nối sau EOI (JPEG) / sau khối RIFF (WebP) — một JPEG có EXIF GPS nối sau ảnh hợp lệ được nhận (PoC) |
| BM3-05 | góp ý | main (lõi) | `check_attachment` nhận JPEG có SOF cao (hoặc rộng) = 0 |
| BM3-06 | góp ý | #30 PWA (có từ v0.1) | `window.__lvhv` để lộ API lưu trữ (`storage.loadStore`, `openStore`…) trong bản phát hành |
| BM3-07 | góp ý | #25 Windows | `LichViet.exe` bản phát hành còn các chế độ CI (`--kiem-nang-cap`, `--tao-du-lieu-mau`, `--chup-man-hinh`…) và biến môi trường `LVHV_DATA_DIR`, `LVHV_TZ_OFFSET_MIN` |

## 2. Đã chạy

| Việc | Kết quả |
|---|---|
| PoC đầu vào độc qua đường nhập thật của main (`lvhv_store::open_backup`), chương trình dùng một lần ngoài kho | §3.1: 6/6 ca độc của vòng 1 và B6 **bị từ chối**, thông báo chung; 4 ca biên mới: 3 được nhận (BM3-04, BM3-05, PDF giả có `%%EOF` — chấp nhận theo R-PDF-1) |
| PoC chuyển đổi 1 → 2 trên bộ mẫu đóng băng `released/0.2-dev-before-attachments`, mở lại sau 8 ngày, xóa mọi hồ sơ | §3.2: bản sao `.pre-v2.bak` bị xóa đúng hạn; **tệp ảnh giấy chứng tử gốc còn lại** (BM3-01) |
| `cargo test --release -p lvhv-backup -p lvhv-store` trên main `8835cd7` | §8 |
| `cargo audit` (cơ sở dữ liệu RustSec `309392b1`, 2026-11-06T16:34+01:00): `Cargo.lock` gốc (main) và `apps/windows/Cargo.lock` (#25) | gốc: 1 cảnh báo `basic-toml` (RUSTSEC-2026-0374), đã ghi ngoại lệ có lý do và ngày ở `deny.toml` và `.cargo/audit.toml`; Windows: như vòng 1 (2 lỗ hổng `quick-xml 0.30.0`, 9 cảnh báo không còn bảo trì) |
| `cargo deny check --disable-fetch advisories bans` (cargo-deny 0.18.9, cùng cơ sở dữ liệu): workspace gốc (main) và `apps/windows` (#25, `apps/windows/deny.toml`, `targets = x86_64/aarch64-pc-windows-msvc`) | cả hai `advisories ok, bans ok` |
| `npm audit --omit=dev` và `npm audit` trong `pwa/` (#30; `package-lock.json` giống main) | `found 0 vulnerabilities` cả hai |
| Dựng PWA bản phát hành của #30 (`node build.mjs`) và tìm tên móc kiểm thử trong `dist/` | 0 lần `__lvhvGates`, `__lvhvFailMigration`, `testGate`, `testFlag`, `__LVHV_TEST_HOOKS__`; còn `window.__lvhv` (BM3-06); CSP trong `dist/index.html` giống từng byte bản 0.1.1 |
| Đọc nhật ký CI Android `ab83b9b` (run 42351718963, job `build`) | bước S6/S5/BM2-18/osv-scanner đều `success`; trích ở §8 |

Không chạy được: ứng dụng native trên thiết bị hay trình giả lập (môi trường rà soát là Linux không có SDK); đo động
R7/S7; `bmgr backupnow` (S5); đo bộ nhớ đỉnh S12 trên thiết bị.

## 3. Bằng chứng PoC

### 3.1 Đầu vào độc qua `open_backup` (main `8835cd7`)

Cách làm như vòng 1 (§8.2): lấy `supplement-yc05/notice-v1.json`, thay byte một khối, sửa `bytes`/`sha256` cho khớp,
đóng gói bằng `export_with_fixed_rng` với mật khẩu của "kẻ gửi", gọi `lvhv_store::open_backup`. Ảnh hợp lệ dùng
`test-vectors/media/jpeg-ok.jpg`; ảnh EXIF dùng `media/jpeg-exif-gps.jpg`; PNG 20 000 × 20 000 (388 871 byte) sinh lại
như vòng 1.

```
[A] baseline (ảnh thật): NHẬN
[A] PNG 20000x20000 khai image/jpeg làm ảnh chân dung: TỪ CHỐI — PHOTO /photos: mime_mismatch | Sai mật khẩu hoặc tệp bị hỏng
[A] PNG bom làm tệp đính kèm ảnh: TỪ CHỐI — ATTACHMENT /attachments: mime_mismatch | Sai mật khẩu hoặc tệp bị hỏng
[A] HTML làm tệp đính kèm ảnh: TỪ CHỐI — ATTACHMENT /attachments: mime_mismatch | Sai mật khẩu hoặc tệp bị hỏng
[A] 'PDF' %PDF-<html> không %%EOF: TỪ CHỐI — ATTACHMENT /attachments: pdf_no_eof | Sai mật khẩu hoặc tệp bị hỏng
[A] ảnh chân dung có EXIF GPS (APP1): TỪ CHỐI — PHOTO /photos: metadata | Sai mật khẩu hoặc tệp bị hỏng
[A] ảnh chân dung hợp lệ + JPEG có EXIF GPS nối sau EOI: NHẬN                      ← BM3-04
[A] tệp đính kèm ảnh hợp lệ + JPEG có EXIF GPS nối sau EOI: NHẬN                   ← BM3-04
[A] tệp đính kèm JPEG có SOF cao = 0: NHẬN                                        ← BM3-05
[A] 'PDF' là HTML có %PDF- và %%EOF: NHẬN                                         ← theo thiết kế (R-PDF-1)
```

Kết luận: lỗ hổng chứng minh ở vòng 1 (bom giải nén, sai nhãn, HTML, PDF giả, EXIF) **không còn tái hiện được**. Thông
báo cho người dùng vẫn là thông báo chung (không lộ lý do — không có kênh dò).

### 3.2 Chuyển đổi kho 1 → 2 và tệp còn lại (main `8835cd7`)

Chép `released/0.2-dev-before-attachments/store` (kho `user_version = 1`, hồ sơ có `certificate_photo_ids`) vào thư
mục tạm; `Store::open_with(now = T0)`; mở lại với `now = T0 + 8 ngày`; xóa mọi hồ sơ bằng `delete_person`.

```
[B] sau chuyển đổi: trạng thái "migrated"
  ảnh có bản ghi: ["…5e09"]                    (5e09 là ảnh chân dung, 5e0c là ảnh giấy chứng tử)
  thư mục kho: attachments/, lvhv.sqlite(+wal,shm), lvhv.sqlite.pre-v2.bak, photos/
  photos/:      …5e09.img (33714 B), …5e0c.img (33714 B)
  attachments/: …5e0c.bin (33714 B), …e5d4c3b2a1f6.bin (33714 B)
[B] sau 8 ngày: lvhv.sqlite.pre-v2.bak đã bị xóa;  photos/: …5e09.img, …5e0c.img
[B] sau khi xóa mọi hồ sơ (3):
  photos/:      …5e0c.img (33714 B)            ← ảnh giấy chứng tử, không còn bản ghi nào trỏ tới
  attachments/: (trống)
  ảnh có bản ghi: 0
```

## 4. Kiểm lại từng phát hiện của vòng 1

| Mã | Mức (vòng 1) | Trạng thái | Bằng chứng |
|---|---|---|---|
| BM2-01 | chặn | **ĐÃ SỬA** | `store/src/media.rs` (chữ ký, kích thước từ phần đầu, không APP1/APP13, WebP không EXIF/XMP, PDF `%PDF-` + `%%EOF`, ≤ 10 MiB) gọi ở `verify_contents` (`store/src/lib.rs:1974-1997`, cả ảnh chân dung nhờ R6v2/B7), `add_photo` (`:945`), `add_attachment` (`:820`), chuyển đổi (`:1503`, `:1572`); bản TS `pwa/src/store/media.ts` dùng ở `db.ts:65, 216, 400, 715, 724`. PoC §3.1. Ca biên còn lại: BM3-04, BM3-05 (góp ý) |
| BM2-02 | chặn | **ĐÃ SỬA** | `store/src/migrate.rs` (`STORE_VERSION = 2`, `migrate_1_to_2` thuần, bản gốc giữ ở khóa `legacy`), chạy trong `Store::open` sau bản sao thô và trong `put_person`/nhập; bộ mẫu đóng băng `test-vectors/released/0.2-dev-before-attachments` + `store/tests/released.rs`. PoC §3.2 ra `migrated`. Phần còn sót: BM3-01 |
| BM2-03 | chặn | **ĐÃ SỬA** | #30 không còn sửa `pwa/src/store/` so với main (dùng chung lớp lưu trữ của #28): `ATTACHMENTS_UNSUPPORTED` và kho `legacy_v1` không còn; `backup-crypto.ts:26` `MAX_BLOB` 10 MiB như Rust; job CI "Lưu trữ + sao lưu (R6, … Rust ↔ PWA)" xanh trên `9e590d8` |
| BM2-04 | nên sửa | **ĐÃ SỬA** | Android `ui/people/Secure.kt` (`FLAG_SECURE` có đếm) dùng ở `DeathNotice.kt:31`, `Forms.kt:284`, `BackupScreen.kt:67`, `PersonScreen.kt:99` (khi `noticeHasContent`, gồm cả chỉ có tệp đính kèm); Apple `PeopleUI.swift:670-694` `PrivacyCover` khi `scenePhase != .active`, áp ở `RootView.swift:97` và từng tấm ở `BackupView.swift:90, 99, 133`. Chưa xem ảnh Recents/app switcher thật |
| BM2-05 | nên sửa | **ĐÃ SỬA** | #30 `people/notice.ts` hộp "Tải tệp xuống máy?" (`att-dl-dialog`, mặc định nút Không), kiểm ở `test/people-flows.mjs:685-698` |
| BM2-06 | nên sửa | **ĐÃ SỬA phần chính**; còn góp ý | `post_migration_cleanup` (`store/src/lib.rs:1412-1462`) xóa `.bak` và `.bak-wal` 7 ngày sau chuyển thành công (PoC §3.2). Chưa làm: xóa `.bak` ngay khi "Thay toàn bộ" thành công (hạ xuống góp ý; hạn 7 ngày đã giới hạn rủi ro) |
| BM2-07 | nên sửa | **ĐÃ SỬA** | PWA: kho `legacy_v1` bỏ, thay bằng khóa `legacy` trong hồ sơ, xóa sau 7 ngày (`db.ts:101-108, 277-294`), không đưa vào tệp sao lưu; Rust cùng quy tắc (`lib.rs:1080-1083, 1412-1462`). Ảnh chuyển được xóa khỏi IndexedDB trong cùng giao dịch (`db.ts:166`) — nhưng bản Rust không xóa tệp, xem BM3-01 |
| BM2-08 | nên sửa | **ĐÃ SỬA** | như BM2-01 (`add_photo`, `add_attachment` gọi `media::check_*`) |
| BM2-09 | nên sửa | **ĐÃ SỬA** | `deny.toml` gốc `all-features = true` + ngoại lệ `basic-toml` có lý do/ngày; `.cargo/audit.toml`; #25 thêm `apps/windows/deny.toml` (đích Windows, 4 ngoại lệ "không bảo trì" có lý do/ngày) và bước CI; #24 sinh `gradle.lockfile` + `osv-scanner` trên `releaseRuntimeClasspath` ("No issues found", 84 gói). `cargo deny` của rà soát: ok cả hai workspace |
| BM2-10 | nên sửa | **ĐÃ SỬA** | PWA `people/photo.ts` `drawPhoto` (đọc kích thước, trần 4096/16 Mpx, lấy mẫu khi giải mã); Android `Photos.bitmap` (`inJustDecodeBounds`, trần 12 000/50 Mpx, `inSampleSize`); Windows `photo::decode_bounded` (`Limits` 4096 px, 80 MiB, chỉ JPEG/PNG/WebP); Apple `PhotoProcessor.cgImage` (trần + `CGImageSourceCreateThumbnailAtIndex`, 2048 px) |
| BM2-11 | nên sửa | **CHƯA SỬA** | không có `secure_delete` trong `store/` hay `ffi/` |
| BM2-12 | nên sửa | **ĐÃ SỬA** | PWA hộp "Đính kèm tệp PDF?" trước khi đọc (`pdf-warn-dialog`), hộp xuất sao lưu ghi số PDF (`people/backup.ts:82-101`); Android/Windows/Apple ghi "Tệp PDF được lưu nguyên bản" khi chọn và ở hộp xuất (Apple `BackupView.swift:310`) |
| BM2-13 | góp ý | **ĐÃ SỬA** | `processPhoto` đọc 2 MB đầu, định dạng không nhận ra → từ chối |
| BM2-14 | góp ý | **ĐÃ SỬA** | Apple `passwordContent()` bỏ `textContentType(.password)` |
| BM2-15 | góp ý | **CHƯA SỬA** | `toasts.rs:100-106` `launch_uri` vẫn mang `c=<content>` |
| BM2-16 | góp ý | **ĐÃ SỬA** | Android `cleanShareCache` khi mở và `onResume` |
| BM2-17 | góp ý | **ĐÃ SỬA phần giao diện** | Android `verified = null`, Apple `verified = nil` khi đóng tấm; mật khẩu trong chuỗi của nền tảng vẫn không xóa được (giới hạn ngôn ngữ, giữ ghi chú) |
| BM2-18 | góp ý | **ĐÃ SỬA** | CI Android: `aapt2 dump xmltree` APK release, `grep SelfCheck` → lỗi; danh sách thành phần exported chỉ `MainActivity` và `ProfileInstallReceiver` (quyền `DUMP`) |
| BM2-19 | góp ý | **ĐÃ SỬA** | `Reminders.acceptOpen` (mã của chính ứng dụng) ở `MainActivity.onCreate/onNewIntent`; `OpenTokenTest.kt` |
| BM2-20 | góp ý | **ĐÃ SỬA** | Apple `protectPhotos`: `photos/`, `attachments/` lớp `.complete` (iOS) |
| BM2-21 | góp ý | **CHƯA SỬA** | `verify_contents` vẫn `bytes.clone()` (`lib.rs:1962, 1982, 1996`) |
| BM2-22 | góp ý | **CHƯA SỬA** | Windows `platform.rs:20-22` vẫn lùi về `%TEMP%` |
| BM2-23 | góp ý | **ĐÃ SỬA** | Windows dùng `lvhv_ffi::backup_password_error`; Android so khớp sau NFC (`BackupScreen.kt:279`) |
| BM2-24 | góp ý | **CHƯA SỬA phần lớn** | `ci.yml` còn `@v2`/`@v4`; Android đã ghim `upload-artifact` theo SHA |

## 5. Đường tệp đính kèm và PDF theo §5.3 (vòng 1)

| Yêu cầu §5.3 | PWA #30 | Android #24 | Windows #25 | Apple #23 |
|---|---|---|---|---|
| 1. Ảnh qua bộ xử lý ảnh (≤ 2048 px, ≤ 2 MB) + kiểm lõi | Đạt (`processPhoto` + `checkAttachment`) | Đạt (`Photos.process(…, ATT_EDGE, ATT_STORED)` + lõi) | Đạt (`photo::reencode_with(DOC_SIDE, DOC_STORED)` + lõi) | Đạt (`PhotoProcessor.process` + lõi) |
| 2. PDF: `%PDF-`, ≤ 10 MB trước khi đọc hết; không hiển thị PDF trong tiến trình | Đạt (`f.size` trước `arrayBuffer`, chỉ tải về) | Đạt (`OpenableColumns.SIZE` trước, đọc tối đa 10 MB + 1, chỉ mở ngoài) | Đạt (`metadata.len()` + `%PDF-` trước, `take(10 MiB + 1)`, chỉ mở ngoài) | Đạt phần kiểm (`PersonFormView.swift:109-111`); hiển thị bằng Quick Look — chạy trong dịch vụ của hệ điều hành, rà soát không xác minh được tiến trình nào giải mã PDF |
| 3. Mở bằng ứng dụng ngoài: tệp tạm riêng, tên chung, quyền một lần, xóa khi quay lại / khi mở | Không áp dụng (chỉ tải về, tên `TepDinhKem-<n>.pdf`) | Đạt (`cache/att/TepDinhKem.pdf`, `<cache-path name="att">`, `ACTION_VIEW` + `FLAG_GRANT_READ_URI_PERMISSION`, không `grantUriPermission`, `revokeUriPermission` + xóa ở `onResume` và `onCreate`) | Đạt phần tệp tạm (`%LOCALAPPDATA%\…\tmp\<uuid>\TepDinhKem.pdf`, xóa khi mở và khi đóng; `explorer.exe <tệp>`) | Đạt phần tệp tạm (`tmp/att/TepDinhKem.pdf|.jpg`, xóa khi đóng xem trước và khi mở ứng dụng) |
| 3'. Qua hộp xác nhận | Đạt (BM2-05) | Đạt ("Mở tệp PDF bằng ứng dụng khác?", nói rõ ứng dụng kia đọc được nội dung) | **Không** (BM3-02) | **Không** (BM3-03) |
| 4. Không đưa vào bảng chia sẻ khi người dùng chưa chọn rõ | Đạt | Đạt (tệp đính kèm không có chia sẻ) | Đạt | **Không** — nút Chia sẻ của Quick Look (BM3-03) |

## 6. Phát hiện mới

### BM3-01 — nên sửa — Tệp ảnh giấy chứng tử gốc còn lại sau chuyển đổi 1 → 2 (lõi Rust)

- **Nơi:** main `store/src/lib.rs:1466-1527` (`migrate_data_1_to_2`) và `:1530-1600` (`migrate_person_in_tx`): sau khi
  chép byte sang `attachments/<aid>.bin`, chỉ xóa **bản ghi** `photo` của ảnh đã chuyển (`DELETE FROM photo`), giữ tệp
  `photos/<pid>.img` ("không xóa tệp ảnh nào" — để bản sao CSDL + thư mục ảnh đủ cho quay lui). `post_migration_cleanup`
  (`:1412-1462`) sau 7 ngày chỉ xóa `.bak` và khóa `legacy`, **không** xóa các tệp ảnh đã chuyển. Tệp ảnh không bản ghi
  chỉ bị dọn trong `apply_import` (`:1342-1347`).
- **Mối đe dọa:** T-I8 (dữ liệu đã xóa còn trên đĩa), A3 (ảnh giấy chứng tử). Trái 01 §3.2a "xóa hồ sơ hoặc gỡ khỏi
  danh sách thì xóa tệp".
- **Kịch bản:** PoC §3.2 — sau chuyển đổi, qua 7 ngày, và sau khi người dùng xóa mọi hồ sơ, `photos/…5e0c.img` (bản
  chụp giấy chứng tử) vẫn nằm trong thư mục ứng dụng, không còn được ứng dụng nhìn thấy hay xóa được, cho tới khi người
  dùng tình cờ "khôi phục" một tệp sao lưu. Bản PWA đã xóa ảnh trong cùng giao dịch (`db.ts:166`) — hai bản lệch nhau.
- **Sửa:** ghi danh sách `moved_photos` vào `meta` (ví dụ khóa `migration_moved_photos`) khi chuyển thành công; trong
  `post_migration_cleanup`, sau 7 ngày xóa các tệp `photos/<pid>.img` có trong danh sách mà không còn bản ghi `photo`.
  Thêm vào `store/tests/released.rs` một ca: chuyển → +8 ngày → không còn tệp `.img` nào không có bản ghi.

### BM3-02 — nên sửa — Windows mở PDF bằng ứng dụng ngoài không qua hộp xác nhận

- **Nơi:** #25 `ui/people.slint:1042-1044` (nút "Mở bằng ứng dụng xem PDF" gọi thẳng `open-attachment`),
  `src/people.rs:3071-3082` (`open_att` → `attach::open_pdf`), `src/attach.rs:102-111` (`explorer.exe <tệp>`).
- **Mối đe dọa:** T-I4, S10 ("0 đường xuất không qua hộp này"); §5.3 mục 3 dòng cuối. Ứng dụng xem PDF mặc định của
  Windows thường là trình duyệt; nó có thể ghi tệp vào lịch sử/"tệp gần đây" và đồng bộ lịch sử lên tài khoản.
- **Sửa:** hộp xác nhận như Android ("Mở tệp PDF bằng ứng dụng khác? Ứng dụng đó đọc được nội dung tệp…"; nút mặc
  định là Hủy) trước `open_att`; thêm ca vào `ui_tests_people.rs` (bấm Hủy → không có tệp trong `tmp\`).

### BM3-03 — nên sửa — Apple: Quick Look có nút Chia sẻ, không qua hộp xác nhận

- **Nơi:** #23 `PersonDetailView.swift:35-36, 213, 276-279` (nút "Xem" → `AttachmentTemp.write` → `.quickLookPreview`),
  `PersonFormView.swift:99-100, 126`.
- **Mối đe dọa:** S10, §5.3 mục 4. Màn xem trước Quick Look của hệ điều hành có sẵn thao tác Chia sẻ (và Lưu vào Tệp,
  In, Đánh dấu); người dùng có thể đưa ảnh/PDF giấy chứng tử sang ứng dụng khác mà ứng dụng không cảnh báo gì. (Suy từ
  hành vi mặc định của Quick Look; rà soát chưa chạy trên thiết bị.)
- **Sửa:** ảnh đính kèm: hiển thị trong ứng dụng bằng `PhotoProcessor.cgImage` (đã có trần), không qua Quick Look; PDF:
  hộp xác nhận trước khi mở Quick Look ("Màn xem trước của hệ thống có nút Chia sẻ: tệp chia sẻ đi sẽ nằm ngoài ứng
  dụng, không có mật khẩu"). Thêm ca vào `PeopleUITests`.

### BM3-04 — góp ý — Kiểm ảnh nhận dữ liệu nối sau ảnh

- **Nơi:** main `store/src/media.rs:114-115` (JPEG: dừng ở SOS, không xét phần sau), `:126-130` (WebP: chỉ đòi
  `riff + 8 ≤ len`, cho phép thừa byte); bản TS giống hệt.
- **Kịch bản:** PoC §3.1 — `jpeg-ok.jpg` nối thêm nguyên `jpeg-exif-gps.jpg` sau EOI được nhận làm ảnh chân dung và tệp
  đính kèm; byte GPS nằm trong kho và đi tiếp vào tệp sao lưu người dùng xuất. Bộ giải mã bỏ qua phần thừa nên không gây
  sập; công cụ đọc siêu dữ liệu thông thường không đọc phần thừa, nhưng công cụ quét toàn tệp sẽ thấy.
- **Sửa:** JPEG: sau SOS phải tìm được `FF D9` và đó là 2 byte cuối của tệp (cho phép tối đa vài byte đệm 0x00);
  WebP: đòi `riff + 8 == len` (hoặc `len − 1` nếu lẻ). Thêm hai ca vào `test-vectors/media/`.

### BM3-05 — góp ý — `check_attachment` nhận JPEG có SOF cao/rộng = 0

- **Nơi:** main `store/src/media.rs:205-223` (chỉ so `max(w, h) ≤ 2048`); `check_photo` không bị vì phải bằng bản ghi.
- **Kịch bản:** PoC §3.1. SOF cao = 0 nghĩa là chiều cao khai sau ở đoạn DNL (tới 65 535) — một đường né trần kích
  thước. libjpeg-turbo và các bộ giải mã phổ biến không hỗ trợ DNL nên hiện chỉ là ảnh hỏng; giữ ở mức góp ý.
- **Sửa:** đòi `w ≥ 1 && h ≥ 1` trong `jpeg_dims`/`webp_dims` (cả Rust và TS).

### BM3-06 — góp ý — `window.__lvhv` trong bản phát hành PWA

- **Nơi:** `pwa/src/main.ts:33-38` (có từ v0.1.4; #30 không đổi): `window.__lvhv = { renderMs, mock, storage }` với
  `storage` = `loadStore`, `openStore`, `backupReminderDue`, `requestPersistentStorage`. Có trong `dist/app-*.js` của
  bản dựng chính (đã dựng và tìm, §2). Kiểm thử khói dùng nó (`test/smoke.mjs:115, 140, 392, 436`).
- **Đánh giá:** không mở thêm quyền: chỉ mã chạy trên cùng nguồn gốc mới đọc được `window`, và mã đó vốn đã đọc thẳng
  được IndexedDB `lvhv` (CSP `script-src 'self'` chặn mã ngoài). Nhưng nó là một bề mặt ổn định, có tên, giúp mã độc cùng
  nguồn gốc (RS-10: các trang khác trên cùng `*.github.io`) hay tiện ích trình duyệt lấy toàn bộ sổ người thân (kể cả
  thông tin báo tử) bằng một dòng, và không có lý do để có trong bản phát hành.
- **Sửa:** bọc phần `storage` (và cả `window.__lvhv`) bằng `if (__LVHV_TEST_HOOKS__)` như `testhooks.ts`; chuyển các
  kiểm thử khói dùng nó sang bản dựng `LVHV_TEST_HOOKS=1`; thêm `__lvhv` vào danh sách `HOOK_NAMES` của `smoke.mjs`.

### BM3-07 — góp ý — Chế độ CI trong `LichViet.exe` bản phát hành (Windows)

- **Nơi:** #25 `src/main.rs:690-775`, `src/upgrade.rs`, `src/platform.rs:14-27, 84`.
- **Đánh giá:** các chế độ `--kiem-thong-bao`, `--chup-man-hinh`, `--do-bo-nho`, `--tao-du-lieu-mau`, `--kiem-nang-cap`,
  `--kiem-mau-dong-bang`, `--kiem-chi-doc` và biến `LVHV_DATA_DIR`, `LVHV_TZ_OFFSET_MIN` có trong bản phát hành.
  `--kiem-nang-cap <tệp>` mở **kho thật**, đặt lại thông báo và in khác biệt (có thể gồm dữ liệu hồ sơ) ra stdout/stderr.
  Không gọi được qua giao thức `lichviet:` (tham số đầu là URI nên không khớp chế độ nào), nên chỉ người đã chạy được lệnh
  trên máy mới dùng — rủi ro thấp.
- **Sửa:** đưa các chế độ này sau một tính năng Cargo (`ci-modes`) chỉ bật khi CI dựng bản kiểm thử, hoặc `cfg(debug_assertions)`
  với một bản dựng riêng cho bước R14; bản MSI phát hành không có.

## 7. Vệ sinh bản phát hành (móc kiểm thử)

| Nền tảng | Kết quả |
|---|---|
| PWA | Móc kiểm thử (`testhooks.ts`) bị esbuild bỏ ở bản chính (`define __LVHV_TEST_HOOKS__ = false`); `build.mjs` từ chối `LVHV_TEST_HOOKS=1` khi ghi vào `dist/`; `smoke.mjs:460-470` kiểm cả hai chiều; rà soát dựng lại và xác nhận 0 tên móc. `forceMigrationFailure` chỉ truyền khi `__LVHV_TEST_HOOKS__` (`people/core.ts:19`). Còn `window.__lvhv` (BM3-06) |
| Lõi / FFI | `OpenOptions { force_migration_failure, now_unix }` chỉ có ở `Store::open_with`; FFI chỉ gọi `Store::open` (không lộ tùy chọn). `lvhv-store-tool` và `uniffi-bindgen` là nhị phân riêng, không vào `.so/.a/.dll` (`uniffi-bindgen` cần tính năng `bindgen-cli`) |
| Android | `SelfCheckActivity` chỉ ở `src/debug`; CI kiểm vắng trong APK release; thành phần exported chỉ `MainActivity` (+ `ProfileInstallReceiver` có quyền `DUMP`) |
| Apple | `DemoSeed`, `lvhvStoreDir`, `lvhvScreen`, `lvhvPrefill`, `demoRestore` đều trong `#if DEBUG`; bản `!DEBUG` có `enum DemoSeed {}` rỗng (`PeopleView.swift:481-485`) |
| Windows | Còn chế độ CI trong bản phát hành (BM3-07) |

## 8. Bằng chứng nguyên văn khác

### 8.1 Kiểm thử của dự án (main `8835cd7`)

```
$ cargo test --release -p lvhv-backup -p lvhv-store          # main 8835cd7, rustc/cargo 1.97.0, Linux x86_64
lvhv_backup (unittests)        ok. 2 passed      lvhv_backup tests/vectors.rs  ok. 3 passed
lvhv_store (unittests)         ok. 5 passed      lvhv_store tests/example.rs   ok. 1 passed
lvhv_store tests/media.rs      ok. 3 passed      lvhv_store tests/notice.rs    ok. 4 passed
lvhv_store tests/r6.rs         ok. 13 passed (44.01 s)
lvhv_store tests/released.rs   ok. 8 passed
EXIT 0
```

Các kiểm thử đạt; `released.rs` chưa có ca "sau 7 ngày không còn tệp ảnh mồ côi" (BM3-01), `media.rs` chưa có ca phần
nối sau EOI và SOF cao = 0 (BM3-04, BM3-05).

### 8.2 CI Android `ab83b9b` (run 42351718963, job `build`)

```
package: vn.clevix.lichviet
uses-permission: name='android.permission.POST_NOTIFICATIONS'
uses-permission: name='android.permission.RECEIVE_BOOT_COMPLETED'
permission: vn.clevix.lichviet.DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION
uses-permission: name='vn.clevix.lichviet.DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION'
A: http://schemas.android.com/apk/res/android:allowBackup(0x01010280)=false
exported: {'kind': 'activity', 'name': 'vn.clevix.lichviet.MainActivity', 'exported': 'true'}
exported: {'kind': 'receiver', 'name': 'androidx.profileinstaller.ProfileInstallReceiver', 'permission': 'android.permission.DUMP', 'exported': 'true'}
Phụ thuộc trong APK release: 84
Scanned …/gradle.lockfile file and found 84 packages
No issues found
```

Bước "Kiểm tra — quyền đúng danh sách cho phép (S6), không INTERNET" và "Kiểm tra — S5 … manifest đã gộp": `success`.

### 8.3 Phụ thuộc

```
$ cargo audit --db <advisory-db 309392b1> -f Cargo.lock                    # main 8835cd7, 120 gói
basic-toml 0.1.10  unmaintained  RUSTSEC-2026-0374      (ngoại lệ ở deny.toml + .cargo/audit.toml, xem lại 2027-02)
$ cargo deny check --disable-fetch advisories bans                         # main, all-features = true
advisories ok, bans ok
$ cargo audit … -f apps/windows/Cargo.lock                                 # #25 5f4dfcc, 586 gói, mọi đích
quick-xml 0.30.0  RUSTSEC-2026-0194, RUSTSEC-2026-0195   (chỉ qua phụ thuộc Linux, xem vòng 1 §8.3)
9 cảnh báo unmaintained (basic-toml, paste, pin-utils, rustybuzz, ttf-parser×2, zbus_names, zvariant, zvariant_derive)
$ cargo deny --manifest-path apps/windows/Cargo.toml check --disable-fetch advisories bans   # đích Windows
advisories ok, bans ok
$ (cd pwa && npm audit --omit=dev; npm audit)                             # #30 9e590d8
found 0 vulnerabilities
found 0 vulnerabilities
```

## 9. Kết luận theo PR

| PR | Kết luận | Điều kiện |
|---|---|---|
| main (`8835cd7`) | **Không còn mục chặn** | BM3-01, BM2-11 nên sửa; BM3-04, BM3-05, BM2-21 góp ý |
| #30 PWA | **Gộp được về mặt bảo mật** | BM3-06 góp ý |
| #24 Android | **Gộp được về mặt bảo mật** | không còn mục nên sửa |
| #25 Windows | **Gộp được về mặt bảo mật**, nên sửa BM3-02 trước phát hành công khai | BM3-02 (S10); BM3-07, BM2-15, BM2-22 góp ý |
| #23 Apple | **Gộp được về mặt bảo mật khi CI xanh** (job `build` của `e3e1ddf` chưa xong lúc rà); nên sửa BM3-03 trước phát hành công khai | BM3-03 (S10) |

## 10. Trạng thái tiêu chí G4 (vòng 2)

| Tiêu chí | PWA | Android | iOS | macOS | Windows | Ghi chú |
|---|---|---|---|---|---|---|
| S1, S2 | ĐẠT (lõi) | — | — | — | — | `core/` không đổi từ vòng 1 (`git diff 6ee647b 8835cd7 -- core` rỗng), `#![forbid(unsafe_code)]`; mật mã/lưu trữ ở crate riêng |
| S2b | **ĐẠT** | — | — | — | — | `dist/index.html` của #30 dựng lại: CSP giống từng byte 0.1.1 |
| S4' | CHƯA ĐỦ BẰNG CHỨNG | — | — | — | — | `smoke.mjs`/`people-flows.mjs` kiểm `localStorage` (chỉ `lvhv.ui`), `sessionStorage`, cookie, IndexedDB `lvhv`; chưa thấy kiểm Cache Storage và URL lịch sử sau kịch bản R7 |
| S5 | — | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | không áp dụng | Android: CI kiểm manifest đã gộp của APK release (`allowBackup=false`, có `dataExtractionRules`) — thiếu `bmgr backupnow`. Apple: `PeopleGlueTests.swift:180` khẳng định `isExcludedFromBackup` trên mô phỏng, nhưng CI Apple của commit đang rà chưa xong |
| S6 | — | **ĐẠT** | — | — | — | Đúng phương pháp đăng ký (`aapt2 dump permissions` trên APK release), CI run 42351718963: đúng và chỉ ba quyền cho phép |
| S7 | — | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | Kiểm tĩnh sạch; thiếu đo động R7 |
| S8 | ĐẠT | ĐẠT | ĐẠT | ĐẠT | ĐẠT | Không đổi so với vòng 1; `media.rs`/`media.ts` chỉ đọc phần đầu tệp, không phải mật mã |
| S9 | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | Vi phạm của vòng 1 đã hết: mọi đường ghi từ chối JPEG có APP1/APP13 và WebP có EXIF/XMP (PoC §3.1), PWA `people-flows.mjs:131, 665` kiểm ảnh lưu không APP1. Còn thiếu: chạy đủ bộ 10 ảnh mẫu của S9 (HEIC, ảnh xoay 90°, hướng ảnh đúng) trên từng nền tảng; phần nối sau EOI (BM3-04) |
| S10 | ĐẠT (theo kiểm thử giao diện) | CHƯA ĐỦ BẰNG CHỨNG | **KHÔNG ĐẠT** | **KHÔNG ĐẠT** | **KHÔNG ĐẠT** | PWA: `.ics`, sao lưu, tải tệp đính kèm đều qua hộp, có kiểm thử. Android: mọi đường đã thấy đều có hộp (rà mã), chưa đọc kiểm thử giao diện S10. Windows: BM3-02. Apple: BM3-03 (suy từ hành vi mặc định của Quick Look) |
| S11 | **ĐẠT** | **ĐẠT** | **ĐẠT** | **ĐẠT** | **ĐẠT** | npm 0; Rust gốc: 0 lỗ hổng, 1 cảnh báo có ngoại lệ ghi lý do/ngày; Android: osv-scanner 0 trên phụ thuộc APK release; Windows: `cargo deny` theo đích Windows ok (ngoại lệ có lý do/ngày). Ghi chú: `quick-xml` có lỗ hổng vẫn nằm trong lockfile Windows nhưng không trong bản Windows |
| S12 | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | Vi phạm của vòng 1 đã hết: tệp độc bị từ chối trước khi ghi (PoC §3.1), mọi nền tảng giải mã hiển thị có trần (BM2-10). Còn thiếu: đo bộ nhớ đỉnh < 200 MB với ba tệp đăng ký (`m_cost` 4 GiB, JSON 10 000 cấp, 10 000 ảnh rỗng) trên từng nền tảng |

**Sẵn sàng G4: CHƯA**, nhưng không còn tiêu chí nào KHÔNG ĐẠT vì lỗ hổng của lõi. Còn KHÔNG ĐẠT: S10 trên Windows,
iOS, macOS (BM3-02, BM3-03, sửa nhỏ ở giao diện). Phần còn lại là CHƯA ĐỦ BẰNG CHỨNG vì thiếu đo trên thiết bị/trình
giả lập (S5 động, S7, S9 bộ 10 ảnh, S12 bộ nhớ đỉnh, S4' Cache Storage/lịch sử). Theo 04 §4, các mục CHƯA ĐỦ BẰNG
CHỨNG không chặn bản thử nghiệm nội bộ nếu chủ dự án ghi chấp nhận rủi ro; mục KHÔNG ĐẠT thì chặn.

Ghi chú trung thực: các mục chuyển từ KHÔNG ĐẠT (vòng 1) sang CHƯA ĐỦ BẰNG CHỨNG ở S9/S12 là vì **vi phạm đã chứng
minh không còn tái hiện được**, không phải vì đã đo đủ theo phương pháp đăng ký. Không mục nào được ghi ĐẠT chỉ nhờ việc
sửa mã.
