# Biên bản nguồn phong tục — màn hình tưởng niệm (bản đầu, PWA đợt 3)

Trạng thái: **bản đầu, chưa đủ cho R9.** Lập ngày 2026-11-07 theo yêu cầu điều phối (vòng duyệt thiết kế TK-24). Sửa
cùng ngày theo quyết định của chủ dự án: bia chỉ ghi quan hệ bằng từ thường ngày, bỏ từ Hán-Việt cổ (§1). Tài liệu
chỉ ghi các lựa chọn mà **giao diện PWA đang dùng** và các nguồn đã đọc toàn văn trang. Việc **duyệt bởi người am hiểu
được hoãn** theo quyết định của chủ dự án ngày 06/11/2026 — rủi ro mở **R-CUL-1** (05 §7). Phong tục khác nhau theo
vùng, tôn giáo và từng gia đình (R-CUL-2), nên chữ trên bia lấy từ ô "Quan hệ" mà người dùng tự gõ.

Quy tắc ghi nguồn: chỉ ghi điều mà trang nguồn **nói ra** (đã đọc toàn văn ngày 2026-11-07, trích nguyên văn ngắn). Điều
không tìm được nguồn kiểm chứng thì **không đưa vào ứng dụng** và ghi ở §4.

## 1. Chữ trên bia: quan hệ bằng từ thường ngày

**Quyết định của chủ dự án (07/11/2026):** không dùng các từ Hán-Việt cổ, ít người còn hiểu (như "Hiển khảo"). Phần lớn
gia đình bây giờ không biết, không hiểu và không dùng các từ này. Vì vậy:

- Ứng dụng **không có** ô "dòng đầu bia" riêng và **không gợi ý** từ Hán-Việt nào.
- Bia trên màn hình tưởng niệm chỉ ghi, phía trên tên, **đúng chữ người dùng gõ ở ô "Quan hệ"**. Ứng dụng viết hoa chữ
  đầu và bỏ phần trong ngoặc, ví dụ "ông nội (bên nội)" → "Ông nội". Ô "Quan hệ" trống thì không ghi gì.
- Ô "Quan hệ" có danh sách gợi ý bằng từ thường ngày. Người dùng vẫn gõ tự do được.
- Trường `memorial.bia_tieu_de` của 01 vẫn được giữ trong dữ liệu để sao lưu dùng được giữa các nền tảng. Giao diện PWA
  không hiển thị và không sửa trường này (08 §12e).

| Gợi ý trong ô "Quan hệ" | Người đã khuất là… | Nguồn |
|---|---|---|
| Bố, Mẹ | cha, mẹ | từ thường ngày |
| Ông nội, Bà nội | cha, mẹ của cha | [N1] "Ông Nội: Cha của cha"; "Miền Bắc và miền Nam đều gọi chung là “ông Nội, bà Nội”" |
| Ông ngoại, Bà ngoại | cha, mẹ của mẹ | [N1] "Bà Ngoại: Mẹ của mẹ" |
| Cụ ông, Cụ bà | đời trên ông bà (miền Bắc) | [N1] "người Bắc thường gọi là “Cụ” trong khi ở miền Nam thường sẽ gọi là “Cố”" |
| Ông cố, Bà cố | như trên (miền Nam) | [N1], cùng câu trên |
| Kỵ ông, Kỵ bà | đời trên cụ (miền Bắc và miền Trung) | [N1] "Trên ông Cố sẽ gọi là “Kỵ ông/ Kỵ bà” ở miền Bắc và miền Trung" |
| Ông sơ, Bà sơ | như trên (miền Nam) | [N1] "Còn miền Nam sẽ gọi là “ông Sơ” và “bà Sơ”" |
| Bác, Chú, Thím, Cô, Dì, Cậu, Mợ, Anh, Chị, Em, Chồng, Vợ, Con | họ hàng gần | từ thường ngày; cách gọi cụ thể theo vùng và từng nhà, nên người dùng tự sửa |

"Kỵ" ở đây là cách gọi của miền Bắc và miền Trung. Miền Nam gọi "sơ" [N1]. Danh sách gợi ý có cả hai cách gọi để người
dùng chọn theo gia đình mình.

