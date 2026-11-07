// Kiểm tra dữ liệu ở lớp lưu trữ (docs/v0.2/01 §6.2): V1 (lược đồ backup-v1, cài bằng mã) và V2–V8.
// Bản thứ hai của store/src/validate.rs — cùng quy tắc, cùng thứ tự kiểm, cùng mã lỗi. Hai bản phải
// chấp nhận/từ chối giống nhau (kiểm chéo bằng docs/v0.2/schema/test-vectors/validation-cases.json).
import { checkPersonDerived, daysInMonth, l2s, lunarYearChecked } from "./derive";
import type { Json, JsonObj } from "./json";

export class ValidationError extends Error {
  constructor(public code: string, public path: string, msg: string) {
    super(`${code} ${path}: ${msg}`);
  }
}

const v1 = (path: string, msg: string) => new ValidationError("V1", path, msg);
const err = (code: string, path: string, msg: string) => new ValidationError(code, path, msg);

export const MAX_JSON_DEPTH = 32;
export const MAX_SAFE_INT = Number.MAX_SAFE_INTEGER;

/** Độ sâu `{`/`[` trên văn bản, trước khi phân tích (03 BK-D1). */
export function checkJsonDepth(text: string): void {
  let depth = 0, inStr = false, esc = false;
  for (let i = 0; i < text.length; i++) {
    const c = text.charCodeAt(i);
    if (inStr) {
      if (esc) esc = false;
      else if (c === 0x5c) esc = true;
      else if (c === 0x22) inStr = false;
      continue;
    }
    if (c === 0x22) inStr = true;
    else if (c === 0x7b || c === 0x5b) { if (++depth > MAX_JSON_DEPTH) throw err("JSON", "", `JSON lồng quá ${MAX_JSON_DEPTH} cấp`); }
    else if (c === 0x7d || c === 0x5d) depth = Math.max(0, depth - 1);
  }
}

/** Chuỗi có nửa cặp thay thế (lone surrogate) — serde_json từ chối, nên ở đây cũng từ chối. */
function hasLoneSurrogate(s: string): boolean {
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if (c >= 0xd800 && c <= 0xdbff) { const d = s.charCodeAt(i + 1); if (!(d >= 0xdc00 && d <= 0xdfff)) return true; i++; }
    else if (c >= 0xdc00 && c <= 0xdfff) return true;
  }
  return false;
}

/** true nếu có chuỗi chứa nửa cặp thay thế hoặc số không hữu hạn (serde_json từ chối cả hai). */
function walkStrings(v: Json): boolean {
  if (typeof v === "number") return !Number.isFinite(v);
  if (typeof v === "string") return hasLoneSurrogate(v);
  if (Array.isArray(v)) return v.some(walkStrings);
  if (v && typeof v === "object") return Object.entries(v).some(([k, x]) => hasLoneSurrogate(k) || walkStrings(x));
  return false;
}

/** Phân tích văn bản JSON của tài liệu sao lưu: độ sâu → JSON.parse → (không có nửa cặp thay thế). */
export function parseJson(text: string): Json {
  checkJsonDepth(text);
  let v: Json;
  try { v = JSON.parse(text) as Json; } catch (e) { throw err("JSON", "", (e as Error).message); }
  if (walkStrings(v)) throw err("JSON", "", "chuỗi có nửa cặp thay thế hoặc số ngoài miền");
  return v;
}

// ---------------------------------------------------------------------------------------------

const isObj = (v: Json): v is JsonObj => typeof v === "object" && v !== null && !Array.isArray(v);
const has = (o: JsonObj, k: string) => Object.prototype.hasOwnProperty.call(o, k);

function obj(v: Json, path: string, keys: readonly string[]): JsonObj {
  if (!isObj(v)) throw v1(path, "phải là đối tượng");
  for (const k of keys) if (!has(v, k)) throw v1(path, `thiếu trường "${k}"`);
  // serde_json::Map là BTreeMap: khóa lạ đầu tiên theo thứ tự byte.
  const extra = Object.keys(v).filter((k) => !keys.includes(k)).sort();
  if (extra.length) throw v1(path, `trường lạ "${extra[0]}"`);
  return v;
}

