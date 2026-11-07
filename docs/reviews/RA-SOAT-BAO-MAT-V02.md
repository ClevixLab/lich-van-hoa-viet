# Rà soát bảo mật và quyền riêng tư — v0.2 đợt 3 (T10) và mô hình đe dọa v0.2

Vai: rà soát bảo mật và quyền riêng tư (T10, docs/v0.2/07 §7) — bên không tham gia xây dựng đặc tả và không viết mã
đang duyệt. Ngày: 2026-11-07. Không sửa mã đang duyệt; mọi bản thử nghiệm (PoC) nằm ngoài kho.

Tài liệu đã đọc: docs/v0.2/01, 02 (phần liên quan), 03, 04 (S1–S12, B2–B5, R6, R7, R14), 07 (R-PRIV-2, R-PDF-1), 08;
docs/reviews/MO-HINH-DE-DOA.md, RA-SOAT-BAO-MAT.md, BIEN-BAN-S1-S4.md, DUYET-MA-DOT3.md.

## 0. Phạm vi và phiên bản

| Nhánh | Commit | Nội dung |
|---|---|---|
| `main` | `6ee647b` | lõi, `lvhv-store`, `lvhv-backup`, FFI, PWA (#15), B5 |
| PR #28 `v0.2-w3-core2` | `5535a49` | tệp đính kèm trong kho và trong tệp sao lưu, `check_header`, quy tắc mật khẩu |
| PR #30 `v0.2-w3-pwa2` | `c9f7995` | PWA vòng 2: tệp đính kèm (ảnh/PDF), chuyển đổi IndexedDB v1 → v2 (R14) |
| PR #24 `v0.2-w3-android` | `94dcc11` | Android: mô-đun Người thân, R14 |
| PR #25 `v0.2-w3-windows` | `42e7d4d` | Windows: mô-đun Người thân, R14, giao thức `lichviet:` |
| PR #23 `v0.2-w3-apple` | `fc4ab56` | iPhone/iPad/Mac: mô-đun Người thân, R14 |

Ghi chú quan trọng về phạm vi: **chỉ PWA (#30) và lõi (#28) có tệp đính kèm YC-2026-07 (ảnh/PDF).** Ba ứng dụng native
vẫn dùng mô hình cũ `certificate_photo_ids` (ảnh giấy báo tử là `Photo`), không có PDF, không mở tệp bằng trình xem
ngoài. Vì vậy các mục "mở PDF bằng ứng dụng ngoài, tệp tạm, quyền content-URI của Android, bản sao tạm trên Windows"
**không có mã để rà** trên native — ghi CHƯA ĐỦ BẰNG CHỨNG, kèm yêu cầu đăng ký trước ở §5.3 để áp khi các nền tảng
này làm tệp đính kèm.

Mức: **chặn** (phải sửa trước khi gộp PR hoặc trước G4), **nên sửa** (sửa trước phát hành công khai, hoặc chủ dự án
ghi chấp nhận rủi ro), **góp ý**.

## 1. Tổng hợp

| Mức | Số |
|---|---|
| Chặn | 3 |
| Nên sửa | 9 |
| Góp ý | 12 |

| Mã | Mức | Nền tảng / PR | Tiêu đề |
|---|---|---|---|
| BM2-01 | **chặn** | main, #28; ảnh hưởng mọi nền tảng | Ảnh và tệp đính kèm nhập từ tệp sao lưu được lưu nguyên byte: không giải mã/mã hóa lại (03 §2.3), không kiểm định dạng hay kích thước điểm ảnh → bom giải nén làm ứng dụng sập mỗi lần mở hồ sơ; EXIF/GPS của người khác vào kho |
| BM2-02 | **chặn** | #28 (B5) | Lõi bỏ `certificate_photo_ids` mà không có hàm chuyển đổi: dữ liệu và tệp sao lưu 0.2-dev bị từ chối; ảnh giấy báo tử cũ thành ảnh mồ côi không bao giờ bị xóa |
| BM2-03 | **chặn** | #28 ↔ #30 | Hai PR cài đặt tệp đính kèm theo hai cách không tương thích (định dạng tệp sao lưu, lược đồ, chuyển đổi) — gộp cả hai sẽ cho một PWA hoặc mất tệp đính kèm khi sao lưu, hoặc không đọc được tệp của native |
| BM2-04 | nên sửa | Android #24, Apple #23 | Ảnh màn hình đa nhiệm (Recents / app switcher) hiện thông tin báo tử, ảnh giấy chứng tử, ô mật khẩu sao lưu |
| BM2-05 | nên sửa | #30 (PWA) | Tải tệp đính kèm ra thư mục Tải về không qua hộp xác nhận (S10) và không báo tệp sẽ nằm ngoài ứng dụng |
| BM2-06 | nên sửa | #28 (lõi) | Bản sao trước chuyển đổi `lvhv.sqlite.pre-v1.bak`: không bao giờ xóa (01 §7 yêu cầu 7 ngày), không bị xóa khi xóa hồ sơ/thay toàn bộ |
| BM2-07 | nên sửa | #30 (PWA) | Kho `legacy_v1` giữ bản sao thông tin báo tử và ảnh giấy chứng tử mãi mãi; xóa hồ sơ, "Thay toàn bộ" không xóa |
| BM2-08 | nên sửa | #28 (lõi) | `add_attachment` gắn `stripped = true` cho mọi tệp không phải PDF mà không kiểm byte là JPEG/WebP đã bỏ siêu dữ liệu |
| BM2-09 | nên sửa | #25 (Windows), CI | `cargo deny` không chạy cho workspace riêng `apps/windows` (Slint, `image`, `windows`); Gradle của Android không có kiểm tra lỗ hổng (S11) |
| BM2-10 | nên sửa | Android, Windows, Apple | Giải mã ảnh để hiển thị không giới hạn kích thước điểm ảnh (`BitmapFactory.decodeByteArray`, `image::load_from_memory` + `to_rgba8`, `CGImageSourceCreateImageAtIndex`) |
| BM2-11 | nên sửa | #28 (lõi), mọi native | SQLite không bật `secure_delete`: lý do mất, ghi chú báo tử đã xóa vẫn nằm trong trang trống của tệp CSDL và WAL |
| BM2-12 | nên sửa | #30 (PWA) | Thông báo "Tệp PDF được lưu nguyên bản" chỉ hiện sau khi đã chọn tệp; không có lựa chọn bỏ; hộp xác nhận xuất sao lưu không nhắc PDF còn siêu dữ liệu (R-PDF-1) |
| BM2-13 | góp ý | #30 (PWA) | `sniffSize` trả `null` với HEIC/AVIF/TIFF/JPEG có SOF sau 256 KB → không có trần điểm ảnh trước `createImageBitmap` |
| BM2-14 | góp ý | Apple #23 | `.textContentType(.password)` cho mật khẩu sao lưu: hệ điều hành đề nghị lưu vào iCloud Keychain (BK-I5 "không lưu mật khẩu") |
| BM2-15 | góp ý | Windows #25 | URI `lichviet:` mang nguyên `content` (tiêu đề + nội dung thông báo, có tên khi O-9 bật) trên dòng lệnh; trang web bất kỳ mở được hồ sơ theo UUID |
| BM2-16 | góp ý | Android #24 | Tệp `.ics` ở `cache/ics/` còn lại tới lần chia sẻ sau (có tên, ngày giỗ) |
| BM2-17 | góp ý | mọi native | `VerifiedBackup` (bản rõ đã giải mã, gồm thông tin báo tử) và mật khẩu nằm trong trạng thái giao diện, không xóa (tiếp MA3-14-05) |
| BM2-18 | góp ý | Android #24 | `SelfCheckActivity` (debug) `exported=true`; CI nên khẳng định nó vắng trong manifest release |
| BM2-19 | góp ý | Android #24 | `MainActivity` (exported) nhận `EXTRA_OPEN` từ ứng dụng bất kỳ → mở thẳng trang hồ sơ có thông tin báo tử |
| BM2-20 | góp ý | Apple #23 | Lớp bảo vệ tệp mặc định (`completeUntilFirstUserAuthentication`); thư mục ảnh/tệp đính kèm có thể dùng `.complete` |
| BM2-21 | góp ý | #28 (lõi) | Bộ nhớ đỉnh với tệp sao lưu hợp lệ 128 MiB > 200 MB (bản sao `bytes.clone()` trong `verify_contents`, `apply_import`) |
| BM2-22 | góp ý | Windows #25 | Gỡ cài đặt giữ dữ liệu (đúng B5) nhưng không có chỗ nào cho người dùng biết hay xóa; `data_dir` lùi về `%TEMP%` nếu thiếu `LOCALAPPDATA` |
| BM2-23 | góp ý | Windows, Android | Kiểm độ dài/so khớp mật khẩu ở giao diện không theo NFC (lõi #28 đã có `backup_password_error`) |
| BM2-24 | góp ý | CI | Các action GitHub ghim theo nhãn (`@v2`, `@v4`), không theo SHA |

## 2. Đã chạy

| Việc | Kết quả |
|---|---|
| `cargo test --release -p lvhv-backup -p lvhv-store -p lvhv-ffi` trên #28 (`5535a49`) | xem §8 (bằng chứng nguyên văn) |
| PoC BM2-01 (crate riêng trong thư mục nháp, gọi `lvhv_store::open_backup` + `apply_import` của #28) | xem §8 |
| `npm audit --omit=dev` và `npm audit` (cả dev) trong `pwa/` (main; `package-lock.json` của #28, #30 giống hệt main) | `found 0 vulnerabilities` cả hai |
| `cargo audit` / `cargo deny` | xem §8 |
| So chuỗi CSP sinh từ `pwa/build.mjs` của v0.1.1, main, #28, #30 | giống hệt từng byte (S2b) |
| Rà grep rò rỉ: `notice`, `reason`, `reported_by`, `attachments` trong `memorial/`, `tsport/src/memorial*.ts`, mã thông báo/ics/tưởng niệm của bốn nền tảng; `console.*`, `Log.*`, `print`, `localStorage`, `caches`, `document.title`, `createObjectURL`, `innerHTML`, clipboard, share | không có đường nào đưa thông tin báo tử vào thông báo, .ics, tưởng niệm, tên tệp, URL, nhật ký; chi tiết §4 |

Không chạy được (ghi rõ): ứng dụng Android/iOS/macOS/Windows trên máy thật hay trình giả lập (môi trường rà soát là
Linux không có SDK); đo động R7/S7; `adb shell bmgr backupnow` (S5); ảnh Recents thật (BM2-04 suy từ mã: không có
`FLAG_SECURE`, `setRecentsScreenshotEnabled`, hay che khi `scenePhase` ≠ `.active`).

## 3. Mô hình đe dọa v0.2 (cập nhật MO-HINH-DE-DOA.md của v0.1 và 03 §3)

### 3.1 Tài sản

| Mã | Tài sản | Độ nhạy |
|---|---|---|
| A1 | Hồ sơ người thân: tên, danh xưng, quan hệ, giới tính, ngày sinh/mất, nơi an táng, ảnh chân dung | Cá nhân (của người dùng và của bên thứ ba) |
| A2 | **Thông tin báo tử** (01 §3.2a): người báo tin, nơi mất, **lý do mất**, ngày nhận tin, ghi chú | **Nhạy cảm cao** (có thể là thông tin sức khỏe) — R-PRIV-2 |
| A3 | **Tệp đính kèm** (01 §3.8a): ảnh tin báo tử, ảnh/PDF giấy chứng tử (có số giấy tờ, nơi cấp, đôi khi số định danh) | **Nhạy cảm cao**; PDF còn siêu dữ liệu máy quét (R-PDF-1) |
| A4 | Tệp sao lưu `.lvhv` và mật khẩu của nó | Chứa A1–A3 |
| A5 | Bản sao trước chuyển đổi (01 §7, B5): `lvhv.sqlite.pre-vN.bak` (native), kho `legacy_v1` (PWA) | Chứa A1–A3 **không mã hóa** |
| A6 | Lịch nhắc đã đặt trong hệ điều hành; tệp `.ics` | Tên, ngày giỗ (không có A2/A3) |
| A7 | Tính sẵn sàng của ứng dụng và của dữ liệu khi nâng cấp | Mất dữ liệu = hậu quả lớn nhất với người dùng |

### 3.2 Ranh giới tin cậy

1. Hộp cát ứng dụng ↔ hệ điều hành (thông báo, Recents/app switcher, sao lưu đám mây của hệ điều hành, bàn phím, bộ chọn
   tệp, bảng chia sẻ).
2. Ứng dụng ↔ tệp người dùng đưa vào: ảnh, PDF, **tệp sao lưu do người khác tạo** (BK-S1: kẻ gửi biết mật khẩu của tệp
   do chính họ tạo, nên AEAD **không** là lớp lọc với tệp độc hại).
3. Ứng dụng ↔ tệp ra ngoài: `.lvhv`, `.ics`, tệp đính kèm tải về (PWA), ảnh trong bảng chia sẻ.
4. Ứng dụng ↔ ứng dụng khác/trang web: Intent Android (exported activity), giao thức `lichviet:` trên Windows.
5. Phiên bản cũ ↔ phiên bản mới của chính ứng dụng (B5): dữ liệu cũ là đầu vào mà mã mới phải đọc được.

### 3.3 Mối đe dọa (STRIDE) — phần mới hoặc đổi so với v0.1 và 03 §3

| # | Loại | Mối đe dọa | Biện pháp đã có | Khoảng trống (phát hiện) | Rủi ro còn lại |
|---|---|---|---|---|---|
| T-I1 | Lộ | A2/A3 lên màn hình khóa qua thông báo | Lõi không đọc `notice` khi sinh lời nhắc; O-9 mặc định ẩn tên; Android `setPublicVersion` | — | Thấp |
| T-I2 | Lộ | A2/A3 vào `.ics`, tưởng niệm, tên tệp, URL | `memorial/` không tham chiếu `notice`; PWA `snap()` bỏ `notice`; tên tệp tải về chung (`TepDinhKem-n.pdf`); URL chỉ có UUID | — | Thấp |
| T-I3 | Lộ | A2/A3 trong ảnh Recents / app switcher | Không có | BM2-04 | Trung bình (ai cầm máy đã mở khóa) |
| T-I4 | Lộ | A3 ra thư mục Tải về / ứng dụng xem PDF | PWA: tên chung, chỉ tải khi bấm | BM2-05, BM2-12; native: chưa có mã (§5.3) | Trung bình |
| T-I5 | Lộ | Siêu dữ liệu ảnh (GPS, máy chụp) | Mã hóa lại khi **thêm** ảnh (bốn nền tảng) | BM2-01 (đường **nhập** không mã hóa lại), BM2-08 | Trung bình cho tới khi sửa |
| T-I6 | Lộ | PDF giữ siêu dữ liệu máy quét | Báo "lưu nguyên bản"; chỉ trong kho và tệp sao lưu mã hóa | BM2-12 | Thấp (chấp nhận theo R-PDF-1) |
| T-I7 | Lộ | Sao lưu đám mây của hệ điều hành chép A1–A5 | Android `allowBackup=false` + quy tắc loại trừ; Apple `isExcludedFromBackup` trên thư mục kho | — (S5 xem §6) | Thấp |
| T-I8 | Lộ | Dữ liệu đã xóa còn trên đĩa | Xóa tệp ảnh/tệp đính kèm khi gỡ khỏi hồ sơ | BM2-06, BM2-07, BM2-11 | Trung bình (pháp y trên máy đã mở khóa) |
| T-I9 | Lộ | Dò mật khẩu tệp sao lưu ngoại tuyến | Argon2id 64 MiB/3/4, ≥ 8 điểm mã | — (giữ BK-I1) | Trung bình với mật khẩu yếu |
| T-D1 | DoS | Tệp sao lưu độc: tham số KDF lớn, tệp lớn, JSON sâu | `check_header`/`parse_header` trước Argon2; kiểm kích thước tệp trước khi đọc (PWA, Android, Windows, Apple); JSON ≤ 32 cấp | — | Thấp |
| T-D2 | DoS | **Bom giải nén trong tệp sao lưu**: ảnh 1 MiB giải ra hàng GB, được lưu vào kho rồi giải mã mỗi lần hiện danh sách | Không có | **BM2-01, BM2-10** | **Cao** — sập lặp lại, phải xóa dữ liệu ứng dụng |
| T-D3 | DoS | Bom giải nén khi người dùng tự chọn ảnh | Trần 12 000 px / 50 Mpx trước khi giải mã (PWA, Windows, Apple); Android dùng `setTargetSize`/`inSampleSize` | BM2-13 | Thấp |
| T-D4 | DoS/mất dữ liệu | Nâng cấp làm mất hoặc từ chối dữ liệu cũ | Giao dịch, bản sao trước chuyển đổi, chế độ chỉ đọc (PWA #30) | **BM2-02, BM2-03** | Cao cho tới khi sửa |
| T-T1 | Can thiệp | Tệp sao lưu bị sửa, hạ tham số | AEAD, AAD = phần đầu, ngưỡng dưới | — | Không đáng kể |
| T-S1 | Giả mạo | Tệp sao lưu "của họ hàng" mang dữ liệu sai | Xem trước, mặc định Gộp | Xem trước không nói có tệp đính kèm/thông tin báo tử (góp ý trong BM2-01) | Thấp |
| T-E1 | Nâng quyền | Lỗi bộ giải mã ảnh của hệ điều hành bị khai thác bằng ảnh trong tệp sao lưu | 03 BK-E1 dựa vào "ảnh mã hóa lại" | BM2-01: ảnh trong kho là byte do kẻ gửi chọn, được giải mã lại **mỗi lần hiển thị** | Thấp–trung bình (phụ thuộc bộ giải mã của hệ điều hành) |
| T-E2 | Nâng quyền | Ứng dụng/trang web khác điều khiển ứng dụng qua Intent/URI | Chỉ mở hồ sơ theo UUID, không có thao tác ghi | BM2-15, BM2-19 | Thấp |

Điều v0.2 **không** bảo vệ (giữ như 03 §3): thiết bị đã bị chiếm quyền hoặc đã mở khóa trong tay người khác; dữ liệu
nằm không mã hóa trong kho (dựa vào mã hóa toàn thiết bị của hệ điều hành); người biết mật khẩu tệp sao lưu.

## 4. Luồng dữ liệu nhạy cảm (R-PRIV-2) — kết quả theo đích

| Đích | PWA (main, #30) | Android #24 | Windows #25 | Apple #23 |
|---|---|---|---|---|
| Thông báo | Không có thông báo hệ thống; thẻ trong trang dùng dữ liệu đã bỏ `notice` (`people/core.ts` `snap`) | Tiêu đề/nội dung lấy từ lõi (`Reminders.kt:192-208`), `VISIBILITY_PRIVATE` + `setPublicVersion` | `toasts.rs` dùng `title`/`body` của lõi | `ReminderScheduler.swift` dùng nội dung của lõi |
| `.ics` | Lõi TS, không đọc `notice` | Lõi Rust (`memorial/src/ics.rs` không tham chiếu `notice`) | như Android | như Android |
| Màn hình tưởng niệm | `memorial.ts` không đọc `notice` | `MemorialScreen.kt` không đọc | không đọc | `MemorialView.swift` không đọc |
| Tên tệp | `TepDinhKem-<n>.<ext>`, `LichVanHoaViet-saoluu-<ngày>.lvhv` | sao lưu: tên chung; ics: tên của lõi | như Android | như Android |
| Trên đĩa: tên tệp đính kèm | `file_name` gốc chỉ trong siêu dữ liệu | lõi: `attachments/<uuid>.bin`, ảnh `photos/<uuid>.img` | như Android | như Android |
| URL / tiêu đề trang | `#/nguoi-than/nguoi/<uuid>`; `document.title` chung | — | `lichviet:nguoi/<uuid>?id=…&c=<nội dung thông báo>` (BM2-15) | — |
| Nhật ký | `console.error(e)` chỉ với lỗi (thông điệp kiểm tra không chứa giá trị trường văn bản) | `Log.w` chỉ tên lớp ngoại lệ (`Reminders.kt:97`) | `eprintln!` chỉ ở chế độ `--kiem-thong-bao` của CI | Không `print`/`Logger` |
| Báo cáo sự cố | Không có | Không có SDK báo lỗi | `panic = "abort"`, không gửi | Không có |
| Clipboard / bảng chia sẻ | Không | Chỉ `.ics` qua `ACTION_SEND` (FileProvider `cache/ics/`) | Không | `ShareLink` cho `.ics` (tệp tạm xóa ở `onDisappear`) |
| Recents / app switcher | Trình duyệt quyết định | **Không che** (BM2-04) | Thumbnail Alt+Tab (chấp nhận) | **Không che** (BM2-04) |

Kết luận §4: các đích R-PRIV-2 đã đăng ký (thông báo, .ics, tưởng niệm, tên tệp, URL) **sạch** ở cả bốn nền tảng theo
rà tĩnh. Khoảng trống nằm ở các đích ngoài danh sách đã đăng ký: ảnh đa nhiệm (BM2-04) và tệp tải về (BM2-05).

## 5. Phát hiện chi tiết

### BM2-01 — **chặn** — Ảnh và tệp đính kèm nhập từ tệp sao lưu được lưu nguyên byte

- **Nơi:** main `store/src/lib.rs:768` (`apply_import`) và `:1079` (`verify_contents`); #28 `store/src/lib.rs:996-1042`
  (ghi `b.photos`/`b.attachments` thẳng bằng `write_atomic`) và `:1467-1497` (chỉ so `sha256`/`bytes`; với PDF chỉ
  kiểm 5 byte `%PDF-`); PWA main `pwa/src/store/db.ts:313-319` (ghi `Blob` từ tệp nhập). Không nền tảng native nào giải
  mã/mã hóa lại sau `apply_import`.
- **Yêu cầu bị vi phạm:** 03 §2.3 (KHÓA G1): "Ảnh nhập từ tệp sao lưu được giải mã và mã hóa lại qua đúng quy trình
  thêm ảnh … để ảnh do người khác tạo không đưa siêu dữ liệu hay dữ liệu dị dạng vào kho"; BK-E1, BK-I4 dựa vào điều
  này; S9 ("byte ảnh đã lưu … 0 đoạn APP1/APP13"); S12 ("ứng dụng không treo, không thoát").
- **Mối đe dọa:** BK-S1 — kẻ gửi tạo tệp `.lvhv` bằng mật khẩu của chính họ ("tệp gia phả của họ hàng"), nên mọi lớp
  AEAD đều qua. Bản ghi `Photo` khai `mime: image/jpeg`, `width: 768`, `height: 1024`, `stripped: true`, nhưng byte là
  bất cứ thứ gì có `sha256` khớp.
- **Kịch bản:** (1) ảnh chân dung là PNG 20 000 × 20 000 điểm ảnh, 389 KB (dưới trần 1 MiB). Sau khi người dùng bấm
  "Gộp", kho giữ nguyên byte này; mỗi lần danh sách hay trang hồ sơ hiện ảnh, Android gọi
  `BitmapFactory.decodeByteArray` không giới hạn (`people/Photos.kt:123`) → cần ~1,6 GB → `OutOfMemoryError`; Windows
  `image::load_from_memory(bytes)?.to_rgba8()` (`src/photo.rs:159`) → cấp 1,6 GB, bản release `panic = "abort"`;
  Apple `CGImageSourceCreateImageAtIndex` (`PhotoProcessing.swift:85`) → bị hệ điều hành giết vì bộ nhớ. Ứng dụng sập
  **mỗi lần mở mô-đun Người thân**; cách thoát duy nhất là xóa dữ liệu ứng dụng — tức mất toàn bộ sổ người thân nếu chưa
  có tệp sao lưu khác. (2) Ảnh JPEG còn EXIF GPS của người khác vào kho, rồi đi tiếp vào tệp sao lưu người dùng xuất
  (S9 phần "ảnh trong tệp sao lưu"). (3) Tệp đính kèm khai `image/jpeg` nhưng là HTML; tệp khai PDF chỉ cần mở đầu
  `%PDF-` (PWA tải về đúng tên `.pdf` và kiểu `application/pdf`).
- **Bằng chứng:** PoC §8.2 trên #28: `open_backup` + `apply_import(Merge)` chấp nhận và lưu nguyên cả ba loại byte.
- **Sửa (cụ thể):**
  1. Trong lõi, `verify_contents` (Rust) và `verifyContents` (TS) — **trước khi** ghi gì — kiểm cho mỗi khối có
     `mime` ảnh: (a) chữ ký đúng `mime` (JPEG `FF D8 FF`; WebP `RIFF????WEBP`); (b) đọc kích thước từ phần đầu (SOFn
     của JPEG, VP8/VP8L/VP8X của WebP — đã có `sniffSize` ở PWA, cần bản Rust tương đương, không giải mã) và đòi
     **bằng** `width`/`height` của bản ghi `Photo` (với `Attachment` ảnh: cạnh dài ≤ 2048); (c) JPEG không có đoạn
     APP1 (`FF E1`) hay APP13 (`FF ED`) trước SOS. Sai → lỗi chung "Sai mật khẩu hoặc tệp bị hỏng". Áp cùng kiểm tra
     cho `Store::add_photo`/`add_attachment` (BM2-08).
  2. Hoặc làm đúng 03 §2.3 ở mỗi nền tảng (giải mã có giới hạn rồi mã hóa lại trước `apply_import`); nếu chọn bước 1
     thay cho mã hóa lại thì ghi một Bổ sung cho 03 §2.3 (đổi biện pháp, chủ dự án ký) — không để đặc tả và mã lệch nhau.
  3. Thêm vào S12 các tệp mẫu: ảnh bom giải nén (PNG/WebP lớn khai `image/jpeg`), ảnh có EXIF GPS, tệp đính kèm "ảnh"
     là văn bản, "PDF" chỉ có `%PDF-` — kỳ vọng bị từ chối, kho không đổi.
  4. Xem trước khi nhập (`ImportPreview`) nên ghi số tệp đính kèm và "có thông tin báo tử" để người dùng biết mình
     đang nhận dữ liệu nhạy cảm của người khác (T-S1).

### BM2-02 — **chặn** — #28 bỏ `certificate_photo_ids` mà không chuyển đổi (B5, R14 (c))

- **Nơi:** #28 `store/src/validate.rs:486-531` (khóa `death_notice` chỉ còn `attachments`; `person_photo_refs` chỉ
  còn `photo_id`), `store/src/lib.rs:342-386` (`Store::open` không có bước dữ liệu nào cho đổi lược đồ này, và vì
  `SCHEMA_VERSION` vẫn là 1 nên nhánh sao lưu trước chuyển đổi cũng không chạy); `ffi/src/tests_v02.rs:950` khẳng định
  khóa cũ **bị từ chối**.
- **Bối cảnh:** main và ba ứng dụng native (#23, #24, #25) đang ghi `certificate_photo_ids`; PWA #30 đã có
  `migrateV1toV2`. 04 B5 mục 2: "Trường bị bỏ hay đổi tên phải được chuyển sang trường mới"; R14 (c) nêu đúng ca
  `certificate_photo_ids → attachments`.
- **Kịch bản:** máy đang chạy bản dựng 0.2 trước #28 (máy thử nội bộ, máy của chủ dự án) cập nhật lên bản có #28: (1)
  `put_person` của hồ sơ đó (kể cả sửa một chữ) và `apply_import` của tệp `.lvhv` xuất từ bản cũ đều lỗi V1 "trường lạ";
  (2) ảnh giấy báo tử cũ vẫn nằm trong bảng `photo` và `photos/` nhưng không còn được tham chiếu theo
  `person_photo_refs` → xóa hồ sơ **không** xóa các ảnh giấy chứng tử đó (A3 tồn tại mãi), và tài liệu sao lưu có ảnh
  không ai tham chiếu.
- **Sửa:** thêm hàm chuyển đổi thuần dùng chung (Rust + TS, như `migrateV1toV2` của #30): mỗi id trong
  `certificate_photo_ids` → một `Attachment` (`kind = "giay_chung_tu"`, cùng byte, `stripped = true`), bỏ bản ghi
  `Photo`; chạy (a) trong `Store::open` một lần, trong giao dịch, sau bản sao trước chuyển đổi (đánh dấu bằng khóa
  `meta`, hoặc tăng `schema_version` lên 2 với `migrate_1_to_2`), và (b) trong `verify_contents` trước kiểm lược đồ, để
  tệp sao lưu cũ nhập được (B5 mục 5). Thêm bộ mẫu đóng băng 0.2-dev vào `test-vectors/released/` (B5 mục 6) và ca
  R14 (c) vào kiểm thử lõi.

### BM2-03 — **chặn** — #28 và #30 không tương thích nhau

- **Nơi:** #28 `backup/src/lib.rs` (`MAX_BLOB` 10 MiB, tệp đính kèm là khối sau ảnh) và `pwa/src/store/*` của #28 (đọc
  /ghi `attachments` trong tệp sao lưu, từ chối `certificate_photo_ids`); #30 `pwa/src/store/db.ts` (`exportBackup`
  ném `ATTACHMENTS_UNSUPPORTED` khi có tệp đính kèm, `db.ts:402`; `backup-crypto.ts:143` vẫn giới hạn mỗi khối
  1 MiB và `verifyContents` (`db.ts:489-496`) coi khối tệp đính kèm là "ảnh thừa" — nên tệp sao lưu có tệp đính kèm
  do native/#28 xuất bị PWA #30 từ chối), IndexedDB v2 + `legacy_v1` chỉ có ở #30. Hai PR cùng sửa `pwa/src/store/db.ts`, `validate.ts`, `people/notice.ts`.
- **Kịch bản:** gộp #30 trước rồi #28 (hay ngược lại) bằng giải xung đột thủ công: dễ ra một PWA hoặc (a) xuất tệp sao
  lưu thiếu tệp đính kèm, hoặc (b) không mở được tệp `.lvhv` của native, hoặc (c) chạy hai lần chuyển đổi
  `certificate_photo_ids` khác nhau. Đây là rủi ro mất A3 và sai R6 (d) "các bản cài đặt đọc được tệp của nhau".
- **Sửa:** điều phối viên chốt **một** thiết kế trước khi gộp: định dạng khối của #28 cho tệp sao lưu + chuyển đổi
  của #30 (đưa vào tsport và Rust như BM2-02); rebase #30 lên #28 (hoặc ngược lại), bỏ `ATTACHMENTS_UNSUPPORTED`; chạy
  `scripts/check-store.sh` và kiểm chéo Rust ↔ PWA với vector `supplement-yc05` (có tệp đính kèm ảnh và PDF).

### BM2-04 — nên sửa — Ảnh đa nhiệm hiện thông tin báo tử (Android, Apple)

- **Nơi:** Android `MainActivity.kt` (không `FLAG_SECURE`, không `setRecentsScreenshotEnabled`); trang hồ sơ hiện
  thông tin báo tử và ảnh giấy báo tử (`ui/people/PersonScreen.kt`), biểu mẫu `DeathNotice.kt`, ô mật khẩu ở
  `BackupScreen.kt`. Apple `RootView.swift:30` có `scenePhase` nhưng không che nội dung khi `.inactive`/`.background`.
- **Kịch bản:** người dùng mở hồ sơ ông bà (có "Lý do: …", ảnh giấy chứng tử) rồi chuyển ứng dụng. Ảnh chụp Recents
  (Android) / app switcher (iOS) giữ đúng màn hình đó; ai cầm máy đã mở khóa vuốt đa nhiệm là đọc được, không cần mở
  ứng dụng. iOS còn lưu ảnh này trong `Library/Caches/Snapshots` của ứng dụng.
- **Sửa:** Android: khi đích điều hướng là trang hồ sơ có `notice`, biểu mẫu báo tử, hay khung sao lưu, đặt
  `window.setFlags(FLAG_SECURE, FLAG_SECURE)` (gỡ khi rời màn hình); trên API 33+ có thể dùng
  `setRecentsScreenshotEnabled(false)` cho cả Activity (không chặn chụp màn hình chủ động). Góp ý: `FLAG_SECURE` cũng
  chặn ảnh chụp màn hình của người dùng — nên chỉ áp cho các màn hình trên, không toàn ứng dụng. Apple: khi
  `scenePhase != .active` phủ một lớp che (màu nền + biểu tượng) lên `WindowGroup`, hoặc ít nhất ẩn khối báo tử. Ghi
  thêm đích "ảnh đa nhiệm" vào R-PRIV-2 (07 §6) và một ca kiểm thử giao diện.

### BM2-05 — nên sửa — PWA tải tệp đính kèm không qua hộp xác nhận (S10)

- **Nơi:** #30 `pwa/src/people/notice.ts:180-183` (nút "Mở / tải tệp PDF"/"Tải ảnh" gọi thẳng `download`),
  `people/backup.ts:25-33`.
- **Mối đe dọa:** tệp giấy chứng tử rời kho IndexedDB của ứng dụng vào thư mục Tải về chung (trình duyệt Android còn
  hiện thông báo "Đã tải xong TepDinhKem-1.pdf" và liệt kê trong ứng dụng Tệp/Tải xuống), nơi mọi ứng dụng có quyền
  bộ nhớ và ổ đám mây đồng bộ Tải về đọc được. S10: "Mọi đường xuất … hiện hộp xác nhận …; 0 đường xuất không qua hộp
  này".
- **Sửa:** trước `download`, hiện `confirmDialog` "Lưu tệp này ra thư mục Tải về của máy? Tệp sẽ nằm ngoài ứng dụng,
  không có mật khẩu, và không tự xóa khi bạn xóa hồ sơ." (+ "Tệp PDF còn thông tin của máy quét" với PDF); thêm đường này
  vào kiểm thử S10. Ghi vào 08 cùng cách làm cho native khi có (§5.3).

### BM2-06 — nên sửa — Bản sao trước chuyển đổi không bao giờ bị xóa (lõi)

- **Nơi:** main `store/src/lib.rs:342`, #28 `store/src/lib.rs:355-359`: chép `lvhv.sqlite` sang
  `lvhv.sqlite.pre-v{N}.bak`; không có mã nào xóa tệp này (`grep pre-v` chỉ ra một chỗ).
- **Yêu cầu:** 01 §7 "giữ 1 bản, xóa sau 7 ngày nếu chuyển đổi thành công".
- **Kịch bản:** sau lần tăng `schema_version` đầu tiên (0.3), tệp `.bak` chứa toàn bộ A1–A2 không mã hóa ở lại vĩnh
  viễn. Người dùng xóa hồ sơ (vì gửi nhầm, vì gia đình yêu cầu) hay "Thay toàn bộ" bằng tệp sao lưu: lý do mất và ghi
  chú báo tử vẫn nằm trong `.bak`. (Hiện chưa kích hoạt vì `SCHEMA_VERSION = 1`, nhưng mã đã nằm trên main.)
- **Sửa:** trong `Store::open`: nếu `.bak` tồn tại, `user_version == SCHEMA_VERSION` và mtime > 7 ngày → xóa; xóa
  `.bak` ngay khi `apply_import(Replace)` thành công và trong chức năng "xóa toàn bộ dữ liệu" (nếu có); chép bằng
  `VACUUM INTO` (một tệp nhất quán, không kèm trang trống chứa dữ liệu đã xóa) thay cho `fs::copy` sau checkpoint. Apple:
  thêm `.bak` vào danh sách `excludeFromBackup` cho rõ (thư mục kho đã được loại trừ nên hiện vẫn đạt S5).

### BM2-07 — nên sửa — PWA: kho `legacy_v1` giữ thông tin báo tử và ảnh giấy chứng tử mãi mãi

- **Nơi:** #30 `pwa/src/store/db.ts:99, 106-137` (ghi `persons` cũ và `photo:<id>` vào `legacy_v1`), `applyImport`
  và `deletePerson` không đụng `legacy_v1`; không có hạn giữ.
- **Kịch bản:** như BM2-06: xóa hồ sơ hay khôi phục "Thay toàn bộ" không xóa bản cũ của thông tin báo tử và ảnh giấy
  chứng tử; người dùng không thấy, không xuất được, không xóa được (chỉ xóa cả dữ liệu trang web).
- **Sửa:** ghi `at` (đã có) và xóa `legacy_v1` sau 7 ngày kể từ chuyển đổi thành công (cùng quy tắc 01 §7); xóa mục
  `persons` tương ứng và `photo:<id>` khi xóa hồ sơ đó; xóa toàn bộ `legacy_v1` khi "Thay toàn bộ" thành công.

### BM2-08 — nên sửa — `add_attachment` tin nền tảng đã bỏ siêu dữ liệu

- **Nơi:** #28 `store/src/lib.rs:591-621`: `stripped: !pdf` cho mọi `mime` không phải PDF; không kiểm chữ ký JPEG/WebP
  hay APP1/APP13; tương tự `add_photo` (`:718`).
- **Kịch bản:** một nền tảng (hiện tại hoặc sau này) gọi `add_attachment(…, "image/jpeg", …, byte gốc của máy ảnh)`
  vì quên bước mã hóa lại → kho ghi `stripped = true` cho ảnh còn GPS; S9 chỉ đo bằng 10 ảnh mẫu qua giao diện nên có
  thể không bắt được trên nền tảng chưa có kiểm thử.
- **Sửa:** dùng chung bộ kiểm của BM2-01 (chữ ký + không APP1/APP13 + kích thước) trong `add_photo`/`add_attachment`;
  lỗi → `Input("Ảnh chưa được xử lý bỏ siêu dữ liệu")`.

### BM2-09 — nên sửa — S11 không phủ phụ thuộc của Windows và Android

- **Nơi:** `.github/workflows/ci.yml:94-112` (`cargo-deny-action` với `--workspace` của workspace gốc). `apps/windows`
  là workspace riêng (`Cargo.toml` có `[workspace]`, `Cargo.lock` riêng: `slint =1.8.0`, `image =0.24.9`,
  `windows =0.58.0`, `png`, `winit`, `skia-safe`…) — **không** được kiểm. Android (Gradle: Compose, AndroidX, JNA) không
  có bước kiểm lỗ hổng.
- **Bằng chứng (§8.3):** `cargo audit` trên `apps/windows/Cargo.lock` (#25) với cơ sở dữ liệu RustSec tải lúc rà
  soát: **2 lỗ hổng** `quick-xml 0.30.0` (RUSTSEC-2026-0194 thời gian chạy bậc hai, RUSTSEC-2026-0195 cấp bộ nhớ không
  giới hạn; bản vá ≥ 0.41.0) và 9 cảnh báo "không còn bảo trì" (`rustybuzz`, `ttf-parser`, `paste`, `pin-utils`,
  `zvariant`…). Truy ngược: `quick-xml` chỉ vào qua `wayland-scanner` (dựng trên Linux) và `zbus_xml` ←
  `accesskit_unix` (Linux) — **không** có trong bản Windows, nên hiện không khai thác được trên Windows; nhưng không ai
  biết điều đó vì không có bước kiểm. Workspace gốc: 1 cảnh báo `basic-toml 0.1.10` không còn bảo trì
  (RUSTSEC-2026-0374, vào cơ sở dữ liệu 2026-10-20) qua `askama` ← `uniffi_bindgen` ← `uniffi` ← `lvhv-ffi`; `deny.toml`
  đặt `unmaintained = "all"` mà job `deps` của main (03:49Z) vẫn xanh — cần xác nhận `cargo deny` có thấy `uniffi_bindgen`
  trong đồ thị hay không (rà soát không chạy được `cargo deny` tại chỗ).
- **Kịch bản:** cảnh báo RustSec cho `image`/`png` (bộ giải mã ảnh — đúng bề mặt BM2-01) trong ứng dụng Windows không
  bao giờ làm CI đỏ.
- **Sửa:** thêm `cargo deny --manifest-path apps/windows/Cargo.toml check advisories bans` với
  `[graph] targets = ["x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"]` (để chỉ báo phụ thuộc thật sự vào bản
  Windows); ghi ngoại lệ có lý do và ngày cho `basic-toml` (chỉ dùng lúc sinh mã ràng buộc) nếu xác nhận đúng; với
  Android, bật Gradle dependency verification (`gradle/verification-metadata.xml`) và một bước quét (ví dụ
  `osv-scanner --lockfile` trên tệp khóa Gradle). Ghi vào biên bản S11 theo từng nền tảng.

### BM2-10 — nên sửa — Giải mã ảnh hiển thị không giới hạn điểm ảnh (native)

- **Nơi:** Android `people/Photos.kt:123`; Windows `src/photo.rs:159`; Apple `PhotoProcessing.swift:85-88`.
- **Liên quan:** BM2-01 (nguồn byte độc), nhưng là lớp phòng thủ thứ hai cần có dù BM2-01 đã sửa (kho có thể đã chứa
  byte độc từ bản trước khi sửa).
- **Sửa:** Android: `ImageDecoder` với `setTargetSize` theo kích thước khung (hoặc `inJustDecodeBounds` + từ chối khi
  > 2048 px); Windows: `image::io::Reader` + `Limits { max_image_width: Some(2048), max_image_height: Some(2048),
  max_alloc: Some(64 << 20) }`; Apple: `CGImageSourceCreateThumbnailAtIndex` với `kCGImageSourceThumbnailMaxPixelSize`
  (như lúc thêm ảnh). Ảnh vượt → hiện khung trống, không sập.

### BM2-11 — nên sửa — SQLite không `secure_delete`

- **Nơi:** main/#28 `store/src/lib.rs` `Store::open` (chỉ `foreign_keys`, `journal_mode = WAL`).
- **Kịch bản:** người dùng xóa "Lý do" hay cả hồ sơ; chuỗi cũ vẫn nằm trong trang trống của `lvhv.sqlite` và trong
  `lvhv.sqlite-wal` cho tới khi bị ghi đè — đọc được bằng công cụ pháp y trên máy đã mở khóa, hoặc trong bản sao
  `.bak` (BM2-06).
- **Sửa:** `conn.pragma_update(None, "secure_delete", "ON")` khi mở kho (chi phí nhỏ với kho cỡ này); sau xóa hồ sơ có
  `notice` gọi `PRAGMA wal_checkpoint(TRUNCATE)`.

### BM2-12 — nên sửa — PDF lưu nguyên bản: báo muộn, xuất sao lưu không nhắc (R-PDF-1)

- **Nơi:** #30 `pwa/src/people/notice.ts:23, 105, 129` — câu "Tệp PDF được lưu nguyên bản" hiện trong danh sách sau khi
  đã đọc tệp; hộp xuất sao lưu (`people/backup.ts`) chỉ ghi "gồm cả thông tin báo tử".
- **Sửa:** khi người dùng chọn PDF, hiện hộp xác nhận một lần ("PDF được lưu nguyên bản, có thể còn tên máy quét, phần
  mềm, ngày quét. Nếu không muốn, hãy chụp ảnh tờ giấy thay vì đính kèm PDF.") — người dùng chọn tiếp hay hủy; hộp xuất
  sao lưu ghi số tệp PDF. Không đề xuất tự làm sạch PDF (cần thư viện phân tích PDF — bề mặt tấn công lớn hơn rủi ro).

### BM2-13 … BM2-24 — góp ý

- **BM2-13** (#30 `pwa/src/people/photo.ts:21-45, 56-60`): `sniffSize` không nhận HEIC/AVIF/TIFF/BMP và JPEG có SOF sau
  256 KB (ảnh có ảnh thu nhỏ EXIF lớn) → bỏ qua trần 12 000 px/50 Mpx. Sửa: khi `sniffSize` trả `null`, gọi
  `createImageBitmap(file, { resizeWidth: maxEdge, resizeQuality: "high" })` (trình duyệt giải mã có thu nhỏ) hoặc từ chối
  định dạng không nhận ra.
- **BM2-14** (#23 `BackupView.swift:603`): `.textContentType(.password)` khiến hệ điều hành đề nghị lưu mật khẩu vào
  iCloud Keychain — trái BK-I5 "không lưu mật khẩu" (dù có thể là điều người dùng muốn). Sửa: quyết định có chủ đích; nếu
  giữ "không lưu" thì bỏ `textContentType` (giữ `SecureField`, tắt tự sửa).
- **BM2-15** (#25 `src/toasts.rs:1-8, 100-106`, `wix/main.wxs` `ProtocolHandler`): `c=<content>` trong URI `lichviet:`
  là tiêu đề + nội dung thông báo (có tên khi O-9 bật), đi vào dòng lệnh của tiến trình và CSDL thông báo của Windows;
  chú thích "URI chỉ có UUID" không đúng. Bất kỳ trang web/ứng dụng nào cũng gọi được `lichviet:nguoi/<uuid>` để mở thẳng
  trang hồ sơ (hiện thông tin báo tử). Sửa: thay `c` bằng SHA-256 rút gọn của `content`; khi mở từ URI chỉ mở danh
  sách, không mở thẳng trang có khối báo tử.
- **BM2-16** (#24 `BackupScreen.kt:352-363`): `.ics` (tên, ngày giỗ) ở `cache/ics/` tới lần chia sẻ sau. Sửa: xóa
  `cache/ics/` ở `onResume` của lần quay lại sau chia sẻ hoặc khi mở ứng dụng.
- **BM2-17** (mọi native; #28 `ffi/src/v02.rs:244-249`): `VerifiedBackup` giữ JSON (có `notice`) và byte tệp đính kèm
  trong bộ nhớ suốt khi tấm xem trước mở; mật khẩu ở `String` Kotlin/Swift/Slint. Sửa: thả `VerifiedBackup` ngay khi
  đóng tấm; xóa ô mật khẩu sau khi dùng (Android/Apple đã xóa ở một số nhánh — làm ở mọi nhánh, kể cả lỗi).
- **BM2-18** (#24 `app/src/debug/AndroidManifest.xml`): `SelfCheckActivity` exported (nhận `op` từ Intent, ghi tệp
  trong `filesDir`). Chỉ có ở debug — đúng; thêm bước CI `aapt2 dump xmltree` trên APK release khẳng định không có.
- **BM2-19** (#24 `MainActivity.kt:111, 135-137`): `EXTRA_OPEN` từ Intent bất kỳ mở trang hồ sơ. Sửa: chỉ nhận
  `EXTRA_OPEN` khi Intent đến từ `PendingIntent` của chính ứng dụng (kèm một mã ngẫu nhiên lưu cục bộ), hoặc chỉ mở
  danh sách.
- **BM2-20** (#23): đặt `FileProtectionType.complete` cho `photos/`, `attachments/` (iOS chỉ đọc chúng khi ứng dụng đang
  mở); giữ mặc định cho `lvhv.sqlite` (bộ lập lịch cần đọc).
- **BM2-21** (#28 `store/src/lib.rs:1479, 1491, 1006, 1026`): `b.bytes.clone()` nhân đôi mọi khối khi nhập; với tệp
  hợp lệ 128 MiB, đỉnh ≈ tệp + bản rõ + `Contents` + `VerifiedBackup` + Argon2 64 MiB > 300 MB. S12 chỉ đòi < 200 MB với
  tệp độc (đã chặn trước), nhưng máy yếu có thể bị giết khi khôi phục tệp lớn hợp lệ. Sửa: chuyển quyền sở hữu
  (`std::mem::take`) thay vì `clone`.
- **BM2-22** (#25 `src/platform.rs:20-22`, `wix/main.wxs`): gỡ cài đặt giữ dữ liệu (đúng B5 mục 4) nhưng người dùng gỡ
  ứng dụng thường tin là đã xóa; thêm mục "Xóa toàn bộ dữ liệu trên máy này" và một dòng trong hướng dẫn gỡ. Thiếu
  `LOCALAPPDATA` thì nên báo lỗi thay vì ghi sổ người thân vào `%TEMP%` (bị Disk Cleanup dọn).
- **BM2-23** (#25 `src/people.rs:2803-2812`, #24 `BackupScreen.kt:145-146`): đếm độ dài bằng `chars()`/`String.length`
  và so `pw1 != pw2` trên chuỗi chưa NFC; lõi #28 đã có `backup_password_error` — dùng nó để mọi nền tảng cùng quy tắc.
- **BM2-24** (`.github/workflows/*.yml`): ghim action theo SHA đầy đủ (chuỗi cung ứng của bản phát hành).

### 5.3 Yêu cầu đăng ký trước cho tệp đính kèm trên native (chưa có mã — áp khi làm)

1. Ảnh đính kèm đi qua đúng bộ xử lý ảnh hiện có (cạnh dài ≤ 2048, ≤ 2 MB) và qua kiểm của BM2-08.
2. PDF: kiểm `%PDF-`, ≤ 10 MB **trước khi** đọc hết (dùng kích thước của bộ chọn tệp), không bao giờ giải mã/hiển thị
   PDF trong tiến trình ứng dụng.
3. Mở PDF bằng ứng dụng ngoài: Android — tệp tạm trong `cache/att/` (thêm `<cache-path name="att" path="att/"/>`, không
   mở rộng `ics`), tên chung `TepDinhKem.pdf`, `ACTION_VIEW` với `FLAG_GRANT_READ_URI_PERMISSION` **chỉ** cho Intent đó
   (không `grantUriPermission` cho gói khác), xóa tệp và `revokeUriPermission` khi quay lại; Windows — chép vào thư mục
   con riêng trong `%LOCALAPPDATA%\ClevixLab\LichVanHoaViet\tmp\` (không phải `%TEMP%` dùng chung), tên chung, xóa khi
   đóng ứng dụng và khi mở lần sau; Apple — `QLPreviewController` trên tệp tạm trong `tmp/` của hộp cát, xóa ở
   `onDisappear`. Mọi đường "Mở bằng…"/"Lưu ra…" qua hộp xác nhận như BM2-05.
4. Không đưa tệp đính kèm vào bảng chia sẻ trừ khi người dùng chọn rõ, có xác nhận.

## 6. Kết luận theo PR

| PR | Kết luận | Điều kiện |
|---|---|---|
| main (đã gộp: lõi, store, backup, FFI, PWA #15) | **Cần sửa** BM2-01 (đường nhập) trước G4 | BM2-06, BM2-11 nên sửa |
| #28 core2 | **CHƯA GỘP ĐƯỢC** | Sửa BM2-01, BM2-02, BM2-03 (chặn); BM2-08 nên sửa cùng lúc |
| #30 PWA vòng 2 | **CHƯA GỘP ĐƯỢC** | BM2-03 (chặn, phối hợp với #28); BM2-05, BM2-07, BM2-12 nên sửa. Phần CSP, IndexedDB, Service Worker, tên tệp tải về: ĐẠT |
| #24 Android | **Gộp được về mặt bảo mật** sau khi #28 có chuyển đổi (BM2-02) | BM2-04, BM2-10 nên sửa trước phát hành công khai; quyền, sao lưu, thông báo: ĐẠT theo rà tĩnh |
| #25 Windows | **Gộp được về mặt bảo mật** | BM2-09, BM2-10 nên sửa; BM2-15, BM2-22 góp ý |
| #23 Apple | **Gộp được về mặt bảo mật** | BM2-04, BM2-10 nên sửa; BM2-14, BM2-20 góp ý |

Ghi chú: "gộp được" của ba PR native là kết luận của rà soát bảo mật; còn phụ thuộc duyệt mã, duyệt thiết kế, và việc
#28 đổi lược đồ (BM2-02) — sau khi #28 vào main, cả ba phải chuyển từ `certificate_photo_ids` sang `attachments`.

## 7. Trạng thái tiêu chí G4

| Tiêu chí | PWA | Android | iOS | macOS | Windows | Ghi chú |
|---|---|---|---|---|---|---|
| S1, S2 | ĐẠT (lõi) | — | — | — | — | `core/` không đổi trong phạm vi này; mật mã ở `lvhv-backup`, lưu trữ ở `lvhv-store` (đúng 04 §3) |
| S2b CSP không đổi | **ĐẠT** | — | — | — | — | Chuỗi sinh từ `build.mjs` của main, #28, #30 giống hệt v0.1.1 (§2); chưa kiểm `dist/` bằng trình duyệt trong lần này |
| S4' chỉ IndexedDB | **CHƯA ĐỦ BẰNG CHỨNG** | — | — | — | — | Rà tĩnh sạch (`localStorage` chỉ `lvhv.ui`; SW chỉ cache tệp tĩnh; blob: chỉ cho tải về; URL chỉ UUID); chưa chạy kiểm trình duyệt sau kịch bản R7 trên #30 |
| S5 không sao lưu đám mây | — | **CHƯA ĐỦ BẰNG CHỨNG** | **CHƯA ĐỦ BẰNG CHỨNG** | **CHƯA ĐỦ BẰNG CHỨNG** | không áp dụng | Tĩnh: Android `allowBackup=false`, `fullBackupContent=false`, `dataExtractionRules` loại mọi miền; Apple `isExcludedFromBackup` trên thư mục kho, SQLite, WAL/SHM, `photos`. Thiếu `bmgr backupnow` và đọc thuộc tính trên thiết bị |
| S6 quyền Android | — | **CHƯA ĐỦ BẰNG CHỨNG** | — | — | — | Manifest nguồn: chỉ `POST_NOTIFICATIONS`, `RECEIVE_BOOT_COMPLETED`; không `INTERNET`, không quyền ảnh/bộ nhớ (Photo Picker, SAF). Thiếu `aapt2 dump permissions` trên APK release đã gộp manifest |
| S7 không mạng | — | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | Tĩnh: không `INTERNET`; Swift không `URLSession`/`Network`; mã Windows không dùng socket/HTTP; `tokio` có trong `apps/windows/Cargo.lock` chỉ qua `wasm-bindgen-futures` ← `winit` (đích wasm32, không vào bản Windows) — cần `cargo tree --target x86_64-pc-windows-msvc -e normal` trên CI để chốt. Thiếu đo động R7 |
| S8 không tự viết mật mã | **ĐẠT** | **ĐẠT** | **ĐẠT** | **ĐẠT** | **ĐẠT** | Chỉ `argon2`, `chacha20poly1305`, `zeroize`, `getrandom`, `@noble/*`, `crypto.getRandomValues`; `sha2`/WebCrypto chỉ cho toàn vẹn. Không nền tảng native nào gọi mật mã ngoài FFI |
| S9 ảnh không còn siêu dữ liệu | **KHÔNG ĐẠT** | **KHÔNG ĐẠT** | **KHÔNG ĐẠT** | **KHÔNG ĐẠT** | **KHÔNG ĐẠT** | Đường thêm ảnh: đạt theo mã (canvas / `Bitmap.compress` / `JPEGSanitizer` / `JpegEncoder`). Đường **nhập sao lưu** lưu nguyên byte (BM2-01) → "ảnh đã lưu và ảnh trong tệp sao lưu" có thể có APP1. Ghi KHÔNG ĐẠT vì đã chỉ ra được vi phạm bằng PoC; các phép đo 10 ảnh mẫu theo phương pháp đăng ký vẫn cần chạy |
| S10 xuất có xác nhận | **KHÔNG ĐẠT** | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | PWA: tải tệp đính kèm không qua hộp xác nhận (BM2-05). Native: .ics và sao lưu có hộp (rà mã), chưa chạy kiểm thử giao diện S10 |
| S11 phụ thuộc ghim, 0 cảnh báo | **ĐẠT** (npm) | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | CHƯA ĐỦ BẰNG CHỨNG | npm: ghim chính xác, `npm audit` 0. Crate mật mã Rust ghim `=x.y.z`. Workspace gốc (lõi dùng chung cho native): 1 cảnh báo `basic-toml` chưa xử lý (BM2-09); Windows: 2 lỗ hổng + 9 cảnh báo trong lockfile, có vẻ chỉ ở phụ thuộc Linux — chưa có bước kiểm theo đích; Gradle không được quét |
| S12 tệp sao lưu độc | **KHÔNG ĐẠT** | **KHÔNG ĐẠT** | **KHÔNG ĐẠT** | **KHÔNG ĐẠT** | **KHÔNG ĐẠT** | Phần đầu/kích thước/độ sâu JSON: chặn đúng trước khi cấp bộ nhớ (lõi, `check_header`; PWA, Android, Windows, Apple kiểm kích thước trước khi đọc). Nhưng tệp hợp lệ về mật mã mang ảnh bom giải nén được nhận vào kho (BM2-01, PoC) và làm ứng dụng thoát khi hiển thị (BM2-10) — trái "ứng dụng không treo, không thoát" |

Ghi chú trung thực về S9/S12: phương pháp đăng ký của S9 (10 ảnh mẫu qua đường thêm ảnh) và S12 (`m_cost` 4 GiB, JSON
10 000 cấp, 10 000 ảnh rỗng) không chứa ca của BM2-01; nếu chỉ chạy đúng các ca đó thì có thể ra ĐẠT. Rà soát này ghi
KHÔNG ĐẠT vì mệnh đề của hai tiêu chí ("byte ảnh đã lưu và ảnh trong tệp sao lưu", "ứng dụng không treo, không thoát")
bị vi phạm bằng một đầu vào cụ thể, và 03 §2.3 (khóa) yêu cầu biện pháp chưa được cài. Đề nghị bổ sung các ca của BM2-01
vào bộ đo S9/S12 bằng một Bổ sung ghi rõ "đề xuất sau khi đã thấy kết quả".

**Sẵn sàng G4: CHƯA.** Chặn G4: BM2-01 (S9, S12 KHÔNG ĐẠT), BM2-05 (S10 PWA KHÔNG ĐẠT), BM2-02/BM2-03 (R14 và R6 (d),
ngoài G4 nhưng chặn phát hành). Sau khi sửa, cần: đo S4' trên #30 bằng trình duyệt, S5/S6/S7 trên thiết bị hoặc trình
giả lập, S10 bằng kiểm thử giao diện, S11 cho Windows và Android. Có thể phát hành bản thử nghiệm nội bộ với các mục
CHƯA ĐỦ BẰNG CHỨNG nếu chủ dự án ghi chấp nhận rủi ro (04 §4); không với các mục KHÔNG ĐẠT.

## 8. Bằng chứng nguyên văn

### 8.1 Kiểm thử của dự án

(điền ở lần cập nhật kế tiếp)

### 8.2 PoC BM2-01

(điền ở lần cập nhật kế tiếp)

### 8.3 Kiểm tra phụ thuộc

```
$ (cd pwa && npm audit --omit=dev)   # main 6ee647b; package-lock.json của #28 và #30 giống hệt
found 0 vulnerabilities
$ (cd pwa && npm audit)
found 0 vulnerabilities

$ cargo audit --db <advisory-db @309392b1, 2026-11-06T16:34+01:00> -f Cargo.lock      # main 6ee647b; #28 giống
Scanning Cargo.lock for vulnerabilities (120 crate dependencies)
Crate:     basic-toml   Version: 0.1.10   Warning: unmaintained   ID: RUSTSEC-2026-0374
warning: 1 allowed warning found

$ cargo audit … -f apps/windows/Cargo.lock                                             # #25 42e7d4d
Scanning apps/windows/Cargo.lock for vulnerabilities (586 crate dependencies)
Crate: quick-xml 0.30.0  RUSTSEC-2026-0194  Quadratic run time when checking a start tag for duplicate attribute names
Crate: quick-xml 0.30.0  RUSTSEC-2026-0195  Unbounded namespace-declaration allocation in `NsReader` …
unmaintained: basic-toml 0.1.10, paste 1.0.15, pin-utils 0.1.1, rustybuzz 0.14.1, ttf-parser 0.21.1, ttf-parser 0.25.1,
              zbus_names 3.0.0, zvariant 4.2.0, zvariant_derive 4.2.0
error: 2 vulnerabilities found!
warning: 9 allowed warnings found

# truy ngược trong Cargo.lock (#25):
quick-xml ← wayland-scanner ← (smithay-client-toolkit | wayland-client) ← winit / softbuffer …   (Linux)
quick-xml ← zbus_xml ← zbus-lockstep ← atspi-common ← accesskit_atspi_common ← accesskit_unix     (Linux)
tokio     ← wasm-bindgen-futures ← winit                                                          (wasm32)
basic-toml ← askama_derive ← askama ← uniffi_bindgen ← uniffi ← lvhv-ffi
```

Job CI `deps` của main (`cargo-deny-action`, workspace gốc) kết thúc `success` lúc 2026-11-07T03:49:42Z (GitHub
check-runs của `6ee647b`). `cargo deny` không được cài trong môi trường rà soát (cài `cargo-audit` 0.22.2 thay thế).
