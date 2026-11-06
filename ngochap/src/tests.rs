use super::*;

/// Chi ngày khởi Thanh Long theo tháng âm (quy tắc 12 thần, như lõi).
fn than_of(lm: u8, chi: u8) -> u8 {
    let start = ((lm - 1) % 6) * 2;
    (chi + 12 - start) % 12
}

fn ctx(lm: u8, ld: u8, can: u8, chi: u8, truc: u8) -> DayCtx {
    DayCtx {
        lunar_month: lm,
        lunar_day: ld,
        leap: false,
        day_can: can,
        day_chi: chi,
        month_chi_tiet: (lm + 1) % 12,
        year_can: 2, // Bính
        year_chi: 6, // Ngọ
        truc,
        tu: 0,
        than: than_of(lm, chi),
        solar_month: 1,
        solar_day: 1,
    }
}

fn has(v: &[&str], s: &str) -> bool {
    v.contains(&s)
}

// ---------- Kiểm tra từng quy tắc (mỗi sao một ca dương và một ca âm) ----------

#[test]
fn every_star_hits_and_misses() {
    // Với mỗi sao trong bảng, phải có ít nhất một ngày có sao và một ngày không có sao.
    let all: Vec<&Star> = GOOD_STARS
        .iter()
        .chain(BAD_STARS.iter())
        .chain(SPECIAL_DAYS.iter())
        .collect();
    for st in all {
        let mut hit = false;
        let mut miss = false;
        for yc in 0..10u8 {
            for lm in 1..=12u8 {
                for ld in [1u8, 3, 5, 7, 8, 9, 11, 13, 19, 21, 23, 25, 27, 29] {
                    for n in 0..60u8 {
                        let mut c = ctx(lm, ld, n % 10, n % 12, 0);
                        c.year_can = yc;
                        if rule_hits(&st.rule, &c) {
                            hit = true;
                        } else {
                            miss = true;
                        }
                    }
                }
            }
        }
        assert!(
            hit && miss,
            "sao {} {} không có ca dương/âm",
            st.id,
            st.name
        );
    }
}

#[test]
fn ids_unique_and_sorted() {
    let all: Vec<&Star> = SPECIAL_DAYS
        .iter()
        .chain(GOOD_STARS.iter())
        .chain(BAD_STARS.iter())
        .chain(THAN.iter())
        .collect();
    let mut ids: Vec<&str> = all.iter().map(|s| s.id).collect();
    let n = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), n);
}

#[test]
fn thien_duc_mixed_can_chi() {
    // Tháng Giêng: Thiên Đức ở can Đinh; tháng Hai ở chi Thân.
    assert!(has(
        &evaluate(&ctx(1, 1, DINH, ZMAO, 0)).sao_tot,
        "Thiên Đức"
    ));
    assert!(!has(
        &evaluate(&ctx(1, 1, BINH, ZDAN, 0)).sao_tot,
        "Thiên Đức"
    ));
    assert!(has(
        &evaluate(&ctx(2, 1, NHAM, ZTHAN, 0)).sao_tot,
        "Thiên Đức"
    ));
    assert!(has(&evaluate(&ctx(11, 1, KY, ZTI, 0)).sao_tot, "Thiên Đức"));
}

#[test]
fn nguyet_duc_by_month_group() {
    for (lm, c) in [
        (1, BINH),
        (5, BINH),
        (9, BINH),
        (2, GIAP),
        (3, NHAM),
        (4, CANH),
    ] {
        let d = (0..12).find(|z| z % 2 == c % 2).unwrap();
        assert!(has(&evaluate(&ctx(lm, 1, c, d, 0)).sao_tot, "Nguyệt Đức"));
    }
}

#[test]
fn thien_an_fifteen_days() {
    let n = (0..60u8)
        .filter(|&n| rule_hits(&GOOD_STARS[43].rule, &ctx(1, 1, n % 10, n % 12, 0)))
        .count();
    assert_eq!(GOOD_STARS[43].name, "Thiên Ân");
    assert_eq!(n, 15);
}

#[test]
fn sat_cong_truc_tinh_nhan_chuyen_groups() {
    // Tháng 9 (nhóm Quý): Quý Mùi là Sát Cống.
    assert!(has(&evaluate(&ctx(9, 1, QUY, ZMUI, 0)).sao_tot, "Sát Cống"));
    // Tháng 1 (nhóm Mạnh): Quý Mùi không phải Sát Cống.
    assert!(!has(
        &evaluate(&ctx(1, 1, QUY, ZMUI, 0)).sao_tot,
        "Sát Cống"
    ));
    // Trực Tinh tháng Trọng = Sát Cống tháng Mạnh.
    assert!(has(
        &evaluate(&ctx(2, 1, DINH, ZMAO, 0)).sao_tot,
        "Trực Tinh"
    ));
    assert!(has(
        &evaluate(&ctx(4, 1, DINH, ZMAO, 0)).sao_tot,
        "Sát Cống"
    ));
    assert!(has(
        &evaluate(&ctx(12, 1, QUY, ZHOI, 0)).sao_tot,
        "Nhân Chuyên"
    ));
}

