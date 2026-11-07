//! # lvhv-store — kho dữ liệu SQLite của Lịch và Văn hóa Việt (v0.2)
//!
//! Cài đặt 01 §4.2 (SQLite), §6 (kiểm tra), §7 (phiên bản lược đồ) và phần nhập/xuất của 03 §2.4. Một thư
//! mục dữ liệu chứa `lvhv.sqlite` và thư mục `photos/` (byte ảnh; bảng `photo` chỉ giữ siêu dữ liệu).
//!
//! Mọi bản ghi đi vào kho đều qua [`validate`] (V1–V8). Các đối tượng nhúng (`birth`, `death`, `memorial`,
//! `reminders`, `date`, `reminder`) lưu dạng cột JSON (TEXT) — cùng một lớp tuần tự hóa với tệp sao lưu.
//! Tài liệu sao lưu (01 §5) là "ảnh chụp" toàn bộ kho ở dạng JSON chuẩn: khóa sắp theo thứ tự byte, mảng
//! sắp theo `id`, không khoảng trắng ([`canonical_json`]) — bản PWA sinh đúng từng byte như vậy.
#![forbid(unsafe_code)]

pub mod derive;
pub mod model;
pub mod validate;

use lvhv_backup::{BackupError, PhotoBlob};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use validate::ValidationError;

/// Phiên bản lược đồ dữ liệu mà bản này hiểu (01 §7).
pub const SCHEMA_VERSION: i64 = 1;
/// Tên tệp cơ sở dữ liệu.
pub const DB_FILE: &str = "lvhv.sqlite";
/// Thư mục ảnh (trong thư mục dữ liệu).
pub const PHOTO_DIR: &str = "photos";

/// Lỗi của kho.
#[derive(Debug)]
pub enum StoreError {
    /// Dữ liệu vi phạm V1–V8.
    Invalid(ValidationError),
    /// Lỗi nhập liệu khi tính trường dẫn xuất (thông báo tiếng Việt theo 01 §6.1).
    Input(String),
    /// Kho có `schema_version` lớn hơn bản này hiểu.
    NewerSchema(i64),
    /// Không tìm thấy bản ghi.
    NotFound,
    /// Lỗi tệp sao lưu (vùng chứa/mật mã).
    Backup(BackupError),
    /// Lỗi SQLite.
    Sql(rusqlite::Error),
    /// Lỗi tệp ảnh.
    Io(std::io::Error),
}

impl StoreError {
    /// Thông báo cho người dùng khi **nhập sao lưu** (03 §2.4: một thông báo chung cho mọi lỗi nội dung).
    pub fn import_message(&self) -> &'static str {
        match self {
            StoreError::Backup(e) => e.user_message(),
            StoreError::NewerSchema(_) => {
                "Tệp được tạo bởi phiên bản mới hơn — hãy cập nhật ứng dụng"
            }
            _ => "Sai mật khẩu hoặc tệp bị hỏng",
        }
    }
}

impl core::fmt::Display for StoreError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StoreError::Invalid(e) => write!(f, "dữ liệu không hợp lệ: {e}"),
            StoreError::Input(s) => f.write_str(s),
            StoreError::NewerSchema(v) => write!(f, "kho có schema_version {v} > {SCHEMA_VERSION}"),
            StoreError::NotFound => f.write_str("không tìm thấy"),
            StoreError::Backup(e) => write!(f, "{e}"),
            StoreError::Sql(e) => write!(f, "SQLite: {e}"),
            StoreError::Io(e) => write!(f, "tệp: {e}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError::Sql(e)
    }
}
impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e)
    }
}
impl From<ValidationError> for StoreError {
    fn from(e: ValidationError) -> Self {
        StoreError::Invalid(e)
    }
}
impl From<BackupError> for StoreError {
    fn from(e: BackupError) -> Self {
        StoreError::Backup(e)
    }
}

/// Kết quả hàm của kho.
pub type Result<T> = core::result::Result<T, StoreError>;

/// Thông tin ứng dụng ghi vào tệp sao lưu (`app`).
#[derive(Debug, Clone)]
pub struct AppInfo {
    /// Phiên bản ứng dụng, dạng X.Y.Z.
    pub version: String,
    /// `"android"` | `"ios"` | `"macos"` | `"windows"` (bản PWA dùng `"pwa"`).
    pub platform: String,
}

/// Cách nhập (01 §6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    /// Thay toàn bộ dữ liệu trên máy.
    Replace,
    /// Gộp theo `id`/`rev`/`updated_at` (mặc định — BK-S1).
    Merge,
}

/// Xem trước khi nhập (01 §6.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPreview {
    /// Số người.
    pub persons: usize,
    /// Số sự kiện.
    pub events: usize,
    /// Số ảnh.
    pub photos: usize,
    /// Ngày xuất (`exported_at`).
    pub exported_at: String,
    /// Nền tảng đã xuất.
    pub platform: String,
}

/// Tệp sao lưu đã giải mã và đã qua toàn bộ kiểm tra (01 §6.3) — chưa ghi gì vào kho.
pub struct VerifiedBackup {
    doc: Value,
    photos: HashMap<String, Vec<u8>>,
}

impl core::fmt::Debug for VerifiedBackup {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VerifiedBackup({:?})", self.preview())
    }
}

