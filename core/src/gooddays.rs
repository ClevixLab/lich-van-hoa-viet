//! Chọn ngày tốt cho một việc (hợp đồng §4: `find_good_days`), dựa trên crate `lvhv-ngochap`.
//!
//! Quy tắc: duyệt mọi ngày trong [from, to], tính `suitability` của việc (kèm tuổi nếu có) và chỉ
//! **giữ** ngày thỏa cả ba điều (content/ngochap/SCORING.md §2a):
//! 1. `allowed = true` — không phạm sao kỵ chính (tiêu chí N3), không xung tuổi với việc cá nhân;
//! 2. điểm việc (đã gồm điểm trừ tuổi xung) `score ≥ MIN_GOOD_DAY_SCORE` (= 0);
//! 3. việc **không** nằm trong danh sách `ky` của tờ lịch ngày đó (`day_info(..).ngoc_hap.ky`).
//!
//! (1)+(2) đã kéo theo (3) vì `ky` = kỵ chính **hoặc** (sao xấu nêu đích danh và điểm việc < 0);
//! điều (3) vẫn được kiểm tường minh để tờ lịch và danh sách chọn ngày không bao giờ mâu thuẫn.
//! Xếp theo điểm giảm dần, cùng điểm thì ngày sớm trước. Điểm trong kết quả là **điểm cho việc
//! đã chọn**, khác điểm chung của ngày (`ngoc_hap.score`, dùng cho nhãn đánh giá tờ lịch).
//!
//! **Tuổi** (C-2): tham số `birth_lunar_year` là **năm âm lịch** của ngày sinh, không phải năm
//! dương. Người sinh trước Tết (thường tháng 1–2 dương) thuộc năm âm trước — ví dụ sinh
//! 15/01/1976 là năm âm 1975 (Ất Mão), không phải Bính Thìn. Dùng `lunar_year_of_birth(y, m, d)`
//! để đổi ngày sinh dương → năm âm. Năm sinh chỉ dùng khi tính, không được lưu (S4).

use crate::canchi::{gio_hoang_dao, hour_range, CHI};
use crate::dayinfo::{day_ctx_jdn_with, ngochap_ctx, write_json_str, GioHoangDao};
use crate::error::{checked_jdn, DateError, MAX_SPAN_DAYS, MIN_YEAR};
use crate::jd::date_from_jdn;
use crate::lunar::{LunarDate, Method, SolarDate, DEFAULT_METHOD};

pub use lvhv_ngochap::Activity;

/// Điểm việc tối thiểu để một ngày được đề xuất (điều 2 ở đầu mô-đun).
pub const MIN_GOOD_DAY_SCORE: i32 = 0;

/// Danh sách việc chọn được (thứ tự cố định, mã ASCII ổn định).
pub fn activities() -> &'static [Activity] {
    lvhv_ngochap::activities()
}

/// Một ngày phù hợp cho việc đã chọn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoodDay {
    pub solar: SolarDate,
    pub lunar: LunarDate,
    /// 0 = Chủ nhật … 6 = Thứ Bảy.
    pub weekday: u32,
    pub jdn: i64,
    /// Điểm của ngày cho việc này (SCORING.md §2), đã gồm điểm trừ tuổi xung nếu có.
    pub score: i32,
    /// `suitability(..).allowed`. Luôn `true` trong kết quả của `find_good_days`.
    pub allowed: bool,
    /// Việc này có nằm trong `ky` của tờ lịch ngày đó không (không xuất ra JSON; luôn `false`
    /// trong kết quả của `find_good_days`).
    pub in_sheet_ky: bool,
    /// Lý do theo thứ tự cố định (SCORING.md §2).
    pub reasons: Vec<String>,
    /// Giờ Hoàng đạo đầu tiên trong ngày (theo thứ tự Tý → Hợi).
    pub first_gio_hoang_dao: GioHoangDao,
}

impl GoodDay {
    /// Ngày có được đề xuất không (ba điều ở đầu mô-đun).
    pub fn is_suggestable(&self) -> bool {
        self.allowed && self.score >= MIN_GOOD_DAY_SCORE && !self.in_sheet_ky
    }

