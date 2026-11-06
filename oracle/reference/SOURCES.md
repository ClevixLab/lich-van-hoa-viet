# Nguồn công bố ngày Tết Nguyên Đán 2000–2100 (cho tiêu chí L1)

Thu thập: 2026-11-06, bởi A4 (oracle). Mọi ngày trong `tet_published.csv` được chép từ trang nguồn đã tải về
(qua công cụ đọc web), KHÔNG lấy từ tính toán của oracle hay trí nhớ. Cột `source_id` cho biết nguồn; một năm có thể có nhiều dòng (nhiều nguồn).

## Nguồn Việt Nam (lịch theo UTC+7)

| source_id | Nguồn | Năm có ngày Tết | Số năm | Ghi chú |
|---|---|---|---|---|
| VN_LICHVIET | https://lichviet.app/tet-nguyen-dan-2003 (danh sách 2000–2008 trên trang) | 2000–2008 | 9 | |
| VN_XEMLICHAM | https://www.xemlicham.com/am-lich/nam/YYYY | 2009, 2030, 2035, 2044, 2053, 2076, 2078, 2080, 2099, 2100 | 10 | Chỉ một số trang năm truy cập được (danh sách cho phép của công cụ). Trang này có thể dùng thuật toán Hồ Ngọc Đức (giống phương pháp lõi) — không hoàn toàn độc lập với lõi. |
| VN_CALENDARIFIC | https://calendarific.com/holiday/vietnam/tet-new-year | 2010–2028 | 19 | Trang ghi chú ngày có thể thay đổi theo công bố chính thức. |
| VN_WIKIPEDIA_EN_TET_MIRROR | https://wiki.riteme.site/wiki/T%C3%AAt (bản sao en.wikipedia "Tết"; en.wikipedia.org bị chặn trực tiếp nhưng đọc được qua WebFetch với bảng giống hệt) | 2020–2043 | 24 | |
| VN_SAPTET | https://www.saptet.com/lich-tet/ | 2027–2032 | 6 | |
| VN_HND_CALRULES | https://www.informatik.uni-leipzig.de/~duc/amlich/calrules_en.html (Hồ Ngọc Đức) | 2007, 2030, 2053 | 3 | Trang nói: thế kỷ 21 có đúng 3 năm Tết Việt Nam khác Trung Quốc: 17/02/2007 (TQ 18/02), 02/02/2030 (TQ 03/02), 18/02/2053 (TQ 19/02). Đây là kết quả tính toán của tác giả thuật toán lõi, không phải nguồn độc lập với lõi. |
| VN_NHANDAN | https://nhandan.vn/chung-quanh-thong-tin-ve-ngay-am-lich-khac-nhau-tren-mot-so-loai-lich-post489744.html | 2007 | 1 | Báo Nhân Dân: Tết 2007 VN 17/2, TQ 18/2; năm 2030 và 2053 Tết VN sớm hơn TQ 1 ngày; nhắc Ban Lịch Nhà nước là cơ quan quản lý lịch. |
| VN_BAOLAMDONG | https://baolamdong.vn/am-duong-lich-ngay-18-thang-2-nam-2053.html | 2053 | 1 | 18/02/2053 = mùng 1 tháng Giêng Quý Dậu. |
| VN_NGAYGIO24 | https://ngaygio24.com/xem-ngay-5-2-2000.html | 2000 | 1 | |

Hợp các nguồn Việt Nam phủ **51 năm**: 2000–2044 (liên tục, 45 năm), 2053, 2076, 2078, 2080, 2099, 2100.

**Năm KHÔNG có nguồn Việt Nam truy cập được (50 năm):** 2045–2052, 2054–2075, 2077, 2079, 2081–2098.

## Nguồn Trung Quốc (lịch theo UTC+8) — đối chứng

| source_id | Nguồn | Năm | Số năm |
|---|---|---|---|
| HKO_CN | Đài Thiên văn Hồng Kông (Hong Kong Observatory), bảng chuyển đổi lịch dương–âm: https://www.hko.gov.hk/en/gts/time/calendar/text/files/TYYYYe.txt | 2000–2100 | 101 |

Đây là lịch Trung Quốc (UTC+8), KHÔNG phải lịch Việt Nam. Dùng làm đối chứng: theo VN_HND_CALRULES và VN_NHANDAN,
Tết Việt Nam khác Trung Quốc trong thế kỷ 21 chỉ ở 2007, 2030, 2053 (tuyên bố này dựa trên tính toán, không phải danh sách quan sát).

## Mâu thuẫn giữa các nguồn

| Năm | Nguồn VN | HKO (TQ) |
|---|---|---|
| 2007 | 2007-02-17 (VN_LICHVIET, VN_HND_CALRULES, VN_NHANDAN) | 2007-02-18 |
| 2030 | 2030-02-02 (VN_WIKIPEDIA, VN_SAPTET, VN_XEMLICHAM, VN_HND_CALRULES) | 2030-02-03 |
| 2053 | 2053-02-18 (VN_XEMLICHAM, VN_BAOLAMDONG, VN_HND_CALRULES) | 2053-02-19 |

