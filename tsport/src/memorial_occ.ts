// Port of memorial/src/occ.rs — occurrences, "Sắp tới" (02 §7.2), texts (02 §7.1, O-9).

import { weekday } from "./jd.ts";
import { solarToLunarWith } from "./lunar.ts";
import type { Cal } from "./memorial_cal.ts";
import { type GioLabel, anniversariesIn, canChiYear, labelText, solarOccurrence, tuoiText } from "./memorial_gio.ts";
import {
  type Dataset, type Event, type Person, type ReminderRule, type Ymd,
  DEFAULT_GIO_HET_NAM, cmpYmd, displayName, iso, ymd, ymdFromJdn, ymdJdn,
} from "./memorial_model.ts";
import { isValidSolar } from "./jd.ts";

export type OccClass = "gio" | "tien_thuong" | "sinh_nhat" | "sinh_nhat_am" | "su_kien_duong_lap" | "su_kien_am_lap" | "su_kien_khong_lap";
/** Declaration order of the Rust enum (derive Ord). */
const CLASS_ORDER: readonly OccClass[] = ["gio", "tien_thuong", "sinh_nhat", "sinh_nhat_am", "su_kien_duong_lap", "su_kien_am_lap", "su_kien_khong_lap"];
export const R3_CLASSES: readonly OccClass[] = ["gio", "sinh_nhat", "sinh_nhat_am", "su_kien_duong_lap", "su_kien_am_lap", "su_kien_khong_lap"];

export function classGroup(c: OccClass): number {
  if (c === "gio") return 0;
  if (c === "tien_thuong") return 1;
  if (c === "sinh_nhat" || c === "sinh_nhat_am") return 2;
  return 3;
}
export function uidKind(c: OccClass): string {
  switch (c) {
    case "gio": return "gio";
    case "tien_thuong": return "tt";
    case "sinh_nhat": return "sn";
    case "sinh_nhat_am": return "sna";
    default: return "ev";
  }
}

/** occ.key: a year number, or "once". */
export type OccKey = number | "once";
export const keyText = (k: OccKey): string => String(k);
function cmpKey(a: OccKey, b: OccKey): number {
  if (a === "once") return b === "once" ? 0 : 1;
  if (b === "once") return -1;
  return a - b;
}

export interface Occurrence {
  class: OccClass;
  entity_id: string;
  key: OccKey;
  date: Ymd;
  n: number | null;
  gio_label: GioLabel | null;
  sort_name: string;
  display: string;
  headline: string;
  rev: number;
}

function personOcc(p: Person, cls: OccClass, key: OccKey, date: Ymd, n: number | null, gioLabel: GioLabel | null): Occurrence {
  const display = displayName(p);
  let headline: string;
  if (cls === "gio") headline = `${labelText(gioLabel ?? "gio", n)} ${display}`;
  else if (cls === "sinh_nhat") headline = n === null ? `Sinh nhật ${display}` : `Sinh nhật ${display} (${n} tuổi)`;
  else if (cls === "sinh_nhat_am") headline = `Sinh nhật âm lịch ${display}`;
  else headline = display;
  return { class: cls, entity_id: p.id, key, date, n, gio_label: gioLabel, sort_name: p.ho_ten, display, headline, rev: p.rev };
}

export function eventTitle(ds: Dataset, ev: Event): string {
  if (ev.tieu_de !== "") return ev.tieu_de;
  return ds.event_types.find((t) => t.id === ev.type_id)?.ten ?? "Sự kiện";
}

function eventOcc(ds: Dataset, ev: Event, cls: OccClass, key: OccKey, date: Ymd, n: number | null): Occurrence {
  const title = eventTitle(ds, ev);
  const isWedding = ds.event_types.find((t) => t.id === ev.type_id)?.builtin === "ngay_cuoi";
  let headline = title;
  if (n !== null && n >= 1) {
    if (isWedding && ev.tieu_de === "") headline = `Kỷ niệm ${n} năm ngày cưới`;
    else if (isWedding) headline = `Kỷ niệm ${n} năm ngày cưới: ${title}`;
    else headline = `${title} (kỷ niệm ${n} năm)`;
  }
  return { class: cls, entity_id: ev.id, key, date, n, gio_label: null, sort_name: title, display: title, headline, rev: ev.rev };
}

export function eventClass(ev: Event): OccClass {
  const rec = ev.recurring || ev.date.precision === "month_day";
  if (ev.date.calendar === "solar" && rec) return "su_kien_duong_lap";
  if (ev.date.calendar === "lunar" && rec) return "su_kien_am_lap";
  return "su_kien_khong_lap";
}

