//! `lvhv-store-tool` — công cụ hộp đen cho bên kiểm định độc lập (docs/v0.2/10-CONG-CU-KIEM-DINH.md).
//!
//! Chỉ gọi API công khai: `lvhv_store` (mở/chuyển đổi kho, nhập sao lưu, ảnh chụp kho) và `lvhv_ffi`
//! (`LvhvStore::save_person_json` — đúng đường lưu hồ sơ mà ứng dụng Android/Apple/Windows gọi). Mọi kết quả in ra
//! stdout là **một dòng JSON chuẩn** (khóa sắp theo byte, không khoảng trắng), kể cả khi lỗi (`"ok": false`).
//!
//! Mã thoát: 0 = thành công; 1 = thao tác bị từ chối / lỗi (JSON vẫn in ra stdout); 2 = sai cú pháp lệnh.
//!
//! ```text
//! lvhv-store-tool open <thư-mục> [--force-migration-failure] [--now <unix_s>]
//! lvhv-store-tool import <tệp-sao-lưu> <mật-khẩu> [--merge|--replace] [--into <thư-mục>]
//! lvhv-store-tool save-person <thư-mục> <person.json | ->
//! lvhv-store-tool suggestions <deceased|living> [nam|nu]
//! lvhv-store-tool display-capitalize <chuỗi>
//! lvhv-store-tool check-media <photo|attachment> <mime> <tệp> [width height]
//! lvhv-store-tool export <thư-mục> <mật-khẩu> <tệp-ra> [--now <unix_s>]
//! lvhv-store-tool inspect-export <tệp-sao-lưu> <mật-khẩu>
//! lvhv-store-tool add-photo <thư-mục> <mime> <tệp> <width> <height> [--id <uuid>]
//! lvhv-store-tool delete-person <thư-mục> <id>
//! lvhv-store-tool scenario tray_photo <thư-mục-làm-việc>
//! ```
//!
//! `add-photo` gọi `LvhvStore::add_photo` (ảnh chân dung hay ảnh mâm lễ — YC-2026-10 — đều là `Photo`); gắn vào hồ sơ
//! bằng `save-person` với `photo_id` / `memorial.tray_photo_id`. `delete-person` gọi `LvhvStore::delete_person`.
//! `scenario tray_photo` chạy kịch bản R15 (c)(d)(e)(f) cố định (04 Bổ sung B8) trên kho mới trong thư mục làm việc và in
//! kết quả đã chuẩn hóa (không có thời điểm, không có byte ngẫu nhiên) — bản TS in đúng cùng JSON (R15 (g)).
//!
//! `--force-migration-failure`, `--now` dùng `lvhv_store::OpenOptions { force_migration_failure, now_unix }`.
//! `export` gọi `Store::export_backup` — hàm mà `LvhvStore::export_backup` của ứng dụng bọc lại (cùng tài liệu, cùng mã
//! hóa); mở kho bằng `open_with` để `--now` áp cho việc dọn sau 7 ngày như `open`; `inspect-export` giải mã bằng `lvhv_backup::import`
//! và in bản rõ (JSON + danh sách khối), không kiểm nội dung.
#![forbid(unsafe_code)]

