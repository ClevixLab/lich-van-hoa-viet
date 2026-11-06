# Rà soát bảo mật — Lịch và Văn hóa Việt v0.1

Vai: A9 (bảo mật), người duyệt độc lập. Ngày: 2026-11-06.
Phạm vi: `core/` (lvhv-core), `ngochap/` (lvhv-ngochap), `tsport/`, `pwa/`; tham khảo `ffi/` (mới xuất hiện).
Mã nguồn thay đổi đồng thời trong lúc duyệt; số dòng dưới đây theo tệp lúc ~03:00–03:10 (giờ tệp).
Môi trường: Linux x86_64, rustc/cargo 1.97.0, Node v22.23.4, Chrome (google-chrome hệ thống) qua playwright-core 1.63.0.

Mức: **Nghiêm trọng / Cao / Trung bình / Thấp / Thông tin**.
Bộ kiểm thử dùng một lần (fuzz/tính chất) nằm ngoài kho, ở
`/tmp/claude-0/-home-claude/d8cfd5b0-b603-5248-8abb-496e9a334290/scratchpad/review/` (`fuzz/`, `dbg/`,
`scan/`, `gd/`, `probe.mjs`, `parity.mjs`, `gdts.mjs`); đầu ra thô: `fuzz-release-v2.txt`.

## Tổng hợp

| Mức | Số phát hiện |
|---|---|
| Nghiêm trọng | 0 |
| Cao | 2 |
| Trung bình | 7 |
| Thấp | 7 |
| Thông tin | 4 |

| Mã | Mức | Tiêu đề ngắn |
|---|---|---|
| RS-01 | Cao | Lõi Rust + TS treo vô hạn với năm/JDN cực lớn (DoS) |
| RS-02 | Cao | Lõi trả kết quả **sai lặng lẽ** rồi panic ngoài khoảng ~−14 600…35 650; `Method::Precise` panic ở năm 31 500 / 100 000; TS ném lỗi không phải lỗi miền |
| RS-03 | Trung bình | API công khai `solar_to_lunar`/`day_info`/`find_good_days` panic với ngày không hợp lệ thay vì trả lỗi |
| RS-04 | Trung bình | Tràn số nguyên: panic ở bản debug, gói quanh (wrap) lặng lẽ ở bản release |
| RS-05 | Trung bình | `ngochap_ctx` ép `u32 → u8` có mất thông tin; chú thích "mọi chỉ số < 60" sai |
| RS-06 | Trung bình | tsport chấp nhận số không nguyên / kiểu sai ở một số hàm công khai (`jdnToLunar(1.5e6+0.5)` → `d: 15.5`) |
| RS-07 | Trung bình | Không có quy trình phát hành có kiểm soát (không CI, không bảo vệ nhánh, không công bố băm) |
| RS-08 | Trung bình | SW `skipWaiting()+clients.claim()` không có luồng cập nhật; trang đang mở chạy lẫn bản cũ/mới |
| RS-09 | Thấp | CSP qua `<meta>`: không có `frame-ancestors` (không hiệu lực trong meta) → chống clickjacking phụ thuộc hosting |
| RS-10 | Trung bình | Rủi ro chung origin trên GitHub Pages (đường dẫn con) |
| RS-11 | Thấp | Kênh nội dung → DOM an toàn hiện tại, nhưng kiểm tra lược đồ ca dao không giới hạn độ dài/ký tự |
| RS-12 | Thấp | Tệp vàng `core/out/*.jsonl.gz` do chính lõi sinh — kiểm tra L7 không phát hiện lỗi chung của cả hai bản |
| RS-13 | Thấp | `find_good_days` (lõi) không giới hạn khoảng; 2000–3000 mất 92 s |
| RS-14 | Thông tin | npm audit: 0 lỗ hổng đã biết; esbuild có `postinstall` |
| RS-15 | Thấp | `tsport/` không có lockfile; `npm audit` không chạy được ở đó |
| RS-16 | Thấp | `pwa/test/metrics.json` đo trên bản dựng lõi giả lập (mock) — không còn đại diện |
| RS-17 | Thông tin | Kiểm chứng S3/S4 bằng trình duyệt thật: 0 yêu cầu ngoài, 0 vi phạm CSP, chỉ một khóa `lvhv.ui` |
| RS-18 | Thông tin | Lời "không gửi thông tin đi đâu" chưa nói tới nhật ký truy cập của hosting |
| RS-19 | Thông tin | Gói JS 431 KB vì nhúng URL nguồn ca dao lặp lại hàng trăm lần |
| RS-20 | Thấp | Giấy phép: thiếu tệp LICENSE (MIT khai báo nhưng không có văn bản); ffi kéo crate MPL-2.0; tên họ phông "Lora" vẫn còn trong bảng `name` của phông đã sửa đổi |

