//! Kiểm tra dữ liệu ở lớp lưu trữ (01 §6.2): V1 (lược đồ `backup-v1.schema.json`, cài đặt bằng mã) và
//! V2–V8. Cùng một bộ quy tắc được cài **lần thứ hai** trong `pwa/src/store/validate.ts`; hai bản phải
//! chấp nhận/từ chối giống nhau (kiểm chéo bằng tệp vector chung, R6(d)).
//!
//! Ghi chú về những điểm lược đồ JSON không nói rõ (quyết định của lớp lưu trữ, ghi trong 03 §6):
//! - Số nguyên: chấp nhận số JSON có phần thập phân bằng 0 (`1.0`) như số nguyên, giống JSON Schema và
//!   giống `JSON.parse`; giá trị tuyệt đối phải ≤ 2^53 − 1 để JavaScript đọc đúng (`rev` không có cận trên
//!   trong lược đồ — cận này là siết thêm duy nhất).
//! - `\S` của mẫu `ho_ten`/`ten`: theo định nghĩa `\s` của JavaScript (ECMA-262 WhiteSpace + LineTerminator).
//! - Độ sâu JSON ≤ 32 (03 BK-D1) kiểm trên văn bản **trước** khi phân tích ([`check_json_depth`]).
use crate::derive;
use lvhv_core::{MAX_YEAR, MIN_YEAR};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use unicode_normalization::UnicodeNormalization;

/// Lỗi kiểm tra: mã bất biến (`V1` … `V8`, hoặc `JSON`), đường dẫn trường, mô tả ngắn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// `V1` … `V8`, `JSON` (phân tích/độ sâu), `PHOTO` (byte ảnh không khớp siêu dữ liệu).
    pub code: &'static str,
    /// Đường dẫn kiểu JSON Pointer, ví dụ `/persons/0/death/lunar`.
    pub path: String,
    /// Mô tả (cho nhật ký kiểm thử; không hiện cho người dùng).
    pub msg: String,
}

impl core::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} {}: {}", self.code, self.path, self.msg)
    }
}

impl std::error::Error for ValidationError {}

type R = Result<(), ValidationError>;

fn err(code: &'static str, path: &str, msg: impl Into<String>) -> ValidationError {
    ValidationError {
        code,
        path: path.to_owned(),
        msg: msg.into(),
    }
}
fn v1(path: &str, msg: impl Into<String>) -> ValidationError {
    err("V1", path, msg)
}

/// Độ sâu tối đa của JSON trong tệp sao lưu.
pub const MAX_JSON_DEPTH: usize = 32;
/// Số nguyên an toàn lớn nhất của JavaScript.
pub const MAX_SAFE_INT: i64 = 9_007_199_254_740_991;

