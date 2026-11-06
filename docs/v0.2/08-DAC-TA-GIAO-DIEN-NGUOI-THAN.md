# Đặc tả giao diện — mô-đun "Người thân" (PWA, v0.2 đợt 3)

Trạng thái: **BẢN NHÁP ĐỂ DUYỆT THIẾT KẾ** — 2026-11-07, vai B4 (PWA). Viết **trước** khi viết mã giao diện của
mô-đun. Mã cài đặt theo đúng tài liệu này; chỗ nào khi cài buộc phải đổi thì sửa tài liệu này trong cùng PR và ghi
vào mục "Sai khác" của mô tả PR.

Căn cứ: `01-MO-HINH-DU-LIEU.md` (01), `02-THUAT-TOAN-GIO-NHAC.md` (02), `03-SAO-LUU.md` (03),
`04-TIEU-CHI-NGHIEM-THU-V02.md` (04, gồm Bổ sung B2 và R13), `06-BIEN-BAN-G1.md` (O-1 … O-9),
`07-PHAN-TICH-YEU-CAU-DOT-3.md` (07: câu chuyện US-M-01 … 06, US-S-01 … 03, US-C-01 … 07; rủi ro R-UX-1, R-UX-2,
R-DATA-2, R-CUL-2; ba câu hỏi §9 theo **mặc định**: một thẻ "Người thân", tuần bắt đầu Thứ Hai, kỳ theo dương lịch và
mỗi mục ghi kèm ngày âm). Quy trình: `docs/08-QUY-TRINH-MO-RONG-TINH-NANG.md`.

Phạm vi: chỉ PWA. Không tài khoản, dữ liệu chỉ trên máy (IndexedDB `lvhv`). Native (Android/Apple/Windows) dùng lại
luồng và câu chữ ở đây nhưng có thông báo hệ thống thật; phần "giới hạn của ứng dụng web" (§5.4) chỉ áp cho PWA.

---

## 1. Kiến trúc thông tin

### 1.1 Vị trí trong ứng dụng

Thêm **một** mục chính thứ năm **"Người thân"** vào thanh mục (giữa "Đổi ngày" và "Giới thiệu"). Lý do chọn một mục
thay vì hai ("Người đã mất", "Người đang sống"): danh sách theo kỳ (R13) và "Sắp tới" (02 §7.2) trộn cả hai loại
người; tách hai mục sẽ buộc người dùng mở hai nơi để biết tuần này nhà có việc gì. Bộ lọc "Người đã mất / Người đang
sống" nằm trong mô-đun.

Thanh mục sau khi thêm (thứ tự): Tờ lịch · Chọn ngày tốt · Đổi ngày · **Người thân** · Giới thiệu.
Điện thoại: 5 ô đều nhau ở đáy (lưới 5 cột). Máy tính (≥ 720 px): hàng nút tròn như cũ.

### 1.2 Bốn trang con (thanh phân đoạn trong mô-đun)

```
Người thân
├── Sắp tới        #/nguoi-than            (trang đầu của mô-đun)
├── Theo kỳ        #/nguoi-than/theo-ky     (R13: tuần / tháng / quý / năm)
├── Hồ sơ          #/nguoi-than/ho-so       (danh sách người đã mất, người đang sống)
└── Sao lưu        #/nguoi-than/sao-luu     (xuất .ics, xuất/nhập sao lưu)

Trang không có trên thanh phân đoạn (mở từ các trang trên):
  #/nguoi-than/them/da-mat        Thêm người đã mất
  #/nguoi-than/them/dang-song     Thêm người đang sống
  #/nguoi-than/nguoi/<id>         Hồ sơ một người
  #/nguoi-than/sua/<id>           Sửa hồ sơ
  #/nguoi-than/su-kien/<id>/<eid|moi>   Thêm / sửa sự kiện của người <id>
  #/nguoi-than/tuong-niem/<id>    Màn hình tưởng niệm
```