export function personOccurrences(cal: Cal, p: Person, from: Ymd, to: Ymd, out: Occurrence[]): void {
  if (cmpYmd(from, to) > 0) return;
  if (p.kind === "deceased") {
    const d = p.death;
    if (d === null) return;
    for (const a of anniversariesIn(cal, d.lunar.year, d.lunar.month, d.lunar.day, p.gio_het_nam, from, to)) {
      out.push(personOcc(p, "gio", a.year, a.solar, a.n, a.label));
    }
    return;
  }
  const b = p.birth;
  if (b === null) return;
  if (b.solar !== null) {
    const s = b.solar;
    for (let y = Math.max(from.y, s.y + 1); y <= to.y; y++) {
      const o = solarOccurrence(s, y, p.feb29_rule);
      if (o !== null && cmpYmd(from, o) <= 0 && cmpYmd(o, to) <= 0) out.push(personOcc(p, "sinh_nhat", y, o, y - s.y, null));
    }
  }
  if (p.sinh_nhat_am && b.lunar !== null) {
    const l = b.lunar;
    for (const a of anniversariesIn(cal, l.year, l.month, l.day, DEFAULT_GIO_HET_NAM, from, to)) {
      out.push(personOcc(p, "sinh_nhat_am", a.year, a.solar, a.n, null));
    }
  }
}

export function eventOccurrences(cal: Cal, ds: Dataset, ev: Event, from: Ymd, to: Ymd, out: Occurrence[]): void {
  if (cmpYmd(from, to) > 0) return;
  const cls = eventClass(ev);
  const dv = ev.date;
  if (cls === "su_kien_duong_lap") {
    if (dv.year === null || dv.month === null || dv.day === null) return;
    const y0 = dv.year;
    const orig = ymd(y0, dv.month, dv.day);
    for (let y = Math.max(from.y, y0); y <= to.y; y++) {
      const o = solarOccurrence(orig, y, "feb28");
      if (o !== null && cmpYmd(from, o) <= 0 && cmpYmd(o, to) <= 0) out.push(eventOcc(ds, ev, cls, y, o, y - y0));
    }
  } else if (cls === "su_kien_am_lap") {
    if (dv.month === null || dv.day === null) return;
    for (const a of anniversariesIn(cal, dv.year, dv.month, dv.day, DEFAULT_GIO_HET_NAM, from, to)) {
      out.push(eventOcc(ds, ev, cls, a.year, a.solar, a.n));
    }
  } else {
    if (dv.year === null || dv.month === null || dv.day === null) return;
    let date: Ymd;
    if (dv.calendar === "solar") {
      if (!isValidSolar(dv.year, dv.month, dv.day)) return;
      date = ymd(dv.year, dv.month, dv.day);
    } else {
      const s = cal.l2s(dv.year, dv.month, dv.day, dv.leap);
      if (s === null) return;
      date = ymd(s.y, s.m, s.d);
    }
    if (cmpYmd(from, date) <= 0 && cmpYmd(date, to) <= 0) out.push(eventOcc(ds, ev, cls, "once", date, null));
  }
}

/** Code-point order (= UTF-8 byte order), not UTF-16 code-unit order. */
export function cmpCodepoints(a: string, b: string): number {
  if (a === b) return 0;
  const ia = a[Symbol.iterator](), ib = b[Symbol.iterator]();
  for (;;) {
    const x = ia.next(), y = ib.next();
    if (x.done) return y.done ? 0 : -1;
    if (y.done) return 1;
    const cx = x.value.codePointAt(0) as number, cy = y.value.codePointAt(0) as number;
    if (cx !== cy) return cx < cy ? -1 : 1;
  }
}
const cmpAscii = (a: string, b: string) => (a < b ? -1 : a > b ? 1 : 0);

export function cmpOcc(a: Occurrence, b: Occurrence): number {
  return (
    cmpYmd(a.date, b.date) ||
    classGroup(a.class) - classGroup(b.class) ||
    cmpCodepoints(a.sort_name, b.sort_name) ||
    CLASS_ORDER.indexOf(a.class) - CLASS_ORDER.indexOf(b.class) ||
    cmpCodepoints(a.entity_id, b.entity_id) ||
    cmpKey(a.key, b.key)
  );
}
void cmpAscii;

export function occurrences(cal: Cal, ds: Dataset, from: Ymd, to: Ymd): Occurrence[] {
  const out: Occurrence[] = [];
  for (const p of ds.persons) personOccurrences(cal, p, from, to, out);
  for (const ev of ds.events) eventOccurrences(cal, ds, ev, from, to, out);
  out.sort(cmpOcc);
  return out;
}

export function tienThuongOf(g: Occurrence): Occurrence {
  return { ...g, class: "tien_thuong", date: ymdFromJdn(ymdJdn(g.date) - 1), headline: `Tiên thường — ${g.headline}` };
}

