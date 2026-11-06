// Port of core/src/terms.rs — 24 solar terms (index i ↔ apparent longitude 15·i°, 0 = Xuân Phân).

import { apparentSunLongitudeUt, localDayOf, localMidnightUt, solveSunLongitude } from "./astro.ts";
export { localMidnightUt };
import { MAX_YEAR, MIN_YEAR, computation, invalidArgument, outOfRange } from "./error.ts";
import { jdnFromDate } from "./jd.ts";
import { jplTermAtMidnight } from "./jpl.ts";
import type { Method } from "./lunar.ts";

export const TERM_NAMES: readonly string[] = [
  "Xuân Phân", "Thanh Minh", "Cốc Vũ", "Lập Hạ", "Tiểu Mãn", "Mang Chủng",
  "Hạ Chí", "Tiểu Thử", "Đại Thử", "Lập Thu", "Xử Thử", "Bạch Lộ",
  "Thu Phân", "Hàn Lộ", "Sương Giáng", "Lập Đông", "Tiểu Tuyết", "Đại Tuyết",
  "Đông Chí", "Tiểu Hàn", "Đại Hàn", "Lập Xuân", "Vũ Thủy", "Kinh Trập",
];

/** Solar-term index (0..23) of day `jdn`. Rust: `(l/15).floor() as usize` (saturating at 0), `.min(23)`. */
export function termIndexOfDay(jdn: number): number {
  const l = apparentSunLongitudeUt(localMidnightUt(jdn + 1));
  let i = Math.floor(l / 15.0);
  if (!(i > 0)) i = 0; // `as usize` saturates negatives / NaN to 0
  return i % 24; // Rust `% 24` (C-7)
}

/** Rust `term_index_of_day_with`: "hnd"/"precise" share the §6 model; "jpl" uses the DE441 table. */
export function termIndexOfDayWith(method: Method, jdn: number): number {
  if (method !== "jpl") return termIndexOfDay(jdn);
  return jplTermAtMidnight(jdn + 1);
}

/** Chi (0 = Tý) of the solar-term month for a term index. */
export function tietMonthChiOfTerm(termIdx: number): number {
  const m = Math.trunc(((termIdx + 24 - 21) % 24) / 2);
  return (m + 2) % 12;
}

export interface SolarTerm {
  index: number;
  /** JD (UT) */
  jdUt: number;
}

export function solarTermName(t: SolarTerm): string {
  return TERM_NAMES[t.index];
}

export function solarTermLocalJdn(t: SolarTerm): number {
  return localDayOf(t.jdUt);
}

/** Throws LvhvError OutOfRange outside 1900–3000 (Rust → Err), InvalidArgument if not an integer. */
export function solarTermsOfYear(year: number): SolarTerm[] {
  return solarTermsOfYearWith("precise", year);
}

/** Rust `solar_terms_of_year_with`. */
export function solarTermsOfYearWith(method: Method, year: number): SolarTerm[] {
  if (typeof year !== "number" || Number.isNaN(year) || (Number.isFinite(year) && !Number.isInteger(year))) {
    throw invalidArgument(`năm ${String(year)}`);
  }
  if (!(year >= MIN_YEAR && year <= MAX_YEAR)) throw outOfRange();
  const start = jdnFromDate(year, 1, 1);
  const end = jdnFromDate(year + 1, 1, 1);
  // "jpl": term instants live only in the Rust core (`lvhv-dump --method jpl terms`); tsport has the day table.
  if (method === "jpl") throw invalidArgument("jpl: thời điểm tiết khí chỉ có ở lõi Rust (tsport dùng bảng ngày)");
  const out: SolarTerm[] = [];
  let guess = localMidnightUt(start) - 15.0;
  let idx = 18;
  for (let n = 0; n < 27; n++) {
    const t = solveSunLongitude(15.0 * idx, guess);
    const term: SolarTerm = { index: idx, jdUt: t };
    const j = solarTermLocalJdn(term);
    if (j >= start && j < end) out.push(term);
    idx = (idx + 1) % 24;
    guess = t + 15.2;
  }
  if (out.length !== 24 || out.some((t) => !Number.isFinite(t.jdUt))) throw computation();
  return out;
}
