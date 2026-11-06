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

export type PeriodKind = "tuan" | "thang" | "quy" | "nam";
export type PeriodFilter = "tat_ca" | "da_mat" | "dang_song";
type Ymd = M.Ymd;

export interface PeriodResult { from: Ymd; to: Ymd; items: M.Occurrence[] }

const dim = (y: number, m: number) => new Date(Date.UTC(y, m, 0)).getUTCDate();

/** Kỳ chứa ngày neo: tuần Thứ Hai → Chủ nhật; tháng, quý (1–3, 4–6, 7–9, 10–12), năm dương lịch. Gồm cả hai đầu. */
export function periodBounds(kind: PeriodKind, anchor: Ymd): { from: Ymd; to: Ymd } {
  if (kind === "tuan") {
    const j = M.ymdJdn(anchor);
    const wd = (j + 1) % 7; // 0 = Chủ nhật (như jd.weekday của lõi)
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
 * Lọc bộ dữ liệu: hồ sơ theo `kind`; sự kiện theo hồ sơ ĐẦU TIÊN trong `person_ids` (giao diện chỉ tạo sự kiện một
 * người); sự kiện không còn người nào thuộc "đang sống" — để "đã mất" ∪ "đang sống" = "tất cả", không trùng.
 * Quy ước này phải được đối chiếu với hàm của vai lõi khi thay.
 */
function filterDataset(ds: M.Dataset, filter: PeriodFilter): M.Dataset {
  if (filter === "tat_ca") return ds;
  const want: M.PersonKind = filter === "da_mat" ? "deceased" : "living";
  const kindOf = new Map(ds.persons.map((p) => [p.id, p.kind] as const));
  return {
    ...ds,
    persons: ds.persons.filter((p) => p.kind === want),
    events: ds.events.filter((e) => (kindOf.get(e.person_ids[0] ?? "") ?? "living") === want),
  };
}

/** R13: mọi lần xuất hiện (giỗ, tiên thường nếu bật, sinh nhật dương/âm, sự kiện) có ngày dương trong kỳ, thứ tự 02 §7.2. */
export function eventsInPeriod(cal: M.Cal, ds: M.Dataset, kind: PeriodKind, anchor: Ymd, filter: PeriodFilter): PeriodResult {
  const { from, to } = periodBounds(kind, anchor);
  return { from, to, items: M.occurrencesWithTt(cal, filterDataset(ds, filter), from, to) };
}
