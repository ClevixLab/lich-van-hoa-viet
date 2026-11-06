# Duyệt mã — Lịch và Văn hóa Việt v0.1

Vai: A10 (duyệt mã), người duyệt độc lập. Ngày: 2026-11-06.
Phạm vi: `core/`, `ngochap/`, `tsport/`, `pwa/` ở trạng thái lúc duyệt (bộ tích hợp đang sửa đồng thời;
các tệp `core/src/gooddays.rs`, `tsport/src/gooddays.ts`, `tsport/src/ngochap*.ts`, `pwa/build.mjs`, `ffi/`
xuất hiện hoặc đổi trong lúc duyệt và đã được đọc lại). Phát hiện bảo mật chi tiết: `RA-SOAT-BAO-MAT.md` (RS-xx).

## 1. Đánh giá chung

Điểm mạnh thực sự:
- Lõi nhỏ, không phụ thuộc ngoài (trừ crate nội bộ), `#![forbid(unsafe_code)]`, công thức có trích nguồn, bộ
  tuần tự hóa JSON viết tay có thứ tự khóa xác định.
- Hai phương pháp (`Hnd`/`Precise`) tách sạch qua `Method`, mặc định đổi bằng một hằng, có kiểm tra tự nhất quán
  khóa danh sách vi phạm đã biết (`core/tests/methods.rs`) — trung thực về khuyết điểm của HND.
- tsport chép rất sát (giữ thứ tự phép tính dấu phẩy động, tự cài `rround` theo ngữ nghĩa `f64::round`). Người
  duyệt chạy lại: 0 sai lệch byte 2000–2100 ở cả hai phương pháp, kể cả `ngoc_hap`; `find_good_days` 314/314 dòng.
- Bảng Ngọc Hạp của TS **được sinh từ Rust** (`ngochap/src/tests.rs::emit_ts_tables`) chứ không chép tay —
  đúng hướng giảm trôi dạt.
- PWA: không `innerHTML` dữ liệu, CSP chặt, SW đơn giản, `localStorage` bọc try/catch và danh sách trắng.

Điểm yếu chính: **lõi không bảo vệ hợp đồng phạm vi của chính nó** (RS-01/02/03/04/05), kiểm tra trôi dạt
Rust↔TS **chưa tự động**, và một số tài liệu/bằng chứng đã cũ so với mã.

## 2. Rủi ro tính đúng

