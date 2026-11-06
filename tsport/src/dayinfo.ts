// Port of core/src/dayinfo.rs — DayCtx, DayInfo and the hand-written JSON serializer
// (fixed key order, architecture contract §4).

import {
  CHI, NAP_AM, THAN, THAN_HOANG_DAO, TRUC, TU,
  canChiName, dayCan, dayChi, gioHoangDao, hourRange, hourTyCan, monthCan, monthChi,
  napAmIndex, thanIndex, trucIndex, tuIndex, yearCan, yearChi,
} from "./canchi.ts";
import { computation } from "./error.ts";
import { dateFromJdn, weekday } from "./jd.ts";
import { DEFAULT_METHOD, type Method, checkMethod, checkedJdn, jdnToLunarWith } from "./lunar.ts";
import { type NgocHap, type NgocHapCtx, evaluate, validateCtx, writeNgocHapJson } from "./ngochap.ts";
import { TERM_NAMES, termIndexOfDayWith, tietMonthChiOfTerm } from "./terms.ts";
import { writeJsonStr } from "./json.ts";

/** Numeric context of a day (Rust `DayCtx`; same field names, snake_case). */
export interface DayCtx {
  jdn: number;
  weekday: number;
  lunar_year: number;
  lunar_month: number;
  lunar_day: number;
  leap: boolean;
  month_len: number;
  day_can: number;
  day_chi: number;
  month_can: number;
  month_chi: number;
  month_chi_tiet: number;
  year_can: number;
  year_chi: number;
  hour_ty_can: number;
  tiet_khi: number;
  truc: number;
  tu: number;
  than: number;
  nap_am: number;
}

export interface GioHoangDao { chi: string; from: number; to: number; }

export interface DayInfo {
  solar: { y: number; m: number; d: number; weekday: number; jdn: number };
  lunar: { y: number; m: number; d: number; leap: boolean; month_len: number };
  can_chi: { year: string; month: string; day: string; hour_ty: string };
  tiet_khi: string;
  nap_am: string;
  truc: string;
  tu: string;
  than: { name: string; hoang_dao: boolean };
  gio_hoang_dao: GioHoangDao[];
  ngoc_hap: NgocHap;
}

/** Throws LvhvError (InvalidArgument / OutOfRange) like Rust `day_ctx_jdn_with` → Err. */
export function dayCtxJdn(jdn: number, method: Method = DEFAULT_METHOD): DayCtx {
  const l = jdnToLunarWith(method, jdn);
  const dcan = dayCan(jdn);
  const dchi = dayChi(jdn);
  const ycan = yearCan(l.y);
  const term = termIndexOfDayWith(method, jdn);
  const mchiTiet = tietMonthChiOfTerm(term);
  return {
    jdn,
    weekday: weekday(jdn),
    lunar_year: l.y,
    lunar_month: l.m,
    lunar_day: l.d,
    leap: l.leap,
    month_len: l.month_len,
    day_can: dcan,
    day_chi: dchi,
    month_can: monthCan(ycan, l.m),
    month_chi: monthChi(l.m),
    month_chi_tiet: mchiTiet,
    year_can: ycan,
    year_chi: yearChi(l.y),
    hour_ty_can: hourTyCan(dcan),
    tiet_khi: term,
    truc: trucIndex(dchi, mchiTiet),
    tu: tuIndex(jdn),
    than: thanIndex(l.m, dchi),
    nap_am: napAmIndex(dcan, dchi),
  };
}

/** Throws LvhvError InvalidDate / OutOfRange (Rust `day_ctx_with` → Err). */
export function dayCtx(y: number, m: number, d: number, method: Method = DEFAULT_METHOD): DayCtx {
  checkMethod(method);
  return dayCtxJdn(checkedJdn(y, m, d), method);
}

/** Rust `dayinfo::ngochap_ctx`: core DayCtx → Ngọc Hạp ctx (with solar month/day); null if any
 *  field is outside its domain (Rust `u8::try_from` + `DayCtx::validate`, RS-05/C-4). */
export function ngochapCtx(c: DayCtx): NgocHapCtx | null {
  const [, sm, sd] = dateFromJdn(c.jdn);
  const n: NgocHapCtx = {
    lunar_month: c.lunar_month,
    lunar_day: c.lunar_day,
    leap: c.leap,
    day_can: c.day_can,
    day_chi: c.day_chi,
    month_chi_tiet: c.month_chi_tiet,
    year_can: c.year_can,
    year_chi: c.year_chi,
    truc: c.truc,
    tu: c.tu,
    than: c.than,
    solar_month: sm,
    solar_day: sd,
  };
  return validateCtx(n) ? n : null;
}

