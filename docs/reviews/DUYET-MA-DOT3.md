# Duyệt mã đợt 3 v0.2 — PR #14 (lõi, FFI) và PR #15 (PWA Người thân)

Trạng thái: **XONG** — 2026-11-07. Kết luận ở §3.

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

## 2. PR #15 — PWA mô-đun "Người thân"

### 2.1 Đã chạy

| Lệnh (trên `04b6e43`) | Kết quả |
|---|---|
| `npm run build` (pwa) | ĐẠT — 26 tệp, 812,9 KB chưa nén; lõi tsport, phương pháp jpl |
| `npm run typecheck` | ĐẠT |
| `node test/smoke.mjs` (gồm `test/people-flows.mjs`) | **ĐẠT** — 229 mục `ok`, mã thoát 0. Có các mục: "Người thân: 0 vi phạm CSP", "không gọi mạng ngoài (0)", "không có lỗi console (0)", "S9: ảnh lưu đã mã hóa lại, không còn đoạn APP1/APP13", "ảnh hiển thị bằng canvas, không có blob: trong trang (S2b)", "S4': localStorage chỉ có lvhv.ui" |
| CI của PR (check-runs trên `04b6e43`) | Mọi việc `success` |
| So bộ chuyển `pwa/src/people/period.ts` với `tsport/src/memorial_period.ts` của PR #14 trên `memorial/testdata/r13-linked.json`, kỳ "năm" 2026–2036, bộ lọc đã mất và đang sống | **24 mục bị xếp sai bộ lọc** (xem MA3-15-01) |
| Hiệu năng, 200 hồ sơ + 200 sự kiện, kỳ năm (Node, phần tính toán) | Bộ chuyển: 2,6 ms (385 mục); hàm lõi tsport: 3,6 ms (385 mục). Phần dựng DOM 385 mục chưa đo riêng |

Đúng đặc tả, đã kiểm và không có lỗi:
- **CSP (S2b) giữ nguyên**: PR không đổi `pwa/build.mjs`, `pwa/public/` hay `index.html`; `img-src 'self'`
  vẫn như cũ. Ảnh vẽ bằng `createImageBitmap(Blob)` lên `<canvas>` (`photo.ts:43-55`). `blob:` chỉ dùng cho
  `<a download>` (`backup.ts:12-20`), đúng 08 §8.3.
- **Không gọi mạng**: trong `pwa/src/people/` không có `fetch`, `XMLHttpRequest`, `WebSocket`, `innerHTML`, `eval`
  hay `new Function`. Mô-đun được nạp lười bằng `import()` (`main.ts`, `loadPeople`).
- **S4'**: dữ liệu chỉ nằm trong IndexedDB. Cờ "Đã hiểu" của cảnh báo lưu trữ ghi vào kho `meta`
  (`views.ts:42-58`), không ghi vào `localStorage`.
- **S9**: ảnh được giải mã, thu nhỏ còn ≤ 1024 px, vẽ lại lên canvas và mã hóa lại JPEG (`photo.ts:17-40`). Kiểm thử
  khói cài một đoạn APP1 có "GPS" vào ảnh mẫu và xác nhận byte lưu không còn đoạn đó.
- **S8**: mật mã chỉ đi qua `store/backup-crypto` (`@noble/ciphers`, `@noble/hashes`), không có mã mật mã nào
  trong `people/`.
- **S10**: xuất .ics có hộp xác nhận nêu đúng loại dữ liệu và số người (`backup.ts:45-55`); xuất sao lưu có hộp
  xác nhận, mật khẩu ≥ 8 ký tự theo NFC và phải nhập hai lần; "Thay toàn bộ" có thêm một hộp xác nhận nữa với tiêu
  điểm ở nút Hủy (`backup.ts:145-149`); xóa hồ sơ phải xác nhận (L13).
- Ngày: tháng nhuận (hộp "nhuận" chỉ hiện khi năm đó có tháng nhuận, L2), ngày không tồn tại (31/02, 15/5 nhuận
  2025), sinh 29/02 (`forms.ts:167-173`, 233), tuần qua năm, quý qua năm (L5) — đều có kiểm thử và đều đạt.
- O-9 không áp dụng cho PWA (không có thông báo hệ thống). Tiêu đề tab tưởng niệm không có tên người (kiểm ở
  `people-flows.mjs:297`).

### 2.2 Các lỗi

#### MA3-15-01 — **chặn** — bộ lọc của bộ chuyển tạm khác quy tắc §7.6 của lõi; kiểm thử L5 không phát hiện được vì lặp lại chính quy tắc sai

