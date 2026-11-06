# Tiêu chí nghiệm thu — v0.2 (đăng ký trước)

**KHÓA (G1) — 2026-11-06, chờ chủ dự án ký.**

Các tiêu chí được khóa **trước khi viết mã** của v0.2. Không được nới ngưỡng, đổi phương pháp đo hay thu hẹp dữ liệu
sau khi đã thấy kết quả. Mọi thay đổi sau khi khóa phải ghi thành một "Bổ sung" có ngày, nêu rõ thay đổi được đề xuất
**trước** hay **sau** khi thấy kết quả (như docs/02 Bổ sung A1–A3 của v0.1), và cần chủ dự án ký.

Ghi chú đối chiếu: nội dung R1–R9 dưới đây được viết từ phạm vi v0.2 (kế hoạch chủ dự án duyệt 06/11/2026; tài liệu
soạn đặc tả chỉ có danh sách phạm vi và mã R1–R9, không có câu chữ từng tiêu chí). Điều phối viên đối chiếu từng mã
với kế hoạch trước khi trình ký; mọi chênh lệch được sửa **trước** khi ký G1, khi chưa có mã và chưa có kết quả.

Mỗi tiêu chí kết luận đúng một trong ba:
- **ĐẠT** — đã đo đúng phương pháp, trên đúng dữ liệu, và đạt ngưỡng.
- **KHÔNG ĐẠT** — đã đo đúng phương pháp và không đạt ngưỡng (một lần vi phạm là đủ).
- **CHƯA ĐỦ BẰNG CHỨNG** — chưa đo, đo thiếu dữ liệu/nền tảng, hoặc phương pháp đo không chạy được (ví dụ chưa có
  thiết bị thật). Không được ghi ĐẠT khi chỉ đo một phần. Mỗi kết luận này phải ghi rõ còn thiếu gì.

Kết luận ghi theo **từng nền tảng** khi tiêu chí áp dụng cho nhiều nền tảng (PWA / Android / iOS / macOS / Windows).
Biên bản ghi máy đo, phiên bản mã (commit), ngày đo và đường dẫn tới dữ liệu thô.

Bên kiểm định độc lập: **bên không tham gia xây dựng đặc tả** và không viết mã lõi v0.2. Bên này chỉ đọc
`01-MO-HINH-DU-LIEU.md`, `02-THUAT-TOAN-GIO-NHAC.md` và tài liệu này; không đọc `core/`, `tsport/`, `pwa/`, `apps/`
trước khi nộp kết quả (như vai A4 ở v0.1).

## 1. Chức năng (G2)

### R1 — Ngày giỗ hằng năm đúng quy tắc (kiểm định độc lập)

| Mục | Nội dung |
|---|---|
| Mệnh đề | Với mọi ngày mất, `anniversary(death, Y)` của ứng dụng cho đúng (năm âm, tháng, ngày, ngày dương, n, nhãn) theo 02 §2–§3 |
| Cài đặt độc lập | Bên không tham gia xây dựng đặc tả viết lại `save_death` + `anniversary` + `label_of` trong thư mục `oracle/v02/` (Python), dùng bảng âm lịch của **oracle v0.1** (PyEphem, `oracle/`) để đổi lịch, không dùng lõi |
| Dữ liệu | (a) **Toàn bộ** ngày mất dương 1900-01-01 → 2099-12-31 (73 049 ngày), mỗi ngày tính 10 năm giỗ kế tiếp (Y = năm âm mất + 1 … + 10); (b) toàn bộ ngày mất âm "không rõ năm": 12 tháng × 30 ngày × {thường}, với Y = 1900 … 2100; (c) 2 000 ngày mất dương chọn ngẫu nhiên (hạt giống `20261106`) trong 2100–2990, 10 năm giỗ mỗi ngày; (d) các ca G-A … G-E và ca biên 02 §9 |
| Ngưỡng | (a), (b), (d): **0 sai lệch** ở mọi trường. (c): không có ngưỡng ĐẠT riêng; mọi sai lệch phải liệt kê và phân loại; sai lệch chỉ được chấp nhận nếu bắt nguồn từ một ngày âm khác nhau giữa lõi và oracle **đã có trong danh sách sai lệch L3 của v0.1** (Sóc sát nửa đêm). Sai lệch "nguyên nhân khác" → KHÔNG ĐẠT |
| Phương pháp | Hai bên xuất JSONL cùng định dạng (`{death_solar, Y, lunar:{y,m,d}, solar:{y,m,d}, n, label}`), so từng byte sau khi sắp; bản TypeScript và lõi Rust phải giống nhau từng byte (như L7) |
| Kết luận | ĐẠT khi (a),(b),(d) = 0 sai lệch và (c) không có "nguyên nhân khác", và Rust ≡ TypeScript. KHÔNG ĐẠT khi có một sai lệch ở (a),(b),(d) hoặc một sai lệch "nguyên nhân khác" ở (c). CHƯA ĐỦ BẰNG CHỨNG khi bên độc lập chưa nộp hoặc thiếu một phần dữ liệu |

