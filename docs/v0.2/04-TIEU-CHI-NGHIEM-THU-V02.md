# Tiêu chí nghiệm thu — v0.2 (đăng ký trước)

**KHÓA (G1) — 2026-11-06, chờ chủ dự án ký.**

Các tiêu chí được khóa **trước khi viết mã** của v0.2. Không được nới ngưỡng, đổi phương pháp đo hay thu hẹp dữ liệu
sau khi đã thấy kết quả. Mọi thay đổi sau khi khóa phải ghi thành một "Bổ sung" có ngày, nêu rõ thay đổi được đề xuất
**trước** hay **sau** khi thấy kết quả (như docs/02 Bổ sung A1–A3 của v0.1), và cần chủ dự án ký.

Ghi chú đối chiếu: mã và mệnh đề R1–R9 được **căn theo kế hoạch chủ dự án duyệt 06/11/2026** (đối chiếu ngày
2026-11-06, trước khi có mã và trước khi có kết quả). Phần phương pháp, dữ liệu và ngưỡng đo được dưới mỗi mã là chi
tiết hóa mệnh đề đã duyệt, không nới mệnh đề nào. R10–R12 là tiêu chí bổ sung, đăng ký cùng lúc (không có trong danh
sách đã duyệt; chủ dự án có thể bỏ khi ký G1, nhưng không được bỏ sau khi thấy kết quả).

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

## 1. Chức năng (G2) — R1–R9 theo kế hoạch đã duyệt

### R1 — Ngày giỗ hằng năm so với tính toán độc lập (tháng nhuận, ngày 30), 1900–3000

| Mục | Nội dung |
|---|---|
| Mệnh đề đã duyệt | Ngày giỗ của ứng dụng khớp tính toán độc lập, gồm quy tắc tháng nhuận và ngày 30; **0 sai lệch, 1900–3000** |
| Cài đặt độc lập | Bên không tham gia xây dựng đặc tả viết lại `save_death` + `anniversary` + `label_of` trong `oracle/v02/` (Python), chỉ từ 01, 02 và tài liệu này |
| Dữ liệu | (a) **Toàn bộ** ngày mất dương 1900-01-01 → 3000-12-31 (402 132 ngày), mỗi ngày 10 năm giỗ kế tiếp (Y = năm âm mất + 1 … + 10, bỏ năm có ngày dương ngoài miền); (b) toàn bộ ngày mất âm "không rõ năm": 12 tháng × 30 ngày, Y = 1900 … 3000; (c) mọi ngày mất âm có năm 1900–2999 mà ngày đó tồn tại (gồm mọi tháng nhuận và mọi ngày 30), 10 năm giỗ mỗi ngày; (d) các ca G-A … G-E và ca biên 02 §9 |
| Hai lớp đối chiếu | **R1-quy tắc:** bản độc lập đổi lịch bằng tệp âm lịch đã công bố của lõi (`core/out/…`, cùng phương pháp `Precise`) — cô lập phần quy tắc giỗ. **R1-đầu-cuối:** bản độc lập đổi lịch bằng bảng âm lịch của **oracle v0.1** (PyEphem, `oracle/`) — kiểm cả quy tắc lẫn đổi lịch |
| Phương pháp | Hai bên xuất JSONL cùng định dạng (`{death, Y, lunar:{y,m,d}, solar:{y,m,d}, n, label}`), so từng byte sau khi sắp; bản TypeScript và lõi Rust giống nhau từng byte (như L7) |
| Ngưỡng | **0 sai lệch** ở mọi trường, (a)–(d), **ở cả hai lớp**, Rust ≡ TypeScript |
| Rủi ro đã biết (ghi trước, không đổi ngưỡng) | v0.1 đã ghi 3 đoạn ngày âm khác nhau giữa `Precise` và oracle trong 2101–3000 (docs/03 §15.5: 2242-07-28, 2962-10-19/20, 2988-06-05/06, Sóc cách nửa đêm < 6 phút). Mọi ngày mất hoặc ngày giỗ chạm các đoạn đó sẽ lệch ở **R1-đầu-cuối** → theo ngưỡng đã duyệt là **KHÔNG ĐẠT**. Biên bản phải liệt kê từng sai lệch kèm nguyên nhân; việc có chấp nhận loại sai lệch "Sóc sát nửa đêm" hay không là quyết định của chủ dự án, ghi thành Bổ sung, và chỉ hợp lệ nếu được quyết **trước** khi chạy R1 |
| Kết luận | ĐẠT khi cả hai lớp 0 sai lệch và Rust ≡ TypeScript. KHÔNG ĐẠT khi có một sai lệch ở bất kỳ lớp nào. CHƯA ĐỦ BẰNG CHỨNG khi bên độc lập chưa nộp hoặc thiếu một phần (a)–(d) |

### R2 — Đổi ngày mất dương → âm khi lưu, khứ hồi, 1900–3000

| Mục | Nội dung |
|---|---|
| Mệnh đề đã duyệt | Ngày mất nhập theo dương được đổi sang âm khi lưu, khứ hồi đúng; **0 lỗi, 1900–3000** |
| Dữ liệu | (a) Mọi ngày dương 1900-01-01 → 3000-12-31 (402 132 ngày) đi qua `save_death` thật (lớp lưu trữ của từng nền tảng có mã, không gọi thẳng hàm đổi lịch); (b) mọi ngày âm tồn tại trong năm âm 1900–2999 (gồm tháng nhuận, ngày 30) nhập trực tiếp theo âm |
| Kiểm | (a) `death.lunar` = `S2L(input)` của lõi; `L2S(death.lunar)` = `input`; `lunar_source = "converted"`; `method = "precise"`; đọc lại bản ghi từ kho (IndexedDB / SQLite) sau khi đóng và mở lại kho: giống hệt từng trường. (b) `death.solar` = `L2S(input)`; `S2L(death.solar)` = `input`; `lunar_source = "entered"`. (c) Ngày dương không tồn tại (30/02, 31/04…) và ngày âm không tồn tại (tháng nhuận không có, ngày 30 tháng thiếu) bị từ chối với thông báo đúng 01 §6.1, không ghi gì |
| Đối chiếu ngoài | `S2L` của lõi trên toàn dải đã được kiểm ở v0.1 (L2, L10, L12); R2 kiểm việc **lưu và đọc lại** không làm sai giá trị, không kiểm lại thiên văn |
| Ngưỡng | 0 lỗi ở (a), (b), (c), mỗi nền tảng có mã; Rust ≡ TypeScript |
| Kết luận | Theo từng nền tảng: ĐẠT / KHÔNG ĐẠT; CHƯA ĐỦ BẰNG CHỨNG nếu chưa chạy đủ dải trên nền tảng đó |

