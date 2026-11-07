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

> **Quyết định 06/11/2026:** chủ dự án chọn giấy phép **MIT** cho mã nguồn. `LICENSE` gốc là văn bản MIT; `core`, `ngochap`, `ffi`, `apps/windows` ghi `license = "MIT"`. Dữ liệu ca dao vẫn theo CC BY-SA 4.0 (`content/cadao/LICENSE`), phông chữ theo OFL.


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

## G. Bổ sung A1 — mở rộng phạm vi 1900-01-01 → 3000-12-31 (2026-11-06)

Tiêu chí khóa trước: docs/02 "Bổ sung A1" (L9–L13). Quy tắc, bằng chứng, giới hạn: docs/03 §16.

| Hạng mục | Thay đổi | Bằng chứng |
|---|---|---|
| Quy tắc múi giờ | `core/src/astro.rs`: `local_day_of`, `local_midnight_ut`, `tz_of_day`, `tz_of_instant`; UTC+8 cho thời điểm trước 1968-01-01 00:00 UTC+8, UTC+7 từ đó — dùng chung cho Hnd và Precise (Sóc, Trung khí, tiết khí). ΔT Espenak–Meeus 1860–1986. tsport: `tsport/src/astro.ts` (cùng thứ tự phép tính) | docs/03 §16.2–16.4 |
| Miền | `MIN_YEAR = 1900`, `MIN_JDN = 2415021`, `MIN_LUNAR_YEAR = 1899`, `BIRTH_MIN_YEAR = MIN_YEAR`, tuế nội bộ 1899–3101; `find_good_days`, `solar_terms_of_year`, `tet` nhận 1900+ | `core/tests/bad_input.rs` (biên 1899/1900) |
| L11 | ĐẠT cả hai phương pháp | `core/tests/range1900.rs::l11_continuity_1967_1969` |
| L12 | Khứ hồi 1900–3000: 402 132 ngày × 2, 0 lỗi; TS ↔ Rust 1900–3000: 0 sai lệch (dayinfo 402 132 dòng, Tết 1 101 năm, gooddays 4 470 dòng, mỗi phương pháp) | `range1900.rs::l12_roundtrip_1900_3000`; `scripts/check-parity.sh` (≈ 1 phút 20 giây) |
| L13 | 2000–3000 giống hệt từng byte tệp chuẩn (hnd, precise) | `check-parity.sh` bước 5 |
| L9 | Tết 1954, 1968, 1969, 1985 khớp nguồn; Tết 1916 lõi 04/02, bảng TK20 của Hồ Ngọc Đức và lịch Trung Quốc 03/02 (giới hạn quy ước UTC+8 trước 1929, docs/03 §16.5) | `range1900.rs::tet_20th_century_published`, `known_limitation_1916` |
| Dữ liệu cho kiểm định | `core/out/{,precise/}{lunar_1900_1999.csv, tet_1900_1999.csv, newmoons_1900_1999.csv, dayinfo_1900_3000.jsonl.gz}`; `lvhv-dump newmoons/terms` ghi độ lệch đã dùng ở mỗi dòng (`+08:00`/`+07:00`) | — |
| `check-parity.sh` | Sinh dayinfo 1900–3000, Tết 1900–1999 + 2000–3000; thêm bước L13 | — |
| ffi | Chỉ sửa test (`ffi/src/tests.rs`: 1999 → 1899, `min_year` 2000 → 1900) để khớp hằng miền mới; chú thích `2000–3000` trong `ffi/src/lib.rs` chưa đổi | `cargo test -p lvhv-ffi` xanh |
| PWA — Đổi ngày | Ba ô số Ngày/Tháng/Năm (thay `<input type=date>`), nút "Đổi sang âm lịch"/"Đổi sang dương lịch" (vẫn tự cập nhật khi gõ); kết quả "Ngày 27 tháng 9 năm Đinh Tỵ" + "Ngày Kỷ Tỵ · tháng Canh Tuất"; âm → dương "Thứ Ba, ngày 08/11/1977"; mọi nhánh lỗi hiện thông báo ("Ứng dụng hỗ trợ từ 01/01/1900 đến 31/12/3000.", "Ngày không hợp lệ: …", tháng nhuận không có, ngày 30 tháng thiếu); ghi chú múi giờ trước 1968 | `pwa/test/smoke.mjs` |
| PWA — khác | Bảng tháng: năm ngoài phạm vi → thông báo; Chọn ngày tốt: khoảng ngoài phạm vi/ngày sai → thông báo; Giới thiệu: phạm vi + ghi chú múi giờ | smoke |

