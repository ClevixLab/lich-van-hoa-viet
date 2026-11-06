//! Kiểm thử phần v0.2 của lớp FFI (kho, dẫn xuất, danh sách, nhắc, .ics, sao lưu).

use super::*;
use lvhv_memorial::remind::SENTINEL_ID;
use serde_json::{json, Value};
use std::sync::Arc;

const EXAMPLE: &str = include_str!("../../docs/v0.2/schema/example-backup-v1.json");
const R3: &str = include_str!("../../memorial/testdata/r3-dataset.json");

fn v(s: &str) -> Value {
    serde_json::from_str(s).unwrap()
}

fn tmp_store() -> (Arc<LvhvStore>, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("lvhv-ffi-test-{}", new_uuid().unwrap()));
    let s = LvhvStore::open(dir.to_string_lossy().into_owned()).unwrap();
    (s, dir)
}

fn sd(y: i32, m: i32, d: i32) -> SolarDay {
    SolarDay { y, m, d }
}

/// Giây Unix của `y-m-d hh:mm` giờ +07:00.
fn unix_vn(y: i32, m: u32, d: u32, hh: i64, mm: i64) -> i64 {
    (lvhv_core::jd::jdn_from_date(y, m, d) - 2_440_588) * 86_400 + hh * 3600 + mm * 60 - 7 * 3600
}

fn vn() -> DeviceZone {
    DeviceZone {
        offset_min: 420,
        transitions: vec![],
    }
}

fn person_a() -> Value {
    let mut p = v(EXAMPLE)["persons"][0].clone();
    p["photo_id"] = Value::Null;
    // Trường dẫn xuất bỏ trống: FFI tự tính.
    for k in ["lunar", "solar", "lunar_source", "method", "tuoi"] {
        p["death"].as_object_mut().unwrap().remove(k);
    }
    for k in ["created_at", "updated_at", "rev"] {
        p.as_object_mut().unwrap().remove(k);
    }
    p
}

