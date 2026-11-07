// Cách gọi, quan hệ, giới tính (01 §3.1, YC-2026-07) — giống hệt store/src/names.rs (Rust, qua FFI cho ứng dụng
// native). Kiểm chéo trong scripts/check-store.sh (names-rust.json).
import type { JsonObj } from "./json";

/** Gợi ý cách gọi cho người đã mất. */
export const DANH_XUNG_DECEASED = ["Ông", "Bà", "Cụ", "Kỵ", "Bác", "Cô", "Chú", "Dì", "Cậu", "Mợ", "Thím", "Anh", "Chị", "Em",
  "Cháu", "Con", "Bạn", "Bố", "Mẹ", "Ông Tổ họ", "Bà Cô Tổ họ", "Bé đỏ"] as const;
/** Chỉ dành cho người đã mất — không gợi ý cho người đang sống. */
export const DANH_XUNG_DECEASED_ONLY: readonly string[] = ["Cụ", "Kỵ", "Ông Tổ họ", "Bà Cô Tổ họ", "Bé đỏ"];
const DANH_XUNG_NAM: readonly string[] = ["Ông", "Chú", "Cậu", "Anh", "Bố", "Ông Tổ họ"];
const DANH_XUNG_NU: readonly string[] = ["Bà", "Cô", "Dì", "Mợ", "Thím", "Chị", "Mẹ", "Bà Cô Tổ họ"];

export const QUAN_HE_DECEASED = ["Bố", "Mẹ", "Ông nội", "Bà nội", "Ông ngoại", "Bà ngoại", "Cụ ông", "Cụ bà", "Kỵ ông", "Kỵ bà",
  "Ông cố", "Bà cố", "Ông sơ", "Bà sơ", "Bác", "Chú", "Thím", "Cô", "Dì", "Cậu", "Mợ", "Anh", "Chị", "Em", "Chồng", "Vợ", "Con"] as const;
export const QUAN_HE_LIVING = ["Bố", "Mẹ", "Ông nội", "Bà nội", "Ông ngoại", "Bà ngoại", "Bác", "Chú", "Thím", "Cô", "Dì", "Cậu", "Mợ",
  "Anh", "Chị", "Em", "Chồng", "Vợ", "Con trai", "Con gái", "Cháu"] as const;

/** Gợi ý cách gọi; `"nam"`/`"nu"`: từ hợp giới tính lên trước, rồi từ chung, rồi từ còn lại (ổn định). */
export function danhXungSuggestions(deceased: boolean, gioiTinh: string | null = null): string[] {
  const base: string[] = DANH_XUNG_DECEASED.filter((x) => deceased || !DANH_XUNG_DECEASED_ONLY.includes(x));
  const order = gioiTinh === "nam" ? [DANH_XUNG_NAM, DANH_XUNG_NU] : gioiTinh === "nu" ? [DANH_XUNG_NU, DANH_XUNG_NAM] : null;
  if (!order) return base;
  const rank = (x: string) => (order[0].includes(x) ? 0 : order[1].includes(x) ? 2 : 1);
  return base.map((x, i) => [x, i] as const).sort((a, b) => rank(a[0]) - rank(b[0]) || a[1] - b[1]).map(([x]) => x);
}

export function quanHeSuggestions(deceased: boolean): string[] {
  return [...(deceased ? QUAN_HE_DECEASED : QUAN_HE_LIVING)];
}

/** Chuẩn hóa chuỗi khi lưu (01 §1): NFC rồi cắt khoảng trắng hai đầu (như Rust `clean_text`). */
export const cleanText = (s: string): string => s.normalize("NFC").trim();

/** Viết hoa chữ cái đầu: NFC, cắt khoảng trắng hai đầu, chữ đầu tiên (một điểm mã) thành chữ hoa, phần còn lại giữ
 *  nguyên. "  chú  " → "Chú"; "   " → "". */
export function capitalizeFirst(s: string): string {
  const n = cleanText(s);
  const cps = [...n];
  if (!cps.length) return "";
  return (cps[0].toUpperCase() + cps.slice(1).join("")).normalize("NFC");
}

/** Viết hoa chữ đầu khi hiển thị cách gọi / quan hệ đã lưu (dữ liệu khôi phục có thể còn viết thường). */
export const displayCapitalize = capitalizeFirst;

/** Chuẩn hóa Person khi lưu từ giao diện (như Rust `normalize_person_names`): viết hoa chữ đầu `danh_xung`,
 *  `quan_he`; `gioi_tinh: null` → bỏ khóa. Không áp khi nhập sao lưu. */
const PERSON_TEXT: [string, string | null][] = [["danh_xung", null], ["ho_ten", null], ["ten_hieu", null], ["quan_he", null],
  ["noi_an_tang", null], ["ghi_chu", null], ["birth", "noi_sinh"], ["memorial", "bia_tieu_de"], ["memorial", "loi_tuong_niem"]];

/** Như Rust `normalize_person_names`: mọi trường chữ tự do NFC + cắt khoảng trắng (chỉ còn khoảng trắng → ""), viết hoa
 *  chữ đầu `danh_xung`, `quan_he`, `gioi_tinh: null` → bỏ khóa. Không áp khi nhập sao lưu. */
export function normalizePersonNames(p: JsonObj): JsonObj {
  const out: JsonObj = structuredClone(p);
  for (const [k, sub] of PERSON_TEXT) {
    if (sub === null) { if (typeof out[k] === "string") out[k] = cleanText(out[k] as string); continue; }
    const o = out[k];
    if (o && typeof o === "object" && !Array.isArray(o) && typeof (o as JsonObj)[sub] === "string") (o as JsonObj)[sub] = cleanText((o as JsonObj)[sub] as string);
  }
  for (const k of ["danh_xung", "quan_he"] as const) if (typeof out[k] === "string") out[k] = capitalizeFirst(out[k] as string);
  if (Object.prototype.hasOwnProperty.call(out, "gioi_tinh") && out.gioi_tinh === null) delete out.gioi_tinh;
  // YC-2026-10: `memorial.tray_photo_id: null` → bỏ khóa (như Rust).
  const m = out.memorial;
  if (m && typeof m === "object" && !Array.isArray(m) && Object.prototype.hasOwnProperty.call(m, "tray_photo_id") && (m as JsonObj).tray_photo_id === null) delete (m as JsonObj).tray_photo_id;
  return out;
}
