# Biên bản cổng G1–G4 — Lịch và Văn hóa Việt v0.1

Ngày lập: 2026-11-06. Người lập: A11 (phát hành). Tiêu chí: `docs/02-TIEU-CHI-NGHIEM-THU.md`, khóa ngày
2026-11-06, trước khi viết mã lõi (giờ tệp 00:11, mã lõi sớm nhất 00:50). Biên bản **không nới lỏng** tiêu
chí nào. Thang kết luận chỉ gồm ba mức: **ĐẠT / KHÔNG ĐẠT / CHƯA ĐỦ BẰNG CHỨNG**.

Trạng thái mã: thư mục làm việc lúc lập biên bản. Kho chưa có commit nên không có mã commit để neo. Mọi số
dưới đây đã được kiểm lại trong đợt này bằng cách chạy lại lệnh hoặc đọc lại tệp, trừ khi có ghi chú khác.

Đã chạy lại trong đợt này (máy 2 vCPU x86_64 Linux, 7 GB RAM):

| Lệnh | Kết quả | Tệp lưu |
|---|---|---|
| `cargo test --workspace --release` | mọi test thường ĐẠT (core 20+10+7+2+8, ffi 20, ngochap 28) | — |
| `cargo test --release -p lvhv-core -- --ignored --nocapture` | 7 test dài ĐẠT, 15 phút 5 giây | `release/tools/ket-qua-test-ignored.log` |
| `scripts/check-parity.sh` | ĐẠT: 0 sai lệch, 1 phút 27 giây | `release/tools/ket-qua-parity.log` |
| `cd pwa && npm run typecheck && npm run build && node test/smoke.mjs` | ĐẠT, 69/69 kiểm tra | `release/tools/ket-qua-smoke.log`, `pwa/test/metrics.json` |
| Sinh lại `lunar-csv`, `tet`, `newmoons` 2000–3000 (Hnd, Precise) và `terms` 2000–3000 từ lõi hiện tại, so `cmp` với `core/out/` | **giống hệt từng byte**, tức kết quả kiểm định dựa trên `core/out/` vẫn ứng với mã hiện tại | — |
| `python3 release/tools/so-sanh-precise-oracle.py core/out` và `… core/out/precise` | xem G2 | `release/tools/ket-qua-hnd.json`, `release/tools/ket-qua-precise.json` |
| `python3 content/cadao/validate.py` | "OK: không có lỗi", 427 câu | — |
| `cargo fmt --all --check` | sạch | — |

Công cụ `release/tools/so-sanh-precise-oracle.py` là công cụ mới của đợt phát hành. Nó chỉ đọc CSV,
không sửa `oracle/` và không ghi đè báo cáo kiểm định. Khi chạy trên `core/out` (Hnd), công cụ cho lại đúng
mọi con số của `docs/reports/BAO-CAO-KIEM-DINH-LICH.md` (60 ngày/2 đoạn; 2 784 ngày/84 đoạn, 6 đoạn
"nguyên nhân khác"; Can Chi 342 ngày; 11 năm Tết khác). Đó là phép thử để tin công cụ khi áp lên Precise.

**Quyết định đang chờ ảnh hưởng nhiều tiêu chí nhất:** phương pháp lịch mặc định, Hnd (hiện là
`DEFAULT_METHOD`) hay Precise (docs/03 §15). Với L2, L3, L4, L6, biên bản ghi kết luận cho **cả hai** cột.
Kết luận áp dụng là cột ứng với phương pháp chủ dự án chọn. Bản dựng `pwa/dist/` hiện tại dùng **Hnd**.

---

## G1 — Hợp đồng kiến trúc và tiêu chí

Cổng G1 không có mã tiêu chí riêng trong docs/02. Nội dung của cổng là khóa hai văn bản trước khi viết mã.

| Hạng mục | Kết luận | Bằng chứng | Ghi chú |
|---|---|---|---|
| Hợp đồng kiến trúc ghi trạng thái KHÓA | ĐẠT | `docs/01-HOP-DONG-KIEN-TRUC.md` dòng 3 "KHÓA (G1) — 2026-11-06"; giờ tệp 00:10 | |
| Tiêu chí nghiệm thu khóa trước khi viết mã lõi | ĐẠT | `docs/02` "Khóa lúc: 2026-11-06, trước khi viết mã lõi"; giờ tệp 00:11, trước tệp mã lõi sớm nhất (`core/src/astro.rs` 00:50) và trước `oracle/oracle.py` (00:15) | Giờ tệp là bằng chứng yếu (có thể bị sửa). Kho chưa có commit nên không có bằng chứng mạnh hơn |
| Tiêu chí không bị sửa sau khi thấy kết quả | ĐẠT (theo giờ tệp) | Lần sửa cuối của `docs/02` lúc 00:11, trước báo cáo kiểm định (00:57) | Như trên |
| Thay đổi so với hợp đồng có ghi lý do | ĐẠT, có ngoại lệ | Ngọc Hạp tách thành crate `ngochap/`, khác §5 "core/src/ngochap*"; ghi ở BIEN-BAN-S1-S4 §S2. Có thêm `Method::Precise` (docs/03 §15). Tên thần thứ 6 là "Kim Đường" (docs/03 §9) | Ba thay đổi này **cần chủ dự án duyệt** (dòng trạng thái của hợp đồng: "Mọi thay đổi phải ghi lý do và được chủ dự án duyệt"). Hiện chưa có chữ ký duyệt |

