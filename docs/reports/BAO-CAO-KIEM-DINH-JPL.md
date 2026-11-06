# Báo cáo kiểm định phương pháp `Jpl` (Bổ sung A4: L14, L15, L17)

Ngày lập: 2026-11-07. Người lập: bên kiểm định độc lập (phòng sạch). Nhánh: `oracle-de441`.
Tiêu chí: docs/02 "Bổ sung A4" (khóa 2026-11-07). Đặc tả đối chiếu: docs/03 §17 bản cuối (nhánh `jpl-method`,
commit `d238c66`), chỉ đọc riêng mục §17.

## 0. Kết luận

| Mã | Kết luận | Tóm tắt |
|---|---|---|
| L14 | **ĐẠT** | 40 090/40 090 sự kiện (13 634 Sóc, 26 456 tiết khí) ghép một-một; max \|Δ\| = **0,0172 s** (ngưỡng 2 s); 0 sự kiện vượt ngưỡng; 0 sự kiện hai bên đặt khác ngày. |
| L15 | **ĐẠT** | 402 132/402 132 ngày (1900-01-01 → 3000-12-31) trùng ở cả 8 trường (năm/tháng/ngày âm, nhuận, Can Chi năm/tháng/ngày, tiết khí): **0 sai lệch**, 0 ngày thiếu/thừa; 0 ngày phải xếp "không phân định được". Hai sự kiện cách nửa đêm < 2 s được liệt kê riêng (mục 4); hai bên đặt cùng ngày. |
| L17 (phần bên kiểm định tự chấm bằng lịch của mình) | **ĐẠT** | Tết 1900–1999: 100/100 năm đúng cách chấm A3 (0 lệch); bảng TK20 của Hồ Ngọc Đức: **1237/1237** tháng khớp. |

L16 và phần L17 chạy trên lõi do nhóm dự án làm (docs/03 §17.7); bên kiểm định chỉ chấm L17 bằng lịch của chính mình
(lịch này trùng kết xuất `Jpl` của lõi ở mọi ngày 1900–3000 theo L15, nên kết quả áp được cho lõi — xem mục 4).

## 1. Thứ tự công việc (phòng sạch)

| Thời điểm (UTC, 2026-11-06) | Việc |
|---|---|
| 17:24 | Tạo nhánh `oracle-de441` từ `main`; dựng chuỗi tính DE441 → TT (chưa có §17). |
| 17:30–17:32 | Đọc §17 bản đầu (`27f98df`), rồi bản cuối (`d238c66`, có dòng FINAL). Chỉ đọc mục §17. Công thức ΔT và quy đổi của hai bản giống nhau; bản cuối thêm mục (d) "không hiệu chỉnh gia tốc thủy triều" — bên kiểm định không áp dụng hiệu chỉnh đó. |
| 17:35 | Máy chủ tích hợp tính xong thời điểm từ DE441 (`out/instants_tt.csv.gz`). |
| **17:36:30** | **Commit `98688cc` kết quả của bên kiểm định** (thời điểm, lịch 1900–3000, L17, so PyEphem) và đẩy lên nhánh — trước khi xem bất kỳ dữ liệu nào của bên xây dựng. |
| 17:36:47 | Bên xây dựng commit bảng (`f2fda7a`, `core/data/jpl_events.csv.gz`). Bên kiểm định chỉ đọc tệp CSV kiểm toán này (L14). |
| ~17:45 | Chạy hộp đen `lvhv-dump --method jpl lunar-csv 1900-01-01 3000-12-31` (dựng từ nhánh `jpl-method` commit `c71a24d`, chỉ chạy, không đọc mã) trên máy chủ tích hợp (L15). |

Không đọc `core/`, `tsport/`, `ffi/`, `tools/jpl/` hay bảng nhị phân của lõi. Từ bản dựng của bên xây dựng chỉ dùng
đầu ra `--help` và kết xuất `lunar-csv`, `tet`.

## 2. Phương pháp của bên kiểm định

### 2.1 Thời điểm sự kiện (TT) — `oracle/de441/instants.py`

