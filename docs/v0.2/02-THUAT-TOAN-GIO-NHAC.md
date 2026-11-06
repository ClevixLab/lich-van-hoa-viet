# Đặc tả thuật toán ngày giỗ, tuổi, nhắc việc và .ics — v0.2

Trạng thái: **KHÓA (G1) — 2026-11-06, chờ chủ dự án ký.** Đây là đặc tả duy nhất cho cả lõi Rust, bản TypeScript và
bản cài đặt độc lập dùng để kiểm định (tiêu chí R1–R3, R10–R12 trong `04-TIEU-CHI-NGHIEM-THU-V02.md`). Bên cài đặt độc lập
chỉ đọc tài liệu này và `01-MO-HINH-DU-LIEU.md`, không đọc mã lõi.

Ký hiệu: `L2S(y, m, d, leap)` = `lunar_to_solar` của lõi (trả `None` nếu ngày không tồn tại); `S2L(y, m, d)` =
`solar_to_lunar`; `LYOB(y, m, d)` = `lunar_year_of_birth` (năm âm chứa ngày dương đó); `len(y, m, leap)` = số ngày
của tháng âm (29 hoặc 30). Phương pháp lịch: mặc định của lõi (`Precise` lúc khóa G1; từ bản 0.1.4 là `Jpl`,
docs/02 Bổ sung A4 — ghi chú Bổ sung B2, 2026-11-07). Mọi phép cộng/trừ ngày là trên ngày dương (số ngày Julian),
không trên giờ.

Ví dụ trong tài liệu được tính bằng `tsport` ngày 2026-11-06 (phương pháp `Precise`); chúng cũng là ca kiểm thử cố
định (§9).

## 1. Ngày mất: lưu và đổi lịch

```
save_death(input):
  if input.calendar == "solar":                       # precision phải là "day"
      L := S2L(input.year, input.month, input.day)     # lỗi nếu ngoài 1900–3000
      death.lunar  := {year: L.y, month: L.m, day: L.d, leap: L.leap}
      death.solar  := {y, m, d của input}
      death.lunar_source := "converted"
  else:                                                # âm
      death.lunar  := {year: input.year (hoặc null), month, day, leap}
      death.solar  := (input.precision == "day" và năm trong miền) ? L2S(...) : null
      death.lunar_source := "entered"
  death.method := DEFAULT_METHOD
```

Ngày âm được **đổi một lần khi lưu** và lưu lại. Sửa ngày mất thì đổi lại. Đổi phương pháp mặc định ở bản sau
không tự sửa ngày đã lưu (01 §3.2).

Ví dụ: mất 2025-08-08 (dương) → 15/6 **nhuận** năm Ất Tỵ 2025.

## 2. Ngày giỗ hằng năm

### 2.1 Quy tắc (chủ dự án đã duyệt 06/11/2026)

1. Ngày giỗ là **ngày âm của ngày mất**, lặp mỗi năm âm.
2. Mất trong **tháng nhuận** → giỗ vào tháng **thường** cùng số (tháng nhuận hiếm khi lặp lại).
3. Mất ngày **30** mà tháng đó trong năm giỗ chỉ có 29 ngày → giỗ ngày **29**.
4. Không biết năm mất → vẫn tính được ngày giỗ mỗi năm từ (tháng, ngày); không gắn nhãn giỗ đầu/giỗ hết.

### 2.2 Thuật toán

```
anniversary(death, Y) -> {lunar: (Y, m, d', false), solar, n, label} | None
  m  := death.lunar.month            # bỏ qua death.lunar.leap (quy tắc 2)
  d  := death.lunar.day
  if death.lunar.year != null and Y <= death.lunar.year: return None   # chưa có giỗ
  d' := (d == 30 and len(Y, m, false) == 29) ? 29 : d                  # quy tắc 3
  if Y < 1899 or Y > 3000: return None
  S  := L2S(Y, m, d', false)          # tháng m thường luôn tồn tại; d' ≤ len(Y, m) theo quy tắc 3
  if S == None: return None           # chỉ xảy ra ở biên miền (ngày dương ngoài 1900–3000)
  n  := (death.lunar.year != null) ? Y - death.lunar.year : null        # giỗ lần thứ n
  label := label_of(n, memorial.gio_het_nam)                            # §3
  return {lunar: (Y, m, d', false), solar: S, n, label}
```

Năm có cả tháng m thường và tháng m nhuận: giỗ luôn ở tháng m **thường** (tháng đứng trước).

### 2.3 Các lần giỗ rơi trong một khoảng ngày dương

```
anniversaries_in(death, from, to):     # from, to: ngày dương, from <= to
  out := []
  for Y in LYOB(from) - 1 .. LYOB(to) + 1:     # năm âm có thể lệch năm dương tới ~2 tháng
      a := anniversary(death, Y)
      if a != None and from <= a.solar <= to: out.append(a)
  return out sắp theo a.solar
```

### 2.4 Ví dụ (ca kiểm thử cố định)

| Ca | Ngày mất | Năm giỗ (âm) | Ngày giỗ âm | Ngày giỗ dương |
|---|---|---|---|---|
| G-A | 2025-08-08 = 15/6 nhuận Ất Tỵ | 2026 (n=1) | 15/6/2026 | 2026-07-28 |
| G-A | | 2027 (n=2) | 15/6/2027 | 2027-07-18 |
| G-B | 2025-09-21 = 30/7/2025 (tháng 7/2025 đủ 30 ngày) | 2026 (n=1) | tháng 7/2026 chỉ 29 ngày → **29/7/2026** | 2026-09-10 |
| G-B | | 2027 (n=2) | 30/7/2027 | 2027-08-31 |
| G-C | âm 10/3, không rõ năm | 2026 | 10/3/2026 | 2026-04-26 |
| G-C | | 2027 | 10/3/2027 | 2027-04-16 |
| G-D | 2024-02-09 = 30/12/2023 | 2024 (n=1) | tháng 12/2024 chỉ 29 ngày → 29/12/2024 | **2025**-01-28 |
| G-D | | 2025 (n=2) | 29/12/2025 | 2026-02-16 |
| G-E | âm 15/5 năm 2027 (giả định) | 2028 (năm có tháng 5 nhuận) | 15/5/2028 **thường** | 2028-06-07 (không phải 2028-07-07 của tháng nhuận) |