#[test]
fn season_rules() {
    // Hoang Vu: Thu (tháng 7–9) ở Hợi, Mão, Mùi.
    for z in [ZHOI, ZMAO, ZMUI] {
        assert!(has(&evaluate(&ctx(8, 1, z % 10, z, 0)).sao_xau, "Hoang Vu"));
    }
    assert!(!has(&evaluate(&ctx(8, 1, 0, ZTY, 0)).sao_xau, "Hoang Vu"));
    // Tứ Thời Cô Quả: Thu ở Mùi, Xuân ở Sửu.
    assert!(has(
        &evaluate(&ctx(9, 1, QUY, ZMUI, 0)).sao_xau,
        "Tứ Thời Cô Quả"
    ));
    assert!(has(
        &evaluate(&ctx(2, 1, AT, ZSUU, 0)).sao_xau,
        "Tứ Thời Cô Quả"
    ));
    // Thiên Xá: Đông ở Giáp Tý.
    assert!(has(
        &evaluate(&ctx(11, 1, GIAP, ZTY, 0)).sao_tot,
        "Thiên Xá"
    ));
    assert!(!has(
        &evaluate(&ctx(11, 1, BINH, ZTY, 0)).sao_tot,
        "Thiên Xá"
    ));
}

#[test]
fn special_lunar_days() {
    for d in [3u8, 7, 13, 18, 22, 27] {
        assert!(has(&evaluate(&ctx(4, d, 0, 0, 0)).sao_xau, "Tam Nương"));
    }
    assert!(!has(&evaluate(&ctx(4, 4, 0, 0, 0)).sao_xau, "Tam Nương"));
    for d in [5u8, 14, 23] {
        assert!(has(&evaluate(&ctx(4, d, 0, 0, 0)).sao_xau, "Nguyệt Kỵ"));
    }
    // Dương Công: 13 ngày.
    let mut n = 0;
    for lm in 1..=12u8 {
        for ld in 1..=30u8 {
            if has(
                &evaluate(&ctx(lm, ld, 0, 0, 0)).sao_xau,
                "Dương Công Kỵ Nhật",
            ) {
                n += 1;
            }
        }
    }
    assert_eq!(n, 13);
    assert!(has(
        &evaluate(&ctx(12, 19, 0, 0, 0)).sao_xau,
        "Dương Công Kỵ Nhật"
    ));
    assert!(has(
        &evaluate(&ctx(7, 1, 0, 0, 0)).sao_xau,
        "Dương Công Kỵ Nhật"
    ));
    // Bản khác ghi 8/7 (rules.md, mục S03) — không chọn.
    assert!(!has(
        &evaluate(&ctx(7, 8, 0, 0, 0)).sao_xau,
        "Dương Công Kỵ Nhật"
    ));
    // Thụ Tử (B14, theo chi): tháng 8 ở Mùi (mọi can). Thọ Tử (S04, theo can chi): tháng 8 Quý Mùi.
    let e = evaluate(&ctx(8, 1, QUY, ZMUI, 0));
    assert!(has(&e.sao_xau, "Thụ Tử") && has(&e.sao_xau, "Thọ Tử"));
    let e = evaluate(&ctx(8, 1, DINH, ZMUI, 0));
    assert!(has(&e.sao_xau, "Thụ Tử") && !has(&e.sao_xau, "Thọ Tử"));
    assert!(!has(&evaluate(&ctx(9, 1, QUY, ZMUI, 0)).sao_xau, "Thọ Tử"));
    // Mọi ngày Thọ Tử đều là Thụ Tử (hai bảng nhất quán).
    for lm in 1..=12u8 {
        for n in 0..60u8 {
            let e = evaluate(&ctx(lm, 1, n % 10, n % 12, 0));
            if has(&e.sao_xau, "Thọ Tử") {
                assert!(has(&e.sao_xau, "Thụ Tử"));
            }
        }
    }
}

#[test]
fn thap_ac_dai_bai() {
    let mut c = ctx(6, 1, KY, ZSUU, 0);
    c.year_can = MAU;
    assert!(has(&evaluate(&c).sao_xau, "Thập Ác Đại Bại"));
    c.year_can = QUY;
    assert!(has(&evaluate(&c).sao_xau, "Thập Ác Đại Bại"));
    c.year_can = GIAP;
    assert!(!has(&evaluate(&c).sao_xau, "Thập Ác Đại Bại"));
}

