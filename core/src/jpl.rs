//! Phương pháp `Jpl` (docs/03 §17, docs/02 "Bổ sung A4"): thời điểm Sóc và 24 tiết khí lấy từ bảng nhúng
//! tính bằng JPL DE441 (thang TT, đơn vị 0,01 s), ΔT theo Morrison–Stephenson–Hohenkerk–Zawilski 2021
//! (bảng S15.2020) + đoạn nối Hermite + parabol dài hạn Stephenson–Morrison–Hohenkerk 2016 (§17.4).
//!
//! Bảng giải mã một lần (`OnceLock`, an toàn, không `unsafe`). Mọi phép tra ngoài phạm vi bảng trả `None`
//! (lớp trên đổi thành `DateError::OutOfRange`) — không bao giờ rơi về `Precise` hay ngoại suy (§17.6.6).
//! Thứ tự phép toán số thực trùng `tsport/src/jpl.ts` để kết quả trùng từng bit.

use std::sync::OnceLock;

/// Bảng nhị phân sinh bởi `tools/jpl/gen_table.py` (định dạng ghi ở đầu tệp đó).
static DATA: &[u8] = include_bytes!("../data/jpl_events.bin");

/// Đầu phạm vi bảng: 1899-10-01 00:00 TT.
pub const JD_TT_START: f64 = 2414928.5;
/// Cuối phạm vi bảng (không gồm): 3002-02-01 00:00 TT.
pub const JD_TT_END: f64 = 2817548.5;

/// Hàng 25..58 của Table S15.2020 (K_i, K_{i+1}, a0, a1, a2, a3), docs/03 §17.4 (a).
const S15_2020: [[f64; 6]; 34] = [
    [1895.0, 1900.0, -5.017, 0.134, 3.138, -0.232],
    [1900.0, 1905.0, -1.977, 5.715, 2.443, -1.257],
    [1905.0, 1910.0, 4.923, 6.828, -1.329, 0.720],
    [1910.0, 1915.0, 11.142, 6.330, 0.831, -0.825],
    [1915.0, 1920.0, 17.479, 5.518, -1.643, 0.262],
    [1920.0, 1925.0, 21.617, 3.020, -0.856, 0.008],
    [1925.0, 1930.0, 23.789, 1.333, -0.831, 0.127],
    [1930.0, 1935.0, 24.418, 0.052, -0.449, 0.142],
    [1935.0, 1940.0, 24.164, -0.419, -0.022, 0.702],
    [1940.0, 1945.0, 24.426, 1.645, 2.086, -1.106],
    [1945.0, 1950.0, 27.050, 2.499, -1.232, 0.614],
    [1950.0, 1953.0, 28.932, 1.127, 0.220, -0.277],
    [1953.0, 1956.0, 30.002, 0.737, -0.610, 0.631],
    [1956.0, 1959.0, 30.760, 1.409, 1.282, -0.799],
    [1959.0, 1962.0, 32.652, 1.577, -1.115, 0.507],
    [1962.0, 1965.0, 33.621, 0.868, 0.406, 0.199],
    [1965.0, 1968.0, 35.093, 2.275, 1.002, -0.414],
    [1968.0, 1971.0, 37.956, 3.035, -0.242, 0.202],
    [1971.0, 1974.0, 40.951, 3.157, 0.364, -0.229],
    [1974.0, 1977.0, 44.244, 3.199, -0.323, 0.172],
    [1977.0, 1980.0, 47.291, 3.069, 0.193, -0.192],
    [1980.0, 1983.0, 50.361, 2.878, -0.384, 0.081],
    [1983.0, 1986.0, 52.936, 2.354, -0.140, -0.165],
    [1986.0, 1989.0, 54.984, 1.577, -0.637, 0.448],
    [1989.0, 1992.0, 56.373, 1.648, 0.708, -0.276],
    [1992.0, 1995.0, 58.453, 2.235, -0.121, 0.110],
    [1995.0, 1998.0, 60.678, 2.324, 0.210, -0.313],
    [1998.0, 2001.0, 62.898, 1.804, -0.729, 0.109],
    [2001.0, 2004.0, 64.083, 0.674, -0.402, 0.199],
    [2004.0, 2007.0, 64.553, 0.466, 0.194, -0.017],
    [2007.0, 2010.0, 65.197, 0.804, 0.144, -0.084],
    [2010.0, 2013.0, 66.061, 0.839, -0.109, 0.128],
    [2013.0, 2016.0, 66.920, 1.007, 0.277, -0.095],
    [2016.0, 2019.0, 68.109, 1.277, -0.007, -0.139],
];

