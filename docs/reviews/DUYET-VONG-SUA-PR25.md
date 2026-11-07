# Duyệt vòng sửa PR #25 (Windows, mô-đun Người thân) — mã và thiết kế

Trạng thái: **XONG** — 2026-11-07. Kết luận ở §4.

Bên duyệt: một bên không tham gia viết mã của PR #25; không sửa nhánh đang duyệt.

| Mục | Giá trị |
|---|---|
| PR | #25 `v0.2-w3-windows` |
| Đầu mã được duyệt | `0652321` (gộp `main` vào nhánh); vòng sửa nằm ở commit `5517627` |
| Đầu nhánh | `8a172cc` — chỉ thêm ảnh chụp của CI (run 296, `[skip ci]`) |
| So với | `git diff 5f4dfcc 0652321` (5f4dfcc = đầu mã của vòng duyệt cuối, `DUYET-MA-DOT3-VONG-CUOI.md`) và `git diff origin/main...0652321` |
| CI | run `windows` #296 (id 42365739507) trên `0652321`: mọi bước `success`, gồm `cargo test` (30 kiểm thử, gồm `ui_tests`, `ui_tests_people`), chụp màn hình (≥ 208 ảnh), thông báo đặt lịch sau khi cài `.msi`, R14, và bước mới "LichViet.exe phát hành không có chế độ kiểm thử". `ci`, `android`, `apple` trên cùng commit: `success` |
| Ảnh | `apps/windows/screenshots/` ở `8a172cc`; đã xem các ảnh đổi so với `5f4dfcc`: `nt-03`, `nt-09b`, `nt-09c` (mới), `nt-18`, `nt-21b`, `nt-23`, `nt-24` (sáng/tối, 1×) |

