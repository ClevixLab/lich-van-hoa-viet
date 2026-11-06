// ENGINE MOCK — chỉ dùng khi chưa có tsport/src/index.ts.
// Mục đích: cho giao diện có dữ liệu trông thật để thử nghiệm. KHÔNG phải lõi lịch
// chính thức, KHÔNG được dùng làm căn cứ kiểm định. Giao diện hiển thị nhãn MOCK.
//
// - Ngày âm: thuật toán công khai của Hồ Ngọc Đức (Meeus rút gọn), múi giờ UTC+7.
// - Can Chi, giờ Tý, nạp âm, trực, thần, giờ hoàng đạo: công thức truyền thống thông dụng.
// - Tú: chu kỳ 28 ngày, mốc canh theo ví dụ 2026-11-05 = Tỉnh.
// - Sao tốt/xấu, nên/kỵ, điểm: GIẢ LẬP (chọn theo JDN), không theo Ngọc Hạp thật,
//   trừ ngày 2026-11-05 được ghi cứng giá trị mẫu.

import type { Activity, DayInfo, GoodDay, LunarDate, SolarDate } from "./types";

const TZ = 7;
const PI = Math.PI;
const INT = Math.floor;

export const CAN = ["Giáp", "Ất", "Bính", "Đinh", "Mậu", "Kỷ", "Canh", "Tân", "Nhâm", "Quý"];
export const CHI = ["Tý", "Sửu", "Dần", "Mão", "Thìn", "Tỵ", "Ngọ", "Mùi", "Thân", "Dậu", "Tuất", "Hợi"];

const TIET_KHI = [
  "Xuân Phân", "Thanh Minh", "Cốc Vũ", "Lập Hạ", "Tiểu Mãn", "Mang Chủng",
  "Hạ Chí", "Tiểu Thử", "Đại Thử", "Lập Thu", "Xử Thử", "Bạch Lộ",
  "Thu Phân", "Hàn Lộ", "Sương Giáng", "Lập Đông", "Tiểu Tuyết", "Đại Tuyết",
  "Đông Chí", "Tiểu Hàn", "Đại Hàn", "Lập Xuân", "Vũ Thủy", "Kinh Trập",
];

const NAP_AM = [
  "Hải Trung Kim", "Lư Trung Hỏa", "Đại Lâm Mộc", "Lộ Bàng Thổ", "Kiếm Phong Kim",
  "Sơn Đầu Hỏa", "Giản Hạ Thủy", "Thành Đầu Thổ", "Bạch Lạp Kim", "Dương Liễu Mộc",
  "Tuyền Trung Thủy", "Ốc Thượng Thổ", "Tích Lịch Hỏa", "Tùng Bách Mộc", "Trường Lưu Thủy",
  "Sa Trung Kim", "Sơn Hạ Hỏa", "Bình Địa Mộc", "Bích Thượng Thổ", "Kim Bạch Kim",
  "Phú Đăng Hỏa", "Thiên Hà Thủy", "Đại Trạch Thổ", "Thoa Xuyến Kim", "Tang Đố Mộc",
  "Đại Khê Thủy", "Sa Trung Thổ", "Thiên Thượng Hỏa", "Thạch Lựu Mộc", "Đại Hải Thủy",
];

const TRUC = ["Kiến", "Trừ", "Mãn", "Bình", "Định", "Chấp", "Phá", "Nguy", "Thành", "Thu", "Khai", "Bế"];

const TU = [
  "Giác", "Cang", "Đê", "Phòng", "Tâm", "Vĩ", "Cơ", "Đẩu", "Ngưu", "Nữ", "Hư", "Nguy", "Thất", "Bích",
  "Khuê", "Lâu", "Vị", "Mão", "Tất", "Chủy", "Sâm", "Tỉnh", "Quỷ", "Liễu", "Tinh", "Trương", "Dực", "Chẩn",
];

const THAN: Array<[string, boolean]> = [
  ["Thanh Long", true], ["Minh Đường", true], ["Thiên Hình", false], ["Chu Tước", false],
  ["Kim Quỹ", true], ["Kim Đường", true], ["Bạch Hổ", false], ["Ngọc Đường", true],
  ["Thiên Lao", false], ["Huyền Vũ", false], ["Tư Mệnh", true], ["Câu Trận", false],
];

const GIO_HD = ["110100101100", "001101001011", "110011010010", "101100110100", "001011001101", "010010110011"];