Lỗi người dùng báo (bên điều phối): 1977-11-08 ở bản 2000+ không hiện gì — nguyên nhân: `<input type=date min=2000-01-01>` bỏ giá trị ngoài min/max và nhánh lỗi không chạy khi `change` không phát. Đã sửa bằng ba ô số + nút + thông báo mọi nhánh. 1977-11-08 nay = 27/9 Đinh Tỵ, ngày Kỷ Tỵ, tháng Canh Tuất (khớp giá trị PyEphem UTC+7 của bên điều phối; năm 1977 sau điểm chuyển nên quy tắc múi giờ không ảnh hưởng).

Kiểm tra cuối đợt: `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings` sạch; `cargo test --workspace --release` xanh; `cargo test --release -p lvhv-core -- --ignored` (xem kết quả trong commit); `scripts/check-parity.sh` ĐẠT; PWA `npm run typecheck && npm run build && node test/smoke.mjs` ĐẠT (94 kiểm tra). Metrics (`pwa/test/metrics.json`): P3 346,3 KB gzip lần tải đầu (< 500), P4 heap 2,43 MB (< 30), P5 offline ĐẠT, S3/S4 ĐẠT.

## H. Bổ sung A2 — giờ địa phương Bắc Kinh trước 1929 (2026-11-06)

Quyết định chủ dự án ghi ở docs/02 "Bổ sung A2" (sau khi thấy kết quả L9 lần 1, có bằng chứng độc lập).
Quy tắc và bằng chứng: docs/03 §16.1, §16.3–16.5.

| Hạng mục | Thay đổi | Bằng chứng |
|---|---|---|
| Quy tắc | `core/src/astro.rs`, `tsport/src/astro.ts`: thêm `TZ_BEIJING = 27940/3600` (UT+7:45:40), `TZ_1929_JDN`; trước 1929-01-01 00:00 giờ Bắc Kinh → giờ Bắc Kinh; 1929 → 1967 UTC+8; từ 1968 UTC+7. Cả hai phương pháp. 1928-12-31 dài 24 giờ, 1929-01-01 dài 23:45:40 | `range1900.rs::switch_constants`, `continuity_around_1929` |
| Tết 1916 | 1916-02-03 (trước đây 02-04), khớp HKO 1916 và TK20 | `range1900.rs::tet_1916_beijing_time`; bỏ `known_limitation_1916` |
| TK20 | Bảng nhúng nguyên văn (có nguồn) trong test; Precise trùng 1237/1237 tháng, Hnd 1233/1237 (vị trí tháng nhuận 1917, 1922 do mô hình Mặt Trời HND; docs/03 §16.4); Tết 100/100 năm cả hai phương pháp | `range1900.rs::tk20_month_starts_1900_1999` |
| Dữ liệu kiểm định | Sinh lại `core/out/{,precise/}{lunar,tet,newmoons}_1900_1999.csv`, `dayinfo_1900_3000.jsonl.gz`; độ lệch trong `newmoons` ghi đến giây (`+07:45:40`) | — |
| L12, L13 | `check-parity.sh` ĐẠT: dayinfo 402 132 dòng × 2, Tết 1 101 × 2, gooddays 4 470 × 2, 0 sai lệch; 2000–3000 giống hệt từng byte tệp chuẩn; khứ hồi 1900–3000: 402 132 ngày × 2, 0 lỗi | — |
| PWA | Ghi chú: "Trước năm 1929, âm lịch tính theo giờ địa phương Bắc Kinh; 1929–1967 theo múi giờ UTC+8; từ 1968 theo UTC+7 (chuẩn miền Bắc). …" (Đổi ngày, Giới thiệu); smoke thêm 03/02/1916 → 1/1 Bính Thìn: 95 kiểm tra ĐẠT; P3 346,4 KB, P4 2,43 MB | `pwa/test/smoke.mjs`, `pwa/test/metrics.json` |

## I. Bổ sung A3 (mặc định Precise) và cập nhật phiên bản PWA (2026-11-06)

### I.1 Phương pháp mặc định → `Precise`

