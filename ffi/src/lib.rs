//! # lvhv-ffi — lớp FFI của "Lịch và Văn hóa Việt" (Clevix Lab)
//!
//! Xuất lõi lịch `lvhv-core` cho Kotlin (Android) và Swift (iOS/macOS) qua UniFFI (macro thủ tục,
//! không có tệp UDL). Ứng dụng Windows (Rust thuần) gọi trực tiếp các hàm Rust ở đây.
//!
//! Mã viết tay trong crate này không dùng `unsafe`. Các macro của UniFFI tự sinh mã `extern "C"`
//! (có `unsafe` bên trong) — đó là mã của thư viện UniFFI, không thuộc tiêu chí S1 (S1 áp dụng cho
//! `lvhv-core`, vốn `#![forbid(unsafe_code)]`).
//!
//! Hàm trả JSON có khóa tiếng Anh, giá trị tiếng Việt, đúng định dạng PWA dùng
//! (`pwa/src/types.ts`), để mọi nền tảng hiển thị cùng một dữ liệu.
//!
//! Mọi kiểm tra miền (2000–3000, ngày hợp lệ, khoảng ≤ 366 ngày, mã việc) do lõi làm và trả
//! `lvhv_core::DateError`; ở đây chỉ đổi sang `LvhvError` với thông điệp tiếng Việt cho người dùng.

uniffi::setup_scaffolding!();

mod cadao;
mod sources;

use lvhv_core::jd::{days_in_month as core_days_in_month, jdn_from_date};
use lvhv_core::{DateError, SolarDate};

pub use lvhv_core::{BIRTH_MIN_YEAR, MAX_SPAN_DAYS, MAX_YEAR, MIN_YEAR};

/// Lỗi trả về cho ứng dụng. Thông điệp bằng tiếng Việt, hiển thị được cho người dùng.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Error)]
#[uniffi(flat_error)]
pub enum LvhvError {
    /// Ngày dương không tồn tại (ví dụ 30/2).
    InvalidDate(String),
    /// Ngoài phạm vi 2000–3000.
    OutOfRange(String),
    /// Mã việc không có trong danh sách `activities_json()`.
    UnknownActivity(String),
    /// Khoảng ngày sai (kết thúc trước bắt đầu, hoặc dài quá một năm).
    BadRange(String),
    /// Chức năng chưa có trong bản dựng này.
    Unavailable(String),
}

impl std::fmt::Display for LvhvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LvhvError::InvalidDate(s)
            | LvhvError::OutOfRange(s)
            | LvhvError::UnknownActivity(s)
            | LvhvError::BadRange(s)
            | LvhvError::Unavailable(s) => f.write_str(s),
        }
    }
}

impl std::error::Error for LvhvError {}

impl From<DateError> for LvhvError {
    fn from(e: DateError) -> Self {
        match e {
            DateError::InvalidDate { year, month, day } => {
                LvhvError::InvalidDate(format!("Ngày {day}/{month}/{year} không tồn tại."))
            }
            DateError::OutOfRange { min, max } => {
                LvhvError::OutOfRange(format!("Lịch hỗ trợ từ năm {min} đến năm {max}."))
            }
            DateError::InvertedRange => {
                LvhvError::BadRange("Ngày kết thúc cần sau ngày bắt đầu.".to_string())
            }
            DateError::RangeTooLong { .. } => {
                LvhvError::BadRange("Khoảng tìm tối đa là một năm. Hãy thu hẹp lại.".to_string())
            }
            DateError::UnknownActivity => {
                LvhvError::UnknownActivity("Không có việc này trong danh sách.".to_string())
            }
            DateError::Computation => LvhvError::Unavailable(format!("Không tính được: {e}.")),
        }
    }
}

/// Ngày dương lịch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, uniffi::Record)]
pub struct SolarDay {
    pub y: i32,
    pub m: i32,
    pub d: i32,
}

/// Ngày âm lịch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, uniffi::Record)]
pub struct LunarDay {
    pub y: i32,
    pub m: i32,
    pub d: i32,
    pub leap: bool,
    /// 29 (tháng thiếu) hoặc 30 (tháng đủ).
    pub month_len: i32,
}

