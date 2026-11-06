use lvhv_core::jd::{date_from_jdn, jdn_from_date};
use lvhv_core::*;

#[test]
fn fixture_2026_11_05() {
    let i = day_info(2026, 11, 5).unwrap();
    assert_eq!(i.jdn, 2461350);
    assert_eq!(i.weekday, 4);
    assert_eq!(
        (i.lunar.year, i.lunar.month, i.lunar.day, i.lunar.leap),
        (2026, 9, 27, false)
    );
    assert_eq!(i.cc_year, "Bính Ngọ");
    assert_eq!(i.cc_month, "Mậu Tuất");
    assert_eq!(i.cc_day, "Quý Mùi");
    assert_eq!(i.cc_hour_ty, "Nhâm Tý");
    assert_eq!(i.truc, "Thu");
    assert_eq!(i.than_name, "Chu Tước");
    assert!(!i.than_hoang_dao);
    assert_eq!(i.nap_am, "Dương Liễu Mộc");
    assert_eq!(i.tiet_khi, "Sương Giáng");
    assert_eq!(i.tu, "Tỉnh");
    let gio: Vec<&str> = i.gio_hoang_dao.iter().map(|g| g.chi).collect();
    assert_eq!(gio, ["Dần", "Mão", "Tỵ", "Thân", "Tuất", "Hợi"]);
    let expected = r#"{"solar":{"y":2026,"m":11,"d":5,"weekday":4,"jdn":2461350},"lunar":{"y":2026,"m":9,"d":27,"leap":false,"month_len":30},"can_chi":{"year":"Bính Ngọ","month":"Mậu Tuất","day":"Quý Mùi","hour_ty":"Nhâm Tý"},"tiet_khi":"Sương Giáng","nap_am":"Dương Liễu Mộc","truc":"Thu","tu":"Tỉnh","than":{"name":"Chu Tước","hoang_dao":false},"gio_hoang_dao":[{"chi":"Dần","from":3,"to":5},{"chi":"Mão","from":5,"to":7},{"chi":"Tỵ","from":9,"to":11},{"chi":"Thân","from":15,"to":17},{"chi":"Tuất","from":19,"to":21},{"chi":"Hợi","from":21,"to":23}],"ngoc_hap":{"sao_tot":["U Vi Tinh","Đại Hồng Sa","Thiên Ân","Sát Cống"],"sao_xau":["Tam Nương","Địa Phá","Hoang Vu","Băng Tiêu Ngọa Hãm","Hà Khôi (Cẩu Giảo)","Nguyệt Hình","Ngũ Hư","Tứ Thời Cô Quả","Chu Tước Hắc Đạo"],"nen":[],"ky":["Cưới hỏi","Xây dựng, sửa nhà","Khai trương","An táng, cải táng","Động thổ","Xuất hành","Giao dịch, ký kết","Cầu tài lộc","Nhập trạch, về nhà mới"],"xung_tuoi":["Ất Sửu","Tân Sửu","Đinh Hợi","Đinh Tỵ"],"score":-6}}"#;
    assert_eq!(i.to_json(), expected);
}

#[test]
fn ctx_indices_2026_11_05() {
    let c = day_ctx(2026, 11, 5).unwrap();
    assert_eq!((c.lunar_month, c.lunar_day, c.leap), (9, 27, false));
    assert_eq!((c.day_can, c.day_chi), (9, 7));
    assert_eq!(c.month_chi_tiet, 10);
    assert_eq!((c.year_can, c.year_chi), (2, 6));
    assert_eq!(c.truc, 9);
    assert_eq!(c.tu, 21);
    assert_eq!(c.than, 3);
}

