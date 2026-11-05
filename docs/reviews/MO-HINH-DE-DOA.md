# Mô hình đe dọa (STRIDE) — Lịch và Văn hóa Việt v0.1

Vai: A9 (bảo mật), người duyệt độc lập — không tham gia viết mã được duyệt.
Ngày: 2026-11-06. Phạm vi: trạng thái thư mục làm việc lúc duyệt (bộ tích hợp đang nối `ngochap`
vào core/tsport/pwa song song; xem "Ảnh chụp trạng thái" cuối tài liệu).
Phát hiện cụ thể kèm mã số (RS-xx) nằm ở `RA-SOAT-BAO-MAT.md`.

## 1. Hệ thống được mô hình hóa

PWA tĩnh, chạy offline, không có tài khoản, không có máy chủ ứng dụng. Phát hành bằng cách chép
`pwa/dist/` lên một máy chủ tĩnh (dự kiến GitHub Pages). Mọi tính toán (âm lịch, Ngọc Hạp, chọn
ngày) chạy trong trình duyệt bằng `tsport` (bản TypeScript của lõi Rust) đã được gói vào
`app-<hash>.js`. Ca dao (427 câu) và bảng nguồn Ngọc Hạp được nhúng vào gói JS lúc build.

```
[Nhà phát triển + máy build] --(git, npm, cargo)--> [dist/] --(đẩy)--> [Máy chủ tĩnh / GitHub Pages]
        ^ npm registry, crates.io                                        |  HTTPS
                                                                         v
                                       [Trình duyệt người dùng: SW + Cache Storage + localStorage]
```

## 2. Tài sản

| Mã | Tài sản | Vì sao quan trọng |
|---|---|---|
| T1 | Tính toàn vẹn của mã chạy trên máy người dùng (`app-*.js`, `sw.js`, `index.html`) | Mã độc trong gói = toàn quyền trên nguồn gốc (origin) của ứng dụng; SW làm nó tồn tại lâu |
| T2 | Tính đúng của kết quả lịch / chọn ngày | Người dùng ra quyết định đời sống (cưới, an táng) theo kết quả; sai lệch lặng lẽ là tác hại thật |
| T3 | Toàn vẹn nội dung (ca dao, nguồn dẫn, lời ghi công CC BY-SA) | Nội dung sai/bị chèn làm mất uy tín; thiếu ghi công vi phạm giấy phép |
| T4 | Quyền riêng tư người dùng (năm sinh nhập vào, tùy chọn giao diện) | Tiêu chí S4; năm sinh là dữ liệu cá nhân |
| T5 | Danh tiếng / nhận diện Clevix Lab (tên, biểu tượng, URL phát hành) | Bị giả mạo bằng bản sao có mã độc |
| T6 | Tính sẵn sàng offline | Tiêu chí P5; SW hỏng có thể "khóa" người dùng vào bản lỗi |

## 3. Ranh giới tin cậy

| Mã | Ranh giới | Ghi chú |
|---|---|---|
| B1 | Registry (npm, crates.io) → máy build | devDependencies: esbuild 0.28.2 (có `postinstall: node install.js`), typescript 7.0.2, playwright-core 1.63.0. Crate `ffi` kéo 46 crate ngoài (uniffi, serde…), không vào PWA |
| B2 | Kho mã / thư mục nội dung → build | `content/cadao/cadao.json`, `content/ngochap/SOURCES.md`, `core/src/lunar.rs` được `build.mjs` đọc và nhúng; nội dung do tác nhân/người khác soạn |
| B3 | Máy build → máy chủ tĩnh | Quy trình đẩy chưa định nghĩa (không có CI trong kho, không có `.github/`) |
| B4 | Máy chủ tĩnh → trình duyệt | HTTPS do GitHub Pages cung cấp; không đặt được tiêu đề HTTP tùy ý (CSP chỉ có thể qua `<meta>`) |
| B5 | Trang ↔ Service Worker ↔ Cache Storage | SW cache-first, phạm vi `./` |
| B6 | Người dùng → ô nhập (ngày, năm sinh, hash URL `#/ngay/YYYY-MM-DD`) | Đầu vào không tin cậy đi vào lõi |

