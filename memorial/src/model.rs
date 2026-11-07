//! Kiểu dữ liệu của docs/v0.2/01 (phần thuật toán đọc) và phân tích từ JSON lược đồ v1.
//! Chỉ đọc các trường mà thuật toán 02 dùng; kiểm tra lược đồ đầy đủ và V1–V8 thuộc lớp lưu trữ.

use crate::json::Value;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ymd {
    pub y: i32,
    pub m: u32,
    pub d: u32,
}

impl Ymd {
    pub fn new(y: i32, m: u32, d: u32) -> Ymd {
        Ymd { y, m, d }
    }
    pub fn jdn(&self) -> i64 {
        lvhv_core::jd::jdn_from_date(self.y, self.m, self.d)
    }
    pub fn from_jdn(j: i64) -> Ymd {
        let (y, m, d) = lvhv_core::jd::date_from_jdn(j);
        Ymd { y, m, d }
    }
    /// `YYYY-MM-DD`.
    pub fn iso(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.y, self.m, self.d)
    }
    pub fn parse(s: &str) -> Option<Ymd> {
        let b = s.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return None;
        }
        let y = s[0..4].parse().ok()?;
        let m = s[5..7].parse().ok()?;
        let d = s[8..10].parse().ok()?;
        lvhv_core::jd::is_valid_solar(y, m, d).then_some(Ymd { y, m, d })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Calendar {
    Solar,
    Lunar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    Day,
    MonthDay,
    Year,
}

/// 01 §2.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateValue {
    pub calendar: Calendar,
    pub precision: Precision,
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
    pub leap: bool,
}

impl DateValue {
    pub fn solar(y: i32, m: u32, d: u32) -> DateValue {
        DateValue {
            calendar: Calendar::Solar,
            precision: Precision::Day,
            year: Some(y),
            month: Some(m),
            day: Some(d),
            leap: false,
        }
    }
    pub fn lunar(y: Option<i32>, m: u32, d: u32, leap: bool) -> DateValue {
        DateValue {
            calendar: Calendar::Lunar,
            precision: if y.is_some() {
                Precision::Day
            } else {
                Precision::MonthDay
            },
            year: y,
            month: Some(m),
            day: Some(d),
            leap,
        }
    }
    pub fn year_only(calendar: Calendar, y: i32) -> DateValue {
        DateValue {
            calendar,
            precision: Precision::Year,
            year: Some(y),
            month: None,
            day: None,
            leap: false,
        }
    }
    /// JSON theo thứ tự khóa của 01 §2.1.
    pub fn to_json(&self, out: &mut String) {
        let opt = |v: Option<i64>| v.map_or("null".to_string(), |x| x.to_string());
        out.push_str(&format!(
            "{{\"calendar\":\"{}\",\"precision\":\"{}\",\"year\":{},\"month\":{},\"day\":{},\"leap\":{}}}",
            match self.calendar {
                Calendar::Solar => "solar",
                Calendar::Lunar => "lunar",
            },
            match self.precision {
                Precision::Day => "day",
                Precision::MonthDay => "month_day",
                Precision::Year => "year",
            },
            opt(self.year.map(i64::from)),
            opt(self.month.map(i64::from)),
            opt(self.day.map(i64::from)),
            self.leap
        ));
    }
}

/// 01 §2.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LunarDay {
    pub year: Option<i32>,
    pub month: u32,
    pub day: u32,
    pub leap: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgeLabel {
    HuongTho,
    HuongDuong,
}

impl AgeLabel {
    pub fn code(&self) -> &'static str {
        match self {
            AgeLabel::HuongTho => "huong_tho",
            AgeLabel::HuongDuong => "huong_duong",
        }
    }
    pub fn text(&self) -> &'static str {
        match self {
            AgeLabel::HuongTho => "Hưởng thọ",
            AgeLabel::HuongDuong => "Hưởng dương",
        }
    }
}

