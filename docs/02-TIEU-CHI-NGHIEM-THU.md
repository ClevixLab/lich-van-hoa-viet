# Tiêu chí nghiệm thu — v0.1 (đăng ký trước, KHÓA)

Khóa lúc: 2026-11-06, trước khi viết mã lõi. Không được nới lỏng sau khi thấy kết quả.
Mỗi tiêu chí kết luận một trong: ĐẠT / KHÔNG ĐẠT / CHƯA ĐỦ BẰNG CHỨNG.

## Lịch (G2)

| Mã | Tiêu chí | Ngưỡng |
|---|---|---|
| L1 | Ngày Tết Nguyên đán 2000–2100 của core so với danh sách công bố (nguồn ghi trong báo cáo) | 0 sai lệch |
| L2 | Ngày âm (năm, tháng, ngày, nhuận) của core so với oracle độc lập, mọi ngày 2000-01-01 → 2100-12-31 | 0 sai lệch |
| L3 | Như L2 cho 2101-01-01 → 3000-12-31 | Không có ngưỡng ĐẠT; MỌI sai lệch phải được liệt kê và phân loại (Sóc/Trung khí cách nửa đêm UTC+7 dưới 30 phút, hoặc nguyên nhân khác). Sai lệch "nguyên nhân khác" = KHÔNG ĐẠT |
| L4 | Can Chi ngày/tháng/năm của core so với oracle, mọi ngày 2000–3000 | 0 sai lệch |
| L5 | Khứ hồi dương → âm → dương, mọi ngày 2000–3000 | 0 lỗi |
| L6 | Tiết khí của core so với oracle, 2000–2100 | 0 sai lệch về ngày bắt đầu |
| L7 | Bản TypeScript (tsport) so với core: toàn bộ JSON `day_info`, mọi ngày 2000–3000 | Giống hệt từng byte |
| L8 | Đối chiếu ngoài: ngày 2026-11-05 khớp các trường quan sát được trên một trang lịch công khai (âm 27/9 Bính Ngọ, ngày Quý Mùi, tháng Mậu Tuất, Trực Thu, Chu Tước Hắc Đạo, Nạp âm Dương Liễu Mộc, tiết Sương Giáng, sao Tỉnh, giờ Hoàng đạo Dần-Mão-Tỵ-Thân-Tuất-Hợi) | Khớp mọi trường; lệch trường nào phải giải thích nguồn quy tắc |

## Ngọc Hạp và nội dung (G2/G4)

| Mã | Tiêu chí | Ngưỡng |
|---|---|---|
| N1 | Mỗi quy tắc sao tốt/xấu có ít nhất 1 nguồn dẫn kiểm chứng được | 100% |
| N2 | Quy tắc mà các nguồn mâu thuẫn được liệt kê, ghi phương án đã chọn | 100% |
| N3 | Chọn ngày: ngày phạm sao kỵ chính của việc đã chọn không xuất hiện trong kết quả | 0 vi phạm trên toàn bộ 2000–3000 |
| C1 | Mỗi câu ca dao/tục ngữ có ghi nguồn; không có câu của tác giả cá nhân | 100% |

## Hiệu năng (G3) — đo trong môi trường build, ghi rõ máy đo

| Mã | Tiêu chí | Ngưỡng |
|---|---|---|
| P1 | `day_info` (bản dựng release, native) | < 1 ms mỗi ngày |
| P2 | `find_good_days` 90 ngày (native) | < 50 ms |
| P3 | PWA: dung lượng tải lần đầu (nén gzip) | < 500 KB |
| P4 | PWA: bộ nhớ heap JavaScript khi xem tờ lịch (Chromium) | < 30 MB |
| P5 | PWA: hoạt động khi không có mạng sau lần mở đầu | Có |

## Bảo mật (G4)

| Mã | Tiêu chí | Ngưỡng |
|---|---|---|
| S1 | Không có `unsafe` trong core | 0 |
| S2 | Thư viện phụ thuộc của core | 0 (lõi không phụ thuộc thư viện ngoài) |
| S3 | PWA không gọi mạng ngoài nguồn gốc của chính nó; có Content-Security-Policy chặt | Đạt |
| S4 | PWA không lưu dữ liệu người dùng (v0.1 không có tài khoản) | Đạt |
