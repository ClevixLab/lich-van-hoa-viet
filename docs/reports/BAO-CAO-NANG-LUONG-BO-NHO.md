# Báo cáo năng lượng và bộ nhớ — Lịch và Văn hóa Việt v0.1

Ngày lập: 2026-11-06. Người lập: A11 (phát hành). Trạng thái mã: thư mục làm việc lúc lập báo cáo (kho
chưa có commit); `pwa/dist/` dựng lại trong đợt này (`app-GDOOI3PQ.js`, cache `lvhv-v0.1-63a7050c50`).

**Quy ước nhãn số liệu**

| Nhãn | Nghĩa |
|---|---|
| **ĐO** | Đo thật trong môi trường dựng: máy ảo Linux x86_64, **2 vCPU** (Intel Xeon 2,10 GHz), **7 GB RAM** (hệ điều hành báo 7,8 GiB), Chromium 153.0.8010.12 headless, Node v22.23.4, rustc 1.97.0. Không có GPU thật, không có pin. |
| **ĐO trên Linux** | Đo thật nhưng trên Linux/Xvfb, **không phải** hệ điều hành đích. Cần đo lại trên máy đích. |
| **ƯỚC TÍNH, CHƯA ĐO** | Suy luận, chưa có phép đo nào trong dự án. Không dùng làm bằng chứng nghiệm thu. |

Không có tiêu chí khóa nào trong `docs/02-TIEU-CHI-NGHIEM-THU.md` về năng lượng. Tiêu chí khóa có liên
quan duy nhất là **P4** (heap JS < 30 MB). Các ngưỡng cho ứng dụng gốc ở §4 (Android < 80 MB, iOS < 60 MB,
khởi động nguội < 1 s, 0 lần đánh thức nền) là **ngưỡng do bên điều phối đặt cho đợt phát hành**, không
nằm trong docs/02. Báo cáo ghi chúng như ngưỡng kiểm tra, không coi là tiêu chí nghiệm thu đã khóa.

Mọi công cụ đo nằm ở `release/tools/` và chạy lại được. Kết quả thô nằm cùng thư mục:
`do-nang-luong-pwa.mjs` → `ket-qua-nang-luong-pwa.json`; `doi-chung-giam-toc.mjs` →
`ket-qua-doi-chung-giam-toc.jsonl`; `maxrss.c` → `ket-qua-rss-loi.jsonl`; `lvhv-dump bench` →
`ket-qua-bench-loi.txt`; `do-windows-xvfb.sh` → `ket-qua-windows-xvfb.jsonl`.

## Tóm tắt

