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
pub mod media;
pub mod migrate;
pub mod model;
pub mod names;
pub mod validate;

use lvhv_backup::{BackupError, PhotoBlob};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;
use validate::ValidationError;

/// Phiên bản lược đồ dữ liệu mà bản này hiểu (01 §7).
pub const SCHEMA_VERSION: i64 = 1;
/// Tên tệp cơ sở dữ liệu.
pub const DB_FILE: &str = "lvhv.sqlite";
/// Thư mục ảnh (trong thư mục dữ liệu).
pub const PHOTO_DIR: &str = "photos";
/// Thư mục tệp đính kèm thông tin báo tử (01 §3.8a, YC-2026-07).
pub const ATTACHMENT_DIR: &str = "attachments";

/// Lỗi của kho.
#[derive(Debug)]
pub enum StoreError {
    /// Dữ liệu vi phạm V1–V8.
    Invalid(ValidationError),
    /// Lỗi nhập liệu khi tính trường dẫn xuất (thông báo tiếng Việt theo 01 §6.1).
    Input(String),
    /// Kho có `schema_version` lớn hơn bản này hiểu.
    NewerSchema(i64),
    /// Kho đang ở chế độ chỉ đọc vì chuyển đổi phiên bản lỗi (04 B5) — dữ liệu cũ giữ nguyên; hãy xuất sao lưu.
    ReadOnly,
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
            StoreError::NewerSchema(v) => {
                write!(f, "kho có phiên bản {v} > {}", migrate::STORE_VERSION)
            }
            StoreError::ReadOnly => f.write_str(
                "Dữ liệu đang ở chế độ chỉ đọc vì nâng cấp chưa xong — dữ liệu cũ vẫn còn nguyên; hãy xuất sao lưu",
            ),
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
    attachments: HashMap<String, Vec<u8>>,
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

/// Chuỗi chuyển đổi lược đồ SQLite: phần tử `i` đưa kho từ `user_version = i` lên `i + 1`. Bước có chuyển dữ liệu
/// gọi thêm hàm thuần của [`migrate`] (1 → 2: [`migrate::migrate_1_to_2`]).
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
    // 1 → 2 (YC-2026-07): bảng tệp đính kèm. `IF NOT EXISTS`: bản dựng 0.2 trung gian đã tạo bảng mà chưa tăng số.
    r#"
    CREATE TABLE IF NOT EXISTS attachment (
        id TEXT PRIMARY KEY,
        json TEXT NOT NULL,
        sha256 TEXT NOT NULL
    ) STRICT;
    "#,
];

/// Tùy chọn khi mở kho.
#[derive(Debug, Clone, Copy, Default)]
pub struct OpenOptions {
    /// Kiểm định: làm bước chuyển đổi thất bại (sau khi đã chạy) để quan sát bản sao trước chuyển đổi, quay lui và
    /// chế độ chỉ đọc. Không dùng trong ứng dụng.
    pub force_migration_failure: bool,
    /// Đồng hồ cho kiểm thử (giây Unix): "bây giờ" khi mở kho — để kiểm việc dọn sau 7 ngày (BM2-06, BM2-07).
    /// `None` = đồng hồ hệ thống.
    pub now_unix: Option<i64>,
}

/// Số ngày giữ khóa `legacy` và bản sao CSDL trước chuyển đổi sau một lần chuyển đổi **thành công** (01 §7, BM2-06,
/// BM2-07). Chuyển đổi lỗi thì không đếm — bản sao giữ nguyên.
pub const MIGRATION_KEEP_DAYS: i64 = 7;