const p = (path: string, k: string) => `${path}/${k}`;
export const int = (v: Json): number | null => (typeof v === "number" && Number.isSafeInteger(v) ? v : null);

function intIn(v: Json, path: string, lo: number, hi: number): number {
  const x = int(v);
  if (x === null) throw v1(path, "phải là số nguyên");
  if (x < lo || x > hi) throw v1(path, `ngoài khoảng ${lo}–${hi}`);
  return x;
}
const optIntIn = (v: Json, path: string, lo: number, hi: number): number | null => (v === null ? null : intIn(v, path, lo, hi));

function boolean(v: Json, path: string): boolean {
  if (typeof v !== "boolean") throw v1(path, "phải là boolean");
  return v;
}
function string(v: Json, path: string): string {
  if (typeof v !== "string") throw v1(path, "phải là chuỗi");
  return v;
}
function enumStr(v: Json, path: string, allowed: readonly string[]): string {
  const s = string(v, path);
  if (!allowed.includes(s)) throw v1(path, `giá trị không hợp lệ "${s}"`);
  return s;
}

function isControl(cp: number, multiline: boolean): boolean {
  return (cp <= 0x1f && !(multiline && cp === 0x0a)) || (cp >= 0x7f && cp <= 0x9f);
}

function text(v: Json, path: string, min: number, max: number, multiline: boolean, needNonspace: boolean): void {
  const s = string(v, path);
  const cps = [...s];
  if (cps.some((c) => isControl(c.codePointAt(0)!, multiline))) throw v1(path, "chứa ký tự điều khiển");
  if (cps.length < min || cps.length > max) throw v1(path, `độ dài ${cps.length} ngoài ${min}–${max}`);
  if (needNonspace && !/\S/.test(s)) throw v1(path, "không được để trống");
}

const UUID_V4 = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;
export const isUuidV4 = (s: string) => UUID_V4.test(s);
function uuid(v: Json, path: string): string {
  const s = string(v, path);
  if (!UUID_V4.test(s)) throw v1(path, "không phải UUID v4 chữ thường");
  return s;
}
const TS = /^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$/;
export const isTimestamp = (s: string) => TS.test(s);
function timestamp(v: Json, path: string): void {
  if (!TS.test(string(v, path))) throw v1(path, "không phải thời điểm RFC 3339 UTC (…Z)");
}
function hhmm(v: Json, path: string): void {
  if (!/^([01][0-9]|2[0-3]):[0-5][0-9]$/.test(string(v, path))) throw v1(path, "không phải giờ HH:MM");
}
const rev = (v: Json, path: string) => void intIn(v, path, 1, MAX_SAFE_INT);

function offsets(v: Json, path: string): void {
  if (!Array.isArray(v)) throw v1(path, "phải là mảng");
  if (v.length > 8) throw v1(path, "quá 8 phần tử");
  let prev = -1;
  const seen = new Set<number>();
  v.forEach((x, i) => {
    const n = intIn(x, p(path, String(i)), 0, 60);
    if (seen.has(n)) throw v1(path, "phần tử trùng");
    seen.add(n);
    if (n <= prev) throw err("V8", path, "offsets_days phải tăng dần");
    prev = n;
  });
}

type Precision = "day" | "month_day" | "year";