| Hạng mục | Kết quả | Nhãn |
|---|---|---|
| PWA — heap JS khi xem tờ lịch (sau thu gom rác) | **2,30 MB** (sau mọi thao tác: 2,80 MB) | ĐO |
| PWA — tiến trình renderer, RSS / PSS khi xem tờ lịch | **127 MB / 82,6 MB** (trang trống đối chứng: 77,9 MB / 43,3 MB → phần do ứng dụng ≈ **+49 MB RSS / +39 MB PSS**) | ĐO |
| PWA — CPU khi để yên trên tờ lịch 60 s | Luồng chính bận **9,9 ms** trong 60 s; thời gian chạy JS **0 ms**; layout **0**; style **0**. CPU mọi tiến trình trình duyệt: **50 ms / 60 s** (≈ 0,08 % một nhân) | ĐO |
| PWA — mỗi lần vuốt sang ngày (100 lần) | Luồng chính **11,8 ms**/lần (JS 1,9 ms); thời gian vẽ tờ lịch trung vị 2,4 ms, p95 4,2 ms; CPU mọi tiến trình ≈ **60 ms**/lần | ĐO |
| PWA — một lần chọn ngày tốt 90 ngày | Luồng chính **26 ms** (JS 7,6 ms); CPU mọi tiến trình ≈ 50 ms | ĐO |
| PWA — giảm tốc CPU 4× (giả lập điện thoại tầm trung) | Vuốt: 50,5 ms/lần luồng chính, vẽ trung vị 10,2 ms, p95 15,1 ms, lớn nhất 27,6 ms; chọn ngày 90 ngày: 130 ms; nghỉ 60 s: luồng chính 31 ms, JS 0 | ĐO (giả lập) |
| PWA — mạng sau khi cài | **0 yêu cầu ra ngoài nguồn gốc**; dữ liệu ứng dụng: 0 byte. Còn **1 yêu cầu 1 141 byte** (`/sw.js`) do trình duyệt tự kiểm tra bản cập nhật service worker khi mở lại trang (xem §1.5) | ĐO |
| PWA — dung lượng lưu trên máy | Cache Storage **13 tệp, 573 549 byte** (≈ 560 KB); trình duyệt báo tổng dùng 934 291 byte (≈ 0,9 MB); `localStorage` trống cho đến khi người dùng đổi cỡ chữ/giao diện | ĐO |
| Lõi Rust — `day_info` + JSON | 25,9–29,9 µs/ngày (Hnd), 36,3–45,3 µs (Precise) | ĐO |
| Lõi Rust — `find_good_days` 90 ngày | 1,98–3,14 ms (Hnd), 2,86–3,47 ms (Precise) | ĐO |
| Lõi Rust — RSS đỉnh khi tính `day_info` cả năm 2026 | **2,6–2,8 MB** (đối chứng `/bin/true`: 1,3 MB); tính cả 2000–3000 cũng chỉ 2,7–2,8 MB | ĐO |
| Windows (Slint) — RSS, renderer phần mềm | **51,0 MB** (PSS 46,9 MB); nghỉ 60 s: CPU **≤ 10 ms** (giới hạn phân giải 10 ms) | ĐO trên Linux, cần đo lại trên Windows |
| Windows (Slint) — RSS, renderer femtovg/OpenGL (llvmpipe) | **161,5 MB**; nghỉ 60 s: CPU 0 ms | ĐO trên Linux, cần đo lại trên Windows |
| Android — bộ nhớ | 35–60 MB PSS | **ƯỚC TÍNH, CHƯA ĐO** |
| iOS — bộ nhớ | 20–40 MB (thước bộ nhớ của Xcode) | **ƯỚC TÍNH, CHƯA ĐO** |
| macOS — bộ nhớ | 40–80 MB | **ƯỚC TÍNH, CHƯA ĐO** |

## 1. PWA trong Chromium headless — ĐO

### 1.1 Cách đo

`release/tools/do-nang-luong-pwa.mjs` phục vụ `pwa/dist/` trên `127.0.0.1` (nén gzip mức 9, giống
`pwa/test/smoke.mjs`). Nó mở Chromium headless với giả lập điện thoại **360×780, mật độ điểm ảnh 2, cảm ứng**,
rồi chạy cùng một kịch bản hai lần: **không giảm tốc CPU** và **giảm tốc CPU 4×**
(`Emulation.setCPUThrottlingRate`). Các bước của kịch bản:

1. Mở `#/ngay/2026-11-05`, chờ service worker sẵn sàng, tải lại trang để service worker điều khiển.
2. Thu gom rác (`HeapProfiler.collectGarbage`), đọc `Runtime.getHeapUsage`. Đọc
   `SystemInfo.getProcessInfo` để lấy PID từng tiến trình, rồi đọc `/proc/<pid>/status` (VmRSS) và
   `/proc/<pid>/smaps_rollup` (PSS, USS).
3. Để yên 60 s. Đo hiệu `Performance.getMetrics` (TaskDuration, ScriptDuration, LayoutDuration,
   RecalcStyleDuration; đồng hồ `threadTicks`) và hiệu thời gian CPU của từng tiến trình
   (`SystemInfo.getProcessInfo.cpuTime`).
4. Vuốt 100 lần bằng sự kiện chạm thật (`Input.dispatchTouchEvent` start/move/end, dịch 110 px), cách
   nhau 300 ms để hoạt ảnh lật trang 0,26 s chạy hết. Ghi thời gian vẽ do ứng dụng tự đo (`__lvhv.renderMs`).
5. Mục Chọn ngày tốt → Cưới hỏi → "3 tháng tới" (90 ngày, không tuổi).
6. Đếm byte và yêu cầu tới máy chủ kể từ sau khi cài (bước 1).

