// Port of memorial/src/dump.rs — verification dump lines (docs/v0.2/02 Phụ lục A); byte-identical to Rust.

import { daysInMonth } from "./jd.ts";
import type { Cal } from "./memorial_cal.ts";
import {
  type Anniversary, anniversary, canChiYear, convertDeath, deriveBirth, saveDeathWith, solarOccurrence, tuoiMat,
} from "./memorial_gio.ts";
import type { JValue } from "./memorial_json.ts";
import {
  type BirthInfo, type DateValue, type Dataset, type Feb29Rule, type LunarDay, type ReminderRule, type Ymd,
  CUSTOM_MAX, CUSTOM_MIN, DEFAULT_HUONG_DUONG_DUOI, PRESET_OFFSETS, dateValueJson, dvLunar, dvSolar, dvYear, fmtHhmm, iso,
  parseDateValue, ymd,
} from "./memorial_model.ts";
import { type OccClass, type Occurrence, type RemType, R3_CLASSES, keyText, occurrences, ruleOf, upcoming } from "./memorial_occ.ts";
import { Mulberry32, type Reminder, Zone, dueToday, fmtLocal, localMinutes, remindersFor, simulate, sortReminders } from "./memorial_remind.ts";
import { icsSummary } from "./memorial_ics.ts";
import { reminderTextNamed } from "./memorial_occ.ts";

export function gioLine(out: string[], death: DateValue, a: Anniversary): void {
  out.push(
    `{"death":${dateValueJson(death)},"Y":${a.year},"lunar":{"y":${a.year},"m":${a.month},"d":${a.day}},` +
    `"solar":{"y":${a.solar.y},"m":${a.solar.m},"d":${a.solar.d}},"n":${a.n === null ? "null" : a.n},"label":"${a.label}"}\n`,
  );
}

export function gioLinesFor(cal: Cal, out: string[], death: DateValue, years: number, unknownRange: [number, number], gioHetNam: number): void {
  let d;
  try { d = saveDeathWith(cal, death, null, DEFAULT_HUONG_DUONG_DUOI); } catch { return; }
  const [a, b] = d.lunar.year !== null ? [d.lunar.year + 1, d.lunar.year + years] : unknownRange;
  for (let y = a; y <= b; y++) {
    const x = anniversary(cal, d.lunar.year, d.lunar.month, d.lunar.day, gioHetNam, y);
    if (x !== null) gioLine(out, death, x);
  }
}

export function lunarDaysOfYear(cal: Cal, ly: number): [number, number, boolean][] {
  const v: [number, number, boolean][] = [];
  for (let m = 1; m <= 12; m++) {
    for (const leap of [false, true]) {
      const len = cal.monthLen(ly, m, leap);
      if (len !== null) for (let d = 1; d <= len; d++) v.push([m, d, leap]);
    }
  }
  return v;
}

// ---- R3

export interface UiState { mask: number; custom: number | null }
export function uiPresets(s: UiState): number[] {
  return PRESET_OFFSETS.filter((_, i) => (s.mask & (1 << i)) !== 0);
}
export function uiOffsets(s: UiState): number[] {
  const v = uiPresets(s);
  if (s.custom !== null && !v.includes(s.custom)) v.push(s.custom);
  return v.sort((a, b) => a - b);
}
export function uiStates(): UiState[] {
  const v: UiState[] = [];
  for (let mask = 0; mask < 64; mask++) {
    v.push({ mask, custom: null });
    for (let c = CUSTOM_MIN; c <= CUSTOM_MAX; c++) v.push({ mask, custom: c });
  }
  return v;
}
export const R3_TIMES: readonly number[] = [0, 7 * 60, 23 * 60 + 59];
export interface R3Config { index: number; state: UiState; tien_thuong: boolean; time_local: number; class: OccClass }
export function r3Configs(): R3Config[] {
  const v: R3Config[] = [];
  for (const st of uiStates()) for (const tt of [false, true]) for (const t of R3_TIMES) for (const c of R3_CLASSES) {
    v.push({ index: v.length, state: st, tien_thuong: tt, time_local: t, class: c });
  }
  return v;
}

