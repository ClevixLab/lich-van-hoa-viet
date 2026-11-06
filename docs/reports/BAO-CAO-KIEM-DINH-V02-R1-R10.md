# Báo cáo kiểm định độc lập v0.2 — R1 (ngày giỗ) và R10 (tuổi, Can Chi, 29/02)

Ngày lập: 2026-11-06. Người lập: bên không tham gia xây dựng đặc tả (kiểm định độc lập, phòng sạch).
Căn cứ: `docs/v0.2/01`, `02`, `04` (R1, R10), `06` (biên bản G1: R1 giữ ngưỡng **0 sai lệch 1900–3000, không có
ngoại lệ Sóc sát nửa đêm**, quyết định trước khi chạy R1; O-2 giỗ hết năm 2; O-3 hưởng dương < 60; O-4 nhuận →
tháng thường).
Không đọc mã nguồn `core/`, `ngochap/`, `tsport/`, `ffi/`, `apps/`, không đọc worktree `/home/claude/wt-*`. Chỉ đọc
**tệp dữ liệu**: âm lịch đã công bố của lõi (`core/out/precise/lunar_*.csv`, cho lớp R1-quy-tắc, như 04 cho phép)
và dữ liệu v0.2 của lõi trong `/home/claude/v02-data/core/`.

Trình tự (để kiểm được tính đăng ký trước): đầu ra oracle R1 (a)–(d) và R10 (a)–(d) cùng tổng băm
`oracle/v02/out/SHA256SUMS` lưu lúc **20:34 +07**. Dữ liệu lõi xuất hiện lúc **21:08** (R1) và **21:28** (R10).
Sau khi có dữ liệu lõi, oracle chỉ thêm phần (b) với `leap = true` (mục 3.1, ghi chú) và các công cụ so sánh; không
sửa quy tắc hay đầu ra đã lưu.

## 1. Kết luận

| Mã | Phần | Kết luận | Số liệu |
|---|---|---|---|
| **R1** | **Toàn tiêu chí** | **KHÔNG ĐẠT** | Lớp đầu-cuối có **2 932** dòng sai lệch trong 1900–3000. Theo 04 và quyết định G1 (06), chỉ một sai lệch ở bất kỳ lớp nào đã là KHÔNG ĐẠT |
| R1 | R1-quy-tắc (a), (b), (c) | 0 sai lệch | (a) 4 001 120 dòng; (b) 792 628; (c) 4 000 820; giống **từng byte** sau khi sắp (mục 4.1) |
| R1 | R1-đầu-cuối (a), (b), (c) | **2 932 sai lệch** | (a) 1 787; (b) 186; (c) 959. Phân loại tham khảo: **2 932/2 932 "Sóc sát nửa đêm"** (đoạn 2242, 2962, 2988), **0 "nguyên nhân khác"** |
| R1 | (d) ca cố định | 50/50 khớp, 2 không kiểm được | G-A…G-E và ca biên 02 §9 trích từ tệp (a)–(c) của lõi. Không kiểm được: `gio_het_nam = 3` (lõi không xuất tệp cho tùy chọn này); mất 1900-01-01, giỗ lần 11 (ngoài 10 năm, không thuộc dữ liệu của 04) |
| R1 | Rust ≡ TypeScript | **CHƯA ĐỦ BẰNG CHỨNG** | Chỉ nhận một bộ tệp. `README.txt` của lõi khẳng định "Rust và TypeScript cho cùng từng byte", nhưng bên kiểm định không có tệp TypeScript để so |
| **R10** | **(a)–(d)** | **ĐẠT** (dữ liệu lõi Rust đã nộp) | (a) 0/46 386; (b) 0/1 102; (c) 0/10 000; (d) 0/2 202; ví dụ 1990-01-20 → Kỷ Tỵ và 1940-05-01/2025-08-08 → 86 hưởng thọ đều đúng (mục 5) |

