# Gói ảnh mâm lễ của Clevix

Thư mục này chứa **ảnh mâm lễ do Clevix duyệt**, đi kèm bản cập nhật ứng dụng (YC-2026-10; quyết định ADR-V02-01 ở
docs/v0.2/07 §10; tiêu chí R15 ở docs/v0.2/04 Bổ sung B8; giao diện ở docs/v0.2/08 §16).

**Hiện tại gói rỗng.** Vì vậy màn hình tưởng niệm **chưa hiện mâm** ở bất kỳ đâu: dưới bia chỉ có bát hương ba nén và
hai nến (Công giáo: hai nến và hoa). Không có hình mâm vẽ thay thế.

Thứ tự hiển thị trên màn hình tưởng niệm của một người: ảnh mâm **gia đình tự chụp** cho người đó → ảnh **Clevix đã
duyệt** cùng kiểu mâm (`tray_style`) → **không có mâm**.

## 1. Thế nào là một ảnh được duyệt

Một ảnh chỉ được đưa vào gói khi đủ **cả bốn** điều sau:

1. **Ảnh chụp thật.** Không hình vẽ, không ảnh dựng 3D, không ảnh do máy hay trí tuệ nhân tạo tạo ra, không ghép thêm
   đồ vật không có thật. Được: cắt khung, chỉnh sáng, cân màu, thu nhỏ.
2. **Rõ quyền dùng.** Một trong hai:
   - ảnh **do Clevix hoặc cộng sự tự chụp**, có câu đồng ý bằng văn bản của người chụp (mẫu ở
     `../HUONG-DAN-CHUP-ANH-MAM.md` §8);
   - ảnh có **giấy phép thoáng đã kiểm trên trang gốc**: phạm vi công cộng / CC0, CC BY, CC BY-SA.
   **Không bao giờ** dùng ảnh trên mạng không rõ quyền (ảnh lấy từ báo, mạng xã hội, trang bán hàng, kết quả tìm kiếm).
3. **Ghi công đúng.** Ảnh CC BY hoặc CC BY-SA bắt buộc có dòng `ghi_cong` (tác giả, tên ảnh, giấy phép), hiện trong mục
   "Giới thiệu" của ứng dụng. Ảnh CC BY-SA: bản đã chỉnh sửa cũng theo CC BY-SA. Ảnh tự chụp hay CC0: `ghi_cong` có thể
   là `null`.
4. **Đạt chất lượng.** Chụp theo `../HUONG-DAN-CHUP-ANH-MAM.md` (ánh sáng cửa sổ một bên, không đèn flash, góc 3/4 hơi
   từ trên xuống, đủ nét, mâm đủ món theo lời chủ dự án: bát cơm có quả trứng luộc, khoanh giò, gà luộc, đĩa xôi, đĩa
   hoa quả, bát hương nhiều chân hương với ba nén đang cháy, lọ hoa ly hoặc cúc vàng). Không có người, tay, chữ, logo,
   nhãn giá. Ảnh trang nghiêm. **Chủ dự án xem và đồng ý** trước khi đổi `trang_thai` thành `duyet`.

## 2. Yêu cầu kỹ thuật của tệp

- JPEG hoặc WebP; **không còn siêu dữ liệu** (JPEG không có đoạn APP1/APP13, WebP không có khối EXIF/XMP — tức là
  không còn vị trí GPS, tên máy, ngày giờ); không có dữ liệu thừa sau cuối ảnh.
- Mỗi ảnh **≤ 400 KB**; cả gói **≤ 2 MB**.
- `width`, `height` trong manifest bằng kích thước thật của ảnh; `sha256` là mã băm của đúng tệp đó.

Cách bỏ siêu dữ liệu và tính `sha256` (máy có Python và Pillow; chạy ở gốc kho mã, thay tên tệp):

```
python3 -c "from PIL import Image; im=Image.open('anh-goc.jpg').convert('RGB'); im.thumbnail((1600,1600)); im.save('content/memorial/trays/truyen-thong-01.jpg', quality=85, optimize=True)"
python3 -c "import hashlib,sys; print(hashlib.sha256(open(sys.argv[1],'rb').read()).hexdigest())" content/memorial/trays/truyen-thong-01.jpg
```

(Lưu lại bằng Pillow như trên không chép EXIF sang ảnh mới.)

## 3. Thêm, tạm dừng hay bỏ một ảnh — không cần sửa mã

**Thêm:** chép tệp ảnh vào thư mục này (tên chữ thường không dấu, ví dụ `truyen-thong-01.jpg`), rồi thêm một mục vào
`items` của `manifest.json`:

```json
{
  "id": "truyen-thong-01",
  "tray_style": "truyen_thong",
  "file": "truyen-thong-01.jpg",
  "mime": "image/jpeg",
  "width": 1600,
  "height": 1067,
  "sha256": "<64 chữ hex>",
  "nguon": "Ảnh tự chụp tại Hà Nội, 10/11/2026",
  "giay_phep": "Clevix LLC — ảnh tự chụp, có giấy đồng ý của người chụp",
  "tac_gia": "<tên người chụp>",
  "ghi_cong": null,
  "duyet_boi": "<tên người duyệt>",
  "ngay_duyet": "2026-11-10",
  "trang_thai": "cho"
}
```

Để `trang_thai: "cho"` khi chờ chủ dự án xem; đổi thành `"duyet"` khi đã đồng ý. Chỉ mục `duyet` được đưa vào bản
dựng. `tray_style` là một trong `truyen_thong`, `phat_giao`, `cong_giao`. Lược đồ đầy đủ: `manifest.schema.json`.

**Tạm dừng:** đổi `trang_thai` về `"cho"` — ảnh không còn trong bản dựng kế tiếp.

**Bỏ hẳn:** xóa mục khỏi `items` và xóa tệp ảnh.

**Kiểm trước khi gửi PR** (chạy ở gốc kho mã, chỉ cần Python):

```
python3 scripts/check-tray-pack.py
python3 scripts/check-tray-pack.py --self-test
```

CI (job `tray-pack` trong `.github/workflows/ci.yml`) chạy cả hai lệnh và lưu danh sách ảnh đã duyệt ở `build/tray-pack.json`
(tạo phẩm `tray-pack`); các nền tảng chỉ nhúng tệp có trong danh sách đó, không bao giờ nhúng `test/`. Kịch bản in danh
sách ảnh đã duyệt. Bản dựng **thất bại** nếu một ảnh đã duyệt thiếu tệp, sai `sha256`, không phải
JPEG/WebP, còn siêu dữ liệu, có dữ liệu thừa, sai kích thước, hoặc quá giới hạn dung lượng. Ảnh mới chỉ đến tay người
dùng khi có bản cập nhật ứng dụng; ứng dụng không tải ảnh qua mạng.

Thư mục `test/` chứa ca kiểm của kịch bản (ảnh một màu rất nhỏ); **không** đưa vào bản dựng.
