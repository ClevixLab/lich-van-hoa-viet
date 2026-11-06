# Quy tắc Ngọc Hạp — bảng quy tắc có dẫn nguồn (lvhv-ngochap v0.1)

Tài liệu cho tiêu chí **N1** (mỗi quy tắc ≥ 1 nguồn kiểm chứng được) và **N2** (mâu thuẫn được liệt kê,
ghi phương án chọn). Mã nguồn nguồn (N01…, F01…) xem `SOURCES.md`. Bảng chính ở cuối tài liệu được
**sinh tự động từ mã** (`cargo test dump_rule_table -- --ignored --nocapture`) rồi ghép cột nguồn, nên
định nghĩa trong bảng chính là đúng thứ mã đang chạy.

Quy tắc chọn khi nguồn mâu thuẫn (đăng ký trước): ưu tiên bản được **nhiều dòng nguồn độc lập** ghi;
hòa thì ưu tiên bản dẫn sách in cũ hơn; vẫn hòa thì phân xử bằng cấu trúc nội tại của bảng (chu kỳ,
quan hệ xung/hợp) và ghi rõ là phân xử cấu trúc. Không tự thêm quy tắc để khớp trang đối chiếu.

## 1. Tổng quan

| Hạng mục | Số lượng |
|---|---|
| Sao tốt (G01–G49) | 49 |
| Sao xấu (B01–B56) | 56 |
| Ngày đặc biệt (S01 Tam Nương, S02 Nguyệt Kỵ, S03 Dương Công Kỵ Nhật, S04 Thọ Tử) | 4 |
| 12 thần Hoàng đạo / Hắc đạo (T01–T12) | 12 (6 tốt, 6 xấu) |
| Bảng tuổi xung (X01) | 60 dòng |
| Quy tắc G/B/S có ≥ 2 dòng nguồn độc lập | 41 / 109 |
| Quy tắc G/B/S chỉ có 1 dòng nguồn | 68 / 109 (phần lớn chỉ có dòng TVTP: N01/N02) |
| Mâu thuẫn được ghi (C01–C18) | 18 |

"Dòng nguồn" = nhóm biên soạn: TVTP (Tân Việt & Thiều Phong 1997: N01, N02, N03), PKB (chú giải
Phan Kế Bính: N05–N08), LHĐP (N10), cùng các nguồn lẻ (N04, N09, N11, N13/N16 Trung Hoa,
N17 Nhật, N19/N20 tứ trụ). Hai trang chép cùng một sách tính là **một** dòng.

## 2. Quy ước

### 2.1 Tháng âm hay tháng tiết (bằng chứng)
Bảng sao tra theo **số tháng âm lịch**; tháng nhuận dùng bảng của tháng chính cùng số. Bằng chứng:
- Trang đối chiếu F02 (2026-11-08, âm 30/9): ngày này đã qua Lập Đông (07/11), tức tháng tiết là Hợi
  (tháng 10), nhưng trang vẫn ghi Thổ Phủ (tháng 9 = Tuất), Tam Tang, Ly Sàng (mùa Thu = Tuất) — khớp
  bảng **tháng 9 âm**, không khớp tháng 10.
- N01/N02 ghi cột theo tên tháng âm (Giêng … Chạp).
- Ngoại lệ chưa xử lý: N21 định mùa cho Thiên Xá theo tiết khí (xem C05). `DayCtx.month_chi_tiet` hiện
  không dùng.

### 2.2 Mùa
Xuân = tháng 1–3, Hạ = 4–6, Thu = 7–9, Đông = 10–12 (tháng âm). Nhóm Mạnh = 1, 4, 7, 10;
Trọng = 2, 5, 8, 11; Quý = 3, 6, 9, 12 (dùng cho Sát Cống, Trực Tinh, Nhân Chuyên).

### 2.3 Bảng N02 chỉ điền 4 cột
Các dòng B40–B45, B47–B50 trong N02 chỉ điền cột Giêng, Mười, M.Một, Chạp; 8 ô còn lại để trống.
Dự án **diễn giải** đó là bốn mùa (Xuân/Hạ/Thu/Đông, vì 4 cột liền nhau Mười–Chạp không thể là 3 tháng
cùng mùa). Bằng chứng ủng hộ: (a) Thổ Cấm (B48) được dòng PKB ghi đủ 12 tháng = đúng mẫu theo mùa;
(b) F01 (tháng 9: Tứ Thời Cô Quả ở Mùi), F02 (tháng 9: Tam Tang, Ly Sàng ở Tuất), F03 (tháng 10:
Nguyệt Kiến Chuyển Sát, Phủ Đầu Sát ở Tý) đều khớp cách diễn giải này. Đây là diễn giải, không phải
quy tắc nguồn ghi tường minh.

### 2.4 Tên gọi
Dùng tên thông dụng: "Hà Khôi (Cẩu Giảo)", "Nguyệt Hỏa (Độc Hỏa)", "Vãng Vong (Thổ Kỵ)",
"Phủ Đầu Sát" (N02 viết "Phủ đầu dát"), "Thổ Cấm" (N02 viết "Thổ cẩm"), "Ngũ Quỷ" (N02 viết "Ngũ Quỹ").
"Thụ Tử" (B14, theo chi) và "Thọ Tử" (S04, theo can chi) là hai quy tắc khác nhau của cùng tên gọi
dân gian; S04 luôn nằm trong B14.

## 3. 12 thần Hoàng đạo / Hắc đạo (T01–T12)

Chỉ số thần do lõi tính (`core/src/canchi.rs`: tháng 1, 7 khởi Thanh Long ở Tý; 2, 8 ở Dần; …).
Crate chỉ gắn tên, trọng số và tác dụng. Nguồn: N01 (bảng Thanh Long, Minh Đường, Kim Đường, Ngọc
Đường), N02 (Chu Tước, Bạch Hổ, Huyền Vũ, Câu Trần), N07 (Thiên Hình, Thiên Lao trong nhóm xấu mọi việc).
Kiểm thử `stars_coinciding_with_than` xác nhận 6 cặp "sao trùng thần" mà N01 ghi (Thiên Thành =
Ngọc Đường, Thiên Quan = Tư Mệnh, Thiên Tài = Kim Quỹ, Địa Tài = Kim Đường, Minh Tinh = Thiên Lao,
Thiên Mã = Bạch Hổ) đúng với quy tắc 12 thần của lõi ở mọi tháng — một kiểm tra chéo độc lập giữa
bảng sao và quy tắc thần.

| Mã | Thần | Loại | Trọng số | Kỵ | Kỵ chính |
|---|---|---|---|---|---|
| T01 | Thanh Long Hoàng Đạo | tốt | 2 | — | — |
| T02 | Minh Đường Hoàng Đạo | tốt | 2 | — | — |
| T03 | Thiên Hình Hắc Đạo | xấu | 2 | mọi việc | — |
| T04 | Chu Tước Hắc Đạo | xấu | 2 | mọi việc, nhất là nhập trạch, khai trương, cầu tài | nhap_trach, khai_truong, cau_tai |
| T05 | Kim Quỹ Hoàng Đạo | tốt | 2 | — | — |
| T06 | Kim Đường Hoàng Đạo (lõi cũng gọi "Kim Đường" từ 2026-11-06; nơi khác "Bảo Quang", "Thiên Đức") | tốt | 2 | — | — |
| T07 | Bạch Hổ Hắc Đạo | xấu | 2 | mọi việc, nhất là an táng | an_tang |
| T08 | Ngọc Đường Hoàng Đạo | tốt | 2 | — | — |
| T09 | Thiên Lao Hắc Đạo | xấu | 2 | mọi việc | — |
| T10 | Huyền Vũ Hắc Đạo | xấu | 2 | mọi việc, nhất là an táng | — |
| T11 | Tư Mệnh Hoàng Đạo | tốt | 2 | — | — |
| T12 | Câu Trần Hắc Đạo | xấu | 2 | mọi việc, nhất là an táng | — |

Ghi chú: N02 ghi Bạch Hổ "trùng ngày với Thiên Giải → sao tốt"; luật hóa giải này **chưa cài**.

## 4. Tuổi xung với ngày (X01)

**Kết luận về quy tắc:** quy tắc "Thiên khắc Địa xung" thuần (can khắc can + chi xung chi; N24) cho
ngày Quý Mùi các tuổi Đinh Sửu, Kỷ Sửu — **không** khớp các trang lịch (F01 ghi Ất Sửu, Tân Sửu,
Đinh Hợi, Đinh Tỵ). Các trang lịch Việt dùng một bảng 60 dòng có sẵn. Dự án **mã hóa nguyên bảng**
(N22, đủ 60 dòng; N23 khớp 20/20 dòng đầu; F01, F02, F03 khớp ba dòng Quý Mùi, Bính Tuất, Mậu Tý).

Cấu trúc quan sát được (dự án tự phân tích, chỉ để giải thích, không dùng để tính): mỗi dòng gồm
(a) các tuổi có chi lục xung với chi ngày và nạp âm khắc với nạp âm ngày (hai chiều), và (b) với ngày
có can Giáp/Ất/Bính/Đinh/Canh/Tân/Nhâm/Quý, các tuổi có can "xung" (Giáp–Canh, Ất–Tân, Bính–Nhâm,
Đinh–Quý; Mậu, Kỷ không có) và nạp âm khắc. Luật (a)+(b) tái tạo đúng **47/60** dòng; 13 dòng lệch
(Bính Dần, Đinh Mão, Bính Tuất, Đinh Hợi, Nhâm Thìn, Quý Tỵ, Giáp Ngọ, Bính Thân, Đinh Dậu, Bính Thìn,
Đinh Tỵ, Nhâm Tuất, Quý Hợi), phần lớn ở chỗ có hay không tuổi cùng chi. Vì không suy được quy tắc
gọn, dùng bảng. Dòng 31 (Giáp Ngọ: "…Canh Dần, Nhâm Dần") có thể là lỗi chép (Nhâm Dần không khắc
theo cả hai luật); giữ nguyên như nguồn.

