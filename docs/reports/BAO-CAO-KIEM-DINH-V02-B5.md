# Báo cáo kiểm định độc lập v0.2 — Bổ sung B5 / R14 (mức lõi) và quy tắc dữ liệu YC-2026-07

Ngày lập: 2026-11-07. Người lập: bên không tham gia xây dựng đặc tả (kiểm định độc lập, phòng sạch).
Nhánh: `oracle-v02-b5` (từ `origin/main` 6ee647b). Đối tượng kiểm: PR #28, nhánh `v0.2-w3-core2`, head
`5535a49` (chưa gộp).

Căn cứ đặc tả (đọc trên `main` 6ee647b, **trước** khi viết kỳ vọng): `docs/v0.2/01` §1, §3.1, §3.2a, §3.8, §3.8a,
§6, §7; `03` §2 và Bổ sung B1; `04` R6 và Bổ sung B5 / R14; `07` (YC-2026-07, YC-2026-08). Tài liệu của PR #28
(01 §3.8a "Cách cài", 03 §2.3, lược đồ `backup-v1.schema.json`) chỉ được đọc **sau** commit kỳ vọng `d14f452`.
Không đọc mã nguồn `memorial/`, `store/`, `backup/`, `core/`, `tsport/`, `ffi/`, `pwa/`, `apps/`. Các tệp dữ liệu
dưới `test-vectors/` chỉ được đọc như dữ liệu. Công cụ của bên xây dựng chỉ được dựng và chạy như hộp đen.

## 1. Kết luận

| Phần | Kết luận | Số liệu |
|---|---|---|
| **R14 mức lõi — chuyển đổi tệp/kho cũ (`certificate_photo_ids` → `attachments`, `gioi_tinh` thiếu → `null`)** | **KHÔNG ĐẠT tại 5535a49** | 4/4 tệp dạng cũ có ảnh giấy chứng tử (M01, M02, M03, M05) bị **từ chối** (`V1`, "thiếu trường \"attachments\""). Không ghi gì, nên không mất bản ghi nào trong kho, nhưng dữ liệu bản 0.2 trước thay đổi lược đồ **không mở được** ở bản mới (trái B5 mục 1, 5; R14 (c)). Ở head này chưa có hàm chuyển đổi: B5 khóa lúc 10:48 (+07), sau commit 5535a49 (10:33 +07) |
| R14 mức lõi — tệp cũ không có thông tin báo tử | ĐẠT | M04 (chính `example-v1`), M07: nhập được, trạng thái kho **giống từng byte** với kỳ vọng (băm trạng thái khớp), 0 bản ghi mất, 0 trường khác |
| R14 mức lõi — từ chối phiên bản mới hơn | **ĐẠT** | `schema_version` 3 và 99 → `NewerSchema(3)`, `NewerSchema(99)`; không đọc một phần. `file_version` = 2 nằm trong vectơ R6 đã khóa |
| R14 mức lõi — vectơ R6 đã khóa vẫn giống từng byte | **ĐẠT** | Dựng 5535a49, sinh lại bằng `gen_test_vectors`: 27/27 tệp của bộ R6 (`example-v1.*`, `dataset-r6.json`, `manifest.json`, `merge-cases.json`, `validation-cases.json`, ảnh mẫu, 21 tệp `corrupt/`) giống từng byte bản trên `main` |
| R14 mức lõi — khóa lạ giữ trong `legacy` | CHƯA RÕ (đã đăng ký trước, A-M7) | M06 bị từ chối cả tệp (`V1`, "trường lạ"). Không bỏ lặng lẽ, nhưng cũng không giữ. Đặc tả có hai quy tắc trái nhau ở đây (B5 mục 2 và 01 §6.3) |
| R14 — nâng cấp tại chỗ trên từng nền tảng | NGOÀI PHẠM VI | Do các job CI của nền tảng chạy (bản web, Android, Windows, Apple) |
| **YC-2026-07 — `gioi_tinh`** | **ĐẠT** | 4/4 giá trị sai bị từ chối `V1` (`"Nam"`, `"khac"`, `""`, `"nữ"`). Thiếu khóa và `null` được nhận. Kho ghi `null` bằng cách **bỏ khóa** (băm trạng thái khớp biến thể "bỏ `gioi_tinh: null`") |
| **YC-2026-07 — tệp đính kèm** (≤ 6, `kind`, `mime`, `%PDF-`, giới hạn cỡ, `sha256`, `bytes`, trùng id, thiếu tệp) | **KHÔNG ĐẠT (1 sai lệch)** | 21/22 tệp không hợp lệ bị từ chối đúng lớp. Có 1 tệp bị **nhận nhầm**: X21, bản ghi khai `image/jpeg` nhưng byte là một tệp PDF. 5/5 tệp hợp lệ, kể cả hai ca đúng biên (PDF 10 485 760 byte, ảnh 2 097 152 byte, đúng 6 tệp), được nhận với trạng thái kho giống từng byte |
| YC-2026-07 — viết hoa `danh_xung`/`quan_he` khi **nhập** (không được áp dụng) | **ĐẠT** | M07: `"ông"`, `"bà cô tổ họ"`, `"BÀ cố"`, `"cụ"` (người sống), `"lê thị c"` giữ nguyên từng byte |
| YC-2026-07 — viết hoa khi **lưu** | CHƯA ĐỦ BẰNG CHỨNG | Hàm lưu từ giao diện (`save_person_json`) chỉ có qua FFI, không có công cụ hộp đen nào gọi được. Kỳ vọng 28 ca đã khóa ở `oracle/v02/out/b5_expected.json` (`names_on_save`) |
| YC-2026-07 — danh sách gợi ý (người mất 22 mục / người sống 17 mục) | CHƯA ĐỦ BẰNG CHỨNG | Không có hàm hộp đen hay vectơ dữ liệu nào lộ danh sách. Kỳ vọng đã khóa ở `b5_expected.json` (`goi_y`) |

