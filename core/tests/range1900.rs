//! Bổ sung A1 — phạm vi 1900–1999 và điểm chuyển múi giờ (docs/02 L9, L11, L12; docs/03 §16).
use lvhv_core::astro::{local_day_of, tz_of_day, TZ_1929_JDN, TZ_BEIJING, TZ_SWITCH_JDN};
use lvhv_core::jd::{date_from_jdn, jdn_from_date};
use lvhv_core::*;

const METHODS: [Method; 2] = [Method::Hnd, Method::Precise];

#[test]
fn switch_constants() {
    assert_eq!(TZ_SWITCH_JDN, jdn_from_date(1968, 1, 1));
    assert_eq!(TZ_1929_JDN, jdn_from_date(1929, 1, 1));
    assert_eq!(TZ_BEIJING * 3600.0, 27940.0); // 7:45:40
    assert_eq!(tz_of_day(TZ_1929_JDN), TZ_BEIJING);
    assert_eq!(tz_of_day(TZ_1929_JDN + 1), 8.0);
    // 1929-01-01 00:00 giờ Bắc Kinh = 1928-12-31 16:14:20 UT = 1929-01-01 00:14:20 UTC+8.
    let e29 = TZ_1929_JDN as f64 - 0.5 - TZ_BEIJING / 24.0;
    assert_eq!(local_day_of(e29 - 1e-6), TZ_1929_JDN - 1);
    assert_eq!(local_day_of(e29 + 1e-6), TZ_1929_JDN);
    // Ngày 1928-12-31 dài 24 giờ; ngày 1929-01-01 dài 23:45:40 (kết thúc 00:00 UTC+8 ngày 2/1).
    let end29 = (TZ_1929_JDN + 1) as f64 - 0.5 - 8.0 / 24.0;
    assert_eq!(local_day_of(end29 - 1e-6), TZ_1929_JDN);
    assert_eq!(local_day_of(end29 + 1e-6), TZ_1929_JDN + 1);
    assert!(((end29 - e29) * 86400.0 - (86400.0 - 860.0)).abs() < 1e-3);
    let mut prev = i64::MIN;
    let mut t = e29 - 2.0;
    while t < e29 + 2.0 {
        let d = local_day_of(t);
        assert!(d >= prev, "local_day_of giảm tại {t}");
        prev = d;
        t += 1.0 / 1440.0;
    }
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
        // Đài Thiên văn Hồng Kông, lịch 1916: tháng Giêng bắt đầu 1916-02-03 (trước 1929 lịch Trung
        // Hoa và lịch Hiệp Kỷ cùng tính theo giờ Bắc Kinh — docs/03 §16.3).
        (1916, (1916, 2, 3)),
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

/// Tết Bính Thìn 1916 = 1916-02-03: Đài Thiên văn Hồng Kông (hko.gov.hk, lịch 1916, tháng Giêng bắt đầu
/// 03/02) và bảng TK20 của Hồ Ngọc Đức. Sóc lúc 1916-02-04 00:05 UTC+8 = 1916-02-03 23:51 giờ Bắc Kinh
/// → trước 1929 tính theo giờ Bắc Kinh (docs/02 Bổ sung A2, docs/03 §16).
#[test]
fn tet_1916_beijing_time() {
    for method in METHODS {
        let t = tet_with(method, 1916).unwrap();
        assert_eq!((t.year, t.month, t.day), (1916, 2, 3), "{method:?}");
        let i = day_info_with(method, 1916, 2, 3).unwrap();
        assert_eq!(
            (i.lunar.month, i.lunar.day, i.cc_year.as_str()),
            (1, 1, "Bính Thìn")
        );
    }
}

/// Bảng TK20 (năm âm 1900–1999) trong mã nguồn `amlich` của Hồ Ngọc Đức (© 2004 Hồ Ngọc Đức,
/// http://www.informatik.uni-leipzig.de/~duc/amlich/; bản phân phối lại: github.com/ichuot/jquery-amlich,
/// src/jquery.amlich.js, mảng `TK20`). Mã hóa mỗi năm: bit 17+ = số ngày từ 1/1 dương tới Tết; bit 16 =
/// tháng nhuận dài 30; bit 4..15 = độ dài 12 tháng (1 = 30 ngày, tháng 1 ở bit cao); bit 0..3 = tháng nhuận.
const TK20: [u32; 100] = [
    0x3c4bd8, 0x624ae0, 0x4ca570, 0x3854d5, 0x5cd260, 0x44d950, 0x315554, 0x5656a0, 0x409ad0,
    0x2a55d2, 0x504ae0, 0x3aa5b6, 0x60a4d0, 0x48d250, 0x33d255, 0x58b540, 0x42d6a0, 0x2cada2,
    0x5295b0, 0x3f4977, 0x644970, 0x4ca4b0, 0x36b4b5, 0x5c6a50, 0x466d40, 0x2fab54, 0x562b60,
    0x409570, 0x2c52f2, 0x504970, 0x3a6566, 0x5ed4a0, 0x48ea50, 0x336a95, 0x585ad0, 0x442b60,
    0x2f86e3, 0x5292e0, 0x3dc8d7, 0x62c950, 0x4cd4a0, 0x35d8a6, 0x5ab550, 0x4656a0, 0x31a5b4,
    0x5625d0, 0x4092d0, 0x2ad2b2, 0x50a950, 0x38b557, 0x5e6ca0, 0x48b550, 0x355355, 0x584da0,
    0x42a5b0, 0x2f4573, 0x5452b0, 0x3ca9a8, 0x60e950, 0x4c6aa0, 0x36aea6, 0x5aab50, 0x464b60,
    0x30aae4, 0x56a570, 0x405260, 0x28f263, 0x4ed940, 0x38db47, 0x5cd6a0, 0x4896d0, 0x344dd5,
    0x5a4ad0, 0x42a4d0, 0x2cd4b4, 0x52b250, 0x3cd558, 0x60b540, 0x4ab5a0, 0x3755a6, 0x5c95b0,
    0x4649b0, 0x30a974, 0x56a4b0, 0x40aa50, 0x29aa52, 0x4e6d20, 0x39ad47, 0x5eab60, 0x489370,
    0x344af5, 0x5a4970, 0x4464b0, 0x2c74a3, 0x50ea50, 0x3d6a58, 0x6256a0, 0x4aaad0, 0x3696d5,
    0x5c92e0,
];

/// Giải mã TK20 → (năm âm, tháng, nhuận, JDN ngày đầu tháng) — theo `decodeLunarYear` của tác giả.
fn tk20_months() -> Vec<(i32, u32, bool, i64)> {
    let mut out = Vec::new();
    for (i, &k) in TK20.iter().enumerate() {
        let yy = 1900 + i as i32;
        let len = |b: u32| if b & 1 == 1 { 30 } else { 29 };
        let leap = k & 0xf;
        let leap_len = len(k >> 16);
        let mut reg = [0i64; 12];
        let mut j = k >> 4;
        for t in 0..12 {
            reg[11 - t] = len(j);
            j >>= 1;
        }
        let mut cur = jdn_from_date(yy, 1, 1) + i64::from(k >> 17);
        for mm in 1..=12u32 {
            out.push((yy, mm, false, cur));
            cur += reg[mm as usize - 1];
            if mm == leap {
                out.push((yy, mm, true, cur));
                cur += leap_len;
            }
        }
    }
    out
}

/// Mọi tháng âm 1900–1999 (1237 tháng của bảng) so với TK20: Precise trùng tất cả; Hnd trùng 1233, lệch đúng 4
/// tháng ở hai năm (vị trí tháng nhuận 1917, 1922) do mô hình Mặt Trời rút gọn của HND đặt Trung khí
/// sát nửa đêm giờ Bắc Kinh sang ngày trước (Cốc Vũ 1917-04-21 00:03, Đại Thử 1922-07-24 00:05 theo
/// mô hình chính xác) — docs/03 §16.4. Danh sách được khóa: mọi thay đổi phải có chủ ý.
#[test]
fn tk20_month_starts_1900_1999() {
    let tk = tk20_months();
    // 1237 mục = 1236 tháng bắt đầu trong 1900–1999 + tháng Chạp năm âm 1999 (bắt đầu 2000-01-07).
    assert_eq!(tk.len(), 1237);
    for method in METHODS {
        let mut bad = Vec::new();
        for &(ly, lm, leap, j) in &tk {
            let got = lunar_to_jdn_with(method, ly, lm, 1, leap);
            if got != Some(j) {
                bad.push((ly, lm, leap));
            }
        }
        match method {
            Method::Precise => assert!(bad.is_empty(), "Precise lệch TK20: {bad:?}"),
            Method::Hnd => assert_eq!(
                bad,
                [
                    (1917, 2, true),
                    (1917, 3, false),
                    (1922, 5, true),
                    (1922, 6, false)
                ],
                "Hnd lệch TK20"
            ),
        }
        eprintln!(
            "TK20 {method:?}: {} / {} tháng trùng",
            tk.len() - bad.len(),
            tk.len()
        );
    }
}

/// Liên tục quanh điểm chuyển giờ Bắc Kinh → UTC+8 (1929-01-01): mỗi ngày đúng một ngày âm, tăng đều.
#[test]
fn continuity_around_1929() {
    for method in METHODS {
        continuity(method, (1928, 1, 1), (1930, 12, 31));
    }
}

fn continuity(method: Method, from: (i32, u32, u32), to: (i32, u32, u32)) {
    let mut seen = std::collections::HashSet::new();
    let mut prev: Option<LunarDate> = None;
    for j in jdn_from_date(from.0, from.1, from.2)..=jdn_from_date(to.0, to.1, to.2) {
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
