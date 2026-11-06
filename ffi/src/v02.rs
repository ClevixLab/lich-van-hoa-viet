//! Phần v0.2 của lớp FFI: hồ sơ người, sự kiện, cài đặt (lvhv-store), ngày giỗ / nhắc / danh sách / .ics
//! (lvhv-memorial), sao lưu (lvhv-backup qua lvhv-store) — cho giao diện native đợt 3 (docs/v0.2/05 §2).
//!
//! Nguyên tắc: ít hàm, mỗi hàm làm trọn một việc; dữ liệu đi vào/ra dạng **JSON** đúng lược đồ
//! `docs/v0.2/schema/backup-v1.schema.json` (Person, Event, EventType, AppSettings, tài liệu sao lưu), để
//! Kotlin/Swift/Rust dùng chung một định dạng với PWA. Lỗi: `LvhvDataError` (thông điệp tiếng Việt).
//!
//! Thời điểm: giây Unix (UTC). Giờ đồng hồ thiết bị: chuỗi `YYYY-MM-DDTHH:MM` (`fire_local`) — dùng
//! cho trigger theo ngày–giờ không gắn múi giờ (02 §7.4). Múi giờ thiết bị: [`DeviceZone`].
//!
//! Lịch: phương pháp mặc định của lõi (`Jpl`).

use crate::SolarDay;
use lvhv_core::DEFAULT_METHOD;
use lvhv_memorial::model::{Dataset, Feb29Rule, Ymd};
use lvhv_memorial::occ::{date_line, days_text, notification_text, upcoming, OccClass, Occurrence};
use lvhv_memorial::period::{self, PeriodKind, PersonFilter};
use lvhv_memorial::remind::{
    all_reminders, fmt_local, reschedule, reschedule_input, to_pending, Pending, Reminder, Zone,
    SENTINEL_ID, SENTINEL_TEXT, WINDOW_DAYS,
};
use lvhv_memorial::Cal;
use lvhv_store::{AppInfo, Store, StoreError};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

/// JDN của 1970-01-01 (mốc giây Unix).
const UNIX_EPOCH_JDN: i64 = 2_440_588;

/// Lỗi của phần dữ liệu v0.2. Thông điệp tiếng Việt, hiển thị được.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Error)]
#[uniffi(flat_error)]
pub enum LvhvDataError {
    /// Dữ liệu nhập sai (ngày không tồn tại, trường thiếu, vi phạm V1–V8…).
    Invalid(String),
    /// Không tìm thấy bản ghi.
    NotFound(String),
    /// Lỗi kho (SQLite, tệp ảnh).
    Storage(String),
    /// Tệp sao lưu sai mật khẩu / hỏng / phiên bản mới hơn (03 §2.4: thông báo chung).
    Backup(String),
}

impl std::fmt::Display for LvhvDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LvhvDataError::Invalid(s)
            | LvhvDataError::NotFound(s)
            | LvhvDataError::Storage(s)
            | LvhvDataError::Backup(s) => f.write_str(s),
        }
    }
}

impl std::error::Error for LvhvDataError {}

impl From<StoreError> for LvhvDataError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::Invalid(v) => LvhvDataError::Invalid(format!("Dữ liệu không hợp lệ: {v}")),
            StoreError::Input(s) => LvhvDataError::Invalid(s),
            StoreError::NotFound => LvhvDataError::NotFound("Không tìm thấy bản ghi.".into()),
            e @ (StoreError::Backup(_) | StoreError::NewerSchema(_)) => {
                LvhvDataError::Backup(e.import_message().into())
            }
            e @ (StoreError::Sql(_) | StoreError::Io(_)) => {
                LvhvDataError::Storage(format!("Lỗi lưu trữ: {e}"))
            }
        }
    }
}

type R<T> = Result<T, LvhvDataError>;

fn bad(msg: impl Into<String>) -> LvhvDataError {
    LvhvDataError::Invalid(msg.into())
}

fn parse_json(text: &str) -> R<Value> {
    lvhv_store::validate::parse_json(text).map_err(|e| bad(format!("JSON không hợp lệ: {e}")))
}

fn cal() -> &'static Cal {
    Cal::get(DEFAULT_METHOD)
}

/// Miền năm dương của lõi (1900–3000, L13).
const YEAR_MIN: i32 = 1900;
const YEAR_MAX: i32 = 3000;
/// Giây Unix của 1900-01-01T00:00Z và 3001-01-01T00:00Z (miền hợp lý của thời điểm).
const UNIX_MIN: i64 = -2_208_988_800;
const UNIX_MAX: i64 = 32_535_216_000;
/// Độ lệch múi giờ tối đa (phút, ±18 giờ — ISO 8601 / java.time).
const OFFSET_MAX: i32 = 18 * 60;

fn ymd(d: SolarDay) -> R<Ymd> {
    if !(YEAR_MIN..=YEAR_MAX).contains(&d.y) {
        return Err(bad(format!(
            "Năm {} ngoài miền {YEAR_MIN}–{YEAR_MAX} của lịch.",
            d.y
        )));
    }
    if d.m < 1 || d.d < 1 || !lvhv_core::jd::is_valid_solar(d.y, d.m as u32, d.d as u32) {
        return Err(bad(format!("Ngày {}/{}/{} không tồn tại.", d.d, d.m, d.y)));
    }
    Ok(Ymd::new(d.y, d.m as u32, d.d as u32))
}

/// Thời điểm (giây Unix) phải nằm trong 1900–3000; ngoài miền thường là lỗi đơn vị (mili giây…).
fn unix_ok(s: i64, what: &str) -> R<i64> {
    if (UNIX_MIN..UNIX_MAX).contains(&s) {
        Ok(s)
    } else {
        Err(bad(format!(
            "{what} không hợp lệ (giây Unix, trong 1900–3000): {s}."
        )))
    }
}

fn solar_day(d: Ymd) -> SolarDay {
    SolarDay {
        y: d.y,
        m: d.m as i32,
        d: d.d as i32,
    }
}

