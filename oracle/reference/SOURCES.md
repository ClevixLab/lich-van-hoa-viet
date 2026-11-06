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
