// Port of core/src/lunar.rs — Vietnamese lunar calendar (UTC+7). Two selectable methods, as in
// Rust: "hnd" (Hồ Ngọc Đức, kept for comparison) and "precise" (default since A3) (Meeus ch.49 full + ΔT Espenak–Meeus; Trung khí
// from the apparent solar longitude used for the displayed solar terms).

import { apparentSunLongitudeUt, hndNewMoon, hndSunSector, localDayOf, localMidnightUt, preciseNewMoonUt } from "./astro.ts";
import {
  BIRTH_MIN_YEAR, INTERNAL_MAX_YY, INTERNAL_MIN_YY, MAX_LUNAR_YEAR, MAX_YEAR, MIN_LUNAR_YEAR, MIN_YEAR,
  checkJdn, computation, invalidArgument, invalidDate, jdnInRange, outOfRange,
} from "./error.ts";
import { dateFromJdn, isValidSolar, jdnFromDate } from "./jd.ts";
import { jplNewMoonDay, jplTermAtMidnight } from "./jpl.ts";

/** Rust `Method` (`Method::Hnd` = "hnd", `Method::Precise` = "precise", `Method::Jpl` = "jpl"). */
export type Method = "hnd" | "precise" | "jpl";

/** Rust `DEFAULT_METHOD` (currently `Method::Precise`, docs/02 Bổ sung A3). */
export const DEFAULT_METHOD: Method = "precise";
// Switch to "jpl" ONLY after L14–L17 PASS and L12, P1, P3 still PASS (docs/02 A4 item 4), together with Rust:
// export const DEFAULT_METHOD: Method = "jpl";

export function isMethod(v: unknown): v is Method {
  return v === "hnd" || v === "precise" || v === "jpl";
}

/** Throws InvalidArgument unless `m` is "hnd" | "precise" | "jpl" (Rust's enum makes this impossible). */
export function checkMethod(m: unknown): Method {
  if (!isMethod(m)) throw invalidArgument(`phương pháp ${String(m)}`);
  return m;
}

/** Day (JDN, UTC+7) containing new moon k (Rust `new_moon_day`). */
export function newMoonDay(method: Method, k: number): number {
  if (method === "jpl") return jplNewMoonDay(k);
  return localDayOf(newMoonUtWith(method, k));
}

/** New moon k as JD UT (Rust `new_moon_ut_with`). */
export function newMoonUtWith(method: Method, k: number): number {
  // "jpl": instants live only in the Rust core (tsport keeps the day table — jpl.ts).
  if (method === "jpl") throw invalidArgument("jpl: thời điểm Sóc chỉ có ở lõi Rust (tsport dùng bảng ngày)");
  return method === "hnd" ? hndNewMoon(k) : preciseNewMoonUt(k);
}

/** 30° sector (0..11) of the Sun at 00:00 UTC+7 of `jdn` (Rust `sun_sector`). */
export function sunSector(method: Method, jdn: number): number {
  if (method === "hnd") return hndSunSector(jdn);
  if (method === "jpl") return Math.trunc(jplTermAtMidnight(jdn) / 2);
  const l = apparentSunLongitudeUt(localMidnightUt(jdn));
  // Rust: `((l / 30.0).floor() as i32).rem_euclid(12)` (C-6).
  const s = Math.floor(l / 30.0);
  return ((s % 12) + 12) % 12;
}

/** Lunar date (Rust `LunarDate`). Field names follow the PWA/JSON shape. */
export interface LunarDate {
  y: number;
  m: number;
  d: number;
  leap: boolean;
  /** 29 or 30 */
  month_len: number;
}

export interface SolarDate {
  y: number;
  m: number;
  d: number;
}

const K_EPOCH = 2415021.076998695;
const SYNODIC = 29.530588853;

/** Rust MAX_K_ADJUST: bounded correction loops (RS-01). */
const MAX_K_ADJUST = 4;

