# Bộ mẫu đóng băng — bản web 0.2.0

Tạo ngày 2026-11-07 bằng `pwa/test/tools/freeze-pwa-release.mjs` trên commit `bf03033` (VERSION 0.2.0),
**qua giao diện của chính bản dựng phát hành** (`npm run build` → `pwa/dist`, Chromium, đồng hồ cố định
2026-11-07 09:00 giờ Việt Nam). Không bao giờ sửa, đổi tên hay xóa tệp nào ở đây (released/README.md).

| Tệp | Nội dung |
|---|---|
| `indexeddb.json` | Kho IndexedDB `lvhv` (phiên bản 3) chép nguyên: mọi kho con, chỉ mục, bản ghi; byte ảnh / tệp là base64 (`pwa/test/tools/idb-dump.mjs`) |
| `backup.lvhv` | Tệp sao lưu tạo bằng nút "Tạo tệp sao lưu…"; mật khẩu trong `manifest.json` |
| `store-snapshot.json` | Ảnh chụp kho dạng tài liệu (để đọc, so sánh) |
| `manifest.json` | sha256 từng tệp, cách tạo |

Dữ liệu: A người đã mất nhập ngày dương (tháng 6 nhuận âm lịch), cách gọi, giới tính, tên hiệu, nơi an táng, ảnh
chân dung, lời tưởng niệm; B người đã mất nhập theo âm lịch; C người đang sống, sinh nhật âm,
hai sự kiện (Tốt nghiệp một lần; Ngày cưới âm lịch lặp hằng năm); D người đang sống chuyển "Cập nhật: đã mất", có thông
tin báo tử và hai tệp đính kèm (ảnh tin báo tử, PDF giấy chứng tử); nhắc sao lưu 60 ngày.

Kỳ vọng (ngoài bộ mẫu): `released/expected/0.2.0-pwa.json` — stateHash của kho, stateHash sau khi khôi phục tệp
sao lưu vào kho trống. Kiểm thử: `pwa/test/upgrade.mjs` ca (d) — đóng băng (sha256), dựng lại kho từ
`indexeddb.json` rồi mở bằng bản đang kiểm, khôi phục `backup.lvhv`; khi đã có thẻ `v0.2.0` thì còn nâng cấp
thật từ bản dựng của thẻ đó qua service worker.
