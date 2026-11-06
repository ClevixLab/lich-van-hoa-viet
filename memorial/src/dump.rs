//! Định dạng dòng của các tệp dump kiểm định (docs/v0.2/02 Phụ lục A). Bản TypeScript
//! (`tsport/src/memorial_dump.ts`) sinh cùng từng byte.

use crate::cal::Cal;
use crate::gio::{
    anniversary, can_chi_year, derive_birth, save_death_with, solar_occurrence, tuoi_mat,
};
use crate::model::*;
use crate::occ::{upcoming, OccClass, Occurrence, RemType};
use crate::remind::{fmt_local, local_minutes, reminders_for, simulate, Reminder, Zone};
use std::fmt::Write as _;

/// Một dòng R1 (04 R1): `{"death":DateValue,"Y":…,"lunar":{y,m,d},"solar":{y,m,d},"n":…,"label":…}`.
pub fn gio_line(out: &mut String, death: &DateValue, a: &crate::gio::Anniversary) {
    out.push_str("{\"death\":");
    death.to_json(out);
    let _ = write!(
        out,
        ",\"Y\":{},\"lunar\":{{\"y\":{},\"m\":{},\"d\":{}}},\"solar\":{{\"y\":{},\"m\":{},\"d\":{}}},\"n\":{},\"label\":\"{}\"}}",
        a.year,
        a.year,
        a.month,
        a.day,
        a.solar.y,
        a.solar.m,
        a.solar.d,
        a.n.map_or("null".to_string(), |n| n.to_string()),
        a.label.code()
    );
    out.push('\n');
}

/// R1 cho một ngày mất: `years` năm giỗ kế tiếp (Y = năm âm mất + 1 … + years) hoặc, khi không rõ
/// năm, Y ∈ `[y_from, y_to]`. Bỏ năm có ngày dương ngoài miền. Lỗi nhập liệu → không dòng nào.
pub fn gio_lines_for(
    cal: &Cal,
    out: &mut String,
    death: &DateValue,
    years: i32,
    unknown_range: (i32, i32),
    gio_het_nam: u32,
) {
    let Ok(d) = save_death_with(cal, death, None, DEFAULT_HUONG_DUONG_DUOI) else {
        return;
    };
    let (a, b) = match d.lunar.year {
        Some(y) => (y + 1, y + years),
        None => unknown_range,
    };
    for y in a..=b {
        if let Some(x) = anniversary(
            cal,
            d.lunar.year,
            d.lunar.month,
            d.lunar.day,
            gio_het_nam,
            y,
        ) {
            gio_line(out, death, &x);
        }
    }
}

/// Mọi ngày âm tồn tại trong năm âm `ly` (theo thứ tự thời gian): (tháng, ngày, nhuận).
pub fn lunar_days_of_year(cal: &Cal, ly: i32) -> Vec<(u32, u32, bool)> {
    let mut v = Vec::new();
    for m in 1..=12u32 {
        for leap in [false, true] {
            if let Some(len) = cal.month_len(ly, m, leap) {
                for d in 1..=len {
                    v.push((m, d, leap));
                }
            }
        }
    }
    // Thứ tự lịch: tháng m thường trước tháng m nhuận.
    v
}

// ---------------------------------------------------------------- R3

/// Một trạng thái giao diện R3: tập con nút có sẵn (bitmask trên `PRESET_OFFSETS`) và mốc tự đặt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiState {
    pub mask: u32,
    pub custom: Option<u32>,
}

impl UiState {
    /// `offsets_days` sau khi gộp (tăng dần, không trùng).
    pub fn offsets(&self) -> Vec<u32> {
        let mut v: Vec<u32> = PRESET_OFFSETS
            .iter()
            .enumerate()
            .filter(|(i, _)| self.mask & (1 << i) != 0)
            .map(|(_, &k)| k)
            .collect();
        if let Some(c) = self.custom {
            if !v.contains(&c) {
                v.push(c);
            }
        }
        v.sort_unstable();
        v
    }
    pub fn presets(&self) -> Vec<u32> {
        PRESET_OFFSETS
            .iter()
            .enumerate()
            .filter(|(i, _)| self.mask & (1 << i) != 0)
            .map(|(_, &k)| k)
            .collect()
    }
}