fn dataset(text: &str) -> R<Dataset> {
    lvhv_memorial::model::parse_dataset(text)
        .map_err(|e| bad(format!("Bộ dữ liệu không hợp lệ: {e}")))
}

/// Giây Unix → phút UTC từ JDN 0 (đơn vị của lvhv-memorial).
fn min_of_unix(s: i64) -> i64 {
    s.div_euclid(60) + UNIX_EPOCH_JDN * 1440
}
/// Phút UTC từ JDN 0 → giây Unix.
fn unix_of_min(m: i64) -> i64 {
    (m - UNIX_EPOCH_JDN * 1440) * 60
}

// ------------------------------------------------------------------------------------------------
// Múi giờ thiết bị
// ------------------------------------------------------------------------------------------------

/// Một lần chuyển độ lệch giờ (ví dụ vào/ra giờ mùa hè).
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct ZoneTransition {
    /// Thời điểm chuyển (giây Unix, UTC).
    pub at_unix_s: i64,
    /// Độ lệch so với UTC từ thời điểm đó (phút, ví dụ 420 = +07:00).
    pub offset_min: i32,
}

/// Múi giờ thiết bị: `offset_min` áp dụng trước lần chuyển đầu tiên; `transitions` tăng dần. Việt Nam:
/// `{offset_min: 420, transitions: []}`. Ứng dụng lấy các lần chuyển sắp tới (≤ 401 ngày) từ hệ điều hành
/// (`ZoneRules.nextTransition` / `TimeZone.nextDaylightSavingTimeTransition`).
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct DeviceZone {
    pub offset_min: i32,
    pub transitions: Vec<ZoneTransition>,
}

/// Kiểm và đổi múi giờ: |độ lệch| ≤ 1080 phút, lần chuyển tăng nghiêm ngặt và trong miền thời điểm.
fn zone(z: &DeviceZone) -> R<Zone> {
    let bad_zone = || {
        bad("Múi giờ không hợp lệ (độ lệch tính bằng phút, |độ lệch| ≤ 1080; lần chuyển tăng dần).")
    };
    if z.offset_min.abs() > OFFSET_MAX {
        return Err(bad_zone());
    }
    let mut prev: Option<i64> = None;
    for t in &z.transitions {
        if t.offset_min.abs() > OFFSET_MAX
            || !(UNIX_MIN..UNIX_MAX).contains(&t.at_unix_s)
            || prev.is_some_and(|p| t.at_unix_s <= p)
        {
            return Err(bad_zone());
        }
        prev = Some(t.at_unix_s);
    }
    Ok(Zone {
        initial: z.offset_min,
        transitions: z
            .transitions
            .iter()
            .map(|t| (min_of_unix(t.at_unix_s), t.offset_min))
            .collect(),
    })
}

// ------------------------------------------------------------------------------------------------
// Kho dữ liệu (lvhv-store)
// ------------------------------------------------------------------------------------------------

/// Kho SQLite của ứng dụng (01 §4.2) trên một thư mục riêng của ứng dụng. Mọi lệnh ghi kiểm V1–V8.
#[derive(uniffi::Object)]
pub struct LvhvStore {
    inner: Mutex<Store>,
}

/// Cách nhập sao lưu (01 §6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum BackupImportMode {
    /// Thay toàn bộ dữ liệu trên máy.
    Replace,
    /// Gộp theo id / rev / updated_at.
    Merge,
}

/// Xem trước tệp sao lưu (01 §6.3).
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct BackupPreview {
    pub persons: u32,
    pub events: u32,
    pub photos: u32,
    pub exported_at: String,
    pub platform: String,
}

/// Tệp sao lưu đã giải mã và kiểm tra xong (chưa ghi gì). Dùng `preview()` rồi `LvhvStore.apply_import`.
#[derive(uniffi::Object)]
pub struct VerifiedBackup {
    inner: lvhv_store::VerifiedBackup,
}

#[uniffi::export]
impl VerifiedBackup {
    pub fn preview(&self) -> BackupPreview {
        let p = self.inner.preview();
        BackupPreview {
            persons: p.persons as u32,
            events: p.events as u32,
            photos: p.photos as u32,
            exported_at: p.exported_at,
            platform: p.platform,
        }
    }
}

/// Giải mã + kiểm tra toàn bộ một tệp sao lưu (03 §2, 01 §6.3) — **không** ghi gì. Sai mật khẩu, tệp
/// hỏng hay bị sửa → `Backup("Sai mật khẩu hoặc tệp bị hỏng")`.
#[uniffi::export]
pub fn open_backup(file: Vec<u8>, password: String) -> R<Arc<VerifiedBackup>> {
    let password = zeroize::Zeroizing::new(password);
    let v = lvhv_store::open_backup(&file, &password)
        .map_err(|e| LvhvDataError::Backup(e.import_message().into()))?;
    Ok(Arc::new(VerifiedBackup { inner: v }))
}

/// UUID v4 ngẫu nhiên (01 §1), chữ thường có gạch.
#[uniffi::export]
pub fn new_uuid() -> R<String> {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b)
        .map_err(|_| LvhvDataError::Storage("Không lấy được số ngẫu nhiên.".into()))?;
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    Ok(lvhv_backup::uuid_to_string(&b))
}

fn to_text(v: &Value) -> String {
    lvhv_store::canonical_json(v)
}

fn arr_text(v: Vec<Value>) -> String {
    to_text(&Value::Array(v))
}

/// Gắn id (nếu thiếu), `created_at` (giữ của bản cũ), `updated_at` = bây giờ, `rev` = cũ + 1.
fn stamp(v: &mut Value, old: Option<&Value>) -> R<()> {
    let o = v
        .as_object_mut()
        .ok_or_else(|| bad("Cần một đối tượng JSON."))?;
    if o.get("id")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        o.insert("id".into(), json!(new_uuid()?));
    }
    let now = lvhv_store::now_rfc3339();
    let (created, rev) = match old {
        Some(x) => (
            x["created_at"].as_str().unwrap_or(&now).to_owned(),
            x["rev"].as_i64().unwrap_or(0) + 1,
        ),
        None => (now.clone(), 1),
    };
    o.insert("created_at".into(), json!(created));
    o.insert("updated_at".into(), json!(now));
    o.insert("rev".into(), json!(rev));
    Ok(())
}