**Đề nghị:** duyệt G1. Kèm theo đó, chủ dự án xác nhận ba thay đổi hợp đồng nêu trên (crate `ngochap/` riêng;
thêm phương pháp Precise; tên "Kim Đường").

Chủ dự án duyệt: ____________________  Ngày: ____________

---

## G2 — Lịch và Ngọc Hạp (L1–L8, N1–N3)

### Bảng kết luận

| Mã | Ngưỡng khóa | Hnd (mặc định hiện tại) | Precise | Bằng chứng | Ghi chú |
|---|---|---|---|---|---|
| L1 | Tết 2000–2100 so danh sách công bố: 0 sai lệch | **CHƯA ĐỦ BẰNG CHỨNG** | **CHƯA ĐỦ BẰNG CHỨNG** | `oracle/reference/tet_published.csv` (51 năm có nguồn VN, 101 năm HKO); `ket-qua-*.json` (`L1_*`) | Nguồn VN chỉ phủ **51/101** năm, với **0** sai lệch ở các năm được phủ. Tết Hnd và Precise trùng nhau ở mọi năm 2000–2100. So với Đài Thiên văn Hồng Kông (lịch Trung Quốc, UTC+8), lõi khác ở **3** năm: 2007, 2030, 2053. Đây đúng là 3 năm mà nguồn VN cho biết lịch Việt khác lịch Trung Quốc. Nếu chủ dự án chấp nhận "danh sách HKO + 3 ngoại lệ đã biết" làm danh sách công bố thì kết luận đổi được, nhưng đó là quyết định của chủ dự án (xem mục quyết định số 2) |
| L2 | Ngày âm 2000-01-01…2100-12-31 so oracle: 0 sai lệch | **KHÔNG ĐẠT**: 60 ngày, 2 đoạn (2072-12-09…2073-01-07; 2077-11-15…2077-12-14) | **ĐẠT**: 0 sai lệch | `oracle/out/mismatches.csv`; `ket-qua-hnd.json`, `ket-qua-precise.json` (`L2_*`) | Cả hai đoạn của Hnd là Sóc cách nửa đêm dưới 2 phút. Ngưỡng 2000–2100 không có ngoại lệ "sát nửa đêm" |
| L3 | 2101–3000: liệt kê, phân loại mọi sai lệch; có "nguyên nhân khác" là KHÔNG ĐẠT | **KHÔNG ĐẠT**: 2 784 ngày, 84 đoạn, **6** đoạn "nguyên nhân khác" (2405, 2682, 2728, 2758, 2864, 2948) | **ĐẠT**: 90 ngày, 3 đoạn (2242-07-28, 2962-10-19, 2988-06-05), cả 3 "sát nửa đêm" (Sóc cách 00:00 lần lượt 0,1′/0,0′; 4,6′/0,1′; 4,9′/0,2′); 0 "nguyên nhân khác" | `ket-qua-precise.json` (`L3_segments`); `docs/reports/BAO-CAO-KIEM-DINH-LICH.md` §4 | 6 đoạn của Hnd là tháng nhuận chứa Trung khí theo chính cột tiết khí của lõi. Lỗi tự mâu thuẫn này được khóa bằng test `self_consistency_hnd_known_violations` (đã chạy lại: đúng 6 tháng đó). Precise: `self_consistency_precise` ĐẠT. Kết luận ĐẠT cho Precise dựa trên công cụ của đợt phát hành, chưa được bên kiểm định độc lập (A4) chạy lại |
| L4 | Can Chi ngày/tháng/năm 2000–3000 so oracle: 0 sai lệch | **KHÔNG ĐẠT**: 342 ngày (năm 40, tháng 342, ngày 0) | **KHÔNG ĐẠT**: 3 ngày (tháng 3: 2242-07-28, 2962-10-19, 2988-06-05) | `ket-qua-*.json` (`L4_*`) | Ở cả hai phương pháp, mọi ngày lệch Can Chi đều là hệ quả của ngày âm lệch (L2/L3). Can Chi ngày khớp 100 %. Ngưỡng là 0 cho toàn dải 2000–3000, không có ngoại lệ sát nửa đêm, nên kết luận cho Precise vẫn là KHÔNG ĐẠT. Trong 2000–2100, Precise có 0 sai lệch Can Chi |
| L5 | Khứ hồi dương → âm → dương 2000–3000: 0 lỗi | **ĐẠT** | **ĐẠT** | `ket-qua-test-ignored.log`: `roundtrip_2000_3000 … ok`, `precise_roundtrip_2000_3000 … ok` | Test của chính lõi (365 608 ngày + tính liên tục). Oracle không kiểm được L5 (báo cáo kiểm định §2). Đây là kiểm tra nội bộ, không phải kiểm định độc lập |
| L6 | Tiết khí 2000–2100: 0 sai lệch ngày bắt đầu | **KHÔNG ĐẠT** | **KHÔNG ĐẠT** | `ket-qua-*.json` (`L6_*`); `BAO-CAO-KIEM-DINH-LICH.md` §6 | 1 tiết khí: **Xử Thử 2074**, lõi 2074-08-23 00:00:06, oracle 2074-08-22 23:59:55 (chênh 11 s quanh nửa đêm). Tiết khí dùng cùng một mô hình cho cả hai phương pháp, nên không sửa được bằng cách chọn phương pháp. Trong 2101–3000 còn 1 ca nữa (Xuân Phân 2963, cách nửa đêm 6 s), ngoài phạm vi L6 |
| L7 | tsport ↔ core: `day_info` JSON 2000–3000 giống hệt từng byte | **ĐẠT** | **ĐẠT** | `ket-qua-parity.log`: dayinfo 365 608/365 608 dòng, 0 sai lệch, ở cả hai phương pháp; Tết 1 001 dòng và gooddays 4 470 dòng cũng 0 sai lệch | Sinh mới từ lõi hiện tại, không dùng tệp vàng cũ |
| L8 | 2026-11-05 khớp mọi trường quan sát | **ĐẠT** | **ĐẠT** | `core/tests/fixtures.rs::fixture_2026_11_05` (so cả JSON); `methods.rs::precise_fixture_2026_11_05`; `ffi` test L8; smoke PWA ("ngày âm 27", Can Chi …) | Mọi trường khớp: âm 27/9 Bính Ngọ, ngày Quý Mùi, tháng Mậu Tuất, Trực Thu, Chu Tước Hắc Đạo, Dương Liễu Mộc, Sương Giáng, sao Tỉnh, giờ Hoàng đạo Dần-Mão-Tỵ-Thân-Tuất-Hợi. Báo cáo kiểm định ghi L8 là CHƯA ĐỦ BẰNG CHỨNG vì oracle chỉ kiểm được 5 trường. Biên bản này kết luận ĐẠT vì tiêu chí khóa là so với **trang lịch công khai**, không phải so với oracle, và test fixture kiểm đủ 10 nhóm trường. Giá trị đối chiếu lấy từ đề bài (trang lichvn.net, rules.md §5), không phải do bên độc lập tra lại |
| N1 | Mỗi quy tắc sao có ≥ 1 nguồn kiểm chứng được: 100 % | **ĐẠT** | **ĐẠT** | `content/ngochap/rules.md` §7 (109 quy tắc G/B/S, mỗi dòng có cột Nguồn); §3 (12 thần: N01, N02, N07); §4 (bảng tuổi: N22, N23); `content/ngochap/SOURCES.md` (27 mục N01–N27, F01–F03, có URL) | Đã kiểm bằng máy: 109/109 dòng có mã nguồn N-xx, và mọi mã đều có URL trong SOURCES.md. Giới hạn: 68/109 quy tắc chỉ có **1 dòng sách** (TVTP). N1 chỉ yêu cầu ≥ 1 nguồn nên vẫn ĐẠT. Nguồn là trang web chép lại sách 1997; sách in gốc chưa được tra (SOURCES.md "Hạn chế") |
| N2 | Mâu thuẫn giữa nguồn được liệt kê, ghi phương án đã chọn: 100 % | **ĐẠT** | **ĐẠT** | `rules.md` C01–C18 | Cả 18 mâu thuẫn đều có phương án ghi trong bảng. Ghi chú: C01, C15 tự ghi "cần người duyệt xác nhận". Biên bản không đánh giá phương án nào đúng, chỉ xác nhận đã liệt kê và ghi lựa chọn |
| N3 | Ngày phạm sao kỵ chính không xuất hiện trong kết quả chọn ngày: 0 vi phạm 2000–3000 | **ĐẠT** | **ĐẠT** | `ket-qua-test-ignored.log`: `n3_all_days_2000_3000` (11 699 456 lượt/phương pháp, 0 vi phạm); `n3_find_good_days_2000_3000` (Hnd 4 563 965, Precise 4 564 673 ngày đề xuất, 0 vi phạm); `suggested_days_consistent_with_sheet_2000_3000` (§2a: Hnd 2 346 064, Precise 2 346 397, 0 vi phạm) | N3 kiểm cài đặt so với bảng `major` của chính dự án, không kiểm bảng đó có đúng với sách hay không (DUYET-MA §3) |

