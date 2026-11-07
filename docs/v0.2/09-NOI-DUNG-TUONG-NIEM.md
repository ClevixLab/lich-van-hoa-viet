# Nội dung tưởng niệm — bản thiết kế ba kiểu mâm lễ

Trạng thái: **BẢN THẢO của vai nội dung (B8), 2026-11-07 — chưa khóa, chờ duyệt thiết kế (docs/08 §6).** Nội dung
**mâm truyền thống đã được chủ dự án quyết** (YC-2026-09, 07/11/2026 11:24): ghép từ ảnh thật (§3).
Tài liệu này là đầu vào cho vai giao diện các nền tảng; vai nội dung không sửa mã ứng dụng.

Căn cứ:
- 05 §1: "Màn hình tưởng niệm ngày giỗ (bia, ảnh, mâm theo 3 phong cách, ba nén hương có khói nhẹ, lời tưởng niệm của
  người dùng); trang nghiêm, không quảng cáo, không âm thanh; hình vẽ mới cho dự án."
- 08 §3.9 (màn hình đang chạy), §6 (trợ năng), §11 (chọn kiểu mâm chưa có trên giao diện).
- 04 R8 (khói, CPU, giảm chuyển động, không âm thanh, Công giáo không hương), R9 (biên bản nguồn phong tục).
- 07 YC-2026-09: mâm truyền thống bằng ảnh thật; 01 §3.7 thêm `toi_gian`.
- Nguồn phong tục: `content/memorial/PHONG-TUC.md` (mã N1–N10). Bản tạm hình vẽ: `content/memorial/mam-*.svg`.
  Hướng dẫn chụp: `content/memorial/HUONG-DAN-CHUP-ANH-MAM.md`. Sổ giấy phép ảnh: `content/memorial/SOURCES.md` §1b.

## 1. Ba kiểu

| Kiểu | Hình | Giá trị `tray_style` (01 §3.7) |
|---|---|---|
| 1. Truyền thống — mâm cơm cúng | **Ảnh thật ghép** (§3); bản tạm `mam-truyen-thong.svg` | `truyen_thong` (mặc định) |
| 2. Chay (thiên về Phật giáo) | Bản tạm hình vẽ `mam-chay.svg` | `phat_giao` |
| 3. Tối giản hiện đại | Bản tạm hình vẽ `mam-toi-gian.svg` | `toi_gian` (Bổ sung YC-2026-09) |

**Q-1 — đã quyết (07/11/2026):** thêm `toi_gian` vào 01 §3.7 (phương án A). Kiểu tối giản **không** thay cho
`cong_giao`: Công giáo là lựa chọn tôn giáo, không phải phong cách; hồ sơ `cong_giao` tiếp tục được vẽ như 08 §3.9
(hai nến và hoa, không bát hương, không khói; R8 (f)).

**Q-2 (mở, cho chủ dự án):** kiểu 2 và kiểu 3 cũng dùng ảnh thật như kiểu 1, hay giữ hình vẽ? Mặc định: giữ hình vẽ
làm bản tạm, không phát hành hai kiểu này cho tới khi chủ dự án trả lời. Nếu dùng ảnh thật, áp dụng nguyên §3.3–§3.5
và hướng dẫn chụp.

## 2. Quy tắc chung cho cả ba kiểu

**Bố cục.** Mâm nằm dưới bia, trên lời tưởng niệm (08 §3.9). Khung vẽ 360 × 200 đơn vị (tỉ lệ 9 : 5) cho kiểu 2, 3;
khung ảnh ghép 360 × 270 cho kiểu 1 (§3.2). Ở 360 px
khung rộng bằng bia (80 % màn hình); ở 1280 px rộng 360 px, căn giữa. Bát hương luôn ở **chính giữa** [N9] "đặt lư
hương bằng đồng (bát hương) ở chính giữa"; hai nến ở hai đầu [N9] "Hai chân nến ở hai bên". Đồ vật xếp một hàng trên
mặt bàn, không chồng lên nhau trừ chén nước đặt trước bát hương [N2].

**Nền.** Cùng nền tối ấm của màn hình tưởng niệm ở cả giao diện sáng và tối (08 §3.9): dải màu
`#5a2a1e → #3a1a14 → #1d0f0c`. Gỗ bàn `#8a4b22 → #5a2c12 → #3d1d0b`, mặt bàn `#6b3a1c`. Đồng `#8a5e0c → #e8c67a`
(biến `--brass`), viền khung ảnh `#b98b3e`.

