# Phát hành bản web 0.2.0 — biên bản

Ngày: 07/11/2026. Nhánh phát hành: `release-pwa-0.2.0` (PR riêng; chưa gộp). Thẻ phát hành sẽ là `v0.2.0`.

## 1. Quyết định của chủ dự án

Nguyên văn (07/11/2026, 19:27):

> "em cứ đẩy bản online 0.2 lên trước để mọi người dùng, cần chỉnh sửa gì thì sẽ update bằng version mới, trong lúc đợi
> hoàn thiện bản android, apple và windows"

Vì vậy chỉ phát hành **bản web (PWA) 0.2.0** cho mọi người dùng. Bản Android, iPhone/iPad/Mac và Windows phát hành sau,
khi hoàn thiện. Danh sách kiểm tra phát hành (08-QUY-TRINH §10) ghi "CI xanh trên cả bốn nền tảng". Lần này chỉ áp
cho nền tảng được phát hành, theo quyết định trên; các bản cài trên máy vẫn chạy CI như thường nhưng chưa phát hành.

## 2. Những gì phát hành

- **Số phiên bản:** `VERSION` = 0.2.0 (một nguồn cho mọi nền tảng). Ghi chú cho người dùng nằm ở `CHANGELOG.md`
  `## [0.2.0] — 2026-11-07`; bản web đưa các dòng đó vào `version.json` và hiện ở thanh "Đã có phiên bản mới 0.2.0".
- **Mục "Người thân":**
  - hồ sơ người đã mất và người đang sống;
  - ngày giỗ âm lịch (cả tháng nhuận), giỗ đầu / giỗ hết, hưởng thọ / hưởng dương;
  - sinh nhật dương hoặc âm, sự kiện riêng;
  - nhắc trước, danh sách "Sắp tới" 30 ngày, xem theo tuần / tháng / quý / năm;
  - xuất .ics.
