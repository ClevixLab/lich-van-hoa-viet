# Duyệt thiết kế — mô-đun "Người thân" trên ứng dụng cài đặt (Android, Windows, Apple) — v0.2 đợt 3

Vai: **bên duyệt thiết kế** (T5 mở rộng cho T6–T8 trong `docs/v0.2/07-PHAN-TICH-YEU-CAU-DOT-3.md` §7). Bên duyệt thiết
kế không viết đặc tả giao diện và không viết mã của các PR này. Ngày duyệt: 2026-11-07.

| Nền tảng | PR | Nhánh, bản xem | Ảnh đã xem |
|---|---|---|---|
| Android | #24 | `v0.2-w3-android` @ `34c9777` | 72 ảnh `apps/android/screenshots/and-NN-*-{light,dark,large}.png` (360 × 800 dp) |
| Windows | #25 | `v0.2-w3-windows` @ `5670e8a` | 46 ảnh `nt-*-{sang,toi}-1x.png`, một phần ảnh `-2x`, ảnh v0.1 `08-chu-lon-*`; thêm bản dựng trên Linux (Skia phần mềm) `winv02-*.png` để đối chiếu |
| Apple | #23 | `v0.2-w3-apple` @ `89dedb3` (ảnh CI lần 121, `2d30829`) | 102 ảnh `apps/apple/screenshots/`: 81 ảnh iPhone `ios-*-{light,dark,large}` (large = Accessibility L), 1 ảnh luồng `ios-flow-added-light`, 20 ảnh iPad `ipad-*-light` — xem §5 |

Căn cứ: đặc tả `docs/v0.2/08-DAC-TA-GIAO-DIEN-NGUOI-THAN.md` (bản trên `main`, `c4d3128`; với Android thêm §13 trên nhánh
PR; với Apple thêm §12-Apple trên nhánh PR), `docs/v0.2/07` (US-M/US-S/US-C, R-UX-1, R-UX-2, R-CUL-2, R-PRIV-2), `docs/v0.2/04` Bổ sung B4, `docs/v0.2/01` §3.2a,
`docs/08-QUY-TRINH-MO-RONG-TINH-NANG.md` §5–6, và phiếu duyệt PWA `docs/reviews/DUYET-THIET-KE-DOT3-PWA.md` (cách làm, ngưỡng).
Ảnh PWA đối chiếu: bộ `v02-*.png` do kiểm thử khói PWA sinh ra.

## 0. Cách duyệt

1. Đọc 08 (cả §12, §12b–§12h và §13 Android), mô tả PR #24/#25, `docs/04-BUILD-NATIVE.md` §3.4a (Windows).
2. Xem **từng** ảnh. Android: ghép ba biến thể sáng | tối | chữ lớn (1,3) của cùng màn hình thành một ảnh để so cạnh nhau.
   Windows: ghép sáng/tối 1× của cùng màn hình (đúng kích thước người dùng thấy ở 100 %), xem thêm 2× để kiểm nét chữ và dấu.
3. Đối chiếu mã ở chỗ ảnh gợi ra câu hỏi (giới hạn số dòng, màu, chế độ chụp ảnh). Không chạy ứng dụng trên máy thật.
4. Tính độ tương phản WCAG từ mã màu trong `ui/Theme.kt` (Android) và `ui/common.slint` (Windows) — hai bản dùng cùng bảng màu.

Ghi "ảnh" là thấy trên ảnh của PR; "mã" là đọc mã của nhánh; "tính" là tính từ mã màu. Mức: **chặn** (sửa trước khi
gộp) · **nên sửa** (sửa trong đợt này hoặc mở việc có hạn trước phát hành v0.2) · **góp ý**. Khi nói "chưa đủ bằng chứng"
nghĩa là bộ ảnh không cho phép kết luận đạt hay không đạt — không phải là đã thấy lỗi.

Bảng tương phản dùng chung (tính, chữ thường cần ≥ 4,5:1; đường viền điều khiển cần ≥ 3:1):

| Cặp màu | Sáng | Tối |
|---|---|---|
| `ink3` trên `paper` (chữ phụ trong thẻ) | 4,95:1 đạt | 6,31:1 đạt |
| `ink3` trên `wall` (chữ phụ trên nền trang) | **3,99:1** không đạt | (tối đạt) |
| `rule` trên `paper` (viền nút viền, ô, thẻ) | **1,33:1** | **1,34:1** |
| `rule` trên `wall` | **1,07:1** | — |
| Chữ trắng trên `red` tối `#FF6B63` (nút "Xóa hồ sơ" trong hộp xóa, Android tối) | — | **2,79:1** |
| Chữ trắng trên `redDeep` (nút chính) | 9,2:1 | 10,9:1 |
| Bia tưởng niệm `#E8C67A` trên `#2B2422` | ≈ 9:1 | ≈ 9:1 |

## 1. Nhận xét chung

Điểm mạnh thật sự, cả hai nền tảng:
- Đi đúng luồng và câu chữ của 08: bốn trang con, "Sắp tới" làm trang đầu, ô ngày có kết quả đổi lịch trực tiếp, hộp
  "Tháng nhuận (năm 2025 nhuận tháng 6)" chỉ hiện khi năm có tháng nhuận, câu "Năm 2024 không có tháng nhuận" (Windows `nt-11*`,
  `nt-14`), câu lỗi bằng lời thường, giờ nhắc 24 giờ hai ô.
- **Quyết định của chủ dự án được giữ đúng ở mọi ảnh đã xem:** bia chỉ ghi chữ ở ô "Quan hệ" ("Ông nội", "Bà nội", "Bà ngoại"),
  không lặp cách gọi, không có từ Hán-Việt cổ (mã: không có "Hiển…"; danh sách gợi ý là Bố, Mẹ, Ông nội…, Kỵ ông, Ông cố, Bà
  sơ…); `bia_tieu_de` cũ được giữ khi lưu. **B4:** biểu mẫu sự kiện của người đã mất không có khối nhắc, có câu giải thích
  (`and-18`, `nt-17`); bộ lọc "Người đã mất" chỉ có giỗ/tiên thường, có câu giải thích (`and-04`, `nt-05`); hồ sơ người đã mất
  có câu "Không nhắc sinh nhật và sự kiện…" (`nt-08`). **Thông tin báo tử** chỉ hiện trên trang hồ sơ (`and-10`, `nt-09`),
  không có trên bia, thẻ nhắc, danh sách, hộp .ics; hộp tạo sao lưu có câu "Tệp gồm cả thông tin báo tử…" (`and-22`, `nt-20`).
  **Tên trên màn hình khóa / trong thông báo** mặc định tắt, có giải thích, lưu theo máy (`and-07`, `nt-01`).
- **Màn hình tưởng niệm trang nghiêm** ở cả hai: nền gỗ tối ấm như nhau ở sáng và tối, bia vòm chữ vàng đồng serif, ảnh khung
  vàng đồng, bàn thờ đơn giản (bát hương ba nén, mâm ngũ quả, hai nến), không âm thanh, không chia sẻ, không thông tin báo tử.
  Ngày giỗ hôm nay ghi "Giỗ · hôm nay 29/9 âm lịch" (`and-12`, `nt-22`).
- Dấu tiếng Việt hiển thị đúng ở mọi ảnh, cả chữ serif cỡ lớn ("Nguyễn", "Thường", "Kỷ Hợi"), dấu chồng (ễ, ỗ, ự, ẫ) ở
  chữ lớn 1,3 (Android) và 2× (Windows). Không thấy dấu bị cắt ở dòng trên.
- Android theo Material 3 hợp lý: nút phân đoạn (`SegmentedButton`), ô viền, công tắc, hộp thoại; nhãn ô Ngày/Tháng/Năm và
  Giờ/Phút đặt **trên** ô nên đọc được ở chữ lớn. Windows thống nhất với ứng dụng v0.1 (thanh mục, nút viên thuốc, thẻ bo góc).

Điểm yếu chính:
- **Chữ lớn bị cắt hoặc rút gọn** trên Android (bộ lọc "Người đang sống" thành "Người đang", tên người thành "Nguyễn V…").
- **Ở A++ thanh trên cùng của Windows tràn**: nút A++ và nút sáng/tối ra ngoài mép cửa sổ (đúng lỗi PWA đã gặp khi thêm mục
  thứ năm, 08 §12b "Thêm"); và không có ảnh nào của mô-đun ở A+/A++.
