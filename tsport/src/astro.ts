// Port of core/src/astro.rs. Order of floating-point operations is kept identical to the Rust
// source (JS evaluates `a + b - c` left to right exactly like Rust; `x += a + b` is x + (a + b)
// in both). No FMA contraction happens in either language.

const PI = Math.PI;
const DR = PI / 180.0;
/** Vietnam calendar time zone (hours) from 1968-01-01 onward (Rust astro::TZ). */
export const TZ = 7.0;
/** Calendar time zone 1929-01-01 → 1967-12-31 (North Vietnam convention before 1968). */
export const TZ_BEFORE_SWITCH = 8.0;
/** Beijing local mean time (116°25′E), UT+7:45:40, before 1929-01-01 (Rust TZ_BEIJING). */
export const TZ_BEIJING = 27940.0 / 3600.0;
/** JDN of 1929-01-01 — first day in UTC+8. */
export const TZ_1929_JDN = 2425613;
/** JDN of 1968-01-01 — first day computed in UTC+7 (docs/03 §16). */
export const TZ_SWITCH_JDN = 2439857;

/** Time zone (hours) of 00:00 at the start of day `jdn` (Rust astro::tz_of_day). */
export function tzOfDay(jdn: number): number {
  return jdn <= TZ_1929_JDN ? TZ_BEIJING : jdn < TZ_SWITCH_JDN ? TZ_BEFORE_SWITCH : TZ;
}

/** JD (UT) of calendar 00:00 at the start of day `jdn`. */
export function localMidnightUt(jdn: number): number {
  return jdn - 0.5 - tzOfDay(jdn) / 24.0;
}

/** Calendar day (JDN) containing instant `jdUt` (Rust astro::local_day_of). */
export function localDayOf(jdUt: number): number {
  const dbj = Math.floor(jdUt + 0.5 + TZ_BEIJING / 24.0);
  if (dbj < TZ_1929_JDN) return dbj;
  const d8 = Math.floor(jdUt + 0.5 + TZ_BEFORE_SWITCH / 24.0);
  return d8 < TZ_SWITCH_JDN ? d8 : Math.floor(jdUt + 0.5 + TZ / 24.0);
}

/** Time zone (hours) used to show instant `jdUt` in calendar time (Rust astro::tz_of_instant). */
export function tzOfInstant(jdUt: number): number {
  if (Math.floor(jdUt + 0.5 + TZ_BEIJING / 24.0) < TZ_1929_JDN) return TZ_BEIJING;
  return Math.floor(jdUt + 0.5 + TZ_BEFORE_SWITCH / 24.0) < TZ_SWITCH_JDN ? TZ_BEFORE_SWITCH : TZ;
}

const sin = Math.sin;
const cos = Math.cos;

// ---------------------------------------------------------------------------
// Hồ Ngọc Đức
// ---------------------------------------------------------------------------

/** JD (UT) of new moon k (k = 0: 1900-01-01). */
export function hndNewMoon(kInt: number): number {
  const k = kInt;
  const t = k / 1236.85;
  const t2 = t * t;
  const t3 = t2 * t;
  let jd1 = 2415020.75933 + 29.53058868 * k + 0.0001178 * t2 - 0.000000155 * t3;
  jd1 += 0.00033 * sin((166.56 + 132.87 * t - 0.009173 * t2) * DR);
  const m = 359.2242 + 29.10535608 * k - 0.0000333 * t2 - 0.00000347 * t3;
  const mpr = 306.0253 + 385.81691806 * k + 0.0107306 * t2 + 0.00001236 * t3;
  const f = 21.2964 + 390.67050646 * k - 0.0016528 * t2 - 0.00000239 * t3;
  let c1 = (0.1734 - 0.000393 * t) * sin(m * DR) + 0.0021 * sin(2.0 * DR * m);
  c1 = c1 - 0.4068 * sin(mpr * DR) + 0.0161 * sin(DR * 2.0 * mpr);
  c1 -= 0.0004 * sin(DR * 3.0 * mpr);
  c1 = c1 + 0.0104 * sin(DR * 2.0 * f) - 0.0051 * sin(DR * (m + mpr));
  c1 = c1 - 0.0074 * sin(DR * (m - mpr)) + 0.0004 * sin(DR * (2.0 * f + m));
  c1 = c1 - 0.0004 * sin(DR * (2.0 * f - m)) - 0.0006 * sin(DR * (2.0 * f + mpr));
  c1 = c1 + 0.0010 * sin(DR * (2.0 * f - mpr)) + 0.0005 * sin(DR * (2.0 * mpr + m));
  const deltat =
    t < -11.0
      ? 0.001 + 0.000839 * t + 0.0002261 * t2 - 0.00000845 * t3 - 0.000000081 * t * t3
      : -0.000278 + 0.000265 * t + 0.000262 * t2;
  return jd1 + c1 - deltat;
}

