// Unit tests for the TypeScript memorial port (mirror of memorial/tests/fixtures.rs, fixed cases of
// docs/v0.2/02). Full byte-parity with Rust is checked by scripts/check-parity.sh.
// Usage: node tsport/test/memorial_unit.mjs   (after `npm run build` in tsport/)
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO = join(HERE, "..", "..");
const M = await import(join(HERE, "..", "dist", "memorial.mjs"));
const cal = M.Cal.get("precise");

let pass = 0, fail = 0;
function eq(a, b, msg) {
  const x = JSON.stringify(a), y = JSON.stringify(b);
  if (x === y) pass++;
  else { fail++; console.log(`FAIL ${msg}\n  got:  ${x}\n  want: ${y}`); }
}
const gio = (dv, y) => {
  const d = M.saveDeath(dv, null, 60);
  const a = M.anniversary(cal, d.lunar.year, d.lunar.month, d.lunar.day, 2, y);
  return a && [a.year, a.month, a.day, M.iso(a.solar), a.n, a.label];
};

// 02 §2.4
eq(gio(M.dvSolar(2025, 8, 8), 2026), [2026, 6, 15, "2026-07-28", 1, "gio_dau"], "G-A 2026");
eq(gio(M.dvSolar(2025, 8, 8), 2027), [2027, 6, 15, "2027-07-18", 2, "gio_het"], "G-A 2027");
eq(gio(M.dvSolar(2025, 9, 21), 2026), [2026, 7, 29, "2026-09-10", 1, "gio_dau"], "G-B 2026");
eq(gio(M.dvSolar(2025, 9, 21), 2027), [2027, 7, 30, "2027-08-31", 2, "gio_het"], "G-B 2027");
eq(gio(M.dvLunar(null, 3, 10, false), 2026), [2026, 3, 10, "2026-04-26", null, "gio"], "G-C 2026");
eq(gio(M.dvLunar(null, 3, 10, false), 2027), [2027, 3, 10, "2027-04-16", null, "gio"], "G-C 2027");
eq(gio(M.dvSolar(2024, 2, 9), 2024), [2024, 12, 29, "2025-01-28", 1, "gio_dau"], "G-D 2024");
eq(gio(M.dvSolar(2024, 2, 9), 2025), [2025, 12, 29, "2026-02-16", 2, "gio_het"], "G-D 2025");
eq(gio(M.dvLunar(2027, 5, 15, false), 2028), [2028, 5, 15, "2028-06-07", 1, "gio_dau"], "G-E 2028");
eq(gio(M.dvSolar(2025, 8, 8), 2025), null, "no giỗ in death year");
eq([M.labelOf(2, 3), M.labelOf(3, 3), M.labelOf(null, 2)], ["gio", "gio_het", "gio"], "label_of O-2");

// 02 §4
const b90 = M.deriveBirth(cal, M.dvSolar(1990, 1, 20));
eq([b90.lunar_year, b90.can_chi], [1989, "Kỷ Tỵ"], "1990-01-20 → Kỷ Tỵ");
eq([M.tuoiDuong(M.ymd(1990, 1, 20), M.ymd(2026, 11, 6), "feb28"), M.tuoiAm(cal, 1989, M.ymd(2026, 11, 6))], [36, 38], "§4.3 ages");
const b40 = M.deriveBirth(cal, M.dvSolar(1940, 5, 1));
eq(M.saveDeath(M.dvSolar(2025, 8, 8), b40, 60).tuoi, { value: 86, label: "huong_tho", manual: false, approx: false }, "86 hưởng thọ");
eq(M.solarOccurrence(M.ymd(2000, 2, 29), 2100, "feb28"), M.ymd(2100, 2, 28), "29/02 → 28/02/2100");
eq(M.solarOccurrence(M.ymd(2000, 2, 29), 2100, "mar1"), M.ymd(2100, 3, 1), "29/02 → 01/03/2100");

// 01 §6.1 messages
const err = (f) => { try { f(); return null; } catch (e) { return [e.code, e.message]; } };
eq(err(() => M.saveDeath(M.dvLunar(2026, 7, 30, false), null, 60)), ["LunarDayMissing", "Tháng 7 âm lịch năm 2026 chỉ có 29 ngày"], "tháng thiếu");
eq(err(() => M.saveDeath(M.dvLunar(2026, 6, 1, true), null, 60)), ["LunarMonthMissing", "Năm 2026 không có tháng 6 nhuận"], "không có nhuận");

