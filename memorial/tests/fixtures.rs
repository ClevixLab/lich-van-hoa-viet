//! Ca kiểm thử cố định của docs/v0.2/02 §2.4, §4, §7.1, §8, §9 và các quyết định G1 (06).

use lvhv_memorial::gio::*;
use lvhv_memorial::ics::{self, IcsOptions};
use lvhv_memorial::model::*;
use lvhv_memorial::occ::*;
use lvhv_memorial::remind::*;
use lvhv_memorial::Cal;

fn cal() -> &'static Cal {
    default_cal()
}

fn gio(death: DateValue, y: i32) -> Option<Anniversary> {
    let d = save_death(&death, None, 60).unwrap();
    anniversary(cal(), d.lunar.year, d.lunar.month, d.lunar.day, 2, y)
}

fn ann(a: Option<Anniversary>) -> (i32, u32, u32, Ymd, Option<i32>, &'static str) {
    let a = a.expect("có giỗ");
    (a.year, a.month, a.day, a.solar, a.n, a.label.code())
}

#[test]
fn g_cases_02_2_4() {
    let ga = DateValue::solar(2025, 8, 8);
    let d = save_death(&ga, None, 60).unwrap();
    assert_eq!(
        (d.lunar.year, d.lunar.month, d.lunar.day, d.lunar.leap),
        (Some(2025), 6, 15, true)
    );
    assert_eq!(d.lunar_source, LunarSource::Converted);
    assert_eq!(
        ann(gio(ga, 2026)),
        (2026, 6, 15, Ymd::new(2026, 7, 28), Some(1), "gio_dau")
    );
    assert_eq!(
        ann(gio(ga, 2027)),
        (2027, 6, 15, Ymd::new(2027, 7, 18), Some(2), "gio_het")
    );
    assert!(gio(ga, 2025).is_none());

    let gb = DateValue::solar(2025, 9, 21);
    assert_eq!(
        ann(gio(gb, 2026)),
        (2026, 7, 29, Ymd::new(2026, 9, 10), Some(1), "gio_dau")
    );
    assert_eq!(
        ann(gio(gb, 2027)),
        (2027, 7, 30, Ymd::new(2027, 8, 31), Some(2), "gio_het")
    );

    let gc = DateValue::lunar(None, 3, 10, false);
    assert_eq!(
        ann(gio(gc, 2026)),
        (2026, 3, 10, Ymd::new(2026, 4, 26), None, "gio")
    );
    assert_eq!(
        ann(gio(gc, 2027)),
        (2027, 3, 10, Ymd::new(2027, 4, 16), None, "gio")
    );

    let gd = DateValue::solar(2024, 2, 9);
    assert_eq!(
        ann(gio(gd, 2024)),
        (2024, 12, 29, Ymd::new(2025, 1, 28), Some(1), "gio_dau")
    );
    assert_eq!(
        ann(gio(gd, 2025)),
        (2025, 12, 29, Ymd::new(2026, 2, 16), Some(2), "gio_het")
    );

    let ge = DateValue::lunar(Some(2027), 5, 15, false);
    assert_eq!(cal().leap_month(2028), Some(5));
    assert_eq!(
        ann(gio(ge, 2028)),
        (2028, 5, 15, Ymd::new(2028, 6, 7), Some(1), "gio_dau")
    );
}

#[test]
fn anniversaries_in_crosses_solar_year() {
    // G-D: năm giỗ âm 2024 rơi vào năm dương 2025 (02 §2.3).
    let d = save_death(&DateValue::solar(2024, 2, 9), None, 60).unwrap();
    let v = anniversaries_in(
        cal(),
        d.lunar.year,
        d.lunar.month,
        d.lunar.day,
        2,
        Ymd::new(2025, 1, 1),
        Ymd::new(2025, 12, 31),
    );
    assert_eq!(v.len(), 1);
    assert_eq!((v[0].year, v[0].solar), (2024, Ymd::new(2025, 1, 28)));
}

