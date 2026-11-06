# Kế hoạch triển khai — v0.2

Trạng thái: **KHÓA (G1) — 2026-11-06, chờ chủ dự án ký.** Kế hoạch đã được chủ dự án duyệt ngày 06/11/2026; tài
liệu này ghi lại phạm vi, các đợt, phân vai, quy tắc nhánh, nơi chạy và các mục còn mở.

## 1. Phạm vi

Có trong v0.2 (chỉ trên thiết bị, không tài khoản, không máy chủ):
- Hồ sơ người đã khuất và người đang sống (01 §3.1–§3.3); ảnh chân dung chỉ trên máy (01 §3.8).
- Ngày giỗ theo âm lịch với quy tắc tháng nhuận, ngày 30, giỗ đầu/giỗ hết, tiên thường tùy chọn (02 §2–§3).
- Sinh nhật dương (tùy chọn thêm sinh nhật âm), tuổi, Can Chi năm sinh, quy tắc 29/02; sự kiện theo người với loại
  tự tạo dùng lại được (02 §4–§6).
- Lời nhắc 1/3/5/10/15 ngày trước + đúng ngày + tự đặt 1–60 ngày, mặc định đúng ngày và 3 ngày trước lúc 07:00;
  danh sách "Sắp tới" 30 ngày; thông báo cục bộ trên Android/iOS/macOS/Windows; PWA có thẻ nhắc trong ứng dụng và
  xuất .ics 10 năm (02 §7–§8).
- Màn hình tưởng niệm ngày giỗ (bia, ảnh, mâm theo 3 phong cách, ba nén hương có khói nhẹ, lời tưởng niệm của người
  dùng); trang nghiêm, không quảng cáo, không âm thanh; hình vẽ mới cho dự án. Nội dung theo phong tục phổ biến của
  người Việt; điều chỉnh sau.
- Lưu trữ trên thiết bị (IndexedDB / SQLite); xuất và nhập tệp sao lưu mã hóa bằng mật khẩu (03).

Không có trong v0.2 (để v0.3 trở đi): tài khoản, đồng bộ giữa thiết bị, chia sẻ hồ sơ cho người khác qua số điện
thoại, sao lưu lên máy chủ, thông báo đẩy từ máy chủ, mã hóa kho cục bộ, bài khấn, thông báo tang gia, album kỷ niệm.

## 2. Các đợt

| Đợt | Mục tiêu | Sản phẩm | Kết thúc khi |
|---|---|---|---|
| **0 (G1)** — tài liệu này | Đặc tả và tiêu chí khóa trước khi viết mã | `docs/v0.2/01–05`, lược đồ JSON, mục CHANGELOG | Chủ dự án ký G1 và quyết định các mục mở §6 |
| **1** — lõi thuật toán và mô hình | Mã thuần, không giao diện | Crate `lvhv-memorial` (Rust, `#![forbid(unsafe_code)]`, chỉ phụ thuộc `lvhv-core`): kiểu dữ liệu 01, kiểm tra §6.2, `save_death`, `anniversary`, `label_of`, `tuoi_mat`, `solar_occurrence`, `reminders_for`, `upcoming`, `reschedule` (mô phỏng), sinh .ics; bản TypeScript tương ứng trong `tsport/`; bộ dữ liệu kiểm thử; dump JSONL cho R1–R4. Song song: bên kiểm định độc lập viết `oracle/v02/` từ tài liệu | R1–R4, R5(a), R6(a)(b) chạy được; Rust ≡ TypeScript từng byte |
| **2** — lưu trữ và sao lưu | Kho dữ liệu, chuyển đổi phiên bản, tệp sao lưu | Crate `lvhv-store` (rusqlite) và `lvhv-backup` (argon2 + chacha20poly1305, ghim); lớp IndexedDB của PWA; cách chạy mật mã trong PWA theo O-7; vector kiểm thử 03 §2.5 | R9, P8, S8, S11, S12 |
| **3** — giao diện và thông báo | Màn hình hồ sơ, danh sách "Sắp tới", cài đặt nhắc, xuất .ics, sao lưu; thông báo cục bộ từng nền tảng | PWA; Android (Compose + AlarmManager + receiver khởi động); iOS/macOS (SwiftUI + UNUserNotificationCenter xoay vòng 64); Windows (Slint + thông báo đặt lịch). FFI mở rộng cho `lvhv-memorial`, `lvhv-store`, `lvhv-backup` | R5(b), R6(c), R7, P3, P4, P7, P9, S2b, S4', S5, S6, S7, S9, S10 |
| **4** — màn hình tưởng niệm, nghiệm thu | Bia, ảnh, ba kiểu mâm, hương và khói; tài nguyên tải lười; rà soát; biên bản | `content/memorial/` (hình vẽ mới, `SOURCES.md`), màn hình trên mọi nền tảng; hướng dẫn sử dụng cập nhật; rà soát bảo mật và mô hình đe dọa v0.2; biên bản G2–G4 | R8, P6; G2–G4 được trình ký |