Ghi chú về kết luận R1:
- Nhóm sai lệch đúng như đã ghi trước ở 04 ("Rủi ro đã biết") và đúng như dự báo đăng ký trước của báo cáo này: số
  dòng lệch của lõi trùng **từng dòng** với khác biệt giữa hai lớp của chính oracle (mục 3.3, 4.2). Tức là **quy tắc
  giỗ của lõi khớp hoàn toàn**. Toàn bộ sai lệch đến từ 90 ngày âm lịch trong đó lõi (`Precise`) và oracle v0.1
  (PyEphem) đặt Sóc ở hai phía khác nhau của nửa đêm.
- Phân loại "sát nửa đêm" **không** làm thay đổi kết luận (06). Nếu chủ dự án muốn chấp nhận loại sai lệch này, thì
  theo 04 đó phải là một Bổ sung quyết định **trước** khi chạy R1. R1 đã chạy, nên mọi thay đổi bây giờ phải ghi là
  quyết định **sau khi thấy kết quả**.
- Câu hỏi bên nào đúng ở 3 đoạn này (lõi hay oracle) **không giải quyết được** bằng hai bản cài đặt hiện có. Chênh
  lệch thời điểm Sóc giữa hai bản khoảng 26 phút ở thế kỷ 30 (do ΔT và mô hình thiên văn), lớn hơn khoảng cách
  Sóc–nửa đêm (0,1′–4,9′). Kết luận phù hợp là TEST INADEQUATE (phép đo không đủ phân giải), không phải "lõi sai".

## 2. Cài đặt độc lập (`oracle/v02/`)

| Tệp | Nội dung |
|---|---|
| `giolib.py` | `save_death`, `anniversary`, `anniversaries_in`, `label_of`, `birth_derive` (`lunar_year`, `lunar_year_approx`, `can_chi`), `tuoi_mat`, `tuoi_song`, `solar_occurrence` (29/02) |
| `gen.py` | Sinh đầu ra R1 (a)–(d) cho hai lớp, R10 (a)–(d) |
| `gen_b_leap.py` | (b) với `leap = true` (thêm sau khi thấy lõi xuất cả hai giá trị cờ) |
| `selfcheck.py` | Kiểm lại toàn bộ đầu ra R1 bằng cách thứ hai (chỉ mục trực tiếp `(năm, tháng, ngày, nhuận)` → ngày dương; độ dài tháng = ngày lớn nhất trong bảng) |
| `layerdiff.py`, `classify.py` | So hai lớp của oracle, phân loại |
| `compare_v02.py`, `compare_stream.py` | So R1 với lõi: chuẩn hóa khóa, sắp ngoài (`sort`), trộn, liệt kê mọi sai lệch theo trường. Bản `stream` dùng cho tệp lớn (bản trong bộ nhớ bị hệ điều hành dừng vì hết bộ nhớ ở ~4 triệu dòng); hai bản cho cùng kết quả trên (b) (0 và 186) |
| `rerender.py` | Viết lại dòng oracle theo dạng `death` của lõi để so từng byte |
| `classify_cmp.py` | Phân loại sai lệch đầu-cuối → `out/r1_dau_cuoi_mismatch_classified.csv` |
| `extract_d.py` | Trích ca (d) từ tệp (a)–(c) của lõi |
| `compare_r10.py` | So R10 (a), (b), (d); (c) tính lại bằng oracle trên **đầu vào của lõi** |

Đổi lịch (S2L, L2S, LYOB, độ dài tháng) đọc từ bảng âm lịch theo ngày, không tính lại thiên văn:
- **lớp đầu-cuối:** bảng của oracle v0.1 (PyEphem 4.2.1; `oracle/oracle.py` cho 2000–3000 UTC+7,
  `oracle/oracle_1900.py` cho 1900–1999 theo quy tắc múi giờ lịch sử của oracle: UT+7:45:40 trước 1929, UTC+8
  tới 1968-01-01 00:00 UTC+7, sau đó UTC+7). Bảng giữ nguyên, không sinh lại.
- **lớp quy-tắc:** `core/out/precise/lunar_1900_1999.csv` + `lunar_2000_3000.csv`.

