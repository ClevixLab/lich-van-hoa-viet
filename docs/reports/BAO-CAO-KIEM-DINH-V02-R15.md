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

Phần đăng ký trước ở §1 được commit riêng (`2493b0b`, 2026-11-07 17:48 +0700) trước lần chạy đầu tiên; không sửa sau đó.

Máy đo: máy Linux x86-64 của bên kiểm định, `cargo build --locked -p lvhv-ffi --bin lvhv-store-tool` (bản debug),
Node 22.23.4, `pwa` cài bằng `npm ci`. Ngày đo: 2026-11-07. Commit đo: `eafa96e` (nhánh `v0.2-yc10-core`).
Dữ liệu thô: `test-vectors/oracle-r15/raw/` (`results.json` gồm mọi ca của hai bản; `scenario-rs.json`,
`scenario-ts.json`; `images-manifest.json` gồm sha256 của mọi ảnh thử). Chạy lại:

```
python3 -I test-vectors/oracle-r15/make_inputs.py <w>/img
python3 -I test-vectors/oracle-r15/run_r15.py <repo> <w> <repo>/target/debug/lvhv-store-tool
```

Kiểm tra bộ đóng gói của bên kiểm định: giải mã `example-v1.lvhv` cho `json_sha256 = b7236778…a1f3` đúng giá trị khóa
trong `test-vectors/manifest.json`, và JSON bản rõ đúng dạng chuẩn. Tệp đối chứng d9 do bộ này mã hóa được cả hai bản
nhận. Như vậy, các ca d5–d8 bị từ chối là do nội dung, không do bộ đóng gói.

### 2.1 (c) — phía lõi

| Ca | Kỳ vọng | Rust | TS |
|---|---|---|---|
| c1 JPEG EXIF+GPS / JPEG XMP / JPEG APP13 / WebP EXIF | từ chối | 4/4 từ chối (`input`) | 4/4 từ chối (`input`) |
| c1 bổ sung: HTML dán nhãn `image/jpeg` | từ chối | từ chối | từ chối |
| c2 JPEG sạch, WebP sạch, JPEG sạch thứ hai | nhận; sha256 giữ; 0 siêu dữ liệu | 3/3 nhận; sha256 bằng tệp vào (đọc từ kho và từ tệp xuất tự giải mã); bên kiểm định tự duyệt đoạn/khối: 0 APP1, 0 APP13, 0 EXIF/XMP | như Rust |
| c3 JPEG sạch 2 305 430 byte | từ chối V1 `/photo/bytes` | `invalid V1 /photo/bytes` | `invalid V1 /photo/bytes` |
| c4 A gắn ảnh mâm lễ P | nhận | nhận | nhận |
| c4 B gắn cùng P làm ảnh mâm lễ | V3 | V3 `/person/memorial/tray_photo_id` | như Rust |
| c4 B dùng P làm `photo_id` | V3 | V3 `/person/photo_id` | như Rust |
| c4 B dùng ảnh chân dung của chính mình làm ảnh mâm lễ | V3 | V3 `/person/memorial/tray_photo_id` | như Rust |
| c4 `tray_photo_id` trỏ ảnh không có | V3 | V3 `/person/memorial/tray_photo_id` | như Rust |
| c4 bổ sung: A dùng ảnh chân dung của B làm ảnh mâm lễ; người sống dùng ảnh mâm lễ của B làm `photo_id` | V3 | V3 (cả hai) | như Rust |
| c4 trạng thái kho sau cùng | 0 ảnh mồ côi | 0 bản ghi, 0 tệp mồ côi | 0 bản ghi mồ côi |
| **c5 (thăm dò)** JPEG sạch 2000 × 120, gắn làm ảnh mâm lễ | (ghi nhận) | **nhận** cả `add-photo` lẫn `save-person` | **nhận** cả hai |

Kết quả kịch bản cố định (khóa `c_add_photo`): `exif = input`, `over_1mib = invalid V1 /photo/bytes`, `ok = ok`, đúng mệnh đề.

