# lvhv-ffi — lớp FFI của Lịch và Văn hóa Việt

Xuất lõi `lvhv-core` cho Kotlin (Android) và Swift (iOS/macOS) bằng **UniFFI 0.29.4** (macro thủ tục,
không có tệp UDL). Ứng dụng Windows (Rust + Slint) gọi thẳng các hàm Rust của crate này.

## Hàm xuất ra

| Rust | Kotlin / Swift | Trả về |
|---|---|---|
| `day_info_json(y,m,d)` | `dayInfoJson` | JSON DayInfo đúng hợp đồng §4 (thứ tự khóa cố định) |
| `solar_to_lunar(y,m,d)` | `solarToLunar` | `LunarDay {y,m,d,leap,month_len}` |
| `lunar_to_solar(ly,lm,ld,leap)` | `lunarToSolar` | `SolarDay?` (null nếu ngày âm không tồn tại / ngoài 2000–3000) |
| `month_lunar_days(y,m)` | `monthLunarDays` | âm lịch cả tháng dương (cho bảng tháng) |
| `find_good_days_json(act, from, to, birth_lunar_year?)` | `findGoodDaysJson` | mảng `GoodDay::to_json` của lõi: `[{"solar","lunar","weekday","jdn","score","allowed","reasons","first_gio_hoang_dao"}]` |
| `lunar_year_of_birth(birth)` | `lunarYearOfBirth` | năm ÂM lịch của ngày sinh dương 1900–3000 (sinh trước Tết → năm trước) |
| `activities_json()` | `activitiesJson` | JSON `[{"id","name"}]` (không lỗi) |
| `cadao_for(y,m,d)` | `cadaoFor` | JSON câu ca dao của ngày (quy tắc `content/cadao/select.md`) |
| `day_rating(score)` | `dayRating` | nhãn "Tốt / Khá tốt / Bình thường / Xấu" |
| `cadao_attribution()`, `cadao_sources_blocks_json()`, `ngochap_sources_blocks_json()` | | ghi công + danh mục nguồn cho màn hình Giới thiệu |
| `days_in_month`, `weekday`, `add_days`, `year_can_chi`, `lunar_method`, `build_info_json`, `cadao_count` | | tiện ích |

Lỗi: `LvhvError` (Kotlin `LvhvException.*`, Swift `LvhvError.*`) với thông điệp tiếng Việt hiển thị
được, đổi từ `lvhv_core::DateError` (`impl From<DateError>`): InvalidDate → `InvalidDate`, OutOfRange →
`OutOfRange`, InvertedRange/RangeTooLong → `BadRange`, UnknownActivity → `UnknownActivity`,
Computation → `Unavailable`. Mọi kiểm tra miền do lõi làm; không hàm nào panic với đầu vào sai.

`day_info_json` và `find_good_days_json` trả **đúng từng byte** JSON của lõi (test
`day_info_is_byte_identical_to_core`, `find_good_days_is_core_output`).

**Năm sinh:** `find_good_days_json` nhận năm ÂM lịch. Ứng dụng native hiện cho người dùng nhập thẳng
"năm sinh âm lịch" (kèm giải thích: sinh trước Tết thì lấy năm trước). Có ngày sinh đầy đủ thì đổi bằng
`lunar_year_of_birth`.

## Phần v0.2 — hồ sơ, ngày giỗ, nhắc, sao lưu (`src/v02.rs`)

Cho giao diện native đợt 3 (docs/v0.2/05 §2). Ít hàm, mỗi hàm làm trọn một việc; dữ liệu vào/ra là **JSON** đúng
lược đồ `docs/v0.2/schema/backup-v1.schema.json` (Person, Event, EventType, AppSettings, tài liệu sao lưu). Lỗi:
`LvhvDataError` (Kotlin `LvhvDataException.*`, Swift `LvhvDataError.*`) — `Invalid`, `NotFound`, `Storage`,
`Backup`; thông điệp tiếng Việt. Thời điểm: giây Unix (UTC); giờ đồng hồ thiết bị: `fire_local` dạng
`YYYY-MM-DDTHH:MM`. Múi giờ thiết bị: `DeviceZone {offset_min, transitions:[{at_unix_s, offset_min}]}`
(Việt Nam: `{420, []}`). Lịch: phương pháp mặc định của lõi (`jpl`).

