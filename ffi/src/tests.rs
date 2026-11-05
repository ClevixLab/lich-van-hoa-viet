use super::*;

#[test]
fn day_info_matches_contract_example_fields() {
    // Ca đối chiếu L8 (hợp đồng §4): 2026-11-05.
    let j = day_info_json(2026, 11, 5).unwrap();
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert_eq!(v["solar"]["jdn"], 2461350);
    assert_eq!(v["solar"]["weekday"], 4);
    assert_eq!(v["lunar"]["d"], 27);
    assert_eq!(v["lunar"]["m"], 9);
    assert_eq!(v["can_chi"]["year"], "Bính Ngọ");
    assert_eq!(v["can_chi"]["day"], "Quý Mùi");
    assert_eq!(v["tiet_khi"], "Sương Giáng");
    // Thứ tự khóa cấp 1 cố định (kiểm trên chuỗi gốc; Map của serde_json tự sắp khóa).
    let keys = [
        "solar",
        "lunar",
        "can_chi",
        "tiet_khi",
        "nap_am",
        "truc",
        "tu",
        "than",
        "gio_hoang_dao",
        "ngoc_hap",
    ];
    let pos: Vec<usize> = keys
        .iter()
        .map(|k| j.find(&format!("\"{k}\":")).expect(k))
        .collect();
    assert!(pos.windows(2).all(|w| w[0] < w[1]), "{pos:?}");
}

#[test]
fn day_info_is_byte_identical_to_core() {
    // JSON của FFI phải giống hệt lõi từng byte (tiêu chí L7 áp cho mọi nền tảng).
    for (y, m, d) in [(2000, 1, 1), (2026, 11, 5), (2072, 12, 10), (3000, 12, 31)] {
        let ffi = day_info_json(y, m, d).unwrap();
        let core = lvhv_core::day_info(y, m as u32, d as u32)
            .unwrap()
            .to_json();
        assert_eq!(ffi, core, "{y}-{m}-{d}");
    }
}

#[test]
fn date_error_mapping() {
    use lvhv_core::DateError as E;
    let cases = [
        (
            E::InvalidDate {
                year: 2026,
                month: 2,
                day: 30,
            },
            "InvalidDate",
        ),
        (E::OUT_OF_RANGE, "OutOfRange"),
        (E::InvertedRange, "BadRange"),
        (E::RangeTooLong { span_days: 400 }, "BadRange"),
        (E::UnknownActivity, "UnknownActivity"),
        (E::Computation, "Unavailable"),
    ];
    for (e, want) in cases {
        let got = match LvhvError::from(e) {
            LvhvError::InvalidDate(_) => "InvalidDate",
            LvhvError::OutOfRange(_) => "OutOfRange",
            LvhvError::BadRange(_) => "BadRange",
            LvhvError::UnknownActivity(_) => "UnknownActivity",
            LvhvError::Unavailable(_) => "Unavailable",
        };
        assert_eq!(got, want, "{e:?}");
        assert!(!LvhvError::from(e).to_string().is_empty());
    }
}

#[test]
fn lunar_year_of_birth_before_and_after_tet() {
    // Tết Đinh Tỵ 1977 là 18/02/1977: sinh 20/01/1977 → năm âm 1976 (Bính Thìn).
    assert_eq!(
        lunar_year_of_birth(SolarDay {
            y: 1977,
            m: 1,
            d: 20
        })
        .unwrap(),
        1976
    );
    assert_eq!(
        lunar_year_of_birth(SolarDay {
            y: 1977,
            m: 3,
            d: 1
        })
        .unwrap(),
        1977
    );
    assert_eq!(year_can_chi(1976), "Bính Thìn");
    assert!(matches!(
        lunar_year_of_birth(SolarDay {
            y: 1899,
            m: 12,
            d: 31
        }),
        Err(LvhvError::OutOfRange(_))
    ));
    assert!(matches!(
        lunar_year_of_birth(SolarDay {
            y: 1990,
            m: 2,
            d: 30
        }),
        Err(LvhvError::InvalidDate(_))
    ));
}

#[test]
fn invalid_and_out_of_range_dates_are_errors_not_panics() {
    assert!(matches!(
        day_info_json(2026, 2, 30),
        Err(LvhvError::InvalidDate(_))
    ));
    assert!(matches!(
        day_info_json(2026, 13, 1),
        Err(LvhvError::InvalidDate(_))
    ));
    assert!(matches!(
        day_info_json(1999, 12, 31),
        Err(LvhvError::OutOfRange(_))
    ));
    assert!(matches!(
        solar_to_lunar(3001, 1, 1),
        Err(LvhvError::OutOfRange(_))
    ));
    assert!(matches!(
        cadao_for(2026, 0, 1),
        Err(LvhvError::InvalidDate(_))
    ));
    assert!(matches!(
        day_info_json(2026, -1, 1),
        Err(LvhvError::InvalidDate(_))
    ));
    assert!(matches!(
        day_info_json(i32::MAX, 1, 1),
        Err(LvhvError::OutOfRange(_))
    ));
    assert_eq!(lunar_to_solar(2026, 13, 1, false), None);
    assert_eq!(lunar_to_solar(2026, 1, 31, false), None);
    assert_eq!(lunar_to_solar(1999, 1, 1, false), None);
}

#[test]
fn solar_lunar_round_trip_sample() {
    let mut day = SolarDay {
        y: 2026,
        m: 1,
        d: 1,
    };
    for _ in 0..800 {
        let l = solar_to_lunar(day.y, day.m, day.d).unwrap();
        assert_eq!(lunar_to_solar(l.y, l.m, l.d, l.leap), Some(day));
        day = add_days(day, 1).unwrap();
    }
}

