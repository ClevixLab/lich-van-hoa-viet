//! R14 (04 Bổ sung B5): nâng cấp tại chỗ không mất dữ liệu.
//!
//! Hai chế độ dòng lệnh dùng trong CI (`.github/workflows/windows.yml`, bước "R14"):
//! - `--tao-du-lieu-mau <ảnh-chụp.json>`: tạo dữ liệu Người thân mẫu trong **thư mục dữ liệu thật**
//!   (`%LOCALAPPDATA%\ClevixLab\LichVanHoaViet`), rồi ghi ảnh chụp kỳ vọng (hồ sơ, sự kiện, loại sự kiện, cài đặt,
//!   băm ảnh, "Để sau") ra tệp. Từ chối nếu thư mục đã có kho — không bao giờ ghi đè dữ liệu của ai.
//! - `--kiem-nang-cap <ảnh-chụp.json>`: chạy sau khi cài MSI mới đè lên; mở kho (chạy chuyển đổi của lõi nếu cần),
//!   so từng phần với ảnh chụp, kiểm "Để sau" của bản 0.1.x, rồi đặt lại thông báo của Windows như lúc mở ứng dụng.
//!
//! Không có mạng. Ảnh chụp không chứa byte ảnh, chỉ băm SHA-256.

use std::path::Path;

use lvhv_ffi::LvhvStore;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::{platform, shots, toasts, update};

fn jv(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or(Value::Null)
}

fn hex(b: &[u8]) -> String {
    Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect()
}