Trạng thái chung: **R14 mức lõi KHÔNG ĐẠT tại 5535a49.** Theo R14, kết quả này chặn phát hành cho đến khi PR #28 có
chuyển đổi và chạy lại đạt. Cách chạy lại ở mục 7.

## 2. Trình tự làm việc (để kiểm được tính đăng ký trước)

| Giờ (+07) | Commit (`oracle-v02-b5`) | Việc |
|---|---|---|
| 11:02 | — | Đọc đặc tả trên `main` 6ee647b. Kiểm bộ đọc/ghi `.lvhv` độc lập (`lvhvfile.py`): giải mã `example-v1.lvhv` và `supplement-yc05/notice-v1.lvhv` của `main`, mã hóa lại với cùng salt/nonce ra **giống từng byte** |
| 11:11 | **`d14f452`** | Commit tham chiếu (`b5_ref.py`: chuyển đổi 1→2, bộ kiểm tra, quy tắc viết hoa, danh sách gợi ý, 15 giả định A-\*), 48 tệp thử (`b5_fixtures.py`, `fixtures/`) và kết quả kỳ vọng (`out/b5_expected.json`, `out/b5_expected/`). Đẩy lên trước khi đọc tài liệu PR #28 hay chạy công cụ của nó |
| 11:12–11:20 | — | Đọc tài liệu PR #28 (01 §3.8a "Cách cài", 03 §2.3, lược đồ). Dựng 5535a49 dưới thư mục nháp. Chạy `--help` cho các ví dụ của `lvhv-store`, sau đó chạy hộp đen |
| 11:21 | **`7f38eb3`** | Bộ chuyển dạng **sau đối chiếu** (`b5_adapt.py`), băm trạng thái độc lập (`b5_statehash.py`), so sánh (`b5_compare.py`, `b5_cmp_docs.py`), kết quả `out/b5_cmp_5535a49-*.json` |

Kỳ vọng (`b5_expected.json`, `b5_expected/*.json`) **không bị sửa** sau `d14f452`. Bốn tệp lớn (G04, G05, X02, X03;
10–11 MB) được sinh tất định lúc chạy (`b5_fixtures.py --big DIR`), không commit. `sha256` của chúng ghi trong
`fixtures/manifest.json` ở `d14f452`.