/** Rust `dayinfo::ngoc_hap_of` (null if the ctx is invalid). */
export function ngocHapOf(c: DayCtx): NgocHap | null {
  const n = ngochapCtx(c);
  return n === null ? null : evaluate(n);
}

/** Build DayInfo from an already computed ctx (Rust `day_info` body). */
export function dayInfoFromCtx(y: number, m: number, d: number, c: DayCtx): DayInfo {
  const nh = ngocHapOf(c);
  if (nh === null) throw computation();
  const gio: GioHoangDao[] = gioHoangDao(c.day_chi).map((h) => {
    const [from, to] = hourRange(h);
    return { chi: CHI[h], from, to };
  });
  return {
    solar: { y, m, d, weekday: c.weekday, jdn: c.jdn },
    lunar: { y: c.lunar_year, m: c.lunar_month, d: c.lunar_day, leap: c.leap, month_len: c.month_len },
    can_chi: {
      year: canChiName(c.year_can, c.year_chi),
      month: canChiName(c.month_can, c.month_chi),
      day: canChiName(c.day_can, c.day_chi),
      hour_ty: canChiName(c.hour_ty_can, 0),
    },
    tiet_khi: TERM_NAMES[c.tiet_khi],
    nap_am: NAP_AM[c.nap_am],
    truc: TRUC[c.truc],
    tu: TU[c.tu],
    than: { name: THAN[c.than], hoang_dao: THAN_HOANG_DAO[c.than] },
    gio_hoang_dao: gio,
    ngoc_hap: nh,
  };
}

/** Throws LvhvError InvalidDate (incl. NaN / non-integers) / OutOfRange (Rust `day_info` → Err). */
export function dayInfo(y: number, m: number, d: number, method: Method = DEFAULT_METHOD): DayInfo {
  return dayInfoFromCtx(y, m, d, dayCtx(y, m, d, method));
}

/** Port of Rust `DayInfo::to_json` — one line, no whitespace, fixed key order. */
export function toJson(di: DayInfo): string {
  const o: string[] = [];
  const s = di.solar;
  const l = di.lunar;
  o.push(`{"solar":{"y":${s.y},"m":${s.m},"d":${s.d},"weekday":${s.weekday},"jdn":${s.jdn}}`);
  o.push(`,"lunar":{"y":${l.y},"m":${l.m},"d":${l.d},"leap":${l.leap},"month_len":${l.month_len}}`);
  o.push(',"can_chi":{"year":');
  writeJsonStr(o, di.can_chi.year);
  o.push(',"month":');
  writeJsonStr(o, di.can_chi.month);
  o.push(',"day":');
  writeJsonStr(o, di.can_chi.day);
  o.push(',"hour_ty":');
  writeJsonStr(o, di.can_chi.hour_ty);
  o.push('},"tiet_khi":');
  writeJsonStr(o, di.tiet_khi);
  o.push(',"nap_am":');
  writeJsonStr(o, di.nap_am);
  o.push(',"truc":');
  writeJsonStr(o, di.truc);
  o.push(',"tu":');
  writeJsonStr(o, di.tu);
  o.push(',"than":{"name":');
  writeJsonStr(o, di.than.name);
  o.push(`,"hoang_dao":${di.than.hoang_dao}}`);
  o.push(',"gio_hoang_dao":[');
  di.gio_hoang_dao.forEach((g, i) => {
    if (i > 0) o.push(",");
    o.push('{"chi":');
    writeJsonStr(o, g.chi);
    o.push(`,"from":${g.from},"to":${g.to}}`);
  });
  o.push('],"ngoc_hap":');
  writeNgocHapJson(o, di.ngoc_hap);
  o.push("}");
  return o.join("");
}

export function dayInfoJson(y: number, m: number, d: number, method: Method = DEFAULT_METHOD): string {
  return toJson(dayInfo(y, m, d, method));
}

/** Rust `day_info_with(method, y, m, d)` (method first, like every other `…With`; A-3). */
export function dayInfoWith(method: Method, y: number, m: number, d: number): DayInfo {
  return dayInfo(y, m, d, method);
}

/** Rust `day_info_with(..).to_json()`. */
export function dayInfoJsonWith(method: Method, y: number, m: number, d: number): string {
  return dayInfoJson(y, m, d, method);
}
