//! Mô-đun "Người thân" (v0.2 đợt 3) cho Windows: nối giao diện `ui/people.slint` với lõi Rust qua
//! `lvhv-ffi` (gọi hàm Rust trực tiếp, không qua UniFFI): kho SQLite `lvhv-store`, ngày giỗ / nhắc / danh sách
//! theo kỳ / .ics của `lvhv-memorial`, sao lưu mã hóa `lvhv-backup`. Giao diện không tự tính thuật toán nào.
//!
//! Đặc tả: docs/v0.2/08-DAC-TA-GIAO-DIEN-NGUOI-THAN.md (dùng chung cho mọi nền tảng). Sai khác của bản
//! Windows ghi ở docs/04-BUILD-NATIVE.md §"Người thân trên Windows".

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use lvhv_ffi::{BackupImportMode, DeviceZone, LvhvStore, VerifiedBackup};
use serde_json::{json, Value};
use slint::{ComponentHandle, Image, Model, ModelRc, SharedString, Timer, VecModel};


use crate::platform::{self, Dialog};
use crate::toasts::{self, SyncResult};
use crate::photo;
use crate::ui::*;
use crate::{sv, D, WEEKDAY};

/// Mốc nhắc nhanh (01 §3.6): đúng ngày, 1, 3, 5, 10, 15 ngày trước.
pub const QUICK: [i64; 6] = [0, 1, 3, 5, 10, 15];
const WD_SHORT: [&str; 7] = ["CN", "T2", "T3", "T4", "T5", "T6", "T7"];
/// Giới hạn kích thước tệp sao lưu (03 §2.3): phần đầu 72 byte + 128 MiB + thẻ 16 byte.
const BACKUP_MAX: u64 = 72 + 128 * 1024 * 1024 + 16;

fn ss(s: impl AsRef<str>) -> SharedString {
    sv(s.as_ref())
}

fn vm<T: Clone + 'static>(v: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(v))
}

fn jv(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or(Value::Null)
}

fn st(v: &Value) -> String {
    v.as_str().unwrap_or_default().to_string()
}

fn upper_first(s: &str) -> String {
    crate::upper_first(s)
}

/// Ngày "YYYY-MM-DD" → D.
fn d_iso(s: &str) -> Option<D> {
    let p: Vec<&str> = s.split('-').collect();
    (p.len() == 3).then(|| D {
        y: p[0].parse().unwrap_or(0),
        m: p[1].parse().unwrap_or(0),
        d: p[2].parse().unwrap_or(0),
    })
}

fn iso(d: D) -> String {
    format!("{:04}-{:02}-{:02}", d.y, d.m, d.d)
}

/// Ngày hôm nay theo đồng hồ và múi giờ của máy.
pub fn local_today() -> D {
    let secs = platform::now_unix() + platform::utc_offset_min() as i64 * 60;
    let days = secs.div_euclid(86_400);
    D { y: 1970, m: 1, d: 1 }
        .add(days as i32)
        .unwrap_or(D { y: 2026, m: 1, d: 1 })
}

/// Múi giờ thiết bị kèm các lần chuyển giờ mùa hè trong 401 ngày tới (MA3N-25-05).
pub fn zone() -> DeviceZone {
    platform::device_zone()
}

/// Ngày âm dạng "15/6 nhuận âm lịch" (+ " năm Ất Tỵ (2025)" khi có năm và `with_year`).
fn lunar_text(l: &Value, with_year: bool) -> String {
    let (m, d) = (
        l["month"].as_i64().or(l["m"].as_i64()).unwrap_or(0),
        l["day"].as_i64().or(l["d"].as_i64()).unwrap_or(0),
    );
    let leap = if l["leap"] == true { " nhuận" } else { "" };
    let y = l["year"].as_i64().or(l["y"].as_i64());
    match (with_year, y) {
        (true, Some(y)) => format!(
            "{d}/{m}{leap} âm lịch năm {} ({y})",
            lvhv_ffi::year_can_chi(y as i32)
        ),
        _ => format!("{d}/{m}{leap} âm lịch"),
    }
}

fn dmy_v(s: &Value) -> String {
    format!(
        "{:02}/{:02}/{}",
        s["d"].as_i64().unwrap_or(0),
        s["m"].as_i64().unwrap_or(0),
        s["y"].as_i64().unwrap_or(0)
    )
}

/// Ngày người dùng đã nhập (`DateValue`) dạng chữ.
fn date_value_text(v: &Value) -> String {
    let lunar = v["calendar"] == "lunar";
    let (y, m, d) = (v["year"].as_i64(), v["month"].as_i64(), v["day"].as_i64());
    match (v["precision"].as_str().unwrap_or(""), lunar) {
        ("year", false) => format!("Năm {}", y.unwrap_or(0)),
        ("year", true) => format!(
            "Năm âm {} ({})",
            y.unwrap_or(0),
            lvhv_ffi::year_can_chi(y.unwrap_or(0) as i32)
        ),
        (_, false) => format!(
            "{:02}/{:02}/{}",
            d.unwrap_or(0),
            m.unwrap_or(0),
            y.unwrap_or(0)
        ),
        (_, true) => lunar_text(
            &json!({"year": y, "month": m, "day": d, "leap": v["leap"]}),
            true,
        ),
    }
}

fn offsets_text(rule: &Value) -> String {
    let offs: Vec<i64> = rule["offsets_days"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_i64).collect())
        .unwrap_or_default();
    if offs.is_empty() {
        return "chưa chọn mốc nhắc nào".into();
    }
    let parts: Vec<String> = offs
        .iter()
        .map(|&k| {
            if k == 0 {
                "Đúng ngày".to_string()
            } else {
                format!("{k} ngày trước")
            }
        })
        .collect();
    format!(
        "{}, lúc {}",
        parts.join(", "),
        rule["time_local"].as_str().unwrap_or("07:00")
    )
}

fn kind_of_class(c: &str) -> (i32, &'static str) {
    match c {
        "gio" => (0, "Ngày giỗ"),
        "tien_thuong" => (1, "Tiên thường"),
        "sinh_nhat" => (2, "Sinh nhật"),
        "sinh_nhat_am" => (2, "Sinh nhật âm"),
        _ => (3, "Sự kiện"),
    }
}

// ------------------------------------------------------------------------------------------------
// Trạng thái
// ------------------------------------------------------------------------------------------------

#[derive(Clone)]
enum PhotoSel {
    /// Giữ ảnh đang có (id hoặc không ảnh).
    Keep(Option<String>),
    /// Ảnh mới đã mã hóa lại, chưa lưu.
    New(photo::Encoded),
}

#[derive(Clone)]
struct CertPhoto {
    id: Option<String>,
    enc: Option<photo::Encoded>,
    img: Image,
}

#[derive(Clone)]
enum FormKind {
    None,
    Person { dead: bool, base: Option<Value> },
    Event { person: String, base: Option<Value> },
    Mark { person: Value },
}

enum DlgAction {
    None,
    DeletePerson(String),
    DeleteEvent(String),
    Ics(String, String),
    CreateBackup,
    Restore,
    ReplaceConfirm,
    Discard,
    Mark(Value, Option<Value>),
}

pub struct PState {
    pub store: Option<Arc<LvhvStore>>,
    pub dir: PathBuf,
    stack: Vec<String>,
    pkind: usize,
    panchor: D,
    pfilter: usize,
    person: Option<String>,
    memorial_back: String,
    form: FormKind,
    photo: PhotoSel,
    certs: Vec<CertPhoto>,
    offs: HashMap<&'static str, Vec<bool>>,
    tried: bool,
    dirty: bool,
    filling: bool,
    hidden_cards: HashSet<String>,
    backup: Option<Arc<VerifiedBackup>>,
    backup_file: Option<PathBuf>,
    dlg: DlgAction,
    toast_gen: Rc<std::cell::Cell<u64>>,
    smoke_gen: Rc<std::cell::Cell<u64>>,
    pub last_sync: Option<SyncResult>,
    /// Lớp thông báo của hệ điều hành (kiểm thử thay bằng `toasts::FakeToasts`).
    pub toasts: Box<dyn toasts::Toasts>,
    pub last_error: Option<String>,
}

impl PState {
    pub fn new(dir: PathBuf) -> Self {
        PState {
            store: None,
            dir,
            stack: vec![],
            pkind: 1,
            panchor: local_today(),
            pfilter: 0,
            person: None,
            memorial_back: "nguoi".into(),
            form: FormKind::None,
            photo: PhotoSel::Keep(None),
            certs: vec![],
            offs: HashMap::new(),
            tried: false,
            dirty: false,
            filling: false,
            hidden_cards: HashSet::new(),
            backup: None,
            backup_file: None,
            dlg: DlgAction::None,
            toast_gen: Rc::default(),
            smoke_gen: Rc::default(),
            last_sync: None,
            toasts: toasts::system(),
            last_error: None,
        }
    }

    fn s(&self) -> Option<&Arc<LvhvStore>> {
        self.store.as_ref()
    }
}

pub type Ps = Rc<RefCell<PState>>;

/// Mở kho nếu chưa mở. Lỗi → khung đỏ "Không mở được dữ liệu trên máy" (lịch vẫn dùng bình thường).
pub fn ensure_store(ui: &AppWindow, ps: &mut PState) -> bool {
    if ps.store.is_some() {
        return true;
    }
    let p = People::get(ui);
    match LvhvStore::open(ps.dir.to_string_lossy().into_owned()) {
        Ok(s) => {
            ps.store = Some(s);
            p.set_store_error(ss(""));
            true
        }
        Err(e) => {
            p.set_store_error(ss(format!(
                "Không mở được dữ liệu trên máy: {e}. Lịch vẫn dùng được bình thường."
            )));
            false
        }
    }
}

fn toast(ui: &AppWindow, ps: &PState, msg: &str) {
    People::get(ui).set_toast(ss(msg));
    let w = ui.as_weak();
    let gen = ps.toast_gen.get() + 1;
    ps.toast_gen.set(gen);
    let g = ps.toast_gen.clone();
    Timer::single_shot(Duration::from_secs(6), move || {
        if g.get() == gen {
            if let Some(ui) = w.upgrade() {
                People::get(&ui).set_toast(ss(""));
            }
        }
    });
}

fn fail(ui: &AppWindow, ps: &mut PState, msg: String) {
    ps.last_error = Some(msg.clone());
    let p = People::get(ui);
    p.set_f_summary(ss(msg));
}

// ------------------------------------------------------------------------------------------------
// Mục sự kiện, danh sách
// ------------------------------------------------------------------------------------------------

struct Summaries {
    tuoi: HashMap<String, String>,
    dead: HashSet<String>,
    names: HashMap<String, String>,
}

fn summaries(ps: &PState, today: D) -> Summaries {
    let mut s = Summaries {
        tuoi: HashMap::new(),
        dead: HashSet::new(),
        names: HashMap::new(),
    };
    let Some(store) = ps.s() else { return s };
    let persons = jv(&store.persons_json().unwrap_or_default());
    for p in persons.as_array().into_iter().flatten() {
        let id = st(&p["id"]);
        if p["kind"] == "deceased" {
            s.dead.insert(id.clone());
        }
        s.names.insert(id.clone(), display_name(p));
        if let Ok(j) = lvhv_ffi::person_summary_json(p.to_string(), today.solar()) {
            if let Some(t) = jv(&j)["tuoi_mat_text"].as_str() {
                s.tuoi.insert(id, t.to_string());
            }
        }
    }
    s
}

/// Một lần xuất hiện (JSON của lõi) → mục hiển thị.
fn item(o: &Value, today: D, sums: &Summaries) -> PItem {
    let date = d_iso(o["date"].as_str().unwrap_or("")).unwrap_or(today);
    let class = o["class"].as_str().unwrap_or("");
    let (kind, kind_text) = kind_of_class(class);
    let name = st(&o["name"]);
    let headline = st(&o["headline"]);
    let line1 = if class == "tien_thuong" && !name.is_empty() {
        format!("Tiên thường · {name}")
    } else if kind <= 2 && !name.is_empty() && headline.ends_with(&name) {
        format!(
            "{} · {name}",
            headline[..headline.len() - name.len()].trim_end()
        )
    } else if kind == 3 {
        // Sự kiện: "Nhãn · tên người" (người đầu tiên của sự kiện, chỉ để hiển thị).
        match o["person_ids"][0].as_str().and_then(|i| sums.names.get(i)) {
            Some(n) => format!("{headline} · {n}"),
            None => headline,
        }
    } else {
        headline
    };
    let pid = if kind <= 2 {
        st(&o["entity_id"])
    } else {
        o["person_ids"][0].as_str().unwrap_or("").to_string()
    };
    let days = date.jdn() - today.jdn();
    let right = match days {
        0 => "Hôm nay".to_string(),
        1 => "Ngày mai".to_string(),
        -1 => "Hôm qua".to_string(),
        n if n < 0 => format!("Đã qua {} ngày", -n),
        n => format!("Còn {n} ngày"),
    };
    let wd = date.weekday();
    let line3 = if kind == 0 {
        sums.tuoi.get(&pid).cloned().unwrap_or_default()
    } else {
        String::new()
    };
    let lunar = if o["lunar"].is_object() {
        format!(" · {}", lunar_text(&o["lunar"], false))
    } else {
        String::new()
    };
    PItem {
        day: ss(date.d.to_string()),
        wd: ss(WD_SHORT[wd % 7]),
        mon: ss(format!("thg {}", date.m)),
        line1: ss(line1),
        line2: ss(format!("{}{lunar}", date.dmy())),
        line3: ss(line3),
        kind,
        kind_text: ss(kind_text),
        right: ss(right),
        today: days == 0,
        past: days < 0,
        sunday: wd == 0,
        pid: ss(pid),
        header: ss(""),
        memorial: kind == 0 && days == 0,
    }
}

const PKIND: [&str; 4] = ["tuan", "thang", "quy", "nam"];
const PFILTER: [&str; 3] = ["tat_ca", "da_mat", "dang_song"];

fn period_title(kind: usize, f: D, t: D) -> String {
    let dm = |d: D| format!("{:02}/{:02}", d.d, d.m);
    match kind {
        0 if f.y == t.y => format!("Tuần {} – {}/{}", dm(f), dm(t), t.y),
        0 => format!("Tuần {} – {}", f.dmy(), t.dmy()),
        1 => format!("Tháng {}/{} · {} – {}", f.m, f.y, dm(f), dm(t)),
        2 => format!(
            "Quý {}/{} · {} – {}",
            ["I", "II", "III", "IV"][((f.m - 1) / 3).clamp(0, 3) as usize],
            f.y,
            dm(f),
            dm(t)
        ),
        _ => format!("Năm {} · {} – {}", f.y, dm(f), dm(t)),
    }
}

fn render_period(ui: &AppWindow, ps: &PState) {
    let p = People::get(ui);
    let Some(store) = ps.s() else { return };
    let today = local_today();
    let sums = summaries(ps, today);
    let r = store
        .events_in_period_json(
            PKIND[ps.pkind].into(),
            ps.panchor.solar(),
            PFILTER[ps.pfilter].into(),
        )
        .map(|j| jv(&j))
        .unwrap_or(Value::Null);
    let (f, t) = (
        d_iso(r["from"].as_str().unwrap_or("")).unwrap_or(today),
        d_iso(r["to"].as_str().unwrap_or("")).unwrap_or(today),
    );
    let mut items = Vec::new();
    let mut last_month = 0;
    for o in r["items"].as_array().into_iter().flatten() {
        let mut it = item(o, today, &sums);
        if ps.pkind >= 2 {
            let d = d_iso(o["date"].as_str().unwrap_or("")).unwrap_or(today);
            if d.m != last_month {
                it.header = ss(format!("Tháng {}/{}", d.m, d.y));
                last_month = d.m;
            }
        }
        items.push(it);
    }
    p.set_pkind(ps.pkind as i32);
    p.set_pfilter(ps.pfilter as i32);
    p.set_ptitle(ss(period_title(ps.pkind, f, t)));
    p.set_pcurrent(f <= today && today <= t);
    p.set_pcount(ss(format!("{} mục", items.len())));
    p.set_pnote(ss(match ps.pfilter {
        1 => "Chỉ có ngày giỗ (và tiên thường nếu bật nhắc). Không nhắc sinh nhật và sự kiện của người đã mất.",
        2 => "Sinh nhật và sự kiện của người đang sống.",
        _ => "",
    }));
    let what = ["tuần này", "tháng này", "quý này", "năm này"][ps.pkind];
    let filt = match ps.pfilter {
        1 => " (đang lọc: Người đã mất)",
        2 => " (đang lọc: Người đang sống)",
        _ => "",
    };
    p.set_pempty(ss(format!(
        "Không có ngày giỗ, sinh nhật hay sự kiện nào trong {what}.{filt}"
    )));
    p.set_pitems(vm(items));
}