- **Lịch thiên văn:** `de441.bsp` (JPL, 3 307 878 400 byte, SHA-256 `9a824e6a8ce3d39f8a513b17278ff7b7ad226f964c76d2019b0a51d8808d6ff5`),
  tải trên máy chủ tích hợp từ <https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/>. Thư mục này hiện **không còn**
  `de441_part-1.bsp`/`de441_part-2.bsp` như §17.2 ghi (tải trả 404, xem `oracle/de441/ref/de441_dir.html`); `de441.bsp` chứa
  hai đoạn SPK cho mỗi cặp thân, nối đầu-cuối đúng tại TDB JD 2440432,5 (1969-07-30). Bên kiểm định dùng đoạn trước cho
  TDB < 2440432,5, đoạn sau từ đó (điểm 2440403,5 của §17.2 không nằm trong đoạn sau của tệp này). Kiểm tra liên tục tại
  điểm nối (`out/overlap_check.txt`): chênh vị trí ≤ 3,4·10⁻⁸ km, vận tốc ≤ 2,3·10⁻¹⁰ km/ngày cho cả 4 cặp thân.
- **Phần mềm khác bên xây dựng:** `jplephem` 2.24 chỉ để đọc hệ số Chebyshev từng đoạn SPK; toàn bộ quy đổi tự viết trên
  `pyerfa` 2.0.1.5 (ERFA 2.0.1), `numpy` 2.2.6, Python 3.12 (máy chủ tích hợp). **Không dùng Skyfield.**
- **Chuỗi quy đổi:** TT → TDB bằng `eraDtdb` (địa tâm); vị trí khối tâm hệ Mặt Trời của Trái Đất (0→3 + 3→399), Mặt Trăng
  (0→3 + 3→301), Mặt Trời (0→10); thời gian ánh sáng lặp đến |Δτ| < 10⁻¹² ngày; lệch hấp dẫn do Mặt Trời cho Mặt Trăng
  (`eraLd`; không áp cho Sao Mộc/Sao Thổ — §17.3 cho phép bỏ); quang sai tương đối tính (`eraAb`) theo vận tốc khối tâm Trái
  Đất; ma trận GCRS → xích đạo thật của ngày `eraPn06` với chương động `eraNut06a` (IAU 2000A + hiệu chỉnh IAU 2006),
  tuế sai IAU 2006, lệch khung; quay sang hoàng đạo thật bằng ε = `eraObl06` + Δε.
- **Tìm nghiệm:** Newton véc-tơ hóa (đạo hàm sai phân trung tâm, bước 10⁻⁴ ngày) đến khi bước < 10⁻⁹ ngày. Phần dư cuối quy
  ra thời gian: Sóc ≤ 0,00027 s, tiết khí ≤ 0,000026 s (ghi trong đầu tệp). Kiểm tra trong mã: khoảng cách Sóc 29–30 ngày,
  tiết khí 14–16,5 ngày, tiết khí liên tiếp tăng đúng 15°.
- **Phạm vi:** mọi sự kiện TT trong [1899-10-01, 3002-02-01) theo §17.3 (rộng hơn A4 để đủ tháng 11 âm năm 3001):
  13 634 Sóc, 26 456 tiết khí.
- **Độ nhạy (thông tin):** chạy lại với chương động IAU 2000B (`out/instants_tt_nut2000B.csv.gz`) đổi thời điểm tối đa
  1,66 s — chỉ để biết mức ảnh hưởng của mô hình chương động; kết quả dùng chấm là IAU 2000A như §17.3.
- **Kiểm tra thô với Meeus:** Sóc 1977-02-18 03:37:39,9 TT (Meeus ví dụ 49.a: 03:37:42 TD, thuật toán gần đúng);
  Xuân phân 2000-03-20 07:36:18,5 TT.

### 2.2 ΔT — `oracle/de441/deltat.py`

