//! Trường dẫn xuất (01 §1 "Trường dẫn xuất", §3.2, §3.3; 02 §1, §4) và kiểm tra V5.
//!
//! `save_death` / `derive_birth` ở đây là cài đặt của **lớp lưu trữ** (dùng cho V5 khi nhập sao lưu và cho R2:
//! "đi qua `save_death` thật của lớp lưu trữ"). Bản TypeScript tương ứng: `pwa/src/store/derive.ts`.
//! Khi crate thuật toán `lvhv-memorial` (đợt 1, vai B1) có `save_death`, hai bản phải cho kết quả giống
//! hệt — xem ghi chú tích hợp trong docs/v0.2/03 §6.
use crate::validate::{ValidationError, MAX_SAFE_INT};
use lvhv_core::canchi::{CAN, CHI};
use lvhv_core::{Method, DEFAULT_METHOD, MAX_LUNAR_YEAR, MAX_YEAR, MIN_LUNAR_YEAR, MIN_YEAR};
use serde_json::{json, Map, Value};

/// Năm âm nằm trong miền đổi lịch (01 §2.1: âm 1899–3000).
pub fn lunar_year_checked(y: i64) -> bool {
    (i64::from(MIN_LUNAR_YEAR)..=i64::from(MAX_LUNAR_YEAR)).contains(&y)
}

/// Năm dương nằm trong miền đổi lịch (1900–3000).
pub fn solar_year_checked(y: i64) -> bool {
    (i64::from(MIN_YEAR)..=i64::from(MAX_YEAR)).contains(&y)
}

fn u(x: i64) -> u32 {
    u32::try_from(x).unwrap_or(0)
}
fn i(x: i64) -> i32 {
    i32::try_from(x).unwrap_or(0)
}

/// `L2S` (02 ký hiệu) → `SolarDay` JSON, `None` nếu ngày âm không tồn tại hoặc ngoài miền.
pub fn l2s(method: Method, y: i64, m: i64, d: i64, leap: bool) -> Option<Value> {
    lvhv_core::lunar_to_solar_with(method, i(y), u(m), u(d), leap)
        .map(|s| json!({"y": s.year, "m": s.month, "d": s.day}))
}

/// `S2L` → `LunarDay` JSON.
pub fn s2l(method: Method, y: i64, m: i64, d: i64) -> Option<Value> {
    lvhv_core::solar_to_lunar_with(method, i(y), u(m), u(d))
        .ok()
        .map(|l| json!({"year": l.year, "month": l.month, "day": l.day, "leap": l.leap}))
}

/// Can Chi của năm âm (02 §4.1; hợp đồng kiến trúc §3).
pub fn can_chi_year(y: i64) -> String {
    let can = y.rem_euclid(10) as usize;
    let can = (can + 6) % 10;
    let chi = (y.rem_euclid(12) as usize + 8) % 12;
    format!("{} {}", CAN[can], CHI[chi])
}

fn method_name(m: Method) -> &'static str {
    match m {
        Method::Precise => "precise",
        Method::Hnd => "hnd",
        // Chưa ghi được vào bản sao lưu (lược đồ v1 chỉ nhận precise/hnd — store/src/validate.rs); chỉ đạt tới khi
        // đổi DEFAULT_METHOD sang Jpl, lúc đó phải mở rộng lược đồ (docs/03 §17).
        Method::Jpl => "jpl",
    }
}

fn gi(v: &Value) -> Option<i64> {
    v.as_i64().filter(|x| x.abs() <= MAX_SAFE_INT)
}

/// Lỗi nhập liệu của `save_death`/`derive_birth` (thông báo theo 01 §6.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputError(pub String);

/// 02 §1 `save_death(input)` → `{lunar, solar, lunar_source, method}` (chưa có `tuoi`).
///
/// `input` phải là `DateValue` đã qua kiểm tra lược đồ. Lỗi: ngày không tồn tại (01 §6.1), dương ngoài
/// 1900–3000, `precision = "year"`.
pub fn save_death(input: &Value) -> Result<Map<String, Value>, InputError> {
    save_death_with(DEFAULT_METHOD, input)
}