fn find_by_id(list: Vec<Value>, id: &str) -> Option<Value> {
    list.into_iter().find(|x| x["id"].as_str() == Some(id))
}

impl LvhvStore {
    fn lock(&self) -> std::sync::MutexGuard<'_, Store> {
        // Một lệnh lỗi giữa chừng không làm hỏng kho (mỗi lệnh ghi là một giao dịch), nên lấy lại khóa.
        self.inner.lock().unwrap_or_else(|p| p.into_inner())
    }
}

#[uniffi::export]
impl LvhvStore {
    /// Mở (hoặc tạo) kho trong thư mục `dir` (tạo `lvhv.sqlite` và `photos/`), chuyển đổi phiên bản nếu cần.
    #[uniffi::constructor]
    pub fn open(dir: String) -> R<Arc<Self>> {
        Ok(Arc::new(LvhvStore {
            inner: Mutex::new(Store::open(dir)?),
        }))
    }

    /// `AppSettings` (01 §3.9).
    pub fn settings_json(&self) -> R<String> {
        Ok(to_text(&self.lock().settings()?))
    }

    /// Ghi `AppSettings` (đủ trường, kiểm V1/V8).
    pub fn put_settings_json(&self, json: String) -> R<()> {
        Ok(self.lock().put_settings(&parse_json(&json)?)?)
    }

    /// Mọi `Person` (mảng JSON, sắp theo id).
    pub fn persons_json(&self) -> R<String> {
        Ok(arr_text(self.lock().persons()?))
    }

    /// Một `Person`.
    pub fn person_json(&self, id: String) -> R<String> {
        Ok(to_text(&self.lock().person(&id)?))
    }

    /// Lưu một `Person` (thêm hoặc sửa): tính lại trường dẫn xuất (ngày âm/dương của ngày mất và ngày
    /// sinh, Can Chi, hưởng thọ/hưởng dương trừ khi `tuoi.manual`), gắn id/`created_at`/`updated_at`/`rev`,
    /// kiểm V1–V8 rồi ghi. Trả bản đã lưu.
    pub fn save_person_json(&self, json: String) -> R<String> {
        let mut p = lvhv_store::build_person(&parse_json(&json)?)?;
        let mut st = self.lock();
        let old = match p["id"].as_str() {
            Some(id) if !id.is_empty() => match st.person(id) {
                Ok(x) => Some(x),
                Err(StoreError::NotFound) => None,
                Err(e) => return Err(e.into()),
            },
            _ => None,
        };
        stamp(&mut p, old.as_ref())?;
        st.put_person(&p)?;
        Ok(to_text(&st.person(p["id"].as_str().unwrap_or_default())?))
    }

    /// Xóa một người (gỡ id khỏi mọi sự kiện, xóa ảnh không còn ai dùng).
    pub fn delete_person(&self, id: String) -> R<()> {
        Ok(self.lock().delete_person(&id)?)
    }

    /// Mọi `Event` (mảng JSON, sắp theo id).
    pub fn events_json(&self) -> R<String> {
        Ok(arr_text(self.lock().events()?))
    }

    /// Lưu một `Event` (thêm hoặc sửa); gắn id/`created_at`/`updated_at`/`rev`, kiểm rồi ghi. Trả bản đã lưu.
    pub fn save_event_json(&self, json: String) -> R<String> {
        let mut e = parse_json(&json)?;
        let mut st = self.lock();
        let old = match e["id"].as_str() {
            Some(id) => find_by_id(st.events()?, id),
            None => None,
        };
        stamp(&mut e, old.as_ref())?;
        st.put_event(&e)?;
        let id = e["id"].as_str().unwrap_or_default().to_owned();
        find_by_id(st.events()?, &id)
            .map(|x| to_text(&x))
            .ok_or_else(|| LvhvDataError::NotFound("Không tìm thấy bản ghi.".into()))
    }

    /// Xóa một sự kiện.
    pub fn delete_event(&self, id: String) -> R<()> {
        Ok(self.lock().delete_event(&id)?)
    }

    /// Mọi `EventType` (mảng JSON).
    pub fn event_types_json(&self) -> R<String> {
        Ok(arr_text(self.lock().event_types()?))
    }

    /// Lưu một `EventType` (tên duy nhất — V7); gắn id/thời điểm/rev. Trả bản đã lưu.
    pub fn save_event_type_json(&self, json: String) -> R<String> {
        let mut t = parse_json(&json)?;
        let mut st = self.lock();
        let old = match t["id"].as_str() {
            Some(id) => find_by_id(st.event_types()?, id),
            None => None,
        };
        stamp(&mut t, old.as_ref())?;
        st.put_event_type(&t)?;
        let id = t["id"].as_str().unwrap_or_default().to_owned();
        find_by_id(st.event_types()?, &id)
            .map(|x| to_text(&x))
            .ok_or_else(|| LvhvDataError::NotFound("Không tìm thấy bản ghi.".into()))
    }

    /// Thêm ảnh chân dung (byte **đã** được nền tảng giải mã và mã hóa lại, không còn EXIF — 01 §3.8).
    /// Trả bản ghi `Photo`; gắn vào người bằng `photo_id` rồi `save_person_json`.
    pub fn add_photo(
        &self,
        id: String,
        mime: String,
        width: u32,
        height: u32,
        bytes: Vec<u8>,
    ) -> R<String> {
        Ok(to_text(
            &self.lock().add_photo(&id, &mime, width, height, &bytes)?,
        ))
    }