function kOfMonthStart(method: Method, startJdn: number): number {
  let k = Math.floor((startJdn - K_EPOCH) / SYNODIC + 0.5);
  let n = 0;
  while (newMoonDay(method, k) > startJdn) {
    k -= 1;
    if (++n > MAX_K_ADJUST) throw computation();
  }
  n = 0;
  while (newMoonDay(method, k + 1) <= startJdn) {
    k += 1;
    if (++n > MAX_K_ADJUST) throw computation();
  }
  return k;
}

/** JDN of the start of lunar month 11 (month containing the winter solstice) of solar year yy.
 *  Internal (Rust: private); callers must stay inside the YearTable domain. */
function lunarMonth11(method: Method, yy: number): number {
  const off = jdnFromDate(yy, 12, 31) - 2415021.0;
  const k = Math.floor(off / SYNODIC);
  const nm = newMoonDay(method, k);
  return sunSector(method, nm) >= 9 ? newMoonDay(method, k - 1) : nm;
}

/** Month table of one lunar "tuế": month 11 of yy-1 up to (excluding) month 11 of yy. */
export class YearTable {
  readonly yy: number;
  readonly n: number;
  /** starts[n] = start of the next month 11. */
  readonly starts: number[];
  readonly month: number[];
  readonly leap: boolean[];

  /** Throws OutOfRange (1900–3101) / Computation like Rust `YearTable::new` → Err. */
  constructor(method: Method, yy: number) {
    if (!Number.isSafeInteger(yy) || yy < INTERNAL_MIN_YY || yy > INTERNAL_MAX_YY) {
      throw outOfRange(INTERNAL_MIN_YY, INTERNAL_MAX_YY);
    }
    const a11 = lunarMonth11(method, yy - 1);
    const b11 = lunarMonth11(method, yy);
    const k0 = kOfMonthStart(method, a11);
    const starts: number[] = new Array(14).fill(0);
    let n = 0;
    for (;;) {
      const s = newMoonDay(method, k0 + n);
      if (s >= b11) break;
      if (n >= 13) throw computation();
      starts[n] = s;
      n += 1;
    }
    if (n < 12) throw computation();
    starts[n] = b11;
    const month: number[] = new Array(13).fill(0);
    const leap: boolean[] = new Array(13).fill(false);
    let leapIdx = -1;
    if (n === 13) {
      for (let i = 1; i < 13; i++) {
        if (sunSector(method, starts[i]) === sunSector(method, starts[i + 1])) {
          leapIdx = i;
          break;
        }
      }
      if (leapIdx < 0) throw computation();
    }
    for (let i = 0; i < n; i++) {
      const len = starts[i + 1] - starts[i];
      if (len < 29 || len > 30) throw computation();
    }
    let num = 11;
    for (let i = 0; i < n; i++) {
      if (i > 0) {
        if (i === leapIdx) leap[i] = true;
        else num = num === 12 ? 1 : num + 1;
      }
      month[i] = num;
    }
    this.yy = yy;
    this.n = n;
    this.starts = starts;
    this.month = month;
    this.leap = leap;
  }

  /** null if i is not a month index of this table (Rust `Option`). */
  lunarYear(i: number): number | null {
    if (!Number.isInteger(i) || i < 0 || i >= this.n) return null;
    return this.month[i] >= 11 && i < 4 ? this.yy - 1 : this.yy;
  }

  leapMonth(): number | null {
    for (let i = 0; i < this.n; i++) if (this.leap[i]) return this.month[i];
    return null;
  }
}

/** Throws LvhvError: InvalidArgument (non-integer/NaN JDN), OutOfRange (outside 1900–3000). */
export function jdnToLunar(jdn: number): LunarDate {
  return jdnToLunarWith(DEFAULT_METHOD, jdn);
}

export function jdnToLunarWith(method: Method, jdn: number): LunarDate {
  checkMethod(method);
  return jdnToLunarInternal(method, checkJdn(jdn));
}