| # | Mức | Vị trí | Vấn đề | Đề xuất |
|---|---|---|---|---|
| C-1 | Cao | `core/src/lunar.rs:81–86, 120–212`; `tsport/src/lunar.ts` | Treo / panic / kết quả vô nghĩa ngoài khoảng an toàn (RS-01, RS-02). Phạm vi 2000–3000 chỉ được thực thi ở UI | Lõi kiểm phạm vi, trả `Option`/`Result` |
| C-2 | Trung bình | `pwa/src/ui/gooddays.ts:73–102`; `ngochap/src/lib.rs:254–259` | Ô "Năm sinh" nhận **năm dương** rồi đổi Can Chi theo công thức **năm âm** `(Y+6) mod 10`. Người sinh trước Tết (tháng 1–2 dương) có tuổi Can Chi của năm trước → xung tuổi sai, ngày "không cho phép" sai. Hiển thị "Tuổi Bính Thìn" cho 1976 dù người sinh 15/01/1976 là Ất Mão (`solarToLunar(1976,1,15)` → năm âm 1975, Ất Mão). Kiểm chứng: `findGoodDays("cuoi_hoi", 2026)` với 1976 → 71 ngày, với 1975 → 74 ngày; khác nhau 8 ngày (ví dụ 2026-02-26 chỉ có ở 1976, 2026-02-07 chỉ có ở 1975) | Đổi nhãn thành "Năm sinh âm lịch", hoặc nhận ngày sinh dương rồi đổi bằng `solar_to_lunar` |
| C-3 | Trung bình | `core/src/dayinfo.rs:190–209` | Ép `u32 → u8`, chú thích "mọi chỉ số < 60" sai cho `lunar_day` ngoài phạm vi (RS-05) | `u8::try_from` + kiểm |
| C-4 | Trung bình | `ngochap/src/lib.rs:167–212` | Mọi chỉ số bị `clamp`/`%` lặng lẽ: tháng 0 = tháng 1, tháng 13 = tháng 12; can/chi lệch chẵn lẻ → `xung_tuoi` rỗng. Lỗi tích hợp (truyền sai trường) sẽ cho kết quả **trông hợp lệ** | `DayCtx::validate()`; trong test, `debug_assert!` |
| C-5 | Trung bình | `core/src/canchi.rs` (`pub fn` không kiểm miền: `thanh_long_day_chi(0)`, `month_can(_,0)`, `hour_range(>11)`, `can_chi_name(≥10,_)`) | Tràn/wrap ở release, panic ở debug (RS-04) | `pub(crate)` hoặc kiểm miền; `overflow-checks = true` |
| C-6 | Thấp | `core/src/lunar.rs:52` vs `:189` | `sun_sector(Precise)` dùng `.clamp(0,11)` sau `floor` — nếu `apparent_sun_longitude` trả đúng 360.0 do làm tròn thì cung 12 bị kẹp về 11 thay vì 0. `norm_deg` dùng `%` nên `x = 360 - ε` cộng hiệu chỉnh có thể ra 360.0. Xác suất rất thấp nhưng sai về hướng (11 thay vì 0) | `rem_euclid(12)` thay `clamp` |
| C-7 | Thấp | `core/src/terms.rs` `term_index_of_day`: `.min(23)` | Cùng mẫu như C-6: kinh độ 360.0 → 24 → kẹp 23 thay vì 0 | `% 24` |
| C-8 | Thấp | `core/src/jd.rs:96–113` `jd_to_datetime` | NaN → trả `(-4713,11,24,0,0,0)` lặng lẽ (`NaN as i64 = 0`); `inf` → ngày vô nghĩa. Chỉ dùng trong công cụ dump | Kiểm `is_finite()` |
| C-9 | Thấp | `core/src/astro.rs:321–340` `solve_sun_longitude` | NaN vào → NaN ra sau 30 vòng, không báo; `solar_terms_of_year(i32::MAX)` → rỗng lặng lẽ (release) | Trả `Option` hoặc `debug_assert!(t.is_finite())` |
| C-10 | Thấp | `pwa/build.mjs:112` | `engineSource = existsSync(TSPORT) ? "tsport" : "mock"` — thiếu tsport thì **lặng lẽ** dựng bản giả lập; UI có nhãn MOCK nhưng một bản phát hành có thể mang dữ liệu giả | `LVHV_ALLOW_MOCK=1` mới cho phép mock; mặc định `fail()` |
| C-11 | Thấp | `pwa/src/format.ts:10–29`, `pwa/src/engine.ts` `cadaoFor` | JDN và Can Chi năm được **cài lại lần thứ ba** (ngoài Rust và tsport), với bảng can/chi xoay vòng khác cách (bắt đầu "Canh"/"Thân"). Hiện đúng (đã thử 1976 → Bính Thìn, 2026 → Bính Ngọ), nhưng là nguồn trôi dạt | Dùng `tsport` (`jd`, `canchi`) qua `engine.ts` |
| C-12 | Thông tin | `core/src/lunar.rs` `YearTable::lunar_year` | Điều kiện `month >= 11 && i < 4` dựa vào giả định vị trí; đúng với 2000–3000 (khứ hồi đã kiểm), không có chú thích chứng minh | Ghi chú lý do `i < 4` |

## 3. Độ đầy đủ của kiểm thử so với tiêu chí