- `<id>` là UUID ngẫu nhiên của hồ sơ (01 §1). **URL không bao giờ chứa tên hay ngày** (S4'). Kỳ đang xem ở "Theo
  kỳ" (loại kỳ, ngày neo, bộ lọc) giữ trong bộ nhớ của trang, không đưa vào URL.
- Thanh phân đoạn là nhóm 4 nút (`role="tablist"` không dùng; dùng `nav` + liên kết `aria-current="page"`), cao
  ≥ 48 px, nằm dưới tiêu đề trang. Ở các trang không có trên thanh (thêm, sửa, hồ sơ, tưởng niệm) có nút
  "‹ Quay lại" về trang trước (lịch sử trình duyệt; nếu mở thẳng từ URL thì về "Hồ sơ").

### 1.3 Tải lười

Toàn bộ mã mô-đun (giao diện, lõi giỗ/nhắc `tsport/src/memorial*`, lớp lưu trữ, mật mã, hình tưởng niệm) chỉ tải khi
mở `#/nguoi-than…` hoặc khi khởi động mà **đã có** cơ sở dữ liệu `lvhv` (để kiểm thẻ nhắc, §5.5). Lần tải đầu của
người chưa từng dùng mô-đun không mở IndexedDB và không tải tệp nào của mô-đun (P3, P6). Khi đang tải: chữ
"Đang mở sổ người thân…" ở giữa vùng nội dung (không có vòng quay chạy mãi).

---

## 2. Thành phần dùng chung

| Thành phần | Mô tả |
|---|---|
| **Mục sự kiện** (`.pitem`) | Một nút rộng cả hàng. Trái: ô ngày dương (số ngày lớn, thứ viết tắt "T5", tháng "thg 11" — không viết "Th 11" vì dễ đọc thành Thứ Bảy, TK-07). Giữa: dòng 1 = nhãn + tên ("Giỗ đầu · Ông Nguyễn Văn A"); dòng 2 = "26/11/2026 · 7/10 âm lịch"; dòng 3 (nếu có) = phụ ("Hưởng thọ 86 tuổi", "Kỷ niệm 5 năm"). Phải: biểu tượng loại + "Còn 3 ngày" / "Hôm nay" / "Ngày mai" / "Đã qua 2 ngày". Chạm → mở hồ sơ người đó (sự kiện → hồ sơ của người đầu tiên trong `person_ids`). Trong khung "Lần tới" của trang hồ sơ, mục **không** bấm được (vẽ bằng phần tử thường, TK-08) |
| **Biểu tượng loại** | Giỗ: ba nén hương; Tiên thường: mâm nhỏ; Sinh nhật (dương và âm): bánh có nến; Sự kiện: ngôi sao. Vẽ nét SVG trong mã, `aria-hidden`; loại luôn có chữ đi kèm (không chỉ dựa vào hình/màu) |
| **Ô nhập ngày** | Ba ô số Ngày / Tháng / Năm như trang "Đổi ngày" (`input.num`, `inputmode="numeric"`), cộng chọn lịch "Dương lịch / Âm lịch" (nhóm nút radio), với âm lịch thêm hộp "Tháng nhuận"; tùy chỗ thêm "Không rõ năm" hoặc "Chỉ biết năm". Dưới ô có dòng **kết quả đổi lịch trực tiếp** (`aria-live="polite"`) |
| **Hộp xác nhận** | `<dialog>` của trình duyệt (`showModal`), tiêu đề, nội dung, hai nút: nút chính bên phải (hành động) và "Hủy"/"Giữ lại". `Esc` = hủy. Tiêu điểm vào nút **an toàn** (Hủy) khi mở hộp xóa; vào nút chính khi mở hộp xuất |
| **Thông báo lỗi trường** | Chữ đỏ ngay dưới trường, `id` riêng, trường có `aria-invalid="true"` và `aria-describedby`. Khi bấm Lưu mà còn lỗi: một khung tóm tắt `role="alert"` ở đầu biểu mẫu "Còn n chỗ cần sửa" + liên kết tới từng trường, tiêu điểm vào khung đó |
| **Thẻ thông báo** | Khung `.notice` nền vàng nhạt (thông tin) hoặc đỏ nhạt (cảnh báo), có nút đóng khi đóng được |

Định dạng: ngày dương `dd/mm/yyyy`; ngày âm `d/m âm lịch` (+ " nhuận"); năm âm kèm Can Chi khi đủ chỗ ("năm Ất Tỵ
(2025)"). Thứ: "Thứ Hai … Chủ nhật" (đầy đủ), "T2 … CN" (viết tắt trong ô ngày). Tên hiển thị = `danh_xung + " " +
ho_ten` (02 §7.1).

---

## 3. Từng trang

### 3.1 Sắp tới (trang đầu mô-đun) — `#/nguoi-than`

Mục đích: biết ngay 30 ngày tới nhà có giỗ, sinh nhật, sự kiện gì (02 §7.2, R11); thấy thẻ nhắc của hôm nay (02
§7.5); được nhắc sao lưu.

Thứ tự từ trên xuống:
1. Tiêu đề "Người thân" + dòng phụ "Ngày giỗ, sinh nhật và sự kiện của gia đình. Dữ liệu chỉ nằm trên máy này."
2. Thanh phân đoạn (Sắp tới đang chọn).
3. **Thẻ nhắc hôm nay** (§5.5), nếu có.
4. **Nhắc sao lưu** (§5.6), nếu đến hạn; **cảnh báo lưu trữ** nếu `persist` bị từ chối (01 §4.1).
5. Khung "30 ngày tới" — danh sách mục sự kiện từ hôm nay đến hôm nay + 30 (cả hai đầu), theo thứ tự 02 §7.2.
6. Khung "Lời nhắc trên ứng dụng web" (§5.4) — thu gọn được (mũi tên ▸/▾), **mở sẵn** ở mọi trạng thái kể cả trống (TK-12).
   Mục giỗ của **hôm nay** trong danh sách có thêm nút "Mở trang tưởng niệm" ngay dưới mục (TK-19), kể cả khi hồ sơ tắt nhắc.
7. Hai nút: "+ Thêm người đã mất", "+ Thêm người đang sống".

Trạng thái:
- **Trống (chưa có hồ sơ nào)**: thay mục 3–5 bằng khung chào: "Chưa có ai trong sổ. Thêm ông bà, cha mẹ đã khuất để
  được nhắc ngày giỗ theo âm lịch; thêm người đang sống để nhớ sinh nhật và ngày kỷ niệm." + hai nút thêm.
- **Có hồ sơ nhưng 30 ngày tới không có gì**: "Trong 30 ngày tới không có ngày giỗ, sinh nhật hay sự kiện nào." +
  liên kết "Xem theo tháng, quý, năm".
- **Đang tải**: "Đang mở sổ người thân…".
- **Lỗi kho** (không mở được IndexedDB, ví dụ trình duyệt chặn bộ nhớ, hoặc dữ liệu của bản mới hơn): khung đỏ
  "Không mở được dữ liệu trên máy: <lý do>. Lịch vẫn dùng được bình thường." Lý do "Dữ liệu được tạo bởi phiên bản mới
  hơn — hãy cập nhật ứng dụng" lấy nguyên văn từ lớp lưu trữ.

### 3.2 Theo kỳ — `#/nguoi-than/theo-ky` (R13)

Mục đích: xem mọi lần xuất hiện (giỗ, tiên thường nếu bật, sinh nhật dương, sinh nhật âm, sự kiện) trong một tuần,
tháng, quý hoặc năm dương lịch.

Bố cục:
```
┌──────────────────────────────────────────┐
│ [ Tuần | Tháng | Quý | Năm ]             │  nhóm nút, aria-pressed
│ [ Tất cả | Người đã mất | Người đang sống ]│
│  ‹    Quý IV/2026 · 01/10 – 31/12    ›   │  tiêu đề kỳ, aria-live
│            [ Hôm nay ]                   │  khi kỳ đang xem chứa hôm nay: thay bằng chữ
│                                          │  "Đang xem kỳ có hôm nay" (không để nút bị tắt, TK-23)
│ 12 mục                                   │
│ ┌─────┬───────────────────────────┬────┐ │
│ │ T2  │ Giỗ · Bà Trần Thị B        │ ✦  │ │
│ │ 12  │ 12/10/2026 · 1/9 âm lịch   │còn │ │
│ │Th 10│ Hưởng thọ 79 tuổi          │ 5 n│ │
│ └─────┴───────────────────────────┴────┘ │
│ …                                        │
└──────────────────────────────────────────┘
```

- Kỳ: **Tuần** = Thứ Hai → Chủ nhật chứa ngày neo; **Tháng** dương; **Quý** dương (1–3, 4–6, 7–9, 10–12); **Năm**
  dương. Mặc định khi mở: Tháng, ngày neo = hôm nay, bộ lọc Tất cả.
- Tiêu đề kỳ (định dạng cố định):
  - Tuần: `Tuần 02/11 – 08/11/2026` (khác năm: `Tuần 28/12/2026 – 03/01/2027`)
  - Tháng: `Tháng 11/2026 · 01/11 – 30/11`
  - Quý: `Quý IV/2026 · 01/10 – 31/12` (số La Mã I–IV)
  - Năm: `Năm 2026 · 01/01 – 31/12`
- "‹" / "›": kỳ trước / kỳ sau cùng loại (nhãn trợ năng "Kỳ trước", "Kỳ sau"). Đổi loại kỳ giữ ngày neo (ví dụ đang
  xem tháng 2/2027 bấm "Quý" → Quý I/2027). "Hôm nay": ngày neo = hôm nay.
- Bộ lọc (Bổ sung B4 mục 3 của 04): **Người đã mất** = ngày giỗ và tiên thường (nếu bật); **Người đang sống** = mọi lần
  xuất hiện còn lại (sinh nhật, sự kiện). Sự kiện gắn với người đã mất không có trong danh sách (B4 mục 1–2). Dưới nhóm
  nút có một dòng giải thích: "Chỉ có ngày giỗ (và tiên thường nếu bật nhắc). Không nhắc sinh nhật và sự kiện của người
  đã mất." / "Sinh nhật và sự kiện của người đang sống." Không phụ thuộc công tắc nhắc.
- Mỗi mục: như §2; "Còn n ngày" tính từ **hôm nay** (mục đã qua trong kỳ: "Đã qua n ngày", chữ nhạt hơn). Mục của hôm
  nay có viền đỏ.
- Dòng đếm: "n mục" (`aria-live="polite"` cùng tiêu đề kỳ).
- Kỳ **Quý** và **Năm** (R-UX-2): danh sách chia nhóm theo tháng dương, mỗi nhóm có tiêu đề "Tháng 10/2026" (`h3`);
  tháng không có mục thì không hiện tiêu đề. Tuần và Tháng không chia nhóm.
- **Trống**: "Không có ngày giỗ, sinh nhật hay sự kiện nào trong <tuần này / tháng này / quý này / năm này>." Với bộ
  lọc khác "Tất cả", thêm "(đang lọc: Người đã mất)". Chưa có hồ sơ nào → khung chào như §3.1.
- Phím: `←`/`→` không dùng ở trang này (tránh xung đột với ô nhập); điều hướng bằng Tab tới các nút.

### 3.3 Hồ sơ (danh sách) — `#/nguoi-than/ho-so`

- Hai khung: **"Người đã mất (n)"** và **"Người đang sống (n)"**, mỗi khung có nút "+ Thêm" riêng ở đầu khung.
- Mỗi dòng (nút rộng): vòng tròn chữ cái đầu của tên (không vẽ ảnh ở danh sách — tránh giải mã nhiều ảnh), tên hiển
  thị, quan hệ (nếu có), và một dòng tóm tắt:
  - Đã mất: `Giỗ 15/6 âm lịch · lần tới 18/07/2027 (còn 253 ngày)`; không rõ năm mất vẫn có lần tới (TK-27):
    `Giỗ 10/3 âm lịch · lần tới 16/04/2027 (còn 265 ngày) (không rõ năm mất)`.
  - Đang sống: `Sinh 20/01/1990 · 36 tuổi` (hoặc `Sinh năm 1990`).
- Sắp xếp trong mỗi khung: theo **tên** (chữ cuối của họ tên) rồi đến cả họ tên (so sánh tiếng Việt của trình duyệt,
  `localeCompare(…, "vi")`) — nhiều nhà cùng họ nên xếp theo họ không giúp tìm (TK-30).
- Trống từng khung: "Chưa có ai." + nút thêm.

### 3.4 Thêm / sửa người đã mất — `#/nguoi-than/them/da-mat`, `#/nguoi-than/sua/<id>`

Tiêu đề: "Thêm người đã mất" / "Sửa hồ sơ". Biểu mẫu chia nhóm (`fieldset` + `legend`):

**Nhóm "Người đã mất"**

| Trường | Điều khiển | Bắt buộc | Kiểm tra → câu báo lỗi |
|---|---|---|---|
| Cách gọi (đứng trước tên) | ô chữ + gợi ý (`datalist`): cụ, kỵ, ông, bà, bác, chú, thím, cô, dì, cậu, mợ, anh, chị, em, bố, mẹ, cha, má, ba (trường `danh_xung`) | không | > 40 ký tự → "Cách gọi tối đa 40 ký tự" |
| Họ và tên | ô chữ, `autocomplete="off"` | **có** | rỗng sau khi cắt khoảng trắng → "Họ tên không được để trống"; > 120 → "Họ tên tối đa 120 ký tự" |
| Tên hiệu / pháp danh | ô chữ | không | > 120 → "Tên hiệu tối đa 120 ký tự" |
| Quan hệ với bạn | ô chữ + gợi ý (`datalist`) bằng từ thường ngày: Bố, Mẹ, Ông nội, Bà nội, Ông ngoại, Bà ngoại, Cụ ông, Cụ bà, Kỵ ông, Kỵ bà, Ông cố, Bà cố, Ông sơ, Bà sơ, Bác, Chú, Thím, Cô, Dì, Cậu, Mợ, Anh, Chị, Em, Chồng, Vợ, Con (người đang sống: Bố … Con trai, Con gái, Cháu). Dưới ô (người đã mất): "Trang tưởng niệm ghi quan hệ này phía trên tên, ví dụ “Ông nội”, “Mẹ”. Để trống thì chỉ ghi tên." | không | > 120 → "Quan hệ tối đa 120 ký tự" |
| Ghi chú | ô nhiều dòng | không | > 2000 → "Ghi chú tối đa 2000 ký tự" |

Mọi chuỗi chuẩn hóa NFC và cắt khoảng trắng hai đầu trước khi kiểm và lưu (01 §1); ký tự điều khiển bị bỏ.

**Nhóm "Ngày mất" (bắt buộc)**

- **Kiểm khi gõ và khi rời ô** (TK-03): lỗi hiện khi rời ô (với ba ô ngày: khi rời cả nhóm) và khi bấm Lưu; khi đang có
  lỗi thì kiểm lại theo từng lần gõ — lỗi tự mất ngay khi đã sửa, khung tóm tắt "Còn n chỗ cần sửa" bỏ dòng tương ứng và
  ẩn khi hết lỗi. Áp cho mọi biểu mẫu của mô-đun.
- Chọn: "Nhập theo dương lịch" (mặc định) / "Nhập theo âm lịch".
- Dương: Ngày / Tháng / Năm (năm 1900–3000). Kết quả trực tiếp: **"Âm lịch: 15/6 nhuận năm Ất Tỵ (2025)"** (đây là
  giá trị sẽ lưu, 02 §1). Thiếu ô → không hiện kết quả.
- Âm: Ngày (1–30) / Tháng (1–12) / Năm, hộp "Tháng nhuận", hộp "Không rõ năm mất" (ẩn ô Năm). Hộp "Tháng nhuận"
  (R-UX-1): khi năm âm đã nhập và năm đó **có** tháng nhuận, hộp hiện với nhãn "Tháng nhuận (năm 2025 nhuận tháng 6)";
  năm không có tháng nhuận → hộp ẩn (coi như không đánh dấu), chú thích nhỏ "Năm 2026 không có tháng nhuận"; chưa nhập
  năm hoặc "Không rõ năm" → hộp hiện, nhãn "Tháng nhuận". Đánh dấu nhuận với tháng khác tháng nhuận của năm đó → lỗi
  "Năm 2025 không có tháng 5 nhuận". Kết quả trực tiếp: có
  năm → **"Dương lịch: Thứ Sáu, 08/08/2025"**; không rõ năm → "Không rõ năm: vẫn nhắc giỗ hằng năm vào 10/3 âm lịch,
  không ghi giỗ đầu, giỗ hết."
- Lỗi (hiện ngay khi rời ô và khi bấm Lưu):

| Tình huống | Câu báo |
|---|---|
| Chưa nhập | "Hãy nhập ngày mất" |
| Thiếu một ô | "Hãy nhập đủ ngày, tháng, năm" (âm không rõ năm: "Hãy nhập đủ ngày và tháng âm") |
| Dương không tồn tại (30/02) | "Tháng 2 năm 2026 chỉ có 28 ngày" |
| Năm ngoài miền | "Năm phải từ 1900 đến 3000" (âm có năm ngoài miền 1899–3000: vẫn lưu, ghi "Năm này ngoài khoảng đổi lịch — không có ngày dương tương ứng") |
| Tháng nhuận không có | "Năm 2026 không có tháng 6 nhuận" |
| Ngày 30 tháng thiếu | "Tháng 7 âm lịch năm 2026 chỉ có 29 ngày" |
| Ngày mất ở tương lai | "Ngày mất không được sau hôm nay" |
| Năm âm mất < năm âm sinh | "Ngày mất trước ngày sinh" |
| Tuổi tính ra > 150 | "Tuổi tính ra lớn hơn 150 — hãy kiểm tra năm sinh và năm mất" |

**Nhóm "Ngày sinh và nơi sinh" (không bắt buộc)**

- Chọn lịch Dương / Âm; chọn "Biết đủ ngày" / "Chỉ biết năm". Âm đủ ngày có "Tháng nhuận".
- Kết quả trực tiếp: "Năm âm 1940 — Canh Thìn" (dương chỉ năm: "khoảng năm âm 1940 — Canh Thìn (nếu sinh trước Tết
  thì là năm trước)").
- Nơi sinh: ô chữ ≤ 300.
- Lỗi: như ngày mất (không có lỗi "tương lai" cho người đã mất; ngày sinh sau hôm nay → "Ngày sinh không được sau hôm nay").

**Tuổi (hiển thị, sửa được)**: dòng "Hưởng thọ 86 tuổi" / "Hưởng dương 45 tuổi" (+ " (khoảng)" khi `approx`), tính
theo 02 §4.2 với ngưỡng của hồ sơ. Nút "Sửa tuổi" mở ô số 1–150 → lưu `manual = true`; khi đang sửa có nút "Tính
lại" (bỏ `manual`). Thiếu năm sinh → "Chưa đủ năm sinh để tính tuổi".

**Nhóm "Nơi an táng và ảnh"**

- Nơi an táng: ô chữ ≤ 300 → "Nơi an táng tối đa 300 ký tự".
- Ảnh chân dung: nút "Chọn ảnh" (`input type=file accept="image/*"`), ô xem trước (canvas 160×200, khung bầu dục);
  khi có ảnh: "Đổi ảnh", "Bỏ ảnh". Xử lý (01 §3.8, S9): giải mã bằng `createImageBitmap` (trình duyệt xoay theo EXIF),
  thu cạnh dài ≤ 1024 px (không phóng to), **vẽ lại vào canvas và mã hóa lại JPEG** — byte lưu không còn EXIF/GPS/XMP/
  IPTC. Ảnh hiển thị bằng cách vẽ lên `<canvas>`; **không dùng `blob:` URL cho ảnh** (S2b, R-CSP-1).
- Lỗi ảnh: tệp > 25 MB → "Ảnh lớn hơn 25 MB — hãy chọn ảnh nhỏ hơn"; không giải mã được → "Không đọc được ảnh này
  (định dạng không hỗ trợ)"; sau khi nén vẫn > 1 MB → "Ảnh quá chi tiết, chưa nén được dưới 1 MB".
- Dòng chú thích: "Ảnh chỉ lưu trong ứng dụng trên máy này, đã bỏ thông tin vị trí và máy chụp."

**Nhóm "Nhắc ngày giỗ"**

| Trường | Điều khiển | Mặc định | Ghi chú |
|---|---|---|---|
| Nhắc giỗ | công tắc (hộp kiểm) "Nhắc ngày giỗ" | bật | tắt → các trường dưới mờ đi nhưng giữ giá trị |
| Nhắc trước | nút bật/tắt: "Đúng ngày", "1 ngày", "3 ngày", "5 ngày", "10 ngày", "15 ngày" (`aria-pressed`) + ô "Tự đặt" 1–60 ngày | Đúng ngày, 3 ngày (`[0, 3]`) | Không chọn mốc nào → "Chưa chọn mốc nhắc nào — sẽ không có lời nhắc" (cảnh báo, vẫn lưu được); ô tự đặt ngoài 1–60 → "Số ngày tự đặt phải từ 1 đến 60". Tối đa 8 mốc → "Tối đa 8 mốc nhắc" |
| Giờ nhắc (24 giờ) | hai ô số giờ : phút (TK-28; không dùng `type=time` vì ô này hiện SA/CH theo ngôn ngữ trình duyệt); số một chữ số được thêm số 0 ("5" → "05") | 07:00 | giờ ngoài 0–23 hoặc phút ngoài 0–59 → "Giờ nhắc phải từ 0 đến 23 giờ, phút từ 0 đến 59" |
| Tiên thường | hộp kiểm "Nhắc thêm ngày tiên thường (lễ cúng hôm trước ngày giỗ)" | **tắt** (O-1) | — |

**Nhóm "Tưởng niệm" (thu gọn, `details`)**

| Trường | Điều khiển | Mặc định |
|---|---|---|
| Giỗ hết (giỗ lần cuối của thời gian để tang) vào | radio "Năm thứ 2 sau năm mất" / "Năm thứ 3 sau năm mất" | năm thứ 2 (O-2) |
| Lời tưởng niệm | ô nhiều dòng ≤ 2000 | trống |
| Mất trước tuổi này thì bia ghi “hưởng dương” (mất sớm) thay cho “hưởng thọ” | ô số 1–150 | 60 (O-3) |

Kiểu mâm (`tray_style`) không có lựa chọn ở bản này: luôn **truyền thống** (01 §3.7 mặc định); ba nén hương bật.
Dữ liệu nhập từ sao lưu có `cong_giao` vẫn được tôn trọng: màn hình tưởng niệm không vẽ hương (V6, R8 (f)). Xem §11.

**Nút cuối biểu mẫu**: "Lưu" (nút chính) · "Hủy". Hủy khi đã sửa → hộp "Bỏ các thay đổi chưa lưu?" ("Bỏ thay đổi" /
"Tiếp tục sửa").

Sau khi lưu thành công: về trang hồ sơ người đó, thông báo ngắn "Đã lưu hồ sơ." Nếu đây là hồ sơ đầu tiên: gọi
`navigator.storage.persist()` (lớp lưu trữ tự gọi); bị từ chối → trang hồ sơ hiện cảnh báo lưu trữ (§5.6).

Lỗi khi ghi (hiếm: kho bị khóa ở thẻ khác, hết chỗ): "Chưa lưu được: <lý do>. Dữ liệu đang nhập vẫn còn trên màn hình."

### 3.5 Thêm / sửa người đang sống — `#/nguoi-than/them/dang-song`, `#/nguoi-than/sua/<id>`

| Trường | Điều khiển | Bắt buộc | Lỗi |
|---|---|---|---|
| Danh xưng, Họ và tên, Quan hệ, Ghi chú | như §3.4 | Họ tên | như §3.4 |
| Ngày sinh | ô nhập ngày: Dương/Âm × "Biết đủ ngày"/"Chỉ biết năm" (năm 1000–3000, TK-33) | không | như §3.4; sau hôm nay → "Ngày sinh không được sau hôm nay" |
| Nơi sinh | ô chữ ≤ 300 | không | — |
| Sinh nhật âm lịch | hộp kiểm "Nhắc cả sinh nhật âm lịch hằng năm"; khi bật có hộp con "Nhắc sinh nhật âm lịch giống nhắc sinh nhật dương lịch" (TK-29, mặc định bật) | tắt | Chỉ bật được khi biết đủ ngày sinh; nếu không: hộp mờ, chú thích "Cần biết đủ ngày sinh" |
| Sinh 29/02 | radio "Năm không nhuận: mừng 28/02" (mặc định) / "… mừng 01/03" | — | chỉ hiện khi ngày sinh dương là 29/02 |
| Nhắc sinh nhật | như "Nhắc ngày giỗ" (không có tiên thường); khối riêng "Nhắc sinh nhật âm lịch" chỉ hiện khi bật sinh nhật âm **và** bỏ chọn "giống nhắc sinh nhật dương lịch" (§12e) | bật, `[0, 3]`, 07:00 | như §3.4 |

Kết quả trực tiếp dưới ngày sinh: "Âm lịch: 24/12 năm Kỷ Tỵ (1989) · Hôm nay 36 tuổi (tuổi âm 38)" (02 §4.3).

Sự kiện của người đang sống được thêm **sau khi lưu**, ở trang hồ sơ (§3.7): biểu mẫu thêm người không chứa sự kiện, để
biểu mẫu ngắn; trang hồ sơ có nút "+ Thêm sự kiện" ngay sau lần lưu đầu.

### 3.6 Hồ sơ một người — `#/nguoi-than/nguoi/<id>`

```
‹ Quay lại
┌──────────── 360 px ─────────────┐   ┌──────────────── 1280 px ────────────────────────┐
│  (ảnh bầu dục)                  │   │ ┌ảnh┐  Ông Nguyễn Văn A           [Sửa][Xóa]   │
│  Ông Nguyễn Văn A               │   │ └───┘  ông nội · Tên hiệu: Phúc Thiện          │
│  ông nội                        │   │ ┌ Thông tin ─────────┐ ┌ Lần tới ─────────────┐ │
│ [Tưởng niệm] [Sửa]              │   │ │ Mất 08/08/2025      │ │ Giỗ đầu 28/07/2026   │ │
│ Lần tới: Giỗ đầu 28/07/2026     │   │ │ = 15/6 nhuận Ất Tỵ  │ │ còn 263 ngày         │ │
│   (15/6 âm lịch) · còn 263 ngày │   │ │ Hưởng thọ 86 tuổi   │ └──────────────────────┘ │
│ ─ Thông tin ─                   │   │ │ …                   │ ┌ Sự kiện ─────────────┐ │
│ Ngày mất  08/08/2025            │   │ └─────────────────────┘ │ + Thêm sự kiện       │ │
│ Âm lịch   15/6 nhuận Ất Tỵ      │   │                         └──────────────────────┘ │
│ …                               │   └──────────────────────────────────────────────────┘
│ ─ Sự kiện ─  [+ Thêm sự kiện]   │
│ ─ Nhắc ─ Đúng ngày, 3 ngày trước│
│   lúc 07:00 · Tiên thường: tắt  │
│ [Xóa hồ sơ]                     │
└─────────────────────────────────┘
```

- Thứ tự khung (TK-08): đầu trang (ảnh, tên, quan hệ, nút) → **"Lần tới"** → "Thông tin" ở cột chính; "Sự kiện", "Nhắc",
  "Xóa hồ sơ" ở cột phụ (360 px: xếp dưới). "Lần tới" liệt kê **mọi** lần xuất hiện trong 12 tháng tới (giỗ, tiên thường,
  sinh nhật dương/âm, sự kiện), tối đa 5 mục (TK-34); mục chỉ để xem, không bấm được.
- Người đã mất: ảnh (canvas), tên, quan hệ, tên hiệu; nút **"Tưởng niệm"** (mở §3.9) và "Sửa"; khung "Lần tới" (giỗ
  kế tiếp, nhãn giỗ đầu/giỗ hết, có tiên thường nếu bật); bảng thông tin: Ngày mất (như đã nhập), Âm lịch (đã lưu,
  nguồn "đổi từ ngày dương" hoặc "nhập theo âm lịch"), Hưởng thọ/dương, Ngày sinh, Năm âm sinh + Can Chi, Nơi sinh, Nơi
  an táng, Ghi chú; khung Sự kiện; khung Nhắc; nút "Xóa hồ sơ" ở cuối (màu đỏ viền, không phải nút chính).
- Người đang sống: không có ảnh/tưởng niệm; "Lần tới" = sinh nhật kế tiếp ("Sinh nhật thứ 37 · 20/01/2027"), sinh nhật
  âm nếu bật; bảng thông tin có "Tuổi: 36 (tuổi âm 38)".
- Khung Sự kiện: mỗi sự kiện một dòng "Tốt nghiệp · 15/06/2026 · một lần" / "Ngày cưới · 10/3 âm lịch · hằng năm"
  với nút "Sửa" và "Xóa" (xóa có hộp xác nhận). Trống: "Chưa có sự kiện nào."
- Không tìm thấy id: "Không tìm thấy hồ sơ này (có thể đã bị xóa)." + liên kết về "Hồ sơ".

### 3.7 Thêm / sửa sự kiện — `#/nguoi-than/su-kien/<id>/<eid|moi>`

| Trường | Điều khiển | Bắt buộc | Lỗi |
|---|---|---|---|
| Loại | chọn: Tốt nghiệp, Ngày cưới, Đính hôn, Du học, các loại đã tự tạo, "Loại khác…" | có | — |
| Tên loại mới | ô chữ (hiện khi chọn "Loại khác…") ≤ 60 | khi "Loại khác…" | rỗng → "Hãy đặt tên cho loại sự kiện"; trùng tên (không phân biệt hoa thường) → dùng lại loại đã có, không báo lỗi |
| Tiêu đề | ô chữ ≤ 120, gợi ý "để trống sẽ dùng tên loại" | không | > 120 → "Tiêu đề tối đa 120 ký tự" |
| Ngày | ô nhập ngày Dương/Âm, Tháng nhuận; với Âm + lặp hằng năm có thêm "Không rõ năm" | có | như §3.4 (không có lỗi "tương lai": sự kiện có thể ở tương lai) |
| Lặp lại hằng năm | hộp kiểm **trong khung "Ngày"**, ngay dưới ba ô ngày (nó điều khiển ô "Không rõ năm", TK-17); mặc định theo loại (Ngày cưới: bật) | — | Không rõ năm mà tắt lặp → "Sự kiện không rõ năm phải lặp hằng năm" |
| Nhắc | như §3.4 (không tiên thường). **Sự kiện của người đã mất (B4):** không có khối nhắc; thay bằng câu "Không nhắc sự kiện của người đã mất — với người đã mất, ứng dụng chỉ nhắc ngày giỗ. Sự kiện vẫn được lưu và hiện trên hồ sơ." Quy tắc nhắc đang lưu (nếu có, từ sao lưu) giữ nguyên | bật, `[0, 3]`, 07:00 | như §3.4 |
| Ghi chú | ô nhiều dòng ≤ 2000 | không | — |

`person_ids` = [người đang xem]. Lưu xong → về hồ sơ người đó. "Hủy" khi đã sửa → cùng hộp "Bỏ các thay đổi chưa lưu?" như
biểu mẫu người (TK-09). Kết quả đổi lịch: "Âm lịch: 3/7 năm Bính Ngọ (2026)" — không lặp chữ "âm lịch" (TK-10).

### 3.8 Sao lưu và lịch điện thoại — `#/nguoi-than/sao-luu`

Ba khung:

**A. "Để điện thoại tự nhắc: xuất lịch (.ics)"** — chú thích §5.4; nút "Xuất tệp lịch (.ics)…". Bấm → hộp xác nhận
(S10, 02 §8.2):
```
Xuất tệp lịch
Tệp này chứa tên, ngày giỗ, sinh nhật và sự kiện của 3 người. Bất kỳ ai có tệp đều đọc được.
Gồm 41 ngày trong 10 năm tới (từ 07/11/2026 đến 06/11/2036), mỗi ngày có lời nhắc như đã đặt.
Xem trước:
  • 28/07/2026 — Giỗ đầu Ông Nguyễn Văn A
  • 10/09/2026 — Giỗ Bà Trần Thị B
  … và 39 ngày khác
Mở tệp bằng ứng dụng Lịch của điện thoại hoặc máy tính để nhập.
                               [Hủy]  [Tải tệp .ics]
```
"n người" = số hồ sơ và sự kiện khác nhau có trong tệp (đếm người: hồ sơ; sự kiện đếm vào "mục"). Nội dung tệp: mọi
mục có nhắc bật (02 §8.1, A.6). Không có mục nào có nhắc bật → nút bị tắt kèm chú thích "Chưa có hồ sơ nào bật nhắc."
Tên tệp `LichVanHoaViet-ngay-gio-<YYYY-MM-DD>.ics`. Tải bằng thẻ `<a download>`; không gửi mạng.

**B. "Sao lưu dữ liệu"** — lần sao lưu gần nhất: "Lần sao lưu gần nhất: 07/11/2026" / "Chưa sao lưu lần nào". Nút
"Tạo tệp sao lưu…" → hộp:
```
Tạo tệp sao lưu
Tệp sẽ chứa 5 người, 2 sự kiện, 1 ảnh — đã mã hóa bằng mật khẩu.
Mật khẩu (ít nhất 8 ký tự)      [••••••••••]
Nhập lại mật khẩu               [••••••••••]
[thẻ vàng] Lưu ý: quên mật khẩu là mất dữ liệu trong tệp — không ai lấy lại được, kể cả Clevix Lab.
Gợi ý: dùng một câu dễ nhớ, ví dụ "nhà ở Hà Đông có cây bàng".
                               [Hủy]  [Tạo và tải tệp]
```
Lỗi (hiện **ngay dưới ô** tương ứng, ô có `aria-invalid` + `aria-describedby`, TK-14): < 8 ký tự → "Mật khẩu cần ít nhất 8
ký tự" (dưới ô Mật khẩu); không khớp → "Hai lần nhập mật khẩu chưa giống nhau" (dưới ô Nhập lại). Câu "quên mật khẩu…"
là thẻ thông báo vàng (lưu ý), không dùng kiểu đỏ của lỗi. Khi chạy: nút
tắt, thanh tiến độ `<progress>` + "Đang mã hóa… (có thể mất vài giây)". Xong: tải tệp
`LichVanHoaViet-saoluu-<YYYY-MM-DD>.lvhv`, hộp đóng, thông báo "Đã tạo tệp sao lưu. Hãy cất tệp ở nơi an toàn (ổ đám
mây, USB) — tệp không tự gửi đi đâu." Ô mật khẩu: `type=password`, `autocomplete="new-password"`,
`autocapitalize="off"`, `spellcheck="false"`.

**C. "Khôi phục từ tệp sao lưu"** — nút "Chọn tệp sao lưu…" (nút của ứng dụng, cao ≥ 44 px; ô chọn tệp gốc của trình
duyệt ẩn hẳn, `accept=".lvhv"`), dòng "Đã chọn: <tên tệp>" / "Chưa chọn tệp.", ô mật khẩu, nút phụ "Mở tệp" (mỗi khung
chỉ một nút chính, TK-06, TK-20). Lỗi theo
03 §2.2/§2.4, nguyên văn từ lớp lưu trữ: "Đây không phải tệp sao lưu của Lịch và Văn hóa Việt", "Tệp được tạo bởi
phiên bản mới hơn — hãy cập nhật ứng dụng", "Sai mật khẩu hoặc tệp bị hỏng". Mở được → hộp xem trước:
```
Khôi phục dữ liệu
Tệp xuất ngày 07/11/2026 từ ứng dụng web: 5 người, 2 sự kiện, 1 ảnh.
Cách nhập:
 (•) Gộp vào dữ liệu đang có (khuyên dùng) — giữ bản mới hơn của từng hồ sơ
 ( ) Thay toàn bộ — xóa dữ liệu đang có trên máy rồi dùng dữ liệu trong tệp
                               [Hủy]  [Khôi phục]
```
Chọn "Thay toàn bộ" → hộp xác nhận thứ hai: "Dữ liệu hiện có trên máy (n người) sẽ bị thay bằng dữ liệu trong tệp xuất
ngày 07/11/2026. Không hoàn tác được." ("Thay toàn bộ" / "Hủy"). Xong: "Đã khôi phục: 5 người, 2 sự kiện." Lỗi khi ghi:
"Chưa khôi phục được — dữ liệu trên máy giữ nguyên."

**D. Cài đặt nhắc sao lưu** — chọn "Nhắc sao lưu khi có thay đổi chưa sao lưu sau: 30 ngày / 60 ngày / 90 ngày / Không
nhắc" (`backup_nag_days`, mặc định 30).

### 3.9 Màn hình tưởng niệm — `#/nguoi-than/tuong-niem/<id>`

Mục đích: một trang trang nghiêm để thắp hương tưởng nhớ vào ngày giỗ (05 §1). Không âm thanh, không quảng cáo, không
chia sẻ.

Vào từ: nút "Tưởng niệm" trên hồ sơ người đã mất; nút "Mở trang tưởng niệm" trên thẻ nhắc **đúng ngày giỗ** (§5.5) và
trên mục giỗ của hôm nay ở "Sắp tới".

```
┌─────────────────────────────────────┐
│ ‹ Về hồ sơ                          │
│      Giỗ đầu · hôm nay 15/6 âm lịch │   dòng ngày (chỉ khi hôm nay là giỗ; khác:
│                                     │   "Ngày giỗ năm nay: 15/6 âm lịch, 28/07/2026")
│            ╭──────────╮             │
│            │  (ảnh)   │  khung bầu dục viền vàng đồng
│            ╰──────────╯             │
│        ╭────────────────────╮       │
│        │      Ông nội       │       │   bia: đỉnh vòm, nền đá sẫm, chữ vàng đồng,
│        │   Nguyễn Văn A     │       │   chữ serif; tên hiệu ở dòng nhỏ
│        │  Pháp danh Phúc Thiện      │
│        │  Canh Thìn 1940 —  │       │
│        │     Ất Tỵ 2025     │       │
│        │ Hưởng thọ 86 tuổi  │       │
│        ╰────────────────────╯       │
│   ~ ~ ~   (khói nhẹ)                │
│    │ │ │   ba nén hương             │
│   ╭─────╮       ╭───────────╮       │
│   │bát  │       │ mâm quả   │       │   bàn thờ: bát hương + mâm ngũ quả + hai nến
│ ══╧═════╧═══════╧═══════════╧══     │
│                                     │
│  "Lời tưởng niệm của gia đình…"     │
│                                     │
│  [ Dừng khói ]                      │
└─────────────────────────────────────┘
```

- Dòng trên tên: **chữ ở ô "Quan hệ"**, viết hoa chữ đầu, bỏ phần trong ngoặc ("ông nội (bên nội)" → "Ông nội"). Ô
  Quan hệ trống → không có dòng này. Không có ô "dòng đầu bia" riêng và không dùng từ Hán-Việt cổ (quyết định của chủ
  dự án, §12e; `content/memorial/PHONG-TUC.md` §1).
- Tên trên bia: khi có dòng quan hệ thì chỉ ghi họ tên, **không** lặp cách gọi ("Ông nội / Nguyễn Văn A", không phải
  "Ông nội / Ông Nguyễn Văn A"); không có dòng quan hệ thì ghi tên hiển thị (cách gọi + họ tên).
- Tên rất dài (> 28 ký tự): chữ nhỏ lại (lớp `.long`), xuống dòng trong bia, không tràn (TK-35).
- Năm trên bia: `Can Chi năm âm (năm)` cho năm sinh (nếu có) và năm mất (nếu có); thiếu năm sinh → chỉ "Mất năm Ất Tỵ
  (2025)"; không rõ năm mất → "Giỗ ngày 10/3 âm lịch".
- Tuổi: "Hưởng thọ 86 tuổi" / "Hưởng dương 45 tuổi" (+ "(khoảng)"); không có tuổi → bỏ dòng.
- Hình: SVG/CSS viết trong mã (không tệp ảnh ngoài), tông nâu gỗ và vàng đồng, cùng một kiểu ở sáng và tối (nền tối
  ấm cả hai giao diện để bia nổi). Ảnh vẽ bằng canvas; không có ảnh → khung bầu dục trống có chữ cái đầu tên.
- Khói (R8): ba vệt khói CSS (`@keyframes` dịch lên + mờ dần), **không dùng `requestAnimationFrame`**. Khói chạy khi
  trang hiển thị, **tự dừng sau 2 phút** (sau đó khói đứng yên dạng tĩnh), dừng ngay khi tab ẩn
  (`visibilitychange`) hoặc rời trang; trở lại tab trong 2 phút đầu → chạy tiếp tới hết 2 phút tính từ lúc mở. Nút
  "Dừng khói"/"Thắp lại" (thắp lại = chạy thêm 2 phút), đặt ngay dưới dòng ngày giỗ để thấy được ở màn hình đầu (TK-25). `prefers-reduced-motion: reduce` → khói tĩnh ngay từ đầu,
  nút ẩn.
- Công giáo (`tray_style = cong_giao`, chỉ đến từ sao lưu ở bản này): không vẽ bát hương và khói; vẽ hai nến và hoa.
- Tiêu đề tab trình duyệt: "Tưởng niệm — Lịch và Văn hóa Việt" (**không** có tên người; O-9 tinh thần riêng tư cho
  những gì hiện ngoài trang).

### 3.10 Xóa

- **Xóa hồ sơ**: hộp "Xóa hồ sơ của Ông Nguyễn Văn A? Ngày giỗ, lời nhắc và ảnh của người này sẽ bị xóa khỏi máy. Sự
  kiện gắn với người này vẫn còn nếu có người khác cùng tham gia. Không hoàn tác được." — nút "Xóa hồ sơ" (đỏ) / "Giữ
  lại" (tiêu điểm mặc định). Xong: về "Hồ sơ", thông báo "Đã xóa hồ sơ."
  Sự kiện chỉ gắn với người bị xóa: lớp lưu trữ gỡ id khỏi sự kiện (01 §3.4), sự kiện còn lại không có người. Giao diện
  **xóa luôn** những sự kiện mà sau khi gỡ không còn ai (để không còn mục mồ côi trong danh sách) — trong cùng thao tác
  sau khi xóa hồ sơ.
- **Xóa sự kiện**: hộp "Xóa sự kiện "Tốt nghiệp"? Không hoàn tác được." — "Xóa sự kiện" / "Giữ lại".

---

## 4. Luồng người dùng

**L1 — Thêm người đã mất theo ngày dương**
1. Mở "Người thân" (thanh mục). Trang "Sắp tới" trống hiện khung chào.
2. Bấm "+ Thêm người đã mất".
3. Gõ Danh xưng "Ông", Họ và tên "Nguyễn Văn A", Quan hệ "ông nội".
4. Ở "Ngày mất", giữ "Nhập theo dương lịch", gõ 8 / 8 / 2025.
5. Dòng kết quả hiện "Âm lịch: 15/6 nhuận năm Ất Tỵ (2025)".
6. (Tùy) Ngày sinh: Dương, đủ ngày 1/5/1940 → "Năm âm 1940 — Canh Thìn"; dòng tuổi "Hưởng thọ 86 tuổi".
7. Giữ nhắc mặc định (Đúng ngày, 3 ngày, 07:00, tiên thường tắt). Bấm "Lưu".
8. Trang hồ sơ hiện "Âm lịch 15/6 nhuận Ất Tỵ (đổi từ ngày dương)", "Lần tới: Giỗ đầu 28/07/2026 (15/6 âm lịch)".

**L2 — Thêm người đã mất theo ngày âm, tháng nhuận**
1–3. Như L1 (Bà Trần Thị B).
4. Chọn "Nhập theo âm lịch", gõ 15 / 6 / 2025, đánh dấu "Tháng nhuận".
5. Dòng kết quả "Dương lịch: Thứ Sáu, 08/08/2025".
6. Thử đổi Tháng thành 5 (vẫn đánh dấu nhuận) → lỗi "Năm 2025 không có tháng 5 nhuận"; đổi Năm thành 2026 → hộp "Tháng
   nhuận" ẩn, chú thích "Năm 2026 không có tháng nhuận". Sửa lại 15 / 6 / 2025, nhuận.
7. Lưu → hồ sơ ghi "Âm lịch 15/6 nhuận Ất Tỵ (nhập theo âm lịch)", giỗ hằng năm vào 15/6 tháng **thường** (O-4).

**L3 — Thêm người đang sống**
1. "Hồ sơ" → khung "Người đang sống" → "+ Thêm".
2. Họ và tên "Lê Văn C", Quan hệ "con trai"; Ngày sinh Dương, đủ ngày 20/01/1990.
3. Dòng kết quả "Âm lịch: 24/12 năm Kỷ Tỵ (1989) · Hôm nay 36 tuổi (tuổi âm 38)".
4. (Tùy) bật "Nhắc cả sinh nhật âm lịch hằng năm". Lưu.
5. Hồ sơ: "Lần tới: Sinh nhật thứ 37 · 20/01/2027".

**L4 — Thêm sự kiện**
1. Trên hồ sơ người đang sống, bấm "+ Thêm sự kiện".
2. Loại "Tốt nghiệp"; Ngày Dương 15/06/2027; "Lặp lại hằng năm" tắt (mặc định của loại).
3. Lưu → khung Sự kiện có "Tốt nghiệp · 15/06/2027 · một lần". Mục xuất hiện ở "Theo kỳ" (Tháng 6/2027).

**L5 — Danh sách theo kỳ**
1. "Theo kỳ". Mặc định Tháng hiện tại, Tất cả.
2. Bấm "Quý" → tiêu đề "Quý IV/2026 · 01/10 – 31/12", danh sách cập nhật.
3. "›" → "Quý I/2027 · 01/01 – 31/03"; "‹" hai lần → "Quý III/2026 …".
4. "Hôm nay" → về quý chứa hôm nay.
5. "Tuần" → "Tuần 02/11 – 08/11/2026"; "Năm" → "Năm 2026 · 01/01 – 31/12".
6. Bộ lọc "Người đã mất" → chỉ còn giỗ/tiên thường; "Người đang sống" → sinh nhật, sự kiện của người đang sống;
   "Tất cả" → hợp hai danh sách (B4).
7. Chạm một mục → hồ sơ người đó.

**L6 — Sắp tới**: mở "Người thân" → danh sách 30 ngày; mục hôm nay ghi "Hôm nay", ngày mai "Ngày mai".

**L7 — Thẻ nhắc trong ứng dụng**
1. Người dùng mở ứng dụng ở "Tờ lịch" (trang mở đầu) hoặc "Người thân › Sắp tới" vào ngày có lời nhắc đến hạn: ngày `D − k` với `k` trong
   `offsets_days` của một lần xuất hiện có nhắc bật, hoặc ngày tiên thường `D − 1` khi bật.
2. Đầu trang hiện thẻ: "Còn 3 ngày: Giỗ đầu Ông Nguyễn Văn A" / dòng phụ "Thứ Ba, 28/07/2026 — 15/6 âm lịch" (02 §7.1).
   Đúng ngày giỗ: "Hôm nay: Giỗ đầu …" + nút "Mở trang tưởng niệm".
3. Nút "Xem" → hồ sơ; nút "Đã biết" → ẩn thẻ cho tới lần mở ứng dụng sau (không lưu gì xuống máy ngoài IndexedDB).

**L8 — Xuất .ics**: "Sao lưu" → "Xuất tệp lịch (.ics)…" → hộp xác nhận + xem trước → "Tải tệp .ics" → trình duyệt tải
tệp; người dùng mở tệp bằng ứng dụng Lịch.

**L9 — Xuất sao lưu**: "Sao lưu" → "Tạo tệp sao lưu…" → nhập mật khẩu hai lần → "Tạo và tải tệp" → tiến độ → tải tệp →
"Lần sao lưu gần nhất: hôm nay"; nhắc sao lưu (nếu có) biến mất.

**L10 — Nhập sao lưu**: "Sao lưu" → "Chọn tệp sao lưu…" → mật khẩu → "Mở tệp" → xem trước → "Gộp" (mặc định) hoặc
"Thay toàn bộ" (+ xác nhận thứ hai) → "Khôi phục" → thông báo kết quả.

**L11 — Màn hình tưởng niệm**: hồ sơ người đã mất → "Tưởng niệm" → trang bia + mâm + hương; khói chạy ≤ 2 phút; "‹ Về
hồ sơ".

**L12 — Sửa**: hồ sơ → "Sửa" → biểu mẫu điền sẵn → đổi ngày mất → dòng kết quả đổi lịch cập nhật → "Lưu" → `rev` tăng 1
(01 §1).

**L13 — Xóa**: hồ sơ → "Xóa hồ sơ" → hộp xác nhận (tiêu điểm ở "Giữ lại") → "Xóa hồ sơ" → về danh sách, người đó
không còn trong "Sắp tới"/"Theo kỳ".

---

## 5. Câu chữ

### 5.1 Nhãn chính

| Chỗ | Chữ |
|---|---|
| Thanh mục | Người thân |
| Thanh phân đoạn | Sắp tới · Theo kỳ · Hồ sơ · Sao lưu |
| Nút thêm | + Thêm người đã mất · + Thêm người đang sống · + Thêm sự kiện |
| Loại kỳ | Tuần · Tháng · Quý · Năm |
| Bộ lọc | Tất cả · Người đã mất · Người đang sống |
| Điều hướng kỳ | ‹ (Kỳ trước) · › (Kỳ sau) · Hôm nay |
| Nhãn lần xuất hiện | Giỗ đầu · Giỗ hết · Giỗ (lần thứ n) · Ngày giỗ (không rõ năm mất) · Tiên thường · Sinh nhật thứ n · Sinh nhật âm lịch (tuổi âm n) · <tên sự kiện> · Kỷ niệm n năm ngày cưới · <tên> (kỷ niệm n năm) |
| Khoảng cách | Hôm nay · Ngày mai · Còn n ngày · Hôm qua · Đã qua n ngày |

### 5.2 Thông báo thành công
"Đã lưu hồ sơ." · "Đã lưu sự kiện." · "Đã xóa hồ sơ." · "Đã xóa sự kiện." · "Đã tạo tệp sao lưu. Hãy cất tệp ở nơi an
toàn (ổ đám mây, USB) — tệp không tự gửi đi đâu." · "Đã khôi phục: n người, m sự kiện." · "Đã tải tệp lịch. Mở tệp
bằng ứng dụng Lịch để nhập." Thông báo hiện ở vùng `role="status"` đầu trang, tự ẩn sau 6 giây (không ẩn khi người dùng
đang đặt tiêu điểm vào nó).

