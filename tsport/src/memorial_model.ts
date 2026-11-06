// Port of memorial/src/model.rs — data types of docs/v0.2/01 read by the algorithms, and the v1 parser.

import type { Method } from "./lunar.ts";
import { dateFromJdn, isValidSolar, jdnFromDate } from "./jd.ts";
import type { JValue } from "./memorial_json.ts";

export interface Ymd { y: number; m: number; d: number }

export const ymd = (y: number, m: number, d: number): Ymd => ({ y, m, d });
export const ymdJdn = (a: Ymd): number => jdnFromDate(a.y, a.m, a.d);
export function ymdFromJdn(j: number): Ymd {
  const [y, m, d] = dateFromJdn(j);
  return { y, m, d };
}
const p2 = (n: number) => String(n).padStart(2, "0");
const p4 = (n: number) => String(n).padStart(4, "0");
export const iso = (a: Ymd): string => `${p4(a.y)}-${p2(a.m)}-${p2(a.d)}`;
export function parseYmd(s: string): Ymd | null {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(s)) return null;
  const y = Number(s.slice(0, 4)), m = Number(s.slice(5, 7)), d = Number(s.slice(8, 10));
  return isValidSolar(y, m, d) ? { y, m, d } : null;
}
export function cmpYmd(a: Ymd, b: Ymd): number {
  return a.y - b.y || a.m - b.m || a.d - b.d;
}

export type Calendar = "solar" | "lunar";
export type Precision = "day" | "month_day" | "year";

export interface DateValue {
  calendar: Calendar;
  precision: Precision;
  year: number | null;
  month: number | null;
  day: number | null;
  leap: boolean;
}

export const dvSolar = (y: number, m: number, d: number): DateValue =>
  ({ calendar: "solar", precision: "day", year: y, month: m, day: d, leap: false });
export const dvLunar = (y: number | null, m: number, d: number, leap: boolean): DateValue =>
  ({ calendar: "lunar", precision: y === null ? "month_day" : "day", year: y, month: m, day: d, leap });
export const dvYear = (calendar: Calendar, y: number): DateValue =>
  ({ calendar, precision: "year", year: y, month: null, day: null, leap: false });

export function dateValueJson(v: DateValue): string {
  const o = (x: number | null) => (x === null ? "null" : String(x));
  return `{"calendar":"${v.calendar}","precision":"${v.precision}","year":${o(v.year)},"month":${o(v.month)},"day":${o(v.day)},"leap":${v.leap}}`;
}

export interface LunarDay { year: number | null; month: number; day: number; leap: boolean }

export type AgeLabel = "huong_tho" | "huong_duong";
export const ageLabelText = (l: AgeLabel): string => (l === "huong_tho" ? "Hưởng thọ" : "Hưởng dương");

export interface Tuoi { value: number | null; label: AgeLabel; manual: boolean; approx: boolean }
export type LunarSource = "entered" | "converted";

export interface DeathInfo {
  input: DateValue;
  lunar: LunarDay;
  solar: Ymd | null;
  lunar_source: LunarSource;
  method: Method;
  tuoi: Tuoi;
}

export interface BirthInfo {
  input: DateValue;
  lunar_year: number | null;
  lunar_year_approx: boolean;
  can_chi: string | null;
  lunar: LunarDay | null;
  solar: Ymd | null;
}

/** time_local in minutes since 00:00. */
export interface ReminderRule { id: string; enabled: boolean; offsets_days: number[]; time_local: number; tien_thuong: boolean }

export const DEFAULT_OFFSETS: readonly number[] = [0, 3];
export const DEFAULT_TIME = 7 * 60;
export const PRESET_OFFSETS: readonly number[] = [0, 1, 3, 5, 10, 15];
export const CUSTOM_MIN = 1;
export const CUSTOM_MAX = 60;
export const DEFAULT_GIO_HET_NAM = 2;
export const DEFAULT_HUONG_DUONG_DUOI = 60;

export function defaultRule(id: string): ReminderRule {
  return { id, enabled: true, offsets_days: [...DEFAULT_OFFSETS], time_local: DEFAULT_TIME, tien_thuong: false };
}

