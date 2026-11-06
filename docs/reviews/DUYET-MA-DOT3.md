# Duyệt mã đợt 3 v0.2 — PR #14 (lõi, FFI) và PR #15 (PWA Người thân)

Trạng thái: **ĐANG DUYỆT** — đã xong phần PR #14; phần PR #15 sẽ được bổ sung.

Bên duyệt mã: một vai không viết mã của hai PR này (docs/08 §6). Bên duyệt mã không sửa mã đang duyệt; mọi
đề xuất sửa ghi trong biên bản này.

| PR | Nhánh | Đầu nhánh được duyệt | So với |
|---|---|---|---|
| #14 | `v0.2-w3-core` | `181c039` | `origin/main` (`git diff origin/main...origin/v0.2-w3-core`) |
| #15 | `v0.2-w3-pwa` | `04b6e43` | `origin/main` (`git diff origin/main...origin/v0.2-w3-pwa`) |

Căn cứ: docs/01, docs/08 §6, docs/v0.2/01–04 (04 gồm Bổ sung B2 và B3), 06 (O-9), 07; docs/v0.2/08 trên nhánh PWA.

Mức lỗi: **chặn** (phải sửa trước khi gộp) / **nên sửa** (sửa trong đợt này hoặc ghi rõ lý do để lại) /
**góp ý**. Mã lỗi: `MA3-<PR>-<số>`.

## 1. PR #14 — lõi R12/R13, FFI cho native

### 1.1 Đã chạy

| Lệnh (trên `181c039`) | Kết quả |
|---|---|
| `cargo test --workspace --locked` | ĐẠT, 0 lỗi (gồm `memorial/tests/period.rs`, `exhaustive.rs`, `ffi/src/tests_v02.rs`) |
| `scripts/check-parity.sh --memorial-only` | **ĐẠT: 0 sai lệch** — gồm r12 `--steps`, r13 trên bộ R3, bộ `r13-linked`, `--method precise`, tệp ví dụ; `reminders --global-off/--rule-off` = 0 lời nhắc; `memorial_unit` 43/43 |
| CI của PR (check-runs trên `181c039`) | Mọi việc `success` |
| Thăm dò biên FFI (crate thử riêng ngoài kho, gọi mọi hàm `#[uniffi::export]` mới dưới `catch_unwind`, bật `overflow-checks`) | **0 panic**: năm 1, 1899, 3001, 9999, −5, `i32::MAX`; giây Unix `i64::MIN`/`i64::MAX`; độ lệch múi giờ `i32::MAX`; JSON lồng 100 000 cấp; `pending_json` với `at_unix_s` cực đại |
| Hiệu năng, 200 hồ sơ + 200 sự kiện (bản phát hành, máy 2 vCPU) | `events_in_period_json` năm/tất cả: 10,6 ms (385 mục); năm/đã mất: 6,5 ms; `reschedule_json`: 7,1 ms; `android_alarm_step_json`: 5,2 ms |

Đúng đặc tả, đã kiểm và không có lỗi:
- `period_bounds`: tuần Thứ Hai → Chủ nhật qua cuối năm (2025-12-29 … 2026-01-04), tháng 2 năm nhuận (2028-02-29),
  quý, năm; `prev`/`next` liền nhau mọi ngày 2026–2036 (`bounds_and_navigation`); số kỳ 575/132/44/11 khớp A.6.
- Tiên thường của giỗ rơi vào ngày đầu kỳ sau thuộc kỳ trước (`occurrences_with_tt` quét tới `to + 1`).
- Bộ lọc §7.6 ở lõi đúng nguyên văn: "đã mất" chỉ khi `person_ids` không rỗng và **mọi** id trỏ tới hồ sơ có ngày
  mất; id không trỏ tới hồ sơ nào → "đang sống" (`memorial/src/period.rs:137-150`, bản TS
  `tsport/src/memorial_period.ts:70-77` tương đương). Phân hoạch kiểm ở `r13c_on`.
- R12/B3: sentinel cùng phút với cái thứ k, `k = 0` → `now + 400 ngày`; I4 được đếm riêng; tạo `reschedule_input`
  quét `+400+500` ngày và sự kiện một lần tới 3000-12-31 — khớp §7.4 Bổ sung B3. Rust và TS khớp từng dòng.