| Rust | Việc |
|---|---|
| `LvhvStore::open(dir)` | Mở/tạo kho SQLite (`lvhv.sqlite`, `photos/`) trong thư mục riêng của ứng dụng (01 §4.2) |
| `settings_json()`, `put_settings_json(json)` | `AppSettings` |
| `mark_deceased_json(person_id, death_date_value_json, notice_json?, today)` | Cập nhật người đang sống thành đã mất trên **cùng** hồ sơ (01 §3.2a, US-M-07): thêm `death` + thông tin báo tử tùy chọn, `kind = deceased`, tăng `rev`, bỏ quy tắc nhắc sinh nhật; theo 02 §6.1 (B4) sinh nhật và sự kiện của người này không còn lần xuất hiện |
| `persons_json()`, `person_json(id)`, `save_person_json(json)`, `delete_person(id)` | Hồ sơ. `save_person_json` tự tính trường dẫn xuất (02 §1, §4), gắn id/`created_at`/`updated_at`/`rev`, kiểm V1–V8 |
| `delete_person_and_orphan_events(id)` → id sự kiện đã xóa | Xóa người và, cùng giao dịch, các sự kiện chỉ gắn người đó (sự kiện vốn không gắn ai thì giữ) |
| `events_json()`, `save_event_json(json)`, `delete_event(id)` | Sự kiện |
| `event_types_json()`, `save_event_type_json(json)` | Loại sự kiện (tên duy nhất — V7) |
| `add_photo(id, mime, width, height, bytes)`, `photo_bytes(id)` | Ảnh chân dung (đã mã hóa lại, không EXIF) |
| `add_attachment(id, mime, file_name, bytes)`, `attachment_bytes(id)`, `delete_attachment(id)` | Tệp đính kèm báo tử (01 §3.8a, YC-2026-07): ảnh đã mã hóa lại (≤ 2 MB, cạnh dài ≤ 2048 px) hoặc PDF nguyên bản (≤ 10 MB, `%PDF-`). Gắn bằng `death.notice.attachments: [{id, kind, file_id}]`; gỡ khỏi danh sách rồi lưu → tự xóa. `delete_attachment` chỉ cho tệp chưa gắn |
| `danh_xung_suggestions(deceased, gioi_tinh)`, `quan_he_suggestions(deceased)`, `display_capitalize(text)` | Gợi ý cách gọi / quan hệ dùng chung (YC-2026-07); `save_person_json` tự viết hoa chữ đầu `danh_xung`, `quan_he` và bỏ `gioi_tinh: null`; nhập sao lưu giữ nguyên, nên giao diện hiển thị `danh_xung`/`quan_he` qua `display_capitalize` |
| `LvhvStore::migration_status()` → `MigrationStatus {state, from_version, to_version, backup_path, message}` | Nâng cấp dữ liệu khi mở kho (01 §7.1, 04 B5/R14). `state`: `ok` / `migrated` / `read_only`. Khi `read_only`: hiện thông báo `message`, chỉ cho xem và xuất sao lưu — mọi thao tác ghi báo `Invalid` |
| `delete_photo(id)` | Xóa ảnh chưa gắn hồ sơ nào (đã `add_photo` rồi hủy). Ảnh hồ sơ bỏ khi đổi/xóa ảnh thì `save_person_json` đã tự xóa. Ảnh còn dùng → `Invalid` |
| `dataset_json()` | Toàn bộ kho dạng tài liệu sao lưu — đầu vào `dataset_json` của các hàm dưới |
| `backup_reminder_due()`, `export_backup(password, app_version, platform)`, `apply_import(backup, mode)` | Sao lưu (03); `mode` = `Replace` \| `Merge`. `export_backup` chỉ tạo byte, **không** đổi `last_backup_at` |
| `mark_backup_saved(saved_at_unix_s)` | Gọi một lần, chỉ khi hệ điều hành báo lưu tệp xong (03 §2.4). Hủy hoặc lỗi ghi thì không gọi. Chưa tạo tệp → `Invalid` |
| `backup_password_error(password, confirm)` → `null` \| câu lỗi | Quy tắc mật khẩu (03 §2.4); ô khôi phục: `confirm = null` |
| `backup_header_check(head, file_len)` | Kiểm 72 byte đầu và kích thước tệp trước khi đọc cả tệp (03 §2.2); lỗi → `Backup(<câu>)` |
| `open_backup(file, password)` → `VerifiedBackup` (`preview()`) | Giải mã + kiểm tra toàn bộ, chưa ghi gì; lỗi → "Sai mật khẩu hoặc tệp bị hỏng" |
| `derive_person_json(json)`, `save_death_json(date_value)`, `derive_birth_json(date_value)` | Tính/kiểm trường dẫn xuất không ghi (xem trước trong màn hình nhập) |
| `person_summary_json(person, today)` | Tên hiển thị, Can Chi, tuổi dương/tuổi âm (02 §4.3), "Hưởng thọ … tuổi" (02 §4.2), ngày mất âm lịch |
| `upcoming_json(dataset, today, days)` | Danh sách "Sắp tới" (02 §7.2) |
| `period_bounds(kind, anchor)`, `period_prev`, `period_next`, `events_in_period_json(dataset, kind, anchor, filter)` | Danh sách theo kỳ (02 §7.6, R13) |
| `reminders_json(dataset, now, zone, horizon_days, show_names = false)` | Lời nhắc (02 §7.1), văn bản màn hình khóa theo O-9 |
| `due_today_json(dataset, today, zone)`, `LvhvStore::due_today_json(today, zone)` | Thẻ nhắc trong ứng dụng hôm nay (02 §7.5): như `reminders_json`, tiêu đề có tên, sắp theo giờ |
| `reschedule_json(dataset, now, zone, pending_json, show_names = false)`, `sentinel_info() -> SentinelInfo {id, text}` | Xoay vòng 64 yêu cầu chờ cho iOS/macOS/Windows (02 §7.4) |
| `android_alarm_step_json(dataset, now, last_run, zone, show_names = false)` | Một bước chuỗi báo thức Android (02 §7.3): lời nhắc cần đăng ngay + báo thức kế tiếp (không giới hạn cửa sổ) |
| `LvhvStore::lock_screen_names()`, `set_lock_screen_names(bool)` | Tùy chọn O-9 "hiện tên trên màn hình khóa", mặc định `false`, lưu cục bộ trong kho (bảng `meta`), không vào sao lưu |
| `LvhvStore::reminders_json(now, zone, horizon)`, `reschedule_json(now, zone, pending)`, `android_alarm_step_json(now, last_run, zone)`, `upcoming_json(today, days)`, `events_in_period_json(kind, anchor, filter)` | Như các hàm tự do nhưng đọc dữ liệu **và tùy chọn O-9** từ kho — ứng dụng native **nên dùng các hàm này** |
| `ics_export(dataset, today, now, only_ids)` → `IcsExport {file_name, content, events, persons, summary}` | Tệp .ics 10 năm (02 §8); `summary` cho hộp xác nhận S10 |
| `ics_summary(dataset, today, only_ids)` → `IcsSummary {events, persons, from, to, kinds, privacy_text, preview}` | Hộp xác nhận S10 mà không sinh tệp: câu riêng tư, tối đa 5 dòng xem trước |
| `lunar_leap_month(year)` → tháng nhuận \| `null` | Năm âm 1899–3000, cho ô "nhuận" (01 §6.1) |
| `new_uuid()` | UUID v4 (01 §1) |

