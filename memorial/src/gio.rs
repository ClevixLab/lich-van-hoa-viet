//! Ngày mất (02 §1), ngày giỗ hằng năm (02 §2), nhãn giỗ đầu / giỗ hết (02 §3),
//! tuổi âm và hưởng thọ / hưởng dương (02 §4), sinh nhật dương và 29/02 (02 §6).

use crate::cal::Cal;
use crate::model::*;
use lvhv_core::canchi::{can_chi_name, year_can, year_chi};
use lvhv_core::jd::{is_leap_year, is_valid_solar};
use lvhv_core::{Method, DEFAULT_METHOD, MAX_LUNAR_YEAR, MAX_YEAR, MIN_LUNAR_YEAR, MIN_YEAR};
use std::fmt;

/// Lỗi nhập liệu (01 §6.1). `code` ổn định (ASCII), dùng chung với TypeScript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputError {
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for InputError {}

fn ie(code: &'static str, message: String) -> InputError {
    InputError { code, message }
}

fn out_of_range() -> InputError {
    ie(
        "OutOfRange",
        format!("Năm ngoài phạm vi đổi lịch ({MIN_YEAR}–{MAX_YEAR})"),
    )
}

/// Kiểm một ngày âm có năm trong miền đổi lịch (1899–3000): tháng (và cờ nhuận) tồn tại, ngày ≤
/// độ dài tháng. Thông báo đúng mẫu 01 §6.1.
pub fn check_lunar_exists(cal: &Cal, y: i32, m: u32, d: u32, leap: bool) -> Result<(), InputError> {
    if !(1..=12).contains(&m) || !(1..=30).contains(&d) {
        return Err(ie(
            "InvalidLunarDate",
            format!("Ngày âm {d}/{m} không hợp lệ"),
        ));
    }
    match cal.month_len(y, m, leap) {
        None if leap => Err(ie(
            "LunarMonthMissing",
            format!("Năm {y} không có tháng {m} nhuận"),
        )),
        None => Err(ie(
            "Computation",
            format!("Không tìm thấy tháng {m} năm {y}"),
        )),
        Some(len) if d > len => Err(ie(
            "LunarDayMissing",
            if leap {
                format!("Tháng {m} nhuận âm lịch năm {y} chỉ có {len} ngày")
            } else {
                format!("Tháng {m} âm lịch năm {y} chỉ có {len} ngày")
            },
        )),
        Some(_) => Ok(()),
    }
}

fn want<T>(v: Option<T>, what: &str) -> Result<T, InputError> {
    v.ok_or_else(|| ie("MissingField", format!("Thiếu {what}")))
}

/// Phần ngày của `save_death` (02 §1) — `tuoi` tính riêng bằng `tuoi_mat`.
pub fn convert_death(
    cal: &Cal,
    input: &DateValue,
) -> Result<(LunarDay, Option<Ymd>, LunarSource), InputError> {
    match (input.calendar, input.precision) {
        (Calendar::Solar, Precision::Day) => {
            let (y, m, d) = (
                want(input.year, "năm")?,
                want(input.month, "tháng")?,
                want(input.day, "ngày")?,
            );
            if input.leap {
                return Err(ie("InvalidLeap", "Ngày dương không có tháng nhuận".into()));
            }
            if !is_valid_solar(y, m, d) {
                return Err(ie(
                    "InvalidSolarDate",
                    format!("Ngày {d:02}/{m:02}/{y} không tồn tại"),
                ));
            }
            if !(MIN_YEAR..=MAX_YEAR).contains(&y) {
                return Err(out_of_range());
            }
            let l = lvhv_core::solar_to_lunar_with(cal.method(), y, m, d)
                .map_err(|_| out_of_range())?;
            Ok((
                LunarDay {
                    year: Some(l.year),
                    month: l.month,
                    day: l.day,
                    leap: l.leap,
                },
                Some(Ymd::new(y, m, d)),
                LunarSource::Converted,
            ))
        }
        (Calendar::Lunar, Precision::Day) => {
            let (y, m, d) = (
                want(input.year, "năm")?,
                want(input.month, "tháng")?,
                want(input.day, "ngày")?,
            );
            if !(1..=12).contains(&m) || !(1..=30).contains(&d) {
                return Err(ie(
                    "InvalidLunarDate",
                    format!("Ngày âm {d}/{m} không hợp lệ"),
                ));
            }
            let solar = if (MIN_LUNAR_YEAR..=MAX_LUNAR_YEAR).contains(&y) {
                check_lunar_exists(cal, y, m, d, input.leap)?;
                // Ngày tồn tại nhưng ngày dương có thể nằm ngoài 1900-01-01 … 3000-12-31.
                cal.l2s(y, m, d, input.leap)
                    .map(|s| Ymd::new(s.year, s.month, s.day))
            } else {
                None
            };
            Ok((
                LunarDay {
                    year: Some(y),
                    month: m,
                    day: d,
                    leap: input.leap,
                },
                solar,
                LunarSource::Entered,
            ))
        }
        (Calendar::Lunar, Precision::MonthDay) => {
            if input.year.is_some() {
                return Err(ie(
                    "InvalidPrecision",
                    "Không rõ năm thì năm phải để trống".into(),
                ));
            }
            let (m, d) = (want(input.month, "tháng")?, want(input.day, "ngày")?);
            if !(1..=12).contains(&m) || !(1..=30).contains(&d) {
                return Err(ie(
                    "InvalidLunarDate",
                    format!("Ngày âm {d}/{m} không hợp lệ"),
                ));
            }
            Ok((
                LunarDay {
                    year: None,
                    month: m,
                    day: d,
                    leap: input.leap,
                },
                None,
                LunarSource::Entered,
            ))
        }
        _ => Err(ie(
            "PrecisionNotAllowed",
            "Ngày mất cần đủ ngày, hoặc ngày và tháng âm lịch".into(),
        )),
    }
}

