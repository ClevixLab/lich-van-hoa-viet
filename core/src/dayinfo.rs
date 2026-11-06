//! `DayInfo`, `DayCtx` và bộ tuần tự hóa JSON viết tay (thứ tự khóa cố định theo hợp đồng §4).

use crate::canchi::*;
use crate::error::{checked_jdn, DateError, MIN_YEAR};
use crate::jd::{date_from_jdn, weekday};
use crate::lunar::{jdn_to_lunar_with, LunarDate, Method, DEFAULT_METHOD};
use crate::terms::{term_index_of_day, tiet_month_chi_of_term, TERM_NAMES};

/// Ngữ cảnh số của một ngày — dành cho bộ quy tắc Ngọc Hạp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayCtx {
    pub jdn: i64,
    /// 0 = Chủ nhật … 6 = Thứ Bảy.
    pub weekday: u32,
    pub lunar_year: i32,
    pub lunar_month: u32,
    pub lunar_day: u32,
    pub leap: bool,
    pub month_len: u32,
    /// Can ngày 0..9 (0 = Giáp).
    pub day_can: u32,
    /// Chi ngày 0..11 (0 = Tý).
    pub day_chi: u32,
    /// Can/chi tháng âm (tháng nhuận dùng của tháng chính).
    pub month_can: u32,
    pub month_chi: u32,
    /// Chi tháng tiết khí 0..11 (Lập Xuân → Dần = 2).
    pub month_chi_tiet: u32,
    pub year_can: u32,
    pub year_chi: u32,
    /// Can của giờ Tý đầu ngày.
    pub hour_ty_can: u32,
    /// Tiết khí 0..23 (0 = Xuân Phân, kinh độ 15·i°).
    pub tiet_khi: u32,
    /// Trực 0..11 (0 = Kiến).
    pub truc: u32,
    /// Nhị thập bát tú 0..27 (0 = Giác).
    pub tu: u32,
    /// 12 thần 0..11 (0 = Thanh Long).
    pub than: u32,
    /// Nạp âm 0..29.
    pub nap_am: u32,
}

/// Tính `DayCtx` từ JDN (phương pháp mặc định). `Err(OutOfRange)` nếu JDN ngoài 2000–3000.
pub fn day_ctx_jdn(jdn: i64) -> Result<DayCtx, DateError> {
    day_ctx_jdn_with(DEFAULT_METHOD, jdn)
}

/// Tính `DayCtx` từ JDN theo phương pháp chỉ định. Chỉ phần âm lịch (và những gì suy ra từ
/// tháng/năm âm: Can Chi tháng/năm, thần ngày) phụ thuộc phương pháp; tiết khí, Trực, tú,
/// Can Chi ngày như nhau ở cả hai.
pub fn day_ctx_jdn_with(method: Method, jdn: i64) -> Result<DayCtx, DateError> {
    let l: LunarDate = jdn_to_lunar_with(method, jdn)?;
    let dcan = day_can(jdn);
    let dchi = day_chi(jdn);
    let ycan = year_can(l.year);
    let term = term_index_of_day(jdn);
    let mchi_tiet = tiet_month_chi_of_term(term);
    Ok(DayCtx {
        jdn,
        weekday: weekday(jdn),
        lunar_year: l.year,
        lunar_month: l.month,
        lunar_day: l.day,
        leap: l.leap,
        month_len: l.month_len,
        day_can: dcan,
        day_chi: dchi,
        month_can: month_can(ycan, l.month),
        month_chi: month_chi(l.month),
        month_chi_tiet: mchi_tiet,
        year_can: ycan,
        year_chi: year_chi(l.year),
        hour_ty_can: hour_ty_can(dcan),
        tiet_khi: term as u32,
        truc: truc_index(dchi, mchi_tiet),
        tu: tu_index(jdn),
        than: than_index(l.month, dchi),
        nap_am: nap_am_index(dcan, dchi),
    })
}

/// Tính `DayCtx` cho ngày dương. `Err(InvalidDate)` nếu ngày không tồn tại, `Err(OutOfRange)`
/// nếu ngoài 2000–3000.
pub fn day_ctx(y: i32, m: u32, d: u32) -> Result<DayCtx, DateError> {
    day_ctx_with(DEFAULT_METHOD, y, m, d)
}

/// Tính `DayCtx` cho ngày dương theo phương pháp chỉ định.
pub fn day_ctx_with(method: Method, y: i32, m: u32, d: u32) -> Result<DayCtx, DateError> {
    day_ctx_jdn_with(method, checked_jdn(y, m, d, MIN_YEAR)?)
}

