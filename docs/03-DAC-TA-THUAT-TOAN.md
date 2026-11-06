# Đặc tả thuật toán — lõi lịch `lvhv-core` v0.1

Trạng thái: bản 0.1 (A2), 2026-11-06; bổ sung §15 (hai phương pháp) sau kiểm định. Tuân theo `01-HOP-DONG-KIEN-TRUC.md` (§2, §3, §4).
Mã nguồn: `core/src/` (`jd.rs`, `astro.rs`, `lunar.rs`, `terms.rs`, `canchi.rs`, `dayinfo.rs`).

## 0. Quy ước chung

- Múi giờ: UTC+7 (`astro::TZ = 7`) cho mọi thời điểm từ 1968-01-01 00:00 UTC+8; trước đó UTC+8
  (quy ước lịch miền Bắc, §16). Miền hỗ trợ 1900-01-01 → 3000-12-31.
- Ngày mới bắt đầu lúc 00:00 UTC+7. Một sự kiện thiên văn tại thời điểm JD (UT) thuộc ngày
  JDN = ⌊JD + 0,5 + 7/24⌋.
- Lịch dương: Gregory đón trước (proleptic). JDN theo công thức số nguyên chuẩn
  (Fliegel–Van Flandern), kiểm tra khứ hồi 1990–3010.
- Thứ trong tuần: `weekday = (JDN + 1) mod 7`, 0 = Chủ nhật.
- Số thực: `f64` (IEEE-754 double). Không có bảng tra lớn; không có bộ nhớ đệm toàn cục.

## 1. Sóc (trăng mới) — phương pháp tham chiếu của âm lịch

Theo thuật toán của Hồ Ngọc Đức ("Thuật toán tính âm lịch"), vốn dựa trên công thức trăng
mới trong Meeus, *Astronomical Algorithms*:

- Chỉ số k: k = 0 là Sóc ngày 1900-01-01 (k_HND = k_Meeus + 1237).
- T = k / 1236,85; JD₁ = 2415020,75933 + 29,53058868·k + 0,0001178·T² − 0,000000155·T³
  + 0,00033·sin(166,56° + 132,87°·T − 0,009173°·T²).
- Ba góc trung bình M (Mặt Trời), M′ (Mặt Trăng), F (đối số vĩ độ) và tổng hiệu chỉnh C₁ gồm
  12 số hạng sin (hệ số 0,1734−0,000393T; 0,0021; −0,4068; 0,0161; −0,0004; 0,0104; −0,0051;
  −0,0074; 0,0004; −0,0004; −0,0006; 0,0010; 0,0005).
- Hiệu chỉnh ΔT của tác giả (đơn vị ngày): nếu T < −11 dùng đa thức bậc 4, ngược lại
  ΔT = −0,000278 + 0,000265·T + 0,000262·T². Kết quả: JD_UT = JD₁ + C₁ − ΔT.
- Ngày Sóc (UTC+7) = ⌊JD_UT + 0,5 + 7/24⌋.

