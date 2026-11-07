//! Lần xuất hiện (giỗ, sinh nhật dương, sinh nhật âm, sự kiện) trong một khoảng ngày dương
//! (02 §2.3, §5, §6), danh sách "Sắp tới" (02 §7.2) và văn bản hiển thị / thông báo (02 §7.1, O-9).

use crate::cal::Cal;
use crate::gio::{
    anniversaries_in, can_chi_year, label_text, solar_occurrence, tuoi_text, GioLabel,
};
use crate::model::*;
use lvhv_core::jd::weekday;
use std::cmp::Ordering;

/// Loại lần xuất hiện (tên mã dùng trong dump R3/R11 và nhóm sắp xếp 02 §7.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OccClass {
    Gio,
    TienThuong,
    SinhNhat,
    SinhNhatAm,
    SuKienDuongLap,
    SuKienAmLap,
    SuKienKhongLap,
}

impl OccClass {
    pub fn code(&self) -> &'static str {
        match self {
            OccClass::Gio => "gio",
            OccClass::TienThuong => "tien_thuong",
            OccClass::SinhNhat => "sinh_nhat",
            OccClass::SinhNhatAm => "sinh_nhat_am",
            OccClass::SuKienDuongLap => "su_kien_duong_lap",
            OccClass::SuKienAmLap => "su_kien_am_lap",
            OccClass::SuKienKhongLap => "su_kien_khong_lap",
        }
    }
    /// Nhóm thứ tự 02 §7.2: giỗ → tiên thường → sinh nhật → sự kiện.
    pub fn group(&self) -> u8 {
        match self {
            OccClass::Gio => 0,
            OccClass::TienThuong => 1,
            OccClass::SinhNhat | OccClass::SinhNhatAm => 2,
            _ => 3,
        }
    }
    pub fn is_event(&self) -> bool {
        self.group() == 3
    }
    /// Loại trong UID .ics (02 §8.2).
    pub fn uid_kind(&self) -> &'static str {
        match self {
            OccClass::Gio => "gio",
            OccClass::TienThuong => "tt",
            OccClass::SinhNhat => "sn",
            OccClass::SinhNhatAm => "sna",
            _ => "ev",
        }
    }
    /// Các loại dùng cho không gian tổ hợp R3, theo thứ tự liệt kê.
    pub const R3: [OccClass; 6] = [
        OccClass::Gio,
        OccClass::SinhNhat,
        OccClass::SinhNhatAm,
        OccClass::SuKienDuongLap,
        OccClass::SuKienAmLap,
        OccClass::SuKienKhongLap,
    ];
}

/// `occ.key` (02 §7.1): năm âm Y, năm dương Y hoặc `"once"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OccKey {
    Year(i32),
    Once,
}

impl OccKey {
    pub fn text(&self) -> String {
        match self {
            OccKey::Year(y) => y.to_string(),
            OccKey::Once => "once".to_string(),
        }
    }
}

/// Một lần xuất hiện.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    pub class: OccClass,
    /// Id của Person (giỗ, sinh nhật) hoặc Event.
    pub entity_id: String,
    pub key: OccKey,
    /// Ngày dương D.
    pub date: Ymd,
    /// Lần thứ n (giỗ lần n; tuổi tròn của sinh nhật; số năm của sự kiện lặp); `None` khi không rõ năm gốc hoặc sự kiện một lần.
    pub n: Option<i32>,
    /// Nhãn giỗ (chỉ `Gio`/`TienThuong`).
    pub gio_label: Option<GioLabel>,
    /// Khóa sắp xếp tên (02 §7.2): `ho_ten` của người; tiêu đề hiển thị của sự kiện.
    pub sort_name: String,
    /// Tên hiển thị (danh xưng + họ tên) hoặc tiêu đề sự kiện.
    pub display: String,
    /// Tiêu đề chính có tên (dùng cho SUMMARY .ics và thông báo khi bật hiện tên). Sự kiện: tiêu đề + " · " +
    /// tên hiển thị của những người gắn với sự kiện, cách nhau ", " (duyệt thiết kế TKN-A-08).
    pub headline: String,
    /// Như `headline` nhưng sự kiện không kèm tên người (để giao diện tự ghép tên riêng). Với hồ sơ: = `headline`.
    pub headline_plain: String,
    /// `rev` của bản ghi nguồn.
    pub rev: i64,
}

