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

// ---------------------------------------------------------------- YC-2026-05: thông tin báo tử

const SECRET: [&str; 4] = ["ZZBAOTIN7Q", "ZZNOIMAT7Q", "ZZLYDO7Q", "ZZGHICHU7Q"];
/// Ngày nhận tin dùng làm chuỗi canh: không trùng ngày nào khác trong dữ liệu ví dụ.
const SECRET_DATE: (i32, u32, u32) = (2025, 8, 9);

fn add_cert_photo(s: &LvhvStore) -> String {
    let id = new_uuid().unwrap();
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../docs/v0.2/schema/test-vectors/photo-0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e09.jpg"
    ))
    .unwrap();
    s.add_photo(id.clone(), "image/jpeg".into(), 64, 64, bytes)
        .unwrap();
    id
}

/// Tên tệp gốc của tệp đính kèm — chỉ hiện trên trang hồ sơ, không bao giờ ra thông báo/.ics/danh sách.
const SECRET_FILE: &str = "ZZTENTEP7Q.pdf";

/// Thêm một tệp đính kèm PDF (YC-2026-07) với tên gốc bí mật; trả id.
fn add_cert_attachment(s: &LvhvStore) -> String {
    let id = new_uuid().unwrap();
    s.add_attachment(
        id.clone(),
        "application/pdf".into(),
        Some(SECRET_FILE.into()),
        b"%PDF-1.4\n%%EOF\n".to_vec(),
    )
    .unwrap();
    id
}

fn secret_notice(cert: &str) -> String {
    json!({
        "reported_by": format!("  {}  ", SECRET[0]),
        "place": SECRET[1],
        "reason": SECRET[2],
        "notice_date": {"calendar":"solar","precision":"day","year":SECRET_DATE.0,"month":SECRET_DATE.1,"day":SECRET_DATE.2,"leap":false},
        "attachments": [{"id": new_uuid().unwrap(), "kind": "giay_chung_tu", "file_id": cert}],
        "note": format!("{}\ndòng hai", SECRET[3]),
    })
    .to_string()
}

/// US-M-07 + B4: cập nhật người đang sống thành đã mất trên cùng hồ sơ; sinh nhật và sự kiện biến mất.
#[test]
fn mark_deceased_same_record_and_b4() {
    let (s, dir) = tmp_store();
    let mut c = v(EXAMPLE)["persons"][2].clone();
    c["id"] = json!("");
    let c = v(&s.save_person_json(c.to_string()).unwrap());
    let cid = c["id"].as_str().unwrap().to_owned();
    let mut e = v(EXAMPLE)["events"][0].clone();
    e["person_ids"] = json!([cid]);
    s.save_event_json(e.to_string()).unwrap();
    let up = |s: &LvhvStore| v(&s.upcoming_json(sd(2026, 1, 1), 365).unwrap());
    let before = up(&s);
    assert!(before
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["entity_id"] == cid.as_str() && x["class"] == "sinh_nhat"));
    assert!(before
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["class"] == "su_kien_duong_lap"));
    let cert = add_cert_attachment(&s);
    let p = v(&s
        .mark_deceased_json(
            cid.clone(),
            r#"{"calendar":"solar","precision":"day","year":2025,"month":8,"day":8,"leap":false}"#
                .into(),
            Some(secret_notice(&cert)),
            sd(2026, 1, 1),
        )
        .unwrap());
    assert_eq!(p["id"], cid.as_str());
    assert_eq!(p["rev"], c["rev"].as_i64().unwrap() + 1);
    assert_eq!(p["created_at"], c["created_at"]);
    assert_eq!(p["kind"], "deceased");
    assert_eq!(p["ho_ten"], c["ho_ten"]);
    assert_eq!(p["birth"], c["birth"]);
    assert_eq!(p["death"]["lunar"]["leap"], true);
    assert_eq!(p["death"]["notice"]["reported_by"], SECRET[0]); // đã cắt khoảng trắng
    assert_eq!(p["reminders"]["sinh_nhat"], Value::Null);
    assert!(p["reminders"]["gio"].is_object());
    assert_eq!(v(&s.persons_json().unwrap()).as_array().unwrap().len(), 1);
    let after = up(&s);
    let a = after.as_array().unwrap();
    assert!(
        a.iter()
            .all(|x| x["class"] == "gio" || x["class"] == "tien_thuong"),
        "{after}"
    );
    assert!(a
        .iter()
        .any(|x| x["entity_id"] == cid.as_str() && x["class"] == "gio"));
    // Ngày mất / ngày nhận tin sau hôm nay → lỗi.
    assert!(s
        .mark_deceased_json(
            cid.clone(),
            r#"{"calendar":"solar","precision":"day","year":2026,"month":5,"day":1,"leap":false}"#
                .into(),
            None,
            sd(2026, 1, 1),
        )
        .is_err());
    // Xóa hồ sơ → xóa tệp đính kèm báo tử.
    s.delete_person(cid).unwrap();
    assert!(matches!(
        s.attachment_bytes(cert),
        Err(LvhvDataError::NotFound(_))
    ));
    let _ = std::fs::remove_dir_all(dir);
}

