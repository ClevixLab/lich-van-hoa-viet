//! # lvhv-core — Lõi lịch âm Việt Nam (Lịch và Văn hóa Việt)
//!
//! Không phụ thuộc thư viện ngoài (chỉ crate nội bộ `lvhv-ngochap`), không dùng `unsafe`.
//! Múi giờ UTC+7 cố định. Đặc tả thuật toán: `docs/03-DAC-TA-THUAT-TOAN.md`.
//!
//! **Miền hỗ trợ** (xem `error`): ngày dương 2000-01-01 … 3000-12-31. Mọi hàm công khai trả
//! `Result<_, DateError>` hoặc `Option` cho đầu vào không hợp lệ hoặc ngoài miền — không treo,
//! không panic, không trả kết quả vô nghĩa (RS-01/RS-02/RS-03). Riêng `lunar_year_of_birth`
//! nhận ngày sinh từ 1900.
#![forbid(unsafe_code)]

pub mod astro;
pub mod canchi;
pub mod dayinfo;
pub mod error;
pub mod gooddays;
pub mod jd;
pub mod lunar;
pub mod terms;

pub use dayinfo::{
    day_ctx, day_ctx_jdn, day_ctx_jdn_with, day_ctx_with, day_info, day_info_with, DayCtx, DayInfo,
    GioHoangDao, NgocHap,
};
pub use error::{
    DateError, BIRTH_MIN_YEAR, MAX_JDN, MAX_LUNAR_YEAR, MAX_SPAN_DAYS, MAX_YEAR, MIN_JDN,
    MIN_LUNAR_YEAR, MIN_YEAR,
};
pub use gooddays::{
    activities, find_good_days, find_good_days_with, Activity, GoodDay, MIN_GOOD_DAY_SCORE,
};
pub use lunar::{
    jdn_to_lunar, jdn_to_lunar_with, lunar_to_jdn, lunar_to_jdn_with, lunar_to_solar,
    lunar_to_solar_with, lunar_year_of_birth, lunar_year_of_birth_with, solar_to_lunar,
    solar_to_lunar_with, LunarDate, Method, SolarDate, DEFAULT_METHOD,
};
pub use terms::{solar_terms_of_year, SolarTerm, TERM_NAMES};

/// Ngày Tết Nguyên đán (mùng 1 tháng 1 âm, không nhuận) của năm âm `ly`.
/// `Err(OutOfRange)` nếu `ly` ngoài 2000–3000 (Tết của năm âm 2000…3000 đều nằm trong miền).
pub fn tet(ly: i32) -> Result<SolarDate, DateError> {
    tet_with(DEFAULT_METHOD, ly)
}

/// Ngày Tết theo phương pháp chỉ định.
pub fn tet_with(method: Method, ly: i32) -> Result<SolarDate, DateError> {
    if !(MIN_YEAR..=MAX_YEAR).contains(&ly) {
        return Err(DateError::OUT_OF_RANGE);
    }
    // Trong miền, tháng Giêng luôn tồn tại; nếu không thì là lỗi của lõi, không phải của người gọi.
    lunar_to_solar_with(method, ly, 1, 1, false).ok_or(DateError::Computation)
}
