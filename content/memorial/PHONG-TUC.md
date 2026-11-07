# Biên bản nguồn phong tục — mô-đun Người thân và màn hình tưởng niệm

Trạng thái: **bản 2 (vai nội dung), 2026-11-07.** Bản đầu do vai PWA lập trong vòng duyệt thiết kế TK-24; bản này bổ
sung giỗ đầu, giỗ hết, giỗ thường, tiên thường, cách gọi theo vùng, Ông Tổ họ, Bà Cô Tổ họ, Bé đỏ và đồ cúng trên mâm.
Việc **duyệt bởi người am hiểu được hoãn** theo quyết định của chủ dự án ngày 06/11/2026 — rủi ro mở **R-CUL-1**
(05 §7). Phong tục khác nhau theo vùng, tôn giáo và từng gia đình (R-CUL-2); ứng dụng theo **cách phổ biến** và để
người dùng tự sửa nhãn.

Quy tắc ghi nguồn:
- Chỉ ghi điều mà nguồn **nói ra**. Câu trong ngoặc kép là trích nguyên văn, đã đối chiếu với trang nguồn ngày
  2026-11-07. Danh sách nguồn đầy đủ ở §6 và `SOURCES.md`.
- Điều chưa tìm được nguồn kiểm chứng thì ghi **CHƯA KIỂM CHỨNG** ở §5, không bịa nguồn.
- Nguồn đã đọc mà **không** nói tới một điều cũng được ghi lại, để người sau khỏi tìm lại chỗ đó.

Quyết định của chủ dự án được giữ nguyên trong toàn bộ tài liệu:
1. Dùng từ thường ngày, viết hoa chữ đầu: Ông, Bà, Cụ, Kỵ, Bác, Cô, Chú, Dì, Cậu, Mợ, Thím, Anh, Chị, Em, Cháu, Con,
   Bạn, Bố, Mẹ, Ông Tổ họ, Bà Cô Tổ họ, Bé đỏ (YC-2026-07).
2. Không dùng từ Hán-Việt cổ trên bia hay trong gợi ý (ví dụ "Hiển khảo") (YC-2026-06).
3. Với người đã mất chỉ nhắc ngày giỗ (04 Bổ sung B4).
4. Thông tin báo tử là riêng tư: không hiện trên màn hình tưởng niệm, lời nhắc, tệp .ics (R-PRIV-2).

## 1. Chữ trên bia: quan hệ bằng từ thường ngày

**Quyết định của chủ dự án (07/11/2026):** không dùng các từ Hán-Việt cổ, ít người còn hiểu (như "Hiển khảo"). Vì vậy:

- Ứng dụng **không có** ô "dòng đầu bia" riêng và **không gợi ý** từ Hán-Việt nào.
- Bia trên màn hình tưởng niệm chỉ ghi, phía trên tên, **đúng chữ người dùng gõ ở ô "Quan hệ với bạn"**. Ứng dụng viết
  hoa chữ đầu và bỏ phần trong ngoặc, ví dụ "ông nội (bên nội)" → "Ông nội". Ô trống thì không ghi gì.
- Trường `memorial.bia_tieu_de` của 01 vẫn được giữ trong dữ liệu để sao lưu dùng được giữa các nền tảng; giao diện
  không hiển thị và không sửa trường này (08 §12e).