Đối chứng (`doi-chung-giam-toc.mjs`): cùng trình duyệt, trang `about:blank`, cùng giả lập màn hình.

### 1.2 Bộ nhớ (không giảm tốc; số ở chế độ giảm tốc gần như trùng)

| Đại lượng | Ứng dụng (tờ lịch 2026-11-05) | Sau mọi thao tác | Trang trống (đối chứng) |
|---|---|---|---|
| Heap JS đang dùng | **2,30 MB** (tổng cấp 3,12 MB) | 2,80 MB | — |
| Renderer RSS | 130 036 KB (127,0 MB) | 144 204 KB (140,8 MB) | 79 780 KB (77,9 MB) |
| Renderer PSS | 84 599 KB (82,6 MB) | 97 214 KB (94,9 MB) | 44 376 KB (43,3 MB) |
| Renderer USS (bộ nhớ riêng) | 60 060 KB (58,7 MB) | — | — |
| GPU RSS / PSS | 63 028 / 26 486 KB | 63 120 / 26 249 KB | 55 924 / 23 937 KB |
| Tiến trình trình duyệt RSS / PSS | 100 436 / 47 691 KB | 101 392 / 47 691 KB | 89 528 / 42 280 KB |
| Dịch vụ mạng RSS / PSS | 71 604 / 26 989 KB | 71 436 / 26 577 KB | 63 912 / 23 599 KB |

Cách đọc:
- **P4** (heap JS < 30 MB): 2,30 MB, xem biên bản cổng G3.
- RSS cộng trùng các thư viện dùng chung giữa các tiến trình Chromium. PSS chia đều phần dùng chung nên
  gần với chi phí thật hơn. Phần ứng dụng thêm vào renderer so với trang trống là **≈ 49 MB RSS, ≈ 39 MB
  PSS**. Phần này gồm cả DOM, phông chữ đã giải mã, ảnh tờ lịch đã vẽ ở mật độ 2× và mã JS (lõi tsport,
  bảng Ngọc Hạp, 427 câu ca dao).
- Các tiến trình trình duyệt, GPU và dịch vụ mạng là của Chromium. Trên điện thoại, chúng được dùng chung với
  mọi tab và ứng dụng web khác, nên không tính cho ứng dụng. Chromium headless trên Linux khác Chrome
  Android về cách chia tiến trình và giới hạn bộ nhớ, nên số tuyệt đối trên điện thoại sẽ khác.

### 1.3 CPU — để yên trên tờ lịch 60 s

| Đại lượng | Không giảm tốc | Giảm tốc 4× |
|---|---|---|
| Luồng chính: tổng thời gian bận (TaskDuration) | 9,9 ms | 31,2 ms |
| — trong đó chạy JS (ScriptDuration) | **0 ms** | **0 ms** |
| — layout / tính lại style | 0 / 0 lần | 0 / 0 lần |
| Yêu cầu tài nguyên mới | 0 | 0 |
| CPU mọi tiến trình Chromium (60 s) | 50 ms (trình duyệt 20, renderer 20, GPU 10) | không dùng được — xem dưới |

Kết luận đo được: khi để yên, ứng dụng **không chạy mã JS nào**. Không có vòng lặp `setInterval` hay
`requestAnimationFrame`, và không có layout lại. 9,9 ms luồng chính là tác vụ nội bộ của trình duyệt, không phải JS của ứng dụng. Báo cáo không đo
đại lượng này trên trang trống, nên không so sánh. Đọc mã cho cùng kết luận. Ứng dụng chỉ có **một**
`setTimeout` tới 00:00:05 để đổi sang ngày mới (`pwa/src/main.ts`, `armMidnight`), và bộ hẹn giờ này bị hủy
khi trang bị ẩn (`visibilitychange`). Service worker không đăng ký đồng bộ định kỳ (periodic sync) hay
thông báo đẩy (push).