Căn cứ: `docs/v0.2/08` §3 (§3.6, §3.8, §3.9), §12j–§12l, §14, §16; `docs/04-BUILD-NATIVE.md`; các biên bản
`DUYET-MA-DOT3-NATIVE.md` (PR #27), `DUYET-THIET-KE-DOT3-NATIVE.md` (PR #26), `DUYET-MA-DOT3-VONG-CUOI.md`,
`DUYET-THIET-KE-DOT3-VONG-CUOI.md`, `RA-SOAT-BAO-MAT-V02.md`, `RA-SOAT-BAO-MAT-V02-VONG2.md`.

Giới hạn: không chạy được Windows. Mọi kết luận về hành vi trên Windows lấy từ mã, nhật ký CI run 296 và ảnh do CI chụp.

Mức: **chặn** / **nên sửa** / **góp ý**. Mã phát hiện mới: `DV25-<số>`.

## 1. Các mục vòng trước

| Mã | Mức cũ | Kết quả | Bằng chứng |
|---|---|---|---|
| TKN3-W-01 | nên sửa | **ĐÃ SỬA** | `people.rs` `Banner` / `read_only_banner` / `store_open_banner`; `people.slint` `StoreBanner`: nền `gold-tint`, chữ `ink` cỡ 16, tiêu đề đậm, lời chính trùng câu chuẩn TKN3-X-03 (bản Windows thay "vào Sao lưu →" bằng nút ngay trong khung, "tải lại trang" bằng "mở lại ứng dụng"). Câu lỗi kỹ thuật tách khỏi lời chính, thu gọn dưới "Chi tiết kỹ thuật". Kiểm thử `chuyen_doi_loi_che_do_chi_doc`: lời chính không có "SQLite", chi tiết có, thu gọn sẵn, bấm mới mở. Ảnh `nt-24-…-sang` (chỉ đọc, khung đóng) và `nt-23-…-toi` (không mở được kho, chi tiết đang mở) đọc rõ. Tên loại tệp đính kèm cũng đổi theo TKN3-X-01 ("Tin báo tử / Giấy chứng tử / Tệp khác", ảnh `nt-09b`) |
| TKN-W-08 | nên sửa (vòng cuối: chưa đủ bằng chứng) | **ĐÃ SỬA** | Đo trên ảnh `nt-03-theo-ky-thang-sang-2x.png`: chữ "Đang xem kỳ có hôm nay" và "6 mục" là `#5e5150` trên nền trang `#e8e3e1` → **5,96:1**; ảnh tối `#cdbfbb` trên `#141011` → **10,58:1**. Mã: `people.slint` `PeriodPage` dùng `Theme.ink2` cho `pnote`, "Đang xem kỳ có hôm nay", `pcount`, `pempty`. (Phần mã đã có từ vòng 1, commit `1e3afab`; vòng này bổ sung bằng chứng đo được) |
| MA3C-25-01 | nên sửa | **ĐÃ SỬA** | `Cargo.toml`: thư viện `lvhv_windows` (`src/app.rs`) + hai tệp chạy. `src/main.rs` chỉ gọi `run_app()`, chỉ hiểu URI `lichviet:`, mọi tham số khác bỏ qua. Mọi cờ CI và việc thay thư mục dữ liệu nằm ở `src/ci.rs` / `lvhv-ci-check.exe`; `platform::data_dir` không còn đọc biến môi trường (`DATA_DIR_OVERRIDE` chỉ `ci.rs` đặt). CI run 296, bước 18: "Tệp trong .zip: LichViet.exe, OFL.txt"; "Đạt: không có chuỗi cờ CI (7 cờ), không tạo tệp khi được truyền cờ"; phép tìm chuỗi được đối chứng (các chuỗi phải có trong `lvhv-ci-check.exe`). `LichViet.exe` trong `.msi` trùng băm với bản trong `.zip` (`windows.yml:282`). `wix/main.wxs` chỉ đóng gói `LichViet.exe`. `set_lock_screen_names(true)` của dữ liệu mẫu nay chỉ còn trong công cụ CI |
| MA3C-25-02 | góp ý | **ĐÃ SỬA** (bằng tài liệu — cách sửa thứ hai được đề xuất) | `04-BUILD-NATIVE.md`: ghi rõ R14 (a) hiện là cùng số phiên bản 0.1.4 → 0.1.4, kết quả R14 (a) của bản 0.2.0 phải lấy từ lần chạy CI trên commit tăng `VERSION`, và từ 0.2.0 nên thêm ca cài từ MSI 0.2.x đã phát hành. Nhật ký run 296 vẫn ghi đúng "0.1.4 → 0.1.4" |
| BM2-15 / MA3C-25-03 | góp ý | **ĐÃ SỬA** | `toasts.rs` `launch_uri`: `c = content_hash(content)` (16 chữ hex đầu của SHA-256), không còn nội dung. Phần còn lại của URI: UUID hồ sơ và id lời nhắc của lõi (`lvhv:<rule_id>:<năm>:<k>`, 02 §7.4 — không có tên, không có ngày đầy đủ). `sync` gọi `reschedule_json` hai lượt: lượt 1 lấy `keep` (lõi chọn `keep` chỉ từ `desired`, không phụ thuộc danh sách chờ — `memorial/src/remind.rs` `select_keep`), đổi băm trùng thành nội dung của kế hoạch, lượt 2 ra kế hoạch thật. Thông báo cũ mang nội dung đầy đủ sẽ bị thay ở lần đồng bộ đầu. Kiểm thử `xml_roundtrip_with_and_without_body` (URI không chứa "Còn 3 ngày", "Thứ Ba"), `sync_twice_is_stable`. Windows thật (run 296, `thong-bao.txt`): đọc lại đúng băm, "Lần 2: added 0, removed 0". Phần "mở từ URI" xem DV25-06 |
| BM2-22 / MA3N-25-09 (phần còn lại) / MA3C-25-04 | góp ý | **ĐÃ SỬA** phần trong ứng dụng; phần bộ cài chỉ đúng một nửa (DV25-01) | Sao lưu có khung "Dữ liệu trên máy này" (đường dẫn, gỡ không xóa, cách xóa hẳn, nút "Mở thư mục dữ liệu") — ảnh `nt-18`; Giới thiệu ghi "Cài lại, nâng cấp hay gỡ ứng dụng đều không xóa sổ này". Thiếu `LOCALAPPDATA` → `data_dir()` trả đường rỗng, `ensure_store` hiện khung lỗi, không ghi vào `%TEMP%`; `attach::cleanup` bỏ qua đường rỗng. Không có nút "Xóa toàn bộ dữ liệu" (chọn hướng dẫn bằng chữ — chấp nhận ở mức góp ý). Gỡ cài đặt vẫn không gỡ thông báo đang chờ (góp ý cũ, giữ nguyên) |
| MA3C-25-05 | góp ý | **ĐÃ SỬA** phần khóa nút; **CHƯA SỬA** phần kiểm thử luồng phụ (DV25-03) | `open_backup_file`: `restore-busy` đặt trước `bg(...)`, xóa trong `done`; nút "Mở tệp" `enabled: !People.restore-busy` (TouchArea và hành động trợ năng đều kiểm `enabled`) |
| BM3-02 | nên sửa | **ĐÃ SỬA** | `ask_open_att` → hộp "Mở tệp bằng ứng dụng khác?" (ba dòng: ứng dụng ngoài, thông tin riêng tư có thể bị lưu/gửi đi, bản chép tạm tên chung bị xóa khi đóng); chỉ `DlgAction::OpenAtt` sau "Mở tệp" mới gọi `open_att`. Áp cho cả biểu mẫu (`on_open_attachment`) và trang hồ sơ (`on_open_v_attachment`). Nút mặc định là Hủy: tô kiểu `primary`, "Mở tệp" kiểu viền; `FocusScope` trong hộp: Esc đóng, Enter = Hủy khi `dlg-default-cancel`. Kiểm thử: hộp hiện, `dlg_default_cancel`, bấm Hủy → không mở và không có thư mục `tmp\`; bấm "Mở tệp" → `TepDinhKem.pdf` trong `tmp\`; trang hồ sơ cũng hỏi. Ảnh `nt-09c-hoi-mo-tep` sáng/tối |

Tóm lại: 8/8 mục được giao đã sửa phần chính; MA3C-25-05 còn phần kiểm thử (góp ý), BM2-22 còn một câu tài liệu sai
về bộ cài (DV25-01).

## 2. Đúng và đã kiểm, không có lỗi

- Tách tệp chạy không làm mất kiểm thử: `cargo test` chạy 30 kiểm thử trong thư viện (cả `ui_tests`, `ui_tests_people`,
  `toasts`, `upgrade`, `photo`, `attach`, `platform`), hai tệp chạy không có kiểm thử riêng. `src/app.rs` là bản dời
  nguyên văn của `main.rs` cũ, chỉ khác phần điểm vào (`run_app` / `run_ui(UiMode, args)`) và hai chỗ bỏ qua đường dữ
  liệu rỗng.
- R14 vẫn đạt trên Windows thật (run 296): (a) dữ liệu khớp từng trường, ảnh khớp SHA-256, "Để sau" giữ, gỡ cài đặt giữ
  4 tệp; (c) bộ mẫu đóng băng: "trạng thái khi kiểm: ok" (tức là chính `LichViet.exe` đã cài chuyển 1 → 2), có
  `lvhv.sqlite.pre-v2.bak`, gỡ giữ 6 tệp; ca lỗi chỉ đọc đạt.
- Hộp xác nhận mới dùng chung `open_dialog` (đặt lại `dlg-default-cancel = false` cho mọi hộp khác) — các hộp cũ không đổi
  nút mặc định.
- Ảnh `nt-23`/`nt-24` ở tối: khung vàng tối `#352a18`, chữ `ink`, chi tiết kỹ thuật `ink2` (7,9:1) — đủ tương phản.

## 3. Phát hiện mới

Không có mục **chặn**.

### DV25-01 — nên sửa — Trang kết thúc của bộ cài không hiện câu "dữ liệu được giữ" khi gỡ bằng `.msi`

- Vị trí: `docs/04-BUILD-NATIVE.md` (đoạn "Gỡ cài đặt (BM2-22, MA3N-25-09)"), chú thích trong `wix/main.wxs` (khối
  `<UI Id='LvhvUI'>`), mô tả commit `5517627`.
- Vấn đề: tài liệu ghi trang kết thúc hiện câu về dữ liệu "khi cài, và khi gỡ bằng cách chạy lại `.msi` → Remove". Trong
  WiX 3, `ExitDialog` chỉ hiện `WIXUI_EXITDIALOGOPTIONALTEXT` với điều kiện
  `WIXUI_EXITDIALOGOPTIONALTEXT AND NOT Installed` (`src/ext/UIExtension/wixlib/ExitDialog.wxs`, điều khiển
  `OptionalText`). Khi gỡ hay sửa chữa, `Installed` có giá trị → câu bị ẩn. Câu chỉ hiện khi cài mới hoặc nâng cấp lớn.
  Như vậy người gỡ ứng dụng không thấy câu này ở bất kỳ đường gỡ nào. (Suy từ mã nguồn WixUI; CI cài/gỡ ở chế độ im lặng
  nên không kiểm được.)
- Cách sửa (chọn một):
  1. Sửa tài liệu và chú thích cho đúng: câu chỉ hiện khi cài; người dùng biết dữ liệu được giữ qua Giới thiệu và
     Sao lưu › "Dữ liệu trên máy này".
  2. Hoặc tự thêm một điều khiển `Text` vào `ExitDialog` (hoặc vào `VerifyReadyDlg` khi gỡ) với điều kiện riêng có
     `REMOVE="ALL"`, rồi xem trên máy thật.

### DV25-02 — góp ý — R14 không bắt buộc "chính bản phát hành đã chuyển đổi và đặt lại thông báo"

- Vị trí: `upgrade.rs` `check_frozen` (chấp nhận cả `migrated` lẫn `ok`); `windows.yml` `Open-App`.
- Vấn đề: thiết kế mới để `LichViet.exe` đã cài mở kho trong 12 giây, rồi `lvhv-ci-check` kiểm. Nếu bản phát hành không mở
  được kho (thoát sớm thì bị bắt; nhưng treo hoặc chưa kịp mở trong 12 giây thì không), `check_frozen` tự mở kho, tự chuyển
  đổi và trả `migrated` — bước vẫn đạt, dù bản phát hành chưa chuyển đổi. Ca (a) cũng vậy: `check` tự đặt lại thông báo.
  Run 296 cho thấy lần này đúng là bản phát hành đã làm ("trạng thái khi kiểm: ok"; "added: 0"), nhưng phép kiểm không bắt
  buộc điều đó. Mã chuyển đổi là cùng thư viện, nên rủi ro thấp.
- Cách sửa: thêm tham số cho công cụ (ví dụ `--kiem-mau-dong-bang <exp> --sau-khi-mo`) để ca (c) đòi `state == "ok"` và có
  `.bak`, ca (a) đòi lần đặt lại đầu tiên của công cụ có `added == 0`.

### DV25-03 — góp ý — Luồng phụ vẫn không có kiểm thử; khóa nút "Mở tệp" không có kiểm thử

- Vị trí: `people.rs` `bg()` (dưới `cfg(test)` chạy tuần tự), `open_backup_file`.
- Vấn đề: phần đầu của MA3C-25-05 chưa làm: `invoke_from_event_loop`, `PS_TLS` và việc trả `restore-busy` về `false`
  không chạy trong kiểm thử nào. Nếu `done` không được gọi (cửa sổ đã đóng, `PS_TLS` rỗng), nút "Mở tệp" sẽ khóa tới
  lần mở lại ứng dụng — hiện không có đường nào như vậy trong mã, nhưng không có kiểm thử giữ điều đó.
- Cách sửa: như MA3C-25-05 (một hàm `bg_sync` tách riêng, hoặc kiểm thử dùng vòng sự kiện của backend kiểm thử Slint);
  ít nhất kiểm `restore-busy` về `false` sau cả nhánh lỗi và nhánh thành công.

### DV25-04 — góp ý — 08 §14 chưa ghi hai điểm mới của vòng này

- Vị trí: `docs/v0.2/08` §14, dòng #17 (tệp đính kèm) và #18.
- Vấn đề: dòng #17 vẫn tả "Mở bằng ứng dụng xem PDF" chép ra `tmp\…` mà không nhắc hộp xác nhận (BM3-02, Hủy mặc định,
  Enter/Esc = Hủy). Khung "Dữ liệu trên máy này" ở trang Sao lưu (BM2-22) là điểm riêng của Windows, chưa có dòng nào.
