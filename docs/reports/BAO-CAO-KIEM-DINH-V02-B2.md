# Báo cáo kiểm định độc lập v0.2 — Bổ sung B2 (R1 với Jpl, R12 chạy lại, R13, R3/R11, Rust ≡ TypeScript)

Ngày lập: 2026-11-07. Người lập: bên không tham gia xây dựng đặc tả (kiểm định độc lập, phòng sạch).
Căn cứ: `docs/v0.2/01`, `02` (bản trên `main` 93cc6c3; riêng §7.6 và Phụ lục A.6 đọc tại commit `206fceb` của nhánh
`v0.2-w3-core`), `04` (R1, R3, R11, R12, Bổ sung B2 và R13), báo cáo trước `BAO-CAO-KIEM-DINH-V02-R1-R10.md`.
Không đọc mã nguồn `memorial/`, `store/`, `backup/`, `core/`, `tsport/`, `ffi/`, `pwa/`. Bộ dữ liệu R3
`memorial/testdata/r3-dataset.json` chỉ được đọc như **dữ liệu** (vị trí theo 02 Phụ lục A.2). Mọi trường dẫn xuất
trong tệp (ngày âm của ngày mất, ngày dương/âm của ngày sinh) được oracle **tính lại** từ `input`, không dùng giá trị
có sẵn. Công cụ của lõi (`lvhv-memorial-dump`, `tsport/test/memorial_dump.mjs`) chỉ được dựng và chạy như hộp đen
(`--help`, `cargo build`, `npm ci`, `npm run build`).

Kết quả `Precise` của R1 lần trước (KHÔNG ĐẠT, 2 932 sai lệch) **giữ nguyên** trong báo cáo cũ, không sửa.

## 1. Kết luận (lần B2)

**Cập nhật sau Bổ sung B3: xem mục 10.** Sau B3, R3 chuyển từ CHƯA ĐỦ BẰNG CHỨNG sang ĐẠT; R12 ĐẠT với I1–I4; R13 ĐẠT
trên cả bộ R3 và bộ `r13-linked`.

