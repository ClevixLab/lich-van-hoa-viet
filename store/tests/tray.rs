//! YC-2026-10 (04 Bổ sung B8, R15 (c)–(f)): `memorial.tray_photo_id` — ảnh mâm lễ của gia đình cho một người.
//! Bản PWA kiểm cùng các ca ở `pwa/test/store/run.mjs`; kịch bản đầy đủ `lvhv-store-tool scenario tray_photo` (hai bản,
//! tệp kỳ vọng `test-vectors/tray-photo/expected-scenario.json`) chạy trong `scripts/check-store.sh`.
use lvhv_store::model::MemorialSettings;
use lvhv_store::{
    canonical_json, merge_docs, open_backup, sha256_hex, validate, AppInfo, ImportMode, Store,
    StoreError,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const PW: &str = "mật khẩu thử nghiệm";
const D1: &str = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01";
const D2: &str = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02";
const L3: &str = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e03";
const T1: &str = "0b8e4f52-3c1a-4d2e-9f60-0000000000c1";
const T2: &str = "0b8e4f52-3c1a-4d2e-9f60-0000000000c2";

fn tv() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/v0.2/schema/test-vectors")
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("lvhv-tray-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// Kho có dữ liệu mẫu `example-v1.lvhv` (tệp sao lưu của bản trước, không có trường mới).
fn example_store(dir: &Path) -> Store {
    let mut s = Store::open(dir).unwrap();
    let v = open_backup(&std::fs::read(tv().join("example-v1.lvhv")).unwrap(), PW).unwrap();
    s.apply_import(&v, ImportMode::Replace).unwrap();
    s
}

fn add(s: &mut Store, id: &str, n: u32) {
    let b = std::fs::read(tv().join(format!("r6v2/photo-{n:02}.jpg"))).unwrap();
    s.add_photo(id, "image/jpeg", 300 + n, 400 + n, &b).unwrap();
}

fn bump(p: &mut Value) {
    p["rev"] = json!(p["rev"].as_i64().unwrap() + 1);
    p["updated_at"] = json!("2026-11-07T00:00:00Z");
}

fn with_tray(s: &mut Store, id: &str, tray: Value) -> lvhv_store::Result<()> {
    let mut p = s.person(id).unwrap();
    p["memorial"]["tray_photo_id"] = tray;
    bump(&mut p);
    lvhv_store::names::normalize_person_names(&mut p);
    s.put_person(&p)
}

fn v3_path(r: lvhv_store::Result<()>) -> String {
    match r {
        Err(StoreError::Invalid(v)) => format!("{} {}", v.code, v.path),
        other => format!("{other:?}"),
    }
}

/// Ảnh mồ côi: bản ghi không hồ sơ nào dùng + tệp `photos/*.img` không có bản ghi (và ngược lại).
fn orphans(s: &Store, dir: &Path) -> usize {
    let ids: std::collections::BTreeSet<String> = s
        .photos()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap().to_owned())
        .collect();
    let used: std::collections::BTreeSet<String> = s
        .persons()
        .unwrap()
        .iter()
        .flat_map(validate::person_photo_refs)
        .collect();
    let files: std::collections::BTreeSet<String> = std::fs::read_dir(dir.join("photos"))
        .unwrap()
        .flatten()
        .filter_map(|e| {
            e.file_name()
                .to_string_lossy()
                .strip_suffix(".img")
                .map(str::to_owned)
        })
        .collect();
    ids.difference(&used).count() + ids.symmetric_difference(&files).count()
}

/// R15 (e): kiểu dữ liệu — thiếu ≡ None, None không ghi khóa (tài liệu cũ giống từng byte).
#[test]
fn memorial_settings_field_optional_and_omitted_when_none() {
    let old = json!({"tray_style": "truyen_thong", "incense": true, "bia_tieu_de": "", "loi_tuong_niem": "",
                     "gio_het_nam": 2, "huong_duong_duoi": 60});
    let m: MemorialSettings = serde_json::from_value(old.clone()).unwrap();
    assert_eq!(m.tray_photo_id, None);
    assert_eq!(serde_json::to_value(&m).unwrap(), old);
    let mut with = old.clone();
    with["tray_photo_id"] = json!(T1);
    let m: MemorialSettings = serde_json::from_value(with.clone()).unwrap();
    assert_eq!(m.tray_photo_id.as_deref(), Some(T1));
    assert_eq!(serde_json::to_value(&m).unwrap(), with);
    // Lược đồ: UUID v4 hoặc null; kiểu khác → V1.
    let mut bad = old.clone();
    bad["tray_photo_id"] = json!("abc");
    let mut p: Value =
        serde_json::from_slice(&std::fs::read(tv().join("example-v1.json")).unwrap()).unwrap();
    p["persons"][0]["memorial"] = bad;
    let e = validate::person(&p["persons"][0], "/person").unwrap_err();
    assert_eq!(
        (e.code, e.path.as_str()),
        ("V1", "/person/memorial/tray_photo_id")
    );
}

/// R15 (e), R14: tệp sao lưu của bản trước mở được, không đổi gì; tệp xuất lại không có khóa mới.
#[test]
fn old_backup_opens_unchanged() {
    let dir = tmp("old");
    let mut s = example_store(&dir);
    let want: Value =
        serde_json::from_slice(&std::fs::read(tv().join("example-v1.json")).unwrap()).unwrap();
    let got = s
        .snapshot(
            &AppInfo {
                version: "0.2.0".into(),
                platform: "pwa".into(),
            },
            "x",
        )
        .unwrap();
    for k in ["persons", "photos", "events", "event_types"] {
        assert_eq!(canonical_json(&got[k]), canonical_json(&want[k]), "{k}");
    }
    let f = s
        .export_backup(
            PW,
            &AppInfo {
                version: "0.2.0".into(),
                platform: "android".into(),
            },
        )
        .unwrap();
    let doc = open_backup(&f, PW).unwrap().document().clone();
    assert!(!canonical_json(&doc).contains("tray_photo_id"));
    let _ = std::fs::remove_dir_all(dir);
}

/// R15 (d), (f): gắn ảnh, xuất / nhập (Thay toàn bộ, Gộp) giữ ảnh và liên kết; V3; đổi, bỏ, đổi về đang sống, xóa hồ
/// sơ → ảnh cũ bị xóa (bản ghi và tệp) trong cùng giao dịch; 0 ảnh mồ côi.
#[test]
fn tray_photo_lifecycle() {
    let dir = tmp("life");
    let mut s = example_store(&dir);
    add(&mut s, T1, 0);
    with_tray(&mut s, D1, json!(T1)).unwrap();
    assert_eq!(s.person(D1).unwrap()["memorial"]["tray_photo_id"], T1);
    assert_eq!(orphans(&s, &dir), 0);
    // V3: không tồn tại; dùng chung với hồ sơ khác (ảnh mâm lễ / ảnh chân dung); trùng ảnh chân dung của chính mình.
    let portrait = s.person(D1).unwrap()["photo_id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        v3_path(with_tray(
            &mut s,
            D2,
            json!("0b8e4f52-3c1a-4d2e-9f60-0000000000ee")
        )),
        "V3 /person/memorial/tray_photo_id"
    );
    assert_eq!(
        v3_path(with_tray(&mut s, D2, json!(T1))),
        "V3 /person/memorial/tray_photo_id"
    );
    assert_eq!(
        v3_path(with_tray(&mut s, D2, json!(portrait))),
        "V3 /person/memorial/tray_photo_id"
    );
    assert_eq!(
        v3_path(with_tray(&mut s, D1, json!(portrait))),
        "V3 /person/memorial/tray_photo_id"
    );
    let mut l = s.person(L3).unwrap();
    l["photo_id"] = json!(T1);
    bump(&mut l);
    assert_eq!(v3_path(s.put_person(&l)), "V3 /person/photo_id");
    // (d) xuất rồi nhập.
    let sha = sha256_hex(&s.photo_bytes(T1).unwrap());
    let f = s
        .export_backup(
            PW,
            &AppInfo {
                version: "0.2.0".into(),
                platform: "android".into(),
            },
        )
        .unwrap();
    for (k, mode, seed) in [
        ("replace", ImportMode::Replace, false),
        ("merge", ImportMode::Merge, false),
        ("merge-old", ImportMode::Merge, true),
    ] {
        let d = tmp(k);
        let mut u = if seed {
            example_store(&d)
        } else {
            Store::open(&d).unwrap()
        };
        u.apply_import(&open_backup(&f, PW).unwrap(), mode).unwrap();
        assert_eq!(
            u.person(D1).unwrap()["memorial"]["tray_photo_id"],
            T1,
            "{k}"
        );
        assert_eq!(sha256_hex(&u.photo_bytes(T1).unwrap()), sha, "{k}");
        assert_eq!(orphans(&u, &d), 0, "{k}");
        drop(u);
        let _ = std::fs::remove_dir_all(d);
    }
    // (f) đổi ảnh → ảnh cũ bị xóa.
    add(&mut s, T2, 2);
    with_tray(&mut s, D1, json!(T2)).unwrap();
    assert!(s.photo_bytes(T1).is_err() && !dir.join(format!("photos/{T1}.img")).exists());
    assert_eq!(orphans(&s, &dir), 0);
    // Bỏ ảnh (null) → ảnh bị xóa, khóa không còn trong tài liệu.
    with_tray(&mut s, D1, Value::Null).unwrap();
    assert!(s.person(D1).unwrap()["memorial"]
        .get("tray_photo_id")
        .is_none());
    assert!(!dir.join(format!("photos/{T2}.img")).exists());
    assert_eq!(orphans(&s, &dir), 0);
    // Đổi về đang sống → `memorial` mất → ảnh mâm lễ bị xóa.
    add(&mut s, T1, 0);
    with_tray(&mut s, D1, json!(T1)).unwrap();
    let mut p = s.person(D1).unwrap();
    for (k, v) in [
        ("kind", json!("living")),
        ("death", Value::Null),
        ("memorial", Value::Null),
        ("noi_an_tang", json!("")),
    ] {
        p[k] = v;
    }
    p["reminders"]["gio"] = Value::Null;
    bump(&mut p);
    s.put_person(&p).unwrap();
    assert!(!dir.join(format!("photos/{T1}.img")).exists());
    assert_eq!(orphans(&s, &dir), 0);
    // Xóa hồ sơ → ảnh mâm lễ bị xóa.
    add(&mut s, T2, 2);
    with_tray(&mut s, D2, json!(T2)).unwrap();
    s.delete_person(D2).unwrap();
    assert!(!dir.join(format!("photos/{T2}.img")).exists());
    assert_eq!(orphans(&s, &dir), 0);
    let _ = std::fs::remove_dir_all(dir);
}