#[test]
fn store_person_event_settings_crud() {
    let (s, dir) = tmp_store();
    assert_eq!(
        v(&s.event_types_json().unwrap()).as_array().unwrap().len(),
        4
    );
    let set = v(&s.settings_json().unwrap());
    assert_eq!(set["upcoming_days"], 30);

    // Người đã khuất: đổi ngày mất dương → âm khi lưu (02 §1), hưởng thọ 86 (02 §4.2).
    let saved = v(&s.save_person_json(person_a().to_string()).unwrap());
    assert_eq!(
        saved["death"]["lunar"],
        json!({"year": 2025, "month": 6, "day": 15, "leap": true})
    );
    assert_eq!(saved["death"]["lunar_source"], "converted");
    assert_eq!(saved["death"]["method"], "jpl");
    assert_eq!(saved["death"]["tuoi"]["value"], 86);
    assert_eq!(saved["death"]["tuoi"]["label"], "huong_tho");
    assert_eq!(saved["rev"], 1);
    let created = saved["created_at"].clone();
    // Sửa: rev tăng, created_at giữ nguyên.
    let mut p2 = saved.clone();
    p2["quan_he"] = json!("ông nội (bên nội)");
    let saved2 = v(&s.save_person_json(p2.to_string()).unwrap());
    assert_eq!(saved2["rev"], 2);
    assert_eq!(saved2["created_at"], created);
    assert_eq!(v(&s.persons_json().unwrap()).as_array().unwrap().len(), 1);

    // Người đang sống, id do FFI sinh.
    let mut c = v(EXAMPLE)["persons"][2].clone();
    c["id"] = json!("");
    let c = v(&s.save_person_json(c.to_string()).unwrap());
    let cid = c["id"].as_str().unwrap().to_owned();
    assert_eq!(cid.len(), 36);
    assert_eq!(c["birth"]["can_chi"], "Kỷ Tỵ");

    // Sự kiện gắn với người đang sống; xóa người → gỡ id khỏi sự kiện.
    let mut e = v(EXAMPLE)["events"][0].clone();
    e["person_ids"] = json!([cid]);
    let e = v(&s.save_event_json(e.to_string()).unwrap());
    assert_eq!(e["person_ids"], json!([cid]));
    s.delete_person(cid.clone()).unwrap();
    let evs = v(&s.events_json().unwrap());
    assert_eq!(evs[0]["person_ids"], json!([]));
    assert_eq!(evs[0]["rev"], 2);
    s.delete_event(e["id"].as_str().unwrap().into()).unwrap();
    assert!(matches!(
        s.delete_event(e["id"].as_str().unwrap().into()),
        Err(LvhvDataError::NotFound(_))
    ));

    // Ngày không tồn tại → Invalid, không ghi gì.
    let mut bad = person_a();
    bad["id"] = json!(new_uuid().unwrap());
    bad["death"]["input"] =
        json!({"calendar":"lunar","precision":"day","year":2026,"month":7,"day":30,"leap":false});
    match s.save_person_json(bad.to_string()) {
        Err(LvhvDataError::Invalid(m)) => assert!(m.contains("29 ngày"), "{m}"),
        other => panic!("{other:?}"),
    }
    assert_eq!(v(&s.persons_json().unwrap()).as_array().unwrap().len(), 1);

    // Cài đặt.
    let mut set = set;
    set["reminders_enabled"] = json!(false);
    s.put_settings_json(set.to_string()).unwrap();
    assert_eq!(v(&s.settings_json().unwrap())["reminders_enabled"], false);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn derive_helpers() {
    let d = v(&save_death_json(
        r#"{"calendar":"solar","precision":"day","year":2025,"month":8,"day":8,"leap":false}"#
            .into(),
    )
    .unwrap());
    assert_eq!(d["lunar"]["leap"], true);
    let b = v(&derive_birth_json(
        r#"{"calendar":"solar","precision":"day","year":1990,"month":1,"day":20,"leap":false}"#
            .into(),
    )
    .unwrap());
    assert_eq!(b["lunar_year"], 1989);
    assert_eq!(b["can_chi"], "Kỷ Tỵ");
    assert!(matches!(
        save_death_json(
            r#"{"calendar":"solar","precision":"day","year":2026,"month":2,"day":30,"leap":false}"#
                .into()
        ),
        Err(LvhvDataError::Invalid(_))
    ));
    let p = v(&derive_person_json(person_a().to_string()).unwrap());
    assert_eq!(p["death"]["tuoi"]["value"], 86);

    // Tóm tắt: 02 §4.3 (sinh 1990-01-20, ngày 2026-11-06 → 36 tuổi dương, 38 tuổi âm) và §4.2.
    let ex = v(EXAMPLE);
    let c = v(&person_summary_json(ex["persons"][2].to_string(), sd(2026, 11, 6)).unwrap());
    assert_eq!(
        (c["tuoi_duong"].clone(), c["tuoi_am"].clone()),
        (json!(36), json!(38))
    );
    assert_eq!(c["display_name"], "Chị Lê Thị C");
    let a = v(&person_summary_json(ex["persons"][0].to_string(), sd(2026, 11, 6)).unwrap());
    assert_eq!(a["tuoi_mat_text"], "Hưởng thọ 86 tuổi");
    assert_eq!(a["death_lunar_text"], "15/6 nhuận âm lịch năm Ất Tỵ (2025)");
    assert_eq!(a["tuoi_duong"], Value::Null);
}

#[test]
fn lists_upcoming_and_period() {
    // 02 §7.1 ví dụ: G-A giỗ đầu 2026-07-28.
    let up = v(&upcoming_json(EXAMPLE.into(), sd(2026, 7, 1), 30).unwrap());
    let g = up
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["class"] == "gio" && x["date"] == "2026-07-28")
        .expect("giỗ đầu");
    assert_eq!(g["headline"], "Giỗ đầu Ông Nguyễn Văn A");
    assert_eq!(g["days"], 27);
    assert_eq!(g["days_text"], "Còn 27 ngày");
    assert_eq!(g["label"], "gio_dau");

    let r = period_bounds("tuan".into(), sd(2026, 1, 1)).unwrap();
    assert_eq!((r.from, r.to), (sd(2025, 12, 29), sd(2026, 1, 4)));
    let r = period_prev("quy".into(), sd(2026, 2, 1)).unwrap();
    assert_eq!((r.from, r.to), (sd(2025, 10, 1), sd(2025, 12, 31)));
    let r = period_next("thang".into(), sd(2026, 12, 5)).unwrap();
    assert_eq!((r.from, r.to), (sd(2027, 1, 1), sd(2027, 1, 31)));
    assert!(period_bounds("ngay".into(), sd(2026, 1, 1)).is_err());

    // Cùng nội dung với lvhv-memorial (dump r13): mọi kỳ của năm 2027, ba bộ lọc.
    let ds = lvhv_memorial::model::parse_dataset(R3).unwrap();
    let cal = lvhv_memorial::Cal::get(lvhv_core::DEFAULT_METHOD);
    for kind in ["tuan", "thang", "quy", "nam"] {
        for f in ["tat_ca", "da_mat", "dang_song"] {
            let p = v(
                &events_in_period_json(R3.into(), kind.into(), sd(2027, 3, 31), f.into()).unwrap(),
            );
            let m = lvhv_memorial::events_in_period(
                cal,
                &ds,
                lvhv_memorial::PeriodKind::parse(kind).unwrap(),
                lvhv_memorial::model::Ymd::new(2027, 3, 31),
                lvhv_memorial::PersonFilter::parse(f).unwrap(),
            );
            let mut want = String::new();
            for it in &m.items {
                lvhv_memorial::period::item_json(&mut want, it);
                want.push('\n');
            }
            let mut got = String::new();
            for it in p["items"].as_array().unwrap() {
                // Các khóa của Phụ lục A.6 phải trùng giá trị.
                let w: Vec<Value> = want.lines().map(v).collect();
                let i = got.lines().count();
                for k in [
                    "date",
                    "lunar",
                    "class",
                    "entity_id",
                    "person_ids",
                    "name",
                    "key",
                    "n",
                    "label",
                ] {
                    assert_eq!(it[k], w[i][k], "{kind} {f} {k}");
                }
                got.push_str("x\n");
            }
            assert_eq!(got.lines().count(), m.items.len());
            assert_eq!(p["from"], m.from.iso());
        }
    }
}

