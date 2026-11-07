# Duyệt PR #47 — Apple (iPhone, iPad, Mac) YC-2026-10: ảnh mâm lễ

- PR: #47, nhánh `v0.2-yc10-apple`. Mã của PR là commit `ab6d475`; đầu nhánh lúc duyệt là `8fb38ed` (hai commit ảnh
  chụp do CI ghi: `d0cf497` Windows, `8fb38ed` Apple upgrade). So với `origin/main` (`bf03033`).
- Ngày duyệt: 07/11/2026. Người duyệt: một bên không tham gia xây dựng mã của PR này.
- Căn cứ: `docs/v0.2/08` §16 (và §3.4, §3.9, mục 12-Apple), `04` R15, `01` §3.7 và "Bổ sung 01-YC10-a",
  `docs/reviews/DUYET-MA-YC10-CORE.md` (MA10-11).
- Phạm vi mã: `TrayPhoto.swift` (mới), `PersonFormView`, `PeopleDrafts`, `PersonDetailView`, `PeopleView`,
  `MemorialView`, `DemoSeed`, `gen-trays.sh` (mới), `build-xcframework.sh`, `project.yml`, `.github/workflows/apple.yml`,
  `Tests/UI/PeopleUITests.swift`, `Tests/Unit/PeopleGlueTests.swift`, một dòng 08 §12-Apple.
- Cách duyệt: đọc mã và CI. Không có Xcode; không chạy được kiểm thử Apple tại chỗ.

## 1. Bằng chứng CI

| Lần chạy | Kết quả |
|---|---|
| `apple` 42370122179 lần 1 (`ab6d475`) | Dựng iOS mô phỏng, iOS arm64, macOS: **đạt** (mã biên dịch được trên cả ba). Bước "Kiểm thử đơn vị + giao diện (iPhone)" **bị hủy** giữa chừng; iPad, ghi ảnh chụp, chạy thử: không chạy. Nhật ký lần 1 không còn tải được |
| `apple` upgrade lần 1 | R14 bị hủy, nhưng bước "Ghi ảnh nâng cấp" vẫn chạy và đẩy `8fb38ed` **xóa** hai ảnh R14 (C) (xem C-2) |
| `apple` 42370122179 lần 2 | Đang xếp hàng lúc viết |
| Các lần chạy trên `8fb38ed` (apple, ci, android, windows) | `action_required` — chưa được phê duyệt chạy |
| `windows` 42370122166 | đạt; đẩy `d0cf497` (xem N-1) |

Kết quả: **chưa có lần nào** `testMemorialAltar`, `testTrayPhoto`, `testTrayPhotoSavedStrippedAndRemoved`,
`testTrayPackKeepsOnlyApprovedVerifiedItems`, `testEverySavePathKeepsTrayPhoto` chạy xong. Thư mục
`apps/apple/screenshots/` không có ảnh `ios-altar-*`, `ios-form-tray-*`, `ipad-*`, `ios-memorial-cg-*` mà mô tả PR nêu
— không có ảnh nào để xem.

## 2. Đối chiếu đặc tả