### R3 — Sinh lời nhắc: mọi tổ hợp lựa chọn

| Mục | Nội dung |
|---|---|
| Mệnh đề đã duyệt | Sinh lời nhắc đúng với mọi tổ hợp lựa chọn; **0 lỗi**. (Chi tiết hóa: `reminders_for` (02 §7.1) cho đúng tập (id, thời điểm, loại)) |
| Không gian tổ hợp (vét cạn) | Trạng thái giao diện: mọi tập con của các nút có sẵn {0, 1, 3, 5, 10, 15} (**64**) × mốc tự đặt ∈ {không có, 1, 2, …, 60} (**61**) = **3 904** trạng thái, cho ra **3 584** tập `offsets_days` khác nhau (mốc tự đặt trùng nút có sẵn thì gộp; tập rỗng = không nhắc). Chạy đủ 3 904 trạng thái (kiểm cả bước gộp) × `tien_thuong` ∈ {tắt, bật} (chỉ áp cho giỗ; loại khác phải bỏ qua) × `time_local` ∈ {00:00, 07:00, 23:59} × loại lần xuất hiện ∈ {giỗ, sinh nhật dương, sinh nhật âm, sự kiện dương lặp, sự kiện âm lặp, sự kiện không lặp} = **140 544** cấu hình |
| Lần xuất hiện | Bộ cố định 40 hồ sơ/sự kiện gồm mọi ca G-A … G-E, ca biên 02 §9 (giỗ mùng 1 Tết, giỗ 30 tháng Chạp, sinh 29/02, offset 60 qua năm), tính trên cửa sổ 2026-01-01 → 2036-12-31 với `now` = 2026-01-01T00:00 giờ thiết bị |
| Đối chiếu | Bộ sinh tham chiếu viết độc lập từ 02 §7.1 (cùng bên ở R1); so tập kết quả từng phần tử |
| Thêm | Tắt công tắc chung → 0 lời nhắc; `rule.enabled = false` → 0; lời nhắc có thời điểm ≤ `now` không được sinh; id ổn định khi đổi `time_local` |
| Ngưỡng | 0 sai lệch trên toàn bộ không gian (ghi tổng số lời nhắc đã so) |
| Kết luận | ĐẠT / KHÔNG ĐẠT; CHƯA ĐỦ BẰNG CHỨNG nếu chưa chạy đủ 140 544 cấu hình |

### R4 — Tệp .ics mở đúng trong Google, Apple, Outlook; ngày khớp

| Mục | Nội dung |
|---|---|
| Mệnh đề đã duyệt | Tệp .ics mở đúng trong Google, Apple, Outlook; ngày khớp. (Chi tiết hóa: tệp xuất đúng 02 §8) |
| (a) Cú pháp | Mọi tệp xuất từ bộ R3 qua một bộ phân tích RFC 5545 độc lập (thư viện Python `icalendar`, ghim phiên bản trong biên bản): 0 lỗi; mọi dòng ≤ 75 octet; CRLF; mọi VALARM có ACTION/DESCRIPTION/TRIGGER; không có RRULE; không có tham số TZID trên giá trị DATE; có VTIMEZONE `Asia/Ho_Chi_Minh` |
| (b) Ngữ nghĩa | Tập (UID, DTSTART) trong tệp ≡ tập lần xuất hiện của R11 trong 10 năm; UID giống nhau giữa hai lần xuất trước/sau khi sửa tên hồ sơ; SEQUENCE tăng |
| (c) Nhập thật | Nhập tệp mẫu 20 sự kiện vào: Google Calendar (web), Apple Calendar (iOS và macOS), Outlook (Windows hoặc web). Ghi: ngày hiển thị, lời nhắc có được giữ hay bị thay bằng mặc định của ứng dụng lịch |
| Ngưỡng | (a), (b): 0 lỗi. (c): ngày hiển thị đúng 20/20 ở mọi ứng dụng thử. Việc ứng dụng lịch bỏ VALARM **ghi nhận, không làm KHÔNG ĐẠT** (ngoài tầm kiểm soát), nhưng phải ghi vào hướng dẫn sử dụng |
| Kết luận | ĐẠT khi (a),(b),(c) đạt; CHƯA ĐỦ BẰNG CHỨNG nếu (c) thiếu ứng dụng nào |

### R5 — Thông báo đặt lịch trên Android/iPhone thật, kể cả sau khởi động lại

| Mục | Nội dung |
|---|---|
| Mệnh đề đã duyệt | Thông báo đặt lịch trên Android và iPhone thật hiện đúng, kể cả sau khởi động lại máy. (Chi tiết hóa: trong cửa sổ trễ cho phép, không cần mạng; thêm macOS và Windows cùng phương pháp) |
| Thiết bị thật | Mỗi nền tảng: 1 máy thật (Android ≥ 14; iPhone iOS ≥ 17; macOS ≥ 14; Windows 11). Chế độ máy bay bật suốt phép thử. Đặt 5 lời nhắc trong 2 giờ tới (đổi giờ hệ thống không được dùng). Một lần khởi động lại máy giữa chừng trên **mọi** nền tảng (sau lời nhắc thứ 2, trước lời nhắc thứ 3); với iPhone, không mở lại ứng dụng sau khi khởi động lại. Ghi thời điểm hiện thông báo (ảnh chụp màn hình có đồng hồ hệ thống) |
| Ngưỡng | 5/5 lời nhắc hiện; trễ so với thời điểm đặt: iOS/macOS ≤ 1 phút; Android ≤ 60 phút với báo thức không chính xác (≤ 1 phút nếu O-6 chọn báo thức chính xác và quyền được cấp); Windows ≤ 1 phút khi ứng dụng đã từng mở sau khởi động. Không hiện trước thời điểm đặt |
| PWA | Không áp dụng (02 §7.5) — PWA chấm thẻ nhắc trong trang: mở ứng dụng vào ngày có lời nhắc → thẻ hiện (kiểm thử tự động) |
| Kết luận | Theo từng nền tảng. Android và iPhone là bắt buộc cho ĐẠT; macOS, Windows chấm riêng. Thiếu máy thật → CHƯA ĐỦ BẰNG CHỨNG cho nền tảng đó (không thay bằng giả lập). Mô phỏng xoay vòng iOS là R12, không thay cho R5 |

