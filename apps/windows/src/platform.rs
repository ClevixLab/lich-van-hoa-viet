//! Phần phụ thuộc hệ điều hành của mô-đun Người thân: thư mục dữ liệu, múi giờ, hộp chọn tệp, thông báo
//! Windows đặt lịch trước (02 §7.5). Trên Linux (kiểm thử, chụp màn hình) các hàm Windows có bản thay thế
//! không làm gì. Không dùng mạng.

use std::cell::RefCell;
use std::path::PathBuf;

/// AppUserModelID của ứng dụng. Phải trùng `System.AppUserModel.ID` của lối tắt Start Menu do bộ cài MSI tạo
/// (wix/main.wxs) — ứng dụng chưa đóng gói (không MSIX) cần lối tắt này để Windows nhận thông báo (R-WIN-1).
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
    /// Lưu tệp với tên gợi ý (".ics" hoặc ".lvhv").
    Save(String),
}

/// Kiểm thử thay hộp chọn tệp bằng hàm trả đường dẫn.
#[allow(dead_code)]
pub fn set_dialog_override(f: Option<Box<dyn Fn(Dialog) -> Option<PathBuf>>>) {
    DIALOG.with(|d| *d.borrow_mut() = f);
}

/// Mở hộp chọn tệp của hệ điều hành. `None` khi người dùng hủy (hoặc không có hộp thoại trên máy này).
pub fn pick(kind: Dialog) -> Option<PathBuf> {
    let o = DIALOG.with(|d| d.borrow().as_ref().map(|f| f(kind.clone())));
    if let Some(r) = o {
        return r;
    }
    os_pick(kind)
}

#[cfg(windows)]
#[allow(unsafe_code)]
fn os_pick(kind: Dialog) -> Option<PathBuf> {
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
fn os_pick(_kind: Dialog) -> Option<PathBuf> {
    None
}

// ------------------------------------------------------------------------------------------------
// Thông báo Windows đặt lịch trước (ScheduledToastNotification)
// ------------------------------------------------------------------------------------------------

/// Một thông báo đang chờ trong hệ thống: `id` của lõi (`lvhv:…`), thời điểm, nội dung `title\nbody`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingToast {
    pub id: String,
    pub at_unix_s: i64,
    pub content: String,
}

/// Mã ngắn (12 chữ hex, 48 bit đầu của FNV-1a 64 bit) của id lõi. `Id`/`Tag` của thông báo đặt lịch có giới hạn
/// độ dài: CI Windows báo 0x803E0120 "The size of the developer id for scheduled notification is too large" với
/// 16 ký tự, nên dùng 12. Xung đột không đáng kể với ≤ 64 thông báo chờ.
pub fn short_id(id: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in id.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:012x}", h >> 16)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn xml_unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// Nội dung XML của một thông báo (mẫu ToastGeneric). `launch` giữ id của lõi để đọc lại khi xoay vòng.
pub fn toast_xml(id: &str, title: &str, body: &str) -> String {
    format!(
        "<toast launch=\"{}\"><visual><binding template=\"ToastGeneric\"><text>{}</text><text>{}</text></binding></visual></toast>",
        xml_escape(id),
        xml_escape(title),
        xml_escape(body)
    )
}

/// Đọc lại (id lõi, nội dung) từ XML do `toast_xml` sinh.
pub fn parse_toast_xml(xml: &str) -> Option<(String, String)> {
    let a = xml.find("launch=\"")? + 8;
    let b = a + xml[a..].find('"')?;
    let id = xml_unescape(&xml[a..b]);
    let mut texts = Vec::new();
    let mut rest = &xml[b..];
    while let Some(i) = rest.find("<text") {
        let s = i + rest[i..].find('>')? + 1;
        let e = s + rest[s..].find("</text>")?;
        texts.push(xml_unescape(&rest[s..e]));
        rest = &rest[e + 7..];
    }
    let content = match texts.as_slice() {
        [t] => t.clone(),
        [t, b, ..] => format!("{t}\n{b}"),
        [] => String::new(),
    };
    Some((id, content))
}

/// Tình trạng thông báo Windows trên máy này.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToastState {
    /// Đặt lịch được.
    Ready,
    /// Windows có thông báo nhưng người dùng/chính sách đang tắt cho ứng dụng này.
    Disabled(String),
    /// Không đọc được cài đặt thông báo (Windows trả lỗi), nhưng vẫn thử đặt lịch được.
    Unknown(String),
    /// Không dùng được (không phải Windows, hoặc không tạo được bộ gửi thông báo).
    Unavailable(String),
}