function dateValue(v: Json, path: string, allowed: readonly Precision[]): { precision: Precision } {
  const o = obj(v, path, ["calendar", "precision", "year", "month", "day", "leap"]);
  const lunar = enumStr(o.calendar, p(path, "calendar"), ["solar", "lunar"]) === "lunar";
  const precision = enumStr(o.precision, p(path, "precision"), ["day", "month_day", "year"]) as Precision;
  const year = optIntIn(o.year, p(path, "year"), 1000, 3000);
  const month = optIntIn(o.month, p(path, "month"), 1, 12);
  const day = optIntIn(o.day, p(path, "day"), 1, lunar ? 30 : 31);
  const leap = boolean(o.leap, p(path, "leap"));
  const shapeOk = precision === "day" ? year !== null && month !== null && day !== null
    : precision === "month_day" ? lunar && year === null && month !== null && day !== null
    : year !== null && month === null && day === null && !leap;
  if (!shapeOk) throw v1(path, `trường không khớp precision = ${precision}`);
  if (!lunar && leap) throw v1(path, "dương lịch không có tháng nhuận");
  if (!allowed.includes(precision)) throw v1(path, `precision = ${precision} không được phép ở đây`);
  if (precision === "day") {
    const [y, m, d] = [year!, month!, day!];
    if (!lunar) {
      if (d > daysInMonth(y, m)) throw err("V4", path, `ngày ${d}/${m}/${y} không tồn tại`);
    } else if (lunarYearChecked(y) && l2s(y, m, d, leap) === null) {
      throw err("V4", path, `ngày âm ${d}/${m}${leap ? " nhuận" : ""}/${y} không tồn tại`);
    }
  }
  return { precision };
}

function lunarDay(v: Json, path: string): void {
  const o = obj(v, path, ["year", "month", "day", "leap"]);
  optIntIn(o.year, p(path, "year"), 1000, 3000);
  intIn(o.month, p(path, "month"), 1, 12);
  intIn(o.day, p(path, "day"), 1, 30);
  boolean(o.leap, p(path, "leap"));
}
function solarDay(v: Json, path: string): void {
  const o = obj(v, path, ["y", "m", "d"]);
  intIn(o.y, p(path, "y"), 1000, 3000);
  intIn(o.m, p(path, "m"), 1, 12);
  intIn(o.d, p(path, "d"), 1, 31);
}
function nullable(v: Json, path: string, f: (v: Json, path: string) => void): void {
  if (v !== null) f(v, path);
}

export function reminderRule(v: Json, path: string): void {
  const o = obj(v, path, ["id", "enabled", "offsets_days", "time_local", "tien_thuong"]);
  uuid(o.id, p(path, "id"));
  boolean(o.enabled, p(path, "enabled"));
  offsets(o.offsets_days, p(path, "offsets_days"));
  hhmm(o.time_local, p(path, "time_local"));
  boolean(o.tien_thuong, p(path, "tien_thuong"));
}

function birthInfo(v: Json, path: string): void {
  const o = obj(v, path, ["input", "noi_sinh", "lunar_year", "lunar_year_approx", "can_chi", "lunar", "solar"]);
  dateValue(o.input, p(path, "input"), ["day", "year"]);
  text(o.noi_sinh, p(path, "noi_sinh"), 0, 300, false, false);
  optIntIn(o.lunar_year, p(path, "lunar_year"), 999, 3000);
  boolean(o.lunar_year_approx, p(path, "lunar_year_approx"));
  if (o.can_chi !== null && [...string(o.can_chi, p(path, "can_chi"))].length > 20) throw v1(p(path, "can_chi"), "quá 20 ký tự");
  nullable(o.lunar, p(path, "lunar"), lunarDay);
  nullable(o.solar, p(path, "solar"), solarDay);
}

/** 01 §3.2a: optional notice string — null, or 1–max code points, trimmed (empty must be null). */
function optTrimmed(v: Json, path: string, max: number, multiline: boolean): void {
  if (v === null) return;
  text(v, path, 1, max, multiline, true);
  if ((v as string).trim() !== v) throw v1(path, "chưa cắt khoảng trắng hai đầu");
}

/** Tối đa 6 tệp đính kèm cho một thông tin báo tử (01 §3.2a, YC-2026-07). */
export const MAX_NOTICE_ATTACHMENTS = 6;
/** Ảnh đính kèm ≤ 2 MiB, PDF ≤ 10 MiB (01 §3.8a). */
export const MAX_ATTACHMENT_IMAGE = 2 * 1024 * 1024;
export const MAX_ATTACHMENT_PDF = 10 * 1024 * 1024;
export const ATTACHMENT_KINDS = ["tin_bao_tu", "giay_chung_tu", "khac"] as const;

