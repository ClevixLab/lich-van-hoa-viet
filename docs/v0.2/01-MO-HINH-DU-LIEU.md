# Mô hình dữ liệu — Lịch và Văn hóa Việt v0.2

Trạng thái: **KHÓA (G1) — 2026-11-06, chờ chủ dự án ký.** Sau khi ký, mọi thay đổi trường, kiểu hoặc quy tắc
kiểm tra phải tăng `schema_version` và có hàm chuyển đổi (§7), ghi lý do trong tài liệu này.

Phạm vi v0.2: dữ liệu chỉ nằm trên thiết bị, không tài khoản, không máy chủ. Tài khoản, đồng bộ và chia sẻ cho
người khác thuộc v0.3. Thuật toán dùng các trường dưới đây được đặc tả ở `02-THUAT-TOAN-GIO-NHAC.md`; tệp sao lưu
ở `03-SAO-LUU.md`; lược đồ JSON dùng chung ở `schema/backup-v1.schema.json`.

## 1. Quy ước chung

| Quy ước | Nội dung |
|---|---|
| Mã định danh (ID) | UUID phiên bản 4, chữ thường, có gạch: `^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$`. Sinh bằng bộ sinh số ngẫu nhiên an toàn của nền tảng (`crypto.randomUUID()`, `java.util.UUID.randomUUID()`, `UUID()` của Swift, crate `uuid` với `getrandom`). Không dùng UUID v1/v7 (lộ thời điểm tạo hoặc địa chỉ máy). ID không bao giờ đổi sau khi tạo. |
| Chuỗi | UTF-8, **chuẩn hóa Unicode NFC** trước khi lưu (bàn phím tiếng Việt trên iOS/Android/Windows có thể sinh NFD hoặc tổ hợp dấu khác nhau). Cắt khoảng trắng hai đầu. Độ dài tính theo **điểm mã** (code point) sau NFC. Không chứa ký tự điều khiển U+0000–U+001F, U+007F–U+009F, trừ `\n` ở các trường ghi chú. |
| Thời điểm hệ thống | `created_at`, `updated_at`, `exported_at`: chuỗi RFC 3339 theo UTC, độ chính xác giây, đuôi `Z` (ví dụ `2026-11-06T03:15:00Z`). |
| Bản sửa | `rev`: số nguyên ≥ 1, tăng 1 mỗi lần lưu thay đổi. Dùng cho `SEQUENCE` của tệp .ics và cho gộp dữ liệu khi nhập sao lưu. |
| Ngày dương | Lịch Gregory đón trước, trong miền hỗ trợ của lõi 1900-01-01 → 3000-12-31 khi cần đổi sang âm. |
| Ngày âm | Lịch âm Việt Nam theo lõi `lvhv-core`, phương pháp mặc định `Precise` (docs/02 Bổ sung A3), quy ước múi giờ lịch sử (docs/03 §16). |
| Trường dẫn xuất | Đánh dấu **(dẫn xuất)**: do ứng dụng tính khi lưu, được lưu kèm để hiển thị nhanh và để không đổi lặng lẽ nếu thuật toán đổi về sau. Khi nhập sao lưu, trường dẫn xuất **được kiểm tra lại** (§6.3), không tin mù quáng. |

## 2. Kiểu dữ liệu dùng chung

### 2.1 `DateValue` — một ngày người dùng nhập

| Trường | Kiểu | Bắt buộc | Kiểm tra |
|---|---|---|---|
| `calendar` | `"solar"` \| `"lunar"` | có | — |
| `precision` | `"day"` \| `"month_day"` \| `"year"` | có | `day` = đủ ngày/tháng/năm; `month_day` = biết ngày và tháng, **không biết năm**; `year` = chỉ biết năm |
| `year` | số nguyên \| `null` | theo `precision` | `day`, `year`: bắt buộc; `month_day`: phải `null`. Khoảng cho phép: 1000–3000 (ghi nhận). Chỉ **đổi lịch** được khi năm nằm trong miền của lõi (dương 1900–3000; âm 1899–3000); ngoài miền vẫn lưu được nhưng không đổi lịch (§2.2) |
| `month` | 1–12 \| `null` | `day`, `month_day`: bắt buộc; `year`: `null` | — |
| `day` | 1–31 (dương) / 1–30 (âm) \| `null` | `day`, `month_day`: bắt buộc; `year`: `null` | Dương có năm: ngày phải tồn tại (kể cả 29/02 năm nhuận). Âm có năm trong miền: tháng (và cờ nhuận) phải tồn tại và `day` ≤ độ dài tháng đó (`lunar_to_solar` ≠ `None`). Âm không có năm: `day` ≤ 30 |
| `leap` | boolean | có (dương: luôn `false`) | Chỉ `true` khi `calendar = "lunar"`. Có năm trong miền: tháng nhuận phải tồn tại trong năm đó |

