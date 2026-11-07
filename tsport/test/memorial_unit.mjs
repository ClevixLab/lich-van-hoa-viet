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
  const plan = M.reschedule(0, all, [], M.Zone.fixed(0));
  const s = plan.keep.find((p) => p.id === M.SENTINEL_ID);
  eq([plan.keep.length, s.at], [64, 5000], "R12 sentinel same minute as 63rd");
  const q = M.applyPlan([], plan);
  eq([q[62].at, q[63].id], [5000, M.SENTINEL_ID], "R12 sentinel ordered after 63rd");
}

// 02 §7.4 (Bổ sung B3): lời nhắc ngoài cửa sổ 400 ngày → thông báo nhắc mở ứng dụng (I4).
{
  const hi = M.WINDOW_DAYS * 1440;
  const p = (i, at) => ({ id: `lvhv:${i.toString(16).padStart(8, "0")}-0000-4000-8000-000000000000:2026:0`, at, content: "" });
  const w = [...Array(10).keys()].map((i) => p(i, 1000 + i));
  const k1 = M.reschedule(0, [...w, p(99, hi + 5)], [], M.Zone.fixed(0)).keep;
  eq([k1.length, k1.at(-1).id, k1.at(-1).at], [11, M.SENTINEL_ID, 1009], "B3 sentinel after last scheduled");
  const k2 = M.reschedule(0, [p(1, hi + 1)], [], M.Zone.fixed(0)).keep;
  eq(k2.map((x) => [x.id, x.at]), [[M.SENTINEL_ID, 399 * 1440 + 540]], "sentinel at 09:00 of day today + 399 when nothing in window (MA3-14-06)");
  const w64 = [...Array(64).keys()].map((i) => p(i, 1000 + i));
  const k3 = M.reschedule(0, [...w64, p(99, hi + 5)], [], M.Zone.fixed(0)).keep;
  eq([k3.length, k3[63].id, k3[63].at], [64, M.SENTINEL_ID, 1062], "B3 64 in window + beyond");
  eq(M.reschedule(0, w64, [], M.Zone.fixed(0)).keep.some((x) => x.id === M.SENTINEL_ID), false, "no sentinel when all fit");
  const tzv7 = M.Zone.vietnam();
  const n0 = tzv7.localToUtc(M.localMinutes(M.ymd(2026, 5, 28), 3 * 60 + 26));
  const kf = M.reschedule(n0, [p(0, n0 + 700 * 1440)], [], tzv7).keep;
  eq([kf.length, M.fmtLocal(tzv7.toLocal(kf[0].at))], [1, "2027-07-01T09:00"], "MA3-14-06 sentinel at 09:00 device time");
  // R3 modes.
  const r3d = M.parseDataset(readFileSync(join(REPO, "memorial/testdata/r3-dataset.json"), "utf8"));
  const tzv = M.Zone.vietnam(), now0 = M.dump.r3Now(tzv);
  let nOff = 0;
  for (const cls of M.R3_CLASSES) {
    const occs = M.dump.classOccurrences(cal, r3d, cls, M.ymd(2026, 1, 1), M.ymd(2036, 12, 31));
    for (const c of M.dump.r3Configs().filter((c) => c.class === cls && c.index % 101 === 0)) {
      nOff += M.dump.r3Reminders(r3d, occs, c, now0, tzv, "global_off").length + M.dump.r3Reminders(r3d, occs, c, now0, tzv, "rule_off").length;
    }
  }
  eq(nOff, 0, "R3 global switch off / rule disabled → 0");
}