- `#![forbid(unsafe_code)]` vẫn có ở `core`, `ngochap`, `memorial`, `store`, `backup`. `ffi` không có `unsafe` viết
  tay (mã `extern "C"` do macro UniFFI sinh, như đã ghi ở `ffi/src/lib.rs:6-8`).
- Lỗi qua FFI: mọi hàm mới trả `Result<_, LvhvDataError>`; khóa `Mutex` bị nhiễm độc được lấy lại
  (`v02.rs:256-259`); không thấy `unwrap`/`expect` trên dữ liệu vào.
- O-9: `notification_text(show_names = false)` cho tiêu đề "Sắp đến ngày giỗ" / "Sắp đến sinh nhật" / "Sắp đến sự
  kiện", dòng phụ rỗng (02 §7.1 Bổ sung B2); sentinel không có tên.
- `uniffi.toml` `android = true`: mã Kotlin sinh ra `import android.os.Build` và `androidx.annotation.RequiresApi`,
  chọn `SystemCleaner` khi API ≥ 34, JNA khi thấp hơn. `androidx.annotation` có sẵn qua `androidx.core:core-ktx`
  (`apps/android/app/build.gradle.kts:160`); kiểm thử JVM của ứng dụng (`app/src/test`) không nạp mã UniFFI. **Không
  phá gì** trên ứng dụng Android hiện có. Hệ quả cần biết: tệp Kotlin này không còn dùng được trên JVM thuần (ngoài
  Android).

### 1.2 Các lỗi

#### MA3-14-01 — **chặn** — chuỗi báo thức Android bỏ lặng lẽ lời nhắc ở xa hơn 400 ngày

- Vị trí: `ffi/src/v02.rs:815` (`to = today + 400 + 61`) và `ffi/src/v02.rs:823` (`r.at <= now + WINDOW_DAYS * 1440`).
- Vấn đề: `next_at_unix_s` chỉ lấy lời nhắc trong 400 ngày tới. Khi lời nhắc sớm nhất ở xa hơn, hàm trả
  `next_at_unix_s: null`, nên ứng dụng không đặt báo thức nào và chuỗi dừng hẳn. 02 §7.3 yêu cầu "đặt báo thức kế
  tiếp" cho lời nhắc sớm nhất, không có cửa sổ. Bổ sung B3 vừa sửa đúng lỗi này cho iOS (I4: "không bỏ lặng lẽ lời
  nhắc ở xa"), nhưng đường Android chưa được sửa.
- Đã tái hiện: bộ dữ liệu chỉ có một sự kiện một lần ngày 2028-06-01 (nhắc 0 và 1 ngày, 07:00), `now` =
  2026-05-28 → `android_alarm_step_json` trả `{"dropped_late":0,"next_at_unix_s":null,"next_fire_local":null,"post":[]}`.
  Cùng dữ liệu đó, `reschedule_json` (iOS) đặt sentinel đúng như B3.
- Kịch bản hỏng: một người dùng Android ghi "Lễ mừng thọ" (sự kiện một lần) vào năm sau nữa, rồi chỉ dùng ứng dụng
  qua thông báo. Nếu máy không khởi động lại và ứng dụng không được mở trong khoảng đó, không có thông báo nào. Tình
  huống này khó gặp hơn trên iOS vì Android còn đặt lại chuỗi khi khởi động máy, nhưng hậu quả giống hệt: mất lời
  nhắc mà không có báo hiệu nào.
- Cách sửa: lấy danh sách từ `reschedule_input(cal(), &ds, now, &tz)` (đã quét `+900` ngày và sự kiện một lần tới
  3000) thay cho `all_reminders(.., to = today + 461)` khi tìm `next`, và bỏ điều kiện `r.at <= now + 400 ngày`
  (`setAndAllowWhileIdle` nhận mọi thời điểm tương lai). Phần `post` giữ như cũ. Thêm kiểm thử ở
  `ffi/src/tests_v02.rs`: sự kiện một lần cách 2 năm → `next_at_unix_s` bằng `at` của lời nhắc đó. Ghi rõ trong
  02 §7.3 rằng `next` không có cửa sổ.