---

## RS-01 — Cao — Treo vô hạn với năm/JDN cực lớn (Rust và TS)

**Vị trí.** `core/src/lunar.rs:81–86` (`k_of_month_start`: hai vòng `while` hiệu chỉnh k không có cận);
`core/src/lunar.rs:193–212` (`jdn_to_lunar_with`); bản TS tương ứng `tsport/src/lunar.ts` (`kOfMonthStart`).

**Bằng chứng.** Bộ fuzz (`fuzz/`, bản release, hạn giờ 5 s mỗi lời gọi):

```
solar_to_lunar(2147483647,12,31) -> TIMEOUT
day_info(2147483647,12,31).to_json -> TIMEOUT
solar_to_lunar(-2147483648,1,1) -> TIMEOUT
lunar_to_solar(2147483647,1,1,false) -> TIMEOUT
tet(2147483647) -> TIMEOUT
jdn_to_lunar(9223372036854775807) -> TIMEOUT
jdn_to_lunar(-1000000000) -> TIMEOUT
```
TS (bundle hiện tại, hạn 10 s): `dayInfoJson(1e9,1,1)` → TIMEOUT; `dayInfoJson(2**53,1,1)` → TIMEOUT.

Nguyên nhân: với k rất lớn, `hnd_new_moon(k)` mất độ chính xác `f64` (hoặc thành `inf`/NaN), phép so sánh
trong `while` không bao giờ đổi chiều. Không có giới hạn số vòng.

**Tác động.** PWA hiện **không** bị (UI kẹp năm 2000–3000 ở mọi lối vào; đã kiểm `main.ts clamp`,
`convert.ts`, `gooddays.ts rangeFor`). Nhưng đây là API công khai của lõi; qua UniFFI, vòng lặp vô hạn
**không** được chuyển thành ngoại lệ như panic — luồng gọi phía Kotlin/Swift bị treo (MASVS-CODE).
`ffi/src/lib.rs:82–94` hiện chặn phạm vi trước khi gọi lõi — biện pháp đúng nhưng đặt sai tầng.

**Khuyến nghị.** Lõi tự kiểm phạm vi: thêm hằng `MIN_YEAR/MAX_YEAR` (hoặc phạm vi an toàn đã kiểm, ví
dụ 1000–9999) và trả `Option`/`Result` cho `solar_to_lunar`, `jdn_to_lunar`, `lunar_to_solar`, `tet`,
`day_info`. Tối thiểu: giới hạn số vòng trong `k_of_month_start` (ví dụ ≤ 4) rồi `panic!`/trả lỗi. Áp
dụng y hệt cho tsport.

## RS-02 — Cao — Kết quả sai lặng lẽ ngoài phạm vi, panic ở biên xa; Precise yếu hơn Hnd

**Vị trí.** `core/src/lunar.rs:120–170` (`YearTable::new`, `assert!` dòng 133, 147), `:193–212`
(`i -= 1` dòng 203 tràn `usize`), `core/src/ngochap`-cầu nối `dayinfo.rs:190–209`.

**Bằng chứng.**
```
solar_to_lunar(1000000,1,1) -> LunarDate { year: 1000001, month: 10, day: 167811, ... }   ← sai, không báo lỗi
jdn_to_lunar(1000000000)    -> LunarDate { year: 2733195, month: 10, day: 3265240, month_len: 26 }
solar_to_lunar_with(Precise,100000,1,1) -> PANIC: năm 13 tháng không có tháng nhuận: 100000
solar_to_lunar_with(Precise,1000000,1,1) -> PANIC: index out of bounds: the len is 14 but the index is 18446744073709551615
day_info(1000000,1,1).ngoc_hap  -> vẫn trả sao_xau=["Phi Ma Sát","Sát Chủ","Lỗ Ban Sát","Chu Tước Hắc Đạo"] (từ lunar_day bị ép u8)
```
Dò tuyến tính (bộ `scan/`, kiểm tính hợp lệ + khứ hồi tại 3 ngày mỗi năm):
```
Hnd:     lỗi đầu tiên đi lên: panic tại 35650-12-31;  đi xuống: panic tại -14621-1-1
Precise: lỗi đầu tiên đi lên: panic tại 31500-1-1;   đi xuống: panic tại -36200-12-31
```
TS: `dayInfoJson(35650,12,31)` → ném "năm 13 tháng không có tháng nhuận: 35651";
`dayInfoJson(-1e6,1,1)` → ném `TypeError: Cannot read properties of undefined (reading 'can')` (lỗi lập
trình lộ ra, không phải lỗi miền).

