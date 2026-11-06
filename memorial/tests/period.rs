//! R13 (04 Bổ sung B2; 02 §7.6) — tự kiểm:
//! (c) nhất quán: tuần cắt theo tháng ghép lại = tháng; 3 tháng = quý; 4 quý = năm; đã mất ∪ đang sống =
//!     tất cả (rời nhau, giữ thứ tự); không trùng mục;
//! (d) ngày đầu và ngày cuối của mỗi kỳ đều được tính;
//! cùng biên kỳ, kỳ trước/sau, quy tắc lọc sự kiện nhiều người.

use lvhv_core::DEFAULT_METHOD;
use lvhv_memorial::model::*;
use lvhv_memorial::occ::occurrences_with_tt;
use lvhv_memorial::period::*;
use lvhv_memorial::Cal;
use std::collections::HashSet;

fn r3() -> Dataset {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/r3-dataset.json");
    parse_dataset(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn keys(p: &Period) -> Vec<(String, String, String)> {
    p.items
        .iter()
        .map(|i| {
            (
                i.occ.class.code().to_string(),
                i.occ.entity_id.clone(),
                i.occ.key.text(),
            )
        })
        .collect()
}

fn clip(p: &Period, from: Ymd, to: Ymd) -> Vec<(String, String, String)> {
    p.items
        .iter()
        .filter(|i| from <= i.occ.date && i.occ.date <= to)
        .map(|i| {
            (
                i.occ.class.code().to_string(),
                i.occ.entity_id.clone(),
                i.occ.key.text(),
            )
        })
        .collect()
}

#[test]
fn bounds_and_navigation() {
    // 2026-01-01 là Thứ Năm → tuần 2025-12-29 (Thứ Hai) … 2026-01-04 (Chủ nhật).
    let a = Ymd::new(2026, 1, 1);
    assert_eq!(
        period_bounds(PeriodKind::Tuan, a),
        (Ymd::new(2025, 12, 29), Ymd::new(2026, 1, 4))
    );
    // Chủ nhật thuộc tuần bắt đầu từ Thứ Hai trước nó.
    assert_eq!(
        period_bounds(PeriodKind::Tuan, Ymd::new(2026, 1, 4)).0,
        Ymd::new(2025, 12, 29)
    );
    assert_eq!(
        period_bounds(PeriodKind::Tuan, Ymd::new(2026, 1, 5)).0,
        Ymd::new(2026, 1, 5)
    );
    assert_eq!(
        period_bounds(PeriodKind::Thang, Ymd::new(2028, 2, 10)),
        (Ymd::new(2028, 2, 1), Ymd::new(2028, 2, 29))
    );
    assert_eq!(
        period_bounds(PeriodKind::Quy, Ymd::new(2026, 8, 31)),
        (Ymd::new(2026, 7, 1), Ymd::new(2026, 9, 30))
    );
    assert_eq!(
        period_bounds(PeriodKind::Nam, Ymd::new(2026, 8, 31)),
        (Ymd::new(2026, 1, 1), Ymd::new(2026, 12, 31))
    );
    assert_eq!(
        prev_period(PeriodKind::Quy, Ymd::new(2026, 2, 1)),
        (Ymd::new(2025, 10, 1), Ymd::new(2025, 12, 31))
    );
    assert_eq!(
        next_period(PeriodKind::Thang, Ymd::new(2026, 12, 5)),
        (Ymd::new(2027, 1, 1), Ymd::new(2027, 1, 31))
    );
    // Mọi ngày 2026–2036: kỳ chứa ngày đó, các tuần liền nhau, 7 ngày, bắt đầu Thứ Hai.
    for j in Ymd::new(2026, 1, 1).jdn()..=Ymd::new(2036, 12, 31).jdn() {
        let d = Ymd::from_jdn(j);
        for k in PeriodKind::ALL {
            let (f, t) = period_bounds(k, d);
            assert!(f <= d && d <= t);
            assert_eq!(next_period(k, d).0.jdn(), t.jdn() + 1);
            assert_eq!(prev_period(k, d).1.jdn(), f.jdn() - 1);
        }
        let (f, t) = period_bounds(PeriodKind::Tuan, d);
        assert_eq!(t.jdn() - f.jdn(), 6);
        assert_eq!(lvhv_core::jd::weekday(f.jdn()), 1);
    }
    let n: Vec<usize> = PeriodKind::ALL
        .iter()
        .map(|&k| periods_covering(k, Ymd::new(2026, 1, 1), Ymd::new(2036, 12, 31)).len())
        .collect();
    assert_eq!(n, vec![575, 132, 44, 11]);
}

/// R13(c) trên bộ R3, mọi tháng/quý/năm 2026–2036, cả ba bộ lọc.
#[test]
fn r13c_consistency() {
    let cal = Cal::get(DEFAULT_METHOD);
    let ds = r3();
    let (lo, hi) = (Ymd::new(2026, 1, 1), Ymd::new(2036, 12, 31));
    let mut total_items = 0usize;
    for f in PersonFilter::ALL {
        for (anchor, mf, mt) in periods_covering(PeriodKind::Thang, lo, hi) {
            let month = events_in_period(cal, &ds, PeriodKind::Thang, anchor, f);
            // Tuần cắt theo tháng.
            let mut from_weeks = Vec::new();
            for (wa, _, _) in periods_covering(PeriodKind::Tuan, mf, mt) {
                let w = events_in_period(cal, &ds, PeriodKind::Tuan, wa, f);
                from_weeks.extend(clip(&w, mf, mt));
            }
            assert_eq!(from_weeks, keys(&month), "{f:?} tháng {}", mf.iso());
            let set: HashSet<_> = keys(&month).into_iter().collect();
            assert_eq!(set.len(), month.items.len(), "trùng mục {}", mf.iso());
            total_items += month.items.len();
        }
        for (qa, qf, qt) in periods_covering(PeriodKind::Quy, lo, hi) {
            let q = events_in_period(cal, &ds, PeriodKind::Quy, qa, f);
            let months: Vec<_> = periods_covering(PeriodKind::Thang, qf, qt)
                .into_iter()
                .flat_map(|(a, _, _)| keys(&events_in_period(cal, &ds, PeriodKind::Thang, a, f)))
                .collect();
            assert_eq!(months, keys(&q), "{f:?} quý {}", qf.iso());
        }
        for (ya, yf, yt) in periods_covering(PeriodKind::Nam, lo, hi) {
            let y = events_in_period(cal, &ds, PeriodKind::Nam, ya, f);
            let qs: Vec<_> = periods_covering(PeriodKind::Quy, yf, yt)
                .into_iter()
                .flat_map(|(a, _, _)| keys(&events_in_period(cal, &ds, PeriodKind::Quy, a, f)))
                .collect();
            assert_eq!(qs, keys(&y), "{f:?} năm {}", yf.iso());
            let set: HashSet<_> = keys(&y).into_iter().collect();
            assert_eq!(set.len(), y.items.len());
        }
    }
    assert!(total_items > 0);
    // Đã mất ⊎ đang sống = tất cả, giữ thứ tự, cho mọi kỳ.
    for k in PeriodKind::ALL {
        for (a, _, _) in periods_covering(k, lo, hi) {
            let all = events_in_period(cal, &ds, k, a, PersonFilter::TatCa);
            let dm = events_in_period(cal, &ds, k, a, PersonFilter::DaMat);
            let ds_ = events_in_period(cal, &ds, k, a, PersonFilter::DangSong);
            assert_eq!(dm.items.len() + ds_.items.len(), all.items.len());
            let dm_k: HashSet<_> = keys(&dm).into_iter().collect();
            let merged_dm: Vec<_> = keys(&all)
                .into_iter()
                .filter(|x| dm_k.contains(x))
                .collect();
            let merged_ds: Vec<_> = keys(&all)
                .into_iter()
                .filter(|x| !dm_k.contains(x))
                .collect();
            assert_eq!(merged_dm, keys(&dm));
            assert_eq!(merged_ds, keys(&ds_));
        }
    }
}

/// R13(d): ngày đầu và ngày cuối mỗi kỳ được tính — so với danh sách dựng trực tiếp trên khoảng rộng
/// hơn, và với bộ dữ liệu có sự kiện đúng tại các biên.
#[test]
fn r13d_first_and_last_day() {
    let cal = Cal::get(DEFAULT_METHOD);
    let ds = r3();
    let (lo, hi) = (Ymd::new(2026, 1, 1), Ymd::new(2036, 12, 31));
    let wide = occurrences_with_tt(cal, &ds, Ymd::new(2025, 12, 1), Ymd::new(2037, 1, 31));
    let mut boundary_hits = 0usize;
    for k in PeriodKind::ALL {
        for (a, f, t) in periods_covering(k, lo, hi) {
            let p = events_in_period(cal, &ds, k, a, PersonFilter::TatCa);
            let want: Vec<_> = wide
                .iter()
                .filter(|o| f <= o.date && o.date <= t)
                .map(|o| {
                    (
                        o.class.code().to_string(),
                        o.entity_id.clone(),
                        o.key.text(),
                    )
                })
                .collect();
            assert_eq!(keys(&p), want, "{k:?} {}", f.iso());
            boundary_hits += p
                .items
                .iter()
                .filter(|i| i.occ.date == f || i.occ.date == t)
                .count();
        }
    }
    assert!(boundary_hits > 0);

    // Sự kiện một lần đúng tại biên năm/quý/tháng/tuần.
    let ev = |id: &str, y: i32, m: u32, d: u32| Event {
        id: id.to_string(),
        type_id: "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0001".into(),
        tieu_de: format!("Biên {y}-{m}-{d}"),
        date: DateValue::solar(y, m, d),
        recurring: false,
        person_ids: Vec::new(),
        reminder: None,
        rev: 1,
    };
    let syn = Dataset {
        reminders_enabled: true,
        persons: Vec::new(),
        events: vec![
            ev("e1", 2027, 1, 1),   // đầu năm, quý, tháng; Thứ Sáu
            ev("e2", 2027, 3, 31),  // cuối quý 1, cuối tháng 3
            ev("e3", 2027, 12, 31), // cuối năm
            ev("e4", 2027, 6, 7),   // Thứ Hai
            ev("e5", 2027, 6, 13),  // Chủ nhật
        ],
        event_types: Vec::new(),
    };
    let has = |k: PeriodKind, a: Ymd, id: &str| {
        events_in_period(cal, &syn, k, a, PersonFilter::TatCa)
            .items
            .iter()
            .any(|i| i.occ.entity_id == id)
    };
    assert!(has(PeriodKind::Nam, Ymd::new(2027, 7, 1), "e1"));
    assert!(has(PeriodKind::Nam, Ymd::new(2027, 7, 1), "e3"));
    assert!(has(PeriodKind::Quy, Ymd::new(2027, 2, 1), "e1"));
    assert!(has(PeriodKind::Quy, Ymd::new(2027, 2, 1), "e2"));
    assert!(!has(PeriodKind::Quy, Ymd::new(2027, 4, 1), "e2"));
    assert!(has(PeriodKind::Thang, Ymd::new(2027, 3, 15), "e2"));
    assert!(has(PeriodKind::Tuan, Ymd::new(2027, 6, 10), "e4"));
    assert!(has(PeriodKind::Tuan, Ymd::new(2027, 6, 10), "e5"));
    assert!(!has(PeriodKind::Tuan, Ymd::new(2027, 6, 14), "e5"));
    assert!(!has(PeriodKind::Tuan, Ymd::new(2027, 6, 6), "e4"));
}

/// Quy tắc lọc sự kiện (02 §7.6): đã mất ⇔ person_ids không rỗng và mọi người có ngày mất.
#[test]
fn filter_rule_for_events() {
    let cal = Cal::get(DEFAULT_METHOD);
    let mut ds = r3();
    let dead = ds
        .persons
        .iter()
        .find(|p| p.death.is_some())
        .unwrap()
        .id
        .clone();
    let dead2 = ds
        .persons
        .iter()
        .filter(|p| p.death.is_some())
        .nth(1)
        .unwrap()
        .id
        .clone();
    let alive = ds
        .persons
        .iter()
        .find(|p| p.death.is_none())
        .unwrap()
        .id
        .clone();
    ds.events[0].person_ids = vec![dead.clone()];
    ds.events[1].person_ids = vec![dead.clone(), alive.clone()];
    ds.events[2].person_ids = vec![dead, dead2];
    ds.events[3].person_ids = vec![alive];
    let e: Vec<String> = ds.events.iter().map(|e| e.id.clone()).collect();
    let y = events_in_period(
        cal,
        &ds,
        PeriodKind::Nam,
        Ymd::new(2027, 6, 1),
        PersonFilter::DaMat,
    );
    let in_dm = |id: &str| y.items.iter().any(|i| i.occ.entity_id == id);
    assert!(in_dm(&e[0]));
    assert!(!in_dm(&e[1]));
    assert!(in_dm(&e[2]));
    assert!(!in_dm(&e[3]));
    assert!(!in_dm(&e[4])); // không gắn ai → đang sống
    let it = y.items.iter().find(|i| i.occ.entity_id == e[2]).unwrap();
    assert_eq!(it.person_ids.len(), 2);
    // Mục của hồ sơ: person_ids = [id], tên hiển thị, ngày âm = S2L(ngày).
    for i in &y.items {
        if !i.occ.class.is_event() {
            assert_eq!(i.person_ids, vec![i.occ.entity_id.clone()]);
        }
        let l =
            lvhv_core::solar_to_lunar_with(cal.method(), i.occ.date.y, i.occ.date.m, i.occ.date.d)
                .unwrap();
        let li = i.lunar.unwrap();
        assert_eq!(
            (li.y, li.m, li.d, li.leap),
            (l.year, l.month, l.day, l.leap)
        );
    }
}
