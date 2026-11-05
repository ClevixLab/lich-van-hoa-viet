// Port of the `lvhv-ngochap` crate (ngochap/src/lib.rs). The tables are generated from the Rust
// source into ngochap_tables.ts (never edited by hand); this file ports the logic faithfully:
// rule_hits, stars_of, score_activity, evaluate, suitability, major_taboos.
// Also the JSON writer for `ngoc_hap` (Rust `write_ngoc_hap_json`).

import { writeJsonStr } from "./json.ts";
import {
  ACTIVITIES, BAD_STARS, CAN_NAME, CHI_NAME, GOOD_STARS, NEN_THRESHOLD, PERSONAL, SPECIAL_DAYS,
  THAN, THAP_AC, TRUC_NAME, TRUC_SCORE, XUNG_PENALTY, XUNG_TUOI,
} from "./ngochap_tables.ts";

export { NEN_THRESHOLD, XUNG_PENALTY };

/** Rust `lvhv_ngochap::DayCtx` (all small integers). */
export interface NgocHapCtx {
  lunar_month: number;
  lunar_day: number;
  leap: boolean;
  day_can: number;
  day_chi: number;
  month_chi_tiet: number;
  year_can: number;
  year_chi: number;
  truc: number;
  tu: number;
  than: number;
  solar_month: number;
  solar_day: number;
}

export interface Activity { readonly id: string; readonly name: string }

export interface NgocHap {
  sao_tot: string[];
  sao_xau: string[];
  nen: string[];
  ky: string[];
  xung_tuoi: string[];
  score: number;
}

export interface Suitability { allowed: boolean; score: number; reasons: string[] }

// ---- table types (shape produced by the generator) ----
export type Acts =
  | { readonly k: "none" }
  | { readonly k: "all" }
  | { readonly k: "allEsp"; readonly l: readonly number[] }
  | { readonly k: "some"; readonly l: readonly number[] };

type Pair = readonly [number, number];
export type Rule =
  | { readonly k: "monthChi"; readonly t: readonly number[] }
  | { readonly k: "monthChi2"; readonly t: ReadonlyArray<readonly number[]> }
  | { readonly k: "monthCan"; readonly t: readonly number[] }
  | { readonly k: "monthCond"; readonly t: ReadonlyArray<{ readonly can?: number; readonly chi?: number }> }
  | { readonly k: "monthCanChi"; readonly t: readonly Pair[] }
  | { readonly k: "monthChiOpt"; readonly t: ReadonlyArray<number | null> }
  | { readonly k: "seasonChi"; readonly t: ReadonlyArray<readonly number[]> }
  | { readonly k: "seasonCan"; readonly t: ReadonlyArray<readonly number[]> }
  | { readonly k: "seasonCanChi"; readonly t: readonly Pair[] }
  | { readonly k: "canChiSet"; readonly t: readonly Pair[] }
  | { readonly k: "canChiByGroup"; readonly t: ReadonlyArray<readonly Pair[]> }
  | { readonly k: "lunarDays"; readonly t: readonly number[] }
  | { readonly k: "monthDays"; readonly t: readonly Pair[] }
  | { readonly k: "thapAc" }
  | { readonly k: "than"; readonly i: number };

export interface Star {
  readonly id: string;
  readonly name: string;
  readonly good: boolean;
  readonly weight: number;
  readonly rule: Rule;
  readonly tot: Acts;
  readonly ky: Acts;
  readonly major: Acts;
}

function actsContains(a: Acts, x: number): boolean {
  switch (a.k) {
    case "none": return false;
    case "all": case "allEsp": return true;
    case "some": return a.l.includes(x);
  }
}

/** Activity named explicitly (not just "mọi việc"). */
function actsNames(a: Acts, x: number): boolean {
  return (a.k === "some" || a.k === "allEsp") && a.l.includes(x);
}

const pairIn = (t: readonly Pair[], c: number, z: number) => t.some((p) => p[0] === c && p[1] === z);

/** Activities in fixed order. */
export function activities(): Activity[] {
  return ACTIVITIES.map((a) => ({ id: a.id, name: a.name }));
}

function monthIdx(ctx: NgocHapCtx): number {
  return Math.min(12, Math.max(1, ctx.lunar_month)) - 1;
}

function season(ctx: NgocHapCtx): number {
  return Math.floor(monthIdx(ctx) / 3);
}

