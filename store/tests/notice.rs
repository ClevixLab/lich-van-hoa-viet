//! YC-2026-05 (01 §3.2a): thông tin báo tử — vector bổ sung tất định, khứ hồi R2/R6 có báo tử, xóa ảnh giấy
//! báo tử theo hồ sơ, kiểm tra §6.3 khi nhập.
use lvhv_store::{canonical_json, open_backup, validate, ImportMode, Store};
use serde_json::{json, Value};

#[path = "../examples/gen_notice_vector.rs"]
#[allow(dead_code)]
mod gen;

fn tmpdir(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("lvhv-notice-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// Vector bổ sung sinh lại đúng từng byte; vector đã khóa không đổi (tệp riêng).
#[test]
fn supplement_vector_deterministic_and_committed() {
    let (doc, photos) = gen::notice_doc();
    let text = canonical_json(&doc);
    let file = gen::notice_file(&doc, &photos);
    let d = gen::out_dir();
    assert_eq!(
        std::fs::read_to_string(d.join("notice-v1.json")).unwrap(),
        text
    );
    assert_eq!(std::fs::read(d.join("notice-v1.lvhv")).unwrap(), file);
    let state = gen::state_after_import(&file);
    assert_eq!(
        std::fs::read_to_string(d.join("manifest.json")).unwrap(),
        gen::manifest(&text, &file, &state)
    );
}

/// R2/R6: khứ hồi có báo tử và ảnh giấy báo tử — giống hệt; xóa hồ sơ xóa ảnh giấy báo tử.
#[test]
fn roundtrip_with_notice_and_cascade() {
    let d = gen::out_dir();
    let file = std::fs::read(d.join("notice-v1.lvhv")).unwrap();
    let v = open_backup(&file, gen::PASSWORD).unwrap();
    assert_eq!(v.preview().photos, 2);
    let dir = tmpdir("rt");
    let mut s = Store::open(&dir).unwrap();
    s.apply_import(&v, ImportMode::Replace).unwrap();
    drop(s);
    let mut s = Store::open(&dir).unwrap();
    let p = s.person("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01").unwrap();
    assert_eq!(p["death"]["notice"]["place"], "Bệnh viện đa khoa tỉnh");
    assert_eq!(
        p["death"]["notice"]["certificate_photo_ids"],
        json!([gen::CERT_ID])
    );
    // Xuất lại → nhập vào kho trống → giống hệt.
    let h = s.state_hash().unwrap();
    let f2 = s
        .export_backup(
            gen::PASSWORD,
            &lvhv_store::AppInfo {
                version: "0.2.0".into(),
                platform: "android".into(),
            },
        )
        .unwrap();
    let dir2 = tmpdir("rt2");
    let mut t = Store::open(&dir2).unwrap();
    t.apply_import(
        &open_backup(&f2, gen::PASSWORD).unwrap(),
        ImportMode::Replace,
    )
    .unwrap();
    let mut st = t.settings().unwrap();
    st["last_backup_at"] = Value::Null;
    t.put_settings(&st).unwrap();
    assert_eq!(t.state_hash().unwrap(), h);
    // Xóa hồ sơ → ảnh giấy báo tử bị xóa; ảnh chân dung không còn ai dùng cũng bị xóa.
    s.delete_person("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01")
        .unwrap();
    assert!(s.photo_bytes(gen::CERT_ID).is_err());
    assert!(s.photos().unwrap().iter().all(|p| p["id"] != gen::CERT_ID));
    let _ = std::fs::remove_dir_all(dir);
    let _ = std::fs::remove_dir_all(dir2);
}

/// §6.3: tệp có báo tử trỏ tới ảnh không tồn tại, hoặc trường sai, bị từ chối khi kiểm tra tài liệu.
#[test]
fn import_checks_notice() {
    let (doc, _) = gen::notice_doc();
    assert!(validate::document(&doc).is_ok());
    let mut bad = doc.clone();
    bad["photos"].as_array_mut().unwrap().pop();
    let e = validate::document(&bad).unwrap_err();
    assert_eq!(e.code, "V3");
    assert_eq!(e.path, "/persons/0/death/notice/certificate_photo_ids/0");
    for (k, val) in [
        ("reason", json!("a".repeat(501))),
        ("place", json!("")),
        ("reported_by", json!(" có khoảng trắng")),
        ("note", json!(1)),
        ("certificate_photo_ids", json!([gen::CERT_ID, gen::CERT_ID])),
    ] {
        let mut b = doc.clone();
        b["persons"][0]["death"]["notice"][k] = val;
        assert!(validate::document(&b).is_err(), "{k}");
    }
    let mut b = doc.clone();
    b["persons"][0]["death"]["notice"]
        .as_object_mut()
        .unwrap()
        .remove("note");
    assert!(validate::document(&b).is_err(), "thiếu khóa");
    // Thiếu khóa `notice` (tài liệu cũ) và `notice: null` đều hợp lệ.
    let mut b = doc.clone();
    b["persons"][0]["death"]
        .as_object_mut()
        .unwrap()
        .remove("notice");
    assert!(validate::document(&b).is_ok());
    b["persons"][0]["death"]["notice"] = Value::Null;
    assert!(validate::document(&b).is_ok());
}