const Y_MIN: f64 = 1895.0;
const Y_TABLE_END: f64 = 2019.0;
const Y_BRIDGE_END: f64 = 2800.0;

fn parabola(y: f64) -> f64 {
    let u = (y - 1825.0) / 100.0;
    -320.0 + 32.5 * u * u
}

fn parabola_slope(y: f64) -> f64 {
    2.0 * 32.5 * (y - 1825.0) / 10000.0
}

/// Năm thập phân của ΔT (§17.4): Y = 2000 + (JD_TT − 2451544,5)/365,2425.
pub fn decimal_year(jd_tt: f64) -> f64 {
    2000.0 + (jd_tt - 2451544.5) / 365.2425
}

/// ΔT (giây) của phương pháp `Jpl` tại năm thập phân `y` (§17.4). `None` nếu `y < 1895` hoặc NaN.
pub fn delta_t(y: f64) -> Option<f64> {
    if y.is_nan() || y < Y_MIN {
        return None;
    }
    if y < Y_TABLE_END {
        for r in S15_2020.iter() {
            if r[0] <= y && y < r[1] {
                let t = (y - r[0]) / (r[1] - r[0]);
                return Some(r[2] + t * (r[3] + t * (r[4] + t * r[5])));
            }
        }
        return None;
    }
    if y < Y_BRIDGE_END {
        let r = S15_2020[33];
        let p0 = r[2] + r[3] + r[4] + r[5];
        let m0 = (r[3] + 2.0 * r[4] + 3.0 * r[5]) / (r[1] - r[0]);
        let p1 = parabola(Y_BRIDGE_END);
        let m1 = parabola_slope(Y_BRIDGE_END);
        let h = Y_BRIDGE_END - Y_TABLE_END;
        let s = (y - Y_TABLE_END) / h;
        let s2 = s * s;
        let s3 = s2 * s;
        let h00 = 2.0 * s3 - 3.0 * s2 + 1.0;
        let h10 = s3 - 2.0 * s2 + s;
        let h01 = -2.0 * s3 + 3.0 * s2;
        let h11 = s3 - s2;
        return Some(h00 * p0 + h10 * h * m0 + h01 * p1 + h11 * h * m1);
    }
    Some(parabola(y))
}

/// JD (TT) từ số nguyên n (0,01 s TT kể từ J2000.0), §17.5.
pub fn jd_tt_of(n: i64) -> f64 {
    2451545.0 + n as f64 / 8640000.0
}

/// JD (UT) = JD_TT − ΔT/86400 (§17.6.1). `None` nếu ΔT không xác định.
pub fn tt_to_ut(jd_tt: f64) -> Option<f64> {
    delta_t(decimal_year(jd_tt)).map(|dt| jd_tt - dt / 86400.0)
}

/// Bảng đã giải mã.
#[derive(Debug)]
pub struct Table {
    /// Chỉ số Hồ Ngọc Đức của Sóc đầu tiên.
    pub k0: i64,
    /// n (0,01 s TT) của từng Sóc.
    pub soc_cs: Vec<i64>,
    /// JD (UT) của từng Sóc.
    pub soc_ut: Vec<f64>,
    /// Chỉ số (0..23) của tiết khí đầu tiên.
    pub i0: usize,
    /// n (0,01 s TT) của từng tiết khí (theo thời gian).
    pub tk_cs: Vec<i64>,
    /// JD (UT) của từng tiết khí.
    pub tk_ut: Vec<f64>,
}

struct Reader<'a> {
    b: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn zz(&mut self) -> Option<i64> {
        let mut u: u64 = 0;
        let mut s = 0u32;
        loop {
            let c = *self.b.get(self.pos)?;
            self.pos += 1;
            if s > 63 {
                return None;
            }
            u |= u64::from(c & 0x7F) << s;
            s += 7;
            if c < 0x80 {
                break;
            }
        }
        Some(((u >> 1) as i64) ^ -((u & 1) as i64))
    }
}