#[test]
fn label_of_o2() {
    assert_eq!(label_of(None, 2), GioLabel::Gio);
    assert_eq!(label_of(Some(1), 2), GioLabel::GioDau);
    assert_eq!(label_of(Some(2), 2), GioLabel::GioHet);
    assert_eq!(label_of(Some(2), 3), GioLabel::Gio);
    assert_eq!(label_of(Some(3), 3), GioLabel::GioHet);
    assert_eq!(label_of(Some(3), 2), GioLabel::Gio);
    assert_eq!(label_text(GioLabel::Gio, None), "Ngày giỗ");
    assert_eq!(label_text(GioLabel::Gio, Some(5)), "Giỗ");
}

#[test]
fn age_examples_02_4() {
    let b = derive_birth(cal(), &DateValue::solar(1990, 1, 20)).unwrap();
    assert_eq!(
        (b.lunar_year, b.can_chi.as_deref(), b.lunar_year_approx),
        (Some(1989), Some("Kỷ Tỵ"), false)
    );
    let today = Ymd::new(2026, 11, 6);
    assert_eq!(
        tuoi_duong(Ymd::new(1990, 1, 20), today, Feb29Rule::Feb28),
        Some(36)
    );
    assert_eq!(tuoi_am(cal(), 1989, today), Some(38));

    let b = derive_birth(cal(), &DateValue::solar(1940, 5, 1)).unwrap();
    assert_eq!(
        (b.lunar_year, b.can_chi.as_deref()),
        (Some(1940), Some("Canh Thìn"))
    );
    let d = save_death(&DateValue::solar(2025, 8, 8), Some(&b), 60).unwrap();
    assert_eq!(
        d.tuoi,
        Tuoi {
            value: Some(86),
            label: AgeLabel::HuongTho,
            manual: false,
            approx: false
        }
    );
    assert_eq!(tuoi_text(&d.tuoi).as_deref(), Some("Hưởng thọ 86 tuổi"));

    // O-3: < 60 hưởng dương, từ 60 hưởng thọ; ngưỡng sửa được.
    let mk = |by: i32| derive_birth(cal(), &DateValue::year_only(Calendar::Lunar, by)).unwrap();
    assert_eq!(
        tuoi_mat(Some(&mk(1966)), Some(2024), 60).unwrap().label,
        AgeLabel::HuongDuong
    ); // 59
    assert_eq!(
        tuoi_mat(Some(&mk(1965)), Some(2024), 60).unwrap().label,
        AgeLabel::HuongTho
    ); // 60
    assert_eq!(
        tuoi_mat(Some(&mk(1965)), Some(2024), 61).unwrap().label,
        AgeLabel::HuongDuong
    );
    // Dương chỉ năm → approx.
    let ys = derive_birth(cal(), &DateValue::year_only(Calendar::Solar, 1950)).unwrap();
    let t = tuoi_mat(Some(&ys), Some(1994), 60).unwrap();
    assert_eq!((t.value, t.approx), (Some(45), true));
    assert_eq!(
        tuoi_text(&t).as_deref(),
        Some("Hưởng dương 45 tuổi (khoảng)")
    );
    // Mâu thuẫn → lỗi.
    assert_eq!(
        tuoi_mat(Some(&mk(2000)), Some(1999), 60).unwrap_err().code,
        "DeathBeforeBirth"
    );
    assert!(save_death(&DateValue::solar(1999, 1, 1), Some(&mk(2000)), 60).is_err());
    // Không có ngày sinh → value null, nhãn mặc định hưởng thọ.
    assert_eq!(tuoi_mat(None, Some(2025), 60).unwrap().value, None);
}

#[test]
fn feb29_r10() {
    let o = Ymd::new(2000, 2, 29);
    assert_eq!(
        solar_occurrence(o, 2100, Feb29Rule::Feb28),
        Some(Ymd::new(2100, 2, 28))
    );
    assert_eq!(
        solar_occurrence(o, 2100, Feb29Rule::Mar1),
        Some(Ymd::new(2100, 3, 1))
    );
    assert_eq!(
        solar_occurrence(o, 2004, Feb29Rule::Mar1),
        Some(Ymd::new(2004, 2, 29))
    );
    assert_eq!(solar_occurrence(o, 1999, Feb29Rule::Feb28), None);
    for y in 1900..=3000 {
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let a = solar_occurrence(Ymd::new(1896, 2, 29), y, Feb29Rule::Feb28).unwrap();
        let b = solar_occurrence(Ymd::new(1896, 2, 29), y, Feb29Rule::Mar1).unwrap();
        if leap {
            assert_eq!((a, b), (Ymd::new(y, 2, 29), Ymd::new(y, 2, 29)));
        } else {
            assert_eq!((a, b), (Ymd::new(y, 2, 28), Ymd::new(y, 3, 1)));
        }
    }
    // Tuổi dương với 29/02: năm không nhuận, tròn tuổi vào 28/02 (feb28) hoặc 01/03 (mar1).
    assert_eq!(
        tuoi_duong(o, Ymd::new(2001, 2, 28), Feb29Rule::Feb28),
        Some(1)
    );
    assert_eq!(
        tuoi_duong(o, Ymd::new(2001, 2, 28), Feb29Rule::Mar1),
        Some(0)
    );
    assert_eq!(
        tuoi_duong(o, Ymd::new(2001, 3, 1), Feb29Rule::Mar1),
        Some(1)
    );
}