G-D cho thấy năm giỗ âm 2024 rơi vào năm dương 2025: mọi danh sách và tệp .ics phải dùng §2.3, không được giả
định "năm âm = năm dương".

## 3. Nhãn giỗ đầu / giỗ hết, tiên thường

```
label_of(n, gio_het_nam):
  if n == null:            return "gio"          # "Ngày giỗ"
  if n == 1:               return "gio_dau"      # "Giỗ đầu" (tiểu tường)
  if n == gio_het_nam:     return "gio_het"      # "Giỗ hết" (đại tường), mặc định n = 2
  return "gio"             # "Giỗ" (có thể ghi thêm "lần thứ n")
```

- `gio_het_nam` mặc định 2; cho chọn 3 vì nhiều nơi làm giỗ hết vào năm thứ ba (mục mở O-2). Với `gio_het_nam = 3`,
  lần n = 2 là "Giỗ" thường.
- **Tiên thường** (lễ chiều hôm trước ngày giỗ): ngày dương = `a.solar − 1 ngày`. Chỉ khi `ReminderRule.tien_thuong
  = true` (mặc định **tắt**, mục mở O-1). Khi bật, sinh thêm **một** lời nhắc đúng ngày tiên thường vào giờ
  `time_local`, với nội dung "Hôm nay lễ tiên thường — ngày mai giỗ …". Các mốc nhắc trước (`offsets_days`) luôn
  tính theo **ngày giỗ**, không theo ngày tiên thường. Nếu một mốc trước trùng ngày tiên thường (offset 1), vẫn
  gửi hai lời nhắc riêng (khác nội dung).

## 4. Tuổi âm, hưởng thọ / hưởng dương, Can Chi năm sinh

### 4.1 Năm âm của ngày sinh

| Ngày sinh nhập | `lunar_year` | `lunar_year_approx` |
|---|---|---|
| Dương đủ ngày | `LYOB(y, m, d)` (sinh trước Tết → năm âm trước) | `false` |
| Âm đủ ngày, hoặc âm chỉ năm | năm âm đã nhập | `false` |
| Dương chỉ năm `y` | `y` | `true` (nếu sinh trước Tết năm `y` thì năm âm thật là `y − 1`) |
| Không nhập | `null` | — |

`can_chi = can_chi_year(lunar_year)` theo hợp đồng kiến trúc §3 (`can = (Y + 6) mod 10`, `chi = (Y + 8) mod 12`).
Ví dụ: sinh 1990-01-20 → 24/12 năm âm 1989 → **Kỷ Tỵ** (không phải Canh Ngọ 1990).

### 4.2 Hưởng thọ / hưởng dương (tuổi âm, cách tính truyền thống)

```
tuoi_mat(birth, death, nguong):
  if birth.lunar_year == null or death.lunar.year == null: value := null
  else: value := death.lunar.year - birth.lunar_year + 1     # tuổi mụ: tính cả năm sinh
  approx := birth.lunar_year_approx
  label  := (value != null and value < nguong) ? "huong_duong" : "huong_tho"   # nguong mặc định 60 (O-3)
```

- Kết quả lưu ở `death.tuoi`. Người dùng sửa được; khi sửa đặt `manual = true` và không tính lại nữa (trừ khi bấm
  "Tính lại").
- Hiển thị: "Hưởng thọ 86 tuổi" / "Hưởng dương 45 tuổi"; khi `approx = true` thêm "(khoảng)".
- `value < 1` (dữ liệu mâu thuẫn) → lỗi nhập liệu (01 §3.2), không lưu.
- (Bổ sung B2, 2026-11-07) `value > 150` → lỗi nhập liệu "Tuổi … vượt quá 150", không lưu (01 §3.2: `tuoi.value`
  trong 1–150).

Ví dụ: sinh 1940-05-01 (năm âm 1940 Canh Thìn), mất 2025-08-08 (năm âm 2025) → 2025 − 1940 + 1 = **86**, hưởng thọ.

### 4.3 Tuổi của người đang sống

- Tuổi dương (hiển thị chính): số lần sinh nhật dương đã qua tính đến hôm nay (dùng ngày đã quy đổi ở §6 cho 29/02).
- Tuổi âm (hiển thị phụ): `năm âm hôm nay − lunar_year + 1`.
- Ví dụ: sinh 1990-01-20, ngày 2026-11-06 (năm âm 2026) → tuổi dương 36, tuổi âm 38.

## 5. Sinh nhật âm và sự kiện âm lặp hằng năm

Dùng đúng `anniversary` của §2.2 với `(month, day)` của ngày gốc, cùng hai quy tắc nhuận → thường và 30 → 29. Năm
lần thứ n: `n = Y − năm âm gốc` (sinh nhật âm: tuổi âm = n + 1). Sự kiện âm không lặp: chỉ một lần, ngày dương
`L2S(...)` của chính ngày nhập.

## 6. Sinh nhật dương, sự kiện dương lặp hằng năm, quy tắc 29/02

```
solar_occurrence(orig (Y0, M, D), Y, feb29_rule):
  if Y < Y0: None            # sinh nhật: Y == Y0 là ngày sinh, không nhắc; lần nhắc đầu là Y0 + 1
  if (M, D) == (2, 29) and not is_leap(Y):
      return feb29_rule == "feb28" ? (Y, 2, 28) : (Y, 3, 1)     # mặc định "feb28"
  return (Y, M, D)
```

- Năm nhuận Gregory: chia hết cho 4 và (không chia hết cho 100 hoặc chia hết cho 400).
- Sự kiện dương không lặp: một lần, ngày đã nhập.
- Sự kiện "Ngày cưới" mặc định lặp hằng năm (01 §3.5), hiển thị "Kỷ niệm n năm ngày cưới".
- (Bổ sung B2, 2026-11-07) Sự kiện dương lặp hằng năm có ngày gốc 29/02: ở năm không nhuận rơi vào **28/02**
  (`solar_occurrence` với `feb29_rule = "feb28"`; sự kiện không có trường `feb29_rule` riêng).

## 7. Sinh lời nhắc

### 7.1 Lời nhắc một lần xuất hiện

