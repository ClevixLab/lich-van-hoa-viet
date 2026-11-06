//! Thiên văn cho lịch âm.
//!
//! Hai nhóm hàm:
//! * `hnd_*`: đúng theo thuật toán Hồ Ngọc Đức ("Thuật toán tính âm lịch") — Sóc theo
//!   công thức Meeus rút gọn có hiệu chỉnh ΔT, kinh độ Mặt Trời "thật" (hình học) theo Meeus
//!   bản độ chính xác thấp. Đây là phương pháp tham chiếu của lõi để tính ngày/tháng âm
//!   (hợp đồng kiến trúc §2).
//! * `apparent_sun_longitude`, `delta_t_seconds`, `meeus_new_moon_jde`: mô hình chính xác hơn
//!   (VSOP87 rút gọn theo Phụ lục III của Meeus + chương động + quang sai; ΔT Espenak–Meeus;
//!   Sóc theo Meeus ch.49 đầy đủ). Dùng cho 24 tiết khí và cho chẩn đoán đối chiếu nội bộ.

use std::f64::consts::PI;

const DR: f64 = PI / 180.0;
/// Múi giờ lịch Việt Nam hiện hành (giờ) — áp dụng cho mọi thời điểm từ 1968-01-01 00:00
/// (giờ UTC+8 cũ) trở đi, tức toàn bộ 1968–3000. Trước đó dùng `TZ_BEFORE_SWITCH` (xem
/// `local_day_of`, docs/03 §16).
pub const TZ: f64 = 7.0;
/// Múi giờ dùng tính âm lịch trước điểm chuyển (quy ước lịch miền Bắc trước 1968).
pub const TZ_BEFORE_SWITCH: f64 = 8.0;
/// JDN của 1968-01-01 — ngày đầu tiên tính theo UTC+7 (docs/03 §16).
pub const TZ_SWITCH_JDN: i64 = 2439857;

/// Múi giờ (giờ) của **ngày** `jdn`: dùng cho thời điểm 00:00 đầu ngày (Trung khí, tiết khí).
pub fn tz_of_day(jdn: i64) -> f64 {
    if jdn < TZ_SWITCH_JDN {
        TZ_BEFORE_SWITCH
    } else {
        TZ
    }
}

/// JD (UT) của 00:00 giờ lịch đầu ngày `jdn` (UTC+8 trước 1968-01-01, UTC+7 từ đó).
pub fn local_midnight_ut(jdn: i64) -> f64 {
    jdn as f64 - 0.5 - tz_of_day(jdn) / 24.0
}

/// Ngày lịch (JDN) chứa thời điểm `jd_ut` (UT). Quy tắc chuyển (docs/03 §16): thời điểm trước
/// 1968-01-01 00:00 UTC+8 (= 1967-12-31 16:00 UT) đổi sang ngày theo UTC+8; từ đó theo UTC+7.
/// Hàm không giảm theo `jd_ut` (thời điểm trong một giờ ngay sau điểm chuyển vẫn thuộc ngày
/// 1967-12-31 — ngày này dài 25 giờ), nên Sóc liên tiếp luôn cho ngày tăng dần.
/// Với mọi thời điểm từ 1968 trở đi, biểu thức giống hệt từng bit bản UTC+7 cố định trước đây.
pub fn local_day_of(jd_ut: f64) -> i64 {
    let d8 = (jd_ut + 0.5 + TZ_BEFORE_SWITCH / 24.0).floor() as i64;
    if d8 < TZ_SWITCH_JDN {
        d8
    } else {
        (jd_ut + 0.5 + TZ / 24.0).floor() as i64
    }
}

/// Múi giờ (giờ) dùng để đổi thời điểm `jd_ut` sang giờ lịch (cùng quy tắc `local_day_of`).
pub fn tz_of_instant(jd_ut: f64) -> f64 {
    if ((jd_ut + 0.5 + TZ_BEFORE_SWITCH / 24.0).floor() as i64) < TZ_SWITCH_JDN {
        TZ_BEFORE_SWITCH
    } else {
        TZ
    }
}