Lược đồ đầu ra: `docs/v0.2/schema/ffi-v02-output.schema.json` (mọi đối tượng đóng; kiểm thử `output_matches_schema`
so đầu ra thật của từng hàm với lược đồ). Đổi hay thêm trường phải sửa lược đồ cùng lúc.

Thông tin báo tử (`death.notice`, 01 §3.2a): chỉ có trong dữ liệu hồ sơ (`person_json`, `persons_json`,
`dataset_json`) và tệp sao lưu mã hóa; **không** có trong lời nhắc, thông báo, .ics, tên tệp, danh sách hay
`person_summary_json` (kiểm thử `notice_never_leaks` gieo chuỗi canh vào mọi trường và dò mọi đầu ra). Xóa hồ sơ xóa
luôn ảnh giấy báo tử. Chuỗi được chuẩn hóa NFC, cắt khoảng trắng; rỗng → `null`.

O-9: khi không hiện tên, phần tử lời nhắc chỉ có `occ = {entity_id, class, date, key}` (không có `name`,
`headline`, `date_line`); **nội dung thông báo chỉ lấy từ `title`/`body`**.

Kiểm dữ liệu vào: năm ngoài 1900–3000, giây Unix ngoài 1900–3000 (ví dụ truyền mili giây), độ lệch múi giờ có
|giá trị| > 1080 phút (ví dụ truyền giây) hoặc lần chuyển giờ không tăng dần → `LvhvDataError.Invalid`.