export const R3_CONFIGS_HEADER = "cfg,presets,custom,offsets,tien_thuong,time_local,class\n";
export function r3ConfigLine(out: string[], c: R3Config): void {
  out.push(`${c.index},${uiPresets(c.state).join(" ")},${c.state.custom ?? ""},${uiOffsets(c.state).join(" ")},${c.tien_thuong ? 1 : 0},${fmtHhmm(c.time_local)},${c.class}\n`);
}
export const rtypeCode = (t: RemType): string => (t === "tt" ? "tt" : `k=${t}`);
export const R3_REMINDERS_HEADER = "cfg,id,at_local,type,occ_date\n";
export function reminderLine(out: string[], cfg: number, r: Reminder): void {
  out.push(`${cfg},${r.id},${fmtLocal(r.at_local)},${rtypeCode(r.rtype)},${iso(r.occ.date)}\n`);
}
export function classOccurrences(cal: Cal, ds: Dataset, cls: OccClass, from: Ymd, to: Ymd): Occurrence[] {
  return occurrences(cal, ds, from, to).filter((o) => o.class === cls);
}
/** R3 dump sub-mode (04 Bổ sung B3 item 2): "normal" | "global_off" | "rule_off". */
export type R3Mode = "normal" | "global_off" | "rule_off";
export function r3Reminders(ds: Dataset, occs: Occurrence[], c: R3Config, now: number, tz: Zone, mode: R3Mode = "normal"): Reminder[] {
  const offs = uiOffsets(c.state);
  const out: Reminder[] = [];
  for (const o of occs) {
    const base = ruleOf(ds, o);
    if (base === null) continue;
    const rule: ReminderRule = { id: base.id, enabled: mode !== "rule_off", offsets_days: offs, time_local: c.time_local, tien_thuong: c.tien_thuong };
    out.push(...remindersFor(mode !== "global_off", rule, o, now, tz));
  }
  sortReminders(out);
  return out;
}

// ---- R11
export const R11_HEADER = "today,rank,days,date,class,entity_id,key,n,label\n";
export function upcomingLines(cal: Cal, ds: Dataset, today: Ymd, out: string[]): void {
  upcoming(cal, ds, today, 30).forEach((it, i) => {
    const o = it.occ;
    out.push(`${iso(today)},${i},${it.days},${iso(o.date)},${o.class},${o.entity_id},${keyText(o.key)},${o.n ?? ""},${o.gio_label ?? ""}\n`);
  });
}

// ---- R10
export const DUE_TODAY_HEADER = "today,rank,at_local,id,type,occ_date,class,entity_id,title\n";
export function dueTodayLines(cal: Cal, ds: Dataset, today: Ymd, tz: Zone, out: string[]): void {
  dueToday(cal, ds, today, tz).forEach((r, i) => {
    const t = reminderTextNamed(cal, r.occ, r.rtype);
    out.push(`${iso(today)},${i},${fmtLocal(r.at_local)},${r.id},${rtypeCode(r.rtype)},${iso(r.occ.date)},${r.occ.class},${r.occ.entity_id},"${t.title.replace(/"/g, '""')}"\n`);
  });
}
export const ICS_SUMMARY_HEADER = "today,events,persons,from,to,kinds,privacy_text,preview\n";
export function icsSummaryLine(cal: Cal, ds: Dataset, today: Ymd, out: string[]): void {
  const s = icsSummary(cal, ds, { today, dtstamp: "", only_ids: null });
  out.push(`${iso(today)},${s.events},${s.persons},${iso(s.from)},${iso(s.to)},${s.kinds.join(" ")},"${s.privacy_text}","${s.preview.join(" | ").replace(/"/g, '""')}"\n`);
}