## 4. Mối đe dọa theo STRIDE

Mức: khả năng × tác động, đánh giá cho v0.1 nội bộ. "Có" = biện pháp hiện có và đã kiểm chứng;
"Thiếu" = chưa có.

### S — Giả mạo (Spoofing)

| # | Mối đe dọa | Biện pháp hiện có | Thiếu | Rủi ro còn lại |
|---|---|---|---|---|
| S-1 | Bản sao (mirror) trên tên miền khác mang tên "Lịch và Văn hóa Việt", biểu tượng Clevix, thêm mã theo dõi/quảng cáo | Không có gì kỹ thuật ngăn được (mã nguồn công khai, MIT). `manifest.id = "./"` gắn với origin nên bản sao là ứng dụng khác | Không có kênh xác thực bản chính thức (URL chuẩn công bố, chữ ký phát hành, trang "bản chính thức ở đâu") | Trung bình. Với ứng dụng miễn phí và không có tài khoản, thiệt hại chủ yếu là danh tiếng và theo dõi người dùng của bản sao |
| S-2 | Chiếm tên miền phụ GitHub Pages (tài khoản/tổ chức bị đổi tên, kho bị xóa rồi người khác lấy lại tên) | — | Quy trình giữ tên tài khoản/tổ chức; tên miền riêng có xác minh (GitHub "verified domains") | Thấp–Trung bình |
| S-3 | Nội dung giả mạo nguồn (cadao ghi nguồn VOH/Wikiquote nhưng câu chữ không khớp) | `build.mjs` bắt buộc mỗi mục có `sources` không rỗng (C1) | Không kiểm chứng câu chữ với nguồn (SOURCES.md §D tự thừa nhận chỉ khớp phần mở đầu) | Thấp về bảo mật; là vấn đề C1/uy tín nội dung |

### T — Can thiệp (Tampering)