### Tổng hợp G2 theo phương pháp

| Phương pháp | ĐẠT | KHÔNG ĐẠT | CHƯA ĐỦ BẰNG CHỨNG |
|---|---|---|---|
| Hnd (hiện tại) | L5, L7, L8, N1, N2, N3 | **L2, L3, L4, L6** | L1 |
| Precise | L2, L3, L5, L7, L8, N1, N2, N3 | **L4, L6** | L1 |

Không có phương pháp nào ĐẠT toàn bộ G2. L6 KHÔNG ĐẠT ở cả hai phương pháp. L4 cũng KHÔNG ĐẠT ở cả hai:
Precise vẫn còn 3 ngày lệch Can Chi tháng sau 2100, do 3 tháng có Sóc sát nửa đêm.

**Đề nghị:**
1. **Chưa thông qua G2** khi giữ nguyên tiêu chí, vì cả hai cột đều có KHÔNG ĐẠT. Muốn mở cổng, chủ dự án cần
   quyết định rõ ràng và ghi lại một trong hai hướng: (a) chấp nhận phát hành v0.1 **thử nghiệm nội bộ** với
   các KHÔNG ĐẠT đã biết; mỗi sai lệch đều sát nửa đêm (dưới 5 phút ở Sóc, 11 s ở tiết khí) và được ghi
   công khai. Hoặc (b) viết **tiêu chí mới** cho v0.2, ví dụ thêm phân loại sát nửa đêm cho 2000–2100 hoặc
   dùng ephemeris chuẩn làm trọng tài. Không được sửa tiêu chí v0.1 sau khi đã thấy kết quả.
