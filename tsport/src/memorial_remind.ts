// Port of memorial/src/remind.rs — reminders_for (02 §7.1), reschedule / 64-pending rotation (02 §7.4),
// R12 simulation. Instants are minutes (UTC) since JDN 0 (exact in doubles: < 2^33).

import type { Cal } from "./memorial_cal.ts";
import { type Dataset, type ReminderRule, type Ymd, cmpYmd, fmtHhmm, iso, parseHhmm, parseYmd, ymdFromJdn, ymdJdn } from "./memorial_model.ts";
import { type Occurrence, type RemType, eventClass, keyText, notificationText, occurrencesWithTt, ruleOf } from "./memorial_occ.ts";

export class Zone {
  readonly initial: number;
  /** [t_utc, offset] ascending by t_utc. */
  readonly transitions: [number, number][];
  constructor(initial: number, transitions: [number, number][] = []) {
    this.initial = initial;
    this.transitions = transitions;
  }
  static fixed(offsetMin: number): Zone { return new Zone(offsetMin); }
  static vietnam(): Zone { return new Zone(7 * 60); }
  offsetAt(t: number): number {
    let off = this.initial;
    for (const [tt, o] of this.transitions) {
      if (t >= tt) off = o;
      else break;
    }
    return off;
  }
  toLocal(t: number): number { return t + this.offsetAt(t); }
  localToUtc(local: number): number {
    const bounds: [number, number, number][] = [];
    let start = -Infinity, off = this.initial;
    for (const [tt, o] of this.transitions) {
      bounds.push([start, tt, off]);
      start = tt;
      off = o;
    }
    bounds.push([start, Infinity, off]);
    let best: number | null = null;
    for (const [a, b, o] of bounds) {
      const u = local - o;
      if (u >= a && u < b && (best === null || u < best)) best = u;
    }
    if (best !== null) return best;
    for (let i = 0; i + 1 < bounds.length; i++) {
      const [, b, o] = bounds[i];
      const o2 = bounds[i + 1][2];
      if (local >= b + o && local < b + o2) return local - o;
    }
    return local - this.initial;
  }
}

export const localMinutes = (d: Ymd, hhmm: number): number => ymdJdn(d) * 1440 + hhmm;

export function fmtLocal(local: number): string {
  const day = Math.floor(local / 1440);
  return `${iso(ymdFromJdn(day))}T${fmtHhmm(local - day * 1440)}`;
}

export function parseLocal(s: string): number | null {
  if (s.length !== 16 || s[10] !== "T") return null;
  const d = parseYmd(s.slice(0, 10)), t = parseHhmm(s.slice(11));
  return d === null || t === null ? null : localMinutes(d, t);
}

export interface Reminder { id: string; at: number; at_local: number; rtype: RemType; occ: Occurrence }

export function notifId(ruleId: string, occ: Occurrence, t: RemType): string {
  return `lvhv:${ruleId}:${keyText(occ.key)}:${t}`;
}

export function remindersFor(remindersEnabled: boolean, rule: ReminderRule, occ: Occurrence, now: number, tz: Zone): Reminder[] {
  const out: Reminder[] = [];
  if (!remindersEnabled || !rule.enabled) return out;
  const d = ymdJdn(occ.date);
  for (const k of rule.offsets_days) {
    const t = tz.localToUtc(localMinutes(ymdFromJdn(d - k), rule.time_local));
    if (t > now) out.push({ id: notifId(rule.id, occ, k), at: t, at_local: tz.toLocal(t), rtype: k, occ });
  }
  if (rule.tien_thuong && occ.class === "gio") {
    const t = tz.localToUtc(localMinutes(ymdFromJdn(d - 1), rule.time_local));
    if (t > now) out.push({ id: notifId(rule.id, occ, "tt"), at: t, at_local: tz.toLocal(t), rtype: "tt", occ });
  }
  return out;
}

/** Byte-order (= code-point order) string compare, like Rust `as_bytes().cmp`. IDs are ASCII. */
const cmpId = (a: string, b: string) => (a < b ? -1 : a > b ? 1 : 0);

export function sortReminders(v: Reminder[]): void {
  v.sort((a, b) => a.at - b.at || cmpId(a.id, b.id));
}

export function allReminders(cal: Cal, ds: Dataset, from: Ymd, to: Ymd, now: number, tz: Zone): Reminder[] {
  const out: Reminder[] = [];
  for (const o of occurrencesWithTt(cal, ds, from, to)) {
    if (o.class === "tien_thuong") continue;
    const rule = ruleOf(ds, o);
    if (rule !== null) out.push(...remindersFor(ds.reminders_enabled, rule, o, now, tz));
  }
  sortReminders(out);
  return out;
}