// ---------------------------------------------------------------------------
// Thuật toán Hồ Ngọc Đức
// ---------------------------------------------------------------------------

/// Thời điểm Sóc thứ k (k = 0 ứng với Sóc ngày 1900-01-01), trả về JD theo UT.
/// Công thức của Hồ Ngọc Đức (dựa trên Meeus), gồm hiệu chỉnh ΔT riêng của tác giả.
pub fn hnd_new_moon(k: i64) -> f64 {
    let k = k as f64;
    let t = k / 1236.85;
    let t2 = t * t;
    let t3 = t2 * t;
    let mut jd1 = 2415020.75933 + 29.53058868 * k + 0.0001178 * t2 - 0.000000155 * t3;
    jd1 += 0.00033 * ((166.56 + 132.87 * t - 0.009173 * t2) * DR).sin();
    let m = 359.2242 + 29.10535608 * k - 0.0000333 * t2 - 0.00000347 * t3;
    let mpr = 306.0253 + 385.81691806 * k + 0.0107306 * t2 + 0.00001236 * t3;
    let f = 21.2964 + 390.67050646 * k - 0.0016528 * t2 - 0.00000239 * t3;
    let mut c1 = (0.1734 - 0.000393 * t) * (m * DR).sin() + 0.0021 * (2.0 * DR * m).sin();
    c1 = c1 - 0.4068 * (mpr * DR).sin() + 0.0161 * (DR * 2.0 * mpr).sin();
    c1 -= 0.0004 * (DR * 3.0 * mpr).sin();
    c1 = c1 + 0.0104 * (DR * 2.0 * f).sin() - 0.0051 * (DR * (m + mpr)).sin();
    c1 = c1 - 0.0074 * (DR * (m - mpr)).sin() + 0.0004 * (DR * (2.0 * f + m)).sin();
    c1 = c1 - 0.0004 * (DR * (2.0 * f - m)).sin() - 0.0006 * (DR * (2.0 * f + mpr)).sin();
    c1 = c1 + 0.0010 * (DR * (2.0 * f - mpr)).sin() + 0.0005 * (DR * (2.0 * mpr + m)).sin();
    let deltat = if t < -11.0 {
        0.001 + 0.000839 * t + 0.0002261 * t2 - 0.00000845 * t3 - 0.000000081 * t * t3
    } else {
        -0.000278 + 0.000265 * t + 0.000262 * t2
    };
    jd1 + c1 - deltat
}

/// Ngày (JDN) theo giờ địa phương chứa Sóc thứ k.
pub fn hnd_new_moon_day(k: i64) -> i64 {
    local_day_of(hnd_new_moon(k))
}

/// Kinh độ Mặt Trời (radian, [0, 2π)) theo Hồ Ngọc Đức: kinh độ thật, không chương động,
/// không quang sai; thời điểm vào là JD (không đổi UT→TT).
pub fn hnd_sun_longitude(jd: f64) -> f64 {
    let t = (jd - 2451545.0) / 36525.0;
    let t2 = t * t;
    let m = 357.52910 + 35999.05030 * t - 0.0001559 * t2 - 0.00000048 * t * t2;
    let l0 = 280.46645 + 36000.76983 * t + 0.0003032 * t2;
    let mut dl = (1.914600 - 0.004817 * t - 0.000014 * t2) * (DR * m).sin();
    dl += (0.019993 - 0.000101 * t) * (DR * 2.0 * m).sin() + 0.000290 * (DR * 3.0 * m).sin();
    let l = (l0 + dl) * DR;
    l - 2.0 * PI * (l / (2.0 * PI)).floor()
}