- **Màn hình tưởng niệm:** bia, ảnh, bát hương ba nén và nến. Gia đình có thể thêm ảnh mâm lễ của nhà mình (YC-2026-10,
  PR #50).
- **Thông tin báo tử** và tệp đính kèm (ảnh, bản quét PDF).
- **Trang Giới thiệu:** đầu trang chỉ ghi "Phiên bản 0.2.0". Bỏ chữ "(thử nghiệm nội bộ)" mà 0.1.x còn ghi; tìm thấy khi
  xem ảnh chụp của bản dựng phát hành. Smoke khẳng định đúng chữ này. Các bản cài trên máy chưa phát hành, nên chưa rà
  chữ này ở đó.
- **Dữ liệu:**
  - chỉ trên máy, không tài khoản;
  - sao lưu ra tệp có mật khẩu và khôi phục;
  - chế độ chỉ đọc nếu nâng cấp dữ liệu không thành.

## 3. Danh sách kiểm tra (08-QUY-TRINH §10) — phần bản web

| Mục | Trạng thái | Bằng chứng |
|---|---|---|
| CI xanh | ĐẠT trên commit của PR này (ghi số commit trong mô tả PR) | Job "L7 (tsport = core) + PWA build + smoke" gồm `npm run test:smoke` và `npm run test:upgrade` |
| R14 nâng cấp tại chỗ 0.1.4 → 0.2.0 | ĐẠT trong CI. Kiểm thử chạy nâng cấp **thật**, không phải cùng một số phiên bản | `pwa/test/upgrade.mjs` ca (a): dựng bản 0.1.4 từ git (`d9650c7`), cài như người dùng, phát hành bản đang kiểm qua service worker ("Để sau" → mở lại → "Cập nhật ngay"). Khẳng định: thanh "Đã có phiên bản mới 0.2.0.", chân trang 0.1.4 → 0.2.0, cỡ chữ và giao diện tối giữ nguyên. Ca (b), (c): dữ liệu bản 0.2 dựng thử (kho v1) → 0.2.0 không mất gì; chuyển đổi lỗi → chỉ đọc, dữ liệu cũ nguyên |
| Bộ dữ liệu mẫu đóng băng của bản này | Đã thêm | `docs/v0.2/schema/test-vectors/released/0.2.0/pwa/`: `indexeddb.json` (kho IndexedDB chép nguyên), `backup.lvhv`, `store-snapshot.json`, `README.md`, `manifest.json` (sha256). Kỳ vọng ở `released/expected/0.2.0-pwa.json`. Tạo **qua giao diện của chính bản dựng phát hành** bằng `pwa/test/tools/freeze-pwa-release.mjs` |
| Bộ mẫu được dùng cho các bản sau | Đã nối | `upgrade.mjs` ca (d), chạy với mọi `released/<bản>/pwa`. (1) sha256 còn nguyên. (2) Dựng lại kho từ `indexeddb.json`, mở bằng bản đang kiểm: cùng phiên bản kho thì stateHash phải giống từng byte. (3) Khôi phục `backup.lvhv` vào kho trống. (4) Khi đã có thẻ `v0.2.0`: dựng bản của thẻ, đặt dữ liệu vào kho của bản cũ rồi nâng cấp thật qua service worker. Ở PR này (4) được bỏ qua và ghi rõ, vì chưa có thẻ |
| Dữ liệu cũ, tệp sao lưu cũ vẫn mở được | ĐẠT | Ca (b), (c); `pwa/test/store/run.mjs` (bộ mẫu `0.2-dev-before-attachments`, R6, R6v2) |
| Ảnh chụp đã xem | Đã xem trên bản dựng phát hành | Danh sách ở mô tả PR (tờ lịch 360/1280, Người thân, tưởng niệm, biểu mẫu, sao lưu, chỉ đọc, nâng cấp) |
| Dung lượng tải và bộ nhớ trong ngưỡng | Bản web: ĐẠT trong smoke | P3 lần tải đầu, P6 tải lười, heap đo trong smoke (`pwa/test/metrics.json` của CI) |
| Ghi chú phát hành bằng lời thường | Đã viết | `CHANGELOG.md` `[0.2.0]` |
| Đường lùi | Có (§5) | Thẻ `v0.1.4` còn; `pwa-pages.yml` có ở thẻ đó |

## 4. Giới hạn đã biết

- **Bản web chưa có thao tác đổi người đã mất thành người đang sống.** Chỉ có "Cập nhật: đã mất" theo chiều ngược lại.
  Câu "Ảnh mâm lễ cũng sẽ bị xóa." của 08 §16.2 sẽ thêm khi có thao tác này.
- **Gói ảnh mâm lễ của Clevix đang rỗng** (`content/memorial/trays/manifest.json`). Ngày giỗ chỉ hiện bát hương và nến,
  trừ khi gia đình tự thêm ảnh. Thêm ảnh Clevix đã duyệt chỉ cần phát hành bản mới.
- **Android, iPhone/iPad/Mac, Windows chưa phát hành** (§1). Dữ liệu trên bản web không tự chuyển sang các bản đó; khi
  có, dùng tệp sao lưu.
- **Chưa đo P8 (Argon2id) và P9 (bộ nhớ native) trên thiết bị thật.** P8 trên điện thoại tầm trung / iPhone và P9 cần
  máy thật; smoke chỉ đo trong Chromium trên máy CI. Cổng G3 (chủ dự án) còn mở cho các mục này.
- **Trình duyệt không có hộp "Lưu tệp" của hệ điều hành** (Safari, Firefox, Chrome Android): trang không biết người dùng
  có hủy việc lưu tệp sao lưu hay không (08 §12l, MA3C-30-03).

## 5. Đường lùi

Đăng lại bản 0.1.4: GitHub → Actions → **pwa-pages** → "Run workflow", chọn ref **thẻ `v0.1.4`**. Tệp `pwa-pages.yml`
có ở thẻ đó (`workflow_dispatch`), dựng `pwa/dist` của 0.1.4 và đăng lên GitHub Pages.

Ảnh hưởng tới người dùng:
- Bản 0.1.4 không mở kho IndexedDB `lvhv`: mục "Người thân" chưa có, và mã lưu trữ chỉ nạp khi gọi.
- Vì vậy dữ liệu người thân đã nhập trên 0.2.0 **vẫn nằm nguyên trên máy**, và hiện lại khi đăng lại bản 0.2.x.
- Người dùng vẫn nên giữ tệp sao lưu.

Sau khi lùi, sửa lỗi rồi phát hành 0.2.1, không đăng lại 0.2.0.

## 6. Việc còn lại sau khi gộp PR phát hành

1. Gộp PR, đặt thẻ `v0.2.0` trên commit gộp. Thẻ kích hoạt `pwa-pages` đăng bản web.
2. Mở trang đã đăng và kiểm: chân trang "Phiên bản 0.2.0"; người đang dùng 0.1.4 thấy thanh "Đã có phiên bản mới 0.2.0."
3. Từ bản sau, `upgrade.mjs` ca (d)(4) tự chạy nâng cấp thật từ thẻ `v0.2.0`.