#### MA3-14-02 — **nên sửa** — O-9 phụ thuộc vào việc mỗi ứng dụng tự truyền đúng `show_names`, không có mặc định và không có nơi lưu

- Vị trí: `ffi/src/v02.rs:699-705`, `739-745`, `796-802` (tham số `show_names: bool` bắt buộc); lược đồ
  `AppSettings` (`docs/v0.2/schema/backup-v1.schema.json`) không có trường nào cho tùy chọn này.
- Vấn đề: 06 O-9 khóa mặc định là **không hiện tên** trên màn hình khóa. FFI để mỗi nền tảng tự lưu tùy chọn và tự
  truyền vào ba hàm khác nhau. Ba ứng dụng native (T6–T8) sẽ lưu ba cách khác nhau, không qua kho và không có trong
  sao lưu. Chỉ cần một chỗ gọi truyền `true`, ví dụ vì cài đặt chưa nạp kịp, là tên người hiện ra trên màn hình khóa.
  Đây đúng là rủi ro R-NAT-1 (07 §6).
- Kịch bản hỏng: trên Android, `BootReceiver` chạy `android_alarm_step_json` trước khi `SharedPreferences` của cài
  đặt "ẩn tên" được đọc, và lập trình viên truyền `true` theo mặc định của biến Kotlin. Thông báo "Còn 3 ngày: Giỗ đầu
  Ông Nguyễn Văn A" hiện lên màn hình khóa.
- Cách sửa: (a) thêm `#[uniffi(default = false)]` cho cả ba tham số, để mặc định của FFI khớp O-9; và (b) ghi tùy
  chọn vào kho, tốt nhất là trường `AppSettings.lock_screen_names: bool` (mặc định `false`; cần sửa lược đồ và 01
  §3.9, nên phải được chủ dự án duyệt). Nếu chưa sửa lược đồ, đặt tùy chọn này trong một bảng cài đặt cục bộ của
  `lvhv-store`, để các hàm nhắc đọc từ kho thay vì nhận từ người gọi. Thêm kiểm thử: gọi với mặc định → tiêu đề
  không chứa `ho_ten`.

#### MA3-14-03 — **nên sửa** — `reminder_json` luôn mang tên người trong `occ`, kể cả khi `show_names = false`

- Vị trí: `ffi/src/v02.rs:670-681` (`"occ": occ_json(ds, &r.occ)` có `name`, `headline`, `date_line`).
- Vấn đề: khi ẩn tên, `title` và `body` đã sạch, nhưng mỗi phần tử của `post` (Android) và `reminders_json` vẫn mang
  `occ.name` / `occ.headline`. Lập trình viên Android dễ dùng `occ.headline` cho `setContentTitle` hoặc cho
  `setPublicVersion`. Nếu vậy, O-9 bị bỏ qua mà không kiểm thử nào phát hiện.
- Kịch bản hỏng: thông báo mở rộng (big text) dùng `occ.headline` → tên hiện trên màn hình khóa khi chế độ hiển thị
  của Android là "hiện mọi nội dung".
- Cách sửa: khi `show_names = false`, chỉ trả `occ` gồm `entity_id`, `class`, `date`, `key` (đủ để mở đúng màn hình
  khi người dùng chạm vào thông báo), không có `name`, `headline`, `date_line`. Ghi rõ trong tài liệu hàm: "chỉ dùng
  `title`/`body` cho nội dung thông báo".

#### MA3-14-04 — **nên sửa** — không kiểm miền của dữ liệu vào: năm ngoài 1900–3000, độ lệch múi giờ, thứ tự lần chuyển giờ

- Vị trí: `ffi/src/v02.rs:87-92` (`ymd` chỉ kiểm ngày có tồn tại); `ffi/src/v02.rs:138-147` (`zone` nhận mọi
  `offset_min`, `transitions` không cần tăng dần).
- Vấn đề, đã tái hiện:
  - `period_bounds("tuan", 2147483647-12-31)` trả `to = -2147483648-01-05` (năm bị tràn). `events_in_period_json`
    năm 1, năm −5 và năm 9999 trả `Ok` thay vì báo lỗi.
  - `DeviceZone{offset_min: 25200}` (người gọi truyền **giây** thay vì phút): `fire_local` vẫn là `07:00`, nhưng
    `at_unix_s` lệch đi 17,5 ngày. Android dùng `at_unix_s` cho `setAndAllowWhileIdle` nên báo thức đến sai ngày, và
    không có lỗi nào được báo.
  - `transitions` không tăng dần vẫn được nhận và cho kết quả.