#[test]
fn month_lunar_days_len() {
    assert_eq!(month_lunar_days(2026, 11).unwrap().len(), 30);
    assert_eq!(month_lunar_days(2028, 2).unwrap().len(), 29);
    assert_eq!(month_lunar_days(2026, 11).unwrap()[4].d, 27);
}

#[test]
fn year_can_chi_examples() {
    assert_eq!(year_can_chi(1976), "Bính Thìn");
    assert_eq!(year_can_chi(2026), "Bính Ngọ");
    assert_eq!(year_can_chi(1984), "Giáp Tý");
}

#[test]
fn cadao_select_md_example() {
    // select.md: 2026-11-06 → JDN 2461351 → pos 412.
    let v: serde_json::Value = serde_json::from_str(&cadao_for(2026, 11, 6).unwrap()).unwrap();
    assert_eq!(v["order"], 412);
    assert!(!v["text"].as_str().unwrap().is_empty());
    assert!(!v["sources"].as_array().unwrap().is_empty());
}

#[test]
fn cadao_order_is_permutation_and_no_repeat_in_n_days() {
    let n = cadao_count() as i64;
    assert!(n > 0);
    let start = jdn_from_date(2026, 1, 1);
    let mut seen = std::collections::HashSet::new();
    for j in start..start + n {
        let e = cadao::entry_for_jdn(j).unwrap();
        assert!(seen.insert(e.id.clone()), "lặp câu {} trong {n} ngày", e.id);
    }
}

#[test]
fn cadao_attribution_still_in_sources_md() {
    assert!(
        cadao::SOURCES_MD.contains(cadao::ATTRIBUTION),
        "Lời ghi công trong SOURCES.md mục C.1 đã đổi — cập nhật cadao::ATTRIBUTION"
    );
}

#[test]
fn rating_thresholds_match_pwa() {
    // pwa/src/format.ts: >= 4 Tốt, >= 1 Khá tốt, >= -3 Bình thường, còn lại Xấu.
    for (sc, label, level) in [
        (13, "Tốt", 3),
        (4, "Tốt", 3),
        (3, "Khá tốt", 2),
        (1, "Khá tốt", 2),
        (0, "Bình thường", 1),
        (-3, "Bình thường", 1),
        (-4, "Xấu", 0),
    ] {
        let r = day_rating(sc);
        assert_eq!((r.label.as_str(), r.level), (label, level), "điểm {sc}");
    }
}

#[test]
fn build_info_is_json() {
    let v: serde_json::Value = serde_json::from_str(&build_info_json()).unwrap();
    assert_eq!(v["min_year"], 2000);
}

mod ngochap {
    use super::*;

    #[test]
    fn activities_json_has_ids() {
        let v: serde_json::Value = serde_json::from_str(&activities_json()).unwrap();
        let a = v.as_array().unwrap();
        assert!(!a.is_empty());
        assert_eq!(a[0]["id"], "cuoi_hoi");
        assert_eq!(a[0]["name"], "Cưới hỏi");
    }

    #[test]
    fn day_info_has_ngoc_hap_filled() {
        let v: serde_json::Value =
            serde_json::from_str(&day_info_json(2026, 11, 5).unwrap()).unwrap();
        assert!(!v["ngoc_hap"]["sao_xau"].as_array().unwrap().is_empty());
    }

    #[test]
    fn find_good_days_sorted_and_allowed() {
        let from = SolarDay {
            y: 2026,
            m: 11,
            d: 6,
        };
        let to = add_days(from, 89).unwrap();
        let s = find_good_days_json("cuoi_hoi".into(), from, to, Some(1976)).unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        let a = v.as_array().unwrap();
        let mut prev = i64::MAX;
        for g in a {
            let sc = g["score"].as_i64().unwrap();
            assert!(sc <= prev);
            assert_eq!(g["allowed"], true);
            prev = sc;
            // N3 theo lõi: chỉ có ngày allowed (không phạm sao kỵ chính). LƯU Ý: lõi KHÔNG loại ngày mà
            // tờ lịch ghi "Kỵ" vì sao xấu nêu đích danh việc + điểm âm (xem ffi/README.md,
            // `cargo run -p lvhv-ffi --release --example gd_check`). FFI giữ đúng đầu ra của lõi.
        }
    }

    #[test]
    fn find_good_days_is_core_output() {
        let from = SolarDay {
            y: 2026,
            m: 1,
            d: 1,
        };
        let to = SolarDay {
            y: 2026,
            m: 3,
            d: 31,
        };
        let ffi = find_good_days_json("khai_truong".into(), from, to, Some(1976)).unwrap();
        let core = lvhv_core::find_good_days(
            "khai_truong",
            lvhv_core::SolarDate {
                year: 2026,
                month: 1,
                day: 1,
            },
            lvhv_core::SolarDate {
                year: 2026,
                month: 3,
                day: 31,
            },
            Some(1976),
        )
        .unwrap();
        let want = format!(
            "[{}]",
            core.iter()
                .map(|g| g.to_json())
                .collect::<Vec<_>>()
                .join(",")
        );
        assert_eq!(ffi, want);
    }

    #[test]
    fn find_good_days_errors() {
        let a = SolarDay {
            y: 2026,
            m: 1,
            d: 1,
        };
        let b = SolarDay {
            y: 2027,
            m: 6,
            d: 1,
        };
        assert!(matches!(
            find_good_days_json("cuoi_hoi".into(), a, b, None),
            Err(LvhvError::BadRange(_))
        ));
        assert!(matches!(
            find_good_days_json("cuoi_hoi".into(), b, a, None),
            Err(LvhvError::BadRange(_))
        ));
        assert!(matches!(
            find_good_days_json("khong_co".into(), a, a, None),
            Err(LvhvError::UnknownActivity(_))
        ));
    }
}
