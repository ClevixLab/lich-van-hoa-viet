//! Âm lịch Việt Nam (UTC+7; trước 1968-01-01 là UTC+8 — `astro::local_day_of`, docs/03 §16). Hai phương pháp chọn được: `Method::Hnd` (Hồ Ngọc Đức, mặc định)
//! và `Method::Precise` (Meeus ch.49 đầy đủ + ΔT Espenak–Meeus; Trung khí theo cùng mô hình
//! Mặt Trời biểu kiến dùng để hiển thị tiết khí).

use crate::astro::{
    apparent_sun_longitude_ut, hnd_new_moon, hnd_new_moon_day, hnd_sun_sector, local_day_of,
    local_midnight_ut, precise_new_moon_ut,
};
use crate::error::{
    check_jdn, checked_jdn, jdn_in_range, DateError, BIRTH_MIN_YEAR, INTERNAL_MAX_YY,
    INTERNAL_MIN_YY, MAX_LUNAR_YEAR, MIN_LUNAR_YEAR, MIN_YEAR,
};
use crate::jd::{date_from_jdn, jdn_from_date};

/// Phương pháp tính âm lịch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    /// Thuật toán Hồ Ngọc Đức (phương pháp tham chiếu theo hợp đồng §2). Trung khí dùng kinh độ
    /// Mặt Trời rút gọn của HND, KHÁC mô hình dùng để hiển thị tiết khí.
    Hnd,
    /// Sóc theo Meeus ch.49 đầy đủ (hiệu chỉnh tuần hoàn + 14 số hạng hành tinh) + ΔT
    /// Espenak–Meeus; Trung khí theo kinh độ biểu kiến (VSOP87 rút gọn + chương động + quang
    /// sai) — cùng mô hình với tiết khí hiển thị.
    Precise,
}

/// Phương pháp mặc định của các hàm không có hậu tố `_with`: `Precise` (quyết định của chủ dự án,
/// docs/02 "Bổ sung A3", 2026-11-06). `Hnd` vẫn giữ để đối chiếu qua các hàm `_with(Method::Hnd, …)`.
pub const DEFAULT_METHOD: Method = Method::Precise;

/// Ngày (JDN, giờ lịch) chứa Sóc thứ k theo phương pháp (nội bộ, không kiểm miền).
pub(crate) fn new_moon_day(method: Method, k: i64) -> i64 {
    match method {
        Method::Hnd => hnd_new_moon_day(k),
        Method::Precise => local_day_of(precise_new_moon_ut(k)),
    }
}

/// Thời điểm Sóc thứ k (JD UT) theo phương pháp (nội bộ, không kiểm miền).
pub(crate) fn new_moon_ut_with(method: Method, k: i64) -> f64 {
    match method {
        Method::Hnd => hnd_new_moon(k),
        Method::Precise => precise_new_moon_ut(k),
    }
}

/// Cung 30° (0..11) của Mặt Trời lúc 00:00 giờ lịch ngày `jdn` theo phương pháp (nội bộ).
pub(crate) fn sun_sector(method: Method, jdn: i64) -> i32 {
    match method {
        Method::Hnd => hnd_sun_sector(jdn),
        Method::Precise => {
            let l = apparent_sun_longitude_ut(local_midnight_ut(jdn));
            // `rem_euclid` (không `clamp`): kinh độ làm tròn thành đúng 360.0 thuộc cung 0 (C-6).
            ((l / 30.0).floor() as i32).rem_euclid(12)
        }
    }
}

/// Ngày âm lịch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LunarDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub leap: bool,
    /// Số ngày của tháng âm này (29 hoặc 30).
    pub month_len: u32,
}

/// Ngày dương lịch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SolarDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

/// Hằng số dùng để ước lượng chỉ số Sóc k từ JDN (Hồ Ngọc Đức).
const K_EPOCH: f64 = 2415021.076998695;
const SYNODIC: f64 = 29.530588853;

/// Số bước hiệu chỉnh tối đa của `k_of_month_start` mỗi chiều (ước lượng ban đầu sai lệch
/// nhiều nhất 1 trong miền nội bộ; vượt cận nghĩa là đầu vào ngoài miền → lỗi, không lặp mãi).
const MAX_K_ADJUST: u32 = 4;

