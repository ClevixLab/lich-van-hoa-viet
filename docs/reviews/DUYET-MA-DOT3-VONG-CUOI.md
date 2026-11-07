# Duyệt mã đợt 3 v0.2 — vòng cuối: PR #30 (PWA vòng 2), PR #24 (Android), PR #25 (Windows); PR #23 (Apple) chưa duyệt

Trạng thái: **XONG** — 2026-11-07. Kết luận ở §6.

Bên duyệt mã: cùng vai đã viết `docs/reviews/DUYET-MA-DOT3.md` (PR #19) và `DUYET-MA-DOT3-NATIVE.md` (PR #27, chưa gộp);
không viết mã của các PR này và không sửa mã đang duyệt. Cùng cách làm: diff của từng PR so với `main` hiện tại, đọc mã,
đọc nhật ký CI, kiểm lại từng phát hiện cũ.

| PR | Nhánh | Đầu nhánh được duyệt | So với |
|---|---|---|---|
| #30 | `v0.2-w3-pwa2` | `9e590d8` | `git diff origin/main...9e590d8` |
| #24 | `v0.2-w3-android` | `ab83b9b` | `git diff origin/main...ab83b9b` |
| #25 | `v0.2-w3-windows` | `5f4dfcc` (đầu nhánh `8b86892` chỉ thêm ảnh chụp, `[skip ci]`) | `git diff origin/main...5f4dfcc` |
| #23 | `v0.2-w3-apple` | — **không duyệt** (§5) | — |

`main` = `8835cd7` (đã gộp #28 core2, #35 B6, #36 công cụ, #37 B7, #38 core3). Cả ba nhánh được duyệt đều đã gộp
`8835cd7`.

Căn cứ: docs/v0.2/01 (§3.2a, §3.8a, §7), 02, 03, 04 (S5–S12, Bổ sung B2–B7, R14), 07, 08; `ffi/src/v02.rs`;
`store/src/{lib,migrate,media,names}.rs`; biên bản rà soát bảo mật `docs/reviews/RA-SOAT-BAO-MAT-V02.md` (PR #34, mục
BM2-xx).

Mức lỗi: **chặn** / **nên sửa** / **góp ý**. Mã lỗi mới: `MA3C-<PR>-<số>`.

## 0. Đã chạy / đã đọc

| Việc | Kết quả |
|---|---|
| CI trên `9e590d8` (#30) | Mọi việc `success` (ci: L7 + PWA build + smoke + **nâng cấp R14** `test:upgrade`; android, apple, windows build) |
| CI trên `ab83b9b` (#24) | Mọi việc `success`, gồm job `ui` (Robolectric), `upgrade` (R14 trên trình giả lập), `screenshots` |
| CI trên `5f4dfcc` (#25), run 239 | Mọi việc `success`. Nhật ký: `cargo test` 30/30; `--kiem-thong-bao`: "**ĐẶT ĐƯỢC, XOAY VÒNG ỔN ĐỊNH**… `lvhv:kiem-thu:khong-dong-phu`: đọc lại đúng ("Kiểm thử")"; lần 2 không gỡ/thêm gì; R14 (a), (c), ca lỗi chỉ đọc: ĐẠT |
| CI của #23 trên đầu nhánh `85850c4` | **Không có run nào** (`action_required`); run `apple` gần nhất (229, `e3e1ddf`) còn `in_progress` — xem §5 |
| `cargo test` của `apps/windows` trên Linux | **Không chạy hết**: đĩa của môi trường duyệt còn < 2 GB (dựng Skia cần ~3 GB), đã dừng để không làm đầy đĩa dùng chung; dựa vào nhật ký CI Windows ở trên (chạy trên Windows thật) |
| Android SDK / Xcode | Không có; đọc mã + nhật ký CI |

## 1. Kiểm lại các phát hiện cũ

### 1.1 DUYET-MA-DOT3 (PR #14, #15) — trên `main` và #30

| Mã | Kết quả |
|---|---|
| MA3-14-01 … -09 | **Đã sửa** ở lõi (đã kiểm ở vòng trước, `ffi/src/v02.rs` trên `main`): `next` không giới hạn cửa sổ, O-9 đọc từ kho, `occ` rút gọn, kiểm miền năm/múi giờ, `Zeroizing`, sentinel 09:00, lược đồ đầu ra, khẳng định chặt, không nuốt lỗi SQLite |
| MA3-15-01 | Đã sửa: `period.ts` dùng `eventsInPeriod` của `tsport` |
| MA3-15-02 | Đã sửa: `backup.ts:143` kiểm `f.size` trước khi đọc |
| MA3-15-03 | Đã sửa: `ui.ts:66` chặn Esc khi `busy`; #30 thêm kiểm thử tất định bằng cổng chỉ có ở bản dựng kiểm thử (§4.1) |
| MA3-15-04 | Đã sửa: `people-flows.mjs:320` so tên hồ sơ mở ra với mục đã chạm |
| MA3-15-05 | Đã sửa: `views.ts:74-78` dùng `M.dueToday` của lõi |
| MA3-15-06 | Đã sửa: ngày theo giờ máy (`localDmy`) |
| MA3-15-07 | Đã sửa; #30 chặt thêm (BM2-13, §2) |
| MA3-15-08 | Đã sửa (xóa ô mật khẩu; `metrics.json` không còn đổi theo mỗi PR) |

### 1.2 DUYET-MA-DOT3-NATIVE (PR #27) — Android #24, Windows #25

| Mã | Kết quả | Bằng chứng |
|---|---|---|
| **MA3N-25-01 (chặn)** | **Đã sửa** | `toasts.rs:4-11, 157-181`: `id` và `content` của lõi nằm nguyên văn trong `launch`; `body` rỗng → một `<text>`; đọc chỉ từ `launch` (chịu được `<text/>`, nháy đơn); thông báo không đọc được bị gỡ (`:494-495`). Bản giả `FakeToasts` có chế độ `normalize_empty` viết lại `<text/>`. Trên Windows thật (CI run 239), thông báo không dòng phụ đọc lại đúng và lần đặt lịch thứ hai không gỡ/thêm gì |
| MA3N-25-02 | Đã sửa | `toasts.rs:249-252` đọc lỗi → `ReadFailed`, không áp kế hoạch |
| MA3N-25-03 | Đã sửa | `toasts.rs:240-245` `Disabled` → `remove_all` |
| MA3N-25-04 | Đã sửa | `SyncResult::Scheduled { confirmed }`; `people.rs:974-981` hai câu khác nhau |
| MA3N-25-05 | Đã sửa | `platform.rs:319-356` dựng lần chuyển giờ từ `GetTimeZoneInformationForYear` |
| MA3N-25-06 | Đã sửa | `people.rs:3032-3058` `bg()` (luồng phụ + `invoke_from_event_loop`) cho sao lưu, mở tệp, khôi phục, ảnh, tệp đính kèm |
| MA3N-25-07 | Đã sửa | `people.rs:3062-3069` `metadata` trước khi đọc; `attach.rs:58-75` |
| MA3N-25-08 | Đã sửa | trait `Toasts` + `FakeToasts`, kiểm thử trong `toasts.rs`; `ui_tests_people.rs` |
| MA3N-25-09 | Phần lớn đã sửa (`restore_pw` xóa khi hủy `:3606-3611`, lỗi cài đặt báo bằng `toast`, xem trước .ics lấy từ `IcsSummary`); **còn**: gỡ cài đặt không báo/không xóa dữ liệu (= BM2-22, §3.2) |
| MA3N-24-01 | Đã sửa | `BackupScreen.kt` `doImport` chạy `Dispatchers.IO`, khóa hộp khi `importing` |
| MA3N-24-02 | Đã sửa | `dropVerified()` gọi `destroy()`; `DisposableEffect` thả khi rời màn hình; mật khẩu xóa ở mọi nhánh |
| MA3N-24-03 | Đã sửa | `readLimited` kiểm 72 byte đầu bằng `backup_header_check`, `readBackupBytes` cấp đúng một mảng |
| MA3N-24-04 | Đã sửa | `Reminders.kt` báo thức thử lại (`RETRY_S`) + `last_error_at`; `ReminderReceiverTest.failingStepSchedulesRetryAndRecordsError` |
| MA3N-24-05 | Đã sửa | `AlarmChainTest.clockMovedBackUsesNowNotNull` |
| MA3N-24-06 | Đã sửa | `Reminders.kt:233` `VISIBILITY_PRIVATE` + `setPublicVersion` khi hiện tên |
| MA3N-24-07 | Đã sửa | `ReminderReceiverTest` (mọi action → đúng một báo thức `RTC_WAKEUP` ở lời nhắc kế tiếp, so bằng giá trị), `AlarmChainTest.b4MarkDeceasedRemovesBirthdayFromChain`, `PhotosTest` (S9, bom giải nén) |
| MA3N-24-08 | Đã sửa | dùng `delete_person_and_orphan_events`, `lunar_leap_month`, `due_today_json`, `ics_summary`, `backup_password_error` của lõi |
| MA3N-23-01 (lõi) | Đã sửa ở lõi | `store/src/lib.rs:1137-1186`: `export_backup` không ghi `last_backup_at`; `mark_backup_saved` riêng. Vị trí gọi ở từng nền tảng: §4.3 |

### 1.3 Rà soát bảo mật (BM2) — các mục áp cho ba PR này

| Mã | PR | Kết quả |
|---|---|---|
| BM2-01, -02, -03, -06, -08 | lõi (#35, #38) | Đã sửa trên `main` (`media::check_attachment` trong `add_attachment`, chuyển đổi `certificate_photo_ids`, dọn bản sao sau 7 ngày); #30 không còn `ATTACHMENTS_UNSUPPORTED` |
| BM2-04 | #24 | Đã sửa: `Secure.kt` `FLAG_SECURE` có đếm, ở biểu mẫu báo tử, biểu mẫu người, trang Sao lưu, trang hồ sơ có báo tử |
| BM2-05 | #30 | Đã sửa: `notice.ts` hộp "Tải tệp xuống máy?" (tiêu điểm ở "Không tải"), tên chung `TepDinhKem-n.pdf/.jpg` |
| BM2-07 | #30 | Đã sửa: `legacy` dọn sau 7 ngày; kiểm ở `upgrade.mjs` (ngày 6 còn, ngày 8 hết, phần còn lại giữ từng byte) |
| BM2-09 | #25 | Đã sửa: `apps/windows/deny.toml` + bước riêng trong `ci.yml`. Gradle của Android vẫn chưa có quét lỗ hổng (ngoài phạm vi ba PR) |
| BM2-10 | #24, #25, #30 | Đã sửa: `Photos.bitmap` (đọc kích thước, `inSampleSize`), `photo::decode_bounded` (giới hạn + `limits`), `drawPhoto` (không giải mã ảnh không rõ kích thước / > 4096 px / > 16 MP) |
| BM2-11 | lõi | **Còn mở** (`secure_delete` chưa bật) — ngoài ba PR này |
| BM2-12 | #30 | Đã sửa: hộp "Đính kèm tệp PDF?" trước khi đọc tệp; hộp xuất sao lưu ghi số PDF |
| BM2-13 | #30 | Đã sửa: đọc 2 MB đầu; không nhận ra định dạng → từ chối |
| BM2-15 | #25 | **Còn** — §3.2 MA3C-25-03 |
| BM2-16 | #24 | Đã sửa: `cleanShareCache` khi quay lại |
| BM2-17 | #24, #25 | Đã sửa (như MA3N-24-02, MA3N-25-09) |
| BM2-18 | #24 | Đã sửa: `SelfCheckActivity` chỉ ở `src/debug`; CI khẳng định vắng trong manifest release (`android.yml:137-138`) |
| BM2-19 | #24 | Đã sửa: mã mở dùng một lần (`OpenTokenTest`) |
| BM2-22 | #25 | **Còn** — góp ý, §3.2 MA3C-25-04 |
| BM2-23 | #24, #25 | Đã sửa: `backup_password_error` của lõi |
| BM2-24 | CI | **Còn**: action ghim theo nhãn (`@v4`) — góp ý, ngoài phạm vi |

## 2. PR #30 — PWA vòng 2

### 2.1 Đúng đặc tả, đã kiểm và không có lỗi

- **Tệp đính kèm / PDF:** PDF ≤ 10 MB, kiểm `%PDF-` trước khi lưu, hỏi xác nhận trước khi đính kèm (BM2-12); PDF không
  bao giờ được giải mã hay hiển thị trong trang; tải xuống qua hộp xác nhận (S10, BM2-05) với tên chung; không có tệp
  tạm (trình duyệt tải thẳng từ `blob:` dùng cho `<a download>`, thu hồi sau 10 s — CSP `img-src 'self'` không đổi).
- **Giới tính, viết hoa:** dùng `displayCapitalize` / `capitalizeFirst` / gợi ý theo người sống/mất của lõi; trang hồ sơ
  hiện "Giới tính" (Nam / Nữ / Không ghi); giá trị hiển thị đã cắt khoảng trắng như lõi #38.
- **Chế độ chỉ đọc (B5 mục 3):** `openStoreRaw` mở lại kho cũ khi nâng cấp quay lui; mọi phương thức ghi của `Store` gọi
  `writable()`; `attachments()` trả rỗng cho kho phiên bản 1; khung `#ro-banner` hướng dẫn xuất sao lưu.
- **R14 (`test/upgrade.mjs`):** dựng **bản cũ thật** từ git (`d9650c7` = 0.1.4, `c4d3128` = 0.2-dev), ghi dữ liệu bằng mã
  của bản cũ, nâng cấp qua **đường cập nhật thật** của service worker ("Để sau" → "Cập nhật ngay"). Ngoài phép so với
  `migrate1to2` của lõi (vòng tròn: kỳ vọng do chính hàm chuyển đổi sinh), kiểm thử có các khẳng định **độc lập**: sha256
  byte ảnh giấy báo tử trước = sau, ảnh chân dung giữ nguyên, sự kiện / loại sự kiện / cài đặt giữ từng byte, `legacy`
  giữ bản gốc, tùy chọn giao diện giữ nguyên, quy tắc nhắc của cả 3 hồ sơ còn. Đủ để kết luận **không mất dữ liệu** cho
  (a) và (b).
- **Móc kiểm thử:** `__LVHV_TEST_HOOKS__` thay lúc dựng; `build.mjs` từ chối `LVHV_TEST_HOOKS=1` khi dựng vào `dist/`;
  `smoke.mjs:463-470` khẳng định `dist/` không chứa tên móc nào và bản dựng kiểm thử thì có; `upgrade.mjs` ca (b) khẳng
  định bản dựng chính bỏ qua cờ `__lvhvFailMigration` mà trang đã đặt. Móc **thật sự vắng** trong bản phát hành.

### 2.2 Các lỗi

#### MA3C-30-01 — **nên sửa** — chế độ chỉ đọc: xuất sao lưu thành công nhưng hộp thoại báo lỗi

- Vị trí: `pwa/src/people/backup.ts:118-123` (`download(...)` rồi `await s.markBackupSaved(nowTs())` không điều kiện);
  `pwa/src/store/db.ts:592-594` (`markBackupSaved` gọi `this.writable()` → ném `StoreError("read_only")`);
  `pwa/src/people/ui.ts:75, 79` (lỗi của `onOk` hiện trong hộp, hộp không đóng).
- Vấn đề: khung chỉ đọc bảo người dùng "Tạo tệp sao lưu…". Khi làm đúng như vậy, `exportBackup` chạy được (không ghi kho),
  tệp đã được tải xuống, rồi `markBackupSaved` ném lỗi. Hộp vẫn mở với câu "Dữ liệu đang ở chế độ chỉ đọc vì nâng cấp chưa
  xong — … hãy xuất sao lưu", ô mật khẩu đã bị xóa, không có thông báo "Đã tạo tệp sao lưu".
- Kịch bản hỏng: đúng tình huống B5 mục 3 (nâng cấp lỗi) — người dùng đang lo mất dữ liệu, xuất sao lưu, thấy báo lỗi,
  tưởng chưa xuất được, xuất lại nhiều lần hoặc bỏ cuộc và xóa dữ liệu trình duyệt để "sửa", trong khi tệp đã nằm trong
  Tải về. Android (`BackupScreen.kt:134`, `runCatching`) và Windows (`people.rs:2931-2934`, kiểm `read_only`) đã xử lý
  đúng; riêng PWA chưa. Kiểm thử ca (c) không thử xuất sao lưu nên không phát hiện.
- Cách sửa: `if (s.migrationStatus().state !== "read_only") await s.markBackupSaved(nowTs());` (hoặc bắt riêng
  `StoreError` mã `read_only`). Thêm vào `upgrade.mjs` ca (c): bấm "Tạo tệp sao lưu…" trong chế độ chỉ đọc → có tệp tải
  về, có thông báo thành công, rồi mở tệp đó bằng `openBackup` + `applyImport` vào một kho trống của bản mới → đủ 3 hồ
  sơ và 2 ảnh giấy báo tử (đã chuyển thành tệp đính kèm).

#### MA3C-30-02 — **nên sửa** — R14 ca (c) kiểm ghi bị chặn trên một `Store` tự dựng, không phải kho của ứng dụng

- Vị trí: `pwa/test/upgrade.mjs:293-306` (`new S.Store(db, { state: "read_only", ... })` rồi `putSettings`).
- Vấn đề: khẳng định "ghi bị chặn" đạt cả khi kho mà **ứng dụng** đang dùng không ở chế độ chỉ đọc, vì kiểm thử tự tạo
  một `Store` mang sẵn trạng thái `read_only`. Khung `#ro-banner` chứng minh `migrationStatus()` của ứng dụng là
  `read_only`, nhưng không chứng minh các nút ghi của giao diện thật sự bị lõi từ chối.
- Cách sửa: thử ghi qua giao diện (ví dụ mở "Sửa" một hồ sơ, bấm Lưu → thấy lỗi chỉ đọc, `stateHash` không đổi), hoặc qua
  `store()` của mô-đun Người thân trong bản dựng kiểm thử.

#### MA3C-30-03 — **góp ý** — "đã sao lưu" được ghi ngay khi giao tệp cho trình duyệt

- Vị trí: `backup.ts:120-122`.
- Vấn đề: với trình duyệt bật "Hỏi nơi lưu", người dùng có thể hủy hộp lưu của trình duyệt; trang không biết, vẫn ghi
  `last_backup_at` và tắt nhắc sao lưu (đúng chủ đề MA3N-23-01). Đây là giới hạn của nền web, không có tín hiệu "đã lưu"
  từ `<a download>`.
- Cách sửa: khi có `window.showSaveFilePicker` (Chromium), dùng nó và chỉ ghi nhận sau `close()` thành công; còn lại ghi
  giới hạn này vào 08 §5.6 / §12.

#### MA3C-30-04 — **góp ý** — ghi `meta` của giao diện đi vòng qua chế độ chỉ đọc

- Vị trí: `views.ts:57-58` (`ui_persist_notice_hidden_at`), `backup.ts:178-179` (`ui_restored_from`),
  `store/db.ts:746-748` (`requestPersistentStorage` ghi `meta.persist`) — đều mở giao dịch `readwrite` thẳng trên
  `s.db`, không qua `writable()`.
- Vấn đề: chỉ là cờ giao diện, không phải dữ liệu người dùng, nên không làm mất gì; nhưng "chỉ đọc" không còn đúng nghĩa
  đen và các đường này không có kiểm thử.
- Cách sửa: thêm `Store.putUiMeta(key, value)` có kiểm `writable()` (bỏ qua lặng lẽ khi chỉ đọc).

## 3. PR #25 — Windows

### 3.1 Đúng đặc tả, đã kiểm và không có lỗi

- MA3N-25-01 … -08 đã sửa (§1.2), có kiểm thử trên bản giả và tự kiểm trên Windows thật.
- **Tệp đính kèm / PDF (`attach.rs`):** nhận PDF theo 5 byte đầu (không theo đuôi tệp); kiểm 10 MiB từ siêu dữ liệu tệp
  **trước khi** đọc, đọc tối đa 10 MiB + 1; kiểm `%PDF-` và `%%EOF`; ảnh đi qua bộ mã hóa lại (≤ 2048 px, ≤ 2 MB, bỏ siêu
  dữ liệu). Mở PDF: bản tạm `…\LichVanHoaViet\tmp\<ngẫu nhiên>\TepDinhKem.pdf` (không ở `%TEMP%` dùng chung, không mang
  tên người), kiểm lại byte trước khi ghi; dọn `tmp\` khi mở và khi đóng ứng dụng (`main.rs:778-780`). Có kiểm thử cho cả
  ba ý.
- **`mark_backup_saved`:** gọi trong luồng phụ **chỉ sau** `std::fs::write` thành công, bỏ qua khi `read_only`
  (`people.rs:2924-2935`). Đúng 03 §2.4.
- **Chỉ đọc:** `people::read_only_message` hiện câu của lõi; R14 ca lỗi (`upgrade.rs:209-245`) so **nội dung từng dòng**
  của mọi bảng trước/sau (`db_content`) và với bản sao `.bak`, thử ghi (bị từ chối), xuất sao lưu và **mở lại** tệp đó.
- **R14:** cài 0.1.4 thật từ GitHub Releases rồi cài MSI mới đè; bộ mẫu đóng băng 0.2-dev chuyển 1 → 2 khớp tệp kỳ vọng
  chung; gỡ cài đặt giữ nguyên từng tệp dữ liệu (so băm). Chứng minh được không mất dữ liệu cho chuyển đổi 1 → 2 và cho
  việc cài đè / gỡ cài đặt (xem giới hạn ở MA3C-25-02).
- `cargo deny` có cho workspace `apps/windows` (BM2-09), ngoại lệ đều ghi lý do và ngày xem lại.

### 3.2 Các lỗi

#### MA3C-25-01 — **nên sửa** — chế độ kiểm thử CI có trong `LichViet.exe` phát hành, một chế độ ghi vào thư mục dữ liệu thật

- Vị trí: `apps/windows/src/main.rs:697-747` (`--kiem-thong-bao`, `--tao-du-lieu-mau`, `--kiem-nang-cap`,
  `--kiem-mau-dong-bang`, `--kiem-chi-doc`); `upgrade.rs:93-117` (`create`).
- Vấn đề: câu hỏi "móc kiểm thử có thật sự vắng trong bản phát hành không" — với Windows: **không**. Các chế độ này nằm
  trong bản `.exe` được đóng gói vào `.msi`. Đáng kể nhất:
  - `--tao-du-lieu-mau <tệp>` tạo **dữ liệu Người thân mẫu** (5 hồ sơ, ảnh, `lock_screen_names = true`) trong
    `%LOCALAPPDATA%\ClevixLab\LichVanHoaViet` nếu chưa có kho, và ghi tệp ảnh chụp ra đường dẫn bất kỳ.
  - `--kiem-thong-bao` gọi `remove_all()` trên thông báo **thật** của người dùng (đặt lại ở lần mở sau).
  Giao thức `lichviet:` không gọi được các chế độ này (`main.rs` chỉ xét `args[0]`, mà `args[0]` luôn bắt đầu bằng
  `lichviet:`), nên cần chạy lệnh trên máy — rủi ro thấp; Android (`src/debug` + CI khẳng định vắng) và PWA (`define` +
  smoke khẳng định vắng) đều đã tách hẳn.
- Kịch bản hỏng: một lối tắt hay tập lệnh do người khác đưa (hoặc người dùng chép nhầm từ tài liệu CI) chạy
  `LichViet.exe --tao-du-lieu-mau x.json` trên máy mới cài → mục Người thân hiện 5 người lạ, và tùy chọn "hiện tên trên
  thông báo" đã bật sẵn.
- Cách sửa: tách các chế độ này ra một tệp chạy riêng `lvhv-ci-check.exe` (cùng crate, `[[bin]]` khác) **không** đưa vào
  `.msi`; CI chạy nó trên thư mục dữ liệu của bản đã cài. Nếu buộc phải giữ trong `LichViet.exe`, ít nhất cho
  `--tao-du-lieu-mau` đòi thêm biến môi trường `LVHV_CI=1` và bỏ `set_lock_screen_names(true)`. Thêm bước CI khẳng định
  `LichViet.exe` trong `.msi` không chứa các chuỗi `--tao-du-lieu-mau`, `--kiem-chi-doc`.

#### MA3C-25-02 — **góp ý** — R14 (a) Windows chạy với cùng số phiên bản cũ và mới

- Vị trí: nhật ký CI run 239: "R14 (a) Windows: ĐẠT (0.1.4 → 0.1.4 bản dựng 239…)"; `windows.yml:194-288`.
- Vấn đề: tệp `VERSION` còn `0.1.4`, nên phép "nâng cấp lớn" của MSI là cài **cùng phiên bản** đè lên (đường ít gặp, cần
  cho phép nâng cấp cùng phiên bản), không phải đường thật 0.1.4 → 0.2.0. Dữ liệu Người thân của ca (a) do chính bản mới
  tạo rồi bản mới đọc lại (không có chuyển đổi); phần chuyển đổi được ca (c) chứng minh. Kết luận "không mất dữ liệu khi
  cài đè/gỡ" vẫn đúng, nhưng chưa thử với phiên bản lớn hơn.
- Cách sửa: trong bước R14, dựng MSI với phiên bản tạm lớn hơn (ví dụ `0.2.0` qua biến dựng) và chạy lại ca (a); hoặc
  ghi rõ trong 04-BUILD-NATIVE rằng R14 (a) phải chạy lại khi `VERSION` đổi sang 0.2.x.

#### MA3C-25-03 — **góp ý** — URI `lichviet:` mang nguyên nội dung thông báo (BM2-15, còn)

- Vị trí: `toasts.rs:100-106` (`c=pct(content)`).
- Vấn đề: khi bật "hiện tên", `content` có tên người và loại ngày; nó nằm trong thuộc tính `launch` và trên dòng lệnh của
  `LichViet.exe` khi người dùng bấm thông báo (thấy được qua danh sách tiến trình, nhật ký của phần mềm giám sát). Việc
  đối chiếu chỉ cần biết nội dung **có đổi** hay không.
- Cách sửa: ghi `c=<sha256(content) 16 chữ hex>` thay cho nội dung; đối chiếu bằng cách băm `content` của lõi (lõi so
  chuỗi, nên ứng dụng đổi `content` của `pending` thành băm trước khi gửi, và băm `content` của kế hoạch trước khi so —
  hoặc thêm vào lõi một tùy chọn so theo băm).

#### MA3C-25-04 — **góp ý** — dữ liệu còn lại sau gỡ cài đặt; thư mục dữ liệu lùi về `%TEMP%` (BM2-22, còn)

- Vị trí: `platform.rs:16-27`; `wix/main.wxs`.
- Cách sửa: như BM2-22 — một dòng ở trang Sao lưu "Gỡ ứng dụng không xóa dữ liệu Người thân; muốn xóa, dùng nút…" + nút
  xóa (gỡ cả thông báo đang chờ); thiếu `LOCALAPPDATA` thì báo lỗi kho, không lùi về `%TEMP%`.

#### MA3C-25-05 — **góp ý** — đường luồng phụ không có kiểm thử; mở tệp sao lưu bấm được hai lần

- `people.rs:3032-3042`: dưới `cfg(test)`, `bg()` chạy tuần tự, nên `invoke_from_event_loop` và `PS_TLS` không bao giờ
  chạy trong kiểm thử. Nên có một kiểm thử dùng vòng sự kiện của backend kiểm thử Slint (hoặc một hàm `bg_sync` tách riêng
  để phần còn lại vẫn là mã thật).
- `open_backup_file` (`:3204-3215`): trong lúc luồng phụ đang chạy Argon2id, nút "Mở tệp" không bị khóa; bấm lần hai chạy
  thêm một luồng (thêm 64 MiB). Khóa nút khi `restore-error` đang là "Đang mở tệp…".

## 4. PR #24 — Android

### 4.1 Đúng đặc tả, đã kiểm và không có lỗi

- MA3N-24-01 … -08 đã sửa (§1.2), có kiểm thử so bằng giá trị chính xác.
- **Tệp đính kèm / PDF (`Attachments.kt`):** chọn PDF qua SAF (`OpenDocument("application/pdf")`, không quyền bộ nhớ),
  kiểm 10 MB theo `OpenableColumns.SIZE` trước khi đọc và trong khi đọc, kiểm `%PDF-`; ảnh đính kèm qua Photo Picker và bộ
  mã hóa lại. PDF không bao giờ được hiển thị trong ứng dụng; "Mở bằng ứng dụng khác…" có hộp xác nhận, bản tạm
  `cache/att/TepDinhKem.pdf`, quyền đọc chỉ cho đúng Intent, thu hồi quyền và xóa bản tạm ở `onResume`
  (`MainActivity.kt:146-154`). FileProvider chỉ lộ `cache/ics/` và `cache/att/`.
- **Giới tính, viết hoa:** `gioi_tinh` trong biểu mẫu, gợi ý và viết hoa từ lõi (`Yc07Test`).
- **Chỉ đọc:** `PeopleModule.kt:137` hiện câu của lõi; lõi từ chối mọi lệnh ghi.
- **`mark_backup_saved`:** chỉ sau khi `openOutputStream(uri, "wt")` ghi xong (`BackupScreen.kt:122-134`); hủy chọn nơi
  lưu / lỗi ghi → không gọi; chỉ đọc → `runCatching` bỏ qua. `Yc07Test.backupSavedOnlyWhenMarked`.
- **Móc kiểm thử:** `SelfCheckActivity` chỉ ở `src/debug`, CI khẳng định vắng trong manifest release (BM2-18);
  `@VisibleForTesting reset()` không có tác dụng ngoài kiểm thử.
- **R14 (`ci/upgrade-test.sh`):** tùy chọn giao diện qua bản cũ thật; bộ mẫu đóng băng 0.2-dev chuyển 1 → 2 khớp tệp kỳ
  vọng chung; ca lỗi chỉ đọc. `MigrationTest` (JVM) chạy cùng hai ca trên lõi thật.

### 4.2 Các lỗi

#### MA3C-24-01 — **nên sửa** — R14 ca lỗi trên Android chỉ đếm số hồ sơ và chỉ kiểm phần đầu tệp sao lưu

- Vị trí: `apps/android/ci/upgrade-test.sh:117-121` (sau khi quay lui chỉ so `user_version` và `count(*) FROM person`);
  `app/src/debug/.../SelfCheckActivity.kt:247-252` (`verifyReadOnly` chỉ gọi `checkBackupHeader` trên 72 byte đầu).
- Vấn đề: câu hỏi "kiểm thử R14 có thật sự chứng minh không mất dữ liệu không" — với ca lỗi của Android: **chưa**. Một
  bước chuyển đổi hỏng mà sửa nội dung hồ sơ (ví dụ xóa `death.notice`) hay xóa ảnh vẫn qua, vì chỉ đếm 3 hồ sơ. Tệp sao
  lưu xuất từ chế độ chỉ đọc chỉ được kiểm phần đầu, không được giải mã, nên một tệp thiếu ảnh hay thiếu thông tin báo tử
  vẫn "ĐẠT". Windows làm đủ cả hai việc (`upgrade.rs:209-245`: so nội dung từng dòng mọi bảng, mở lại tệp sao lưu).
- Cách sửa: trong `upgrade-test.sh`, so `sqlite3 … .dump` của các bảng `settings photo person event_type event
  event_person` trước và sau (như `db_content` của Windows; WAL không ảnh hưởng nội dung), và so băm các tệp
  `files/photos/*`. Trong `verifyReadOnly`, `PeopleRepo.openBackupFile(bytes, mật khẩu)` rồi khẳng định
  `preview.persons == 3`, `preview.photos == ` số ảnh của bộ mẫu.

#### MA3C-24-02 — **góp ý** — ca "dữ liệu người thân" của R14 là cài đè cùng một bản; R14 chạy trên biến thể debug

- Vị trí: `upgrade-test.sh:67-77` (`seed` bằng `$NEW`, rồi `adb install -r "$NEW"`), `PKG=vn.clevix.lichviet.debug`.
- Vấn đề: ca này chứng minh cài đè không xóa `files/` và báo thức được đặt lại sau `MY_PACKAGE_REPLACED` — có ích — nhưng
  không có chuyển đổi nào. Toàn bộ R14 chạy trên APK debug (không R8, khóa ký debug); bản release (rút gọn mã bằng R8) đi
  qua cùng đường chuyển đổi của lõi nhưng không được thử.
- Cách sửa: ghi rõ phạm vi này trong 08 §13 / 04-BUILD-NATIVE. Chép kho bộ mẫu vào APK release cần `run-as`, mà `run-as`
  chỉ dùng được với bản debuggable, nên để R14 ở biến thể debug là chấp nhận được nếu ghi lý do; phần R8 có thể được che
  bằng một lần mở bản release trên kho do bản debug cùng mã đã chuyển đổi (cài đè release lên debug không được vì khác
  `applicationId`, nên cần một biến thể `releaseTest` cùng gói với debug và bật R8).

#### MA3C-24-03 — **góp ý** — PDF: không kiểm `%%EOF`, không hỏi trước khi đính kèm

- Vị trí: `Attachments.kt:92-97` (`checkPdf` chỉ kiểm `%PDF-`), `:139` (chọn PDF không qua hộp như PWA BM2-12).
- Vấn đề: Windows từ chối PDF cụt (`%%EOF` gần cuối), Android thì lưu. Câu "PDF được lưu nguyên bản" có hiện sẵn dưới
  nút (đủ cho R-PDF-1), nhưng ba nền tảng hỏi khác nhau.
- Cách sửa: thêm kiểm `%%EOF` trong 1 024 byte cuối như `attach::check_pdf`; nên đưa hàm kiểm PDF vào lõi
  (`media::check_attachment` đã có chỗ) để ba nền tảng giống nhau.

## 5. PR #23 — Apple: **không duyệt ở vòng này**

Điều kiện của vòng này là CI xanh trên đầu nhánh mới nhất. Đầu nhánh `85850c4` (commit ảnh chụp của bot) **không có run
nào** (cả bốn workflow ở trạng thái `action_required`, chờ cho phép chạy); run `apple` gần nhất có mã (`e3e1ddf`, run 229)
còn `in_progress` lúc duyệt. Ngoài ra nhánh **chưa gộp** `main` hiện tại (thiếu `6ac7364` #36 và `8835cd7` #38 — core3 đổi
kiểm tra ảnh khi nhập, dọn bản sao sau 7 ngày, cắt khoảng trắng), nên diff so với `main` chưa phản ánh mã sẽ được gộp.
Các phát hiện MA3N-23-01 … -04 và BM2-04, -14, -17, -20 của Apple **chưa được kiểm lại**. Cần: gộp `main`, cho CI chạy
trên đầu nhánh, rồi duyệt.

## 6. Kết luận

| PR | Kết luận | Chặn | Nên sửa | Góp ý |
|---|---|---|---|---|
| #30 PWA vòng 2 `9e590d8` | **DUYỆT CÓ ĐIỀU KIỆN** | 0 | 2 (MA3C-30-01, -02) | 2 (MA3C-30-03, -04) |
| #24 Android `ab83b9b` | **DUYỆT CÓ ĐIỀU KIỆN** | 0 | 1 (MA3C-24-01) | 2 (MA3C-24-02, -03) |
| #25 Windows `5f4dfcc` | **DUYỆT CÓ ĐIỀU KIỆN** | 0 | 1 (MA3C-25-01) | 4 (MA3C-25-02 … -05) |
| #23 Apple | **CHƯA DUYỆT** — CI chưa chạy trên đầu nhánh, nhánh chưa gộp `main` | — | — | — |

Không còn lỗi chặn nào trong ba PR được duyệt. Lỗi chặn duy nhất của vòng trước (MA3N-25-01) đã sửa và đã được chứng minh
trên Windows thật.

Điều kiện gộp:
1. **#30:** sửa MA3C-30-01 (đúng đường người dùng được hướng tới khi nâng cấp lỗi) kèm kiểm thử xuất + khôi phục từ chế độ
   chỉ đọc; MA3C-30-02 cùng lúc.
2. **#24:** MA3C-24-01 (so nội dung, mở lại tệp sao lưu) trong đợt này, hoặc ghi lý do vào PR.
3. **#25:** MA3C-25-01 trong đợt này, hoặc ghi lý do (và bỏ ít nhất `set_lock_screen_names(true)` của
   `--tao-du-lieu-mau`).
4. **#23:** gộp `main`, CI xanh trên đầu nhánh, duyệt lại.
5. Ngoài ba PR: BM2-11 (`secure_delete`, lõi), BM2-24 (ghim action theo SHA), quét lỗ hổng Gradle (BM2-09 phần Android)
   vẫn mở.

### 6.1 Hai điểm bên dựng PWA nêu

1. **`window.__lvhv` còn trong bản phát hành** (`pwa/src/main.ts:33-38`, có từ `main` trước #30; mang `storage` = các hàm
   nạp mô-đun lưu trữ). **Không cần sửa vì lý do bảo mật.** Với CSP `script-src 'self'`, chỉ mã cùng nguồn gốc (hoặc tiện
   ích trình duyệt có quyền trên trang) chạy được trong trang; mã đó vốn đã đọc/ghi được IndexedDB `lvhv` và tự
   `import()` được mô-đun `store/index` cùng nguồn gốc. `__lvhv.storage` không mở thêm quyền nào, không giữ mật khẩu hay bản
   rõ. Đây là vệ sinh mã: **góp ý** — chuyển `storage` (và `renderMs`, `mock`) sau `__LVHV_TEST_HOOKS__`, cho `smoke.mjs` và
   `upgrade.mjs` dùng bản dựng kiểm thử; riêng ca R14 ghi dữ liệu bằng **bản cũ** thì vẫn cần `__lvhv` của bản cũ (đã có
   sẵn trong lịch sử, không bị ảnh hưởng).
2. **Tên tùy chọn `forceMigrationFailure` trong mã lưu trữ** (`pwa/src/store/db.ts:230, 249`; Rust
   `OpenOptions::force_migration_failure`, `store/src/lib.rs:337`). **Không cần sửa.** Bản Rust không có qua FFI (ứng dụng
   native không gọi được; chỉ `lvhv-store-tool` và kiểm thử dùng). Bản PWA: giao diện chỉ truyền tùy chọn này dưới
   `__LVHV_TEST_HOOKS__` (`people/core.ts:19`), nên bản phát hành luôn mở kho với `{}`; mã cùng nguồn gốc gọi được
   `openStore(indexedDB, { forceMigrationFailure: true })` nhưng tùy chọn chỉ có tác dụng **trong** một lần nâng cấp phiên
   bản IndexedDB, và kết quả là đường an toàn đã thiết kế (quay lui, giữ dữ liệu cũ, chỉ đọc) — kém hơn hẳn những gì mã đó
   vốn làm được (xóa kho). **Góp ý:** đổi tên thành `testOnlyForceMigrationFailure` và ghi rõ "không dùng trong ứng dụng"
   ở chú thích kiểu `OpenOptions` của TS như bản Rust đã ghi.
