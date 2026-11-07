//! `lvhv-store-tool` — công cụ hộp đen cho bên kiểm định độc lập (docs/v0.2/10-CONG-CU-KIEM-DINH.md).
//!
//! Chỉ gọi API công khai: `lvhv_store` (mở/chuyển đổi kho, nhập sao lưu, ảnh chụp kho) và `lvhv_ffi`
//! (`LvhvStore::save_person_json` — đúng đường lưu hồ sơ mà ứng dụng Android/Apple/Windows gọi). Mọi kết quả in ra
//! stdout là **một dòng JSON chuẩn** (khóa sắp theo byte, không khoảng trắng), kể cả khi lỗi (`"ok": false`).
//!
//! Mã thoát: 0 = thành công; 1 = thao tác bị từ chối / lỗi (JSON vẫn in ra stdout); 2 = sai cú pháp lệnh.
//!
//! ```text
//! lvhv-store-tool open <thư-mục> [--force-migration-failure]
//! lvhv-store-tool import <tệp-sao-lưu> <mật-khẩu> [--merge|--replace] [--into <thư-mục>]
//! lvhv-store-tool save-person <thư-mục> <person.json | ->
//! lvhv-store-tool suggestions <deceased|living> [nam|nu]
//! lvhv-store-tool display-capitalize <chuỗi>
//! lvhv-store-tool check-media <photo|attachment> <mime> <tệp> [width height]
//! ```
//!
//! `--force-migration-failure` dùng `lvhv_store::OpenOptions { force_migration_failure: true }`.
#![forbid(unsafe_code)]