| Gợi ý trong ô "Quan hệ với bạn" | Người đã khuất là… | Nguồn |
|---|---|---|
| Bố, Mẹ | cha, mẹ | từ thường ngày (không cần nguồn) |
| Ông nội, Bà nội | cha, mẹ của cha | [N1] "Ông Nội: Cha của cha."; "Miền Bắc và miền Nam đều gọi chung là “ông Nội, bà Nội” và “ông Ngoại, bà Ngoại”." |
| Ông ngoại, Bà ngoại | cha, mẹ của mẹ | [N1] "Ông Ngoại: Cha của mẹ." |
| Cụ ông, Cụ bà | ông bà của cha hoặc của mẹ — cách gọi miền Bắc | [N1] "Cố: Ông bà của cha."; "Cố: Ông bà của mẹ."; "người Bắc thường gọi là “Cụ” trong khi ở miền Nam thường sẽ gọi là “Cố”" |
| Ông cố, Bà cố | như trên — cách gọi miền Nam | [N1], cùng câu trên |
| Kỵ ông, Kỵ bà | đời trên cụ — miền Bắc và miền Trung | [N1] "Trên ông Cố sẽ gọi là “Kỵ ông/ Kỵ bà” ở miền Bắc và miền Trung." |
| Ông sơ, Bà sơ | như trên — miền Nam | [N1] "Còn miền Nam sẽ gọi là “ông Sơ” và “bà Sơ”." |
| Bác | anh của cha (và vợ của bác: Bác gái) | [N1] "Bác: Anh trai của cha."; "Bác gái: Vợ của bác." |
| Chú | em trai của cha | [N1] "Chú: Em trai của cha."; miền Nam hay gọi kèm thứ: "chú Ba" |
| Thím | vợ của chú | [N1] "Thím: Vợ của chú." |
| Cô | chị hoặc em gái của cha | [N1] "Cô: Chị gái và em gái của cha." |
| Dì | chị hoặc em gái của mẹ | [N1] "Dì: Chị và em gái của mẹ đều được gọi là dì." |
| Cậu | anh hoặc em trai của mẹ | [N1] "Cậu: Anh và em trai của mẹ đều được gọi là cậu"; miền Nam có "Cậu Út" |
| Mợ | vợ của cậu | [N1] "Mợ: Vợ của cậu được gọi là mợ." |
| Anh, Chị, Em, Cháu, Con, Chồng, Vợ, Bạn | người cùng đời, đời dưới, vợ chồng, bạn | từ thường ngày; người dùng tự sửa |
| Ông Tổ họ | ông tổ chung của cả dòng họ | [N5] "Mỗi họ có một ông Tổ chung. Ngày giỗ ông Tổ gọi là ngày giỗ họ hoặc giỗ Tổ." |
| Bà Cô Tổ họ | người con gái trong họ mất khi còn trẻ, chưa chồng, được họ nội thờ | [N6] "a family or clan's female member who died at a young age while being unmarried"; "highly respected and worshipped by the patrilineage’s male members in Northern Vietnam" |
| Bé đỏ | thai nhi mất (vong linh thai nhi) | [N7] "lễ cầu siêu cho vong linh thai nhi (còn gọi là các Bé Đỏ)" |

Ghi chú về vùng miền và cách viết:
- "Kỵ" là cách gọi của miền Bắc và miền Trung, miền Nam gọi "sơ"; "Cụ" là cách gọi miền Bắc, miền Nam gọi "cố" [N1].
  Danh sách gợi ý có cả hai cách để người dùng chọn theo gia đình mình.
- [N1] chỉ nêu khác biệt Bắc – Nam ở Cụ/Cố, Kỵ/Sơ, Chú, Cậu; với Bác, Thím, Cô, Dì, Mợ nguồn này không nêu khác biệt.
- [N6] viết "Bà Tổ Cô"; chủ dự án dùng "Bà Cô Tổ họ". Ứng dụng giữ cách viết của chủ dự án. Hai cách viết là một
  người hay không: **chưa có nguồn đã đọc nói rõ** (§5). [N6] cũng nói tới "Ông Mãnh" — "men who died young and
  unmarried"; ứng dụng chưa gợi ý từ này.
- Nghĩa của "Bé đỏ" trong [N7] là thai nhi. Dùng "Bé đỏ" cho **trẻ đã sinh ra rồi mất sớm** thì **chưa kiểm chứng** (§5).
- Theo YC-2026-07, Cụ, Kỵ, Bà Cô Tổ họ, Ông Tổ họ, Bé đỏ **không** có trong gợi ý "Cách gọi" của người đang sống.

Ô "Cách gọi (đứng trước tên)" (`danh_xung`) chỉ dùng khi ô "Quan hệ" trống. Khi bia đã ghi quan hệ thì không ghi cách
gọi trước tên nữa, để không lặp ("Ông nội / Nguyễn Văn A", không phải "Ông nội / Ông Nguyễn Văn A").

## 2. Ngày giỗ: các lựa chọn ứng dụng đang dùng