### R6 — Sao lưu: xuất → nhập giống hệt; sai mật khẩu bị từ chối

| Mục | Nội dung |
|---|---|
| Mệnh đề đã duyệt | Xuất → nhập sao lưu cho dữ liệu giống hệt; sai mật khẩu bị từ chối. (Chi tiết hóa: tệp theo 03 §2; tệp bị sửa/dị dạng bị từ chối mà không ghi gì; các bản cài đặt đọc được tệp của nhau) |
| (a) Khứ hồi | Bộ dữ liệu R3 + 20 ảnh: xuất → nhập "Thay toàn bộ" trên máy trống → so toàn bộ dữ liệu (JSON chuẩn hóa + sha256 ảnh): giống hệt. Lặp lại cho mỗi nền tảng có mã |
| (a2) Sai mật khẩu | Mỗi tệp mẫu × 20 mật khẩu sai (rỗng, sai 1 ký tự, khác chuẩn hóa NFC/NFD của cùng chuỗi — phải được coi là **đúng**, đúng mật khẩu nhưng thêm dấu cách): mọi mật khẩu sai bị từ chối với một thông báo chung "Sai mật khẩu hoặc tệp bị hỏng", kho dữ liệu không đổi (so băm trước/sau); bản NFD của mật khẩu đúng được chấp nhận |
| (b) Chống sửa | Các tệp hỏng của 03 §2.5 + lật **từng bit** của 4 096 byte đầu và 4 096 byte cuối của một tệp hợp lệ (65 536 biến thể): 100 % bị từ chối, kho dữ liệu không đổi (so băm trước/sau) |
| (c) Vector chuẩn | Argon2id RFC 9106 §5.3 và XChaCha20-Poly1305 (`draft-irtf-cfrg-xchacha` A.3): khớp |
| (d) Kiểm chéo | Tệp xuất từ PWA nhập được vào native và ngược lại (mỗi cặp nền tảng có mã); nếu O-7 chọn thư viện JavaScript thì thêm: một bản giải mã độc lập (Python, thư viện `argon2-cffi` + `PyNaCl`/libsodium, viết bởi bên không tham gia xây dựng đặc tả) giải mã đúng 10 tệp mẫu |
| (e) Gộp | 01 §6.3: bộ 30 ca gộp (trùng id khác rev, trùng tên loại sự kiện, ảnh trùng) cho đúng kết quả mô tả |
| Ngưỡng | (a), (c), (d), (e): 0 sai lệch; (a2): 0 mật khẩu sai được chấp nhận; (b): 0 tệp hỏng được chấp nhận |
| Kết luận | ĐẠT / KHÔNG ĐẠT; CHƯA ĐỦ BẰNG CHỨNG nếu thiếu cặp nền tảng nào ở (d) |

### R7 — Không dữ liệu hồ sơ nào rời thiết bị khi không có thao tác chia sẻ (0 yêu cầu)

| Mục | Nội dung |
|---|---|
| Mệnh đề đã duyệt | Không dữ liệu hồ sơ nào rời thiết bị nếu người dùng không thực hiện thao tác chia sẻ (theo dõi mạng); **0 yêu cầu**. (Chi tiết hóa: native không mở kết nối mạng nào nếu người dùng không bấm một nút mở liên kết; xuất .ics và sao lưu chỉ ghi tệp qua hộp thoại của hệ điều hành, không phải yêu cầu mạng) |
| Kịch bản | Một lượt dùng đầy đủ: tạo 3 người đã khuất (có ảnh) + 2 người sống + 3 sự kiện, mở màn hình tưởng niệm 3 phút, bật nhắc, xuất .ics, xuất và nhập sao lưu, xóa dữ liệu |
| PWA | Chromium qua Playwright: ghi mọi yêu cầu của trang **và** Service Worker (CDP `Network` cho cả hai đích); ngưỡng: 0 yêu cầu tới nguồn gốc khác; yêu cầu cùng nguồn gốc chỉ là tệp tĩnh trong danh sách precache; 0 yêu cầu có thân (POST/PUT); 0 vi phạm CSP. Chạy thêm một lần ở chế độ ngoại tuyến: kịch bản hoàn tất |
| Android | (i) Tĩnh: `aapt2 dump permissions` không có `INTERNET` (khi không có quyền này hệ điều hành chặn mọi socket); (ii) động: trình giả lập, `tcpdump` trên giao diện mạng của trình giả lập trong suốt kịch bản, lọc theo UID của ứng dụng (`/proc/net/xt_qtaguid` hoặc `dumpsys netstats detail`) — 0 byte |
| iOS / macOS | (i) Tĩnh: mã Swift không dùng `URLSession`, `NWConnection`, `Network.framework`, `CFNetwork`, WebKit (grep + `nm` trên binary: 0 ký hiệu ngoài danh sách cho phép, chỉ `UIApplication.open`/`NSWorkspace.open` cho nút cập nhật); (ii) động: iPhone thật qua `rvictl` + `tcpdump` trên Mac, hoặc `nettop -p <pid>` cho macOS — 0 kết nối của tiến trình ứng dụng |
| Windows | (i) Tĩnh: `cargo tree` của ứng dụng không có crate HTTP/socket (`reqwest`, `hyper`, `ureq`, `tokio` với `net`); (ii) động: Resource Monitor/`netstat -abno` hoặc Process Monitor lọc theo tiến trình — 0 sự kiện TCP/UDP |
| Ngưỡng | 0 kết nối/0 byte ở mọi nền tảng trong kịch bản; ngoại lệ duy nhất: ngay sau khi người dùng bấm "Kiểm tra cập nhật" (hệ điều hành mở trình duyệt/cửa hàng — tiến trình khác) |
| Kết luận | Theo từng nền tảng; chỉ có kiểm tĩnh mà thiếu kiểm động → CHƯA ĐỦ BẰNG CHỨNG |

### R8 — Màn hình tưởng niệm: khói dừng khi bị ẩn, CPU gần 0 khi để yên 60 s