Quy tắc đã cài (02 §2.1, quyết định G1):
1. Giỗ là ngày âm của ngày mất, lặp mỗi năm âm Y lớn hơn năm mất; n = Y − năm âm mất.
2. Mất trong tháng nhuận → giỗ tháng **thường** cùng số. Năm có cả tháng m thường và m nhuận → giỗ tháng thường (O-4).
3. Mất ngày 30, mà tháng m thường của năm Y chỉ có 29 ngày → giỗ ngày 29.
4. Không rõ năm → giỗ mọi năm Y, n = null, nhãn `gio`.
5. Nhãn: n = 1 → `gio_dau`; n = `gio_het_nam` (mặc định 2, O-2) → `gio_het`; còn lại → `gio`.
6. Ngày giỗ có ngày dương ngoài 1900-01-01…3000-12-31 → bỏ (không lỗi).

Can Chi năm: cài độc lập theo mốc **1984 = Giáp Tý**. Năm nhuận Gregory: thử tạo ngày 29/02, không dùng công thức
chia hết. Tuổi: tuổi mụ `năm âm mất − năm âm sinh + 1`; nhãn `huong_duong` khi giá trị nhỏ hơn ngưỡng; giá trị < 1
→ lỗi.

**Giới hạn về tính độc lập:** oracle được viết từ cùng đặc tả 02 mà lõi dùng (02 có giả mã). Nếu chính đặc tả hiểu sai
phong tục, cả hai bên cùng sai và phép so không phát hiện được. Phần đó thuộc R9, không thuộc R1/R10. Phần đổi lịch
của lớp đầu-cuối thì độc lập thật (thiên văn khác, mã khác).

### 2.1 Tự kiểm của oracle (trước khi có dữ liệu lõi)

| Kiểm | Kết quả |
|---|---|
| Ví dụ 02 §2.4 G-A … G-E (11 dòng, gồm n và nhãn) | **11/11 khớp** ở cả hai lớp (G-D: 29/12/2024 → **2025**-01-28; G-E: 2028-06-07, không phải 2028-07-07 của tháng 5 nhuận) |
| `gio_het_nam = 3` | 2027 (n = 2) → `gio`, 2028 (n = 3) → `gio_het` |
| Ví dụ 02 §1, §4 | 2025-08-08 → 15/6 nhuận 2025; 1990-01-20 → 24/12/1989, Kỷ Tỵ; 1940/2025 → 86 hưởng thọ; tuổi dương 36, tuổi âm 38 (2026-11-06); 29/02/2000 → 28/02/2100 |
| `selfcheck.py` (cách thứ hai, toàn bộ (a)–(c), hai lớp) | **16 796 508 dòng, 0 khác** |
| Can Chi theo mốc 1984 so với công thức hợp đồng §3, 1899–3000 | 0/1 102 khác |

## 3. Đầu ra oracle (`oracle/v02/out/`)

Định dạng R1: mỗi dòng một đối tượng JSON gọn `{"death","Y","lunar":{y,m,d},"solar":{y,m,d},"n","label"}`, UTF-8.

### 3.1 R1

| Phần | Dữ liệu | Số dòng / lớp | Tệp |
|---|---|---|---|
| (a) | 402 132 ngày mất dương, Y = năm âm mất + 1…+10, bỏ giỗ ngoài miền | 4 001 120 | `r1_{dau_cuoi,quy_tac}_a.jsonl.gz` |
| (b) | 12 tháng × 30 ngày, không rõ năm, Y = 1900…3000; `leap` false và true | 396 314 + 396 314 | `r1_*_b.jsonl.gz`, `r1_*_b_leap.jsonl.gz`, gộp `r1_*_b_all.jsonl.gz` |
| (c) | 401 763 ngày âm tồn tại năm 1900–2999 (gồm tháng nhuận, ngày 30), 10 năm giỗ | 4 000 820 | `r1_*_c.jsonl.gz` |
| (d) | G-A…G-E, G-A với giỗ hết năm 3, ca biên 02 §9 | 54 | `r1_*_d.jsonl` |

