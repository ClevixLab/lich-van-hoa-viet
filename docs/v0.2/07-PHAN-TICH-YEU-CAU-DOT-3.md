# Phân tích yêu cầu, rủi ro và phân công — đợt 3 v0.2: mô-đun Người đã mất và Người đang sống

Trạng thái: **ĐÃ DUYỆT (cổng G1-Đ3) — chủ dự án đồng ý ngày 07/11/2026 lúc 06:17**, kèm ba điểm mặc định ở §9 và yêu cầu
bổ sung YC-2026-05. Lập theo quy trình docs/08, ngày 2026-11-07.

Ghi rõ để trung thực: việc lập trình đợt 3 (lõi, PWA, kiểm định) đã khởi động lúc 02:3x ngày 07/11/2026, **trước khi
có tài liệu này**. Tiêu chí R13 đã khóa trước khi viết mã (04, Bổ sung B2), nhưng phân tích yêu cầu, đặc tả giao diện
và sổ rủi ro được lập bù sau đó. Không PR nào của đợt 3 được gộp trước khi tài liệu này và đặc tả giao diện (08) được
duyệt; nếu duyệt đổi yêu cầu, phần mã đã viết được sửa theo.

## 1. Phiếu yêu cầu

| Mã | Ngày | Nguyên văn chủ dự án | Ưu tiên |
|---|---|---|---|
| YC-2026-01 | 06/11 | Mô-đun giỗ: danh xưng, tên hiệu, quan hệ, ngày sinh, nơi sinh, ngày mất dương (tự đổi âm) hoặc âm, nơi an táng; nhắc 1/3/5/10/15 ngày hoặc tự đặt; mâm cúng có bia, ảnh, 3 nén hương; sinh nhật; sự kiện (tốt nghiệp, cưới, tự đặt) | Cao |
| YC-2026-02 | 06/11 | Ai cũng tự thêm được, không cần đăng ký; muốn lưu lâu dài, chia sẻ thì cần tài khoản (v0.3) | Cao |
| YC-2026-03 | 07/11 | "anh muốn bổ sung ngay module cho người đã mất, và module cho người còn sống; lưu ý người dùng có thể xem danh sách sự kiện cho người sống/mất theo tuần/tháng/quý/năm" | Cao |
| YC-2026-04 | 07/11 | Mọi mở rộng đều theo quy trình chuyên nghiệp (docs/08) | Bắt buộc |
| YC-2026-05 | 07/11 | "Khi cập nhật một người từ sống thành mất thì nên có phần ghi chú ( ai báo, mất ở đâu, lý do, giấy báo tử nếu có)" | Cao |
| YC-2026-08 | 07/11 | "phần mềm phải đảm bảo khi nâng cấp không làm mất dữ liệu đã có của khách hàng và updated vào version mới trên máy tính, hoặc điện thoại của khách hàng nhé" — 04 Bổ sung B5, R14 | Bắt buộc |
| YC-2026-07 | 07/11 | "trong phần cách gọi, các đại từ nhân xưng như: Ông/Bà/Cụ/Kỵ/Bác/Cô/Chú/Dì/Cậu/Mợ/Thím/Anh/Chị/Cháu/Con/Bạn, Ông Tổ họ, Bà Cô Tổ họ, Bé đỏ... đều phải viết hóa chữ cái đầu tiên để thể hiện sự tôn trọng. Thông tin báo tử nên cho phép người dùng load và lưu file ảnh tin báo tử, hoặc bản scanned giấy chứng tử, hoặc một trường text để có thể note lại. Lưu ý các đại từ nhân xưng như: Cụ, Kỵ, Bà Cô Tổ, Ông Tổ họ, Bé đỏ không xuất hiện trong mục danh xưng người sống nhé (cách gọi đứng trước tên). Trong mục người thân nên có thêm mục giới tính" — xử lý ở 01 §3.1, §3.2a, §3.8a | Cao |
| YC-2026-06 | 07/11 | Với người đã mất chỉ nhắc ngày giỗ; dùng từ thường ngày (ông/bà/cụ/kỵ…), bỏ "Hiển khảo" (04 Bổ sung B4) | Cao |

