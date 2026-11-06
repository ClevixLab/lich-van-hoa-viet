// Port of memorial/src/gio.rs — save_death (02 §1), anniversary (02 §2), label_of (02 §3),
// age / Can Chi (02 §4), solar occurrence & 29/02 (02 §6).

import { CAN, CHI } from "./canchi.ts";
import { MAX_LUNAR_YEAR, MAX_YEAR, MIN_LUNAR_YEAR, MIN_YEAR } from "./error.ts";
import { isLeapYear, isValidSolar, remEuclid } from "./jd.ts";
import { DEFAULT_METHOD, type Method, solarToLunarWith } from "./lunar.ts";
import { Cal } from "./memorial_cal.ts";
import {
  type BirthInfo, type DateValue, type DeathInfo, type Feb29Rule, type LunarDay, type LunarSource, type Tuoi, type Ymd,
  ageLabelText, cmpYmd, ymd,
} from "./memorial_model.ts";

export class InputError extends Error {
  readonly code: string;
  constructor(code: string, message: string) {
    super(message);
    this.code = code;
    this.name = "InputError";
  }
}
const ie = (code: string, message: string) => new InputError(code, message);
const outOfRange = () => ie("OutOfRange", `Năm ngoài phạm vi đổi lịch (${MIN_YEAR}–${MAX_YEAR})`);

export function checkLunarExists(cal: Cal, y: number, m: number, d: number, leap: boolean): void {
  if (m < 1 || m > 12 || d < 1 || d > 30) throw ie("InvalidLunarDate", `Ngày âm ${d}/${m} không hợp lệ`);
  const len = cal.monthLen(y, m, leap);
  if (len === null) {
    if (leap) throw ie("LunarMonthMissing", `Năm ${y} không có tháng ${m} nhuận`);
    throw ie("Computation", `Không tìm thấy tháng ${m} năm ${y}`);
  }
  if (d > len) {
    throw ie("LunarDayMissing", leap ? `Tháng ${m} nhuận âm lịch năm ${y} chỉ có ${len} ngày` : `Tháng ${m} âm lịch năm ${y} chỉ có ${len} ngày`);
  }
}

function want(v: number | null, what: string): number {
  if (v === null) throw ie("MissingField", `Thiếu ${what}`);
  return v;
}
const p2 = (n: number) => String(n).padStart(2, "0");

export function convertDeath(cal: Cal, input: DateValue): [LunarDay, Ymd | null, LunarSource] {
  if (input.calendar === "solar" && input.precision === "day") {
    const y = want(input.year, "năm"), m = want(input.month, "tháng"), d = want(input.day, "ngày");
    if (input.leap) throw ie("InvalidLeap", "Ngày dương không có tháng nhuận");
    if (!isValidSolar(y, m, d)) throw ie("InvalidSolarDate", `Ngày ${p2(d)}/${p2(m)}/${y} không tồn tại`);
    if (y < MIN_YEAR || y > MAX_YEAR) throw outOfRange();
    let l;
    try { l = solarToLunarWith(cal.method, y, m, d); } catch { throw outOfRange(); }
    return [{ year: l.y, month: l.m, day: l.d, leap: l.leap }, ymd(y, m, d), "converted"];
  }
  if (input.calendar === "lunar" && input.precision === "day") {
    const y = want(input.year, "năm"), m = want(input.month, "tháng"), d = want(input.day, "ngày");
    if (m < 1 || m > 12 || d < 1 || d > 30) throw ie("InvalidLunarDate", `Ngày âm ${d}/${m} không hợp lệ`);
    let solar: Ymd | null = null;
    if (y >= MIN_LUNAR_YEAR && y <= MAX_LUNAR_YEAR) {
      checkLunarExists(cal, y, m, d, input.leap);
      const s = cal.l2s(y, m, d, input.leap);
      solar = s === null ? null : ymd(s.y, s.m, s.d);
    }
    return [{ year: y, month: m, day: d, leap: input.leap }, solar, "entered"];
  }
  if (input.calendar === "lunar" && input.precision === "month_day") {
    if (input.year !== null) throw ie("InvalidPrecision", "Không rõ năm thì năm phải để trống");
    const m = want(input.month, "tháng"), d = want(input.day, "ngày");
    if (m < 1 || m > 12 || d < 1 || d > 30) throw ie("InvalidLunarDate", `Ngày âm ${d}/${m} không hợp lệ`);
    return [{ year: null, month: m, day: d, leap: input.leap }, null, "entered"];
  }
  throw ie("PrecisionNotAllowed", "Ngày mất cần đủ ngày, hoặc ngày và tháng âm lịch");
}