```
reminders_for(rule, occ, now, tz):          # occ: một lần xuất hiện (ngày dương D, loại, người…)
  if not settings.reminders_enabled or not rule.enabled: return []
  out := []
  for k in rule.offsets_days:                       # k = 0 là đúng ngày
      day := D - k ngày
      t   := local_instant(day, rule.time_local, tz)
      if t > now: out.append({id: notif_id(rule.id, occ, k), at: t, k, occ})
  if rule.tien_thuong and occ.kind == "gio":
      t := local_instant(D - 1 ngày, rule.time_local, tz)
      if t > now: out.append({id: notif_id(rule.id, occ, "tt"), at: t, kind: "tien_thuong", occ})
  return out
```

- `local_instant(day, "HH:MM", tz)`: giờ đồng hồ trên **múi giờ hiện tại của thiết bị**. Giờ không tồn tại (bước
  nhảy giờ mùa hè) → dời sau đúng khoảng nhảy; giờ lặp (lùi giờ) → lấy lần đầu. Việt Nam không đổi giờ mùa hè; quy
  tắc chỉ có tác dụng khi người dùng ở nước khác.
- Đổi múi giờ thiết bị → lập lịch lại toàn bộ (§7.4).
- `notif_id(rule_id, occ, k)` = `"lvhv:" + rule_id + ":" + occ.key + ":" + k`, trong đó `occ.key` là năm âm Y (giỗ,
  sinh nhật âm, sự kiện âm) hoặc năm dương Y (sinh nhật dương, sự kiện dương) hoặc `"once"`. Mã này ổn định: sửa giờ
  nhắc không đổi mã, nên thay thế đúng thông báo cũ.
- Nội dung (không có số liệu ngoài hồ sơ): tiêu đề "Còn 3 ngày: Giỗ đầu Ông Nguyễn Văn A" hoặc "Hôm nay: …"; dòng
  phụ "Thứ Ba, 28/07/2026 — 15/6 âm lịch". Tên hiển thị = `danh_xung + " " + ho_ten`. Tùy chọn khóa màn hình ở mục
  mở O-9.
- (Bổ sung B2, 2026-11-07) Màn hình khóa khi không hiện tên (O-9): tiêu đề "Sắp đến ngày giỗ" (giỗ, tiên thường),
  "Sắp đến sinh nhật" (sinh nhật dương, sinh nhật âm) và **"Sắp đến sự kiện"** (mọi loại sự kiện); không có dòng phụ.
- Ví dụ: G-A năm 2026, `[0, 3]`, `07:00` → 2026-07-25 07:00 và 2026-07-28 07:00 (giờ thiết bị).

### 7.2 Danh sách "Sắp tới" (30 ngày)

Mọi lần xuất hiện (giỗ, tiên thường nếu bật, sinh nhật dương, sinh nhật âm, sự kiện) có ngày dương trong
`[hôm nay, hôm nay + 30 ngày]` (gồm cả hai đầu), sắp theo ngày, cùng ngày thì theo thứ tự giỗ → tiên thường →
sinh nhật → sự kiện, rồi theo `ho_ten` (so sánh theo quy tắc chữ tiếng Việt của nền tảng; kiểm thử dùng so sánh
điểm mã sau NFC để xác định). Danh sách **không phụ thuộc** công tắc nhắc: tắt nhắc vẫn thấy trong "Sắp tới".
Hiển thị: "Hôm nay", "Ngày mai", "Còn n ngày".

### 7.3 Lập lịch trên Android (đề xuất, chờ O-6)

- Quyền cần khai báo: `POST_NOTIFICATIONS` (Android 13+, hỏi người dùng lúc bật nhắc lần đầu) và
  `RECEIVE_BOOT_COMPLETED` (đặt lại báo thức sau khi khởi động lại — tài liệu Android ghi rõ mọi báo thức bị hủy
  khi tắt máy). **Không** có `INTERNET`. CI hiện tại cấm mọi `uses-permission` (`android.yml`) → phải đổi thành
  danh sách cho phép đúng các quyền này (S6).
- Cơ chế (kiểm ở R5): **một chuỗi báo thức** — chỉ đặt **một** báo thức cho thời điểm nhắc sớm nhất; khi báo thức chạy, đăng
  mọi thông báo đã đến hạn (kể cả những cái bị trễ do máy tắt, nếu trễ ≤ 24 giờ; trễ hơn thì bỏ, vì "Sắp tới" vẫn
  hiển thị) rồi đặt báo thức kế tiếp. Không có giới hạn số lượng lời nhắc.
- Loại báo thức mặc định (không cần quyền đặc biệt): `setAndAllowWhileIdle(RTC_WAKEUP, t)` — không bao giờ chạy sớm
  hơn `t`; từ Android 12 hệ thống giao trong vòng một giờ nếu không có hạn chế tiết kiệm pin. Lời nhắc giỗ lúc 07:00
  đến muộn tối đa khoảng một giờ là chấp nhận được cho việc "nhắc trước".
- Tùy chọn (O-6): nếu chủ dự án muốn đúng giờ, khai báo `SCHEDULE_EXACT_ALARM`, gọi `canScheduleExactAlarms()`,
  dùng `setExactAndAllowWhileIdle` khi được cấp, quay về loại không chính xác khi bị từ chối, và lắng nghe
  `ACTION_SCHEDULE_EXACT_ALARM_PERMISSION_STATE_CHANGED`. **Không** dùng `USE_EXACT_ALARM` trừ khi chủ dự án xác
  nhận ứng dụng đủ điều kiện chính sách Google Play (xem 05 §7 R-AND-1).
- Đặt lại chuỗi khi: `BOOT_COMPLETED`, `MY_PACKAGE_REPLACED`, `TIME_SET`, `TIMEZONE_CHANGED`, mở ứng dụng, lưu dữ
  liệu, nhập sao lưu.

### 7.4 Lập lịch trên iOS / macOS: xoay vòng 64 thông báo chờ

Giới hạn: mỗi ứng dụng chỉ có tối đa **64** yêu cầu thông báo cục bộ đang chờ; đây là giới hạn hệ thống (xác nhận
của kỹ sư Apple trên diễn đàn nhà phát triển, 05 §7 R-IOS-1). Thuật toán:

```
CAP := 64
reschedule(now):
  hi      := now + 400 ngày
  desired := mọi lời nhắc (§7.1) có at > now, trong cửa sổ (now, hi], sắp theo (at, id)
  beyond  := có ít nhất một lời nhắc (§7.1) có at > hi                 # Bổ sung B3
  if len(desired) > CAP or beyond:                                     # còn lời nhắc không được đặt
      k    := min(len(desired), CAP-1)                                 # tối đa 63 lời nhắc sớm nhất
      keep := desired[0 .. k-1]
      at_s := (k > 0) ? keep[k-1].at : hi                             # Bổ sung B2/B3: cùng phút với cái cuối đã đặt
      sentinel := {id: "lvhv:sentinel", at: at_s,
                   nội dung: "Mở Lịch Việt để tiếp tục nhận lời nhắc giỗ và sinh nhật"}
      keep.append(sentinel)
  else: keep := desired
  pending := getPendingNotificationRequests()
  remove(pending.id ∖ keep.id)                             # chỉ gỡ cái không còn cần
  add(keep ∖ pending)                                      # UNCalendarNotificationTrigger(dateMatching: y-m-d h:m, repeats: false)
  thay thế những cái cùng id nhưng khác at hoặc nội dung
```

- Gọi `reschedule` khi: mở ứng dụng / trở lại tiền cảnh, lưu dữ liệu, nhập sao lưu, đổi múi giờ
  (`NSSystemTimeZoneDidChange`), người dùng bấm vào một thông báo, và trong tác vụ nền `BGAppRefreshTask` (iOS tự
  quyết định có chạy hay không; không dựa vào nó).
- Bảo đảm (kiểm ở R12 bằng mô phỏng, ở R5 trên iPhone thật): tại mọi thời điểm, nếu có ≥ 64 lời nhắc tương lai thì 63 cái sớm nhất đều đang chờ và thông báo nhắc mở
  ứng dụng nằm ngay sau cái thứ 63.
- (Bổ sung B2, 2026-11-07) `sentinel.at = keep[CAP-2].at`: thông báo nhắc mở ứng dụng ở **cùng phút** với cái thứ
  63 và đứng sau nó theo thứ tự (at, id) — `"lvhv:sentinel"` lớn hơn mọi id `"lvhv:<uuid>:…"` khi so theo byte vì
  `s` lớn hơn mọi chữ số hex. Lý do: cái thứ 64 có thể cùng phút với cái thứ 63; nếu đặt `+ 1 phút` thì cái thứ 64
  bị bỏ mà không có thông báo nhắc nào đứng trước nó (vi phạm I3 của R12, 04 Bổ sung B2 mục 1). Người dùng không mở ứng dụng trong thời gian dài (ví dụ 63 lời nhắc ≈ vài tháng
  với 10 hồ sơ × 2 mốc) sẽ thấy thông báo nhắc mở ứng dụng thay vì mất lời nhắc lặng lẽ.
- (Bổ sung B3, 2026-11-07) Thông báo nhắc mở ứng dụng được thêm **khi còn bất kỳ lời nhắc tương lai nào không được
  đặt** — vì vượt 63 cái, hoặc vì nằm ngoài cửa sổ 400 ngày (`beyond`). Nó luôn ở cùng phút với lời nhắc cuối cùng
  đã đặt và xếp sau nó (thứ tự (at, id) như trên). Khi không đặt được lời nhắc nào trong cửa sổ mà vẫn có lời nhắc ở
  xa hơn, thông báo đặt tại `now + 400 ngày` — không muộn hơn lời nhắc sớm nhất chưa đặt (lời nhắc đó có `at > hi`).
  Khi mọi lời nhắc tương lai đều đã đặt (≤ 64 trong cửa sổ, không có gì ngoài cửa sổ) thì không có thông báo này.
  Bảo đảm thêm (I4 của R12): nếu có lời nhắc tương lai không được đặt thì có thông báo nhắc mở ứng dụng với `at` ≤
  lời nhắc sớm nhất trong số đó. Để biết `beyond` mà không phải tính mọi năm tới 3000, cài đặt quét các lần xuất hiện
  tới `hôm nay + 400 + 500 ngày` (đủ để mục lặp hằng năm, năm âm dài nhất 385 ngày, mốc nhắc trước tối đa 60 ngày,
  lộ ra một lời nhắc ngoài cửa sổ) và mọi sự kiện không lặp tới 3000-12-31.
- Trigger dùng `DateComponents` (năm, tháng, ngày, giờ, phút) **không** gắn múi giờ → chạy theo giờ đồng hồ của thiết
  bị, khớp §7.1.

### 7.5 Windows và PWA

- Windows: thông báo hệ thống đặt lịch trước (`ScheduledToastNotification`); dùng cùng thuật toán xoay vòng §7.4 với
  `CAP = 64` để có một cài đặt chung; lập lại khi mở ứng dụng. Ứng dụng không chạy nền thường trực (giữ ngưỡng 0
  lần đánh thức nền của v0.1). Chi tiết thành phần (ứng dụng chưa đóng gói cần AppUserModelID qua lối tắt Start
  Menu) quyết định ở đợt 3; nếu không đặt lịch được khi ứng dụng tắt thì ghi là giới hạn, không giả lập bằng tiến
  trình nền.
- PWA: **không có** thông báo đặt lịch khi trang đóng — API Notification Triggers đã ngừng phát triển (05 §7
  R-PWA-2), Web Push cần máy chủ (ngoài phạm vi). PWA có: danh sách "Sắp tới" trong ứng dụng; khi người dùng mở
  ứng dụng vào ngày có lời nhắc đến hạn, hiện thẻ nhắc trong trang (không cần quyền); và xuất **.ics** (§8) để lịch
  của hệ điều hành nhắc thay. Giao diện nói rõ giới hạn này.

### 7.6 Danh sách theo kỳ (R13) — Bổ sung B2, 2026-11-07

Đăng ký **trước khi viết mã** (04 Bổ sung B2 mục 4, R13). Mục này nói lại R13 cho chính xác; không đổi mệnh đề R13.