2. Về phương pháp: Precise giảm KHÔNG ĐẠT từ 4 xuống 2 và bỏ được 6 năm tự mâu thuẫn. Hnd có lợi thế trùng
   với các lịch Việt phổ biến dùng cùng thuật toán. Đổi mặc định chỉ cần sửa một hằng (`DEFAULT_METHOD`, cả
   Rust và tsport), sau đó dựng lại và chạy `scripts/check-parity.sh`. Biên bản này không chọn thay chủ dự án.
3. Nhờ bên kiểm định độc lập (A4) chạy lại phân loại L3 cho Precise để xác nhận kết quả của công cụ phát hành.

Chủ dự án duyệt: ____________________  Ngày: ____________   Phương pháp chọn: ☐ Hnd  ☐ Precise

---

## G3 — Hiệu năng (P1–P5)

Máy đo: Linux x86_64, 2 vCPU (Intel Xeon 2,10 GHz), 7 GB RAM; Chromium 153 headless; bản release có LTO
và `overflow-checks = true`.

| Mã | Ngưỡng khóa | Kết luận | Giá trị đo | Bằng chứng | Ghi chú |
|---|---|---|---|---|---|
| P1 | `day_info` native release < 1 ms/ngày | **ĐẠT** | Hnd 25,9–29,9 µs; Precise 36,3–45,3 µs (trung bình 3 lần); p99 lớn nhất 103 µs | `release/tools/ket-qua-bench-loi.txt` | Đã tính cả `to_json` |
| P2 | `find_good_days` 90 ngày native < 50 ms | **ĐẠT** | Hnd 1,98–3,14 ms; Precise 2,86–3,47 ms (trung bình); lớn nhất 7,28 ms | như trên | Lần đo này cao hơn docs/05 §E (0,74–1,92 ms). Nguyên nhân chưa tách (sandbox nhiễu). Dư ngưỡng khoảng 7 lần |
| P3 | PWA tải lần đầu gzip < 500 KB | **ĐẠT** | 313,9 KB (321 463 byte, 20 yêu cầu) | `pwa/test/metrics.json`; `release/tools/ket-qua-nang-luong-pwa.json` (`install_bytes_gzip`) | Hai phép đo độc lập cho cùng số byte |
| P4 | Heap JS khi xem tờ lịch < 30 MB | **ĐẠT** | 2,43 MB (smoke, 360×780); 2,30 MB (sau thu gom rác, giả lập điện thoại) | `metrics.json`; `ket-qua-nang-luong-pwa.json` | Sau 100 lần vuốt và một lần chọn ngày: 2,80 MB |
| P5 | Chạy được khi không có mạng sau lần mở đầu | **ĐẠT** | Có | smoke: "offline lần hai" (`metrics.json` `offline_second_visit: true`) | |