fn person_occ(
    p: &Person,
    class: OccClass,
    key: OccKey,
    date: Ymd,
    n: Option<i32>,
    gio_label: Option<GioLabel>,
) -> Occurrence {
    let display = p.display_name();
    let headline = match class {
        OccClass::Gio => format!(
            "{} {display}",
            label_text(gio_label.unwrap_or(GioLabel::Gio), n)
        ),
        OccClass::SinhNhat => match n {
            Some(k) => format!("Sinh nhật {display} ({k} tuổi)"),
            None => format!("Sinh nhật {display}"),
        },
        OccClass::SinhNhatAm => format!("Sinh nhật âm lịch {display}"),
        _ => display.clone(),
    };
    Occurrence {
        class,
        entity_id: p.id.clone(),
        key,
        date,
        n,
        gio_label,
        sort_name: p.ho_ten.clone(),
        display,
        headline_plain: headline.clone(),
        headline,
        rev: p.rev,
    }
}

/// Tiêu đề sự kiện: `tieu_de`, hoặc tên loại khi rỗng (01 §3.4).
pub fn event_title(ds: &Dataset, ev: &Event) -> String {
    if !ev.tieu_de.is_empty() {
        return ev.tieu_de.clone();
    }
    ds.event_type(&ev.type_id)
        .map_or_else(|| "Sự kiện".to_string(), |t| t.ten.clone())
}

fn event_occ(
    ds: &Dataset,
    ev: &Event,
    class: OccClass,
    key: OccKey,
    date: Ymd,
    n: Option<i32>,
) -> Occurrence {
    let title = event_title(ds, ev);
    let is_wedding = ds
        .event_type(&ev.type_id)
        .and_then(|t| t.builtin.as_deref())
        == Some("ngay_cuoi");
    let headline = match n {
        Some(k) if k >= 1 && is_wedding && ev.tieu_de.is_empty() => {
            format!("Kỷ niệm {k} năm ngày cưới")
        }
        Some(k) if k >= 1 && is_wedding => format!("Kỷ niệm {k} năm ngày cưới: {title}"),
        Some(k) if k >= 1 => format!("{title} (kỷ niệm {k} năm)"),
        _ => title.clone(),
    };
    let names: Vec<String> = ev
        .person_ids
        .iter()
        .filter_map(|id| ds.person(id))
        .map(|p| p.display_name())
        .collect();
    let headline_plain = headline;
    let headline = if names.is_empty() {
        headline_plain.clone()
    } else {
        format!("{headline_plain} · {}", names.join(", "))
    };
    Occurrence {
        class,
        entity_id: ev.id.clone(),
        key,
        date,
        n,
        gio_label: None,
        sort_name: title.clone(),
        display: title,
        headline,
        headline_plain,
        rev: ev.rev,
    }
}

/// Lớp của một sự kiện theo kiểu ngày và `recurring`.
pub fn event_class(ev: &Event) -> OccClass {
    match (
        ev.date.calendar,
        ev.recurring || ev.date.precision == Precision::MonthDay,
    ) {
        (Calendar::Solar, true) => OccClass::SuKienDuongLap,
        (Calendar::Lunar, true) => OccClass::SuKienAmLap,
        _ => OccClass::SuKienKhongLap,
    }
}

/// Mọi lần xuất hiện của một người có ngày dương trong `[from, to]` (chưa sắp). 02 §6.1 (Bổ sung B4):
/// hồ sơ có ngày mất chỉ có giỗ; không có sinh nhật dương/âm dù còn dữ liệu ngày sinh.
pub fn person_occurrences(cal: &Cal, p: &Person, from: Ymd, to: Ymd, out: &mut Vec<Occurrence>) {
    if from > to {
        return;
    }
    match &p.death {
        Some(d) => {
            for a in anniversaries_in(
                cal,
                d.lunar.year,
                d.lunar.month,
                d.lunar.day,
                p.gio_het_nam,
                from,
                to,
            ) {
                out.push(person_occ(
                    p,
                    OccClass::Gio,
                    OccKey::Year(a.year),
                    a.solar,
                    a.n,
                    Some(a.label),
                ));
            }
        }
        None => {
            let Some(b) = &p.birth else { return };
            if let Some(s) = b.solar {
                for y in from.y.max(s.y + 1)..=to.y {
                    if let Some(o) = solar_occurrence(s, y, p.feb29_rule) {
                        if from <= o && o <= to {
                            out.push(person_occ(
                                p,
                                OccClass::SinhNhat,
                                OccKey::Year(y),
                                o,
                                Some(y - s.y),
                                None,
                            ));
                        }
                    }
                }
            }
            if p.sinh_nhat_am {
                if let Some(l) = b.lunar {
                    for a in
                        anniversaries_in(cal, l.year, l.month, l.day, DEFAULT_GIO_HET_NAM, from, to)
                    {
                        out.push(person_occ(
                            p,
                            OccClass::SinhNhatAm,
                            OccKey::Year(a.year),
                            a.solar,
                            a.n,
                            None,
                        ));
                    }
                }
            }
        }
    }
}

