# Duyệt mã YC-2026-10 (T21) — PR #51 `v0.2-yc10-core`: `memorial.tray_photo_id` ở lõi

Trạng thái: **XONG** — 2026-11-07. Kết luận ở §5.

Bên duyệt mã: vai rà soát (T27, 07 §10.3); không viết mã của PR này và không sửa nhánh đang duyệt.

| PR | Nhánh | Đầu nhánh được duyệt | So với |
|---|---|---|---|
| #51 | `v0.2-yc10-core` | `eafa96e` | `git diff origin/main...eafa96e` (`main` = `95ca823`) |

Căn cứ đã khóa: 01 §3.7 (dòng `tray_photo_id`), §3.8 (đoạn "Ảnh không còn dùng"), §6.2 V3, §6.3 (Gộp), "Ghi chú
YC-2026-10"; 04 Bổ sung B6, B8 / R15, R14; 07 §10; 03 §2.5 (vector đã khóa); 10 (công cụ kiểm định).

Mức lỗi: **chặn** / **nên sửa** / **góp ý**. Mã lỗi: `MA10-xx`.

## 0. Đã chạy / đã đọc

| Việc | Kết quả |
|---|---|
| `cargo test --locked -p lvhv-store` và `-p lvhv-ffi` trên `eafa96e` | Mọi bài ĐẠT (gồm `store/tests/tray.rs` 4/4, `ffi` `notice_never_leaks`, `save_tray_photo_null_omitted`) |
| `cross_platform write` → `node pwa/test/store/run.mjs --quick` → `cross_platform check` | **ĐẠT**; năm dòng R15 mới đều `ok`; Rust mở tệp của PWA, `state_hash` khớp |
| `lvhv-store-tool scenario tray_photo` (Rust, bản debug) và bản TS | Rust = TS = `tray-photo/expected-scenario.json` |
| Sinh lại vector (`gen_test_vectors`, `gen_notice_vector`) | Không khác bản đã commit: vector 03 §2.5, `example-v1.*`, `released/` **không đổi** |
| `tsc --noEmit` của `pwa/` | Không có lỗi ở `src/store/`; các lỗi còn lại chỉ do thiếu `src/gen/content` (cần bản dựng), không liên quan PR |
| CI trên `eafa96e` | Rust fmt/clippy/test, Lưu trữ + sao lưu, L7, memorial Rust ≡ TS, cargo-deny, `ui`, `build`: `success`; `upgrade`, `screenshots` còn chạy lúc duyệt |
| Thử thêm (mã thử tạm ngoài nhánh, đã xóa) | Ba ca ở MA10-02, MA10-03, MA10-04, chạy trên **cả hai** bản; kết quả ghi ở từng mục |

Đã đọc toàn bộ diff (18 tệp): `store/src/{lib,model,names,validate}.rs`, `store/tests/tray.rs`, `ffi/src/tests_v02.rs`,
`ffi/src/bin/lvhv-store-tool.rs`, `pwa/src/store/{db,merge,names,validate}.ts`, `pwa/test/store/run.mjs`,
`pwa/test/tools/lvhv-store-tool.mjs`, `scripts/check-store{,-tool}.sh`, `backup-v1.schema.json`, tệp kỳ vọng, 10 §3.9–§3.11.

## 1. Đối chiếu với đặc tả