```
period_bounds(kind, anchor) -> (from, to)              # anchor: ngày dương bất kỳ; from ≤ anchor ≤ to
  "tuan":  w    := thứ(anchor)                          # 0 = Chủ nhật, 1 = Thứ Hai, …, 6 = Thứ Bảy
           from := anchor − ((w + 6) mod 7) ngày        # Thứ Hai của tuần chứa anchor
           to   := from + 6 ngày                        # Chủ nhật
  "thang": from := (anchor.y, anchor.m, 1);  to := ngày cuối tháng anchor.m (28/29/30/31)
  "quy":   q    := (anchor.m − 1) div 3                 # 0…3: tháng 1–3, 4–6, 7–9, 10–12
           from := (anchor.y, 3q + 1, 1);    to := ngày cuối tháng 3q + 3
  "nam":   from := (anchor.y, 1, 1);         to := (anchor.y, 12, 31)

events_in_period(ds, kind, anchor, filter) -> {kind, anchor, from, to, filter, items}
  (from, to) := period_bounds(kind, anchor)
  items := [ o ∈ L(from, to) | thuoc(o, filter) ]       # giữ nguyên thứ tự của L

prev(kind, anchor) := period_bounds(kind, from − 1 ngày)    # kỳ liền trước
next(kind, anchor) := period_bounds(kind, to + 1 ngày)      # kỳ liền sau
```

- `L(from, to)`: **đúng tập và đúng thứ tự** của danh sách §7.2, chỉ thay cửa sổ `[hôm nay, hôm nay + 30]` bằng
  `[from, to]` (**gồm cả hai đầu**): giỗ; tiên thường khi quy tắc nhắc giỗ của hồ sơ có `tien_thuong = true`; sinh
  nhật dương; sinh nhật âm khi `sinh_nhat_am = true`; sự kiện (dương lặp, âm lặp, một lần). Mỗi lần xuất hiện thuộc
  kỳ theo **ngày dương của chính nó**: lễ tiên thường (ngày `D − 1`) của một giỗ ngày đầu kỳ sau thuộc kỳ trước.
  Giống §7.2, danh sách **không phụ thuộc** công tắc nhắc chung, `rule.enabled` hay `offsets_days`.
- Bộ lọc `filter`:
  - `"tat_ca"`: mọi phần tử của `L`.
  - `"da_mat"`: lần xuất hiện của một hồ sơ **có ngày mất** (`death ≠ null`: giỗ, tiên thường); và sự kiện mà
    `person_ids` **không rỗng** và **mọi** id trỏ tới hồ sơ có ngày mất.
  - `"dang_song"`: phần còn lại — lần xuất hiện của hồ sơ không có ngày mất (sinh nhật dương, sinh nhật âm); sự
    kiện có ít nhất một người không có ngày mất (hoặc id không trỏ tới hồ sơ nào), hoặc không gắn với ai
    (`person_ids` rỗng).
  - Hệ quả (kiểm ở R13(c)): `da_mat` và `dang_song` rời nhau và hợp lại đúng bằng `tat_ca`, giữ thứ tự.
  - Ghi chú: R13 viết "sự kiện theo trạng thái của hồ sơ chứa nó". Sự kiện gắn với nhiều người có trạng thái khác
    nhau, hoặc không gắn với ai, không được R13 nói tới; quy tắc trên là lựa chọn của đặc tả này để hai bộ lọc là
    một phân hoạch, chờ chủ dự án xác nhận.
- Mỗi phần tử `items[i]`:

| Trường | Nội dung |
|---|---|
| `date` | Ngày dương của lần xuất hiện |
| `lunar` | `S2L(date)` theo phương pháp đang dùng: `{y, m, d, leap}` (với giỗ trùng `(Y, m, d', false)` của §2.2) |
| `class` | `gio`, `tien_thuong`, `sinh_nhat`, `sinh_nhat_am`, `su_kien_duong_lap`, `su_kien_am_lap`, `su_kien_khong_lap` (như A.3) |
| `entity_id` | Id hồ sơ (giỗ, tiên thường, sinh nhật) hoặc id sự kiện |
| `person_ids` | Hồ sơ liên quan: `[id hồ sơ]` cho lần xuất hiện của hồ sơ; `person_ids` của sự kiện (thứ tự đã lưu) cho sự kiện |
| `name` | Hồ sơ: tên hiển thị `danh_xung + " " + ho_ten` (không có dấu cách thừa khi `danh_xung` rỗng). Sự kiện: `tieu_de`, rỗng thì tên loại sự kiện (01 §3.4), không có loại thì `"Sự kiện"` |
| `key` | `occ.key` (§7.1): năm âm/dương Y hoặc `"once"` |
| `n` | Như A.3: giỗ lần thứ n = Y − năm âm mất (`null` khi không rõ năm), tiên thường lấy `n` của giỗ; sinh nhật dương: tuổi tròn = Y − năm sinh; sinh nhật âm: n = Y − năm âm sinh (tuổi âm = n + 1); sự kiện lặp: số năm = Y − năm gốc (`null` khi không rõ năm); sự kiện một lần: `null` |
| `label` | Nhãn giỗ §3 (`gio`, `gio_dau`, `gio_het`) cho giỗ và cho tiên thường (nhãn của giỗ hôm sau); `null` cho loại khác. Loại "tiên thường" đọc ở `class` |