/// Cung 30° (0..11) của Mặt Trời lúc 00:00 giờ địa phương của ngày `jdn` — dùng để xét Trung khí.
pub fn hnd_sun_sector(jdn: i64) -> i32 {
    let l = hnd_sun_longitude(local_midnight_ut(jdn));
    ((l / PI * 6.0).floor() as i32).clamp(0, 11)
}

// ---------------------------------------------------------------------------
// ΔT (TT − UT), giây — đa thức Espenak & Meeus (NASA, 2006)
// ---------------------------------------------------------------------------

/// ΔT tính theo năm thập phân. Sau 2150 là ngoại suy parabol −20 + 32u²; độ bất định rất lớn
/// (hàng chục phút vào năm 3000).
pub fn delta_t_seconds(year: f64) -> f64 {
    let y = year;
    if y < 1860.0 {
        // Ngoài phạm vi dùng của dự án; parabol dài hạn
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    } else if y < 1900.0 {
        // Espenak–Meeus 1860–1900 (chỉ dùng ở đầu miền: Sóc/tiết khí cuối 1899).
        let t = y - 1860.0;
        7.62 + 0.5737 * t - 0.251754 * t * t + 0.01680668 * t.powi(3) - 0.0004473624 * t.powi(4)
            + t.powi(5) / 233174.0
    } else if y < 1920.0 {
        let t = y - 1900.0;
        -2.79 + 1.494119 * t - 0.0598939 * t * t + 0.0061966 * t.powi(3) - 0.000197 * t.powi(4)
    } else if y < 1941.0 {
        let t = y - 1920.0;
        21.20 + 0.84493 * t - 0.076100 * t * t + 0.0020936 * t.powi(3)
    } else if y < 1961.0 {
        let t = y - 1950.0;
        29.07 + 0.407 * t - t * t / 233.0 + t.powi(3) / 2547.0
    } else if y < 1986.0 {
        let t = y - 1975.0;
        45.45 + 1.067 * t - t * t / 260.0 - t.powi(3) / 718.0
    } else if y < 2005.0 {
        let t = y - 2000.0;
        63.86 + 0.3345 * t - 0.060374 * t * t
            + 0.0017275 * t.powi(3)
            + 0.000651814 * t.powi(4)
            + 0.00002373599 * t.powi(5)
    } else if y < 2050.0 {
        let t = y - 2000.0;
        62.92 + 0.32217 * t + 0.005589 * t * t
    } else if y < 2150.0 {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u - 0.5628 * (2150.0 - y)
    } else {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    }
}

/// Năm thập phân gần đúng của một JD.
pub fn jd_to_year(jd: f64) -> f64 {
    2000.0 + (jd - 2451544.5) / 365.2425
}

/// Đổi JD (UT) sang JDE (TT).
pub fn ut_to_tt(jd_ut: f64) -> f64 {
    jd_ut + delta_t_seconds(jd_to_year(jd_ut)) / 86400.0
}

/// Đổi JDE (TT) sang JD (UT) (lặp một lần là đủ vì ΔT biến thiên chậm).
pub fn tt_to_ut(jde: f64) -> f64 {
    let ut = jde - delta_t_seconds(jd_to_year(jde)) / 86400.0;
    jde - delta_t_seconds(jd_to_year(ut)) / 86400.0
}

// ---------------------------------------------------------------------------
// Kinh độ biểu kiến của Mặt Trời — VSOP87 rút gọn (Meeus, Phụ lục III, Trái Đất)
// ---------------------------------------------------------------------------

