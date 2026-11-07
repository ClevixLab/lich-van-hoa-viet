//! 04 Bổ sung B5 / R14 — nâng cấp không mất dữ liệu: bộ mẫu đóng băng `test-vectors/released/` (README ở đó).
//!
//! `0.2-dev-before-attachments`: kho SQLite **thật** do bản dựng 0.2 trước YC-2026-07 tạo (commit 7338f04,
//! `make_dev_fixture.rs.txt`), có `death.notice.certificate_photo_ids`. Mở bằng bản này → chuyển 1 → 2:
//! - 0 bản ghi mất, 0 trường đổi ngoài phần chuyển đổi đã định (01 §7, migrate::migrate_1_to_2);
//! - kết quả = `released/expected/0.2-dev-before-attachments.json` — cùng tệp mà bản PWA (TS) phải ra;
//! - tệp sao lưu của bản dựng đó nhập được và cho cùng trạng thái;
//! - ca lỗi: quay lui, chỉ đọc, dữ liệu cũ còn nguyên, vẫn xuất được sao lưu.
//!
//! Sinh lại tệp kỳ vọng (chỉ khi cố ý đổi phép chuyển đổi): `LVHV_REGEN_RELEASED=1 cargo test -p lvhv-store --test released`.
use lvhv_store::{canonical_json, open_backup, sha256_hex, AppInfo, ImportMode, Store, StoreError};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const FIX: &str = "0.2-dev-before-attachments";
const PASSWORD: &str = "mật khẩu thử nghiệm";

fn released() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/v0.2/schema/test-vectors/released")
}

fn tmpdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("lvhv-released-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let t = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &t);
        } else {
            std::fs::copy(e.path(), t).unwrap();
        }
    }
}

/// Ảnh chụp kho không có `app`/`exported_at` (như `state_hash`).
fn state_doc(s: &Store) -> Value {
    let mut d = s
        .snapshot(
            &AppInfo {
                version: "0.0.0".into(),
                platform: "pwa".into(),
            },
            "1970-01-01T00:00:00Z",
        )
        .unwrap();
    d.as_object_mut().unwrap().remove("app");
    d.as_object_mut().unwrap().remove("exported_at");
    d
}

fn expected_path() -> PathBuf {
    released().join("expected").join(format!("{FIX}.json"))
}

/// Nội dung logic của một tệp SQLite (user_version + mọi dòng của các bảng dữ liệu) — để so "bản sao dữ liệu cũ"
/// với kho gốc (byte phần đầu tệp đổi khi SQLite chuyển chế độ nhật ký WAL, dữ liệu thì không).
fn db_content(path: &Path) -> Vec<String> {
    let c = rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .unwrap();
    let mut out = vec![format!(
        "user_version={}",
        c.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap()
    )];
    for t in [
        "settings",
        "photo",
        "person",
        "event_type",
        "event",
        "event_person",
    ] {
        let mut st = c
            .prepare(&format!("SELECT * FROM {t} ORDER BY 1, 2"))
            .unwrap();
        let n = st.column_count();
        let rows = st
            .query_map([], |r| {
                Ok((0..n)
                    .map(|i| format!("{:?}", r.get_ref(i).unwrap()))
                    .collect::<Vec<_>>()
                    .join("|"))
            })
            .unwrap();
        for r in rows {
            out.push(format!("{t}:{}", r.unwrap()));
        }
    }
    out
}

/// Tệp bộ mẫu không bao giờ đổi: so sha256 với `manifest.json` của bộ mẫu.
#[test]
fn fixture_is_frozen() {
    let d = released().join(FIX);
    let m: Value =
        serde_json::from_slice(&std::fs::read(d.join("manifest.json")).unwrap()).unwrap();
    let files = m["files"].as_object().unwrap();
    assert!(files.len() >= 5);
    for (f, sha) in files {
        assert_eq!(
            sha256_hex(&std::fs::read(d.join(f)).unwrap()),
            sha.as_str().unwrap(),
            "{f} đã bị sửa — bộ mẫu đóng băng không được sửa"
        );
    }
}