Lưu ý: trong khoảng 1…10 000 kết quả hợp lệ về **hình thức** (khứ hồi đúng), nhưng không có kiểm định
thiên văn ngoài 2000–3000; tài liệu nói rõ phạm vi 2000–3000 nhưng **mã không thực thi** điều đó.

**Tác động.** Một API lịch trả ngày âm "167 811" mà không báo lỗi là vi phạm tính đúng (T2). Với bản
gốc/thư viện, đầu vào từ người dùng (ví dụ ô năm không giới hạn trên ứng dụng gốc) có thể dẫn tới panic
hoặc kết quả vô nghĩa.

**Khuyến nghị.** Như RS-01: lõi từ chối năm ngoài phạm vi đã kiểm định. Thêm hậu kiểm rẻ trong
`jdn_to_lunar_with`: `debug_assert!`→kiểm tra thật `1 ≤ day ≤ month_len ∈ {29,30}`. Thay `i -= 1` bằng
vòng có cận (`checked_sub`).

## RS-03 — Trung bình — Panic với ngày dương không hợp lệ ở API công khai

**Vị trí.** `core/src/lunar.rs:220–226` (`solar_to_lunar_with` `assert!`), `core/src/dayinfo.rs:90`
(`day_ctx_with`), `core/src/gooddays.rs:160–168` (`find_good_days_with`), `core/src/lib.rs:34`
(`tet_with` `expect`).

**Bằng chứng.**
```
solar_to_lunar(2026,2,29) -> PANIC: ngày dương không hợp lệ: 2026-2-29
day_info(2026,13,1)       -> PANIC
find_good_days(cuoi_hoi, 2026-02-30..2026-03-01) -> PANIC: ngày dương không hợp lệ: 2026-2-30
```
Hành vi được ghi trong doc-comment ("Ngày dương không hợp lệ sẽ gây panic"), nên không phải lỗi ẩn —
nhưng hợp đồng §4 đã cho `lunar_to_solar` trả `Option`; hai chiều không đối xứng. TS ném `Error` (đúng
tương đương), PWA không gọi với ngày sai.

**Khuyến nghị.** Đổi chữ ký sang `Result<_, DateError>` trước khi khóa API cho UniFFI (thay đổi sau khi
có ứng dụng gốc sẽ tốn kém hơn). Nếu giữ panic, ghi vào hợp đồng §4 và đảm bảo mọi `#[uniffi::export]`
kiểm tra trước (hiện `ffi` có làm).

## RS-04 — Trung bình — Tràn số: debug panic, release gói quanh

**Vị trí và bằng chứng (bản debug, `dbg/`):**
```
core/src/canchi.rs:151:6  thanh_long_day_chi(0)      -> attempt to subtract with overflow   ((lm - 1) với lm = 0)
core/src/terms.rs:79:29   solar_terms_of_year(i32::MAX) -> attempt to add with overflow     (year + 1)
core/src/lunar.rs:203:9   (Precise, 1e6)             -> attempt to subtract with overflow   (i -= 1)
```
Bản release (không `overflow-checks`): `thanh_long_day_chi(0)` → 6 (sai lặng lẽ);
`solar_terms_of_year(i32::MAX)` → `len 0`; `canchi::month_can(0,0)` → 1 (u32 wrap rồi `% 10`);
`hour_range(100)` → `(7, 201)`. Các hàm `canchi::*` là `pub` nhưng không kiểm miền.
`canchi::can_chi_name(10,12)` → panic chỉ số mảng (cả hai bản).

**Khuyến nghị.** Đặt `overflow-checks = true` trong `[profile.release]` của workspace (chi phí không đáng
kể cho mã này; P1 dư 100×). Hoặc dùng `rem_euclid`/`checked_*` có chủ đích. Cân nhắc thu hẹp `pub` của
các hàm chỉ số nội bộ (`pub(crate)`) hoặc ghi rõ miền.

## RS-05 — Trung bình — `ngochap_ctx` ép `u32 → u8` mất thông tin