/** Calendar-time JDN containing new moon k. */
export function hndNewMoonDay(k: number): number {
  return localDayOf(hndNewMoon(k));
}

/** HND geometric sun longitude (radians, [0, 2π)). */
export function hndSunLongitude(jd: number): number {
  const t = (jd - 2451545.0) / 36525.0;
  const t2 = t * t;
  const m = 357.5291 + 35999.0503 * t - 0.0001559 * t2 - 0.00000048 * t * t2;
  const l0 = 280.46645 + 36000.76983 * t + 0.0003032 * t2;
  let dl = (1.9146 - 0.004817 * t - 0.000014 * t2) * sin(DR * m);
  dl += (0.019993 - 0.000101 * t) * sin(DR * 2.0 * m) + 0.00029 * sin(DR * 3.0 * m);
  const l = (l0 + dl) * DR;
  return l - 2.0 * PI * Math.floor(l / (2.0 * PI));
}

/** 30° sector (0..11) of the sun at local 00:00 of day `jdn`. */
export function hndSunSector(jdn: number): number {
  const l = hndSunLongitude(localMidnightUt(jdn));
  const s = Math.floor((l / PI) * 6.0);
  return s < 0 ? 0 : s > 11 ? 11 : s;
}

// ---------------------------------------------------------------------------
// ΔT (Espenak & Meeus 2006), seconds
// ---------------------------------------------------------------------------

// Rust f64::powi(n) with constant n is expanded by LLVM by binary exponentiation
// (ExpandPowI / __powidf2): powi(x,3) = x·(x·x), powi(x,4) = (x²)², powi(x,5) = x·(x²)².
function powi3(x: number): number { const x2 = x * x; return x * x2; }
function powi4(x: number): number { const x2 = x * x; return x2 * x2; }
function powi5(x: number): number { const x2 = x * x; const x4 = x2 * x2; return x * x4; }

export function deltaTSeconds(year: number): number {
  const y = year;
  if (y < 1860.0) {
    const u = (y - 1820.0) / 100.0;
    return -20.0 + 32.0 * u * u;
  } else if (y < 1900.0) {
    const t = y - 1860.0;
    return 7.62 + 0.5737 * t - 0.251754 * t * t + 0.01680668 * powi3(t) - 0.0004473624 * powi4(t) + powi5(t) / 233174.0;
  } else if (y < 1920.0) {
    const t = y - 1900.0;
    return -2.79 + 1.494119 * t - 0.0598939 * t * t + 0.0061966 * powi3(t) - 0.000197 * powi4(t);
  } else if (y < 1941.0) {
    const t = y - 1920.0;
    return 21.2 + 0.84493 * t - 0.0761 * t * t + 0.0020936 * powi3(t);
  } else if (y < 1961.0) {
    const t = y - 1950.0;
    return 29.07 + 0.407 * t - (t * t) / 233.0 + powi3(t) / 2547.0;
  } else if (y < 1986.0) {
    const t = y - 1975.0;
    return 45.45 + 1.067 * t - (t * t) / 260.0 - powi3(t) / 718.0;
  } else if (y < 2005.0) {
    const t = y - 2000.0;
    return (
      63.86 + 0.3345 * t - 0.060374 * t * t +
      0.0017275 * powi3(t) +
      0.000651814 * powi4(t) +
      0.00002373599 * powi5(t)
    );
  } else if (y < 2050.0) {
    const t = y - 2000.0;
    return 62.92 + 0.32217 * t + 0.005589 * t * t;
  } else if (y < 2150.0) {
    const u = (y - 1820.0) / 100.0;
    return -20.0 + 32.0 * u * u - 0.5628 * (2150.0 - y);
  } else {
    const u = (y - 1820.0) / 100.0;
    return -20.0 + 32.0 * u * u;
  }
}