**Giới hạn của phép đo giảm tốc.** Với trang trống `about:blank` nghỉ 20 s, renderer tốn **0 ms** CPU khi
không giảm tốc, nhưng tốn **16 430 ms** khi bật giảm tốc 4× (tệp `ket-qua-doi-chung-giam-toc.jsonl`; một lần chạy trước cho 16 730 ms nhưng kết quả đó không được lưu). Vậy chính cơ chế giả lập
giảm tốc của Chromium ngốn CPU, ở mức khoảng 0,8 giây CPU cho mỗi giây thực. Vì thế, ở chế độ giảm tốc,
báo cáo chỉ dùng thời gian luồng chính (`Performance.getMetrics`). Không dùng CPU tiến trình (đo được
50 210 ms trong 60 s, gần như toàn bộ là chi phí giả lập).

### 1.4 CPU — thao tác

| Thao tác | Đại lượng | Không giảm tốc | Giảm tốc 4× |
|---|---|---|---|
| Vuốt sang ngày (100 lần) | Luồng chính / lần | 11,8 ms | 50,5 ms |
| | JS / lần | 1,87 ms | 9,25 ms |
| | Vẽ tờ lịch (ứng dụng tự đo): trung vị / p95 / lớn nhất | 2,4 / 4,2 / 5,0 ms | 10,2 / 15,1 / 27,6 ms |
| | CPU mọi tiến trình / lần | 59,9 ms (renderer 29,0; GPU 26,5; trình duyệt 4,2) | — (xem §1.3) |
| | Ngày sau 100 lần vuốt | 2027-02-13 (đúng 100 ngày) | 2027-02-13 |
| Chọn ngày tốt, cưới hỏi, 90 ngày | Luồng chính | 26,4 ms (JS 7,6 ms) | 129,9 ms (JS 34,2 ms) |
| | CPU mọi tiến trình | 50 ms | — |
| | Kết quả hiển thị | 15 ngày (trang đầu; danh sách đầy đủ: nút "Xem thêm") | 15 |
| Mở lần đầu tới khi thấy tờ lịch (máy chủ cục bộ) | Thời gian thực | 240 ms | 430 ms |

Phần GPU (26,5 ms mỗi lần vuốt) là vẽ bằng phần mềm (SwiftShader) vì máy đo không có GPU. Trên điện
thoại có GPU thật, phần này thường nhỏ hơn, nhưng dự án chưa đo điều đó. Ở chế độ giảm tốc 4×, lần vẽ chậm
nhất là 27,6 ms, tức có thể rơi 1 khung hình (60 Hz = 16,7 ms/khung). Trung vị 10,2 ms nằm trong một khung.

### 1.5 Mạng và lưu trữ

| Đại lượng | Kết quả |
|---|---|
| Lần cài đầu (tải trang + service worker tải trước) | 20 yêu cầu, **321 463 byte** gzip (313,9 KB), trùng `pwa/test/metrics.json` (P3) |
| Sau khi cài: yêu cầu tới origin khác | **0** (cả hai cấu hình; trùng kết quả S3 của `smoke.mjs`) |
| Sau khi cài: dữ liệu ứng dụng tải qua mạng | **0 byte**: mọi tệp lấy từ Cache Storage |
| Sau khi cài: yêu cầu còn lại | **1 yêu cầu `/sw.js`, 1 141 byte**, xảy ra khi tải lại trang |
| Cache Storage | 13 mục, 573 549 byte (chưa nén) |
| `navigator.storage.estimate().usage` | 934 291 byte |
| `localStorage` | trống; chỉ có khóa `lvhv.ui` sau khi người dùng đổi cỡ chữ hoặc giao diện (S4) |

**Về "0 byte mạng sau khi cài":** điều này **không đúng tuyệt đối**. Mỗi khi trang được mở hoặc tải lại
lúc có mạng, trình duyệt tự tải lại `sw.js` (khoảng 1 KB) để xem có bản mới không. Đây là cơ chế chuẩn
của service worker, được thực hiện bởi trình duyệt chứ không phải mã của ứng dụng, và chỉ gọi tới chính nguồn gốc ứng dụng.
Khi không có mạng, yêu cầu này thất bại lặng lẽ và ứng dụng vẫn chạy (P5). Nhờ cơ chế này mà người dùng
nhận được bản sửa lỗi. Không có yêu cầu nào khác, không có phân tích, quảng cáo hay phông chữ từ máy chủ
ngoài.

