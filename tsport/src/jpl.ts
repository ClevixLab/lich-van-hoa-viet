// Port of core/src/jpl.rs — method "jpl" (docs/03 §17): new moons and the 24 solar terms from the
// embedded DE441 table (TT, 0.01 s units), ΔT = Table S15.2020 + Hermite bridge + SMH 2016 parabola.
// Every floating-point expression follows the Rust order of operations exactly (bit-identical results).
// Lookups outside the table return null; callers turn that into LvhvError OutOfRange (never a fallback).

import { JPL_EVENTS_B64 } from "./jpl_data.ts";

export const JD_TT_START = 2414928.5;
export const JD_TT_END = 2817548.5;

const S15_2020: readonly (readonly number[])[] = [
  [1895.0, 1900.0, -5.017, 0.134, 3.138, -0.232],
  [1900.0, 1905.0, -1.977, 5.715, 2.443, -1.257],
  [1905.0, 1910.0, 4.923, 6.828, -1.329, 0.720],
  [1910.0, 1915.0, 11.142, 6.330, 0.831, -0.825],
  [1915.0, 1920.0, 17.479, 5.518, -1.643, 0.262],
  [1920.0, 1925.0, 21.617, 3.020, -0.856, 0.008],
  [1925.0, 1930.0, 23.789, 1.333, -0.831, 0.127],
  [1930.0, 1935.0, 24.418, 0.052, -0.449, 0.142],
  [1935.0, 1940.0, 24.164, -0.419, -0.022, 0.702],
  [1940.0, 1945.0, 24.426, 1.645, 2.086, -1.106],
  [1945.0, 1950.0, 27.050, 2.499, -1.232, 0.614],
  [1950.0, 1953.0, 28.932, 1.127, 0.220, -0.277],
  [1953.0, 1956.0, 30.002, 0.737, -0.610, 0.631],
  [1956.0, 1959.0, 30.760, 1.409, 1.282, -0.799],
  [1959.0, 1962.0, 32.652, 1.577, -1.115, 0.507],
  [1962.0, 1965.0, 33.621, 0.868, 0.406, 0.199],
  [1965.0, 1968.0, 35.093, 2.275, 1.002, -0.414],
  [1968.0, 1971.0, 37.956, 3.035, -0.242, 0.202],
  [1971.0, 1974.0, 40.951, 3.157, 0.364, -0.229],
  [1974.0, 1977.0, 44.244, 3.199, -0.323, 0.172],
  [1977.0, 1980.0, 47.291, 3.069, 0.193, -0.192],
  [1980.0, 1983.0, 50.361, 2.878, -0.384, 0.081],
  [1983.0, 1986.0, 52.936, 2.354, -0.140, -0.165],
  [1986.0, 1989.0, 54.984, 1.577, -0.637, 0.448],
  [1989.0, 1992.0, 56.373, 1.648, 0.708, -0.276],
  [1992.0, 1995.0, 58.453, 2.235, -0.121, 0.110],
  [1995.0, 1998.0, 60.678, 2.324, 0.210, -0.313],
  [1998.0, 2001.0, 62.898, 1.804, -0.729, 0.109],
  [2001.0, 2004.0, 64.083, 0.674, -0.402, 0.199],
  [2004.0, 2007.0, 64.553, 0.466, 0.194, -0.017],
  [2007.0, 2010.0, 65.197, 0.804, 0.144, -0.084],
  [2010.0, 2013.0, 66.061, 0.839, -0.109, 0.128],
  [2013.0, 2016.0, 66.920, 1.007, 0.277, -0.095],
  [2016.0, 2019.0, 68.109, 1.277, -0.007, -0.139],
];

const Y_MIN = 1895.0;
const Y_TABLE_END = 2019.0;
const Y_BRIDGE_END = 2800.0;

function parabola(y: number): number {
  const u = (y - 1825.0) / 100.0;
  return -320.0 + 32.5 * u * u;
}

function parabolaSlope(y: number): number {
  return 2.0 * 32.5 * (y - 1825.0) / 10000.0;
}

/** Rust `jpl::decimal_year`. */
export function decimalYear(jdTt: number): number {
  return 2000.0 + (jdTt - 2451544.5) / 365.2425;
}

/** Rust `jpl::delta_t` (seconds); null if y < 1895 or NaN. */
export function deltaTJpl(y: number): number | null {
  if (Number.isNaN(y) || y < Y_MIN) return null;
  if (y < Y_TABLE_END) {
    for (const r of S15_2020) {
      if (r[0] <= y && y < r[1]) {
        const t = (y - r[0]) / (r[1] - r[0]);
        return r[2] + t * (r[3] + t * (r[4] + t * r[5]));
      }
    }
    return null;
  }
  if (y < Y_BRIDGE_END) {
    const r = S15_2020[33];
    const p0 = r[2] + r[3] + r[4] + r[5];
    const m0 = (r[3] + 2.0 * r[4] + 3.0 * r[5]) / (r[1] - r[0]);
    const p1 = parabola(Y_BRIDGE_END);
    const m1 = parabolaSlope(Y_BRIDGE_END);
    const h = Y_BRIDGE_END - Y_TABLE_END;
    const s = (y - Y_TABLE_END) / h;
    const s2 = s * s;
    const s3 = s2 * s;
    const h00 = 2.0 * s3 - 3.0 * s2 + 1.0;
    const h10 = s3 - 2.0 * s2 + s;
    const h01 = -2.0 * s3 + 3.0 * s2;
    const h11 = s3 - s2;
    return h00 * p0 + h10 * h * m0 + h01 * p1 + h11 * h * m1;
  }
  return parabola(y);
}

