//! Thông báo Windows đặt lịch trước (02 §7.4–7.5, R-WIN-1) — lớp đối chiếu giữa kế hoạch xoay vòng 64 của lõi
//! (`LvhvStore::reschedule_json`) và danh sách thông báo đang chờ của Windows.
//!
//! - Nội dung của lõi (`content` = `title` hoặc `title\nbody`) và id của lõi được ghi **nguyên văn** trong thuộc tính
//!   `launch` của thông báo (một URI `lichviet:` đã mã hóa phần trăm), không đọc lại từ chữ hiển thị (MA3N-25-01: khi
//!   ẩn tên, `body` rỗng). Khi `body` rỗng chỉ ghi một `<text>`.
//! - Bấm vào thông báo: Windows mở URI `lichviet:nguoi/<id hồ sơ>?…` — bộ cài `.msi` đăng ký giao thức này để chạy
//!   `LichViet.exe "<URI>"`, ứng dụng mở đúng hồ sơ. URI chỉ có UUID, không có tên hay ngày (S4').
//! - Thông báo của ứng dụng mà không đọc được thì bị gỡ, không bỏ qua; đọc danh sách chờ lỗi thì **không** áp kế hoạch
//!   (MA3N-25-02); Windows tắt thông báo của ứng dụng thì gỡ hết (MA3N-25-03).
//! - Lớp hệ điều hành là trait [`Toasts`]; kiểm thử dùng [`FakeToasts`] trong bộ nhớ (MA3N-25-08).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use lvhv_ffi::{DeviceZone, LvhvStore};
use serde_json::{json, Value};

/// Một thông báo đang chờ: id của lõi (`lvhv:…`), thời điểm (giây Unix), nội dung của lõi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingToast {
    pub id: String,
    pub at_unix_s: i64,
    pub content: String,
}

/// Một thông báo cần đặt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewToast {
    pub id: String,
    pub at_unix_s: i64,
    pub title: String,
    pub body: String,
    /// `content` của lõi (so sánh ở lần xoay vòng sau).
    pub content: String,
    /// Hồ sơ mở ra khi bấm vào thông báo.
    pub person: Option<String>,
}

/// Tình trạng thông báo Windows trên máy này.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToastState {
    /// Windows xác nhận ứng dụng được phép hiện thông báo.
    Ready,
    /// Người dùng / chính sách tắt thông báo của ứng dụng.
    Disabled(String),
    /// Tạo được bộ gửi nhưng Windows không cho đọc cài đặt (mã lỗi) — vẫn đặt lịch, nhưng không hứa sẽ hiện.
    Unknown(String),
    /// Không dùng được (không phải Windows, chế độ chụp màn hình, không tạo được bộ gửi).
    Unavailable(String),
}

/// Lớp thông báo của hệ điều hành.
pub trait Toasts {
    fn state(&self) -> ToastState;
    /// Thông báo của ứng dụng đang chờ. Thông báo không đọc được (thiếu `launch` hợp lệ) bị gỡ ngay.
    fn pending(&self) -> Result<Vec<PendingToast>, String>;
    fn remove(&self, ids: &[String]) -> Result<(), String>;
    fn remove_all(&self) -> Result<(), String>;
    fn add(&self, t: &NewToast) -> Result<(), String>;
}

// ------------------------------------------------------------------------------------------------
// URI và XML
// ------------------------------------------------------------------------------------------------

fn pct(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            o.push(b as char);
        } else {
            o.push_str(&format!("%{b:02X}"));
        }
    }
    o
}

fn unpct(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            let h = std::str::from_utf8(b.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(h, 16).ok()?);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// URI mở khi bấm thông báo; mang id và nội dung của lõi để đối chiếu.
pub fn launch_uri(person: Option<&str>, id: &str, content: &str) -> String {
    let path = match person {
        Some(p) if !p.is_empty() => format!("nguoi/{}", pct(p)),
        _ => "mo".to_string(),
    };
    format!("lichviet:{path}?id={}&c={}", pct(id), pct(content))
}

/// Phần đã giải của URI `lichviet:`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Launch {
    pub person: Option<String>,
    pub id: String,
    pub content: String,
}