/** `DeathNotice` (01 §3.2a, YC-2026-05, YC-2026-07) — mirror of store/src/validate.rs `death_notice`. */
export function deathNotice(v: Json, path: string): void {
  const o = obj(v, path, ["reported_by", "place", "reason", "notice_date", "attachments", "note"]);
  optTrimmed(o.reported_by, p(path, "reported_by"), 200, false);
  optTrimmed(o.place, p(path, "place"), 200, false);
  optTrimmed(o.reason, p(path, "reason"), 500, false);
  optTrimmed(o.note, p(path, "note"), 2000, true);
  nullable(o.notice_date, p(path, "notice_date"), (d, dp) => {
    dateValue(d, dp, ["day"]);
    if ((d as JsonObj).calendar !== "solar") throw v1(dp, "ngày nhận tin phải là ngày dương");
  });
  const ap = p(path, "attachments");
  const list = o.attachments;
  if (!Array.isArray(list)) throw v1(ap, "phải là mảng");
  if (list.length > MAX_NOTICE_ATTACHMENTS) throw v1(ap, `quá ${MAX_NOTICE_ATTACHMENTS} tệp`);
  const ids = new Set<string>(), files = new Set<string>();
  list.forEach((x, i) => {
    const xp = `${ap}/${i}`;
    const a = obj(x, xp, ["id", "kind", "file_id"]);
    const id = uuid(a.id, p(xp, "id"));
    if (ids.has(id)) throw v1(p(xp, "id"), "id trùng");
    ids.add(id);
    enumStr(a.kind, p(xp, "kind"), ATTACHMENT_KINDS);
    const f = uuid(a.file_id, p(xp, "file_id"));
    if (files.has(f)) throw v1(p(xp, "file_id"), "tệp trùng");
    files.add(f);
  });
}

/** Mọi id ảnh mà một Person tham chiếu: `photo_id`, `memorial.tray_photo_id` (YC-2026-10) — như Rust `person_photo_refs`. */
export function personPhotoRefs(pj: JsonObj): string[] {
  const t = trayPhotoRef(pj);
  return [...(typeof pj.photo_id === "string" ? [pj.photo_id] : []), ...(t === null ? [] : [t])];
}

/** `memorial.tray_photo_id` (YC-2026-10) hoặc null — như Rust `tray_photo_ref`. */
export function trayPhotoRef(pj: JsonObj): string | null {
  const m = pj.memorial;
  return m && isObj(m) && typeof (m as JsonObj).tray_photo_id === "string" ? ((m as JsonObj).tray_photo_id as string) : null;
}

/** Như Rust `tray_photo_shared`: ảnh mâm lễ chỉ thuộc một người. Trả đường dẫn tương đối của trường vi phạm hoặc null. */
export function trayPhotoShared(pj: JsonObj, persons: JsonObj[]): string | null {
  const tray = trayPhotoRef(pj);
  const ph = typeof pj.photo_id === "string" ? pj.photo_id : null;
  if (tray !== null && tray === ph) return "memorial/tray_photo_id";
  for (const q of persons) {
    if (q.id === pj.id) continue;
    if (tray !== null && (q.photo_id === tray || trayPhotoRef(q) === tray)) return "memorial/tray_photo_id";
    const qt = trayPhotoRef(q);
    if (ph !== null && qt !== null && ph === qt) return "photo_id";
  }
  return null;
}

/** Every attachment id a Person references: `death.notice.attachments[].file_id` (YC-2026-07). */
export function personAttachmentRefs(pj: JsonObj): string[] {
  const d = pj.death as JsonObj | null | undefined;
  const n = d && isObj(d) ? (d.notice as JsonObj | null | undefined) : null;
  if (!n || !isObj(n) || !Array.isArray(n.attachments)) return [];
  return n.attachments.flatMap((x) => (isObj(x) && typeof x.file_id === "string" ? [x.file_id] : []));
}

