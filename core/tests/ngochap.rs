//! Tích hợp Ngọc Hạp: `day_info.ngoc_hap`, `find_good_days`, tiêu chí N3.
use lvhv_core::dayinfo::{day_ctx_jdn_with, ngochap_ctx};
use lvhv_core::gooddays::evaluate_day_for;
use lvhv_core::jd::{date_from_jdn, jdn_from_date};
use lvhv_core::*;

fn sd(year: i32, month: u32, day: u32) -> SolarDate {
    SolarDate { year, month, day }
}

fn sorted(v: &[String]) -> Vec<&str> {
    let mut x: Vec<&str> = v.iter().map(String::as_str).collect();
    x.sort_unstable();
    x
}

/// Đối chiếu trang lịch công khai (content/ngochap/rules.md §5) qua lõi thật (không qua ctx tay).
#[test]
fn fixture_2026_11_05_via_core() {
    let n = day_info(2026, 11, 5).unwrap().ngoc_hap;
    let mut want = vec!["Đại Hồng Sa", "Sát Cống", "Thiên Ân", "U Vi Tinh"];
    want.sort_unstable();
    assert_eq!(sorted(&n.sao_tot), want);
    let mut want = vec![
        "Tam Nương",
        "Băng Tiêu Ngọa Hãm",
        "Chu Tước Hắc Đạo",
        "Địa Phá",
        "Hà Khôi (Cẩu Giảo)",
        "Hoang Vu",
        "Ngũ Hư",
        "Nguyệt Hình",
        "Tứ Thời Cô Quả",
    ];
    want.sort_unstable();
    assert_eq!(sorted(&n.sao_xau), want);
    assert_eq!(n.xung_tuoi, ["Ất Sửu", "Tân Sửu", "Đinh Hợi", "Đinh Tỵ"]);
    assert_eq!(n.score, -6);
}

#[test]
fn fixture_2026_11_08_and_10_via_core() {
    let n = day_info(2026, 11, 8).unwrap().ngoc_hap;
    let mut w = vec!["Thiên Đức", "Nguyệt Đức", "Thiên Mã", "Mãn Đức Tinh"];
    w.sort_unstable();
    assert_eq!(sorted(&n.sao_tot), w);
    assert_eq!(
        n.xung_tuoi,
        ["Mậu Thìn", "Nhâm Thìn", "Nhâm Ngọ", "Nhâm Tý"]
    );
    let n = day_info(2026, 11, 10).unwrap().ngoc_hap;
    let mut w = vec!["Thiên Mã", "U Vi Tinh", "Yếu Yên"];
    w.sort_unstable();
    assert_eq!(sorted(&n.sao_tot), w);
    assert_eq!(n.xung_tuoi, ["Bính Ngọ", "Giáp Ngọ"]);
}

#[test]
fn ctx_mapping_fields() {
    let c = ngochap_ctx(&day_ctx(2026, 11, 5).unwrap()).unwrap();
    assert_eq!((c.lunar_month, c.lunar_day, c.leap), (9, 27, false));
    assert_eq!((c.day_can, c.day_chi, c.truc, c.than), (9, 7, 9, 3));
    assert_eq!(
        (c.month_chi_tiet, c.year_can, c.year_chi, c.tu),
        (10, 2, 6, 21)
    );
    assert_eq!((c.solar_month, c.solar_day), (11, 5));
}

#[test]
fn activities_exposed() {
    let a = activities();
    assert_eq!(a.len(), 16);
    assert_eq!(a[0].id, "cuoi_hoi");
}

#[test]
fn find_good_days_sorted_and_allowed() {
    for act in activities() {
        for birth in [None, Some(1976), Some(1985)] {
            let r = find_good_days(act.id, sd(2026, 1, 1), sd(2026, 12, 31), birth).unwrap();
            assert!(!r.is_empty(), "{} không có ngày nào", act.id);
            for w in r.windows(2) {
                assert!(
                    w[0].score > w[1].score || (w[0].score == w[1].score && w[0].jdn < w[1].jdn),
                    "thứ tự sai"
                );
            }
            for g in &r {
                assert!(g.allowed);
                let s = lvhv_ngochap::suitability(
                    &ngochap_ctx(&day_ctx(g.solar.year, g.solar.month, g.solar.day).unwrap())
                        .unwrap(),
                    act.id,
                    birth,
                );
                assert_eq!((s.score, s.allowed), (g.score, true));
                assert_eq!(
                    g.first_gio_hoang_dao,
                    day_info(g.solar.year, g.solar.month, g.solar.day)
                        .unwrap()
                        .gio_hoang_dao[0]
                );
            }
        }
    }
}