/// 3 904 trạng thái theo thứ tự: mask 0…63, trong mỗi mask custom = không có, 1, …, 60.
pub fn ui_states() -> Vec<UiState> {
    let mut v = Vec::with_capacity(64 * 61);
    for mask in 0..64u32 {
        v.push(UiState { mask, custom: None });
        for c in CUSTOM_MIN..=CUSTOM_MAX {
            v.push(UiState {
                mask,
                custom: Some(c),
            });
        }
    }
    v
}

pub const R3_TIMES: [u32; 3] = [0, 7 * 60, 23 * 60 + 59];

/// Một cấu hình R3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct R3Config {
    pub index: usize,
    pub state: UiState,
    pub tien_thuong: bool,
    pub time_local: u32,
    pub class: OccClass,
}

/// 140 544 cấu hình theo thứ tự: trạng thái × tiên thường (tắt, bật) × giờ × loại.
pub fn r3_configs() -> Vec<R3Config> {
    let mut v = Vec::with_capacity(140_544);
    for st in ui_states() {
        for tt in [false, true] {
            for &t in &R3_TIMES {
                for c in OccClass::R3 {
                    v.push(R3Config {
                        index: v.len(),
                        state: st,
                        tien_thuong: tt,
                        time_local: t,
                        class: c,
                    });
                }
            }
        }
    }
    v
}

fn join_u32(v: &[u32], sep: &str) -> String {
    v.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(sep)
}

/// Dòng tiêu đề và một dòng của `r3-configs.csv`.
pub const R3_CONFIGS_HEADER: &str = "cfg,presets,custom,offsets,tien_thuong,time_local,class\n";
pub fn r3_config_line(out: &mut String, c: &R3Config) {
    let _ = writeln!(
        out,
        "{},{},{},{},{},{},{}",
        c.index,
        join_u32(&c.state.presets(), " "),
        c.state.custom.map_or(String::new(), |x| x.to_string()),
        join_u32(&c.state.offsets(), " "),
        u8::from(c.tien_thuong),
        fmt_hhmm(c.time_local),
        c.class.code()
    );
}

/// Kiểu của lời nhắc trong dump: `k=<số>` hoặc `tt`.
pub fn rtype_code(t: RemType) -> String {
    match t {
        RemType::Offset(k) => format!("k={k}"),
        RemType::TienThuong => "tt".to_string(),
    }
}

pub const R3_REMINDERS_HEADER: &str = "cfg,id,at_local,type,occ_date\n";
pub fn reminder_line(out: &mut String, cfg: usize, r: &Reminder) {
    let _ = writeln!(
        out,
        "{cfg},{},{},{},{}",
        r.id,
        fmt_local(r.at_local),
        rtype_code(r.rtype),
        r.occ.date.iso()
    );
}

/// Lần xuất hiện của lớp `class` trong `[from, to]` (đã sắp theo 02 §7.2).
pub fn class_occurrences(
    cal: &Cal,
    ds: &Dataset,
    class: OccClass,
    from: Ymd,
    to: Ymd,
) -> Vec<Occurrence> {
    crate::occ::occurrences(cal, ds, from, to)
        .into_iter()
        .filter(|o| o.class == class)
        .collect()
}

/// Chế độ phụ của dump R3 (04 Bổ sung B3 mục 2, 02 Phụ lục A.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R3Mode {
    /// `reminders_enabled = true`, `rule.enabled = true` (mặc định).
    Normal,
    /// Công tắc nhắc chung tắt: `reminders_enabled = false` → phải ra 0 lời nhắc.
    GlobalOff,
    /// `rule.enabled = false` ở mọi quy tắc → phải ra 0 lời nhắc.
    RuleOff,
}