## 3. Phương pháp hộp đen

- **Đường vào.** Ở 5535a49, `lvhv-store` không có công cụ dòng lệnh nào nhận một tệp sao lưu tùy ý. `cross_platform
  --help` in ra "dùng: cross_platform write|check". Lệnh `check` báo thiếu `r6-from-pwa.lvhv` (đường dẫn lấy từ
  thông báo lỗi của chính nó). Bên kiểm định đặt lần lượt từng tệp thử vào chỗ đó rồi chạy `check`. Lệnh này mở tệp
  bằng mật khẩu mẫu của 03 §2.5, nhập vào kho SQLite trống, rồi so băm trạng thái với bộ R6.
  - Tệp bị từ chối: chương trình in lỗi kèm mã lớp và đường dẫn, ví dụ `Invalid(ValidationError { code: "V1", path,
    msg })`, `NewerSchema(n)` hoặc `Backup(MalformedPlaintext)`.
  - Tệp được nhận: chương trình in `left:` là băm trạng thái kho sau khi nhập.
- **Băm trạng thái độc lập.** `b5_statehash.py` cài lại theo 03 B1.4 mục 10. Với tệp đính kèm, đặc tả trên `main`
  chưa nói gì. Biến thể "sha256 của ảnh theo `id`, rồi của tệp đính kèm theo `id`" được chọn vì tái tạo đúng
  `state_sha256` công bố trong vectơ `supplement-yc05/manifest.json` của PR (`ac8a9dce…`), và tái tạo đúng `left` của
  G01 (`c369a8e3…`). Hai băm bằng nhau nghĩa là JSON chuẩn toàn kho và byte của mọi ảnh/tệp đều giống, tức 0 bản
  ghi mất và 0 trường khác.
- **Chuyển dạng sau đối chiếu** (`b5_adapt.py`, không sửa kỳ vọng). PR #28 giữ `schema_version` = 1 cho lược đồ mới
  và chỉ ghi khóa gốc `attachments` khi mảng khác rỗng (01 §3.8a trên PR). Tham chiếu của bên kiểm định đặt
  `schema_version` = 2 (A-V1). Để so các quy tắc của trường mới, các tệp v2 được hạ về 1 và bỏ `attachments` rỗng.
  Các khối nhị phân được xếp ảnh trước, tệp đính kèm sau (03 §2.3 trên PR). Bố cục khối trùng với giả định A-C1.
  Chạy không chuyển dạng: `out/b5_cmp_5535a49-raw.json`. Ở lượt đó, mọi tệp v2 bị từ chối `NewerSchema(2)`.

## 4. Kết quả từng tệp thử (lượt đã chuyển dạng, `out/b5_cmp_5535a49-adapted.json`)

Tổng: 38 KHỚP, 4 sai lệch "từ chối tệp phải nhận", 2 sai lệch "nhận tệp phải từ chối", 4 ghi nhận (ca đã đăng ký là
chưa rõ).

