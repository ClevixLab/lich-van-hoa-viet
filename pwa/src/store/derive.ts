// Trường dẫn xuất (docs/v0.2/01 §3.2, §3.3; 02 §1, §4) và kiểm tra V5 — bản TypeScript của
// store/src/derive.rs. Đổi lịch bằng lõi tsport (cùng phương pháp mặc định "jpl" với lõi Rust; bản ghi cũ "precise"/"hnd" vẫn đọc được).
import * as core from "#engine-impl";
import type { Json, JsonObj } from "./json";
import { ValidationError, int } from "./validate";

const METHOD = core.DEFAULT_METHOD;
type Method = typeof METHOD;

export const lunarYearChecked = (y: number) => y >= core.MIN_LUNAR_YEAR && y <= core.MAX_LUNAR_YEAR;
export const solarYearChecked = (y: number) => y >= core.MIN_YEAR && y <= core.MAX_YEAR;
export const isLeapYear = (y: number) => (y % 4 === 0 && y % 100 !== 0) || y % 400 === 0;
export function daysInMonth(y: number, m: number): number {
  return [31, isLeapYear(y) ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][m - 1] ?? 0;
}

export function l2s(y: number, m: number, d: number, leap: boolean, method: Method = METHOD): JsonObj | null {
  const s = core.lunarToSolarWith(method, y, m, d, leap);
  return s ? { y: s.y, m: s.m, d: s.d } : null;
}
export function s2l(y: number, m: number, d: number, method: Method = METHOD): JsonObj | null {
  try {
    const l = core.solarToLunarWith(method, y, m, d);
    return { year: l.y, month: l.m, day: l.d, leap: l.leap };
  } catch { return null; }
}

/** Can Chi của năm âm (02 §4.1). */
export function canChiYear(y: number): string {
  return `${core.canchi.CAN[core.canchi.yearCan(y)]} ${core.canchi.CHI[core.canchi.yearChi(y)]}`;
}

export class InputError extends Error {}

function lunarMissingMsg(y: number, m: number, d: number, leap: boolean): string {
  if (leap && core.lunarToSolarWith(METHOD, y, m, 1, true) === null) return `Năm ${y} không có tháng ${m} nhuận`;
  if (d === 30 && core.lunarToSolarWith(METHOD, y, m, 29, leap) !== null) return `Tháng ${m}${leap ? " nhuận" : ""} âm lịch năm ${y} chỉ có 29 ngày`;
  return `Ngày ${d}/${m} âm lịch năm ${y} không tồn tại`;
}

/** 02 §1 `save_death(input)` → {lunar, solar, lunar_source, method}. */
export function saveDeath(input: JsonObj): JsonObj {
  const lunarCal = input.calendar === "lunar";
  const pr = input.precision;
  const [y, m, d, leap] = [int(input.year), int(input.month), int(input.day), input.leap === true];
  if (!lunarCal) {
    if (y === null || m === null || d === null || pr !== "day") throw new InputError("Ngày mất theo dương lịch phải đủ ngày, tháng, năm");
    if (!solarYearChecked(y)) throw new InputError(`Năm ${y} ngoài khoảng ${core.MIN_YEAR}–${core.MAX_YEAR}`);
    if (d > daysInMonth(y, m)) throw new InputError(`Tháng ${m} năm ${y} chỉ có ${daysInMonth(y, m)} ngày`);
    const l = s2l(y, m, d);
    if (!l) throw new InputError("Không đổi được sang âm lịch");
    return { lunar: l, lunar_source: "converted", method: METHOD, solar: { y, m, d } };
  }
  if (m === null || d === null) throw new InputError("Ngày mất phải có ngày và tháng");
  if (pr === "day") {
    if (y === null) throw new InputError("Thiếu năm");
    let solar: Json = null;
    if (lunarYearChecked(y)) {
      solar = l2s(y, m, d, leap);
      if (!solar) throw new InputError(lunarMissingMsg(y, m, d, leap));
    }
    return { lunar: { year: y, month: m, day: d, leap }, lunar_source: "entered", method: METHOD, solar };
  }
  if (pr === "month_day") return { lunar: { year: null, month: m, day: d, leap }, lunar_source: "entered", method: METHOD, solar: null };
  throw new InputError("Ngày mất phải biết ít nhất ngày và tháng âm");
}

/** 02 §4.1 + 01 §3.3: trường dẫn xuất của BirthInfo. */
export function deriveBirth(input: JsonObj): JsonObj {
  const lunarCal = input.calendar === "lunar";
  const pr = input.precision;
  const [y, m, d, leap] = [int(input.year), int(input.month), int(input.day), input.leap === true];
  if (y === null) throw new InputError("Thiếu năm sinh");
  let ly: number, approx: boolean, lunar: Json, solar: Json;
  if (!lunarCal && pr === "day") {
    if (m === null || d === null) throw new InputError("Thiếu ngày sinh");
    if (d > daysInMonth(y, m)) throw new InputError(`Tháng ${m} năm ${y} chỉ có ${daysInMonth(y, m)} ngày`);
    if (solarYearChecked(y)) {
      try { ly = core.lunarYearOfBirthWith(METHOD, y, m, d); } catch { throw new InputError("Không đổi được sang âm lịch"); }
      approx = false; lunar = s2l(y, m, d); solar = { y, m, d };
    } else {
      ly = y; approx = true; lunar = null; solar = { y, m, d };
    }
  } else if (!lunarCal && pr === "year") {
    ly = y; approx = true; lunar = null; solar = null;
  } else if (lunarCal && pr === "day") {
    if (m === null || d === null) throw new InputError("Thiếu ngày sinh");
    solar = null;
    if (lunarYearChecked(y)) {
      solar = l2s(y, m, d, leap);
      if (!solar) throw new InputError(lunarMissingMsg(y, m, d, leap));
    }
    ly = y; approx = false; lunar = { year: y, month: m, day: d, leap };
  } else if (lunarCal && pr === "year") {
    ly = y; approx = false; lunar = null; solar = null;
  } else {
    throw new InputError("Ngày sinh phải đủ ngày hoặc chỉ năm");
  }
  return { can_chi: canChiYear(ly), lunar, lunar_year: ly, lunar_year_approx: approx, solar };
}