const SAO_TOT = ["Thiên Đức", "Nguyệt Đức", "Thiên Hỷ", "Thiên Phúc", "Sinh Khí", "Ích Hậu", "Tục Thế", "Mẫu Thương", "Phúc Sinh", "Thiên Quý", "Lộc Khố", "Dịch Mã"];
const SAO_XAU = ["Thiên Cương", "Địa Phá", "Hoang Vu", "Nguyệt Phá", "Thổ Phủ", "Băng Tiêu Ngọa Hãm", "Hà Khôi", "Cô Thần", "Quả Tú", "Lục Bất Thành", "Nguyệt Hỏa", "Thiên Tặc"];

const ACTIVITIES: Activity[] = [
  { id: "cuoi_hoi", name: "Cưới hỏi" },
  { id: "dong_tho", name: "Động thổ, làm nhà" },
  { id: "nhap_trach", name: "Về nhà mới" },
  { id: "khai_truong", name: "Khai trương" },
  { id: "ky_ket", name: "Ký kết, giao dịch" },
  { id: "xuat_hanh", name: "Xuất hành, đi xa" },
  { id: "cau_an", name: "Cúng lễ, cầu an" },
  { id: "an_tang", name: "An táng, cải táng" },
];
const ACT_NAME: Record<string, string> = Object.fromEntries(ACTIVITIES.map((a) => [a.id, a.name]));

// ---------- Thiên văn (Hồ Ngọc Đức) ----------
export function jdFromDate(dd: number, mm: number, yy: number): number {
  const a = INT((14 - mm) / 12);
  const y = yy + 4800 - a;
  const m = mm + 12 * a - 3;
  return dd + INT((153 * m + 2) / 5) + 365 * y + INT(y / 4) - INT(y / 100) + INT(y / 400) - 32045;
}

export function jdToDate(jd: number): SolarDate {
  const a = jd + 32044;
  const b = INT((4 * a + 3) / 146097);
  const c = a - INT((b * 146097) / 4);
  const d = INT((4 * c + 3) / 1461);
  const e = c - INT((1461 * d) / 4);
  const m = INT((5 * e + 2) / 153);
  return { d: e - INT((153 * m + 2) / 5) + 1, m: m + 3 - 12 * INT(m / 10), y: b * 100 + d - 4800 + INT(m / 10) };
}

function newMoon(k: number): number {
  const T = k / 1236.85, T2 = T * T, T3 = T2 * T, dr = PI / 180;
  let jd1 = 2415020.75933 + 29.53058868 * k + 0.0001178 * T2 - 0.000000155 * T3;
  jd1 += 0.00033 * Math.sin((166.56 + 132.87 * T - 0.009173 * T2) * dr);
  const M = 359.2242 + 29.10535608 * k - 0.0000333 * T2 - 0.00000347 * T3;
  const Mpr = 306.0253 + 385.81691806 * k + 0.0107306 * T2 + 0.00001236 * T3;
  const F = 21.2964 + 390.67050646 * k - 0.0016528 * T2 - 0.00000239 * T3;
  let C1 = (0.1734 - 0.000393 * T) * Math.sin(M * dr) + 0.0021 * Math.sin(2 * dr * M);
  C1 = C1 - 0.4068 * Math.sin(Mpr * dr) + 0.0161 * Math.sin(dr * 2 * Mpr);
  C1 = C1 - 0.0004 * Math.sin(dr * 3 * Mpr);
  C1 = C1 + 0.0104 * Math.sin(dr * 2 * F) - 0.0051 * Math.sin(dr * (M + Mpr));
  C1 = C1 - 0.0074 * Math.sin(dr * (M - Mpr)) + 0.0004 * Math.sin(dr * (2 * F + M));
  C1 = C1 - 0.0004 * Math.sin(dr * (2 * F - M)) - 0.0006 * Math.sin(dr * (2 * F + Mpr));
  C1 = C1 + 0.001 * Math.sin(dr * (2 * F - Mpr)) + 0.0005 * Math.sin(dr * (2 * Mpr + M));
  const deltat = T < -11
    ? 0.001 + 0.000839 * T + 0.0002261 * T2 - 0.00000845 * T3 - 0.000000081 * T * T3
    : -0.000278 + 0.000265 * T + 0.000262 * T2;
  return jd1 + C1 - deltat;
}

const newMoonDay = (k: number) => INT(newMoon(k) + 0.5 + TZ / 24);