| Yêu cầu | Kết quả |
|---|---|
| 01 §3.7: UUID \| null, thiếu ≡ null, **không ghi khóa khi null**, `schema_version` không đổi | ĐÚNG. Rust `#[serde(default, skip_serializing_if)]` (`model.rs:236-239`); hai bộ kiểm khóa tùy chọn (`validate.rs:670-688`, `validate.ts:320-323`); `normalize_person_names` / `normalizePersonNames` bỏ `null` (`names.rs:200-205`, `names.ts:62-64`). Vector 03 §2.5 không đổi. Xem MA10-06 về đường lưu không qua chuẩn hóa |
| V3: trỏ ảnh tồn tại | ĐÚNG ở lưu (`lib.rs:1784-1808`, `db.ts:443-449`) và nhập (`validate.rs:1093-1101`, `validate.ts:499-500`) |
| V3 / Ghi chú YC-2026-10: một ảnh chỉ thuộc một người | ĐÚNG: ảnh mâm lễ ≠ ảnh chân dung của mình, ≠ ảnh chân dung / mâm lễ của người khác; ảnh chân dung ≠ ảnh mâm lễ của người khác (`validate.rs:551-573`, `validate.ts:265-277`). Ảnh chân dung dùng chung giữa hai người vẫn được phép như trước |
| 01 §3.8 dọn ảnh: bỏ / đổi / xóa hồ sơ / đổi về đang sống, cùng giao dịch, 0 mồ côi | ĐÚNG. `person_photo_refs` / `personPhotoRefs` thêm `tray_photo_id`, nên `put_person`, `delete_person_with`, `gc_photos` và `putPerson`, `deletePerson` tự xét trường mới; tệp xóa sau `commit` như ảnh chân dung. Kiểm bằng `tray.rs`, `run.mjs`, kịch bản `f_cleanup` |
| 01 §6.3 Gộp giữ ảnh nhập mà hồ sơ sau gộp còn dùng | ĐÚNG (`lib.rs:1983-1994`, `merge.ts:46-48`). Xem MA10-03 về ảnh **trên máy** thành mồ côi |
| B6 khi nhập | ĐÚNG: ảnh mâm lễ là `Photo` nên qua cùng kiểm tra `media::check_photo` / `checkPhoto` (kịch bản `tray_with_exif` → từ chối) |
| R15 (c) cạnh dài ≤ 1600 px | **KHÔNG được lõi kiểm** — MA10-02 |
| R15 (f) không vào thông báo, .ics, tên tệp, URL | Rust: `notice_never_leaks` thêm id ảnh mâm lễ vào danh sách cấm. Trường không được đọc ở `memorial`, `.ics` hay lịch nhắc (không có mã nào ngoài lõi lưu trữ tham chiếu trường). Phía PWA chưa có kiểm thử tương ứng — MA10-07 |
| R14 / R15 (e) dữ liệu và tệp sao lưu cũ | ĐÚNG: `example-v1.lvhv` mở được, kho giống tệp, xuất lại không có khóa; bộ `released/` qua `run.mjs` và `released.rs` không đổi |
| R15 (g) Rust ≡ TS | ĐÚNG trên kịch bản đã định; lệch ở ca lỗi kép — MA10-04 |

## 2. Phát hiện

### MA10-01 — chặn — `scenario tray_photo` xóa đệ quy thư mục người dùng đưa vào

`ffi/src/bin/lvhv-store-tool.rs:1120` gọi `std::fs::remove_dir_all(&work)` trên đúng đối số `<thư-mục-làm-việc>`, và
`fresh()` (`:905-906`) xóa `work/a`, `work/d-*` trước khi mở kho. Không kiểm thư mục có rỗng hay do công cụ tạo không.
Đã thử: tạo `<thư-mục>/keep/file.txt` rồi chạy `lvhv-store-tool scenario tray_photo <thư-mục>` → thoát 0, **cả thư mục
và tệp không liên quan bị xóa**. Gõ nhầm `.` hay `~` sẽ mất dữ liệu trên máy của bên kiểm định (T28 được giao chạy lệnh
này, 10 §3.11; tài liệu chỉ ghi "xóa khi xong", không cảnh báo). Bản TS bỏ qua đối số nên hai bản cũng không cùng hành vi.

Sửa: từ chối khi `<thư-mục>` đã tồn tại và không rỗng (hoặc luôn tạo thư mục con duy nhất bằng `tempfile`/`mkdtemp`
bên trong và chỉ xóa thư mục con đó); bỏ `remove_dir_all` trong `fresh()` hoặc chỉ áp cho thư mục con do công cụ tạo.
Thêm một ca vào `check-store-tool.sh`: thư mục có sẵn tệp → `exit 2` và tệp còn nguyên. Ghi rõ ở 10 §3.11.

