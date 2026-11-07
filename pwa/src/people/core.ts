// Dữ liệu và cách diễn đạt của mô-đun Người thân (docs/v0.2/08). Mọi tính toán giỗ/sinh nhật/nhắc/.ics gọi lõi
// tsport (memorial*), không viết lại thuật toán ở giao diện (08 §8.1). Mô-đun này chỉ được nạp lười.
import { testFlag } from "../testhooks";
import * as M from "../../../tsport/src/memorial.ts";
import { DEFAULT_METHOD } from "../gen/content";
import { weekday as coreWeekday } from "../../../tsport/src/jd.ts";
import { WEEKDAY, WEEKDAY_SHORT } from "../format";
import { displayCapitalize, openStore, type Store, type JsonObj } from "../store/index";

export { M };
export type { JsonObj, Store };
export type Ymd = M.Ymd;

let storeP: Promise<Store> | null = null;

// TKN3-P-01: chế độ chỉ đọc (04 B5 mục 3 — chuyển đổi lỗi). index.ts đặt cờ từ `migrationStatus()` trước mỗi lần dựng
// trang; các trang ẩn mọi việc ghi (thêm, sửa, xóa, "Cập nhật: đã mất", khôi phục, cài đặt nhắc) — như Android và
// Windows. Vẫn xem được, tạo tệp sao lưu và xuất .ics được.
let readOnlyMode = false;
export function setReadOnly(v: boolean): void { readOnlyMode = v; }
export function isReadOnly(): boolean { return readOnlyMode; }
// MA3C-30-02: bản dựng kiểm thử cho kiểm thử R14 dùng ĐÚNG kho mà mô-đun Người thân đang dùng (không tự dựng Store).
if (__LVHV_TEST_HOOKS__) (globalThis as { __lvhvTest?: unknown }).__lvhvTest = { peopleStore: () => store() };

/** Phần tử ghi dữ liệu: ở chế độ chỉ đọc thì không dựng. */
export function w<T>(el: T): T | null { return readOnlyMode ? null : el; }
/** Một kết nối kho dùng chung cho cả mô-đun. */
export function store(): Promise<Store> {
  // Móc kiểm thử R14 (ca chuyển đổi lỗi, pwa/test/upgrade.mjs) — CHỈ ở bản dựng kiểm thử (LVHV_TEST_HOOKS=1,
  // src/testhooks.ts). Bản dựng chính: __LVHV_TEST_HOOKS__ = false → esbuild bỏ hẳn, trang không đặt được cờ nào.
  const opts = __LVHV_TEST_HOOKS__ ? { forceMigrationFailure: testFlag("__lvhvFailMigration") } : {};
  return (storeP ??= openStore(indexedDB, opts).catch((e) => { storeP = null; throw e; }));
}

/** Hiển thị `danh_xung` / `quan_he` đã lưu: viết hoa chữ đầu bằng hàm của lõi (`display_capitalize`) — tệp khôi phục
 *  từ bản cũ giữ nguyên byte nên có thể còn chữ thường. */
export const dispCap = (s: string): string => displayCapitalize(s);

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
  /** id các hồ sơ đã mất (B4). */
  dead: Set<string>;
}

/** Ảnh chụp toàn kho + bộ dữ liệu của lõi (01 §5 → memorial_model.parseDataset). */
export async function snap(): Promise<Snap> {
  const s = await store();
  const [persons, events, types, settings] = await Promise.all([s.persons(), s.events(), s.eventTypes(), s.settings()]);
  // Bổ sung B4 do lõi áp (02 §6.1: `personOccurrences`, `eventHasDeceased`). `dead` chỉ để giao diện biết khi nào
  // ẩn khối nhắc / hiện câu giải thích.
  const dead = new Set(persons.filter((p) => p.death !== null).map((p) => p.id as string));
  // R-PRIV-2 (01 §3.2a): thông tin báo tử không bao giờ vào lõi tính lần xuất hiện / lời nhắc / .ics.
  const forCore = persons.map((p) => {
    const d = p.death as JsonObj | null;
    if (!d || !("notice" in d)) return p;
    const { notice: _n, ...rest } = d;
    void _n;
    return { ...p, death: rest };
  });
  const ds = M.parseDataset(JSON.stringify({ settings, persons: forCore, events, event_types: types }));
  return { persons, events, types, settings, ds, dead, byId: new Map(persons.map((p) => [p.id as string, p])) };
}

