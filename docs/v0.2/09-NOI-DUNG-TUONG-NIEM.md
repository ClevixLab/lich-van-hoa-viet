# Nội dung tưởng niệm — bản thiết kế ba kiểu mâm lễ

Trạng thái: **BẢN THẢO của vai nội dung (B8), 2026-11-07 — chưa khóa, chờ duyệt thiết kế (docs/08 §6).**
Tài liệu này là đầu vào cho vai giao diện các nền tảng; vai nội dung không sửa mã ứng dụng.

Căn cứ:
- 05 §1: "Màn hình tưởng niệm ngày giỗ (bia, ảnh, mâm theo 3 phong cách, ba nén hương có khói nhẹ, lời tưởng niệm của
  người dùng); trang nghiêm, không quảng cáo, không âm thanh; hình vẽ mới cho dự án."
- 08 §3.9 (màn hình đang chạy), §6 (trợ năng), §11 (chọn kiểu mâm chưa có trên giao diện).
- 04 R8 (khói, CPU, giảm chuyển động, không âm thanh, Công giáo không hương), R9 (biên bản nguồn phong tục).
- Nguồn phong tục: `content/memorial/PHONG-TUC.md` (mã N1–N10). Bản mẫu hình: `content/memorial/mam-*.svg`.

## 1. Ba kiểu và chỗ còn phải quyết

| Kiểu trong tài liệu này | Tệp mẫu | Giá trị `tray_style` (01 §3.7) |
|---|---|---|
| 1. Truyền thống (đơn giản, như bản đang chạy) | `mam-truyen-thong.svg` | `truyen_thong` (mặc định) |
| 2. Chay (thiên về Phật giáo) | `mam-chay.svg` | `phat_giao` |
| 3. Tối giản hiện đại | `mam-toi-gian.svg` | **chưa có** — xem câu hỏi Q-1 |

**Q-1 (cho điều phối viên và chủ dự án).** Mô hình 01 §3.7 đã khóa ba giá trị `phat_giao`, `truyen_thong`,
`cong_giao`. Kiểu "tối giản hiện đại" không trùng giá trị nào. Hai cách:
- **(A) Đề xuất:** thêm giá trị `toi_gian` bằng một Bổ sung cho 01 và lược đồ JSON. Cần tính B5 (R14): bản cũ đọc tệp
  sao lưu có `toi_gian` phải không mất dữ liệu (ví dụ coi như `truyen_thong` khi hiển thị nhưng giữ nguyên giá trị).
- (B) Không thêm giá trị; kiểu tối giản chỉ là một cách vẽ khác của `truyen_thong`, chọn theo cài đặt hiển thị của máy
  (không vào sao lưu).
- **Không** dùng kiểu tối giản thay cho `cong_giao`: Công giáo là lựa chọn tôn giáo, không phải phong cách. Hồ sơ
  `cong_giao` tiếp tục được vẽ như 08 §3.9 (hai nến và hoa, không bát hương, không khói; R8 (f)).

Mặc định nếu chưa trả lời: giữ một kiểu `truyen_thong` như hiện nay; hai kiểu còn lại chờ Q-1.

## 2. Quy tắc chung cho cả ba kiểu

**Bố cục.** Mâm nằm dưới bia, trên lời tưởng niệm (08 §3.9). Khung vẽ 360 × 200 đơn vị (tỉ lệ 9 : 5). Ở 360 px
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
- Tương phản đồ họa (WCAG 1.4.11, ≥ 3:1 với nền `#3a1a14` ở giữa khung): các hình nhận biết được nhờ phần sáng (ngọn
  lửa, đồng, hoa, đồ sứ trắng). Bảng đo ở §6. Ba chỗ dưới 3:1 cần sửa khi cài: thân nến đỏ, chân hương, bình xanh của
  kiểu 1 (§6).
- Không truyền thông tin chỉ bằng màu; mâm là trang trí, mọi thông tin (ngày giỗ, tuổi) đã có bằng chữ trên bia.
- Cỡ chữ A/A+/A++ không đổi mâm (mâm không chứa chữ); mâm co theo bề rộng, không tràn ngang.