| Hạng mục | Thay đổi | Bằng chứng |
|---|---|---|
| Lõi | `core/src/lunar.rs`: `DEFAULT_METHOD = Method::Precise`; `lvhv-dump` ghi "mặc định: precise" | `fixtures.rs::default_method_is_precise` |
| tsport | `tsport/src/lunar.ts`: `DEFAULT_METHOD = "precise"` | `pwa/build.mjs` dừng nếu khác lõi Rust (giữ nguyên kiểm tra) |
| Test HND | Các test riêng của HND gọi tường minh `Method::Hnd`: `fixtures.rs::roundtrip_2000_3000` (khứ hồi HND), `roundtrip_2000_2100` (cả mặc định và HND), `tet_known` (thêm cột HND), `methods.rs::precise_fixture_2026_11_05` (L8 cho cả hai) | `cargo test --workspace --release`: mọi test ĐẠT |
| Parity, L12, L13 | `scripts/check-parity.sh` (cả hai phương pháp 1900–3000): dayinfo 402 132 dòng × 2, Tết 1 101 × 2, gooddays 4 470 × 2, **0 sai lệch**; L13 hnd và precise **giống hệt từng byte** tệp chuẩn (tệp chuẩn tách theo phương pháp nên không đổi); 1 phút 27 giây | — |
| Test dài | `cargo test --release --workspace -- --ignored`: roundtrip 2000–3000 (HND), precise roundtrip, tự nhất quán (Precise 0; HND đúng 6 năm đã khóa), N3 cả dải, §2a, L12 khứ hồi 1900–3000 — ĐẠT | — |
| PWA | Đọc phương pháp từ lõi Rust lúc build (`metrics.json` `"method": "precise"`); Giới thiệu: "Ngày âm lịch tính theo mô hình thiên văn chính xác (Meeus)… Thuật toán Hồ Ngọc Đức… vẫn được giữ… để đối chiếu" | smoke |
| CI | `ci.yml` sinh dữ liệu đối chiếu 1900–3000 (trước đây chỉ 2000–3000 nên `tsport/test/compare.mjs` báo MISSING, CI đỏ từ commit 94fbf17) | — |

Hai phương pháp khác ngày âm ở 178 ngày trong 1900–2100 (1917-03-23→05-20, 1922-06-25→08-22,
2072-12-09→2073-01-07, 2077-11-15→12-14; so `lvhv-dump lunar-csv` của hai phương pháp). docs/03 §15.7.

### I.2 Hiện phiên bản và nhắc cập nhật (PWA)

| Hạng mục | Thay đổi |
|---|---|
| Nguồn số phiên bản | `pwa/build.mjs` đọc tệp `VERSION` ở gốc kho (bỏ hằng "0.1"); `LVHV_VERSION` chỉ dùng khi kiểm thử. Mã bản dựng = mã commit git rút gọn (thêm `-dirty` nếu nguồn PWA có sửa chưa commit), không có git thì băm nội dung nguồn; ngày dựng theo UTC |
| `dist/version.json` | `{"version","build","date","notes"}`; `notes` = các dòng "- …" dưới `## [X.Y.Z]` của `CHANGELOG.md` (thiếu mục của phiên bản đang dựng → build thất bại). **Không** nằm trong precache |
| Service worker | `version.json`: chỉ qua mạng, `cache: "no-store"`, mất mạng → lỗi (trang báo "đang ngoại tuyến"); tệp khác giữ cache-first. Nhận thông điệp `"skipWaiting"` (bản 0.1) và `SKIP_WAITING` (bản mới). Vẫn không tự `skipWaiting()` (RS-08) |
| Giao diện | Chân trang "Bản quyền © 2026 của Clevix Lab, version X.Y.Z" (số thật). Giới thiệu → mục **Phiên bản**: "Phiên bản đang dùng: X.Y.Z (bản dựng …, ngày …)", nút **Kiểm tra cập nhật** (Đang kiểm tra… / Bạn đang dùng phiên bản mới nhất. / Có phiên bản mới X.Y.Z + ghi chú + **Cập nhật ngay** / Không kiểm tra được (đang ngoại tuyến).) |
| Tự nhắc | Lúc mở ứng dụng và khi trang hiện lại, tối đa 6 giờ một lần (mốc chỉ trong bộ nhớ của trang — S4), kiểm tra im lặng; có bản mới → **một** thanh ở đầu trang "Đã có phiên bản mới X.Y.Z. [Cập nhật ngay] [Để sau]" (thay thanh "Có bản mới — Tải lại" cũ). `role="status"`, nút bấm được bằng bàn phím, không tràn ngang ở 360 px với A++ |
| Cập nhật ngay | `registration.update()` nếu chưa có bản chờ → chờ cài xong → `postMessage({type:"SKIP_WAITING"})` → tải lại một lần khi `controllerchange` |
| Kiểm thử | `pwa/test/smoke.mjs` mục 1b: bản A (VERSION) → bản B (`LVHV_VERSION` = A + 1 bản vá, dựng vào thư mục tạm qua `LVHV_OUT_DIR`); kiểm thanh nhắc chứa số bản B, tự nhắc khi mở lại, "Để sau", "Cập nhật ngay" → chân trang hiện bản B; ngoại tuyến → thông báo ngoại tuyến |

