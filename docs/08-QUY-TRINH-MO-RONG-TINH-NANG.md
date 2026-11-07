# Quy trình mở rộng tính năng và thêm mô-đun

Trạng thái: **ÁP DỤNG từ 2026-11-07** cho mọi tính năng mới, mô-đun mới và mọi thay đổi người dùng nhìn thấy được.
Chủ dự án yêu cầu (07/11/2026): mỗi lần mở rộng đều phải được thiết kế, phân tích và đánh giá yêu cầu, viết đặc tả,
thiết kế quy trình, quản trị rủi ro và phân công việc rõ ràng cho từng vai, theo cách các công ty phần mềm chuyên nghiệp
đang làm.

Tài liệu này bổ sung cho hợp đồng kiến trúc (docs/01) và tiêu chí nghiệm thu (docs/02, docs/v0.2/04); nó không thay
thế chúng. Lỗi nhỏ không đổi hành vi (chính tả, căn lề, lỗi rõ ràng) đi theo đường sửa lỗi ở §9.

## 1. Vòng đời một yêu cầu

| Bước | Tên | Đầu ra bắt buộc | Ai làm | Cổng / ai ký |
|---|---|---|---|---|
| 0 | Tiếp nhận | Phiếu yêu cầu: mã `YC-<năm>-<số>`, nguyên văn lời chủ dự án, ngày, mức ưu tiên | Điều phối viên | — |
| 1 | Phân tích yêu cầu | Câu chuyện người dùng (§3), phạm vi có / không có, yêu cầu phi chức năng, phân tích ảnh hưởng tới phần đã có, câu hỏi còn mở | Điều phối viên | Chủ dự án trả lời câu hỏi mở |
| 2 | Thiết kế | Thiết kế dữ liệu và kiến trúc (ghi quyết định theo mẫu ADR §4), đặc tả giao diện (§5), nguồn nội dung văn hóa nếu có | Vai thiết kế giao diện + vai lõi | Duyệt thiết kế (§6) |
| 3 | Đặc tả và tiêu chí | Đặc tả thuật toán / hành vi đủ để một bên khác cài lại; tiêu chí nghiệm thu đo được, **khóa trước khi viết mã** | Điều phối viên + vai lõi | **Cổng G1** — chủ dự án ký |
| 4 | Rủi ro | Cập nhật sổ rủi ro (§7) | Điều phối viên | Đi cùng G1 |
| 5 | Kế hoạch và phân công | Bảng chia việc, ma trận trách nhiệm, phụ thuộc, việc chạy song song, bản giao việc từng vai theo mẫu §8 | Điều phối viên | Đi cùng G1 |
| 6 | Thực hiện | Nhánh và PR riêng từng vai; kiểm thử tự động; ảnh chụp màn hình | Các vai thực hiện | CI xanh |
| 7 | Rà soát | Duyệt mã bởi một vai không viết mã đó; duyệt giao diện trên ảnh chụp | Vai rà soát | Không còn lỗi mức "chặn" |
| 8 | Kiểm định độc lập | Bên kiểm định độc lập tự cài từ đặc tả, nộp kết quả trước khi so | Bên kiểm định độc lập | **Cổng G2–G4** — chủ dự án ký biên bản |
| 9 | Phát hành | Danh sách kiểm tra phát hành (§10), nhật ký thay đổi, đường lùi | Điều phối viên | Chủ dự án đồng ý phát hành |
| 10 | Hậu kiểm | Theo dõi lỗi sau phát hành; nhìn lại (điều gì tốt, điều gì cần đổi) ghi vào docs/05 | Điều phối viên | — |

Quy tắc không đổi:
- Tiêu chí đã khóa không được nới sau khi thấy kết quả. Mọi thay đổi sau khi thấy kết quả là một **Bổ sung** có ngày,
  ghi rõ "SAU KHI THẤY KẾT QUẢ" và lý do.
- Không gộp PR khi chưa qua cổng tương ứng. Bước 6 có thể bắt đầu khi bước 2–3 đã có bản nháp, nhưng PR chỉ được
  gộp sau khi thiết kế và tiêu chí đã khóa.
