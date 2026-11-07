# Duyệt PR #62 (Windows YC-2026-10, ảnh mâm lễ) — mã và thiết kế

Trạng thái: **XONG** — 2026-11-07. Kết luận ở §5.

Bên duyệt: một bên không tham gia viết mã của PR #62; không sửa nhánh đang duyệt.

| Mục | Giá trị |
|---|---|
| PR | #62 `v0.2-yc10-windows` → `main` (dựng trên `main` + #57) |
| Đầu mã được duyệt | `b9321fb` |
| Đầu nhánh | `6d58ae4` — chỉ thêm ảnh chụp của CI (run 343, `[skip ci]`) |
| So với | `git diff origin/v0.2-w3-windows-dv25 b9321fb` (tách riêng phần YC-2026-10, bỏ phần của #57): `windows.yml`, `Cargo.toml`, `build.rs`, `src/app.rs`, `src/trays.rs` (mới), `src/people.rs`, `src/shots.rs`, `src/ui_tests_people.rs`, `ui/people.slint`, 08 §14 #20. Ngoài `apps/windows/` và `docs/` chỉ có `windows.yml`; ảnh Apple đẩy nhầm đã được bỏ (`3e7308c`) |
| CI | `windows` #343 (run 42379250597) trên `b9321fb`: mọi bước `success` — bước mới "Kiểm gói ảnh mâm lễ trước khi nhúng" (`ĐẠT: 0 ảnh đã duyệt — gói rỗng, không hiện mâm (R15 (a))`), `cargo test` **31/31** (gồm `ui_tests_people::anh_mam_le`, `nguoi_da_mat_tuong_niem_sua`), chụp ≥ 220 ảnh, `.msi`, thông báo, R14, kiểm bản phát hành không có chế độ kiểm thử. `ci` (run 42379250550): mọi việc `success`, gồm "Gói ảnh mâm lễ (04 B8 / R15 (b))" |
| Ảnh | `apps/windows/screenshots/` ở `6d58ae4` (220 PNG): đã xem `nt-21-tuong-niem` (không mâm), `nt-22b-tuong-niem-cong-giao`, `nt-25-tuong-niem-anh-mam`, `nt-25b-sua-anh-mam` (sáng, 1×, 1180 × 860) |

Căn cứ: `docs/v0.2/08` §16 (16.1–16.4) và §14; `04` R15; `01` §3.7 và "Bổ sung 01-YC10-a"; `07` §10.4;
`docs/reviews/DUYET-MA-YC10-CORE.md` (MA10-11); để so nền tảng: `DUYET-PR46-ANH-MAM-LE-ANDROID.md`,
`DUYET-PR50-ANH-MAM-LE-PWA.md` (G-03, G-04).

Giới hạn: không chạy được Windows; không dựng Rust cục bộ. Kết luận về hành vi trên Windows lấy từ mã, nhật ký CI run 343
và ảnh do CI chụp.

Mức: **chặn** / **nên sửa** / **góp ý**. Mã phát hiện: `P62-<số>`.

## 1. Đối chiếu đặc tả

| Yêu cầu | Kết quả | Bằng chứng |
|---|---|---|
| §16.1 bỏ mâm vẽ và lọ hoa: nến · bát hương ba nén (khói) · nến | **ĐÚNG** | `people.slint` `MemorialPage`: xóa khối "Mâm ngũ quả"; hai nến x = 120/284, bát hương x = 170 (giữa mặt bàn 60…360, tâm 210 — đối xứng), khói dời theo. Kiểm thử ca 1: `Mâm ngũ quả` = 0, `Hoa` = 0, `Nến` = 2, `Bát hương với ba nén hương` = 1. Ảnh `nt-21` |
| §16.1 Công giáo: nến · hoa · nến, không hương | **ĐÚNG** | `m-catholic` (đặt từ `tray_style == "cong_giao"`), bó hoa trắng có nhãn "Hoa"; ca 4: `Hoa` = 1, `Nến` = 2, không bát hương, không nút khói. Ảnh `nt-22b` |
| §16.1 thứ tự: ảnh gia đình → ảnh Clevix cùng `tray_style` → không có mâm | **ĐÚNG** | `open_memorial`: `tray_photo_id` → `photo_image`; không có thì `trays::pick(style)`; không có nữa → `m-tray-kind = 0`. Ca 5: ảnh gia đình thắng; bỏ ảnh gia đình → ảnh Clevix + ghi công; khác kiểu mâm hay gói rỗng → không có mâm |
| §16.1 ảnh dưới bàn thờ, 480 × 360 (4:3 cắt giữa), bo 12 px, viền vàng đồng, nhãn trợ năng | **ĐÚNG** | `Rectangle { width: 480px; height: 360px; border-radius: 12px; clip: true; border-color: #b8893a }` + `image-fit: cover`; nhãn "Ảnh mâm lễ của gia đình" / "Ảnh mâm lễ". Ảnh `nt-25`: ảnh nằm dưới mặt bàn thờ, trên lời tưởng niệm |
| §16.1 dòng ghi công | ĐÚNG theo câu chữ 08 (xem P62-04 về khác biệt với PWA) | `build.rs`: `ghi_cong` khác null → "Ảnh: <tác_giả> · <giấy_phép>"; hiện dưới ảnh, cỡ 13 × scale |
| §16.1 nút chữ "Thêm ảnh mâm lễ của gia đình" dưới lời tưởng niệm; có ảnh gia đình thì không có | **ĐÚNG** | Điều kiện `m-tray-kind != 1 && !read-only`; nhãn trợ năng + hành động mặc định. Ca 5: có ảnh gia đình → 0 nút; bấm nút → trang `form`, nhóm Tưởng niệm mở. Ảnh `nt-22b` (có nút), `nt-25` (không) |
| §16.2 mục "Ảnh mâm lễ ngày giỗ" ngay sau "Lời tưởng niệm", đúng câu chữ | **ĐÚNG** | So từng chuỗi: "Ảnh mâm lễ ngày giỗ", "Chưa có ảnh. Ngày giỗ chỉ hiện bát hương và nến.", "Chọn ảnh mâm lễ…", "Đang xử lý ảnh…", "Đổi ảnh", "Bỏ ảnh", "Không đọc được ảnh này. Hãy chọn một ảnh khác.", "Ảnh quá lớn (tối đa 25 MB).", dòng gợi ý "Ảnh chỉ lưu trên máy này và trong tệp sao lưu mã hóa. Ứng dụng tự bỏ thông tin vị trí và máy chụp trong ảnh." — khớp 08 §16.2. Thu nhỏ 120 × 90. Ảnh `nt-25b` |
| §16.2 đang xử lý → khóa nút Lưu | **ĐÚNG** (chưa có kiểm thử, P62-07) | `FormButtons`: `enabled: People.f-tray-state != 1`; `save_form` chỉ có một lối gọi (`on_save_form`); nút chọn/đổi/bỏ ẩn khi đang xử lý nên không chọn chồng |
| §16.2 / 01 §3.8: giải mã, ≤ 1600 px, ≤ 1 MB, không phóng to, bỏ siêu dữ liệu; giới hạn đầu vào | **ĐÚNG** | `load_tray`: `metadata().len() > 25 MB` → từ chối **trước** khi đọc; `photo::reencode_with(…, 1600, 1 MB)`: kiểm lại 25 MB, chỉ nhận JPEG/PNG/GIF/WebP/BMP, đọc kích thước từ phần đầu tệp rồi từ chối > 12 000 px hay > 50 Mpx **trước khi giải mã**, giải mã với `Limits { max_image_width/height = 12 000, max_alloc = 256 MB }`, xoay theo EXIF (JPEG) rồi mã hóa lại JPEG từ điểm ảnh (không mang đoạn APP1/APP13 nào). Chạy ở luồng phụ (`bg`). Ca 2: JPEG 2400 × 1800 có EXIF hướng 6 + APP13 → ảnh lưu không có APP1/APP13, không còn chuỗi GPS giả, 1200 × 1600, ≤ 1 MB. Lõi còn kiểm cạnh ≤ 1600 khi lưu (MA10-02) |
| MA10-11: ảnh chỉ ghi vào kho khi bấm Lưu | **ĐÚNG** | `PState.tray: PhotoSel` giữ ảnh trong bộ nhớ; `save_form` mới `add_photo`; ca 2 kiểm thư mục `photos\` rỗng trước Lưu. Hủy biểu mẫu → không có gì để xóa. Lưu lỗi → `delete_photo(new_tray)` |
| `tray_photo_id` giữ trên mọi đường lưu | **ĐÚNG** | Chỉ một đường lưu hồ sơ trong ứng dụng (`save_form` → `save_person_json`; `mark_deceased_json` chỉ áp cho người đang sống, chưa có ảnh). `build_person_json` sao chép `memorial` của bản gốc (luôn là object với người đã mất), rồi `Keep(Some)` gán lại id, `Keep(None)` bỏ khóa, `New` ghi ảnh mới; ảnh cũ do lõi dọn (`gc_photos`). Ca 3: bỏ ảnh → không còn khóa, `photo_bytes` lỗi, không còn tệp `<id>.img` |
| §16.2 dòng hồ sơ "Ảnh mâm lễ: Đã có / Chưa có" | **ĐÚNG** | `render_person`; ca 2 và ca 3 kiểm cả hai giá trị |
| §16.2 "Ảnh mâm lễ cũng sẽ bị xóa" khi đổi đã mất → đang sống | Không áp dụng | Windows không có thao tác này (biểu mẫu cố định `dead` khi mở); ghi ở 08 §14 #20 |
| §16.3 / R15 (b): chỉ nhúng mục `duyet`, có tệp, đúng `sha256`; không tải qua mạng | **ĐÚNG** | `build.rs embed_tray_pack`: bỏ qua mục khác `duyet`; mục đã duyệt thiếu tệp / sai `sha256` / sai chữ ký JPEG-WebP → `panic!` (dựng thất bại); `include_bytes!` vào `OUT_DIR/trays.rs`. CI chạy `scripts/check-tray-pack.py` (cấu trúc, `duyet_boi`, B6, ≤ 400 KB/ảnh, ≤ 2 MB cả gói, kích thước) trước `cargo`. Gói hiện rỗng; `pack_len() == 0` được kiểm |
| 07 §10.4: không ảnh mâm nào vào bản phát hành khi chưa được chủ dự án duyệt | **ĐÚNG** | Gói rỗng → `PACK = &[]`. Tệp ảnh thử `content/memorial/trays/test/04-dat/a.jpg` chỉ đọc trong `#[cfg(test)]` qua `TEST_PACK`; `build.rs` từ chối mọi `file` có `/` nên không thể nhúng `test/…`. Ảnh mâm trong `nt-25`/`nt-25b` là hình tổng hợp vẽ bằng mã (`shots::demo_tray`), chỉ dùng ở chế độ chụp màn hình (xem P62-10) |
| R15 (f) / R-PRIV: ảnh không lộ vào thông báo, .ics, URI `lichviet:`, tên tệp | **ĐÚNG** | Không chạm `toasts.rs`, `.ics` (lõi đã kiểm, `DUYET-MA-YC10-CORE`); URI vẫn chỉ có UUID + id lời nhắc + băm. Đường dẫn tệp gốc chỉ dùng để đọc, không lưu; câu lỗi là chuỗi cố định, không chứa tên tệp; ảnh lưu theo id ngẫu nhiên |
| §16.4 ảnh chụp (1), (2), (4) ở 360 px và 1280 px | Một phần, đã giải thích | `nt-21`, `nt-25`, `nt-22b` sáng/tối, 1×/2× ở cửa sổ 1180 × 860; không có 360 px vì Windows có bề rộng tối thiểu 560 px (08 §14 #20). Xem P62-11 |
| `windows.yml`: không commit ảnh khi run bị hủy | ĐÚNG cho ca hủy; chưa đủ cho ca lỗi (P62-01) | `if: ${{ !cancelled() && … }}` |

## 2. Đúng và đã kiểm, không có lỗi

- `trays::pick` giải mã ảnh gói qua `photo::to_slint` (có trần 4096 px / 80 MB) — gói lỗi không làm treo màn hình.
- `TEST_PACK` là `thread_local` chỉ có trong `cfg(test)`; bản phát hành không có đường thay gói lúc chạy.
- `build.rs` dùng `serde_json` và `sha2 =0.11.1` chỉ ở `[build-dependencies]`; `cargo-deny` đạt.
- Ảnh `nt-25b`: mục ảnh nằm đúng sau "Lời tưởng niệm", trước ô "hưởng dương"; nút Hủy/Lưu thấy được sau khi cuộn tới cuối
  (nhóm Tưởng niệm là nhóm cuối). Ảnh `nt-25`: khung 4:3 bo góc, viền mảnh, lời tưởng niệm ở dưới; không có nút thêm ảnh.
  Ảnh `nt-22b`: nến · hoa · nến cân đối, nút chữ "Thêm ảnh mâm lễ của gia đình" gạch chân, cùng màu với "Dừng khói".
- Ảnh hỏng → câu lỗi đúng, trạng thái trở về trước (`before`), không đổi `ps.tray`.

## 3. Phát hiện

Không có mục **chặn**.

### P62-01 — nên sửa — `!cancelled()` vẫn commit bộ ảnh thiếu khi bước chụp màn hình **lỗi**

- Vị trí: `.github/workflows/windows.yml`, bước "Commit ảnh chụp vào nhánh của PR".
- Vấn đề: bước này xóa hẳn `apps\windows\screenshots` rồi chép lại từ `$RUNNER_TEMP\screens`. `!cancelled()` chặn ca run bị
  hủy (run 339), nhưng vẫn chạy khi run **thất bại**. Nếu bước chụp màn hình dừng giữa chừng (quá 240 giây ở lượt 2×, thoát
  mã khác 0, hay < 220 ảnh) thì thư mục tạm có bộ ảnh thiếu và bước commit vẫn xóa các ảnh còn lại trên nhánh — đúng lỗi
  mà thay đổi này muốn tránh. (Khi lỗi ở bước trước khi chụp thì thư mục tạm không có, `Copy-Item` lỗi và
  `continue-on-error` nuốt lỗi — không hại.)
- Cách sửa: đặt `id: shots` cho bước chụp màn hình và đổi điều kiện thành
  `!cancelled() && steps.shots.outcome == 'success' && …` (vẫn commit được khi các bước sau — `.msi`, R14 — lỗi, là lúc
  ảnh có ích nhất).

### P62-02 — góp ý — Kết quả xử lý ảnh ở luồng phụ có thể rơi vào biểu mẫu khác

- Vị trí: `people.rs` `on_pick_tray` (và `on_pick_photo` có sẵn — cùng mẫu).
- Vấn đề: `done` gán `ps.tray = New(e)` và `f-tray-state = 2` vào **biểu mẫu đang mở lúc xong việc**. Chọn một ảnh lớn
  (vài giây giải mã), bấm Hủy (nút Hủy không bị khóa), mở biểu mẫu sửa của một người đã mất khác trước khi xong → ảnh của
  người trước hiện ở biểu mẫu của người sau và sẽ được lưu cho người đó nếu bấm Lưu. Người dùng vẫn thấy ảnh thu nhỏ nên
  khó lưu nhầm mà không biết; xác suất thấp. Ảnh chân dung có cùng mẫu từ trước PR này.
- Cách sửa: thêm một số đếm thế hệ biểu mẫu trong `PState` (tăng ở `open_person_form` / `cancel_form`), chụp lại khi gọi
  `bg`, và bỏ kết quả nếu không khớp. Áp cho cả ảnh chân dung và tệp đính kèm.

### P62-03 — góp ý — `build.rs` chặt hơn lược đồ: `file` có thư mục con làm dừng dựng

- Vị trí: `build.rs` `assert!(!s("file").contains('/') …)`; `manifest.schema.json` (`file` cho phép `/`); `check-tray-pack.py`
  (cho phép thư mục con, chỉ đòi nằm trong thư mục gói).
- Vấn đề: một mục đã duyệt hợp lệ theo lược đồ và đạt kiểm của CI (ví dụ `file: "2026/mam-01.jpg"`) sẽ làm bản Windows dựng
  thất bại, trong khi các nền tảng khác nhận. Hướng lỗi an toàn (không lộ ảnh `test/`), nhưng là khác biệt giữa nền tảng.
- Cách sửa: cho phép thư mục con, chỉ từ chối `..`, đường tuyệt đối và tiền tố `test/` (hoặc kiểm `canonicalize` nằm trong
  thư mục gói và không trong `test/`); hoặc thu hẹp lược đồ + README về "tệp nằm ngay trong thư mục gói" cho mọi nền tảng.

### P62-04 — góp ý — Dòng ghi công khác bản web

- Vị trí: `build.rs` (`credit = "Ảnh: {tac_gia} · {giay_phep}"` khi `ghi_cong` khác null).
- Vấn đề: sau G-03 của `DUYET-PR50`, PWA hiện **nguyên văn** `ghi_cong` (`pwa/build.mjs`); Android và Windows dựng chuỗi từ
  `tac_gia` + `giay_phep`. Cùng một mục, bốn nền tảng có thể hiện hai câu khác nhau; lược đồ mô tả `ghi_cong` là "dòng ghi
  công hiện trong ứng dụng". Không ảnh hưởng hiện nay (gói rỗng).
- Cách sửa: chủ dự án / vai nội dung chọn một quy ước, ghi vào 08 §16.1 và `content/memorial/trays/README.md`, rồi các nền
  tảng theo. Kiểm thử ca 5 dùng chuỗi ghi công viết sẵn trong `TEST_PACK`, nên quy tắc dựng chuỗi của `build.rs` hiện không
  có kiểm thử nào.

### P62-05 — góp ý — Đường mã nhúng gói **có ảnh** chưa từng được dựng trên Windows

- Vị trí: `build.rs` (sinh `include_bytes!` với đường dẫn từ `fs::canonicalize`, trên Windows có dạng `\\?\C:\…`).
- Vấn đề: gói rỗng và kiểm thử ca 5 đi qua `TEST_PACK`, nên mã sinh ra cho một mục đã duyệt (đường dẫn, thoát ký tự,
  `credit`, `rerun-if-changed`) chưa được biên dịch lần nào. Lần đầu chủ dự án duyệt một ảnh, bản dựng có thể hỏng đúng lúc
  phát hành.
- Cách sửa: cho `build.rs` đọc đường manifest từ biến môi trường (như `LVHV_TRAY_MANIFEST` của PWA) và thêm một bước CI
  `cargo test` với `content/memorial/trays/test/04-dat/manifest.json` để kiểm `pack_len() == 1` và chuỗi ghi công. Khi có
  biến môi trường mà tệp không tồn tại thì dừng dựng (bài học G-04): hiện `if let Ok(text) = fs::read_to_string(&man)` lặng lẽ
  coi thiếu manifest là gói rỗng.

### P62-06 — góp ý — `build.rs` không kiểm `duyet_boi` và các giới hạn R15 (b)

- Vị trí: `build.rs`.
- Vấn đề: `duyet_boi` (07 §10.4), giới hạn 400 KB / 2 MB và B6 chỉ được kiểm bởi `check-tray-pack.py` trong CI. Bản phát hành
  chỉ dựng trong CI nên hiện đủ; một bản dựng cục bộ thì không.
- Cách sửa: ít nhất kiểm `duyet_boi` khác rỗng và cỡ tệp ≤ 400 KB trong `build.rs` (hai dòng), hoặc ghi rõ trong
  `docs/04-BUILD-NATIVE.md` rằng bản dựng ngoài CI phải chạy `check-tray-pack.py` trước.

### P62-07 — góp ý — Chưa có kiểm thử cho "khóa Lưu khi đang xử lý"

- Vị trí: `ui_tests_people.rs` `anh_mam_le`.
- Vấn đề: trong kiểm thử `bg` chạy tuần tự nên trạng thái 1 không bao giờ thấy được; thuộc tính `enabled` của nút Lưu
  không được kiểm.
- Cách sửa: đặt `f_tray_state = 1`, bấm "Lưu", kiểm trang vẫn là `form` và kho không đổi; kiểm câu "Đang xử lý ảnh…" có
  mặt.

### P62-08 — góp ý — Ảnh quá nhiều điểm ảnh bị báo "tối đa 25 MB"

- Vị trí: `people.rs` `load_tray` (ánh xạ mọi lỗi có "quá lớn" sang "Ảnh quá lớn (tối đa 25 MB).").
- Vấn đề: một PNG 3 MB kích thước 14 000 × 9 000 bị từ chối vì số điểm ảnh, nhưng câu báo nói về 25 MB — người dùng không
  biết sửa gì. "Ảnh quá chi tiết, chưa nén được dưới 1 MB" thì thành "Không đọc được ảnh này".
- Cách sửa: đặc tả chỉ có hai câu; hoặc thêm câu thứ ba vào 08 §16.2 (và ảnh chân dung) cho ca kích thước điểm ảnh, hoặc
  chấp nhận và ghi trong 08 §14 #20.

### P62-09 — góp ý — Ảnh mâm lễ có id nhưng không giải mã được: biểu mẫu nói "Chưa có ảnh"

- Vị trí: `people.rs` `open_person_form` (`f_tray_state = 0` khi `photo_image` trả `None`, nhưng `ps.tray = Keep(Some(id))`).
- Vấn đề: trang hồ sơ ghi "Ảnh mâm lễ: Đã có", biểu mẫu ghi "Chưa có ảnh…" và không có nút "Bỏ ảnh"; lưu thì id vẫn được
  giữ. Không mất dữ liệu, nhưng người dùng không bỏ được ảnh hỏng ngoài cách chọn ảnh khác.
- Cách sửa: khi có id mà không giải mã được, vẫn hiện trạng thái 2 với khung trống (hoặc câu "Không hiển thị được ảnh đã
  lưu") để có "Đổi ảnh" / "Bỏ ảnh".

### P62-10 — góp ý — Chưa có bằng chứng mã chụp màn hình (có hình mâm tổng hợp) bị loại khỏi `LichViet.exe`

- Vị trí: `app.rs` `run_ui` (`UiMode::Shots` → `shots::start`), `shots::demo_tray`; bước CI "LichViet.exe phát hành không có
  chế độ kiểm thử".
- Vấn đề: `main.rs` chỉ tạo `UiMode::Normal`, nên với `lto = true` nhánh chụp màn hình gần như chắc bị loại, nhưng bước CI
  chỉ tìm 7 chuỗi cờ. 07 §10.4 yêu cầu ảnh tổng hợp không bao giờ vào bản phát hành; hình ở đây là mã vẽ, không thể hiện ra
  ở bản phát hành, nên chỉ là việc thêm bằng chứng.
- Cách sửa: thêm vào bước đó một chuỗi chỉ có trong `shots.rs` (ví dụ `nt-25-tuong-niem-anh-mam`) vào danh sách "không được
  có trong `LichViet.exe`, phải có trong `lvhv-ci-check.exe`".

### P62-11 — góp ý — Ghi chú bề rộng ảnh chụp trong 08 §14 #20

- 08 §16.4 đòi ảnh 360 px **và 1280 px**; #20 giải thích không có 360 px nhưng không nói ảnh máy tính là 1180 px (cửa sổ
  CI). Thêm nửa câu để bên kiểm định R15 khỏi hỏi lại. Cũng nên nói ca "Đổi ảnh" (thay ảnh cũ → lõi dọn ảnh cũ) chỉ được
  kiểm ở lõi, chưa có ca giao diện Windows.

## 4. R15 trên Windows (theo bằng chứng của PR)

| Mệnh đề | Kết quả | Ghi chú |
|---|---|---|
| (a) | ĐẠT | Ca 1, ca 5 (gói rỗng), ảnh `nt-21`, `nt-22b`; CI: gói thật 0 ảnh |
| (b) | ĐẠT | `build.rs` + `check-tray-pack.py` (CI); phần "gói có ảnh" chưa được dựng trên Windows (P62-05) |
| (c) | ĐẠT | Ca 2 (không APP1/APP13, ≤ 1600, ≤ 1 MB, chỉ đúng người, thắng ảnh Clevix) |
| (d)(e)(g) | Thuộc lõi | Không đổi trong PR này |
| (f) | ĐẠT phần giao diện | Bỏ ảnh → không còn bản ghi và tệp (ca 3); không lộ vào thông báo / .ics / URI / tên tệp |

## 5. Kết luận

| Mức | Số | Mã |
|---|---|---|
| Chặn | **0** | — |
| Nên sửa | 1 | P62-01 |
| Góp ý | 10 | P62-02 … P62-11 |

**GỘP ĐƯỢC sau khi gộp #57.** Bản Windows đúng 08 §16.1–16.3: không còn mâm vẽ, đúng thứ tự ảnh, đúng câu chữ, ảnh mã hóa
lại có trần giải mã và không còn siêu dữ liệu, ảnh chỉ ghi khi Lưu (MA10-11), `tray_photo_id` được giữ, gói chỉ nhúng mục
đã duyệt và hiện rỗng (07 §10.4). Đề nghị sửa P62-01 (một dòng `id` + điều kiện) trong PR này hoặc ngay sau; P62-04 cần chủ
dự án chọn quy ước ghi công chung cho bốn nền tảng trước khi duyệt ảnh đầu tiên; P62-05 nên làm trước lần duyệt ảnh đó.