| Mục | Nội dung |
|---|---|
| Mệnh đề đã duyệt | Khói dừng khi màn hình bị ẩn; CPU gần 0 khi để yên 60 s. (Chi tiết hóa: "gần 0" = ≤ 1 % một lõi, trung bình trên cửa sổ 60 s; khói chỉ chạy khi hiển thị, tự dừng sau 2 phút, không chạy khi người dùng chọn giảm chuyển động; không âm thanh, không quảng cáo) |
| Đo PWA | Chromium (Playwright), khung nhìn 360×780, giả lập CPU chậm 4× (`Emulation.setCPUThrottlingRate`). Đếm lời gọi `requestAnimationFrame` bằng bộ đếm thử nghiệm trong bản dựng kiểm thử; đo thời gian CPU luồng chính bằng `Performance.getMetrics` (`TaskDuration`) mỗi cửa sổ 10 s; 3 lần chạy, lấy lần xấu nhất |
| Ngưỡng PWA | (a) 0–120 s: CPU luồng chính ≤ 15 % một lõi (sau giả lập 4×), trung bình mỗi cửa sổ 10 s; (b) để yên 60 s sau khi khói đã dừng (tức 121 s → 180 s): 0 lời gọi rAF và CPU ≤ 1 % trung bình cửa sổ 60 s; (c) tab ẩn (`visibilitychange`) hoặc rời màn hình ở giây thứ 30: 0 rAF trong vòng 1 s, rồi CPU ≤ 1 % trung bình 60 s kế tiếp; (d) `prefers-reduced-motion: reduce`: 0 rAF ngay từ đầu, khói hiển thị dạng tĩnh; (e) 0 phần tử `<audio>`/`<video>`, 0 lời gọi `AudioContext`; (f) Công giáo: 0 phần tử hương |
| Đo native | Android: `adb shell top -d 10 -p <pid>` và `dumpsys gfxinfo` (số khung) trên máy thật; iOS: Instruments (Time Profiler + Core Animation FPS); Windows: Process Explorer (CPU của tiến trình) |
| Ngưỡng native | (a) ≤ 10 % một lõi trung bình trong 120 s đầu; (b) để yên 60 s sau khi khói đã dừng: 0 khung vẽ mới (gfxinfo/Core Animation) và CPU ≤ 1 % trung bình cửa sổ 60 s; ẩn màn hình (về màn hình chính, khóa máy) ở giây thứ 30: dừng trong 1 s; (c), (d), (e), (f) như PWA (đọc cài đặt giảm chuyển động của hệ điều hành) |
| Nội dung | Kiểm danh sách tệp hình: mọi tệp trong `content/memorial/` có dòng nguồn "vẽ mới cho dự án, giấy phép …" trong `content/memorial/SOURCES.md`; 0 tệp lấy từ nguồn ngoài |
| Kết luận | Theo từng nền tảng; thiếu máy thật cho native → CHƯA ĐỦ BẰNG CHỨNG |

### R9 — Nội dung phong tục

| Mục | Nội dung |
|---|---|
| Mệnh đề đã duyệt (nguyên văn) | "Có biên bản ghi nguồn phong tục phổ biến đã theo; duyệt bởi người am hiểu được hoãn theo quyết định chủ dự án, ghi là rủi ro mở" |
| Căn cứ | Quyết định của chủ dự án ngày 06/11/2026: theo phong tục phổ biến của người Việt bây giờ, điều chỉnh sau |
| Biên bản | Tệp `content/memorial/PHONG-TUC.md`, mỗi lựa chọn phong tục một dòng: giỗ theo ngày âm; tháng nhuận → tháng thường; ngày 30 → 29; giỗ đầu; giỗ hết (năm 2, chọn 3); tiên thường; tuổi âm/hưởng thọ/hưởng dương và ngưỡng; dòng đầu bia (Hiển khảo, Hiển tỷ…); ba kiểu mâm (Phật giáo: hương hoa quả mâm chay; truyền thống: mâm cỗ mặn; Công giáo: nến và hoa, không hương); ba nén hương. Mỗi dòng: lựa chọn đã theo, **ít nhất 1 nguồn** kiểm chứng được (tên, tác giả/đơn vị, ngày truy cập hoặc trang sách), các nguồn khác ý kiến nếu có, và mã mục mở liên quan (O-1 … O-4) |
| Ngưỡng | 100 % lựa chọn hiện trong ứng dụng có dòng trong biên bản; 100 % dòng có ≥ 1 nguồn; mục "duyệt bởi người am hiểu" ghi là **hoãn, rủi ro mở R-CUL-1** (05 §7) |
| Kết luận | ĐẠT khi đạt ngưỡng trên — ĐẠT này **không** có nghĩa nội dung đã được người am hiểu duyệt. KHÔNG ĐẠT khi một lựa chọn trong ứng dụng không có trong biên bản hoặc không có nguồn. CHƯA ĐỦ BẰNG CHỨNG khi chưa có biên bản |

## 1b. Tiêu chí bổ sung (đăng ký cùng lúc, ngoài danh sách đã duyệt)

### R10 — Tuổi âm, hưởng thọ/hưởng dương, Can Chi năm sinh, quy tắc 29/02

| Mục | Nội dung |
|---|---|
| Mệnh đề | 02 §4 (`lunar_year`, `can_chi`, `tuoi_mat`, nhãn theo ngưỡng) và 02 §6 (29/02) cài đặt đúng |
| Dữ liệu | (a) Mọi ngày sinh dương 1900-01-01 → 2026-12-31 → `lunar_year` so với `LYOB` của oracle; (b) Can Chi của mọi năm 1899–3000 so với công thức hợp đồng §3 viết độc lập; (c) 10 000 cặp (sinh, mất) ngẫu nhiên (hạt `20261106`) gồm mọi tổ hợp kiểu nhập (dương/âm × đủ ngày/chỉ năm) và ngưỡng {59, 60, 61}; (d) 29/02 với mọi năm 1900–3000 × hai lựa chọn `feb28`/`mar1` |
| Ngưỡng | 0 sai lệch ở (a)–(d); ca sinh 1990-01-20 → Kỷ Tỵ, ca 1940-05-01/2025-08-08 → 86 hưởng thọ đúng |
| Kết luận | ĐẠT / KHÔNG ĐẠT theo ngưỡng; CHƯA ĐỦ BẰNG CHỨNG nếu thiếu (a)–(d) |

### R11 — Danh sách "Sắp tới" 30 ngày

| Mục | Nội dung |
|---|---|
| Mệnh đề | 02 §7.2: đúng tập lần xuất hiện trong [hôm nay, hôm nay + 30], đúng thứ tự, không phụ thuộc công tắc nhắc |
| Dữ liệu | Bộ hồ sơ của R3 × mọi "hôm nay" từ 2026-01-01 đến 2036-12-31 (4 018 ngày) |
| Ngưỡng | 0 sai lệch so với bộ tham chiếu độc lập; ngày đầu và ngày thứ 30 đều có mặt |
| Kết luận | ĐẠT / KHÔNG ĐẠT / CHƯA ĐỦ BẰNG CHỨNG |

