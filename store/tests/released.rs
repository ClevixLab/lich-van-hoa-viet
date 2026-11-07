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
            ..Default::default()
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

fn open_at(dir: &Path, now_unix: i64, fail: bool) -> Store {
    Store::open_with(
        dir,
        lvhv_store::OpenOptions {
            force_migration_failure: fail,
            now_unix: Some(now_unix),
        },
    )
    .unwrap()
}

fn has_legacy(s: &Store) -> usize {
    s.persons()
        .unwrap()
        .iter()
        .filter(|p| p.get("legacy").is_some())
        .count()
}

/// BM2-06, BM2-07 (01 §7): 7 ngày sau chuyển đổi THÀNH CÔNG, bỏ `legacy` và xóa bản sao CSDL trước chuyển đổi; tệp
/// sao lưu xuất trong 7 ngày đó không mang `legacy`; chuyển đổi lỗi thì bản sao giữ nguyên.
#[test]
fn legacy_and_backup_pruned_after_7_days() {
    let fix = released().join(FIX);
    let dir = tmpdir("prune");
    copy_dir(&fix.join("store"), &dir);
    let t0 = lvhv_store::unix_from_rfc3339("2026-11-10T00:00:00Z").unwrap();
    let day = 86_400;
    let mut s = open_at(&dir, t0, false);
    assert_eq!(s.migration_status().state, "migrated");
    let bak = PathBuf::from(s.migration_status().backup_path.clone().unwrap());
    assert!(bak.exists());
    assert_eq!(has_legacy(&s), 2);
    // Xuất trong 7 ngày: không có `legacy` (tệp nhập vào máy khác không mang theo bản gốc nội bộ).
    let f = s
        .export_backup(
            PASSWORD,
            &AppInfo {
                version: "0.2.0".into(),
                platform: "android".into(),
            },
        )
        .unwrap();
    let v = open_backup(&f, PASSWORD).unwrap();
    assert!(v.document()["persons"]
        .as_array()
        .unwrap()
        .iter()
        .all(|p| p.get("legacy").is_none()));
    let before_hash = s.state_hash().unwrap();
    drop(s);
    // 6 ngày 23 giờ: chưa dọn.
    let s = open_at(&dir, t0 + 7 * day - 3600, false);
    assert_eq!(has_legacy(&s), 2);
    assert!(bak.exists());
    assert_eq!(s.state_hash().unwrap(), before_hash);
    drop(s);
    // Đủ 7 ngày: bỏ legacy, xóa bản sao; dữ liệu khác giữ nguyên (rev không đổi).
    let s = open_at(&dir, t0 + 7 * day, false);
    assert_eq!(s.migration_status().state, "ok");
    assert_eq!(has_legacy(&s), 0);
    assert!(
        !bak.exists(),
        "bản sao trước chuyển đổi phải bị xóa sau 7 ngày"
    );
    let want: Value = serde_json::from_slice(&std::fs::read(expected_path()).unwrap()).unwrap();
    let mut exp_persons = want["state"]["persons"].clone();
    for p in exp_persons.as_array_mut().unwrap() {
        p.as_object_mut().unwrap().remove("legacy");
    }
    assert_eq!(state_doc(&s)["persons"], exp_persons);
    assert_eq!(state_doc(&s)["attachments"], want["state"]["attachments"]);
    drop(s);
    let _ = std::fs::remove_dir_all(dir);
    // Chuyển đổi lỗi: không bắt đầu đếm; 30 ngày sau bản sao vẫn còn.
    let dir = tmpdir("prune-fail");
    copy_dir(&fix.join("store"), &dir);
    let s = open_at(&dir, t0, true);
    assert_eq!(s.migration_status().state, "read_only");
    let bak = PathBuf::from(s.migration_status().backup_path.clone().unwrap());
    drop(s);
    let s = open_at(&dir, t0 + 30 * day, true);
    assert_eq!(s.migration_status().state, "read_only");
    assert!(bak.exists(), "chuyển đổi lỗi: bản sao giữ nguyên");
    drop(s);
    let _ = std::fs::remove_dir_all(dir);
}

/// Mọi tệp trong thư mục kho (đường dẫn tương đối), trừ tệp tạm của SQLite.
fn file_set(dir: &Path) -> std::collections::BTreeSet<String> {
    fn walk(base: &Path, d: &Path, out: &mut std::collections::BTreeSet<String>) {
        for e in std::fs::read_dir(d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else {
                let rel = p.strip_prefix(base).unwrap().to_string_lossy().into_owned();
                if !rel.ends_with("-wal") && !rel.ends_with("-shm") {
                    out.insert(rel);
                }
            }
        }
    }
    let mut out = Default::default();
    walk(dir, dir, &mut out);
    out
}