| Mã | Phần | Kết luận | Số liệu |
|---|---|---|---|
| **R1 (Jpl)** | **Toàn tiêu chí** | **ĐẠT** | 0 sai lệch ở (a)–(d); Rust ≡ TS từng byte. Không dùng ngoại lệ nào (mục 4) |
| R1 (Jpl) | (a) | 0 / 4 001 120 dòng | sha256 sau khi sắp trùng nhau: `6b7dd80a…` |
| R1 (Jpl) | (b) | 0 / 792 628 dòng (`leap` false và true) | `c6ec0609…` |
| R1 (Jpl) | (c) | 0 / 4 000 820 dòng (401 763 ngày mất âm) | `d2434c5c…` |
| R1 (Jpl) | (d) | 54 / 54 khớp | Gồm G-A với `gio_het_nam = 3` (3 dòng) và ca mất 1900-01-01, giỗ lần 11. Lần trước hai ca này chưa kiểm được; nay kiểm được nhờ `--gio-het 3` và `gio-dates list` |
| R1 (Precise, lần trước) | — | KHÔNG ĐẠT (giữ nguyên) | 2 932 sai lệch, `BAO-CAO-KIEM-DINH-V02-R1-R10.md` |
| **R12** | 9 ô (3 cỡ × 3 kịch bản) | **ĐẠT** cả 9 ô | 0 vi phạm I1/I2/I3 ở mọi ô, cả phía lõi (`v0.2-w3-core` 2eea0bb, PR #14) lẫn oracle (mục 6). Có một phát hiện nằm ngoài các bất biến (mục 6.3) |
| **R13** | (a) | **ĐẠT** | 0 sai lệch: 2 286 / 2 286 dòng, giống **từng byte** (sha256 `a950787a…`) |
| R13 | (b) Rust ≡ TS | **ĐẠT** | Bên kiểm định chạy cả hai bản: cùng sha256 `a950787a…` (cả với `--method jpl` lẫn mặc định) |
| R13 | (c) nhất quán | **ĐẠT** | 0 vi phạm: 561 phép "hợp kỳ con = kỳ cha" (tuần→tháng, tháng→quý, quý→năm, × 3 bộ lọc); 762 kỳ có `da_mat` ∪ `dang_song` = `tat_ca` (rời nhau, giữ thứ tự); 0 mục trùng |
| R13 | (d) ngày đầu/cuối | **ĐẠT** | 0 vi phạm ở 762 kỳ; 88 kỳ có mục đúng ngày đầu, 97 kỳ có mục đúng ngày cuối, nên phép kiểm không rỗng |
| R13 | Toàn tiêu chí | **ĐẠT** | Giới hạn của dữ liệu: mục 7.3 |
| **R11** | Toàn tiêu chí | **ĐẠT** (kèm một lần sửa oracle **sau đối chiếu**, mục 5) | 13 691 dòng, 4 018 "hôm nay". Trước khi sửa: 1 023 dòng khác, **chỉ** ở trường `label` của dòng tiên thường. Sau khi sửa: 0 (giống từng byte) |
| **R3** | Đối chiếu tập lời nhắc | 0 sai lệch | 140 544 cấu hình (giống từng byte); 38 432 256 lời nhắc (sha256 trùng `86bcc3d2…`) |
| R3 | Toàn tiêu chí | **CHƯA ĐỦ BẰNG CHỨNG** | Hai mục "Thêm" của 04 R3 không kiểm được qua dump: tắt công tắc chung → 0, và `rule.enabled = false` → 0. Hai mục "Thêm" còn lại kiểm được (mục 8) |
| **Rust ≡ TS** (memorial) | 16 lệnh × 2 nhánh, thêm `r13` | **ĐẠT** | `main` 93cc6c3: 16/16 giống; `v0.2-w3-core` 2eea0bb: 18/18 giống |

## 2. Trình tự làm việc (để kiểm được tính đăng ký trước)

| Giờ (+07) | Commit (nhánh `oracle-v02-b2`) | Việc |
|---|---|---|
| 02:42 | — | Dựng `lvhv-memorial-dump` từ `main` 93cc6c3 (chưa chạy lệnh nào ngoài `--help`) |
| 02:46 | `83110a1` | Đầu ra oracle R1 Jpl (a)–(d), tổng băm `out/SHA256SUMS.b2-r1`, tự kiểm cách thứ hai. Làm **trước** khi chạy dump R1 của lõi |
| 02:47–02:48 | — | Chạy dump R1 của lõi (`--method jpl`), so tổng băm sau khi sắp; so R1(d) |
| 02:49 | `bf73147` | Đầu ra oracle R11, R12, mã R3 (`b2_r3.py`) và cấu hình R3. Làm **trước** khi chạy `upcoming`, `r12`, `reminders` của lõi để so |
| 02:49 | — | Đọc §7.6 + A.6 tại `206fceb` (sau khi có tệp báo sẵn sàng) |
| 02:50 | `ebe5248` | Đầu ra oracle R13 (2 286 dòng) và tự kiểm (c), (d). Làm **trước** khi có dump `r13` của lõi |
| 02:51–02:54 | `650151f` | So R11, R3, cấu hình R3; chạy Rust ≡ TS trên `main`; sửa oracle R11 **sau đối chiếu** (mục 5) |
| 03:07 | — | PR #14 (`v0.2-w3-core`, 2eea0bb) xuất hiện. Dựng dump từ commit này, chạy `r12`, `r13`, so; chạy Rust ≡ TS lần hai |
| sau 03:10 | commit báo cáo | Biến thể R12 sau đối chiếu (mục 6.2) và báo cáo này |

Ghi chú về trình tự R3: bảng Rust ≡ TS (chạy nền từ 02:48) in sha256 đầu ra lệnh `reminders` của lõi vào khoảng 02:51,
khi oracle R3 còn đang sinh. Mã sinh R3 của oracle đã nằm trong commit `bf73147` (02:49) và không sửa sau đó. Lần sửa
`b2_occ.py` ở mục 5 chỉ đổi nhãn tiên thường; R3 không dùng trường này, và tiến trình R3 đã nạp mã trước lần sửa.
Tổng băm đầu ra R3 của oracle được ghi ở `650151f`.

## 3. Cài đặt độc lập (`oracle/v02/`, phần thêm trong B2)

| Tệp | Nội dung |
|---|---|
| `giolib.py` | Thêm nguồn lịch `de441` = `oracle/de441/out/days_1900_3000.csv.gz` (lịch tự lập từ DE441; L15 khớp lõi 402 132/402 132 ngày). Quy tắc giỗ không đổi |
| `b2_gen_r1.py`, `b2_selfcheck_r1.py` | R1 Jpl (a)–(d), dòng viết thẳng theo 02 A.1; tự kiểm bằng cách thứ hai |
| `b2_occ.py` | Bộ sinh lần xuất hiện dùng chung cho R3/R11/R12/R13 (giỗ, tiên thường, sinh nhật dương/âm, sự kiện) và `reminders_for` (§7.1) |
| `b2_r3.py`, `b2_r11.py`, `b2_r12.py`, `b2_r13.py` | R3 (A.2), R11 (A.3), R12 (§7.4 đã sửa, A.5), R13 (§7.6, A.6) và kiểm (c), (d) |
| `b2_run_core_r1.sh`, `b2_cmp_r1d.py`, `b2_cmp_r12.py`, `b2_parity.sh`, `b2_sums.sh` | Chạy dump của lõi (hộp đen), so R1(d), so R12, so Rust ≡ TS, tổng băm |

Ở những chỗ đặc tả chưa nói rõ, oracle đã chọn như sau (ghi trong docstring `b2_occ.py`, trước khi so):
- Người đã mất chỉ có giỗ, cộng tiên thường khi quy tắc giỗ có `tien_thuong = true` (kể cả khi `enabled = false`).
- Sinh nhật dương tính từ năm sinh + 1.
- Sinh nhật âm và sự kiện âm lặp tính theo `anniversary` (§2.2).
- Sự kiện dương lặp tính từ năm gốc; 29/02 → 28/02.
- Sự kiện một lần nhập theo âm được đổi sang dương, có tính cờ nhuận.
- Cùng ngày thì sắp theo hạng lớp, rồi tên (hồ sơ: `ho_ten`; sự kiện: `tieu_de`, rỗng thì tên loại) so theo điểm mã
  sau NFC, rồi lớp, id, khóa năm.

## 4. R1 với phương pháp Jpl (Bổ sung B2 mục 3)

Phương pháp:
- Oracle dùng quy tắc giỗ của lần trước (không sửa) và đổi lịch bằng **lịch DE441 của bên kiểm định**.
- Lõi chạy `lvhv-memorial-dump --method jpl gio-dates solar 1900-01-01 3000-12-31 10`, `unknown 1900 3000` và
  `lunar 1900 2999 10`, dựng từ `main` 93cc6c3. Bản 2eea0bb cho cùng sha256 đầu ra.
- So sha256 của toàn bộ dòng sau khi giải nén và sắp (`LC_ALL=C sort`).
- Ngưỡng: 0 sai lệch, **không** có ngoại lệ Sóc sát nửa đêm.

| Phần | Dòng oracle | Dòng lõi | sha256 sau khi sắp (hai bên trùng) | Sai lệch |
|---|---|---|---|---|
| (a) | 4 001 120 | 4 001 120 | `6b7dd80a9dccebce…` | **0** |
| (b) | 792 628 | 792 628 | `c6ec0609a26e1277…` | **0** |
| (c) | 4 000 820 | 4 000 820 | `d2434c5cff1d0874…` | **0** |
| (d) | 54 | 54 (từ `gio-dates list`, `--gio-het 2` và `3`) | so từng dòng theo (ngày mất, Y) | **0** |

Tự kiểm của oracle trước khi so:
- Cách thứ hai (chỉ mục trực tiếp (năm, tháng, ngày, nhuận) → ngày dương) trên toàn bộ 8 794 568 dòng (a)–(c):
  0 khác.
- 11 ví dụ của 02 §2.4: 11/11 khớp.

Về hai lớp của R1: với Jpl, "lớp đầu-cuối" (bảng tham chiếu độc lập) là lịch DE441 của bên kiểm định. Lịch này trùng
lịch Jpl của lõi từng ngày (L15), nên lớp "quy tắc" và lớp "đầu-cuối" dùng cùng một bảng và một lần so là đủ cho cả
hai. Ba đoạn 2242/2962/2988 không còn gây sai lệch vì cả hai bên cùng dùng DE441.

**Kết luận R1 (Jpl): ĐẠT.** Quyết định chạy lại R1 bằng Jpl là quyết định **sau khi thấy kết quả** (04 Bổ sung B2
mục 3 tự ghi như vậy). Báo cáo này chỉ đo theo quyết định đó. Kết quả `Precise` (KHÔNG ĐẠT) vẫn đúng cho phương pháp
đó.

## 5. R11 — danh sách "Sắp tới" 30 ngày (Jpl)

- Lõi: `upcoming r3-dataset.json 2026-01-01 2036-12-31` (`main` 93cc6c3; bản 2eea0bb cho cùng sha256).
- Oracle: `b2_r11.py`, bản nộp `out/b2_r11_upcoming.csv` (`bf73147`).
- Hai bên có cùng 13 691 dòng, cùng tập, cùng thứ tự, và cùng các trường `days`, `date`, `class`, `entity_id`, `key`,
  `n`.

**Sai lệch trước khi sửa: 1 023 dòng.** Tất cả là dòng `class = tien_thuong` và chỉ khác trường `label`: oracle ghi
`tien_thuong`, lõi ghi nhãn của giỗ hôm sau (cả 1 023 dòng đều là `gio`). Danh sách đầy đủ tái tạo được bằng
`diff out/b2_r11_upcoming.csv out/b2_r11_upcoming_sau_sua.csv`; hai tệp chỉ khác nhau ở đúng 1 023 dòng này. Ví dụ:
`2026-01-16,7,30,2026-02-15,tien_thuong,a0000000-0000-4000-8000-000000000016,2025,15,…`.

**Sửa oracle sau đối chiếu** (`b2_occ.py`, có ghi chú trong mã): nhãn của lần xuất hiện tiên thường = nhãn của giỗ hôm
sau.
- Căn cứ: 02 A.3 không định nghĩa nhãn cho tiên thường. 02 §7.6 ghi "`label`: nhãn giỗ … cho giỗ và cho tiên
  thường" và "`n` như A.3".
- Bên kiểm định đã đọc §7.6 lúc 02:49, tức trước khi so R11 lúc 02:51, nhưng chưa sửa bản nộp R11. Vì vậy lần sửa này
  phải tính là **sau đối chiếu**.
- Ảnh hưởng: R11 sau khi sửa giống lõi **từng byte** (`out/b2_r11_upcoming_sau_sua.csv`). R13 không đổi, vì đã tính
  nhãn tiên thường đúng theo §7.6 từ đầu. R3 và R12 không dùng trường này.

Ngày đầu và ngày thứ 30 đều có mặt: 441 dòng `days = 0`, 442 dòng `days = 30`; giá trị `days` lớn nhất là 30.

**Kết luận R11: ĐẠT**, với điều kiện chủ dự án chấp nhận rằng trường `label` của tiên thường trong A.3 được hiểu theo
§7.6. Nếu không chấp nhận lần sửa sau đối chiếu, bản nộp ban đầu có 1 023 sai lệch (chỉ ở trường này).

## 6. R12 — chạy lại với `sentinel.at` = `at` của lời nhắc thứ 63 (Bổ sung B2 mục 1)

Oracle (`b2_r12.py`, commit `bf73147`, có trước PR #14):
- Dữ liệu: bộ R3 (27 hồ sơ rồi 13 sự kiện). Bản ghi i = bản ghi (i mod 40); mọi id trong bản ghi (thực thể và quy
  tắc nhắc) đổi 8 ký tự đầu thành i dạng hex.
- Mỗi ngày mở ứng dụng lúc 09:00. Mỗi lần mở:
  1. giao các thông báo đã đến hạn;
  2. chạy `reschedule`: lấy các lời nhắc trong cửa sổ (now, now + 400 ngày], sắp theo (at, id); nếu nhiều hơn 64 thì
     giữ 63 cái sớm nhất cộng thông báo nhắc mở ứng dụng tại `at₆₃`;
  3. kiểm I1, I2, I3 trực tiếp trên tập đang chờ.

Lõi: `r12 DATASET --steps`, dựng từ PR #14 (2eea0bb).

### 6.1 Từng ô

| Cỡ / kịch bản | Lần mở (lõi / oracle) | Chờ tối đa | Vi phạm lõi | Vi phạm oracle | Bỏ lỡ không có thông báo nhắc mở ứng dụng (lõi / oracle) | Kết luận |
|---|---|---|---|---|---|---|
| 10 / p1 | 1 096 / 1 096 | 37 / 37 | 0 | 0 | 0 / 0 | **ĐẠT** |
| 10 / p0.1 | 100 / 99 | 36 / 35 | 0 | 0 | 0 / 0 | **ĐẠT** |
| 10 / p0 | 1 / 1 | 33 / 33 | 0 | 0 | 38 / 38 (ngoài cửa sổ 400 ngày, mục 6.3) | **ĐẠT** |
| 50 / p1 | 1 096 / 1 096 | 64 / 64 | 0 | 0 | 0 / 0 | **ĐẠT** |
| 50 / p0.1 | 100 / 99 | 64 / 64 | 0 | 0 | 0 / 0 | **ĐẠT** |
| 50 / p0 | 1 / 1 | 64 / 64 | 0 | 0 | 0 / 0 | **ĐẠT** |
| 200 / p1 | 1 096 / 1 096 | 64 / 64 | 0 | 0 | 0 / 0 | **ĐẠT** |
| 200 / p0.1 | 100 / 99 | 64 / 64 | 0 | 0 | 0 / 0 | **ĐẠT** |
| 200 / p0 | 1 / 1 | 64 / 64 | 0 | 0 | 0 / 0 | **ĐẠT** |

Khối `--steps` của lõi có tổng vi phạm 0 ở mọi ô. So từng bước (số lời nhắc tương lai, số đang chờ, có hay không
thông báo nhắc mở ứng dụng, số bị bỏ, số vi phạm) ở những ngày mở có ở cả hai bên:
- p1 và p0: giống hoàn toàn (1 096 bước và 1 bước cho mỗi cỡ).
- p0.1: bản nộp chỉ có 6 ngày mở trùng với lõi, và cả 6 bước đó giống nhau.

Lần chạy trước (dump của `main`, sentinel đặt `+ 1 phút`) có vi phạm ở 5 ô: 363 (50-p1), 33 (50-p0.1), 1 096 (200-p1),
100 (200-p0.1), 1 (200-p0). Nay cả 5 ô đều 0.

### 6.2 Khác biệt ngoài bất biến (không thuộc ngưỡng), và một biến thể sau đối chiếu

Danh sách đầy đủ các ô khác nhau ở bảng tổng hợp (`out/b2_cmp_r12.json`):
- `opens` ở p0.1: lõi 100, oracle 99 (cả ba cỡ). Kéo theo khác ở `max_pending` 10-p0.1 (36 / 35), `delivered` 200-p0.1
  (967 / 949) và `missed` 200-p0.1 (161 / 176).
- `delivered` ở p0, cỡ 50 và 200: lõi 64, oracle 63.

Nguyên nhân (xác định **sau đối chiếu**):
1. A.5 ghi "ngày đầu luôn mở; p0.1 dùng mulberry32 … một số mỗi ngày" nhưng không nói ngày đầu có rút số hay không.
   Oracle rút số cả ở ngày đầu; lõi bắt đầu rút từ ngày thứ hai.
2. Lõi tính thông báo nhắc mở ứng dụng vào `delivered` (64 = 63 + 1); oracle tính riêng nó ở `sentinel_delivered`.

Biến thể sau đối chiếu: cờ `--rut-tu-ngay-2`. Biến thể này **không** thay bản nộp `b2_r12.csv`; đầu ra ở
`out/b2_r12_bien_the_rut_tu_ngay_2.csv`, kết quả so ở `out/b2_cmp_r12_bien_the.json`. Với biến thể này:
- mọi cột và mọi bước của khối `--steps` giống lõi ở cả 9 ô (ở p0.1: 100/100 bước chung);
- chỉ còn `delivered` lệch, và lệch đúng bằng `sentinel_delivered` (50-p0: 1; 200-p0.1: 3; 200-p0: 1), tức là khác
  định nghĩa cột.

Kết luận về vi phạm không đổi (0) ở cả hai cách rút số.

### 6.3 Phát hiện ngoài bất biến (ghi để chủ dự án quyết, không đổi kết luận)

Ở ô 10 / p0 (mở ứng dụng một lần rồi không mở lại), **38 lời nhắc** trong 2027–2028 không bao giờ được đặt và không có
thông báo nhắc mở ứng dụng nào. Điều này xảy ra ở cả lõi lẫn oracle.
- Lý do: 38 lời nhắc này nằm ngoài cửa sổ 400 ngày của `reschedule`. Số lời nhắc trong cửa sổ là 33, nhỏ hơn 64, nên
  thuật toán không đặt thông báo nhắc mở ứng dụng.
- I3 chỉ nói về lời nhắc bị bỏ do giới hạn 64. Vì vậy đây **không** phải vi phạm theo bất biến đã đăng ký, và R12 ĐẠT
  theo ngưỡng.
- Tuy vậy, mệnh đề R12 viết "không bỏ lời nhắc tương lai một cách lặng lẽ", và với người không mở ứng dụng hơn 400 ngày
  thì 38 lời nhắc này bị bỏ lặng lẽ.

Nếu muốn mệnh đề đúng cả trong trường hợp này thì cần sửa đặc tả, ví dụ luôn đặt thông báo nhắc mở ứng dụng tại
`now + 400 ngày` khi vẫn còn lời nhắc ngoài cửa sổ. Đó là quyết định của chủ dự án; vì kết quả đã được thấy, quyết định
đó phải ghi là quyết định sau khi thấy kết quả.

## 7. R13 — danh sách theo tuần / tháng / quý / năm (mới)

Đặc tả đã đọc: 04 R13 (Bổ sung B2 mục 4), và 02 §7.6 + Phụ lục A.6 tại `206fceb`.
- Oracle `b2_r13.py` dùng chính bộ sinh lần xuất hiện độc lập của R3/R11/R12 (`b2_occ.py`) và lịch DE441.
- Kỳ tính theo `period_bounds` (tuần từ Thứ Hai đến Chủ nhật). Lấy mọi tuần/tháng/quý/năm có ít nhất một ngày trong
  2026-01-01 → 2036-12-31: 575 + 132 + 44 + 11 = 762 kỳ, × 3 bộ lọc = 2 286 dòng.
- Oracle nộp ở `ebe5248` (02:50). Dump `r13` của lõi chỉ có từ 2eea0bb (03:06).

### 7.1 (a) Đối chiếu và (b) Rust ≡ TS

Tệp lõi `lvhv-memorial-dump --method jpl r13 r3-dataset.json` (2eea0bb) và tệp oracle `out/b2_r13.jsonl` **giống từng
byte**, sha256 `a950787aacaecd8791684890724886e07096a5646f00f05cfde8410017f47e6a`. Có 0 sai lệch, và oracle R13 không
được sửa gì sau đối chiếu. Sau lần sửa nhãn tiên thường ở mục 5, tệp oracle sinh lại vẫn giống từng byte bản nộp, vì
R13 đã tính nhãn tiên thường theo §7.6 từ đầu.

Bản TypeScript (`tsport/test/memorial_dump.mjs`, dựng từ 2eea0bb) cho cùng sha256, nên (b) ĐẠT.

### 7.2 (c), (d) trên đầu ra của oracle (lặp lại trên tệp lõi: cùng kết quả)

Chạy `b2_r13.py --check`; kết quả ở `out/b2_r13_check_oracle.json` và `out/b2_r13_check_core.json`.

(c) Nhất quán:
- Hợp các tuần (cắt theo ranh giới tháng) = tháng; hợp 3 tháng = quý; hợp 4 quý = năm. Mỗi phép làm cho cả 3 bộ lọc và
  so cả thứ tự: 561 phép, 0 khác.
- `da_mat` ∩ `dang_song` = ∅, và `da_mat` ∪ `dang_song` = `tat_ca` với thứ tự giữ nguyên: 762 kỳ, 0 khác.
- Mục trùng trong một dòng: 0.

(d) Ngày đầu và ngày cuối:
- Với mỗi kỳ, các mục của ngày `from` đúng bằng danh sách của riêng ngày đó; tương tự với ngày `to`.
- Hợp danh sách của từng ngày trong kỳ = danh sách của kỳ.
- 762 kỳ, 0 khác. Phép kiểm không rỗng: 88 kỳ có mục ở ngày đầu, 97 kỳ có mục ở ngày cuối. Ví dụ, sự kiện "Mốc cuối
  cửa sổ" ngày 2036-12-31 nằm ở ngày cuối của tháng 12, quý 4 và năm 2036.

### 7.3 Giới hạn của dữ liệu R13

Trong bộ R3, mọi sự kiện có `person_ids` rỗng. Vì vậy dữ liệu **không kiểm tới** nhánh "sự kiện gắn với người đã mất"
của bộ lọc `da_mat`, cũng như ca sự kiện gắn với nhiều người có trạng thái khác nhau: mọi sự kiện đều rơi vào
`dang_song`. Theo 04, R13 dùng đúng bộ R3, nên kết luận vẫn là ĐẠT. Nhưng quy tắc lọc sự kiện của §7.6 (đặc tả tự ghi
"chờ chủ dự án xác nhận") mới được kiểm ở một nhánh. Tổng số mục theo năm qua 11 năm: `da_mat` 218 (giỗ, tiên thường),
`dang_song` 223.

## 8. R3 — sinh lời nhắc, 140 544 cấu hình (Jpl)

- Lõi: `reminders configs` và `reminders r3-dataset.json` (`main` 93cc6c3; bản 2eea0bb cho cùng sha256).
- Oracle: `b2_r3.py` (mã nằm trong `bf73147`).

| Phần | Oracle | Lõi | Sai lệch |
|---|---|---|---|
| Cấu hình | 140 544 dòng, 3 584 tập `offsets` khác nhau | 140 544 | **0** (giống từng byte) |
| Lời nhắc | 38 432 256 dòng | 38 432 256 | **0** (sha256 toàn tệp `86bcc3d2f08822b0…` trùng nhau) |

Bốn mục "Thêm" của 04 R3:
- Kiểm được gián tiếp qua đối chiếu (0 sai lệch):
  - "lời nhắc có thời điểm ≤ now không được sinh": cả hai bên lọc `at > now`;
  - "id ổn định khi đổi `time_local`": id của oracle không chứa giờ.
- **Không kiểm được:**
  - "tắt công tắc chung → 0": dump `reminders` cố định `reminders_enabled = true`;
  - "`rule.enabled = false` → 0": bộ R3 không có quy tắc nào `enabled = false`.

04 R3 đặt cả bốn mục này trong tiêu chí, và chỉ được ghi ĐẠT khi đo đủ. Vì vậy **kết luận R3: CHƯA ĐỦ BẰNG CHỨNG**.
Còn thiếu hai mục không kiểm được ở trên; cần một lệnh dump, hoặc một bộ dữ liệu có công tắc chung tắt và quy tắc tắt.

## 9. Rust ≡ TypeScript (phía bên kiểm định)

Hai bản dump chạy như hộp đen với cùng lệnh; so sha256 của đầu ra. Mỗi lệnh chạy hai lần: có `--method jpl` và dùng
mặc định.

| Nhánh | Lệnh | Kết quả |
|---|---|---|
| `main` 93cc6c3 | `gio-dates solar`, `unknown`, `lunar` (1900–3000); `--gio-het 3 gio-dates solar` (1900–3000); `reminders configs`; `reminders DATASET`; `upcoming DATASET 2026-01-01 2036-12-31`; `r12 DATASET --steps` | 16/16 giống (`out/b2_parity_main_93cc6c3.tsv`) |
| `v0.2-w3-core` 2eea0bb | Như trên, thêm `r13 DATASET` | 18/18 giống (`out/b2_parity_w3_2eea0bb.tsv`) |

- Giữa hai nhánh, chỉ đầu ra `r12` thay đổi (do sửa sentinel). Mọi đầu ra R1, R3, R11 giống nhau, nên kết quả đối chiếu
  R1/R3/R11 ở trên cũng áp dụng cho 2eea0bb.
- Mặc định của cả hai bản là Jpl: đầu ra có và không có `--method jpl` trùng nhau.
- Phần "Rust ≡ TypeScript" của R1, lần trước ghi CHƯA ĐỦ BẰNG CHỨNG, nay **ĐẠT** cho Jpl.

## 10. Kiểm định lại sau Bổ sung B3 (2026-11-07)

Căn cứ: 04 Bổ sung B3 (`main` 433f4d8, PR #17); 02 §7.4, §7.6, Phụ lục A.3, A.5, A.6 tại `181c039` (PR #14). Không
đọc mã của lõi. Bộ dữ liệu bổ sung `memorial/testdata/r13-linked.json` chỉ được đọc như dữ liệu; bản đã dùng được chép
nguyên vào `oracle/v02/out/b3_input_r13-linked_181c039.json`. Các mục 1–9 và 11 giữ nguyên làm hồ sơ của lần B2.

### 10.1 Kết luận

| Mã | Kết luận | Số liệu |
|---|---|---|
| **R12** (§7.4 mới, I1–I4) | **ĐẠT** cả 9 ô | 0 vi phạm I1, I2, I3, I4 ở mọi ô, cả lõi (181c039) lẫn oracle. Bảng tổng hợp giống lõi ở **mọi cột** của 9 ô; khối `--steps` giống ở mọi bước (1 096 / 100 / 1 bước mỗi cỡ) |
| **R3** (thêm `--global-off`, `--rule-off`) | **ĐẠT** | Cả hai chế độ: 0 lời nhắc ở cả hai bên (tệp chỉ có dòng tiêu đề, giống từng byte). Phần đối chiếu đầy đủ không đổi: 0 / 38 432 256, sha256 `86bcc3d2…` trùng ở 181c039. Bốn mục "Thêm" nay đều đã kiểm |
| **R13** bộ `r13-linked` | **ĐẠT** (a), (b), (c), (d) | (a) 2 286 / 2 286 dòng giống từng byte; (b) Rust ≡ TS; (c) 561 phép hợp + 762 kỳ phân hoạch bộ lọc, 0 vi phạm; (d) 762 kỳ, 0 vi phạm (115 kỳ có mục ở ngày đầu, 98 ở ngày cuối) |
| R13 bộ R3 | **ĐẠT** (giữ) | Đầu ra `r13` của 181c039 vẫn giống từng byte bản nộp `b2_r13.jsonl` |
| R11 | **ĐẠT** (giữ) | Đầu ra `upcoming` của 181c039 giống từng byte `b2_r11_upcoming_sau_sua.csv`. A.3 nay ghi đúng cách hiểu đã dùng; lần sửa oracle vẫn là **sau đối chiếu** (mục 5) |
| R1 (Jpl) | **ĐẠT** (giữ) | sha256 đầu ra `gio-dates` của 181c039 trùng bản `main` đã đối chiếu |
| Rust ≡ TS | **ĐẠT** | 181c039: 24/24 lệnh giống (16 lệnh cũ + `r13` hai bộ + `reminders --global-off/--rule-off`, mỗi lệnh có và không có `--method jpl`), `out/b3_parity_181c039.tsv` |

### 10.2 Trình tự

| Giờ (+07) | Commit | Việc |
|---|---|---|
| ~03:35 | — | Đọc 04 B3 và các mục 02 tại 181c039; đọc dữ liệu `r13-linked.json` |
| 03:41 | `950949d` | Đầu ra oracle: `b3_r12.csv` (I1–I4), `b3_r3_global_off.csv`, `b3_r3_rule_off.csv`, `b3_r13_linked.jsonl` + tự kiểm (c), (d); tổng băm `SHA256SUMS.b3`. **Trước** khi chạy bất kỳ dump nào của 181c039 |
| sau 03:42 | commit báo cáo | Dựng dump 181c039 (Rust, TypeScript), chạy và so; cập nhật báo cáo |

Không có sửa oracle nào sau đối chiếu ở lần B3.

### 10.3 R12 — cài đặt oracle (`b3_r12.py`; `b2_r12.py` giữ nguyên)

- `reschedule`: hi = now + 400 ngày; desired = lời nhắc có now < at ≤ hi, sắp (at, id); `beyond` = có lời nhắc at > hi.
  Nếu len(desired) > 64 hoặc `beyond`: giữ k = min(len, 63) cái sớm nhất, thông báo nhắc mở ứng dụng tại at của cái
  thứ k (k = 0 → tại hi).
- Tập lời nhắc theo A.5: lần xuất hiện có ngày trong [2026-01-01, 2028-12-31 + 900 ngày] và mọi sự kiện không lặp tới
  3000-12-31.
- Cột theo A.5 đã làm rõ: `delivered` gồm thông báo nhắc mở ứng dụng; giao lúc 09:00 mỗi ngày; `missed` trong
  (09:00 ngày đầu, 09:00 ngày cuối]; p0.1 không rút số ở ngày đầu, từ ngày 2 mở khi `u32 × 10 < 2³²`.
- I4 kiểm trực tiếp: lời nhắc tương lai sớm nhất (trong hay ngoài cửa sổ) không đang chờ phải có thông báo nhắc mở
  ứng dụng với at ≤ nó. **Đối chứng âm** (kiểm bộ kiểm có tác dụng): chạy thuật toán B2 (không xét `beyond`) dưới bộ
  kiểm B3 cho I4 = 1 096 / 100 / 1 vi phạm ở 10-p1 / 10-p0.1 / 10-p0, đúng những ô cỡ 10 có lời nhắc ngoài cửa sổ mà
  chưa tới 64 trong cửa sổ.

| Ô | opens | max_pending | delivered | sentinel_delivered | missed | missed_without_sentinel | i1 | i2 | i3 | i4 |
|---|---|---|---|---|---|---|---|---|---|---|
| 10 / p1 | 1 096 | 38 | 71 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| 10 / p0.1 | 100 | 37 | 71 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| 10 / p0 | 1 | 34 | 34 | 1 | 38 | **0** | 0 | 0 | 0 | 0 |
| 50 / p1 | 1 096 | 64 | 296 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| 50 / p0.1 | 100 | 64 | 296 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| 50 / p0 | 1 | 64 | 64 | 1 | 233 | 0 | 0 | 0 | 0 | 0 |
| 200 / p1 | 1 096 | 64 | 1 125 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| 200 / p0.1 | 100 | 64 | 967 | 3 | 161 | 0 | 0 | 0 | 0 | 0 |
| 200 / p0 | 1 | 64 | 64 | 1 | 1 062 | 0 | 0 | 0 | 0 | 0 |

Bảng trên là của oracle và trùng từng giá trị với lõi (`out/b3_cmp_r12.json`: 0 ô khác, 0 bước khác). Phát hiện ở
mục 6.3 đã được xử lý: ô 10 / p0 nay có thông báo nhắc mở ứng dụng được giao trước 38 lời nhắc ở xa
(`missed_without_sentinel` 38 → 0).

### 10.4 R13 — bộ `r13-linked`

Bộ dữ liệu kiểm tới cả hai nhánh của quy tắc lọc sự kiện. Đếm mục theo năm, tổng 11 năm, ở đầu ra của oracle:
- `da_mat`: giỗ 185, tiên thường 33, và các sự kiện gắn toàn người đã mất ("Giỗ tổ dòng họ", "Họp họ đầu xuân", "Tất
  niên dòng họ", "Lễ tạ mộ" — cùng ngày với giỗ của người đó, xếp sau giỗ —, "Ngày cưới ông bà" 29/02 với quy tắc nhắc
  tắt, sự kiện không tiêu đề hiện tên loại "Tốt nghiệp", và hai sự kiện một lần).
- `dang_song`: sinh nhật 87, sinh nhật âm 54, sự kiện gắn người sống, sự kiện hỗn hợp (đã mất + sống: "Ngày cưới",
  "Mốc cuối cửa sổ", "Mừng thọ") và sự kiện không gắn ai ("Kỷ niệm thành lập họ").
(c), (d) chạy trên cả tệp oracle và tệp lõi, cùng kết quả (`out/b3_r13_linked_check_oracle.json`,
`out/b3_r13_linked_check_core.json`).

### 10.5 Tệp thêm (`oracle/v02/out/`)

`b3_r12.csv`, `b3_core_r12_181c039.csv`, `b3_cmp_r12.json`; `b3_r3_global_off.csv`, `b3_r3_rule_off.csv`,
`b3_core_r3_global_off.csv`, `b3_core_r3_rule_off.csv`; `b3_input_r13-linked_181c039.json`, `b3_r13_linked.jsonl`,
`b3_core_r13_linked_181c039.jsonl`, `b3_r13_linked_check_oracle.json`, `b3_r13_linked_check_core.json`;
`b3_parity_181c039.tsv`; `SHA256SUMS.b3`. Mã: `b3_r12.py`, `b3_cmp_r12.py`; `b2_r3.py` thêm hai cờ `--global-off`,
`--rule-off` (chế độ thường không đổi: sinh lại cho cùng sha256 `86bcc3d2…`); `b2_parity.sh` thêm lệnh B3.

## 11. Tệp (lần B2; tệp lần B3 ở mục 10.5)

Đầu ra oracle (có trong kho), thư mục `oracle/v02/out/`:
- R1: `r1_jpl_d.jsonl`, `b2_r1_jpl_summary.json`.
- R11: `b2_r11_upcoming.csv` (bản nộp trước đối chiếu), `b2_r11_upcoming_sau_sua.csv` (sau khi sửa).
- R12: `b2_r12.csv`.
- R13: `b2_r13.jsonl`, `b2_r13_check_oracle.json`.
- Tổng băm: `SHA256SUMS.b2-r1`, `SHA256SUMS.b2-r3-r11-r12`, `SHA256SUMS.b2-r13`, `SHA256SUMS.b2-r3-reminders`.
- Tệp lớn (R1 `.jsonl.gz`, 38 triệu dòng R3, cấu hình R3) không đưa vào kho. Sinh lại bằng `b2_gen_r1.py`, `b2_r3.py`
  và đối chiếu bằng tổng băm.

Đối chiếu (cùng thư mục):
- R1: `b2_core_r1_jpl_SUMS_93cc6c3.txt`, `b2_cmp_r1_jpl_d.json`.
- R12: `b2_core_r12_2eea0bb.csv`, `b2_cmp_r12.json`, `b2_r12_bien_the_rut_tu_ngay_2.csv`, `b2_cmp_r12_bien_the.json`.
- R13: `b2_core_r13_2eea0bb.jsonl`, `b2_r13_check_core.json`.
- Rust ≡ TS: `b2_parity_main_93cc6c3.tsv`, `b2_parity_w3_2eea0bb.tsv`.

Máy đo: Linux x86_64, 2 vCPU, 7 GB RAM; Python 3 (chỉ thư viện chuẩn), Rust 1.97 (bản release), Node 22. Không dùng
máy chạy GitHub: mọi phép tính chạy được trong môi trường kiểm định (R3 đầy đủ mất vài phút).