Tự cài theo §17.4: Y = 2000 + (JD_TT − 2451544,5)/365,2425; (a) Y < 2019,0: bảng "Table S15.2020" (Morrison, Stephenson,
Hohenkerk & Zawilski 2021), hàng K_i ≤ Y < K_{i+1}; (b) 2019,0 ≤ Y < 2800,0: Hermite bậc ba nối giá trị và đạo hàm của bảng
tại 2019,0 (69,240 s; 0,282 s/năm) với parabol tại 2800,0 (2769,53125 s; 6,3375 s/năm); (c) Y ≥ 2800,0: −320 + 32,5·u²,
u = (Y − 1825)/100. Không hiệu chỉnh gia tốc thủy triều (§17.4 d).

- Nguồn bảng: trang HM Nautical Almanac Office (astro.ukho.gov.uk) trả 503 cho máy chủ tích hợp và bị chặn từ môi trường làm
  việc; bên kiểm định dùng bản sao nguyên văn `Table-S15.2020.txt` (tiêu đề "Table S15 … v. 2020", 58 hàng, SHA-256
  `cfcb7dfc…e5e723`) lấy từ kho mã công khai của một thư viện thiên văn (raw.githubusercontent.com, skyfielders/python-skyfield,
  tệp dữ liệu, không dùng mã). Đây cùng loại bản sao mà §17.4 nêu; 34 hàng 1895–2019 trùng nguyên văn bảng in trong §17.4.
  Kiểm tra nội bộ: các đoạn liên tiếp liên tục (nhảy ≤ 0,001 s tại nút do làm tròn 3 chữ số).
- Khớp bảng giá trị kiểm tra của §17.4 ở cả 14 năm (chênh ≤ 3·10⁻⁷ s). Lưu ý cài đặt: tại nút Y = 1950,0 nếu chọn hàng
  1945–1950 (t = 1) cho 28,931 thay vì 28,932 — bên kiểm định theo đúng quy tắc nửa mở của §17.4 (sửa trước khi có kết quả).

### 2.3 Lịch — `oracle/de441/calendar_de441.py`

- JD_UT = JD_TT − ΔT/86400. Múi giờ (docs/03 §16.1 = A2): UT+7:45:40 trước 1928-12-31 16:14:20 UT; UTC+8 đến trước
  1967-12-31 16:00 UT; UTC+7 từ đó. Ngày chứa sự kiện = ⌊JD_UT + lệch + 0,5⌋ (00:00 đúng thuộc ngày mới).
- Quy tắc docs/01 §2: tháng 11 chứa Đông chí; tuế 13 tháng → nhuận là tháng đầu tiên sau tháng 11 không chứa Trung khí
  (theo ngày); Can Chi docs/01 §3; tiết khí của ngày = tiết khí muộn nhất có ngày bắt đầu ≤ ngày đó. Viết mới, không dùng
  PyEphem; logic giống `oracle/oracle.py` đã được kiểm định trước đây.
- Độ nhạy mốc chuyển (thông tin): mốc 1929 = 16:00 UT (quy tắc oracle cũ) hoặc mốc 1968 = 17:00 UT cho lịch **trùng từng
  byte** với quy tắc §16.1 (không sự kiện nào nằm trong các khoảng đó).
- Đầu ra: `out/days_1900_3000.csv.gz` (402 132 ngày; SHA-256 `162b911c…daf`), `out/months_1900_3000.csv`,
  `out/tet_1900_3000.csv`, `out/events.csv.gz` (mỗi sự kiện: ΔT, UT, giờ lịch, khoảng cách tới nửa đêm).

## 3. L14 — thời điểm TT so với bảng của bên xây dựng

Nguồn bên xây dựng: `core/data/jpl_events.csv.gz` tại `f2fda7a` (cột `n_cs`, thời điểm làm tròn 0,01 s; bên kiểm định
dùng JD_TT = 2451545 + n_cs/8 640 000). So sánh: `oracle/de441/compare_builder.py`; chi tiết `oracle/de441/cmp_builder/`
(`l14_diff.csv.gz`, `l14_by_century.csv`, `l14_stdout.txt`).

| Loại | n | max \|Δ\| | TB \|Δ\| | TB có dấu (KĐ − XD) |
|---|---|---|---|---|
| Sóc | 13 634 | 0,0054 s | 0,0025 s | +0,0000 s |
| Tiết khí | 26 456 | 0,0172 s | 0,0042 s | +0,0001 s |

