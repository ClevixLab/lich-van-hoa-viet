//! Lịch và Văn hóa Việt — ứng dụng Windows (Slint), phần thư viện dùng chung cho hai tệp chạy:
//! - `LichViet.exe` (`src/main.rs`) — bản phát hành, đóng gói vào `.msi`/`.zip`; chỉ nhận URI `lichviet:` của thông
//!   báo, **không** có chế độ kiểm thử nào (MA3C-25-01).
//! - `lvhv-ci-check.exe` (`src/bin/lvhv-ci-check.rs`) — chỉ dùng trong CI, không đóng gói: chụp màn hình, đo bộ nhớ,
//!   tự kiểm thông báo, R14 (`ci.rs`).
//!
//! Gọi trực tiếp crate Rust `lvhv-ffi` (không qua FFI C). Không dùng mạng.
// Mã viết tay không dùng `unsafe`; mã Slint sinh ra (mô-đun `ui`) cần `unsafe` nội bộ nên được miễn.
#![deny(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use serde::Deserialize;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

pub mod ci;
mod people;
mod photo;
mod platform;
mod shots;
mod toasts;
mod trays;
#[cfg(test)]
mod ui_tests;
#[cfg(test)]
mod ui_tests_people;
mod attach;
mod update;
mod upgrade;

#[allow(unsafe_code, dead_code, clippy::all)]
mod ui {
    slint::include_modules!();
}
use ui::*;

const MIN_YEAR: i32 = lvhv_ffi::MIN_YEAR;
const MAX_YEAR: i32 = lvhv_ffi::MAX_YEAR;
const PAGE: usize = 15;

const WEEKDAY: [&str; 7] = [
    "Chủ nhật",
    "Thứ Hai",
    "Thứ Ba",
    "Thứ Tư",
    "Thứ Năm",
    "Thứ Sáu",
    "Thứ Bảy",
];
const LUNAR_MONTH: [&str; 13] = [
    "",
    "Giêng",
    "Hai",
    "Ba",
    "Tư",
    "Năm",
    "Sáu",
    "Bảy",
    "Tám",
    "Chín",
    "Mười",
    "Mười Một",
    "Chạp",
];

// ---------- JSON của lõi ----------

#[derive(Deserialize)]
struct JSolar {
    weekday: i32,
}
#[derive(Deserialize)]
struct JLunar {
    m: i32,
    d: i32,
    leap: bool,
    month_len: i32,
}
#[derive(Deserialize)]
struct JCanChi {
    year: String,
    month: String,
    day: String,
    hour_ty: String,
}
#[derive(Deserialize)]
struct JThan {
    name: String,
    hoang_dao: bool,
}
#[derive(Deserialize)]
struct JGio {
    chi: String,
    from: i32,
    to: i32,
}
#[derive(Deserialize)]
struct JNgocHap {
    sao_tot: Vec<String>,
    sao_xau: Vec<String>,
    nen: Vec<String>,
    ky: Vec<String>,
    xung_tuoi: Vec<String>,
    score: i32,
}
#[derive(Deserialize)]
struct JDay {
    solar: JSolar,
    lunar: JLunar,
    can_chi: JCanChi,
    tiet_khi: String,
    nap_am: String,
    truc: String,
    tu: String,
    than: JThan,
    gio_hoang_dao: Vec<JGio>,
    ngoc_hap: JNgocHap,
}
#[derive(Deserialize)]
struct JAct {
    id: String,
    name: String,
}
#[derive(Deserialize)]
struct JGoodSolar {
    y: i32,
    m: i32,
    d: i32,
}
#[derive(Deserialize)]
struct JGood {
    solar: JGoodSolar,
    score: i32,
    reasons: Vec<String>,
}
#[derive(Deserialize)]
struct JCadao {
    #[serde(rename = "type")]
    kind: String,
    lines: Vec<String>,
}
#[derive(Deserialize)]
struct JBlock {
    kind: String,
    text: String,
}

// ---------- Ngày ----------

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct D {
    y: i32,
    m: i32,
    d: i32,
}