export const R10A_HEADER = "birth,lunar_year,can_chi\n";
export function r10aLine(cal: Cal, out: string[], d: Ymd): void {
  let b: BirthInfo;
  try { b = deriveBirth(cal, dvSolar(d.y, d.m, d.d)); } catch { return; }
  out.push(`${iso(d)},${b.lunar_year ?? ""},${b.can_chi ?? ""}\n`);
}
export const R10B_HEADER = "lunar_year,can_chi\n";
export function r10bLine(out: string[], y: number): void { out.push(`${y},${canChiYear(y)}\n`); }
export const R10D_HEADER = "year,rule,date\n";
export function r10dLine(out: string[], y: number, rule: Feb29Rule): void {
  const o = solarOccurrence(ymd(1896, 2, 29), y, rule);
  out.push(`${y},${rule},${o === null ? "" : iso(o)}\n`);
}

type InKind = "solar_day" | "solar_year" | "lunar_day" | "lunar_year";
const IN_KINDS: readonly InKind[] = ["solar_day", "solar_year", "lunar_day", "lunar_year"];

function randDate(cal: Cal, rng: Mulberry32, k: InKind, y0: number, y1: number): DateValue {
  const span = y1 - y0 + 1;
  const y = y0 + (rng.nextU32() % span);
  if (k === "solar_day") {
    const m = 1 + (rng.nextU32() % 12);
    const d = 1 + (rng.nextU32() % daysInMonth(y, m));
    return dvSolar(y, m, d);
  }
  if (k === "solar_year") return dvYear("solar", y);
  if (k === "lunar_year") return dvYear("lunar", y);
  const days = lunarDaysOfYear(cal, y);
  const [m, d, leap] = days[rng.nextU32() % days.length];
  return dvLunar(y, m, d, leap);
}

export const R10C_HEADER = "i,birth,death,nguong,birth_lunar_year,approx,death_lunar_year,value,label,error\n";
export function r10cLines(cal: Cal, out: string[], count: number, seed: number): void {
  const rng = new Mulberry32(seed);
  const q = (s: string) => `"${s.replace(/"/g, '""')}"`;
  for (let i = 0; i < count; i++) {
    const bk = IN_KINDS[i % 4];
    const dk: InKind = Math.floor(i / 4) % 2 === 0 ? "solar_day" : "lunar_day";
    const nguong = [59, 60, 61][i % 3];
    const b = randDate(cal, rng, bk, 1900, 2000);
    const by = b.year ?? 1900;
    const dy0 = Math.min(by + (rng.nextU32() % 110), 2999);
    const d = randDate(cal, rng, dk, dy0, dy0);
    let birth: BirthInfo | null = null, berr: string | null = null;
    try { birth = deriveBirth(cal, b); } catch (e) { berr = (e as { code: string }).code; }
    let dl: LunarDay | null = null, derr: string | null = null;
    try { dl = convertDeath(cal, d)[0]; } catch (e) { derr = (e as { code: string }).code; }
    let res: { value: number | null; label: string } | null = null;
    let err: string | null = berr ?? derr;
    if (err === null && birth !== null && dl !== null) {
      if (birth.lunar_year !== null && dl.year !== null && dl.year < birth.lunar_year) err = "DeathBeforeBirth";
      else {
        try { const t = tuoiMat(birth, dl.year, nguong); res = { value: t.value, label: t.label }; } catch (e) { err = (e as { code: string }).code; }
      }
    }
    const bly = birth?.lunar_year ?? null;
    const approx = birth !== null && birth.lunar_year_approx ? 1 : 0;
    const dly = dl?.year ?? null;
    const head = `${i},${q(dateValueJson(b))},${q(dateValueJson(d))},${nguong},${bly ?? ""},${approx},${dly ?? ""},`;
    out.push(res !== null ? `${head}${res.value ?? ""},${res.label},\n` : `${head},,${err}\n`);
  }
}

// ---- R12
export function scaledDataset(base: Dataset, n: number): Dataset {
  const total = base.persons.length + base.events.length;
  const ds: Dataset = { reminders_enabled: base.reminders_enabled, persons: [], events: [], event_types: base.event_types };
  if (total === 0) return ds;
  const re = (id: string, i: number) => i.toString(16).padStart(8, "0") + id.slice(Math.min(8, id.length));
  const rr = (r: ReminderRule | null, i: number) => (r === null ? null : { ...r, id: re(r.id, i) });
  for (let i = 0; i < n; i++) {
    const j = i % total;
    if (j < base.persons.length) {
      const p = base.persons[j];
      ds.persons.push({ ...p, id: re(p.id, i), rule_gio: rr(p.rule_gio, i), rule_sinh_nhat: rr(p.rule_sinh_nhat, i), rule_sinh_nhat_am: rr(p.rule_sinh_nhat_am, i) });
    } else {
      const e = base.events[j - base.persons.length];
      ds.events.push({ ...e, id: re(e.id, i), reminder: rr(e.reminder, i), person_ids: [] });
    }
  }
  return ds;
}

