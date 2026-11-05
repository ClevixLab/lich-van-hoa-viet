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