Ghi chú (b): 396 314 = 360 × 1 101 − 46. 46 dòng bị bỏ là giỗ năm âm 3000 có ngày dương sau 3000-12-31. 04 ghi
"12 tháng × 30 ngày"; lõi xuất thêm `leap = true` (theo quy tắc 2 cho cùng kết quả). Oracle bổ sung phần này sau khi
thấy tệp lõi, không đổi quy tắc.

### 3.2 R10

| Phần | Dữ liệu | Số dòng | Tệp |
|---|---|---|---|
| (a) | ngày sinh dương 1900-01-01→2026-12-31 → `lunar_year`, `can_chi` (LYOB theo bảng oracle) | 46 386 | `r10_a.jsonl` |
| (b) | Can Chi năm 1899–3000 | 1 102 | `r10_b.jsonl` |
| (c) | bộ 10 000 cặp riêng của oracle (hạt 20261106). Lõi rút bộ khác nên **phép so dùng đầu vào của lõi** (mục 5) | 10 000 | `r10_c.jsonl` |
| (d) | 29/02 × Y 1900–3000 × {feb28, mar1} | 4 404 | `r10_d.jsonl` |

### 3.3 Dự báo đăng ký trước: khác biệt giữa hai lớp của chính oracle

Bảng oracle v0.1 và bảng `Precise` khác nhau đúng **90 ngày** trong 1900–3000 (2242-07-28→08-26, 2962-10-19→11-17,
2988-06-05→07-04; 1900–2241: 0 ngày). Cùng quy tắc, hai lớp khác nhau ở (a) 1 787, (b) 93 (cho mỗi giá trị `leap`),
(c) 959, (d) 0 dòng. Danh sách: `out/layerdiff_{a,b,c,d}.jsonl`. Báo cáo bản nộp lúc 20:34 đã ghi trước: *"nếu lõi
cài đúng quy tắc và đổi lịch bằng `Precise`, R1-quy-tắc sẽ 0 sai lệch, R1-đầu-cuối khoảng 2 839 sai lệch (chưa tính
(b) nhân đôi do `leap = true`)… R1 gần như chắc chắn KHÔNG ĐẠT"*.

## 4. R1 — so với lõi

Tệp lõi (sha256 trong `oracle/v02/out/SHA256SUMS.sau-doi-chieu`): `r1a_solar_1900_3000.jsonl.gz` (7114cb0d…),
`r1b_unknown_1900_3000.jsonl.gz` (5b07d729…), `r1c_lunar_1900_2999.jsonl.gz` (87c0cabb…),
`r1a_solar_1900_2099.jsonl.gz` (phần con, 730 490 dòng, 0 sai lệch so với phần tương ứng của oracle).
Lõi ghi trường `death` dạng `DateValue` (`calendar, precision, year, month, day, leap`). Oracle ghi dạng gọn
(`cal, y, m, d, leap`). Phép so: (i) chuẩn hóa ngữ nghĩa từng trường (`compare_stream.py`); (ii) viết lại dòng oracle
theo dạng của lõi rồi so từng byte sau khi sắp (`rerender.py`).

### 4.1 R1-quy-tắc (đổi lịch bằng bảng `Precise` đã công bố)

| Phần | Dòng lõi | Dòng oracle | Trùng khóa | Sai lệch | Từng byte sau khi sắp (sha256 hai bên) |
|---|---|---|---|---|---|
| (a) | 4 001 120 | 4 001 120 | 0 | **0** | giống: eec9ac46… |
| (b) | 792 628 | 792 628 | 0 | **0** | giống: 47199483… |
| (c) | 4 000 820 | 4 000 820 | 0 | **0** | giống: 75026a22… |

### 4.2 R1-đầu-cuối (đổi lịch bằng bảng oracle v0.1)

| Phần | Sai lệch | Trường khác | Năm giỗ Y | Phân loại |
|---|---|---|---|---|
| (a) | **1 787** | chỉ `solar` 870; `lunar` + `solar` 907; chỉ `lunar` 10 | 2242–2998 | 1 787 sát nửa đêm, 0 khác |
| (b) | **186** (93 × 2 giá trị `leap`) | `solar` 174; `lunar` + `solar` 6; `lunar` 6 | 2242, 2962, 2988 | 186 sát nửa đêm, 0 khác |
| (c) | **959** | `solar` 870; `lunar` + `solar` 19; `lunar` 10; ngày mất chỉ tồn tại ở một bên 30 + 30 | 2242–2998 | 959 sát nửa đêm, 0 khác |
| **Tổng** | **2 932** | | | **2 932 / 0** |