### R2 — Tuổi âm, hưởng thọ/hưởng dương, Can Chi năm sinh, quy tắc 29/02

| Mục | Nội dung |
|---|---|
| Mệnh đề | 02 §4 (`lunar_year`, `can_chi`, `tuoi_mat`, nhãn theo ngưỡng) và 02 §6 (29/02) cài đặt đúng |
| Dữ liệu | (a) Mọi ngày sinh dương 1900-01-01 → 2026-12-31 → `lunar_year` so với `LYOB` của oracle; (b) Can Chi của mọi năm 1899–3000 so với công thức hợp đồng §3 viết độc lập; (c) 10 000 cặp (sinh, mất) ngẫu nhiên (hạt `20261106`) gồm mọi tổ hợp kiểu nhập (dương/âm × đủ ngày/chỉ năm) và ngưỡng {59, 60, 61}; (d) 29/02 với mọi năm 1900–3000 × hai lựa chọn `feb28`/`mar1` |
| Ngưỡng | 0 sai lệch ở (a)–(d); ca sinh 1990-01-20 → Kỷ Tỵ, ca 1940-05-01/2025-08-08 → 86 hưởng thọ đúng |
| Kết luận | ĐẠT / KHÔNG ĐẠT theo ngưỡng; CHƯA ĐỦ BẰNG CHỨNG nếu thiếu (a)–(d) |

### R3 — Sinh lời nhắc: mọi tổ hợp lựa chọn

| Mục | Nội dung |
|---|---|
| Mệnh đề | `reminders_for` (02 §7.1) cho đúng tập (id, thời điểm, loại) với **mọi** tổ hợp lựa chọn hợp lệ |
| Không gian tổ hợp (vét cạn) | Trạng thái giao diện: mọi tập con của các nút có sẵn {0, 1, 3, 5, 10, 15} (**64**) × mốc tự đặt ∈ {không có, 1, 2, …, 60} (**61**) = **3 904** trạng thái, cho ra **3 584** tập `offsets_days` khác nhau (mốc tự đặt trùng nút có sẵn thì gộp; tập rỗng = không nhắc). Chạy đủ 3 904 trạng thái (kiểm cả bước gộp) × `tien_thuong` ∈ {tắt, bật} (chỉ áp cho giỗ; loại khác phải bỏ qua) × `time_local` ∈ {00:00, 07:00, 23:59} × loại lần xuất hiện ∈ {giỗ, sinh nhật dương, sinh nhật âm, sự kiện dương lặp, sự kiện âm lặp, sự kiện không lặp} = **140 544** cấu hình |
| Lần xuất hiện | Bộ cố định 40 hồ sơ/sự kiện gồm mọi ca G-A … G-E, ca biên 02 §9 (giỗ mùng 1 Tết, giỗ 30 tháng Chạp, sinh 29/02, offset 60 qua năm), tính trên cửa sổ 2026-01-01 → 2036-12-31 với `now` = 2026-01-01T00:00 giờ thiết bị |
| Đối chiếu | Bộ sinh tham chiếu viết độc lập từ 02 §7.1 (cùng bên ở R1); so tập kết quả từng phần tử |
| Thêm | Tắt công tắc chung → 0 lời nhắc; `rule.enabled = false` → 0; lời nhắc có thời điểm ≤ `now` không được sinh; id ổn định khi đổi `time_local` |
| Ngưỡng | 0 sai lệch trên toàn bộ không gian (ghi tổng số lời nhắc đã so) |
| Kết luận | ĐẠT / KHÔNG ĐẠT; CHƯA ĐỦ BẰNG CHỨNG nếu chưa chạy đủ 140 544 cấu hình |

