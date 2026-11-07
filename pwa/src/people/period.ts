// R13 — danh sách sự kiện theo kỳ (docs/v0.2/08 §8.2). Dùng thẳng hàm của lõi (`tsport/src/memorial_period.ts`,
// cổng của memorial/src/period.rs): kỳ, kỳ trước/sau, bộ lọc 02 §7.6 (Bổ sung B4) — PWA không tự tính lại.
export {
  eventsInPeriod, periodBounds, prevPeriod, nextPeriod,
  type Period, type PeriodItem, type PeriodKind, type PeriodRange, type PersonFilter,
} from "../../../tsport/src/memorial_period.ts";