#[test]
fn find_good_days_edge_cases() {
    assert_eq!(
        find_good_days("khong_co", sd(2026, 1, 1), sd(2026, 1, 31), None),
        Err(DateError::UnknownActivity)
    );
    assert_eq!(
        find_good_days("cuoi_hoi", sd(2026, 2, 1), sd(2026, 1, 1), None),
        Err(DateError::InvertedRange)
    );
    // 2026-11-05: Tam Nương là kỵ chính của cưới hỏi → không xuất hiện.
    let r = find_good_days("cuoi_hoi", sd(2026, 11, 5), sd(2026, 11, 5), None).unwrap();
    assert!(r.is_empty());
    assert!(
        !evaluate_day_for(Method::Hnd, jdn_from_date(2026, 11, 5), "cuoi_hoi", None)
            .unwrap()
            .allowed
    );
    // 1985 (Ất Sửu) xung ngày Quý Mùi: lý do ghi rõ tuổi (tế tự không bị chặn vì tuổi).
    let g = evaluate_day_for(Method::Hnd, jdn_from_date(2026, 11, 5), "te_tu", Some(1985)).unwrap();
    assert!(g.reasons.iter().any(|r| r.contains("Ất Sửu")));
}

#[test]
fn gooddays_json_shape() {
    let g = evaluate_day_for(Method::Hnd, jdn_from_date(2026, 11, 6), "cuoi_hoi", None).unwrap();
    let j = g.to_json();
    assert!(j.starts_with(r#"{"solar":{"y":2026,"m":11,"d":6},"lunar":{"y":2026,"m":9,"d":28,"leap":false,"month_len":30},"weekday":5,"jdn":2461351,"score":"#));
    assert!(j.ends_with(r#""first_gio_hoang_dao":{"chi":"Tý","from":23,"to":1}}"#));
}

/// Tiêu chí N3 trên toàn bộ 2000–3000: mọi ngày × mọi việc × (không tuổi, tuổi 1976), cả hai phương
/// pháp. Định nghĩa "sao kỵ chính" lấy từ chính crate Ngọc Hạp (`major_taboos`).
/// Chậm ở bản debug: `cargo test --release -p lvhv-core --test ngochap -- --ignored`.
fn n3_full(method: Method) -> (u64, u64, u64) {
    let a = jdn_from_date(2000, 1, 1);
    let b = jdn_from_date(3000, 12, 31);
    let (mut checked, mut taboo_days, mut violations) = (0u64, 0u64, 0u64);
    for j in a..=b {
        let ctx = ngochap_ctx(&day_ctx_jdn_with(method, j).unwrap()).unwrap();
        for act in activities() {
            let has_major = !lvhv_ngochap::major_taboos(&ctx, act.id).is_empty();
            if has_major {
                taboo_days += 1;
            }
            for birth in [None, Some(1976)] {
                checked += 1;
                let s = lvhv_ngochap::suitability(&ctx, act.id, birth);
                if has_major && s.allowed {
                    violations += 1;
                    let (y, m, d) = date_from_jdn(j);
                    eprintln!("N3 vi phạm: {y}-{m}-{d} {}", act.id);
                }
            }
        }
    }
    (checked, taboo_days, violations)
}

#[test]
#[ignore = "chậm ở bản debug; chạy: cargo test --release -p lvhv-core --test ngochap -- --ignored"]
fn n3_all_days_2000_3000() {
    for method in [Method::Hnd, Method::Precise] {
        let (checked, taboo, viol) = n3_full(method);
        eprintln!("N3 {method:?}: {checked} lượt (ngày×việc×tuổi), {taboo} cặp ngày×việc phạm kỵ chính, {viol} vi phạm");
        assert_eq!(viol, 0);
    }
}

/// N3 qua `find_good_days` (đường người dùng thật), 2000–3000, mọi việc, cả hai phương pháp.
#[test]
#[ignore = "chậm ở bản debug; chạy: cargo test --release -p lvhv-core --test ngochap -- --ignored"]
fn n3_find_good_days_2000_3000() {
    for method in [Method::Hnd, Method::Precise] {
        let mut n = 0usize;
        for act in activities() {
            for birth in [None, Some(1976)] {
                // Lõi giới hạn khoảng ≤ MAX_SPAN_DAYS (RS-13): duyệt 2000–3000 theo khúc liền nhau.
                let (mut a, end) = (jdn_from_date(2000, 1, 1), jdn_from_date(3000, 12, 31));
                while a <= end {
                    let b = (a + MAX_SPAN_DAYS).min(end);
                    let (fa, fb) = (date_from_jdn(a), date_from_jdn(b));
                    let list = find_good_days_with(
                        method,
                        act.id,
                        sd(fa.0, fa.1, fa.2),
                        sd(fb.0, fb.1, fb.2),
                        birth,
                    )
                    .unwrap();
                    for g in list {
                        n += 1;
                        let ctx = ngochap_ctx(&day_ctx_jdn_with(method, g.jdn).unwrap()).unwrap();
                        let m = lvhv_ngochap::major_taboos(&ctx, act.id);
                        assert!(m.is_empty(), "{:?} {} {:?}", g.solar, act.id, m);
                    }
                    a = b + 1;
                }
            }
        }
        eprintln!("N3 find_good_days {method:?}: {n} ngày được đề xuất, 0 vi phạm");
    }
}

/// Quy tắc đề xuất (SCORING.md §2a) trên các ngày `[a, b]`: mọi ngày được đề xuất có việc ∉ `ky`
/// của tờ lịch, điểm việc ≥ 0, không phạm kỵ chính. Trả số ngày đề xuất.
fn check_suggestions(method: Method, a: i64, b: i64, birth: Option<i32>) -> usize {
    let mut n = 0;
    let mut j = a;
    while j <= b {
        let e = (j + MAX_SPAN_DAYS).min(b);
        let (fa, fb) = (date_from_jdn(j), date_from_jdn(e));
        for act in activities() {
            let list = find_good_days_with(
                method,
                act.id,
                sd(fa.0, fa.1, fa.2),
                sd(fb.0, fb.1, fb.2),
                birth,
            )
            .unwrap();
            for g in list {
                n += 1;
                let s = g.solar;
                let sheet = day_info_with(method, s.year, s.month, s.day).unwrap();
                assert!(
                    !sheet.ngoc_hap.ky.iter().any(|k| k == act.name),
                    "{s:?} {}: tờ lịch ghi Kỵ",
                    act.id
                );
                assert!(
                    g.score >= MIN_GOOD_DAY_SCORE,
                    "{s:?} {}: điểm {}",
                    act.id,
                    g.score
                );
                assert!(g.allowed && !g.in_sheet_ky);
            }
        }
        j = e + 1;
    }
    n
}

/// Bản nhanh (chạy mặc định): 2026, cả hai phương pháp, có/không tuổi.
#[test]
fn suggested_days_consistent_with_sheet_2026() {
    let (a, b) = (jdn_from_date(2026, 1, 1), jdn_from_date(2026, 12, 31));
    for method in [Method::Hnd, Method::Precise] {
        for birth in [None, Some(1975)] {
            assert!(check_suggestions(method, a, b, birth) > 1000);
        }
    }
}

/// Bản toàn dải 2000–3000 (chạy một lần ở bản release: `--ignored`).
#[test]
#[ignore = "chậm; chạy: cargo test --release -p lvhv-core --test ngochap -- --ignored"]
fn suggested_days_consistent_with_sheet_2000_3000() {
    let (a, b) = (jdn_from_date(2000, 1, 1), jdn_from_date(3000, 12, 31));
    for method in [Method::Hnd, Method::Precise] {
        let n = check_suggestions(method, a, b, None);
        eprintln!("§2a {method:?}: {n} ngày đề xuất 2000–3000, 0 vi phạm");
    }
}