/// R-PRIV-2: không trường nào của thông tin báo tử xuất hiện trong lời nhắc, thông báo, .ics, tên tệp, danh
/// sách hay tóm tắt hồ sơ — ở cả hai chế độ hiện/ẩn tên. Tệp sao lưu (mã hóa) không lộ bản rõ.
#[test]
fn notice_never_leaks() {
    let (s, dir) = tmp_store();
    let cert = add_cert_attachment(&s);
    let mut a = person_a();
    a["death"]["notice"] = v(&secret_notice(&cert));
    let a = v(&s.save_person_json(a.to_string()).unwrap());
    assert_eq!(a["death"]["notice"]["place"], SECRET[1]);
    let ds = s.dataset_json().unwrap();
    assert!(
        ds.contains(SECRET[2]),
        "dataset_json là tài liệu sao lưu, phải có báo tử"
    );
    let date_iso = format!(
        "{:04}-{:02}-{:02}",
        SECRET_DATE.0, SECRET_DATE.1, SECRET_DATE.2
    );
    let date_ics = date_iso.replace('-', "");
    let mut outs: Vec<(String, String)> = Vec::new();
    for names in [false, true] {
        s.set_lock_screen_names(names).unwrap();
        for now in [unix_vn(2026, 1, 1, 0, 0), unix_vn(2026, 7, 25, 7, 1)] {
            outs.push((
                "reminders".into(),
                s.reminders_json(now, vn(), 3660).unwrap(),
            ));
            outs.push((
                "reschedule".into(),
                s.reschedule_json(now, vn(), "[]".into()).unwrap(),
            ));
            outs.push((
                "android".into(),
                s.android_alarm_step_json(now, Some(now - 30 * 86_400), vn())
                    .unwrap(),
            ));
        }
    }
    outs.push((
        "upcoming".into(),
        s.upcoming_json(sd(2026, 1, 1), 3660).unwrap(),
    ));
    for k in ["tuan", "thang", "quy", "nam"] {
        for f in ["tat_ca", "da_mat", "dang_song"] {
            outs.push((
                "period".into(),
                s.events_in_period_json(k.into(), sd(2026, 7, 28), f.into())
                    .unwrap(),
            ));
        }
    }
    outs.push((
        "summary".into(),
        person_summary_json(a.to_string(), sd(2026, 11, 6)).unwrap(),
    ));
    for only in [None, Some(vec![a["id"].as_str().unwrap().to_owned()])] {
        let f = ics_export(
            ds.clone(),
            sd(2025, 1, 1),
            unix_vn(2026, 11, 6, 10, 0),
            only,
        )
        .unwrap();
        outs.push(("ics.name".into(), f.file_name));
        outs.push(("ics".into(), f.content.replace("\r\n ", "")));
    }
    let mut needles: Vec<String> = SECRET.iter().map(|x| x.to_string()).collect();
    needles.extend([
        cert.clone(),
        date_iso,
        date_ics,
        "dòng hai".into(),
        SECRET_FILE.into(),
        "ZZTENTEP7Q".into(),
    ]);
    for (what, text) in &outs {
        for n in &needles {
            assert!(!text.contains(n.as_str()), "{what} chứa {n}");
        }
    }
    // Tệp sao lưu: bản rõ không lộ trong byte tệp; khôi phục giữ nguyên báo tử và ảnh giấy báo tử (R2/R6).
    let file = s
        .export_backup("mật khẩu đúng".into(), "0.2.0".into(), "android".into())
        .unwrap();
    for n in &needles {
        assert!(
            !file.windows(n.len()).any(|w| w == n.as_bytes()),
            "tệp sao lưu lộ {n}"
        );
    }
    let (t, dir2) = tmp_store();
    t.apply_import(
        open_backup(file, "mật khẩu đúng".into()).unwrap(),
        BackupImportMode::Replace,
    )
    .unwrap();
    assert_eq!(t.persons_json().unwrap(), s.persons_json().unwrap());
    assert_eq!(
        t.attachment_bytes(cert.clone()).unwrap(),
        s.attachment_bytes(cert).unwrap()
    );
    assert!(t.dataset_json().unwrap().contains(SECRET_FILE));
    let _ = std::fs::remove_dir_all(dir);
    let _ = std::fs::remove_dir_all(dir2);
}

