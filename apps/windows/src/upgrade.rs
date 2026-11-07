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

/// Mọi id ảnh chân dung mà hồ sơ trỏ tới.
fn photo_ids(persons: &Value) -> Vec<String> {
    let mut ids: Vec<String> = persons
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| p["photo_id"].as_str().map(str::to_string))
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

/// Mọi `file_id` tệp đính kèm mà hồ sơ trỏ tới (01 §3.8a).
fn attachment_ids(persons: &Value) -> Vec<String> {
    let mut ids: Vec<String> = persons
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|p| p["death"]["notice"]["attachments"].as_array().cloned().unwrap_or_default())
        .filter_map(|a| a["file_id"].as_str().map(str::to_string))
        .collect();
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
    let mut atts = serde_json::Map::new();
    for id in attachment_ids(&persons) {
        let b = store.attachment_bytes(id.clone()).map_err(|x| format!("tệp đính kèm {id}: {x}"))?;
        atts.insert(id, json!(hex(&b)));
    }
    Ok(json!({
        "attachments": atts,
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

/// Trạng thái kho dạng tài liệu (như `state_doc` của store/tests/released.rs): `dataset_json` bỏ `app`, `exported_at`.
pub fn state_doc(store: &LvhvStore) -> Result<Value, String> {
    let mut d = jv(&store.dataset_json().map_err(|e| e.to_string())?);
    let o = d.as_object_mut().ok_or("dataset_json không phải đối tượng")?;
    o.remove("app");
    o.remove("exported_at");
    Ok(d)
}

/// Mật khẩu của tệp sao lưu trong bộ mẫu phát hành (ghi cả trong `manifest.json` của bộ mẫu).
pub const RELEASE_FIXTURE_PASSWORD: &str = "mật khẩu bộ mẫu Windows";

/// Trạng thái của một kho trống sau khi khôi phục (Thay toàn bộ) tệp sao lưu `bytes`.
fn restored_state(bytes: Vec<u8>, password: &str, work: &Path) -> Result<Value, String> {
    let _ = std::fs::remove_dir_all(work);
    let b = lvhv_ffi::open_backup(bytes, password.into()).map_err(|e| format!("mở tệp sao lưu: {e}"))?;
    let s = LvhvStore::open(work.to_string_lossy().into_owned()).map_err(|e| e.to_string())?;
    s.apply_import(b, lvhv_ffi::BackupImportMode::Replace).map_err(|e| format!("khôi phục: {e}"))?;
    let st = state_doc(&s);
    drop(s);
    let _ = std::fs::remove_dir_all(work);
    st
}

/// R14 / 04 B5 quy tắc 5–6 — `--dong-bang-ban-phat-hanh <thư mục ra> <tệp kỳ vọng>`: tạo bộ mẫu đóng băng của **chính bản
/// này** (chạy một lần trước khi phát hành, bằng mã của bản đó): `store/` (kho SQLite + ảnh + tệp đính kèm do lõi ghi),
/// `backup.lvhv` (tạo như nút "Tạo tệp sao lưu…"), `store-snapshot.json` (`dataset_json`), và tệp kỳ vọng ngoài bộ mẫu
/// (`state` của kho, `backup_state` sau khi khôi phục tệp sao lưu vào kho trống). `manifest.json` (sha256) ghi sau.
/// Dữ liệu: dữ liệu mẫu của R14 (a) (5 hồ sơ: âm lịch, tháng nhuận, người đang sống có sinh nhật âm và sự kiện, người
/// mất có thông tin báo tử + ảnh tin báo tử + PDF giấy chứng tử, ảnh chân dung) + ảnh mâm lễ của gia đình + cài đặt
/// khác mặc định.
pub fn freeze(out: &Path, expected: &Path) -> Result<String, String> {
    if out.exists() {
        return Err(format!("{} đã có — bộ mẫu đóng băng không bao giờ bị ghi đè", out.display()));
    }
    let dir = out.join("store");
    let store = LvhvStore::open(dir.to_string_lossy().into_owned()).map_err(|e| e.to_string())?;
    let (a, _, _) = shots::seed_store(&store);
    let tray = crate::photo::reencode_with(&shots::demo_tray(), crate::people::TRAY_SIDE, crate::photo::MAX_STORED)?;
    let tid = lvhv_ffi::new_uuid().map_err(|e| e.to_string())?;
    store
        .add_photo(tid.clone(), "image/jpeg".into(), tray.width, tray.height, tray.bytes)
        .map_err(|e| e.to_string())?;
    let mut pv = jv(&store.person_json(a).map_err(|e| e.to_string())?);
    pv["memorial"]["tray_photo_id"] = json!(tid);
    store.save_person_json(pv.to_string()).map_err(|e| e.to_string())?;
    let mut s = jv(&store.settings_json().map_err(|e| e.to_string())?);
    s["default_offsets_days"] = json!([0, 1, 7]);
    s["default_time_local"] = json!("06:30");
    s["backup_nag_days"] = json!(60);
    store.put_settings_json(s.to_string()).map_err(|e| e.to_string())?;
    let backup = store
        .export_backup(RELEASE_FIXTURE_PASSWORD.into(), update::VERSION.into(), "windows".into())
        .map_err(|e| e.to_string())?;
    let state = state_doc(&store)?;
    let snapshot = jv(&store.dataset_json().map_err(|e| e.to_string())?);
    // Đọc user_version khi kho còn mở: mở riêng sau khi đóng sẽ tạo lại -wal/-shm trong bộ mẫu.
    let sv = schema_version(&dir);
    drop(store);
    for e in std::fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        let n = e.file_name().to_string_lossy().into_owned();
        if n.ends_with("-wal") || n.ends_with("-shm") || n.ends_with("-journal") {
            return Err(format!("kho chưa đóng hẳn: còn {n}"));
        }
    }
    std::fs::write(out.join("backup.lvhv"), &backup).map_err(|e| e.to_string())?;
    std::fs::write(out.join("store-snapshot.json"), serde_json::to_string_pretty(&snapshot).unwrap() + "\n")
        .map_err(|e| e.to_string())?;
    let backup_state = restored_state(backup, RELEASE_FIXTURE_PASSWORD, &out.join("..").join(".khoi-phuc-tam"))?;
    if backup_state != state {
        return Err("khôi phục tệp sao lưu vào kho trống không ra đúng trạng thái kho".into());
    }
    let exp = json!({
        "fixture": format!("{}/windows", update::VERSION),
        "schema_version": sv,
        "state": state,
        "backup_state": backup_state,
    });
    std::fs::write(expected, serde_json::to_string_pretty(&exp).unwrap() + "\n").map_err(|e| e.to_string())?;
    Ok(format!(
        "Đã tạo bộ mẫu {} (lược đồ {}): {} hồ sơ, {} sự kiện, {} ảnh, {} tệp đính kèm",
        out.display(),
        sv,
        exp["state"]["persons"].as_array().map_or(0, Vec::len),
        exp["state"]["events"].as_array().map_or(0, Vec::len),
        exp["state"]["photos"].as_array().map_or(0, Vec::len),
        exp["state"]["attachments"].as_array().map_or(0, Vec::len),
    ))
}

/// R14 (d) — `--kiem-mau-phat-hanh <bộ mẫu> <expected.json>`: thư mục dữ liệu đang chứa bản chép `store/` của bộ mẫu
/// một bản Windows đã phát hành, và `LichViet.exe` đã cài (bản đang kiểm) đã mở nó. Kiểm: sha256 từng tệp của bộ mẫu
/// đúng `manifest.json` (không ai sửa); kho mở được, không ở chế độ chỉ đọc; trạng thái trùng `state` của tệp kỳ vọng
/// (khi lõi đổi lược đồ thì cập nhật tệp kỳ vọng có duyệt, như 0.2-dev-before-attachments); tệp `backup.lvhv` của bộ
/// mẫu khôi phục vào kho trống ra đúng `backup_state`; đặt lại được thông báo.
pub fn check_release_fixture(dir: &Path, fixture: &Path, expected: &Path, t: &dyn toasts::Toasts, now: i64) -> Result<String, String> {
    let man = jv(&std::fs::read_to_string(fixture.join("manifest.json")).map_err(|e| format!("đọc manifest.json: {e}"))?);
    let files = man["files"].as_object().ok_or("manifest.json thiếu files")?;
    for (f, sha) in files {
        let got = hex(&std::fs::read(fixture.join(f)).map_err(|e| format!("đọc {f}: {e}"))?);
        if Some(got.as_str()) != sha.as_str() {
            return Err(format!("{f} của bộ mẫu đã bị sửa (sha256 khác manifest.json)"));
        }
    }
    let want = jv(&std::fs::read_to_string(expected).map_err(|e| format!("đọc tệp kỳ vọng: {e}"))?);
    let before = schema_version(dir);
    let store = LvhvStore::open(dir.to_string_lossy().into_owned()).map_err(|e| format!("KHÔNG mở được kho: {e}"))?;
    let ms = store.migration_status();
    if !matches!(ms.state.as_str(), "ok" | "migrated") {
        return Err(format!("trạng thái chuyển đổi {}: {:?}", ms.state, ms.message));
    }
    let got = state_doc(&store)?;
    if got != want["state"] {
        let keys: Vec<String> = want["state"].as_object().map(|o| o.keys().cloned().collect()).unwrap_or_default();
        let d: Vec<String> = keys.into_iter().filter(|k| got[k] != want["state"][k]).collect();
        return Err(format!("trạng thái kho KHÁC tệp kỳ vọng ở: {}", d.join(", ")));
    }
    let pw = man["password"].as_str().ok_or("manifest.json thiếu password")?;
    let bytes = std::fs::read(fixture.join("backup.lvhv")).map_err(|e| e.to_string())?;
    let work = std::env::temp_dir().join(format!("lvhv-kiem-mau-phat-hanh-{}", std::process::id()));
    if restored_state(bytes, pw, &work)? != want["backup_state"] {
        return Err("khôi phục backup.lvhv của bộ mẫu vào kho trống KHÁC tệp kỳ vọng".into());
    }
    let r = toasts::sync(t, &store, now, platform::device_zone());
    if !matches!(r, toasts::SyncResult::Scheduled { .. }) {
        return Err(format!("chưa đặt lại được thông báo: {r:?}"));
    }
    Ok(format!(
        "R14 (d) bộ mẫu {}: ĐẠT. sha256 {} tệp khớp manifest; lược đồ {before} → {} (trạng thái khi kiểm: {}); trạng thái trùng tệp kỳ vọng ({} hồ sơ, {} sự kiện, {} ảnh, {} tệp đính kèm); khôi phục backup.lvhv vào kho trống khớp; đặt lại thông báo: {r:?}",
        man["fixture"].as_str().unwrap_or("?"),
        files.len(),
        schema_version(dir),
        ms.state,
        got["persons"].as_array().map_or(0, Vec::len),
        got["events"].as_array().map_or(0, Vec::len),
        got["photos"].as_array().map_or(0, Vec::len),
        got["attachments"].as_array().map_or(0, Vec::len),
    ))
}

/// R14 (c) — `--kiem-mau-dong-bang <expected.json>`: thư mục dữ liệu đang chứa kho của bộ mẫu đóng băng
/// `0.2-dev-before-attachments` (do bản dựng 0.2 trước YC-2026-07 tạo); mở bằng bản này → lõi chuyển 1 → 2; trạng thái
/// phải trùng `expected/<bộ mẫu>.json` (cùng tệp mà bản Rust của lõi và bản PWA phải ra), rồi đặt lại thông báo.
pub fn check_frozen(dir: &Path, expected: &Path, t: &dyn toasts::Toasts, now: i64) -> Result<String, String> {
    let want = jv(&std::fs::read_to_string(expected).map_err(|e| format!("đọc tệp kỳ vọng: {e}"))?);
    let before = schema_version(dir);
    let store = LvhvStore::open(dir.to_string_lossy().into_owned()).map_err(|e| format!("KHÔNG mở được kho: {e}"))?;
    // Trên CI, LichViet.exe đã cài mở kho trước (chuyển 1 → 2) rồi công cụ này mới chạy: trạng thái là "ok", bằng chứng
    // chuyển đổi là phiên bản 2 + bản sao trước chuyển đổi. Chạy trực tiếp trên kho phiên bản 1 thì là "migrated".
    let ms = store.migration_status();
    let bak = dir.join("lvhv.sqlite.pre-v2.bak");
    let ok = match ms.state.as_str() {
        "migrated" => true,
        "ok" => bak.exists() && schema_version(dir) == 2,
        _ => false,
    };
    if !ok {
        return Err(format!("trạng thái chuyển đổi {} (kỳ vọng đã chuyển sang phiên bản 2): {:?}", ms.state, ms.message));
    }
    let got = state_doc(&store)?;
    if got != want["state"] {
        let mut d = vec![];
        for k in want["state"].as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default() {
            if got[&k] != want["state"][&k] {
                d.push(k);
            }
        }
        return Err(format!("trạng thái sau chuyển đổi KHÁC tệp kỳ vọng ở: {}", d.join(", ")));
    }
    let r = toasts::sync(t, &store, now, platform::device_zone());
    if !matches!(r, toasts::SyncResult::Scheduled { .. }) {
        return Err(format!("chưa đặt lại được thông báo: {r:?}"));
    }
    Ok(format!(
        "R14 (c) bộ mẫu đóng băng: ĐẠT. Lược đồ {before} → {} (trạng thái khi kiểm: {}); trạng thái trùng tệp kỳ vọng ({} hồ sơ, {} tệp đính kèm, {} ảnh); bản sao trước chuyển đổi: {}; đặt lại thông báo: {r:?}",
        schema_version(dir),
        ms.state,
        got["persons"].as_array().map_or(0, Vec::len),
        got["attachments"].as_array().map_or(0, Vec::len),
        got["photos"].as_array().map_or(0, Vec::len),
        if bak.exists() { "lvhv.sqlite.pre-v2.bak" } else { "(không có)" }
    ))
}

/// Nội dung logic của một tệp SQLite (user_version + mọi dòng) — byte phần đầu tệp có thể đổi khi đổi chế độ nhật ký.
fn db_content(path: &Path) -> Result<Vec<String>, String> {
    let c = rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| e.to_string())?;
    let mut out = vec![format!(
        "user_version={}",
        c.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0)).map_err(|e| e.to_string())?
    )];
    for t in ["settings", "photo", "person", "event_type", "event", "event_person"] {
        let mut st = c.prepare(&format!("SELECT * FROM {t} ORDER BY 1, 2")).map_err(|e| e.to_string())?;
        let n = st.column_count();
        let rows = st
            .query_map([], |r| {
                let mut v = vec![];
                for i in 0..n {
                    v.push(format!("{:?}", r.get_ref(i)?));
                }
                Ok(v.join("|"))
            })
            .map_err(|e| e.to_string())?;
        for r in rows {
            out.push(format!("{t}:{}", r.map_err(|e| e.to_string())?));
        }
    }
    Ok(out)
}