// R13 (02 §7.6): biên kỳ, (c) nhất quán, (d) ngày đầu/cuối, quy tắc lọc sự kiện.
{
  const calJ = M.Cal.get("jpl");
  const r3Base = M.parseDataset(readFileSync(join(REPO, "memorial/testdata/r3-dataset.json"), "utf8"));
  const r3 = r3Base;
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
  const linked = M.parseDataset(readFileSync(join(REPO, "memorial/testdata/r13-linked.json"), "utf8"));
  for (const [dsName, r3] of [["R3", r3Base], ["r13-linked", linked]]) {
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
  eq([okC, okDup, okF, okD, n > 0, hits > 0], [true, true, true, true, true, true], `R13(c)(d) on ${dsName} dataset 2026–2036`);
  }
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
  r3b.events[3].person_ids = [alive];
  const allY = M.eventsInPeriod(calJ, r3b, "nam", M.ymd(2027, 6, 1), "tat_ca").items.map((i) => i.entity_id);
  const dsY = M.eventsInPeriod(calJ, r3b, "nam", M.ymd(2027, 6, 1), "dang_song").items.map((i) => i.entity_id);
  const dmC = M.eventsInPeriod(calJ, r3b, "nam", M.ymd(2027, 6, 1), "da_mat").items.map((i) => i.class);
  // Bổ sung B4: sự kiện có người đã mất (một mình, hỗn hợp, hai người) → không có; chỉ người sống / không ai → đang sống.
  eq([0, 1, 2].map((i) => allY.includes(r3b.events[i].id)), [false, false, false], "B4 event with a deceased person gives nothing");
  eq([3, 4].map((i) => dsY.includes(r3b.events[i].id)), [true, true], "B4 living-only / unlinked event in dang_song");
  eq(dmC.every((c) => c === "gio" || c === "tien_thuong"), true, "B4 da_mat = gio + tien_thuong");
  // B4: người đã mất có ngày sinh + sinh nhật âm + quy tắc sinh nhật → chỉ giỗ (và tiên thường).
  const r3c = structuredClone(r3);
  const a = r3c.persons.find((p) => p.ho_ten === "Nguyễn Văn A");
  const rl = (id) => ({ id, enabled: true, offsets_days: [0, 3], time_local: 420, tien_thuong: true });
  a.sinh_nhat_am = true; a.rule_sinh_nhat = rl("c9000000-0000-4000-8000-000000000001");
  a.rule_sinh_nhat_am = rl("c9000000-0000-4000-8000-000000000002"); a.rule_gio = rl("c9000000-0000-4000-8000-000000000003");
  r3c.events[0].person_ids = [a.id];
  const occA = M.occurrencesWithTt(calJ, r3c, M.ymd(2026, 1, 1), M.ymd(2036, 12, 31));
  eq([...new Set(occA.filter((o) => o.entity_id === a.id).map((o) => o.class))].sort(), ["gio", "tien_thuong"], "B4 deceased with birthday → only giỗ");
  eq(occA.some((o) => o.entity_id === r3c.events[0].id), false, "B4 deceased person's event → nothing");
  // B4: người đang sống được cập nhật ngày mất → reschedule kế tiếp gỡ lời nhắc sinh nhật/sự kiện.
  const r3d = structuredClone(r3);
  const death = r3d.persons.find((p) => p.ho_ten === "Nguyễn Văn A").death;
  const c = r3d.persons.find((p) => p.ho_ten === "Lê Thị C");
  r3d.persons = [c]; r3d.events = [r3d.events[0]]; r3d.events[0].person_ids = [c.id];
  const tzB = M.Zone.vietnam();
  const nowB = tzB.localToUtc(M.localMinutes(M.ymd(2026, 3, 1), 540));
  const desired = (d) => M.rescheduleInput(calJ, d, nowB, tzB).map((r) => M.toPending(calJ, r, false));
  const ids = [c.rule_sinh_nhat.id, c.rule_sinh_nhat_am.id, r3d.events[0].reminder.id];
  const q1 = M.applyPlan([], M.reschedule(nowB, desired(r3d), [], tzB));
  eq(ids.map((r) => q1.some((p) => p.id.includes(r))), [true, true, true], "B4 before: birthday/event reminders scheduled");
  c.kind = "deceased"; c.death = death; c.rule_gio = rl("c9000000-0000-4000-8000-0000000000aa");
  const plan2 = M.reschedule(nowB, desired(r3d), q1, tzB);
  const q2 = M.applyPlan(q1, plan2);
  eq([ids.map((r) => q2.some((p) => p.id.includes(r))), q2.some((p) => p.id.includes("0000000000aa"))], [[false, false, false], true], "B4 after death update: removed at next reschedule");
}

// Quy tắc mật khẩu sao lưu (03 §2.4) — cùng ca với ffi/src/tests_v02.rs password_rules_leap_month_header_check.
{
  const nfd = "Ma\u0323\u0302t kha\u0302\u0309u";
  const cases = [
    ["1234567", null, "Mật khẩu cần ít nhất 8 ký tự"],
    ["12345678", null, null],
    ["😀😀😀😀", null, "Mật khẩu cần ít nhất 8 ký tự"],
    [nfd, "Mật khẩu", null],
    ["mật khẩu đúng", "mật khẩu sai", "Hai lần nhập mật khẩu chưa giống nhau"],
    ["mật khẩu đúng", "mật khẩu đúng", null],
  ];
  for (const [pw, c, want] of cases) eq(M.backupPasswordError(pw, c), want, `backupPasswordError ${pw}`);
  eq(M.backupPasswordError("12345678"), null, "backupPasswordError no confirm arg");
  const calJ2 = M.Cal.get("jpl");
  eq([calJ2.leapMonth(2025), calJ2.leapMonth(2026), calJ2.leapMonth(2028)], [6, null, 5], "leapMonth 2025/2026/2028");
}

console.log(`memorial_unit: ${pass} passed, ${fail} failed`);
process.exit(fail ? 1 : 0);
