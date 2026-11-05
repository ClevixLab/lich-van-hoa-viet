// Port of core/src/canchi.rs — Can Chi, Nạp âm, Trực, 28 tú, 12 thần, giờ Hoàng đạo.
// All inputs are small non-negative integers (Rust u32), so `%` matches.

import { remEuclid } from "./jd.ts";

export const CAN: readonly string[] = ["Giáp", "Ất", "Bính", "Đinh", "Mậu", "Kỷ", "Canh", "Tân", "Nhâm", "Quý"];
export const CHI: readonly string[] = ["Tý", "Sửu", "Dần", "Mão", "Thìn", "Tỵ", "Ngọ", "Mùi", "Thân", "Dậu", "Tuất", "Hợi"];
export const TRUC: readonly string[] = ["Kiến", "Trừ", "Mãn", "Bình", "Định", "Chấp", "Phá", "Nguy", "Thành", "Thu", "Khai", "Bế"];
export const TU: readonly string[] = [
  "Giác", "Cang", "Đê", "Phòng", "Tâm", "Vĩ", "Cơ", "Đẩu", "Ngưu", "Nữ", "Hư", "Nguy", "Thất",
  "Bích", "Khuê", "Lâu", "Vị", "Mão", "Tất", "Chủy", "Sâm", "Tỉnh", "Quỷ", "Liễu", "Tinh",
  "Trương", "Dực", "Chẩn",
];
export const THAN: readonly string[] = [
  "Thanh Long", "Minh Đường", "Thiên Hình", "Chu Tước", "Kim Quỹ", "Kim Đường",
  "Bạch Hổ", "Ngọc Đường", "Thiên Lao", "Huyền Vũ", "Tư Mệnh", "Câu Trần",
];
export const THAN_HOANG_DAO: readonly boolean[] = [
  true, true, false, false, true, true, false, true, false, false, true, false,
];
export const NAP_AM: readonly string[] = [
  "Hải Trung Kim", "Lô Trung Hỏa", "Đại Lâm Mộc", "Lộ Bàng Thổ", "Kiếm Phong Kim", "Sơn Đầu Hỏa",
  "Giản Hạ Thủy", "Thành Đầu Thổ", "Bạch Lạp Kim", "Dương Liễu Mộc", "Tuyền Trung Thủy", "Ốc Thượng Thổ",
  "Tích Lịch Hỏa", "Tùng Bách Mộc", "Trường Lưu Thủy", "Sa Trung Kim", "Sơn Hạ Hỏa", "Bình Địa Mộc",
  "Bích Thượng Thổ", "Kim Bạch Kim", "Phú Đăng Hỏa", "Thiên Hà Thủy", "Đại Trạch Thổ", "Thoa Xuyến Kim",
  "Tang Đố Mộc", "Đại Khê Thủy", "Sa Trung Thổ", "Thiên Thượng Hỏa", "Thạch Lựu Mộc", "Đại Hải Thủy",
];

export const TU_ANCHOR_JDN = 2461350;
export const TU_ANCHOR_INDEX = 21;

export function dayCan(jdn: number): number { return remEuclid(jdn + 9, 10); }
export function dayChi(jdn: number): number { return remEuclid(jdn + 1, 12); }
export function yearCan(ly: number): number { return remEuclid(ly + 6, 10); }
export function yearChi(ly: number): number { return remEuclid(ly + 8, 12); }

export function monthCan(ycan: number, lm: number): number {
  const canThang1 = (ycan % 5) * 2 + 2;
  return (canThang1 + lm - 1) % 10;
}
export function monthChi(lm: number): number { return (lm + 1) % 12; }
export function hourTyCan(dcan: number): number { return (dcan % 5) * 2; }

export function canChiName(can: number, chi: number): string {
  return `${CAN[can]} ${CHI[chi]}`;
}

export function sexagenaryIndex(can: number, chi: number): number { return (6 * can + 55 * chi) % 60; }
export function napAmIndex(can: number, chi: number): number { return Math.trunc(sexagenaryIndex(can, chi) / 2); }
export function trucIndex(dchi: number, tietMonthChi: number): number { return (dchi + 12 - tietMonthChi) % 12; }
export function tuIndex(jdn: number): number { return remEuclid(jdn - TU_ANCHOR_JDN + TU_ANCHOR_INDEX, 28); }
export function thanhLongDayChi(lm: number): number { return ((lm - 1) % 6) * 2; }
export function thanIndex(lm: number, dchi: number): number { return (dchi + 12 - thanhLongDayChi(lm)) % 12; }
export function thanhLongHourChi(dchi: number): number { return (8 + 2 * (dchi % 6)) % 12; }
export function hourThanIndex(dchi: number, hchi: number): number { return (hchi + 12 - thanhLongHourChi(dchi)) % 12; }

/** The six Hoàng đạo hour chi, ordered Tý → Hợi. */
export function gioHoangDao(dchi: number): number[] {
  const out: number[] = [];
  for (let h = 0; h < 12; h++) if (THAN_HOANG_DAO[hourThanIndex(dchi, h)]) out.push(h);
  return out;
}

/** Clock hours [from, to] of hour chi: Tý 23–1, Sửu 1–3, … */
export function hourRange(hchi: number): [number, number] {
  return [(hchi * 2 + 23) % 24, hchi * 2 + 1];
}