## 2. Lõi Rust (bản release, LTO) — ĐO

`target/release/lvhv-dump bench`, chạy 3 lần cho mỗi phương pháp, xen kẽ (`release/tools/ket-qua-bench-loi.txt`):

| Phép đo | Hnd (3 lần: trung bình) | Precise (3 lần: trung bình) | Ngưỡng khóa |
|---|---|---|---|
| `day_info` + `to_json`, 365 ngày 2026 | 29,90 / 25,88 / 26,17 µs (p99 ≤ 67 µs) | 36,25 / 45,30 / 37,31 µs (p99 ≤ 103 µs) | P1 < 1 ms |
| `find_good_days` 90 ngày, 16 việc × (không tuổi, 1976) | 3,06 / 3,14 / 1,98 ms (lớn nhất 7,28 ms) | 2,98 / 3,47 / 2,86 ms (lớn nhất 4,67 ms) | P2 < 50 ms |
| `solar_to_lunar`, 365 608 ngày 2000–3000 | 3,86 / 4,03 / 3,93 µs | 20,53 / 21,47 / 19,08 µs | — |

Số `find_good_days` lần này **cao hơn** số ghi ở `docs/05` §E (739–1 923 µs). Sandbox 2 vCPU dùng chung
nên nhiễu lớn, và lần đo này chưa được tách nguyên nhân. Dù vậy, cả lần chậm nhất (7,28 ms) vẫn thấp hơn
ngưỡng P2 khoảng 7 lần.

**RSS đỉnh** (`release/tools/maxrss.c`: tiến trình C nhỏ `fork` + `execvp` + `wait4`, đọc `ru_maxrss` của
tiến trình con; sandbox không có `/usr/bin/time`). Mỗi lệnh chạy 3 lần (`release/tools/ket-qua-rss-loi.jsonl`):

| Lệnh | RSS đỉnh | Thời gian CPU |
|---|---|---|
| `/bin/true` (đối chứng: chi phí nạp tiến trình) | 1 252–1 380 KB | ~1 ms |
| `lvhv-dump --method hnd dayinfo-jsonl 2026-01-01 2026-12-31` (365 ngày, ra JSON) | **2 640–2 780 KB** | 11–13 ms |
| `lvhv-dump --method precise dayinfo-jsonl 2026-01-01 2026-12-31` | 2 736–2 784 KB | 15–17 ms |
| `lvhv-dump gooddays cuoi_hoi 2026-11-01 2027-01-29 none` | 2 624–2 976 KB | ~4 ms |
| `lvhv-dump --method hnd dayinfo-jsonl 2000-01-01 3000-12-31` (365 608 ngày) | 2 712–2 784 KB | 10,2–10,8 s |

Lõi dùng bộ nhớ **không đổi** theo số ngày: tính 1 năm hay 1 000 năm đều có RSS đỉnh khoảng 2,7 MB (gồm
mã chương trình, khoảng 1,3 MB là chi phí nạp tiến trình). Lõi không có bảng tra lớn và không có bộ nhớ
đệm toàn cục (docs/03 §0). Thư viện FFI dựng cho Linux x86_64 (`liblvhv_ffi.so`) nặng 1 264 080 byte.

## 3. Ứng dụng Windows (Slint 1.8.0) — ĐO trên Linux, cần đo lại trên Windows

Dựng lại bản release trong đợt này trên Linux (`apps/windows`, `cargo build --release`, 7 phút 26 giây,
tệp `LichViet` 12 182 480 byte). Đo dưới Xvfb 1280×900×24 bằng `release/tools/do-windows-xvfb.sh`: khởi
động, chờ 8 s, đọc `/proc`, để yên 60 s, đọc lại CPU (`utime + stime`, đơn vị tick 10 ms).