## 2. Câu chuyện người dùng

Người dùng chính: người lo việc giỗ chạp trong gia đình (thường 35–70 tuổi, dùng điện thoại là chính, có thể không
quen công nghệ); người trẻ muốn nhớ sinh nhật và ngày giỗ ông bà.

**Người đã mất**
- **US-M-01** Là người lo giỗ, tôi muốn thêm hồ sơ người đã mất bằng ngày mất dương lịch và thấy ngay ngày âm tương
  ứng, để khỏi tra lịch.
  Cho ngày mất dương hợp lệ / Khi lưu / Thì hiện ngày âm (có ghi "nhuận" nếu là tháng nhuận) và ngày giỗ năm nay. — R1, R10
- **US-M-02** … thêm bằng ngày âm (kể cả tháng nhuận, ngày 30), vì gia đình chỉ nhớ ngày âm. Khi ngày âm không tồn tại
  (ví dụ 30 của tháng thiếu) thì báo lỗi bằng lời thường. — R1, 01 §6.1
- **US-M-03** … biết giỗ năm nay là giỗ đầu, giỗ hết hay giỗ thường, và hưởng thọ / hưởng dương bao nhiêu. — R10, O-2, O-3
- **US-M-04** … được nhắc trước 1/3/5/10/15 ngày hoặc số ngày tôi chọn, đúng giờ tôi chọn. — R3, R5, R12
- **US-M-05** … mở màn hình tưởng niệm vào ngày giỗ: bia, ảnh, mâm lễ, ba nén hương. — R8, R9
- **US-M-06** … ghi nơi an táng, quan hệ với tôi, tên hiệu, ảnh chân dung. — 01 §3.1, S9
- **US-M-07** Là người trong gia đình, khi một người thân đang sống qua đời, tôi muốn **cập nhật hồ sơ đó thành người
  đã mất** (không phải tạo hồ sơ mới) và ghi lại **thông tin báo tử**: ai báo tin, mất ở đâu, lý do, ảnh giấy báo tử
  nếu có, ghi chú — để gia đình có một chỗ lưu giữ.
  Cho hồ sơ người đang sống / Khi bấm "Cập nhật: đã mất", nhập ngày mất và (tùy chọn) thông tin báo tử / Thì hồ sơ
  chuyển sang nhóm đã mất, giữ nguyên tên, ảnh, ngày sinh; lời nhắc sinh nhật và sự kiện của người đó bị gỡ (B4); bắt
  đầu có ngày giỗ; thông tin báo tử chỉ hiện trên trang hồ sơ. Người tạo hồ sơ người đã mất mới cũng nhập được phần
  này. — 01 §3.2a, R2, R6, S9, S10, B4

**Người đang sống**
- **US-S-01** … thêm người thân với ngày sinh dương, tùy chọn nhắc thêm sinh nhật âm. — 02 §5–6, R10
- **US-S-02** … thêm sự kiện riêng (tốt nghiệp, cưới, tự đặt), lặp hằng năm hoặc một lần, theo ngày dương hoặc âm. — 01 §3.4
- **US-S-03** … biết sinh nhật năm nay là lần thứ mấy, tuổi bao nhiêu. — R10

**Chung**
- **US-C-01** Xem danh sách sự kiện theo **tuần / tháng / quý / năm**, lùi tới từng kỳ, về "hôm nay". — **R13**
- **US-C-02** Lọc danh sách: tất cả / người đã mất / người đang sống. — **R13**
- **US-C-03** Thấy "Sắp tới" 30 ngày ngay khi mở mô-đun. — R11
- **US-C-04** Xuất lịch .ics để lịch của điện thoại nhắc khi ứng dụng web đóng. — R4, S10
- **US-C-05** Sao lưu ra tệp có mật khẩu và khôi phục trên máy khác. — R2, R6, S10
- **US-C-06** Sửa, xóa hồ sơ; xóa phải hỏi lại. — S10
- **US-C-07** Không cần tài khoản, không cần mạng, dữ liệu chỉ ở trên máy. — YC-2026-02, S4', S7

## 3. Phạm vi