/// R15 (d): tài liệu nhập có `tray_photo_id` trỏ ảnh không có, hoặc dùng chung → V3, từ chối toàn bộ.
#[test]
fn import_rejects_dangling_or_shared_tray() {
    let mut doc: Value =
        serde_json::from_slice(&std::fs::read(tv().join("example-v1.json")).unwrap()).unwrap();
    let portrait = doc["persons"][0]["photo_id"].clone();
    doc["persons"][1]["memorial"]["tray_photo_id"] = json!(T1);
    let e = validate::document(&doc).unwrap_err();
    assert_eq!(
        (e.code, e.path.as_str()),
        ("V3", "/persons/1/memorial/tray_photo_id")
    );
    doc["persons"][1]["memorial"]["tray_photo_id"] = portrait;
    let e = validate::document(&doc).unwrap_err();
    assert_eq!((e.code, e.path.as_str()), ("V3", "/persons/0/photo_id"));
}

/// 01 §6.3: Gộp giữ ảnh mâm lễ mà hồ sơ (của bản nhập) sau gộp còn dùng.
#[test]
fn merge_keeps_incoming_tray_photo() {
    let local: Value =
        serde_json::from_slice(&std::fs::read(tv().join("example-v1.json")).unwrap()).unwrap();
    let mut incoming = local.clone();
    let photo = json!({"id": T1, "mime": "image/jpeg", "width": 300, "height": 400, "bytes": 10,
                       "sha256": "a".repeat(64), "stripped": true, "created_at": "2026-11-07T00:00:00Z"});
    incoming["photos"].as_array_mut().unwrap().push(photo);
    incoming["persons"][1]["memorial"]["tray_photo_id"] = json!(T1);
    incoming["persons"][1]["rev"] = json!(9);
    let m = merge_docs(&local, &incoming);
    assert!(m["photos"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["id"] == T1));
    assert_eq!(m["persons"][1]["memorial"]["tray_photo_id"], T1);
    // Hồ sơ trên máy thắng (rev lớn hơn) và không dùng ảnh → ảnh nhập không được thêm.
    let mut local2 = local.clone();
    local2["persons"][1]["rev"] = json!(99);
    let m = merge_docs(&local2, &incoming);
    assert!(!m["photos"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["id"] == T1));
}
