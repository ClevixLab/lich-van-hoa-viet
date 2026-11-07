# Bộ mẫu đóng băng — bản Windows 0.2.1

Bộ mẫu của bản Windows phát hành đầu tiên của v0.2 (VERSION 0.2.1). Tạo ngày 2026-11-07 bằng
`lvhv-ci-check --dong-bang-ban-phat-hanh <thư mục> <tệp kỳ vọng>` (hàm `upgrade::freeze`, `apps/windows/src/upgrade.rs`;
bản chép: `upgrade.rs.txt`, dữ liệu mẫu ở `shots.rs.txt` › `seed_store`, `demo_tray`), dựng từ mã của nhánh
`release-windows-0.2.1` trên `main` bdc38e5 (VERSION 0.2.1) — cùng lõi, cùng mã giao diện Windows với bản phát hành.
Kho được ghi bằng lõi (`lvhv-ffi` › `LvhvStore`), không viết tay. Chạy trên Linux: kho SQLite, thư mục ảnh và tệp đính
kèm có cùng định dạng trên mọi hệ điều hành; bước R14 (d) của CI Windows mở bộ mẫu bằng `LichViet.exe` đã cài.
Không bao giờ sửa, đổi tên hay xóa tệp nào ở đây (released/README.md).

| Tệp | Nội dung |
|---|---|
| `store/lvhv.sqlite` | Kho SQLite (lược đồ 2), đã đóng hẳn (không còn `-wal`, `-shm`) |
| `store/photos/*.img` | Ảnh chân dung (có EXIF trước khi nén lại) và ảnh mâm lễ của gia đình (≤ 1600 px) |
| `store/attachments/*.bin` | Tệp đính kèm báo tử: ảnh tin báo tử (JPEG) và bản quét giấy chứng tử (PDF) |
| `backup.lvhv` | Tệp sao lưu tạo như nút "Tạo tệp sao lưu…" (`export_backup`, nền tảng `windows`, phiên bản 0.2.1); mật khẩu trong `manifest.json` |
| `store-snapshot.json` | `dataset_json` của kho (để đọc, so sánh; cũng dựng lại được kho của bản khác) |
| `upgrade.rs.txt`, `shots.rs.txt` | Mã tạo bộ mẫu (không biên dịch) |
| `manifest.json` | sha256 từng tệp (trừ chính nó), mật khẩu, cách tạo |

Dữ liệu (dữ liệu mẫu của R14 (a) + ảnh mâm lễ + cài đặt khác mặc định): 5 hồ sơ — Ông Nguyễn Văn An (đã mất, ngày mất
nhập dương lịch, tên hiệu, ảnh chân dung, lời tưởng niệm, **ảnh mâm lễ của gia đình**), Bà Trần Thị Bình (đã mất, ngày
mất nhập âm lịch), Cụ Lê Thị Cúc (đã mất, ngày âm không rõ năm, tên hiệu), Bác Đỗ Văn Minh (đã mất, có thông tin báo tử
và hai tệp đính kèm), Anh Lê Văn Cường (đang sống, sinh nhật, một sự kiện); cài đặt: nhắc trước 0/1/7 ngày lúc 06:30,
nhắc sao lưu 60 ngày.

Kỳ vọng (ngoài bộ mẫu): `released/expected/0.2.1-windows.json` — `state` (trạng thái kho, `dataset_json` bỏ `app` và
`exported_at`) và `backup_state` (trạng thái một kho trống sau khi khôi phục "Thay toàn bộ" tệp `backup.lvhv`; lúc tạo,
hai trạng thái trùng nhau). Khi lõi đổi lược đồ ở bản sau, tệp kỳ vọng được cập nhật có duyệt (04 B5), bộ mẫu thì không.

Kiểm: CI Windows, bước R14 (d) — với mỗi `released/<bản>/windows/`: chép `store/` vào thư mục dữ liệu thật, cài `.msi`
đang kiểm, mở `LichViet.exe` đã cài, rồi `lvhv-ci-check --kiem-mau-phat-hanh <bộ mẫu> <tệp kỳ vọng>` (sha256 khớp
`manifest.json`; kho không ở chế độ chỉ đọc; trạng thái trùng `state`; khôi phục `backup.lvhv` vào kho trống trùng
`backup_state`; đặt lại thông báo), gỡ cài đặt và kiểm thư mục dữ liệu còn nguyên.