#[test]
fn can_chi_r10b() {
    let can = [
        "Giáp", "Ất", "Bính", "Đinh", "Mậu", "Kỷ", "Canh", "Tân", "Nhâm", "Quý",
    ];
    let chi = [
        "Tý", "Sửu", "Dần", "Mão", "Thìn", "Tỵ", "Ngọ", "Mùi", "Thân", "Dậu", "Tuất", "Hợi",
    ];
    for y in 1899..=3000 {
        let w = format!(
            "{} {}",
            can[((y + 6) % 10) as usize],
            chi[((y + 8) % 12) as usize]
        );
        assert_eq!(can_chi_year(y), w);
    }
    assert_eq!(can_chi_year(2026), "Bính Ngọ");
}

#[test]
fn edge_cases_02_9() {
    // Mất ngày đầu/cuối miền: giỗ ngoài miền trả None, không lỗi.
    let first = save_death(&DateValue::solar(1900, 1, 1), None, 60).unwrap();
    assert_eq!(first.lunar.year, Some(1899));
    assert!(anniversary(
        cal(),
        Some(1899),
        first.lunar.month,
        first.lunar.day,
        2,
        1900
    )
    .is_some());
    let last = save_death(&DateValue::solar(3000, 12, 31), None, 60).unwrap();
    assert_eq!(last.lunar.year, Some(3000));
    assert!(anniversary(cal(), Some(3000), last.lunar.month, last.lunar.day, 2, 3001).is_none());
    // Không rõ năm: tháng 11/12 âm năm 3000 rơi sang 3001 → ngoài miền → None.
    assert!(anniversary(cal(), None, 12, 30, 2, 3000).is_none());
    // Mất ngày 30 tháng Chạp → giỗ rơi sang năm dương sau.
    let a = anniversary(cal(), Some(1997), 12, 30, 2, 2025).unwrap();
    assert_eq!((a.day, a.solar.y), (29, 2026));
    // Ngày dương ngoài miền / không tồn tại.
    assert_eq!(
        save_death(&DateValue::solar(1899, 12, 31), None, 60)
            .unwrap_err()
            .code,
        "OutOfRange"
    );
    assert_eq!(
        save_death(&DateValue::solar(2023, 2, 29), None, 60)
            .unwrap_err()
            .code,
        "InvalidSolarDate"
    );
    assert_eq!(
        save_death(&DateValue::solar(2023, 4, 31), None, 60)
            .unwrap_err()
            .code,
        "InvalidSolarDate"
    );
    // Ngày âm không tồn tại (01 §6.1).
    let e = save_death(&DateValue::lunar(Some(2026), 7, 30, false), None, 60).unwrap_err();
    assert_eq!(
        (e.code, e.message.as_str()),
        ("LunarDayMissing", "Tháng 7 âm lịch năm 2026 chỉ có 29 ngày")
    );
    let e = save_death(&DateValue::lunar(Some(2026), 6, 1, true), None, 60).unwrap_err();
    assert_eq!(
        (e.code, e.message.as_str()),
        ("LunarMonthMissing", "Năm 2026 không có tháng 6 nhuận")
    );
    // precision = year không cho phép với ngày mất.
    assert_eq!(
        save_death(&DateValue::year_only(Calendar::Lunar, 2000), None, 60)
            .unwrap_err()
            .code,
        "PrecisionNotAllowed"
    );
    // Âm có năm ngoài miền đổi lịch: lưu được, không đổi lịch.
    let old = save_death(&DateValue::lunar(Some(1800), 3, 10, false), None, 60).unwrap();
    assert_eq!(old.solar, None);
}