#[test]
fn reminders_reschedule_android_chain() {
    let tz = vn();
    let now = unix_vn(2026, 7, 1, 0, 0);
    let rs = v(&reminders_json(EXAMPLE.into(), now, tz.clone(), 60, true).unwrap());
    let a: Vec<&Value> = rs
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| {
            r["id"]
                .as_str()
                .unwrap()
                .starts_with("lvhv:0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e11:2026:")
        })
        .collect();
    assert_eq!(a.len(), 2);
    assert_eq!(a[0]["fire_local"], "2026-07-25T07:00");
    assert_eq!(a[0]["at_unix_s"], unix_vn(2026, 7, 25, 7, 0));
    assert_eq!(a[0]["title"], "Còn 3 ngày: Giỗ đầu Ông Nguyễn Văn A");
    assert_eq!(a[1]["fire_local"], "2026-07-28T07:00");
    // O-9: mặc định không hiện tên.
    let hidden = v(&reminders_json(EXAMPLE.into(), now, tz.clone(), 60, false).unwrap());
    assert!(hidden
        .as_array()
        .unwrap()
        .iter()
        .all(|r| !r["title"].as_str().unwrap().contains("Nguyễn")));

    // reschedule: lần đầu thêm hết; lần sau với hàng đợi đã đặt → không đổi gì.
    let plan = v(&reschedule_json(R3.into(), now, tz.clone(), "[]".into(), false).unwrap());
    let keep = plan["keep"].as_array().unwrap();
    assert_eq!(keep.len(), 64);
    assert_eq!(plan["add"].as_array().unwrap().len(), keep.len());
    let pending: Vec<Value> = keep
        .iter()
        .map(|p| json!({"id": p["id"], "at_unix_s": p["at_unix_s"], "content": p["content"]}))
        .collect();
    let plan2 = v(&reschedule_json(
        R3.into(),
        now,
        tz.clone(),
        Value::Array(pending).to_string(),
        false,
    )
    .unwrap());
    assert!(plan2["add"].as_array().unwrap().is_empty());
    assert!(plan2["remove"].as_array().unwrap().is_empty());
    assert!(plan2["replace"].as_array().unwrap().is_empty());
    // Bộ R3 có > 64 lời nhắc trong 400 ngày → thông báo nhắc mở ứng dụng cùng phút với cái thứ 63.
    assert_eq!(keep.len(), 64);
    assert_eq!(keep[63]["id"], SENTINEL_ID);
    assert_eq!(keep[63]["at_unix_s"], keep[62]["at_unix_s"]);
    assert_eq!(sentinel_info().id, SENTINEL_ID);

    // Chuỗi báo thức Android: chạy lúc 07:01 ngày 2026-07-25, lần trước 06:00 → đăng lời nhắc 07:00,
    // báo thức kế tiếp là lời nhắc sớm nhất sau đó.
    let step = v(&android_alarm_step_json(
        EXAMPLE.into(),
        unix_vn(2026, 7, 25, 7, 1),
        Some(unix_vn(2026, 7, 25, 6, 0)),
        tz.clone(),
        false,
    )
    .unwrap());
    let post = step["post"].as_array().unwrap();
    assert!(post
        .iter()
        .any(|r| r["id"] == "lvhv:0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e11:2026:3"));
    assert!(post.iter().all(|r| r["fire_local"] == "2026-07-25T07:00"));
    assert_eq!(step["dropped_late"], 0);
    let next = step["next_at_unix_s"].as_i64().unwrap();
    // Lời nhắc kế tiếp: đúng ngày giỗ đầu 2026-07-28 07:00 (+07).
    assert_eq!(next, unix_vn(2026, 7, 28, 7, 0));
    let all = v(&reminders_json(
        EXAMPLE.into(),
        unix_vn(2026, 7, 25, 7, 1),
        tz.clone(),
        400,
        false,
    )
    .unwrap());
    assert_eq!(all[0]["at_unix_s"].as_i64().unwrap(), next);
    // Máy tắt 3 ngày: lời nhắc trễ hơn 24 giờ bị bỏ (đếm), trễ ≤ 24 giờ được đăng.
    let late = v(&android_alarm_step_json(
        EXAMPLE.into(),
        unix_vn(2026, 7, 28, 8, 0),
        Some(unix_vn(2026, 7, 24, 0, 0)),
        tz,
        false,
    )
    .unwrap());
    // 2026-07-25 07:00 (trễ > 24 giờ) bị bỏ; 2026-07-28 07:00 (trễ 1 giờ) được đăng.
    assert_eq!(late["dropped_late"], 1);
    assert_eq!(late["post"].as_array().unwrap().len(), 1);
    assert!(late["post"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["fire_local"] == "2026-07-28T07:00"));
}