| Mục | Kết quả |
|---|---|
| §16.1 hàng bàn thờ: nến · bát hương ba nén · nến; Công giáo nến · hoa · nến; bỏ `fruit` | Đúng (`MemorialView.altar`); nhãn "Bàn thờ: hai cây nến và bát hương với ba nén hương" / "Bàn thờ: hoa và hai cây nến" không nhắc mâm |
| §16.1 thứ tự ảnh gia đình → Clevix cùng `tray_style` → không có | Đúng (`TrayResolve.choose`) |
| §16.1 khung: dưới bàn thờ, ≤ 480, lề 16, cắt 4:3 giữa, bo 12, viền vàng đồng | Đúng (`TrayImageView`; `padding(.horizontal, 16)` của trang) |
| §16.1 nhãn "Ảnh mâm lễ của gia đình" / "Ảnh mâm lễ"; dòng "Ảnh: <tác giả> · <giấy phép>" | Đúng; dòng ghi công chỉ hiện khi mục có `ghi_cong` |
| §16.1 nút "Thêm ảnh mâm lễ của gia đình" dưới lời tưởng niệm, ẩn khi có ảnh gia đình, mở biểu mẫu và cuộn tới mục | Đúng (`PRoute.editTray`, `focusTray`, `scrollTo("form.traySection")`); ẩn ở chế độ chỉ đọc — hợp lý. Lưu xong quay lại màn hình tưởng niệm |
| §16.2 vị trí ngay sau "Lời tưởng niệm"; tiêu đề "Ảnh mâm lễ ngày giỗ" | Đúng |
| §16.2 chữ: "Chưa có ảnh. Ngày giỗ chỉ hiện bát hương và nến.", "Chọn ảnh mâm lễ…", "Đang xử lý ảnh…", "Đổi ảnh", "Bỏ ảnh", hai câu lỗi, dòng gợi ý | Đúng từng chữ |
| §16.2 "Đang xử lý" khóa Lưu | Đúng: `disabled(trayBusy)` ở cả hai nút Lưu và chốt `guard !trayBusy` trong `save()` |
| §16.2 ảnh thu nhỏ 120 × 90 | Đúng (`PhotoView … size 120×90, oval: false`) |
| §16.2 trang hồ sơ "Ảnh mâm lễ: Đã có / Chưa có" | Đúng (`InfoRow`, chỉ người đã mất) |
| §16.2 câu "Ảnh mâm lễ cũng sẽ bị xóa." | Không áp dụng: đã kiểm, biểu mẫu sửa giữ nguyên `kind`; `MarkDeceasedView` chỉ đi một chiều sống → mất. Đã ghi ở 08 §12-Apple |
| 01 §3.7 null ≡ thiếu, không ghi khóa | Đúng: `mem.removing("tray_photo_id")` rồi chỉ ghi khi có id; kiểm thử đơn vị kiểm chuỗi JSON |
| 01 §3.8 / S9 xử lý ảnh | Đúng (xem §3) |
| 01 Bổ sung YC10-a, MA10-11 | Đúng và chặt hơn yêu cầu: ảnh mới **chỉ** `add_photo` lúc bấm Lưu (`save()`), nên Hủy không để lại tệp và Gộp lúc biểu mẫu đang mở không thể xóa ảnh chưa gắn. Lưu hỏng → `discardPhoto` ảnh vừa ghi |
| Giữ `tray_photo_id` trên mọi đường lưu | Đã kiểm: đường ghi hồ sơ duy nhất của app là `PeopleModel.savePerson` (biểu mẫu, và `DemoSeed` chỉ ở Debug), đều đi qua `PersonDraft(person:)` → `build`, giữ `trayPhotoId`. `markDeceasedJson` chỉ cho người đang sống (chưa có ảnh). Kiểm thử hồi quy `testEverySavePathKeepsTrayPhoto` (7 trường + thêm ảnh chân dung) |
| 08 §16.3 / R15 (b) gói | `gen-trays.sh` chạy `check-tray-pack.py --manifest … --out`, `set -euo pipefail` → kiểm không đạt thì dừng dựng; chép đúng tệp đã duyệt, chặn tên có `/` hay bắt đầu bằng `.`. Lúc chạy `TrayPack.load` kiểm lại `trang_thai`, MIME, có tệp, `sha256`. Gói thử chỉ có ở Debug |
| Không lộ ra thông báo, .ics, tên tệp | Đúng: id ảnh không đi vào văn bản nào của app; tệp lưu theo UUID của lõi; kiểm thử đơn vị kiểm .ics. Thông báo do lõi dựng (MA10-07 đã kiểm ở lõi) |
| Info.plist | `NSCameraUsageDescription` ở `project.yml` (iOS). Không cần quyền thư viện (PhotosPicker). Mac không dùng máy ảnh |

## 3. Xử lý ảnh

`TrayPhoto.process` / nhánh Mac đều gọi `PhotoProcessor.process(data, maxSide: 1600, maxBytes: 1_048_576)`:

- Kiểm ≤ 25 MB trước khi giải mã; Mac kiểm `fileSizeKey` **trước** khi đọc tệp.
- Đọc kích thước từ phần đầu (`CGImageSourceCopyPropertiesAtIndex`), từ chối > 12 000 px mỗi cạnh hoặc > 50 Mpx.
- Giải mã có lấy mẫu xuống bằng `CGImageSourceCreateThumbnailAtIndex` (`MaxPixelSize` = 1600, `WithTransform` xoay theo
  EXIF), không giải mã đầy đủ ảnh gốc; không phóng to.