| Tiêu chí | Kiểm thử hiện có | Đánh giá |
|---|---|---|
| L5 khứ hồi | `fixtures.rs::roundtrip_2000_2100` (chạy mặc định), `roundtrip_2000_3000` `#[ignore]`; `methods.rs::precise_roundtrip_2000_3000` `#[ignore]` | Đủ nếu CI chạy `--ignored` ở bản release. Hiện không có CI → bằng chứng chỉ là lời khai trong docs/03 |
| L7 Rust↔TS | `tsport/test/compare.mjs` so với tệp vàng `core/out/…jsonl.gz` | Đúng hướng; tệp vàng phải sinh lại mỗi lần đổi lõi (RS-12). Người duyệt sinh lại 2000–2100: 0 sai lệch |
| N3 | `ngochap/src/tests.rs::n3_major_taboo_never_allowed`, `n3_holds_for_any_than_value`; `core/tests/ngochap.rs::n3_all_days_2000_3000`, `n3_find_good_days_2000_3000` (`#[ignore]`) | Tốt về thiết kế (kiểm cả qua `find_good_days`). Lưu ý: N3 kiểm tra **sự nhất quán** giữa `major` và `allowed` — tức là kiểm cài đặt so với chính bảng; không kiểm bảng `major` có đúng nguồn (đó là N1/N2) |
| N1/N2 | `content/ngochap/rules.md`, `SOURCES.md` | Không thuộc phạm vi duyệt mã; không có kiểm tra tự động liên kết mã quy tắc ↔ rules.md (ví dụ mọi `id` trong `tables.rs` có dòng trong rules.md) — **nên thêm**, rẻ |
| S1/S2 | Không có kiểm tra tự động | Thêm vào CI: `grep`/`#![forbid]` có sẵn; `cargo tree -p lvhv-core --prefix none` phải chỉ gồm crate `path` |
| S3/S4 | `pwa/test/smoke.mjs` (đếm yêu cầu ngoài, khóa localStorage) | Tốt. Nhưng `metrics.json` hiện có đo trên bản mock (RS-16) |
| Đầu vào xấu | `fixtures.rs::invalid_lunar_dates` (tháng 0/13, ngày 0/31, nhuận không tồn tại) | **Thiếu**: không có test cho năm ngoài phạm vi, JDN cực trị, `solar_to_lunar` ngày sai, NaN — đó là nơi người duyệt tìm thấy lỗi |
| P1/P2 | `lvhv-dump bench`; docs/03 §13 | §13 đo **trước** khi `day_info` gọi Ngọc Hạp; cần đo lại. Người duyệt: `find_good_days` 90 ngày = 0,69 ms (P2 đạt xa) |

Đề xuất thêm (ít công): một test tính chất trong `core/tests/` gọi API công khai với năm ∈ {−10⁶, 0, 1999, 3001,
10⁶, i32::MIN, i32::MAX} và yêu cầu: không treo (kết thúc < 1 s), trả lỗi/None rõ ràng.

## 4. Trùng lặp Rust ↔ TS và cơ chế chặn trôi dạt

Hiện trạng: tsport là bản **chép tay** của `core/src/{jd,astro,lunar,terms,canchi,dayinfo,gooddays}.rs` và logic
`ngochap/src/lib.rs`; bảng Ngọc Hạp sinh tự động. Hằng `DEFAULT_METHOD` được `build.mjs:92–104` đối chiếu bằng
regex giữa `lunar.rs` và `lunar.ts` — tốt nhưng chỉ cho một hằng.

Rủi ro trôi dạt cụ thể quan sát được trong lúc duyệt: các tệp Rust và TS được sửa ở các thời điểm khác nhau
(ví dụ `core/src/canchi.rs` 02:49:46, `tsport/src/canchi.ts` 02:55:24; tên thần thứ 6 đổi "Thiên Đức" →
"Kim Đường" ở cả ba nơi). Không có gì ngoài kỷ luật của tác nhân bảo đảm hai bên đổi cùng lúc.

**Cơ chế đề xuất (bắt buộc trước G4):** một tác vụ CI duy nhất, chạy trên mỗi thay đổi:
1. `cargo build --release -p lvhv-core --bin lvhv-dump`
2. Sinh mới (không dùng tệp đã commit) vào thư mục tạm:
   `lvhv-dump dayinfo-jsonl 2000-01-01 3000-12-31`, cùng với `--method precise`; `tet`, và một tệp
   `gooddays` mẫu (≥ 3 việc × có/không năm sinh × 1 năm).
3. Build tsport, sinh cùng các dòng bằng Node; `cmp` từng byte (hoặc so băm sha256 của cả tệp).
4. Chạy lại `emit_ts_tables` và `git diff --exit-code tsport/src/ngochap_tables.ts` (bảng TS phải khớp Rust).
5. Thất bại nếu bất kỳ sai lệch nào.

Lâu dài: thay tsport bằng `lvhv-core` biên dịch sang WebAssembly (hợp đồng §1 đã ghi "tạm thời, đến khi có
WebAssembly qua CI") — loại bỏ hẳn trùng lặp. Lưu ý: khi đó cần kiểm lại S3 (`script-src` cần
`'wasm-unsafe-eval'`).

## 5. Tính nhất quán API