/// Lời nhắc của một cấu hình R3 trên các lần xuất hiện cho trước (cùng lớp), quy tắc của mỗi lần
/// là quy tắc của bản ghi với `offsets_days/tien_thuong/time_local` thay bằng cấu hình (id giữ
/// nguyên), `enabled = true` và `reminders_enabled = true` (chế độ `Normal`). Sắp theo (at, id).
pub fn r3_reminders(
    ds: &Dataset,
    occs: &[Occurrence],
    c: &R3Config,
    now: i64,
    tz: &Zone,
) -> Vec<Reminder> {
    r3_reminders_mode(ds, occs, c, now, tz, R3Mode::Normal)
}

/// Như [`r3_reminders`] với chế độ `mode`.
pub fn r3_reminders_mode(
    ds: &Dataset,
    occs: &[Occurrence],
    c: &R3Config,
    now: i64,
    tz: &Zone,
    mode: R3Mode,
) -> Vec<Reminder> {
    let offs = c.state.offsets();
    let mut out = Vec::new();
    for o in occs {
        let Some(base) = crate::occ::rule_of(ds, o) else {
            continue;
        };
        let rule = ReminderRule {
            id: base.id.clone(),
            enabled: mode != R3Mode::RuleOff,
            offsets_days: offs.clone(),
            time_local: c.time_local,
            tien_thuong: c.tien_thuong,
        };
        out.extend(reminders_for(mode != R3Mode::GlobalOff, &rule, o, now, tz));
    }
    crate::remind::sort_reminders(&mut out);
    out
}

// ---------------------------------------------------------------- R11

pub const R11_HEADER: &str = "today,rank,days,date,class,entity_id,key,n,label\n";
pub fn upcoming_lines(cal: &Cal, ds: &Dataset, today: Ymd, out: &mut String) {
    for (i, it) in upcoming(cal, ds, today, 30).iter().enumerate() {
        let o = &it.occ;
        let _ = writeln!(
            out,
            "{},{},{},{},{},{},{},{},{}",
            today.iso(),
            i,
            it.days,
            o.date.iso(),
            o.class.code(),
            o.entity_id,
            o.key.text(),
            o.n.map_or(String::new(), |n| n.to_string()),
            o.gio_label.map_or("", |l| l.code())
        );
    }
}

// ---------------------------------------------------------------- R10

pub const R10A_HEADER: &str = "birth,lunar_year,can_chi\n";
pub fn r10a_line(cal: &Cal, out: &mut String, d: Ymd) {
    if let Ok(b) = derive_birth(cal, &DateValue::solar(d.y, d.m, d.d)) {
        let _ = writeln!(
            out,
            "{},{},{}",
            d.iso(),
            b.lunar_year.map_or(String::new(), |y| y.to_string()),
            b.can_chi.unwrap_or_default()
        );
    }
}

pub const R10B_HEADER: &str = "lunar_year,can_chi\n";
pub fn r10b_line(out: &mut String, y: i32) {
    let _ = writeln!(out, "{y},{}", can_chi_year(y));
}

pub const R10D_HEADER: &str = "year,rule,date\n";
pub fn r10d_line(out: &mut String, y: i32, rule: Feb29Rule) {
    // Gốc 29/02/1896 (năm nhuận trước 1900) để mọi Y 1900–3000 ≥ Y0.
    let o = solar_occurrence(Ymd::new(1896, 2, 29), y, rule);
    let _ = writeln!(
        out,
        "{y},{},{}",
        match rule {
            Feb29Rule::Feb28 => "feb28",
            Feb29Rule::Mar1 => "mar1",
        },
        o.map_or(String::new(), |d| d.iso())
    );
}

/// Kiểu nhập R10(c).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InKind {
    SolarDay,
    SolarYear,
    LunarDay,
    LunarYear,
}

impl InKind {
    pub const ALL: [InKind; 4] = [
        InKind::SolarDay,
        InKind::SolarYear,
        InKind::LunarDay,
        InKind::LunarYear,
    ];
    pub fn code(&self) -> &'static str {
        match self {
            InKind::SolarDay => "solar_day",
            InKind::SolarYear => "solar_year",
            InKind::LunarDay => "lunar_day",
            InKind::LunarYear => "lunar_year",
        }
    }
}