fn sort_key(name: &str) -> (String, String) {
    let n = name.trim().to_lowercase();
    (n.split_whitespace().last().unwrap_or("").to_string(), n)
}

fn render_list(ui: &AppWindow, ps: &PState) {
    let p = People::get(ui);
    let Some(store) = ps.s() else { return };
    let today = local_today();
    let persons = jv(&store.persons_json().unwrap_or_default());
    let next = jv(&store.upcoming_json(today.solar(), 400).unwrap_or_default());
    let mut dead = Vec::new();
    let mut living = Vec::new();
    let mut ps_sorted: Vec<&Value> = persons.as_array().into_iter().flatten().collect();
    ps_sorted.sort_by_key(|p| sort_key(p["ho_ten"].as_str().unwrap_or("")));
    for v in ps_sorted {
        let id = st(&v["id"]);
        let name = display_name(v);
        let initial = v["ho_ten"]
            .as_str()
            .unwrap_or("")
            .split_whitespace()
            .last()
            .and_then(|w| w.chars().next())
            .map(|c| c.to_uppercase().collect::<String>())
            .unwrap_or_default();
        let summary = if v["kind"] == "deceased" {
            let l = &v["death"]["lunar"];
            // Giỗ hằng năm vào tháng thường kể cả khi mất vào tháng nhuận (O-4) — không ghi "nhuận" (TKN-W-03).
            let mut s = format!(
                "Giỗ {}",
                lunar_text(&json!({"month": l["month"], "day": l["day"], "leap": false}), false)
            );
            if let Some(o) = next
                .as_array()
                .into_iter()
                .flatten()
                .find(|o| o["class"] == "gio" && o["entity_id"] == v["id"])
            {
                let d = d_iso(o["date"].as_str().unwrap_or("")).unwrap_or(today);
                let n = d.jdn() - today.jdn();
                s += &format!(
                    " · lần tới {} ({})",
                    d.dmy(),
                    if n == 0 {
                        "hôm nay".to_string()
                    } else {
                        format!("còn {n} ngày")
                    }
                );
            }
            if l["year"].is_null() {
                s += " (không rõ năm mất)";
            }
            s
        } else {
            let b = &v["birth"];
            match (&b["solar"], b["input"]["precision"].as_str()) {
                (Value::Object(_), Some("day")) => {
                    let age = lvhv_ffi::person_summary_json(v.to_string(), today.solar())
                        .ok()
                        .and_then(|j| jv(&j)["tuoi_duong"].as_i64());
                    format!(
                        "Sinh {}{}",
                        dmy_v(&b["solar"]),
                        age.map(|a| format!(" · {a} tuổi")).unwrap_or_default()
                    )
                }
                _ if b.is_object() => date_value_text(&b["input"]).replace("Năm", "Sinh năm"),
                _ => "Chưa có ngày sinh".into(),
            }
        };
        let row = PRow {
            id: ss(&id),
            initial: ss(initial),
            name: ss(name),
            rel: ss(upper_first(st(&v["quan_he"]).trim())),
            summary: ss(summary),
        };
        if v["kind"] == "deceased" {
            dead.push(row);
        } else {
            living.push(row);
        }
    }
    p.set_has_any(!dead.is_empty() || !living.is_empty());
    p.set_dead(vm(dead));
    p.set_living(vm(living));
}

/// Gợi ý cách gọi cho người đã mất (YC-2026-07, 01 §3.1), viết hoa chữ đầu. Lõi chưa đưa danh sách này qua FFI
/// (PR #28 không có) — khi có sẽ lấy từ lõi.
pub(crate) const DX_DEAD: [&str; 22] = [
    "Ông", "Bà", "Cụ", "Kỵ", "Bác", "Cô", "Chú", "Dì", "Cậu", "Mợ", "Thím", "Anh", "Chị", "Em", "Cháu", "Con", "Bạn",
    "Bố", "Mẹ", "Ông Tổ họ", "Bà Cô Tổ họ", "Bé đỏ",
];
/// Người đang sống: như trên nhưng không có năm cách gọi chỉ dành cho người đã mất.
pub(crate) const DX_DEAD_ONLY: [&str; 5] = ["Cụ", "Kỵ", "Ông Tổ họ", "Bà Cô Tổ họ", "Bé đỏ"];

pub(crate) fn dx_suggestions(dead: bool) -> Vec<&'static str> {
    DX_DEAD.iter().copied().filter(|x| dead || !DX_DEAD_ONLY.contains(x)).collect()
}

/// Chia gợi ý thành hàng (Slint 1.8 không có bố cục tự xuống dòng): mỗi hàng tối đa `budget` ký tự (cộng 4 cho lề nút).
fn chip_rows(items: &[&str], budget: usize) -> Vec<ChipRow> {
    let mut rows: Vec<Vec<SharedString>> = vec![];
    let mut used = usize::MAX;
    for it in items {
        let w = it.chars().count() + 4;
        if used.saturating_add(w) > budget {
            rows.push(vec![]);
            used = 0;
        }
        used += w;
        rows.last_mut().unwrap().push(ss(*it));
    }
    rows.into_iter().map(|r| ChipRow { items: vm(r) }).collect()
}

/// Viết hoa chữ cái đầu (NFC, bỏ khoảng trắng hai đầu), phần còn lại giữ như người dùng gõ (YC-2026-07, 01 §3.1).
pub(crate) fn cap_first(s: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    let t: String = s.trim().nfc().collect();
    upper_first(&t)
}

/// Lõi đã nhận trường `gioi_tinh` (YC-2026-07) chưa. Lược đồ của lõi đóng (trường lạ → lỗi), nên ô Giới tính chỉ hiện
/// khi lõi kiểm hợp lệ một hồ sơ mẫu có trường này. Cùng cách của bản Android.
pub(crate) fn gender_supported() -> bool {
    static OK: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *OK.get_or_init(|| lvhv_ffi::derive_person_json(gender_probe(true).to_string()).is_ok())
}

pub(crate) fn gender_probe(with: bool) -> Value {
    let mut v = json!({
        "id": "00000000-0000-4000-8000-000000000001", "kind": "living", "danh_xung": "", "ho_ten": "Mẫu",
        "ten_hieu": "", "quan_he": "", "birth": null, "death": null, "noi_an_tang": "", "photo_id": null,
        "memorial": null, "reminders": {"gio": null, "sinh_nhat": null, "sinh_nhat_am": null},
        "sinh_nhat_am": false, "feb29_rule": "feb28", "ghi_chu": ""
    });
    if with {
        v["gioi_tinh"] = json!("nam");
    }
    v
}

#[cfg(test)]
mod yc07_tests {
    use super::*;

    #[test]
    fn cap_first_and_lists() {
        assert_eq!(cap_first("  ông tổ họ "), "Ông tổ họ");
        assert_eq!(cap_first("bé đỏ"), "Bé đỏ");
        // Dạng tách dấu (NFD) → NFC rồi viết hoa.
        assert_eq!(cap_first("e\u{0302}\u{0301}m"), "Ếm");
        assert_eq!(cap_first("Bà Cô Tổ họ"), "Bà Cô Tổ họ");
        assert_eq!(cap_first(""), "");
        let live = dx_suggestions(false);
        assert_eq!(live.len(), 17);
        for w in DX_DEAD_ONLY {
            assert!(!live.contains(&w));
            assert!(dx_suggestions(true).contains(&w));
        }
        let rows = chip_rows(&dx_suggestions(true), 44);
        let n: usize = rows.iter().map(|r| r.items.row_count()).sum();
        assert_eq!(n, 22);
        assert!(rows.len() >= 3);
    }

    #[test]
    fn gender_probe_is_meaningful() {
        // Hồ sơ mẫu không có giới tính phải hợp lệ, để lần dò có giới tính chỉ phụ thuộc vào trường mới.
        assert!(lvhv_ffi::derive_person_json(gender_probe(false).to_string()).is_ok());
    }
}

fn gender_text(v: &Value) -> &'static str {
    match v["gioi_tinh"].as_str() {
        Some("nam") => "Nam",
        Some("nu") => "Nữ",
        _ => "",
    }
}

fn display_name(v: &Value) -> String {
    let dx = upper_first(v["danh_xung"].as_str().unwrap_or("").trim());
    let dx = dx.as_str();
    let ht = v["ho_ten"].as_str().unwrap_or("");
    if dx.is_empty() {
        ht.to_string()
    } else {
        format!("{dx} {ht}")
    }
}

/// Ngày đầu của hôm nay (giờ máy) dạng giây Unix.
fn local_midnight_unix(today: D) -> i64 {
    (today.jdn() - 2_440_588) * 86_400 - platform::utc_offset_min() as i64 * 60
}

/// Thẻ nhắc hôm nay (L7): lời nhắc có ngày theo giờ máy = hôm nay, trừ thẻ đã bấm "Đã biết". Trong ứng dụng có ghi
/// tên (O-9 chỉ áp cho thông báo của hệ điều hành). Trả (id lời nhắc, thẻ).
fn today_cards(ps: &PState) -> Vec<(String, RemCard)> {
    let Some(store) = ps.s() else { return vec![] };
    let today = local_today();
    let mut cards = Vec::new();
    if let Ok(ds) = store.dataset_json() {
        let from = local_midnight_unix(today) - 60;
        if let Ok(j) = lvhv_ffi::reminders_json(ds, from, zone(), 2, true) {
            let pre = iso(today);
            for r in jv(&j).as_array().into_iter().flatten() {
                if !r["fire_local"].as_str().unwrap_or("").starts_with(&pre) {
                    continue;
                }
                let id = st(&r["id"]);
                if ps.hidden_cards.contains(&id) || id == "lvhv:sentinel" {
                    continue;
                }
                let o = &r["occ"];
                let class = o["class"].as_str().unwrap_or("");
                let pid = if class == "gio" || class.starts_with("sinh") || class == "tien_thuong" {
                    st(&o["entity_id"])
                } else {
                    o["person_ids"][0].as_str().unwrap_or("").to_string()
                };
                cards.push((
                    id,
                    RemCard {
                        title: ss(st(&r["title"])),
                        sub: ss(st(&r["body"])),
                        pid: ss(pid),
                        memorial: class == "gio" && o["date"].as_str() == Some(pre.as_str()),
                    },
                ));
            }
        }
    }
    cards
}

fn render_upcoming(ui: &AppWindow, ps: &PState) {
    let p = People::get(ui);
    let Some(store) = ps.s() else { return };
    let today = local_today();
    let sums = summaries(ps, today);
    let up = jv(&store.upcoming_json(today.solar(), 30).unwrap_or_default());
    let items: Vec<PItem> = up
        .as_array()
        .into_iter()
        .flatten()
        .map(|o| item(o, today, &sums))
        .collect();
    p.set_upcoming(vm(items));
    let cards: Vec<RemCard> = today_cards(ps).into_iter().map(|(_, c)| c).collect();
    let mut cards = cards;
    if let Some(first) = cards.first() {
        let pc = People::get(ui);
        pc.set_sheet_card(first.clone());
        pc.set_sheet_has_card(true);
        pc.set_sheet_more(cards.len() as i32 - 1);
    } else {
        p.set_sheet_has_card(false);
        p.set_sheet_more(0);
    }
    let more = cards.len().saturating_sub(3);
    cards.truncate(3);
    p.set_cards(vm(cards));
    p.set_cards_more(more as i32);
    let settings = jv(&store.settings_json().unwrap_or_default());
    let due = store.backup_reminder_due().unwrap_or(false);
    p.set_backup_nag(due);
    p.set_backup_nag_text(ss(format!(
        "Dữ liệu người thân đã thay đổi hơn {} ngày mà chưa sao lưu. Nếu máy hỏng hoặc mất máy, sẽ không lấy lại được.",
        settings["backup_nag_days"].as_i64().unwrap_or(30)
    )));
}

fn set_offs(ui: &AppWindow, ps: &mut PState, key: &'static str, offs: &[i64]) {
    let v: Vec<bool> = QUICK.iter().map(|q| offs.contains(q)).collect();
    let custom: Vec<i64> = offs
        .iter()
        .copied()
        .filter(|k| !QUICK.contains(k))
        .collect();
    let p = People::get(ui);
    let c = custom
        .first()
        .map(|k| k.to_string())
        .unwrap_or_default();
    match key {
        "r" => {
            p.set_r_offs(vm(v.clone()));
            p.set_r_custom(ss(c));
        }
        "r2" => {
            p.set_r2_offs(vm(v.clone()));
            p.set_r2_custom(ss(c));
        }
        "er" => {
            p.set_er_offs(vm(v.clone()));
            p.set_er_custom(ss(c));
        }
        _ => {
            p.set_def_offs(vm(v.clone()));
            p.set_def_custom(ss(c));
        }
    }
    ps.offs.insert(key, v);
}

fn render_settings(ui: &AppWindow, ps: &mut PState) {
    let Some(store) = ps.store.clone() else {
        return;
    };
    let p = People::get(ui);
    let s = jv(&store.settings_json().unwrap_or_default());
    p.set_rem_enabled(s["reminders_enabled"] != false);
    p.set_show_names(store.lock_screen_names().unwrap_or(false));
    let offs: Vec<i64> = s["default_offsets_days"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_i64).collect())
        .unwrap_or(vec![0, 3]);
    set_offs(ui, ps, "def", &offs);
    let t = s["default_time_local"].as_str().unwrap_or("07:00");
    p.set_def_hh(ss(&t[..2.min(t.len())]));
    p.set_def_mm(ss(t.get(3..5).unwrap_or("00")));
    p.set_def_error(ss(""));
    p.set_nag_index(match s["backup_nag_days"].as_i64() {
        Some(60) => 1,
        Some(90) => 2,
        Some(0) => 3,
        _ => 0,
    });
    let last = match s["last_backup_at"].as_str().and_then(local_date_of_rfc3339) {
        Some(d) => format!("Lần sao lưu gần nhất: {}", d.dmy()),
        None => "Chưa sao lưu lần nào trên máy này".into(),
    };
    let restored = std::fs::read_to_string(ps.dir.join("ui-restored-from.txt"))
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| format!("\nDữ liệu được khôi phục từ bản sao lưu ngày {}", s.trim()))
        .unwrap_or_default();
    p.set_last_backup(ss(format!("{last}{restored}")));
    if People::get(ui).get_page() == "sao-luu" {
        render_ics_state(ui, ps);
    }
}

/// Nút "Xuất tệp lịch" bật khi có ít nhất một lần nhắc trong 10 năm tới (chỉ tính khi mở trang Sao lưu).
fn render_ics_state(ui: &AppWindow, ps: &PState) {
    let Some(store) = ps.s() else { return };
    let p = People::get(ui);
    let today = local_today();
    let n = store
        .dataset_json()
        .ok()
        .and_then(|ds| lvhv_ffi::ics_export(ds, today.solar(), platform::now_unix(), None).ok())
        .map(|i| i.events)
        .unwrap_or(0);
    p.set_ics_enabled(n > 0);
    p.set_ics_note(ss(if n > 0 { "" } else { "Chưa có hồ sơ nào bật nhắc." }));
}

fn local_date_of_rfc3339(s: &str) -> Option<D> {
    let d = d_iso(s.get(0..10)?)?;
    let hh: i64 = s.get(11..13)?.parse().ok()?;
    let mm: i64 = s.get(14..16)?.parse().ok()?;
    let mins = hh * 60 + mm + platform::utc_offset_min() as i64;
    d.add(mins.div_euclid(1440) as i32)
}