/// Các ngày kề 2026-11-05, đối chiếu với trang lịch công khai (xem đặc tả §8).
#[test]
fn neighbours_2026_11() {
    let a = day_info(2026, 11, 7).unwrap();
    assert_eq!(a.tiet_khi, "Lập Đông");
    assert_eq!(a.truc, "Khai"); // trực lặp lại ngày giao tiết
    assert_eq!(a.tu, "Liễu");
    assert!(a.than_hoang_dao);
    // Tên thống nhất "Kim Đường" (= Bảo Quang/Thiên Đức), trùng tên thần trong Ngọc Hạp.
    assert_eq!(a.than_name, "Kim Đường");
    assert!(a
        .ngoc_hap
        .sao_tot
        .iter()
        .any(|s| s == "Kim Đường Hoàng Đạo"));
    let b = day_info(2026, 11, 8).unwrap();
    assert_eq!(b.truc, "Bế");
    assert_eq!(b.tu, "Tinh");
    assert_eq!(b.than_name, "Bạch Hổ");
}

#[test]
fn tet_known() {
    // Các ngày Tết Việt Nam đã biết (trùng với nhiều nguồn công bố).
    let known = [
        (2000, (2000, 2, 5)),
        (2001, (2001, 1, 24)),
        (2002, (2002, 2, 12)),
        (2003, (2003, 2, 1)),
        (2004, (2004, 1, 22)),
        (2007, (2007, 2, 17)), // Trung Quốc: 2007-02-18 (UTC+8)
        (2020, (2020, 1, 25)),
        (2021, (2021, 2, 12)),
        (2022, (2022, 2, 1)),
        (2023, (2023, 1, 22)),
        (2024, (2024, 2, 10)),
        (2025, (2025, 1, 29)),
        (2026, (2026, 2, 17)),
    ];
    for (ly, (y, m, d)) in known {
        let t = tet(ly).unwrap();
        assert_eq!((t.year, t.month, t.day), (y, m, d), "Tết {ly}");
    }
}

#[test]
fn leap_months_known() {
    // Tháng nhuận đã biết: 2001 nhuận 4, 2004 nhuận 2, 2006 nhuận 7, 2009 nhuận 5,
    // 2012 nhuận 4, 2014 nhuận 9, 2017 nhuận 6, 2020 nhuận 4, 2023 nhuận 2, 2025 nhuận 6.
    for (ly, lm) in [
        (2001, 4),
        (2004, 2),
        (2006, 7),
        (2009, 5),
        (2012, 4),
        (2014, 9),
        (2017, 6),
        (2020, 4),
        (2023, 2),
        (2025, 6),
    ] {
        assert!(lunar_to_solar(ly, lm, 1, true).is_some(), "{ly} nhuận {lm}");
    }
    assert!(lunar_to_solar(2026, 5, 1, true).is_none());
}

#[test]
fn invalid_lunar_dates() {
    assert!(lunar_to_solar(2026, 13, 1, false).is_none());
    assert!(lunar_to_solar(2026, 1, 0, false).is_none());
    assert!(lunar_to_solar(2026, 1, 31, false).is_none());
}

fn roundtrip(from: (i32, u32, u32), to: (i32, u32, u32)) {
    let a = jdn_from_date(from.0, from.1, from.2);
    let b = jdn_from_date(to.0, to.1, to.2);
    let mut prev: Option<LunarDate> = None;
    for j in a..=b {
        let (y, m, d) = date_from_jdn(j);
        let l = solar_to_lunar(y, m, d).unwrap();
        let s = lunar_to_solar(l.year, l.month, l.day, l.leap).expect("ngày âm phải tồn tại");
        assert_eq!((s.year, s.month, s.day), (y, m, d), "{l:?}");
        assert!(l.month_len == 29 || l.month_len == 30);
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

#[test]
fn roundtrip_2000_2100() {
    roundtrip((2000, 1, 1), (2100, 12, 31));
}

#[test]
#[ignore = "chậm ở bản debug; chạy bằng: cargo test --release -- --ignored"]
fn roundtrip_2000_3000() {
    roundtrip((2000, 1, 1), (3000, 12, 31));
}