/// Sinh ngày ngẫu nhiên của kiểu `k` trong năm [y0, y1] (mulberry32). Ngày âm luôn tồn tại.
fn rand_date(
    cal: &Cal,
    rng: &mut crate::remind::Mulberry32,
    k: InKind,
    y0: i32,
    y1: i32,
) -> DateValue {
    let span = (y1 - y0 + 1) as u32;
    let y = y0 + (rng.next_u32() % span) as i32;
    match k {
        InKind::SolarDay => {
            let m = 1 + rng.next_u32() % 12;
            let d = 1 + rng.next_u32() % lvhv_core::jd::days_in_month(y, m);
            DateValue::solar(y, m, d)
        }
        InKind::SolarYear => DateValue::year_only(Calendar::Solar, y),
        InKind::LunarYear => DateValue::year_only(Calendar::Lunar, y),
        InKind::LunarDay => {
            let days = lunar_days_of_year(cal, y);
            let (m, d, leap) = days[(rng.next_u32() as usize) % days.len()];
            DateValue::lunar(Some(y), m, d, leap)
        }
    }
}

pub const R10C_HEADER: &str =
    "i,birth,death,nguong,birth_lunar_year,approx,death_lunar_year,value,label,error\n";

/// R10(c): `count` cặp (sinh, mất) từ `Mulberry32(seed)`: kiểu sinh và kiểu mất xoay vòng qua 4×2
/// tổ hợp (mất: dương đủ ngày / âm đủ ngày), ngưỡng xoay {59, 60, 61}; năm sinh 1900–2000, năm mất
/// = năm sinh + 0…109 (kẹp ≤ 2999).
pub fn r10c_lines(cal: &Cal, out: &mut String, count: usize, seed: u32) {
    let mut rng = crate::remind::Mulberry32(seed);
    for i in 0..count {
        let bk = InKind::ALL[i % 4];
        let dk = if (i / 4) % 2 == 0 {
            InKind::SolarDay
        } else {
            InKind::LunarDay
        };
        let nguong = [59, 60, 61][i % 3];
        let b = rand_date(cal, &mut rng, bk, 1900, 2000);
        let by = b.year.unwrap_or(1900);
        let dy0 = (by + (rng.next_u32() % 110) as i32).min(2999);
        let d = rand_date(cal, &mut rng, dk, dy0, dy0);
        let mut bs = String::new();
        b.to_json(&mut bs);
        let mut dsx = String::new();
        d.to_json(&mut dsx);
        let q = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));
        let birth = derive_birth(cal, &b);
        let death = crate::gio::convert_death(cal, &d);
        let res = match (&birth, &death) {
            (Ok(bi), Ok((dl, _, _))) => {
                if bi.lunar_year.zip(dl.year).is_some_and(|(x, y)| y < x) {
                    Err("DeathBeforeBirth")
                } else {
                    tuoi_mat(Some(bi), dl.year, nguong).map_err(|e| e.code)
                }
            }
            (Err(e), _) | (_, Err(e)) => Err(e.code),
        };
        let bly = birth.as_ref().ok().and_then(|b| b.lunar_year);
        let approx = birth.as_ref().is_ok_and(|b| b.lunar_year_approx);
        let dly = death.as_ref().ok().and_then(|d| d.0.year);
        let _ = match res {
            Ok(t) => writeln!(
                out,
                "{i},{},{},{nguong},{},{},{},{},{},",
                q(&bs),
                q(&dsx),
                bly.map_or(String::new(), |y| y.to_string()),
                u8::from(approx),
                dly.map_or(String::new(), |y| y.to_string()),
                t.value.map_or(String::new(), |v| v.to_string()),
                t.label.code()
            ),
            Err(code) => writeln!(
                out,
                "{i},{},{},{nguong},{},{},{},,,{code}",
                q(&bs),
                q(&dsx),
                bly.map_or(String::new(), |y| y.to_string()),
                u8::from(approx),
                dly.map_or(String::new(), |y| y.to_string()),
            ),
        };
    }
}

