//! Phần phụ thuộc hệ điều hành của mô-đun Người thân: thư mục dữ liệu, múi giờ (kể cả giờ mùa hè), hộp chọn tệp
//! (con của cửa sổ chính), định danh ứng dụng. Thông báo đặt lịch: `toasts.rs`. Trên Linux (kiểm thử, chụp màn
//! hình) các hàm Windows có bản thay thế không làm gì. Không dùng mạng.

use std::cell::RefCell;
use std::path::PathBuf;

/// AppUserModelID của ứng dụng. Phải trùng `System.AppUserModel.ID` của lối tắt Start Menu do bộ cài MSI tạo
/// (wix/main.wxs) — ứng dụng chưa đóng gói (không MSIX) cần lối tắt này để Windows nhận thông báo (R-WIN-1).
#[cfg_attr(not(windows), allow(dead_code))]
pub const AUMID: &str = "ClevixLab.LichVanHoaViet";

/// Thư mục dữ liệu riêng của ứng dụng (01 §4.2): `%LOCALAPPDATA%\ClevixLab\LichVanHoaViet`.
/// Biến môi trường `LVHV_DATA_DIR` thay thế (kiểm thử, chế độ chụp màn hình).
pub fn data_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("LVHV_DATA_DIR").filter(|d| !d.is_empty()) {
        return PathBuf::from(d);
    }
    #[cfg(windows)]
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    #[cfg(not(windows))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(std::env::temp_dir);
    base.join("ClevixLab").join("LichVanHoaViet")
}

/// Giây Unix hiện tại. Kiểm thử đặt đồng hồ giả bằng `set_fake_now`.
pub fn now_unix() -> i64 {
    if let Some(t) = FAKE_NOW.with(|f| *f.borrow()) {
        return t;
    }
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

thread_local! {
    static FAKE_NOW: RefCell<Option<i64>> = const { RefCell::new(None) };
    static FAKE_OFFSET: RefCell<Option<i32>> = const { RefCell::new(None) };
    static DIALOG: RefCell<Option<Box<dyn Fn(Dialog) -> Option<PathBuf>>>> = const { RefCell::new(None) };
}

#[allow(dead_code)]
pub fn set_fake_now(t: Option<i64>) {
    FAKE_NOW.with(|f| *f.borrow_mut() = t);
}

/// Độ lệch giờ hiện tại của máy so với UTC (phút; Việt Nam = 420). Không có danh sách lần chuyển giờ mùa
/// hè: lời nhắc được đặt lại mỗi lần mở ứng dụng, nên lệch tối đa 1 giờ chỉ ở lời nhắc nằm sau một lần
/// đổi giờ mùa hè mà ứng dụng chưa được mở lại (giới hạn đã ghi trong docs/04-BUILD-NATIVE.md).
pub fn utc_offset_min() -> i32 {
    if let Some(o) = FAKE_OFFSET.with(|f| *f.borrow()) {
        return o;
    }
    os_offset().unwrap_or(420)
}

#[allow(dead_code)]
pub fn set_fake_offset(o: Option<i32>) {
    FAKE_OFFSET.with(|f| *f.borrow_mut() = o);
}

#[cfg(windows)]
#[allow(unsafe_code)]
fn os_offset() -> Option<i32> {
    use windows::Win32::System::Time::{GetTimeZoneInformation, TIME_ZONE_INFORMATION};
    let mut tz = TIME_ZONE_INFORMATION::default();
    // SAFETY: con trỏ tới biến cục bộ đủ kích thước; hàm chỉ ghi vào đó.
    let r = unsafe { GetTimeZoneInformation(&mut tz) };
    if r == u32::MAX {
        return None;
    }
    let bias = tz.Bias + if r == 2 { tz.DaylightBias } else { tz.StandardBias };
    Some(-bias)
}

#[cfg(not(windows))]
fn os_offset() -> Option<i32> {
    std::env::var("LVHV_TZ_OFFSET_MIN").ok()?.parse().ok()
}

/// Loại hộp chọn tệp.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dialog {
    /// Chọn ảnh để mở.
    OpenImage,
    /// Chọn tệp sao lưu (.lvhv) để mở.
    OpenBackup,
    /// Chọn tệp đính kèm báo tử: ảnh hoặc PDF (01 §3.8a).
    OpenAttachment,
    /// Lưu tệp với tên gợi ý (".ics" hoặc ".lvhv").
    Save(String),
}

