# Báo cáo kiểm định độc lập v0.2 — Bổ sung B5 / R14 (mức lõi) và quy tắc dữ liệu YC-2026-07

Ngày lập: 2026-11-07. Người lập: bên không tham gia xây dựng đặc tả (kiểm định độc lập, phòng sạch).
Nhánh: `oracle-v02-b5` (từ `origin/main` 6ee647b). Đối tượng kiểm: PR #28, nhánh `v0.2-w3-core2`, chưa gộp.
- Lần 1: head `5535a49`. Có tệp đính kèm, **chưa** có chuyển đổi.
- Lần 2: head `8a2ed41`. Có chuyển đổi 1→2 và bộ mẫu đóng băng.
- **Lần 3 (kết luận hiện hành, mục 0):** `main` 43c3639 (PR #28 đã gộp, gồm B6) cộng PR #36 `9106ae9`, có công cụ
  hộp đen `lvhv-store-tool` (bản Rust và bản TS, theo `docs/v0.2/10`). Mục 1–9 giữ nguyên kết quả lần 1 và lần 2.

## 0. Lần 3 — `main` 43c3639 + PR #36 `9106ae9` (kết luận hiện hành)

Ở lần 3 chỉ đọc thêm tài liệu: `docs/v0.2/10`, 04 Bổ sung B6 và B7. Không đọc mã của công cụ hay của lõi. Mọi phép
thử chạy trên cả hai bản, Rust (`target/release/lvhv-store-tool`) và TS (`node pwa/test/tools/lvhv-store-tool.mjs`).
Hai bản cho cùng kết quả ở **mọi** ca. Số liệu: `oracle/v02/out/b5_r3_9106ae9.json` và `.log`.

### 0.1 Kết luận

| Phần | Kết luận | Số liệu |
|---|---|---|
| **R14 mức lõi** (Rust + TS) | **ĐẠT**, kèm hai khiếm khuyết đã biết (mục 0.3). Hai khiếm khuyết này không vi phạm ngưỡng | Xem các dòng dưới |
| Chuyển kho SQLite tại chỗ (Rust) / IndexedDB (TS) | ĐẠT | 9 kho phiên bản 1 hợp lệ (bộ mẫu đóng băng + M01–M05, M07, M08, M10): 0 bản ghi mất, 0 trường khác so với chuyển đổi tham chiếu, byte ảnh và tệp đính kèm trùng. Kho Rust đạt `user_version` 2; TS `state: migrated`. Hai kho thuộc ca phải lỗi đi đúng đường lỗi: M06 (khóa lạ) và M09 (id ảnh treo) → `read_only`, `user_version` vẫn 1, mọi hàng của CSDL không đổi, `.pre-v2.bak` có mặt |
| Ca lỗi (`--force-migration-failure`) | ĐẠT | 11/11 kho, Rust và TS: `state = read_only`; `user_version` vẫn 1; các hàng CSDL giống hệt trước khi mở; ảnh không đổi; `.pre-v2.bak` có mặt. `put_settings`, `put_person`, `delete_person` lỗi `ReadOnly`; `export_backup` thành công. TS: tài liệu kho = dữ liệu cũ |
| Nhập tệp sao lưu cũ | ĐẠT | M01–M05, M07: 0 bản ghi mất, 0 trường khác (`store.document` so trực tiếp, không còn qua băm). `example-v1.lvhv` nhập ra `state_sha256` `177b73e2…`, đúng băm do bên kiểm định tự tính ở lần 1 |
| Từ chối phiên bản mới hơn | ĐẠT | `schema_version` 3 và 99 → `NewerSchema` (cả hai bản) |
| Vectơ R6 đã khóa | ĐẠT | Dựng 9106ae9, sinh lại bằng `gen_test_vectors`: 28/28 tệp giống từng byte bản trên `main` 6ee647b. Theo B7, bộ R6v2 chưa có trong kho, nên chưa kiểm |
| **YC-2026-07 — tệp đính kèm** | **ĐẠT** | 22/22 tệp không hợp lệ bị từ chối, kể cả X21 (`mime_mismatch`) và X22 ("có cả attachments lẫn certificate_photo_ids — tài liệu dị dạng"). 5/5 tệp hợp lệ, gồm cả ca đúng biên, nhập ra 0 trường khác. `check-media` (Rust và TS): PDF dán nhãn `image/jpeg` → `mime_mismatch`; PDF đúng nhãn → `ok` |
| YC-2026-07 — `gioi_tinh` | ĐẠT | 4/4 giá trị sai bị từ chối; thiếu khóa và `null` được nhận |
| YC-2026-07 — không viết hoa khi nhập | ĐẠT | M07 giữ nguyên từng byte |
| **YC-2026-07 — viết hoa khi lưu và khi hiển thị** | **KHÔNG ĐẠT (2/28 ca)** | `save-person` và `display-capitalize` khớp kỳ vọng ở 26/28 ca, kể cả mọi ca NFD và mọi ca chưa rõ theo A-N1 (`"(bên nội) ông"`, `"2 bà"`, `"«bà»"`, dấu kết hợp đứng đầu: giữ nguyên). 2 ca sai: `"  chú  "` lưu thành `"  chú  "` (kỳ vọng `"Chú"`), và `"   "` lưu thành `"   "` (kỳ vọng `""`). Khoảng trắng hai đầu không bị cắt (trái 01 §1), nên chữ đầu không được viết hoa. `ho_ten` không bao giờ bị viết hoa (28/28) |
| **YC-2026-07 — danh sách gợi ý** | **ĐẠT** | Không truyền giới tính: người mất đúng 22 mục, người sống đúng 17 mục, **đúng thứ tự** của 01 §3.1. Có `nam`/`nu`: cùng tập mục, thứ tự đổi (từ hợp giới tính lên trước). Quy tắc thứ tự này là phần thêm của PR, 01 §3.1 không quy định |
| M10, U01 (ảnh/tệp dùng chung) | ghi nhận: không mất | M10: chuyển đổi tại chỗ và nhập đều cho **một** tệp dùng chung, hai hồ sơ cùng trỏ tới, 0 trường khác. U01 được nhận |

### 0.2 Kiểm lại X21, X22, M09 và ảnh dùng chung

| Ca | Lần 2 (`8a2ed41`) | Lần 3 |
|---|---|---|
| X21 (khai `image/jpeg`, byte PDF) | nhận nhầm | từ chối, `Invalid … mime_mismatch` (Rust, TS) — **đã sửa** |
| X22 (trộn hai dạng) | nhận, không rõ đã ghi gì | từ chối: "có cả attachments lẫn certificate_photo_ids (khóa cũ) — tài liệu dị dạng" — **đã sửa** |
| M09 (id ảnh giấy chứng tử treo) | nhận, giữ id trong `legacy` | nhập: từ chối `V3` "ảnh không tồn tại". Chuyển tại chỗ: `read_only`, dữ liệu cũ nguyên vẹn — **khớp kỳ vọng đăng ký trước** |
| Ảnh dùng chung (M03, M10, bộ mẫu đóng băng) | M03 khớp băm | 0 trường khác ở cả nhập lẫn chuyển tại chỗ. Ảnh chân dung còn nguyên; ảnh dùng cho hai hồ sơ → một tệp dùng chung |

### 0.3 Quan sát về hai khiếm khuyết bên xây dựng đang sửa (không chờ bản sửa)

1. **Tệp đính kèm mồ côi khi chuyển đổi lỗi.** Ở ca buộc lỗi, thư mục `attachments/` còn tệp ở 6/11 kho: bộ mẫu
   đóng băng (2 tệp), M01 (3), M02 (4), M03 (1), M06 (3) và M10 (2). Ca lỗi thật M06 (không buộc) cũng để lại 3 tệp.
   - Các tệp này là bản chép của ảnh giấy chứng tử. CSDL đã quay lui, không trỏ tới chúng.
   - Không mất dữ liệu. Nhưng thư mục đã không còn "như trước khi mở", và có thêm bản chép của dữ liệu nhạy cảm (R-PRIV-2).
   - Các kho không có ảnh giấy chứng tử (M04, M05, M07, M08) và M09 không có tệp mồ côi.
2. **`.pre-v2.bak` không giống từng byte tệp CSDL gốc** ở 11/11 kho, cả ca thường lẫn ca lỗi. Bên kiểm định đọc bản sao ở chế độ chỉ đọc: mọi
   bảng và mọi hàng, cùng `user_version` 1, giống hệt bản gốc ở 11/11 kho. Như vậy không mất dữ liệu, chỉ khác byte.
   Ghi chú phương pháp: việc đọc bản sao bằng `sqlite3` ở chế độ chỉ đọc tạo thêm `-shm`/`-wal` cạnh `.bak`. Hai tệp đó
   do bên kiểm định sinh ra, không phải do lõi.

### 0.4 Sai lệch và ghi nhận của lần 3

- **Viết hoa khi lưu/hiển thị — 2 ca khoảng trắng** (mục 0.1). Hai bản giống nhau.
- **N03 / N04 (đánh số phiên bản):** giữ nguyên như mục 5.4 của lần 2. Lượt không chuyển dạng: N04 (`schema_version`
  2 + dạng cũ) → `NewerSchema(2)`. Theo cách đánh số của PR (tài liệu sao lưu vẫn là 1) thì kết quả này đúng; nó chỉ
  khác lớp so với A-V1. Không tính là lỗi.
- M06: khóa lạ vẫn bị từ chối, không có `legacy`. Theo A-M7: CHƯA RÕ, không bỏ lặng lẽ.
- M08: NFD vẫn được giữ nguyên khi nhập; khi chuyển tại chỗ cũng vậy.
- **Lỗi của chính tệp thử, đã sửa trước khi chạy lần 3 (commit `b9d8b07`).** Bản `d14f452` ghi kích thước ảnh cố
  định 64×48, trong khi ảnh thật có kích thước khác. Chỗ sai lộ ra khi đọc 04 B6, vì kiểm tra kích thước mới bật. Bản
  sửa chỉ đổi `width`/`height` trong các tệp thử M01, M02, M06, N04 và thêm M10. Không kỳ vọng cũ nào đổi:
  `b5_expected.json` chỉ có thêm mục M10.

### 0.5 Trình tự lần 3

| Giờ (+07) | Commit | Việc |
|---|---|---|
| 13:2x | — | Đọc `docs/v0.2/10`, 04 B6/B7. Dựng `lvhv-store-tool` và các ví dụ của `lvhv-store` từ `9106ae9`; chạy `npm ci` cho TS; chạy `--help` |
| 13:27 | `b9d8b07` | Sửa kích thước ảnh trong tệp thử, thêm M10. Commit **trước** khi chạy công cụ trên tệp thử |
| 13:30–13:40 | — | `b5_mkstore.py` dựng kho SQLite phiên bản 1 từ tệp thử. Bố cục bảng lấy từ CSDL của bộ mẫu đóng băng, đọc như dữ liệu. `b5_round3.py` chạy toàn bộ |
| 13:41 | `936d260` | Kết quả lần 3 |

### 0.6 Ngoài phạm vi / chưa làm (lần 3)

- Nâng cấp tại chỗ trên từng nền tảng và lịch nhắc đặt lại sau nâng cấp: do các job CI nền tảng chạy.
- R6 trên R6v2 (B7): bộ R6v2 chưa có trong kho.
- Dựng lại bộ mẫu đóng băng từ commit 7338f04: chưa làm.

Căn cứ đặc tả, đọc trên `main` 6ee647b **trước** khi viết kỳ vọng:
- `docs/v0.2/01` §1, §3.1, §3.2a, §3.8, §3.8a, §6, §7;
- `03` §2 và Bổ sung B1;
- `04` R6 và Bổ sung B5 / R14;
- `07` (YC-2026-07, YC-2026-08).

Tài liệu của PR #28 (01 §3.8a "Cách cài", 01 §7.1, 03 §2.3, lược đồ, `test-vectors/released/README.md`) chỉ được đọc
**sau** commit kỳ vọng `d14f452`.

Không đọc mã nguồn của `memorial/`, `store/`, `backup/`, `core/`, `tsport/`, `ffi/`, `pwa/`, `apps/`. Tệp
`make_dev_fixture.rs.txt` của bộ mẫu là mã, nên cũng không đọc; chỉ kiểm sha256 của nó. Các tệp dưới `test-vectors/`
chỉ được đọc như dữ liệu. Công cụ của bên xây dựng chỉ được dựng và chạy như hộp đen.

## 1. Kết luận lần 2 (`8a2ed41`) — đã được lần 3 thay thế

| Phần | Kết luận | Số liệu |
|---|---|---|
| **R14 mức lõi — toàn phần** | **CHƯA ĐỦ BẰNG CHỨNG** | Đường **nhập tệp sao lưu cũ** ĐẠT (hai dòng dưới). Đường **chuyển kho SQLite tại chỗ** (`user_version` 1 → 2, bản sao `.pre-v2.bak`, ca lỗi → chỉ đọc) không có công cụ hộp đen nào gọi được, nên bên kiểm định chưa chạy được |
| R14 lõi — nhập tệp cũ có ảnh giấy chứng tử (`certificate_photo_ids` → `attachments`, `gioi_tinh` thiếu → `null`) | **ĐẠT** | 5/5 tệp cũ hợp lệ (M01–M05) được nhận. Băm trạng thái kho sau khi nhập **trùng từng byte** với kết quả của hàm chuyển đổi tham chiếu, cộng phần `Person.legacy` mà PR mô tả ở 01 §7.1 (mục 3). Tức là 0 bản ghi mất và 0 trường khác. Có M03, ca ảnh giấy chứng tử trùng ảnh chân dung: ảnh chân dung vẫn còn, tệp đính kèm lấy id thay thế |
| R14 lõi — bộ mẫu đóng băng `released/0.2-dev-before-attachments` | **ĐẠT** | 6/6 tệp khớp sha256 trong `manifest.json`; kho SQLite có `user_version` = 1. Chạy chuyển đổi **tham chiếu** trên `backup.lvhv` của bộ mẫu, so với `released/expected/…json` của bên xây dựng: **0 trường khác** (so không phụ thuộc id, tách `legacy`); 0 chuỗi nào của tệp cũ bị mất |
| R14 lõi — tệp cũ không có thông tin báo tử | ĐẠT | M04 (chính `example-v1`) và M07: băm trùng tệp gốc |
| R14 lõi — từ chối phiên bản mới hơn | **ĐẠT** | `schema_version` 3 và 99 → `NewerSchema(3)`, `NewerSchema(99)`. `file_version` = 2 có trong vectơ R6 đã khóa |
| R14 lõi — vectơ R6 đã khóa | **ĐẠT** | Dựng `8a2ed41`, sinh lại bằng `gen_test_vectors`: **28/28** tệp của bộ R6 giống từng byte bản trên `main` (`example-v1.*`, `dataset-r6.json`, `manifest.json`, `merge-cases.json`, `validation-cases.json`, ảnh mẫu, 21 tệp `corrupt/`) |
| R14 lõi — khóa lạ giữ trong `legacy` | CHƯA RÕ (đăng ký trước, A-M7) | M06 bị từ chối cả tệp (`V1` "trường lạ"). Không bỏ lặng lẽ, nhưng cũng không giữ |
| R14 — nâng cấp tại chỗ trên từng nền tảng | NGOÀI PHẠM VI | Do các job CI nền tảng chạy |
| **YC-2026-07 — `gioi_tinh`** | **ĐẠT** | 4/4 giá trị sai bị từ chối `V1`; thiếu khóa và `null` được nhận. Kho ghi `null` bằng cách bỏ khóa |
| **YC-2026-07 — tệp đính kèm** | **KHÔNG ĐẠT** | 21/22 tệp v2 không hợp lệ bị từ chối đúng lớp. 5/5 tệp hợp lệ (PDF đúng 10 485 760 B, ảnh đúng 2 097 152 B, đúng 6 tệp) được nhận, băm trùng. **X21** (khai `image/jpeg`, byte là PDF) bị **nhận nhầm** (mục 5.1). Cùng với X22 và M09 (mục 5.2, 5.3) |
| YC-2026-07 — không viết hoa khi **nhập** | **ĐẠT** | M07: `"ông"`, `"bà cô tổ họ"`, `"BÀ cố"`, `"cụ"` (người sống), `"lê thị c"` giữ nguyên từng byte |
| YC-2026-07 — viết hoa khi **lưu** / khi hiển thị | CHƯA ĐỦ BẰNG CHỨNG | `save_person_json` và `display_capitalize` chỉ có qua FFI, không có công cụ hộp đen. 28 ca kỳ vọng đã khóa ở `oracle/v02/out/b5_expected.json` (`names_on_save`) |
| YC-2026-07 — danh sách gợi ý (người mất 22 / người sống 17) | CHƯA ĐỦ BẰNG CHỨNG | Không có đường hộp đen nào lộ danh sách. Kỳ vọng ở `b5_expected.json` (`goi_y`). Ở mức tài liệu, danh sách trong 01 §3.1 của PR giống hệt `main` |

R14 mức lõi chưa có KHÔNG ĐẠT nào: mọi phép thử chạy được trên đường nhập sao lưu đều đạt. Chỉ còn đường chuyển kho
SQLite tại chỗ chưa có bằng chứng hộp đen. YC-2026-07 có một sai lệch thật (X21) và hai chỗ hành vi khác kỳ vọng đã
đăng ký (mục 5).

## 2. Trình tự làm việc (để kiểm được tính đăng ký trước)

| Giờ (+07) | Commit (`oracle-v02-b5`) | Việc |
|---|---|---|
| 11:02 | — | Đọc đặc tả trên `main` 6ee647b. Kiểm bộ đọc/ghi `.lvhv` độc lập (`lvhvfile.py`): giải mã `example-v1.lvhv` và `supplement-yc05/notice-v1.lvhv`, mã hóa lại với cùng salt/nonce ra giống từng byte |
| 11:11 | **`d14f452`** | Tham chiếu (`b5_ref.py`: chuyển đổi 1→2, bộ kiểm tra, quy tắc viết hoa, danh sách gợi ý, 15 giả định A-\*), 48 tệp thử (`b5_fixtures.py`, `fixtures/`) và kết quả kỳ vọng (`out/b5_expected.json`, `out/b5_expected/`). Đẩy lên **trước** khi đọc tài liệu PR #28 và trước khi chạy công cụ của nó |
| 11:12–11:21 | **`7f38eb3`** | Lần 1 (`5535a49`). Bộ chuyển dạng sau đối chiếu (`b5_adapt.py`), băm trạng thái độc lập (`b5_statehash.py`), so sánh (`b5_compare.py`, `b5_cmp_docs.py`) |
| 11:2x | `30a79f7` | Báo cáo lần 1. Mở PR #33 |
| 11:31–11:34 | **`8ac9824`** | Lần 2 (`8a2ed41`). Đối chiếu bộ mẫu đóng băng (`b5_cmp_released.py`). Thêm vào `b5_compare.py` ứng viên "biểu diễn của PR" (`builder_repr`), viết **sau** khi đọc 01 §7.1 của PR |

Kỳ vọng (`b5_expected.json`, `b5_expected/*.json`) **không bị sửa** sau `d14f452`. Bốn tệp lớn (G04, G05, X02, X03;
10–11 MB) được sinh tất định lúc chạy (`b5_fixtures.py --big DIR`). `sha256` của chúng ghi trong `fixtures/manifest.json`.

## 3. Phương pháp hộp đen

- **Đường vào.** `lvhv-store` không có công cụ dòng lệnh nào nhận một tệp sao lưu tùy ý, ở cả hai head. Hai ví dụ
  có sẵn là `cross_platform` ("dùng: cross_platform write|check") và `r2_full`.
  - `check` báo thiếu `pwa/test/store/out/r6-from-pwa.lvhv`. Đường dẫn này lấy từ thông báo lỗi của chính chương
    trình.
  - Bên kiểm định đặt từng tệp thử vào chỗ đó rồi chạy `check`. Chương trình mở tệp bằng mật khẩu mẫu của 03 §2.5,
    nhập vào kho SQLite trống, rồi so với bộ R6.
  - Tệp bị từ chối: chương trình in lỗi kèm mã lớp và đường dẫn (`Invalid(ValidationError { code, path, msg })`,
    `NewerSchema(n)`, `Backup(MalformedPlaintext)`).
  - Tệp được nhận: chương trình in `left:` là băm trạng thái kho sau khi nhập.
- **Băm trạng thái độc lập.** `b5_statehash.py` cài lại theo 03 B1.4 mục 10. Đặc tả trên `main` chưa nói phần tệp
  đính kèm được băm thế nào. Biến thể "sha256 ảnh theo `id`, rồi sha256 tệp đính kèm theo `id`" được chọn vì tái tạo
  đúng `state_sha256` công bố của `supplement-yc05` (`ac8a9dce…`) và đúng `left` của G01 (`c369a8e3…`). Hai băm bằng
  nhau nghĩa là JSON chuẩn toàn kho và byte của mọi ảnh/tệp đều giống, tức 0 bản ghi mất và 0 trường khác.
- **Chuyển dạng sau đối chiếu** (`b5_adapt.py`, không sửa kỳ vọng). PR giữ `schema_version` = 1 cho tài liệu sao lưu.
  Phiên bản kho được tăng riêng (`STORE_VERSION` = 2; IndexedDB 3), theo 01 §7.1 của PR. Vì vậy:
  - tệp v2 của bên kiểm định được hạ `schema_version` về 1 và bỏ `attachments` rỗng;
  - khối nhị phân xếp ảnh trước, tệp đính kèm sau (khớp giả định A-C1).
  - Lượt không chuyển dạng: `out/b5_cmp_8a2ed41-raw.json`. Ở lượt đó, mọi tệp v2 bị `NewerSchema(2)`, đúng như cách
    PR đánh số phiên bản.
- **Ứng viên biểu diễn của PR cho tệp cũ** (`builder_repr`). Ứng viên = kết quả `migrate_1_to_2` tham chiếu (không
  đổi), cộng các phần mà 01 §7.1 của PR quy định thêm:
  - `Person.legacy = {certificate_photo_ids, certificate_photos}`;
  - ảnh chung với ảnh chân dung → tệp đính kèm mang id thay thế (`alt_id`). Giá trị đọc từ
    `released/expected/…json` của PR cho đúng ảnh `…5e09`;
  - `schema_version` giữ 1.
  - Các phần này chỉ **thêm** (`legacy`) hoặc đổi **id**, mà A-M1/A-M7 đã đăng ký là được phép. Không trường dữ liệu
    nào của kỳ vọng bị sửa.

## 4. Kết quả từng tệp thử (lần 2, `out/b5_cmp_8a2ed41-adapted.json` và `-raw.json`)

Lượt đã chuyển dạng: 39 KHỚP, 5 "nhận tệp phải từ chối", 4 ghi nhận. Trong 5 ca "nhận nhầm", N04 là sản phẩm của bộ
chuyển dạng (mục 5.5), không tính là sai lệch.

| Nhóm | Tệp | Kỳ vọng | Lõi `8a2ed41` | Đánh giá |
|---|---|---|---|---|
| Tệp cũ có ảnh giấy chứng tử | M01 (3 ảnh, 2 hồ sơ), M02 (4 ảnh), M03 (trùng ảnh chân dung) | nhận, chuyển | nhận, băm trùng ứng viên `legacy` | KHỚP |
| | M05 (`notice` rỗng, danh sách ảnh rỗng) | nhận, chuyển | nhận, băm trùng ứng viên có `legacy` cả khi danh sách rỗng | KHỚP |
| Tệp cũ không có thông tin báo tử | M04, M07 | nhận, giữ nguyên | nhận, băm trùng tệp gốc | KHỚP |
| NFD | M08 | chưa rõ (A-N2) | nhận, giữ nguyên NFD | ghi nhận |
| Khóa lạ | M06 | `legacy` hoặc chưa rõ | từ chối `V1` "trường lạ" | ghi nhận |
| Tệp cũ có id ảnh không tồn tại | **M09** | từ chối `V3` | **nhận**; id treo được giữ trong `legacy.certificate_photo_ids`, không có tệp đính kèm (băm trùng ứng viên đó) | **SAI LỆCH so với đăng ký trước** (mục 5.3) |
| Phiên bản mới hơn | N01 (3), N02 (99) | `NEWER` | `NewerSchema` | KHỚP |
| Phiên bản | N03 (1 + trường mới) | từ chối (A-V1) | nhận | khác đánh số phiên bản (mục 5.4) |
| | N05 (0) | từ chối | `V1` "phải là 1" | KHỚP |
| Hợp lệ v2 | G01–G05 (gồm đúng biên 10 MiB, 2 MiB, 6 tệp) | nhận | nhận, băm trùng | KHỚP 5/5 |
| PDF sai đầu | X01, X01b (BOM), X01c (`%pdf-`) | từ chối | `ATTACHMENT` "PDF không hợp lệ" | KHỚP |
| Cỡ | X02 (PDF 10 485 761 B), X03 (ảnh 2 097 153 B) | từ chối | `MalformedPlaintext` / `V1` "ngoài khoảng 1–2097152" | KHỚP |
| Trùng id | X04, X04b, X05 | từ chối | `V1` "id trùng" / "tệp trùng" | KHỚP |
| Tham chiếu, byte | X06 (`V3`), X07 (thiếu byte), X19 (khối thừa) | từ chối | `V3` / `ATTACHMENT` / `PHOTO` | KHỚP |
| Số lượng, enum, toàn vẹn, `stripped`, `file_name` | X08–X14, X18, X20, X23 | từ chối | `V1` / `ATTACHMENT` | KHỚP |
| `gioi_tinh` | X15–X17b | từ chối | `V1` | KHỚP 4/4 |
| Kiểu nội dung | **X21** (`image/jpeg`, byte PDF) | từ chối | **nhận** | **SAI LỆCH** (mục 5.1) |
| Trộn hai dạng | **X22** (có `attachments` và cả `certificate_photo_ids: []`) | từ chối | **nhận**; trạng thái sau nhập **không trùng** ứng viên nào trong 19 ứng viên đã thử | **SAI LỆCH** (mục 5.2) |
| Chưa rõ | U01 (hai hồ sơ dùng chung một tệp), U02 (tệp không ai trỏ tới) | chưa rõ (A-U1) | nhận | ghi nhận. Chuyển đổi của PR cũng cố ý tạo tệp dùng chung |

So với lần 1 (`5535a49`): M01, M02, M03, M05 chuyển từ "từ chối" sang KHỚP. M09 và X22 chuyển từ "từ chối" sang
"nhận". X21 không đổi.

## 5. Danh sách sai lệch (lần 2)

1. **X21: ảnh đính kèm khai `image/jpeg` nhưng byte là PDF vẫn được lõi nhận.** Kỳ vọng đăng ký trước là từ chối (bộ
   kiểm tra kiểm đầu tệp cả với ảnh).
   - Có thể bên xây dựng cho rằng không cần kiểm ở lõi: 01 §3.8a của PR giao việc giải mã và mã hóa lại ảnh cho nền
     tảng, và 03 §2.3 nói ảnh nhập từ sao lưu được giải mã và mã hóa lại.
   - Ở mức lõi, sai lệch vẫn được ghi. Bước của nền tảng nằm ngoài phạm vi.
2. **X22: tài liệu trộn hai dạng được nhận, không biết lõi đã ghi gì.**
   - Đầu vào: một hồ sơ vừa có `notice.attachments` (3 tệp) vừa có `certificate_photo_ids: []`.
   - Bên kiểm định đã thử 19 ứng viên trạng thái: giữ/bỏ `attachments`, có/không `legacy` ở hồ sơ hoặc ở `notice`, có
     tỉa tệp không ai dùng hay không. Không ứng viên nào trùng băm.
   - Vì vậy không loại được khả năng các tệp đính kèm sẵn có bị ghi đè. Không bản dựng nào sinh ra dạng này, nên tác
     động chỉ đến từ tệp do người khác cố ý tạo. Muốn biết chắc, cần một công cụ in được kho sau khi nhập.
3. **M09: tệp cũ có `certificate_photo_ids` trỏ tới ảnh không tồn tại được nhận.** Lõi giữ id treo trong `legacy`,
   không tạo tệp đính kèm (01 §7.1 của PR: "kể cả id không có ảnh"). Đăng ký trước là từ chối `V3`, vì tệp này đã vi
   phạm V3 ngay trong lược đồ cũ. Cách của PR không mất dữ liệu, nhưng nhận một tệp mà bản cũ lẽ ra đã từ chối. Chủ dự
   án cần quyết định cách nào đúng.
4. **Không tăng `schema_version` của tài liệu sao lưu (N03).** 01 §7 trên `main` yêu cầu tăng. PR giữ 1, tăng phiên
   bản kho riêng, và nêu lý do: để băm R6 đã khóa không đổi.
   - Hệ quả: hai dạng tài liệu cùng mang số 1, chỉ phân biệt được bằng hình dạng.
   - Tệp v2 của bên kiểm định bị coi là "mới hơn". Nếu bản sau dùng số 2, tệp v2 của tham chiếu này sẽ mang nghĩa khác.
   - Không mất dữ liệu. Quyết định thuộc chủ dự án.
5. **N04 (không tính).** Sau khi chuyển dạng, N04 chỉ còn là một tệp dạng cũ (`schema_version` 1 + `certificate_photo_ids`),
   giống M01, và được chuyển đổi đúng. Ở lượt không chuyển dạng, N04 bị `NewerSchema(2)`. Không phải sai lệch.

Bên kiểm định tự phát hiện một điểm không nhất quán trong tham chiếu của mình, ghi lại, không sửa sau đối chiếu:
- Bộ kiểm tra (`b5_ref.validate`) coi id phần tử đính kèm trùng **giữa hai hồ sơ** là `DUP` (X04b).
- Nhưng hàm chuyển đổi của chính tham chiếu đặt id phần tử = id ảnh. Nếu một ảnh giấy chứng tử dùng cho hai hồ sơ, như
  bộ mẫu đóng băng của PR, kết quả chuyển đổi sẽ bị chính bộ kiểm tra từ chối.
- Bộ mẫu của PR cho thấy lõi chấp nhận id trùng giữa hai hồ sơ.
- X04b vẫn bị lõi từ chối, nhưng vì lý do khác ("tệp trùng" trong **cùng** hồ sơ). Phép thử "trùng id giữa hai hồ sơ"
  nên được coi là CHƯA RÕ.

Ghi nhận khác (không tính sai lệch):
- M06: khóa lạ làm cả tệp bị từ chối, không có `legacy` cho khóa lạ.
- M08: NFD giữ nguyên khi nhập.
- Lược đồ PR đòi `file_name` khác rỗng. Như vậy là chặt hơn 01 §3.8a, và không có tệp thử nào cho ca này.
- PR thêm thứ tự gợi ý theo giới tính. 01 §3.1 không có quy tắc này.
- PR viết hoa cả khi hiển thị (`display_capitalize`). Đây là phần mới của 01 §3.1 trên PR; trên `main` không có.
- Nguồn gốc bộ mẫu đóng băng: tài liệu của PR ghi là do mã ở commit 7338f04 tạo ra. Bên kiểm định chỉ kiểm được sha256
  và `user_version` = 1, chưa dựng lại từ 7338f04.
- Lần 1 ghi là vectơ dạng cũ đã bị ghi đè. Ở lần 2, việc này đã được khắc phục: dạng cũ có trong `released/`.

## 6. Giả định đã đăng ký trước (nguyên văn trong `oracle/v02/b5/b5_ref.py`, `GIA_DINH`)

| Mã | Nội dung | Sau đối chiếu |
|---|---|---|
| A-V1 | Đổi lược đồ thì tăng `schema_version` 1 → 2 | PR giữ 1, tăng phiên bản kho riêng (mục 5.4) |
| A-L1 | 2 MB = 2 097 152 B, 10 MB = 10 485 760 B, đúng biên là hợp lệ | Khớp |
| A-R1, A-C1 | `attachments` là mảng gốc; byte đi chung danh sách khối với ảnh | Khớp |
| A-M1 | Mỗi id ảnh → một phần tử `giay_chung_tu`, giữ thứ tự; id tự do | Khớp. Id phần tử = id ảnh, như tham chiếu |
| A-M2 | `Photo` → `Attachment` cùng byte; `width`/`height` được phép bỏ | Khớp. PR còn giữ bản ghi `Photo` gốc trong `legacy` |
| A-M3, A-M4 | Ảnh chân dung chung vẫn còn; ảnh chỉ là giấy chứng tử thì chuyển hẳn | Khớp |
| A-M5 | `gioi_tinh` thiếu = `null` | Khớp |
| A-M6, A-N2 | Chuyển đổi/nhập không viết hoa | Khớp |
| A-M7 | Khóa lạ → `legacy`; từ chối cả tệp → chưa rõ | Lõi từ chối |
| A-N1 | Viết hoa điểm mã đầu sau khi cắt khoảng trắng và NFC | Chưa kiểm được |
| A-U1 | Tệp dùng chung, tệp không ai trỏ tới → chưa rõ | Lõi nhận; chuyển đổi của PR cố ý dùng chung |

## 7. Chạy lại

```
python3 -I oracle/v02/b5/b5_fixtures.py --big <nháp>/b5big
python3 -I oracle/v02/b5/b5_adapt.py <nháp>/adapted <nháp>/b5big
git archive <head> | tar -x -C <nháp>/build
(cd <nháp>/build && cargo build --release --locked -p lvhv-store --examples)
python3 -I oracle/v02/b5/b5_compare.py <nháp>/build <nháp>/adapted <head>-adapted
python3 -I oracle/v02/b5/b5_compare.py <nháp>/build oracle/v02/b5/fixtures <head>-raw
python3 -I oracle/v02/b5/b5_cmp_released.py <nháp>/build <head>
```

Để đóng các mục CHƯA ĐỦ BẰNG CHỨNG, cần một công cụ hộp đen của lõi (ví dụ một ví dụ `cargo` có tham số) làm được ba
việc:
- (i) mở một thư mục kho SQLite rồi in tài liệu kho và `migration_status()`;
- (ii) nhập một tệp sao lưu tùy ý rồi in tài liệu kho, không chỉ in băm;
- (iii) gọi `save_person_json`, `danh_xung_suggestions` và `quan_he_suggestions` với dữ liệu vào cho trước.

## 8. Ngoài phạm vi và chưa làm

- Nâng cấp tại chỗ trên từng nền tảng (bản web thay service worker; `adb install -r`; MSI; Apple), lịch nhắc đặt lại
  sau nâng cấp: do các job CI nền tảng chạy.
- Chuyển kho SQLite tại chỗ ở mức lõi, gồm bản sao `.pre-v2.bak`, ca lỗi → chỉ đọc, từ chối kho mới hơn: chưa có đường
  hộp đen (mục 7). Kiểm thử `store/tests/released.rs` của bên xây dựng không được đọc và không được tính là bằng chứng
  độc lập.
- Viết hoa khi lưu/hiển thị, danh sách gợi ý: chưa có đường hộp đen.
- Bản TypeScript/PWA: không chạy.
- Nguồn gốc bộ mẫu đóng băng (dựng lại từ 7338f04): chưa làm.

## 9. Tệp

- Tham chiếu: `oracle/v02/b5/b5_ref.py`, `lvhvfile.py`, `b5_statehash.py`.
- Tệp thử và kỳ vọng: `oracle/v02/b5/b5_fixtures.py`, `oracle/v02/b5/fixtures/`, `oracle/v02/out/b5_expected.json`,
  `oracle/v02/out/b5_expected/`.
- Đối chiếu: `oracle/v02/b5/b5_adapt.py`, `b5_compare.py`, `b5_cmp_docs.py`, `b5_cmp_released.py`,
  `b5_probe_check.sh`.
- Kết quả: `oracle/v02/out/b5_cmp_8a2ed41-{adapted,raw}.json`, `b5_cmp_released_8a2ed41.json`; lần 1:
  `b5_cmp_5535a49-{adapted,raw}.json`, `b5_cmp_docs_5535a49.json`, `b5_probe_raw_5535a49.tsv`.