/// 01 §3.2a: giới hạn và kiểm tra của thông tin báo tử.
#[test]
fn notice_validation() {
    let (s, dir) = tmp_store();
    let cert = add_cert_attachment(&s);
    let base = v(&secret_notice(&cert));
    let save = |n: Value| {
        let mut a = person_a();
        a["id"] = json!(new_uuid().unwrap());
        a["death"]["notice"] = n;
        s.save_person_json(a.to_string())
    };
    assert!(save(base.clone()).is_ok());
    assert!(save(Value::Null).is_ok());
    let bad = |k: &str, val: Value| {
        let mut n = base.clone();
        n[k] = val;
        n
    };
    let long = |n: usize| json!("a".repeat(n));
    assert!(save(bad("reported_by", long(200))).is_ok());
    assert!(save(bad("reported_by", long(201))).is_err());
    assert!(save(bad("place", long(201))).is_err());
    assert!(save(bad("reason", long(500))).is_ok());
    assert!(save(bad("reason", long(501))).is_err());
    assert!(save(bad("note", long(2001))).is_err());
    assert!(save(bad("reason", json!("dòng\nhai"))).is_err()); // chỉ note được xuống dòng
    assert!(save(bad("place", json!("   "))).is_ok()); // rỗng sau khi cắt → null
    let att =
        |file: &str, kind: &str| json!({"id": new_uuid().unwrap(), "kind": kind, "file_id": file});
    assert!(save(bad(
        "attachments",
        json!([att(&new_uuid().unwrap(), "khac")])
    ))
    .is_err()); // V3
    let seven: Vec<Value> = (0..7)
        .map(|_| att(&add_cert_attachment(&s), "tin_bao_tu"))
        .collect();
    assert!(save(bad("attachments", json!(seven[..6]))).is_ok());
    assert!(save(bad("attachments", json!(seven))).is_err()); // > 6
    assert!(save(bad(
        "attachments",
        json!([att(&cert, "khac"), att(&cert, "khac")])
    ))
    .is_err());
    assert!(save(bad("attachments", json!([att(&cert, "giay")]))).is_err());
    assert!(save(bad("certificate_photo_ids", json!([]))).is_err()); // khóa cũ không còn
                                                                     // Tệp đính kèm: PDF phải bắt đầu bằng %PDF-, ảnh ≤ 2 MB, PDF ≤ 10 MB.
    let id = new_uuid().unwrap();
    assert!(s
        .add_attachment(
            id.clone(),
            "application/pdf".into(),
            None,
            b"GIF89a".to_vec()
        )
        .is_err());
    assert!(s
        .add_attachment(
            id.clone(),
            "image/jpeg".into(),
            None,
            vec![0; 2 * 1024 * 1024 + 1]
        )
        .is_err());
    let m = v(&s
        .add_attachment(
            id.clone(),
            "image/webp".into(),
            Some("ảnh.webp".into()),
            vec![1; 100],
        )
        .unwrap());
    assert_eq!(
        (m["stripped"].clone(), m["bytes"].clone()),
        (json!(true), json!(100))
    );
    s.delete_attachment(id.clone()).unwrap();
    assert!(matches!(
        s.delete_attachment(id),
        Err(LvhvDataError::NotFound(_))
    ));
    assert!(matches!(
        s.delete_attachment(cert.clone()),
        Err(LvhvDataError::Invalid(_))
    )); // còn dùng
    assert!(save(bad(
        "notice_date",
        json!({"calendar":"lunar","precision":"day","year":2025,"month":6,"day":15,"leap":false})
    ))
    .is_err());
    assert!(save(bad("extra", json!(1))).is_err());
    // Người đang sống không có `death`, nên không có báo tử.
    let _ = std::fs::remove_dir_all(dir);
}

