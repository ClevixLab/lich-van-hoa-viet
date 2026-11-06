// Port of core/src/jd.rs — Julian Day Number and proleptic Gregorian conversion.
// All values are integers well within 2^53, so JS numbers are exact. Rust `div_euclid` by a
// positive divisor == Math.floor(a / b); `rem_euclid` by positive n == ((a % n) + n) % n.

/** Rust i64::div_euclid for a positive divisor. */
export function divEuclid(a: number, b: number): number {
  return Math.floor(a / b);
}

/** Rust i64::rem_euclid for a positive divisor. */
export function remEuclid(a: number, n: number): number {
  const r = a % n;
  return r < 0 ? r + n : r;
}

/** Rust f64::round: half away from zero (JS Math.round rounds half toward +inf). */
export function rround(x: number): number {
  const t = Math.trunc(x);
  return Math.abs(x - t) >= 0.5 ? t + Math.sign(x) : t;
}

export function isLeapYear(y: number): boolean {
  return (y % 4 === 0 && y % 100 !== 0) || y % 400 === 0;
}

export function daysInMonth(y: number, m: number): number {
  switch (m) {
    case 1: case 3: case 5: case 7: case 8: case 10: case 12:
      return 31;
    case 4: case 6: case 9: case 11:
      return 30;
    case 2:
      return isLeapYear(y) ? 29 : 28;
    default:
      return 0;
  }
}

export function isValidSolar(y: number, m: number, d: number): boolean {
  return (
    Number.isInteger(y) && Number.isInteger(m) && Number.isInteger(d) &&
    m >= 1 && m <= 12 && d >= 1 && d <= daysInMonth(y, m)
  );
}

export function jdnFromDate(y: number, m: number, d: number): number {
  const a = divEuclid(14 - m, 12);
  const yy = y + 4800 - a;
  const mm = m + 12 * a - 3;
  return (
    d + divEuclid(153 * mm + 2, 5) + 365 * yy + divEuclid(yy, 4) - divEuclid(yy, 100) +
    divEuclid(yy, 400) -
    32045
  );
}

export function dateFromJdn(jdn: number): [number, number, number] {
  const a = jdn + 32044;
  const b = divEuclid(4 * a + 3, 146097);
  const c = a - divEuclid(146097 * b, 4);
  const d = divEuclid(4 * c + 3, 1461);
  const e = c - divEuclid(1461 * d, 4);
  const m = divEuclid(5 * e + 2, 153);
  const day = e - divEuclid(153 * m + 2, 5) + 1;
  const month = m + 3 - 12 * divEuclid(m, 10);
  const year = b * 100 + d - 4800 + divEuclid(m, 10);
  return [year, month, day];
}

/** 0 = Sunday … 6 = Saturday. */
export function weekday(jdn: number): number {
  return remEuclid(jdn + 1, 7);
}

/** JD (UT) → [y, m, d, hh, mm, ss] in zone `tzHours`, seconds rounded (Rust f64::round). */
export function jdToDatetime(jdUt: number, tzHours: number): [number, number, number, number, number, number] {
  const local = jdUt + tzHours / 24.0 + 0.5;
  let day = Math.floor(local);
  let secs = Math.trunc(rround((local - day) * 86400.0));
  if (secs >= 86400) {
    secs -= 86400;
    day += 1.0;
  }
  const [y, m, d] = dateFromJdn(Math.trunc(day));
  return [y, m, d, Math.trunc(secs / 3600), Math.trunc((secs % 3600) / 60), secs % 60];
}

function pad(n: number, w: number): string {
  const s = String(Math.abs(n));
  return (n < 0 ? "-" : "") + (s.length < w ? "0".repeat(w - s.length) + s : s);
}

/** ISO `YYYY-MM-DD` (Rust `{y:04}-{m:02}-{d:02}`). */
export function fmtDate(y: number, m: number, d: number): string {
  return `${pad(y, 4)}-${pad(m, 2)}-${pad(d, 2)}`;
}
