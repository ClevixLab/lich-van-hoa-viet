# Duyệt mã và thiết kế PR #46 — Android YC-2026-10: ảnh mâm lễ (T24)

Trạng thái: **XONG** — 2026-11-07. Kết luận ở §5.

Bên duyệt: vai rà soát (07 §10.3); không viết mã của PR này và không sửa nhánh đang duyệt.

| PR | Nhánh | Đầu nhánh được duyệt | So với |
|---|---|---|---|
| #46 | `v0.2-yc10-android` | `50af90a` (đầu PR hiện là `02884e0`, chỉ thêm ảnh chụp Windows `[skip ci]`, không đổi `apps/android`) | `git diff origin/main...HEAD -- apps/android` |

Căn cứ: 08 §16 (+ §3.4, §3.9, §13 Android); 04 R15; 01 §3.7, "Ghi chú YC-2026-10", "Bổ sung 01-YC10-a";
`docs/reviews/DUYET-MA-YC10-CORE.md` (MA10-11).

Mức lỗi: **chặn** / **nên sửa** / **góp ý**. Mã: `P46-xx`.

## 0. Đã đọc / đã kiểm

| Việc | Kết quả |
|---|---|
| Toàn bộ diff `apps/android` (10 tệp mã + 25 ảnh): `build.gradle.kts`, `MainActivity.kt`, `people/TrayPack.kt`, `ui/people/{Forms,MemorialScreen,PeopleModule,PersonScreen,TrayForm}.kt`, `res/xml/file_paths.xml`, `test/.../MemorialTrayTest.kt` | Đã đọc; kèm `people/Photos.kt`, `AndroidManifest.xml`, `Widgets.kt` (không đổi) để lần đường xử lý ảnh |
| CI trên `50af90a` | `ui`, `build`, `upgrade`, `screenshots`, "Gói ảnh mâm lễ (R15 (b))", Rust: `success`. Nhật ký `ui` và `build` có `> Task :app:trayPack` chạy trước `preBuild`, `BUILD SUCCESSFUL` |
| Ảnh chụp CI trong nhánh (`apps/android/screenshots/and-yc10-*`, `and-11-*`, `and-12-*`, `and-38-*`) | Đã xem từng ảnh (§3). Bản sáng và tối của trang tưởng niệm trùng từng byte — đúng 08 §3.9 ("cùng một kiểu ở sáng và tối") |
| Không chạy được trình giả lập ở vai này | Đường chụp ảnh bằng máy ảnh chỉ suy từ mã (xem P46-04) |

## 1. Đối chiếu đặc tả