### R12 — Mô phỏng xoay vòng 64 thông báo chờ của iOS

| Mục | Nội dung |
|---|---|
| Mệnh đề | `reschedule` (02 §7.4) không bao giờ vượt 64 yêu cầu chờ và không bỏ lời nhắc tương lai một cách lặng lẽ |
| Phương pháp | Hàm `reschedule` (02 §7.4) chạy trên bộ R3 với 10, 50, 200 hồ sơ; mô phỏng thời gian trôi 2026-01-01 → 2028-12-31, mỗi ngày "mở ứng dụng" với xác suất {1, 0,1, 0} (ba kịch bản). Bất biến kiểm sau mỗi bước: số yêu cầu chờ ≤ 64; nếu tổng tương lai ≥ 64 thì 63 cái sớm nhất đang chờ + thông báo nhắc mở ứng dụng ngay sau cái thứ 63; không bao giờ có lời nhắc tương lai bị bỏ mà không có thông báo nhắc mở ứng dụng đứng trước nó |
| Ngưỡng | 0 vi phạm bất biến ở cả ba kịch bản × ba cỡ dữ liệu |
| Kết luận | ĐẠT / KHÔNG ĐẠT; CHƯA ĐỦ BẰNG CHỨNG nếu thiếu kịch bản nào. R12 là kiểm logic, không thay cho R5 trên iPhone thật |

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
| S12 | Tệp sao lưu chịu tệp độc hại | R6 (b) + tệp `m_cost_kib` = 4 GiB, JSON lồng 10 000 cấp, 10 000 ảnh rỗng | Từ chối trước khi cấp bộ nhớ lớn (bộ nhớ đỉnh < 200 MB); ứng dụng không treo, không thoát |

## 4. Trình tự cổng

| Cổng | Nội dung | Ai ký |
|---|---|---|
| G1 | Đặc tả 01–05 và tiêu chí này khóa; các mục mở O-1 … O-9 (05 §6) được quyết định hoặc giữ mặc định | Chủ dự án |
| G2 | R1–R9 (đã duyệt) và R10–R12 (bổ sung) | Chủ dự án, sau biên bản của bên kiểm định độc lập |
| G3 | P3, P4, P6–P9 | Chủ dự án |
| G4 | S1, S2, S2b, S4', S5–S12; rà soát bảo mật và mô hình đe dọa cập nhật | Chủ dự án |

Phát hành 0.2.0 cần G1–G4. Tiêu chí CHƯA ĐỦ BẰNG CHỨNG không chặn bản thử nghiệm nội bộ nếu chủ dự án ghi chấp nhận
rủi ro trong biên bản; nó chặn bản phát hành công khai trên cửa hàng.

## Bổ sung B2 — quyết định ngày 2026-11-07