/// Một giờ Hoàng đạo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GioHoangDao {
    pub chi: &'static str,
    pub from: u32,
    pub to: u32,
}

/// Kết quả Ngọc Hạp (do crate `lvhv-ngochap` tính, xem `ngochap_ctx`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NgocHap {
    pub sao_tot: Vec<String>,
    pub sao_xau: Vec<String>,
    pub nen: Vec<String>,
    pub ky: Vec<String>,
    pub xung_tuoi: Vec<String>,
    pub score: i32,
}

/// Thông tin đầy đủ của một ngày.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayInfo {
    pub solar_y: i32,
    pub solar_m: u32,
    pub solar_d: u32,
    pub weekday: u32,
    pub jdn: i64,
    pub lunar: LunarDate,
    pub cc_year: String,
    pub cc_month: String,
    pub cc_day: String,
    pub cc_hour_ty: String,
    pub tiet_khi: &'static str,
    pub nap_am: &'static str,
    pub truc: &'static str,
    pub tu: &'static str,
    pub than_name: &'static str,
    pub than_hoang_dao: bool,
    pub gio_hoang_dao: Vec<GioHoangDao>,
    pub ngoc_hap: NgocHap,
    /// Ngữ cảnh số (không xuất ra JSON).
    pub ctx: DayCtx,
}

/// Tính `DayInfo` cho ngày dương. `Err(InvalidDate)` nếu ngày không tồn tại (30/2, tháng 13,
/// 29/2 năm không nhuận…), `Err(OutOfRange)` nếu ngoài 2000-01-01 … 3000-12-31.
pub fn day_info(y: i32, m: u32, d: u32) -> Result<DayInfo, DateError> {
    day_info_with(DEFAULT_METHOD, y, m, d)
}

/// Tính `DayInfo` theo phương pháp chỉ định.
pub fn day_info_with(method: Method, y: i32, m: u32, d: u32) -> Result<DayInfo, DateError> {
    let c = day_ctx_with(method, y, m, d)?;
    let name = |can, chi| can_chi_name(can, chi).ok_or(DateError::Computation);
    let gio = gio_hoang_dao(c.day_chi)
        .iter()
        .map(|&h| {
            let (from, to) = hour_range(h);
            GioHoangDao {
                chi: CHI[h as usize],
                from,
                to,
            }
        })
        .collect();
    Ok(DayInfo {
        solar_y: y,
        solar_m: m,
        solar_d: d,
        weekday: c.weekday,
        jdn: c.jdn,
        lunar: LunarDate {
            year: c.lunar_year,
            month: c.lunar_month,
            day: c.lunar_day,
            leap: c.leap,
            month_len: c.month_len,
        },
        cc_year: name(c.year_can, c.year_chi)?,
        cc_month: name(c.month_can, c.month_chi)?,
        cc_day: name(c.day_can, c.day_chi)?,
        cc_hour_ty: name(c.hour_ty_can, 0)?,
        tiet_khi: TERM_NAMES[c.tiet_khi as usize],
        nap_am: NAP_AM[c.nap_am as usize],
        truc: TRUC[c.truc as usize],
        tu: TU[c.tu as usize],
        than_name: THAN[c.than as usize],
        than_hoang_dao: THAN_HOANG_DAO[c.than as usize],
        gio_hoang_dao: gio,
        ngoc_hap: ngoc_hap_of(&c).ok_or(DateError::Computation)?,
        ctx: c,
    })
}

/// Đổi `DayCtx` của lõi sang ngữ cảnh của crate Ngọc Hạp (chỉ số nhỏ, kèm ngày/tháng dương).
///
/// `None` nếu một trường không vừa `u8` hoặc ngữ cảnh không qua `lvhv_ngochap::DayCtx::validate`
/// (tháng 1..12, ngày 1..30, can < 10, chi < 12, can/chi cùng chẵn lẻ…). Trước đây dùng `as u8`
/// với chú thích sai "mọi chỉ số < 60" — `lunar_day` của đầu vào ngoài miền có thể là 167 811
/// và bị cắt lặng lẽ (RS-05/C-3). Mọi `DayCtx` do lõi tạo (đã qua kiểm miền) đều hợp lệ.
pub fn ngochap_ctx(c: &DayCtx) -> Option<lvhv_ngochap::DayCtx> {
    let (_, sm, sd) = date_from_jdn(c.jdn);
    let u = |x: u32| u8::try_from(x).ok();
    let n = lvhv_ngochap::DayCtx {
        lunar_month: u(c.lunar_month)?,
        lunar_day: u(c.lunar_day)?,
        leap: c.leap,
        day_can: u(c.day_can)?,
        day_chi: u(c.day_chi)?,
        month_chi_tiet: u(c.month_chi_tiet)?,
        year_can: u(c.year_can)?,
        year_chi: u(c.year_chi)?,
        truc: u(c.truc)?,
        tu: u(c.tu)?,
        than: u(c.than)?,
        solar_month: u(sm)?,
        solar_day: u(sd)?,
    };
    n.validate().ok()?;
    Some(n)
}