// ---------------- định dạng ----------------
const p2 = (n: number) => String(n).padStart(2, "0");
export const dmy = (d: Ymd) => `${p2(d.d)}/${p2(d.m)}/${d.y}`;
export const dm = (d: Ymd) => `${p2(d.d)}/${p2(d.m)}`;
/** Thứ trong tuần (0 = Chủ nhật) — dùng hàm của lõi (MA3-15-08), không tự tính. */
export const weekdayOf = (d: Ymd) => coreWeekday(M.ymdJdn(d));
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
  // YC-2026-07: viết hoa chữ đầu khi hiển thị (dữ liệu khôi phục từ tệp cũ giữ nguyên byte) — cùng lõi.
  // #38: cắt khoảng trắng khi hiển thị (lõi cắt lúc lưu; tệp khôi phục từ bản cũ có thể còn khoảng trắng).
  const dx = displayCapitalize((p.danh_xung as string) ?? "");
  const ten = ((p.ho_ten as string) ?? "").normalize("NFC").trim();
  return dx ? `${dx} ${ten}` : ten;
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
      return `Tiên thường — lễ cúng hôm trước ${g}`;
    }
    case "sinh_nhat": return o.n !== null ? `Sinh nhật thứ ${o.n}` : "Sinh nhật";
    case "sinh_nhat_am": return o.n !== null ? `Sinh nhật âm lịch (tuổi âm ${o.n + 1})` : "Sinh nhật âm lịch";
    default: return o.headline_plain; // tên người ghép riêng (itemTitle)
  }
}

/** Hồ sơ "chủ" của một lần xuất hiện (sự kiện → người đầu tiên). */
export function ownerOf(sn: Snap, o: M.Occurrence): JsonObj | null {
  if (o.class === "gio" || o.class === "tien_thuong" || o.class === "sinh_nhat" || o.class === "sinh_nhat_am") return sn.byId.get(o.entity_id) ?? null;
  const ev = sn.events.find((e) => e.id === o.entity_id);
  const pid = (ev?.person_ids as string[] | undefined)?.[0];
  return pid ? sn.byId.get(pid) ?? null : null;
}

// ---------------- cách gọi thường ngày (quyết định của chủ dự án 07/11: không dùng từ Hán-Việt cổ như "Hiển khảo") ----------------
// Gợi ý quan hệ và cách gọi: hàm của lõi lưu trữ (store/names.ts ≡ store/src/names.rs, YC-2026-07) —
// danhXungSuggestions, quanHeSuggestions; bốn giao diện dùng chung.

/** Chữ quan hệ trên bia: lấy từ ô "Quan hệ" (bỏ phần trong ngoặc), viết hoa chữ đầu — "ông nội (bên nội)" → "Ông nội". */
export function biaFromRelation(quanHe: string): string {
  const t = quanHe.replace(/\([^)]*\)/g, "").replace(/\s+/g, " ").trim();
  return t ? t.charAt(0).toUpperCase() + t.slice(1) : "";
}
/**
 * Dòng trên tên ở bia: chỉ theo ô "Quan hệ" (yêu cầu 07/11 06:00 — bỏ hẳn "dòng đầu bia" riêng). Trường
 * `memorial.bia_tieu_de` của 01 vẫn được giữ nguyên khi lưu/sao lưu nhưng không hiển thị, không sửa trên giao diện.
 * Quan hệ trống → không có dòng này.
 */
export function biaHeading(p: JsonObj): string {
  return biaFromRelation((p.quan_he as string) ?? "");
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
