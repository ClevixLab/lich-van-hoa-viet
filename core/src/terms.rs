//! 24 tiết khí.
//!
//! Chỉ số tiết khí `i` (0..23) ứng với kinh độ biểu kiến của Mặt Trời `15·i` độ,
//! bắt đầu từ Xuân Phân (0°).

pub use crate::astro::local_midnight_ut;
use crate::astro::{apparent_sun_longitude_ut, local_day_of, solve_sun_longitude};
use crate::error::{DateError, MAX_YEAR, MIN_YEAR};
use crate::jd::jdn_from_date;
use crate::lunar::Method;

/// Tên 24 tiết khí, chỉ số i ↔ kinh độ 15·i°.
pub const TERM_NAMES: [&str; 24] = [
    "Xuân Phân",
    "Thanh Minh",
    "Cốc Vũ",
    "Lập Hạ",
    "Tiểu Mãn",
    "Mang Chủng",
    "Hạ Chí",
    "Tiểu Thử",
    "Đại Thử",
    "Lập Thu",
    "Xử Thử",
    "Bạch Lộ",
    "Thu Phân",
    "Hàn Lộ",
    "Sương Giáng",
    "Lập Đông",
    "Tiểu Tuyết",
    "Đại Tuyết",
    "Đông Chí",
    "Tiểu Hàn",
    "Đại Hàn",
    "Lập Xuân",
    "Vũ Thủy",
    "Kinh Trập",
];

/// Chỉ số tiết khí (0..23) của ngày `jdn`: tiết khí mới nhất có thời điểm bắt đầu
/// trước 00:00 (giờ lịch: UTC+7, trước 1968 là UTC+8) của ngày hôm sau, tức là rơi vào ngày này hoặc trước đó.
pub fn term_index_of_day(jdn: i64) -> usize {
    let l = apparent_sun_longitude_ut(local_midnight_ut(jdn + 1));
    // `% 24` (không `.min(23)`): kinh độ làm tròn thành đúng 360.0 thuộc tiết 0 (C-7).
    ((l / 15.0).floor() as usize) % 24
}

/// Chỉ số tiết khí của ngày `jdn` theo phương pháp. `Hnd` và `Precise` dùng chung mô hình §6
/// (`term_index_of_day`); `Jpl` tra bảng DE441: tiết khí muộn nhất có JD_UT ≤ 00:00 đầu ngày hôm sau
/// (docs/03 §17.6.4). `Err(OutOfRange)` nếu ngoài bảng.
pub fn term_index_of_day_with(method: Method, jdn: i64) -> Result<usize, DateError> {
    match method {
        Method::Hnd | Method::Precise => Ok(term_index_of_day(jdn)),
        Method::Jpl => {
            crate::jpl::term_at(local_midnight_ut(jdn + 1)).ok_or(DateError::OUT_OF_RANGE)
        }
    }
}

/// Chi (0 = Tý … 11 = Hợi) của tháng tiết khí chứa ngày `jdn`.
/// Ranh giới tháng là 12 "Tiết" (kinh độ 15° + 30°·n); Lập Xuân (315°) mở đầu tháng Dần.
pub fn tiet_month_chi_of_term(term_idx: usize) -> u32 {
    // term_idx 21 (Lập Xuân, 315°) và 22 (Vũ Thủy) → Dần (2); mỗi 2 tiết khí sang một chi.
    let m = (term_idx + 24 - 21) % 24 / 2; // 0 = Dần
    ((m + 2) % 12) as u32
}

/// Một tiết khí đã tính thời điểm.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolarTerm {
    /// Chỉ số 0..23 (Xuân Phân = 0).
    pub index: usize,
    /// Thời điểm bắt đầu (JD, UT).
    pub jd_ut: f64,
}

impl SolarTerm {
    pub fn name(&self) -> &'static str {
        TERM_NAMES[self.index]
    }
    /// JDN của ngày (giờ lịch, `astro::local_day_of`) chứa thời điểm bắt đầu.
    pub fn local_jdn(&self) -> i64 {
        local_day_of(self.jd_ut)
    }
}

