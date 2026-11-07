# Nguồn của thư mục `content/memorial/`

Tệp này đáp ứng phần "Nội dung" của R8 (04): mọi tệp hình trong thư mục này có một dòng nguồn. Nguồn phong tục (chữ,
đồ cúng, cách gọi) ghi ở `PHONG-TUC.md` §6.

## 1. Tệp hình

| Tệp | Nguồn | Giấy phép |
|---|---|---|
| `mam-truyen-thong.svg` (bản tạm) | Vẽ mới cho dự án (Clevix Lab, 2026-11-07), viết tay bằng SVG; không lấy hình, nét hay mẫu từ nguồn ngoài | MIT, như mã nguồn của dự án (`LICENSE`) |
| `mam-chay.svg` (bản tạm) | Vẽ mới cho dự án (Clevix Lab, 2026-11-07), viết tay bằng SVG; không lấy hình, nét hay mẫu từ nguồn ngoài | MIT, như mã nguồn của dự án (`LICENSE`) |
| `mam-toi-gian.svg` (bản tạm) | Vẽ mới cho dự án (Clevix Lab, 2026-11-07), viết tay bằng SVG; không lấy hình, nét hay mẫu từ nguồn ngoài | MIT, như mã nguồn của dự án (`LICENSE`) |
| `photos/originals/*`, `photos/cutouts/*`, `photos/webp/*`, `photos/preview/*`, `photos/discover/*` | Ảnh chụp thật của người khác và bản cắt nền / thu nhỏ / ghép từ các ảnh đó — xem sổ §1b | Theo từng ảnh ở §1b; bản phái sinh có thành phần CC BY-SA: CC BY-SA 4.0 |

Số tệp **hình vẽ** lấy từ nguồn ngoài: **0**. Ba tệp trên là **bản tạm (placeholder)** — chỉ dùng cho tới khi có
ảnh thật được duyệt (YC-2026-09, docs/v0.2/09 §1, §3); vai giao diện có thể nhúng trực tiếp trong thời gian chờ. Hình
đang chạy trong PWA viết trong `pwa/src/people/memorial.ts`, cũng là hình vẽ mới cho dự án.

Phông chữ: các tệp SVG không chứa chữ. Không dùng phông, ảnh nhúng (`<image>`), liên kết ngoài hay mã chạy (`<script>`).

## 1b. Sổ giấy phép ảnh thật — mâm truyền thống (YC-2026-09)

Quy tắc (chỉ thị của điều phối viên 07/11/2026, theo ý chủ dự án):
- Chỉ **ảnh chụp thật**; không hình vẽ, không ảnh do máy hay trí tuệ nhân tạo tạo ra. Ảnh trên Commons nằm trong thể
  loại "AI-generated" bị loại tự động.
- Nguồn theo thứ tự ưu tiên: ảnh nhóm chủ dự án tự chụp (`HUONG-DAN-CHUP-ANH-MAM.md`); Pexels; Unsplash; Pixabay;
  Wikimedia Commons (CC0, phạm vi công cộng, CC BY, CC BY-SA).
- Ảnh nào không kiểm được giấy phép thì **không dùng**.

Cách kiểm giấy phép, ghi trung thực:
- **Unsplash, Pixabay:** đã mở trang ảnh bằng công cụ đọc trang ngày 2026-11-07 và chép câu giấy phép trên trang
  ("Free to use under the Unsplash License"; "Free for use under the Pixabay Content License").
- **Pexels:** trang ảnh **không đọc được** từ môi trường làm việc (trang trả lỗi) → không chọn ảnh Pexels nào, vì không
  kiểm được trên trang ảnh.