pub(crate) fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)?.flatten() {
        let t = to.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_dir(&e.path(), &t)?;
        } else {
            std::fs::copy(e.path(), t)?;
        }
    }
    Ok(())
}

/// R14 ca lỗi — `--kiem-chi-doc <kho của bộ mẫu>`: chép kho phiên bản 1 của bộ mẫu vào một thư mục tạm, thêm một
/// trigger chặn ghi bảng `meta` để bước chuyển 1 → 2 lỗi thật ở câu SQL cuối giao dịch; dữ liệu cũ vẫn hợp lệ. Mở bằng bản này: lõi
/// phải quay lui, giữ bản sao dữ liệu cũ, mở chỉ đọc (`migration_status = read_only`), từ chối ghi, và **vẫn xuất được
/// sao lưu** (mở lại được bằng mật khẩu). `OpenOptions::force_migration_failure` của lõi không có qua FFI, nên dùng
/// lỗi thật này; kiểm thử Rust của ứng dụng còn chạy thêm ca `force_migration_failure` trên chính bộ mẫu.
pub fn check_read_only(fixture_store: &Path, work: &Path) -> Result<String, String> {
    let _ = std::fs::remove_dir_all(work);
    copy_dir(fixture_store, work).map_err(|e| format!("chép bộ mẫu: {e}"))?;
    let db = work.join("lvhv.sqlite");
    {
        let c = rusqlite::Connection::open(&db).map_err(|e| e.to_string())?;
        c.execute("CREATE TRIGGER kiem_dinh_loi_chuyen_doi BEFORE INSERT ON meta BEGIN SELECT RAISE(ABORT, 'kiểm định: lỗi chuyển đổi'); END", []).map_err(|e| e.to_string())?;
    }
    let before = db_content(&db)?;
    let store = LvhvStore::open(work.to_string_lossy().into_owned()).map_err(|e| format!("không mở được: {e}"))?;
    let ms = store.migration_status();
    if ms.state != "read_only" {
        return Err(format!("trạng thái {} (kỳ vọng read_only)", ms.state));
    }
    if db_content(&db)? != before {
        return Err("kho cũ đã bị đổi sau khi chuyển đổi lỗi".into());
    }
    let bak = ms.backup_path.clone().ok_or("không có bản sao trước chuyển đổi")?;
    if db_content(Path::new(&bak))? != before {
        return Err("bản sao trước chuyển đổi khác kho cũ".into());
    }
    if store.put_settings_json(store.settings_json().map_err(|e| e.to_string())?).is_ok() {
        return Err("kho chỉ đọc vẫn nhận ghi".into());
    }
    let pw = "mật khẩu kiểm định chỉ đọc";
    let bytes = store
        .export_backup(pw.into(), crate::update::VERSION.into(), "windows".into())
        .map_err(|e| format!("không xuất được sao lưu từ kho chỉ đọc: {e}"))?;
    let b = lvhv_ffi::open_backup(bytes, pw.into()).map_err(|e| format!("tệp sao lưu xuất ra không mở được: {e}"))?;
    let pv = b.preview();
    let msg = format!(
        "R14 ca lỗi: ĐẠT. Chuyển đổi lỗi → read_only; kho cũ giữ nguyên ({} dòng); bản sao {}; ghi bị từ chối; xuất sao lưu được ({} người). Thông báo: {}",
        before.len(),
        Path::new(&bak).file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default(),
        pv.persons,
        ms.message.unwrap_or_default()
    );
    drop(store);
    let _ = std::fs::remove_dir_all(work);
    Ok(msg)
}

