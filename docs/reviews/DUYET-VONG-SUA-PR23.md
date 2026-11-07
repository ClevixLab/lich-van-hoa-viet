# Duyệt vòng sửa PR #23 (Apple, mô-đun Người thân) — mã, thiết kế và các mục bảo mật

Trạng thái: **XONG** — 2026-11-07. Kết luận ở §4.

Bên duyệt: một bên không tham gia viết mã của PR #23; không sửa nhánh đang duyệt.

| Mục | Giá trị |
|---|---|
| PR | #23 `v0.2-w3-apple` |
| Đầu mã được duyệt | `181eedc` (gộp `main` 41cd3af — core3, core4, đặc tả YC-2026-10 — vào nhánh) |
| Đầu nhánh | `3ea63ff` — chỉ thêm ảnh chụp của CI (run `apple` 272, ba commit ảnh) |
| So với | `git diff origin/main...181eedc` (26 tệp ngoài ảnh, +6540 dòng) và `git diff f25ec2d 181eedc` (f25ec2d = đầu mã của `DUYET-MA-DOT3-NATIVE.md`) |
| CI | `apple` run 272 (id 42361938587) lần chạy 2: job `build` và `upgrade` `success`. Lần chạy 1 của job `build` thất bại ở `testPeriodFlow` với `kAXErrorServerNotFound` / "Timed out waiting for AX loaded notification" (lỗi máy mô phỏng, không phải lỗi kiểm thử). Nhật ký lần 2: XCTest 19/19 đạt, XCUITest 8/8 đạt (2 ca R14 bỏ qua có chủ đích — chỉ chạy trong job `upgrade`); dựng Release không ký cho iOS Simulator, iOS arm64, macOS; job `upgrade`: "R14 (Apple, mô phỏng): ĐẠT ca A, B và C" (`migration=migrated from=1 to=2`, `migration=read_only from=1 to=1`). `ci`, `android`, `windows` trên cùng commit: `success` |
| Ảnh | `apps/apple/screenshots/` ở `3ea63ff`: 196 ảnh iPhone/iPad + 6 ảnh `upgrade/`. Đã ghép sáng / tối / Accessibility L của cùng màn hình để so; xem kỹ các ảnh liên quan tới từng mục bên dưới |