use lvhv_store::{
    canonical_json, migrate::MigrationStatus, sha256_hex, AppInfo, ImportMode, OpenOptions, Store,
    StoreError,
};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "dùng:
  lvhv-store-tool open <thư-mục> [--force-migration-failure]
  lvhv-store-tool import <tệp-sao-lưu> <mật-khẩu> [--merge|--replace] [--into <thư-mục>]
  lvhv-store-tool save-person <thư-mục> <person.json | ->
  lvhv-store-tool suggestions <deceased|living> [nam|nu]
  lvhv-store-tool display-capitalize <chuỗi>
  lvhv-store-tool check-media <photo|attachment> <mime> <tệp> [width height]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let r = match args.first().map(String::as_str) {
        Some("open") => cmd_open(&args[1..]),
        Some("import") => cmd_import(&args[1..]),
        Some("save-person") => cmd_save_person(&args[1..]),
        Some("suggestions") => cmd_suggestions(&args[1..]),
        Some("display-capitalize") => cmd_display(&args[1..]),
        Some("check-media") => cmd_check_media(&args[1..]),
        _ => Err(Usage(String::new())),
    };
    match r {
        Ok((v, ok)) => {
            println!("{}", canonical_json(&v));
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(Usage(m)) => {
            if !m.is_empty() {
                eprintln!("lỗi: {m}");
            }
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// Sai cú pháp lệnh (mã thoát 2).
struct Usage(String);

/// (JSON in ra, thành công?)
type Out = Result<(Value, bool), Usage>;

fn usage<T>(m: &str) -> Result<T, Usage> {
    Err(Usage(m.into()))
}

// ------------------------------------------------------------------------------------------------
// Định dạng lỗi và trạng thái
// ------------------------------------------------------------------------------------------------

fn err_json(e: &StoreError) -> Value {
    let kind = match e {
        StoreError::Invalid(_) => "Invalid",
        StoreError::Input(_) => "Input",
        StoreError::NewerSchema(_) => "NewerSchema",
        StoreError::ReadOnly => "ReadOnly",
        StoreError::NotFound => "NotFound",
        StoreError::Backup(_) => "Backup",
        StoreError::Sql(_) => "Sql",
        StoreError::Io(_) => "Io",
    };
    let mut o = json!({
        "kind": kind,
        "display": e.to_string(),
        "debug": format!("{e:?}"),
        "import_message": e.import_message(),
    });
    match e {
        StoreError::Invalid(v) => {
            o["code"] = json!(v.code);
            o["path"] = json!(v.path);
            o["msg"] = json!(v.msg);
        }
        StoreError::NewerSchema(n) => o["version"] = json!(n),
        StoreError::Backup(b) => o["backup_error"] = json!(format!("{b:?}")),
        _ => {}
    }
    o
}

fn ffi_err_json(e: &lvhv_ffi::LvhvDataError) -> Value {
    json!({ "display": e.to_string(), "debug": format!("{e:?}") })
}

fn status_json(m: &MigrationStatus) -> Value {
    json!({
        "state": m.state,
        "from_version": m.from_version,
        "to_version": m.to_version,
        "backup_path": m.backup_path,
        "backup_exists": m.backup_path.as_ref().map(|p| Path::new(p).exists()),
        "error": m.error,
    })
}

fn fail(command: &str, stage: &str, e: &StoreError) -> Out {
    Ok((
        json!({ "command": command, "ok": false, "stage": stage, "error": err_json(e) }),
        false,
    ))
}

/// Ảnh chụp toàn kho như `state_hash` băm (không có `app`, `exported_at`), cùng băm byte từng ảnh / tệp đính kèm.
fn store_json(s: &Store) -> Result<Value, StoreError> {
    let mut doc = s.snapshot(
        &AppInfo {
            version: "0.0.0".into(),
            platform: "pwa".into(),
        },
        "1970-01-01T00:00:00Z",
    )?;
    if let Some(o) = doc.as_object_mut() {
        o.remove("app");
        o.remove("exported_at");
    }
    let mut photos = Map::new();
    for p in s.photos()? {
        let id = p["id"].as_str().unwrap_or_default().to_owned();
        let v = match s.photo_bytes(&id) {
            Ok(b) => json!({ "bytes": b.len(), "sha256": sha256_hex(&b) }),
            Err(e) => json!({ "error": err_json(&e) }),
        };
        photos.insert(id, v);
    }
    let mut atts = Map::new();
    for a in s.attachments()? {
        let id = a["id"].as_str().unwrap_or_default().to_owned();
        let v = match s.attachment_bytes(&id) {
            Ok(b) => json!({ "bytes": b.len(), "sha256": sha256_hex(&b) }),
            Err(e) => json!({ "error": err_json(&e) }),
        };
        atts.insert(id, v);
    }
    Ok(json!({
        "document": doc,
        "photo_bytes": photos,
        "attachment_bytes": atts,
        "state_sha256": s.state_hash().ok(),
        "user_version": s.schema_version()?,
        "backup_pending_since": s.backup_pending_since()?,
    }))
}

/// Mọi tệp trong thư mục kho (đường dẫn tương đối, cỡ, sha256), sắp theo đường dẫn.
fn files_json(dir: &Path) -> Value {
    fn walk(base: &Path, d: &Path, out: &mut Vec<(String, Value)>) {
        let Ok(rd) = std::fs::read_dir(d) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else if let Ok(b) = std::fs::read(&p) {
                let rel = p
                    .strip_prefix(base)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((rel, json!({ "bytes": b.len(), "sha256": sha256_hex(&b) })));
            }
        }
    }
    let mut v = Vec::new();
    walk(dir, dir, &mut v);
    v.sort_by(|a, b| a.0.cmp(&b.0));
    Value::Object(v.into_iter().collect())
}

// ------------------------------------------------------------------------------------------------
// open
// ------------------------------------------------------------------------------------------------

fn cmd_open(a: &[String]) -> Out {
    let mut dir = None;
    let mut force = false;
    for x in a {
        match x.as_str() {
            "--force-migration-failure" => force = true,
            s if s.starts_with("--") => return usage(&format!("tùy chọn lạ: {s}")),
            s if dir.is_none() => dir = Some(PathBuf::from(s)),
            s => return usage(&format!("đối số thừa: {s}")),
        }
    }
    let Some(dir) = dir else {
        return usage("thiếu <thư-mục>");
    };
    let files_before = files_json(&dir);
    let mut s = match Store::open_with(
        &dir,
        OpenOptions {
            force_migration_failure: force,
            ..Default::default()
        },
    ) {
        Ok(s) => s,
        Err(e) => {
            return Ok((
                json!({ "command": "open", "ok": false, "stage": "open", "error": err_json(&e),
                        "files_before": files_before, "files_after": files_json(&dir) }),
                false,
            ))
        }
    };
    let status = s.migration_status().clone();
    let store = match store_json(&s) {
        Ok(v) => v,
        Err(e) => return fail("open", "read", &e),
    };
    let mut out = json!({
        "command": "open",
        "ok": true,
        "force_migration_failure": force,
        "migration_status": status_json(&status),
        "store": store,
        "files_before": files_before,
    });
    // Chế độ chỉ đọc (04 B5 mục 3): thử ghi (phải báo ReadOnly) và thử xuất sao lưu (phải được).
    if status.state == "read_only" {
        let mut probe = Map::new();
        let r = s.settings().and_then(|st| s.put_settings(&st));
        probe.insert("put_settings".into(), result_json(r));
        if let Some(p) = s.persons().ok().and_then(|v| v.into_iter().next()) {
            probe.insert("put_person".into(), result_json(s.put_person(&p)));
            let id = p["id"].as_str().unwrap_or_default().to_owned();
            probe.insert("delete_person".into(), result_json(s.delete_person(&id)));
        }
        let x = s.export_backup(
            "mật khẩu thử nghiệm",
            &AppInfo {
                version: "0.0.0".into(),
                platform: "pwa".into(),
            },
        );
        probe.insert(
            "export_backup".into(),
            match x {
                Ok(b) => json!({ "ok": true, "bytes": b.len() }),
                Err(e) => json!({ "ok": false, "error": err_json(&e) }),
            },
        );
        out["read_only_probe"] = Value::Object(probe);
    }
    drop(s);
    out["files_after"] = files_json(&dir);
    Ok((out, true))
}

fn result_json(r: Result<(), StoreError>) -> Value {
    match r {
        Ok(()) => json!({ "ok": true }),
        Err(e) => json!({ "ok": false, "error": err_json(&e) }),
    }
}

// ------------------------------------------------------------------------------------------------
// import
// ------------------------------------------------------------------------------------------------

fn cmd_import(a: &[String]) -> Out {
    let mut pos = Vec::new();
    let mut mode = None;
    let mut into = None;
    let mut it = a.iter();
    while let Some(x) = it.next() {
        match x.as_str() {
            "--merge" | "--replace" if mode.is_some() => return usage("chỉ một --merge/--replace"),
            "--merge" => mode = Some(ImportMode::Merge),
            "--replace" => mode = Some(ImportMode::Replace),
            "--into" => match it.next() {
                Some(d) => into = Some(PathBuf::from(d)),
                None => return usage("--into cần <thư-mục>"),
            },
            s if s.starts_with("--") => return usage(&format!("tùy chọn lạ: {s}")),
            s => pos.push(s.to_owned()),
        }
    }
    let [file, password] = pos.as_slice() else {
        return usage("cần <tệp-sao-lưu> <mật-khẩu>");
    };
    // Mặc định Gộp, như ứng dụng (BK-S1).
    let mode = mode.unwrap_or(ImportMode::Merge);
    let mode_s = if mode == ImportMode::Merge {
        "merge"
    } else {
        "replace"
    };
    let bytes = match std::fs::read(file) {
        Ok(b) => b,
        Err(e) => return fail("import", "read_file", &StoreError::Io(e)),
    };
    let (dir, temp) = match into {
        Some(d) => (d, false),
        None => {
            let d = std::env::temp_dir().join(format!(
                "lvhv-store-tool-{}-{}",
                std::process::id(),
                lvhv_store::now_rfc3339().replace(':', "")
            ));
            let _ = std::fs::remove_dir_all(&d);
            (d, true)
        }
    };
    let r = import_into(&bytes, password, mode, &dir);
    if temp {
        let _ = std::fs::remove_dir_all(&dir);
    }
    let mut out = match r {
        Ok(v) => v,
        Err((stage, e, extra)) => {
            let mut o =
                json!({ "command": "import", "ok": false, "stage": stage, "error": err_json(&e) });
            if let Some(x) = extra {
                o["before"] = x;
            }
            o["mode"] = json!(mode_s);
            o["file_sha256"] = json!(sha256_hex(&bytes));
            return Ok((o, false));
        }
    };
    out["mode"] = json!(mode_s);
    out["file_sha256"] = json!(sha256_hex(&bytes));
    out["into"] = if temp {
        Value::Null
    } else {
        json!(dir.to_string_lossy())
    };
    Ok((out, true))
}

type ImportErr = (&'static str, StoreError, Option<Value>);

fn import_into(
    bytes: &[u8],
    password: &str,
    mode: ImportMode,
    dir: &Path,
) -> Result<Value, ImportErr> {
    let vb = lvhv_store::open_backup(bytes, password).map_err(|e| ("open_backup", e, None))?;
    let p = vb.preview();
    let preview = json!({
        "persons": p.persons, "events": p.events, "photos": p.photos,
        "exported_at": p.exported_at, "platform": p.platform,
    });
    let mut s = Store::open(dir).map_err(|e| ("open_store", e, None))?;
    let status = status_json(s.migration_status());
    let before = json!({
        "migration_status": status,
        "state_sha256": s.state_hash().ok(),
    });
    let before_c = before.clone();
    s.apply_import(&vb, mode)
        .map_err(|e| ("apply_import", e, Some(before_c)))?;
    let store = store_json(&s).map_err(|e| ("read", e, None))?;
    Ok(json!({
        "command": "import",
        "ok": true,
        "preview": preview,
        "verified_document": vb.document(),
        "before": before,
        "store": store,
    }))
}

// ------------------------------------------------------------------------------------------------
// save-person
// ------------------------------------------------------------------------------------------------

fn cmd_save_person(a: &[String]) -> Out {
    let [dir, file] = a else {
        return usage("cần <thư-mục> <person.json | ->");
    };
    let input = if file == "-" {
        let mut t = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut t).map(|_| t)
    } else {
        std::fs::read_to_string(file)
    };
    let input = match input {
        Ok(t) => t,
        Err(e) => return fail("save-person", "read_file", &StoreError::Io(e)),
    };
    let st = match lvhv_ffi::LvhvStore::open(dir.clone()) {
        Ok(s) => s,
        Err(e) => {
            return Ok((
                json!({ "command": "save-person", "ok": false, "stage": "open", "error": ffi_err_json(&e) }),
                false,
            ))
        }
    };
    let m = st.migration_status();
    let ms = json!({
        "state": m.state, "from_version": m.from_version, "to_version": m.to_version,
        "backup_path": m.backup_path, "message": m.message,
    });
    match st.save_person_json(input) {
        Ok(saved) => {
            let v: Value = serde_json::from_str(&saved).unwrap_or(Value::String(saved));
            Ok((
                json!({ "command": "save-person", "ok": true, "migration_status": ms, "saved": v }),
                true,
            ))
        }
        Err(e) => Ok((
            json!({ "command": "save-person", "ok": false, "stage": "save_person_json",
                    "migration_status": ms, "error": ffi_err_json(&e) }),
            false,
        )),
    }
}

// ------------------------------------------------------------------------------------------------
// suggestions, display-capitalize
// ------------------------------------------------------------------------------------------------

fn cmd_suggestions(a: &[String]) -> Out {
    let (deceased, g) = match a {
        [k] => (k.as_str(), None),
        [k, g] => (k.as_str(), Some(g.as_str())),
        _ => return usage("cần <deceased|living> [nam|nu]"),
    };
    let deceased = match deceased {
        "deceased" => true,
        "living" => false,
        s => return usage(&format!("phải là deceased hoặc living: {s}")),
    };
    if let Some(g) = g {
        if g != "nam" && g != "nu" {
            return usage(&format!("giới tính phải là nam hoặc nu: {g}"));
        }
    }
    Ok((
        json!({
            "command": "suggestions",
            "ok": true,
            "deceased": deceased,
            "gioi_tinh": g,
            "danh_xung": lvhv_store::names::danh_xung_suggestions(deceased, g),
            "quan_he": lvhv_store::names::quan_he_suggestions(deceased),
        }),
        true,
    ))
}

fn cmd_display(a: &[String]) -> Out {
    let [s] = a else {
        return usage("cần <chuỗi>");
    };
    Ok((
        json!({
            "command": "display-capitalize",
            "ok": true,
            "input": s,
            "output": lvhv_store::names::display_capitalize(s),
        }),
        true,
    ))
}

/// Kiểm tra byte ảnh / tệp đính kèm (01 §3.8a, B6) — `lvhv_store::media`; cùng mã với `mediaVerdict` của PWA.
fn cmd_check_media(a: &[String]) -> Out {
    let (kind, mime, file, dims) = match a {
        [k, m, f] => (k, m, f, None),
        [k, m, f, w, h] => match (w.parse::<u32>(), h.parse::<u32>()) {
            (Ok(w), Ok(h)) => (k, m, f, Some((w, h))),
            _ => return usage("width/height phải là số nguyên không âm"),
        },
        _ => return usage("cần <photo|attachment> <mime> <tệp> [width height]"),
    };
    let bytes = match std::fs::read(file) {
        Ok(b) => b,
        Err(e) => return fail("check-media", "read_file", &StoreError::Io(e)),
    };
    let r = match kind.as_str() {
        "photo" => {
            let (w, h) = dims.unwrap_or((0, 0));
            lvhv_store::media::check_photo(mime, &bytes, w, h)
        }
        "attachment" => lvhv_store::media::check_attachment(mime, &bytes),
        s => return usage(&format!("phải là photo hoặc attachment: {s}")),
    };
    Ok((
        json!({
            "command": "check-media",
            "ok": true,
            "kind": kind,
            "mime": mime,
            "bytes": bytes.len(),
            "sha256": sha256_hex(&bytes),
            "verdict": r.map_or_else(|e| e.code(), |()| "ok"),
        }),
        true,
    ))
}
