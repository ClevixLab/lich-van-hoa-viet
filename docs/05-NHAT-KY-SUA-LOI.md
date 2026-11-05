# Nhật ký sửa lỗi sau duyệt độc lập — v0.1

Ngày: 2026-11-06. Đầu vào: `docs/reviews/{RA-SOAT-BAO-MAT.md, DUYET-MA.md, BIEN-BAN-S1-S4.md,
MO-HINH-DE-DOA.md}` và hai yêu cầu bổ sung của bên điều phối (§B). Mỗi dòng: mã phát hiện → thay đổi →
bằng chứng (lệnh có thể chạy lại). Kho chưa có commit; bằng chứng ứng với thư mục làm việc lúc ghi.
Máy đo: Linux x86_64, 2 vCPU (sandbox), rustc/cargo 1.97.0, Node v22.23.4, Chromium 153.

Trạng thái: **ĐÃ SỬA** / **MỘT PHẦN** / **ĐỂ LẠI** (kèm lý do) / **CHỦ DỰ ÁN QUYẾT**.

## A. Kiểm tra chung (chạy cuối đợt)

```
cargo fmt --check -p lvhv-core -p lvhv-ngochap                       → sạch
cargo clippy --workspace --all-targets --offline -- -D warnings      → 0 cảnh báo (kể cả ffi)
cargo test --workspace --release                                      → mọi test thường: ok
cargo test --release -p lvhv-core -- --ignored                        → khứ hồi 2000–3000 (Hnd, Precise),
    tự nhất quán Hnd/Precise, N3 toàn dải, §2a toàn dải: ok (xem §B.1)
scripts/check-parity.sh                                               → ĐẠT: 0 sai lệch (≈ 1 phút 40 giây)
cd pwa && npm run typecheck && npm run build && node test/smoke.mjs   → ĐẠT (69/69 kiểm tra)
node tsport/test/bad_input.mjs                                        → 765 passed, 0 failed
```

**Lệnh chặn trôi dạt Rust ↔ TS (cho CI):** `scripts/check-parity.sh` (tùy chọn `--method hnd|precise`,
`KEEP=1` để giữ thư mục tạm). Script build `lvhv-dump` release, sinh **mới** JSONL `day_info` mọi ngày
2000-01-01 … 3000-12-31 + `tet` 2000…3000 + mẫu `gooddays` (16 việc × 2026 × không tuổi/năm âm 1976) cho
**cả hai** phương pháp vào thư mục tạm, build + typecheck tsport, chạy test đầu vào xấu của tsport, so
từng byte (`LVHV_GOLDEN_DIR=<tạm> node tsport/test/compare.mjs`), rồi sinh lại
`tsport/src/ngochap_tables.ts` từ Rust và `cmp` với tệp đang có. Thoát ≠ 0 nếu có bất kỳ sai lệch,
tệp thiếu, hay lỗi build nào. Cần `pwa/node_modules` đã cài. Đã thử âm tính: đổi một tên thần trong
`tsport/src/canchi.ts` → `compare.mjs` báo 5 568 dòng lệch, mã thoát 1.

Kết quả lần chạy cuối:
```
[hnd]     dayinfo: lines=365608 expected=365608 mismatches=0   tet: rows=1001 mismatches=0   gooddays: rust=4470 ts=4470 mismatches=0
[precise] dayinfo: lines=365608 expected=365608 mismatches=0   tet: rows=1001 mismatches=0   gooddays: rust=4470 ts=4470 mismatches=0
[parity] ĐẠT: 0 sai lệch
```
Tệp vàng `core/out/` đã sinh lại. `dayinfo_2000_3000.jsonl.gz`, `tet.csv`, `lunar_2000_3000.csv`,
`newmoons.csv`, `terms_2000_3000.csv` (cả `precise/`) **giống hệt từng byte** bản trước khi sửa (so
`cmp` sau khi giải nén) — sửa lỗi miền không đổi một kết quả nào trong 2000–3000. Chỉ
`gooddays_sample.jsonl` đổi (5 625 → 4 470 dòng mỗi phương pháp) do quy tắc §2a (§B.1).

