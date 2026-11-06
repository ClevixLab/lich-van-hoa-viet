//! Bảng tháng âm dựng sẵn từ `lvhv_core::lunar::YearTable` — chỉ để tính nhanh `L2S` và `len`
//! (02 §0). Kết quả **phải** trùng `lvhv_core::lunar_to_jdn_with` từng giá trị; kiểm vét cạn ở
//! `tests/cal_vs_core.rs` (mọi năm âm 1899–3000 × tháng × nhuận × ngày 1–30).

use lvhv_core::error::jdn_in_range;
use lvhv_core::jd::date_from_jdn;
use lvhv_core::lunar::YearTable;
use lvhv_core::{Method, SolarDate, MAX_LUNAR_YEAR, MIN_LUNAR_YEAR};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy)]
struct MonthEntry {
    month: u32,
    leap: bool,
    start: i64,
    len: u32,
}

/// Bảng tháng của mọi năm âm 1899–3000 theo một phương pháp.
#[derive(Debug)]
pub struct Cal {
    method: Method,
    /// `years[Y − 1899]` = các tháng của năm âm Y theo thứ tự thời gian.
    years: Vec<Vec<MonthEntry>>,
}

impl Cal {
    fn build(method: Method) -> Cal {
        let n = (MAX_LUNAR_YEAR - MIN_LUNAR_YEAR + 1) as usize;
        let mut years: Vec<Vec<MonthEntry>> = vec![Vec::new(); n];
        // Tuế yy chứa tháng 11–12 của năm âm yy−1 và tháng 1–10 của năm âm yy.
        for yy in MIN_LUNAR_YEAR..=MAX_LUNAR_YEAR + 1 {
            let Ok(t) = YearTable::new(method, yy) else {
                continue;
            };
            for i in 0..t.n {
                let Some(ly) = t.lunar_year(i) else { continue };
                if !(MIN_LUNAR_YEAR..=MAX_LUNAR_YEAR).contains(&ly) {
                    continue;
                }
                years[(ly - MIN_LUNAR_YEAR) as usize].push(MonthEntry {
                    month: t.month[i],
                    leap: t.leap[i],
                    start: t.starts[i],
                    len: (t.starts[i + 1] - t.starts[i]) as u32,
                });
            }
        }
        for y in &mut years {
            y.sort_by_key(|e| e.start);
        }
        Cal { method, years }
    }

    /// Bảng dùng chung (dựng một lần) cho phương pháp `method`.
    pub fn get(method: Method) -> &'static Cal {
        static HND: OnceLock<Cal> = OnceLock::new();
        static PRECISE: OnceLock<Cal> = OnceLock::new();
        static JPL: OnceLock<Cal> = OnceLock::new();
        match method {
            Method::Hnd => HND.get_or_init(|| Cal::build(Method::Hnd)),
            Method::Precise => PRECISE.get_or_init(|| Cal::build(Method::Precise)),
            Method::Jpl => JPL.get_or_init(|| Cal::build(Method::Jpl)),
        }
    }

    pub fn method(&self) -> Method {
        self.method
    }

    fn entry(&self, ly: i32, lm: u32, leap: bool) -> Option<&MonthEntry> {
        if !(MIN_LUNAR_YEAR..=MAX_LUNAR_YEAR).contains(&ly) {
            return None;
        }
        self.years[(ly - MIN_LUNAR_YEAR) as usize]
            .iter()
            .find(|e| e.month == lm && e.leap == leap)
    }

    /// `len(y, m, leap)`: số ngày của tháng âm (29/30), `None` nếu tháng không tồn tại.
    pub fn month_len(&self, ly: i32, lm: u32, leap: bool) -> Option<u32> {
        self.entry(ly, lm, leap).map(|e| e.len)
    }

    /// Tháng nhuận của năm âm `ly` (nếu có).
    pub fn leap_month(&self, ly: i32) -> Option<u32> {
        if !(MIN_LUNAR_YEAR..=MAX_LUNAR_YEAR).contains(&ly) {
            return None;
        }
        self.years[(ly - MIN_LUNAR_YEAR) as usize]
            .iter()
            .find(|e| e.leap)
            .map(|e| e.month)
    }

    /// `L2S` dạng JDN — trùng `lunar_to_jdn_with(method, …)`.
    pub fn l2s_jdn(&self, ly: i32, lm: u32, ld: u32, leap: bool) -> Option<i64> {
        if !(1..=30).contains(&ld) {
            return None;
        }
        let e = self.entry(ly, lm, leap)?;
        if ld > e.len {
            return None;
        }
        let j = e.start + i64::from(ld) - 1;
        jdn_in_range(j).then_some(j)
    }

    /// `L2S` — trùng `lunar_to_solar_with(method, …)`.
    pub fn l2s(&self, ly: i32, lm: u32, ld: u32, leap: bool) -> Option<SolarDate> {
        self.l2s_jdn(ly, lm, ld, leap).map(|j| {
            let (year, month, day) = date_from_jdn(j);
            SolarDate { year, month, day }
        })
    }
}
