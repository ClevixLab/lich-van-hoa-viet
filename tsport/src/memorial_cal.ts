// Port of memorial/src/cal.rs — per-lunar-year month tables built once from YearTable.

import { MAX_LUNAR_YEAR, MIN_LUNAR_YEAR, jdnInRange } from "./error.ts";
import { dateFromJdn } from "./jd.ts";
import { type Method, type SolarDate, YearTable } from "./lunar.ts";

interface MonthEntry { month: number; leap: boolean; start: number; len: number }

export class Cal {
  readonly method: Method;
  private readonly years: MonthEntry[][];

  private constructor(method: Method) {
    this.method = method;
    const n = MAX_LUNAR_YEAR - MIN_LUNAR_YEAR + 1;
    const years: MonthEntry[][] = Array.from({ length: n }, () => []);
    for (let yy = MIN_LUNAR_YEAR; yy <= MAX_LUNAR_YEAR + 1; yy++) {
      let t: YearTable;
      try { t = new YearTable(method, yy); } catch { continue; }
      for (let i = 0; i < t.n; i++) {
        const ly = t.lunarYear(i);
        if (ly === null || ly < MIN_LUNAR_YEAR || ly > MAX_LUNAR_YEAR) continue;
        years[ly - MIN_LUNAR_YEAR].push({ month: t.month[i], leap: t.leap[i], start: t.starts[i], len: t.starts[i + 1] - t.starts[i] });
      }
    }
    for (const y of years) y.sort((a, b) => a.start - b.start);
    this.years = years;
  }

  private static cache: Partial<Record<Method, Cal>> = {};
  static get(method: Method): Cal {
    return (Cal.cache[method] ??= new Cal(method));
  }

  private entry(ly: number, lm: number, leap: boolean): MonthEntry | null {
    if (!Number.isInteger(ly) || ly < MIN_LUNAR_YEAR || ly > MAX_LUNAR_YEAR) return null;
    return this.years[ly - MIN_LUNAR_YEAR].find((e) => e.month === lm && e.leap === leap) ?? null;
  }

  monthLen(ly: number, lm: number, leap: boolean): number | null {
    return this.entry(ly, lm, leap)?.len ?? null;
  }

  leapMonth(ly: number): number | null {
    if (!Number.isInteger(ly) || ly < MIN_LUNAR_YEAR || ly > MAX_LUNAR_YEAR) return null;
    return this.years[ly - MIN_LUNAR_YEAR].find((e) => e.leap)?.month ?? null;
  }

  l2sJdn(ly: number, lm: number, ld: number, leap: boolean): number | null {
    if (!Number.isInteger(ld) || ld < 1 || ld > 30) return null;
    const e = this.entry(ly, lm, leap);
    if (e === null || ld > e.len) return null;
    const j = e.start + ld - 1;
    return jdnInRange(j) ? j : null;
  }

  l2s(ly: number, lm: number, ld: number, leap: boolean): SolarDate | null {
    const j = this.l2sJdn(ly, lm, ld, leap);
    if (j === null) return null;
    const [y, m, d] = dateFromJdn(j);
    return { y, m, d };
  }
}