/// `death.tuoi` (01 §3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tuoi {
    pub value: Option<i32>,
    pub label: AgeLabel,
    pub manual: bool,
    pub approx: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LunarSource {
    Entered,
    Converted,
}

/// 01 §3.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeathInfo {
    pub input: DateValue,
    pub lunar: LunarDay,
    pub solar: Option<Ymd>,
    pub lunar_source: LunarSource,
    pub method: lvhv_core::Method,
    pub tuoi: Tuoi,
}

/// 01 §3.3 (phần dẫn xuất).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BirthInfo {
    pub input: DateValue,
    pub lunar_year: Option<i32>,
    pub lunar_year_approx: bool,
    pub can_chi: Option<String>,
    pub lunar: Option<LunarDay>,
    pub solar: Option<Ymd>,
}

/// 01 §3.6. `time_local` lưu dạng số phút từ 00:00.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderRule {
    pub id: String,
    pub enabled: bool,
    pub offsets_days: Vec<u32>,
    pub time_local: u32,
    pub tien_thuong: bool,
}

impl ReminderRule {
    /// Quy tắc mặc định (01 §3.6): `[0, 3]`, 07:00, tiên thường tắt (O-1).
    pub fn default_with_id(id: &str) -> ReminderRule {
        ReminderRule {
            id: id.to_string(),
            enabled: true,
            offsets_days: DEFAULT_OFFSETS.to_vec(),
            time_local: DEFAULT_TIME,
            tien_thuong: false,
        }
    }
}

pub const DEFAULT_OFFSETS: [u32; 2] = [0, 3];
pub const DEFAULT_TIME: u32 = 7 * 60;
/// Nút chọn nhanh trong giao diện (01 §3.6).
pub const PRESET_OFFSETS: [u32; 6] = [0, 1, 3, 5, 10, 15];
pub const CUSTOM_MIN: u32 = 1;
pub const CUSTOM_MAX: u32 = 60;
/// Mặc định O-2 / O-3.
pub const DEFAULT_GIO_HET_NAM: u32 = 2;
pub const DEFAULT_HUONG_DUONG_DUOI: i32 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonKind {
    Deceased,
    Living,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feb29Rule {
    Feb28,
    Mar1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    pub id: String,
    pub kind: PersonKind,
    pub danh_xung: String,
    pub ho_ten: String,
    pub birth: Option<BirthInfo>,
    pub death: Option<DeathInfo>,
    /// `memorial.gio_het_nam` (2 | 3).
    pub gio_het_nam: u32,
    /// `memorial.huong_duong_duoi`.
    pub huong_duong_duoi: i32,
    pub rule_gio: Option<ReminderRule>,
    pub rule_sinh_nhat: Option<ReminderRule>,
    pub rule_sinh_nhat_am: Option<ReminderRule>,
    pub sinh_nhat_am: bool,
    pub feb29_rule: Feb29Rule,
    pub rev: i64,
}

impl Person {
    /// Tên hiển thị = `danh_xung + " " + ho_ten` (02 §7.1); bỏ dấu cách khi không có danh xưng.
    /// Tên hiển thị: cách gọi (viết hoa chữ cái đầu khi hiển thị — YC-2026-07, kể cả dữ liệu nhập từ sao lưu cũ
    /// còn viết thường) + họ tên.
    pub fn display_name(&self) -> String {
        if self.danh_xung.is_empty() {
            self.ho_ten.clone()
        } else {
            format!("{} {}", upper_first(&self.danh_xung), self.ho_ten)
        }
    }
}

/// Khoảng trắng theo `String.prototype.trim` của JavaScript.
fn js_space(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}

/// Cắt khoảng trắng hai đầu rồi viết hoa chữ đầu, phần còn lại giữ nguyên (cùng `upperFirst` của tsport).
pub fn upper_first(s: &str) -> String {
    let mut it = s.trim_matches(js_space).chars();
    match it.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(it).collect(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventType {
    pub id: String,
    pub builtin: Option<String>,
    pub ten: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub id: String,
    pub type_id: String,
    pub tieu_de: String,
    pub date: DateValue,
    pub recurring: bool,
    pub person_ids: Vec<String>,
    pub reminder: Option<ReminderRule>,
    pub rev: i64,
}

/// Tập dữ liệu thuật toán cần (một tài liệu sao lưu lược đồ v1 đã giải mã).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Dataset {
    pub reminders_enabled: bool,
    pub persons: Vec<Person>,
    pub events: Vec<Event>,
    pub event_types: Vec<EventType>,
}

impl Dataset {
    pub fn person(&self, id: &str) -> Option<&Person> {
        self.persons.iter().find(|p| p.id == id)
    }
    pub fn event_type(&self, id: &str) -> Option<&EventType> {
        self.event_types.iter().find(|t| t.id == id)
    }
}

/// Lỗi đọc tài liệu JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelError(pub String);

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn e<T>(what: &str) -> Result<T, ModelError> {
    Err(ModelError(format!("trường không hợp lệ: {what}")))
}

fn field<'a>(v: &'a Value, k: &str) -> Result<&'a Value, ModelError> {
    v.get(k)
        .ok_or_else(|| ModelError(format!("thiếu trường: {k}")))
}
fn f_str(v: &Value, k: &str) -> Result<String, ModelError> {
    match field(v, k)?.as_str() {
        Some(s) => Ok(s.to_string()),
        None => e(k),
    }
}
fn f_bool(v: &Value, k: &str) -> Result<bool, ModelError> {
    match field(v, k)?.as_bool() {
        Some(b) => Ok(b),
        None => e(k),
    }
}
fn f_int(v: &Value, k: &str) -> Result<i64, ModelError> {
    match field(v, k)?.as_i64() {
        Some(i) => Ok(i),
        None => e(k),
    }
}
fn f_opt_int(v: &Value, k: &str) -> Result<Option<i64>, ModelError> {
    let x = field(v, k)?;
    if x.is_null() {
        return Ok(None);
    }
    match x.as_i64() {
        Some(i) => Ok(Some(i)),
        None => e(k),
    }
}
fn to_i32(x: i64, k: &str) -> Result<i32, ModelError> {
    i32::try_from(x).or_else(|_| e(k))
}
fn to_u32(x: i64, k: &str) -> Result<u32, ModelError> {
    u32::try_from(x).or_else(|_| e(k))
}