| # | Vấn đề |
|---|---|
| A-1 | Hợp đồng §4: `solar_to_lunar -> LunarDate` (panic khi sai), `lunar_to_solar -> Option` — hai chiều xử lý lỗi khác nhau. `find_good_days`: ngày sai → panic, mã việc sai → `Vec` rỗng, `from > to` → rỗng: ba kiểu lỗi, hai cách báo |
| A-2 | Tên trường: Rust `LunarDate { year, month, day }`, `SolarDate { year, month, day }`; JSON và TS `{ y, m, d }`. `pwa/src/types.ts LunarDate` thiếu `month_len`. Chấp nhận được nhưng cần ghi trong hợp đồng |
| A-3 | `tsport` xuất `dayInfo(y,m,d, method?)` (tham số cuối tùy chọn) **và** `…With(method, …)` — hai kiểu chọn phương pháp; Rust chỉ có `_with`. Chọn một |
| A-4 | `pwa/src/engine.ts` dò hàm theo cả camelCase và snake_case (`need("dayInfo","day_info")`) — linh hoạt cho giai đoạn mock nhưng che lỗi đổi tên; nay tsport đã ổn định, nên gọi trực tiếp, để `tsc` bắt lỗi kiểu |
| A-5 | `lvhv_ngochap::NgocHap.sao_tot: Vec<&'static str>` nhưng `lvhv_core::NgocHap.sao_tot: Vec<String>` — hai kiểu cùng tên `NgocHap` ở hai crate, chuyển đổi bằng tay ở `dayinfo.rs:211–222` |
| A-6 | `GoodDay` (Rust) có `allowed` "luôn true trong kết quả" — trường thừa trong JSON công khai |

## 6. Xử lý lỗi

- Rust: `assert!`/`expect` trong API công khai (RS-03); không có kiểu lỗi miền. `ffi` định nghĩa `LvhvError`
  riêng và kiểm tra trước — đúng, nhưng nên dời xuống lõi để một nguồn sự thật.