### 5.3 Lỗi
Như các bảng ở §3. Lỗi từ lớp lưu trữ/sao lưu dùng nguyên văn câu của lớp đó (03 §2.2, §2.4; 01 §7).

### 5.4 Giới hạn của ứng dụng web (02 §7.5, R-PWA-2)
Tiêu đề: **"Lời nhắc trên ứng dụng web"**. Nội dung: "Ứng dụng web không gửi được thông báo khi đang đóng. Lời nhắc chỉ
hiện ở đầu trang khi bạn mở ứng dụng vào ngày cần nhắc. Để điện thoại tự nhắc cả khi không mở ứng dụng, hãy xuất tệp
lịch (.ics) và mở bằng ứng dụng Lịch của máy." + nút "Xuất tệp lịch (.ics)…" (mở hộp §3.8 A).

### 5.5 Thẻ nhắc
Tiêu đề và dòng phụ đúng 02 §7.1 (`reminderTextNamed` của lõi). Trong ứng dụng **có** hiện tên (O-9 chỉ áp cho màn hình
khóa; PWA không có thông báo màn hình khóa). Trên **Tờ lịch** chỉ hiện **một** thẻ; các lời nhắc còn lại gộp
thành một dòng "Còn n lời nhắc khác hôm nay — Xem" dẫn tới Sắp tới, để tờ lịch không bị đẩy khỏi màn hình đầu (TK-15). Ở
khổ hẹp, nút chính "Mở trang tưởng niệm" chiếm một hàng riêng. Trên **Sắp tới**: nhiều thẻ cùng ngày xếp theo thời điểm
nhắc rồi thứ tự 02 §7.2; quá 3 thẻ
→ hiện 3 và "… và n lời nhắc khác" (liên kết tới "Sắp tới").

