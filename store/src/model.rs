//! Kiểu dữ liệu của 01 §2–§3 (tên trường đúng như lược đồ `backup-v1.schema.json`).
//!
//! Mọi giá trị đi vào kho đều qua [`crate::validate`] trước; các kiểu ở đây chỉ là dạng có kiểu của
//! JSON đã hợp lệ. Thứ tự trường không quan trọng: JSON xuất ra luôn ở dạng chuẩn (khóa sắp xếp, xem
//! [`crate::canonical_json`]).
use serde::{Deserialize, Serialize};

/// `"solar"` | `"lunar"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Calendar {
    /// Dương lịch.
    Solar,
    /// Âm lịch.
    Lunar,
}

/// `"day"` | `"month_day"` | `"year"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Precision {
    /// Đủ ngày/tháng/năm.
    Day,
    /// Biết ngày và tháng, không biết năm (chỉ âm lịch).
    MonthDay,
    /// Chỉ biết năm.
    Year,
}

/// 01 §2.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DateValue {
    pub calendar: Calendar,
    pub precision: Precision,
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
    pub leap: bool,
}

/// 01 §2.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LunarDay {
    pub year: Option<i32>,
    pub month: u32,
    pub day: u32,
    pub leap: bool,
}

/// 01 §2.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SolarDay {
    pub y: i32,
    pub m: u32,
    pub d: u32,
}

/// `"entered"` | `"converted"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LunarSource {
    Entered,
    Converted,
}

/// `"precise"` | `"hnd"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MethodName {
    Precise,
    Hnd,
}

impl MethodName {
    /// Phương pháp tương ứng của lõi.
    pub fn core(self) -> lvhv_core::Method {
        match self {
            MethodName::Precise => lvhv_core::Method::Precise,
            MethodName::Hnd => lvhv_core::Method::Hnd,
        }
    }
    /// Từ phương pháp của lõi.
    pub fn of(m: lvhv_core::Method) -> MethodName {
        match m {
            lvhv_core::Method::Precise => MethodName::Precise,
            lvhv_core::Method::Hnd => MethodName::Hnd,
        }
    }
}

/// `"huong_tho"` | `"huong_duong"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TuoiLabel {
    HuongTho,
    HuongDuong,
}

/// `DeathInfo.tuoi`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tuoi {
    pub value: Option<i32>,
    pub label: TuoiLabel,
    pub manual: bool,
    pub approx: bool,
}

/// 01 §3.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeathInfo {
    pub input: DateValue,
    pub lunar: LunarDay,
    pub solar: Option<SolarDay>,
    pub lunar_source: LunarSource,
    pub method: MethodName,
    pub tuoi: Tuoi,
}

/// 01 §3.3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BirthInfo {
    pub input: DateValue,
    pub noi_sinh: String,
    pub lunar_year: Option<i32>,
    pub lunar_year_approx: bool,
    pub can_chi: Option<String>,
    pub lunar: Option<LunarDay>,
    pub solar: Option<SolarDay>,
}

/// 01 §3.6.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReminderRule {
    pub id: String,
    pub enabled: bool,
    pub offsets_days: Vec<u32>,
    pub time_local: String,
    pub tien_thuong: bool,
}

/// `Person.reminders`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reminders {
    pub gio: Option<ReminderRule>,
    pub sinh_nhat: Option<ReminderRule>,
    pub sinh_nhat_am: Option<ReminderRule>,
}

/// `"phat_giao"` | `"truyen_thong"` | `"cong_giao"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrayStyle {
    PhatGiao,
    TruyenThong,
    CongGiao,
}

/// 01 §3.7.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemorialSettings {
    pub tray_style: TrayStyle,
    pub incense: bool,
    pub bia_tieu_de: String,
    pub loi_tuong_niem: String,
    pub gio_het_nam: u32,
    pub huong_duong_duoi: i32,
}

/// `"deceased"` | `"living"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Deceased,
    Living,
}

/// `"feb28"` | `"mar1"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Feb29Rule {
    Feb28,
    Mar1,
}

/// 01 §3.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Person {
    pub id: String,
    pub kind: Kind,
    pub danh_xung: String,
    pub ho_ten: String,
    pub ten_hieu: String,
    pub quan_he: String,
    pub birth: Option<BirthInfo>,
    pub death: Option<DeathInfo>,
    pub noi_an_tang: String,
    pub photo_id: Option<String>,
    pub memorial: Option<MemorialSettings>,
    pub reminders: Reminders,
    pub sinh_nhat_am: bool,
    pub feb29_rule: Feb29Rule,
    pub ghi_chu: String,
    pub created_at: String,
    pub updated_at: String,
    pub rev: i64,
}

/// 01 §3.4.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub id: String,
    pub type_id: String,
    pub tieu_de: String,
    pub date: DateValue,
    pub recurring: bool,
    pub person_ids: Vec<String>,
    pub ghi_chu: String,
    pub reminder: Option<ReminderRule>,
    pub created_at: String,
    pub updated_at: String,
    pub rev: i64,
}

/// 01 §3.5.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventType {
    pub id: String,
    pub builtin: Option<String>,
    pub ten: String,
    pub default_recurring: bool,
    pub created_at: String,
    pub updated_at: String,
    pub rev: i64,
}

/// 01 §3.8 (siêu dữ liệu; byte ảnh nằm ngoài).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Photo {
    pub id: String,
    pub mime: String,
    pub width: u32,
    pub height: u32,
    pub bytes: u32,
    pub sha256: String,
    pub stripped: bool,
    pub created_at: String,
}

/// 01 §3.9.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppSettings {
    pub reminders_enabled: bool,
    pub default_offsets_days: Vec<u32>,
    pub default_time_local: String,
    pub upcoming_days: u32,
    pub last_backup_at: Option<String>,
    pub backup_nag_days: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            reminders_enabled: true,
            default_offsets_days: vec![0, 3],
            default_time_local: "07:00".into(),
            upcoming_days: 30,
            last_backup_at: None,
            backup_nag_days: 30,
        }
    }
}

/// `app` của tài liệu sao lưu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub platform: String,
}

/// Tài liệu sao lưu (01 §5) — cũng là "ảnh chụp" toàn bộ kho.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupDoc {
    pub format: String,
    pub schema_version: u32,
    pub app: AppInfo,
    pub exported_at: String,
    pub settings: AppSettings,
    pub event_types: Vec<EventType>,
    pub persons: Vec<Person>,
    pub events: Vec<Event>,
    pub photos: Vec<Photo>,
}

/// Bốn loại sự kiện có sẵn (01 §3.5): (builtin, id cố định, tên mặc định, default_recurring).
pub const BUILTIN_EVENT_TYPES: [(&str, &str, &str, bool); 4] = [
    (
        "tot_nghiep",
        "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0001",
        "Tốt nghiệp",
        false,
    ),
    (
        "ngay_cuoi",
        "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0002",
        "Ngày cưới",
        true,
    ),
    (
        "dinh_hon",
        "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0003",
        "Đính hôn",
        false,
    ),
    (
        "du_hoc",
        "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0004",
        "Du học",
        false,
    ),
];