| Nhóm | Tệp | Kỳ vọng | Lõi 5535a49 | Đánh giá |
|---|---|---|---|---|
| Tệp cũ có ảnh giấy chứng tử | M01 (3 ảnh, 2 hồ sơ), M02 (4 ảnh), M03 (ảnh giấy chứng tử trùng ảnh chân dung), M05 (`notice` rỗng) | nhận, chuyển sang `attachments` | từ chối `V1` "thiếu trường attachments" | **SAI LỆCH** (mục 5.1) |
| Tệp cũ không có thông tin báo tử | M04, M07 | nhận, giữ nguyên | nhận, băm khớp | KHỚP |
| Tệp cũ NFD | M08 | chưa rõ (A-N2) | nhận, **giữ nguyên NFD** (băm khớp tệp gốc) | ghi nhận |
| Khóa lạ | M06 | nhận + `legacy`, hoặc chưa rõ | từ chối `V1` "trường lạ \"ghi_chu_ban_cu\"" | ghi nhận: không bỏ lặng lẽ |
| Tham chiếu ảnh thiếu (cũ) | M09 | từ chối `V3` | từ chối `V1` (cả dạng cũ bị từ chối trước) | khớp ở mức nhận/từ chối, khác lớp chi tiết |
| Phiên bản mới hơn | N01 (3), N02 (99) | `NEWER` | `NewerSchema(3)`, `NewerSchema(99)` | KHỚP |
| Phiên bản | N03 (`schema_version` 1 nhưng có trường mới) | từ chối (A-V1) | nhận | **SAI LỆCH theo A-V1** (mục 5.3) |
| Phiên bản | N04 (2 + khóa cũ), N05 (0) | từ chối | từ chối `V1` | KHỚP |
| Hợp lệ v2 | G01 (ảnh + PDF, ba `kind`, nam/nữ/null), G02 (đúng 6 tệp), G03 (thiếu `gioi_tinh`), G04 (PDF 10 485 760 B), G05 (ảnh 2 097 152 B) | nhận | nhận, băm trạng thái khớp | KHỚP (5/5) |
| PDF sai đầu | X01 `%PDX-`, X01b BOM trước `%PDF-`, X01c `%pdf-` | từ chối | `ATTACHMENT` "PDF không hợp lệ" | KHỚP |
| Cỡ | X02 PDF 10 485 761 B | từ chối | `Backup(MalformedPlaintext)` (giới hạn khối 10 MiB của vùng chứa) | KHỚP |
| Cỡ | X03 ảnh 2 097 153 B | từ chối | `V1` "ngoài khoảng 1–2097152" | KHỚP (A-L1 đúng: 2 MB = 2 097 152) |
| Trùng id | X04 (cùng hồ sơ), X04b (hai hồ sơ), X05 (hai bản ghi `Attachment` cùng `id`) | từ chối | `V1` "id trùng" / "tệp trùng" | KHỚP |
| Tham chiếu | X06 (`file_id` không tồn tại) | `V3` | `V3` "tệp đính kèm không tồn tại" | KHỚP |
| Thiếu/thừa byte | X07 (thiếu), X19 (khối thừa) | từ chối | `ATTACHMENT` "thiếu tệp đính kèm" / `PHOTO` "ảnh thừa" | KHỚP |
| Số lượng | X08 (7 tệp) | từ chối | `V1` "quá 6 tệp" | KHỚP |
| Enum | X09 `kind` sai, X10 `image/png` | từ chối | `V1` | KHỚP |
| Toàn vẹn | X11 `sha256` sai, X12 `bytes` sai | từ chối | `ATTACHMENT` "sha256/bytes không khớp" | KHỚP |
| `stripped` | X13 PDF `true`, X14 ảnh `false` | từ chối | `V1` | KHỚP |
| `gioi_tinh` | X15–X17b | từ chối | `V1` | KHỚP (4/4) |
| `file_name` | X18 (121 điểm mã), X23 (ký tự điều khiển) | từ chối | `V1` | KHỚP |
| Kiểu nội dung | X20 (`application/pdf`, byte JPEG) | từ chối | `ATTACHMENT` "PDF không hợp lệ" | KHỚP |
| Kiểu nội dung | **X21** (`image/jpeg`, byte là PDF) | từ chối | **nhận** | **SAI LỆCH** (mục 5.2) |
| Khóa cũ trong dạng mới | X22 | từ chối | `V1` "trường lạ certificate_photo_ids" | KHỚP |
| Chưa rõ | U01 (hai hồ sơ chung một tệp), U02 (tệp không ai trỏ tới) | chưa rõ (A-U1) | nhận | ghi nhận |

Cột "Lõi" là lỗi do lõi in ra, nguyên văn. Mọi tệp bị từ chối đều không làm đổi kho (chương trình dừng trước khi ghi).

## 5. Danh sách sai lệch

