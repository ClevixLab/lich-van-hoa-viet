//! Miền hỗ trợ (phạm vi ngày) và kiểu lỗi của API công khai.
//!
//! **Miền công khai** (hợp đồng §2, tiêu chí L2–L7): ngày dương 2000-01-01 … 3000-12-31
//! (JDN `MIN_JDN..=MAX_JDN`). Mọi hàm công khai nhận ngày dương / JDN / năm âm sẽ trả lỗi
//! (`Err(DateError::OutOfRange{..})` hoặc `None`) thay vì treo, panic hay trả kết quả vô nghĩa
//! khi đầu vào nằm ngoài miền này.
//!
//! **Miền nội bộ**: bảng tháng (`YearTable`) được lập cho "tuế" `yy ∈ 1900..=3101`, đủ để các
//! năm âm ở hai đầu miền công khai (ví dụ tháng 11–12 âm của năm 1999, tháng 11–12 âm của năm
//! 3000 kéo sang tháng 1/3001) và để đổi **ngày sinh** từ 1900 (xem `lunar_year_of_birth`).
//!
//! **Ngày sinh**: `lunar_year_of_birth` nhận ngày dương 1900-01-01 … 3000-12-31. Phần 1900–1999
//! **chưa** được kiểm định độc lập (oracle chỉ chạy 2000–3000); sai số có thể có chỉ ảnh hưởng
//! những ngày sinh sát Tết (năm âm có thể lệch 1 nếu ngày Sóc tháng Giêng tính lệch).

use std::fmt;

/// Năm dương nhỏ nhất của miền công khai.
pub const MIN_YEAR: i32 = 2000;
/// Năm dương lớn nhất của miền công khai.
pub const MAX_YEAR: i32 = 3000;
/// JDN của 2000-01-01.
pub const MIN_JDN: i64 = 2451545;
/// JDN của 3000-12-31.
pub const MAX_JDN: i64 = 2817152;
/// Năm âm nhỏ nhất chấp nhận được ở `lunar_to_solar`: tháng 11–12 âm năm 1999 rơi vào
/// tháng 1–2/2000 (kết quả vẫn phải nằm trong miền công khai, nếu không trả `None`).
pub const MIN_LUNAR_YEAR: i32 = MIN_YEAR - 1;
/// Năm âm lớn nhất chấp nhận được ở `lunar_to_solar`.
pub const MAX_LUNAR_YEAR: i32 = MAX_YEAR;
/// Năm dương nhỏ nhất của ngày sinh (`lunar_year_of_birth`).
pub const BIRTH_MIN_YEAR: i32 = 1900;
/// Khoảng tối đa của `find_good_days`: `to − from ≤ MAX_SPAN_DAYS` (tức tối đa 367 ngày tính
/// cả hai đầu) — cùng quy ước với PWA và lớp `ffi`.
pub const MAX_SPAN_DAYS: i64 = 366;

/// Tuế nhỏ nhất/lớn nhất mà `YearTable::new` chấp nhận (miền nội bộ).
pub(crate) const INTERNAL_MIN_YY: i32 = 1900;
pub(crate) const INTERNAL_MAX_YY: i32 = 3101;

/// Lỗi của API công khai (thông điệp tiếng Việt qua `Display`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DateError {
    /// Ngày dương không tồn tại (tháng 13, 30/2, 29/2 năm không nhuận…).
    InvalidDate { year: i32, month: u32, day: u32 },
    /// Ngoài miền hỗ trợ (năm `min..=max`).
    OutOfRange { min: i32, max: i32 },
    /// `find_good_days`: ngày kết thúc trước ngày bắt đầu.
    InvertedRange,
    /// `find_good_days`: `to − from` vượt `MAX_SPAN_DAYS`.
    RangeTooLong { span_days: i64 },
    /// `find_good_days`: mã việc không có trong `activities()`.
    UnknownActivity,
    /// Bất biến lịch bị vi phạm khi tính (không xảy ra trong miền đã kiểm định; nếu gặp là lỗi
    /// của lõi — báo lại).
    Computation,
}

impl DateError {
    /// Lỗi ngoài miền công khai 2000–3000.
    pub const OUT_OF_RANGE: DateError = DateError::OutOfRange {
        min: MIN_YEAR,
        max: MAX_YEAR,
    };

    /// Mã ổn định (ASCII), dùng chung với tsport (`LvhvError.code`).
    pub fn code(&self) -> &'static str {
        match self {
            DateError::InvalidDate { .. } => "InvalidDate",
            DateError::OutOfRange { .. } => "OutOfRange",
            DateError::InvertedRange => "InvertedRange",
            DateError::RangeTooLong { .. } => "RangeTooLong",
            DateError::UnknownActivity => "UnknownActivity",
            DateError::Computation => "Computation",
        }
    }
}

impl fmt::Display for DateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DateError::InvalidDate { year, month, day } => {
                write!(f, "ngày dương không hợp lệ: {year}-{month}-{day}")
            }
            DateError::OutOfRange { min, max } => {
                write!(f, "ngoài phạm vi hỗ trợ (năm {min} đến năm {max})")
            }
            DateError::InvertedRange => f.write_str("ngày kết thúc trước ngày bắt đầu"),
            DateError::RangeTooLong { span_days } => write!(
                f,
                "khoảng ngày dài {span_days} ngày, tối đa {MAX_SPAN_DAYS} ngày"
            ),
            DateError::UnknownActivity => f.write_str("mã việc không tồn tại"),
            DateError::Computation => f.write_str("lỗi tính toán nội bộ của lõi lịch"),
        }
    }
}

impl std::error::Error for DateError {}

/// JDN có thuộc miền công khai không.
pub fn jdn_in_range(jdn: i64) -> bool {
    (MIN_JDN..=MAX_JDN).contains(&jdn)
}

/// `Ok(())` nếu JDN thuộc miền công khai.
pub(crate) fn check_jdn(jdn: i64) -> Result<(), DateError> {
    if jdn_in_range(jdn) {
        Ok(())
    } else {
        Err(DateError::OUT_OF_RANGE)
    }
}

/// Kiểm ngày dương hợp lệ (trước) rồi thuộc miền `min_year..=MAX_YEAR` (sau); trả JDN.
pub(crate) fn checked_jdn(y: i32, m: u32, d: u32, min_year: i32) -> Result<i64, DateError> {
    if !crate::jd::is_valid_solar(y, m, d) {
        return Err(DateError::InvalidDate {
            year: y,
            month: m,
            day: d,
        });
    }
    if !(min_year..=MAX_YEAR).contains(&y) {
        return Err(DateError::OutOfRange {
            min: min_year,
            max: MAX_YEAR,
        });
    }
    Ok(crate::jd::jdn_from_date(y, m, d))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jd::jdn_from_date;

    #[test]
    fn bounds_match_dates() {
        assert_eq!(jdn_from_date(MIN_YEAR, 1, 1), MIN_JDN);
        assert_eq!(jdn_from_date(MAX_YEAR, 12, 31), MAX_JDN);
    }
}
