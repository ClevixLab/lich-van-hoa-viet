# Duyệt PR #57 (Windows) — vòng sửa theo `DUYET-VONG-SUA-PR25.md` (DV25-01, 03, 04, 05, 06)

Trạng thái: **XONG** — 2026-11-07. Kết luận ở §4.

Bên duyệt: một bên không tham gia viết mã của PR #57; không sửa nhánh đang duyệt.

| Mục | Giá trị |
|---|---|
| PR | #57 `v0.2-w3-windows-dv25` → `main` |
| Đầu mã được duyệt | `935cc56` (gộp `main` vào nhánh); phần sửa nằm ở commit `278f8ca` |
| Đầu nhánh | `03ef952` — chỉ thêm ảnh chụp của CI (run 341, `[skip ci]`) |
| So với | `git diff origin/main...935cc56` (6 tệp, ngoài ảnh chụp: `people.rs`, `ui_tests_people.rs`, `people.slint`, `wix/main.wxs`, `docs/04-BUILD-NATIVE.md`, `docs/v0.2/08`) |
| CI | `windows` #341 (run 42379196893) trên `935cc56`: mọi bước `success` — `cargo test` 30/30 (gồm `ui_tests_people::*`), chụp màn hình, `.msi`, thông báo sau khi cài, R14, "LichViet.exe phát hành không có chế độ kiểm thử" ("Đạt: không có chuỗi cờ CI (7 cờ) …, .zip chỉ có: LichViet.exe, OFL.txt"). `ci` (Rust fmt/clippy/test, cargo-deny, Rust ≡ TS, lưu trữ + sao lưu, L7/PWA, gói ảnh mâm lễ): `success` |
| Ảnh | `apps/windows/screenshots/` ở `03ef952`. So với `main`, chỉ đổi `06-gioi-thieu*`, `06b-*`, `nt-21b-*` (khác vài trăm byte trở xuống — ngày dựng, khói động) và hai tệp `.txt`. Các ảnh hộp thoại (`nt-09c-hoi-mo-tep`, `nt-18-sao-luu`, `nt-20-hop-tao-sao-luu`, `nt-20b-hop-khoi-phuc`, `a3-nt-20-*`) **trùng từng byte** với `main` |

Căn cứ: `docs/reviews/DUYET-VONG-SUA-PR25.md` §3; `docs/v0.2/08` §14; `docs/04-BUILD-NATIVE.md`.

Giới hạn: không chạy được Windows. Kết luận về hành vi trên Windows lấy từ mã, nhật ký CI run 341 và ảnh do CI chụp.

Mức: **chặn** / **nên sửa** / **góp ý**. Mã phát hiện mới: `P57-<số>`.

## 1. Các mục được giao