export function ruleOf(ds: Dataset, occ: Occurrence): ReminderRule | null {
  switch (occ.class) {
    case "gio": case "tien_thuong": return ds.persons.find((p) => p.id === occ.entity_id)?.rule_gio ?? null;
    case "sinh_nhat": return ds.persons.find((p) => p.id === occ.entity_id)?.rule_sinh_nhat ?? null;
    case "sinh_nhat_am": return ds.persons.find((p) => p.id === occ.entity_id)?.rule_sinh_nhat_am ?? null;
    default: return ds.events.find((e) => e.id === occ.entity_id)?.reminder ?? null;
  }
}

export function occurrencesWithTt(cal: Cal, ds: Dataset, from: Ymd, to: Ymd): Occurrence[] {
  const to1 = ymdFromJdn(ymdJdn(to) + 1);
  const out: Occurrence[] = [];
  for (const o of occurrences(cal, ds, from, to1)) {
    if (o.class === "gio" && ruleOf(ds, o)?.tien_thuong === true) {
      const t = tienThuongOf(o);
      if (cmpYmd(from, t.date) <= 0 && cmpYmd(t.date, to) <= 0) out.push(t);
    }
    if (cmpYmd(o.date, to) <= 0) out.push(o);
  }
  out.sort(cmpOcc);
  return out;
}

export interface UpcomingItem { days: number; occ: Occurrence }

export function upcoming(cal: Cal, ds: Dataset, today: Ymd, days: number): UpcomingItem[] {
  const t0 = ymdJdn(today);
  return occurrencesWithTt(cal, ds, today, ymdFromJdn(t0 + days)).map((occ) => ({ days: ymdJdn(occ.date) - t0, occ }));
}

export function daysText(days: number): string {
  if (days === 0) return "Hôm nay";
  if (days === 1) return "Ngày mai";
  return `Còn ${days} ngày`;
}

export const WEEKDAY_NAMES: readonly string[] = ["Chủ nhật", "Thứ Hai", "Thứ Ba", "Thứ Tư", "Thứ Năm", "Thứ Sáu", "Thứ Bảy"];

export function lunarShort(cal: Cal, d: Ymd): [string, number] | null {
  let l;
  try { l = solarToLunarWith(cal.method, d.y, d.m, d.d); } catch { return null; }
  return [`${l.d}/${l.m}${l.leap ? " nhuận" : ""} âm lịch`, l.y];
}

const p2 = (n: number) => String(n).padStart(2, "0");
export function dateLine(cal: Cal, d: Ymd): string {
  const base = `${WEEKDAY_NAMES[weekday(ymdJdn(d))]}, ${p2(d.d)}/${p2(d.m)}/${String(d.y).padStart(4, "0")}`;
  const l = lunarShort(cal, d);
  return l === null ? base : `${base} — ${l[0]}`;
}

export interface NotifText { title: string; body: string }
/** Reminder type: k ≥ 0 = k days before; "tt" = tiên thường. */
export type RemType = number | "tt";

function lowerGioLabel(l: GioLabel): string {
  return l === "gio_dau" ? "giỗ đầu" : l === "gio_het" ? "giỗ hết" : "giỗ";
}

export function reminderTextNamed(cal: Cal, occ: Occurrence, t: RemType): NotifText {
  if (t === "tt") {
    const ttDay = occ.class === "tien_thuong" ? occ.date : ymdFromJdn(ymdJdn(occ.date) - 1);
    return {
      title: `Hôm nay lễ tiên thường — ngày mai ${lowerGioLabel(occ.gio_label ?? "gio")} ${occ.display}`,
      body: dateLine(cal, ttDay),
    };
  }
  return { title: t === 0 ? `Hôm nay: ${occ.headline}` : `Còn ${t} ngày: ${occ.headline}`, body: dateLine(cal, occ.date) };
}

/** Lock-screen text (O-9): names hidden by default. */
export function notificationText(cal: Cal, occ: Occurrence, t: RemType, showNames: boolean): NotifText {
  if (showNames) return reminderTextNamed(cal, occ, t);
  const g = classGroup(occ.class);
  return { title: g <= 1 ? "Sắp đến ngày giỗ" : g === 2 ? "Sắp đến sinh nhật" : "Sắp đến sự kiện", body: "" };
}

export function description(cal: Cal, ds: Dataset, occ: Occurrence): string {
  const lunar = (d: Ymd) => {
    const l = lunarShort(cal, d);
    return l === null ? "" : `${l[0]} năm ${canChiYear(l[1])} (${l[1]}).`;
  };
  if (occ.class === "gio") {
    let s = lunar(occ.date);
    const d = ds.persons.find((p) => p.id === occ.entity_id)?.death;
    const t = d ? tuoiText(d.tuoi) : null;
    if (t !== null) s += ` ${t}.`;
    return s;
  }
  if (occ.class === "tien_thuong") {
    return `Lễ tiên thường, hôm trước ngày giỗ ${lunar(ymdFromJdn(ymdJdn(occ.date) + 1))}`;
  }
  return lunar(occ.date);
}

export { iso };