Chủ dự án giao cho điều phối viên chọn phương án tối ưu cho các mục còn mở (07/11/2026, "Em thấy cái gì tối ưu nhất
thì anh sẽ quyết làm theo hướng đó"). Các mục 1–3 được quyết **sau khi đã thấy kết quả** kiểm định, ghi rõ ở đây.
Mục 4 là yêu cầu mới, đăng ký **trước khi viết mã**.

1. **R12 (SAU KHI THẤY KẾT QUẢ).** Lần chạy trước KHÔNG ĐẠT bất biến I3, vì lời nhắc thứ 63 và thứ 64 có thể rơi cùng
   một phút. Nếu thông báo nhắc mở ứng dụng đặt ở phút sau cái thứ 63, thì cái thứ 64 cùng phút với cái thứ 63 bị bỏ
   mà không có thông báo nhắc đứng trước nó. Sửa 02 §7.4: `sentinel.at := keep[CAP-2].at` (cùng phút với cái thứ 63,
   xếp sau nó theo id), không phải `+ 1 phút`. Mệnh đề và ngưỡng của R12 giữ nguyên; chạy lại cả ba kịch bản × ba cỡ.
2. **Năm điểm đặc tả còn trống mà đợt 1 đã tự chọn (SAU KHI THẤY KẾT QUẢ)** được chấp nhận đúng như đã cài, và ghi
   vào 02: nhãn "Sắp đến sự kiện"; tiêu đề .ics "Tiên thường — …"; sự kiện dương lặp hằng năm vào 29/02 rơi vào 28/02
   ở năm không nhuận; tuổi trên 150 bị từ chối khi nhập; nhắc "đúng lúc" ghi `TRIGGER:PT0S` trong .ics.
3. **R1 với phương pháp Jpl (SAU KHI THẤY KẾT QUẢ).** R1 lần trước KHÔNG ĐẠT (2 932 sai lệch, đều trong ba đoạn Sóc sát
   nửa đêm 2242/2962/2988 của `Precise` so với bên kiểm định dùng PyEphem). Từ bản 0.1.4 lịch mặc định là `Jpl`
   (docs/02 Bổ sung A4), và bên kiểm định độc lập đã có lịch tự lập từ DE441 khớp lõi 402 132/402 132 ngày (L15).
   R1 được chạy lại với `Jpl`, đối chiếu với lịch DE441 của bên kiểm định, giữ nguyên ngưỡng 0 sai lệch và không có
   ngoại lệ sát nửa đêm. Kết quả `Precise` lần trước được giữ nguyên trong báo cáo, không xóa.
4. **R13 — danh sách sự kiện theo kỳ (yêu cầu mới, KHÓA trước khi viết mã).** Chủ dự án yêu cầu xem danh sách sự kiện
   của người đã mất và người đang sống theo tuần, tháng, quý, năm.

### R13 — Danh sách sự kiện theo tuần / tháng / quý / năm

| Mục | Nội dung |
|---|---|
| Mệnh đề | `events_in_period(dữ liệu, kỳ, ngày neo, bộ lọc)` trả **mọi** lần xuất hiện như 02 §7.2: giỗ, tiên thường nếu bật, sinh nhật dương, sinh nhật âm, sự kiện. Lấy những lần có ngày dương nằm trong kỳ chứa ngày neo. **Kỳ:** tuần từ Thứ Hai đến Chủ nhật; tháng dương; quý dương (tháng 1–3, 4–6, 7–9, 10–12); năm dương. **Bộ lọc:** tất cả / người đã mất (hồ sơ có ngày mất) / người đang sống (hồ sơ không có ngày mất); sự kiện theo trạng thái của hồ sơ chứa nó. **Thứ tự:** như §7.2. Mỗi mục có ngày dương, ngày âm, loại, hồ sơ, nhãn (giỗ đầu, giỗ hết, tiên thường, tuổi…). Không phụ thuộc công tắc nhắc |
| Dữ liệu | Bộ hồ sơ của R3; mọi tuần, tháng, quý và năm có ngày neo trong 2026-01-01 → 2036-12-31; cả ba bộ lọc; phương pháp mặc định (`Jpl`) |
| Ngưỡng | (a) 0 sai lệch so với bộ tham chiếu do bên kiểm định độc lập tự cài từ mệnh đề này; (b) Rust ≡ TypeScript từng byte; (c) nhất quán: hợp các tuần (cắt theo ranh giới) = tháng, hợp 3 tháng = quý, hợp 4 quý = năm, "đã mất" ∪ "đang sống" = "tất cả", không trùng mục; (d) ngày đầu và ngày cuối của mỗi kỳ đều được tính |
| Kết luận | ĐẠT / KHÔNG ĐẠT / CHƯA ĐỦ BẰNG CHỨNG |

## Bổ sung B3 — sau báo cáo kiểm định B2 (2026-11-07, SAU KHI ĐÃ THẤY KẾT QUẢ)

Chủ dự án đã giao cho điều phối viên chọn phương án tối ưu (xem B2). Báo cáo `docs/reports/BAO-CAO-KIEM-DINH-V02-B2.md`
của bên kiểm định độc lập nêu bốn điểm dưới đây. Mọi quyết định ở đây được đưa ra sau khi đã thấy kết quả.

1. **Lời nhắc ở xa bị bỏ lặng lẽ (R12, ô 10 hồ sơ / kịch bản không mở lại).** Cửa sổ lập lịch 400 ngày khiến lời nhắc ở
   xa hơn không bao giờ được đặt khi người dùng không mở lại ứng dụng, mà cũng không có thông báo nhắc mở ứng dụng.
   Lỗi này trái với mệnh đề R12 ("không bỏ lời nhắc tương lai một cách lặng lẽ"), dù không vi phạm bất biến đã đăng ký.
   Sửa 02 §7.4: thông báo nhắc mở ứng dụng được thêm **khi còn bất kỳ lời nhắc tương lai nào không được đặt**, vì
   vượt 63 cái hoặc vì nằm ngoài cửa sổ 400 ngày. Trường hợp đủ chỗ thì thông báo này đặt cùng phút với lời nhắc cuối
   cùng đã đặt, xếp sau nó. Thêm bất biến I4 vào R12: "nếu có lời nhắc tương lai không được đặt thì có thông báo nhắc mở
   ứng dụng không muộn hơn lời nhắc sớm nhất trong số đó". Ngưỡng 0 vi phạm cho cả ba kịch bản × ba cỡ. Chạy lại R12.
2. **R3 chưa kiểm được hai trường hợp.** Đó là công tắc nhắc chung tắt (phải ra 0 lời nhắc) và quy tắc nhắc
   `enabled = false` (phải ra 0). Đây là thiếu sót của công cụ kiểm thử, không phải của mệnh đề: lệnh dump được mở rộng
   để chạy hai trường hợp này, ngưỡng giữ nguyên.
3. **R13 chưa thử nhánh "sự kiện của người đã mất".** Bộ R3 không có sự kiện gắn với hồ sơ. Thêm một bộ dữ liệu bổ
   sung có sự kiện gắn với người đã mất và với người đang sống; R13 chạy trên cả bộ R3 và bộ bổ sung, ngưỡng giữ nguyên.
4. **Nhãn của dòng tiên thường trong R11.** Phụ lục A.3 không định nghĩa trường này; §7.6 định nghĩa nó là nhãn của lần
   giỗ ngày hôm sau. Chấp nhận cách hiểu của §7.6 và ghi vào A.3. Bên kiểm định đã sửa mã của mình **sau khi so sánh**;
   R11 vẫn ghi rõ điều đó.

Ngoài các tiêu chí: hai chỗ đếm khác nhau trong cột tổng hợp của R12 (lần rút số ngẫu nhiên ngày đầu; có tính thông
báo nhắc mở ứng dụng vào `delivered` hay không) được thống nhất trong Phụ lục A.5. Hai chỗ này không ảnh hưởng bất biến.

## Bổ sung B4 — với người đã mất chỉ nhắc ngày giỗ (2026-11-07, quyết định của chủ dự án)

Nguyên văn chủ dự án (07/11/2026, 06:12): "Người Việt nam không hay tổ chức lễ kỷ niệm ngày cưới, đặc biệt nếu 1 người
đã mất thì mọi người bỏ qua luôn sự kiện đó không nhắc tới, nên với người đã mất, sự kiện duy nhất cần báo là ngày giỗ,
mặc dù các dữ liệu khác vẫn còn lưu."

Đây là thay đổi yêu cầu sản phẩm, được quyết **sau khi** đã có kết quả kiểm định R3, R11, R12, R13. Ngưỡng các tiêu chí
giữ nguyên; tập lần xuất hiện được định nghĩa lại như sau và mọi tiêu chí liên quan phải chạy lại.

1. **Hồ sơ có ngày mất:** chỉ sinh ra lần xuất hiện **giỗ**, và **tiên thường** nếu người dùng bật (O-1, mặc định tắt).
   Sinh nhật dương, sinh nhật âm và sự kiện của người này **không** sinh lần xuất hiện, không có lời nhắc, không vào
   "Sắp tới", danh sách theo kỳ hay tệp .ics. Dữ liệu vẫn được lưu, hiển thị trên trang hồ sơ, và vẫn có trong sao lưu.
2. **Sự kiện gắn với nhiều người:** nếu **có ít nhất một** người đã mất thì sự kiện không sinh lần xuất hiện, với lý
   do như mục 1. Sự kiện chỉ gắn với người đang sống, hoặc không gắn với ai, sinh lần xuất hiện như cũ.
3. **Bộ lọc R13:** "Người đã mất" = giỗ và tiên thường; "Người đang sống" = mọi lần xuất hiện còn lại. Yêu cầu
   "đã mất ∪ đang sống = tất cả, không trùng" giữ nguyên.
4. **Chuyển trạng thái:** khi một người đang sống được cập nhật ngày mất, các lời nhắc sinh nhật và sự kiện của người
   đó bị gỡ ở lần lập lịch kế tiếp (02 §7.3–7.5).
5. **Phạm vi chạy lại:** R3, R4, R11, R12, R13 (bên kiểm định độc lập tự cài lại từ mục 1–3); kiểm thử giao diện
   các nền tảng.

## Bổ sung B5 — nâng cấp không mất dữ liệu (YC-2026-08, KHÓA 2026-11-07, trước khi viết mã)

Nguyên văn chủ dự án (07/11/2026, 10:48): "phần mềm phải đảm bảo khi nâng cấp không làm mất dữ liệu đã có của khách
hàng và updated vào version mới trên máy tính, hoặc điện thoại của khách hàng nhé".

Nguyên tắc (bổ sung cho 01 §7):
1. Mọi bản phát hành mới phải mở được dữ liệu của **mọi bản đã phát hành trước** trên cùng nền tảng, tự chuyển sang
   lược đồ mới, và không mất, không đổi nghĩa bất kỳ trường nào.
2. Không có phép chuyển đổi xóa dữ liệu. Trường bị bỏ hay đổi tên phải được chuyển sang trường mới. Nếu không chuyển
   được thì giữ nguyên trong một khóa `legacy` thay vì xóa.
3. Trước khi chuyển đổi, ứng dụng tự giữ một bản sao dữ liệu cũ trên máy (01 §7). Nếu chuyển đổi lỗi, ứng dụng quay
   về dữ liệu cũ, mở ở chế độ chỉ đọc và hướng dẫn xuất sao lưu. Không bao giờ xóa.
4. Cập nhật ứng dụng (bản web thay service worker; Android cài đè; iPhone/Mac cập nhật; Windows cài đè bằng MSI mới)
   không được xóa dữ liệu. Gỡ cài đặt trên Windows không tự xóa thư mục dữ liệu.
5. Tệp sao lưu tạo từ mọi bản đã phát hành trước đều phải nhập được vào bản mới.
6. Từ bản 0.2.0, mỗi bản phát hành lưu một **bộ dữ liệu mẫu đóng băng** của chính nó trong
   `test-vectors/released/<phiên bản>/` (cơ sở dữ liệu hoặc kho của từng nền tảng, kèm tệp sao lưu). Các bộ này không
   bao giờ được sửa.

### R14 — Nâng cấp tại chỗ giữ nguyên dữ liệu

| Mục | Nội dung |
|---|---|
| Mệnh đề | Với mỗi nền tảng: cài bản cũ, tạo dữ liệu, cài bản mới đè lên. Sau khi mở bản mới, mọi hồ sơ, sự kiện, ảnh, tệp đính kèm, cài đặt và lịch nhắc đều còn nguyên và đã chuyển sang lược đồ mới |
| Bản cũ phải thử | (a) 0.1.4, bản đang phát hành: cài đặt giao diện (cỡ chữ, sáng/tối) và trạng thái "Để sau" phải giữ nguyên. (b) Từ 0.2.0 trở đi: mọi bản đã phát hành có bộ mẫu ở `test-vectors/released/`. (c) Trước khi phát hành 0.2.0: bộ mẫu tạo bằng bản dựng 0.2 trước thay đổi lược đồ gần nhất (ví dụ `certificate_photo_ids` → `attachments`) |
| Phương pháp | Bản web: trình duyệt tự động nạp bản cũ, ghi dữ liệu vào IndexedDB, thay bằng bản mới qua service worker như người dùng thật. Android: trình giả lập trên CI, `adb install` bản cũ, tạo dữ liệu, `adb install -r` bản mới. Windows: trên máy CI, cài MSI cũ, tạo dữ liệu, cài MSI mới (nâng cấp lớn), rồi gỡ cài đặt và kiểm tra thư mục dữ liệu còn nguyên. iPhone/Mac: trình giả lập, cài đè bản mới. Mọi nền tảng: so dữ liệu sau nâng cấp với dữ liệu kỳ vọng do hàm chuyển đổi thuần (01 §7) sinh ra; thử thêm một ca chuyển đổi lỗi và kiểm tra bản sao dữ liệu cũ còn nguyên |
| Ngưỡng | 0 bản ghi mất; 0 trường khác với kỳ vọng; ca lỗi không mất dữ liệu; lịch nhắc được đặt lại sau nâng cấp |
| Kết luận | Theo từng nền tảng: ĐẠT / KHÔNG ĐẠT / CHƯA ĐỦ BẰNG CHỨNG. **Chặn phát hành** nếu KHÔNG ĐẠT |

## Bổ sung B6 — kiểm tra ảnh và tệp đính kèm khi nhập sao lưu (2026-11-07, SAU KHI ĐÃ THẤY KẾT QUẢ)

Lý do: bản rà soát bảo mật `docs/reviews/RA-SOAT-BAO-MAT-V02.md` (BM2-01) chứng minh đường nhập sao lưu lưu nguyên byte
ảnh và tệp đính kèm, không kiểm định dạng hay kích thước điểm ảnh. Một ảnh PNG 20000×20000 dán nhãn `image/jpeg`
(cần khoảng 1,6 GB để giải mã) đã được nhận. 03 §BK-S1 ghi biện pháp "ảnh mã hóa lại", nhưng §2.3 không quy định cách
làm.

Quyết định của điều phối viên, theo ủy quyền của chủ dự án (B2):
1. **Không** mã hóa lại ảnh khi nhập. Lý do: mã hóa lại làm đổi byte và `sha256`, trái với R6 (a)/(d), vốn yêu cầu dữ
   liệu sau khôi phục giống hệt. Ảnh do ứng dụng tạo đã được mã hóa lại và bỏ siêu dữ liệu lúc chụp hoặc chọn (01 §3.8),
   nên tệp sao lưu hợp lệ vẫn qua được.
2. Thay vào đó, lõi (Rust và TypeScript, giống hệt nhau) **kiểm tra chặt** mọi ảnh và tệp đính kèm, trước khi ghi bất
   cứ thứ gì, ở cả bốn đường: nhập sao lưu, `add_photo`, `add_attachment` và chuyển đổi lược đồ.
   - Chữ ký tệp khớp `mime`.
   - JPEG và WebP: đọc kích thước từ phần đầu tệp, không giải mã; kích thước phải bằng bản ghi và ≤ giới hạn của
     01 §3.8 / §3.8a.
   - JPEG: không được có đoạn APP1 (EXIF/XMP) hay APP13 (IPTC). WebP: không được có khối EXIF hay XMP.
   - PDF: bắt đầu bằng `%PDF-`, có `%%EOF` gần cuối, kích thước ≤ 10 MiB. PDF vẫn được lưu nguyên bản (R-PDF-1).
   - Sai bất kỳ điều nào thì từ chối **toàn bộ** tệp sao lưu, với thông báo chung.
3. Mỗi nền tảng giải mã ảnh để hiển thị phải đặt giới hạn kích thước điểm ảnh, lấy mẫu xuống trước khi giải mã đầy
   đủ (BM2-10).
4. Bổ sung vào bộ kiểm thử của S9 và S12 các tệp độc hại sau: ảnh khổng lồ, ảnh sai nhãn, ảnh còn EXIF, "ảnh" thực chất
   là HTML, PDF giả. Ngưỡng của S9 và S12 giữ nguyên.

## Bổ sung B7 — bộ dữ liệu R6 dùng ảnh thật (2026-11-07, SAU KHI ĐÃ THẤY KẾT QUẢ)

Lý do: không thể bật đầy đủ kiểm tra ảnh khi nhập sao lưu theo B6 (BM2-01). Bộ dữ liệu R6 đã khóa (`dataset-r6.json` và
tệp có salt cố định) chứa 20 "ảnh" chỉ là byte giả, không có phần đầu ảnh hợp lệ, trong đó 10 tệp còn mang nhãn
`image/webp`. Bật kiểm tra thì chính bộ R6 bị từ chối. Bỏ kiểm tra ảnh chân dung khi nhập thì lỗ hổng làm sập ứng dụng
vẫn còn. Quyết định của điều phối viên, theo ủy quyền của chủ dự án (B2), ưu tiên an toàn:
1. Tạo bộ **R6v2**: cùng cấu trúc, cùng số hồ sơ và số ảnh như R6, nhưng ảnh là JPEG và WebP thật cỡ nhỏ, đã bỏ siêu
   dữ liệu, kích thước khớp bản ghi. Bộ này được khóa từ khi tạo, kèm `sha256` của mọi tệp.
2. R6 (a)–(d) chạy trên R6v2, ngưỡng giữ nguyên. Bộ R6 cũ giữ nguyên trong kho để đối chiếu lịch sử. Kỳ vọng mới với
   bộ R6 cũ là bị từ chối khi nhập, vì nó chứa ảnh giả.
3. Bật kiểm tra B6 cho ảnh chân dung ở đường nhập sao lưu, như B6 đã quy định.
4. Bên kiểm định độc lập chạy lại R6 trên R6v2.

## Bổ sung B8 — ảnh mâm lễ (YC-2026-10, KHÓA 2026-11-07, trước khi viết mã)

Nguyên văn chủ dự án (07/11/2026): "anh muốn software có chế độ cập nhật ảnh mâm lễ, với bức ảnh hiện tại anh đánh giá
chất lượng thấp, không phù hợp. Khi chưa có ảnh mâm lễ phù hợp thì ngày giỗ sẽ chưa hiện mâm giỗ ra mà chi có bát hương
với nến bên dưới bia mộ như đã thiết kế"; "clevix cập nhật qua app, còn từng người dùng có thể tự chụp ảnh mâm lễ của
mình để load lên software cho ngày giỗ của người nhà mình". Thiết kế: 07 §10 (ADR-V02-01), 01 §3.7, 08 §16.
Khóa theo ủy quyền của chủ dự án ("Em thấy cái gì tối ưu nhất thì anh sẽ quyết làm theo hướng đó").

R8 (a)–(f) giữ nguyên, trừ phần nói tới "mâm": từ nay R8 không đòi vẽ mâm.

### R15 — Ảnh mâm lễ

| Mục | Nội dung |
|---|---|
| Mệnh đề | (a) Không có ảnh mâm lễ cho người đó và không có ảnh Clevix đã duyệt cùng `tray_style` → màn hình tưởng niệm **không** có hình mâm hay đồ cúng nào; dưới bia chỉ có bát hương ba nén và hai nến (Công giáo: hai nến và hoa, không hương). (b) Gói ảnh Clevix chỉ dùng mục `trang_thai = "duyet"`; mục đã duyệt mà `sha256` sai, không qua kiểm tra B6, hoặc quá giới hạn (≤ 400 KB mỗi ảnh, ≤ 2 MB cả gói) → **bản dựng thất bại**; gói rỗng → như (a). (c) Ảnh gia đình chọn từ tệp có EXIF/GPS/XMP → ảnh lưu không còn APP1, APP13, EXIF, XMP; cạnh dài ≤ 1600 px; ≤ 1 MB; chỉ hiện trên màn hình tưởng niệm của đúng người đó, ưu tiên hơn ảnh Clevix. (d) Xuất rồi nhập (Thay toàn bộ và Gộp) giữ nguyên ảnh (`sha256` bằng nhau) và liên kết; tệp sao lưu có `tray_photo_id` trỏ ảnh không tồn tại → từ chối (V3); ảnh giả hay còn siêu dữ liệu → từ chối (B6). (e) Dữ liệu và tệp sao lưu của bản trước (không có trường) mở được, `tray_photo_id = null`, không mất gì; tài liệu không dùng trường giống hệt từng byte bản trước (vector 03 §2.5 không đổi). (f) Bỏ ảnh, đổi ảnh, xóa hồ sơ, đổi về đang sống → ảnh cũ bị xóa (bản ghi và tệp) trong cùng giao dịch; ảnh không bao giờ có trong thông báo, .ics, tên tệp, URL. (g) Rust ≡ TS trên kịch bản `tray_photo` của `lvhv-store-tool` |
| Phương pháp | (a)(b): kiểm thử giao diện trên bốn nền tảng (không có phần tử mâm khi không có ảnh; có ảnh khi có) + ảnh chụp 360 px và 1280 px, sáng và tối; kịch bản kiểm gói chạy trong CI với gói rỗng, gói có mục chờ duyệt, gói có mục sai `sha256`. (c)–(g): kiểm thử lõi hai bản và `lvhv-store-tool`; bên kiểm định độc lập chạy (c)(d)(e)(g) |
| Ngưỡng | Mọi mệnh đề đúng; 0 ảnh còn siêu dữ liệu; 0 ảnh mồ côi sau (f) |
| Kết luận | Theo từng nền tảng: ĐẠT / KHÔNG ĐẠT / CHƯA ĐỦ BẰNG CHỨNG. (a) KHÔNG ĐẠT trên nền tảng nào thì **chặn phát hành** nền tảng đó |