**Chuyển động (R8).** Chỉ có khói chuyển động, bằng CSS `@keyframes`, không `requestAnimationFrame`. Ngọn nến **đứng
yên** (không chập chờn) để giữ CPU gần 0 và tránh gây khó chịu. Khói tự dừng sau 2 phút, dừng khi trang ẩn.
`prefers-reduced-motion: reduce` → khói tĩnh như trong tệp mẫu (ba nét cong mờ), không hiệu ứng nào. Kiểu không có
hương (`incense = false`, `cong_giao`) thì không có khói.

**Hương (`incense`).** Ba nén hương [N3] "Thông thường thắp 1 hoặc 3 nén hương là thắp cho gia tiên". Khi
`incense = false`: bỏ nén hương và khói. Kiểu 1, 2 giữ bát hương; kiểu 3 bỏ cả bát hương và dời nến vào giữa.

**Trợ năng.**
- Toàn bộ mâm là **một** hình có nhãn (`role="img"` + `aria-label` hoặc `<title>`/`<desc>`), các phần bên trong ẩn với
  trình đọc màn hình. Nhãn theo §3–§5 ("Mô tả cho trình đọc màn hình"). Nhãn **không** có tên người, không có thông tin
  báo tử.
- Tương phản đồ họa (WCAG 1.4.11, ≥ 3:1 với nền `#3a1a14` ở giữa khung; áp cho hình vẽ — với ảnh thật, kiểm bằng mắt
  trên ảnh chụp màn hình rằng mép đồ vật tách khỏi nền tối, thêm viền sáng mờ 1 điểm nếu cần): các hình nhận biết được nhờ phần sáng (ngọn
  lửa, đồng, hoa, đồ sứ trắng). Bảng đo ở §6. Ba chỗ dưới 3:1 cần sửa khi cài: thân nến đỏ, chân hương, bình xanh của
  kiểu 1 (§6).
- Không truyền thông tin chỉ bằng màu; mâm là trang trí, mọi thông tin (ngày giỗ, tuổi) đã có bằng chữ trên bia.
- Cỡ chữ A/A+/A++ không đổi mâm (mâm không chứa chữ); mâm co theo bề rộng, không tràn ngang.