/// Chỉ số k của Sóc bắt đầu tháng có ngày bắt đầu là `start_jdn` (đã biết là một ngày Sóc).
/// Vòng hiệu chỉnh có cận (RS-01).
fn k_of_month_start(method: Method, start_jdn: i64) -> Result<i64, DateError> {
    let mut k = ((start_jdn as f64 - K_EPOCH) / SYNODIC + 0.5).floor() as i64;
    let mut n = 0;
    while new_moon_day(method, k) > start_jdn {
        k -= 1;
        n += 1;
        if n > MAX_K_ADJUST {
            return Err(DateError::Computation);
        }
    }
    let mut n = 0;
    while new_moon_day(method, k + 1) <= start_jdn {
        k += 1;
        n += 1;
        if n > MAX_K_ADJUST {
            return Err(DateError::Computation);
        }
    }
    Ok(k)
}

/// JDN ngày bắt đầu tháng 11 âm lịch (tháng chứa Đông chí) của năm dương `yy`.
/// Chỉ dùng nội bộ, với `yy` thuộc miền nội bộ (đã kiểm ở `YearTable::new`).
fn lunar_month11(method: Method, yy: i32) -> i64 {
    let off = jdn_from_date(yy, 12, 31) as f64 - 2415021.0;
    let k = (off / SYNODIC).floor() as i64;
    let nm = new_moon_day(method, k);
    // Cung ≥ 9 (≥ 270°) lúc đầu ngày Sóc: Đông chí đã qua trước tháng này → lấy tháng trước.
    if sun_sector(method, nm) >= 9 {
        new_moon_day(method, k - 1)
    } else {
        nm
    }
}

/// Bảng tháng của một "tuế" âm lịch: từ tháng 11 năm dương `yy-1` đến trước tháng 11 năm `yy`.
#[derive(Debug, Clone)]
pub struct YearTable {
    /// Năm dương `yy` (tháng 11 cuối nằm trong năm này).
    pub yy: i32,
    /// Số tháng (12 hoặc 13).
    pub n: usize,
    /// Ngày bắt đầu từng tháng; phần tử `n` là ngày bắt đầu tháng 11 kế tiếp.
    pub starts: [i64; 14],
    /// Số tháng âm (1..12).
    pub month: [u32; 13],
    /// Có phải tháng nhuận.
    pub leap: [bool; 13],
}

impl YearTable {
    /// Lập bảng tháng giữa tháng 11 của năm `yy-1` và tháng 11 của năm `yy`.
    ///
    /// `Err(OutOfRange)` nếu `yy` ngoài miền nội bộ 1900–3101; `Err(Computation)` nếu bảng vi
    /// phạm bất biến (12 hoặc 13 tháng, tháng dài 29/30 ngày, năm 13 tháng có tháng nhuận) —
    /// thay cho `assert!`/panic trước đây (RS-02).
    pub fn new(method: Method, yy: i32) -> Result<YearTable, DateError> {
        if !(INTERNAL_MIN_YY..=INTERNAL_MAX_YY).contains(&yy) {
            return Err(DateError::OutOfRange {
                min: INTERNAL_MIN_YY,
                max: INTERNAL_MAX_YY,
            });
        }
        let a11 = lunar_month11(method, yy - 1);
        let b11 = lunar_month11(method, yy);
        let k0 = k_of_month_start(method, a11)?;
        let mut starts = [0i64; 14];
        let mut n = 0usize;
        loop {
            let s = new_moon_day(method, k0 + n as i64);
            if s >= b11 {
                break;
            }
            if n >= 13 {
                return Err(DateError::Computation);
            }
            starts[n] = s;
            n += 1;
        }
        if n < 12 {
            return Err(DateError::Computation);
        }
        starts[n] = b11;
        let mut month = [0u32; 13];
        let mut leap = [false; 13];
        // vị trí tháng nhuận: tháng đầu tiên sau tháng 11 không chứa Trung khí
        let mut leap_idx = usize::MAX;
        if n == 13 {
            for i in 1..13 {
                if sun_sector(method, starts[i]) == sun_sector(method, starts[i + 1]) {
                    leap_idx = i;
                    break;
                }
            }
            if leap_idx == usize::MAX {
                return Err(DateError::Computation);
            }
        }
        if (0..n).any(|i| !(29..=30).contains(&(starts[i + 1] - starts[i]))) {
            return Err(DateError::Computation);
        }
        let mut num = 11u32;
        for i in 0..n {
            if i > 0 {
                if i == leap_idx {
                    leap[i] = true;
                } else {
                    num = if num == 12 { 1 } else { num + 1 };
                }
            }
            month[i] = num;
        }
        Ok(YearTable {
            yy,
            n,
            starts,
            month,
            leap,
        })
    }