/** `Attachment` (01 §3.8a) — mirror of Rust `attachment`. */
export function attachment(v: Json, path: string): void {
  const o = obj(v, path, ["id", "mime", "bytes", "sha256", "stripped", "file_name", "created_at"]);
  uuid(o.id, p(path, "id"));
  const pdf = enumStr(o.mime, p(path, "mime"), ["image/jpeg", "image/webp", "application/pdf"]) === "application/pdf";
  intIn(o.bytes, p(path, "bytes"), 1, pdf ? MAX_ATTACHMENT_PDF : MAX_ATTACHMENT_IMAGE);
  const h = string(o.sha256, p(path, "sha256"));
  if (!/^[0-9a-f]{64}$/.test(h)) throw v1(p(path, "sha256"), "phải là 64 ký tự hex thường");
  if (o.stripped !== !pdf) throw v1(p(path, "stripped"), pdf ? "PDF lưu nguyên bản: phải là false" : "ảnh luôn được mã hóa lại: phải là true");
  nullable(o.file_name, p(path, "file_name"), (x, xp) => text(x, xp, 1, 120, false, true));
  timestamp(o.created_at, p(path, "created_at"));
}

function deathInfo(v: Json, path: string): void {
  // `notice` (01 §3.2a) is an optional key: absent ≡ null (old v1 documents and vectors stay valid).
  const hasNotice = isObj(v) && has(v, "notice");
  const base = ["input", "lunar", "solar", "lunar_source", "method", "tuoi"];
  const o = obj(v, path, hasNotice ? [...base, "notice"] : base);
  if (hasNotice) nullable(o.notice, p(path, "notice"), deathNotice);
  dateValue(o.input, p(path, "input"), ["day", "month_day"]);
  lunarDay(o.lunar, p(path, "lunar"));
  nullable(o.solar, p(path, "solar"), solarDay);
  enumStr(o.lunar_source, p(path, "lunar_source"), ["entered", "converted"]);
  enumStr(o.method, p(path, "method"), ["precise", "hnd", "jpl"]);
  const tp = p(path, "tuoi");
  const t = obj(o.tuoi, tp, ["value", "label", "manual", "approx"]);
  optIntIn(t.value, p(tp, "value"), 1, 150);
  enumStr(t.label, p(tp, "label"), ["huong_tho", "huong_duong"]);
  boolean(t.manual, p(tp, "manual"));
  boolean(t.approx, p(tp, "approx"));
}

function memorial(v: Json, path: string): void {
  // `tray_photo_id` (YC-2026-10, 01 §3.7): khóa tùy chọn, thiếu ≡ null — như Rust.
  const hasTray = isObj(v) && has(v, "tray_photo_id");
  const o = obj(v, path, ["tray_style", "incense", "bia_tieu_de", "loi_tuong_niem", "gio_het_nam", "huong_duong_duoi", ...(hasTray ? ["tray_photo_id"] : [])]);
  if (hasTray && o.tray_photo_id !== null) uuid(o.tray_photo_id, p(path, "tray_photo_id"));
  const style = enumStr(o.tray_style, p(path, "tray_style"), ["phat_giao", "truyen_thong", "cong_giao"]);
  const incense = boolean(o.incense, p(path, "incense"));
  text(o.bia_tieu_de, p(path, "bia_tieu_de"), 0, 60, false, false);
  text(o.loi_tuong_niem, p(path, "loi_tuong_niem"), 0, 2000, true, false);
  const g = int(o.gio_het_nam);
  if (g === null) throw v1(p(path, "gio_het_nam"), "phải là số nguyên");
  if (g !== 2 && g !== 3) throw v1(p(path, "gio_het_nam"), "chỉ 2 hoặc 3");
  intIn(o.huong_duong_duoi, p(path, "huong_duong_duoi"), 1, 150);
  if (style === "cong_giao" && incense) throw err("V6", p(path, "incense"), "Công giáo không có hương");
}

const PERSON_KEYS = ["id", "kind", "danh_xung", "ho_ten", "ten_hieu", "quan_he", "birth", "death", "noi_an_tang", "photo_id", "memorial",
  "reminders", "sinh_nhat_am", "feb29_rule", "ghi_chu", "created_at", "updated_at", "rev"] as const;