#[test]
fn ics_export_example() {
    let f = ics_export(
        EXAMPLE.into(),
        sd(2026, 7, 1),
        unix_vn(2026, 11, 6, 10, 15),
        None,
    )
    .unwrap();
    assert_eq!(f.file_name, "LichVanHoaViet-ngay-gio-2026-07-01.ics");
    assert!(f.content.starts_with("BEGIN:VCALENDAR\r\n"));
    assert!(f.content.contains("DTSTAMP:20261106T031500Z\r\n"));
    let unfolded = f.content.replace("\r\n ", "");
    assert!(unfolded.contains("SUMMARY:Giỗ đầu Ông Nguyễn Văn A\r\n"));
    assert_eq!(f.events as usize, unfolded.matches("BEGIN:VEVENT").count());
    assert_eq!((f.persons, f.events), (3, 60));
    let only = ics_export(
        EXAMPLE.into(),
        sd(2026, 7, 1),
        0,
        Some(vec!["0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01".into()]),
    )
    .unwrap();
    assert_eq!(only.persons, 1);
    assert_eq!(only.events, 10);
}

#[test]
fn backup_export_import_roundtrip() {
    let (s, dir) = tmp_store();
    s.save_person_json(person_a().to_string()).unwrap();
    let mut c = v(EXAMPLE)["persons"][2].clone();
    c["id"] = json!("");
    s.save_person_json(c.to_string()).unwrap();
    let file = s
        .export_backup("mật khẩu đúng".into(), "0.2.0".into(), "android".into())
        .unwrap();
    match open_backup(file.clone(), "mật khẩu sai".into()) {
        Err(LvhvDataError::Backup(m)) => assert_eq!(m, "Sai mật khẩu hoặc tệp bị hỏng"),
        Err(e) => panic!("{e:?}"),
        Ok(_) => panic!("sai mật khẩu mà vẫn mở được"),
    }
    let mut broken = file.clone();
    let n = broken.len();
    broken[n - 1] ^= 1;
    assert!(matches!(
        open_backup(broken, "mật khẩu đúng".into()),
        Err(LvhvDataError::Backup(_))
    ));
    let vb = open_backup(file, "mật khẩu đúng".into()).unwrap();
    let pv = vb.preview();
    assert_eq!((pv.persons, pv.events, pv.photos), (2, 0, 0));
    assert_eq!(pv.platform, "android");

    let (s2, dir2) = tmp_store();
    s2.apply_import(vb, BackupImportMode::Replace).unwrap();
    assert_eq!(s.persons_json().unwrap(), s2.persons_json().unwrap());
    // Bộ dữ liệu của kho dùng được cho các hàm danh sách.
    let up = v(&upcoming_json(s2.dataset_json().unwrap(), sd(2027, 7, 1), 30).unwrap());
    assert!(up.as_array().unwrap().iter().any(|x| x["class"] == "gio"));
    assert!(!s2.backup_reminder_due().unwrap());
    let _ = std::fs::remove_dir_all(dir);
    let _ = std::fs::remove_dir_all(dir2);
}

