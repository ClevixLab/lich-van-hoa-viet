# Hướng dẫn sử dụng — Lịch và Văn hóa Việt (bản 0.1 thử nghiệm)

Lịch và Văn hóa Việt là lịch âm dương Việt Nam cho điện thoại và máy tính. Mỗi ngày có một tờ lịch, cho
biết ngày âm, Can Chi, tiết khí và giờ hoàng đạo, kèm một câu ca dao, tục ngữ. Ứng dụng có thêm mục chọn
ngày tốt, mục đổi ngày âm ↔ dương và (từ bản 0.2) mục **Người thân** để nhớ ngày giỗ, sinh nhật của gia
đình (mục 5). Ứng dụng miễn phí, không cần đăng ký, không có quảng cáo, và **chạy
được cả khi không có mạng** sau lần mở đầu tiên.

Bản 0.1 là **ứng dụng web cài được** (PWA). Bạn mở bằng trình duyệt rồi "cài" như một ứng dụng. Các bản trên
cửa hàng ứng dụng (Google Play, App Store, Microsoft Store) sẽ có sau.

> Địa chỉ trang: do Clevix Lab thông báo khi phát hành (dự kiến `https://lich.clevix.vn`).

## 1. Cài lên máy

Mở địa chỉ trang **một lần khi có mạng**. Sau khi cài, ứng dụng có biểu tượng riêng và mở được khi không
có mạng.

### Điện thoại Android (Chrome)
1. Mở trang bằng **Chrome**.
2. Bấm nút **⋮** (góc trên bên phải) → **Thêm vào màn hình chính** (hoặc **Cài đặt ứng dụng**).
3. Bấm **Cài đặt**. Biểu tượng "Lịch Việt" xuất hiện trên màn hình chính.

### iPhone, iPad (Safari)
1. Mở trang bằng **Safari**. Trình duyệt khác trên iPhone có thể không có mục này.
2. Bấm nút **Chia sẻ** (ô vuông có mũi tên lên) ở thanh dưới.
3. Kéo xuống, chọn **Thêm vào MH chính** (Add to Home Screen) → **Thêm**.

### Máy tính Windows hoặc Mac (Chrome hoặc Edge)
1. Mở trang bằng **Chrome** hoặc **Microsoft Edge**.
2. Bấm biểu tượng **cài đặt** ở cuối thanh địa chỉ (màn hình có mũi tên xuống), hoặc vào menu **⋮** /
   **…** → **Cài đặt Lịch và Văn hóa Việt** (Edge: **Ứng dụng → Cài đặt trang này như một ứng dụng**).
3. Ứng dụng mở trong cửa sổ riêng và có trong menu Start (Windows) hoặc Launchpad (Mac).

Trên Mac dùng **Safari** (macOS 14 trở lên): menu **Tệp → Thêm vào Dock**.

Không muốn cài cũng được: chỉ cần mở trang trong trình duyệt là dùng được ngay.

**Gỡ cài đặt:** gỡ như mọi ứng dụng (nhấn giữ biểu tượng → Gỡ). Trên máy tính, mở ứng dụng → menu **⋮** →
**Gỡ cài đặt**.

**Cập nhật:** xem mục 4 "Xem phiên bản và cập nhật".

### Ứng dụng cài từ cửa hàng hoặc tệp cài (Android, iPhone/iPad, Mac, Windows)

Các bản ứng dụng riêng cho từng hệ điều hành đang thử nghiệm; khi phát hành sẽ có trên cửa hàng ứng dụng
hoặc ở trang tải về https://clevixlab.github.io/lich-van-hoa-viet/ .

- **Xem phiên bản đang dùng:** mở mục **Giới thiệu**. Dòng "Phiên bản đang dùng: 0.1.2"
  cho biết số phiên bản; cuối trang có hai dòng "© 2026 Clevix Lab" và "Phiên bản …".
- **Cập nhật:** trong **Giới thiệu**, bấm **Kiểm tra cập nhật / Cập nhật**. Ứng dụng không tự kết nối
  mạng để bảo vệ quyền riêng tư, nên nút này mở cửa hàng ứng dụng (Google Play trên Android) hoặc trang
  tải về trong trình duyệt. Ở đó, nếu có bản mới hơn số phiên bản của bạn, bấm **Cập nhật** hoặc tải bản
  mới về cài đè.