- **Wikimedia Commons:** công cụ đọc trang **không mở được** commons.wikimedia.org từ môi trường làm việc. Giấy phép,
  tác giả và ngày được đọc bằng **API của Commons** (`prop=imageinfo&iiprop=extmetadata`: `LicenseShortName`,
  `LicenseUrl`, `Artist`) trên máy chạy GitHub, trong `content/memorial/photos/tools/discover.py` và đọc lại mỗi lần
  tải trong `fetch_originals.py` (sai giấy phép → dừng). Đây là cùng dữ liệu trang Commons hiển thị, nhưng **chưa có
  người mở trang bằng mắt** — vai nội dung đề nghị chủ dự án hoặc điều phối viên bấm mở từng trang trong cột "Trang ảnh"
  trước khi phát hành.

Hệ quả giấy phép cho ảnh ghép:
- **CC BY / CC BY-SA:** phải **ghi công** (tác giả, tên ảnh, giấy phép, liên kết) ở nơi người dùng xem được — đề xuất
  trong mục "Giới thiệu" của ứng dụng, cùng chỗ ghi công ca dao.
- **CC BY-SA (chia sẻ tương tự):** ảnh cắt nền, các tệp WebP và **ảnh ghép** là tác phẩm phái sinh, nên phải phát hành
  theo **CC BY-SA** (các bản 2.0 và 3.0 cho phép dùng bản sau; chọn **CC BY-SA 4.0** cho toàn bộ ảnh mâm). Điều này
  **không** làm mã nguồn ứng dụng (MIT) phải đổi giấy phép: ảnh là tệp riêng đi kèm, nhưng thư mục ảnh mâm cần một tệp
  ghi giấy phép riêng. Nếu chủ dự án không muốn ràng buộc chia sẻ tương tự, thay các ảnh CC BY-SA bằng ảnh tự chụp.
- **Unsplash / Pixabay:** được dùng, sửa, không cần ghi công (vẫn nên ghi); không được bán lại ảnh nguyên bản hay dùng
  để dựng dịch vụ ảnh tương tự.

Bản gốc: ảnh được tải trên máy chạy GitHub (`.github/workflows/memorial-assets.yml`) vào
`content/memorial/photos/originals/`. Ảnh lớn được tải ở bản thu nhỏ của chính nhà cung cấp (cạnh dài ≤ 2400 px) để kho
mã nhẹ — URL thực tế đã tải ghi ở cột "Ảnh đã tải". Mã sha256 được ghi ở lần tải đầu và kiểm lại ở mọi lần sau. Dữ liệu
máy đọc được: `content/memorial/photos/ledger.json`.