export function saveDeathWith(cal: Cal, input: DateValue, birth: BirthInfo | null, nguong: number): DeathInfo {
  const [lunar, solar, lunar_source] = convertDeath(cal, input);
  const by = birth?.lunar_year ?? null;
  if (by !== null && lunar.year !== null && lunar.year < by) throw ie("DeathBeforeBirth", "Ngày mất trước ngày sinh");
  const tuoi = tuoiMat(birth, lunar.year, nguong);
  return { input: { ...input }, lunar, solar, lunar_source, method: cal.method, tuoi };
}

export function saveDeath(input: DateValue, birth: BirthInfo | null, nguong: number): DeathInfo {
  return saveDeathWith(Cal.get(DEFAULT_METHOD), input, birth, nguong);
}

export type GioLabel = "gio" | "gio_dau" | "gio_het";

export interface Anniversary { year: number; month: number; day: number; solar: Ymd; n: number | null; label: GioLabel }

export function labelOf(n: number | null, gioHetNam: number): GioLabel {
  if (n === null) return "gio";
  if (n === 1) return "gio_dau";
  if (n === gioHetNam) return "gio_het";
  return "gio";
}

export function labelText(label: GioLabel, n: number | null): string {
  if (label === "gio_dau") return "Giỗ đầu";
  if (label === "gio_het") return "Giỗ hết";
  return n === null ? "Ngày giỗ" : "Giỗ";
}

export function anniversary(cal: Cal, origYear: number | null, m: number, d: number, gioHetNam: number, y: number): Anniversary | null {
  if (origYear !== null && y <= origYear) return null;
  if (y < MIN_LUNAR_YEAR || y > MAX_LUNAR_YEAR || m < 1 || m > 12) return null;
  const len = cal.monthLen(y, m, false);
  if (len === null) return null;
  const d2 = d === 30 && len === 29 ? 29 : d;
  const s = cal.l2s(y, m, d2, false);
  if (s === null) return null;
  const n = origYear === null ? null : y - origYear;
  return { year: y, month: m, day: d2, solar: ymd(s.y, s.m, s.d), n, label: labelOf(n, gioHetNam) };
}

export function lyobClamped(cal: Cal, day: Ymd): number {
  if (day.y < MIN_YEAR) return MIN_LUNAR_YEAR;
  if (day.y > MAX_YEAR) return MAX_LUNAR_YEAR;
  try { return solarToLunarWith(cal.method, day.y, day.m, day.d).y; } catch { return MAX_LUNAR_YEAR; }
}

export function anniversariesIn(cal: Cal, origYear: number | null, m: number, d: number, gioHetNam: number, from: Ymd, to: Ymd): Anniversary[] {
  const out: Anniversary[] = [];
  if (cmpYmd(from, to) > 0) return out;
  const a = lyobClamped(cal, from) - 1, b = lyobClamped(cal, to) + 1;
  for (let y = a; y <= b; y++) {
    const x = anniversary(cal, origYear, m, d, gioHetNam, y);
    if (x !== null && cmpYmd(from, x.solar) <= 0 && cmpYmd(x.solar, to) <= 0) out.push(x);
  }
  out.sort((p, q) => cmpYmd(p.solar, q.solar));
  return out;
}

export function canChiYear(ly: number): string {
  return `${CAN[remEuclid(ly + 6, 10)]} ${CHI[remEuclid(ly + 8, 12)]}`;
}