/// `"HH:MM"` → số phút; `None` nếu sai dạng hoặc ngoài 00:00–23:59.
pub fn parse_hhmm(s: &str) -> Option<u32> {
    let b = s.as_bytes();
    if b.len() != 5
        || b[2] != b':'
        || !b
            .iter()
            .enumerate()
            .all(|(i, c)| i == 2 || c.is_ascii_digit())
    {
        return None;
    }
    let h: u32 = s[0..2].parse().ok()?;
    let m: u32 = s[3..5].parse().ok()?;
    (h < 24 && m < 60).then_some(h * 60 + m)
}

pub fn fmt_hhmm(t: u32) -> String {
    format!("{:02}:{:02}", t / 60, t % 60)
}

pub fn parse_date_value(v: &Value) -> Result<DateValue, ModelError> {
    let calendar = match f_str(v, "calendar")?.as_str() {
        "solar" => Calendar::Solar,
        "lunar" => Calendar::Lunar,
        _ => return e("calendar"),
    };
    let precision = match f_str(v, "precision")?.as_str() {
        "day" => Precision::Day,
        "month_day" => Precision::MonthDay,
        "year" => Precision::Year,
        _ => return e("precision"),
    };
    let year = f_opt_int(v, "year")?
        .map(|x| to_i32(x, "year"))
        .transpose()?;
    let month = f_opt_int(v, "month")?
        .map(|x| to_u32(x, "month"))
        .transpose()?;
    let day = f_opt_int(v, "day")?.map(|x| to_u32(x, "day")).transpose()?;
    Ok(DateValue {
        calendar,
        precision,
        year,
        month,
        day,
        leap: f_bool(v, "leap")?,
    })
}