| Lựa chọn trong ứng dụng | Nguồn | Mục mở |
|---|---|---|
| Giỗ tính theo **ngày mất âm lịch**, hằng năm | [N9] "Đây là ngày kỷ niệm ngày mất, được tính theo âm lịch."; "được tổ chức hằng năm vào ngày mất theo âm lịch". Ý kiến khác: [N10] (Công giáo) "tính theo Dương lịch hay Âm lịch tùy người" | — |
| **Giỗ đầu** = lần giỗ thứ nhất, một năm sau năm mất | [N4] "Giỗ Đầu gọi là Tiểu tường (chữ Hán: 小祥), là ngày giỗ đầu tiên, sau thời gian người mất đúng một năm"; [N9] "Giỗ đầu là quan trọng nhất." | — |
| **Giỗ hết** = năm thứ 2 sau năm mất (mặc định), cho chọn năm thứ 3 | [N4] "Giỗ Hết gọi là Đại tường (chữ Hán: 大祥), là ngày giỗ sau thời gian người mất hai năm, vẫn nằm trong kỳ tang chế." Năm thứ 3: [N9] chỉ nói "Sau ba năm, người đã khuất được coi là đã hoàn toàn hòa nhập vào dòng dõi tổ tiên." — không nói giỗ hết vào năm thứ 3. Lựa chọn "năm thứ 3" **chưa có nguồn trực tiếp** (§5) | O-2 |
| **Giỗ thường** (từ năm thứ 3 trở đi) — ứng dụng ghi "Giỗ (lần thứ n)" | [N4] "Giỗ Thường còn gọi là ngày Kiết kỵ (chữ Hán: 吉忌), là ngày giỗ sau thời gian người mất từ ba năm trở đi." Ứng dụng **không** dùng chữ "Kiết kỵ"/"Cát kỵ" (quyết định dùng từ thường ngày) | — |
| **Tiên thường** = lễ cúng hôm trước ngày giỗ; mặc định **tắt nhắc** | [N4] "Lễ Tiên thường (先嘗) lễ cúng vào ngày trước người chết qua đời một hôm, lễ Chính kỵ (正忌) chính là ngày mất."; "Tiên thường còn được gọi là ngày Cáo giỗ". [N5] cũng có "ăn uống từ bữa tiên thường". Mặc định tắt là quyết định sản phẩm (O-1), không phải phong tục | O-1 |
| Ngày giỗ chính ("Chính kỵ") là đúng ngày mất âm lịch | [N4], câu trên | — |
| Giỗ họ (giỗ Ông Tổ họ) | [N5] "Ngày giỗ ông Tổ gọi là ngày giỗ họ hoặc giỗ Tổ."; "Mỗi họ đều có một nhà thờ Tổ. Giỗ Tổ cúng tại nhà thờ này." Ứng dụng chỉ nhắc theo ngày người dùng nhập; không có chức năng riêng cho giỗ họ | — |
| Người mất **tháng nhuận** → giỗ vào tháng thường cùng số | **CHƯA KIỂM CHỨNG** — chưa đọc được nguồn nói điều này (§5) | O-4 |
| Mất **ngày 30** mà năm đó tháng chỉ có 29 ngày → giỗ ngày 29 | **CHƯA KIỂM CHỨNG** (§5) | — |
| **Hưởng thọ / hưởng dương**, ngưỡng 60 tuổi âm | **CHƯA KIỂM CHỨNG** — từ ngữ dùng phổ biến, nhưng chưa đọc được nguồn nêu ngưỡng 60 (§5). Ngưỡng sửa được theo từng hồ sơ | O-3 |
| Tuổi âm (tuổi mụ) | Theo đặc tả 02 §4; chưa có dòng nguồn phong tục riêng (§5) | — |

## 3. Bàn thờ và mâm lễ trên màn hình tưởng niệm

### 3.1 Thành phần chung (kiểu truyền thống đang có trong ứng dụng)

| Thành phần trong hình | Nguồn | Ghi chú |
|---|---|---|
| Bát hương ở giữa | [N9] "Một chiếc bàn hoặc kệ gỗ đặt lư hương bằng đồng (bát hương) ở chính giữa."; [N2] "đặt phía trước bát hương là một bát nước trong" | |
| Hai cây nến hai bên | [N9] "Hai chân nến ở hai bên."; [N2] "hai cây đèn tượng trưng cho mặt trời mặt trăng, còn hương là tinh tú." | |
| Đĩa quả | [N9] "Trái cây tươi là lễ vật dâng hằng ngày phổ biến nhất."; [N2] "đặt xen giữa đèn và hương là hai cái đĩa để đặt hoa quả lễ" | Hình vẽ **một** đĩa quả (nguồn nói hai đĩa) — giản lược có chủ ý. [N2] không có cụm "mâm ngũ quả" |
| Bình hoa | [N9] "Hoa tươi, thường là cúc vạn thọ hoặc hoa huệ, đặt ở một bên."; [N2] "trên bàn thờ của những gia đình theo Phật giáo phổ biến còn bày một chiếc độc bình lớn" | [N2] không nói bình đặt bên nào; [N9] chỉ nói "ở một bên". Hình đặt bên phải bát hương |
| Ba nén hương | [N3] "Thông thường thắp 1 hoặc 3 nén hương là thắp cho gia tiên."; "3 nén hương tượng trưng cho sự hòa hợp của Thiên - Địa - Nhân"; "Cắm từng nén hương vào bát hương." | |
| Bát nước trong | [N2] câu trên | Chưa vẽ trong bản đang chạy; đề xuất thêm (docs/v0.2/09) |