/// Sự kiện gắn với ít nhất một hồ sơ có ngày mất (02 §6.1, Bổ sung B4) — không sinh lần xuất hiện.
pub fn event_has_deceased(ds: &Dataset, ev: &Event) -> bool {
    ev.person_ids
        .iter()
        .any(|id| ds.person(id).is_some_and(|p| p.death.is_some()))
}

/// Mọi lần xuất hiện của một sự kiện có ngày dương trong `[from, to]` (chưa sắp). Sự kiện gắn với người đã
/// mất không có lần xuất hiện nào (02 §6.1, Bổ sung B4).
pub fn event_occurrences(
    cal: &Cal,
    ds: &Dataset,
    ev: &Event,
    from: Ymd,
    to: Ymd,
    out: &mut Vec<Occurrence>,
) {
    if from > to || event_has_deceased(ds, ev) {
        return;
    }
    let class = event_class(ev);
    let dv = &ev.date;
    match class {
        OccClass::SuKienDuongLap => {
            let (Some(y0), Some(m), Some(d)) = (dv.year, dv.month, dv.day) else {
                return;
            };
            let orig = Ymd::new(y0, m, d);
            for y in from.y.max(y0)..=to.y {
                if let Some(o) = solar_occurrence(orig, y, Feb29Rule::Feb28) {
                    if from <= o && o <= to {
                        out.push(event_occ(ds, ev, class, OccKey::Year(y), o, Some(y - y0)));
                    }
                }
            }
        }
        OccClass::SuKienAmLap => {
            let (Some(m), Some(d)) = (dv.month, dv.day) else {
                return;
            };
            for a in anniversaries_in(cal, dv.year, m, d, DEFAULT_GIO_HET_NAM, from, to) {
                out.push(event_occ(ds, ev, class, OccKey::Year(a.year), a.solar, a.n));
            }
        }
        _ => {
            let (Some(y), Some(m), Some(d)) = (dv.year, dv.month, dv.day) else {
                return;
            };
            let date = match dv.calendar {
                Calendar::Solar => {
                    if !lvhv_core::jd::is_valid_solar(y, m, d) {
                        return;
                    }
                    Ymd::new(y, m, d)
                }
                Calendar::Lunar => match cal.l2s(y, m, d, dv.leap) {
                    Some(s) => Ymd::new(s.year, s.month, s.day),
                    None => return,
                },
            };
            if from <= date && date <= to {
                out.push(event_occ(ds, ev, class, OccKey::Once, date, None));
            }
        }
    }
}

/// So sánh điểm mã (UTF-8 theo byte = thứ tự điểm mã), như 02 §7.2 cho kiểm thử.
pub fn cmp_codepoints(a: &str, b: &str) -> Ordering {
    a.as_bytes().cmp(b.as_bytes())
}

/// Thứ tự hiển thị 02 §7.2: ngày → nhóm (giỗ, tiên thường, sinh nhật, sự kiện) → tên (điểm mã) →
/// (phân xử thêm, để xác định) lớp → id thực thể → khóa năm.
pub fn cmp_occ(a: &Occurrence, b: &Occurrence) -> Ordering {
    a.date
        .cmp(&b.date)
        .then(a.class.group().cmp(&b.class.group()))
        .then_with(|| cmp_codepoints(&a.sort_name, &b.sort_name))
        .then(a.class.cmp(&b.class))
        .then_with(|| a.entity_id.as_bytes().cmp(b.entity_id.as_bytes()))
        .then(a.key.cmp(&b.key))
}