- Vẽ lại vào `CGContext` sRGB mới (bỏ alpha, nền trắng), mã hóa JPEG với chất lượng giảm dần 0,85 → 0,3 tới khi ≤ 1 MB,
  rồi `JPEGSanitizer.strip` bỏ APP1…APP15 và COM. Không đạt → `stillTooBig`.
- HEIC: PhotosPicker trả byte gốc (`loadTransferable(type: Data.self)`), ImageIO giải mã HEIC; đường xử lý giống JPEG.
  Ảnh 48 MP (≈ 48,8 Mpx) vẫn dưới trần 50 Mpx. ProRAW lớn hơn 25 MB → "Ảnh quá lớn (tối đa 25 MB)." Cần thử máy thật
  như PR đã ghi.
- Máy ảnh: `UIImage.jpegData(0.9)` (hướng ghi trong EXIF, được áp dụng rồi bỏ ở bước trên).
- Kiểm thử đơn vị: ảnh 3000 × 2000 có GPS + tên máy → ≤ 1600 px, ≤ 1 MB, không còn `0xE1`, không còn chuỗi tên máy.

## 4. Phát hiện

### Chặn

**C-1 — Chưa có bằng chứng kiểm thử và ảnh chụp trên đầu nhánh.** Lần chạy `apple` duy nhất của mã PR bị hủy ngay ở bước
kiểm thử; các lần chạy trên `8fb38ed` đang `action_required`. Không có kết quả nào của năm kiểm thử mới, không có ảnh
chụp iPhone/iPad nào của §16.4 (1)(2)(4). R15 (a) là điều kiện chặn phát hành và phương pháp của R15 đòi kiểm thử giao
diện + ảnh chụp sáng/tối. Cần: một lần `apple` chạy trọn (iPhone + iPad) **xanh** trên đầu nhánh, ảnh
`ios-altar-*`, `ios-form-tray-*`, `ipad-altar-*`, `ios-memorial-cg-*` được ghi vào nhánh, rồi duyệt lại ảnh.
Lưu ý thêm N-2: có nguy cơ thật `testTrayPhoto` sẽ trượt ở `form.tray.thumb`.

**C-2 — Commit CI `8fb38ed` xóa hai ảnh bằng chứng R14 khỏi `main` nếu gộp.** Job `upgrade` bị hủy giữa bước R14 nhưng bước
"Ghi ảnh nâng cấp" vẫn chạy và đẩy `apps/apple/screenshots/upgrade/upgrade-readonly-diag.png` và
`upgrade-readonly-profiles.png` ở trạng thái **đã xóa** (384 880 và 407 354 byte → 0). Gộp nguyên trạng làm mất bằng chứng
R14 (C) trên `main`. Cần khôi phục hai tệp (hoặc bỏ commit này) trước khi gộp. Nên sửa thêm ở workflow: bước ghi ảnh
upgrade chỉ chạy khi bước R14 `success`, và không bao giờ ghi việc xóa tệp.

### Nên sửa

**N-1 — PR có kèm `apps/windows/screenshots/*` (commit CI `d0cf497`) không thuộc PR này.** Sáu tệp:
`bo-nho.txt`, `nang-cap.txt` và bốn ảnh `nt-21b-tuong-niem-khoi-dang-chay-*` — chỉ là số đo bộ nhớ, số lần chạy, GUID
sản phẩm và khung khói khác vài byte; không có thay đổi Windows nào trong PR. Chúng làm nhiễu diff, lẫn bằng chứng Windows
của PR #56/#62 và gây xung đột không cần thiết. Cần bỏ commit này khỏi nhánh (hoặc khôi phục sáu tệp về `origin/main`).
Gốc vấn đề: `windows.yml` chạy với mọi `pull_request` và ghi ảnh vào nhánh PR bất kể PR có chạm `apps/windows/` hay không —
nên thêm `paths` cho `pull_request` (có thể nhánh `v0.2-ci-apple-paths` / `ci-android-paths` đang làm việc tương tự).

