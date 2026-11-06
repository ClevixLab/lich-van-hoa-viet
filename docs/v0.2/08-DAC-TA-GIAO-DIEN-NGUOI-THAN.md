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
| **Mục sự kiện** (`.pitem`) | Một nút rộng cả hàng. Trái: ô ngày dương (số ngày lớn, thứ viết tắt "T5", tháng "Th 11"). Giữa: dòng 1 = nhãn + tên ("Giỗ đầu · Ông Nguyễn Văn A"); dòng 2 = "26/11/2026 · 7/10 âm lịch"; dòng 3 (nếu có) = phụ ("Hưởng thọ 86 tuổi", "Kỷ niệm 5 năm"). Phải: biểu tượng loại + "Còn 3 ngày" / "Hôm nay" / "Ngày mai" / "Đã qua 2 ngày". Chạm → mở hồ sơ người đó (sự kiện → hồ sơ của người đầu tiên trong `person_ids`) |
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
6. Khung "Lời nhắc trên ứng dụng web" (§5.4) — thu gọn được, mở sẵn lần đầu.
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
│            [ Hôm nay ]                   │  tắt khi kỳ đang xem chứa hôm nay
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
- Bộ lọc: **Người đã mất** = hồ sơ `kind = "deceased"`; **Người đang sống** = `kind = "living"`; sự kiện theo trạng
  thái của hồ sơ chứa nó (§8.2). Không phụ thuộc công tắc nhắc.
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
  - Đã mất: `Giỗ 15/6 âm lịch · lần tới 18/07/2027 (còn 253 ngày)`; không rõ năm mất: `Giỗ 10/3 âm lịch (không rõ năm mất)`.
  - Đang sống: `Sinh 20/01/1990 · 36 tuổi` (hoặc `Sinh năm 1990`).
- Sắp xếp trong mỗi khung: theo `ho_ten` (so sánh tiếng Việt của trình duyệt, `localeCompare(…, "vi")`).
- Trống từng khung: "Chưa có ai." + nút thêm.

### 3.4 Thêm / sửa người đã mất — `#/nguoi-than/them/da-mat`, `#/nguoi-than/sua/<id>`

Tiêu đề: "Thêm người đã mất" / "Sửa hồ sơ". Biểu mẫu chia nhóm (`fieldset` + `legend`):

**Nhóm "Người đã mất"**

| Trường | Điều khiển | Bắt buộc | Kiểm tra → câu báo lỗi |
|---|---|---|---|
| Danh xưng | ô chữ + gợi ý (`datalist`): cụ, kị, ông, bà, bác, chú, thím, cô, dì, cậu, mợ, anh, chị, em, bố, mẹ, cha, má, ba | không | > 40 ký tự → "Danh xưng tối đa 40 ký tự" |
| Họ và tên | ô chữ, `autocomplete="off"` | **có** | rỗng sau khi cắt khoảng trắng → "Họ tên không được để trống"; > 120 → "Họ tên tối đa 120 ký tự" |
| Tên hiệu / pháp danh | ô chữ | không | > 120 → "Tên hiệu tối đa 120 ký tự" |
| Quan hệ | ô chữ, gợi ý "ví dụ: ông nội, bà ngoại bên mẹ" | không | > 120 → "Quan hệ tối đa 120 ký tự" |
| Ghi chú | ô nhiều dòng | không | > 2000 → "Ghi chú tối đa 2000 ký tự" |

Mọi chuỗi chuẩn hóa NFC và cắt khoảng trắng hai đầu trước khi kiểm và lưu (01 §1); ký tự điều khiển bị bỏ.

**Nhóm "Ngày mất" (bắt buộc)**

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
| Giờ nhắc | `input type=time` | 07:00 | — |
| Tiên thường | hộp kiểm "Nhắc thêm ngày tiên thường (chiều hôm trước ngày giỗ)" | **tắt** (O-1) | — |

**Nhóm "Tưởng niệm" (thu gọn, `details`)**

