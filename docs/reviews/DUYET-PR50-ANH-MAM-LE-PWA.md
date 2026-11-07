# Duyệt PR #50 — PWA YC-2026-10 bước 2: ảnh mâm lễ

- PR: #50, nhánh `v0.2-yc10-pwa2`, đầu `ce34c7a`; so với `origin/main`.
- Ngày duyệt: 07/11/2026. Người duyệt: vai duyệt mã và thiết kế, không tham gia viết mã của PR này.
- Căn cứ: `docs/v0.2/08` §16 (và §3.4, §3.9), `04` R15, `01` §3.7 và "Bổ sung 01-YC10-a",
  `docs/reviews/DUYET-MA-YC10-CORE.md` (MA10-11).
- Phạm vi: `pwa/build.mjs`, `pwa/src/people/{forms,index,memorial,notice,views}.ts`, `pwa/src/style.css`,
  `pwa/test/{people-flows,smoke}.mjs`, `CHANGELOG.md` (10 tệp, +406/−16).

## 1. Đã chạy

| Lệnh | Kết quả |
|---|---|
| `cd pwa && npm ci` | đạt |
| `cd tsport && npm run build` (smoke cần `tsport/dist/lvhv.mjs`) | đạt |
| `cd pwa && npm run build` | đạt — "gói ảnh mâm lễ: 0 ảnh đã duyệt (gói rỗng — không hiện mâm)" |
| `npm run typecheck` (sau build, vì cần `src/gen/content.ts`) | đạt, 0 lỗi |
| `npm run test:smoke` | **ĐẠT**, 320 dòng `ok`, 0 lỗi; gồm toàn bộ ca YC-2026-10 / §16.4 (1)–(5), hồi quy giữ ảnh, MA10-11 |

Ảnh chụp: `pwa/test/shots/` bị `.gitignore` bỏ qua (PWA đưa ảnh lên CI dạng artifact `pwa-smoke`, không cam kết vào
nhánh). Người duyệt đã tự dựng lại và xem `v02-tray-none-360-dark`, `v02-tray-family-360-light`,
`v02-tray-family-1280-dark`, `v02-form-tray-360` (xem §3).

## 2. Đối chiếu đặc tả