// 02 §7.1 + O-9 + 02 §8.1
const ds = M.parseDataset(readFileSync(join(REPO, "docs/v0.2/schema/example-backup-v1.json"), "utf8"));
const tz = M.Zone.vietnam();
const p = ds.persons[0];
const occ = M.occurrences(cal, ds, M.ymd(2026, 7, 1), M.ymd(2026, 7, 31)).find((o) => o.entity_id === p.id);
const now = tz.localToUtc(M.localMinutes(M.ymd(2026, 1, 1), 0));
eq(M.remindersFor(true, p.rule_gio, occ, now, tz).map((r) => [r.id, M.fmtLocal(r.at_local)]), [
  ["lvhv:0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e11:2026:0", "2026-07-28T07:00"],
  ["lvhv:0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e11:2026:3", "2026-07-25T07:00"],
], "§7.1 example");
eq(M.notificationText(cal, occ, 3, true), { title: "Còn 3 ngày: Giỗ đầu Ông Nguyễn Văn A", body: "Thứ Ba, 28/07/2026 — 15/6 âm lịch" }, "named text");
eq(M.notificationText(cal, occ, 3, false), { title: "Sắp đến ngày giỗ", body: "" }, "O-9 default hides names");
const ics = M.generateIcs(cal, ds, { today: M.ymd(2026, 7, 1), dtstamp: "20261106T031500Z", only_ids: null });
eq(ics.replaceAll("\r\n ", "").includes("SUMMARY:Giỗ đầu Ông Nguyễn Văn A\r\nDESCRIPTION:15/6 âm lịch năm Bính Ngọ (2026). Hưởng thọ 86 tuổi.\r\n"), true, "ics VEVENT");
eq([M.trigger(3, 420), M.trigger(0, 420), M.trigger(1, 0), M.trigger(0, 0)], ["-P2DT17H", "PT7H", "-P1D", "PT0S"], "triggers");
eq(ics.split("\r\n").every((l) => Buffer.byteLength(l) <= 75), true, "≤ 75 octets");
eq(M.cmpCodepoints("\u{1F600}", "￿") > 0, true, "code-point order (not UTF-16)");

// 02 §7.4 (Bổ sung B2): thông báo nhắc mở ứng dụng cùng phút với cái thứ 63, đứng sau nó.
{
  const all = [];
  for (let i = 0; i < 62; i++) all.push({ id: `lvhv:${i.toString(16).padStart(8, "0")}-0000-4000-8000-000000000000:2026:0`, at: 1000 + i, content: "" });
  for (let k = 0; k < 3; k++) all.push({ id: `lvhv:ffffffff-0000-4000-8000-00000000000${k}:2026:0`, at: 5000, content: "" });
  const plan = M.reschedule(0, all, []);
  const s = plan.keep.find((p) => p.id === M.SENTINEL_ID);
  eq([plan.keep.length, s.at], [64, 5000], "R12 sentinel same minute as 63rd");
  const q = M.applyPlan([], plan);
  eq([q[62].at, q[63].id], [5000, M.SENTINEL_ID], "R12 sentinel ordered after 63rd");
}

