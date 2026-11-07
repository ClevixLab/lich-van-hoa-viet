# Báo cáo kiểm định độc lập — v0.2, R15 (c)(d)(e)(g): ảnh mâm lễ (`memorial.tray_photo_id`)

Bên kiểm định: một bên không tham gia xây dựng đặc tả và không viết mã lõi v0.2.
Tiêu chí: `docs/v0.2/04-TIEU-CHI-NGHIEM-THU-V02.md`, Bổ sung B8, R15 (c)(d)(e)(g).
Mã được kiểm: nhánh `v0.2-yc10-core`, commit `eafa96ec8f6427f7a9a1ab6e3e9dce88817f1a44` (2026-11-07 17:16 +0700).

Tài liệu đã đọc: 01, 03, 04, 07 §10, 10 (cách dùng công cụ), cùng vector đặc tả trong `docs/v0.2/schema/`.
**Không** đọc mã cài đặt (`core/`, `memorial/`, `store/`, `backup/`, `ffi/src` ngoài `--help` của công cụ, `tsport/`,
`pwa/src`) và không đọc kiểm thử của bên cài đặt. Mọi đầu vào thử do bên kiểm định tự dựng từ đặc tả.

## 1. Đăng ký trước (viết và commit TRƯỚC khi chạy bất kỳ phép thử nào)

Ngưỡng của 04 B8 / R15: "Mọi mệnh đề đúng; 0 ảnh còn siêu dữ liệu; 0 ảnh mồ côi sau (f)". Một vi phạm là đủ để
KHÔNG ĐẠT (04, mục đầu). Không nới ngưỡng sau khi thấy kết quả.

Mỗi phép thử chạy **trên cả hai bản** (Rust `lvhv-store-tool`, TS `pwa/test/tools/lvhv-store-tool.mjs`) với **cùng tệp
đầu vào**. Đầu vào tự dựng: JPEG/WebP thật cỡ nhỏ tạo bằng Pillow; JPEG có EXIF + GPS; JPEG có XMP (APP1); JPEG có
APP13 (IPTC); WebP có khối EXIF; "ảnh" giả (HTML dán nhãn `image/jpeg`); tệp sao lưu tự mã hóa bằng bộ đóng gói độc
lập viết theo 03 §2.1–§2.3 (Argon2id + XChaCha20-Poly1305), gồm tệp đối chứng hợp lệ.

### (c) — ảnh gia đình bỏ siêu dữ liệu, giới hạn, chỉ của đúng người đó

Phép thử phía lõi (qua `add-photo`, `save-person`, `open`):
- c1. JPEG có EXIF+GPS, JPEG có XMP, JPEG có APP13, WebP có EXIF → `add-photo` **từ chối** (ok = false). Ngưỡng: 4/4
  bị từ chối ở cả hai bản.
- c2. JPEG sạch và WebP sạch → nhận; byte đọc lại từ kho (`photo_bytes`) có `sha256` bằng tệp đầu vào; bên kiểm định tự
  duyệt đoạn JPEG / khối RIFF của byte đọc lại: 0 APP1, 0 APP13, 0 khối EXIF/XMP. Ngưỡng: 0 ảnh còn siêu dữ liệu.
- c3. Ảnh sạch > 1 048 576 byte → từ chối (V1 `/photo/bytes`). Ngưỡng: từ chối ở cả hai bản.
- c4. Liên kết đúng người: A (đã mất) gắn ảnh mâm lễ P → nhận. B (đã mất) gắn cùng P làm ảnh mâm lễ → từ chối V3; B dùng
  P làm `photo_id` → từ chối V3; A dùng ảnh chân dung của chính mình làm ảnh mâm lễ → từ chối V3 (theo 10 §3.11, khóa
  `set`); `tray_photo_id` trỏ ảnh không tồn tại → từ chối V3. Ngưỡng: mọi ca đúng như trên, cả hai bản.
- c5 (thăm dò, ghi nhận, không tự đổi kết luận): ảnh sạch cạnh dài 2000 px gắn làm ảnh mâm lễ. 01 §3.7 ghi "cạnh dài
  ≤ 1600 px"; 10 §3.9 giao việc thu nhỏ cho nền tảng. Kết quả chỉ ghi thành phát hiện.

Phần **không** kiểm được bằng công cụ hộp đen (theo 10 §3.9 là việc của nền tảng): mã hóa lại khi người dùng chọn tệp có
EXIF/GPS/XMP, thu nhỏ về ≤ 1600 px, hiển thị chỉ trên màn hình của đúng người đó, ưu tiên hơn ảnh Clevix.
Quy tắc kết luận (khóa trước): bất kỳ ca c1–c4 sai → **KHÔNG ĐẠT**. c1–c4 đúng hết → phần lõi đạt, nhưng (c) toàn phần
là **CHƯA ĐỦ BẰNG CHỨNG** (thiếu bằng chứng nền tảng), trừ khi có bằng chứng giao diện được nộp riêng.