/// Kiểm thử thay hộp chọn tệp bằng hàm trả đường dẫn.
#[allow(dead_code)]
pub fn set_dialog_override(f: Option<Box<dyn Fn(Dialog) -> Option<PathBuf>>>) {
    DIALOG.with(|d| *d.borrow_mut() = f);
}

/// Mở hộp chọn tệp của hệ điều hành. `None` khi người dùng hủy (hoặc không có hộp thoại trên máy này).
pub fn pick(owner: Option<isize>, kind: Dialog) -> Option<PathBuf> {
    let o = DIALOG.with(|d| d.borrow().as_ref().map(|f| f(kind.clone())));
    if let Some(r) = o {
        return r;
    }
    os_pick(owner, kind)
}

#[cfg(windows)]
#[allow(unsafe_code)]
fn os_pick(owner: Option<isize>, kind: Dialog) -> Option<PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    use windows::core::{PCWSTR, PWSTR};
    use windows::Win32::UI::Controls::Dialogs::{
        GetOpenFileNameW, GetSaveFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR,
        OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW,
    };
    fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }
    let (filter, title, def_ext, initial): (&str, &str, &str, String) = match &kind {
        Dialog::OpenImage => (
            "Ảnh (*.jpg;*.jpeg;*.png;*.webp;*.gif;*.bmp)\0*.jpg;*.jpeg;*.png;*.webp;*.gif;*.bmp\0\0",
            "Chọn ảnh",
            "",
            String::new(),
        ),
        Dialog::OpenAttachment => (
            "Ảnh hoặc PDF (*.jpg;*.jpeg;*.png;*.webp;*.pdf)\0*.jpg;*.jpeg;*.png;*.webp;*.gif;*.bmp;*.pdf\0\0",
            "Chọn tệp đính kèm",
            "",
            String::new(),
        ),
        Dialog::OpenBackup => (
            "Tệp sao lưu Lịch và Văn hóa Việt (*.lvhv)\0*.lvhv\0Mọi tệp (*.*)\0*.*\0\0",
            "Chọn tệp sao lưu",
            "",
            String::new(),
        ),
        Dialog::Save(name) if name.ends_with(".ics") => (
            "Tệp lịch (*.ics)\0*.ics\0\0",
            "Lưu tệp lịch",
            "ics",
            name.clone(),
        ),
        Dialog::Save(name) => (
            "Tệp sao lưu (*.lvhv)\0*.lvhv\0\0",
            "Lưu tệp sao lưu",
            "lvhv",
            name.clone(),
        ),
    };
    let filter = w(filter);
    let mut title = w(title);
    title.push(0);
    let mut ext = w(def_ext);
    ext.push(0);
    let mut buf = vec![0u16; 1024];
    for (i, c) in initial.encode_utf16().take(1000).enumerate() {
        buf[i] = c;
    }
    let save = matches!(kind, Dialog::Save(_));
    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: windows::Win32::Foundation::HWND(owner.unwrap_or(0) as *mut core::ffi::c_void),
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrFile: PWSTR(buf.as_mut_ptr()),
        nMaxFile: buf.len() as u32,
        lpstrTitle: PCWSTR(title.as_ptr()),
        lpstrDefExt: if def_ext.is_empty() {
            PCWSTR::null()
        } else {
            PCWSTR(ext.as_ptr())
        },
        Flags: OFN_EXPLORER
            | OFN_NOCHANGEDIR
            | OFN_PATHMUSTEXIST
            | if save {
                OFN_OVERWRITEPROMPT
            } else {
                OFN_FILEMUSTEXIST
            },
        ..Default::default()
    };
    // SAFETY: mọi con trỏ trong `ofn` trỏ tới bộ đệm cục bộ còn sống suốt lời gọi; độ dài đúng.
    let ok = unsafe {
        if save {
            GetSaveFileNameW(&mut ofn)
        } else {
            GetOpenFileNameW(&mut ofn)
        }
    };
    if !ok.as_bool() {
        return None;
    }
    let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(PathBuf::from(std::ffi::OsString::from_wide(&buf[..n])))
}

#[cfg(not(windows))]
fn os_pick(_owner: Option<isize>, _kind: Dialog) -> Option<PathBuf> {
    None
}


// ------------------------------------------------------------------------------------------------
// Chế độ dữ liệu mẫu, định danh ứng dụng, cửa sổ cha của hộp thoại
// ------------------------------------------------------------------------------------------------

