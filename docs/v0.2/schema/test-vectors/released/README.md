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

## Kiểm thử

- Rust: `store/tests/released.rs` — đóng băng, chuyển kho SQLite, nhập tệp sao lưu cũ (Thay toàn bộ, Gộp), ca lỗi
  (quay lui, chỉ đọc, bản sao CSDL cũ còn nguyên, vẫn xuất sao lưu), từ chối kho mới hơn.
- PWA: `pwa/test/store/run.mjs` — dựng kho IndexedDB phiên bản 1 từ `store-snapshot.json` + ảnh, mở bằng bản mới
  (chuyển trong `onupgradeneeded`), so với cùng tệp kỳ vọng; nhập tệp sao lưu cũ; ca lỗi → chỉ đọc.
