//! `lvhv-ngochap` — quy tắc xem ngày theo Ngọc Hạp Thông Thư cho ứng dụng
//! "Lịch và Văn hóa Việt" (Clevix Lab).
//!
//! Crate này chỉ nhận ngữ cảnh ngày đã tính sẵn (`DayCtx`, do lõi lịch cung cấp) và trả
//! về sao tốt, sao xấu, việc nên làm, việc kỵ, tuổi xung và điểm. Mọi quy tắc được ghi
//! nguồn trong `content/ngochap/rules.md`; cách tính điểm ở `content/ngochap/SCORING.md`.
//!
//! Quy ước:
//! - Bảng sao tra theo **số tháng âm lịch** (tháng nhuận dùng số của tháng chính cùng số).
//!   Bằng chứng: xem rules.md, mục "Tháng âm hay tháng tiết".
//! - "Mùa" của các sao theo mùa: Xuân = tháng 1–3, Hạ = 4–6, Thu = 7–9, Đông = 10–12 (âm lịch).
//! - Thứ tự đầu ra cố định: theo thứ tự bảng (ngày đặc biệt → sao xấu theo bảng → thần Hắc đạo;
//!   sao tốt theo bảng → thần Hoàng đạo); việc theo thứ tự `activities()`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod tables;

use tables::*;

/// Ngữ cảnh một ngày (do lõi lịch tính).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DayCtx {
    /// Tháng âm lịch 1..=12.
    pub lunar_month: u8,
    /// Ngày âm lịch 1..=30.
    pub lunar_day: u8,
    /// Tháng nhuận.
    pub leap: bool,
    /// Can ngày 0..10 (0 = Giáp).
    pub day_can: u8,
    /// Chi ngày 0..12 (0 = Tý).
    pub day_chi: u8,
    /// Chi của tháng tiết khí (chưa dùng trong bảng sao; giữ cho tương thích).
    pub month_chi_tiet: u8,
    /// Can năm âm lịch.
    pub year_can: u8,
    /// Chi năm âm lịch.
    pub year_chi: u8,
    /// Trực 0 = Kiến … 11 = Bế.
    pub truc: u8,
    /// Nhị thập bát tú 0 = Giác … 27 = Chẩn (chưa dùng để chấm điểm: chưa có nguồn bảng tốt/xấu).
    pub tu: u8,
    /// 12 thần 0 = Thanh Long … 11 = Câu Trần.
    pub than: u8,
    /// Tháng dương lịch.
    pub solar_month: u8,
    /// Ngày dương lịch.
    pub solar_day: u8,
}

/// Trường không hợp lệ của `DayCtx` (C-4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CtxError {
    /// `lunar_month` ngoài 1..=12.
    LunarMonth(u8),
    /// `lunar_day` ngoài 1..=30.
    LunarDay(u8),
    /// `day_can` ≥ 10 hoặc `year_can` ≥ 10.
    Can(u8),
    /// `day_chi` ≥ 12, `year_chi` ≥ 12 hoặc `month_chi_tiet` ≥ 12.
    Chi(u8),
    /// Cặp can/chi (ngày hoặc năm) lệch chẵn lẻ — không có trong vòng 60 Giáp Tý.
    Parity,
    /// `truc` ≥ 12, `than` ≥ 12 hoặc `tu` ≥ 28.
    Index(u8),
    /// `solar_month`/`solar_day` không hợp lệ (tháng 1..=12, ngày 1..=31).
    Solar,
}

impl DayCtx {
    /// Kiểm miền mọi trường. Các hàm `evaluate`/`suitability`/`major_taboos` vẫn **kẹp**
    /// (`clamp`, `%`) để không panic với đầu vào bất kỳ, nên kết quả với ngữ cảnh không hợp lệ
    /// là vô nghĩa nhưng trông hợp lệ — bên gọi (lõi `lvhv-core`) phải gọi `validate()` trước.
    pub fn validate(&self) -> Result<(), CtxError> {
        if !(1..=12).contains(&self.lunar_month) {
            return Err(CtxError::LunarMonth(self.lunar_month));
        }
        if !(1..=30).contains(&self.lunar_day) {
            return Err(CtxError::LunarDay(self.lunar_day));
        }
        for c in [self.day_can, self.year_can] {
            if c >= 10 {
                return Err(CtxError::Can(c));
            }
        }
        for z in [self.day_chi, self.year_chi, self.month_chi_tiet] {
            if z >= 12 {
                return Err(CtxError::Chi(z));
            }
        }
        if self.day_can % 2 != self.day_chi % 2 || self.year_can % 2 != self.year_chi % 2 {
            return Err(CtxError::Parity);
        }
        for i in [self.truc, self.than] {
            if i >= 12 {
                return Err(CtxError::Index(i));
            }
        }
        if self.tu >= 28 {
            return Err(CtxError::Index(self.tu));
        }
        if !(1..=12).contains(&self.solar_month) || !(1..=31).contains(&self.solar_day) {
            return Err(CtxError::Solar);
        }
        Ok(())
    }
}