| Mã | Mức cũ | Kết quả | Bằng chứng |
|---|---|---|---|
| DV25-01 | nên sửa | **ĐÃ SỬA** (cách 1: sửa tài liệu) | `docs/04-BUILD-NATIVE.md` đoạn "Gỡ cài đặt": câu "dữ liệu được giữ" ở trang kết thúc **chỉ hiện khi cài mới hoặc nâng cấp**; khi gỡ (chạy lại `.msi` → Remove, hay Cài đặt Windows) không hiện, vì `ExitDialog` của WixUI có điều kiện `NOT Installed`. Chú thích khối `<UI Id='LvhvUI'>` trong `wix/main.wxs` ghi cùng ý (không dấu, đúng quy ước của tệp). Ghi rõ người dùng biết dữ liệu được giữ qua trang Giới thiệu và Người thân › Sao lưu › "Dữ liệu trên máy này", và lý do chưa thêm điều khiển riêng cho trang gỡ (CI cài/gỡ im lặng). 08 §14 #19 cũng nhắc. Đúng với mã nguồn WixUI đã dẫn ở vòng trước |
| DV25-03 | góp ý | **ĐÃ SỬA** (phần tối thiểu được đề nghị) | `people.rs`: phần chạy trên luồng giao diện của `bg` tách thành `deliver(w, r, done) -> bool` (tìm lại cửa sổ + `PS_TLS`, gọi `done`; trả `false` khi cửa sổ đã đóng hoặc mô-đun chưa nối); kết quả mở tệp sao lưu tách thành `backup_opened`, đặt `restore-busy = false` ở **cả hai** nhánh. Bản phát hành gọi đúng `deliver` trong `invoke_from_event_loop`. Kiểm thử `theo_ky_ics_sao_luu_the_nhac`: `restore-busy` về `false` sau nhánh lỗi (sai mật khẩu) và nhánh thành công; đặt `restore-busy = true` rồi bấm "Mở tệp" → không có hộp nào mở (phép kiểm có ý nghĩa: nếu nút không khóa, mật khẩu đúng sẽ mở hộp `khoi-phuc`); `deliver(Err("thử"), backup_opened)` qua `PS_TLS` trả `true`, mở khóa nút, hiện lỗi. Còn lại: `thread::spawn` + `invoke_from_event_loop` vẫn không chạy trong kiểm thử (xem P57-02) |
| DV25-04 | góp ý | **ĐÃ SỬA** | 08 §14 #17: thêm hộp "Mở tệp bằng ứng dụng khác?" (BM3-02, Hủy mặc định, Enter và Esc = Hủy, ảnh `nt-09c`) trước bước chép ra `tmp\`. Dòng mới #19 "Sao lưu › Dữ liệu trên máy này" (đường dẫn, gỡ không xóa, cách xóa hẳn, "Mở thư mục dữ liệu", không có nút xóa toàn bộ, ảnh `nt-18`) |
| DV25-05 | góp ý | **ĐÃ SỬA** | `people.slint` `Dialog`: toàn bộ `VerticalLayout` của hộp (tiêu đề, dòng, ô mật khẩu, lựa chọn khôi phục, nút) nay nằm **trong** `fs := FocusScope`, nên phím ô nhập không dùng (Esc là ký tự điều khiển, `TextInput` bỏ qua) nổi lên tới `key-pressed` của hộp. Enter trong ô một dòng vẫn do ô nhập nhận (`accepted`), và hộp sao lưu có `dlg-default-cancel = false` nên Enter không đóng hộp — đúng. Kiểm thử: Enter và Esc ở hộp `mo-tep` đóng hộp, không mở tệp, không tạo `tmp\`; bấm vào ô "Mật khẩu (ít nhất 8 ký tự)" rồi Esc → hộp đóng. Bố cục không đổi: ảnh các hộp trùng từng byte với `main` |
| DV25-06 | góp ý | **ĐÃ SỬA** (bằng đặc tả, như đề nghị) | 08 §14 #12: mở từ URI vào thẳng trang hồ sơ là cố ý; khối "Thông tin báo tử" thu gọn như mọi lần mở hồ sơ, không phải biện pháp riêng; ghi rủi ro còn lại (trang web khác chỉ làm hiện tên và ngày giỗ trên chính màn hình người dùng, phải biết UUID). Dòng `set_v_notice_open(false)` trong `app.rs` vẫn còn (vô hại, xem P57-03) |

Không thuộc vòng này, theo mô tả PR: DV25-02 (góp ý — R14 chưa bắt buộc chính bản phát hành chuyển đổi) để sau; DV25-07
không còn áp dụng vì #25 đã gộp. Chấp nhận cả hai.

Tóm lại: **5/5 mục được giao ĐÃ SỬA**; không mục nào CHƯA SỬA.

## 2. Đúng và đã kiểm, không có lỗi

- `deliver` dùng chung cho mọi lời gọi `bg` (sao lưu, khôi phục, giải mã ảnh) — đổi hành vi bằng không so với mã cũ (cùng
  hai điều kiện `upgrade()` và `PS_TLS`), chỉ thêm giá trị trả về.
- Đưa nội dung vào `FocusScope` không đổi thứ tự trợ năng hay bố cục (ảnh trùng byte); `TouchArea { }` chặn bấm xuyên
  hộp vẫn đứng trước `FocusScope`.
- Tài liệu và chú thích WiX không còn hứa điều bộ cài không làm.
- Diff không chạm mã nào ngoài phạm vi các mục trên.

## 3. Phát hiện mới

Không có mục **chặn**. Không có mục **nên sửa**.

### P57-01 — góp ý — Ca "Esc trong ô mật khẩu" không chứng minh tiêu điểm đã ở ô nhập

- Vị trí: `ui_tests_people.rs` `theo_ky_ics_sao_luu_the_nhac` (`click(... "Mật khẩu (ít nhất 8 ký tự)")` rồi `key(Escape)`).
- Vấn đề: nếu cú bấm giả lập không chuyển tiêu điểm vào `LineEdit` (ví dụ sau này ô nằm dưới một lớp phủ), Esc đi thẳng
  tới `FocusScope` và ca vẫn đạt — tức là ca không phân biệt được bố cục mới với bố cục cũ của DV25-05.
- Cách sửa: trước Esc, gửi một ký tự (ví dụ `"a"`) và kiểm `pw1 == "a"` (chứng minh ô đang có tiêu điểm), rồi mới Esc.

### P57-02 — góp ý — Đường `thread::spawn` → `invoke_from_event_loop` vẫn ngoài kiểm thử

- Vị trí: `people.rs` `bg` (nhánh `cfg(not(test))`).
- Vấn đề: `deliver` đã được kiểm, nhưng việc bản phát hành thật sự gọi tới `deliver` (và lời gọi `invoke_from_event_loop`
  không lỗi) chỉ được bảo đảm bằng việc đọc mã. Rủi ro thấp: ba dòng, không nhánh.
- Cách sửa (khi tiện): một kiểm thử chạy vòng sự kiện của backend kiểm thử Slint, hoặc chấp nhận và ghi vào biên bản
  rằng đây là phần còn lại có chủ ý.

### P57-03 — góp ý — Chú thích trong `app.rs` còn gọi việc thu gọn là biện pháp BM2-15

- Vị trí: `app.rs` (`// BM2-15: mở từ URI … → khối thông tin báo tử luôn thu gọn.` + `set_v_notice_open(false)`).
- Vấn đề: 08 §14 #12 nay nói rõ đây không phải biện pháp riêng (DV25-06); chú thích trong mã nói ngược lại.
- Cách sửa: sửa chú thích thành "đã thu gọn trong `open_person`; dòng này chỉ để rõ ý" hoặc bỏ dòng.

### Còn mở từ vòng trước (không đổi)

- DV25-02 (góp ý): R14 chưa bắt buộc "chính `LichViet.exe` đã chuyển đổi / đặt lại thông báo".
- Việc phải thử trên máy thật: R5 Windows (thông báo khi ứng dụng đóng, sau khởi động lại), giao diện bộ cài khi cài và
  khi gỡ.

## 4. Kết luận

| Mức | Số | Mã |
|---|---|---|
| Chặn | **0** | — |
| Nên sửa | 0 | — |
| Góp ý | 3 | P57-01, P57-02, P57-03 |

**GỘP ĐƯỢC.** DV25-01, DV25-03, DV25-04, DV25-05, DV25-06 đều ĐÃ SỬA, có bằng chứng (mã, kiểm thử, CI Windows run 341,
ảnh trùng byte). Không có lỗi chặn mới. Nên gộp #57 trước #62 (PR #62 dựng trên nhánh này).
