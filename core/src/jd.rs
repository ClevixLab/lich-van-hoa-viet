//! Số ngày Julian (JDN) và chuyển đổi lịch Gregory (đón trước — proleptic).

/// Năm nhuận Gregory.
pub fn is_leap_year(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Số ngày của tháng dương lịch.
pub fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(y) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Ngày dương hợp lệ?
pub fn is_valid_solar(y: i32, m: u32, d: u32) -> bool {
    (1..=12).contains(&m) && d >= 1 && d <= days_in_month(y, m)
}

/// JDN (số nguyên, ngày bắt đầu lúc trưa theo quy ước thiên văn) của ngày Gregory.
/// Tính trên `i64` nên không tràn với mọi `i32`/`u32` đầu vào (không kiểm ngày hợp lệ — dùng
/// `is_valid_solar` trước nếu cần).
pub fn jdn_from_date(y: i32, m: u32, d: u32) -> i64 {
    let y = y as i64;
    let m = m as i64;
    let d = d as i64;
    let a = (14 - m).div_euclid(12);
    let yy = y + 4800 - a;
    let mm = m + 12 * a - 3;
    d + (153 * mm + 2).div_euclid(5) + 365 * yy + yy.div_euclid(4) - yy.div_euclid(100)
        + yy.div_euclid(400)
        - 32045
}

/// Ngày Gregory từ JDN. Đúng (khứ hồi) cho JDN trong khoảng năm 1990–3010 đã kiểm, và về
/// hình thức cho mọi năm vừa `i32`; năm ngoài `i32` bị cắt — chỉ gọi với JDN đã kiểm miền.
pub fn date_from_jdn(jdn: i64) -> (i32, u32, u32) {
    let a = jdn + 32044;
    let b = (4 * a + 3).div_euclid(146097);
    let c = a - (146097 * b).div_euclid(4);
    let d = (4 * c + 3).div_euclid(1461);
    let e = c - (1461 * d).div_euclid(4);
    let m = (5 * e + 2).div_euclid(153);
    let day = e - (153 * m + 2).div_euclid(5) + 1;
    let month = m + 3 - 12 * m.div_euclid(10);
    let year = b * 100 + d - 4800 + m.div_euclid(10);
    (year as i32, month as u32, day as u32)
}

/// Thứ trong tuần: 0 = Chủ nhật, 1 = Thứ Hai, …, 6 = Thứ Bảy.
pub fn weekday(jdn: i64) -> u32 {
    (jdn + 1).rem_euclid(7) as u32
}

/// Thời điểm (JD thực, UT) → (năm, tháng, ngày, giờ, phút, giây) theo múi giờ `tz_hours`.
/// Giây được làm tròn đến giây gần nhất. `None` nếu đầu vào không hữu hạn (NaN/∞, C-8) hoặc
/// ngày kết quả ngoài miền công khai 2000–3000.
pub fn jd_to_datetime(jd_ut: f64, tz_hours: f64) -> Option<(i32, u32, u32, u32, u32, u32)> {
    if !jd_ut.is_finite() || !tz_hours.is_finite() {
        return None;
    }
    let local = jd_ut + tz_hours / 24.0 + 0.5;
    let mut day = local.floor();
    let mut secs = ((local - day) * 86400.0).round() as i64;
    if secs >= 86400 {
        secs -= 86400;
        day += 1.0;
    }
    let day = day as i64;
    if !crate::error::jdn_in_range(day) {
        return None;
    }
    let (y, m, d) = date_from_jdn(day);
    // 0 ≤ secs < 86400 nên các phép đổi kiểu dưới đây không mất thông tin.
    Some((
        y,
        m,
        d,
        (secs / 3600) as u32,
        ((secs % 3600) / 60) as u32,
        (secs % 60) as u32,
    ))
}

/// Định dạng ISO `YYYY-MM-DD`.
pub fn fmt_date(y: i32, m: u32, d: u32) -> String {
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_jdn() {
        assert_eq!(jdn_from_date(2000, 1, 1), 2451545);
        assert_eq!(jdn_from_date(2026, 11, 5), 2461350);
        assert_eq!(date_from_jdn(2461350), (2026, 11, 5));
        assert_eq!(weekday(2461350), 4); // Thứ Năm
    }

    #[test]
    fn jd_to_datetime_rejects_non_finite() {
        assert_eq!(jd_to_datetime(f64::NAN, 7.0), None);
        assert_eq!(jd_to_datetime(f64::INFINITY, 7.0), None);
        assert_eq!(jd_to_datetime(1e12, 7.0), None);
        assert_eq!(jd_to_datetime(2451545.0, 0.0), Some((2000, 1, 1, 12, 0, 0)));
    }

    #[test]
    fn jdn_from_date_extremes_do_not_overflow() {
        let _ = jdn_from_date(i32::MAX, u32::MAX, u32::MAX);
        let _ = jdn_from_date(i32::MIN, 0, 0);
        assert!(!is_valid_solar(i32::MAX, u32::MAX, 1));
    }

    #[test]
    fn roundtrip_range() {
        for j in jdn_from_date(1990, 1, 1)..jdn_from_date(3010, 1, 1) {
            let (y, m, d) = date_from_jdn(j);
            assert!(is_valid_solar(y, m, d));
            assert_eq!(jdn_from_date(y, m, d), j);
        }
    }
}