export function jdToYear(jd: number): number {
  return 2000.0 + (jd - 2451544.5) / 365.2425;
}

export function utToTt(jdUt: number): number {
  return jdUt + deltaTSeconds(jdToYear(jdUt)) / 86400.0;
}

export function ttToUt(jde: number): number {
  const ut = jde - deltaTSeconds(jdToYear(jde)) / 86400.0;
  return jde - deltaTSeconds(jdToYear(ut)) / 86400.0;
}

// ---------------------------------------------------------------------------
// Apparent solar longitude — abridged VSOP87 (Meeus App. III). Flat [A,B,C,...] arrays.
// ---------------------------------------------------------------------------

const L0 = [
  175347046.0, 0.0, 0.0,
  3341656.0, 4.6692568, 6283.07585,
  34894.0, 4.6261, 12566.1517,
  3497.0, 2.7441, 5753.3849,
  3418.0, 2.8289, 3.5231,
  3136.0, 3.6277, 77713.7715,
  2676.0, 4.4181, 7860.4194,
  2343.0, 6.1352, 3930.2097,
  1324.0, 0.7425, 11506.7698,
  1273.0, 2.0371, 529.691,
  1199.0, 1.1096, 1577.3435,
  990.0, 5.233, 5884.927,
  902.0, 2.045, 26.298,
  857.0, 3.508, 398.149,
  780.0, 1.179, 5223.694,
  753.0, 2.533, 5507.553,
  505.0, 4.583, 18849.228,
  492.0, 4.205, 775.523,
  357.0, 2.92, 0.067,
  317.0, 5.849, 11790.629,
  284.0, 1.899, 796.298,
  271.0, 0.315, 10977.079,
  243.0, 0.345, 5486.778,
  206.0, 4.806, 2544.314,
  205.0, 1.869, 5573.143,
  202.0, 2.458, 6069.777,
  156.0, 0.833, 213.299,
  132.0, 3.411, 2942.463,
  126.0, 1.083, 20.775,
  115.0, 0.645, 0.98,
  103.0, 0.636, 4694.003,
  102.0, 0.976, 15720.839,
  102.0, 4.267, 7.114,
  99.0, 6.21, 2146.17,
  98.0, 0.68, 155.42,
  86.0, 5.98, 161000.69,
  85.0, 1.3, 6275.96,
  85.0, 3.67, 71430.7,
  80.0, 1.81, 17260.15,
  79.0, 3.04, 12036.46,
  75.0, 1.76, 5088.63,
  74.0, 3.5, 3154.69,
  74.0, 4.68, 801.82,
  70.0, 0.83, 9437.76,
  62.0, 3.98, 8827.39,
  61.0, 1.82, 7084.9,
  57.0, 2.78, 6286.6,
  56.0, 4.39, 14143.5,
  56.0, 3.47, 6279.55,
  52.0, 0.19, 12139.55,
  52.0, 1.33, 1748.02,
  51.0, 0.28, 5856.48,
  49.0, 0.49, 1194.45,
  41.0, 5.37, 8429.24,
  41.0, 2.4, 19651.05,
  39.0, 6.17, 10447.39,
  37.0, 6.04, 10213.29,
  37.0, 2.57, 1059.38,
  36.0, 1.71, 2352.87,
  36.0, 1.78, 6812.77,
  33.0, 0.59, 17789.85,
  30.0, 0.44, 83996.85,
  30.0, 2.74, 1349.87,
  25.0, 3.16, 4690.48,
];
const L1 = [
  628331966747.0, 0.0, 0.0,
  206059.0, 2.678235, 6283.07585,
  4303.0, 2.6351, 12566.1517,
  425.0, 1.59, 3.523,
  119.0, 5.796, 26.298,
  109.0, 2.966, 1577.344,
  93.0, 2.59, 18849.23,
  72.0, 1.14, 529.69,
  68.0, 1.87, 398.15,
  67.0, 4.41, 5507.55,
  59.0, 2.89, 5223.69,
  56.0, 2.17, 155.42,
  45.0, 0.4, 796.3,
  36.0, 0.47, 775.52,
  29.0, 2.65, 7.11,
  21.0, 5.34, 0.98,
  19.0, 1.85, 5486.78,
  19.0, 4.97, 213.3,
  17.0, 2.99, 6275.96,
  16.0, 0.03, 2544.31,
  16.0, 1.43, 2146.17,
  15.0, 1.21, 10977.08,
  12.0, 2.83, 1748.02,
  12.0, 3.26, 5088.63,
  12.0, 5.27, 1194.45,
  12.0, 2.08, 4694.0,
  11.0, 0.77, 553.57,
  10.0, 1.3, 6286.6,
  10.0, 4.24, 1349.87,
  9.0, 2.7, 242.73,
  9.0, 5.64, 951.72,
  8.0, 5.3, 2352.87,
  6.0, 2.65, 9437.76,
  6.0, 4.67, 4690.48,
];
const L2 = [
  52919.0, 0.0, 0.0,
  8720.0, 1.0721, 6283.0758,
  309.0, 0.867, 12566.152,
  27.0, 0.05, 3.52,
  16.0, 5.19, 26.3,
  16.0, 3.68, 155.42,
  10.0, 0.76, 18849.23,
  9.0, 2.06, 77713.77,
  7.0, 0.83, 775.52,
  5.0, 4.66, 1577.34,
  4.0, 1.03, 7.11,
  4.0, 3.44, 5573.14,
  3.0, 5.14, 796.3,
  3.0, 6.05, 5507.55,
  3.0, 1.19, 242.73,
  3.0, 6.12, 529.69,
  3.0, 0.31, 398.15,
  3.0, 2.28, 553.57,
  2.0, 4.38, 5223.69,
  2.0, 3.75, 0.98,
];
const L3 = [
  289.0, 5.844, 6283.076,
  35.0, 0.0, 0.0,
  17.0, 5.49, 12566.15,
  3.0, 5.2, 155.42,
  1.0, 4.72, 3.52,
  1.0, 5.3, 18849.23,
  1.0, 5.97, 242.73,
];
const L4 = [
  114.0, 3.142, 0.0,
  8.0, 4.13, 6283.08,
  1.0, 3.84, 12566.15,
];
const L5 = [1.0, 3.14, 0.0];