/** Kinh độ Mặt Trời (độ, 0..360) tại thời điểm JD (TT xấp xỉ). */
function sunLongitudeDeg(jdn: number): number {
  const T = (jdn - 2451545.0) / 36525, T2 = T * T, dr = PI / 180;
  const M = 357.5291 + 35999.0503 * T - 0.0001559 * T2 - 0.00000048 * T * T2;
  const L0 = 280.46645 + 36000.76983 * T + 0.0003032 * T2;
  let DL = (1.9146 - 0.004817 * T - 0.000014 * T2) * Math.sin(dr * M);
  DL += (0.019993 - 0.000101 * T) * Math.sin(dr * 2 * M) + 0.00029 * Math.sin(dr * 3 * M);
  const L = L0 + DL;
  return ((L % 360) + 360) % 360;
}

const sunSector = (dayNumber: number) => INT(sunLongitudeDeg(dayNumber - 0.5 - TZ / 24) / 30);

function lunarMonth11(yy: number): number {
  const off = jdFromDate(31, 12, yy) - 2415021;
  const k = INT(off / 29.530588853);
  let nm = newMoonDay(k);
  if (sunSector(nm) >= 9) nm = newMoonDay(k - 1);
  return nm;
}

function leapMonthOffset(a11: number): number {
  const k = INT((a11 - 2415021.076998695) / 29.530588853 + 0.5);
  let last = 0, i = 1;
  let arc = sunSector(newMoonDay(k + i));
  do {
    last = arc;
    i++;
    arc = sunSector(newMoonDay(k + i));
  } while (arc !== last && i < 14);
  return i - 1;
}

interface LunarFull extends LunarDate { monthStart: number; monthLen: number }

function lunarFull(dd: number, mm: number, yy: number): LunarFull {
  const dayNumber = jdFromDate(dd, mm, yy);
  const k = INT((dayNumber - 2415021.076998695) / 29.530588853);
  let monthStart = newMoonDay(k + 1);
  let next = newMoonDay(k + 2);
  if (monthStart > dayNumber) { next = monthStart; monthStart = newMoonDay(k); }
  let a11 = lunarMonth11(yy), b11 = a11, lunarYear: number;
  if (a11 >= monthStart) { lunarYear = yy; a11 = lunarMonth11(yy - 1); }
  else { lunarYear = yy + 1; b11 = lunarMonth11(yy + 1); }
  const lunarDay = dayNumber - monthStart + 1;
  const diff = INT((monthStart - a11) / 29);
  let leap = false, lunarMonth = diff + 11;
  if (b11 - a11 > 365) {
    const lmd = leapMonthOffset(a11);
    if (diff >= lmd) { lunarMonth = diff + 10; if (diff === lmd) leap = true; }
  }
  if (lunarMonth > 12) lunarMonth -= 12;
  if (lunarMonth >= 11 && diff < 4) lunarYear -= 1;
  return { y: lunarYear, m: lunarMonth, d: lunarDay, leap, monthStart, monthLen: next - monthStart };
}

export function solarToLunar(y: number, m: number, d: number): LunarDate {
  const f = lunarFull(d, m, y);
  return { y: f.y, m: f.m, d: f.d, leap: f.leap };
}

export function lunarToSolar(ly: number, lm: number, ld: number, leap: boolean): SolarDate | null {
  if (lm < 1 || lm > 12 || ld < 1 || ld > 30) return null;
  let a11: number, b11: number;
  if (lm < 11) { a11 = lunarMonth11(ly - 1); b11 = lunarMonth11(ly); }
  else { a11 = lunarMonth11(ly); b11 = lunarMonth11(ly + 1); }
  const k = INT(0.5 + (a11 - 2415021.076998695) / 29.530588853);
  let off = lm - 11;
  if (off < 0) off += 12;
  if (b11 - a11 > 365) {
    const leapOff = leapMonthOffset(a11);
    let leapMonth = leapOff - 2;
    if (leapMonth < 0) leapMonth += 12;
    if (leap && lm !== leapMonth) return null;
    if (leap || off >= leapOff) off += 1;
  } else if (leap) {
    return null;
  }
  const monthStart = newMoonDay(k + off);
  const len = newMoonDay(k + off + 1) - monthStart;
  if (ld > len) return null;
  return jdToDate(monthStart + ld - 1);
}

// ---------- Can Chi & quy tắc truyền thống ----------
const mod = (a: number, n: number) => ((a % n) + n) % n;
const canChi = (c: number, z: number) => `${CAN[mod(c, 10)]} ${CHI[mod(z, 12)]}`;
const sexagenary = (c: number, z: number) => mod(6 * c - 5 * z, 60); // chỉ số trong vòng Giáp Tý