## B. Yêu cầu bổ sung của bên điều phối

### B.1 Cao — Danh sách chọn ngày mâu thuẫn tờ lịch — ĐÃ SỬA

**Thay đổi.** `core/src/gooddays.rs`, `tsport/src/gooddays.ts`: một ngày chỉ được đề xuất khi
(a) `allowed` (không kỵ chính, không xung tuổi với việc cá nhân), (b) điểm việc `≥ MIN_GOOD_DAY_SCORE = 0`,
(c) việc **không** nằm trong `ky` của tờ lịch ngày đó (kiểm tường minh qua `lvhv_ngochap::evaluate`;
trường mới `GoodDay.in_sheet_ky`, không xuất JSON). `GoodDay::is_suggestable()` / `isSuggestable()`.
Quy tắc ghi ở `content/ngochap/SCORING.md` §2a. PWA ghi "điểm việc", "Cách tính điểm cho việc …",
và giải thích ở đầu kết quả rằng điểm việc khác điểm chung dùng cho nhãn tờ lịch.

**Bằng chứng — trước/sau**, 2026-01-01 … 2029-12-31, 16 việc, không tuổi:

| | Ngày đề xuất | Tờ lịch ghi Kỵ chính việc đó | Điểm âm |
|---|---|---|---|
| Trước (`lvhv-dump gooddays`) | 11 671 | 613 | 2 270 |
| Trước (bên điều phối, qua ffi `gd_check`) | 11 707 | 616 | 2 273 |
| Sau (`lvhv-dump gooddays`) | 9 401 | **0** | **0** |
| Sau (`cargo run -p lvhv-ffi --release --example gd_check`) | 9 434 | **0** | **0** |

Số ngày đề xuất mỗi năm (trung vị 2026–2029; không tuổi / tuổi năm âm 1975), cho thấy danh sách không rỗng:

| Việc | Không tuổi | Tuổi 1975 | | Việc | Không tuổi | Tuổi 1975 |
|---|---|---|---|---|---|---|
| cuoi_hoi | 55 | 53,5 | | giao_dich | 147 | 139,5 |
| xay_dung | 52,5 | 50 | | cau_tai | 110 | 103,5 |
| khai_truong | 89 | 84,5 | | to_tung | 206,5 | 196,5 |
| an_tang | 114,5 | 111,5 | | lam_phuc | 211,5 | 205 |
| te_tu | 186 | 180 | | nhap_trach | 109 | 103 |
| dong_tho | 94,5 | 89,5 | | khai_nghiep | 208 | 195 |
| xuat_hanh | 142,5 | 133,5 | | cau_tu | 208 | 195 |
| | | | | nhap_hoc / cau_suc_khoe | 208 / 212 | 195 / 199 |

(Trước khi sửa: cuoi_hoi 76, xay_dung 104,5, … — giảm 15–50 %. Khoảng 90 ngày "3 tháng tới" cho cưới hỏi
vẫn có 18 ngày — ví dụ 06/11/2026 … 03/02/2027.)

**Test thường trực.** `core/tests/ngochap.rs::suggested_days_consistent_with_sheet_2026` (mặc định, cả
hai phương pháp, có/không tuổi) và `::suggested_days_consistent_with_sheet_2000_3000` (`--ignored`). Lần
chạy toàn dải (release, 568 s): `§2a Hnd: 2 346 064 ngày đề xuất 2000–3000, 0 vi phạm`;
`§2a Precise: 2 346 397 …, 0 vi phạm`. Mỗi ngày được đề xuất có việc ∉ `ky` của tờ lịch, điểm ≥ 0 và
không kỵ chính. N3 qua `find_good_days` toàn dải: chạy lại trong cùng đợt (`n3_*`, xem §A).