- Ghép một-một: 0 sự kiện của bên xây dựng không có cặp, 0 sự kiện của bên kiểm định không có cặp; chỉ số tiết khí khớp kinh độ.
- Sự kiện lệch lớn nhất: tiết khí i = 9 (Lập Thu, 135°) 2960-08-06 03:30:13,24 TT, Δ = 0,0172 s.
- **Vượt 2 s: 0.** Max theo thế kỷ: Sóc 0,0049–0,0054 s mọi thế kỷ (≈ sai số làm tròn 0,005 s của bảng); tiết khí tăng đều
  0,0063 s (1900) → 0,0172 s (2900). Phần vượt mức làm tròn của tiết khí tăng theo thời gian; nguyên nhân khả dĩ (chưa
  kiểm): bên kiểm định dùng chương động có hiệu chỉnh IAU 2006 (`eraNut06a`), §17.8 ghi thư viện của bên xây dựng dùng
  IAU 2000A — §17.3 bước 4 cho phép cả hai. Không ảnh hưởng ngưỡng.
- Đặt ngày (cùng ΔT §17.4 và múi giờ §16.1 áp cho thời điểm của cả hai bên): **0 sự kiện** hai bên đặt khác ngày lịch.

**Kết luận L14: ĐẠT.**

## 4. L15 — lịch từng ngày 1900-01-01 → 3000-12-31

Đầu vào bên xây dựng: `lvhv-dump --method jpl lunar-csv 1900-01-01 3000-12-31` dựng từ `jpl-method` commit
`c71a24d35789aafefaaba210ce75c488727a27cc` trên máy chủ tích hợp (run 42264388574); tệp
`oracle/de441/builder/jpl_lunar_1900_3000.csv.gz` (402 133 dòng gồm tiêu đề). Bảng sự kiện nhúng ở commit này trùng
từng byte tệp kiểm toán dùng cho L14 (SHA-256 `3f563433…540f97` ở cả `f2fda7a` và `c71a24d`).
Đầu vào bên kiểm định: `oracle/de441/out/days_1900_3000.csv.gz` (commit `98688cc`, trước khi có kết xuất).

| Hạng mục | Kết quả |
|---|---|
| Ngày so sánh | 402 132 (thiếu ở bên xây dựng: 0; thừa: 0) |
| Ngày lệch ở bất kỳ trường nào (ly, lm, ld, leap, cc_year, cc_month, cc_day, tiet_khi) | **0** |
| Đoạn lệch "không phân định được" | 0 |
| Đoạn lệch tính vào tiêu chí | 0 |

Ngày bắt đầu tiết khí được so gián tiếp qua cột `tiet_khi` của từng ngày (ngày đầu tiên mang tên tiết khí mới = ngày bắt
đầu); 0 lệch. Đối chiếu thêm `lvhv-dump --method jpl tet 1900 3000`: 1101/1101 năm trùng `out/tet_1900_3000.csv`.

**Sự kiện cách nửa đêm lịch < 2 s (liệt kê riêng theo L15; ΔT và múi giờ áp như nhau cho thời điểm của hai bên):**

| Sự kiện | Bên kiểm định | Bên xây dựng | Ngày |
|---|---|---|---|
| Đông Chí (270°) | 1951-12-23 00:00:01,533 (+08:00) | 1951-12-23 00:00:01,530 | hai bên cùng 1951-12-23 |
| Tiểu Tuyết (240°) | 2341-11-22 23:59:59,342 (+07:00) | 23:59:59,338 | hai bên cùng 2341-11-22 |

Hai sự kiện này không gây sai lệch nào (hai bên cùng ngày) nhưng ngày của chúng phụ thuộc vào mô hình ở mức giây
(ví dụ PyEphem cũ đặt Đông Chí 1951 vào 12-22). Các sự kiện kế tiếp gần nửa đêm nhất (< 10 s): Kinh Trập 2109-03-05
(9,3 s), Sóc 2130-05-08 (8,9 s), Sóc 2415-12-01 (6,2 s), Xuân Phân 2439-03-21 (5,9 s), Xuân Phân 2505-03-21 (6,9 s),
Cốc Vũ 2702-04-20 (4,2 s), Lập Xuân 2961-02-03 (4,7 s); hai bên cùng ngày ở mọi sự kiện, chênh ≤ 0,012 s.