// ---------------------------------------------------------------- R12

/// Tập N hồ sơ/sự kiện từ bộ R3: bản ghi thứ i là bản ghi thứ `i mod len` của bộ gốc (người trước,
/// rồi sự kiện), id và id quy tắc được thay bằng `<8 hex của i>-<phần còn lại của id gốc>`.
pub fn scaled_dataset(base: &Dataset, n: usize) -> Dataset {
    let total = base.persons.len() + base.events.len();
    let mut ds = Dataset {
        reminders_enabled: base.reminders_enabled,
        persons: Vec::new(),
        events: Vec::new(),
        event_types: base.event_types.clone(),
    };
    if total == 0 {
        return ds;
    }
    let re = |id: &str, i: usize| format!("{i:08x}{}", &id[8.min(id.len())..]);
    let rr = |r: &Option<ReminderRule>, i: usize| {
        r.as_ref().map(|r| ReminderRule {
            id: re(&r.id, i),
            ..r.clone()
        })
    };
    for i in 0..n {
        let j = i % total;
        if j < base.persons.len() {
            let mut p = base.persons[j].clone();
            p.id = re(&p.id, i);
            p.rule_gio = rr(&p.rule_gio, i);
            p.rule_sinh_nhat = rr(&p.rule_sinh_nhat, i);
            p.rule_sinh_nhat_am = rr(&p.rule_sinh_nhat_am, i);
            ds.persons.push(p);
        } else {
            let mut e = base.events[j - base.persons.len()].clone();
            e.id = re(&e.id, i);
            e.reminder = rr(&e.reminder, i);
            e.person_ids.clear();
            ds.events.push(e);
        }
    }
    ds
}

pub const R12_HEADER: &str = "size,scenario,opens,max_pending,delivered,sentinel_delivered,violations,missed,missed_without_sentinel,i1,i2,i3,i4\n";

/// Ba kịch bản R12: xác suất mở ứng dụng mỗi ngày 1, 0,1, 0.
pub const R12_SCENARIOS: [(&str, u32, u32); 3] = [("p1", 1, 1), ("p0.1", 1, 10), ("p0", 0, 1)];

/// R12: một dòng tổng hợp mỗi (cỡ, kịch bản); `steps` (nếu có) nhận nhật ký từng lần mở.
pub fn r12_lines(
    cal: &Cal,
    base: &Dataset,
    sizes: &[usize],
    out: &mut String,
    mut steps: Option<&mut String>,
) {
    let tz = Zone::vietnam();
    for &n in sizes {
        let ds = scaled_dataset(base, n);
        for (name, pn, pd) in R12_SCENARIOS {
            let r = simulate(
                cal,
                &ds,
                &tz,
                Ymd::new(2026, 1, 1),
                Ymd::new(2028, 12, 31),
                9 * 60,
                pn,
                pd,
                20_261_106,
                false,
            );
            let _ = writeln!(
                out,
                "{n},{name},{},{},{},{},{},{},{},{},{},{},{}",
                r.opens,
                r.max_pending,
                r.delivered,
                r.sentinel_delivered,
                r.violations,
                r.missed,
                r.missed_without_sentinel,
                r.by_inv[0],
                r.by_inv[1],
                r.by_inv[2],
                r.by_inv[3]
            );
            if let Some(s) = steps.as_deref_mut() {
                for st in &r.steps {
                    let _ = writeln!(
                        s,
                        "{n},{name},{},{},{},{},{},{}",
                        st.day.iso(),
                        st.future,
                        st.pending,
                        u8::from(st.sentinel),
                        st.dropped,
                        st.violations.join(" ")
                    );
                }
            }
        }
    }
}

pub const R12_STEPS_HEADER: &str = "size,scenario,day,future,pending,sentinel,dropped,violations\n";

/// `now` của R3/R11: 2026-01-01T00:00 giờ thiết bị.
pub fn r3_now(tz: &Zone) -> i64 {
    tz.local_to_utc(local_minutes(Ymd::new(2026, 1, 1), 0))
}