/// Một loại việc để chọn ngày.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Activity {
    /// Mã ổn định (ASCII).
    pub id: &'static str,
    /// Tên hiển thị tiếng Việt.
    pub name: &'static str,
}

const ACTIVITIES: [Activity; N_ACT] = [
    Activity {
        id: "cuoi_hoi",
        name: "Cưới hỏi",
    },
    Activity {
        id: "xay_dung",
        name: "Xây dựng, sửa nhà",
    },
    Activity {
        id: "khai_truong",
        name: "Khai trương",
    },
    Activity {
        id: "an_tang",
        name: "An táng, cải táng",
    },
    Activity {
        id: "te_tu",
        name: "Tế tự, cúng lễ",
    },
    Activity {
        id: "dong_tho",
        name: "Động thổ",
    },
    Activity {
        id: "xuat_hanh",
        name: "Xuất hành",
    },
    Activity {
        id: "giao_dich",
        name: "Giao dịch, ký kết",
    },
    Activity {
        id: "cau_tai",
        name: "Cầu tài lộc",
    },
    Activity {
        id: "to_tung",
        name: "Kiện tụng, giải oan",
    },
    Activity {
        id: "lam_phuc",
        name: "Làm phúc, cầu phúc",
    },
    Activity {
        id: "nhap_trach",
        name: "Nhập trạch, về nhà mới",
    },
    Activity {
        id: "khai_nghiep",
        name: "Khởi sự, nhậm chức",
    },
    Activity {
        id: "cau_tu",
        name: "Cầu tự",
    },
    Activity {
        id: "nhap_hoc",
        name: "Nhập học",
    },
    Activity {
        id: "cau_suc_khoe",
        name: "Chữa bệnh, cầu sức khỏe",
    },
];

/// Danh sách việc (thứ tự cố định).
pub fn activities() -> &'static [Activity] {
    &ACTIVITIES
}

/// Kết quả xem ngày theo Ngọc Hạp.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NgocHap {
    /// Sao tốt (kể cả thần Hoàng đạo).
    pub sao_tot: Vec<&'static str>,
    /// Sao xấu (kể cả ngày đặc biệt và thần Hắc đạo).
    pub sao_xau: Vec<&'static str>,
    /// Việc nên làm (tên hiển thị).
    pub nen: Vec<&'static str>,
    /// Việc kỵ (tên hiển thị).
    pub ky: Vec<&'static str>,
    /// Tuổi (Can Chi năm sinh) xung khắc với ngày.
    pub xung_tuoi: Vec<String>,
    /// Điểm chung của ngày.
    pub score: i32,
}

/// Mức phù hợp của ngày cho một việc.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suitability {
    /// Có nên chọn ngày này cho việc này không (false nếu phạm sao kỵ chính hoặc tuổi xung).
    pub allowed: bool,
    /// Điểm cho việc này.
    pub score: i32,
    /// Lý do (theo thứ tự cố định).
    pub reasons: Vec<String>,
}

/// Ngưỡng điểm để một việc được ghi vào "nên làm" (xem SCORING.md).
pub const NEN_THRESHOLD: i32 = 3;
/// Điểm trừ khi tuổi người xem xung với ngày.
pub const XUNG_PENALTY: i32 = 3;

fn month_idx(ctx: &DayCtx) -> usize {
    (ctx.lunar_month.clamp(1, 12) - 1) as usize
}

fn season(ctx: &DayCtx) -> usize {
    month_idx(ctx) / 3
}

fn sexagenary(can: u8, chi: u8) -> Option<usize> {
    let (c, z) = (can as usize % 10, chi as usize % 12);
    if c % 2 != z % 2 {
        return None;
    }
    (0..60).find(|n| n % 10 == c && n % 12 == z)
}

