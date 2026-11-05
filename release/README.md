# Phát hành v0.1 — Lịch và Văn hóa Việt (Clevix Lab · Clevix LLC)

Ngày: 2026-11-06. Thư mục này chứa các gói phát hành. Git chỉ theo dõi `README.md`; zip, PDF và tệp tạm nằm
trong `.gitignore`. Tạo lại toàn bộ bằng `release/tools/dong-goi.sh`, sau khi đã chạy `cd pwa && npm run
build && node test/smoke.mjs`, `cargo test --workspace --release` và `scripts/check-parity.sh`. Kiểm tổng
băm: `sha256sum -c SHA256SUMS.txt`.

**Kiểm tra trước khi đóng gói (2026-11-06, máy 2 vCPU x86_64 Linux, 7 GB RAM):** `cargo test --workspace --release`
ĐẠT; test dài `-- --ignored` ĐẠT (15 phút); `scripts/check-parity.sh` ĐẠT (0 sai lệch, cả Hnd và Precise);
smoke PWA ĐẠT 69/69. Gói mã nguồn đầy đủ đã được giải nén ra thư mục sạch và chạy lại: test, smoke và
parity đều ĐẠT, `app-*.js` giống hệt từng byte. Gói Windows giải nén ra thư mục sạch và dựng release thành
công trên Linux.

| Gói | Dung lượng | Nội dung chính | Trạng thái |
|---|---|---|---|
| `LichVanHoaViet-v0.1-PWA.zip` | 1,6 MB (28 tệp) | `dist/` sẵn để đưa lên máy chủ tĩnh; `DEPLOY.md` (GitHub Pages + `lich.clevix.vn`, chạy thử trên máy); `Chay-Thu-Tren-May.ps1` (phục vụ `dist/` tại http://localhost:8080 bằng `py -3`, nếu không có thì dùng `python`; mở trình duyệt); hướng dẫn sử dụng (md + PDF); `bao-cao/` 5 PDF | **Đã dựng và kiểm thử** (Chromium). Phương pháp lịch Hnd. Chưa thử trên điện thoại thật |
| `LichVanHoaViet-v0.1-Android.zip` | 1,9 MB (155 tệp) | `core/`, `ngochap/`, `ffi/` (kèm `generated/`), `content/`, `apps/android/`, `ci.yml`, `android.yml`, `New-AndroidUploadKeystore.ps1`, `GOOGLE-PLAY-CHECKLIST.md`, `docs/04`, PDF | **Chưa có APK.** Mã nguồn + CI; giao diện chưa từng biên dịch |
| `LichVanHoaViet-v0.1-Apple.zip` | 2,0 MB (133 tệp) | như trên với `apps/apple/`, `apple.yml`, `APP-STORE-CHECKLIST.md` | **Chưa có bản dựng.** Swift chưa từng biên dịch; cần máy Mac hoặc runner macOS |
| `LichVanHoaViet-v0.1-Windows.zip` | 2,0 MB (113 tệp) | `apps/windows/` (không có `target/`), lõi, `windows.yml`, `Build-Windows.ps1` (ở gốc gói), `MICROSOFT-STORE-CHECKLIST.md`, PDF | **Đã biên dịch trên Linux**, chưa dựng trên Windows; chưa có `.exe` |
| `LichVanHoaViet-v0.1-TaiLieu.zip` | 3,3 MB (51 tệp) | Mọi tài liệu `docs/` (md), quy tắc/nguồn nội dung, 3 danh sách nộp cửa hàng, `DEPLOY-PWA.md`; `pdf/` 14 PDF | Hoàn tất |
| `LichVanHoaViet-v0.1-MaNguon-DayDu.zip` | 1,1 MB (327 tệp) | Toàn bộ mã nguồn + `.github/` + `docs/` + công cụ `release/`. Không có `target/`, `node_modules/`, `dist/`, `pwa/src/gen`, ảnh test. Tệp vàng lớn (`core/out`, `oracle/out`) thay bằng mẫu nhỏ `core/out-mau/`, `oracle/out-mau/`; sinh lại bằng `lvhv-dump` (CI tự làm) | Đã kiểm dựng lại từ thư mục sạch |

Dung lượng chính xác (byte) và tổng băm nằm ở `SHA256SUMS.txt` và trong danh sách tệp ở trên.

## Tài liệu mới của đợt phát hành
- `docs/reports/BAO-CAO-NANG-LUONG-BO-NHO.md`: số đo (ĐO) và ước tính (ƯỚC TÍNH, CHƯA ĐO), kèm quy trình
  đo trên thiết bị.
- `docs/reports/BIEN-BAN-CONG-G1-G4.md`: kết luận từng tiêu chí, kèm dòng ký duyệt.
- `docs/06-HUONG-DAN-SU-DUNG.md`, `docs/07-TONG-QUAN-DU-AN.md`.

## Công cụ (`release/tools/`)
`so-sanh-precise-oracle.py` (L1–L6 cho bất kỳ thư mục `core/out*`), `do-nang-luong-pwa.mjs`,
`doi-chung-giam-toc.mjs`, `maxrss.c`, `do-windows-xvfb.sh`, `md2pdf.py`, `dong-goi.sh`; kết quả thô `ket-qua-*`.

## Việc của chủ dự án
Xem `docs/07-TONG-QUAN-DU-AN.md` mục "Chủ dự án cần làm tiếp". Việc đầu tiên: chọn phương pháp lịch và ký
biên bản cổng G1–G4.
