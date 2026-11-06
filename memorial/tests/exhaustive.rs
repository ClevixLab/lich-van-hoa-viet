//! Kiểm vét cạn: bảng tháng `Cal` ≡ `lvhv_core::lunar_to_jdn_with`; không gian R3 (04);
//! bất biến R12 (I1, I2); R10(a)(b)(d) so với hàm lõi.

use lvhv_core::{lunar_to_jdn_with, Method};
use lvhv_memorial::dump::*;
use lvhv_memorial::model::*;
use lvhv_memorial::occ::{OccClass, RemType};
use lvhv_memorial::remind::*;
use lvhv_memorial::Cal;
use std::collections::HashSet;

#[test]
fn cal_matches_core_every_lunar_day() {
    for method in [Method::Precise, Method::Hnd] {
        let cal = Cal::get(method);
        let mut n = 0u64;
        for ly in 1899..=3000 {
            for m in 1..=12 {
                for leap in [false, true] {
                    for d in 0..=31 {
                        assert_eq!(
                            cal.l2s_jdn(ly, m, d, leap),
                            lunar_to_jdn_with(method, ly, m, d, leap),
                            "{method:?} {ly}-{m}-{d} leap={leap}"
                        );
                        n += 1;
                    }
                }
            }
        }
        assert_eq!(n, 1102 * 12 * 2 * 32);
    }
}

fn ds() -> Dataset {
    parse_dataset(include_str!("../testdata/r3-dataset.json")).unwrap()
}

#[test]
fn r3_space_shape() {
    let st = ui_states();
    assert_eq!(st.len(), 3904);
    let distinct: HashSet<Vec<u32>> = st.iter().map(|s| s.offsets()).collect();
    assert_eq!(distinct.len(), 3584);
    for s in &st {
        let o = s.offsets();
        assert!(o.windows(2).all(|w| w[0] < w[1]));
        assert!(o.len() <= 7 && o.iter().all(|&k| k <= 60));
    }
    assert_eq!(r3_configs().len(), 140_544);
}

/// R3 tự kiểm (không thay bộ tham chiếu độc lập): mỗi cấu hình cho đúng số lời nhắc tính trực tiếp
/// từ định nghĩa 02 §7.1, mọi `at > now`, id duy nhất, tiên thường chỉ ở giỗ.
#[test]
fn r3_counts_match_definition() {
    let ds = ds();
    assert_eq!(ds.persons.len() + ds.events.len(), 40);
    let cal = lvhv_memorial::gio::default_cal();
    let tz = Zone::vietnam();
    let now = r3_now(&tz);
    let (f, t) = (Ymd::new(2026, 1, 1), Ymd::new(2036, 12, 31));
    let occ: Vec<(OccClass, Vec<_>)> = OccClass::R3
        .iter()
        .map(|&c| (c, class_occurrences(cal, &ds, c, f, t)))
        .collect();
    let mut total = 0usize;
    for c in r3_configs() {
        let occs = &occ.iter().find(|x| x.0 == c.class).unwrap().1;
        let r = r3_reminders(&ds, occs, &c, now, &tz);
        let mut want = 0usize;
        for o in occs
            .iter()
            .filter(|o| lvhv_memorial::occ::rule_of(&ds, o).is_some())
        {
            let d = o.date.jdn();
            for k in c.state.offsets() {
                if (d - i64::from(k)) * 1440 + i64::from(c.time_local) - 420 > now {
                    want += 1;
                }
            }
            if c.tien_thuong
                && c.class == OccClass::Gio
                && (d - 1) * 1440 + i64::from(c.time_local) - 420 > now
            {
                want += 1;
            }
        }
        assert_eq!(r.len(), want, "cfg {}", c.index);
        assert!(r.iter().all(|x| x.at > now));
        assert!(c.class == OccClass::Gio || r.iter().all(|x| x.rtype != RemType::TienThuong));
        let ids: HashSet<&str> = r.iter().map(|x| x.id.as_str()).collect();
        assert_eq!(ids.len(), r.len());
        total += r.len();
    }
    assert_eq!(total, 38_432_256);
}