Theo đoạn: ngày giỗ rơi trong đoạn 2242: 658 dòng; 2962: 664; 2988: 662. Còn 948 dòng có ngày giỗ ở năm khác nhưng
ngày mất (dương, hoặc tháng âm) nằm trong đoạn, nên hai bảng cho ngày âm mất khác nhau.
Tập sai lệch đầu-cuối **trùng đúng** tập khác biệt giữa hai lớp oracle (mục 3.3) ở (a), (c), và ở (b) với cả hai giá
trị `leap`. Trường `n` và `label` không lệch ở dòng nào.

Tiêu chí phân loại (`classify.py`, đặt trước khi có dữ liệu lõi): "sát nửa đêm" khi ngày mất hoặc ngày giỗ dương
(bên nào cũng được) nằm trong 90 ngày lệch đã biết, hoặc tháng âm của ngày giỗ/ngày mất là một trong 6 tháng bị ảnh
hưởng (6–7/2242, 8–9/2962, 4–5/2988, đều là tháng thường). Ngược lại là "khác". Ba đoạn đều có Sóc cách 00:00 dưới
5 phút ở một trong hai bên (docs/reports/BAO-CAO-KIEM-DINH-LICH.md; BIEN-BAN-CONG-G1-G4.md L3).

**Danh sách đầy đủ, từng sai lệch:** `oracle/v02/out/r1_dau_cuoi_mismatch_classified.csv` (2 932 dòng: phần, ngày mất,
Y, trường khác, ngày âm/dương của oracle và của lõi, n, nhãn, phân loại). Bản JSON:
`oracle/v02/out/cmp_dau_cuoi_{a,b,c}_mismatch.jsonl`. Ví dụ:

| Phần | Ngày mất | Y | Oracle | Lõi |
|---|---|---|---|---|
| (a) | dương 2232-08-17 (= 1/7/2232) | 2242 (n = 10) | 1/7/2242 = 2242-07-29 | 1/7/2242 = 2242-07-28 |
| (b) | âm 30/4, không rõ năm | 2988 | 29/4/2988 = 2988-06-04 (tháng 4 thiếu) | 30/4/2988 = 2988-06-05 (tháng 4 đủ) |
| (b) | âm 1/5, không rõ năm | 2988 | 2988-06-05 | 2988-06-06 |
| (c) | âm 10/7/2232 | 2242 | 2242-08-07 | 2242-08-06 |

### 4.3 R1 (d) — ca cố định

Lõi không nộp tệp (d) riêng. Bên kiểm định trích từ (a)–(c) của lõi các dòng có cùng ngày mất và năm Y với 52 dòng
(d) của oracle (`gio_het_nam = 2`): **50 khớp, 0 khác**. Hai dòng không có trong dữ liệu lõi:
- mất 1900-01-01, Y = 1910 (n = 11): ngoài phạm vi 10 năm của 04 (do oracle tự thêm). Không phải sai lệch.
- G-A với `gio_het_nam = 3` (3 dòng): lõi không xuất dữ liệu cho tùy chọn này → **chưa kiểm** (02 §3, O-2 cho chọn 3).

## 5. R10 — so với lõi

| Phần | Phép so | Dòng lõi | Sai lệch |
|---|---|---|---|
| (a) | `lunar_year`, `can_chi` so với LYOB của **bảng oracle**, 1900-01-01→2026-12-31 | 46 386 | **0** |
| (b) | `can_chi` 1899–3000 so với Can Chi độc lập (mốc 1984) | 1 102 | **0** |
| (c) | Oracle tính lại `birth_lunar_year`, `approx`, `death_lunar_year`, `value`, `label` (và lỗi) trên **đầu vào của lõi** (`r10c.csv`: `birth`, `death`, `nguong`), đổi lịch bằng bảng oracle | 10 000 | **0** |
| (d) | 29/02 × Y 1900–3000 × {feb28, mar1} (oracle tính với ngày gốc 29/02 trước 1900) | 2 202 | **0** |