Ngoài tiêu chí khóa (chỉ để tham khảo, xem `docs/reports/BAO-CAO-NANG-LUONG-BO-NHO.md`): để yên 60 s thì
JS chạy 0 ms; ứng dụng Windows trên Linux dùng 51 MB; Android/iOS **chưa đo**.

**Đề nghị:** thông qua G3. Lưu ý rằng P1/P2 chỉ đo trên máy dựng, chưa đo trên điện thoại hay qua lớp FFI.

Chủ dự án duyệt: ____________________  Ngày: ____________

---

## G4 — Nội dung và bảo mật (C1, S1–S4)

| Mã | Ngưỡng khóa | Kết luận | Bằng chứng | Ghi chú |
|---|---|---|---|---|
| C1 | Mỗi câu ca dao/tục ngữ có nguồn; không có câu của tác giả cá nhân: 100 % | **ĐẠT** (theo câu chữ của tiêu chí) | `content/cadao/validate.py` → "OK: không có lỗi" (427 câu, mỗi câu có trường `sources`); `content/cadao/README.md` (chính sách chỉ nhận văn học dân gian); `SOURCES.md` | **Rủi ro mở:** nguồn trực tuyến chỉ xác nhận **phần mở đầu** (khoảng 5–7 chữ) của mỗi câu (SOURCES.md §D). Toàn văn chưa được soát với tuyển tập in. Câu chữ của các dòng sau trong ca dao dài có thể sai. Phần "không có tác giả cá nhân" là đánh giá của người biên soạn, chưa có người đọc độc lập kiểm lại. Ngoài C1: các câu có dị bản được liệt kê ở README "Điểm chưa chắc" |
| S1 | Không có `unsafe` trong core | **ĐẠT** | Chạy lại `grep -rn --include=*.rs -E '\bunsafe\b' core/src ngochap/src`: chỉ còn 1 dòng chú thích (`core/src/lib.rs:3`), cộng `#![forbid(unsafe_code)]`; `docs/reviews/BIEN-BAN-S1-S4.md` | |
| S2 | Lõi không phụ thuộc thư viện ngoài | **ĐẠT** (có điều kiện diễn giải) | Chạy lại `cargo tree -p lvhv-core -e normal,build,dev`: chỉ có `lvhv-ngochap` (đường dẫn trong kho, không có phụ thuộc con) | Điều kiện diễn giải theo BIEN-BAN-S1-S4 §S2: nếu đọc "0" theo nghĩa đen là không có crate nào khác thì kết luận đổi thành KHÔNG ĐẠT. Chủ dự án cần xác nhận |
| S3 | PWA không gọi mạng ngoài nguồn gốc của mình; CSP chặt | **ĐẠT** | smoke 69/69 lần này ("không gọi mạng ngoài (0)", "CSP chặt", "không có script nội tuyến", "lạ: 0"); `ket-qua-nang-luong-pwa.json`: sau khi cài chỉ có 1 yêu cầu `/sw.js` tới chính nguồn gốc | Bản dựng được kiểm: `app-GDOOI3PQ.js` (sha256 `446ac163…`), `index.html` (`2f4397b1…`), `sw.js` (`c069db37…`). CSP qua thẻ `<meta>`: `frame-ancestors` không có hiệu lực (RS-09) |
| S4 | PWA không lưu dữ liệu người dùng | **ĐẠT** | smoke: "ngày sinh không được lưu ({ls:{}, ss:{}, cookie:"", idb:[]})"; đo: `localStorage` trống cho tới khi người dùng đổi cỡ chữ/giao diện | Cách hiểu "tùy chọn giao diện không phải dữ liệu người dùng" theo BIEN-BAN-S1-S4 §S4 |

Các phát hiện Cao của rà soát bảo mật (RS-01, RS-02) đã ĐÃ SỬA theo `docs/05` §C. Bằng chứng là
`core/tests/bad_input.rs`, test thường chạy lại trong đợt này (10/10 ĐẠT). RS-10 (origin phát hành) vẫn chờ
chủ dự án quyết định.