    /// Byte của một ảnh (kiểm `sha256` khi đọc).
    pub fn photo_bytes(&self, id: String) -> R<Vec<u8>> {
        Ok(self.lock().photo_bytes(&id)?)
    }

    /// Toàn bộ kho dạng tài liệu sao lưu v1 (01 §5, chưa có byte ảnh) — đầu vào `dataset_json` của các
    /// hàm ngày giỗ / nhắc / danh sách / .ics bên dưới.
    pub fn dataset_json(&self) -> R<String> {
        let app = AppInfo {
            version: env!("CARGO_PKG_VERSION").into(),
            platform: "native".into(),
        };
        Ok(to_text(
            &self.lock().snapshot(&app, &lvhv_store::now_rfc3339())?,
        ))
    }

    /// Có nên nhắc người dùng sao lưu không (01 §3.9 `backup_nag_days`).
    pub fn backup_reminder_due(&self) -> R<bool> {
        Ok(self
            .lock()
            .backup_reminder_due(&lvhv_store::now_rfc3339())?)
    }

    /// Xuất tệp sao lưu mã hóa (03 §2). `platform`: "android" | "ios" | "macos" | "windows";
    /// `app_version` dạng X.Y.Z. Ứng dụng ghi byte qua hộp thoại lưu tệp của hệ điều hành.
    pub fn export_backup(
        &self,
        password: String,
        app_version: String,
        platform: String,
    ) -> R<Vec<u8>> {
        let password = zeroize::Zeroizing::new(password);
        Ok(self.lock().export_backup(
            &password,
            &AppInfo {
                version: app_version,
                platform,
            },
        )?)
    }

    /// Ghi tệp đã kiểm tra (Thay toàn bộ / Gộp) trong một giao dịch; lỗi → không ghi gì.
    pub fn apply_import(&self, backup: Arc<VerifiedBackup>, mode: BackupImportMode) -> R<()> {
        let m = match mode {
            BackupImportMode::Replace => lvhv_store::ImportMode::Replace,
            BackupImportMode::Merge => lvhv_store::ImportMode::Merge,
        };
        self.lock()
            .apply_import(&backup.inner, m)
            .map_err(|e| match e {
                StoreError::Sql(_) | StoreError::Io(_) => e.into(),
                other => LvhvDataError::Backup(other.import_message().into()),
            })
    }
}

// ------------------------------------------------------------------------------------------------
// Dẫn xuất / kiểm tra hồ sơ
// ------------------------------------------------------------------------------------------------

/// Tính trường dẫn xuất của một `Person` (như `save_person_json`, không ghi) và kiểm V1–V8; trả bản
/// đã tính. Dùng để xem trước trong màn hình nhập. Lỗi nhập liệu có thông báo theo 01 §6.1.
#[uniffi::export]
pub fn derive_person_json(json: String) -> R<String> {
    let mut p = lvhv_store::build_person(&parse_json(&json)?)?;
    if p.get("created_at").is_none_or(Value::is_null) {
        stamp(&mut p, None)?;
    }
    lvhv_store::validate::person(&p, "/person")
        .map_err(|e| bad(format!("Dữ liệu không hợp lệ: {e}")))?;
    Ok(to_text(&p))
}

/// `save_death` (02 §1): `DateValue` của ngày mất → `{lunar, solar, lunar_source, method}`. Đổi dương → âm
/// (hoặc âm → dương) khi lưu.
#[uniffi::export]
pub fn save_death_json(date_value_json: String) -> R<String> {
    let m = lvhv_store::derive::save_death(&parse_json(&date_value_json)?).map_err(|e| bad(e.0))?;
    Ok(to_text(&Value::Object(m)))
}

/// Ngày sinh (02 §4.1): `DateValue` → `{lunar_year, lunar_year_approx, can_chi, lunar, solar}`.
#[uniffi::export]
pub fn derive_birth_json(date_value_json: String) -> R<String> {
    let m =
        lvhv_store::derive::derive_birth(&parse_json(&date_value_json)?).map_err(|e| bad(e.0))?;
    Ok(to_text(&Value::Object(m)))
}

/// Tóm tắt hiển thị của một `Person` (đã có trường dẫn xuất) tại ngày `today`:
/// `{"display_name","can_chi","lunar_year","tuoi_duong","tuoi_am","tuoi_mat_text","death_lunar_text"}`
/// (`null` khi không xác định). `tuoi_duong`/`tuoi_am` chỉ cho người đang sống (02 §4.3);
/// `tuoi_mat_text` ("Hưởng thọ 86 tuổi", "(khoảng)") cho người đã khuất (02 §4.2).
#[uniffi::export]
pub fn person_summary_json(person_json: String, today: SolarDay) -> R<String> {
    let p = parse_json(&person_json)?;
    let today = ymd(today)?;
    let ho_ten = p["ho_ten"].as_str().unwrap_or_default();
    let dx = p["danh_xung"].as_str().unwrap_or_default();
    let display = if dx.is_empty() {
        ho_ten.to_owned()
    } else {
        format!("{dx} {ho_ten}")
    };
    let b = &p["birth"];
    let lunar_year = b["lunar_year"].as_i64().map(|x| x as i32);
    let living = p["kind"] == "living";
    let feb = if p["feb29_rule"] == "mar1" {
        Feb29Rule::Mar1
    } else {
        Feb29Rule::Feb28
    };
    let tuoi_duong = match (&b["solar"], living) {
        (Value::Object(s), true) => {
            let g = |k: &str| {
                s.get(k)
                    .and_then(Value::as_i64)
                    .and_then(|x| i32::try_from(x).ok())
                    .unwrap_or(0)
            };
            ymd(SolarDay {
                y: g("y"),
                m: g("m"),
                d: g("d"),
            })
            .ok()
            .and_then(|birth| lvhv_memorial::tuoi_duong(birth, today, feb))
        }
        _ => None,
    };
    let tuoi_am = if living {
        lunar_year.and_then(|ly| lvhv_memorial::tuoi_am(cal(), ly, today))
    } else {
        None
    };
    let d = &p["death"];
    let tuoi_mat_text = if d.is_object() {
        let t = &d["tuoi"];
        lvhv_memorial::gio::tuoi_text(&lvhv_memorial::model::Tuoi {
            value: t["value"].as_i64().map(|x| x as i32),
            label: if t["label"] == "huong_duong" {
                lvhv_memorial::model::AgeLabel::HuongDuong
            } else {
                lvhv_memorial::model::AgeLabel::HuongTho
            },
            manual: t["manual"] == true,
            approx: t["approx"] == true,
        })
    } else {
        None
    };
    let death_lunar_text = if d.is_object() {
        let l = &d["lunar"];
        let (m, dd, leap) = (
            l["month"].as_i64().unwrap_or(0),
            l["day"].as_i64().unwrap_or(0),
            l["leap"] == true,
        );
        let base = format!("{dd}/{m}{} âm lịch", if leap { " nhuận" } else { "" });
        Some(match l["year"].as_i64() {
            Some(y) => format!("{base} năm {} ({y})", lvhv_memorial::can_chi_year(y as i32)),
            None => base,
        })
    } else {
        None
    };
    Ok(to_text(&json!({
        "display_name": display,
        "can_chi": b["can_chi"].clone(),
        "lunar_year": lunar_year,
        "tuoi_duong": tuoi_duong,
        "tuoi_am": tuoi_am,
        "tuoi_mat_text": tuoi_mat_text,
        "death_lunar_text": death_lunar_text,
    })))
}