/// R12 I3 (04 Bổ sung B2 mục 1): cái thứ 63 và thứ 64 cùng phút → cái thứ 64 bị bỏ nhưng thông báo
/// nhắc mở ứng dụng không muộn hơn nó.
#[test]
fn reschedule_same_minute_63_64() {
    let mut all: Vec<Pending> = (0..62)
        .map(|i| Pending {
            id: format!("lvhv:{i:08x}-0000-4000-8000-000000000000:2026:0"),
            at: 1000 + i,
            content: String::new(),
        })
        .collect();
    for k in 0..3 {
        all.push(Pending {
            id: format!("lvhv:ffffffff-0000-4000-8000-00000000000{k}:2026:0"),
            at: 5000,
            content: String::new(),
        });
    }
    let plan = reschedule(0, &all, &[]);
    assert_eq!(plan.keep.len(), 64);
    let s = plan.keep.iter().find(|p| p.id == SENTINEL_ID).unwrap();
    assert_eq!(s.at, 5000);
    let dropped: Vec<&Pending> = all
        .iter()
        .filter(|a| !plan.keep.iter().any(|k| k.id == a.id))
        .collect();
    assert_eq!(dropped.len(), 2);
    assert!(dropped.iter().all(|d| d.at >= s.at));
    let q = apply_plan(&[], &plan);
    assert_eq!(q.last().unwrap().id, SENTINEL_ID);
}

#[test]
fn reschedule_basics() {
    let p = |i: i64| Pending {
        id: format!("lvhv:x:{i:03}"),
        at: 1000 + i,
        content: String::new(),
    };
    let all: Vec<Pending> = (0..100).map(p).collect();
    let plan = reschedule(0, &all, &[]);
    assert_eq!(plan.keep.len(), 64);
    assert_eq!(plan.keep[63].id, SENTINEL_ID);
    // 02 §7.4 (Bổ sung B2): cùng phút với cái thứ 63, không + 1 phút.
    assert_eq!(plan.keep[63].at, plan.keep[62].at);
    let q = apply_plan(&[], &plan);
    // Lần sau: không đổi gì.
    let plan2 = reschedule(0, &all, &q);
    assert!(plan2.add.is_empty() && plan2.remove.is_empty() && plan2.replace.is_empty());
    // ≤ 64 → không có thông báo nhắc mở ứng dụng.
    let plan3 = reschedule(0, &all[..64], &q);
    assert_eq!(plan3.keep.len(), 64);
    assert!(plan3.keep.iter().all(|k| k.id != SENTINEL_ID));
    assert!(plan3.remove.contains(&SENTINEL_ID.to_string()));
    // Ngoài cửa sổ 400 ngày và ≤ now bị loại.
    let far = Pending {
        id: "far".into(),
        at: 401 * 1440,
        content: String::new(),
    };
    assert!(reschedule(0, std::slice::from_ref(&far), &[])
        .keep
        .is_empty());
    assert!(reschedule(1000, &[p(0)], &[]).keep.is_empty());
}

/// R12: bất biến I1 và I2 (02 §7.4 đúng như viết) ở 3 cỡ × 3 kịch bản. I3 (thông báo nhắc mở ứng
/// dụng đứng trước mọi lời nhắc bị bỏ) **không** được bảo đảm bởi 02 §7.4 khi lời nhắc thứ 63 và
/// 64 cùng phút — xem mục mở trong mô tả PR; không kiểm ở đây để không che kết quả của dump R12.
#[test]
fn r12_i1_i2() {
    let base = ds();
    let cal = lvhv_memorial::gio::default_cal();
    let tz = Zone::vietnam();
    for n in [10, 50, 200] {
        let d = scaled_dataset(&base, n);
        for (_, pn, pd) in R12_SCENARIOS {
            let r = simulate(
                cal,
                &d,
                &tz,
                Ymd::new(2026, 1, 1),
                Ymd::new(2028, 12, 31),
                540,
                pn,
                pd,
                20_261_106,
                false,
            );
            assert!(r.max_pending <= CAP);
            for s in &r.steps {
                assert!(
                    !s.violations.contains(&"I1") && !s.violations.contains(&"I2"),
                    "{n} {s:?}"
                );
            }
        }
    }
}

#[test]
fn r10_against_core() {
    let cal = lvhv_memorial::gio::default_cal();
    let mut out = String::new();
    for j in Ymd::new(1900, 1, 1).jdn()..=Ymd::new(2026, 12, 31).jdn() {
        let d = Ymd::from_jdn(j);
        out.clear();
        r10a_line(cal, &mut out, d);
        let ly = lvhv_core::lunar_year_of_birth(d.y, d.m, d.d).unwrap();
        assert!(out.starts_with(&format!("{},{ly},", d.iso())), "{out}");
    }
    out.clear();
    r10c_lines(cal, &mut out, 10_000, 20_261_106);
    assert_eq!(out.lines().count(), 10_000);
}