### 2.2 (d)

| Ca | Rust | TS |
|---|---|---|
| d1 xuất: tài liệu có `tray_photo_id`; khối ảnh có sha256 bằng tệp gốc (đọc bằng `inspect-export` và bằng bộ giải mã độc lập) | đúng | đúng |
| d2 Thay toàn bộ vào kho trống | liên kết giữ, sha256 bằng, 0 mồ côi | như Rust |
| d3 Gộp vào kho trống | liên kết giữ, sha256 bằng, 0 mồ côi | như Rust |
| d4 Gộp vào kho có bản cũ (`rev` 1, chưa có ảnh; tệp có `rev` 2) | liên kết giữ, sha256 bằng, 0 mồ côi | như Rust |
| d5 `tray_photo_id` trỏ ảnh không có (Thay / Gộp) | từ chối, `open_backup`, V3 `/persons/1/memorial/tray_photo_id`; kho không đổi | như Rust; thông báo chung "Sai mật khẩu hoặc tệp bị hỏng" |
| d6 ảnh mâm lễ JPEG EXIF+GPS; d6b JPEG XMP; d6c WebP EXIF (Thay / Gộp) | từ chối, `open_backup`, `PHOTO /photos: metadata`; kho không đổi | như Rust |
| d7 "ảnh" HTML (Thay / Gộp) | từ chối, `open_backup`, `PHOTO /photos: mime_mismatch` | như Rust |
| d8 hai người dùng chung ảnh mâm lễ (Thay / Gộp) | từ chối, `open_backup`, V3 `/persons/0/memorial/tray_photo_id` | như Rust |
| d8b bổ sung: ảnh mâm lễ của người này là ảnh chân dung của người kia | từ chối, V3 `/persons/0/photo_id` | như Rust |
| d9 đối chứng hợp lệ (Thay / Gộp) | nhận, sha256 bằng, 0 mồ côi | như Rust |
| d4b (thêm sau đăng ký, ngoài ngưỡng): Gộp khi bản trên máy mới hơn và không có ảnh | ảnh của tệp **không** được thêm, 0 mồ côi | như Rust |

Kịch bản cố định (`d_round_trip`): ba lần nhập `sha_equal = true`, `orphan_records = orphan_files = 0`; `crafted` gồm
`dangling_tray`, `shared_tray`, `tray_with_exif` đều `backup` (thông báo chung), `control = ok`.

### 2.3 (e)

| Ca | Rust | TS |
|---|---|---|
| e1 nhập `example-v1.lvhv`, Thay toàn bộ / Gộp vào kho trống | ok; tài liệu kho bằng từng byte tài liệu của tệp (bỏ `app`, `exported_at`) sau chuẩn hóa; không có khóa `tray_photo_id`; ảnh giữ sha256 | như Rust |
| e2 mở bộ mẫu đóng băng `0.2-dev-before-attachments` | `migrated`; `state_sha256` bằng tệp kỳ vọng; không có `tray_photo_id` | như Rust |
| e3 xuất lại kho của e1 rồi giải mã | không có chuỗi `tray_photo_id`; tài liệu (bỏ `app`, `exported_at`) bằng từng byte bản rõ của `example-v1.lvhv`; khối ảnh bằng; JSON chuẩn; `legacy_paths = []` | như Rust |
| e4 sha256 `example-v1.lvhv` | `a17f91d9…5a0f` = `manifest.json` = tệp ở `41cd3af^` (trước YC-2026-10) = `origin/main` | — |
| **e5 (thăm dò)** tệp có `"tray_photo_id": null` tường minh | nhận; kho **giữ** khóa `null`; xuất lại tệp **có** `"tray_photo_id":null` | như Rust |

Kịch bản cố định (`e_old_backup`): `unchanged = true`, `has_tray_key = false`, `tray_d1 = absent`.

### 2.4 (g)