- Tài liệu và hồ sơ không ghi tên sản phẩm hay nhà cung cấp trí tuệ nhân tạo; bên kiểm định được gọi là "bên kiểm định
  độc lập" — một bên không tham gia xây dựng đặc tả.

## 2. Phân tích yêu cầu — các câu phải trả lời

1. Ai dùng, trong hoàn cảnh nào, để làm gì (câu chuyện người dùng)?
2. Có gì trong phạm vi, có gì để lại cho sau?
3. Dữ liệu nào được tạo ra, lưu ở đâu, ai thấy được; có cần tài khoản hay mạng không?
4. Tính năng ảnh hưởng tới gì đang có: dữ liệu cũ, tệp sao lưu, hiệu năng, quyền ứng dụng, dung lượng tải, tiêu chí đã
   đạt?
5. Yêu cầu phi chức năng: riêng tư, ngoại tuyến, hiệu năng (P*), bảo mật (S*), trợ năng, bốn nền tảng, ngôn ngữ.
6. Nội dung văn hóa (phong tục, cách gọi, nghi lễ): nguồn nào, có ý kiến khác nhau không, chọn cách phổ biến và ghi
   nguồn.
7. Câu hỏi nào chỉ chủ dự án trả lời được? Liệt kê kèm phương án đề xuất và mặc định nếu chưa trả lời.

## 3. Mẫu câu chuyện người dùng

`US-<mô-đun>-<số>`: **Là** <ai>, **tôi muốn** <làm gì>, **để** <được gì>.
Tiêu chí chấp nhận viết theo dạng **Cho** <bối cảnh> / **Khi** <hành động> / **Thì** <kết quả quan sát được>.
Mỗi câu chuyện gắn với ít nhất một tiêu chí nghiệm thu (R*, P*, S*) hoặc một ca kiểm thử giao diện.

## 4. Mẫu ghi quyết định thiết kế (ADR)

`ADR-<số>` — Bối cảnh · Các phương án (ưu, nhược) · Quyết định · Hệ quả · Ai quyết, ngày. Quyết định của chủ dự án
ghi nguyên văn.

## 5. Đặc tả giao diện — nội dung tối thiểu

Kiến trúc thông tin và điều hướng; từng màn hình (mục đích, lối vào, từng trường với kiểm tra và lời báo lỗi, các
trạng thái trống / đang tải / lỗi / xác nhận); luồng người dùng đánh số; toàn bộ câu chữ hiển thị; trợ năng (bàn phím,
thứ tự tiêu điểm, nhãn, độ tương phản, giảm chuyển động, cỡ chữ); bố cục 360 px và 1280 px, sáng và tối; khung dây
(wireframe); bảng nối câu chuyện → màn hình → kiểm thử.

## 6. Duyệt thiết kế và duyệt mã

- **Duyệt thiết kế:** một vai không viết đặc tả giao diện đọc đặc tả và ảnh chụp, chấm từng mục §5, ghi lỗi theo mức
  chặn / nên sửa / góp ý.
- **Duyệt mã:** một vai không viết mã đó đọc PR: đúng đặc tả, kiểm thử đủ, bảo mật, riêng tư, hiệu năng, khả năng bảo
  trì. Lỗi mức chặn phải sửa trước khi gộp.
- **Ảnh chụp màn hình là bắt buộc** trước khi phát hành bất kỳ thay đổi giao diện nào, trên mọi nền tảng bị ảnh hưởng.

## 7. Sổ rủi ro

Mỗi rủi ro: mã, mô tả, khả năng (thấp / vừa / cao), tác động (thấp / vừa / cao), dấu hiệu sớm, cách giảm, người theo
dõi, trạng thái. Rủi ro mức cao × cao phải có cách giảm trước cổng G1.

## 8. Mẫu giao việc cho từng vai

