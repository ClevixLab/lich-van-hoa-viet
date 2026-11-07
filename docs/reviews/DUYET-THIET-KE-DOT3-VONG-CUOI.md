# Duyệt thiết kế vòng cuối — mô-đun "Người thân" (PWA vòng 2, Android, Windows) — v0.2 đợt 3

Vai: **bên duyệt thiết kế**, cùng vai và cùng cách làm như `DUYET-THIET-KE-DOT3-PWA.md` và
`DUYET-THIET-KE-DOT3-NATIVE.md` (PR #26). Bên duyệt không viết đặc tả và không viết mã của các PR này. Ngày duyệt: 2026-11-07.

| Nền tảng | PR | Bản xem | Ảnh đã xem |
|---|---|---|---|
| PWA vòng 2 | #30 | `v0.2-w3-pwa2` @ `9e590d8` | 87 ảnh `v02-*.png`: bên duyệt **tự dựng lại** từ `9e590d8` (`npm run test:smoke`: ĐẠT; `npm run test:upgrade`: ĐẠT — R14 bản web) vì ảnh không được commit; 12 ảnh khác bản trước và 4 ảnh mới (`attachment-download-dialog`, `person-living-trimmed`, `upgrade-014`, `upgrade-02dev`, `upgrade-readonly`) xem kỹ, các ảnh còn lại so từng byte với bộ ảnh vòng trước |
| Android | #24 | `v0.2-w3-android` @ `ab83b9b` | 140 ảnh `apps/android/screenshots/` (67 mới, 72 đổi); ghép sáng/tối/chữ lớn, ảnh dài 6400 px cắt thành đoạn; 5 ảnh `-xlarge` (cỡ 2,0) |
| Windows | #25 | `v0.2-w3-windows` @ `5f4dfcc` (ảnh CI run 239, `8b86892`) | 207 ảnh (91 mới, 116 đổi): mọi ảnh `nt-*` 1× sáng/tối, 13 ảnh `a3-*` (A++), `08-chu-lon` |
| Apple | #23 | — | **Không duyệt ở vòng này** (xem §5) |

Căn cứ: `docs/v0.2/08` trên `main` (`8835cd7`, gồm §12 dòng YC-2026-07, TKN-A-08) và các mục sai khác trên nhánh PR (PWA §12i–§12k,
Android §13, Windows §14); `docs/v0.2/01` §3.1 (`danh_xung`, `gioi_tinh`, `quan_he`), §3.2a, §3.8a, §7.1; `docs/v0.2/04` Bổ sung B4, B5/R14;
`docs/v0.2/07` YC-2026-07, YC-2026-08, R-PRIV-2, R-PDF-1; `docs/08` §5–6. Mức: **chặn** · **nên sửa** · **góp ý**.

## 1. Đối chiếu các mục chặn và nên sửa của vòng trước

### 1.1 Android (PR #24)

| Mã | Mức cũ | Trạng thái | Bằng chứng |
|---|---|---|---|
| TKN-A-01 | chặn | **Đã sửa** | `and-02…05-xlarge`, `-large`: nhóm 4 ô thành lưới 2 × 2, bộ lọc ba hàng, "Người đang sống" đủ chữ |
| TKN-A-02 | chặn | **Đã sửa** | Có ảnh `and-25…42`: âm + nhuận + "Dương lịch: Thứ Sáu, 08/08/2025", lỗi + khung tóm tắt, sinh nhật âm, thông báo tắt, nhắc sao lưu, khôi phục, Thay toàn bộ, khói đang chạy + "Dừng khói", thanh đáy, thông báo, chỉ đọc; biểu mẫu chụp trọn |
| TKN-A-03 | nên sửa | Đã sửa | `and-08-large`, `and-09-large`: tên không bị "…", loại + khoảng cách xuống dòng dưới |
| TKN-A-04 | nên sửa | Đã sửa | Viền nút viền, ô chọn, thẻ mục thấy rõ (`and-01`, `and-05`) |
| TKN-A-05 | nên sửa | Đã sửa | `and-07`: công tắc tắt có viền và núm rõ |
| TKN-A-06 | nên sửa | Đã sửa | `and-20-dark`: nền đỏ sẫm, chữ trắng |
| TKN-A-07 | nên sửa | Đã sửa | Mã: `Fine()` dùng `ink2` (5,96:1 trên `wall`) |
| TKN-A-08 | nên sửa | Đã sửa | "Tốt nghiệp đại học · Lê Văn Cường" ở danh sách và xem trước .ics (`and-02`, `and-21`) |
| TKN-A-09 | nên sửa | Đã sửa | Hộp thoại to theo cỡ chữ (`and-20-large`, `and-21-large`, `and-10c-large`) |

### 1.2 Windows (PR #25)

| Mã | Mức cũ | Trạng thái | Bằng chứng |
|---|---|---|---|
| TKN-W-01 | chặn | **Đã sửa** | `08-chu-lon`, mọi `a3-*`: ở A++ thanh mục xuống hàng thứ hai, nút A++ và nút sáng/tối nằm trong cửa sổ |
| TKN-W-02 | chặn | **Đã sửa** | 13 ảnh `a3-*` ở A++ (Sắp tới, Theo kỳ, Hồ sơ, biểu mẫu, lỗi, hộp .ics, hộp sao lưu, tưởng niệm) — không tràn, không cắt chữ |
| TKN-W-03 | nên sửa | Đã sửa | `nt-07`: "Giỗ 15/6 âm lịch" (bỏ "nhuận") |
| TKN-W-04 | nên sửa | Đã sửa | `nt-21`: "Ngày giỗ sắp tới: 15/6 âm lịch, 18/07/2027" |
| TKN-W-05 | nên sửa | Đã sửa | `nt-12`, `a3-nt-12`: khung tóm tắt liệt kê từng lỗi thành liên kết, ô lỗi viền đỏ |
| TKN-W-06 | nên sửa | Đã sửa | Mã: không còn chữ 12–13 px trong `people.slint` (phần lớn 15 px) |
| TKN-W-07 | nên sửa | Đã sửa | Viền ô nhập và nút viền thấy rõ (`nt-09b`, `nt-13`) |
| TKN-W-08 | nên sửa | Chưa đủ bằng chứng | Không đo lại được từ ảnh; "Đang xem kỳ có hôm nay" trông đậm hơn. Không giữ ở mức nên sửa |
| TKN-W-09 | nên sửa | Đã sửa | 08 §14 trên nhánh, 18 mục |

Góp ý cũ đã làm, thấy trên ảnh: TKN-W-10 (hộp kiểm đỏ), TKN-W-13 (khung mặc định thu gọn), TKN-W-15 (ảnh khói, thông báo, khôi
phục, tự nhập tuổi), TKN-W-17 (khung ảnh bầu dục), TKN-X-01, X-03, X-04, X-05, X-06, X-07 (Windows).

### 1.3 Nhất quán chung (vòng trước)

| Mã | Trạng thái |
|---|---|
| TKN-X-02 năm trên bia | **Chưa thống nhất** — xem TKN3-A-02 |
| TKN-X-04 câu "Không gồm thông tin báo tử" | Đã có ở PWA, Android, Windows |
| TKN-X-05 một cặp chữ chọn lịch | Android, Windows đạt; PWA còn "Dương lịch / Âm lịch" ở ô ngày sinh (TKN3-X-02) |
| TKN-X-06 "Ngày giỗ sắp tới" | Đạt ở PWA (§12i), Android, Windows |

## 2. PWA vòng 2 (PR #30)

Nội dung mới đạt yêu cầu:
- **Cách gọi viết hoa, gợi ý theo người sống/mất**: gợi ý ô trống "ví dụ: Anh, Chị, Cô, Chú" (người đang sống), giá trị lưu
  "Ông", quan hệ "Ông nội (bên nội)" (`form-deceased-360-light`, `form-living-360-light`).
- **Giới tính** Nam / Nữ / Không ghi ở cả hai biểu mẫu; hồ sơ có dòng "Giới tính: Không ghi" (`person-notice-360`).
- **Tệp đính kèm báo tử**: trên trang hồ sơ mỗi tệp có loại, tên gốc, dung lượng; PDF có "Tệp PDF được lưu nguyên bản"; nút "Tải ảnh…",
  "Tải tệp PDF…". Hộp "Tải tệp xuống máy?" nói rõ tệp nằm ngoài ứng dụng, không mật khẩu, không tự xóa; PDF có câu về thông tin máy quét;
  "Không tải" là nút an toàn (`attachment-download-dialog-360`). Hộp tạo sao lưu ghi số tệp PDF lưu nguyên bản (`backup-notice-dialog-360`).
- **R-PRIV-2**: chuỗi canh gác trong tên tệp và nội dung chỉ thấy trên trang hồ sơ; hộp .ics, xem trước, tưởng niệm không có.
- **B4**: hồ sơ người đã mất có câu "Không nhắc sinh nhật và sự kiện của người đã mất — chỉ nhắc ngày giỗ…"; hộp "Cập nhật … là người đã mất?"
  liệt kê thay đổi (`convert-confirm-360`).
- **B5/R14 chỉ đọc** (`upgrade-readonly-360`): khung đầu trang "Chưa chuyển được dữ liệu sang bản mới. Dữ liệu cũ vẫn còn nguyên…",
  hướng dẫn từng bước và nút "Đi tới Sao lưu" — lời thường, không mã kỹ thuật. Đây là cách viết tốt nhất trong ba bản.
- Dấu tiếng Việt và cỡ A++ đạt (`form-deceased-360-xl`).

### 2.1 Nên sửa

#### TKN3-P-01 — Chế độ chỉ đọc vẫn hiện nút "+ Thêm" — nên sửa

- Ảnh: `v02-upgrade-readonly-360.png` — khung báo nói "chưa thêm hay sửa được", nhưng danh sách Hồ sơ vẫn có hai nút "+ Thêm"
  (và nhắc sao lưu, cảnh báo lưu trữ phía dưới). Bấm vào sẽ mở biểu mẫu rồi mới báo lỗi `read_only` khi lưu: người dùng gõ xong mới biết
  là không lưu được, đúng lúc họ đang lo mất dữ liệu.
- Sửa: khi `migrationStatus().state === "read_only"`, ẩn mọi nút thêm/sửa/xóa, "Cập nhật: đã mất", khôi phục (như Windows §14 mục 18,
  Android `and-41`); thêm kiểm thử đếm 0 nút ghi ở chế độ chỉ đọc.

### 2.2 Góp ý

| ID | Ảnh | Điều thấy | Đề xuất |
|---|---|---|---|
| TKN3-P-02 | `form-*-360-light` | "Giới tính" là ba nút tròn xếp dọc trong khung `fieldset` có viền mặc định của trình duyệt — chiếm ~1/3 màn hình, kiểu khác hẳn các nhóm chọn khác của biểu mẫu | Dùng nhóm nút viên thuốc một hàng như "Dương lịch / Âm lịch" (giữ `radio` cho trợ năng), bỏ viền `fieldset` |
| TKN3-P-03 | `ics-dialog-360` | Câu riêng tư "Tệp này chứa tên…" in đỏ đậm như lỗi | Dùng thẻ lưu ý vàng như hộp sao lưu (TK-14) |

**Kết luận PWA: DUYỆT** — không có mục chặn; TKN3-P-01 sửa trong đợt này hoặc mở việc có hạn trước phát hành.

## 3. Android (PR #24)

Nội dung mới đạt yêu cầu:
- Giới tính ba nút một hàng, chữ lớn xếp dọc (`and-13`, `and-16`); gợi ý "Gợi ý ▾" lấy từ lõi `danhXungSuggestions(deceased, gioiTinh)` (mã).
- Tệp đính kèm: chọn loại trước (Tin báo tử / Giấy chứng tử / Tệp khác), "Thêm ảnh" / "Thêm tệp PDF", câu "Tệp PDF được lưu nguyên bản (có
  thể còn tên máy quét…)"; "Bỏ tệp" (`and-19b`, `and-33`). Hồ sơ hiện ảnh giấy và PDF; "Mở bằng ứng dụng khác…" có hộp xác nhận nói rõ
  ứng dụng kia đọc được, chỉ cấp quyền một lần, tên chung `TepDinhKem.pdf`, xóa bản tạm (`and-10c`) — đúng tinh thần BM2-05.