// ------------------------------------------------------------------------------------------------
// Danh sách "Sắp tới" (02 §7.2) và danh sách theo kỳ (02 §7.6)
// ------------------------------------------------------------------------------------------------

fn class_text(c: OccClass) -> &'static str {
    match c {
        OccClass::Gio => "Ngày giỗ",
        OccClass::TienThuong => "Tiên thường",
        OccClass::SinhNhat => "Sinh nhật",
        OccClass::SinhNhatAm => "Sinh nhật âm lịch",
        _ => "Sự kiện",
    }
}

fn occ_json(ds: &Dataset, o: &Occurrence) -> Value {
    item_json(&period::period_item(cal(), ds, o.clone()))
}

fn item_json(it: &period::PeriodItem) -> Value {
    let o = &it.occ;
    json!({
        "date": o.date.iso(),
        "lunar": it.lunar.map(|l| json!({"y": l.y, "m": l.m, "d": l.d, "leap": l.leap})),
        "class": o.class.code(),
        "class_text": class_text(o.class),
        "entity_id": o.entity_id,
        "person_ids": it.person_ids,
        "name": it.name,
        "key": o.key.text(),
        "n": o.n,
        "label": match o.class {
            OccClass::Gio | OccClass::TienThuong => o.gio_label.map(|l| l.code()),
            _ => None,
        },
        "headline": o.headline,
        "date_line": date_line(cal(), o.date),
    })
}

/// Danh sách "Sắp tới" (02 §7.2): mọi lần xuất hiện trong `[today, today + days]` (gồm hai đầu), đúng
/// thứ tự hiển thị. Mảng `[{"days","days_text","date","lunar","class","class_text","entity_id",
/// "person_ids","name","key","n","label","headline","date_line"}]`. Không phụ thuộc công tắc nhắc.
#[uniffi::export]
pub fn upcoming_json(dataset_json: String, today: SolarDay, days: u32) -> R<String> {
    upcoming_impl(&dataset(&dataset_json)?, today, days)
}

fn upcoming_impl(ds: &Dataset, today: SolarDay, days: u32) -> R<String> {
    let list: Vec<Value> = upcoming(cal(), ds, ymd(today)?, days.min(3660))
        .iter()
        .map(|it| {
            let mut v = occ_json(ds, &it.occ);
            v["days"] = json!(it.days);
            v["days_text"] = json!(days_text(it.days));
            v
        })
        .collect();
    Ok(to_text(&Value::Array(list)))
}

/// Biên một kỳ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct PeriodRange {
    pub from: SolarDay,
    pub to: SolarDay,
}

fn kind_of(kind: &str) -> R<PeriodKind> {
    PeriodKind::parse(kind)
        .ok_or_else(|| bad("Kỳ phải là \"tuan\", \"thang\", \"quy\" hoặc \"nam\"."))
}

fn range((f, t): (Ymd, Ymd)) -> PeriodRange {
    PeriodRange {
        from: solar_day(f),
        to: solar_day(t),
    }
}

/// Biên kỳ chứa `anchor` (02 §7.6): `kind` ∈ "tuan" (Thứ Hai → Chủ nhật), "thang", "quy", "nam".
#[uniffi::export]
pub fn period_bounds(kind: String, anchor: SolarDay) -> R<PeriodRange> {
    Ok(range(period::period_bounds(kind_of(&kind)?, ymd(anchor)?)))
}

/// Kỳ liền trước kỳ chứa `anchor`.
#[uniffi::export]
pub fn period_prev(kind: String, anchor: SolarDay) -> R<PeriodRange> {
    Ok(range(period::prev_period(kind_of(&kind)?, ymd(anchor)?)))
}

/// Kỳ liền sau kỳ chứa `anchor`.
#[uniffi::export]
pub fn period_next(kind: String, anchor: SolarDay) -> R<PeriodRange> {
    Ok(range(period::next_period(kind_of(&kind)?, ymd(anchor)?)))
}

/// `events_in_period` (02 §7.6, R13). `filter` ∈ "tat_ca", "da_mat", "dang_song". JSON
/// `{"kind","anchor","filter","from","to","items":[…]}`; mỗi mục có các trường của 02 Phụ lục A.6
/// (`date, lunar, class, entity_id, person_ids, name, key, n, label`) cộng `class_text`, `headline`,
/// `date_line` để hiển thị.
#[uniffi::export]
pub fn events_in_period_json(
    dataset_json: String,
    kind: String,
    anchor: SolarDay,
    filter: String,
) -> R<String> {
    events_in_period_impl(&dataset(&dataset_json)?, &kind, anchor, &filter)
}