- **Nhắc cập nhật:** khi bản đang dùng đã phát hành hơn 2 tháng, đầu màn hình Tờ lịch hiện một thẻ nhắc.
  Bấm **Kiểm tra cập nhật** để mở cửa hàng/trang tải về, hoặc **Để sau** để ẩn thẻ trong 30 ngày. Thẻ
  không chặn việc dùng lịch. Nếu cửa hàng không có bản mới hơn thì bạn đang dùng bản mới nhất.
- Cài từ Google Play: Play cũng tự cập nhật ứng dụng nếu bạn bật tự động cập nhật trong Google Play.

## 2. Các màn hình

Thanh dưới cùng có bốn mục: **Tờ lịch**, **Chọn ngày tốt**, **Đổi ngày**, **Giới thiệu**. Từ bản 0.2 có
thêm mục thứ năm **Người thân**, nằm giữa "Đổi ngày" và "Giới thiệu" (xem mục 5).

### Tờ lịch
- Ngày dương lịch in to ở giữa. Bên dưới là **ngày âm lịch** và tên tháng âm (tháng đủ 30 ngày hay thiếu 29
  ngày), Can Chi của năm, tháng, ngày, và **tiết khí**.
- **Hoàng đạo / Hắc đạo**, **Trực**, **Đánh giá** (Tốt, Khá tốt, Bình thường, Xấu), **giờ hoàng đạo**, và
  các mục **Nên làm** / **Kỵ**.
- Bấm **Chi tiết** để xem sao tốt, sao xấu, Nhị thập bát tú, Nạp âm, tuổi xung, điểm, và liên kết **Nguồn
  quy tắc**.
- Cuối tờ lịch là **câu ca dao, tục ngữ** của ngày.
- **Lật ngày:** vuốt sang trái hoặc phải (điện thoại); bấm nút **‹ ›**; hoặc phím **← →** (máy tính). Nút
  **Hôm nay** (hoặc phím **T**) đưa về ngày hiện tại.
- **Chọn ngày bất kỳ:** bấm vào dòng tháng/năm (biểu tượng lịch) để mở **bảng tháng**. Chọn tháng, gõ năm,
  rồi chạm vào ngày muốn xem.
- **Cỡ chữ:** các nút **A / A+ / A++** ở trên cùng. **Sáng/tối:** nút hình tròn nửa đen nửa trắng.

### Chọn ngày tốt
1. Chọn **việc** (cưới hỏi, khai trương, động thổ, xuất hành, …).
2. Chọn **khoảng thời gian**: Tuần này, Tháng này, 3 tháng tới, hoặc Tự chọn (từ ngày … đến ngày …).
3. (Không bắt buộc) Nhập **ngày sinh dương lịch** để loại các ngày xung tuổi. Ứng dụng tự tính tuổi âm
   lịch; người sinh trước Tết thuộc năm âm trước. Nếu chỉ biết năm âm lịch, bấm "Tôi chỉ biết năm âm lịch /
   Can Chi".
4. Danh sách hiện các ngày đề xuất, xếp theo **điểm cho việc đã chọn**, kèm lý do. Chạm vào một ngày để mở
   tờ lịch của ngày đó.

Ngày sinh chỉ dùng trong lúc tính và **không được lưu**.

### Đổi ngày
- **Dương lịch sang âm lịch:** chọn ngày dương, ứng dụng cho ngày âm và Can Chi.
- **Âm lịch sang dương lịch:** nhập năm âm, chọn tháng, ngày. Đánh dấu **Tháng nhuận** nếu là tháng nhuận.
- Bấm **Mở tờ lịch ngày này** để xem chi tiết.
- Phạm vi: năm 1900 đến năm 3000.

### Giới thiệu
Ở đây có số phiên bản và nút kiểm tra cập nhật, thông tin về ứng dụng (cách tính ngày âm), quyền riêng tư,
nguồn quy tắc xem ngày, nguồn ca dao (ghi công theo giấy phép CC BY-SA 4.0) và phông chữ.

## 3. Cần biết

- **Phần xem ngày** (sao, trực, hoàng đạo, nên/kỵ, điểm) theo quan niệm truyền thống trong *Ngọc Hạp Thông
  Thư*. Đây là nét văn hóa dân gian, **chỉ để tham khảo**. Điểm số là quy ước của ứng dụng, không có trong
  sách.