impl D {
    fn solar(self) -> lvhv_ffi::SolarDay {
        lvhv_ffi::SolarDay {
            y: self.y,
            m: self.m,
            d: self.d,
        }
    }
    fn from(s: lvhv_ffi::SolarDay) -> D {
        D {
            y: s.y,
            m: s.m,
            d: s.d,
        }
    }
    fn add(self, n: i32) -> Option<D> {
        lvhv_ffi::add_days(self.solar(), n).ok().map(D::from)
    }
    fn weekday(self) -> usize {
        lvhv_ffi::weekday(self.y, self.m, self.d).unwrap_or(0) as usize
    }
    fn dmy(self) -> String {
        format!("{:02}/{:02}/{}", self.d, self.m, self.y)
    }
    fn parse(s: &str) -> Option<D> {
        let p: Vec<&str> = s.trim().split(['/', '-', '.']).collect();
        if p.len() != 3 {
            return None;
        }
        let d = D {
            d: p[0].trim().parse().ok()?,
            m: p[1].trim().parse().ok()?,
            y: p[2].trim().parse().ok()?,
        };
        lvhv_ffi::weekday(d.y, d.m, d.d).ok().map(|_| d)
    }
    fn in_range(self) -> bool {
        (MIN_YEAR..=MAX_YEAR).contains(&self.y)
    }
    fn days_in_month(self) -> i32 {
        lvhv_ffi::days_in_month(self.y, self.m).unwrap_or(30)
    }
    fn jdn(self) -> i64 {
        // Fliegel–Van Flandern (như lõi) — chỉ dùng để so khoảng cách.
        let (y, m, d) = (self.y as i64, self.m as i64, self.d as i64);
        let a = (14 - m) / 12;
        let yy = y + 4800 - a;
        let mm = m + 12 * a - 3;
        d + (153 * mm + 2) / 5 + 365 * yy + yy / 4 - yy / 100 + yy / 400 - 32045
    }
}

/// Ngày hôm nay theo giờ địa phương của máy. Không có thư viện múi giờ: dùng UTC+7 (giờ Việt Nam,
/// đúng quy ước lịch của lõi) — người dùng ở múi giờ khác có thể thấy "hôm nay" lệch gần nửa đêm.
fn today() -> D {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
        + 7 * 3600;
    let days = secs.div_euclid(86400);
    // 1970-01-01 = JDN 2440588
    let j = days + 2440588;
    let a = j + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - (b * 146097) / 4;
    let dd = (4 * c + 3) / 1461;
    let e = c - (1461 * dd) / 4;
    let m = (5 * e + 2) / 153;
    let day = e - (153 * m + 2) / 5 + 1;
    let month = m + 3 - 12 * (m / 10);
    let year = b * 100 + dd - 4800 + m / 10;
    let t = D {
        y: year as i32,
        m: month as i32,
        d: day as i32,
    };
    clamp(t)
}

fn clamp(d: D) -> D {
    if d.y < MIN_YEAR {
        D {
            y: MIN_YEAR,
            m: 1,
            d: 1,
        }
    } else if d.y > MAX_YEAR {
        D {
            y: MAX_YEAR,
            m: 12,
            d: 31,
        }
    } else {
        d
    }
}

fn sv(s: impl Into<SharedString>) -> SharedString {
    s.into()
}

fn strs(v: &[String]) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(
        v.iter().map(|s| sv(s.as_str())).collect::<Vec<_>>(),
    ))
}

/// Điểm có dấu (+8, 0, −3) như bản web.
fn signed(n: i32) -> String {
    match n {
        n if n > 0 => format!("+{n}"),
        n if n < 0 => format!("−{}", -n),
        _ => "0".to_string(),
    }
}

/// Chia các sao thành từng hàng chip (Slint 1.8 chưa có FlowLayout). Mỗi hàng tối đa ~44 ký tự
/// (vừa cột tờ lịch 516 px ở cỡ chữ lớn nhất) và tối đa 4 chip.
fn chip_rows(items: &[String]) -> ModelRc<ChipRow> {
    const MAX_CHARS: usize = 34;
    let mut rows: Vec<Vec<SharedString>> = Vec::new();
    let mut used = 0usize;
    for it in items {
        let n = it.chars().count() + 4;
        match rows.last_mut() {
            Some(r) if used + n <= MAX_CHARS && r.len() < 4 => {
                r.push(sv(it.as_str()));
                used += n;
            }
            _ => {
                rows.push(vec![sv(it.as_str())]);
                used = n;
            }
        }
    }
    ModelRc::new(VecModel::from(
        rows.into_iter()
            .map(|r| ChipRow {
                items: ModelRc::new(VecModel::from(r)),
            })
            .collect::<Vec<_>>(),
    ))
}

fn upper_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

// ---------- Trạng thái ----------

struct State {
    date: D,
    today: D,
    view: (i32, i32),
    acts: Vec<JAct>,
    good_all: Vec<JGood>,
    good_shown: usize,
    solar_target: Option<D>,
    lunar_target: Option<D>,
    /// Mô-đun Người thân (kho dữ liệu, trang đang mở, biểu mẫu).
    people: people::Ps,
}