/// Chế độ chụp màn hình / đo bộ nhớ / kiểm thử: dữ liệu mẫu không được đặt thành thông báo thật.
static DEMO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_demo_mode() {
    DEMO.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub fn is_demo() -> bool {
    DEMO.load(std::sync::atomic::Ordering::Relaxed)
}

/// Đặt định danh ứng dụng cho tiến trình (thanh tác vụ và thông báo dùng chung một AUMID).
pub fn init_app_identity() {
    #[cfg(windows)]
    #[allow(unsafe_code)]
    {
        use windows::core::HSTRING;
        use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;
        // SAFETY: chuỗi HSTRING hợp lệ suốt lời gọi.
        let _ = unsafe { SetCurrentProcessExplicitAppUserModelID(&HSTRING::from(AUMID)) };
    }
}

/// HWND của cửa sổ chính (để hộp chọn tệp là hộp thoại con, luôn nằm trên cửa sổ và chặn cửa sổ cha).
#[allow(unused_variables)]
pub fn hwnd_of(w: &slint::Window) -> Option<isize> {
    #[cfg(windows)]
    {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let h = w.window_handle();
        match h.window_handle().ok()?.as_raw() {
            RawWindowHandle::Win32(x) => Some(x.hwnd.get()),
            _ => None,
        }
    }
    #[cfg(not(windows))]
    {
        None
    }
}

// ------------------------------------------------------------------------------------------------
// Múi giờ: các lần chuyển giờ mùa hè trong 401 ngày tới (MA3N-25-05), như Android/Apple
// ------------------------------------------------------------------------------------------------

/// Quy tắc chuyển giờ kiểu `SYSTEMTIME` của Windows: `year == 0` → "tuần thứ `day` (5 = cuối cùng) của tháng,
/// thứ `dow` (0 = Chủ nhật), lúc `hour:minute` giờ địa phương **trước** khi chuyển".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TzRule {
    pub year: u16,
    pub month: u16,
    pub dow: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
}

/// Một năm của múi giờ: `bias` (phút, UTC = địa phương + bias), độ lệch giờ chuẩn / giờ mùa hè.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TzYear {
    pub bias: i32,
    pub std_bias: i32,
    pub dst_bias: i32,
    pub std_date: TzRule,
    pub dst_date: TzRule,
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    // Số ngày kể từ 1970-01-01 (thuật toán của H. Hinnant).
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn days_in_month(y: i64, m: i64) -> i64 {
    days_from_civil(if m == 12 { y + 1 } else { y }, if m == 12 { 1 } else { m + 1 }, 1)
        - days_from_civil(y, m, 1)
}

/// Thời điểm (giây Unix, giờ địa phương coi như UTC) của quy tắc trong năm `y`.
fn rule_local_secs(r: &TzRule, y: i64) -> Option<i64> {
    if r.month == 0 {
        return None;
    }
    let (m, yy) = (r.month as i64, if r.year != 0 { r.year as i64 } else { y });
    let day = if r.year != 0 {
        r.day as i64
    } else {
        let first = days_from_civil(yy, m, 1);
        let wd_first = (first + 4).rem_euclid(7); // 1970-01-01 là Thứ Năm (4)
        let mut d = 1 + (r.dow as i64 - wd_first).rem_euclid(7) + 7 * (r.day as i64 - 1);
        while d > days_in_month(yy, m) {
            d -= 7;
        }
        d
    };
    Some(days_from_civil(yy, m, day) * 86_400 + r.hour as i64 * 3600 + r.minute as i64 * 60)
}

/// Các lần chuyển `(giây Unix UTC, độ lệch mới tính bằng phút)` trong một năm.
pub fn year_transitions(y: i64, z: &TzYear) -> Vec<(i64, i32)> {
    let std_off = -(z.bias + z.std_bias);
    let dst_off = -(z.bias + z.dst_bias);
    let mut v = Vec::new();
    if z.dst_date.month == 0 || z.std_date.month == 0 || std_off == dst_off {
        return v;
    }
    // Vào giờ mùa hè: giờ địa phương theo giờ chuẩn; ra: theo giờ mùa hè.
    if let Some(l) = rule_local_secs(&z.dst_date, y) {
        v.push((l - std_off as i64 * 60, dst_off));
    }
    if let Some(l) = rule_local_secs(&z.std_date, y) {
        v.push((l - dst_off as i64 * 60, std_off));
    }
    v.sort();
    v
}