impl VerifiedBackup {
    /// Số liệu cho màn hình xem trước.
    pub fn preview(&self) -> ImportPreview {
        let n = |k: &str| self.doc[k].as_array().map_or(0, Vec::len);
        ImportPreview {
            persons: n("persons"),
            events: n("events"),
            photos: n("photos"),
            exported_at: self.doc["exported_at"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            platform: self.doc["app"]["platform"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
        }
    }
    /// Tài liệu JSON (đã chuẩn hóa số).
    pub fn document(&self) -> &Value {
        &self.doc
    }
}

/// JSON chuẩn: khóa sắp theo byte (serde_json dùng `BTreeMap`), không khoảng trắng.
pub fn canonical_json(v: &Value) -> String {
    // `serde_json::Map` mặc định là BTreeMap → khóa đã sắp; `to_string` không thêm khoảng trắng.
    serde_json::to_string(v).unwrap_or_default()
}

/// SHA-256 hex chữ thường.
pub fn sha256_hex(b: &[u8]) -> String {
    to_hex(&Sha256::digest(b))
}

fn to_hex(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 2);
    for x in b {
        s.push_str(&format!("{x:02x}"));
    }
    s
}

/// Thời điểm hiện tại, RFC 3339 UTC độ chính xác giây.
pub fn now_rfc3339() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    rfc3339_from_unix(secs)
}

/// Giây Unix → `YYYY-MM-DDTHH:MM:SSZ`.
pub fn rfc3339_from_unix(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // civil_from_days (H. Hinnant)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// `YYYY-MM-DDTHH:MM:SSZ` → giây Unix.
pub fn unix_from_rfc3339(s: &str) -> Option<i64> {
    if !validate::is_timestamp(s) {
        return None;
    }
    let n = |a: usize, b: usize| s[a..b].parse::<i64>().ok();
    let (y, m, d, hh, mm, ss) = (
        n(0, 4)?,
        n(5, 7)?,
        n(8, 10)?,
        n(11, 13)?,
        n(14, 16)?,
        n(17, 19)?,
    );
    let y2 = if m <= 2 { y - 1 } else { y };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(days * 86_400 + hh * 3600 + mm * 60 + ss)
}

/// Lời nhắc sao lưu (01 §3.9 `backup_nag_days`): đến hạn khi đã có thay đổi chưa sao lưu từ
/// `pending_since` và đã quá `nag_days` ngày. `nag_days = 0` → tắt.
pub fn backup_due(nag_days: i64, pending_since: Option<&str>, now: &str) -> bool {
    if nag_days <= 0 {
        return false;
    }
    match (
        pending_since.and_then(unix_from_rfc3339),
        unix_from_rfc3339(now),
    ) {
        (Some(p), Some(n)) => n - p >= nag_days * 86_400,
        _ => false,
    }
}

// ---------------------------------------------------------------------------------------------
// Lược đồ SQLite và chuyển đổi phiên bản
// ---------------------------------------------------------------------------------------------

/// Chuỗi chuyển đổi: phần tử `i` đưa kho từ `user_version = i` lên `i + 1`.
const MIGRATIONS: &[&str] = &[
    // 0 → 1: lược đồ ban đầu (01 §4.2).
    r#"
    CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL) STRICT;
    CREATE TABLE settings (id INTEGER PRIMARY KEY CHECK (id = 1), json TEXT NOT NULL) STRICT;
    CREATE TABLE photo (
        id TEXT PRIMARY KEY,
        json TEXT NOT NULL,
        sha256 TEXT NOT NULL
    ) STRICT;
    CREATE TABLE person (
        id TEXT PRIMARY KEY,
        kind TEXT NOT NULL CHECK (kind IN ('deceased', 'living')),
        photo_id TEXT REFERENCES photo(id) ON DELETE SET NULL,
        rev INTEGER NOT NULL CHECK (rev >= 1),
        updated_at TEXT NOT NULL,
        json TEXT NOT NULL
    ) STRICT;
    CREATE TABLE event_type (
        id TEXT PRIMARY KEY,
        name_key TEXT NOT NULL UNIQUE,
        rev INTEGER NOT NULL CHECK (rev >= 1),
        updated_at TEXT NOT NULL,
        json TEXT NOT NULL
    ) STRICT;
    CREATE TABLE event (
        id TEXT PRIMARY KEY,
        type_id TEXT NOT NULL REFERENCES event_type(id),
        rev INTEGER NOT NULL CHECK (rev >= 1),
        updated_at TEXT NOT NULL,
        json TEXT NOT NULL
    ) STRICT;
    CREATE TABLE event_person (
        event_id TEXT NOT NULL REFERENCES event(id) ON DELETE CASCADE,
        person_id TEXT NOT NULL REFERENCES person(id),
        pos INTEGER NOT NULL,
        PRIMARY KEY (event_id, person_id)
    ) STRICT;
    CREATE INDEX event_person_by_person ON event_person(person_id);
    "#,
];

/// Kho dữ liệu trên một thư mục.
#[derive(Debug)]
pub struct Store {
    conn: Connection,
    dir: PathBuf,
}

fn jtext(v: &Value) -> String {
    canonical_json(v)
}

fn parse_col(s: &str) -> Result<Value> {
    let mut v: Value =
        serde_json::from_str(s).map_err(|e| StoreError::Input(format!("cột JSON hỏng: {e}")))?;
    validate::normalize_numbers(&mut v);
    Ok(v)
}