Thứ tự phụ thuộc: 1 → 2 → 3 → 4. Bên kiểm định độc lập (R1–R4, R9(d)) chạy song song với đợt 1–2. Phần hình vẽ của
đợt 4 có thể bắt đầu cùng đợt 2 vì không phụ thuộc mã.

## 3. Phân vai

Giữ cách phân vai của v0.1 (hợp đồng kiến trúc §5): mỗi vai chỉ ghi vào thư mục của mình; vai kiểm định không đọc mã
của vai bị kiểm định trước khi nộp kết quả.

| Mã | Vai | Thư mục được ghi | Không được đọc (trước khi nộp) |
|---|---|---|---|
| B0 | Điều phối: tích hợp, nhánh, PR, biên bản | `docs/v0.2/`, `docs/reports/`, `CHANGELOG.md` | — |
| B1 | Thuật toán giỗ/nhắc/.ics (Rust + TypeScript) | `memorial/` (crate mới), `tsport/src/memorial*` | `oracle/` |
| B2 | Lưu trữ và sao lưu | `store/`, `backup/` (crate mới), `pwa/src/store*` | `oracle/` |
| B3 | Kiểm định độc lập — **bên không tham gia xây dựng đặc tả** | `oracle/v02/` | `core/`, `tsport/`, `memorial/`, `store/`, `backup/`, `pwa/`, `apps/` |
| B4 | PWA (giao diện, thẻ nhắc, xuất .ics, màn hình tưởng niệm) | `pwa/` | `oracle/` |
| B5 | Android | `apps/android/` | `oracle/` |
| B6 | Apple (iOS, macOS) | `apps/apple/` | `oracle/` |
| B7 | Windows | `apps/windows/` | `oracle/` |
| B8 | Hình vẽ và nội dung tưởng niệm | `content/memorial/` | — |
| B9 | Bảo mật / duyệt mã (người duyệt không viết mã được duyệt) | `docs/reviews/` | — |
| FFI | Thuộc B1 cho phần kiểu dữ liệu; B5–B7 chỉ đọc | `ffi/` | — |

## 4. Nhánh và pull request

- `main` luôn dựng được và qua CI. Không đẩy thẳng lên `main`.
- Mỗi đợt một nhánh `v0.2-w<N>-<vai>` (ví dụ `v0.2-w1-memorial`), tách từ `main` mới nhất; đợt 0 là `v0.2-g1`.
- Mỗi PR: một mục đích; mô tả gồm tiêu chí liên quan (Rxx/Pxx/Sxx), cách kiểm, kết quả kiểm; tài liệu không nêu tên
  sản phẩm hay nhà cung cấp công cụ hỗ trợ viết mã; người duyệt độc lập được gọi là "bên không tham gia xây dựng đặc
  tả".
- **Không tự gộp (merge).** Gộp PR của một đợt cần điều phối viên ký xác nhận; gộp PR kết thúc một cổng cần chủ dự án
  ký cổng đó.
