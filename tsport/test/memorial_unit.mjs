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

console.log(`memorial_unit: ${pass} passed, ${fail} failed`);
process.exit(fail ? 1 : 0);