function jdnToLunarInternal(method: Method, jdn: number): LunarDate {
  const [yy] = dateFromJdn(jdn);
  const a = lunarMonth11(method, yy);
  const table = new YearTable(method, jdn >= a ? yy + 1 : yy);
  let i = table.n - 1;
  while (i >= 0 && table.starts[i] > jdn) i -= 1;
  if (i < 0) throw computation();
  const d = jdn - table.starts[i] + 1;
  const len = table.starts[i + 1] - table.starts[i];
  if (len < 29 || len > 30 || d < 1 || d > len) throw computation();
  const y = table.lunarYear(i);
  if (y === null) throw computation();
  return { y, m: table.month[i], d, leap: table.leap[i], month_len: len };
}

/** Rust `checked_jdn`: InvalidDate first (incl. NaN / non-integers), then OutOfRange. */
export function checkedJdn(y: number, m: number, d: number, minYear: number = MIN_YEAR): number {
  if (!isValidSolar(y, m, d)) throw invalidDate(y, m, d);
  if (y < minYear || y > MAX_YEAR) throw outOfRange(minYear, MAX_YEAR);
  return jdnFromDate(y, m, d);
}

/** Throws LvhvError InvalidDate (incl. NaN, ±Infinity, non-integers) or OutOfRange. */
export function solarToLunar(y: number, m: number, d: number): LunarDate {
  return solarToLunarWith(DEFAULT_METHOD, y, m, d);
}

export function solarToLunarWith(method: Method, y: number, m: number, d: number): LunarDate {
  checkMethod(method);
  return jdnToLunarInternal(method, checkedJdn(y, m, d));
}

/**
 * Lunar year of a solar BIRTH date (Rust `lunar_year_of_birth`) — use it for the birth Can Chi
 * (`birthLunarYear` of findGoodDays). Domain 1900-01-01 … 3000-12-31; throws like solarToLunar
 * (OutOfRange with min 1900).
 */
export function lunarYearOfBirth(y: number, m: number, d: number): number {
  return lunarYearOfBirthWith(DEFAULT_METHOD, y, m, d);
}

export function lunarYearOfBirthWith(method: Method, y: number, m: number, d: number): number {
  checkMethod(method);
  return jdnToLunarInternal(method, checkedJdn(y, m, d, BIRTH_MIN_YEAR)).y;
}

/** null if the lunar date does not exist, the lunar year is outside 1899–3000, an argument is not
 *  an integer / `leap` is not a boolean, or the result falls outside 1900-01-01 … 3000-12-31. */
export function lunarToJdn(ly: number, lm: number, ld: number, leap: boolean): number | null {
  return lunarToJdnWith(DEFAULT_METHOD, ly, lm, ld, leap);
}

export function lunarToJdnWith(method: Method, ly: number, lm: number, ld: number, leap: boolean): number | null {
  checkMethod(method);
  if (!Number.isSafeInteger(ly) || !Number.isInteger(lm) || !Number.isInteger(ld)) return null;
  if (typeof leap !== "boolean") return null;
  if (ly < MIN_LUNAR_YEAR || ly > MAX_LUNAR_YEAR || lm < 1 || lm > 12 || ld < 1 || ld > 30) return null;
  const yy = lm >= 11 ? ly + 1 : ly;
  const t = new YearTable(method, yy);
  for (let i = 0; i < t.n; i++) {
    if (t.month[i] === lm && t.leap[i] === leap && t.lunarYear(i) === ly) {
      const len = t.starts[i + 1] - t.starts[i];
      if (ld > len) return null;
      const j = t.starts[i] + ld - 1;
      return jdnInRange(j) ? j : null;
    }
  }
  return null;
}

export function lunarToSolar(ly: number, lm: number, ld: number, leap: boolean): SolarDate | null {
  return lunarToSolarWith(DEFAULT_METHOD, ly, lm, ld, leap);
}

export function lunarToSolarWith(method: Method, ly: number, lm: number, ld: number, leap: boolean): SolarDate | null {
  const j = lunarToJdnWith(method, ly, lm, ld, leap);
  if (j === null) return null;
  const [y, m, d] = dateFromJdn(j);
  return { y, m, d };
}

export function newMoonUt(k: number): number {
  return newMoonUtWith(DEFAULT_METHOD, k);
}