| Mục | Kết quả |
|---|---|
| §16.2 chữ: nhãn mục, "Chưa có ảnh. Ngày giỗ chỉ hiện bát hương và nến.", "Chọn ảnh mâm lễ…", "Đang xử lý ảnh…", "Đổi ảnh", "Bỏ ảnh", hai câu lỗi, dòng gợi ý | Đúng từng chữ (so trực tiếp mã và kiểm thử `f0`, lỗi ảnh hỏng, > 25 MB) |
| §16.2 vị trí "ngay sau ô Lời tưởng niệm" | Đúng (`fLoi.el, traySec`); kiểm thử `afterLoi` |
| §16.2 vô hiệu hóa Lưu khi đang xử lý | Đúng (`saveBtn.disabled`, thêm chốt `if (tray.busy) return`) |
| §16.2 / 01 §3.7 xử lý ảnh: giải mã, vẽ lại canvas (bỏ EXIF), cạnh dài ≤ 1600, ≤ 1 MB, không phóng to | Đúng: `processPhoto(f, { maxEdge: 1600, maxStored: 1 MiB })`, `scale = min(1, …)`. Kiểm thử: đầu vào 2400×1800 có APP1 chứa vị trí giả → lưu 1600×1200, 42 413 byte, không APP1/APP13, không còn chuỗi vị trí |
| §16.2 "Bỏ / Đổi có hiệu lực khi Lưu"; hủy biểu mẫu không để lại ảnh | Đúng: ảnh mới chỉ nằm trong bộ nhớ (`tray.pending`), ghi cùng giao dịch `putPerson(p, newPhotos)`; lõi tự xóa ảnh cũ. Kiểm thử hủy → 0 ảnh mới; bỏ ảnh → khóa mất, ảnh bị xóa, 0 mồ côi |
| MA10-11 (không ghi gì trước Lưu) | Đạt theo thiết kế: không gọi `addPhoto` trước Lưu, nên Gộp ở thẻ khác không thể xóa ảnh đang chờ; có kiểm thử hai thẻ |
| Giữ `memorial.tray_photo_id` trên mọi đường lưu | Đạt. Chỉ có hai chỗ giao diện dựng `memorial`: `forms.ts` (giữ `tray.oldId` trừ khi bỏ / thay) và `notice.ts` (chuyển đang sống → đã mất, giữ nếu có — phòng xa). Các đường ghi khác ở lõi (`db.ts` nhập, chuyển đổi, `pruneAfterMigration`) không dựng lại `memorial`. Kiểm thử hồi quy "sửa ghi chú rồi Lưu → cùng id, cùng số byte" đạt |
| 01 §3.7 không ghi khóa khi `null` | Đúng (`...(trayId ? { tray_photo_id } : {})`, thêm `normNotice` ở lõi) |
| §16.1 thứ tự gia đình → Clevix cùng `tray_style` → không mâm | Đúng; ca (5) kiểm cả hai chiều |
| §16.1 không mâm: nến · bát hương · nến; Công giáo nến · hoa · nến, không hương | Đúng (kiểm thử `order`, ca (4)) |
| §16.1 ảnh dưới hàng bàn thờ, ≤ 480 px, 4:3 cắt giữa, bo 12 px, viền vàng đồng, nhãn trợ năng | Đúng (CSS `.mem-tray*`, `role="img"` + `aria-label` đúng hai chuỗi) |
| §16.1 dòng ghi công "Ảnh: <tác giả> · <giấy phép>" | Đúng định dạng (xem G-03) |
| §16.1 nút "Thêm ảnh mâm lễ của gia đình" dưới lời tưởng niệm, mất khi có ảnh gia đình; mở biểu mẫu và cuộn tới mục | Đúng; chế độ chỉ đọc không hiện (TKN3-P-01); tiêu điểm đặt vào "Chọn ảnh mâm lễ…" |
| §16.2 dòng hồ sơ "Ảnh mâm lễ: Đã có / Chưa có" | Đúng |
| §16.3 / R15 (b) gói: chỉ mục đã duyệt, đúng `sha256`, build thất bại khi gói lỗi | Đạt: gọi `scripts/check-tray-pack.py` (trạng thái, sha256, EXIF/XMP, ≤ 400 KB, ≤ 2 MB, chống `..`), lỗi → `fail()`; kiểm lại `sha256` trong `build.mjs`. Gói thử: chỉ `thu-a` vào `dist/trays/`, ảnh "cho" bị bỏ, có trong precache |
| §16.3 "không tải gì qua mạng"; CSP | CSP không đổi (`img-src 'self'`, `connect-src 'self'`). Ảnh gói đọc bằng `fetch` cùng gốc tới tệp đã precache, ảnh gia đình đọc từ IndexedDB; vẽ lên canvas, không `blob:`/`data:` URL. Smoke "URL ngoài danh mục: 0" |
| R15 (f) không lộ vào thông báo / .ics / URL / tên tệp / tiêu đề | Đạt ở phần PR thêm: URL mới `#/nguoi-than/sua/<person id>/mam-le` chỉ chứa id người, không id ảnh; không đổi tiêu đề trang, tên tệp; thông báo và .ics nằm ở lõi (MA10-07 đã kiểm) |

## 3. Bố cục 360 / 1280, sáng / tối

Đã xem ảnh dựng lại: không mâm (360 tối) — hàng bàn thờ cân, nút chữ vàng nhạt đọc rõ trên nền tối; ảnh gia đình (360
sáng, 1280 tối) — ảnh nằm giữa, dưới kệ, viền vàng đồng và bo góc đúng, không tràn ngang; mục biểu mẫu (360) — ảnh thu
nhỏ 120 × 90, hai nút cùng hàng, chữ gợi ý xuống dòng gọn. Không thấy lỗi bố cục.

## 4. Phát hiện

### Chặn

Không có.

### Nên sửa