### MA10-02 — nên sửa — lõi không kiểm giới hạn cạnh dài ≤ 1600 px của ảnh mâm lễ

01 §3.7 ghi giới hạn này ở cột "Kiểm tra" của trường; R15 (c) yêu cầu "cạnh dài ≤ 1600 px", và phương pháp R15 giao
(c) cho "kiểm thử lõi hai bản và `lvhv-store-tool`". Lõi chỉ kiểm ≤ 2048 chung cho mọi `Photo` (`store/src/media.rs:17`,
`validate.rs:925-926`). Đã thử trên cả hai bản: ảnh JPEG 2000×1000 → `add_photo` nhận, gắn làm `tray_photo_id` nhận, xuất
rồi nhập (Thay toàn bộ) nhận. PR chuyển trách nhiệm sang giao diện ở 10 §3.9 (`docs/v0.2/10-CONG-CU-KIEM-DINH.md:207`)
mà không có bổ sung đặc tả. Có tiền lệ (ảnh chân dung 1024 px chỉ là mặc định của giao diện, lõi cho tới 2048), nên
không xếp "chặn"; nhưng như hiện tại R15 (c) phần kích thước **không kiểm được** bằng lõi và tệp sao lưu tự dựng có ảnh
mâm lễ 2048 px vẫn vào được.

Sửa (chọn một, điều phối viên quyết): (a) ở `check_refs_person` (`lib.rs:1784`) và `validate::document`
(`validate.rs:1093`) cùng bản TS, ảnh mà `tray_photo_id` trỏ tới phải có `max(width, height) ≤ 1600` (mã V3 hay V1 theo
quyết định), kèm ca thử 1601 px trong kịch bản; hoặc (b) điều phối viên bổ sung B8 ghi rõ 1600 px là việc của nền tảng
và R15 (c) phần kích thước kiểm bằng kiểm thử giao diện.

### MA10-03 — nên sửa — Gộp để lại ảnh mâm lễ cũ của máy thành mồ côi, không bao giờ được dọn và đi theo các tệp sao lưu sau

`merge_docs` giữ **mọi** ảnh trên máy (`lib.rs:1992`; `merge.ts:48`). Khi cùng một hồ sơ có bản nhập thắng và bản đó
dùng ảnh mâm lễ khác, ảnh mâm lễ cũ trên máy còn lại mà không hồ sơ nào dùng. Đã thử trên cả hai bản: máy A gắn `T0`
cho 5e01; máy B gắn `T1` (rev lớn hơn) rồi xuất; A gộp tệp của B → 5e01 dùng `T1`, `T0` mồ côi; tệp sao lưu A xuất sau
đó **vẫn chứa `T0`**. `gc_photos` chỉ xét ảnh ứng viên của lần lưu, nên `T0` nằm mãi (người dùng không thấy ảnh này ở
đâu để xóa). Đây là hành vi có sẵn với ảnh chân dung và đúng câu chữ 01 §6.3 ("giữ mọi bản trên máy"), nhưng với ảnh
mâm lễ nó trái ý của R15 (ngưỡng "0 ảnh mồ côi") và R-PRIV-3 (ảnh riêng tư mà người dùng đã thay vẫn bị sao lưu đi).
Kịch bản `merge_over_old` không bắt được vì kho cũ không có ảnh mâm lễ.

Sửa: điều phối viên quyết lại câu chữ 01 §6.3; đề xuất "giữ ảnh trên máy mà hồ sơ sau gộp còn dùng hoặc **không hồ sơ
nào trên máy đang dùng** (ảnh vừa thêm chưa gắn)", tức bỏ ảnh mà hồ sơ trên máy đang dùng nhưng sau gộp không ai dùng;
xóa tệp sau giao dịch như `put_person`. Thêm ca gộp này vào `tray.rs`, `run.mjs` và kịch bản (`merge_replaces_local_tray`).