// R13 (02 §7.6): biên kỳ, (c) nhất quán, (d) ngày đầu/cuối, quy tắc lọc sự kiện.
{
  const calJ = M.Cal.get("jpl");
  const r3 = M.parseDataset(readFileSync(join(REPO, "memorial/testdata/r3-dataset.json"), "utf8"));
  const lo = M.ymd(2026, 1, 1), hi = M.ymd(2036, 12, 31);
  const k = (it) => `${it.class}|${it.entity_id}|${it.key}`;
  const keys = (p) => p.items.map(k);
  const clip = (p, f, t) => p.items.filter((i) => M.cmpYmd(f, i.date) <= 0 && M.cmpYmd(i.date, t) <= 0).map(k);
  const B = (kind, d) => { const r = M.periodBounds(kind, d); return [M.iso(r.from), M.iso(r.to)]; };
  eq(B("tuan", M.ymd(2026, 1, 1)), ["2025-12-29", "2026-01-04"], "week Mon–Sun");
  eq(B("tuan", M.ymd(2026, 1, 4)), ["2025-12-29", "2026-01-04"], "Sunday ends the week");
  eq(B("thang", M.ymd(2028, 2, 10)), ["2028-02-01", "2028-02-29"], "month");
  eq(B("quy", M.ymd(2026, 8, 31)), ["2026-07-01", "2026-09-30"], "quarter");
  eq(B("nam", M.ymd(2026, 8, 31)), ["2026-01-01", "2026-12-31"], "year");
  const pr = M.prevPeriod("quy", M.ymd(2026, 2, 1)), nx = M.nextPeriod("thang", M.ymd(2026, 12, 5));
  eq([M.iso(pr.from), M.iso(pr.to), M.iso(nx.from), M.iso(nx.to)], ["2025-10-01", "2025-12-31", "2027-01-01", "2027-01-31"], "prev/next");
  eq(M.PERIOD_KINDS.map((kd) => M.periodsCovering(kd, lo, hi).length), [575, 132, 44, 11], "period counts");
  let okC = true, okDup = true, okF = true, okD = true, n = 0;
  for (const f of M.PERSON_FILTERS) {
    for (const m of M.periodsCovering("thang", lo, hi)) {
      const month = M.eventsInPeriod(calJ, r3, "thang", m.anchor, f);
      const fromWeeks = M.periodsCovering("tuan", m.from, m.to).flatMap((w) => clip(M.eventsInPeriod(calJ, r3, "tuan", w.anchor, f), m.from, m.to));
      if (JSON.stringify(fromWeeks) !== JSON.stringify(keys(month))) okC = false;
      if (new Set(keys(month)).size !== month.items.length) okDup = false;
      n += month.items.length;
    }
    for (const q of M.periodsCovering("quy", lo, hi)) {
      const months = M.periodsCovering("thang", q.from, q.to).flatMap((m) => keys(M.eventsInPeriod(calJ, r3, "thang", m.anchor, f)));
      if (JSON.stringify(months) !== JSON.stringify(keys(M.eventsInPeriod(calJ, r3, "quy", q.anchor, f)))) okC = false;
    }
    for (const y of M.periodsCovering("nam", lo, hi)) {
      const qs = M.periodsCovering("quy", y.from, y.to).flatMap((q) => keys(M.eventsInPeriod(calJ, r3, "quy", q.anchor, f)));
      if (JSON.stringify(qs) !== JSON.stringify(keys(M.eventsInPeriod(calJ, r3, "nam", y.anchor, f)))) okC = false;
    }
  }
  const wide = M.occurrencesWithTt(calJ, r3, M.ymd(2025, 12, 1), M.ymd(2037, 1, 31));
  let hits = 0;
  for (const kd of M.PERIOD_KINDS) {
    for (const p of M.periodsCovering(kd, lo, hi)) {
      const all = M.eventsInPeriod(calJ, r3, kd, p.anchor, "tat_ca");
      const dm = keys(M.eventsInPeriod(calJ, r3, kd, p.anchor, "da_mat"));
      const dsg = keys(M.eventsInPeriod(calJ, r3, kd, p.anchor, "dang_song"));
      const s = new Set(dm);
      if (JSON.stringify(keys(all).filter((x) => s.has(x))) !== JSON.stringify(dm)) okF = false;
      if (JSON.stringify(keys(all).filter((x) => !s.has(x))) !== JSON.stringify(dsg)) okF = false;
      const want = wide.filter((o) => M.cmpYmd(p.from, o.date) <= 0 && M.cmpYmd(o.date, p.to) <= 0).map((o) => `${o.class}|${o.entity_id}|${M.keyText(o.key)}`);
      if (JSON.stringify(want) !== JSON.stringify(keys(all))) okD = false;
      hits += all.items.filter((i) => M.cmpYmd(i.date, p.from) === 0 || M.cmpYmd(i.date, p.to) === 0).length;
    }
  }
  eq([okC, okDup, okF, okD, n > 0, hits > 0], [true, true, true, true, true, true], "R13(c)(d) on R3 dataset 2026–2036");
  // Sự kiện đúng tại biên.
  const ev = (id, y, m, d) => ({ id, type_id: "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0001", tieu_de: `Biên ${id}`, date: M.dvSolar(y, m, d), recurring: false, person_ids: [], reminder: null, rev: 1 });
  const syn = { reminders_enabled: true, persons: [], events: [ev("e1", 2027, 1, 1), ev("e2", 2027, 3, 31), ev("e3", 2027, 12, 31), ev("e4", 2027, 6, 7), ev("e5", 2027, 6, 13)], event_types: [] };
  const has = (kd, a, id) => M.eventsInPeriod(calJ, syn, kd, a, "tat_ca").items.some((i) => i.entity_id === id);
  eq([
    has("nam", M.ymd(2027, 7, 1), "e1"), has("nam", M.ymd(2027, 7, 1), "e3"), has("quy", M.ymd(2027, 2, 1), "e2"),
    has("quy", M.ymd(2027, 4, 1), "e2"), has("tuan", M.ymd(2027, 6, 10), "e4"), has("tuan", M.ymd(2027, 6, 10), "e5"),
    has("tuan", M.ymd(2027, 6, 14), "e5"),
  ], [true, true, true, false, true, true, false], "R13(d) boundary days");
  // Quy tắc lọc sự kiện nhiều người.
  const r3b = structuredClone(r3);
  const dead = r3b.persons.filter((p) => p.death !== null).map((p) => p.id);
  const alive = r3b.persons.find((p) => p.death === null).id;
  r3b.events[0].person_ids = [dead[0]];
  r3b.events[1].person_ids = [dead[0], alive];
  r3b.events[2].person_ids = [dead[0], dead[1]];
  const dmY = M.eventsInPeriod(calJ, r3b, "nam", M.ymd(2027, 6, 1), "da_mat").items.map((i) => i.entity_id);
  eq([0, 1, 2, 3].map((i) => dmY.includes(r3b.events[i].id)), [true, false, true, false], "event filter rule");
}

console.log(`memorial_unit: ${pass} passed, ${fail} failed`);
process.exit(fail ? 1 : 0);
