# Cách tính điểm Ngọc Hạp — lvhv-ngochap v0.1

**Lưu ý trung thực:** cách chấm điểm này là **quy ước của dự án**, không phải quy tắc truyền thống.
Ngọc Hạp Thông Thư chỉ ghi sao nào tốt/xấu cho việc nào; sách không cho trọng số. Trọng số dưới đây
là lựa chọn thiết kế, công khai để người dùng và người duyệt kiểm tra được.

## 1. Trọng số sao (`weight`)

| Nhóm | Trọng số | Sao |
|---|---|---|
| Sao tốt lớn | 2 | Thiên Đức, Nguyệt Đức, Sát Cống, Trực Tinh, Nhân Chuyên |
| Sao tốt thường | 1 | Mọi sao tốt còn lại |
| Sao xấu đại hung | 3 | Thiên Cương, Đại Hao, Kiếp Sát, Thụ Tử, Trùng Tang, Sát Chủ, Thập Ác Đại Bại, Dương Công Kỵ Nhật |
| Ngày kỵ dân gian | 2 | Tam Nương, Nguyệt Kỵ |
| Sao xấu thường | 1 | Mọi sao xấu còn lại, Thọ Tử (vì luôn trùng Thụ Tử, tránh trừ hai lần quá nặng) |
| 12 thần | ±2 | Hoàng đạo +2, Hắc đạo −2 |

Lý do chọn nhóm "đại hung": các sao này được nguồn N02 ghi "xấu mọi việc" hoặc được trang
lichngaytot (giới thiệu sao xấu) xếp nhóm "đại hung"; Sát Cống/Trực Tinh/Nhân Chuyên được N03 gọi là
"tốt nhất" trong hệ Kim Phù.

## 2. Điểm cho một việc — `suitability(ctx, việc, năm_sinh)`

Với việc `a`:

1. **Sao tốt**: nếu sao có ghi "kỵ" việc `a` (ví dụ Nguyệt Đức Hợp kỵ tố tụng, Thiên Xá kỵ động thổ) → `−weight`.
   Ngược lại, nếu sao tốt cho `a` → `+weight`, **cộng thêm 1** nếu việc `a` được nêu đích danh
   ("tốt cho cưới hỏi", "mọi việc, nhất là xây dựng").
2. **Sao xấu** (gồm ngày đặc biệt và thần Hắc đạo): nếu sao kỵ việc `a` → `−weight`, **trừ thêm 1** nếu `a` được nêu đích danh.
3. **Trực**: Kiến, Mãn, Bình, Định, Thành, Khai `+1`; Trừ, Chấp, Thu `0`; Phá, Nguy, Bế `−1`
   (phân loại theo N05/N06; chỉ là phần nhỏ của điểm).
4. **Tuổi xung** (nếu có năm sinh): Y là **năm âm lịch của ngày sinh** (`birth_lunar_year`), không
   phải năm dương — người sinh trước Tết thuộc năm âm trước (sinh 15/01/1976 → năm âm 1975, Ất Mão).
   Lõi đổi ngày sinh dương → năm âm bằng `lunar_year_of_birth(y, m, d)` (miền 1900–3000). Y đổi ra
   Can Chi theo quy ước lõi (can = (Y+6) mod 10, chi = (Y+8) mod 12); nếu Can Chi đó nằm trong danh
   sách tuổi xung của ngày → `−3`.

`allowed = false` khi một trong các điều sau đúng:

- **Sao kỵ chính (N3)**: có ít nhất một sao trong ngày mà `major` chứa việc `a` (danh sách ở rules.md, cột "Kỵ chính").
- **Tuổi xung với việc cá nhân**: tuổi xung ngày và `a` thuộc nhóm việc cá nhân:
  cưới hỏi, xây dựng, khai trương, động thổ, xuất hành, giao dịch, cầu tài, tố tụng, nhập trạch,
  khởi sự, cầu tự, nhập học, cầu sức khỏe.
  **Không** thuộc nhóm cá nhân: an táng, tế tự, làm phúc (việc làm cho người đã khuất hoặc việc chung
  của gia đình; tuổi người xem vẫn bị trừ điểm nhưng không chặn).
- Mã việc không tồn tại.