/// Câu trạng thái "Lời nhắc trên Windows" cho một kết quả đặt lịch; `true` = thông báo chắc chắn đang hoạt động.
pub fn notif_status(r: &SyncResult, enabled: bool) -> (bool, String) {
    const TAIL: &str = "Lời nhắc vẫn hiện trong mục Sắp tới khi bạn mở ứng dụng; có thể xuất tệp lịch (.ics) để lịch của máy nhắc.";
    match r {
        SyncResult::Unavailable(t) => (false, format!("{t} {TAIL}")),
        SyncResult::Disabled { reason, cleared } => (
            false,
            match cleared {
                Ok(()) => format!("{reason} Các thông báo đã đặt trước đó đã được gỡ; khi bật lại thông báo, hãy mở ứng dụng để đặt lại. {TAIL}"),
                Err(e) => format!("{reason} Chưa gỡ được các thông báo đã đặt ({e}). {TAIL}"),
            },
        ),
        SyncResult::ReadFailed(e) => (
            false,
            format!("Chưa đọc được danh sách thông báo của Windows ({e}) — sẽ thử lại khi mở ứng dụng. {TAIL}"),
        ),
        SyncResult::Failed(e) => (
            false,
            format!("Chưa đặt được thông báo của Windows ({e}). Thông báo cần bản cài đặt (.msi), vì bộ cài tạo lối tắt Start Menu mang định danh của ứng dụng. {TAIL}"),
        ),
        SyncResult::Scheduled { .. } if !enabled => (
            false,
            "Lời nhắc đang tắt. Bật lại để Windows nhắc ngày giỗ, sinh nhật.".to_string(),
        ),
        SyncResult::Scheduled { pending, confirmed: true, .. } => (
            true,
            format!("Thông báo của Windows đang bật. Đã đặt {pending} lời nhắc sắp tới; Windows tự hiện đúng giờ, kể cả khi ứng dụng đã đóng. Mỗi lần mở ứng dụng, lời nhắc được đặt lại (tối đa 64)."),
        ),
        SyncResult::Scheduled { pending, confirmed: false, .. } => (
            false,
            format!("Đã đặt {pending} lời nhắc, nhưng Windows chưa xác nhận ứng dụng được phép hiện thông báo. Nếu bạn dùng bản không cài đặt (.zip), hãy cài bản .msi; có thể xuất tệp .ics để lịch của máy nhắc."),
        ),
    }
}

/// Đặt lại thông báo Windows theo kế hoạch xoay vòng 64 của lõi (02 §7.4–7.5) rồi cập nhật dòng trạng thái.
pub fn reschedule(ui: Option<&AppWindow>, ps: &mut PState) {
    let Some(store) = ps.store.clone() else {
        return;
    };
    let enabled = jv(&store.settings_json().unwrap_or_default())["reminders_enabled"] != false;
    let r = toasts::sync(ps.toasts.as_ref(), &store, platform::now_unix(), zone());
    ps.last_sync = Some(r.clone());
    if let Some(ui) = ui {
        let (ok, text) = notif_status(&r, enabled);
        let p = People::get(ui);
        p.set_notif_ok(ok);
        p.set_notif_status(ss(text));
    }
}

/// Vẽ lại mọi trang danh sách sau khi dữ liệu đổi.
pub fn refresh(ui: &AppWindow, ps: &mut PState) {
    if ps.store.is_none() {
        return;
    }
    render_list(ui, ps);
    render_upcoming(ui, ps);
    render_period(ui, ps);
    render_settings(ui, ps);
    if let Some(id) = ps.person.clone() {
        render_person(ui, ps, &id);
    }
}

fn after_change(ui: &AppWindow, ps: &mut PState) {
    refresh(ui, ps);
    reschedule(Some(ui), ps);
}

// ------------------------------------------------------------------------------------------------
// Điều hướng
// ------------------------------------------------------------------------------------------------

fn back_label(prev: &str) -> &'static str {
    match prev {
        "ho-so" => "‹ Về danh sách hồ sơ",
        "nguoi" => "‹ Về hồ sơ",
        "sap-toi" => "‹ Về Sắp tới",
        "theo-ky" => "‹ Về Theo kỳ",
        _ => "‹ Quay lại",
    }
}

fn show(ui: &AppWindow, ps: &mut PState, page: &str) {
    let p = People::get(ui);
    let cur = p.get_page().to_string();
    if matches!(page, "sap-toi" | "theo-ky" | "ho-so" | "sao-luu") {
        ps.stack.clear();
    } else if cur != page {
        ps.stack.push(cur.clone());
    }
    p.set_back_label(ss(back_label(ps.stack.last().map(String::as_str).unwrap_or("ho-so"))));
    p.set_scroll_frac(0.0);
    p.set_page(ss(page));
}

pub fn go(ui: &AppWindow, ps: &mut PState, page: &str) {
    if !ensure_store(ui, ps) {
        People::get(ui).set_page(ss(page));
        return;
    }
    ps.person = None;
    show(ui, ps, page);
    refresh(ui, ps);
}

fn back(ui: &AppWindow, ps: &mut PState) {
    let p = People::get(ui);
    ps.smoke_gen.set(ps.smoke_gen.get() + 1);
    let prev = ps.stack.pop().unwrap_or_else(|| "ho-so".into());
    if prev == "nguoi" {
        if let Some(id) = ps.person.clone() {
            render_person(ui, ps, &id);
        }
    } else if !matches!(prev.as_str(), "form" | "su-kien" | "bao-tu" | "tuong-niem") {
        ps.person = None;
    }
    p.set_back_label(ss(back_label(ps.stack.last().map(String::as_str).unwrap_or("ho-so"))));
    p.set_scroll_frac(0.0);
    p.set_page(ss(prev));
    ps.form = FormKind::None;
    p.set_f_summary(ss(""));
}

// ------------------------------------------------------------------------------------------------
// Hồ sơ một người
// ------------------------------------------------------------------------------------------------

fn person(ps: &PState, id: &str) -> Option<Value> {
    ps.s()?.person_json(id.into()).ok().map(|j| jv(&j))
}

fn photo_image(ps: &PState, id: &str) -> Option<Image> {
    photo::to_slint(&ps.s()?.photo_bytes(id.into()).ok()?)
}

fn events_of(ps: &PState, id: &str) -> Vec<Value> {
    let Some(store) = ps.s() else { return vec![] };
    jv(&store.events_json().unwrap_or_default())
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|e| {
            e["person_ids"]
                .as_array()
                .is_some_and(|a| a.iter().any(|x| x == id))
        })
        .collect()
}

fn type_name(ps: &PState, type_id: &Value) -> String {
    ps.s()
        .and_then(|s| s.event_types_json().ok())
        .map(|j| jv(&j))
        .and_then(|t| {
            t.as_array()?
                .iter()
                .find(|x| &x["id"] == type_id)
                .map(|x| st(&x["ten"]))
        })
        .unwrap_or_else(|| "Sự kiện".into())
}

fn render_person(ui: &AppWindow, ps: &mut PState, id: &str) {
    let p = People::get(ui);
    let Some(v) = person(ps, id) else {
        p.set_v_missing(true);
        return;
    };
    p.set_v_missing(false);
    let today = local_today();
    let sum = jv(&lvhv_ffi::person_summary_json(v.to_string(), today.solar()).unwrap_or_default());
    let dead = v["kind"] == "deceased";
    p.set_v_id(ss(id));
    p.set_v_dead(dead);
    let name = display_name(&v);
    p.set_v_name(ss(&name));
    p.set_v_initial(ss(v["ho_ten"]
        .as_str()
        .unwrap_or("")
        .split_whitespace()
        .last()
        .and_then(|w| w.chars().next())
        .map(String::from)
        .unwrap_or_default()));
    p.set_v_rel(ss(upper_first(st(&v["quan_he"]).trim())));
    let hieu = st(&v["ten_hieu"]);
    p.set_v_sub(ss(if hieu.is_empty() {
        String::new()
    } else {
        format!("Tên hiệu / pháp danh: {hieu}")
    }));
    let img = v["photo_id"].as_str().and_then(|pid| photo_image(ps, pid));
    p.set_v_has_photo(img.is_some());
    p.set_v_photo(img.unwrap_or_default());
    // Lần tới: mọi lần trong 12 tháng tới, tối đa 5 mục (TK-34); với người đã mất lõi chỉ sinh giỗ/tiên thường.
    let sums = summaries(ps, today);
    let up = jv(&ps
        .s()
        .and_then(|s| s.upcoming_json(today.solar(), 366).ok())
        .unwrap_or_default());
    let next: Vec<PItem> = up
        .as_array()
        .into_iter()
        .flatten()
        .filter(|o| {
            o["entity_id"] == id
                || o["person_ids"]
                    .as_array()
                    .is_some_and(|a| a.iter().any(|x| x == id))
        })
        .take(5)
        .map(|o| item(o, today, &sums))
        .collect();
    p.set_v_next(vm(next));
    let mut info: Vec<KV> = Vec::new();
    let mut kv = |k: &str, val: String| {
        if !val.trim().is_empty() {
            info.push(KV {
                k: ss(k),
                v: ss(val),
            });
        }
    };
    let b = &v["birth"];
    if dead {
        let d = &v["death"];
        kv("Ngày mất (đã nhập)", date_value_text(&d["input"]));
        kv(
            "Âm lịch",
            format!(
                "{} ({})",
                st(&sum["death_lunar_text"]),
                if d["lunar_source"] == "converted" {
                    "đổi từ ngày dương"
                } else {
                    "nhập theo âm lịch"
                }
            ),
        );
        if d["input"]["calendar"] == "lunar" && d["solar"].is_object() {
            let s = &d["solar"];
            let dd = D {
                y: s["y"].as_i64().unwrap_or(0) as i32,
                m: s["m"].as_i64().unwrap_or(0) as i32,
                d: s["d"].as_i64().unwrap_or(0) as i32,
            };
            kv("Dương lịch", format!("{}, {}", WEEKDAY[dd.weekday()], dd.dmy()));
        }
        kv("Tuổi", st(&sum["tuoi_mat_text"]));
    }
    kv("Giới tính", gender_text(&v).to_string());
    if b.is_object() {
        kv("Ngày sinh", date_value_text(&b["input"]));
        if b["input"]["calendar"] == "solar" && b["lunar"].is_object() {
            kv("Âm lịch", lunar_text(&b["lunar"], true));
        }
        if let Some(ly) = b["lunar_year"].as_i64() {
            kv(
                "Năm âm sinh",
                format!(
                    "{ly} — {}{}",
                    st(&b["can_chi"]),
                    if b["lunar_year_approx"] == true {
                        " (khoảng)"
                    } else {
                        ""
                    }
                ),
            );
        }
        kv("Nơi sinh", st(&b["noi_sinh"]));
    }
    if !dead {
        if let Some(t) = sum["tuoi_duong"].as_i64() {
            kv(
                "Tuổi",
                match sum["tuoi_am"].as_i64() {
                    Some(a) => format!("{t} (tuổi âm {a})"),
                    None => t.to_string(),
                },
            );
        }
    }
    if dead {
        kv("Nơi an táng", st(&v["noi_an_tang"]));
    }
    kv("Ghi chú", st(&v["ghi_chu"]));
    p.set_v_info(vm(info));
    // Thông tin báo tử (R-PRIV-2): chỉ ở trang này.
    let n = &v["death"]["notice"];
    let mut nk: Vec<KV> = Vec::new();
    if n.is_object() {
        for (k, key) in [
            ("Người báo tin", "reported_by"),
            ("Mất tại", "place"),
            ("Ngày nhận tin", "notice_date"),
            ("Lý do", "reason"),
            ("Ghi chú", "note"),
        ] {
            let val = if key == "notice_date" {
                if n[key].is_object() {
                    date_value_text(&n[key])
                } else {
                    String::new()
                }
            } else {
                st(&n[key])
            };
            if !val.is_empty() {
                nk.push(KV {
                    k: ss(k),
                    v: ss(val),
                });
            }
        }
    }
    let nphotos: Vec<Image> = n["certificate_photo_ids"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|x| x.as_str().and_then(|i| photo_image(ps, i)))
        .collect();
    p.set_v_notice(vm(nk));
    p.set_v_notice_photos(vm(nphotos));
    // Sự kiện
    let evs: Vec<EvRow> = events_of(ps, id)
        .iter()
        .map(|e| {
            let title = if st(&e["tieu_de"]).is_empty() {
                type_name(ps, &e["type_id"])
            } else {
                st(&e["tieu_de"])
            };
            let dv = &e["date"];
            let date = if dv["calendar"] == "lunar" {
                if dv["year"].is_null() {
                    lunar_text(dv, false)
                } else {
                    lunar_text(dv, true)
                }
            } else {
                date_value_text(dv)
            };
            EvRow {
                id: ss(st(&e["id"])),
                text: ss(format!(
                    "{title} · {date} · {}",
                    if e["recurring"] == true {
                        "hằng năm"
                    } else {
                        "một lần"
                    }
                )),
            }
        })
        .collect();
    p.set_v_events(vm(evs));
    let r = &v["reminders"];
    let remind = if dead {
        let g = &r["gio"];
        if !g.is_object() || g["enabled"] == false {
            "Đang tắt nhắc ngày giỗ.".to_string()
        } else {
            format!(
                "Ngày giỗ: {} · Tiên thường: {}",
                offsets_text(g),
                if g["tien_thuong"] == true { "bật" } else { "tắt" }
            )
        }
    } else {
        let s1 = &r["sinh_nhat"];
        let mut t = if !s1.is_object() || s1["enabled"] == false {
            "Đang tắt nhắc sinh nhật.".to_string()
        } else {
            format!("Sinh nhật: {}", offsets_text(s1))
        };
        if v["sinh_nhat_am"] == true && r["sinh_nhat_am"].is_object() {
            let s2 = &r["sinh_nhat_am"];
            t += &if s2["enabled"] == false {
                " · Sinh nhật âm lịch: tắt".to_string()
            } else {
                format!(" · Sinh nhật âm lịch: {}", offsets_text(s2))
            };
        }
        t
    };
    p.set_v_remind(ss(remind));
}

fn open_person(ui: &AppWindow, ps: &mut PState, id: &str) {
    if id.is_empty() || !ensure_store(ui, ps) {
        return;
    }
    ps.person = Some(id.to_string());
    People::get(ui).set_v_notice_open(false);
    render_person(ui, ps, id);
    show(ui, ps, "nguoi");
}

// ------------------------------------------------------------------------------------------------
// Màn hình tưởng niệm
// ------------------------------------------------------------------------------------------------

fn open_memorial(ui: &AppWindow, ps: &mut PState, id: &str) {
    if !ensure_store(ui, ps) {
        return;
    }
    let Some(v) = person(ps, id) else { return };
    if v["kind"] != "deceased" {
        return;
    }
    ps.person = Some(id.to_string());
    let p = People::get(ui);
    let today = local_today();
    let sum = jv(&lvhv_ffi::person_summary_json(v.to_string(), today.solar()).unwrap_or_default());
    let rel = st(&v["quan_he"]);
    let rel = rel.split('(').next().unwrap_or("").trim().to_string();
    let rel = upper_first(&rel);
    let name = if rel.is_empty() {
        display_name(&v)
    } else {
        st(&v["ho_ten"])
    };
    p.set_m_rel(ss(&rel));
    p.set_m_long(name.chars().count() > 28);
    p.set_m_name(ss(&name));
    let hieu = st(&v["ten_hieu"]);
    p.set_m_hieu(ss(if hieu.is_empty() {
        String::new()
    } else {
        format!("Pháp danh {hieu}")
    }));
    let d = &v["death"];
    let dy = d["lunar"]["year"].as_i64();
    let by = v["birth"]["lunar_year"].as_i64();
    // "Canh Thìn 1940 — Ất Tỵ 2025" như Android và PWA (TKN-X-02; 08 §3.9 sửa theo).
    let cc = |y: i64| format!("{} {y}", lvhv_ffi::year_can_chi(y as i32));
    p.set_m_years(ss(match (by, dy) {
        (Some(b), Some(dd)) => format!("{} — {}", cc(b), cc(dd)),
        (None, Some(dd)) => format!("Mất năm {}", cc(dd)),
        _ => format!("Giỗ ngày {}", lunar_text(&d["lunar"], false)),
    }));
    p.set_m_tuoi(ss(st(&sum["tuoi_mat_text"])));
    p.set_m_loi(ss(st(&v["memorial"]["loi_tuong_niem"])));
    p.set_m_initial(ss(v["ho_ten"]
        .as_str()
        .unwrap_or("")
        .split_whitespace()
        .last()
        .and_then(|w| w.chars().next())
        .map(String::from)
        .unwrap_or_default()));
    let img = v["photo_id"].as_str().and_then(|pid| photo_image(ps, pid));
    p.set_m_has_photo(img.is_some());
    p.set_m_photo(img.unwrap_or_default());
    let catholic = v["memorial"]["tray_style"] == "cong_giao";
    p.set_m_incense(!catholic && v["memorial"]["incense"] != false);
    // Dòng ngày (§3.9): lấy lần giỗ kế tiếp từ lõi.
    let up = jv(&ps
        .s()
        .and_then(|s| s.upcoming_json(today.solar(), 400).ok())
        .unwrap_or_default());
    let g = up
        .as_array()
        .into_iter()
        .flatten()
        .find(|o| o["class"] == "gio" && o["entity_id"] == id)
        .cloned();
    p.set_m_dateline(ss(match g {
        Some(o) if o["date"].as_str() == Some(iso(today).as_str()) => {
            let label = st(&o["headline"]);
            let label = label
                .strip_suffix(&display_name(&v))
                .unwrap_or(&label)
                .trim()
                .to_string();
            format!("{label} · hôm nay {}", lunar_text(&o["lunar"], false))
        }
        Some(o) => {
            let d = d_iso(o["date"].as_str().unwrap_or(""));
            format!(
                // Mọi nền tảng: "Ngày giỗ sắp tới" (quyết định điều phối, TKN-X-06).
                "Ngày giỗ sắp tới: {}, {}",
                lunar_text(&o["lunar"], false),
                d.map(|x| x.dmy()).unwrap_or_default()
            )
        }
        None => format!("Ngày giỗ: {}", lunar_text(&d["lunar"], false)),
    }));
    let cur = p.get_page().to_string();
    ps.memorial_back = cur;
    start_smoke(ui, ps);
    show(ui, ps, "tuong-niem");
    p.set_back_label(ss("‹ Về hồ sơ"));
}