**N-2 — Ảnh thu nhỏ trong biểu mẫu bị ẩn khỏi trợ năng, và kiểm thử dựa vào nó.** `PhotoView` đặt
`.accessibilityHidden(true)`; `PersonFormView.swift:617` gắn `.accessibilityIdentifier("form.tray.thumb")` lên phần tử đã
ẩn. (a) Người dùng VoiceOver không biết đã có ảnh nào, chỉ nghe "Đổi ảnh", "Bỏ ảnh". (b) XCUITest thường không thấy phần
tử bị ẩn, nên `XCTAssertTrue(any(app, "form.tray.thumb").waitForExistence…)` trong `testTrayPhoto` có khả năng trượt (chưa
xác nhận được vì CI chưa chạy tới). Sửa: bọc thumbnail bằng phần tử có `accessibilityLabel("Ảnh mâm lễ đã chọn")`,
`.isImage`, và đặt định danh ở đó.

**N-3 — Hai cặp nút trùng tên trong cùng biểu mẫu.** Mục ảnh chân dung và mục ảnh mâm lễ đều có "Đổi ảnh" và "Bỏ ảnh"
(`PersonFormView.swift:525`, `:532`, `:619`, `:620`). Với VoiceOver / Khẩu lệnh không phân biệt được. Giữ chữ hiển thị theo
§16.2, thêm `accessibilityLabel("Đổi ảnh mâm lễ")` / `("Bỏ ảnh mâm lễ")`.

**N-4 — Mac: hộp chọn tệp ảnh có thể không mở được trong hộp cát.** `macOS/LichViet.entitlements` và `project.yml` chỉ có
`com.apple.security.app-sandbox`, không có `com.apple.security.files.user-selected.read-only` (hay `read-write`). Với app
trong hộp cát thiếu quyền này, `NSOpenPanel` (nền của `fileImporter`) thường không hiện được. Vấn đề có từ trước (khôi phục
sao lưu, chọn PDF cũng dùng `fileImporter`), nhưng PR này đưa đường chọn ảnh mâm lễ duy nhất của Mac lên nó. Cần thử trên
Mac thật (PR đã ghi "Cần máy thật") và, nếu đúng, thêm `com.apple.security.files.user-selected.read-write` (cần cả cho
`fileExporter`). Nếu xác nhận hỏng thì là điều kiện chặn phát hành bản Mac.

**N-5 — Câu xin quyền máy ảnh nói thiếu.** `project.yml:52`: "… Ảnh chỉ lưu trên máy này." trong khi dòng gợi ý §16.2 nói
ảnh còn ở "tệp sao lưu mã hóa". Đề nghị: "Ứng dụng dùng máy ảnh khi bạn chọn chụp ảnh mâm lễ ngày giỗ. Ảnh chỉ lưu trên
máy này và trong tệp sao lưu mã hóa." Thêm vào bước "Kiểm tra" của CI một dòng xác nhận Info.plist đã dựng của app iOS có
`NSCameraUsageDescription` (tệp `iOS/Info.plist` trong kho là bản rỗng, do XcodeGen sinh).

### Góp ý

**G-1 — Giải mã lặp lại.** `MemorialView.swift:116` gọi `TrayResolve.choose` trong `body`; với ảnh Clevix, mỗi lần đánh giá
lại `body` (bấm Dừng khói / Thắp lại, model đổi) đều đọc tệp và giải mã lại (`TrayPhoto.swift:210`), không qua
`PhotoCache`. Tương tự `PhotoView` với `newPhoto` giải mã JPEG 1600 px mỗi lần biểu mẫu vẽ lại (mỗi phím gõ). Nên đệm theo
`it.id` / theo `newTray`.

**G-2 — Máy ảnh mã hóa trên luồng chính.** `TrayPhoto.swift:136` `jpegData(0.9)` của ảnh 12 MP chạy trong delegate (luồng
chính) trước khi `busy = true` hiện "Đang xử lý ảnh…" — có thể khựng 0,3–1 giây. Có thể chuyển `UIImage` sang
`Task.detached` rồi mã hóa ở đó.