### 5.6 Sao lưu và lưu trữ
- Nhắc sao lưu (khi `backupReminderDue`, 01 §3.9): "Dữ liệu người thân đã thay đổi hơn 30 ngày mà chưa sao lưu. Nếu mất
  máy hoặc trình duyệt xóa dữ liệu, sẽ không lấy lại được." + nút "Sao lưu ngay".
- Lưu trữ không bền (`persist` = false): "Trình duyệt có thể xóa dữ liệu khi máy thiếu chỗ — hãy sao lưu. Trên iPhone,
  thêm ứng dụng vào Màn hình chính để dữ liệu được giữ lâu hơn." (01 §4.1, R-PWA-1).

---

## 6. Trợ năng

- **Bàn phím**: mọi điều khiển là phần tử gốc (`button`, `a`, `input`, `select`, `dialog`). Thứ tự tiêu điểm theo thứ tự
  đọc: tiêu đề → thanh phân đoạn → điều khiển trang → danh sách. Mở trang con → tiêu điểm vào `h1` của trang
  (`tabindex="-1"`). Đóng hộp → tiêu điểm về nút đã mở hộp.
- **Nhãn**: mọi ô có `label` thấy được; nhóm có `fieldset/legend`; nút chỉ có biểu tượng có `aria-label` ("Kỳ trước",
  "Kỳ sau"). Nhóm nút chọn (loại kỳ, bộ lọc, mốc nhắc) dùng `aria-pressed`. Tiêu đề kỳ và số mục: `aria-live="polite"`.
