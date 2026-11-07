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