export function deriveBirth(cal: Cal, input: DateValue): BirthInfo {
  const b: BirthInfo = { input: { ...input }, lunar_year: null, lunar_year_approx: false, can_chi: null, lunar: null, solar: null };
  if (input.calendar === "solar" && input.precision === "day") {
    const y = want(input.year, "năm"), m = want(input.month, "tháng"), d = want(input.day, "ngày");
    if (!isValidSolar(y, m, d) || input.leap) throw ie("InvalidSolarDate", `Ngày ${p2(d)}/${p2(m)}/${y} không tồn tại`);
    b.solar = ymd(y, m, d);
    if (y >= MIN_YEAR && y <= MAX_YEAR) {
      let l;
      try { l = solarToLunarWith(cal.method, y, m, d); } catch { throw outOfRange(); }
      b.lunar_year = l.y;
      b.lunar = { year: l.y, month: l.m, day: l.d, leap: l.leap };
    }
  } else if (input.calendar === "lunar" && input.precision === "day") {
    const y = want(input.year, "năm"), m = want(input.month, "tháng"), d = want(input.day, "ngày");
    if (m < 1 || m > 12 || d < 1 || d > 30) throw ie("InvalidLunarDate", `Ngày âm ${d}/${m} không hợp lệ`);
    if (y >= MIN_LUNAR_YEAR && y <= MAX_LUNAR_YEAR) {
      checkLunarExists(cal, y, m, d, input.leap);
      const s = cal.l2s(y, m, d, input.leap);
      b.solar = s === null ? null : ymd(s.y, s.m, s.d);
    }
    b.lunar_year = y;
    b.lunar = { year: y, month: m, day: d, leap: input.leap };
  } else if (input.calendar === "lunar" && input.precision === "year") {
    b.lunar_year = want(input.year, "năm");
  } else if (input.calendar === "solar" && input.precision === "year") {
    b.lunar_year = want(input.year, "năm");
    b.lunar_year_approx = true;
  } else {
    throw ie("PrecisionNotAllowed", "Ngày sinh cần đủ ngày hoặc ít nhất năm");
  }
  b.can_chi = b.lunar_year === null ? null : canChiYear(b.lunar_year);
  return b;
}

export function tuoiMat(birth: BirthInfo | null, deathLunarYear: number | null, nguong: number): Tuoi {
  const by = birth?.lunar_year ?? null;
  const value = by !== null && deathLunarYear !== null ? deathLunarYear - by + 1 : null;
  if (value !== null) {
    if (value < 1) throw ie("DeathBeforeBirth", "Ngày mất trước ngày sinh");
    if (value > 150) throw ie("AgeTooLarge", `Tuổi ${value} vượt quá 150`);
  }
  return {
    value,
    label: value !== null && value < nguong ? "huong_duong" : "huong_tho",
    manual: false,
    approx: birth !== null && birth.lunar_year_approx,
  };
}

export function tuoiText(t: Tuoi): string | null {
  return t.value === null ? null : `${ageLabelText(t.label)} ${t.value} tuổi${t.approx ? " (khoảng)" : ""}`;
}

export function solarOccurrence(orig: Ymd, y: number, rule: Feb29Rule): Ymd | null {
  if (y < orig.y) return null;
  if (orig.m === 2 && orig.d === 29 && !isLeapYear(y)) return rule === "feb28" ? ymd(y, 2, 28) : ymd(y, 3, 1);
  return ymd(y, orig.m, orig.d);
}

export function tuoiDuong(birth: Ymd, today: Ymd, rule: Feb29Rule): number | null {
  if (cmpYmd(today, birth) < 0) return null;
  const occ = solarOccurrence(birth, today.y, rule);
  if (occ === null) return null;
  return today.y - birth.y - (cmpYmd(today, occ) < 0 ? 1 : 0);
}

export function tuoiAm(cal: Cal, lunarYear: number, today: Ymd): number | null {
  if (today.y < MIN_YEAR || today.y > MAX_YEAR) return null;
  try { return solarToLunarWith(cal.method, today.y, today.m, today.d).y - lunarYear + 1; } catch { return null; }
}

export const methodName = (m: Method): string => m;