export type PersonKind = "deceased" | "living";
export type Feb29Rule = "feb28" | "mar1";

export interface Person {
  id: string;
  kind: PersonKind;
  danh_xung: string;
  ho_ten: string;
  birth: BirthInfo | null;
  death: DeathInfo | null;
  gio_het_nam: number;
  huong_duong_duoi: number;
  rule_gio: ReminderRule | null;
  rule_sinh_nhat: ReminderRule | null;
  rule_sinh_nhat_am: ReminderRule | null;
  sinh_nhat_am: boolean;
  feb29_rule: Feb29Rule;
  rev: number;
}

export const displayName = (p: Person): string => (p.danh_xung === "" ? p.ho_ten : `${p.danh_xung} ${p.ho_ten}`);

export interface EventType { id: string; builtin: string | null; ten: string }
export interface Event {
  id: string;
  type_id: string;
  tieu_de: string;
  date: DateValue;
  recurring: boolean;
  person_ids: string[];
  reminder: ReminderRule | null;
  rev: number;
}

export interface Dataset { reminders_enabled: boolean; persons: Person[]; events: Event[]; event_types: EventType[] }

export class ModelError extends Error {}

export function parseHhmm(s: string): number | null {
  if (!/^\d\d:\d\d$/.test(s)) return null;
  const h = Number(s.slice(0, 2)), m = Number(s.slice(3, 5));
  return h < 24 && m < 60 ? h * 60 + m : null;
}
export const fmtHhmm = (t: number): string => `${p2(Math.floor(t / 60))}:${p2(t % 60)}`;

type Obj = { [k: string]: JValue };
function field(v: JValue, k: string): JValue {
  if (v === null || typeof v !== "object" || Array.isArray(v) || !(k in v)) throw new ModelError(`thiếu trường: ${k}`);
  return (v as Obj)[k];
}
const bad = (k: string): never => { throw new ModelError(`trường không hợp lệ: ${k}`); };
function fStr(v: JValue, k: string): string { const x = field(v, k); return typeof x === "string" ? x : bad(k); }
function fBool(v: JValue, k: string): boolean { const x = field(v, k); return typeof x === "boolean" ? x : bad(k); }
function fInt(v: JValue, k: string): number { const x = field(v, k); return typeof x === "number" && Number.isSafeInteger(x) ? x : bad(k); }
function fOptInt(v: JValue, k: string): number | null { const x = field(v, k); return x === null ? null : fInt(v, k); }

export function parseDateValue(v: JValue): DateValue {
  const calendar = fStr(v, "calendar");
  if (calendar !== "solar" && calendar !== "lunar") bad("calendar");
  const precision = fStr(v, "precision");
  if (precision !== "day" && precision !== "month_day" && precision !== "year") bad("precision");
  const u = (x: number | null, k: string) => (x !== null && x < 0 ? bad(k) : x);
  return {
    calendar: calendar as Calendar, precision: precision as Precision,
    year: fOptInt(v, "year"), month: u(fOptInt(v, "month"), "month"), day: u(fOptInt(v, "day"), "day"),
    leap: fBool(v, "leap"),
  };
}

function parseLunarDay(v: JValue): LunarDay {
  return { year: fOptInt(v, "year"), month: fInt(v, "month"), day: fInt(v, "day"), leap: fBool(v, "leap") };
}
function parseYmdOpt(v: JValue): Ymd | null {
  return v === null ? null : { y: fInt(v, "y"), m: fInt(v, "m"), d: fInt(v, "d") };
}

export function parseRule(v: JValue): ReminderRule | null {
  if (v === null) return null;
  const a = field(v, "offsets_days");
  if (!Array.isArray(a)) bad("offsets_days");
  const offs = (a as JValue[]).map((x) => (typeof x === "number" && Number.isSafeInteger(x) && x >= 0 ? x : bad("offsets_days")));
  const t = parseHhmm(fStr(v, "time_local"));
  if (t === null) bad("time_local");
  return { id: fStr(v, "id"), enabled: fBool(v, "enabled"), offsets_days: offs, time_local: t as number, tien_thuong: fBool(v, "tien_thuong") };
}

