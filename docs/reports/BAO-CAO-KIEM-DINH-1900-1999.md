# Báo cáo kiểm định lịch 1900–1999 (Bổ sung A1)

Ngày lập: 2026-11-06. Người lập: bên không tham gia xây dựng đặc tả (kiểm định độc lập, phòng sạch).
So sánh chỉ dựa trên file dữ liệu trong `core/out/` (`lunar_1900_1999.csv`, `tet_1900_1999.csv`,
`newmoons_1900_1999.csv`, bản `precise/` tương ứng, `dayinfo_1900_3000.jsonl.gz`, `baseline-2000-3000/*.jsonl.gz`).
Không đọc mã nguồn core/, ngochap/, tsport/. Quy tắc múi giờ của oracle được đặt và kết quả oracle được lưu
TRƯỚC khi đọc docs/03 (xem mục 7).

## 1. Phương pháp oracle và quy tắc múi giờ (do bên kiểm định tự đặt)

- Thiên văn: giống oracle 2000–3000 (`oracle/oracle.py`): Sóc `ephem.next_new_moon` (PyEphem 4.2.1), tiết khí
  bằng chia đôi trên kinh độ biểu kiến Mặt Trời. Mã mở rộng: `oracle/oracle_1900.py`.
- **Quy tắc múi giờ của oracle:** sự kiện tại thời điểm UT `t` được quy ra ngày dân sự theo UTC+8 nếu
  `t < 1967-12-31 17:00 UT` (= 1968-01-01 00:00 UTC+7), ngược lại UTC+7. Ngày mới bắt đầu 00:00 theo múi giờ áp dụng.
- **Bằng chứng chọn mốc 1968-01-01 00:00:** Quyết định 121-CP ký 08/8/1967, hiệu lực "from 0 hours on 1.1.1968"
  (Trần Tiến Bình, *Lịch Việt Nam thế kỷ XX–XXI*, Ban Lịch Nhà nước, dẫn qua tz@iana.org); en.wikipedia
  "Time in Vietnam": Bắc Việt UTC+7 chính thức từ 1968-01-01; phỏng vấn Lê Thành Lân / Trịnh Tiến Điều
  (RFA 2007): "Bắt đầu từ 68 chúng ta ăn tết khác Trung Quốc". Nguồn đầy đủ: `oracle/reference/SOURCES.md`.
- Biến thể đo độ nhạy (không phải quy tắc): mốc 1967-08-08 (ngày ký) → khác quy tắc oracle **30 ngày**
  (1967-12-01 → 1967-12-30: Sóc 1967-12-01 23:10 UTC+7 / 12-02 00:10 UTC+8). Cả hai mốc cho Tết 1968 = 01-29.
- Đầu ra (lưu 11:5x, trước khi có dữ liệu lõi và trước khi đọc docs/03): `oracle/out/lunar_1900_1999.csv`,
  `tet_1900_1999.csv`, `newmoons_1900_1999.csv`, `terms_1900_1999.csv`; biến thể `*_var0808`, `*_utc8`, `*_utc7`,
  `tet_variants_1900_1999.csv`.

## 2. Kết luận theo tiêu chí

| Mã | Kết luận | Tóm tắt |
|---|---|---|
| L9 | **KHÔNG ĐẠT** | Nguồn VN phủ 100/100 năm. Lõi (cả hai phương pháp) lệch ít nhất một nguồn VN ở 4 năm: 1916 (mọi nguồn VN: 02-03; lõi 02-04), 1903, 1935, 1965 (chogia lệch, xemlicham khớp). Các nguồn VN tự mâu thuẫn ở 1903/1925/1935/1965 (mục 3). |
| L10 (hnd) | **ĐẠT** | 0/36 524 ngày sai lệch (năm, tháng, ngày, nhuận). |
| L10 (precise) | **ĐẠT** | 0/36 524 ngày sai lệch. |
| L11 | **ĐẠT** (cả hai phương pháp) | 1967-01-01→1969-12-31: 1096/1096 ngày, mỗi ngày đúng một ngày âm; 0 lỗi tăng đều/thiếu/trùng; Tết 1968 = 1968-01-29. |
| L13 | **ĐẠT** (kiểm từ dữ liệu) | Phần 2000-01-01→3000-12-31 của `dayinfo_1900_3000.jsonl.gz` (dòng 36 525→402 132) trùng từng byte với `baseline-2000-3000/hnd.jsonl.gz` và `precise.jsonl.gz` (sha256 khớp: hnd 84e998…7bb44, precise 95049e…d4943). |

Ngoài tiêu chí L10 (không thuộc trường năm/tháng/ngày/nhuận): cột `tiet_khi` lệch đúng **1 ngày** ở cả hai phương
pháp: 1951-12-22 (oracle: Đông Chí; lõi: Đại Tuyết). Đông Chí 1951 theo oracle lúc 1951-12-22 23:59:57 UTC+8
(cách nửa đêm 3 giây) → loại **sát nửa đêm**. Lõi không xuất file thời điểm tiết khí 1900–1999 nên không đo được
thời điểm của lõi. Sự kiện này không đổi tháng âm (hai bản trùng mọi ngày âm).

