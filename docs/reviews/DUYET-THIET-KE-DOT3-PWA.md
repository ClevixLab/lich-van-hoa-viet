# Duyệt thiết kế — mô-đun "Người thân" trên PWA (v0.2 đợt 3, PR #15)

Vai: **bên duyệt thiết kế** (việc T5 trong `docs/v0.2/07-PHAN-TICH-YEU-CAU-DOT-3.md` §7). Bên duyệt thiết kế không viết
đặc tả giao diện và không viết mã của PR này. Ngày duyệt: 2026-11-07.

Đối tượng duyệt: nhánh `v0.2-w3-pwa`, bản dựng `dbf215b` (PR #15).
Căn cứ: `docs/08-QUY-TRINH-MO-RONG-TINH-NANG.md` §5–6; đặc tả `docs/v0.2/08-DAC-TA-GIAO-DIEN-NGUOI-THAN.md` (trên nhánh
PR, gọi tắt là "08"); `docs/v0.2/07` (câu chuyện US-M/US-S/US-C, rủi ro); `docs/v0.2/01`, `02`, `04`.

## 0. Cách duyệt

1. Đọc toàn bộ 08 (kể cả §12 "Sai khác"), 07, docs/08 §5–6, và mã `pwa/src/people/*`, `pwa/src/style.css`,
   `pwa/test/people-flows.mjs`.
2. Xem **từng** ảnh trong 58 ảnh `v02-*.png` do kiểm thử khói sinh ra (360 px và 1280 px, sáng và tối, luồng, hộp thoại,
   A++). Ảnh dài được cắt thành đoạn để xem đủ chi tiết.
3. Tự dựng PWA (`npm ci && npm run build`), phục vụ `dist/` và chạy các luồng bằng Chromium không giao diện:
   khung nhìn 360×780 (tỉ lệ điểm ảnh 2, chế độ điện thoại, có cảm ứng), 1280×800; `vi-VN`, múi giờ Hà Nội; đồng hồ trang
   đặt 25/07/2026 (còn 3 ngày tới giỗ đầu) và 28/07/2026 (đúng ngày giỗ); có và không có "giảm chuyển động"; cỡ chữ A
   và A++. Đo: chiều rộng cuộn so với chiều rộng khung nhìn, vị trí thanh mục đáy so với phần cuối trang, kích thước vùng
   chạm của mọi phần tử bấm được, độ tương phản màu chữ/nền (WCAG), thứ tự Tab, tiêu điểm khi mở trang, hộp thoại và Esc.
   Kịch bản đo là công cụ riêng của bên duyệt, không đưa vào kho mã.

Chỗ nào dưới đây ghi "chạy thử" là đã kiểm trên bản dựng đang chạy; chỗ ghi "ảnh" là thấy trên ảnh của PR.

Mức lỗi: **chặn** (phải sửa trước khi gộp) · **nên sửa** (sửa trong đợt này hoặc ghi vào sổ việc có hạn) · **góp ý**.

## 1. Nhận xét chung

Điểm mạnh thật sự:
- Kiến trúc thông tin đúng nhu cầu: một mục "Người thân" với bốn trang con; "Sắp tới" làm trang đầu trả lời ngay câu
  "tuần này nhà có việc gì"; danh sách theo kỳ trộn người đã mất và đang sống, có bộ lọc.
- Ô nhập ngày dương/âm có **kết quả đổi lịch trực tiếp** ("Âm lịch: 15/6 nhuận năm Ất Tỵ (2025)"); hộp "Tháng nhuận"
  chỉ hiện khi năm đó có tháng nhuận và nhãn nói rõ "năm 2025 nhuận tháng 6"; câu lỗi bằng lời thường ("Tháng 7 âm lịch
  năm 2026 chỉ có 29 ngày") — đúng R-UX-1. Chạy thử: đổi năm từ 2025 sang 2026 sau khi đã đánh dấu nhuận thì hộp ẩn và
  giá trị nhuận bị bỏ qua đúng cách.
- Danh sách theo kỳ: bốn loại kỳ, tiêu đề kỳ đúng định dạng, nhóm theo tháng ở Quý/Năm (R-UX-2), câu trống có ghi bộ
  lọc đang dùng, `aria-live` cho tiêu đề và số mục, thứ tự Tab hợp lý (thanh phân đoạn → loại kỳ → bộ lọc → ‹ › → danh
  sách), viền tiêu điểm 3 px rõ.
- Màn hình tưởng niệm trang nghiêm: nền gỗ tối ấm, bia vòm chữ vàng đồng (tương phản ≈ 11–15:1 đo trên nền tối), ảnh
  khung bầu dục, mâm quả, bát hương ba nén, hai nến, bình hoa cúc; không âm thanh, không chia sẻ; tiêu đề tab không có
  tên người; khói dừng sau 2 phút, có nút "Dừng khói"/"Thắp lại", giảm chuyển động thì khói tĩnh và ẩn nút. Ngày giỗ
  hiện "Giỗ đầu · hôm nay 15/6 âm lịch". Không có ảnh thì khung hiện chữ cái đầu của tên — trang trọng, không trống trải.
- Hộp xóa đặt tiêu điểm vào "Giữ lại"; hộp "Bỏ các thay đổi chưa lưu?" đặt tiêu điểm vào "Tiếp tục sửa"; Esc đóng hộp.
- Dấu tiếng Việt hiển thị đúng ở cả hai họ chữ (serif cho tiêu đề, sans cho nội dung), kể cả chữ in hoa giãn cách
  "HIỂN KHẢO" và các dấu chồng (ễ, ỗ, ở, ự) ở cỡ A++. Không thấy chữ bị cắt dấu ở dòng trên.
- Bố cục 1280 px của "Sắp tới", "Hồ sơ", "Hồ sơ một người", "Sao lưu" dùng hai cột hợp lý.
- Thanh mục đáy cố định **không** che nội dung cuối trang ở mọi trang mới (chạy thử, cỡ A: chân trang kết thúc ở
  704 px, thanh mục bắt đầu ở 707 px), **trừ** trang biểu mẫu người đã mất (xem TK-01). Ở A++ thanh mục cao 82 px và che
  6 px dưới cùng của chân trang (dòng "Phiên bản" vẫn đọc được) — không ảnh hưởng nội dung mô-đun.

Điểm yếu chính: **biểu mẫu người đã mất làm trang tràn ngang** ở mọi khổ màn hình, và **phép kiểm tràn ngang của kiểm thử
khói không phát hiện được lỗi này** (ảnh 360 px của PR rộng 395 px CSS mà kiểm thử vẫn báo đạt). Ngoài ra còn một loạt
chỗ chưa thân thiện với người lớn tuổi: lỗi trường không tự mất khi đã sửa, vùng chạm dưới 44 px, ô hộp kiểm bị co nhỏ,
nhãn "Th 7" dễ đọc thành "Thứ Bảy", ô chọn tệp gốc của trình duyệt.

## 2. Danh sách phát hiện

### 2.1 Mức chặn

#### TK-01 — Trang "Thêm/Sửa người đã mất" tràn ngang; thanh mục đáy bị đẩy ra khỏi màn hình — **chặn**

- Màn hình: `#/nguoi-than/them/da-mat`, `#/nguoi-than/sua/<id>`. Ảnh: `v02-form-deceased-360.png` (rộng 790 điểm ảnh =
  395 px CSS thay vì 360), `v02-form-deceased-360-light/-dark.png`, `v02-form-deceased-360-xl.png` (401 px),
  `v02-form-deceased-1280-light/-dark.png` (rộng 1594 px CSS thay vì 1280).
- Sai ở đâu (chạy thử): `scrollWidth` = 395 ở 360 px (A), 401 ở A++, 1594 ở 1280 px. Phần tử gây tràn là ô chọn ảnh ẩn
  `input#pf-file.sr-only` rộng 360 px (360) / 1280 px (1280). Nguyên nhân: luật `.people input[type="file"] { width: 100% }`
  (style.css dòng 813) có độ ưu tiên cao hơn `.sr-only { width: 1px }` nên ô "ẩn" vẫn rộng bằng khung. Hậu quả trên điện
  thoại: trình duyệt nới khung nhìn bố cục lên 395 px, trang bị thu nhỏ hoặc cuộn ngang được, và **thanh mục đáy nằm dưới
  mép màn hình** (chạy thử: đỉnh thanh mục ở 783 px trong khung 780 px — không thấy thanh mục); trên máy tính có thanh
  cuộn ngang 314 px. Đây là biểu mẫu chính của câu chuyện US-M-01/02/06, người dùng 35–70 tuổi sẽ gặp ngay lần đầu.
- Sửa: đổi luật thành `.people input[type="file"]:not(.sr-only) { … }` **và** làm `.sr-only` bền hơn
  (`width:1px!important; height:1px!important; margin:-1px; padding:0; border:0; left:0; top:0;`). Chụp lại các ảnh
  `form-deceased-*`; ảnh 360 px phải rộng đúng 720 điểm ảnh.

#### TK-02 — Phép kiểm "không tràn ngang" của kiểm thử khói không phát hiện được tràn ngang — **chặn**

- Màn hình: mọi trang mới (08 §6 "ở A++ và 360 px không tràn ngang (kiểm thử khói đo `scrollWidth`)", 08 §9 dòng "Chung").
- Sai ở đâu: `people-flows.mjs` đo `document.documentElement.scrollWidth - window.innerWidth` trong ngữ cảnh
  `isMobile: true`. Ở chế độ điện thoại, khi trang tràn, `window.innerWidth` **nở theo** (395) nên hiệu số = 0 và kiểm
  thử báo đạt — dù ảnh do chính kiểm thử chụp rộng 790 điểm ảnh. Kết luận "không tràn ngang 360 px A++ ở mọi trang mới"
  trong PR vì vậy chưa có bằng chứng (phép kiểm không đủ năng lực, không phải mệnh đề sai).
- Sửa: so với chiều rộng cố định của khung nhìn đã đặt (360/1280) hoặc với `document.documentElement.clientWidth` và
  `visualViewport.width`; thêm phép kiểm `getBoundingClientRect().right` của `.tabbar` ≤ chiều rộng khung nhìn; thêm phép
  kiểm chiều rộng ảnh chụp toàn trang = 2 × chiều rộng khung nhìn. Chạy lại cho mọi màn hình trong danh sách `screens`,
  cả biểu mẫu thêm mới (có và không có ảnh), ở A và A++.

### 2.2 Mức nên sửa

#### TK-03 — Lỗi trường không tự mất khi đã sửa; không kiểm khi rời ô — nên sửa

- Màn hình: biểu mẫu người đã mất/đang sống/sự kiện. Ảnh: `v02-form-deceased-360.png` — khung "Còn 2 chỗ cần sửa: Họ tên
  không được để trống…" và chữ đỏ dưới ô **trong khi ô đã có "Nguyễn Văn A"**.
- Sai ở đâu (chạy thử): sau khi bấm Lưu khi trống rồi gõ tên, cả chữ đỏ dưới ô lẫn khung tóm tắt vẫn còn cho tới lần bấm
  Lưu sau. Rời ô "Họ và tên" khi trống không báo gì, trái 08 §3.4 ("Lỗi hiện ngay khi rời ô và khi bấm Lưu"); đây là sai
  khác chưa ghi ở §12. Với người không quen công nghệ, màn hình nói "không được để trống" khi ô đã có chữ làm họ nghĩ ứng
  dụng hỏng.
- Sửa: kiểm lại trường ở sự kiện `input` (xóa lỗi khi hết lỗi) và `blur` (báo lỗi); khi một lỗi đã hết, bỏ dòng tương
  ứng khỏi khung tóm tắt (hoặc ẩn khung khi rỗng). Thêm kiểm thử: lưu trống → gõ tên → `#pf-ten-err` ẩn.

#### TK-04 — Ô hộp kiểm bị co nhỏ khi nhãn xuống dòng; kiểu ô không đồng nhất — nên sửa

- Màn hình: "Nhắc thêm ngày tiên thường…" (biểu mẫu người đã mất), "Nhắc cả sinh nhật âm lịch hằng năm" (người đang
  sống). Ảnh: `v02-form-deceased-360*.png`, `v02-form-living-360-light.png`, bản tối cùng tên.
- Sai ở đâu (chạy thử): ô kiểm còn 14×24 px và 13×24 px (ô "Bật nhắc" cạnh đó 24×24 px). Ở giao diện tối ô hiện màu xám
  mặc định, gần như chìm vào nền. Nguyên nhân: `.check input` thiếu `flex: none` nên bị co khi nhãn dài.
- Sửa: `.check input { flex: none; }`; dùng cùng `accent-color` và cùng cỡ 24 px cho mọi ô kiểm ở cả hai giao diện.

#### TK-05 — Vùng chạm dưới 44 px — nên sửa

- Màn hình và số đo (chạy thử, cỡ A, 360 px): "+ Thêm" ở Hồ sơ 86×40; "+ Thêm sự kiện" 144×40; "Xem", "Đã biết" trên thẻ
  nhắc 70×42 và 88×42 (A++: 43); tiêu đề gấp "Lời nhắc trên ứng dụng web" cao 40; liên kết "‹ Quay lại / ‹ Hồ sơ / ‹ Về hồ
  sơ" cao 38; ô chọn tệp sao lưu cao 25 (A++: 32). 08 §6 cam kết ≥ 44×44.
- Sửa: `min-height: 44px` (và `min-width: 44px`) cho `.btnlink.sm`, `.btn2` trong thẻ nhắc, `.notice-web > summary`,
  `.pback`; ô chọn tệp xem TK-06. Thêm vào kiểm thử khói một phép đo vùng chạm các phần tử bấm được của mô-đun.

#### TK-06 — Ô chọn tệp sao lưu dùng nút gốc của trình duyệt — nên sửa

- Màn hình: Sao lưu, khung "Khôi phục từ tệp sao lưu". Ảnh: `v02-backup-360*.png`, `v02-backup-1280-*.png` — hiện
  "Choose File / No file chosen" (chữ tiếng Anh theo ngôn ngữ trình duyệt), nút xám nhỏ, cao 25 px.
- Sai ở đâu: 08 §3.8 C quy định nút "Chọn tệp sao lưu…"; sai khác chưa ghi ở §12. Chữ tiếng Anh và nút nhỏ là trở ngại
  lớn cho người dùng chính, đúng ở bước khôi phục — bước người dùng thường làm khi đang lo mất dữ liệu.
- Sửa: làm như ô chọn ảnh — `input` ẩn (đúng `sr-only`, xem TK-01) + `label` dạng nút "Chọn tệp sao lưu…" cao ≥ 44 px;
  sau khi chọn, hiện "Đã chọn: LichVanHoaViet-saoluu-2026-07-25.lvhv".

#### TK-07 — Ô ngày "T3 / 28 / Th 7": "Th 7" dễ đọc thành "Thứ Bảy" — nên sửa

- Màn hình: mọi mục sự kiện (Sắp tới, Theo kỳ, Hồ sơ một người). Ảnh: `v02-upcoming-360.png`, `v02-period-*.png`.
- Sai ở đâu: trên lịch Việt, "Th 7"/"T7" là cách viết tắt quen thuộc của **Thứ Bảy**; ở đây "Th 7" lại là **tháng 7**, đặt
  ngay dưới "T3" (Thứ Ba). Người lớn tuổi đọc lướt dễ hiểu "28, Thứ Bảy". Dòng ngày đầy đủ bên cạnh giúp đỡ phần nào nhưng
  ô ngày là thứ mắt nhìn trước.
- Sửa: ghi "tháng 7" (hoặc "thg 7") ở dòng dưới; giữ "T3" ở dòng trên. Sửa 08 §2 cho khớp.

#### TK-08 — Hồ sơ một người ở 360 px: "Lần tới" bị đẩy xuống dưới bảng thông tin; mục "Lần tới" bấm không làm gì — nên sửa

- Màn hình: `#/nguoi-than/nguoi/<id>`. Ảnh: `v02-person-deceased-360.png`, `v02-person-360-*.png`, `v02-person-living-360.png`.
- Sai ở đâu: 08 §3.6 (khung dây 360 px) đặt "Lần tới: Giỗ đầu 28/07/2026" **ngay dưới** nút Tưởng niệm/Sửa; bản cài đặt
  đặt nó sau bảng "Thông tin" dài 8–9 dòng, phải cuộn gần hai màn hình mới thấy — thông tin người dùng cần nhất bị giấu.
  Sai khác này không có trong §12. Ngoài ra mục trong "Lần tới" là `button` đang bật nhưng bấm không làm gì (chạy thử:
  `disabled=false`, hàm mở rỗng) — trông như bấm được.
- Sửa: ở khổ hẹp, đặt khung "Lần tới" trước "Thông tin" (đổi thứ tự DOM hoặc `order` trong lưới); mục trong "Lần tới" vẽ
  bằng phần tử không bấm được (hoặc, với giỗ, dẫn tới trang tưởng niệm).

#### TK-09 — Biểu mẫu sự kiện: "Hủy" bỏ dữ liệu đang nhập mà không hỏi — nên sửa

- Màn hình: `#/nguoi-than/su-kien/<id>/moi`. Chạy thử: gõ tiêu đề rồi bấm "Hủy" → về hồ sơ ngay, không có hộp xác nhận.
  Biểu mẫu người thì có hộp "Bỏ các thay đổi chưa lưu?". Hai biểu mẫu cùng mô-đun xử lý khác nhau.
- Sửa: dùng cùng hộp xác nhận khi đã sửa; ghi vào 08 §3.7.

#### TK-10 — Câu lặp "âm lịch" trong kết quả đổi ngày của biểu mẫu sự kiện — nên sửa

- Màn hình: biểu mẫu sự kiện, nhập ngày dương. Ảnh: `v02-form-event-360.png`. Chạy thử: "Âm lịch: 3/7 âm lịch năm Bính
  Ngọ (2026)".
- Sửa: dùng cùng hàm định dạng như biểu mẫu người (`lunarText`) → "Âm lịch: 3/7 năm Bính Ngọ (2026)".

#### TK-11 — Hộp "Xuất tệp lịch": câu riêng tư chưa đúng nội dung tệp; mục xem trước thiếu tên người — nên sửa

- Màn hình: hộp xuất .ics. Ảnh: `v02-ics-dialog-360.png` — "Tệp này chứa tên và ngày giỗ của 4 người" trong khi tệp còn
  chứa sinh nhật và sự kiện của người đang sống (xem trước có "Sinh nhật Lê Văn C (37 tuổi)", "Tốt nghiệp", "Kỷ niệm 6
  năm ngày cưới"). Câu cảnh báo riêng tư (S10) phải nói đúng cái gì bị lộ. Hai mục "Tốt nghiệp", "Kỷ niệm 6 năm ngày
  cưới" không có tên người nên người dùng không biết của ai.
- Sửa: "Tệp này chứa tên, ngày giỗ, sinh nhật và sự kiện của n người. Bất kỳ ai có tệp đều đọc được."; câu xem trước dùng
  cùng nhãn có tên như mục sự kiện ("Tốt nghiệp · Lê Văn C"). Sửa câu mẫu ở 08 §3.8 A.

#### TK-12 — Phần gấp/mở không có dấu hiệu bấm được — nên sửa

- Màn hình: "Tưởng niệm (giỗ hết, dòng đầu bia, lời tưởng niệm)" ở biểu mẫu thêm người đã mất (gấp sẵn); "Lời nhắc trên
  ứng dụng web" ở trang Sắp tới trống. Ảnh: `v02-empty-360.png`; chạy thử biểu mẫu thêm mới.
- Sai ở đâu: `summary` dùng `display:flex` nên mất tam giác mặc định; không có mũi tên nào thay thế — trông như một khung
  tiêu đề rỗng. Người dùng sẽ không biết bên trong có "Dòng đầu bia", "Lời tưởng niệm". Ngoài ra 08 §3.1 nói khung "Lời
  nhắc trên ứng dụng web" "mở sẵn lần đầu", nhưng ở trạng thái trống mã truyền `open = false` (sai khác chưa ghi).
- Sửa: thêm mũi tên ▸/▾ (SVG, `aria-hidden`) và chữ phụ "Bấm để mở"; mở sẵn khung giới hạn web ở trạng thái trống.

#### TK-13 — Độ tương phản dưới ngưỡng — nên sửa

- Chạy thử (WCAG, chữ 17 px thường):
  - Dòng quan hệ dưới tên ở đầu trang hồ sơ (`.muted`, "ông nội") trên nền trang sáng: **3,99:1** (< 4,5). Ảnh
    `v02-person-360-light.png`, `v02-person-1280-light.png`.
  - Chữ gợi ý trong ô (placeholder, "ví dụ: ông, bà, bác, cô") ở giao diện tối: màu mặc định `rgb(117,117,117)` trên
    `rgb(33,26,27)` ≈ **3,7:1**. Ảnh `v02-form-living-360-dark.png`.
  - Các chữ khác đo đều đạt: nhãn trường 7,4:1; "Hưởng thọ 86 tuổi" vàng 5,55:1 (sáng) / 8,9:1 (tối); dòng phụ
    `.fine` 4,95:1; chữ đỏ báo lỗi 6,7:1; bia tưởng niệm ≥ 11:1.
- Sửa: `.phead .muted` dùng `--ink-2`; đặt `::placeholder { color: var(--ink-3); }` cho cả hai giao diện và đo lại.
  08 §6 nên ghi bảng đo tương phản thực tế thay vì chỉ nêu ngưỡng.

#### TK-14 — Hộp "Tạo tệp sao lưu": lỗi nằm xa ô, cảnh báo trông như lỗi — nên sửa

- Màn hình: hộp tạo sao lưu. Ảnh: `v02-backup-dialog-360.png`.
- Sai ở đâu: "Hai lần nhập mật khẩu chưa giống nhau" hiện ở cuối hộp, dưới câu gợi ý, cách ô "Nhập lại mật khẩu" ba khối;
  ô không được đánh dấu lỗi. Câu cảnh báo "Quên mật khẩu là mất dữ liệu…" dùng khung đỏ chữ đỏ giống hệt lỗi, nên người
  dùng khó phân biệt "đã sai" với "lưu ý".
- Sửa: đặt lỗi ngay dưới ô nhập lại, `aria-invalid` + `aria-describedby`; cảnh báo dùng kiểu thẻ thông báo vàng nhạt như
  08 §2 "Thẻ thông báo".

#### TK-15 — Thẻ nhắc đúng ngày giỗ ở 360 px: ba nút chen một hàng; thẻ đẩy tờ lịch xuống — nên sửa

- Màn hình: Tờ lịch và Sắp tới vào ngày giỗ (chạy thử ngày 28/07/2026). Nút "Mở trang tưởng niệm" bị ép thành hình bầu
  dục 4 dòng chữ bên cạnh "Xem", "Đã biết". Ảnh `v02-reminder-card-360.png`: hai thẻ cùng ngày chiếm ~360 px chiều cao,
  tờ lịch — nội dung chính của ứng dụng — bị đẩy xuống nửa dưới màn hình; với tối đa 3 thẻ + "… và n lời nhắc khác",
  tờ lịch ra khỏi màn hình đầu.
- Sửa: ở khổ hẹp, nút chính "Mở trang tưởng niệm" chiếm một hàng riêng, "Xem"/"Đã biết" ở hàng dưới; trên Tờ lịch, từ thẻ
  thứ hai trở đi gộp thành một dòng "Còn 2 lời nhắc hôm nay — Xem" dẫn tới Sắp tới.

#### TK-16 — Sai khác §12 mục 5: khôi phục "Thay toàn bộ" làm "Lần sao lưu gần nhất" thành "Chưa sao lưu lần nào" — nên sửa (cần quyết định trước khi phát hành)

- Màn hình: Sao lưu sau khi khôi phục. Người vừa khôi phục từ tệp sao lưu lại được báo "Chưa sao lưu lần nào", và có thể
  bị nhắc sao lưu ngay — thông tin sai về độ an toàn dữ liệu, đúng lúc người dùng đang lo.
- Sửa: vai lưu trữ quyết định và ghi vào 01/03; đề xuất của bên duyệt thiết kế: sau khôi phục, "Lần sao lưu gần nhất" =
  ngày xuất của tệp vừa khôi phục (tệp đó chính là một bản sao lưu đang có), và không nhắc sao lưu cho tới khi có thay
  đổi mới. Không chặn gộp, nhưng phải có quyết định trước phát hành.

#### TK-17 — Biểu mẫu sự kiện: hộp "Lặp lại hằng năm" nằm lơ lửng ngoài khung — nên sửa

- Màn hình: biểu mẫu sự kiện. Ảnh: `v02-form-event-360*.png`, `v02-form-event-1280-*.png` — hộp kiểm nằm giữa hai khung,
  trên nền trang, lệch lề so với các khung.
- Sai ở đâu: hộp này điều khiển ô "Không rõ năm" trong khung "Ngày" nên phải ở cùng khung; tách ra làm người dùng không
  thấy liên hệ.
- Sửa: đưa vào `fieldset` "Ngày", ngay dưới ba ô ngày.

#### TK-18 — Cảnh báo lưu trữ không đóng được và không có hành động — nên sửa

- Màn hình: Sắp tới, Hồ sơ. Ảnh: `v02-upcoming-*.png`, `v02-list-*.png` — khung vàng 5 dòng ở 360 px, hiện mỗi lần mở.
- Sai ở đâu: 08 §2 "Thẻ thông báo … có nút đóng khi đóng được"; câu bảo "hãy sao lưu" nhưng không có nút "Sao lưu ngay"
  như nhắc sao lưu. Thông báo không bao giờ mất sẽ bị người dùng bỏ qua, kể cả khi nó quan trọng.
- Sửa: thêm "Sao lưu ngay" và "Đã hiểu" (ẩn tới lần sao lưu kế tiếp hoặc 30 ngày); rút gọn câu ở 360 px.

#### TK-19 — Vào ngày giỗ, mục giỗ trong "Sắp tới" không có lối vào trang tưởng niệm — nên sửa

- Chạy thử 28/07/2026: mục "Hôm nay · Giỗ đầu…" trong danh sách 30 ngày không có nút "Mở trang tưởng niệm" (08 §3.9 ghi
  lối vào này). Lối vào duy nhất trên Sắp tới là thẻ nhắc — mà thẻ nhắc chỉ có khi hồ sơ **bật nhắc**. Người tắt nhắc
  phải vào Hồ sơ → tên người → Tưởng niệm.
- Sửa: thêm nút "Mở trang tưởng niệm" dưới mục giỗ của hôm nay (hoặc ghi sai khác vào §12 nếu cố ý bỏ).

### 2.3 Góp ý

| ID | Màn hình / ảnh | Điều thấy | Đề xuất |
|---|---|---|---|
| TK-20 | Biểu mẫu (`v02-form-*`), Sao lưu (`v02-backup-*`) | Nút "Lưu", "Xuất tệp lịch", "Tạo tệp sao lưu", "Mở tệp" là hình chữ nhật bo góc; nút chính ở nơi khác và trong hộp thoại là hình viên thuốc. Trang Sao lưu có ba nút đỏ đặc cùng lúc | Một hình dạng cho nút chính trong toàn ứng dụng; mỗi khung chỉ một nút chính; "Mở tệp" (khôi phục) dùng nút phụ |
| TK-21 | Mọi trang con | Ba chữ quay lại khác nhau: "‹ Quay lại", "‹ Hồ sơ", "‹ Về hồ sơ" (08 §1.2 chỉ ghi "‹ Quay lại") | Thống nhất theo đích: "‹ Về danh sách hồ sơ", "‹ Về hồ sơ"; ghi vào 08 |
| TK-22 | Theo kỳ (`v02-period-*`) | Hai nhóm nút liền nhau dùng hai kiểu "đang chọn" khác nhau (đỏ đặc / trắng nổi); kiểu trắng nổi trùng với thanh phân đoạn phía trên nên khó phân tầng. Ở giao diện tối, nền ô đang chọn gần như bằng nền nhóm — trạng thái chỉ còn nhờ màu chữ và độ đậm | Một kiểu "đang chọn" cho cả hai nhóm (hoặc thêm dấu ✓); ở tối, tăng tương phản nền ô đang chọn ≥ 3:1 với nền nhóm |
| TK-23 | Theo kỳ | Nút "Hôm nay" bị tắt khi đang xem kỳ hiện tại — trông như hỏng | Ẩn nút hoặc đổi thành chữ "Đang xem kỳ hiện tại" |
| TK-24 | Tưởng niệm (`v02-memorial-*`) | Bia ghi "HIỂN KHẢO / Ông Nguyễn Văn A" cho người có quan hệ "ông nội": "Hiển khảo" là cha đã khuất, ông là "Hiển tổ khảo"; danh xưng "Ông" lặp nghĩa với dòng đầu bia. Gợi ý chỉ có "Hiển khảo, Hiển tỷ, Cố", không giải nghĩa. Biên bản phong tục `content/memorial/PHONG-TUC.md` (R9) chưa có | Danh sách gợi ý có nghĩa: Hiển khảo (cha), Hiển tỷ (mẹ), Hiển tổ khảo (ông), Hiển tổ tỷ (bà), Hiển tằng tổ khảo/tỷ (cụ); gợi ý theo "Quan hệ"; khi có dòng đầu bia thì bỏ danh xưng trên bia. Ghi các lựa chọn bia và cách bày bàn thờ (vị trí bình hoa, mâm quả, hai nến) vào PHONG-TUC.md trước phát hành (R9, R-CUL-2) |
| TK-25 | Tưởng niệm 360 px | Nút "Dừng khói" nằm dưới cảnh, ngoài màn hình đầu (chạy thử: y ≈ 966 trong khung 780) | Chấp nhận được vì khói tự dừng; có thể đặt nút nhỏ ngay dưới dòng ngày giỗ |
| TK-26 | Biểu mẫu 1280 px (`v02-form-*-1280-*`), Theo kỳ 1280 px | Biểu mẫu rộng 680 px nhưng dạt trái trong khung 960 px, bên phải trống ~300 px; danh sách Theo kỳ rộng hết khung (08 §7 nói tối đa 720 px) | Căn giữa biểu mẫu hoặc dùng cột phải cho chú thích/"Kết quả đổi lịch"; giới hạn danh sách 720 px |
| TK-27 | Hồ sơ (`v02-list-*`) | Người đã mất không rõ năm ("Cụ Lê Văn D") không có "lần tới" dù giỗ vẫn lặp hằng năm | Hiện "Giỗ 10/3 âm lịch · lần tới dd/mm/yyyy (không rõ năm mất)" |
| TK-28 | Biểu mẫu, ô "Giờ nhắc" | Hiện "07:00 AM" trên trình duyệt đặt tiếng Anh; định dạng theo ngôn ngữ hệ thống, không theo `lang="vi"` | Ghi chú giới hạn này; cân nhắc hai ô số giờ:phút 24 giờ |
| TK-29 | Biểu mẫu người đang sống (`v02-form-living-360.png`) | Bật sinh nhật âm thêm nguyên một khối nhắc thứ hai — biểu mẫu dài gần 5 màn hình | Mặc định "Nhắc giống sinh nhật dương lịch" (một hộp kiểm), chỉ mở khối riêng khi bỏ chọn |
| TK-30 | Hồ sơ | Sắp xếp theo chuỗi "họ tên" nên thực chất xếp theo họ; nhiều nhà cùng họ | Cân nhắc xếp theo tên (chữ cuối) rồi họ, hoặc theo thế hệ/quan hệ |
| TK-31 | Tờ lịch (`v02-sheet-reminder-1280-*`) | Lịch tháng không đánh dấu ngày giỗ; thẻ nhắc ở 1280 px căn giữa 520 px, không thẳng lề với tờ lịch | Đợt sau: chấm nhỏ ở ngày có giỗ/sinh nhật; thẻ nhắc thẳng lề trái với tờ lịch |
| TK-32 | Biểu mẫu | "Ngày sinh và nơi sinh" rồi ngay dưới lại "Ngày sinh" — hai tiêu đề chồng; nhóm "Tưởng niệm" các trường sát nhau (nhãn dính vào nhóm nút phía trên) | Bỏ tiêu đề trong (giữ `legend` cho trợ năng bằng `sr-only`); thêm khoảng cách đều như các nhóm khác |
| TK-33 | Biểu mẫu, ngày sinh "Chỉ biết năm" | Mã chấp nhận năm 1000–3000, 08 §3.4 ghi 1900–3000 | Giữ 1000 (hữu ích cho tổ tiên) nhưng sửa 08 cho khớp |
| TK-34 | Hồ sơ một người đang sống | "Lần tới" chỉ hiện sự kiện gần nhất (Tốt nghiệp 15/08), bỏ "Kỷ niệm 6 năm ngày cưới" 18/11/2026 dù sớm hơn sinh nhật | Hiện mọi lần tới trong 12 tháng (tối đa 5 mục) |
| TK-35 | Đặc tả 08 | Thiếu: khung dây biểu mẫu ở 1280 px; trạng thái tên dài trên bia (xuống dòng/thu chữ); kiểm thử cho nhắc sao lưu, cho trường hợp > 3 thẻ nhắc, cho lỗi ảnh, cho lỗi kho; bảng đo tương phản thực | Bổ sung vào §3.9, §6, §9 |

## 3. Chấm theo docs/08 §5 (nội dung tối thiểu của đặc tả)

| Mục §5 | Đánh giá | Ghi chú |
|---|---|---|
| Kiến trúc thông tin, điều hướng | Đạt | §1.1–1.2 rõ; URL không chứa tên/ngày. Lệch nhỏ: chữ quay lại (TK-21) |
| Từng màn hình: mục đích, lối vào | Đạt | Thiếu lối vào tưởng niệm từ mục giỗ hôm nay trong bản cài (TK-19) |
| Từng trường: kiểm tra, lời báo lỗi | Đạt (đặc tả) / chưa khớp (cài đặt) | Bảng lỗi đầy đủ; cài đặt chưa kiểm khi rời ô và không xóa lỗi cũ (TK-03) |
| Trạng thái trống / đang tải / lỗi / xác nhận | Đạt | Có đủ cho Sắp tới, Theo kỳ, Hồ sơ, hồ sơ không tìm thấy, hộp xóa/khôi phục |
| Luồng người dùng đánh số | Đạt | L1–L13 |
| Toàn bộ câu chữ | Gần đạt | Câu .ics chưa đúng nội dung (TK-11); câu lặp "âm lịch" (TK-10) |
| Trợ năng | Gần đạt | Có bàn phím, tiêu điểm, nhãn, giảm chuyển động; vùng chạm (TK-05), tương phản (TK-13) chưa đạt cam kết |
| Bố cục 360/1280, sáng/tối | **Chưa đạt** | Tràn ngang biểu mẫu người đã mất (TK-01) |
| Khung dây | Đạt một phần | Có cho Theo kỳ, hồ sơ một người, tưởng niệm, các hộp; thiếu biểu mẫu 1280 px (TK-35) |
| Bảng câu chuyện → màn hình → kiểm thử | Đạt | §10 đủ 16 câu chuyện; nhưng kiểm thử tràn ngang không đủ năng lực (TK-02) |

## 4. Đối chiếu từng câu chuyện (07 §2)

| Câu chuyện | Màn hình | Trạng thái | Câu chữ | Kiểm thử | Kết luận của bên duyệt |
|---|---|---|---|---|---|
| US-M-01 | §3.4, §3.6 | có | có | L1 | Đạt về thiết kế; bị TK-01, TK-03 |
| US-M-02 | §3.4 (âm, nhuận, ngày 30) | có | có | L2 | Đạt; chạy thử "Tháng 7 âm lịch năm 2026 chỉ có 29 ngày" đúng |
| US-M-03 | nhãn §5.1, §3.6, §3.9 | có | có | L1, L5, L11 | Đạt; góp ý dòng đầu bia (TK-24) |
| US-M-04 | §3.4 nhóm nhắc, §5.4–5.5 | có | có | L1, L7 (chỉ ngày D−3) | Đạt trong giới hạn web đã nêu; TK-04, TK-15 |
| US-M-05 | §3.9 | có | có | L11, R8 một phần | Đạt về thiết kế; R8 đo CPU chưa có (§5 dưới) |
| US-M-06 | §3.4 nơi an táng, ảnh | có | có | L1 (APP1) | Đạt; bị TK-01 |
| US-S-01 | §3.5 | có | có | L3 | Đạt; góp ý TK-29 |
| US-S-02 | §3.7 | có | có | L4 | Đạt; TK-09, TK-10, TK-17 |
| US-S-03 | §3.5, §3.6 | có | có | L3 | Đạt |
| US-C-01 | §3.2 | có | có | L5 | Đạt |
| US-C-02 | §3.2 bộ lọc | có | có | L5 | Đạt; góp ý TK-22 |
| US-C-03 | §3.1 | có | có | L6 | Đạt; TK-18, TK-19 |
| US-C-04 | §3.8 A, §5.4 | có | chưa đúng | L8 | Nên sửa TK-11 |
| US-C-05 | §3.8 B–D | có | có | L9, L10 | Đạt; TK-06, TK-14, TK-16 |
| US-C-06 | §3.4–3.7, §3.10 | có | có | L12, L13 | Đạt; TK-09 |
| US-C-07 | §1.2, §1.3, §5.6 | có | có | localStorage, mạng ngoài | Đạt |

## 5. Các sai khác đã liệt kê ở 08 §12

| # | Sai khác | Đánh giá của bên duyệt thiết kế |
|---|---|---|
| 1 | Thẻ nhắc chỉ ở Tờ lịch và Sắp tới | **Chấp nhận.** Lý do hợp lý (không chen vào biểu mẫu, tưởng niệm). Lưu ý TK-15 về chiều cao thẻ trên Tờ lịch |
| 2 | Dòng "Dương lịch" chỉ hiện khi ngày mất nhập theo âm | **Chấp nhận.** Tránh lặp thông tin |
| 3 | Ghi chú trong nhóm đầu; hộp "Bật nhắc" | **Chấp nhận.** Gọn hơn; cần sửa bảng §3.4 cho khớp (đã làm) |
| 4 | Thông báo ngắn mang sang trang kế tiếp | **Chấp nhận.** Đúng cách để người dùng thấy kết quả |
| 5 | Khôi phục "Thay toàn bộ" thay cả cài đặt, "Lần sao lưu gần nhất" trở về giá trị trong tệp | **Chấp nhận tạm, có điều kiện**: cần quyết định ghi vào 01/03 trước phát hành (TK-16) |
| 6 | R8 trong kiểm thử khói chỉ đo phần nhanh | **Chấp nhận cho duyệt thiết kế.** Hành vi khói đúng thiết kế (chạy thử). Nhưng R8 (CPU 120 s/60 s) vẫn là **CHƯA ĐỦ BẰNG CHỨNG**; không được ghi ĐẠT cho tới khi đo theo 04 R8 |

Sai khác **chưa** được liệt kê mà bên duyệt thấy: thứ tự khung trên hồ sơ một người ở 360 px (TK-08); ô chọn tệp sao lưu
gốc (TK-06); không kiểm khi rời ô (TK-03); khung giới hạn web gấp sẵn ở trạng thái trống (TK-12); thiếu nút tưởng niệm ở
mục giỗ hôm nay (TK-19); danh sách Theo kỳ 1280 px không giới hạn 720 px (TK-26); chữ quay lại (TK-21); miền năm "chỉ biết
năm" (TK-33). Theo docs/08 và lời đầu của 08, mỗi điểm phải được sửa theo đặc tả hoặc ghi vào §12.

## 6. Kết luận

Số phát hiện: **2 chặn**, **17 nên sửa**, **16 góp ý**.

**Kết luận: DUYỆT CÓ ĐIỀU KIỆN.**

Điều kiện để gộp (các mục chặn):
1. **TK-01** — Sửa tràn ngang ở biểu mẫu thêm/sửa người đã mất: `.people input[type="file"]:not(.sr-only)` và `.sr-only`
   bền hơn; chụp lại ảnh `form-deceased-*` (360 px phải rộng đúng 720 điểm ảnh, 1280 px đúng 2560), thanh mục đáy thấy
   được trên trang này.
2. **TK-02** — Sửa phép kiểm tràn ngang của kiểm thử khói để không bị khung nhìn điện thoại "nở" che mất (so với chiều
   rộng cố định hoặc `clientWidth`/`visualViewport.width`, kiểm cả `.tabbar` và chiều rộng ảnh chụp), chạy lại cho mọi
   màn hình mới ở A và A++, và chứng minh phép kiểm mới **bắt được** lỗi TK-01 trước khi sửa.

Các mục nên sửa: sửa trong đợt này hoặc mở việc có hạn trước phát hành v0.2; ưu tiên TK-03, TK-05, TK-06, TK-07, TK-08,
TK-11, TK-16 vì ảnh hưởng trực tiếp người dùng 35–70 tuổi trên điện thoại. Sau khi sửa, bên duyệt thiết kế cần xem lại
bộ ảnh mới (ít nhất `form-deceased-*`, `person-*`, `backup-*`, `form-event-*`).