fn rule_hits(rule: &Rule, ctx: &DayCtx) -> bool {
    let m = month_idx(ctx);
    let can = ctx.day_can % 10;
    let chi = ctx.day_chi % 12;
    match rule {
        Rule::MonthChi(t) => t[m] == chi,
        Rule::MonthChi2(t) => t[m].contains(&chi),
        Rule::MonthCan(t) => t[m] == can,
        Rule::MonthCond(t) => match t[m] {
            Cond::Can(c) => c == can,
            Cond::Chi(z) => z == chi,
        },
        Rule::MonthCanChi(t) => t[m] == (can, chi),
        Rule::MonthChiOpt(t) => t[m] == Some(chi),
        Rule::SeasonChi(t) => t[season(ctx)].contains(&chi),
        Rule::SeasonCan(t) => t[season(ctx)].contains(&can),
        Rule::SeasonCanChi(t) => t[season(ctx)] == (can, chi),
        Rule::CanChiSet(set) => set.contains(&(can, chi)),
        Rule::CanChiByGroup(g) => g[m % 3].contains(&(can, chi)),
        Rule::LunarDays(days) => days.contains(&ctx.lunar_day),
        Rule::MonthDays(md) => md.contains(&(m as u8 + 1, ctx.lunar_day)),
        Rule::ThapAc => {
            let grp = ctx.year_can % 5;
            THAP_AC
                .iter()
                .any(|&(g, mo, c, z)| g == grp && mo as usize == m + 1 && c == can && z == chi)
        }
        Rule::Than(i) => ctx.than % 12 == *i,
    }
}

/// Các sao hiện diện trong ngày: (sao tốt, sao xấu), thứ tự cố định.
fn stars_of(ctx: &DayCtx) -> (Vec<&'static Star>, Vec<&'static Star>) {
    let mut good: Vec<&'static Star> = GOOD_STARS
        .iter()
        .filter(|s| rule_hits(&s.rule, ctx))
        .collect();
    let mut bad: Vec<&'static Star> = SPECIAL_DAYS
        .iter()
        .chain(BAD_STARS.iter())
        .filter(|s| rule_hits(&s.rule, ctx))
        .collect();
    // Hoạt Điệu (G30) mất tác dụng khi gặp Thụ Tử (B14) — nguồn N01.
    if bad.iter().any(|s| s.id == "B14") {
        good.retain(|s| s.id != "G30");
    }
    let than = &THAN[(ctx.than % 12) as usize];
    if than.good {
        good.push(than);
    } else {
        bad.push(than);
    }
    (good, bad)
}

fn xung_list(can: u8, chi: u8) -> &'static [(u8, u8)] {
    match sexagenary(can, chi) {
        Some(n) => XUNG_TUOI[n],
        None => &[],
    }
}

fn can_chi_name(can: u8, chi: u8) -> String {
    format!(
        "{} {}",
        CAN_NAME[(can % 10) as usize],
        CHI_NAME[(chi % 12) as usize]
    )
}

/// Can Chi của năm sinh (năm âm lịch, quy ước như lõi: can = (Y+6) mod 10, chi = (Y+8) mod 12).
fn year_can_chi(y: i32) -> (u8, u8) {
    (
        (y as i64 + 6).rem_euclid(10) as u8,
        (y as i64 + 8).rem_euclid(12) as u8,
    )
}