| Yêu cầu | Kết quả |
|---|---|
| 08 §16.1 thứ tự: ảnh gia đình → ảnh Clevix cùng `tray_style` → không mâm | ĐÚNG (`MemorialScreen.kt:143-152`) |
| §16.1 / R15 (a): không mâm vẽ; nến · bát hương ba nén · nến; Công giáo nến · hoa · nến, không hương | ĐÚNG. Đã xóa hẳn mâm ngũ quả trong `Altar`; nến dời về 0.24/0.76. Nhãn `ALTAR_DESC*` không còn chữ "mâm". Ảnh `and-yc10-1-*`, `and-yc10-4-*` khớp |
| §16.1 ảnh dưới hàng bàn thờ, ≤ 480 dp, 360: toàn chiều rộng trừ lề 16, 4:3 cắt giữa, bo 12, viền vàng đồng; nhãn "Ảnh mâm lễ của gia đình" / "Ảnh mâm lễ" | ĐÚNG (`TrayPhoto`). Đo trên ảnh: 1280 dp → 480 px rộng ở mdpi; 360 dp → 656 px / xhdpi = 328 dp = 360 − 2×16 |
| §16.1 dòng ghi công "Ảnh: <tác giả> · <giấy phép>" | ĐÚNG khi đủ trường (xem góp ý P46-11) |
| §16.1 nút "Thêm ảnh mâm lễ của gia đình" dưới lời tưởng niệm, mở biểu mẫu và cuộn tới mục; ẩn khi có ảnh gia đình, ẩn ở chế độ chỉ đọc | ĐÚNG (`MemorialScreen.kt:161-165`, `PRoute.EditPerson(tray = true)`, `BringIntoViewRequester`). Ảnh `and-yc10-form-chua-co-anh-360-light` cho thấy đã cuộn tới mục |
| §16.2 vị trí ngay sau "Lời tưởng niệm"; câu chữ "Ảnh mâm lễ ngày giỗ", "Chưa có ảnh. Ngày giỗ chỉ hiện bát hương và nến.", "Chọn ảnh mâm lễ…", "Đang xử lý ảnh…", "Đổi ảnh", "Bỏ ảnh", hai câu lỗi, dòng gợi ý | ĐÚNG từng chữ (đã so với 08 §16.2; kiểm thử `formSectionAndProfileLine` khẳng định phần lớn) |
| §16.2 thu nhỏ 120 × 90 | ĐÚNG (`Modifier.size(120.dp, 90.dp)`, `ContentScale.Crop`) |
| §16.2 đang xử lý → vô hiệu hóa Lưu | ĐÚNG (`Forms.kt:344`), nhưng xem P46-01 |
| §16.2 / 01 §3.7: cạnh dài ≤ 1600, ≤ 1 MB, mã hóa lại, bỏ siêu dữ liệu, không phóng to | ĐÚNG: `Photos.encode(raw, 1600, 1 MB)` — `ImageDecoder.setTargetSize` (API ≥ 28) hoặc `inSampleSize` + `ExifInterface` (26–27), chặn > 12 000 px / > 50 Mpx trước khi giải mã, vẽ lại trên nền trắng, `Bitmap.compress` JPEG (không chép APP1/APP13). Đầu vào ≤ 25 MB, đọc có giới hạn khi đi qua `ContentResolver` |
| §16.2 đổi / bỏ có hiệu lực khi Lưu; chỉ ghi vào kho khi Lưu | ĐÚNG: ảnh chỉ giữ trong `PersonFormState.tray` tới `save()`; `addPhoto` ngay trước `savePerson`, lỗi → `deletePhoto` cả ảnh mâm lễ mới. Hủy biểu mẫu không để lại gì trong kho |
| MA10-11 (gộp xóa ảnh đã thêm chưa gắn) | Android **tránh được** theo thiết kế: không có ảnh chờ gắn trong kho trong lúc biểu mẫu mở |
| `tray_photo_id` giữ ở mọi đường lưu | ĐÚNG: `build()` chép `memorial` từ bản cũ, chỉ đổi khóa khi có ảnh mới / bỏ ảnh; `savePerson` chỉ có một chỗ gọi; `markDeceased` (sống → mất) không đụng `memorial`; Android không có đường đổi mất → sống, nên câu "Ảnh mâm lễ cũng sẽ bị xóa." của §16.2 không áp dụng. Kiểm thử hồi quy `editingAnotherFieldKeepsTrayPhoto` sửa hai ô khác qua biểu mẫu thật (khối Tưởng niệm đóng rồi mở) → id và tệp còn |
| §16.2 dòng hồ sơ "Ảnh mâm lễ: Đã có / Chưa có" | ĐÚNG (`PersonScreen.kt:198-199`, chỉ người đã mất) |
| §16.3 gói Clevix nhúng lúc dựng, chỉ mục đã duyệt, đúng `sha256`, không tải mạng; R15 (b) | ĐÚNG: `trayPack` gọi `scripts/check-tray-pack.py` (lỗi → `GradleException`), chép đúng tệp đã duyệt vào `assets/trays/`; lúc chạy `TrayPack` kiểm lại `sha256`, tệp hỏng → bỏ. Gói rỗng → chỉ `approved.json` rỗng |
| Không xin quyền máy ảnh | ĐÚNG: manifest không có `CAMERA`; chụp qua `ACTION_IMAGE_CAPTURE` của ứng dụng máy ảnh; thư viện qua `ACTION_GET_CONTENT image/*` (không quyền bộ nhớ) |
| FileProvider / tệp tạm | Phạm vi đúng: `cache/capture/mam.jpg` (tên chung, không chứa tên người), provider không exported, quyền đọc/ghi chỉ trên Intent chụp, `revokeUriPermission` + xóa sau xử lý, trước mỗi lần mở hộp chọn và khi mở ứng dụng. Có một đường rò (P46-01) |
| Không lộ ra thông báo / .ics / tên tệp | ĐÚNG: không mã nào ngoài `ui/people/{Forms,MemorialScreen,PersonScreen,TrayForm,PeopleModule}` và `TrayPack` nhắc tới `tray`; tệp tạm tên chung; phần lõi đã kiểm ở MA10-07 |
| Sửa bố cục bia 1280 dp | ĐÚNG: đổi thứ tự `widthIn(max).fillMaxWidth(0.8f)` → bia không còn bị kéo giãn; ảnh `and-yc10-1-khong-mam-1280-*` cho thấy bia 360 dp ở giữa |
| §16.4 ảnh (1), (2), (4) ở 360 và 1280, sáng và tối | CÓ đủ 12 ảnh trong nhánh |

