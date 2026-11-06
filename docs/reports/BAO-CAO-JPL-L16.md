# Báo cáo L16 — mọi ngày `Jpl` khác `Precise`, 1900–3000 (docs/02 Bổ sung A4)

Sinh tự động bởi `tools/jpl/l16.py` (không sửa tay). Precise = lõi hiện hành, đã kiểm trùng tệp chuẩn L13 (`core/out/baseline-2000-3000/precise.jsonl.gz`) ở 365608 ngày 2000–3000 (năm/tháng/ngày âm, nhuận, Can Chi năm/tháng/ngày, tiết khí); 1900–1999 lấy từ lõi hiện hành (đã qua L10/L12 với Precise).

Trường so: năm/tháng/ngày âm, nhuận, Can Chi năm/tháng/ngày, tên tiết khí đang hiệu lực (tức ngày bắt đầu tiết khí). Giờ là giờ lịch (§16.1: +07:45:40 trước 1929, +08:00 1929–1967, +07:00 từ 1968). "Cách nửa đêm" = khoảng cách tới 00:00 gần nhất theo giờ lịch.

## Tóm tắt

- Số ngày khác: **1238**; số đoạn: **107**.
- Đoạn "nguyên nhân khác": **0**.
- Sự kiện hai phương pháp đặt khác ngày lịch: 107 (Sóc 39, tiết khí 68); trong đó 0 sự kiện không làm khác ngày nào (Trung khí/Sóc khác ngày nhưng không đổi tháng nhuận, tháng 11, hay ngày đầu tháng).
- Kết luận L16: **ĐẠT** (ngưỡng: mọi đoạn do một sự kiện mà hai phương pháp đặt khác ngày; đoạn "nguyên nhân khác" = KHÔNG ĐẠT).

| Thế kỷ | Số đoạn | Số ngày |
|---|---|---|
| 2000–2099 | 2 | 60 |
| 2100–2199 | 7 | 152 |
| 2200–2299 | 7 | 94 |
| 2300–2399 | 14 | 101 |
| 2400–2499 | 14 | 159 |
| 2500–2599 | 15 | 131 |
| 2600–2699 | 10 | 68 |
| 2700–2799 | 14 | 130 |
| 2800–2899 | 15 | 305 |
| 2900–2999 | 9 | 38 |

## Danh sách đoạn