- Chế độ chỉ đọc ẩn nút thêm/sửa, có nút "Tạo tệp sao lưu…" (`and-41`, `and-42`).
- Mọi mục chặn và nên sửa cũ đã sửa (§1.1).

### 3.1 Nên sửa

#### TKN3-A-01 — Khung chỉ đọc hiện mã lỗi kỹ thuật — nên sửa

- Ảnh: `and-41-chi-doc-*`, `and-42-*`, `and-41-chi-doc-xlarge`: "Chưa nâng cấp được dữ liệu (dữ liệu không hợp lệ: V1 /person/death/
  notice/place: chưa cắt khoảng trắng hai đầu)…". Người 35–70 tuổi không hiểu "V1 /person/death/notice/place"; câu làm họ nghĩ dữ liệu
  hỏng. Ở chữ lớn khung dài hơn một màn hình.
- Sửa: câu chính như PWA ("Chưa chuyển được dữ liệu sang bản mới. Dữ liệu cũ vẫn còn nguyên…") + hướng dẫn; chi tiết kỹ thuật đưa vào
  mục thu gọn "Chi tiết để gửi Clevix Lab" hoặc chỉ ghi vào tệp nhật ký.

#### TKN3-A-02 — Năm trên bia vẫn có ngoặc, ngược với Windows và PWA; hai PR sửa §3.9 theo hai hướng — nên sửa