fn events_in_period_impl(ds: &Dataset, kind: &str, anchor: SolarDay, filter: &str) -> R<String> {
    let f = PersonFilter::parse(filter)
        .ok_or_else(|| bad("Bộ lọc phải là \"tat_ca\", \"da_mat\" hoặc \"dang_song\"."))?;
    let p = period::events_in_period(cal(), ds, kind_of(kind)?, ymd(anchor)?, f);
    let items: Vec<Value> = p.items.iter().map(item_json).collect();
    Ok(to_text(&json!({
        "kind": p.kind.code(),
        "anchor": p.anchor.iso(),
        "filter": p.filter.code(),
        "from": p.from.iso(),
        "to": p.to.iso(),
        "items": items,
    })))
}

// ------------------------------------------------------------------------------------------------
// Lời nhắc (02 §7.1), xoay vòng 64 (02 §7.4), chuỗi báo thức Android (02 §7.3)
// ------------------------------------------------------------------------------------------------

fn rtype_code(r: &Reminder) -> String {
    match r.rtype {
        lvhv_memorial::RemType::Offset(k) => format!("k={k}"),
        lvhv_memorial::RemType::TienThuong => "tt".into(),
    }
}

/// Một lời nhắc dạng JSON. `show_names = false` (O-9): `title`/`body` không có tên **và** `occ` chỉ gồm
/// `entity_id`, `class`, `date`, `key` (đủ để mở đúng màn hình khi chạm), không có `name`, `headline`,
/// `date_line`, `person_ids`, `lunar`. Nội dung thông báo chỉ lấy từ `title`/`body`.
fn reminder_json(ds: &Dataset, r: &Reminder, show_names: bool) -> Value {
    let t = notification_text(cal(), &r.occ, r.rtype, show_names);
    let occ = if show_names {
        occ_json(ds, &r.occ)
    } else {
        json!({
            "entity_id": r.occ.entity_id,
            "class": r.occ.class.code(),
            "date": r.occ.date.iso(),
            "key": r.occ.key.text(),
        })
    };
    json!({
        "id": r.id,
        "at_unix_s": unix_of_min(r.at),
        "fire_local": fmt_local(r.at_local),
        "type": rtype_code(r),
        "title": t.title,
        "body": t.body,
        "occ": occ,
    })
}

/// Lời nhắc tương lai (`at > now`) trong `[now, now + horizon_days]`.
fn future_reminders(ds: &Dataset, now: i64, tz: &Zone, horizon_days: i64) -> Vec<Reminder> {
    let today = Ymd::from_jdn(tz.to_local(now).div_euclid(1440));
    let to = Ymd::from_jdn(today.jdn() + horizon_days + 61);
    let hi = now + horizon_days * 1440;
    all_reminders(cal(), ds, today, to, now, tz)
        .into_iter()
        .filter(|r| r.at <= hi)
        .collect()
}

fn reminders_impl(
    ds: &Dataset,
    now_unix_s: i64,
    zone: &DeviceZone,
    horizon_days: u32,
    show_names: bool,
) -> R<String> {
    let now = min_of_unix(unix_ok(now_unix_s, "now_unix_s")?);
    let tz = self::zone(zone)?;
    let list: Vec<Value> = future_reminders(ds, now, &tz, i64::from(horizon_days.min(3660)))
        .iter()
        .map(|r| reminder_json(ds, r, show_names))
        .collect();
    Ok(to_text(&Value::Array(list)))
}

/// Mọi lời nhắc (02 §7.1) có thời điểm trong `(now, now + horizon_days ngày]`, sắp theo (thời điểm, id):
/// `[{"id","at_unix_s","fire_local","type","title","body","occ"}]`. `show_names` mặc định `false` (O-9) →
/// tiêu đề không có tên ("Sắp đến ngày giỗ" …) và `occ` rút gọn (xem `reminder_json`). Tôn trọng công tắc
/// chung và `rule.enabled`. Ứng dụng native nên gọi `LvhvStore.reminders_json` (đọc tùy chọn O-9 từ kho).
#[uniffi::export(default(show_names = false))]
pub fn reminders_json(
    dataset_json: String,
    now_unix_s: i64,
    zone: DeviceZone,
    horizon_days: u32,
    show_names: bool,
) -> R<String> {
    reminders_impl(
        &dataset(&dataset_json)?,
        now_unix_s,
        &zone,
        horizon_days,
        show_names,
    )
}

fn pending_value(p: &Pending, tz: &Zone) -> Value {
    let (title, body) = match p.content.split_once('\n') {
        Some((a, b)) => (a.to_owned(), b.to_owned()),
        None => (p.content.clone(), String::new()),
    };
    json!({
        "id": p.id,
        "at_unix_s": unix_of_min(p.at),
        "fire_local": fmt_local(tz.to_local(p.at)),
        "title": title,
        "body": body,
        "content": p.content,
    })
}

fn reschedule_impl(
    ds: &Dataset,
    now_unix_s: i64,
    zone: &DeviceZone,
    pending_json: &str,
    show_names: bool,
) -> R<String> {
    let now = min_of_unix(unix_ok(now_unix_s, "now_unix_s")?);
    let tz = self::zone(zone)?;
    let pend_v = parse_json(pending_json)?;
    let pending: Vec<Pending> = pend_v
        .as_array()
        .ok_or_else(|| bad("pending_json phải là mảng."))?
        .iter()
        .map(|x| {
            Ok(Pending {
                id: x["id"]
                    .as_str()
                    .ok_or_else(|| bad("pending_json: thiếu id."))?
                    .to_owned(),
                at: min_of_unix(unix_ok(
                    x["at_unix_s"]
                        .as_i64()
                        .ok_or_else(|| bad("pending_json: thiếu at_unix_s."))?,
                    "pending_json.at_unix_s",
                )?),
                content: x["content"].as_str().unwrap_or_default().to_owned(),
            })
        })
        .collect::<R<_>>()?;
    let desired: Vec<Pending> = reschedule_input(cal(), ds, now, &tz)
        .iter()
        .map(|r| to_pending(cal(), r, show_names))
        .collect();
    let plan = reschedule(now, &desired, &pending);
    let list = |v: &[Pending]| -> Vec<Value> { v.iter().map(|p| pending_value(p, &tz)).collect() };
    Ok(to_text(&json!({
        "keep": list(&plan.keep),
        "remove": plan.remove,
        "add": list(&plan.add),
        "replace": list(&plan.replace),
    })))
}