/// `save_death` (02 §1, 01 §3.2) với phương pháp mặc định.
pub fn save_death(
    input: &DateValue,
    birth: Option<&BirthInfo>,
    nguong: i32,
) -> Result<DeathInfo, InputError> {
    save_death_with(Cal::get(DEFAULT_METHOD), input, birth, nguong)
}

pub fn save_death_with(
    cal: &Cal,
    input: &DateValue,
    birth: Option<&BirthInfo>,
    nguong: i32,
) -> Result<DeathInfo, InputError> {
    let (lunar, solar, lunar_source) = convert_death(cal, input)?;
    if let (Some(b), Some(dy)) = (birth.and_then(|b| b.lunar_year), lunar.year) {
        if dy < b {
            return Err(ie("DeathBeforeBirth", "Ngày mất trước ngày sinh".into()));
        }
    }
    let tuoi = tuoi_mat(birth, lunar.year, nguong)?;
    Ok(DeathInfo {
        input: *input,
        lunar,
        solar,
        lunar_source,
        method: cal.method(),
        tuoi,
    })
}

/// Một lần giỗ (02 §2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Anniversary {
    /// Năm âm Y của lần giỗ.
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub solar: Ymd,
    /// Lần thứ n (`None` khi không biết năm mất).
    pub n: Option<i32>,
    pub label: GioLabel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GioLabel {
    /// "Ngày giỗ" (không rõ năm) hoặc "Giỗ" (lần thứ n bất kỳ khác).
    Gio,
    GioDau,
    GioHet,
}

impl GioLabel {
    pub fn code(&self) -> &'static str {
        match self {
            GioLabel::Gio => "gio",
            GioLabel::GioDau => "gio_dau",
            GioLabel::GioHet => "gio_het",
        }
    }
}

/// `label_of` (02 §3).
pub fn label_of(n: Option<i32>, gio_het_nam: u32) -> GioLabel {
    match n {
        None => GioLabel::Gio,
        Some(1) => GioLabel::GioDau,
        Some(k) if i64::from(k) == i64::from(gio_het_nam) => GioLabel::GioHet,
        Some(_) => GioLabel::Gio,
    }
}

/// Văn bản nhãn: "Ngày giỗ" khi không rõ năm, "Giỗ đầu", "Giỗ hết", "Giỗ".
pub fn label_text(label: GioLabel, n: Option<i32>) -> &'static str {
    match (label, n) {
        (GioLabel::Gio, None) => "Ngày giỗ",
        (GioLabel::Gio, Some(_)) => "Giỗ",
        (GioLabel::GioDau, _) => "Giỗ đầu",
        (GioLabel::GioHet, _) => "Giỗ hết",
    }
}

