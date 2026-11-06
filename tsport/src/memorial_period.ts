// Port of memorial/src/period.rs — "Danh sách theo kỳ" (R13, docs/v0.2/02 §7.6) and the `r13` dump
// line (02 Phụ lục A.6); byte-identical to Rust.
//
// API cho PWA (đúng chữ ký đã thống nhất):
//   eventsInPeriod(cal, dataset, kind: "tuan"|"thang"|"quy"|"nam", anchor: Ymd,
//                  filter: "tat_ca"|"da_mat"|"dang_song") -> { kind, anchor, filter, from, to, items }
//   periodBounds(kind, anchor) -> { from, to }        (tuần: Thứ Hai → Chủ nhật)
//   prevPeriod(kind, anchor) / nextPeriod(kind, anchor) -> { from, to }   (kỳ liền trước / liền sau)
// Mỗi item: { date, lunar: {y,m,d,leap} | null, class, entity_id, person_ids, name, key, n, label, occ }
// (`occ` là Occurrence đầy đủ — headline, display… — để giao diện hiển thị).

import { daysInMonth, weekday } from "./jd.ts";
import { solarToLunarWith } from "./lunar.ts";
import type { Cal } from "./memorial_cal.ts";
import type { GioLabel } from "./memorial_gio.ts";
import { jstr } from "./memorial_json.ts";
import { type Dataset, type Ymd, cmpYmd, iso, ymd, ymdFromJdn, ymdJdn } from "./memorial_model.ts";
import { type OccClass, type Occurrence, classGroup, eventTitle, keyText, occurrencesWithTt } from "./memorial_occ.ts";

export type PeriodKind = "tuan" | "thang" | "quy" | "nam";
export type PersonFilter = "tat_ca" | "da_mat" | "dang_song";
export const PERIOD_KINDS: readonly PeriodKind[] = ["tuan", "thang", "quy", "nam"];
export const PERSON_FILTERS: readonly PersonFilter[] = ["tat_ca", "da_mat", "dang_song"];

export interface PeriodRange { from: Ymd; to: Ymd }
export interface LunarYmd { y: number; m: number; d: number; leap: boolean }
export interface PeriodItem {
  date: Ymd;
  lunar: LunarYmd | null;
  class: OccClass;
  entity_id: string;
  person_ids: string[];
  name: string;
  key: string;
  n: number | null;
  label: GioLabel | null;
  occ: Occurrence;
}
export interface Period { kind: PeriodKind; anchor: Ymd; filter: PersonFilter; from: Ymd; to: Ymd; items: PeriodItem[] }

export function periodBounds(kind: PeriodKind, anchor: Ymd): PeriodRange {
  switch (kind) {
    case "tuan": {
      const j = ymdJdn(anchor);
      const from = j - ((weekday(j) + 6) % 7);
      return { from: ymdFromJdn(from), to: ymdFromJdn(from + 6) };
    }
    case "thang":
      return { from: ymd(anchor.y, anchor.m, 1), to: ymd(anchor.y, anchor.m, daysInMonth(anchor.y, anchor.m)) };
    case "quy": {
      const q = Math.floor((anchor.m - 1) / 3);
      const m1 = 3 * q + 3;
      return { from: ymd(anchor.y, 3 * q + 1, 1), to: ymd(anchor.y, m1, daysInMonth(anchor.y, m1)) };
    }
    case "nam":
      return { from: ymd(anchor.y, 1, 1), to: ymd(anchor.y, 12, 31) };
  }
}

export function prevPeriod(kind: PeriodKind, anchor: Ymd): PeriodRange {
  return periodBounds(kind, ymdFromJdn(ymdJdn(periodBounds(kind, anchor).from) - 1));
}
export function nextPeriod(kind: PeriodKind, anchor: Ymd): PeriodRange {
  return periodBounds(kind, ymdFromJdn(ymdJdn(periodBounds(kind, anchor).to) + 1));
}

const isEvent = (c: OccClass) => classGroup(c) === 3;