### R4 — Danh sách "Sắp tới" 30 ngày

| Mục | Nội dung |
|---|---|
| Mệnh đề | 02 §7.2: đúng tập lần xuất hiện trong [hôm nay, hôm nay + 30], đúng thứ tự, không phụ thuộc công tắc nhắc |
| Dữ liệu | Bộ hồ sơ của R3 × mọi "hôm nay" từ 2026-01-01 đến 2036-12-31 (4 018 ngày) |
| Ngưỡng | 0 sai lệch so với bộ tham chiếu độc lập; ngày đầu và ngày thứ 30 đều có mặt |
| Kết luận | ĐẠT / KHÔNG ĐẠT / CHƯA ĐỦ BẰNG CHỨNG |

### R5 — Thông báo cục bộ trên thiết bị (không mạng)

| Mục | Nội dung |
|---|---|
| Mệnh đề | Lời nhắc đến đúng ngày, trong cửa sổ trễ cho phép, không cần mạng, kể cả sau khởi động lại; iOS không mất lời nhắc lặng lẽ khi có > 64 lời nhắc |
| (a) Mô phỏng xoay vòng iOS | Hàm `reschedule` (02 §7.4) chạy trên bộ R3 với 10, 50, 200 hồ sơ; mô phỏng thời gian trôi 2026-01-01 → 2028-12-31, mỗi ngày "mở ứng dụng" với xác suất {1, 0,1, 0} (ba kịch bản). Bất biến kiểm sau mỗi bước: số yêu cầu chờ ≤ 64; nếu tổng tương lai ≥ 64 thì 63 cái sớm nhất đang chờ + thông báo nhắc mở ứng dụng ngay sau cái thứ 63; không bao giờ có lời nhắc tương lai bị bỏ mà không có thông báo nhắc mở ứng dụng đứng trước nó |
| (b) Thiết bị thật | Mỗi nền tảng: 1 máy thật (Android ≥ 14; iPhone iOS ≥ 17; macOS ≥ 14; Windows 11). Chế độ máy bay bật suốt phép thử. Đặt 5 lời nhắc trong 2 giờ tới (đổi giờ hệ thống không được dùng). Một lần khởi động lại máy giữa chừng (Android, Windows). Ghi thời điểm hiện thông báo (ảnh chụp màn hình có đồng hồ hệ thống) |
| Ngưỡng | (a) 0 vi phạm bất biến. (b) 5/5 lời nhắc hiện; trễ so với thời điểm đặt: iOS/macOS ≤ 1 phút; Android ≤ 60 phút với báo thức không chính xác (≤ 1 phút nếu O-6 chọn báo thức chính xác và quyền được cấp); Windows ≤ 1 phút khi ứng dụng đã từng mở sau khởi động. Không hiện trước thời điểm đặt |
| PWA | Không áp dụng (02 §7.5) — PWA chấm thẻ nhắc trong trang: mở ứng dụng vào ngày có lời nhắc → thẻ hiện (kiểm thử tự động) |
| Kết luận | Theo từng nền tảng. Thiếu máy thật → CHƯA ĐỦ BẰNG CHỨNG cho nền tảng đó (không thay bằng giả lập) |

### R6 — Tệp .ics