- Thay đổi tài liệu đã khóa (01–05 của v0.2, docs/02 của v0.1) → PR riêng, ghi "Bổ sung" có ngày và lý do, ghi rõ đề
  xuất trước hay sau khi thấy kết quả.
- Phiên bản: `VERSION` giữ `0.1.1` cho đến khi phát hành; đổi sang `0.2.0` trong PR phát hành cuối đợt 4, cùng lúc đổi
  tiêu đề `## [Chưa phát hành — 0.2]` của `CHANGELOG.md` thành `## [0.2.0] — <ngày>` (bản dựng PWA đọc ghi chú phát
  hành theo đúng tiêu đề số phiên bản).

## 5. Chạy ở đâu

| Việc | Nơi chạy | Ghi chú |
|---|---|---|
| Test đơn vị, R1–R4 (cửa sổ ngắn), R5(a), R6(a)(b), R9(a)(b)(c)(e), lược đồ JSON | CI GitHub Actions (`ci.yml` mở rộng) và môi trường dựng của các vai | Nhẹ, vài phút |
| R1 đầy đủ (73 049 ngày × 10 năm + 2 000 ngày xa), R3 đầy đủ (140 544 cấu hình × 10 năm), R4 (4 018 ngày) | **Máy trạm của chủ dự án** (gói script chạy sẵn, không cần sửa tay) hoặc CI nếu dưới 30 phút | Kết quả thô nén lại, gửi về để chấm |
| P3, P4, P6, P7 (PWA), R7 (PWA), R8 (PWA), S2b, S4' | CI (Chromium qua Playwright), như `metrics.json` v0.1 | — |
| Dựng Android/Apple/Windows | CI (`android.yml`, `apple.yml`, `windows.yml`) | Apple cần runner macOS |
| R5(b), R6(c), R7 native động, R8 native, P8, P9 | **Thiết bị thật của chủ dự án** (Android ≥ 14, iPhone, Mac, PC Windows 11) | Gửi kèm hướng dẫn từng bước và script; thiếu thiết bị → CHƯA ĐỦ BẰNG CHỨNG cho nền tảng đó |
| Nhập .ics vào Google Calendar, Apple Calendar, Outlook (R6(c)) | Chủ dự án | Cần tài khoản của chủ dự án; tệp mẫu do B4 tạo |

## 6. Mục còn mở — cần chủ dự án quyết định trước khi gộp G1

Mỗi mục có **giá trị mặc định** đã ghi trong đặc tả; nếu chủ dự án không đổi thì giá trị mặc định là quyết định.

| Mã | Câu hỏi | Mặc định trong đặc tả | Ảnh hưởng |
|---|---|---|---|
| **O-1** | **Ngày tiên thường** (lễ hôm trước ngày giỗ): bật hay tắt nhắc mặc định? | **TẮT** (người dùng tự bật theo từng hồ sơ) | 01 §3.6, 02 §3, R3 |
| O-2 | Giỗ hết vào năm thứ 2 hay thứ 3 sau năm mất? | Năm thứ 2, cho chọn 3 theo từng hồ sơ | 01 §3.7, 02 §3 |
| O-3 | Ngưỡng tuổi phân biệt "hưởng dương" / "hưởng thọ" | Dưới 60 tuổi âm là hưởng dương; sửa được | 01 §3.7, 02 §4.2 |
| O-4 | Giỗ người mất tháng nhuận rơi vào năm có cùng tháng nhuận: giữ tháng thường (như quy tắc) hay theo tháng nhuận? | Luôn tháng thường | 02 §2.2, ca G-E |
| O-5 | Sự kiện trong .ics là sự kiện cả ngày (ngày "trôi", có VTIMEZONE và X-WR-TIMEZONE) hay sự kiện có giờ `TZID=Asia/Ho_Chi_Minh`? | Cả ngày | 02 §8, R6 |
| O-6 | Android: báo thức không chính xác (trễ tới ~1 giờ, không cần quyền đặc biệt) hay xin `SCHEDULE_EXACT_ALARM`? | Không chính xác | 02 §7.3, R5, S6 |
| O-7 | PWA chạy mật mã bằng WebAssembly (cần thêm `'wasm-unsafe-eval'` vào CSP) hay thư viện JavaScript `@noble/*` (CSP không đổi, Argon2 chưa kiểm toán)? | WebAssembly + ngoại lệ CSP duy nhất | 03 §4.3, S2b |
| O-8 | Chấp nhận crate `argon2` chưa có kiểm toán độc lập công bố (bù bằng vector chuẩn + kiểm chéo + ghi rõ trong tài liệu phát hành)? | Chấp nhận có điều kiện (03 §4.2 phương án A) | 03 §4, S8, S11 |
| O-9 | Nội dung thông báo trên màn hình khóa có hiện tên người không? | Có hiện; có tùy chọn "ẩn tên trên màn hình khóa" | 02 §7.1 |

