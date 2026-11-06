//! Đầu vào xấu (RS-01/02/03/04/05/13, DUYET-MA §3): mọi hàm công khai phải kết thúc nhanh và
//! trả lỗi rõ ràng (`Err`/`None`) — không treo, không panic, không trả kết quả vô nghĩa.
//! Mỗi lời gọi chạy trong luồng riêng có hạn giờ, nên một vòng lặp vô hạn làm test THẤT BẠI
//! thay vì làm treo cả bộ test.
use lvhv_core::jd::jdn_from_date;
use lvhv_core::lunar::{new_moons_between, YearTable};
use lvhv_core::*;
use std::sync::mpsc;
use std::time::Duration;

/// Chạy `f` với hạn giờ; panic (test thất bại) nếu quá hạn hoặc `f` panic.
fn within<T: Send + 'static>(what: &str, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(v) => v,
        Err(mpsc::RecvTimeoutError::Timeout) => panic!("{what}: TREO (quá 5 s)"),
        Err(mpsc::RecvTimeoutError::Disconnected) => panic!("{what}: PANIC"),
    }
}

fn sd(year: i32, month: u32, day: u32) -> SolarDate {
    SolarDate { year, month, day }
}

const OOR: DateError = DateError::OutOfRange {
    min: MIN_YEAR,
    max: MAX_YEAR,
};

/// Năm ngoài miền: cực trị, 0, âm, sát biên, rất lớn.
const BAD_YEARS: [i32; 12] = [
    i32::MIN,
    -1_000_000,
    -14_621,
    -1,
    0,
    1,
    1899,
    3001,
    35_650,
    1_000_000,
    100_000,
    i32::MAX,
];

#[test]
fn solar_years_out_of_range_are_errors() {
    for y in BAD_YEARS {
        for method in [Method::Hnd, Method::Precise] {
            let r = within("solar_to_lunar", move || {
                solar_to_lunar_with(method, y, 1, 1)
            });
            assert_eq!(r, Err(OOR), "solar_to_lunar({y},1,1) {method:?}");
            let r = within("day_info", move || {
                day_info_with(method, y, 12, 31).map(|_| ())
            });
            assert_eq!(r, Err(OOR), "day_info({y},12,31) {method:?}");
            let r = within("day_ctx", move || {
                day_ctx_with(method, y, 6, 15).map(|_| ())
            });
            assert_eq!(r, Err(OOR), "day_ctx({y}) {method:?}");
            let r = within("tet", move || tet_with(method, y));
            assert_eq!(r, Err(OOR), "tet({y}) {method:?}");
            let r = within("lunar_to_solar", move || {
                lunar_to_solar_with(method, y, 1, 1, false)
            });
            // 1899 là năm âm hợp lệ cho tháng 12 (chứa 1900-01-01), nhưng tháng Giêng 1899 thì
            // rơi vào 1899 → ngoài miền → None.
            assert_eq!(r, None, "lunar_to_solar({y},1,1) {method:?}");
        }
        let r = within("solar_terms_of_year", move || {
            solar_terms_of_year(y).map(|v| v.len())
        });
        assert_eq!(r, Err(OOR), "solar_terms_of_year({y})");
    }
}

#[test]
fn jdn_extremes_are_errors() {
    for j in [
        i64::MIN,
        -1_000_000_000,
        -1,
        0,
        MIN_JDN - 1,
        MAX_JDN + 1,
        1_000_000_000,
        9_223_372_036_854_775_807,
    ] {
        for method in [Method::Hnd, Method::Precise] {
            let r = within("jdn_to_lunar", move || jdn_to_lunar_with(method, j));
            assert_eq!(r, Err(OOR), "jdn_to_lunar({j}) {method:?}");
            let r = within("day_ctx_jdn", move || {
                day_ctx_jdn_with(method, j).map(|_| ())
            });
            assert_eq!(r, Err(OOR), "day_ctx_jdn({j}) {method:?}");
            let r = within("evaluate_day_for", move || {
                gooddays::evaluate_day_for(method, j, "cuoi_hoi", None).map(|_| ())
            });
            assert_eq!(r, Err(OOR), "evaluate_day_for({j}) {method:?}");
        }
    }
    assert_eq!(
        new_moons_between(Method::Hnd, 0, MAX_JDN),
        Err(OOR),
        "new_moons_between"
    );
    assert_eq!(
        new_moons_between(Method::Hnd, MAX_JDN, MIN_JDN),
        Err(DateError::InvertedRange)
    );
}