- **Bộ ảnh Android không chụp ô nhập ngày khi đang dùng** (âm lịch, tháng nhuận, kết quả, lỗi) — đúng chỗ rủi ro R-UX-1 lớn
  nhất của đợt này.
- Đường viền nút viền/thẻ/công tắc gần như không thấy (1,07–1,34:1) ở cả hai nền tảng; vài chữ phụ trên nền trang 3,99:1.

## 2. Android (PR #24)

### 2.1 Mức chặn

#### TKN-A-01 — Chữ lớn: nhãn bộ lọc bị cắt "Người đang sống" → "Người đang" — **chặn**

- Màn hình: Theo kỳ. Ảnh: `and-02-theo-ky-thang-large.png`, `and-03-theo-ky-quy-large.png`, `and-04-theo-ky-da-mat-large.png`.
- Sai ở đâu: ở cỡ chữ hệ thống 1,3, ô thứ ba của nhóm bộ lọc chỉ còn hai dòng "Người / đang"; chữ "sống" mất hẳn. Mã:
  `Choice()` trong `ui/people/Widgets.kt` đặt `maxLines = 2` cho nhãn của `SegmentedButton`, ba ô chia đều 360 dp nên ô hẹp.
  Người 35–70 tuổi hay bật chữ lớn; nhãn còn lại "Người đang" không có nghĩa, và đây là bộ lọc của R13/B4. Ở cỡ 1,3 các ô khác
  cũng đã xuống hai dòng ("Sắp / tới", "Theo / kỳ"); ở cỡ 1,5–2,0 (Android cho phép) sẽ còn cắt nhiều hơn — chưa có ảnh.
- Sửa: bỏ `maxLines` (hoặc để ô cao theo nội dung) **và** khi `fontScale ≥ 1,15` xếp lại như PWA ở A++ (`v02-period-360-xl.png`):
  nhóm kỳ thành lưới 2 × 2, bộ lọc thành ba hàng dọc rộng hết khung. Thêm kiểm thử Robolectric ở `fontScale` 1,3 và 2,0 khẳng
  định không nút nào của mô-đun có `TextLayoutResult.hasVisualOverflow`/bị cắt dòng, và chụp thêm ảnh `-xlarge` (2,0) cho 02–05.

#### TKN-A-02 — Bộ ảnh thiếu các màn hình cốt lõi: không thể duyệt ô nhập ngày âm/nhuận và trạng thái lỗi — **chặn** (chưa đủ bằng chứng)

- Ảnh có: `and-13` chỉ có nhóm đầu của biểu mẫu trống; `and-14` nhóm đầu khi sửa; `and-15` phần ảnh/báo tử/nhắc; `and-17`,
  `and-18`, `and-19` ô ngày **trống**. Không ảnh nào có:
  - ô ngày ở chế độ "Nhập theo âm lịch" với hộp "Tháng nhuận (năm 2025 nhuận tháng 6)", câu "Năm 2026 không có tháng nhuận",
    "Không rõ năm mất", và **dòng kết quả đổi lịch** (L1, L2, R-UX-1 — rủi ro "Cao" của 07);
  - biểu mẫu khi có lỗi: chữ đỏ dưới ô, khung "Còn n chỗ cần sửa" (mã có, `Forms.kt` dòng 183–187);
  - thanh điều hướng đáy với thẻ thứ tư "Người thân" (§13 dòng 1) — mọi ảnh chụp màn hình con không có thanh đáy;
  - thẻ "Thông báo của ứng dụng đang tắt…" ở Sắp tới, nhắc sao lưu, hộp xem trước khôi phục và hộp xác nhận thứ hai "Thay toàn bộ",
    nút "Dừng khói"/khói đang chạy (ảnh chụp với hoạt ảnh tắt nên nút bị ẩn), nội dung thông báo trên màn hình khóa.
- Vì sao chặn: docs/08 §6 bắt buộc ảnh chụp trước khi phát hành; bên duyệt thiết kế chấm trên ảnh. Phần quan trọng nhất của
  US-M-01/02 (người lớn tuổi nhập ngày âm, tháng nhuận) hiện **không duyệt được**. Đây là kiểm thử không đủ năng lực, không phải
  đã thấy lỗi.
