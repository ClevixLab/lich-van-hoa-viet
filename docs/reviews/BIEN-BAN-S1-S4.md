# Biên bản nghiệm thu bảo mật S1–S4 (G4) — v0.1

Người kết luận: A9/A10, là bên không tham gia viết mã được nghiệm thu.
Ngày: 2026-11-06. Tiêu chí theo `docs/02-TIEU-CHI-NGHIEM-THU.md` (khóa trước khi viết mã; không nới lỏng).
Thang kết luận: ĐẠT / KHÔNG ĐẠT / CHƯA ĐỦ BẰNG CHỨNG.
Trạng thái mã: thư mục làm việc lúc ~03:00–03:10 (giờ tệp). Kho chưa có commit nào, nên không có mã
commit để neo; các băm `pwa/dist/` liệt kê ở S3. Kết luận dưới đây chỉ có giá trị cho đúng trạng thái này.

## Tóm tắt

| Mã | Tiêu chí (nguyên văn) | Ngưỡng | Kết luận |
|---|---|---|---|
| S1 | Không có `unsafe` trong core | 0 | **ĐẠT** |
| S2 | Thư viện phụ thuộc của core | 0 (lõi không phụ thuộc thư viện ngoài) | **ĐẠT**, có điều kiện diễn giải (xem dưới) |
| S3 | PWA không gọi mạng ngoài nguồn gốc của chính nó; có Content-Security-Policy chặt | Đạt | **ĐẠT** |
| S4 | PWA không lưu dữ liệu người dùng (v0.1 không có tài khoản) | Đạt | **ĐẠT** |

Kết luận ĐẠT ở S1–S4 **không** có nghĩa là lõi an toàn với đầu vào xấu: các tiêu chí S1–S4 không đo điều đó.
Các phát hiện Cao RS-01/RS-02 (treo vô hạn, kết quả sai lặng lẽ ngoài phạm vi năm) nằm ngoài S1–S4 và được
xếp vào mục "bắt buộc sửa trước G4" trong `DUYET-MA.md` §9.

---

## S1 — ĐẠT

**Mệnh đề.** Mã nguồn của crate `lvhv-core` (`core/`) không chứa `unsafe`.
Tiêu chí ĐẠT đã định trước khi chạy lệnh: grep không thấy khối/hàm/impl/trait `unsafe` nào ngoài chú thích,
và crate có `#![forbid(unsafe_code)]` để trình biên dịch tự chặn.

**Lệnh và kết quả:**
```
$ grep -rn "unsafe" --include=*.rs core ngochap
core/src/lib.rs:3://! Không phụ thuộc thư viện ngoài, không dùng `unsafe`. Múi giờ UTC+7 cố định cho 2000–3000.
core/src/lib.rs:5:#![forbid(unsafe_code)]
core/src/bin/lvhv-dump.rs:2:#![forbid(unsafe_code)]
ngochap/src/lib.rs:15:#![forbid(unsafe_code)]
```
Ba dòng là thuộc tính `forbid`, dòng còn lại là chú thích. Lõi biên dịch được với `forbid(unsafe_code)`
(`cargo test --release -p lvhv-core` thành công: unit 14, fixtures 7, methods 2, ngochap 7, tất cả passed).
`forbid` không thể bị `#[allow]` ghi đè ở mức trong.

**Ngoài phạm vi S1, ghi để đầy đủ:**
- `lvhv-ngochap`, crate mà lõi nay phụ thuộc, cũng có `#![forbid(unsafe_code)]`.
- `ffi/` (UniFFI) chứa `unsafe` do macro sinh (`ffi/src/lib.rs:6–8` tự ghi rõ). S1 chỉ nói "core" nên `ffi` không
  thuộc tiêu chí này. Nếu chủ dự án muốn S1 bao phủ phần chạy trên ứng dụng gốc thì phải viết tiêu chí mới,
  không được diễn giải lại S1.
- Thư viện chuẩn của Rust có `unsafe` bên trong; theo cách hiểu thông thường, điều này không thuộc S1.

## S2 — ĐẠT (có điều kiện diễn giải)

**Mệnh đề.** `lvhv-core` không phụ thuộc thư viện ngoài.
Tiêu chí đã định trước: `cargo tree` (normal, build, dev) của `lvhv-core` không chứa crate nào có nguồn là
registry, git, hay đường dẫn ra ngoài kho mã.