## 3. L9 — Tết so với nguồn Việt Nam

Dữ liệu: `oracle/reference/tet_published_1900_1999.csv` (125 dòng); nguồn: `oracle/reference/SOURCES.md`.
Tết lõi hnd = Tết lõi precise = Tết oracle ở cả 100 năm.

| Năm | Lõi | xemlicham | chogia | HKO (TQ, hỗ trợ) | Sóc (oracle, UTC+8) |
|---|---|---|---|---|---|
| 1903 | 01-29 | 01-29 ✔ | 01-28 ✘ | — | 01-29 00:38 |
| 1916 | 02-04 | 02-03 ✘ | 02-03 ✘ | 02-03 | 02-04 00:05 |
| 1925 | 01-24 | 01-25 (trang năm) / 01-24 (trang ngày) | 01-24 ✔ | — | 01-24 22:45 |
| 1935 | 02-04 | 02-04 ✔ | 02-03 ✘ | — | 02-04 00:27 |
| 1965 | 02-02 | 02-02 ✔ | 02-01 ✘ | 02-02 | 02-02 00:36 |
| 1968 | 01-29 | 01-29 ✔ | 01-29 ✔ | 01-30 (đúng như kỳ vọng: TQ UTC+8) | 01-29 23:29 UTC+7 |

Các năm còn lại: lõi khớp mọi nguồn VN có cho năm đó (xemlicham 100 năm; vi.wikipedia bản sao 1968, 1996–1999; baolamdong 1954; chogia 1900, 1914, 1950, 1969).

Lý do KHÔNG ĐẠT: ngưỡng khóa là "0 sai lệch ở các năm có nguồn". Năm 1916 mọi nguồn VN truy cập được đều ghi
1916-02-03 còn lõi ghi 02-04; ba năm 1903, 1935, 1965 lõi lệch chogia. Phân tích (không thay đổi kết luận):
- Các nguồn VN là trang tra cứu tính toán, không phải bảng in của Ban Lịch Nhà nước, và **mâu thuẫn nhau**:
  chogia khớp đúng phép tính UTC+7 suốt (mọi năm kiểm, kể cả 1903/1916/1935/1965); xemlicham khớp lịch Trung Quốc
  (UTC+8, riêng 1916 theo giờ địa phương Bắc Kinh). Không cách cài đặt nào khớp cả hai nguồn ở 1903/1935/1965.
- 1916: Sóc rơi 00:05 UTC+8. Lịch Trung Quốc trước 1929 tính theo kinh tuyến Bắc Kinh (~UTC+7:46) nên cho 02-03 (HKO xác nhận).
  Quy tắc chủ dự án "UTC+8 trước khi chuyển" cho 02-04. Đây là sai lệch do quy ước, không phải lỗi cài đặt.
- 1925: trang năm của xemlicham (01-25) mâu thuẫn chính tiêu đề trang ngày của nó (24/01/1925 = "1-1-1925"); Sóc cách nửa
  đêm >1 giờ ở mọi múi giờ 7–8, nên 01-25 nhiều khả năng là lỗi dữ liệu trang. Lõi khớp hai nguồn VN còn lại.
- Muốn L9 ĐẠT cần chủ dự án quyết định trước (không nới sau khi thấy kết quả): nguồn nào là "nguồn công bố" có thẩm
  quyền cho trước 1968, và quy ước 1900–1928 (UTC+8 hay giờ Bắc Kinh).

## 4. L10 — sai lệch ngày âm 1900–1999

| Phương pháp | Ngày so | Thiếu ở lõi | Sai lệch âm lịch | Sát nửa đêm | Nguyên nhân khác |
|---|---|---|---|---|---|
| hnd | 36 524 | 0 | 0 | 0 | 0 |
| precise | 36 524 | 0 | 0 | 0 | 0 |

Sóc lệch ngày giữa oracle và lõi: 0 (cả hai phương pháp). Danh sách ngày: `oracle/out/mismatches_1900_1999_{hnd,precise}.csv`
(chỉ chứa dòng tiet_khi 1951-12-22).

## 5. Chênh lệch thời điểm Sóc (oracle − lõi), 1900–1999

| Phương pháp | n | max \|Δ\| | trung bình \|Δ\| | trung bình có dấu | 3 chênh lớn nhất |
|---|---|---|---|---|---|
| hnd | 1237 | 202 s | 55,5 s | −44,4 s | 1978-04-07 22:15 (−202 s); 1978-03-09 09:36 (−193 s); 1978-05-07 11:46 (−184 s) |
| precise | 1237 | 15 s | 3,5 s | +0,1 s | 1940-02-08 15:44 (−15 s); 1966-09-15 03:13 (+14 s); 1901-11-11 15:34 (−13 s) |