use lvhv_store::{
    canonical_json, migrate::MigrationStatus, sha256_hex, AppInfo, ImportMode, OpenOptions, Store,
    StoreError,
};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "dùng:
  lvhv-store-tool open <thư-mục> [--force-migration-failure] [--now <unix_s>]
  lvhv-store-tool import <tệp-sao-lưu> <mật-khẩu> [--merge|--replace] [--into <thư-mục>]
  lvhv-store-tool save-person <thư-mục> <person.json | ->
  lvhv-store-tool suggestions <deceased|living> [nam|nu]
  lvhv-store-tool display-capitalize <chuỗi>
  lvhv-store-tool check-media <photo|attachment> <mime> <tệp> [width height]
  lvhv-store-tool export <thư-mục> <mật-khẩu> <tệp-ra> [--now <unix_s>]
  lvhv-store-tool inspect-export <tệp-sao-lưu> <mật-khẩu>
  lvhv-store-tool add-photo <thư-mục> <mime> <tệp> <width> <height> [--id <uuid>]
  lvhv-store-tool delete-person <thư-mục> <id>
  lvhv-store-tool scenario tray_photo <thư-mục-làm-việc>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let r = match args.first().map(String::as_str) {
        Some("open") => cmd_open(&args[1..]),
        Some("import") => cmd_import(&args[1..]),
        Some("save-person") => cmd_save_person(&args[1..]),
        Some("suggestions") => cmd_suggestions(&args[1..]),
        Some("display-capitalize") => cmd_display(&args[1..]),
        Some("check-media") => cmd_check_media(&args[1..]),
        Some("export") => cmd_export(&args[1..]),
        Some("inspect-export") => cmd_inspect_export(&args[1..]),
        Some("add-photo") => cmd_add_photo(&args[1..]),
        Some("delete-person") => cmd_delete_person(&args[1..]),
        Some("scenario") => cmd_scenario(&args[1..]),
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
    let mut now = None;
    let mut it = a.iter();
    while let Some(x) = it.next() {
        match x.as_str() {
            "--force-migration-failure" => force = true,
            "--now" => match it.next().map(|v| v.parse::<i64>()) {
                Some(Ok(v)) => now = Some(v),
                _ => return usage("--now cần <unix_s> (số nguyên giây)"),
            },
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
            now_unix: now,
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
        "now_unix": now,
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

// ------------------------------------------------------------------------------------------------
// export, inspect-export
// ------------------------------------------------------------------------------------------------

/// Có khóa `legacy` ở bất kỳ mức nào không (01 §7: tệp sao lưu xuất ra không được mang `legacy`).
fn legacy_paths(v: &Value, path: &str, out: &mut Vec<String>) {
    match v {
        Value::Object(o) => {
            for (k, x) in o {
                let p = format!("{path}/{k}");
                if k == "legacy" {
                    out.push(p.clone());
                }
                legacy_paths(x, &p, out);
            }
        }
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                legacy_paths(x, &format!("{path}/{i}"), out);
            }
        }
        _ => {}
    }
}

fn cmd_export(a: &[String]) -> Out {
    let mut pos = Vec::new();
    let mut now = None;
    let mut it = a.iter();
    while let Some(x) = it.next() {
        match x.as_str() {
            "--now" => match it.next().map(|v| v.parse::<i64>()) {
                Some(Ok(v)) => now = Some(v),
                _ => return usage("--now cần <unix_s> (số nguyên giây)"),
            },
            s if s.starts_with("--") => return usage(&format!("tùy chọn lạ: {s}")),
            s => pos.push(s.to_owned()),
        }
    }
    let [dir, password, out_file] = pos.as_slice() else {
        return usage("cần <thư-mục> <mật-khẩu> <tệp-ra>");
    };
    let mut s = match Store::open_with(
        dir,
        OpenOptions {
            now_unix: now,
            ..Default::default()
        },
    ) {
        Ok(s) => s,
        Err(e) => return fail("export", "open", &e),
    };
    let ms = status_json(s.migration_status());
    let bytes = match s.export_backup(
        password,
        &AppInfo {
            version: "0.2.0".into(),
            platform: "android".into(),
        },
    ) {
        Ok(b) => b,
        Err(e) => {
            return Ok((
                json!({ "command": "export", "ok": false, "stage": "export_backup",
                        "migration_status": ms, "error": err_json(&e) }),
                false,
            ))
        }
    };
    drop(s);
    if let Err(e) = std::fs::write(out_file, &bytes) {
        return fail("export", "write_file", &StoreError::Io(e));
    }
    Ok((
        json!({ "command": "export", "ok": true, "now_unix": now, "migration_status": ms,
                "out_file": out_file, "bytes": bytes.len(), "sha256": sha256_hex(&bytes) }),
        true,
    ))
}

fn cmd_inspect_export(a: &[String]) -> Out {
    let [file, password] = a else {
        return usage("cần <tệp-sao-lưu> <mật-khẩu>");
    };
    let bytes = match std::fs::read(file) {
        Ok(b) => b,
        Err(e) => return fail("inspect-export", "read_file", &StoreError::Io(e)),
    };
    let c = match lvhv_backup::import(&bytes, password) {
        Ok(c) => c,
        Err(e) => return fail("inspect-export", "decrypt", &StoreError::Backup(e)),
    };
    let doc: Value = match serde_json::from_str(&c.json) {
        Ok(v) => v,
        Err(e) => {
            return fail(
                "inspect-export",
                "parse_json",
                &StoreError::Input(format!("JSON bản rõ hỏng: {e}")),
            )
        }
    };
    let mut legacy = Vec::new();
    legacy_paths(&doc, "", &mut legacy);
    let blobs: Vec<Value> = c
        .photos
        .iter()
        .map(|b| {
            json!({ "id": lvhv_backup::uuid_to_string(&b.id), "bytes": b.bytes.len(),
                    "sha256": sha256_hex(&b.bytes) })
        })
        .collect();
    Ok((
        json!({
            "command": "inspect-export",
            "ok": true,
            "file_sha256": sha256_hex(&bytes),
            "json_sha256": sha256_hex(c.json.as_bytes()),
            "json_is_canonical": canonical_json(&doc) == c.json,
            "document": doc,
            "blobs": blobs,
            "legacy_paths": legacy,
        }),
        true,
    ))
}

// ------------------------------------------------------------------------------------------------
// add-photo, delete-person (YC-2026-10)
// ------------------------------------------------------------------------------------------------

fn new_uuid_v4() -> String {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).expect("getrandom");
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    lvhv_backup::uuid_to_string(&b)
}