**Lệnh và kết quả (từ workspace gốc, sau khi tích hợp):**
```
$ cargo tree -p lvhv-core -e normal,build,dev --offline
lvhv-core v0.1.0 (/home/claude/lich-van-hoa-viet/core)
└── lvhv-ngochap v0.1.0 (/home/claude/lich-van-hoa-viet/ngochap)

$ cargo tree -p lvhv-ngochap -e normal,build,dev --offline
lvhv-ngochap v0.1.0 (/home/claude/lich-van-hoa-viet/ngochap)
```
`core/Cargo.toml [dependencies]`: `lvhv-ngochap = { path = "../ngochap" }`; `ngochap/Cargo.toml [dependencies]`: rỗng.
Trước khi tích hợp (~01:10), lệnh `cargo tree` chạy trong `core/` chỉ cho ra `lvhv-core v0.1.0`.

**Điều kiện diễn giải, cần chủ dự án xác nhận.** Ngưỡng ghi "0", kèm chú thích "lõi không phụ thuộc thư viện
ngoài". Hiện có đúng 1 phụ thuộc, là crate nội bộ cùng kho (`lvhv-ngochap`), không có phụ thuộc bắc cầu, và
không có mã bên thứ ba. Người duyệt đọc "thư viện ngoài" là mã bên thứ ba, nên kết luận ĐẠT. Nếu đọc "0" theo
nghĩa đen là không có bất kỳ crate nào khác, thì kết luận phải là KHÔNG ĐẠT. Kết quả này đúng với chính hợp
đồng §5, nơi Ngọc Hạp được dự kiến đặt trong `core/src/ngochap*`; tách thành crate riêng chỉ là thay đổi cấu trúc.

`ffi/` kéo 46 crate bên ngoài (uniffi, serde, serde_json…), nhưng `ffi` không phải "core", nên không thuộc S2.

## S3 — ĐẠT

**Mệnh đề.** (a) Bản PWA đã dựng không phát sinh yêu cầu mạng nào tới origin khác origin phục vụ nó.
(b) Có CSP chặt.
Tiêu chí đã định trước: (a) trình duyệt thật ghi nhận 0 yêu cầu ra ngoài origin khi duyệt mọi màn hình và thao
tác chính, đồng thời mã tĩnh không có URL ngoài nào ở vị trí tải (`src`, `href`, `fetch`, `import`, CSS `url()`,
`@import`); (b) CSP có `default-src 'self'` và `script-src` không chứa `'unsafe-inline'`/`'unsafe-eval'`/nguồn
ngoài, `object-src 'none'`, `base-uri` bị giới hạn, `index.html` không có script nội tuyến, và 0 vi phạm CSP
lúc chạy.

**Sản phẩm kiểm:** `pwa/dist/` dựng lúc 03:02:29 — `index.html` sha256 `6741770e43ea3489…`, `sw.js`
`5de2d4835c54fe86…`, `app-GSOED2I2.js` `78f1c05a96946a7d…` (lõi tsport, không còn mock).

**Bằng chứng tĩnh:**
```
$ grep -o '<meta http-equiv="Content-Security-Policy"[^>]*>' pwa/dist/index.html
<meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self'; style-src 'self';
 img-src 'self' data:; font-src 'self'; connect-src 'self'; manifest-src 'self'; worker-src 'self';
 object-src 'none'; base-uri 'self'; form-action 'none'">

$ grep -cE "<style|<script>|<script [^>]*>[^<]+|style=\"" pwa/dist/index.html
0

$ grep -oE "localStorage|sessionStorage|indexedDB|document\.cookie|innerHTML|outerHTML|insertAdjacentHTML|eval\(|new Function|importScripts|fetch\(|XMLHttpRequest|WebSocket|sendBeacon|\.style\.[a-z]" pwa/dist/app-*.js | sort | uniq -c
      2 localStorage
```
URL tuyệt đối có trong gói gồm hai loại. Một là `http://www.w3.org/2000/svg`, chỉ là không gian tên. Hai là
URL nguồn ca dao/Ngọc Hạp (voh.com.vn, vi.wikiquote.org…), chỉ được hiển thị dạng văn bản. Không có `href`/`src`
nào trỏ tới các URL này: các thuộc tính `href` trong mã đều là `#/…` hoặc `RULES_HREF` nội bộ. CSS chỉ có
`url("./fonts/…")` và không có `@import`. `sw.js` có điều kiện `if (url.origin !== self.location.origin) return;`.