#[test]
fn uuid_is_v4() {
    let u = new_uuid().unwrap();
    assert!(lvhv_store::validate::is_uuid_v4(&u), "{u}");
    assert_ne!(u, new_uuid().unwrap());
}

/// MA3-14-01: lời nhắc ở xa hơn 400 ngày vẫn là báo thức kế tiếp của chuỗi Android (02 §7.3).
#[test]
fn android_next_alarm_has_no_window() {
    let mut doc = v(EXAMPLE);
    doc["persons"] = json!([]);
    doc["photos"] = json!([]);
    let mut e = doc["events"][0].clone();
    e["person_ids"] = json!([]);
    e["recurring"] = json!(false);
    e["date"] =
        json!({"calendar":"solar","precision":"day","year":2028,"month":6,"day":1,"leap":false});
    e["reminder"]["offsets_days"] = json!([0, 1]);
    doc["events"] = json!([e]);
    let now = unix_vn(2026, 5, 28, 12, 0);
    let step = v(&android_alarm_step_json(doc.to_string(), now, None, vn(), false).unwrap());
    assert_eq!(step["next_at_unix_s"], unix_vn(2028, 5, 31, 7, 0));
    assert_eq!(step["next_fire_local"], "2028-05-31T07:00");
    // Không còn lời nhắc tương lai nào → null.
    let after = v(&android_alarm_step_json(
        doc.to_string(),
        unix_vn(2028, 6, 2, 0, 0),
        None,
        vn(),
        false,
    )
    .unwrap());
    assert_eq!(after["next_at_unix_s"], Value::Null);
}

/// O-9 / MA3-14-02, -03: tùy chọn lưu trong kho, mặc định không hiện tên; khi ẩn tên, `occ` không có tên.
#[test]
fn o9_lock_screen_names_from_store() {
    let (s, dir) = tmp_store();
    s.save_person_json(person_a().to_string()).unwrap();
    assert!(!s.lock_screen_names().unwrap());
    let now = unix_vn(2026, 7, 1, 0, 0);
    let rs = s.reminders_json(now, vn(), 60).unwrap();
    assert!(!rs.contains("Nguyễn Văn A"), "{rs}");
    let first = &v(&rs)[0];
    assert_eq!(first["title"], "Sắp đến ngày giỗ");
    let keys: Vec<&String> = first["occ"].as_object().unwrap().keys().collect();
    assert_eq!(keys, ["class", "date", "entity_id", "key"]);
    let step = s
        .android_alarm_step_json(unix_vn(2026, 7, 25, 7, 1), None, vn())
        .unwrap();
    assert!(!step.contains("Nguyễn Văn A"), "{step}");
    let plan = s.reschedule_json(now, vn(), "[]".into()).unwrap();
    assert!(!plan.contains("Nguyễn Văn A"));
    s.set_lock_screen_names(true).unwrap();
    assert!(s.lock_screen_names().unwrap());
    assert!(s
        .reminders_json(now, vn(), 60)
        .unwrap()
        .contains("Nguyễn Văn A"));
    // Kho mở lại vẫn nhớ tùy chọn.
    drop(s);
    let s = LvhvStore::open(dir.to_string_lossy().into_owned()).unwrap();
    assert!(s.lock_screen_names().unwrap());
    // Danh sách theo kho = danh sách trên dataset_json.
    assert_eq!(
        s.upcoming_json(sd(2026, 7, 1), 30).unwrap(),
        upcoming_json(s.dataset_json().unwrap(), sd(2026, 7, 1), 30).unwrap()
    );
    let _ = std::fs::remove_dir_all(dir);
}