#[test]
fn edges_of_domain_work() {
    for method in [Method::Hnd, Method::Precise] {
        // 1900-01-01 là mùng 1 tháng 12 năm âm 1899 (Sóc 1900-01-01 21:51 UTC+8; bảng TK20
        // của Hồ Ngọc Đức cũng cho Tết 1900 = 1900-01-31, tức tháng 12/1899 dài 30 ngày).
        let a = solar_to_lunar_with(method, 1900, 1, 1).unwrap();
        assert_eq!((a.year, a.month, a.day, a.leap), (1899, 12, 1, false));
        let b = solar_to_lunar_with(method, 3000, 12, 31).unwrap();
        assert!(b.year == 3000 && b.month >= 11, "{b:?}");
        // Năm âm 1899 tháng 12 (nằm trong 1900) đổi ngược được.
        let s = lunar_to_solar_with(method, a.year, a.month, a.day, a.leap).unwrap();
        assert_eq!((s.year, s.month, s.day), (1900, 1, 1));
        let s = lunar_to_solar_with(method, b.year, b.month, b.day, b.leap).unwrap();
        assert_eq!((s.year, s.month, s.day), (3000, 12, 31));
        // Tháng 11 âm 1899 nằm trước 1900-01-01 → None (kết quả ngoài miền).
        assert_eq!(lunar_to_solar_with(method, 1899, 11, 1, false), None);
        assert_eq!(lunar_to_solar_with(method, 1899, 11, 30, false), None);
        // Tết 1900 và 3000 trong miền; Tết 1899 ngoài miền.
        assert!(tet_with(method, 1900).is_ok() && tet_with(method, 3000).is_ok());
        assert_eq!(tet_with(method, 1899), Err(OOR));
        // Ranh giới cũ 2000 vẫn liền mạch: 2000-01-01 = 25/11/1999.
        let c = solar_to_lunar_with(method, 2000, 1, 1).unwrap();
        assert_eq!((c.year, c.month, c.day), (1999, 11, 25));
        assert!(day_info_with(method, 1900, 1, 1).is_ok());
        assert!(day_info_with(method, 2000, 1, 1).is_ok());
        assert!(day_info_with(method, 3000, 12, 31).is_ok());
    }
    let ts = solar_terms_of_year(3000).unwrap();
    assert_eq!(ts.len(), 24);
    assert_eq!(solar_terms_of_year(1900).unwrap().len(), 24);
}

#[test]
fn invalid_solar_dates_are_errors_not_panics() {
    let cases: [(i32, u32, u32); 12] = [
        (2026, 2, 29), // 2026 không nhuận
        (2100, 2, 29), // 2100 không nhuận (chia hết 100)
        (2026, 2, 30),
        (2026, 4, 31),
        (2026, 13, 1),
        (2026, 0, 1),
        (2026, 1, 0),
        (2026, 1, 32),
        (2026, u32::MAX, 1),
        (2026, 1, u32::MAX),
        (i32::MAX, 13, 1), // không hợp lệ được báo trước ngoài miền
        (0, 2, 30),
    ];
    for (y, m, d) in cases {
        let want = Err(DateError::InvalidDate {
            year: y,
            month: m,
            day: d,
        });
        assert_eq!(
            within("s2l", move || solar_to_lunar(y, m, d)),
            want,
            "{y}-{m}-{d}"
        );
        assert_eq!(
            within("day_info", move || day_info(y, m, d).map(|_| ())),
            want.map(|_: LunarDate| ()),
            "{y}-{m}-{d}"
        );
        assert_eq!(
            within("fgd", move || find_good_days(
                "cuoi_hoi",
                sd(y, m, d),
                sd(2026, 3, 1),
                None
            )
            .map(|_| ())),
            want.map(|_: LunarDate| ()),
            "{y}-{m}-{d}"
        );
    }
    // 29/2 năm nhuận là hợp lệ.
    assert!(solar_to_lunar(2024, 2, 29).is_ok());
    assert!(solar_to_lunar(2000, 2, 29).is_ok());
}