#[cfg(windows)]
mod toast_os {
    use super::*;
    use windows::core::HSTRING;
    use windows::Data::Xml::Dom::XmlDocument;
    use windows::Foundation::DateTime;
    use windows::UI::Notifications::{
        NotificationSetting, ScheduledToastNotification, ToastNotificationManager, ToastNotifier,
    };

    const EPOCH_DIFF_S: i64 = 11_644_473_600;

    #[allow(unsafe_code)]
    pub fn init() {
        use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;
        // SAFETY: chuỗi HSTRING hợp lệ suốt lời gọi.
        let _ = unsafe { SetCurrentProcessExplicitAppUserModelID(&HSTRING::from(AUMID)) };
    }

    fn notifier() -> windows::core::Result<ToastNotifier> {
        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))
    }

    pub fn state() -> ToastState {
        let n = match notifier() {
            Ok(n) => n,
            Err(e) => {
                return ToastState::Unavailable(format!(
                    "Windows chưa cho ứng dụng gửi thông báo ({}).",
                    e.code()
                ))
            }
        };
        match n.Setting() {
            Ok(NotificationSetting::Enabled) => ToastState::Ready,
            Ok(NotificationSetting::DisabledForApplication) => {
                ToastState::Disabled("Thông báo của ứng dụng đang tắt trong Cài đặt Windows (Hệ thống › Thông báo).".into())
            }
            Ok(NotificationSetting::DisabledForUser) => {
                ToastState::Disabled("Thông báo đang tắt cho tài khoản Windows này.".into())
            }
            Ok(NotificationSetting::DisabledByGroupPolicy) => {
                ToastState::Disabled("Thông báo bị tắt bởi chính sách của máy.".into())
            }
            Ok(_) => ToastState::Disabled("Windows đang tắt thông báo trên máy này.".into()),
            // 0x80070490 (không tìm thấy) gặp trên máy CI ngay sau khi cài; vẫn thử đặt lịch.
            Err(e) => ToastState::Unknown(format!("{}", e.code())),
        }
    }

    pub fn pending() -> windows::core::Result<Vec<PendingToast>> {
        let n = notifier()?;
        let v = n.GetScheduledToastNotifications()?;
        let mut out = Vec::new();
        for i in 0..v.Size()? {
            let s = v.GetAt(i)?;
            let xml = s.Content()?.GetXml()?.to_string();
            let Some((id, content)) = parse_toast_xml(&xml) else {
                continue;
            };
            let at = s.DeliveryTime()?.UniversalTime / 10_000_000 - EPOCH_DIFF_S;
            out.push(PendingToast {
                id,
                at_unix_s: at,
                content,
            });
        }
        Ok(out)
    }

    pub fn remove(ids: &[String]) -> windows::core::Result<usize> {
        let n = notifier()?;
        let v = n.GetScheduledToastNotifications()?;
        let mut k = 0;
        for i in 0..v.Size()? {
            let s = v.GetAt(i)?;
            let xml = s.Content()?.GetXml()?.to_string();
            if let Some((id, _)) = parse_toast_xml(&xml) {
                if ids.contains(&id) {
                    n.RemoveFromSchedule(&s)?;
                    k += 1;
                }
            }
        }
        Ok(k)
    }

    pub fn add(id: &str, at_unix_s: i64, title: &str, body: &str) -> windows::core::Result<()> {
        let n = notifier()?;
        let doc = XmlDocument::new()?;
        doc.LoadXml(&HSTRING::from(toast_xml(id, title, body)))?;
        let t = DateTime {
            UniversalTime: (at_unix_s + EPOCH_DIFF_S) * 10_000_000,
        };
        let s = ScheduledToastNotification::CreateScheduledToastNotification(&doc, t)?;
        let sid = short_id(id);
        s.SetId(&HSTRING::from(sid.as_str()))?;
        s.SetTag(&HSTRING::from(sid.as_str()))?;
        s.SetGroup(&HSTRING::from("lvhv"))?;
        n.AddToSchedule(&s)
    }
}

/// Đặt định danh ứng dụng cho tiến trình (thanh tác vụ và thông báo dùng chung một AUMID).
pub fn init_app_identity() {
    #[cfg(windows)]
    toast_os::init();
}