/// Kiểm tra độ sâu lồng `{`/`[` trên văn bản JSON, trước khi phân tích.
pub fn check_json_depth(text: &str) -> R {
    let mut depth = 0usize;
    let mut in_str = false;
    let mut esc = false;
    for &b in text.as_bytes() {
        if in_str {
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' | b'[' => {
                depth += 1;
                if depth > MAX_JSON_DEPTH {
                    return Err(err(
                        "JSON",
                        "",
                        format!("JSON lồng quá {MAX_JSON_DEPTH} cấp"),
                    ));
                }
            }
            b'}' | b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    Ok(())
}

/// Phân tích văn bản JSON của tài liệu sao lưu: độ sâu → `serde_json` → chuẩn hóa số nguyên.
pub fn parse_json(text: &str) -> Result<Value, ValidationError> {
    check_json_depth(text)?;
    let mut v: Value = serde_json::from_str(text).map_err(|e| err("JSON", "", e.to_string()))?;
    normalize_numbers(&mut v);
    Ok(v)
}

/// Số có phần thập phân bằng 0 và |x| ≤ 2^53 − 1 → số nguyên (như `JSON.parse`).
pub fn normalize_numbers(v: &mut Value) {
    match v {
        Value::Number(n) if n.as_i64().is_none() && n.as_u64().is_none() => {
            if let Some(f) = n.as_f64() {
                if f.fract() == 0.0 && f.abs() <= MAX_SAFE_INT as f64 {
                    *v = Value::from(f as i64);
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(normalize_numbers),
        Value::Object(o) => o.values_mut().for_each(normalize_numbers),
        _ => {}
    }
}

// ---------------------------------------------------------------------------------------------
// Kiểu nguyên thủy
// ---------------------------------------------------------------------------------------------

fn obj<'a>(
    v: &'a Value,
    path: &str,
    keys: &[&str],
) -> Result<&'a Map<String, Value>, ValidationError> {
    let o = v.as_object().ok_or_else(|| v1(path, "phải là đối tượng"))?;
    for k in keys {
        if !o.contains_key(*k) {
            return Err(v1(path, format!("thiếu trường \"{k}\"")));
        }
    }
    if let Some(k) = o.keys().find(|k| !keys.contains(&k.as_str())) {
        return Err(v1(path, format!("trường lạ \"{k}\"")));
    }
    Ok(o)
}

fn p(path: &str, k: &str) -> String {
    format!("{path}/{k}")
}

fn int(v: &Value) -> Option<i64> {
    v.as_i64().filter(|x| x.abs() <= MAX_SAFE_INT)
}

fn int_in(v: &Value, path: &str, lo: i64, hi: i64) -> Result<i64, ValidationError> {
    let x = int(v).ok_or_else(|| v1(path, "phải là số nguyên"))?;
    if x < lo || x > hi {
        return Err(v1(path, format!("ngoài khoảng {lo}–{hi}")));
    }
    Ok(x)
}

fn opt_int_in(v: &Value, path: &str, lo: i64, hi: i64) -> Result<Option<i64>, ValidationError> {
    if v.is_null() {
        Ok(None)
    } else {
        int_in(v, path, lo, hi).map(Some)
    }
}

fn boolean(v: &Value, path: &str) -> Result<bool, ValidationError> {
    v.as_bool().ok_or_else(|| v1(path, "phải là boolean"))
}

fn string<'a>(v: &'a Value, path: &str) -> Result<&'a str, ValidationError> {
    v.as_str().ok_or_else(|| v1(path, "phải là chuỗi"))
}

fn enum_str<'a>(v: &'a Value, path: &str, allowed: &[&str]) -> Result<&'a str, ValidationError> {
    let s = string(v, path)?;
    if allowed.contains(&s) {
        Ok(s)
    } else {
        Err(v1(path, format!("giá trị không hợp lệ \"{s}\"")))
    }
}

/// `\s` của JavaScript.
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

fn is_control(c: char, multiline: bool) -> bool {
    let u = c as u32;
    (u <= 0x1f && !(multiline && u == 0x0a)) || (0x7f..=0x9f).contains(&u)
}

/// `text` / `multiline` của lược đồ + `minLength`/`maxLength` (điểm mã) + tùy chọn `\S`.
fn text(v: &Value, path: &str, min: usize, max: usize, multiline: bool, need_nonspace: bool) -> R {
    let s = string(v, path)?;
    if s.chars().any(|c| is_control(c, multiline)) {
        return Err(v1(path, "chứa ký tự điều khiển"));
    }
    let n = s.chars().count();
    if n < min || n > max {
        return Err(v1(path, format!("độ dài {n} ngoài {min}–{max}")));
    }
    if need_nonspace && !s.chars().any(|c| !js_space(c)) {
        return Err(v1(path, "không được để trống"));
    }
    Ok(())
}

fn is_hex_lower(b: u8) -> bool {
    b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
}

/// Mẫu UUID v4 chữ thường (01 §1).
pub fn is_uuid_v4(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 36
        && b.iter().enumerate().all(|(i, &c)| match i {
            8 | 13 | 18 | 23 => c == b'-',
            14 => c == b'4',
            19 => matches!(c, b'8' | b'9' | b'a' | b'b'),
            _ => is_hex_lower(c),
        })
}

