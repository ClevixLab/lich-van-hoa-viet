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

## Bổ sung A1 — mở rộng phạm vi 1900–1999 (KHÓA 2026-11-06, trước khi sửa mã)

Quyết định của chủ dự án (06/11/2026): phạm vi đổi ngày mở rộng thành 1900-01-01 → 3000-12-31, theo
**phương án lịch sử chuẩn miền Bắc**: âm lịch tính theo UTC+8 trước khi chuyển đổi, UTC+7 từ khi chuyển đổi
(miền Bắc đổi giờ chuẩn ngày 08/8/1967; Tết Mậu Thân miền Bắc là 1968-01-29). Điểm chuyển cụ thể trong thuật
toán phải được ghi trong docs/03 kèm bằng chứng. App ghi chú: miền Nam dùng UTC+8 đến 1975 nên một số tháng
1968–1975 có thể khác lịch miền Nam thời đó.

| Mã | Tiêu chí | Ngưỡng |
|---|---|---|
| L9 | Ngày Tết 1900–1999 của core so với nguồn Việt Nam đã công bố (mỗi năm ghi nguồn) | 0 sai lệch ở các năm có nguồn; năm không có nguồn → CHƯA ĐỦ BẰNG CHỨNG cho năm đó, không tự điền |
| L10 | Ngày âm (năm, tháng, ngày, nhuận) của core so với oracle độc lập dùng cùng quy tắc múi giờ, mọi ngày 1900-01-01 → 1999-12-31, cả hai phương pháp | 0 sai lệch (cùng quy tắc như L2; mọi sai lệch được liệt kê và phân loại) |
| L11 | Tính liên tục quanh điểm chuyển múi giờ: mọi ngày 1967-01-01 → 1969-12-31 có đúng một ngày âm; ngày âm tăng đều trong tháng, không thiếu không trùng; Tết 1968 = 1968-01-29 | Đạt cả ba |
| L12 | L5 (khứ hồi) và L7 (TypeScript trùng từng byte với Rust) mở rộng cho 1900-01-01 → 3000-12-31, cả hai phương pháp | 0 lỗi, 0 sai lệch |
| L13 | Kết quả 2000–3000 không đổi so với trước khi mở rộng (so với tệp chuẩn đã lưu) | Giống hệt từng byte |

## Bổ sung A2 — quy tắc múi giờ trước 1929 (2026-11-06, SAU KHI ĐÃ THẤY KẾT QUẢ)

Ghi rõ: thay đổi này được đề xuất **sau khi** kiểm định L9 lần 1 cho kết quả KHÔNG ĐẠT (Tết 1916 lệch mọi nguồn
Việt Nam; 4 tháng 1906–1920 lệch bảng lịch 1900–1999 do Hồ Ngọc Đức công bố). Lý do chấp nhận: có bằng chứng lịch
sử độc lập với kết quả — trước năm 1929 lịch Trung Hoa (lịch triều Nguyễn theo cùng phép tính) dùng giờ địa phương
Bắc Kinh; bảng lịch Đài Thiên văn Hồng Kông năm 1916 xác nhận. Quyết định của chủ dự án (06/11/2026):

- Trước 1929-01-01: giờ địa phương Bắc Kinh (UT+7:45:40, kinh độ 116°25′ Đ).
- 1929-01-01 → 1967-12-31: UTC+8. Từ 1968-01-01: UTC+7 (như A1).

Tiêu chí L9, L10, L11, L12, L13 giữ nguyên ngưỡng; phải chạy lại toàn bộ với quy tắc mới. Bên kiểm định độc lập tự
cài quy tắc từ bằng chứng lịch sử, không đọc mã lõi. Việc chấm L9 khi các nguồn Việt Nam mâu thuẫn nhau (1903, 1925,
1935, 1965) CHƯA được chủ dự án quyết định: báo cáo phải nêu kết quả theo từng nguồn, không tự chọn nguồn.

## Bổ sung A3 — quyết định của chủ dự án (2026-11-06, SAU KHI ĐÃ THẤY KẾT QUẢ)

1. **Phương pháp lịch mặc định đổi sang `Precise`** (Sóc theo Meeus đầy đủ + ΔT Espenak–Meeus; một mô hình Mặt Trời
   cho cả tháng nhuận và tiết khí). Căn cứ: L2 và L10 ĐẠT (0 sai lệch so với bên kiểm định độc lập 2000–2100 và
   1900–1999), không tự mâu thuẫn tháng nhuận, khớp 1237/1237 tháng của bảng lịch 1900–1999 do Hồ Ngọc Đức công bố
   (phương pháp `Hnd`: 1233/1237, sai vị trí tháng nhuận 1917 và 1922). `Hnd` vẫn giữ trong mã để đối chiếu.
2. **L9 khi các nguồn Việt Nam mâu thuẫn nhau** (1903, 1935, 1965; và 1925 khi một nguồn tự mâu thuẫn): chấm theo
   nguồn tuân thủ quy ước lịch sử đã chọn (lịch tính theo giờ Bắc Kinh trước 1929 / UTC+8 đến 1967). Biên bản phải
   liệt kê từng năm, ngày của mỗi nguồn và ghi rõ nguồn khác lệch bao nhiêu ngày. Đây là cách chấm được quyết định
   sau khi thấy kết quả; ngưỡng "0 sai lệch" giữ nguyên với các năm mà các nguồn thống nhất.