#[test]
fn impossible_lunar_dates_are_none() {
    // Tháng nhuận không tồn tại trong năm (2026 không có nhuận; 2025 nhuận 6 chứ không nhuận 5).
    assert_eq!(lunar_to_solar(2026, 5, 1, true), None);
    assert_eq!(lunar_to_solar(2025, 5, 1, true), None);
    assert!(lunar_to_solar(2025, 6, 1, true).is_some());
    // Ngày 30 của tháng thiếu (29 ngày): tìm một tháng thiếu thật rồi thử ngày 30.
    let l = solar_to_lunar(2026, 2, 17).unwrap(); // mùng 1 Tết 2026
    let short = (1..=12u32)
        .find(|&m| lunar_to_solar(2026, m, 30, false).is_none())
        .expect("năm 2026 có ít nhất một tháng thiếu");
    assert!(lunar_to_solar(2026, short, 29, false).is_some());
    assert_eq!(lunar_to_solar(2026, short, 30, false), None);
    assert_eq!(l.day, 1);
    // Tháng/ngày ngoài khoảng, năm cực trị.
    for (ly, lm, ld) in [
        (2026, 0, 1),
        (2026, 13, 1),
        (2026, 1, 0),
        (2026, 1, 31),
        (2026, u32::MAX, u32::MAX),
        (i32::MAX, 1, 1),
        (i32::MIN, 12, 30),
    ] {
        for leap in [false, true] {
            let r = within("l2s", move || lunar_to_solar(ly, lm, ld, leap));
            assert_eq!(r, None, "{ly}-{lm}-{ld} {leap}");
            let r = within("l2j", move || lunar_to_jdn(ly, lm, ld, leap));
            assert_eq!(r, None, "{ly}-{lm}-{ld} {leap}");
        }
    }
}

#[test]
fn find_good_days_range_rules() {
    // Đảo ngược.
    assert_eq!(
        find_good_days("cuoi_hoi", sd(2026, 3, 1), sd(2026, 2, 1), None),
        Err(DateError::InvertedRange)
    );
    // Quá dài (RS-13): 1900–3000.
    let r = within("fgd long", || {
        find_good_days("cuoi_hoi", sd(1900, 1, 1), sd(3000, 12, 31), None)
    });
    assert_eq!(
        r,
        Err(DateError::RangeTooLong {
            span_days: MAX_JDN - MIN_JDN
        })
    );
    // Đúng cận: to − from = 366 được, 367 thì không.
    let a = jdn_from_date(2026, 1, 1);
    let to366 = lvhv_core::jd::date_from_jdn(a + MAX_SPAN_DAYS);
    assert!(find_good_days(
        "cuoi_hoi",
        sd(2026, 1, 1),
        sd(to366.0, to366.1, to366.2),
        None
    )
    .is_ok());
    let to367 = lvhv_core::jd::date_from_jdn(a + MAX_SPAN_DAYS + 1);
    assert_eq!(
        find_good_days(
            "cuoi_hoi",
            sd(2026, 1, 1),
            sd(to367.0, to367.1, to367.2),
            None
        ),
        Err(DateError::RangeTooLong {
            span_days: MAX_SPAN_DAYS + 1
        })
    );
    // Ngoài miền ở một đầu.
    assert_eq!(
        find_good_days("cuoi_hoi", sd(1899, 12, 31), sd(1900, 1, 5), None),
        Err(OOR)
    );
    assert!(find_good_days("cuoi_hoi", sd(1900, 1, 1), sd(1900, 3, 31), None).is_ok());
    assert!(find_good_days("cuoi_hoi", sd(1967, 12, 1), sd(1968, 2, 29), Some(1940)).is_ok());
    assert_eq!(
        find_good_days("cuoi_hoi", sd(3000, 12, 31), sd(3001, 1, 1), None),
        Err(OOR)
    );
    // Mã việc không tồn tại.
    assert_eq!(
        find_good_days("", sd(2026, 1, 1), sd(2026, 1, 2), None),
        Err(DateError::UnknownActivity)
    );
    // Năm sinh âm cực trị không làm hỏng gì (chỉ dùng để tính Can Chi, rem_euclid).
    for b in [i32::MIN, -1, 0, i32::MAX] {
        assert!(find_good_days("cuoi_hoi", sd(2026, 1, 1), sd(2026, 1, 31), Some(b)).is_ok());
    }
}