// Hệ số giữ nguyên như bảng công bố (3.142, 3.14 không phải π).
// (A, B, C): A·cos(B + C·τ), τ = thiên niên kỷ Julian từ J2000 (TT); đơn vị 1e-8 rad.
const L0: [(f64, f64, f64); 64] = [
    (175347046.0, 0.0, 0.0),
    (3341656.0, 4.6692568, 6283.0758500),
    (34894.0, 4.62610, 12566.15170),
    (3497.0, 2.7441, 5753.3849),
    (3418.0, 2.8289, 3.5231),
    (3136.0, 3.6277, 77713.7715),
    (2676.0, 4.4181, 7860.4194),
    (2343.0, 6.1352, 3930.2097),
    (1324.0, 0.7425, 11506.7698),
    (1273.0, 2.0371, 529.6910),
    (1199.0, 1.1096, 1577.3435),
    (990.0, 5.233, 5884.927),
    (902.0, 2.045, 26.298),
    (857.0, 3.508, 398.149),
    (780.0, 1.179, 5223.694),
    (753.0, 2.533, 5507.553),
    (505.0, 4.583, 18849.228),
    (492.0, 4.205, 775.523),
    (357.0, 2.920, 0.067),
    (317.0, 5.849, 11790.629),
    (284.0, 1.899, 796.298),
    (271.0, 0.315, 10977.079),
    (243.0, 0.345, 5486.778),
    (206.0, 4.806, 2544.314),
    (205.0, 1.869, 5573.143),
    (202.0, 2.458, 6069.777),
    (156.0, 0.833, 213.299),
    (132.0, 3.411, 2942.463),
    (126.0, 1.083, 20.775),
    (115.0, 0.645, 0.980),
    (103.0, 0.636, 4694.003),
    (102.0, 0.976, 15720.839),
    (102.0, 4.267, 7.114),
    (99.0, 6.21, 2146.17),
    (98.0, 0.68, 155.42),
    (86.0, 5.98, 161000.69),
    (85.0, 1.30, 6275.96),
    (85.0, 3.67, 71430.70),
    (80.0, 1.81, 17260.15),
    (79.0, 3.04, 12036.46),
    (75.0, 1.76, 5088.63),
    (74.0, 3.50, 3154.69),
    (74.0, 4.68, 801.82),
    (70.0, 0.83, 9437.76),
    (62.0, 3.98, 8827.39),
    (61.0, 1.82, 7084.90),
    (57.0, 2.78, 6286.60),
    (56.0, 4.39, 14143.50),
    (56.0, 3.47, 6279.55),
    (52.0, 0.19, 12139.55),
    (52.0, 1.33, 1748.02),
    (51.0, 0.28, 5856.48),
    (49.0, 0.49, 1194.45),
    (41.0, 5.37, 8429.24),
    (41.0, 2.40, 19651.05),
    (39.0, 6.17, 10447.39),
    (37.0, 6.04, 10213.29),
    (37.0, 2.57, 1059.38),
    (36.0, 1.71, 2352.87),
    (36.0, 1.78, 6812.77),
    (33.0, 0.59, 17789.85),
    (30.0, 0.44, 83996.85),
    (30.0, 2.74, 1349.87),
    (25.0, 3.16, 4690.48),
];
const L1: [(f64, f64, f64); 34] = [
    (628331966747.0, 0.0, 0.0),
    (206059.0, 2.678235, 6283.07585),
    (4303.0, 2.6351, 12566.1517),
    (425.0, 1.590, 3.523),
    (119.0, 5.796, 26.298),
    (109.0, 2.966, 1577.344),
    (93.0, 2.59, 18849.23),
    (72.0, 1.14, 529.69),
    (68.0, 1.87, 398.15),
    (67.0, 4.41, 5507.55),
    (59.0, 2.89, 5223.69),
    (56.0, 2.17, 155.42),
    (45.0, 0.40, 796.30),
    (36.0, 0.47, 775.52),
    (29.0, 2.65, 7.11),
    (21.0, 5.34, 0.98),
    (19.0, 1.85, 5486.78),
    (19.0, 4.97, 213.30),
    (17.0, 2.99, 6275.96),
    (16.0, 0.03, 2544.31),
    (16.0, 1.43, 2146.17),
    (15.0, 1.21, 10977.08),
    (12.0, 2.83, 1748.02),
    (12.0, 3.26, 5088.63),
    (12.0, 5.27, 1194.45),
    (12.0, 2.08, 4694.00),
    (11.0, 0.77, 553.57),
    (10.0, 1.30, 6286.60),
    (10.0, 4.24, 1349.87),
    (9.0, 2.70, 242.73),
    (9.0, 5.64, 951.72),
    (8.0, 5.30, 2352.87),
    (6.0, 2.65, 9437.76),
    (6.0, 4.67, 4690.48),
];
const L2: [(f64, f64, f64); 20] = [
    (52919.0, 0.0, 0.0),
    (8720.0, 1.0721, 6283.0758),
    (309.0, 0.867, 12566.152),
    (27.0, 0.05, 3.52),
    (16.0, 5.19, 26.30),
    (16.0, 3.68, 155.42),
    (10.0, 0.76, 18849.23),
    (9.0, 2.06, 77713.77),
    (7.0, 0.83, 775.52),
    (5.0, 4.66, 1577.34),
    (4.0, 1.03, 7.11),
    (4.0, 3.44, 5573.14),
    (3.0, 5.14, 796.30),
    (3.0, 6.05, 5507.55),
    (3.0, 1.19, 242.73),
    (3.0, 6.12, 529.69),
    (3.0, 0.31, 398.15),
    (3.0, 2.28, 553.57),
    (2.0, 4.38, 5223.69),
    (2.0, 3.75, 0.98),
];
const L3: [(f64, f64, f64); 7] = [
    (289.0, 5.844, 6283.076),
    (35.0, 0.0, 0.0),
    (17.0, 5.49, 12566.15),
    (3.0, 5.20, 155.42),
    (1.0, 4.72, 3.52),
    (1.0, 5.30, 18849.23),
    (1.0, 5.97, 242.73),
];
#[allow(clippy::approx_constant)]
const L4: [(f64, f64, f64); 3] = [
    (114.0, 3.142, 0.0),
    (8.0, 4.13, 6283.08),
    (1.0, 3.84, 12566.15),
];
#[allow(clippy::approx_constant)]
const L5: [(f64, f64, f64); 1] = [(1.0, 3.14, 0.0)];