| # | Từ | Đến | Số ngày | Trường khác | Sự kiện gây khác | Precise (giờ lịch) | Jpl (giờ lịch) | Cách nửa đêm Precise / Jpl | Phân loại |
|---|---|---|---|---|---|---|---|---|---|
| 1 | 2077-11-15 | 2077-12-14 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 2200 | 2077-11-15T23:59:50+07:00 | 2077-11-16T00:00:43+07:00 | 10 s / 43 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 2 | 2085-10-18 | 2085-11-16 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 2298 | 2085-10-18T23:59:32+07:00 | 2085-10-19T00:00:37+07:00 | 28 s / 37 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 3 | 2130-05-07 | 2130-06-05 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 2849 | 2130-05-07T23:58:06+07:00 | 2130-05-08T00:00:09+07:00 | 114 s / 9 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 4 | 2150-05-26 | 2150-06-24 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 3097 | 2150-05-26T23:59:28+07:00 | 2150-05-27T00:01:51+07:00 | 32 s / 111 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 5 | 2159-05-17 | 2159-06-15 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 3208 | 2159-05-17T23:58:09+07:00 | 2159-05-18T00:00:40+07:00 | 111 s / 40 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 6 | 2175-01-22 | 2175-02-20 | 30 | Can Chi năm, Can Chi tháng, ngày âm, năm âm, tháng âm | Sóc 3402 | 2175-01-22T23:59:36+07:00 | 2175-01-23T00:02:20+07:00 | 24 s / 140 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 7 | 2180-10-22 | 2180-10-22 | 1 | tiết khí | Tiết khí Sương Giáng 2180 | 2180-10-22T23:58:28+07:00 | 2180-10-23T00:01:26+07:00 | 92 s / 86 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 8 | 2186-12-21 | 2186-12-21 | 1 | tiết khí | Tiết khí Đông Chí 2186 | 2186-12-21T23:58:55+07:00 | 2186-12-22T00:01:56+07:00 | 65 s / 116 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 9 | 2199-01-26 | 2199-02-24 | 30 | Can Chi năm, Can Chi tháng, ngày âm, năm âm, tháng âm | Sóc 3699 | 2199-01-26T23:57:53+07:00 | 2199-01-27T00:00:52+07:00 | 127 s / 52 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 10 | 2203-11-04 | 2203-12-03 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 3758 | 2203-11-04T23:59:15+07:00 | 2203-11-05T00:02:20+07:00 | 45 s / 140 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 11 | 2217-01-05 | 2217-01-05 | 1 | tiết khí | Tiết khí Tiểu Hàn 2217 | 2217-01-05T23:59:31+07:00 | 2217-01-06T00:02:53+07:00 | 29 s / 173 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 12 | 2217-11-07 | 2217-11-07 | 1 | tiết khí | Tiết khí Lập Đông 2217 | 2217-11-07T23:57:52+07:00 | 2217-11-08T00:01:20+07:00 | 128 s / 80 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 13 | 2240-02-23 | 2240-03-23 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 4207 | 2240-02-23T23:58:06+07:00 | 2240-02-24T00:01:26+07:00 | 114 s / 86 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 14 | 2242-07-28 | 2242-08-26 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 4237 | 2242-07-28T23:59:55+07:00 | 2242-07-29T00:03:24+07:00 | 5 s / 204 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 15 | 2246-05-05 | 2246-05-05 | 1 | tiết khí | Tiết khí Lập Hạ 2246 | 2246-05-05T23:58:14+07:00 | 2246-05-06T00:01:53+07:00 | 106 s / 113 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 16 | 2248-02-04 | 2248-02-04 | 1 | tiết khí | Tiết khí Lập Xuân 2248 | 2248-02-04T23:59:53+07:00 | 2248-02-05T00:03:32+07:00 | 7 s / 212 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 17 | 2301-11-01 | 2301-11-30 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 4970 | 2301-11-01T23:58:47+07:00 | 2301-11-02T00:02:30+07:00 | 73 s / 150 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 18 | 2317-01-20 | 2317-01-20 | 1 | tiết khí | Tiết khí Đại Hàn 2317 | 2317-01-20T23:58:09+07:00 | 2317-01-21T00:02:27+07:00 | 111 s / 147 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 19 | 2322-12-22 | 2322-12-22 | 1 | tiết khí | Tiết khí Đông Chí 2322 | 2322-12-22T23:57:15+07:00 | 2322-12-23T00:01:40+07:00 | 165 s / 100 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 20 | 2349-11-07 | 2349-11-07 | 1 | tiết khí | Tiết khí Lập Đông 2349 | 2349-11-07T23:55:52+07:00 | 2349-11-08T00:00:35+07:00 | 248 s / 35 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 21 | 2350-01-20 | 2350-01-20 | 1 | tiết khí | Tiết khí Đại Hàn 2350 | 2350-01-20T23:58:49+07:00 | 2350-01-21T00:03:26+07:00 | 71 s / 206 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 22 | 2353-11-26 | 2353-12-25 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 5614 | 2353-11-26T23:58:35+07:00 | 2353-11-27T00:02:43+07:00 | 85 s / 163 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 23 | 2354-12-07 | 2354-12-07 | 1 | tiết khí | Tiết khí Đại Tuyết 2354 | 2354-12-07T23:56:58+07:00 | 2354-12-08T00:01:37+07:00 | 182 s / 97 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 24 | 2355-04-20 | 2355-04-20 | 1 | tiết khí | Tiết khí Cốc Vũ 2355 | 2355-04-20T23:58:46+07:00 | 2355-04-21T00:03:23+07:00 | 74 s / 203 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 25 | 2358-06-07 | 2358-07-06 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 5670 | 2358-06-07T23:59:49+07:00 | 2358-06-08T00:03:56+07:00 | 11 s / 236 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 26 | 2358-08-07 | 2358-08-07 | 1 | tiết khí | Tiết khí Lập Thu 2358 | 2358-08-07T23:56:47+07:00 | 2358-08-08T00:01:28+07:00 | 193 s / 88 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 27 | 2370-05-05 | 2370-05-05 | 1 | tiết khí | Tiết khí Lập Hạ 2370 | 2370-05-05T23:57:11+07:00 | 2370-05-06T00:01:54+07:00 | 169 s / 114 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 28 | 2371-06-21 | 2371-06-21 | 1 | tiết khí | Tiết khí Hạ Chí 2371 | 2371-06-21T23:58:50+07:00 | 2371-06-22T00:03:29+07:00 | 70 s / 209 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 29 | 2387-12-07 | 2387-12-07 | 1 | tiết khí | Tiết khí Đại Tuyết 2387 | 2387-12-07T23:57:29+07:00 | 2387-12-08T00:02:25+07:00 | 151 s / 145 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 30 | 2394-07-22 | 2394-07-22 | 1 | tiết khí | Tiết khí Đại Thử 2394 | 2394-07-22T23:58:22+07:00 | 2394-07-23T00:03:15+07:00 | 98 s / 195 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 31 | 2406-03-20 | 2406-03-20 | 1 | tiết khí | Tiết khí Xuân Phân 2406 | 2406-03-20T23:59:41+07:00 | 2406-03-21T00:04:34+07:00 | 19 s / 274 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 32 | 2407-11-22 | 2407-11-22 | 1 | tiết khí | Tiết khí Tiểu Tuyết 2407 | 2407-11-22T23:56:10+07:00 | 2407-11-23T00:01:16+07:00 | 230 s / 76 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 33 | 2425-09-12 | 2425-10-11 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 6502 | 2425-09-12T23:57:43+07:00 | 2425-09-13T00:02:10+07:00 | 137 s / 130 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 34 | 2439-03-20 | 2439-03-20 | 1 | tiết khí | Tiết khí Xuân Phân 2439 | 2439-03-20T23:55:09+07:00 | 2439-03-21T00:00:06+07:00 | 291 s / 6 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 35 | 2440-06-04 | 2440-06-04 | 1 | tiết khí | Tiết khí Mang Chủng 2440 | 2440-06-04T23:55:16+07:00 | 2440-06-05T00:00:20+07:00 | 284 s / 20 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 36 | 2456-01-05 | 2456-01-05 | 1 | tiết khí | Tiết khí Tiểu Hàn 2456 | 2456-01-05T23:59:54+07:00 | 2456-01-06T00:05:06+07:00 | 6 s / 306 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 37 | 2460-07-18 | 2460-08-16 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 6933 | 2460-07-18T23:59:23+07:00 | 2460-07-19T00:03:57+07:00 | 37 s / 237 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 38 | 2473-11-21 | 2473-11-21 | 1 | tiết khí | Tiết khí Tiểu Tuyết 2473 | 2473-11-21T23:59:01+07:00 | 2473-11-22T00:04:19+07:00 | 59 s / 259 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 39 | 2476-12-16 | 2477-01-14 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 7136 | 2476-12-16T23:58:42+07:00 | 2476-12-17T00:03:17+07:00 | 78 s / 197 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 40 | 2483-06-05 | 2483-07-04 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 7216 | 2483-06-05T23:57:36+07:00 | 2483-06-06T00:02:11+07:00 | 144 s / 131 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 41 | 2487-02-03 | 2487-02-03 | 1 | tiết khí | Tiết khí Lập Xuân 2487 | 2487-02-03T23:58:50+07:00 | 2487-02-04T00:04:14+07:00 | 70 s / 254 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 42 | 2492-02-18 | 2492-02-18 | 1 | tiết khí | Tiết khí Vũ Thủy 2492 | 2492-02-18T23:59:25+07:00 | 2492-02-19T00:04:43+07:00 | 35 s / 283 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 43 | 2496-09-21 | 2496-09-21 | 1 | tiết khí | Tiết khí Thu Phân 2496 | 2496-09-21T23:56:06+07:00 | 2496-09-22T00:01:27+07:00 | 234 s / 87 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 44 | 2497-06-30 | 2497-07-29 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 7390 | 2497-06-30T23:55:38+07:00 | 2497-07-01T00:00:21+07:00 | 262 s / 21 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 45 | 2505-03-20 | 2505-03-20 | 1 | tiết khí | Tiết khí Xuân Phân 2505 | 2505-03-20T23:54:47+07:00 | 2505-03-21T00:00:07+07:00 | 313 s / 7 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 46 | 2509-03-05 | 2509-03-05 | 1 | tiết khí | Tiết khí Kinh Trập 2509 | 2509-03-05T23:55:52+07:00 | 2509-03-06T00:01:16+07:00 | 248 s / 76 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 47 | 2509-06-18 | 2509-07-17 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 7538 | 2509-06-18T23:58:50+07:00 | 2509-06-19T00:03:39+07:00 | 70 s / 219 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 48 | 2515-03-16 | 2515-04-14 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 7609 | 2515-03-16T23:59:25+07:00 | 2515-03-17T00:04:04+07:00 | 35 s / 244 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 49 | 2521-05-20 | 2521-05-20 | 1 | tiết khí | Tiết khí Tiểu Mãn 2521 | 2521-05-20T23:56:10+07:00 | 2521-05-21T00:01:42+07:00 | 230 s / 102 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 50 | 2524-02-05 | 2524-03-05 | 30 | Can Chi năm, Can Chi tháng, ngày âm, năm âm, tháng âm | Sóc 7719 | 2524-02-05T23:57:41+07:00 | 2524-02-06T00:02:16+07:00 | 139 s / 136 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 51 | 2538-03-20 | 2538-03-20 | 1 | tiết khí | Tiết khí Xuân Phân 2538 | 2538-03-20T23:56:26+07:00 | 2538-03-21T00:01:58+07:00 | 214 s / 118 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 52 | 2542-03-05 | 2542-03-05 | 1 | tiết khí | Tiết khí Kinh Trập 2542 | 2542-03-05T23:56:00+07:00 | 2542-03-06T00:01:34+07:00 | 240 s / 94 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 53 | 2554-08-22 | 2554-08-22 | 1 | tiết khí | Tiết khí Xử Thử 2554 | 2554-08-22T23:55:05+07:00 | 2554-08-23T00:00:30+07:00 | 295 s / 30 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 54 | 2556-01-20 | 2556-01-20 | 1 | tiết khí | Tiết khí Đại Hàn 2556 | 2556-01-20T23:59:18+07:00 | 2556-01-21T00:04:50+07:00 | 42 s / 290 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 55 | 2572-11-21 | 2572-11-21 | 1 | tiết khí | Tiết khí Tiểu Tuyết 2572 | 2572-11-21T23:57:10+07:00 | 2572-11-22T00:02:41+07:00 | 170 s / 161 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 56 | 2583-08-22 | 2583-08-22 | 1 | tiết khí | Tiết khí Xử Thử 2583 | 2583-08-22T23:56:59+07:00 | 2583-08-23T00:02:38+07:00 | 181 s / 158 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 57 | 2585-11-12 | 2585-12-11 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 8483 | 2585-11-12T23:58:41+07:00 | 2585-11-13T00:03:27+07:00 | 79 s / 207 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 58 | 2589-01-19 | 2589-01-19 | 1 | tiết khí | Tiết khí Đại Hàn 2589 | 2589-01-19T23:58:13+07:00 | 2589-01-20T00:03:48+07:00 | 107 s / 228 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 59 | 2589-06-04 | 2589-06-04 | 1 | tiết khí | Tiết khí Mang Chủng 2589 | 2589-06-04T23:57:50+07:00 | 2589-06-05T00:03:26+07:00 | 130 s / 206 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 60 | 2607-04-20 | 2607-04-20 | 1 | tiết khí | Tiết khí Cốc Vũ 2607 | 2607-04-20T23:55:29+07:00 | 2607-04-21T00:01:07+07:00 | 271 s / 67 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 61 | 2609-08-21 | 2609-09-19 | 30 | Can Chi tháng, ngày âm, nhuận, tháng âm | Sóc 8777 | 2609-08-21T23:57:03+07:00 | 2609-08-22T00:01:48+07:00 | 177 s / 108 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 62 | 2612-05-20 | 2612-05-20 | 1 | tiết khí | Tiết khí Tiểu Mãn 2612 | 2612-05-20T23:58:58+07:00 | 2612-05-21T00:04:36+07:00 | 62 s / 276 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 63 | 2618-10-10 | 2618-11-08 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 8890 | 2618-10-10T23:59:14+07:00 | 2618-10-11T00:04:11+07:00 | 46 s / 251 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 64 | 2627-12-22 | 2627-12-22 | 1 | tiết khí | Tiết khí Đông Chí 2627 | 2627-12-22T23:56:36+07:00 | 2627-12-23T00:02:19+07:00 | 204 s / 139 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 65 | 2641-03-05 | 2641-03-05 | 1 | tiết khí | Tiết khí Kinh Trập 2641 | 2641-03-05T23:59:04+07:00 | 2641-03-06T00:04:46+07:00 | 56 s / 286 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 66 | 2655-12-07 | 2655-12-07 | 1 | tiết khí | Tiết khí Đại Tuyết 2655 | 2655-12-07T23:54:37+07:00 | 2655-12-08T00:00:27+07:00 | 323 s / 27 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 67 | 2692-01-20 | 2692-01-20 | 1 | tiết khí | Tiết khí Đại Hàn 2692 | 2692-01-20T23:59:36+07:00 | 2692-01-21T00:05:26+07:00 | 24 s / 326 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 68 | 2694-02-18 | 2694-02-18 | 1 | tiết khí | Tiết khí Vũ Thủy 2694 | 2694-02-18T23:57:57+07:00 | 2694-02-19T00:03:43+07:00 | 123 s / 223 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 69 | 2695-01-05 | 2695-01-05 | 1 | tiết khí | Tiết khí Tiểu Hàn 2695 | 2695-01-05T23:57:18+07:00 | 2695-01-06T00:03:09+07:00 | 162 s / 189 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 70 | 2702-06-17 | 2702-07-16 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 9925 | 2702-06-17T23:58:30+07:00 | 2702-06-18T00:03:18+07:00 | 90 s / 198 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 71 | 2711-09-23 | 2711-09-23 | 1 | tiết khí | Tiết khí Thu Phân 2711 | 2711-09-23T23:58:50+07:00 | 2711-09-24T00:04:47+07:00 | 70 s / 287 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 72 | 2713-09-07 | 2713-09-07 | 1 | tiết khí | Tiết khí Bạch Lộ 2713 | 2713-09-07T23:55:56+07:00 | 2713-09-08T00:01:51+07:00 | 244 s / 111 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 73 | 2717-07-22 | 2717-07-22 | 1 | tiết khí | Tiết khí Đại Thử 2717 | 2717-07-22T23:56:58+07:00 | 2717-07-23T00:02:49+07:00 | 182 s / 169 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 74 | 2726-10-08 | 2726-10-08 | 1 | tiết khí | Tiết khí Hàn Lộ 2726 | 2726-10-08T23:56:25+07:00 | 2726-10-09T00:02:26+07:00 | 215 s / 146 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 75 | 2727-06-21 | 2727-06-21 | 1 | tiết khí | Tiết khí Hạ Chí 2727 | 2727-06-21T23:59:53+07:00 | 2727-06-22T00:05:42+07:00 | 7 s / 342 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 76 | 2765-05-20 | 2765-05-20 | 1 | tiết khí | Tiết khí Tiểu Mãn 2765 | 2765-05-20T23:56:29+07:00 | 2765-05-21T00:02:24+07:00 | 211 s / 144 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 77 | 2775-07-22 | 2775-07-22 | 1 | tiết khí | Tiết khí Đại Thử 2775 | 2775-07-22T23:58:11+07:00 | 2775-07-23T00:04:11+07:00 | 109 s / 251 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 78 | 2779-09-02 | 2779-10-01 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 10880 | 2779-09-02T23:58:42+07:00 | 2779-09-03T00:03:30+07:00 | 78 s / 210 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 79 | 2783-04-23 | 2783-05-22 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 10925 | 2783-04-23T23:56:10+07:00 | 2783-04-24T00:00:57+07:00 | 230 s / 57 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 80 | 2787-08-05 | 2787-09-03 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 10978 | 2787-08-05T23:58:41+07:00 | 2787-08-06T00:03:31+07:00 | 79 s / 211 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 81 | 2787-12-07 | 2787-12-07 | 1 | tiết khí | Tiết khí Đại Tuyết 2787 | 2787-12-07T23:56:24+07:00 | 2787-12-08T00:02:25+07:00 | 216 s / 145 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 82 | 2792-02-04 | 2792-02-04 | 1 | tiết khí | Tiết khí Lập Xuân 2792 | 2792-02-04T23:56:58+07:00 | 2792-02-05T00:02:53+07:00 | 182 s / 173 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 83 | 2797-04-19 | 2797-04-19 | 1 | tiết khí | Tiết khí Cốc Vũ 2797 | 2797-04-19T23:54:39+07:00 | 2797-04-20T00:00:33+07:00 | 321 s / 33 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 84 | 2808-05-16 | 2808-06-14 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 11235 | 2808-05-16T23:59:32+07:00 | 2808-05-17T00:04:15+07:00 | 28 s / 255 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 85 | 2821-12-15 | 2822-01-13 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 11403 | 2821-12-15T23:58:42+07:00 | 2821-12-16T00:03:27+07:00 | 78 s / 207 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 86 | 2829-06-04 | 2829-06-04 | 1 | tiết khí | Tiết khí Mang Chủng 2829 | 2829-06-04T23:59:42+07:00 | 2829-06-05T00:05:38+07:00 | 18 s / 338 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 87 | 2830-05-13 | 2830-06-11 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 11507 | 2830-05-13T23:57:35+07:00 | 2830-05-14T00:02:11+07:00 | 145 s / 131 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 88 | 2835-02-18 | 2835-03-19 | 30 | Can Chi năm, Can Chi tháng, ngày âm, năm âm, tháng âm | Sóc 11566 | 2835-02-18T23:58:33+07:00 | 2835-02-19T00:03:10+07:00 | 87 s / 190 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 89 | 2846-01-18 | 2846-02-16 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 11701 | 2846-01-18T23:58:21+07:00 | 2846-01-19T00:03:03+07:00 | 99 s / 183 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 90 | 2849-10-22 | 2849-10-22 | 1 | tiết khí | Tiết khí Sương Giáng 2849 | 2849-10-22T23:59:46+07:00 | 2849-10-23T00:05:46+07:00 | 14 s / 346 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 91 | 2852-10-04 | 2852-11-02 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 11784 | 2852-10-04T23:58:14+07:00 | 2852-10-05T00:03:01+07:00 | 106 s / 181 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 92 | 2859-05-23 | 2859-06-21 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 11866 | 2859-05-23T23:56:37+07:00 | 2859-05-24T00:01:18+07:00 | 203 s / 78 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 93 | 2862-12-21 | 2862-12-21 | 1 | tiết khí | Tiết khí Đông Chí 2862 | 2862-12-21T23:58:13+07:00 | 2862-12-22T00:04:09+07:00 | 107 s / 249 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 94 | 2865-06-14 | 2865-07-13 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 11941 | 2865-06-14T23:56:29+07:00 | 2865-06-15T00:01:13+07:00 | 211 s / 73 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 95 | 2876-10-08 | 2876-11-06 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 12081 | 2876-10-08T23:59:23+07:00 | 2876-10-09T00:04:04+07:00 | 37 s / 244 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 96 | 2888-08-05 | 2888-08-05 | 1 | tiết khí | Tiết khí Lập Thu 2888 | 2888-08-05T23:56:21+07:00 | 2888-08-06T00:02:19+07:00 | 219 s / 139 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 97 | 2890-06-08 | 2890-07-07 | 30 | Can Chi tháng, ngày âm, tháng âm | Sóc 12250 | 2890-06-08T23:58:51+07:00 | 2890-06-09T00:03:26+07:00 | 69 s / 206 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 98 | 2898-11-06 | 2898-11-06 | 1 | tiết khí | Tiết khí Lập Đông 2898 | 2898-11-06T23:55:42+07:00 | 2898-11-07T00:01:46+07:00 | 258 s / 106 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 99 | 2922-09-22 | 2922-09-22 | 1 | tiết khí | Tiết khí Thu Phân 2922 | 2922-09-22T23:55:33+07:00 | 2922-09-23T00:01:34+07:00 | 267 s / 94 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 100 | 2928-02-04 | 2928-02-04 | 1 | tiết khí | Tiết khí Lập Xuân 2928 | 2928-02-04T23:55:41+07:00 | 2928-02-05T00:01:37+07:00 | 259 s / 97 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 101 | 2928-05-04 | 2928-05-04 | 1 | tiết khí | Tiết khí Lập Hạ 2928 | 2928-05-04T23:58:35+07:00 | 2928-05-05T00:04:33+07:00 | 85 s / 273 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 102 | 2942-02-06 | 2942-03-07 | 30 | Can Chi năm, Can Chi tháng, ngày âm, năm âm, tháng âm | Sóc 12889 | 2942-02-06T23:58:10+07:00 | 2942-02-07T00:02:45+07:00 | 110 s / 165 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 103 | 2947-05-20 | 2947-05-20 | 1 | tiết khí | Tiết khí Tiểu Mãn 2947 | 2947-05-20T23:58:08+07:00 | 2947-05-21T00:04:01+07:00 | 112 s / 241 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 104 | 2953-07-21 | 2953-07-21 | 1 | tiết khí | Tiết khí Đại Thử 2953 | 2953-07-21T23:54:39+07:00 | 2953-07-22T00:00:42+07:00 | 321 s / 42 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 105 | 2967-04-04 | 2967-04-04 | 1 | tiết khí | Tiết khí Thanh Minh 2967 | 2967-04-04T23:56:59+07:00 | 2967-04-05T00:03:04+07:00 | 181 s / 184 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 106 | 2997-01-19 | 2997-01-19 | 1 | tiết khí | Tiết khí Đại Hàn 2997 | 2997-01-19T23:57:09+07:00 | 2997-01-20T00:03:13+07:00 | 171 s / 193 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |
| 107 | 2997-08-21 | 2997-08-21 | 1 | tiết khí | Tiết khí Xử Thử 2997 | 2997-08-21T23:57:07+07:00 | 2997-08-22T00:03:07+07:00 | 173 s / 187 s | sát nửa đêm / hai phương pháp đặt sự kiện khác ngày |

## Sự kiện khác ngày nhưng không làm khác ngày nào

| Sự kiện | Precise | Jpl | Cách nửa đêm Precise / Jpl |
|---|---|---|---|