export function yearCanChi(y: number): string {
  return canChi(y + 6, y + 8);
}

/** Pseudo-random ổn định theo JDN (chỉ dùng cho phần giả lập). */
function rng(seed: number): () => number {
  let s = seed >>> 0;
  return () => {
    s = (s * 1664525 + 1013904223) >>> 0;
    return s / 4294967296;
  };
}

function pick<T>(arr: readonly T[], n: number, r: () => number): T[] {
  const pool = arr.slice();
  const out: T[] = [];
  while (out.length < n && pool.length) out.push(pool.splice(INT(r() * pool.length), 1)[0]);
  return out;
}

const NEN_POOL = ["Cúng lễ, cầu an", "Ký kết, giao dịch", "Khai trương", "Xuất hành, đi xa", "Cưới hỏi", "Động thổ, làm nhà", "Về nhà mới", "Nhập học", "Sửa chữa nhà cửa", "Gieo trồng", "Thu nợ, đòi nợ", "Cắt may"];
const KY_POOL = ["An táng, cải táng", "Kiện tụng", "Đào ao, giếng", "Động thổ, làm nhà", "Cưới hỏi", "Xuất hành, đi xa", "Khai trương", "Mua xe", "Chuyển nhà"];

function mockDayInfo(y: number, m: number, d: number): DayInfo {
  const jdn = jdFromDate(d, m, y);
  const lf = lunarFull(d, m, y);
  const dayCan = mod(jdn + 9, 10), dayChi = mod(jdn + 1, 12);
  const ly = lf.y;
  const yearCan = mod(ly + 6, 10);
  const monthCan = mod(mod(yearCan, 5) * 2 + 2 + (lf.m - 1), 10);
  const monthChi = mod(lf.m + 1, 12);
  const hourTyCan = mod(dayCan, 5) * 2;

  const endOfDay = jdn + 0.5 - TZ / 24; // 24:00 giờ VN
  const L = sunLongitudeDeg(endOfDay);
  const tietKhi = TIET_KHI[INT(L / 15)];
  const tietMonthChi = mod(INT(mod(L - 315, 360) / 30) + 2, 12);

  const truc = TRUC[mod(dayChi - tietMonthChi, 12)];
  const tu = TU[mod(jdn + 11, 28)];
  const thanStart = [0, 2, 4, 6, 8, 10][mod(monthChi - 2, 6)]; // Dần/Thân→Tý, Mão/Dậu→Dần, …
  const [thanName, hoangDao] = THAN[mod(dayChi - thanStart, 12)];
  const napAm = NAP_AM[INT(sexagenary(dayCan, dayChi) / 2)];

  const bits = GIO_HD[dayChi % 6];
  const gio = [];
  for (let i = 0; i < 12; i++) {
    if (bits[i] === "1") gio.push({ chi: CHI[i], from: mod(2 * i - 1, 24), to: 2 * i + 1 });
  }

  const xungChi = mod(dayChi + 6, 12);
  const xung = [canChi(dayCan + 4, xungChi), canChi(dayCan + 6, xungChi)];

  let ngoc_hap;
  if (y === 2026 && m === 11 && d === 5) {
    ngoc_hap = {
      sao_tot: ["Thiên Phúc", "Mẫu Thương", "Tục Thế"],
      sao_xau: ["Thiên Cương", "Địa Phá", "Hoang Vu", "Thổ Phủ"],
      nen: ["Cúng lễ, cầu an", "Thu nợ, đòi nợ", "Gieo trồng", "Cắt may", "Sửa chữa nhà cửa"],
      ky: ["Động thổ, làm nhà", "Khai trương", "Cưới hỏi", "Xuất hành, đi xa", "An táng, cải táng"],
      xung_tuoi: ["Đinh Sửu", "Kỷ Sửu", "Ất Mùi", "Ất Sửu"],
      score: 42,
    };
  } else {
    const r = rng(jdn);
    const goodTruc = ["Trừ", "Định", "Thành", "Khai", "Mãn"].includes(truc);
    const score = Math.max(5, Math.min(95, Math.round(30 + (hoangDao ? 25 : 0) + (goodTruc ? 15 : 0) + r() * 25 - 5)));
    const nen = pick(NEN_POOL, 2 + INT(r() * 4), r);
    ngoc_hap = {
      sao_tot: pick(SAO_TOT, 1 + INT(r() * 4), r),
      sao_xau: pick(SAO_XAU, 1 + INT(r() * 4), r),
      nen,
      ky: pick(KY_POOL.filter((k) => !nen.includes(k)), 2 + INT(r() * 4), r),
      xung_tuoi: xung,
      score,
    };
  }

  return {
    solar: { y, m, d, weekday: mod(jdn + 1, 7), jdn },
    lunar: { y: ly, m: lf.m, d: lf.d, leap: lf.leap, month_len: lf.monthLen },
    can_chi: {
      year: canChi(yearCan, ly + 8),
      month: canChi(monthCan, monthChi),
      day: canChi(dayCan, dayChi),
      hour_ty: canChi(hourTyCan, 0),
    },
    tiet_khi: tietKhi,
    nap_am: napAm,
    truc,
    tu,
    than: { name: thanName, hoang_dao: hoangDao },
    gio_hoang_dao: gio,
    ngoc_hap,
  };
}