Mỗi bản giao việc ghi đủ:
1. Mục tiêu và lý do (nguyên văn yêu cầu của chủ dự án nếu có).
2. Tài liệu phải đọc trước.
3. Thư mục được ghi; thư mục **không được đọc** (với vai kiểm định).
4. Nhánh, cách mở PR, điều cấm (không tự gộp, không sửa tiêu chí đã khóa).
5. Đầu ra cụ thể và định nghĩa "xong" (Definition of Done): kiểm thử, ảnh chụp, số đo, tài liệu cập nhật.
6. Phụ thuộc vào vai khác và cách nhận tín hiệu (ví dụ tệp báo "đặc tả đã sẵn sàng").
7. Cách báo cáo: ngắn, trung thực, ghi rõ ĐẠT / KHÔNG ĐẠT / CHƯA ĐỦ BẰNG CHỨNG và việc chưa làm.

Các vai chuẩn: điều phối viên; lõi (Rust + bản TypeScript); giao diện từng nền tảng (PWA, Android, Apple, Windows);
thiết kế giao diện; nội dung văn hóa; rà soát mã; rà soát bảo mật; bên kiểm định độc lập. Việc không phụ thuộc nhau
được chạy song song để không ai phải chờ.

## 9. Đường sửa lỗi

Lỗi do người dùng báo: ghi vào docs/05 (hiện tượng, nguyên nhân gốc, cách sửa, kiểm thử chống tái diễn); sửa trên
nhánh riêng; thêm kiểm thử tái hiện lỗi **trước** khi sửa; ảnh chụp nếu là lỗi giao diện; phát hành bản vá.

## 10. Danh sách kiểm tra phát hành

CI xanh trên cả bốn nền tảng; cổng tương ứng đã ký; **R14 nâng cấp tại chỗ ĐẠT trên mọi nền tảng phát hành (không mất dữ liệu của khách hàng)**; lưu bộ dữ liệu mẫu đóng băng của bản này vào `test-vectors/released/`; ảnh chụp đã xem; số phiên bản và nhật ký thay đổi; dữ liệu cũ và
tệp sao lưu cũ vẫn mở được; dung lượng tải và bộ nhớ trong ngưỡng; ghi chú phát hành bằng lời thường; đường lùi (bản
trước vẫn tải được).

Bản web (từ 0.2.0, `docs/reports/PHAT-HANH-PWA-0.2.0.md`):
- R14 nâng cấp tại chỗ là `pwa/test/upgrade.mjs` ca (a). Kiểm thử phải đi **từ bản web đang phát hành lên đúng số phiên
  bản mới**: khẳng định chân trang cũ → mới và thanh "Đã có phiên bản mới X.Y.Z". Cùng một số phiên bản thì không tính.
- Bộ mẫu đóng băng: `cd pwa && npm run build && node test/tools/freeze-pwa-release.mjs` trên commit phát hành, tạo
  `released/<bản>/pwa/`. Ca (d) của `upgrade.mjs` tự dùng mọi bộ mẫu đó cho các bản sau.
- Đường lùi: chạy `pwa-pages` (workflow_dispatch) trên thẻ của bản trước.

Bản Android (từ 0.2.1):
- Khóa ký: một lần, trên máy Windows của chủ dự án, `scripts/tao-khoa-ky-android.ps1` (tạo khóa, in dấu vân tay,
  đặt 4 secret `LVHV_ANDROID_*`). Mọi bản sau phải ký bằng cùng khóa, nếu không người dùng không cài đè được.
- Đăng: `android.yml` chạy tay với `publish_release` trên `main` (hoặc thẻ `v*`). Job `publish` chạy sau `build`, `ui`,
  `upgrade` (R14 trên trình giả lập) và chỉ đăng bản **đã ký**: `LichViet-<VERSION>.apk`, `.aab`, tệp SHA-256, ghi chú
  từ mục `[<VERSION>]` của CHANGELOG. Chưa ký thì job đỏ, không đăng gì.
- Bộ mẫu đóng băng: `released/<bản>/android/` (kho `store/`, `shared_prefs/`, `backup.lvhv`, `manifest.json`) và
  `released/expected/<bản>-android.json`. `apps/android/ci/upgrade-test.sh` tự dùng mọi bộ mẫu đó cho các bản sau
  (op `verify-released`). Bộ 0.2.1 tạo bằng lõi 0.2.1 qua API FFI mà ứng dụng gọi, không qua giao diện trên máy thật
  (README của bộ mẫu). Từ bản sau, nên tạo bằng chính APK phát hành trên trình giả lập.