### B.2 `cargo fmt` — ĐÃ SỬA
`cargo fmt -p lvhv-core -p lvhv-ngochap`; `cargo fmt --check -p lvhv-core -p lvhv-ngochap` sạch. `ffi/`
không sửa.

## C. Bảng phát hiện → thay đổi → bằng chứng

### Cao

| Mã | Trạng thái | Thay đổi | Bằng chứng |
|---|---|---|---|
| **RS-01** (treo vô hạn) / **C-1** | ĐÃ SỬA | `core/src/error.rs` mới: miền công khai 2000-01-01 … 3000-12-31 (`MIN_YEAR/MAX_YEAR/MIN_JDN/MAX_JDN`), miền nội bộ `YearTable` 1900–3101. Mọi hàm công khai kiểm miền **trước** khi tính. `k_of_month_start` có cận 4 bước mỗi chiều → `Err(Computation)` thay vì lặp mãi. tsport: `tsport/src/error.ts` + cùng kiểm tra | `core/tests/bad_input.rs` (mỗi lời gọi trong luồng riêng, hạn 5 s — treo = test thất bại): `i32::MIN/MAX`, 0, −1, 1, 1999, 3001, 35 650, ±10⁶, 100 000; JDN `i64::MIN`, ±10⁹, `i64::MAX`, biên ±1 → `Err(OutOfRange)`. `tsport/test/bad_input.mjs` (worker, hạn 5 s): như trên + `1e9`, `2**53`, `2**53−1` |
| **RS-02** (sai lặng lẽ / panic) | ĐÃ SỬA | Bỏ mọi `assert!`/`expect` trong đường công khai: `YearTable::new → Result`, kiểm 12–13 tháng, tháng 29/30 ngày, năm 13 tháng có nhuận; `jdn_to_lunar` hậu kiểm `1 ≤ day ≤ month_len ∈ {29,30}`; `i -= 1` thay bằng tìm có cận. `lunar_to_solar` trả `None` khi kết quả ngoài miền. `tet`, `solar_terms_of_year` trả `Result` | `bad_input.rs::{solar_years_out_of_range_are_errors, jdn_extremes_are_errors, edges_of_domain_work, year_table_internal_domain}`; tệp vàng 2000–3000 giống hệt từng byte (§A) |

**Ngữ nghĩa lỗi (Rust ↔ TS, đã ghi trong `core/src/lib.rs`, `tsport/src/index.ts`, `tsport/src/error.ts`).**
Rust `Err(DateError::X)` ⇔ TS ném `LvhvError` (lớp con của `RangeError`) có `code === "X"`, thông điệp
giống hệt `Display` của Rust; Rust `None` ⇔ TS `null`. Mã: `InvalidDate`, `OutOfRange`, `InvertedRange`,
`RangeTooLong`, `UnknownActivity`, `Computation`; riêng TS thêm `InvalidArgument` cho đầu vào mà kiểu
của Rust không cho phép (JDN/năm không nguyên, NaN, phương pháp lạ). Ngày dương sai (kể cả NaN, ±∞, số
không nguyên, chuỗi) → `InvalidDate` ở cả hai bên — được kiểm **trước** miền.

**API công khai đã đổi** (`ffi/` đã được bên phụ trách cập nhật song song; `cargo test -p lvhv-ffi` xanh):
`solar_to_lunar(_with)`, `jdn_to_lunar(_with)`, `day_info(_with)`, `day_ctx(_with)`, `day_ctx_jdn(_with)`,
`tet(_with)`, `solar_terms_of_year`, `find_good_days(_with)` → `Result<_, DateError>`; `ngochap_ctx`,
`ngoc_hap_of`, `can_chi_name`, `jd_to_datetime`, `YearTable::lunar_year` → `Option`; `YearTable::new →
Result`. Mới: `lunar_year_of_birth(_with)`, `lunar::new_moons_between`, `DateError`, các hằng miền,
`MIN_GOOD_DAY_SCORE`, `lvhv_ngochap::DayCtx::validate`. `new_moon_day`, `new_moon_ut_with`,
`sun_sector`, `lunar_month11` thu về `pub(crate)` (công cụ dump dùng `new_moons_between`). tsport mới:
`dayInfoWith`, `dayInfoJsonWith` (A-3), `lunarYearOfBirth(With)`, `LvhvError`, hằng miền, `isSuggestable`.