export function jdTtOf(n: number): number {
  return 2451545.0 + n / 8640000.0;
}

export function jplTtToUt(jdTt: number): number | null {
  const dt = deltaTJpl(decimalYear(jdTt));
  return dt === null ? null : jdTt - dt / 86400.0;
}

export interface JplTable {
  k0: number;
  socCs: number[];
  socUt: number[];
  i0: number;
  tkCs: number[];
  tkUt: number[];
}

const B64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

function b64decode(s: string): Uint8Array {
  const clean = s.replace(/=+$/, "");
  const out = new Uint8Array(Math.floor((clean.length * 3) / 4));
  let acc = 0;
  let bits = 0;
  let o = 0;
  for (let i = 0; i < clean.length; i++) {
    const v = B64.indexOf(clean[i]);
    if (v < 0) throw new Error("jpl_data: base64 hỏng");
    acc = (acc << 6) | v;
    bits += 6;
    if (bits >= 8) {
      bits -= 8;
      out[o++] = (acc >> bits) & 0xff;
    }
  }
  return out;
}

function decode(b: Uint8Array): JplTable | null {
  const magic = "LVHVJPL1";
  for (let i = 0; i < 8; i++) if (b[i] !== magic.charCodeAt(i)) return null;
  let pos = 8;
  // zigzag varint → exact integer (|value| < 2^53); no 32-bit bitwise ops on large values.
  const zz = (): number => {
    let u = 0;
    let mul = 1;
    for (;;) {
      if (pos >= b.length) throw new Error("jpl_data: hết dữ liệu");
      const c = b[pos++];
      u += (c & 0x7f) * mul;
      mul *= 128;
      if (c < 0x80) break;
    }
    return u % 2 === 1 ? -(u + 1) / 2 : u / 2;
  };
  const k0 = zz();
  const ns = zz();
  const i0 = zz();
  const nt = zz();
  if (i0 < 0 || i0 > 23 || ns < 3 || nt < 49) return null;
  const soc: number[] = [];
  for (let j = 0; j < ns; j++) {
    const e = zz();
    soc.push(j < 3 ? e : e + 3 * soc[j - 1] - 3 * soc[j - 2] + soc[j - 3]);
  }
  const tk: number[] = [];
  let rPrev = 0;
  for (let j = 0; j < nt; j++) {
    const e = zz();
    const rr = j <= 48 ? e : e + rPrev;
    rPrev = rr;
    tk.push(j < 48 ? rr : rr + 2 * tk[j - 24] - tk[j - 48]);
  }
  if (pos !== b.length) return null;
  const ut = (v: number[]): number[] | null => {
    const o: number[] = [];
    for (const n of v) {
      const x = jplTtToUt(jdTtOf(n));
      if (x === null) return null;
      o.push(x);
    }
    return o;
  };
  const socUt = ut(soc);
  const tkUt = ut(tk);
  if (socUt === null || tkUt === null) return null;
  for (let i = 1; i < socUt.length; i++) if (!(socUt[i] > socUt[i - 1])) return null;
  for (let i = 1; i < tkUt.length; i++) if (!(tkUt[i] > tkUt[i - 1])) return null;
  return { k0, socCs: soc, socUt, i0, tkCs: tk, tkUt };
}

let cached: JplTable | null | undefined;

/** Decoded table (decoded on first use); null only if the embedded data is corrupt. */
export function jplTable(): JplTable | null {
  if (cached === undefined) cached = decode(b64decode(JPL_EVENTS_B64));
  return cached;
}

/** Rust `jpl::new_moon_ut`: JD UT of new moon k (HND index), null outside the table. */
export function jplNewMoonUt(k: number): number | null {
  const t = jplTable();
  if (t === null) return null;
  const j = k - t.k0;
  if (!Number.isSafeInteger(j) || j < 0 || j >= t.socUt.length) return null;
  return t.socUt[j];
}

/** Number of elements x of the sorted array with pred(x) true (Rust `partition_point`). */
function partitionPoint(a: number[], pred: (x: number) => boolean): number {
  let lo = 0;
  let hi = a.length;
  while (lo < hi) {
    const mid = (lo + hi) >>> 1;
    if (pred(a[mid])) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

/** Rust `jpl::term_at`. */
export function jplTermAt(jdUt: number): number | null {
  const t = jplTable();
  if (t === null) return null;
  const last = t.tkUt[t.tkUt.length - 1];
  if (Number.isNaN(jdUt) || jdUt < t.tkUt[0] || jdUt >= last) return null;
  const cnt = partitionPoint(t.tkUt, (x) => x <= jdUt);
  return (t.i0 + cnt - 1) % 24;
}

/** Rust `jpl::terms_between`: [index, jdUt] with jdUt in [fromUt, toUt). */
export function jplTermsBetween(fromUt: number, toUt: number): [number, number][] | null {
  const t = jplTable();
  if (t === null) return null;
  const a = partitionPoint(t.tkUt, (x) => x < fromUt);
  const b = partitionPoint(t.tkUt, (x) => x < toUt);
  const out: [number, number][] = [];
  for (let j = a; j < b; j++) out.push([(t.i0 + j) % 24, t.tkUt[j]]);
  return out;
}
