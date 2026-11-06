# Tổng quan dự án — Lịch và Văn hóa Việt v0.1

Ngày: 2026-11-06. Thương hiệu: Clevix Lab. Pháp nhân: Clevix LLC. Liên hệ: khanh@clevix.vn.

## Đã làm được gì

- **Lõi lịch Rust** (`core/`, `ngochap/`): âm lịch Việt Nam UTC+7 cho năm 2000–3000, Can Chi, 24 tiết khí,
  Trực, Nhị thập bát tú, 12 thần Hoàng đạo/Hắc đạo, giờ Hoàng đạo, Nạp âm, và 121 quy tắc Ngọc Hạp có dẫn
  nguồn (49 sao tốt, 56 sao xấu, 4 ngày kỵ, 12 thần) cộng bảng tuổi xung. Chọn ngày tốt cho 16 việc. Lõi
  không dùng `unsafe` và không phụ thuộc thư viện ngoài. Có hai phương pháp tính: Hnd (mặc định) và Precise.
- **Kiểm định độc lập** (`oracle/`, Python + PyEphem): so từng ngày 2000–3000.
  Kết quả: `docs/reports/BAO-CAO-KIEM-DINH-LICH.md`.
- **Nội dung**: 427 câu ca dao, tục ngữ, thành ngữ có nguồn (`content/cadao/`, CC BY-SA 4.0).
- **Ứng dụng web cài được (PWA)** (`pwa/`, dùng bản TypeScript của lõi `tsport/`, giống lõi Rust từng byte):
  có tờ lịch, chọn ngày tốt, đổi ngày, giới thiệu; chạy không cần mạng.
- **Mã nguồn ứng dụng gốc**: Android (Kotlin/Compose), iOS/macOS (SwiftUI), Windows (Slint). Cả ba gọi cùng lõi
  qua `ffi/` (UniFFI). Có CI GitHub Actions cho từng nền tảng.
- **Rà soát**: bảo mật, duyệt mã, mô hình đe dọa, biên bản S1–S4 (`docs/reviews/`); nhật ký sửa lỗi
  (`docs/05`); biên bản cổng G1–G4 (`docs/reports/BIEN-BAN-CONG-G1-G4.md`); báo cáo năng lượng/bộ nhớ
  (`docs/reports/BAO-CAO-NANG-LUONG-BO-NHO.md`).

## Trạng thái theo nền tảng

| Nền tảng | Trạng thái | Đã kiểm | Chưa kiểm |
|---|---|---|---|
| **PWA** (mọi trình duyệt) | **Đã dựng và kiểm thử** | Dựng `pwa/dist/`; kiểm thử khói 69/69 trên Chromium; offline; 0 yêu cầu mạng ngoài; heap 2,3 MB; tải đầu 314 KB | Chưa thử trên Safari iPhone và Chrome Android thật; chưa đăng lên tên miền |
| **Android** | Có mã nguồn và CI, **chưa từng biên dịch** | Lớp dữ liệu Kotlin + thư viện Rust chạy trên JVM (Linux) | Giao diện Compose, Gradle, APK/AAB, máy thật |
| **iOS / macOS** | Có mã nguồn và CI, **chưa từng biên dịch** | Cú pháp plist/YAML | Swift, Xcode, chạy trên máy |
| **Windows** | **Đã biên dịch trên Linux**, chưa dựng trên Windows | Dựng release trên Linux, giao diện chạy dưới Xvfb, 51 MB RAM | Dựng bằng MSVC, MSI, ký số, chạy trên Windows |

## Kết quả nghiệm thu (tóm tắt, chi tiết ở biên bản G1–G4)

| Cổng | Hnd (mặc định) | Precise |
|---|---|---|
| G2 Lịch | KHÔNG ĐẠT: L2, L3, L4, L6; CHƯA ĐỦ BẰNG CHỨNG: L1 | KHÔNG ĐẠT: L4, L6; CHƯA ĐỦ BẰNG CHỨNG: L1 |
| G2 Ngọc Hạp N1–N3 | ĐẠT | ĐẠT |
| G3 Hiệu năng P1–P5 | ĐẠT | ĐẠT |
| G4 C1, S1–S4 | ĐẠT (C1 còn rủi ro: chưa soát toàn văn; S2 có điều kiện diễn giải) | như bên trái |

Mọi sai lệch lịch còn lại đều do trăng mới hoặc tiết khí rơi sát nửa đêm (dưới vài phút). Theo tiêu chí đã
khóa, đó vẫn là KHÔNG ĐẠT.

## Chủ dự án cần làm tiếp (theo thứ tự)

1. **Chọn phương pháp lịch** Hnd hay Precise (biên bản G2), rồi ký các cổng G1–G4 trong
   `docs/reports/BIEN-BAN-CONG-G1-G4.md`.
2. Quyết định các mục còn mở trong biên bản: bằng chứng L1, ghi chú L6, tên "Kim Đường", ngưỡng nhãn đánh
   giá, điểm tối thiểu 0, giấy phép Slint. (Giấy phép mã nguồn: đã chọn MIT ngày 06/11/2026.)
3. **Tạo kho GitHub** (riêng tư) và đẩy mã nguồn (gói `LichVanHoaViet-v0.1-MaNguon-DayDu.zip`). Bật
   Settings → Pages → Source = GitHub Actions. Lần đầu chạy CI, xem lỗi biên dịch Android/Apple (dự kiến
   sẽ có lỗi nhỏ) và gửi log để sửa.
4. **Phát hành PWA thử nghiệm nội bộ** trên `lich.clevix.vn` theo `DEPLOY.md` (gói PWA): xác minh tên miền
   `clevix.vn` trên GitHub, tạo bản ghi CNAME `lich`, bật HTTPS. Muốn thử trên máy trước thì chạy
   `Chay-Thu-Tren-May.ps1`.
5. Thử PWA trên điện thoại thật (Android Chrome, iPhone Safari) theo `docs/06`.
6. Cho một người **soát chữ 427 câu ca dao** theo tuyển tập in (rủi ro C1).
7. **Windows**: cài Rust (rustup-init.exe, toolchain MSVC) và Visual Studio Build Tools 2022 (C++), rồi chạy
   `Build-Windows.ps1` (gói Windows). Đo bộ nhớ và CPU theo báo cáo năng lượng §4.1.
8. **Android**: tạo khóa ký bằng `New-AndroidUploadKeystore.ps1`, đặt 4 GitHub Secrets, đăng ký Google Play
   Console (tổ chức Clevix LLC), làm theo `GOOGLE-PLAY-CHECKLIST.md`.
9. **Apple**: đăng ký Apple Developer Program cho Clevix LLC (cần D-U-N-S), dựng trên máy Mac theo
   `docs/04` §3.3, làm theo `APP-STORE-CHECKLIST.md`.
10. Đo bộ nhớ, khởi động và đánh thức nền trên thiết bị thật (báo cáo năng lượng §4.1). Ngưỡng: Android
    < 80 MB, iOS < 60 MB, khởi động < 1 s, 0 lần đánh thức nền.