#[test]
fn migrate_dev_store_zero_loss() {
    let fix = released().join(FIX);
    let dir = tmpdir("store");
    copy_dir(&fix.join("store"), &dir);
    let before_db = db_content(&dir.join(lvhv_store::DB_FILE));
    let old: Value =
        serde_json::from_slice(&std::fs::read(fix.join("store-snapshot.json")).unwrap()).unwrap();

    let s = Store::open(&dir).unwrap();
    let st = s.migration_status().clone();
    assert_eq!(
        (st.state, st.from_version, st.to_version),
        ("migrated", 1, 2),
        "{st:?}"
    );
    // Bản sao dữ liệu cũ = đúng tệp CSDL trước khi chuyển.
    assert!(
        db_content(Path::new(st.backup_path.as_ref().unwrap())) == before_db,
        "bản sao dữ liệu cũ phải giống kho gốc"
    );
    assert_eq!(s.schema_version().unwrap(), 2);
    let now = state_doc(&s);

    // 0 mất: cài đặt, loại sự kiện, sự kiện giống hệt; mọi hồ sơ còn, mọi trường giống hệt trừ phần chuyển đổi.
    for k in [
        "settings",
        "event_types",
        "events",
        "format",
        "schema_version",
    ] {
        assert_eq!(now[k], old[k], "{k}");
    }
    let olds = old["persons"].as_array().unwrap();
    let news = now["persons"].as_array().unwrap();
    assert_eq!(olds.len(), news.len());
    for (o, n) in olds.iter().zip(news) {
        let mut o2 = o.clone();
        let mut n2 = n.clone();
        let cert = o2
            .get_mut("death")
            .and_then(|d| d.get_mut("notice"))
            .and_then(Value::as_object_mut)
            .and_then(|x| x.remove("certificate_photo_ids"));
        if let Some(cert) = cert {
            // Danh sách gốc nằm nguyên trong legacy; mỗi id thành một tệp đính kèm loại giấy chứng tử.
            assert_eq!(n2["legacy"]["certificate_photo_ids"], cert);
            let items = n2["death"]["notice"]["attachments"]
                .as_array()
                .unwrap()
                .clone();
            assert_eq!(items.len(), cert.as_array().unwrap().len());
            for (it, pid) in items.iter().zip(cert.as_array().unwrap()) {
                assert_eq!(it["kind"], "giay_chung_tu");
                assert_eq!(it["id"], *pid);
                let pid = pid.as_str().unwrap();
                let aid = it["file_id"].as_str().unwrap();
                // Byte tệp đính kèm = byte ảnh gốc.
                let orig =
                    std::fs::read(fix.join("store/photos").join(format!("{pid}.img"))).unwrap();
                assert_eq!(s.attachment_bytes(aid).unwrap(), orig);
                let rec = n2["legacy"]["certificate_photos"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["id"] == pid)
                    .unwrap()
                    .clone();
                let photo_old = old["photos"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["id"] == pid)
                    .unwrap();
                assert_eq!(
                    &rec, photo_old,
                    "bản ghi Photo gốc (cả width/height) giữ trong legacy"
                );
            }
            n2.as_object_mut().unwrap().remove("legacy");
            n2["death"]["notice"]
                .as_object_mut()
                .unwrap()
                .remove("attachments");
        }
        assert_eq!(n2, o2, "hồ sơ {}", o["id"]);
    }
    // Ảnh chân dung vẫn đọc được; ảnh chỉ là giấy báo tử đã chuyển sang tệp đính kèm (cùng id, cùng byte).
    for p in old["photos"].as_array().unwrap() {
        let id = p["id"].as_str().unwrap();
        let orig = std::fs::read(fix.join("store/photos").join(format!("{id}.img"))).unwrap();
        let got = s
            .photo_bytes(id)
            .or_else(|_| s.attachment_bytes(id))
            .unwrap();
        assert_eq!(got, orig, "{id}");
    }

    // Kết quả = tệp kỳ vọng dùng chung với bản TS.
    let exp = json!({"state": now, "state_sha256": s.state_hash().unwrap()});
    if std::env::var("LVHV_REGEN_RELEASED").is_ok() {
        std::fs::create_dir_all(expected_path().parent().unwrap()).unwrap();
        std::fs::write(expected_path(), canonical_json(&exp) + "\n").unwrap();
    }
    let want: Value = serde_json::from_slice(&std::fs::read(expected_path()).unwrap()).unwrap();
    assert_eq!(exp, want, "kết quả chuyển đổi khác tệp kỳ vọng");

    // Mở lại: không chuyển nữa, không đổi gì.
    drop(s);
    let s = Store::open(&dir).unwrap();
    assert_eq!(s.migration_status().state, "ok");
    assert_eq!(state_doc(&s), now);
    let _ = std::fs::remove_dir_all(dir);
}