- Từ bản 0.1.4, ngày âm lịch tính từ **lịch thiên văn JPL DE441 của NASA JPL** (thời điểm trăng mới và tiết
  khí), với ΔT (chênh lệch giữa giờ thiên văn và giờ đồng hồ) theo Morrison và Stephenson. Từ 1968 lịch tính theo
  giờ Việt Nam (UTC+7); trước đó theo quy ước lịch sử (xem mục Giới thiệu).
- Với những năm rất xa, kết quả kém chắc chắn hơn vì không ai dự báo chính xác được tốc độ quay của Trái Đất
  (ΔT). Rất hiếm khi trăng mới hoặc tiết khí rơi sát nửa đêm; khi đó các lịch khác nhau có thể lệch nhau một
  ngày. Sau năm 2077 có một số ngày như vậy so với cách tính cũ của ứng dụng (báo cáo `docs/reports/BAO-CAO-JPL-L16.md`).
- **Quyền riêng tư:** ứng dụng không thu thập hay gửi dữ liệu nào. Máy chỉ ghi nhớ cỡ chữ và chế độ
  sáng/tối bạn chọn, và — nếu bạn dùng mục Người thân — sổ người thân của bạn, **chỉ trên máy này** (mục 5.8). Máy chủ lưu trang có thể ghi nhật ký truy cập thông thường như mọi trang web.
- **Pin và dung lượng:** ứng dụng chiếm khoảng 1 MB trên máy và không chạy ngầm khi bạn không mở nó.

## 4. Xem phiên bản và cập nhật

- **Xem phiên bản:** số phiên bản ghi ở cuối mỗi trang ("© 2026 Clevix Lab", dòng dưới "Phiên bản 0.1.2").
  Trong mục **Giới thiệu**, dòng "Phiên bản đang dùng: 0.1.2" chỉ ghi số phiên bản — khi góp ý,
  bạn chép dòng này để Clevix Lab biết bạn dùng bản nào.
- **Tự nhắc:** khi bạn mở ứng dụng (lúc có mạng), ứng dụng tự xem có phiên bản mới không. Nếu có, đầu màn
  hình hiện dòng "Đã có phiên bản mới X.Y.Z." với hai nút:
  - **Cập nhật ngay:** ứng dụng tải lại một lần và chuyển sang phiên bản mới. Không mất gì: cỡ chữ và giao
    diện sáng/tối vẫn giữ.
  - **Để sau:** ẩn dòng nhắc; bạn vẫn dùng bản cũ bình thường. Ứng dụng sẽ nhắc lại sau (nhiều nhất 6 giờ
    một lần), hoặc tự dùng bản mới khi bạn đóng hẳn ứng dụng rồi mở lại.
- **Tự kiểm tra:** vào **Giới thiệu** → bấm **Kiểm tra cập nhật**. Kết quả là một trong ba dòng:
  "Bạn đang dùng phiên bản mới nhất."; "Có phiên bản mới X.Y.Z" kèm danh sách điểm mới và nút **Cập nhật
  ngay**; hoặc "Không kiểm tra được (đang ngoại tuyến)." — khi đó hãy bật mạng rồi bấm lại.
- Việc kiểm tra chỉ đọc một tệp nhỏ trên trang của ứng dụng; không gửi thông tin gì của bạn đi.
- **Nếu đang dùng bản 0.1** (bản đầu tiên): bản này chưa có các nút trên. Khi có bản mới, cuối màn hình hiện
  dòng "Có bản mới của ứng dụng." kèm nút **Tải lại** — bấm để lên bản mới. Không bấm thì bản mới được dùng
  khi bạn đóng hẳn ứng dụng rồi mở lại.

## 5. Người thân — ngày giỗ, sinh nhật, sự kiện (bản 0.2)

Mục **Người thân** là cuốn sổ nhỏ của gia đình: ghi ông bà, cha mẹ đã khuất để được nhắc **ngày giỗ theo âm
lịch**, và ghi người thân đang sống để nhớ **sinh nhật, ngày cưới, ngày tốt nghiệp**. Không cần tài khoản, không cần
mạng. Mọi thứ chỉ nằm trên máy của bạn.