/// Đổi (y, m, d) kiểu i32 của FFI sang kiểu của lõi; tháng/ngày âm → `InvalidDate`.
/// Kiểm tra miền 2000–3000 và ngày hợp lệ: `check_solar` (dùng cho các tiện ích ở lớp này,
/// cùng quy tắc với lõi); các hàm gọi lõi nhận lỗi trực tiếp từ lõi.
fn to_core(y: i32, m: i32, d: i32) -> Result<(i32, u32, u32), LvhvError> {
    if m < 1 || d < 1 {
        return Err(DateError::InvalidDate {
            year: y,
            month: m.max(0) as u32,
            day: d.max(0) as u32,
        }
        .into());
    }
    Ok((y, m as u32, d as u32))
}

/// Kiểm tra ngày dương hợp lệ (trước) rồi trong miền 2000–3000 (sau) — cùng thứ tự với lõi.
fn check_solar(y: i32, m: i32, d: i32) -> Result<(i32, u32, u32), LvhvError> {
    let (y, m, d) = to_core(y, m, d)?;
    if !lvhv_core::jd::is_valid_solar(y, m, d) {
        return Err(DateError::InvalidDate {
            year: y,
            month: m,
            day: d,
        }
        .into());
    }
    if !(MIN_YEAR..=MAX_YEAR).contains(&y) {
        return Err(DateError::OUT_OF_RANGE.into());
    }
    Ok((y, m, d))
}

/// JDN của ngày dương đã kiểm tra.
fn jdn_of(day: SolarDay) -> Result<i64, LvhvError> {
    let (y, m, d) = check_solar(day.y, day.m, day.d)?;
    Ok(jdn_from_date(y, m, d))
}

fn solar_date(day: SolarDay) -> Result<SolarDate, LvhvError> {
    let (year, month, day) = to_core(day.y, day.m, day.d)?;
    Ok(SolarDate { year, month, day })
}

/// Thông tin đầy đủ của một ngày, dạng JSON theo hợp đồng kiến trúc §4 (thứ tự khóa cố định),
/// đúng từng byte như `lvhv_core::DayInfo::to_json` (kể cả phần `ngoc_hap`).
#[uniffi::export]
pub fn day_info_json(y: i32, m: i32, d: i32) -> Result<String, LvhvError> {
    let (y, m, d) = to_core(y, m, d)?;
    Ok(lvhv_core::day_info(y, m, d)?.to_json())
}

/// Đổi dương → âm.
#[uniffi::export]
pub fn solar_to_lunar(y: i32, m: i32, d: i32) -> Result<LunarDay, LvhvError> {
    let (y, m, d) = to_core(y, m, d)?;
    let l = lvhv_core::solar_to_lunar(y, m, d)?;
    Ok(LunarDay {
        y: l.year,
        m: l.month as i32,
        d: l.day as i32,
        leap: l.leap,
        month_len: l.month_len as i32,
    })
}

/// Đổi âm → dương. `None` nếu ngày âm không tồn tại (tháng nhuận không có, ngày 30 của tháng
/// thiếu…) hoặc kết quả ra ngoài 2000-01-01 … 3000-12-31 (lõi kiểm).
#[uniffi::export]
pub fn lunar_to_solar(ly: i32, lm: i32, ld: i32, leap: bool) -> Option<SolarDay> {
    if lm < 1 || ld < 1 {
        return None;
    }
    let s = lvhv_core::lunar_to_solar(ly, lm as u32, ld as u32, leap)?;
    Some(SolarDay {
        y: s.year,
        m: s.month as i32,
        d: s.day as i32,
    })
}

/// Âm lịch của cả một tháng dương (phần tử i là ngày i+1) — dùng cho bảng tháng, đỡ gọi FFI
/// từng ngày.
#[uniffi::export]
pub fn month_lunar_days(y: i32, m: i32) -> Result<Vec<LunarDay>, LvhvError> {
    check_solar(y, m, 1)?;
    let n = core_days_in_month(y, m as u32) as i32;
    (1..=n).map(|d| solar_to_lunar(y, m, d)).collect()
}

/// Số ngày của tháng dương.
#[uniffi::export]
pub fn days_in_month(y: i32, m: i32) -> Result<i32, LvhvError> {
    check_solar(y, m, 1)?;
    Ok(core_days_in_month(y, m as u32) as i32)
}

