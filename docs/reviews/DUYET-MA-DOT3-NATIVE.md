# Duyệt mã đợt 3 v0.2 — giao diện native mô-đun "Người thân": PR #24 (Android), PR #25 (Windows), PR #23 (Apple)

Trạng thái: **XONG** — 2026-11-07. Kết luận ở §5.

Bên duyệt mã: một vai không viết mã của ba PR này (docs/08 §6). Bên duyệt mã không sửa mã đang duyệt; mọi đề xuất sửa
ghi trong biên bản này. Cùng cách làm với `docs/reviews/DUYET-MA-DOT3.md` (PR #14, #15); §4 đối chiếu các chủ đề của
biên bản đó.

| PR | Nhánh | Đầu nhánh được duyệt | So với |
|---|---|---|---|
| #24 | `v0.2-w3-android` | `34c9777` | `git diff origin/main...origin/v0.2-w3-android` |
| #25 | `v0.2-w3-windows` | `5670e8a` (mã ở `8631ac2`; `5670e8a` chỉ thêm ảnh chụp) | `git diff origin/main...origin/v0.2-w3-windows` |
| #23 | `v0.2-w3-apple` | `df4faa2` (mã ở `f25ec2d`; `df4faa2` chỉ thêm ảnh chụp) | `git diff origin/main...origin/v0.2-w3-apple` |

`origin/main` = `c4d3128` (đã có PR #14, #15; FFI đã sửa MA3-14-01 … -05: `next` không giới hạn cửa sổ, O-9 đọc
từ kho, `occ` rút gọn khi ẩn tên, kiểm múi giờ/năm, `Zeroizing` cho mật khẩu).

Căn cứ: docs/v0.2/01 (§3.2a), 02 §6.1, §7.1–7.5, 03 §2.2, 04 (S5, S6, S9, S10, S12; Bổ sung B2–B4), 07, 08 (gồm §13
Android trên nhánh PR), `ffi/src/v02.rs` và `docs/v0.2/schema/ffi-v02-output.schema.json`, `docs/04-BUILD-NATIVE.md`
§3.4a (nhánh Windows).

Mức lỗi: **chặn** (phải sửa trước khi gộp) / **nên sửa** (sửa trong đợt này hoặc ghi rõ lý do để lại) / **góp ý**.
Mã lỗi: `MA3N-<PR>-<số>`. Vị trí `tệp:dòng` theo tệp trên đầu nhánh được duyệt.

## 0. Đã chạy / đã đọc

| Việc | Kết quả |
|---|---|
| CI PR #24 trên `34c9777` (check-runs) | Mọi việc `success`, gồm `build` (aapt2 so đúng danh sách quyền S6, kiểm `allowBackup=false` + `dataExtractionRules` S5) và `ui` (Robolectric + Roborazzi với lõi Rust thật) |
| CI PR #25 trên `8631ac2` (run 119) | Mọi việc `success`. `apps/windows/screenshots/thong-bao.txt`: `--kiem-thong-bao` đặt/đọc/gỡ được một thông báo thử (trạng thái `Unknown("0x80070490")`). `bo-nho.txt`: Người thân + tưởng niệm 92,0 MB Working Set |
| CI PR #23 trên `f25ec2d` (run 114) | Bước "Kiểm thử đơn vị + giao diện (iPhone mô phỏng)" `success`; bước cuối "Chạy thử trên iOS Simulator" **còn đang chạy** lúc duyệt (bắt đầu 02:02Z). Các lần 97/104/109 trước đó `failure` |
| `cargo test --locked` của `apps/windows` trên Linux (`8631ac2`, một `CARGO_TARGET_DIR` riêng, đã xóa sau) | ĐẠT 11/11 (§2.1) |
| Thăm dò `toast_xml` → `parse_toast_xml` với `body = ""` (chép nguyên hai hàm của `platform.rs`) | **Không khớp** nội dung của lõi — xem MA3N-25-01 |
| Android SDK / Xcode | Không có trong môi trường duyệt: phần Android và Apple duyệt bằng đọc mã + nhật ký CI |

## 1. PR #24 — Android

### 1.1 Đúng đặc tả, đã kiểm và không có lỗi

- **Chuỗi báo thức (02 §7.3).** `AlarmChain.run` (`notify/AlarmChain.kt:45-55`) gọi `LvhvStore.android_alarm_step_json`
  (biến thể đọc O-9 từ kho, `Reminders.kt:117-118`), đăng `post`, đặt **một** báo thức
  `setAndAllowWhileIdle(RTC_WAKEUP, next × 1000)` (`Reminders.kt:151-155`), `next = null` → hủy báo thức. PendingIntent
  cố định (requestCode 0, `FLAG_UPDATE_CURRENT|FLAG_IMMUTABLE`) nên đặt lại thay báo thức cũ, không nhân đôi.
- **Đủ điểm đặt lại chuỗi:** báo thức, `BOOT_COMPLETED`, `MY_PACKAGE_REPLACED`, `TIME_SET`, `TIMEZONE_CHANGED`
  (manifest + `ReminderReceiver.kt:13-21`, `goAsync()`), mở ứng dụng (`MainActivity.kt:140-143` `onResume`), mọi lần
  ghi qua `PeopleCtx.changed` → `Reminders.rearmAsync` (`MainActivity.kt:302`), kết quả hỏi quyền
  (`PeopleModule.kt:109-112`). Receiver `exported=false` vẫn nhận broadcast của hệ thống; các broadcast thời gian nằm
  trong danh sách miễn trừ của Android 8. Mọi bước chạy trên một luồng `lvhv-nhac` duy nhất nên không chạy song song.
- **Không đăng trùng** ở luồng bình thường: `last_run` lưu sau mỗi bước, lõi chỉ đăng `(max(last, now−24h), now]`; mã
  thông báo ổn định (FNV-1a của id lời nhắc, `AlarmChain.kt:58-66`), đăng lại cùng id thì thay.
- **O-9:** nội dung thông báo chỉ lấy `title`/`body` của lõi (`Reminders.kt:136-138`), `occ` chỉ dùng `entity_id` để
  mở hồ sơ khi chạm. Thẻ nhắc trong ứng dụng gọi `reminders_json(…, show_names = true)` có chủ đích (08 §5.5).
- **S5/S6:** `allowBackup="false"`, `fullBackupContent="false"`, `data_extraction_rules.xml` loại trừ mọi miền cho cả
  `cloud-backup` và `device-transfer`; quyền đúng và chỉ `POST_NOTIFICATIONS`, `RECEIVE_BOOT_COMPLETED` (+ quyền nội bộ
  của androidx); không `INTERNET`, không `SCHEDULE_EXACT_ALARM`; CI so khớp chính xác (`android.yml`). Ảnh chọn bằng
  `PickVisualMedia` (Photo Picker), không có quyền bộ nhớ. FileProvider chỉ lộ `cache/ics/`.
- **S9:** `Photos.kt` giải mã (`ImageDecoder` xoay theo EXIF và thu nhỏ ngay khi giải mã; API 26–27 `BitmapFactory`
  + `inSampleSize` + `ExifInterface`), vẽ lại lên nền trắng, `Bitmap.compress` JPEG — không chép siêu dữ liệu. Kích
  thước kiểm trước khi đọc và trong khi đọc (25 MB). Ảnh rất nhiều điểm ảnh không bị giải mã đầy đủ (chủ đề MA3-15-07
  đã được xử lý).
- **S12 (phần kích thước):** `readLimited` (`BackupScreen.kt:273-291`) từ chối theo `OpenableColumns.SIZE` **trước**
  khi đọc và ngắt khi luồng vượt giới hạn (chủ đề MA3-15-02 đã được xử lý); thông báo chung "Sai mật khẩu hoặc tệp bị
  hỏng" (03 §2.4).
- **Thông tin báo tử:** chỉ hiện ở `PersonScreen.kt:93-94, 200-215` (thu gọn mặc định). Không có trong thông báo,
  .ics (lõi), màn hình tưởng niệm (`MemorialScreen.kt` không đọc `notice`), tên tệp (`LichVanHoaViet-saoluu-<ngày>.lvhv`,
  tên .ics của lõi). Không có `Log.*`/`println` nào trong mã mới.
- **B4 / đang sống → đã mất:** `MarkDeceasedForm` gọi `mark_deceased_json` (cùng `id`, lõi tăng `rev`), rồi
  `x.changed()` → bước chuỗi kế tiếp tính từ tập lần xuất hiện mới, nên lời nhắc sinh nhật / sự kiện bị bỏ tự nhiên.
  Kiểm thử `flowMarkDeceased` khẳng định `rev = 2`, `sinh_nhat = null`, báo tử không vào .ics.
- **S10:** .ics có hộp xác nhận nêu số người, số ngày, xem trước 2 mục, "Không gồm thông tin báo tử"
  (`BackupScreen.kt:86-107`); tạo sao lưu nêu số người/sự kiện/ảnh và "gồm cả thông tin báo tử" ngay trong khung (sai
  khác đã ghi ở 08 §13); "Thay toàn bộ" có hộp xác nhận thứ hai.
- **Mật khẩu:** giữ trong `remember` (không `rememberSaveable`, nên không vào `Bundle` khi tiến trình bị hủy); xóa
  ô sau khi tạo sao lưu thành công. Argon2id chạy trên `Dispatchers.Default` cho cả xuất và mở tệp.

### 1.2 Các lỗi

#### MA3N-24-01 — **nên sửa** — "Khôi phục" (`apply_import`) chạy trên luồng giao diện

- Vị trí: `ui/people/BackupScreen.kt:201-217` (`doImport` gọi `x.repo.applyImport` thẳng trong `onConfirm`).
- Vấn đề: ghi một giao dịch SQLite và tới hàng nghìn tệp ảnh (tệp tối đa 128 MiB) trên luồng chính. Mở tệp và xuất
  đã đưa ra luồng nền, riêng bước nặng nhất này thì không. Nút "Khôi phục" không bị khóa trong lúc chạy.
- Kịch bản hỏng: khôi phục một tệp 300 ảnh trên điện thoại tầm trung → màn hình đứng > 5 giây → Android hiện hộp
  "Ứng dụng không phản hồi"; người dùng chọn "Đóng ứng dụng" giữa giao dịch (lõi vẫn an toàn nhờ giao dịch, nhưng
  người dùng không biết dữ liệu đã thay hay chưa).
- Cách sửa: `scope.launch { busy = true; val r = withContext(Dispatchers.IO) { runCatching { x.repo.applyImport(…) } }; busy = false; … }`,
  hiện `LinearProgressIndicator` và khóa hai nút của hộp thoại (như phần xuất ở `:116-133`).

#### MA3N-24-02 — **nên sửa** — bản sao lưu đã giải mã và mật khẩu khôi phục còn trong bộ nhớ sau khi người dùng hủy

- Vị trí: `BackupScreen.kt:172` (`verified`), `:222` (`onDismiss = { verified = null }`), `:213` (lỗi), `:169`/`:209`
  (`pwIn` chỉ xóa khi khôi phục thành công).
- Vấn đề: `VerifiedBackup` là đối tượng UniFFI giữ **bản rõ** (gồm thông tin báo tử, ảnh giấy báo tử). Gán `null`
  chỉ bỏ tham chiếu Kotlin; vùng nhớ Rust chỉ được giải phóng khi `Cleaner` chạy sau GC, không biết lúc nào. Ô mật
  khẩu khôi phục giữ nguyên chữ khi người dùng hủy hộp xem trước. MA3-14-05 đã đề nghị gọi trong `use { }`.
- Kịch bản hỏng: người dùng mở tệp sao lưu, thấy xem trước rồi bấm Hủy và để máy đó; bản rõ (lý do mất, người báo
  tin) nằm trong bộ nhớ tiến trình đến khi GC chạy, mật khẩu vẫn nằm trong ô nhập.
- Cách sửa: một hàm `dropVerified()` gọi `verified?.first?.destroy()` rồi `verified = null; pwIn = ""`, dùng ở
  `onDismiss`, nhánh lỗi và sau `applyImport` (trong `finally`). Khi tạo sao lưu lỗi hoặc hủy hộp "Lưu tệp" thì
  giữ ô (để thử lại) là chấp nhận được.

#### MA3N-24-03 — **nên sửa** — đọc tệp sao lưu gần giới hạn tốn 3–5 lần kích thước tệp (S12: đỉnh < 200 MB)

- Vị trí: `BackupScreen.kt:278-290` (`ByteArrayOutputStream` tự nới + `toByteArray()`), rồi `openBackup(bytes, …)`
  (UniFFI chép mảng sang Rust), rồi lõi giải mã ra bản rõ.
- Vấn đề: tệp 128 MiB: bộ đệm `ByteArrayOutputStream` nới tới 256 MiB, `toByteArray` chép thêm 128 MiB, UniFFI chép
  thêm 128 MiB, bản rõ thêm ~128 MiB → khoảng 500–640 MiB. Heap Java của ứng dụng thường 256–512 MiB.
- Kịch bản hỏng: một tệp `.lvhv` độc hại có phần đầu hợp lệ và `plaintext_len` = 128 MiB (vượt qua kiểm tra
  03 §2.2) → `OutOfMemoryError`; `runCatching` bắt được ở luồng này nhưng các luồng khác (Compose) có thể hỏng tiếp.
  Tệp hợp lệ lớn của chính người dùng cũng không khôi phục được trên máy RAM thấp.
- Cách sửa: khi biết `size`, cấp đúng `ByteArray(size.toInt())` và đọc đầy (`DataInputStream.readFully`), kiểm
  thêm một byte để chắc hết tệp; khi không biết, giữ cách hiện tại. Tốt hơn (sửa ở lõi, dùng chung ba nền tảng): thêm
  FFI `backup_header_check(first_72_bytes, file_len)` kiểm 03 §2.2 **trước** khi đọc phần thân. Thêm kiểm thử S12 trên
  JVM với tệp có phần đầu hợp lệ và `m_cost_kib` = 4 GiB (phải bị từ chối, không cấp bộ nhớ).

#### MA3N-24-04 — **nên sửa** — chuỗi báo thức dừng lặng lẽ khi một bước lỗi

- Vị trí: `notify/Reminders.kt:64-70` (`try { rearm(app) } catch (_: Exception) { }`), `:73-77`.
- Vấn đề: khi `step` ném lỗi (kho không mở được, `LvhvDataException` do dữ liệu hoặc múi giờ, `InternalException` của
  UniFFI), không có báo thức nào được đặt lại và lỗi bị nuốt, không ghi lại ở đâu. Nếu lỗi xảy ra ở lần báo thức tự
  chạy (không ai mở ứng dụng), chuỗi dừng cho tới lần khởi động lại hoặc mở ứng dụng kế tiếp. Đây đúng loại "bỏ lời
  nhắc lặng lẽ" mà B3/MA3-14-01 muốn tránh.
- Kịch bản hỏng: một lần nâng cấp làm `LvhvStore.open` lỗi tạm thời (tệp bị khóa) đúng lúc báo thức chạy → không còn
  thông báo nào trong nhiều tuần nếu người dùng chỉ dựa vào thông báo.
- Cách sửa: trong nhánh `catch`, đặt một báo thức thử lại (ví dụ `now + 6 giờ`) bằng cùng `PendingIntent`, và ghi
  `last_error_at` vào `SharedPreferences` để trang "Cài đặt nhắc" hiện "Lần đặt lời nhắc gần nhất bị lỗi — mở ứng
  dụng để thử lại". Không ghi nội dung lỗi có dữ liệu người dùng.

#### MA3N-24-05 — **góp ý** — đồng hồ lùi (TIME_SET) đăng lại các thông báo của 24 giờ trước

- Vị trí: `AlarmChain.kt:47-49` (`lastOk = last?.takeIf { it <= nowUnixS }` → `null`).
- Vấn đề: khi `last_run` ở tương lai, truyền `null` khiến lõi đăng lại mọi lời nhắc trong `(now − 24h, now]`, kể cả
  những cái người dùng đã đọc và gạt đi. Mã thông báo ổn định nên không nhân đôi trong khay, nhưng thông báo đã gạt hiện
  lại.
- Cách sửa: khi `last > now` thì truyền `nowUnixS` (không có gì "đến hạn"), không phải `null`. Kiểm thử
  `clockMovedBackTreatsLastRunAsUnknown` đổi theo.

#### MA3N-24-06 — **góp ý** — quyền thông báo bị từ chối: lời nhắc trong ngày bị "tiêu" mất; bật tên thì lấn át cài đặt riêng tư của máy

- Vị trí: `AlarmChain.kt:50-51` (`setLastRun` cả khi `canPost() = false`); `Reminders.kt:85`, `:140`
  (`VISIBILITY_PUBLIC`).
- Vấn đề: (a) quyền bị tắt rồi bật lại trong vòng 24 giờ thì lời nhắc trong khoảng đó không bao giờ đăng, dù 02 §7.3
  cho đăng bù tới 24 giờ. (b) Khi người dùng bật "Hiện tên", `VISIBILITY_PUBLIC` khiến tên hiện cả khi cài đặt hệ thống
  là "Ẩn nội dung nhạy cảm".
- Cách sửa: (a) chỉ `setLastRun` khi `canPost()`; lõi đã tự giới hạn 24 giờ. (b) Khi `show_names = true` dùng
  `VISIBILITY_PRIVATE` + `setPublicVersion(...)` với tiêu đề không tên ("Sắp đến ngày giỗ"); khi `false` giữ
  `PUBLIC`.

#### MA3N-24-07 — **nên sửa** — kiểm thử chưa bắt được lỗi ở phần dễ hỏng nhất

- Vị trí: `src/test/.../notify/AlarmChainTest.kt:94-114`, `ui/people/PeopleScreensTest.kt:241-261`;
  `.github/workflows/android.yml:100`.
- Vấn đề:
  1. Không có kiểm thử nào cho `ReminderReceiver` (đủ năm action) và cho việc lưu / xóa / nhập sao lưu / đổi cài đặt
     gọi `rearmAsync` — xóa một dòng `x.changed()` không làm đỏ kiểm thử nào.
  2. B4 qua chuỗi báo thức chưa được kiểm: không có ca "đặt chuỗi → `mark_deceased` → bước kế tiếp không còn lời
     nhắc sinh nhật của người đó".
  3. Khẳng định lỏng: `:106` `scheduled > first + 60` (đúng cả khi đặt sai lời nhắc) — nên `assertEquals` với
     `at_unix_s` của lời nhắc thứ hai trong `remindersJson`; `:110` tìm "Bạch Mai" trong `body` vốn luôn rỗng khi ẩn
     tên, nên không kiểm gì; nên kiểm trên `remindersJson(..., showNames = true)` là chỗ có thể lộ.
  4. Không có kiểm thử S9 cho `Photos.encode` (Robolectric hỗ trợ `@GraphicsMode(NATIVE)`): JPEG có APP1 GPS + APP13
     → byte ra không còn hai đoạn đó và hướng ảnh đúng.
  5. CI đổi `testReleaseUnitTest` thành `testDebugUnitTest`: bản release (R8) không còn chạy kiểm thử nào.
- Cách sửa: thêm các ca trên; với (1) dùng `ShadowAlarmManager.getNextScheduledAlarm()` và gửi từng intent vào
  receiver; với (5) chạy thêm `:app:testReleaseUnitTest` cho phần không cần `ui-test-manifest`, hoặc ghi lý do.

#### MA3N-24-08 — **góp ý** — logic lặp lại lẽ ra nằm ở lõi; CI có quyền ghi chạy mã của PR

- `people/PeopleRepo.kt:106-110` xóa sự kiện mồ côi sau `delete_person` — cùng logic ở Windows
  (`people.rs:2575-2595`) và Apple (`PeopleCore.swift:426-432`), mỗi bản một cách viết. Nên là tùy chọn của
  `LvhvStore.delete_person` (một giao dịch).
- `PeopleRepo.kt:156-164` dò tháng nhuận bằng 12 lần `save_death_json` — Windows có `leap_month` riêng
  (`people.rs:1273`). Nên thêm FFI `lunar_leap_month(year)`.
- `PeopleRepo.kt:90-96` chọn "lời nhắc hôm nay" cho thẻ nhắc — cùng ý ở Windows (`people.rs:593-626` và lặp lại ở
  `:3150-3168`) và Apple (`PeopleCore.swift:350-360`). Đây là chủ đề MA3-15-05 lặp lại trên ba nền tảng: nên thêm
  `LvhvStore.due_today_json(today, zone)` ở lõi.
- `BackupScreen.kt:45-60` tự đọc .ics để xem trước — Windows (`people.rs:2722-2731`) và Apple (`IcsPreview`) mỗi bản
  một cách. Nên để `IcsExport` mang sẵn `preview: Vec<String>`.
- Kiểm độ dài mật khẩu: Android `pw1.length` (đơn vị UTF-16), Windows `chars().count()` chưa NFC, Apple `count`
  (cụm chữ); lõi đếm điểm mã sau NFC (`backup/src/lib.rs:65, 458`). Bốn emoji qua được kiểm tra của Android rồi bị lõi
  từ chối **sau khi** hộp "Lưu tệp" đã tạo một tệp rỗng. Nên thêm FFI `backup_password_error(pw) -> Option<String>`.
- `android.yml` job `ui` có `contents: write` và chạy Gradle của nhánh PR, rồi đẩy commit ảnh vào nhánh. Điều kiện
  chỉ cho PR trong kho, nhưng nên tách bước commit sang một job riêng không chạy mã của PR (nhận ảnh qua artifact).

### 1.3 Kết luận PR #24

**DUYỆT CÓ ĐIỀU KIỆN** — không có lỗi chặn. Chuỗi báo thức, quyền, S5/S6/S9, O-9 và B4 đúng đặc tả. Nên sửa
MA3N-24-01 … -04 và -07 trong đợt 3, trước T11 (thử máy thật R5/P8).

## 2. PR #25 — Windows

### 2.1 Đã chạy

`cargo test --locked` trong `apps/windows` trên `8631ac2` (Linux, backend kiểm thử của Slint, một `CARGO_TARGET_DIR`
riêng, đã xóa sau khi chạy): **ĐẠT, 11/11** — gồm `ui_tests_people::people_flows`, `photo::tests::strips_metadata_and_rotates`,
`photo::tests::downscales_and_errors`, `platform::tests::toast_xml_roundtrip`, `ui_tests::all_clickables_work`. Trên Linux,
các hàm thông báo của Windows là bản thay thế không làm gì, nên kết quả này **không** phủ đường đặt thông báo
(MA3N-25-08).

### 2.2 Đúng đặc tả, đã kiểm và không có lỗi

- **Xoay vòng 64 (02 §7.4–7.5):** ứng dụng đọc danh sách chờ của Windows, gọi `LvhvStore::reschedule_json` (biến thể
  đọc O-9 từ kho) và áp `remove`/`add`/`replace` (`people.rs:725-785`, `platform.rs:431-447`); không tự tính lời
  nhắc. Đặt lại khi khởi động nếu đã có kho (`main.rs`, `setup`), khi mở mục Người thân lần đầu, sau mọi lần ghi
  (`after_change`: lưu, xóa, đánh dấu đã mất, khôi phục) và khi đổi cài đặt. Không có tiến trình nền.
- **AUMID:** `ClevixLab.LichVanHoaViet` trùng giữa `platform.rs:10`, `SetCurrentProcessExplicitAppUserModelID` và
  `ShortcutProperty System.AppUserModel.ID` của lối tắt Start Menu (`wix/main.wxs`); có kiểm thử so chuỗi hai chỗ
  (`platform.rs:501-503`) và tự kiểm trên CI sau khi cài `.msi`. Chế độ chụp màn hình/đo bộ nhớ không đặt thông báo
  thật (`set_demo_mode`).
- **Dữ liệu:** `%LOCALAPPDATA%\ClevixLab\LichVanHoaViet` (`platform.rs:14-28`); kho chỉ tạo khi mở mục Người thân.
  Không có mã mạng nào mới (crate `windows` chỉ bật Foundation, Xml, Notifications, Time, Shell, Dialogs).
- **S9:** `photo.rs` kiểm kích thước điểm ảnh **trước** khi giải mã (≤ 12 000 px mỗi cạnh, ≤ 50 MP — chủ đề MA3-15-07
  đã xử lý), xoay theo EXIF, thu nhỏ, trộn nền trắng, mã hóa lại JPEG; kiểm thử khẳng định không còn APP1/APP13 và
  không còn chuỗi GPS cài vào.
- **S12 (kích thước tệp sao lưu):** `fs::metadata` trước `fs::read` (`people.rs:2815`); thông báo chung.
- **Thông tin báo tử:** chỉ ở trang hồ sơ (`render_person`, `:1026-1060`, thu gọn mặc định); `open_memorial` không
  đọc `notice`; không có `println!`/`eprintln!` ngoài chế độ `--kiem-thong-bao`. Kiểm thử khẳng định chuỗi bí mật của
  báo tử không có trong .ics và có trên trang hồ sơ.
- **B4:** `do_mark` gọi `mark_deceased_json` (cùng `id`, `rev + 1`) rồi `after_change` → `reschedule`; kiểm thử
  khẳng định cùng `id`, `rev`, giữ ngày sinh, Sắp tới chỉ còn giỗ / tiên thường của người đó.
- **S10:** .ics và tạo sao lưu đều qua hộp xác nhận nêu số người/số mục/loại dữ liệu ("gồm cả thông tin báo tử" chỉ
  khi có); "Thay toàn bộ" có hộp thứ hai; hộp không đóng được khi `dlg_busy`. Ô mật khẩu tạo sao lưu được xóa khi
  đóng hộp (`close_dialog`); `ps.backup` (bản rõ) được bỏ ngay khi hủy (`:3135-3140`) và bị hủy sau `apply_import`.

### 2.3 Các lỗi

#### MA3N-25-01 — **chặn** — thông báo không có dòng phụ (mặc định O-9) không đọc lại đúng nội dung → xoay vòng hỏng ở chế độ mặc định

- Vị trí: `platform.rs:241-248` (`toast_xml` luôn ghi hai `<text>`, cái thứ hai rỗng khi `body = ""`),
  `platform.rs:251-269` (`parse_toast_xml`), `platform.rs:334-352` (`pending`), `:354-369` (`remove`).
- Vấn đề: khi không hiện tên (mặc định, 02 §7.1 Bổ sung B2), lõi trả `body = ""` cho **mọi** lời nhắc và cho thông
  báo nhắc mở ứng dụng; `to_pending` của lõi đặt `content = title` (`memorial/src/remind.rs:362-366`). Ứng dụng ghi
  `<text>title</text><text></text>` rồi đọc lại. Đã thăm dò với đúng hai hàm này:
  `content đọc lại = "Sắp đến ngày giỗ\n"`, `content của lõi = "Sắp đến ngày giỗ"` → **không khớp**. Ngoài ra, nếu
  Windows trả XML đã chuẩn hóa với phần tử rỗng dạng `<text/>` (cách tuần tự hóa thường gặp của DOM XML; môi trường
  duyệt không có Windows để xác nhận), `parse_toast_xml` không tìm thấy `</text>` thứ hai và trả `None`.
- Kịch bản hỏng (một trong hai, tùy cách Windows trả XML; cả hai đều xảy ra với cài đặt mặc định của mọi người dùng):
  1. Nội dung lệch → lõi coi mọi thông báo đang chờ là "khác nội dung" → mỗi lần mở ứng dụng, mỗi lần lưu, cả 64 thông
     báo bị gỡ rồi đặt lại. Nếu một lần `add` lỗi giữa chừng (`apply_plan` trả `Err` sau khi đã gỡ), các thông báo
     còn lại mất cho tới lần mở sau.
  2. `None` → `pending()` bỏ qua (`continue`) mọi thông báo của ứng dụng → lõi thấy danh sách chờ rỗng → mỗi lần mở
     ứng dụng **thêm** 64 thông báo mới mà không gỡ cái cũ (`remove` cũng không nhận ra chúng). Thông báo bị nhân đôi
     mỗi lần mở, và thông báo sinh nhật của người vừa được cập nhật đã mất, hoặc của hồ sơ đã xóa, **không bao giờ bị
     gỡ** — trái B4 mục 4 và 02 §7.4.
  Tự kiểm trên CI (`--kiem-thong-bao`) dùng `body` không rỗng nên không phát hiện; kiểm thử `toast_xml_roundtrip` cũng
  chỉ thử `body` không rỗng.
- Cách sửa:
  1. Không dựa vào việc đọc lại văn bản hiển thị để so: ghi nguyên `content` của lõi vào một thuộc tính riêng, ví dụ
     `<toast launch="{id}" lvhv-content="{xml_escape(content)}">` (hoặc mã hóa `id` + `content` trong `launch`), và
     đọc lại từ đó. Khi `body` rỗng chỉ ghi **một** `<text>`.
  2. `parse_toast_xml` chấp nhận `<text/>`; mọi thông báo thuộc nhóm `lvhv` mà không đọc được thì đưa vào `remove`
     (gỡ đi), không bỏ qua.
  3. Kiểm thử: vòng `toast_xml` → `parse_toast_xml` với `body = ""`, với XML viết lại dạng `<text/>`, và với
     `content` của `to_pending` khi `show_names = false` → kế hoạch thứ hai phải có `add = replace = remove = []`.
     Thêm vào `--kiem-thong-bao` một thông báo `body` rỗng và khẳng định nội dung đọc lại bằng `content` của lõi.

#### MA3N-25-02 — **nên sửa** — đọc danh sách chờ lỗi thì coi như rỗng → nhân đôi thông báo

- Vị trí: `people.rs:734` (`platform::pending_toasts().unwrap_or_default()`).
- Vấn đề: lỗi WinRT khi đọc danh sách chờ (thường gặp ngay sau khi cài, xem `0x80070490` trên CI) bị đổi thành "không
  có gì đang chờ"; lõi trả `add` cho toàn bộ, không `remove` gì, và ứng dụng đặt thêm tới 64 thông báo trùng.
- Cách sửa: lỗi đọc → không áp kế hoạch, hiện trạng thái "Chưa đọc được danh sách thông báo của Windows — sẽ thử lại
  khi mở ứng dụng"; hoặc gỡ cả nhóm `lvhv` (`RemoveFromSchedule` mọi mục có `Group = "lvhv"`) rồi đặt lại từ đầu.

#### MA3N-25-03 — **nên sửa** — thông báo bị tắt trong Cài đặt Windows thì không gỡ thông báo cũ (B4, xóa hồ sơ, O-9)

- Vị trí: `people.rs:776-777` (`Disabled`, `Unavailable`: chỉ đổi dòng trạng thái).
- Vấn đề: khi `Setting()` khác `Enabled`, ứng dụng không đụng tới danh sách chờ. Các thông báo đã đặt trước đó vẫn
  nằm trong Windows.
- Kịch bản hỏng: người dùng tắt thông báo của ứng dụng, cập nhật một người là đã mất (hoặc xóa hồ sơ, hoặc tắt "hiện
  tên"), rồi bật lại thông báo trong Cài đặt Windows mà chưa mở lại ứng dụng → thông báo sinh nhật của người đã mất,
  hoặc thông báo có tên, vẫn hiện.
- Cách sửa: khi tạo được `ToastNotifier` thì luôn đối chiếu; với `Disabled`, gỡ mọi thông báo đang chờ của ứng dụng
  (giống Apple khi bị từ chối, `ReminderScheduler.swift:200-205`) để lần bật lại đặt từ đầu.

#### MA3N-25-04 — **nên sửa** — trạng thái `Unknown` vẫn báo "Thông báo của Windows đang bật"

- Vị trí: `people.rs:733` (`Ready | Unknown(_)` chung một nhánh) và chuỗi ở `:766`.
- Vấn đề: `docs/04-BUILD-NATIVE.md` §3.4a ghi bản `.zip` (không có lối tắt mang AUMID) "Windows có thể từ chối thông
  báo; giao diện ghi rõ". Nhưng khi Windows không đọc được cài đặt (`Unknown`, đúng tình huống của bản không cài),
  `AddToSchedule` vẫn có thể thành công mà thông báo không bao giờ hiện, và giao diện ghi "Thông báo của Windows đang
  bật. Đã đặt n lời nhắc …".
- Cách sửa: với `Unknown`, ghi "Đã đặt n lời nhắc, nhưng Windows chưa xác nhận ứng dụng được phép hiện thông báo. Nếu
  bạn dùng bản không cài đặt (.zip), hãy cài bản .msi; có thể xuất tệp .ics để lịch của máy nhắc." và để
  `notif_ok = false`.

#### MA3N-25-05 — **nên sửa** — múi giờ không có lần chuyển giờ mùa hè (lệch với Android/Apple, R-NAT-1)

- Vị trí: `people.rs:74-79` (`transitions: vec![]`), `platform.rs:52-79`.
- Vấn đề: Android (`Zones.device`) và Apple (`DeviceZones.current`) truyền các lần chuyển trong 401 ngày; Windows chỉ
  truyền độ lệch hiện tại, rồi đặt thông báo theo **thời điểm UTC** (`DeliveryTime`). Với người dùng ở nước có giờ mùa
  hè, mọi lời nhắc sau lần đổi giờ lệch 1 giờ cho tới khi mở lại ứng dụng — và thông báo nhắc mở ứng dụng có thể ở xa
  tới 400 ngày, nên "mở lại" không phải là chuyện chắc chắn. Giới hạn này đã được ghi, nhưng sửa không tốn nhiều.
- Cách sửa: dùng `GetTimeZoneInformationForYear` (năm nay và năm sau) để dựng các lần chuyển từ `StandardDate` /
  `DaylightDate` (quy tắc "tuần thứ n của tháng"), lọc trong `[now, now + 401 ngày]`, truyền vào `DeviceZone`. Đặt lại
  lịch khi nhận `WM_TIMECHANGE` / `WM_SETTINGCHANGE("intl")` nếu ứng dụng đang mở. Kiểm thử thuần Rust cho hàm dựng
  lần chuyển (ví dụ Europe/Berlin 2026: 29/03 01:00Z và 25/10 01:00Z).

#### MA3N-25-06 — **nên sửa** — Argon2id, đọc tệp, khôi phục và giải mã ảnh chạy trên luồng giao diện

- Vị trí: `people.rs:2637-2643` (`set_dlg_busy(true)` rồi `export_backup` đồng bộ), `:2819-2826` (`fs::read` +
  `open_backup`), `:2690-2700` (`apply_import`), `:3040`, `:3064` (`photo::reencode`).
- Vấn đề: vòng sự kiện bị chặn nên trạng thái "đang chạy" không bao giờ được vẽ; cửa sổ đứng 1–2 giây (theo tài liệu
  của PR) và lâu hơn trên máy yếu (P8 cho tới 3 giây; một ảnh 50 MP giải mã + xoay ~1 giây). Quá 5 giây Windows đánh dấu
  "Không phản hồi". Cú bấm của người dùng trong lúc đứng được xếp hàng và đến tay phần tử nằm dưới sau khi hộp đã đóng.
- Đánh giá: không làm hỏng dữ liệu (một giao dịch, hộp có cờ `busy`), nên không xếp "chặn"; nhưng đây là hành vi mà
  Android và Apple đã tránh được.
- Cách sửa: `LvhvStore` là `Arc` + `Mutex` (`Send + Sync`), nên chạy `export_backup` / `open_backup` / `apply_import` /
  `reencode` trong `std::thread::spawn`, trả kết quả bằng `slint::invoke_from_event_loop` (giữ `Weak<AppWindow>`),
  khóa các nút bằng `dlg_busy` trong lúc chờ. Sửa dòng "Giới hạn đã biết" ở 04-BUILD-NATIVE §3.4a.

#### MA3N-25-07 — **nên sửa** — chọn ảnh đọc toàn bộ tệp trước khi kiểm 25 MB

- Vị trí: `people.rs:3040`, `:3064` (`std::fs::read(&path)` rồi mới `photo::reencode`, nơi kiểm `MAX_INPUT`).
- Vấn đề: cùng chủ đề MA3-15-02 cho ảnh. Hộp chọn tệp lọc theo đuôi, nhưng người dùng vẫn gõ được tên tệp bất kỳ, và
  một tệp `.jpg` vài GB bị đọc hết vào bộ nhớ trên luồng giao diện.
- Cách sửa: `fs::metadata(&path)?.len() > MAX_INPUT as u64` → báo "Ảnh lớn hơn 25 MB" trước khi đọc.

#### MA3N-25-08 — **nên sửa** — kiểm thử không phủ phần thông báo; một kiểm thử khổng lồ

- Vị trí: `ui_tests_people.rs` (một hàm `people_flows` ~340 dòng), `platform.rs:493-504`.
- Vấn đề: trên Linux `apply_plan` trả `Err` và `pending_toasts` trả rỗng, nên không kiểm thử nào chạy qua đường
  `reschedule` → `apply_plan`. MA3N-25-01/-02/-03 đều nằm ở đó. Kiểm thử luồng giao diện thì tốt (khẳng định giá trị
  chính xác, so với lõi), nhưng một lỗi ở đầu làm mọi phần sau không chạy.
- Cách sửa: tách `platform` thông báo thành một trait `Toasts { pending, remove, add, state }` có bản giả trong bộ nhớ
  (giống `AlarmChain.Host` của Android và `NotificationCenterLike` của Apple); thêm kiểm thử: (a) lập lịch hai lần liền
  nhau → lần hai không gỡ/thêm/thay gì, ở cả `show_names` bật và tắt; (b) cập nhật đã mất → `remove` chứa id sinh
  nhật của người đó; (c) đọc danh sách chờ lỗi → không thêm gì; (d) `Disabled` → gỡ hết. Tách `people_flows` thành
  vài hàm theo luồng.

#### MA3N-25-09 — **góp ý** — dọn thông báo cũ, mật khẩu khôi phục, lỗi bị nuốt, việc làm thừa

- Dọn thông báo cũ: gỡ cài `.msi` không gỡ thông báo đang chờ và không xóa `%LOCALAPPDATA%\ClevixLab\LichVanHoaViet`
  (sổ người thân, ảnh, giấy báo tử vẫn còn trên máy). Nên ghi rõ trong trang Giới thiệu / tài liệu phát hành, hoặc thêm
  nút "Xóa toàn bộ dữ liệu Người thân trên máy này" (gỡ cả nhóm thông báo `lvhv` và `ToastNotificationHistory` của
  AUMID).
- `People.restore-pw` không được xóa khi hủy hộp xem trước (`people.rs:3135-3140` chỉ xóa `pw1`/`pw2` qua
  `close_dialog`) — nên `set_restore_pw("")` ở đó.
- `people.rs:2888-2889`: `let _ = store.put_settings_json(…)`, `let _ = store.set_lock_screen_names(…)` — lỗi ghi
  bị nuốt, giao diện hiện trạng thái chưa được lưu. Nên báo lỗi bằng `toast` và vẽ lại từ kho.
- `render_settings` (`:703-707`) sinh toàn bộ tệp .ics 10 năm sau **mỗi** lần `refresh` chỉ để biết có mục nào
  không — nên dùng `upcoming_json` hoặc một hàm đếm của lõi.
- `export_ics` (`:2722-2726`): `&d[6..8.min(d.len())]` sẽ hoảng nếu `d` ngắn hơn 6 ký tự. Lõi luôn sinh 8 chữ số nên
  hiện không xảy ra; dùng `d.get(6..8)` (hoặc lấy xem trước từ lõi, MA3N-24-08).
- Chọn thẻ nhắc hôm nay viết hai lần (`:593-626` và `:3150-3168`) — xem MA3N-24-08.
- CI commit ảnh vào nhánh PR với `[skip ci]`, nên đầu nhánh PR không có check-run nào (`5670e8a`). Nếu bật bảo vệ
  nhánh theo trạng thái, PR sẽ không gộp được; và người duyệt phải tự tìm commit mã cuối cùng.

### 2.4 Kết luận PR #25

**CHƯA DUYỆT** — sửa **MA3N-25-01** (chặn) kèm kiểm thử, rồi duyệt lại phần thông báo. MA3N-25-02, -03 nên sửa cùng
lúc vì cùng nằm trong `reschedule`/`apply_plan`; MA3N-25-08 là điều kiện để các sửa này có kiểm thử. Phần còn lại
(giao diện, S9, S10, B4, dữ liệu) đạt.

## 3. PR #23 — Apple (đang mở, đã duyệt)

### 3.1 Đúng đặc tả, đã kiểm và không có lỗi

- **Lập lịch lại (02 §7.4):** `ReminderScheduler.reschedule` gọi `LvhvStore.reschedule_json` (O-9 từ kho), áp
  `remove + replace` rồi `add`/`replace`; trigger `UNCalendarNotificationTrigger(dateMatching: y-m-d h:m)` từ
  `fire_local`, không gắn múi giờ. `content` đọc lại theo đúng quy tắc `to_pending` của lõi (`title` khi `body` rỗng,
  `ReminderScheduler.swift:15`) — kiểm thử `testRescheduleHidesNamesByDefaultAndIsIdempotent` khẳng định lần hai không
  thêm/thay gì (đúng chỗ Windows hỏng ở MA3N-25-01).
- **Điểm gọi:** trở lại tiền cảnh (`RootView.swift:99-104`), `NSSystemTimeZoneDidChange` (`:108`), lưu/xóa/đánh dấu đã
  mất/đổi cài đặt (`PeopleCore.swift:404-408`), nhập sao lưu (`:502`), chạm vào thông báo
  (`ReminderScheduler.swift:229-243`), cấp quyền (`:147-151`). Các lần gọi chồng nhau được gộp (`running`/`rerun`).
- **Quyền:** không hỏi khi mở ứng dụng hay đổi múi giờ, chỉ khi lưu dữ liệu có nhắc; bị từ chối → gỡ mọi yêu cầu cũ.
- **S5:** `isExcludedFromBackup` cho thư mục kho (bao cả `photos/`, `-wal`, `-shm`) sau khi mở và sau mỗi lần thêm ảnh;
  có kiểm thử. **S9:** ImageIO + vẽ lại + `JPEGSanitizer` bỏ APP1…APP15 và COM; kiểm thử xoay và bỏ đoạn.
- **S12:** kiểm `fileSize` trước khi đọc; Argon2id ở `Task.detached`. Hộp tạo sao lưu khóa vuốt đóng khi `busy`.
  Mật khẩu: ô `SecureField`, tắt tự sửa; xóa sau khi xuất; `verified` (ARC) và mật khẩu khôi phục bỏ ngay sau khi
  khôi phục hoặc đóng hộp.
- **Báo tử:** chỉ ở `PersonDetailView` (thu gọn); tiêu đề điều hướng không có tên; .ics xác nhận đủ (S10), tệp tạm
  bị xóa khi đóng hộp.

### 3.2 Các lỗi

#### MA3N-23-01 — **nên sửa** — hủy hộp "Lưu" vẫn ghi nhận là đã sao lưu (lõi đặt `last_backup_at` trước khi tệp được lưu)

- Vị trí: `BackupView.swift:333-345` (`m.exportBackup` trước), `:122-128` (`fileExporter` sau); lõi
  `store/src/lib.rs:727-733` (`export_backup` gọi `after_export` → `last_backup_at = now`).
- Vấn đề: trên Apple, byte được tạo (và lõi ghi "đã sao lưu") **trước** khi người dùng chọn nơi lưu. Nếu người dùng
  bấm Hủy ở `fileExporter`, không có tệp nào, nhưng "Lần sao lưu gần nhất" là hôm nay và lời nhắc sao lưu
  (`backup_nag_days`) tắt. Android và Windows chọn nơi lưu trước, nhưng cũng gặp cùng lỗi khi ghi tệp thất bại
  (`BackupScreen.kt:123`, `people.rs:2642`).
- Kịch bản hỏng: người dùng mở hộp tạo sao lưu, đổi ý ở bước chọn thư mục; 30 ngày sau máy hỏng, ứng dụng chưa từng
  nhắc sao lưu lại.
- Cách sửa (ở lõi, dùng chung): tách `export_backup` (chỉ tạo byte) và `LvhvStore.mark_backup_saved(exported_at)` (gọi
  sau khi hệ điều hành báo ghi xong: `fileExporter` `.success`, `openOutputStream` ghi xong, `fs::write` `Ok`). Sửa cả ba
  ứng dụng và R6 nếu cần.

#### MA3N-23-02 — **nên sửa** — khôi phục và đọc tệp 128 MiB trên luồng chính

- Vị trí: `PeopleCore.swift:498-503` (`applyImport` đồng bộ trên `@MainActor`); `BackupView.swift:159-165`
  (`Task { … Data(contentsOf:) }` kế thừa `MainActor`, nên đọc tệp chạy trên luồng chính).
- Vấn đề, kịch bản: như MA3N-24-01 — giao diện đứng khi khôi phục tệp lớn; iOS có thể chấm dứt ứng dụng nếu luồng
  chính không phản hồi lâu.
- Cách sửa: `Task.detached` cho `Data(contentsOf:)` và cho `applyImport` (lõi an toàn đa luồng), rồi `reload()` và
  `reschedule` trên `MainActor`; hiện tiến trình trong `RestoreSheet`, khóa nút.

#### MA3N-23-03 — **góp ý** — kiểm thử xoay vòng còn lỏng; chưa kiểm B4 qua bộ lập lịch

- Vị trí: `Tests/Unit/PeopleGlueTests.swift:213-228` (`XCTAssertLessThanOrEqual(fake.store.count, 64)`).
- Vấn đề: 70 hồ sơ × 2 mốc chắc chắn vượt 64, nên kết quả đúng phải là **đúng** 64 (63 + thông báo nhắc mở ứng dụng),
  thông báo nhắc cùng phút với cái thứ 63. Đặt 1 cái vẫn qua kiểm thử hiện tại. Chưa có ca "đặt lịch → `markDeceased`
  → lần sau `remove` có id sinh nhật".
- Cách sửa: `XCTAssertEqual(fake.store.count, 64)`; so `atUnix` của `lvhv:sentinel` với cái muộn nhất còn lại; thêm ca
  B4 dùng `FakeCenter`.

#### MA3N-23-04 — **góp ý** — `BGAppRefreshTask` chưa có; CI chưa xanh trọn vẹn trên đầu nhánh

- 02 §7.4 liệt kê `BGAppRefreshTask` là một điểm gọi `reschedule` ("không dựa vào nó"). Chưa cài; thông báo nhắc mở
  ứng dụng đã che trường hợp này. Nếu để lại, ghi vào phần sai khác của 08.
- Run 114 trên `f25ec2d`: kiểm thử đơn vị và giao diện đạt, bước chạy thử trên Simulator chưa xong lúc duyệt; các run
  97/104/109 trước đó thất bại. Cần một run xanh trọn vẹn trước khi gộp.

### 3.3 Kết luận PR #23

**DUYỆT CÓ ĐIỀU KIỆN** — không có lỗi chặn trong mã. Điều kiện: CI `apple` xanh trọn vẹn trên commit mã cuối cùng.
Nên sửa MA3N-23-01 (cùng với sửa lõi) và MA3N-23-02 trong đợt 3.

## 4. Đối chiếu các chủ đề của DUYET-MA-DOT3

| Chủ đề (mã cũ) | Native |
|---|---|
| Chuỗi Android không giới hạn cửa sổ (MA3-14-01) | Đã sửa ở lõi; Android dùng đúng `next_at_unix_s` |
| O-9 lưu trong kho, mặc định tắt (MA3-14-02, -03) | Ba ứng dụng đều gọi biến thể `LvhvStore.*` và chỉ dùng `title`/`body`. Riêng Windows: chính việc ẩn tên (body rỗng) làm lộ MA3N-25-01 |
| Kiểm miền múi giờ (MA3-14-04) | Lõi kiểm; Android/Apple truyền lần chuyển; **Windows không** (MA3N-25-05) |
| Mật khẩu / bản rõ trong bộ nhớ (MA3-14-05) | Apple, Windows (bản rõ) đạt; Android giữ `VerifiedBackup` tới GC (MA3N-24-02); `restore-pw` của Windows (MA3N-25-09) |
| Từ chối trước khi đọc tệp lớn (MA3-15-02) | Sao lưu: cả ba kiểm kích thước trước khi đọc. **Lặp lại** ở ảnh Windows (MA3N-25-07) và ở hệ số bộ nhớ của Android (MA3N-24-03) |
| Hộp thoại đóng được khi đang chạy (MA3-15-03) | Không lặp lại: Windows chặn đóng khi `busy`, Apple `interactiveDismissDisabled`, Android khóa nút khi xuất. Nhưng chạy đồng bộ trên luồng giao diện (MA3N-24-01, -25-06, -23-02) là biến thể mới của cùng rủi ro |
| Khẳng định lỏng (MA3-15-04, MA3-14-08) | **Lặp lại**: MA3N-24-07 (3), MA3N-23-03 |
| Giao diện tự viết lại logic của lõi (MA3-15-05) | **Lặp lại trên cả ba**: thẻ nhắc hôm nay, xóa sự kiện mồ côi, xem trước .ics, kiểm độ dài mật khẩu, tháng nhuận (MA3N-24-08) |
| Ngày theo UTC (MA3-15-06) | Không lặp lại: cả ba đổi `last_backup_at` / `exported_at` sang ngày giờ máy |
| Ảnh nhiều điểm ảnh (MA3-15-07) | Không lặp lại: cả ba kiểm kích thước điểm ảnh / giải mã thu nhỏ |
| Tệp sinh ra làm nhiễu PR (MA3-15-08) | **Lặp lại** dưới dạng CI commit ảnh vào nhánh PR (MA3N-24-08, -25-09) |

## 5. Kết luận

| PR | Kết luận | Chặn | Nên sửa | Góp ý |
|---|---|---|---|---|
| #24 Android `34c9777` | **DUYỆT CÓ ĐIỀU KIỆN** | 0 | 5 (MA3N-24-01, -02, -03, -04, -07) | 3 (MA3N-24-05, -06, -08) |
| #25 Windows `5670e8a` (mã `8631ac2`) | **CHƯA DUYỆT** | 1 (MA3N-25-01) | 7 (MA3N-25-02 … -08) | 1 (MA3N-25-09) |
| #23 Apple `df4faa2` (mã `f25ec2d`) | **DUYỆT CÓ ĐIỀU KIỆN** | 0 | 2 (MA3N-23-01, -02) | 2 (MA3N-23-03, -04) |

Điều kiện gộp:
1. **PR #25:** sửa MA3N-25-01 (không so nội dung thông báo qua văn bản hiển thị; một `<text>` khi `body` rỗng; đọc
   được `<text/>`; thông báo không đọc được thì gỡ), có kiểm thử vòng đọc/ghi với `body` rỗng và kiểm thử "lập lịch
   hai lần không đổi gì" qua bản giả của Windows (MA3N-25-08); `--kiem-thong-bao` thử thêm `body` rỗng. Nên sửa
   MA3N-25-02 và -03 cùng lúc.
2. **PR #23:** một run CI `apple` xanh trọn vẹn trên commit mã cuối cùng.
3. **PR #24:** không có điều kiện chặn.
4. **Lõi (một PR riêng, trước khi sửa MA3N-23-01):** tách `export_backup` / `mark_backup_saved`; nên thêm các hàm của
   MA3N-24-08 (`due_today_json`, `lunar_leap_month`, `backup_password_error`, `IcsExport.preview`, xóa sự kiện mồ côi
   trong `delete_person`) và `backup_header_check` (MA3N-24-03) để ba ứng dụng bỏ phần tự viết.
5. Mục "nên sửa" làm trong đợt 3; mục nào để lại thì ghi lý do vào PR. R5 (máy thật, khởi động lại), P8, P9 vẫn chờ
   T11 — biên bản này không thay cho việc thử đó, và không thay cho bước duyệt 07/08 (R-PROC-1).