**Vị trí.** `core/src/dayinfo.rs:190–209`:
```rust
// Mọi chỉ số đều < 60 nên phép đổi sang u8 không mất thông tin.
let u = |x: u32| x as u8;
```
**Bằng chứng.** Chú thích sai với chính đầu vào hợp lệ về kiểu: `lunar_day` = 167 811 (RS-02) → `u8` =
`167811 mod 256`. `day_info(1000000,1,1)` vẫn trả danh sách sao xấu dựa trên giá trị đã bị cắt. Trong
2000–3000 giá trị thật đều nhỏ nên không sai — nhưng bất biến không được kiểm. Thêm nữa,
`lvhv-ngochap` tự `clamp(1,12)` tháng (`ngochap/src/lib.rs:167–169`) và `% 10`/`% 12` mọi chỉ số, nên
dữ liệu sai **không bao giờ báo lỗi** — bộ fuzz: `lunar_month=13` cho kết quả y hệt tháng 12; `lunar_month=0`
y hệt tháng 1; can/chi lệch chẵn lẻ (Ất Tý — không tồn tại) → `xung_tuoi = []` lặng lẽ.

**Khuyến nghị.** `u8::try_from(x).expect(...)` hoặc `debug_assert!(x < 60)`; ở `lvhv-ngochap`, xác thực
`DayCtx` một lần (`fn validate(&self) -> Result<(), _>`) thay vì kẹp lặng lẽ.

## RS-06 — Trung bình — tsport: kiểm tra kiểu đầu vào không nhất quán

**Vị trí.** `tsport/src/lunar.ts` (`jdnToLunar`, `lunarToSolar`), `tsport/src/jd.ts` (`isValidSolar`
có `Number.isInteger` — tốt).

**Bằng chứng (bundle hiện tại):**
```
jdnToLunar(1.5e6+0.5)          -> {"y":-606,"m":8,"d":15.5,...}     ← ngày âm không nguyên
lunarToSolar(2025,6,1,1)       -> {"y":2025,"m":7,"d":25}            ← leap = 1 bị ép thành true
lunarToSolar(2026,1,1,"yes")   -> null                               ← (đúng tình cờ)
tet(2026.5)                    -> THROW "tháng Giêng luôn tồn tại"   ← thông điệp sai bản chất
jdnToLunar(NaN)                -> THROW "bảng tháng bất thường quanh năm NaN"
```
`solarToLunar`/`dayInfo` kiểm `Number.isInteger` (tốt) nhưng `jdnToLunar`/`dayCtxJdn` thì không.

**Khuyến nghị.** Mọi hàm xuất khẩu của tsport kiểm `Number.isSafeInteger` và phạm vi ở đầu; `leap` kiểm
`typeof === "boolean"`. Giữ hành vi giống Rust (Rust không cho phép những kiểu này nên TS phải tự chặn).

## RS-07 — Trung bình — Không có quy trình phát hành có kiểm soát

**Bằng chứng.** Không có `.github/` hay CI nào trong kho (`ls .github` → không tồn tại); `git log` → chưa
có commit nào (mọi tệp untracked). `pwa/.gitignore` loại `dist/` (tốt — không commit sản phẩm build), nên
cách đưa `dist/` lên Pages chưa được định nghĩa.

**Tác động.** Mã chạy trên máy người dùng = mã ai đó đã đẩy lên hosting. CSP không bảo vệ trước mã độc
cùng origin. Đây là đường tấn công thực tế nhất (MO-HINH-DE-DOA T-1, T-3).

**Khuyến nghị (trước khi công khai URL, không nhất thiết trước G4 nội bộ).** CI: `npm ci --ignore-scripts`
(rồi chạy riêng `node node_modules/esbuild/install.js` nếu cần), `cargo test`, `tsport` so byte, `npm run
build`, triển khai bằng GitHub Actions `deploy-pages` từ nhánh được bảo vệ; bật 2FA khóa phần cứng; ghi
sha256 của `dist/` vào ghi chú phát hành. Soạn sẵn một `sw.js` "công tắc hủy" (unregister + xóa cache).

## RS-08 — Trung bình — Luồng cập nhật Service Worker

**Vị trí.** `pwa/src/sw.js` (`install` → `self.skipWaiting()`; `activate` → xóa cache `lvhv-*` khác
tên hiện tại rồi `clients.claim()`; `fetch` cache-first, `ignoreSearch: true`); `pwa/src/main.ts:283–286`
(chỉ `register`, không lắng nghe `updatefound`/`controllerchange`).