Chi tiết: `oracle/de441/cmp_builder/compare_stdout.txt`, `l15_mismatch_segments.csv` (rỗng), `l15_mismatch_days.csv.gz` (rỗng),
`events_day_differ.csv` (rỗng).

**Kết luận L15: ĐẠT.**

## 5. L17 — Tết 1900–1999 và bảng TK20 (bên kiểm định chấm bằng lịch của mình)

Script: `oracle/de441/l17.py`; kết quả `oracle/de441/out/l17_tet_1900_1999.csv`, `l17_tk20.csv`.

**Tết:** nguồn `oracle/reference/tet_published_1900_1999.csv` (nguồn VN phủ 100/100 năm). 96 năm các nguồn VN thống nhất:
khớp mọi nguồn. 4 năm nguồn VN mâu thuẫn, chấm theo A3 (nguồn tuân thủ quy ước giờ Bắc Kinh trước 1929 / UTC+8 đến 1967):

| Năm | Bên kiểm định (Jpl) | xemlicham | chogia | Nguồn chấm (A3) | Nguồn khác lệch |
|---|---|---|---|---|---|
| 1903 | 01-29 | 01-29 | 01-28 | xemlicham | chogia −1 ngày |
| 1925 | 01-24 | 01-25 (trang năm) / 01-24 (trang ngày) | 01-24 | xemlicham trang ngày (= chogia) | xemlicham trang năm +1 ngày |
| 1935 | 02-04 | 02-04 | 02-03 | xemlicham | chogia −1 ngày |
| 1965 | 02-02 | 02-02 | 02-01 | xemlicham | chogia −1 ngày |

Tết 1916 = 1916-02-03, Tết 1968 = 1968-01-29 (khớp mọi nguồn VN). Lệch: **0/100**.

**TK20:** bảng `TK20` (năm âm 1900–1999) trong mã nguồn `amlich` của Hồ Ngọc Đức (© 2004), bản phân phối lại
`ichuot/jquery-amlich` commit `2579ae15`, tải bởi máy chủ tích hợp (`oracle/de441/ref/FETCH-LOG.txt`); giải mã theo hàm
`decodeLunarYear` của tác giả (`oracle/de441/tk20.py`; kiểm tra liên tục giữa các năm). So ngày đầu (năm, tháng, nhuận)
của cả 1237 tháng: **1237/1237 khớp**.

**Kết luận L17 (phần bên kiểm định): ĐẠT** — không kém `Precise` (Precise: L9 theo A3 0 lệch, TK20 1237/1237).

## 6. Thông tin: DE441 so với oracle PyEphem cũ

Script `oracle/de441/compare_pyephem.py`, `compare_old_calendar.py`; kết quả `out/pyephem_vs_de441_stats.csv`,
`out/pyephem_vs_de441_dayflips.csv`, `out/old_oracle_vs_de441_days.txt`. Thời điểm PyEphem quy ra TT bằng
`ephem.delta_t` (PyEphem 4.2.1).

| | Sóc: max \|ΔTT\| | Tiết khí: max \|ΔTT\| | Sóc: max \|ΔUT\| | Tiết khí: max \|ΔUT\| |
|---|---|---|---|---|
| 1900–1999 | 1,98 s | 17,9 s | 2,2 s | 18,2 s |
| 2000–2099 | 1,82 s | 18,0 s | 105 s | 118 s |
| 2100–2899 | 2,0–3,3 s | 22 → 90 s (tăng đều) | 180–292 s | 200–367 s |
| 2900–3000 | 317 s (*) | 100 s | 587 s | 370 s |

(*) Từ Sóc 2950-01-09 trở đi chênh TT của Sóc nhảy ~ +278 s trong khi `ephem.delta_t` liên tục — tức hàm Sóc của PyEphem
dùng ΔT nội bộ khác `ephem.delta_t` từ 2950; đây là đặc điểm của oracle cũ, không phải của DE441.