**G-3 — Gỡ `TrayPickButton` khi đang xử lý.** Khi `busy = true`, mục ảnh chuyển sang nhánh "Đang xử lý ảnh…", nên
`TrayPickButton` (chủ của `fullScreenCover` / `photosPicker` / `fileImporter` và `Task`) bị bỏ khỏi cây trong lúc lớp chọn
ảnh còn đang đóng. Hiện vẫn đúng vì `Task` không gắn vòng đời view và `onPicked` ghi vào `@State` của biểu mẫu, nhưng dễ vỡ
và có thể sinh cảnh báo trình bày; cần xem trên máy thật cùng ca máy ảnh. Cách chắc hơn: giữ nút trong cây và chỉ ẩn/vô
hiệu.

**G-4 — Lỗi theo điểm ảnh dùng câu "Ảnh quá lớn (tối đa 25 MB)."** Ảnh > 50 Mpx nhưng < 25 MB nhận câu nói về MB. Đặc tả
chỉ có hai câu nên chấp nhận; ghi lại để khớp với PWA/Android.

**G-5 — Phạm vi kiểm thử.** (a) `testEverySavePathKeepsTrayPhoto` đi qua `PersonDraft.build` chứ không qua
`PersonFormView.save()` (nơi có nhánh `newTray` và `!deceased`); thêm một ca giao diện "đổi ảnh rồi Hủy → số ảnh trong kho
không đổi" sẽ khóa được hành vi MA10-11. (b) Không có ảnh chụp biểu mẫu ở chữ rất lớn (chỉ có `ios-memorial-cg-large`); nên
thêm `ios-form-tray-large` để xem hàng ảnh 120 × 90 + hai nút. (c) 08 §12-Apple liệt kê kiểm thử nhưng thiếu
`testEverySavePathKeepsTrayPhoto`. (d) R15 nói ảnh chụp ở 360 và 1280 px: bản Apple có iPhone + iPad, chưa có Mac.

**G-6 — Xung đột với PR #61.** Gộp thử #47 với #61 (`26d37fb`): xung đột ở `apps/apple/Tests/UI/PeopleUITests.swift`; năm
tệp khác tự gộp được. Cần thống nhất thứ tự gộp.

## 5. Kết luận

**CHƯA GỘP ĐƯỢC — 2 chặn, 5 nên sửa, 6 góp ý.**

Mã đúng đặc tả §16 ở mọi chỗ đọc được: câu chữ khớp từng chữ, thứ tự chọn ảnh, khung ảnh, xử lý ảnh (lấy mẫu xuống, ≤ 1600
px, ≤ 1 MB, bỏ siêu dữ liệu, có cả HEIC và tệp trên Mac), ảnh chỉ ghi khi bấm Lưu (đáp ứng MA10-11 chặt hơn yêu cầu), giữ
`tray_photo_id` trên đường lưu duy nhất, gói ảnh kiểm hai lớp. Hai điểm chặn đều nằm ở bằng chứng và nhánh, không ở logic:
C-1 cần một lần CI `apple` chạy trọn và xanh cùng ảnh chụp để duyệt; C-2 cần khôi phục hai ảnh R14 bị commit CI xóa.
N-1 (bỏ ảnh Windows khỏi PR) nên làm cùng lúc. Sau khi có CI xanh và ảnh chụp, người duyệt xem lại ảnh và N-2 trước khi
đổi kết luận. Không đổi gì trên nhánh của PR #47.

---

## Vòng 2 — 07/11/2026

- Đầu nhánh lúc duyệt: `6efec2e`. Mã ở `43a1992` (gộp `origin/main` vào nhánh). Hai commit sau đó (`4e6ce7f`, `6efec2e`)
  do CI ghi và **chỉ** có ảnh trong `apps/apple/screenshots/` (đã kiểm `git diff --name-only 43a1992 6efec2e`).
- So với `origin/main` (`d47812a`): 16 tệp mã/tài liệu + ảnh chụp Apple. Không có tệp nào bị xóa, không có tệp ngoài
  `apps/apple/`, `.github/workflows/apple.yml` và một dòng 08 §12-Apple. Gộp thử với `main` và với PR #61 (`26d37fb`): không
  xung đột.

### V2.1 Bằng chứng CI

`apple` 42379199813 trên `43a1992`: **đạt** cả hai job.