export const R12_HEADER = "size,scenario,opens,max_pending,delivered,sentinel_delivered,violations,missed,missed_without_sentinel,i1,i2,i3,i4\n";
export const R12_STEPS_HEADER = "size,scenario,day,future,pending,sentinel,dropped,violations\n";
export const R12_SCENARIOS: readonly [string, number, number][] = [["p1", 1, 1], ["p0.1", 1, 10], ["p0", 0, 1]];

export function r12Lines(cal: Cal, base: Dataset, sizes: number[], out: string[], steps: string[] | null): void {
  const tz = Zone.vietnam();
  for (const n of sizes) {
    const ds = scaledDataset(base, n);
    for (const [name, pn, pd] of R12_SCENARIOS) {
      const r = simulate(cal, ds, tz, ymd(2026, 1, 1), ymd(2028, 12, 31), 9 * 60, pn, pd, 20261106, false);
      out.push(`${n},${name},${r.opens},${r.max_pending},${r.delivered},${r.sentinel_delivered},${r.violations},${r.missed},${r.missed_without_sentinel},${r.by_inv.join(",")}\n`);
      if (steps !== null) {
        for (const st of r.steps) {
          steps.push(`${n},${name},${iso(st.day)},${st.future},${st.pending},${st.sentinel ? 1 : 0},${st.dropped},${st.violations.join(" ")}\n`);
        }
      }
    }
  }
}

export const r3Now = (tz: Zone): number => tz.localToUtc(localMinutes(ymd(2026, 1, 1), 0));

// ---- derive
export function deriveDocument(cal: Cal, doc: JValue): JValue {
  const copy = JSON.parse(JSON.stringify(doc)) as { [k: string]: JValue };
  const persons = copy.persons;
  if (!Array.isArray(persons)) throw new Error("thiếu persons");
  const lunarJson = (l: LunarDay): JValue => ({ year: l.year, month: l.month, day: l.day, leap: l.leap });
  const ymdJson = (d: Ymd | null): JValue => (d === null ? null : { y: d.y, m: d.m, d: d.d });
  for (const p of persons as { [k: string]: JValue }[]) {
    const mem = p.memorial as { [k: string]: JValue } | null;
    const nguong = mem !== null && typeof mem === "object" && typeof mem.huong_duong_duoi === "number" ? mem.huong_duong_duoi : DEFAULT_HUONG_DUONG_DUOI;
    let bi: BirthInfo | null = null;
    const b = p.birth as { [k: string]: JValue } | null;
    if (b !== null && b !== undefined) {
      bi = deriveBirth(cal, parseDateValue(b.input));
      const set = (k: string, v: JValue) => { if (k in b) b[k] = v; };
      set("lunar_year", bi.lunar_year);
      set("lunar_year_approx", bi.lunar_year_approx);
      set("can_chi", bi.can_chi);
      set("lunar", bi.lunar === null ? null : lunarJson(bi.lunar));
      set("solar", ymdJson(bi.solar));
    }
    const d = p.death as { [k: string]: JValue } | null;
    if (d !== null && d !== undefined) {
      const di = saveDeathWith(cal, parseDateValue(d.input), bi, nguong);
      const set = (k: string, v: JValue) => { if (k in d) d[k] = v; };
      set("lunar", lunarJson(di.lunar));
      set("solar", ymdJson(di.solar));
      set("lunar_source", di.lunar_source);
      set("method", di.method);
      set("tuoi", { value: di.tuoi.value, label: di.tuoi.label, manual: false, approx: di.tuoi.approx });
    }
  }
  return copy;
}
