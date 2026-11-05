// Port of core/src/error.rs — supported domain and the typed error of the public API.
//
// Semantics mirror Rust exactly:
//   Rust `Err(DateError::X)`  ⇔  TS throws `LvhvError` (a RangeError) with `code === "X"` and the
//                                  same Vietnamese message as Rust `Display`;
//   Rust `None`               ⇔  TS returns `null`.
// One TS-only code, "InvalidArgument", covers inputs Rust's type system makes impossible
// (NaN, ±Infinity, non-integer JDN/year, unknown method string, non-boolean `leap` is NOT one of
// these — the lunar functions return null for it, like any other impossible lunar date).

/** Smallest / largest solar year of the public domain (2000-01-01 … 3000-12-31). */
export const MIN_YEAR = 2000;
export const MAX_YEAR = 3000;
/** JDN of 2000-01-01 and 3000-12-31. */
export const MIN_JDN = 2451545;
export const MAX_JDN = 2817152;
/** Lunar years accepted by lunarToSolar (lunar months 11–12 of 1999 fall in Jan–Feb 2000). */
export const MIN_LUNAR_YEAR = MIN_YEAR - 1;
export const MAX_LUNAR_YEAR = MAX_YEAR;
/** Smallest solar year of a birth date (lunarYearOfBirth). 1900–1999 not independently verified. */
export const BIRTH_MIN_YEAR = 1900;
/** findGoodDays: to − from ≤ MAX_SPAN_DAYS. */
export const MAX_SPAN_DAYS = 366;
/** Internal YearTable domain (Rust INTERNAL_MIN_YY / INTERNAL_MAX_YY). */
export const INTERNAL_MIN_YY = 1900;
export const INTERNAL_MAX_YY = 3101;

export type LvhvErrorCode =
  | "InvalidDate"
  | "OutOfRange"
  | "InvertedRange"
  | "RangeTooLong"
  | "UnknownActivity"
  | "Computation"
  | "InvalidArgument";

/** Typed error thrown by every tsport function whose Rust counterpart returns `Result`. */
export class LvhvError extends RangeError {
  readonly code: LvhvErrorCode;
  constructor(code: LvhvErrorCode, message: string) {
    super(message);
    this.name = "LvhvError";
    this.code = code;
  }
}

export function invalidDate(y: unknown, m: unknown, d: unknown): LvhvError {
  return new LvhvError("InvalidDate", `ngày dương không hợp lệ: ${String(y)}-${String(m)}-${String(d)}`);
}

export function outOfRange(min: number = MIN_YEAR, max: number = MAX_YEAR): LvhvError {
  return new LvhvError("OutOfRange", `ngoài phạm vi hỗ trợ (năm ${min} đến năm ${max})`);
}

export function invertedRange(): LvhvError {
  return new LvhvError("InvertedRange", "ngày kết thúc trước ngày bắt đầu");
}

export function rangeTooLong(spanDays: number): LvhvError {
  return new LvhvError("RangeTooLong", `khoảng ngày dài ${spanDays} ngày, tối đa ${MAX_SPAN_DAYS} ngày`);
}

export function unknownActivity(): LvhvError {
  return new LvhvError("UnknownActivity", "mã việc không tồn tại");
}

export function computation(): LvhvError {
  return new LvhvError("Computation", "lỗi tính toán nội bộ của lõi lịch");
}

export function invalidArgument(what: string): LvhvError {
  return new LvhvError("InvalidArgument", `đối số không hợp lệ: ${what}`);
}

export function isSafeInt(v: unknown): v is number {
  return typeof v === "number" && Number.isSafeInteger(v);
}

export function jdnInRange(jdn: number): boolean {
  return isSafeInt(jdn) && jdn >= MIN_JDN && jdn <= MAX_JDN;
}

/** Throws InvalidArgument for a non-safe-integer JDN, OutOfRange outside the public domain. */
export function checkJdn(jdn: unknown): number {
  if (!isSafeInt(jdn)) throw invalidArgument(`JDN ${String(jdn)}`);
  if (!jdnInRange(jdn)) throw outOfRange();
  return jdn;
}