/// Mọi id ảnh mà hồ sơ trỏ tới (chân dung + ảnh giấy báo tử).
fn photo_ids(persons: &Value) -> Vec<String> {
    let mut ids = vec![];
    for p in persons.as_array().into_iter().flatten() {
        if let Some(id) = p["photo_id"].as_str() {
            ids.push(id.to_string());
        }
        for c in p["death"]["notice"]["certificate_photo_ids"].as_array().into_iter().flatten() {
            if let Some(id) = c.as_str() {
                ids.push(id.to_string());
            }
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

/// Ảnh chụp toàn bộ dữ liệu của kho, đủ để so "không mất, không đổi nghĩa".
pub fn dump(store: &LvhvStore) -> Result<Value, String> {
    let e = |x: lvhv_ffi::LvhvDataError| x.to_string();
    let persons = jv(&store.persons_json().map_err(e)?);
    let mut photos = serde_json::Map::new();
    for id in photo_ids(&persons) {
        let b = store.photo_bytes(id.clone()).map_err(|x| format!("ảnh {id}: {x}"))?;
        photos.insert(id, json!(hex(&b)));
    }
    Ok(json!({
        "persons": persons,
        "events": jv(&store.events_json().map_err(e)?),
        "event_types": jv(&store.event_types_json().map_err(e)?),
        "settings": jv(&store.settings_json().map_err(e)?),
        "lock_screen_names": store.lock_screen_names().map_err(e)?,
        "photos": photos,
    }))
}

/// "Để sau" của thẻ nhắc cập nhật (bản 0.1.x ghi `%LOCALAPPDATA%\Clevix\LichVanHoaViet\nhac-cap-nhat.txt`).
fn de_sau() -> Value {
    update::snoozed_until().map(|d| json!(update::days_to_string(d))).unwrap_or(Value::Null)
}

/// `PRAGMA user_version` của kho (lõi chưa đưa qua FFI).
fn schema_version(dir: &Path) -> i64 {
    rusqlite::Connection::open_with_flags(dir.join("lvhv.sqlite"), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .and_then(|c| c.query_row("PRAGMA user_version", [], |r| r.get(0)))
        .unwrap_or(-1)
}

/// `--tao-du-lieu-mau`.
pub fn create(dir: &Path, snapshot: &Path) -> Result<String, String> {
    if dir.join("lvhv.sqlite").exists() {
        return Err(format!("{} đã có dữ liệu — không ghi đè", dir.display()));
    }
    let store = LvhvStore::open(dir.to_string_lossy().into_owned()).map_err(|e| e.to_string())?;
    shots::seed_store(&store);
    // Cài đặt khác mặc định, để thấy được nếu bị đặt lại.
    let mut s = jv(&store.settings_json().map_err(|e| e.to_string())?);
    s["default_offsets_days"] = json!([0, 1, 7]);
    s["default_time_local"] = json!("06:30");
    s["backup_nag_days"] = json!(60);
    store.put_settings_json(s.to_string()).map_err(|e| e.to_string())?;
    store.set_lock_screen_names(true).map_err(|e| e.to_string())?;
    let mut d = dump(&store)?;
    d["de_sau"] = de_sau();
    d["schema_version"] = json!(schema_version(dir));
    std::fs::write(snapshot, serde_json::to_string_pretty(&d).unwrap()).map_err(|e| e.to_string())?;
    Ok(format!(
        "Đã tạo dữ liệu mẫu ở {}: {} hồ sơ, {} sự kiện, {} ảnh; Để sau = {}",
        dir.display(),
        d["persons"].as_array().map_or(0, Vec::len),
        d["events"].as_array().map_or(0, Vec::len),
        d["photos"].as_object().map_or(0, |o| o.len()),
        d["de_sau"]
    ))
}

/// So hai ảnh chụp, trả danh sách khác biệt (rỗng = giống hệt).
pub fn diff(want: &Value, got: &Value) -> Vec<String> {
    let mut out = vec![];
    for k in ["persons", "events", "event_types", "settings", "lock_screen_names", "photos", "de_sau"] {
        if want[k] != got[k] {
            out.push(format!("{k}: kỳ vọng {} — thực tế {}", want[k], got[k]));
        }
    }
    out
}

/// `--kiem-nang-cap`.
pub fn check(dir: &Path, snapshot: &Path, t: &dyn toasts::Toasts, now: i64) -> Result<String, String> {
    let want = jv(&std::fs::read_to_string(snapshot).map_err(|e| format!("đọc ảnh chụp: {e}"))?);
    let mut log = vec![];
    let store = LvhvStore::open(dir.to_string_lossy().into_owned())
        .map_err(|e| format!("KHÔNG mở được kho sau nâng cấp: {e}"))?;
    let ver = schema_version(dir);
    log.push(format!("Lược đồ: {} → {ver}", want["schema_version"]));
    let mut got = dump(&store)?;
    got["de_sau"] = de_sau();
    let d = diff(&want, &got);
    if !d.is_empty() {
        return Err(format!("Dữ liệu KHÁC sau nâng cấp: {}", d.join("; ")));
    }
    log.push(format!(
        "Dữ liệu giữ nguyên: {} hồ sơ, {} sự kiện, {} loại sự kiện, {} ảnh (SHA-256 khớp), cài đặt, hiện tên = {}, Để sau = {}",
        got["persons"].as_array().map_or(0, Vec::len),
        got["events"].as_array().map_or(0, Vec::len),
        got["event_types"].as_array().map_or(0, Vec::len),
        got["photos"].as_object().map_or(0, |o| o.len()),
        got["lock_screen_names"],
        got["de_sau"]
    ));
    // Lịch nhắc được đặt lại như lúc mở ứng dụng (main.rs: `people::reschedule` khi có kho).
    let r = toasts::sync(t, &store, now, platform::device_zone());
    log.push(format!("Đặt lại thông báo: {r:?}"));
    let ok = matches!(r, toasts::SyncResult::Scheduled { pending, .. } if pending > 0);
    if !ok {
        return Err(format!("Chưa đặt lại được thông báo. {}", log.join("; ")));
    }
    Ok(format!("R14 Windows: ĐẠT. {}", log.join("; ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("lvhv-r14-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn rows(dir: &Path) -> (i64, i64) {
        let c = rusqlite::Connection::open(dir.join("lvhv.sqlite")).unwrap();
        let n: i64 = c.query_row("SELECT count(*) FROM person", [], |r| r.get(0)).unwrap();
        let v: i64 = c.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        (n, v)
    }

    /// Mở lại kho (như cài bản mới đè lên) giữ nguyên từng trường.
    #[test]
    fn reopen_keeps_everything() {
        let d = tmp("ok");
        let s = LvhvStore::open(d.to_string_lossy().into_owned()).unwrap();
        shots::seed_store(&s);
        let a = dump(&s).unwrap();
        drop(s);
        let s = LvhvStore::open(d.to_string_lossy().into_owned()).unwrap();
        let b = dump(&s).unwrap();
        assert!(diff(&a, &b).is_empty());
        assert_eq!(photo_ids(&a["persons"]).len(), a["photos"].as_object().unwrap().len());
        assert!(!a["photos"].as_object().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Ca chuyển đổi lỗi (B5 mục 3): lõi lùi giao dịch, không xóa gì; ứng dụng báo lỗi chứ không tạo kho mới.
    #[test]
    fn failed_migration_keeps_old_data() {
        let d = tmp("fail");
        let s = LvhvStore::open(d.to_string_lossy().into_owned()).unwrap();
        shots::seed_store(&s);
        drop(s);
        let (n, _) = rows(&d);
        assert!(n > 0);
        // Giả một kho "cũ" mà bước chuyển đổi 0 → 1 sẽ lỗi (bảng đã có): lõi phải lùi lại toàn bộ.
        rusqlite::Connection::open(d.join("lvhv.sqlite")).unwrap().pragma_update(None, "user_version", 0).unwrap();
        let e = LvhvStore::open(d.to_string_lossy().into_owned()).err().expect("phải lỗi");
        assert_eq!(rows(&d), (n, 0), "không mất hồ sơ, không đổi phiên bản lược đồ");
        let m = crate::people::store_open_message(&e);
        assert!(m.contains("vẫn còn nguyên"), "{m}");
        // Kho của bản mới hơn: không mở, không sửa.
        rusqlite::Connection::open(d.join("lvhv.sqlite")).unwrap().pragma_update(None, "user_version", 99).unwrap();
        let e = LvhvStore::open(d.to_string_lossy().into_owned()).err().expect("phải lỗi");
        assert_eq!(rows(&d), (n, 99));
        assert!(crate::people::store_open_message(&e).contains("bản mới hơn"));
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Kiểm sau nâng cấp với thông báo giả: dữ liệu khớp và lịch nhắc được đặt.
    #[test]
    fn check_passes_and_detects_changes() {
        let d = tmp("check");
        let snap = d.with_extension("json");
        create(&d, &snap).unwrap();
        assert!(create(&d, &snap).is_err(), "không ghi đè kho đã có");
        let t = toasts::FakeToasts::default();
        let now = platform::now_unix();
        let ok = check(&d, &snap, &t, now).unwrap();
        assert!(ok.starts_with("R14 Windows: ĐẠT"), "{ok}");
        // Đổi một trường → phải bị phát hiện.
        let s = LvhvStore::open(d.to_string_lossy().into_owned()).unwrap();
        s.set_lock_screen_names(false).unwrap();
        drop(s);
        let e = check(&d, &snap, &t, now).unwrap_err();
        assert!(e.contains("lock_screen_names"), "{e}");
        let _ = std::fs::remove_dir_all(&d);
        let _ = std::fs::remove_file(&snap);
    }
}