| Mục | Nội dung |
|---|---|
| Mệnh đề | Tệp xuất đúng 02 §8 và nhập được vào ứng dụng lịch phổ biến với đúng ngày |
| (a) Cú pháp | Mọi tệp xuất từ bộ R3 qua một bộ phân tích RFC 5545 độc lập (thư viện Python `icalendar`, ghim phiên bản trong biên bản): 0 lỗi; mọi dòng ≤ 75 octet; CRLF; mọi VALARM có ACTION/DESCRIPTION/TRIGGER; không có RRULE; không có tham số TZID trên giá trị DATE; có VTIMEZONE `Asia/Ho_Chi_Minh` |
| (b) Ngữ nghĩa | Tập (UID, DTSTART) trong tệp ≡ tập lần xuất hiện của R4 trong 10 năm; UID giống nhau giữa hai lần xuất trước/sau khi sửa tên hồ sơ; SEQUENCE tăng |
| (c) Nhập thật | Nhập tệp mẫu 20 sự kiện vào: Google Calendar (web), Apple Calendar (iOS và macOS), Outlook (Windows hoặc web). Ghi: ngày hiển thị, lời nhắc có được giữ hay bị thay bằng mặc định của ứng dụng lịch |
| Ngưỡng | (a), (b): 0 lỗi. (c): ngày hiển thị đúng 20/20 ở mọi ứng dụng thử. Việc ứng dụng lịch bỏ VALARM **ghi nhận, không làm KHÔNG ĐẠT** (ngoài tầm kiểm soát), nhưng phải ghi vào hướng dẫn sử dụng |
| Kết luận | ĐẠT khi (a),(b),(c) đạt; CHƯA ĐỦ BẰNG CHỨNG nếu (c) thiếu ứng dụng nào |

### R7 — Không có kết nối mạng do dữ liệu cá nhân

| Mục | Nội dung |
|---|---|
| Mệnh đề | Không nền tảng nào gửi dữ liệu ra mạng; native không mở kết nối mạng nào nếu người dùng không bấm một nút mở liên kết |
| Kịch bản | Một lượt dùng đầy đủ: tạo 3 người đã khuất (có ảnh) + 2 người sống + 3 sự kiện, mở màn hình tưởng niệm 3 phút, bật nhắc, xuất .ics, xuất và nhập sao lưu, xóa dữ liệu |
| PWA | Chromium qua Playwright: ghi mọi yêu cầu của trang **và** Service Worker (CDP `Network` cho cả hai đích); ngưỡng: 0 yêu cầu tới nguồn gốc khác; yêu cầu cùng nguồn gốc chỉ là tệp tĩnh trong danh sách precache; 0 yêu cầu có thân (POST/PUT); 0 vi phạm CSP. Chạy thêm một lần ở chế độ ngoại tuyến: kịch bản hoàn tất |
| Android | (i) Tĩnh: `aapt2 dump permissions` không có `INTERNET` (khi không có quyền này hệ điều hành chặn mọi socket); (ii) động: trình giả lập, `tcpdump` trên giao diện mạng của trình giả lập trong suốt kịch bản, lọc theo UID của ứng dụng (`/proc/net/xt_qtaguid` hoặc `dumpsys netstats detail`) — 0 byte |
| iOS / macOS | (i) Tĩnh: mã Swift không dùng `URLSession`, `NWConnection`, `Network.framework`, `CFNetwork`, WebKit (grep + `nm` trên binary: 0 ký hiệu ngoài danh sách cho phép, chỉ `UIApplication.open`/`NSWorkspace.open` cho nút cập nhật); (ii) động: iPhone thật qua `rvictl` + `tcpdump` trên Mac, hoặc `nettop -p <pid>` cho macOS — 0 kết nối của tiến trình ứng dụng |
| Windows | (i) Tĩnh: `cargo tree` của ứng dụng không có crate HTTP/socket (`reqwest`, `hyper`, `ureq`, `tokio` với `net`); (ii) động: Resource Monitor/`netstat -abno` hoặc Process Monitor lọc theo tiến trình — 0 sự kiện TCP/UDP |
| Ngưỡng | 0 kết nối/0 byte ở mọi nền tảng trong kịch bản; ngoại lệ duy nhất: ngay sau khi người dùng bấm "Kiểm tra cập nhật" (hệ điều hành mở trình duyệt/cửa hàng — tiến trình khác) |
| Kết luận | Theo từng nền tảng; chỉ có kiểm tĩnh mà thiếu kiểm động → CHƯA ĐỦ BẰNG CHỨNG |