    /// Năm âm lịch của tháng thứ i trong bảng (`None` nếu `i >= n`).
    ///
    /// Vì sao `i < 4` (C-12): bảng bắt đầu ở tháng 11 của năm âm `yy−1` (chỉ số 0), tiếp theo
    /// là tháng 12 (chỉ số 1, hoặc 2 nếu có tháng 11 nhuận), có thể thêm một tháng nhuận 11/12,
    /// nên mọi tháng 11–12 của năm `yy−1` nằm ở chỉ số 0..=3. Tháng 11–12 ở chỉ số ≥ 4 không
    /// tồn tại vì bảng chỉ có 12–13 tháng và tháng 1..10 của năm `yy` chiếm ít nhất 10 vị trí.
    pub fn lunar_year(&self, i: usize) -> Option<i32> {
        if i >= self.n {
            return None;
        }
        Some(if self.month[i] >= 11 && i < 4 {
            self.yy - 1
        } else {
            self.yy
        })
    }

    /// Tháng nhuận của tuế này (số tháng), nếu có.
    pub fn leap_month(&self) -> Option<u32> {
        (0..self.n).find(|&i| self.leap[i]).map(|i| self.month[i])
    }
}

/// Đổi JDN sang ngày âm (phương pháp mặc định). `Err(OutOfRange)` nếu JDN ngoài
/// 1900-01-01 … 3000-12-31.
pub fn jdn_to_lunar(jdn: i64) -> Result<LunarDate, DateError> {
    jdn_to_lunar_with(DEFAULT_METHOD, jdn)
}

/// Đổi JDN sang ngày âm theo phương pháp chỉ định (miền công khai 1900–3000).
pub fn jdn_to_lunar_with(method: Method, jdn: i64) -> Result<LunarDate, DateError> {
    check_jdn(jdn)?;
    jdn_to_lunar_internal(method, jdn)
}

/// Như `jdn_to_lunar_with` nhưng chỉ giới hạn bởi miền nội bộ của `YearTable`.
fn jdn_to_lunar_internal(method: Method, jdn: i64) -> Result<LunarDate, DateError> {
    let (yy, _, _) = date_from_jdn(jdn);
    let a = lunar_month11(method, yy);
    let table = YearTable::new(method, if jdn >= a { yy + 1 } else { yy })?;
    // Tìm có cận (không còn `i -= 1` tràn `usize`, RS-04).
    let i = (0..table.n)
        .rev()
        .find(|&i| table.starts[i] <= jdn)
        .ok_or(DateError::Computation)?;
    let day = jdn - table.starts[i] + 1;
    let month_len = table.starts[i + 1] - table.starts[i];
    // Hậu kiểm rẻ (RS-02): 1 ≤ ngày ≤ độ dài tháng ∈ {29, 30}.
    if !(29..=30).contains(&month_len) || !(1..=month_len).contains(&day) {
        return Err(DateError::Computation);
    }
    Ok(LunarDate {
        year: table.lunar_year(i).ok_or(DateError::Computation)?,
        month: table.month[i],
        day: u32::try_from(day).map_err(|_| DateError::Computation)?,
        leap: table.leap[i],
        month_len: u32::try_from(month_len).map_err(|_| DateError::Computation)?,
    })
}

/// Dương lịch → âm lịch (phương pháp mặc định).
///
/// `Err(InvalidDate)` nếu ngày dương không tồn tại; `Err(OutOfRange)` nếu ngoài 1900–3000.
pub fn solar_to_lunar(y: i32, m: u32, d: u32) -> Result<LunarDate, DateError> {
    solar_to_lunar_with(DEFAULT_METHOD, y, m, d)
}

/// Dương lịch → âm lịch theo phương pháp chỉ định.
pub fn solar_to_lunar_with(method: Method, y: i32, m: u32, d: u32) -> Result<LunarDate, DateError> {
    jdn_to_lunar_internal(method, checked_jdn(y, m, d, MIN_YEAR)?)
}

/// Năm âm lịch của một **ngày sinh** dương lịch (phương pháp mặc định) — dùng để tính tuổi
/// Can Chi (`birth_lunar_year` của `find_good_days`). Người sinh trước Tết thuộc năm âm trước.
///
/// Miền: 1900-01-01 … 3000-12-31 (trùng miền công khai). `Err(InvalidDate)` nếu ngày không
/// tồn tại, `Err(OutOfRange { min: 1900, max: 3000 })` nếu ngoài miền.
pub fn lunar_year_of_birth(y: i32, m: u32, d: u32) -> Result<i32, DateError> {
    lunar_year_of_birth_with(DEFAULT_METHOD, y, m, d)
}