Ràng buộc: `calendar = "solar"` với `precision = "month_day"` **không được phép** ở mọi nơi trong v0.2. Với ngày
mất, không đổi được ngày dương sang âm khi không biết năm (§3.2). Với sinh nhật, phải biết năm để tính tuổi. Giá trị
`month_day` chỉ dùng cho **ngày âm**: ngày mất không rõ năm (§3.2) và sự kiện âm lặp hằng năm (§3.4). Bảng cho phép
từng trường nằm ở §3.2–§3.4.

### 2.2 `LunarDay` — ngày âm chuẩn hóa (dẫn xuất)

```
{ "year": số nguyên | null, "month": 1–12, "day": 1–30, "leap": boolean }
```
`year = null` khi không biết năm. Đây là dạng mà thuật toán giỗ (02 §2) đọc.

### 2.3 `SolarDay` — ngày dương (dẫn xuất)

```
{ "y": số nguyên, "m": 1–12, "d": 1–31 }
```

## 3. Thực thể

Sơ đồ quan hệ:

```
Person 1 ──── 0..1 DeathInfo        (bắt buộc khi kind = "deceased", cấm khi "living")
Person 1 ──── 0..1 BirthInfo
Person 1 ──── 0..1 Photo            (qua photo_id)
Person 1 ──── 0..1 MemorialSettings (chỉ "deceased")
Person 1 ──── 1..3 ReminderRule     (giỗ | sinh nhật dương | sinh nhật âm; nhúng trong Person)
Event   n ──── m   Person           (Event.person_ids, 0..50 người)
Event   n ──── 1   EventType
Event   1 ──── 1   ReminderRule     (nhúng)
```

### 3.1 `Person` — hồ sơ một người

| Trường | Kiểu | Bắt buộc | Kiểm tra / ghi chú |
|---|---|---|---|
| `id` | UUID v4 | có | §1 |
| `kind` | `"deceased"` \| `"living"` | có | Đổi `living` → `deceased` được (thêm `death`); đổi ngược lại phải xóa `death` và `memorial`, có hộp xác nhận |
| `danh_xung` | chuỗi 0–40 | không | Cách gọi đứng trước tên. **Viết hoa chữ cái đầu** (YC-2026-07): lớp lưu trữ chuẩn hóa chữ cái đầu tiên thành chữ hoa khi lưu (NFC, phần còn lại giữ nguyên như người dùng gõ). Gợi ý cho **người đã mất**: Ông, Bà, Cụ, Kỵ, Bác, Cô, Chú, Dì, Cậu, Mợ, Thím, Anh, Chị, Em, Cháu, Con, Bạn, Bố, Mẹ, Ông Tổ họ, Bà Cô Tổ họ, Bé đỏ. Gợi ý cho **người đang sống**: như trên nhưng **không có** Cụ, Kỵ, Ông Tổ họ, Bà Cô Tổ họ, Bé đỏ. Người dùng gõ tự do được; không kiểm tra theo danh sách |
| `gioi_tinh` | `"nam"` \| `"nu"` \| `null` | không | Giới tính (YC-2026-07). Hiển thị "Nam" / "Nữ" / "Không ghi". Dùng để gợi ý cách gọi phù hợp (ví dụ Ông/Bà); **không** ảnh hưởng ngày giỗ, tuổi, lời nhắc. Thiếu khóa = `null` (tệp cũ vẫn hợp lệ) |
| `ho_ten` | chuỗi 1–120 | **có** | Không rỗng sau khi cắt khoảng trắng |
| `ten_hieu` | chuỗi 0–120 | không | Tên hiệu, pháp danh, tên thánh… (nhãn hiển thị chung "Tên hiệu / pháp danh") |
| `quan_he` | chuỗi 0–120 | không | Quan hệ gia đình, tự do (ví dụ "Ông nội", "Bà ngoại bên mẹ"). Chữ cái đầu viết hoa khi lưu, như `danh_xung` |
| `birth` | `BirthInfo` \| `null` | không | §3.3 |
| `death` | `DeathInfo` \| `null` | khi `deceased` | §3.2 |
| `noi_an_tang` | chuỗi 0–300 | không | Chỉ `deceased` |
| `photo_id` | UUID \| `null` | không | Trỏ tới `Photo` tồn tại; xóa ảnh thì đặt `null` |
| `memorial` | `MemorialSettings` \| `null` | khi `deceased` | §3.7; tạo mặc định khi tạo hồ sơ người đã khuất |
| `reminders` | đối tượng | có | `{ "gio": ReminderRule\|null, "sinh_nhat": ReminderRule\|null, "sinh_nhat_am": ReminderRule\|null }`. `deceased`: chỉ `gio`; `living`: `sinh_nhat`, `sinh_nhat_am` |
| `sinh_nhat_am` | boolean | có | Chỉ `living`; mặc định `false`. Bật thì có thêm sinh nhật âm hằng năm |
| `feb29_rule` | `"feb28"` \| `"mar1"` | có | Mặc định `"feb28"`; chỉ có tác dụng khi sinh ngày 29/02 dương (02 §6) |
| `ghi_chu` | chuỗi 0–2000 | không | Cho phép `\n` |
| `created_at`, `updated_at` | RFC 3339 UTC | có | §1 |
| `rev` | số nguyên ≥ 1 | có | §1 |