- Vị trí: `pwa/src/people/period.ts:48-62` (`filterDataset`: sự kiện được xếp theo `person_ids[0]`); bản sao của
  cùng quy tắc trong kiểm thử: `pwa/test/people-flows.mjs:201-206` (`expected`) và `:253-255` (R13 (c)); đặc tả
  `docs/v0.2/08-DAC-TA-GIAO-DIEN-NGUOI-THAN.md` §8.2 (dòng 619-622).
- Vấn đề: 02 §7.6 (PR #14) định nghĩa "đã mất" cho sự kiện là `person_ids` **không rỗng và mọi** id trỏ tới hồ sơ có
  ngày mất. Bộ chuyển chỉ nhìn người **đầu tiên**. Với sự kiện của nhiều người có trạng thái khác nhau, hai hàm cho
  kết quả khác nhau. Đã đo trên bộ `r13-linked.json`: 24 mục lệch trong 2026–2036. Ví dụ: sự kiện
  `d0000000-…-000000000061` ngày 28/02 có người đầu đã mất và người sau còn sống; bộ chuyển xếp nó vào "đã mất", còn
  lõi xếp vào "đang sống". Kiểm thử L5 tính "kết quả mong đợi" bằng **đúng quy tắc của bộ chuyển**, và bộ dữ liệu
  của kiểm thử chỉ có sự kiện một người, nên lỗi này không lộ ra.
- Kịch bản hỏng: người dùng khôi phục trên PWA một tệp sao lưu từ Android, có sự kiện "Ngày cưới" gắn
  `[Ông (đã mất), Bà (còn sống)]`. Ở mục "Theo kỳ" lọc "Người đã mất", PWA hiện ngày cưới; Android (dùng FFI → lõi
  Rust) thì không. Đây là sai lệch giữa các nền tảng (R-NAT-1), và mệnh đề R13(c) chỉ đạt trên PWA vì PWA kiểm bằng
  quy tắc của chính nó.
- **Nơi phải đổi**: tệp `pwa/src/people/period.ts`, ngay sau khi PR #14 được gộp vào `main`. Việc thay **không phải
  một dòng** như 08 §8.2 ghi, vì API khác nhau ở ba điểm:
  1. `tsport` có `prevPeriod`/`nextPeriod` trả `PeriodRange {from, to}`, còn bộ chuyển trả `Ymd`. Nơi gọi:
     `views.ts:185` và `:187` (lấy `.from`).
  2. `items` của lõi là `PeriodItem` (có `date`, `lunar`, `person_ids`, `name`, `occ`), còn bộ chuyển trả
     `Occurrence`. Nơi gọi: `views.ts:175-177` (`occItem(sn, it.occ, …)`, tháng nhóm theo `it.date.m`).
  3. Tên kiểu: lõi dùng `PersonFilter`, bộ chuyển dùng `PeriodFilter` (`views.ts:8`).
- Cách sửa:
  1. Gộp PR #14 trước. Sau đó viết lại `period.ts` thành lớp mỏng re-export từ `tsport/src/memorial_period.ts`,
     hoặc xóa hẳn tệp và import thẳng; rồi sửa ba nơi gọi trên.
  2. Sửa `people-flows.mjs:201-206` để `expected` gọi `eventsInPeriod` của `tsport`, không tự viết lại quy tắc. Thêm
     vào dữ liệu mẫu của L5 một sự kiện gắn `[người đã mất, người đang sống]` và khẳng định nó ở "Người đang sống".
  3. Sửa 08 §8.2 cho đúng §7.6 ("mọi người trong `person_ids` đã mất"). Dòng chính của mục (`ui.ts` `itemTitle`, theo
     `ownerOf` người đầu) có thể giữ, nhưng ghi rõ là chỉ dùng để hiển thị và không dùng cho bộ lọc.

#### MA3-15-02 — **chặn** — khôi phục sao lưu đọc toàn bộ tệp vào bộ nhớ trước khi kiểm kích thước (S12)

- Vị trí: `pwa/src/people/backup.ts:118-123` (`new Uint8Array(await f.arrayBuffer())` rồi mới `openBackup`); giới
  hạn nằm ở `pwa/src/store/backup-crypto.ts:21` (`MAX_PLAINTEXT = 128 MiB`) và `:70`, chỉ được kiểm **sau** khi đã có
  toàn bộ byte.
- Vấn đề: S12 (04) yêu cầu "từ chối trước khi cấp bộ nhớ lớn (bộ nhớ đỉnh < 200 MB); ứng dụng không treo, không
  thoát". Đường khôi phục của PWA, lối vào duy nhất của tệp sao lưu trên bản web, cấp bộ nhớ bằng đúng kích thước
  tệp người dùng chọn. Mọi tệp hợp lệ đều ≤ `MAX_PLAINTEXT` + phần đầu + thẻ xác thực, nên có thể từ chối sớm mà không
  cần đọc tệp.
- Kịch bản hỏng: người dùng chọn nhầm một tệp video 2 GB, hoặc nhận một tệp `.lvhv` độc hại dung lượng lớn. Trên
  điện thoại 3 GB RAM, tab bị hệ điều hành đóng giữa chừng và người dùng không thấy thông báo nào.
- Cách sửa: trước `f.arrayBuffer()`, thêm
  `if (f.size > MAX_PLAINTEXT + HEADER_LEN + TAG_LEN) { impErr.textContent = "Sai mật khẩu hoặc tệp bị hỏng"; … return; }`
  (thông báo chung theo 03 §2.4; xuất `HEADER_LEN`/`TAG_LEN` từ `store/backup-crypto`). Thêm kiểm thử khói: một
  `File` có `size` lớn hơn giới hạn (tạo bằng `new File([new Uint8Array(1)], …)` rồi ghi đè `size`, hoặc dùng một
  tệp thưa) → báo lỗi ngay và không gọi `arrayBuffer`.

#### MA3-15-03 — **nên sửa** — phím Esc đóng hộp thoại khi khôi phục hoặc xuất đang chạy; dữ liệu vẫn bị thay mà giao diện không báo

- Vị trí: `pwa/src/people/ui.ts:58-78` (`confirmDialog`: khóa hai nút trong lúc `onOk` chạy nhưng không chặn sự kiện
  `cancel` của `<dialog>`).
- Vấn đề: theo HTML, `<dialog>` mở bằng `showModal()` sẽ đóng khi nhấn Esc, trừ khi `cancel` bị `preventDefault()`.
  Nếu người dùng nhấn Esc trong lúc Argon2id và `applyImport` đang chạy (vài giây), hộp đóng với `result = false`
  nhưng `onOk` vẫn chạy tiếp: dữ liệu bị "Thay toàn bộ", còn trang không `rerender()` và không có thông báo.
- Kịch bản hỏng: người dùng chọn "Thay toàn bộ", bấm đồng ý, rồi đổi ý nhấn Esc vì nghĩ là đã hủy được. Dữ liệu cũ
  đã bị thay, nhưng màn hình vẫn hiện danh sách cũ cho tới khi tải lại trang.
- Cách sửa: trong `confirmDialog`, đặt cờ `busy` trong lúc chờ `onOk` và thêm
  `dlg.addEventListener("cancel", (e) => { if (busy) e.preventDefault(); })`. Thêm kiểm thử: nhấn Esc trong lúc
  khôi phục → hộp vẫn mở và cuối cùng có thông báo "Đã khôi phục".

#### MA3-15-04 — **nên sửa** — kiểm thử khói có khẳng định rỗng hoặc lỏng

- Vị trí: `pwa/test/people-flows.mjs:258` (`check(true, "L5: chạm một mục → mở hồ sơ")`); `:251`
  (`.pmonth count >= 3`); `:253-255` (R13 (c) kiểm trên tập "mong đợi" do chính kiểm thử tính, không kiểm trên giao
  diện).
- Vấn đề: dòng 258 luôn đạt, vì chỉ có `waitForSelector("#person-name")` đứng trước nó, nên mở **sai** hồ sơ vẫn
  qua. Dòng 251 vẫn qua khi nhóm tháng bị lặp hoặc thiếu. Khối 253-255 chỉ kiểm mã của kiểm thử.
- Cách sửa: dòng 258 đọc `#person-name` và so với tên của mục đầu tiên. Dòng 251 so bằng đúng số tháng phân biệt
  trong `expected("nam", …)`. Khối 253-255 dùng `shown()` (giao diện) của ba bộ lọc, sau khi đã sửa MA3-15-01.

#### MA3-15-05 — **góp ý** — thẻ nhắc trong trang tự viết lại logic chọn lời nhắc

- Vị trí: `pwa/src/people/views.ts:67-79` (`dueToday`).
- Vấn đề: 08 §8.1 nói giao diện không viết lại thuật toán. `dueToday` lặp lại một phần `reminders_for` (02 §7.1):
  so `offsets_days` với số ngày, và tự xử lý tiên thường. Hiện kết quả đúng (L7 đạt), nhưng nếu lõi đổi quy tắc, ví
  dụ thêm điều kiện cho tiên thường, thẻ nhắc của PWA sẽ không đổi theo.
- Cách sửa: dùng `M.allReminders(cal(), ds, t, addDays(t, 61), -Infinity, zone)` rồi lọc lời nhắc có ngày giờ
  địa phương = hôm nay. Như vậy PWA dùng cùng hàm mà parity R3 đã kiểm.

#### MA3-15-06 — **góp ý** — ngày "Lần sao lưu gần nhất" lấy theo UTC

- Vị trí: `pwa/src/people/backup.ts:65` (cắt chuỗi `last_backup_at` ở dạng `…Z`).
- Kịch bản: sao lưu lúc 06:30 sáng 25/07 giờ Việt Nam (23:30Z ngày 24/07) thì trang ghi "24/07". Cùng lỗi ở
  `:128` (`exported_at` trong xem trước khôi phục).
- Cách sửa: `const d = new Date(last); dmy({y: d.getFullYear(), m: d.getMonth() + 1, d: d.getDate()})`.

#### MA3-15-07 — **góp ý** — ảnh có kích thước điểm ảnh rất lớn được giải mã đầy đủ

- Vị trí: `pwa/src/people/photo.ts:17-21`.
- Vấn đề: giới hạn 25 MB tính theo dung lượng tệp. Một tệp PNG 20 000 × 20 000 nén còn vài MB vẫn qua được, và
  `createImageBitmap` giải mã ra khoảng 1,6 GB. Khi đó tab bị đóng (cùng tinh thần S12, cho ảnh).
- Cách sửa: đọc kích thước từ phần đầu tệp (IHDR của PNG, SOF của JPEG) và từ chối khi lớn hơn 12 000 px mỗi cạnh,
  hoặc lớn hơn 50 MP. Hoặc trên Chromium, gọi `createImageBitmap(file, {resizeWidth, resizeQuality})` sau khi đã biết
  kích thước.

#### MA3-15-08 — **góp ý** — bảo trì

- `pwa/test/metrics.json` (+121/−34 dòng) là tệp đo được ghi lại sau mỗi lần chạy, nên mỗi PR đều có thay đổi nhiễu ở
  tệp này. Nên chỉ commit khi cập nhật ngưỡng, hoặc tách số đo theo lần chạy ra khỏi git.
- `main.ts:201`: `indexedDB.databases` chưa có trên Firefox < 126, nên trên các bản đó thẻ nhắc ở Tờ lịch không bao
  giờ hiện, và không có thông báo nào. Có thể đánh dấu "đã dùng Người thân" trong `lvhv.ui` (localStorage đã được
  phép theo S4') để làm phương án dự phòng.
- `core.ts:46` (`weekdayOf`) và `period.ts:27` lặp lại `jd.weekday` của lõi. Khi xóa bộ chuyển (MA3-15-01), dùng
  luôn hàm của lõi.
- `backup.ts:72-73`, `:95`: sau khi xuất xong, mật khẩu vẫn còn trong hai ô nhập cho tới khi hộp bị gỡ. Nên đặt
  `pw1.value = pw2.value = ""` trong `onOk`, như cách `importFlow` đã làm ở `:126`.

### 2.3 Đánh giá rủi ro Rust ≡ TS của PR #15

Phần tính toán của PWA đi qua `tsport`, nên đã trùng byte với Rust theo parity. Có hai chỗ PWA tự tính và chưa có
bản lõi để so: bộ lọc theo kỳ (MA3-15-01, **đã lệch**) và `dueToday` (MA3-15-05, hiện chưa lệch). Sau khi sửa hai
mục này, không còn chỗ nào PWA tính khác lõi.

## 3. Kết luận

| PR | Kết luận | Chặn | Nên sửa | Góp ý |
|---|---|---|---|---|
| #14 `v0.2-w3-core` @ `181c039` | **DUYỆT CÓ ĐIỀU KIỆN** | 1 (MA3-14-01) | 3 (MA3-14-02, -03, -04) | 5 (MA3-14-05 … -09) |
| #15 `v0.2-w3-pwa` @ `04b6e43` | **DUYỆT CÓ ĐIỀU KIỆN** | 2 (MA3-15-01, -02) | 2 (MA3-15-03, -04) | 4 (MA3-15-05 … -08) |

Điều kiện gộp:
1. **PR #14**: sửa MA3-14-01 (chuỗi báo thức Android tìm `next` trên `reschedule_input`, không giới hạn 400 ngày) và
   thêm kiểm thử cho nó. Gộp PR #14 **trước** PR #15.
2. **PR #15**: sau khi PR #14 vào `main`, thay bộ chuyển `period.ts` bằng hàm `eventsInPeriod` của `tsport`, sửa ba
   nơi gọi ở `views.ts`, và cho kiểm thử L5 so với hàm của lõi, có thêm một sự kiện gắn nhiều người (MA3-15-01).
   Kiểm `f.size` trước khi đọc tệp khôi phục (MA3-15-02).
3. Mục "nên sửa" làm trong đợt 3. Riêng MA3-14-02 và MA3-14-04 làm trước khi T6–T8 bắt đầu dùng FFI. Nếu để lại mục
   nào thì ghi lý do vào PR.

Theo docs/08 §6 và 07 §6 R-PROC-1, hai PR vẫn chỉ được gộp sau khi 07 và 08 được duyệt. Biên bản này không thay cho
bước duyệt đó.
