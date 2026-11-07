# 10 — Công cụ kiểm định hộp đen `lvhv-store-tool`

Công cụ dòng lệnh cho bên kiểm định độc lập. Nó mở kho, nhập sao lưu, lưu hồ sơ và in danh sách gợi ý, rồi in kết
quả ra dạng JSON để so với kỳ vọng mà không phải đọc mã nguồn. Công cụ làm theo yêu cầu ở mục 7 của
`docs/reports/BAO-CAO-KIEM-DINH-V02-B5.md` (PR #33), gồm ba việc:
- (i) mở thư mục kho SQLite, in tài liệu kho và trạng thái chuyển đổi;
- (ii) nhập tệp sao lưu tùy ý, in toàn bộ kho sau khi nhập;
- (iii) gọi lưu hồ sơ và danh sách gợi ý cách gọi, quan hệ.

Có hai bản, cùng lệnh, cùng dạng kết quả:

| Bản | Tệp | Gọi |
|---|---|---|
| Rust (lõi, như ứng dụng Android/Apple/Windows) | `ffi/src/bin/lvhv-store-tool.rs` | `lvhv_store` (mở/chuyển đổi kho, nhập, ảnh chụp kho) và `lvhv_ffi::LvhvStore::save_person_json` (đúng đường lưu hồ sơ của ứng dụng) |
| TypeScript (PWA) | `pwa/test/tools/lvhv-store-tool.mjs` | API công khai của `pwa/src/store/index.ts`, IndexedDB giả lập (`fake-indexeddb`) |

Bản Rust nằm trong crate `lvhv-ffi`, không nằm trong `lvhv-store`. Lý do: phần lưu hồ sơ của ứng dụng nằm ở
`save_person_json` của lớp FFI. Phần này gắn id/`rev`/thời điểm, chuẩn hóa thông tin báo tử và viết hoa chữ đầu, rồi
mới gọi kho. `lvhv-store` không phụ thuộc được vào `lvhv-ffi`.

## 1. Dựng

```
# Rust — bản thường
cargo build --locked -p lvhv-ffi --bin lvhv-store-tool
# Rust — có móc kiểm thử, cần cho open --force-migration-failure
cargo build --locked -p lvhv-ffi --features test-hooks --bin lvhv-store-tool
# → target/debug/lvhv-store-tool  (thêm --release nếu muốn; bản dev đã tối ưu argon2)

# TypeScript
(cd pwa && npm ci)
node pwa/test/tools/lvhv-store-tool.mjs …
```

Tính năng `test-hooks` không bao giờ được bật khi dựng ứng dụng. Không có nó, `--force-migration-failure` báo sai cú
pháp (mã thoát 2) và không làm gì.

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

### 3.1 `open <thư-mục> [--force-migration-failure]`

Rust: mở (hoặc tạo) kho SQLite trong `<thư-mục>`. Kho phiên bản cũ được **chuyển đổi tại chỗ**, nên phải chép bộ
mẫu ra chỗ khác trước khi mở. Không bao giờ mở thẳng thư mục trong `test-vectors/released/`.

```
cp -r docs/v0.2/schema/test-vectors/released/0.2-dev-before-attachments/store /tmp/kho
target/debug/lvhv-store-tool open /tmp/kho
```

In ra:
- `migration_status`: `state` (`ok` / `migrated` / `read_only`), `from_version`, `to_version`, `backup_path`
  (bản sao `lvhv.sqlite.pre-v2.bak`), `error`;
- `store`;
- `files_before`, `files_after`: mọi tệp trong thư mục, kèm cỡ và sha256, trước và sau khi mở.

Kho của bản mới hơn: `ok: false`, `error.kind = "NewerSchema"`.

`--force-migration-failure` (cần `test-hooks`) ép bước chuyển đổi lỗi **sau** khi mọi bước đã chạy trong giao
dịch, ngay trước `COMMIT`. Như vậy quan sát được đúng đường lỗi của 04 B5 mục 3:
- giao dịch quay lui: `user_version` vẫn là 1, dữ liệu cũ còn nguyên;
- bản sao `.pre-v2.bak` còn trên đĩa;
- `state = "read_only"`.

Ở chế độ chỉ đọc, công cụ thử thêm và in kết quả trong `read_only_probe`:
- `put_settings`, `put_person`, `delete_person` phải lỗi `ReadOnly`;
- `export_backup` phải thành công.

Kho mới tạo (phiên bản 0) không bị móc này ảnh hưởng.

TS: `<thư-mục>` là **thư mục bộ mẫu** theo bố cục `test-vectors/released/<bản>/`, gồm `store-snapshot.json` và
`store/photos/<id>.img`. Công cụ dựng kho IndexedDB phiên bản 1 từ thư mục đó, như bản dựng trước YC-2026-07, rồi mở
bằng `openStore` của bản này. Thư mục không có `store-snapshot.json`, hoặc `-`, cho kho trống. Không ghi gì vào thư
mục. Ở TS, `--force-migration-failure` không cần móc trong mã PWA: công cụ làm lệnh ghi `meta.store_version` cuối
giao dịch nâng cấp ném lỗi ở tầng IndexedDB. Giao dịch nâng cấp vì vậy quay lui, và kho được mở lại ở chế độ chỉ
đọc. `to_version` của TS là phiên bản IndexedDB (3), không phải 2.

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

### 3.3 `save-person <thư-mục> <person.json>`

Lưu một `Person` như màn hình hồ sơ. `id` rỗng thì sinh id mới. Công cụ gắn `created_at`/`updated_at`/`rev`, viết
hoa chữ đầu `danh_xung` và `quan_he` (YC-2026-07), bỏ khóa `gioi_tinh: null`, tính trường dẫn xuất, kiểm V1–V8, rồi
in **bản đã lưu** (`saved`) đọc lại từ kho.
- Rust gọi `LvhvStore::save_person_json` trên kho ở `<thư-mục>`, kho được tạo nếu chưa có.
- TS làm như `src/people/forms.ts`: `buildPerson(normalizePersonNames(…))`, rồi `validate.person`, rồi `putPerson`.
  Ở TS, `<thư-mục>` là thư mục bộ mẫu, hoặc `-` cho kho trống.

```
target/debug/lvhv-store-tool save-person /tmp/kho-moi nguoi.json
```

### 3.4 `suggestions <deceased|living> [nam|nu]`

In `danh_xung` (`danh_xung_suggestions`: 22 mục cho người mất, 17 mục cho người sống; có `nam`/`nu` thì sắp theo
giới tính) và `quan_he` (`quan_he_suggestions`).

### 3.5 `display-capitalize <chuỗi>`

In `display_capitalize(<chuỗi>)`: quy tắc viết hoa khi hiển thị cách gọi/quan hệ đã lưu.

## 4. Kiểm trên CI

`scripts/check-store-tool.sh` (job `store` của `.github/workflows/ci.yml`) chạy mọi lệnh của cả hai bản trên vector
trong `docs/v0.2/schema/test-vectors/`:
- chuyển bộ mẫu đóng băng; so với `released/expected/0.2-dev-before-attachments.json`;
- mở lại, rồi `--force-migration-failure`;
- nhập `backup.lvhv` cũ, `example-v1.lvhv`, `supplement-yc05/notice-v1.lvhv`, một tệp sai mật khẩu và 21 tệp
  `corrupt/` (TS: 4 tệp);
- `save-person`, `suggestions`, `display-capitalize`.

Script kiểm kết quả với các tệp kỳ vọng đã commit, rồi so bản Rust với bản TS: tài liệu kho, byte ảnh và tệp đính
kèm, gợi ý, hồ sơ đã lưu (trừ id và thời điểm). Chạy tại chỗ: `scripts/check-store-tool.sh`, cần `cargo`, `node` ≥
20, `python3` và `pwa/node_modules`.