/// 24 tiết khí có thời điểm bắt đầu (theo UTC+7) trong năm dương `year`, theo thứ tự thời gian.
/// `Err(OutOfRange)` nếu `year` ngoài 1900–3000 (trước đây `year + 1` tràn với `i32::MAX`, RS-04);
/// `Err(Computation)` nếu bộ giải không ra đủ 24 tiết khí hữu hạn.
pub fn solar_terms_of_year(year: i32) -> Result<Vec<SolarTerm>, DateError> {
    solar_terms_of_year_with(Method::Precise, year)
}

/// Như `solar_terms_of_year` theo phương pháp: `Hnd`/`Precise` cùng mô hình §6; `Jpl` từ bảng DE441
/// (§17.6.4: ngày bắt đầu = `local_day_of(JD_UT)`).
pub fn solar_terms_of_year_with(method: Method, year: i32) -> Result<Vec<SolarTerm>, DateError> {
    if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
        return Err(DateError::OUT_OF_RANGE);
    }
    if method == Method::Jpl {
        let start = jdn_from_date(year, 1, 1);
        let end = jdn_from_date(year + 1, 1, 1);
        // Mở rộng 2 ngày mỗi phía rồi lọc theo ngày lịch (múi giờ ≤ 8 giờ).
        let v = crate::jpl::terms_between(start as f64 - 2.5, end as f64 + 1.5)
            .ok_or(DateError::OUT_OF_RANGE)?;
        let out: Vec<SolarTerm> = v
            .into_iter()
            .map(|(index, jd_ut)| SolarTerm { index, jd_ut })
            .filter(|t| (start..end).contains(&t.local_jdn()))
            .collect();
        if out.len() != 24 {
            return Err(DateError::Computation);
        }
        return Ok(out);
    }
    let start = jdn_from_date(year, 1, 1);
    // year ≤ 3000 nên year + 1 không tràn.
    let end = jdn_from_date(year + 1, 1, 1);
    let mut out = Vec::with_capacity(24);
    // Tiểu Hàn (285°) rơi vào khoảng 5–6/1; bắt đầu dò từ Đông Chí năm trước.
    let mut guess = local_midnight_ut(start) - 15.0;
    let mut idx = 18usize; // Đông Chí
    for _ in 0..27 {
        let t = solve_sun_longitude(15.0 * idx as f64, guess);
        let term = SolarTerm {
            index: idx,
            jd_ut: t,
        };
        let j = term.local_jdn();
        if j >= start && j < end {
            out.push(term);
        }
        idx = (idx + 1) % 24;
        guess = t + 15.2;
    }
    if out.len() != 24 || out.iter().any(|t| !t.jd_ut.is_finite()) {
        return Err(DateError::Computation);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiet_month_chi() {
        assert_eq!(tiet_month_chi_of_term(21), 2); // Lập Xuân → Dần
        assert_eq!(tiet_month_chi_of_term(23), 3); // Kinh Trập → Mão
        assert_eq!(tiet_month_chi_of_term(1), 4); // Thanh Minh → Thìn
        assert_eq!(tiet_month_chi_of_term(13), 10); // Hàn Lộ → Tuất
        assert_eq!(tiet_month_chi_of_term(14), 10); // Sương Giáng → Tuất
        assert_eq!(tiet_month_chi_of_term(17), 0); // Đại Tuyết → Tý
        assert_eq!(tiet_month_chi_of_term(19), 1); // Tiểu Hàn → Sửu
        assert_eq!(tiet_month_chi_of_term(20), 1); // Đại Hàn → Sửu
    }

    #[test]
    fn terms_out_of_range() {
        for y in [i32::MIN, -1, 0, 1899, 3001, i32::MAX] {
            assert_eq!(solar_terms_of_year(y), Err(DateError::OUT_OF_RANGE), "{y}");
        }
    }

    #[test]
    fn terms_consistent_with_day_rule() {
        for y in [1900, 1967, 1968, 2000, 2026, 2100, 2500, 3000] {
            let ts = solar_terms_of_year(y).unwrap();
            assert_eq!(ts.len(), 24, "năm {y}");
            for t in ts {
                let j = t.local_jdn();
                assert_eq!(term_index_of_day(j), t.index);
                assert_ne!(term_index_of_day(j - 1), t.index);
            }
        }
    }
}