/// Khói chạy tối đa 2 phút rồi đứng yên (R8); "Thắp lại" chạy thêm 2 phút.
fn start_smoke(ui: &AppWindow, ps: &PState) {
    People::get(ui).set_m_smoke(true);
    let w = ui.as_weak();
    // Mỗi lần thắp là một bộ hẹn giờ mới, có số thế hệ: tránh khởi động lại một Timer đang chờ ngay trong
    // lượt xử lý hẹn giờ (Slint 1.8 báo lỗi khẳng định `!timer.being_activated`).
    let gen = ps.smoke_gen.get() + 1;
    ps.smoke_gen.set(gen);
    let g = ps.smoke_gen.clone();
    Timer::single_shot(Duration::from_secs(120), move || {
        if g.get() == gen {
            if let Some(ui) = w.upgrade() {
                People::get(&ui).set_m_smoke(false);
            }
        }
    });
}

// ------------------------------------------------------------------------------------------------
// Biểu mẫu: ô ngày
// ------------------------------------------------------------------------------------------------

#[derive(Clone)]
struct DateFields {
    cal: i32,
    prec: i32,
    d: String,
    m: String,
    y: String,
    leap: bool,
    unknown: bool,
}

fn num(s: &str) -> Option<i64> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        t.parse().ok()
    }
}

/// Tháng nhuận của năm âm (None nếu không có).
fn leap_month(ly: i32) -> Option<i32> {
    (1..=12).find(|&m| lvhv_ffi::lunar_to_solar(ly, m, 1, true).is_some())
}

enum DateRes {
    Empty,
    Incomplete(String),
    Bad(String),
    Ok(Value),
}

/// Ô ngày → `DateValue`. `kind`: "death" | "birth" | "event".
fn date_value(f: &DateFields, kind: &str) -> DateRes {
    let lunar = f.cal == 1;
    let year_only = f.prec == 1;
    let unknown = lunar && f.unknown && kind != "birth";
    let (d, m, y) = (num(&f.d), num(&f.m), num(&f.y));
    let any = !f.d.trim().is_empty() || !f.m.trim().is_empty() || !f.y.trim().is_empty();
    if !any {
        return DateRes::Empty;
    }
    if year_only {
        let Some(y) = y else {
            return DateRes::Incomplete("Hãy nhập năm".into());
        };
        if !(1000..=3000).contains(&y) {
            return DateRes::Bad("Năm phải từ 1000 đến 3000".into());
        }
        return DateRes::Ok(json!({"calendar": if lunar {"lunar"} else {"solar"}, "precision": "year", "year": y, "month": null, "day": null, "leap": false}));
    }
    if unknown {
        let (Some(d), Some(m)) = (d, m) else {
            return DateRes::Incomplete("Hãy nhập đủ ngày và tháng âm".into());
        };
        if !(1..=12).contains(&m) || !(1..=30).contains(&d) {
            return DateRes::Bad("Ngày âm từ 1 đến 30, tháng từ 1 đến 12".into());
        }
        return DateRes::Ok(json!({"calendar": "lunar", "precision": "month_day", "year": null, "month": m, "day": d, "leap": f.leap}));
    }
    let (Some(d), Some(m), Some(y)) = (d, m, y) else {
        return DateRes::Incomplete("Hãy nhập đủ ngày, tháng, năm".into());
    };
    let (lo, hi) = if kind == "birth" { (1000, 3000) } else { (1900, 3000) };
    if !(lo..=hi).contains(&y) {
        return DateRes::Bad(format!("Năm phải từ {lo} đến {hi}"));
    }
    if !(1..=12).contains(&m) || d < 1 || d > if lunar { 30 } else { 31 } {
        return DateRes::Bad("Ngày hoặc tháng không hợp lệ".into());
    }
    if !lunar && lvhv_ffi::days_in_month(y as i32, m as i32).is_ok_and(|n| d > n as i64) {
        return DateRes::Bad(format!(
            "Tháng {m} năm {y} chỉ có {} ngày",
            lvhv_ffi::days_in_month(y as i32, m as i32).unwrap_or(31)
        ));
    }
    DateRes::Ok(json!({"calendar": if lunar {"lunar"} else {"solar"}, "precision": "day", "year": y, "month": m, "day": d, "leap": lunar && f.leap}))
}

/// Dòng kết quả đổi lịch + lỗi + hiển thị hộp "Tháng nhuận" của một ô ngày.
struct DateView {
    result: String,
    error: String,
    leap_visible: bool,
    leap_label: String,
    note: String,
    value: Option<Value>,
}

fn date_view(f: &DateFields, kind: &str, today: D, living: bool, show_err: bool) -> DateView {
    let mut v = DateView {
        result: String::new(),
        error: String::new(),
        leap_visible: true,
        leap_label: "Tháng nhuận".into(),
        note: String::new(),
        value: None,
    };
    if f.cal == 1 && f.prec == 0 && !(f.unknown && kind != "birth") {
        if let Some(y) = num(&f.y).filter(|y| (1899..=3000).contains(y)) {
            match leap_month(y as i32) {
                Some(lm) => v.leap_label = format!("Tháng nhuận (năm {y} nhuận tháng {lm})"),
                None => {
                    v.leap_visible = false;
                    v.note = format!("Năm {y} không có tháng nhuận");
                }
            }
        }
    }
    let mut f2 = f.clone();
    if !v.leap_visible {
        f2.leap = false;
    }
    match date_value(&f2, kind) {
        DateRes::Empty => {
            if show_err && kind != "birth" {
                v.error = if kind == "death" {
                    "Hãy nhập ngày mất".into()
                } else {
                    "Hãy nhập ngày".into()
                };
            }
        }
        DateRes::Incomplete(e) => {
            if show_err {
                v.error = e;
            }
        }
        DateRes::Bad(e) => v.error = e,
        DateRes::Ok(dv) => {
            let derived = if kind == "birth" {
                lvhv_ffi::derive_birth_json(dv.to_string())
            } else {
                lvhv_ffi::save_death_json(dv.to_string())
            };
            match derived {
                Err(e) => v.error = e.to_string(),
                Ok(j) => {
                    let o = jv(&j);
                    let solar = &o["solar"];
                    let sd = solar.is_object().then(|| D {
                        y: solar["y"].as_i64().unwrap_or(0) as i32,
                        m: solar["m"].as_i64().unwrap_or(0) as i32,
                        d: solar["d"].as_i64().unwrap_or(0) as i32,
                    });
                    if let Some(sd) = sd.filter(|s| *s > today) {
                        if kind != "event" {
                            v.error = if kind == "death" {
                                "Ngày mất không được sau hôm nay".into()
                            } else {
                                "Ngày sinh không được sau hôm nay".into()
                            };
                            let _ = sd;
                        }
                    }
                    v.result = match (kind, dv["calendar"].as_str(), dv["precision"].as_str()) {
                        ("birth", _, Some("year")) => match o["lunar_year"].as_i64() {
                            Some(ly) if dv["calendar"] == "solar" => format!(
                                "khoảng năm âm {ly} — {} (nếu sinh trước Tết thì là năm trước)",
                                st(&o["can_chi"])
                            ),
                            Some(ly) => format!("Năm âm {ly} — {}", st(&o["can_chi"])),
                            None => String::new(),
                        },
                        ("birth", Some("solar"), _) => {
                            let base = if o["lunar"].is_object() {
                                format!("Âm lịch: {}", lunar_text(&o["lunar"], true).replace(" âm lịch năm", " năm"))
                            } else {
                                format!(
                                    "Năm âm {} — {}",
                                    o["lunar_year"].as_i64().unwrap_or(0),
                                    st(&o["can_chi"])
                                )
                            };
                            if living {
                                let p = json!({"kind": "living", "danh_xung": "", "ho_ten": "", "birth": o, "death": null});
                                let s = jv(&lvhv_ffi::person_summary_json(p.to_string(), today.solar())
                                    .unwrap_or_default());
                                match (s["tuoi_duong"].as_i64(), s["tuoi_am"].as_i64()) {
                                    (Some(t), Some(a)) => format!("{base} · Hôm nay {t} tuổi (tuổi âm {a})"),
                                    (Some(t), None) => format!("{base} · Hôm nay {t} tuổi"),
                                    _ => base,
                                }
                            } else {
                                format!(
                                    "Năm âm {} — {}",
                                    o["lunar_year"].as_i64().unwrap_or(0),
                                    st(&o["can_chi"])
                                )
                            }
                        }
                        ("birth", _, _) => match sd {
                            Some(s) => format!(
                                "Dương lịch: {}, {} · Năm âm {} — {}",
                                WEEKDAY[s.weekday()],
                                s.dmy(),
                                o["lunar_year"].as_i64().unwrap_or(0),
                                st(&o["can_chi"])
                            ),
                            None => format!("Năm âm {} — {}", o["lunar_year"].as_i64().unwrap_or(0), st(&o["can_chi"])),
                        },
                        (_, Some("solar"), _) => format!("Âm lịch: {}", lunar_text(&o["lunar"], true).replace(" âm lịch năm", " năm")),
                        (_, _, Some("month_day")) => if kind == "death" {
                            format!(
                                "Không rõ năm: vẫn nhắc giỗ hằng năm vào {}, không ghi giỗ đầu, giỗ hết.",
                                lunar_text(&o["lunar"], false)
                            )
                        } else {
                            format!("Hằng năm vào {}", lunar_text(&o["lunar"], false))
                        },
                        _ => match sd {
                            Some(s) => format!("Dương lịch: {}, {}", WEEKDAY[s.weekday()], s.dmy()),
                            None => "Năm này ngoài khoảng đổi lịch — không có ngày dương tương ứng".into(),
                        },
                    };
                    if v.error.is_empty() {
                        v.value = Some(dv);
                    }
                }
            }
        }
    }
    v
}

fn fields_dd(p: &People) -> DateFields {
    DateFields {
        cal: p.get_dd_cal(),
        prec: 0,
        d: p.get_dd_d().into(),
        m: p.get_dd_m().into(),
        y: p.get_dd_y().into(),
        leap: p.get_dd_leap(),
        unknown: p.get_dd_unknown(),
    }
}
fn fields_bd(p: &People) -> DateFields {
    DateFields {
        cal: p.get_bd_cal(),
        prec: p.get_bd_prec(),
        d: p.get_bd_d().into(),
        m: p.get_bd_m().into(),
        y: p.get_bd_y().into(),
        leap: p.get_bd_leap(),
        unknown: false,
    }
}
fn fields_md(p: &People) -> DateFields {
    DateFields {
        cal: p.get_md_cal(),
        prec: 0,
        d: p.get_md_d().into(),
        m: p.get_md_m().into(),
        y: p.get_md_y().into(),
        leap: p.get_md_leap(),
        unknown: p.get_md_unknown(),
    }
}
fn fields_ed(p: &People) -> DateFields {
    DateFields {
        cal: p.get_ed_cal(),
        prec: 0,
        d: p.get_ed_d().into(),
        m: p.get_ed_m().into(),
        y: p.get_ed_y().into(),
        leap: p.get_ed_leap(),
        unknown: p.get_ed_unknown() && p.get_ev_recurring(),
    }
}

fn set_dd(p: &People, v: &DateView) {
    p.set_dd_result(ss(&v.result));
    p.set_dd_error(ss(&v.error));
    p.set_dd_leap_visible(v.leap_visible);
    p.set_dd_leap_label(ss(&v.leap_label));
    p.set_dd_note(ss(&v.note));
}
fn set_bd(p: &People, v: &DateView) {
    p.set_bd_result(ss(&v.result));
    p.set_bd_error(ss(&v.error));
    p.set_bd_leap_visible(v.leap_visible);
    p.set_bd_leap_label(ss(&v.leap_label));
    p.set_bd_note(ss(&v.note));
}
fn set_md(p: &People, v: &DateView) {
    p.set_md_result(ss(&v.result));
    p.set_md_error(ss(&v.error));
    p.set_md_leap_visible(v.leap_visible);
    p.set_md_leap_label(ss(&v.leap_label));
    p.set_md_note(ss(&v.note));
}
fn set_ed(p: &People, v: &DateView) {
    p.set_ed_result(ss(&v.result));
    p.set_ed_error(ss(&v.error));
    p.set_ed_leap_visible(v.leap_visible);
    p.set_ed_leap_label(ss(&v.leap_label));
    p.set_ed_note(ss(&v.note));
}

/// Điền ô ngày từ `DateValue`.
fn fill_date(v: &Value) -> DateFields {
    let s = |k: &str| v[k].as_i64().map(|x| x.to_string()).unwrap_or_default();
    DateFields {
        cal: if v["calendar"] == "lunar" { 1 } else { 0 },
        prec: if v["precision"] == "year" { 1 } else { 0 },
        d: s("day"),
        m: s("month"),
        y: s("year"),
        leap: v["leap"] == true,
        unknown: v["precision"] == "month_day",
    }
}

// ------------------------------------------------------------------------------------------------
// Biểu mẫu: nhắc
// ------------------------------------------------------------------------------------------------

/// Quy tắc nhắc từ ô nhập. Lỗi theo §3.4.
fn rule_from(
    ps: &PState,
    key: &'static str,
    enabled: bool,
    custom: &str,
    hh: &str,
    mm: &str,
    tt: bool,
    old: &Value,
) -> Result<(Value, String), String> {
    let mut offs: Vec<i64> = ps
        .offs
        .get(key)
        .map(|v| {
            v.iter()
                .zip(QUICK)
                .filter(|(b, _)| **b)
                .map(|(_, k)| k)
                .collect()
        })
        .unwrap_or_default();
    if !custom.trim().is_empty() {
        match num(custom).filter(|k| (1..=60).contains(k)) {
            Some(k) => offs.push(k),
            None => return Err("Số ngày tự đặt phải từ 1 đến 60".into()),
        }
    }
    offs.sort_unstable();
    offs.dedup();
    if offs.len() > 8 {
        return Err("Tối đa 8 mốc nhắc".into());
    }
    let (h, m) = (num(hh), num(mm));
    let (Some(h), Some(m)) = (h, m) else {
        return Err("Giờ nhắc phải từ 0 đến 23 giờ, phút từ 0 đến 59".into());
    };
    if !(0..=23).contains(&h) || !(0..=59).contains(&m) {
        return Err("Giờ nhắc phải từ 0 đến 23 giờ, phút từ 0 đến 59".into());
    }
    let warn = if offs.is_empty() && enabled {
        "Chưa chọn mốc nhắc nào — sẽ không có lời nhắc".to_string()
    } else {
        String::new()
    };
    let id = old["id"]
        .as_str()
        .map(String::from)
        .or_else(|| lvhv_ffi::new_uuid().ok())
        .unwrap_or_default();
    Ok((
        json!({"id": id, "enabled": enabled, "offsets_days": offs, "time_local": format!("{h:02}:{m:02}"), "tien_thuong": tt}),
        warn,
    ))
}

fn fill_rule(ui: &AppWindow, ps: &mut PState, key: &'static str, rule: &Value, defaults: &Value) {
    let src = if rule.is_object() { rule } else { defaults };
    let offs: Vec<i64> = src["offsets_days"]
        .as_array()
        .or(defaults["default_offsets_days"].as_array())
        .map(|a| a.iter().filter_map(Value::as_i64).collect())
        .unwrap_or(vec![0, 3]);
    set_offs(ui, ps, key, &offs);
    let t = src["time_local"]
        .as_str()
        .or(defaults["default_time_local"].as_str())
        .unwrap_or("07:00")
        .to_string();
    let p = People::get(ui);
    let (h, m) = (t[..2].to_string(), t[3..5].to_string());
    let en = src["enabled"] != false;
    match key {
        "r" => {
            p.set_r_enabled(en);
            p.set_r_hh(ss(h));
            p.set_r_mm(ss(m));
            p.set_r_tt(rule["tien_thuong"] == true);
        }
        "r2" => {
            p.set_r2_enabled(en);
            p.set_r2_hh(ss(h));
            p.set_r2_mm(ss(m));
        }
        _ => {
            p.set_er_enabled(en);
            p.set_er_hh(ss(h));
            p.set_er_mm(ss(m));
        }
    }
}