pub fn parse_launch(uri: &str) -> Option<Launch> {
    let rest = uri.trim().strip_prefix("lichviet:")?;
    let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
    let path = path.trim_start_matches('/');
    let person = match path.strip_prefix("nguoi/") {
        Some(p) => Some(unpct(p.trim_end_matches('/'))?).filter(|p| !p.is_empty()),
        None => None,
    };
    let mut id = None;
    let mut content = None;
    for kv in query.split('&') {
        match kv.split_once('=') {
            Some(("id", v)) => id = Some(unpct(v)?),
            Some(("c", v)) => content = Some(unpct(v)?),
            _ => {}
        }
    }
    Some(Launch {
        person,
        id: id.unwrap_or_default(),
        content: content.unwrap_or_default(),
    })
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn xml_unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// XML của một thông báo (mẫu ToastGeneric, kích hoạt bằng giao thức). `body` rỗng → một `<text>`.
pub fn toast_xml(t: &NewToast) -> String {
    let body = if t.body.is_empty() {
        String::new()
    } else {
        format!("<text>{}</text>", xml_escape(&t.body))
    };
    format!(
        "<toast activationType=\"protocol\" launch=\"{}\"><visual><binding template=\"ToastGeneric\"><text>{}</text>{body}</binding></visual></toast>",
        xml_escape(&launch_uri(t.person.as_deref(), &t.id, &t.content)),
        xml_escape(&t.title),
    )
}

/// Đọc (id, nội dung của lõi) từ XML do `toast_xml` sinh — chỉ dựa vào thuộc tính `launch`, nên `<text/>`, dấu nháy
/// đơn hay thứ tự thuộc tính do Windows chuẩn hóa không ảnh hưởng. `None` → thông báo hỏng/lạ, sẽ bị gỡ.
pub fn parse_toast_xml(xml: &str) -> Option<(String, String)> {
    let i = xml.find("launch=")? + 7;
    let q = *xml.as_bytes().get(i)?;
    if q != b'"' && q != b'\'' {
        return None;
    }
    let end = i + 1 + xml[i + 1..].find(q as char)?;
    let l = parse_launch(&xml_unescape(&xml[i + 1..end]))?;
    (!l.id.is_empty()).then_some((l.id, l.content))
}

/// Mã ngắn (12 chữ hex) làm `Id`/`Tag` của thông báo: CI Windows từ chối 16 ký tự (0x803E0120).
pub fn short_id(id: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in id.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:012x}", h >> 16)
}

// ------------------------------------------------------------------------------------------------
// Đối chiếu với lõi
// ------------------------------------------------------------------------------------------------

/// Kết quả một lần đặt lại thông báo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncResult {
    Unavailable(String),
    /// Windows tắt thông báo của ứng dụng; đã gỡ mọi thông báo đang chờ (hoặc gỡ lỗi).
    Disabled { reason: String, cleared: Result<(), String> },
    /// Không đọc được danh sách chờ — không áp kế hoạch (tránh đặt trùng).
    ReadFailed(String),
    /// Đã áp kế hoạch. `confirmed`: Windows xác nhận cho phép hiện thông báo.
    Scheduled { pending: usize, added: usize, removed: usize, confirmed: bool },
    /// Áp kế hoạch lỗi giữa chừng.
    Failed(String),
}

/// id quy tắc nhắc → id hồ sơ (để thông báo mở đúng hồ sơ khi bấm).
fn rule_owners(ds: &Value) -> HashMap<String, String> {
    let mut m = HashMap::new();
    for p in ds["persons"].as_array().into_iter().flatten() {
        for k in ["gio", "sinh_nhat", "sinh_nhat_am"] {
            if let (Some(r), Some(pid)) = (p["reminders"][k]["id"].as_str(), p["id"].as_str()) {
                m.insert(r.to_string(), pid.to_string());
            }
        }
    }
    for e in ds["events"].as_array().into_iter().flatten() {
        if let (Some(r), Some(pid)) = (e["reminder"]["id"].as_str(), e["person_ids"][0].as_str()) {
            m.insert(r.to_string(), pid.to_string());
        }
    }
    m
}

