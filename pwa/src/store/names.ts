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

/** Viết hoa chữ cái đầu: NFC, chữ đầu tiên (một điểm mã) thành chữ hoa, phần còn lại giữ nguyên. */
export function capitalizeFirst(s: string): string {
  const n = s.normalize("NFC");
  const cps = [...n];
  if (!cps.length) return "";
  return (cps[0].toUpperCase() + cps.slice(1).join("")).normalize("NFC");
}

/** Chuẩn hóa Person khi lưu từ giao diện (như Rust `normalize_person_names`): viết hoa chữ đầu `danh_xung`,
 *  `quan_he`; `gioi_tinh: null` → bỏ khóa. Không áp khi nhập sao lưu. */
export function normalizePersonNames(p: JsonObj): JsonObj {
  const out: JsonObj = { ...p };
  for (const k of ["danh_xung", "quan_he"] as const) if (typeof out[k] === "string") out[k] = capitalizeFirst(out[k] as string);
  if (Object.prototype.hasOwnProperty.call(out, "gioi_tinh") && out.gioi_tinh === null) delete out.gioi_tinh;
  return out;
}
