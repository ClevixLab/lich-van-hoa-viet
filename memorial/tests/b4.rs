//! Bổ sung B4 (04; 02 §6.1): với người đã mất chỉ có giỗ (+ tiên thường nếu bật); sự kiện gắn người đã mất
//! không có lần xuất hiện; khi một người được cập nhật ngày mất, lời nhắc sinh nhật/sự kiện bị gỡ ở lần
//! lập lịch kế tiếp.

use lvhv_core::DEFAULT_METHOD;
use lvhv_memorial::ics::{generate, IcsOptions};
use lvhv_memorial::model::*;
use lvhv_memorial::occ::{occurrences, occurrences_with_tt, upcoming, OccClass};
use lvhv_memorial::remind::*;
use lvhv_memorial::Cal;

fn r3() -> Dataset {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/r3-dataset.json");
    parse_dataset(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn by_name<'a>(ds: &'a mut Dataset, name: &str) -> &'a mut Person {
    ds.persons.iter_mut().find(|p| p.ho_ten == name).unwrap()
}

fn rule(id: &str) -> ReminderRule {
    ReminderRule {
        id: id.into(),
        enabled: true,
        offsets_days: vec![0, 3],
        time_local: 7 * 60,
        tien_thuong: true,
    }
}

/// Người đã mất có ngày sinh (dương + âm) và sự kiện riêng → chỉ giỗ và tiên thường.
#[test]
fn deceased_with_birthday_and_events_gives_only_gio() {
    let cal = Cal::get(DEFAULT_METHOD);
    let mut ds = r3();
    let a = by_name(&mut ds, "Nguyễn Văn A");
    assert!(a.death.is_some() && a.birth.as_ref().unwrap().solar.is_some());
    a.sinh_nhat_am = true;
    a.rule_sinh_nhat = Some(rule("c9000000-0000-4000-8000-000000000001"));
    a.rule_sinh_nhat_am = Some(rule("c9000000-0000-4000-8000-000000000002"));
    a.rule_gio = Some(rule("c9000000-0000-4000-8000-000000000003"));
    let aid = a.id.clone();
    ds.events[0].person_ids = vec![aid.clone()];
    let eid = ds.events[0].id.clone();
    let (f, t) = (Ymd::new(2026, 1, 1), Ymd::new(2036, 12, 31));
    let occ = occurrences_with_tt(cal, &ds, f, t);
    let classes: std::collections::BTreeSet<OccClass> = occ
        .iter()
        .filter(|o| o.entity_id == aid)
        .map(|o| o.class)
        .collect();
    assert_eq!(
        classes,
        [OccClass::Gio, OccClass::TienThuong].into_iter().collect()
    );
    assert!(occ.iter().all(|o| o.entity_id != eid));
    // Không lời nhắc, không .ics cho sinh nhật/sự kiện của người này.
    let tz = Zone::vietnam();
    let rs = all_reminders(cal, &ds, f, t, i64::MIN, &tz);
    assert!(rs
        .iter()
        .all(|r| !r.id.contains("c9000000-0000-4000-8000-000000000001")
            && !r.id.contains("c9000000-0000-4000-8000-000000000002")
            && r.occ.entity_id != eid));
    assert!(rs
        .iter()
        .any(|r| r.id.contains("c9000000-0000-4000-8000-000000000003")));
    let ics = generate(
        cal,
        &ds,
        &IcsOptions {
            today: Ymd::new(2026, 1, 1),
            dtstamp: "20260101T000000Z".into(),
            only_ids: Some(vec![aid.clone(), eid.clone()]),
        },
    );
    assert!(!ics.contains(&format!("UID:{aid}-sn-")));
    assert!(!ics.contains(&format!("UID:{aid}-sna-")));
    assert!(!ics.contains(&format!("UID:{eid}-")));
    assert!(ics.contains(&format!("UID:{aid}-gio-")));
}

/// Sự kiện gắn [người đã mất, người đang sống] → không lần xuất hiện nào; gắn chỉ người sống → có.
#[test]
fn mixed_event_gives_nothing() {
    let cal = Cal::get(DEFAULT_METHOD);
    let mut ds = r3();
    let dead = by_name(&mut ds, "Trần Thị B").id.clone();
    let alive = by_name(&mut ds, "Lê Thị C").id.clone();
    ds.events[0].person_ids = vec![alive.clone(), dead];
    ds.events[1].person_ids = vec![alive];
    let (e0, e1) = (ds.events[0].id.clone(), ds.events[1].id.clone());
    let occ = occurrences(cal, &ds, Ymd::new(2026, 1, 1), Ymd::new(2036, 12, 31));
    assert!(occ.iter().all(|o| o.entity_id != e0));
    assert!(occ.iter().any(|o| o.entity_id == e1));
    let up = upcoming(cal, &ds, Ymd::new(2026, 12, 1), 30);
    assert!(up.iter().all(|u| u.occ.entity_id != e0));
}

/// Người đang sống được cập nhật ngày mất → lần `reschedule` kế tiếp gỡ lời nhắc sinh nhật và sự kiện của
/// người đó, thêm lời nhắc giỗ.
#[test]
fn living_to_deceased_drops_birthday_and_event_reminders() {
    let cal = Cal::get(DEFAULT_METHOD);
    let tz = Zone::vietnam();
    let mut ds = r3();
    let death = by_name(&mut ds, "Nguyễn Văn A").death.clone();
    let c = by_name(&mut ds, "Lê Thị C");
    let cid = c.id.clone();
    let sn_rule = c.rule_sinh_nhat.as_ref().unwrap().id.clone();
    let sna_rule = c.rule_sinh_nhat_am.as_ref().unwrap().id.clone();
    // Chỉ giữ người C và một sự kiện của C để dưới 64 lời nhắc.
    ds.persons.retain(|p| p.id == cid);
    ds.events.truncate(1);
    ds.events[0].person_ids = vec![cid.clone()];
    let ev_rule = ds.events[0].reminder.as_ref().unwrap().id.clone();
    let now = tz.local_to_utc(local_minutes(Ymd::new(2026, 3, 1), 9 * 60));
    let desired = |ds: &Dataset| -> Vec<Pending> {
        reschedule_input(cal, ds, now, &tz)
            .iter()
            .map(|r| to_pending(cal, r, false))
            .collect()
    };
    let plan1 = reschedule(now, &desired(&ds), &[], &tz);
    let q = apply_plan(&[], &plan1);
    for r in [&sn_rule, &sna_rule, &ev_rule] {
        assert!(q.iter().any(|p| p.id.contains(r.as_str())), "trước: {r}");
    }
    // Cập nhật ngày mất.
    let c = ds.persons.iter_mut().find(|p| p.id == cid).unwrap();
    c.kind = PersonKind::Deceased;
    c.death = death;
    c.rule_gio = Some(rule("c9000000-0000-4000-8000-0000000000aa"));
    let plan2 = reschedule(now, &desired(&ds), &q, &tz);
    let q2 = apply_plan(&q, &plan2);
    for r in [&sn_rule, &sna_rule, &ev_rule] {
        assert!(q2.iter().all(|p| !p.id.contains(r.as_str())), "sau: {r}");
        assert!(plan2.remove.iter().any(|id| id.contains(r.as_str())));
    }
    assert!(q2
        .iter()
        .any(|p| p.id.contains("c9000000-0000-4000-8000-0000000000aa")));
}