// ---------------------------------------------------------------- core2 (duyệt mã native)

/// MA3N-23-01: tạo byte sao lưu không ghi nhận "đã sao lưu"; chỉ `mark_backup_saved` mới ghi.
#[test]
fn export_does_not_mark_backup_saved() {
    let (s, dir) = tmp_store();
    s.save_person_json(person_a().to_string()).unwrap();
    assert!(matches!(
        s.mark_backup_saved(unix_vn(2026, 7, 1, 9, 0)),
        Err(LvhvDataError::Invalid(_))
    ));
    let _file = s
        .export_backup("mật khẩu đúng".into(), "0.2.0".into(), "ios".into())
        .unwrap();
    assert_eq!(
        v(&s.settings_json().unwrap())["last_backup_at"],
        Value::Null
    );
    s.mark_backup_saved(unix_vn(2026, 7, 1, 9, 0)).unwrap();
    assert_eq!(
        v(&s.settings_json().unwrap())["last_backup_at"],
        "2026-07-01T02:00:00Z"
    );
    assert!(!s.backup_reminder_due().unwrap());
    assert!(s.mark_backup_saved(unix_vn(2026, 7, 1, 9, 0)).is_err());
    assert!(s.mark_backup_saved(i64::MAX).is_err());
    let _ = std::fs::remove_dir_all(dir);
}

/// MA3N-24-08: xóa người cùng sự kiện chỉ của người đó, một giao dịch.
#[test]
fn delete_person_and_orphan_events_works() {
    let (s, dir) = tmp_store();
    let mut c = v(EXAMPLE)["persons"][2].clone();
    c["id"] = json!("");
    let c = v(&s.save_person_json(c.to_string()).unwrap());
    let cid = c["id"].as_str().unwrap().to_owned();
    let mut d = v(EXAMPLE)["persons"][2].clone();
    d["id"] = json!("");
    d["ho_ten"] = json!("Lê Văn D");
    let d = v(&s.save_person_json(d.to_string()).unwrap());
    let did = d["id"].as_str().unwrap().to_owned();
    let mk = |pids: Value| {
        let mut e = v(EXAMPLE)["events"][0].clone();
        e["id"] = json!("");
        e["reminder"]["id"] = json!(new_uuid().unwrap());
        e["person_ids"] = pids;
        v(&s.save_event_json(e.to_string()).unwrap())["id"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let only_c = mk(json!([cid]));
    let shared = mk(json!([cid, did]));
    let nobody = mk(json!([]));
    let removed = s.delete_person_and_orphan_events(cid).unwrap();
    assert_eq!(removed, vec![only_c]);
    let ids: Vec<String> = v(&s.events_json().unwrap())
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap().to_owned())
        .collect();
    assert!(ids.contains(&shared) && ids.contains(&nobody) && ids.len() == 2);
    let _ = std::fs::remove_dir_all(dir);
}

/// 02 §7.5: thẻ nhắc hôm nay — cùng kết quả với lõi (`due_today`), tiêu đề có tên, sắp theo giờ.
#[test]
fn due_today_matches_core() {
    let r = v(&due_today_json(EXAMPLE.into(), sd(2026, 7, 25), vn()).unwrap());
    let a = r.as_array().unwrap();
    assert_eq!(a.len(), 1);
    assert_eq!(a[0]["fire_local"], "2026-07-25T07:00");
    assert_eq!(a[0]["title"], "Còn 3 ngày: Giỗ đầu Ông Nguyễn Văn A");
    assert_eq!(a[0]["occ"]["name"], "Ông Nguyễn Văn A");
    let ds = lvhv_memorial::model::parse_dataset(R3).unwrap();
    let c = lvhv_memorial::Cal::get(lvhv_core::DEFAULT_METHOD);
    for day in [sd(2026, 7, 25), sd(2026, 7, 28), sd(2027, 1, 1)] {
        let got = v(&due_today_json(R3.into(), day, vn()).unwrap());
        let want = lvhv_memorial::remind::due_today(
            c,
            &ds,
            lvhv_memorial::model::Ymd::new(day.y, day.m as u32, day.d as u32),
            &lvhv_memorial::remind::Zone::vietnam(),
        );
        let ids: Vec<&str> = got
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, want.iter().map(|r| r.id.as_str()).collect::<Vec<_>>());
        check_out("due_today_json", &got.to_string());
    }
}