    /// JSON một dòng, thứ tự khóa cố định:
    /// `{"solar":{y,m,d},"lunar":{y,m,d,leap,month_len},"weekday","jdn","score","allowed","reasons","first_gio_hoang_dao":{chi,from,to}}`.
    pub fn to_json(&self) -> String {
        let mut o = String::with_capacity(320);
        o.push_str(&format!(
            "{{\"solar\":{{\"y\":{},\"m\":{},\"d\":{}}}",
            self.solar.year, self.solar.month, self.solar.day
        ));
        o.push_str(&format!(
            ",\"lunar\":{{\"y\":{},\"m\":{},\"d\":{},\"leap\":{},\"month_len\":{}}}",
            self.lunar.year,
            self.lunar.month,
            self.lunar.day,
            self.lunar.leap,
            self.lunar.month_len
        ));
        o.push_str(&format!(
            ",\"weekday\":{},\"jdn\":{},\"score\":{},\"allowed\":{},\"reasons\":[",
            self.weekday, self.jdn, self.score, self.allowed
        ));
        for (i, r) in self.reasons.iter().enumerate() {
            if i > 0 {
                o.push(',');
            }
            write_json_str(&mut o, r);
        }
        o.push_str("],\"first_gio_hoang_dao\":{\"chi\":");
        let g = &self.first_gio_hoang_dao;
        write_json_str(&mut o, g.chi);
        o.push_str(&format!(",\"from\":{},\"to\":{}}}}}", g.from, g.to));
        o
    }
}

/// Đánh giá một ngày (JDN) cho việc; trả `GoodDay` kể cả khi `allowed = false`.
/// `Err(OutOfRange)` nếu JDN ngoài 1900–3000; `Err(UnknownActivity)` nếu mã việc không tồn tại.
pub fn evaluate_day_for(
    method: Method,
    jdn: i64,
    activity_id: &str,
    birth_lunar_year: Option<i32>,
) -> Result<GoodDay, DateError> {
    if !activities().iter().any(|a| a.id == activity_id) {
        return Err(DateError::UnknownActivity);
    }
    let c = day_ctx_jdn_with(method, jdn)?;
    let nctx = ngochap_ctx(&c).ok_or(DateError::Computation)?;
    let s = lvhv_ngochap::suitability(&nctx, activity_id, birth_lunar_year);
    let name = activities()
        .iter()
        .find(|a| a.id == activity_id)
        .map(|a| a.name)
        .ok_or(DateError::UnknownActivity)?;
    let in_sheet_ky = lvhv_ngochap::evaluate(&nctx).ky.contains(&name);
    let (year, month, day) = date_from_jdn(jdn);
    let h = gio_hoang_dao(c.day_chi)[0];
    let (from, to) = hour_range(h);
    Ok(GoodDay {
        solar: SolarDate { year, month, day },
        lunar: LunarDate {
            year: c.lunar_year,
            month: c.lunar_month,
            day: c.lunar_day,
            leap: c.leap,
            month_len: c.month_len,
        },
        weekday: c.weekday,
        jdn,
        score: s.score,
        allowed: s.allowed,
        in_sheet_ky,
        reasons: s.reasons,
        first_gio_hoang_dao: GioHoangDao {
            chi: CHI[h as usize],
            from,
            to,
        },
    })
}

/// Ngày tốt cho việc `activity_id` trong [from, to] (phương pháp mặc định).
///
/// `birth_lunar_year`: năm âm lịch của ngày sinh (xem đầu mô-đun), `None` nếu không xét tuổi.
///
/// Lỗi (một kiểu, một cách báo — A-1):
/// - `InvalidDate` — `from`/`to` không tồn tại;
/// - `OutOfRange` — `from`/`to` ngoài 1900-01-01 … 3000-12-31;
/// - `InvertedRange` — `from > to`;
/// - `RangeTooLong` — `to − from > MAX_SPAN_DAYS` (366, tức tối đa 367 ngày kể cả hai đầu);
/// - `UnknownActivity` — mã việc không có trong `activities()`.
pub fn find_good_days(
    activity_id: &str,
    from: SolarDate,
    to: SolarDate,
    birth_lunar_year: Option<i32>,
) -> Result<Vec<GoodDay>, DateError> {
    find_good_days_with(DEFAULT_METHOD, activity_id, from, to, birth_lunar_year)
}

/// Ngày tốt theo phương pháp chỉ định (lỗi như `find_good_days`).
pub fn find_good_days_with(
    method: Method,
    activity_id: &str,
    from: SolarDate,
    to: SolarDate,
    birth_lunar_year: Option<i32>,
) -> Result<Vec<GoodDay>, DateError> {
    let a = checked_jdn(from.year, from.month, from.day, MIN_YEAR)?;
    let b = checked_jdn(to.year, to.month, to.day, MIN_YEAR)?;
    if a > b {
        return Err(DateError::InvertedRange);
    }
    if b - a > MAX_SPAN_DAYS {
        return Err(DateError::RangeTooLong { span_days: b - a });
    }
    if !activities().iter().any(|x| x.id == activity_id) {
        return Err(DateError::UnknownActivity);
    }
    let mut out = Vec::new();
    for j in a..=b {
        let g = evaluate_day_for(method, j, activity_id, birth_lunar_year)?;
        if g.is_suggestable() {
            out.push(g);
        }
    }
    out.sort_by(|p, q| q.score.cmp(&p.score).then(p.jdn.cmp(&q.jdn)));
    Ok(out)
}