/// Thứ trong tuần: 0 = Chủ nhật, 1 = Thứ Hai, …, 6 = Thứ Bảy.
#[uniffi::export]
pub fn weekday(y: i32, m: i32, d: i32) -> Result<i32, LvhvError> {
    let (y, m, d) = check_solar(y, m, d)?;
    Ok(lvhv_core::jd::weekday(jdn_from_date(y, m, d)) as i32)
}

/// Cộng `n` ngày vào một ngày dương (kết quả phải còn trong 2000–3000).
#[uniffi::export]
pub fn add_days(day: SolarDay, n: i32) -> Result<SolarDay, LvhvError> {
    let j = jdn_of(day)? + n as i64;
    let (y, m, d) = lvhv_core::jd::date_from_jdn(j);
    check_solar(y, m as i32, d as i32)?;
    Ok(SolarDay {
        y,
        m: m as i32,
        d: d as i32,
    })
}

/// Năm âm lịch của một ngày sinh dương lịch (1900-01-01 … 3000-12-31): người sinh trước Tết
/// thuộc năm âm trước. Dùng kết quả làm `birth_lunar_year` của `find_good_days_json` và để hiển thị
/// tuổi Can Chi (`year_can_chi`). Phần 1900–1999 chưa kiểm định độc lập (xem core/src/error.rs).
#[uniffi::export]
pub fn lunar_year_of_birth(birth: SolarDay) -> Result<i32, LvhvError> {
    let s = solar_date(birth)?;
    Ok(lvhv_core::lunar_year_of_birth(s.year, s.month, s.day)?)
}

/// Can Chi của một năm ÂM lịch (năm tuổi), ví dụ 1976 → "Bính Thìn". Với ngày sinh dương lịch,
/// đổi trước bằng `lunar_year_of_birth`.
#[uniffi::export]
pub fn year_can_chi(y: i32) -> String {
    let can = (y + 6).rem_euclid(10) as usize;
    let chi = (y + 8).rem_euclid(12) as usize;
    format!(
        "{} {}",
        lvhv_core::canchi::CAN[can],
        lvhv_core::canchi::CHI[chi]
    )
}

/// Nhãn "Đánh giá" của một ngày theo điểm chung Ngọc Hạp.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct DayRating {
    /// "Tốt", "Khá tốt", "Bình thường", "Xấu".
    pub label: String,
    /// 3 = Tốt, 2 = Khá tốt, 1 = Bình thường, 0 = Xấu (để tô màu).
    pub level: u8,
}

/// Ngưỡng nhãn đánh giá — GIỐNG `rating()` của PWA (`pwa/src/format.ts`) để mọi nền tảng hiển thị
/// cùng nhãn: ≥ 4 Tốt, 1..3 Khá tốt, −3..0 Bình thường, ≤ −4 Xấu. Phân bố 2000–2100
/// (`cargo run -p lvhv-ffi --release --example score_hist`): Tốt ≈ 17 %, Khá tốt ≈ 21 %,
/// Bình thường ≈ 37 %, Xấu ≈ 26 %. Ngưỡng là quy ước của ứng dụng (chưa được A3 khóa).
pub const RATING_GOOD: i32 = 4;
/// Ngưỡng "Khá tốt".
pub const RATING_FAIR: i32 = 1;
/// Ngưỡng "Bình thường".
pub const RATING_AVG: i32 = -3;

/// Nhãn đánh giá cho điểm chung `score` (trường `ngoc_hap.score` của `day_info_json`).
#[uniffi::export]
pub fn day_rating(score: i32) -> DayRating {
    let (label, level) = if score >= RATING_GOOD {
        ("Tốt", 3)
    } else if score >= RATING_FAIR {
        ("Khá tốt", 2)
    } else if score >= RATING_AVG {
        ("Bình thường", 1)
    } else {
        ("Xấu", 0)
    };
    DayRating {
        label: label.to_string(),
        level,
    }
}

/// Danh sách việc để chọn ngày, JSON `[{"id":"cuoi_hoi","name":"Cưới hỏi"}, …]`.
#[uniffi::export]
pub fn activities_json() -> String {
    let mut o = String::from("[");
    for (i, a) in lvhv_core::activities().iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        o.push_str("{\"id\":");
        lvhv_core::dayinfo::write_json_str(&mut o, a.id);
        o.push_str(",\"name\":");
        lvhv_core::dayinfo::write_json_str(&mut o, a.name);
        o.push('}');
    }
    o.push(']');
    o
}