/// MA3-14-04: năm ngoài 1900–3000, độ lệch múi giờ sai đơn vị, lần chuyển không tăng, giây Unix vô lý → lỗi.
#[test]
fn input_domain_checks() {
    for y in [1, 1899, 3001, 9999, -5, i32::MAX] {
        assert!(matches!(
            period_bounds("tuan".into(), sd(y, 1, 1)),
            Err(LvhvDataError::Invalid(_))
        ));
        assert!(
            events_in_period_json(EXAMPLE.into(), "nam".into(), sd(y, 1, 1), "tat_ca".into())
                .is_err()
        );
        assert!(upcoming_json(EXAMPLE.into(), sd(y, 1, 1), 30).is_err());
    }
    assert!(period_bounds("tuan".into(), sd(3000, 12, 31)).is_ok());
    let now = unix_vn(2026, 7, 1, 0, 0);
    let secs = DeviceZone {
        offset_min: 25_200,
        transitions: vec![],
    };
    assert!(matches!(
        reminders_json(EXAMPLE.into(), now, secs, 60, false),
        Err(LvhvDataError::Invalid(_))
    ));
    let unordered = DeviceZone {
        offset_min: 60,
        transitions: vec![
            ZoneTransition {
                at_unix_s: now + 1000,
                offset_min: 120,
            },
            ZoneTransition {
                at_unix_s: now,
                offset_min: 60,
            },
        ],
    };
    assert!(reschedule_json(EXAMPLE.into(), now, unordered, "[]".into(), false).is_err());
    let bad_tr = DeviceZone {
        offset_min: 60,
        transitions: vec![ZoneTransition {
            at_unix_s: now,
            offset_min: 7200,
        }],
    };
    assert!(reminders_json(EXAMPLE.into(), now, bad_tr, 60, false).is_err());
    // Mili giây thay vì giây.
    assert!(reminders_json(EXAMPLE.into(), now * 1000, vn(), 60, false).is_err());
    assert!(android_alarm_step_json(EXAMPLE.into(), now, Some(i64::MIN), vn(), false).is_err());
    assert!(reschedule_json(
        EXAMPLE.into(),
        now,
        vn(),
        r#"[{"id":"x","at_unix_s":9223372036854775807,"content":""}]"#.into(),
        false
    )
    .is_err());
    assert!(ics_export(EXAMPLE.into(), sd(2026, 7, 1), i64::MAX, None).is_err());
    // Một độ lệch hợp lệ có lần chuyển giờ mùa hè.
    let dst = DeviceZone {
        offset_min: 60,
        transitions: vec![ZoneTransition {
            at_unix_s: now + 86_400,
            offset_min: 120,
        }],
    };
    assert!(reminders_json(EXAMPLE.into(), now, dst, 60, false).is_ok());
}

// ---------------------------------------------------------------- MA3-14-07: lược đồ đầu ra

const OUT_SCHEMA: &str = include_str!("../../docs/v0.2/schema/ffi-v02-output.schema.json");

