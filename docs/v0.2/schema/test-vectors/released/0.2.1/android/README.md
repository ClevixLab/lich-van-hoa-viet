# Bộ mẫu đóng băng: Android 0.2.1

Kho dữ liệu của mô-đun Người thân trên Android 0.2.1 (bản Android đầu tiên có mô-đun này) và tệp sao lưu của nó.
Tạo ngày 2026-11-07 từ commit e3a0276 (VERSION 0.2.1). Không bao giờ sửa, đổi tên hay xóa tệp nào ở đây
(released/README.md); `manifest.json` ghi sha256 từng tệp.

## Cách tạo — đọc kỹ trước khi dùng

Kho **không** được tạo bằng cách bấm giao diện ứng dụng trên điện thoại. Nó được ghi bởi **lõi 0.2.1** (Rust, `lvhv-ffi`)
trên máy Linux, qua đúng các hàm `LvhvStore` mà ứng dụng Android gọi qua UniFFI (`add_photo`, `add_attachment`,
`save_person_json`, `save_event_type_json`, `save_event_json`, `put_settings_json`, `set_lock_screen_names`,
`export_backup`). Mã tạo: `make_android_fixture.rs.txt` — chạy như ví dụ của `lvhv-ffi` trên commit trên
(`ffi/examples/make_android_fixture.rs`, `cargo run --release -p lvhv-ffi --example make_android_fixture -- <ảnh> <ra>`),
không giữ trong cây mã. Ảnh JPEG vẽ bằng Pillow, không có EXIF: `make_images.py.txt`.

Hệ quả cần biết:

- Định dạng kho (SQLite `lvhv.sqlite` chế độ WAL, `photos/<id>.img`, `attachments/<id>.bin`) là của lõi, giống hệt trên
  máy thật — ứng dụng Android không tự ghi tệp nào vào kho.
- Phần ứng dụng Android tự làm trước khi gọi lõi (thu nhỏ và mã hóa lại ảnh, bỏ EXIF) không được chạy: ảnh đầu vào đã là
  JPEG sạch có kích thước trong giới hạn. Lõi vẫn kiểm (`stripped: true`).
- `shared_prefs/*.xml` viết tay theo đúng tên tệp, khóa và kiểu mà bản 0.2.1 đọc (`MainActivity.Prefs`, `Updates`,
  `Reminders`): cỡ chữ A++ (`size` 3), giao diện tối (`theme` 2), "Để sau" tới 2099-12-31, đã hỏi quyền thông báo.
- Tệp sao lưu `backup.lvhv` do `export_backup(…, "0.2.1", "android")` tạo — cùng hàm nút "Tạo tệp sao lưu" gọi.
  Mật khẩu: `mật khẩu thử nghiệm`.

## Nội dung

- A — người đã mất: ngày mất âm lịch 15/6 **nhuận** năm 2025, ảnh chân dung, ảnh mâm lễ (`tray_photo_id`), thông tin báo
  tử có ảnh giấy chứng tử (JPEG) + tin báo tử (PDF), giới tính, ghi chú nhiều dòng, nhắc giỗ 0/3/10 ngày có tiên thường.
- B — người đã mất: chỉ biết ngày/tháng âm lịch (không năm), Công giáo, không thắp hương.
- C — người đang sống: sinh 29/02/1992 (mừng 01/03), sinh nhật âm lịch, nhắc sinh nhật dương và âm.
- Loại sự kiện tự tạo "Lên lão" (lặp lại âm lịch, gắn C); sự kiện một lần "Tốt nghiệp" gắn C và A.
- Cài đặt: nhắc sao lưu 60 ngày, giờ nhắc mặc định 06:30; hiện tên trên màn hình khóa: bật.

## Kỳ vọng và kiểm thử

Kỳ vọng (ngoài bộ mẫu): `released/expected/0.2.1-android.json` — trạng thái kho (`state`, `state_sha256`), sha256 từng ảnh
và tệp đính kèm, tùy chọn trên máy.

- Trình giả lập (CI `android.yml`, job `upgrade`, `apps/android/ci/upgrade-test.sh`): chép `store/` và `shared_prefs/` vào
  thư mục riêng của bản đang kiểm rồi `SelfCheckActivity --es op verify-released` — trạng thái `ok` (không chuyển đổi),
  dữ liệu đúng từng trường, từng ảnh/tệp đúng sha256, tùy chọn còn nguyên; tệp sao lưu của bộ mẫu và tệp máy vừa xuất đều
  nhập được vào kho trống cho đúng trạng thái kỳ vọng.
- JVM (Robolectric): `ReleasedFixtureTest` — sha256 theo `manifest.json`, mở bản sao kho bằng lõi qua FFI.
- Rust: `store/tests/released.rs` — `released_0_2_1_android_is_frozen`, `released_0_2_1_android_opens_unchanged`.