#[test]
fn hoat_dieu_cancelled_by_thu_tu() {
    // Luật loại trừ: ngày nào có Thụ Tử thì không được ghi Hoạt Điệu.
    for lm in 1..=12u8 {
        for z in 0..12u8 {
            let e = evaluate(&ctx(lm, 1, z % 10, z, 0));
            if has(&e.sao_xau, "Thụ Tử") {
                assert!(!has(&e.sao_tot, "Hoạt Điệu"));
            }
        }
    }
}

#[test]
fn than_hoang_hac_dao() {
    // Tháng 9, ngày Mùi → Chu Tước (Hắc đạo).
    let e = evaluate(&ctx(9, 27, QUY, ZMUI, 9));
    assert!(has(&e.sao_xau, "Chu Tước Hắc Đạo"));
    // Tháng 1, ngày Tý → Thanh Long (Hoàng đạo).
    assert!(has(
        &evaluate(&ctx(1, 1, GIAP, ZTY, 0)).sao_tot,
        "Thanh Long Hoàng Đạo"
    ));
}

// ---------- Tuổi xung ----------

#[test]
fn xung_tuoi_quy_mui() {
    let e = evaluate(&ctx(9, 27, QUY, ZMUI, 9));
    assert_eq!(e.xung_tuoi, ["Ất Sửu", "Tân Sửu", "Đinh Hợi", "Đinh Tỵ"]);
}

#[test]
fn xung_tuoi_table_well_formed() {
    for (n, row) in XUNG_TUOI.iter().enumerate() {
        assert!((2..=4).contains(&row.len()), "dòng {n}");
        for &(c, z) in row.iter() {
            assert_eq!(c % 2, z % 2, "cặp can chi sai tính chẵn lẻ ở dòng {n}");
        }
        assert!(
            row.iter().any(|&(_, z)| z == ((n % 12) as u8 + 6) % 12),
            "dòng {n} thiếu lục xung"
        );
    }
}

#[test]
fn birth_year_clash_blocks_personal() {
    // 1985 = Ất Sửu, xung ngày Quý Mùi.
    let c = ctx(9, 26, QUY, ZMUI, 9); // ngày 26 (không phải Tam Nương)
    let s = suitability(&c, "xuat_hanh", Some(1985));
    assert!(!s.allowed);
    assert!(s.reasons.iter().any(|r| r.contains("Ất Sửu")));
    // Tế tự không phải việc cá nhân → vẫn tính điểm, không chặn vì tuổi.
    let s2 = suitability(&c, "te_tu", Some(1985));
    let s3 = suitability(&c, "te_tu", None);
    assert_eq!(s2.allowed, s3.allowed);
    assert_eq!(s2.score, s3.score - XUNG_PENALTY);
    // 1990 = Canh Ngọ, không xung.
    assert_eq!(
        suitability(&c, "xuat_hanh", Some(1990)).score,
        suitability(&c, "xuat_hanh", None).score
    );
}

#[test]
fn validate_rejects_bad_ctx() {
    let good = ctx(9, 27, 9, 7, 9);
    assert_eq!(good.validate(), Ok(()));
    let mut c = good;
    c.lunar_month = 13;
    assert_eq!(c.validate(), Err(CtxError::LunarMonth(13)));
    c = good;
    c.lunar_month = 0;
    assert_eq!(c.validate(), Err(CtxError::LunarMonth(0)));
    c = good;
    c.lunar_day = 31;
    assert_eq!(c.validate(), Err(CtxError::LunarDay(31)));
    c = good;
    c.day_can = 1;
    c.day_chi = 0; // Ất Tý không tồn tại
    assert_eq!(c.validate(), Err(CtxError::Parity));
    c = good;
    c.day_chi = 12;
    assert_eq!(c.validate(), Err(CtxError::Chi(12)));
    c = good;
    c.tu = 28;
    assert_eq!(c.validate(), Err(CtxError::Index(28)));
    c = good;
    c.solar_day = 0;
    assert_eq!(c.validate(), Err(CtxError::Solar));
}

#[test]
fn unknown_activity() {
    let s = suitability(&ctx(1, 1, 0, 0, 0), "khong_co", None);
    assert!(!s.allowed);
}

#[test]
fn activities_list() {
    let a = activities();
    assert_eq!(a.len(), 16);
    assert_eq!(a[0].id, "cuoi_hoi");
    let mut ids: Vec<&str> = a.iter().map(|x| x.id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 16);
}

// ---------- Tiêu chí N3 ----------