- Ảnh: `and-11`, `and-38`: "Canh Thìn (1940) — Ất Tỵ (2025)", "Mậu Dần (1938) — Kỷ Hợi (2019)". Windows `nt-21` và PWA: "Canh Thìn
  1940 — Ất Tỵ 2025". 08 §13 (Android) ghi dạng có ngoặc là "đúng §3.9", còn PR #25 sửa chính §3.9 thành dạng **không** ngoặc "thống nhất
  PWA, Android, Windows". Gộp cả hai PR sẽ để lại đặc tả tự mâu thuẫn. Ngoặc còn làm dòng năm xuống hai dòng trên bia ở 360 dp.
- Sửa: Android theo dạng không ngoặc; bỏ dòng "Năm trên bia" khỏi bảng §13. Điều phối xác nhận một dạng trước khi gộp PR thứ hai.

### 3.2 Góp ý

| ID | Ảnh | Điều thấy | Đề xuất |
|---|---|---|---|
| TKN3-A-03 | `and-10b` | Tên loại "Tin báo tử", PWA/Windows "Ảnh tin báo tử" | Một bộ tên loại chung (TKN3-X-01) |

**Kết luận Android: DUYỆT** — hai mục chặn cũ đã sửa, không có chặn mới; TKN3-A-01, TKN3-A-02 sửa trước khi gộp hoặc mở việc có hạn.