#[test]
fn ics_summary_and_export_preview() {
    let s = ics_summary(EXAMPLE.into(), sd(2026, 7, 1), None).unwrap();
    let f = ics_export(
        EXAMPLE.into(),
        sd(2026, 7, 1),
        unix_vn(2026, 7, 1, 0, 0),
        None,
    )
    .unwrap();
    assert_eq!(f.summary, s);
    assert_eq!((s.events, s.persons), (f.events, f.persons));
    assert_eq!((s.from, s.to), (sd(2026, 7, 1), sd(2036, 6, 30)));
    assert_eq!(s.preview.len(), 5);
    assert!(
        s.preview[0].starts_with("28/07/2026 — Giỗ đầu Ông Nguyễn Văn A"),
        "{:?}",
        s.preview
    );
    assert!(s.privacy_text.starts_with("Tệp này chứa tên, ngày giỗ"));
    assert!(s.privacy_text.contains(&format!("của {} người", s.persons)));
    let none = ics_summary(EXAMPLE.into(), sd(2026, 7, 1), Some(vec![])).unwrap();
    assert_eq!((none.events, none.privacy_text.as_str()), (0, ""));
}

#[test]
fn password_rules_leap_month_header_check() {
    // Cùng ca với tsport/test/memorial_unit.mjs (backupPasswordError).
    let nfd = "Ma\u{0323}\u{0302}t kha\u{0302}\u{0309}u"; // "Mật khẩu" dạng NFD, 8 điểm mã sau NFC
    let cases: [(&str, Option<&str>, Option<&str>); 6] = [
        ("1234567", None, Some("Mật khẩu cần ít nhất 8 ký tự")),
        ("12345678", None, None),
        ("😀😀😀😀", None, Some("Mật khẩu cần ít nhất 8 ký tự")),
        (nfd, Some("Mật khẩu"), None),
        (
            "mật khẩu đúng",
            Some("mật khẩu sai"),
            Some("Hai lần nhập mật khẩu chưa giống nhau"),
        ),
        ("mật khẩu đúng", Some("mật khẩu đúng"), None),
    ];
    for (pw, c, want) in cases {
        assert_eq!(
            backup_password_error(pw.into(), c.map(str::to_owned)).as_deref(),
            want,
            "{pw}"
        );
    }
    assert_eq!(lunar_leap_month(2025).unwrap(), Some(6));
    assert_eq!(lunar_leap_month(2026).unwrap(), None);
    assert_eq!(lunar_leap_month(2028).unwrap(), Some(5));
    assert!(lunar_leap_month(3001).is_err());
    let (s, dir) = tmp_store();
    let f = s
        .export_backup("mật khẩu đúng".into(), "0.2.0".into(), "ios".into())
        .unwrap();
    assert!(backup_header_check(f[..72].to_vec(), f.len() as u64).is_ok());
    assert!(backup_header_check(f[..72].to_vec(), f.len() as u64 + 1).is_err());
    assert!(backup_header_check(f[..40].to_vec(), f.len() as u64).is_err());
    assert!(backup_header_check(vec![0u8; 72], 4 << 30).is_err());
    let _ = std::fs::remove_dir_all(dir);
}