/// `save_death` theo phương pháp chỉ định (chỉ dùng để kiểm bản ghi cũ có `method` khác).
pub fn save_death_with(method: Method, input: &Value) -> Result<Map<String, Value>, InputError> {
    let lunar_cal = input["calendar"] == "lunar";
    let pr = input["precision"].as_str().unwrap_or_default();
    let (y, m, d, leap) = (
        gi(&input["year"]),
        gi(&input["month"]),
        gi(&input["day"]),
        input["leap"] == true,
    );
    let mut out = Map::new();
    if !lunar_cal {
        let (Some(y), Some(m), Some(d)) = (y, m, d) else {
            return Err(InputError(
                "Ngày mất theo dương lịch phải đủ ngày, tháng, năm".into(),
            ));
        };
        if pr != "day" {
            return Err(InputError(
                "Ngày mất theo dương lịch phải đủ ngày, tháng, năm".into(),
            ));
        }
        if !solar_year_checked(y) {
            return Err(InputError(format!(
                "Năm {y} ngoài khoảng {MIN_YEAR}–{MAX_YEAR}"
            )));
        }
        if d > crate::validate::days_in_month(y, m) {
            return Err(InputError(format!(
                "Tháng {m} năm {y} chỉ có {} ngày",
                crate::validate::days_in_month(y, m)
            )));
        }
        let l =
            s2l(method, y, m, d).ok_or_else(|| InputError("Không đổi được sang âm lịch".into()))?;
        out.insert("lunar".into(), l);
        out.insert("solar".into(), json!({"y": y, "m": m, "d": d}));
        out.insert("lunar_source".into(), json!("converted"));
    } else {
        let (Some(m), Some(d)) = (m, d) else {
            return Err(InputError("Ngày mất phải có ngày và tháng".into()));
        };
        match pr {
            "day" => {
                let Some(y) = y else {
                    return Err(InputError("Thiếu năm".into()));
                };
                let solar = if lunar_year_checked(y) {
                    match l2s(method, y, m, d, leap) {
                        Some(s) => s,
                        None => return Err(InputError(lunar_missing_msg(method, y, m, d, leap))),
                    }
                } else {
                    Value::Null
                };
                out.insert(
                    "lunar".into(),
                    json!({"year": y, "month": m, "day": d, "leap": leap}),
                );
                out.insert("solar".into(), solar);
            }
            "month_day" => {
                out.insert(
                    "lunar".into(),
                    json!({"year": null, "month": m, "day": d, "leap": leap}),
                );
                out.insert("solar".into(), Value::Null);
            }
            _ => {
                return Err(InputError(
                    "Ngày mất phải biết ít nhất ngày và tháng âm".into(),
                ))
            }
        }
        out.insert("lunar_source".into(), json!("entered"));
    }
    out.insert("method".into(), json!(method_name(method)));
    Ok(out)
}

/// Thông báo 01 §6.1 cho ngày âm không tồn tại.
fn lunar_missing_msg(method: Method, y: i64, m: i64, d: i64, leap: bool) -> String {
    if leap && lvhv_core::lunar_to_solar_with(method, i(y), u(m), 1, true).is_none() {
        return format!("Năm {y} không có tháng {m} nhuận");
    }
    if d == 30 && lvhv_core::lunar_to_solar_with(method, i(y), u(m), 29, leap).is_some() {
        return format!(
            "Tháng {m}{} âm lịch năm {y} chỉ có 29 ngày",
            if leap { " nhuận" } else { "" }
        );
    }
    format!("Ngày {d}/{m} âm lịch năm {y} không tồn tại")
}

/// 02 §4.1 + 01 §3.3: trường dẫn xuất của `BirthInfo` từ `input`.
///
/// Ngày dương đủ ngày nhưng ngoài miền đổi lịch (năm 1000–1899): `lunar_year = y`, `approx = true`,
/// `lunar = null`, `solar = input` (01 §2.1 "ngoài miền vẫn lưu được nhưng không đổi lịch"; đặc tả không
/// nói rõ — ghi ở 03 §6).
pub fn derive_birth(input: &Value) -> Result<Map<String, Value>, InputError> {
    let method = DEFAULT_METHOD;
    let lunar_cal = input["calendar"] == "lunar";
    let pr = input["precision"].as_str().unwrap_or_default();
    let (y, m, d, leap) = (
        gi(&input["year"]),
        gi(&input["month"]),
        gi(&input["day"]),
        input["leap"] == true,
    );
    let Some(y) = y else {
        return Err(InputError("Thiếu năm sinh".into()));
    };
    let mut out = Map::new();
    let (ly, approx, lunar, solar) = match (lunar_cal, pr) {
        (false, "day") => {
            let (Some(m), Some(d)) = (m, d) else {
                return Err(InputError("Thiếu ngày sinh".into()));
            };
            if d > crate::validate::days_in_month(y, m) {
                return Err(InputError(format!(
                    "Tháng {m} năm {y} chỉ có {} ngày",
                    crate::validate::days_in_month(y, m)
                )));
            }
            if solar_year_checked(y) {
                let ly = lvhv_core::lunar_year_of_birth_with(method, i(y), u(m), u(d))
                    .map_err(|_| InputError("Không đổi được sang âm lịch".into()))?;
                (
                    json!(ly),
                    false,
                    s2l(method, y, m, d).unwrap_or(Value::Null),
                    json!({"y": y, "m": m, "d": d}),
                )
            } else {
                (json!(y), true, Value::Null, json!({"y": y, "m": m, "d": d}))
            }
        }
        (false, "year") => (json!(y), true, Value::Null, Value::Null),
        (true, "day") => {
            let (Some(m), Some(d)) = (m, d) else {
                return Err(InputError("Thiếu ngày sinh".into()));
            };
            let solar = if lunar_year_checked(y) {
                l2s(method, y, m, d, leap)
                    .ok_or_else(|| InputError(lunar_missing_msg(method, y, m, d, leap)))?
            } else {
                Value::Null
            };
            (
                json!(y),
                false,
                json!({"year": y, "month": m, "day": d, "leap": leap}),
                solar,
            )
        }
        (true, "year") => (json!(y), false, Value::Null, Value::Null),
        _ => return Err(InputError("Ngày sinh phải đủ ngày hoặc chỉ năm".into())),
    };
    let cc = gi(&ly)
        .map(|v| json!(can_chi_year(v)))
        .unwrap_or(Value::Null);
    out.insert("lunar_year".into(), ly);
    out.insert("lunar_year_approx".into(), json!(approx));
    out.insert("can_chi".into(), cc);
    out.insert("lunar".into(), lunar);
    out.insert("solar".into(), solar);
    Ok(out)
}