## 4. Windows (PR #25)

Nội dung mới đạt yêu cầu:
- A++ đạt (§1.2). Giới tính đứng đầu nhóm, gợi ý cách gọi là hàng nút; người đang sống **không** có Cụ, Kỵ, Ông Tổ họ, Bà Cô Tổ họ, Bé đỏ
  (`nt-13`), người đã mất có đủ (`a3-nt-12`). Ô "bà nội" gõ thường, lõi viết hoa khi lưu (§14 mục 2).
- Tệp đính kèm: mỗi tệp một dòng có ảnh thu nhỏ/nhãn PDF, tên gốc, ba nút loại, "Bỏ tệp", "Mở bằng ứng dụng xem PDF", câu PDF nguyên
  bản (`nt-09b`); trang hồ sơ liệt kê tệp trong khối "Thông tin báo tử" (`nt-09`).
- Chế độ chỉ đọc ẩn nút ghi (`nt-24`, `nt-24b`).
- Trạng thái thông báo thật (`nt-02b…02e`), thẻ nhắc Tờ lịch (`nt-02f`), hộp khôi phục và Thay toàn bộ (`nt-20b`, `nt-20c`), khói đang chạy (`nt-21b`).

### 4.1 Nên sửa

#### TKN3-W-01 — Khung chỉ đọc: chữ đỏ nhỏ và có "Chi tiết kỹ thuật: SQLite: table meta already exists" — nên sửa

- Ảnh: `nt-23-chuyen-doi-loi-*`, `nt-24-che-do-chi-doc-*`. Cả khung là chữ đỏ cỡ nhỏ trên nền hồng, bốn dòng dày đặc, cuối câu có
  chuỗi tiếng Anh kỹ thuật (mã: `people.rs` dòng ~352). Như TKN3-A-01.