### 3.2 `DeathInfo` — ngày mất

| Trường | Kiểu | Bắt buộc | Kiểm tra / ghi chú |
|---|---|---|---|
| `input` | `DateValue` | có | Đúng như người dùng nhập. Cho phép: dương `day` (năm 1900–3000); âm `day`; âm `month_day` (không biết năm). **Không** cho phép `precision = "year"` (không xác định được ngày giỗ) |
| `lunar` | `LunarDay` | có **(dẫn xuất)** | Nếu `input` là dương: đổi sang âm **khi lưu** bằng `solar_to_lunar` (phương pháp mặc định). Nếu `input` là âm: chép nguyên (`year` = `null` khi `month_day`) |
| `solar` | `SolarDay` \| `null` | **(dẫn xuất)** | Dương: chép nguyên. Âm có năm trong miền: `lunar_to_solar`. Âm không năm hoặc năm ngoài miền: `null` |
| `lunar_source` | `"entered"` \| `"converted"` | **(dẫn xuất)** | — |
| `method` | `"precise"` \| `"hnd"` | **(dẫn xuất)** | Phương pháp đã dùng khi đổi. Không tự đổi lại khi phương pháp mặc định thay đổi ở bản sau; ứng dụng chỉ báo "ngày âm có thể khác nếu tính lại" khi hai phương pháp cho kết quả khác |
| `tuoi` | đối tượng | có | `{ "value": số nguyên 1–150 \| null, "label": "huong_tho" \| "huong_duong", "manual": boolean, "approx": boolean }`. Tính theo 02 §4; `manual = true` khi người dùng sửa — khi đó không tính lại tự động |

Quy tắc: nếu cả ngày sinh và ngày mất có năm âm xác định thì năm âm mất ≥ năm âm sinh; nếu vi phạm → lỗi "Ngày mất
trước ngày sinh". Ngày mất ở tương lai (so với ngày hôm nay của thiết bị) → lỗi.

### 3.2a `DeathNotice` — thông tin báo tử (tùy chọn; Bổ sung YC-2026-05, 2026-11-07)

Nhúng trong `DeathInfo` dưới khóa `notice` (`null` hoặc đối tượng). Mọi trường đều tùy chọn; chuỗi đã cắt khoảng trắng
hai đầu, rỗng ghi là `null`.

| Trường | Kiểu | Kiểm tra / ghi chú |
|---|---|---|
| `reported_by` | chuỗi ≤ 200 ký tự \| `null` | Ai báo tin |
| `place` | chuỗi ≤ 200 \| `null` | Mất ở đâu (khác nơi an táng ở `Person`) |
| `reason` | chuỗi ≤ 500 \| `null` | Lý do. **Nhạy cảm**: chỉ hiện trên trang hồ sơ |
| `notice_date` | `DateValue` \| `null` | Ngày nhận tin (dương), không được sau hôm nay |
| `attachments` | mảng 0–6 phần tử `{ "id": UUID, "kind": "tin_bao_tu" \| "giay_chung_tu" \| "khac", "file_id": UUID }` | Tệp đính kèm (YC-2026-07, **thay** `certificate_photo_ids`): ảnh tin báo tử, bản chụp/quét giấy chứng tử, hoặc tệp khác. `file_id` trỏ tới một `Attachment` (§3.8a). Thuộc về hồ sơ này; xóa hồ sơ hoặc gỡ khỏi danh sách thì xóa tệp |
| `note` | chuỗi ≤ 2000 \| `null` | Ghi chú |

