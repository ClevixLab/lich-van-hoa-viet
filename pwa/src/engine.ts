// Giao diện lõi cho PWA. Mọi màn hình chỉ gọi lõi qua mô-đun này.
//
// "#engine-impl" được build.mjs trỏ tới ../../tsport/src/index.ts (lõi thật). Chỉ khi build với cờ
// tường minh `node build.mjs --mock` mới trỏ tới ./engine.mock.ts (giao diện hiện nhãn MOCK);
// thiếu tsport mà không có cờ đó thì build THẤT BẠI (C-10).
//
// Gọi tsport trực tiếp bằng tên (A-4) để `tsc` bắt lỗi đổi tên/kiểu; không dò tên hàm lúc chạy.
//
// Phương pháp âm lịch: hằng DEFAULT_METHOD sinh lúc build (gen/content.ts), sao chép từ hằng
// `DEFAULT_METHOD` của lõi Rust (core/src/lunar.rs) — hiện là "jpl" (bổ sung A4, từ bản 0.1.4). Đổi phương pháp chỉ cần đổi
// hằng của lõi (hoặc biến môi trường LVHV_METHOD khi build); không đổi ở đây.
//
// Lỗi: hàm lõi ném `LvhvError` (RangeError có `code`) cho ngày ngoài 1900–3000 hoặc không hợp lệ —
// giống Rust `Err(DateError)`; trả `null` khi Rust trả `None`.

import * as core from "#engine-impl";
import { CADAO, DEFAULT_METHOD, ENGINE_SOURCE } from "./gen/content";
import { jdn } from "./format";
import type { Activity, Cadao, DayInfo, GoodDay, LunarDate, Method, SolarDate } from "./types";

export type { Activity, Cadao, DayInfo, GoodDay, LunarDate, Method, SolarDate } from "./types";

/** true khi đang dùng lõi giả lập (engine.mock.ts, chỉ với `build.mjs --mock`). */
export const IS_MOCK: boolean = ENGINE_SOURCE === "mock";

/** Phương pháp âm lịch đang dùng (giống mặc định của lõi Rust). */
export const METHOD: Method = DEFAULT_METHOD;

/** Miền ngày công khai của lõi (Rust MIN_YEAR / MAX_YEAR). */
export const MIN_YEAR: number = core.MIN_YEAR;
export const MAX_YEAR: number = core.MAX_YEAR;
/** Năm dương nhỏ nhất của ngày sinh (Rust BIRTH_MIN_YEAR). */
export const BIRTH_MIN_YEAR: number = core.BIRTH_MIN_YEAR;
/** Khoảng tìm ngày tốt tối đa: to − from ≤ MAX_SPAN_DAYS (Rust MAX_SPAN_DAYS). */
export const MAX_SPAN_DAYS: number = core.MAX_SPAN_DAYS;

/** Lỗi miền của lõi (ngoài 1900–3000, ngày không tồn tại, khoảng quá dài…). */
export function isCoreError(e: unknown): e is Error & { code: string } {
  return e instanceof core.LvhvError;
}

export function dayInfo(y: number, m: number, d: number): DayInfo {
  return core.dayInfoWith(METHOD, y, m, d);
}

export function solarToLunar(y: number, m: number, d: number): LunarDate {
  return core.solarToLunarWith(METHOD, y, m, d);
}

export function lunarToSolar(ly: number, lm: number, ld: number, leap: boolean): SolarDate | null {
  return core.lunarToSolarWith(METHOD, ly, lm, ld, leap);
}

/**
 * Ngày tốt cho việc. `birthLunarYear` là NĂM ÂM LỊCH của ngày sinh (dùng `lunarYearOfBirth`),
 * không phải năm dương (C-2). Ném `LvhvError` nếu khoảng sai/quá dài.
 */
export function findGoodDays(activityId: string, from: SolarDate, to: SolarDate, birthLunarYear?: number): GoodDay[] {
  return core.findGoodDaysWith(METHOD, activityId, from, to, birthLunarYear ?? null);
}

/** Năm âm lịch của ngày sinh dương lịch (1900-01-01 … 3000-12-31). Ném `LvhvError` nếu sai. */
export function lunarYearOfBirth(y: number, m: number, d: number): number {
  return core.lunarYearOfBirthWith(METHOD, y, m, d);
}

/** Can Chi của một năm ÂM lịch, ví dụ 1975 → "Ất Mão" (bảng và công thức của lõi, C-11). */
export function lunarYearCanChi(ly: number): string {
  return `${core.canchi.CAN[core.canchi.yearCan(ly)]} ${core.canchi.CHI[core.canchi.yearChi(ly)]}`;
}

/** Mọi Can Chi năm (vòng 60, bắt đầu Giáp Tý) — cho lựa chọn "Tôi chỉ biết năm âm lịch / Can Chi". */
export function sexagenaryCycle(): string[] {
  const out: string[] = [];
  for (let n = 0; n < 60; n++) out.push(`${core.canchi.CAN[n % 10]} ${core.canchi.CHI[n % 12]}`);
  return out;
}

let actsCache: Activity[] | null = null;
export function activities(): Activity[] {
  return (actsCache ??= core.activities());
}

/**
 * Ca dao/tục ngữ trong ngày — quy tắc content/cadao/select.md:
 * pos = (JDN − 2451545) mod N (Euclid), chọn câu có `order == pos`.
 * build.mjs đã sắp CADAO theo `order` và kiểm tra `order` là hoán vị 0..N−1, nên CADAO[pos].order == pos.
 */
export function cadaoFor(y: number, m: number, d: number): Cadao | null {
  const n = CADAO.length;
  if (n === 0) return null;
  const pos = (((jdn(y, m, d) - 2451545) % n) + n) % n;
  const e = CADAO[pos];
  return e && e.order === pos ? e : CADAO.find((c) => c.order === pos) ?? null;
}
