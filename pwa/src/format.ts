// JDN lấy từ lõi (tsport) — không cài lại lần thứ ba (C-11). Can Chi năm: engine.lunarYearCanChi.
import { jd as coreJd } from "#engine-impl";
import type { GioHoangDao, SolarDate } from "./types";

export const WEEKDAY = ["Chủ nhật", "Thứ Hai", "Thứ Ba", "Thứ Tư", "Thứ Năm", "Thứ Sáu", "Thứ Bảy"];
export const WEEKDAY_SHORT = ["CN", "T2", "T3", "T4", "T5", "T6", "T7"];
export const LUNAR_MONTH_NAME = ["", "Giêng", "Hai", "Ba", "Tư", "Năm", "Sáu", "Bảy", "Tám", "Chín", "Mười", "Mười Một", "Chạp"];
export function jdn(y: number, m: number, d: number): number {
  return coreJd.jdnFromDate(y, m, d);
}

export function fromJdn(j: number): SolarDate {
  const [y, m, d] = coreJd.dateFromJdn(j);
  return { y, m, d };
}

export const weekdayOf = (s: SolarDate) => coreJd.weekday(jdn(s.y, s.m, s.d));

export function addDays(s: SolarDate, n: number): SolarDate {
  return fromJdn(jdn(s.y, s.m, s.d) + n);
}

export function daysInMonth(y: number, m: number): number {
  return new Date(Date.UTC(y, m, 0)).getUTCDate();
}

export function todayLocal(): SolarDate {
  const t = new Date();
  return { y: t.getFullYear(), m: t.getMonth() + 1, d: t.getDate() };
}

export const sameDay = (a: SolarDate, b: SolarDate) => a.y === b.y && a.m === b.m && a.d === b.d;

const p2 = (n: number) => String(n).padStart(2, "0");
export const isoDate = (s: SolarDate) => `${String(s.y).padStart(4, "0")}-${p2(s.m)}-${p2(s.d)}`;
export const dmy = (s: SolarDate) => `${p2(s.d)}/${p2(s.m)}/${s.y}`;

export function parseIso(v: string): SolarDate | null {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(v);
  if (!m) return null;
  const s = { y: +m[1], m: +m[2], d: +m[3] };
  if (s.m < 1 || s.m > 12 || s.d < 1 || s.d > daysInMonth(s.y, s.m)) return null;
  return s;
}

export function inRange(s: SolarDate, minY: number, maxY: number): boolean {
  return s.y >= minY && s.y <= maxY;
}

export function hourRange(g: GioHoangDao): string {
  return `${g.from}–${g.to} giờ`;
}

/**
 * Nhãn đánh giá từ điểm chung của ngày (lvhv-ngochap, SCORING.md §3: Σ sao tốt − Σ sao xấu + trực).
 * Điểm thực tế nằm trong khoảng −23…+13 (2000–2099: trung vị −1; 11% ngày ≥ 5; 26% ngày ≤ −4).
 * Ngưỡng là quy ước giao diện (không phải quy tắc truyền thống), cần chủ dự án duyệt.
 */
export function rating(score: number): { label: string; cls: string } {
  if (score >= 4) return { label: "Tốt", cls: "r-good" };
  if (score >= 1) return { label: "Khá tốt", cls: "r-fair" };
  if (score >= -3) return { label: "Bình thường", cls: "r-avg" };
  return { label: "Xấu", cls: "r-bad" };
}

/** Điểm có dấu (dấu trừ U+2212 như trong lý do của lõi). */
export function signed(n: number): string {
  return n > 0 ? `+${n}` : n < 0 ? `−${-n}` : "0";
}

/** "tục ngữ" → "Tục ngữ" (nhãn hiển thị cho trường `type` của cadao.json). */
export function capFirst(s: string): string {
  return s ? s.charAt(0).toUpperCase() + s.slice(1) : s;
}