/// Duyệt thiết kế TKN-A-08: tiêu đề sự kiện kèm tên người — trong .ics và thông báo khi bật hiện tên; ẩn tên
/// (O-9, mặc định) vẫn là câu chung không tên.
#[test]
fn event_titles_name_the_people() {
    let want = "Kỷ niệm 11 năm ngày cưới: Ngày cưới anh chị C · Chị Lê Thị C";
    let f = ics_export(
        EXAMPLE.into(),
        sd(2026, 7, 1),
        unix_vn(2026, 7, 1, 0, 0),
        None,
    )
    .unwrap();
    let unfolded = f.content.replace("\r\n ", "");
    assert!(
        unfolded.contains(&format!("SUMMARY:{want}\r\n")),
        "{unfolded}"
    );
    let now = unix_vn(2026, 12, 10, 0, 0);
    let ev = |show: bool| -> Vec<Value> {
        v(&reminders_json(EXAMPLE.into(), now, vn(), 60, show).unwrap())
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| {
                r["id"]
                    .as_str()
                    .unwrap()
                    .contains("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e22")
            })
            .cloned()
            .collect()
    };
    let named = ev(true);
    assert_eq!(named[0]["title"], format!("Còn 1 ngày: {want}"));
    let hidden = ev(false);
    assert_eq!(hidden.len(), named.len());
    assert!(hidden
        .iter()
        .all(|r| r["title"] == "Sắp đến sự kiện" && !r.to_string().contains("Lê Thị C")));
    // "Sắp tới": headline có tên (như .ics).
    let up = v(&upcoming_json(EXAMPLE.into(), sd(2026, 12, 1), 30).unwrap());
    let w = up
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["class"] == "su_kien_duong_lap")
        .unwrap();
    assert_eq!(w["headline"], want);
    assert_eq!(
        w["headline_plain"],
        "Kỷ niệm 11 năm ngày cưới: Ngày cưới anh chị C"
    );
    // Sự kiện chưa gắn ai → không có " · ".
    let mut ds = v(EXAMPLE);
    ds["events"][0]["person_ids"] = json!([]);
    let f = ics_export(
        ds.to_string(),
        sd(2026, 7, 1),
        unix_vn(2026, 7, 1, 0, 0),
        None,
    )
    .unwrap();
    assert!(f
        .content
        .replace("\r\n ", "")
        .contains("SUMMARY:Kỷ niệm 11 năm ngày cưới: Ngày cưới anh chị C\r\n"));
}