## 2. Phát hiện

### Chặn

Không có.

### Nên sửa

#### P46-01 — Thu gọn khối "Tưởng niệm" trong lúc đang xử lý ảnh → nút Lưu bị khóa vĩnh viễn, tệp tạm không bị xóa

`TraySection` chạy việc xử lý bằng `rememberCoroutineScope()` của chính nó (`TrayForm.kt:80`, `:90-100`), nhưng
`TraySection` chỉ nằm trong cây khi khối "Tưởng niệm" đang mở (`Forms.kt:290-300`). Bấm "Thu gọn ▴" trong lúc
"Đang xử lý ảnh…" (vài trăm ms tới vài giây với ảnh 12–50 Mpx trên máy yếu) → mục rời cây → scope bị hủy →
`withContext` ném `CancellationException` → hai dòng sau không chạy:

- `f.trayBusy = false` không bao giờ chạy; `f` sống ở `PersonForm` nên Lưu (`enabled = !busy && !f.trayBusy`) bị khóa
  tới khi rời biểu mẫu; mở lại khối chỉ thấy "Đang xử lý ảnh…", không có nút nào. Người dùng chỉ còn cách Hủy và mất
  mọi chỉnh sửa khác.
- `TrayCapture.cleanup` không chạy: `cache/capture/mam.jpg` (ảnh gốc từ máy ảnh, **còn EXIF/GPS**) nằm lại tới lần mở
  hộp chọn sau hoặc lần mở ứng dụng sau, quyền URI cấp cho ứng dụng máy ảnh chưa thu hồi.

Sửa: đặt `trayBusy = false` và `cleanup` trong `finally` (bọc `withContext(NonCancellable)` cho phần dọn), hoặc chuyển
launcher + scope lên `PersonForm` như bộ chọn ảnh chân dung (`Forms.kt:188-194`). Thêm một ca kiểm thử: đặt
`trayBusy`, thu gọn khối, khẳng định Lưu bật lại / tệp tạm không còn.

#### P46-02 — Bỏ qua `resultCode`: ảnh người dùng đã hủy ở máy ảnh vẫn được nhận

`TrayForm.kt:84-87` không xét `r.resultCode`. Nhiều ứng dụng máy ảnh ghi `EXTRA_OUTPUT` ngay khi bấm chụp, rồi người
dùng bấm "Hủy/Chụp lại → quay về" ở màn xem lại → `RESULT_CANCELED` nhưng `mam.jpg` khác rỗng → ứng dụng xử lý và
hiện ảnh người dùng đã từ chối như ảnh đã chọn. Không mất dữ liệu (chưa Lưu), nhưng sai ý người dùng. Sửa:
`if (r.resultCode != Activity.RESULT_OK) { TrayCapture.cleanup(ctx); return }`.

#### P46-03 — Đường biểu mẫu "Đổi ảnh" / "Bỏ ảnh" + Lưu chưa có kiểm thử

Ca §16.4 (2) tạo ảnh bằng `Photos.encode` rồi `savePerson` trực tiếp; ca (3) xóa khóa rồi `savePerson` trực tiếp. Cả hai
bỏ qua `PersonFormState.build()/save()` — đúng chỗ mã mới quyết định ghi/bỏ `tray_photo_id`. Kiểm thử hồi quy hiện chỉ
chứng minh **giữ** khóa. Thêm ca ở mức `PersonFormState` (không cần launcher): (a) `tray = Photos.encode(exifJpeg)` →
`save` → khóa mới, ảnh cũ bị lõi xóa (bản ghi và tệp); (b) `trayRemoved = true` → `save` → không còn khóa, không còn
tệp (R15 (f), 0 mồ côi); (c) `savePerson` lỗi (ví dụ hồ sơ sai) → ảnh mâm lễ vừa `addPhoto` bị xóa; (d) mới chọn rồi
"Bỏ ảnh" trên hồ sơ chưa có ảnh → không có khóa.

#### P46-04 — Chưa có bằng chứng đường chụp ảnh chạy trên máy thật