function sexagenary(can: number, chi: number): number | null {
  const c = can % 10, z = chi % 12;
  if (c % 2 !== z % 2) return null;
  for (let n = 0; n < 60; n++) if (n % 10 === c && n % 12 === z) return n;
  return null;
}

function ruleHits(rule: Rule, ctx: NgocHapCtx): boolean {
  const m = monthIdx(ctx);
  const can = ctx.day_can % 10;
  const chi = ctx.day_chi % 12;
  switch (rule.k) {
    case "monthChi": return rule.t[m] === chi;
    case "monthChi2": return rule.t[m].includes(chi);
    case "monthCan": return rule.t[m] === can;
    case "monthCond": {
      const c = rule.t[m];
      return c.can !== undefined ? c.can === can : c.chi === chi;
    }
    case "monthCanChi": return rule.t[m][0] === can && rule.t[m][1] === chi;
    case "monthChiOpt": return rule.t[m] === chi;
    case "seasonChi": return rule.t[season(ctx)].includes(chi);
    case "seasonCan": return rule.t[season(ctx)].includes(can);
    case "seasonCanChi": return rule.t[season(ctx)][0] === can && rule.t[season(ctx)][1] === chi;
    case "canChiSet": return pairIn(rule.t, can, chi);
    case "canChiByGroup": return pairIn(rule.t[m % 3], can, chi);
    case "lunarDays": return rule.t.includes(ctx.lunar_day);
    case "monthDays": return pairIn(rule.t, m + 1, ctx.lunar_day);
    case "thapAc": {
      const grp = ctx.year_can % 5;
      return THAP_AC.some(([g, mo, c, z]) => g === grp && mo === m + 1 && c === can && z === chi);
    }
    case "than": return ctx.than % 12 === rule.i;
  }
}

/** (good stars, bad stars) of the day, fixed order (Rust `stars_of`). */
function starsOf(ctx: NgocHapCtx): [Star[], Star[]] {
  let good = GOOD_STARS.filter((s) => ruleHits(s.rule, ctx));
  const bad = SPECIAL_DAYS.concat(BAD_STARS).filter((s) => ruleHits(s.rule, ctx));
  // Hoạt Điệu (G30) loses effect with Thụ Tử (B14) — source N01.
  if (bad.some((s) => s.id === "B14")) good = good.filter((s) => s.id !== "G30");
  const than = THAN[ctx.than % 12];
  if (than.good) good.push(than); else bad.push(than);
  return [good, bad];
}

function xungList(can: number, chi: number): ReadonlyArray<Pair> {
  const n = sexagenary(can, chi);
  return n === null ? [] : XUNG_TUOI[n];
}

function canChiName(can: number, chi: number): string {
  return `${CAN_NAME[can % 10]} ${CHI_NAME[chi % 12]}`;
}

const remEuclid = (a: number, n: number) => ((a % n) + n) % n;

/** Can Chi of a (lunar) birth year: can = (Y+6) mod 10, chi = (Y+8) mod 12. */
function yearCanChi(y: number): Pair {
  return [remEuclid(y + 6, 10), remEuclid(y + 8, 12)];
}

function scoreActivity(ctx: NgocHapCtx, good: readonly Star[], bad: readonly Star[], a: number): [number, boolean, string[]] {
  let score = 0;
  let major = false;
  const reasons: string[] = [];
  for (const s of bad) {
    if (actsContains(s.major, a)) {
      major = true;
      reasons.push(`Phạm sao kỵ chính: ${s.name}`);
    }
  }
  for (const s of good) {
    if (actsContains(s.ky, a)) {
      score -= s.weight;
      reasons.push(`−${s.weight} ${s.name} (kỵ việc này)`);
    } else if (actsContains(s.tot, a)) {
      const w = s.weight + (actsNames(s.tot, a) ? 1 : 0);
      score += w;
      reasons.push(`+${w} ${s.name}`);
    }
  }
  for (const s of bad) {
    if (actsContains(s.ky, a)) {
      const w = s.weight + (actsNames(s.ky, a) ? 1 : 0);
      score -= w;
      reasons.push(`−${w} ${s.name}`);
    }
  }
  const t = TRUC_SCORE[ctx.truc % 12];
  if (t !== 0) {
    reasons.push(`${t > 0 ? "+" : "−"}${Math.abs(t)} Trực ${TRUC_NAME[ctx.truc % 12]}`);
    score += t;
  }
  return [score, major, reasons];
}

const inU8 = (v: unknown, lo: number, hi: number) => typeof v === "number" && Number.isInteger(v) && v >= lo && v <= hi;