| # | Mối đe dọa | Biện pháp hiện có | Thiếu | Rủi ro còn lại |
|---|---|---|---|---|
| T-1 | **Chuỗi cung ứng dev**: gói npm bị chiếm (esbuild có postinstall chạy mã khi `npm install`; typescript 7 tải nhị phân nền tảng) chèn mã vào `dist/` | `package-lock.json` có `integrity` sha512 cho mọi gói; phiên bản ghim chính xác; `npm audit` 0 lỗ hổng đã biết (RS-14) | `npm ci --ignore-scripts` trong quy trình build; build tái lập được (so băm `dist/` giữa hai máy); xét duyệt khi nâng phiên bản; tsport không có lockfile (RS-15) | Trung bình. Đây là đường tấn công thực tế nhất vào T1 |
| T-2 | Chuỗi cung ứng Rust: crate `ffi` kéo 46 crate (uniffi MPL-2.0, serde…); một trong số đó có `build.rs`/proc-macro chạy lúc build | Lõi `lvhv-core` chỉ phụ thuộc `lvhv-ngochap` (nội bộ, đường dẫn); `Cargo.lock` ghim | `cargo-audit`/`cargo-deny` không có trong môi trường; chưa có chính sách giấy phép/nguồn | Thấp cho v0.1 PWA (ffi không vào PWA); Trung bình cho bản gốc (native) |
| T-3 | **Triển khai bị xâm nhập** (token đẩy Pages lộ, tài khoản GitHub bị chiếm) → `sw.js` + `app.js` độc | HTTPS; CSP chặn tải script ngoài nguồn gốc nhưng mã độc **cùng nguồn gốc** thì CSP không ngăn | 2FA/khóa phần cứng cho tài khoản phát hành; nhánh `gh-pages` được bảo vệ; triển khai chỉ từ CI có xét duyệt; công bố băm của bản phát hành | **Cao về tác động**. Trình duyệt tự kiểm tra lại tệp script của SW đã đăng ký (khi điều hướng, tối đa mỗi 24 h, bỏ qua HTTP cache), nên nếu kẻ tấn công chỉ thay nội dung `sw.js` thì đẩy lại `sw.js` sạch là khôi phục được. Nhưng nếu mã độc đăng ký SW ở **đường dẫn khác** (ví dụ `sw2.js`) và bản sạch không có tệp đó (404), trình duyệt giữ nguyên SW độc; khôi phục khi đó đòi hỏi phục vụ một SW "công tắc hủy" ở đúng đường dẫn ấy. Dữ liệu đã bị lấy trong thời gian bị xâm nhập thì không khôi phục được. Xem RS-07 |
| T-4 | **Cache SW cũ / ngộ độc cache**: bản lỗi hoặc bản độc nằm trong Cache Storage; người dùng không bao giờ thấy bản sửa nếu `sw.js` không đổi | Tên cache = băm sha256 của toàn bộ tệp precache → mọi thay đổi nội dung đổi `sw.js`; `activate` xóa cache cũ có tiền tố `lvhv-`; `install` dùng `cache: "reload"` | Thông báo "có bản mới"; nút làm mới; `skipWaiting()` + `clients.claim()` làm trang đang mở chạy **lẫn** JS cũ với SW mới (RS-08); `ignoreSearch: true` | Thấp–Trung bình. Đã thử: sửa `index.html` trên máy chủ mà không đổi `sw.js` → trình duyệt tiếp tục nhận bản cũ (đúng thiết kế cache-first; chỉ an toàn nếu mọi lần triển khai đều qua `build.mjs`) |
| T-5 | Nội dung độc trong `cadao.json`/`SOURCES.md` (chuỗi chứa `<script>`, `javascript:` URL) → XSS | Mọi nội dung đi vào DOM bằng `textContent`/`append(string)` qua `h()`; không có `innerHTML` cho dữ liệu; URL nguồn chỉ hiển thị dạng chữ, không gắn `href`; CSP `script-src 'self'` không `unsafe-inline` | Kiểm tra lược đồ nội dung chặt hơn (độ dài, ký tự điều khiển) | Thấp. Xem RS-11 |
| T-6 | Sửa `localStorage` (`lvhv.ui`) bằng tiện ích trình duyệt | `prefs.ts` chỉ nhận giá trị trong danh sách trắng; lỗi JSON → mặc định | — | Rất thấp |
| T-7 | Sai lệch Rust ↔ TS (tsport là bản chép tay): người dùng PWA thấy kết quả khác ứng dụng gốc | `tsport/test/compare.mjs` so từng byte với tệp vàng; người duyệt chạy lại 2000–2100 cả hai phương pháp: 0 sai lệch, `find_good_days` 2026 (314 dòng): 0 sai lệch | CI bắt buộc; tệp vàng trong `core/out/` do chính lõi sinh nên phải sinh lại mỗi lần đổi lõi (RS-12, DUYET-MA §4) | Trung bình về T2 |

### R — Chối bỏ (Repudiation)

| # | Mối đe dọa | Ghi chú |
|---|---|---|
| R-1 | Không xác định được bản nào đã được phát hành và bởi ai | Không có nhật ký phát hành, không có thẻ git/băm công bố; kho chưa có commit. Với v0.1 không có tài khoản nên R chủ yếu liên quan đến quy trình phát hành (T-3). Khuyến nghị: thẻ git ký + ghi băm `dist/` vào ghi chú phát hành |

### I — Lộ thông tin (Information disclosure)