/// Kết quả Ngọc Hạp của một ngày (`None` nếu ngữ cảnh không hợp lệ, xem `ngochap_ctx`).
pub fn ngoc_hap_of(c: &DayCtx) -> Option<NgocHap> {
    let e = lvhv_ngochap::evaluate(&ngochap_ctx(c)?);
    let own = |v: Vec<&'static str>| v.into_iter().map(String::from).collect();
    Some(NgocHap {
        sao_tot: own(e.sao_tot),
        sao_xau: own(e.sao_xau),
        nen: own(e.nen),
        ky: own(e.ky),
        xung_tuoi: e.xung_tuoi,
        score: e.score,
    })
}

/// Ghi chuỗi JSON có thoát ký tự (UTF-8 giữ nguyên, không thoát \u cho ký tự không phải ASCII).
pub fn write_json_str(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write_str_array(out: &mut String, v: &[String]) {
    out.push('[');
    for (i, s) in v.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_json_str(out, s);
    }
    out.push(']');
}

/// Ghi phần `ngoc_hap` (dùng chung cho bộ tích hợp).
pub fn write_ngoc_hap_json(out: &mut String, n: &NgocHap) {
    out.push_str("{\"sao_tot\":");
    write_str_array(out, &n.sao_tot);
    out.push_str(",\"sao_xau\":");
    write_str_array(out, &n.sao_xau);
    out.push_str(",\"nen\":");
    write_str_array(out, &n.nen);
    out.push_str(",\"ky\":");
    write_str_array(out, &n.ky);
    out.push_str(",\"xung_tuoi\":");
    write_str_array(out, &n.xung_tuoi);
    out.push_str(",\"score\":");
    out.push_str(&n.score.to_string());
    out.push('}');
}

impl DayInfo {
    /// JSON một dòng, không khoảng trắng, thứ tự khóa đúng hợp đồng §4.
    pub fn to_json(&self) -> String {
        let mut o = String::with_capacity(640);
        o.push_str(&format!(
            "{{\"solar\":{{\"y\":{},\"m\":{},\"d\":{},\"weekday\":{},\"jdn\":{}}}",
            self.solar_y, self.solar_m, self.solar_d, self.weekday, self.jdn
        ));
        o.push_str(&format!(
            ",\"lunar\":{{\"y\":{},\"m\":{},\"d\":{},\"leap\":{},\"month_len\":{}}}",
            self.lunar.year,
            self.lunar.month,
            self.lunar.day,
            self.lunar.leap,
            self.lunar.month_len
        ));
        o.push_str(",\"can_chi\":{\"year\":");
        write_json_str(&mut o, &self.cc_year);
        o.push_str(",\"month\":");
        write_json_str(&mut o, &self.cc_month);
        o.push_str(",\"day\":");
        write_json_str(&mut o, &self.cc_day);
        o.push_str(",\"hour_ty\":");
        write_json_str(&mut o, &self.cc_hour_ty);
        o.push_str("},\"tiet_khi\":");
        write_json_str(&mut o, self.tiet_khi);
        o.push_str(",\"nap_am\":");
        write_json_str(&mut o, self.nap_am);
        o.push_str(",\"truc\":");
        write_json_str(&mut o, self.truc);
        o.push_str(",\"tu\":");
        write_json_str(&mut o, self.tu);
        o.push_str(",\"than\":{\"name\":");
        write_json_str(&mut o, self.than_name);
        o.push_str(&format!(",\"hoang_dao\":{}}}", self.than_hoang_dao));
        o.push_str(",\"gio_hoang_dao\":[");
        for (i, g) in self.gio_hoang_dao.iter().enumerate() {
            if i > 0 {
                o.push(',');
            }
            o.push_str("{\"chi\":");
            write_json_str(&mut o, g.chi);
            o.push_str(&format!(",\"from\":{},\"to\":{}}}", g.from, g.to));
        }
        o.push_str("],\"ngoc_hap\":");
        write_ngoc_hap_json(&mut o, &self.ngoc_hap);
        o.push('}');
        o
    }
}