fn render_sheet(ui: &AppWindow, st: &State) {
    let g = ui.global::<Data>();
    let d = st.date;
    let Ok(json) = lvhv_ffi::day_info_json(d.y, d.m, d.d) else {
        return;
    };
    let Ok(info) = serde_json::from_str::<JDay>(&json) else {
        return;
    };
    let wd = info.solar.weekday as usize;
    let r = lvhv_ffi::day_rating(info.ngoc_hap.score);
    let cad = lvhv_ffi::cadao_for(d.y, d.m, d.d)
        .ok()
        .and_then(|j| serde_json::from_str::<JCadao>(&j).ok());
    let sheet = Sheet {
        month_title: sv(format!("Tháng {}", d.m)),
        year: sv(d.y.to_string()),
        weekday: sv(WEEKDAY[wd % 7]),
        day: sv(d.d.to_string()),
        sunday: wd == 0,
        is_today: d == st.today,
        lunar_day: sv(info.lunar.d.to_string()),
        lunar_key: info.lunar.d == 1 || info.lunar.d == 15,
        lunar_month: sv(format!(
            "tháng {}{}",
            LUNAR_MONTH[info.lunar.m.clamp(0, 12) as usize],
            if info.lunar.leap { " (nhuận)" } else { "" }
        )),
        month_len: sv(if info.lunar.month_len == 30 {
            "tháng đủ"
        } else {
            "tháng thiếu"
        }),
        cc_year: sv(info.can_chi.year),
        cc_month: sv(info.can_chi.month),
        cc_day: sv(info.can_chi.day),
        tiet: sv(info.tiet_khi),
        than_kind: sv(if info.than.hoang_dao {
            "Hoàng đạo"
        } else {
            "Hắc đạo"
        }),
        than_name: sv(info.than.name),
        hoang_dao: info.than.hoang_dao,
        truc: sv(info.truc),
        rating: sv(r.label),
        rating_level: r.level as i32,
        nen: strs(&info.ngoc_hap.nen),
        ky: strs(&info.ngoc_hap.ky),
        sao_tot: chip_rows(&info.ngoc_hap.sao_tot),
        sao_xau: chip_rows(&info.ngoc_hap.sao_xau),
        score: sv(signed(info.ngoc_hap.score)),
        tu: sv(format!("Sao {}", info.tu)),
        nap_am: sv(info.nap_am),
        hour_ty: sv(info.can_chi.hour_ty),
        xung: sv(if info.ngoc_hap.xung_tuoi.is_empty() {
            "Không có".to_string()
        } else {
            info.ngoc_hap.xung_tuoi.join(", ")
        }),
        has_cadao: cad.is_some(),
        cadao_type: sv(cad
            .as_ref()
            .map(|c| upper_first(&c.kind))
            .unwrap_or_default()),
        cadao: strs(&cad.map(|c| c.lines).unwrap_or_default()),
    };
    g.set_sheet(sheet);
    let gio: Vec<Gio> = info
        .gio_hoang_dao
        .iter()
        .map(|h| Gio {
            chi: sv(h.chi.as_str()),
            range: sv(format!("{}–{} giờ", h.from, h.to)),
        })
        .collect();
    g.set_gio(ModelRc::new(VecModel::from(gio)));
    g.set_can_prev(
        d > D {
            y: MIN_YEAR,
            m: 1,
            d: 1,
        },
    );
    g.set_can_next(
        d < D {
            y: MAX_YEAR,
            m: 12,
            d: 31,
        },
    );
    g.set_is_today(d == st.today);
}

fn render_month(ui: &AppWindow, st: &State) {
    let g = ui.global::<Data>();
    let (y, m) = st.view;
    let first = D { y, m, d: 1 };
    let lead = (first.weekday() + 6) % 7;
    let lunar = lvhv_ffi::month_lunar_days(y, m).unwrap_or_default();
    let mut cells = Vec::with_capacity(42);
    for _ in 0..lead {
        cells.push(Cell {
            blank: true,
            ..Default::default()
        });
    }
    for (i, l) in lunar.iter().enumerate() {
        let day = i as i32 + 1;
        let date = D { y, m, d: day };
        let col = (lead + i) % 7;
        let key = l.d == 1 || l.d == 15;
        let lunar_txt = if l.d == 1 {
            format!("{}/{}{}", l.d, l.m, if l.leap { "n" } else { "" })
        } else {
            l.d.to_string()
        };
        let is_today = date == st.today;
        cells.push(Cell {
            day,
            lunar: sv(lunar_txt),
            key,
            sunday: col == 6,
            today: is_today,
            selected: date == st.date,
            blank: false,
            label: sv(format!(
                "{} {} tháng {}, âm lịch {} tháng {}{}{}",
                WEEKDAY[(col + 1) % 7],
                day,
                m,
                l.d,
                l.m,
                if l.leap { " nhuận" } else { "" },
                if is_today { ", hôm nay" } else { "" }
            )),
        });
    }
    g.set_cells(ModelRc::new(VecModel::from(cells)));
    g.set_view_month(m);
    g.set_view_year(sv(y.to_string()));
    g.set_view_can_prev(!(y == MIN_YEAR && m == 1));
    g.set_view_can_next(!(y == MAX_YEAR && m == 12));
}