#[test]
fn n3_major_taboo_never_allowed() {
    // Duyệt mọi tổ hợp (tháng âm × ngày âm × 60 can chi × 10 can năm × trực) đủ phủ mọi quy tắc
    // (quy tắc chỉ phụ thuộc các trường này); mọi việc phạm sao kỵ chính đều phải allowed = false.
    for yc in 0..10u8 {
        for lm in 1..=12u8 {
            for ld in 1..=30u8 {
                for n in 0..60u8 {
                    let mut c = ctx(lm, ld, n % 10, n % 12, (n + ld) % 12);
                    c.year_can = yc;
                    let (_, bad) = stars_of(&c);
                    for (i, act) in ACTIVITIES.iter().enumerate() {
                        let has_major = bad.iter().any(|s| s.major.contains(i as Act));
                        let s = suitability(&c, act.id, None);
                        if has_major {
                            assert!(!s.allowed, "{lm}/{ld} n={n} {}", act.id);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn deterministic() {
    let c = ctx(9, 27, QUY, ZMUI, 9);
    assert_eq!(evaluate(&c), evaluate(&c));
}

// ---------- Đối chiếu trang lịch công khai (fixture) ----------

/// 2026-11-05: âm 27/9 Bính Ngọ, ngày Quý Mùi, tháng Mậu Tuất, Trực Thu, Chu Tước, tú Tỉnh.
///
/// Trang công khai liệt kê:
///   tốt: Đại Hồng Sa, Sát Cống, Thiên Ân, U Vi Tinh
///   xấu: Tam Nương, Băng Tiêu Ngọa Hãm, Chu Tước Hắc Đạo, Địa Phá, Hà Khôi/Cẩu Giảo,
///        Hoang Vu, Ngũ Hư, Nguyệt Hình, Tứ Thời Cô Quả
///   xung: Ất Sửu, Đinh Hợi, Tân Sửu, Đinh Tỵ
/// Quy tắc có nguồn của crate cho ra ĐÚNG các sao trên, không thừa không thiếu (xem rules.md §Đối chiếu).
/// Không có sai lệch cần ghi.
#[test]
fn fixture_2026_11_05() {
    let mut c = ctx(9, 27, QUY, ZMUI, 9);
    c.month_chi_tiet = ZTUAT;
    c.tu = 21;
    c.solar_month = 11;
    c.solar_day = 5;
    assert_eq!(c.than, 3);
    let e = evaluate(&c);
    let mut tot = e.sao_tot.clone();
    tot.sort();
    let mut want_tot = vec!["Đại Hồng Sa", "Sát Cống", "Thiên Ân", "U Vi Tinh"];
    want_tot.sort();
    assert_eq!(tot, want_tot);
    let mut xau = e.sao_xau.clone();
    xau.sort();
    let mut want_xau = vec![
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
    want_xau.sort();
    assert_eq!(xau, want_xau);
    assert_eq!(e.xung_tuoi, ["Ất Sửu", "Tân Sửu", "Đinh Hợi", "Đinh Tỵ"]);
    // Thứ tự xuất cố định (thứ tự bảng).
    assert_eq!(e.sao_xau[0], "Tam Nương");
    assert_eq!(*e.sao_xau.last().unwrap(), "Chu Tước Hắc Đạo");
}

/// Đối chiếu thêm (cùng trang công khai, tra ngày 2026-11-06):
/// 2026-11-08 âm 30/9, Bính Tuất, Trực Bế — tốt: Thiên Đức, Nguyệt Đức, Thiên Mã, Mãn Đức Tinh;
/// xấu: Thổ Phủ, Bạch Hổ, Tội Chỉ, Tam Tang, Ly Sàng, Quỷ Khốc; xung: Mậu Thìn, Nhâm Thìn, Nhâm Ngọ, Nhâm Tý.
///
/// Quy tắc có nguồn cho ra đúng các sao này, không thừa không thiếu. Ngày này lịch tiết đã sang
/// tháng Hợi (Lập Đông 07/11) nhưng trang vẫn dùng bảng tháng 9 âm → bằng chứng bảng sao tra theo
/// tháng âm (rules.md §Quy ước tháng).
#[test]
fn fixture_2026_11_08() {
    let c = ctx(9, 30, BINH, ZTUAT, 11);
    let e = evaluate(&c);
    let mut tot = e.sao_tot.clone();
    tot.sort();
    let mut w = vec!["Thiên Đức", "Nguyệt Đức", "Thiên Mã", "Mãn Đức Tinh"];
    w.sort();
    assert_eq!(tot, w);
    let mut xau = e.sao_xau.clone();
    xau.sort();
    let mut w = vec![
        "Thổ Phủ",
        "Bạch Hổ Hắc Đạo",
        "Tội Chỉ",
        "Tam Tang",
        "Ly Sàng",
        "Quỷ Khốc",
    ];
    w.sort();
    assert_eq!(xau, w);
    assert_eq!(
        e.xung_tuoi,
        ["Mậu Thìn", "Nhâm Thìn", "Nhâm Ngọ", "Nhâm Tý"]
    );
}

/// 2026-11-10 âm 2/10, Mậu Tý, Trực Trừ — tốt: Thiên Mã, U Vi Tinh, Yếu Yên;
/// xấu: Bạch Hổ, Nguyệt Kiến Chuyển Sát, Phủ Đầu Sát (trang ghi "Phủ đầu dát"); xung: Bính Ngọ, Giáp Ngọ.
#[test]
fn fixture_2026_11_10() {
    let c = ctx(10, 2, MAU, ZTY, 1);
    let e = evaluate(&c);
    let mut tot = e.sao_tot.clone();
    tot.sort();
    let mut w = vec!["Thiên Mã", "U Vi Tinh", "Yếu Yên"];
    w.sort();
    assert_eq!(tot, w);
    let mut xau = e.sao_xau.clone();
    xau.sort();
    let mut w = vec!["Bạch Hổ Hắc Đạo", "Nguyệt Kiến Chuyển Sát", "Phủ Đầu Sát"];
    w.sort();
    assert_eq!(xau, w);
    assert_eq!(e.xung_tuoi, ["Bính Ngọ", "Giáp Ngọ"]);
}

#[test]
fn fixture_scores_reasonable() {
    let c = ctx(9, 27, QUY, ZMUI, 9);
    // Tam Nương là kỵ chính của cưới hỏi → không cho phép.
    assert!(!suitability(&c, "cuoi_hoi", None).allowed);
    // Chu Tước kỵ chính nhập trạch.
    assert!(!suitability(&c, "nhap_trach", None).allowed);
    // Địa Phá kỵ chính động thổ.
    assert!(!suitability(&c, "dong_tho", None).allowed);
    let e = evaluate(&c);
    assert!(has(&e.ky, "Cưới hỏi"));
}

#[test]
fn sat_cong_truc_tinh_structure() {
    // Kiểm tra cấu trúc (rules.md G45–G47): Trực Tinh nhóm g = Sát Cống nhóm g−1 (Mạnh→Trọng→Quý),
    // và trong mỗi nhóm các ngày cách nhau đúng 9 vị trí trong vòng 60 Giáp Tý.
    let get = |id: &str| -> [&'static [(u8, u8)]; 3] {
        match GOOD_STARS.iter().find(|s| s.id == id).unwrap().rule {
            Rule::CanChiByGroup(g) => g,
            _ => unreachable!(),
        }
    };
    let sc = get("G45");
    let tt = get("G46");
    assert_eq!(tt[1], sc[0]);
    assert_eq!(tt[2], sc[1]);
    for g in [sc, tt, get("G47")] {
        for grp in g {
            let idx: Vec<usize> = grp
                .iter()
                .map(|&(c, z)| sexagenary(c, z).expect("cặp can chi hợp lệ"))
                .collect();
            for w in idx.windows(2) {
                assert_eq!((w[0] + 9) % 60, w[1]);
            }
        }
    }
}

#[test]
fn stars_coinciding_with_than() {
    // Nguồn N01 ghi các sao trùng vị trí với 12 thần: Thiên Thành = Ngọc Đường, Thiên Quan = Tư Mệnh,
    // Thiên Tài = Kim Quỹ, Địa Tài = Bảo Quang (Kim Đường), Minh Tinh = Thiên Lao, Thiên Mã = Bạch Hổ.
    // Kiểm tra bảng sao khớp với quy tắc 12 thần (cùng quy tắc với lõi).
    let pairs = [
        ("Thiên Thành", "Ngọc Đường Hoàng Đạo"),
        ("Thiên Quan", "Tư Mệnh Hoàng Đạo"),
        ("Thiên Tài", "Kim Quỹ Hoàng Đạo"),
        ("Địa Tài", "Kim Đường Hoàng Đạo"),
        ("Minh Tinh", "Thiên Lao Hắc Đạo"),
        ("Thiên Mã", "Bạch Hổ Hắc Đạo"),
    ];
    for lm in 1..=12u8 {
        for z in 0..12u8 {
            let e = evaluate(&ctx(lm, 1, z % 10, z, 0));
            for (star, than) in pairs {
                let t = has(&e.sao_tot, than) || has(&e.sao_xau, than);
                assert_eq!(
                    has(&e.sao_tot, star),
                    t,
                    "tháng {lm} chi {z}: {star} / {than}"
                );
            }
        }
    }
}

#[test]
fn quan_nhat_and_nguyet_pha() {
    assert!(has(&evaluate(&ctx(2, 1, AT, ZMAO, 0)).sao_tot, "Quan Nhật"));
    assert!(!has(
        &evaluate(&ctx(1, 1, AT, ZMAO, 0)).sao_tot,
        "Quan Nhật"
    ));
    // Nguyệt Phá (bản đã chọn, rules.md B08): tháng 2 ở Tuất, tháng 8 ở Thìn.
    assert!(has(
        &evaluate(&ctx(2, 1, GIAP, ZTUAT, 0)).sao_xau,
        "Nguyệt Phá"
    ));
    assert!(!has(
        &evaluate(&ctx(2, 1, AT, ZDAU, 0)).sao_xau,
        "Nguyệt Phá"
    ));
    assert!(has(
        &evaluate(&ctx(8, 1, GIAP, ZTHIN, 0)).sao_xau,
        "Nguyệt Phá"
    ));
    // Trùng Phục tháng 10 = can Bính (rules.md B33).
    assert!(has(
        &evaluate(&ctx(10, 1, BINH, ZTY, 0)).sao_xau,
        "Trùng Phục"
    ));
    assert!(!has(
        &evaluate(&ctx(10, 1, NHAM, ZTY, 0)).sao_xau,
        "Trùng Phục"
    ));
}

/// Sinh bảng định nghĩa quy tắc từ chính mã (dùng để viết rules.md, bảo đảm tài liệu khớp mã).
/// Chạy: `cargo test dump_rule_table -- --ignored --nocapture`
#[test]
#[ignore]
fn dump_rule_table() {
    let mname = |m: usize| {
        [
            "Giêng", "Hai", "Ba", "Tư", "Năm", "Sáu", "Bảy", "Tám", "Chín", "Mười", "M.Một", "Chạp",
        ][m]
    };
    let acts = |a: Acts| -> String {
        match a {
            Acts::None => "—".into(),
            Acts::All => "mọi việc".into(),
            Acts::AllEsp(l) => format!(
                "mọi việc, nhất là {}",
                l.iter()
                    .map(|&i| ACTIVITIES[i as usize].id)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Acts::Some(l) => l
                .iter()
                .map(|&i| ACTIVITIES[i as usize].id)
                .collect::<Vec<_>>()
                .join(", "),
        }
    };
    let all: Vec<&Star> = SPECIAL_DAYS
        .iter()
        .chain(GOOD_STARS.iter())
        .chain(BAD_STARS.iter())
        .collect();
    for st in all {
        let def = match st.rule {
            Rule::LunarDays(d) => format!("ngày âm {:?} mọi tháng", d),
            Rule::MonthDays(md) => md
                .iter()
                .map(|(m, d)| format!("{d}/{m}"))
                .collect::<Vec<_>>()
                .join(", "),
            Rule::ThapAc => THAP_AC
                .iter()
                .map(|&(g, m, c, z)| {
                    let gs = ["Giáp/Kỷ", "Ất/Canh", "Bính/Tân", "Đinh/Nhâm", "Mậu/Quý"][g as usize];
                    format!("năm {gs} tháng {m}: {}", can_chi_name(c, z))
                })
                .collect::<Vec<_>>()
                .join("; "),
            _ => {
                let mut cells = Vec::new();
                for m in 0..12usize {
                    let c = ctx(m as u8 + 1, 1, 0, 0, 0);
                    let mut hits = Vec::new();
                    for n in 0..60u8 {
                        let mut d = c;
                        d.day_can = n % 10;
                        d.day_chi = n % 12;
                        if rule_hits(&st.rule, &d) {
                            hits.push((n % 10, n % 12));
                        }
                    }
                    // Rút gọn: theo chi (đủ 5 can), theo can (đủ 6 chi), hoặc liệt kê can chi.
                    let mut parts = Vec::new();
                    let mut used = vec![false; hits.len()];
                    for z in 0..12u8 {
                        let k: Vec<usize> = (0..hits.len()).filter(|&i| hits[i].1 == z).collect();
                        if k.len() == 5 {
                            parts.push(CHI_NAME[z as usize].to_string());
                            for i in k {
                                used[i] = true;
                            }
                        }
                    }
                    for cn in 0..10u8 {
                        let k: Vec<usize> = (0..hits.len())
                            .filter(|&i| !used[i] && hits[i].0 == cn)
                            .collect();
                        if k.len() == 6 {
                            parts.push(format!("can {}", CAN_NAME[cn as usize]));
                            for i in k {
                                used[i] = true;
                            }
                        }
                    }
                    for (i, &(cn, z)) in hits.iter().enumerate() {
                        if !used[i] {
                            parts.push(can_chi_name(cn, z));
                        }
                    }
                    cells.push(if parts.is_empty() {
                        "—".to_string()
                    } else {
                        parts.join(", ")
                    });
                }
                if cells.iter().all(|x| *x == cells[0]) {
                    format!("mọi tháng: {}", cells[0])
                } else {
                    cells
                        .iter()
                        .enumerate()
                        .map(|(m, x)| format!("{}: {x}", mname(m)))
                        .collect::<Vec<_>>()
                        .join("; ")
                }
            }
        };
        println!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            st.id,
            st.name,
            if st.good { "tốt" } else { "xấu" },
            st.weight,
            def,
            if st.good { acts(st.tot) } else { "—".into() },
            acts(st.ky),
            acts(st.major)
        );
    }
}

#[test]
fn n3_holds_for_any_than_value() {
    // Bổ sung cho n3_major_taboo_never_allowed: thần lấy mọi giá trị 0..11 (không suy từ tháng/chi),
    // phòng trường hợp lõi truyền vào thần bất kỳ.
    for yc in 0..5u8 {
        for lm in 1..=12u8 {
            for ld in [1u8, 3, 5, 7, 8, 13, 14, 19, 23, 27, 29] {
                for n in 0..60u8 {
                    for th in 0..12u8 {
                        let mut c = ctx(lm, ld, n % 10, n % 12, 0);
                        c.year_can = yc;
                        c.than = th;
                        let (_, bad) = stars_of(&c);
                        for (i, act) in ACTIVITIES.iter().enumerate() {
                            if bad.iter().any(|s| s.major.contains(i as Act)) {
                                assert!(!suitability(&c, act.id, Some(1990)).allowed);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn major_taboos_matches_suitability() {
    for lm in 1..=12u8 {
        for ld in [1u8, 3, 5, 7, 13, 14, 23, 27] {
            for n in 0..60u8 {
                let c = ctx(lm, ld, n % 10, n % 12, (n + ld) % 12);
                for act in ACTIVITIES.iter() {
                    let m = major_taboos(&c, act.id);
                    let s = suitability(&c, act.id, None);
                    assert_eq!(m.is_empty(), s.allowed, "{lm}/{ld} n={n} {}", act.id);
                }
            }
        }
    }
    assert!(major_taboos(&ctx(1, 1, 0, 0, 0), "khong_co").is_empty());
    // 2026-11-05: Tam Nương và Tứ Thời Cô Quả là kỵ chính của cưới hỏi.
    assert_eq!(
        major_taboos(&ctx(9, 27, QUY, ZMUI, 9), "cuoi_hoi"),
        ["Tam Nương", "Tứ Thời Cô Quả"]
    );
}

/// Sinh `tsport/src/ngochap_tables.ts` từ chính bảng Rust (để bản TypeScript không chép tay bảng).
/// Chạy: `cargo test -p lvhv-ngochap emit_ts_tables -- --ignored` (ghi tệp ra tsport/src/).
#[test]
#[ignore]
fn emit_ts_tables() {
    fn nums(v: &[u8]) -> String {
        format!(
            "[{}]",
            v.iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
    fn pairs(v: &[(u8, u8)]) -> String {
        format!(
            "[{}]",
            v.iter()
                .map(|(a, b)| format!("[{a},{b}]"))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
    fn acts(a: Acts) -> String {
        match a {
            Acts::None => "{ k: \"none\" }".into(),
            Acts::All => "{ k: \"all\" }".into(),
            Acts::AllEsp(l) => format!("{{ k: \"allEsp\", l: {} }}", nums(l)),
            Acts::Some(l) => format!("{{ k: \"some\", l: {} }}", nums(l)),
        }
    }
    fn rule(r: &Rule) -> String {
        match r {
            Rule::MonthChi(t) => format!("{{ k: \"monthChi\", t: {} }}", nums(t)),
            Rule::MonthChi2(t) => format!(
                "{{ k: \"monthChi2\", t: [{}] }}",
                t.iter().map(|x| nums(x)).collect::<Vec<_>>().join(",")
            ),
            Rule::MonthCan(t) => format!("{{ k: \"monthCan\", t: {} }}", nums(t)),
            Rule::MonthCond(t) => format!(
                "{{ k: \"monthCond\", t: [{}] }}",
                t.iter()
                    .map(|c| match c {
                        Cond::Can(x) => format!("{{ can: {x} }}"),
                        Cond::Chi(x) => format!("{{ chi: {x} }}"),
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Rule::MonthCanChi(t) => format!("{{ k: \"monthCanChi\", t: {} }}", pairs(t)),
            Rule::MonthChiOpt(t) => format!(
                "{{ k: \"monthChiOpt\", t: [{}] }}",
                t.iter()
                    .map(|x| x.map_or("null".into(), |v| v.to_string()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Rule::SeasonChi(t) => format!(
                "{{ k: \"seasonChi\", t: [{}] }}",
                t.iter().map(|x| nums(x)).collect::<Vec<_>>().join(",")
            ),
            Rule::SeasonCan(t) => format!(
                "{{ k: \"seasonCan\", t: [{}] }}",
                t.iter().map(|x| nums(x)).collect::<Vec<_>>().join(",")
            ),
            Rule::SeasonCanChi(t) => format!("{{ k: \"seasonCanChi\", t: {} }}", pairs(t)),
            Rule::CanChiSet(s) => format!("{{ k: \"canChiSet\", t: {} }}", pairs(s)),
            Rule::CanChiByGroup(g) => format!(
                "{{ k: \"canChiByGroup\", t: [{}] }}",
                g.iter().map(|x| pairs(x)).collect::<Vec<_>>().join(",")
            ),
            Rule::LunarDays(d) => format!("{{ k: \"lunarDays\", t: {} }}", nums(d)),
            Rule::MonthDays(d) => format!("{{ k: \"monthDays\", t: {} }}", pairs(d)),
            Rule::ThapAc => "{ k: \"thapAc\" }".into(),
            Rule::Than(i) => format!("{{ k: \"than\", i: {i} }}"),
        }
    }
    fn star(s: &Star) -> String {
        format!(
            "  {{ id: {:?}, name: {:?}, good: {}, weight: {}, rule: {}, tot: {}, ky: {}, major: {} }},\n",
            s.id,
            s.name,
            s.good,
            s.weight,
            rule(&s.rule),
            acts(s.tot),
            acts(s.ky),
            acts(s.major)
        )
    }
    let mut o = String::new();
    o.push_str("// TỆP SINH TỰ ĐỘNG từ ngochap/src/tables.rs — không sửa tay.\n");
    o.push_str("// Tạo lại: cargo test -p lvhv-ngochap emit_ts_tables -- --ignored\n");
    o.push_str("import type { Star } from \"./ngochap.ts\";\n\n");
    o.push_str(&format!(
        "export const CAN_NAME: readonly string[] = {:?};\n",
        CAN_NAME
    ));
    o.push_str(&format!(
        "export const CHI_NAME: readonly string[] = {:?};\n",
        CHI_NAME
    ));
    o.push_str(&format!(
        "export const ACTIVITIES: ReadonlyArray<{{ readonly id: string; readonly name: string }}> = [\n{}];\n",
        ACTIVITIES
            .iter()
            .map(|a| format!("  {{ id: {:?}, name: {:?} }},\n", a.id, a.name))
            .collect::<String>()
    ));
    o.push_str(&format!(
        "export const PERSONAL: readonly number[] = {};\n",
        nums(&PERSONAL)
    ));
    o.push_str(&format!(
        "export const TRUC_SCORE: readonly number[] = [{}];\n",
        TRUC_SCORE
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",")
    ));
    o.push_str(&format!(
        "export const TRUC_NAME: readonly string[] = {:?};\n",
        TRUC_NAME
    ));
    o.push_str(&format!(
        "export const NEN_THRESHOLD = {NEN_THRESHOLD};\nexport const XUNG_PENALTY = {XUNG_PENALTY};\n"
    ));
    for (name, list) in [
        ("GOOD_STARS", GOOD_STARS),
        ("BAD_STARS", BAD_STARS),
        ("SPECIAL_DAYS", SPECIAL_DAYS),
        ("THAN", &THAN[..]),
    ] {
        o.push_str(&format!("export const {name}: readonly Star[] = [\n"));
        for s in list {
            o.push_str(&star(s));
        }
        o.push_str("];\n");
    }
    o.push_str(&format!(
        "export const THAP_AC: ReadonlyArray<readonly [number, number, number, number]> = [{}];\n",
        THAP_AC
            .iter()
            .map(|(a, b, c, d)| format!("[{a},{b},{c},{d}]"))
            .collect::<Vec<_>>()
            .join(",")
    ));
    o.push_str(
        "export const XUNG_TUOI: ReadonlyArray<ReadonlyArray<readonly [number, number]>> = [\n",
    );
    for row in XUNG_TUOI.iter() {
        o.push_str(&format!("  {},\n", pairs(row)));
    }
    o.push_str("];\n");
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../tsport/src/ngochap_tables.ts"
    );
    std::fs::write(path, o).expect("ghi tệp");
}
