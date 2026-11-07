# 10 — Công cụ kiểm định hộp đen `lvhv-store-tool`

Công cụ dòng lệnh cho bên kiểm định độc lập. Nó mở kho, nhập sao lưu, lưu hồ sơ và in danh sách gợi ý, rồi in kết
quả ra dạng JSON để so với kỳ vọng mà không phải đọc mã nguồn. Công cụ làm theo yêu cầu ở mục 7 của
`docs/reports/BAO-CAO-KIEM-DINH-V02-B5.md` (PR #33), gồm ba việc:
- (i) mở thư mục kho SQLite, in tài liệu kho và trạng thái chuyển đổi;
- (ii) nhập tệp sao lưu tùy ý, in toàn bộ kho sau khi nhập;
- (iii) gọi lưu hồ sơ và danh sách gợi ý cách gọi, quan hệ.

Bản sau thêm `--now` (giả lập đồng hồ để kiểm việc dọn sau 7 ngày), kho TS giữ được giữa các lần chạy (`--state`),
`export` và `inspect-export` (kiểm tệp sao lưu xuất ra không mang `legacy`). YC-2026-10 thêm `add-photo`,
`delete-person` và `scenario tray_photo` (ảnh mâm lễ, 04 Bổ sung B8 / R15 (c)(d)(e)(g) — §3.9–§3.11).

Có hai bản, cùng lệnh, cùng dạng kết quả:

| Bản | Tệp | Gọi |
|---|---|---|
| Rust (lõi, như ứng dụng Android/Apple/Windows) | `ffi/src/bin/lvhv-store-tool.rs` | `lvhv_store` (mở/chuyển đổi kho, nhập, xuất, ảnh chụp kho), `lvhv_backup::import` (giải mã cho `inspect-export`) và `lvhv_ffi::LvhvStore::save_person_json` (đúng đường lưu hồ sơ của ứng dụng) |
| TypeScript (PWA) | `pwa/test/tools/lvhv-store-tool.mjs` | API công khai của `pwa/src/store/index.ts`, IndexedDB giả lập (`fake-indexeddb`) |

Bản Rust nằm trong crate `lvhv-ffi`, không nằm trong `lvhv-store`. Lý do: phần lưu hồ sơ của ứng dụng nằm ở
`save_person_json` của lớp FFI. Phần này gắn id/`rev`/thời điểm, chuẩn hóa thông tin báo tử và viết hoa chữ đầu, rồi
mới gọi kho. `lvhv-store` không phụ thuộc được vào `lvhv-ffi`.

## 1. Dựng

```
# Rust
cargo build --locked -p lvhv-ffi --bin lvhv-store-tool
# → target/debug/lvhv-store-tool  (thêm --release nếu muốn; bản dev đã tối ưu argon2)

# TypeScript
(cd pwa && npm ci)
node pwa/test/tools/lvhv-store-tool.mjs …
```

## 2. Kết quả và mã thoát

- stdout: đúng **một dòng JSON chuẩn**. Khóa sắp theo byte, không khoảng trắng. Kết quả có dạng JSON cả khi lệnh bị
  từ chối (`"ok": false`, `"stage"`, `"error"`).
- Mã thoát:
  - `0`: thành công;
  - `1`: thao tác bị từ chối hoặc lỗi; JSON vẫn được in ra;
  - `2`: sai cú pháp; lời nhắc cách dùng in ra stderr.
- Lỗi của bản Rust có:
  - `kind`: `Invalid`, `Input`, `NewerSchema`, `ReadOnly`, `NotFound`, `Backup`, `Sql` hoặc `Io`;
  - `display` và `debug`;
  - `import_message`: thông báo chung mà ứng dụng hiện khi nhập;
  - với `Invalid` có thêm `code` (`V1`…`V8`, `ATTACHMENT`, `PHOTO`, `JSON`), `path`, `msg`;
  - với `NewerSchema` có thêm `version`;
  - với `Backup` có thêm `backup_error` (`WrongPasswordOrCorrupt`, `MalformedPlaintext`, …).
- Lỗi của bản TS có `kind`, `code` (`invalid`, `backup`, `read_only`, `newer_schema`, …) và `cause` (lỗi gốc, ví dụ
  `BackupError` với `wrong_password_or_corrupt`).

Phần `store` (lệnh `open` và `import`) gồm:

| Khóa | Nội dung |
|---|---|
| `document` | Tài liệu toàn kho như `state_hash` băm: tài liệu sao lưu 01 §5 bỏ `app` và `exported_at`. Có `attachments` khi khác rỗng |
| `photo_bytes`, `attachment_bytes` | Theo `id`: `{bytes, sha256}` của byte đọc từ kho (đã kiểm sha256), hoặc `{error}` |
| `state_sha256` | `Store::state_hash` / `stateHash` (03 B1.4 mục 10); `null` nếu không băm được |
| `user_version` (Rust) / `idb_version` (TS) | Phiên bản kho sau khi mở: `PRAGMA user_version`, hoặc phiên bản IndexedDB |
| `backup_pending_since` | Thời điểm thay đổi đầu tiên chưa sao lưu |

## 3. Lệnh

### 3.0 Kho TS giữ giữa các lần chạy: `--state <tệp>`

Bản Rust có thư mục kho trên đĩa, nên chạy nhiều lệnh liên tiếp trên cùng một kho là việc tự nhiên. Kho IndexedDB của
bản TS chỉ sống trong tiến trình. Tùy chọn chung `--state <tệp>` (cho `open`, `import`, `save-person`, `export`):
- tệp đã có: kho được dựng lại từ tệp, thay cho `<thư-mục>` (kết quả có `"source": "state"`);
- tệp chưa có: kho lấy từ `<thư-mục>` như bình thường (`fixture_v1` hoặc `fresh`);
- sau lệnh: toàn bộ IndexedDB được ghi lại vào tệp ở dạng JSON chuẩn: tên, phiên bản, từng kho đối tượng (`keyPath`,
  chỉ mục, mọi bản ghi theo khóa); Blob ghi là `{"$blob": {"type", "base64"}}`.

```
node pwa/test/tools/lvhv-store-tool.mjs open docs/v0.2/schema/test-vectors/released/0.2-dev-before-attachments --state /tmp/kho-ts.json
node pwa/test/tools/lvhv-store-tool.mjs open - --state /tmp/kho-ts.json      # mở lại cùng kho: state = "ok"
```

### 3.1 `open <thư-mục> [--force-migration-failure] [--now <unix_s>]`

Rust: mở (hoặc tạo) kho SQLite trong `<thư-mục>`. Kho phiên bản cũ được **chuyển đổi tại chỗ**, nên phải chép bộ
mẫu ra chỗ khác trước khi mở. Không bao giờ mở thẳng thư mục trong `test-vectors/released/`.

```
cp -r docs/v0.2/schema/test-vectors/released/0.2-dev-before-attachments/store /tmp/kho
target/debug/lvhv-store-tool open /tmp/kho
```

In ra:
- `migration_status`: `state` (`ok` / `migrated` / `read_only`), `from_version`, `to_version`, `backup_path`
  (bản sao `lvhv.sqlite.pre-v2.bak`), `backup_exists`, `error`;
- `store`;
- `files_before`, `files_after`: mọi tệp trong thư mục, kèm cỡ và sha256, trước và sau khi mở.

Kho của bản mới hơn: `ok: false`, `error.kind = "NewerSchema"`.

`--force-migration-failure` mở bằng `Store::open_with(dir, OpenOptions { force_migration_failure: true })`: bước
chuyển đổi lỗi **sau** khi đã chạy trong giao dịch, trước `COMMIT`. Như vậy quan sát được đúng đường lỗi của 04 B5 mục 3:
- giao dịch quay lui: `user_version` vẫn là 1, dữ liệu cũ còn nguyên;
- bản sao `.pre-v2.bak` còn trên đĩa;
- `state = "read_only"`.

Ở chế độ chỉ đọc, công cụ thử thêm và in kết quả trong `read_only_probe`:
- `put_settings`, `put_person`, `delete_person` phải lỗi `ReadOnly`;
- `export_backup` phải thành công.

TS: `<thư-mục>` là **thư mục bộ mẫu** theo bố cục `test-vectors/released/<bản>/`, gồm `store-snapshot.json` và
`store/photos/<id>.img`. Công cụ dựng kho IndexedDB phiên bản 1 từ thư mục đó, như bản dựng trước YC-2026-07, rồi mở
bằng `openStore` của bản này. Thư mục không có `store-snapshot.json`, hoặc `-`, cho kho trống. Không ghi gì vào thư
mục. Ở TS, `--force-migration-failure` gọi `openStore(factory, { forceMigrationFailure: true })`: giao dịch nâng cấp
quay lui, và kho được mở lại ở chế độ chỉ đọc. `to_version` của TS là phiên bản IndexedDB (3), không phải 2.

`--now <unix_s>` đặt "bây giờ" khi mở kho (giây Unix): Rust `OpenOptions { now_unix: Some(unix_s) }`, TS
`openStore(factory, { now })` với `now` là thời điểm RFC 3339 tương ứng. Thời điểm này được ghi làm lúc chuyển đổi xong,
và dùng để quyết định việc dọn sau 7 ngày (01 §7, BM2-06/07): xóa `Person.legacy` và bản sao `.pre-v2.bak`. Không cần
sửa CSDL để giả lập. Ví dụ, trên cùng một kho:

```
T0=1793000000
target/debug/lvhv-store-tool open /tmp/kho --now $T0                     # chuyển đổi; còn legacy, còn .pre-v2.bak
target/debug/lvhv-store-tool open /tmp/kho --now $((T0 + 86400))         # 1 ngày sau: chưa dọn
target/debug/lvhv-store-tool open /tmp/kho --now $((T0 + 8 * 86400))     # 8 ngày sau: đã dọn
```

Bản TS chạy cùng chuỗi lệnh với `--state` (mục 3.0). Kết quả in thêm `now_unix`.

### 3.2 `import <tệp-sao-lưu> <mật-khẩu> [--merge|--replace] [--into <thư-mục>]`

Giải mã và kiểm tra tệp (`open_backup`), rồi ghi vào kho (`apply_import`). Mặc định là `--merge`, như ứng dụng
(BK-S1). Không có `--into` thì nhập vào một kho trống tạm, xóa sau khi chạy. Có `--into` thì nhập vào kho ở thư mục
đó: Rust ghi vào thư mục, còn TS dùng thư mục bộ mẫu như mục 3.1 và không ghi gì.

```
target/debug/lvhv-store-tool import docs/v0.2/schema/test-vectors/released/0.2-dev-before-attachments/backup.lvhv "mật khẩu thử nghiệm" --replace
```

In ra:
- `preview`: số người, số sự kiện, số ảnh, `exported_at`, `platform`;
- `verified_document`: tài liệu đã kiểm tra và đã chuyển đổi, trước khi ghi;
- `before`: trạng thái chuyển đổi và `state_sha256` của kho đích trước khi nhập;
- `store`: **toàn bộ kho sau khi nhập**, không chỉ băm;
- `mode`, `file_sha256`.

Tệp bị từ chối: `ok: false`, `stage` (`open_backup`, `open_store` hoặc `apply_import`) và `error`.

### 3.3 `save-person <thư-mục> <person.json | ->`

Lưu một `Person` như màn hình hồ sơ. `id` rỗng thì sinh id mới. Công cụ gắn `created_at`/`updated_at`/`rev`, viết
hoa chữ đầu `danh_xung` và `quan_he` (YC-2026-07), bỏ khóa `gioi_tinh: null`, tính trường dẫn xuất, kiểm V1–V8, rồi
in **bản đã lưu** (`saved`) đọc lại từ kho. `-` thay cho tên tệp: đọc JSON từ stdin.
- Rust gọi `LvhvStore::save_person_json` trên kho ở `<thư-mục>`, kho được tạo nếu chưa có.
- TS làm như `src/people/forms.ts`: `buildPerson(normalizePersonNames(…))`, rồi `validate.person`, rồi `putPerson`.
  Ở TS, `<thư-mục>` là thư mục bộ mẫu, hoặc `-` cho kho trống; thêm `--state` để lưu nhiều lần trên cùng kho
  (`rev` tăng 1 → 2, `created_at` giữ nguyên).

```
target/debug/lvhv-store-tool save-person /tmp/kho-moi nguoi.json
```

### 3.4 `suggestions <deceased|living> [nam|nu]`

In `danh_xung` (`danh_xung_suggestions`: 22 mục cho người mất, 17 mục cho người sống; có `nam`/`nu` thì sắp theo
giới tính) và `quan_he` (`quan_he_suggestions`).

### 3.5 `display-capitalize <chuỗi>`

In `display_capitalize(<chuỗi>)`: quy tắc viết hoa khi hiển thị cách gọi/quan hệ đã lưu.

### 3.6 `check-media <photo|attachment> <mime> <tệp> [width height]`

Kiểm byte một ảnh hoặc tệp đính kèm như khi nhập (01 §3.8a, Bổ sung B6): Rust `lvhv_store::media::check_photo` /
`check_attachment`, TS `mediaVerdict`. In `verdict`: `ok` hoặc mã lỗi (`bad_mime`, `mime_mismatch`, `bad_header`,
`metadata`, `dimensions_mismatch`, `too_large`, `pdf_no_eof`, `trailing_data` — rà soát bảo mật vòng 2, BM3-04). Với ảnh, `width height` là kích thước khai trong bản
ghi; thiếu thì là 0 0.

### 3.7 `export <thư-mục> <mật-khẩu> <tệp-ra> [--now <unix_s>]`

Xuất tệp sao lưu bằng đường xuất thật của ứng dụng rồi ghi ra `<tệp-ra>`. In `bytes`, `sha256` của tệp và trạng thái
chuyển đổi.
- Rust: `Store::open_with` (với `--now` như `open`), rồi `Store::export_backup`. Đây chính là hàm mà
  `LvhvStore::export_backup` của lớp FFI gọi. `app` = `{version: "0.2.0", platform: "android"}`.
- TS: `openStore`, rồi `Store.exportBackup`, như nút "Xuất sao lưu" của PWA. `app.platform` = `"pwa"`. Dùng được với
  `--state`.

Mật khẩu ngắn hơn 8 ký tự: `ok: false`, `stage = "export_backup"`. Như ứng dụng, lệnh này ghi lại thời điểm tạo tệp
(`last_export_at`) nhưng không đánh dấu "đã sao lưu".

### 3.8 `inspect-export <tệp-sao-lưu> <mật-khẩu>`

Chỉ giải mã (Rust `lvhv_backup::import`, TS `backup-crypto.importBackup`) và in **bản rõ** đúng như trong tệp. Không
kiểm tra nội dung, không chuyển đổi, không ghi vào kho nào. In ra:
- `document`: tài liệu JSON của bản rõ;
- `json_is_canonical`: chuỗi JSON trong tệp có đúng là JSON chuẩn không;
- `json_sha256`, `file_sha256`;
- `blobs`: danh sách khối theo thứ tự trong tệp (`id`, `bytes`, `sha256`);
- `legacy_paths`: mọi đường dẫn có khóa `legacy` ở bất kỳ mức nào. Phải là `[]` với tệp do `export` tạo ra (01 §7).

Sai mật khẩu hoặc tệp hỏng: `ok: false`, `stage = "decrypt"`.

### 3.9 `add-photo <thư-mục> <mime> <tệp> <width> <height> [--id <uuid>]`

Thêm một ảnh (`Photo`, 01 §3.8) chưa gắn hồ sơ: Rust `LvhvStore::add_photo`, TS `Store.addPhoto` — đúng hàm ứng dụng
gọi cho ảnh chân dung và ảnh mâm lễ (YC-2026-10). `width`/`height` là kích thước khai trong bản ghi. Lõi kiểm như khi
nhập: bản ghi (≤ 1 MiB — V1 `/photo/bytes`) và byte (04 B6: chữ ký, không EXIF/XMP/IPTC, không dữ liệu nối sau ảnh,
kích thước đúng, ≤ 2048 px). Việc thu nhỏ về cạnh dài ≤ 1600 px và mã hóa lại là của nền tảng (giao diện), không phải
của lõi: lõi chỉ nhận byte đã xử lý. In `photo` (bản ghi) khi nhận; khi từ chối in `ok: false`, `stage = "add_photo"`,
`outcome` (§3.11). Không có `--id` → id ngẫu nhiên. Gắn ảnh vào hồ sơ bằng `save-person` với `photo_id` hoặc
`memorial.tray_photo_id`. TS cần `--state` để giữ kho (§3.0).

### 3.10 `delete-person <thư-mục> <id>`

Xóa một hồ sơ (Rust `LvhvStore::delete_person`, TS `Store.deletePerson`). Ảnh chân dung và ảnh mâm lễ của người đó bị
xóa trong cùng giao dịch nếu không hồ sơ nào khác dùng (R15 (f)). Kiểm kết quả bằng `open` (Rust) hoặc `open -
--state` (TS): `store.photo_bytes` không còn id ảnh.

### 3.11 `scenario tray_photo <thư-mục-làm-việc>`

Kịch bản cố định của R15 (04 Bổ sung B8), chạy trên kho mới (Rust: trong `<thư-mục-làm-việc>`, xóa khi xong; TS: kho
trong tiến trình, đối số này bỏ qua). Mọi thao tác đi qua đúng đường của ứng dụng (Rust: `LvhvStore` của FFI; TS: `Store`
+ `buildPerson(normalizePersonNames(…))` như màn hình hồ sơ). In `result` đã chuẩn hóa: không có thời điểm hay byte
ngẫu nhiên, nên hai bản phải in **cùng một JSON** (R15 (g)); tệp kỳ vọng
`docs/v0.2/schema/test-vectors/tray-photo/expected-scenario.json`. Mỗi thao tác cho một `outcome`: `ok`,
`invalid <mã> <đường dẫn>` (ví dụ `invalid V3 /person/memorial/tray_photo_id`), `input` (lỗi đầu vào không có mã V, ví dụ
ảnh còn EXIF), `not_found`, `backup` (thông báo chung khi nhập tệp: "Sai mật khẩu hoặc tệp bị hỏng"), `storage`.

| Khóa | Mệnh đề | Nội dung |
|---|---|---|
| `e_old_backup` | (e) | Nhập `example-v1.lvhv` (bản trước, không có trường): `unchanged` = kho giống tệp; `has_tray_key = false`; `tray_d1 = "absent"` |
| `c_add_photo` | (c) phía lõi | Ảnh còn EXIF → `input`; ảnh > 1 MiB → `invalid V1 /photo/bytes`; ảnh sạch → `ok` |
| `set` | V3 | Gắn ảnh mâm lễ cho người đã mất 5e01. `v3`: ảnh không tồn tại; ảnh mâm lễ của người khác; ảnh chân dung của người khác; ảnh chân dung của chính mình; ảnh chân dung trỏ ảnh mâm lễ của người khác → mọi ca `invalid V3 …` |
| `d_round_trip` | (d) | Xuất rồi nhập (Thay toàn bộ; Gộp vào kho trống; Gộp vào kho có bản cũ): `tray_d1` giữ, `sha_equal = true`, 0 ảnh mồ côi. `crafted`: tệp sửa tay có ảnh mâm lễ không tồn tại, ảnh mâm lễ còn EXIF, ảnh mâm lễ dùng chung → `open_backup = "backup"` (thông báo chung) và `verify` cho lý do (`invalid V3 …`, `invalid PHOTO /photos`); `control` (tệp đúng) → `ok` |
| `f_cleanup` | (f) | Đổi ảnh, bỏ ảnh (`null` → khóa bị bỏ), đổi về đang sống, xóa hồ sơ → ảnh cũ không còn trong `photos.ids`; `orphan_records = 0` (bản ghi ảnh không hồ sơ nào dùng); `orphan_files = 0` (Rust: tệp `photos/*.img` không có bản ghi và ngược lại; TS: bản ghi không đọc được byte) |

Chạy cả hai bản và so:

```
target/debug/lvhv-store-tool scenario tray_photo /tmp/tray-rs > rs.json
node pwa/test/tools/lvhv-store-tool.mjs scenario tray_photo > ts.json
cmp rs.json ts.json && cmp <(python3 -c 'import json,sys; print(json.dumps(json.load(open(sys.argv[1])),sort_keys=True))' rs.json) \
  <(python3 -c 'import json,sys; print(json.dumps(json.load(open(sys.argv[1])),sort_keys=True))' docs/v0.2/schema/test-vectors/tray-photo/expected-scenario.json)
```

Ngoài kịch bản, bên kiểm định có thể tự làm từng bước bằng lệnh rời: `import` một tệp → `add-photo` → `save-person` với
`memorial.tray_photo_id` → `export` → `inspect-export` (tệp có khóa và khối ảnh) → `import` tệp đó (`--replace` /
`--merge`) → `delete-person` → `open` (ảnh không còn). Tệp sao lưu tự dựng có `tray_photo_id` trỏ ảnh không có, hay
ảnh có siêu dữ liệu, phải bị `import` từ chối ở `stage = "open_backup"`.

## 4. Kiểm trên CI

`scripts/check-store-tool.sh` (job `store` của `.github/workflows/ci.yml`) chạy mọi lệnh của cả hai bản trên vector
trong `docs/v0.2/schema/test-vectors/`:
- chuyển bộ mẫu đóng băng; so với `released/expected/0.2-dev-before-attachments.json`;
- mở lại, rồi `--force-migration-failure`;
- nhập `backup.lvhv` cũ, `example-v1.lvhv`, `supplement-yc05/notice-v1.lvhv`, một tệp sai mật khẩu và 21 tệp
  `corrupt/` (TS: 4 tệp);
- `save-person`, `suggestions`, `display-capitalize`;
- `check-media` cho mọi ca của `media/manifest.json`;
- `--now`: mở cùng một kho ba lần (T0, T0 + 1 ngày, T0 + 8 ngày). Kiểm `legacy` và bản sao còn ở lần hai, đã dọn ở
  lần ba, và việc dọn chỉ bỏ `legacy`. TS chạy qua `--state`;
- `save-person` hai lần trên cùng kho (`rev` 1 → 2);
- `export` ở T0 + 1 ngày (khi kho còn `legacy`), ở T0 + 8 ngày và từ kho mới; mật khẩu ngắn bị từ chối;
- `inspect-export` trên mọi tệp xuất của cả hai bản và trên `example-v1.lvhv`: không có `legacy`, JSON chuẩn, tệp xuất
  khớp kho (bỏ `legacy`, `app`, `exported_at`). Mỗi bản đọc tệp của bản kia, và hai bản cho cùng bản rõ, cùng khối;
- YC-2026-10: `scenario tray_photo` của hai bản = `tray-photo/expected-scenario.json` (và hai bản bằng nhau); lệnh rời
  `add-photo` (ảnh EXIF bị từ chối, ảnh sạch nhận), `save-person` với `tray_photo_id` (trỏ ảnh chưa có → từ chối; sau
  `add-photo` → nhận), `export` + `inspect-export` (tệp có khóa và khối ảnh, hai bản cùng khối), `delete-person` (ảnh bị
  xóa). `scripts/check-store.sh` (bước 5/5) cũng so kịch bản này giữa hai bản.

Script kiểm kết quả với các tệp kỳ vọng đã commit, rồi so bản Rust với bản TS: tài liệu kho, byte ảnh và tệp đính
kèm, gợi ý, hồ sơ đã lưu (trừ id và thời điểm), kho ở từng mốc `--now`, tài liệu xuất (trừ `app`, `exported_at`). Chạy tại chỗ: `scripts/check-store-tool.sh`, cần `cargo`, `node` ≥
20, `python3` và `pwa/node_modules`.
