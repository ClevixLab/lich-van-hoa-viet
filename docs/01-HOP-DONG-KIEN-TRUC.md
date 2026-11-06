# Hợp đồng kiến trúc — Lịch và Văn hóa Việt v0.1

Trạng thái: KHÓA (G1) — 2026-11-06. Mọi thay đổi phải ghi lý do và được chủ dự án duyệt.

## 1. Cấu trúc kho mã

```
core/      Lõi Rust (crate `lvhv-core`), không phụ thuộc thư viện ngoài, #![forbid(unsafe_code)]
tsport/    Bản chuyển ngữ TypeScript của lõi, dùng cho PWA (tạm thời, đến khi có WebAssembly qua CI)
oracle/    Bản kiểm định độc lập (Python + PyEphem), viết "phòng sạch", KHÔNG đọc core/ hay tsport/
content/   Dữ liệu nội dung: quy tắc Ngọc Hạp (có dẫn nguồn), ca dao/tục ngữ (có nguồn)
pwa/       Ứng dụng web cài được (PWA), chạy offline
docs/      Tài liệu quy trình, kỹ thuật, bảo mật, báo cáo
```

## 2. Quy ước thiên văn và lịch (bắt buộc cho core, tsport, oracle)

- Múi giờ lịch Việt Nam: UTC+7 cho mọi ngày từ 2000 đến 3000.
- Ngày âm lịch bắt đầu từ ngày (theo giờ UTC+7) chứa thời điểm Sóc (trăng mới).
- Tháng 11 âm lịch là tháng chứa Đông chí.
- Năm có 13 tháng (giữa hai tháng 11 có 13 lần Sóc) thì tháng nhuận là tháng đầu tiên sau tháng 11 không chứa Trung khí; tháng nhuận mang số của tháng trước nó.
- Trung khí = các tiết khí có kinh độ Mặt Trời là bội số của 30°.
- Ngày mới bắt đầu lúc 00:00 UTC+7 (không dùng quy ước giờ Tý bắt đầu 23:00 cho việc đổi ngày).
- Phương pháp tham chiếu của core: thuật toán Hồ Ngọc Đức (Sóc theo Meeus, kinh độ Mặt Trời theo Meeus). Oracle dùng phương pháp KHÁC (PyEphem/VSOP87) để kiểm định độc lập.

## 3. Can Chi

- Can (10): Giáp, Ất, Bính, Đinh, Mậu, Kỷ, Canh, Tân, Nhâm, Quý
- Chi (12): Tý, Sửu, Dần, Mão, Thìn, Tỵ, Ngọ, Mùi, Thân, Dậu, Tuất, Hợi
- Can Chi ngày: từ số ngày Julian (JDN): can = (JDN + 9) mod 10, chi = (JDN + 1) mod 12.
- Can Chi năm âm lịch Y: can = (Y + 6) mod 10, chi = (Y + 8) mod 12.
- Can Chi tháng âm lịch: chi tháng 1 = Dần, tháng 2 = Mão, ...; can tháng 1 tính theo can năm (Giáp/Kỷ → Bính Dần; Ất/Canh → Mậu Dần; Bính/Tân → Canh Dần; Đinh/Nhâm → Nhâm Dần; Mậu/Quý → Giáp Dần). Tháng nhuận dùng Can Chi của tháng chính cùng số.
- Can giờ Tý đầu ngày theo can ngày (Giáp/Kỷ → Giáp Tý; Ất/Canh → Bính Tý; Bính/Tân → Mậu Tý; Đinh/Nhâm → Canh Tý; Mậu/Quý → Nhâm Tý).
- Tiết khí của một ngày: tiết khí gần nhất mà thời điểm bắt đầu (theo UTC+7) rơi vào ngày đó hoặc trước đó.
- Trực, Nhị thập bát tú, Hoàng đạo/Hắc đạo (12 thần), giờ Hoàng đạo, Nạp âm: theo quy tắc truyền thống, ghi công thức và nguồn trong docs/03-DAC-TA-THUAT-TOAN.md.
  - Trực: tính theo chi tháng TIẾT (tháng tính theo tiết khí, không theo tháng âm) và chi ngày. Ngày có chi trùng chi tháng tiết là trực Kiến.
  - Nhị thập bát tú: chu kỳ 28 ngày theo JDN, mốc ghi trong đặc tả.

## 4. Giao diện công khai của lõi (core và tsport phải giống hệt)

Hàm:
- `solar_to_lunar(y, m, d) -> LunarDate`
- `lunar_to_solar(ly, lm, ld, leap) -> Option<SolarDate>`
- `day_info(y, m, d) -> DayInfo`
- `find_good_days(activity_id, from(y,m,d), to(y,m,d), birth_year: Option) -> Vec<GoodDay>` (phần Ngọc Hạp)

`DayInfo` xuất ra JSON (khóa tiếng Anh, giá trị tiếng Việt có dấu), thứ tự khóa cố định:

```json
{
  "solar": {"y": 2026, "m": 11, "d": 5, "weekday": 4, "jdn": 2461350},
  "lunar": {"y": 2026, "m": 9, "d": 27, "leap": false, "month_len": 30},
  "can_chi": {"year": "Bính Ngọ", "month": "Mậu Tuất", "day": "Quý Mùi", "hour_ty": "Nhâm Tý"},
  "tiet_khi": "Sương Giáng",
  "nap_am": "Dương Liễu Mộc",
  "truc": "Thu",
  "tu": "Tỉnh",
  "than": {"name": "Chu Tước", "hoang_dao": false},
  "gio_hoang_dao": [{"chi": "Dần", "from": 3, "to": 5}],
  "ngoc_hap": {"sao_tot": [], "sao_xau": [], "nen": [], "ky": [], "xung_tuoi": [], "score": 0}
}
```
- `weekday`: 0 = Chủ nhật, 1 = Thứ Hai, …, 6 = Thứ Bảy.
- `gio_hoang_dao.from/to`: giờ đồng hồ (Tý = 23–1 ghi from 23, to 1).
- Ví dụ trên chỉ minh họa định dạng; giá trị đúng do kiểm định quyết định.

## 5. Phân vai (bản 0.1)

| Mã | Vai | Thư mục được ghi | Không được đọc |
|---|---|---|---|
| A2 | Lõi lịch Rust | core/ (trừ core/src/ngochap*) | oracle/ |
| A3 | Quy tắc Ngọc Hạp | content/ngochap/, core/src/ngochap* | oracle/ |
| A4 | Kiểm định độc lập | oracle/ | core/, tsport/ (trước khi nộp kết quả) |
| A5 | Nội dung ca dao | content/cadao/ | — |
| A6 | PWA | pwa/, tsport/ | oracle/ |
| A9/A10 | Bảo mật / Duyệt mã | docs/reviews/ | — |