- **Lỗi**: §2; không chỉ dùng màu đỏ — luôn có chữ.
- **Tương phản**: chữ thường ≥ 4,5:1, chữ lớn và đường viền điều khiển ≥ 3:1, ở cả sáng và tối, dùng biến màu có sẵn
  (`--ink`, `--ink-2`, `--red-ink`…). Bia tưởng niệm: chữ vàng đồng `#e8c67a` trên nền đá `#2b2422` (≈ 9:1).
- **Giảm chuyển động**: `prefers-reduced-motion: reduce` → không khói động, không hiệu ứng chuyển trang.
- **Cỡ chữ A / A+ / A++**: mọi kích thước tính bằng `rem`/`em`; ở A++ và 360 px không tràn ngang (kiểm thử khói đo
  `scrollWidth`); hàng ba ô ngày giữ nguyên, các nhóm nút xuống dòng.
- **Vùng chạm** ≥ 44 × 44 px — kiểm thử khói đo mọi nút, liên kết, ô nhập, `summary`, nhãn hộp kiểm trong vùng nội dung của
  mô-đun ở A và A++ (TK-05). Ô hộp kiểm 24 × 24 px không co khi nhãn xuống dòng; vùng chạm là cả nhãn (TK-04).
- **Bảng đo tương phản** (WCAG, tính từ biến màu; TK-13):

  | Chữ / nền | Sáng | Tối |
  |---|---|---|
  | Dòng quan hệ dưới tên hồ sơ (`--ink-2` trên nền trang) | 5,96:1 | 10,58:1 |
  | Chữ gợi ý trong ô (`::placeholder`, `--ink-3` trên `--paper`) | 4,95:1 | 6,31:1 |
  | Dòng phụ `.fine`, "Đang xem kỳ có hôm nay" (`--ink-3` trên `--paper`) | 4,95:1 | 6,31:1 |
  | Nút đang chọn: chữ trên nền | trắng / `--red-deep` 9,21:1 | `#2a0d0c` / `--red-ink` 7,94:1 |
  | Nút đang chọn: nền ô so với nền nhóm (≥ 3:1, TK-22) | 8,27:1 | 6,82:1 |
  | Bia tưởng niệm (chữ vàng đồng trên đá) | ≥ 9:1 | ≥ 9:1 |
- `lang="vi"` có sẵn ở trang; tên người hiển thị nguyên văn (không viết hoa tự động ngoài dòng đầu bia).

## 7. Bố cục đáp ứng

| | 360 px (điện thoại) | 1280 px (máy tính) |
|---|---|---|
| Khung trang | Một cột, lề 16 px, bề rộng tối đa 640 px | Bề rộng tối đa 960 px, căn giữa |
| Sắp tới | Danh sách một cột | Hai cột: trái danh sách 30 ngày; phải (320 px) giới hạn web + nút thêm + nhắc sao lưu |
| Theo kỳ | Hai nhóm nút chồng nhau, tiêu đề kỳ một dòng (có thể xuống 2 dòng ở A++) | Hai nhóm nút cùng hàng; danh sách một cột rộng tối đa 720 px |
| Hồ sơ (danh sách) | Hai khung chồng | Hai khung cạnh nhau |
| Biểu mẫu | Một cột; ba ô ngày một hàng | Một cột rộng tối đa 640 px |
| Hồ sơ một người | Một cột: "Lần tới" rồi "Thông tin", rồi sự kiện, nhắc (§3.6) | Hai cột: "Lần tới" + "Thông tin" trái; sự kiện + nhắc + xóa phải |
| Tưởng niệm | Bia rộng 80 % màn hình | Bia rộng 360 px, căn giữa, nền tối phủ cả vùng nội dung |
| Hộp xác nhận | Rộng `100vw − 24px` | Rộng tối đa 480 px |

---

## 8. Ghi chú kỹ thuật ràng buộc giao diện

### 8.1 Tính toán
- Lần xuất hiện, nhãn, thứ tự, câu nhắc, .ics: **chỉ** dùng hàm của lõi `tsport/src/memorial*` (`occurrencesWithTt`,
  `upcoming`, `reminderTextNamed`, `generateIcs`, `tuoiDuong`, `tuoiAm`…), không tự viết lại thuật toán ở giao diện.
- Lưu trữ: `pwa/src/store` (đợt 2): `buildPerson`, `putPerson`, `putEvent`, `putEventType`, `deletePerson`,
  `deleteEvent`, `exportBackup`, `openBackup`, `applyImport`, `backupReminderDue`, `persistResult`.