export function person(v: Json, path: string): void {
  // `gioi_tinh` (YC-2026-07): khóa tùy chọn, thiếu ≡ null.
  const hasGt = isObj(v) && has(v, "gioi_tinh");
  // `legacy` (04 B5): bản gốc của trường cũ mà phép chuyển đổi không đặt được vào trường mới — đối tượng tự do.
  const hasLegacy = isObj(v) && has(v, "legacy");
  const o = obj(v, path, [...PERSON_KEYS, ...(hasGt ? ["gioi_tinh"] : []), ...(hasLegacy ? ["legacy"] : [])]);
  if (hasLegacy && !isObj(o.legacy)) throw v1(p(path, "legacy"), "phải là đối tượng");
  if (hasGt && o.gioi_tinh !== null) enumStr(o.gioi_tinh, p(path, "gioi_tinh"), ["nam", "nu"]);
  uuid(o.id, p(path, "id"));
  const deceased = enumStr(o.kind, p(path, "kind"), ["deceased", "living"]) === "deceased";
  text(o.danh_xung, p(path, "danh_xung"), 0, 40, false, false);
  text(o.ho_ten, p(path, "ho_ten"), 1, 120, false, true);
  text(o.ten_hieu, p(path, "ten_hieu"), 0, 120, false, false);
  text(o.quan_he, p(path, "quan_he"), 0, 120, false, false);
  nullable(o.birth, p(path, "birth"), birthInfo);
  nullable(o.death, p(path, "death"), deathInfo);
  text(o.noi_an_tang, p(path, "noi_an_tang"), 0, 300, false, false);
  if (o.photo_id !== null) uuid(o.photo_id, p(path, "photo_id"));
  nullable(o.memorial, p(path, "memorial"), memorial);
  const rp = p(path, "reminders");
  const r = obj(o.reminders, rp, ["gio", "sinh_nhat", "sinh_nhat_am"]);
  for (const k of ["gio", "sinh_nhat", "sinh_nhat_am"]) nullable(r[k], p(rp, k), reminderRule);
  const snAm = boolean(o.sinh_nhat_am, p(path, "sinh_nhat_am"));
  enumStr(o.feb29_rule, p(path, "feb29_rule"), ["feb28", "mar1"]);
  text(o.ghi_chu, p(path, "ghi_chu"), 0, 2000, true, false);
  timestamp(o.created_at, p(path, "created_at"));
  timestamp(o.updated_at, p(path, "updated_at"));
  rev(o.rev, p(path, "rev"));
  if (deceased) {
    if (r.sinh_nhat !== null || r.sinh_nhat_am !== null || snAm) throw v1(path, "người đã khuất chỉ có nhắc giỗ");
  } else if (o.noi_an_tang !== "" || r.gio !== null) {
    throw v1(path, "người đang sống không có nơi an táng / nhắc giỗ");
  }
  if (deceased !== (o.death !== null) || deceased !== (o.memorial !== null)) throw err("V2", path, "kind = deceased ⇔ death ≠ null ⇔ memorial ≠ null");
  checkPersonDerived(o, path);
}

export const BUILTIN_EVENT_TYPES: readonly [string, string, string, boolean][] = [
  ["tot_nghiep", "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0001", "Tốt nghiệp", false],
  ["ngay_cuoi", "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0002", "Ngày cưới", true],
  ["dinh_hon", "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0003", "Đính hôn", false],
  ["du_hoc", "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0004", "Du học", false],
];

export function eventType(v: Json, path: string): void {
  const o = obj(v, path, ["id", "builtin", "ten", "default_recurring", "created_at", "updated_at", "rev"]);
  const id = uuid(o.id, p(path, "id"));
  if (o.builtin !== null) {
    const b = enumStr(o.builtin, p(path, "builtin"), ["tot_nghiep", "ngay_cuoi", "dinh_hon", "du_hoc"]);
    if (BUILTIN_EVENT_TYPES.find((t) => t[0] === b)?.[1] !== id) throw v1(p(path, "id"), "loại có sẵn phải dùng id cố định (01 §3.5)");
  }
  text(o.ten, p(path, "ten"), 1, 60, false, true);
  boolean(o.default_recurring, p(path, "default_recurring"));
  timestamp(o.created_at, p(path, "created_at"));
  timestamp(o.updated_at, p(path, "updated_at"));
  rev(o.rev, p(path, "rev"));
}