/// 02 §4.2 `tuoi_mat` (chưa xét `manual`).
pub fn tuoi_mat(birth: &Value, death_lunar_year: Option<i64>, nguong: i64) -> Value {
    let by = if birth.is_null() {
        None
    } else {
        gi(&birth["lunar_year"])
    };
    let value = match (by, death_lunar_year) {
        (Some(b), Some(dy)) => Some(dy - b + 1),
        _ => None,
    };
    let approx = !birth.is_null() && birth["lunar_year_approx"] == true;
    let label = if value.is_some_and(|v| v < nguong) {
        "huong_duong"
    } else {
        "huong_tho"
    };
    json!({"value": value, "label": label, "manual": false, "approx": approx})
}

fn v5(path: &str, msg: impl Into<String>) -> ValidationError {
    ValidationError {
        code: "V5",
        path: path.to_owned(),
        msg: msg.into(),
    }
}
fn v4(path: &str, msg: impl Into<String>) -> ValidationError {
    ValidationError {
        code: "V4",
        path: path.to_owned(),
        msg: msg.into(),
    }
}

/// V5 cho một `Person` (đã qua lược đồ).
pub fn check_person_derived(o: &Map<String, Value>, path: &str) -> Result<(), ValidationError> {
    let birth = &o["birth"];
    if !birth.is_null() {
        let bp = format!("{path}/birth");
        let d = derive_birth(&birth["input"]).map_err(|e| v4(&format!("{bp}/input"), e.0))?;
        for (k, want) in &d {
            if &birth[k] != want {
                return Err(v5(
                    &format!("{bp}/{k}"),
                    format!("khác giá trị tính lại {want}"),
                ));
            }
        }
    }
    let death = &o["death"];
    if !death.is_null() {
        let dp = format!("{path}/death");
        // Luôn tính lại bằng phương pháp hiện hành. 01 §6.2 V5: khi `method` đã lưu khác phương pháp hiện
        // hành, giữ `lunar`/`solar`/`method` đã lưu (không so ba trường này).
        let d = save_death(&death["input"]).map_err(|e| v4(&format!("{dp}/input"), e.0))?;
        let strict = death["method"] == method_name(DEFAULT_METHOD);
        for (k, want) in &d {
            if !strict && (k == "lunar" || k == "solar" || k == "method") {
                continue;
            }
            if &death[k] != want {
                return Err(v5(
                    &format!("{dp}/{k}"),
                    format!("khác giá trị tính lại {want}"),
                ));
            }
        }
        let t = &death["tuoi"];
        let dy = gi(&death["lunar"]["year"]);
        if t["manual"] != true {
            let nguong = gi(&o["memorial"]["huong_duong_duoi"]).unwrap_or(60);
            let want = tuoi_mat(birth, dy, nguong);
            if want["value"].as_i64().is_some_and(|v| v < 1) {
                return Err(v4(&format!("{dp}/input"), "Ngày mất trước ngày sinh"));
            }
            if t != &want {
                return Err(v5(
                    &format!("{dp}/tuoi"),
                    format!("khác giá trị tính lại {want}"),
                ));
            }
        } else if let (Some(by), Some(dy)) = (
            if birth.is_null() {
                None
            } else {
                gi(&birth["lunar_year"])
            },
            dy,
        ) {
            if dy < by {
                return Err(v4(&format!("{dp}/input"), "Ngày mất trước ngày sinh"));
            }
        }
    }
    Ok(())
}
