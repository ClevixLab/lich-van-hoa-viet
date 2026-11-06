//! Tệp .ics (02 §8, RFC 5545): sự kiện cả ngày (O-5), VALARM theo `offsets_days` và
//! `time_local`, UID ổn định, không RRULE, mọi lần xuất hiện trong `[today, today + 10 năm)`.

use crate::cal::Cal;
use crate::model::*;
use crate::occ::{
    description, occurrences_with_tt, reminder_text_named, rule_of, OccClass, Occurrence, RemType,
};

pub const PRODID: &str = "-//Clevix Lab//Lich va Van hoa Viet 0.2.0//VI";
pub const CALNAME: &str = "Ngày giỗ và sinh nhật";
pub const UID_DOMAIN: &str = "lich.clevix.vn";

/// Tham số xuất.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IcsOptions {
    /// "Hôm nay" (ngày dương của thiết bị).
    pub today: Ymd,
    /// `DTSTAMP` (UTC) dạng `YYYYMMDDTHHMMSSZ`.
    pub dtstamp: String,
    /// Chỉ xuất các mục có id trong danh sách (Person hoặc Event); `None` = mặc định 02 §8.1:
    /// mọi mục có quy tắc nhắc bật.
    pub only_ids: Option<Vec<String>>,
}

/// Ngày cuối (không gồm) của cửa sổ 10 năm: cùng tháng/ngày năm + 10 (29/02 → 01/03).
pub fn ten_years_end(today: Ymd) -> Ymd {
    let y = today.y + 10;
    if lvhv_core::jd::is_valid_solar(y, today.m, today.d) {
        Ymd::new(y, today.m, today.d)
    } else {
        Ymd::new(y, 3, 1)
    }
}

/// Tên tệp (02 §8.1).
pub fn file_name(today: Ymd) -> String {
    format!("LichVanHoaViet-ngay-gio-{}.ics", today.iso())
}

/// Thoát văn bản (RFC 5545 §3.3.11).
pub fn escape_text(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => o.push_str("\\\\"),
            ';' => o.push_str("\\;"),
            ',' => o.push_str("\\,"),
            '\n' => o.push_str("\\n"),
            '\r' => {}
            c => o.push(c),
        }
    }
    o
}

/// Gấp một dòng nội dung: mỗi dòng vật lý ≤ 75 octet (không tính CRLF), dòng tiếp bắt đầu bằng
/// một dấu cách (tính vào 75), không cắt giữa ký tự UTF-8 (RFC 5545 §3.1). Ghi kèm CRLF.
pub fn fold_into(out: &mut String, line: &str) {
    let mut used = 0usize;
    for c in line.chars() {
        let n = c.len_utf8();
        if used + n > 75 {
            out.push_str("\r\n ");
            used = 1;
        }
        out.push(c);
        used += n;
    }
    out.push_str("\r\n");
}

/// TRIGGER tương đối với đầu ngày (02 §8.2): `dur = time_local − k × 24h`, dạng chuẩn
/// `[-]P{d}DT{h}H{m}M` bỏ phần bằng 0; `dur = 0` → `PT0S`.
pub fn trigger(k: u32, time_local: u32) -> String {
    let dur = i64::from(time_local) - i64::from(k) * 1440;
    if dur == 0 {
        return "PT0S".to_string();
    }
    let neg = dur < 0;
    let a = dur.unsigned_abs();
    let (d, h, m) = (a / 1440, a % 1440 / 60, a % 60);
    let mut s = String::new();
    if neg {
        s.push('-');
    }
    s.push('P');
    if d > 0 {
        s.push_str(&format!("{d}D"));
    }
    if h > 0 || m > 0 {
        s.push('T');
        if h > 0 {
            s.push_str(&format!("{h}H"));
        }
        if m > 0 {
            s.push_str(&format!("{m}M"));
        }
    }
    s
}

fn ics_date(d: Ymd) -> String {
    format!("{:04}{:02}{:02}", d.y, d.m, d.d)
}

/// UID (02 §8.2): `<id thực thể>-<loại>-<khóa năm>@lich.clevix.vn`.
pub fn uid(occ: &Occurrence) -> String {
    format!(
        "{}-{}-{}@{UID_DOMAIN}",
        occ.entity_id,
        occ.class.uid_kind(),
        occ.key.text()
    )
}