- Giao diện hiển thị theo `class`, `label`, `n` (ví dụ "Giỗ đầu", "Tiên thường", "Sinh nhật (36 tuổi)", "Kỷ niệm 5
  năm ngày cưới") và có nút kỳ trước / kỳ sau (`prev`, `next`).
- Phương pháp lịch: mặc định của lõi (`Jpl` từ bản 0.1.4).

## 8. Tệp .ics (RFC 5545)

### 8.1 Phạm vi và cấu trúc

- Nội dung: mọi lần xuất hiện có ngày dương trong `[hôm nay, hôm nay + 10 năm)` của các mục người dùng chọn xuất
  (mặc định: tất cả có nhắc bật). Mỗi lần xuất hiện là **một VEVENT riêng với ngày cụ thể**; **không dùng RRULE**
  (lịch âm không biểu diễn được bằng RRULE; sinh nhật 29/02 cũng vậy).
- Tên tệp: `LichVanHoaViet-ngay-gio-<YYYY-MM-DD>.ics`; kiểu `text/calendar; charset=utf-8`; dòng kết thúc CRLF.

```
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Clevix Lab//Lich va Van hoa Viet 0.2.0//VI
CALSCALE:GREGORIAN
METHOD:PUBLISH
X-WR-CALNAME:Ngày giỗ và sinh nhật
X-WR-TIMEZONE:Asia/Ho_Chi_Minh
BEGIN:VTIMEZONE
TZID:Asia/Ho_Chi_Minh
BEGIN:STANDARD
DTSTART:19700101T000000
TZOFFSETFROM:+0700
TZOFFSETTO:+0700
TZNAME:+07
END:STANDARD
END:VTIMEZONE
BEGIN:VEVENT
UID:0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01-gio-2026@lich.clevix.vn
DTSTAMP:20261106T031500Z
SEQUENCE:1
DTSTART;VALUE=DATE:20260728
DTEND;VALUE=DATE:20260729
SUMMARY:Giỗ đầu Ông Nguyễn Văn A
DESCRIPTION:15/6 âm lịch năm Bính Ngọ (2026). Hưởng thọ 86 tuổi.
TRANSP:TRANSPARENT
CATEGORIES:Ngày giỗ
BEGIN:VALARM
ACTION:DISPLAY
DESCRIPTION:Còn 3 ngày: Giỗ đầu Ông Nguyễn Văn A
TRIGGER;RELATED=START:-P2DT17H
END:VALARM
BEGIN:VALARM
ACTION:DISPLAY
DESCRIPTION:Hôm nay: Giỗ đầu Ông Nguyễn Văn A
TRIGGER;RELATED=START:PT7H
END:VALARM
END:VEVENT
END:VCALENDAR
```

### 8.2 Quy tắc từng trường

| Trường | Quy tắc |
|---|---|
| `DTSTART`/`DTEND` | Sự kiện cả ngày: `VALUE=DATE`, `DTEND` = ngày kế tiếp (RFC 5545 §3.6.1: DTEND không tính vào sự kiện) |
| `TZID` | RFC 5545 **không cho phép** tham số `TZID` trên giá trị `DATE` (§3.2.19). Vì vậy tệp mang `VTIMEZONE` `Asia/Ho_Chi_Minh` và `X-WR-TIMEZONE` để các ứng dụng lịch đặt múi giờ mặc định cho lịch nhập vào, còn các ngày giỗ là ngày "trôi" (theo ngày, đúng bản chất ngày âm). Phương án khác — sự kiện có giờ `DTSTART;TZID=Asia/Ho_Chi_Minh:20260728T070000` — là mục mở **O-5** |
| `UID` | `<id thực thể>-<loại>-<khóa năm>@lich.clevix.vn`; loại ∈ `gio`, `tt` (tiên thường), `sn` (sinh nhật dương), `sna` (sinh nhật âm), `ev` (sự kiện); khóa năm = năm âm Y hoặc năm dương Y như `occ.key` (§7.1). **Ổn định**: xuất lại sau khi sửa hồ sơ cho cùng UID, nên ứng dụng lịch cập nhật thay vì nhân đôi. UID chỉ chứa UUID ngẫu nhiên, không chứa tên |
| `SEQUENCE` | `rev` của bản ghi nguồn (Person hoặc Event) |
| `DTSTAMP` | Thời điểm xuất, UTC |
| `VALARM` | Một khối cho mỗi phần tử `offsets_days`; `ACTION:DISPLAY` + `DESCRIPTION` + `TRIGGER` (đủ ba trường bắt buộc). Trigger tương đối với đầu ngày: `dur = time_local − k × 24h`, viết dạng chuẩn `[-]P{d}DT{h}H{m}M` bỏ phần bằng 0 (k=3, 07:00 → `-P2DT17H`; k=0, 07:00 → `PT7H`; k=1, 00:00 → `-P1D`). (Bổ sung B2, 2026-11-07) Nhắc đúng lúc bắt đầu ngày (`dur = 0`, tức k = 0 và `time_local = 00:00`) ghi **`TRIGGER;RELATED=START:PT0S`** |
| Tiên thường | Nếu bật: VEVENT cả ngày riêng (UID loại `tt`) vào ngày `D − 1`, một VALARM `PT{time_local}`. (Bổ sung B2, 2026-11-07) `SUMMARY` của VEVENT này là **`Tiên thường — `** + tiêu đề của giỗ hôm sau (ví dụ `Tiên thường — Giỗ đầu Ông Nguyễn Văn A`) |
| Văn bản | Thoát `\` → `\\`, `;` → `\;`, `,` → `\,`, xuống dòng → `\n` (RFC 5545 §3.3.11) |
| Gấp dòng | Mỗi dòng ≤ 75 **octet** (không tính CRLF); dòng tiếp theo bắt đầu bằng một dấu cách; không cắt giữa một ký tự UTF-8 nhiều byte (§3.1) |
| Riêng tư | Tệp chứa tên và ngày; trước khi xuất hiện hộp xác nhận "Tệp này chứa tên và ngày giỗ của … người. Bất kỳ ai có tệp đều đọc được." |

Giới hạn đã biết: một số ứng dụng lịch bỏ qua `VALARM` khi nhập tệp và áp lời nhắc mặc định của chúng — chưa kiểm
chứng ở G1, sẽ đo ở R4 (`04`).

## 9. Dữ liệu kiểm thử cố định

- Các ca G-A … G-E (§2.4), ví dụ §4.1–§4.3, §7.1 và tệp `schema/example-backup-v1.json`.
- Ca biên bắt buộc thêm: mất ngày 29 hoặc 30 của tháng 12 (giỗ rơi sang năm dương sau); mất ngày đầu/cuối miền
  (1900-01-01, 3000-12-31 — giỗ có ngày dương ngoài miền trả `None`, không lỗi); sinh 29/02/2000 (năm 2100 không nhuận →
  28/02/2100); nhắc 15 ngày trước một giỗ đầu tháng Giêng (rơi vào năm dương trước); offset 60 qua ranh giới năm;
  `time_local = "00:00"` và `"23:59"`.

## Phụ lục A — Định dạng tệp dump kiểm định (bổ sung 2026-11-06, đợt 1, trước khi có kết quả kiểm định)

Phụ lục này **chỉ bổ sung** định dạng tệp mà 04 chưa định nghĩa; không đổi quy tắc nào ở §1–§9. Công cụ:
`lvhv-memorial-dump` (Rust, crate `memorial/`) và bản đôi `tsport/test/memorial_dump.mjs` (TypeScript) nhận cùng lệnh
và cho cùng từng byte (`scripts/check-parity.sh`, phần 6). Mọi ngày dạng `YYYY-MM-DD`; giờ đồng hồ thiết bị dạng
`YYYY-MM-DDTHH:MM`; phương pháp lịch mặc định `Precise` (`--method hnd` để đối chiếu).

**A.1 R1 — `gio-dates` (JSONL, đúng mẫu 04 R1).** Mỗi dòng một lần giỗ:
`{"death":DateValue,"Y":Y,"lunar":{"y":Y,"m":m,"d":d'},"solar":{"y":…,"m":…,"d":…},"n":n|null,"label":"gio|gio_dau|gio_het"}`.
`death` là `DateValue` nhập (01 §2.1, thứ tự khóa `calendar, precision, year, month, day, leap`). Thứ tự dòng: theo
ngày mất nhập rồi theo Y tăng dần; năm có ngày dương ngoài miền bị bỏ (§2.2). `gio_het_nam` = 2 (`--gio-het 3` để đổi).
- `gio-dates solar FROM TO [N]` — 04 R1(a): mọi ngày mất dương, Y = năm âm mất + 1 … + N (mặc định 10).
- `gio-dates unknown FROM_Y TO_Y` — 04 R1(b): tháng 1…12 × ngày 1…30 × `leap` ∈ {false, true} (720 đầu vào), Y = FROM_Y … TO_Y.
- `gio-dates lunar FROM_Y TO_Y [N]` — 04 R1(c): mọi ngày âm tồn tại của năm âm (theo tháng 1…12, tháng thường trước tháng nhuận cùng số).
- `gio-dates list FILE [N]` — mỗi dòng của FILE là một `DateValue` JSON; ngày không rõ năm dùng Y = 1900 … 3000.

**A.2 R3 — `reminders`.** `reminders configs` → `cfg,presets,custom,offsets,tien_thuong,time_local,class` (140 544
dòng): `cfg` = chỉ số 0…; thứ tự lồng: mask nút có sẵn 0…63 (bit i ↔ {0,1,3,5,10,15}[i]) → mốc tự đặt (trống, 1…60)
→ `tien_thuong` (0, 1) → `time_local` (00:00, 07:00, 23:59) → `class` (`gio`, `sinh_nhat`, `sinh_nhat_am`,
`su_kien_duong_lap`, `su_kien_am_lap`, `su_kien_khong_lap`). `presets`/`offsets` ngăn bằng dấu cách; `offsets` là tập
sau khi gộp. `reminders DATASET [FROM TO] [--global-off|--rule-off]` → `cfg,id,at_local,type,occ_date`: với mỗi cấu hình, mọi lần xuất hiện của
lớp đó trong bộ dữ liệu có ngày trong [FROM, TO] (mặc định 2026-01-01 … 2036-12-31), mỗi lần dùng quy tắc của bản ghi
với `offsets_days`/`tien_thuong`/`time_local` thay bằng cấu hình (`id` giữ nguyên), `reminders_enabled = true`, `now` =
2026-01-01T00:00 giờ thiết bị, múi giờ +07:00. `type` = `k=<số>` hoặc `tt`. Dòng sắp theo (cfg, thời điểm, id).
Bộ dữ liệu cố định 40 hồ sơ/sự kiện: `memorial/testdata/r3-dataset.json` (sinh từ `make_r3_dataset.py` + `derive`).
(Bổ sung B3, 2026-11-07) Hai chế độ phụ, cùng định dạng và cùng 140 544 cấu hình: `--global-off` chạy với
`reminders_enabled = false` (công tắc nhắc chung tắt); `--rule-off` chạy với `rule.enabled = false` ở mọi quy tắc
(`reminders_enabled = true`). Cả hai **phải** ra 0 lời nhắc, tức tệp chỉ có dòng tiêu đề (04 R3 "Thêm").

**A.3 R11 — `upcoming DATASET FROM TO`** → `today,rank,days,date,class,entity_id,key,n,label`: với mỗi hôm nay,
danh sách §7.2 theo thứ tự hiển thị (`rank` 0…); `class` thêm `tien_thuong`; `key` = năm âm/dương hoặc `once`;
`n`, `label` trống khi không có. Phân xử thêm (sau tên, để xác định): lớp, id thực thể, khóa năm. (Bổ sung B3,
2026-11-07) Dòng `tien_thuong` có `key`, `n` và `label` **của lần giỗ ngày hôm sau** (như §7.6), ví dụ
`…,tien_thuong,<id>,2026,1,gio_dau`.

**A.4 R10 — `r10 a|b|c|d`.** (a) `birth,lunar_year,can_chi` cho mọi ngày sinh dương 1900-01-01 … 2026-12-31;
(b) `lunar_year,can_chi` 1899 … 3000; (c) `i,birth,death,nguong,birth_lunar_year,approx,death_lunar_year,value,label,error`
cho 10 000 cặp từ bộ sinh mulberry32 hạt 20261106 (kiểu sinh xoay vòng dương-đủ/dương-năm/âm-đủ/âm-năm, kiểu mất xoay
dương-đủ/âm-đủ, ngưỡng xoay 59/60/61; `birth`, `death` là `DateValue` JSON trong ngoặc kép CSV); (d) `year,rule,date`
cho sinh 29/02 (gốc 29/02/1896), mọi năm 1900 … 3000 × `feb28`/`mar1`.

**A.5 R12 — `r12 DATASET [--steps]`** → `size,scenario,opens,max_pending,delivered,sentinel_delivered,violations,missed,missed_without_sentinel,i1,i2,i3,i4`
cho cỡ 10/50/200 (bản ghi thứ i = bản ghi i mod 40 của bộ R3, id đổi 8 ký tự đầu thành i dạng hex) × kịch bản `p1`,
`p0.1`, `p0`; mô phỏng 2026-01-01 … 2028-12-31, mở ứng dụng lúc 09:00 (ngày đầu luôn mở; `p0.1` dùng mulberry32 hạt
20261106, một số mỗi ngày). `--steps` thêm khối `size,scenario,day,future,pending,sentinel,dropped,violations` cho mỗi
lần mở. Bất biến: I1 ≤ 64 chờ; I2 63 sớm nhất + thông báo nhắc mở ứng dụng tại `at₆₃` (Bổ sung B2, 2026-11-07; trước đó `at₆₃ + 1 phút`); I3 mọi lời nhắc bị bỏ có
`at ≥` thông báo nhắc mở ứng dụng.
- (Bổ sung B3, 2026-11-07) **I4**: sau mỗi lần mở, nếu có lời nhắc tương lai nào (trong hay ngoài cửa sổ) không đang
  chờ thì có thông báo nhắc mở ứng dụng với `at` ≤ lời nhắc sớm nhất trong số đó. I2 theo §7.4 mới: khi trong cửa sổ có
  ≥ 64 lời nhắc, 63 cái sớm nhất đang chờ, và thông báo nhắc mở ứng dụng có tại `at₆₃` nếu còn lời nhắc không được đặt
  (> 64 trong cửa sổ hoặc có lời nhắc ngoài cửa sổ), không có nếu không còn.
- Cột: `violations` = tổng số vi phạm; `i1`…`i4` = số vi phạm của từng bất biến (cộng lại bằng `violations`). I1 kiểm
  mọi ngày (sau khi giao), I2–I4 sau mỗi lần mở. `delivered` = số thông báo đã giao **gồm cả** thông báo nhắc mở ứng
  dụng; `sentinel_delivered` = phần trong đó là thông báo nhắc mở ứng dụng. Một thông báo được giao khi `at ≤` 09:00 của
  một ngày mô phỏng (giao trước khi mở ứng dụng ngày đó). `missed` = lời nhắc có `at` trong (09:00 ngày đầu, 09:00 ngày
  cuối] không được giao; `missed_without_sentinel` = phần trong đó không có thông báo nhắc mở ứng dụng nào được giao
  với `at` ≤ lời nhắc đó (thông tin, không phải bất biến).
- Số ngẫu nhiên: ngày đầu (2026-01-01) luôn mở và **không** rút số; từ ngày thứ hai, mỗi ngày rút đúng một số
  `r = next_u32()` (chỉ ở `p0.1`) và mở khi `r × 10 < 2³²`. `p1` mở mọi ngày, `p0` chỉ mở ngày đầu; hai kịch bản này
  không rút số.
- Tập lời nhắc của mô phỏng: mọi lời nhắc của các lần xuất hiện có ngày trong [2026-01-01, 2028-12-31 + 900 ngày] và mọi
  lời nhắc của sự kiện không lặp tới 3000-12-31 (đủ cho I4 theo §7.4).

**A.6 R13 — `r13 DATASET [FROM TO]`** (Bổ sung B2, 2026-11-07, trước khi viết mã; mặc định FROM = 2026-01-01,
TO = 2036-12-31). JSONL, **một dòng cho mỗi (kỳ, bộ lọc)**:
`{"kind":K,"anchor":"YYYY-MM-DD","from":"YYYY-MM-DD","to":"YYYY-MM-DD","filter":F,"items":[ITEM,…]}`, với
`ITEM` = `{"date":"YYYY-MM-DD","lunar":{"y":…,"m":…,"d":…,"leap":true|false},"class":"…","entity_id":"…","person_ids":["…",…],"name":"…","key":"…","n":số|null,"label":"…"|null}`
(trường và nghĩa ở §7.6; `key` luôn là chuỗi, ví dụ `"2026"` hoặc `"once"`).
- Các kỳ: mọi tuần, tháng, quý, năm (theo `period_bounds` §7.6) có **ít nhất một ngày** trong [FROM, TO]. `anchor`
  = ngày đầu tiên của kỳ nằm trong [FROM, TO] (tức `max(from, FROM)`); `from`, `to` là biên **đầy đủ** của kỳ (không
  cắt theo FROM/TO; tuần đầu của dải mặc định là 2025-12-29 … 2026-01-04) và `items` tính trên toàn kỳ.
- Thứ tự dòng: `kind` theo `tuan`, `thang`, `quy`, `nam`; trong mỗi `kind` theo `from` tăng dần; trong mỗi kỳ theo
  bộ lọc `tat_ca`, `da_mat`, `dang_song`. `items` theo thứ tự §7.2. Không có khoảng trắng ngoài chuỗi; thứ tự khóa
  đúng như mẫu.
- Chuỗi JSON: ký tự ngoài ASCII ghi nguyên UTF-8 (không thoát `\u`); chỉ thoát `"` → `\"`, `\` → `\\`, xuống
  dòng `\n`, `\r`, tab `\t`, và ký tự điều khiển khác < U+0020 dạng `\u00xx` (chữ thường).
- Dải mặc định cho 575 tuần + 132 tháng + 44 quý + 11 năm = 762 kỳ × 3 bộ lọc = **2 286 dòng**. Phương pháp mặc
  định `Jpl` (`--method` để đổi).
- (Bổ sung B3, 2026-11-07) R13 chạy trên **hai** bộ dữ liệu, cùng lệnh: bộ R3 (`memorial/testdata/r3-dataset.json`,
  không có sự kiện gắn với hồ sơ) và bộ bổ sung `memorial/testdata/r13-linked.json` (sinh bằng
  `python3 memorial/testdata/make_r13_linked.py`): cùng hồ sơ và cài đặt với bộ R3; 13 sự kiện của bộ R3 được gắn
  `person_ids` và thêm 4 sự kiện, phủ các ca: một người sống; hai người sống; một người đã mất; hai người đã mất;
  hỗn hợp đã mất + sống; không gắn ai; sự kiện âm lặp không rõ năm gắn người đã mất, cùng ngày với giỗ của người đó;
  ngày cưới 29/02 của hai người đã mất với quy tắc nhắc tắt; sự kiện không tiêu đề (hiện tên loại) không có quy tắc
  nhắc; sự kiện một lần đã qua. Mọi `person_ids` trỏ tới hồ sơ có thật (V3).

**A.7 .ics — `ics DATASET TODAY DTSTAMP`** (trước Bổ sung B2 đánh số A.6): tệp §8 cho mọi mục có quy tắc nhắc bật, `[TODAY, TODAY + 10 năm)`. Chi
tiết đã chọn trong phạm vi §8: thứ tự VEVENT theo §7.2; VALARM theo `offsets_days` giảm dần (như ví dụ §8.1);
`TRIGGER` bằng 0 viết `PT0S`; VEVENT tiên thường có SUMMARY `Tiên thường — …` và một VALARM nội dung tiên thường.