/** 02 §7.6: "đã mất" = hồ sơ có ngày mất; sự kiện có person_ids không rỗng và mọi id trỏ tới hồ sơ có ngày mất. */
export function isDeceasedOcc(ds: Dataset, occ: Occurrence): boolean {
  if (isEvent(occ.class)) {
    const ev = ds.events.find((e) => e.id === occ.entity_id);
    if (ev === undefined) return false;
    return ev.person_ids.length > 0 && ev.person_ids.every((id) => ds.persons.find((p) => p.id === id)?.death != null);
  }
  return ds.persons.find((p) => p.id === occ.entity_id)?.death != null;
}

export function passes(ds: Dataset, occ: Occurrence, filter: PersonFilter): boolean {
  if (filter === "tat_ca") return true;
  return isDeceasedOcc(ds, occ) === (filter === "da_mat");
}

export function periodItem(cal: Cal, ds: Dataset, occ: Occurrence): PeriodItem {
  let lunar: LunarYmd | null = null;
  try {
    const l = solarToLunarWith(cal.method, occ.date.y, occ.date.m, occ.date.d);
    lunar = { y: l.y, m: l.m, d: l.d, leap: l.leap };
  } catch { lunar = null; }
  let personIds: string[];
  let name: string;
  if (isEvent(occ.class)) {
    const ev = ds.events.find((e) => e.id === occ.entity_id);
    personIds = ev === undefined ? [] : ev.person_ids.slice();
    name = ev === undefined ? occ.display : eventTitle(ds, ev);
  } else {
    personIds = [occ.entity_id];
    name = occ.display;
  }
  const label = occ.class === "gio" || occ.class === "tien_thuong" ? occ.gio_label : null;
  return {
    date: occ.date, lunar, class: occ.class, entity_id: occ.entity_id, person_ids: personIds, name,
    key: keyText(occ.key), n: occ.n, label, occ,
  };
}

export function eventsInPeriod(cal: Cal, dataset: Dataset, kind: PeriodKind, anchor: Ymd, filter: PersonFilter): Period {
  const { from, to } = periodBounds(kind, anchor);
  const items = occurrencesWithTt(cal, dataset, from, to)
    .filter((o) => passes(dataset, o, filter))
    .map((o) => periodItem(cal, dataset, o));
  return { kind, anchor, filter, from, to, items };
}

/** Periods (full bounds) with at least one day in [from, to], in order, with anchor = max(period.from, from). */
export function periodsCovering(kind: PeriodKind, from: Ymd, to: Ymd): { anchor: Ymd; from: Ymd; to: Ymd }[] {
  const v: { anchor: Ymd; from: Ymd; to: Ymd }[] = [];
  if (cmpYmd(from, to) > 0) return v;
  let p = periodBounds(kind, from);
  while (cmpYmd(p.from, to) <= 0) {
    v.push({ anchor: cmpYmd(p.from, from) >= 0 ? p.from : from, from: p.from, to: p.to });
    p = periodBounds(kind, ymdFromJdn(ymdJdn(p.to) + 1));
  }
  return v;
}

export function itemJson(it: PeriodItem): string {
  const l = it.lunar === null ? "null" : `{"y":${it.lunar.y},"m":${it.lunar.m},"d":${it.lunar.d},"leap":${it.lunar.leap}}`;
  return (
    `{"date":"${iso(it.date)}","lunar":${l},"class":"${it.class}","entity_id":${jstr(it.entity_id)},` +
    `"person_ids":[${it.person_ids.map(jstr).join(",")}],"name":${jstr(it.name)},"key":${jstr(it.key)},` +
    `"n":${it.n === null ? "null" : it.n},"label":${it.label === null ? "null" : `"${it.label}"`}}`
  );
}

export function periodLine(out: { push: (s: string) => void }, p: Period): void {
  out.push(
    `{"kind":"${p.kind}","anchor":"${iso(p.anchor)}","from":"${iso(p.from)}","to":"${iso(p.to)}","filter":"${p.filter}","items":[` +
    p.items.map(itemJson).join(",") + "]}\n",
  );
}

export function r13Lines(cal: Cal, ds: Dataset, from: Ymd, to: Ymd, out: { push: (s: string) => void }): void {
  for (const kind of PERIOD_KINDS) {
    for (const { anchor } of periodsCovering(kind, from, to)) {
      for (const f of PERSON_FILTERS) periodLine(out, eventsInPeriod(cal, ds, kind, anchor, f));
    }
  }
}
