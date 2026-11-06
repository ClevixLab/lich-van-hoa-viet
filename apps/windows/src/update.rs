//! Phiên bản, nhắc cập nhật và nút cập nhật — KHÔNG dùng mạng.
//!
//! Ứng dụng không biết bản mới nhất là bản nào; nó chỉ biết ngày dựng của chính nó (build.rs ghi vào
//! `LVHV_BUILD_DATE`). Bản dựng hơn 60 ngày thì hiện thẻ nhắc; "Để sau" ẩn 30 ngày (lưu đúng một dòng
//! ngày vào `%LOCALAPPDATA%\Clevix\LichVanHoaViet\nhac-cap-nhat.txt`). Nút cập nhật giao địa chỉ cho
//! hệ điều hành mở (trình duyệt hoặc Microsoft Store); ứng dụng không tự kết nối.

use std::path::PathBuf;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BUILD_NUMBER: &str = env!("LVHV_BUILD_NUMBER");
pub const BUILD_DATE: &str = env!("LVHV_BUILD_DATE");
pub const MS_STORE_ID: &str = env!("LVHV_MS_STORE_ID");

pub const REMIND_AFTER_DAYS: i64 = 60;
pub const SNOOZE_DAYS: i64 = 30;
pub const DOWNLOAD_PAGE: &str = "https://clevixlab.github.io/lich-van-hoa-viet/";

pub fn copyright() -> String {
    format!("Bản quyền © 2026 của Clevix Lab, version {VERSION}")
}

pub fn running_version() -> String {
    format!("Phiên bản đang dùng: {VERSION} (bản dựng {BUILD_NUMBER})")
}

/// Địa chỉ nút cập nhật mở: trang Microsoft Store khi đã có mã sản phẩm, nếu không thì trang tải về.
pub fn update_url(store_id: &str) -> String {
    if store_id.is_empty() {
        DOWNLOAD_PAGE.to_string()
    } else {
        format!("ms-windows-store://pdp/?productid={store_id}")
    }
}

/// "YYYY-MM-DD" → số ngày kể từ 1970-01-01. Sai định dạng → None.
pub fn parse_days(s: &str) -> Option<i64> {
    let s = s.trim();
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let y: i64 = s[0..4].parse().ok()?;
    let m: i64 = s[5..7].parse().ok()?;
    let d: i64 = s[8..10].parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    // Howard Hinnant, days_from_civil.
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe - 719468)
}

pub fn days_to_string(z: i64) -> String {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}")
}

pub fn should_remind(build_day: Option<i64>, today: i64, snoozed_until: Option<i64>) -> bool {
    match build_day {
        None => false,
        Some(b) if today - b <= REMIND_AFTER_DAYS => false,
        Some(_) => snoozed_until.is_none_or(|u| today >= u),
    }
}

fn flag_file() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_STATE_HOME").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state")))?;
    Some(
        base.join("Clevix")
            .join("LichVanHoaViet")
            .join("nhac-cap-nhat.txt"),
    )
}

pub fn snoozed_until() -> Option<i64> {
    let s = std::fs::read_to_string(flag_file()?).ok()?;
    parse_days(&s)
}

/// Ghi ngày hết hạn "Để sau". Lỗi ghi tệp chỉ làm thẻ hiện lại lần mở sau, không ảnh hưởng gì khác.
pub fn snooze(today: i64) {
    if let Some(f) = flag_file() {
        if let Some(dir) = f.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(f, days_to_string(today + SNOOZE_DAYS));
    }
}

/// Giao địa chỉ cho hệ điều hành mở bằng ứng dụng mặc định. Không có mã mạng trong ứng dụng.
pub fn open_with_os(url: &str) -> bool {
    #[cfg(windows)]
    let r = std::process::Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn();
    #[cfg(target_os = "macos")]
    let r = std::process::Command::new("open").arg(url).spawn();
    #[cfg(all(not(windows), not(target_os = "macos")))]
    let r = std::process::Command::new("xdg-open").arg(url).spawn();
    r.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_round_trip() {
        assert_eq!(parse_days("1970-01-01"), Some(0));
        let d = parse_days("2026-11-06").unwrap();
        assert_eq!(days_to_string(d), "2026-11-06");
        assert_eq!(days_to_string(d + 61), "2027-01-06");
        assert_eq!(parse_days("2026-13-01"), None);
        assert_eq!(parse_days("hôm nay"), None);
        assert!(parse_days(BUILD_DATE).is_some(), "BUILD_DATE {BUILD_DATE}");
    }

    #[test]
    fn reminder_rules() {
        let b = parse_days("2026-11-06").unwrap();
        assert!(!should_remind(Some(b), b + 60, None));
        assert!(should_remind(Some(b), b + 61, None));
        let until = b + 70 + SNOOZE_DAYS;
        assert!(!should_remind(Some(b), b + 70 + 29, Some(until)));
        assert!(should_remind(Some(b), b + 70 + 30, Some(until)));
        assert!(!should_remind(None, b + 999, None));
    }

    #[test]
    fn texts_and_urls() {
        assert_eq!(VERSION, include_str!("../../../VERSION").trim());
        assert_eq!(
            copyright(),
            format!("Bản quyền © 2026 của Clevix Lab, version {VERSION}")
        );
        assert_eq!(update_url(""), DOWNLOAD_PAGE);
        assert_eq!(update_url("9ABC"), "ms-windows-store://pdp/?productid=9ABC");
    }
}