export function dayInfo(y: number, m: number, d: number): DayInfo {
  return mockDayInfo(y, m, d);
}

export function activities(): Activity[] {
  return ACTIVITIES.slice();
}

export function findGoodDays(activityId: string, from: SolarDate, to: SolarDate, birthYear?: number): GoodDay[] {
  const name = ACT_NAME[activityId];
  const a = jdFromDate(from.d, from.m, from.y);
  const b = jdFromDate(to.d, to.m, to.y);
  const birthChi = birthYear === undefined ? -1 : mod(birthYear + 8, 12);
  const out: GoodDay[] = [];
  for (let j = a; j <= b; j++) {
    const s = jdToDate(j);
    const info = mockDayInfo(s.y, s.m, s.d);
    if (name && info.ngoc_hap.ky.includes(name)) continue;
    const dayChi = mod(j + 1, 12);
    if (birthChi >= 0 && mod(dayChi - birthChi, 12) === 6) continue; // xung tuổi
    let score = info.ngoc_hap.score + (name && info.ngoc_hap.nen.includes(name) ? 10 : 0);
    score = Math.min(100, score);
    if (score < 50) continue;
    const reasons: string[] = [];
    reasons.push(info.than.hoang_dao ? `Ngày Hoàng đạo (${info.than.name})` : `Hắc đạo (${info.than.name}) nhưng nhiều sao tốt`);
    reasons.push(`Trực ${info.truc}`);
    if (info.ngoc_hap.sao_tot.length) reasons.push(`Sao tốt: ${info.ngoc_hap.sao_tot.slice(0, 2).join(", ")}`);
    out.push({ solar: s, score, reasons });
  }
  out.sort((p, q) => q.score - p.score || jdFromDate(p.solar.d, p.solar.m, p.solar.y) - jdFromDate(q.solar.d, q.solar.m, q.solar.y));
  return out;
}

// ---------- Bề mặt giống tsport (để engine.ts gọi trực tiếp, A-4) ----------
// Chỉ dùng với `node build.mjs --mock`. Không kiểm miền đầy đủ như lõi thật.
export const MIN_YEAR = 2000;
export const MAX_YEAR = 3000;
export const BIRTH_MIN_YEAR = 1900;
export const MAX_SPAN_DAYS = 366;
export class LvhvError extends RangeError {
  readonly code: string;
  constructor(code: string, message: string) { super(message); this.code = code; }
}
export function dayInfoWith(_m: string, y: number, m: number, d: number): DayInfo { return dayInfo(y, m, d); }
export function solarToLunarWith(_m: string, y: number, m: number, d: number): LunarDate { return solarToLunar(y, m, d); }
export function lunarToSolarWith(_m: string, ly: number, lm: number, ld: number, leap: boolean): SolarDate | null { return lunarToSolar(ly, lm, ld, leap); }
export function findGoodDaysWith(_m: string, id: string, from: SolarDate, to: SolarDate, birth?: number | null): GoodDay[] {
  return findGoodDays(id, from, to, birth ?? undefined);
}
export function lunarYearOfBirthWith(_m: string, y: number, m: number, d: number): number { return solarToLunar(y, m, d).y; }
export const canchi = {
  CAN, CHI,
  yearCan: (y: number) => (((y + 6) % 10) + 10) % 10,
  yearChi: (y: number) => (((y + 8) % 12) + 12) % 12,
};
export const jd = {
  jdnFromDate: (y: number, m: number, d: number) => jdFromDate(d, m, y),
  dateFromJdn: (j: number): [number, number, number] => { const s = jdToDate(j); return [s.y, s.m, s.d]; },
  weekday: (j: number) => (((j + 1) % 7) + 7) % 7,
};