// ---------------------------------------------------------------- derive

/// Tính lại mọi trường dẫn xuất (01 §3.2–§3.3) của một tài liệu sao lưu v1 (giữ nguyên mọi trường
/// khác và thứ tự khóa). Dùng để dựng bộ dữ liệu kiểm thử từ phần nhập.
pub fn derive_document(cal: &Cal, doc: &crate::json::Value) -> Result<crate::json::Value, String> {
    use crate::json::{obj, opt_int, s, Value};
    let mut doc = doc.clone();
    let Value::Obj(top) = &mut doc else {
        return Err("không phải đối tượng".into());
    };
    let Some((_, Value::Arr(persons))) = top.iter_mut().find(|(k, _)| k == "persons") else {
        return Err("thiếu persons".into());
    };
    let lunar_json = |l: &LunarDay| {
        obj(vec![
            ("year", opt_int(l.year.map(i64::from))),
            ("month", Value::Int(i64::from(l.month))),
            ("day", Value::Int(i64::from(l.day))),
            ("leap", Value::Bool(l.leap)),
        ])
    };
    let ymd_json = |d: Option<Ymd>| {
        d.map_or(Value::Null, |d| {
            obj(vec![
                ("y", Value::Int(i64::from(d.y))),
                ("m", Value::Int(i64::from(d.m))),
                ("d", Value::Int(i64::from(d.d))),
            ])
        })
    };
    for p in persons.iter_mut() {
        let Value::Obj(pf) = p else { continue };
        let nguong = pf
            .iter()
            .find(|(k, _)| k == "memorial")
            .and_then(|(_, m)| m.get("huong_duong_duoi"))
            .and_then(Value::as_i64)
            .map_or(DEFAULT_HUONG_DUONG_DUOI, |x| x as i32);
        let mut birth_info: Option<BirthInfo> = None;
        if let Some((_, b)) = pf.iter_mut().find(|(k, _)| k == "birth") {
            if !b.is_null() {
                let input = parse_date_value(b.get("input").ok_or("thiếu birth.input")?)
                    .map_err(|e| e.0)?;
                let bi = derive_birth(cal, &input).map_err(|e| e.message)?;
                let Value::Obj(bf) = b else { unreachable!() };
                for (k, v) in bf.iter_mut() {
                    match k.as_str() {
                        "lunar_year" => *v = opt_int(bi.lunar_year.map(i64::from)),
                        "lunar_year_approx" => *v = Value::Bool(bi.lunar_year_approx),
                        "can_chi" => *v = bi.can_chi.as_deref().map_or(Value::Null, s),
                        "lunar" => *v = bi.lunar.as_ref().map_or(Value::Null, lunar_json),
                        "solar" => *v = ymd_json(bi.solar),
                        _ => {}
                    }
                }
                birth_info = Some(bi);
            }
        }
        if let Some((_, d)) = pf.iter_mut().find(|(k, _)| k == "death") {
            if !d.is_null() {
                let input = parse_date_value(d.get("input").ok_or("thiếu death.input")?)
                    .map_err(|e| e.0)?;
                let di = save_death_with(cal, &input, birth_info.as_ref(), nguong)
                    .map_err(|e| e.message)?;
                let Value::Obj(df) = d else { unreachable!() };
                for (k, v) in df.iter_mut() {
                    match k.as_str() {
                        "lunar" => *v = lunar_json(&di.lunar),
                        "solar" => *v = ymd_json(di.solar),
                        "lunar_source" => {
                            *v = s(match di.lunar_source {
                                LunarSource::Entered => "entered",
                                LunarSource::Converted => "converted",
                            })
                        }
                        "method" => *v = s(crate::gio::method_name(di.method)),
                        "tuoi" => {
                            *v = obj(vec![
                                ("value", opt_int(di.tuoi.value.map(i64::from))),
                                ("label", s(di.tuoi.label.code())),
                                ("manual", Value::Bool(false)),
                                ("approx", Value::Bool(di.tuoi.approx)),
                            ])
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(doc)
}