Đây là khác biệt múi giờ đã biết, không phải lỗi nguồn. Ngoài 3 năm này, không có mâu thuẫn nào giữa các nguồn (kể cả giữa các nguồn VN với nhau).

## Không truy cập được

- vi.wikipedia.org "Tết Nguyên Đán" (lỗi url_not_found / không nằm trong danh sách cho phép), wikiwand vi.
- Trang chủ amlich của Hồ Ngọc Đức (không nằm trong danh sách cho phép); thuvienphapluat lich-am-duong/YYYY (lỗi phía trang); baolamdong trang năm 2100 (lỗi).
- Tải trực tiếp bằng curl bị proxy chặn (403), nên mọi giá trị được đọc từng trang qua WebFetch.
- Không tìm được bảng công bố của Ban Lịch Nhà nước / Viện Vật lý cho toàn bộ 2000–2100.

## Đối chiếu oracle (chỉ để thông tin; không dùng để điền chỗ trống)

oracle/out/tet.csv khớp mọi dòng nguồn Việt Nam (51 năm); khác HKO đúng ở 2007, 2030, 2053, và theo đúng hướng nguồn VN.

---

# Nguồn công bố ngày Tết Nguyên Đán 1900–1999 (cho tiêu chí L9, Bổ sung A1)

Thu thập: 2026-11-06, bởi bên kiểm định (oracle). Dữ liệu: `tet_published_1900_1999.csv` (125 dòng).
Mọi ngày được chép từ trang nguồn đã đọc qua công cụ đọc web (WebFetch); KHÔNG lấy từ tính toán của oracle
hay trí nhớ. Một năm có thể có nhiều dòng (nhiều nguồn). Tải trực tiếp (curl) bị proxy chặn (403).

## Nguồn Việt Nam

| source_id | Nguồn | Năm | Số năm | Ghi chú |
|---|---|---|---|---|
| VN_XEMLICHAM | https://www.xemlicham.com/am-lich/nam/YYYY (ô "1/1" trong bảng tháng); 1914 lấy từ trang ngày .../1914/thang/1/ngay/14; 1916 xác nhận thêm từ .../1916/thang/2/ngay/1; 1925 có thêm trang ngày .../1925/thang/1/ngay/24 | 1900–1999 (đủ 100 năm) | 100 | Trang KHÔNG nêu phương pháp/múi giờ. Có dấu hiệu là kết quả tính toán (không phải bản chụp lịch in): năm 1925 trang năm ghi 25/01 nhưng trang ngày 24/01/1925 có tiêu đề "Ngày Âm Lịch: 1-1-1925" (tự mâu thuẫn). |
| VN_CHOGIA | https://chogia.vn/lich-am/sinh-nam/YYYY/ ("Năm … bắt đầu từ DD/MM/YYYY (mùng 1 Tết)") | 1900, 1903, 1914, 1916, 1925, 1935, 1950, 1965, 1968, 1969 | 10 | Chỉ lấy các năm phân biệt quy ước múi giờ + một số năm kiểm tra. Không nêu phương pháp. Kết quả khớp đúng phép tính UTC+7 suốt (không theo UTC+8 trước 1968). |
| VN_WIKIPEDIA_VI_COPY | https://www.namkyluctinh.org/a-vh-vminh/tet-wiki.pdf (bản in bài vi.wikipedia "Tết Nguyên đán", chân trang "Nguồn: http://vi.wikipedia.org") | 1968 (miền Bắc 29/01), 1996–1999 | 5 | Văn bản: "Trước năm 1967, Việt Nam lấy giờ Bắc Kinh làm chuẩn cho âm lịch. Ngày 8 tháng 8 năm 1967 … đổi lịch dùng múi giờ GMT +7 … miền bắc ngày 29 tháng 1 trong khi miền nam thì ngày 30 tháng 1". vi.wikipedia.org trực tiếp không truy cập được. |
| VN_BAOLAMDONG | https://baolamdong.vn/am-duong-lich-nam-1954.html | 1954 | 1 | "3 1/1 … Mồng 1 Tết Nguyên Đán". Trang các năm khác lỗi phía trang. |

Hợp các nguồn VN phủ **100/100 năm** 1900–1999 (mọi năm có ít nhất xemlicham).

**Mâu thuẫn giữa các nguồn VN (4 năm):**