/// R2 (phần thuật toán): mọi ngày dương 1900–3000 qua `save_death` khứ hồi; mọi ngày âm tồn tại
/// 1900–2999 nhập theo âm khứ hồi. (Phần "đọc lại từ kho" thuộc lớp lưu trữ, đợt 2.)
#[test]
fn r2_roundtrip_exhaustive() {
    let c = cal();
    for j in Ymd::new(1900, 1, 1).jdn()..=Ymd::new(3000, 12, 31).jdn() {
        let s = Ymd::from_jdn(j);
        let d = save_death(&DateValue::solar(s.y, s.m, s.d), None, 60).unwrap();
        let l = d.lunar;
        assert_eq!(d.solar, Some(s));
        assert_eq!(d.lunar_source, LunarSource::Converted);
        assert_eq!(
            c.l2s_jdn(l.year.unwrap(), l.month, l.day, l.leap),
            Some(j),
            "{s:?}"
        );
    }
    let mut n = 0u64;
    for ly in 1900..=2999 {
        for (m, d, leap) in lvhv_memorial::dump::lunar_days_of_year(c, ly) {
            let x = save_death(&DateValue::lunar(Some(ly), m, d, leap), None, 60).unwrap();
            let s = x.solar.unwrap();
            let back = lvhv_core::solar_to_lunar(s.y, s.m, s.d).unwrap();
            assert_eq!(
                (back.year, back.month, back.day, back.leap),
                (ly, m, d, leap)
            );
            assert_eq!(x.lunar_source, LunarSource::Entered);
            n += 1;
        }
    }
    assert!(n > 390_000);
}

fn ds() -> Dataset {
    parse_dataset(include_str!(
        "../../docs/v0.2/schema/example-backup-v1.json"
    ))
    .unwrap()
}

#[test]
fn reminder_example_02_7_1() {
    let ds = ds();
    let tz = Zone::vietnam();
    let p = &ds.persons[0];
    let occ = occurrences(cal(), &ds, Ymd::new(2026, 7, 1), Ymd::new(2026, 7, 31))
        .into_iter()
        .find(|o| o.entity_id == p.id)
        .unwrap();
    let now = tz.local_to_utc(local_minutes(Ymd::new(2026, 1, 1), 0));
    let r = reminders_for(true, p.rule_gio.as_ref().unwrap(), &occ, now, &tz);
    let got: Vec<(String, String)> = r
        .iter()
        .map(|x| (x.id.clone(), fmt_local(x.at_local)))
        .collect();
    assert_eq!(
        got,
        [
            (
                "lvhv:0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e11:2026:0".to_string(),
                "2026-07-28T07:00".to_string()
            ),
            (
                "lvhv:0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e11:2026:3".to_string(),
                "2026-07-25T07:00".to_string()
            ),
        ]
    );
    let t = notification_text(cal(), &occ, RemType::Offset(3), true);
    assert_eq!(t.title, "Còn 3 ngày: Giỗ đầu Ông Nguyễn Văn A");
    assert_eq!(t.body, "Thứ Ba, 28/07/2026 — 15/6 âm lịch");
    // O-9: mặc định không hiện tên.
    let t = notification_text(cal(), &occ, RemType::Offset(3), false);
    assert_eq!(
        (t.title.as_str(), t.body.as_str()),
        ("Sắp đến ngày giỗ", "")
    );
    let sn = occurrences(cal(), &ds, Ymd::new(2027, 1, 1), Ymd::new(2027, 1, 31))
        .into_iter()
        .find(|o| o.class == OccClass::SinhNhat)
        .unwrap();
    assert_eq!(
        notification_text(cal(), &sn, RemType::Offset(0), false).title,
        "Sắp đến sinh nhật"
    );
    assert!(!notification_text(cal(), &sn, RemType::Offset(0), false)
        .title
        .contains("Lê"));
    // Tiên thường: nội dung riêng; ngày D − 1.
    let b = &ds.persons[1];
    let g = occurrences(cal(), &ds, Ymd::new(2026, 4, 1), Ymd::new(2026, 4, 30))
        .into_iter()
        .find(|o| o.entity_id == b.id)
        .unwrap();
    let r = reminders_for(true, b.rule_gio.as_ref().unwrap(), &g, now, &tz);
    let tt = r.iter().find(|x| x.rtype == RemType::TienThuong).unwrap();
    assert_eq!(fmt_local(tt.at_local), "2026-04-25T07:00");
    assert!(tt.id.ends_with(":2026:tt"));
    assert_eq!(
        notification_text(cal(), &g, RemType::TienThuong, true).title,
        "Hôm nay lễ tiên thường — ngày mai giỗ Cụ Trần Thị B"
    );
}

