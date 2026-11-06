// Kiểu dữ liệu theo hợp đồng kiến trúc (docs/01-HOP-DONG-KIEN-TRUC.md, mục 4).
// DayInfo khớp từng khóa của JSON trong hợp đồng. GoodDay khớp `GoodDay` của lõi
// (core/src/gooddays.rs, tsport/src/gooddays.ts). Cadao khớp lược đồ content/cadao/README.md.

/** Phương pháp âm lịch của lõi (Rust `Method::Hnd` / `Method::Precise`). */
export type Method = "hnd" | "precise";

export interface SolarInfo {
  y: number;
  m: number;
  d: number;
  /** 0 = Chủ nhật, 1 = Thứ Hai, …, 6 = Thứ Bảy */
  weekday: number;
  jdn: number;
}

export interface LunarInfo {
  y: number;
  m: number;
  d: number;
  leap: boolean;
  month_len: number;
}

export interface CanChi {
  year: string;
  month: string;
  day: string;
  hour_ty: string;
}

export interface Than {
  name: string;
  hoang_dao: boolean;
}

export interface GioHoangDao {
  chi: string;
  /** giờ đồng hồ bắt đầu (Tý: 23) */
  from: number;
  /** giờ đồng hồ kết thúc (Tý: 1) */
  to: number;
}

export interface NgocHap {
  sao_tot: string[];
  sao_xau: string[];
  nen: string[];
  ky: string[];
  xung_tuoi: string[];
  score: number;
}

export interface DayInfo {
  solar: SolarInfo;
  lunar: LunarInfo;
  can_chi: CanChi;
  tiet_khi: string;
  nap_am: string;
  truc: string;
  tu: string;
  than: Than;
  gio_hoang_dao: GioHoangDao[];
  ngoc_hap: NgocHap;
}

export interface SolarDate {
  y: number;
  m: number;
  d: number;
}

export interface LunarDate {
  y: number;
  m: number;
  d: number;
  leap: boolean;
}

export interface GoodDay {
  solar: SolarDate;
  lunar?: LunarInfo;
  weekday?: number;
  jdn?: number;
  score: number;
  /** luôn true trong kết quả (ngày phạm kỵ chính hoặc xung tuổi đã bị loại) */
  allowed?: boolean;
  /** lý do theo thứ tự cố định, tiếng Việt (ví dụ "+2 Sát Cống", "−1 Trực Nguy") */
  reasons: string[];
  first_gio_hoang_dao?: GioHoangDao;
}

export interface Activity {
  id: string;
  name: string;
}

export interface CadaoSource {
  title: string;
  url: string;
  license: string;
}

/** Một mục của content/cadao/cadao.json (giữ đúng tên trường). */
export interface Cadao {
  id: string;
  /** "ca dao" | "tục ngữ" | "thành ngữ" */
  type: string;
  text: string;
  lines: string[];
  theme: string;
  meaning: string;
  sources: CadaoSource[];
  order: number;
}

export interface SourceBlock {
  kind: "h" | "p" | "li" | "quote";
  text: string;
}

/** Một dòng bảng nguồn (content/ngochap/SOURCES.md), dựng lúc build. */
export interface RuleSource {
  code: string;
  title: string;
  url: string;
  lineage: string;
  usedFor: string;
}
