//! YC-2026-05 / YC-2026-07 (01 §3.2a, §3.8a): thông tin báo tử và tệp đính kèm — vector bổ sung tất định, khứ
//! hồi R2/R6 có báo tử, xóa tệp đính kèm theo hồ sơ và khi gỡ khỏi danh sách, kiểm tra §6.3 khi nhập.
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
    assert_eq!(
        std::fs::read(d.join("giay-chung-tu.pdf")).unwrap(),
        gen::PDF_BYTES
    );
    let state = gen::state_after_import(&file);
    assert_eq!(
        std::fs::read_to_string(d.join("manifest.json")).unwrap(),
        gen::manifest(&text, &file, &state)
    );
}

/// R2/R6: khứ hồi có báo tử và tệp đính kèm — giống hệt; gỡ tệp khỏi danh sách hoặc xóa hồ sơ thì xóa tệp.
#[test]
fn roundtrip_with_notice_and_cascade() {
    let d = gen::out_dir();
    let file = std::fs::read(d.join("notice-v1.lvhv")).unwrap();
    let v = open_backup(&file, gen::PASSWORD).unwrap();
    assert_eq!(v.preview().photos, 1);
    let dir = tmpdir("rt");
    let mut s = Store::open(&dir).unwrap();
    s.apply_import(&v, ImportMode::Replace).unwrap();
    drop(s);
    let mut s = Store::open(&dir).unwrap();
    let p = s.person("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01").unwrap();
    assert_eq!(p["death"]["notice"]["place"], "Bệnh viện đa khoa tỉnh");
    assert_eq!(p["gioi_tinh"], "nam");
    assert_eq!(
        p["death"]["notice"]["attachments"][1]["file_id"],
        gen::PDF_ID
    );
    assert_eq!(s.attachment_bytes(gen::PDF_ID).unwrap(), gen::PDF_BYTES);
    assert_eq!(s.attachments().unwrap().len(), 2);
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
    // Gỡ PDF khỏi danh sách → tệp PDF bị xóa (bản ghi và tệp trên máy), ảnh tin báo tử còn.
    let mut p2 = p.clone();
    p2["death"]["notice"]["attachments"]
        .as_array_mut()
        .unwrap()
        .pop();
    p2["rev"] = json!(p["rev"].as_i64().unwrap() + 1);
    s.put_person(&p2).unwrap();
    assert!(matches!(
        s.attachment_bytes(gen::PDF_ID),
        Err(lvhv_store::StoreError::NotFound)
    ));
    assert!(!dir
        .join("attachments")
        .join(format!("{}.bin", gen::PDF_ID))
        .exists());
    assert!(s.attachment_bytes(gen::CERT_ID).is_ok());
    // Tệp đang dùng không xóa được bằng delete_attachment; tệp chưa gắn thì xóa được.
    assert!(matches!(
        s.delete_attachment(gen::CERT_ID),
        Err(lvhv_store::StoreError::Input(_))
    ));
    let loose = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e1f";
    let m = s
        .add_attachment(
            loose,
            "application/pdf",
            Some("  Bản quét.pdf "),
            gen::PDF_BYTES,
        )
        .unwrap();
    assert_eq!(
        (m["stripped"].clone(), m["file_name"].clone()),
        (json!(false), json!("Bản quét.pdf"))
    );
    s.delete_attachment(loose).unwrap();
    assert!(s.attachment_bytes(loose).is_err());
    // Thêm tệp sai: PDF thiếu %PDF-, ảnh quá 2 MiB, PDF quá 10 MiB.
    assert!(s
        .add_attachment(loose, "application/pdf", None, b"<html>")
        .is_err());
    assert!(s
        .add_attachment(loose, "image/jpeg", None, &vec![1u8; 2 * 1024 * 1024 + 1])
        .is_err());
    let mut big = gen::PDF_BYTES.to_vec();
    big.resize(10 * 1024 * 1024 + 1, b' ');
    assert!(s
        .add_attachment(loose, "application/pdf", None, &big)
        .is_err());
    assert!(s.attachments().unwrap().iter().all(|a| a["id"] != loose));
    // Xóa hồ sơ → tệp đính kèm còn lại bị xóa.
    s.delete_person("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01")
        .unwrap();
    assert!(s.attachment_bytes(gen::CERT_ID).is_err());
    assert!(s.attachments().unwrap().is_empty());
    let _ = std::fs::remove_dir_all(dir);
    let _ = std::fs::remove_dir_all(dir2);
}

/// §6.3: tệp có báo tử trỏ tới tệp đính kèm không tồn tại, hoặc trường sai, bị từ chối khi kiểm tra tài liệu.
#[test]
fn import_checks_notice() {
    let (doc, _) = gen::notice_doc();
    assert!(validate::document(&doc).is_ok());
    let mut bad = doc.clone();
    bad["attachments"].as_array_mut().unwrap().remove(0);
    let e = validate::document(&bad).unwrap_err();
    assert_eq!(e.code, "V3");
    assert_eq!(e.path, "/persons/0/death/notice/attachments/0/file_id");
    // Bản ghi Attachment sai.
    for (k, val) in [
        ("stripped", json!(true)),
        ("mime", json!("application/zip")),
        ("bytes", json!(10 * 1024 * 1024 + 1)),
        ("file_name", json!("")),
        ("file_name", json!("a\nb")),
    ] {
        let mut b = doc.clone();
        b["attachments"][1][k] = val;
        assert!(validate::document(&b).is_err(), "attachment {k}");
    }
    let mut b = doc.clone();
    b["attachments"][0]["bytes"] = json!(2 * 1024 * 1024 + 1);
    assert!(validate::document(&b).is_err(), "ảnh đính kèm > 2 MiB");
    // Giới tính.
    let mut b = doc.clone();
    b["persons"][0]["gioi_tinh"] = json!("khac");
    assert!(validate::document(&b).is_err(), "gioi_tinh lạ");
    b["persons"][0]["gioi_tinh"] = Value::Null;
    assert!(validate::document(&b).is_ok(), "gioi_tinh null");
    b["persons"][0].as_object_mut().unwrap().remove("gioi_tinh");
    assert!(validate::document(&b).is_ok(), "thiếu gioi_tinh");
    for (k, val) in [
        ("reason", json!("a".repeat(501))),
        ("place", json!("")),
        ("reported_by", json!(" có khoảng trắng")),
        ("note", json!(1)),
        (
            "attachments",
            json!([
                {"id": "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e0e", "kind": "tin_bao_tu", "file_id": gen::CERT_ID},
                {"id": "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e0f", "kind": "khac", "file_id": gen::CERT_ID}
            ]),
        ),
        (
            "attachments",
            json!([{"id": "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e0e", "kind": "giay", "file_id": gen::CERT_ID}]),
        ),
        (
            "attachments",
            json!([{"id": "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e0e", "kind": "khac"}]),
        ),
        ("attachments", json!(vec![json!({}); 7])),
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