export const CAP = 64;
export const WINDOW_DAYS = 400;
export const SENTINEL_ID = "lvhv:sentinel";
export const SENTINEL_TEXT = "Mở Lịch Việt để tiếp tục nhận lời nhắc giỗ và sinh nhật";

export interface Pending { id: string; at: number; content: string }
export interface Plan { keep: Pending[]; remove: string[]; add: Pending[]; replace: Pending[] }

const cmpPending = (a: Pending, b: Pending) => a.at - b.at || cmpId(a.id, b.id);

/** 02 §7.4 (Bổ sung B3): sentinel whenever any future reminder is left unscheduled (cap or beyond window). */
export function selectKeep(desired: Pending[], beyond: boolean, fallbackAt: number): Pending[] {
  if (desired.length > CAP || beyond) {
    const keep = desired.slice(0, Math.min(desired.length, CAP - 1));
    const at = keep.length > 0 ? keep[keep.length - 1].at : fallbackAt;
    keep.push({ id: SENTINEL_ID, at, content: SENTINEL_TEXT });
    return keep;
  }
  return desired.slice();
}

export const BEYOND_SCAN_DAYS = 500;

function onceRemindersAfter(cal: Cal, ds: Dataset, after: Ymd, now: number, tz: Zone): Reminder[] {
  const once: Dataset = { ...ds, persons: [], events: ds.events.filter((e) => eventClass(e) === "su_kien_khong_lap") };
  const from = ymdFromJdn(ymdJdn(after) + 1);
  const end = { y: 3000, m: 12, d: 31 };
  if (once.events.length === 0 || cmpYmd(from, end) > 0) return [];
  return allReminders(cal, once, from, end, now, tz);
}

/** Input `desiredAll` for `reschedule` at `now` (mirror of Rust `reschedule_input`). */
export function rescheduleInput(cal: Cal, ds: Dataset, now: number, tz: Zone): Reminder[] {
  const today = ymdFromJdn(Math.floor(tz.toLocal(now) / 1440));
  const to = ymdFromJdn(ymdJdn(today) + WINDOW_DAYS + BEYOND_SCAN_DAYS);
  const v = allReminders(cal, ds, today, to, now, tz);
  v.push(...onceRemindersAfter(cal, ds, to, now, tz));
  sortReminders(v);
  return v;
}

export const SENTINEL_FALLBACK_HHMM = 9 * 60;
/** 09:00 device time of day `today + 399` (02 §7.4, MA3-14-06). */
export function sentinelFallback(now: number, tz: Zone): number {
  const today = Math.floor(tz.toLocal(now) / 1440);
  return tz.localToUtc((today + WINDOW_DAYS - 1) * 1440 + SENTINEL_FALLBACK_HHMM);
}

export function reschedule(now: number, desiredAll: Pending[], pending: Pending[], tz: Zone): Plan {
  const hi = now + WINDOW_DAYS * 1440;
  const desired = desiredAll.filter((p) => p.at > now && p.at <= hi).sort(cmpPending);
  const keep = selectKeep(desired, desiredAll.some((p) => p.at > hi), sentinelFallback(now, tz));
  const keepIds = new Set(keep.map((k) => k.id));
  const byId = new Map(pending.map((p) => [p.id, p] as const));
  const plan: Plan = { keep, remove: [], add: [], replace: [] };
  for (const p of pending) if (!keepIds.has(p.id)) plan.remove.push(p.id);
  for (const k of keep) {
    const p = byId.get(k.id);
    if (p === undefined) plan.add.push(k);
    else if (p.at !== k.at || p.content !== k.content) plan.replace.push(k);
  }
  return plan;
}

export function applyPlan(pending: Pending[], plan: Plan): Pending[] {
  const rm = new Set(plan.remove);
  const q = pending.filter((p) => !rm.has(p.id)).map((p) => plan.replace.find((r) => r.id === p.id) ?? p);
  q.push(...plan.add);
  return q.sort(cmpPending);
}

export function toPending(cal: Cal, r: Reminder, showNames: boolean): Pending {
  const t = notificationText(cal, r.occ, r.rtype, showNames);
  return { id: r.id, at: r.at, content: t.body === "" ? t.title : `${t.title}\n${t.body}` };
}