- Sửa: chữ thường màu `ink` cỡ nội dung, câu chính như PWA; chi tiết kỹ thuật thu gọn hoặc chỉ ghi nhật ký; giữ hai nút "Tạo tệp sao lưu…",
  "Mở thư mục dữ liệu".

### 4.2 Góp ý

| ID | Ảnh | Điều thấy | Đề xuất |
|---|---|---|---|
| TKN3-W-02 | `nt-09b` | Tên loại "Khác" (Android "Tệp khác"); PDF ở trạng thái thường có loại "Khác" dù tên là bản quét giấy chứng tử | Tên loại chung (TKN3-X-01) |

**Kết luận Windows: DUYỆT** — hai mục chặn cũ đã sửa, không có chặn mới; TKN3-W-01 sửa trước khi gộp hoặc mở việc có hạn.

## 5. Apple (PR #23) — không duyệt ở vòng này

Điều kiện của điều phối: chỉ duyệt khi CI xanh trên đầu nhánh mới nhất. Lúc duyệt (2026-11-07, 15:05 giờ Hà Nội):
- đầu nhánh `85850c4` là commit ảnh chụp; các lần chạy trên nó ở trạng thái `action_required` (chưa chạy);
- commit mã mới nhất `e3e1ddf`: `ci`, `android`, `windows` xanh, nhưng workflow **`apple` vẫn đang chạy** (job `build` `in_progress`).

Vì vậy Apple **chưa** được duyệt vòng cuối; các mục TKN-I-01 … TKN-I-09 của PR #26 vẫn giữ nguyên trạng thái cũ. Điều phối gửi riêng
khi CI Apple xanh.

## 6. Nhất quán giữa các nền tảng — góp ý

| ID | Điều thấy | Đề xuất |
|---|---|---|
| TKN3-X-01 | Tên loại tệp đính kèm khác nhau: PWA "Ảnh tin báo tử / Giấy chứng tử (ảnh hoặc bản quét PDF) / Khác"; Android "Tin báo tử / Giấy chứng tử / Tệp khác"; Windows "Ảnh tin báo tử / Giấy chứng tử / Khác" | Ghi một bộ tên vào 08 (đề xuất: "Tin báo tử", "Giấy chứng tử", "Tệp khác" — tin báo tử có thể là PDF) và dùng ở cả ba |
| TKN3-X-02 | Ô ngày sinh của PWA còn "Dương lịch / Âm lịch"; Android, Windows dùng "Nhập theo dương lịch / Nhập theo âm lịch" ở mọi ô (TKN-X-05) | PWA dùng cùng cặp chữ |
| TKN3-X-03 | Khung chỉ đọc: PWA lời thường, Android và Windows có mã kỹ thuật (TKN3-A-01, TKN3-W-01) | Ghi câu chuẩn vào 08 §12j cho mọi nền tảng |
| TKN3-X-04 | Nút mở tệp đính kèm: PWA "Tải tệp PDF…", Android "Mở bằng ứng dụng khác…", Windows "Mở bằng ứng dụng xem PDF" | Chấp nhận theo nền tảng; mọi bản đều có hộp xác nhận — đạt BM2-05 |

## 7. Kết luận

| Nền tảng | Mục chặn cũ | Chặn mới | Nên sửa mới | Góp ý mới | Kết luận |
|---|---|---|---|---|---|
| PWA vòng 2 (#30) | — | 0 | 1 | 2 | **DUYỆT** |
| Android (#24) | 2/2 đã sửa | 0 | 2 | 1 | **DUYỆT** |
| Windows (#25) | 2/2 đã sửa | 0 | 1 | 1 | **DUYỆT** |
| Chung | — | 0 | 0 | 4 | — |
| Apple (#23) | — | — | — | — | **Không duyệt vòng này** — CI Apple chưa xanh (§5) |

Tổng mới: **0 chặn, 4 nên sửa, 8 góp ý.** Không còn mục chặn nào của bên duyệt thiết kế trên PWA, Android, Windows.

Các mục nên sửa (sửa trước khi gộp hoặc mở việc có hạn trước phát hành v0.2): TKN3-P-01 (ẩn nút ghi ở chế độ chỉ đọc), TKN3-A-01 và
TKN3-W-01 (bỏ mã kỹ thuật khỏi khung chỉ đọc), TKN3-A-02 (một dạng năm trên bia; điều phối chốt §3.9 trước khi gộp PR thứ hai).