- Kịch bản hỏng: phía Kotlin truyền `TimeZone.getDefault().rawOffset / 1000` (giây). Mọi lời nhắc trên Android đến
  sai ngày, trong khi kiểm thử iOS vẫn đúng vì iOS dùng `fire_local`. Hai nền tảng lệch nhau (R-NAT-1).
- Cách sửa: `ymd()` trả `Invalid` khi `y ∉ [1900, 3000]` (miền của lõi, L13). Thêm `fn zone(z) -> R<Zone>`: kiểm
  `|offset_min| ≤ 1080` cho `initial` và cho mọi lần chuyển, và kiểm `at_unix_s` tăng nghiêm ngặt. Nếu sai, trả
  `Invalid("Múi giờ không hợp lệ (độ lệch tính bằng phút).")`. Thêm kiểm thử cho ba trường hợp trên.

#### MA3-14-05 — **góp ý** — mật khẩu sao lưu đi qua FFI dưới dạng `String` không được xóa

- Vị trí: `ffi/src/v02.rs:201-205` (`open_backup`), `ffi/src/v02.rs:402-415` (`export_backup`).
- Vấn đề: `lvhv-backup` xóa bản chuẩn hóa NFC của mật khẩu (`normalize_password` → `Zeroizing`), nhưng `String` mà
  UniFFI chép sang vẫn nằm trong vùng nhớ đã giải phóng sau khi hàm trả về. Bản bên Kotlin/Swift thì không xóa được,
  nên lợi ích có giới hạn. Dù vậy, phía Rust nên làm đúng phần việc của mình.
- Cách sửa: dòng đầu mỗi hàm thêm `let password = zeroize::Zeroizing::new(password);` (thêm `zeroize` vào
  `ffi/Cargo.toml`, crate đã có trong cây phụ thuộc và nằm trong danh sách cho phép của S8). Ghi trong README rằng
  `VerifiedBackup` giữ bản rõ cho tới khi đối tượng bị hủy, nên phía Kotlin gọi trong `use { }` và phía Swift giải
  phóng ngay sau `apply_import`.

#### MA3-14-06 — **góp ý** — sentinel "không đặt được cái nào" có thể kêu lúc nửa đêm

- Vị trí: `memorial/src/remind.rs` `select_keep` (`at = hi`), và tương ứng trong `tsport/src/memorial_remind.ts`.
- Vấn đề: khi không có lời nhắc nào trong cửa sổ, sentinel đặt đúng `now + 400 ngày` tính tới từng phút. Theo thăm
  dò, gọi lúc 03:26 thì sentinel kêu lúc 03:26 (`fire_local: 2027-07-03T03:26`). Cách làm này đúng 02 §7.4 Bổ sung B3,
  nhưng người dùng sẽ khó chịu.
- Cách sửa (cần sửa đặc tả trước, rồi sửa Rust và TS cùng lúc để giữ parity): đặt tại 09:00 giờ thiết bị của ngày
  `now + 399` (vẫn ≤ `hi` nên I4 vẫn đúng).

#### MA3-14-07 — **góp ý** — thiết kế API UniFFI: đủ thô, nhưng hợp đồng JSON đầu ra chưa có lược đồ

- Vị trí: `ffi/README.md` (bảng hàm), các chú thích `///` ở `v02.rs:578-581`, `633-636`, `695-697`, `731-737`, `788-794`.
- Nhận xét: mức thô của API là hợp lý. Mỗi màn hình cần một lời gọi, mọi tính toán nằm ở lõi (R-NAT-1), không có
  callback và không có đối tượng sống lâu ngoài `LvhvStore`. Còn ba điểm:
  1. Lược đồ chính thức chỉ có cho **đầu vào** (backup-v1). Các **đầu ra** (`upcoming_json`, `events_in_period_json`,
     `reminders_json`, `reschedule_json`, `android_alarm_step_json`, `person_summary_json`) chỉ được mô tả trong chú
     thích, và các trường thêm (`class_text`, `headline`, `date_line`, `days_text`) không được ghi ở 02 Phụ lục A.6.
     Chưa có gì ngăn việc đổi tên trường.
  2. `sentinel_info() -> Vec<String>` trả mảng theo vị trí. Nên dùng một `uniffi::Record {id, text}`.
  3. Mỗi lời gọi danh sách đều phải `dataset_json()` rồi đưa lại vào, nghĩa là tuần tự hóa hai lần (với 200 hồ sơ
     chỉ tốn vài ms, nên không phải lỗi hiệu năng).