### Trung bình

| Mã | Trạng thái | Thay đổi | Bằng chứng |
|---|---|---|---|
| **RS-03** (panic với ngày sai) / **A-1** | ĐÃ SỬA | Hai chiều đối xứng: ngày dương sai → `Err(InvalidDate)`; `find_good_days`: một kiểu lỗi cho mọi trường hợp (ngày sai, ngoài miền, đảo ngược, quá dài, mã việc lạ) | `bad_input.rs::invalid_solar_dates_are_errors_not_panics` (29/2/2026, 29/2/2100, 30/2, 31/4, tháng 0/13/`u32::MAX`, ngày 0/32/`u32::MAX`), `::impossible_lunar_dates_are_none` (nhuận 5/2025, nhuận 2026, ngày 30 tháng thiếu, năm cực trị) |
| **RS-04** (tràn số) / **C-5** | ĐÃ SỬA | `canchi.rs`: mọi hàm chỉ số rút gọn đầu vào (`%`) trước khi cộng/trừ — không còn `lm - 1` (dòng 151 cũ), `month_can` không trừ; `can_chi_name → Option` (từ chối can ≥ 10, chi ≥ 12, lệch chẵn lẻ). `terms.rs:79` (`year + 1`): kiểm năm 2000–3000 trước. `lunar.rs:203` (`i -= 1`): tìm có cận. `jd_to_datetime` kiểm `is_finite` (C-8). **`[profile.release] overflow-checks = true`** ở `Cargo.toml` gốc | `canchi.rs::tests::total_on_u32_no_overflow` (0, 1, 11–13, 100, `u32::MAX−1`, `u32::MAX` cho mọi hàm; trong miền khớp công thức cũ), `can_chi_name_checks_domain`, `terms.rs::terms_out_of_range`, `jd.rs::{jd_to_datetime_rejects_non_finite, jdn_from_date_extremes_do_not_overflow}`; toàn bộ test release chạy với `overflow-checks = true` |
| **RS-05** / **C-3** (`as u8`) | ĐÃ SỬA | `ngochap_ctx` dùng `u8::try_from` + `DayCtx::validate()`, trả `None`; chú thích sai đã bỏ | `bad_input.rs::ngochap_ctx_rejects_bad_core_ctx` (`lunar_day = 167 811`, tháng 13, Ất Tý) |
| **C-4** (kẹp lặng lẽ trong ngochap) | MỘT PHẦN | `lvhv_ngochap::DayCtx::validate() -> Result<(), CtxError>`; lõi luôn gọi trước `evaluate/suitability`. `evaluate`/`suitability` vẫn kẹp để không panic với đầu vào bất kỳ (giữ chữ ký vì `ffi`), có ghi chú | `ngochap/src/tests.rs::validate_rejects_bad_ctx` |
| **RS-06** (tsport kiểu đầu vào) | ĐÃ SỬA | Mọi hàm xuất khẩu kiểm: ngày dương qua `isValidSolar` (`Number.isInteger`), JDN `Number.isSafeInteger`, `leap` phải `boolean` (không còn `!!leap`), phương pháp phải `"hnd"|"precise"`, năm sinh safe integer; `tet(2026.5)` → `InvalidArgument` (không còn "tháng Giêng luôn tồn tại") | `tsport/test/bad_input.mjs`: `jdnToLunar(1.5e6+0.5)` → `InvalidArgument`; `lunarToSolar(2025,6,1,1)` → `null`; NaN/±∞/2026.5/"2026"/null/undefined ở từng vị trí |
| **RS-13** (khoảng không giới hạn) | ĐÃ SỬA | Lõi: `to − from ≤ MAX_SPAN_DAYS = 366` (như PWA/ffi) → `Err(RangeTooLong)`; `from > to` → `Err(InvertedRange)`. `lvhv-dump gooddays` tự chia khúc 366 ngày | `bad_input.rs::find_good_days_range_rules` (2000–3000 → lỗi tức thì thay vì 92 s; cận 366 được, 367 không) |
| **C-2** (năm sinh âm/dương) | ĐÃ SỬA | Lõi: `lunar_year_of_birth(y,m,d)` (miền 1900–3000); tham số đổi tên `birth_lunar_year` ở Rust (`find_good_days`, `evaluate_day_for`, `lvhv_ngochap::suitability`) và `birthLunarYear` ở TS. PWA: ô **ngày sinh dương lịch** (ngày/tháng/năm) → đổi qua lõi → hiện "Tuổi Ất Mão (năm âm lịch 1975 — sinh trước Tết nên thuộc năm âm trước)"; ô chọn "Tôi chỉ biết năm âm lịch / Can Chi" nhận thẳng năm âm. Không lưu (S4) | `bad_input.rs::birth_lunar_year_before_tet` (15/01/1976 → 1975 Ất Mão; 31/01/1976 → 1976; 16–17/02/2026); smoke: "ngày sinh 15/01/1976 → tuổi Ất Mão, năm âm 1975", "năm âm 1976 → Bính Thìn", "ngày sinh không được lưu ({ls:{}, ss:{}, cookie:"", idb:[]})". Phần 1900–1999 chưa có kiểm định độc lập (oracle chỉ 2000–3000) — ghi trong `core/src/error.rs` |
| **RS-08** (cập nhật SW) | ĐÃ SỬA | `sw.js` bỏ `skipWaiting()` tự động, nhận `postMessage("skipWaiting")`; `main.ts` theo dõi `updatefound`/`waiting`, hiện thanh "Có bản mới của ứng dụng. [Tải lại]", tải lại một lần khi `controllerchange` (chỉ khi đã có SW cũ) | Thử trong Chrome (kịch bản dùng một lần ngoài kho): lần đầu không có thanh; đổi `sw.js` trên máy chủ → thanh hiện → bấm → trang tải lại, Cache Storage chỉ còn cache mới, 0 lỗi trang. Smoke: offline vẫn ĐẠT |
| **RS-10** (chung origin) | CHỦ DỰ ÁN QUYẾT | Không sửa mã. **Khuyến nghị:** phát hành trên origin riêng `https://lich.clevix.vn` bằng tên miền tùy chỉnh của GitHub Pages: (1) xác minh tên miền `clevix.vn` cho tài khoản/tổ chức GitHub (Settings → Pages → Verified domains, bản ghi TXT `_github-pages-challenge-…`) để chống chiếm tên miền phụ (S-2); (2) CNAME `lich` → `<org>.github.io`; (3) bật "Enforce HTTPS"; (4) giữ đường dẫn tương đối (đã có) và `scope: "./"`. Không dùng `<user>.github.io/<repo>/` dùng chung với ứng dụng khác (ví dụ trình quét tài liệu) | — |
| **RS-07** (quy trình phát hành) | ĐỂ LẠI cho tác nhân CI | Ngoài phạm vi ghi (`.github/`). Đã cung cấp `scripts/check-parity.sh` tự chứa để CI gọi; `build.mjs` không còn lặng lẽ dựng mock (C-10) | — |
| **C-11** (cài lại JDN/Can Chi lần 3) | ĐÃ SỬA | `pwa/src/format.ts` lấy `jdn`, `fromJdn`, `weekdayOf` từ `tsport/jd`; xóa bảng can/chi xoay vòng; `engine.lunarYearCanChi` dùng `tsport/canchi` | typecheck; smoke (tờ lịch, đổi ngày, ca dao theo JDN) ĐẠT |
| **A-4** (dò tên hàm) | ĐÃ SỬA | `pwa/src/engine.ts` gọi `#engine-impl` trực tiếp (tsc kiểm kiểu với `tsport/src/index.ts`); mock chỉ qua `--mock`, có bề mặt giống tsport | `npm run typecheck` sạch |

