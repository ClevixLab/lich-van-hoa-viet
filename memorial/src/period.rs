//! Danh sách theo kỳ (R13, 02 §7.6): tuần (Thứ Hai → Chủ nhật), tháng, quý, năm dương; bộ lọc tất cả /
//! đã mất / đang sống; cùng tập và thứ tự với "Sắp tới" (02 §7.2). Dump `r13` (02 Phụ lục A.6).

use crate::cal::Cal;
use crate::json::write_json_str as write_str;
use crate::model::*;
use crate::occ::{event_title, occurrences_with_tt, OccClass, Occurrence};
use lvhv_core::jd::{days_in_month, weekday};
use std::fmt::Write as _;

/// Loại kỳ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PeriodKind {
    Tuan,
    Thang,
    Quy,
    Nam,
}

impl PeriodKind {
    pub const ALL: [PeriodKind; 4] = [
        PeriodKind::Tuan,
        PeriodKind::Thang,
        PeriodKind::Quy,
        PeriodKind::Nam,
    ];
    pub fn code(&self) -> &'static str {
        match self {
            PeriodKind::Tuan => "tuan",
            PeriodKind::Thang => "thang",
            PeriodKind::Quy => "quy",
            PeriodKind::Nam => "nam",
        }
    }
    pub fn parse(s: &str) -> Option<PeriodKind> {
        PeriodKind::ALL.into_iter().find(|k| k.code() == s)
    }
}

/// Bộ lọc theo trạng thái hồ sơ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersonFilter {
    TatCa,
    DaMat,
    DangSong,
}

impl PersonFilter {
    pub const ALL: [PersonFilter; 3] = [
        PersonFilter::TatCa,
        PersonFilter::DaMat,
        PersonFilter::DangSong,
    ];
    pub fn code(&self) -> &'static str {
        match self {
            PersonFilter::TatCa => "tat_ca",
            PersonFilter::DaMat => "da_mat",
            PersonFilter::DangSong => "dang_song",
        }
    }
    pub fn parse(s: &str) -> Option<PersonFilter> {
        PersonFilter::ALL.into_iter().find(|k| k.code() == s)
    }
}

/// `period_bounds(kind, anchor)` (02 §7.6): biên đầy đủ `(from, to)` của kỳ chứa `anchor`.
pub fn period_bounds(kind: PeriodKind, anchor: Ymd) -> (Ymd, Ymd) {
    match kind {
        PeriodKind::Tuan => {
            let j = anchor.jdn();
            let w = i64::from(weekday(j)); // 0 = Chủ nhật
            let from = j - (w + 6) % 7;
            (Ymd::from_jdn(from), Ymd::from_jdn(from + 6))
        }
        PeriodKind::Thang => (
            Ymd::new(anchor.y, anchor.m, 1),
            Ymd::new(anchor.y, anchor.m, days_in_month(anchor.y, anchor.m)),
        ),
        PeriodKind::Quy => {
            let q = (anchor.m - 1) / 3;
            let (m0, m1) = (3 * q + 1, 3 * q + 3);
            (
                Ymd::new(anchor.y, m0, 1),
                Ymd::new(anchor.y, m1, days_in_month(anchor.y, m1)),
            )
        }
        PeriodKind::Nam => (Ymd::new(anchor.y, 1, 1), Ymd::new(anchor.y, 12, 31)),
    }
}

/// Kỳ liền trước kỳ chứa `anchor`.
pub fn prev_period(kind: PeriodKind, anchor: Ymd) -> (Ymd, Ymd) {
    let (from, _) = period_bounds(kind, anchor);
    period_bounds(kind, Ymd::from_jdn(from.jdn() - 1))
}

/// Kỳ liền sau kỳ chứa `anchor`.
pub fn next_period(kind: PeriodKind, anchor: Ymd) -> (Ymd, Ymd) {
    let (_, to) = period_bounds(kind, anchor);
    period_bounds(kind, Ymd::from_jdn(to.jdn() + 1))
}

/// Ngày âm `{y, m, d, leap}` của một ngày dương.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LunarYmd {
    pub y: i32,
    pub m: u32,
    pub d: u32,
    pub leap: bool,
}

/// Một mục của danh sách theo kỳ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeriodItem {
    pub occ: Occurrence,
    /// `S2L(occ.date)`; `None` chỉ khi ngày ngoài miền của lõi.
    pub lunar: Option<LunarYmd>,
    /// `[id hồ sơ]` hoặc `person_ids` của sự kiện.
    pub person_ids: Vec<String>,
    /// Tên hiển thị của hồ sơ hoặc tiêu đề sự kiện.
    pub name: String,
}

/// Kết quả `events_in_period`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Period {
    pub kind: PeriodKind,
    pub anchor: Ymd,
    pub filter: PersonFilter,
    pub from: Ymd,
    pub to: Ymd,
    pub items: Vec<PeriodItem>,
}

/// Lần xuất hiện thuộc "đã mất" (02 §7.6): của hồ sơ có ngày mất; sự kiện có `person_ids` không rỗng
/// và mọi id trỏ tới hồ sơ có ngày mất.
pub fn is_deceased_occ(ds: &Dataset, occ: &Occurrence) -> bool {
    if occ.class.is_event() {
        let Some(ev) = ds.events.iter().find(|e| e.id == occ.entity_id) else {
            return false;
        };
        !ev.person_ids.is_empty()
            && ev
                .person_ids
                .iter()
                .all(|id| ds.person(id).is_some_and(|p| p.death.is_some()))
    } else {
        ds.person(&occ.entity_id).is_some_and(|p| p.death.is_some())
    }
}

