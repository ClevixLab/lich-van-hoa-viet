# Nhật ký thay đổi — Lịch và Văn hóa Việt

Số phiên bản lấy từ tệp `VERSION` ở gốc kho (nguồn duy nhất cho mọi nền tảng). Bản PWA đọc các dòng "- …"
dưới tiêu đề của phiên bản đang dựng và đưa vào `version.json` làm ghi chú phát hành hiển thị cho người dùng,
nên mỗi dòng nên ngắn, viết bằng tiếng Việt thường ngày.

## [0.1.1] — 2026-11-06

- Mở rộng phạm vi lịch: từ năm 1900 đến năm 3000.
- Âm lịch trước năm 1968 tính theo quy ước lịch sử: giờ địa phương Bắc Kinh trước 1929, múi giờ UTC+8 từ 1929 đến 1967, UTC+7 (giờ Hà Nội) từ 1968.
- Màn hình Đổi ngày mới: ba ô Ngày / Tháng / Năm, nút đổi lớn, thông báo rõ khi nhập sai.
- Ngày âm lịch mặc định tính theo mô hình thiên văn chính xác (Meeus); thuật toán Hồ Ngọc Đức vẫn giữ để đối chiếu.
- Thêm logo Clevix ở chân trang và trang Giới thiệu.
- Hiện số phiên bản đang dùng, nút Kiểm tra cập nhật và lời nhắc khi có phiên bản mới.

## [0.1.0] — 2026-11-06

- Bản thử nghiệm nội bộ đầu tiên (ứng dụng web cài được, chạy cả khi không có mạng).
- Tờ lịch hằng ngày: ngày âm, Can Chi, tiết khí, giờ hoàng đạo, Trực, sao tốt và sao xấu theo Ngọc Hạp Thông Thư.
- Chọn ngày tốt theo việc và theo tuổi; đổi ngày âm ↔ dương (năm 2000 đến năm 3000).
- Mỗi ngày một câu ca dao, tục ngữ có ghi nguồn.