**Có:** mọi câu chuyện ở §2 trên PWA, Android, iPhone/iPad/Mac và Windows; thông báo cục bộ trên bản cài; thẻ nhắc
trong ứng dụng và .ics trên bản web.
**Không (để v0.3 trở đi):** tài khoản, đồng bộ, chia sẻ qua số điện thoại, sao lưu lên máy chủ, bài khấn, thông báo
tang gia, đi lễ, ngôn ngữ khác ngoài tiếng Việt (tiếng Anh làm ở đợt i18n riêng).

## 4. Yêu cầu phi chức năng

Riêng tư (S4', S5, S7, O-9: màn hình khóa không hiện tên); ngoại tuyến hoàn toàn; tải lần đầu bản web < 500 KB gzip
(P3; mô-đun người thân tải lười); bộ nhớ (P4, P9); trợ năng (bàn phím, đọc màn hình, tương phản, cỡ chữ A/A+/A++,
giảm chuyển động); giọng văn trang trọng, ấm áp khi nói về người đã khuất; bốn nền tảng dùng chung lõi Rust, bản web
dùng bản TypeScript trùng từng byte.

## 5. Phân tích ảnh hưởng

| Phần đã có | Ảnh hưởng | Xử lý |
|---|---|---|
| Lịch mặc định đổi sang Jpl (0.1.4) | Ngày âm của ngày mất sau 2077 có thể khác bản cũ | R1 chạy lại với Jpl (B2 mục 3); hồ sơ lưu cả ngày âm đã quy đổi và phương pháp |
| Tệp sao lưu | Thêm phương pháp "jpl" | Đã làm ở 0.1.4; tệp cũ vẫn mở được |
| Dung lượng tải bản web | Mô-đun mới làm tăng | Tải lười; đo P3 |
| CSP bản web | Ảnh chân dung không được dùng `blob:` | Vẽ ảnh lên `<canvas>` (R-CSP-1) |
| Ứng dụng native | Cần FFI mới, quyền thông báo | FFI mở rộng (vai lõi); quyền theo S6, O-6 |

## 6. Sổ rủi ro (bổ sung cho 05 §7)

| Mã | Rủi ro | Khả năng | Tác động | Cách giảm | Theo dõi |
|---|---|---|---|---|---|
| R-PROC-1 | Lập trình đi trước thiết kế → làm lại | Cao (đã xảy ra) | Vừa | Không gộp PR trước khi duyệt 07 và 08; đặc tả giao diện viết trước phần còn lại | Điều phối viên |
| R-UX-1 | Người lớn tuổi khó nhập ngày âm, tháng nhuận | Vừa | Cao | Bộ chọn ngày âm có ô "nhuận" chỉ hiện khi năm đó có tháng nhuận; xem trước ngày dương; lời báo lỗi thường | Vai giao diện |
| R-UX-2 | Danh sách năm quá dài (nhiều hồ sơ) | Vừa | Thấp | Nhóm theo tháng, có tiêu đề tháng; cuộn mượt | Vai giao diện |
| R-DATA-2 | Người dùng mất dữ liệu bản web khi trình duyệt dọn | Vừa | Cao | `persist()`, nhắc sao lưu, khuyên cài lên màn hình chính (R-PWA-1) | Vai PWA |
| R-CUL-2 | Hình mâm cúng, bia không hợp phong tục vùng miền | Vừa | Vừa | Một phong cách trung tính trước; nguồn trong `content/memorial/PHONG-TUC.md`; thêm phong cách ở đợt 4 | Vai nội dung |
| R-NAT-1 | Bốn nền tảng làm giao diện riêng → lệch hành vi | Cao | Vừa | Mọi tính toán ở lõi qua FFI; giao diện chỉ hiển thị; cùng đặc tả 08; ca kiểm thử giao diện chung | Điều phối viên |
| R-PRIV-2 | Lý do mất và giấy báo tử là thông tin nhạy cảm | Vừa | Cao | Chỉ lưu trên máy; không bao giờ đưa vào thông báo, .ics, màn hình tưởng niệm, tên tệp, URL; có trong sao lưu mã hóa; hộp xác nhận xuất ghi rõ "gồm thông tin báo tử"; ảnh giấy tờ bỏ siêu dữ liệu (S9) | Vai lõi + PWA |
| R-PDF-1 | Tệp PDF quét giấy chứng tử lưu nguyên bản, có thể còn siêu dữ liệu (tên máy quét, phần mềm) | Vừa | Thấp | Chỉ lưu trên máy và trong sao lưu mã hóa; giao diện báo rõ; ảnh thì luôn bỏ siêu dữ liệu | Vai lõi |
| R-PERF-1 | Tính danh sách năm cho nhiều hồ sơ chậm trên máy yếu | Thấp | Vừa | Đo với 200 hồ sơ; ngưỡng ghi vào biên bản | Vai lõi |

## 7. Kế hoạch và phân công

| Việc | Vai | Nhánh | Phụ thuộc | Đầu ra | Trạng thái |
|---|---|---|---|---|---|
| T1 Phân tích yêu cầu, rủi ro, phân công (tài liệu này) | Điều phối viên | `main` qua PR | — | 07 | Đang duyệt |
| T2 Sửa R12; đặc tả §7.6 + hàm danh sách theo kỳ (Rust + TS); FFI cho native | Lõi | `v0.2-w3-core` | — | PR, dump r12/r13 | Đang làm |
| T3 Đặc tả giao diện 08, rồi giao diện PWA | Giao diện PWA | `v0.2-w3-pwa` | T2 (tạm dùng lớp chuyển tiếp) | 08, PR, ảnh chụp | Đang làm |
| T4 Kiểm định R1 (Jpl), R12, R13, R3/R11 | Bên kiểm định độc lập | `oracle-v02-b2` | Đặc tả §7.6 của T2 | Báo cáo B2 | Đang làm |
| T5 Duyệt thiết kế 08 | Rà soát giao diện (vai riêng) | — | T3 bước 1 | Phiếu duyệt | Chờ |
| T6 Giao diện Android | Giao diện Android | `v0.2-w3-android` | T2 (FFI), T5 | PR, ảnh chụp | Chờ |
| T7 Giao diện iPhone/iPad/Mac | Giao diện Apple | `v0.2-w3-apple` | T2, T5 | PR, ảnh chụp | Chờ |
| T8 Giao diện Windows | Giao diện Windows | `v0.2-w3-windows` | T2, T5 | PR, ảnh chụp | Chờ |
| T9 Duyệt mã từng PR | Rà soát mã (vai riêng) | — | T2, T3, T6–T8 | Phiếu duyệt | Chờ |
| T10 Rà soát bảo mật và riêng tư đợt 3 | Rà soát bảo mật | — | T3, T6–T8 | Báo cáo | Chờ |
| T11 Thử trên máy thật (R5, P8, P9) | Chủ dự án + điều phối viên | — | T6–T8 | Biên bản | Chờ |

Song song: T2, T3, T4 chạy cùng lúc; T6–T8 chạy cùng lúc ngay khi T2 có FFI và T5 duyệt xong. Mỗi vai nhận bản giao
việc theo mẫu docs/08 §8.

## 8. Định nghĩa "xong" của đợt 3

Mọi câu chuyện §2 có màn hình và kiểm thử tương ứng trên bốn nền tảng; R12 và R13 ĐẠT; R1 với Jpl có kết luận; P3, P4
trong ngưỡng; ảnh chụp mọi màn hình mới (360 px, 1280 px, sáng, tối) đã được xem; duyệt thiết kế và duyệt mã không
còn lỗi mức chặn; chủ dự án ký.

## 9. Câu hỏi cho chủ dự án (có mặc định nếu chưa trả lời)

1. Mô-đun đặt tên "Người thân" (một thẻ, bên trong chia "Đã mất" / "Đang sống") hay hai thẻ riêng? **Mặc định:** một
   thẻ "Người thân".
2. Tuần bắt đầu Thứ Hai (như lịch Việt Nam thông dụng)? **Mặc định:** Thứ Hai.
3. Danh sách theo kỳ chỉ dùng tháng/quý/năm **dương lịch**, hay thêm chế độ tháng/năm **âm lịch**? **Mặc định:**
   dương lịch, mỗi mục ghi kèm ngày âm; tháng âm để đợt sau nếu anh muốn.