| Bước | Kết quả |
|---|---|
| Dựng iOS mô phỏng, iOS arm64, macOS | đạt |
| Kiểm tra (không quyền mạng, PrivacyInfo, phông) + mới: `NSCameraUsageDescription` trong Info.plist đã dựng, quyền `files.user-selected.read-write` | đạt; nhật ký in "NSCameraUsageDescription: Ứng dụng dùng máy ảnh khi bạn chọn chụp ảnh mâm lễ ngày giỗ. Ảnh chỉ lưu trên máy này và trong tệp sao lưu mã hóa." |
| Đơn vị (22 ca, 0 lỗi) | `testTrayPhotoSavedStrippedAndRemoved`, `testTrayPackKeepsOnlyApprovedVerifiedItems`, `testEverySavePathKeepsTrayPhoto` **đạt** |
| Giao diện iPhone (10 ca chạy, 0 lỗi; 2 ca `PeopleUpgradeTests` bỏ qua đúng thiết kế, chạy ở job upgrade) | `testMemorialAltar` (35,8 s), `testTrayPhoto` (187,2 s) **đạt** |
| Giao diện iPad (3 ca, 0 lỗi) | `testMemorialAltar`, `testTrayPhoto` **đạt** |
| upgrade: R14 + ghi ảnh | đạt |

Các lần chạy trên `4e6ce7f`, `6efec2e` ở trạng thái `action_required` (commit của bot, chỉ có ảnh), nên GitHub hiện PR là
`unstable`. Không ảnh hưởng mã đã kiểm.

### V2.2 Trạng thái phát hiện vòng 1

