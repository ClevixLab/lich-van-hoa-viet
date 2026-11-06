# Biên bản cổng G1 — phiên bản 0.2

Ngày: 2026-11-06. Chủ dự án duyệt các tài liệu 01–05 trong thư mục này và các quyết định dưới đây.
Quyết định ghi tại đây **thay thế** cột "mặc định đề xuất" trong 05 nếu hai bên khác nhau.

## Quyết định O-1 → O-9

| Mã | Quyết định của chủ dự án | Ghi chú |
|---|---|---|
| O-1 | Nhắc ngày tiên thường: **tắt mặc định**, người dùng bật theo từng hồ sơ | Như 05 |
| O-2 | Giỗ hết: **năm thứ 2**, cho chọn năm thứ 3 theo từng hồ sơ | Như 05 |
| O-3 | Dưới **60** tuổi âm là "hưởng dương", từ 60 là "hưởng thọ"; sửa được | Như 05 |
| O-4 | Mất tháng nhuận: **luôn giỗ tháng thường** cùng số | Như 05 |
| O-5 | Tệp .ics: **sự kiện cả ngày**, nhắc lúc 7:00 | Như 05 |
| O-6 | Android: **báo thức không chính xác** (trễ tới ~1 giờ), không xin `SCHEDULE_EXACT_ALARM` | Như 05 |
| O-7 | PWA chạy mật mã bằng **thư viện JavaScript**; **CSP giữ nguyên như 0.1.1**, không thêm `'wasm-unsafe-eval'` | **Khác 05** (05 đề xuất WebAssembly). Ngoại lệ S2b không dùng tới |
| O-8 | **Chấp nhận có điều kiện** thư viện Argon2 chưa có kiểm toán độc lập công bố; bù bằng vector chuẩn, kiểm chéo Rust ↔ JavaScript, ghi rõ trong tài liệu phát hành; ghi là rủi ro mở | Như 05 |
| O-9 | Thông báo trên màn hình khóa **không hiện tên người**, chỉ ghi "Sắp đến ngày giỗ" / "Sắp đến sinh nhật"; người dùng có thể bật hiện tên | **Khác 05** (05 đề xuất hiện tên, có tùy chọn ẩn) |

Ghi chú: O-7 và O-9 khác 05 vì bảng trình chủ dự án duyệt ghi các phương án này; quyết định theo đúng bảng đã trình.

## Tiêu chí R1

Chủ dự án chọn **giữ nguyên ngưỡng R1: 0 sai lệch 1900–3000**, không thêm ngoại lệ cho các trường hợp Sóc sát nửa
đêm (đã biết ở 2242, 2962, 2988). Nếu ngày giỗ chạm các giai đoạn đó, R1 được chấm **KHÔNG ĐẠT** và biên bản liệt kê
từng trường hợp. Quyết định được đưa ra **trước** khi chạy R1.

## Kết luận G1

Các tài liệu 01–05 được khóa; tiêu chí R1–R12 khóa như 04 (R10–R12 được giữ). Cho phép bắt đầu đợt 1.

Chủ dự án duyệt: Trương Xuân Khánh — 2026-11-06 (qua trao đổi trực tuyến)