impl Store {
    /// Mở (hoặc tạo) kho trong thư mục `dir`, chạy chuyển đổi phiên bản nếu cần (01 §7).
    pub fn open(dir: impl AsRef<Path>) -> Result<Store> {
        let dir = dir.as_ref().to_path_buf();
        std::fs::create_dir_all(dir.join(PHOTO_DIR))?;
        let mut conn = Connection::open(dir.join(DB_FILE))?;
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        let ver: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if ver > SCHEMA_VERSION {
            return Err(StoreError::NewerSchema(ver));
        }
        if ver < SCHEMA_VERSION {
            if ver > 0 {
                // 01 §7: bản sao nội bộ không mã hóa trước khi chuyển đổi (giữ 1 bản).
                let bak = dir.join(format!("{DB_FILE}.pre-v{SCHEMA_VERSION}.bak"));
                conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
                std::fs::copy(dir.join(DB_FILE), bak)?;
            }
            let tx = conn.transaction_with_behavior(TransactionBehavior::Exclusive)?;
            for m in &MIGRATIONS[ver as usize..] {
                tx.execute_batch(m)?;
            }
            tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            if ver == 0 {
                tx.execute(
                    "INSERT INTO meta(key, value) VALUES ('schema_version', ?1), ('created_with_app_version', ?2)",
                    params![SCHEMA_VERSION.to_string(), env!("CARGO_PKG_VERSION")],
                )?;
                tx.execute(
                    "INSERT INTO settings(id, json) VALUES (1, ?1)",
                    params![jtext(&default_settings())],
                )?;
                for t in default_event_types() {
                    insert_event_type(&tx, &t)?;
                }
            } else {
                tx.execute(
                    "UPDATE meta SET value = ?1 WHERE key = 'schema_version'",
                    params![SCHEMA_VERSION.to_string()],
                )?;
            }
            tx.commit()?;
        }
        Ok(Store { conn, dir })
    }