Lõi dùng **đúng** công thức này (hợp đồng §2: "phương pháp tham chiếu của core: thuật toán
Hồ Ngọc Đức"), không thay bằng mô hình chính xác hơn, để kết quả trùng với các lịch Việt Nam
phổ biến dựng trên cùng thuật toán.

## 2. Kinh độ Mặt Trời cho Trung khí (âm lịch)

Cũng theo Hồ Ngọc Đức: kinh độ hình học (không chương động, không quang sai) theo dạng rút gọn
của Meeus ch.25: L₀ = 280,46645 + 36000,76983·T + 0,0003032·T²; M = 357,52910 + 35999,05030·T
− 0,0001559·T² − 0,00000048·T³; C = (1,914600 − 0,004817T − 0,000014T²)·sin M +
(0,019993 − 0,000101T)·sin 2M + 0,000290·sin 3M; λ = L₀ + C; T tính từ J2000 theo JD (không đổi
sang TT).

"Cung" của một ngày = ⌊λ / 30°⌋ tại 00:00 UTC+7 của ngày đó (0..11).

## 3. Tháng 11, tháng nhuận, đổi dương ↔ âm

1. `lunar_month11(Y)`: k = ⌊(JDN(31/12/Y) − 2415021) / 29,530588853⌋; nm = ngày Sóc k.
   Nếu cung(nm) ≥ 9 (Mặt Trời đã qua 270° lúc đầu ngày Sóc, tức Đông chí rơi vào tháng trước)
   thì lấy Sóc k−1. Kết quả là ngày đầu tháng 11 âm (tháng chứa Đông chí).
2. Bảng một "tuế": các Sóc liên tiếp từ tháng 11 năm Y−1 đến trước tháng 11 năm Y; có 12 hoặc
   13 tháng.
3. Năm 13 tháng: tháng nhuận là tháng thứ i ≥ 1 đầu tiên mà cung(đầu tháng i) = cung(đầu
   tháng i+1), tức tháng không chứa Trung khí (bội số 30°). Tháng nhuận mang số của tháng
   trước nó. (Cách so cung tại hai đầu tháng của Hồ Ngọc Đức tương đương với "không có Trung
   khí bắt đầu trong tháng", vì Trung khí cách nhau ~30,4 ngày > 1 tháng âm.)
4. Năm âm của tháng 11, 12 (ở đầu bảng) là Y−1; các tháng còn lại là Y.
5. `solar_to_lunar`: chọn bảng tuế chứa ngày, tìm tháng có ngày đầu ≤ JDN.
   `lunar_to_solar(ly, lm, ld, leap)`: tháng 11, 12 nằm trong bảng của Y = ly+1, tháng 1..10
   trong bảng Y = ly; trả `None` nếu tháng (nhuận) không tồn tại hoặc ld > độ dài tháng.

Kiểm tra: khứ hồi dương → âm → dương mọi ngày 2000-01-01 → 3000-12-31, đồng thời kiểm tra
tính liên tục (ngày âm tăng 1 mỗi ngày, mùng 1 chỉ sau ngày cuối tháng): 0 lỗi (365 608 ngày,
2,6 s bản release).

## 4. Can Chi

Đúng công thức hợp đồng §3:
- Ngày: can = (JDN + 9) mod 10, chi = (JDN + 1) mod 12.
- Năm âm Y: can = (Y + 6) mod 10, chi = (Y + 8) mod 12.
- Tháng âm lm: chi = (lm + 1) mod 12 (tháng 1 = Dần); can = (2·(can_năm mod 5) + 2 + lm − 1)
  mod 10 (Giáp/Kỷ → Bính Dần, …, Mậu/Quý → Giáp Dần). Tháng nhuận dùng Can Chi của tháng chính.
- Can giờ Tý = 2·(can_ngày mod 5) (Giáp/Kỷ → Giáp Tý, …, Mậu/Quý → Nhâm Tý).
- Vị trí trong vòng 60 Giáp Tý: n = (6·can + 55·chi) mod 60 (định lý số dư Trung Hoa).

## 5. Nạp âm

Nạp âm thứ ⌊n/2⌋ trong 30 tên (Hải Trung Kim, Lô Trung Hỏa, …, Đại Hải Thủy), n là vị trí
60 Giáp Tý của Can Chi ngày.

## 6. 24 tiết khí — mô hình chính xác hơn

Tiết khí thứ i (0..23) bắt đầu khi kinh độ **biểu kiến** của Mặt Trời = 15·i° (Xuân Phân = 0°).
Khác với §2 (dành cho âm lịch theo hợp đồng), ở đây dùng mô hình tốt hơn:

- Kinh độ hình học: VSOP87 rút gọn theo Phụ lục III của Meeus (L0 64 số hạng, L1 34, L2 20,
  L3 7, L4 3, L5 1), hiệu chỉnh về FK5 (−0,09033″), chương động theo kinh độ Δψ (4 số hạng,
  Meeus ch.22, sai số ~0,5″), quang sai −20,4898″/R.
- Thời gian: VSOP87 cần TT; đổi UT → TT bằng ΔT theo đa thức Espenak–Meeus (NASA 2006),
  xem §11 (từ bổ sung A1 có thêm các khoảng 1860–1986, §16.1).
- Giải phương trình λ(t) = 15·i bằng Newton với đạo hàm số (bước 0,01 ngày), dừng khi bước
  < 1e-8 ngày.
- Tiết khí của một ngày (hợp đồng §3) = tiết khí mới nhất bắt đầu trước 00:00 UTC+7 ngày hôm
  sau: i = ⌊λ(00:00 ngày kế, UTC+7) / 15°⌋.

Độ chính xác:
- Ví dụ 25.b của Meeus (1992-10-13): lệch < 2″ so với VSOP87 đầy đủ (kiểm tra tự động).
- Xuân phân/hạ chí/đông chí 2000 và xuân phân 2024: lệch < 2 phút so với thời điểm công bố
  (kiểm tra tự động; dung sai đặt rộng vì giá trị công bố làm tròn đến phút).
- Ước lượng tổng: ≲ 1 phút cho 2000–2100 (Mặt Trời đi 1″ trong ~24 s). Sau 2100 sai số do ΔT
  chiếm ưu thế (§11).
- Gần nửa đêm, 2000–2100: tiết khí sát 00:00 UTC+7 nhất là Xử Thử 2074-08-23T00:00:06
  (cách 6 giây), Đại Thử 2038-07-22T23:59:40 (20 s), Kinh Trập 2076-03-05T00:00:29, Hàn Lộ
  2003-10-09T00:00:36, Tiểu Mãn 2004-05-20T23:59:18. Ngày bắt đầu của các tiết khí này
  **không chắc chắn** ở mức độ chính xác của mô hình; cần đối chiếu với oracle (tiêu chí L6).

**Lưu ý quan trọng — hai mô hình Mặt Trời:** tiết khí hiển thị (§6) và Trung khí dùng để xác
định tháng nhuận (§2) tính bằng hai mô hình khác nhau. Kinh độ của §2 lệch so với kinh độ
biểu kiến khoảng +43″ (2000), +27″ (2100), −30″ (2500), −175″ (3000). Đối chiếu nội bộ: cung
30° lúc nửa đêm khác nhau ở 8 ngày trong 2000–2100 và 206 ngày trong 2000–3000. Hệ quả: trong
trường hợp hiếm, ngày bắt đầu Trung khí hiển thị có thể lệch 1 ngày so với Trung khí mà thuật
toán âm lịch "thấy". Đây là đánh đổi có chủ ý: âm lịch theo phương pháp tham chiếu (hợp đồng),
tiết khí theo mô hình chính xác hơn.

## 7. Tháng tiết và Trực

- Tháng tiết: ranh giới là 12 "Tiết" (kinh độ 315° + 30°·n). Lập Xuân → Dần, Kinh Trập → Mão,
  Thanh Minh → Thìn, Lập Hạ → Tỵ, Mang Chủng → Ngọ, Tiểu Thử → Mùi, Lập Thu → Thân, Bạch Lộ →
  Dậu, Hàn Lộ → Tuất, Lập Đông → Hợi, Đại Tuyết → Tý, Tiểu Hàn → Sửu.
  Công thức: chi_tháng_tiết = ((i + 3) mod 24) div 2 + 2 (mod 12), i là chỉ số tiết khí của ngày.
- Trực = (chi_ngày − chi_tháng_tiết) mod 12, 0 = Kiến (Kiến, Trừ, Mãn, Bình, Định, Chấp, Phá,
  Nguy, Thành, Thu, Khai, Bế).
- Hệ quả tự nhiên: ngày giao Tiết, trực "lặp lại" trực của ngày hôm trước. Ví dụ: 2026-11-06
  (Giáp Thân, tháng Tuất) là Khai; 2026-11-07 Lập Đông (Ất Dậu, tháng Hợi) cũng là Khai. Hai
  trang lịch công khai (ngaydep.com, xemlicham.com) cho 2026-11-07 là Khai và 2026-11-08 là
  Bế, khớp với lõi. (Một trang khác ghi 2026-11-07 là Bế, tức không đổi tháng tiết ở ngày Lập
  Đông — nguồn đó không nhất quán với quy tắc hợp đồng.)
- Ngày giao tiết được xác định theo ngày (UTC+7) chứa thời điểm giao tiết, không chia đôi ngày.

## 8. Nhị thập bát tú

- Thứ tự 28 tú: Giác, Cang, Đê, Phòng, Tâm, Vĩ, Cơ, Đẩu, Ngưu, Nữ, Hư, Nguy, Thất, Bích, Khuê,
  Lâu, Vị, Mão, Tất, Chủy, Sâm, Tỉnh, Quỷ, Liễu, Tinh, Trương, Dực, Chẩn.
- Công thức: tú = (JDN − 2461350 + 21) mod 28, tức mốc 2026-11-05 (JDN 2461350) = Tỉnh (chỉ số
  21), giá trị quan sát trên trang lịch công khai (tiêu chí L8).
- Đối chiếu quy tắc truyền thống: mỗi tú gắn với một "thất diệu" (Mộc, Kim, Thổ, Nhật, Nguyệt,
  Hỏa, Thủy lặp 4 lần): Giác Mộc Giao, Cang Kim Long, Đê Thổ Lạc, Phòng Nhật Thố, Tâm Nguyệt
  Hồ, Vĩ Hỏa Hổ, Cơ Thủy Báo, … Vì 28 chia hết cho 7, tú nào cũng luôn rơi vào cùng một thứ:
  Mộc = Thứ Năm, Kim = Thứ Sáu, Thổ = Thứ Bảy, Nhật = Chủ nhật, Nguyệt = Thứ Hai, Hỏa = Thứ Ba,
  Thủy = Thứ Tư. Với mốc trên: 2026-11-05 là Thứ Năm và Tỉnh là "Tỉnh Mộc Hãn" (Mộc) — nhất
  quán. Tổng quát, tú thứ i rơi vào thứ (4 + i) mod 7; Giác (i=0) luôn là Thứ Năm. Kiểm tra tự
  động `tu_weekday_consistency`.
- Đối chiếu thêm (trang công khai): 2026-11-07 = Liễu (Thổ, Thứ Bảy) và 2026-11-08 = Tinh
  (Nhật, Chủ nhật) — khớp. (Một trang khác ghi Nguy cho 2026-11-07, không nhất quán với chu kỳ
  thứ; không theo.)
- Vì mốc tương đương với quy tắc "Giác = Thứ Năm", chu kỳ được định nghĩa hoàn toàn bằng JDN,
  không phụ thuộc thiên văn.

## 9. 12 thần Hoàng đạo / Hắc đạo theo ngày

- Thứ tự: Thanh Long (HĐ), Minh Đường (HĐ), Thiên Hình, Chu Tước, Kim Quỹ (HĐ), Thiên Đức (HĐ),
  Bạch Hổ, Ngọc Đường (HĐ), Thiên Lao, Huyền Vũ, Tư Mệnh (HĐ), Câu Trần.
- Tên thần thứ 6: **"Kim Đường"** (quyết định đặt tên khi tích hợp Ngọc Hạp, 2026-11-06 — cần chủ
  dự án duyệt). Trước đây lõi gọi "Thiên Đức"; nhiều nguồn gọi "Bảo Quang"; crate Ngọc Hạp và một số
  trang lịch Việt ghi "Kim Đường Hoàng Đạo" (2026-11-07 trên ngaydep.com rơi đúng vị trí này). Ba tên
  cùng chỉ một vị trí. Lý do chọn "Kim Đường": (1) "Thiên Đức" trùng tên sao tốt G01 Thiên Đức trong
  bảng Ngọc Hạp — cùng một tờ lịch có thể hiện "Hoàng đạo Thiên Đức" và sao "Thiên Đức" là hai thứ khác
  nhau, dễ hiểu lầm (ví dụ 2026-11-08 có sao Thiên Đức nhưng thần là Bạch Hổ); (2) bảng nguồn N01 của
  crate Ngọc Hạp dùng "Kim Đường"; (3) trùng với trang lịch công khai dùng đối chiếu. Một tên duy nhất
  trong toàn ứng dụng: `than.name` = "Kim Đường", `ngoc_hap.sao_tot` chứa "Kim Đường Hoàng Đạo"
  (`core/src/canchi.rs::THAN[5]`, `tsport/src/canchi.ts::THAN[5]`, `ngochap/src/tables.rs::THAN[5]`).
  Đổi tên chỉ đổi chuỗi hiển thị; mọi kiểm định số (L1–L6) không bị ảnh hưởng.
- Khởi Thanh Long (chi ngày) theo **số tháng âm lịch**: tháng 1, 7 → Tý; 2, 8 → Dần;
  3, 9 → Thìn; 4, 10 → Ngọ; 5, 11 → Thân; 6, 12 → Tuất. Công thức: chi khởi = 2·((lm − 1) mod 6),
  thần = (chi_ngày − chi khởi) mod 12. Tháng nhuận dùng số của tháng chính.
- **Lựa chọn tháng âm (không dùng tháng tiết)** — lý do:
  1. Ca kiểm 2026-11-05: tháng âm 9 → khởi Thìn, ngày Mùi → Chu Tước (Hắc đạo), khớp L8.
     Theo tháng tiết (Tuất ≡ tháng 9) cũng ra Chu Tước, nên ca này **không phân biệt** được hai
     cách.
  2. Ca phân biệt: 2026-11-07 (âm 29/9, nhưng đã sang tháng tiết Hợi ≡ tháng 10, ngày Dậu).
     Theo tháng âm (khởi Thìn) → Thiên Đức (HĐ); theo tháng tiết (khởi Ngọ) → Chu Tước (Hắc đạo).
     Trang ngaydep.com ghi "Kim Đường Hoàng Đạo" (= vị trí Thiên Đức/Bảo Quang). 2026-11-08 (âm
     30/9, ngày Tuất): trang này ghi "Bạch Hổ Hắc Đạo" — theo tháng âm ra Bạch Hổ (Hắc đạo), theo
     tháng tiết ra Kim Quỹ (Hoàng đạo). Cả hai ca ủng hộ tháng âm.
  3. Đây là cách phổ biến trong các bảng "ngày Hoàng đạo" của lịch Việt (tra theo tháng âm).
  Mức bằng chứng: hai ngày phân biệt, từ **một** trang công khai. Nhiều sách cổ (hệ "Hiệp kỷ
  biện phương") dựa vào nguyệt kiến tức tháng tiết; vì vậy cần đối chiếu thêm nguồn nếu chủ dự
  án muốn khóa quy tắc.

## 10. Giờ Hoàng đạo

- Cùng chuỗi 12 thần, áp lên 12 giờ; chi giờ khởi Thanh Long theo chi ngày: Tý/Ngọ → Thân;
  Sửu/Mùi → Tuất; Dần/Thân → Tý; Mão/Dậu → Dần; Thìn/Tuất → Thìn; Tỵ/Hợi → Ngọ.
  Công thức: khởi = (8 + 2·(chi_ngày mod 6)) mod 12.
- Xuất 6 giờ Hoàng đạo theo thứ tự Tý → Hợi; giờ đồng hồ của chi h: from = (2h + 23) mod 24,
  to = 2h + 1 (Tý 23–1, Sửu 1–3, …, Hợi 21–23).
- Kiểm tra: ngày Mùi → Dần, Mão, Tỵ, Thân, Tuất, Hợi (L8); ngày Dậu → Tý, Dần, Mão, Ngọ, Mùi,
  Dậu (khớp hai trang công khai cho 2026-11-07); đủ 6 cặp ngày so với bảng truyền thống.

## 11. ΔT và độ bất định xa

- Âm lịch (§1) dùng ΔT nội tại của Hồ Ngọc Đức: khoảng 0,000262·T² ngày, tức ~0,4 phút năm
  2000, ~23 phút năm 3000 (T ≈ 11,3).
- Tiết khí (§6) dùng Espenak–Meeus: 64 s (2000), 93 s (2050), 203 s (2100), 442 s (2200),
  ~24 phút (2500), ~74 phút (3000). Sau 2150 là ngoại suy parabol −20 + 32·u²
  (u = (năm − 1820)/100). Giá trị thực của ΔT sau thế kỷ 21 **không dự báo được** chính xác;
  độ bất định vào năm 3000 cỡ hàng chục phút đến hơn một giờ.
- Hệ quả: mọi sự kiện (Sóc, tiết khí) trong vòng vài chục phút quanh nửa đêm UTC+7 sau ~2200
  đều có ngày không xác định về mặt vật lý. Kết quả của lõi chỉ là kết quả của **quy ước**
  thuật toán; tiêu chí L3 yêu cầu liệt kê các sai lệch với oracle chứ không đặt ngưỡng đạt.

### Chẩn đoán nội bộ (không phải kiểm định độc lập)

So Sóc của Hồ Ngọc Đức với Sóc theo Meeus ch.49 đầy đủ + ΔT Espenak–Meeus (`astro::precise_new_moon_ut`):
- Lệch tối đa từng thế kỷ: 3,5 phút (2000–2099), 5,4 (2100s), … 21,7 phút (2900s).
- Ngày Sóc khác nhau: 79 lần trong 2000–3000, trong đó 2 lần trước 2101:
  k=2139 (HND 2072-12-10T00:00:14, Meeus 2072-12-09T23:58:48) và
  k=2200 (HND 2077-11-16T00:00:13, Meeus 2077-11-15T23:59:50).
  Ở hai tháng này ngày âm theo lõi (HND) muộn hơn 1 ngày so với mô hình chính xác hơn
  (ví dụ 2072-12-10 lõi = mùng 1/11, mô hình khác = mùng 2/11). Lõi giữ HND theo hợp đồng.
  Đây là hai ứng viên sai lệch rõ nhất cho tiêu chí L2 khi chạy oracle.
- Các Sóc sát nửa đêm khác (2000–2100, theo HND): 2085-10-18T23:59:13, 2031-04-21T23:59:09,
  2054-05-08T00:01:55.

## 12. DayInfo, DayCtx, JSON

- `day_info(y,m,d)` trả `DayInfo`; `to_json()` là bộ tuần tự hóa viết tay, không khoảng trắng,
  thứ tự khóa đúng hợp đồng §4; chuỗi UTF-8 xuất nguyên (không thoát `\u` cho ký tự có dấu),
  chỉ thoát `"`, `\` và ký tự điều khiển.
- `ngoc_hap` do crate `lvhv-ngochap` (`ngochap/`, thành viên workspace, phụ thuộc đường dẫn của lõi)
  tính: `dayinfo::ngochap_ctx` đổi `DayCtx` của lõi sang `lvhv_ngochap::DayCtx` (kèm `solar_month`,
  `solar_day`), `ngoc_hap_of` gọi `lvhv_ngochap::evaluate`. Thứ tự khóa cố định
  `{"sao_tot":[…],"sao_xau":[…],"nen":[…],"ky":[…],"xung_tuoi":[…],"score":N}`; thứ tự phần tử theo
  bảng (xem `ngochap/src/lib.rs`). Quy tắc và cách chấm: `content/ngochap/rules.md`, `SCORING.md`.
  Ngọc Hạp tra theo tháng âm nên phụ thuộc phương pháp (Hnd/Precise) qua ngày âm.
- `day_ctx(y,m,d)` trả chỉ số số (lunar_month, lunar_day, leap, day_can, day_chi,
  month_chi_tiet, year_can, year_chi, truc, tu, than, …) cho crate Ngọc Hạp.

### 12.1 Chọn ngày tốt — `find_good_days` (`core/src/gooddays.rs`)

- `find_good_days(activity_id, from, to, birth_year: Option<i32>) -> Vec<GoodDay>` (và
  `find_good_days_with(method, …)`); `activities()` trả 16 việc (mã ASCII ổn định) của crate Ngọc Hạp.
- Mỗi ngày trong [from, to]: `lvhv_ngochap::suitability(ctx, việc, năm sinh)`; **loại** ngày
  `allowed = false` (phạm sao kỵ chính — N3 — hoặc tuổi xung với việc cá nhân); xếp điểm giảm dần, cùng
  điểm thì ngày sớm trước. `from > to` hoặc mã việc lạ → rỗng; ngày dương không hợp lệ → panic.
- `GoodDay`: solar, lunar, weekday, jdn, score, allowed (luôn true), reasons, `first_gio_hoang_dao`.
  JSON (`GoodDay::to_json`): `{"solar":{y,m,d},"lunar":{y,m,d,leap,month_len},"weekday","jdn","score",
  "allowed","reasons":[…],"first_gio_hoang_dao":{chi,from,to}}`.
- `lvhv-dump gooddays ACT|all FROM TO [BIRTH_YEAR|none]` in JSONL, mỗi dòng thêm tiền tố
  `"activity"`, `"birth_year"`. Mẫu cố định: `core/out/gooddays_sample.jsonl` (16 việc × 2026 × năm sinh
  none, 1976; 5 625 dòng); bản Precise ở `core/out/precise/` (năm 2026 trùng HND).
- N3 (`core/tests/ngochap.rs`, `#[ignore]`, chạy release): mọi ngày 2000–3000 × 16 việc × {không tuổi,
  1976}, cả hai phương pháp, định nghĩa "sao kỵ chính" lấy từ `lvhv_ngochap::major_taboos`: 0 vi phạm
  (11 699 456 lượt/phương pháp); qua `find_good_days` toàn dải: 5 698 084 (HND) / 5 698 233 (Precise)
  ngày đề xuất, 0 vi phạm.

## 13. Hiệu năng (đo 2026-11-06, sandbox Linux x86_64, 2 CPU, bản release, LTO)

- `day_info` + `to_json`, 365 ngày năm 2026: trung bình 6,8 µs, p99 30,5 µs (P1: < 1 ms).
- `solar_to_lunar`, 365 608 ngày 2000–3000: trung bình 4,1 µs, p99 7,8 µs; tổng 1,5 s.
  (Không có bộ nhớ đệm; mỗi lần gọi dựng lại bảng một tuế.)

## 15. Hai phương pháp: `Method::Hnd` và `Method::Precise` (bổ sung sau kiểm định)

### 15.1 Lý do

Báo cáo kiểm định độc lập (`docs/reports/BAO-CAO-KIEM-DINH-LICH.md`) chỉ ra:
(a) ở 6 năm (2405, 2682, 2728, 2758, 2864, 2948) phương pháp HND đánh dấu nhuận một tháng mà
theo chính cột tiết khí hiển thị của lõi lại chứa Trung khí — mâu thuẫn hai mô hình Mặt Trời
(§6). Đây là **lỗi** so với quy tắc nhuận của hợp đồng như người dùng nhìn thấy;
(b) L2: 2072-12 và 2077-11 lệch (Sóc HND sau nửa đêm ~13–14 s, oracle trước nửa đêm);
(c) L6: Xử Thử 2074 (lõi 00:00:06, oracle 23:59:55).

Lõi không đổi mặc định một cách lặng lẽ; chủ dự án quyết định.

### 15.2 Cách chọn

- `lvhv_core::Method { Hnd, Precise }`; hằng `DEFAULT_METHOD = Method::Hnd` (tương thích ngược:
  đầu ra HND giống hệt từng byte bản trước — đã kiểm tra bằng `cmp` trên 2000–3000).
- API cũ (`solar_to_lunar`, `lunar_to_solar`, `day_info`, `day_ctx`, `tet`, …) giữ nguyên chữ ký
  và dùng `DEFAULT_METHOD`. Mỗi hàm có bản `_with(method, …)` (`solar_to_lunar_with`,
  `lunar_to_solar_with`, `day_info_with`, `day_ctx_with`, `day_ctx_jdn_with`, `tet_with`,
  `jdn_to_lunar_with`, `lunar_to_jdn_with`). Đổi mặc định = sửa một hằng.
- `lvhv-dump --method hnd|precise LỆNH …` (mặc định hnd). Lệnh `terms` không phụ thuộc phương
  pháp (tiết khí luôn theo mô hình chính xác §6) và nhận mọi dải năm, ví dụ 2000–3000.

### 15.3 `Method::Precise`

- Sóc: Meeus ch.49 đầy đủ — 25 số hạng tuần hoàn (có hệ số E của độ lệch tâm quỹ đạo Trái Đất)
  + 14 số hạng hành tinh; JDE (TT) → UT bằng ΔT Espenak–Meeus (§11). Kiểm tra: ví dụ 49.a
  của Meeus (1977-02-18, JDE 2443192,65118) khớp < 2e-5 ngày.
- Trung khí (tháng 11, tháng nhuận): cung 30° của kinh độ **biểu kiến** lúc 00:00 UTC+7 —
  đúng mô hình dùng để hiển thị tiết khí (§6). Vì vậy quyết định nhuận và tiết khí hiển thị không
  thể mâu thuẫn.
- Quy tắc lịch (§3) giữ nguyên; chỉ thay hai nguồn thiên văn.
- Can Chi ngày, tiết khí, tháng tiết, Trực, tú, giờ Hoàng đạo không phụ thuộc phương pháp; Can
  Chi năm/tháng và thần ngày (theo tháng âm) thay đổi khi ngày âm thay đổi.

### 15.4 Kiểm tra tự nhất quán (test `core/tests/methods.rs`)

Với mọi tháng nhuận giao với 2000–3000 (> 350 tháng), kiểm tra: không có Trung khí nào bắt đầu
(theo `term_index_of_day`, tức tiết khí hiển thị) trong tháng đó.

| Phương pháp | Kết quả |
|---|---|
| Hnd | KHÔNG ĐẠT — 6 tháng nhuận chứa Trung khí: 7/2405 (từ 2405-08-25), 7/2682 (2682-08-25), 11/2728 (2728-12-23), 8/2758 (2758-09-24), 7/2864 (2864-08-23), 8/2948 (2948-09-24). Trùng đúng 6 đoạn "nguyên nhân khác" của báo cáo kiểm định. Test khóa danh sách này. |
| Precise | ĐẠT — 0 vi phạm. |

Precise cũng qua khứ hồi dương → âm → dương và tính liên tục cho mọi ngày 2000–3000 (0 lỗi).

### 15.5 So sánh số liệu (2000-01-01 → 3000-12-31)

Ngày âm (năm, tháng, ngày, nhuận) khác nhau:

| Cặp | 2000–2100 | 2101–3000 |
|---|---|---|
| HND vs Precise | 60 ngày, 2 đoạn | 2754 ngày, 83 đoạn |
| HND vs oracle (từ file dữ liệu oracle) | 60 ngày, 2 đoạn | 2784 ngày, 84 đoạn |
| Precise vs oracle | **0** | 90 ngày, 3 đoạn |

- Tết: HND và Precise trùng nhau mọi năm 2000–2100; khác 11 năm trong 2000–3000. Tết Precise
  trùng Tết oracle mọi năm 2000–3000.
- Hai ca 2072/2077 dưới Precise: Sóc k=2139 lúc 2072-12-09T23:58:48 → 2072-12-09 là mùng 1/11
  (HND: 30/10); Sóc k=2200 lúc 2077-11-15T23:59:50 → 2077-11-15 là mùng 1/10 (HND: 30/9). Cả
  hai khớp oracle. Ca 2077 chỉ cách nửa đêm 10 giây — dưới mức tin cậy vật lý (sai số mô hình
  Mặt Trăng + ΔT), nên "đúng" ở đây nghĩa là khớp với mô hình độc lập, không phải chân lý.
- Sóc so với oracle (lệch trung bình có dấu, lõi − oracle / lớn nhất):
  HND 2000–2100: +75 s / 232 s; 2101–3000: +569 s / 1590 s.
  Precise 2000–2100: +11 s / 34 s; 2101–3000: +17 s / 321 s.
- 3 đoạn Precise vs oracle còn lại (2101–3000), đều Sóc cách nửa đêm < 6 phút:
  2242-07-28 (Precise 23:59:55, oracle 00:00:02 hôm sau), 2962-10-19/20 (Precise 00:04:36,
  oracle 23:59:55), 2988-06-05/06 (Precise 00:04:55, oracle 23:59:50) — loại "sát nửa đêm".
- Tiết khí (như nhau ở hai phương pháp) so với oracle 2000–3000: 2 ngày bắt đầu khác nhau, cả
  hai cách nửa đêm vài giây: Xử Thử 2074 (lõi 2074-08-23T00:00:06, oracle 2074-08-22 23:59:55)
  và Xuân Phân 2963 (lõi 00:00:06, oracle 23:59:58 hôm trước). Ca 2074 vẫn làm L6 KHÔNG ĐẠT ở
  cả hai phương pháp; sai số 11 s nằm trong sai số mô hình (VSOP87 rút gọn ~1″ ≈ 24 s), không
  sửa được bằng việc chọn phương pháp. Cần chủ dự án quyết định cách xử lý (ví dụ chấp nhận
  ca sát nửa đêm có ghi chú, hoặc dùng VSOP87 đầy đủ).

### 15.6 Đánh đổi

- HND: trùng thuật toán các lịch Việt phổ biến (lợi thế "khớp lịch người dùng đang xem"),
  nhưng có 6 năm tự mâu thuẫn và lệch oracle ở 2072/2077.
- Precise: tự nhất quán, khớp oracle 2000–2100; có thể khác các lịch dùng HND ở 2072-12-09 và
  2077-11-15 (và nhiều hơn sau 2100). Chậm hơn: `day_info` trung bình 15,8 µs (p99 36 µs),
  `solar_to_lunar` 20 µs (p99 49 µs) — vẫn xa ngưỡng 1 ms.
- Đề xuất (để chủ dự án quyết định, không phải kết luận): nếu ưu tiên tiêu chí L2/L3 và tính
  nhất quán nội bộ thì đổi `DEFAULT_METHOD` sang `Precise`; nếu ưu tiên khớp lịch HND phổ biến
  thì giữ `Hnd` và ghi 6 năm vi phạm thành sai lệch đã biết. Cả hai lựa chọn đều không làm L6 ĐẠT.

## 16. Phạm vi 1900–1999 và điểm chuyển múi giờ (bổ sung A1 + A2, 2026-11-06)

Quyết định của chủ dự án: phạm vi 1900-01-01 → 3000-12-31 theo **quy ước lịch miền Bắc**. Tiêu chí khóa
trước: docs/02 "Bổ sung A1" (L9–L13). **Bổ sung A2** (docs/02, ghi rõ là quyết định SAU khi thấy kết quả
L9 lần 1, có bằng chứng lịch sử độc lập): trước 1929 dùng giờ địa phương Bắc Kinh.

### 16.1 Quy tắc đã chọn (A2)

> **Ba giai đoạn, mỗi điểm chuyển tính theo múi giờ CŨ:**
> 1. Thời điểm (Sóc, Trung khí, tiết khí) trước **1929-01-01 00:00 giờ địa phương Bắc Kinh**
>    (= 1928-12-31 16:14:20 UT) → ngày theo **UT+7:45:40** (giờ trung bình kinh tuyến 116°25′ Đ).
> 2. Từ đó đến trước **1968-01-01 00:00 UTC+8** (= 1967-12-31 16:00 UT) → ngày theo **UTC+8**.
> 3. Từ đó trở đi → ngày theo **UTC+7**.
>
> "00:00 đầu ngày" dùng để xét Trung khí/tiết khí là thời điểm bắt đầu ngày theo đúng quy tắc trên:
> giờ Bắc Kinh cho các ngày đến hết 1929-01-01, UTC+8 cho 1929-01-02 … 1967-12-31, UTC+7 từ 1968-01-01.

Độ dài ngày ở điểm chuyển (giờ lịch): **1928-12-31 dài đúng 24 giờ**; 1929-01-01 dài 23 giờ 45 phút
40 giây (00:14:20 → 24:00 UTC+8); 1967-12-31 dài 25 giờ. `local_day_of` không giảm theo thời gian nên
không ngày âm nào mất hay lặp (test `switch_constants`, `continuity_around_1929`, `l11_continuity_1967_1969`).

Mã: `core/src/astro.rs` — `TZ = 7`, `TZ_BEFORE_SWITCH = 8`, `TZ_BEIJING = 27940/3600`,
`TZ_1929_JDN = 2425613` (1929-01-01), `TZ_SWITCH_JDN = 2439857` (1968-01-01),
`local_day_of(jd_ut)`, `local_midnight_ut(jdn)`, `tz_of_day`, `tz_of_instant`. Đây là **hàm múi giờ
duy nhất**, dùng chung cho cả `Method::Hnd` và `Method::Precise` (Sóc: `hnd_new_moon_day`,
`lunar::new_moon_day`; Trung khí: `hnd_sun_sector`, `lunar::sun_sector`; tiết khí:
`terms::term_index_of_day`, `SolarTerm::local_jdn`). tsport: `tsport/src/astro.ts` cùng tên (camelCase).

Hệ quả:
- Ngày dương 1967-12-31 "dài 25 giờ" theo giờ lịch (16:00 UT → 17:00 UT ngày 1/1 đều thuộc ngày
  31/12). `local_day_of` **không giảm** theo thời gian, nên các Sóc liên tiếp luôn cho ngày tăng dần:
  không thể có ngày âm bị thiếu hay trùng (L11). Không có Sóc hay Trung khí nào rơi vào giờ đó.
- Với mọi thời điểm ≥ 1968-01-01 00:00 UTC+8, biểu thức `⌊jd + 0,5 + 7/24⌋` giống hệt từng bit bản UTC+7
  cố định trước đây → kết quả 2000–3000 **không đổi một byte** (L13, §16.5).
- ΔT của Precise cho 1860–1986 lấy theo các đa thức Espenak–Meeus (NASA) cho từng khoảng (trước đây chỉ
  dùng parabol dài hạn dưới 1986 — không ảnh hưởng 2000–3000 vì nhánh ≥ 1986 giữ nguyên).
- Miền nội bộ `YearTable`: tuế 1899–3101 (1900-01-01 là mùng 1 tháng Chạp năm âm 1899). `MIN_YEAR = 1900`,
  `MIN_JDN = 2415021`, `MIN_LUNAR_YEAR = 1899`; `BIRTH_MIN_YEAR = MIN_YEAR`.

### 16.2 Vì sao chọn mốc 1968-01-01 00:00 UTC+8 (không phải 08/8/1967, cũng không phải "theo tháng 11")

Sóc quanh điểm chuyển (HND; Precise lệch < 2 phút):

| k | UTC+8 | UTC+7 | Ngày khác? |
|---|---|---|---|
| 835 | 1967-07-08 01:00 | 1967-07-08 00:00 | không |
| 836 | 1967-08-06 10:48 | 09:48 | không |
| 837–839 | 1967-09-04 … 1967-11-02 | | không |
| **840** | **1967-12-02 00:10** | **1967-12-01 23:10** | **có** |
| 841 | 1967-12-31 11:39 | 10:39 | không |
| **842** | **1968-01-30 00:31** | **1968-01-29 23:31** | **có** (Tết Mậu Thân) |

Ba ứng viên:
1. **Đổi ngay ngày 08/8/1967 (ngày nghị định):** k = 840 tính theo UTC+7 → tháng 11 âm Đinh Mùi bắt đầu
   1967-12-01. **Bảng âm lịch 1900–1999 do chính Hồ Ngọc Đức công bố** (mảng `TK20` trong mã nguồn
   `amlich` của tác giả, © 2004 Hồ Ngọc Đức, bản phân phối lại `jquery-amlich` trên GitHub) ghi tháng 11
   Đinh Mùi bắt đầu **1967-12-02**, tức Sóc k = 840 tính theo UTC+8. Ứng viên này trái với bảng công bố.
2. **"Múi giờ của tuế = múi giờ lúc tháng 11 bắt đầu":** tháng 11 của tuế 1968 bắt đầu tháng 12/1967 (trước
   chuyển đổi về mặt sự kiện k = 840 theo bảng) → cả tuế 1968 theo UTC+8 → Tết 1968 = 1968-01-30 (giống
   miền Nam). **Trái với yêu cầu (a)** Tết miền Bắc 1968 = 1968-01-29.
3. **Mốc 1968-01-01 00:00 UTC+8 (đã chọn):** k = 840 theo UTC+8 (2/12/1967, khớp bảng TK20), k = 842 theo
   UTC+7 (29/01/1968, khớp Tết miền Bắc). Giữa hai mốc chỉ có Sóc k = 841 (31/12/1967 11:39, cùng ngày ở
   cả hai múi), Đông chí 1967-12-22 21:16 UTC+8 (cùng ngày) và Tiểu Hàn 1968-01-06 — không sự kiện nào nằm
   gần nửa đêm, nên mọi mốc trong khoảng 1967-12-03 … 1968-01-29 cho cùng kết quả; chọn 1968-01-01 vì
   (i) trùng mốc "UTC+7 từ 1968" mà nhiều nguồn ghi (§16.3), (ii) là ranh giới ngày tròn, dễ kiểm.
   Liên tục (L11): đảm bảo bởi tính đơn điệu của `local_day_of` (§16.1) và test `l11_continuity_1967_1969`.

### 16.3 Bằng chứng (truy cập 2026-11-06)

| Nguồn | Nội dung liên quan |
|---|---|
| Wikipedia (en) "Vietnamese calendar", dẫn Aslaksen, *The Mathematics of the Chinese Calendar*, tr. 29–30 | "On 8 August 1967, the North Vietnamese government issued a decree to change Vietnamese standard time from UTC+8 to UTC+7"; "North and South Vietnam celebrated Tết 1968 on different days"; Tết 1985 = 21/01/1985 |
| Danh sách thư tz (IANA), A. Treindl, 19/5/2011 | "North Vietnam went from UTC+8 to UTC+7 on August 8, 1967"; miền Nam giữ UTC+8 đến sau chiến tranh; chênh lệch làm Tết 1968 khác ngày |
| sad-pony-mathematician.blogspot.com, "An Off-by-one Error in the Tet Offensive (1968)", 4/2/2019 | Sóc 1968-01-29 16:31 UT → 23:31 giờ miền Bắc (Tết 29/01), 00:31 ngày 30 giờ miền Nam (Tết 30/01) |
| thuviengdpt.info, "Âm lịch Việt Nam qua các thời kỳ lịch sử" (Quang Mai, "theo tài liệu của Hồ Ngọc Đức", dựa trên Hoàng Xuân Hãn *Lịch và lịch Việt Nam*) | 1813–1945: lịch Hiệp Kỷ theo phép Thời Hiến nhà Thanh, "không có sự khác biệt giữa lịch Việt Nam và lịch Trung Hoa"; 1946–1954 theo Vạn niên thư Trung Quốc; miền Bắc "múi giờ thứ 8 tới năm 1967 và múi giờ thứ 7 từ 1968"; ghi chú: "từ 8/8/1967 trở đi dùng múi giờ thứ 7"; miền Nam múi 8; từ 1976 cả nước múi 7 |
| RFA (Gia Minh, 7/2/2007), phỏng vấn Lê Thành Lân (Viện KH&CN VN) và Trịnh Tiến Điều (Ủy ban Lịch) | "Việc tính lịch là từ năm 67 rồi"; "Bắt đầu từ 68 chúng ta ăn tết khác Trung Quốc một ngày"; cuối 68 đầu 69 cũng chênh một ngày |
| Nhân Dân 29/6/2006, "Chung quanh thông tin về ngày âm lịch khác nhau trên một số loại lịch" (Hà Hồng) | Tết Mậu Thân 1968 và Kỷ Dậu 1969 của Việt Nam sớm hơn Trung Quốc một ngày; lịch Việt tính theo múi 7 |
| Đài Thiên văn Hồng Kông (hko.gov.hk), lịch năm 1954, 1969, 1916 | Tháng Giêng bắt đầu 1954-02-03, 1969-02-17, 1916-02-03 (lịch Trung Quốc) |
| Mã nguồn `amlich` của Hồ Ngọc Đức (© 2004), mảng `TK20` (1900–1999) | Bảng ngày đầu tháng âm; giải mã và so với lõi ở §16.4; nhúng nguyên văn trong `core/tests/range1900.rs` |
| ytliu0.github.io/ChineseCalendar/computation.html ("Calendar Calculation") | Thời Thanh tính theo kinh tuyến Bắc Kinh và giờ mặt trời thật; "after 1913 … Beijing's mean solar time (UT+7:46)"; "After 1928, times were calculated for the meridians of 120°E (UT+8)" |
| Đài Thiên văn Hồng Kông, lịch năm 1916 (hko.gov.hk …/1916e.pdf) | Tháng Giêng Bính Thìn bắt đầu 1916-02-03 — chỉ khớp khi Sóc 1916-02-03 23:51 giờ Bắc Kinh (00:05 UTC+8 ngày 4) tính theo giờ Bắc Kinh |
| Lịch Trung Quốc (ytliu0.github.io/ChineseCalendar; hằng `UTC_OFFSET_PRE_1929` của crate `calendrical_calculations`) | Trước 1929 lịch Trung Quốc tính theo giờ địa phương Bắc Kinh (UT+1397/180 h ≈ UT+7:46); từ 1929 theo 120°E (UT+8) |

**Mâu thuẫn nguồn về ngày chuyển:** nghị định 08/8/1967 (Wikipedia, tz) so với "từ 1968" (thuviengdpt,
RFA, trang "Time in Vietnam" ghi "confirmed official UTC+07:00 from 1 January 1968"). Với lịch âm, hai cách
chỉ khác nhau ở tháng 11 Đinh Mùi (k = 840); bảng công bố của Hồ Ngọc Đức theo cách thứ hai. **Chưa tìm
được bản in lịch miền Bắc năm 1967–1968** (tờ lịch Nhà xuất bản/Nha Khí tượng năm đó) để xác nhận trực
tiếp tháng 11 Đinh Mùi bắt đầu 01/12 hay 02/12/1967 — ghi là CHƯA ĐỦ BẰNG CHỨNG cho riêng tháng đó;
lựa chọn dựa trên bảng của Hồ Ngọc Đức.

### 16.4 Đối chiếu với bảng TK20 của Hồ Ngọc Đức (1237 tháng âm của năm âm 1900–1999)

Đã giải mã `TK20` (ngày Tết + độ dài 12 tháng + tháng nhuận mỗi năm; 1237 tháng = 1236 tháng bắt đầu
trong 1900–1999 + tháng Chạp năm âm 1999) và so ngày đầu từng tháng (năm, tháng, nhuận):

| Quy tắc thử | Tháng lệch so với TK20 |
|---|---|
| UTC+7 cho mọi năm | 27 (toàn trước 1968) |
| UTC+8 cho mọi năm | 25 (toàn từ 1968, gồm Tết 1968, Tết 1969, 1984–1985 …) |
| A1 (UTC+8 trước 1968, UTC+7 từ 1968) | 4: 4/1906, 10/1914, 1/1916 (Tết), 10/1920 — Sóc 00:02–00:08 UTC+8 |
| **A2 (đã chọn), Precise** | **0 — trùng cả 1237/1237 tháng** |
| **A2 (đã chọn), Hnd** | **4 — trùng 1233/1237**: tháng 2 nhuận/3 năm 1917, tháng 5 nhuận/6 năm 1922 |

Bốn tháng còn lệch của Hnd chỉ là **vị trí tháng nhuận** (ngày đầu tháng trùng TK20): Trung khí sát nửa
đêm giờ Bắc Kinh — Cốc Vũ 1917-04-21 00:03:11 và Đại Thử 1922-07-24 00:05:30 (mô hình chính xác §6) —
nhưng kinh độ rút gọn của HND (§2) lớn hơn kinh độ biểu kiến +20″ (1917) và +37″ (1922) lúc nửa đêm
đó (≈ 8 và 15 phút thời gian), nên đặt hai Trung khí này sang trước nửa đêm, nên HND thấy tháng khác "không có Trung khí" và chọn tháng nhuận khác. Đây là
giới hạn của mô hình Mặt Trời HND (cùng loại với 6 năm §15.4), không phải của quy tắc múi giờ. Test
`tk20_month_starts_1900_1999` khóa: Precise 0 lệch; Hnd đúng 4 tháng trên. Tết 1900–1999 của cả hai phương
pháp trùng TK20 cả 100 năm, gồm **Tết 1916 = 1916-02-03** (test `tet_1916_beijing_time`).

### 16.5 Giới hạn và vấn đề mở

1. **1900–1928 — ĐÃ QUYẾT (A2):** giờ địa phương Bắc Kinh. Lịch Trung Hoa 1913–1928 dùng giờ trung
   bình Bắc Kinh; thời Thanh (1900–1911) dùng **giờ mặt trời thật** Bắc Kinh (ytliu0) — lõi dùng giờ
   trung bình cho cả 1900–1928 (chênh lệch phương trình thời gian tới ±16 phút có thể đổi ngày của một
   sự kiện rất sát nửa đêm; trong 1900–1912 không tháng nào lệch TK20 ở Precise). Gợi ý "kinh tuyến Huế"
   (Wikipedia, dòng Hiệp Kỷ 1841–1954) chưa có bằng chứng cụ thể; không tìm được bản in Hiệp Kỷ lịch
   1900–1945.
2. **1943–1945:** thuviengdpt ghi giờ hành chính GMT+8 từ 01/01/1943 và GMT+9 (giờ Tokyo) từ 1945; quy
   ước lịch của lõi không theo giờ hành chính từng giai đoạn, chỉ theo quy ước lịch (UTC+8).
3. **Miền Nam 1968–1975:** miền Nam tính theo UTC+8; lõi theo miền Bắc. Các tháng khác nhau (ví dụ Tết
   1968, Tết 1969, tháng 11/1974, 3/1975…) là chủ đích, ghi chú trong app (Đổi ngày, Giới thiệu).
4. **Tiết khí hiển thị trước 1968** tính theo giờ lịch của giai đoạn (Bắc Kinh / UTC+8, cùng hàm múi giờ), nên ngày giao tiết và trực
   thống nhất với âm lịch của cùng giai đoạn.
5. Kiểm định độc lập 1900–1999 (L10) do bên kiểm định thực hiện; tệp dữ liệu cho bên đó: §16.6.

### 16.6 Kết quả kiểm tra (2026-11-06, sandbox 2 vCPU)

| Tiêu chí | Kết quả | Bằng chứng |
|---|---|---|
| L11 liên tục 1967–1969 + Tết 1968 = 29/01 | ĐẠT (cả hai phương pháp) | `core/tests/range1900.rs::l11_continuity_1967_1969`, `switch_constants` |
| Liên tục quanh 1929-01-01 (1928–1930) | ĐẠT (cả hai phương pháp) | `range1900.rs::continuity_around_1929`, `switch_constants` |
| TK20 1900–1999 | Precise 1237/1237; Hnd 1233/1237 (vị trí tháng nhuận 1917, 1922) | `range1900.rs::tk20_month_starts_1900_1999` |
| L12 khứ hồi 1900–3000 | ĐẠT, 402 132 ngày × 2 phương pháp, 0 lỗi | `range1900.rs::l12_roundtrip_1900_3000` (`--ignored`, release) |
| L12 TS ↔ Rust 1900–3000 | ĐẠT, 402 132 dòng × 2, 0 sai lệch; Tết 1101 năm × 2, 0 sai lệch | `scripts/check-parity.sh` |
| L13 2000–3000 không đổi | ĐẠT, giống hệt từng byte (hnd, precise) | `check-parity.sh` bước 5; so `core/out/baseline-2000-3000/*.jsonl.gz` |
| L9 Tết có nguồn | 1916, 1954, 1968, 1969, 1985: khớp (L9 đầy đủ do bên kiểm định chấm) | `range1900.rs::tet_20th_century_published`, `tet_1916_beijing_time` |
| L10 | Chờ bên kiểm định độc lập | — |

Tệp cho bên kiểm định: `core/out/lunar_1900_1999.csv`, `tet_1900_1999.csv`, `newmoons_1900_1999.csv`
(cột `datetime_local` ghi kèm độ lệch đã dùng, ví dụ `1916-02-03T23:51:…+07:45:40`, `1967-12-02T00:10:30+08:00`,
`1968-01-29T23:31:10+07:00`), `dayinfo_1900_3000.jsonl.gz`, và bản `core/out/precise/…` tương ứng.
`lvhv-dump terms` cho năm < 1968 xuất cột `date_local,datetime_local` (độ lệch theo từng dòng).

## 14. Tài liệu tham khảo

- Jean Meeus, *Astronomical Algorithms*, 2nd ed., Willmann-Bell, 1998 — ch.7 (ngày Julian),
  ch.22 (chương động), ch.25 (vị trí Mặt Trời), ch.49 (pha Mặt Trăng), Phụ lục III (VSOP87 rút
  gọn).
- Hồ Ngọc Đức, "Thuật toán tính âm lịch" (bài viết và mã nguồn công bố trực tuyến).
- F. Espenak, J. Meeus, "Polynomial Expressions for Delta T", NASA Eclipse Web Site (2006).
- Nguồn cho phạm vi 1900–1999 và điểm chuyển múi giờ: §16.3.
- Trang lịch công khai dùng đối chiếu điểm (2026-11-05/07/08): ngaydep.com, xemlicham.com,
  baohatinh.vn, chogia.vn (truy cập 2026-11-06). Các trang này mâu thuẫn nhau ở Trực/tú/giờ
  của 2026-11-07; lựa chọn theo trang nhất quán với quy tắc hợp đồng, xem §7–§10.