Ô "Cách gọi (đứng trước tên)" (`danh_xung` — "ông", "bà", "cụ", "kỵ"…) chỉ dùng khi ô "Quan hệ" trống. Khi bia đã ghi
quan hệ thì không ghi cách gọi trước tên nữa, để không lặp ("Ông nội / Nguyễn Văn A", không phải "Ông nội / Ông Nguyễn Văn A").

## 2. Bàn thờ trên màn hình tưởng niệm (một kiểu "truyền thống")

| Thành phần trong hình | Nguồn | Ghi chú |
|---|---|---|
| Bát hương ở giữa | [N2] "Đằng sau bát hương…", "đặt phía trước bát hương là một bát nước trong" (bát hương là vật trung tâm của bàn thờ); [N3] "cắm hương vào bát hương" | Hình chưa vẽ bát nước — xem §4 |
| Hai cây nến hai bên | [N2] "hai cây đèn tượng trưng cho mặt trời mặt trăng", "thắp hai ngọn nến tượng trưng cho hai vầng nhật nguyệt là đủ" | |
| Đĩa/mâm hoa quả | [N2] "đặt xen giữa đèn và hương là hai cái đĩa để đặt hoa quả lễ" | Hình vẽ **một** mâm quả (nguồn nói hai đĩa) — giản lược có chủ ý, ghi ở §4 |
| Bình hoa | [N2] "còn bày một chiếc độc bình lớn" | [N2] không nói bình đặt bên nào; hình đặt bên phải bát hương. Có nguồn khác bàn về bên trái/phải nhưng chưa đọc toàn văn — không dùng làm căn cứ |
| Ba nén hương | [N3] "Thông thường thắp 1 hoặc 3 nén hương là thắp cho gia tiên"; "3 nén hương tượng trưng cho sự hòa hợp của Thiên - Địa - Nhân" | Mục mở liên quan: không (O-1 … O-4 không bàn số nén) |

Hình vẽ bàn thờ, bia, nến, mâm quả, bình hoa, hương và khói là **hình vẽ mới cho dự án**, viết thẳng bằng SVG/CSS trong
`pwa/src/people/memorial.ts` (không có tệp ảnh trong thư mục này, không lấy hình từ nguồn ngoài).

## 3. Nguồn (đọc toàn văn ngày 2026-11-07)

| Mã | Nguồn |
|---|---|
| N1 | Hồng Phượng, "Sơ đồ cách xưng hô trong họ hàng…", VOH (Đài Tiếng nói Nhân dân TP.HCM), 31/01/2025. https://voh.com.vn/thuong-thuc/so-do-cach-xung-ho-trong-ho-hang-575769.html |
| N2 | "Bài trí và ý nghĩa của bàn thờ tổ tiên ngày Tết", VOV, 16/01/2009 (ghi "Theo VHNT Ẩm thực"). https://vov.vn/huong-vi-tet/bai-tri-va-y-nghia-cua-ban-tho-to-tien-ngay-tet |
| N3 | Phương Nga, "Thắp hương ngày Tết thế nào cho đúng?", VOH, 09/02/2024. https://voh.com.vn/thuong-thuc/thap-huong-ngay-tet-514764.html |

## 4. Chưa kiểm chứng được — không đưa vào ứng dụng / còn thiếu

- Cách gọi họ hàng cụ thể (bác/chú/cô/dì/cậu/mợ/thím…) khác nhau theo vùng và từng nhà. Ứng dụng chỉ gợi ý, không
  tự suy ra từ sơ đồ họ hàng.
- Bát nước trong, hai đĩa quả (thay vì một mâm), đỉnh trầm, mâm cỗ mặn ngày giỗ: chưa vẽ.
- Kiểu Phật giáo (mâm chay) và Công giáo (nến, hoa, không hương) của 01 §3.7: chưa có hình riêng, chưa có nguồn.
- Giỗ đầu, giỗ hết (năm 2/3), tiên thường, hưởng thọ/hưởng dương (ngưỡng 60), tháng nhuận → tháng thường, ngày 30 → 29:
  là lựa chọn đã duyệt ở G1 (06, O-1 … O-4) nhưng **chưa** có dòng nguồn trong biên bản này — phải bổ sung trước khi chấm R9.