### 3.2 Đồ cúng trên mâm theo ba kiểu (01 §3.7, R9)

| Kiểu | Đồ cúng thường gặp | Nguồn |
|---|---|---|
| Truyền thống — mâm cơm cúng (**chủ dự án quyết, YC-2026-09**) | Theo lời chủ dự án: bát cơm có quả trứng luộc, một khoanh giò, một con gà luộc, một đĩa xôi, một đĩa hoa quả tổng hợp, ba nén nhang trên bát hương có rất nhiều chân hương đã cháy hết, tàn cong; lọ hoa ly hoặc cúc vàng. Nguồn hỗ trợ: món người mất ưa thích; xôi, gà luộc, món thịt lợn; miền Bắc có bánh chưng, miền Nam có bánh tét | [N9] "Các món ăn yêu thích của người đã khuất được chuẩn bị"; "Xôi, gà luộc và các món thịt heo là phổ biến."; "Ở miền Bắc, bánh chưng … thường xuất hiện."; "Ở miền Nam, bánh tét … phổ biến hơn." |
| Phật giáo — mâm chay | Hương, hoa, đèn (nến), trà, quả, món chay; tối thiểu nước sạch, hoa quả, một nén hương | [N8] "Vào những ngày giỗ kỵ tổ tiên, ông bà hay bố mẹ, người thân của mình nên làm chay thanh tịnh mới tốt."; "có hương, hoa, đăng; có trà, quả, thực"; "chén nước lạnh tinh khiết với hoa quả, một nén hương thành tâm là đủ". [N4] "Nhà nghèo thì chỉ cần nhang đèn bông trái, vài món ăn giản dị cúng người mất cũng được rồi." |
| Công giáo — nến và hoa, **không hương** | Cầu nguyện, dâng lễ ở nhà thờ; không cúng | [N10] "Chúng ta không cúng nhưng cầu nguyện cho người thân đã qua đời."; "Cũng tổ chức lễ giỗ tại tư gia, tại nhà thờ." [N10] **không** nhắc nến, hoa, hương hay bàn thờ — phần "nến và hoa" của 01 §3.7 **chưa có nguồn** (§5) |

Mâm truyền thống dùng **ảnh chụp thật** ghép lại (YC-2026-09, docs/v0.2/09 §3), không hình vẽ, không ảnh do máy tạo.
Hình vẽ `mam-truyen-thong.svg` chỉ là bản tạm; hình vẽ không vẽ món mặn (dễ thô ở cỡ nhỏ). Sổ giấy phép ảnh:
`SOURCES.md` §1b.

Hình vẽ bàn thờ, bia, nến, đĩa quả, bình hoa, hương và khói là **hình vẽ mới cho dự án**. Bản đang chạy viết thẳng bằng
SVG/CSS trong `pwa/src/people/memorial.ts`; bản mẫu ba kiểu mâm ở `content/memorial/mam-*.svg` (`SOURCES.md`).

## 4. Không đưa vào ứng dụng (quyết định, không phải thiếu nguồn)

- Từ Hán-Việt trên bia ("Hiển khảo", "Hiển tỷ"…), chữ "Tiểu tường", "Đại tường", "Kiết kỵ" trong nhãn — theo quyết
  định dùng từ thường ngày. Các từ này chỉ xuất hiện trong biên bản để ghi nguồn.
- Bài khấn, văn khấn — ngoài phạm vi v0.2 (07 §3).
- Vàng mã, tiền giấy, hình Phật, thánh giá, bùa chú — không vẽ (docs/v0.2/09 §5).

## 5. Chưa kiểm chứng — cần nguồn trước khi chấm R9