/// `lunar_year_of_birth` theo phương pháp chỉ định.
pub fn lunar_year_of_birth_with(method: Method, y: i32, m: u32, d: u32) -> Result<i32, DateError> {
    let j = checked_jdn(y, m, d, BIRTH_MIN_YEAR)?;
    Ok(jdn_to_lunar_internal(method, j)?.year)
}

/// Âm lịch → JDN (phương pháp mặc định). `None` nếu ngày âm không tồn tại, năm âm ngoài
/// 1899–3000, hoặc kết quả nằm ngoài miền công khai 1900-01-01 … 3000-12-31.
pub fn lunar_to_jdn(ly: i32, lm: u32, ld: u32, leap: bool) -> Option<i64> {
    lunar_to_jdn_with(DEFAULT_METHOD, ly, lm, ld, leap)
}

/// Âm lịch → JDN theo phương pháp chỉ định.
pub fn lunar_to_jdn_with(method: Method, ly: i32, lm: u32, ld: u32, leap: bool) -> Option<i64> {
    if !(MIN_LUNAR_YEAR..=MAX_LUNAR_YEAR).contains(&ly)
        || !(1..=12).contains(&lm)
        || !(1..=30).contains(&ld)
    {
        return None;
    }
    // Tháng 11, 12 nằm trong tuế kết thúc ở năm dương ly+1; tháng 1..10 trong tuế kết thúc ở ly.
    let yy = if lm >= 11 { ly + 1 } else { ly };
    let t = YearTable::new(method, yy).ok()?;
    for i in 0..t.n {
        if t.month[i] == lm && t.leap[i] == leap && t.lunar_year(i) == Some(ly) {
            let len = t.starts[i + 1] - t.starts[i];
            if i64::from(ld) > len {
                return None;
            }
            let j = t.starts[i] + i64::from(ld) - 1;
            return jdn_in_range(j).then_some(j);
        }
    }
    None
}

/// Âm lịch → dương lịch. `None` nếu ngày âm không tồn tại (ví dụ tháng nhuận không có, ngày 30 tháng thiếu).
pub fn lunar_to_solar(ly: i32, lm: u32, ld: u32, leap: bool) -> Option<SolarDate> {
    lunar_to_solar_with(DEFAULT_METHOD, ly, lm, ld, leap)
}

/// Âm lịch → dương lịch theo phương pháp chỉ định.
pub fn lunar_to_solar_with(
    method: Method,
    ly: i32,
    lm: u32,
    ld: u32,
    leap: bool,
) -> Option<SolarDate> {
    lunar_to_jdn_with(method, ly, lm, ld, leap).map(|j| {
        let (year, month, day) = date_from_jdn(j);
        SolarDate { year, month, day }
    })
}

/// Một lần Sóc (trăng mới) trong danh sách chẩn đoán.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NewMoon {
    /// Chỉ số k theo quy ước Hồ Ngọc Đức (k = 0: Sóc 1900-01-01).
    pub k: i64,
    /// Thời điểm (JD, UT).
    pub jd_ut: f64,
    /// JDN của ngày (giờ lịch) chứa Sóc.
    pub local_jdn: i64,
}

/// Mọi lần Sóc có ngày (giờ lịch) trong `[from_jdn, to_jdn]` — công cụ chẩn đoán (`lvhv-dump
/// newmoons`). `Err(OutOfRange)` nếu một đầu nằm ngoài miền công khai, `Err(InvertedRange)` nếu
/// `from_jdn > to_jdn`.
pub fn new_moons_between(
    method: Method,
    from_jdn: i64,
    to_jdn: i64,
) -> Result<Vec<NewMoon>, DateError> {
    check_jdn(from_jdn)?;
    check_jdn(to_jdn)?;
    if from_jdn > to_jdn {
        return Err(DateError::InvertedRange);
    }
    let mut k = ((from_jdn as f64 - K_EPOCH) / SYNODIC).floor() as i64 - 1;
    let mut out = Vec::new();
    loop {
        let day = new_moon_day(method, k);
        if day > to_jdn {
            break;
        }
        if day >= from_jdn {
            out.push(NewMoon {
                k,
                jd_ut: new_moon_ut_with(method, k),
                local_jdn: day,
            });
        }
        k += 1;
    }
    Ok(out)
}