/// Hồ sơ của một lời nhắc theo id lõi `lvhv:<id quy tắc>:<khóa>:<loại>`.
fn owner_of(owners: &HashMap<String, String>, id: &str) -> Option<String> {
    let rule = id.strip_prefix("lvhv:")?.split(':').next()?;
    owners.get(rule).cloned()
}

/// Đặt lại thông báo theo kế hoạch của lõi (02 §7.4) trên lớp `t`.
pub fn sync(t: &dyn Toasts, store: &LvhvStore, now: i64, zone: DeviceZone) -> SyncResult {
    let state = t.state();
    let confirmed = match &state {
        ToastState::Unavailable(r) => return SyncResult::Unavailable(r.clone()),
        ToastState::Disabled(r) => {
            return SyncResult::Disabled {
                reason: r.clone(),
                cleared: t.remove_all(),
            }
        }
        ToastState::Ready => true,
        ToastState::Unknown(_) => false,
    };
    let pending = match t.pending() {
        Ok(p) => p,
        Err(e) => return SyncResult::ReadFailed(e),
    };
    let pj: Vec<Value> = pending
        .iter()
        .map(|p| json!({"id": p.id, "at_unix_s": p.at_unix_s, "content": p.content}))
        .collect();
    let plan = match store.reschedule_json(now, zone, Value::Array(pj).to_string()) {
        Ok(p) => serde_json::from_str::<Value>(&p).unwrap_or(Value::Null),
        Err(e) => return SyncResult::Failed(e.to_string()),
    };
    let owners = store
        .dataset_json()
        .ok()
        .and_then(|d| serde_json::from_str::<Value>(&d).ok())
        .map(|d| rule_owners(&d))
        .unwrap_or_default();
    let s = |v: &Value| v.as_str().unwrap_or_default().to_string();
    let mut remove: Vec<String> = plan["remove"].as_array().into_iter().flatten().map(s).collect();
    let mut add = Vec::new();
    for k in ["add", "replace"] {
        for a in plan[k].as_array().into_iter().flatten() {
            let id = s(&a["id"]);
            if k == "replace" {
                remove.push(id.clone());
            }
            add.push(NewToast {
                person: owner_of(&owners, &id),
                id,
                at_unix_s: a["at_unix_s"].as_i64().unwrap_or(0),
                title: s(&a["title"]),
                body: s(&a["body"]),
                content: s(&a["content"]),
            });
        }
    }
    if !remove.is_empty() {
        if let Err(e) = t.remove(&remove) {
            return SyncResult::Failed(e);
        }
    }
    let mut added = 0;
    for a in add.iter().filter(|a| a.at_unix_s > now) {
        if let Err(e) = t.add(a) {
            return SyncResult::Failed(e);
        }
        added += 1;
    }
    match t.pending() {
        Ok(p) => SyncResult::Scheduled {
            pending: p.len(),
            added,
            removed: remove.len(),
            confirmed,
        },
        Err(e) => SyncResult::Failed(e),
    }
}

// ------------------------------------------------------------------------------------------------
// Bản không có thông báo và bản giả cho kiểm thử
// ------------------------------------------------------------------------------------------------

/// Không có thông báo (Linux, chế độ chụp màn hình).
pub struct NoToasts(pub String);

impl Toasts for NoToasts {
    fn state(&self) -> ToastState {
        ToastState::Unavailable(self.0.clone())
    }
    fn pending(&self) -> Result<Vec<PendingToast>, String> {
        Err(self.0.clone())
    }
    fn remove(&self, _: &[String]) -> Result<(), String> {
        Err(self.0.clone())
    }
    fn remove_all(&self) -> Result<(), String> {
        Err(self.0.clone())
    }
    fn add(&self, _: &NewToast) -> Result<(), String> {
        Err(self.0.clone())
    }
}

/// Bản giả trong bộ nhớ: lưu đúng XML như Windows, tùy chọn viết lại `<text></text>` thành `<text/>` (như DOM chuẩn
/// hóa), làm lỗi lần đọc danh sách chờ, đổi trạng thái.
#[derive(Clone, Default)]
#[allow(dead_code)]
pub struct FakeToasts(pub Rc<RefCell<FakeInner>>);