| Mã | Món | Trang ảnh | Ảnh đã tải | Người chụp | Giấy phép — URL giấy phép | Ngày kiểm; cách kiểm | Ứng viên cắt nền (nền, góc, ánh sáng) | Trạng thái | Tệp | Nơi lưu bản gốc | sha256 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `ga-luoc-1` | Gà luộc | https://commons.wikimedia.org/wiki/File:Chicken_in_Hanoi.jpg | https://upload.wikimedia.org/wikipedia/commons/e/e2/Chicken_in_Hanoi.jpg?utm_source=commons.wikimedia.org&utm_campaign=imageinfo&utm_content=original | Rosa Cabecinhas, Alcino Cunha | CC BY-SA 2.0 — https://creativecommons.org/licenses/by-sa/2.0 | 2026-11-07; API Commons (extmetadata) trên máy chạy CI, 2026-11-07 | Tốt: gà luộc Việt Nam nguyên con, da vàng; nền tối lẫn đồ vật — cần cắt cẩn thận, có hai con chồng nhau | đang dùng trong bản ghép | `content/memorial/photos/originals/ga-luoc-1.jpg` | kho mã, nhánh `v0.2-content`; bản gốc đầy đủ ở trang nguồn | `2c14f25bcf03027221df89620d582af0068ff043455f446f0b7843468f76703b` |
| `ga-luoc-2` | Gà luộc | https://commons.wikimedia.org/wiki/File:BeiQieJi-WhiteCutChicken.jpg | https://upload.wikimedia.org/wikipedia/commons/a/ae/BeiQieJi-WhiteCutChicken.jpg?utm_source=commons.wikimedia.org&utm_campaign=imageinfo&utm_content=original | Sjschen | CC BY-SA 3.0 — https://creativecommons.org/licenses/by-sa/3.0 | 2026-11-07; API Commons (extmetadata) trên máy chạy CI, 2026-11-07 | Trung bình: gà đã chặt, nhìn từ trên; không phải nguyên con | dự phòng (chưa tải) | — | trang nguồn | `—` |
| `khoanh-gio-1` | Khoanh giò | https://commons.wikimedia.org/wiki/File:Gi%C3%B2_l%E1%BB%A5a.jpg | https://thumb.wikimedia.org/wikipedia/commons/thumb/9/95/Gi%C3%B2_l%E1%BB%A5a.jpg/3840px-Gi%C3%B2_l%E1%BB%A5a.jpg?utm_source=commons.wikimedia.org&utm_campaign=imageinfo&utm_content=thumbnail | Viethavvh | CC BY-SA 4.0 — https://creativecommons.org/licenses/by-sa/4.0 | 2026-11-07; API Commons (extmetadata) trên máy chạy CI, 2026-11-07 | Khá: miếng giò lụa trên đĩa trắng, nền gỗ trơn; nhìn từ trên xuống, cắt miếng tam giác (không phải khoanh tròn) | đang dùng trong bản ghép | `content/memorial/photos/originals/khoanh-gio-1.jpg` | kho mã, nhánh `v0.2-content`; bản gốc đầy đủ ở trang nguồn | `e428f26f3728642538e598c51847ce045a0e237dd5c09eace4da5513fc59fce2` |
| `khoanh-gio-2` | Khoanh giò | https://commons.wikimedia.org/wiki/File:Gi%C3%B2_l%E1%BB%A5a_2.jpg | https://upload.wikimedia.org/wikipedia/commons/a/a5/Gi%C3%B2_l%E1%BB%A5a_2.jpg?utm_source=commons.wikimedia.org&utm_campaign=imageinfo&utm_content=original | Viethavvh | CC BY-SA 4.0 — https://creativecommons.org/licenses/by-sa/4.0 | 2026-11-07; API Commons (extmetadata) trên máy chạy CI, 2026-11-07 | Khá: như trên, ảnh dọc | dự phòng (chưa tải) | — | trang nguồn | `—` |
| `dia-xoi-1` | Đĩa xôi | https://commons.wikimedia.org/wiki/File:X%C3%B4i_g%E1%BA%A5c_c%C3%BAng_giao_th%E1%BB%ABa_T%E1%BA%BFt_%C4%90inh_D%E1%BA%ADu_n%C4%83m_2017_t%E1%BA%A1i_%C4%90%C3%B4ng_H%C3%A0_(2).jpg | https://upload.wikimedia.org/wikipedia/commons/1/11/X%C3%B4i_g%E1%BA%A5c_c%C3%BAng_giao_th%E1%BB%ABa_T%E1%BA%BFt_%C4%90inh_D%E1%BA%ADu_n%C4%83m_2017_t%E1%BA%A1i_%C4%90%C3%B4ng_H%C3%A0_%282%29.jpg?utm_source=commons.wikimedia.org&utm_campaign=imageinfo&utm_content=thumbnail_unscaled | Phương Huy | Public domain —  | 2026-11-07; API Commons (extmetadata) trên máy chạy CI, 2026-11-07 | Tốt: đĩa xôi gấc cúng giao thừa trên đĩa trắng, nền tối; nhìn gần từ trên | đang dùng trong bản ghép | `content/memorial/photos/originals/dia-xoi-1.jpg` | kho mã, nhánh `v0.2-content`; bản gốc đầy đủ ở trang nguồn | `e46c1247dd3fa6ed08632b091ca373c3b4396f3ab147b0fb7e0894670edae17a` |
| `dia-xoi-2` | Đĩa xôi | https://commons.wikimedia.org/wiki/File:X%C3%B4i_g%E1%BA%A5c_c%C3%BAng_giao_th%E1%BB%ABa_T%E1%BA%BFt_%C4%90inh_D%E1%BA%ADu_n%C4%83m_2017_t%E1%BA%A1i_%C4%90%C3%B4ng_H%C3%A0_(1).jpg | https://upload.wikimedia.org/wikipedia/commons/f/f4/X%C3%B4i_g%E1%BA%A5c_c%C3%BAng_giao_th%E1%BB%ABa_T%E1%BA%BFt_%C4%90inh_D%E1%BA%ADu_n%C4%83m_2017_t%E1%BA%A1i_%C4%90%C3%B4ng_H%C3%A0_%281%29.jpg?utm_source=commons.wikimedia.org&utm_campaign=imageinfo&utm_content=original | Phương Huy | Public domain — https://creativecommons.org/publicdomain/mark/1.0/ | 2026-11-07; API Commons (extmetadata) trên máy chạy CI, 2026-11-07 | Tốt: cùng mâm, góc khác | dự phòng (chưa tải) | — | trang nguồn | `—` |
| `bat-com-1` | Bát cơm / trứng | https://commons.wikimedia.org/wiki/File:Steamed_rice_in_bowl_01.jpg | https://upload.wikimedia.org/wikipedia/commons/7/77/Steamed_rice_in_bowl_01.jpg?utm_source=commons.wikimedia.org&utm_campaign=imageinfo&utm_content=original | Anna Frodesiak | CC0 — http://creativecommons.org/publicdomain/zero/1.0/deed.en | 2026-11-07; API Commons (extmetadata) trên máy chạy CI, 2026-11-07 | Tốt: bát cơm trắng góc 3/4, nền tối trơn | đang dùng trong bản ghép | `content/memorial/photos/originals/bat-com-1.jpg` | kho mã, nhánh `v0.2-content`; bản gốc đầy đủ ở trang nguồn | `2288ddd76d09f67151dd029a3b03a7a24f46a20c574963bac123399eefb83ec9` |
| `bat-com-2` | Bát cơm / trứng | https://commons.wikimedia.org/wiki/File:A_bowl_of_rice.jpg | https://upload.wikimedia.org/wikipedia/commons/d/d6/A_bowl_of_rice.jpg?utm_source=commons.wikimedia.org&utm_campaign=imageinfo&utm_content=original | Douglas Perkins | CC0 — http://creativecommons.org/publicdomain/zero/1.0/deed.en | 2026-11-07; API Commons (extmetadata) trên máy chạy CI, 2026-11-07 | Khá: bát cơm trên nền trắng, nhìn gần từ trên | dự phòng (chưa tải) | — | trang nguồn | `—` |
| `trung-1` | Bát cơm / trứng | https://commons.wikimedia.org/wiki/File:CodazziBoiledEgg2.jpg | https://upload.wikimedia.org/wikipedia/commons/3/36/CodazziBoiledEgg2.jpg?utm_source=commons.wikimedia.org&utm_campaign=imageinfo&utm_content=original | 小太刀 | CC BY-SA 3.0 — http://creativecommons.org/licenses/by-sa/3.0/ | 2026-11-07; API Commons (extmetadata) trên máy chạy CI, 2026-11-07 | Không dùng: xem ảnh gốc thấy trứng còn vỏ (vỏ nứt), không phải trứng bóc vỏ. Ảnh đã tải ở lần chạy đầu, giữ lại cùng sha256 để đối chiếu | loại (trứng còn vỏ); đã tải ở lần đầu | `content/memorial/photos/originals/trung-1.jpg` | kho mã, nhánh `v0.2-content`; bản gốc đầy đủ ở trang nguồn | `b2297dc038824b74f6edcb0106d6227df7b4a41330c7111dcac61745582a8c84` |
| `trung-2` | Bát cơm / trứng | https://commons.wikimedia.org/wiki/File:Hard_boiled_eggs_for_thit_heo_kho,_peeled.jpg | https://upload.wikimedia.org/wikipedia/commons/d/df/Hard_boiled_eggs_for_thit_heo_kho%2C_peeled.jpg?utm_source=commons.wikimedia.org&utm_campaign=imageinfo&utm_content=original | HungryHuy | CC BY 2.0 — https://creativecommons.org/licenses/by/2.0 | 2026-11-07; API Commons (extmetadata) trên máy chạy CI, 2026-11-07 | Khá: nhiều trứng bóc vỏ trên thớt; phải cắt một quả | đang dùng trong bản ghép | `content/memorial/photos/originals/trung-2.jpg` | kho mã, nhánh `v0.2-content`; bản gốc đầy đủ ở trang nguồn | `fcde152672d180bd2acdcc8ac34f6b2f2783c0d0e2eb1d7a0957cdf4ffc1de79` |
| `hoa-qua-1` | Đĩa hoa quả | https://commons.wikimedia.org/wiki/File:M%C3%A2m_ng%C5%A9_qu%E1%BA%A3_(t%C3%A1o_%C4%91%E1%BB%8F_%C4%90%C3%A0_L%E1%BA%A1t),_T%E1%BA%BFt_2022_(3).jpg | https://upload.wikimedia.org/wikipedia/commons/d/d1/M%C3%A2m_ng%C5%A9_qu%E1%BA%A3_%28t%C3%A1o_%C4%91%E1%BB%8F_%C4%90%C3%A0_L%E1%BA%A1t%29%2C_T%E1%BA%BFt_2022_%283%29.jpg?utm_source=commons.wikimedia.org&utm_campaign=imageinfo&utm_content=thumbnail_unscaled | Phương Huy | CC BY-SA 4.0 — https://creativecommons.org/licenses/by-sa/4.0 | 2026-11-07; API Commons (extmetadata) trên máy chạy CI, 2026-11-07 | Khá: đĩa táo đỏ trên mâm đồng (mâm ngũ quả Tết); chỉ một loại quả; nền khăn hoa văn | đang dùng trong bản ghép | `content/memorial/photos/originals/hoa-qua-1.jpg` | kho mã, nhánh `v0.2-content`; bản gốc đầy đủ ở trang nguồn | `feb37017092253e64e4aa38c594b0e365c39f5adcce97448954b15808a2a503d` |
| `hoa-qua-2` | Đĩa hoa quả | https://commons.wikimedia.org/wiki/File:M%E1%BA%ABu_trang_tr%C3%AD_m%C3%A2m_hoa_qu%E1%BA%A3_ng%C3%A0y_T%E1%BA%BFt.jpg | https://upload.wikimedia.org/wikipedia/commons/d/d8/M%E1%BA%ABu_trang_tr%C3%AD_m%C3%A2m_hoa_qu%E1%BA%A3_ng%C3%A0y_T%E1%BA%BFt.jpg?utm_source=commons.wikimedia.org&utm_campaign=imageinfo&utm_content=original | Rachel41Ng | CC0 — http://creativecommons.org/publicdomain/zero/1.0/deed.en | 2026-11-07; API Commons (extmetadata) trên máy chạy CI, 2026-11-07 | Kém: mâm hoa quả nhiều loại nhưng nền rất rối | dự phòng (chưa tải) | — | trang nguồn | `—` |
| `nen-1` | Nến | https://commons.wikimedia.org/wiki/File:Candle_1_(Unsplash).jpg | https://thumb.wikimedia.org/wikipedia/commons/thumb/2/2d/Candle_1_%28Unsplash%29.jpg/3840px-Candle_1_%28Unsplash%29.jpg?utm_source=commons.wikimedia.org&utm_campaign=imageinfo&utm_content=thumbnail | David Monje davidmonje | CC0 — http://creativecommons.org/publicdomain/zero/1.0/deed.en | 2026-11-07; API Commons (extmetadata) trên máy chạy CI, 2026-11-07 | Tốt: nến đỏ đang cháy trên nền đen trơn; không có chân nến | đang dùng trong bản ghép | `content/memorial/photos/originals/nen-1.jpg` | kho mã, nhánh `v0.2-content`; bản gốc đầy đủ ở trang nguồn | `cd5b9d3b8d4f083f000c1417aeeff9b97629fde1892ca5722fbc4d1f9e26c40b` |
| `nen-2` | Nến | https://unsplash.com/photos/three-red-candles-are-lit-in-the-dark-y5Us50JgIBk | https://images.unsplash.com/photo-1632770385660-5c51b3adbf60?w=2400&q=90&fm=jpg | Gülfer ERGİN | Unsplash License — https://unsplash.com/license | 2026-11-07; mở trang ảnh bằng công cụ đọc trang, 2026-11-07 | Tốt: ba nến đỏ đang cháy trên nền đen; cắt một cây | đang dùng trong bản ghép | `content/memorial/photos/originals/nen-2.jpg` | kho mã, nhánh `v0.2-content`; bản gốc đầy đủ ở trang nguồn | `796dbdd86be3f49a4faf94e0e2973ba1ccd923b81be4e20b8870b5ed8fc792a9` |
| `lo-hoa-1` | Lọ hoa | https://unsplash.com/photos/white-lilies-bloom-in-a-glass-vase-lunbqFx8SkQ | https://images.unsplash.com/photo-1749221479907-c77891a3404c?w=2400&q=90&fm=jpg | Jerry Wang | Unsplash License — https://unsplash.com/license | 2026-11-07; mở trang ảnh bằng công cụ đọc trang, 2026-11-07 | Trung bình: lọ hoa ly trắng, nền bếp mờ; lọ thủy tinh khó cắt nền | đang dùng trong bản ghép | `content/memorial/photos/originals/lo-hoa-1.jpg` | kho mã, nhánh `v0.2-content`; bản gốc đầy đủ ở trang nguồn | `727c33fe80a1e97aa33d381a50891373621d4cc58e7f8d2df41770f49a87db8f` |
| `lo-hoa-2` | Lọ hoa | https://unsplash.com/photos/a-bouquet-of-white-lilies-in-a-vase-HKaX84_IMmA | https://images.unsplash.com/photo-1749221479901-8c901e5150aa | Jerry Wang | Unsplash License — https://unsplash.com/license | 2026-11-07; mở trang ảnh bằng công cụ đọc trang, 2026-11-07 | Trung bình: như trên, góc khác | dự phòng (chưa tải) | — | trang nguồn | `—` |
| `bat-huong-1` | Bát hương | https://pixabay.com/photos/incense-pots-smoke-oriental-1285138/ | https://cdn.pixabay.com/photo/2016/03/28/09/32/incense-1285138_1280.jpg | Pexels (người đăng trên Pixabay) | Pixabay Content License — https://pixabay.com/service/license-summary/ | 2026-11-07; mở trang ảnh bằng công cụ đọc trang, 2026-11-07 | Loại sau khi thử: lư đồng ở đền có chữ Hán; tách nền làm mất chân hương. Giữ tệp đã tải để đối chiếu sha256 | đã tải, đã thử, **loại** (lư đồng ở đền, có chữ Hán, cắt nền hỏng) | `content/memorial/photos/originals/bat-huong-1.jpg` | kho mã, nhánh `v0.2-content`; bản gốc đầy đủ ở trang nguồn | `d21b84c28a25d87c86811bf924b97bcb57794f117636c12ad983a73f8aec2059` |
| `bat-huong-2` | Bát hương | https://commons.wikimedia.org/wiki/File:Quang_Minh_Ni_T%E1%BB%B1_(Ban_th%E1%BB%9D_Ph%E1%BA%ADt_ngo%C3%A0i_tr%E1%BB%9Di,_b%C3%A1t_h%C6%B0%C6%A1ng).jpg | https://upload.wikimedia.org/wikipedia/commons/d/de/Quang_Minh_Ni_T%E1%BB%B1_%28Ban_th%E1%BB%9D_Ph%E1%BA%ADt_ngo%C3%A0i_tr%E1%BB%9Di%2C_b%C3%A1t_h%C6%B0%C6%A1ng%29.jpg?utm_source=commons.wikimedia.org&utm_campaign=imageinfo&utm_content=thumbnail_unscaled | Phương Huy | CC BY-SA 4.0 — https://creativecommons.org/licenses/by-sa/4.0 | 2026-11-07; API Commons (extmetadata) trên máy chạy CI, 2026-11-07 | Trung bình: bát hương sứ trắng Việt Nam, rất nhiều chân hương, nền rất rối nhưng tách nền được; không có ba nén đang cháy, không thấy tàn cong | đang dùng trong bản ghép | `content/memorial/photos/originals/bat-huong-2.jpg` | kho mã, nhánh `v0.2-content`; bản gốc đầy đủ ở trang nguồn | `8ffb41155a5b0e167a39c918979c15f38fed6387df9741e74f5544d895b17e03` |