| # | Mối đe dọa | Biện pháp hiện có | Thiếu | Rủi ro còn lại |
|---|---|---|---|---|
| I-1 | Năm sinh bị lưu/gửi đi | Năm sinh chỉ giữ trong biến bộ nhớ (`st.birth`), không ghi `localStorage`; không có `fetch`/XHR/beacon nào trong gói. Đã kiểm chứng bằng trình duyệt (RS-17) | — | Rất thấp |
| I-2 | Rò rỉ qua Referer khi người dùng bấm liên kết ngoài | `<meta name="referrer" content="no-referrer">`; URL nguồn hiển thị dạng chữ, không có liên kết ngoài | — | Rất thấp |
| I-3 | Máy chủ tĩnh (GitHub) ghi nhật ký IP, User-Agent, thời điểm tải | Không tránh được với Pages; sau lần đầu SW phục vụ từ cache nên ít yêu cầu | Ghi rõ trong mục "Quyền riêng tư" rằng nhà cung cấp hosting có nhật ký truy cập | Thấp — nhưng câu "không gửi thông tin đi đâu" ở màn Giới thiệu nên nói chính xác hơn (RS-18) |
| I-4 | `window.__lvhv` (chỉ số render) lộ trên đối tượng toàn cục | Không chứa dữ liệu cá nhân | — | Thông tin |

### D — Từ chối dịch vụ (Denial of service)

| # | Mối đe dọa | Biện pháp hiện có | Thiếu | Rủi ro còn lại |
|---|---|---|---|---|
| D-1 | Đầu vào ngoài phạm vi làm treo/đứng tab (năm rất lớn) | PWA chặn năm ngoài 2000–3000 ở mọi lối vào (hash URL qua `clamp`, ô nhập qua kiểm tra MIN/MAX_YEAR); khoảng chọn ngày ≤ 366 ngày | **Lõi** (Rust và TS) không tự giới hạn: `day_info(i32::MAX,…)`, `jdnToLunar(1e9)` treo vô hạn; năm ≥ 35650 panic (RS-01, RS-02) | Thấp cho PWA hiện tại (lớp UI chắn); **Cao** cho bất kỳ lời gọi trực tiếp nào tới lõi (bản gốc, thư viện) |
| D-2 | SW hỏng → ứng dụng không mở được offline / kẹt bản lỗi | `install` lỗi thì SW mới không kích hoạt, SW cũ tiếp tục | Không có "công tắc hủy" (kill-switch) `sw.js` soạn sẵn | Thấp |
| D-3 | Dung lượng: gói JS tăng từ 38 KB lên 431 KB (56 KB gzip) sau khi nhúng nguồn ca dao (URL lặp hàng trăm lần) | P3 vẫn đạt (< 500 KB gzip) | Khử trùng lặp URL nguồn (RS-19) | Thông tin |

### E — Nâng quyền (Elevation of privilege)

| # | Mối đe dọa | Biện pháp hiện có | Thiếu | Rủi ro còn lại |
|---|---|---|---|---|
| E-1 | XSS → chạy mã trên origin, đăng ký SW độc | CSP `default-src 'self'; script-src 'self'; object-src 'none'; base-uri 'self'; form-action 'none'`; không `eval`, không `innerHTML` dữ liệu; `svg()` chỉ phân tích chuỗi SVG tĩnh trong mã | `frame-ancestors` **không có hiệu lực trong `<meta>`** → chống clickjacking phải dựa vào tiêu đề HTTP mà GitHub Pages không cho đặt; `require-trusted-types-for 'script'` chưa dùng; `upgrade-insecure-requests` không cần (cùng origin) | Thấp. Xem RS-09 |
| E-2 | Dùng chung origin với ứng dụng khác của cùng chủ (ví dụ `user.github.io/lich/` và `user.github.io/quet-tai-lieu/`) → chung `localStorage`, Cache Storage, và một SW có phạm vi rộng hơn có thể chặn yêu cầu của ứng dụng kia | SW đăng ký phạm vi `./`; `activate` chỉ xóa cache có tiền tố `lvhv-` | Phát hành trên **origin riêng** (tên miền riêng hoặc kho `*.github.io` riêng) | **Trung bình** nếu triển khai dạng đường dẫn con của một trang Pages dùng chung. Xem RS-10 |