/** Rust `lvhv_ngochap::DayCtx::validate().is_ok()` (C-4): every field inside its domain. */
export function validateCtx(c: NgocHapCtx): boolean {
  return (
    inU8(c.lunar_month, 1, 12) && inU8(c.lunar_day, 1, 30) && typeof c.leap === "boolean" &&
    inU8(c.day_can, 0, 9) && inU8(c.year_can, 0, 9) &&
    inU8(c.day_chi, 0, 11) && inU8(c.year_chi, 0, 11) && inU8(c.month_chi_tiet, 0, 11) &&
    c.day_can % 2 === c.day_chi % 2 && c.year_can % 2 === c.year_chi % 2 &&
    inU8(c.truc, 0, 11) && inU8(c.than, 0, 11) && inU8(c.tu, 0, 27) &&
    inU8(c.solar_month, 1, 12) && inU8(c.solar_day, 1, 31)
  );
}

/** Rust `lvhv_ngochap::evaluate`. */
export function evaluate(ctx: NgocHapCtx): NgocHap {
  const [good, bad] = starsOf(ctx);
  const nen: string[] = [];
  const ky: string[] = [];
  ACTIVITIES.forEach((act, a) => {
    const [sc, major] = scoreActivity(ctx, good, bad, a);
    const namedBad = bad.some((s) => actsNames(s.ky, a));
    if (major || (namedBad && sc < 0)) ky.push(act.name);
    else if (!namedBad && sc >= NEN_THRESHOLD) nen.push(act.name);
  });
  let score = TRUC_SCORE[ctx.truc % 12];
  for (const s of good) score += s.weight;
  for (const s of bad) score -= s.weight;
  return {
    sao_tot: good.map((s) => s.name),
    sao_xau: bad.map((s) => s.name),
    nen,
    ky,
    xung_tuoi: xungList(ctx.day_can, ctx.day_chi).map(([c, z]) => canChiName(c, z)),
    score,
  };
}

/** Rust `lvhv_ngochap::suitability`. `birthLunarYear`: LUNAR year of the birth date (C-2). */
export function suitability(ctx: NgocHapCtx, activityId: string, birthLunarYear?: number | null): Suitability {
  const a = ACTIVITIES.findIndex((x) => x.id === activityId);
  if (a < 0) return { allowed: false, score: 0, reasons: [`Không có việc mã "${activityId}"`] };
  const [good, bad] = starsOf(ctx);
  let [score, major, reasons] = scoreActivity(ctx, good, bad, a);
  let allowed = !major;
  if (birthLunarYear !== undefined && birthLunarYear !== null) {
    const [c, z] = yearCanChi(birthLunarYear);
    if (pairIn(xungList(ctx.day_can, ctx.day_chi), c, z)) {
      score -= XUNG_PENALTY;
      const personal = PERSONAL.includes(a);
      if (personal) allowed = false;
      reasons.push(`−${XUNG_PENALTY} Tuổi ${canChiName(c, z)} xung với ngày ${canChiName(ctx.day_can, ctx.day_chi)}${personal ? " (không nên chọn)" : ""}`);
    }
  }
  return { allowed, score, reasons };
}

/** Rust `lvhv_ngochap::major_taboos` (N3 definition). */
export function majorTaboos(ctx: NgocHapCtx, activityId: string): string[] {
  const a = ACTIVITIES.findIndex((x) => x.id === activityId);
  if (a < 0) return [];
  return starsOf(ctx)[1].filter((s) => actsContains(s.major, a)).map((s) => s.name);
}

function writeStrArray(out: string[], v: readonly string[]): void {
  out.push("[");
  for (let i = 0; i < v.length; i++) {
    if (i > 0) out.push(",");
    writeJsonStr(out, v[i]);
  }
  out.push("]");
}

/** Port of Rust `write_ngoc_hap_json`. */
export function writeNgocHapJson(out: string[], n: NgocHap): void {
  out.push('{"sao_tot":');
  writeStrArray(out, n.sao_tot);
  out.push(',"sao_xau":');
  writeStrArray(out, n.sao_xau);
  out.push(',"nen":');
  writeStrArray(out, n.nen);
  out.push(',"ky":');
  writeStrArray(out, n.ky);
  out.push(',"xung_tuoi":');
  writeStrArray(out, n.xung_tuoi);
  out.push(',"score":');
  out.push(String(n.score | 0));
  out.push("}");
}