**Đề nghị:** thông qua G4 cho S1–S4. Với C1: thông qua theo câu chữ của tiêu chí, kèm điều kiện ghi rõ rủi
ro chưa soát toàn văn. Khuyến nghị soát 427 câu theo tuyển tập in trước khi phát hành công khai (ngoài vòng
thử nghiệm nội bộ). Chủ dự án xác nhận cách diễn giải S2.

Chủ dự án duyệt: ____________________  Ngày: ____________

---

## Các quyết định đang chờ chủ dự án

| # | Quyết định | Ảnh hưởng | Phương án | Tài liệu |
|---|---|---|---|---|
| 1 | **Phương pháp lịch mặc định**: Hnd hay Precise | L2, L3, L4, L6; bản dựng mọi nền tảng | **ĐÃ QUYẾT 06/11/2026: Precise** (docs/02 "Bổ sung A3"; xem "Cập nhật 06/11/2026" cuối biên bản). Hnd giữ trong mã để đối chiếu | docs/03 §15.7; G2 |
| 2 | **Khoảng trống bằng chứng L1** | L1 | (a) Chấp nhận danh sách HKO 2000–2100 + 3 ngoại lệ đã biết (2007, 2030, 2053) làm "danh sách công bố", rồi chạy lại so sánh. (b) Giữ CHƯA ĐỦ BẰNG CHỨNG và tìm thêm nguồn VN cho 50 năm còn thiếu | BAO-CAO-KIEM-DINH-LICH §3 |
| 3 | **Ghi chú sát nửa đêm cho L6** (Xử Thử 2074, 11 s) | L6 cả hai phương pháp | Ghi thành sai lệch đã biết (không sửa tiêu chí v0.1); hoặc v0.2 dùng VSOP87 đầy đủ/ephemeris chuẩn làm trọng tài | docs/03 §15.5 |
| 4 | **Origin phát hành** `lich.clevix.vn` (RS-10) | S3 (tách origin), DEPLOY | Tên miền tùy chỉnh của GitHub Pages; xác minh tên miền `clevix.vn` trên GitHub | docs/05 §C RS-10; `DEPLOY.md` trong gói PWA |
| 5 | **Giấy phép mã nguồn** | LICENSE, các Cargo.toml | **ĐÃ QUYẾT 06/11/2026: MIT.** `LICENSE` gốc là văn bản MIT; mọi Cargo.toml ghi `license = "MIT"` | LICENSE; docs/05 §D.1 |
| 6 | **Tên thần thứ 6 "Kim Đường"** (thay "Thiên Đức"/"Bảo Quang") | Chuỗi hiển thị | Duyệt hoặc đổi (chỉ đổi chuỗi, không ảnh hưởng L1–L6) | docs/03 §9 |
| 7 | **Ngưỡng nhãn đánh giá** (Tốt ≥ 4, Khá tốt ≥ 1, Bình thường ≥ −3, còn lại Xấu) | Nhãn trên tờ lịch | Duyệt hoặc đổi ngưỡng; đây là quy ước giao diện, không phải quy tắc truyền thống | `pwa/src/format.ts::rating`; SCORING.md |
| 8 | **Soát chữ ca dao** theo tuyển tập in | C1 (rủi ro mở) | Một người đọc 427 câu theo *Kho tàng tục ngữ/ca dao người Việt* | content/cadao/SOURCES.md §D |
| 9 | **Điểm tối thiểu 0 cho ngày đề xuất** (`MIN_GOOD_DAY_SCORE`) | Độ dài danh sách chọn ngày | Giữ 0, hoặc nâng lên 3 (`NEN_THRESHOLD`) để khớp mục "Nên làm" | SCORING.md §2a; docs/05 §D.4 |
| 10 | **Ghi công giấy phép Slint** (Royalty-free 2.0, "Made with Slint") | Bản Windows | Đọc điều khoản tại slint.dev/terms-and-conditions và xác nhận | docs/04 §3.4 |

Các quyết định khác còn mở trong tài liệu: diễn giải S2 (G4); duyệt các thay đổi hợp đồng (G1); mở rộng
kiểm định cho ngày sinh 1900–1999 (docs/05 §D.3); phương án C01/C15 trong rules.md.

---

## Cập nhật 06/11/2026 (bổ sung A1–A3)

