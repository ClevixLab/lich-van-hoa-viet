# Nhật ký thay đổi — Lịch và Văn hóa Việt

Số phiên bản lấy từ tệp `VERSION` ở gốc kho (nguồn duy nhất cho mọi nền tảng). Bản PWA đọc các dòng "- …"
dưới tiêu đề của phiên bản đang dựng và đưa vào `version.json` làm ghi chú phát hành hiển thị cho người dùng,
nên mỗi dòng nên ngắn, viết bằng tiếng Việt thường ngày.

## [Chưa phát hành — 0.2]

Đang xây dựng. Đặc tả và tiêu chí nghiệm thu: `docs/v0.2/`. Khi phát hành, tiêu đề này đổi thành `## [0.2.0] — <ngày>`
và các dòng dưới đây được viết lại thành ghi chú cho người dùng.

- Hồ sơ người thân đã khuất và người đang sống; ngày giỗ theo âm lịch, sinh nhật, sự kiện riêng của từng người.
- Nhắc trước 1, 3, 5, 10, 15 ngày hoặc tự đặt, cả khi không có mạng; danh sách "Sắp tới" 30 ngày; xuất lịch .ics.
- Màn hình tưởng niệm ngày giỗ: bia, ảnh, ba nén hương và nến; có thể thêm ảnh mâm lễ của chính gia đình.
- Dữ liệu chỉ lưu trên máy; sao lưu ra tệp có mật khẩu.
- (Đợt 1, nội bộ) Lõi tính ngày giỗ, tuổi, lời nhắc và tệp .ics (`memorial/`, `tsport/src/memorial*.ts`), chưa có giao diện.
- (Nội bộ, đợt 2) Kho dữ liệu SQLite và IndexedDB, tệp sao lưu mã hóa đọc được giữa ứng dụng web và bản cài trên máy.
- Bấm "Để sau" ở thanh báo bản mới thì thanh không còn tự hiện lại sau vài giây.
- (Nội bộ, đợt 3) Danh sách sự kiện theo tuần, tháng, quý, năm (lọc người đã mất / đang sống); lớp nối cho giao diện hồ sơ trên Android, iPhone, Mac và Windows.

## [0.1.4] — 2026-11-07

- Lịch tính theo lịch thiên văn JPL DE441 của NASA — chính xác nhất hiện nay; một số ngày sau năm 2077 có thể khác lịch cũ.
- Từ năm 1900 đến năm 2076, ngày âm, Can Chi và tiết khí không thay đổi so với bản trước.
- Trang Giới thiệu ghi rõ nguồn tính lịch và vì sao các năm rất xa kém chắc chắn hơn.
- Bản web: sửa lỗi nút "Cập nhật ngay" có thể mất tiêu điểm bàn phím.

## [0.1.3] — 2026-11-06

- Windows: chữ đẹp và rõ hơn — phông chữ có đủ dấu tiếng Việt được nhúng sẵn, vẽ chữ sắc nét ở mọi tỉ lệ màn hình.
- Windows: bấm "Chi tiết", "Xem thêm" và dải tháng trên tờ lịch đã hoạt động trở lại.
- Windows: thêm logo Clevix ở cuối trang Giới thiệu, giống bản web.
- Mục Phiên bản chỉ ghi số phiên bản (bỏ số bản dựng và ngày dựng) trên mọi nền tảng.

## [0.1.2] — 2026-11-06

- Dòng bản quyền gọn và chuyên nghiệp hơn: © 2026 Clevix Lab · Phiên bản 0.1.2; áp dụng cho bản web và ứng dụng.

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