| Renderer (`SLINT_BACKEND`) | RSS sau 8 s | PSS | RSS sau 60 s nghỉ | Luồng | CPU khi nghỉ 60 s |
|---|---|---|---|---|---|
| `winit-software` | **52 208 KB (51,0 MB)** | 48 062 KB | 52 208 KB | 4 | **10 ms** (= 1 tick, giới hạn phân giải) |
| `winit-femtovg` (OpenGL qua llvmpipe, Mesa phần mềm) | 165 424 KB (161,5 MB) | 143 143 KB | 165 424 KB | 9 | 0 ms |

Số đo khớp với `docs/04` §8 (~52 MB / ~170 MB). Khi để yên, ứng dụng không tiêu CPU đáng kể và bộ nhớ
không tăng. **Chưa đo trên Windows.** Trên Windows, renderer femtovg dùng driver OpenGL của GPU thật, không
qua llvmpipe, nên bộ nhớ có thể khác nhiều theo cả hai chiều. Workflow `windows.yml` in Working Set vào
`mem.txt` sau 8 s.

## 4. Android, iOS, macOS — ƯỚC TÍNH, CHƯA ĐO

Ba nền tảng này **chưa từng được biên dịch** (docs/04 §8). Mọi số dưới đây là **ƯỚC TÍNH, CHƯA ĐO** và
không phải bằng chứng.

| Nền tảng | Ước tính bộ nhớ | Cách suy ra |
|---|---|---|
| Android (Compose, minSdk 26) | **35–60 MB PSS** (`dumpsys meminfo` TOTAL PSS) | Lõi Rust: khoảng 1,3 MB mã (`.so`) và dưới 2 MB bộ nhớ làm việc (§2, đo trên Linux). JNA thêm vài MB. Phần lớn còn lại là nền của một ứng dụng Jetpack Compose một Activity (ART runtime, Compose, Material3, phông chữ đã nạp), theo kinh nghiệm thông thường cỡ vài chục MB. Bảng ca dao và Ngọc Hạp nằm trong `.so`. |
| iOS (SwiftUI, iOS 16+) | **20–40 MB** ("Memory" trong Xcode Debug Navigator) | Lõi tĩnh liên kết vào ứng dụng (khoảng 2 MB như trên) cộng nền của một ứng dụng SwiftUI nhỏ và 3 tệp phông TTF. |
| macOS (SwiftUI, macOS 13+) | **40–80 MB** (Activity Monitor, cột Memory) | Nền ứng dụng AppKit/SwiftUI trên macOS thường lớn hơn iOS vì có cửa sổ, thanh menu và bộ nhớ đệm vẽ ở độ phân giải Retina. |
| Khởi động nguội | dưới 1 s (cả ba nền tảng) | Lõi tính một ngày trong khoảng 30 µs và không đọc tệp hay mạng khi khởi động. Thời gian chủ yếu là của hệ điều hành và framework giao diện. |
| Đánh thức nền | **0** | Theo **đọc mã**: Android không xin quyền nào, không có WorkManager/AlarmManager/JobScheduler/Service/BroadcastReceiver; cập nhật "hôm nay" chỉ khi `ON_RESUME` (`MainActivity.kt`). iOS/macOS không có background mode trong `project.yml`; dùng `scenePhase` (`RootView.swift`). Windows không có hẹn giờ nền. Chưa kiểm bằng công cụ trên thiết bị. |

### 4.1 Quy trình đo trên thiết bị của chủ dự án

Ngưỡng đạt cho đợt này (do bên điều phối đặt, xem đầu báo cáo): **Android < 80 MB**, **iOS < 60 MB**,
**khởi động nguội < 1 s**, **0 lần đánh thức nền**. Ghi kết quả theo mẫu ở §4.2.

**Android** (máy thật, tốt nhất là máy tầm trung; cài APK từ artifact `android-release` của CI):
1. Bật Tùy chọn nhà phát triển và Gỡ lỗi USB. Cắm máy vào máy tính đã cài Android Studio (có `adb`).
2. Khởi động nguội: `adb shell am force-stop vn.clevix.lichviet`, rồi
   `adb shell am start -W -n vn.clevix.lichviet/.MainActivity`. Ghi `TotalTime` (ms). Lặp 5 lần, lấy trung vị.