Năm sinh → Can Chi theo quy ước lõi (năm âm lịch: can = (Y+6) mod 10, chi = (Y+8) mod 12). Người
sinh tháng Chạp/Giêng cần dùng **năm âm lịch** của ngày sinh — trách nhiệm của tầng giao diện.

## 5. Đối chiếu trang lịch công khai

| Ngày | Trang ghi | Crate cho ra | Kết quả |
|---|---|---|---|
| 2026-11-05 (F01) âm 27/9, Quý Mùi, Trực Thu, Chu Tước | tốt: Đại Hồng Sa, Sát Cống, Thiên Ân, U Vi Tinh; xấu: Tam Nương, Băng Tiêu Ngọa Hãm, Chu Tước Hắc Đạo, Địa Phá, Hà Khôi/Cẩu Giảo, Hoang Vu, Ngũ Hư, Nguyệt Hình, Tứ Thời Cô Quả; xung: Ất Sửu, Đinh Hợi, Tân Sửu, Đinh Tỵ | đúng các sao và tuổi đó | **Khớp hoàn toàn** (4/4 tốt, 9/9 xấu, 4/4 tuổi; không thừa) |
| 2026-11-08 (F02) âm 30/9, Bính Tuất, Trực Bế | tốt: Thiên Đức, Nguyệt Đức, Thiên Mã, Mãn Đức Tinh; xấu: Thổ Phủ, Bạch Hổ, Tội Chỉ, Tam Tang, Ly Sàng, Quỷ Khốc; xung: Mậu Thìn, Nhâm Thìn, Nhâm Ngọ, Nhâm Tý | như trang | **Khớp hoàn toàn** |
| 2026-11-10 (F03) âm 2/10, Mậu Tý, Trực Trừ | tốt: Thiên Mã, U Vi Tinh, Yếu Yên; xấu: Bạch Hổ, Nguyệt Kiến Chuyển Sát, Phủ Đầu Sát; xung: Bính Ngọ, Giáp Ngọ | như trang | **Khớp hoàn toàn** |

Lưu ý thận trọng: (1) F01–F03 là cùng một trang (lichvn.net), có lẽ dùng cùng dòng sách TVTP như
N01/N02 — khớp chứng tỏ cài đúng **dòng sách đó**, không chứng tỏ dòng sách đó "đúng" so với dòng khác.
(2) Khi tra F01 bằng công cụ, bản trang tải về chỉ hiện U Vi Tinh, Đại Hồng Sa (tốt) và không hiện
Tam Nương; danh sách đầy đủ ở hàng 1 lấy từ đề bài của điều phối viên. Sát Cống, Thiên Ân, Tam Nương
đều được crate suy ra từ quy tắc có nguồn (N03/N04, N03/N17/N18, N05/N10/N11), không phải thêm để khớp.
(3) Các sao không thuộc "sao trong ngày" (giờ hoàng đạo, sao năm) không được đối chiếu.

## 6. Những gì chưa cài (không có đủ nguồn trong đợt này)

Có tên trong đề bài nhưng **không cài** vì chưa tìm được bảng quy tắc kiểm chứng được:
- **Thiên Phúc** (N01 có bảng nhưng nhiều ô trống, ô ghi hai can không rõ nghĩa) — tạm hoãn.
- **Tai Sát, Kim Thần Thất Sát, Thiên Cẩu, Tứ Ly, Tứ Tuyệt, Hoang Vu Tứ Quý, Ly Sào, Cửu Thổ Quỷ,
  Hỏa Tinh, Thiên Y, Quý Nhân, Ngũ Hợp** — có nguồn một phần (N03, N10, N11) nhưng cần thêm trường
  (năm, tiết khí) hoặc chưa đủ 12 tháng; để bản sau.
- **Nhị thập bát tú tốt/xấu** — không dùng chấm điểm (xem SCORING.md).

## 7. Bảng quy tắc chính (sinh từ mã)

Cột "Định nghĩa": "Thân" = ngày có chi Thân (mọi can); "can Đinh" = ngày có can Đinh (mọi chi);
"Quý Mùi" = đúng ngày can chi đó. Cột việc dùng mã việc (`activities()`): cuoi_hoi, xay_dung,
khai_truong, an_tang, te_tu, dong_tho, xuat_hanh, giao_dich, cau_tai, to_tung, lam_phuc, nhap_trach,
khai_nghiep, cau_tu, nhap_hoc, cau_suc_khoe. "Kỵ chính" = phạm thì `allowed = false` (N3). Các gán
"kỵ chính" và việc cụ thể là **diễn giải của dự án** từ mô tả tác dụng trong nguồn (N02, N07 mục sao
kỵ theo từng việc) — xem SCORING.md.