/// Mọi lần xuất hiện (giỗ, sinh nhật, sự kiện — chưa gồm tiên thường) trong `[from, to]`, đã sắp
/// theo `cmp_occ`.
pub fn occurrences(cal: &Cal, ds: &Dataset, from: Ymd, to: Ymd) -> Vec<Occurrence> {
    let mut out = Vec::new();
    for p in &ds.persons {
        person_occurrences(cal, p, from, to, &mut out);
    }
    for ev in &ds.events {
        event_occurrences(cal, ds, ev, from, to, &mut out);
    }
    out.sort_by(cmp_occ);
    out
}

/// Lần tiên thường (02 §3) suy ra từ một lần giỗ: ngày D − 1, cùng khóa.
pub fn tien_thuong_of(g: &Occurrence) -> Occurrence {
    let mut t = g.clone();
    t.class = OccClass::TienThuong;
    t.date = Ymd::from_jdn(g.date.jdn() - 1);
    t.headline = format!("Tiên thường — {}", g.headline);
    t.headline_plain = t.headline.clone();
    t
}

/// Quy tắc nhắc của một lần xuất hiện (theo lớp).
pub fn rule_of<'a>(ds: &'a Dataset, occ: &Occurrence) -> Option<&'a ReminderRule> {
    match occ.class {
        OccClass::Gio | OccClass::TienThuong => ds.person(&occ.entity_id)?.rule_gio.as_ref(),
        OccClass::SinhNhat => ds.person(&occ.entity_id)?.rule_sinh_nhat.as_ref(),
        OccClass::SinhNhatAm => ds.person(&occ.entity_id)?.rule_sinh_nhat_am.as_ref(),
        _ => ds
            .events
            .iter()
            .find(|e| e.id == occ.entity_id)?
            .reminder
            .as_ref(),
    }
}

/// Giỗ có bật tiên thường (`rule_gio.tien_thuong`) — không phụ thuộc `enabled` (02 §7.2: danh
/// sách không phụ thuộc công tắc nhắc).
fn tien_thuong_on(ds: &Dataset, g: &Occurrence) -> bool {
    g.class == OccClass::Gio && rule_of(ds, g).is_some_and(|r| r.tien_thuong)
}

/// Mọi lần xuất hiện **kể cả tiên thường đã bật** có ngày trong `[from, to]`, đã sắp.
pub fn occurrences_with_tt(cal: &Cal, ds: &Dataset, from: Ymd, to: Ymd) -> Vec<Occurrence> {
    let to1 = Ymd::from_jdn(to.jdn() + 1);
    let all = occurrences(cal, ds, from, to1);
    let mut out = Vec::with_capacity(all.len());
    for o in all {
        if tien_thuong_on(ds, &o) {
            let t = tien_thuong_of(&o);
            if from <= t.date && t.date <= to {
                out.push(t);
            }
        }
        if o.date <= to {
            out.push(o);
        }
    }
    out.sort_by(cmp_occ);
    out
}

/// Một dòng của danh sách "Sắp tới".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpcomingItem {
    pub days: i64,
    pub occ: Occurrence,
}

/// Danh sách "Sắp tới" (02 §7.2): `[today, today + days]` gồm hai đầu.
pub fn upcoming(cal: &Cal, ds: &Dataset, today: Ymd, days: u32) -> Vec<UpcomingItem> {
    let t0 = today.jdn();
    occurrences_with_tt(cal, ds, today, Ymd::from_jdn(t0 + i64::from(days)))
        .into_iter()
        .map(|occ| UpcomingItem {
            days: occ.date.jdn() - t0,
            occ,
        })
        .collect()
}

/// "Hôm nay", "Ngày mai", "Còn n ngày".
pub fn days_text(days: i64) -> String {
    match days {
        0 => "Hôm nay".to_string(),
        1 => "Ngày mai".to_string(),
        n => format!("Còn {n} ngày"),
    }
}

pub const WEEKDAY_NAMES: [&str; 7] = [
    "Chủ nhật",
    "Thứ Hai",
    "Thứ Ba",
    "Thứ Tư",
    "Thứ Năm",
    "Thứ Sáu",
    "Thứ Bảy",
];