### MA10-04 — góp ý — Rust ≢ TS ở ca lỗi kép (thứ tự kiểm V3)

Rust kiểm tệp đính kèm treo → ảnh treo → dùng chung (`lib.rs:1770-1826`); TS kiểm dùng chung trước giao dịch (`db.ts:429-430`),
rồi ảnh / tệp đính kèm treo trong giao dịch. Đã thử: `photo_id = tray_photo_id = X` (X không tồn tại) → Rust
`V3 /person/photo_id: ảnh không tồn tại`; TS `V3 /person/memorial/tray_photo_id: ảnh mâm lễ dùng chung với hồ sơ khác`.
Cả hai đều từ chối, nên không sai về an toàn; nhưng R15 (g) nói "Rust ≡ TS". Sửa: đưa kiểm dùng chung của TS về sau
kiểm treo (hoặc Rust lên trước) và thêm ca này vào kịch bản.

### MA10-05 — góp ý — thông điệp sai khi ảnh mâm lễ trùng ảnh chân dung **của chính người đó**

`tray_photo_shared` trả `memorial/tray_photo_id` cho ca này (`validate.rs:554-556`, `validate.ts:268-269`), và các nơi báo
"ảnh mâm lễ dùng chung với hồ sơ khác" (`lib.rs:1823`, `validate.rs:1106`, `validate.ts:502`, `db.ts:430`). Nên tách
thông điệp, ví dụ "ảnh mâm lễ không được trùng ảnh chân dung", vì nền tảng có thể hiện thông điệp này cho người dùng.

### MA10-06 — góp ý — `Store::put_person` / `Store.putPerson` vẫn ghi `"tray_photo_id": null` nếu bên gọi không chuẩn hóa

Đã thử: gọi thẳng `Store::put_person` với `null` → khóa còn trong kho (TS giống vậy). FFI `save_person_json` và biểu
mẫu PWA (`forms.ts:322`, `notice.ts:245`) đều gọi chuẩn hóa nên đường của ứng dụng đúng; đây cùng mẫu với `gioi_tinh`.
Nhưng 01 §3.7 nói "không ghi" là bất biến của tài liệu. Nên bỏ khóa ngay trong `put_person` / `putPerson` (rẻ, hai bản
cùng chỗ) để một đường gọi mới không làm lệch byte.

### MA10-07 — góp ý — chưa có kiểm thử phía PWA rằng ảnh mâm lễ không vào thông báo / .ics

Rust có (`ffi/src/tests_v02.rs:804-908`). PWA không đổi mã thông báo / .ics nên rủi ro thấp, nhưng R15 (f) đòi "không
bao giờ"; nên thêm id ảnh mâm lễ vào danh sách chuỗi cấm ở kiểm thử thông báo / .ics hiện có của PWA.

### MA10-08 — góp ý — `check_refs_person` đọc và phân tích mọi hồ sơ ở mỗi lần lưu

`lib.rs:1810-1817` chạy `SELECT json FROM person WHERE id <> ?1` và `parse_col` mọi dòng mỗi lần lưu có ảnh; khi mở kho
sau chuyển đổi (`lib.rs:531-535`) nó chạy cho từng hồ sơ nên thành O(n²) lần phân tích JSON. Với vài trăm hồ sơ vẫn
nhanh; nếu muốn, đọc một lần tập id ảnh mâm lễ / ảnh chân dung của người khác, hoặc truy vấn `json_extract`.

### MA10-09 — góp ý — công cụ kiểm định

- `tv()` (`lvhv-store-tool.rs:828`) dùng `CARGO_MANIFEST_DIR` lúc dựng: tệp nhị phân chỉ chạy được trên đúng cây mã đã
  dựng nó; nhiều `unwrap()` trong kịch bản → thoát 101 thay vì JSON `ok: false`. Nên ghi điều này ở 10 §3.11 hoặc nhận
  đường dẫn vector bằng tùy chọn.