/// Lần xuất hiện có qua bộ lọc không.
pub fn passes(ds: &Dataset, occ: &Occurrence, filter: PersonFilter) -> bool {
    match filter {
        PersonFilter::TatCa => true,
        PersonFilter::DaMat => is_deceased_occ(ds, occ),
        PersonFilter::DangSong => !is_deceased_occ(ds, occ),
    }
}

/// Dựng một mục (ngày âm, hồ sơ liên quan, tên).
pub fn period_item(cal: &Cal, ds: &Dataset, occ: Occurrence) -> PeriodItem {
    let lunar = lvhv_core::solar_to_lunar_with(cal.method(), occ.date.y, occ.date.m, occ.date.d)
        .ok()
        .map(|l| LunarYmd {
            y: l.year,
            m: l.month,
            d: l.day,
            leap: l.leap,
        });
    let (person_ids, name) = if occ.class.is_event() {
        match ds.events.iter().find(|e| e.id == occ.entity_id) {
            Some(ev) => (ev.person_ids.clone(), event_title(ds, ev)),
            None => (Vec::new(), occ.display.clone()),
        }
    } else {
        (vec![occ.entity_id.clone()], occ.display.clone())
    };
    PeriodItem {
        occ,
        lunar,
        person_ids,
        name,
    }
}

/// `events_in_period(ds, kind, anchor, filter)` (02 §7.6).
pub fn events_in_period(
    cal: &Cal,
    ds: &Dataset,
    kind: PeriodKind,
    anchor: Ymd,
    filter: PersonFilter,
) -> Period {
    let (from, to) = period_bounds(kind, anchor);
    let items = occurrences_with_tt(cal, ds, from, to)
        .into_iter()
        .filter(|o| passes(ds, o, filter))
        .map(|o| period_item(cal, ds, o))
        .collect();
    Period {
        kind,
        anchor,
        filter,
        from,
        to,
        items,
    }
}

/// Các kỳ (biên đầy đủ) có ít nhất một ngày trong `[from, to]`, theo thứ tự thời gian, kèm ngày neo
/// `max(kỳ.from, from)` (02 Phụ lục A.6).
pub fn periods_covering(kind: PeriodKind, from: Ymd, to: Ymd) -> Vec<(Ymd, Ymd, Ymd)> {
    let mut v = Vec::new();
    if from > to {
        return v;
    }
    let mut p = period_bounds(kind, from);
    while p.0 <= to {
        v.push((p.0.max(from), p.0, p.1));
        p = period_bounds(kind, Ymd::from_jdn(p.1.jdn() + 1));
    }
    v
}

/// Một dòng JSONL của dump `r13` (02 Phụ lục A.6).
pub fn period_line(out: &mut String, p: &Period) {
    let _ = write!(
        out,
        "{{\"kind\":\"{}\",\"anchor\":\"{}\",\"from\":\"{}\",\"to\":\"{}\",\"filter\":\"{}\",\"items\":[",
        p.kind.code(),
        p.anchor.iso(),
        p.from.iso(),
        p.to.iso(),
        p.filter.code()
    );
    for (i, it) in p.items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        item_json(out, it);
    }
    out.push_str("]}\n");
}

/// Một mục dạng JSON gọn (thứ tự khóa của 02 Phụ lục A.6).
pub fn item_json(out: &mut String, it: &PeriodItem) {
    let o = &it.occ;
    let _ = write!(out, "{{\"date\":\"{}\",\"lunar\":", o.date.iso());
    match it.lunar {
        Some(l) => {
            let _ = write!(
                out,
                "{{\"y\":{},\"m\":{},\"d\":{},\"leap\":{}}}",
                l.y, l.m, l.d, l.leap
            );
        }
        None => out.push_str("null"),
    }
    let _ = write!(out, ",\"class\":\"{}\",\"entity_id\":", o.class.code());
    write_str(out, &o.entity_id);
    out.push_str(",\"person_ids\":[");
    for (i, id) in it.person_ids.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_str(out, id);
    }
    out.push_str("],\"name\":");
    write_str(out, &it.name);
    out.push_str(",\"key\":");
    write_str(out, &o.key.text());
    let _ = write!(
        out,
        ",\"n\":{},\"label\":",
        o.n.map_or("null".to_string(), |n| n.to_string())
    );
    match o.gio_label {
        Some(l) if matches!(o.class, OccClass::Gio | OccClass::TienThuong) => {
            let _ = write!(out, "\"{}\"", l.code());
        }
        _ => out.push_str("null"),
    }
    out.push('}');
}

/// Dump `r13 DATASET [FROM TO]` (02 Phụ lục A.6).
pub fn r13_lines(cal: &Cal, ds: &Dataset, from: Ymd, to: Ymd, out: &mut String) {
    for kind in PeriodKind::ALL {
        for (anchor, _, _) in periods_covering(kind, from, to) {
            for f in PersonFilter::ALL {
                period_line(out, &events_in_period(cal, ds, kind, anchor, f));
            }
        }
    }
}