| Mã | Sao | Loại | Trọng số | Định nghĩa (tháng âm lịch) | Tốt cho | Kỵ | Kỵ chính (chặn chọn ngày) | Nguồn | Số dòng nguồn độc lập | Mâu thuẫn / ghi chú |
|---|---|---|---|---|---|---|---|---|---|---|
| S01 | Tam Nương | xấu | 2 | ngày âm [3, 7, 13, 18, 22, 27] mọi tháng | — | cuoi_hoi, khai_truong, giao_dich, xay_dung, xuat_hanh | cuoi_hoi, khai_truong, giao_dich | N05, N06, N10, N11, N12 | 3 (PKB, LHĐP, N11) | — |
| S02 | Nguyệt Kỵ | xấu | 2 | ngày âm [5, 14, 23] mọi tháng | — | mọi việc | xuat_hanh, giao_dich, cau_tai, khai_truong | N05, N06, N10, N11, N12 | 3 (PKB, LHĐP, N11) | Mức "đại kỵ" (tháng 1,4,7,10 → ngày 5; 2,5,8,11 → 14; 3,6,9,12 → 23) theo N10/N12 chỉ dùng để ghi chú, không đổi tập ngày. |
| S03 | Dương Công Kỵ Nhật | xấu | 3 | 13/1, 11/2, 9/3, 7/4, 5/5, 3/6, 1/7, 29/7, 27/8, 25/9, 23/10, 21/11, 19/12 | — | mọi việc | mọi việc | N10, N11, N12, N13, N14, N15, N16 | ≥3 (LHĐP, Trung Hoa N13/N16, N11) | **C01**: tháng 7 — "1/7" (N11, N13, N16) vs "8/7" (N10, N12, N14, N15); tháng 2 — "11/2" (N10, N11, N12, N13, N16) vs "12/2" (N14, N15). Chọn **11/2** (đa số rõ). Chọn **1/7** dù số trang ít hơn (3 vs 4): bên "1/7" gồm ít nhất 2 dòng độc lập (N11; gốc Trung Hoa 杨公十三忌 N13/N16: …六月初三, 七月初一, 七月廿九 — mỗi tháng lùi 2 ngày), bên "8/7" là 4 trang tiếng Việt không ghi sách (có thể chép nhau) và phá chuỗi lùi 2 ngày. **Lựa chọn có tranh cãi; cần người duyệt xác nhận.** |
| S04 | Thọ Tử | xấu | 1 | Giêng: Bính Tuất; Hai: Nhâm Thìn; Ba: Tân Hợi; Tư: Đinh Tỵ; Năm: Mậu Tý; Sáu: Bính Ngọ; Bảy: Ất Sửu; Tám: Quý Mùi; Chín: Giáp Dần; Mười: Mậu Thân; M.Một: Tân Mão; Chạp: Tân Dậu | — | mọi việc | mọi việc | N10, N12, N26 | 2 (LHĐP, N12); N26 xác minh 3 ngày năm 2017 khớp (Nhâm Thìn, Bính Ngọ, Tân Mão) | Bản can chi luôn là tập con của Thụ Tử theo chi (B14) — kiểm thử `special_lunar_days`. |
| G01 | Thiên Đức | tốt | 2 | Giêng: can Đinh; Hai: Thân; Ba: can Nhâm; Tư: can Tân; Năm: Hợi; Sáu: can Giáp; Bảy: can Quý; Tám: Dần; Chín: can Bính; Mười: can Ất; M.Một: Tỵ; Chạp: can Canh | mọi việc | — | — | N01, N19, N20 (N05/N06: bảng khác) | 2 (TVTP, tứ trụ N19/N20) | **C02**: N05/N06 cho bảng "Thiên đức" theo chi (Tỵ, Mùi, Dậu, Hợi, Sửu, Mão, lặp 2 lần) — đây đúng là vị trí **thần Thiên Đức/Kim Đường Hoàng Đạo** (thần số 5), không phải sao Thiên Đức; đã tính qua T06. Sao Thiên Đức dùng bảng N01 (khớp N19, N20). |
| G02 | Thiên Đức Hợp | tốt | 1 | Giêng: can Nhâm; Hai: Tỵ; Ba: can Đinh; Tư: can Bính; Năm: Dần; Sáu: can Kỷ; Bảy: can Mậu; Tám: Hợi; Chín: can Tân; Mười: can Canh; M.Một: Thân; Chạp: can Ất | mọi việc | — | — | N01, N19, N20 | 2 | **C18**: tháng 2: N01 ghi "Tý", N19 và N20 ghi **Tỵ** (天德合 卯月 = 巳, lục hợp của Thân). Chọn Tỵ (2 nguồn; khớp quy tắc hợp của Thiên Đức tháng 2 = Thân). |
| G03 | Nguyệt Đức | tốt | 2 | Giêng: can Bính; Hai: can Giáp; Ba: can Nhâm; Tư: can Canh; Năm: can Bính; Sáu: can Giáp; Bảy: can Nhâm; Tám: can Canh; Chín: can Bính; Mười: can Giáp; M.Một: can Nhâm; Chạp: can Canh | mọi việc | — | — | N01, N19, N20 (N05/N06: bảng khác) | 2 | **C03**: N05/N06 ghi "Nguyệt đức" theo chi (Hợi, Tuất, Dậu…). Chọn bảng can của N01 (khớp N19, N20). |
| G04 | Nguyệt Đức Hợp | tốt | 1 | Giêng: can Tân; Hai: can Kỷ; Ba: can Đinh; Tư: can Ất; Năm: can Tân; Sáu: can Kỷ; Bảy: can Đinh; Tám: can Ất; Chín: can Tân; Mười: can Kỷ; M.Một: can Đinh; Chạp: can Ất | mọi việc | to_tung | — | N01, N20 | 2 | — |
| G05 | Thiên Hỷ | tốt | 1 | Giêng: Tuất; Hai: Hợi; Ba: Tý; Tư: Sửu; Năm: Dần; Sáu: Mão; Bảy: Thìn; Tám: Tỵ; Chín: Ngọ; Mười: Mùi; M.Một: Thân; Chạp: Dậu | mọi việc, nhất là cuoi_hoi | — | — | N01, N05, N06, N10 | 3 (TVTP, PKB, LHĐP) | — |
| G06 | Thiên Phú | tốt | 1 | Giêng: Thìn; Hai: Tỵ; Ba: Ngọ; Tư: Mùi; Năm: Thân; Sáu: Dậu; Bảy: Tuất; Tám: Hợi; Chín: Tý; Mười: Sửu; M.Một: Dần; Chạp: Mão | mọi việc, nhất là xay_dung, khai_truong, an_tang, cau_tai | — | — | N01 | 1 | — |
| G07 | Thiên Quý | tốt | 1 | Giêng: can Giáp, can Ất; Hai: can Giáp, can Ất; Ba: can Giáp, can Ất; Tư: can Bính, can Đinh; Năm: can Bính, can Đinh; Sáu: can Bính, can Đinh; Bảy: can Canh, can Tân; Tám: can Canh, can Tân; Chín: can Canh, can Tân; Mười: can Nhâm, can Quý; M.Một: can Nhâm, can Quý; Chạp: can Nhâm, can Quý | mọi việc | — | — | N01 (N05/N06: bảng khác) | 1 | **C04**: N05/N06 ghi "Thiên quý" theo chi (Dần, Thân, Mão…), trùng bảng Yếu Yên (G35) của N01, mà N01 cũng chú "Yếu yên (Thiên quý)". Chọn: Thiên Quý = bảng can theo mùa (N01); bảng chi của N05 được cài dưới tên Yếu Yên. |
| G08 | Thiên Xá | tốt | 1 | Giêng: Mậu Dần; Hai: Mậu Dần; Ba: Mậu Dần; Tư: Giáp Ngọ; Năm: Giáp Ngọ; Sáu: Giáp Ngọ; Bảy: Mậu Thân; Tám: Mậu Thân; Chín: Mậu Thân; Mười: Giáp Tý; M.Một: Giáp Tý; Chạp: Giáp Tý | te_tu, to_tung | dong_tho | — | N01, N21 | 2 | **C05**: N01 để trống ô tháng 5 và 11; N21 ghi theo mùa đủ (Xuân Mậu Dần, Hạ Giáp Ngọ, Thu Mậu Thân, Đông Giáp Tý) nhưng định mùa theo tiết khí. Mã cài **theo mùa tháng âm, đủ 12 tháng**. Chưa chắc ở hai điểm: ô trống của N01, và tháng âm vs tiết khí. |
| G09 | Sinh Khí | tốt | 1 | Giêng: Tý; Hai: Sửu; Ba: Dần; Tư: Mão; Năm: Thìn; Sáu: Tỵ; Bảy: Ngọ; Tám: Mùi; Chín: Thân; Mười: Dậu; M.Một: Tuất; Chạp: Hợi | mọi việc, nhất là xay_dung, dong_tho | — | — | N01, N05, N06 | 2 (TVTP, PKB) | — |
| G10 | Thiên Thành | tốt | 1 | Giêng: Mùi; Hai: Dậu; Ba: Hợi; Tư: Sửu; Năm: Mão; Sáu: Tỵ; Bảy: Mùi; Tám: Dậu; Chín: Hợi; Mười: Sửu; M.Một: Mão; Chạp: Tỵ | mọi việc | — | — | N01, N05, N06 | 2 | **C06**: N05/N06 tháng 9 ghi "Mùi", N01 ghi "Hợi". Chọn Hợi: N01 khớp quy tắc Ngọc Đường hoàng đạo mà chính N01 ghi trùng (kiểm thử `stars_coinciding_with_than`), N05 tháng 9 phá chu kỳ 6 tháng — nhiều khả năng lỗi chép. |
| G11 | Thiên Quan | tốt | 1 | Giêng: Tuất; Hai: Tý; Ba: Dần; Tư: Thìn; Năm: Ngọ; Sáu: Thân; Bảy: Tuất; Tám: Tý; Chín: Dần; Mười: Thìn; M.Một: Ngọ; Chạp: Thân | mọi việc | — | — | N01, N05, N06 | 2 | — |
| G12 | Thiên Mã | tốt | 1 | Giêng: Ngọ; Hai: Thân; Ba: Tuất; Tư: Tý; Năm: Dần; Sáu: Thìn; Bảy: Ngọ; Tám: Thân; Chín: Tuất; Mười: Tý; M.Một: Dần; Chạp: Thìn | xuat_hanh, giao_dich, cau_tai | — | — | N01, N05/N06 ("Lộc mã") | 2 | N01 gắn nhãn "xấu" vì trùng Bạch Hổ nhưng mô tả tốt cho xuất hành, cầu tài. Chọn: xếp sao tốt (tác dụng theo việc), còn Bạch Hổ Hắc Đạo vẫn được tính riêng (T07). |
| G13 | Thiên Tài | tốt | 1 | Giêng: Thìn; Hai: Ngọ; Ba: Thân; Tư: Tuất; Năm: Tý; Sáu: Dần; Bảy: Thìn; Tám: Ngọ; Chín: Thân; Mười: Tuất; M.Một: Tý; Chạp: Dần | cau_tai, khai_truong | — | — | N01 | 1 (trùng Kim Quỹ — kiểm thử) | — |
| G14 | Địa Tài | tốt | 1 | Giêng: Tỵ; Hai: Mùi; Ba: Dậu; Tư: Hợi; Năm: Sửu; Sáu: Mão; Bảy: Tỵ; Tám: Mùi; Chín: Dậu; Mười: Hợi; M.Một: Sửu; Chạp: Mão | cau_tai, khai_truong | — | — | N01 | 1 (trùng Kim Đường/Bảo Quang — kiểm thử) | — |
| G15 | Nguyệt Tài | tốt | 1 | Giêng: Ngọ; Hai: Tỵ; Ba: Tỵ; Tư: Mùi; Năm: Dậu; Sáu: Hợi; Bảy: Ngọ; Tám: Tỵ; Chín: Tỵ; Mười: Mùi; M.Một: Dậu; Chạp: Hợi | cau_tai, khai_truong, xuat_hanh, giao_dich, nhap_trach | — | — | N01 | 1 | — |
| G16 | Nguyệt Ân | tốt | 1 | Giêng: can Bính; Hai: can Đinh; Ba: can Canh; Tư: can Kỷ; Năm: can Mậu; Sáu: can Tân; Bảy: can Nhâm; Tám: can Quý; Chín: can Canh; Mười: can Ất; M.Một: can Giáp; Chạp: can Tân | mọi việc | — | — | N01 | 1 | — |
| G17 | Nguyệt Không | tốt | 1 | Giêng: can Nhâm; Hai: can Canh; Ba: can Bính; Tư: can Giáp; Năm: can Nhâm; Sáu: can Canh; Bảy: can Bính; Tám: can Giáp; Chín: can Nhâm; Mười: can Canh; M.Một: can Bính; Chạp: can Giáp | xay_dung | — | — | N01 | 1 | — |
| G18 | Minh Tinh | tốt | 1 | Giêng: Thân; Hai: Tuất; Ba: Tý; Tư: Dần; Năm: Thìn; Sáu: Ngọ; Bảy: Thân; Tám: Tuất; Chín: Tý; Mười: Dần; M.Một: Thìn; Chạp: Ngọ | to_tung, an_tang | — | — | N01 | 1 (trùng Thiên Lao — kiểm thử) | — |
| G19 | Thánh Tâm | tốt | 1 | Giêng: Hợi; Hai: Tỵ; Ba: Tý; Tư: Ngọ; Năm: Sửu; Sáu: Mùi; Bảy: Dần; Tám: Thân; Chín: Mão; Mười: Dậu; M.Một: Thìn; Chạp: Tuất | mọi việc, nhất là te_tu, lam_phuc | — | — | N01 | 1 | — |
| G20 | Ngũ Phú | tốt | 1 | Giêng: Hợi; Hai: Dần; Ba: Tỵ; Tư: Thân; Năm: Hợi; Sáu: Dần; Bảy: Tỵ; Tám: Thân; Chín: Hợi; Mười: Dần; M.Một: Tỵ; Chạp: Thân | mọi việc | — | — | N01 | 1 | — |
| G21 | Lộc Khố | tốt | 1 | Giêng: Thìn; Hai: Tỵ; Ba: Ngọ; Tư: Mùi; Năm: Thân; Sáu: Dậu; Bảy: Tuất; Tám: Hợi; Chín: Tý; Mười: Sửu; M.Một: Dần; Chạp: Mão | cau_tai, giao_dich, khai_truong | — | — | N01 | 1 | — |
| G22 | Phúc Sinh | tốt | 1 | Giêng: Dậu; Hai: Mão; Ba: Tuất; Tư: Thìn; Năm: Hợi; Sáu: Tỵ; Bảy: Tý; Tám: Ngọ; Chín: Sửu; Mười: Mùi; M.Một: Dần; Chạp: Thân | mọi việc | — | — | N01, N05, N06 | 2 | — |
| G23 | Cát Khánh | tốt | 1 | Giêng: Dậu; Hai: Dần; Ba: Hợi; Tư: Thìn; Năm: Sửu; Sáu: Ngọ; Bảy: Mão; Tám: Thân; Chín: Tỵ; Mười: Tuất; M.Một: Mùi; Chạp: Tý | mọi việc | — | — | N01 | 1 | — |
| G24 | Âm Đức | tốt | 1 | Giêng: Dậu; Hai: Mùi; Ba: Tỵ; Tư: Mão; Năm: Sửu; Sáu: Hợi; Bảy: Dậu; Tám: Mùi; Chín: Tỵ; Mười: Mão; M.Một: Sửu; Chạp: Hợi | mọi việc | — | — | N01 | 1 | — |
| G25 | U Vi Tinh | tốt | 1 | Giêng: Hợi; Hai: Thìn; Ba: Sửu; Tư: Ngọ; Năm: Mão; Sáu: Thân; Bảy: Tỵ; Tám: Tuất; Chín: Mùi; Mười: Tý; M.Một: Dậu; Chạp: Dần | mọi việc | — | — | N01; F01, F03 khớp | 1 | — |
| G26 | Mãn Đức Tinh | tốt | 1 | Giêng: Dần; Hai: Mùi; Ba: Thìn; Tư: Dậu; Năm: Ngọ; Sáu: Hợi; Bảy: Thân; Tám: Sửu; Chín: Tuất; Mười: Mão; M.Một: Tý; Chạp: Tỵ | mọi việc | — | — | N01; F02 khớp | 1 | — |
| G27 | Kính Tâm | tốt | 1 | Giêng: Mùi; Hai: Sửu; Ba: Thân; Tư: Dần; Năm: Dậu; Sáu: Mão; Bảy: Tuất; Tám: Thìn; Chín: Hợi; Mười: Tỵ; M.Một: Tý; Chạp: Ngọ | an_tang | — | — | N01 | 1 | — |
| G28 | Tuế Hợp | tốt | 1 | Giêng: Sửu; Hai: Tý; Ba: Hợi; Tư: Tuất; Năm: Dậu; Sáu: Thân; Bảy: Mùi; Tám: Ngọ; Chín: Tỵ; Mười: Thìn; M.Một: Mão; Chạp: Dần | mọi việc | — | — | N01 | 1 | — |
| G29 | Nguyệt Giải | tốt | 1 | Giêng: Thân; Hai: Thân; Ba: Dậu; Tư: Dậu; Năm: Tuất; Sáu: Tuất; Bảy: Hợi; Tám: Hợi; Chín: Ngọ; Mười: Ngọ; M.Một: Mùi; Chạp: Mùi | mọi việc | — | — | N01 | 1 | — |
| G30 | Hoạt Điệu | tốt | 1 | Giêng: Tỵ; Hai: Tuất; Ba: Mùi; Tư: Tý; Năm: Dậu; Sáu: Dần; Bảy: Hợi; Tám: Thìn; Chín: Sửu; Mười: Ngọ; M.Một: Mão; Chạp: Thân | mọi việc | — | — | N01 | 1 | Luật kèm theo (N01): gặp Thụ Tử thì mất tác dụng — đã cài. |
| G31 | Giải Thần | tốt | 1 | Giêng: Thân; Hai: Thân; Ba: Tuất; Tư: Tuất; Năm: Tý; Sáu: Tý; Bảy: Dần; Tám: Dần; Chín: Thìn; Mười: Thìn; M.Một: Ngọ; Chạp: Ngọ | te_tu, to_tung | — | — | N01, N05, N06 | 2 | — |
| G32 | Phổ Hộ | tốt | 1 | Giêng: Thân; Hai: Dần; Ba: Dậu; Tư: Mão; Năm: Tuất; Sáu: Thìn; Bảy: Hợi; Tám: Tỵ; Chín: Tý; Mười: Ngọ; M.Một: Sửu; Chạp: Mùi | mọi việc, nhất là lam_phuc, cuoi_hoi, xuat_hanh | — | — | N01 | 1 | — |
| G33 | Ích Hậu | tốt | 1 | Giêng: Tý; Hai: Ngọ; Ba: Sửu; Tư: Mùi; Năm: Dần; Sáu: Thân; Bảy: Mão; Tám: Dậu; Chín: Thìn; Mười: Tuất; M.Một: Tỵ; Chạp: Hợi | mọi việc, nhất là cuoi_hoi | — | — | N01 | 1 | — |
| G34 | Tục Thế | tốt | 1 | Giêng: Sửu; Hai: Mùi; Ba: Dần; Tư: Thân; Năm: Mão; Sáu: Dậu; Bảy: Thìn; Tám: Tuất; Chín: Tỵ; Mười: Hợi; M.Một: Ngọ; Chạp: Tý | mọi việc, nhất là cuoi_hoi | — | — | N01 | 1 | — |
| G35 | Yếu Yên | tốt | 1 | Giêng: Dần; Hai: Thân; Ba: Mão; Tư: Dậu; Năm: Thìn; Sáu: Tuất; Bảy: Tỵ; Tám: Hợi; Chín: Ngọ; Mười: Tý; M.Một: Mùi; Chạp: Sửu | cuoi_hoi | — | — | N01, N05/N06 (dưới tên "Thiên quý") | 2 | Xem C04. |
| G36 | Dịch Mã | tốt | 1 | Giêng: Thân; Hai: Tỵ; Ba: Dần; Tư: Hợi; Năm: Thân; Sáu: Tỵ; Bảy: Dần; Tám: Hợi; Chín: Thân; Mười: Tỵ; M.Một: Dần; Chạp: Hợi | xuat_hanh, cau_suc_khoe | — | — | N01 | 1 | — |
| G37 | Tam Hợp | tốt | 1 | Giêng: Ngọ, Tuất; Hai: Mùi, Hợi; Ba: Tý, Thân; Tư: Sửu, Dậu; Năm: Dần, Tuất; Sáu: Mão, Hợi; Bảy: Tý, Thìn; Tám: Sửu, Tỵ; Chín: Dần, Ngọ; Mười: Mão, Mùi; M.Một: Thìn, Thân; Chạp: Tỵ, Dậu | mọi việc | — | — | N01, N05/N06 (một chi) | 2 | N05/N06 chỉ ghi chi thứ nhất (Ngọ, Mùi, Thân…); N01 ghi hai chi. Chọn N01 (đủ hơn, cùng quy tắc tam hợp). |
| G38 | Lục Hợp | tốt | 1 | Giêng: Hợi; Hai: Tuất; Ba: Dậu; Tư: Thân; Năm: Mùi; Sáu: Ngọ; Bảy: Tỵ; Tám: Thìn; Chín: Mão; Mười: Dần; M.Một: Sửu; Chạp: Tý | mọi việc | — | — | N01 | 1 | — |
| G39 | Mẫu Thương | tốt | 1 | Giêng: Tý, Hợi; Hai: Tý, Hợi; Ba: Tý, Hợi; Tư: Dần, Mão; Năm: Dần, Mão; Sáu: Dần, Mão; Bảy: Sửu, Thìn; Tám: Sửu, Thìn; Chín: Sửu, Thìn; Mười: Thân, Dậu; M.Một: Thân, Dậu; Chạp: Thân, Dậu | cau_tai, khai_truong | — | — | N01 | 1 | — |
| G40 | Phúc Hậu | tốt | 1 | Giêng: Dần; Hai: Dần; Ba: Dần; Tư: Tỵ; Năm: Tỵ; Sáu: Tỵ; Bảy: Thân; Tám: Thân; Chín: Thân; Mười: Hợi; M.Một: Hợi; Chạp: Hợi | cau_tai, khai_truong | — | — | N01 | 1 | — |
| G41 | Đại Hồng Sa | tốt | 1 | Giêng: Tý, Sửu; Hai: Tý, Sửu; Ba: Tý, Sửu; Tư: Thìn, Tỵ; Năm: Thìn, Tỵ; Sáu: Thìn, Tỵ; Bảy: Ngọ, Mùi; Tám: Ngọ, Mùi; Chín: Ngọ, Mùi; Mười: Thân, Tuất; M.Một: Thân, Tuất; Chạp: Thân, Tuất | mọi việc | — | — | N01; F01 khớp | 1 | — |
| G42 | Dân Nhật | tốt | 1 | Giêng: Ngọ; Hai: Ngọ; Ba: Ngọ; Tư: Dậu; Năm: Dậu; Sáu: Dậu; Bảy: Tý; Tám: Tý; Chín: Tý; Mười: Mão; M.Một: Mão; Chạp: Mão | dong_tho, xay_dung | — | — | N01 | 1 | — |
| G43 | Hoàng Ân | tốt | 1 | Giêng: Tuất; Hai: Sửu; Ba: Dần; Tư: Tỵ; Năm: Dậu; Sáu: Mão; Bảy: Tý; Tám: Ngọ; Chín: Hợi; Mười: Thìn; M.Một: Thân; Chạp: Mùi | xay_dung, khai_truong | — | — | N01 (không ghi tác dụng) | 1 | N01 không ghi tốt cho việc gì; N05/N06 ghi bảng **cùng giá trị** dưới tên "Thiên ân" (tốt cho làm nhà, khai trương). **C07**: tên "Thiên Ân" ở N05 ≠ Thiên Ân 15 ngày ở N03/N17/N18. Chọn: bảng này mang tên Hoàng Ân (N01), tác dụng lấy từ N05 (xây dựng, khai trương). |
| G44 | Thiên Ân | tốt | 1 | mọi tháng: Giáp Tý, Ất Sửu, Bính Dần, Đinh Mão, Mậu Thìn, Kỷ Mão, Canh Thìn, Tân Tỵ, Nhâm Ngọ, Quý Mùi, Kỷ Dậu, Canh Tuất, Tân Hợi, Nhâm Tý, Quý Sửu | mọi việc | — | — | N03, N17, N18; F01 khớp | 3 (TVTP, Nhật 暦林問答集, Trung Hoa) | **C08**: N03 ghi 14 ngày (có Nhâm Thân, thiếu Kỷ Dậu và Nhâm Tý); N17, N18 ghi 15 ngày (Kỷ Dậu, Nhâm Tý, không Nhâm Thân). Chọn 15 ngày (2 dòng độc lập, có sách in cổ; N03 lệch 1 chỗ có thể là lỗi chép). |
| G45 | Sát Cống | tốt | 2 | Giêng: Đinh Mão, Bính Tý, Ất Dậu, Giáp Ngọ, Quý Mão, Nhâm Tý, Tân Dậu; Hai: Bính Dần, Ất Hợi, Giáp Thân, Quý Tỵ, Nhâm Dần, Tân Hợi, Canh Thân; Ba: Ất Sửu, Giáp Tuất, Quý Mùi, Nhâm Thìn, Tân Sửu, Canh Tuất, Kỷ Mùi; Tư: Đinh Mão, Bính Tý, Ất Dậu, Giáp Ngọ, Quý Mão, Nhâm Tý, Tân Dậu; Năm: Bính Dần, Ất Hợi, Giáp Thân, Quý Tỵ, Nhâm Dần, Tân Hợi, Canh Thân; Sáu: Ất Sửu, Giáp Tuất, Quý Mùi, Nhâm Thìn, Tân Sửu, Canh Tuất, Kỷ Mùi; Bảy: Đinh Mão, Bính Tý, Ất Dậu, Giáp Ngọ, Quý Mão, Nhâm Tý, Tân Dậu; Tám: Bính Dần, Ất Hợi, Giáp Thân, Quý Tỵ, Nhâm Dần, Tân Hợi, Canh Thân; Chín: Ất Sửu, Giáp Tuất, Quý Mùi, Nhâm Thìn, Tân Sửu, Canh Tuất, Kỷ Mùi; Mười: Đinh Mão, Bính Tý, Ất Dậu, Giáp Ngọ, Quý Mão, Nhâm Tý, Tân Dậu; M.Một: Bính Dần, Ất Hợi, Giáp Thân, Quý Tỵ, Nhâm Dần, Tân Hợi, Canh Thân; Chạp: Ất Sửu, Giáp Tuất, Quý Mùi, Nhâm Thìn, Tân Sửu, Canh Tuất, Kỷ Mùi | mọi việc | — | — | N03, N04; F01 khớp | 2 (TVTP, N04) | **C09**: nhóm Mạnh: N03 "Nhâm Tý", N04 "Nhâm Tỵ" (không phải can chi hợp lệ). Chọn Nhâm Tý. Kiểm thử `sat_cong_truc_tinh_structure`: trong mỗi nhóm các ngày cách nhau đúng 9 vị trí trong vòng 60. |
| G46 | Trực Tinh | tốt | 2 | Giêng: Mậu Thìn, Đinh Sửu, Bính Tuất, Ất Mùi, Giáp Thìn, Quý Sửu, Nhâm Tuất; Hai: Đinh Mão, Bính Tý, Ất Dậu, Giáp Ngọ, Quý Mão, Nhâm Tý, Tân Dậu; Ba: Bính Dần, Ất Hợi, Giáp Thân, Quý Tỵ, Nhâm Dần, Tân Hợi, Canh Thân; Tư: Mậu Thìn, Đinh Sửu, Bính Tuất, Ất Mùi, Giáp Thìn, Quý Sửu, Nhâm Tuất; Năm: Đinh Mão, Bính Tý, Ất Dậu, Giáp Ngọ, Quý Mão, Nhâm Tý, Tân Dậu; Sáu: Bính Dần, Ất Hợi, Giáp Thân, Quý Tỵ, Nhâm Dần, Tân Hợi, Canh Thân; Bảy: Mậu Thìn, Đinh Sửu, Bính Tuất, Ất Mùi, Giáp Thìn, Quý Sửu, Nhâm Tuất; Tám: Đinh Mão, Bính Tý, Ất Dậu, Giáp Ngọ, Quý Mão, Nhâm Tý, Tân Dậu; Chín: Bính Dần, Ất Hợi, Giáp Thân, Quý Tỵ, Nhâm Dần, Tân Hợi, Canh Thân; Mười: Mậu Thìn, Đinh Sửu, Bính Tuất, Ất Mùi, Giáp Thìn, Quý Sửu, Nhâm Tuất; M.Một: Đinh Mão, Bính Tý, Ất Dậu, Giáp Ngọ, Quý Mão, Nhâm Tý, Tân Dậu; Chạp: Bính Dần, Ất Hợi, Giáp Thân, Quý Tỵ, Nhâm Dần, Tân Hợi, Canh Thân | mọi việc | — | — | N03, N04 | 2 | N04 nhóm Trọng ghi "Nhâm Tý" khớp N03; nhóm Quý N04 ghi "Tân Mão" (phá cấu trúc cách 9) và "Canh Tân" (không phải can chi) — N03 ghi "Tân Hợi", "Canh Thân". Chọn N03 (thỏa cấu trúc cách 9 và quy tắc Trực Tinh nhóm g = Sát Cống nhóm g−1). |
| G47 | Nhân Chuyên | tốt | 2 | Giêng: Tân Mùi, Canh Thìn, Kỷ Sửu, Mậu Tuất, Đinh Mùi, Bính Thìn; Hai: Canh Ngọ, Kỷ Mão, Mậu Tý, Đinh Dậu, Bính Ngọ, Ất Mão; Ba: Kỷ Tỵ, Mậu Dần, Đinh Hợi, Bính Thân, Ất Tỵ, Giáp Dần, Quý Hợi; Tư: Tân Mùi, Canh Thìn, Kỷ Sửu, Mậu Tuất, Đinh Mùi, Bính Thìn; Năm: Canh Ngọ, Kỷ Mão, Mậu Tý, Đinh Dậu, Bính Ngọ, Ất Mão; Sáu: Kỷ Tỵ, Mậu Dần, Đinh Hợi, Bính Thân, Ất Tỵ, Giáp Dần, Quý Hợi; Bảy: Tân Mùi, Canh Thìn, Kỷ Sửu, Mậu Tuất, Đinh Mùi, Bính Thìn; Tám: Canh Ngọ, Kỷ Mão, Mậu Tý, Đinh Dậu, Bính Ngọ, Ất Mão; Chín: Kỷ Tỵ, Mậu Dần, Đinh Hợi, Bính Thân, Ất Tỵ, Giáp Dần, Quý Hợi; Mười: Tân Mùi, Canh Thìn, Kỷ Sửu, Mậu Tuất, Đinh Mùi, Bính Thìn; M.Một: Canh Ngọ, Kỷ Mão, Mậu Tý, Đinh Dậu, Bính Ngọ, Ất Mão; Chạp: Kỷ Tỵ, Mậu Dần, Đinh Hợi, Bính Thân, Ất Tỵ, Giáp Dần, Quý Hợi | mọi việc | — | — | N03, N04 | 2 | Nhóm Trọng: N03 "Mậu Tý", N04 "Nhâm Tý"; nhóm Quý: N03 "Đinh Hợi", N04 "Kỷ Hợi". Chọn N03 (thỏa cấu trúc cách 9; bản N04 phá cấu trúc). |
| G48 | Thiên Thụy | tốt | 1 | mọi tháng: Mậu Dần, Kỷ Mão, Tân Tỵ, Canh Dần, Nhâm Tý | mọi việc | — | — | N03 | 1 | — |
| G49 | Quan Nhật | tốt | 1 | Giêng: —; Hai: Mão; Ba: —; Tư: —; Năm: Ngọ; Sáu: —; Bảy: —; Tám: Dậu; Chín: —; Mười: —; M.Một: Tý; Chạp: — | khai_nghiep | — | — | N01 | 1 | N01 chỉ ghi 4 tháng (2, 5, 8, 11); các tháng khác để trống → không có sao. |
| B01 | Thiên Cương | xấu | 3 | Giêng: Tỵ; Hai: Tý; Ba: Mùi; Tư: Dần; Năm: Dậu; Sáu: Thìn; Bảy: Hợi; Tám: Ngọ; Chín: Sửu; Mười: Thân; M.Một: Mão; Chạp: Tuất | — | mọi việc | mọi việc | N02, N05, N06, N07, N08 | 2 (TVTP, PKB) | — |
| B02 | Thiên Lại | xấu | 1 | Giêng: Dậu; Hai: Ngọ; Ba: Mão; Tư: Tý; Năm: Dậu; Sáu: Ngọ; Bảy: Mão; Tám: Tý; Chín: Dậu; Mười: Ngọ; M.Một: Mão; Chạp: Tý | — | mọi việc | — | N02 | 1 | — |
| B03 | Thiên Ngục | xấu | 1 | Giêng: Tý; Hai: Mão; Ba: Ngọ; Tư: Dậu; Năm: Tý; Sáu: Mão; Bảy: Ngọ; Tám: Dậu; Chín: Tý; Mười: Mão; M.Một: Ngọ; Chạp: Dậu | — | mọi việc | — | N02 | 1 | N02 ghi chung dòng "Thiên Ngục; Thiên Hỏa"; tách thành hai sao cùng vị trí, tác dụng khác nhau. |
| B04 | Thiên Hỏa | xấu | 1 | Giêng: Tý; Hai: Mão; Ba: Ngọ; Tư: Dậu; Năm: Tý; Sáu: Mão; Bảy: Ngọ; Tám: Dậu; Chín: Tý; Mười: Mão; M.Một: Ngọ; Chạp: Dậu | — | xay_dung | xay_dung | N02, N05, N06, N07 | 2 | — |
| B05 | Tiểu Hồng Sa | xấu | 1 | Giêng: Tỵ; Hai: Dậu; Ba: Sửu; Tư: Tỵ; Năm: Dậu; Sáu: Sửu; Bảy: Tỵ; Tám: Dậu; Chín: Sửu; Mười: Tỵ; M.Một: Dậu; Chạp: Sửu | — | mọi việc | — | N02 | 1 | — |
| B06 | Đại Hao | xấu | 3 | Giêng: Ngọ; Hai: Mùi; Ba: Thân; Tư: Dậu; Năm: Tuất; Sáu: Hợi; Bảy: Tý; Tám: Sửu; Chín: Dần; Mười: Mão; M.Một: Thìn; Chạp: Tỵ | — | mọi việc | mọi việc | N02, N05, N06, N07 | 2 | — |
| B07 | Tiểu Hao | xấu | 1 | Giêng: Tỵ; Hai: Ngọ; Ba: Mùi; Tư: Thân; Năm: Dậu; Sáu: Tuất; Bảy: Hợi; Tám: Tý; Chín: Sửu; Mười: Dần; M.Một: Mão; Chạp: Thìn | — | cau_tai, giao_dich, khai_truong | — | N02, N05, N06, N07 | 2 | — |
| B08 | Nguyệt Phá | xấu | 1 | Giêng: Thân; Hai: Tuất; Ba: Tuất; Tư: Hợi; Năm: Sửu; Sáu: Sửu; Bảy: Dần; Tám: Thìn; Chín: Thìn; Mười: Tỵ; M.Một: Mùi; Chạp: Mùi | — | xay_dung, khai_truong, cau_tai | cau_tai, khai_truong | N02, N05, N06, N07, N08 | 2 | **C10**: N02 ghi tháng 2, 5, 8, 11 là "Dậu (Tuất)", "Tý (Sửu)", "Mão (Thìn)", "Ngọ (Mùi)"; N05–N08 ghi Tuất, Sửu, Thìn, Mùi. Chọn bản **Tuất/Sửu/Thìn/Mùi** (được cả hai dòng TVTP-trong-ngoặc và PKB ghi). Giá trị kia chỉ được N02 ghi là lựa chọn chính. |
| B09 | Kiếp Sát | xấu | 3 | Giêng: Hợi; Hai: Thân; Ba: Tỵ; Tư: Dần; Năm: Hợi; Sáu: Thân; Bảy: Tỵ; Tám: Dần; Chín: Hợi; Mười: Thân; M.Một: Tỵ; Chạp: Dần | — | xuat_hanh, cuoi_hoi, an_tang, xay_dung | xuat_hanh, cuoi_hoi, an_tang, xay_dung | N02 | 1 | — |
| B10 | Địa Phá | xấu | 1 | Giêng: Hợi; Hai: Tý; Ba: Sửu; Tư: Dần; Năm: Mão; Sáu: Thìn; Bảy: Tỵ; Tám: Ngọ; Chín: Mùi; Mười: Thân; M.Một: Dậu; Chạp: Tuất | — | xay_dung, dong_tho | dong_tho | N02; F01 khớp | 1 | — |
| B11 | Thổ Phủ | xấu | 1 | Giêng: Dần; Hai: Mão; Ba: Thìn; Tư: Tỵ; Năm: Ngọ; Sáu: Mùi; Bảy: Thân; Tám: Dậu; Chín: Tuất; Mười: Hợi; M.Một: Tý; Chạp: Sửu | — | xay_dung, dong_tho | dong_tho | N02; F02 khớp | 1 | — |
| B12 | Thổ Ôn | xấu | 1 | Giêng: Thìn; Hai: Tỵ; Ba: Ngọ; Tư: Mùi; Năm: Thân; Sáu: Dậu; Bảy: Tuất; Tám: Hợi; Chín: Tý; Mười: Sửu; M.Một: Dần; Chạp: Mão | — | xay_dung, dong_tho, te_tu | dong_tho | N02 | 1 | — |
| B13 | Thiên Ôn | xấu | 1 | Giêng: Mùi; Hai: Tuất; Ba: Thìn; Tư: Dần; Năm: Ngọ; Sáu: Tý; Bảy: Dậu; Tám: Thân; Chín: Tỵ; Mười: Hợi; M.Một: Tý; Chạp: Mão | — | xay_dung, khai_truong, cau_tai | xay_dung, khai_truong, cau_tai | N02 | 1 | Tháng 11 ghi "Tý" trùng tháng 6 — giữ nguyên như nguồn (không tự sửa). |
| B14 | Thụ Tử | xấu | 3 | Giêng: Tuất; Hai: Thìn; Ba: Hợi; Tư: Tỵ; Năm: Tý; Sáu: Ngọ; Bảy: Sửu; Tám: Mùi; Chín: Dần; Mười: Thân; M.Một: Mão; Chạp: Dậu | — | mọi việc | mọi việc | N02, N05, N06, N07, N08 | 2 | Tên "Thụ Tử" (N02, N05) / "Thọ Tử" dùng lẫn; bản can chi ở S04. |
| B15 | Hoang Vu | xấu | 1 | Giêng: Sửu, Tỵ, Dậu; Hai: Sửu, Tỵ, Dậu; Ba: Sửu, Tỵ, Dậu; Tư: Tý, Thìn, Thân; Năm: Tý, Thìn, Thân; Sáu: Tý, Thìn, Thân; Bảy: Mão, Mùi, Hợi; Tám: Mão, Mùi, Hợi; Chín: Mão, Mùi, Hợi; Mười: Dần, Ngọ, Tuất; M.Một: Dần, Ngọ, Tuất; Chạp: Dần, Ngọ, Tuất | — | mọi việc | — | N02; F01 khớp | 1 | **C11**: N10 và N26 ghi "Hoang vu tứ quý" khác (Xuân Thân, Hạ Dần, Thu Thìn, Đông Tỵ). Đó là một sao khác tên gần giống (tứ quý); không gộp. Chưa cài Hoang Vu Tứ Quý (ứng viên bản sau: N10 + N26). |
| B16 | Thiên Tặc | xấu | 1 | Giêng: Thìn; Hai: Dậu; Ba: Dần; Tư: Mùi; Năm: Tý; Sáu: Tỵ; Bảy: Tuất; Tám: Mão; Chín: Thân; Mười: Sửu; M.Một: Ngọ; Chạp: Hợi | — | xay_dung, dong_tho, nhap_trach, khai_truong, cau_tai | xay_dung, khai_truong, cau_tai | N02 | 1 | — |
| B17 | Địa Tặc | xấu | 1 | Giêng: Sửu; Hai: Tý; Ba: Hợi; Tư: Tuất; Năm: Dậu; Sáu: Thân; Bảy: Mùi; Tám: Ngọ; Chín: Tỵ; Mười: Thìn; M.Một: Mão; Chạp: Dần | — | xay_dung, an_tang, dong_tho, xuat_hanh | dong_tho | N02 | 1 | — |
| B18 | Hỏa Tai | xấu | 1 | Giêng: Sửu; Hai: Mùi; Ba: Dần; Tư: Thân; Năm: Mão; Sáu: Dậu; Bảy: Thìn; Tám: Tuất; Chín: Tỵ; Mười: Hợi; M.Một: Ngọ; Chạp: Tý | — | xay_dung | — | N02, N05, N06, N07 | 2 | **C12**: tháng 11–12: N02 ghi Ngọ, Tý; dòng PKB (N05–N07) ghi Tý, Ngọ. Mỗi bên 1 dòng (hòa). Chọn **N02 (Ngọ, Tý)**, phân xử bằng cấu trúc bảng: tháng lẻ Sửu→Dần→Mão→Thìn→Tỵ→**Ngọ**, tháng chẵn Mùi→Thân→Dậu→Tuất→Hợi→**Tý**; bản PKB đảo hai ô cuối (nhiều khả năng lỗi chép). |
| B19 | Nguyệt Hỏa (Độc Hỏa) | xấu | 1 | Giêng: Tỵ; Hai: Thìn; Ba: Mão; Tư: Dần; Năm: Sửu; Sáu: Tý; Bảy: Hợi; Tám: Tuất; Chín: Dậu; Mười: Thân; M.Một: Mùi; Chạp: Ngọ | — | xay_dung | — | N02 | 1 | — |
| B20 | Nguyệt Yếm Đại Họa | xấu | 1 | Giêng: Tuất; Hai: Dậu; Ba: Thân; Tư: Mùi; Năm: Ngọ; Sáu: Tỵ; Bảy: Thìn; Tám: Mão; Chín: Dần; Mười: Sửu; M.Một: Tý; Chạp: Hợi | — | xuat_hanh, cuoi_hoi | — | N02 | 1 | — |
| B21 | Nguyệt Hư | xấu | 1 | Giêng: Sửu; Hai: Tuất; Ba: Mùi; Tư: Thìn; Năm: Sửu; Sáu: Tuất; Bảy: Mùi; Tám: Thìn; Chín: Sửu; Mười: Tuất; M.Một: Mùi; Chạp: Thìn | — | cuoi_hoi, khai_truong | cuoi_hoi | N02 | 1 | — |
| B22 | Hoàng Sa | xấu | 1 | Giêng: Ngọ; Hai: Dần; Ba: Tý; Tư: Ngọ; Năm: Dần; Sáu: Tý; Bảy: Ngọ; Tám: Dần; Chín: Tý; Mười: Ngọ; M.Một: Dần; Chạp: Tý | — | xuat_hanh | — | N02 | 1 | — |
| B23 | Lục Bất Thành | xấu | 1 | Giêng: Dần; Hai: Ngọ; Ba: Tuất; Tư: Tỵ; Năm: Dậu; Sáu: Sửu; Bảy: Thân; Tám: Tý; Chín: Thìn; Mười: Hợi; M.Một: Mão; Chạp: Mùi | — | xay_dung, khai_truong, cau_tai | cau_tai, khai_truong | N02 | 1 | — |
| B24 | Nhân Cách | xấu | 1 | Giêng: Dậu; Hai: Mùi; Ba: Tỵ; Tư: Mão; Năm: Sửu; Sáu: Hợi; Bảy: Dậu; Tám: Mùi; Chín: Tỵ; Mười: Mão; M.Một: Sửu; Chạp: Hợi | — | cuoi_hoi, xay_dung | cuoi_hoi | N02 | 1 | — |
| B25 | Thần Cách | xấu | 1 | Giêng: Tỵ; Hai: Mão; Ba: Sửu; Tư: Hợi; Năm: Dậu; Sáu: Mùi; Bảy: Tỵ; Tám: Mão; Chín: Sửu; Mười: Hợi; M.Một: Dậu; Chạp: Mùi | — | te_tu | te_tu | N02 | 1 | — |
| B26 | Phi Ma Sát | xấu | 1 | Giêng: Tý; Hai: Dậu; Ba: Ngọ; Tư: Mão; Năm: Tý; Sáu: Dậu; Bảy: Ngọ; Tám: Mão; Chín: Tý; Mười: Dậu; M.Một: Ngọ; Chạp: Mão | — | cuoi_hoi, nhap_trach | nhap_trach | N02, N09 | 2 (TVTP, N09) | — |
| B27 | Ngũ Quỷ | xấu | 1 | Giêng: Ngọ; Hai: Dần; Ba: Thìn; Tư: Dậu; Năm: Mão; Sáu: Thân; Bảy: Sửu; Tám: Tỵ; Chín: Tý; Mười: Hợi; M.Một: Mùi; Chạp: Tuất | — | xuat_hanh | xuat_hanh | N02, N09 | 2 | — |
| B28 | Băng Tiêu Ngọa Hãm | xấu | 1 | Giêng: Tỵ; Hai: Tý; Ba: Sửu; Tư: Dần; Năm: Mão; Sáu: Tuất; Bảy: Hợi; Tám: Ngọ; Chín: Mùi; Mười: Thân; M.Một: Dậu; Chạp: Thìn | — | mọi việc | — | N02, N09, N05, N06, N07; F01 khớp | 2 (TVTP, PKB) — có mâu thuẫn | **C13**: tháng 4 và tháng 10: N02, N09 ghi **Dần, Thân**; N05–N08 ghi **Thân, Dần** (đảo). Chọn N02/N09 (2 trang, trong đó N09 ngoài dòng TVTP). Lưu ý dòng PKB gọi "Băng tiêu ngọa giải". |
| B29 | Hà Khôi (Cẩu Giảo) | xấu | 1 | Giêng: Hợi; Hai: Ngọ; Ba: Sửu; Tư: Thân; Năm: Mão; Sáu: Tuất; Bảy: Tỵ; Tám: Tý; Chín: Mùi; Mười: Dần; M.Một: Dậu; Chạp: Thìn | — | mọi việc | — | N02, N09; F01 khớp | 2 | — |
| B30 | Vãng Vong (Thổ Kỵ) | xấu | 1 | Giêng: Dần; Hai: Tỵ; Ba: Thân; Tư: Hợi; Năm: Mão; Sáu: Ngọ; Bảy: Dậu; Tám: Tý; Chín: Thìn; Mười: Mùi; M.Một: Tuất; Chạp: Sửu | — | xuat_hanh, cuoi_hoi, cau_tai, dong_tho | dong_tho | N02, N05, N06, N07, N10 | 3 (TVTP, PKB, LHĐP) | — |
| B31 | Cửu Không | xấu | 1 | Giêng: Thìn; Hai: Sửu; Ba: Tuất; Tư: Mùi; Năm: Mão; Sáu: Tý; Bảy: Dậu; Tám: Ngọ; Chín: Dần; Mười: Hợi; M.Một: Thân; Chạp: Tỵ | — | xuat_hanh, cau_tai, khai_truong | — | N02 | 1 | — |
| B32 | Trùng Tang | xấu | 3 | Giêng: can Giáp; Hai: can Ất; Ba: can Kỷ; Tư: can Bính; Năm: can Đinh; Sáu: can Kỷ; Bảy: can Canh; Tám: can Tân; Chín: can Kỷ; Mười: can Nhâm; M.Một: can Quý; Chạp: can Kỷ | — | cuoi_hoi, an_tang, xay_dung | cuoi_hoi, an_tang, xay_dung | N02, N05, N06, N07, N08 | 2 — có mâu thuẫn | **C14**: tháng 3, 12: N02 ghi **Kỷ**; N05–N08 ghi **Mậu**. Chọn Kỷ theo N02, vì N02 ghi rõ "điều chỉnh theo Ngọc Hạp thông thư triều Nguyễn" và ghi chú tháng 3, 9, 11, 12 Trùng tang và Trùng phục cùng ngày (khớp bản Kỷ). Gốc Trung Hoa (重丧: 寅月庚… theo tháng tiết) cũng khác cả hai. Độ tin cậy trung bình. |
| B33 | Trùng Phục | xấu | 1 | Giêng: can Canh; Hai: can Tân; Ba: can Kỷ; Tư: can Nhâm; Năm: can Quý; Sáu: can Mậu; Bảy: can Giáp; Tám: can Ất; Chín: can Kỷ; Mười: can Bính; M.Một: can Quý; Chạp: can Kỷ | — | cuoi_hoi, an_tang | an_tang | N02, N05, N06, N07, N08 | 2 — có mâu thuẫn | **C15**: tháng 10, 11, 12: N02 ghi **Nhâm, Quý, Kỷ**; N05–N08 ghi **Bính, Đinh, Mậu**. Đã cài: Bính (tháng 10), Quý (11), Kỷ (12). Lý do: tháng 10 bản N02 trùng Trùng Tang tháng 10 (Nhâm) trái với ghi chú của chính N02 ("chỉ tháng 3, 9, 11, 12 cùng ngày") → nhận Bính của dòng PKB; tháng 11–12 giữ N02 vì đúng ghi chú đó. Đây là chọn có lập luận, không phải đa số rõ ràng — **cần người duyệt xem lại**. |
| B34 | Lôi Công | xấu | 1 | Giêng: Dần; Hai: Hợi; Ba: Tỵ; Tư: Thân; Năm: Dần; Sáu: Hợi; Bảy: Tỵ; Tám: Thân; Chín: Dần; Mười: Hợi; M.Một: Tỵ; Chạp: Thân | — | xay_dung | xay_dung | N02 | 1 | — |
| B35 | Cô Thần | xấu | 1 | Giêng: Tuất; Hai: Hợi; Ba: Tý; Tư: Sửu; Năm: Dần; Sáu: Mão; Bảy: Thìn; Tám: Tỵ; Chín: Ngọ; Mười: Mùi; M.Một: Thân; Chạp: Dậu | — | cuoi_hoi, nhap_trach | cuoi_hoi, nhap_trach | N02, N05, N06, N07, N08 | 2 | — |
| B36 | Quả Tú | xấu | 1 | Giêng: Thìn; Hai: Tỵ; Ba: Ngọ; Tư: Mùi; Năm: Thân; Sáu: Dậu; Bảy: Tuất; Tám: Hợi; Chín: Tý; Mười: Sửu; M.Một: Dần; Chạp: Mão | — | cuoi_hoi, nhap_trach | cuoi_hoi, nhap_trach | N02, N05, N06, N07, N08 | 2 | — |
| B37 | Sát Chủ | xấu | 3 | Giêng: Tỵ; Hai: Tý; Ba: Mùi; Tư: Mão; Năm: Thân; Sáu: Tuất; Bảy: Sửu; Tám: Hợi; Chín: Ngọ; Mười: Dậu; M.Một: Dần; Chạp: Thìn | — | mọi việc | mọi việc | N02, N10 (bản 2), N05–N08 (khác) | 2 — có mâu thuẫn | **C16**: tháng 1–2: N02 và N10 (bản 2) ghi **Tỵ, Tý**; dòng PKB ghi **Tý, Tỵ** → đa số 2 dòng chọn Tỵ, Tý. Tháng 7–8: N02 và PKB ghi **Sửu, Hợi**; N10 (bản 2) ghi **Hợi, Sửu** → đa số chọn Sửu, Hợi. Kết quả trùng đúng N02. N10 còn ghi 2 bản Sát Chủ khác (theo nhóm tháng, theo mùa) và Sát Chủ theo giờ — không cài. |
| B38 | Nguyệt Hình | xấu | 1 | Giêng: Tỵ; Hai: Tý; Ba: Thìn; Tư: Thân; Năm: Ngọ; Sáu: Sửu; Bảy: Dần; Tám: Dậu; Chín: Mùi; Mười: Hợi; M.Một: Mão; Chạp: Tuất | — | mọi việc | — | N02; F01 khớp | 1 | — |
| B39 | Tội Chỉ | xấu | 1 | Giêng: Ngọ; Hai: Tý; Ba: Mùi; Tư: Sửu; Năm: Thân; Sáu: Dần; Bảy: Dậu; Tám: Mão; Chín: Tuất; Mười: Thìn; M.Một: Hợi; Chạp: Tỵ | — | te_tu, to_tung | — | N02; F02 khớp | 1 | — |
| B40 | Nguyệt Kiến Chuyển Sát | xấu | 1 | Giêng: Mão; Hai: Mão; Ba: Mão; Tư: Ngọ; Năm: Ngọ; Sáu: Ngọ; Bảy: Dậu; Tám: Dậu; Chín: Dậu; Mười: Tý; M.Một: Tý; Chạp: Tý | — | dong_tho | dong_tho | N02; F03 khớp (tháng 10 Tý) | 1 | N02 chỉ điền cột Giêng, Mười, M.Một, Chạp. Hiểu là theo mùa (Xuân, Hạ, Thu, Đông) — **diễn giải của dự án**, được F03 hỗ trợ (tháng 10 âm, ngày Tý ra sao này). Tương tự cho B41–B45, B47–B50. |
| B41 | Thiên Địa Chính Chuyển | xấu | 1 | Giêng: Quý Mão; Hai: Quý Mão; Ba: Quý Mão; Tư: Bính Ngọ; Năm: Bính Ngọ; Sáu: Bính Ngọ; Bảy: Đinh Dậu; Tám: Đinh Dậu; Chín: Đinh Dậu; Mười: Canh Tý; M.Một: Canh Tý; Chạp: Canh Tý | — | dong_tho | dong_tho | N02 | 1 | Diễn giải theo mùa — xem B40. |
| B42 | Thiên Địa Chuyển Sát | xấu | 1 | Giêng: Ất Mão; Hai: Ất Mão; Ba: Ất Mão; Tư: Bính Ngọ; Năm: Bính Ngọ; Sáu: Bính Ngọ; Bảy: Tân Dậu; Tám: Tân Dậu; Chín: Tân Dậu; Mười: Nhâm Tý; M.Một: Nhâm Tý; Chạp: Nhâm Tý | — | dong_tho | dong_tho | N02 | 1 | Diễn giải theo mùa — xem B40. |
| B43 | Lỗ Ban Sát | xấu | 1 | Giêng: Tý; Hai: Tý; Ba: Tý; Tư: Mão; Năm: Mão; Sáu: Mão; Bảy: Ngọ; Tám: Ngọ; Chín: Ngọ; Mười: Dậu; M.Một: Dậu; Chạp: Dậu | — | xay_dung | xay_dung | N02 | 1 | Diễn giải theo mùa — xem B40. |
| B44 | Phủ Đầu Sát | xấu | 1 | Giêng: Thìn; Hai: Thìn; Ba: Thìn; Tư: Mùi; Năm: Mùi; Sáu: Mùi; Bảy: Dậu; Tám: Dậu; Chín: Dậu; Mười: Tý; M.Một: Tý; Chạp: Tý | — | xay_dung | — | N02; F03 khớp | 1 | Nguồn viết "Phủ đầu dát"; dùng tên thông dụng "Phủ Đầu Sát". Diễn giải theo mùa — xem B40. |
| B45 | Tam Tang | xấu | 1 | Giêng: Thìn; Hai: Thìn; Ba: Thìn; Tư: Mùi; Năm: Mùi; Sáu: Mùi; Bảy: Tuất; Tám: Tuất; Chín: Tuất; Mười: Sửu; M.Một: Sửu; Chạp: Sửu | — | xay_dung, cuoi_hoi, an_tang | an_tang | N02; F02 khớp (tháng 9 Tuất) | 1 | Diễn giải theo mùa — xem B40. |
| B46 | Ngũ Hư | xấu | 1 | Giêng: Tỵ; Hai: Dậu; Ba: Sửu; Tư: Thân; Năm: Tý; Sáu: Thìn; Bảy: Hợi; Tám: Mão; Chín: Mùi; Mười: Dần; M.Một: Ngọ; Chạp: Tuất | — | xay_dung, cuoi_hoi, an_tang | — | N02; F01 khớp | 1 | — |
| B47 | Tứ Thời Đại Mộ | xấu | 1 | Giêng: Ất Mùi; Hai: Ất Mùi; Ba: Ất Mùi; Tư: Bính Tuất; Năm: Bính Tuất; Sáu: Bính Tuất; Bảy: Tân Sửu; Tám: Tân Sửu; Chín: Tân Sửu; Mười: Nhâm Thìn; M.Một: Nhâm Thìn; Chạp: Nhâm Thìn | — | an_tang, xay_dung | an_tang, xay_dung | N02 | 1 | Diễn giải theo mùa — xem B40. |
| B48 | Thổ Cấm | xấu | 1 | Giêng: Hợi; Hai: Hợi; Ba: Hợi; Tư: Dần; Năm: Dần; Sáu: Dần; Bảy: Tỵ; Tám: Tỵ; Chín: Tỵ; Mười: Thân; M.Một: Thân; Chạp: Thân | — | xay_dung, an_tang, dong_tho | dong_tho | N02, N05, N06, N07, N08 | 2 | N05–N08 ghi đủ 12 tháng (Hợi×3, Dần×3, Tỵ×3, Thân×3), **xác nhận độc lập** cách diễn giải theo mùa của B40. |
| B49 | Ly Sàng | xấu | 1 | Giêng: Dậu; Hai: Dậu; Ba: Dậu; Tư: Dần, Ngọ; Năm: Dần, Ngọ; Sáu: Dần, Ngọ; Bảy: Tuất; Tám: Tuất; Chín: Tuất; Mười: Tỵ; M.Một: Tỵ; Chạp: Tỵ | — | cuoi_hoi, nhap_trach | cuoi_hoi, nhap_trach | N02; F02 khớp (tháng 9 Tuất) | 1 | Diễn giải theo mùa — xem B40. Ô mùa Hạ có hai chi (Dần, Ngọ). |
| B50 | Tứ Thời Cô Quả | xấu | 1 | Giêng: Sửu; Hai: Sửu; Ba: Sửu; Tư: Thìn; Năm: Thìn; Sáu: Thìn; Bảy: Mùi; Tám: Mùi; Chín: Mùi; Mười: Tuất; M.Một: Tuất; Chạp: Tuất | — | cuoi_hoi, nhap_trach | cuoi_hoi, nhap_trach | N02; F01 khớp (tháng 9 Mùi) | 1 | Diễn giải theo mùa — xem B40. |
| B51 | Không Phòng | xấu | 1 | Giêng: Thìn; Hai: Tỵ; Ba: Tý; Tư: Tuất; Năm: Hợi; Sáu: Mùi; Bảy: Dần; Tám: Mão; Chín: Ngọ; Mười: Thân; M.Một: Dậu; Chạp: Sửu | — | cuoi_hoi | cuoi_hoi | N02 (N10: bảng khác) | 1 — có mâu thuẫn | **C17**: N10 ghi Không Phòng theo mùa (Xuân Thìn Tỵ Tý; Hạ Tuất Hợi Mùi; Thu Mão Dần Ngọ; Đông Thân Dậu Sửu) — **cùng 12 chi nhưng gộp theo mùa**, tức N02 là bản theo tháng của cùng quy tắc (tháng 1 Thìn, 2 Tỵ, 3 Tý…). Chọn N02 (chính xác hơn, khớp N10 về tập chi mỗi mùa). |
| B52 | Âm Thác | xấu | 1 | Giêng: Canh Tuất; Hai: Tân Dậu; Ba: Canh Thân; Tư: Đinh Mùi; Năm: Bính Ngọ; Sáu: Đinh Tỵ; Bảy: Giáp Thìn; Tám: Ất Mão; Chín: Giáp Dần; Mười: Quý Sửu; M.Một: Nhâm Tý; Chạp: Quý Hợi | — | xuat_hanh, cuoi_hoi, an_tang | — | N02 | 1 | — |
| B53 | Dương Thác | xấu | 1 | Giêng: Giáp Dần; Hai: Ất Mão; Ba: Giáp Thìn; Tư: Đinh Tỵ; Năm: Bính Ngọ; Sáu: Đinh Mùi; Bảy: Canh Thân; Tám: Tân Dậu; Chín: Canh Tuất; Mười: Quý Hợi; M.Một: Nhâm Tý; Chạp: Quý Sửu | — | xuat_hanh, cuoi_hoi, an_tang | — | N02 | 1 | — |
| B54 | Quỷ Khốc | xấu | 1 | mọi tháng: Tuất | — | te_tu, an_tang, xay_dung | an_tang, xay_dung | N02; F02 khớp | 1 | Nguồn ghi Tuất cả 12 tháng. |
| B55 | Địa Hỏa | xấu | 1 | Giêng: Tuất; Hai: Dậu; Ba: Thân; Tư: Mùi; Năm: Ngọ; Sáu: Tỵ; Bảy: Thìn; Tám: Mão; Chín: Dần; Mười: Sửu; M.Một: Tý; Chạp: Hợi | — | mọi việc | — | N05, N06, N07, N08 | 1 (PKB) | Không có trong N02. |
| B56 | Thập Ác Đại Bại | xấu | 3 | năm Giáp/Kỷ tháng 3: Mậu Tuất; năm Giáp/Kỷ tháng 7: Quý Hợi; năm Giáp/Kỷ tháng 10: Bính Thân; năm Giáp/Kỷ tháng 11: Đinh Hợi; năm Ất/Canh tháng 4: Nhâm Thân; năm Ất/Canh tháng 9: Ất Tỵ; năm Bính/Tân tháng 3: Tân Tỵ; năm Bính/Tân tháng 9: Canh Thìn; năm Bính/Tân tháng 10: Giáp Thìn; năm Mậu/Quý tháng 6: Kỷ Sửu | — | cau_tai, giao_dich, xay_dung, khai_truong | cau_tai, giao_dich, xay_dung, khai_truong | N10, N11 | 2 (LHĐP, N11) | — |