#[derive(Default)]
#[allow(dead_code)]
pub struct FakeInner {
    /// (XML, thời điểm)
    pub items: Vec<(String, i64)>,
    pub state: Option<ToastState>,
    pub fail_pending: bool,
    pub normalize_empty: bool,
    pub adds: usize,
    pub removes: usize,
}

#[allow(dead_code)]
impl FakeToasts {
    pub fn contents(&self) -> Vec<(String, String)> {
        self.0
            .borrow()
            .items
            .iter()
            .filter_map(|(x, _)| parse_toast_xml(x))
            .collect()
    }
    pub fn xml(&self) -> Vec<String> {
        self.0.borrow().items.iter().map(|(x, _)| x.clone()).collect()
    }
}

impl Toasts for FakeToasts {
    fn state(&self) -> ToastState {
        self.0.borrow().state.clone().unwrap_or(ToastState::Ready)
    }
    fn pending(&self) -> Result<Vec<PendingToast>, String> {
        let mut f = self.0.borrow_mut();
        if f.fail_pending {
            return Err("giả lập: không đọc được danh sách chờ".into());
        }
        let mut out = Vec::new();
        f.items.retain(|(x, at)| match parse_toast_xml(x) {
            Some((id, content)) => {
                out.push(PendingToast { id, at_unix_s: *at, content });
                true
            }
            None => false,
        });
        Ok(out)
    }
    fn remove(&self, ids: &[String]) -> Result<(), String> {
        let mut f = self.0.borrow_mut();
        let before = f.items.len();
        f.items
            .retain(|(x, _)| parse_toast_xml(x).is_none_or(|(id, _)| !ids.contains(&id)));
        f.removes += before - f.items.len();
        Ok(())
    }
    fn remove_all(&self) -> Result<(), String> {
        let mut f = self.0.borrow_mut();
        f.removes += f.items.len();
        f.items.clear();
        Ok(())
    }
    fn add(&self, t: &NewToast) -> Result<(), String> {
        let mut f = self.0.borrow_mut();
        let mut x = toast_xml(t);
        if f.normalize_empty {
            x = x.replace("<text></text>", "<text/>");
        }
        f.items.push((x, t.at_unix_s));
        f.adds += 1;
        Ok(())
    }
}

// ------------------------------------------------------------------------------------------------
// Windows
// ------------------------------------------------------------------------------------------------

#[cfg(windows)]
pub use win::WinToasts;

#[cfg(windows)]
mod win {
    use super::*;
    use crate::platform::AUMID;
    use windows::core::HSTRING;
    use windows::Data::Xml::Dom::XmlDocument;
    use windows::Foundation::DateTime;
    use windows::UI::Notifications::{
        NotificationSetting, ScheduledToastNotification, ToastNotificationManager, ToastNotifier,
    };

    const EPOCH_DIFF_S: i64 = 11_644_473_600;

    pub struct WinToasts;