/// "15/6 âm lịch" (thêm " nhuận" sau tháng nếu là tháng nhuận).
pub fn lunar_short(cal: &Cal, d: Ymd) -> Option<(String, i32)> {
    let l = lvhv_core::solar_to_lunar_with(cal.method(), d.y, d.m, d.d).ok()?;
    Some((
        format!(
            "{}/{}{} âm lịch",
            l.day,
            l.month,
            if l.leap { " nhuận" } else { "" }
        ),
        l.year,
    ))
}

/// Dòng phụ 02 §7.1: "Thứ Ba, 28/07/2026 — 15/6 âm lịch".
pub fn date_line(cal: &Cal, d: Ymd) -> String {
    let wd = WEEKDAY_NAMES[weekday(d.jdn()) as usize];
    let base = format!("{wd}, {:02}/{:02}/{:04}", d.d, d.m, d.y);
    match lunar_short(cal, d) {
        Some((l, _)) => format!("{base} — {l}"),
        None => base,
    }
}

/// Văn bản thông báo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifText {
    pub title: String,
    pub body: String,
}

/// Loại lời nhắc của một lần xuất hiện.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RemType {
    /// Nhắc trước k ngày (k = 0: đúng ngày).
    Offset(u32),
    /// Nhắc ngày tiên thường.
    TienThuong,
}

fn lower_gio_label(l: GioLabel) -> &'static str {
    match l {
        GioLabel::GioDau => "giỗ đầu",
        GioLabel::GioHet => "giỗ hết",
        GioLabel::Gio => "giỗ",
    }
}

/// Văn bản có tên (dùng trong ứng dụng, VALARM .ics và thông báo khi bật hiện tên).
pub fn reminder_text_named(cal: &Cal, occ: &Occurrence, t: RemType) -> NotifText {
    match t {
        RemType::TienThuong => {
            // Ngày tiên thường: `occ` là lần giỗ (lời nhắc) hoặc chính lần tiên thường (.ics).
            let tt_day = if occ.class == OccClass::TienThuong {
                occ.date
            } else {
                Ymd::from_jdn(occ.date.jdn() - 1)
            };
            NotifText {
                title: format!(
                    "Hôm nay lễ tiên thường — ngày mai {} {}",
                    lower_gio_label(occ.gio_label.unwrap_or(GioLabel::Gio)),
                    occ.display
                ),
                body: date_line(cal, tt_day),
            }
        }
        RemType::Offset(k) => NotifText {
            title: if k == 0 {
                format!("Hôm nay: {}", occ.headline)
            } else {
                format!("Còn {k} ngày: {}", occ.headline)
            },
            body: date_line(cal, occ.date),
        },
    }
}

/// Văn bản trên màn hình khóa (O-9, biên bản G1): mặc định **không hiện tên** — chỉ "Sắp đến ngày
/// giỗ" / "Sắp đến sinh nhật" (sự kiện: "Sắp đến sự kiện"), không có dòng phụ. `show_names = true`
/// (người dùng bật) → `reminder_text_named`.
pub fn notification_text(cal: &Cal, occ: &Occurrence, t: RemType, show_names: bool) -> NotifText {
    if show_names {
        return reminder_text_named(cal, occ, t);
    }
    let title = match occ.class.group() {
        0 | 1 => "Sắp đến ngày giỗ",
        2 => "Sắp đến sinh nhật",
        _ => "Sắp đến sự kiện",
    };
    NotifText {
        title: title.to_string(),
        body: String::new(),
    }
}

/// Mô tả (DESCRIPTION .ics) của một lần xuất hiện.
pub fn description(cal: &Cal, ds: &Dataset, occ: &Occurrence) -> String {
    let lunar = |d: Ymd| match lunar_short(cal, d) {
        Some((s, ly)) => format!("{s} năm {} ({ly}).", can_chi_year(ly)),
        None => String::new(),
    };
    match occ.class {
        OccClass::Gio => {
            let mut s = lunar(occ.date);
            if let Some(t) = ds
                .person(&occ.entity_id)
                .and_then(|p| p.death.as_ref())
                .and_then(|d| tuoi_text(&d.tuoi))
            {
                s.push_str(&format!(" {t}."));
            }
            s
        }
        OccClass::TienThuong => {
            let g = Ymd::from_jdn(occ.date.jdn() + 1);
            format!("Lễ tiên thường, hôm trước ngày giỗ {}", lunar(g))
        }
        _ => lunar(occ.date),
    }
}