- g1: `scenario tray_photo` của hai bản giống nhau từng byte (sha256 `ff5a9d48…2371`, hai lần chạy) và bằng
  `tray-photo/expected-scenario.json` sau chuẩn hóa. Các giá trị trong đó khớp mệnh đề (xem 2.1–2.3; `f_cleanup`:
  mọi `orphan_records = orphan_files = 0`).
- g2: 30 kho cuối của các ca (lệnh rời) có tài liệu (bỏ thời điểm) và sha256 ảnh **giống hệt** giữa hai bản
  (29/29 cặp so được; cặp e2 không so được bằng `--state` vì bản TS mở thẳng thư mục bộ mẫu, nhưng cả hai bản đều
  bằng `state_sha256` kỳ vọng). Mọi ca cho cùng kết luận nhận/từ chối, cùng `outcome`, cùng mã V và đường dẫn.
  **Khác biệt duy nhất:** 7 ca `save-person` bị từ chối (c4) in `stage = "save_person_json"` ở Rust và
  `stage = "save_person"` ở TS. Mã V, đường dẫn và thông báo giống nhau. 10 §3.3 không quy định tên `stage` cho lệnh
  này (chỉ §3.2 quy định cho `import`).

## 3. Phát hiện

- **F1 (c, cạnh dài > 1600 px).** Ở `eafa96e`, cả hai lõi nhận ảnh mâm lễ 2000 × 120 qua `add-photo` và `save-person`.
  01 §3.7 ghi "cạnh dài ≤ 1600 px"; 10 §3.9 giao việc thu nhỏ cho nền tảng. Từ đó, bất biến này **không** được lõi bảo
  đảm: một nền tảng thu nhỏ sai, hay một tệp sao lưu sửa tay, đều đưa được ảnh > 1600 px vào kho. Trên `origin/main`
  đã có "Bổ sung 01-YC10-a" (MA10-02, ghi sau khi thấy kết quả duyệt mã), yêu cầu lõi từ chối ở cả lưu và nhập; commit
  được đo chưa cài phần này. Khi chạy lại trên commit sửa, đây sẽ là một ca từ chối mới, theo bổ sung đó; ca này
  **không** làm thay đổi ngưỡng đã đăng ký ở §1.
- **F2 (e5, `null` tường minh).** Tệp có `"tray_photo_id": null` được nhận và giữ nguyên khóa. Tệp xuất ra sau đó có
  khóa `null`, trong khi 01 §3.7 ghi "khi `null` thì **không ghi** khóa này". Hai bản xử lý như nhau. Đây là hệ quả
  của quy tắc "nhập giữ nguyên byte", và tệp do ứng dụng tạo không có dạng này. Cần quyết định bằng một trong hai cách:
  từ chối hoặc chuẩn hóa khi nhập, hoặc ghi rõ ngoại lệ trong 01 §3.7.
- **F3 (g2, tên `stage`).** Lệnh `save-person` từ chối in tên `stage` khác nhau ở hai bản (§2.4). Cách sửa: thống nhất
  tên, hoặc ghi rõ trong 10 §3.3 rằng tên `stage` của lệnh này là tên hàm của từng bản.
- **F4 (an toàn công cụ).** Ở `eafa96e`, `scenario tray_photo <thư-mục>` của bản Rust xóa đệ quy thư mục được truyền
  vào (điều phối viên đã báo; bên cài đặt đang sửa). Lần đo này chỉ truyền thư mục tạm mới.
- Ghi nhận thêm (ngoài phạm vi (c)(d)(e)(g), không kết luận): MA10-03 trong "Bổ sung 01-YC10-a" quy định dọn ảnh trên
  máy không còn được dùng sau khi Gộp. Ca d4b chỉ kiểm chiều ngược lại (ảnh của tệp không được thêm khi bản trên máy
  thắng), nên chưa kiểm MA10-03.

## 4. Kết luận (theo quy tắc khóa ở §1)