## 5. Rủi ro còn lại — tóm tắt cho v0.1

1. Cao nhất: xâm nhập kênh phát hành (T-3) và chuỗi cung ứng npm (T-1). Không có CI, không có quy trình
   phát hành có kiểm soát. Biện pháp kỹ thuật trong ứng dụng (CSP, SW) **không** bảo vệ được trước mã độc
   cùng origin.
2. Chia sẻ origin trên GitHub Pages (E-2) — quyết định triển khai, cần chốt trước G4.
3. Lõi không chống đầu vào xấu (D-1) — hiện được lớp UI che, sẽ lộ ngay khi lõi được gọi qua UniFFI
   hoặc dùng làm thư viện (lớp `ffi/` đang có kiểm tra phạm vi riêng — tốt, nhưng là kiểm tra trùng lặp
   ở lớp ngoài, không phải ở lõi).
4. Tính đúng (T2): sai lệch Rust↔TS đã kiểm 0 trên phần chạy lại; rủi ro nằm ở quy trình (không có CI).

## 6. Hướng tới v0.2 (tài khoản, nhật ký mã hóa đầu-cuối) — tóm tắt

Thay đổi về bản chất: v0.2 có **máy chủ**, **danh tính**, **dữ liệu nhạy cảm** (ngày giỗ, nhật ký).
Phần lớn biện pháp của v0.1 không còn đủ.

- Mã hóa đầu-cuối trong web là bài toán khó: máy chủ phục vụ chính mã JS làm việc mã hóa. Ai kiểm soát
  việc phát hành JS thì kiểm soát được khóa (T-3 trở thành **Nghiêm trọng**). Cần: tách origin ứng dụng
  với origin API; khóa dẫn xuất từ mật khẩu bằng Argon2id (tham số công bố) hoặc khóa thiết bị +
  khóa khôi phục; WebCrypto, khóa không xuất được (`extractable: false`) khi có thể; xem xét phân phối
  bản web E2EE qua gói đã ký (ví dụ Web Bundles/đóng gói gốc) hoặc chỉ cung cấp E2EE trên ứng dụng gốc.
- Chế độ "tiêu chuẩn" và "E2EE" phải hiển thị rõ khác biệt khôi phục: quên mật khẩu ở E2EE = mất dữ liệu.
- Dữ liệu ngày giỗ/nhật ký trong IndexedDB phải được mã hóa ở trạng thái nghỉ nếu là chế độ E2EE.
- CSP phải đặt bằng **tiêu đề HTTP** (cần hosting khác GitHub Pages hoặc CDN phía trước) để có
  `frame-ancestors 'none'`, báo cáo vi phạm (`report-to`), và HSTS.
- Xác thực: WebAuthn/passkey ưu tiên; phiên bằng cookie `HttpOnly; Secure; SameSite=Strict` trên origin API.
- Mô hình đe dọa riêng cho máy chủ (OWASP ASVS L2) — ngoài phạm vi tài liệu này.

## 7. Ứng dụng gốc (lõi Rust qua UniFFI) — ánh xạ OWASP MASVS v2.1

Đề xuất hồ sơ: **MAS-L1** cho bản chỉ có lịch (giống v0.1); **MAS-L2** khi có tài khoản + nhật ký E2EE
(dữ liệu nhạy cảm, có khóa mật mã). MASVS-RESILIENCE (chống dịch ngược, MAS-R) không cần — ứng dụng
miễn phí, mã nguồn mở, không có bí mật phía khách.