Chụp ảnh đi qua `EXTRA_OUTPUT` nằm trong `EXTRA_INITIAL_INTENTS` của hộp chọn; quyền ghi URI phụ thuộc hệ thống chuyển
`EXTRA_OUTPUT` sang `ClipData` cho Intent lồng, và hành vi trả kết quả khác nhau giữa các ứng dụng máy ảnh (`data` rỗng,
`data` = URI đầu ra, ảnh xoay bằng EXIF). Robolectric không thử được phần này, CI không có ca nào. Cần một lần thử ghi
biên bản (trình giả lập API 26, 28, 34 với ứng dụng máy ảnh mặc định, ảnh dọc và ngang; thư viện ảnh qua `GET_CONTENT`
trên API 33+ mở Photo Picker), hoặc một ca instrumented trong job `upgrade` (đã có trình giả lập API 34) dùng
`Intents.intending(hasAction(ACTION_IMAGE_CAPTURE))` ghi sẵn tệp có EXIF vào URI đầu ra.

#### P46-05 — Tài liệu lệch với mã

- `res/xml/file_paths.xml` đầu tệp vẫn ghi "Chỉ hai thư mục tạm: cache/ics/ … cache/att/" — nay có ba.
- 08 §13 (Android) chưa có dòng YC-2026-10; dòng "Ảnh (§3.4, §12g)" nói Photo Picker, còn mục mâm lễ dùng
  `GET_CONTENT` + `ACTION_IMAGE_CAPTURE` qua hộp chọn; dòng tệp đính kèm báo tử ghi "FileProvider … chỉ `cache/ics/` và
  `cache/att/`". Thêm một dòng §13: bộ chọn, `cache/capture/mam.jpg` và vòng đời của nó, không quyền CAMERA, bước dựng
  cần `python3`, trang tưởng niệm sáng ≡ tối.
- `RA-SOAT-BAO-MAT-V02*.md` liệt kê thư mục FileProvider của Android: cần ghi thêm `capture/` để vòng rà soát bảo mật
  sau không đối chiếu với danh sách cũ.

### Góp ý

- **P46-06** — Nhánh máy ảnh `captured.readBytes()` đọc hết tệp rồi mới so 25 MB (`TrayForm.kt:93`); so
  `captured.length()` trước khi đọc.
- **P46-07** — Khóa việc "không xin CAMERA" bằng `<uses-permission android:name="android.permission.CAMERA"
  tools:node="remove"/>`: nếu sau này một thư viện đưa `CAMERA` vào manifest gộp mà ứng dụng chưa được cấp, mọi lần
  `ACTION_IMAGE_CAPTURE` sẽ ném `SecurityException`.
- **P46-08** — `trayPack` gọi `python3` ở mọi bản dựng (manifest luôn có). Máy dựng không có `python3` (Windows thường
  chỉ có `py`/`python`) nhận `IOException` khó hiểu. Bắt lỗi khởi chạy và ném `GradleException` có câu hướng dẫn; ghi
  yêu cầu vào hướng dẫn dựng Android.
- **P46-09** — Bước Gradle mới chỉ được CI chạy với gói rỗng. Kịch bản kiểm gói (job "Gói ảnh mâm lễ") đã thử gói xấu,
  nhưng phần Gradle (mã thoát → bản dựng thất bại; chép tệp trong thư mục con; `approved.json` vào APK) chưa được thử.
  Có thể thêm `-Plvhv.trayManifest=content/memorial/trays/test/<ca>/manifest.json` và một bước CI chạy
  `:app:trayPack` mong thất bại / mong có tệp.
- **P46-10** — `trayError` đưa "Ảnh quá lớn (W × H điểm ảnh)" và "Ảnh quá chi tiết…" về "Không đọc được ảnh này…".
  Đúng hai câu của §16.2, nhưng ảnh 60 Mpx hợp lệ sẽ bị báo "không đọc được"; cân nhắc dùng "Ảnh quá lớn (tối đa 25 MB)."
  cho ca vượt trần điểm ảnh hoặc xin điều phối thêm một câu.
- **P46-11** — Dòng ghi công dựng từ `tac_gia`, `giay_phep` nhưng điều kiện hiện là `ghi_cong != null`; thiếu một trong
  hai trường → "Ảnh:  · CC BY 4.0". Cho `check-tray-pack.py` bắt buộc `tac_gia`, `giay_phep` khi `ghi_cong` khác null.
- **P46-12** — `TrayPack.items` đọc assets và `rememberPhoto` giải mã ảnh mâm trên luồng giao diện (giống ảnh chân dung
  hiện có). Gói ≤ 2 MB, ảnh ≤ 1600 px nên chấp nhận được; về sau có thể dùng `produceState` + `Dispatchers.IO`.