**Những điều không bao giờ hiện trên màn hình tưởng niệm (mọi kiểu).**
- Quảng cáo, liên kết mua hàng, nút chia sẻ mạng xã hội, âm thanh, video, rung (05 §1, R8 (e)).
- Thông tin báo tử: người báo tin, nơi mất, lý do, ảnh giấy báo tử, ghi chú báo tử (R-PRIV-2, 08 §12g).
- Tên người trong tiêu đề tab, URL, tên tệp (08 §3.9, S4').
- Món mặn ở dạng **hình vẽ** (dễ thô ở cỡ nhỏ). Món mặn chỉ xuất hiện ở kiểu 1, bằng **ảnh thật** (§3, YC-2026-09).
- Ảnh giả: ảnh do máy hay trí tuệ nhân tạo tạo ra, ảnh dựng 3D; ảnh không kiểm được giấy phép.
- Vàng mã, tiền giấy, bùa chú; hình đầu lâu, ma quỷ, máu; mặt cười hay mắt mũi trên đồ vật (kể cả vô tình — xem §7).
- Biểu tượng tôn giáo người dùng không chọn: tượng Phật, chữ Vạn, thánh giá, tràng hạt. Kiểu 2 chỉ "thiên về" Phật
  giáo qua đồ cúng chay và hoa sen, không vẽ tượng hay chữ.
- Hiệu ứng lửa cháy lớn, pháo, hiệu ứng lấp lánh, chữ chạy.
- Chữ Hán-Việt cổ ("Hiển khảo"…) (quyết định của chủ dự án, PHONG-TUC §1).

## 3. Kiểu 1 — Truyền thống: mâm cơm cúng ghép từ ảnh thật (YC-2026-09, chủ dự án quyết)

**Lời chủ dự án (07/11/2026, 11:24, nguyên văn):** "truyền thống của người việt nam mâm cơm cúng sẽ có bát cơm quả
chứng luộc, một khoanh giò, một con gà luộc, một đĩa xôi, một đĩa hoa quả tổng hợp, và thắp 3 nén nhang trên một bát
hương đã có rất nhiều nén hương đã cháy hết và đậu tàn cong đẹp. Lọ hoa thì có thể là hoa ly hoặc hoa cúc vàng. Nên
là hình ảnh các thứ đều thật, chỉ là phối cảnh thành một mâm cơm cúng chứ đừng lấy ảnh giả"

Hệ quả:
- Mâm truyền thống là **ảnh ghép (phối cảnh) từ ảnh chụp thật** của từng món. **Không** hình vẽ, **không** ảnh tạo
  bằng máy: ảnh do trí tuệ nhân tạo sinh ra, ảnh dựng 3D, ảnh "tổng hợp" đều là **ảnh giả** và bị cấm. Không dùng thao
  tác "lấp đầy / vẽ thêm bằng máy" khi xử lý ảnh. Được phép: cắt nền, chỉnh sáng, cân màu, thu phóng, xoay nhẹ, ghép
  lớp, giữ bóng thật tách từ ảnh.
- Nguồn ảnh: ưu tiên ảnh do chủ dự án và cộng sự tự chụp theo `content/memorial/HUONG-DAN-CHUP-ANH-MAM.md`. Chấp nhận
  ảnh thật có giấy phép thoáng (phạm vi công cộng / CC0, hoặc CC BY có ghi công) **đã kiểm trên trang nguồn**. Ảnh nào
  không kiểm được giấy phép thì không dùng. Mỗi ảnh một dòng trong sổ giấy phép (`content/memorial/SOURCES.md` §1b).
- Cho tới khi có ảnh thật được duyệt, ứng dụng dùng hình vẽ `mam-truyen-thong.svg` làm **bản tạm** (đã ghi rõ trong tệp).

### 3.1 Đồ vật

| Mã tệp | Đồ vật | Ghi chú | Nguồn |
|---|---|---|---|
| `bat-com-trung` | Bát cơm trắng, trên có một quả trứng luộc bóc vỏ | Bát sứ trắng hoặc men lam, không chữ, không hoa văn hãng | Chủ dự án (YC-2026-09) |
| `khoanh-gio` | Một khoanh giò lụa (cắt dày, thấy rõ mặt cắt), đặt trên đĩa nhỏ hoặc trên lá chuối | Còn viền lá chuối quanh khoanh giò thì đẹp hơn | Chủ dự án; [N9] "các món thịt heo là phổ biến" |
| `ga-luoc` | Một con gà luộc nguyên con, da vàng, đặt trên đĩa | Gà đặt ngửa đầu hướng lên (cách bày thường gặp); "gà ngậm hoa" **chưa kiểm chứng** — chỉ làm nếu chủ dự án muốn | Chủ dự án; [N9] "Xôi, gà luộc …" |
| `dia-xoi` | Một đĩa xôi (xôi trắng, xôi gấc hoặc xôi đỗ), nặn tròn hoặc úp bát | | Chủ dự án; [N9] |
| `dia-hoa-qua` | Đĩa (hoặc mâm bồng) hoa quả tổng hợp: chuối, bưởi, cam, táo, nho… | Quả tươi, không nhãn dán | Chủ dự án; [N9] "Trái cây tươi là lễ vật dâng hằng ngày phổ biến nhất." |
| `bat-huong` | Bát hương **có rất nhiều chân hương đã cháy hết**, tàn hương cong đẹp, **ba nén đang cháy** | Bát sứ hoặc đồng; đốm lửa đầu hương thấy rõ | Chủ dự án; [N3] ba nén |
| `bat-huong-khong-thap` | Cùng bát hương, chưa thắp ba nén mới | Dùng khi `incense = false` | — |
| `lo-hoa` | Lọ hoa ly **hoặc** cúc vàng (chủ dự án chọn một khi duyệt ảnh) | Hoa tươi, không héo | Chủ dự án; [N9] "Hoa tươi, thường là cúc vạn thọ hoặc hoa huệ" |
| `nen-trai`, `nen-phai` | Hai cây nến đỏ trên chân nến đồng, đang cháy | Hai ảnh riêng (không lật gương một ảnh — sai hướng sáng) | [N9] "Hai chân nến ở hai bên."; [N2] |
| `chen-nuoc` | Chén nước trong | Đặt trên bàn, bên phải phía trước bát hương | [N2] "đặt phía trước bát hương là một bát nước trong" |
| `mam` | Mâm tròn (gỗ sơn, đồng hoặc nhôm) đựng các món mặn | Mâm trống, chụp riêng | — |
| `mat-ban` | Mặt bàn thờ gỗ (một dải ảnh gỗ thật) | Không bắt buộc; nếu không có thì để nền tối của màn hình | — |

Nền phía sau là nền màu tối ấm của màn hình tưởng niệm (CSS, 08 §3.9) — nền màu của giao diện, không phải hình vẽ đồ
vật.

### 3.2 Bố cục (phối cảnh)

Khung ghép: **360 × 270 điểm CSS** (tỉ lệ 4 : 3); bản 3× là 1080 × 810 điểm ảnh. Khói được phép tràn lên trên khung
tới 40 điểm (vùng trong suốt). Ở 360 px khung rộng bằng bia; ở 1280 px rộng 360 px, căn giữa (như 08 §7).

```
 y=0  ┌──────────────────────────────────────────────┐   (khói tràn lên trên)
      │                 ~ ~ ~                         │
      │                 │ │ │       ✿✿ lọ hoa         │
      │   🕯            │ │ │       ✿✿                │   🕯
      │  nến     đĩa    ╲▁▁╱      chén   ║            │  nến
 y=165│  trái   hoa quả bát hương  nước  lọ          │  phải   ← hàng sau, trên mặt bàn
      │      ╭──────────── mâm ─────────────╮         │
      │      │ đĩa xôi    gà luộc    bát cơm │         │
      │      │        khoanh giò     + trứng │         │
 y=235│      ╰──────────────────────────────╯         │   ← hàng trước, trên mâm
 y=270└──────────────────────────────────────────────┘
```

| Lớp (sau → trước) | Đồ vật | Tâm x | Chân (đáy) y | Rộng xấp xỉ |
|---|---|---|---|---|
| 0 | `mat-ban` | 180 | 175 (mép trước) | 360 |
| 1 | `nen-trai`, `nen-phai` | 28, 332 | 165 | 22 |
| 2 | `dia-hoa-qua` | 92 | 165 | 80 |
| 3 | `bat-huong` | 180 | 165 | 62 |
| 4 | `chen-nuoc` | 222 | 168 | 20 |
| 5 | `lo-hoa` | 272 | 165 | 70 |
| 6 | `mam` | 180 | 250 | 230 |
| 7 | `ga-luoc` | 180 | 222 | 110 |
| 8 | `dia-xoi` | 112 | 232 | 72 |
| 9 | `bat-com-trung` | 248 | 232 | 48 |
| 10 | `khoanh-gio` | 160 | 246 | 44 |

Bát hương ở **chính giữa** hàng sau [N9]; mâm ở chính giữa phía trước. Con số là điểm xuất phát; vai ghép được xê
dịch ± 10 điểm để không che nhau quá nhiều, nhưng **bát hương và ba nén hương không bị món nào che**.

**Tỉ lệ thật.** Lấy chung một thước: hàng trước khoảng **4,4 điểm CSS / cm**, hàng sau nhỏ hơn khoảng 15 % (xa máy
hơn, ≈ 3,8 điểm / cm). Kích thước thật để đối chiếu (đo lúc chụp, ghi vào sổ): mâm Ø 50 cm; gà dài 25–30 cm; đĩa xôi
Ø 16–20 cm; bát cơm Ø 11 cm; khoanh giò Ø 9–10 cm; bát hương Ø 15–18 cm; nến cao 20–25 cm; lọ hoa cả hoa 45–55 cm. Mỗi
món có một ảnh kèm thước dây (HUONG-DAN §4) để ghép đúng tỉ lệ.

### 3.3 Góc máy, ánh sáng, màu, bóng — phải **giống nhau ở mọi ảnh**

- **Góc máy:** chụp từ phía trước, lệch sang trái khoảng 15–20° (góc 3/4 phía trước), **hơi từ trên xuống 25–35°**.
  Mọi món cùng một góc, nếu không miệng bát, mặt mâm sẽ có độ dẹt khác nhau và ảnh ghép trông giả.
- **Ống kính:** ống chính 1× hoặc tele 2–3×, **không** dùng ống góc rộng 0,5× (méo hình). Khoảng cách 1–1,5 m.
- **Ánh sáng:** ánh sáng cửa sổ (trời râm hoặc qua rèm mỏng) **từ bên trái**, chếch khoảng 45° từ trên xuống; một tấm
  bìa trắng bên phải để đỡ bóng. **Không đèn flash**, không bật đèn trần màu vàng lẫn với ánh sáng cửa sổ.
- **Nhiệt độ màu:** ánh sáng ban ngày 5000–5600 K, khóa cân bằng trắng (hoặc chụp kèm thẻ xám/tờ giấy trắng để chỉnh
  về cùng một màu). Khi ghép, đưa mọi lớp về cùng một màu rồi phủ chung một lớp ấm nhẹ (khoảng +300 K) cho hợp nền tối
  ấm; không chỉnh từng món một kiểu.
- **Bóng:** bóng đổ về phía **phải – sau** (ngược hướng sáng). Ưu tiên **bóng thật** tách từ ảnh chụp trên nền trơn
  (một lớp bóng riêng, chế độ hòa trộn nhân). Nếu không tách được, dùng một bóng tiếp xúc mờ (hình elip mờ, độ đậm
  ≤ 30 %) ngay dưới đáy đồ vật — đây là kỹ thuật ghép, không vẽ thêm đồ vật.
- **Độ nét:** mọi món nét toàn bộ (khép khẩu hoặc không dùng chế độ xóa phông / chân dung, vì chế độ này làm mờ mép
  sai và khó cắt nền).

### 3.4 Khói hương trên ảnh thật

- Ảnh `bat-huong` chụp lúc khói **mỏng** (hoặc chọn khung hình ít khói) để khói động không chồng lên khói thật.
- Tệp kê khai (§3.6) ghi tọa độ **ba đầu hương** (x, y trên ảnh gốc). Ứng dụng đặt ba vệt khói CSS nhỏ đúng ở ba đầu
  hương: rộng 6–10 điểm, cao 40–60 điểm, màu `#e9e4dc`, độ đậm 0,10–0,18, mờ biên; mỗi vệt 6–9 giây, lệch pha nhau.
  Không dùng `requestAnimationFrame` (R8). Đốm lửa đầu hương là ảnh thật, **không** nhấp nháy.
- Khói tự dừng sau 2 phút, dừng khi trang ẩn (R8). `prefers-reduced-motion: reduce` → khói **đứng yên** (một vệt mờ
  tĩnh, độ đậm 0,12) ngay từ đầu.
- `incense = false` → dùng ảnh `bat-huong-khong-thap`, không có khói.
- Lưu ý để chủ dự án xác nhận: vệt khói động là **hiệu ứng** của giao diện (05 §1 "ba nén hương có khói nhẹ"), không
  phải ảnh đồ vật. Nếu chủ dự án muốn khói cũng phải "thật", phương án thay thế là để khói thật trong ảnh và **bỏ** hiệu
  ứng động.

### 3.5 Quy cách tệp ảnh

**Bản gốc (master)** cho từng món: PNG nền trong suốt, hệ màu sRGB, **cạnh dài ≥ 2000 điểm ảnh**, mép cắt sạch (không
viền trắng), đã **xóa siêu dữ liệu** (EXIF, GPS, tên máy). Bản gốc chưa cắt (JPEG/HEIC/RAW từ điện thoại) cũng được giữ.
Bản gốc **không** đưa vào kho mã (nặng); lưu ở kho ảnh của chủ dự án, ghi đường dẫn và mã băm SHA-256 trong sổ giấy
phép.

**Bản phát hành:** WebP có kênh trong suốt, ba mật độ **1×, 2×, 3×** theo bề rộng hiển thị ở §3.2 (ví dụ `ga-luoc`
rộng 110 điểm CSS → 110, 220, 330 điểm ảnh). Chất lượng 75–82, kênh trong suốt chất lượng ≥ 90, không siêu dữ liệu.
Tên tệp: `mam-tt-<mã>@1x.webp`, `@2x`, `@3x`.

**Ngân sách:** toàn bộ ảnh mâm truyền thống, **cả ba mật độ cộng lại, ≤ 600 KB cho mỗi nền tảng**. Phân bổ gợi ý
(tổng 1× + 2× + 3× của từng món):

| Mã | KB | Mã | KB |
|---|---|---|---|
| `ga-luoc` | 90 | `dia-xoi` | 40 |
| `bat-huong` | 80 | `bat-com-trung` | 35 |
| `bat-huong-khong-thap` | 45 | `khoanh-gio` | 25 |
| `lo-hoa` | 70 | `mam` | 40 |
| `dia-hoa-qua` | 70 | `nen-trai` + `nen-phai` | 50 |
| `chen-nuoc` | 15 | `mat-ban` | 30 |
| | | **Tổng** | **590** |

**Tải lười, không ảnh hưởng lần tải đầu:**
- PWA: ảnh ở thư mục riêng của mô-đun tưởng niệm, chỉ tải khi mở `#/nguoi-than/tuong-niem/<id>` (`<img srcset>` hoặc
  `image-set`, `decoding="async"`); **không** nằm trong danh sách tải trước của service worker lúc cài; lưu bộ nhớ đệm
  sau lần xem đầu để dùng ngoại tuyến. Kiểm thử P3/P6: lần tải đầu có **0 byte** ảnh mâm. Ảnh cùng nguồn gốc nên CSP
  `img-src 'self'` không đổi.
- Android: `drawable-mdpi` (1×), `drawable-xhdpi` (2×), `drawable-xxhdpi` (3×). iPhone/iPad/Mac: danh mục tài nguyên
  `@1x`/`@2x`/`@3x`. Windows: `scale-100`/`scale-200`/`scale-300`. Bản cài đóng gói sẵn ảnh (không tải mạng, R7) và
  chỉ giải mã khi mở màn hình tưởng niệm.

### 3.6 Tệp kê khai ghép

Khi có ảnh, vai nội dung thêm `content/memorial/anh/mam-truyen-thong.json`: với mỗi lớp ghi mã, tên tệp, thứ tự lớp,
tâm x, chân y, bề rộng (điểm CSS theo §3.2), và với `bat-huong` thêm `incense_tips: [[x, y], [x, y], [x, y]]` (điểm
CSS trong khung). Bốn nền tảng đọc chung tệp này để ghép giống nhau.

### 3.7 Lời mô tả (alt) từng món

| Mã | Lời mô tả |
|---|---|
| `bat-com-trung` | "Bát cơm trắng, trên có quả trứng luộc" |
| `khoanh-gio` | "Khoanh giò lụa" |
| `ga-luoc` | "Gà luộc nguyên con trên đĩa" |
| `dia-xoi` | "Đĩa xôi" |
| `dia-hoa-qua` | "Đĩa hoa quả" |
| `bat-huong` | "Bát hương nhiều chân hương, ba nén hương đang cháy" |
| `bat-huong-khong-thap` | "Bát hương" |
| `lo-hoa` | "Lọ hoa ly" hoặc "Lọ hoa cúc vàng" (theo ảnh được chọn) |
| `nen-trai`, `nen-phai` | "Cây nến đang cháy" |
| `chen-nuoc` | "Chén nước trong" |
| `mam` | "Mâm cúng" |
| `mat-ban` | "" (trang trí, bỏ qua) |

Cả mâm vẫn là **một** hình với một nhãn chung (§2 Trợ năng): "Mâm cơm cúng: bát cơm có trứng luộc, khoanh giò, gà
luộc, đĩa xôi, đĩa hoa quả; bát hương với ba nén hương đang cháy; lọ hoa; hai cây nến; chén nước." Lời mô tả từng món
dùng khi một nền tảng hiển thị các lớp như hình riêng có thể đọc được; khi đó nhãn chung ở khung, từng lớp mang lời mô
tả của mình.

### 3.8 Riêng kiểu này không hiện

- Hình vẽ, ảnh do máy tạo ra, ảnh dựng 3D; ảnh mẫu có hình mờ (watermark); ảnh lấy trên mạng không rõ giấy phép.
- Người, tay, mặt, đồ dùng có chữ hoặc logo hãng, nhãn giá, túi ni lông, màng bọc thực phẩm, đũa nhựa, điện thoại, phản
  chiếu người chụp trên đồ đồng.
- Thịt sống, máu, món cháy, đồ ăn bị cắn dở; hoa héo, quả dập.
- Lật gương ảnh để làm cặp nến (sai hướng sáng).

Hình vẽ kiểu 1 trước đây (bố cục, màu ở bản tạm SVG) giữ nguyên trong `mam-truyen-thong.svg` cho tới khi có ảnh.
Màu nền, đồng và chữ bia vẫn theo §2 và §6.

## 4. Kiểu 2 — Chay (thiên về Phật giáo)

Tệp mẫu: `content/memorial/mam-chay.svg`. Theo sáu thứ dâng cúng mà [N8] nêu: "có hương, hoa, đăng; có trà, quả,
thực", và lời khuyên [N8] "Vào những ngày giỗ kỵ … nên làm chay thanh tịnh mới tốt." Không có món mặn.

**Đồ vật, từ trái sang phải**

| Vị trí (x) | Đồ vật | Nguồn |
|---|---|---|
| 38 | Nến ("đăng") | [N8]; [N9] |
| 88 | Đĩa quả trên chân gỗ ("quả") | [N8]; [N4] "nhang đèn bông trái" |
| 134 | Chén trà trên đĩa lót ("trà") | [N8] |
| 180 | Bát hương, ba nén hương ("hương") | [N8]; [N3]; [N9] |
| 180 (phía trước) | Bát nước trong | [N2]; [N8] "chén nước lạnh tinh khiết với hoa quả, một nén hương thành tâm là đủ" |
| 228 | Bát cơm chay ("thực") | [N8] |
| 278 | Độc bình men xanh nhạt cắm sen (một bông nở, một nụ, một lá) ("hoa") | [N2] "trên bàn thờ của những gia đình theo Phật giáo phổ biến còn bày một chiếc độc bình lớn". **Hoa sen: chưa có nguồn đã đọc nói dùng sen trên bàn thờ giỗ** — chọn vì gợi Phật giáo mà không cần biểu tượng tôn giáo; có thể thay bằng hoa huệ [N9] nếu duyệt thấy cần |
| 322 | Nến | như trên |

**Màu:** giữ nến, đồng, gỗ như kiểu 1 để cùng một bộ; thay đỏ của chân đĩa bằng gỗ `#5a2c12`/`#8a4b22`, mặt đĩa
`#d9c49a`; đồ sứ men xanh nhạt `#c9d6c3`, viền `#a9b8a2`/`#8fa587`; cơm `#fbf7ee`; trà `#b08a3e`; sen `#f3c1cc`,
`#e7a3b3`, `#d9879b`; lá và cuống `#5f8f4a`.

**Mô tả cho trình đọc màn hình:** "Mâm chay: hai cây nến, đĩa quả, chén trà, bát hương có ba nén hương, bát nước trong,
bát cơm, bình hoa sen."

**Riêng kiểu này không hiện:** món mặn, rượu; tượng Phật, chữ Vạn, chữ Phạn, bánh xe pháp — kiểu này dành cho cả
người ăn chay vì gia đình, không chỉ người theo đạo.

## 5. Kiểu 3 — Tối giản hiện đại

Tệp mẫu: `content/memorial/mam-toi-gian.svg`. Dành cho nhà ở chung cư, bàn thờ treo nhỏ, hoặc người thích ít chi tiết.
Ít đồ, nhiều khoảng trống; vẫn giữ bát hương ở giữa.

**Đồ vật, từ trái sang phải**

| Vị trí (x) | Đồ vật | Nguồn |
|---|---|---|
| 64–296 | Kệ gỗ thấp, mỏng, viền đồng mảnh, hai chân ngắn | — (thiết kế) |
| 116 | Một nến trắng trong cốc thủy tinh | [N4] "chỉ cần nhang đèn bông trái" (một đèn — giản lược; nguồn nói hai đèn [N2], [N9]) |
| 180 | Bát hương nhỏ dáng tròn, ba nén hương | [N9]; [N3] |
| 214 | Chén nước trong | [N2]; [N8] |
| 250 | Bình gốm mảnh màu kem, một bông hoa trắng | [N9] "Hoa tươi … đặt ở một bên" |

Không có đĩa quả: nguồn coi quả là lễ vật phổ biến [N9] nhưng kiểu này chủ ý tối giản. Nếu duyệt thấy thiếu, thêm một
đĩa nhỏ ba quả ở x ≈ 146. Khi `incense = false`: bỏ bát hương, nén hương và khói; nến dời vào x = 180.

**Màu:** kệ `#6b3a1c`, viền `#e8c67a` (độ mờ 0,55), chân `#3d1d0b`; nến `#efe6d6`, cốc thủy tinh `#f3e6d0` mờ 12 %
viền `#e8c67a`; bát hương `#b98b3e`, miệng `#e8c67a`; chén `#efe6d6`; bình `#d9cdb8` viền `#8a6a32`; hoa `#f3e6d0`, nhụy
`#e3b45c` (biến `--gold` tối); cuống, lá `#5f8f4a`. Không dùng đỏ ngoài phần đỏ nhỏ của chân hương.

**Mô tả cho trình đọc màn hình:** "Kệ thờ nhỏ: một ngọn nến, bát hương có ba nén hương, chén nước, bình một bông hoa
trắng."

**Riêng kiểu này không hiện:** đồ trang trí hiện đại không liên quan (đèn LED nhấp nháy, nến điện tử, chữ, biểu
tượng cảm xúc).

## 6. Bảng đo tương phản (đồ họa, WCAG 1.4.11; tính bằng công thức độ sáng tương đối)

Nền: `#5a2a1e` (phần sáng nhất, trên cùng), `#3a1a14` (giữa, nơi đặt mâm), `#1d0f0c` (góc tối).

| Màu | Dùng cho | `#5a2a1e` | `#3a1a14` | `#1d0f0c` |
|---|---|---|---|---|
| `#ffd27a` | ngọn lửa | 8,27 | 11,03 | 13,10 |
| `#e8c67a` | đồng, chữ bia | 7,17 | 9,56 | 11,36 |
| `#b98b3e` | bát hương kiểu 3, viền ảnh | 3,82 | 5,10 | 6,06 |
| `#f2b632` | cúc vàng | 6,45 | 8,61 | 10,23 |
| `#e7a3b3` | sen | 5,76 | 7,69 | 9,14 |
| `#c9d6c3` | đồ sứ men xanh | 7,78 | 10,38 | 12,34 |
| `#efe6d6` | chén, nến trắng | 9,50 | 12,68 | 15,06 |
| `#d9cdb8` | bình gốm kiểu 3 | 7,49 | 10,00 | 11,88 |
| `#7fae3e` | bưởi | 4,50 | 6,00 | 7,13 |
| `#e8892b` | cam | 4,51 | 6,01 | 7,14 |
| `#5f8f4a` | lá, cuống | 3,09 | 4,12 | 4,90 |
| `#ff8a3a` | đầu hương | 5,01 | 6,69 | 7,95 |
| `#c4161c` | thân nến đỏ | 1,95 | **2,60** | 3,09 |
| `#7a5a3a` | chân hương | 1,88 | **2,50** | 2,98 |
| `#2f5d8a` | bình xanh kiểu 1 | 1,71 | **2,28** | 2,71 |

Đề xuất sửa khi cài (giữ tinh thần màu): chân hương `#a8845a` (4,57 trên `#3a1a14`); bình xanh `#5a8cc0` (4,45); thân
nến đỏ giữ `#c4161c` nhưng thêm viền 1 px `#e8c67a` hai bên, hoặc đổi `#e04a3c` (3,90). Chữ bia `#e8c67a` trên đá
`#2b2422`: 9,28 : 1 (khớp 08 §6).

## 7. Bản tạm hình vẽ và những gì đã sửa sau khi xem

Bản mẫu được dựng ra PNG bằng Chromium (Playwright có sẵn), cỡ 1080 × 600, rồi xem bằng mắt:
- Kiểu 1, bản đầu: đường trang trí trên bình hoa trông như **một khuôn mặt cười** → đổi thành hai đường viền ngang.
  Bát nước trôi lơ lửng ngoài mép bàn → làm mặt bàn có chiều sâu (mặt trên `#6b3a1c`, cạnh trước dải gỗ), đồ vật đứng
  trên mặt bàn. Khói quá đậm, giống sợi dây → nét dày hơn, mờ hơn (độ mờ 0,16).
- Kiểu 2: bát cơm trông như quả trứng → thân bát men xanh, cơm trắng có vài hạt.
- Kiểu 3: nến tealight quá thấp, lửa trôi trên cốc → nến cao trong cốc thủy tinh; bình màu đá `#2b2422` gần như lẫn vào
  nền (≈ 1,03 : 1) → đổi kem `#d9cdb8` (10 : 1).

Ảnh PNG không đưa vào kho (tạo lại được từ SVG). Vai giao diện chụp ảnh màn hình thật theo docs/08 §6 khi cài.

## 8. Việc cho vai giao diện (khi Q-1 đã có trả lời)

1. Thêm lựa chọn "Kiểu mâm" vào nhóm "Tưởng niệm" của biểu mẫu người đã mất (08 §3.4): ba nút có hình nhỏ và chữ
   "Truyền thống", "Mâm chay", "Tối giản"; giữ `cong_giao` từ sao lưu như hiện nay. Lược đồ và kiểm tra dữ liệu nhận
   `toi_gian` (01 Bổ sung YC-2026-09).
2. Kiểu 1: ghép ảnh theo §3 và tệp kê khai §3.6; dùng bản tạm SVG cho tới khi ảnh được duyệt; đo ngân sách 600 KB và
   kiểm 0 byte ảnh mâm ở lần tải đầu.
3. Kiểu 2, 3 vẽ theo các tệp mẫu (bản tạm, chờ Q-2); mọi tệp hình mới thêm dòng vào `content/memorial/SOURCES.md` (R8 phần "Nội dung").
4. Kiểm thử khói: R8 (a)–(f) cho từng kiểu; ảnh chụp 360/1280 px, sáng/tối, mỗi kiểu một ảnh, và một ảnh
   `prefers-reduced-motion`.
5. Sửa ba màu dưới 3 : 1 ở §6 (cho hình vẽ).
