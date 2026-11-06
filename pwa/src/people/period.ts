// ============================================================================================================
// BỘ CHUYỂN TẠM (R13 — danh sách sự kiện theo kỳ) — docs/v0.2/08 §8.2.
//
// Hàm chính thức `eventsInPeriod` / `periodBounds` / kỳ trước–sau đang được vai lõi thêm vào tsport (nhánh
// `v0.2-w3-core`). Tới khi hàm đó có trên `main`, PWA dựng CÙNG kết quả từ hàm sẵn có của lõi
// (`occurrencesWithTt`, thứ tự 02 §7.2). Mọi màn hình chỉ import từ tệp này.
//
// KHI tsport có hàm chính thức: xóa phần cài đặt bên dưới và thay bằng MỘT dòng, ví dụ
//   export { eventsInPeriod, periodBounds, periodPrev, periodNext } from "../../../tsport/src/memorial.ts";
// (đổi tên periodPrev/periodNext nếu bản lõi đặt tên khác), rồi chạy lại kiểm thử khói.
// Không sửa tsport/ ở đây — tsport thuộc vai lõi.
// ============================================================================================================
import * as M from "../../../tsport/src/memorial.ts";
import { weekday } from "../../../tsport/src/jd.ts";

export type PeriodKind = "tuan" | "thang" | "quy" | "nam";
export type PeriodFilter = "tat_ca" | "da_mat" | "dang_song";
type Ymd = M.Ymd;

export interface PeriodResult { from: Ymd; to: Ymd; items: M.Occurrence[] }

const dim = (y: number, m: number) => new Date(Date.UTC(y, m, 0)).getUTCDate();

/** Kỳ chứa ngày neo: tuần Thứ Hai → Chủ nhật; tháng, quý (1–3, 4–6, 7–9, 10–12), năm dương lịch. Gồm cả hai đầu. */
export function periodBounds(kind: PeriodKind, anchor: Ymd): { from: Ymd; to: Ymd } {
  if (kind === "tuan") {
    const j = M.ymdJdn(anchor);
    const wd = weekday(j); // 0 = Chủ nhật (hàm của lõi, MA3-15-08)
    const mon = j - ((wd + 6) % 7);
    return { from: M.ymdFromJdn(mon), to: M.ymdFromJdn(mon + 6) };
  }
  if (kind === "thang") return { from: M.ymd(anchor.y, anchor.m, 1), to: M.ymd(anchor.y, anchor.m, dim(anchor.y, anchor.m)) };
  if (kind === "quy") {
    const q0 = Math.floor((anchor.m - 1) / 3) * 3 + 1;
    return { from: M.ymd(anchor.y, q0, 1), to: M.ymd(anchor.y, q0 + 2, dim(anchor.y, q0 + 2)) };
  }
  return { from: M.ymd(anchor.y, 1, 1), to: M.ymd(anchor.y, 12, 31) };
}

/** Ngày neo của kỳ trước (ngày đầu kỳ trước). */
export function periodPrev(kind: PeriodKind, anchor: Ymd): Ymd {
  return periodBounds(kind, M.ymdFromJdn(M.ymdJdn(periodBounds(kind, anchor).from) - 1)).from;
}
/** Ngày neo của kỳ sau (ngày đầu kỳ sau). */
export function periodNext(kind: PeriodKind, anchor: Ymd): Ymd {
  return M.ymdFromJdn(M.ymdJdn(periodBounds(kind, anchor).to) + 1);
}

/**
 * Bộ lọc theo Bổ sung B4 mục 3 (04): "Người đã mất" = giỗ và tiên thường; "Người đang sống" = mọi lần xuất hiện còn
 * lại. Bộ dữ liệu vào đã bỏ sự kiện gắn người đã mất (core.ts `snap`), nên "đã mất" ∪ "đang sống" = "tất cả", không trùng.
 */
const isDeadClass = (o: M.Occurrence) => o.class === "gio" || o.class === "tien_thuong";

/** R13: mọi lần xuất hiện (giỗ, tiên thường nếu bật, sinh nhật dương/âm, sự kiện) có ngày dương trong kỳ, thứ tự 02 §7.2. */
export function eventsInPeriod(cal: M.Cal, ds: M.Dataset, kind: PeriodKind, anchor: Ymd, filter: PeriodFilter): PeriodResult {
  const { from, to } = periodBounds(kind, anchor);
  const all = M.occurrencesWithTt(cal, ds, from, to);
  return { from, to, items: filter === "tat_ca" ? all : all.filter((o) => isDeadClass(o) === (filter === "da_mat")) };
}