### Thấp / Thông tin

| Mã | Trạng thái | Thay đổi / lý do | Bằng chứng |
|---|---|---|---|
| **RS-20** (giấy phép, tên phông) | ĐÃ SỬA (phần mã/phông) | `LICENSE` gốc: độc quyền "All rights reserved, Clevix LLC"; `core/`, `ngochap/` Cargo.toml đổi `license = "MIT"` → `license-file = "../LICENSE"`. `content/cadao/LICENSE`: CC BY-SA 4.0 + ghi công nguyên văn SOURCES.md §C.1 + phạm vi. Phông: `pwa/scripts/subset-font.py` rút gọn lại và **đổi tên** bảng `name` → họ "LichViet Serif" (PS "LichVietSerif-Regular/Italic"), tệp `lichviet-serif*.woff2`, CSS bỏ `"Lora"` khỏi chuỗi phông; `OFL.txt` thêm ghi chú bản sửa đổi; màn Giới thiệu ghi rõ | Script tự kiểm: không có "Lora" ở nameID nào ngoài 0 (bản quyền), 13, 14 (giấy phép) — OFL bắt buộc giữ các trường này. `ffi/Cargo.toml` vẫn ghi MIT — xem §D |
| **RS-16** (metrics trên mock) | ĐÃ SỬA | Đo lại trên bản build hiện tại (lõi tsport) | `pwa/test/metrics.json` (`engine: "tsport"`): **P3** 313,9 KB gzip lần tải đầu (< 500 KB, ĐẠT); **P4** heap JS 2,43 MB (360×780) / 2,38 MB (1280×800) (< 30 MB, ĐẠT); **P5** offline lần hai ĐẠT; S3 0 yêu cầu ngoài; S4 chỉ `lvhv.ui` |
| **C-10** (mock lặng lẽ) | ĐÃ SỬA | `build.mjs` thất bại (mã 1) nếu thiếu `tsport/src/index.ts`, `cadao.json`, `content/cadao/SOURCES.md` hoặc `content/ngochap/SOURCES.md`; chỉ `node build.mjs --mock` mới cho dựng giả lập (cảnh báo "KHÔNG được phát hành"); đối số lạ → mã 2 | Thử: tạm chuyển `tsport/src/index.ts` → `LỖI: thiếu lõi tsport …` exit 1; `--mock` → exit 0, `lõi: mock`; `--bogus` → exit 2 |
| **RS-12** (tệp vàng tự sinh) | ĐÃ SỬA | `check-parity.sh` sinh mới vào thư mục tạm; `compare.mjs` nhận `LVHV_GOLDEN_DIR`, báo THIẾU tệp `tet`/`gooddays` là lỗi (trước đây bỏ qua) | §A |
| **RS-09** (CSP) | MỘT PHẦN | Bỏ `data:` khỏi `img-src` (không dùng). **Chưa** thêm `require-trusted-types-for 'script'`: `dom.ts svg()` dùng `DOMParser.parseFromString` (sink của Trusted Types) → cần thêm chính sách; để đợt sau. `frame-ancestors` cần tiêu đề HTTP — phụ thuộc hosting (RS-10) | CSP trong `metrics.json` |
| **RS-11** (nội dung → DOM) | MỘT PHẦN | `h()` từ chối khóa `on*` có giá trị không phải hàm (ném `TypeError`). Chưa giới hạn độ dài/ký tự điều khiển trong `loadCadao` | typecheck + smoke |
| **RS-18** (quyền riêng tư) | ĐÃ SỬA | Màn Giới thiệu thêm: "Máy chủ lưu trữ trang (GitHub Pages) có thể ghi nhật ký truy cập thông thường (địa chỉ IP, loại trình duyệt) như mọi trang web." | smoke ĐẠT |
| **C-6 / C-7** (`clamp`/`min` ở 360°) | ĐÃ SỬA | `sun_sector(Precise)`: `rem_euclid(12)`; `term_index_of_day`: `% 24`; tsport tương tự | Tệp vàng 2000–3000 không đổi byte nào (trường hợp 360,0 không xảy ra trong miền) |
| **C-8** (`jd_to_datetime` NaN) | ĐÃ SỬA | → `Option`, `None` khi không hữu hạn hoặc ngoài miền | `jd.rs::jd_to_datetime_rejects_non_finite` |
| **C-9** (Newton không báo) | MỘT PHẦN | `solar_terms_of_year` trả `Err(Computation)` nếu không đủ 24 tiết khí hữu hạn; `solve_sun_longitude` vẫn trả `f64` (nội bộ) | `terms.rs` tests |
| **C-12** (`i < 4`) | ĐÃ SỬA | Chú thích chứng minh trong `YearTable::lunar_year` | — |
| **A-3** (hai kiểu chọn phương pháp) | MỘT PHẦN | Thêm `dayInfoWith`/`dayInfoJsonWith` (thứ tự đối số như Rust); bản có tham số cuối tùy chọn giữ lại vì `compare.mjs`/smoke dùng | — |
| §6 PWA ranh giới lỗi | ĐÃ SỬA | `main.ts renderDay` bắt lỗi lõi → "Không tính được ngày …"; `render()` lúc khởi động được bọc; `gooddays.ts` bắt `LvhvError` | smoke ĐẠT |
| §8 thụt lề `addItems` | ĐÃ SỬA | `pwa/src/ui/gooddays.ts` | — |
| **RS-14, RS-17** | Thông tin | Không cần sửa | — |
| **RS-15** (tsport không lockfile) | ĐỂ LẠI | tsport không có phụ thuộc riêng; dùng `pwa/package-lock.json` (đã ghi trong `check-parity.sh`). Thêm lockfile rỗng không đổi rủi ro | — |
| **RS-19** (URL nguồn lặp) | ĐỂ LẠI | Tối ưu kích thước, P3 vẫn đạt rộng (313,9 KB); sửa cần đổi lược đồ nhúng cadao | — |
| **A-2, A-5, A-6** | ĐỂ LẠI | Chỉ là quy ước tên / trường thừa; đổi sẽ làm vỡ JSON công khai và ffi — cần quyết định hợp đồng | — |
| DUYET-MA §3 N1 (test liên kết `tables.rs` ↔ `rules.md`) | ĐỂ LẠI | Thuộc A3/nội dung; không chặn | — |
| §7 docs/03 §13 (hiệu năng cũ) | MỘT PHẦN | Không có quyền ghi `docs/03`; số đo mới ở §E dưới đây — người phụ trách docs/03 chép vào §13 | §E |