#[test]
fn zone_dst_rules() {
    // Múi giờ giả định: +60 → +120 lúc 01:00 UTC ngày X (bước nhảy 02:00 → 03:00 giờ đồng hồ),
    // rồi +120 → +60 lúc 01:00 UTC ngày Y (03:00 → 02:00, giờ 02:xx lặp).
    let x = Ymd::new(2026, 3, 29).jdn() * 1440 + 60;
    let y = Ymd::new(2026, 10, 25).jdn() * 1440 + 60;
    let z = Zone {
        initial: 60,
        transitions: vec![(x, 120), (y, 60)],
    };
    // 02:30 không tồn tại → dời sau 1 giờ (03:30 giờ mới).
    let t = z.local_to_utc(local_minutes(Ymd::new(2026, 3, 29), 150));
    assert_eq!(fmt_local(z.to_local(t)), "2026-03-29T03:30");
    // 02:30 lặp → lần đầu (độ lệch +120).
    let t = z.local_to_utc(local_minutes(Ymd::new(2026, 10, 25), 150));
    assert_eq!(z.offset_at(t), 120);
    assert_eq!(fmt_local(z.to_local(t)), "2026-10-25T02:30");
}

#[test]
fn reminders_off_and_stable_ids() {
    let ds = ds();
    let tz = Zone::vietnam();
    let now = i64::MIN;
    let p = &ds.persons[0];
    let occ = occurrences(cal(), &ds, Ymd::new(2026, 7, 1), Ymd::new(2026, 7, 31))
        .into_iter()
        .find(|o| o.entity_id == p.id)
        .unwrap();
    let mut rule = p.rule_gio.clone().unwrap();
    assert!(reminders_for(false, &rule, &occ, now, &tz).is_empty());
    rule.enabled = false;
    assert!(reminders_for(true, &rule, &occ, now, &tz).is_empty());
    rule.enabled = true;
    let a: Vec<String> = reminders_for(true, &rule, &occ, now, &tz)
        .into_iter()
        .map(|r| r.id)
        .collect();
    rule.time_local = 23 * 60 + 59;
    let b: Vec<String> = reminders_for(true, &rule, &occ, now, &tz)
        .into_iter()
        .map(|r| r.id)
        .collect();
    assert_eq!(a, b);
    // at ≤ now không sinh.
    let at0 = tz.local_to_utc(local_minutes(Ymd::new(2026, 7, 25), rule.time_local));
    let r = reminders_for(true, &rule, &occ, at0, &tz);
    assert!(r.iter().all(|x| x.at > at0));
    assert_eq!(r.len(), 1);
    // Tiên thường chỉ cho giỗ.
    let sn = occurrences(cal(), &ds, Ymd::new(2027, 1, 1), Ymd::new(2027, 1, 31))
        .into_iter()
        .find(|o| o.class == OccClass::SinhNhat)
        .unwrap();
    let mut r2 = ReminderRule::default_with_id("x");
    r2.tien_thuong = true;
    assert!(reminders_for(true, &r2, &sn, now, &tz)
        .iter()
        .all(|x| x.rtype != RemType::TienThuong));
}