// ------------------------------------------------------------------------------------------------
// Biểu mẫu người
// ------------------------------------------------------------------------------------------------

fn clear_errors(p: &People) {
    for f in [
        People::set_e_danh_xung,
        People::set_e_ho_ten,
        People::set_e_ten_hieu,
        People::set_e_quan_he,
        People::set_e_ghi_chu,
        People::set_e_noi_sinh,
        People::set_e_tuoi,
        People::set_e_noi_an_tang,
        People::set_e_photo,
        People::set_e_notice,
        People::set_r_error,
        People::set_r_warn,
        People::set_r2_error,
        People::set_e_nguong,
        People::set_e_new_type,
        People::set_e_title,
        People::set_er_error,
        People::set_e_ev_ghi_chu,
        People::set_dd_error,
        People::set_bd_error,
        People::set_md_error,
        People::set_ed_error,
        People::set_f_summary,
    ] {
        f(p, ss(""));
    }
}

fn clear_notice(ui: &AppWindow, ps: &mut PState) {
    let p = People::get(ui);
    for f in [
        People::set_n_by,
        People::set_n_place,
        People::set_n_d,
        People::set_n_m,
        People::set_n_y,
        People::set_n_reason,
        People::set_n_note,
    ] {
        f(&p, ss(""));
    }
    ps.certs.clear();
    p.set_n_photos(vm(vec![]));
}

fn open_person_form(ui: &AppWindow, ps: &mut PState, dead: bool, base: Option<Value>) {
    let Some(store) = ps.store.clone() else { return };
    let p = People::get(ui);
    ps.filling = true;
    clear_errors(&p);
    clear_notice(ui, ps);
    let settings = jv(&store.settings_json().unwrap_or_default());
    let b = base.clone().unwrap_or(Value::Null);
    let edit = base.is_some();
    p.set_f_dead(dead);
    p.set_f_title(ss(match (edit, dead) {
        (true, _) => "Sửa hồ sơ",
        (false, true) => "Thêm người đã mất",
        (false, false) => "Thêm người đang sống",
    }));
    p.set_f_subtitle(ss(if edit { display_name(&b) } else { String::new() }));
    p.set_f_danh_xung(ss(upper_first(&st(&b["danh_xung"]))));
    p.set_dx_rows(vm(chip_rows(&dx_suggestions(dead), 44)));
    p.set_gender_ok(gender_supported());
    p.set_f_gender(match b["gioi_tinh"].as_str() {
        Some("nam") => 0,
        Some("nu") => 1,
        _ => 2,
    });
    p.set_f_ho_ten(ss(st(&b["ho_ten"])));
    p.set_f_ten_hieu(ss(st(&b["ten_hieu"])));
    p.set_f_quan_he(ss(upper_first(&st(&b["quan_he"]))));
    p.set_f_ghi_chu(ss(st(&b["ghi_chu"])));
    let dd = fill_date(&b["death"]["input"]);
    p.set_dd_cal(dd.cal);
    p.set_dd_d(ss(dd.d));
    p.set_dd_m(ss(dd.m));
    p.set_dd_y(ss(dd.y));
    p.set_dd_leap(dd.leap);
    p.set_dd_unknown(dd.unknown);
    let bd = fill_date(&b["birth"]["input"]);
    p.set_bd_cal(bd.cal);
    p.set_bd_prec(bd.prec);
    p.set_bd_d(ss(bd.d));
    p.set_bd_m(ss(bd.m));
    p.set_bd_y(ss(bd.y));
    p.set_bd_leap(bd.leap);
    p.set_f_noi_sinh(ss(st(&b["birth"]["noi_sinh"])));
    let manual = b["death"]["tuoi"]["manual"] == true;
    p.set_f_tuoi_manual(manual);
    p.set_f_tuoi(ss(b["death"]["tuoi"]["value"]
        .as_i64()
        .map(|x| x.to_string())
        .unwrap_or_default()));
    p.set_f_noi_an_tang(ss(st(&b["noi_an_tang"])));
    ps.photo = PhotoSel::Keep(b["photo_id"].as_str().map(String::from));
    let img = b["photo_id"].as_str().and_then(|i| photo_image(ps, i));
    p.set_f_has_photo(img.is_some());
    p.set_f_photo(img.unwrap_or_default());
    // Thông tin báo tử: thêm mới → thu gọn; sửa hồ sơ đã có → mở sẵn.
    let n = &b["death"]["notice"];
    p.set_n_open(n.is_object());
    if n.is_object() {
        p.set_n_by(ss(st(&n["reported_by"])));
        p.set_n_place(ss(st(&n["place"])));
        p.set_n_reason(ss(st(&n["reason"])));
        p.set_n_note(ss(st(&n["note"])));
        let nd = fill_date(&n["notice_date"]);
        p.set_n_d(ss(nd.d));
        p.set_n_m(ss(nd.m));
        p.set_n_y(ss(nd.y));
        for id in n["certificate_photo_ids"].as_array().into_iter().flatten() {
            if let Some(i) = id.as_str() {
                if let Some(img) = photo_image(ps, i) {
                    ps.certs.push(CertPhoto {
                        id: Some(i.into()),
                        enc: None,
                        img,
                    });
                }
            }
        }
        p.set_n_photos(vm(ps.certs.iter().map(|c| c.img.clone()).collect()));
    }
    let r = &b["reminders"];
    if dead {
        fill_rule(ui, ps, "r", &r["gio"], &settings);
    } else {
        fill_rule(ui, ps, "r", &r["sinh_nhat"], &settings);
        let am = &r["sinh_nhat_am"];
        let same = !am.is_object()
            || (am["enabled"] == r["sinh_nhat"]["enabled"]
                && am["offsets_days"] == r["sinh_nhat"]["offsets_days"]
                && am["time_local"] == r["sinh_nhat"]["time_local"]);
        fill_rule(ui, ps, "r2", am, &settings);
        p.set_f_am(b["sinh_nhat_am"] == true);
        p.set_f_am_same(same);
    }
    p.set_f_feb29_rule(if b["feb29_rule"] == "mar1" { 1 } else { 0 });
    let m = &b["memorial"];
    p.set_f_mem_open(false);
    p.set_f_gio_het(m["gio_het_nam"].as_i64().unwrap_or(2) as i32);
    p.set_f_loi(ss(st(&m["loi_tuong_niem"])));
    p.set_f_nguong(ss(m["huong_duong_duoi"].as_i64().unwrap_or(60).to_string()));
    ps.form = FormKind::Person { dead, base };
    ps.tried = false;
    ps.dirty = false;
    ps.filling = false;
    form_changed(ui, ps);
    ps.dirty = false;
    show(ui, ps, "form");
}

fn text_err(v: &str, max: usize, req: bool, label: &str) -> String {
    let n = v.trim().chars().count();
    if req && n == 0 {
        return format!("{label} không được để trống");
    }
    if n > max {
        return format!("{label} tối đa {max} ký tự");
    }
    String::new()
}

/// Thông tin báo tử từ ô nhập. Lỗi → chuỗi.
fn notice_from(p: &People, today: D) -> Result<Option<Value>, String> {
    let (by, place, reason, note) = (
        p.get_n_by().to_string(),
        p.get_n_place().to_string(),
        p.get_n_reason().to_string(),
        p.get_n_note().to_string(),
    );
    for (v, max, l) in [
        (&by, 200, "Người báo tin"),
        (&place, 200, "Nơi mất"),
        (&reason, 500, "Lý do"),
        (&note, 2000, "Ghi chú báo tử"),
    ] {
        let e = text_err(v, max, false, l);
        if !e.is_empty() {
            return Err(e);
        }
    }
    let f = DateFields {
        cal: 0,
        prec: 0,
        d: p.get_n_d().into(),
        m: p.get_n_m().into(),
        y: p.get_n_y().into(),
        leap: false,
        unknown: false,
    };
    let date = match date_value(&f, "death") {
        DateRes::Empty => Value::Null,
        DateRes::Incomplete(e) | DateRes::Bad(e) => return Err(format!("Ngày nhận tin: {e}")),
        DateRes::Ok(v) => {
            let d = D {
                y: v["year"].as_i64().unwrap_or(0) as i32,
                m: v["month"].as_i64().unwrap_or(0) as i32,
                d: v["day"].as_i64().unwrap_or(0) as i32,
            };
            if d > today {
                return Err("Ngày nhận tin không được sau hôm nay".into());
            }
            v
        }
    };
    let empty = [&by, &place, &reason, &note].iter().all(|s| s.trim().is_empty())
        && date.is_null()
        && p.get_n_photos().row_count() == 0;
    if empty {
        return Ok(None);
    }
    Ok(Some(json!({
        "reported_by": by, "place": place, "reason": reason, "note": note,
        "notice_date": date, "certificate_photo_ids": [],
    })))
}

/// Kiểm toàn bộ biểu mẫu người; trả JSON để lưu (chưa có id ảnh mới) hoặc số lỗi.
fn build_person_json(ui: &AppWindow, ps: &mut PState, show_err: bool) -> Result<Value, usize> {
    let p = People::get(ui);
    let FormKind::Person { dead, base } = ps.form.clone() else {
        return Err(1);
    };
    let today = local_today();
    let mut errs = 0;
    macro_rules! set {
        ($f:ident, $e:expr) => {{
            let e: String = $e;
            if !e.is_empty() {
                errs += 1;
            }
            p.$f(ss(if show_err { e } else { String::new() }));
        }};
    }
    set!(set_e_danh_xung, text_err(&p.get_f_danh_xung(), 40, false, "Cách gọi"));
    set!(set_e_ho_ten, text_err(&p.get_f_ho_ten(), 120, true, "Họ tên"));
    set!(set_e_ten_hieu, text_err(&p.get_f_ten_hieu(), 120, false, "Tên hiệu"));
    set!(set_e_quan_he, text_err(&p.get_f_quan_he(), 120, false, "Quan hệ"));
    set!(set_e_ghi_chu, text_err(&p.get_f_ghi_chu(), 2000, false, "Ghi chú"));
    set!(set_e_noi_sinh, text_err(&p.get_f_noi_sinh(), 300, false, "Nơi sinh"));
    set!(set_e_noi_an_tang, text_err(&p.get_f_noi_an_tang(), 300, false, "Nơi an táng"));
    // Ngày mất / ngày sinh
    let ddv = date_view(&fields_dd(&p), "death", today, false, show_err);
    let bdv = date_view(&fields_bd(&p), "birth", today, !dead, show_err);
    if dead {
        set_dd(&p, &ddv);
        if ddv.value.is_none() {
            errs += 1;
        }
    }
    set_bd(&p, &bdv);
    let bd_empty = matches!(date_value(&fields_bd(&p), "birth"), DateRes::Empty);
    if !bdv.error.is_empty() || (!bd_empty && bdv.value.is_none()) {
        errs += 1;
    }
    let mut birth_err = String::new();
    if bd_empty && !p.get_f_noi_sinh().trim().is_empty() {
        birth_err = "Hãy nhập ngày sinh hoặc năm sinh để lưu nơi sinh".into();
        errs += 1;
        if show_err {
            p.set_bd_error(ss(&birth_err));
        }
    }
    // Năm âm mất < năm âm sinh
    if dead {
        if let (Some(d), Some(b)) = (&ddv.value, &bdv.value) {
            let dl = lvhv_ffi::save_death_json(d.to_string()).ok().map(|j| jv(&j));
            let bl = lvhv_ffi::derive_birth_json(b.to_string()).ok().map(|j| jv(&j));
            if let (Some(dy), Some(by)) = (
                dl.as_ref().and_then(|x| x["lunar"]["year"].as_i64()),
                bl.as_ref().and_then(|x| x["lunar_year"].as_i64()),
            ) {
                if dy < by {
                    errs += 1;
                    if show_err {
                        p.set_dd_error(ss("Ngày mất trước ngày sinh"));
                    }
                } else if dy - by + 1 > 150 {
                    errs += 1;
                    if show_err {
                        p.set_dd_error(ss("Tuổi tính ra lớn hơn 150 — hãy kiểm tra năm sinh và năm mất"));
                    }
                }
            }
        }
    }
    let _ = birth_err;
    // Nhắc
    let b = base.clone().unwrap_or(json!({}));
    let settings = jv(&ps
        .s()
        .and_then(|s| s.settings_json().ok())
        .unwrap_or_default());
    let main_old = if dead {
        &b["reminders"]["gio"]
    } else {
        &b["reminders"]["sinh_nhat"]
    };
    let r1 = rule_from(
        ps,
        "r",
        p.get_r_enabled(),
        &p.get_r_custom(),
        &p.get_r_hh(),
        &p.get_r_mm(),
        dead && p.get_r_tt(),
        main_old,
    );
    match &r1 {
        Ok((_, w)) => {
            p.set_r_error(ss(""));
            p.set_r_warn(ss(w));
        }
        Err(e) => {
            errs += 1;
            p.set_r_error(ss(e));
        }
    }
    let am_allowed = !dead && bdv.value.as_ref().is_some_and(|v| v["precision"] == "day");
    p.set_f_am_allowed(am_allowed);
    let mut r2 = Ok((Value::Null, String::new()));
    if !dead && am_allowed && p.get_f_am() {
        r2 = if p.get_f_am_same() {
            r1.clone().map(|(mut r, w)| {
                r["id"] = b["reminders"]["sinh_nhat_am"]["id"]
                    .as_str()
                    .map(|s| json!(s))
                    .unwrap_or_else(|| json!(lvhv_ffi::new_uuid().unwrap_or_default()));
                (r, w)
            })
        } else {
            rule_from(
                ps,
                "r2",
                p.get_r2_enabled(),
                &p.get_r2_custom(),
                &p.get_r2_hh(),
                &p.get_r2_mm(),
                false,
                &b["reminders"]["sinh_nhat_am"],
            )
        };
        match &r2 {
            Ok(_) => p.set_r2_error(ss("")),
            Err(e) => {
                errs += 1;
                p.set_r2_error(ss(e));
            }
        }
    }
    // Ngưỡng hưởng dương, tuổi tự nhập
    let nguong = num(&p.get_f_nguong()).filter(|n| (1..=150).contains(n));
    if dead && nguong.is_none() {
        errs += 1;
        p.set_e_nguong(ss("Tuổi phải từ 1 đến 150"));
    } else {
        p.set_e_nguong(ss(""));
    }
    let tuoi_manual = dead && p.get_f_tuoi_manual();
    let tuoi = num(&p.get_f_tuoi()).filter(|n| (1..=150).contains(n));
    if tuoi_manual && tuoi.is_none() {
        errs += 1;
        if show_err {
            p.set_e_tuoi(ss("Tuổi phải từ 1 đến 150"));
        }
    } else {
        p.set_e_tuoi(ss(""));
    }
    let notice = if dead {
        match notice_from(&p, today) {
            Ok(n) => {
                p.set_e_notice(ss(""));
                n
            }
            Err(e) => {
                errs += 1;
                p.set_e_notice(ss(e));
                None
            }
        }
    } else {
        None
    };
    if errs > 0 {
        return Err(errs);
    }
    let mut v = b.clone();
    let o = v.as_object_mut().ok_or(1usize)?;
    o.insert("kind".into(), json!(if dead { "deceased" } else { "living" }));
    o.insert("danh_xung".into(), json!(cap_first(&p.get_f_danh_xung())));
    o.insert("ho_ten".into(), json!(p.get_f_ho_ten().trim()));
    o.insert("ten_hieu".into(), json!(p.get_f_ten_hieu().trim()));
    o.insert("quan_he".into(), json!(cap_first(&p.get_f_quan_he())));
    if gender_supported() {
        o.insert(
            "gioi_tinh".into(),
            match p.get_f_gender() {
                0 => json!("nam"),
                1 => json!("nu"),
                _ => Value::Null,
            },
        );
    } else {
        o.remove("gioi_tinh");
    }
    o.insert("ghi_chu".into(), json!(p.get_f_ghi_chu().trim()));
    o.insert(
        "birth".into(),
        match &bdv.value {
            Some(bv) => json!({"input": bv, "noi_sinh": p.get_f_noi_sinh().trim()}),
            None => Value::Null,
        },
    );
    let (r1v, _) = r1.map_err(|_| 1usize)?;
    if dead {
        let mut death = json!({"input": ddv.value});
        if tuoi_manual {
            let t = tuoi.unwrap_or(1);
            death["tuoi"] = json!({"value": t, "label": if t < nguong.unwrap_or(60) {"huong_duong"} else {"huong_tho"}, "manual": true, "approx": false});
        }
        if let Some(n) = notice {
            death["notice"] = n;
        }
        o.insert("death".into(), death);
        o.insert("noi_an_tang".into(), json!(p.get_f_noi_an_tang().trim()));
        let mut mem = b["memorial"].clone();
        if !mem.is_object() {
            mem = json!({"tray_style": "truyen_thong", "incense": true, "bia_tieu_de": ""});
        }
        mem["gio_het_nam"] = json!(p.get_f_gio_het());
        mem["loi_tuong_niem"] = json!(p.get_f_loi().trim());
        mem["huong_duong_duoi"] = json!(nguong.unwrap_or(60));
        o.insert("memorial".into(), mem);
        o.insert(
            "reminders".into(),
            json!({"gio": r1v, "sinh_nhat": null, "sinh_nhat_am": null}),
        );
        o.insert("sinh_nhat_am".into(), json!(false));
    } else {
        o.insert("death".into(), Value::Null);
        o.insert("noi_an_tang".into(), json!(""));
        o.insert("memorial".into(), Value::Null);
        let am = am_allowed && p.get_f_am();
        let r2v = if am {
            r2.map(|x| x.0).unwrap_or(Value::Null)
        } else {
            Value::Null
        };
        o.insert(
            "reminders".into(),
            json!({"gio": null, "sinh_nhat": r1v, "sinh_nhat_am": r2v}),
        );
        o.insert("sinh_nhat_am".into(), json!(am));
    }
    o.insert(
        "feb29_rule".into(),
        json!(if p.get_f_feb29_rule() == 1 { "mar1" } else { "feb28" }),
    );
    if !o.contains_key("photo_id") {
        o.insert("photo_id".into(), Value::Null);
    }
    let _ = settings;
    Ok(v)
}