- TS: `throw new Error(<tiếng Việt>)` — thông điệp đôi khi sai bản chất (`tet(2026.5)` → "tháng Giêng luôn tồn
  tại"; `jdnToLunar(NaN)` → "bảng tháng bất thường"); `dayInfoJson(-1e6,…)` → `TypeError` nội bộ (RS-06).
- PWA: `main.ts` bọc lời gọi đầu tiên `dayInfo` trong try/catch rồi vẫn `render()` — nếu lõi ném, `renderSheet`
  sẽ ném lại không được bắt → màn hình trống, không có thông báo cho người dùng. Nên có ranh giới lỗi hiển thị
  "Không tính được ngày này".

## 7. Độ chính xác tài liệu (docs/03 so với mã)

| Mục docs/03 | Khớp mã? | Ghi chú |
|---|---|---|
| §0 "Lịch dương … kiểm tra khứ hồi 1990–3010" | Khớp `jd.rs::roundtrip_range` | — |
| §0 / hợp đồng: "UTC+7 cho 2000–3000" | Khớp ý định; **mã không thực thi** phạm vi | Thêm câu: "ngoài 2000–3000 hành vi không xác định / trả lỗi" sau khi sửa RS-01 |
| §1–§5, §7–§10 công thức | Khớp mã (người duyệt so công thức Trực, tú, thần, giờ HĐ, can tháng, giờ Tý với `canchi.rs`; các test đơn vị khớp các ví dụ) | — |
| §6 Newton "dừng khi bước < 1e-8 ngày" | Khớp, nhưng mã còn giới hạn 30 vòng và **không báo** khi không hội tụ | Ghi rõ |
| §9 tên thần thứ 6 | Đã cập nhật "Kim Đường" (dòng 152–160) khớp `canchi.rs:34`, `canchi.ts:15`, `tables.rs:1405` | Cần chủ dự án duyệt (doc tự ghi) |
| §12 `ngoc_hap` | Đã cập nhật theo tích hợp | — |
| §13 hiệu năng | **Cũ**: đo trước khi `day_info` gọi `lvhv_ngochap::evaluate` | Đo lại |
| Hợp đồng §1 "core/ … không phụ thuộc thư viện ngoài"; `core/Cargo.toml` description | Vẫn đúng theo nghĩa "không crate bên ngoài kho", nhưng lõi **nay phụ thuộc `lvhv-ngochap`**. Hợp đồng §5 cấp quyền ghi `core/src/ngochap*` cho A3, tức dự kiến Ngọc Hạp **nằm trong** core, không phải crate riêng | Ghi rõ trong hợp đồng: "phụ thuộc nội bộ (path) được phép" — cần chủ dự án duyệt vì tiêu chí S2 diễn đạt là "0" |
| `content/ngochap/SCORING.md` §3 ví dụ 2026-11-05 điểm **−6** | Không kiểm lại được trong đợt này với dữ liệu hiện tại (tên thần đổi, có thể đổi danh sách) | Kiểm lại bằng `lvhv-dump` |
| `pwa/src/ui/about.ts` "Năm sinh … chỉ dùng trong lúc tính, không được lưu lại" | Khớp (RS-17) | — |

## 8. Dễ đọc

- Rust: rõ, chú thích tiếng Việt tốt, tên hàm nhất quán. `lvhv-dump.rs` dài nhưng thẳng.
- `pwa/build.mjs`: nhiều trách nhiệm (đọc/kiểm nội dung, phân tích Markdown bằng regex, kiểm hằng Rust bằng regex,
  bundle, SW) trong 224 dòng; phần phân tích `SOURCES.md` bằng regex mong manh (đổi tiêu đề mục là hỏng). Tách
  kiểm nội dung ra script riêng có test.
- `pwa/src/ui/gooddays.ts`: thụt lề lệch trong `addItems` (dòng ~115–130 bản trước); hàm `run()` dài.
- `tsport/src/astro.ts` chú thích "No FMA contraction happens in either language" — đúng với rustc mặc định và
  V8, nhưng là giả định nền tảng; byte-compare trong CI là cách duy nhất giữ nó.

## 9. Danh sách việc

### Bắt buộc sửa trước G4

1. **Lõi thực thi phạm vi năm** và không treo/panic với đầu vào xấu — `core/src/lunar.rs` (`k_of_month_start`,
   `YearTable::new`, `jdn_to_lunar_with`, `solar_to_lunar_with`), `core/src/lib.rs` (`tet_with`),
   `core/src/dayinfo.rs`, `core/src/gooddays.rs`; tương ứng trong `tsport/src/lunar.ts`, `dayinfo.ts`,
   `gooddays.ts`. Thêm test đầu vào xấu (§3). (RS-01, RS-02, RS-03, RS-13)
2. **CI chặn trôi dạt Rust↔TS** theo §4 (sinh mới, so byte, `emit_ts_tables` + `git diff --exit-code`), chạy cả
   các test `#[ignore]` ở bản release. Không có nó thì L7 và N3 chỉ là bằng chứng một lần.
3. **`overflow-checks = true`** ở `[profile.release]` (gốc `Cargo.toml`) và sửa các phép trừ `u32` không kiểm
   trong `canchi.rs` (C-5), `ngochap_ctx` dùng `try_from` (C-3). (RS-04, RS-05)
4. **Ngữ nghĩa "Năm sinh"** (C-2): sửa nhãn hoặc cách đổi — đây là sai tính đúng mà người dùng thấy được, trực
   tiếp ảnh hưởng kết quả "allowed".
5. **Đo lại P1–P5 trên bản build cuối** (không phải bản mock) và cập nhật docs/03 §13 + `pwa/test/metrics.json`.
   (RS-16)
6. **Quyết định origin triển khai** (RS-10) và **LICENSE + đổi tên phông trong bảng `name`** (RS-20) — hai việc
   pháp lý/triển khai, rẻ, nên chốt trước khi URL được chia sẻ.
7. `build.mjs` không được lặng lẽ dựng bản mock (C-10).

### Nên làm (không chặn G4)

- Luồng cập nhật SW có thông báo, bỏ `skipWaiting` tự động (RS-08); kịch bản "công tắc hủy".
- `require-trusted-types-for 'script'` trong CSP; bỏ `img-src data:` nếu không dùng (RS-09).
- `DayCtx::validate()` trong `lvhv-ngochap` thay cho kẹp lặng lẽ (C-4).
- `rem_euclid` thay `clamp/min` ở cung/tiết khí (C-6, C-7).
- Hợp nhất xử lý lỗi thành `Result<_, LvhvError>` ở lõi; `ffi` chỉ chuyển tiếp (A-1, §6).
- Gỡ bộ dò tên hàm trong `engine.ts`, gọi tsport trực tiếp (A-4); bỏ cài lại JDN/Can Chi trong `format.ts` (C-11).
- Test liên kết `tables.rs` id ↔ `rules.md` (§3, N1).
- Khử trùng lặp URL nguồn trong gói (RS-19); lockfile/README cho tsport (RS-15).
- Ranh giới lỗi hiển thị trong PWA (§6).
- Ghi rõ trong hợp đồng: phụ thuộc nội bộ `lvhv-ngochap` của lõi (§7).