fn decode(b: &[u8]) -> Option<Table> {
    if b.get(..8)? != b"LVHVJPL1" {
        return None;
    }
    let mut r = Reader { b, pos: 8 };
    let k0 = r.zz()?;
    let ns = usize::try_from(r.zz()?).ok()?;
    let i0 = usize::try_from(r.zz()?).ok()?;
    let nt = usize::try_from(r.zz()?).ok()?;
    if i0 > 23 || ns < 3 || nt < 49 {
        return None;
    }
    let mut soc: Vec<i64> = Vec::with_capacity(ns);
    for j in 0..ns {
        let e = r.zz()?;
        soc.push(if j < 3 {
            e
        } else {
            e + 3 * soc[j - 1] - 3 * soc[j - 2] + soc[j - 3]
        });
    }
    let mut tk: Vec<i64> = Vec::with_capacity(nt);
    let mut r_prev = 0i64;
    for j in 0..nt {
        let e = r.zz()?;
        let rr = if j <= 48 { e } else { e + r_prev };
        r_prev = rr;
        tk.push(if j < 48 {
            rr
        } else {
            rr + 2 * tk[j - 24] - tk[j - 48]
        });
    }
    if r.pos != b.len() {
        return None;
    }
    let ut =
        |v: &Vec<i64>| -> Option<Vec<f64>> { v.iter().map(|&n| tt_to_ut(jd_tt_of(n))).collect() };
    let soc_ut = ut(&soc)?;
    let tk_ut = ut(&tk)?;
    // Bất biến: tăng ngặt.
    if soc_ut.windows(2).any(|w| w[1] <= w[0]) || tk_ut.windows(2).any(|w| w[1] <= w[0]) {
        return None;
    }
    Some(Table {
        k0,
        soc_cs: soc,
        soc_ut,
        i0,
        tk_cs: tk,
        tk_ut,
    })
}

/// Bảng đã giải mã (giải mã lần đầu gọi). `None` chỉ khi tệp nhúng hỏng (lỗi build).
pub fn table() -> Option<&'static Table> {
    static T: OnceLock<Option<Table>> = OnceLock::new();
    T.get_or_init(|| decode(DATA)).as_ref()
}

/// JD (UT) của Sóc thứ k (chỉ số Hồ Ngọc Đức). `None` nếu ngoài bảng.
pub fn new_moon_ut(k: i64) -> Option<f64> {
    let t = table()?;
    let j = usize::try_from(k.checked_sub(t.k0)?).ok()?;
    t.soc_ut.get(j).copied()
}

/// Chỉ số (0..23) của tiết khí muộn nhất có JD_UT ≤ `jd_ut` (§17.6.3–4). `None` nếu `jd_ut` trước tiết
/// khí đầu tiên hoặc không trước tiết khí cuối cùng của bảng (khi đó không chắc không có tiết khí nào sau).
pub fn term_at(jd_ut: f64) -> Option<usize> {
    let t = table()?;
    let last = *t.tk_ut.last()?;
    if jd_ut.is_nan() || jd_ut < t.tk_ut[0] || jd_ut >= last {
        return None;
    }
    // số phần tử ≤ jd_ut, ≥ 1
    let cnt = t.tk_ut.partition_point(|&x| x <= jd_ut);
    Some((t.i0 + cnt - 1) % 24)
}

/// Các tiết khí (chỉ số, JD UT) có JD_UT trong [from_ut, to_ut).
pub fn terms_between(from_ut: f64, to_ut: f64) -> Option<Vec<(usize, f64)>> {
    let t = table()?;
    let a = t.tk_ut.partition_point(|&x| x < from_ut);
    let b = t.tk_ut.partition_point(|&x| x < to_ut);
    Some((a..b).map(|j| ((t.i0 + j) % 24, t.tk_ut[j])).collect())
}

/// Ngày lịch D của tiết khí tại `t` (JD UT) theo nghĩa của `term_at`: D = ngày nhỏ nhất có 00:00 đầu ngày
/// (`local_midnight_ut`) ≥ t. Khi đó tiết khí được tính cho cung của ngày J ⇔ D ≤ J (§17.6.3), và cho tiết khí
/// của ngày J ⇔ D ≤ J + 1 (§17.6.4).
fn term_day(t: f64) -> i64 {
    use crate::astro::{local_day_of, local_midnight_ut};
    let mut j = local_day_of(t);
    while local_midnight_ut(j) < t {
        j += 1;
    }
    while local_midnight_ut(j - 1) >= t {
        j -= 1;
    }
    j
}

