//! Kiểm tra theo phương pháp: tự nhất quán tháng nhuận ↔ tiết khí hiển thị; fixture cho Precise.
use lvhv_core::jd::{date_from_jdn, jdn_from_date};
use lvhv_core::lunar::YearTable;
use lvhv_core::terms::term_index_of_day;
use lvhv_core::*;

/// Các tháng nhuận (năm âm, tháng, ngày đầu ISO) chứa ít nhất một Trung khí theo tiết khí
/// HIỂN THỊ (`term_index_of_day`), xét mọi tuế có tháng nằm trong 2000–3000.
fn leap_months_with_trung_khi(method: Method) -> Vec<(i32, u32, String)> {
    let lo = jdn_from_date(2000, 1, 1);
    let hi = jdn_from_date(3000, 12, 31);
    let mut bad = Vec::new();
    let mut checked = 0;
    for yy in 2000..=3001 {
        let t = YearTable::new(method, yy).unwrap();
        for i in 0..t.n {
            if !t.leap[i] {
                continue;
            }
            let (s, e) = (t.starts[i], t.starts[i + 1]);
            if e <= lo || s > hi {
                continue;
            }
            checked += 1;
            // Trung khí bắt đầu vào ngày j ⇔ chỉ số tiết khí của j chẵn và khác ngày j−1.
            let has = (s..e).any(|j| {
                let a = term_index_of_day(j);
                a.is_multiple_of(2) && a != term_index_of_day(j - 1)
            });
            if has {
                let (y, m, d) = date_from_jdn(s);
                bad.push((
                    t.lunar_year(i).unwrap(),
                    t.month[i],
                    format!("{y:04}-{m:02}-{d:02}"),
                ));
            }
        }
    }
    assert!(checked > 350, "số tháng nhuận đã xét quá ít: {checked}");
    bad
}

#[test]
#[ignore = "chạy bằng: cargo test --release -- --ignored"]
fn self_consistency_precise() {
    let bad = leap_months_with_trung_khi(Method::Precise);
    assert!(
        bad.is_empty(),
        "Precise: tháng nhuận chứa Trung khí: {bad:?}"
    );
}

/// HND KHÔNG tự nhất quán (hai mô hình Mặt Trời khác nhau). Khóa danh sách vi phạm đã biết để
/// phát hiện mọi thay đổi ngoài ý muốn.
#[test]
#[ignore = "chạy bằng: cargo test --release -- --ignored"]
fn self_consistency_hnd_known_violations() {
    let bad = leap_months_with_trung_khi(Method::Hnd);
    println!("HND vi phạm: {bad:?}");
    let years: Vec<i32> = bad.iter().map(|b| b.0).collect();
    assert_eq!(years, HND_VIOLATION_YEARS, "{bad:?}");
}

/// Năm âm có tháng nhuận vi phạm (khớp 6 đoạn "nguyên nhân khác" của báo cáo kiểm định).
const HND_VIOLATION_YEARS: [i32; 6] = [2405, 2682, 2728, 2758, 2864, 2948];

#[test]
fn precise_fixture_2026_11_05() {
    let i = day_info_with(Method::Precise, 2026, 11, 5).unwrap();
    assert_eq!(i.to_json(), day_info(2026, 11, 5).unwrap().to_json());
}

#[test]
fn precise_tet_known() {
    for (ly, (y, m, d)) in [
        (2000, (2000, 2, 5)),
        (2007, (2007, 2, 17)),
        (2023, (2023, 1, 22)),
        (2024, (2024, 2, 10)),
        (2025, (2025, 1, 29)),
        (2026, (2026, 2, 17)),
        (2030, (2030, 2, 2)),
        (2053, (2053, 2, 18)),
    ] {
        let t = tet_with(Method::Precise, ly).unwrap();
        assert_eq!((t.year, t.month, t.day), (y, m, d), "Tết {ly}");
    }
}

#[test]
#[ignore = "chạy bằng: cargo test --release -- --ignored"]
fn precise_roundtrip_2000_3000() {
    let mut prev: Option<LunarDate> = None;
    for j in jdn_from_date(2000, 1, 1)..=jdn_from_date(3000, 12, 31) {
        let (y, m, d) = date_from_jdn(j);
        let l = solar_to_lunar_with(Method::Precise, y, m, d).unwrap();
        let s = lunar_to_solar_with(Method::Precise, l.year, l.month, l.day, l.leap).unwrap();
        assert_eq!((s.year, s.month, s.day), (y, m, d), "{l:?}");
        if let Some(p) = prev {
            if l.day != 1 {
                assert_eq!(
                    (l.year, l.month, l.leap, l.day),
                    (p.year, p.month, p.leap, p.day + 1)
                );
            } else {
                assert_eq!(p.day, p.month_len);
            }
        }
        prev = Some(l);
    }
}
