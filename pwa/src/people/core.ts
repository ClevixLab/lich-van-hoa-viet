// Dữ liệu và cách diễn đạt của mô-đun Người thân (docs/v0.2/08). Mọi tính toán giỗ/sinh nhật/nhắc/.ics gọi lõi
// tsport (memorial*), không viết lại thuật toán ở giao diện (08 §8.1). Mô-đun này chỉ được nạp lười.
import * as M from "../../../tsport/src/memorial.ts";
import { DEFAULT_METHOD } from "../gen/content";
import { WEEKDAY, WEEKDAY_SHORT } from "../format";
import { openStore, type Store, type JsonObj } from "../store/index";

export { M };
export type { JsonObj, Store };
export type Ymd = M.Ymd;

let storeP: Promise<Store> | null = null;
/** Một kết nối kho dùng chung cho cả mô-đun. */
export function store(): Promise<Store> {
  return (storeP ??= openStore().catch((e) => { storeP = null; throw e; }));
}

export const cal = (): M.Cal => M.Cal.get(DEFAULT_METHOD);

export function today(): Ymd {
  const t = new Date();
  return { y: t.getFullYear(), m: t.getMonth() + 1, d: t.getDate() };
}

export interface Snap {
  persons: JsonObj[];
  events: JsonObj[];
  types: JsonObj[];
  settings: JsonObj;
  ds: M.Dataset;
  byId: Map<string, JsonObj>;
}

/** Ảnh chụp toàn kho + bộ dữ liệu của lõi (01 §5 → memorial_model.parseDataset). */
export async function snap(): Promise<Snap> {
  const s = await store();
  const [persons, events, types, settings] = await Promise.all([s.persons(), s.events(), s.eventTypes(), s.settings()]);
  const ds = M.parseDataset(JSON.stringify({ settings, persons, events, event_types: types }));
  return { persons, events, types, settings, ds, byId: new Map(persons.map((p) => [p.id as string, p])) };
}

// ---------------- định dạng ----------------
const p2 = (n: number) => String(n).padStart(2, "0");
export const dmy = (d: Ymd) => `${p2(d.d)}/${p2(d.m)}/${d.y}`;
export const dm = (d: Ymd) => `${p2(d.d)}/${p2(d.m)}`;
export const weekdayOf = (d: Ymd) => (M.ymdJdn(d) + 1) % 7;
export const weekdayName = (d: Ymd) => WEEKDAY[weekdayOf(d)];
export const weekdayShort = (d: Ymd) => WEEKDAY_SHORT[weekdayOf(d)];
export const daysBetween = (a: Ymd, b: Ymd) => M.ymdJdn(b) - M.ymdJdn(a);
export const addDays = (a: Ymd, n: number) => M.ymdFromJdn(M.ymdJdn(a) + n);

/** "Hôm nay", "Ngày mai", "Còn n ngày", "Hôm qua", "Đã qua n ngày". */
export function relText(days: number): string {
  if (days === -1) return "Hôm qua";
  if (days < 0) return `Đã qua ${-days} ngày`;
  return M.daysText(days);
}

export function lunarText(l: { year: number | null; month: number; day: number; leap: boolean }, withYear = true, suffix = true): string {
  const base = `${l.day}/${l.month}${l.leap ? " nhuận" : ""}${suffix ? " âm lịch" : ""}`;
  return withYear && l.year !== null ? `${base} năm ${M.canChiYear(l.year)} (${l.year})` : base;
}

/** Ngày âm ngắn của một ngày dương ("15/6 âm lịch"), "" nếu ngoài miền. */
export function lunarOf(d: Ymd): string {
  return M.lunarShort(cal(), d)?.[0] ?? "";
}

export function displayName(p: JsonObj): string {
  const dx = (p.danh_xung as string) ?? "";
  return dx ? `${dx} ${p.ho_ten as string}` : (p.ho_ten as string);
}

export type ItemKind = "gio" | "tt" | "sn" | "ev";
export function itemKind(o: M.Occurrence): ItemKind {
  if (o.class === "gio") return "gio";
  if (o.class === "tien_thuong") return "tt";
  if (o.class === "sinh_nhat" || o.class === "sinh_nhat_am") return "sn";
  return "ev";
}

/** Nhãn của một lần xuất hiện (08 §5.1). */
export function occLabel(o: M.Occurrence): string {
  switch (o.class) {
    case "gio":
      if (o.gio_label === "gio" && o.n !== null) return `Giỗ lần thứ ${o.n}`;
      return M.labelText(o.gio_label ?? "gio", o.n);
    case "tien_thuong": {
      const g = o.gio_label === "gio_dau" ? "giỗ đầu" : o.gio_label === "gio_het" ? "giỗ hết" : "giỗ";
      return `Tiên thường (hôm trước ${g})`;
    }
    case "sinh_nhat": return o.n !== null ? `Sinh nhật thứ ${o.n}` : "Sinh nhật";
    case "sinh_nhat_am": return o.n !== null ? `Sinh nhật âm lịch (tuổi âm ${o.n + 1})` : "Sinh nhật âm lịch";
    default: return o.headline;
  }
}

/** Hồ sơ "chủ" của một lần xuất hiện (sự kiện → người đầu tiên). */
export function ownerOf(sn: Snap, o: M.Occurrence): JsonObj | null {
  if (o.class === "gio" || o.class === "tien_thuong" || o.class === "sinh_nhat" || o.class === "sinh_nhat_am") return sn.byId.get(o.entity_id) ?? null;
  const ev = sn.events.find((e) => e.id === o.entity_id);
  const pid = (ev?.person_ids as string[] | undefined)?.[0];
  return pid ? sn.byId.get(pid) ?? null : null;
}

export function tuoiText(t: JsonObj | null | undefined): string | null {
  if (!t || t.value === null || t.value === undefined) return null;
  return `${t.label === "huong_duong" ? "Hưởng dương" : "Hưởng thọ"} ${t.value} tuổi${t.approx ? " (khoảng)" : ""}`;
}

export const uuid = () => crypto.randomUUID();
export const nowTs = () => new Date().toISOString().slice(0, 19) + "Z";

/** Chuẩn hóa chuỗi người dùng gõ (01 §1): NFC, bỏ ký tự điều khiển (giữ \n nếu nhiều dòng), cắt khoảng trắng. */
export function clean(s: string, multiline = false): string {
  // eslint-disable-next-line no-control-regex
  const re = multiline ? /[\u0000-\u0009\u000b-\u001f\u007f-\u009f]/g : /[\u0000-\u001f\u007f-\u009f]/g;
  return s.normalize("NFC").replace(/\r\n?/g, "\n").replace(re, "").trim();
}
export const cpLen = (s: string) => [...s].length;