/// `anniversary(death, Y)` (02 §2.2) — dùng chung cho sinh nhật âm và sự kiện âm lặp (02 §5).
/// `orig_year` = năm âm gốc (`None` = không rõ năm), `(m, d)` = tháng/ngày âm gốc (bỏ cờ nhuận).
pub fn anniversary(
    cal: &Cal,
    orig_year: Option<i32>,
    m: u32,
    d: u32,
    gio_het_nam: u32,
    y: i32,
) -> Option<Anniversary> {
    if let Some(oy) = orig_year {
        if y <= oy {
            return None;
        }
    }
    if !(MIN_LUNAR_YEAR..=MAX_LUNAR_YEAR).contains(&y) || !(1..=12).contains(&m) {
        return None;
    }
    let len = cal.month_len(y, m, false)?;
    let d2 = if d == 30 && len == 29 { 29 } else { d };
    let s = cal.l2s(y, m, d2, false)?;
    let n = orig_year.map(|oy| y - oy);
    Some(Anniversary {
        year: y,
        month: m,
        day: d2,
        solar: Ymd::new(s.year, s.month, s.day),
        n,
        label: label_of(n, gio_het_nam),
    })
}

/// Năm âm chứa ngày dương (LYOB), kẹp vào miền: trước 1900 → 1899, sau 3000 → 3000.
pub fn lyob_clamped(cal: &Cal, day: Ymd) -> i32 {
    if day.y < MIN_YEAR {
        return MIN_LUNAR_YEAR;
    }
    if day.y > MAX_YEAR {
        return MAX_LUNAR_YEAR;
    }
    lvhv_core::solar_to_lunar_with(cal.method(), day.y, day.m, day.d)
        .map(|l| l.year)
        .unwrap_or(MAX_LUNAR_YEAR)
}

/// `anniversaries_in` (02 §2.3): các lần (âm lặp) có ngày dương trong `[from, to]`, sắp theo ngày.
pub fn anniversaries_in(
    cal: &Cal,
    orig_year: Option<i32>,
    m: u32,
    d: u32,
    gio_het_nam: u32,
    from: Ymd,
    to: Ymd,
) -> Vec<Anniversary> {
    let mut out = Vec::new();
    if from > to {
        return out;
    }
    let (a, b) = (lyob_clamped(cal, from) - 1, lyob_clamped(cal, to) + 1);
    for y in a..=b {
        if let Some(x) = anniversary(cal, orig_year, m, d, gio_het_nam, y) {
            if from <= x.solar && x.solar <= to {
                out.push(x);
            }
        }
    }
    out.sort_by_key(|x| x.solar);
    out
}

/// Can Chi của năm âm (hợp đồng kiến trúc §3).
pub fn can_chi_year(ly: i32) -> String {
    can_chi_name(year_can(ly), year_chi(ly)).unwrap_or_default()
}

/// Trường dẫn xuất của `BirthInfo` (02 §4.1, 01 §3.3). Ngày dương đủ ngày ngoài 1900–3000 hoặc
/// ngày âm có năm ngoài 1899–3000: lưu được nhưng không đổi lịch → `lunar_year = null`.
pub fn derive_birth(cal: &Cal, input: &DateValue) -> Result<BirthInfo, InputError> {
    let mut b = BirthInfo {
        input: *input,
        lunar_year: None,
        lunar_year_approx: false,
        can_chi: None,
        lunar: None,
        solar: None,
    };
    match (input.calendar, input.precision) {
        (Calendar::Solar, Precision::Day) => {
            let (y, m, d) = (
                want(input.year, "năm")?,
                want(input.month, "tháng")?,
                want(input.day, "ngày")?,
            );
            if !is_valid_solar(y, m, d) || input.leap {
                return Err(ie(
                    "InvalidSolarDate",
                    format!("Ngày {d:02}/{m:02}/{y} không tồn tại"),
                ));
            }
            b.solar = Some(Ymd::new(y, m, d));
            if (MIN_YEAR..=MAX_YEAR).contains(&y) {
                let l = lvhv_core::solar_to_lunar_with(cal.method(), y, m, d)
                    .map_err(|_| out_of_range())?;
                b.lunar_year = Some(l.year);
                b.lunar = Some(LunarDay {
                    year: Some(l.year),
                    month: l.month,
                    day: l.day,
                    leap: l.leap,
                });
            }
        }
        (Calendar::Lunar, Precision::Day) => {
            let (y, m, d) = (
                want(input.year, "năm")?,
                want(input.month, "tháng")?,
                want(input.day, "ngày")?,
            );
            if !(1..=12).contains(&m) || !(1..=30).contains(&d) {
                return Err(ie(
                    "InvalidLunarDate",
                    format!("Ngày âm {d}/{m} không hợp lệ"),
                ));
            }
            if (MIN_LUNAR_YEAR..=MAX_LUNAR_YEAR).contains(&y) {
                check_lunar_exists(cal, y, m, d, input.leap)?;
                b.solar = cal
                    .l2s(y, m, d, input.leap)
                    .map(|s| Ymd::new(s.year, s.month, s.day));
            }
            b.lunar_year = Some(y);
            b.lunar = Some(LunarDay {
                year: Some(y),
                month: m,
                day: d,
                leap: input.leap,
            });
        }
        (Calendar::Lunar, Precision::Year) => {
            b.lunar_year = Some(want(input.year, "năm")?);
        }
        (Calendar::Solar, Precision::Year) => {
            b.lunar_year = Some(want(input.year, "năm")?);
            b.lunar_year_approx = true;
        }
        _ => {
            return Err(ie(
                "PrecisionNotAllowed",
                "Ngày sinh cần đủ ngày hoặc ít nhất năm".into(),
            ))
        }
    }
    b.can_chi = b.lunar_year.map(can_chi_year);
    Ok(b)
}