- `orphan_files` của TS (`pwa/test/tools/lvhv-store-tool.mjs:440-446`) chỉ đếm bản ghi không đọc được byte, còn Rust đếm
  cả tệp thừa không có bản ghi. JSON bằng nhau nhưng hai bản không đo cùng một thứ; 10 §3.11 đã ghi, nên giữ ghi chú này
  khi viết báo cáo kiểm định.

### MA10-10 — góp ý — ca thử còn thiếu

Ngoài các ca đã nêu ở MA10-01..04: ảnh mâm lễ WebP (mọi ca hiện dùng JPEG); nhập Gộp mà hai bên có hai hồ sơ khác nhau
cùng trỏ một ảnh mâm lễ (hiện bị `validate::document(&merged)` từ chối — đúng V3, nhưng chưa có ca khóa hành vi này);
đổi `tray_photo_id` từ ảnh A sang ảnh chân dung cũ của chính mình sau khi bỏ ảnh chân dung trong cùng lần lưu.

## 3. Bảo mật và riêng tư

- Không có đường mới đọc byte ảnh ra ngoài kho; ảnh mâm lễ đi qua cùng kiểm tra B6 như ảnh chân dung ở `add_photo` và
  ở nhập sao lưu. Tên tệp trên máy vẫn là `photos/<id>.img` (01 §3.8), id là UUID đã kiểm.
- Lược đồ `backup-v1.schema.json` thêm khóa với `oneOf [uuid, null]`, giữ `additionalProperties: false`.
- Rủi ro còn lại: MA10-03 (ảnh đã thay vẫn đi theo sao lưu), MA10-01 (công cụ, không thuộc ứng dụng phát hành).

## 4. Khả năng bảo trì

Mã gọn, hai bản đặt tên song song (`tray_photo_ref` / `trayPhotoRef`, `tray_photo_shared` / `trayPhotoShared`), chú
thích dẫn đúng mục đặc tả. Việc gộp `tray_photo_id` vào `person_photo_refs` làm mọi đường dọn ảnh hiện có tự đúng mà
không phải sửa từng chỗ — cách làm tốt. `photoRecord` tách khỏi `putPerson` để `addPhoto` dùng lại, đúng như Rust.

## 5. Kết luận

**CHƯA GỘP ĐƯỢC** — 1 phát hiện chặn (MA10-01), 2 nên sửa (MA10-02, MA10-03), 7 góp ý.

Phần lõi của ứng dụng (lược đồ, V3, dọn ảnh, gộp, sao lưu, FFI, R14) đúng đặc tả và hai bản cho cùng kết quả trên kịch
bản đã định; mọi kiểm thử chạy lại đều ĐẠT. Chặn duy nhất nằm ở công cụ kiểm định: `scenario tray_photo` xóa đệ quy
thư mục được đưa vào. Sau khi sửa MA10-01 có thể gộp; MA10-02 và MA10-03 cần điều phối viên quyết (sửa mã hay bổ sung
đặc tả) trước khi bên kiểm định chạy R15 (c)(d) (T28), nếu không R15 (c) phần kích thước sẽ là CHƯA ĐỦ BẰNG CHỨNG.

## Vòng 2 — 2026-11-07: kiểm lại trên `375d6d6`