- Sửa: thêm vào `PeopleScreensTest` các ảnh (sáng, tối, chữ lớn): `them-da-mat-am-nhuan` (15/6/2025, nhuận, kết quả "Dương lịch:
  Thứ Sáu, 08/08/2025"), `them-da-mat-duong-ket-qua` (08/08/2025 → "Âm lịch: 15/6 nhuận năm Ất Tỵ (2025)"),
  `them-da-mat-loi` (lưu khi trống + một lỗi ngày), `them-dang-song-sinh-nhat-am`, `sap-toi-thong-bao-tat`, `nhac-sao-luu`,
  `hop-khoi-phuc` + `hop-thay-toan-bo`, `tuong-niem-khoi-chay` (hoạt ảnh bật, thấy nút "Dừng khói"), một ảnh toàn màn hình có
  thanh đáy, và ảnh thông báo (nội dung `title`/`body` mặc định không tên). Cuộn để chụp **cả** biểu mẫu, không chỉ đầu trang.

### 2.2 Mức nên sửa

#### TKN-A-03 — Chữ lớn: tên người bị rút gọn bằng "…" trong mục sự kiện — nên sửa

- Ảnh: `and-08-ho-so-da-mat-large.png` ("Giỗ hết · Ông / Nguyễn V…"), `and-09-ho-so-dang-song-large.png` ("Sinh nhật Lê Văn /
  Cường (37…"). Mã: `OccRow` đặt `maxLines = 3, overflow = Ellipsis` cho dòng 1; cột phải ("Còn 253 ngày") rộng tới 96 dp nên
  cột giữa hẹp.
- Sửa: bỏ giới hạn dòng của dòng 1; ở `fontScale ≥ 1,15` đưa "Còn n ngày" + biểu tượng xuống dưới dòng ngày (một cột).

#### TKN-A-04 — Đường viền nút viền, ô chọn, thẻ mục gần như không thấy (1,07–1,34:1) — nên sửa

- Ảnh: nút "Sửa" (`and-08-*-light`), "Cập nhật: đã mất", "Sửa" (`and-09-*-light`), "+ Thêm" (`and-05`), "Hủy" (`and-22`), các ô
  chưa chọn của thanh phân đoạn, viền thẻ mục (`OccRow`). Tính: `rule #E4DBD5` trên `paper` 1,33:1, trên `wall` 1,07:1 (cần ≥ 3:1,
  WCAG 1.4.11). Ở ảnh sáng, nút "Sửa" trông như chữ trôi, không giống nút.
- Sửa: thêm biến `outline` cho viền điều khiển, ≥ 3:1 với cả `paper` và `wall` (ví dụ sáng `#8F807D`, tối `#7A6B69`); dùng cho
  `OutlinedButton`, `SegmentedButtonDefaults.inactiveBorderColor`, viền ô mục. Giữ `rule` cho đường kẻ trang trí.

#### TKN-A-05 — Công tắc ở trạng thái tắt gần như vô hình — nên sửa

- Ảnh: `and-07-cai-dat-nhac-light.png`, `-dark.png` — công tắc "Hiện tên người trên màn hình khóa" (O-9) ở trạng thái tắt: rãnh
  và núm gần trùng màu nền thẻ. Đây chính là công tắc riêng tư mà người dùng cần thấy rõ đang tắt.
- Sửa: `SwitchDefaults.colors(uncheckedBorderColor = outline, uncheckedThumbColor = ink3, uncheckedTrackColor = paper2)` để
  viền và núm ≥ 3:1 như mặc định M3.

#### TKN-A-06 — Giao diện tối: nút "Xóa hồ sơ" trong hộp xóa chỉ 2,79:1 — nên sửa

- Ảnh: `and-20-hop-xoa-dark.png` — chữ trắng trên nền hồng đỏ `#FF6B63`. Nút này là hành động không hoàn tác được; chữ phải đọc rõ.
- Sửa: ở tối dùng nền `redDeep` (10,9:1) hoặc chữ sẫm `#2A0D0C` trên `red`; đo lại.

#### TKN-A-07 — Chữ phụ trên nền trang 3,99:1 — nên sửa

- Ảnh: "Đang xem kỳ có hôm nay" (`and-02…04-light`), dòng phụ dưới tiêu đề khi chữ nằm trên `wall`. Tính `ink3`/`wall` 3,99:1.
  PWA đã sửa đúng lỗi này (TK-13, 08 §6 bảng tương phản).
- Sửa: chữ phụ đặt trên `wall` dùng `ink2` (5,96:1).

#### TKN-A-08 — Mục sự kiện không ghi tên người — nên sửa

- Ảnh: `and-01` ("Tốt nghiệp đại học"), `and-02`, `and-03`, `and-09` (Lần tới), hộp .ics `and-21` ("20/11/2026 — Tốt nghiệp
  đại học"). 08 §2 quy định dòng 1 = "nhãn + tên"; PWA (TK-11) và Windows (`nt-02`: "Tốt nghiệp · Anh Lê Văn Cường") đều có tên.
  Trong danh sách trộn nhiều người, "Tốt nghiệp đại học" không cho biết của ai.
- Mã: `occTitle()` đã định nối " · tên" cho sự kiện nhưng ảnh cho thấy `o.name` rỗng với sự kiện — tên người không được điền từ
  `person_ids` của sự kiện.
- Sửa: điền `name` cho lần xuất hiện của sự kiện (người đầu tiên trong `person_ids`), dùng cùng chuỗi cho xem trước .ics; thêm
  kiểm thử khẳng định "Tốt nghiệp đại học · Lê Văn Cường".

#### TKN-A-09 — Hộp thoại không theo cỡ chữ lớn — nên sửa (chưa đủ bằng chứng)

- Ảnh: `and-20-hop-xoa-large.png`, `and-21-hop-ics-large.png`, `and-22` — chữ trong hộp thoại ở biến thể "large" **cùng cỡ** với
  biến thể thường, trong khi trang phía sau đã to lên. Hoặc ảnh chụp hộp thoại (`captureScreenRoboImage`) không áp `fontScale`,
  hoặc hộp thoại thật không theo cỡ chữ hệ thống.
- Sửa: xác định nguyên nhân; nếu là ảnh, chụp hộp thoại trong cùng cấu hình `fontScale`; nếu là mã, bỏ cỡ chữ cố định trong
  hộp thoại. Chụp lại ba hộp ở chữ lớn.

### 2.3 Góp ý

| ID | Màn hình / ảnh | Điều thấy | Đề xuất |
|---|---|---|---|
| TKN-A-10 | Mục sự kiện (`and-01…04`) | Không có dòng 3 ("Hưởng thọ 86 tuổi") và không có chữ loại dưới biểu tượng; Windows và PWA có ("Ngày giỗ", "Sinh nhật", "Sự kiện") — 08 §2 "loại luôn có chữ đi kèm" | Thêm dòng 3 khi có; thêm chữ loại nhỏ dưới biểu tượng (hoặc ghi rõ sai khác vào §13) |
| TKN-A-11 | Hồ sơ, báo tử (`and-10`, `and-19`) | Nút "Thu gọn ▾" dùng mũi tên xuống khi khối đang mở, trong khi "Mở ▸" dùng mũi tên phải | Mở: "Thu gọn ▴"; gọn: "Mở ▾" — hoặc dùng biểu tượng mở rộng M3 xoay 180° |
| TKN-A-12 | Sao lưu (`and-06`); dữ liệu chụp | Tiêu đề khung A đổi thành "Để lịch điện thoại cùng nhắc: xuất lịch (.ics)" — hợp lý cho bản cài nhưng chưa ghi ở §13. Dữ liệu mẫu tên hiệu "Pháp danh Phúc Thiện" làm ảnh hiện "Tên hiệu / pháp danh: Pháp danh Phúc Thiện" | Ghi câu mới vào §13; dữ liệu mẫu dùng "Phúc Thiện" |
| TKN-A-13 | Biểu mẫu sự kiện (`and-17`) | Ô "Loại" là nút viên thuốc căn giữa "Tốt nghiệp ▾" — trông như nút hành động, không như ô chọn | Dùng `ExposedDropdownMenuBox` (ô viền, chữ căn trái, mũi tên phải) như M3 |
| TKN-A-14 | Biểu mẫu sửa (`and-14`) | Nhãn ô viền M3 thu nhỏ còn ~12 sp khi ô đã có chữ ("Cách gọi (đứng trước tên)") — nhỏ cho người lớn tuổi | Cân nhắc nhãn cố định phía trên ô như đã làm cho ô ngày (đã là sai khác được chấp nhận ở §13) |

### 2.4 Các sai khác ghi ở 08 §13 (Android)

| Sai khác | Đánh giá |
|---|---|
| Điều hướng: thẻ thứ tư ở thanh đáy, nút Quay lại hệ thống | **Chấp nhận** — đúng M3. Chưa có ảnh thanh đáy (TKN-A-02) |
| Trang "Cài đặt nhắc" (quyền, công tắc chung, tên trên màn hình khóa, báo thức không chính xác) | **Chấp nhận.** Câu giải thích O-6 rõ ràng. Sửa công tắc (TKN-A-05) |
| Hỏi `POST_NOTIFICATIONS` khi lưu hồ sơ đầu tiên có nhắc | **Chấp nhận** — hỏi đúng lúc người dùng hiểu lý do |
| §5.4 thay bằng thẻ "thông báo đang tắt" | **Chấp nhận.** Chưa có ảnh (TKN-A-02) |
| Không có thẻ nhắc trên Tờ lịch | **Chấp nhận** cho Android (đã có thông báo hệ thống). Lưu ý: Windows vẫn hiện thẻ ở Sắp tới; nên ghi quyết định chung cho bản cài |
| .ics → bảng chia sẻ, "Chia sẻ tệp .ics", thêm "Không gồm thông tin báo tử" | **Chấp nhận**; câu thêm nên đưa vào 08 chung (TKN-X-04) |
| Ô mật khẩu sao lưu mở ngay trong khung thay vì hộp thoại | **Chấp nhận** về trải nghiệm (bàn phím ảo trong hộp thoại thường che ô; mở tại chỗ dễ hơn trên điện thoại). Lý do ghi trong §13 ("kiểm thử không về trạng thái nghỉ") là lý do kỹ thuật, nên ghi thêm lý do người dùng |
| Photo Picker, giải mã + xoay EXIF, mã hóa lại JPEG | **Chấp nhận** (không cần quyền bộ nhớ) |
| "Gợi ý ▸" mở hàng nút chọn nhanh thay `datalist` | **Chấp nhận** — tốt hơn `datalist` cho người lớn tuổi. Danh sách đúng từ thường ngày |
| Nhãn Ngày/Tháng/Năm, Giờ/Phút nằm trên ô | **Chấp nhận** — đọc được ở chữ lớn (`and-15-large`) |
| Bia: "Ngày giỗ sắp tới: …" khi giỗ năm nay đã qua | **Chấp nhận** và đề nghị đưa vào 08 §3.9 cho mọi nền tảng (Windows đang sai, TKN-W-04) |
| Thông báo: nội dung chỉ từ `title`/`body` của lõi, chạm → hồ sơ, không thông tin báo tử | **Chấp nhận.** Chưa có ảnh thông báo (TKN-A-02) |
| Giảm chuyển động theo "Xóa hiệu ứng"/tỷ lệ hoạt ảnh 0 | **Chấp nhận** |

Sai khác **chưa** ghi ở §13 mà bên duyệt thấy: mục sự kiện thiếu dòng 3 và chữ loại (TKN-A-10); tên khung A trang Sao lưu
(TKN-A-12); ô "Loại" dạng nút (TKN-A-13). Mục sự kiện không có tên người (TKN-A-08) là lỗi, không phải sai khác.

## 3. Windows (PR #25)

### 3.1 Mức chặn

#### TKN-W-01 — Ở A++ thanh trên cùng tràn: nút A++ và nút sáng/tối ra ngoài cửa sổ — **chặn**

- Màn hình: mọi trang (thanh trên cùng). Ảnh: `08-chu-lon-1x.png`, `08-chu-lon-2x.png` (chụp ở `Theme.scale = 1,32`, cửa sổ
  1180 px): sau "Giới thiệu" chỉ còn "A" và nửa chữ "A+"; nút "A++" (đang chọn) và nút sáng/tối nằm ngoài mép phải. Bản dựng
  Linux `winv02-08-chu-lon-1x.png` giống hệt.
- Hậu quả: người dùng đã chọn A++ không thấy nút đang chọn và **không bấm được nút đổi sáng/tối**; cỡ chữ lớn nhất dành đúng cho
  người dùng chính. Nguyên nhân: thêm mục thứ năm "Người thân" vào thanh mục — PWA đã gặp đúng lỗi này và sửa ở 08 §12b ("thanh
  mục chuyển xuống dòng riêng dưới thanh trên cùng khi A+/A++").
- Sửa: khi bề rộng không đủ (hoặc khi `scale > 1`), đưa thanh mục xuống hàng thứ hai dưới tên ứng dụng và nút cỡ chữ; hoặc bỏ
  chữ, chỉ giữ biểu tượng + tooltip ở A++. Đặt bề rộng tối thiểu cửa sổ phù hợp. Thêm kiểm thử `ui_tests` đo cạnh phải của nút
  sáng/tối ≤ bề rộng cửa sổ ở 1180 px với scale 1,0 / 1,16 / 1,32, và chụp lại `08-chu-lon`.

#### TKN-W-02 — Không có ảnh nào của mô-đun ở A+/A++ — **chặn** (chưa đủ bằng chứng)

- `--chup-man-hinh` chụp 23 màn hình Người thân chỉ ở `scale 1.0` (`shots.rs`: `nt-01` đặt lại `set_scale(1.0)`). 08 §6 cam
  kết "ở A++ … không tràn ngang; hàng ba ô ngày giữ nguyên, các nhóm nút xuống dòng". Giao diện Windows có nhiều kích thước cố
  định (ô ngày ~72 px, ô "Tự đặt", nhóm nút kỳ và bộ lọc một hàng, cột phải 320 px ở Sắp tới) — chưa thấy ở cỡ lớn.
- Sửa: chụp thêm ở scale 1,32 (1×) ít nhất `nt-02`, `nt-03`, `nt-05`, `nt-07`, `nt-08`, `nt-11b`, `nt-12`, `nt-13b`, `nt-15`,
  `nt-19`, `nt-20`, `nt-21`; xem lại sau khi sửa TKN-W-01.

### 3.2 Mức nên sửa

#### TKN-W-03 — Danh sách Hồ sơ ghi "Giỗ 15/6 nhuận âm lịch" — dễ hiểu sai tháng cúng — nên sửa

- Ảnh: `nt-07-ho-so-{sang,toi}-1x.png`, dòng "Ông Nguyễn Văn An": "Giỗ 15/6 **nhuận** âm lịch · lần tới 18/07/2027".
- Sai ở đâu: theo O-4 (08 L2, `Lists.kt` Android: "Giỗ hằng năm vào tháng thường kể cả khi mất vào tháng nhuận"), giỗ hằng năm
  là 15/6 **tháng thường**. Chữ "nhuận" trong dòng giỗ khiến người dùng nghĩ phải đợi tháng 6 nhuận (nhiều năm không có) hoặc cúng
  sai tháng. Android ghi "Giỗ 15/6 âm lịch" (`and-05`), đúng 08 §3.3.
- Sửa: dòng tóm tắt giỗ không ghi "nhuận"; "nhuận" chỉ ở dòng "Âm lịch" của bảng thông tin (ngày mất).

#### TKN-W-04 — Bia tưởng niệm: "Ngày giỗ năm nay: 15/6 âm lịch, 18/07/2027" — sai năm — nên sửa

- Ảnh: `nt-21-tuong-niem-{sang,toi}-1x.png` (hôm nay 07/11/2026). Mã: `people.rs` ~dòng 1214 luôn ghi "năm nay" cho mọi lần
  giỗ không phải hôm nay. Android ("Ngày giỗ sắp tới: …", `and-11`) và ảnh PWA (`v02-memorial-360.png`) đã ghi "sắp tới".
- Sửa: "Ngày giỗ năm nay" chỉ khi lần giỗ kế tiếp thuộc năm dương hiện tại; khác: "Ngày giỗ sắp tới: …". Sửa 08 §3.9 cho khớp
  (TKN-X-06).

#### TKN-W-05 — Khung "Còn n chỗ cần sửa" không liệt kê lỗi — nên sửa

- Ảnh: `nt-12-them-da-mat-loi-{sang,toi}-1x.png`: khung đầu biểu mẫu chỉ ghi "Còn 2 chỗ cần sửa"; thấy một lỗi ("Họ tên không
  được để trống"), lỗi thứ hai nằm ngoài màn hình và người dùng không biết ở đâu. 08 §2: khung tóm tắt có "liên kết tới từng
  trường". Ô có lỗi cũng không đổi viền.
- Sửa: liệt kê từng lỗi thành dòng bấm được (cuộn tới và đặt tiêu điểm vào ô); viền đỏ cho ô lỗi; cập nhật danh sách khi gõ (TK-03).

#### TKN-W-06 — Chữ nhỏ ở 100 % — nên sửa

- Ảnh: mọi `nt-*-1x`: chữ gợi ý dưới ô ("Ví dụ: Bố, Mẹ, Ông nội…"), dòng giải thích bộ lọc, câu O-9, "Sinh nhật âm"/"Ngày giỗ" dưới
  biểu tượng là 12–13 px; nội dung chính 14–15 px (mã: 32 chỗ `size: 14`, 16 chỗ `size: 13`, chữ loại `size: 12`). Với người
  35–70 tuổi ở màn hình 1080p 100 %, 12–13 px là khó đọc; PWA dùng chữ gốc 17 px.
- Sửa: tối thiểu 14 px cho chữ phụ, 15–16 px cho nội dung ở A; hoặc mặc định A+ cho mô-đun. Chụp lại sau sửa.

#### TKN-W-07 — Viền ô nhập và nút viền gần như không thấy — nên sửa

- Ảnh: `nt-11`, `nt-14`, `nt-16` (ô nhập chỉ còn một vạch mảnh ở đáy), nút "Sửa", "Xem", "Đã biết", "Hủy", các ô chọn chưa chọn
  (`nt-02`, `nt-08`, `nt-19`). Tính: `rule` trên `paper` 1,33:1 (cần ≥ 3:1). Ô trống trên thẻ trắng khó nhận ra là chỗ gõ được.
- Sửa: như TKN-A-04 — biến `outline` ≥ 3:1 cho viền ô và nút viền.

#### TKN-W-08 — Chữ phụ trên nền trang 3,99:1 — nên sửa

- Ảnh: "Đang xem kỳ có hôm nay", dòng giải thích bộ lọc, "n mục" (`nt-03…06-sang`) nằm trên `wall`. Như TKN-A-07: dùng `ink2`.

#### TKN-W-09 — Sai khác của Windows chưa ghi vào đặc tả 08 — nên sửa

- PR #25 liệt kê 6 sai khác trong mô tả PR và `docs/04-BUILD-NATIVE.md` §3.4a, nhưng không thêm mục vào 08 như Android (§13).
  Lời đầu của 08: "chỗ nào khi cài buộc phải đổi thì sửa tài liệu này trong cùng PR".
- Sửa: thêm "§14 Windows" vào 08 với bảng sai khác (gồm cả những điểm ở §3.4 dưới đây).

### 3.3 Góp ý

| ID | Màn hình / ảnh | Điều thấy | Đề xuất |
|---|---|---|---|
| TKN-W-10 | Mọi hộp kiểm (`nt-01`, `nt-11`, `nt-13b`, `nt-16`) | Ô đánh dấu màu xanh lam mặc định của Fluent (tối: xanh nhạt) lạc với tông đỏ của ứng dụng; PWA dùng `accent-color` đỏ | Đặt màu đánh dấu = `redDeep` (sáng) / `red-ink` (tối) |
| TKN-W-11 | Hồ sơ một người (`nt-08`, `nt-10`) | Một cột 760 px ở cửa sổ 1180 px; 08 §7 ghi hai cột ở máy tính | Chấp nhận được (dễ đọc hơn); ghi vào §14 |
| TKN-W-12 | Theo kỳ (`nt-03…05`) | Nhãn "Tiên thường — Giỗ · Bà Trần Thị Bình" khó đọc (hai dấu nối) | "Tiên thường · Bà Trần Thị Bình" (08 §5.1); kiểm Android cùng dữ liệu |
| TKN-W-13 | Sắp tới trống (`nt-01`) | Lần đầu mở đã thấy khung "Mặc định nhắc cho hồ sơ mới" với 6 nút và ô giờ — nặng cho người mới | Đưa vào trang Sao lưu/Cài đặt hoặc thu gọn sẵn |
| TKN-W-14 | Hộp thoại tối (`nt-15`, `nt-19`, `nt-20` tối) | Hộp và lớp phủ gần cùng màu, mép hộp khó thấy | Viền 1 px `outline` hoặc nền hộp sáng hơn ở tối |
| TKN-W-15 | Bộ ảnh | Thiếu: khói đang chạy với nút "Dừng khói" (`shots.rs` đặt `m_smoke(false)`), trạng thái thật của "Lời nhắc trên Windows" (đặt được / bản .zip không có AUMID / đang tắt) thay cho câu "Chế độ chụp màn hình…", thẻ nhắc trên Tờ lịch (nếu có), hộp "Tự nhập tuổi", hộp xem trước khôi phục và "Thay toàn bộ", lỗi khi đã nhập đủ ba ô ngày | Chụp thêm |
| TKN-W-16 | Ô nhập, nút trong thẻ nhắc | Ô nhập và nút cao ~32 px ở 1× (chuẩn Fluent cho chuột), dưới mức 44 px của 08 §6 | Chấp nhận cho máy tính dùng chuột; ghi vào §14; ở A++ kích thước tự tăng theo `Theme.scale` |
| TKN-W-17 | Tưởng niệm (`nt-21`, `nt-22`) | Khung ảnh hình viên thuốc (cạnh thẳng), 08 §3.9 ghi bầu dục; Android, PWA bầu dục | Khung bầu dục |

### 3.4 Các sai khác Windows đã nêu trong PR

| # | Sai khác | Đánh giá |
|---|---|---|
| 1 | "Lời nhắc trên Windows" thay §5.4; thêm "Mặc định nhắc cho hồ sơ mới" | **Chấp nhận.** Xem TKN-W-13 về chỗ đặt khung mặc định |
| 2 | Không có `datalist`: gợi ý cách gọi/quan hệ bằng dòng chú thích | **Chấp nhận tạm.** Từ gợi ý đúng từ thường ngày. Đề nghị đợt sau dùng nút chọn nhanh như Android (bấm thay vì gõ) |
| 3 | Kiểm lỗi ngày khi đủ ba ô hoặc sau lần Lưu đầu (không có sự kiện rời ô) | **Chấp nhận** — sau đó kiểm theo từng lần gõ như TK-03. Chưa có ảnh lỗi ngày (TKN-W-15) |
| 4 | TK-16: "Dữ liệu được khôi phục từ bản sao lưu ngày …" | **Chấp nhận** — đúng §12h |
| 5 | "Tự nhập tuổi" thay "Sửa tuổi"/"Tính lại" | **Chấp nhận** (cùng dữ liệu `manual`). Chưa có ảnh |
| 6 | Sự kiện cho một người | **Chấp nhận** — như PWA (08 §11) |

Sai khác **chưa** nêu mà bên duyệt thấy: một cột ở hồ sơ một người (TKN-W-11); "Ngày giỗ năm nay" (TKN-W-04, là lỗi); khung
tóm tắt lỗi không liệt kê (TKN-W-05); vùng bấm 32 px (TKN-W-16); thanh mục không xuống dòng ở A++ (TKN-W-01, là lỗi).

## 4. Nhất quán giữa các nền tảng và với PWA — góp ý

| ID | Điều thấy | Đề xuất (ghi vào 08 cho mọi nền tảng) |
|---|---|---|
| TKN-X-01 | Hồ sơ người đang sống: Android "Cập nhật: đã mất" trước "Sửa" (`and-09`), Windows và Apple "Sửa" trước (`nt-10`, `ios-person-living`). 08 §12g: nút phụ "cạnh Sửa" | "Sửa" trước, "Cập nhật: đã mất" sau, ở mọi nền tảng |
| TKN-X-02 | Năm trên bia: Android, Apple và PWA "Canh Thìn 1940 — Ất Tỵ 2025"; Windows "Canh Thìn (1940) — Ất Tỵ (2025)"; 08 §3.9 ghi dạng có ngoặc | Chọn một dạng, sửa 08 và hai bản còn lại |
| TKN-X-03 | Bàn thờ: nến đỏ (Android, Apple, PWA) / nến trắng (Windows); khung ảnh bầu dục (Android, Apple, PWA) / viên thuốc (TKN-W-17) | Thống nhất theo PWA và ghi chi tiết bàn thờ vào `content/memorial/PHONG-TUC.md` |
| TKN-X-04 | Hộp .ics: Android thêm "Không gồm thông tin báo tử", Windows và Apple (`ios-ics-confirm`) không | Đưa câu này vào 08 §3.8 A cho mọi nền tảng (R-PRIV-2 nói rõ cho người dùng) |
| TKN-X-05 | Nút chọn lịch ở biểu mẫu sự kiện: Windows và Apple "Dương lịch / Âm lịch" (`nt-16`, `ios-event`), biểu mẫu người và Android "Nhập theo dương lịch / Nhập theo âm lịch" | Dùng một cặp chữ trong toàn mô-đun |
| TKN-X-06 | 08 §3.9 vẫn ghi "Ngày giỗ năm nay: …" cho mọi trường hợp không phải hôm nay. Ba bản cài ghi ba cách: Windows "Ngày giỗ năm nay" (sai năm, TKN-W-04), Android "Ngày giỗ sắp tới", Apple "Ngày giỗ tới" (A-4) | Sửa 08: "năm nay" chỉ khi lần giỗ kế tiếp trong năm dương hiện tại, khác "Ngày giỗ sắp tới: …"; Apple đổi "tới" → "sắp tới" |
| TKN-X-07 | Biểu tượng loại: PWA, Android, Windows dùng ba nén hương / bánh có nến / ngôi sao; Apple dùng ngọn lửa (giỗ), hộp quà (sinh nhật), dao nĩa (tiên thường) (A-5) | Đợt sau vẽ biểu tượng riêng (nén hương, bánh) cho Apple như các nền tảng khác |

## 5. Apple (PR #23)

Duyệt bản `89dedb3` (CI xanh, ảnh CI lần 121). Đã xem cả 102 ảnh: iPhone ghép sáng | tối | Accessibility L của cùng màn hình (27 bộ),
ảnh luồng `ios-flow-added-light`, và 20 ảnh iPad (ghép ba ảnh một). Đối chiếu mã ở `apps/apple/Shared/Theme.swift`,
`RootView.swift`, `People/MemorialView.swift`; tương phản tính từ mã màu và đo điểm ảnh trên ảnh tối.

Nhận xét chung: bản Apple **gần PWA nhất** về nội dung và câu chữ, và là bản **xử lý chữ lớn tốt nhất** trong ba bản cài: ở
Accessibility L mọi tiêu đề, nút viên thuốc, bộ lọc ("Người đang sống" xuống hàng riêng, không bị cắt — `ios-period-large`), dòng
mục, hộp xác nhận (`ios-delete-confirm-large`, `ios-mark-confirm-large`) đều to lên và xuống dòng đúng; thanh thẻ đáy giữ đủ 5 nhãn.
Theo HIG hợp lý: thanh thẻ đáy, `NavigationStack` với nút quay lại ghi tên trang trước, `alert` cho xác nhận (nút an toàn in đậm),
tấm `sheet` cho .ics và sao lưu, `Toggle`, `Picker` phân đoạn, gợi ý cách gọi/quan hệ là hàng nút cuộn ngang dưới ô (danh sách đúng từ
thường ngày: Cụ, Kỵ, Ông, Bà… / Bố, Mẹ, Ông nội…). Quyết định của chủ dự án được giữ: bia "Ông nội / Nguyễn Văn A", "Bà ngoại /
Trần Thị B" (bỏ ngoặc "(bên nội)", không lặp cách gọi, không chữ Hán-Việt cổ); B4 (`ipad-event-deceased-light`: khối "Nhắc" thay
bằng câu giải thích; hồ sơ người đã mất có câu "Không nhắc sinh nhật và sự kiện…"; hộp "Cập nhật: đã mất" ghi "Sẽ không còn nhắc
sinh nhật…"); thông tin báo tử chỉ ở khối thu gọn trên trang hồ sơ (`ios-person-notice-*`), không có trên bia, tấm .ics, Sắp tới;
"Hiện tên người trên màn hình khóa" mặc định tắt, có câu giải thích (`ios-settings-*`). Tưởng niệm trang nghiêm: nền gỗ tối, ảnh
bầu dục viền vàng đồng, bia vòm chữ serif vàng, khói mờ, bàn thờ nến đỏ – bát hương ba nén – mâm quả (`ipad-memorial*`), có
"Dừng khói" ngay dưới dòng ngày (TK-25).

Điểm yếu chính: **giao diện tối dùng màu nhấn đỏ sẫm cho chữ** nên nút quay lại, Hủy/Lưu, nút trong hộp xác nhận gần như không đọc
được; và bộ ảnh, như Android, **không chụp ô ngày khi đang nhập**.

### 5.1 Mức chặn

#### TKN-I-01 — Giao diện tối: chữ màu nhấn (quay lại, Hủy/Lưu, nút hộp xác nhận, ô chọn) chỉ 1,3–1,7:1 — **chặn**

- Ảnh: mọi ảnh `-dark` có thanh điều hướng — `ios-person-*-dark` ("‹ Người thân"), `ios-form-*-dark` và `ios-event*-dark` ("Hủy",
  "Lưu"), `ios-event-dark` (ô "Loại": "Tốt nghiệp ◇"), `ios-ics-confirm-dark` ("Hủy", "Lưu / chia sẻ tệp .ics"),
  `ios-backup-export-dark` ("Tạo và lưu tệp"), `ios-mark-confirm-dark` ("Quay lại", "Lưu thay đổi"), `ios-delete-confirm-dark`
  ("Giữ lại"), `ios-person-notice-dark` (mũi tên thu gọn).
- Sai ở đâu (mã + tính + đo): `RootView.swift` đặt `.tint(C.redDeep)` cho cả ứng dụng; `redDeep` ở tối là `#7A1214` — màu dành cho
  **nền** nút đặc (chữ trắng trên nó 10,9:1). Dùng làm màu **chữ**: trên nền `#141011` **1,73:1** (đo điểm ảnh nút quay lại: `#7E1214`
  trên `#141011`), trên nền hộp `alert` tối ≈ **1,3:1**. Cần ≥ 4,5:1. Hậu quả với người 35–70 tuổi bật giao diện tối: không thấy nút
  **Lưu** của biểu mẫu, không đọc được nút "Lưu thay đổi"/"Quay lại" của hộp "Cập nhật: đã mất" và nút "Giữ lại" của hộp xóa — đúng
  những chỗ cần đọc kỹ trước khi bấm.
- Sửa: màu nhấn chung = `C.redInk` (sáng `#B3151B` 5,4:1 trên `wall`; tối `#FF8A83` 8,3:1), hoặc một màu nhấn riêng có biến thể tối
  sáng hơn; giữ `redDeep` chỉ cho nền nút đặc. Kiểm lại cả `Picker`, `Toggle`, `alert`, `sheet` toolbar. Chụp lại toàn bộ ảnh `-dark`.

#### TKN-I-02 — Bộ ảnh không có ô ngày khi đang nhập, trạng thái lỗi, khôi phục — **chặn** (chưa đủ bằng chứng)

- Ảnh có: mọi ô Ngày/Tháng/Năm đều **trống** (`ios-form-deceased-2`, `ios-form-living-2`, `ios-event`, `ios-mark`, iPad cùng tên).
  Không ảnh nào có: chế độ "Nhập theo âm lịch" với hộp "Tháng nhuận (năm 2025 nhuận tháng 6)" / câu "Năm 2026 không có tháng nhuận" /
  "Không rõ năm mất"; **dòng kết quả đổi lịch**; lỗi dưới ô và khung "Còn n chỗ cần sửa" (A-3 đổi cách hiện lỗi nhưng không có ảnh);
  tấm khôi phục (Gộp / Thay toàn bộ) và hộp xác nhận thứ hai; trạng thái lỗi kho; Sắp tới khi quyền thông báo bị từ chối; nội dung
  thông báo trên màn hình khóa; bàn thờ trên iPhone (ảnh tưởng niệm iPhone dừng ở khói, chỉ iPad thấy bàn thờ).
- Vì sao chặn: như TKN-A-02 — R-UX-1 (rủi ro "Cao" của 07) và US-M-01/02 không duyệt được trên iPhone. Đây là thiếu bằng chứng,
  không phải đã thấy lỗi.
- Sửa: dùng tham số `-lvhvScreen` (A-11) chụp thêm, ở sáng/tối/Accessibility L: `form-deceased-am-nhuan` (15/6/2025, nhuận, kết quả
  "Dương lịch: Thứ Sáu, 08/08/2025"), `form-deceased-duong-ket-qua`, `form-deceased-loi` (lưu khi trống), `form-living-sinh-nhat-am`,
  `upcoming-thong-bao-tat`, `store-error`, `backup-nag`, `restore-preview`, `restore-replace-confirm`, `memorial-ban-tho` (cuộn xuống),
  và một ảnh thông báo mặc định không tên.

### 5.2 Mức nên sửa

#### TKN-I-03 — Tưởng niệm: tiêu đề thanh điều hướng không thấy ở giao diện sáng; nút "‹ Hồ sơ" 1,9:1 — nên sửa

- Ảnh: `ios-memorial-light`, `ios-memorial-a-light`, `ios-memorial-large`, `ios-memorial-a-large`, `ipad-memorial-light`,
  `ipad-memorial-a-light` — chữ "Tưởng niệm" màu tối trên thanh nền gỗ tối, gần như vô hình (ở ảnh `-dark` thì đọc được). Mã:
  `MemorialView.swift` có `.toolbarColorScheme(.dark, for: .navigationBar)` nhưng ảnh cho thấy không có tác dụng khi máy ở sáng.
  "‹ Hồ sơ" màu `redDeep` sáng `#8F1416` trên nền `#1E1714`: 1,92:1.
- Sửa: đặt màu chữ tiêu đề và nút quay lại của màn hình này tường minh (vàng đồng `#E8C67A`, ≈ 9:1), hoặc bỏ tiêu đề (trang đã có
  dòng ngày giỗ). Chụp lại ở sáng.

#### TKN-I-04 — Danh sách Hồ sơ ghi "Giỗ 15/6 nhuận âm lịch" — nên sửa

- Ảnh: `ios-profiles-*`, `ipad-profiles-light` ("Ông Nguyễn Văn A … Giỗ 15/6 nhuận âm lịch · lần tới 18/07/2027"). Như TKN-W-03: giỗ
  hằng năm vào tháng **thường** (O-4); chữ "nhuận" có thể làm gia đình cúng sai tháng. Sửa: bỏ "nhuận" khỏi dòng giỗ; giữ ở dòng
  "Âm lịch" của bảng thông tin.

#### TKN-I-05 — Accessibility L: điều khiển phân đoạn hệ thống không to theo — nên sửa

- Ảnh: `ios-*-large` — thanh trang con "Sắp tới / Theo kỳ / Hồ sơ / Sao lưu", "Nhập theo dương lịch / Nhập theo âm lịch",
  "Dương lịch / Âm lịch", "Biết đủ ngày / Chỉ biết năm" và nhóm "A / A+ / A++" giữ cỡ ~13 pt trong khi mọi chữ khác đã gấp đôi
  (`ios-form-deceased-large-2`, `ios-period-large`). `Picker(.segmented)` của UIKit không theo Dynamic Type; người dùng chữ lớn vẫn
  phải đọc chữ nhỏ đúng ở chỗ chọn dương/âm (R-UX-1).
- Sửa: thay bằng nhóm nút viên thuốc như "Tuần/Tháng/Quý/Năm" (đã có `isSelected`, đã to theo chữ) khi `dynamicTypeSize ≥ .xLarge`,
  hoặc dùng cho mọi cỡ để thống nhất.

#### TKN-I-06 — Ô nhập gần như không có viền ở giao diện sáng — nên sửa

- Ảnh: `ios-form-*-light`, `ios-mark-*-light`, `ios-backup-export-light`, iPad: ô nhập trắng `#FFFFFF` trên thẻ `#FDFCFA`, viền mảnh
  `rule` (1,33:1). Ô trống khó nhận ra là chỗ gõ được. (Ở tối, ô đen tuyền trên thẻ `#211A1B` thấy rõ.) Như TKN-A-04/TKN-W-07.
- Sửa: viền ô ≥ 3:1 với nền thẻ (ví dụ `#8F807D`), hoặc nền ô `paper2` + viền.

#### TKN-I-07 — Sai khác A-10: Hủy ở hộp "Lưu" vẫn tính là đã sao lưu — nên sửa (trước phát hành)

- Hậu quả cho người dùng: bấm "Tạo và lưu tệp" rồi Hủy ở hộp chọn nơi lưu → trang Sao lưu ghi "Lần sao lưu gần nhất: hôm nay" và tắt
  nhắc sao lưu, trong khi **không có tệp nào**. Người dùng tin dữ liệu đã an toàn. Đây là thông tin sai về an toàn dữ liệu (cùng loại
  với TK-16).
- Sửa: chờ `mark_backup_saved` của lõi như A-10 ghi; cho tới lúc đó, nếu được, giữ trạng thái "đã sao lưu" ở giao diện chỉ khi
  `fileExporter` báo thành công. Không phát hành v0.2 khi A-10 còn mở.

#### TKN-I-08 — Sai khác A-6/A-7: ảnh giấy báo tử đã "Bỏ" vẫn còn trên máy — nên sửa (trước phát hành)

- Giao diện cho người dùng bấm "Bỏ ảnh" ảnh giấy báo tử (R-PRIV-2: thông tin nhạy cảm) nhưng ảnh vẫn nằm trong kho cho tới khi xóa
  hồ sơ, và còn đi vào tệp sao lưu (cần xác nhận với vai lõi). Người dùng nghĩ đã xóa giấy tờ. A-6 (lưu hỏng để lại ảnh mồ côi) cùng
  gốc.
- Sửa: vai lõi thêm dọn ảnh không còn ai dùng khi `save_person_json` (hoặc FFI xóa ảnh); cho tới đó, giao diện ghi rõ "Ảnh sẽ được xóa
  khỏi máy khi xóa hồ sơ" thay vì để người dùng hiểu là đã xóa. Không phát hành v0.2 khi A-7 còn mở.

#### TKN-I-09 — "Cập nhật: đã mất" còn nút quay lại hệ thống, không có Hủy/Lưu trên thanh — nên sửa (sai khác chưa ghi)

- Ảnh: `ios-mark-*`, `ipad-mark-light*` — thanh điều hướng có "‹ Hồ sơ" (vuốt/bấm là rời trang, mất dữ liệu đang nhập, kể cả ảnh giấy
  báo tử đã chọn), trong khi §12-Apple ghi "nút quay lại của hệ thống ẩn ở biểu mẫu để không mất dữ liệu đang nhập" và các biểu mẫu
  khác (`ios-form-*`, `ios-event*`) có Hủy / Lưu.
- Sửa: dùng cùng thanh Hủy / Lưu và hộp "Bỏ các thay đổi chưa lưu?" như biểu mẫu khác; chặn vuốt quay lại khi đã sửa
  (`interactiveDismissDisabled` / ẩn nút quay lại).

### 5.3 Góp ý

| ID | Màn hình / ảnh | Điều thấy | Đề xuất |
|---|---|---|---|
| TKN-I-10 | Tấm .ics (`ios-ics-confirm-large`) | Ở chữ lớn tiêu đề tấm bị rút "Xuất tệp…" vì nút phải "Lưu / chia sẻ tệp .ics" quá dài; thiếu câu "Không gồm thông tin báo tử" (TKN-X-04) | Nút phải "Lưu tệp…" hoặc đưa nút chính xuống cuối tấm (nút lớn rộng hết khung); thêm câu báo tử |
| TKN-I-11 | iPad (`ipad-*`) | Bố cục điện thoại kéo rộng: trang hồ sơ, Theo kỳ, Sắp tới một cột rộng ~800 pt, chữ ~13 pt so với khoảng trống lớn; 08 §7 ghi hai cột ở khổ rộng (ví dụ Sắp tới: danh sách + cột phải; hồ sơ: Lần tới + Thông tin trái, sự kiện + nhắc phải) | Đợt sau dùng `NavigationSplitView` hoặc lưới hai cột khi `horizontalSizeClass == .regular`; giới hạn bề rộng nội dung ~700 pt |
| TKN-I-12 | Mục sự kiện (`ios-upcoming`, `ios-period*`) | Biểu tượng SF Symbols (A-5): ngọn lửa cho giỗ, hộp quà cho sinh nhật — khác ba nền tảng kia (TKN-X-07). Dòng 1 "Giỗ Bà Trần Thị B" không có "·" như 08 §2 ("Giỗ · Bà Trần Thị B") | Vẽ biểu tượng riêng (nén hương, bánh); thống nhất dấu "·" |
| TKN-I-13 | Tưởng niệm (`ios-memorial-a-*`) | Dòng ngày ghi "Ngày giỗ tới: …" (A-4) — đúng ý nhưng là cách viết thứ ba (TKN-X-06) | "Ngày giỗ sắp tới: …" như Android |

### 5.4 Các sai khác ghi ở 08 §12-Apple

| Mã / chỗ | Sai khác | Đánh giá |
|---|---|---|
| HIG §1.1, §1.2 | Thanh thẻ đáy 5 ô; `NavigationStack`, đường đi chỉ có id; nút quay lại ghi tên trang trước | **Chấp nhận** — đúng HIG, giữ S4'. Màu nút quay lại: TKN-I-01 |
| HIG §2 hộp xác nhận | `alert` (nút an toàn là nút hủy) cho xóa, bỏ thay đổi, "Cập nhật: đã mất", "Thay toàn bộ"; `sheet` cho .ics, sao lưu, khôi phục | **Chấp nhận.** Ảnh xác nhận "Giữ lại"/"Quay lại" in đậm là nút mặc định — đúng TK-08 tinh thần tiêu điểm an toàn. Màu ở tối: TKN-I-01 |
| HIG §2 ô nhập, `datalist` | Nhãn phía trên ô; gợi ý là hàng nút cuộn ngang | **Chấp nhận** — tốt cho người lớn tuổi (như Android). Ở chữ lớn chỉ thấy 4–5 nút đầu, không có dấu hiệu cuộn được — nên thêm mờ dần ở mép phải |
| HIG §2 nhóm nút | Nút viên thuốc `isSelected`; `Picker` phân đoạn; `Toggle` | **Chấp nhận có điều kiện** — `Picker` phân đoạn không to theo chữ (TKN-I-05) |
| HIG §3.4–§3.7 | Hủy / Lưu trên thanh + hai nút cuối biểu mẫu; ẩn nút quay lại | **Chấp nhận** — trừ biểu mẫu "Cập nhật: đã mất" chưa làm (TKN-I-09) |
| HIG §3.8 | `ShareLink` "Lưu / chia sẻ tệp .ics", tệp tạm bị xóa; `fileExporter`/`fileImporter` | **Chấp nhận** (TKN-I-10 về nhãn) |
| HIG §5.2 | Thẻ thông báo ngắn tự ẩn 6 giây, có nút đóng | **Chấp nhận** — chưa có ảnh (TKN-I-02) |
| HIG §5.4 | Không áp dụng; trang Sao lưu nói .ics là tùy chọn | **Chấp nhận** — câu "Ứng dụng đã tự nhắc bằng thông báo trên máy này…" rõ ràng (`ios-backup-*`) |
| HIG §6 cỡ chữ | A/A+/A++ cộng Dynamic Type | **Chấp nhận** — làm tốt (trừ TKN-I-05) |
| HIG §6 giảm chuyển động | Khói tĩnh, không nút | **Chấp nhận** |
| A-1 | Không thẻ nhắc trên Tờ lịch | **Chấp nhận** — như Android |
| A-2 | Trang Cài đặt nhắc (nút chuông) | **Chấp nhận** — câu chữ rõ; "Hiện tên người trên màn hình khóa" mặc định tắt (`ios-settings-*`). Nút chuông chỉ là biểu tượng: cần `accessibilityLabel` "Cài đặt nhắc"; cân nhắc ghi chữ như Android ("Cài đặt nhắc") vì người lớn tuổi khó đoán biểu tượng |
| A-3 | Lỗi hiện khi trường đã sửa và khi Lưu; "Hãy nhập đủ…" chỉ khi Lưu | **Chấp nhận** (tránh báo lỗi khi đang gõ dở) — chưa có ảnh (TKN-I-02) |
| A-4 | "Ngày giỗ tới: …" | **Chấp nhận về ý** (đúng hơn "năm nay"); đổi chữ thành "Ngày giỗ sắp tới" để thống nhất (TKN-I-13, TKN-X-06) |
| A-5 | SF Symbols: ngọn lửa, dao nĩa, hộp quà, ngôi sao, luôn kèm chữ | **Chấp nhận tạm** cho đợt này (luôn có chữ loại kèm theo); đợt sau vẽ biểu tượng riêng (TKN-X-07) |
| A-6, A-7 | Ảnh lưu hai giao dịch; ảnh đã bỏ chưa bị xóa khỏi máy | **Chấp nhận tạm có điều kiện** — phải xong trước phát hành; với ảnh giấy báo tử là vấn đề riêng tư người dùng thấy được (TKN-I-08) |
| A-8 | Không `BGAppRefreshTask` | **Chấp nhận** — không ảnh hưởng giao diện; thông báo nhắc mở ứng dụng có câu giải thích ở Cài đặt nhắc (`ipad-settings-light`) |
| A-9 | TK-16 chưa làm | **Chấp nhận tạm** — cần làm trước phát hành để thống nhất với PWA, Android, Windows |
| A-10 | Hủy ở hộp Lưu vẫn tính là đã sao lưu | **Không chấp nhận cho phát hành** — chấp nhận để gộp nếu có việc có hạn trước v0.2 (TKN-I-07) |
| A-11 | Tham số khởi chạy chỉ ở bản Debug | **Chấp nhận** — dùng nó để chụp thêm ảnh (TKN-I-02) |

Sai khác **chưa** ghi ở §12-Apple mà bên duyệt thấy: biểu mẫu "Cập nhật: đã mất" còn nút quay lại (TKN-I-09); màu nhấn ở tối
(TKN-I-01, là lỗi); bố cục iPad một cột (TKN-I-11); dòng 1 mục thiếu "·" (TKN-I-12).

## 6. Chấm theo docs/08 §5 (trên ảnh của từng nền tảng)

| Mục §5 | Android | Windows | Apple |
|---|---|---|---|
| Kiến trúc thông tin, điều hướng | Đạt (chưa thấy thanh đáy — TKN-A-02) | Đạt; **tràn ở A++** (TKN-W-01) | Đạt (thanh thẻ đáy, NavigationStack); TKN-I-09 |
| Từng màn hình: mục đích, lối vào | Đạt | Đạt | Đạt |
| Trường, kiểm tra, lời báo lỗi | **Chưa đủ bằng chứng** (TKN-A-02) | Gần đạt (TKN-W-05) | **Chưa đủ bằng chứng** (TKN-I-02) |
| Trạng thái trống / tải / lỗi / xác nhận | Đạt (trống, lỗi kho, hộp xóa, .ics, sao lưu); thiếu ảnh khôi phục | Đạt; thiếu ảnh khôi phục | Đạt (trống, mọi hộp xác nhận); thiếu ảnh lỗi kho, khôi phục |
| Câu chữ | Gần đạt (TKN-A-08) | Gần đạt (TKN-W-03, TKN-W-04) | Gần đạt (TKN-I-04, TKN-X-06) |
| Trợ năng: tương phản | Chưa đạt (TKN-A-04…07) | Chưa đạt (TKN-W-07, TKN-W-08) | **Chưa đạt ở tối** (TKN-I-01, TKN-I-03, TKN-I-06) |
| Trợ năng: cỡ chữ | **Chưa đạt** (TKN-A-01, TKN-A-03, TKN-A-09) | **Chưa đủ bằng chứng / chưa đạt** (TKN-W-01, TKN-W-02, TKN-W-06) | Gần đạt — chữ lớn tốt nhất trong ba bản, trừ điều khiển phân đoạn (TKN-I-05) |
| Sáng / tối | Đạt (trừ TKN-A-06) | Đạt (góp ý TKN-W-14) | **Chưa đạt** ở tối (TKN-I-01) |
| Quyết định của chủ dự án (từ ngữ, B4, báo tử, tên màn hình khóa) | Đạt | Đạt | Đạt (TKN-I-08 về ảnh giấy báo tử đã bỏ) |
| Tưởng niệm trang nghiêm | Đạt | Đạt (TKN-W-04 về câu ngày) | Đạt (TKN-I-03 về thanh tiêu đề) |

## 7. Kết luận

| Nền tảng | Chặn | Nên sửa | Góp ý | Kết luận |
|---|---|---|---|---|
| Android (#24) | 2 | 7 | 5 | **DUYỆT CÓ ĐIỀU KIỆN** |
| Windows (#25) | 2 | 7 | 8 | **DUYỆT CÓ ĐIỀU KIỆN** |
| Apple (#23) | 2 | 7 | 4 | **DUYỆT CÓ ĐIỀU KIỆN** |
| Chung (§4) | 0 | 0 | 7 | — |

Tổng: **6 chặn, 21 nên sửa, 24 góp ý.**

Điều kiện để gộp:

**Android**
1. **TKN-A-01** — Bỏ `maxLines = 2` của nhãn nhóm nút; ở chữ lớn xếp nhóm kỳ 2 × 2 và bộ lọc ba hàng như PWA A++; kiểm thử không
   tràn chữ ở `fontScale` 1,3 và 2,0; chụp lại 02–05 (thêm biến thể 2,0).
2. **TKN-A-02** — Bổ sung ảnh: ô ngày âm/nhuận có kết quả, ô ngày dương có kết quả, biểu mẫu có lỗi + khung tóm tắt, sinh nhật âm,
   thanh đáy, thẻ "thông báo đang tắt", nhắc sao lưu, hộp khôi phục và "Thay toàn bộ", khói đang chạy + "Dừng khói", nội dung thông
   báo; chụp toàn bộ biểu mẫu. Bên duyệt xem lại bộ ảnh mới trước khi gộp.

**Windows**
1. **TKN-W-01** — Thanh mục xuống hàng riêng (hoặc chỉ biểu tượng) khi không đủ chỗ ở A+/A++; kiểm thử cạnh phải nút sáng/tối ≤
   bề rộng cửa sổ ở scale 1,0 / 1,16 / 1,32; chụp lại `08-chu-lon`.
2. **TKN-W-02** — Chụp các màn hình Người thân chính ở A++ (scale 1,32); bên duyệt xem lại.

**Apple**
1. **TKN-I-01** — Đổi màu nhấn chung `.tint(C.redDeep)` (`RootView.swift`) thành `C.redInk` (tối `#FF8A83`, 8,3:1); giữ `redDeep` chỉ làm
   nền nút đặc. Chụp lại mọi ảnh `-dark` (biểu mẫu, hộp xác nhận, tấm .ics/sao lưu, hồ sơ) và ảnh tưởng niệm.
2. **TKN-I-02** — Bổ sung ảnh (sáng, tối, Accessibility L): ô ngày âm + "Tháng nhuận (năm 2025 nhuận tháng 6)" + kết quả, ô ngày dương +
   kết quả, biểu mẫu có lỗi (A-3), Sắp tới khi quyền thông báo bị từ chối, lỗi kho, nhắc sao lưu, tấm khôi phục và hộp "Thay toàn bộ",
   tưởng niệm cuộn tới bàn thờ, nội dung thông báo màn hình khóa. Bên duyệt xem lại trước khi gộp.

Các mục nên sửa: sửa trong đợt này hoặc mở việc có hạn trước phát hành v0.2; ưu tiên TKN-W-03 (chữ "nhuận" có thể làm cúng sai
tháng; cùng lỗi ở Apple TKN-I-04), TKN-W-04, TKN-A-08, TKN-I-07 và TKN-I-08 (sai khác A-10, A-7: không phát hành khi còn mở), TKN-A-04/TKN-W-07 (viền điều khiển), TKN-A-06. Thử trên máy thật (T11) nên kiểm thêm: chữ lớn 2,0 trên
điện thoại Android thật; Windows ở 125 %/150 % và A++ trên màn hình 1366 × 768; iPhone thật ở giao diện tối và cỡ chữ AX3–AX5.