fn uuid(v: &Value, path: &str) -> Result<String, ValidationError> {
    let s = string(v, path)?;
    if is_uuid_v4(s) {
        Ok(s.to_owned())
    } else {
        Err(v1(path, "không phải UUID v4 chữ thường"))
    }
}

/// `^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$`.
pub fn is_timestamp(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 20
        && b.iter().enumerate().all(|(i, &c)| match i {
            4 | 7 => c == b'-',
            10 => c == b'T',
            13 | 16 => c == b':',
            19 => c == b'Z',
            _ => c.is_ascii_digit(),
        })
}

fn timestamp(v: &Value, path: &str) -> R {
    if is_timestamp(string(v, path)?) {
        Ok(())
    } else {
        Err(v1(path, "không phải thời điểm RFC 3339 UTC (…Z)"))
    }
}

fn hhmm(v: &Value, path: &str) -> R {
    let b = string(v, path)?.as_bytes();
    let ok = b.len() == 5
        && b[2] == b':'
        && ((b[0] == b'0' || b[0] == b'1') && b[1].is_ascii_digit()
            || b[0] == b'2' && (b'0'..=b'3').contains(&b[1]))
        && (b'0'..=b'5').contains(&b[3])
        && b[4].is_ascii_digit();
    if ok {
        Ok(())
    } else {
        Err(v1(path, "không phải giờ HH:MM"))
    }
}

fn rev(v: &Value, path: &str) -> R {
    int_in(v, path, 1, MAX_SAFE_INT).map(|_| ())
}