**NS-01 — Hai cặp nút "Đổi ảnh" / "Bỏ ảnh" trùng tên với trình đọc màn hình.** Mục ảnh chân dung và mục ảnh mâm lễ
cùng có nút "Đổi ảnh", "Bỏ ảnh"; nhãn "Ảnh mâm lễ ngày giỗ" là một `<p>` không gắn với các nút. Người dùng trình đọc
màn hình nghe hai nút giống hệt nhau khi duyệt theo danh sách nút. Đề nghị: bọc mục trong `role="group"` có
`aria-labelledby` trỏ tới nhãn, hoặc thêm `aria-label` "Đổi ảnh mâm lễ" / "Bỏ ảnh mâm lễ" (chữ hiện trên màn hình
giữ nguyên theo §16.2).

**NS-02 — Mục ảnh mâm lễ nằm trong khung thu gọn có tiêu đề không nhắc tới nó.** `<summary>` vẫn là "Tưởng niệm (giỗ
hết, lời tưởng niệm)"; khi tạo hồ sơ mới khung đóng, người dùng không biết có chỗ thêm ảnh mâm lễ (chỉ lối vào từ màn
hình tưởng niệm tự mở khung). Đề nghị đổi thành "Tưởng niệm (giỗ hết, lời tưởng niệm, ảnh mâm lễ)" — nếu vai đặc tả
đồng ý vì §3.4 quy định chữ của tiêu đề này.

### Góp ý

**G-01 — Chiều rộng ảnh ở 360 px.** Kiểm thử đo 294 × 221 px ở 360 px: ảnh lấp đầy vùng trong của khung `.mem-scene`
(lề trang 16 px + đệm khung 16 px mỗi bên). §16.1 ghi "360 px: toàn chiều rộng trừ lề 16 px" (≈ 328 px). Hình nhìn
cân đối nên không đề nghị đổi mã; đề nghị vai đặc tả xác nhận cách hiểu "lề 16 px" là lề của khung tưởng niệm, hoặc
sửa câu chữ.

**G-02 — Ảnh gia đình bị mất byte thì không còn lối thêm lại.** `famTray` khác `null` nhưng `photoBytes` lỗi →
`figure` bị gỡ, nút "Thêm ảnh mâm lễ của gia đình" cũng không hiện. Lõi đã chặn ảnh treo (V3) nên ca này khó xảy ra;
có thể hiện lại nút trong nhánh `catch`.

**G-03 — Dòng ghi công bỏ qua nội dung `ghi_cong`.** `build.mjs` chỉ dùng `ghi_cong` để biết có cần ghi công không,
rồi dựng chuỗi từ `tac_gia` và `giay_phep`. Đúng định dạng §16.1, nhưng nếu vai nội dung muốn câu ghi công riêng (yêu
cầu của một số giấy phép) thì sẽ không hiện. Nên ghi rõ quy ước này trong `content/memorial/trays/README.md`.

**G-04 — Thiếu tệp manifest được coi là gói rỗng.** Nay `content/memorial/trays/manifest.json` đã có trong kho, nên
nhánh `!existsSync → []` chỉ còn che lỗi đường dẫn (ví dụ `LVHV_TRAY_MANIFEST` gõ sai sẽ dựng ra bản không mâm mà
không báo). Đề nghị: khi biến môi trường được đặt mà tệp không tồn tại thì `fail()`.

**G-05 — Thứ tự chạy cục bộ.** `npm run typecheck` trước `npm run build` báo thiếu `src/gen/content` và `test:smoke`
cần `tsport/dist` đã dựng. CI đã chạy đúng thứ tự; chỉ cần ghi một dòng trong `pwa/README` cho người chạy tay.

**G-06 — Ảnh chụp ca (5) chỉ có 360 sáng.** §16.4 chỉ đòi (1), (2), (4) ở bốn biến thể nên đủ; nêu để biết.

## 5. Kết luận

**ĐẠT — không có phát hiện chặn.** Chữ khớp §16.2, xử lý ảnh đúng 01 §3.7 và R15 (c) (có kiểm thử byte), `tray_photo_id`
được giữ ở mọi đường lưu của giao diện, không ghi gì vào kho trước khi bấm "Lưu" (đóng MA10-11 phía PWA), gói ảnh chỉ
nhúng mục đã duyệt và bản dựng thất bại khi gói lỗi, CSP không đổi, không tải qua mạng. Có thể hợp nhất; NS-01 và NS-02
nên sửa trước khi phát hành v0.2 (có thể làm trong PR sau), các góp ý tùy chủ dự án.