/// Múi giờ thiết bị cho lõi: độ lệch hiện tại + các lần chuyển trong `(now, now + 401 ngày]`.
pub fn device_zone() -> lvhv_ffi::DeviceZone {
    let now = now_unix();
    let offset = utc_offset_min();
    let mut transitions = Vec::new();
    if FAKE_OFFSET.with(|f| f.borrow().is_none()) {
        let y = 1970 + now.div_euclid(86_400 * 365.2425 as i64);
        for yy in y - 1..=y + 2 {
            if let Some(z) = os_year(yy) {
                for (at, off) in year_transitions(yy, &z) {
                    if at > now && at <= now + 401 * 86_400 {
                        transitions.push(lvhv_ffi::ZoneTransition { at_unix_s: at, offset_min: off });
                    }
                }
            }
        }
        transitions.sort_by_key(|t| t.at_unix_s);
        transitions.dedup_by_key(|t| t.at_unix_s);
    }
    lvhv_ffi::DeviceZone { offset_min: offset, transitions }
}

#[cfg(windows)]
#[allow(unsafe_code)]
fn os_year(y: i64) -> Option<TzYear> {
    use windows::Win32::System::Time::{GetTimeZoneInformationForYear, TIME_ZONE_INFORMATION};
    let mut tz = TIME_ZONE_INFORMATION::default();
    // SAFETY: con trỏ tới biến cục bộ; `None` = múi giờ hiện tại của máy.
    unsafe { GetTimeZoneInformationForYear(u16::try_from(y).ok()?, None, &mut tz) }.ok()?;
    let r = |s: &windows::Win32::Foundation::SYSTEMTIME| TzRule {
        year: s.wYear,
        month: s.wMonth,
        dow: s.wDayOfWeek,
        day: s.wDay,
        hour: s.wHour,
        minute: s.wMinute,
    };
    Some(TzYear {
        bias: tz.Bias,
        std_bias: tz.StandardBias,
        dst_bias: tz.DaylightBias,
        std_date: r(&tz.StandardDate),
        dst_date: r(&tz.DaylightDate),
    })
}

#[cfg(not(windows))]
fn os_year(_y: i64) -> Option<TzYear> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn berlin_2026_transitions() {
        // Europe/Berlin: Bias −60, giờ mùa hè −60; vào: Chủ nhật cuối tháng 3 lúc 02:00; ra: Chủ nhật cuối tháng 10
        // lúc 03:00 → 29/03/2026 01:00Z và 25/10/2026 01:00Z.
        let z = TzYear {
            bias: -60,
            std_bias: 0,
            dst_bias: -60,
            std_date: TzRule { year: 0, month: 10, dow: 0, day: 5, hour: 3, minute: 0 },
            dst_date: TzRule { year: 0, month: 3, dow: 0, day: 5, hour: 2, minute: 0 },
        };
        let t = year_transitions(2026, &z);
        let utc = |y: i64, m: i64, d: i64, h: i64| days_from_civil(y, m, d) * 86_400 + h * 3600;
        assert_eq!(t, vec![(utc(2026, 3, 29, 1), 120), (utc(2026, 10, 25, 1), 60)]);
        // Việt Nam: không có giờ mùa hè.
        let vn = TzYear { bias: -420, std_bias: 0, dst_bias: 0, std_date: TzRule { year: 0, month: 0, dow: 0, day: 0, hour: 0, minute: 0 }, dst_date: TzRule { year: 0, month: 0, dow: 0, day: 0, hour: 0, minute: 0 } };
        assert!(year_transitions(2026, &vn).is_empty());
        // Hoa Kỳ (New York): Chủ nhật thứ hai tháng 3 lúc 02:00, Chủ nhật đầu tháng 11 lúc 02:00 → 08/03 07:00Z, 01/11 06:00Z.
        let ny = TzYear {
            bias: 300,
            std_bias: 0,
            dst_bias: -60,
            std_date: TzRule { year: 0, month: 11, dow: 0, day: 1, hour: 2, minute: 0 },
            dst_date: TzRule { year: 0, month: 3, dow: 0, day: 2, hour: 2, minute: 0 },
        };
        assert_eq!(year_transitions(2026, &ny), vec![(utc(2026, 3, 8, 7), -240), (utc(2026, 11, 1, 6), -300)]);
    }
}