/// Kiểm tối giản theo tập con JSON Schema dùng trong `ffi-v02-output.schema.json`: `type` (một hoặc
/// mảng), `enum`, `properties` + `required` + `additionalProperties: false`, `items`, `$ref`, `oneOf`.
fn conforms(root: &Value, s: &Value, x: &Value, path: &str) -> Result<(), String> {
    if let Some(r) = s.get("$ref").and_then(Value::as_str) {
        let name = r.trim_start_matches("#/$defs/");
        return conforms(root, &root["$defs"][name], x, path);
    }
    if let Some(alts) = s.get("oneOf").and_then(Value::as_array) {
        let ok = alts
            .iter()
            .filter(|a| conforms(root, a, x, path).is_ok())
            .count();
        return if ok == 1 {
            Ok(())
        } else {
            Err(format!("{path}: oneOf khớp {ok} nhánh"))
        };
    }
    if let Some(t) = s.get("type") {
        let ty = match x {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(n) if n.is_i64() || n.is_u64() => "integer",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        };
        let allowed: Vec<&str> = match t {
            Value::String(t) => vec![t.as_str()],
            Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
            _ => vec![],
        };
        if !allowed.contains(&ty) {
            return Err(format!("{path}: kiểu {ty}, lược đồ {t}"));
        }
    }
    if let Some(e) = s.get("enum").and_then(Value::as_array) {
        if !e.contains(x) {
            return Err(format!("{path}: {x} không thuộc enum"));
        }
    }
    if let (Some(props), Value::Object(o)) = (s.get("properties"), x) {
        let props = props.as_object().unwrap();
        for k in s["required"].as_array().into_iter().flatten() {
            if !o.contains_key(k.as_str().unwrap()) {
                return Err(format!("{path}: thiếu {k}"));
            }
        }
        for (k, val) in o {
            match props.get(k) {
                Some(ps) => conforms(root, ps, val, &format!("{path}.{k}"))?,
                None if s["additionalProperties"] == false => {
                    return Err(format!("{path}: khóa lạ {k}"))
                }
                None => {}
            }
        }
    }
    if let (Some(items), Value::Array(a)) = (s.get("items"), x) {
        for (i, val) in a.iter().enumerate() {
            conforms(root, items, val, &format!("{path}[{i}]"))?;
        }
    }
    Ok(())
}

fn check_out(func: &str, text: &str) {
    let root = v(OUT_SCHEMA);
    let x = v(text);
    if let Err(e) = conforms(&root, &root["properties"][func], &x, func) {
        panic!("{e}");
    }
}

/// Mọi hàm FFI trả JSON đúng `docs/v0.2/schema/ffi-v02-output.schema.json` (khóa, kiểu, không khóa lạ).
#[test]
fn output_matches_schema() {
    let now = unix_vn(2026, 7, 1, 0, 0);
    let up = upcoming_json(R3.into(), sd(2026, 1, 1), 365).unwrap();
    assert!(v(&up).as_array().unwrap().len() > 3);
    check_out("upcoming_json", &up);
    for f in ["tat_ca", "da_mat", "dang_song"] {
        let p = events_in_period_json(
            include_str!("../../memorial/testdata/r13-linked.json").into(),
            "nam".into(),
            sd(2027, 1, 1),
            f.into(),
        )
        .unwrap();
        check_out("events_in_period_json", &p);
    }
    for names in [false, true] {
        check_out(
            "reminders_json",
            &reminders_json(R3.into(), now, vn(), 120, names).unwrap(),
        );
        check_out(
            "reschedule_json",
            &reschedule_json(R3.into(), now, vn(), "[]".into(), names).unwrap(),
        );
        check_out(
            "android_alarm_step_json",
            &android_alarm_step_json(
                EXAMPLE.into(),
                unix_vn(2026, 7, 28, 8, 0),
                Some(unix_vn(2026, 7, 24, 0, 0)),
                vn(),
                names,
            )
            .unwrap(),
        );
    }
    let ex = v(EXAMPLE);
    for p in ex["persons"].as_array().unwrap() {
        check_out(
            "person_summary_json",
            &person_summary_json(p.to_string(), sd(2026, 11, 6)).unwrap(),
        );
    }
    check_out(
        "save_death_json",
        &save_death_json(ex["persons"][0]["death"]["input"].to_string()).unwrap(),
    );
    check_out(
        "derive_birth_json",
        &derive_birth_json(ex["persons"][0]["birth"]["input"].to_string()).unwrap(),
    );
    // Lược đồ phát hiện được khóa lạ.
    let root = v(OUT_SCHEMA);
    let mut bad = v(&up);
    bad[0]["extra"] = json!(1);
    assert!(conforms(&root, &root["properties"]["upcoming_json"], &bad, "x").is_err());
}