Trong mục có bốn trang, chọn ở hàng nút phía trên:
**Sắp tới** · **Theo kỳ** · **Hồ sơ** · **Sao lưu**.

Lần đầu mở, trang còn trống và có hai nút **+ Thêm người đã mất**, **+ Thêm người đang sống**
(ảnh `v02-empty-360.png`).

> Tên các ảnh trong mục này là ảnh chụp màn hình do kiểm thử tự động tạo ra, nằm trong thư mục `pwa/test/shots/`
> của dự án. Ảnh chụp bản web; bản cài trên điện thoại và máy tính trông gần giống.

### 5.1 Thêm người đã mất

Bấm **+ Thêm người đã mất** (ảnh `v02-form-deceased-360.png`).

1. **Họ và tên** là ô duy nhất bắt buộc, cùng với ngày mất. Các ô khác có thể để trống, điền sau.
2. **Cách gọi (đứng trước tên)** và **Quan hệ với bạn**: gõ hoặc chọn trong danh sách gợi ý, ví dụ Ông nội, Bà
   ngoại, Bố, Mẹ, Cụ ông, Kỵ bà, Bác, Chú, Cô, Dì, Cậu, Mợ, Thím, Ông Tổ họ, Bà Cô Tổ họ, Bé đỏ. Ứng dụng tự viết hoa
   chữ đầu. Miền Bắc gọi "Cụ", "Kỵ"; miền Nam gọi "Cố", "Sơ" — danh sách có cả hai, bạn chọn theo nhà mình.
   Chữ ở ô **Quan hệ** được ghi phía trên tên trên trang tưởng niệm (mục 5.7).
3. **Ngày mất** — chọn một trong hai cách:
   - **Nhập theo dương lịch** (ngày trên giấy tờ): gõ ngày, tháng, năm. Ngay bên dưới hiện ngày âm tương ứng, ví dụ
     "Âm lịch: 15/6 nhuận năm Ất Tỵ (2025)". Ngày giỗ được tính từ ngày âm này.
   - **Nhập theo âm lịch** (khi gia đình chỉ nhớ ngày âm): gõ ngày, tháng, năm âm. Nếu năm đó có **tháng nhuận**, hiện
     ô **Tháng nhuận** để đánh dấu; năm không có tháng nhuận thì ô này không hiện. Không nhớ năm mất thì đánh dấu
     **Không rõ năm mất** — ứng dụng vẫn nhắc giỗ hằng năm.
   - Gõ sai (ví dụ ngày 30 của tháng chỉ có 29 ngày), ứng dụng báo ngay bằng lời, ví dụ "Tháng 7 âm lịch năm 2026
     chỉ có 29 ngày".
4. **Ngày sinh**, **nơi sinh**, **nơi an táng**, **ảnh chân dung** (không bắt buộc). Có năm sinh thì ứng dụng tự ghi
   "Hưởng thọ … tuổi" (hoặc "Hưởng dương" nếu mất sớm). Ảnh được thu nhỏ và bỏ thông tin vị trí, máy chụp.
5. **Nhắc ngày giỗ**: mặc định nhắc **đúng ngày** và **3 ngày trước**, lúc **07:00**. Bạn bấm chọn thêm 1, 5, 10, 15
   ngày hoặc tự đặt số ngày. Muốn được nhắc cả **ngày tiên thường** (lễ cúng hôm trước ngày giỗ) thì đánh dấu ô đó.
6. Bấm **Lưu**. Trang hồ sơ của người đó mở ra (ảnh `v02-person-deceased-360.png`), ghi lần giỗ tới, ví dụ
   "Giỗ đầu 28/07/2026 · còn 263 ngày".

Ứng dụng tự ghi **giỗ đầu** (năm thứ nhất sau năm mất), **giỗ hết** (mặc định năm thứ hai; đổi được thành năm thứ ba
trong phần "Tưởng niệm" của hồ sơ) và từ đó về sau là giỗ thường.

**Với người đã mất, ứng dụng chỉ nhắc ngày giỗ.** Sinh nhật và sự kiện cũ của người đó vẫn được giữ trong hồ sơ nhưng
không nhắc nữa (ảnh `v02-person-deceased-b4-360.png`).

### 5.2 Thêm người đang sống