function parsePerson(v: JValue): Person {
  const kind = fStr(v, "kind");
  if (kind !== "deceased" && kind !== "living") bad("kind");
  const b = field(v, "birth");
  const birth: BirthInfo | null = b === null ? null : {
    input: parseDateValue(field(b, "input")),
    lunar_year: fOptInt(b, "lunar_year"),
    lunar_year_approx: fBool(b, "lunar_year_approx"),
    can_chi: field(b, "can_chi") === null ? null : fStr(b, "can_chi"),
    lunar: field(b, "lunar") === null ? null : parseLunarDay(field(b, "lunar")),
    solar: parseYmdOpt(field(b, "solar")),
  };
  const d = field(v, "death");
  let death: DeathInfo | null = null;
  if (d !== null) {
    const t = field(d, "tuoi");
    const ls = fStr(d, "lunar_source");
    if (ls !== "entered" && ls !== "converted") bad("lunar_source");
    const me = fStr(d, "method");
    if (me !== "precise" && me !== "hnd" && me !== "jpl") bad("method");
    const lb = fStr(t, "label");
    if (lb !== "huong_tho" && lb !== "huong_duong") bad("label");
    death = {
      input: parseDateValue(field(d, "input")), lunar: parseLunarDay(field(d, "lunar")), solar: parseYmdOpt(field(d, "solar")),
      lunar_source: ls as LunarSource, method: me as Method,
      tuoi: { value: fOptInt(t, "value"), label: lb as AgeLabel, manual: fBool(t, "manual"), approx: fBool(t, "approx") },
    };
  }
  const m = field(v, "memorial");
  const r = field(v, "reminders");
  const f29 = fStr(v, "feb29_rule");
  if (f29 !== "feb28" && f29 !== "mar1") bad("feb29_rule");
  return {
    id: fStr(v, "id"), kind: kind as PersonKind, danh_xung: fStr(v, "danh_xung"), ho_ten: fStr(v, "ho_ten"),
    birth, death,
    gio_het_nam: m === null ? DEFAULT_GIO_HET_NAM : fInt(m, "gio_het_nam"),
    huong_duong_duoi: m === null ? DEFAULT_HUONG_DUONG_DUOI : fInt(m, "huong_duong_duoi"),
    rule_gio: parseRule(field(r, "gio")), rule_sinh_nhat: parseRule(field(r, "sinh_nhat")), rule_sinh_nhat_am: parseRule(field(r, "sinh_nhat_am")),
    sinh_nhat_am: fBool(v, "sinh_nhat_am"), feb29_rule: f29 as Feb29Rule, rev: fInt(v, "rev"),
  };
}

function parseEvent(v: JValue): Event {
  const a = field(v, "person_ids");
  if (!Array.isArray(a)) bad("person_ids");
  return {
    id: fStr(v, "id"), type_id: fStr(v, "type_id"), tieu_de: fStr(v, "tieu_de"), date: parseDateValue(field(v, "date")),
    recurring: fBool(v, "recurring"),
    person_ids: (a as JValue[]).map((x) => (typeof x === "string" ? x : bad("person_ids"))),
    reminder: parseRule(field(v, "reminder")), rev: fInt(v, "rev"),
  };
}

export function parseDataset(text: string): Dataset {
  let v: JValue;
  try { v = JSON.parse(text) as JValue; } catch (e) { throw new ModelError(`JSON lỗi: ${String(e)}`); }
  const arr = (k: string): JValue[] => { const x = field(v, k); return Array.isArray(x) ? x : bad(k); };
  return {
    reminders_enabled: fBool(field(v, "settings"), "reminders_enabled"),
    persons: arr("persons").map(parsePerson),
    events: arr("events").map(parseEvent),
    event_types: arr("event_types").map((t) => ({ id: fStr(t, "id"), builtin: field(t, "builtin") === null ? null : fStr(t, "builtin"), ten: fStr(t, "ten") })),
  };
}