File chi tiết: `oracle/out/nm_diff_1900_1999_{hnd,precise}.csv`. Múi giờ lõi ghi trong file Sóc: +08:00 cho 842 Sóc
1900-01-01 → 1967-12-31 11:39 (UTC+8), +07:00 cho 395 Sóc từ 1968-01-29.

## 6. Câu hỏi quy ước 1900–1945 — bằng chứng nói gì

- Giờ hành chính Đông Dương: UTC+7:06 (đến 1911), UTC+7 (1911–1942), UTC+8 (1942–1945), UTC+9 (1945); sau 1945 miền
  Bắc chủ yếu UTC+7 (tz@iana.org; en.wikipedia "Time in Vietnam"). **Giờ đồng hồ KHÔNG phải múi giờ tính âm lịch**:
  trước 1968 âm lịch dân gian dùng lịch Trung Quốc (vi.wikipedia: "Trước năm 1967, Việt Nam lấy giờ Bắc Kinh làm chuẩn
  cho âm lịch").
- Triều Nguyễn có lịch Hiệp Kỷ (đến 1945/1954 tùy nguồn), khác lịch Trung Quốc do kinh độ (en.wikipedia "Vietnamese
  calendar"); suikawiki: VNCal 1301–1945 theo Hoàng Xuân Hãn *Lịch và lịch Việt Nam*. Không tìm được bảng Hiệp Kỷ
  1900–1945 trực tuyến để kiểm.
- "Giờ Bắc Kinh" trước 1929 là giờ địa phương kinh tuyến Bắc Kinh (~UTC+7:46), không phải UTC+8. Trong 1900–1928 điều này
  chỉ đổi 120 ngày (1906, 1914, 1916, 1920) so với UTC+8 thuần, trong đó chỉ Tết 1916 đổi.
- Độ nhạy (oracle, so với quy tắc chủ dự án): UTC+7 suốt → khác 957 ngày (27 năm, 4 Tết: 1903, 1916, 1935, 1965);
  giờ Huế ~UTC+7:10 cho 1900–1945 → khác 538 ngày (4 Tết: 1903, 1916, 1935 — và không 1965); Bắc Kinh địa phương trước 1929
  → 120 ngày (1 Tết: 1916).
- Kết luận bằng chứng: quy ước "UTC+8 trước 1968" là xấp xỉ hợp lý của tập quán dân gian (theo lịch Trung Quốc) cho
  1929–1967; cho 1900–1928 xấp xỉ này lệch lịch Trung Quốc thực tế ở 1916; cho 1900–1945 lịch chính thức triều Nguyễn
  (Hiệp Kỷ) có thể khác cả hai — **CHƯA ĐỦ BẰNG CHỨNG** để nói lịch nào là "lịch Việt Nam" của thời kỳ này.

## 7. Quy tắc chuyển múi giờ của lõi so với oracle

Thời điểm đọc docs/03: sau khi lưu toàn bộ đầu ra oracle (mục 1). Tại thời điểm lập báo cáo (12:1x), docs/03 (sửa lần
cuối 11:09) **chưa có** mục quy tắc chuyển múi giờ (chỉ ghi "UTC+7 cố định cho mọi năm 2000–3000"). Vì vậy so sánh
quy tắc dựa trên dữ liệu:
- Lõi gắn +08:00 cho Sóc 1967-12-02 00:10 và 1967-12-31 11:39 (UTC+8), +07:00 cho Sóc 1968-01-29 → mốc chuyển của lõi
  nằm trong khoảng (1967-12-31 03:39 UT, 1968-01-29 16:31 UT); khớp mốc oracle 1967-12-31 17:00 UT.
- Lõi KHÔNG dùng mốc 1967-08-08 (nếu dùng thì Sóc 1967-12-01 23:10 UTC+7 sẽ mang +07:00 và tháng 11/1967 bắt đầu 12-01;
  lõi bắt đầu 12-02 như oracle).
- Ảnh hưởng mức ngày giữa quy tắc lõi và quy tắc oracle: **0 ngày** (1900–1999, cả hai phương pháp).
- Mục này phải được đối chiếu lại khi docs/03 ghi quy tắc chính thức; nếu văn bản khác dữ liệu, đó là lỗi tài liệu.

## 8. Giới hạn

- Nguồn VN 1900–1999 đều là trang tính toán trực tuyến; không có bảng chính thức của Ban Lịch Nhà nước; HKO chỉ là bằng chứng hỗ trợ.
- ΔT 1900–1999 là đo đạc nên sai số thiên văn nhỏ (chênh precise ≤ 15 s). Sự kiện sát nửa đêm nhất: Đông Chí 1951 (3 s).
- L12 (khứ hồi, TypeScript) không thuộc phạm vi bên kiểm định trong báo cáo này.