fn set_date(ui: &AppWindow, st: &mut State, d: D) {
    st.date = clamp(d);
    st.view = (st.date.y, st.date.m);
    render_sheet(ui, st);
    render_month(ui, st);
}

fn recompute_good(ui: &AppWindow, st: &mut State) {
    let g = ui.global::<Data>();
    let t = st.today;
    let birth_txt = g.get_birth().to_string();
    let birth: Option<i32> = birth_txt
        .trim()
        .parse()
        .ok()
        .filter(|y| (1900..=t.y).contains(y));
    g.set_birth_cc(sv(birth
        .map(|y| format!("Tuổi {}", lvhv_ffi::year_can_chi(y)))
        .unwrap_or_default()));
    st.good_all.clear();
    st.good_shown = 0;
    g.set_hint(sv(""));
    let act = g.get_act();
    if act < 0 || act as usize >= st.acts.len() {
        g.set_summary(sv("Chọn một việc ở trên để xem những ngày phù hợp."));
        g.set_good(ModelRc::new(VecModel::from(Vec::<GoodRow>::new())));
        g.set_more(0);
        return;
    }
    let a = &st.acts[act as usize];
    let range = match g.get_range() {
        0 => t.add(((7 - t.weekday()) % 7) as i32).map(|e| (t, e)),
        1 => Some((
            t,
            D {
                d: t.days_in_month(),
                ..t
            },
        )),
        2 => t.add(89).map(|e| (t, e)),
        _ => match (D::parse(&g.get_from_text()), D::parse(&g.get_to_text())) {
            (Some(f), Some(e)) => Some((f, e)),
            _ => None,
        },
    };
    let msg = match range {
        None => "Hãy nhập đủ ngày bắt đầu và ngày kết thúc (dạng ngày/tháng/năm).".to_string(),
        Some((f, e)) if !f.in_range() || !e.in_range() => {
            format!("Lịch hỗ trợ từ năm {MIN_YEAR} đến năm {MAX_YEAR}.")
        }
        Some((f, e)) if e.jdn() < f.jdn() => "Ngày kết thúc cần sau ngày bắt đầu.".to_string(),
        Some((f, e)) if e.jdn() - f.jdn() > 366 => {
            "Khoảng tìm tối đa là một năm. Hãy thu hẹp lại.".to_string()
        }
        Some((f, e)) => {
            match lvhv_ffi::find_good_days_json(a.id.clone(), f.solar(), e.solar(), birth) {
                Err(err) => err.to_string(),
                Ok(j) => {
                    st.good_all = serde_json::from_str(&j).unwrap_or_default();
                    let n = st.good_all.len();
                    if n == 0 {
                        g.set_hint(sv(
                            "Hãy thử mở rộng khoảng thời gian, chẳng hạn 3 tháng tới.",
                        ));
                    }
                    format!(
                        "{}, từ {} đến {}{}: {}.",
                        a.name,
                        f.dmy(),
                        e.dmy(),
                        birth
                            .map(|y| format!(", tuổi {}", lvhv_ffi::year_can_chi(y)))
                            .unwrap_or_default(),
                        if n > 0 {
                            format!("{n} ngày phù hợp")
                        } else {
                            "chưa có ngày phù hợp".to_string()
                        }
                    )
                }
            }
        }
    };
    g.set_summary(sv(msg));
    st.good_shown = PAGE.min(st.good_all.len());
    render_good(ui, st);
}