| Nhóm MASVS | Áp dụng | Hiện trạng / việc cần làm |
|---|---|---|
| MASVS-STORAGE | L1: chỉ tùy chọn giao diện. L2: nhật ký, ngày giỗ, khóa | Dùng Keystore (Android) / Keychain (iOS/macOS) / DPAPI (Windows) cho khóa; không ghi năm sinh/nhật ký vào log, ảnh chụp đa nhiệm, sao lưu đám mây không mã hóa |
| MASVS-CRYPTO | L2 | Không tự chế; dùng thư viện đã kiểm định (ví dụ `ring`/`RustCrypto` có kiểm định, hoặc API nền tảng); Argon2id; nonce không lặp. Lõi lịch hiện không có mật mã — tốt |
| MASVS-AUTH | L2 | Passkey; khóa sinh trắc chỉ để mở khóa khóa cục bộ, không thay xác thực máy chủ |
| MASVS-NETWORK | L1: không mạng. L2: API | TLS 1.2+; không tắt kiểm tra chứng chỉ; cân nhắc ghim khóa công khai cho API riêng |
| MASVS-PLATFORM | L1 | Thông báo (ngày giỗ) không lộ nội dung trên màn hình khóa nếu người dùng chọn; WebView (nếu có) tắt JS bridge; deep link kiểm tra đầu vào |
| MASVS-CODE | **L1 — có phát hiện ngay** | Lõi **panic/treo** với đầu vào xấu (RS-01, RS-02). Panic vượt qua ranh giới FFI: UniFFI bắt panic và chuyển thành lỗi phía Kotlin/Swift, nhưng **vòng lặp vô hạn** thì không — luồng gọi bị treo. Lớp `ffi/src/lib.rs` hiện kiểm tra `MIN_YEAR..=MAX_YEAR` trước khi gọi lõi (tốt), nhưng `#[uniffi::export]` mới nào quên kiểm tra sẽ mở lại lỗ hổng. Khuyến nghị: lõi tự trả `Result`/`Option` cho năm ngoài phạm vi. Cũng cần: theo dõi lỗ hổng phụ thuộc (`cargo audit`, `cargo deny`) cho 46 crate của `ffi`; `overflow-checks = true` ở bản release cho lõi (RS-04) |
| MASVS-RESILIENCE | Không áp dụng (không MAS-R) | — |
| MASVS-PRIVACY | L1 và L2 | Không SDK quảng cáo/phân tích ở v0.x; khai báo quyền tối thiểu (không vị trí, không danh bạ); nhãn quyền riêng tư của cửa hàng khớp hành vi thật; module "thông báo tang gia" và chia sẻ Zalo/FB sau này cần đánh giá riêng (dữ liệu bên thứ ba, ảnh người đã khuất) |

## Ảnh chụp trạng thái khi duyệt

Thư mục làm việc thay đổi trong lúc duyệt (bộ tích hợp ghi đồng thời). Kết luận dựa trên các tệp
sau, lấy lúc khoảng 03:00–03:10 (giờ hệ thống tệp):

- `pwa/dist/` dựng lúc 03:02:29: `index.html` sha256 `6741770e43ea3489…`, `sw.js` `5de2d4835c54fe86…`,
  `app-GSOED2I2.js` `78f1c05a96946a7d…` (430 772 byte, lõi `tsport`, không còn nhãn MOCK).
- `core/src/gooddays.rs` (02:50), `core/src/dayinfo.rs`, `canchi.rs`, `lib.rs` (02:49–02:50) — lõi nay
  phụ thuộc `lvhv-ngochap`.
- Bản `pwa/dist/` cũ hơn (00:43, `app-3PJRK7RE.js`) dựng với **lõi giả lập (mock)**. Nếu bản đó được
  dùng làm bằng chứng P3/P4/P5 trong `pwa/test/metrics.json` thì bằng chứng đó không còn đại diện
  (xem RA-SOAT-BAO-MAT RS-16).