### R8 — Màn hình tưởng niệm: hoạt ảnh tiết chế, tiêu thụ CPU

| Mục | Nội dung |
|---|---|
| Mệnh đề | Khói hương chỉ chạy khi màn hình đang hiển thị, tự dừng sau 2 phút, không chạy khi người dùng chọn giảm chuyển động; không âm thanh, không quảng cáo |
| Đo PWA | Chromium (Playwright), khung nhìn 360×780, giả lập CPU chậm 4× (`Emulation.setCPUThrottlingRate`). Đếm lời gọi `requestAnimationFrame` bằng bộ đếm thử nghiệm trong bản dựng kiểm thử; đo thời gian CPU luồng chính bằng `Performance.getMetrics` (`TaskDuration`) mỗi cửa sổ 10 s; 3 lần chạy, lấy lần xấu nhất |
| Ngưỡng PWA | (a) 0–120 s: CPU luồng chính ≤ 15 % một lõi (sau giả lập 4×), trung bình mỗi cửa sổ 10 s; (b) từ 121 s: 0 lời gọi rAF và ≤ 1 % CPU; (c) tab ẩn (`visibilitychange`) hoặc rời màn hình: 0 rAF trong vòng 1 s; (d) `prefers-reduced-motion: reduce`: 0 rAF ngay từ đầu, khói hiển thị dạng tĩnh; (e) 0 phần tử `<audio>`/`<video>`, 0 lời gọi `AudioContext`; (f) Công giáo: 0 phần tử hương |
| Đo native | Android: `adb shell top -d 10 -p <pid>` và `dumpsys gfxinfo` (số khung) trên máy thật; iOS: Instruments (Time Profiler + Core Animation FPS); Windows: Process Explorer (CPU của tiến trình) |
| Ngưỡng native | (a) ≤ 10 % một lõi trung bình trong 120 s đầu; (b) sau 120 s: 0 khung vẽ mới mỗi giây (gfxinfo/Core Animation) và ≤ 1 % CPU; (c), (d), (e), (f) như PWA (đọc cài đặt giảm chuyển động của hệ điều hành) |
| Nội dung | Kiểm danh sách tệp hình: mọi tệp trong `content/memorial/` có dòng nguồn "vẽ mới cho dự án, giấy phép …" trong `content/memorial/SOURCES.md`; 0 tệp lấy từ nguồn ngoài |
| Kết luận | Theo từng nền tảng; thiếu máy thật cho native → CHƯA ĐỦ BẰNG CHỨNG |

### R9 — Sao lưu mã hóa: khứ hồi, chống sửa, kiểm chéo

| Mục | Nội dung |
|---|---|
| Mệnh đề | Tệp sao lưu theo 03 §2 khôi phục đúng dữ liệu; mọi tệp bị sửa/dị dạng bị từ chối mà không ghi gì; hai bản cài đặt đọc được tệp của nhau |
| (a) Khứ hồi | Bộ dữ liệu R3 + 20 ảnh: xuất → nhập "Thay toàn bộ" trên máy trống → so toàn bộ dữ liệu (JSON chuẩn hóa + sha256 ảnh): giống hệt. Lặp lại cho mỗi nền tảng có mã |
| (b) Chống sửa | Các tệp hỏng của 03 §2.5 + lật **từng bit** của 4 096 byte đầu và 4 096 byte cuối của một tệp hợp lệ (65 536 biến thể): 100 % bị từ chối, kho dữ liệu không đổi (so băm trước/sau) |
| (c) Vector chuẩn | Argon2id RFC 9106 §5.3 và XChaCha20-Poly1305 (`draft-irtf-cfrg-xchacha` A.3): khớp |
| (d) Kiểm chéo | Tệp xuất từ PWA nhập được vào native và ngược lại (mỗi cặp nền tảng có mã); nếu O-7 chọn thư viện JavaScript thì thêm: một bản giải mã độc lập (Python, thư viện `argon2-cffi` + `PyNaCl`/libsodium, viết bởi bên không tham gia xây dựng đặc tả) giải mã đúng 10 tệp mẫu |
| (e) Gộp | 01 §6.3: bộ 30 ca gộp (trùng id khác rev, trùng tên loại sự kiện, ảnh trùng) cho đúng kết quả mô tả |
| Ngưỡng | (a), (c), (d), (e): 0 sai lệch; (b): 0 tệp hỏng được chấp nhận |
| Kết luận | ĐẠT / KHÔNG ĐẠT; CHƯA ĐỦ BẰNG CHỨNG nếu thiếu cặp nền tảng nào ở (d) |