/// Chọn ngày tốt cho việc `activity` trong khoảng [`from`, `to`] (`to − from ≤ 366` ngày) —
/// `lvhv_core::find_good_days`. `birth_lunar_year`: năm ÂM lịch của ngày sinh (xem
/// `lunar_year_of_birth`). JSON là mảng `GoodDay::to_json` của lõi:
/// `[{"solar":{y,m,d},"lunar":{y,m,d,leap,month_len},"weekday","jdn","score","allowed","reasons":[..],"first_gio_hoang_dao":{chi,from,to}}, …]`,
/// xếp điểm giảm dần, cùng điểm thì ngày sớm trước; chỉ gồm ngày không phạm sao kỵ chính (N3).
#[uniffi::export]
pub fn find_good_days_json(
    activity: String,
    from: SolarDay,
    to: SolarDay,
    birth_lunar_year: Option<i32>,
) -> Result<String, LvhvError> {
    let list = lvhv_core::find_good_days(
        &activity,
        solar_date(from)?,
        solar_date(to)?,
        birth_lunar_year,
    )?;
    let mut o = String::with_capacity(64 + list.len() * 320);
    o.push('[');
    for (i, g) in list.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        o.push_str(&g.to_json());
    }
    o.push(']');
    Ok(o)
}

/// Câu ca dao/tục ngữ/thành ngữ của ngày, JSON
/// `{"id":..,"type":..,"text":..,"lines":[..],"theme":..,"order":..,"sources":[{"title","url","license"}]}`.
/// Quy tắc chọn: `content/cadao/select.md` (pos = (JDN − 2451545) mod N, câu có `order == pos`).
#[uniffi::export]
pub fn cadao_for(y: i32, m: i32, d: i32) -> Result<String, LvhvError> {
    let (y, m, d) = check_solar(y, m, d)?;
    Ok(cadao::for_jdn(jdn_from_date(y, m, d)))
}

/// Số câu trong bộ ca dao nhúng.
#[uniffi::export]
pub fn cadao_count() -> u32 {
    cadao::count() as u32
}

/// Lời ghi công bắt buộc cho bộ ca dao (content/cadao/SOURCES.md, mục C.1), hiển thị ở màn hình
/// Giới thiệu.
#[uniffi::export]
pub fn cadao_attribution() -> String {
    cadao::attribution()
}

/// Toàn văn content/cadao/SOURCES.md (Markdown), cho màn hình "Nguồn dữ liệu".
#[uniffi::export]
pub fn cadao_sources_markdown() -> String {
    cadao::SOURCES_MD.to_string()
}

/// Danh mục nguồn ca dao (content/cadao/SOURCES.md) dạng khối
/// `[{"kind":"h"|"p"|"li","text":..}]`.
#[uniffi::export]
pub fn cadao_sources_blocks_json() -> String {
    sources::blocks_json(cadao::SOURCES_MD)
}

/// Danh mục nguồn quy tắc Ngọc Hạp (content/ngochap/SOURCES.md) dạng khối như trên.
#[uniffi::export]
pub fn ngochap_sources_blocks_json() -> String {
    sources::blocks_json(sources::NGOCHAP_SOURCES_MD)
}

/// Phương pháp âm lịch mặc định của lõi: "hnd" hoặc "precise" (docs/03 §15).
#[uniffi::export]
pub fn lunar_method() -> String {
    match lvhv_core::DEFAULT_METHOD {
        lvhv_core::Method::Hnd => "hnd".to_string(),
        lvhv_core::Method::Precise => "precise".to_string(),
    }
}

/// Thông tin phiên bản, JSON
/// `{"ffi":"0.1.0","method":"hnd","cadao_count":427,"activities":16,"min_year":2000,"max_year":3000}`.
#[uniffi::export]
pub fn build_info_json() -> String {
    format!(
        "{{\"ffi\":\"{}\",\"method\":\"{}\",\"cadao_count\":{},\"activities\":{},\"min_year\":{},\"max_year\":{}}}",
        env!("CARGO_PKG_VERSION"),
        lunar_method(),
        cadao::count(),
        lvhv_core::activities().len(),
        MIN_YEAR,
        MAX_YEAR
    )
}

#[cfg(test)]
mod tests;