`reasons` liệt kê theo thứ tự cố định: các sao kỵ chính → từng sao tốt (thứ tự bảng) → từng sao xấu → trực → tuổi.

## 2a. Ngày được đề xuất — `find_good_days` (sửa 2026-11-06)

Một ngày chỉ được đưa vào danh sách chọn ngày cho việc `a` khi thỏa **cả ba** điều:

1. `allowed = true` (§2: không phạm sao kỵ chính — N3 —, không xung tuổi với việc cá nhân);
2. **điểm việc** (§2, đã gồm `−3` tuổi xung) `≥ 0` (`MIN_GOOD_DAY_SCORE`);
3. việc `a` **không** có trong danh sách `ky` của tờ lịch ngày đó (§3).

Lý do: trước đây chỉ có điều 1, nên danh sách đề xuất cả ngày mà chính tờ lịch ghi "Kỵ" việc đó
(sao xấu nêu đích danh việc và điểm âm) và ngày điểm âm. Đo 2026–2029, mọi việc, không tuổi
(`lvhv-dump gooddays all 2026-01-01 2029-12-31 none`): trước 11 671 ngày đề xuất, trong đó 613 ngày
tờ lịch ghi Kỵ chính việc đó và 2 270 ngày điểm âm; sau 9 401 ngày, 0 và 0. (Con số của bên điều phối
đo qua `ffi` — 11 707/616/2 273 — hơi khác vì `ffi` còn dùng cầu nối riêng.) Với điều 1 + 2, điều 3
đã được bảo đảm về logic (vì "kỵ" = kỵ chính **hoặc** sao xấu nêu đích danh và điểm < 0); lõi vẫn kiểm
tường minh để tờ lịch và danh sách không bao giờ mâu thuẫn. Test thường trực:
`core/tests/ngochap.rs::suggested_days_consistent_with_sheet_2026` (chạy mặc định) và
`…_2000_3000` (`--ignored`, toàn dải, cả hai phương pháp).

**Hai loại điểm khác nhau.** Điểm trong danh sách chọn ngày là **điểm cho việc đã chọn** (§2).
Nhãn đánh giá trên tờ lịch ("Tốt", "Khá tốt"…) dùng **điểm chung của ngày** (§3, không theo việc).
PWA ghi rõ "điểm việc" / "Điểm cho việc …" trong danh sách để hai con số không bị nhầm.

## 3. Danh sách "nên" / "kỵ" và điểm chung — `evaluate(ctx)`

- Một việc vào **kỵ** nếu phạm sao kỵ chính, **hoặc** có sao xấu nêu đích danh việc đó và điểm việc < 0.
- Một việc vào **nên** nếu không có sao xấu nào nêu đích danh việc đó và điểm việc ≥ 3 (`NEN_THRESHOLD`).
- Còn lại: không ghi (trung tính). Không có việc nào vừa "nên" vừa "kỵ".
- **Điểm chung** của ngày = Σ trọng số sao tốt − Σ trọng số sao xấu + điểm trực (không theo việc, không theo tuổi).

Ví dụ 2026-11-05 (Quý Mùi, tháng 9, Trực Thu, Chu Tước): 4 sao tốt (1+1+1+2 = 5) − 9 sao xấu
(Tam Nương 2 + 7 sao thường 7 + Chu Tước 2 = 11) + trực Thu 0 = **−6**. Không việc nào "nên".

## 4. Những gì CHƯA đưa vào điểm (có chủ ý)

- **Nhị thập bát tú**: chưa có bảng tốt/xấu từng tú có nguồn kiểm chứng trong đợt này → không tính.
- **Tiết khí (Tứ Ly, Tứ Tuyệt)**: cần ngày tiết khí từ lõi; `DayCtx` chưa có trường này → chưa cài.
- **Sao năm** (Tuế Đức, Kim Thần Thất Sát…): chưa cài.
- **Giải sao** (sao tốt "giải" sao xấu, ví dụ Thiên Giải hóa Bạch Hổ): chỉ cài luật Hoạt Điệu mất tác dụng khi gặp Thụ Tử
  (có nguồn N01). Các luật hóa giải khác chưa có nguồn đủ rõ → không cài.