- Chênh UT sau 2000 chủ yếu do ΔT (PyEphem ≠ MSHZ + nối §17.4); chênh TT của tiết khí tăng đều theo thời gian (mô hình
  Mặt Trời/tuế sai của PyEphem).
- Sự kiện đổi ngày lịch giữa PyEphem cũ và DE441 (cùng quy tắc múi giờ của oracle cũ): **112** (1900–1999: 1; 2000–2099: 3;
  còn lại sau 2100). Lịch: 1900–1999 lệch **1 ngày** (1951-12-22 cột tiết khí); 2000–3000 lệch 1270 ngày / 110 đoạn.
- **Các ngày sát nửa đêm còn tồn đọng trước đây:**
  - Đông Chí 1951: PyEphem 1951-12-22 23:59:56,7 UTC+8 (cách 3 s); DE441 **1951-12-23 00:00:01,533** UTC+8 (cách 1,5 s, ΔTT
    +4,5 s). Với DE441, tiết khí ngày 1951-12-22 là Đại Tuyết — trùng lõi `Precise` trước đây, khác oracle cũ. Sự kiện này
    cách nửa đêm < 2 s nên theo L15 thuộc loại "không phân định được".
  - Sóc 2072-12-09 (L3/L4 cũ: oracle 23:58:28, lõi `Precise` 00:00:14): DE441 đặt cùng ngày với oracle cũ (2072-12-09).
  - 2000–2099 các sự kiện đổi ngày: Sóc 2077-11-15/16, Sóc 2085-10-18/19, Xử Thử 2074-08-22/23 — đều do ΔT (\|ΔTT\| ≤ 1,5 s,
    ΔUT 66–82 s).
- Sự kiện của bên kiểm định cách nửa đêm lịch < 2 s trong 1900–3000: **2** — Đông Chí 1951-12-23 00:00:01,533 (+08:00) và
  Tiểu Tuyết 2341-11-22 23:59:59,342 (+07:00); < 10 s: 9 sự kiện (`out/events.csv.gz`, cột `secs_to_nearest_midnight`).

## 7. Sửa sau so sánh

Không có. Sau khi xem dữ liệu của bên xây dựng, bên kiểm định không sửa mã tính thời điểm, ΔT hay lịch. Các thay đổi sau
mốc `98688cc` chỉ là: script so sánh (`compare_builder.py`: dùng cột `n_cs` thay `jd_tt` vì chính xác hơn; ghi tệp nén;
cho phép chạy riêng L14), `compare_old_calendar.py` (thông tin), tác vụ `overlap` (kiểm điểm nối DE441) và báo cáo này.
`out/instants_tt.csv.gz` và `out/days_1900_3000.csv.gz` không đổi từ `98688cc` (SHA-256: `instants_tt.csv.gz` 130164ed…523a7a, `days_1900_3000.csv.gz` 162b911c…daf).

## 8. Giới hạn

- ΔT sau ~2050 là quy ước (§17.4); kết quả ngày sau 2100 đúng theo quy ước đó, không phải dự báo vật lý.
- Bảng ΔT dùng bản sao (máy chủ HMNAO không truy cập được); bản sao trùng nguyên văn 34 hàng in trong §17.4.
- Tệp `de441_part-1/2.bsp` không còn trên máy chủ JPL; dùng `de441.bsp` (cùng lời giải, nối tại 1969-07-30).
- Lệch hấp dẫn do Sao Mộc/Sao Thổ không áp dụng (§17.3 bước 2 cho phép).

## 9. Tệp

- Mã: `oracle/de441/{instants,deltat,calendar_de441,tk20,l17,compare_builder,compare_pyephem,compare_old_calendar,overlap_check}.py`;
  quy trình `.github/workflows/oracle-de441.yml` (tác vụ `fetch-refs`, `instants`, `overlap`, `builder-help`, `builder-dump`).
- Tài liệu nguồn đã tải: `oracle/de441/ref/`.
- Kết quả bên kiểm định: `oracle/de441/out/`; so sánh với bên xây dựng: `oracle/de441/cmp_builder/`;
  đầu ra hộp đen: `oracle/de441/builder/`.