| Trường | Điều khiển | Mặc định |
|---|---|---|
| Giỗ hết vào | radio "Năm thứ 2 sau năm mất" / "Năm thứ 3" | năm thứ 2 (O-2) |
| Dòng đầu bia | ô chữ ≤ 60, gợi ý: Hiển khảo, Hiển tỷ, Cố | trống |
| Lời tưởng niệm | ô nhiều dòng ≤ 2000 | trống |
| Tuổi dưới bao nhiêu ghi "hưởng dương" | ô số 1–150 | 60 (O-3) |

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
| Ngày sinh | ô nhập ngày: Dương/Âm × "Biết đủ ngày"/"Chỉ biết năm" | không | như §3.4; sau hôm nay → "Ngày sinh không được sau hôm nay" |
| Nơi sinh | ô chữ ≤ 300 | không | — |
| Sinh nhật âm lịch | hộp kiểm "Nhắc cả sinh nhật âm lịch hằng năm" | tắt | Chỉ bật được khi biết đủ ngày sinh; nếu không: hộp mờ, chú thích "Cần biết đủ ngày sinh" |
| Sinh 29/02 | radio "Năm không nhuận: mừng 28/02" (mặc định) / "… mừng 01/03" | — | chỉ hiện khi ngày sinh dương là 29/02 |
| Nhắc sinh nhật | như "Nhắc ngày giỗ" (không có tiên thường); nếu bật sinh nhật âm có thêm một khối "Nhắc sinh nhật âm lịch" | bật, `[0, 3]`, 07:00 | như §3.4 |

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
| Lặp lại hằng năm | hộp kiểm; mặc định theo loại (Ngày cưới: bật) | — | Không rõ năm mà tắt lặp → "Sự kiện không rõ năm phải lặp hằng năm" |
| Nhắc | như §3.4 (không tiên thường) | bật, `[0, 3]`, 07:00 | như §3.4 |
| Ghi chú | ô nhiều dòng ≤ 2000 | không | — |

`person_ids` = [người đang xem]. Lưu xong → về hồ sơ người đó.

### 3.8 Sao lưu và lịch điện thoại — `#/nguoi-than/sao-luu`

Ba khung:

**A. "Để điện thoại tự nhắc: xuất lịch (.ics)"** — chú thích §5.4; nút "Xuất tệp lịch (.ics)…". Bấm → hộp xác nhận
(S10, 02 §8.2):
```
Xuất tệp lịch
Tệp này chứa tên và ngày giỗ của 3 người. Bất kỳ ai có tệp đều đọc được.
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
⚠ Quên mật khẩu là mất dữ liệu trong tệp — không ai lấy lại được, kể cả Clevix Lab.
Gợi ý: dùng một câu dễ nhớ, ví dụ "nhà ở Hà Đông có cây bàng".
                               [Hủy]  [Tạo và tải tệp]
```
Lỗi: < 8 ký tự → "Mật khẩu cần ít nhất 8 ký tự"; không khớp → "Hai lần nhập mật khẩu chưa giống nhau". Khi chạy: nút
tắt, thanh tiến độ `<progress>` + "Đang mã hóa… (có thể mất vài giây)". Xong: tải tệp
`LichVanHoaViet-saoluu-<YYYY-MM-DD>.lvhv`, hộp đóng, thông báo "Đã tạo tệp sao lưu. Hãy cất tệp ở nơi an toàn (ổ đám
mây, USB) — tệp không tự gửi đi đâu." Ô mật khẩu: `type=password`, `autocomplete="new-password"`,
`autocapitalize="off"`, `spellcheck="false"`.

**C. "Khôi phục từ tệp sao lưu"** — nút "Chọn tệp sao lưu…" (`accept=".lvhv"`), ô mật khẩu, nút "Mở tệp". Lỗi theo
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
│        │     HIỂN KHẢO      │       │   bia: đỉnh vòm, nền đá sẫm, chữ vàng đồng,
│        │   Ông Nguyễn Văn A │       │   chữ serif; tên hiệu ở dòng nhỏ
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

- Năm trên bia: `Can Chi năm âm (năm)` cho năm sinh (nếu có) và năm mất (nếu có); thiếu năm sinh → chỉ "Mất năm Ất Tỵ
  (2025)"; không rõ năm mất → "Giỗ ngày 10/3 âm lịch".
- Tuổi: "Hưởng thọ 86 tuổi" / "Hưởng dương 45 tuổi" (+ "(khoảng)"); không có tuổi → bỏ dòng.
- Hình: SVG/CSS viết trong mã (không tệp ảnh ngoài), tông nâu gỗ và vàng đồng, cùng một kiểu ở sáng và tối (nền tối
  ấm cả hai giao diện để bia nổi). Ảnh vẽ bằng canvas; không có ảnh → khung bầu dục trống có chữ cái đầu tên.