**Điểm tốt (đã kiểm).** Tên cache = `lvhv-v0.1-` + 10 hex đầu của sha256 trên mọi tệp precache
(`build.mjs:212–215`) → mọi thay đổi nội dung đổi `sw.js`, kích hoạt cập nhật. Phạm vi `./`. Chỉ xử lý
`GET` cùng origin. `install` dùng `cache: "reload"` (không lấy từ HTTP cache cũ).

**Vấn đề.**
1. Trang đang mở giữ `app-OLD.js` trong bộ nhớ; SW mới `activate` xóa ngay cache cũ và `claim()`. Nếu mã
   cũ tải lười một tài nguyên cũ (hiện chưa có, nhưng phông chữ italic là ví dụ: tải khi cần) → 404 trên
   mạng / lỗi khi offline. Với gói đơn tệp hiện tại rủi ro thấp.
2. Không báo người dùng có bản mới; bản sửa lỗi tính toán (T2) chỉ có hiệu lực ở lần mở thứ hai.
3. Kiểm chứng bằng trình duyệt (`probe.mjs`): sửa `index.html` trên máy chủ mà **không** đổi `sw.js` → hai
   lần tải lại vẫn nhận `index.html` cũ (`redeployedHtmlServedReload1/2: false`). Đúng thiết kế cache-first,
   nhưng nghĩa là mọi sửa nóng trên máy chủ ngoài `build.mjs` đều vô hiệu — nên ghi vào quy trình phát hành.
4. `ignoreSearch: true` làm `?v=…` vô tác dụng (thông tin).

**Khuyến nghị.** Bỏ `skipWaiting()` tự động; hiện thông báo "Có bản mới — Tải lại" khi `registration.waiting`
xuất hiện, gửi `postMessage("skipWaiting")` khi người dùng đồng ý; hoặc giữ cache cũ đến khi mọi client cũ
đóng.

## RS-09 — Thấp — CSP: kiểm tra bản build thực

**Bằng chứng** (`pwa/dist/index.html` dựng 03:02:29, sha256 `6741770e…`):
```
<meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self'; style-src 'self';
 img-src 'self' data:; font-src 'self'; connect-src 'self'; manifest-src 'self'; worker-src 'self';
 object-src 'none'; base-uri 'self'; form-action 'none'">
```
- Không có `<script>` nội tuyến, không `<style>`, không thuộc tính `style=` trong `index.html` (grep: 0).
- Gói JS không dùng `.style.` hay `setAttribute("style")` (grep `dist/app-*.js`: 0) → `style-src 'self'`
  không bị vi phạm. Không `eval`, `new Function`, `innerHTML`, `insertAdjacentHTML`, `document.write`.
- Chỉ URL tuyệt đối trong gói: `http://www.w3.org/2000/svg` (không gian tên, không phải yêu cầu mạng)
  và các URL **nguồn ca dao / Ngọc Hạp hiển thị dạng chữ** (voh.com.vn, wikiquote…) — không có
  `href`/`src` nào trỏ tới chúng (grep `href:` trong mã: chỉ `#/…` và `RULES_HREF` nội bộ).
- CSS: chỉ `url("./fonts/…woff2")`, không `@import`.
- Trình duyệt thật: 0 sự kiện `securitypolicyviolation`, 0 lỗi console.

**Vấn đề còn lại.** `frame-ancestors` bị bỏ qua khi đặt trong `<meta>` (đặc tả CSP3) → trang có thể bị
nhúng iframe trên trang khác (clickjacking). Tác động thấp (không có hành động nhạy cảm), nhưng tiêu chí S3
"CSP chặt" nên ghi rõ giới hạn này. `img-src data:` cần thiết? — `dom.ts svg()` tạo SVG bằng DOMParser, không
dùng `data:`; nếu không có nơi nào dùng thì bỏ `data:`.

**Khuyến nghị.** Thêm `require-trusted-types-for 'script'` (gói không dùng sink HTML nên không vỡ); khi có
hosting cho phép tiêu đề HTTP: đặt CSP + `frame-ancestors 'none'` + `X-Content-Type-Options: nosniff` qua tiêu đề.

## RS-10 — Trung bình — Chung origin trên GitHub Pages