3. Bộ nhớ: mở tờ lịch, vuốt 20 ngày, chờ 10 s, rồi chạy `adb shell dumpsys meminfo vn.clevix.lichviet`.
   Ghi **TOTAL PSS** và **TOTAL RSS**. Có thể xem thêm trong Android Studio → Profiler → Memory.
4. CPU khi để yên: để ứng dụng trên tờ lịch 60 s. Trong Android Studio → Profiler → CPU, CPU của ứng dụng
   phải gần 0 %.
5. Đánh thức nền và pin: `adb shell dumpsys batterystats --reset`, mở ứng dụng 1 phút, đưa về nền,
   rút cáp, để 2 giờ. Cắm lại, chạy `adb bugreport bugreport.zip`, rồi mở tệp trong Battery Historian
   (https://github.com/google/battery-historian) hoặc chạy
   `adb shell dumpsys batterystats vn.clevix.lichviet`. Kiểm tra: **0** wakelock, **0** alarm, **0** job,
   0 byte mạng cho `vn.clevix.lichviet`.
6. Đạt nếu: TOTAL PSS < 80 MB, `TotalTime` < 1 000 ms, mọi mục ở bước 5 bằng 0.

**iOS** (iPhone thật, bản dựng từ Xcode có Team của Clevix LLC):
1. Xcode → Product → Profile (⌘I). Dùng mẫu **App Launch** để đo khởi động nguội, chạy 5 lần, lấy trung
   vị thời gian tới khung hình đầu.
2. Bộ nhớ: chạy bằng Xcode (⌘R), mở Debug Navigator → Memory, vuốt 20 ngày, ghi số ổn định. Có thể dùng
   thêm Instruments → **Allocations** / **VM Tracker**.
3. Năng lượng: Instruments → **Energy Log** (trên thiết bị: Cài đặt → Nhà phát triển → Logging → Energy,
   bấm Start, dùng ứng dụng 5 phút rồi để nền 1 giờ, bấm Stop, rồi nhập vào Instruments). Mức "Energy
   Impact" khi để yên phải là Low/0. Khi ở nền không được có hoạt động CPU hay mạng.
4. Đạt nếu: bộ nhớ < 60 MB, khởi động < 1 s, 0 hoạt động nền.

**macOS:** Activity Monitor → tab Memory (cột Memory) và tab Energy (cột "Energy Impact", "Preventing
Sleep" = No). Đo khởi động bằng Instruments → App Launch.

**Windows** (sau khi dựng bằng `Build-Windows.ps1`):
1. Mở `LichViet.exe`, chờ 10 s. Trong Task Manager → Details, bật cột "Memory (private working set)"
   và "CPU time". Ghi bộ nhớ, để yên 60 s, kiểm tra CPU time không tăng.
2. Năng lượng: chạy PowerShell **với quyền quản trị**: `powercfg /srumutil /output $env:USERPROFILE\Desktop\srum.csv`
   (dữ liệu SRUM cập nhật khoảng mỗi giờ; dùng ứng dụng ít nhất 1 giờ trước khi xuất). Lọc dòng
   `LichViet.exe` và xem các cột năng lượng/CPU. Có thể dùng thêm `powercfg /energy` (60 s) để xem có
   bộ hẹn giờ độ phân giải cao hay không.
3. So với số đo trên Linux ở §3 (51 MB với renderer phần mềm).

### 4.2 Mẫu ghi kết quả

| Nền tảng | Máy (model, hệ điều hành) | Bộ nhớ (MB) | Khởi động nguội (ms) | CPU nghỉ 60 s | Đánh thức nền | Kết luận |
|---|---|---|---|---|---|---|
| Android | | | | | | ĐẠT / KHÔNG ĐẠT |
| iOS | | | | | | ĐẠT / KHÔNG ĐẠT |
| macOS | | | | | | ĐẠT / KHÔNG ĐẠT |
| Windows | | | | | | ĐẠT / KHÔNG ĐẠT |

## 5. Kết luận cho người dùng thông thường — "có tiện lợi khi dùng không?"

Những điều dưới đây có phép đo hoặc đọc mã làm căn cứ, ghi kèm trong ngoặc. Những gì chưa đo được thì
ghi rõ là chưa đo.

- **Nhẹ khi tải.** Bản web tải về một lần khoảng 314 KB (đo). Sau đó mọi thứ nằm sẵn trên máy, chiếm
  khoảng 1 MB bộ nhớ lưu trữ (đo). Để so sánh: theo các thống kê công khai về dung lượng trang web (ví dụ HTTP Archive), một trang tin
  tức hay mạng xã hội trên điện thoại thường nặng cỡ vài MB **cho mỗi lần mở**. Đây là số tham khảo bên
  ngoài, dự án không đo và không kiểm lại trong đợt này.
- **Không cần mạng.** Sau lần mở đầu, ứng dụng chạy cả khi tắt mạng (đo, P5). Ứng dụng không gửi dữ liệu
  đi đâu, không có quảng cáo, không có theo dõi (đo, S3). Khi có mạng, trình duyệt chỉ hỏi máy chủ của
  ứng dụng "có bản mới không" bằng khoảng 1 KB mỗi lần mở (đo).
- **Không chạy ngầm.** Khi để yên trên tờ lịch, ứng dụng không chạy chút mã nào (đo, 0 ms trong 60 s).
  Khi đóng hoặc chuyển sang ứng dụng khác, ứng dụng không làm gì cả (đọc mã: không có tác vụ nền, thông báo hay
  đồng bộ). Vì vậy ứng dụng gần như không tốn pin khi không nhìn vào nó. Với ứng dụng gốc trên điện thoại,
  điều này mới được kiểm bằng đọc mã, chưa đo trên máy thật.
- **Nhanh khi dùng.** Mỗi lần vuốt sang ngày mất khoảng 12 ms xử lý (đo). Trên máy được giả lập chậm hơn 4
  lần, mức này là khoảng 50 ms (đo, giả lập), vẫn nhanh hơn cái chớp mắt. Tìm ngày tốt trong 3 tháng mất
  khoảng 26 ms (đo), hoặc 130 ms khi giả lập chậm 4 lần.
- **Bộ nhớ vừa phải.** Phần ứng dụng thêm vào trình duyệt khoảng 40–50 MB so với một trang trống (đo trên
  Chromium máy tính). Bản Windows dùng khoảng 51 MB (đo trên Linux). Bản Android/iPhone **chưa đo**; ước
  tính 20–60 MB.
- **Điều gì làm tốn pin?** Khi dùng một ứng dụng lịch như thế này, phần tốn pin chủ yếu là **màn hình
  sáng** trong lúc bạn xem. Đây là nguyên lý chung của điện thoại, không phải số đo của dự án. Phần xử lý của ứng
  dụng rất nhỏ (đo: khoảng 60 ms CPU cho mỗi lần vuốt trên máy đo). Xem lịch vài phút mỗi ngày thì mức tốn
  pin chủ yếu phụ thuộc vào độ sáng màn hình và thời gian bạn nhìn. Chế độ giao diện tối có thể giúp tiết kiệm trên màn
  hình OLED (nguyên lý chung, dự án chưa đo).
- **Chưa biết chắc:** con số pin thực tế (mAh, % pin) trên điện thoại thật. Sandbox không có pin; cần
  đo theo §4.1.

## 6. Giới hạn của báo cáo

- Mọi phép đo PWA chạy trên Chromium headless, máy chủ, không GPU, nên không phải Chrome Android hay Safari iOS.
  Safari (iPhone) chưa được đo.
- Giảm tốc CPU 4× chỉ là giả lập luồng chính, không giả lập GPU, bộ nhớ hay nhiệt. Nó cũng không giả lập cơ chế điều chỉnh xung nhịp
  của điện thoại.
- Mỗi phép đo PWA chạy 1 lần cho mỗi cấu hình (lần chạy 5 s nghỉ trước đó cho số cùng bậc). Phép đo lõi
  chạy 3 lần. Sandbox dùng chung nên nhiễu lớn.
- Phép đo năng lượng (J, mAh) không thực hiện được ở đây. "CPU-ms" chỉ là đại lượng thay thế.