/// Báo cáo PR #36: (1) bản sao trước chuyển đổi giống TỪNG BYTE kho cũ; (2) chuyển đổi lỗi (sau khi đã ghi tệp đính kèm)
/// không để lại tệp mồ côi — tập tệp của thư mục kho như trước, chỉ thêm bản sao trước chuyển đổi.
#[test]
fn failed_migration_leaves_folder_unchanged_and_copy_is_byte_identical() {
    let fix = released().join(FIX);
    let orig = std::fs::read(fix.join("store").join(lvhv_store::DB_FILE)).unwrap();
    for fail in [true, false] {
        let dir = tmpdir(if fail { "orph-fail" } else { "orph-ok" });
        copy_dir(&fix.join("store"), &dir);
        let before = file_set(&dir);
        let s = Store::open_with(
            &dir,
            lvhv_store::OpenOptions {
                force_migration_failure: fail,
                ..Default::default()
            },
        )
        .unwrap();
        let st = s.migration_status().clone();
        let bak = PathBuf::from(st.backup_path.clone().unwrap());
        assert_eq!(
            std::fs::read(&bak).unwrap(),
            orig,
            "bản sao phải giống từng byte kho cũ"
        );
        drop(s);
        let mut after = file_set(&dir);
        assert!(after.remove(&format!("{}.pre-v2.bak", lvhv_store::DB_FILE)));
        if fail {
            assert_eq!(st.state, "read_only");
            assert_eq!(after, before, "chuyển đổi lỗi không để lại tệp nào");
        } else {
            assert_eq!(st.state, "migrated");
            let added: Vec<_> = after.difference(&before).cloned().collect();
            assert_eq!(added.len(), 2, "{added:?}"); // hai tệp đính kèm mới
            assert!(added.iter().all(|f| f.starts_with("attachments/")));
        }
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// Tên các tệp `photos/*.img` (không đuôi) và id các bản ghi `photo`.
fn img_files(dir: &Path) -> std::collections::BTreeSet<String> {
    std::fs::read_dir(dir.join("photos"))
        .unwrap()
        .flatten()
        .filter_map(|e| {
            e.file_name()
                .to_string_lossy()
                .strip_suffix(".img")
                .map(str::to_owned)
        })
        .collect()
}
fn photo_records(s: &Store) -> std::collections::BTreeSet<String> {
    s.photos()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap().to_owned())
        .collect()
}

/// Rà soát bảo mật vòng 2, BM3-01: tệp ảnh gốc giấy báo tử đã chuyển sang tệp đính kèm (`photos/<id>.img`) giữ để quay
/// lui trong 7 ngày, rồi bị xóa khi dọn sau 7 ngày; xóa hồ sơ (trong 7 ngày) cũng xóa nó khi không còn hồ sơ nào giữ
/// ảnh đó trong `legacy`. Cuối cùng không còn tệp `.img` nào không có bản ghi.
#[test]
fn moved_certificate_photos_removed_after_7_days_and_on_delete() {
    let fix = released().join(FIX);
    let t0 = lvhv_store::unix_from_rfc3339("2026-11-10T00:00:00Z").unwrap();
    let day = 86_400;
    // (a) Chuyển → trong 7 ngày tệp gốc còn (quay lui được) → +8 ngày: chỉ còn tệp có bản ghi → xóa hết hồ sơ: 0 tệp.
    let dir = tmpdir("moved-7d");
    copy_dir(&fix.join("store"), &dir);
    let before = img_files(&dir);
    let s = open_at(&dir, t0, false);
    assert_eq!(s.migration_status().state, "migrated");
    let moved: Vec<String> = before.difference(&photo_records(&s)).cloned().collect();
    assert!(
        !moved.is_empty(),
        "bộ mẫu có ảnh giấy báo tử được chuyển hẳn"
    );
    assert_eq!(
        img_files(&dir),
        before,
        "trong 7 ngày: tệp gốc còn để quay lui"
    );
    drop(s);
    let s = open_at(&dir, t0 + 7 * day - 3600, false);
    assert_eq!(img_files(&dir), before);
    drop(s);
    let mut s = open_at(&dir, t0 + 8 * day, false);
    assert_eq!(
        img_files(&dir),
        photo_records(&s),
        "sau 7 ngày: không còn tệp ảnh mồ côi"
    );
    for p in s.persons().unwrap() {
        s.delete_person(p["id"].as_str().unwrap()).unwrap();
    }
    assert!(img_files(&dir).is_empty() && photo_records(&s).is_empty());
    assert!(s.attachments().unwrap().is_empty());
    drop(s);
    let _ = std::fs::remove_dir_all(&dir);

    // (b) Trong 7 ngày, xóa hồ sơ: ảnh giấy báo tử dùng chung (5e0c của 5e01 và 5e02) còn tới khi hồ sơ cuối cùng giữ
    // nó trong `legacy` bị xóa; xóa hết → 0 tệp `.img`, ngay cả trước hạn 7 ngày.
    let dir = tmpdir("moved-delete");
    copy_dir(&fix.join("store"), &dir);
    let mut s = open_at(&dir, t0, false);
    let shared = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e0c";
    assert!(moved.iter().any(|m| m == shared));
    assert!(img_files(&dir).contains(shared));
    s.delete_person("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01")
        .unwrap();
    assert!(
        img_files(&dir).contains(shared),
        "hồ sơ 5e02 còn giữ ảnh này trong legacy"
    );
    s.delete_person("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02")
        .unwrap();
    assert!(!img_files(&dir).contains(shared));
    for p in s.persons().unwrap() {
        s.delete_person(p["id"].as_str().unwrap()).unwrap();
    }
    assert!(img_files(&dir).is_empty(), "{:?}", img_files(&dir));
    drop(s);
    // Mở lại sau 8 ngày: dọn không lỗi khi tệp đã bị xóa trước.
    let s = open_at(&dir, t0 + 8 * day, false);
    assert_eq!(s.migration_status().state, "ok");
    assert!(img_files(&dir).is_empty());
    drop(s);
    let _ = std::fs::remove_dir_all(dir);
}

/// BM2-11 (01 §4.2): `secure_delete` bật — chuỗi của hồ sơ đã xóa không còn trong `lvhv.sqlite` hay `-wal`.
#[test]
fn deleted_person_text_not_left_in_database_files() {
    let dir = tmpdir("secure-delete");
    let mut s = Store::open(&dir).unwrap();
    let marker = "Chuỗi-đánh-dấu-xóa-an-toàn-7f3a";
    let ex: Value =
        serde_json::from_slice(&std::fs::read(released().join("../example-v1.json")).unwrap())
            .unwrap();
    let mut p = ex["persons"][0].clone();
    let id = p["id"].as_str().unwrap().to_owned();
    p["photo_id"] = Value::Null;
    p["ho_ten"] = json!(marker);
    p["ghi_chu"] = json!(marker);
    s.put_person(&p).unwrap();
    let find = |dir: &Path| -> bool {
        ["lvhv.sqlite", "lvhv.sqlite-wal"].iter().any(|f| {
            std::fs::read(dir.join(f))
                .map(|b| b.windows(marker.len()).any(|w| w == marker.as_bytes()))
                .unwrap_or(false)
        })
    };
    assert!(
        find(&dir),
        "đối chứng: chuỗi có trong tệp CSDL trước khi xóa"
    );
    s.delete_person(&id).unwrap();
    assert!(
        !find(&dir),
        "sau khi xóa: chuỗi không còn trong lvhv.sqlite / -wal"
    );
    drop(s);
    assert!(!find(&dir));
    let _ = std::fs::remove_dir_all(dir);
}

// ---------- Bộ mẫu Android 0.2.1 (bản Android đầu tiên có mô-đun Người thân; README của bộ mẫu ghi cách tạo) ----------

const FIX_ANDROID_021: &str = "0.2.1/android";

fn android_021_expected() -> Value {
    serde_json::from_slice(
        &std::fs::read(released().join("expected").join("0.2.1-android.json")).unwrap(),
    )
    .unwrap()
}

#[test]
fn released_0_2_1_android_is_frozen() {
    let d = released().join(FIX_ANDROID_021);
    let m: Value =
        serde_json::from_slice(&std::fs::read(d.join("manifest.json")).unwrap()).unwrap();
    let files = m["files"].as_object().unwrap();
    assert!(files.len() >= 10);
    for (f, sha) in files {
        assert_eq!(
            sha256_hex(&std::fs::read(d.join(f)).unwrap()),
            sha.as_str().unwrap(),
            "{f} đã bị sửa — bộ mẫu đóng băng không được sửa"
        );
    }
}

/// Kho cùng phiên bản lược đồ: mở bản sao → `ok` (không chuyển đổi), trạng thái đúng từng byte như tệp kỳ vọng,
/// từng ảnh/tệp đúng sha256; tệp sao lưu của bộ mẫu nhập vào kho trống cho cùng trạng thái.
#[test]
fn released_0_2_1_android_opens_unchanged() {
    let fix = released().join(FIX_ANDROID_021);
    let want = android_021_expected();
    let dir = tmpdir("android-021");
    copy_dir(&fix.join("store"), &dir);
    let s = Store::open(&dir).unwrap();
    assert_eq!(s.migration_status().state, "ok");
    assert_eq!(state_doc(&s), want["state"]);
    assert_eq!(s.state_hash().unwrap(), want["state_sha256"]);
    assert_eq!(
        json!(s.lock_screen_names().unwrap()),
        want["lock_screen_names"]
    );
    for (id, sha) in want["photo_sha256"].as_object().unwrap() {
        assert_eq!(
            sha256_hex(&s.photo_bytes(id).unwrap()),
            sha.as_str().unwrap()
        );
    }
    for (id, sha) in want["attachment_sha256"].as_object().unwrap() {
        assert_eq!(
            sha256_hex(&s.attachment_bytes(id).unwrap()),
            sha.as_str().unwrap()
        );
    }
    drop(s);
    let v = open_backup(&std::fs::read(fix.join("backup.lvhv")).unwrap(), PASSWORD).unwrap();
    let dir2 = tmpdir("android-021-import");
    let mut t = Store::open(&dir2).unwrap();
    t.apply_import(&v, ImportMode::Replace).unwrap();
    assert_eq!(t.state_hash().unwrap(), want["state_sha256"]);
    let _ = std::fs::remove_dir_all(dir);
    let _ = std::fs::remove_dir_all(dir2);
}