export function event(v: Json, path: string): void {
  const o = obj(v, path, ["id", "type_id", "tieu_de", "date", "recurring", "person_ids", "ghi_chu", "reminder", "created_at", "updated_at", "rev"]);
  uuid(o.id, p(path, "id"));
  uuid(o.type_id, p(path, "type_id"));
  text(o.tieu_de, p(path, "tieu_de"), 0, 120, false, false);
  const d = dateValue(o.date, p(path, "date"), ["day", "month_day"]);
  const recurring = boolean(o.recurring, p(path, "recurring"));
  if (d.precision === "month_day" && !recurring) throw v1(p(path, "recurring"), "sự kiện không rõ năm phải lặp hằng năm");
  const pp = p(path, "person_ids");
  const ids = o.person_ids;
  if (!Array.isArray(ids)) throw v1(pp, "phải là mảng");
  if (ids.length > 50) throw v1(pp, "quá 50 người");
  const seen = new Set<string>();
  ids.forEach((x, i) => {
    const s = uuid(x, p(pp, String(i)));
    if (seen.has(s)) throw v1(pp, "id trùng");
    seen.add(s);
  });
  text(o.ghi_chu, p(path, "ghi_chu"), 0, 2000, true, false);
  nullable(o.reminder, p(path, "reminder"), reminderRule);
  timestamp(o.created_at, p(path, "created_at"));
  timestamp(o.updated_at, p(path, "updated_at"));
  rev(o.rev, p(path, "rev"));
}

export function photo(v: Json, path: string): void {
  const o = obj(v, path, ["id", "mime", "width", "height", "bytes", "sha256", "stripped", "created_at"]);
  uuid(o.id, p(path, "id"));
  enumStr(o.mime, p(path, "mime"), ["image/jpeg", "image/webp"]);
  intIn(o.width, p(path, "width"), 1, 2048);
  intIn(o.height, p(path, "height"), 1, 2048);
  intIn(o.bytes, p(path, "bytes"), 1, 1_048_576);
  if (!/^[0-9a-f]{64}$/.test(string(o.sha256, p(path, "sha256")))) throw v1(p(path, "sha256"), "phải là 64 ký tự hex thường");
  if (o.stripped !== true) throw v1(p(path, "stripped"), "phải là true");
  timestamp(o.created_at, p(path, "created_at"));
}

export function settings(v: Json, path: string): void {
  const o = obj(v, path, ["reminders_enabled", "default_offsets_days", "default_time_local", "upcoming_days", "last_backup_at", "backup_nag_days"]);
  boolean(o.reminders_enabled, p(path, "reminders_enabled"));
  offsets(o.default_offsets_days, p(path, "default_offsets_days"));
  hhmm(o.default_time_local, p(path, "default_time_local"));
  if (int(o.upcoming_days) !== 30) throw v1(p(path, "upcoming_days"), "phải là 30");
  nullable(o.last_backup_at, p(path, "last_backup_at"), timestamp);
  const n = int(o.backup_nag_days);
  if (n === null || ![0, 30, 60, 90].includes(n)) throw v1(p(path, "backup_nag_days"), "chỉ 0, 30, 60, 90");
}

/** Khóa so sánh tên loại sự kiện (01 §3.5). */
export const nameKey = (ten: string) => ten.normalize("NFC").toLowerCase();

const ROOT_KEYS = ["format", "schema_version", "app", "exported_at", "settings", "event_types", "persons", "events", "photos"] as const;