Căn cứ: `docs/v0.2/08` §3 (§3.6, §3.8, §3.9), §12g, §12l và §12-Apple trên nhánh PR; `docs/v0.2/04` (S5, S9, S10, S12, B4–B6,
R14); các biên bản `DUYET-MA-DOT3-NATIVE.md` (PR #27), `DUYET-THIET-KE-DOT3-NATIVE.md` (PR #26), `DUYET-MA-DOT3-VONG-CUOI.md`,
`DUYET-THIET-KE-DOT3-VONG-CUOI.md`, `RA-SOAT-BAO-MAT-V02.md`, `RA-SOAT-BAO-MAT-V02-VONG2.md`; cách viết theo `DUYET-VONG-SUA-PR25.md`.

Giới hạn: không chạy được Xcode, máy mô phỏng hay thiết bị Apple. Mọi kết luận về hành vi lấy từ mã, nhật ký CI run 272 và ảnh
do CI chụp; chỗ nào là suy luận thì ghi rõ.

Mức: **chặn** / **nên sửa** / **góp ý**. Mã phát hiện mới: `DV23-<số>`.

## 1. Các mục vòng trước giao cho Apple

### 1.1 Duyệt mã (`DUYET-MA-DOT3-NATIVE.md` §3)

| Mã | Mức cũ | Kết quả | Bằng chứng |
|---|---|---|---|
| MA3N-23-01 | nên sửa | **ĐÃ SỬA** | Lõi core2 tách `export_backup` (chỉ tạo byte) và `mark_backup_saved`. `BackupView.swift:101-109`: `m.markBackupSaved()` chỉ trong nhánh `.success` của `fileExporter`; Hủy hay lỗi → không ghi nhận. `PeopleCore.swift:551-555`. Kiểm thử `testBackupCountsOnlyAfterSaved`: sau `exportBackup` thì `last_backup_at` vẫn rỗng, sau `markBackupSaved` mới có |
| MA3N-23-02 | nên sửa | **ĐÃ SỬA** phần chính; còn một chỗ (DV23-01) | `BackupView.swift:156-199`: kiểm kích thước, đọc 72 byte đầu + `backupHeaderCheck`, đọc tệp và `openBackup` (Argon2id) trong `Task.detached`; `PeopleCore.swift:597-603` `applyImport` chạy ở `Task.detached`, `reload` và lập lịch lại trên `MainActor`. `RestoreSheet` có "Đang khôi phục…", khóa nút "Khôi phục" và vuốt đóng (`:444`, `:454`). Nhưng nút "Hủy" trên thanh của tấm khôi phục không bị khóa khi đang chạy — DV23-01 |
| MA3N-23-03 | góp ý | **ĐÃ SỬA** | `PeopleGlueTests.swift:214-230`: `XCTAssertEqual(fake.store.count, 64)`, `lvhv:sentinel` cùng thời điểm với lời nhắc muộn nhất còn lại; ca B4 mới `testMarkDeceasedRemovesBirthdayNotifications` (`:234-252`) dùng `FakeCenter`: đặt nhắc sinh nhật → `markDeceasedJson` → lần sau `removed > 0`, không còn id sinh nhật, có "Sắp đến ngày giỗ". Cả hai đạt trong run 272 |
| MA3N-23-04 | góp ý | **ĐÃ SỬA** | `BGAppRefreshTask`: ghi là sai khác A-8 ở 08 §12-Apple kèm lý do (thông báo nhắc mở ứng dụng, P9). CI xanh trọn vẹn trên commit mã cuối `181eedc` (run 272, lần 2). Lần 1 hỏng vì máy mô phỏng — xem DV23-07 |

### 1.2 Duyệt thiết kế (`DUYET-THIET-KE-DOT3-NATIVE.md` §5)

| Mã | Mức cũ | Kết quả | Bằng chứng |
|---|---|---|---|
| TKN-I-01 | chặn | **ĐÃ SỬA** | `RootView.swift:93` `.tint(C.redInk)`; `redDeep` chỉ làm nền nút đặc. Ảnh `-dark`: "‹ Người thân" (`ios-person-deceased-dark`), "Hủy"/"Lưu" (`ios-mark-dark`, `ios-form-*-dark`), nút hộp xác nhận (`ios-att-pdf-confirm-dark`, `ios-restore-replace-confirm-dark`), "Lưu tệp…" (`ios-ics-confirm-dark`) đều là đỏ sáng, đọc rõ trên nền tối |
| TKN-I-02 | chặn (chưa đủ bằng chứng) | **ĐÃ SỬA** | Có đủ ảnh sáng/tối/Accessibility L: âm lịch + "Tháng nhuận (năm 2025 nhuận tháng 6)" + "Dương lịch: Thứ Sáu, 08/08/2025" (`ios-form-am-nhuan-*-2`), dương lịch + "Âm lịch: 15/6 nhuận năm Ất Tỵ (2025)" (`ios-form-duong-*`), lỗi dưới ô + khung "Còn 3 chỗ cần sửa" (`ios-form-loi-*`), sinh nhật âm (`ios-form-living-am-*`), thông báo bị tắt (`ios-upcoming-denied-*`), nhắc sao lưu (`ios-backup-nag-*`), lỗi kho (`ios-store-error-*`), tấm khôi phục và hộp "Thay toàn bộ" (`ios-restore-*`), bàn thờ sau khi cuộn (`ios-memorial-*-2`), xem trước thông báo màn hình khóa (`ios-settings-*`, A-12). Đã xem lại; phát hiện mới về tấm khôi phục ở DV23-03 |
| TKN-I-03 | nên sửa | **ĐÃ SỬA** | `ios-memorial-light`, `ios-memorial-a-light`, `-large`: thanh điều hướng nền gỗ, "‹ Về hồ sơ" và "Tưởng niệm" màu vàng đồng, đọc rõ ở cả sáng và tối |
| TKN-I-04 | nên sửa | **ĐÃ SỬA** | `PFmt.gioText` (`PeopleCore.swift:245-249`) không ghi "nhuận"; `ios-profiles-light`: "Giỗ 15/6 âm lịch · lần tới 18/07/2027"; "nhuận" chỉ còn ở dòng "Âm lịch" của bảng Thông tin |
| TKN-I-05 | nên sửa | **ĐÃ SỬA** trong mô-đun | Thanh trang con, Dương/Âm, "Biết đủ ngày / Chỉ biết năm", Tuần/Tháng/Quý/Năm, bộ lọc, giới tính đều là nút viên thuốc to theo chữ (`ios-period-quy-*`, `ios-form-*-large`). Nhóm A/A+/A++ của thanh trên v0.1 vẫn là `Picker` phân đoạn — sai khác A-13, chấp nhận. Tấm khôi phục lại dùng điều khiển hệ thống không to theo chữ — DV23-03 |
| TKN-I-06 | nên sửa | **ĐÃ SỬA** | Ô nhập có viền thấy rõ ở sáng (`ios-form-loi-light`, `ios-mark-light`, `ios-settings-light-2`); 08 §12-Apple ghi `#8F807D` / `#8A7B78` |
| TKN-I-07 | nên sửa (trước phát hành) | **ĐÃ SỬA** | Như MA3N-23-01; A-10 ghi "Đã đóng, core2" |
| TKN-I-08 | nên sửa (trước phát hành) | **ĐÃ SỬA** | Lõi core2 `save_person_json` xóa ảnh/tệp bị bỏ trong cùng giao dịch; lưu hỏng → ứng dụng xóa ảnh/tệp vừa ghi (`PersonFormView.swift:597-602`, `:731-734`). Kiểm thử `testAttachmentsAndPhotoCleanup`: bỏ PDF và ảnh chân dung rồi lưu → `attachmentBytes`/`photoBytes` báo lỗi |
| TKN-I-09 | nên sửa | **ĐÃ SỬA** | `MarkDeceasedView` (`PersonFormView.swift:686-694`): ẩn nút quay lại, Hủy / Lưu trên thanh, "Bỏ các thay đổi chưa lưu?" khi đã nhập. Ảnh `ios-mark-*` có "Hủy … Lưu" trên thanh |
| TKN-I-10 | góp ý | **ĐÃ SỬA** | `ios-ics-confirm-*`: nút "Lưu tệp…", tiêu đề "Xuất tệp lịch" không bị cắt ở Accessibility L, có câu "Không gồm thông tin báo tử" |
| TKN-I-11 | góp ý | **ĐÃ SỬA một phần** | Sắp tới hai cột trên iPad (`ipad-upcoming-light`). Trang hồ sơ và danh sách Hồ sơ vẫn một cột ở iPad dọc 820 pt, trái lời A-14 — DV23-05 |
| TKN-I-12 | góp ý | **ĐÃ SỬA** | Biểu tượng nén hương / mâm nhỏ / bánh có nến vẽ trong mã (`PeopleUI.swift` `IncenseIcon`, `TrayIcon`, `CakeIcon`); "Giỗ · Bà Trần Thị B" có dấu "·" (`ios-upcoming-light`) |
| TKN-I-13 | góp ý | **ĐÃ SỬA** | `ios-memorial-a-light`: "Ngày giỗ sắp tới: 15/6 âm lịch, 18/07/2027" |

Các mục chung giữa các nền tảng (vòng cuối, `DUYET-THIET-KE-DOT3-VONG-CUOI.md` §6 và 08 §12l):

| Mã | Kết quả trên Apple | Bằng chứng |
|---|---|---|
| TKN-X-02 / TKN3-A-02 năm trên bia không ngoặc | **ĐẠT** | `ios-memorial-*`: "Ất Hợi 1935 — Kỷ Hợi 2019", "Canh Thìn 1940 — Ất Tỵ 2025" |
| TKN-X-04 câu báo tử ở hộp .ics | **ĐẠT** | `BackupView.swift:234`, ảnh `ios-ics-confirm-*` |
| TKN-X-06 "Ngày giỗ sắp tới" | **ĐẠT** | như TKN-I-13 |
| TKN3-X-01 tên loại tệp đính kèm | **ĐẠT** | `PeopleDrafts.swift:361-369`: "Tin báo tử", "Giấy chứng tử", "Tệp khác" |
| TKN3-X-02 "Nhập theo dương lịch / âm lịch" ở mọi ô ngày | **CHƯA** | chỉ ô ngày mất; ngày sinh, ngày sự kiện vẫn "Dương lịch / Âm lịch" — DV23-06 (góp ý) |
| TKN3-X-03 câu chuẩn của khung chỉ đọc; TKN3-P-01 ẩn việc ghi ở chế độ chỉ đọc | **CHƯA** | DV23-02 |

### 1.3 Bảo mật (`RA-SOAT-BAO-MAT-V02.md`, `RA-SOAT-BAO-MAT-V02-VONG2.md`)

| Mã | Mức cũ | Kết quả | Bằng chứng |
|---|---|---|---|
| BM2-04 | nên sửa | **ĐÃ SỬA** cho cửa sổ và các tấm; còn trình xem PDF — DV23-04 | `PeopleUI.swift:676-695` `PrivacyCover` khi `scenePhase != .active`, áp ở `RootView.swift:94, 97`, `BackupView.swift:90, 99, 133`, trang xem ảnh đính kèm `PersonFormView.swift:223` |
| BM2-10 | nên sửa | **ĐÃ SỬA** | `PhotoProcessor.cgImage` (trần 12 000 px / 50 Mpx, `CGImageSourceCreateThumbnailAtIndex` ≤ 2048); kiểm thử `testDisplayDecodeIsPixelBounded` (13 000 px → không giải mã; 4000 × 3000 → ≤ 2048) |
| BM2-14 | góp ý | **ĐÃ SỬA** | `BackupView.swift:589-597` `passwordContent()` không đặt `textContentType(.password)` |
| BM2-17 | góp ý | **ĐÃ SỬA** phần giao diện | Mật khẩu khôi phục xóa ở mọi nhánh (`BackupView.swift:185, 196`); `verified = nil` khi đóng tấm (`:118`) và sau khôi phục (`:129`); ô mật khẩu tạo sao lưu xóa sau khi tạo (`:359`). Chuỗi Swift không xóa được trong bộ nhớ (giới hạn ngôn ngữ, như vòng 2) |
| BM2-20 | góp ý | **ĐÃ SỬA** | `PeopleCore.swift:94-107` `protectPhotos`: `photos/`, `attachments/` và từng tệp trong đó lớp `.complete` (iOS); `lvhv.sqlite` giữ mặc định. Gọi sau mở kho và sau mỗi lần thêm ảnh/tệp |
| §5.3 mục 1–3 | — | **ĐẠT** | Ảnh đính kèm qua `PhotoProcessor` (≤ 2048 px, ≤ 2 MB) + lõi; PDF: `fileSize` ≤ 10 MB trước khi đọc, rồi `%PDF-` (`PersonFormView.swift:103-118`), lõi kiểm thêm `%%EOF`; tệp tạm `tmp/att/TepDinhKem.pdf` (`PhotoProcessing.swift:171-183`), xóa khi đóng trình xem (`PersonFormView.swift:178`) và khi mở ứng dụng (`LichVietApp.swift:13`); kiểm thử `testAttachmentTempFileIsNeutralAndCleared` |
| BM3-03 (S10, §5.3 mục 3', 4) | nên sửa | **ĐÃ SỬA** | `AttachmentViewer` (`PersonFormView.swift:146-181`), dùng ở trang hồ sơ (`PersonDetailView.swift:35, 277-280`) và biểu mẫu (`PersonFormView.swift:99, 120-126`). Ảnh: giải mã có trần, xem trong tấm riêng của ứng dụng, chỉ có nút "Đóng", không ghi tệp tạm (`ios-att-image-*`). PDF: hộp "Mở tệp PDF?" nói rõ trình xem có nút Chia sẻ và có thể gửi tệp ra ngoài ứng dụng; "Hủy" là nút hủy và nút mặc định (in đậm trên ảnh `ios-att-pdf-confirm-*`); chỉ "Mở tệp" mới ghi bản chép tạm và mở Quick Look (`ios-att-pdf-open-light` cho thấy trình xem hệ thống có nút Chia sẻ và Đánh dấu, đúng như hộp đã báo). Kiểm thử giao diện `testAttachmentViewing` ở sáng, tối, Accessibility L: ảnh mở trong ứng dụng không có nút Chia sẻ; PDF hiện hộp có chữ "Chia sẻ"; Hủy → trình xem không mở; "Mở tệp" → trình xem phủ lên. Đạt trong run 272 |

Tóm lại: 4/4 mục duyệt mã, 13/13 mục duyệt thiết kế và 7/7 mục bảo mật giao cho Apple đã sửa phần chính. Còn lại: phần nút
Hủy của MA3N-23-02 (DV23-01), TKN-I-11 trên trang hồ sơ iPad (DV23-05), trình xem PDF ngoài lớp che của BM2-04 (DV23-04), và
hai mục chung của vòng cuối chưa áp cho Apple (TKN3-X-03/TKN3-P-01 — DV23-02; TKN3-X-02 — DV23-06).

## 2. Đúng và đã kiểm, không có lỗi

- **Chế độ chỉ đọc và sao lưu (tương ứng MA3C-30-01 của PWA):** ở `read_only`, "Tạo tệp sao lưu…" vẫn bấm được; lõi tạo byte;
  `markBackupSaved` bị lõi từ chối và lỗi bị bỏ qua (`try?`), nên không ghi gì vào kho cũ; người dùng vẫn nhận tệp. "Mở tệp" khôi
  phục bị khóa (`BackupView.swift:73`).
- **Tham số kiểm thử chỉ ở Debug (tương ứng MA3C-25-01 của Windows):** mọi khóa `lvhv*` đọc từ `UserDefaults` (`lvhvSeed`,
  `lvhvToday`, `lvhvStoreDir`, `lvhvScreen`, `lvhvPrefill`, `lvhvExpand`, `lvhvFakeDenied`, `lvhvNagDue`, `lvhvPeriodKind`) đều nằm
  trong `#if DEBUG` (đã quét toàn bộ `apps/apple/Shared`); bản Release có `enum DemoSeed {}` rỗng (`PeopleView.swift:486-490`).
- **Lập lịch lại:** `ReminderScheduler` chỉ đổi thêm cờ Debug `lvhvFakeDenied` so với bản đã duyệt; các kiểm thử O-9 (ẩn tên mặc
  định, lần hai không đổi gì, bật tên → thay) vẫn đạt.
- **R14 trên mô phỏng:** ca B dùng bản cũ dựng từ lõi trước core2 (`89dedb3`) nên chuyển đổi 1 → 2 là thật (`migrated`); ca C
  `read_only`, dữ liệu còn nguyên. Ảnh `upgrade-*` khớp nhật ký.
- **Biểu mẫu:** gợi ý cách gọi theo người sống/mất từ lõi (người đang sống không có Cụ, Kỵ — `ios-form-living-am-light`); giới
  tính một hàng nút; lưu hỏng giữ dữ liệu đang nhập và dọn ảnh/tệp vừa ghi.
- Nút chuông có `accessibilityLabel("Cài đặt nhắc")` (góp ý A-2 cũ).

## 3. Phát hiện mới

Không có mục **chặn**.

### DV23-01 — nên sửa — Nút "Hủy" của tấm khôi phục không bị khóa khi đang khôi phục; khôi phục vẫn chạy tiếp sau khi bấm Hủy

- Vị trí: `BackupView.swift:441` (`ToolbarItem(placement: .cancellationAction) { Button("Hủy") { dismiss() } }` — không có
  `.disabled(busy)`), so với `:444` (nút "Khôi phục" có khóa) và `:454` (chỉ khóa vuốt). Luồng: `run(_:)` (`:389-395`) gọi
  `onApply` → `m.applyImport` (`PeopleCore.swift:597-603`, `Task.detached`).
- Vấn đề: khi "Đang khôi phục…", người dùng bấm "Hủy" (hay Esc trên Mac) thì tấm đóng, `preview`/`verified` về `nil`, nhưng tác vụ
  `applyImport` đã chạy và vẫn ghi xong; sau đó hiện "Đã khôi phục: n người…". Với "Thay toàn bộ", người dùng tưởng đã dừng nhưng
  dữ liệu trên máy đã bị thay. Mô tả PR ghi "hộp khôi phục … khóa nút và vuốt đóng" — đúng với nút chính, chưa đúng với Hủy.
  (Bước đã được xác nhận hai lần nên không mất dữ liệu ngoài ý muốn ban đầu; lỗi là giao diện nói sai trạng thái.)
- Cách sửa: `.disabled(busy)` cho nút Hủy của `RestoreSheet` (như `BackupExportSheet` `:341`). Nên thêm vào kiểm thử giao diện
  một khẳng định Hủy bị khóa khi tấm ở trạng thái đang chạy.

### DV23-02 — nên sửa — Khung chỉ đọc chưa theo câu chuẩn TKN3-X-03 và hiện mã kỹ thuật; nút ghi chỉ mờ đi, một số còn bấm được (TKN3-P-01)

- Vị trí: `PeopleView.swift:94-112` (`readOnlyBanner`), `PersonDetailView.swift:246-248` (Sửa / Xóa sự kiện không kiểm
  `readOnly`), `BackupView.swift:58-87` (khối "Khôi phục từ tệp sao lưu" và "Nhắc sao lưu" vẫn hiện), `PeopleSettingsView`
  (`BackupView.swift:472-497`, công tắc và "Lưu mặc định" ghi bằng `try?`). Ảnh `upgrade/upgrade-readonly-profiles.png`,
  `upgrade-readonly-diag.png`.
- Vấn đề:
  1. 08 §12l (có trên nhánh PR sau khi gộp `main`) chốt **cho mọi nền tảng** câu "**Chưa chuyển được dữ liệu sang bản mới.** Dữ
     liệu cũ vẫn còn nguyên trên máy và đang mở ở chế độ chỉ đọc: xem được, nhưng chưa thêm hay sửa được." + hướng dẫn tạo tệp sao
     lưu + nút "Đi tới Sao lưu", và "Không hiện mã lỗi kỹ thuật". Bản Apple dùng câu khác, nút "Xuất sao lưu", và luôn hiện
     "Chi tiết: Chưa nâng cấp được dữ liệu (ảnh giấy báo tử 54764191-0f68-…: tệp không đúng định dạng đã khai (mime_mismatch))…"
     — mã UUID và tên lỗi tiếng Anh, không thu gọn. Đây chính là điều TKN3-A-01 / TKN3-W-01 đã yêu cầu sửa ở Android và Windows
     (Windows đã thu gọn dưới "Chi tiết kỹ thuật", `DUYET-VONG-SUA-PR25.md`). 08 §12-Apple ghi "câu của lõi hiện bên dưới ở dòng
     'Chi tiết'" nhưng không ghi là sai khác so với §12l.
  2. TKN3-P-01 (08 §12l): chế độ chỉ đọc **ẩn** mọi việc ghi ("+ Thêm", Sửa, Xóa hồ sơ, Cập nhật: đã mất, "+ Thêm sự kiện",
     sửa/xóa sự kiện, khối Khôi phục, Nhắc sao lưu). Apple chỉ làm mờ (`.disabled`) phần lớn nút (ảnh: "+ Thêm" mờ), còn **Sửa / Xóa
     sự kiện không bị khóa**: bấm Xóa → hộp xác nhận → lõi báo `read_only` → hộp "Chưa thực hiện được" với câu của lõi; bấm Sửa →
     mở biểu mẫu, gõ xong mới báo lỗi khi lưu — đúng tình huống TKN3-P-01 muốn tránh. Công tắc ở Cài đặt nhắc và ô "Nhắc sao lưu"
     bật/tắt rồi tự bật lại không lời giải thích.
- Cách sửa: dùng câu chuẩn TKN3-X-03 (thay "tải lại trang" bằng "mở lại ứng dụng" như Windows), nút "Đi tới Sao lưu"; đưa câu
  của lõi vào mục thu gọn "Chi tiết kỹ thuật" (đóng sẵn). Ẩn (không chỉ làm mờ) các nút ghi theo danh sách TKN3-P-01, gồm Sửa/Xóa
  sự kiện, khối Khôi phục, Nhắc sao lưu, các điều khiển ghi của Cài đặt nhắc. Thêm vào `PeopleUpgradeTests.testFailedMigrationIsReadOnly`
  khẳng định không có nút ghi và lời chính không chứa "mime" hay mã UUID. Chụp lại `upgrade-readonly-*`.

### DV23-03 — nên sửa — Tấm khôi phục: lựa chọn Gộp / Thay toàn bộ hiện thành bánh xe cuộn, chữ bị cắt, không to theo cỡ chữ

- Vị trí: `BackupView.swift:415-421` (`Picker(...).pickerStyle(.inline).labelsHidden()` trong `ScrollView`, không trong `Form`).
  Ảnh `ios-restore-preview-light/-dark/-large`, `ios-restore-replace-confirm-dark`.
- Vấn đề: ngoài `Form`, kiểu `.inline` trên iOS vẽ thành bánh xe chọn. Ảnh cho thấy hai dòng bị cắt: "Gộp vào dữ liệu đang có
  (khuyên d…" và "Thay toàn bộ — xóa dữ liệu đang c…"; ở Accessibility L chữ **không to lên** (cùng cỡ như ảnh sáng), lựa chọn
  đang chọn chỉ phân biệt bằng dải nền mờ. Phần bị cắt đúng là phần cảnh báo ("xóa dữ liệu đang có trên máy"). Người 35–70 tuổi
  khó biết mình đang chọn gì ở bước ghi đè dữ liệu. Hộp xác nhận thứ hai "Thay toàn bộ dữ liệu?" có câu đầy đủ và mặc định là
  Gộp, nên không mất dữ liệu ngoài ý muốn — vì vậy giữ ở mức nên sửa. Cùng loại với TKN-I-05.
- Cách sửa: hai hàng chọn dạng nút/radio tự viết (như `ChoiceChip` đã dùng trong mô-đun) với chữ xuống dòng đầy đủ, có
  `isSelected`; hoặc đặt `Picker` trong `Form`. Chụp lại ba ảnh `ios-restore-preview-*`.

### DV23-04 — nên sửa — Trình xem PDF (Quick Look) nằm ngoài lớp che ảnh đa nhiệm (BM2-04)

- Vị trí: `PersonFormView.swift:177` (`.quickLookPreview($preview)`), `PeopleUI.swift:676-695` (`PrivacyCover` là `overlay` của
  cây SwiftUI). Ảnh `ios-att-pdf-open-light` cho thấy trình xem phủ toàn màn hình.
- Vấn đề (suy từ cách hệ điều hành trình bày, chưa thử trên thiết bị): `.quickLookPreview` trình bày `QLPreviewController` như một
  màn hình phủ riêng của hệ thống, nằm trên cửa sổ chính; lớp che của `RootView` và của các tấm không phủ lên nó. Mở PDF giấy chứng
  tử rồi vuốt về màn hình chính thì ảnh của bộ chuyển ứng dụng (và `Library/Caches/Snapshots`) có thể chứa trang giấy tờ —
  đúng điều BM2-04 muốn chặn. Ảnh đính kèm (trang xem trong ứng dụng) đã có lớp che riêng (`PersonFormView.swift:223`).
- Cách sửa (chọn một): (a) đóng trình xem khi `scenePhase` rời `.active` (`preview = nil` — tệp tạm cũng được xóa nhờ `:178`);
  (b) tự trình bày `QLPreviewController` trong một `.sheet` qua `UIViewControllerRepresentable` rồi gắn `.privacyCover()` như các
  tấm khác. Góp ý kèm: đặt `FileProtectionType.complete` cho `tmp/att/` khi ghi bản chép tạm, như `photos/`, `attachments/` (BM2-20).

### DV23-05 — góp ý — iPad: trang hồ sơ và danh sách Hồ sơ vẫn một cột ở 820 pt, trái lời A-14

- Ảnh `ipad-person-deceased-light`, `ipad-profiles-light` (iPad dọc, 820 pt): một cột rộng; chỉ Sắp tới hai cột
  (`ipad-upcoming-light`). 08 §12-Apple A-14 ghi "iPad / Mac (≥ 740 pt): … hồ sơ một người hai cột, Hồ sơ hai khung cạnh nhau".
  `ViewThatFits` (`PersonDetailView.swift:49-64`, `PeopleView.swift:412-422`) chọn bản một cột vì bản hai cột cần rộng hơn chỗ có
  (khung `frame(width: 340)` + nội dung cột trái).
- Đề xuất: hạ yêu cầu bề rộng của bản hai cột (ví dụ cột phải 300 pt, cột trái co giãn) hoặc sửa A-14 cho đúng ("hai cột khi
  ≥ 900 pt: iPad ngang, Mac").

### DV23-06 — góp ý — TKN3-X-02: ô ngày sinh và ngày sự kiện vẫn ghi "Dương lịch / Âm lịch"

- `PeopleUI.swift:265`: chỉ `purpose == .death` dùng "Nhập theo dương lịch / Nhập theo âm lịch". 08 §12l TKN3-X-02 chốt một cặp
  chữ cho mọi ô ngày (cả ngày sinh, ngày sự kiện). Ảnh `ios-form-living-am-light-2`, `ios-form-edit-light-3`.
- Đề xuất: dùng cùng cặp chữ cho mọi `purpose`.

### DV23-07 — góp ý — CI: quyền ghi cho toàn bộ workflow; một lần chạy hỏng vì máy mô phỏng

- `.github/workflows/apple.yml:13-15`: `permissions: contents: write` ở cấp workflow, áp cho mọi job và mọi sự kiện (cả `push`
  lên `main` và thẻ `v*` sau khi gộp), trong khi chỉ bước "Ghi ảnh chụp vào nhánh PR" cần quyền ghi. Mã của PR (kịch bản dựng,
  kiểm thử) chạy với mã thông báo có quyền ghi còn trong `.git/config` (checkout giữ thông tin xác thực mặc định). PR từ kho
  tách nhánh bị GitHub hạ quyền nên rủi ro chỉ với nhánh cùng kho — cùng nhận xét như MA3N-24-08.
- Đề xuất: `contents: read` ở cấp workflow; cấp `write` cho riêng job/bước ghi ảnh (hoặc `persist-credentials: false` khi checkout
  và chỉ đưa mã thông báo cho bước ghi ảnh); sau khi gộp, bước ghi ảnh không chạy trên `main` nên không cần quyền ghi ở đó.
- Run 272 lần 1: `testPeriodFlow` hỏng với `kAXErrorServerNotFound`, rồi trình chạy kiểm thử giao diện không khởi động được
  ("Timed out waiting for AX loaded notification"). Nên thêm `-retry-tests-on-failure -test-iterations 2` (Xcode 13+) hoặc khởi
  động lại máy mô phỏng một lần khi gặp lỗi này, để CI không cần chạy lại bằng tay.

### DV23-08 — góp ý — Chưa có bước CI khẳng định bản Release không chứa tham số kiểm thử; R14 kiểm bằng bản Debug

- Mã đúng (§2), nhưng không có phép kiểm tự động như Windows ("LichViet.exe phát hành không có chế độ kiểm thử"). R14 (job
  `upgrade`) cài bản mới dựng Debug để có màn hình kiểm tra sau nâng cấp, nên chưa thử chính bản Release chuyển đổi kho (cùng loại
  với DV25-02, rủi ro thấp vì cùng thư viện lõi).
- Đề xuất: sau bước dựng Release, `strings` trên tệp chạy iOS/macOS và báo lỗi nếu thấy `lvhvSeed`, `lvhvScreen`, `lvhvStoreDir`,
  `lvhvFakeDenied` (đối chứng: các chuỗi này phải có trong bản Debug).

### DV23-09 — góp ý — Câu lỗi khi tệp sao lưu rỗng hoặc quá lớn

- `BackupView.swift:168-171`: tệp 0 byte hoặc lớn hơn 128 MiB + 88 byte → "Sai mật khẩu hoặc tệp bị hỏng" ngay, trước khi thử mật
  khẩu. Người dùng có thể đi đoán lại mật khẩu. Đề xuất: "Tệp này không phải tệp sao lưu của ứng dụng hoặc quá lớn" (hoặc dùng câu
  của `backupHeaderCheck` của lõi cho cả hai trường hợp).

### DV23-10 — góp ý — Thứ tự gộp với PR #47 (YC-2026-10)

- `main` hiện có 08 §3.9 / §16 (bàn thờ không mâm: nến · bát hương · nến). PR #23 vẫn vẽ mâm quả (`ios-memorial-*-2`); PR #47
  (`v0.2-yc10-apple`, dựng trên nhánh #23) bỏ mâm ở bước 1. Nhánh #23 sau `181eedc` chưa có các commit mới của `main`
  (#25, #32, #48, #53–#55, #58); GitHub báo gộp được, không xung đột.
- Đề xuất: gộp #47 bước 1 ngay sau #23 (hoặc ghi trong mô tả #23 rằng màn hình tưởng niệm trên Apple chưa theo §16 cho tới #47),
  để `main` không có một bản Apple trái §3.9 lâu.

## 4. Kết luận

| Nhóm | Mục cũ đã sửa | Chặn mới | Nên sửa mới | Góp ý mới |
|---|---|---|---|---|
| Mã (MA3N-23-01…04) | 4/4 (MA3N-23-02 còn phần nút Hủy) | 0 | 1 (DV23-01) | 4 (DV23-07, -08, -09, -10) |
| Thiết kế (TKN-I-01…13, mục chung) | 13/13 phần chính (TKN-I-11 một phần) | 0 | 2 (DV23-02, -03) | 2 (DV23-05, -06) |
| Bảo mật (BM2-04/10/14/17/20, §5.3, BM3-03) | 7/7 | 0 | 1 (DV23-04) | — |
| **Tổng** | | **0** | **4** | **6** |

**PR #23: DUYỆT CÓ ĐIỀU KIỆN — gộp được về mã, thiết kế và bảo mật.** Hai mục chặn cũ (TKN-I-01, TKN-I-02) đã sửa và có ảnh;
mọi mục nên sửa cũ đã sửa; BM3-03 đã sửa đúng đề xuất (ảnh xem trong ứng dụng, PDF hỏi lại với "Hủy" mặc định trước Quick Look),
có kiểm thử giao diện. CI `apple` xanh trên commit mã cuối `181eedc`.

Điều kiện:
1. DV23-01, DV23-02, DV23-03, DV23-04 sửa trong đợt này (đều là thay đổi nhỏ ở giao diện), hoặc mở việc có hạn trước phát hành
   v0.2 và ghi lý do vào PR. DV23-02 nên sửa trước khi gộp vì là quyết định chung đã ghi ở 08 §12l cho mọi nền tảng.
2. Gộp #47 bước 1 ngay sau #23 (DV23-10).
3. Không thay cho thử trên máy thật (T11): R5 (thông báo đúng giờ, sau khởi động lại, xoay vòng 64), P8, P9, thông báo trên macOS,
   ảnh HEIC thật, Quick Look trên Mac, ảnh bộ chuyển ứng dụng khi đang mở PDF (DV23-04), giao diện tối và cỡ chữ AX3–AX5 trên
   iPhone thật.