1. **Không có chuyển đổi cho dữ liệu dạng cũ (M01, M02, M03, M05).** Ở lượt không chuyển dạng, tệp v1 có
   `death.notice.certificate_photo_ids` bị từ chối toàn bộ. Lỗi là `V1` "thiếu trường \"attachments\"" tại
   `/persons/0/death/notice`. Vectơ dạng cũ duy nhất trong kho (`supplement-yc05/notice-v1.*`) đã bị **ghi đè** sang
   dạng mới trong PR. Vì vậy không còn bộ mẫu đóng băng "trước thay đổi lược đồ" mà R14 (c) cần; bộ cũ chỉ còn trong
   lịch sử git ở `main`. Đây là lý do chính của KHÔNG ĐẠT. Cách sửa thuộc bên xây dựng. Phép thử lại có sẵn:
   M01–M05 kèm kỳ vọng chuyển đổi.
2. **X21: ảnh đính kèm khai `image/jpeg` nhưng byte là PDF vẫn được lõi nhận.** Kỳ vọng đăng ký trước (bộ kiểm tra
   kiểm đầu tệp cho cả ảnh) là từ chối.
   - Lời giải thích có thể có: 01 §3.8a trên PR giao việc giải mã và mã hóa lại ảnh cho nền tảng, còn 03 §2.3 nói ảnh
     nhập từ sao lưu được giải mã và mã hóa lại. Nếu mọi nền tảng làm đúng như vậy trước khi ghi, byte PDF sẽ bị chặn ở
     đó.
   - Ở mức lõi, sai lệch vẫn được ghi. Bên kiểm định không kiểm được bước của nền tảng (ngoài phạm vi).
3. **N03 và việc không tăng `schema_version`.** 01 §7 yêu cầu đổi trường thì tăng `schema_version`. PR giữ 1 và ghi
   rõ đây là "ngoại lệ có chủ ý" vì v0.2 chưa phát hành.
   - Hệ quả có thể kiểm: tệp `schema_version` 1 có thể ở hai dạng. Lõi không có cách nào phân biệt hai dạng để chuyển
     đổi, ngoài cách nhìn hình dạng tài liệu.
   - Tệp v2 của bên kiểm định bị coi là "mới hơn" (`NewerSchema(2)`). Nếu một bản sau dùng 2 cho lần đổi tiếp theo,
     tệp do bộ tham chiếu này sinh ra sẽ mang nghĩa khác.
   - Quyết định thuộc chủ dự án. Sai lệch này không làm mất dữ liệu.
4. **M09: khác lớp lỗi chi tiết.** Kỳ vọng `V3`, thực tế `V1`, vì cả dạng cũ đều bị từ chối trước khi xét tham
   chiếu. Ở mức nhận/từ chối thì khớp. Lớp chi tiết sẽ kiểm được khi có chuyển đổi.

Ghi nhận (không tính sai lệch, đã đăng ký trước là chưa rõ):
- M06: khóa lạ làm cả tệp bị từ chối, không có `legacy`.
- M08: chuỗi NFD trong tệp nhập được giữ nguyên NFD. 01 §1 ("chuẩn hóa NFC trước khi lưu") và "không áp dụng khi
  nhập" chưa được hòa giải.