## 2. Hiệu năng (G3)

Đo như v0.1 (`pwa/test/metrics.json`, ghi máy đo). Mốc so sánh v0.1.1: tải đầu 351,6 KB gzip, heap 2,45 MB
(360×780, Chromium 153, 2026-11-06).

| Mã | Tiêu chí | Phương pháp | Ngưỡng |
|---|---|---|---|
| P3 (giữ) | PWA: dung lượng tải lần đầu (gzip), mở tờ lịch, chưa có dữ liệu | Như v0.1 | **< 500 KB**; biên bản ghi mức tăng so với 351,6 KB |
| P6 | Tài nguyên tưởng niệm và sao lưu tải lười | Danh sách yêu cầu ở lần tải đầu (P3) | 0 tệp của màn hình tưởng niệm (hình mâm, bia, hương, khói), 0 tệp mật mã/WebAssembly; chúng chỉ được tải khi mở màn hình tương ứng; tổng gói tưởng niệm (gzip) ghi vào biên bản, < 300 KB |
| P4 (giữ, mở rộng) | PWA: heap JavaScript | Như v0.1; thêm hai trạng thái | Tờ lịch, chưa có dữ liệu: < 30 MB. Tờ lịch + "Sắp tới" với 200 hồ sơ và 1 000 sự kiện: < 30 MB. Màn hình tưởng niệm sau 120 s: < 50 MB |
| P7 | Tính lại toàn bộ lời nhắc (200 hồ sơ, 1 000 sự kiện, cửa sổ 400 ngày) | Bản release native; PWA trên Chromium, giả lập CPU 4× | Native < 200 ms; PWA < 1 s |
| P8 | Argon2id 64 MiB, t=3, p=4 | Trung vị 5 lần; thiết bị tham chiếu: một điện thoại Android tầm trung (ghi mẫu máy), một iPhone, một máy tính | ≤ 5 s trên điện thoại, ≤ 3 s trên máy tính; bộ nhớ đỉnh ghi vào biên bản. Vượt ngưỡng → KHÔNG ĐẠT; đổi tham số là một Bổ sung (03 §2.1 chỉ cho giảm t/p) |
| P9 | Bộ nhớ native khi xem tờ lịch (như ngưỡng v0.1, báo cáo năng lượng §4.1) | Như v0.1 | Android < 80 MB, iOS < 60 MB; 0 lần đánh thức nền ngoài báo thức nhắc đã đặt |

## 3. Bảo mật và quyền riêng tư (G4)