#[test]
fn upcoming_ordering_and_bounds() {
    let ds = ds();
    // Hôm nay = ngày giỗ G-A 2026 − 30: ngày thứ 30 có mặt.
    let today = Ymd::from_jdn(Ymd::new(2026, 7, 28).jdn() - 30);
    let v = upcoming(cal(), &ds, today, 30);
    assert!(v
        .iter()
        .any(|i| i.days == 30 && i.occ.class == OccClass::Gio));
    let v = upcoming(cal(), &ds, Ymd::new(2026, 7, 28), 30);
    assert_eq!((v[0].days, v[0].occ.class), (0, OccClass::Gio));
    assert_eq!(days_text(0), "Hôm nay");
    assert_eq!(days_text(1), "Ngày mai");
    assert_eq!(days_text(7), "Còn 7 ngày");
    // Không phụ thuộc công tắc nhắc.
    let mut off = ds.clone();
    off.reminders_enabled = false;
    for p in &mut off.persons {
        for r in [
            &mut p.rule_gio,
            &mut p.rule_sinh_nhat,
            &mut p.rule_sinh_nhat_am,
        ]
        .into_iter()
        .flatten()
        {
            r.enabled = false;
        }
    }
    for j in 0..400 {
        let t = Ymd::from_jdn(Ymd::new(2026, 1, 1).jdn() + j);
        assert_eq!(upcoming(cal(), &ds, t, 30), upcoming(cal(), &off, t, 30));
    }
    // Tiên thường (B bật) xuất hiện ngay trước giỗ, cùng khóa.
    let v = upcoming(cal(), &ds, Ymd::new(2026, 4, 20), 30);
    let i = v
        .iter()
        .position(|x| x.occ.class == OccClass::TienThuong)
        .unwrap();
    assert_eq!(v[i + 1].occ.class, OccClass::Gio);
    assert_eq!(v[i].days + 1, v[i + 1].days);
}

#[test]
fn ics_example_02_8_1() {
    let ds = ds();
    let opt = IcsOptions {
        today: Ymd::new(2026, 7, 1),
        dtstamp: "20261106T031500Z".into(),
        only_ids: None,
    };
    let folded = ics::generate(cal(), &ds, &opt);
    // Dòng DESCRIPTION dài hơn 75 octet được gấp (RFC 5545 §3.1); so sau khi mở gấp.
    assert!(folded.contains("Hưởng thọ 86 tu\r\n ổi."));
    let s = folded.replace("\r\n ", "");
    let want = "BEGIN:VEVENT\r\nUID:0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01-gio-2026@lich.clevix.vn\r\nDTSTAMP:20261106T031500Z\r\nSEQUENCE:1\r\nDTSTART;VALUE=DATE:20260728\r\nDTEND;VALUE=DATE:20260729\r\nSUMMARY:Giỗ đầu Ông Nguyễn Văn A\r\nDESCRIPTION:15/6 âm lịch năm Bính Ngọ (2026). Hưởng thọ 86 tuổi.\r\nTRANSP:TRANSPARENT\r\nCATEGORIES:Ngày giỗ\r\nBEGIN:VALARM\r\nACTION:DISPLAY\r\nDESCRIPTION:Còn 3 ngày: Giỗ đầu Ông Nguyễn Văn A\r\nTRIGGER;RELATED=START:-P2DT17H\r\nEND:VALARM\r\nBEGIN:VALARM\r\nACTION:DISPLAY\r\nDESCRIPTION:Hôm nay: Giỗ đầu Ông Nguyễn Văn A\r\nTRIGGER;RELATED=START:PT7H\r\nEND:VALARM\r\nEND:VEVENT\r\n";
    assert!(s.contains(want), "{s}");
    let head = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Clevix Lab//Lich va Van hoa Viet 0.2.0//VI\r\nCALSCALE:GREGORIAN\r\nMETHOD:PUBLISH\r\nX-WR-CALNAME:Ngày giỗ và sinh nhật\r\nX-WR-TIMEZONE:Asia/Ho_Chi_Minh\r\nBEGIN:VTIMEZONE\r\nTZID:Asia/Ho_Chi_Minh\r\nBEGIN:STANDARD\r\nDTSTART:19700101T000000\r\nTZOFFSETFROM:+0700\r\nTZOFFSETTO:+0700\r\nTZNAME:+07\r\nEND:STANDARD\r\nEND:VTIMEZONE\r\n";
    assert!(s.starts_with(head));
    assert!(s.ends_with("END:VCALENDAR\r\n"));
    assert_eq!(
        ics::file_name(Ymd::new(2026, 11, 6)),
        "LichVanHoaViet-ngay-gio-2026-11-06.ics"
    );
}