**Món không có ảnh mạng dùng được — CẦN CHỤP** (xem `HUONG-DAN-CHUP-ANH-MAM.md` §1):
- **Bát hương** đúng ý chủ dự án (rất nhiều chân hương đã cháy hết, tàn cong, **ba nén đang cháy**): không tìm thấy.
  Ảnh `bat-huong-2` là bát hương sứ thật ở chùa, nhiều chân hương nhưng **không** có ba nén đang cháy và tàn cong; ảnh
  `bat-huong-1` là lư đồng ở đền có chữ Hán — đã thử và **loại** khỏi bản ghép.
- **Lọ cúc vàng:** kết quả Commons chỉ có tranh vẽ (Van Gogh, đồ gốm vẽ) — là hình vẽ nên **loại**. Đang dùng hoa ly
  (`lo-hoa-1`); lọ thủy tinh bị cắt mất khi tách nền nên cuống hoa được giấu sau đĩa quả.
- **Chén nước**, **mâm tròn**, **mặt bàn gỗ**: chưa có.
- **Khoanh giò** tròn: ảnh đang dùng là miếng giò cắt tam giác; **đĩa xôi** mất đĩa khi tách nền.

**Ảnh xem trước:** `content/memorial/photos/preview/mam-anh-that-preview.png` (1080 × 810, khung 360 × 270 ở 3×), dựng
trên máy chạy GitHub. Tổng dung lượng WebP 1×/2×/3× của các lớp đang dùng: **183 568 byte** (≤ 600 KB) —
`content/memorial/photos/webp/budget.json`. Đây là **bản nháp ghép**, chưa duyệt: ánh sáng và góc máy của các ảnh
khác nhau rõ (ảnh chụp ở nhiều nơi), nên ảnh nhóm chủ dự án tự chụp theo hướng dẫn vẫn là cách tốt nhất.