/// Mã nguồn `tsport/src/jpl_days.ts`: bảng **mức ngày** suy ra từ bảng thời điểm (ngày Sóc theo
/// `local_day_of`, ngày D của tiết khí theo `term_day`). Bản TypeScript (PWA) chỉ cần các ngày này để cho kết
/// quả giống hệt từng byte lõi Rust, nên không phải nhúng bảng thời điểm (giữ P3). Test
/// `core/tests/jpl_ts.rs` bắt buộc tệp đã commit trùng đầu ra hàm này.
pub fn days_ts_source() -> Option<String> {
    use crate::astro::local_day_of;
    let t = table()?;
    let soc: Vec<i64> = t.soc_ut.iter().map(|&x| local_day_of(x)).collect();
    let tk: Vec<i64> = t.tk_ut.iter().map(|&x| term_day(x)).collect();
    let mut sd = String::with_capacity(soc.len());
    for w in soc.windows(2) {
        let d = w[1] - w[0];
        if !(29..=30).contains(&d) {
            return None;
        }
        sd.push(if d == 29 { '0' } else { '1' });
    }
    let mut td = String::with_capacity(tk.len());
    for w in tk.windows(2) {
        let d = w[1] - w[0];
        if !(10..=19).contains(&d) {
            return None;
        }
        td.push(char::from(b'0' + (d - 10) as u8));
    }
    let sha = DATA_SHA_NOTE;
    Some(format!(
        "// SINH TỰ ĐỘNG bởi lõi Rust (`lvhv-dump jpl-days-ts`, core/src/jpl.rs::days_ts_source) — không sửa tay.\n\
         // Bảng mức ngày của phương pháp Jpl suy ra từ core/data/jpl_events.bin ({sha}). docs/03 §17.5–17.6.\n\
         // socDelta: ngày Sóc liên tiếp cách nhau 29 (\"0\") hoặc 30 (\"1\") ngày.\n\
         // tkDelta: ngày D (00:00 đầu ngày đầu tiên ≥ thời điểm tiết khí) liên tiếp cách nhau 10 + chữ số ngày.\n\
         export const JPL_DAYS = {{\n  k0: {k0},\n  socDay0: {s0},\n  i0: {i0},\n  tkDay0: {t0},\n  socDelta:\n    \"{sd}\",\n  tkDelta:\n    \"{td}\",\n}} as const;\n",
        k0 = t.k0,
        s0 = soc[0],
        i0 = t.i0,
        t0 = tk[0],
    ))
}

/// Ghi chú nguồn trong tệp sinh ra (độ dài bảng nhị phân; tổng kiểm ở core/data/jpl_events.meta.txt).
const DATA_SHA_NOTE: &str = "xem core/data/jpl_events.meta.txt";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_t_check_values() {
        // docs/03 §17.4, bảng giá trị kiểm tra (tools/jpl/deltat.py)
        let cases = [
            (1899.75, -2.256566),
            (1900.0, -1.977),
            (1929.0, 24.388584),
            (1950.0, 28.932),
            (1968.0, 37.956),
            (2000.0, 63.808963),
            (2019.0, 69.24),
            (2026.0, 71.431601),
            (2050.0, 82.238467),
            (2100.0, 120.983118),
            (2200.0, 263.006864),
            (2500.0, 1179.196651),
            (2800.0, 2769.53125),
            (3000.0, 4167.03125),
        ];
        for (y, v) in cases {
            let d = delta_t(y).unwrap();
            assert!((d - v).abs() < 1e-6, "{y}: {d} ≠ {v}");
        }
        assert_eq!(delta_t(1894.9), None);
        assert_eq!(delta_t(f64::NAN), None);
        // liên tục tại 2019 và 2800
        for y in [2019.0, 2800.0] {
            let a = delta_t(y - 1e-9).unwrap();
            let b = delta_t(y).unwrap();
            assert!((a - b).abs() < 1e-6, "{y}");
        }
    }

    #[test]
    fn table_decodes_and_covers_range() {
        let t = table().expect("bảng Jpl nhúng hỏng");
        assert!(jd_tt_of(t.soc_cs[0]) >= JD_TT_START && jd_tt_of(t.tk_cs[0]) >= JD_TT_START);
        assert!(*t.soc_cs.last().unwrap() < ((JD_TT_END - 2451545.0) * 8640000.0) as i64);
        // Khoảng cách hợp lý.
        for w in t.soc_ut.windows(2) {
            assert!((29.2..29.9).contains(&(w[1] - w[0])));
        }
        for w in t.tk_ut.windows(2) {
            assert!((14.6..15.8).contains(&(w[1] - w[0])));
        }
    }
}