| Mục | Tình trạng |
|---|---|
| Tháng nhuận → giỗ tháng thường (O-4) | Không tìm được nguồn đọc được. Một bài trên Tạp chí Nghiên cứu và Phát triển (Lê Công Lý, 2025) bàn ngày giỗ âm lịch, nhưng chỉ đọc được tóm tắt — tóm tắt không nói tháng nhuận |
| Ngày 30 → 29 | Không tìm được nguồn |
| Ngưỡng 60 tuổi cho hưởng dương / hưởng thọ (O-3) | Không tìm được nguồn |
| Tuổi âm (tuổi mụ) khi ghi hưởng thọ | Không có nguồn phong tục trong biên bản |
| Giỗ hết vào năm thứ 3 (lựa chọn phụ của O-2) | [N4] nói năm thứ 2; [N9] chỉ nói "sau ba năm" hòa vào tổ tiên. Chưa có nguồn nói giỗ hết năm thứ 3 |
| Công giáo dùng nến và hoa | [N10] không nhắc. Cần một nguồn của Giáo hội về cách bày nơi tưởng nhớ |
| "Bé đỏ" cho trẻ đã sinh rồi mất sớm | [N7] chỉ nói thai nhi |
| "Bà Cô Tổ" và "Bà Tổ Cô" là một | [N6] chỉ dùng "Bà Tổ Cô"; chưa đọc được nguồn tiếng Việt dùng "Bà Cô Tổ" |
| Thứ tự trái/phải của bình hoa và đĩa quả | [N2], [N9] không nói. Câu "đông bình tây quả" hay được nhắc trên mạng nhưng chưa đọc được nguồn đáng tin |
| Cách gọi họ hàng cụ thể ngoài [N1] | Khác theo vùng và từng nhà; ứng dụng chỉ gợi ý, không tự suy ra |

## 6. Nguồn (đọc ngày 2026-11-07)

| Mã | Nguồn |
|---|---|
| N1 | Hồng Phượng, "Sơ đồ cách xưng hô trong họ hàng và thứ bậc, vai vế trong gia đình Việt", VOH (Đài Tiếng nói Nhân dân TP.HCM), 31/01/2025. https://voh.com.vn/thuong-thuc/so-do-cach-xung-ho-trong-ho-hang-575769.html |
| N2 | "Bài trí và ý nghĩa của bàn thờ tổ tiên ngày Tết", Báo điện tử VOV, 16/01/2009 (ghi "Theo VHNT Ẩm thực"). https://vov.vn/huong-vi-tet/bai-tri-va-y-nghia-cua-ban-tho-to-tien-ngay-tet |
| N3 | Phương Nga, "Thắp hương ngày Tết thế nào cho đúng?", VOH, 09/02/2024. https://voh.com.vn/thuong-thuc/thap-huong-ngay-tet-514764.html |
| N4 | Thích Thiện Phước, "Ý nghĩa ngày lễ húy kỵ - ngày đám giỗ", phatgiao.org.vn (Giáo hội Phật giáo Việt Nam), 16/09/2019. https://phatgiao.org.vn/y-nghia-ngay-le-huy-ky--ngay-dam-gio-d36866.html |
| N5 | Toan Ánh, "Ngày giỗ họ của người Việt có ý nghĩa gì?" (trích sách *Nếp cũ*, NXB Trẻ), Znews, 15/06/2026. https://znews.vn/ngay-gio-ho-cua-nguoi-viet-co-y-nghia-gi-post1659626.html |
| N6 | Nguyen D.L., Nguyen K.N., Phan Q.A., "The sanctity of goddesses: an insight into the worship of Bà Tổ Cô in Northern Vietnam", *Humanities and Social Sciences Communications* 9, 2 (2022). doi:10.1057/s41599-021-01018-y. https://www.nature.com/articles/s41599-021-01018-y |
| N7 | Thanh Hòa, "Cha mẹ cầu siêu cho những đứa con không kịp chào đời", VnExpress, 07/09/2015. https://vnexpress.net/cha-me-cau-sieu-cho-nhung-dua-con-khong-kip-chao-doi-3275310.html |
| N8 | Thích Tâm Hiệp, "Cúng mặn hay cúng chay?", Báo Pháp luật Việt Nam, 07/08/2021. https://baophapluat.vn/cung-man-hay-cung-chay-post406472.html |
| N9 | "Thờ cúng tổ tiên ở Việt Nam: bàn thờ, nghi lễ và những điều du khách nên biết", vietnamtourism.com, cập nhật 07/08/2026 (không ghi tác giả). https://www.vietnamtourism.com/vi/tho-cung-to-tien-o-viet-nam-ban-tho-nghi-le-va-nhung-ieu-du- |
| N10 | Phêrô Nguyễn Văn Vịnh, "Ăn giỗ", tgpsaigon.net (Tổng Giáo phận Sài Gòn), 14/09/2012. https://tgpsaigon.net/bai-viet/an-gio-36067 |