fn series(terms: &[(f64, f64, f64)], tau: f64) -> f64 {
    terms.iter().map(|&(a, b, c)| a * (b + c * tau).cos()).sum()
}

fn norm_deg(x: f64) -> f64 {
    let r = x % 360.0;
    if r < 0.0 {
        r + 360.0
    } else {
        r
    }
}

/// Kinh độ hoàng đạo biểu kiến của Mặt Trời (độ, [0,360)) tại thời điểm JDE (TT).
/// VSOP87 rút gọn + hiệu chỉnh FK5 + chương động (4 số hạng) + quang sai.
/// Sai số ước lượng ≲ 1–2″ trong 2000–3000 (≈ 0,5 phút thời gian), chưa kể sai số ΔT.
pub fn apparent_sun_longitude(jde: f64) -> f64 {
    let tau = (jde - 2451545.0) / 365250.0;
    let l = (series(&L0, tau)
        + tau
            * (series(&L1, tau)
                + tau
                    * (series(&L2, tau)
                        + tau
                            * (series(&L3, tau)
                                + tau * (series(&L4, tau) + tau * series(&L5, tau))))))
        / 1e8;
    let t = tau * 10.0; // thế kỷ Julian
                        // kinh độ hình học của Mặt Trời (độ)
    let mut lon = norm_deg(l / DR + 180.0);
    // hiệu chỉnh về hệ FK5
    lon -= 0.09033 / 3600.0;
    // chương động theo kinh độ (Meeus ch.22, độ chính xác 0,5″)
    let omega = (125.04452 - 1934.136261 * t) * DR;
    let lsun = (280.4665 + 36000.7698 * t) * DR;
    let lmoon = (218.3165 + 481267.8813 * t) * DR;
    let dpsi = -17.20 * omega.sin() - 1.32 * (2.0 * lsun).sin() - 0.23 * (2.0 * lmoon).sin()
        + 0.21 * (2.0 * omega).sin();
    // khoảng cách R (đvtv) để tính quang sai
    let m = (357.52911 + 35999.05029 * t - 0.0001537 * t * t) * DR;
    let e = 0.016708634 - 0.000042037 * t;
    let c = ((1.914602 - 0.004817 * t) * m.sin() + 0.019993 * (2.0 * m).sin()) * DR;
    let r = 1.000001018 * (1.0 - e * e) / (1.0 + e * (m + c).cos());
    let aberr = -20.4898 / r;
    norm_deg(lon + (dpsi + aberr) / 3600.0)
}