**Bằng chứng chạy thật** (`probe.mjs`: Chrome qua playwright-core 1.63.0, máy chủ HTTP cục bộ phục vụ bản sao
của `dist/` trên; lắng nghe mọi `request` và `securitypolicyviolation`). Đã đi qua tờ lịch, chọn ngày tốt (chọn
việc, nhập năm sinh), đổi ngày, giới thiệu, đổi cỡ chữ và giao diện, rồi tải lại khi offline:
```
"external": []
"consoleErrors": []
csp violations: []
```
Bộ kiểm thử khói của dự án (`pwa/test/smoke.mjs`) cũng đếm yêu cầu ngoài, nhưng `metrics.json` hiện có là của
bản mock nên người duyệt không dùng làm bằng chứng.

**Giới hạn đã biết (không làm đổi kết luận, ghi lại để không bị hiểu quá mức):**
- CSP đặt qua `<meta>`, nên `frame-ancestors` không có tác dụng và không có cơ chế báo cáo vi phạm. "Chặt" ở đây
  chỉ là chặt trong giới hạn mà GitHub Pages cho phép (RS-09).
- `img-src` cho phép `data:`; điều này không tạo kênh gọi mạng.
- CSP không bảo vệ được trước mã độc cùng origin, ví dụ khi việc triển khai bị xâm nhập (RS-07).
- S3 được kiểm trên **một** bản build. Mỗi bản build mới cần kiểm lại, và việc này nên đưa vào CI.

## S4 — ĐẠT

**Mệnh đề.** PWA không lưu dữ liệu người dùng; nói cụ thể là không có dữ liệu do người dùng nhập về bản thân
(năm sinh) hay dữ liệu nhận diện. Tùy chọn giao diện (cỡ chữ, sáng/tối) được hiểu là thiết lập ứng dụng,
không phải dữ liệu người dùng. Cách hiểu này đã định trước khi kiểm, và khớp ghi chú trong `pwa/src/prefs.ts` và
màn Giới thiệu.
Tiêu chí đã định trước: sau khi nhập năm sinh và dùng mọi màn hình, `localStorage` chỉ chứa khóa `lvhv.ui` với
giá trị thuộc danh sách trắng; `sessionStorage`, cookie và IndexedDB đều rỗng; Cache Storage chỉ chứa tệp tĩnh
của ứng dụng; mã nguồn không có lối ghi nào khác.

**Bằng chứng mã:** chỉ `pwa/src/prefs.ts:7,14,30` truy cập `localStorage` (khóa `"lvhv.ui"`, giá trị
`{size ∈ {"1","2","3"}, theme ∈ {"auto","light","dark"}}`). Năm sinh chỉ nằm trong biến bộ nhớ `st.birth`
(`pwa/src/ui/gooddays.ts:14,39,79,101–102`), không có lời gọi lưu trữ nào. Gói build chỉ có 2 lần xuất hiện
`localStorage` (đọc và ghi trong prefs) và không có `sessionStorage`, `indexedDB`, `document.cookie`.

**Bằng chứng chạy thật** (`probe.mjs`, sau khi nhập năm sinh 1976 và chạy chọn ngày):
```
localStorage (trước khi đổi tùy chọn): {}
sessionStorage: []   cookie: ""   indexedDB: []   caches: ["lvhv-v0.1-3ba9c3b255"]
localStorage (sau khi đổi cỡ chữ, giao diện): {"lvhv.ui":"{\"size\":\"2\",\"theme\":\"light\"}"}
```

**Giới hạn:** máy chủ hosting ghi nhật ký truy cập (IP, User-Agent). Đây không phải dữ liệu PWA tự lưu, nên
không thuộc S4, nhưng nên nói ra trong mục Quyền riêng tư (RS-18).

---

## Các tiêu chí liên quan mà người duyệt có chạm tới (không thuộc S, không kết luận chính thức)

- **N3:** người duyệt chạy lại `cargo test --release -p lvhv-core --test ngochap -- --ignored`:
  ```
  test n3_all_days_2000_3000 ... ok
  test n3_find_good_days_2000_3000 ... ok
  test result: ok. 2 passed; 0 failed; ... finished in 599.03s
  ```
  Các test này kiểm sự nhất quán giữa cài đặt và bảng `major`, không kiểm bảng có đúng nguồn hay không
  (DUYET-MA §3).
- **L7 (một phần, 2000–2100):** sinh mới từ lõi và so với tsport hiện tại: 0 sai lệch (Hnd và Precise, 36 890
  dòng mỗi phương pháp). Chưa chạy lại 2101–3000, nên L7 toàn dải thuộc trách nhiệm người nghiệm thu G2.
- **P2:** `find_good_days` 90 ngày = 0,69 ms (bản release, sandbox 2 CPU), một lần đo.
- **P3/P4/P5:** `metrics.json` hiện có là của bản mock, nên CHƯA ĐỦ BẰNG CHỨNG cho bản hiện tại (RS-16).