/// `offsets` của lược đồ + V8 (tăng dần nghiêm ngặt).
fn offsets(v: &Value, path: &str) -> R {
    let a = v.as_array().ok_or_else(|| v1(path, "phải là mảng"))?;
    if a.len() > 8 {
        return Err(v1(path, "quá 8 phần tử"));
    }
    let mut prev = -1i64;
    let mut seen = HashSet::new();
    for (i, x) in a.iter().enumerate() {
        let n = int_in(x, &p(path, &i.to_string()), 0, 60)?;
        if !seen.insert(n) {
            return Err(v1(path, "phần tử trùng"));
        }
        if n <= prev {
            return Err(err("V8", path, "offsets_days phải tăng dần"));
        }
        prev = n;
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Ngày
// ---------------------------------------------------------------------------------------------

/// Năm nhuận Gregory.
pub fn is_leap_year(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Số ngày của tháng dương.
pub fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(y) => 29,
        2 => 28,
        _ => 0,
    }
}

/// DateValue đã kiểm tra lược đồ (chỉ giữ phần bên gọi cần).
#[derive(Debug, Clone, Copy)]
struct Dv {
    precision: &'static str,
}

/// `DateValue` (lược đồ, allOf) + V4. `allowed`: các `precision` cho phép ở vị trí này.
fn date_value(v: &Value, path: &str, allowed: &[&'static str]) -> Result<Dv, ValidationError> {
    let o = obj(
        v,
        path,
        &["calendar", "precision", "year", "month", "day", "leap"],
    )?;
    let lunar = enum_str(&o["calendar"], &p(path, "calendar"), &["solar", "lunar"])? == "lunar";
    let pr = enum_str(
        &o["precision"],
        &p(path, "precision"),
        &["day", "month_day", "year"],
    )?;
    let precision: &'static str = match pr {
        "day" => "day",
        "month_day" => "month_day",
        _ => "year",
    };
    let year = opt_int_in(&o["year"], &p(path, "year"), 1000, 3000)?;
    let month = opt_int_in(&o["month"], &p(path, "month"), 1, 12)?;
    let day = opt_int_in(&o["day"], &p(path, "day"), 1, if lunar { 30 } else { 31 })?;
    let leap = boolean(&o["leap"], &p(path, "leap"))?;
    let shape_ok = match precision {
        "day" => year.is_some() && month.is_some() && day.is_some(),
        "month_day" => lunar && year.is_none() && month.is_some() && day.is_some(),
        _ => year.is_some() && month.is_none() && day.is_none() && !leap,
    };
    if !shape_ok {
        return Err(v1(
            path,
            format!("trường không khớp precision = {precision}"),
        ));
    }
    if !lunar && leap {
        return Err(v1(path, "dương lịch không có tháng nhuận"));
    }
    if !allowed.contains(&precision) {
        return Err(v1(
            path,
            format!("precision = {precision} không được phép ở đây"),
        ));
    }
    let dv = Dv { precision };
    // V4: ngày phải tồn tại.
    if precision == "day" {
        let (y, m, d) = (year.unwrap_or(0), month.unwrap_or(0), day.unwrap_or(0));
        if !lunar {
            if d > days_in_month(y, m) {
                return Err(err("V4", path, format!("ngày {d}/{m}/{y} không tồn tại")));
            }
        } else if derive::lunar_year_checked(y)
            && derive::l2s(lvhv_core::DEFAULT_METHOD, y, m, d, leap).is_none()
        {
            return Err(err(
                "V4",
                path,
                format!(
                    "ngày âm {d}/{m}{}/{y} không tồn tại",
                    if leap { " nhuận" } else { "" }
                ),
            ));
        }
    }
    Ok(dv)
}

fn lunar_day(v: &Value, path: &str) -> R {
    let o = obj(v, path, &["year", "month", "day", "leap"])?;
    opt_int_in(&o["year"], &p(path, "year"), 1000, 3000)?;
    int_in(&o["month"], &p(path, "month"), 1, 12)?;
    int_in(&o["day"], &p(path, "day"), 1, 30)?;
    boolean(&o["leap"], &p(path, "leap"))?;
    Ok(())
}

fn solar_day(v: &Value, path: &str) -> R {
    let o = obj(v, path, &["y", "m", "d"])?;
    int_in(&o["y"], &p(path, "y"), 1000, 3000)?;
    int_in(&o["m"], &p(path, "m"), 1, 12)?;
    int_in(&o["d"], &p(path, "d"), 1, 31)?;
    Ok(())
}

fn nullable(v: &Value, path: &str, f: impl Fn(&Value, &str) -> R) -> R {
    if v.is_null() {
        Ok(())
    } else {
        f(v, path)
    }
}

// ---------------------------------------------------------------------------------------------
// Thực thể
// ---------------------------------------------------------------------------------------------

/// `ReminderRule` (01 §3.6).
pub fn reminder_rule(v: &Value, path: &str) -> R {
    let o = obj(
        v,
        path,
        &["id", "enabled", "offsets_days", "time_local", "tien_thuong"],
    )?;
    uuid(&o["id"], &p(path, "id"))?;
    boolean(&o["enabled"], &p(path, "enabled"))?;
    offsets(&o["offsets_days"], &p(path, "offsets_days"))?;
    hhmm(&o["time_local"], &p(path, "time_local"))?;
    boolean(&o["tien_thuong"], &p(path, "tien_thuong"))?;
    Ok(())
}

fn birth_info(v: &Value, path: &str) -> R {
    let o = obj(
        v,
        path,
        &[
            "input",
            "noi_sinh",
            "lunar_year",
            "lunar_year_approx",
            "can_chi",
            "lunar",
            "solar",
        ],
    )?;
    date_value(&o["input"], &p(path, "input"), &["day", "year"])?;
    text(&o["noi_sinh"], &p(path, "noi_sinh"), 0, 300, false, false)?;
    opt_int_in(&o["lunar_year"], &p(path, "lunar_year"), 999, 3000)?;
    boolean(&o["lunar_year_approx"], &p(path, "lunar_year_approx"))?;
    if !o["can_chi"].is_null() {
        let s = string(&o["can_chi"], &p(path, "can_chi"))?;
        if s.chars().count() > 20 {
            return Err(v1(&p(path, "can_chi"), "quá 20 ký tự"));
        }
    }
    nullable(&o["lunar"], &p(path, "lunar"), lunar_day)?;
    nullable(&o["solar"], &p(path, "solar"), solar_day)?;
    Ok(())
}

fn death_info(v: &Value, path: &str) -> R {
    let o = obj(
        v,
        path,
        &["input", "lunar", "solar", "lunar_source", "method", "tuoi"],
    )?;
    date_value(&o["input"], &p(path, "input"), &["day", "month_day"])?;
    lunar_day(&o["lunar"], &p(path, "lunar"))?;
    nullable(&o["solar"], &p(path, "solar"), solar_day)?;
    enum_str(
        &o["lunar_source"],
        &p(path, "lunar_source"),
        &["entered", "converted"],
    )?;
    enum_str(&o["method"], &p(path, "method"), &["precise", "hnd", "jpl"])?;
    let tp = p(path, "tuoi");
    let t = obj(&o["tuoi"], &tp, &["value", "label", "manual", "approx"])?;
    opt_int_in(&t["value"], &p(&tp, "value"), 1, 150)?;
    enum_str(&t["label"], &p(&tp, "label"), &["huong_tho", "huong_duong"])?;
    boolean(&t["manual"], &p(&tp, "manual"))?;
    boolean(&t["approx"], &p(&tp, "approx"))?;
    Ok(())
}

fn memorial(v: &Value, path: &str) -> R {
    let o = obj(
        v,
        path,
        &[
            "tray_style",
            "incense",
            "bia_tieu_de",
            "loi_tuong_niem",
            "gio_het_nam",
            "huong_duong_duoi",
        ],
    )?;
    let style = enum_str(
        &o["tray_style"],
        &p(path, "tray_style"),
        &["phat_giao", "truyen_thong", "cong_giao"],
    )?;
    let incense = boolean(&o["incense"], &p(path, "incense"))?;
    text(
        &o["bia_tieu_de"],
        &p(path, "bia_tieu_de"),
        0,
        60,
        false,
        false,
    )?;
    text(
        &o["loi_tuong_niem"],
        &p(path, "loi_tuong_niem"),
        0,
        2000,
        true,
        false,
    )?;
    let g =
        int(&o["gio_het_nam"]).ok_or_else(|| v1(&p(path, "gio_het_nam"), "phải là số nguyên"))?;
    if g != 2 && g != 3 {
        return Err(v1(&p(path, "gio_het_nam"), "chỉ 2 hoặc 3"));
    }
    int_in(&o["huong_duong_duoi"], &p(path, "huong_duong_duoi"), 1, 150)?;
    if style == "cong_giao" && incense {
        return Err(err("V6", &p(path, "incense"), "Công giáo không có hương"));
    }
    Ok(())
}

const PERSON_KEYS: [&str; 18] = [
    "id",
    "kind",
    "danh_xung",
    "ho_ten",
    "ten_hieu",
    "quan_he",
    "birth",
    "death",
    "noi_an_tang",
    "photo_id",
    "memorial",
    "reminders",
    "sinh_nhat_am",
    "feb29_rule",
    "ghi_chu",
    "created_at",
    "updated_at",
    "rev",
];

/// Một `Person`: lược đồ (V1) + V2, V4, V5, V6, V8. Không kiểm tham chiếu (V3).
pub fn person(v: &Value, path: &str) -> R {
    let o = obj(v, path, &PERSON_KEYS)?;
    uuid(&o["id"], &p(path, "id"))?;
    let deceased = enum_str(&o["kind"], &p(path, "kind"), &["deceased", "living"])? == "deceased";
    text(&o["danh_xung"], &p(path, "danh_xung"), 0, 40, false, false)?;
    text(&o["ho_ten"], &p(path, "ho_ten"), 1, 120, false, true)?;
    text(&o["ten_hieu"], &p(path, "ten_hieu"), 0, 120, false, false)?;
    text(&o["quan_he"], &p(path, "quan_he"), 0, 120, false, false)?;
    nullable(&o["birth"], &p(path, "birth"), birth_info)?;
    nullable(&o["death"], &p(path, "death"), death_info)?;
    text(
        &o["noi_an_tang"],
        &p(path, "noi_an_tang"),
        0,
        300,
        false,
        false,
    )?;
    if !o["photo_id"].is_null() {
        uuid(&o["photo_id"], &p(path, "photo_id"))?;
    }
    nullable(&o["memorial"], &p(path, "memorial"), memorial)?;
    let rp = p(path, "reminders");
    let r = obj(&o["reminders"], &rp, &["gio", "sinh_nhat", "sinh_nhat_am"])?;
    for k in ["gio", "sinh_nhat", "sinh_nhat_am"] {
        nullable(&r[k], &p(&rp, k), reminder_rule)?;
    }
    let sn_am = boolean(&o["sinh_nhat_am"], &p(path, "sinh_nhat_am"))?;
    enum_str(&o["feb29_rule"], &p(path, "feb29_rule"), &["feb28", "mar1"])?;
    text(&o["ghi_chu"], &p(path, "ghi_chu"), 0, 2000, true, false)?;
    timestamp(&o["created_at"], &p(path, "created_at"))?;
    timestamp(&o["updated_at"], &p(path, "updated_at"))?;
    rev(&o["rev"], &p(path, "rev"))?;
    // Điều kiện theo `kind` của lược đồ (allOf) — trừ phần "death/memorial ≠ null" là V2.
    if deceased {
        if !r["sinh_nhat"].is_null() || !r["sinh_nhat_am"].is_null() || sn_am {
            return Err(v1(path, "người đã khuất chỉ có nhắc giỗ"));
        }
    } else {
        if o["noi_an_tang"].as_str() != Some("") || !r["gio"].is_null() {
            return Err(v1(path, "người đang sống không có nơi an táng / nhắc giỗ"));
        }
    }
    // V2
    if deceased != !o["death"].is_null() || deceased != !o["memorial"].is_null() {
        return Err(err(
            "V2",
            path,
            "kind = deceased ⇔ death ≠ null ⇔ memorial ≠ null",
        ));
    }
    // V5
    derive::check_person_derived(o, path)
}

/// `EventType` (lược đồ + ràng buộc id cố định của loại có sẵn).
pub fn event_type(v: &Value, path: &str) -> R {
    let o = obj(
        v,
        path,
        &[
            "id",
            "builtin",
            "ten",
            "default_recurring",
            "created_at",
            "updated_at",
            "rev",
        ],
    )?;
    let id = uuid(&o["id"], &p(path, "id"))?;
    if !o["builtin"].is_null() {
        let b = enum_str(
            &o["builtin"],
            &p(path, "builtin"),
            &["tot_nghiep", "ngay_cuoi", "dinh_hon", "du_hoc"],
        )?;
        let fixed = crate::model::BUILTIN_EVENT_TYPES
            .iter()
            .find(|t| t.0 == b)
            .map(|t| t.1);
        if fixed != Some(id.as_str()) {
            return Err(v1(
                &p(path, "id"),
                "loại có sẵn phải dùng id cố định (01 §3.5)",
            ));
        }
    }
    text(&o["ten"], &p(path, "ten"), 1, 60, false, true)?;
    boolean(&o["default_recurring"], &p(path, "default_recurring"))?;
    timestamp(&o["created_at"], &p(path, "created_at"))?;
    timestamp(&o["updated_at"], &p(path, "updated_at"))?;
    rev(&o["rev"], &p(path, "rev"))
}

/// `Event` (lược đồ + V4, V8). Không kiểm tham chiếu (V3).
pub fn event(v: &Value, path: &str) -> R {
    let o = obj(
        v,
        path,
        &[
            "id",
            "type_id",
            "tieu_de",
            "date",
            "recurring",
            "person_ids",
            "ghi_chu",
            "reminder",
            "created_at",
            "updated_at",
            "rev",
        ],
    )?;
    uuid(&o["id"], &p(path, "id"))?;
    uuid(&o["type_id"], &p(path, "type_id"))?;
    text(&o["tieu_de"], &p(path, "tieu_de"), 0, 120, false, false)?;
    let d = date_value(&o["date"], &p(path, "date"), &["day", "month_day"])?;
    let recurring = boolean(&o["recurring"], &p(path, "recurring"))?;
    if d.precision == "month_day" && !recurring {
        return Err(v1(
            &p(path, "recurring"),
            "sự kiện không rõ năm phải lặp hằng năm",
        ));
    }
    let pp = p(path, "person_ids");
    let ids = o["person_ids"]
        .as_array()
        .ok_or_else(|| v1(&pp, "phải là mảng"))?;
    if ids.len() > 50 {
        return Err(v1(&pp, "quá 50 người"));
    }
    let mut seen = HashSet::new();
    for (i, x) in ids.iter().enumerate() {
        if !seen.insert(uuid(x, &p(&pp, &i.to_string()))?) {
            return Err(v1(&pp, "id trùng"));
        }
    }
    text(&o["ghi_chu"], &p(path, "ghi_chu"), 0, 2000, true, false)?;
    nullable(&o["reminder"], &p(path, "reminder"), reminder_rule)?;
    timestamp(&o["created_at"], &p(path, "created_at"))?;
    timestamp(&o["updated_at"], &p(path, "updated_at"))?;
    rev(&o["rev"], &p(path, "rev"))
}

/// `Photo` (siêu dữ liệu).
pub fn photo(v: &Value, path: &str) -> R {
    let o = obj(
        v,
        path,
        &[
            "id",
            "mime",
            "width",
            "height",
            "bytes",
            "sha256",
            "stripped",
            "created_at",
        ],
    )?;
    uuid(&o["id"], &p(path, "id"))?;
    enum_str(&o["mime"], &p(path, "mime"), &["image/jpeg", "image/webp"])?;
    int_in(&o["width"], &p(path, "width"), 1, 2048)?;
    int_in(&o["height"], &p(path, "height"), 1, 2048)?;
    int_in(&o["bytes"], &p(path, "bytes"), 1, 1_048_576)?;
    let h = string(&o["sha256"], &p(path, "sha256"))?;
    if h.len() != 64 || !h.bytes().all(is_hex_lower) {
        return Err(v1(&p(path, "sha256"), "phải là 64 ký tự hex thường"));
    }
    if o["stripped"] != Value::Bool(true) {
        return Err(v1(&p(path, "stripped"), "phải là true"));
    }
    timestamp(&o["created_at"], &p(path, "created_at"))
}

/// `AppSettings`.
pub fn settings(v: &Value, path: &str) -> R {
    let o = obj(
        v,
        path,
        &[
            "reminders_enabled",
            "default_offsets_days",
            "default_time_local",
            "upcoming_days",
            "last_backup_at",
            "backup_nag_days",
        ],
    )?;
    boolean(&o["reminders_enabled"], &p(path, "reminders_enabled"))?;
    offsets(&o["default_offsets_days"], &p(path, "default_offsets_days"))?;
    hhmm(&o["default_time_local"], &p(path, "default_time_local"))?;
    if int(&o["upcoming_days"]) != Some(30) {
        return Err(v1(&p(path, "upcoming_days"), "phải là 30"));
    }
    nullable(&o["last_backup_at"], &p(path, "last_backup_at"), timestamp)?;
    let n = int(&o["backup_nag_days"]);
    if !matches!(n, Some(0 | 30 | 60 | 90)) {
        return Err(v1(&p(path, "backup_nag_days"), "chỉ 0, 30, 60, 90"));
    }
    Ok(())
}

/// Khóa so sánh tên loại sự kiện (01 §3.5): NFC rồi chữ thường Unicode.
pub fn name_key(ten: &str) -> String {
    ten.nfc().collect::<String>().to_lowercase()
}

/// Toàn bộ tài liệu sao lưu: V1–V8.
pub fn document(v: &Value) -> R {
    let root = obj(
        v,
        "",
        &[
            "format",
            "schema_version",
            "app",
            "exported_at",
            "settings",
            "event_types",
            "persons",
            "events",
            "photos",
        ],
    )?;
    if root["format"].as_str() != Some("lvhv-backup") {
        return Err(v1("/format", "phải là \"lvhv-backup\""));
    }
    if int(&root["schema_version"]) != Some(1) {
        return Err(v1("/schema_version", "phải là 1"));
    }
    let app = obj(&root["app"], "/app", &["name", "version", "platform"])?;
    if app["name"].as_str() != Some("Lịch và Văn hóa Việt") {
        return Err(v1("/app/name", "tên ứng dụng sai"));
    }
    let ver = string(&app["version"], "/app/version")?;
    let parts: Vec<&str> = ver.split('.').collect();
    if parts.len() != 3
        || parts
            .iter()
            .any(|x| x.is_empty() || !x.bytes().all(|c| c.is_ascii_digit()))
    {
        return Err(v1("/app/version", "không đúng dạng X.Y.Z"));
    }
    enum_str(
        &app["platform"],
        "/app/platform",
        &["pwa", "android", "ios", "macos", "windows"],
    )?;
    timestamp(&root["exported_at"], "/exported_at")?;
    settings(&root["settings"], "/settings")?;

    let arr = |k: &str, max: usize| -> Result<&Vec<Value>, ValidationError> {
        let a = root[k]
            .as_array()
            .ok_or_else(|| v1(&format!("/{k}"), "phải là mảng"))?;
        if a.len() > max {
            return Err(v1(&format!("/{k}"), format!("quá {max} phần tử")));
        }
        Ok(a)
    };
    let types = arr("event_types", 500)?;
    let persons = arr("persons", 10_000)?;
    let events = arr("events", 50_000)?;
    let photos = arr("photos", 10_000)?;

    let ids = |a: &Vec<Value>,
               k: &str,
               f: fn(&Value, &str) -> R|
     -> Result<HashSet<String>, ValidationError> {
        let mut s = HashSet::new();
        for (i, x) in a.iter().enumerate() {
            let path = format!("/{k}/{i}");
            f(x, &path)?;
            let id = x["id"].as_str().unwrap_or_default().to_owned();
            if !s.insert(id) {
                return Err(v1(&p(&path, "id"), "id trùng"));
            }
        }
        Ok(s)
    };
    let photo_ids = ids(photos, "photos", photo)?;
    let type_ids = ids(types, "event_types", event_type)?;
    let person_ids = ids(persons, "persons", person)?;
    ids(events, "events", event)?;

    // V7
    let mut names: HashMap<String, usize> = HashMap::new();
    let mut builtins = HashSet::new();
    for (i, t) in types.iter().enumerate() {
        if names
            .insert(name_key(t["ten"].as_str().unwrap_or_default()), i)
            .is_some()
        {
            return Err(err(
                "V7",
                &format!("/event_types/{i}/ten"),
                "tên loại sự kiện trùng",
            ));
        }
        if let Some(b) = t["builtin"].as_str() {
            if !builtins.insert(b.to_owned()) {
                return Err(v1(
                    &format!("/event_types/{i}/builtin"),
                    "loại có sẵn trùng",
                ));
            }
        }
    }
    // V3
    for (i, x) in persons.iter().enumerate() {
        if let Some(pid) = x["photo_id"].as_str() {
            if !photo_ids.contains(pid) {
                return Err(err(
                    "V3",
                    &format!("/persons/{i}/photo_id"),
                    "ảnh không tồn tại",
                ));
            }
        }
    }
    for (i, e) in events.iter().enumerate() {
        if !type_ids.contains(e["type_id"].as_str().unwrap_or_default()) {
            return Err(err(
                "V3",
                &format!("/events/{i}/type_id"),
                "loại sự kiện không tồn tại",
            ));
        }
        for (j, pid) in e["person_ids"].as_array().into_iter().flatten().enumerate() {
            if !person_ids.contains(pid.as_str().unwrap_or_default()) {
                return Err(err(
                    "V3",
                    &format!("/events/{i}/person_ids/{j}"),
                    "người không tồn tại",
                ));
            }
        }
    }
    Ok(())
}

/// Năm dương trong miền đổi lịch của lõi.
pub fn solar_year_in_domain(y: i64) -> bool {
    (i64::from(MIN_YEAR)..=i64::from(MAX_YEAR)).contains(&y)
}