/// Chế độ chụp màn hình / đo bộ nhớ: dữ liệu mẫu không được đặt thành thông báo thật trên máy.
static DEMO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_demo_mode() {
    DEMO.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub fn toast_state() -> ToastState {
    if DEMO.load(std::sync::atomic::Ordering::Relaxed) {
        return ToastState::Unavailable(
            "Chế độ chụp màn hình với dữ liệu mẫu: không đặt thông báo thật trên máy này.".into(),
        );
    }
    #[cfg(windows)]
    {
        toast_os::state()
    }
    #[cfg(not(windows))]
    {
        ToastState::Unavailable("Thông báo của Windows chỉ có trên Windows.".into())
    }
}

/// Thông báo đang chờ do ứng dụng này đặt.
pub fn pending_toasts() -> Result<Vec<PendingToast>, String> {
    #[cfg(windows)]
    {
        toast_os::pending().map_err(|e| e.to_string())
    }
    #[cfg(not(windows))]
    {
        Ok(Vec::new())
    }
}

/// Áp kế hoạch `reschedule` của lõi: gỡ `remove`, thay `replace` (gỡ rồi thêm), thêm `add`.
/// Trả số thông báo đang chờ sau khi áp. Thông báo có thời điểm đã qua bị Windows từ chối — bỏ qua.
#[allow(unused_variables)]
pub fn apply_plan(remove: &[String], add: &[PendingToastNew]) -> Result<usize, String> {
    #[cfg(windows)]
    {
        let mut gone: Vec<String> = remove.to_vec();
        gone.extend(add.iter().filter(|a| a.replace).map(|a| a.id.clone()));
        toast_os::remove(&gone).map_err(|e| e.to_string())?;
        let now = now_unix();
        for a in add.iter().filter(|a| a.at_unix_s > now) {
            toast_os::add(&a.id, a.at_unix_s, &a.title, &a.body).map_err(|e| e.to_string())?;
        }
        Ok(toast_os::pending().map_err(|e| e.to_string())?.len())
    }
    #[cfg(not(windows))]
    {
        Err("Thông báo của Windows chỉ có trên Windows.".into())
    }
}

/// Tự kiểm (CI, R-WIN-1): đặt một thông báo thử sau 1 giờ, đọc lại từ danh sách chờ của Windows, gỡ đi.
pub fn self_test_toast() -> Result<String, String> {
    let state = toast_state();
    #[cfg(windows)]
    {
        let id = "lvhv:kiem-thu-thong-bao".to_string();
        let at = now_unix() + 3600;
        let before = toast_os::pending().map_err(|e| format!("{state:?}; đọc danh sách chờ lỗi: {e}"))?.len();
        toast_os::add(&id, at, "Kiểm thử", "Thông báo thử của Lịch Việt")
            .map_err(|e| format!("{state:?}; đặt thông báo lỗi: {e}"))?;
        let list = toast_os::pending().map_err(|e| format!("{state:?}; đọc lại lỗi: {e}"))?;
        let found = list.iter().find(|p| p.id == id).cloned();
        toast_os::remove(&[id]).map_err(|e| format!("gỡ lỗi: {e}"))?;
        let after = toast_os::pending().map(|l| l.len()).unwrap_or(usize::MAX);
        match found {
            Some(p) if (p.at_unix_s - at).abs() <= 1 => Ok(format!(
                "Thông báo đặt lịch: ĐẶT ĐƯỢC. Trạng thái {state:?}; chờ trước {before}, sau khi đặt {}, sau khi gỡ {after}; nội dung đọc lại: {:?}",
                list.len(),
                p.content
            )),
            Some(p) => Err(format!("đọc lại sai thời điểm: {} ≠ {at}", p.at_unix_s)),
            None => Err(format!("{state:?}; đã đặt nhưng không thấy trong danh sách chờ")),
        }
    }
    #[cfg(not(windows))]
    {
        Err(format!("{state:?}"))
    }
}

/// Một thông báo cần đặt (từ `add` hoặc `replace` của `reschedule_json`).
#[derive(Clone, Debug)]
pub struct PendingToastNew {
    pub id: String,
    pub at_unix_s: i64,
    pub title: String,
    pub body: String,
    pub replace: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_xml_roundtrip() {
        let x = toast_xml("lvhv:abc:2026-07-28:k=3", "Còn 3 ngày: <Giỗ> & \"x\"", "Thứ Ba, 28/07/2026");
        let (id, c) = parse_toast_xml(&x).unwrap();
        assert_eq!(id, "lvhv:abc:2026-07-28:k=3");
        assert_eq!(c, "Còn 3 ngày: <Giỗ> & \"x\"\nThứ Ba, 28/07/2026");
        assert_eq!(short_id("lvhv:sentinel").len(), 12);
        assert_ne!(short_id("a"), short_id("b"));
        // Lối tắt Start Menu của bộ cài mang đúng AUMID này (R-WIN-1).
        assert!(include_str!("../wix/main.wxs")
            .contains(&format!("Key='System.AppUserModel.ID' Value='{AUMID}'")));
    }
}