fn form_changed(ui: &AppWindow, ps: &mut PState) {
    if ps.filling {
        return;
    }
    ps.dirty = true;
    let p = People::get(ui);
    let today = local_today();
    match ps.form.clone() {
        FormKind::Person { dead, .. } => {
            let bd = fields_bd(&p);
            p.set_f_feb29(!dead && bd.cal == 0 && bd.prec == 0 && bd.d.trim() == "29" && bd.m.trim() == "2");
            let r = build_person_json(ui, ps, ps.tried);
            // Dòng tuổi (người đã mất)
            if dead {
                let line = match &r {
                    Ok(v) => lvhv_ffi::derive_person_json(v.to_string())
                        .ok()
                        .and_then(|j| lvhv_ffi::person_summary_json(j, today.solar()).ok())
                        .and_then(|j| jv(&j)["tuoi_mat_text"].as_str().map(String::from)),
                    Err(_) => None,
                };
                p.set_f_tuoi_line(ss(line.unwrap_or_else(|| "Chưa đủ năm sinh để tính tuổi".into())));
            }
            update_summary(ui, ps, r.err());
        }
        FormKind::Event { .. } => {
            let r = build_event_json(ui, ps, ps.tried);
            update_summary(ui, ps, r.err());
        }
        FormKind::Mark { .. } => {
            let v = date_view(&fields_md(&p), "death", today, false, ps.tried);
            set_md(&p, &v);
            let mut errs = usize::from(v.value.is_none());
            match notice_from(&p, today) {
                Ok(_) => p.set_e_notice(ss("")),
                Err(e) => {
                    errs += 1;
                    p.set_e_notice(ss(e));
                }
            }
            update_summary(ui, ps, (errs > 0).then_some(errs));
        }
        FormKind::None => {}
    }
}

/// Khung tóm tắt lỗi (08 §2, TKN-W-05): "Còn n chỗ cần sửa" + từng lỗi, bấm để cuộn tới chỗ đó. Chỉ hiện sau lần bấm
/// Lưu đầu tiên; cập nhật theo từng lần gõ (TK-03).
fn update_summary(ui: &AppWindow, ps: &PState, errs: Option<usize>) {
    let p = People::get(ui);
    let mut items: Vec<ErrItem> = Vec::new();
    if ps.tried && errs.is_some_and(|n| n > 0) {
        let fields: Vec<(&str, &str, SharedString)> = vec![
            ("ten", "Họ và tên", p.get_e_ho_ten()),
            ("ten", "Cách gọi", p.get_e_danh_xung()),
            ("ten", "Tên hiệu", p.get_e_ten_hieu()),
            ("ten", "Quan hệ", p.get_e_quan_he()),
            ("ten", "Ghi chú", p.get_e_ghi_chu()),
            ("ngay-mat", "Ngày mất", p.get_dd_error()),
            ("ngay-mat", "Ngày mất", p.get_md_error()),
            ("ngay-sinh", "Ngày sinh", p.get_bd_error()),
            ("ngay-sinh", "Nơi sinh", p.get_e_noi_sinh()),
            ("ngay-sinh", "Tuổi", p.get_e_tuoi()),
            ("an-tang", "Nơi an táng", p.get_e_noi_an_tang()),
            ("an-tang", "Ảnh", p.get_e_photo()),
            ("bao-tu", "Thông tin báo tử", p.get_e_notice()),
            ("nhac", "Nhắc", p.get_r_error()),
            ("nhac", "Nhắc sinh nhật âm lịch", p.get_r2_error()),
            ("tuong-niem", "Tưởng niệm", p.get_e_nguong()),
            ("ten", "Tên loại mới", p.get_e_new_type()),
            ("ten", "Tiêu đề", p.get_e_title()),
            ("ten", "Ghi chú", p.get_e_ev_ghi_chu()),
            ("ngay", "Ngày", p.get_ed_error()),
            ("nhac", "Nhắc", p.get_er_error()),
        ];
        // Mỗi biểu mẫu xóa hết lỗi khi mở (clear_errors), nên chỉ lỗi của biểu mẫu đang mở còn chữ.
        for (key, label, e) in fields {
            if !e.is_empty() && !items.iter().any(|i| i.text.ends_with(e.as_str())) {
                items.push(ErrItem { text: ss(format!("{label}: {e}")), key: ss(key) });
            }
        }
    }
    let n = errs.unwrap_or(0).max(items.len());
    p.set_f_summary(ss(if ps.tried && n > 0 { format!("Còn {n} chỗ cần sửa") } else { String::new() }));
    p.set_f_errors(vm(items));
}

/// Vị trí cuộn gần đúng của từng nhóm trong biểu mẫu (0 = đầu, 1 = cuối).
fn error_scroll(key: &str) -> f32 {
    match key {
        "ten" => 0.0,
        "ngay-mat" | "ngay" => 0.2,
        "ngay-sinh" => 0.35,
        "an-tang" => 0.5,
        "bao-tu" => 0.6,
        "nhac" => 0.8,
        _ => 1.0,
    }
}

fn save_photo(store: &LvhvStore, e: &photo::Encoded) -> Result<String, String> {
    let id = lvhv_ffi::new_uuid().map_err(|e| e.to_string())?;
    store
        .add_photo(id.clone(), "image/jpeg".into(), e.width, e.height, e.bytes.clone())
        .map_err(|e| e.to_string())?;
    Ok(id)
}

/// Lưu ảnh giấy báo tử mới; trả danh sách id theo thứ tự trên màn hình.
fn save_certs(store: &LvhvStore, certs: &[CertPhoto]) -> Result<Vec<String>, String> {
    certs
        .iter()
        .map(|c| match (&c.id, &c.enc) {
            (Some(id), _) => Ok(id.clone()),
            (None, Some(e)) => save_photo(store, e),
            _ => Err("Ảnh không hợp lệ".into()),
        })
        .collect()
}

fn save_form(ui: &AppWindow, ps: &mut PState) {
    ps.tried = true;
    let Some(store) = ps.store.clone() else { return };
    match ps.form.clone() {
        FormKind::Person { dead, .. } => {
            let r = build_person_json(ui, ps, true);
            let Ok(mut v) = r else {
                update_summary(ui, ps, r.err());
                return;
            };
            let res = (|| -> Result<String, String> {
                match &ps.photo {
                    PhotoSel::Keep(id) => v["photo_id"] = json!(id),
                    PhotoSel::New(e) => v["photo_id"] = json!(save_photo(&store, e)?),
                }
                if dead && v["death"]["notice"].is_object() {
                    v["death"]["notice"]["certificate_photo_ids"] = json!(save_certs(&store, &ps.certs)?);
                }
                store.save_person_json(v.to_string()).map_err(|e| e.to_string())
            })();
            match res {
                Ok(j) => {
                    let id = st(&jv(&j)["id"]);
                    ps.form = FormKind::None;
                    ps.stack.retain(|s| s != "form");
                    after_change(ui, ps);
                    ps.person = Some(id.clone());
                    render_person(ui, ps, &id);
                    let p = People::get(ui);
                    if ps.stack.last().map(String::as_str) == Some("nguoi") {
                        ps.stack.pop();
                    }
                    p.set_back_label(ss(back_label(ps.stack.last().map(String::as_str).unwrap_or("ho-so"))));
                    p.set_scroll_frac(0.0);
                    p.set_page(ss("nguoi"));
                    p.set_f_summary(ss(""));
                    toast(ui, ps, "Đã lưu hồ sơ.");
                }
                Err(e) => fail(ui, ps, format!("Chưa lưu được: {e}. Dữ liệu đang nhập vẫn còn trên màn hình.")),
            }
        }
        FormKind::Event { person, .. } => {
            let r = build_event_json(ui, ps, true);
            let Ok(mut v) = r else {
                update_summary(ui, ps, r.err());
                return;
            };
            let p = People::get(ui);
            let res = (|| -> Result<String, String> {
                if p.get_ev_other() {
                    let name = p.get_ev_new_type().trim().to_string();
                    let types = jv(&store.event_types_json().map_err(|e| e.to_string())?);
                    let found = types.as_array().into_iter().flatten().find(|t| {
                        t["ten"].as_str().is_some_and(|n| n.to_lowercase() == name.to_lowercase())
                    });
                    let tid = match found {
                        Some(t) => st(&t["id"]),
                        None => st(&jv(&store
                            .save_event_type_json(json!({"builtin": null, "ten": name, "default_recurring": p.get_ev_recurring()}).to_string())
                            .map_err(|e| e.to_string())?)["id"]),
                    };
                    v["type_id"] = json!(tid);
                }
                store.save_event_json(v.to_string()).map_err(|e| e.to_string())
            })();
            match res {
                Ok(_) => {
                    ps.form = FormKind::None;
                    after_change(ui, ps);
                    ps.person = Some(person.clone());
                    render_person(ui, ps, &person);
                    back(ui, ps);
                    toast(ui, ps, "Đã lưu sự kiện.");
                }
                Err(e) => fail(ui, ps, format!("Chưa lưu được: {e}. Dữ liệu đang nhập vẫn còn trên màn hình.")),
            }
        }
        FormKind::Mark { person } => {
            let p = People::get(ui);
            let today = local_today();
            let v = date_view(&fields_md(&p), "death", today, false, true);
            set_md(&p, &v);
            let notice = notice_from(&p, today);
            if let Err(e) = &notice {
                p.set_e_notice(ss(e));
            }
            let (Some(dv), Ok(notice)) = (v.value, notice) else {
                update_summary(ui, ps, Some(1));
                return;
            };
            update_summary(ui, ps, None);
            // Hộp xác nhận (§12g).
            let name = display_name(&person);
            let lunar = lvhv_ffi::save_death_json(dv.to_string())
                .map(|j| lunar_text(&jv(&j)["lunar"], false))
                .unwrap_or_default();
            let settings = jv(&store.settings_json().unwrap_or_default());
            let rule = json!({"offsets_days": settings["default_offsets_days"], "time_local": settings["default_time_local"]});
            let has_events = !events_of(ps, &st(&person["id"])).is_empty();
            let mut lines = vec![
                format!(
                    "Sẽ không còn nhắc sinh nhật{} của {name}.",
                    if has_events { " và sự kiện" } else { "" }
                ),
                format!(
                    "Bắt đầu nhắc ngày giỗ hằng năm vào {lunar} ({}).",
                    offsets_text(&rule).to_lowercase()
                ),
                "Họ tên, ảnh, ngày sinh và sự kiện vẫn được giữ trong hồ sơ.".into(),
            ];
            if notice.is_some() {
                lines.push("Thông tin báo tử chỉ hiện trên trang hồ sơ này.".into());
            }
            open_dialog(
                ui,
                ps,
                "xac-nhan",
                &format!("Cập nhật {name} là người đã mất?"),
                lines,
                "Lưu thay đổi",
                "Quay lại",
                false,
                DlgAction::Mark(dv, notice),
            );
        }
        FormKind::None => {}
    }
}

fn do_mark(ui: &AppWindow, ps: &mut PState, dv: Value, notice: Option<Value>) {
    let Some(store) = ps.store.clone() else { return };
    let FormKind::Mark { person } = ps.form.clone() else {
        return;
    };
    let id = st(&person["id"]);
    let res = (|| -> Result<String, String> {
        let n = match notice {
            Some(mut n) => {
                n["certificate_photo_ids"] = json!(save_certs(&store, &ps.certs)?);
                Some(n.to_string())
            }
            None => None,
        };
        store
            .mark_deceased_json(id.clone(), dv.to_string(), n, local_today().solar())
            .map_err(|e| e.to_string())
    })();
    match res {
        Ok(_) => {
            ps.form = FormKind::None;
            after_change(ui, ps);
            ps.person = Some(id.clone());
            render_person(ui, ps, &id);
            back(ui, ps);
            toast(ui, ps, "Đã cập nhật hồ sơ.");
        }
        Err(e) => fail(ui, ps, format!("Chưa lưu được: {e}. Dữ liệu đang nhập vẫn còn trên màn hình.")),
    }
}

fn open_mark(ui: &AppWindow, ps: &mut PState) {
    let Some(id) = ps.person.clone() else { return };
    let Some(v) = person(ps, &id) else { return };
    let p = People::get(ui);
    ps.filling = true;
    clear_errors(&p);
    clear_notice(ui, ps);
    p.set_md_name(ss(display_name(&v)));
    p.set_md_cal(0);
    for f in [People::set_md_d, People::set_md_m, People::set_md_y] {
        f(&p, ss(""));
    }
    p.set_md_leap(false);
    p.set_md_unknown(false);
    p.set_n_open(true);
    ps.form = FormKind::Mark { person: v };
    ps.tried = false;
    ps.filling = false;
    form_changed(ui, ps);
    ps.dirty = false;
    show(ui, ps, "bao-tu");
    p.set_back_label(ss("‹ Về hồ sơ"));
}

// ------------------------------------------------------------------------------------------------
// Biểu mẫu sự kiện
// ------------------------------------------------------------------------------------------------

fn event_types(ps: &PState) -> Vec<Value> {
    ps.s()
        .and_then(|s| s.event_types_json().ok())
        .map(|j| jv(&j).as_array().cloned().unwrap_or_default())
        .unwrap_or_default()
}

fn open_event_form(ui: &AppWindow, ps: &mut PState, base: Option<Value>) {
    let Some(pid) = ps.person.clone() else { return };
    let Some(pv) = person(ps, &pid) else { return };
    let p = People::get(ui);
    ps.filling = true;
    clear_errors(&p);
    let types = event_types(ps);
    let mut names: Vec<SharedString> = types.iter().map(|t| ss(st(&t["ten"]))).collect();
    names.push(ss("Loại khác…"));
    p.set_ev_types(vm(names));
    let b = base.clone().unwrap_or(Value::Null);
    let idx = types
        .iter()
        .position(|t| t["id"] == b["type_id"])
        .unwrap_or(0);
    p.set_ev_type(idx as i32);
    p.set_ev_other(false);
    p.set_ev_new_type(ss(""));
    let dead = pv["kind"] == "deceased";
    p.set_ev_dead(dead);
    p.set_ev_heading(ss(format!(
        "{} · {}",
        if base.is_some() { "Sửa sự kiện" } else { "Thêm sự kiện" },
        display_name(&pv)
    )));
    p.set_ev_title(ss(st(&b["tieu_de"])));
    p.set_ev_ghi_chu(ss(st(&b["ghi_chu"])));
    let d = fill_date(&b["date"]);
    p.set_ed_cal(d.cal);
    p.set_ed_d(ss(d.d));
    p.set_ed_m(ss(d.m));
    p.set_ed_y(ss(d.y));
    p.set_ed_leap(d.leap);
    p.set_ed_unknown(d.unknown);
    p.set_ev_recurring(if base.is_some() {
        b["recurring"] == true
    } else {
        types.get(idx).is_some_and(|t| t["default_recurring"] == true)
    });
    let settings = jv(&ps
        .s()
        .and_then(|s| s.settings_json().ok())
        .unwrap_or_default());
    fill_rule(ui, ps, "er", &b["reminder"], &settings);
    if base.is_some() && !b["reminder"].is_object() {
        p.set_er_enabled(false);
    }
    ps.form = FormKind::Event { person: pid, base };
    ps.tried = false;
    ps.filling = false;
    form_changed(ui, ps);
    ps.dirty = false;
    show(ui, ps, "su-kien");
    p.set_back_label(ss("‹ Về hồ sơ"));
}