/// So hai ảnh chụp, trả danh sách khác biệt (rỗng = giống hệt).
pub fn diff(want: &Value, got: &Value) -> Vec<String> {
    let mut out = vec![];
    for k in ["persons", "events", "event_types", "settings", "lock_screen_names", "photos", "attachments", "de_sau"] {
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
        assert_eq!(a["attachments"].as_object().unwrap().len(), 2, "ảnh + PDF đính kèm");
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
        let s = LvhvStore::open(d.to_string_lossy().into_owned()).unwrap();
        let ms = s.migration_status();
        assert_eq!(ms.state, "read_only", "lõi quay lui và mở chỉ đọc");
        assert!(crate::people::read_only_banner(&ms).text().contains("Tạo tệp sao lưu"));
        drop(s);
        assert_eq!(rows(&d), (n, 0), "không mất hồ sơ, không đổi phiên bản lược đồ");
        // Kho của bản mới hơn: không mở, không sửa.
        rusqlite::Connection::open(d.join("lvhv.sqlite")).unwrap().pragma_update(None, "user_version", 99).unwrap();
        let e = LvhvStore::open(d.to_string_lossy().into_owned()).err().expect("phải lỗi");
        assert_eq!(rows(&d), (n, 99));
        assert!(crate::people::store_open_banner(&e).text().contains("bản mới hơn"));
        let _ = std::fs::remove_dir_all(&d);
    }

    fn fixture() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/v0.2/schema/test-vectors/released/0.2-dev-before-attachments")
    }

    fn expected() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/v0.2/schema/test-vectors/released/expected/0.2-dev-before-attachments.json")
    }

    /// R14 (c): kho thật của bản dựng 0.2 trước tệp đính kèm → bản này chuyển 1 → 2, trùng tệp kỳ vọng.
    #[test]
    fn frozen_fixture_migrates_to_expected() {
        let d = tmp("frozen");
        copy_dir(&fixture().join("store"), &d).unwrap();
        let t = toasts::FakeToasts::default();
        let ok = check_frozen(&d, &expected(), &t, platform::now_unix()).unwrap();
        assert!(ok.contains("ĐẠT"), "{ok}");
        // Lần mở sau: không chuyển nữa.
        let s = LvhvStore::open(d.to_string_lossy().into_owned()).unwrap();
        assert_eq!(s.migration_status().state, "ok");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Ca lỗi qua FFI (lỗi thật) và ca `force_migration_failure` của lõi trên chính bộ mẫu.
    #[test]
    fn migration_failure_is_read_only_and_exportable() {
        let w = tmp("ro");
        let ok = check_read_only(&fixture().join("store"), &w).unwrap();
        assert!(ok.contains("ĐẠT"), "{ok}");
        let d = tmp("forced");
        copy_dir(&fixture().join("store"), &d).unwrap();
        let before = db_content(&d.join("lvhv.sqlite")).unwrap();
        let s = lvhv_store::Store::open_with(&d, lvhv_store::OpenOptions { force_migration_failure: true, ..Default::default() }).unwrap();
        assert_eq!(s.migration_status().state, "read_only");
        assert_eq!(db_content(&d.join("lvhv.sqlite")).unwrap(), before);
        drop(s);
        let s = LvhvStore::open(d.to_string_lossy().into_owned()).unwrap();
        assert_eq!(s.migration_status().state, "migrated", "mở lại bình thường thì chuyển được");
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