Quy tắc: `notice` chỉ có khi có `DeathInfo`. Không trường nào của `notice` được đưa vào thông báo, tệp .ics, màn hình
tưởng niệm, tên tệp xuất hay URL. `notice` có trong tệp sao lưu mã hóa (03) và đi qua kiểm tra khi nhập (§6.3).
Chuyển một hồ sơ đang sống thành đã mất = thêm `death` (và tùy chọn `notice`) vào **cùng** hồ sơ, tăng `rev`; không
tạo hồ sơ mới.

### 3.3 `BirthInfo` — ngày sinh

| Trường | Kiểu | Bắt buộc | Kiểm tra / ghi chú |
|---|---|---|---|
| `input` | `DateValue` | có | Dương `day` hoặc `year`; âm `day` hoặc `year`. Không cho phép `month_day` |
| `noi_sinh` | chuỗi 0–300 | không | — |
| `lunar_year` | số nguyên \| `null` | **(dẫn xuất)** | Năm âm của ngày sinh (02 §4.1). `null` nếu không xác định |
| `lunar_year_approx` | boolean | **(dẫn xuất)** | `true` khi chỉ biết năm dương (có thể là năm âm trước nếu sinh trước Tết) |
| `can_chi` | chuỗi \| `null` | **(dẫn xuất)** | Can Chi của `lunar_year` (hợp đồng kiến trúc §3), ví dụ "Kỷ Tỵ" |
| `lunar` | `LunarDay` \| `null` | **(dẫn xuất)** | Ngày âm của ngày sinh khi biết đủ ngày (dùng cho sinh nhật âm) |
| `solar` | `SolarDay` \| `null` | **(dẫn xuất)** | Ngày dương khi biết đủ ngày |

### 3.4 `Event` — sự kiện gắn với người

| Trường | Kiểu | Bắt buộc | Kiểm tra / ghi chú |
|---|---|---|---|
| `id` | UUID v4 | có | — |
| `type_id` | UUID v4 | có | Trỏ tới `EventType` tồn tại |
| `tieu_de` | chuỗi 0–120 | không | Rỗng → hiển thị tên loại |
| `date` | `DateValue` | có | `precision = "day"` (dương hoặc âm). Sự kiện lặp âm không biết năm: `month_day` âm được phép |
| `recurring` | boolean | có | Mặc định lấy theo loại (`ngay_cuoi`: `true`) |
| `person_ids` | mảng UUID, 0–50, không trùng | có | Mỗi phần tử trỏ tới `Person` tồn tại. Xóa người → gỡ id khỏi mọi sự kiện (sự kiện còn lại) |
| `ghi_chu` | chuỗi 0–2000 | không | — |
| `reminder` | `ReminderRule` \| `null` | có | `null` = không nhắc. Không lặp và đã qua → không sinh nhắc |
| `created_at`, `updated_at`, `rev` | | có | §1 |

### 3.5 `EventType` — loại sự kiện

| Trường | Kiểu | Bắt buộc | Kiểm tra / ghi chú |
|---|---|---|---|
| `id` | UUID v4 | có | Loại có sẵn dùng **UUID cố định** (bảng dưới) để sao lưu từ hai máy gộp được |
| `builtin` | `"tot_nghiep"` \| `"ngay_cuoi"` \| `"dinh_hon"` \| `"du_hoc"` \| `null` | có | `null` = loại người dùng tự tạo |
| `ten` | chuỗi 1–60 | có | Duy nhất (so sánh sau NFC + chữ thường theo Unicode) |
| `default_recurring` | boolean | có | — |
| `created_at`, `updated_at`, `rev` | | có | — |

Loại có sẵn (không xóa được, đổi tên được):