| Mệnh đề | Kết luận | Lý do |
|---|---|---|
| (c) | **CHƯA ĐỦ BẰNG CHỨNG** | Phía lõi: c1–c4 đúng hết ở cả hai bản, 0 ảnh còn siêu dữ liệu. Còn thiếu bằng chứng của nền tảng: mã hóa lại khi chọn tệp có EXIF/GPS/XMP, thu nhỏ ≤ 1600 px, ảnh chỉ hiện trên màn hình của đúng người đó, ưu tiên hơn ảnh Clevix. Công cụ hộp đen không kiểm được các phần này. Xem thêm F1: lõi ở commit này không chặn được ảnh > 1600 px |
| (d) | **ĐẠT** (Rust và TS) | d1–d4, d9 đúng; d5–d8 bị từ chối ở `open_backup` cả khi Thay toàn bộ lẫn khi Gộp, kho không đổi; sha256 bằng nhau; 0 ảnh mồ côi |
| (e) | **ĐẠT** (Rust và TS) | e1–e4 đúng: 0 bản ghi mất, 0 trường khác, vector 03 §2.5 không đổi. F2 là ca thăm dò, ngoài ngưỡng |
| (g) | **KHÔNG ĐẠT** theo quy tắc đã khóa ở §1 | g1 (đúng mệnh đề 04: kịch bản `tray_photo`) **đạt**: hai bản giống nhau từng byte và bằng tệp kỳ vọng. g2 (phép so rộng hơn mà bên kiểm định đã đăng ký: cùng `stage`) có 7 khác biệt về tên `stage` (F3). Mọi kết quả thực chất đều giống nhau. Theo ngưỡng "0 khác biệt" đã khóa, kết luận là KHÔNG ĐẠT. Thống nhất tên `stage`, hoặc ghi vào 10 §3.3 bằng một Bổ sung có chủ dự án ký, rồi chạy lại g2 thì mới đổi được kết luận. Bên kiểm định không tự nới quy tắc |

Những việc cần làm trên commit sửa (theo thông báo của điều phối viên): chạy lại toàn bộ `run_r15.py`. Thêm ca từ chối
ảnh mâm lễ > 1600 px khi lưu và khi nhập (theo Bổ sung 01-YC10-a, MA10-02), đăng ký trước khi chạy. Tách việc kiểm
MA10-03 thành ca riêng nếu được giao.

---

# Vòng 2 — đo lại trên commit sửa `375d6d66530055ed91b77cbdf23e25b522ade767`

## 5. Đăng ký bổ sung V2 (2026-11-07; viết và commit TRƯỚC khi chạy vòng 2, SAU KHI đã thấy kết quả vòng 1)

Phần này được viết sau khi đã thấy kết quả vòng 1 (§2–§4) và sau khi đặc tả có "Bổ sung 01-YC10-a" (01, MA10-02,
MA10-03) cùng các sửa ở 10 §3.3 và §3.11. Đăng ký này không nới ngưỡng nào của §1. Nó chỉ thêm ca thử cho các quy tắc
mới. Tài liệu đã đọc thêm: phần đổi của 01 và 10 giữa `eafa96e` và `375d6d6`. Vẫn không đọc mã cài đặt hay kiểm thử
của bên cài đặt.

- **V2-0 (chạy lại).** Chạy lại `run_r15.py` không sửa, ngưỡng §1 giữ nguyên. Theo 01-YC10-a, c5 (ảnh mâm lễ 2000 px)
  nay phải bị từ chối. c5 vẫn là ca thăm dò; quy tắc mới được kiểm bằng V2-1.
- **V2-1 (c, MA10-02 khi lưu).** Hai bản thêm ảnh sạch tự dựng 1601 × 100, 100 × 1601 và 1600 × 100.
  - Gắn 1601 × 100 hoặc 100 × 1601 làm ảnh mâm lễ → `save-person` từ chối.
  - Gắn 1600 × 100 → nhận.
  - Dùng ảnh 1601 × 100 làm **ảnh chân dung** (`photo_id`) → nhận, vì giới hạn này chỉ áp cho ảnh mâm lễ; ảnh chân
    dung giới hạn 2048 px (01 §3.8).
  - Ngưỡng: đúng 4/4 ca ở cả hai bản. Ca từ chối không ghi gì: ảnh mâm lễ cũ của người đó, nếu có, còn nguyên.