### (d) — xuất rồi nhập giữ ảnh và liên kết; tệp sửa tay bị từ chối

- d1. Kho dựng bằng lệnh rời: `add-photo` (JPEG sạch) → `save-person` người đã mất có `memorial.tray_photo_id` → `export`
  → `inspect-export`: tài liệu có khóa `tray_photo_id` đúng id; danh sách khối có id đó với `sha256` bằng tệp đầu vào.
- d2. `import --replace` tệp đó vào kho trống: `tray_photo_id` giữ; `photo_bytes[id].sha256` = sha256 tệp gốc.
- d3. `import --merge` vào kho trống: như d2.
- d4. `import --merge` vào kho đã có bản cũ hơn (`rev` nhỏ hơn, chưa gắn ảnh) của cùng người: sau gộp người đó có
  `tray_photo_id`, ảnh có mặt, `sha256` bằng.
- d5. Tệp tự mã hóa: `tray_photo_id` trỏ ảnh không có → từ chối (`ok = false`, `stage = "open_backup"`).
- d6. Tệp tự mã hóa: ảnh mâm lễ là JPEG còn EXIF+GPS (bản ghi `sha256`/`bytes`/kích thước khớp) → từ chối, `open_backup`.
- d7. Tệp tự mã hóa: ảnh mâm lễ là "ảnh" giả (HTML, nhãn `image/jpeg`) → từ chối, `open_backup`.
- d8. Tệp tự mã hóa: hai người dùng chung một ảnh mâm lễ → từ chối (01, Ghi chú YC-2026-10: không dùng chung).
- d9. Tệp đối chứng tự mã hóa, hợp lệ, có ảnh mâm lễ → nhận, `sha256` giữ (chứng minh d5–d8 bị từ chối vì nội dung,
  không vì bộ đóng gói của bên kiểm định sai).
- d5–d8 chạy cả `--replace` và `--merge`.
Ngưỡng: d1–d4 và d9 đúng 100 %; d5–d8 bị từ chối 100 %; mọi ảnh sau nhập có `sha256` bằng bản gốc; 0 ảnh mồ côi
(bản ghi ảnh không hồ sơ nào dùng) sau mỗi lần nhập. Một ca sai → KHÔNG ĐẠT.

### (e) — dữ liệu và tệp của bản trước

- e1. `import example-v1.lvhv` (`--replace` và `--merge` vào kho trống): ok; tài liệu kho bằng tài liệu của tệp (bỏ `app`,
  `exported_at`) từng byte sau chuẩn hóa JSON; không hồ sơ nào có khóa `tray_photo_id`; ảnh giữ `sha256`.
- e2. Mở bộ mẫu đóng băng `released/0.2-dev-before-attachments` (bản sao) bằng `open`: ok; kho khớp
  `released/expected/0.2-dev-before-attachments.json` theo cách so của tệp đó; không hồ sơ nào có `tray_photo_id`.
- e3. Từ kho của e1, `export` rồi `inspect-export`: tài liệu không có chuỗi `tray_photo_id`; tài liệu (bỏ `app`,
  `exported_at`) bằng từng byte bản rõ của `example-v1.lvhv` (bỏ `app`, `exported_at`); khối ảnh bằng nhau.
- e4. `sha256` của `example-v1.lvhv` bằng giá trị đã khóa trong `test-vectors/manifest.json` và bằng tệp cùng tên trên
  `origin/main` trước YC-2026-10 (vector 03 §2.5 không đổi).
- e5 (thăm dò, không vào ngưỡng): tệp tự mã hóa có `"tray_photo_id": null` tường minh — ghi nhận nhận hay từ chối, và hai
  bản có như nhau không.
Ngưỡng: e1–e4 đúng hết (0 bản ghi mất, 0 trường khác). Một ca sai → KHÔNG ĐẠT.

### (g) — Rust ≡ TS

- g1. `scenario tray_photo`: stdout của hai bản giống từng byte, và bằng `tray-photo/expected-scenario.json` sau chuẩn
  hóa JSON. Bên kiểm định đọc các giá trị trong kết quả và đối chiếu với mệnh đề (e.g. `unchanged = true`,
  `sha_equal = true`, `orphan_records = 0`).
- g2. Trên mọi ca c1–c5, d1–d9, e1–e5: hai bản cho cùng kết luận nhận/từ chối và cùng `stage`; khi nhận, cùng tài liệu
  kho (bỏ id ngẫu nhiên, thời điểm, `app`) và cùng `sha256` từng ảnh. Mã lỗi hai bản có dạng khác nhau theo thiết kế
  (10 §2), nên chỉ so mã V/đường dẫn khi cả hai bản đều in ra.
Ngưỡng: 0 khác biệt. Một khác biệt → KHÔNG ĐẠT.

## 2. Kết quả

(điền sau khi chạy)