fn render_good(ui: &AppWindow, st: &State) {
    let g = ui.global::<Data>();
    let rows: Vec<GoodRow> = st.good_all[..st.good_shown]
        .iter()
        .map(|x| {
            let s = D {
                y: x.solar.y,
                m: x.solar.m,
                d: x.solar.d,
            };
            let wd = s.weekday();
            let l = lvhv_ffi::solar_to_lunar(s.y, s.m, s.d).ok();
            let info = lvhv_ffi::day_info_json(s.y, s.m, s.d)
                .ok()
                .and_then(|j| serde_json::from_str::<JDay>(&j).ok());
            let (ld, lm, leap) = l.map(|l| (l.d, l.m, l.leap)).unwrap_or((0, 0, false));
            let first = info.as_ref().and_then(|i| i.gio_hoang_dao.first());
            GoodRow {
                y: s.y,
                m: s.m,
                d: s.d,
                day: sv(s.d.to_string()),
                mon: sv(format!("/{}", s.m)),
                l1: sv(format!("{}, {}", WEEKDAY[wd], s.dmy())),
                l2: sv(format!(
                    "Âm lịch {}/{}{}, ngày {}",
                    ld,
                    lm,
                    if leap { " nhuận" } else { "" },
                    info.as_ref().map(|i| i.can_chi.day.as_str()).unwrap_or("")
                )),
                l3: sv(first
                    .map(|f| format!("Giờ tốt đầu tiên: {} ({}–{} giờ)", f.chi, f.from, f.to))
                    .unwrap_or_default()),
                reasons: sv(x.reasons.join("; ")),
                score: sv(x.score.to_string()),
                sunday: wd == 0,
            }
        })
        .collect();
    g.set_good(ModelRc::new(VecModel::from(rows)));
    g.set_more(PAGE.min(st.good_all.len() - st.good_shown) as i32);
}

fn convert_solar(ui: &AppWindow, st: &mut State) {
    let g = ui.global::<Data>();
    st.solar_target = None;
    match D::parse(&g.get_s_text()).filter(|d| d.in_range()) {
        None => {
            g.set_s_ok(false);
            g.set_s_big(sv(format!(
                "Hãy nhập một ngày dương lịch từ năm {MIN_YEAR} đến năm {MAX_YEAR}."
            )));
            g.set_s_sub(sv(""));
        }
        Some(s) => {
            let info = lvhv_ffi::day_info_json(s.y, s.m, s.d)
                .ok()
                .and_then(|j| serde_json::from_str::<JDay>(&j).ok());
            if let Some(i) = info {
                g.set_s_ok(true);
                g.set_s_big(sv(format!(
                    "{} tháng {}{}",
                    i.lunar.d,
                    LUNAR_MONTH[i.lunar.m.clamp(0, 12) as usize],
                    if i.lunar.leap { " (nhuận)" } else { "" }
                )));
                g.set_s_sub(sv(format!(
                    "Năm {} · ngày {} · tháng {}",
                    i.can_chi.year, i.can_chi.day, i.can_chi.month
                )));
                g.set_s_link(sv(format!(
                    "Mở tờ lịch {}, {}",
                    WEEKDAY[s.weekday()],
                    s.dmy()
                )));
                st.solar_target = Some(s);
            }
        }
    }
}

fn convert_lunar(ui: &AppWindow, st: &mut State) {
    let g = ui.global::<Data>();
    st.lunar_target = None;
    g.set_l_ok(false);
    let Some(ly) = g
        .get_l_year()
        .trim()
        .parse::<i32>()
        .ok()
        .filter(|y| (MIN_YEAR..=MAX_YEAR).contains(y))
    else {
        g.set_l_big(sv(format!(
            "Hãy nhập năm âm lịch từ {MIN_YEAR} đến {MAX_YEAR}."
        )));
        return;
    };
    let (lm, ld, leap) = (g.get_l_month(), g.get_l_day(), g.get_l_leap());
    match lvhv_ffi::lunar_to_solar(ly, lm, ld, leap).map(D::from) {
        Some(s) => {
            g.set_l_ok(true);
            g.set_l_big(sv(format!("{}, {}", WEEKDAY[s.weekday()], s.dmy())));
            st.lunar_target = Some(s);
        }
        None => {
            let plain = leap && lvhv_ffi::lunar_to_solar(ly, lm, ld, false).is_some();
            g.set_l_big(sv(if leap {
                format!(
                    "Năm {ly} không có tháng {lm} nhuận{}. Hãy bỏ chọn \"Tháng nhuận\" hoặc chọn tháng khác.",
                    if plain {
                        String::new()
                    } else {
                        format!(", hoặc tháng này không có ngày {ld}")
                    }
                )
            } else {
                format!("Tháng {lm} âm lịch năm {ly} không có ngày {ld} (tháng thiếu chỉ có 29 ngày).")
            }));
        }
    }
}

fn blocks(json: &str) -> ModelRc<Block> {
    let v: Vec<JBlock> = serde_json::from_str(json).unwrap_or_default();
    ModelRc::new(VecModel::from(
        v.into_iter()
            .map(|b| Block {
                kind: sv(b.kind),
                text: sv(b.text),
            })
            .collect::<Vec<_>>(),
    ))
}