/// Kinh độ biểu kiến của Mặt Trời tại thời điểm JD theo UT.
pub fn apparent_sun_longitude_ut(jd_ut: f64) -> f64 {
    apparent_sun_longitude(ut_to_tt(jd_ut))
}

/// Tìm thời điểm (JD UT) Mặt Trời có kinh độ biểu kiến `target` (độ), gần `guess_ut`.
/// Newton với đạo hàm số; hội tụ đến ~1e-8 ngày.
pub fn solve_sun_longitude(target: f64, guess_ut: f64) -> f64 {
    let mut t = guess_ut;
    for _ in 0..30 {
        let l = apparent_sun_longitude_ut(t);
        let mut diff = target - l;
        diff -= 360.0 * (diff / 360.0).round();
        let h = 0.01;
        let rate = {
            let mut dl = apparent_sun_longitude_ut(t + h) - l;
            dl -= 360.0 * (dl / 360.0).round();
            dl / h
        };
        let step = diff / rate;
        t += step;
        if step.abs() < 1e-8 {
            break;
        }
    }
    t
}

// ---------------------------------------------------------------------------
// Sóc theo Meeus ch.49 đầy đủ (dùng cho chẩn đoán đối chiếu)
// ---------------------------------------------------------------------------

/// JDE (TT) của trăng mới thứ k theo Meeus (k = 0 ứng với 2000-01-06).
pub fn meeus_new_moon_jde(k: i64) -> f64 {
    let k = k as f64;
    let t = k / 1236.85;
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let jde =
        2451550.09766 + 29.530588861 * k + 0.00015437 * t2 - 0.000000150 * t3 + 0.00000000073 * t4;
    let e = 1.0 - 0.002516 * t - 0.0000074 * t2;
    let m = (2.5534 + 29.10535670 * k - 0.0000014 * t2 - 0.00000011 * t3) * DR;
    let mp =
        (201.5643 + 385.81693528 * k + 0.0107582 * t2 + 0.00001238 * t3 - 0.000000058 * t4) * DR;
    let f =
        (160.7108 + 390.67050284 * k - 0.0016118 * t2 - 0.00000227 * t3 + 0.000000011 * t4) * DR;
    let om = (124.7746 - 1.56375588 * k + 0.0020672 * t2 + 0.00000215 * t3) * DR;
    let s = -0.40720 * mp.sin()
        + 0.17241 * e * m.sin()
        + 0.01608 * (2.0 * mp).sin()
        + 0.01039 * (2.0 * f).sin()
        + 0.00739 * e * (mp - m).sin()
        - 0.00514 * e * (mp + m).sin()
        + 0.00208 * e * e * (2.0 * m).sin()
        - 0.00111 * (mp - 2.0 * f).sin()
        - 0.00057 * (mp + 2.0 * f).sin()
        + 0.00056 * e * (2.0 * mp + m).sin()
        - 0.00042 * (3.0 * mp).sin()
        + 0.00042 * e * (m + 2.0 * f).sin()
        + 0.00038 * e * (m - 2.0 * f).sin()
        - 0.00024 * e * (2.0 * mp - m).sin()
        - 0.00017 * om.sin()
        - 0.00007 * (mp + 2.0 * m).sin()
        + 0.00004 * (2.0 * mp - 2.0 * f).sin()
        + 0.00004 * (3.0 * m).sin()
        + 0.00003 * (mp + m - 2.0 * f).sin()
        + 0.00003 * (2.0 * mp + 2.0 * f).sin()
        - 0.00003 * (mp + m + 2.0 * f).sin()
        + 0.00003 * (mp - m + 2.0 * f).sin()
        - 0.00002 * (mp - m - 2.0 * f).sin()
        - 0.00002 * (3.0 * mp + m).sin()
        + 0.00002 * (4.0 * mp).sin();
    let a: [(f64, f64, f64); 14] = [
        (0.000325, 299.77, 0.107408),
        (0.000165, 251.88, 0.016321),
        (0.000164, 251.83, 26.651886),
        (0.000126, 349.42, 36.412478),
        (0.000110, 84.66, 18.206239),
        (0.000062, 141.74, 53.303771),
        (0.000060, 207.14, 2.453732),
        (0.000056, 154.84, 7.306860),
        (0.000047, 34.52, 27.261239),
        (0.000042, 207.19, 0.121824),
        (0.000040, 291.34, 1.844379),
        (0.000037, 161.72, 24.198154),
        (0.000035, 239.56, 25.513099),
        (0.000023, 331.55, 3.592518),
    ];
    let mut add = 0.0;
    for (i, &(coef, a0, a1)) in a.iter().enumerate() {
        let mut arg = a0 + a1 * k;
        if i == 0 {
            arg -= 0.009173 * t2;
        }
        add += coef * (arg * DR).sin();
    }
    jde + s + add
}