fn open_ffi(command: &str, dir: &str) -> Result<std::sync::Arc<lvhv_ffi::LvhvStore>, Value> {
    lvhv_ffi::LvhvStore::open(dir.to_owned()).map_err(
        |e| json!({ "command": command, "ok": false, "stage": "open", "error": ffi_err_json(&e) }),
    )
}

fn cmd_add_photo(a: &[String]) -> Out {
    let mut pos = Vec::new();
    let mut id = None;
    let mut it = a.iter();
    while let Some(x) = it.next() {
        match x.as_str() {
            "--id" => match it.next() {
                Some(v) => id = Some(v.clone()),
                None => return usage("--id cần <uuid>"),
            },
            s if s.starts_with("--") => return usage(&format!("tùy chọn lạ: {s}")),
            s => pos.push(s.to_owned()),
        }
    }
    let [dir, mime, file, w, h] = pos.as_slice() else {
        return usage("cần <thư-mục> <mime> <tệp> <width> <height>");
    };
    let (Ok(w), Ok(h)) = (w.parse::<u32>(), h.parse::<u32>()) else {
        return usage("width/height phải là số nguyên không âm");
    };
    let bytes = match std::fs::read(file) {
        Ok(b) => b,
        Err(e) => return fail("add-photo", "read_file", &StoreError::Io(e)),
    };
    let st = match open_ffi("add-photo", dir) {
        Ok(s) => s,
        Err(v) => return Ok((v, false)),
    };
    let id = id.unwrap_or_else(new_uuid_v4);
    match st.add_photo(id, mime.clone(), w, h, bytes) {
        Ok(t) => Ok((
            json!({ "command": "add-photo", "ok": true, "photo": serde_json::from_str::<Value>(&t).unwrap_or(Value::Null) }),
            true,
        )),
        Err(e) => Ok((
            json!({ "command": "add-photo", "ok": false, "stage": "add_photo", "error": ffi_err_json(&e),
                    "outcome": ffi_outcome(&e) }),
            false,
        )),
    }
}

fn cmd_delete_person(a: &[String]) -> Out {
    let [dir, id] = a else {
        return usage("cần <thư-mục> <id>");
    };
    let st = match open_ffi("delete-person", dir) {
        Ok(s) => s,
        Err(v) => return Ok((v, false)),
    };
    match st.delete_person(id.clone()) {
        Ok(()) => Ok((
            json!({ "command": "delete-person", "ok": true, "id": id }),
            true,
        )),
        Err(e) => Ok((
            json!({ "command": "delete-person", "ok": false, "stage": "delete_person", "error": ffi_err_json(&e),
                    "outcome": ffi_outcome(&e) }),
            false,
        )),
    }
}

