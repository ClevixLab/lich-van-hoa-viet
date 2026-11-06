//! Bổ sung A1 — phạm vi 1900–1999 và điểm chuyển múi giờ (docs/02 L9, L11, L12; docs/03 §16).
use lvhv_core::astro::{local_day_of, tz_of_day, TZ_SWITCH_JDN};
use lvhv_core::jd::{date_from_jdn, jdn_from_date};
use lvhv_core::*;

const METHODS: [Method; 2] = [Method::Hnd, Method::Precise];

#[test]
fn switch_constants() {
    assert_eq!(TZ_SWITCH_JDN, jdn_from_date(1968, 1, 1));
    assert_eq!(tz_of_day(TZ_SWITCH_JDN - 1), 8.0);
    assert_eq!(tz_of_day(TZ_SWITCH_JDN), 7.0);
    // 1967-12-31 16:00 UT = 1968-01-01 00:00 UTC+8 = 1967-12-31 23:00 UTC+7.
    let edge = TZ_SWITCH_JDN as f64 - 0.5 - 8.0 / 24.0;
    // Trong một giờ ngay sau điểm chuyển: vẫn là ngày 1967-12-31 (ngày dài 25 giờ).
    assert_eq!(local_day_of(edge - 1e-6), TZ_SWITCH_JDN - 1);
    assert_eq!(local_day_of(edge + 0.5 / 24.0), TZ_SWITCH_JDN - 1);
    assert_eq!(local_day_of(edge + 1.0 / 24.0 + 1e-6), TZ_SWITCH_JDN);
    // Không giảm theo thời gian (quét dày quanh điểm chuyển).
    let mut prev = i64::MIN;
    let mut t = edge - 2.0;
    while t < edge + 2.0 {
        let d = local_day_of(t);
        assert!(d >= prev, "local_day_of giảm tại {t}");
        prev = d;
        t += 1.0 / 1440.0;
    }
}

/// L11: mọi ngày 1967-01-01 → 1969-12-31 có đúng một ngày âm; ngày âm tăng đều; không thiếu,
/// không trùng; Tết 1968 = 1968-01-29.
#[test]
fn l11_continuity_1967_1969() {
    for method in METHODS {
        let a = jdn_from_date(1967, 1, 1);
        let b = jdn_from_date(1969, 12, 31);
        let mut seen = std::collections::HashSet::new();
        let mut prev: Option<LunarDate> = None;
        for j in a..=b {
            let l = jdn_to_lunar_with(method, j).unwrap();
            assert!(
                seen.insert((l.year, l.month, l.leap, l.day)),
                "{method:?}: trùng {l:?}"
            );
            assert_eq!(
                lunar_to_jdn_with(method, l.year, l.month, l.day, l.leap),
                Some(j)
            );
            if let Some(p) = prev {
                if l.day == 1 {
                    assert_eq!(
                        p.day, p.month_len,
                        "{method:?}: thiếu ngày cuối tháng {p:?}"
                    );
                    // Tháng kế: tháng nhuận cùng số, hoặc số tháng + 1 (12 → 1 sang năm mới).
                    let ok = (l.leap && !p.leap && l.month == p.month && l.year == p.year)
                        || (!l.leap
                            && l.month == p.month % 12 + 1
                            && l.year == if p.month == 12 { p.year + 1 } else { p.year });
                    assert!(ok, "{method:?}: tháng nhảy {p:?} → {l:?}");
                } else {
                    assert_eq!(
                        (l.year, l.month, l.leap, l.day),
                        (p.year, p.month, p.leap, p.day + 1),
                        "{method:?} JDN {j}"
                    );
                }
            }
            prev = Some(l);
        }
        let t = tet_with(method, 1968).unwrap();
        assert_eq!((t.year, t.month, t.day), (1968, 1, 29), "{method:?}");
        let i = day_info_with(method, 1968, 1, 29).unwrap();
        assert_eq!((i.lunar.month, i.lunar.day, i.lunar.leap), (1, 1, false));
        assert_eq!(i.cc_year, "Mậu Thân");
        let e = day_info_with(method, 1968, 1, 28).unwrap();
        assert_eq!((e.lunar.year, e.lunar.month, e.lunar.day), (1967, 12, 29));
    }
}