| Mã | Tiêu chí | Phương pháp | Ngưỡng |
|---|---|---|---|
| S1, S2 (giữ) | `core/` không `unsafe`, không phụ thuộc ngoài | Như v0.1 | Như v0.1. Mật mã và lưu trữ nằm ở crate riêng (`lvhv-backup`, `lvhv-store`), không ở `core/` |
| S2b | CSP của PWA **không đổi** | So chuỗi `Content-Security-Policy` trong `pwa/dist/index.html` với bản 0.1.1 (`default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; manifest-src 'self'; worker-src 'self'; object-src 'none'; base-uri 'self'; form-action 'none'`) | Giống hệt từng byte. **Ngoại lệ đăng ký trước, chỉ khi chủ dự án chọn O-7 (A):** thêm đúng một từ `'wasm-unsafe-eval'` vào `script-src`, không gì khác. `img-src` dùng `blob:` cho ảnh chân dung **không** được phép — ảnh hiển thị qua `URL.createObjectURL` cần `blob:` nên giao diện phải vẽ ảnh vào `<canvas>` hoặc chủ dự án duyệt thêm ngoại lệ trước khi viết mã (rủi ro R-CSP-1) |
| S4' (thay S4) | PWA chỉ lưu dữ liệu người dùng trong IndexedDB của chính nguồn gốc | Kiểm tra trình duyệt sau kịch bản R7: liệt kê `localStorage` (chỉ khóa `lvhv.ui`), `sessionStorage` (rỗng), Cache Storage (chỉ tệp tĩnh), cookie (0), URL lịch sử (không chứa tên/ngày) | Đạt cả năm |
| S5 | Không sao lưu tự động lên đám mây của hệ điều hành | Android: manifest đã gộp có `allowBackup=false` và quy tắc trích xuất loại trừ; `adb shell bmgr backupnow` không chứa tệp dữ liệu. Apple: thuộc tính `isExcludedFromBackup` của hai đường dẫn | Đạt |
| S6 | Quyền Android đúng danh sách cho phép | `aapt2 dump permissions` trên APK release | Đúng và chỉ: `POST_NOTIFICATIONS`, `RECEIVE_BOOT_COMPLETED` (+ `SCHEDULE_EXACT_ALARM` nếu O-6 chọn), + quyền nội bộ `DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION` như v0.1. Không `INTERNET`, không quyền vị trí/danh bạ/bộ nhớ ngoài/ảnh toàn bộ (chọn ảnh qua Photo Picker) |
| S7 | Native không có kết nối mạng nếu người dùng không thao tác | Kết quả R7 phần native | 0 kết nối |
| S8 | Không tự viết mật mã | Rà mã (grep) | Chỉ các API của `argon2`, `chacha20poly1305`, `zeroize`, bộ sinh ngẫu nhiên của hệ điều hành (và `@noble/*` nếu O-7 (B)); 0 hàm tự cài phép XOR/hoán vị/băm cho mục đích mật mã |
| S9 | Ảnh không còn siêu dữ liệu | 10 ảnh mẫu có EXIF GPS, XMP, IPTC, ảnh HEIC iPhone, ảnh xoay 90° | Byte ảnh đã lưu và ảnh trong tệp sao lưu: 0 đoạn APP1 (EXIF/XMP), APP13 (IPTC); hướng ảnh đúng (đã xoay theo EXIF trước khi bỏ) |
| S10 | Xuất/chia sẻ luôn có xác nhận và xem trước | Kiểm thử giao diện | Mọi đường xuất (.ics, sao lưu, chia sẻ ảnh tưởng niệm nếu có) hiện hộp xác nhận ghi số người/số mục và loại dữ liệu; 0 đường xuất không qua hộp này; không có chia sẻ tự động |
| S11 | Phụ thuộc mật mã được ghim và không có cảnh báo bảo mật | `Cargo.lock`; `cargo deny check advisories` trên CI; `npm audit --omit=dev` nếu O-7 (B) | Ghim chính xác (`=x.y.z`); 0 cảnh báo chưa xử lý |
| S12 | Tệp sao lưu chịu tệp độc hại | R9 (b) + tệp `m_cost_kib` = 4 GiB, JSON lồng 10 000 cấp, 10 000 ảnh rỗng | Từ chối trước khi cấp bộ nhớ lớn (bộ nhớ đỉnh < 200 MB); ứng dụng không treo, không thoát |

## 4. Trình tự cổng

| Cổng | Nội dung | Ai ký |
|---|---|---|
| G1 | Đặc tả 01–05 và tiêu chí này khóa; các mục mở O-1 … O-9 (05 §6) được quyết định hoặc giữ mặc định | Chủ dự án |
| G2 | R1–R9 | Chủ dự án, sau biên bản của bên kiểm định độc lập |
| G3 | P3, P4, P6–P9 | Chủ dự án |
| G4 | S1, S2, S2b, S4', S5–S12; rà soát bảo mật và mô hình đe dọa cập nhật | Chủ dự án |

Phát hành 0.2.0 cần G1–G4. Tiêu chí CHƯA ĐỦ BẰNG CHỨNG không chặn bản thử nghiệm nội bộ nếu chủ dự án ghi chấp nhận
rủi ro trong biên bản; nó chặn bản phát hành công khai trên cửa hàng.