// ------------------------------------------------------------------------------------------------
// scenario tray_photo (04 Bổ sung B8, R15 (c)–(g))
// ------------------------------------------------------------------------------------------------

/// "V3 /person/x" trong thông điệp lỗi (dạng `ValidationError` hiển thị: "<mã> <đường dẫn>: <câu>").
fn v_code(msg: &str) -> Option<String> {
    let b = msg.as_bytes();
    for i in 0..b.len().saturating_sub(3) {
        if b[i] == b'V'
            && b[i + 1].is_ascii_digit()
            && b[i + 2] == b' '
            && b[i + 3] == b'/'
            && (i == 0 || b[i - 1] == b' ')
        {
            let rest = &msg[i..];
            let end = rest.find(": ").unwrap_or(rest.len());
            return Some(rest[..end].to_owned());
        }
    }
    None
}

/// Kết quả chuẩn hóa của một thao tác qua FFI — cùng dạng với bản TS: "ok" | "invalid <mã> <đường dẫn>" | "input" |
/// "not_found" | "backup" | "storage".
fn ffi_outcome(e: &lvhv_ffi::LvhvDataError) -> String {
    use lvhv_ffi::LvhvDataError as E;
    match e {
        E::Invalid(m) => v_code(m).map_or_else(|| "input".into(), |c| format!("invalid {c}")),
        E::NotFound(_) => "not_found".into(),
        E::Backup(_) => "backup".into(),
        E::Storage(_) => "storage".into(),
    }
}

fn outcome<T>(r: &Result<T, lvhv_ffi::LvhvDataError>) -> String {
    match r {
        Ok(_) => "ok".into(),
        Err(e) => ffi_outcome(e),
    }
}

fn store_outcome<T>(r: &lvhv_store::Result<T>) -> String {
    match r {
        Ok(_) => "ok".into(),
        Err(StoreError::Invalid(v)) => format!("invalid {} {}", v.code, v.path),
        Err(StoreError::Input(_)) => "input".into(),
        Err(StoreError::NotFound) => "not_found".into(),
        Err(StoreError::Backup(_) | StoreError::NewerSchema(_)) => "backup".into(),
        Err(_) => "storage".into(),
    }
}

const TP: &str = "mật khẩu thử nghiệm";
const T1: &str = "0b8e4f52-3c1a-4d2e-9f60-0000000000a1";
const T2: &str = "0b8e4f52-3c1a-4d2e-9f60-0000000000a2";
const T3: &str = "0b8e4f52-3c1a-4d2e-9f60-0000000000a3";
const T4: &str = "0b8e4f52-3c1a-4d2e-9f60-0000000000a4";
const TX: &str = "0b8e4f52-3c1a-4d2e-9f60-0000000000af";
const D1: &str = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01";
const D2: &str = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02";
const L3: &str = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e03";
const PORTRAIT: &str = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e09";

fn tv() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/v0.2/schema/test-vectors")
}

struct Tray {
    st: std::sync::Arc<lvhv_ffi::LvhvStore>,
    dir: PathBuf,
}

