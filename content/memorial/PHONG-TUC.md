# Biên bản nguồn phong tục — màn hình tưởng niệm (bản đầu, PWA đợt 3)

Trạng thái: **bản đầu, chưa đủ cho R9.** Lập ngày 2026-11-07 theo yêu cầu điều phối (vòng duyệt thiết kế TK-24). Tài liệu
chỉ ghi các lựa chọn mà **giao diện PWA đang dùng** và các nguồn đã đọc toàn văn trang. Việc **duyệt bởi người am hiểu
được hoãn** theo quyết định của chủ dự án ngày 06/11/2026 — rủi ro mở **R-CUL-1** (05 §7). Phong tục khác nhau theo
vùng, tôn giáo và từng gia đình (R-CUL-2), nên mọi chữ trên bia đều sửa được trong ứng dụng.

Quy tắc ghi nguồn: chỉ ghi điều mà trang nguồn **nói ra** (đã đọc toàn văn ngày 2026-11-07, trích nguyên văn ngắn). Điều
không tìm được nguồn kiểm chứng thì **không đưa vào ứng dụng** và ghi ở §4.

## 1. Cách gọi người đã khuất trên bia (dòng đầu bia — `memorial.bia_tieu_de`)

Ứng dụng gợi ý theo ô "Quan hệ"/"Danh xưng" và cho chọn một trong các cách gọi dưới đây; người dùng luôn sửa được, kể cả
để trống. Bia không lặp danh xưng ("Ông", "Bà") khi đã có dòng đầu bia.

| Cách gọi | Nghĩa (người đã khuất là…) | Nguồn | Ghi chú |
|---|---|---|---|
| Hiển khảo | cha | [N1] "Cha, mẹ mất đã chôn: Hiển Khảo, Hiến Tỷ"; [N2] "Nếu trước ngày giỗ bố thì phải khấn là: Hiển khảo" | [N1] in "Hiến Tỷ" (dấu sắc) — lỗi chính tả của trang; [N2] ghi "Hiển tỷ" |
| Hiển tỷ | mẹ | [N1]; [N2] "Nếu trước ngày giỗ mẹ thì phải khấn là: Hiển tỷ" | |
| Tổ khảo | ông | [N2] dùng "Tổ khảo", "Tổ tỷ" gắn với ông, bà | [N2] **không** có dạng "Hiển tổ khảo" |
| Tổ tỷ | bà | [N2] | như trên |
| Tằng tổ khảo | cụ ông | [N2] dùng "Tằng Tổ Khảo", "Tằng Tổ Tỷ" gắn với cụ ông, cụ bà | |
| Tằng tổ tỷ | cụ bà | [N2] | |

Thứ bậc đời (để hiểu "tổ", "tằng tổ"): [N3] "Trong phổ hệ chín đời gồm cao tổ, tằng tổ, tổ, phụ, bản thân, con, cháu,
chắt và chút" — tổ là đời ông, tằng tổ là đời cụ.

Cách gọi khác thấy trong nguồn nhưng **chưa** đưa vào gợi ý (giao diện bản này chỉ gợi ý theo quan hệ trực hệ): [N1] "Cha
mẹ vợ mất con tự xưng là: Hiển Nhạc Khảo, Hiển Nhạc Tỷ"; "Vợ: Tần…", "Chồng: Tiên Phu", "Con: Ái Tử". Người dùng vẫn tự
gõ được.

## 2. Bàn thờ trên màn hình tưởng niệm (một kiểu "truyền thống")

| Thành phần trong hình | Nguồn | Ghi chú |
|---|---|---|
| Bát hương ở giữa | [N4] "Đằng sau bát hương…", "đặt phía trước bát hương là một bát nước trong" (bát hương là vật trung tâm của bàn thờ); [N5] "cắm hương vào bát hương" | Hình chưa vẽ bát nước — xem §4 |
| Hai cây nến hai bên | [N4] "hai cây đèn tượng trưng cho mặt trời mặt trăng", "thắp hai ngọn nến tượng trưng cho hai vầng nhật nguyệt là đủ" | |
| Đĩa/mâm hoa quả | [N4] "đặt xen giữa đèn và hương là hai cái đĩa để đặt hoa quả lễ" | Hình vẽ **một** mâm quả (nguồn nói hai đĩa) — giản lược có chủ ý, ghi ở §4 |
| Bình hoa | [N4] "còn bày một chiếc độc bình lớn" | [N4] không nói bình đặt bên nào; hình đặt bên phải bát hương. Có nguồn khác bàn về bên trái/phải nhưng chưa đọc toàn văn — không dùng làm căn cứ |
| Ba nén hương | [N5] "Thông thường thắp 1 hoặc 3 nén hương là thắp cho gia tiên"; "3 nén hương tượng trưng cho sự hòa hợp của Thiên - Địa - Nhân" | Mục mở liên quan: không (O-1 … O-4 không bàn số nén) |