Phần này **bổ sung**, không sửa các mục trên. Căn cứ: `docs/02` "Bổ sung A1" (L9–L13, khóa trước khi sửa mã),
"Bổ sung A2" và "Bổ sung A3" (cả hai ghi rõ là quyết định của chủ dự án **sau khi đã thấy kết quả**); báo cáo
`docs/reports/BAO-CAO-KIEM-DINH-1900-1999.md` **Lần 2** (sau A2), do bên không tham gia xây dựng đặc tả lập
(chỉ đọc tệp dữ liệu, không đọc mã lõi). Thang kết luận giữ nguyên: ĐẠT / KHÔNG ĐẠT / CHƯA ĐỦ BẰNG CHỨNG.
Không ngưỡng nào bị nới: A3 chỉ quy định **chấm theo nguồn nào** khi các nguồn Việt Nam mâu thuẫn nhau và
**cột phương pháp nào** là cột áp dụng.

### Quyết định A3 được áp dụng trong biên bản

1. Phương pháp mặc định là **Precise**. Từ nay cột áp dụng của L2, L3, L4, L6, L10, L12 là cột Precise; cột Hnd
   giữ để đối chiếu. Mã đã đổi: `DEFAULT_METHOD = Method::Precise` (lõi Rust), `"precise"` (tsport); bản PWA đọc
   giá trị từ lõi lúc build (`pwa/test/metrics.json`: `"method": "precise"`).
2. **L9 khi nguồn Việt Nam mâu thuẫn:** chấm theo nguồn tuân thủ quy ước lịch sử đã chọn (giờ địa phương Bắc Kinh
   trước 1929, UTC+8 đến hết 1967). Theo báo cáo Lần 2 §L2.3, nguồn đó là xemlicham (khớp lịch Trung Quốc cùng
   quy ước, HKO xác nhận ở 1916 và 1965); chogia khớp phép tính UTC+7 suốt, tức không theo quy ước đã chọn.

### L9 — từng năm nguồn Việt Nam mâu thuẫn (số liệu từ báo cáo Lần 2 và `oracle/reference/tet_published_1900_1999.csv`)

| Năm | xemlicham | chogia | Nguồn khác | Lõi (Hnd = Precise = oracle) | Nguồn theo quy ước A3 | Nguồn còn lại lệch |
|---|---|---|---|---|---|---|
| 1903 | 1903-01-29 | 1903-01-28 | — | 1903-01-29 | xemlicham: khớp | chogia sớm hơn **1 ngày** |
| 1925 | 1925-01-25 (trang năm) / 1925-01-24 (trang ngày) | 1925-01-24 | — | 1925-01-24 | xemlicham trang ngày: khớp | xemlicham trang năm muộn hơn **1 ngày** (nguồn tự mâu thuẫn; Sóc cách nửa đêm > 1 giờ ở mọi múi giờ 7–8, báo cáo §3) |
| 1935 | 1935-02-04 | 1935-02-03 | — | 1935-02-04 | xemlicham: khớp | chogia sớm hơn **1 ngày** |
| 1965 | 1965-02-02 | 1965-02-01 | HKO (Trung Quốc, hỗ trợ): 1965-02-02 | 1965-02-02 | xemlicham: khớp | chogia sớm hơn **1 ngày** |

Ghi chú 1925: A3 coi 1925 là năm "một nguồn tự mâu thuẫn". Chọn trang ngày của xemlicham là áp dụng A3 (giá trị
nhất quán với quy ước và với nguồn VN còn lại); trang năm 01-25 được ghi là lệch, không bị xóa.

### Bảng kết luận L9–L13