impl Tray {
    fn doc(&self) -> Value {
        serde_json::from_str(&self.st.dataset_json().unwrap()).unwrap()
    }
    fn person(&self, id: &str) -> Value {
        serde_json::from_str(&self.st.person_json(id.into()).unwrap()).unwrap()
    }
    fn save(&self, p: &Value) -> String {
        outcome(&self.st.save_person_json(p.to_string()))
    }
    fn with_tray(&self, id: &str, tray: Value) -> String {
        let mut p = self.person(id);
        p["memorial"]["tray_photo_id"] = tray;
        self.save(&p)
    }
    fn add(&self, id: &str, file: &str, w: u32, h: u32) -> String {
        let b = std::fs::read(tv().join(file)).unwrap();
        outcome(&self.st.add_photo(id.into(), "image/jpeg".into(), w, h, b))
    }
    /// Ảnh: bản ghi (id sắp), tệp `photos/*.img`, ảnh mồ côi (bản ghi không hồ sơ nào dùng; tệp không có bản ghi).
    fn photos(&self) -> Value {
        let d = self.doc();
        let mut ids: Vec<String> = d["photos"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["id"].as_str().unwrap().to_owned())
            .collect();
        ids.sort();
        let used: std::collections::HashSet<String> = d["persons"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(lvhv_store::validate::person_photo_refs)
            .collect();
        let files: std::collections::BTreeSet<String> = std::fs::read_dir(self.dir.join("photos"))
            .map(|rd| {
                rd.flatten()
                    .filter_map(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .strip_suffix(".img")
                            .map(str::to_owned)
                    })
                    .collect()
            })
            .unwrap_or_default();
        let orphan_records = ids.iter().filter(|i| !used.contains(*i)).count();
        let orphan_files = files.iter().filter(|f| !ids.contains(f)).count()
            + ids.iter().filter(|i| !files.contains(*i)).count();
        json!({ "ids": ids, "orphan_records": orphan_records, "orphan_files": orphan_files })
    }
    fn tray_of(&self, id: &str) -> Value {
        let p = self.person(id);
        match p["memorial"].as_object() {
            None => json!("no_memorial"),
            Some(m) if !m.contains_key("tray_photo_id") => json!("absent"),
            Some(m) => m["tray_photo_id"].clone(),
        }
    }
    fn photo_sha(&self, id: &str) -> Value {
        self.st
            .photo_bytes(id.into())
            .map(|b| json!(sha256_hex(&b)))
            .unwrap_or(Value::Null)
    }
}

fn fresh(dir: &Path) -> Tray {
    let _ = std::fs::remove_dir_all(dir);
    Tray {
        st: lvhv_ffi::LvhvStore::open(dir.to_string_lossy().into_owned()).unwrap(),
        dir: dir.to_owned(),
    }
}

fn tray_import(t: &Tray, file: &[u8], mode: lvhv_ffi::BackupImportMode) -> String {
    match lvhv_ffi::open_backup(file.to_vec(), TP.into()) {
        Ok(v) => outcome(&t.st.apply_import(v, mode)),
        Err(e) => ffi_outcome(&e),
    }
}

/// Tài liệu (bỏ `app`, `exported_at`) + danh sách khối của kho — để dựng tệp sao lưu sửa tay.
fn contents_of(t: &Tray) -> (Value, Vec<(String, Vec<u8>)>) {
    let mut d = t.doc();
    d["app"] = json!({ "name": "Lịch và Văn hóa Việt", "version": "0.2.0", "platform": "android" });
    d["exported_at"] = json!("2026-11-07T00:00:00Z");
    let blobs = d["photos"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            let id = p["id"].as_str().unwrap().to_owned();
            let b = t.st.photo_bytes(id.clone()).unwrap();
            (id, b)
        })
        .collect();
    (d, blobs)
}

/// Kiểm tài liệu sửa tay như khi nhập: (kết quả mở tệp mã hóa — thông báo chung, kết quả kiểm chi tiết).
fn crafted(doc: &Value, blobs: &[(String, Vec<u8>)]) -> Value {
    let photos: Vec<lvhv_backup::PhotoBlob> = blobs
        .iter()
        .map(|(id, b)| lvhv_backup::PhotoBlob {
            id: lvhv_backup::uuid_from_str(id).unwrap(),
            bytes: b.clone(),
        })
        .collect();
    let json = canonical_json(doc);
    let detail = store_outcome(&lvhv_store::verify_contents(&lvhv_backup::Contents {
        json: json.clone(),
        photos: photos.clone(),
    }));
    let file =
        lvhv_backup::export_with_fixed_rng(TP, &json, &photos, [0x11; 16], [0x22; 24]).unwrap();
    let open = match lvhv_ffi::open_backup(file, TP.into()) {
        Ok(_) => "ok".to_owned(),
        Err(e) => ffi_outcome(&e),
    };
    json!({ "open_backup": open, "verify": detail })
}