Chi tiết (c): 24 tổ hợp (sinh {dương đủ ngày, dương chỉ năm, âm đủ ngày, âm chỉ năm} × mất {dương, âm đủ ngày} ×
ngưỡng {59, 60, 61}), mỗi tổ hợp 416–417 cặp. 3 dòng lõi báo `DeathBeforeBirth` (sinh 1917-03-27 / mất
1917-01-10; sinh "1915" / mất 1915-01-04; sinh "1913" / mất 1913-01-06; năm âm mất nhỏ hơn năm âm sinh). Oracle cũng
báo lỗi ở đúng 3 dòng này. Kết quả của oracle không đổi khi đổi lịch bằng bảng `Precise` (0 dòng khác), vì mọi năm
trong (c) nằm trong 1900–2108, nơi hai bảng trùng nhau (hai bảng chỉ khác từ 2242).
Ví dụ: 1990-01-20 → 1989 Kỷ Tỵ, 1940-05-01 → 1940 Canh Thìn (`r10a.csv`). Ca 1940-05-01/2025-08-08 → 86
`huong_tho` có trong `r3-dataset.json` của lõi. Oracle tính lại mọi trường dẫn xuất ngày sinh/ngày mất/tuổi của
tệp đó: 35/35 khớp.

Ghi nhận (không thuộc ngưỡng R10): 3 110/10 000 cặp của lõi có năm mất (theo trường `year` của đầu vào) sau 2026, tức ngày mất ở tương lai
(tới năm 2108). Theo 01 §3.2, ngày mất
ở tương lai là lỗi nhập liệu. R10(c) chỉ kiểm công thức tuổi nên không ảnh hưởng kết luận. Nếu bộ này được dùng lại
cho R2 thì phải lọc.

Kết luận R10: **ĐẠT** trên dữ liệu lõi đã nộp. Ngưỡng R10 không yêu cầu Rust ≡ TypeScript. Bên kiểm định không nhận
tệp TypeScript riêng.

## 6. Còn thiếu / việc tiếp theo

- R1 Rust ≡ TypeScript: cần tệp R1 do bản TypeScript sinh ra (hoặc băm của chúng kèm lệnh sinh) để so độc lập.
- R1 (d) với `gio_het_nam = 3`: cần lõi xuất ca G-A với tùy chọn này.
- R1 KHÔNG ĐẠT chỉ thay đổi được bằng (i) quyết định của chủ dự án về loại "Sóc sát nửa đêm", ghi rõ là **sau khi
  thấy kết quả**, hoặc (ii) đổi phương pháp đo/bảng tham chiếu. Cả hai đều là Bổ sung cần chủ dự án ký. Bên kiểm định
  không đề xuất nới ngưỡng.

## 7. Tệp

- Oracle: `oracle/v02/*.py`; đầu ra `oracle/v02/out/r1_*`, `r10_*`; tổng băm trước đối chiếu `out/SHA256SUMS` (20:34),
  sau đối chiếu `out/SHA256SUMS.sau-doi-chieu` (gồm cả băm tệp lõi).
- Đối chiếu: `out/cmp_{quy_tac,dau_cuoi}_{a,b,c}.json` và `_mismatch.jsonl`, `out/cmp_a2099_quy_tac.json`,
  `out/cmp_r10.json`, `out/cmp_r10_mismatch.jsonl` (rỗng), `out/r1_dau_cuoi_mismatch_classified.csv`.
- Không chạy: R3, R4, R11, R12 (tệp lõi có trong `/home/claude/v02-data/core/` nhưng ngoài phạm vi báo cáo này).

Máy đo: Linux x86_64, 2 vCPU, 7 GB RAM; Python 3, PyEphem 4.2.1 (chỉ dùng bảng v0.1 đã lưu). Kho chưa có commit nên
không có mã commit để neo; tệp lõi được neo bằng sha256.