## D. Câu hỏi mở cho chủ dự án

1. **Giấy phép mã nguồn.** Chủ dự án chưa chọn giấy phép mã nguồn mở. `LICENSE` gốc tạm ghi "All rights
   reserved, Clevix LLC". Trước đây `Cargo.toml` của `core`, `ngochap`, `ffi` ghi `license = "MIT"` mà
   không có văn bản MIT. Đã đổi `core`/`ngochap` sang `license-file = "../LICENSE"`; **`ffi/Cargo.toml`
   vẫn ghi `license = "MIT"`** (ngoài phạm vi ghi của đợt này) — người phụ trách `ffi/` nên đổi cho nhất
   quán. Nếu chọn MIT/Apache-2.0 sau này: thay `LICENSE`, đổi lại các `Cargo.toml`, giữ
   `content/cadao/LICENSE` (CC BY-SA) và `OFL.txt` riêng.
2. **Origin phát hành** (RS-10): khuyến nghị `lich.clevix.vn` (§C).
3. **Ngày sinh 1900–1999.** Lõi nhận ngày sinh từ 1900 để tính tuổi, nhưng kiểm định độc lập chỉ phủ
   2000–3000. Có cần mở rộng oracle cho 1900–1999 (ít nhất ngày Tết) không?
4. **Ngưỡng §2a** (`MIN_GOOD_DAY_SCORE = 0`) là quy ước dự án; chọn ≥ 0 để không bao giờ đề xuất ngày điểm
   âm mà danh sách vẫn đủ dài (§B.1). Có thể nâng lên `NEN_THRESHOLD = 3` nếu muốn danh sách khớp mục
   "Nên làm"; khi đó số ngày cưới hỏi/năm giảm thêm.