| Mã | Trạng thái | Bằng chứng |
|---|---|---|
| C-1 | **ĐÃ SỬA** | Lần chạy xanh trọn ở V2.1; năm kiểm thử YC-2026-10 đạt trên iPhone và (hai ca giao diện) iPad; ảnh chụp đã ghi vào nhánh (xem V2.3) |
| C-2 | **ĐÃ SỬA** | Hai ảnh `upgrade-readonly-{diag,profiles}.png` còn nguyên (chỉ chụp lại, 384 880 → 386 191 và 407 354 → 409 469 byte). Gốc đã chặn: `commit_shots.sh` không còn `rm`, chỉ chép ảnh khác rỗng và báo lỗi nếu chỉ mục có dòng `D`; bước ghi ảnh upgrade chỉ chạy khi `steps.r14.outcome == 'success'`, bước ghi ảnh build dùng `!cancelled()` |
| N-1 | **ĐÃ SỬA** | Diff không còn tệp `apps/windows/`. `windows.yml` đã có lọc đường dẫn và chỉ ghi ảnh trên nhánh Windows (#64, trên `main`); `apple.yml` nay cũng có `paths` và chỉ ghi ảnh khi tên nhánh có `apple` |
| N-2 | **ĐÃ SỬA** | Ảnh thu nhỏ được bọc trong `ZStack` có `accessibilityElement(children: .ignore)`, `.isImage`, nhãn "Ảnh mâm lễ đã chọn", định danh `form.tray.thumb`; kiểm thử khẳng định nhãn này |
| N-3 | **ĐÃ SỬA** | Nhãn trợ năng: "Đổi ảnh mâm lễ", "Bỏ ảnh mâm lễ"; mục chân dung: "Chọn ảnh chân dung" / "Đổi ảnh chân dung", "Bỏ ảnh chân dung". Chữ hiển thị giữ đúng §16.2. Kiểm thử khẳng định nhãn "Bỏ ảnh mâm lễ" |
| N-4 | **ĐÃ SỬA (mã)** — còn thử máy thật | `com.apple.security.files.user-selected.read-write` ở `LichViet.entitlements` và `project.yml`; CI kiểm có khóa này. Việc hộp chọn tệp thật sự mở được trên Mac vẫn chỉ xác nhận được trên máy Mac (danh sách "Cần máy thật") |
| N-5 | **ĐÃ SỬA** | Câu xin quyền mới (trên); CI đọc Info.plist đã dựng bằng `PlistBuddy` và trượt nếu thiếu "tệp sao lưu mã hóa" |

Góp ý vòng 1: G-1 sửa một phần (ảnh Clevix nay qua `PhotoCache` với khóa `traypack:<id>`; `PhotoView` vẫn giải mã
`newTray` mỗi lần vẽ lại biểu mẫu); G-2 đã sửa (`jpegData` chạy trong `Task.detached`); G-3 đã sửa (nút chọn ảnh luôn ở
trong cây, chỉ bị khóa khi đang xử lý; có định danh `form.tray.busy`); G-4 giữ nguyên (chấp nhận); G-5 (b)(c) đã sửa
(`ios-form-tray-large*`, `ipad-form-tray-large*`; 08 §12-Apple liệt kê `testEverySavePathKeepsTrayPhoto`), (a)(d) chưa;
G-6 hết (không còn xung đột với #61).

### V2.3 Ảnh chụp đã xem

Đã mở và xem: `ios-altar-huong-light`, `ios-altar-cong-giao-light`, `ios-altar-tray-light`, `ios-altar-tray-dark-2`,
`ios-altar-clevix-dark`, `ios-form-tray-light`, `ios-form-tray-empty-dark`, `ios-form-tray-large`,
`ipad-altar-huong-dark`, `ipad-altar-tray-light`, `ipad-altar-clevix-light`.

| Ca §16.4 | Thấy trong ảnh |
|---|---|
| (1) không ảnh, gói rỗng | iPad: hai nến, bát hương ba nén có khói, kệ gỗ; **không** mâm, không lọ hoa; nút chữ "Thêm ảnh mâm lễ của gia đình" dưới bàn thờ |
| (4) Công giáo | iPhone: nến · hoa · nến, không hương, không nút "Dừng khói"; có nút thêm ảnh |
| (2) ảnh gia đình | iPhone `-dark-2`: ảnh nằm dưới hàng bàn thờ, rộng hết cột trừ lề 16, khung 4:3, bo góc, viền vàng đồng; lời tưởng niệm dưới ảnh; không có nút thêm. iPad: ảnh rộng hơn kệ (≤ 480 so với kệ ≤ 380), căn giữa — đúng §16.1 |
| (5) gói thử | iPad: ảnh Clevix dưới bàn thờ khi chưa có ảnh gia đình |
| Biểu mẫu | Mục ngay sau "Lời tưởng niệm"; chưa có ảnh: đúng câu + "Chọn ảnh mâm lễ…"; có ảnh: thu nhỏ 120 × 90, "Đổi ảnh", "Bỏ ảnh" (đỏ); dòng gợi ý đúng chữ. Chữ rất lớn: không cắt chữ, hàng ảnh + hai nút vẫn vừa |

### V2.4 Phát hiện mới

**V2-01 — góp ý — Ảnh trang đầu trên iPhone không thấy được phần cần chứng minh.** `ios-altar-huong-*`,
`ios-altar-tray-light` và `ios-altar-clevix-*` chỉ chụp màn đầu: hàng bàn thờ bị thanh thẻ che (chỉ thấy đầu nén hương),
nên ảnh không cho thấy ca (1) "không có mâm" hay ảnh Clevix và dòng ghi công trên iPhone. Ca (2) có ảnh cuộn `-2`; các ca
khác chỉ được kiểm bằng khẳng định trong kiểm thử (đã đạt) và bằng ảnh iPad. Đề nghị thêm ảnh sau khi cuộn
(`altar-huong-*-2`, `altar-clevix-*-2`) để ảnh chụp tự đứng làm bằng chứng R15 (a) trên iPhone. Không chặn.

**V2-02 — góp ý — Dòng ghi công Clevix chưa có trong ảnh nào.** Chỉ được khẳng định bằng `memorial.trayCredit`. Gộp với
V2-01.

Không có phát hiện chặn mới.

### V2.5 Kết luận vòng 2

**GỘP ĐƯỢC — 0 chặn.** C-1, C-2, N-1, N-2, N-3, N-5 đã sửa; N-4 đã sửa trong mã và CI, còn chờ thử trên Mac thật như mọi
mục "Cần máy thật" (máy ảnh, ảnh HEIC thật, hộp chọn tệp Mac), không chặn gộp nhưng phải xong trước khi phát hành bản Mac.
Hai góp ý mới (V2-01, V2-02) về ảnh chụp. Không đổi gì trên nhánh của PR #47.