/// `reschedule(now)` (02 §7.4) cho iOS/macOS (`UNUserNotificationCenter`) và Windows
/// (`ScheduledToastNotification`), giới hạn 64 yêu cầu chờ. `pending_json`: các yêu cầu đang chờ do hệ
/// điều hành trả về, `[{"id","at_unix_s","content"}]` (`content` = `title + "\n" + body` đã đặt lần trước;
/// thiếu → ""). Kết quả `{"keep":[P],"remove":[id],"add":[P],"replace":[P]}`, `P` =
/// `{"id","at_unix_s","fire_local","title","body","content"}`; ứng dụng gỡ `remove`, thêm `add`, thay
/// `replace` (trigger theo `fire_local`, không gắn múi giờ). Còn lời nhắc không đặt được (> 64 trong cửa
/// sổ 400 ngày, hoặc ở ngoài cửa sổ) → tối đa 63 sớm nhất + thông báo nhắc mở ứng dụng
/// (`id = "lvhv:sentinel"`) cùng phút với cái cuối cùng đã đặt (không có cái nào: tại now + 400 ngày) —
/// 02 §7.4, Bổ sung B3. `show_names` mặc định `false` (O-9); nên gọi `LvhvStore.reschedule_json`.
#[uniffi::export(default(show_names = false))]
pub fn reschedule_json(
    dataset_json: String,
    now_unix_s: i64,
    zone: DeviceZone,
    pending_json: String,
    show_names: bool,
) -> R<String> {
    reschedule_impl(
        &dataset(&dataset_json)?,
        now_unix_s,
        &zone,
        &pending_json,
        show_names,
    )
}

/// Thông báo nhắc mở ứng dụng (02 §7.4).
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct SentinelInfo {
    /// `"lvhv:sentinel"`.
    pub id: String,
    /// Nội dung thông báo.
    pub text: String,
}

/// Mã và nội dung của thông báo nhắc mở ứng dụng (02 §7.4).
#[uniffi::export]
pub fn sentinel_info() -> SentinelInfo {
    SentinelInfo {
        id: SENTINEL_ID.to_string(),
        text: SENTINEL_TEXT.to_string(),
    }
}

fn android_step_impl(
    ds: &Dataset,
    now_unix_s: i64,
    last_run_unix_s: Option<i64>,
    zone: &DeviceZone,
    show_names: bool,
) -> R<String> {
    let now = min_of_unix(unix_ok(now_unix_s, "now_unix_s")?);
    let last = match last_run_unix_s {
        Some(l) => Some(min_of_unix(unix_ok(l, "last_run_unix_s")?)),
        None => None,
    };
    let tz = self::zone(zone)?;
    let late_lo = now - 24 * 60;
    // Phần đã đến hạn: từ lần chạy trước (nếu cũ hơn 24 giờ thì từ đó để đếm phần bỏ), không quá 400 ngày.
    let since = last
        .unwrap_or(late_lo)
        .min(late_lo)
        .max(now - WINDOW_DAYS * 1440);
    let from_day = Ymd::from_jdn(tz.to_local(since).div_euclid(1440));
    let today = Ymd::from_jdn(tz.to_local(now).div_euclid(1440));
    let due_to = Ymd::from_jdn(today.jdn() + 61);
    let due = all_reminders(cal(), ds, from_day, due_to, since, &tz);
    let lo = last.map_or(late_lo, |l| l.max(late_lo));
    let mut post = Vec::new();
    let mut dropped = 0u32;
    for r in due.iter().filter(|r| r.at <= now) {
        if r.at > lo {
            post.push(reminder_json(ds, r, show_names));
        } else if last.is_some_and(|l| r.at > l) {
            dropped += 1;
        }
    }
    // Báo thức kế tiếp: lời nhắc sớm nhất sau `now`, **không** giới hạn cửa sổ (MA3-14-01) — cùng tập
    // `reschedule_input` của 02 §7.4 (quét +900 ngày cho mục lặp, sự kiện một lần tới 3000-12-31).
    let next = reschedule_input(cal(), ds, now, &tz).into_iter().next();
    Ok(to_text(&json!({
        "post": post,
        "dropped_late": dropped,
        "next_at_unix_s": next.as_ref().map(|r| unix_of_min(r.at)),
        "next_fire_local": next.as_ref().map(|r| fmt_local(r.at_local)),
    })))
}

/// Một bước của chuỗi báo thức Android (02 §7.3): gọi khi báo thức chạy, khi khởi động lại, đổi giờ/múi
/// giờ, mở ứng dụng, lưu dữ liệu, nhập sao lưu. `last_run_unix_s`: lần gọi trước (đã đăng thông báo tới
/// thời điểm đó; `null` = chưa có). Kết quả
/// `{"post":[R],"dropped_late":n,"next_at_unix_s":t|null,"next_fire_local":s|null}`:
/// `post` = lời nhắc có thời điểm trong `(max(last_run, now − 24 giờ), now]` (đăng ngay; trễ hơn 24 giờ
/// thì bỏ và đếm ở `dropped_late`), `next_*` = lời nhắc sớm nhất sau `now`, **không giới hạn cửa sổ**
/// (đặt **một** báo thức `setAndAllowWhileIdle(RTC_WAKEUP, next_at_unix_s × 1000)`); `null` chỉ khi không
/// còn lời nhắc tương lai nào. `R` như `reminders_json`. `show_names` mặc định `false` (O-9); nên gọi
/// `LvhvStore.android_alarm_step_json`.
#[uniffi::export(default(show_names = false))]
pub fn android_alarm_step_json(
    dataset_json: String,
    now_unix_s: i64,
    last_run_unix_s: Option<i64>,
    zone: DeviceZone,
    show_names: bool,
) -> R<String> {
    android_step_impl(
        &dataset(&dataset_json)?,
        now_unix_s,
        last_run_unix_s,
        &zone,
        show_names,
    )
}