Hình vẽ bàn thờ, bia, nến, mâm quả, bình hoa, hương và khói là **hình vẽ mới cho dự án**, viết thẳng bằng SVG/CSS trong
`pwa/src/people/memorial.ts` (không có tệp ảnh trong thư mục này, không lấy hình từ nguồn ngoài).

## 3. Nguồn (đọc toàn văn ngày 2026-11-07)

| Mã | Nguồn |
|---|---|
| N1 | Hồng Phượng, "Tìm hiểu về cách xưng hô trong văn cúng của người Việt", VOH (Đài Tiếng nói Nhân dân TP.HCM), 23/09/2025. https://voh.com.vn/thuong-thuc/cach-xung-ho-trong-van-cung-622142.html |
| N2 | "Văn khấn ngoài mộ ngày giỗ", HoaTieu.vn, ngày 09/11/2025, không ghi tác giả. https://hoatieu.vn/tai-lieu/van-khan-ngoai-mo-ngay-gio-206140 — **nguồn yếu** (trang tổng hợp tài liệu, không tác giả); chỉ dùng cho cách gọi Tổ khảo/Tổ tỷ/Tằng tổ khảo/Tằng tổ tỷ, cần người am hiểu xác nhận (R-CUL-1) |
| N3 | An Chi (học giả), trích "Chuyện đông chuyện tây", Znews, 07/07/2026, "Cửu huyền thất tổ là những đời tổ nào". https://znews.vn/cuu-huyen-that-to-la-nhung-doi-to-nao-post1666494.html |
| N4 | "Bài trí và ý nghĩa của bàn thờ tổ tiên ngày Tết", VOV, 16/01/2009 (ghi "Theo VHNT Ẩm thực"). https://vov.vn/huong-vi-tet/bai-tri-va-y-nghia-cua-ban-tho-to-tien-ngay-tet |
| N5 | Phương Nga, "Thắp hương ngày Tết thế nào cho đúng?", VOH, 09/02/2024. https://voh.com.vn/thuong-thuc/thap-huong-ngay-tet-514764.html |

## 4. Chưa kiểm chứng được — không đưa vào ứng dụng / còn thiếu

- **"Hiển tổ khảo", "Hiển tổ tỷ", "Hiển tằng tổ khảo", "Hiển tằng tổ tỷ"** (dạng có chữ "Hiển" cho ông, bà, cụ): đã tìm
  nhưng **không thấy** trong trang nào đọc được toàn văn (các trang văn khấn khác không tải được hoặc chỉ có chỗ trống
  "Hiển…"). Vì vậy ứng dụng **không gợi ý** các dạng này mà gợi ý "Tổ khảo", "Tổ tỷ", "Tằng tổ khảo", "Tằng tổ tỷ" theo
  [N2]. Người dùng vẫn tự gõ được. Cần người am hiểu xác nhận cách viết đúng trên bia (R-CUL-1).
- "Cố" (đã có trong gợi ý bản trước): không tìm được nguồn → đã bỏ khỏi gợi ý.
- Bát nước trong, hai đĩa quả (thay vì một mâm), đỉnh trầm, mâm cỗ mặn ngày giỗ: chưa vẽ.
- Kiểu Phật giáo (mâm chay) và Công giáo (nến, hoa, không hương) của 01 §3.7: chưa có hình riêng, chưa có nguồn.
- Giỗ đầu, giỗ hết (năm 2/3), tiên thường, hưởng thọ/hưởng dương (ngưỡng 60), tháng nhuận → tháng thường, ngày 30 → 29:
  là lựa chọn đã duyệt ở G1 (06, O-1 … O-4) nhưng **chưa** có dòng nguồn trong biên bản này — phải bổ sung trước khi chấm R9.