/** Σ A·cos(B + C·τ), summed left to right like Rust `Iterator::sum`. */
function series(terms: readonly number[], tau: number): number {
  let s = 0.0;
  for (let i = 0; i < terms.length; i += 3) {
    s += terms[i] * cos(terms[i + 1] + terms[i + 2] * tau);
  }
  return s;
}

/** Rust `x % 360.0` (fmod, same as JS %), then shift negatives. */
function normDeg(x: number): number {
  const r = x % 360.0;
  return r < 0.0 ? r + 360.0 : r;
}

/** Apparent ecliptic longitude of the sun (degrees, [0,360)) at JDE (TT). */
export function apparentSunLongitude(jde: number): number {
  const tau = (jde - 2451545.0) / 365250.0;
  const l =
    (series(L0, tau) +
      tau *
        (series(L1, tau) +
          tau *
            (series(L2, tau) +
              tau * (series(L3, tau) + tau * (series(L4, tau) + tau * series(L5, tau)))))) /
    1e8;
  const t = tau * 10.0;
  let lon = normDeg(l / DR + 180.0);
  lon -= 0.09033 / 3600.0;
  const omega = (125.04452 - 1934.136261 * t) * DR;
  const lsun = (280.4665 + 36000.7698 * t) * DR;
  const lmoon = (218.3165 + 481267.8813 * t) * DR;
  const dpsi =
    -17.2 * sin(omega) - 1.32 * sin(2.0 * lsun) - 0.23 * sin(2.0 * lmoon) + 0.21 * sin(2.0 * omega);
  const m = (357.52911 + 35999.05029 * t - 0.0001537 * t * t) * DR;
  const e = 0.016708634 - 0.000042037 * t;
  const c = ((1.914602 - 0.004817 * t) * sin(m) + 0.019993 * sin(2.0 * m)) * DR;
  const r = (1.000001018 * (1.0 - e * e)) / (1.0 + e * cos(m + c));
  const aberr = -20.4898 / r;
  return normDeg(lon + (dpsi + aberr) / 3600.0);
}

export function apparentSunLongitudeUt(jdUt: number): number {
  return apparentSunLongitude(utToTt(jdUt));
}

/** Rust f64::round (half away from zero). */
function rround(x: number): number {
  const t = Math.trunc(x);
  return Math.abs(x - t) >= 0.5 ? t + Math.sign(x) : t;
}