/// Chế độ chạy giao diện. Bản phát hành chỉ dùng `Normal`. `Ci` mang một hàm do `ci.rs` truyền vào (chụp màn hình, đo
/// bộ nhớ): `LichViet.exe` không có đường gọi nào tới mã chụp màn hình (`shots.rs`, gồm hình mâm vẽ bằng mã), nên trình
/// liên kết bỏ hẳn mã đó khỏi bản phát hành — CI kiểm bằng cách tìm tên bước chụp trong tệp (P62-10).
pub(crate) enum UiMode {
    Normal,
    Ci(Box<dyn FnOnce(&AppWindow, Rc<RefCell<State>>)>),
}

/// Điểm vào của `LichViet.exe`. Tham số duy nhất được hiểu: URI `lichviet:nguoi/<id>?…` khi người dùng bấm vào thông
/// báo (giao thức do bộ cài đăng ký). Mọi tham số khác bị bỏ qua.
pub fn run_app() -> Result<(), slint::PlatformError> {
    run_ui(UiMode::Normal, std::env::args().skip(1).collect())
}

pub(crate) fn run_ui(mode: UiMode, args: Vec<String>) -> Result<(), slint::PlatformError> {
    let open_person = args
        .iter()
        .find_map(|a| toasts::parse_launch(a))
        .and_then(|l| l.person);
    let ui = AppWindow::new()?;
    let st = setup(&ui);
    match mode {
        UiMode::Ci(f) => f(&ui, st),
        UiMode::Normal => {
            if let Some(id) = open_person {
                // Mở đúng hồ sơ của lời nhắc (chỉ khi hồ sơ còn trong kho).
                ui.set_tab(4);
                ui.global::<Data>().invoke_open_people();
                // Mở thẳng trang hồ sơ là cố ý (bấm thông báo mở đúng người). Khối thông tin báo tử đã thu gọn sẵn trong
                // `open_person` như mọi lần mở hồ sơ — không phải biện pháp riêng cho URI (08 §14 #12, DV25-06, P57-03).
                ui.global::<People>().invoke_open_person(id.into());
            }
        }
    }
    // §5.3: bản sao tạm của tệp đính kèm chỉ sống trong một lần chạy — dọn khi mở và khi đóng.
    let data = platform::data_dir();
    if !data.as_os_str().is_empty() {
        attach::cleanup(&data);
    }
    let r = ui.run();
    if !data.as_os_str().is_empty() {
        attach::cleanup(&data);
    }
    r
}

