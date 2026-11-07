# Bộ dữ liệu mẫu đóng băng (04 Bổ sung B5, R14)

Thư mục này giữ dữ liệu **thật** do các bản đã phát hành (hoặc bản dựng xem trước mà chủ dự án đã dùng) tạo ra,
để mọi bản mới chứng minh được: mở dữ liệu cũ, tự chuyển sang lược đồ mới, **0 bản ghi mất, 0 trường đổi nghĩa**.

## Quy tắc đóng băng

1. Mỗi thư mục con `<phiên bản>/` là một bộ mẫu. Sau khi commit, **không bao giờ sửa, đổi tên hay xóa** tệp nào
   trong đó. Kiểm thử `fixture_is_frozen` (store/tests/released.rs) so sha256 từng tệp với `manifest.json` của bộ mẫu.
2. Bộ mẫu được tạo bằng **mã của chính bản đó** (không tạo lại bằng mã mới). `manifest.json` ghi commit và cách tạo;
   mã tạo được giữ kèm (`*.txt`, không biên dịch).
3. Mỗi bộ mẫu có, nếu nền tảng có: kho trên máy (SQLite `store/lvhv.sqlite` + `store/photos/`, …), ảnh chụp kho
   dạng tài liệu (`store-snapshot.json`, cũng là đầu vào để dựng lại kho IndexedDB của bản web), tệp sao lưu
   (`backup.lvhv`, mật khẩu trong `manifest.json`).
4. Kết quả kỳ vọng sau khi chuyển đổi nằm **ngoài** bộ mẫu, ở `expected/<phiên bản>.json` (trạng thái kho và
   `state_sha256`). Bản Rust và bản TypeScript phải cho đúng tệp này. Tệp kỳ vọng chỉ sinh lại khi cố ý đổi phép
   chuyển đổi (`LVHV_REGEN_RELEASED=1 cargo test -p lvhv-store --test released`), và thay đổi đó phải được duyệt.
5. Từ bản 0.2.0, mỗi bản phát hành thêm bộ mẫu của chính nó vào đây trước khi phát hành (04 phần phát hành).

## Bộ mẫu hiện có

| Thư mục | Tạo bởi | Nội dung |
|---|---|---|
| `0.2-dev-before-attachments/` | bản dựng 0.2 trước YC-2026-07 (commit 7338f04) | Kho phiên bản 1 có `death.notice.certificate_photo_ids` — R14 (c). Chuyển 1 → 2 đổi thành `attachments` loại `giay_chung_tu`; danh sách gốc và bản ghi `Photo` gốc giữ trong `legacy` |
| `0.2.1/windows/` | bản Windows 0.2.1 (bản Windows phát hành đầu tiên của v0.2), `lvhv-ci-check --dong-bang-ban-phat-hanh` (lõi ghi kho; README trong thư mục) | Kho SQLite lược đồ 2 + ảnh (chân dung, ảnh mâm lễ) + tệp đính kèm (ảnh, PDF), tệp sao lưu, `store-snapshot.json`. Kỳ vọng: `expected/0.2.1-windows.json` (`state`, `backup_state`). Kiểm: CI Windows R14 (d) |
| `0.2.0/pwa/` | bản web 0.2.0 (bản phát hành đầu tiên của v0.2), qua giao diện của chính bản dựng (`pwa/test/tools/freeze-pwa-release.mjs`) | Kho IndexedDB phiên bản 3 chép nguyên (`indexeddb.json`), tệp sao lưu, ảnh chụp tài liệu. Kỳ vọng: `expected/0.2.0-pwa.json` (stateHash kho; stateHash sau khi khôi phục tệp sao lưu vào kho trống). Kiểm: `pwa/test/upgrade.mjs` ca (d) |

## Kiểm thử

- Rust: `store/tests/released.rs` — đóng băng, chuyển kho SQLite, nhập tệp sao lưu cũ (Thay toàn bộ, Gộp), ca lỗi
  (quay lui, chỉ đọc, bản sao CSDL cũ còn nguyên, vẫn xuất sao lưu), từ chối kho mới hơn.
- PWA, bộ mẫu `<bản>/pwa/`: `pwa/test/upgrade.mjs` ca (d) — sha256; dựng lại kho từ `indexeddb.json` rồi mở bằng bản
  đang kiểm; khôi phục `backup.lvhv` vào kho trống; có thẻ `v<bản>` thì nâng cấp thật từ bản dựng của thẻ qua service worker.
- Windows, bộ mẫu `<bản>/windows/`: CI Windows, bước R14 (d) — với mỗi bộ mẫu: có `.msi` của bản đó trên GitHub Releases
  (và khác bản đang kiểm) thì cài nó, chép `store/` vào thư mục dữ liệu, mở bản cũ, rồi cài `.msi` mới đè lên; không có
  thì chép `store/` rồi cài `.msi` mới. Mở `LichViet.exe` đã cài, `lvhv-ci-check --kiem-mau-phat-hanh` (sha256 theo
  `manifest.json`, trạng thái trùng `state`, khôi phục `backup.lvhv` trùng `backup_state`, đặt lại thông báo), gỡ cài
  đặt, thư mục dữ liệu còn nguyên.
- PWA: `pwa/test/store/run.mjs` — dựng kho IndexedDB phiên bản 1 từ `store-snapshot.json` + ảnh, mở bằng bản mới
  (chuyển trong `onupgradeneeded`), so với cùng tệp kỳ vọng; nhập tệp sao lưu cũ; ca lỗi → chỉ đọc.