    /// `PRAGMA user_version`.
    pub fn schema_version(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))?)
    }

    fn meta_get(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT value FROM meta WHERE key = ?1", params![key], |r| {
                r.get(0)
            })
            .optional()?)
    }

    /// Thời điểm của thay đổi dữ liệu đầu tiên chưa được sao lưu (`None` = không có thay đổi chờ).
    pub fn backup_pending_since(&self) -> Result<Option<String>> {
        self.meta_get("backup_pending_since")
    }

    /// Tùy chọn "hiện tên trên màn hình khóa" (06 O-9; mặc định **không** hiện = `false`). Là tùy chọn cục bộ
    /// của thiết bị, lưu ở bảng `meta` (khóa `lock_screen_names`), không nằm trong `AppSettings` (lược đồ v1
    /// đóng, 01 §5) và không đi vào tệp sao lưu — như tùy chọn giao diện `lvhv.ui` của PWA.
    pub fn lock_screen_names(&self) -> Result<bool> {
        Ok(self.meta_get("lock_screen_names")?.as_deref() == Some("1"))
    }

    /// Ghi tùy chọn "hiện tên trên màn hình khóa" (O-9).
    pub fn set_lock_screen_names(&mut self, show: bool) -> Result<()> {
        self.conn.execute(
            "INSERT INTO meta(key, value) VALUES ('lock_screen_names', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![if show { "1" } else { "0" }],
        )?;
        Ok(())
    }

    /// Hook nhắc sao lưu: `true` khi nên nhắc người dùng xuất tệp sao lưu (01 §3.9).
    pub fn backup_reminder_due(&self, now: &str) -> Result<bool> {
        let s = self.settings()?;
        Ok(backup_due(
            s["backup_nag_days"].as_i64().unwrap_or(30),
            self.backup_pending_since()?.as_deref(),
            now,
        ))
    }

    fn mark_changed(tx: &rusqlite::Transaction<'_>, now: &str) -> Result<()> {
        tx.execute(
            "INSERT OR IGNORE INTO meta(key, value) VALUES ('backup_pending_since', ?1)",
            params![now],
        )?;
        Ok(())
    }

    // ---------------- đọc ----------------

    /// `AppSettings`.
    pub fn settings(&self) -> Result<Value> {
        let s: String = self
            .conn
            .query_row("SELECT json FROM settings WHERE id = 1", [], |r| r.get(0))?;
        parse_col(&s)
    }

    fn all(&self, table: &str) -> Result<Vec<Value>> {
        let mut st = self
            .conn
            .prepare(&format!("SELECT json FROM {table} ORDER BY id"))?;
        let rows = st.query_map([], |r| r.get::<_, String>(0))?;
        rows.map(|r| parse_col(&r?)).collect()
    }

    /// Mọi `Person`, sắp theo `id`.
    pub fn persons(&self) -> Result<Vec<Value>> {
        self.all("person")
    }
    /// Mọi `Event`, sắp theo `id`.
    pub fn events(&self) -> Result<Vec<Value>> {
        self.all("event")
    }
    /// Mọi `EventType`, sắp theo `id`.
    pub fn event_types(&self) -> Result<Vec<Value>> {
        self.all("event_type")
    }
    /// Siêu dữ liệu mọi `Photo`, sắp theo `id`.
    pub fn photos(&self) -> Result<Vec<Value>> {
        self.all("photo")
    }

    /// Một `Person`.
    pub fn person(&self, id: &str) -> Result<Value> {
        let s: Option<String> = self
            .conn
            .query_row("SELECT json FROM person WHERE id = ?1", params![id], |r| {
                r.get(0)
            })
            .optional()?;
        parse_col(&s.ok_or(StoreError::NotFound)?)
    }

    /// Byte của một ảnh (kiểm tra `sha256` khi đọc).
    pub fn photo_bytes(&self, id: &str) -> Result<Vec<u8>> {
        let sha: Option<String> = self
            .conn
            .query_row("SELECT sha256 FROM photo WHERE id = ?1", params![id], |r| {
                r.get(0)
            })
            .optional()?;
        let sha = sha.ok_or(StoreError::NotFound)?;
        let b = std::fs::read(self.photo_path(id))?;
        if sha256_hex(&b) != sha {
            return Err(StoreError::Invalid(ValidationError {
                code: "PHOTO",
                path: id.into(),
                msg: "sha256 không khớp".into(),
            }));
        }
        Ok(b)
    }

    fn photo_path(&self, id: &str) -> PathBuf {
        self.dir.join(PHOTO_DIR).join(format!("{id}.img"))
    }

    // ---------------- ghi ----------------

    /// Ghi (thêm/sửa) một `Person` đã đủ trường (kể cả trường dẫn xuất); kiểm tra V1–V8 trước khi ghi.
    pub fn put_person(&mut self, p: &Value) -> Result<()> {
        validate::person(p, "/person")?;
        let now = now_rfc3339();
        let tx = self.conn.transaction()?;
        check_refs_person(&tx, p)?;
        upsert_person(&tx, p)?;
        Self::mark_changed(&tx, &now)?;
        tx.commit()?;
        Ok(())
    }

    /// Ghi một `Event` (V1, V3, V4, V8).
    pub fn put_event(&mut self, e: &Value) -> Result<()> {
        validate::event(e, "/event")?;
        let now = now_rfc3339();
        let tx = self.conn.transaction()?;
        upsert_event(&tx, e).map_err(|err| match err {
            StoreError::Sql(rusqlite::Error::SqliteFailure(f, _))
                if f.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                StoreError::Invalid(ValidationError {
                    code: "V3",
                    path: "/event".into(),
                    msg: "tham chiếu không tồn tại".into(),
                })
            }
            other => other,
        })?;
        Self::mark_changed(&tx, &now)?;
        tx.commit()?;
        Ok(())
    }

    /// Ghi một `EventType` (V1, V7).
    pub fn put_event_type(&mut self, t: &Value) -> Result<()> {
        validate::event_type(t, "/event_type")?;
        let now = now_rfc3339();
        let tx = self.conn.transaction()?;
        let key = validate::name_key(t["ten"].as_str().unwrap_or_default());
        let clash: Option<String> = tx
            .query_row(
                "SELECT id FROM event_type WHERE name_key = ?1 AND id <> ?2",
                params![key, t["id"].as_str()],
                |r| r.get(0),
            )
            .optional()?;
        if clash.is_some() {
            return Err(StoreError::Invalid(ValidationError {
                code: "V7",
                path: "/event_type/ten".into(),
                msg: "tên trùng".into(),
            }));
        }
        upsert_event_type(&tx, t)?;
        Self::mark_changed(&tx, &now)?;
        tx.commit()?;
        Ok(())
    }

    /// Thêm ảnh (byte đã được nền tảng giải mã và mã hóa lại — 01 §3.8). Trả bản ghi `Photo`.
    pub fn add_photo(
        &mut self,
        id: &str,
        mime: &str,
        width: u32,
        height: u32,
        bytes: &[u8],
    ) -> Result<Value> {
        let meta = json!({
            "id": id, "mime": mime, "width": width, "height": height, "bytes": bytes.len(),
            "sha256": sha256_hex(bytes), "stripped": true, "created_at": now_rfc3339(),
        });
        validate::photo(&meta, "/photo")?;
        write_atomic(&self.photo_path(id), bytes)?;
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT OR REPLACE INTO photo(id, json, sha256) VALUES (?1, ?2, ?3)",
            params![id, jtext(&meta), meta["sha256"].as_str()],
        )?;
        Self::mark_changed(&tx, &now_rfc3339())?;
        tx.commit()?;
        Ok(meta)
    }

    /// Xóa một người: gỡ id khỏi mọi sự kiện (01 §3.4), xóa ảnh chân dung và ảnh giấy báo tử (01 §3.2a) của
    /// người đó nếu không hồ sơ nào khác còn dùng.
    pub fn delete_person(&mut self, id: &str) -> Result<()> {
        self.delete_person_with(id, false).map(|_| ())
    }

    /// Như [`Store::delete_person`]; khi `delete_orphan_events`, trong **cùng** giao dịch xóa luôn sự kiện mà
    /// người này là người duy nhất được gắn (sau khi gỡ thì `person_ids` rỗng). Sự kiện vốn không gắn ai không bị
    /// đụng tới. Trả id các sự kiện đã xóa (duyệt mã MA3N-24-08: ba ứng dụng không tự viết lại việc này).
    pub fn delete_person_with(
        &mut self,
        id: &str,
        delete_orphan_events: bool,
    ) -> Result<Vec<String>> {
        let now = now_rfc3339();
        let mut removed_events = Vec::new();
        let tx = self.conn.transaction()?;
        let pj: Option<String> = tx
            .query_row("SELECT json FROM person WHERE id = ?1", params![id], |r| {
                r.get(0)
            })
            .optional()?;
        let Some(pj) = pj else {
            return Err(StoreError::NotFound);
        };
        let refs = validate::person_photo_refs(&parse_col(&pj)?);
        let ev: Vec<String> = {
            let mut st = tx.prepare("SELECT event_id FROM event_person WHERE person_id = ?1")?;
            let r = st
                .query_map(params![id], |r| r.get(0))?
                .collect::<core::result::Result<_, _>>()?;
            r
        };
        for eid in ev {
            let s: String =
                tx.query_row("SELECT json FROM event WHERE id = ?1", params![eid], |r| {
                    r.get(0)
                })?;
            let mut e = parse_col(&s)?;
            if let Some(a) = e["person_ids"].as_array_mut() {
                a.retain(|x| x.as_str() != Some(id));
            }
            if delete_orphan_events && e["person_ids"].as_array().is_some_and(Vec::is_empty) {
                tx.execute("DELETE FROM event WHERE id = ?1", params![eid])?;
                removed_events.push(eid);
                continue;
            }
            e["updated_at"] = json!(now);
            e["rev"] = json!(e["rev"].as_i64().unwrap_or(1) + 1);
            upsert_event(&tx, &e)?;
        }
        tx.execute("DELETE FROM person WHERE id = ?1", params![id])?;
        // Ảnh còn được hồ sơ khác dùng thì giữ.
        let mut used: std::collections::HashSet<String> = Default::default();
        {
            let mut st = tx.prepare("SELECT json FROM person")?;
            let rows = st.query_map([], |r| r.get::<_, String>(0))?;
            for r in rows {
                used.extend(validate::person_photo_refs(&parse_col(&r?)?));
            }
        }
        let gone: Vec<String> = refs.into_iter().filter(|r| !used.contains(r)).collect();
        for pid in &gone {
            tx.execute("DELETE FROM photo WHERE id = ?1", params![pid])?;
        }
        Self::mark_changed(&tx, &now)?;
        tx.commit()?;
        for pid in gone {
            let _ = std::fs::remove_file(self.photo_path(&pid));
        }
        Ok(removed_events)
    }

    /// Xóa một sự kiện.
    pub fn delete_event(&mut self, id: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        if tx.execute("DELETE FROM event WHERE id = ?1", params![id])? == 0 {
            return Err(StoreError::NotFound);
        }
        Self::mark_changed(&tx, &now_rfc3339())?;
        tx.commit()?;
        Ok(())
    }

    /// Cập nhật `AppSettings` (V1, V8).
    pub fn put_settings(&mut self, s: &Value) -> Result<()> {
        validate::settings(s, "/settings")?;
        self.conn.execute(
            "UPDATE settings SET json = ?1 WHERE id = 1",
            params![jtext(s)],
        )?;
        Ok(())
    }

    // ---------------- ảnh chụp / sao lưu ----------------

    /// Toàn bộ kho dưới dạng tài liệu sao lưu (01 §5), chưa có byte ảnh.
    pub fn snapshot(&self, app: &AppInfo, exported_at: &str) -> Result<Value> {
        Ok(json!({
            "format": "lvhv-backup",
            "schema_version": SCHEMA_VERSION,
            "app": { "name": "Lịch và Văn hóa Việt", "version": app.version, "platform": app.platform },
            "exported_at": exported_at,
            "settings": self.settings()?,
            "event_types": self.event_types()?,
            "persons": self.persons()?,
            "events": self.events()?,
            "photos": self.photos()?,
        }))
    }

    /// Tài liệu + byte ảnh (theo thứ tự `photos`).
    pub fn snapshot_with_photos(
        &self,
        app: &AppInfo,
        exported_at: &str,
    ) -> Result<(Value, Vec<PhotoBlob>)> {
        let doc = self.snapshot(app, exported_at)?;
        let mut blobs = Vec::new();
        for ph in doc["photos"].as_array().into_iter().flatten() {
            let id = ph["id"].as_str().unwrap_or_default();
            let raw = lvhv_backup::uuid_from_str(id).ok_or(StoreError::NotFound)?;
            blobs.push(PhotoBlob {
                id: raw,
                bytes: self.photo_bytes(id)?,
            });
        }
        Ok((doc, blobs))
    }

    /// Băm SHA-256 của toàn bộ kho (JSON chuẩn không có `exported_at`/`app` + băm từng ảnh) — dùng để
    /// kiểm "kho dữ liệu không đổi" (R6 (a2), (b)) và "giống hệt" (R6 (a)).
    pub fn state_hash(&self) -> Result<String> {
        let mut doc = self.snapshot(
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
        let mut h = Sha256::new();
        h.update(canonical_json(&doc).as_bytes());
        for ph in doc["photos"].as_array().into_iter().flatten() {
            h.update(
                sha256_hex(&self.photo_bytes(ph["id"].as_str().unwrap_or_default())?).as_bytes(),
            );
        }
        Ok(to_hex(&h.finalize()))
    }

    /// Tạo byte tệp sao lưu mã hóa (03 §2.4). **Không** đổi `settings.last_backup_at` hay trạng thái "chờ sao
    /// lưu": ứng dụng gọi [`Store::mark_backup_saved`] chỉ khi hệ điều hành báo đã ghi tệp xong (duyệt mã
    /// MA3N-23-01 — hủy hộp "Lưu" hay ghi lỗi không được tính là đã sao lưu). Ghi nhớ thời điểm tạo
    /// (`meta.last_export_at`) để `mark_backup_saved` biết thay đổi nào đã nằm trong tệp. `last_backup_at` trong tệp là
    /// giá trị trước lần xuất này.
    pub fn export_backup(&mut self, password: &str, app: &AppInfo) -> Result<Vec<u8>> {
        let now = now_rfc3339();
        let (doc, blobs) = self.snapshot_with_photos(app, &now)?;
        let file = lvhv_backup::export(password, &canonical_json(&doc), &blobs)?;
        self.conn.execute(
            "INSERT INTO meta(key, value) VALUES ('last_export_at', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![now],
        )?;
        Ok(file)
    }

    /// Ghi nhận tệp sao lưu vừa tạo bằng [`Store::export_backup`] **đã được lưu** (03 §2.4, MA3N-23-01): đặt
    /// `settings.last_backup_at = saved_at` và xóa trạng thái "chờ sao lưu" nếu mọi thay đổi chờ đã có trong tệp
    /// (thay đổi đầu tiên chưa sao lưu không muộn hơn thời điểm tạo tệp). Chưa tạo tệp nào → lỗi `Input`.
    pub fn mark_backup_saved(&mut self, saved_at: &str) -> Result<()> {
        if !validate::is_timestamp(saved_at) {
            return Err(StoreError::Input(
                "Thời điểm không hợp lệ (RFC 3339 UTC).".into(),
            ));
        }
        let Some(exported) = self.meta_get("last_export_at")? else {
            return Err(StoreError::Input(
                "Chưa tạo tệp sao lưu nào để ghi nhận.".into(),
            ));
        };
        let pending = self.backup_pending_since()?;
        let mut s = self.settings()?;
        s["last_backup_at"] = json!(saved_at);
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE settings SET json = ?1 WHERE id = 1",
            params![jtext(&s)],
        )?;
        if pending.is_some_and(|p| p.as_str() <= exported.as_str()) {
            tx.execute("DELETE FROM meta WHERE key = 'backup_pending_since'", [])?;
        }
        tx.execute("DELETE FROM meta WHERE key = 'last_export_at'", [])?;
        tx.commit()?;
        Ok(())
    }

    /// Như [`Store::export_backup`] nhưng với salt/nonce/thời điểm cố định (vector kiểm thử).
    pub fn export_backup_fixed(
        &self,
        password: &str,
        app: &AppInfo,
        exported_at: &str,
        salt: [u8; 16],
        nonce: [u8; 24],
    ) -> Result<Vec<u8>> {
        let (doc, blobs) = self.snapshot_with_photos(app, exported_at)?;
        Ok(lvhv_backup::export_with_fixed_rng(
            password,
            &canonical_json(&doc),
            &blobs,
            salt,
            nonce,
        )?)
    }

    /// Ghi một tài liệu đã kiểm tra (Thay toàn bộ hoặc Gộp) trong **một** giao dịch. Lỗi → không ghi gì.
    pub fn apply_import(&mut self, b: &VerifiedBackup, mode: ImportMode) -> Result<()> {
        let target = match mode {
            ImportMode::Replace => b.doc.clone(),
            ImportMode::Merge => {
                let local = self.snapshot(
                    &AppInfo {
                        version: "0.0.0".into(),
                        platform: "pwa".into(),
                    },
                    "1970-01-01T00:00:00Z",
                )?;
                let merged = merge_docs(&local, &b.doc);
                validate::document(&merged)?;
                merged
            }
        };
        // Ảnh cần có byte: từ tệp nhập, hoặc (khi gộp) từ kho hiện có.
        let mut new_files: Vec<(String, Vec<u8>)> = Vec::new();
        for ph in target["photos"].as_array().into_iter().flatten() {
            let id = ph["id"].as_str().unwrap_or_default();
            let sha = ph["sha256"].as_str().unwrap_or_default();
            let on_disk = std::fs::read(self.photo_path(id))
                .ok()
                .filter(|x| sha256_hex(x) == sha);
            if on_disk.is_none() {
                match b.photos.get(id) {
                    Some(bytes) => new_files.push((id.to_owned(), bytes.clone())),
                    None => {
                        return Err(StoreError::Invalid(ValidationError {
                            code: "PHOTO",
                            path: id.into(),
                            msg: "thiếu byte ảnh".into(),
                        }))
                    }
                }
            }
        }
        for (id, bytes) in &new_files {
            write_atomic(&self.photo_path(id), bytes)?;
        }
        let now = now_rfc3339();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let res = (|| -> Result<()> {
            for t in ["event_person", "event", "person", "event_type", "photo"] {
                tx.execute(&format!("DELETE FROM {t}"), [])?;
            }
            for ph in target["photos"].as_array().into_iter().flatten() {
                tx.execute(
                    "INSERT INTO photo(id, json, sha256) VALUES (?1, ?2, ?3)",
                    params![ph["id"].as_str(), jtext(ph), ph["sha256"].as_str()],
                )?;
            }
            for t in target["event_types"].as_array().into_iter().flatten() {
                insert_event_type(&tx, t)?;
            }
            for p in target["persons"].as_array().into_iter().flatten() {
                upsert_person(&tx, p)?;
            }
            for e in target["events"].as_array().into_iter().flatten() {
                upsert_event(&tx, e)?;
            }
            if mode == ImportMode::Replace {
                tx.execute(
                    "UPDATE settings SET json = ?1 WHERE id = 1",
                    params![jtext(&target["settings"])],
                )?;
            }
            Self::mark_changed(&tx, &now)?;
            Ok(())
        })();
        match res {
            Ok(()) => tx.commit()?,
            Err(e) => {
                drop(tx);
                for (id, _) in &new_files {
                    let _ = std::fs::remove_file(self.photo_path(id));
                }
                return Err(e);
            }
        }
        // Dọn tệp ảnh không còn bản ghi.
        let keep: std::collections::HashSet<String> = self
            .photos()?
            .iter()
            .filter_map(|p| p["id"].as_str().map(|s| format!("{s}.img")))
            .collect();
        if let Ok(rd) = std::fs::read_dir(self.dir.join(PHOTO_DIR)) {
            for f in rd.flatten() {
                let name = f.file_name().to_string_lossy().into_owned();
                if name.ends_with(".img") && !keep.contains(&name) {
                    let _ = std::fs::remove_file(f.path());
                }
            }
        }
        Ok(())
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn check_refs_person(tx: &rusqlite::Transaction<'_>, p: &Value) -> Result<()> {
    for pid in validate::person_photo_refs(p) {
        let pid = pid.as_str();
        let n: i64 = tx.query_row(
            "SELECT count(*) FROM photo WHERE id = ?1",
            params![pid],
            |r| r.get(0),
        )?;
        if n == 0 {
            return Err(StoreError::Invalid(ValidationError {
                code: "V3",
                path: "/person/photo_id".into(),
                msg: "ảnh không tồn tại".into(),
            }));
        }
    }
    Ok(())
}

fn upsert_person(tx: &rusqlite::Transaction<'_>, p: &Value) -> Result<()> {
    tx.execute(
        "INSERT INTO person(id, kind, photo_id, rev, updated_at, json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET kind = excluded.kind, photo_id = excluded.photo_id, rev = excluded.rev,
           updated_at = excluded.updated_at, json = excluded.json",
        params![p["id"].as_str(), p["kind"].as_str(), p["photo_id"].as_str(), p["rev"].as_i64(), p["updated_at"].as_str(), jtext(p)],
    )?;
    Ok(())
}

fn insert_event_type(tx: &rusqlite::Transaction<'_>, t: &Value) -> Result<()> {
    upsert_event_type(tx, t)
}

fn upsert_event_type(tx: &rusqlite::Transaction<'_>, t: &Value) -> Result<()> {
    tx.execute(
        "INSERT INTO event_type(id, name_key, rev, updated_at, json) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET name_key = excluded.name_key, rev = excluded.rev,
           updated_at = excluded.updated_at, json = excluded.json",
        params![
            t["id"].as_str(),
            validate::name_key(t["ten"].as_str().unwrap_or_default()),
            t["rev"].as_i64(),
            t["updated_at"].as_str(),
            jtext(t)
        ],
    )?;
    Ok(())
}

fn upsert_event(tx: &rusqlite::Transaction<'_>, e: &Value) -> Result<()> {
    let id = e["id"].as_str();
    tx.execute(
        "INSERT INTO event(id, type_id, rev, updated_at, json) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET type_id = excluded.type_id, rev = excluded.rev,
           updated_at = excluded.updated_at, json = excluded.json",
        params![
            id,
            e["type_id"].as_str(),
            e["rev"].as_i64(),
            e["updated_at"].as_str(),
            jtext(e)
        ],
    )?;
    tx.execute("DELETE FROM event_person WHERE event_id = ?1", params![id])?;
    for (i, pid) in e["person_ids"].as_array().into_iter().flatten().enumerate() {
        tx.execute(
            "INSERT INTO event_person(event_id, person_id, pos) VALUES (?1, ?2, ?3)",
            params![id, pid.as_str(), i as i64],
        )?;
    }
    Ok(())
}

/// `AppSettings` mặc định (01 §3.9).
pub fn default_settings() -> Value {
    json!({
        "reminders_enabled": true, "default_offsets_days": [0, 3], "default_time_local": "07:00",
        "upcoming_days": 30, "last_backup_at": null, "backup_nag_days": 30,
    })
}

/// Bốn loại sự kiện có sẵn, `created_at = updated_at = 1970-01-01T00:00:00Z`, `rev = 1` — giá trị cố định
/// để kho mới trên hai máy giống nhau từng byte và gộp được (01 §3.5).
pub fn default_event_types() -> Vec<Value> {
    model::BUILTIN_EVENT_TYPES
        .iter()
        .map(|(b, id, ten, rec)| {
            json!({"id": id, "builtin": b, "ten": ten, "default_recurring": rec,
                   "created_at": "1970-01-01T00:00:00Z", "updated_at": "1970-01-01T00:00:00Z", "rev": 1})
        })
        .collect()
}

/// Bản nhập có "thắng" bản trên máy không (01 §6.3): `rev` lớn hơn; bằng thì `updated_at` muộn hơn;
/// vẫn bằng thì giữ bản trên máy.
fn incoming_wins(local: &Value, incoming: &Value) -> bool {
    let (lr, ir) = (
        local["rev"].as_i64().unwrap_or(0),
        incoming["rev"].as_i64().unwrap_or(0),
    );
    if ir != lr {
        return ir > lr;
    }
    // Chuỗi RFC 3339 cùng định dạng cố định → so chuỗi = so thời điểm.
    incoming["updated_at"].as_str().unwrap_or_default()
        > local["updated_at"].as_str().unwrap_or_default()
}

fn by_id(a: &Value) -> BTreeMap<String, Value> {
    a.as_array()
        .into_iter()
        .flatten()
        .map(|x| (x["id"].as_str().unwrap_or_default().to_owned(), x.clone()))
        .collect()
}

/// Gộp (01 §6.3). Hàm thuần; kết quả còn phải qua [`validate::document`]. Bản TypeScript: `merge.ts`.
pub fn merge_docs(local: &Value, incoming: &Value) -> Value {
    // Loại sự kiện: trùng id → luật rev; khác id nhưng trùng tên → dùng id trên máy, đổi type_id của sự kiện nhập.
    let mut types = by_id(&local["event_types"]);
    let local_key: HashMap<String, String> = types
        .values()
        .map(|t| {
            (
                validate::name_key(t["ten"].as_str().unwrap_or_default()),
                t["id"].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect();
    let mut remap: HashMap<String, String> = HashMap::new();
    for (id, t) in by_id(&incoming["event_types"]) {
        if let Some(l) = types.get(&id) {
            if incoming_wins(l, &t) {
                types.insert(id, t);
            }
            continue;
        }
        let key = validate::name_key(t["ten"].as_str().unwrap_or_default());
        match local_key.get(&key) {
            Some(lid) => {
                remap.insert(id, lid.clone());
            }
            None => {
                types.insert(id, t);
            }
        }
    }
    let merge_list = |k: &str, fix: &dyn Fn(Value) -> Value| -> Vec<Value> {
        let mut m = by_id(&local[k]);
        for (id, x) in by_id(&incoming[k]) {
            let x = fix(x);
            match m.get(&id) {
                Some(l) if !incoming_wins(l, &x) => {}
                _ => {
                    m.insert(id, x);
                }
            }
        }
        m.into_values().collect()
    };
    let persons = merge_list("persons", &|x| x);
    let events = merge_list("events", &|mut e: Value| {
        if let Some(n) = e["type_id"].as_str().and_then(|t| remap.get(t)) {
            e["type_id"] = json!(n);
        }
        e
    });
    // Ảnh: bản trên máy giữ nguyên; thêm ảnh mới; chỉ giữ ảnh còn được tham chiếu.
    let mut photos = by_id(&local["photos"]);
    for (id, x) in by_id(&incoming["photos"]) {
        photos.entry(id).or_insert(x);
    }
    let used: std::collections::HashSet<&str> = persons
        .iter()
        .filter_map(|p| p["photo_id"].as_str())
        .collect();
    let local_photo_ids: std::collections::HashSet<String> =
        by_id(&local["photos"]).into_keys().collect();
    let photos: Vec<Value> = photos
        .into_iter()
        .filter(|(id, _)| used.contains(id.as_str()) || local_photo_ids.contains(id))
        .map(|(_, v)| v)
        .collect();
    let mut out = local.clone();
    let o = out.as_object_mut().map(core::mem::take).unwrap_or_default();
    let mut o: Map<String, Value> = o;
    o.insert(
        "event_types".into(),
        Value::Array(types.into_values().collect()),
    );
    o.insert("persons".into(), Value::Array(persons));
    o.insert("events".into(), Value::Array(events));
    o.insert("photos".into(), Value::Array(photos));
    Value::Object(o)
}

/// Giải mã + kiểm tra toàn bộ (01 §6.3) — **không** ghi gì. Thứ tự: 03 §2.2 → Argon2id → AEAD → mở gói →
/// JSON (độ sâu ≤ 32, ≤ 64 MiB) → lược đồ + V2–V8 → ảnh (đủ, không thừa, `sha256` và `bytes` khớp).
pub fn open_backup(file: &[u8], password: &str) -> Result<VerifiedBackup> {
    let c = lvhv_backup::import(file, password)?;
    verify_contents(&c)
}

/// Phần kiểm tra nội dung của [`open_backup`] (sau khi giải mã).
pub fn verify_contents(c: &lvhv_backup::Contents) -> Result<VerifiedBackup> {
    let doc = validate::parse_json(&c.json)?;
    if let Some(v) = doc.get("schema_version").and_then(Value::as_i64) {
        if v > SCHEMA_VERSION {
            return Err(StoreError::NewerSchema(v));
        }
    }
    validate::document(&doc)?;
    let meta: HashMap<String, &Value> = doc["photos"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|p| (p["id"].as_str().unwrap_or_default().to_owned(), p))
        .collect();
    let mut photos = HashMap::new();
    let perr = |m: &str| {
        StoreError::Invalid(ValidationError {
            code: "PHOTO",
            path: "/photos".into(),
            msg: m.into(),
        })
    };
    for b in &c.photos {
        let id = lvhv_backup::uuid_to_string(&b.id);
        let m = meta.get(&id).ok_or_else(|| perr("ảnh thừa"))?;
        if m["sha256"].as_str() != Some(sha256_hex(&b.bytes).as_str())
            || m["bytes"].as_u64() != Some(b.bytes.len() as u64)
        {
            return Err(perr("sha256/bytes không khớp"));
        }
        if photos.insert(id, b.bytes.clone()).is_some() {
            return Err(perr("ảnh trùng"));
        }
    }
    if photos.len() != meta.len() {
        return Err(perr("thiếu ảnh"));
    }
    Ok(VerifiedBackup { doc, photos })
}

/// Một người mới với trường dẫn xuất tính bằng [`derive`] — tiện cho kiểm thử và cho lớp giao diện.
pub fn build_person(base: &Value) -> Result<Value> {
    let mut p = base.clone();
    if !p["birth"].is_null() {
        let d = derive::derive_birth(&p["birth"]["input"]).map_err(|e| StoreError::Input(e.0))?;
        for (k, v) in d {
            p["birth"][k] = v;
        }
    }
    if !p["death"].is_null() {
        let d = derive::save_death(&p["death"]["input"]).map_err(|e| StoreError::Input(e.0))?;
        for (k, v) in d {
            p["death"][k] = v;
        }
        if p["death"]["tuoi"]["manual"] != true {
            let nguong = p["memorial"]["huong_duong_duoi"].as_i64().unwrap_or(60);
            p["death"]["tuoi"] =
                derive::tuoi_mat(&p["birth"], p["death"]["lunar"]["year"].as_i64(), nguong);
        }
    }
    Ok(p)
}