Mật khẩu sao lưu: phía Rust xóa bản sao `String` nhận qua FFI (`Zeroizing`) sau khi dùng. `VerifiedBackup` giữ bản
rõ đã giải mã tới khi đối tượng bị hủy: Kotlin dùng trong `use { }`, Swift giải phóng ngay sau `apply_import`.

Kotlin: `[bindings.kotlin] android = true` trong `uniffi.toml` để mã dọn đối tượng không dùng
`java.lang.ref.Cleaner` (API 33) với `minSdk 26`.

## Dữ liệu nhúng

`include_str!` của `content/cadao/cadao.json`, `content/cadao/SOURCES.md`, `content/ngochap/SOURCES.md`.
Sửa các tệp này thì phải dựng lại thư viện.

## Phát hiện cần A3 (Ngọc Hạp) xem

`find_good_days` của lõi chỉ loại ngày `allowed = false` (phạm sao kỵ chính). Nó **vẫn trả** những ngày mà
tờ lịch của chính ngày đó ghi việc ấy vào mục "Kỵ" (vì có sao xấu nêu đích danh việc + điểm âm), và
những ngày điểm âm. Đo 2026–2029, 16 việc, không năm sinh: 11 707 ngày trả về, **616** ngày bị tờ lịch
ghi Kỵ cho chính việc đó, 2 273 ngày điểm âm
(`cargo run -p lvhv-ffi --release --example gd_check`). Ví dụ 2026-11-12 "Cưới hỏi": điểm −1, tờ lịch
ghi Kỵ Cưới hỏi nhưng vẫn có trong danh sách ngày tốt. Người dùng sẽ thấy mâu thuẫn. FFI giữ đúng đầu
ra của lõi (không tự lọc) — cần A3/chủ dự án quyết định quy tắc.

## Quy ước chung với PWA (kiểm bằng test)

1. **Nhãn "Đánh giá"** (`day_rating`): ≥ 4 Tốt, 1..3 Khá tốt, −3..0 Bình thường, ≤ −4 Xấu — giống
   `rating()` của `pwa/src/format.ts`. Đây là quy ước của ứng dụng, chưa được A3 khóa; đổi ở một nơi thì
   phải đổi cả hai (test `rating_thresholds_match_pwa`).
2. **Ca dao trong ngày**: theo `content/cadao/select.md` (`order == (JDN − 2451545) mod N`), giống
   `cadaoFor` của `pwa/src/engine.ts` (test `cadao_select_md_example`: 2026-11-06 → order 412).

## Sinh mã Kotlin/Swift

```bash
cargo build -p lvhv-ffi --release
cargo build -p lvhv-ffi --features bindgen-cli --bin uniffi-bindgen
target/debug/uniffi-bindgen generate --library target/release/liblvhv_ffi.so \
  --language kotlin --out-dir ffi/generated/kotlin --config ffi/uniffi.toml --no-format
target/debug/uniffi-bindgen generate --library target/release/liblvhv_ffi.so \
  --language swift  --out-dir ffi/generated/swift  --config ffi/uniffi.toml --no-format
```

`ffi/generated/` chỉ để đọc/đối chiếu; bản dựng Android/Apple tự sinh lại mã (`build-rust.sh`,
`build-xcframework.sh`), CI cảnh báo nếu bản commit đã cũ.

## `unsafe`

Mã viết tay ở đây không có `unsafe`. Macro UniFFI sinh các hàm `extern "C"` có `unsafe` bên trong — đó
là mã của thư viện UniFFI. Tiêu chí S1 áp dụng cho `lvhv-core` (`#![forbid(unsafe_code)]`), không áp
dụng cho lớp FFI.