## 2. Nguồn phong tục đã dùng cho nội dung hình

Chi tiết từng câu trích ở `PHONG-TUC.md` §1–§3. Mọi nguồn đã đọc ngày 2026-11-07.

| Mã | Nguồn | Dùng cho |
|---|---|---|
| N1 | Hồng Phượng, "Sơ đồ cách xưng hô trong họ hàng và thứ bậc, vai vế trong gia đình Việt", VOH, 31/01/2025. https://voh.com.vn/thuong-thuc/so-do-cach-xung-ho-trong-ho-hang-575769.html | Cách gọi họ hàng, Cụ/Cố, Kỵ/Sơ |
| N2 | "Bài trí và ý nghĩa của bàn thờ tổ tiên ngày Tết", VOV, 16/01/2009. https://vov.vn/huong-vi-tet/bai-tri-va-y-nghia-cua-ban-tho-to-tien-ngay-tet | Bát hương, bát nước trong, hai đèn, đĩa quả, độc bình |
| N3 | Phương Nga, "Thắp hương ngày Tết thế nào cho đúng?", VOH, 09/02/2024. https://voh.com.vn/thuong-thuc/thap-huong-ngay-tet-514764.html | Ba nén hương |
| N4 | Thích Thiện Phước, "Ý nghĩa ngày lễ húy kỵ - ngày đám giỗ", phatgiao.org.vn, 16/09/2019. https://phatgiao.org.vn/y-nghia-ngay-le-huy-ky--ngay-dam-gio-d36866.html | Giỗ đầu, giỗ hết, giỗ thường, tiên thường; nhang đèn bông trái |
| N5 | Toan Ánh, "Ngày giỗ họ của người Việt có ý nghĩa gì?" (trích *Nếp cũ*, NXB Trẻ), Znews, 15/06/2026. https://znews.vn/ngay-gio-ho-cua-nguoi-viet-co-y-nghia-gi-post1659626.html | Ông Tổ họ, giỗ họ |
| N6 | Nguyen D.L., Nguyen K.N., Phan Q.A. (2022), "The sanctity of goddesses: an insight into the worship of Bà Tổ Cô in Northern Vietnam", *Humanit. Soc. Sci. Commun.* 9, 2. doi:10.1057/s41599-021-01018-y | Bà Cô Tổ họ |
| N7 | Thanh Hòa, "Cha mẹ cầu siêu cho những đứa con không kịp chào đời", VnExpress, 07/09/2015. https://vnexpress.net/cha-me-cau-sieu-cho-nhung-dua-con-khong-kip-chao-doi-3275310.html | Bé đỏ |
| N8 | Thích Tâm Hiệp, "Cúng mặn hay cúng chay?", Báo Pháp luật Việt Nam, 07/08/2021. https://baophapluat.vn/cung-man-hay-cung-chay-post406472.html | Mâm chay: hương, hoa, đăng, trà, quả, thực |
| N9 | "Thờ cúng tổ tiên ở Việt Nam: bàn thờ, nghi lễ và những điều du khách nên biết", vietnamtourism.com, cập nhật 07/08/2026. https://www.vietnamtourism.com/vi/tho-cung-to-tien-o-viet-nam-ban-tho-nghi-le-va-nhung-ieu-du- | Giỗ theo âm lịch; bố cục bàn thờ; mâm cỗ mặn; hoa cúc vạn thọ, hoa huệ |
| N10 | Phêrô Nguyễn Văn Vịnh, "Ăn giỗ", tgpsaigon.net, 14/09/2012. https://tgpsaigon.net/bai-viet/an-gio-36067 | Công giáo: cầu nguyện, không cúng |

## 3. Nguồn đã đọc nhưng không dùng

| Nguồn | Vì sao không dùng |
|---|---|
| Nguyễn Phong Vũ, "Nghi thức cúng ông bà tổ tiên của tín đồ theo đạo Tứ Ân Hiếu Nghĩa ở huyện Tri Tôn, tỉnh An Giang", *Nghiên cứu Tôn giáo* số 12 (2015), vjol.info.vn | Bàn về một đạo riêng; không có giỗ đầu, giỗ hết, tiên thường |
| Lê Công Lý, "Anh hùng dân tộc Trương Định: từ ngày mất đến ngày giỗ âm lịch", *Tạp chí Nghiên cứu và Phát triển* số 3 (2025), vjol.info.vn | Chỉ đọc được tóm tắt; tóm tắt không nói tháng nhuận hay ngày 30 |
| Bài "Death anniversary", Wikipedia tiếng Anh | Nguồn thứ cấp; không có giỗ đầu, giỗ hết, tiên thường |
| Các trang bán đồ thờ, trang văn khấn thương mại | Không đủ tin cậy; không trích |