export class Mulberry32 {
  state: number;
  constructor(seed: number) { this.state = seed >>> 0; }
  nextU32(): number {
    this.state = (this.state + 0x6d2b79f5) >>> 0;
    let t = this.state;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return (t ^ (t >>> 14)) >>> 0;
  }
}

export interface SimStep { day: Ymd; future: number; pending: number; sentinel: boolean; dropped: number; violations: string[] }
export interface SimResult {
  steps: SimStep[]; opens: number; delivered: number; sentinel_delivered: number; max_pending: number; violations: number; by_inv: number[];
  missed: number; missed_without_sentinel: number;
}

export function simulate(
  cal: Cal, ds: Dataset, tz: Zone, start: Ymd, end: Ymd, openHhmm: number, pNum: number, pDen: number, seed: number, showNames: boolean,
): SimResult {
  const horizonTo = ymdFromJdn(ymdJdn(end) + WINDOW_DAYS + BEYOND_SCAN_DAYS);
  const all = allReminders(cal, ds, start, horizonTo, -Infinity, tz);
  all.push(...onceRemindersAfter(cal, ds, horizonTo, -Infinity, tz));
  sortReminders(all);
  const allP = all.map((r) => toPending(cal, r, showNames));
  const rng = new Mulberry32(seed);
  let pending: Pending[] = [];
  const res: SimResult = { steps: [], opens: 0, delivered: 0, sentinel_delivered: 0, max_pending: 0, violations: 0, by_inv: [0, 0, 0, 0], missed: 0, missed_without_sentinel: 0 };
  const deliveredIds = new Set<string>();
  const sentinelTimes: number[] = [];
  const tStart = tz.localToUtc(localMinutes(start, openHhmm));
  for (let j = ymdJdn(start); j <= ymdJdn(end); j++) {
    const day = ymdFromJdn(j);
    const tOpen = tz.localToUtc(localMinutes(day, openHhmm));
    pending = pending.filter((p) => {
      if (p.at <= tOpen) {
        res.delivered++;
        if (p.id === SENTINEL_ID) { res.sentinel_delivered++; sentinelTimes.push(p.at); }
        else deliveredIds.add(p.id);
        return false;
      }
      return true;
    });
    let opened: boolean;
    if (j === ymdJdn(start)) opened = true;
    else if (pNum === 0) opened = false;
    else if (pNum >= pDen) opened = true;
    else opened = rng.nextU32() * pDen < pNum * 4294967296;
    if (opened) {
      res.opens++;
      const plan = reschedule(tOpen, allP, pending, tz);
      pending = applyPlan(pending, plan);
      const hi = tOpen + WINDOW_DAYS * 1440;
      const fut = allP.filter((p) => p.at > tOpen && p.at <= hi);
      const sentinel = pending.find((p) => p.id === SENTINEL_ID)?.at ?? null;
      const v: string[] = [];
      if (fut.length >= CAP) {
        const ok63 = fut.slice(0, CAP - 1).every((f) => pending.some((p) => p.id === f.id && p.at === f.at));
        const beyond = allP.some((p) => p.at > hi);
        const wantS = fut.length > CAP || beyond ? fut[CAP - 2].at : null;
        if (!ok63 || sentinel !== wantS) v.push("I2");
      }
      const kept = new Set(pending.map((p) => p.id));
      const dropped = fut.filter((f) => !kept.has(f.id));
      if (dropped.some((f) => sentinel === null || f.at < sentinel)) v.push("I3");
      let earliest: number | null = null;
      for (const p of allP) if (p.at > tOpen && !kept.has(p.id) && (earliest === null || p.at < earliest)) earliest = p.at;
      if (earliest !== null && (sentinel === null || sentinel > earliest)) v.push("I4");
      if (pending.length > CAP) v.push("I1");
      for (const c of v) res.by_inv[c === "I1" ? 0 : c === "I2" ? 1 : c === "I3" ? 2 : 3]++;
      res.violations += v.length;
      res.max_pending = Math.max(res.max_pending, pending.length);
      res.steps.push({ day, future: fut.length, pending: pending.length, sentinel: sentinel !== null, dropped: dropped.length, violations: v });
    } else if (pending.length > CAP) {
      res.violations++;
      res.by_inv[0]++;
    }
  }
  const tEnd = tz.localToUtc(localMinutes(end, openHhmm));
  for (const p of allP) {
    if (p.at > tStart && p.at <= tEnd && !deliveredIds.has(p.id)) {
      res.missed++;
      if (!sentinelTimes.some((s) => s <= p.at)) res.missed_without_sentinel++;
    }
  }
  return res;
}