**Bằng chứng.** `build.mjs` dùng đường dẫn tương đối và `manifest.webmanifest` có `scope: "./"`, `id: "./"` —
thiết kế cho phép chạy ở đường dẫn con `https://<user>.github.io/<repo>/`. Mọi kho Pages của cùng một tài
khoản dùng chung origin `https://<user>.github.io`: chung `localStorage`, Cache Storage, IndexedDB; script
của một ứng dụng đọc/xóa được dữ liệu của ứng dụng kia, và đăng ký được SW cho đường dẫn của ứng dụng kia
(SW scope chỉ giới hạn theo vị trí tệp `sw.js`, trừ khi có `Service-Worker-Allowed`). Chủ dự án đã có ít
nhất một PWA khác dự kiến trên GitHub Pages.

**Khuyến nghị.** Phát hành trên **origin riêng**: tên miền riêng (`lich.clevix.vn` chẳng hạn) có xác minh
tên miền GitHub, hoặc kho `<org>.github.io` riêng cho ứng dụng này. Quyết định cần chủ dự án duyệt.

## RS-11 — Thấp — Nội dung → DOM

**Bằng chứng.** `pwa/src/dom.ts`: `h()` dùng `createElement`, `setAttribute(k, String(v))`, `el.append(string)`
(chèn nút văn bản). Không có `innerHTML` cho dữ liệu; `svg()` (dòng 30–33) dùng `DOMParser` nhưng chỉ nhận
hằng trong `icons.ts`. `cadao.json`: 7/427 mục chứa `<`, `>`, `&` (trong lời ghi nguồn) — hiển thị an toàn
dạng chữ. `build.mjs` nhúng nội dung bằng `JSON.stringify` vào tệp `.ts` sinh ra → không có đường chèn mã
(JSON.stringify thoát `"`/`\`; esbuild xử lý `</script>` không liên quan vì là tệp JS riêng). Một rủi ro nhỏ:
`setAttribute` với khóa bắt đầu bằng `on` nhưng giá trị không phải hàm sẽ được đặt làm thuộc tính sự kiện
dạng chuỗi — hiện không có lời gọi nào truyền dữ liệu nội dung làm tên thuộc tính, và CSP chặn trình xử lý
nội tuyến.

**Khuyến nghị.** Trong `h()`, từ chối khóa `on*` có giá trị không phải hàm; trong `loadCadao`, giới hạn độ dài
dòng và loại ký tự điều khiển (U+0000–U+001F trừ `\n`), URL nguồn phải `https:`.

## RS-12 — Thấp — Kiểm tra L7 so với tệp vàng do chính lõi sinh

**Bằng chứng.** `tsport/test/compare.mjs` so với `core/out/dayinfo_2000_3000.jsonl.gz` (24,6 MB, sinh lúc
02:56 bởi `lvhv-dump`). Đây là phép so **Rust↔TS**, đúng mục đích L7; nhưng tệp vàng phải được sinh lại mỗi khi
lõi đổi, nếu không phép so sẽ dùng tệp cũ (ví dụ trước 02:56 tệp này chưa có Ngọc Hạp). Không có gì ép buộc
việc sinh lại.

**Kiểm chứng độc lập của người duyệt** (sinh mới từ lõi hiện tại, so với bundle tsport hiện tại):
```
parity default (hnd): lines=36890 mismatches=0     (2000-01-01..2100-12-31, gồm ngoc_hap)
parity precise:       lines=36890 mismatches=0
goodDays parity (2026, cuoi_hoi / an_tang 1976 / khai_truong 1985): rust=314 ts=314 mismatches=0
```
(2101–3000 không chạy lại trong đợt duyệt này.)

**Khuyến nghị.** Trong CI: build `lvhv-dump` → sinh JSONL vào thư mục tạm → so byte với tsport; không đọc tệp
vàng đã commit. Xem DUYET-MA §4.

## RS-13 — Thấp — `find_good_days` không giới hạn khoảng

**Vị trí.** `core/src/gooddays.rs:153–180`. **Bằng chứng.** `find_good_days("cuoi_hoi", 2000-01-01, 3000-12-31)`
→ 74 927 kết quả, **92 s**, bộ nhớ tỉ lệ thuận. PWA giới hạn 366 ngày (`gooddays.ts:17,32`), `ffi` cũng giới
hạn `MAX_SPAN_DAYS = 366` (`ffi/src/lib.rs:30,230`) — lõi thì không. P2 (90 ngày): 0,69 ms — đạt rộng.

**Khuyến nghị.** Đưa `MAX_SPAN_DAYS` vào lõi, trả lỗi nếu vượt.

## RS-14 — Thông tin — Phụ thuộc npm

**Lệnh và kết quả** (`pwa/`):
```
$ npm audit --omit=dev     → found 0 vulnerabilities
$ npm audit                → found 0 vulnerabilities
$ npm ls --all (rút gọn)
lich-van-hoa-viet-pwa@0.1.0
+-- esbuild@0.28.2 (+ @esbuild/linux-x64@0.28.2; 25 gói nền tảng khác UNMET OPTIONAL)
+-- playwright-core@1.63.0
`-- typescript@7.0.2
```
Không có phụ thuộc runtime (`dependencies` rỗng) → `--omit=dev` không có gì để kiểm; gói phát hành không chứa
mã bên thứ ba ngoài phần esbuild sinh ra. `package-lock.json`: mọi gói có `integrity` sha512, `resolved` từ
`registry.npmjs.org`. Giấy phép: esbuild MIT, typescript Apache-2.0, playwright-core Apache-2.0 — tương thích.
`esbuild` có `postinstall: "node install.js"` (tải/kiểm nhị phân nền tảng) — điểm chạy mã khi cài đặt.
`npm audit` chỉ phát hiện lỗ hổng **đã công bố**; không chứng minh gói sạch.

## RS-15 — Thấp — `tsport/` không có lockfile

`tsport/package.json` không có phụ thuộc, dùng `../pwa/node_modules/.bin/tsc` và `esbuild` của `pwa/`.
`npm ls` → `(empty)`; `npm audit --omit=dev` → `ENOLOCK`. Rủi ro thực tế thấp (không có phụ thuộc riêng),
nhưng phiên bản công cụ build tsport phụ thuộc ngầm vào `pwa/package-lock.json`. Ghi rõ trong README tsport
hoặc thêm lockfile rỗng.

## RS-16 — Thấp — Bằng chứng hiệu năng đo trên bản mock

`pwa/test/metrics.json` (ngày `2026-11-05T17:43:32Z`, `first_load_transfer_gzip_kb: 229.7`) ứng với bản dựng
00:43 có `ENGINE_SOURCE = "mock"` (người duyệt xác nhận: `pwa/src/gen/content.ts` lúc đó ghi `"mock"`, gói
`app-3PJRK7RE.js` 38 KB có nhãn "MOCK — dữ liệu giả lập"). Bản dựng hiện tại dùng tsport, `app-GSOED2I2.js`
431 KB. Ước lượng của người duyệt (gzip -9 trên tệp, không phải đo mạng): HTML+JS+CSS+SW+manifest ≈ 63,7 KB +
phông 79,6 KB (woff2 đã nén) + biểu tượng → vẫn dưới 500 KB, nhưng **P3/P4/P5 phải đo lại** trên bản build
cuối trước khi kết luận.

## RS-17 — Thông tin — Kiểm chứng S3/S4 bằng trình duyệt

`probe.mjs` (máy chủ HTTP cục bộ phục vụ bản sao `pwa/dist/` 03:02:29; Chrome thật; ghi lại mọi yêu cầu):
duyệt `#/ngay`, `#/chon-ngay` (chọn việc, nhập năm sinh 1976), `#/doi-ngay`, `#/gioi-thieu`, đổi cỡ chữ và
giao diện:
```
external: []                         consoleErrors: []        csp violations: []
localStorage (trước khi đổi tùy chọn): {}
localStorage (sau khi đổi):  {"lvhv.ui":"{\"size\":\"2\",\"theme\":\"light\"}"}
sessionStorage: []   cookie: ""   indexedDB: []   caches: ["lvhv-v0.1-3ba9c3b255"]   sw: true
goodDaysSummary: "Cưới hỏi, từ 06/11/2026 đến 30/11/2026, tuổi Bính Thìn: 8 ngày không phạm sao kỵ chính."
offline reload: trang vẫn mở
```
Năm sinh không xuất hiện trong bất kỳ kho lưu trữ nào.

## RS-18 — Thông tin — Lời cam kết quyền riêng tư

Màn Giới thiệu: "không gửi thông tin đi đâu". Đúng với mã ứng dụng; nhưng lần tải đầu và mỗi lần kiểm tra
cập nhật `sw.js`, máy chủ hosting (GitHub) nhận IP/User-Agent. Đề xuất thêm một câu: "Máy chủ lưu trữ trang
(GitHub Pages) có thể ghi nhật ký truy cập thông thường như mọi trang web."

## RS-19 — Thông tin — Kích thước gói

`app-GSOED2I2.js` 430 772 byte; URL nguồn ca dao lặp lại (ví dụ `voh.com.vn/song-dep/tuc-ngu-viet-nam-379232.html`
×173). Nén gzip xử lý tốt (56 KB), nhưng thời gian phân tích JS trên máy yếu tăng. Đề xuất: bảng nguồn
khử trùng lặp, mỗi câu tham chiếu chỉ số.

## RS-20 — Thấp — Giấy phép

| Thành phần | Giấy phép | Nhận xét |
|---|---|---|
| lvhv-core, lvhv-ngochap, lvhv-ffi | MIT (khai báo trong Cargo.toml) | **Không có tệp LICENSE** ở gốc kho hay trong crate — MIT yêu cầu kèm văn bản giấy phép và dòng bản quyền khi phân phối |
| Phông Lora (đã rút gọn) | SIL OFL 1.1 | `OFL.txt` có trong `dist/fonts/`; ghi chú nói "Reserved Font Name không được dùng" và CSS khai báo `font-family "LVHV Serif"`. **Nhưng** bảng `name` bên trong tệp woff2 vẫn ghi họ phông `Lora` / `Lora-Regular` / `Lora Italic` (fontTools: nameID 1/4/6). OFL §3 cấm phiên bản sửa đổi dùng Reserved Font Name. Rút gọn (subset) là sửa đổi. Cần đổi tên trong bảng `name` khi rút gọn (fontTools `subset` không tự đổi) — hoặc dựa vào FAQ của OFL về "subsetting cho web" nếu người giữ bản quyền đã cấp phép rõ cho việc đó (cần kiểm nguồn) |
| cadao.json | CC BY-SA 4.0 (tự áp, thận trọng) | Ghi công hiển thị ở Giới thiệu (`about.ts:41–42`, lấy từ SOURCES.md §C) — đạt nghĩa vụ BY. SA: dữ liệu nhúng trong gói JS MIT; SOURCES.md lập luận đây là "tập hợp" — chấp nhận được, nhưng nên phát hành kèm tệp dữ liệu riêng hoặc ghi rõ trong LICENSE rằng phần dữ liệu trong gói theo CC BY-SA |
| Nguồn nhóm B (VOH, thohay, luatminhkhue…) | Không có giấy phép mở | Chỉ dùng câu dân gian; app hiển thị URL của họ dạng chữ — không vấn đề |
| Crate của `ffi` (46 crate) | MIT/Apache-2.0 phần lớn; **uniffi\* MPL-2.0** (8 crate); 1 crate Unicode-3.0 | MPL-2.0 là copyleft mức tệp: tương thích với ứng dụng MIT/đóng nếu không sửa tệp của uniffi; phải cung cấp mã nguồn tệp MPL nếu sửa, và ghi chú giấy phép trong ứng dụng gốc. Không vào PWA |
| npm dev | MIT, Apache-2.0 | Không vào gói phát hành |

---

## Phụ lục A — lệnh S1/S2 (bằng chứng nguyên văn)

```
$ grep -rn "unsafe" --include=*.rs core ngochap
core/src/lib.rs:3://! Không phụ thuộc thư viện ngoài, không dùng `unsafe`. ...
core/src/lib.rs:5:#![forbid(unsafe_code)]
core/src/bin/lvhv-dump.rs:2:#![forbid(unsafe_code)]
ngochap/src/lib.rs:15:#![forbid(unsafe_code)]

$ cargo tree -p lvhv-core -e normal,build,dev --offline        (workspace gốc, lúc ~03:05)
lvhv-core v0.1.0 (/home/claude/lich-van-hoa-viet/core)
└── lvhv-ngochap v0.1.0 (/home/claude/lich-van-hoa-viet/ngochap)

$ cargo tree -p lvhv-ngochap -e normal,build,dev --offline
lvhv-ngochap v0.1.0 (/home/claude/lich-van-hoa-viet/ngochap)
```
(Lần chạy trước khi tích hợp, ~01:10: `cargo tree` của `core/` chỉ có `lvhv-core v0.1.0`.)

## Phụ lục B — bộ kiểm thử của dự án (người duyệt chạy lại, bản release)

```
lvhv-core  unit 14 passed; fixtures 7 passed (1 ignored); methods 2 passed (3 ignored); ngochap 7 passed (2 ignored)
lvhv-ngochap unit 27 passed (2 ignored)
```
Các bài `#[ignore]` (khứ hồi 2000–3000, N3 toàn dải) — xem BIEN-BAN-S1-S4 và DUYET-MA §3.