**Những điều không bao giờ hiện trên màn hình tưởng niệm (mọi kiểu).**
- Quảng cáo, liên kết mua hàng, nút chia sẻ mạng xã hội, âm thanh, video, rung (05 §1, R8 (e)).
- Thông tin báo tử: người báo tin, nơi mất, lý do, ảnh giấy báo tử, ghi chú báo tử (R-PRIV-2, 08 §12g).
- Tên người trong tiêu đề tab, URL, tên tệp (08 §3.9, S4').
- Món mặn vẽ thật (gà, đầu lợn, thịt) — dễ thô ở cỡ nhỏ; mâm cỗ mặn được gợi bằng đĩa quả (và đĩa xôi nếu thêm).
- Vàng mã, tiền giấy, bùa chú; hình đầu lâu, ma quỷ, máu; mặt cười hay mắt mũi trên đồ vật (kể cả vô tình — xem §7).
- Biểu tượng tôn giáo người dùng không chọn: tượng Phật, chữ Vạn, thánh giá, tràng hạt. Kiểu 2 chỉ "thiên về" Phật
  giáo qua đồ cúng chay và hoa sen, không vẽ tượng hay chữ.
- Hiệu ứng lửa cháy lớn, pháo, hiệu ứng lấp lánh, chữ chạy.
- Chữ Hán-Việt cổ ("Hiển khảo"…) (quyết định của chủ dự án, PHONG-TUC §1).

## 3. Kiểu 1 — Truyền thống (đơn giản)

Tệp mẫu: `content/memorial/mam-truyen-thong.svg`. Giữ đúng bố cục của bản đang chạy (`pwa/src/people/memorial.ts`),
thêm **bát nước trong** trước bát hương.

**Đồ vật, từ trái sang phải**

| Vị trí (x) | Đồ vật | Nguồn |
|---|---|---|
| 38 | Nến đỏ trên chân đồng | [N9] "Hai chân nến ở hai bên."; [N2] "hai cây đèn tượng trưng cho mặt trời mặt trăng" |
| 104 | Đĩa quả trên chân đỏ: một quả bưởi xanh, bốn quả cam/quýt | [N9] "Trái cây tươi là lễ vật dâng hằng ngày phổ biến nhất."; [N2] "hai cái đĩa để đặt hoa quả lễ" (vẽ một đĩa — giản lược) |
| 180 | Bát hương đồng, ba nén hương, khói nhẹ | [N9] bát hương ở chính giữa; [N3] ba nén |
| 180 (phía trước) | Bát nước trong | [N2] "đặt phía trước bát hương là một bát nước trong" |
| 256 | Bình hoa cúc vàng | [N9] "Hoa tươi, thường là cúc vạn thọ hoặc hoa huệ, đặt ở một bên." |
| 322 | Nến đỏ trên chân đồng | như trên |

Tùy chọn khi cài (không bắt buộc): một đĩa xôi nhỏ (khối tròn màu `#efe6d6`/`#f2d27a`) giữa đĩa quả và bát hương, gợi
mâm cỗ [N9] "Xôi, gà luộc và các món thịt heo là phổ biến." Chỉ vẽ xôi, không vẽ món thịt (§2).

**Màu:** nến `#c4161c`/`#8f1416` (biến `--red`, `--red-deep`), ngọn lửa `#ffd27a` + lõi `#ff8a3a`; chân nến và bát
hương dải đồng `#8a5e0c → #e8c67a`; đĩa quả chân `#8f1416`, mặt `#c99a4a`; bưởi `#7fae3e`, cam `#e8892b`/`#f0a23a`;
hoa `#f2b632`, nhụy `#c98a12`, lá `#4f7a3a`; bình `#2f5d8a` (đề xuất đổi `#5a8cc0`, §6); chén nước `#efe6d6`, mặt
nước `#bcd3dc`; chân hương `#7a5a3a` (đề xuất `#a8845a`), phần đỏ `#a3271b`, đầu hương `#ff8a3a`.

**Mô tả cho trình đọc màn hình:** "Bàn thờ: hai cây nến, đĩa quả, bát hương có ba nén hương, bát nước trong, bình hoa
cúc." (`incense = false`: bỏ "có ba nén hương".)

**Riêng kiểu này không hiện:** món mặn vẽ thật; hoa đặt giữa thay cho bát hương.

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

## 7. Ảnh bản mẫu và những gì đã sửa sau khi xem

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
   "Truyền thống", "Mâm chay", "Tối giản"; giữ `cong_giao` từ sao lưu như hiện nay.
2. Vẽ theo các tệp mẫu; mọi tệp hình mới thêm dòng vào `content/memorial/SOURCES.md` (R8 phần "Nội dung").
3. Kiểm thử khói: R8 (a)–(f) cho từng kiểu; ảnh chụp 360/1280 px, sáng/tối, mỗi kiểu một ảnh, và một ảnh
   `prefers-reduced-motion`.
4. Sửa ba màu dưới 3 : 1 ở §6.