- Cách sửa: thêm vào #17 "… → hộp 'Mở tệp bằng ứng dụng khác?' (Hủy mặc định) → …"; thêm một dòng cho "Dữ liệu trên máy
  này".

### DV25-05 — góp ý — Bàn phím trong hộp thoại: Esc/Enter chỉ có tác dụng khi tiêu điểm còn ở `FocusScope`

- Vị trí: `people.slint` `Dialog`, `fs := FocusScope { init => { self.focus(); } … }`.
- Vấn đề: `FocusScope` không bao các ô nhập của hộp (nó là phần tử anh em với `VerticalLayout`). Ở hộp có ô mật khẩu (tạo
  sao lưu, khôi phục), sau khi bấm vào ô, Esc không còn đóng hộp. Với hộp BM3-02 (không có ô nhập) hành vi đúng như mô tả.
  Không có kiểm thử phím Enter/Esc.
- Cách sửa: đặt nội dung hộp bên trong `FocusScope` để phím nổi lên từ ô nhập; thêm ca gửi phím Enter vào hộp `mo-tep` và
  kiểm không mở tệp.

### DV25-06 — góp ý — "Mở từ URI: khối báo tử thu gọn" không thay đổi hành vi

- Vị trí: `app.rs` `run_ui` (`set_v_notice_open(false)` sau `invoke_open_person`).
- Vấn đề: `people::open_person` vốn đã đặt `v-notice-open = false` mỗi lần mở hồ sơ, nên dòng mới chỉ lặp lại mặc định.
  BM2-15 đề xuất thêm "mở từ URI chỉ mở danh sách"; PR chọn giữ việc mở thẳng trang hồ sơ (cần để bấm thông báo mở đúng
  người), khối báo tử thu gọn như mọi lần. Rủi ro còn lại thấp (trang web khác chỉ làm hiện tên và ngày giỗ trên chính
  màn hình người dùng, cần biết UUID). Chấp nhận; nên ghi đúng như vậy trong 08 §14 #12 thay vì như một biện pháp riêng.