#[test]
fn year_table_internal_domain() {
    for yy in [i32::MIN, 0, 1898, 3102, i32::MAX] {
        assert!(YearTable::new(Method::Hnd, yy).is_err(), "{yy}");
        assert!(YearTable::new(Method::Precise, yy).is_err(), "{yy}");
    }
    let t = YearTable::new(Method::Hnd, 2026).unwrap();
    assert_eq!(t.lunar_year(t.n), None);
    assert_eq!(t.lunar_year(usize::MAX), None);
}

/// C-2: tuổi Can Chi theo năm ÂM của ngày sinh.
#[test]
fn birth_lunar_year_before_tet() {
    // Tết 1976 là 31/01/1976: sinh 15/01/1976 thuộc năm âm 1975 (Ất Mão).
    assert_eq!(lunar_year_of_birth(1976, 1, 15), Ok(1975));
    assert_eq!(lunar_year_of_birth(1976, 1, 30), Ok(1975));
    assert_eq!(lunar_year_of_birth(1976, 1, 31), Ok(1976));
    assert_eq!(lunar_year_of_birth(1976, 12, 31), Ok(1976));
    assert_eq!(lunar_year_of_birth(2026, 2, 16), Ok(2025));
    assert_eq!(lunar_year_of_birth(2026, 2, 17), Ok(2026));
    let cc = |y: i32| {
        lvhv_core::canchi::can_chi_name(
            lvhv_core::canchi::year_can(y),
            lvhv_core::canchi::year_chi(y),
        )
        .unwrap()
    };
    assert_eq!(cc(1975), "Ất Mão");
    assert_eq!(cc(1976), "Bính Thìn");
    // Miền ngày sinh 1900–3000 (= miền công khai).
    assert!(lunar_year_of_birth(1900, 1, 1).is_ok());
    assert_eq!(lunar_year_of_birth(1900, 1, 1), Ok(1899));
    assert_eq!(
        lunar_year_of_birth(1899, 12, 31),
        Err(DateError::OutOfRange {
            min: BIRTH_MIN_YEAR,
            max: MAX_YEAR
        })
    );
    assert!(matches!(
        lunar_year_of_birth(1990, 2, 30),
        Err(DateError::InvalidDate { .. })
    ));
    for y in [i32::MIN, i32::MAX, 3001] {
        assert!(within("birth", move || lunar_year_of_birth(y, 1, 1)).is_err());
    }
    // Ngày sinh 1900–1999 đi qua mọi ngày không lỗi (miền nội bộ đủ rộng).
    let mut j = jdn_from_date(1900, 1, 1);
    while j <= jdn_from_date(1999, 12, 31) {
        let (y, m, d) = lvhv_core::jd::date_from_jdn(j);
        lunar_year_of_birth(y, m, d).unwrap();
        lunar_year_of_birth_with(Method::Precise, y, m, d).unwrap();
        j += 7;
    }
}

#[test]
fn ngochap_ctx_rejects_bad_core_ctx() {
    use lvhv_core::dayinfo::{ngoc_hap_of, ngochap_ctx};
    let good = day_ctx(2026, 11, 5).unwrap();
    assert!(ngochap_ctx(&good).is_some());
    let mut c = good;
    c.lunar_day = 167_811; // RS-05: trước đây bị cắt về u8 lặng lẽ
    assert!(ngochap_ctx(&c).is_none());
    assert!(ngoc_hap_of(&c).is_none());
    let mut c = good;
    c.lunar_month = 13;
    assert!(ngochap_ctx(&c).is_none());
    let mut c = good;
    c.day_can = 1;
    c.day_chi = 0;
    assert!(ngochap_ctx(&c).is_none());
}

#[test]
fn error_messages_and_codes() {
    assert_eq!(OOR.code(), "OutOfRange");
    assert_eq!(
        OOR.to_string(),
        "ngoài phạm vi hỗ trợ (năm 1900 đến năm 3000)"
    );
    let e = solar_to_lunar(2026, 2, 30).unwrap_err();
    assert_eq!(e.code(), "InvalidDate");
    assert_eq!(e.to_string(), "ngày dương không hợp lệ: 2026-2-30");
}