/// Các hàm danh sách / nhắc đọc thẳng từ kho; tùy chọn O-9 lấy từ kho (`lock_screen_names`, mặc định
/// không hiện tên), người gọi không truyền.
#[uniffi::export]
impl LvhvStore {
    /// Tùy chọn "hiện tên trên màn hình khóa" (O-9), mặc định `false`. Lưu cục bộ trên thiết bị, không
    /// đi vào tệp sao lưu.
    pub fn lock_screen_names(&self) -> R<bool> {
        Ok(self.lock().lock_screen_names()?)
    }

    /// Đặt tùy chọn "hiện tên trên màn hình khóa" (O-9).
    pub fn set_lock_screen_names(&self, show: bool) -> R<()> {
        Ok(self.lock().set_lock_screen_names(show)?)
    }

    /// `upcoming_json` trên dữ liệu của kho.
    pub fn upcoming_json(&self, today: SolarDay, days: u32) -> R<String> {
        upcoming_impl(&self.dataset()?, today, days)
    }

    /// `events_in_period_json` trên dữ liệu của kho.
    pub fn events_in_period_json(
        &self,
        kind: String,
        anchor: SolarDay,
        filter: String,
    ) -> R<String> {
        events_in_period_impl(&self.dataset()?, &kind, anchor, &filter)
    }

    /// `reminders_json` trên dữ liệu của kho, tùy chọn O-9 của kho.
    pub fn reminders_json(
        &self,
        now_unix_s: i64,
        zone: DeviceZone,
        horizon_days: u32,
    ) -> R<String> {
        let (ds, show) = self.dataset_and_names()?;
        reminders_impl(&ds, now_unix_s, &zone, horizon_days, show)
    }

    /// `reschedule_json` trên dữ liệu của kho, tùy chọn O-9 của kho.
    pub fn reschedule_json(
        &self,
        now_unix_s: i64,
        zone: DeviceZone,
        pending_json: String,
    ) -> R<String> {
        let (ds, show) = self.dataset_and_names()?;
        reschedule_impl(&ds, now_unix_s, &zone, &pending_json, show)
    }

    /// `android_alarm_step_json` trên dữ liệu của kho, tùy chọn O-9 của kho.
    pub fn android_alarm_step_json(
        &self,
        now_unix_s: i64,
        last_run_unix_s: Option<i64>,
        zone: DeviceZone,
    ) -> R<String> {
        let (ds, show) = self.dataset_and_names()?;
        android_step_impl(&ds, now_unix_s, last_run_unix_s, &zone, show)
    }
}

impl LvhvStore {
    fn dataset(&self) -> R<Dataset> {
        dataset(&self.dataset_json()?)
    }
    fn dataset_and_names(&self) -> R<(Dataset, bool)> {
        let show = self.lock().lock_screen_names()?;
        Ok((self.dataset()?, show))
    }
}

// ------------------------------------------------------------------------------------------------
// .ics (02 §8)
// ------------------------------------------------------------------------------------------------

/// Tệp .ics đã sinh.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct IcsExport {
    /// `LichVanHoaViet-ngay-gio-<YYYY-MM-DD>.ics`.
    pub file_name: String,
    /// Nội dung (UTF-8, CRLF); kiểu `text/calendar; charset=utf-8`.
    pub content: String,
    /// Số VEVENT.
    pub events: u32,
    /// Số người khác nhau có tên trong tệp (cho hộp xác nhận S10: "Tệp này chứa tên và ngày giỗ của … người").
    pub persons: u32,
}

/// Sinh tệp .ics 10 năm từ `today` (02 §8). `only_ids`: chỉ xuất các hồ sơ/sự kiện này; `null` = mặc
/// định (mọi mục có quy tắc nhắc bật). `now_unix_s` cho `DTSTAMP`.
#[uniffi::export]
pub fn ics_export(
    dataset_json: String,
    today: SolarDay,
    now_unix_s: i64,
    only_ids: Option<Vec<String>>,
) -> R<IcsExport> {
    let ds = dataset(&dataset_json)?;
    let today = ymd(today)?;
    let stamp = lvhv_store::rfc3339_from_unix(unix_ok(now_unix_s, "now_unix_s")?); // YYYY-MM-DDTHH:MM:SSZ
    let dtstamp: String = stamp.chars().filter(|c| *c != '-' && *c != ':').collect();
    let opt = lvhv_memorial::ics::IcsOptions {
        today,
        dtstamp,
        only_ids,
    };
    let occs = lvhv_memorial::ics::ics_occurrences(cal(), &ds, &opt);
    let mut persons: Vec<&str> = occs
        .iter()
        .flat_map(|(o, _)| -> Vec<&str> {
            if o.class.is_event() {
                ds.events
                    .iter()
                    .find(|e| e.id == o.entity_id)
                    .map(|e| e.person_ids.iter().map(String::as_str).collect())
                    .unwrap_or_default()
            } else {
                vec![o.entity_id.as_str()]
            }
        })
        .collect();
    persons.sort_unstable();
    persons.dedup();
    Ok(IcsExport {
        file_name: lvhv_memorial::ics::file_name(today),
        content: lvhv_memorial::ics::generate(cal(), &ds, &opt),
        events: occs.len() as u32,
        persons: persons.len() as u32,
    })
}