    fn notifier() -> Result<ToastNotifier, String> {
        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))
            .map_err(|e| format!("không tạo được bộ gửi thông báo ({})", e.code()))
    }

    fn each(
        n: &ToastNotifier,
        mut f: impl FnMut(&ScheduledToastNotification, Option<(String, String)>) -> windows::core::Result<()>,
    ) -> Result<(), String> {
        let v = n
            .GetScheduledToastNotifications()
            .map_err(|e| format!("không đọc được danh sách chờ ({})", e.code()))?;
        let size = v.Size().map_err(|e| e.to_string())?;
        // Lấy hết trước rồi mới gỡ: gỡ trong lúc duyệt không làm lệch chỉ số.
        let mut all = Vec::new();
        for i in 0..size {
            all.push(v.GetAt(i).map_err(|e| e.to_string())?);
        }
        for s in &all {
            let parsed = s
                .Content()
                .and_then(|c| c.GetXml())
                .ok()
                .and_then(|x| parse_toast_xml(&x.to_string()));
            f(s, parsed).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    impl Toasts for WinToasts {
        fn state(&self) -> ToastState {
            let n = match notifier() {
                Ok(n) => n,
                Err(e) => return ToastState::Unavailable(e),
            };
            match n.Setting() {
                Ok(NotificationSetting::Enabled) => ToastState::Ready,
                Ok(NotificationSetting::DisabledForApplication) => ToastState::Disabled(
                    "Thông báo của ứng dụng đang tắt trong Cài đặt Windows (Hệ thống › Thông báo).".into(),
                ),
                Ok(NotificationSetting::DisabledForUser) => {
                    ToastState::Disabled("Thông báo đang tắt cho tài khoản Windows này.".into())
                }
                Ok(NotificationSetting::DisabledByGroupPolicy) => {
                    ToastState::Disabled("Thông báo bị tắt bởi chính sách của máy.".into())
                }
                Ok(_) => ToastState::Disabled("Windows đang tắt thông báo trên máy này.".into()),
                Err(e) => ToastState::Unknown(format!("{}", e.code())),
            }
        }
        fn pending(&self) -> Result<Vec<PendingToast>, String> {
            let n = notifier()?;
            let mut out = Vec::new();
            each(&n, |s, p| {
                match p {
                    Some((id, content)) => out.push(PendingToast {
                        id,
                        at_unix_s: s.DeliveryTime()?.UniversalTime / 10_000_000 - EPOCH_DIFF_S,
                        content,
                    }),
                    // Thông báo của ứng dụng mà không đọc được: gỡ (MA3N-25-01).
                    None => n.RemoveFromSchedule(s)?,
                }
                Ok(())
            })?;
            Ok(out)
        }
        fn remove(&self, ids: &[String]) -> Result<(), String> {
            let n = notifier()?;
            each(&n, |s, p| {
                if p.is_some_and(|(id, _)| ids.contains(&id)) {
                    n.RemoveFromSchedule(s)?;
                }
                Ok(())
            })
        }
        fn remove_all(&self) -> Result<(), String> {
            let n = notifier()?;
            each(&n, |s, _| n.RemoveFromSchedule(s))
        }
        fn add(&self, t: &NewToast) -> Result<(), String> {
            let n = notifier()?;
            let r = (|| -> windows::core::Result<()> {
                let doc = XmlDocument::new()?;
                doc.LoadXml(&HSTRING::from(toast_xml(t)))?;
                let at = DateTime {
                    UniversalTime: (t.at_unix_s + EPOCH_DIFF_S) * 10_000_000,
                };
                let s = ScheduledToastNotification::CreateScheduledToastNotification(&doc, at)?;
                let sid = short_id(&t.id);
                s.SetId(&HSTRING::from(sid.as_str()))?;
                s.SetTag(&HSTRING::from(sid.as_str()))?;
                s.SetGroup(&HSTRING::from("lvhv"))?;
                n.AddToSchedule(&s)
            })();
            r.map_err(|e| format!("đặt thông báo lỗi: {} ({})", e.message(), e.code()))
        }
    }
}

/// Lớp thông báo dùng khi chạy: Windows thật, trừ chế độ chụp màn hình / kiểm thử (không đặt thông báo thật).
pub fn system() -> Box<dyn Toasts> {
    if crate::platform::is_demo() {
        return Box::new(NoToasts(
            "Chế độ chụp màn hình với dữ liệu mẫu: không đặt thông báo thật trên máy này.".into(),
        ));
    }
    #[cfg(windows)]
    {
        Box::new(WinToasts)
    }
    #[cfg(not(windows))]
    {
        Box::new(NoToasts("Thông báo của Windows chỉ có trên Windows.".into()))
    }
}

/// Tự kiểm trên Windows thật (`--kiem-thong-bao`, CI): (1) thông báo có dòng phụ và **không** có dòng phụ đọc lại đúng
/// nội dung của lõi; (2) kho tạm với dữ liệu mẫu, ẩn tên (mặc định): đặt lịch hai lần liền, lần hai không gỡ/thêm gì;
/// (3) gỡ hết. Không đụng sổ người thân thật; thông báo thật (nếu có) được đặt lại ở lần mở ứng dụng kế tiếp.
pub fn self_test(t: &dyn Toasts, store: &LvhvStore, now: i64, zone: DeviceZone) -> Result<String, String> {
    let state = t.state();
    let mut log = vec![format!("Trạng thái {state:?}")];
    t.remove_all().map_err(|e| format!("gỡ hết lỗi: {e}"))?;
    for (id, body) in [("lvhv:kiem-thu:co-dong-phu", "Thông báo thử của Lịch Việt"), ("lvhv:kiem-thu:khong-dong-phu", "")] {
        let content = if body.is_empty() {
            "Kiểm thử".to_string()
        } else {
            format!("Kiểm thử\n{body}")
        };
        t.add(&NewToast {
            id: id.into(),
            at_unix_s: now + 3600,
            title: "Kiểm thử".into(),
            body: body.into(),
            content: content.clone(),
            person: Some("00000000-0000-4000-8000-000000000000".into()),
        })?;
        let p = t.pending()?;
        let got = p.iter().find(|x| x.id == id).ok_or(format!("không thấy {id} trong danh sách chờ"))?;
        if got.content != content || (got.at_unix_s - (now + 3600)).abs() > 1 {
            return Err(format!("{id}: đọc lại {:?} ≠ {:?}", got.content, content));
        }
        log.push(format!("{id}: đọc lại đúng ({:?})", got.content));
    }
    t.remove_all().map_err(|e| format!("gỡ hết lỗi: {e}"))?;
    let r1 = sync(t, store, now, zone.clone());
    let r2 = sync(t, store, now, zone);
    log.push(format!("Lần 1: {r1:?}"));
    log.push(format!("Lần 2: {r2:?}"));
    let ok = matches!(r2, SyncResult::Scheduled { added: 0, removed: 0, .. })
        && matches!(r1, SyncResult::Scheduled { added, .. } if added > 0);
    t.remove_all().map_err(|e| format!("gỡ hết lỗi: {e}"))?;
    if ok {
        Ok(format!("Thông báo đặt lịch: ĐẶT ĐƯỢC, XOAY VÒNG ỔN ĐỊNH. {}", log.join("; ")))
    } else {
        Err(format!("Xoay vòng không ổn định: {}", log.join("; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(id: &str, title: &str, body: &str) -> NewToast {
        NewToast {
            id: id.into(),
            at_unix_s: 2_000_000_000,
            title: title.into(),
            body: body.into(),
            content: if body.is_empty() { title.into() } else { format!("{title}\n{body}") },
            person: Some("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01".into()),
        }
    }

    #[test]
    fn xml_roundtrip_with_and_without_body() {
        let a = t("lvhv:r1:2026-07-28:k=3", "Còn 3 ngày: <Giỗ> & \"x\" 'y'", "Thứ Ba, 28/07/2026");
        let x = toast_xml(&a);
        assert_eq!(x.matches("<text>").count(), 2);
        assert_eq!(parse_toast_xml(&x), Some((a.id.clone(), a.content.clone())));
        // MA3N-25-01: dòng phụ rỗng (mặc định ẩn tên) → một <text>, nội dung đọc lại = content của lõi.
        let b = t("lvhv:sentinel", "Sắp đến ngày giỗ", "");
        let x = toast_xml(&b);
        assert_eq!(x.matches("<text>").count(), 1);
        assert_eq!(parse_toast_xml(&x), Some((b.id.clone(), "Sắp đến ngày giỗ".into())));
        // Windows chuẩn hóa: phần tử rỗng <text/>, nháy đơn, thực thể &apos;.
        let y = x.replace("\"", "'").replace("<text>Sắp đến ngày giỗ</text>", "<text>Sắp đến ngày giỗ</text><text/>");
        assert_eq!(parse_toast_xml(&y), Some((b.id.clone(), "Sắp đến ngày giỗ".into())));
        assert_eq!(parse_toast_xml("<toast><visual/></toast>"), None, "thông báo lạ → gỡ");
        let l = parse_launch(&launch_uri(Some("abc-1"), "lvhv:x:y:k=0", "A\nB")).unwrap();
        assert_eq!(l.person.as_deref(), Some("abc-1"));
        assert_eq!(l.content, "A\nB");
        assert_eq!(parse_launch("lichviet:mo").unwrap().person, None);
        assert_eq!(short_id("lvhv:sentinel").len(), 12);
        assert!(include_str!("../wix/main.wxs")
            .contains(&format!("Key='System.AppUserModel.ID' Value='{}'", crate::platform::AUMID)));
    }
}