/// Nạp dữ liệu ban đầu và nối mọi callback giữa giao diện và lõi. Dùng chung cho `main` và kiểm thử.
fn setup(ui: &AppWindow) -> Rc<RefCell<State>> {
    let t = today();
    let acts: Vec<JAct> = serde_json::from_str(&lvhv_ffi::activities_json()).unwrap_or_default();
    let st = Rc::new(RefCell::new(State {
        date: t,
        today: t,
        view: (t.y, t.m),
        acts,
        good_all: Vec::new(),
        good_shown: 0,
        solar_target: None,
        lunar_target: None,
        people: Rc::new(RefCell::new(people::PState::new(platform::data_dir()))),
    }));
    platform::init_app_identity();
    let ps = st.borrow().people.clone();
    people::wire(ui, ps.clone());
    // Đã từng dùng mô-đun (có kho trên máy): mở kho và đặt lại thông báo Windows ngay khi mở ứng dụng
    // (02 §7.5). Chưa từng dùng thì không tạo tệp nào.
    if ps.borrow().dir.join("lvhv.sqlite").exists() {
        let mut p = ps.borrow_mut();
        if people::ensure_store(ui, &mut p) {
            people::reschedule(Some(ui), &mut p);
        }
    }

    {
        let g = ui.global::<Data>();
        let s = st.borrow();
        g.set_activities(ModelRc::new(VecModel::from(
            s.acts
                .iter()
                .map(|a| sv(a.name.as_str()))
                .collect::<Vec<_>>(),
        )));
        g.set_from_text(sv(t.dmy()));
        g.set_to_text(sv(t.add(30).unwrap_or(t).dmy()));
        g.set_s_text(sv(t.dmy()));
        if let Ok(l) = lvhv_ffi::solar_to_lunar(t.y, t.m, t.d) {
            g.set_l_day(l.d);
            g.set_l_month(l.m);
            g.set_l_year(sv(l.y.to_string()));
            g.set_l_leap(l.leap);
        }
        g.set_today_text(sv(format!("Hôm nay (giờ Việt Nam): {}.", t.dmy())));
        g.set_range_text(sv(format!(
            "Đổi qua lại giữa dương lịch và âm lịch, từ ngày 01/01/{MIN_YEAR} đến ngày 31/12/{MAX_YEAR}."
        )));
        g.set_version(sv(update::VERSION));
        g.set_running_version(sv(update::running_version()));
        g.set_copyright_sign(sv(update::COPYRIGHT_SIGN));
        g.set_copyright_holder(sv(update::COPYRIGHT_HOLDER));
        g.set_version_line(sv(update::version_line()));
        let today_days =
            update::parse_days(&format!("{:04}-{:02}-{:02}", t.y, t.m, t.d)).unwrap_or(0);
        g.set_remind(update::should_remind(
            update::parse_days(update::BUILD_DATE),
            today_days,
            update::snoozed_until(),
        ));
        let method = match lvhv_ffi::lunar_method().as_str() {
            "jpl" => "lịch thiên văn JPL DE441 của NASA JPL, với ΔT theo Morrison và Stephenson. Với những năm rất xa, kết quả kém chắc chắn hơn vì không ai dự báo chính xác được tốc độ quay của Trái Đất (ΔT). Sau năm 2077, vài ngày có trăng mới hoặc tiết khí rơi sát nửa đêm có thể lệch một ngày so với các lịch khác".to_string(),
            "precise" => "mô hình thiên văn chính xác hơn (Meeus đầy đủ)".to_string(),
            "hnd" => "thuật toán Hồ Ngọc Đức (phổ biến trong các lịch Việt)".to_string(),
            other => other.to_string(),
        };
        g.set_method_text(sv(format!("Ngày âm lịch tính theo {method}.")));
        g.set_attribution(sv(lvhv_ffi::cadao_attribution()));
        g.set_cadao_count_text(sv(format!(
            "Bộ dữ liệu gồm {} câu, phát hành theo giấy phép CC BY-SA 4.0 (Creative Commons Ghi công – Chia sẻ tương tự). Wikiquote và Wikimedia không liên quan và không xác nhận ứng dụng này.",
            lvhv_ffi::cadao_count()
        )));
        g.set_rule_sources(blocks(&lvhv_ffi::ngochap_sources_blocks_json()));
        g.set_cadao_sources(blocks(&lvhv_ffi::cadao_sources_blocks_json()));
    }
    render_sheet(ui, &st.borrow());
    render_month(ui, &st.borrow());
    convert_solar(ui, &mut st.borrow_mut());
    convert_lunar(ui, &mut st.borrow_mut());

    let g = ui.global::<Data>();
    let w = ui.as_weak();

    macro_rules! with {
        ($body:expr) => {{
            let w = w.clone();
            let st = st.clone();
            move || {
                if let Some(ui) = w.upgrade() {
                    #[allow(clippy::redundant_closure_call)]
                    ($body)(&ui, &mut st.borrow_mut());
                }
            }
        }};
    }
    macro_rules! with1 {
        ($t:ty, $body:expr) => {{
            let w = w.clone();
            let st = st.clone();
            move |a: $t| {
                if let Some(ui) = w.upgrade() {
                    #[allow(clippy::redundant_closure_call)]
                    ($body)(&ui, &mut st.borrow_mut(), a);
                }
            }
        }};
    }

    g.on_step(with1!(i32, |ui: &AppWindow, st: &mut State, n: i32| {
        if let Some(d) = st.date.add(n).filter(|d| d.in_range()) {
            set_date(ui, st, d);
        }
    }));
    g.on_go_today(with!(|ui: &AppWindow, st: &mut State| {
        st.today = today();
        let t = st.today;
        set_date(ui, st, t);
    }));
    g.on_open_month(with!(|ui: &AppWindow, st: &mut State| {
        st.view = (st.date.y, st.date.m);
        render_month(ui, st);
    }));
    g.on_month_shift(with1!(i32, |ui: &AppWindow, st: &mut State, n: i32| {
        let t = st.view.0 * 12 + (st.view.1 - 1) + n;
        let (y, m) = (t.div_euclid(12), t.rem_euclid(12) + 1);
        st.view = if y < MIN_YEAR {
            (MIN_YEAR, 1)
        } else if y > MAX_YEAR {
            (MAX_YEAR, 12)
        } else {
            (y, m)
        };
        render_month(ui, st);
    }));
    g.on_month_set(with1!(i32, |ui: &AppWindow, st: &mut State, m: i32| {
        st.view.1 = m.clamp(1, 12);
        render_month(ui, st);
    }));
    g.on_year_set(with1!(
        SharedString,
        |ui: &AppWindow, st: &mut State, t: SharedString| {
            if let Some(y) = t
                .trim()
                .parse::<i32>()
                .ok()
                .filter(|y| (MIN_YEAR..=MAX_YEAR).contains(y))
            {
                st.view.0 = y;
                render_month(ui, st);
            }
        }
    ));
    g.on_month_pick(with1!(i32, |ui: &AppWindow, st: &mut State, d: i32| {
        let (y, m) = st.view;
        set_date(ui, st, D { y, m, d });
    }));
    g.on_recompute_good(with!(|ui: &AppWindow, st: &mut State| recompute_good(
        ui, st
    )));
    g.on_load_more(with!(|ui: &AppWindow, st: &mut State| {
        st.good_shown = (st.good_shown + PAGE).min(st.good_all.len());
        render_good(ui, st);
    }));
    g.on_open_good(with1!(i32, |ui: &AppWindow, st: &mut State, i: i32| {
        if let Some(x) = st.good_all.get(i as usize) {
            let d = D {
                y: x.solar.y,
                m: x.solar.m,
                d: x.solar.d,
            };
            set_date(ui, st, d);
            ui.set_tab(0);
        }
    }));
    g.on_convert_solar(with!(|ui: &AppWindow, st: &mut State| convert_solar(
        ui, st
    )));
    g.on_convert_lunar(with!(|ui: &AppWindow, st: &mut State| convert_lunar(
        ui, st
    )));
    g.on_open_solar(with!(|ui: &AppWindow, st: &mut State| {
        if let Some(d) = st.solar_target {
            set_date(ui, st, d);
            ui.set_tab(0);
        }
    }));
    g.on_open_lunar(with!(|ui: &AppWindow, st: &mut State| {
        if let Some(d) = st.lunar_target {
            set_date(ui, st, d);
            ui.set_tab(0);
        }
    }));

    {
        let w = w.clone();
        let ps = ps.clone();
        g.on_open_people(move || {
            if let Some(ui) = w.upgrade() {
                let mut p = ps.borrow_mut();
                let fresh = p.store.is_none();
                let page = ui.global::<People>().get_page().to_string();
                if fresh {
                    people::go(&ui, &mut p, if page.is_empty() { "sap-toi" } else { &page });
                    people::reschedule(Some(&ui), &mut p);
                } else {
                    people::refresh(&ui, &mut p);
                }
            }
        });
    }
    g.on_check_update(|| {
        update::open_with_os(&update::update_url(update::MS_STORE_ID));
    });
    {
        let w = w.clone();
        g.on_remind_later(move || {
            let t = today();
            if let Some(d) = update::parse_days(&format!("{:04}-{:02}-{:02}", t.y, t.m, t.d)) {
                update::snooze(d);
            }
            if let Some(ui) = w.upgrade() {
                ui.global::<Data>().set_remind(false);
            }
        });
    }
    {
        let w = w.clone();
        g.on_open_rules(move || {
            if let Some(ui) = w.upgrade() {
                ui.global::<Data>().set_about_rules(true);
                ui.set_tab(3);
            }
        });
    }
    st
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_dmy() {
        let d = D::parse("05/11/2026").unwrap();
        assert_eq!(
            d,
            D {
                y: 2026,
                m: 11,
                d: 5
            }
        );
        assert_eq!(d.dmy(), "05/11/2026");
        assert!(D::parse("30/02/2026").is_none());
        assert_eq!(d.weekday(), 4);
        assert_eq!(
            d.add(3).unwrap(),
            D {
                y: 2026,
                m: 11,
                d: 8
            }
        );
    }

    #[test]
    fn today_in_range() {
        assert!(today().in_range());
    }

    #[test]
    fn json_shapes_match_core() {
        let j: JDay = serde_json::from_str(&lvhv_ffi::day_info_json(2026, 11, 5).unwrap()).unwrap();
        assert_eq!(j.can_chi.day, "Quý Mùi");
        let a: Vec<JAct> = serde_json::from_str(&lvhv_ffi::activities_json()).unwrap();
        assert!(!a.is_empty());
        let _: Vec<JGood> = serde_json::from_str(
            &lvhv_ffi::find_good_days_json(
                a[0].id.clone(),
                D {
                    y: 2026,
                    m: 11,
                    d: 6,
                }
                .solar(),
                D {
                    y: 2027,
                    m: 1,
                    d: 31,
                }
                .solar(),
                None,
            )
            .unwrap(),
        )
        .unwrap();
        let _: JCadao = serde_json::from_str(&lvhv_ffi::cadao_for(2026, 11, 6).unwrap()).unwrap();
        let b: Vec<JBlock> =
            serde_json::from_str(&lvhv_ffi::ngochap_sources_blocks_json()).unwrap();
        assert!(!b.is_empty());
    }
}