- **P46-13** — Trợ năng: "Đang xử lý ảnh…" thay chỗ nút vừa bấm, không có `liveRegion` → TalkBack mất tiêu điểm, không
  báo; sau "Bỏ ảnh" tiêu điểm cũng rơi. Thêm `liveRegion = Polite` cho dòng xử lý và đưa tiêu điểm về "Chọn ảnh mâm lễ…".
  Thứ tự đọc trên trang tưởng niệm hợp lý (bàn thờ → ảnh mâm → ghi công → lời tưởng niệm → nút thêm ảnh); nút chữ đủ
  48 dp; tương phản `GOLD_DIM` trên nền ≈ 7,9 : 1. Chưa có ảnh chụp cỡ chữ lớn cho mục biểu mẫu và cho ảnh mâm / nút
  thêm ảnh (ảnh `and-11-tuong-niem-large` dừng ở bàn thờ); nên thêm một ảnh `large` cho mỗi chỗ.
- **P46-14** — Công giáo: `Altar` cao cố định 220 dp mà bình hoa thấp → khoảng trống lớn phía trên (ảnh
  `and-yc10-4-cong-giao-360`). Chỉ là thẩm mỹ.
- **P46-15** — Khi tiến trình bị hệ thống dừng trong lúc mở máy ảnh, `MainActivity.onCreate` xóa `mam.jpg` trước khi
  kết quả về, và biểu mẫu (giữ bằng `remember`) cũng mất — giống bộ chọn ảnh chân dung hiện có. Tốt cho riêng tư; ghi
  nhận, không cần sửa trong YC này.
- **P46-16** — `TrayPack.override` là móc kiểm thử công khai trong mã phát hành; đánh dấu `@VisibleForTesting`.

## 3. Ảnh chụp đã xem

| Ảnh | Nhận xét |
|---|---|
| `and-yc10-1-khong-mam-360-*`, `-1280-*` | Nến · bát hương ba nén có khói · nến; không mâm, không lọ hoa. 1280: bia 360 dp ở giữa, không giãn; có lời tưởng niệm và nút "Thêm ảnh mâm lễ của gia đình" |
| `and-yc10-2-anh-gia-dinh-360-*`, `-1280-*` | Ảnh nằm dưới bàn thờ, bo góc, viền vàng đồng, 4:3; 360 đủ lề 16 dp, 1280 rộng 480 dp |
| `and-yc10-4-cong-giao-360-*`, `-1280-*` | Nến · hoa · nến, không hương; khoảng trống trên bình hoa (P46-14) |
| `and-yc10-form-chua-co-anh-360-light`, `and-yc10-form-co-anh-360-light` | Mục nằm ngay sau "Lời tưởng niệm", đã cuộn tới; có ảnh: thu nhỏ 120 × 90 + "Đổi ảnh", "Bỏ ảnh"; dòng gợi ý đủ chữ |
| `and-11-*`, `and-12-*`, `and-38-*` (cập nhật) | Không còn mâm vẽ; bố cục khác không đổi |

## 4. Kiểm thử trong PR

`MemorialTrayTest` (Robolectric, chạy trong job `ui`): ca (1), (2), (4) ở 360/1280 có ảnh chụp; (3) bỏ ảnh → không còn
tệp; (5) gói thử một ảnh → ảnh Clevix + ghi công, rồi ảnh gia đình thắng; câu chữ biểu mẫu và dòng hồ sơ; hồi quy giữ
`tray_photo_id` khi sửa ô khác. Ảnh mẫu có APP1 chứa "GPS"; ảnh lưu được khẳng định không còn APP1/APP13, cạnh dài
1600, ≤ 1 MB. Thiếu: đường biểu mẫu đổi/bỏ (P46-03), đường máy ảnh (P46-04), ca thu gọn khi đang xử lý (P46-01).

## 5. Kết luận

**GỘP ĐƯỢC SAU KHI SỬA P46-01 và P46-02** (khuyến nghị) — **0 chặn**, 5 nên sửa, 11 góp ý.

R15 (a) trên Android: **ĐẠT** theo kiểm thử và ảnh chụp. R15 (c) phần giao diện (thu nhỏ, bỏ siêu dữ liệu trước khi gọi
lõi): **ĐẠT** trên đường thư viện ảnh qua `Photos.encode`; đường máy ảnh: **CHƯA ĐỦ BẰNG CHỨNG** tới khi có P46-04.
Ghi chỉ khi Lưu, giữ `tray_photo_id` trên mọi đường lưu, không xin quyền máy ảnh, không lộ ra thông báo/.ics/tên tệp:
đúng. P46-01 và P46-02 là lỗi trạng thái nhỏ, sửa vài dòng; P46-03–05 nên đi cùng vòng sửa này để phần kiểm thử và tài
liệu khớp với mã.