fn parse_lunar_day(v: &Value) -> Result<LunarDay, ModelError> {
    Ok(LunarDay {
        year: f_opt_int(v, "year")?
            .map(|x| to_i32(x, "year"))
            .transpose()?,
        month: to_u32(f_int(v, "month")?, "month")?,
        day: to_u32(f_int(v, "day")?, "day")?,
        leap: f_bool(v, "leap")?,
    })
}

fn parse_ymd_opt(v: &Value) -> Result<Option<Ymd>, ModelError> {
    if v.is_null() {
        return Ok(None);
    }
    Ok(Some(Ymd {
        y: to_i32(f_int(v, "y")?, "y")?,
        m: to_u32(f_int(v, "m")?, "m")?,
        d: to_u32(f_int(v, "d")?, "d")?,
    }))
}

pub fn parse_rule(v: &Value) -> Result<Option<ReminderRule>, ModelError> {
    if v.is_null() {
        return Ok(None);
    }
    let offs = match field(v, "offsets_days")?.as_arr() {
        Some(a) => a
            .iter()
            .map(|x| {
                x.as_i64()
                    .map_or(e("offsets_days"), |i| to_u32(i, "offsets_days"))
            })
            .collect::<Result<Vec<u32>, _>>()?,
        None => return e("offsets_days"),
    };
    let time_local = match parse_hhmm(&f_str(v, "time_local")?) {
        Some(t) => t,
        None => return e("time_local"),
    };
    Ok(Some(ReminderRule {
        id: f_str(v, "id")?,
        enabled: f_bool(v, "enabled")?,
        offsets_days: offs,
        time_local,
        tien_thuong: f_bool(v, "tien_thuong")?,
    }))
}

fn parse_person(v: &Value) -> Result<Person, ModelError> {
    let kind = match f_str(v, "kind")?.as_str() {
        "deceased" => PersonKind::Deceased,
        "living" => PersonKind::Living,
        _ => return e("kind"),
    };
    let birth = {
        let b = field(v, "birth")?;
        if b.is_null() {
            None
        } else {
            Some(BirthInfo {
                input: parse_date_value(field(b, "input")?)?,
                lunar_year: f_opt_int(b, "lunar_year")?
                    .map(|x| to_i32(x, "lunar_year"))
                    .transpose()?,
                lunar_year_approx: f_bool(b, "lunar_year_approx")?,
                can_chi: {
                    let c = field(b, "can_chi")?;
                    if c.is_null() {
                        None
                    } else {
                        Some(c.as_str().map_or(e("can_chi"), |s| Ok(s.to_string()))?)
                    }
                },
                lunar: {
                    let l = field(b, "lunar")?;
                    if l.is_null() {
                        None
                    } else {
                        Some(parse_lunar_day(l)?)
                    }
                },
                solar: parse_ymd_opt(field(b, "solar")?)?,
            })
        }
    };
    let death = {
        let d = field(v, "death")?;
        if d.is_null() {
            None
        } else {
            let t = field(d, "tuoi")?;
            Some(DeathInfo {
                input: parse_date_value(field(d, "input")?)?,
                lunar: parse_lunar_day(field(d, "lunar")?)?,
                solar: parse_ymd_opt(field(d, "solar")?)?,
                lunar_source: match f_str(d, "lunar_source")?.as_str() {
                    "entered" => LunarSource::Entered,
                    "converted" => LunarSource::Converted,
                    _ => return e("lunar_source"),
                },
                method: match f_str(d, "method")?.as_str() {
                    "precise" => lvhv_core::Method::Precise,
                    "hnd" => lvhv_core::Method::Hnd,
                    "jpl" => lvhv_core::Method::Jpl,
                    _ => return e("method"),
                },
                tuoi: Tuoi {
                    value: f_opt_int(t, "value")?
                        .map(|x| to_i32(x, "value"))
                        .transpose()?,
                    label: match f_str(t, "label")?.as_str() {
                        "huong_tho" => AgeLabel::HuongTho,
                        "huong_duong" => AgeLabel::HuongDuong,
                        _ => return e("label"),
                    },
                    manual: f_bool(t, "manual")?,
                    approx: f_bool(t, "approx")?,
                },
            })
        }
    };
    let (gio_het_nam, huong_duong_duoi) = {
        let m = field(v, "memorial")?;
        if m.is_null() {
            (DEFAULT_GIO_HET_NAM, DEFAULT_HUONG_DUONG_DUOI)
        } else {
            (
                to_u32(f_int(m, "gio_het_nam")?, "gio_het_nam")?,
                to_i32(f_int(m, "huong_duong_duoi")?, "huong_duong_duoi")?,
            )
        }
    };
    let r = field(v, "reminders")?;
    Ok(Person {
        id: f_str(v, "id")?,
        kind,
        danh_xung: f_str(v, "danh_xung")?,
        ho_ten: f_str(v, "ho_ten")?,
        birth,
        death,
        gio_het_nam,
        huong_duong_duoi,
        rule_gio: parse_rule(field(r, "gio")?)?,
        rule_sinh_nhat: parse_rule(field(r, "sinh_nhat")?)?,
        rule_sinh_nhat_am: parse_rule(field(r, "sinh_nhat_am")?)?,
        sinh_nhat_am: f_bool(v, "sinh_nhat_am")?,
        feb29_rule: match f_str(v, "feb29_rule")?.as_str() {
            "feb28" => Feb29Rule::Feb28,
            "mar1" => Feb29Rule::Mar1,
            _ => return e("feb29_rule"),
        },
        rev: f_int(v, "rev")?,
    })
}