- U01, U02: được nhận.
- Lược đồ của PR đòi `file_name` khác rỗng (`minLength 1`, `\S`). Như vậy là chặt hơn 01 §3.8a ("chuỗi ≤ 120 \|
  null"), và không có tệp thử nào cho ca này.
- PR thêm thứ tự gợi ý theo giới tính ("từ hợp giới tính đứng trước"). 01 §3.1 không có quy tắc này.
- PR đặt viết hoa ở bước lưu từ giao diện (FFI), không đặt ở lớp ghi thấp. 01 §3.1 trên `main` ghi "lớp lưu trữ chuẩn
  hóa… khi lưu". Hai câu cần thống nhất.

## 6. Giả định đã đăng ký trước (tóm tắt; nguyên văn trong `oracle/v02/b5/b5_ref.py`, `GIA_DINH`)

| Mã | Nội dung | Sau đối chiếu |
|---|---|---|
| A-V1 | Đổi lược đồ thì tăng `schema_version` 1 → 2 | PR không tăng (mục 5.3) |
| A-L1 | 2 MB = 2 097 152 B; 10 MB = 10 485 760 B; đúng biên là hợp lệ | Khớp |
| A-R1, A-C1 | `attachments` là mảng gốc; byte đi chung danh sách khối với ảnh | Khớp (PR: ảnh trước, tệp đính kèm sau) |
| A-M1…A-M4 | Mỗi id ảnh giấy chứng tử thành một phần tử `{kind: giay_chung_tu}`, giữ thứ tự. Id tự do nhưng phải là UUID v4 duy nhất. `Photo` chuyển thành `Attachment`, ảnh chân dung chung vẫn còn | Chưa kiểm được (chưa có chuyển đổi) |
| A-M5 | `gioi_tinh` thiếu = `null` | Khớp; lõi ghi bằng cách bỏ khóa |
| A-M6, A-N2 | Chuyển đổi và nhập không viết hoa | Khớp ở phía nhập (M07) |
| A-M7 | Khóa lạ vào `legacy`; từ chối cả tệp → chưa rõ | Lõi từ chối (M06) |
| A-N1 | Viết hoa đúng điểm mã đầu sau khi cắt khoảng trắng và NFC | Chưa kiểm được |
| A-U1 | Tệp dùng chung, tệp không ai trỏ tới → chưa rõ | Lõi nhận |

## 7. Chạy lại khi PR #28 có chuyển đổi

```
python3 -I oracle/v02/b5/b5_fixtures.py --big <nháp>/b5big        # sinh lại, so sha256 với fixtures/manifest.json
python3 -I oracle/v02/b5/b5_adapt.py <nháp>/adapted <nháp>/b5big  # chỉ khi PR vẫn giữ schema_version 1
git archive <head> | tar -x -C <nháp>/build
(cd <nháp>/build && cargo build --release --locked -p lvhv-store --examples)
python3 -I oracle/v02/b5/b5_compare.py <nháp>/build <nháp>/adapted <head>-adapted
python3 -I oracle/v02/b5/b5_compare.py <nháp>/build oracle/v02/b5/fixtures <head>-raw
```

Với M01–M05, `b5_compare.py` thử cả băm của kỳ vọng chuyển đổi (A-M1 với id giữ nguyên) lẫn băm của tệp gốc. Nếu bên
xây dựng sinh id mới cho phần tử `attachments`, băm sẽ khác dù dữ liệu không mất. Khi đó cần một công cụ hộp đen in
được kho đã chuyển đổi, để so từng trường không phụ thuộc id (`b5_ref.norm_for_compare`).

## 8. Ngoài phạm vi và chưa làm

- Nâng cấp tại chỗ trên từng nền tảng (bản web thay service worker; `adb install -r`; MSI; Apple), ca chuyển đổi lỗi
  có bản sao dữ liệu cũ, lịch nhắc đặt lại sau nâng cấp: do các job CI nền tảng chạy.
- Chuyển đổi kho SQLite tại chỗ ở mức lõi (01 §7: mở kho `schema_version` thấp hơn): không có công cụ hộp đen, và PR
  không đổi `schema_version` nên chưa có đường chuyển đổi để kiểm.
- Viết hoa khi lưu, danh sách gợi ý: chưa có đường hộp đen (mục 1).
- Bản TypeScript/PWA (`pwa/src/store`): không chạy trong lượt này.

## 9. Tệp

- Tham chiếu: `oracle/v02/b5/b5_ref.py`, `lvhvfile.py` (đọc/ghi `.lvhv` độc lập), `b5_statehash.py`.
- Tệp thử và kỳ vọng: `oracle/v02/b5/b5_fixtures.py`, `oracle/v02/b5/fixtures/`, `oracle/v02/out/b5_expected.json`,
  `oracle/v02/out/b5_expected/`.
- Đối chiếu: `oracle/v02/b5/b5_adapt.py`, `b5_compare.py`, `b5_cmp_docs.py`, `b5_probe_check.sh`;
  `oracle/v02/out/b5_cmp_5535a49-adapted.json`, `b5_cmp_5535a49-raw.json`, `b5_cmp_docs_5535a49.json`,
  `b5_probe_raw_5535a49.tsv`.