fn categories(c: OccClass) -> &'static str {
    match c {
        OccClass::Gio | OccClass::TienThuong => "Ngày giỗ",
        OccClass::SinhNhat | OccClass::SinhNhatAm => "Sinh nhật",
        _ => "Sự kiện",
    }
}

/// Các lần xuất hiện sẽ xuất (đã sắp theo 02 §7.2), cùng quy tắc nhắc của chúng.
pub fn ics_occurrences<'a>(
    cal: &Cal,
    ds: &'a Dataset,
    opt: &IcsOptions,
) -> Vec<(Occurrence, Option<&'a ReminderRule>)> {
    let end_excl = ten_years_end(opt.today);
    let to = Ymd::from_jdn(end_excl.jdn() - 1);
    occurrences_with_tt(cal, ds, opt.today, to)
        .into_iter()
        .filter_map(|o| {
            let rule = rule_of(ds, &o);
            let selected = match &opt.only_ids {
                Some(ids) => ids.contains(&o.entity_id),
                None => rule.is_some_and(|r| r.enabled),
            };
            selected.then_some((o, rule))
        })
        .collect()
}

/// Sinh nội dung tệp .ics (CRLF, UTF-8).
pub fn generate(cal: &Cal, ds: &Dataset, opt: &IcsOptions) -> String {
    let mut out = String::new();
    let l = |out: &mut String, s: &str| fold_into(out, s);
    for s in [
        "BEGIN:VCALENDAR",
        "VERSION:2.0",
        &format!("PRODID:{PRODID}"),
        "CALSCALE:GREGORIAN",
        "METHOD:PUBLISH",
        &format!("X-WR-CALNAME:{}", escape_text(CALNAME)),
        "X-WR-TIMEZONE:Asia/Ho_Chi_Minh",
        "BEGIN:VTIMEZONE",
        "TZID:Asia/Ho_Chi_Minh",
        "BEGIN:STANDARD",
        "DTSTART:19700101T000000",
        "TZOFFSETFROM:+0700",
        "TZOFFSETTO:+0700",
        "TZNAME:+07",
        "END:STANDARD",
        "END:VTIMEZONE",
    ] {
        l(&mut out, s);
    }
    for (o, rule) in ics_occurrences(cal, ds, opt) {
        l(&mut out, "BEGIN:VEVENT");
        l(&mut out, &format!("UID:{}", uid(&o)));
        l(&mut out, &format!("DTSTAMP:{}", opt.dtstamp));
        l(&mut out, &format!("SEQUENCE:{}", o.rev));
        l(
            &mut out,
            &format!("DTSTART;VALUE=DATE:{}", ics_date(o.date)),
        );
        l(
            &mut out,
            &format!(
                "DTEND;VALUE=DATE:{}",
                ics_date(Ymd::from_jdn(o.date.jdn() + 1))
            ),
        );
        l(&mut out, &format!("SUMMARY:{}", escape_text(&o.headline)));
        let desc = description(cal, ds, &o);
        if !desc.is_empty() {
            l(&mut out, &format!("DESCRIPTION:{}", escape_text(&desc)));
        }
        l(&mut out, "TRANSP:TRANSPARENT");
        l(
            &mut out,
            &format!("CATEGORIES:{}", escape_text(categories(o.class))),
        );
        if let Some(r) = rule.filter(|r| r.enabled) {
            let alarms: Vec<(u32, RemType)> = if o.class == OccClass::TienThuong {
                vec![(0, RemType::TienThuong)]
            } else {
                // Thứ tự: mốc xa nhất trước (như ví dụ 02 §8.1: -P2DT17H rồi PT7H).
                r.offsets_days
                    .iter()
                    .rev()
                    .map(|&k| (k, RemType::Offset(k)))
                    .collect()
            };
            for (k, t) in alarms {
                let txt = reminder_text_named(cal, &o, t);
                l(&mut out, "BEGIN:VALARM");
                l(&mut out, "ACTION:DISPLAY");
                l(
                    &mut out,
                    &format!("DESCRIPTION:{}", escape_text(&txt.title)),
                );
                l(
                    &mut out,
                    &format!("TRIGGER;RELATED=START:{}", trigger(k, r.time_local)),
                );
                l(&mut out, "END:VALARM");
            }
        }
        l(&mut out, "END:VEVENT");
    }
    l(&mut out, "END:VCALENDAR");
    out
}