### 8.2 Danh sách theo kỳ (R13) — hàm của lõi

`pwa/src/people/period.ts` chỉ xuất lại hàm của `tsport/src/memorial_period.ts` (cổng của `memorial/src/period.rs`, có
trên `main` từ PR #14, 25fe94f): `eventsInPeriod`, `periodBounds`, `prevPeriod`, `nextPeriod` và các kiểu `Period`,
`PeriodItem`, `PersonFilter`. Bộ chuyển tạm đã bị xóa (MA3-15-01).

- `views.ts`: kỳ trước/sau dùng `.from` của `{from, to}`; mỗi mục là `PeriodItem` (`it.date` để chia nhóm tháng, `it.occ`
  để hiển thị).
- Bổ sung B4 do lõi áp: `personOccurrences` không sinh sinh nhật cho hồ sơ có ngày mất; `eventHasDeceased` bỏ sự kiện gắn
  ít nhất một người đã mất; bộ lọc 02 §7.6: "Người đã mất" = giỗ + tiên thường. PWA không còn đoạn lọc trùng nào.
  `snap()` chỉ giữ một tập `dead` để giao diện biết khi nào ẩn khối nhắc và hiện câu giải thích.
- Kiểm thử L5 so danh sách hiển thị với `eventsInPeriod` của lõi (4 loại kỳ × 3 bộ lọc). Sự kiện "Họp mặt chung" gắn
  [người đã mất, người đang sống] không có ở bộ lọc nào; nếu bỏ B4 thì sự kiện đó sinh 1 lần xuất hiện.

Phần không đổi: tuần bắt đầu Thứ Hai; kỳ gồm cả ngày đầu và ngày cuối; dòng chính của mục ("Nhãn · tên") lấy theo người
đầu tiên chỉ để **hiển thị**, không dùng cho bộ lọc.

### 8.3 Ảnh
Không `blob:` trong `img-src`; ảnh luôn vẽ vào `<canvas>` (`createImageBitmap(Blob)`). Tải tệp xuất (.ics, .lvhv) dùng
`URL.createObjectURL` cho thẻ `<a download>` — đây là điều hướng tải xuống, không phải nạp ảnh, nên không cần đổi CSP
(S2b giữ nguyên từng byte).

---

## 9. Kiểm thử tương ứng (pwa/test/smoke.mjs)

| Luồng | Kiểm |
|---|---|
| L1 | Dòng kết quả hiện "15/6 nhuận" và "Ất Tỵ" cho 08/08/2025; hồ sơ lưu `death.lunar = {2025, 6, 15, leap: true}`, `lunar_source = "converted"` |
| L2 | 15/5/2025 nhuận → câu "Năm 2025 không có tháng 5 nhuận"; năm 2026 → hộp nhuận ẩn; 15/6 nhuận 2025 → "08/08/2025"; lưu `lunar_source = "entered"` |
| L3, L4 | Hồ sơ đang sống + "Tốt nghiệp"; lần tới sinh nhật đúng |
| L5 | Bốn loại kỳ, tiêu đề đúng chuỗi, ‹ › Hôm nay, ba bộ lọc: tập mục mong đợi (tính độc lập trong kiểm thử bằng `tsport/dist/memorial.mjs`); "đã mất" ∪ "đang sống" = "tất cả" |
| L6, L7 | "Sắp tới" có mục; thẻ nhắc hiện khi đồng hồ trang đặt vào ngày `D − 3` (Playwright `clock`) |
| L8 | Hộp xác nhận có câu riêng tư và số người; tải được tệp `.ics` bắt đầu `BEGIN:VCALENDAR` |
| L9, L10 | Xuất → xóa → nhập "Thay toàn bộ" → `stateHash` như trước; sai mật khẩu → câu chung |
| L11 | Trang tưởng niệm: dòng trên tên = "Ông nội" (từ ô Quan hệ "ông nội (bên nội)"), tên không lặp cách gọi, "Hưởng thọ"; có 3 nén hương; 0 `<audio>/<video>` |
| B4 | Biểu mẫu sự kiện của người đã mất không có khối nhắc, có câu giải thích; hồ sơ người đã mất vẫn hiện sự kiện, có câu "Không nhắc sinh nhật và sự kiện của người đã mất"; sự kiện gắn [đã mất, đang sống] không có trong Sắp tới, Theo kỳ, .ics; bộ lọc "Người đã mất" chỉ có giỗ/tiên thường; L5/L6 so với tập mong đợi đã áp B4 |
| US-M-07, R-PRIV-2 | Người đang sống → "Cập nhật: đã mất" → cùng `id`, `rev` 2, giữ ngày sinh và sự kiện; sinh nhật và sự kiện của người đó không còn trong Sắp tới và Theo kỳ; ảnh giấy báo tử không còn APP1/APP13; chuỗi canh gác trong `notice` không có trong tệp .ics (nội dung, xem trước, tên tệp), DOM màn hình tưởng niệm và thẻ nhắc; hộp tạo sao lưu có câu về thông tin báo tử; tài liệu sao lưu có `notice` qua kiểm tra, ảnh không tồn tại → V3, chuỗi rỗng → V1 |
| TK-28, TK-29 | Không có ô `type=time`; giờ 25 → lỗi; 18 : 5 → lưu "18:05"; sinh nhật âm mặc định "giống", khi lưu chép cài đặt sinh nhật dương với `id` riêng |
| TK-31 (b) | 1280 px: thẻ nhắc trên Tờ lịch cùng cạnh trái và cùng chiều rộng với tờ lịch |
| TK-35 | Tên dài trên bia không tràn; nhắc sao lưu đến hạn ("Sao lưu ngay"); 5 lời nhắc hôm nay → 3 thẻ + "… và 2 lời nhắc khác", Tờ lịch → 1 thẻ + "Còn 4 lời nhắc khác hôm nay — Xem"; kho phiên bản mới hơn → "Không mở được dữ liệu trên máy…", Tờ lịch vẫn mở được |
| L12, L13 | Sửa tăng `rev`; xóa có hộp xác nhận, hủy thì còn, đồng ý thì mất |
| Chung | CSP từng byte (S2b), 0 URL ngoài, 0 lỗi console, P3 < 500 KB, P6 (0 tệp mô-đun ở lần tải đầu), ảnh chụp 360 × 780 và 1280 × 800, sáng và tối |
| Bố cục (TK-02) | 11 màn hình × 360/1280 px × sáng/tối ở A, và ở A++: `scrollWidth`, `clientWidth`, `visualViewport.width`, `innerWidth` ≤ chiều rộng khung nhìn đã đặt; cạnh phải thanh mục đáy ≤ khung nhìn; mọi ảnh chụp rộng đúng 2 × khung nhìn |
| Vùng chạm (TK-05) | Mọi phần tử bấm được trong vùng nội dung của mô-đun ≥ 44 × 44 px, ở A (360/1280) và A++ (360) |
| Biểu mẫu (TK-03, TK-24, TK-35) | Lưu khi trống → gõ tên → lỗi dưới ô mất, khung tóm tắt bỏ dòng; rời ô trống → báo lỗi; không còn ô "dòng đầu bia"; ô Quan hệ gợi ý từ thường ngày, không có từ Hán-Việt cổ; ảnh không đọc được → câu lỗi |
| Hộp thoại (TK-11, TK-14, TK-06, TK-15) | Câu riêng tư .ics đúng loại dữ liệu, xem trước có tên người; lỗi mật khẩu dưới từng ô; nút "Chọn tệp sao lưu…" + "Đã chọn: …"; Tờ lịch chỉ một thẻ nhắc + dòng gộp |

## 10. Ánh xạ yêu cầu → câu chuyện (07) → trang → kiểm thử

| Câu chuyện (07 §2) | Yêu cầu / tiêu chí | Trang / thành phần | Luồng, kiểm thử |
|---|---|---|---|
| US-M-01 thêm bằng ngày mất dương, thấy ngay ngày âm (có "nhuận") và giỗ năm nay | 01 §3.2; 02 §1; R1, R2, R10 | §3.4 ô ngày mất (kết quả trực tiếp); §3.6 "Lần tới" | L1 |
| US-M-02 thêm bằng ngày âm (tháng nhuận, ngày 30), lỗi bằng lời thường | 01 §2.1, §6.1; R1, R2(c); R-UX-1 | §3.4 ô ngày âm, hộp nhuận, bảng lỗi | L2 |
| US-M-03 giỗ đầu / giỗ hết / giỗ thường; hưởng thọ / hưởng dương | 02 §3–§4; R10; O-2, O-3, O-4 | nhãn §5.1; §3.4 tuổi; §3.6; §3.9 | L1, L5, L11 |
| US-M-04 nhắc 1/3/5/10/15 ngày hoặc tự đặt, đúng giờ | 01 §3.6; 02 §7.1, §7.5; R3, R5 (PWA: thẻ trong trang); O-1 | §3.4 nhóm nhắc; §5.4; §5.5 | L1, L7 |
| US-M-05 màn hình tưởng niệm: bia, ảnh, mâm, ba nén hương | 05 §1; R8 (a)–(f); R9; R-CUL-2 | §3.9 | L11 |
| US-M-06 nơi an táng, quan hệ, tên hiệu, ảnh chân dung | 01 §3.1, §3.8; S9; S2b; R-CSP-1 | §3.4 | L1 (kiểm byte JPEG lưu không có APP1) |
| US-S-01 người thân với ngày sinh dương, tùy chọn sinh nhật âm | 01 §3.3; 02 §5–§6; R10 | §3.5 | L3 |
| US-S-02 sự kiện riêng, lặp hoặc một lần, dương hoặc âm | 01 §3.4–§3.5 | §3.7 | L4 |
| US-S-03 sinh nhật lần thứ mấy, tuổi bao nhiêu | 02 §4.3, §6; R10 | §3.5 kết quả trực tiếp; §3.6 "Lần tới"; nhãn §5.1 | L3 |
| US-C-01 danh sách tuần / tháng / quý / năm, lùi/tới, về hôm nay | 04 B2 R13; 07 §9 câu 2–3 (Thứ Hai; dương lịch + ngày âm); R-UX-2 | §3.2 | L5 |
| US-C-02 lọc tất cả / đã mất / đang sống | 04 B2 R13 | §3.2 bộ lọc; §8.2 | L5 |
| US-C-03 "Sắp tới" 30 ngày khi mở mô-đun | 02 §7.2; R11 | §3.1 | L6 |
| US-C-04 xuất .ics | 02 §8; R4; S10; O-5; R-PWA-2 | §3.8 A; §5.4 | L8 |
| US-C-05 sao lưu có mật khẩu, khôi phục | 03; R6; S10; R-DATA-2 | §3.8 B, C, D; §5.6 | L9, L10 |
| US-C-06 sửa, xóa; xóa phải hỏi lại | 01 §1 (`rev`); S10 | §3.4–§3.7; §3.10 | L12, L13 |
| US-C-07 không tài khoản, không mạng, dữ liệu chỉ trên máy | YC-2026-02; S4', S7, R7; 01 §4.1 (`persist()`); R-PWA-1 | §1.2 (URL không có tên/ngày); §1.3; §5.6 | Kiểm `localStorage`, 0 yêu cầu ngoài, kết quả `persist` có ghi |
| (phi chức năng 07 §4) tải lần đầu < 500 KB, mô-đun tải lười | P3, P6 | §1.3 | Danh sách tệp lần tải đầu, số KB |
| (phi chức năng 07 §4) trợ năng, cỡ chữ, giảm chuyển động | 07 §4 | §6, §7 | Không tràn ngang 360 px A++; `reducedMotion` |
| (O-9) màn hình khóa không hiện tên | 06 O-9 | Không áp dụng PWA (không có thông báo hệ thống); tiêu đề tab tưởng niệm không có tên | — |

## 11. Ngoài phạm vi bản này / giới hạn đã biết

- Chọn kiểu mâm (Phật giáo / truyền thống / Công giáo): dữ liệu có trường, giao diện chưa cho chọn — một kiểu "truyền
  thống" (theo yêu cầu "một kiểu là đủ"; R-CUL-2: thêm phong cách ở đợt 4). Công giáo đến từ sao lưu được vẽ không hương.
- Sự kiện gắn nhiều người: dữ liệu cho phép tới 50 người; giao diện bản này chỉ tạo/sửa sự kiện của một người.
- Đổi người đang sống thành người đã mất (01 §3.1): chưa có nút; làm ở bản sau.
- Công tắc chung `reminders_enabled` và mặc định nhắc cho hồ sơ mới (`default_offsets_days`, `default_time_local`):
  dùng giá trị trong cài đặt, chưa có màn hình sửa.
- Danh sách theo **tháng/năm âm lịch** (07 §9 câu 3): để đợt sau theo mặc định của chủ dự án.
- Đo R8 (CPU, số lần rAF) và P4 trạng thái "200 hồ sơ" / "tưởng niệm sau 120 s": chưa có trong kiểm thử khói đợt này.

## 12. Sai khác khi cài đặt (ghi trong cùng PR, 2026-11-07; cập nhật sau duyệt thiết kế cùng ngày)

Các điểm dưới đây được đổi hoặc làm rõ khi cài; phần trên đã sửa theo, mô tả PR liệt kê lại.

1. **Thẻ nhắc khi mở ứng dụng** chỉ hiện ở "Tờ lịch" và "Sắp tới" (không ở mọi trang): ở các trang biểu mẫu, hồ sơ,
   tưởng niệm, thẻ nhắc chen vào đầu trang làm rối và đẩy nội dung xuống. L7 đã sửa câu chữ.
2. **Dòng "Dương lịch"** trên hồ sơ người đã mất chỉ hiện khi ngày mất nhập theo âm lịch (khi nhập dương, nó trùng dòng
   "Ngày mất (đã nhập)").
3. **Ghi chú** của hồ sơ và của sự kiện nằm trong nhóm đầu của biểu mẫu (không có nhóm "Ghi chú" riêng — nhãn bị lặp).
   Nhóm nhắc có hộp "Bật nhắc" thay vì lặp lại tiêu đề nhóm.
4. **Thông báo ngắn** ("Đã lưu hồ sơ.", "Đã xóa hồ sơ.") được mang sang trang kế tiếp khi lưu/xóa xong rồi chuyển trang.
5. **Khôi phục "Thay toàn bộ"** thay cả cài đặt bằng cài đặt trong tệp (hành vi của lớp lưu trữ đợt 2, 01 §6.3), nên
   "Lần sao lưu gần nhất" trở về giá trị lúc xuất tệp (thường là "Chưa sao lưu lần nào"). Chưa sửa ở đợt này — cần quyết
   định của vai lưu trữ (giữ `last_backup_at` của máy hay của tệp).
6. **R8 trong kiểm thử khói** chỉ kiểm phần đo được nhanh: khói là CSS (`smokeRise`), nút dừng, 0 lần
   `requestAnimationFrame`, giảm chuyển động → tĩnh, không audio/video. Đo CPU 120 s / 60 s theo 04 R8 chưa làm.

### 12b. Sau duyệt thiết kế (`docs/reviews/DUYET-THIET-KE-DOT3-PWA.md`, PR #18) — đã sửa trong PR #15

| Mã | Việc đã làm |
|---|---|
| TK-01 (chặn) | Ô chọn tệp ẩn không còn làm trang tràn: luật chiều rộng chỉ áp cho `input[type=file]:not(.sr-only)`; `.sr-only` bền hơn (`!important`, `clip-path`, `left/top: 0`). Ô chọn ảnh và ô chọn tệp sao lưu đều là ô ẩn + **nút thật** (bàn phím bấm được) |
| TK-02 (chặn) | Kiểm thử khói đo bố cục so với **chiều rộng cố định** của khung nhìn (`scrollWidth`, `clientWidth`, `visualViewport.width`, `innerWidth`), cạnh phải của thanh mục đáy, và **chiều rộng ảnh chụp** = 2 × khung nhìn; chạy cho 11 màn hình × 360/1280 px × sáng/tối ở A, và ở A++. Đã chứng minh phép kiểm mới **bắt được** TK-01 trước khi sửa (bản dựng `dbf215b`: phép cũ báo đạt, phép mới báo 395/401/1594 px) |
| TK-03 … TK-15, TK-17 … TK-19 | Như các mục ở §2, §3.1–§3.9, §5.5, §6 đã sửa ở trên (đánh dấu mã TK) |
| TK-16 | **Chưa làm trong PR này — chuyển vai lưu trữ.** Quyết định của điều phối: sau "Thay toàn bộ", giữ lịch sử sao lưu của máy (`last_backup_at`, `backup_nag_days`) thay vì lấy từ tệp. Không làm được cục bộ ở PWA vì: (1) `lvhv-store` (Rust, `apply_import`) thay cả `settings` bằng tệp — PWA đổi riêng sẽ làm hai bản lệch nhau; (2) R6 (a)/(d) so `stateHash` "giống hệt" giữa kho trước và sau, và giữa Rust và PWA; bộ R6 có `backup_nag_days = 60`, `last_backup_at = 2026-10-01T00:00:00Z` nên quy tắc mới đổi kết quả R6. Cần Bổ sung cho 01 §6.3 / 03 §2.4 và định nghĩa lại R6 (a)/(d), sửa cả hai bản cài đặt |
| Góp ý đã làm | TK-20 (nút chính hình viên thuốc, một nút chính mỗi khung), TK-21 (chữ quay lại theo đích: "‹ Về danh sách hồ sơ" / "‹ Về hồ sơ"), TK-22 (một kiểu "đang chọn", đủ tương phản ở tối), TK-23, TK-24 (PHONG-TUC.md; phần gợi ý dòng đầu bia đã bỏ theo quyết định của chủ dự án — §12e), TK-25, TK-26 (biểu mẫu căn giữa; danh sách Theo kỳ ≤ 760 px), TK-27, TK-30, TK-32 (bỏ tiêu đề "Ngày sinh" lặp — giữ `legend` cho trình đọc màn hình), TK-33 (năm "chỉ biết năm" 1000–3000, có ích cho tổ tiên — §3.4 sửa theo), TK-34 (Lần tới: mọi lần trong 12 tháng, tối đa 5 mục), TK-35 một phần (kiểm thử lỗi ảnh; bảng tương phản §6) |
| Thêm (do phép kiểm mới) | Ở 1280 px cỡ A+/A++, thanh mục (5 mục, chữ không xuống dòng) đè lên nút cỡ chữ — thanh mục chuyển xuống dòng riêng dưới thanh trên cùng khi A+/A++ |

### 12c. Góp ý chưa làm và lý do

| Mã | Lý do |
|---|---|
| TK-28, TK-29, TK-35 | Đã làm ở vòng sau — xem §12e |
| TK-31 (a) | Chấm ngày giỗ/sinh nhật trên lịch tháng của Tờ lịch **đổi giao diện v0.1 mà người dùng thấy** — dừng lại, báo chủ dự án (§12e) |

### 12d. Sau duyệt mã (`docs/reviews/DUYET-MA-DOT3.md`, PR #19)

| Mã | Mức | Việc |
|---|---|---|
| MA3-15-01 | chặn | Đã làm (sau PR #14): dùng `eventsInPeriod`/`periodBounds`/`prevPeriod`/`nextPeriod` của lõi, xóa bộ chuyển; L5 so với lõi; xem §8.2 |
| MA3-15-02 | chặn | Đã sửa: khôi phục kiểm `File.size` > 72 + 128 MiB + 16 **trước** khi đọc tệp → "Sai mật khẩu hoặc tệp bị hỏng" (S12). Kiểm thử: tệp thưa 130 MiB → thông báo chung, 0 lần đọc tệp; giới hạn khớp hằng số của `backup-crypto` |
| MA3-15-03 | nên sửa | Đã sửa: trong lúc mã hóa/khôi phục, Esc không đóng hộp thoại (`cancel` bị chặn, `aria-busy`); kiểm thử nhấn Esc ở cả hai việc. Đã thử ngược: bỏ chặn thì kiểm thử báo lỗi |
| MA3-15-04 | nên sửa | Đã sửa: chạm mục → so tên hồ sơ mở ra; nhóm tháng so đúng danh sách tháng; R13 (c) đo trên giao diện ba bộ lọc |
| MA3-15-05 | góp ý | Đã sửa: thẻ nhắc dùng `allReminders` của lõi (múi giờ của máy), lọc lời nhắc có ngày địa phương = hôm nay |
| MA3-15-06 | góp ý | Đã sửa: "Lần sao lưu gần nhất" và ngày xuất trong xem trước khôi phục theo giờ của máy |
| MA3-15-07 | góp ý | Đã sửa: đọc kích thước điểm ảnh từ phần đầu tệp (PNG, GIF, JPEG, WebP); > 12 000 px mỗi cạnh hoặc > 50 MP → "Ảnh quá lớn (w × h điểm ảnh) — hãy chọn ảnh nhỏ hơn", không giải mã |
| MA3-15-08 | góp ý | Đã sửa: xóa mật khẩu khỏi ô sau khi xuất. Thẻ nhắc trên trình duyệt không có `indexedDB.databases()`: cờ "đã dùng Người thân" (`people: true`) trong `lvhv.ui`. `weekdayOf` (và tuần của bộ chuyển) dùng `jd.weekday` của lõi (§12e). **Chưa:** `metrics.json` vẫn là tệp có trong kho — xem §12e |

### 12e. Vòng sửa theo yêu cầu của chủ dự án (07/11/2026)

**Từ ngữ (quyết định của chủ dự án).** Không dùng từ Hán-Việt cổ, ít người hiểu ("Hiển khảo"…). Ứng dụng dùng cách gọi
thường ngày: ông/bà/cụ/kỵ…

- Bỏ hẳn ô "Dòng đầu bia" và mọi gợi ý Hán-Việt khỏi biểu mẫu.
- Bia ghi chữ ở ô "Quan hệ" phía trên tên (§3.9). Ô "Quan hệ" có gợi ý bằng từ thường ngày, ghi rõ cách gọi theo vùng:
  cụ/kỵ ở miền Bắc, cố/sơ ở miền Nam (nguồn ở PHONG-TUC.md §1).
- **Trường `memorial.bia_tieu_de` (01 §3.7) vẫn còn trong dữ liệu** để sao lưu dùng chung được giữa các nền tảng.
  PWA không hiển thị và không sửa trường này. Khi lưu hồ sơ, PWA giữ nguyên giá trị đang có.
- Rà câu chữ:
  - "Danh xưng" → "Cách gọi (đứng trước tên)";
  - "Quan hệ" → "Quan hệ với bạn";
  - "Tiên thường (lễ cúng hôm trước ngày giỗ)" ở hộp kiểm, tóm tắt nhắc và nhãn lần xuất hiện;
  - "Giỗ hết (giỗ lần cuối của thời gian để tang)";
  - "hưởng dương (mất sớm) thay cho hưởng thọ";
  - "kị" → "kỵ".
  - Các từ thường ngày giữ nguyên: ngày giỗ, giỗ đầu, hưởng thọ, nơi an táng, pháp danh.

**Quyết định chờ chủ dự án xác nhận — TK-29.** Chọn cách đơn giản nhất, không đổi mô hình 01:

- Khi bật "Nhắc cả sinh nhật âm lịch", có thêm hộp "Nhắc sinh nhật âm lịch giống nhắc sinh nhật dương lịch", **mặc định bật**.
- Khi hộp này bật, lúc lưu ứng dụng chép `enabled`, `offsets_days`, `time_local` của `reminders.sinh_nhat` sang
  `reminders.sinh_nhat_am`, giữ `id` riêng của quy tắc âm.
- Bỏ chọn → hiện khối "Nhắc sinh nhật âm lịch" riêng như trước.
- Mở lại hồ sơ cũ: hộp được chọn sẵn khi hai quy tắc có cùng cài đặt, hoặc khi chưa có quy tắc âm.

**Đã làm:**

| Mã | Việc |
|---|---|
| TK-28 | Giờ nhắc là hai ô số 24 giờ (giờ : phút), báo lỗi khi ngoài khoảng; kiểm thử |
| TK-29 | Như trên; kiểm thử việc lưu và `id` riêng |
| TK-31 (b) | ≥ 960 px: thẻ nhắc trên Tờ lịch thẳng cột với tờ lịch (khung 936 px, thẻ ≤ 520 px bên trái); kiểm thử đo cạnh trái và chiều rộng |
| TK-35 | Kiểm thử: nhắc sao lưu đến hạn, > 3 thẻ nhắc (Sắp tới và Tờ lịch), lỗi kho (kho phiên bản mới hơn), tên rất dài trên bia (lớp `.long`). Ảnh chụp: `v02-backup-nag-360`, `v02-reminders-many-360`, `v02-sheet-reminders-many-1280`, `v02-store-error-360`, `v02-memorial-longname-360`. Khung dây biểu mẫu 1280 px ở dưới |
| MA3-15-08 | `weekdayOf` và tuần của bộ chuyển dùng `jd.weekday` của lõi |

Khung dây biểu mẫu ở 1280 px (TK-35):

```
┌ thanh trên: Lịch và Văn hóa Việt · Tờ lịch · Chọn ngày tốt · Đổi ngày · [Người thân] · Giới thiệu  A A+ A++ ◐ ┐
│                    ‹ Về hồ sơ                                                                               │
│                    Sửa hồ sơ                                                                                │
│                    Ông Nguyễn Văn A                                                                         │
│                    ┌──────────── một cột ≤ 680 px, căn giữa ────────────┐                                   │
│                    │ Người đã mất: Cách gọi · Họ và tên * · Tên hiệu ·   │                                   │
│                    │   Quan hệ với bạn (+ ghi chú về bia) · Ghi chú      │                                   │
│                    ├─────────────────────────────────────────────────────┤                                   │
│                    │ Ngày mất *: [Dương | Âm]  Ngày Tháng Năm → kết quả  │                                   │
│                    ├─────────────────────────────────────────────────────┤                                   │
│                    │ Ngày sinh và nơi sinh                               │                                   │
│                    │ Nơi an táng và ảnh                                  │                                   │
│                    │ Nhắc ngày giỗ (mốc, giờ 24 giờ, tiên thường)        │                                   │
│                    │ ▸ Tưởng niệm (giỗ hết, lời tưởng niệm, ngưỡng)      │                                   │
│                    └─────────────────────────────── [ Hủy ] [  Lưu  ] ──┘                                   │
└─────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

**Dừng lại, báo chủ dự án (không làm):**

| Mã | Vì sao |
|---|---|
| TK-31 (a) | Chấm ngày giỗ/sinh nhật trên lịch tháng của Tờ lịch **đổi giao diện lịch v0.1 mà người dùng thấy**. Theo quy tắc của vòng này, việc đó phải hỏi trước. Nếu được đồng ý: thêm một chấm nhỏ dưới số ngày, tính một lần cho cả tháng |
| MA3-15-08 (`metrics.json`) | Bỏ `pwa/test/metrics.json` khỏi kho **đổi tệp số đo v0.1 đang được theo dõi** và đổi đường dẫn tạo phẩm CI mà các vòng duyệt trước dùng → cần điều phối quyết định. PR này không commit số đo mới |
| TK-16 | Vẫn thuộc vai lưu trữ (§12b) |
| MA3-15-01 | Đã làm ở vòng sau (§8.2, §12h) |

### 12f. Bổ sung B4 — với người đã mất chỉ nhắc ngày giỗ (04, PR #20, quyết định của chủ dự án 07/11/2026)

- Hồ sơ người đã mất (§3.6): sinh nhật và sự kiện vẫn hiện. Dưới danh sách sự kiện có câu "Không nhắc sinh nhật và sự
  kiện của người đã mất — chỉ nhắc ngày giỗ. Thông tin vẫn được lưu và sao lưu." "Lần tới" chỉ có giỗ/tiên thường.
- Biểu mẫu sự kiện (§3.7): sự kiện của người đã mất không có khối nhắc.
- Theo kỳ (§3.2): "Người đã mất" chỉ có ngày giỗ (và tiên thường), có câu giải thích dưới bộ lọc.
- Quy tắc do lõi áp (§8.2); PWA không lọc lại.
- Ảnh chụp mới: `v02-form-event-deceased-360`, `v02-person-deceased-b4-360`, `v02-period-deceased-filter-360`.

### 12g. YC-2026-05 — "Cập nhật: đã mất" và thông tin báo tử (01 §3.2a, US-M-07, R-PRIV-2)

**Trang hồ sơ người đang sống (§3.6).** Đầu trang có thêm nút "Cập nhật: đã mất" (kiểu nút phụ, cạnh "Sửa").
Nút mở `#/nguoi-than/bao-tu/<id>`.

**Biểu mẫu "Cập nhật: đã mất"** — `#/nguoi-than/bao-tu/<id>`:

```
‹ Về hồ sơ
Cập nhật: đã mất
Đỗ Văn M
Ghi lại ngày mất của Đỗ Văn M. Hồ sơ, ảnh và ngày sinh được giữ nguyên; ứng dụng sẽ nhắc ngày giỗ hằng năm.
┌ Ngày mất * ──────────────────────────────┐   như §3.4: dương/âm, nhuận, "Không rõ năm mất",
│ [Nhập theo dương lịch | Nhập theo âm lịch]│   dòng kết quả đổi lịch
│  Ngày  Tháng  Năm                         │
│  Dương lịch: Thứ Tư, 17/07/2024           │
└───────────────────────────────────────────┘
▾ Thông tin báo tử (không bắt buộc)            mở sẵn ở biểu mẫu này
   "Chỉ hiện trên trang hồ sơ này. Không đưa vào lời nhắc, tệp lịch (.ics) hay trang tưởng niệm; có trong tệp
    sao lưu (đã mã hóa)."
   Người báo tin (≤ 200) · Mất tại (≤ 200) · Ngày nhận tin (dương, không sau hôm nay) · Lý do (≤ 500, nhiều dòng)
   Ảnh giấy báo tử (tối đa 4): ảnh nhỏ + "Bỏ ảnh"; "Thêm ảnh giấy báo tử"
   Ghi chú (≤ 2000)
                                   [ Hủy ] [ Lưu ]
```

- Khi bấm "Lưu" sẽ hiện hộp xác nhận "Cập nhật <tên> là người đã mất?", liệt kê các thay đổi:
  - "Sẽ không còn nhắc sinh nhật và sự kiện của <tên>." (chỉ ghi "và sự kiện" khi người đó có sự kiện);
  - "Bắt đầu nhắc ngày giỗ hằng năm vào 12/6 âm lịch (đúng ngày, 3 ngày trước, lúc 07:00).";
  - "Họ tên, ảnh, ngày sinh và sự kiện vẫn được giữ trong hồ sơ.";
  - thêm một dòng về thông tin báo tử, nếu có nhập.
- Nút của hộp: "Lưu thay đổi" và "Quay lại".
- Lưu là cập nhật **cùng** hồ sơ (cùng `id`, tăng `rev`), không tạo hồ sơ mới. Các trường đổi:
  - `kind = "deceased"`; `death = {input, notice?}`, phần dẫn xuất do `buildPerson` điền;
  - `memorial` lấy mặc định;
  - `reminders = {gio: theo cài đặt chung (tiên thường tắt), sinh_nhat: null, sinh_nhat_am: null}`, `sinh_nhat_am = false`;
  - `noi_an_tang = ""`.
- Giữ nguyên: họ tên, cách gọi, quan hệ, ảnh, ngày sinh, ghi chú, sự kiện. Sự kiện không còn được nhắc (B4).
- Ảnh giấy báo tử và hồ sơ được ghi trong **một** giao dịch.

**Biểu mẫu người đã mất (§3.4).** Có cùng khối "Thông tin báo tử (không bắt buộc)", đặt sau "Nơi an táng và ảnh":
- thêm mới: **thu gọn**;
- sửa hồ sơ đã có thông tin báo tử: mở sẵn.

Bỏ một ảnh giấy báo tử rồi lưu thì ảnh đó bị xóa khỏi máy. Xóa hồ sơ thì xóa luôn mọi ảnh giấy báo tử.

**Hiển thị (R-PRIV-2).** Thông tin báo tử chỉ hiện ở khối thu gọn "Thông tin báo tử" trên trang hồ sơ người đã mất,
nằm dưới "Thông tin". Khối có:
- Người báo tin, Mất tại, Ngày nhận tin, Lý do, Ghi chú — chỉ những dòng có dữ liệu;
- ảnh giấy báo tử, vẽ bằng canvas, không cắt.

`snap()` bỏ `death.notice` trước khi đưa dữ liệu cho lõi. Vì vậy các chỗ sau **không** đọc được thông tin báo tử: thẻ
nhắc, Sắp tới, Theo kỳ, "Lần tới", tệp .ics, tên tệp, màn hình tưởng niệm. Hộp "Tạo tệp sao lưu" có câu: "Tệp gồm cả
thông tin báo tử (người báo tin, nơi mất, lý do, ảnh giấy báo tử) — chỉ ai biết mật khẩu mới đọc được."

**Lớp lưu trữ PWA** (`pwa/src/store`, giữ tên trường như 01 §3.2a để khớp lõi ở PR #14):

| Phần | Cách làm |
|---|---|
| `validate.deathNotice` | Đủ sáu khóa. Chuỗi đã cắt khoảng trắng, không rỗng, hoặc `null`. `notice_date` là ngày dương `day`. Tối đa 4 id ảnh, không trùng |
| Kiểm tra tài liệu | Mỗi id ảnh giấy báo tử phải có trong `photos`, nếu không → V3 |
| `putPerson(p, ảnh chân dung?, ảnh giấy báo tử[])` | Ghi ảnh và hồ sơ trong một giao dịch, kiểm ảnh tồn tại |
| `deletePerson` | Xóa cả ảnh giấy báo tử |

**Khóa `notice` khi không có thông tin báo tử — theo lõi (Rust là chuẩn).** `store/src/model.rs` khai báo
`notice` với `#[serde(default, skip_serializing_if = "Option::is_none")]`. Vì vậy `"notice": null` khi đọc ≡ vắng khóa,
và khi ghi thì không có khóa. Lớp lưu trữ PWA làm đúng như vậy: `normNotice` trong `pwa/src/store/db.ts` bỏ
`"notice": null` trước khi ghi, ở cả `putPerson` và `applyImport`. Nhờ đó `stateHash` và tệp xuất giống Rust từng byte.

Kiểm chéo bằng `scripts/check-store.sh` (CI "Lưu trữ + sao lưu"), gồm ba trường hợp:
- tệp có thông tin báo tử và tệp không có (vectơ bổ sung `supplement-yc05`: một hồ sơ có `notice`, một hồ sơ đã mất
  không có);
- tài liệu ghi `"notice": null` → `stateHash` = `state_sha256` của Rust;
- bộ R6 (không có `notice`) theo hai chiều Rust ↔ PWA.

Ảnh chụp: `v02-convert-form-360`, `v02-convert-confirm-360`, `v02-person-notice-360`, `v02-backup-notice-dialog-360`,
`v02-convert-form-1280-{light,dark}`, `v02-convert-form-360-dark`, `v02-person-notice-1280-{light,dark}`,
`v02-person-notice-360-dark`.

### 12h. Vòng sau khi PR #14 vào `main` (MA3-15-01, TK-16, O-9)

| Mã | Việc |
|---|---|
| MA3-15-01 | Như §8.2 |
| TK-16 | Lõi không đổi được cách "Thay toàn bộ" lưu `settings` (vectơ R6 đã khóa), nên chỉ sửa ở giao diện, không đổi dữ liệu đã lưu hay logic nhắc sao lưu. Sau khi "Thay toàn bộ", trang Sao lưu ghi hai dòng: "Chưa sao lưu lần nào trên máy này" (hoặc "Lần sao lưu gần nhất: …") và "Dữ liệu được khôi phục từ bản sao lưu ngày dd/mm/yyyy". Ngày lấy từ `exported_at` trong phần đầu tệp đã khôi phục. Ngày này được ghi ở khóa giao diện `ui_restored_from` của kho `meta` trong IndexedDB, giống `ui_persist_notice_hidden_at`: không thuộc tài liệu sao lưu, không vào `stateHash`. Kiểm thử: `stateHash` trước và sau khôi phục như nhau, hai dòng chữ đúng |
| O-9 `show_names` | Lõi lưu tùy chọn "hiện tên trên màn hình khóa" theo từng máy, mặc định **không** hiện (`lock_screen_names`, chỉ trong kho Rust). PWA không có thông báo trên màn hình khóa nên không có tùy chọn này. Thẻ nhắc trong ứng dụng vẫn ghi tên, vì chỉ hiện khi người dùng đã mở ứng dụng. Khung "Ứng dụng web" ở Sắp tới có câu giải thích (`#web-names`) để thống nhất với ứng dụng điện thoại và máy tính |
| MA3-15-03 (kiểm thử) | Kiểm thử Esc khi mã hóa trước đây đếm hộp còn mở, nên kết quả phụ thuộc thời gian mã hóa. Nay kiểm thử ghi lại sự kiện `cancel`: phải có ít nhất một lần nhấn Esc khi hộp đang bận và lần đó bị chặn |
| TK-31 (a), `metrics.json` | Giữ nguyên như hiện tại; điều phối sẽ hỏi chủ dự án |