### I.3 Người đang dùng bản 0.1 (bản đang chạy trên GitHub Pages) sẽ thấy gì

Đã kiểm bằng mô phỏng: dựng lại bản 0.1 từ commit ed08f16 (tên cache `lvhv-v0.1-4eae0158fe`, trùng bản đang
chạy), mở trong Chromium, rồi máy chủ đổi sang bản 0.1.1.

- Service worker 0.1 không tự `skipWaiting()`; khi kích hoạt nó gọi `clients.claim()` và xóa cache `lvhv-*`
  cũ. Trình duyệt tự kiểm `sw.js` mỗi lần mở trang (máy chủ trả `sw.js` khác → cài bản mới ở trạng thái chờ).
- **Lần mở đầu sau khi phát hành:** ứng dụng vẫn hiện bản 0.1 (tờ lịch theo phương pháp HND), và hiện thanh
  cũ ở **cuối** màn hình: "Có bản mới của ứng dụng." + nút **Tải lại**. Bấm **Tải lại** → bản mới kích hoạt
  (service worker mới nhận thông điệp cũ `"skipWaiting"`), trang tải lại một lần → chân trang
  "version 0.1.1". Đã kiểm: ĐẠT.
- **Không bấm:** khi đóng hẳn mọi cửa sổ/thẻ của ứng dụng rồi mở lại, bản 0.1.1 tự được dùng. Đã kiểm: ĐẠT.
- Bản 0.1 không có nút Kiểm tra cập nhật hay thanh ở đầu trang; từ 0.1.1 trở đi mới có (thanh ở đầu trang,
  có số phiên bản).
- Khi kiểm tra `sw.js`, trình duyệt bỏ qua bộ nhớ đệm HTTP của chính nó (mặc định `updateViaCache: "imports"`).
  Bộ nhớ đệm phía máy chủ/CDN của GitHub Pages có thể làm chậm vài phút sau khi phát hành (chưa đo được từ
  sandbox vì proxy chặn truy cập trang).

## J. "Để sau" bị bỏ qua — thanh cập nhật tự hiện lại (PWA, 2026-11-07)

**Triệu chứng.** Thanh "Đã có phiên bản mới X.Y.Z. [Cập nhật ngay] [Để sau]": người dùng bấm **Để sau**, vài giây sau
thanh tự hiện lại. Tìm thấy qua kiểm thử nâng cấp `pwa/test/upgrade.mjs` chập chờn: 1 lỗi trong 20 lần chạy, đúng
kiểm tra `(a) "Để sau" → ẩn thanh, bản cũ chạy tiếp` (nhật ký lần 13). 19 lần còn lại đạt.

**Nguyên nhân.** Một lần phát hành được báo tới thanh hai lần:
- lần 1 khi đọc `version.json` (autoCheck lúc mở ứng dụng);
- lần 2 khi service worker mới cài xong (`attachRegistration`).

Lúc mở lại ứng dụng mà đã có bản chờ, hai lần kiểm tra chạy song song. Mỗi lần báo cùng phiên bản đều đặt
`updateBar.hidden = false`. Nếu người dùng bấm "Để sau" sau lần báo thứ nhất nhưng trước lần báo thứ hai, lần báo thứ
hai bỏ qua lựa chọn đó. Lỗi có trong bản 0.1.4 đang phát hành và bản 0.2-dev. Không liên quan tới IndexedDB hay
việc kích hoạt service worker.

**Sửa (trong ứng dụng).**
- `update.ts`: mỗi lần kiểm tra mang nguồn (`user` / `auto` / `install`) và số thứ tự lúc bắt đầu.
- `main.ts`: sau "Để sau", cùng phiên bản đó không tự hiện lại vì lần báo `install`, cũng không vì một lần kiểm tra đã
  bắt đầu trước khi bấm.
- Vẫn nhắc lại khi có lần kiểm tra mới: mở lại ứng dụng (tối đa 6 giờ một lần) hoặc bấm **Kiểm tra cập nhật**.
- Phiên bản khác thì báo ngay, như hướng dẫn sử dụng (06) đã mô tả.

**Kiểm thử.**
- `smoke.mjs` mục 1b, kiểm tất định. Sau "Để sau", cho một service worker mới cài xong, chờ đúng lần báo của nó tới
  thanh (`data-reports` tăng), rồi kiểm thanh vẫn ẩn. Trước khi sửa: LỖI mỗi lần. Sau khi sửa: ĐẠT.
