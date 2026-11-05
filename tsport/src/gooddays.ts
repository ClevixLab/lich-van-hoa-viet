// Port of core/src/gooddays.rs — find_good_days / activities / GoodDay JSON.

import { CHI, gioHoangDao, hourRange } from "./canchi.ts";
import { type GioHoangDao, dayCtxJdn, ngochapCtx } from "./dayinfo.ts";
import {
  MAX_SPAN_DAYS, computation, invalidArgument, invertedRange, isSafeInt, rangeTooLong, unknownActivity,
} from "./error.ts";
import { dateFromJdn } from "./jd.ts";
import { writeJsonStr } from "./json.ts";
import { DEFAULT_METHOD, type LunarDate, type Method, type SolarDate, checkMethod, checkedJdn } from "./lunar.ts";
import { activities, evaluate, suitability } from "./ngochap.ts";

/** Rust `MIN_GOOD_DAY_SCORE`: minimum activity score of a suggested day. */
export const MIN_GOOD_DAY_SCORE = 0;

export interface GoodDay {
  solar: SolarDate;
  lunar: LunarDate;
  /** 0 = Chủ nhật … 6 = Thứ Bảy */
  weekday: number;
  jdn: number;
  score: number;
  /** always true in findGoodDays results */
  allowed: boolean;
  /** activity listed in the day sheet's `ky` (not serialized; always false in results) */
  in_sheet_ky: boolean;
  reasons: string[];
  first_gio_hoang_dao: GioHoangDao;
}

/**
 * Rust `gooddays::evaluate_day_for` (returns the day even when allowed = false).
 * Throws LvhvError: UnknownActivity, InvalidArgument / OutOfRange (JDN), InvalidArgument for a
 * birth year that is not a safe integer.
 */
export function evaluateDayFor(method: Method, jdn: number, activityId: string, birthLunarYear?: number | null): GoodDay {
  checkMethod(method);
  checkBirth(birthLunarYear);
  if (!activities().some((a) => a.id === activityId)) throw unknownActivity();
  const c = dayCtxJdn(jdn, method);
  const nctx = ngochapCtx(c);
  if (nctx === null) throw computation();
  const s = suitability(nctx, activityId, birthLunarYear);
  const name = activities().find((a) => a.id === activityId)!.name;
  const inSheetKy = evaluate(nctx).ky.includes(name);
  const [y, m, d] = dateFromJdn(jdn);
  const h = gioHoangDao(c.day_chi)[0];
  const [from, to] = hourRange(h);
  return {
    solar: { y, m, d },
    lunar: { y: c.lunar_year, m: c.lunar_month, d: c.lunar_day, leap: c.leap, month_len: c.month_len },
    weekday: c.weekday,
    jdn,
    score: s.score,
    allowed: s.allowed,
    in_sheet_ky: inSheetKy,
    reasons: s.reasons,
    first_gio_hoang_dao: { chi: CHI[h], from, to },
  };
}

function checkBirth(b: unknown): void {
  if (b !== undefined && b !== null && !isSafeInt(b)) throw invalidArgument(`năm sinh âm lịch ${String(b)}`);
}

function checkSolarArg(s: unknown): SolarDate {
  if (typeof s !== "object" || s === null) throw invalidArgument("ngày dương phải là {y, m, d}");
  return s as SolarDate;
}

/** Rust `GoodDay::is_suggestable`: allowed, score ≥ MIN_GOOD_DAY_SCORE, activity ∉ sheet `ky`. */
export function isSuggestable(g: GoodDay): boolean {
  return g.allowed && g.score >= MIN_GOOD_DAY_SCORE && !g.in_sheet_ky;
}

/**
 * Rust `find_good_days_with`: every day in [from, to], keep only suggestable days (see
 * isSuggestable / SCORING.md §2a), sort by score desc then date asc. `birthLunarYear` = LUNAR year of the birth date (see lunarYearOfBirth), or
 * null/undefined. Throws LvhvError (same order of checks as Rust):
 *   InvalidDate → OutOfRange (from, then to) → InvertedRange → RangeTooLong (to − from > 366)
 *   → UnknownActivity.
 */
export function findGoodDaysWith(method: Method, activityId: string, from: SolarDate, to: SolarDate, birthLunarYear?: number | null): GoodDay[] {
  checkMethod(method);
  const f = checkSolarArg(from);
  const t = checkSolarArg(to);
  const a = checkedJdn(f.y, f.m, f.d);
  const b = checkedJdn(t.y, t.m, t.d);
  if (a > b) throw invertedRange();
  if (b - a > MAX_SPAN_DAYS) throw rangeTooLong(b - a);
  if (!activities().some((x) => x.id === activityId)) throw unknownActivity();
  checkBirth(birthLunarYear);
  const out: GoodDay[] = [];
  for (let j = a; j <= b; j++) {
    const g = evaluateDayFor(method, j, activityId, birthLunarYear);
    if (isSuggestable(g)) out.push(g);
  }
  out.sort((p, q) => q.score - p.score || p.jdn - q.jdn);
  return out;
}

/** Rust `find_good_days` (default method). */
export function findGoodDays(activityId: string, from: SolarDate, to: SolarDate, birthLunarYear?: number | null): GoodDay[] {
  return findGoodDaysWith(DEFAULT_METHOD, activityId, from, to, birthLunarYear);
}

/** Port of Rust `GoodDay::to_json`. */
export function goodDayJson(g: GoodDay): string {
  const o: string[] = [];
  o.push(`{"solar":{"y":${g.solar.y},"m":${g.solar.m},"d":${g.solar.d}}`);
  o.push(`,"lunar":{"y":${g.lunar.y},"m":${g.lunar.m},"d":${g.lunar.d},"leap":${g.lunar.leap},"month_len":${g.lunar.month_len}}`);
  o.push(`,"weekday":${g.weekday},"jdn":${g.jdn},"score":${g.score},"allowed":${g.allowed},"reasons":[`);
  g.reasons.forEach((r, i) => {
    if (i > 0) o.push(",");
    writeJsonStr(o, r);
  });
  o.push('],"first_gio_hoang_dao":{"chi":');
  writeJsonStr(o, g.first_gio_hoang_dao.chi);
  o.push(`,"from":${g.first_gio_hoang_dao.from},"to":${g.first_gio_hoang_dao.to}}}`);
  return o.join("");
}