Bấm **+ Thêm người đang sống** (ảnh `v02-form-living-360.png`). Điền họ tên, quan hệ, **ngày sinh** (dương lịch hoặc
âm lịch, hoặc chỉ năm). Ứng dụng cho biết tuổi, ví dụ "Hôm nay 36 tuổi (tuổi âm 38)".

- Muốn nhớ cả **sinh nhật âm lịch**, đánh dấu "Nhắc cả sinh nhật âm lịch hằng năm".
- Người sinh ngày **29/02**: chọn mừng sinh nhật ngày 28/02 hay 01/03 ở năm không nhuận.
- **Sự kiện** (tốt nghiệp, ngày cưới, đính hôn hoặc loại bạn tự đặt): lưu hồ sơ trước, rồi trên trang hồ sơ bấm
  **+ Thêm sự kiện** (ảnh `v02-form-event-360.png`). Sự kiện có thể lặp hằng năm hoặc chỉ một lần.

### 5.3 Khi người thân qua đời: "Cập nhật: đã mất"

Bạn **không cần** tạo hồ sơ mới. Mở hồ sơ của người đó, bấm **Cập nhật: đã mất** (ảnh `v02-convert-form-360.png`).

1. Nhập **ngày mất** (dương lịch hoặc âm lịch, như mục 5.1).
2. Phần **Thông tin báo tử** (không bắt buộc): người báo tin, mất tại đâu, ngày nhận tin, lý do, ghi chú, và có thể
   đính kèm **ảnh tin báo tử** hoặc **ảnh / bản quét PDF giấy chứng tử**. Ảnh được bỏ thông tin vị trí và máy chụp;
   tệp PDF được lưu nguyên bản.
3. Bấm **Lưu**. Ứng dụng hỏi lại và nói rõ điều gì sẽ thay đổi (ảnh `v02-convert-confirm-360.png`): thôi nhắc sinh
   nhật và sự kiện, bắt đầu nhắc ngày giỗ; tên, ảnh, ngày sinh vẫn giữ nguyên. Bấm **Lưu thay đổi**.

**Thông tin báo tử là riêng tư.** Nó chỉ hiện trong khối "Thông tin báo tử" trên trang hồ sơ của người đó (ảnh
`v02-person-notice-360.png`). Nó **không** hiện trong lời nhắc, trang Sắp tới, Theo kỳ, tệp lịch .ics hay trang
tưởng niệm. Nó **có** trong tệp sao lưu, và tệp sao lưu được khóa bằng mật khẩu (mục 5.6).

### 5.4 Xem theo tuần, tháng, quý, năm

Trang **Theo kỳ** (ảnh `v02-period-month-360.png`) liệt kê mọi ngày giỗ, sinh nhật, sự kiện trong một khoảng:

- Chọn **Tuần** (Thứ Hai đến Chủ nhật), **Tháng**, **Quý** hoặc **Năm** — theo dương lịch; mỗi dòng ghi kèm ngày âm.
- Bấm **‹** / **›** để lùi, tới một kỳ; bấm **Hôm nay** để về kỳ hiện tại.
- Lọc: **Tất cả**, **Người đã mất** (chỉ ngày giỗ, và tiên thường nếu bạn bật), **Người đang sống** (sinh nhật và sự
  kiện) — ảnh `v02-period-deceased-filter-360.png`.
- Xem theo quý hoặc năm, danh sách chia theo từng tháng cho dễ đọc. Mỗi dòng ghi "Còn n ngày" hoặc "Đã qua n ngày".

### 5.5 Sắp tới và lời nhắc

Trang **Sắp tới** (ảnh `v02-upcoming-360.png`) là trang đầu của mục Người thân: mọi việc trong **30 ngày tới**. Vào
đúng ngày cần nhắc, đầu trang Tờ lịch và trang Sắp tới có **thẻ nhắc** (ảnh `v02-reminder-card-360.png`); hôm nay là
ngày giỗ thì thẻ có nút **Mở trang tưởng niệm**.

Lời nhắc trên từng loại máy:

| Bạn dùng | Lời nhắc đến thế nào |
|---|---|
| **Bản web** (mở bằng trình duyệt hoặc đã "cài" lên màn hình chính) | **Không gửi được thông báo khi ứng dụng đang đóng.** Lời nhắc chỉ hiện khi bạn mở ứng dụng vào ngày đó. Muốn điện thoại tự nhắc, hãy **xuất tệp lịch .ics** (dưới đây) |
| **Ứng dụng Android** (khi phát hành) | Thông báo trên điện thoại, kể cả khi ứng dụng đóng và sau khi tắt mở lại máy. Lần đầu bật nhắc, máy hỏi quyền gửi thông báo — hãy bấm **Cho phép**. Thông báo có thể đến muộn tới khoảng một giờ. Một số máy (Xiaomi, Oppo, Samsung…) chặn ứng dụng chạy ngầm để tiết kiệm pin: nếu không thấy nhắc, hãy cho ứng dụng "không hạn chế pin" trong Cài đặt |
| **iPhone, iPad, Mac** (khi phát hành) | Thông báo của máy. Máy chỉ giữ được một số lời nhắc đặt trước; nếu lâu không mở ứng dụng, bạn sẽ nhận một thông báo "Mở Lịch Việt để tiếp tục nhận lời nhắc giỗ và sinh nhật" — chỉ cần mở ứng dụng một lần |
| **Windows** (khi phát hành) | Thông báo của Windows; cách hoạt động khi ứng dụng đóng sẽ được ghi rõ khi phát hành |

Để giữ riêng tư, thông báo trên **màn hình khóa** của bản cài mặc định **không hiện tên** người, chỉ ghi "Sắp đến
ngày giỗ", "Sắp đến sinh nhật".

**Xuất tệp lịch (.ics) — cho bản web.** Vào **Sao lưu** → **Xuất tệp lịch (.ics)…** (ảnh `v02-ics-dialog-360.png`).
Hộp xác nhận cho biết tệp có bao nhiêu người và xem trước vài ngày. Bấm **Tải tệp .ics**, rồi mở tệp bằng ứng dụng
Lịch của điện thoại hoặc máy tính (Lịch Google, Lịch của iPhone, Outlook) để nhập. Tệp chứa ngày giỗ, sinh nhật
**trong 10 năm tới**. Lưu ý: **ai có tệp này đều đọc được tên và ngày** — đừng gửi cho người lạ. Khi thêm, sửa người
thân, hãy xuất lại tệp.

### 5.6 Sao lưu và khôi phục

Dữ liệu chỉ nằm trên máy. **Mất máy, đổi máy, hoặc trình duyệt tự dọn dữ liệu là mất sổ người thân** nếu bạn chưa
sao lưu. Ứng dụng sẽ nhắc khi đã lâu chưa sao lưu (mặc định 30 ngày).

**Tạo tệp sao lưu:** vào **Sao lưu** → **Tạo tệp sao lưu…** (ảnh `v02-backup-dialog-360.png`).
1. Đặt **mật khẩu** ít nhất 8 ký tự, gõ lại lần nữa. Nên dùng một câu dễ nhớ, ví dụ "nhà ở Hà Đông có cây bàng".
2. Bấm **Tạo và tải tệp**. Tệp có tên dạng `LichVanHoaViet-saoluu-2026-11-07.lvhv`.
3. Cất tệp ở nơi an toàn: ổ đám mây (Google Drive, iCloud, OneDrive), USB, hoặc gửi vào email của chính bạn.

> **Quan trọng: quên mật khẩu là mất dữ liệu trong tệp.** Không ai lấy lại được, kể cả Clevix Lab. Hãy ghi mật khẩu
> ở nơi chắc chắn, ví dụ sổ tay trong nhà.

**Tệp sao lưu có gì:** mọi hồ sơ, sự kiện, lời nhắc đã đặt, ảnh chân dung, **thông tin báo tử và tệp đính kèm** (ảnh
`v02-backup-notice-dialog-360.png`). Tất cả đã được mã hóa: không có mật khẩu thì không đọc được.

**Khôi phục** (trên máy mới hoặc sau khi mất dữ liệu): **Sao lưu** → **Chọn tệp sao lưu…** → gõ mật khẩu →
**Mở tệp** (ảnh `v02-import-dialog-360.png`). Chọn:
- **Gộp vào dữ liệu đang có** (nên dùng): giữ cả hai, hồ sơ nào trùng thì giữ bản mới hơn.
- **Thay toàn bộ**: xóa dữ liệu trên máy rồi dùng dữ liệu trong tệp. Ứng dụng hỏi lại trước khi làm, vì không hoàn
  tác được.