## 7. Rủi ro đã thấy ở G1 (nguồn truy cập 2026-11-06)

| Mã | Rủi ro | Bằng chứng | Biện pháp / ảnh hưởng |
|---|---|---|---|
| R-AND-1 | **Quyền báo thức chính xác trên Android.** Từ Android 14, `SCHEDULE_EXACT_ALARM` **không còn được cấp sẵn** cho hầu hết ứng dụng mới cài nhắm Android 13+ (mặc định bị từ chối); cần cho `setExact()`, `setExactAndAllowWhileIdle()`, `setAlarmClock()`, nếu không sẽ ném `SecurityException`. `USE_EXACT_ALARM` được cấp khi cài và người dùng không thu hồi được, nhưng chỉ dành cho ứng dụng đồng hồ báo thức hoặc ứng dụng lịch, và Google Play không cho phát hành nếu ứng dụng không đủ điều kiện theo chính sách | developer.android.com/about/versions/14/changes/schedule-exact-alarms; developer.android.com/training/scheduling/alarms; esper.io "Android 13 exact alarm API restrictions" (tóm tắt chính sách Play: chỉ ứng dụng báo thức/đồng hồ, hoặc ứng dụng lịch hiện thông báo sự kiện sắp tới). **Chưa đọc được trực tiếp** trang chính sách Play Console (support.google.com, bị chặn từ môi trường làm việc) — chủ dự án cần đọc trước khi chọn O-6 | Mặc định dùng báo thức không chính xác `setAndAllowWhileIdle` (giao trong vòng ~1 giờ từ Android 12, trừ khi bị tiết kiệm pin hạn chế). Không dùng `USE_EXACT_ALARM` khi chưa có xác nhận chính sách. Cần `RECEIVE_BOOT_COMPLETED` vì báo thức bị hủy khi tắt máy. CI phải đổi từ "cấm mọi quyền" sang danh sách cho phép (S6) |
| R-AND-2 | Tiết kiệm pin của hãng (Xiaomi, Oppo, Samsung…) có thể trì hoãn hoặc chặn báo thức của ứng dụng ít mở | Kinh nghiệm chung; chưa đo | R5(b) đo trên máy thật; hướng dẫn sử dụng có mục "nếu không nhận được lời nhắc" |
| R-IOS-1 | **Giới hạn 64 thông báo cục bộ đang chờ** mỗi ứng dụng iOS; giới hạn hệ thống, không vượt được | Kỹ sư Apple trả lời trên developer.apple.com/forums/thread/811171 ("there is a limit of 64 … This is a system limit"); không tìm thấy câu này trong tài liệu API chính thức | Xoay vòng 63 + thông báo nhắc mở ứng dụng (02 §7.4); R5(a) kiểm bất biến. Người không mở ứng dụng nhiều tháng sẽ chỉ nhận thông báo nhắc mở ứng dụng |
| R-PWA-1 | **Trình duyệt có thể xóa dữ liệu PWA.** WebKit (Safari 17+): có `navigator.storage.persist()`, cấp theo suy đoán (ví dụ đã thêm vào màn hình chính); khi thiếu chỗ, xóa theo nguồn gốc dùng lâu nhất (LRU); trang web không được tương tác "một thời gian" có thể bị xóa. Giới hạn 7 ngày cho dữ liệu do script ghi của Safari áp cho trang mở trong trình duyệt; ứng dụng đã thêm vào màn hình chính có bộ đếm riêng | webkit.org "Updates to Storage Policy" (10/8/2023); searchengineland.com "What Safari's 7-day cap on script-writeable storage means for PWA developers" (dẫn lời WebKit) | Gọi `persist()` khi lưu hồ sơ đầu tiên; giao diện khuyên cài lên màn hình chính trên iPhone; nhắc sao lưu theo `backup_nag_days`; tài liệu nói rõ "dữ liệu PWA không an toàn bằng bản cài từ cửa hàng" |
| R-PWA-2 | **PWA không đặt lịch thông báo được khi đóng.** API Notification Triggers của Chromium đã ngừng phát triển; Web Push cần máy chủ | developer.chrome.com/docs/web-platform/notification-triggers ("The development of Notification Triggers API … has ended") | PWA chỉ có thẻ nhắc trong ứng dụng + .ics (02 §7.5); nói rõ trong giao diện và hướng dẫn |
| R-CSP-1 | Xem ảnh chân dung bằng `blob:` URL và chạy WebAssembly đều xung đột với CSP hiện tại | CSP 0.1.1 (`img-src 'self'`, `script-src 'self'`); `'wasm-unsafe-eval'` là điều kiện để biên dịch WebAssembly (tài liệu CSP trích MDN) | Ảnh vẽ qua `<canvas>` từ `createImageBitmap`; WebAssembly theo O-7; S2b đã đăng ký trước ngoại lệ duy nhất |
| R-CRY-1 | Crate `argon2` chưa có kiểm toán độc lập công bố; kiểm toán `chacha20poly1305` là của mã năm 2020 và không nói tới XChaCha | 03 §4.2 | O-8 |
| R-DATA-1 | Dữ liệu chỉ trên máy + tắt sao lưu tự động của hệ điều hành → mất máy là mất dữ liệu nếu chưa xuất tệp | 01 §4.2 | Nhắc sao lưu; hướng dẫn; tài khoản/đồng bộ ở v0.3 |
| R-BAK-2 | Mã hóa một lần trong bộ nhớ giới hạn tệp 128 MiB (khoảng 120 ảnh 1 MiB) | 03 §2.3 | Đủ cho v0.2; mã hóa theo luồng ở v0.3 |
| R-CUL-1 | Phong tục giỗ (giỗ hết năm 2 hay 3, tiên thường, cách gọi trên bia, mâm cỗ) khác nhau theo vùng, tôn giáo, gia đình | Phạm vi đã duyệt: "theo phong tục phổ biến, điều chỉnh sau" | Mọi nhãn sửa được; O-2, O-3; nội dung màn hình tưởng niệm do B8 ghi nguồn tham khảo; cho người am hiểu phong tục xem lại trước G4 |
| R-WIN-1 | Thông báo đặt lịch của ứng dụng Windows chưa đóng gói (MSI, không MSIX) cần định danh ứng dụng qua lối tắt Start Menu; chưa kiểm | Chưa có nguồn kiểm chứng trong đợt này | Quyết định ở đợt 3; nếu không làm được → ghi giới hạn, R5(b) Windows CHƯA ĐỦ BẰNG CHỨNG hoặc KHÔNG ĐẠT |
| R-SCOPE-1 | Đây là bản có dữ liệu cá nhân đầu tiên: nhãn quyền riêng tư của cửa hàng (Google Play Data safety, App Store Privacy) phải đổi từ "không thu thập" — dữ liệu chỉ trên máy vẫn được coi là không thu thập nếu không rời máy, nhưng cần rà lại câu chữ | docs/04 v0.1 (Data safety "Không thu thập dữ liệu") | B9 rà trước G4 |