fn parse_event(v: &Value) -> Result<Event, ModelError> {
    let pids = match field(v, "person_ids")?.as_arr() {
        Some(a) => a
            .iter()
            .map(|x| x.as_str().map_or(e("person_ids"), |s| Ok(s.to_string())))
            .collect::<Result<Vec<_>, _>>()?,
        None => return e("person_ids"),
    };
    Ok(Event {
        id: f_str(v, "id")?,
        type_id: f_str(v, "type_id")?,
        tieu_de: f_str(v, "tieu_de")?,
        date: parse_date_value(field(v, "date")?)?,
        recurring: f_bool(v, "recurring")?,
        person_ids: pids,
        reminder: parse_rule(field(v, "reminder")?)?,
        rev: f_int(v, "rev")?,
    })
}

/// Đọc tài liệu sao lưu lược đồ v1 (phần JSON, 01 §5) thành `Dataset`.
pub fn parse_dataset(text: &str) -> Result<Dataset, ModelError> {
    let v = crate::json::parse(text)
        .map_err(|p| ModelError(format!("JSON lỗi tại byte {}: {}", p.pos, p.msg)))?;
    let arr = |k: &str| -> Result<Vec<Value>, ModelError> {
        match field(&v, k)?.as_arr() {
            Some(a) => Ok(a.to_vec()),
            None => e(k),
        }
    };
    let settings = field(&v, "settings")?;
    Ok(Dataset {
        reminders_enabled: f_bool(settings, "reminders_enabled")?,
        persons: arr("persons")?
            .iter()
            .map(parse_person)
            .collect::<Result<_, _>>()?,
        events: arr("events")?
            .iter()
            .map(parse_event)
            .collect::<Result<_, _>>()?,
        event_types: arr("event_types")?
            .iter()
            .map(|t| {
                Ok(EventType {
                    id: f_str(t, "id")?,
                    builtin: {
                        let b = field(t, "builtin")?;
                        if b.is_null() {
                            None
                        } else {
                            Some(b.as_str().map_or(e("builtin"), |s| Ok(s.to_string()))?)
                        }
                    },
                    ten: f_str(t, "ten")?,
                })
            })
            .collect::<Result<_, ModelError>>()?,
    })
}