/** Newton solve for apparent longitude = target (deg) near guessUt (JD UT). */
export function solveSunLongitude(target: number, guessUt: number): number {
  let t = guessUt;
  for (let it = 0; it < 30; it++) {
    const l = apparentSunLongitudeUt(t);
    let diff = target - l;
    diff -= 360.0 * rround(diff / 360.0);
    const h = 0.01;
    let dl = apparentSunLongitudeUt(t + h) - l;
    dl -= 360.0 * rround(dl / 360.0);
    const rate = dl / h;
    const step = diff / rate;
    t += step;
    if (Math.abs(step) < 1e-8) break;
  }
  return t;
}

// ---------------------------------------------------------------------------
// Full Meeus ch.49 new moon (diagnostics only)
// ---------------------------------------------------------------------------

const MEEUS_A: readonly number[] = [
  0.000325, 299.77, 0.107408,
  0.000165, 251.88, 0.016321,
  0.000164, 251.83, 26.651886,
  0.000126, 349.42, 36.412478,
  0.00011, 84.66, 18.206239,
  0.000062, 141.74, 53.303771,
  0.00006, 207.14, 2.453732,
  0.000056, 154.84, 7.30686,
  0.000047, 34.52, 27.261239,
  0.000042, 207.19, 0.121824,
  0.00004, 291.34, 1.844379,
  0.000037, 161.72, 24.198154,
  0.000035, 239.56, 25.513099,
  0.000023, 331.55, 3.592518,
];

export function meeusNewMoonJde(kInt: number): number {
  const k = kInt;
  const t = k / 1236.85;
  const t2 = t * t;
  const t3 = t2 * t;
  const t4 = t3 * t;
  const jde = 2451550.09766 + 29.530588861 * k + 0.00015437 * t2 - 0.00000015 * t3 + 0.00000000073 * t4;
  const e = 1.0 - 0.002516 * t - 0.0000074 * t2;
  const m = (2.5534 + 29.1053567 * k - 0.0000014 * t2 - 0.00000011 * t3) * DR;
  const mp = (201.5643 + 385.81693528 * k + 0.0107582 * t2 + 0.00001238 * t3 - 0.000000058 * t4) * DR;
  const f = (160.7108 + 390.67050284 * k - 0.0016118 * t2 - 0.00000227 * t3 + 0.000000011 * t4) * DR;
  const om = (124.7746 - 1.56375588 * k + 0.0020672 * t2 + 0.00000215 * t3) * DR;
  const s =
    -0.4072 * sin(mp) +
    0.17241 * e * sin(m) +
    0.01608 * sin(2.0 * mp) +
    0.01039 * sin(2.0 * f) +
    0.00739 * e * sin(mp - m) -
    0.00514 * e * sin(mp + m) +
    0.00208 * e * e * sin(2.0 * m) -
    0.00111 * sin(mp - 2.0 * f) -
    0.00057 * sin(mp + 2.0 * f) +
    0.00056 * e * sin(2.0 * mp + m) -
    0.00042 * sin(3.0 * mp) +
    0.00042 * e * sin(m + 2.0 * f) +
    0.00038 * e * sin(m - 2.0 * f) -
    0.00024 * e * sin(2.0 * mp - m) -
    0.00017 * sin(om) -
    0.00007 * sin(mp + 2.0 * m) +
    0.00004 * sin(2.0 * mp - 2.0 * f) +
    0.00004 * sin(3.0 * m) +
    0.00003 * sin(mp + m - 2.0 * f) +
    0.00003 * sin(2.0 * mp + 2.0 * f) -
    0.00003 * sin(mp + m + 2.0 * f) +
    0.00003 * sin(mp - m + 2.0 * f) -
    0.00002 * sin(mp - m - 2.0 * f) -
    0.00002 * sin(3.0 * mp + m) +
    0.00002 * sin(4.0 * mp);
  let add = 0.0;
  for (let i = 0; i < MEEUS_A.length; i += 3) {
    let arg = MEEUS_A[i + 1] + MEEUS_A[i + 2] * k;
    if (i === 0) arg -= 0.009173 * t2;
    add += MEEUS_A[i] * sin(arg * DR);
  }
  return jde + s + add;
}

export function preciseNewMoonUt(kHnd: number): number {
  return ttToUt(meeusNewMoonJde(kHnd - 1237));
}