| `builtin` | `id` cố định | Tên mặc định | `default_recurring` |
|---|---|---|---|
| `tot_nghiep` | `6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0001` | Tốt nghiệp | `false` |
| `ngay_cuoi` | `6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0002` | Ngày cưới | `true` |
| `dinh_hon` | `6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0003` | Đính hôn | `false` |
| `du_hoc` | `6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0004` | Du học | `false` |

(Các id này đúng mẫu UUID v4 và được sinh một lần, ghi cố định ở đây.)

### 3.6 `ReminderRule` — quy tắc nhắc (nhúng)

| Trường | Kiểu | Bắt buộc | Kiểm tra / ghi chú |
|---|---|---|---|
| `id` | UUID v4 | có | Ổn định; dùng làm tiền tố định danh thông báo (02 §7) |
| `enabled` | boolean | có | Mặc định `true` |
| `offsets_days` | mảng số nguyên | có | Tăng dần, không trùng, mỗi phần tử 0–60, 0–8 phần tử. `0` = đúng ngày. Giao diện có các nút chọn nhanh 0 (đúng ngày), 1, 3, 5, 10, 15 và một ô "tự đặt" 1–60. **Mặc định `[0, 3]`** |
| `time_local` | chuỗi `"HH:MM"` 00:00–23:59 | có | Giờ đồng hồ **của thiết bị**. Mặc định `"07:00"` |
| `tien_thuong` | boolean | có | Chỉ có nghĩa với `gio`; luôn `false` ở loại khác. **Mặc định `false`** (mục mở O-1, `05-KE-HOACH-TRIEN-KHAI.md`) |

### 3.7 `MemorialSettings` — màn hình tưởng niệm

| Trường | Kiểu | Bắt buộc | Kiểm tra / ghi chú |
|---|---|---|---|
| `tray_style` | `"phat_giao"` \| `"truyen_thong"` \| `"cong_giao"` \| `"toi_gian"` | có | Mặc định `"truyen_thong"`. `phat_giao`: hương, hoa, quả, mâm chay; `truyen_thong`: mâm cơm cúng mặn — **nội dung do chủ dự án quyết (YC-2026-09)**: bát cơm có quả trứng luộc, khoanh giò, gà luộc, đĩa xôi, đĩa hoa quả, bát hương nhiều chân hương với ba nén đang cháy, lọ hoa ly hoặc cúc vàng; ghép từ **ảnh chụp thật** (09 §3); `cong_giao`: nến và hoa, **không có hương**; `toi_gian` (Bổ sung YC-2026-09): kệ nhỏ, một nến, bát hương, chén nước, một bình hoa (09 §5), có hương theo `incense` |
| `incense` | boolean | có | Ba nén hương. Bắt buộc `false` khi `cong_giao` |
| `bia_tieu_de` | chuỗi 0–60 | không | Dòng đầu bia (ví dụ "Hiển khảo", "Hiển tỷ", "Cố"…), tự do |
| `loi_tuong_niem` | chuỗi 0–2000 | không | Lời của chính người dùng; cho phép `\n` |
| `gio_het_nam` | 2 \| 3 | có | Năm thứ mấy sau năm mất là "giỗ hết". Mặc định 2 (mục mở O-2) |
| `huong_duong_duoi` | số nguyên 1–150 | có | Tuổi âm nhỏ hơn số này ghi "hưởng dương", còn lại "hưởng thọ". Mặc định 60 (mục mở O-3) |

**Bổ sung YC-2026-09 (2026-11-07) — thêm `toi_gian`.** Lý do: chủ dự án quyết nội dung mâm truyền thống; điều phối viên chỉ đạo (07/11) thêm kiểu tối giản theo đề xuất
Q-1 của 09 §1. `cong_giao` giữ nguyên, không thay bằng `toi_gian`.
- Phiên bản lược đồ: v0.2 **chưa phát hành**, nên làm như các bổ sung YC-2026-05 và YC-2026-07 — giữ
  `schema_version = 1`, sửa thẳng `schema/backup-v1.schema.json`. Nếu trước khi gộp đã có bộ mẫu phát hành trong
  `test-vectors/released/` thì phải theo §7: tăng `schema_version` và có hàm chuyển đổi. Điều phối viên xác nhận.
- Ảnh hưởng B5 / R14 (c): bản dựng 0.2 trước thay đổi này không đọc được tệp có `toi_gian` (báo lỗi, không ghi gì
  — §6.3). Dữ liệu trên máy không đổi vì không có phép chuyển đổi nào; chấp nhận vì chưa phát hành.