- **V2-2 (d, MA10-02 khi nhập).** Tệp tự mã hóa có ảnh mâm lễ 1601 × 100, bản ghi và byte khớp.
  - Thay toàn bộ và Gộp → từ chối ở `open_backup`, kho không đổi.
  - Tệp đối chứng có ảnh mâm lễ 1600 × 100 → nhận.
  - Tệp có ảnh 1601 × 100 làm ảnh chân dung → nhận.
  - Ngưỡng: 100 % ở cả hai bản.
- **V2-3 (d, MA10-03 dọn sau Gộp).** Kho trên máy dựng bằng nhập tệp tự mã hóa A:
  - 5e01: `photo_id` P0 và một tệp đính kèm báo tử F0 (PDF hợp lệ, tự dựng);
  - 5e02: ảnh mâm lễ T0;
  - 5e03 (đang sống): ảnh chân dung Q0.

  Sau đó Gộp tệp B. Trong B, 5e01 và 5e02 có `rev` lớn hơn: 5e01 không có `photo_id`, không có tệp đính kèm; 5e02 có
  ảnh mâm lễ T1. B không có 5e03.

  Kỳ vọng sau Gộp:
  - P0, F0, T0 không còn (bản ghi; ở Rust, cả tệp `photos/*.img`);
  - T1 có, `sha256` bằng;
  - Q0 **còn** (vẫn được dùng; không mất dữ liệu đang dùng);
  - 0 bản ghi ảnh hoặc tệp đính kèm mà không hồ sơ nào trỏ tới; Rust 0 tệp `photos/*.img` thừa hoặc thiếu.

  Thêm ca ngược: Gộp B vào kho mà 5e01 và 5e02 có `rev` **lớn hơn** B. Kỳ vọng: P0, F0, T0 còn; T1 không được thêm.

  Ngưỡng: đúng hết, cả hai bản.
- **V2-4 (e, `null` tường minh).**
  - Nhập tệp có `"tray_photo_id": null` (Thay toàn bộ và Gộp) → nhận; tài liệu kho **không** có khóa `tray_photo_id`;
    xuất lại → bản rõ không có chuỗi `tray_photo_id`.
  - `save-person` với `"tray_photo_id": null` → bản đã lưu không có khóa.
  - Ngưỡng: đúng hết, cả hai bản.
- **V2-5 (g, tên `stage`).** Mọi lần `save-person` bị từ chối trong V2-0 và V2-1: `stage` giống nhau ở hai bản và thuộc
  {`read_file`, `open`, `save_person`} (10 §3.3 mới). Ngưỡng g2 của §1 (0 khác biệt) giữ nguyên trên mọi ca của V2-0..V2-4.
  Thêm g1: kịch bản `tray_photo` của hai bản giống nhau từng byte và bằng `expected-scenario.json` mới. Rust chạy trên
  một thư mục làm việc có sẵn `keep/file.txt`; sau khi chạy, tệp đó phải còn nguyên và không có gì khác bị để lại
  (10 §3.11 mới, MA10-01).

Quy tắc kết luận vòng 2 (khóa trước):
- (c): V2-0 phần c và V2-1 đúng hết → phía lõi đạt. (c) toàn phần vẫn **CHƯA ĐỦ BẰNG CHỨNG** vì thiếu bằng chứng nền
  tảng, như §1. Một ca sai → KHÔNG ĐẠT.
- (d): V2-0 phần d, V2-2, V2-3 đúng hết → ĐẠT. Một ca sai → KHÔNG ĐẠT.
- (e): V2-0 phần e và V2-4 đúng hết → ĐẠT. Một ca sai → KHÔNG ĐẠT.
- (g): g1 và V2-5 đúng hết, 0 khác biệt → ĐẠT. Một khác biệt → KHÔNG ĐẠT.