/// 04 B5 mục 5: tệp sao lưu của bản dựng cũ nhập được (Thay toàn bộ và Gộp) và cho cùng trạng thái.
#[test]
fn import_dev_backup() {
    let fix = released().join(FIX);
    let file = std::fs::read(fix.join("backup.lvhv")).unwrap();
    let v = open_backup(&file, PASSWORD).unwrap();
    let want: Value = serde_json::from_slice(&std::fs::read(expected_path()).unwrap()).unwrap();
    let dir = tmpdir("import");
    let mut s = Store::open(&dir).unwrap();
    s.apply_import(&v, ImportMode::Replace).unwrap();
    assert_eq!(s.state_hash().unwrap(), want["state_sha256"]);
    let dir2 = tmpdir("import-merge");
    let mut t = Store::open(&dir2).unwrap();
    t.apply_import(&v, ImportMode::Merge).unwrap();
    assert_eq!(state_doc(&t)["persons"], want["state"]["persons"]);
    assert_eq!(state_doc(&t)["attachments"], want["state"]["attachments"]);
    let _ = std::fs::remove_dir_all(dir);
    let _ = std::fs::remove_dir_all(dir2);
}

/// 04 B5 mục 3: chuyển đổi lỗi → quay lui, chỉ đọc, dữ liệu cũ còn nguyên, vẫn xuất được sao lưu.
#[test]
fn failed_migration_is_read_only() {
    let fix = released().join(FIX);
    let dir = tmpdir("fail");
    copy_dir(&fix.join("store"), &dir);
    // Làm hỏng một hồ sơ để bước kiểm 01 §6.2 sau chuyển đổi thất bại (chuỗi chưa cắt khoảng trắng).
    {
        let c = rusqlite::Connection::open(dir.join(lvhv_store::DB_FILE)).unwrap();
        let j: String = c
            .query_row(
                "SELECT json FROM person WHERE id = '0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let mut p: Value = serde_json::from_str(&j).unwrap();
        p["death"]["notice"]["place"] = json!("  Quê nhà");
        c.execute(
            "UPDATE person SET json = ?1 WHERE id = '0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02'",
            [canonical_json(&p)],
        )
        .unwrap();
    }
    let before = db_content(&dir.join(lvhv_store::DB_FILE));
    let mut s = Store::open(&dir).unwrap();
    let st = s.migration_status().clone();
    assert_eq!(
        (st.state, st.from_version, st.to_version),
        ("read_only", 1, 1)
    );
    assert!(st.error.is_some());
    assert!(db_content(Path::new(st.backup_path.as_ref().unwrap())) == before);
    assert!(
        db_content(&dir.join(lvhv_store::DB_FILE)) == before,
        "kho sau quay lui giống hệt trước"
    );
    assert_eq!(s.schema_version().unwrap(), 1, "giao dịch đã quay lui");
    assert!(s.attachments().unwrap().is_empty());
    let p = s.person("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02").unwrap();
    assert!(p["death"]["notice"]["certificate_photo_ids"].is_array());
    // Ghi → lỗi chỉ đọc; xuất sao lưu vẫn được và tệp nhập được vào bản mới (sau khi sửa dữ liệu hỏng thì chuyển).
    assert!(matches!(s.put_person(&p), Err(StoreError::ReadOnly)));
    assert!(matches!(
        s.delete_person("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02"),
        Err(StoreError::ReadOnly)
    ));
    let f = s
        .export_backup(
            PASSWORD,
            &AppInfo {
                version: "0.2.0".into(),
                platform: "android".into(),
            },
        )
        .unwrap();
    assert!(
        open_backup(&f, PASSWORD).is_err(),
        "dữ liệu hỏng vẫn bị kiểm khi nhập"
    );
    // Tệp của bản mới hơn → từ chối, không đọc một phần.
    drop(s);
    let c = rusqlite::Connection::open(dir.join(lvhv_store::DB_FILE)).unwrap();
    c.pragma_update(None, "user_version", 3).unwrap();
    drop(c);
    assert!(matches!(Store::open(&dir), Err(StoreError::NewerSchema(3))));
    let _ = std::fs::remove_dir_all(dir);
}

/// Báo cáo kiểm định B5 mục 4: chuyển đổi có thể làm hai hồ sơ dùng chung **một** tệp đính kèm (bộ mẫu: 5e0c là ảnh
/// giấy báo tử của cả 5e01 và 5e02). Xóa hồ sơ đếm tham chiếu: chỉ xóa tệp khi không còn hồ sơ nào dùng.
#[test]
fn shared_attachment_reference_counted() {
    let fix = released().join(FIX);
    let dir = tmpdir("shared");
    copy_dir(&fix.join("store"), &dir);
    let mut s = Store::open(&dir).unwrap();
    let shared = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e0c";
    let users = |s: &Store| -> usize {
        s.persons()
            .unwrap()
            .iter()
            .filter(|p| {
                lvhv_store::validate::person_attachment_refs(p)
                    .iter()
                    .any(|a| a == shared)
            })
            .count()
    };
    assert_eq!(users(&s), 2);
    s.delete_person("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01")
        .unwrap();
    assert_eq!(users(&s), 1);
    assert!(
        s.attachment_bytes(shared).is_ok(),
        "còn hồ sơ 5e02 dùng → giữ tệp"
    );
    assert!(dir
        .join("attachments")
        .join(format!("{shared}.bin"))
        .exists());
    s.delete_person("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02")
        .unwrap();
    assert!(
        s.attachment_bytes(shared).is_err(),
        "không còn ai dùng → xóa"
    );
    assert!(s.attachments().unwrap().is_empty());
    let _ = std::fs::remove_dir_all(dir);
}

/// Kiểm định: `OpenOptions::force_migration_failure` → bản sao trước chuyển đổi, quay lui, chỉ đọc.
#[test]
fn forced_migration_failure() {
    let fix = released().join(FIX);
    let dir = tmpdir("forced");
    copy_dir(&fix.join("store"), &dir);
    let before = db_content(&dir.join(lvhv_store::DB_FILE));
    let s = Store::open_with(
        &dir,
        lvhv_store::OpenOptions {
            force_migration_failure: true,
        },
    )
    .unwrap();
    let st = s.migration_status().clone();
    assert_eq!(st.state, "read_only");
    assert!(db_content(&dir.join(lvhv_store::DB_FILE)) == before);
    assert!(db_content(Path::new(st.backup_path.as_ref().unwrap())) == before);
    drop(s);
    // Mở lại bình thường → chuyển được.
    let s = Store::open(&dir).unwrap();
    assert_eq!(s.migration_status().state, "migrated");
    let _ = std::fs::remove_dir_all(dir);
}
