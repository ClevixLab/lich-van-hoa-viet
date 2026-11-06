// lvhv-core TypeScript port — public API used by the PWA (pwa/src/engine.ts).
// Zero runtime dependencies. Mirrors core/src/lib.rs (+ the lvhv-ngochap crate).
//
// Method selection mirrors Rust: every function defaults to DEFAULT_METHOD ("hnd" = Method::Hnd);
// the `…With(method, …)` variants (and the optional trailing `method` argument of dayInfo /
// dayInfoJson / dayCtx) select "hnd" | "precise" explicitly.
//
// Errors mirror Rust (see error.ts): a Rust `Err(DateError::X)` is a thrown `LvhvError`
// (subclass of RangeError) with `code === "X"`; a Rust `None` is `null`. Supported domain:
// solar dates 1900-01-01 … 3000-12-31 (lunar calendar in UTC+8 before 1968-01-01, UTC+7 after).

import { MAX_YEAR, MIN_YEAR, computation, invalidArgument, outOfRange } from "./error.ts";
import { DEFAULT_METHOD, type Method, type SolarDate, checkMethod, lunarToSolarWith } from "./lunar.ts";

export {
  dayCtx, dayCtxJdn, dayInfo, dayInfoJson, dayInfoJsonWith, dayInfoWith, ngocHapOf, ngochapCtx, toJson,
} from "./dayinfo.ts";
export type { DayCtx, DayInfo, GioHoangDao } from "./dayinfo.ts";
export { activities, evaluate, majorTaboos, suitability } from "./ngochap.ts";
export type { Activity, NgocHap, NgocHapCtx, Suitability } from "./ngochap.ts";
export { MIN_GOOD_DAY_SCORE, evaluateDayFor, findGoodDays, findGoodDaysWith, goodDayJson, isSuggestable } from "./gooddays.ts";
export type { GoodDay } from "./gooddays.ts";
export {
  DEFAULT_METHOD, isMethod, jdnToLunar, jdnToLunarWith, lunarToJdn, lunarToJdnWith, lunarToSolar,
  lunarToSolarWith, lunarYearOfBirth, lunarYearOfBirthWith, solarToLunar, solarToLunarWith,
} from "./lunar.ts";
export {
  BIRTH_MIN_YEAR, LvhvError, MAX_JDN, MAX_LUNAR_YEAR, MAX_SPAN_DAYS, MAX_YEAR, MIN_JDN, MIN_LUNAR_YEAR,
  MIN_YEAR,
} from "./error.ts";
export type { LvhvErrorCode } from "./error.ts";
export type { LunarDate, Method, SolarDate } from "./lunar.ts";
export { solarTermsOfYear, TERM_NAMES } from "./terms.ts";
export type { SolarTerm } from "./terms.ts";
export * as jd from "./jd.ts";
export * as astro from "./astro.ts";
export * as canchi from "./canchi.ts";

/** Tết Nguyên đán (1st day of lunar month 1, non-leap) of lunar year `ly`.
 *  Throws LvhvError OutOfRange outside 1900–3000, InvalidArgument for a non-integer. */
export function tet(ly: number): SolarDate {
  return tetWith(DEFAULT_METHOD, ly);
}

export function tetWith(method: Method, ly: number): SolarDate {
  checkMethod(method);
  if (typeof ly !== "number" || Number.isNaN(ly) || (Number.isFinite(ly) && !Number.isInteger(ly))) {
    throw invalidArgument(`năm âm ${String(ly)}`);
  }
  if (!(ly >= MIN_YEAR && ly <= MAX_YEAR)) throw outOfRange();
  const t = lunarToSolarWith(method, ly, 1, 1, false);
  if (t === null) throw computation();
  return t;
}