| Mã | Ngưỡng khóa (A1) | Hnd (đối chiếu) | **Precise (mặc định, A3)** | Bằng chứng | Ghi chú |
|---|---|---|---|---|---|
| L9 | Tết 1900–1999 so nguồn VN: 0 sai lệch ở các năm có nguồn | **ĐẠT** (theo A3) | **ĐẠT** (theo A3) | Báo cáo Lần 2 §L2.2, §L2.3; bảng trên | Tết hai phương pháp và oracle trùng nhau cả 100 năm. Nguồn VN phủ 100/100 năm. Ở 96 năm nguồn thống nhất: 0 sai lệch. 4 năm mâu thuẫn chấm theo A3: 0 sai lệch. **Nếu không áp dụng A3** (đọc ngưỡng là "khớp mọi nguồn"), kết luận là KHÔNG ĐẠT như báo cáo Lần 2 ghi. Cách chấm A3 có sau khi thấy kết quả — người đọc cần biết điều này |
| L10 | Ngày âm 1900–1999 so oracle cùng quy tắc múi giờ: 0 sai lệch | **KHÔNG ĐẠT**: 118/36 524 ngày, 2 đoạn (1917-03-23→05-20; 1922-06-25→08-22), vị trí tháng nhuận do Trung khí sát nửa đêm giờ Bắc Kinh (3,2′ và 5,6′) | **ĐẠT**: 0/36 524 | Báo cáo Lần 2 §L2.2, §L2.4; `oracle/out/mismatches_1900_1999_{hnd,precise}.csv` | Ngưỡng L10 "cùng quy tắc như L2" không có ngoại lệ sát nửa đêm |
| L11 | Liên tục 1967–1969; Tết 1968 = 1968-01-29 | **ĐẠT** | **ĐẠT** | Báo cáo Lần 2 §L2.2 (1096/1096 ngày, 0 lỗi); `core/tests/range1900.rs::l11_continuity_1967_1969` | Ngoài tiêu chí: liên tục quanh 1929-01-01 (1928–1930) ĐẠT cả hai |
| L12 | Khứ hồi và TS = Rust từng byte, 1900–3000, cả hai phương pháp | **ĐẠT** | **ĐẠT** | Chạy lại trong đợt này sau khi đổi mặc định: `scripts/check-parity.sh` 0 sai lệch (dayinfo 402 132 dòng × 2, Tết 1 101 × 2, gooddays 4 470 × 2); `range1900.rs::l12_roundtrip_1900_3000` 402 132 ngày × 2, 0 lỗi | Kiểm tra nội bộ, không phải kiểm định độc lập (bên kiểm định không kiểm được khứ hồi) |
| L13 | 2000–3000 không đổi so với tệp chuẩn | **ĐẠT** | **ĐẠT** | Báo cáo Lần 2 §L2.2 (kiểm từ dữ liệu); chạy lại trong đợt này: `check-parity.sh` bước 5 "giống hệt từng byte" với `core/out/baseline-2000-3000/{hnd,precise}.jsonl.gz` | Tệp chuẩn tách theo phương pháp nên đổi mặc định không làm tệp chuẩn mất hiệu lực |

Ngoài tiêu chí L10 (không thuộc năm/tháng/ngày/nhuận): cột tiết khí lệch 1 ngày ở cả hai phương pháp,
1951-12-22 (Đông Chí cách nửa đêm UTC+8 3 giây theo oracle). Ghi lại để biết; không đổi kết luận nào.

### L2/L10 — chấm cho phương pháp mặc định Precise

| Mã | Precise | Ghi chú |
|---|---|---|
| L2 (2000–2100) | **ĐẠT** — 0 sai lệch | Không đổi so với mục G2 ở trên; nay là cột áp dụng |
| L10 (1900–1999) | **ĐẠT** — 0 sai lệch | Như bảng trên |
| L3 (2101–3000) | **ĐẠT** — 3 đoạn, cả 3 "sát nửa đêm", 0 "nguyên nhân khác" | Không đổi so với mục G2; vẫn dựa trên công cụ của đợt phát hành, chưa được bên kiểm định độc lập chạy lại cho Precise |

### Không đổi: L4, L6 (các ca sát nửa đêm 2000–3000)

- **L4 Precise: KHÔNG ĐẠT** như trước — 3 ngày lệch Can Chi tháng (2242-07-28, 2962-10-19, 2988-06-05), hệ quả
  của 3 tháng có Sóc sát nửa đêm (dưới 5 phút).
- **L6: KHÔNG ĐẠT** ở cả hai phương pháp như trước — Xử Thử 2074 (lõi 2074-08-23 00:00:06, oracle 2074-08-22
  23:59:55). Tiết khí dùng chung một mô hình nên đổi phương pháp không ảnh hưởng.
- **L1: CHƯA ĐỦ BẰNG CHỨNG** như trước (nguồn VN phủ 51/101 năm).
- A3 không quyết định gì về L1, L4, L6. Các mục 2 và 3 trong "Các quyết định đang chờ chủ dự án" vẫn mở.

### Tổng hợp sau A3 (cột áp dụng: Precise)

| Nhóm | ĐẠT | KHÔNG ĐẠT | CHƯA ĐỦ BẰNG CHỨNG |
|---|---|---|---|
| L1–L8 (2000–3000) | L2, L3, L5, L7, L8 | **L4, L6** | L1 |
| L9–L13 (1900–1999, A1) | L9 (theo A3), L10, L11, L12, L13 | — | — |
| N1–N3 | N1, N2, N3 | — | — |

Đối chiếu Hnd: thêm KHÔNG ĐẠT ở L2, L3, L10.

**Đề nghị:** G2 vẫn **chưa thông qua được** khi giữ nguyên tiêu chí, vì L4 và L6 KHÔNG ĐẠT ở phương pháp mặc
định. Mục quyết định số 1 (phương pháp mặc định) **đã quyết: Precise** (A3, 06/11/2026). Hướng (a) hoặc (b) ở
mục G2 vẫn cần chủ dự án chọn và ghi lại.

Chủ dự án duyệt: ____________________  Ngày: ____________