fn build_event_json(ui: &AppWindow, ps: &mut PState, show_err: bool) -> Result<Value, usize> {
    let p = People::get(ui);
    let FormKind::Event { person, base } = ps.form.clone() else {
        return Err(1);
    };
    let types = event_types(ps);
    let ti = p.get_ev_type().max(0) as usize;
    let other = ti >= types.len();
    p.set_ev_other(other);
    let mut errs = 0;
    let e_new = if other {
        let n = p.get_ev_new_type();
        if n.trim().is_empty() {
            "Hãy đặt tên cho loại sự kiện".to_string()
        } else {
            text_err(&n, 60, false, "Tên loại")
        }
    } else {
        String::new()
    };
    if !e_new.is_empty() {
        errs += 1;
    }
    p.set_e_new_type(ss(if show_err { e_new } else { String::new() }));
    let e_t = text_err(&p.get_ev_title(), 120, false, "Tiêu đề");
    if !e_t.is_empty() {
        errs += 1;
    }
    p.set_e_title(ss(e_t));
    let e_g = text_err(&p.get_ev_ghi_chu(), 2000, false, "Ghi chú");
    if !e_g.is_empty() {
        errs += 1;
    }
    p.set_e_ev_ghi_chu(ss(e_g));
    let today = local_today();
    let f = fields_ed(&p);
    let mut dv = date_view(&f, "event", today, false, show_err);
    if p.get_ed_unknown() && p.get_ed_cal() == 1 && !p.get_ev_recurring() {
        dv.error = "Sự kiện không rõ năm phải lặp hằng năm".into();
        dv.value = None;
    }
    set_ed(&p, &dv);
    if dv.value.is_none() {
        errs += 1;
    }
    let b = base.clone().unwrap_or(json!({}));
    let dead = p.get_ev_dead();
    let reminder = if dead {
        // B4: không có khối nhắc; quy tắc đang lưu (nếu có) giữ nguyên.
        Ok(b["reminder"].clone())
    } else {
        rule_from(
            ps,
            "er",
            p.get_er_enabled(),
            &p.get_er_custom(),
            &p.get_er_hh(),
            &p.get_er_mm(),
            false,
            &b["reminder"],
        )
        .map(|x| x.0)
    };
    match &reminder {
        Ok(_) => p.set_er_error(ss("")),
        Err(e) => {
            errs += 1;
            p.set_er_error(ss(e));
        }
    }
    if errs > 0 {
        return Err(errs);
    }
    let mut v = b;
    let o = v.as_object_mut().ok_or(1usize)?;
    o.insert(
        "type_id".into(),
        types
            .get(ti)
            .map(|t| t["id"].clone())
            .unwrap_or(json!("6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0001")),
    );
    o.insert("tieu_de".into(), json!(p.get_ev_title().trim()));
    o.insert("date".into(), dv.value.unwrap_or(Value::Null));
    o.insert("recurring".into(), json!(p.get_ev_recurring()));
    if !o.contains_key("person_ids") {
        o.insert("person_ids".into(), json!([person]));
    }
    o.insert("ghi_chu".into(), json!(p.get_ev_ghi_chu().trim()));
    o.insert("reminder".into(), reminder.map_err(|_| 1usize)?);
    Ok(v)
}

// ------------------------------------------------------------------------------------------------
// Hộp thoại
// ------------------------------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn open_dialog(
    ui: &AppWindow,
    ps: &mut PState,
    kind: &str,
    title: &str,
    lines: Vec<String>,
    ok: &str,
    cancel: &str,
    danger: bool,
    action: DlgAction,
) {
    let p = People::get(ui);
    p.set_dlg_title(ss(title));
    p.set_dlg_lines(vm(lines.into_iter().map(ss).collect()));
    p.set_dlg_ok(ss(ok));
    p.set_dlg_cancel(ss(cancel));
    p.set_dlg_danger(danger);
    p.set_dlg_busy(false);
    p.set_dlg(ss(kind));
    ps.dlg = action;
}

fn close_dialog(ui: &AppWindow, ps: &mut PState) {
    let p = People::get(ui);
    p.set_dlg(ss(""));
    p.set_pw1(ss(""));
    p.set_pw2(ss(""));
    p.set_e_pw1(ss(""));
    p.set_e_pw2(ss(""));
    ps.dlg = DlgAction::None;
}

fn dlg_confirm(ui: &AppWindow, ps: &mut PState) {
    let p = People::get(ui);
    let Some(store) = ps.store.clone() else { return };
    match std::mem::replace(&mut ps.dlg, DlgAction::None) {
        DlgAction::None => close_dialog(ui, ps),
        DlgAction::DeletePerson(id) => {
            close_dialog(ui, ps);
            let orphan: Vec<String> = events_of(ps, &id)
                .iter()
                .filter(|e| e["person_ids"].as_array().is_some_and(|a| a.len() == 1))
                .map(|e| st(&e["id"]))
                .collect();
            match store.delete_person(id) {
                Ok(()) => {
                    for e in orphan {
                        let _ = store.delete_event(e);
                    }
                    ps.person = None;
                    after_change(ui, ps);
                    show(ui, ps, "ho-so");
                    toast(ui, ps, "Đã xóa hồ sơ.");
                }
                Err(e) => toast(ui, ps, &format!("Chưa xóa được: {e}")),
            }
        }
        DlgAction::DeleteEvent(id) => {
            close_dialog(ui, ps);
            match store.delete_event(id) {
                Ok(()) => {
                    after_change(ui, ps);
                    toast(ui, ps, "Đã xóa sự kiện.");
                }
                Err(e) => toast(ui, ps, &format!("Chưa xóa được: {e}")),
            }
        }
        DlgAction::Ics(name, content) => {
            close_dialog(ui, ps);
            if let Some(path) = platform::pick(platform::hwnd_of(ui.window()), Dialog::Save(name)) {
                match std::fs::write(&path, content.as_bytes()) {
                    Ok(()) => toast(ui, ps, "Đã lưu tệp lịch. Mở tệp bằng Outlook hoặc ứng dụng Lịch để nhập."),
                    Err(e) => toast(ui, ps, &format!("Chưa ghi được tệp: {e}")),
                }
            }
        }
        DlgAction::CreateBackup => {
            let (a, b) = (p.get_pw1().to_string(), p.get_pw2().to_string());
            let e1 = if a.chars().count() < 8 {
                "Mật khẩu cần ít nhất 8 ký tự"
            } else {
                ""
            };
            let e2 = if e1.is_empty() && a != b {
                "Hai lần nhập mật khẩu chưa giống nhau"
            } else {
                ""
            };
            p.set_e_pw1(ss(e1));
            p.set_e_pw2(ss(e2));
            if !e1.is_empty() || !e2.is_empty() {
                ps.dlg = DlgAction::CreateBackup;
                return;
            }
            let name = format!("LichVanHoaViet-saoluu-{}.lvhv", iso(local_today()));
            let Some(path) = platform::pick(platform::hwnd_of(ui.window()), Dialog::Save(name)) else {
                ps.dlg = DlgAction::CreateBackup;
                return;
            };
            // Argon2id 64 MiB chạy ở luồng phụ; hộp giữ trạng thái "đang xử lý" (MA3N-25-06).
            p.set_dlg_busy(true);
            p.set_dlg_lines(vm(vec![ss("Đang mã hóa… (có thể mất vài giây)")]));
            ps.dlg = DlgAction::CreateBackup;
            bg(
                ui,
                ps,
                move || {
                    store
                        .export_backup(a, crate::update::VERSION.into(), "windows".into())
                        .map_err(|e| e.to_string())
                        .and_then(|bytes| std::fs::write(&path, bytes).map_err(|e| e.to_string()))
                },
                |ui, ps, r| {
                    close_dialog(ui, ps);
                    match r {
                        Ok(()) => {
                            let _ = std::fs::remove_file(ps.dir.join("ui-restored-from.txt"));
                            refresh(ui, ps);
                            toast(ui, ps, "Đã tạo tệp sao lưu. Hãy cất tệp ở nơi an toàn (ổ đám mây, USB) — tệp không tự gửi đi đâu.");
                        }
                        Err(e) => toast(ui, ps, &format!("Chưa tạo được tệp sao lưu: {e}")),
                    }
                },
            );
        }
        DlgAction::Restore => {
            if p.get_restore_mode() == 1 {
                let n = jv(&store.persons_json().unwrap_or_default())
                    .as_array()
                    .map_or(0, Vec::len);
                let date = ps
                    .backup
                    .as_ref()
                    .and_then(|b| local_date_of_rfc3339(&b.preview().exported_at))
                    .map(|d| d.dmy())
                    .unwrap_or_default();
                open_dialog(
                    ui,
                    ps,
                    "xac-nhan",
                    "Thay toàn bộ dữ liệu?",
                    vec![format!("Dữ liệu hiện có trên máy ({n} người) sẽ bị thay bằng dữ liệu trong tệp xuất ngày {date}. Không hoàn tác được.")],
                    "Thay toàn bộ",
                    "Hủy",
                    true,
                    DlgAction::ReplaceConfirm,
                );
            } else {
                apply_restore(ui, ps, BackupImportMode::Merge);
            }
        }
        DlgAction::ReplaceConfirm => apply_restore(ui, ps, BackupImportMode::Replace),
        DlgAction::Discard => {
            close_dialog(ui, ps);
            back(ui, ps);
        }
        DlgAction::Mark(dv, notice) => {
            close_dialog(ui, ps);
            do_mark(ui, ps, dv, notice);
        }
    }
}

fn apply_restore(ui: &AppWindow, ps: &mut PState, mode: BackupImportMode) {
    let (Some(store), Some(b)) = (ps.store.clone(), ps.backup.take()) else {
        close_dialog(ui, ps);
        return;
    };
    let p = People::get(ui);
    p.set_dlg_busy(true);
    p.set_dlg_lines(vm(vec![ss("Đang khôi phục… (có thể mất vài giây)")]));
    let pv = b.preview();
    bg(
        ui,
        ps,
        move || {
            store
                .apply_import(b, mode)
                .map(|_| (mode == BackupImportMode::Replace, pv.exported_at, pv.persons, pv.events))
                .map_err(|e| e.to_string())
        },
        |ui, ps, r| {
            close_dialog(ui, ps);
            People::get(ui).set_restore_pw(ss(""));
            match r {
                Ok((replace, exported_at, persons, events)) => {
                    if replace {
                        if let Some(d) = local_date_of_rfc3339(&exported_at) {
                            let _ = std::fs::write(ps.dir.join("ui-restored-from.txt"), d.dmy());
                        }
                    }
                    after_change(ui, ps);
                    toast(ui, ps, &format!("Đã khôi phục: {persons} người, {events} sự kiện."));
                }
                Err(_) => toast(ui, ps, "Chưa khôi phục được — dữ liệu trên máy giữ nguyên."),
            }
        },
    );
}

thread_local! {
    /// Trạng thái mô-đun cho phần việc chạy ở luồng phụ quay về luồng giao diện.
    static PS_TLS: RefCell<Option<Ps>> = const { RefCell::new(None) };
}

/// Chạy `work` ở luồng phụ, rồi `done` trên luồng giao diện (MA3N-25-06: Argon2id, giải mã tệp sao lưu, khôi phục,
/// giải mã ảnh không làm đứng cửa sổ). Trong kiểm thử (không có vòng sự kiện) chạy tuần tự.
fn bg<T: Send + 'static>(
    ui: &AppWindow,
    ps: &mut PState,
    work: impl FnOnce() -> T + Send + 'static,
    done: fn(&AppWindow, &mut PState, T),
) {
    #[cfg(test)]
    {
        let r = work();
        done(ui, ps, r);
    }
    #[cfg(not(test))]
    {
        let _ = ps;
        let w = ui.as_weak();
        std::thread::spawn(move || {
            let r = work();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = w.upgrade() {
                    let ps = PS_TLS.with(|t| t.borrow().clone());
                    if let Some(ps) = ps {
                        done(&ui, &mut ps.borrow_mut(), r);
                    }
                }
            });
        });
    }
}

/// Đọc + mã hóa lại một ảnh; kiểm 25 MB **trước** khi đọc tệp (MA3N-25-07).
fn load_photo(path: std::path::PathBuf) -> Result<photo::Encoded, String> {
    let len = std::fs::metadata(&path).map_err(|e| e.to_string())?.len();
    if len > photo::MAX_INPUT as u64 {
        return Err("Ảnh lớn hơn 25 MB — hãy chọn ảnh nhỏ hơn".into());
    }
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    photo::reencode(&bytes)
}

fn export_ics(ui: &AppWindow, ps: &mut PState) {
    let Some(store) = ps.store.clone() else { return };
    let today = local_today();
    let Some(ics) = store.dataset_json().ok().and_then(|ds| {
        lvhv_ffi::ics_export(ds, today.solar(), platform::now_unix(), None).ok()
    }) else {
        return;
    };
    // Xem trước 2 mục đầu: DTSTART + SUMMARY của từng VEVENT.
    let mut prev = Vec::new();
    let mut date = String::new();
    for line in ics.content.lines() {
        if let Some(d) = line.split(':').nth(1).filter(|_| line.starts_with("DTSTART")) {
            date = format!(
                "{}/{}/{}",
                d.get(6..8).unwrap_or(""),
                d.get(4..6).unwrap_or(""),
                d.get(..4).unwrap_or("")
            );
        }
        if let Some(s) = line.strip_prefix("SUMMARY:") {
            prev.push(format!("• {date} — {}", s.replace("\\,", ",").replace("\\;", ";")));
        }
    }
    let end = today.add(3652).map(|d| d.add(-1).unwrap_or(d)).unwrap_or(today);
    let mut lines = vec![
        format!(
            "Tệp này chứa tên, ngày giỗ, sinh nhật và sự kiện của {} người. Bất kỳ ai có tệp đều đọc được. Không gồm thông tin báo tử.",
            ics.persons
        ),
        format!(
            "Gồm {} ngày trong 10 năm tới (từ {} đến {}), mỗi ngày có lời nhắc như đã đặt.",
            ics.events,
            today.dmy(),
            end.dmy()
        ),
        "Xem trước:".into(),
    ];
    lines.extend(prev.iter().take(2).cloned());
    if prev.len() > 2 {
        lines.push(format!("… và {} ngày khác", prev.len() - 2));
    }
    lines.push("Mở tệp bằng Outlook hoặc ứng dụng Lịch của máy để nhập.".into());
    open_dialog(
        ui,
        ps,
        "ics",
        "Xuất tệp lịch",
        lines,
        "Lưu tệp .ics",
        "Hủy",
        false,
        DlgAction::Ics(ics.file_name, ics.content),
    );
}

fn create_backup(ui: &AppWindow, ps: &mut PState) {
    let Some(store) = ps.store.clone() else { return };
    let ds = jv(&store.dataset_json().unwrap_or_default());
    let np = ds["persons"].as_array().map_or(0, Vec::len);
    let ne = ds["events"].as_array().map_or(0, Vec::len);
    let nph = ds["photos"].as_array().map_or(0, Vec::len);
    let mut lines = vec![format!(
        "Tệp sẽ chứa {np} người, {ne} sự kiện, {nph} ảnh — đã mã hóa bằng mật khẩu."
    )];
    if ds["persons"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|p| p["death"]["notice"].is_object())
    {
        lines.push("Tệp gồm cả thông tin báo tử (người báo tin, nơi mất, lý do, ảnh giấy báo tử) — chỉ ai biết mật khẩu mới đọc được.".into());
    }
    open_dialog(
        ui,
        ps,
        "tao-sao-luu",
        "Tạo tệp sao lưu",
        lines,
        "Tạo và lưu tệp",
        "Hủy",
        false,
        DlgAction::CreateBackup,
    );
}