/** Toàn bộ tài liệu sao lưu: V1–V8. Ném ValidationError ở lỗi đầu tiên (cùng thứ tự với Rust). */
export function document(v: Json): void {
  // `attachments` (01 §3.8a): khóa tùy chọn, thiếu ≡ [].
  const hasAtt = isObj(v) && has(v, "attachments");
  const root = obj(v, "", hasAtt ? [...ROOT_KEYS, "attachments"] : ROOT_KEYS);
  if (root.format !== "lvhv-backup") throw v1("/format", "phải là \"lvhv-backup\"");
  if (int(root.schema_version) !== 1) throw v1("/schema_version", "phải là 1");
  const app = obj(root.app, "/app", ["name", "version", "platform"]);
  if (app.name !== "Lịch và Văn hóa Việt") throw v1("/app/name", "tên ứng dụng sai");
  if (!/^[0-9]+\.[0-9]+\.[0-9]+$/.test(string(app.version, "/app/version"))) throw v1("/app/version", "không đúng dạng X.Y.Z");
  enumStr(app.platform, "/app/platform", ["pwa", "android", "ios", "macos", "windows"]);
  timestamp(root.exported_at, "/exported_at");
  settings(root.settings, "/settings");
  const arr = (k: string, max: number): Json[] => {
    const a = root[k];
    if (!Array.isArray(a)) throw v1(`/${k}`, "phải là mảng");
    if (a.length > max) throw v1(`/${k}`, `quá ${max} phần tử`);
    return a;
  };
  const types = arr("event_types", 500), persons = arr("persons", 10_000), events = arr("events", 50_000), photos = arr("photos", 10_000);
  const ids = (a: Json[], k: string, f: (v: Json, path: string) => void): Set<string> => {
    const s = new Set<string>();
    a.forEach((x, i) => {
      const path = `/${k}/${i}`;
      f(x, path);
      const id = (x as JsonObj).id as string;
      if (s.has(id)) throw v1(p(path, "id"), "id trùng");
      s.add(id);
    });
    return s;
  };
  const photoIds = ids(photos, "photos", photo);
  const attIds = ids(hasAtt ? arr("attachments", 10_000) : [], "attachments", attachment);
  const typeIds = ids(types, "event_types", eventType);
  const personIds = ids(persons, "persons", person);
  ids(events, "events", event);
  const names = new Set<string>(), builtins = new Set<string>();
  types.forEach((t, i) => {
    const o = t as JsonObj;
    const k = nameKey(o.ten as string);
    if (names.has(k)) throw err("V7", `/event_types/${i}/ten`, "tên loại sự kiện trùng");
    names.add(k);
    if (typeof o.builtin === "string") {
      if (builtins.has(o.builtin)) throw v1(`/event_types/${i}/builtin`, "loại có sẵn trùng");
      builtins.add(o.builtin);
    }
  });
  persons.forEach((x, i) => {
    const pid = (x as JsonObj).photo_id;
    if (typeof pid === "string" && !photoIds.has(pid)) throw err("V3", `/persons/${i}/photo_id`, "ảnh không tồn tại");
    const tray = trayPhotoRef(x as JsonObj);
    if (tray !== null && !photoIds.has(tray)) throw err("V3", `/persons/${i}/memorial/tray_photo_id`, "ảnh mâm lễ không tồn tại");
    const sh = trayPhotoShared(x as JsonObj, persons as JsonObj[]);
    if (sh !== null) throw err("V3", `/persons/${i}/${sh}`, "ảnh mâm lễ dùng chung với hồ sơ khác");
    const d = (x as JsonObj).death as JsonObj | null;
    const n = d && isObj(d) ? (d.notice as JsonObj | null | undefined) : null;
    if (n && isObj(n)) {
      (n.attachments as JsonObj[]).forEach((c, j) => {
        if (!attIds.has(c.file_id as string)) throw err("V3", `/persons/${i}/death/notice/attachments/${j}/file_id`, "tệp đính kèm không tồn tại");
      });
    }
  });
  events.forEach((x, i) => {
    const e = x as JsonObj;
    if (!typeIds.has(e.type_id as string)) throw err("V3", `/events/${i}/type_id`, "loại sự kiện không tồn tại");
    (e.person_ids as string[]).forEach((pid, j) => {
      if (!personIds.has(pid)) throw err("V3", `/events/${i}/person_ids/${j}`, "người không tồn tại");
    });
  });
}