## E. Hiệu năng trước/sau `overflow-checks = true` (RS-04)

`lvhv-dump bench` (release, LTO, codegen-units = 1), cùng mã nguồn, hai bản dựng chỉ khác
`overflow-checks` (bản "tắt" dựng bằng `CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=false`), chạy **xen kẽ** 5 lần
mỗi bản trên sandbox 2 vCPU (nhiễu lớn); số là **trung vị** của các trung bình mỗi lần:

| Phép đo | Hnd tắt | Hnd bật | Precise tắt | Precise bật |
|---|---|---|---|---|
| `day_info + to_json`, 365 ngày 2026 (µs/ngày) | 27,9 | 36,8 | 46,6 | 49,8 |
| `find_good_days` 90 ngày, 16 việc × 2 (µs/lần) | 1 002 | 739 | 1 560 | 1 923 |
| `solar_to_lunar` 2000–3000 (µs/ngày) | 4,07 | 4,40 | 20,1 | 20,5 |
| `solar_to_lunar` 2000–3000, tổng (s) | 1,51 | 1,64 | 7,38 | 7,53 |

Phép đo ổn định nhất (`solar_to_lunar`, 365 608 lời gọi) cho chi phí **≈ +8 % (Hnd), ≈ +2 % (Precise)**;
hai phép đo ngắn dao động ±30 % giữa các lần nên chênh lệch nằm trong nhiễu. Ngưỡng vẫn dư xa: **P1**
`day_info` ≤ 0,05 ms/ngày (< 1 ms), **P2** `find_good_days` 90 ngày ≤ 2 ms (< 50 ms) — kể cả kiểm tra §2a
(mỗi ngày gọi thêm `evaluate`). Trước mọi thay đổi của đợt này (bản gốc, `overflow-checks` tắt):
`day_info` 26,6–39,9 µs, `find_good_days` 660–744 µs, `solar_to_lunar` 4,02–4,12 µs (tổng 1,50–1,54 s).
Kết luận: giữ `overflow-checks = true`.

tsport (Node 22, `compare.mjs`): `dayInfo` 27 µs, `dayInfoJson` 44 µs, `findGoodDays` 90 ngày 2,8 ms.

## F. Ảnh hưởng tới `ffi/` (không sửa trong đợt này)

Bên phụ trách `ffi/` đã cập nhật song song theo API mới (`?` với `From<DateError> for LvhvError`,
`lvhv_core::find_good_days`, `lunar_year_of_birth`); `cargo clippy --workspace -D warnings` và
`cargo test -p lvhv-ffi --release` xanh, `gd_check` 0/0. Còn lại: `ffi/Cargo.toml license = "MIT"` (§D.1).