- Cách sửa: thêm `docs/v0.2/schema/ffi-v02-output.schema.json` (hoặc một mục ở 02), cùng một kiểm thử vàng so khóa
  JSON của từng hàm với lược đồ đó. Đổi `sentinel_info` thành Record trước khi T6–T8 bắt đầu dùng. Cân nhắc thêm
  `LvhvStore::upcoming(today, days)` và các hàm tương tự, đọc thẳng từ kho.

#### MA3-14-08 — **góp ý** — một số khẳng định trong `ffi/src/tests_v02.rs` quá lỏng

- Vị trí: `ffi/src/tests_v02.rs:262` (`keep.len() <= 64 && !keep.is_empty()`), `:302` (`next > …`), `:321`
  (`dropped_late >= 1`), `:344` (`persons >= 2`).
- Vấn đề: các khẳng định này vẫn qua nếu số lượng sai, ví dụ đặt 1 thay vì 40 lời nhắc, hoặc `next` là lời nhắc thứ
  hai thay vì thứ nhất. Dòng 311 có so `next` với `all[0]`, nên dòng 302 thừa; các dòng còn lại thì không có kiểm
  tra nào bù.
- Cách sửa: so bằng giá trị chính xác lấy từ bộ dữ liệu ví dụ (đếm trước bằng `lvhv-memorial-dump`), ví dụ
  `assert_eq!(keep.len(), N)`, `assert_eq!(f.persons, 3)`, `assert_eq!(late["dropped_late"], K)`.

#### MA3-14-09 — **góp ý** — việc làm thừa và nuốt lỗi nhỏ trong `v02.rs`

- `ffi/src/v02.rs:648` cùng `557-558`: `events_in_period_json` gọi `period_item` hai lần cho mỗi mục (một lần trong
  `events_in_period`, một lần trong `occ_json`). Nên dùng `it` có sẵn.
- `ffi/src/v02.rs:325-327`, `350-352`: `st.events().ok()?` nuốt lỗi SQLite. Khi đọc lỗi, bản ghi cũ bị coi là chưa
  có, nên được ghi đè với `rev = 1` và `created_at = bây giờ`, làm hỏng quy tắc gộp theo `rev` (01 §6.3). Nên trả
  lỗi (`?`) thay vì `.ok()?`.
- `ffi/src/v02.rs:488-495`: `person_summary_json` nhận `birth.solar` sai miền (`m = 0`, `d = 99`) và vẫn trả
  `tuoi_duong: 26`. Dữ liệu từ kho đã qua V1–V8 nên không xảy ra trong thực tế, nhưng nên `ymd()` trước rồi trả
  `null` khi sai.

### 1.3 Đánh giá rủi ro Rust ≡ TS của PR #14

Thấp. Parity theo byte phủ r12/r13 trên ba bộ dữ liệu và hai phương pháp. Các điểm còn ngoài vùng parity: (a) mọi
hàm ở `ffi/src/v02.rs` chỉ có bản Rust, nên phần ghép JSON của FFI không có bản TS để so (PWA gọi thẳng `tsport`),
và hai giao diện có thể lệch ở các trường hiển thị `class_text`/`days_text` (`v02.rs:547-555`). (b) `once_reminders_after`
dùng `from > end` (Rust) và `from.y > 3000` (TS); hai cách tương đương trong miền hiện tại.

### 1.4 Kết luận PR #14

**DUYỆT CÓ ĐIỀU KIỆN** — gộp được sau khi sửa **MA3-14-01** (chặn) và có kiểm thử cho nó. MA3-14-02 và MA3-14-04
nên sửa trước khi T6–T8 bắt đầu viết mã gọi FFI, vì đổi chữ ký về sau sẽ tốn hơn.