Sai mật khẩu, ứng dụng báo "Sai mật khẩu hoặc tệp bị hỏng" và dữ liệu trên máy giữ nguyên.

### 5.7 Trang tưởng niệm

Trên hồ sơ người đã mất, bấm **Tưởng niệm**; vào đúng ngày giỗ có thể mở từ thẻ nhắc (ảnh `v02-memorial-360.png`).
Trang có ảnh chân dung, bia ghi quan hệ (ví dụ "Ông nội"), họ tên, tên hiệu, năm sinh – năm mất, "Hưởng thọ … tuổi",
bàn thờ có ba nén hương với khói nhẹ, và **lời tưởng niệm** do gia đình viết (ô "Lời tưởng niệm" trong phần
"Tưởng niệm" của hồ sơ).

- Trang **không có quảng cáo, không có âm thanh**, không có nút chia sẻ.
- Khói tự dừng sau 2 phút; bấm **Dừng khói** / **Thắp lại** tùy ý. Nếu máy đặt chế độ "giảm chuyển động", khói đứng
  yên.
- Trang không ghi thông tin báo tử. Tên người không hiện trên tiêu đề cửa sổ.
- Bia dùng từ thường ngày theo ô Quan hệ; ứng dụng không dùng các chữ Hán-Việt cổ.

### 5.8 Riêng tư: dữ liệu ở đâu

- Sổ người thân (tên, ngày, ảnh, thông tin báo tử) **chỉ lưu trên máy này**. Ứng dụng không gửi gì lên mạng, không có
  máy chủ, không có tài khoản.
- Dữ liệu chỉ rời máy khi **chính bạn** tạo tệp sao lưu (đã khóa bằng mật khẩu) hoặc xuất tệp lịch .ics (không khóa).
- Địa chỉ trang trong trình duyệt không chứa tên hay ngày của người thân.
- **Bản web:** trình duyệt có thể xóa dữ liệu khi máy thiếu chỗ. Trên iPhone, hãy **thêm ứng dụng vào Màn hình chính**
  (mục 1) để dữ liệu được giữ lâu hơn — và nhớ sao lưu. Xóa dữ liệu trang web trong cài đặt trình duyệt cũng xóa sổ
  người thân.
- Gỡ ứng dụng trên điện thoại sẽ xóa dữ liệu của ứng dụng — hãy sao lưu trước khi gỡ.

### 5.9 Cập nhật ứng dụng không mất dữ liệu

Khi bạn cập nhật lên phiên bản mới (bấm **Cập nhật ngay** ở bản web, cập nhật từ cửa hàng ứng dụng, hoặc cài đè bản
mới trên máy tính), **sổ người thân, ảnh, lời nhắc và cài đặt đều được giữ nguyên** và tự chuyển sang phiên bản mới.
Đây là yêu cầu bắt buộc của mọi bản phát hành.

- Trước khi chuyển, ứng dụng tự giữ một bản sao dữ liệu cũ trên máy. Nếu có trục trặc, ứng dụng quay về dữ liệu cũ,
  mở ở chế độ **chỉ xem** và hướng dẫn bạn tạo tệp sao lưu — dữ liệu không bị xóa.
- Tệp sao lưu tạo từ bản cũ vẫn mở được ở bản mới.
- Trên Windows, gỡ cài đặt không tự xóa thư mục dữ liệu.
- Nếu thấy dòng "Dữ liệu được tạo bởi phiên bản mới hơn — hãy cập nhật ứng dụng": bạn đang mở dữ liệu bằng bản cũ hơn
  (ví dụ trên một máy khác) — hãy cập nhật ứng dụng ở máy đó.

Dù vậy, **sao lưu định kỳ** vẫn là cách chắc chắn nhất để giữ sổ người thân.

### 5.10 Về phong tục

Cách tính giỗ đầu, giỗ hết, tiên thường và cách gọi họ hàng theo **cách phổ biến** của người Việt; mỗi vùng, mỗi nhà
có thể khác. Mọi chữ (cách gọi, quan hệ) bạn đều sửa được. Nguồn tham khảo ghi trong
`content/memorial/PHONG-TUC.md` của dự án.

Góp ý: khanh@clevix.vn — Clevix Lab (Clevix LLC).