- Khói (R8): ba vệt khói CSS (`@keyframes` dịch lên + mờ dần), **không dùng `requestAnimationFrame`**. Khói chạy khi
  trang hiển thị, **tự dừng sau 2 phút** (sau đó khói đứng yên dạng tĩnh), dừng ngay khi tab ẩn
  (`visibilitychange`) hoặc rời trang; trở lại tab trong 2 phút đầu → chạy tiếp tới hết 2 phút tính từ lúc mở. Nút
  "Dừng khói"/"Thắp lại" (thắp lại = chạy thêm 2 phút). `prefers-reduced-motion: reduce` → khói tĩnh ngay từ đầu,
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
6. Bộ lọc "Người đã mất" → chỉ còn giỗ/tiên thường (và sự kiện của người đã mất); "Người đang sống" → sinh nhật, sự kiện
   của người đang sống; "Tất cả" → hợp hai danh sách.
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
khóa; PWA không có thông báo màn hình khóa). Nhiều thẻ cùng ngày: xếp theo thời điểm nhắc rồi thứ tự 02 §7.2; quá 3 thẻ
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
- **Vùng chạm** ≥ 44 × 44 px.
- `lang="vi"` có sẵn ở trang; tên người hiển thị nguyên văn (không viết hoa tự động ngoài dòng đầu bia).

## 7. Bố cục đáp ứng

| | 360 px (điện thoại) | 1280 px (máy tính) |
|---|---|---|
| Khung trang | Một cột, lề 16 px, bề rộng tối đa 640 px | Bề rộng tối đa 960 px, căn giữa |
| Sắp tới | Danh sách một cột | Hai cột: trái danh sách 30 ngày; phải (320 px) giới hạn web + nút thêm + nhắc sao lưu |
| Theo kỳ | Hai nhóm nút chồng nhau, tiêu đề kỳ một dòng (có thể xuống 2 dòng ở A++) | Hai nhóm nút cùng hàng; danh sách một cột rộng tối đa 720 px |
| Hồ sơ (danh sách) | Hai khung chồng | Hai khung cạnh nhau |
| Biểu mẫu | Một cột; ba ô ngày một hàng | Một cột rộng tối đa 640 px |
| Hồ sơ một người | Một cột (§3.6) | Hai cột: thông tin trái, "Lần tới" + sự kiện + nhắc phải |
| Tưởng niệm | Bia rộng 80 % màn hình | Bia rộng 360 px, căn giữa, nền tối phủ cả vùng nội dung |
| Hộp xác nhận | Rộng `100vw − 24px` | Rộng tối đa 480 px |

---

## 8. Ghi chú kỹ thuật ràng buộc giao diện

### 8.1 Tính toán
- Lần xuất hiện, nhãn, thứ tự, câu nhắc, .ics: **chỉ** dùng hàm của lõi `tsport/src/memorial*` (`occurrencesWithTt`,
  `upcoming`, `reminderTextNamed`, `generateIcs`, `tuoiDuong`, `tuoiAm`…), không tự viết lại thuật toán ở giao diện.
- Lưu trữ: `pwa/src/store` (đợt 2): `buildPerson`, `putPerson`, `putEvent`, `putEventType`, `deletePerson`,
  `deleteEvent`, `exportBackup`, `openBackup`, `applyImport`, `backupReminderDue`, `persistResult`.

### 8.2 Danh sách theo kỳ (R13) — bộ chuyển tạm
`eventsInPeriod(cal, dataset, kind, anchor, filter) → {from, to, items}`, `periodBounds(kind, anchor)` và hàm kỳ
trước/sau do vai B1 thêm vào `tsport` (nhánh `v0.2-w3-core`). Tới khi hàm đó có trên `main`, PWA dùng **một mô-đun
chuyển tạm duy nhất** `pwa/src/people/period.ts` dựng cùng kết quả từ `occurrencesWithTt`; đổi sang hàm của `tsport`
là sửa **một dòng** `export … from` trong tệp đó. Quy ước của bộ chuyển (phải khớp hàm của B1 khi thay):
- Tuần bắt đầu Thứ Hai; kỳ gồm cả ngày đầu và ngày cuối.
- Bộ lọc của sự kiện: theo hồ sơ **đầu tiên** trong `person_ids` (giao diện chỉ tạo sự kiện một người); sự kiện không
  còn người nào: thuộc "Người đang sống" (để "đã mất" ∪ "đang sống" = "tất cả", không trùng). Nếu hàm của B1 quy ước
  khác, theo hàm của B1 và sửa dòng này.

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
| L11 | Trang tưởng niệm có bia, tên, "Hưởng thọ"; có 3 nén hương; 0 `<audio>/<video>` |
| L12, L13 | Sửa tăng `rev`; xóa có hộp xác nhận, hủy thì còn, đồng ý thì mất |
| Chung | CSP từng byte (S2b), 0 URL ngoài, 0 lỗi console, P3 < 500 KB, P6 (0 tệp mô-đun ở lần tải đầu), không tràn ngang 360 px A++ ở mọi trang mới, ảnh chụp 360 × 780 và 1280 × 800, sáng và tối |

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

## 12. Sai khác khi cài đặt (ghi trong cùng PR, 2026-11-07)

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