/// Kho dữ liệu trên một thư mục.
#[derive(Debug)]
pub struct Store {
    conn: Connection,
    dir: PathBuf,
    status: migrate::MigrationStatus,
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
        Self::open_with(dir, OpenOptions::default())
    }

    /// Như [`Store::open`] với tùy chọn kiểm định (`lvhv-store-tool --force-migration-failure`).
    pub fn open_with(dir: impl AsRef<Path>, opts: OpenOptions) -> Result<Store> {
        let dir = dir.as_ref().to_path_buf();
        std::fs::create_dir_all(dir.join(PHOTO_DIR))?;
        std::fs::create_dir_all(dir.join(ATTACHMENT_DIR))?;
        // Bản sao trước chuyển đổi phải giống TỪNG BYTE kho cũ (báo cáo PR #36): đọc `user_version` thẳng từ phần đầu tệp
        // (byte 60–63, big-endian) và chép thô tệp chính (+ `-wal` nếu có) TRƯỚC khi mở kết nối — mở kết nối và đặt
        // chế độ WAL sẽ ghi lại phần đầu tệp (byte 18–19, bộ đếm thay đổi).
        let bak = dir.join(format!("{DB_FILE}.pre-v{}.bak", migrate::STORE_VERSION));
        let mut raw_copy = None;
        if let Ok(head) = std::fs::read(dir.join(DB_FILE)).map(|b| b.get(..100).map(<[u8]>::to_vec))
        {
            if let Some(h) = head.filter(|h| h.starts_with(b"SQLite format 3\0")) {
                let uv = i64::from(u32::from_be_bytes([h[60], h[61], h[62], h[63]]));
                if uv > 0 && uv < migrate::STORE_VERSION && !bak.exists() {
                    std::fs::copy(dir.join(DB_FILE), &bak)?;
                    let wal = dir.join(format!("{DB_FILE}-wal"));
                    if wal.exists() {
                        std::fs::copy(
                            &wal,
                            dir.join(format!("{DB_FILE}.pre-v{}.bak-wal", migrate::STORE_VERSION)),
                        )?;
                    }
                    raw_copy = Some(bak.clone());
                }
            }
        }
        let mut conn = Connection::open(dir.join(DB_FILE))?;
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        // BM2-11 (01 §4.2): nội dung bị xóa được ghi đè bằng 0 trong trang của CSDL (thiết lập theo kết nối, không lưu
        // trong tệp). Sau các thao tác xóa, `checkpoint_truncate` đưa trang đã ghi đè vào tệp chính và làm rỗng WAL.
        conn.pragma_update(None, "secure_delete", true)?;
        let ver: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        // 01 §7: kho của bản mới hơn → từ chối, không cố đọc một phần.
        if ver > migrate::STORE_VERSION {
            return Err(StoreError::NewerSchema(ver));
        }
        let mut status = migrate::MigrationStatus {
            state: "ok",
            from_version: ver,
            to_version: ver.max(migrate::STORE_VERSION),
            backup_path: None,
            error: None,
        };
        let now = opts
            .now_unix
            .map(rfc3339_from_unix)
            .unwrap_or_else(now_rfc3339);
        if ver < migrate::STORE_VERSION {
            if ver > 0 {
                // 01 §7, 04 B5: bản sao nội bộ không mã hóa trước khi chuyển đổi; xóa 7 ngày sau khi chuyển THÀNH CÔNG
                // (BM2-06), giữ nguyên nếu chuyển lỗi. Tệp ảnh và tệp đính kèm không bị xóa bởi phép chuyển nào, nên bản
                // sao CSDL + thư mục ảnh = toàn bộ dữ liệu cũ.
                if raw_copy.is_none() && !bak.exists() {
                    // Dự phòng (phần đầu tệp không đọc được như trên): chép sau khi gộp WAL — dữ liệu đủ, byte phần
                    // đầu có thể khác.
                    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
                    std::fs::copy(dir.join(DB_FILE), &bak)?;
                }
                status.backup_path = Some(bak.to_string_lossy().into_owned());
            }
            match Self::run_migrations(&mut conn, &dir, ver, opts.force_migration_failure) {
                Ok(()) => {
                    if ver > 0 {
                        status.state = "migrated";
                        // Bắt đầu đếm 7 ngày (01 §7): thời điểm chuyển xong + tên bản sao cần dọn.
                        conn.execute(
                            "INSERT OR REPLACE INTO meta(key, value) VALUES ('migrated_at', ?1), ('migration_backup', ?2)",
                            params![
                                now,
                                format!("{DB_FILE}.pre-v{}.bak", migrate::STORE_VERSION)
                            ],
                        )?;
                    }
                }
                Err(e) => {
                    // Giao dịch đã quay lui: kho cũ giữ nguyên; mở ở chế độ chỉ đọc (04 B5 mục 3).
                    status.state = "read_only";
                    status.to_version = ver;
                    status.error = Some(e.to_string());
                }
            }
        }
        if status.state != "read_only" {
            post_migration_cleanup(&mut conn, &dir, &now)?;
        }
        Ok(Store { conn, dir, status })
    }

    /// Chạy chuỗi chuyển đổi từ `ver` lên [`migrate::STORE_VERSION`] trong **một** giao dịch, rồi kiểm 01 §6.2 trên
    /// mọi hồ sơ và tệp đính kèm. Lỗi → quay lui toàn bộ.
    fn run_migrations(conn: &mut Connection, dir: &Path, ver: i64, force_fail: bool) -> Result<()> {
        let mut written = Vec::new();
        let r = Self::run_migrations_tx(conn, dir, ver, force_fail, &mut written);
        if r.is_err() {
            // Giao dịch đã quay lui: xóa tệp đính kèm mới ghi trong lúc chuyển — thư mục kho trở lại như trước, chỉ
            // thêm bản sao trước chuyển đổi (báo cáo PR #36).
            for f in written {
                let _ = std::fs::remove_file(f);
            }
        }
        r
    }

    fn run_migrations_tx(
        conn: &mut Connection,
        dir: &Path,
        ver: i64,
        force_fail: bool,
        written: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let tx = conn.transaction_with_behavior(TransactionBehavior::Exclusive)?;
        for m in &MIGRATIONS[ver as usize..] {
            tx.execute_batch(m)?;
        }
        if ver == 1 {
            migrate_data_1_to_2(&tx, dir, written)?;
        }
        if force_fail && ver > 0 {
            // Chỉ để kiểm định quan sát quay lui + chỉ đọc (01 §7.1); giao dịch bị hủy khi trả lỗi.
            return Err(StoreError::Input(
                "buộc lỗi chuyển đổi (tùy chọn kiểm định)".into(),
            ));
        }
        tx.pragma_update(None, "user_version", migrate::STORE_VERSION)?;
        {
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
            tx.execute(
                "INSERT OR REPLACE INTO meta(key, value) VALUES ('store_version', ?1)",
                params![migrate::STORE_VERSION.to_string()],
            )?;
        }
        // 01 §6.2 sau chuyển đổi: mọi hồ sơ hợp lệ và tham chiếu tồn tại.
        if ver > 0 {
            let rows: Vec<String> = {
                let mut st = tx.prepare("SELECT json FROM person ORDER BY id")?;
                let r = st
                    .query_map([], |r| r.get(0))?
                    .collect::<core::result::Result<_, _>>()?;
                r
            };
            for r in rows {
                let p = parse_col(&r)?;
                validate::person(&p, "/person")?;
                check_refs_person(&tx, &p)?;
            }
            let rows: Vec<String> = {
                let mut st = tx.prepare("SELECT json FROM attachment ORDER BY id")?;
                let r = st
                    .query_map([], |r| r.get(0))?
                    .collect::<core::result::Result<_, _>>()?;
                r
            };
            for r in rows {
                validate::attachment(&parse_col(&r)?, "/attachment")?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Trạng thái chuyển đổi khi mở kho (04 B5): giao diện hiện chế độ chỉ đọc / lỗi khi `state = "read_only"`.
    pub fn migration_status(&self) -> &migrate::MigrationStatus {
        &self.status
    }

    fn writable(&self) -> Result<()> {
        if self.status.state == "read_only" {
            Err(StoreError::ReadOnly)
        } else {
            Ok(())
        }
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
        self.writable()?;
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
    /// Siêu dữ liệu mọi `Attachment` (01 §3.8a), sắp theo `id`.
    pub fn attachments(&self) -> Result<Vec<Value>> {
        // Kho phiên bản 1 mở chỉ đọc sau chuyển đổi lỗi (04 B5) chưa có bảng này.
        let has: i64 = self.conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'attachment'",
            [],
            |r| r.get(0),
        )?;
        if has == 0 {
            return Ok(Vec::new());
        }
        self.all("attachment")
    }

    /// Byte của một tệp đính kèm (kiểm tra `sha256` khi đọc).
    pub fn attachment_bytes(&self, id: &str) -> Result<Vec<u8>> {
        let sha: Option<String> = self
            .conn
            .query_row(
                "SELECT sha256 FROM attachment WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )
            .optional()?;
        let sha = sha.ok_or(StoreError::NotFound)?;
        let b = std::fs::read(self.attachment_path(id))?;
        if sha256_hex(&b) != sha {
            return Err(StoreError::Invalid(ValidationError {
                code: "ATTACHMENT",
                path: id.into(),
                msg: "sha256 không khớp".into(),
            }));
        }
        Ok(b)
    }

    fn attachment_path(&self, id: &str) -> PathBuf {
        self.dir.join(ATTACHMENT_DIR).join(format!("{id}.bin"))
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
    ///
    /// Ảnh mà bản cũ của hồ sơ này dùng nhưng bản mới bỏ (đổi/xóa ảnh chân dung, bỏ ảnh giấy báo tử) bị xóa trong
    /// cùng giao dịch nếu không hồ sơ nào khác còn dùng.
    ///
    /// BM2-02 / 04 B5: hồ sơ gửi tới còn khóa cũ `death.notice.certificate_photo_ids` (ứng dụng chưa cập nhật) được
    /// chuyển bằng [`migrate::migrate_1_to_2`] trong **cùng** giao dịch: ảnh giấy báo tử thành tệp đính kèm.
    pub fn put_person(&mut self, p: &Value) -> Result<()> {
        self.writable()?;
        let mut written = Vec::new();
        let r = self.put_person_inner(p, &mut written);
        if r.is_err() {
            // Tệp đính kèm ghi trong lúc chuyển khi lưu mà giao dịch không xác nhận → xóa (báo cáo PR #36).
            for f in written {
                let _ = std::fs::remove_file(f);
            }
        }
        r
    }

    fn put_person_inner(&mut self, p: &Value, written: &mut Vec<PathBuf>) -> Result<()> {
        let now = now_rfc3339();
        let dir = self.dir.clone();
        let tx = self.conn.transaction()?;
        let migrated;
        let p = if migrate::has_certificate_photo_ids(&json!({ "persons": [p] })) {
            migrated = migrate_person_in_tx(&tx, &dir, p, written)?;
            note_legacy(&tx, &now)?;
            &migrated
        } else {
            p
        };
        validate::person(p, "/person")?;
        check_refs_person(&tx, p)?;
        let old: Option<String> = tx
            .query_row(
                "SELECT json FROM person WHERE id = ?1",
                params![p["id"].as_str()],
                |r| r.get(0),
            )
            .optional()?;
        let (old_refs, old_atts) = match old {
            Some(s) => {
                let o = parse_col(&s)?;
                (
                    validate::person_photo_refs(&o),
                    validate::person_attachment_refs(&o),
                )
            }
            None => (Vec::new(), Vec::new()),
        };
        upsert_person(&tx, p)?;
        let gone = gc_photos(&tx, old_refs)?;
        let gone_att = gc_attachments(&tx, old_atts)?;
        Self::mark_changed(&tx, &now)?;
        tx.commit()?;
        for pid in gone {
            let _ = std::fs::remove_file(self.photo_path(&pid));
        }
        for aid in gone_att {
            let _ = std::fs::remove_file(self.attachment_path(&aid));
        }
        Ok(())
    }

    /// Thêm một tệp đính kèm thông tin báo tử (01 §3.8a, YC-2026-07). Ảnh (`image/jpeg`, `image/webp`): byte
    /// **đã** được nền tảng giải mã và mã hóa lại (bỏ siêu dữ liệu, cạnh dài ≤ 2048 px), ≤ 2 MiB. PDF: lưu nguyên
    /// byte, ≤ 10 MiB, phải bắt đầu bằng `%PDF-`. `file_name`: tên gốc chỉ để hiển thị (không dùng làm tên tệp
    /// trên máy). Trả bản ghi `Attachment`; gắn vào hồ sơ bằng `death.notice.attachments[].file_id`.
    pub fn add_attachment(
        &mut self,
        id: &str,
        mime: &str,
        file_name: Option<&str>,
        bytes: &[u8],
    ) -> Result<Value> {
        self.writable()?;
        let pdf = mime == "application/pdf";
        if pdf && !bytes.starts_with(b"%PDF-") {
            return Err(StoreError::Input(
                "Tệp không phải PDF hợp lệ (thiếu phần đầu %PDF-)".into(),
            ));
        }
        let name = file_name
            .map(|s| s.nfc().collect::<String>().trim().to_owned())
            .filter(|s| !s.is_empty());
        let meta = json!({
            "id": id, "mime": mime, "bytes": bytes.len(), "sha256": sha256_hex(bytes), "stripped": !pdf,
            "file_name": name, "created_at": now_rfc3339(),
        });
        validate::attachment(&meta, "/attachment")?;
        media::check_attachment(mime, bytes).map_err(|e| media_input(e, "Tệp đính kèm"))?;
        write_atomic(&self.attachment_path(id), bytes)?;
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT OR REPLACE INTO attachment(id, json, sha256) VALUES (?1, ?2, ?3)",
            params![id, jtext(&meta), meta["sha256"].as_str()],
        )?;
        Self::mark_changed(&tx, &now_rfc3339())?;
        tx.commit()?;
        Ok(meta)
    }

    /// Xóa một tệp đính kèm không hồ sơ nào dùng (đã thêm rồi người dùng hủy). Tệp hồ sơ gỡ khỏi danh sách thì
    /// [`Store::put_person`] đã tự xóa. Còn được dùng → `Input`; không có → `NotFound`.
    pub fn delete_attachment(&mut self, id: &str) -> Result<()> {
        self.writable()?;
        let tx = self.conn.transaction()?;
        let n: i64 = tx.query_row(
            "SELECT count(*) FROM attachment WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )?;
        if n == 0 {
            return Err(StoreError::NotFound);
        }
        if gc_attachments(&tx, vec![id.to_owned()])?.is_empty() {
            return Err(StoreError::Input(
                "Tệp đang được dùng trong một hồ sơ nên chưa xóa được".into(),
            ));
        }
        Self::mark_changed(&tx, &now_rfc3339())?;
        tx.commit()?;
        let _ = std::fs::remove_file(self.attachment_path(id));
        Ok(())
    }

    /// Xóa một ảnh không hồ sơ nào dùng — ví dụ ảnh đã thêm bằng [`Store::add_photo`] rồi người dùng bỏ không lưu
    /// hồ sơ. Ảnh còn được hồ sơ dùng → `Input`; không có → `NotFound`.
    pub fn delete_photo(&mut self, id: &str) -> Result<()> {
        self.writable()?;
        let tx = self.conn.transaction()?;
        let n: i64 = tx.query_row(
            "SELECT count(*) FROM photo WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )?;
        if n == 0 {
            return Err(StoreError::NotFound);
        }
        if gc_photos(&tx, vec![id.to_owned()])?.is_empty() {
            return Err(StoreError::Input(
                "Ảnh đang được dùng trong một hồ sơ nên chưa xóa được".into(),
            ));
        }
        Self::mark_changed(&tx, &now_rfc3339())?;
        tx.commit()?;
        let _ = std::fs::remove_file(self.photo_path(id));
        Ok(())
    }

    /// Ghi một `Event` (V1, V3, V4, V8).
    pub fn put_event(&mut self, e: &Value) -> Result<()> {
        self.writable()?;
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
        self.writable()?;
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
        self.writable()?;
        let meta = json!({
            "id": id, "mime": mime, "width": width, "height": height, "bytes": bytes.len(),
            "sha256": sha256_hex(bytes), "stripped": true, "created_at": now_rfc3339(),
        });
        validate::photo(&meta, "/photo")?;
        media::check_photo(mime, bytes, width, height).map_err(|e| media_input(e, "Ảnh"))?;
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
        self.writable()?;
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
        let pv = parse_col(&pj)?;
        let refs = validate::person_photo_refs(&pv);
        let att_refs = validate::person_attachment_refs(&pv);
        let legacy_refs = legacy_certificate_ids(&pv);
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
        // Ảnh / tệp đính kèm còn được hồ sơ khác dùng thì giữ.
        let gone = gc_photos(&tx, refs)?;
        let gone_att = gc_attachments(&tx, att_refs)?;
        // BM3-01: tệp ảnh gốc giấy báo tử đã chuyển sang tệp đính kèm (giữ để quay lui trong 7 ngày) — hồ sơ bị xóa
        // thì không còn gì cần quay lui cho nó.
        let gone_moved = prune_moved_photos(&tx, legacy_refs)?;
        Self::mark_changed(&tx, &now)?;
        tx.commit()?;
        for pid in gone.iter().chain(&gone_moved) {
            let _ = std::fs::remove_file(self.photo_path(pid));
        }
        for aid in gone_att {
            let _ = std::fs::remove_file(self.attachment_path(&aid));
        }
        checkpoint_truncate(&self.conn);
        Ok(removed_events)
    }

    /// Xóa một sự kiện.
    pub fn delete_event(&mut self, id: &str) -> Result<()> {
        self.writable()?;
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
        self.writable()?;
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
        let mut doc = json!({
            "format": "lvhv-backup",
            "schema_version": SCHEMA_VERSION,
            "app": { "name": "Lịch và Văn hóa Việt", "version": app.version, "platform": app.platform },
            "exported_at": exported_at,
            "settings": self.settings()?,
            "event_types": self.event_types()?,
            "persons": self.persons()?,
            "events": self.events()?,
            "photos": self.photos()?,
        });
        // `attachments` chỉ có khi khác rỗng: tài liệu không có tệp đính kèm giống hệt bản trước YC-2026-07
        // (vector R6 đã khóa).
        let atts = self.attachments()?;
        if !atts.is_empty() {
            doc["attachments"] = Value::Array(atts);
        }
        Ok(doc)
    }

    /// Tài liệu + byte ảnh (theo thứ tự `photos`).
    pub fn snapshot_with_photos(
        &self,
        app: &AppInfo,
        exported_at: &str,
    ) -> Result<(Value, Vec<PhotoBlob>)> {
        let mut doc = self.snapshot(app, exported_at)?;
        // 01 §7 (BM2-07): khóa `legacy` chỉ để khôi phục trên chính máy đã chuyển đổi — không đưa vào tệp sao lưu.
        for p in doc["persons"].as_array_mut().into_iter().flatten() {
            if let Some(o) = p.as_object_mut() {
                o.remove("legacy");
            }
        }
        let mut blobs = Vec::new();
        for ph in doc["photos"].as_array().into_iter().flatten() {
            let id = ph["id"].as_str().unwrap_or_default();
            let raw = lvhv_backup::uuid_from_str(id).ok_or(StoreError::NotFound)?;
            blobs.push(PhotoBlob {
                id: raw,
                bytes: self.photo_bytes(id)?,
            });
        }
        // Tệp đính kèm đi sau ảnh, cùng danh sách khối (03 §2.3).
        for a in doc["attachments"].as_array().into_iter().flatten() {
            let id = a["id"].as_str().unwrap_or_default();
            let raw = lvhv_backup::uuid_from_str(id).ok_or(StoreError::NotFound)?;
            blobs.push(PhotoBlob {
                id: raw,
                bytes: self.attachment_bytes(id)?,
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
        for a in doc["attachments"].as_array().into_iter().flatten() {
            h.update(
                sha256_hex(&self.attachment_bytes(a["id"].as_str().unwrap_or_default())?)
                    .as_bytes(),
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
        // Chế độ chỉ đọc (04 B5): vẫn xuất được sao lưu, chỉ không ghi gì vào kho.
        if self.writable().is_ok() {
            self.conn.execute(
                "INSERT INTO meta(key, value) VALUES ('last_export_at', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![now],
            )?;
        }
        Ok(file)
    }

    /// Ghi nhận tệp sao lưu vừa tạo bằng [`Store::export_backup`] **đã được lưu** (03 §2.4, MA3N-23-01): đặt
    /// `settings.last_backup_at = saved_at` và xóa trạng thái "chờ sao lưu" nếu mọi thay đổi chờ đã có trong tệp
    /// (thay đổi đầu tiên chưa sao lưu không muộn hơn thời điểm tạo tệp). Chưa tạo tệp nào → lỗi `Input`.
    pub fn mark_backup_saved(&mut self, saved_at: &str) -> Result<()> {
        self.writable()?;
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
        self.writable()?;
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
        let mut new_atts: Vec<(String, Vec<u8>)> = Vec::new();
        for a in target["attachments"].as_array().into_iter().flatten() {
            let id = a["id"].as_str().unwrap_or_default();
            let sha = a["sha256"].as_str().unwrap_or_default();
            let on_disk = std::fs::read(self.attachment_path(id))
                .ok()
                .filter(|x| sha256_hex(x) == sha);
            if on_disk.is_none() {
                match b.attachments.get(id) {
                    Some(bytes) => new_atts.push((id.to_owned(), bytes.clone())),
                    None => {
                        return Err(StoreError::Invalid(ValidationError {
                            code: "ATTACHMENT",
                            path: id.into(),
                            msg: "thiếu byte tệp đính kèm".into(),
                        }))
                    }
                }
            }
        }
        for (id, bytes) in &new_files {
            write_atomic(&self.photo_path(id), bytes)?;
        }
        for (id, bytes) in &new_atts {
            write_atomic(&self.attachment_path(id), bytes)?;
        }
        let now = now_rfc3339();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let res = (|| -> Result<()> {
            for t in [
                "event_person",
                "event",
                "person",
                "event_type",
                "photo",
                "attachment",
            ] {
                tx.execute(&format!("DELETE FROM {t}"), [])?;
            }
            for a in target["attachments"].as_array().into_iter().flatten() {
                tx.execute(
                    "INSERT INTO attachment(id, json, sha256) VALUES (?1, ?2, ?3)",
                    params![a["id"].as_str(), jtext(a), a["sha256"].as_str()],
                )?;
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
            if target["persons"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|p| p.get("legacy").is_some())
            {
                note_legacy(&tx, &now)?;
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
                for (id, _) in &new_atts {
                    let _ = std::fs::remove_file(self.attachment_path(id));
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
        let keep: std::collections::HashSet<String> = self
            .attachments()?
            .iter()
            .filter_map(|p| p["id"].as_str().map(|s| format!("{s}.bin")))
            .collect();
        if let Ok(rd) = std::fs::read_dir(self.dir.join(ATTACHMENT_DIR)) {
            for f in rd.flatten() {
                let name = f.file_name().to_string_lossy().into_owned();
                if name.ends_with(".bin") && !keep.contains(&name) {
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

/// Xóa (bảng `photo`) những ảnh trong `candidates` mà không hồ sơ nào còn dùng; trả id đã xóa (sắp, không lặp) để
/// xóa tệp sau khi giao dịch xong.
fn gc_photos(tx: &rusqlite::Transaction<'_>, mut candidates: Vec<String>) -> Result<Vec<String>> {
    candidates.sort();
    candidates.dedup();
    if candidates.is_empty() {
        return Ok(candidates);
    }
    let mut used: std::collections::HashSet<String> = Default::default();
    {
        let mut st = tx.prepare("SELECT json FROM person")?;
        let rows = st.query_map([], |r| r.get::<_, String>(0))?;
        for r in rows {
            used.extend(validate::person_photo_refs(&parse_col(&r?)?));
        }
    }
    let mut gone = Vec::new();
    for pid in candidates {
        if !used.contains(&pid) && tx.execute("DELETE FROM photo WHERE id = ?1", params![pid])? > 0
        {
            gone.push(pid);
        }
    }
    Ok(gone)
}

/// BM2-11: gộp WAL vào tệp chính rồi cắt WAL về 0 byte, để trang đã ghi đè (`secure_delete`) thay bản cũ còn trong
/// WAL. Lỗi (ví dụ có kết nối đọc khác) bỏ qua: SQLite tự gộp ở lần sau và khi đóng kết nối cuối.
fn checkpoint_truncate(conn: &Connection) {
    let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)");
}

/// Khóa `meta` giữ id các ảnh giấy báo tử đã chuyển sang tệp đính kèm mà tệp `photos/<id>.img` gốc còn giữ (để quay
/// lui bằng bản sao CSDL trong 7 ngày) — BM3-01. Mảng JSON id, sắp, không lặp.
const MOVED_PHOTOS_KEY: &str = "migration_moved_photos";

/// id dùng được làm tên tệp (UUID: chữ, số, `-`).
fn safe_file_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
}

fn moved_photos(conn: &Connection) -> Result<Vec<String>> {
    let v: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = ?1",
            params![MOVED_PHOTOS_KEY],
            |r| r.get(0),
        )
        .optional()?;
    Ok(v.and_then(|t| serde_json::from_str::<Vec<String>>(&t).ok())
        .unwrap_or_default())
}

fn set_moved_photos(tx: &rusqlite::Transaction<'_>, mut ids: Vec<String>) -> Result<()> {
    ids.sort();
    ids.dedup();
    if ids.is_empty() {
        tx.execute("DELETE FROM meta WHERE key = ?1", params![MOVED_PHOTOS_KEY])?;
    } else {
        tx.execute(
            "INSERT OR REPLACE INTO meta(key, value) VALUES (?1, ?2)",
            params![
                MOVED_PHOTOS_KEY,
                serde_json::to_string(&ids).unwrap_or_default()
            ],
        )?;
    }
    Ok(())
}

/// Thêm `ids` (ảnh vừa chuyển trong giao dịch `tx`) vào danh sách [`MOVED_PHOTOS_KEY`].
fn record_moved_photos(tx: &rusqlite::Transaction<'_>, ids: &[String]) -> Result<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut cur = moved_photos(tx)?;
    cur.extend(ids.iter().cloned());
    set_moved_photos(tx, cur)
}

/// `legacy.certificate_photo_ids` của một hồ sơ (danh sách gốc trước chuyển đổi).
fn legacy_certificate_ids(p: &Value) -> Vec<String> {
    p["legacy"]["certificate_photo_ids"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|x| x.as_str().map(str::to_owned))
        .collect()
}

/// BM3-01: trong các ảnh ở danh sách [`MOVED_PHOTOS_KEY`] cộng `extra` (id gốc lấy từ `legacy` vừa bỏ hoặc của hồ sơ
/// vừa xóa — cũng phủ kho đã chuyển trước khi có danh sách), chọn những ảnh **không còn bản ghi `photo`** và **không
/// còn hồ sơ nào giữ trong `legacy`** (không còn gì cần quay lui); bỏ chúng khỏi danh sách và trả id để xóa tệp
/// `photos/<id>.img` sau khi giao dịch xong.
fn prune_moved_photos(tx: &rusqlite::Transaction<'_>, extra: Vec<String>) -> Result<Vec<String>> {
    let listed = moved_photos(tx)?;
    let mut cand: Vec<String> = listed.iter().cloned().chain(extra).collect();
    cand.sort();
    cand.dedup();
    if cand.is_empty() {
        return Ok(cand);
    }
    let mut held: std::collections::HashSet<String> = Default::default();
    {
        let mut st = tx.prepare("SELECT json FROM person")?;
        let rows = st.query_map([], |r| r.get::<_, String>(0))?;
        for r in rows {
            held.extend(legacy_certificate_ids(&parse_col(&r?)?));
        }
    }
    let mut gone = Vec::new();
    let mut keep = Vec::new();
    for pid in cand {
        let has_record = tx
            .query_row(
                "SELECT 1 FROM photo WHERE id = ?1",
                params![pid],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if has_record || held.contains(&pid) {
            if listed.contains(&pid) {
                keep.push(pid);
            }
        } else if safe_file_id(&pid) {
            gone.push(pid);
        }
    }
    set_moved_photos(tx, keep)?;
    Ok(gone)
}

/// Ghi lại rằng kho vừa nhận khóa `legacy` (chuyển đổi khi lưu, hoặc nhập tài liệu đã chuyển): bắt đầu (lại) đếm
/// 7 ngày tới lúc dọn (BM2-07).
fn note_legacy(tx: &rusqlite::Transaction<'_>, now: &str) -> Result<()> {
    tx.execute(
        "INSERT OR REPLACE INTO meta(key, value) VALUES ('migrated_at', ?1)",
        params![now],
    )?;
    Ok(())
}

/// 01 §7, BM2-06/BM2-07: 7 ngày sau lần chuyển đổi thành công (`meta.migrated_at`), bỏ khóa `legacy` khỏi mọi hồ sơ
/// (không đổi `rev`/`updated_at`: đây là dọn nội bộ, không phải sửa của người dùng) và xóa bản sao CSDL trước chuyển
/// đổi ghi ở `meta.migration_backup`. Chưa đủ 7 ngày, hoặc chưa từng chuyển thành công → không làm gì.
fn post_migration_cleanup(conn: &mut Connection, dir: &Path, now: &str) -> Result<()> {
    let get = |k: &str| -> Result<Option<String>> {
        Ok(conn
            .query_row("SELECT value FROM meta WHERE key = ?1", params![k], |r| {
                r.get(0)
            })
            .optional()?)
    };
    let Some(at) = get("migrated_at")? else {
        return Ok(());
    };
    let backup = get("migration_backup")?;
    let (Some(a), Some(n)) = (unix_from_rfc3339(&at), unix_from_rfc3339(now)) else {
        return Ok(());
    };
    if n - a < MIGRATION_KEEP_DAYS * 86_400 {
        return Ok(());
    }
    let tx = conn.transaction()?;
    let rows: Vec<String> = {
        let mut st = tx.prepare("SELECT json FROM person ORDER BY id")?;
        let r = st
            .query_map([], |r| r.get(0))?
            .collect::<core::result::Result<_, _>>()?;
        r
    };
    let mut legacy_ids = Vec::new();
    for r in rows {
        let mut p = parse_col(&r)?;
        legacy_ids.extend(legacy_certificate_ids(&p));
        if p.as_object_mut().and_then(|o| o.remove("legacy")).is_some() {
            upsert_person(&tx, &p)?;
        }
    }
    // BM3-01: hết hạn quay lui → xóa tệp ảnh gốc giấy báo tử đã chuyển (không còn bản ghi `photo` nào trỏ tới).
    let gone_moved = prune_moved_photos(&tx, legacy_ids)?;
    tx.execute(
        "DELETE FROM meta WHERE key IN ('migrated_at', 'migration_backup')",
        [],
    )?;
    tx.execute(
        "INSERT OR REPLACE INTO meta(key, value) VALUES ('legacy_pruned_at', ?1)",
        params![now],
    )?;
    tx.commit()?;
    for pid in gone_moved {
        let _ = std::fs::remove_file(dir.join(PHOTO_DIR).join(format!("{pid}.img")));
    }
    if let Some(b) =
        backup.filter(|b| b.starts_with(DB_FILE) && b.ends_with(".bak") && !b.contains('/'))
    {
        let _ = std::fs::remove_file(dir.join(format!("{b}-wal")));
        let _ = std::fs::remove_file(dir.join(b));
    }
    checkpoint_truncate(conn);
    Ok(())
}

/// Bước dữ liệu của chuyển đổi kho 1 → 2 (YC-2026-07): đọc hồ sơ, ảnh, tệp đính kèm; áp
/// [`migrate::migrate_1_to_2`]; chép byte ảnh sang tệp đính kèm (chưa xóa tệp ảnh nào — tệp gốc của ảnh đã chuyển
/// được ghi ở [`MOVED_PHOTOS_KEY`] và xóa sau 7 ngày hoặc khi xóa hồ sơ, BM3-01); ghi lại trong giao dịch `tx`.
fn migrate_data_1_to_2(
    tx: &rusqlite::Transaction<'_>,
    dir: &Path,
    written: &mut Vec<PathBuf>,
) -> Result<()> {
    let col = |sql: &str| -> Result<Vec<Value>> {
        let mut st = tx.prepare(sql)?;
        let rows: Vec<String> = st
            .query_map([], |r| r.get(0))?
            .collect::<core::result::Result<_, _>>()?;
        rows.iter().map(|r| parse_col(r)).collect()
    };
    let doc = json!({
        "persons": col("SELECT json FROM person ORDER BY id")?,
        "photos": col("SELECT json FROM photo ORDER BY id")?,
        "attachments": col("SELECT json FROM attachment ORDER BY id")?,
    });
    let m = migrate::migrate_1_to_2(&doc)?;
    if !m.changed {
        return Ok(());
    }
    // 04 B6: kiểm chặt byte của mỗi tệp đính kèm mới TRƯỚC khi ghi; lỗi → quay lui, chỉ đọc.
    let mime: HashMap<&str, &str> = m.doc["attachments"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|a| {
            (
                a["id"].as_str().unwrap_or_default(),
                a["mime"].as_str().unwrap_or_default(),
            )
        })
        .collect();
    let mut blobs = Vec::new();
    for (aid, pid) in &m.blob_sources {
        let b = std::fs::read(dir.join(PHOTO_DIR).join(format!("{pid}.img")))?;
        media::check_attachment(mime.get(aid.as_str()).copied().unwrap_or_default(), &b)
            .map_err(|e| media_input(e, &format!("ảnh giấy báo tử {pid}")))?;
        blobs.push((aid, b));
    }
    for (aid, b) in blobs {
        // Ghi trước khi giao dịch được xác nhận; người gọi xóa các tệp này nếu giao dịch quay lui (báo cáo PR #36).
        let f = dir.join(ATTACHMENT_DIR).join(format!("{aid}.bin"));
        if !f.exists() {
            written.push(f.clone());
        }
        write_atomic(&f, &b)?;
    }
    for a in m.doc["attachments"].as_array().into_iter().flatten() {
        tx.execute(
            "INSERT OR REPLACE INTO attachment(id, json, sha256) VALUES (?1, ?2, ?3)",
            params![a["id"].as_str(), jtext(a), a["sha256"].as_str()],
        )?;
    }
    for p in m.doc["persons"].as_array().into_iter().flatten() {
        upsert_person(tx, p)?;
    }
    for pid in &m.moved_photos {
        tx.execute("DELETE FROM photo WHERE id = ?1", params![pid])?;
    }
    record_moved_photos(tx, &m.moved_photos)?;
    Ok(())
}

/// Chuyển một hồ sơ còn `certificate_photo_ids` (BM2-02) trong giao dịch `tx`: dựng tài liệu từ kho (hồ sơ khác + hồ sơ
/// này, ảnh, tệp đính kèm), áp [`migrate::migrate_1_to_2`], kiểm chặt byte (04 B6), ghi tệp đính kèm mới, chuyển bản
/// ghi ảnh. Trả hồ sơ đã chuyển (người gọi kiểm và ghi).
fn migrate_person_in_tx(
    tx: &rusqlite::Transaction<'_>,
    dir: &Path,
    p: &Value,
    written: &mut Vec<PathBuf>,
) -> Result<Value> {
    let col = |sql: &str| -> Result<Vec<Value>> {
        let mut st = tx.prepare(sql)?;
        let rows: Vec<String> = st
            .query_map([], |r| r.get(0))?
            .collect::<core::result::Result<_, _>>()?;
        rows.iter().map(|r| parse_col(r)).collect()
    };
    let id = p["id"].as_str().unwrap_or_default();
    let mut persons: Vec<Value> = col("SELECT json FROM person ORDER BY id")?
        .into_iter()
        .filter(|x| x["id"].as_str() != Some(id))
        .collect();
    persons.push(p.clone());
    let doc = json!({
        "persons": persons,
        "photos": col("SELECT json FROM photo ORDER BY id")?,
        "attachments": col("SELECT json FROM attachment ORDER BY id")?,
    });
    let m = migrate::migrate_1_to_2(&doc)?;
    let mime: HashMap<&str, &str> = m.doc["attachments"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|a| {
            (
                a["id"].as_str().unwrap_or_default(),
                a["mime"].as_str().unwrap_or_default(),
            )
        })
        .collect();
    let mut blobs = Vec::new();
    for (aid, pid) in &m.blob_sources {
        let b = std::fs::read(dir.join(PHOTO_DIR).join(format!("{pid}.img")))?;
        media::check_attachment(mime.get(aid.as_str()).copied().unwrap_or_default(), &b)
            .map_err(|e| media_input(e, &format!("ảnh giấy báo tử {pid}")))?;
        blobs.push((aid.clone(), b));
    }
    let new_ids: std::collections::HashSet<&str> =
        m.blob_sources.iter().map(|(a, _)| a.as_str()).collect();
    for a in m.doc["attachments"].as_array().into_iter().flatten() {
        if new_ids.contains(a["id"].as_str().unwrap_or_default()) {
            tx.execute(
                "INSERT OR REPLACE INTO attachment(id, json, sha256) VALUES (?1, ?2, ?3)",
                params![a["id"].as_str(), jtext(a), a["sha256"].as_str()],
            )?;
        }
    }
    for pid in &m.moved_photos {
        tx.execute("DELETE FROM photo WHERE id = ?1", params![pid])?;
    }
    record_moved_photos(tx, &m.moved_photos)?;
    for (aid, b) in blobs {
        // Ghi trước khi giao dịch được xác nhận; người gọi xóa các tệp này nếu giao dịch quay lui (báo cáo PR #36).
        let f = dir.join(ATTACHMENT_DIR).join(format!("{aid}.bin"));
        if !f.exists() {
            written.push(f.clone());
        }
        write_atomic(&f, &b)?;
    }
    m.doc["persons"]
        .as_array()
        .and_then(|a| a.iter().find(|x| x["id"].as_str() == Some(id)))
        .cloned()
        .ok_or(StoreError::NotFound)
}

/// Lỗi kiểm chặt ảnh (04 B6) khi **thêm** ảnh / tệp: câu cho người dùng. (Khi nhập sao lưu dùng thông báo chung.)
fn media_input(e: media::MediaError, what: &str) -> StoreError {
    StoreError::Input(match e {
        media::MediaError::Metadata => format!("{what}: ảnh chưa được xử lý bỏ siêu dữ liệu"),
        media::MediaError::TooLarge => format!("{what}: tệp quá lớn"),
        media::MediaError::PdfNoEof => format!("{what}: tệp PDF không hoàn chỉnh"),
        _ => format!("{what}: tệp không đúng định dạng đã khai ({})", e.code()),
    })
}

/// Như [`gc_photos`] cho tệp đính kèm (bảng `attachment`).
fn gc_attachments(
    tx: &rusqlite::Transaction<'_>,
    mut candidates: Vec<String>,
) -> Result<Vec<String>> {
    candidates.sort();
    candidates.dedup();
    if candidates.is_empty() {
        return Ok(candidates);
    }
    let mut used: std::collections::HashSet<String> = Default::default();
    {
        let mut st = tx.prepare("SELECT json FROM person")?;
        let rows = st.query_map([], |r| r.get::<_, String>(0))?;
        for r in rows {
            used.extend(validate::person_attachment_refs(&parse_col(&r?)?));
        }
    }
    let mut gone = Vec::new();
    for id in candidates {
        if !used.contains(&id)
            && tx.execute("DELETE FROM attachment WHERE id = ?1", params![id])? > 0
        {
            gone.push(id);
        }
    }
    Ok(gone)
}

fn check_refs_person(tx: &rusqlite::Transaction<'_>, p: &Value) -> Result<()> {
    for (j, aid) in validate::person_attachment_refs(p).iter().enumerate() {
        let n: i64 = tx.query_row(
            "SELECT count(*) FROM attachment WHERE id = ?1",
            params![aid],
            |r| r.get(0),
        )?;
        if n == 0 {
            return Err(StoreError::Invalid(ValidationError {
                code: "V3",
                path: format!("/person/death/notice/attachments/{j}/file_id"),
                msg: "tệp đính kèm không tồn tại".into(),
            }));
        }
    }
    let tray = validate::tray_photo_ref(p);
    for pid in validate::person_photo_refs(p) {
        let pid = pid.as_str();
        let n: i64 = tx.query_row(
            "SELECT count(*) FROM photo WHERE id = ?1",
            params![pid],
            |r| r.get(0),
        )?;
        if n == 0 {
            let is_tray = tray == Some(pid) && p["photo_id"].as_str() != Some(pid);
            return Err(StoreError::Invalid(ValidationError {
                code: "V3",
                path: if is_tray {
                    "/person/memorial/tray_photo_id".into()
                } else {
                    "/person/photo_id".into()
                },
                msg: if is_tray {
                    "ảnh mâm lễ không tồn tại".into()
                } else {
                    "ảnh không tồn tại".into()
                },
            }));
        }
    }
    // YC-2026-10: ảnh mâm lễ chỉ thuộc một người.
    if tray.is_some() || p["photo_id"].is_string() {
        let others: Vec<Value> = {
            let mut st = tx.prepare("SELECT json FROM person WHERE id <> ?1")?;
            let rows: Vec<String> = st
                .query_map(params![p["id"].as_str()], |r| r.get(0))?
                .collect::<core::result::Result<_, _>>()?;
            rows.iter().map(|r| parse_col(r)).collect::<Result<_>>()?
        };
        let refs: Vec<&Value> = others.iter().collect();
        if let Some(f) = validate::tray_photo_shared(p, &refs) {
            return Err(StoreError::Invalid(ValidationError {
                code: "V3",
                path: format!("/person/{f}"),
                msg: "ảnh mâm lễ dùng chung với hồ sơ khác".into(),
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
    // Ảnh chân dung và ảnh mâm lễ (YC-2026-10) mà hồ sơ sau gộp còn dùng.
    let used: std::collections::HashSet<String> = persons
        .iter()
        .flat_map(validate::person_photo_refs)
        .collect();
    let local_photo_ids: std::collections::HashSet<String> =
        by_id(&local["photos"]).into_keys().collect();
    let photos: Vec<Value> = photos
        .into_iter()
        .filter(|(id, _)| used.contains(id) || local_photo_ids.contains(id))
        .map(|(_, v)| v)
        .collect();
    // Tệp đính kèm: như ảnh — giữ của máy, thêm tệp mới mà hồ sơ sau gộp còn dùng.
    let mut atts = by_id(&local["attachments"]);
    let local_att_ids: std::collections::HashSet<String> = atts.keys().cloned().collect();
    for (id, x) in by_id(&incoming["attachments"]) {
        atts.entry(id).or_insert(x);
    }
    let used_att: std::collections::HashSet<String> = persons
        .iter()
        .flat_map(validate::person_attachment_refs)
        .collect();
    let atts: Vec<Value> = atts
        .into_iter()
        .filter(|(id, _)| used_att.contains(id) || local_att_ids.contains(id))
        .map(|(_, v)| v)
        .collect();
    let mut out = local.clone();
    let o = out.as_object_mut().map(core::mem::take).unwrap_or_default();
    let mut o: Map<String, Value> = o;
    o.remove("attachments");
    if !atts.is_empty() {
        o.insert("attachments".into(), Value::Array(atts));
    }
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
    // 04 B5 mục 5: tệp của bản dựng 0.2 trước YC-2026-07 (`certificate_photo_ids`) → chuyển bằng cùng hàm thuần,
    // rồi xếp lại khối theo tài liệu mới (ảnh rồi tệp đính kèm; byte tệp đính kèm lấy từ ảnh nguồn).
    let mut doc = doc;
    let mut migrated: Option<Vec<(String, Vec<u8>)>> = None;
    if migrate::has_certificate_photo_ids(&doc) {
        let m = migrate::migrate_1_to_2(&doc)?;
        let src: HashMap<String, &[u8]> = c
            .photos
            .iter()
            .map(|b| (lvhv_backup::uuid_to_string(&b.id), b.bytes.as_slice()))
            .collect();
        let from: HashMap<&str, &str> = m
            .blob_sources
            .iter()
            .map(|(a, p)| (a.as_str(), p.as_str()))
            .collect();
        let mut list = Vec::new();
        for (k, is_att) in [("photos", false), ("attachments", true)] {
            for r in m.doc[k].as_array().into_iter().flatten() {
                let id = r["id"].as_str().unwrap_or_default();
                let key = if is_att {
                    from.get(id).copied().unwrap_or(id)
                } else {
                    id
                };
                if let Some(b) = src.get(key) {
                    list.push((id.to_owned(), b.to_vec()));
                }
            }
        }
        migrated = Some(list);
        doc = m.doc;
    }
    validate::document(&doc)?;
    let blobs: Vec<(String, Vec<u8>)> = match migrated {
        Some(l) => l,
        None => c
            .photos
            .iter()
            .map(|b| (lvhv_backup::uuid_to_string(&b.id), b.bytes.clone()))
            .collect(),
    };
    let meta: HashMap<String, &Value> = doc["photos"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|p| (p["id"].as_str().unwrap_or_default().to_owned(), p))
        .collect();
    let att_meta: HashMap<String, &Value> = doc["attachments"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|p| (p["id"].as_str().unwrap_or_default().to_owned(), p))
        .collect();
    let mut photos = HashMap::new();
    let mut attachments = HashMap::new();
    let perr = |m: &str| {
        StoreError::Invalid(ValidationError {
            code: "PHOTO",
            path: "/photos".into(),
            msg: m.into(),
        })
    };
    let aerr = |m: &str| {
        StoreError::Invalid(ValidationError {
            code: "ATTACHMENT",
            path: "/attachments".into(),
            msg: m.into(),
        })
    };
    // Khối: ảnh trước, tệp đính kèm sau (03 §2.3); mỗi khối khớp đúng một bản ghi theo id.
    for (id, bytes) in &blobs {
        let id = id.clone();
        let b = PhotoBlob {
            id: [0; 16],
            bytes: bytes.clone(),
        };
        if let Some(m) = meta.get(&id) {
            if m["sha256"].as_str() != Some(sha256_hex(&b.bytes).as_str())
                || m["bytes"].as_u64() != Some(b.bytes.len() as u64)
            {
                return Err(perr("sha256/bytes không khớp"));
            }
            if !attachments.is_empty() {
                return Err(perr("ảnh phải đứng trước tệp đính kèm"));
            }
            // 04 B6 + B7 (BM2-01): ảnh chân dung — chữ ký khớp mime, kích thước bằng bản ghi và ≤ 2048, không EXIF/XMP/IPTC.
            if let Err(e) = media::check_photo(
                m["mime"].as_str().unwrap_or_default(),
                &b.bytes,
                m["width"].as_u64().unwrap_or(0) as u32,
                m["height"].as_u64().unwrap_or(0) as u32,
            ) {
                return Err(perr(e.code()));
            }
            if photos.insert(id, b.bytes.clone()).is_some() {
                return Err(perr("ảnh trùng"));
            }
        } else if let Some(m) = att_meta.get(&id) {
            if m["sha256"].as_str() != Some(sha256_hex(&b.bytes).as_str())
                || m["bytes"].as_u64() != Some(b.bytes.len() as u64)
            {
                return Err(aerr("sha256/bytes không khớp"));
            }
            // 04 B6 (BM2-01, X21): chữ ký khớp mime, kích thước, không siêu dữ liệu, PDF có %%EOF.
            if let Err(e) =
                media::check_attachment(m["mime"].as_str().unwrap_or_default(), &b.bytes)
            {
                return Err(aerr(e.code()));
            }
            if attachments.insert(id, b.bytes.clone()).is_some() {
                return Err(aerr("tệp đính kèm trùng"));
            }
        } else {
            return Err(perr("ảnh thừa"));
        }
    }
    if photos.len() != meta.len() {
        return Err(perr("thiếu ảnh"));
    }
    if attachments.len() != att_meta.len() {
        return Err(aerr("thiếu tệp đính kèm"));
    }
    Ok(VerifiedBackup {
        doc,
        photos,
        attachments,
    })
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