| Năm | xemlicham | chogia | Ghi chú |
|---|---|---|---|
| 1903 | 1903-01-29 | 1903-01-28 | Sóc 1903-01-28 23:38 UTC+7 / 00:38 UTC+8 (oracle) — phụ thuộc múi giờ |
| 1925 | 1925-01-25 (trang năm) / 1925-01-24 (tiêu đề trang ngày) | 1925-01-24 | Sóc 1925-01-24 22:45 UTC+8 (oracle) — không múi giờ 7–8 nào cho 25/01; trang năm xemlicham nghi lỗi dữ liệu |
| 1935 | 1935-02-04 | 1935-02-03 | Sóc 1935-02-03 23:27 UTC+7 / 00:27 UTC+8 |
| 1965 | 1965-02-02 | 1965-02-01 | Sóc 1965-02-01 23:36 UTC+7 / 00:36 UTC+8 |

Hai trang VN tra cứu phổ biến dùng HAI quy ước khác nhau cho trước 1968 (chogia ≈ UTC+7 suốt; xemlicham ≈ lịch
Trung Quốc: UTC+8, riêng 1916 theo kinh tuyến Bắc Kinh). Không trang nào là bảng chính thức của Ban Lịch Nhà nước.

## Nguồn hỗ trợ (KHÔNG phải nguồn Việt Nam — chỉ dùng làm bằng chứng phụ)

| source_id | Nguồn | Năm | Ghi chú |
|---|---|---|---|
| HKO_CN_SUPPORT | Đài Thiên văn Hồng Kông: .../calendar/pdf/files/1916e.pdf, .../text/files/T1917e.txt, .../pdf/files/1965e.pdf, .../pdf/files/1967e.pdf, https://www.hko.gov.hk/tc/gts/time/calendar/text/files/T1968c.txt | 1916 (02-03), 1917 (01-23), 1965 (02-02), 1967 (02-09), 1968 (01-30) | Lịch Trung Quốc. 1916-02-03 ≠ UTC+8 thuần (02-04): lịch TQ trước 1929 dùng giờ địa phương Bắc Kinh (~UTC+7:46). Bảng PDF 1967 bị vỡ bố cục; đọc gần đúng. |
| SUPPORT_SADPONY_BLOG | https://sad-pony-mathematician.blogspot.com/2019/02/an-off-by-one-error-in-tet-offensive.html | 1968 (Bắc 01-29, Nam 01-30) | Blog; Sóc 1968-01-29 16:31 UT. |

## Bằng chứng về quy ước múi giờ (dùng để đặt quy tắc oracle)

- tz@iana.org (Brian Inglis, 2014, theo "Lịch Việt Nam thế kỷ XX–XXI", Trần Tiến Bình, Ban Lịch Nhà nước):
  "Decision 121/CP dated 8.8.1967 confirmed the country's official time is seventh time zone … from 0 hours on 1.1.1968".
  https://lists.iana.org/hyperkitty/list/tz@iana.org/message/LP343ZE3RZSV2V3IBP3RBODT4ZAXMIRX/
- Cùng nguồn: giờ hành chính miền Bắc 1945–1967 phần lớn là UTC+7 (Hà Nội UTC+7 từ 10/1954) — tức giờ đồng hồ
  KHÁC múi giờ dùng tính âm lịch (UTC+8, theo lịch Trung Quốc) trước 1968.
- en.wikipedia "Vietnamese calendar" (bản sao wiki.riteme.site): "On 8 August 1967, the North Vietnamese government issued a decree to change Vietnamese standard time from UTC+8 to UTC+7"; Hiệp Kỷ 1841–1954 khác lịch Thời Hiến do kinh độ.
- en.wikipedia "Time in Vietnam" (bản sao wiki.krisyotam.com): Bắc Việt "1 January 1968 – 12 June 1975: UTC+07:00 … confirmed as official".
- vnexpress "Vì sao…"/RFA 2007 (Lê Thành Lân, Trịnh Tiến Điều): "Việc tính lịch là từ năm 67"; "Bắt đầu từ 68 chúng ta ăn tết khác Trung Quốc một ngày"; "Cuối 68 đầu 69 cũng chênh nhau một ngày"; năm 84/85 khác một tháng.
  https://www.rfa.org/vietnamese/in_depth/WhyVietnameseLunarCalendarDiffersFromChina_GMinh-20070207.html
- suikawiki (tổng hợp nhiều nguồn): VNCal 1301–1945 theo Hoàng Xuân Hãn, 1946 trở đi tính toán; "1967年以降は…UTC+07:00で計算したものと非常によく一致".
  https://wiki.suikawiki.org/i/25985
- Không truy cập được văn bản gốc Quyết định 121-CP (thuvienphapluat lỗi phía trang).

## Đối chiếu oracle (chỉ để thông tin; không dùng để điền chỗ trống)

Oracle (UTC+8 trước 1968-01-01 00:00 UTC+7) khớp xemlicham 97/100 năm (lệch 1916 — quy ước kinh tuyến Bắc Kinh —
và 1925 — mâu thuẫn nội bộ của xemlicham); khớp chogia 6/10 (lệch 1903, 1916, 1935, 1965 — chogia theo UTC+7 suốt).