/// `tuoi_mat` (02 §4.2). `value < 1` hoặc `> 150` (lược đồ 01 §3.2) → lỗi nhập liệu.
pub fn tuoi_mat(
    birth: Option<&BirthInfo>,
    death_lunar_year: Option<i32>,
    nguong: i32,
) -> Result<Tuoi, InputError> {
    let by = birth.and_then(|b| b.lunar_year);
    let value = match (by, death_lunar_year) {
        (Some(b), Some(d)) => Some(i64::from(d) - i64::from(b) + 1),
        _ => None,
    };
    if let Some(v) = value {
        if v < 1 {
            return Err(ie("DeathBeforeBirth", "Ngày mất trước ngày sinh".into()));
        }
        if v > 150 {
            return Err(ie("AgeTooLarge", format!("Tuổi {v} vượt quá 150")));
        }
    }
    let value = value.map(|v| v as i32);
    Ok(Tuoi {
        value,
        label: if value.is_some_and(|v| v < nguong) {
            AgeLabel::HuongDuong
        } else {
            AgeLabel::HuongTho
        },
        manual: false,
        approx: birth.is_some_and(|b| b.lunar_year_approx),
    })
}

/// "Hưởng thọ 86 tuổi" / "Hưởng dương 45 tuổi" (+ " (khoảng)").
pub fn tuoi_text(t: &Tuoi) -> Option<String> {
    t.value.map(|v| {
        format!(
            "{} {v} tuổi{}",
            t.label.text(),
            if t.approx { " (khoảng)" } else { "" }
        )
    })
}

/// `solar_occurrence` (02 §6): lần xuất hiện năm `y` của ngày dương gốc; `None` nếu `y < y0`.
pub fn solar_occurrence(orig: Ymd, y: i32, rule: Feb29Rule) -> Option<Ymd> {
    if y < orig.y {
        return None;
    }
    if (orig.m, orig.d) == (2, 29) && !is_leap_year(y) {
        return Some(match rule {
            Feb29Rule::Feb28 => Ymd::new(y, 2, 28),
            Feb29Rule::Mar1 => Ymd::new(y, 3, 1),
        });
    }
    Some(Ymd::new(y, orig.m, orig.d))
}

/// Tuổi dương của người đang sống tại `today` (02 §4.3): số lần sinh nhật dương (đã quy đổi 29/02)
/// đã qua. `None` nếu `today` trước ngày sinh.
pub fn tuoi_duong(birth: Ymd, today: Ymd, rule: Feb29Rule) -> Option<i32> {
    if today < birth {
        return None;
    }
    let occ = solar_occurrence(birth, today.y, rule)?;
    Some(today.y - birth.y - i32::from(today < occ))
}

/// Tuổi âm tại `today` (02 §4.3): `năm âm hôm nay − lunar_year + 1`.
pub fn tuoi_am(cal: &Cal, lunar_year: i32, today: Ymd) -> Option<i32> {
    if !(MIN_YEAR..=MAX_YEAR).contains(&today.y) {
        return None;
    }
    let ly = lvhv_core::solar_to_lunar_with(cal.method(), today.y, today.m, today.d)
        .ok()?
        .year;
    Some(ly - lunar_year + 1)
}

/// Phương pháp mặc định (để tiện gọi).
pub fn default_cal() -> &'static Cal {
    Cal::get(DEFAULT_METHOD)
}

/// Phương pháp từ tên (`"precise"`/`"hnd"`).
pub fn method_name(m: Method) -> &'static str {
    match m {
        Method::Precise => "precise",
        Method::Hnd => "hnd",
    }
}