Đầu nhánh được duyệt: `375d6d6` (`v0.2-yc10-core`; đã gộp `main` ở `b7c2b81`, gồm Bổ sung 01-YC10-a #53 và phiếu này
#52). Phần sửa: `git diff b7c2b81..375d6d6`. Căn cứ thêm: 01 "Bổ sung 01-YC10-a"; báo cáo kiểm định
`docs/reports/BAO-CAO-KIEM-DINH-V02-R15.md` (PR #54, đo trên `eafa96e`), mục F1–F3.

### V2.0 Đã chạy

| Việc | Kết quả |
|---|---|
| `cargo test --locked -p lvhv-store`, `-p lvhv-ffi` | Mọi bài ĐẠT (`tray.rs` 9/9) |
| `cross_platform write` → `node pwa/test/store/run.mjs --quick` → `cross_platform check` | **ĐẠT**; `state_hash` khớp |
| `node tsport/test/memorial_unit.mjs` (sau `npm run build`) | 60 đạt, 0 lỗi (gồm ca mới của MA10-07) |
| `scenario tray_photo` Rust (debug) và TS | Hai bản **giống từng byte** và bằng `tray-photo/expected-scenario.json` mới |
| Sinh lại vector (`gen_test_vectors`, `gen_notice_vector`) | Không khác bản đã commit (03 §2.5, `merge-cases.json`, R6v2 không đổi) |
| Chạy lại các ca thử của vòng 1 (mã thử tạm, đã xóa) trên **cả hai** bản | Kết quả ở V2.1 |
| CI trên `375d6d6` | Mọi việc `success` (Rust fmt/clippy/test, Lưu trữ + sao lưu, L7, memorial Rust ≡ TS, cargo-deny, gói ảnh mâm lễ, `ui`, `upgrade`, `screenshots`, `build`) |

### V2.1 Kiểm lại từng phát hiện

| Mã | Mức | Kết quả | Bằng chứng |
|---|---|---|---|
| MA10-01 | chặn | **ĐÃ SỬA** | `lvhv-store-tool.rs:983-994`, `:1241-1254`: tạo thư mục con mới `lvhv-tray-<16 hex>` bằng `create_dir` (không trùng thứ có sẵn), chỉ xóa thư mục đó; chỉ `remove_dir` (khi rỗng) thư mục người dùng nếu chính công cụ tạo; `fresh()` không xóa nữa. `import` không `--into` dùng cùng cách. Đã thử lại: thư mục có sẵn `keep/file.txt` → thoát 0, sau khi chạy chỉ còn `keep/` và `keep/file.txt`. CI kiểm ca này (`check-store-tool.sh`) |
| MA10-02 / F1 | nên sửa | **ĐÃ SỬA** | Theo Bổ sung 01-YC10-a: `check_refs_person` (`lib.rs:1800-1815`), `validate::document` (`validate.rs:1137-1143`), `putPerson` (`db.ts:447`), `validate.ts` `document`. Đã thử: ảnh 2000×1000 → hai bản `V1 /person/memorial/tray_photo_id: ảnh mâm lễ có cạnh dài quá 1600 px`; tệp nhập 1601 px → `V1 /persons/0/memorial/tray_photo_id` (kịch bản `tray_too_large`) |
| MA10-03 | nên sửa | **ĐÃ SỬA** | `merge_docs` (`lib.rs:2022`, `:2036`), `mergeDocs` (`merge.ts:48`, `:53`) chỉ giữ ảnh / tệp đính kèm mà hồ sơ sau gộp còn dùng; tệp bị dọn sau giao dịch (`apply_import`). Đã thử lại ca máy A `T0` / máy B `T1`: hai bản 0 mồ côi, tệp `T0` không còn. Kịch bản `merge_replaces_local_tray`. Xem thêm V2.2 MA10-11 |
| MA10-04 | góp ý | **ĐÃ SỬA** | Cùng thứ tự ở ba nơi (ảnh treo → cỡ ảnh mâm lễ → dùng chung → tệp đính kèm treo). Ca `photo_id = tray_photo_id = X` (không có) → hai bản `V3 /person/photo_id: ảnh không tồn tại` (kịch bản `g_rules.double_dangling`) |
| MA10-05 | góp ý | **ĐÃ SỬA** | `TRAY_IS_PORTRAIT` "ảnh mâm lễ không được trùng ảnh chân dung" (`validate.rs:556`, `validate.ts`); đã thử, hai bản đúng câu này |
| MA10-06 / F2 | góp ý | **ĐÃ SỬA** | `upsert_person` bỏ khóa `null` ở mọi đường ghi (`lib.rs:1851-1862`, `validate::without_null_tray` `validate.rs:577`); TS `normNotice` (`db.ts:35-43`). Đã thử: gọi thẳng `Store::put_person` / `putPerson` với `null` → khóa không còn. Tệp nhập ghi rõ `null` → nhập được, kho và tệp xuất lại không có khóa (kịch bản `explicit_null`) |
| MA10-07 | góp ý | **ĐÃ SỬA** | `tsport/test/memorial_unit.mjs`: .ics (toàn bộ và chọn người), `icsSummary`, `upcoming`, `dueToday`, `notificationText` không chứa id ảnh mâm lễ; chạy ĐẠT |
| MA10-08 | góp ý | **CHƯA SỬA** | `lib.rs:1819` vẫn đọc mọi hồ sơ ở mỗi lần lưu. Chấp nhận cho v0.2 (dữ liệu nhỏ); không chặn |
| MA10-09 | góp ý | **ĐÃ SỬA một phần** | 10 §3.11 ghi rõ bản Rust đọc vector từ cây mã đã dựng nó và `orphan_files` của TS đo khác Rust. Các `unwrap()` trong kịch bản còn (thoát 101 thay vì JSON) — chấp nhận, công cụ kiểm định |
| MA10-10 | góp ý | **ĐÃ SỬA** | Thêm: ảnh mâm lễ WebP (`f_cleanup.webp`), gộp làm hai hồ sơ dùng chung ảnh mâm lễ → từ chối, kho không đổi (`merge_shared_tray`), lỗi kép (`double_dangling`). Ca "đổi ảnh mâm lễ sang ảnh chân dung cũ trong cùng lần lưu" chưa có — không cần thêm, đã bị V3 / dọn ảnh bao |
| F2 (kiểm định) | — | **ĐÃ SỬA** | Như MA10-06 |
| F3 (kiểm định) | — | **ĐÃ SỬA** | `save-person` bị từ chối → `stage = "save_person"` ở cả hai bản (`lvhv-store-tool.rs:461`, `lvhv-store-tool.mjs:287`); 10 §3.3 ghi tên chung; CI so hai bản. Bên kiểm định cần chạy lại g2 để đổi kết luận (G) của mình |

### V2.2 Phát hiện mới

#### MA10-11 — góp ý — Gộp nay xóa cả ảnh vừa thêm mà chưa gắn hồ sơ

Theo đúng câu chữ Bổ sung 01-YC10-a ("ảnh trên máy mà không hồ sơ nào trong kết quả gộp còn trỏ tới bị xóa"), ảnh đã
`add_photo` / `addPhoto` nhưng biểu mẫu chưa lưu cũng bị xóa khi Gộp (`lib.rs:2022`, `merge.ts:48`). Nếu người dùng mở
biểu mẫu, chọn ảnh, rồi nhập sao lưu (Gộp) ở cửa sổ / thẻ khác, lần lưu biểu mẫu sau đó báo `V3 … ảnh không tồn tại`.
Không mất dữ liệu đã lưu và đúng đặc tả đã bổ sung; chỉ cần giao diện (T23–T26) chặn nhập sao lưu khi đang có ảnh chờ
gắn, hoặc thêm lại ảnh khi gặp lỗi này. Không chặn.

Không có phát hiện chặn mới.

### V2.3 Kết luận vòng 2

**GỘP ĐƯỢC** — 0 chặn. MA10-01..07 và MA10-10 đã sửa; MA10-09 sửa một phần (chấp nhận); MA10-08 chưa sửa (góp ý,
chấp nhận); F2, F3 của báo cáo kiểm định đã sửa; một góp ý mới (MA10-11) chuyển cho giao diện. Mọi kiểm thử chạy lại đều
ĐẠT, Rust ≡ TS từng byte trên kịch bản `tray_photo`, vector đã khóa không đổi. Bên kiểm định cần đo lại R15 (c)(d)(g) trên
`375d6d6` để cập nhật kết luận của mình (F1 nay là ca từ chối; g2 không còn khác biệt `stage`).