/// Điểm của ngày cho việc `a` (không tính tuổi), kèm lý do và cờ phạm kỵ chính.
fn score_activity(
    ctx: &DayCtx,
    good: &[&'static Star],
    bad: &[&'static Star],
    a: Act,
) -> (i32, bool, Vec<String>) {
    let mut score = 0;
    let mut major = false;
    let mut reasons = Vec::new();
    for s in bad {
        if s.major.contains(a) {
            major = true;
            reasons.push(format!("Phạm sao kỵ chính: {}", s.name));
        }
    }
    for s in good {
        if s.ky.contains(a) {
            score -= s.weight;
            reasons.push(format!("−{} {} (kỵ việc này)", s.weight, s.name));
        } else if s.tot.contains(a) {
            let w = s.weight + i32::from(s.tot.names(a));
            score += w;
            reasons.push(format!("+{} {}", w, s.name));
        }
    }
    for s in bad {
        if s.ky.contains(a) {
            let w = s.weight + i32::from(s.ky.names(a));
            score -= w;
            reasons.push(format!("−{} {}", w, s.name));
        }
    }
    let t = TRUC_SCORE[(ctx.truc % 12) as usize];
    if t != 0 {
        // Dấu trừ U+2212 như các lý do khác (không dùng "-" ASCII của `{:+}`).
        let sign = if t > 0 { '+' } else { '−' };
        reasons.push(format!(
            "{sign}{} Trực {}",
            t.abs(),
            TRUC_NAME[(ctx.truc % 12) as usize]
        ));
        score += t;
    }
    (score, major, reasons)
}

const TRUC_NAME: [&str; 12] = [
    "Kiến", "Trừ", "Mãn", "Bình", "Định", "Chấp", "Phá", "Nguy", "Thành", "Thu", "Khai", "Bế",
];

/// Xem ngày: sao tốt, sao xấu, việc nên làm, việc kỵ, tuổi xung, điểm chung.
pub fn evaluate(ctx: &DayCtx) -> NgocHap {
    let (good, bad) = stars_of(ctx);
    let mut nen = Vec::new();
    let mut ky = Vec::new();
    for (i, act) in ACTIVITIES.iter().enumerate() {
        let a = i as Act;
        let (sc, major, _) = score_activity(ctx, &good, &bad, a);
        // "Kỵ": phạm sao kỵ chính, hoặc có sao xấu nêu đích danh việc này và điểm âm.
        let named_bad = bad.iter().any(|s| s.ky.names(a));
        if major || (named_bad && sc < 0) {
            ky.push(act.name);
        } else if !named_bad && sc >= NEN_THRESHOLD {
            nen.push(act.name);
        }
    }
    let score = good.iter().map(|s| s.weight).sum::<i32>()
        - bad.iter().map(|s| s.weight).sum::<i32>()
        + TRUC_SCORE[(ctx.truc % 12) as usize];
    NgocHap {
        sao_tot: good.iter().map(|s| s.name).collect(),
        sao_xau: bad.iter().map(|s| s.name).collect(),
        nen,
        ky,
        xung_tuoi: xung_list(ctx.day_can, ctx.day_chi)
            .iter()
            .map(|&(c, z)| can_chi_name(c, z))
            .collect(),
        score,
    }
}

/// Mức phù hợp của ngày cho việc `activity_id`, tùy chọn theo **năm âm lịch** của ngày sinh
/// `birth_lunar_year` (không phải năm dương — người sinh trước Tết thuộc năm âm trước; C-2).
///
/// `allowed = false` khi: mã việc không tồn tại; ngày phạm sao kỵ chính của việc (tiêu chí N3);
/// hoặc tuổi người xem xung với ngày và việc thuộc nhóm việc cá nhân (xem SCORING.md).
pub fn suitability(ctx: &DayCtx, activity_id: &str, birth_lunar_year: Option<i32>) -> Suitability {
    let Some(a) = ACTIVITIES.iter().position(|x| x.id == activity_id) else {
        return Suitability {
            allowed: false,
            score: 0,
            reasons: vec![format!("Không có việc mã \"{activity_id}\"")],
        };
    };
    let a = a as Act;
    let (good, bad) = stars_of(ctx);
    let (mut score, major, mut reasons) = score_activity(ctx, &good, &bad, a);
    let mut allowed = !major;
    if let Some(y) = birth_lunar_year {
        let (c, z) = year_can_chi(y);
        if xung_list(ctx.day_can, ctx.day_chi).contains(&(c, z)) {
            score -= XUNG_PENALTY;
            let personal = PERSONAL.contains(&a);
            if personal {
                allowed = false;
            }
            reasons.push(format!(
                "−{} Tuổi {} xung với ngày {}{}",
                XUNG_PENALTY,
                can_chi_name(c, z),
                can_chi_name(ctx.day_can, ctx.day_chi),
                if personal {
                    " (không nên chọn)"
                } else {
                    ""
                }
            ));
        }
    }
    Suitability {
        allowed,
        score,
        reasons,
    }
}

/// Tên các sao **kỵ chính** của việc `activity_id` có mặt trong ngày (định nghĩa dùng cho tiêu
/// chí N3: ngày có ít nhất một sao như vậy thì `suitability(..).allowed == false`).
/// Trả danh sách rỗng nếu không có sao nào hoặc mã việc không tồn tại. Thứ tự cố định (thứ tự bảng).
pub fn major_taboos(ctx: &DayCtx, activity_id: &str) -> Vec<&'static str> {
    let Some(a) = ACTIVITIES.iter().position(|x| x.id == activity_id) else {
        return Vec::new();
    };
    let (_, bad) = stars_of(ctx);
    bad.iter()
        .filter(|s| s.major.contains(a as Act))
        .map(|s| s.name)
        .collect()
}

#[cfg(test)]
mod tests;