fn cmd_scenario(a: &[String]) -> Out {
    let [name, work] = a else {
        return usage("cần tray_photo <thư-mục-làm-việc>");
    };
    if name != "tray_photo" {
        return usage(&format!("kịch bản lạ: {name}"));
    }
    let work = PathBuf::from(work);
    let mut out = Map::new();
    let example = std::fs::read(tv().join("example-v1.lvhv")).unwrap();
    let example_doc: Value =
        serde_json::from_slice(&std::fs::read(tv().join("example-v1.json")).unwrap()).unwrap();
    let strip = |mut d: Value| {
        let o = d.as_object_mut().unwrap();
        o.remove("app");
        o.remove("exported_at");
        o.remove("settings");
        d
    };

    // (e) R14: tệp sao lưu của bản trước (không có trường) mở được, không đổi gì; tài liệu không có khóa mới.
    let t = fresh(&work.join("a"));
    let r = tray_import(&t, &example, lvhv_ffi::BackupImportMode::Replace);
    let got = strip(t.doc());
    let unchanged = canonical_json(&got["persons"]) == canonical_json(&example_doc["persons"])
        && canonical_json(&got["photos"]) == canonical_json(&example_doc["photos"])
        && canonical_json(&got["events"]) == canonical_json(&example_doc["events"]);
    out.insert(
        "e_old_backup".into(),
        json!({ "import": r, "unchanged": unchanged, "has_tray_key": canonical_json(&got).contains("tray_photo_id"),
                "tray_d1": t.tray_of(D1) }),
    );

    // (c) phía lõi: ảnh còn siêu dữ liệu, quá 1 MiB → từ chối; ảnh hợp lệ → nhận.
    let exif = t.add(TX, "media/jpeg-exif-gps.jpg", 768, 1024);
    let mut big = std::fs::read(tv().join("media/jpeg-ok.jpg")).unwrap();
    big.resize(1_048_577, 0);
    let too_big = outcome(
        &t.st
            .add_photo(TX.into(), "image/jpeg".into(), 768, 1024, big),
    );
    let add1 = t.add(T1, "r6v2/photo-00.jpg", 300, 400);
    out.insert(
        "c_add_photo".into(),
        json!({ "exif": exif, "over_1mib": too_big, "ok": add1 }),
    );

    // Gắn ảnh mâm lễ; V3: ảnh không tồn tại, dùng chung giữa hai người, trùng ảnh chân dung → từ chối.
    let set = t.with_tray(D1, json!(T1));
    let mut l3 = t.person(L3);
    l3["photo_id"] = json!(T1);
    let v3 = json!({
        "dangling": t.with_tray(D2, json!("0b8e4f52-3c1a-4d2e-9f60-0000000000ee")),
        "shared_tray": t.with_tray(D2, json!(T1)),
        "tray_is_other_portrait": t.with_tray(D2, json!(PORTRAIT)),
        "tray_is_own_portrait": t.with_tray(D1, json!(PORTRAIT)),
        "portrait_is_other_tray": t.save(&l3),
    });
    out.insert(
        "set".into(),
        json!({ "save": set, "tray_d1": t.tray_of(D1), "photos": t.photos(), "v3": v3 }),
    );

    // (d) xuất rồi nhập (Thay toàn bộ; Gộp vào kho trống; Gộp vào kho có bản cũ của hồ sơ): ảnh và liên kết giữ nguyên.
    let file =
        t.st.export_backup(TP.into(), "0.2.0".into(), "android".into())
            .unwrap();
    let sha1 = t.photo_sha(T1);
    let mut d = Map::new();
    for (k, mode, seed) in [
        ("replace", lvhv_ffi::BackupImportMode::Replace, false),
        ("merge_empty", lvhv_ffi::BackupImportMode::Merge, false),
        ("merge_over_old", lvhv_ffi::BackupImportMode::Merge, true),
    ] {
        let u = fresh(&work.join(format!("d-{k}")));
        if seed {
            tray_import(&u, &example, lvhv_ffi::BackupImportMode::Replace);
        }
        let r = tray_import(&u, &file, mode);
        d.insert(
            k.into(),
            json!({ "import": r, "tray_d1": u.tray_of(D1), "sha_equal": u.photo_sha(T1) == sha1 && !sha1.is_null(),
                    "photos": u.photos() }),
        );
    }
    // Tệp sửa tay: ảnh mâm lễ không có (V3), ảnh mâm lễ còn EXIF (B6), ảnh mâm lễ dùng chung (V3).
    let (doc, blobs) = contents_of(&t);
    let pos = |d: &Value, id: &str| {
        d["persons"]
            .as_array()
            .unwrap()
            .iter()
            .position(|p| p["id"] == id)
            .unwrap()
    };
    let mut dangling = doc.clone();
    dangling["photos"]
        .as_array_mut()
        .unwrap()
        .retain(|p| p["id"] != T1);
    let dangling_blobs: Vec<_> = blobs.iter().filter(|(i, _)| i != T1).cloned().collect();
    let exif_b = std::fs::read(tv().join("media/jpeg-exif-gps.jpg")).unwrap();
    let mut exif_doc = doc.clone();
    for ph in exif_doc["photos"].as_array_mut().unwrap() {
        if ph["id"] == T1 {
            ph["bytes"] = json!(exif_b.len());
            ph["sha256"] = json!(sha256_hex(&exif_b));
            ph["width"] = json!(768);
            ph["height"] = json!(1024);
        }
    }
    let exif_blobs: Vec<_> = blobs
        .iter()
        .map(|(i, b)| (i.clone(), if i == T1 { exif_b.clone() } else { b.clone() }))
        .collect();
    let mut shared = doc.clone();
    let l = pos(&shared, L3);
    shared["persons"][l]["photo_id"] = json!(T1);
    d.insert(
        "crafted".into(),
        json!({
            "dangling_tray": crafted(&dangling, &dangling_blobs),
            "tray_with_exif": crafted(&exif_doc, &exif_blobs),
            "shared_tray": crafted(&shared, &blobs),
            "control": crafted(&doc, &blobs),
        }),
    );
    out.insert("d_round_trip".into(), Value::Object(d));

    // (f) đổi ảnh, bỏ ảnh, đổi về đang sống, xóa hồ sơ → ảnh cũ bị xóa (bản ghi và tệp); 0 ảnh mồ côi.
    let mut f = Map::new();
    t.add(T2, "r6v2/photo-02.jpg", 302, 402);
    f.insert(
        "replace".into(),
        json!({ "save": t.with_tray(D1, json!(T2)), "photos": t.photos() }),
    );
    f.insert(
        "remove".into(),
        json!({ "save": t.with_tray(D1, Value::Null), "tray_d1": t.tray_of(D1), "photos": t.photos() }),
    );
    t.add(T3, "r6v2/photo-04.jpg", 304, 404);
    let s3 = t.with_tray(D1, json!(T3));
    let mut p = t.person(D1);
    p["kind"] = json!("living");
    p["death"] = Value::Null;
    p["memorial"] = Value::Null;
    p["reminders"]["gio"] = Value::Null;
    p["noi_an_tang"] = json!("");
    f.insert(
        "to_living".into(),
        json!({ "set": s3, "save": t.save(&p), "tray_d1": t.tray_of(D1), "photos": t.photos() }),
    );
    t.add(T4, "r6v2/photo-06.jpg", 306, 406);
    let s4 = t.with_tray(D2, json!(T4));
    f.insert(
        "delete_person".into(),
        json!({ "set": s4, "delete": outcome(&t.st.delete_person(D2.into())), "photos": t.photos() }),
    );
    out.insert("f_cleanup".into(), Value::Object(f));
    let _ = std::fs::remove_dir_all(&work);
    Ok((
        json!({ "command": "scenario", "scenario": "tray_photo", "ok": true, "result": Value::Object(out) }),
        true,
    ))
}