#[test]
fn ics_triggers_and_text() {
    assert_eq!(ics::trigger(3, 420), "-P2DT17H");
    assert_eq!(ics::trigger(0, 420), "PT7H");
    assert_eq!(ics::trigger(1, 0), "-P1D");
    assert_eq!(ics::trigger(0, 0), "PT0S");
    assert_eq!(ics::trigger(0, 23 * 60 + 59), "PT23H59M");
    assert_eq!(ics::trigger(60, 23 * 60 + 59), "-P59DT1M");
    assert_eq!(ics::trigger(15, 30), "-P14DT23H30M");
    assert_eq!(ics::escape_text("a,b;c\\d\ne"), "a\\,b\\;c\\\\d\\ne");
    let mut o = String::new();
    let long = "SUMMARY:".to_string() + &"Giỗ ".repeat(40);
    ics::fold_into(&mut o, &long);
    for l in o.split("\r\n").filter(|l| !l.is_empty()) {
        assert!(l.len() <= 75, "{} octet", l.len());
    }
    assert_eq!(o.replace("\r\n ", "").trim_end_matches("\r\n"), long);
}

/// R4(a)(b) phần tự kiểm: cú pháp và tập (UID, DTSTART) = lần xuất hiện R11 trong 10 năm;
/// UID ổn định sau khi sửa tên, SEQUENCE theo rev.
#[test]
fn ics_r3_dataset_syntax_semantics() {
    let ds = parse_dataset(include_str!("../testdata/r3-dataset.json")).unwrap();
    let today = Ymd::new(2026, 11, 6);
    let opt = IcsOptions {
        today,
        dtstamp: "20261106T031500Z".into(),
        only_ids: Some(
            ds.persons
                .iter()
                .map(|p| p.id.clone())
                .chain(ds.events.iter().map(|e| e.id.clone()))
                .collect(),
        ),
    };
    let s = ics::generate(cal(), &ds, &opt);
    assert!(!s.contains("RRULE"));
    assert!(!s.contains("TZID="));
    for l in s.split("\r\n") {
        assert!(l.len() <= 75);
        assert!(!l.contains('\n') && !l.contains('\r'));
    }
    // Mở gấp dòng và đếm.
    let un = s.replace("\r\n ", "");
    let lines: Vec<&str> = un.split("\r\n").collect();
    let mut set = Vec::new();
    let (mut uid, mut va, mut fields) = (String::new(), false, 0);
    for l in &lines {
        if let Some(u) = l.strip_prefix("UID:") {
            uid = u.to_string();
        }
        if let Some(d) = l.strip_prefix("DTSTART;VALUE=DATE:") {
            set.push((uid.clone(), d.to_string()));
        }
        if *l == "BEGIN:VALARM" {
            va = true;
            fields = 0;
        }
        if va
            && (l.starts_with("ACTION:")
                || l.starts_with("DESCRIPTION:")
                || l.starts_with("TRIGGER"))
        {
            fields += 1;
        }
        if *l == "END:VALARM" {
            assert_eq!(fields, 3);
            va = false;
        }
    }
    let end = Ymd::from_jdn(ics::ten_years_end(today).jdn() - 1);
    let occ = lvhv_memorial::occ::occurrences_with_tt(cal(), &ds, today, end);
    let want: Vec<(String, String)> = occ
        .iter()
        .map(|o| {
            (
                ics::uid(o),
                format!("{:04}{:02}{:02}", o.date.y, o.date.m, o.date.d),
            )
        })
        .collect();
    assert_eq!(set, want);
    assert_eq!(
        want.iter()
            .map(|w| &w.0)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        want.len()
    );
    // Sửa tên + tăng rev: cùng UID, SEQUENCE tăng.
    let mut ds2 = ds.clone();
    ds2.persons[0].ho_ten = "Nguyễn Văn Á".into();
    ds2.persons[0].rev += 1;
    let s2 = ics::generate(cal(), &ds2, &opt);
    let uids = |s: &str| {
        s.replace("\r\n ", "")
            .lines()
            .filter(|l| l.starts_with("UID:"))
            .map(String::from)
            .collect::<Vec<_>>()
    };
    assert_eq!(uids(&s), uids(&s2));
    assert!(s2.contains("SEQUENCE:2"));
}