/// Ảnh không còn dùng (yêu cầu từ ứng dụng Apple): đổi/bỏ ảnh trong `save_person_json` xóa ảnh cũ nếu không hồ sơ
/// nào khác dùng; `delete_photo` xóa ảnh chưa gắn hồ sơ nào.
#[test]
fn orphan_photos_are_removed() {
    let (s, dir) = tmp_store();
    let photo_ids = |s: &LvhvStore| -> Vec<String> {
        let mut v: Vec<String> = v(&s.dataset_json().unwrap())["photos"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["id"].as_str().unwrap().to_owned())
            .collect();
        v.sort();
        v
    };
    let a = add_cert_photo(&s);
    let mut c = v(EXAMPLE)["persons"][2].clone();
    c["id"] = json!("");
    c["photo_id"] = json!(a);
    let c = v(&s.save_person_json(c.to_string()).unwrap());
    // Hồ sơ thứ hai dùng chung ảnh A.
    let mut d = v(EXAMPLE)["persons"][2].clone();
    d["id"] = json!("");
    d["ho_ten"] = json!("Lê Văn D");
    d["photo_id"] = json!(a);
    let d = v(&s.save_person_json(d.to_string()).unwrap());
    // C đổi sang ảnh B: A còn D dùng → giữ.
    let b = add_cert_photo(&s);
    let mut c2 = c.clone();
    c2["photo_id"] = json!(b);
    let c2 = v(&s.save_person_json(c2.to_string()).unwrap());
    let mut want = vec![a.clone(), b.clone()];
    want.sort();
    assert_eq!(photo_ids(&s), want);
    // D bỏ ảnh: A không còn ai dùng → xóa (bản ghi và tệp).
    let mut d2 = d.clone();
    d2["photo_id"] = Value::Null;
    s.save_person_json(d2.to_string()).unwrap();
    assert_eq!(photo_ids(&s), vec![b.clone()]);
    assert!(matches!(
        s.photo_bytes(a.clone()),
        Err(LvhvDataError::NotFound(_))
    ));
    assert!(!dir.join("photos").join(format!("{a}.img")).exists());
    assert!(dir.join("photos").join(format!("{b}.img")).exists());
    // Lưu lại không đổi ảnh → giữ ảnh.
    s.save_person_json(c2.to_string()).unwrap();
    assert_eq!(photo_ids(&s), vec![b.clone()]);
    // delete_photo: ảnh đang dùng → Invalid; ảnh chưa gắn → xóa; không có → NotFound.
    assert!(matches!(
        s.delete_photo(b.clone()),
        Err(LvhvDataError::Invalid(_))
    ));
    let x = add_cert_photo(&s);
    s.delete_photo(x.clone()).unwrap();
    assert_eq!(photo_ids(&s), vec![b.clone()]);
    assert!(matches!(s.delete_photo(x), Err(LvhvDataError::NotFound(_))));
    let _ = std::fs::remove_dir_all(dir);
}

/// YC-2026-07: viết hoa chữ đầu cách gọi / quan hệ khi lưu, giới tính, gợi ý dùng chung.
#[test]
fn names_gender_and_suggestions() {
    let (s, dir) = tmp_store();
    let mut a = person_a();
    a["danh_xung"] = json!("ông");
    a["quan_he"] = json!("o\u{0302}ng nội bên bố"); // NFD
    a["gioi_tinh"] = json!("nam");
    let a = v(&s.save_person_json(a.to_string()).unwrap());
    assert_eq!(a["danh_xung"], "Ông");
    assert_eq!(a["quan_he"], "Ông nội bên bố");
    assert_eq!(a["gioi_tinh"], "nam");
    let mut b = a.clone();
    b["gioi_tinh"] = Value::Null;
    b["danh_xung"] = json!("bà Cô Tổ họ");
    let b = v(&s.save_person_json(b.to_string()).unwrap());
    assert!(b.get("gioi_tinh").is_none(), "null ≡ thiếu khóa");
    assert_eq!(b["danh_xung"], "Bà Cô Tổ họ");
    let mut c = b.clone();
    c["gioi_tinh"] = json!("khac");
    assert!(s.save_person_json(c.to_string()).is_err());
    assert_eq!(capitalize_first("bé đỏ".into()), "Bé đỏ");
    let dead = danh_xung_suggestions(true, None);
    let living = danh_xung_suggestions(false, None);
    assert_eq!((dead.len(), living.len()), (22, 17));
    for x in ["Cụ", "Kỵ", "Ông Tổ họ", "Bà Cô Tổ họ", "Bé đỏ"] {
        assert!(
            dead.iter().any(|d| d == x) && !living.iter().any(|d| d == x),
            "{x}"
        );
    }
    assert_eq!(danh_xung_suggestions(false, Some("nam".into()))[0], "Ông");
    assert_eq!(danh_xung_suggestions(false, Some("nu".into()))[0], "Bà");
    assert_eq!(danh_xung_suggestions(true, Some("x".into())), dead);
    assert!(quan_he_suggestions(true).contains(&"Kỵ ông".to_string()));
    assert!(!quan_he_suggestions(false).contains(&"Kỵ ông".to_string()));
    let _ = std::fs::remove_dir_all(dir);
}