/// Sóc theo Meeus ch.49 + ΔT Espenak–Meeus, chỉ số k theo quy ước Hồ Ngọc Đức (k=0: 1900-01-01).
pub fn precise_new_moon_ut(k_hnd: i64) -> f64 {
    tt_to_ut(meeus_new_moon_jde(k_hnd - 1237))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meeus_example_49a() {
        // Meeus, ví dụ 49.a: trăng mới 1977-02-18, k = −283, JDE = 2443192.65118
        let jde = meeus_new_moon_jde(-283);
        assert!((jde - 2443192.65118).abs() < 2e-5, "{jde}");
    }

    #[test]
    fn hnd_k_offset_matches_meeus() {
        for k in [1237i64, 1300, 2000, 13000] {
            let a = hnd_new_moon(k);
            let b = precise_new_moon_ut(k);
            assert!((a - b).abs() < 0.1, "k={k}: {a} vs {b}");
        }
    }

    #[test]
    fn meeus_example_25b() {
        // Meeus ví dụ 25.b: 1992-10-13 0h TD, λ biểu kiến = 199°54′21.818″ (VSOP87 đầy đủ)
        let l = apparent_sun_longitude(2448908.5);
        let want = 199.0 + 54.0 / 60.0 + 21.818 / 3600.0;
        assert!(
            (l - want).abs() * 3600.0 < 2.0,
            "lệch {}″",
            (l - want) * 3600.0
        );
    }

    #[test]
    fn equinox_solstice_times() {
        // Thời điểm (UT) công bố rộng rãi, sai số cho phép 2 phút.
        let cases = [
            (0.0, 2000, 3, 20, 7.0 + 35.0 / 60.0),
            (90.0, 2000, 6, 21, 1.0 + 48.0 / 60.0),
            (270.0, 2000, 12, 21, 13.0 + 37.0 / 60.0),
            (0.0, 2024, 3, 20, 3.0 + 6.0 / 60.0),
        ];
        for (lon, y, m, d, h) in cases {
            let guess = crate::jd::jdn_from_date(y, m, d) as f64 - 0.5;
            let t = solve_sun_longitude(lon, guess);
            let want = guess + h / 24.0;
            assert!(
                (t - want).abs() * 1440.0 < 2.0,
                "{y}-{m}-{d}: lệch {} phút",
                (t - want) * 1440.0
            );
        }
    }
}