/// Tết thế kỷ 20 có nguồn công bố (mỗi dòng ghi nguồn). Chỉ các năm có nguồn kiểm chứng được.
#[test]
fn tet_20th_century_published() {
    let known = [
        // Wikipedia "Vietnamese calendar" (dẫn Aslaksen, *Mathematics of the Chinese calendar*,
        // tr. 29–30): miền Bắc và miền Nam ăn Tết 1968 khác ngày do đổi múi giờ; Sóc
        // 1968-01-29 16:31 UT → miền Bắc (UTC+7) 1968-01-29, miền Nam (UTC+8) 1968-01-30
        // (sad-pony-mathematician.blogspot.com, "An Off-by-one Error in the Tet Offensive", 2019).
        (1968, (1968, 1, 29)),
        // Nhân Dân 29/06/2006 "Chung quanh thông tin về ngày âm lịch khác nhau trên một số loại
        // lịch": Tết Kỷ Dậu 1969 của Việt Nam sớm hơn Trung Quốc một ngày; Đài Thiên văn Hồng
        // Kông (hko.gov.hk, lịch 1969) ghi tháng Giêng bắt đầu 1969-02-17 → Việt Nam 1969-02-16.
        (1969, (1969, 2, 16)),
        // Wikipedia "Vietnamese calendar": Tết 1985 của Việt Nam là 21/01/1985 (sớm hơn Trung
        // Quốc một tháng).
        (1985, (1985, 1, 21)),
        // Trước 1968 lịch Việt tính theo UTC+8, trùng lịch Trung Quốc (thuviengdpt.info, "Âm lịch
        // Việt Nam qua các thời kỳ lịch sử", theo tài liệu của Hồ Ngọc Đức: 1946–1954 "không có
        // sự khác biệt"). Đài Thiên văn Hồng Kông, lịch 1954: tháng Giêng bắt đầu 1954-02-03.
        (1954, (1954, 2, 3)),
    ];
    for method in METHODS {
        for (ly, (y, m, d)) in known {
            let t = tet_with(method, ly).unwrap();
            assert_eq!((t.year, t.month, t.day), (y, m, d), "Tết {ly} {method:?}");
        }
    }
}

/// Ca đối chiếu của bên điều phối (PyEphem, UTC+7): 1977-11-08 = 27/9 Đinh Tỵ, ngày Kỷ Tỵ,
/// tháng Canh Tuất.
#[test]
fn fixture_1977_11_08() {
    for method in METHODS {
        let i = day_info_with(method, 1977, 11, 8).unwrap();
        assert_eq!(
            (i.lunar.year, i.lunar.month, i.lunar.day, i.lunar.leap),
            (1977, 9, 27, false)
        );
        assert_eq!(i.cc_year, "Đinh Tỵ");
        assert_eq!(i.cc_day, "Kỷ Tỵ");
        assert_eq!(i.cc_month, "Canh Tuất");
    }
}

/// Giới hạn đã biết của quy ước UTC+8 cho 1900–1928 (docs/03 §16.4): Sóc tháng Giêng 1916 lúc
/// 1916-02-04 00:05–00:07 UTC+8, nên lõi cho Tết 1916 = 1916-02-04, trong khi bảng TK20 của Hồ
/// Ngọc Đức và lịch Trung Quốc (giờ địa phương Bắc Kinh trước 1929, Đài Thiên văn Hồng Kông)
/// ghi 1916-02-03. Test khóa hành vi hiện tại để mọi thay đổi quy ước phải có chủ ý.
#[test]
fn known_limitation_1916() {
    for method in METHODS {
        let t = tet_with(method, 1916).unwrap();
        assert_eq!((t.year, t.month, t.day), (1916, 2, 4), "{method:?}");
    }
}

fn roundtrip(method: Method, from: (i32, u32, u32), to: (i32, u32, u32)) -> usize {
    let a = jdn_from_date(from.0, from.1, from.2);
    let b = jdn_from_date(to.0, to.1, to.2);
    let mut prev: Option<LunarDate> = None;
    for j in a..=b {
        let (y, m, d) = date_from_jdn(j);
        let l = solar_to_lunar_with(method, y, m, d).unwrap();
        let s = lunar_to_solar_with(method, l.year, l.month, l.day, l.leap)
            .expect("ngày âm phải tồn tại");
        assert_eq!((s.year, s.month, s.day), (y, m, d), "{method:?} {l:?}");
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
    (b - a + 1) as usize
}

#[test]
fn roundtrip_1900_1999() {
    for method in METHODS {
        roundtrip(method, (1900, 1, 1), (1999, 12, 31));
    }
}

/// L12 (phần L5): khứ hồi mọi ngày 1900-01-01 → 3000-12-31, cả hai phương pháp.
#[test]
#[ignore = "chậm ở bản debug; chạy bằng: cargo test --release -- --ignored"]
fn l12_roundtrip_1900_3000() {
    for method in METHODS {
        let n = roundtrip(method, (1900, 1, 1), (3000, 12, 31));
        eprintln!("L12 khứ hồi {method:?}: {n} ngày, 0 lỗi");
    }
}