- Việc của các vai khác (vai nội dung **không** sửa mã): lược đồ JSON (`enum` của `tray_style`), `store/src/validate.rs`,
  `pwa/src/store/validate.ts`, FFI và giao diện từng nền tảng; vectơ kiểm thử thêm một hồ sơ `toi_gian`.
- Ràng buộc `incense = false` khi `cong_giao` giữ nguyên; `toi_gian` cho phép cả hai giá trị.

### 3.8 `Photo` — ảnh chân dung (chỉ trên thiết bị)

| Trường | Kiểu | Bắt buộc | Kiểm tra / ghi chú |
|---|---|---|---|
| `id` | UUID v4 | có | — |
| `mime` | `"image/jpeg"` \| `"image/webp"` | có | Sau khi mã hóa lại |
| `width`, `height` | 1–2048 | có | Cạnh dài ≤ 1024 px sau khi thu nhỏ (mặc định); không phóng to |
| `bytes` | 1–1 048 576 | có | Ảnh đầu vào tối đa 25 MB; ảnh lưu tối đa 1 MB |
| `sha256` | 64 ký tự hex thường | có | Băm của byte ảnh đã lưu; kiểm tra khi nhập sao lưu |
| `stripped` | `true` | có | Ảnh **luôn** được giải mã và mã hóa lại (canvas / `Bitmap` / `CGImage`), nên không còn EXIF, GPS, XMP, IPTC. Giá trị khác `true` → từ chối |
| `created_at` | RFC 3339 UTC | có | — |