### DV25-07 — góp ý — Mô tả PR #25 đã cũ

- `LichViet.exe --kiem-thong-bao` (nay là `lvhv-ci-check.exe`), "Ảnh chụp: 204 tệp" (CI đòi ≥ 208), dòng tệp đính kèm chưa
  nhắc hộp xác nhận trước khi mở PDF, và bảng "Vòng sửa" chưa có các mục của vòng này. Nên cập nhật trước khi gộp để người đọc lịch
  sử không hiểu sai.

### Ngoài phạm vi vòng này (ghi để không quên)

- Màn hình tưởng niệm Windows vẫn vẽ mâm ngũ quả (ảnh `nt-21b`), trong khi 08 §16.1 (đã vào `main`, áp cho cả bốn nền
  tảng) bỏ mâm vẽ. Theo 07 §10, đây là việc T26 trên nhánh riêng `v0.2-yc10-windows`, không phải điều kiện của PR #25;
  nhưng nếu #25 gộp trước T26 thì `main` tạm có bản Windows trái 08 §16 — nên ghi một dòng vào 08 §14 hoặc mô tả PR.

## 4. Kết luận

| Mức | Số | Mã |
|---|---|---|
| Chặn | **0** | — |
| Nên sửa | 1 | DV25-01 |
| Góp ý | 6 | DV25-02 … DV25-07 |

**GỘP ĐƯỢC.** Mọi mục được giao ở vòng này đã sửa phần chính và có bằng chứng (mã, kiểm thử, CI Windows thật run 296,
ảnh). Không có lỗi chặn mới. Đề nghị sửa DV25-01 (một đoạn tài liệu và chú thích, hoặc thêm điều khiển cho trường hợp gỡ)
trong PR này hoặc ngay sau khi gộp; DV25-04 và DV25-07 là sửa chữ, nên làm cùng lúc.

Việc còn lại phải thử trên máy thật (không đổi so với vòng trước): R5 Windows (thông báo đúng giờ khi ứng dụng đóng, sau
khởi động lại), giao diện bộ cài khi cài và khi gỡ (DV25-01).