fn choose_backup(ui: &AppWindow, ps: &mut PState) {
    let p = People::get(ui);
    if let Some(path) = platform::pick(platform::hwnd_of(ui.window()), Dialog::OpenBackup) {
        p.set_restore_file(ss(format!(
            "Đã chọn: {}",
            path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
        )));
        p.set_restore_error(ss(""));
        ps.backup_file = Some(path);
    }
}

fn open_backup_file(ui: &AppWindow, ps: &mut PState) {
    let p = People::get(ui);
    let Some(path) = ps.backup_file.clone() else {
        p.set_restore_error(ss("Hãy chọn tệp sao lưu trước."));
        return;
    };
    let generic = "Sai mật khẩu hoặc tệp bị hỏng";
    if std::fs::metadata(&path).map(|m| m.len() > BACKUP_MAX).unwrap_or(true) {
        p.set_restore_error(ss(generic));
        return;
    }
    p.set_restore_error(ss("Đang mở tệp… (có thể mất vài giây)"));
    let pw = p.get_restore_pw().to_string();
    bg(
        ui,
        ps,
        move || {
            let bytes = std::fs::read(&path).map_err(|e| format!("Không đọc được tệp: {e}"))?;
            lvhv_ffi::open_backup(bytes, pw).map_err(|e| e.to_string())
        },
        |ui, ps, r| {
            let p = People::get(ui);
            match r {
                Err(e) => p.set_restore_error(ss(e)),
                Ok(b) => {
                    p.set_restore_error(ss(""));
                    show_restore_preview(ui, ps, b);
                }
            }
        },
    );
}

/// Hộp xem trước khôi phục (03 §2.4, 08 §3.8 C) cho một tệp đã giải mã và kiểm tra.
pub fn show_restore_preview(ui: &AppWindow, ps: &mut PState, b: std::sync::Arc<lvhv_ffi::VerifiedBackup>) {
    let p = People::get(ui);
    let pv = b.preview();
    let date = local_date_of_rfc3339(&pv.exported_at)
        .map(|d| d.dmy())
        .unwrap_or_default();
    let platform = match pv.platform.as_str() {
        "pwa" => "ứng dụng web",
        "android" => "Android",
        "ios" => "iPhone/iPad",
        "macos" => "máy Mac",
        "windows" => "Windows",
        o => o,
    }
    .to_string();
    ps.backup = Some(b);
    p.set_restore_mode(0);
    open_dialog(
        ui,
        ps,
        "khoi-phuc",
        "Khôi phục dữ liệu",
        vec![format!(
            "Tệp xuất ngày {date} từ {platform}: {} người, {} sự kiện, {} ảnh.",
            pv.persons, pv.events, pv.photos
        )],
        "Khôi phục",
        "Hủy",
        false,
        DlgAction::Restore,
    );
}

fn settings_changed(ui: &AppWindow, ps: &mut PState) {
    let Some(store) = ps.store.clone() else { return };
    let p = People::get(ui);
    let mut s = jv(&store.settings_json().unwrap_or_default());
    s["reminders_enabled"] = json!(p.get_rem_enabled());
    match rule_from(
        ps,
        "def",
        true,
        &p.get_def_custom(),
        &p.get_def_hh(),
        &p.get_def_mm(),
        false,
        &Value::Null,
    ) {
        Ok((r, _)) => {
            p.set_def_error(ss(""));
            s["default_offsets_days"] = r["offsets_days"].clone();
            s["default_time_local"] = r["time_local"].clone();
        }
        Err(e) => {
            p.set_def_error(ss(e));
            return;
        }
    }
    let r = store
        .put_settings_json(s.to_string())
        .and_then(|_| store.set_lock_screen_names(p.get_show_names()));
    if let Err(e) = r {
        toast(ui, ps, &format!("Chưa lưu được cài đặt: {e}"));
        render_settings(ui, ps);
        return;
    }
    reschedule(Some(ui), ps);
}

fn toggle_off(ui: &AppWindow, ps: &mut PState, k: &str, i: i32) {
    let key: &'static str = match k {
        "r" => "r",
        "r2" => "r2",
        "er" => "er",
        _ => "def",
    };
    let mut v = ps.offs.get(key).cloned().unwrap_or(vec![false; 6]);
    if let Some(b) = v.get_mut(i as usize) {
        *b = !*b;
    }
    let offs: Vec<i64> = v.iter().zip(QUICK).filter(|(b, _)| **b).map(|(_, q)| q).collect();
    let p = People::get(ui);
    let custom = match key {
        "r" => p.get_r_custom(),
        "r2" => p.get_r2_custom(),
        "er" => p.get_er_custom(),
        _ => p.get_def_custom(),
    };
    set_offs(ui, ps, key, &offs);
    match key {
        "r" => p.set_r_custom(custom),
        "r2" => p.set_r2_custom(custom),
        "er" => p.set_er_custom(custom),
        _ => p.set_def_custom(custom),
    }
    if key == "def" {
        settings_changed(ui, ps);
    } else {
        form_changed(ui, ps);
    }
}

// ------------------------------------------------------------------------------------------------
// Nối callback
// ------------------------------------------------------------------------------------------------

pub fn wire(ui: &AppWindow, ps: Ps) {
    PS_TLS.with(|t| *t.borrow_mut() = Some(ps.clone()));
    let p = People::get(ui);
    let w = ui.as_weak();
    macro_rules! cb0 {
        ($body:expr) => {{
            let w = w.clone();
            let ps = ps.clone();
            move || {
                if let Some(ui) = w.upgrade() {
                    #[allow(clippy::redundant_closure_call)]
                    ($body)(&ui, &mut ps.borrow_mut());
                }
            }
        }};
    }
    macro_rules! cb1 {
        ($t:ty, $body:expr) => {{
            let w = w.clone();
            let ps = ps.clone();
            move |a: $t| {
                if let Some(ui) = w.upgrade() {
                    #[allow(clippy::redundant_closure_call)]
                    ($body)(&ui, &mut ps.borrow_mut(), a);
                }
            }
        }};
    }
    p.on_go(cb1!(SharedString, |ui: &AppWindow, ps: &mut PState, s: SharedString| go(ui, ps, &s)));
    p.on_back(cb0!(|ui: &AppWindow, ps: &mut PState| back(ui, ps)));
    p.on_open_person(cb1!(SharedString, |ui: &AppWindow, ps: &mut PState, s: SharedString| open_person(ui, ps, &s)));
    p.on_open_memorial(cb1!(SharedString, |ui: &AppWindow, ps: &mut PState, s: SharedString| open_memorial(ui, ps, &s)));
    p.on_add_dead(cb0!(|ui: &AppWindow, ps: &mut PState| {
        if ensure_store(ui, ps) {
            open_person_form(ui, ps, true, None)
        }
    }));
    p.on_add_living(cb0!(|ui: &AppWindow, ps: &mut PState| {
        if ensure_store(ui, ps) {
            open_person_form(ui, ps, false, None)
        }
    }));
    p.on_edit_person(cb0!(|ui: &AppWindow, ps: &mut PState| {
        if let Some(v) = ps.person.clone().and_then(|id| person(ps, &id)) {
            open_person_form(ui, ps, v["kind"] == "deceased", Some(v));
        }
    }));
    p.on_delete_person(cb0!(|ui: &AppWindow, ps: &mut PState| {
        if let Some(v) = ps.person.clone().and_then(|id| person(ps, &id)) {
            let name = display_name(&v);
            open_dialog(
                ui,
                ps,
                "xac-nhan",
                &format!("Xóa hồ sơ của {name}?"),
                vec!["Ngày giỗ, lời nhắc và ảnh của người này sẽ bị xóa khỏi máy. Sự kiện gắn với người này vẫn còn nếu có người khác cùng tham gia. Không hoàn tác được.".into()],
                "Xóa hồ sơ",
                "Giữ lại",
                true,
                DlgAction::DeletePerson(st(&v["id"])),
            );
        }
    }));
    p.on_mark_deceased(cb0!(|ui: &AppWindow, ps: &mut PState| open_mark(ui, ps)));
    p.on_add_event(cb0!(|ui: &AppWindow, ps: &mut PState| open_event_form(ui, ps, None)));
    p.on_edit_event(cb1!(SharedString, |ui: &AppWindow, ps: &mut PState, id: SharedString| {
        let e = ps
            .s()
            .and_then(|s| s.events_json().ok())
            .and_then(|j| jv(&j).as_array()?.iter().find(|e| e["id"] == id.as_str()).cloned());
        if e.is_some() {
            open_event_form(ui, ps, e);
        }
    }));
    p.on_delete_event(cb1!(SharedString, |ui: &AppWindow, ps: &mut PState, id: SharedString| {
        let e = ps
            .s()
            .and_then(|s| s.events_json().ok())
            .and_then(|j| jv(&j).as_array()?.iter().find(|e| e["id"] == id.as_str()).cloned());
        if let Some(e) = e {
            let t = if st(&e["tieu_de"]).is_empty() {
                type_name(ps, &e["type_id"])
            } else {
                st(&e["tieu_de"])
            };
            open_dialog(
                ui,
                ps,
                "xac-nhan",
                &format!("Xóa sự kiện \"{t}\"?"),
                vec!["Không hoàn tác được.".into()],
                "Xóa sự kiện",
                "Giữ lại",
                true,
                DlgAction::DeleteEvent(id.to_string()),
            );
        }
    }));
    p.on_form_changed(cb0!(|ui: &AppWindow, ps: &mut PState| form_changed(ui, ps)));
    {
        let w = w.clone();
        let ps = ps.clone();
        p.on_toggle_off(move |k: SharedString, i: i32| {
            if let Some(ui) = w.upgrade() {
                toggle_off(&ui, &mut ps.borrow_mut(), &k, i);
            }
        });
    }
    p.on_pick_photo(cb0!(|ui: &AppWindow, ps: &mut PState| {
        let Some(path) = platform::pick(platform::hwnd_of(ui.window()), Dialog::OpenImage) else { return };
        People::get(ui).set_e_photo(ss("Đang đọc ảnh…"));
        bg(ui, ps, move || load_photo(path), |ui, ps, r| {
            let p = People::get(ui);
            match r {
                Ok(e) => {
                    p.set_f_photo(photo::to_slint(&e.bytes).unwrap_or_default());
                    p.set_f_has_photo(true);
                    p.set_e_photo(ss(""));
                    ps.photo = PhotoSel::New(e);
                    ps.dirty = true;
                }
                Err(e) => p.set_e_photo(ss(e)),
            }
        });
    }));
    p.on_drop_photo(cb0!(|ui: &AppWindow, ps: &mut PState| {
        let p = People::get(ui);
        ps.photo = PhotoSel::Keep(None);
        p.set_f_has_photo(false);
        p.set_f_photo(Image::default());
        ps.dirty = true;
    }));
    p.on_add_notice_photo(cb0!(|ui: &AppWindow, ps: &mut PState| {
        if ps.certs.len() >= 4 {
            return;
        }
        let Some(path) = platform::pick(platform::hwnd_of(ui.window()), Dialog::OpenImage) else { return };
        People::get(ui).set_e_notice(ss("Đang đọc ảnh…"));
        bg(ui, ps, move || load_photo(path), |ui, ps, r| {
            let p = People::get(ui);
            match r {
                Ok(e) if ps.certs.len() < 4 => {
                    let img = photo::to_slint(&e.bytes).unwrap_or_default();
                    ps.certs.push(CertPhoto { id: None, enc: Some(e), img });
                    p.set_n_photos(vm(ps.certs.iter().map(|c| c.img.clone()).collect()));
                    p.set_e_notice(ss(""));
                    ps.dirty = true;
                }
                Ok(_) => p.set_e_notice(ss("")),
                Err(e) => p.set_e_notice(ss(e)),
            }
        });
    }));
    p.on_drop_notice_photo(cb1!(i32, |ui: &AppWindow, ps: &mut PState, i: i32| {
        if (i as usize) < ps.certs.len() {
            ps.certs.remove(i as usize);
        }
        People::get(ui).set_n_photos(vm(ps.certs.iter().map(|c| c.img.clone()).collect()));
        ps.dirty = true;
    }));
    p.on_save_form(cb0!(|ui: &AppWindow, ps: &mut PState| save_form(ui, ps)));
    p.on_cancel_form(cb0!(|ui: &AppWindow, ps: &mut PState| {
        if ps.dirty {
            open_dialog(
                ui,
                ps,
                "xac-nhan",
                "Bỏ các thay đổi chưa lưu?",
                vec![],
                "Bỏ thay đổi",
                "Tiếp tục sửa",
                true,
                DlgAction::Discard,
            );
        } else {
            back(ui, ps);
        }
    }));
    p.on_period_kind(cb1!(i32, |ui: &AppWindow, ps: &mut PState, k: i32| {
        ps.pkind = k.clamp(0, 3) as usize;
        render_period(ui, ps);
    }));
    p.on_period_filter(cb1!(i32, |ui: &AppWindow, ps: &mut PState, k: i32| {
        ps.pfilter = k.clamp(0, 2) as usize;
        render_period(ui, ps);
    }));
    p.on_period_step(cb1!(i32, |ui: &AppWindow, ps: &mut PState, n: i32| {
        let r = if n < 0 {
            lvhv_ffi::period_prev(PKIND[ps.pkind].into(), ps.panchor.solar())
        } else {
            lvhv_ffi::period_next(PKIND[ps.pkind].into(), ps.panchor.solar())
        };
        if let Ok(r) = r {
            ps.panchor = D::from(r.from);
            render_period(ui, ps);
        }
    }));
    p.on_period_today(cb0!(|ui: &AppWindow, ps: &mut PState| {
        ps.panchor = local_today();
        render_period(ui, ps);
    }));
    p.on_settings_changed(cb0!(|ui: &AppWindow, ps: &mut PState| settings_changed(ui, ps)));
    p.on_export_ics(cb0!(|ui: &AppWindow, ps: &mut PState| export_ics(ui, ps)));
    p.on_create_backup(cb0!(|ui: &AppWindow, ps: &mut PState| create_backup(ui, ps)));
    p.on_choose_backup(cb0!(|ui: &AppWindow, ps: &mut PState| choose_backup(ui, ps)));
    p.on_open_backup(cb0!(|ui: &AppWindow, ps: &mut PState| open_backup_file(ui, ps)));
    p.on_nag_changed(cb1!(i32, |ui: &AppWindow, ps: &mut PState, i: i32| {
        let Some(store) = ps.store.clone() else { return };
        let mut s = jv(&store.settings_json().unwrap_or_default());
        s["backup_nag_days"] = json!([30, 60, 90, 0][i.clamp(0, 3) as usize]);
        if let Err(e) = store.put_settings_json(s.to_string()) {
            toast(ui, ps, &format!("Chưa lưu được cài đặt: {e}"));
        }
        refresh(ui, ps);
    }));
    p.on_dismiss_sheet_card(cb0!(|ui: &AppWindow, ps: &mut PState| {
        if let Some((id, _)) = today_cards(ps).first() {
            ps.hidden_cards.insert(id.clone());
        }
        render_upcoming(ui, ps);
    }));
    p.on_goto_error(cb1!(SharedString, |ui: &AppWindow, _ps: &mut PState, k: SharedString| {
        People::get(ui).set_scroll_frac(error_scroll(&k));
    }));
    p.on_dlg_confirm(cb0!(|ui: &AppWindow, ps: &mut PState| dlg_confirm(ui, ps)));
    p.on_dlg_dismiss(cb0!(|ui: &AppWindow, ps: &mut PState| {
        if !People::get(ui).get_dlg_busy() {
            if ps.backup.take().is_some() {
                People::get(ui).set_restore_pw(ss(""));
            }
            close_dialog(ui, ps);
        }
    }));
    p.on_smoke_toggle(cb0!(|ui: &AppWindow, ps: &mut PState| {
        if People::get(ui).get_m_smoke() {
            ps.smoke_gen.set(ps.smoke_gen.get() + 1);
            People::get(ui).set_m_smoke(false);
        } else {
            start_smoke(ui, ps);
        }
    }));
    p.on_dismiss_card(cb1!(i32, |ui: &AppWindow, ps: &mut PState, i: i32| {
        if let Some((id, _)) = today_cards(ps).get(i as usize) {
            ps.hidden_cards.insert(id.clone());
        }
        render_upcoming(ui, ps);
    }));
}