/** 02 §4.2 `tuoi_mat`. */
export function tuoiMat(birth: Json, deathLunarYear: number | null, nguong: number): JsonObj {
  const b = birth as JsonObj | null;
  const by = b ? int(b.lunar_year) : null;
  const value = by !== null && deathLunarYear !== null ? deathLunarYear - by + 1 : null;
  return { approx: !!b && b.lunar_year_approx === true, label: value !== null && value < nguong ? "huong_duong" : "huong_tho", manual: false, value };
}

/** So sâu hai giá trị JSON. */
export function jsonEq(a: Json, b: Json): boolean {
  if (a === b) return true;
  if (typeof a !== "object" || typeof b !== "object" || a === null || b === null) return false;
  if (Array.isArray(a) !== Array.isArray(b)) return false;
  if (Array.isArray(a)) return a.length === (b as Json[]).length && a.every((x, i) => jsonEq(x, (b as Json[])[i]));
  const ka = Object.keys(a), kb = Object.keys(b as JsonObj);
  return ka.length === kb.length && ka.every((k) => Object.prototype.hasOwnProperty.call(b, k) && jsonEq(a[k], (b as JsonObj)[k]));
}

const sortedEntries = (o: JsonObj) => Object.keys(o).sort().map((k) => [k, o[k]] as const);

/** V5 cho một Person (đã qua lược đồ) — cùng thứ tự kiểm với Rust (khóa theo thứ tự chữ cái). */
export function checkPersonDerived(o: JsonObj, path: string): void {
  const birth = o.birth as JsonObj | null;
  if (birth !== null) {
    const bp = `${path}/birth`;
    let d: JsonObj;
    try { d = deriveBirth(birth.input as JsonObj); } catch (e) { throw new ValidationError("V4", `${bp}/input`, (e as Error).message); }
    for (const [k, want] of sortedEntries(d)) if (!jsonEq(birth[k], want)) throw new ValidationError("V5", `${bp}/${k}`, `khác giá trị tính lại ${JSON.stringify(want)}`);
  }
  const death = o.death as JsonObj | null;
  if (death !== null) {
    const dp = `${path}/death`;
    let d: JsonObj;
    try { d = saveDeath(death.input as JsonObj); } catch (e) { throw new ValidationError("V4", `${dp}/input`, (e as Error).message); }
    const strict = death.method === METHOD;
    for (const [k, want] of sortedEntries(d)) {
      if (!strict && (k === "lunar" || k === "solar" || k === "method")) continue;
      if (!jsonEq(death[k], want)) throw new ValidationError("V5", `${dp}/${k}`, `khác giá trị tính lại ${JSON.stringify(want)}`);
    }
    const t = death.tuoi as JsonObj;
    const dy = int((death.lunar as JsonObj).year);
    if (t.manual !== true) {
      const nguong = int(((o.memorial ?? {}) as JsonObj).huong_duong_duoi ?? null) ?? 60;
      const want = tuoiMat(birth, dy, nguong);
      if (typeof want.value === "number" && want.value < 1) throw new ValidationError("V4", `${dp}/input`, "Ngày mất trước ngày sinh");
      if (!jsonEq(t, want)) throw new ValidationError("V5", `${dp}/tuoi`, `khác giá trị tính lại ${JSON.stringify(want)}`);
    } else {
      const by = birth ? int(birth.lunar_year) : null;
      if (by !== null && dy !== null && dy < by) throw new ValidationError("V4", `${dp}/input`, "Ngày mất trước ngày sinh");
    }
  }
}

/** Điền trường dẫn xuất cho một Person (tiện ích cho giao diện, đợt 3). */
export function buildPerson(base: JsonObj): JsonObj {
  const p = structuredClone(base) as JsonObj;
  const birth = p.birth as JsonObj | null;
  if (birth) Object.assign(birth, deriveBirth(birth.input as JsonObj));
  const death = p.death as JsonObj | null;
  if (death) {
    Object.assign(death, saveDeath(death.input as JsonObj));
    if ((death.tuoi as JsonObj | undefined)?.manual !== true) {
      const nguong = int(((p.memorial ?? {}) as JsonObj).huong_duong_duoi ?? null) ?? 60;
      death.tuoi = tuoiMat(birth, int((death.lunar as JsonObj).year), nguong);
    }
  }
  return p;
}