Dữ liệu ảnh: PWA lưu `Blob` trong kho `photos` của IndexedDB; native lưu tệp trong thư mục riêng của ứng dụng
(`files/photos/<id>.jpg` trên Android, `Application Support/photos/` trên Apple, `%LOCALAPPDATA%\ClevixLab\LichVanHoaViet\photos\` trên
Windows), bảng SQLite chỉ giữ siêu dữ liệu trên. Ảnh **không** nằm trong thư viện ảnh chung của máy.

### 3.8a `Attachment` — tệp đính kèm thông tin báo tử (YC-2026-07, 2026-11-07)

| Trường | Kiểu | Kiểm tra / ghi chú |
|---|---|---|
| `id` | UUID v4 | — |
| `mime` | `"image/jpeg"` \| `"image/webp"` \| `"application/pdf"` | Ảnh: luôn giải mã và mã hóa lại như `Photo` (§3.8, bỏ siêu dữ liệu, cạnh dài ≤ 2048 px để giấy tờ còn đọc được, ≤ 2 MB). PDF (bản quét): lưu **nguyên byte**, ≤ 10 MB, phải bắt đầu bằng `%PDF-`; giao diện báo rõ "Tệp PDF được lưu nguyên bản" |
| `bytes` | số nguyên | Theo giới hạn trên |
| `sha256` | 64 hex | Kiểm tra khi nhập sao lưu |
| `stripped` | boolean | `true` với ảnh; `false` với PDF |
| `file_name` | chuỗi ≤ 120 \| `null` | Tên gốc để hiển thị; **không** dùng làm tên tệp khi lưu trên máy |
| `created_at` | RFC 3339 UTC | — |

Lưu trên máy như ảnh (§3.8), thư mục `attachments/`. Có trong tệp sao lưu mã hóa; giới hạn tổng tệp sao lưu (03) giữ nguyên.
Không bao giờ đưa vào thông báo, .ics, màn hình tưởng niệm.

### 3.9 `AppSettings` — cài đặt chung (một bản ghi)

| Trường | Kiểu | Mặc định | Ghi chú |
|---|---|---|---|
| `reminders_enabled` | boolean | `true` | Công tắc chung |
| `default_offsets_days` | như §3.6 | `[0, 3]` | Áp cho hồ sơ mới |
| `default_time_local` | `"HH:MM"` | `"07:00"` | Áp cho hồ sơ mới |
| `upcoming_days` | số nguyên | 30 | Danh sách "Sắp tới" (cố định 30 ở v0.2, lưu để mở rộng) |
| `last_backup_at` | RFC 3339 UTC \| `null` | `null` | Dùng cho lời nhắc sao lưu |
| `backup_nag_days` | 0, 30, 60, 90 | 30 | Nhắc sao lưu khi dữ liệu thay đổi và chưa sao lưu quá số ngày này; 0 = tắt |

## 4. Lưu trữ trên thiết bị

### 4.1 PWA — IndexedDB

- Cơ sở dữ liệu `lvhv`, phiên bản IndexedDB = `schema_version` (bắt đầu 1). Kho (object store): `persons`
  (keyPath `id`), `events` (`id`, chỉ mục `by_person` multiEntry trên `person_ids`), `event_types` (`id`, chỉ mục
  duy nhất `by_name_key`), `photos` (`id`; giá trị `{meta, blob}`), `settings` (khóa cố định `"app"`), `meta`
  (`schema_version`, `created_with_app_version`).
- `localStorage` chỉ giữ tùy chọn giao diện như v0.1 (`lvhv.ui`). **Không** đặt dữ liệu cá nhân vào `localStorage`,
  `sessionStorage`, Cache Storage hay URL.
- Khi người dùng lưu hồ sơ đầu tiên: gọi `navigator.storage.persist()` (nếu có) và ghi kết quả; nếu bị từ chối, giao
  diện nói rõ "trình duyệt có thể xóa dữ liệu khi máy thiếu chỗ — hãy sao lưu" (rủi ro R-PWA-1, 05 §7).
- Mọi thao tác ghi một hồ sơ là **một giao dịch** (transaction) gồm cả ảnh và sự kiện liên quan.

### 4.2 Native — SQLite

- Một tệp `lvhv.sqlite` trong thư mục riêng của ứng dụng (Android: thư mục `databases/` mặc định; Apple:
  `Application Support/`; Windows: `%LOCALAPPDATA%\ClevixLab\LichVanHoaViet\`).
- **Tắt** sao lưu tự động lên đám mây của hệ điều hành cho cơ sở dữ liệu và thư mục ảnh trong v0.2 (Android:
  `android:allowBackup="false"` và `dataExtractionRules` loại trừ; Apple: `isExcludedFromBackup` cho hai đường dẫn).
  Lý do: sao lưu tự động của hệ điều hành nằm ngoài phạm vi "chỉ trên thiết bị" của v0.2 và không do người dùng chủ
  động. Hệ quả: mất máy là mất dữ liệu nếu người dùng chưa xuất tệp sao lưu (rủi ro R-DATA-1, 05 §7). Tiêu chí: `04`
  S5.
- `PRAGMA user_version` = `schema_version`; `PRAGMA foreign_keys = ON`; chế độ `journal_mode = WAL`.
- Bảng `person`, `event`, `event_person` (n–m), `event_type`, `photo`, `settings`. Các đối tượng nhúng
  (`DeathInfo`, `BirthInfo`, `MemorialSettings`, `ReminderRule`) lưu dạng cột JSON (TEXT) đã kiểm tra theo lược
  đồ ở cùng lớp mã — để một lớp tuần tự hóa duy nhất dùng chung cho SQLite và tệp sao lưu.
- Đề xuất (quyết định ở đợt 3): một crate Rust `lvhv-store` (rusqlite, SQLite đóng gói kèm) gọi qua `ffi/` cho cả
  Android, Apple và Windows, để chỉ có **một** cài đặt lưu trữ và chuyển đổi phiên bản cho ba nền tảng.

## 5. Lược đồ JSON dùng chung (sao lưu, kiểm thử, trao đổi giữa PWA và native)

- Tệp: `docs/v0.2/schema/backup-v1.schema.json` (JSON Schema draft 2020-12). Đối tượng gốc:

```json
{
  "format": "lvhv-backup",
  "schema_version": 1,
  "app": { "name": "Lịch và Văn hóa Việt", "version": "0.2.0", "platform": "pwa" },
  "exported_at": "2026-11-06T03:15:00Z",
  "settings": { … AppSettings … },
  "event_types": [ … ],
  "persons": [ … ],
  "events": [ … ],
  "photos": [ { …Photo (siêu dữ liệu)… } ]
}
```

- Byte ảnh **không** nằm trong JSON; chúng đi kèm trong vùng chứa của tệp sao lưu (03 §2.3), khớp theo `id` và
  `sha256`.
- Lược đồ cấm khóa lạ (`additionalProperties: false`) ở mọi đối tượng, để lỗi chính tả hoặc dữ liệu lạ bị phát hiện
  ngay. Một số ràng buộc chéo không diễn đạt được bằng JSON Schema (ngày tồn tại trong lịch âm, tham chiếu id tồn
  tại, `deceased` ⇒ có `death`…) được liệt kê ở §6 và kiểm tra bằng mã.
- Tệp ví dụ hợp lệ: `docs/v0.2/schema/example-backup-v1.json` (dùng làm dữ liệu kiểm thử đợt 1).

## 6. Kiểm tra dữ liệu

### 6.1 Ở lớp nhập liệu (giao diện)

Thông báo lỗi bằng tiếng Việt, chỉ rõ trường. Không cho lưu khi còn lỗi. Ví dụ: "Họ tên không được để trống",
"Tháng 7 âm lịch năm 2026 chỉ có 29 ngày", "Năm 2026 không có tháng 6 nhuận".

### 6.2 Ở lớp lưu trữ (bất biến)

| Mã | Bất biến |
|---|---|
| V1 | Mọi bản ghi khớp lược đồ JSON (§5) |
| V2 | `kind = "deceased"` ⇔ `death ≠ null` ⇔ `memorial ≠ null` |
| V3 | `photo_id`, `type_id`, `person_ids[*]` trỏ tới bản ghi tồn tại |
| V4 | Mọi `DateValue` có năm trong miền đổi lịch là ngày tồn tại (§2.1) |
| V5 | Trường dẫn xuất khớp kết quả tính lại từ `input` (cho phép khác **chỉ** ở `death.lunar` khi `method` khác phương pháp hiện hành — khi đó giữ giá trị đã lưu) |
| V6 | `memorial.tray_style = "cong_giao"` ⇒ `memorial.incense = false` |
| V7 | Tên `EventType` duy nhất theo khóa so sánh (§3.5) |
| V8 | `offsets_days` tăng dần, không trùng, trong 0–60 |

### 6.3 Khi nhập tệp sao lưu

Thứ tự: giải mã (03) → phân tích JSON (giới hạn độ sâu 32, kích thước 64 MB) → kiểm tra lược đồ → kiểm tra V2–V8 →
kiểm tra `sha256` từng ảnh → xem trước (số người, số sự kiện, số ảnh, ngày xuất) → người dùng chọn **Thay toàn bộ**
hoặc **Gộp** → ghi trong một giao dịch. Bất kỳ lỗi nào → không ghi gì, báo lỗi chung, dữ liệu hiện có giữ nguyên.

Gộp: theo `id`; cùng `id` thì giữ bản có `rev` lớn hơn, bằng nhau thì giữ bản có `updated_at` muộn hơn, vẫn bằng
thì giữ bản trên máy. Loại sự kiện trùng tên (khác id) → gộp về id trên máy và đổi `type_id` của sự kiện nhập vào.

## 7. Phiên bản lược đồ và chuyển đổi

- `schema_version` là số nguyên, bắt đầu **1** ở v0.2 (v0.1 không lưu dữ liệu người dùng nên không có phiên bản 0
  cần chuyển; tùy chọn giao diện `lvhv.ui` giữ nguyên).
- Thêm trường tùy chọn có giá trị mặc định, thêm giá trị enum, siết hay nới ràng buộc, đổi tên trường → đều **tăng**
  `schema_version` (không dùng "phiên bản phụ" để tránh nhập nhằng khi kiểm tra lược đồ đóng).
- Mỗi lần tăng có một hàm chuyển `migrate_N_to_N+1(doc) -> doc` thuần (không I/O), được cài **một lần** trong lõi
  Rust và bản TypeScript, và kiểm thử bằng tệp mẫu cố định của phiên bản N.
- Mở cơ sở dữ liệu trên máy có `schema_version` < phiên bản ứng dụng: sao một bản sao lưu nội bộ không mã hóa trong
  thư mục riêng của ứng dụng (giữ 1 bản, xóa sau 7 ngày nếu chuyển đổi thành công) → chạy chuỗi chuyển đổi trong
  một giao dịch → kiểm tra §6.2 → ghi `schema_version` mới. Lỗi → quay lui, mở ở chế độ chỉ đọc và hướng dẫn xuất
  sao lưu.
- Cơ sở dữ liệu hoặc tệp sao lưu có `schema_version` **lớn hơn** phiên bản ứng dụng hiểu: **từ chối** với thông
  báo "Tệp được tạo bởi phiên bản mới hơn — hãy cập nhật ứng dụng". Không cố đọc một phần.
- Lược đồ cũ được giữ nguyên trong kho (`schema/backup-vN.schema.json`), không sửa sau khi phát hành.