- `upgrade.mjs`. Bản cũ là mã đã phát hành, không sửa được, nên kiểm thử mở lại ứng dụng khi bản mới đã ở trạng thái
  chờ, rồi chờ đủ hai lần đọc `version.json` của bản cũ (và mạng yên) mới bấm "Để sau". Đây là chờ đúng điều kiện,
  không thử lại. Kết quả: 20/20 lần đạt.
- Người đang dùng 0.1.4 có thể còn gặp lỗi này một lần cho tới khi cập nhật lên bản có bản sửa.

## K. MA3-15-03 (Esc trong lúc mã hóa / khôi phục) — kiểm thử tất định; không móc kiểm thử nào trong bản phát hành (PWA, 2026-11-07)

**Triệu chứng.** Kiểm tra MA3-15-03 (Esc trong lúc mã hóa sao lưu) thỉnh thoảng lỗi với `esc: []`. Mã hóa xong trước
khi kiểm thử kịp nhấn Esc. Kiểm tra Esc trong lúc khôi phục thì luôn đạt nhưng không thật sự thử chốt chặn: khôi phục
xong trước khi nhấn Esc thì Esc không chạm hộp nào. Đây là lỗi của kiểm thử. Chốt chặn Esc trong
`src/people/ui.ts` vẫn đúng.

**Móc chỉ có ở bản dựng kiểm thử.**
- `src/testhooks.ts` có `testGate(name)` và `testFlag(name)`. Cờ `__LVHV_TEST_HOOKS__` do `build.mjs` thay lúc dựng
  (khai báo ở `src/globals.d.ts`). Mọi nơi gọi đều bọc `if (__LVHV_TEST_HOOKS__)`.
- `LVHV_TEST_HOOKS=1` chỉ được nhận cùng `LVHV_OUT_DIR`. Bản dựng chính `dist/` luôn có cờ false, nên esbuild bỏ hẳn
  lời gọi và tên móc.
- Cổng `backup-encrypt` nằm ngay trước bước mã hóa. Cổng `backup-restore` nằm ngay trước `applyImport`.
- Cờ `__lvhvFailMigration` (R14 ca chuyển đổi lỗi) trước đây có trong bản phát hành. Nay chỉ có ở bản dựng kiểm thử,
  qua `testFlag`.
- Smoke kiểm mọi tệp trong `dist/` không chứa `__lvhvGates`, `__lvhvFailMigration`, `backup-encrypt`, `backup-restore`,
  `testGate`, `testFlag`, `__LVHV_TEST_HOOKS__`.

**Kiểm thử.**
- Smoke mục 1e dựng bản kiểm thử vào thư mục tạm và chạy `runEscGate()`:
  1. bấm "Tạo và tải tệp", giữ bước mã hóa;
  2. nhấn Esc, kiểm `cancel` bị chặn và hộp vẫn mở, đang bận;
  3. mở cổng, kiểm tệp được tải;
  4. mở lại chính tệp đó, chọn "Thay toàn bộ", giữ bước khôi phục;
  5. nhấn Esc, kiểm như bước 2;
  6. mở cổng, kiểm có thông báo "Đã khôi phục".
- Luồng L9/L10 trên bản dựng chính vẫn xuất và khôi phục thật, chỉ bỏ phần nhấn Esc phụ thuộc thời gian.
- `upgrade.mjs`:
  - ca (a) và (b) dùng bản mới dựng chính `dist/`. Ca (b) còn đặt cờ `__lvhvFailMigration`, và bản chính bỏ qua: chuyển
    đổi bình thường;
  - chỉ ca (c) phát hành bản mới dựng kiểm thử (`test/out-upgrade/dist-new-testhooks`);
  - bản cũ (0.1.4, 0.2-dev) vẫn là bản dựng chính của commit cũ.

**Chứng minh kiểm thử bắt lỗi.** Tạm gỡ chốt chặn Esc trong `ui.ts` → cả hai kiểm tra LỖI
(`Esc [{"busy":"true","prevented":false}], hộp mở 0`). Trả lại chốt chặn → ĐẠT.

**Còn lại trong bản phát hành.**
- `window.__lvhv` (số đo thời gian dựng trang và lối vào mô-đun lưu trữ): các bộ kiểm thử đọc trên bản dựng chính.
  Đây không phải móc làm đổi hành vi, nhưng là lối vào dữ liệu từ trang. Muốn bỏ thì phải chuyển các kiểm thử đó sang
  bản dựng kiểm thử. Chưa làm.
- Tên tùy chọn `forceMigrationFailure` trong mã lưu trữ của lõi (`store/db.ts`, dùng cho `store-tool`). Ứng dụng
  luôn truyền `{}` nên trang không chạm tới được.
