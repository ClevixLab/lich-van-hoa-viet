// Bad-input tests for tsport (RS-01/02/03/06/13, C-2) — mirror core/tests/bad_input.rs.
// Every call runs in a worker with a hard timeout so an infinite loop FAILS instead of hanging.
// Usage: node tsport/test/bad_input.mjs   (after `npm run build` in tsport/)
import { Worker } from "node:worker_threads";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const LIB = pathToFileURL(join(HERE, "..", "dist", "lvhv.mjs")).href;
const lib = await import(LIB);
const { LvhvError } = lib;

let fail = 0, pass = 0;
const ok = (c, msg) => { if (c) pass++; else { fail++; console.log("FAIL:", msg); } };

/** Run `expr` (a string of JS using `L` = library) in a worker; resolve {ok,value} | {err:{name,code,message}} | "TIMEOUT". */
function run(expr, ms = 5000) {
  const src = `
    const { parentPort } = require("node:worker_threads");
    import(${JSON.stringify(LIB)}).then((L) => {
      try { const v = (${expr}); parentPort.postMessage({ ok: true, value: JSON.stringify(v) ?? "undefined" }); }
      catch (e) { parentPort.postMessage({ err: { name: e?.name, code: e?.code, message: e?.message, isRange: e instanceof RangeError, isLvhv: e instanceof L.LvhvError } }); }
    });`;
  return new Promise((res) => {
    const w = new Worker(src, { eval: true });
    const t = setTimeout(() => { w.terminate(); res("TIMEOUT"); }, ms);
    w.once("message", (m) => { clearTimeout(t); w.terminate(); res(m); });
    w.once("error", (e) => { clearTimeout(t); res({ err: { name: "WorkerError", message: String(e) } }); });
  });
}

async function expectErr(expr, code) {
  const r = await run(expr);
  ok(r !== "TIMEOUT", `${expr}: TIMEOUT`);
  if (r === "TIMEOUT") return;
  ok(!!r.err, `${expr}: expected throw ${code}, got ${r.value}`);
  if (r.err) {
    ok(r.err.isLvhv && r.err.isRange, `${expr}: not an LvhvError/RangeError (${r.err.name}: ${r.err.message})`);
    ok(r.err.code === code, `${expr}: code ${r.err.code} ≠ ${code} (${r.err.message})`);
  }
}
async function expectVal(expr, want) {
  const r = await run(expr);
  ok(r !== "TIMEOUT" && r.ok && r.value === JSON.stringify(want), `${expr}: got ${JSON.stringify(r)} want ${JSON.stringify(want)}`);
}

const BAD_YEARS = ["-2147483648", "-1e6", "-14621", "-1", "0", "1", "1899", "3001", "35650", "100000", "1e6", "1e9", "2147483647", "2**53"];
for (const y of BAD_YEARS) {
  for (const m of ['"hnd"', '"precise"', '"jpl"']) {
    await expectErr(`L.solarToLunarWith(${m}, ${y}, 1, 1)`, "OutOfRange");
    await expectErr(`L.dayInfoJson(${y}, 12, 31, ${m})`, "OutOfRange");
    await expectErr(`L.tetWith(${m}, ${y})`, "OutOfRange");
    await expectVal(`L.lunarToSolarWith(${m}, ${y}, 1, 1, false)`, null);
  }
  await expectErr(`L.solarTermsOfYear(${y})`, "OutOfRange");
}
// NaN / Infinity / non-integers
for (const v of ["NaN", "Infinity", "-Infinity", "2026.5", '"2026"', "null", "undefined"]) {
  await expectErr(`L.solarToLunar(${v}, 1, 1)`, "InvalidDate");
  await expectErr(`L.dayInfo(2026, ${v}, 1)`, "InvalidDate");
  await expectErr(`L.dayInfo(2026, 1, ${v})`, "InvalidDate");
  await expectErr(`L.jdnToLunar(${v})`, "InvalidArgument");
  await expectVal(`L.lunarToSolar(${v}, 1, 1, false)`, null);
  await expectVal(`L.lunarToSolar(2026, ${v}, 1, false)`, null);
}
for (const v of ["NaN", "2026.5", '"2026"']) await expectErr(`L.tet(${v})`, "InvalidArgument");
await expectErr(`L.tet(Infinity)`, "OutOfRange");
// JDN extremes (RS-01)
await expectErr(`L.jdnToLunar(2**53)`, "InvalidArgument"); // không phải safe integer
for (const j of ["0", "-1", "1e9", "-1e9", "2**53 - 1", "2415020", "2817153"]) await expectErr(`L.jdnToLunar(${j})`, "OutOfRange");
await expectErr(`L.jdnToLunar(1.5e6 + 0.5)`, "InvalidArgument"); // RS-06: trước đây d = 15.5
await expectErr(`L.dayCtxJdn(2461350.5)`, "InvalidArgument");
// Invalid solar dates
for (const [y, m, d] of [[2026, 2, 29], [2100, 2, 29], [2026, 2, 30], [2026, 4, 31], [2026, 13, 1], [2026, 0, 1], [2026, 1, 0], [2026, 1, 32], [2147483647, 13, 1]]) {
  await expectErr(`L.solarToLunar(${y}, ${m}, ${d})`, "InvalidDate");
  await expectErr(`L.dayInfoJson(${y}, ${m}, ${d})`, "InvalidDate");
  await expectErr(`L.findGoodDays("cuoi_hoi", {y:${y},m:${m},d:${d}}, {y:2026,m:3,d:1})`, "InvalidDate");
}
await expectVal(`L.solarToLunar(2024, 2, 29).d > 0`, true);
// Impossible lunar dates
await expectVal(`L.lunarToSolar(2026, 5, 1, true)`, null);
await expectVal(`L.lunarToSolar(2025, 5, 1, true)`, null);
await expectVal(`L.lunarToSolar(2025, 6, 1, true) !== null`, true);
await expectVal(`(() => { const m = [1,2,3,4,5,6,7,8,9,10,11,12].find((m) => L.lunarToSolar(2026, m, 30, false) === null); return [L.lunarToSolar(2026, m, 29, false) !== null, L.lunarToSolar(2026, m, 30, false)]; })()`, [true, null]);
await expectVal(`L.lunarToSolar(2025, 6, 1, 1)`, null); // RS-06: leap phải là boolean
await expectVal(`L.lunarToSolar(2026, 1, 1, "yes")`, null);
await expectVal(`L.lunarToSolar(1899, 11, 1, false)`, null); // kết quả trước 1900-01-01
// Edges
await expectVal(`[L.solarToLunar(2000,1,1).y, L.solarToLunar(2000,1,1).m, L.solarToLunar(2000,1,1).d]`, [1999, 11, 25]);
await expectVal(`[L.solarToLunar(1900,1,1).y, L.solarToLunar(1900,1,1).m, L.solarToLunar(1900,1,1).d]`, [1899, 12, 1]);
await expectVal(`[L.tet(1968).y, L.tet(1968).m, L.tet(1968).d]`, [1968, 1, 29]);
await expectVal(`[L.tet(1916).y, L.tet(1916).m, L.tet(1916).d]`, [1916, 2, 3]);
await expectVal(`L.dayInfo(3000, 12, 31).solar.jdn`, 2817152);
// findGoodDays range rules (RS-13)
await expectErr(`L.findGoodDays("cuoi_hoi", {y:2026,m:3,d:1}, {y:2026,m:2,d:1})`, "InvertedRange");
await expectErr(`L.findGoodDays("cuoi_hoi", {y:1900,m:1,d:1}, {y:3000,m:12,d:31})`, "RangeTooLong");
await expectVal(`L.findGoodDays("cuoi_hoi", {y:2026,m:1,d:1}, L.jd.dateFromJdn(L.jd.jdnFromDate(2026,1,1)+366).reduce((o,v,i)=>(o["ymd"[i]]=v,o),{})).length > 0`, true);
await expectErr(`L.findGoodDays("cuoi_hoi", {y:2026,m:1,d:1}, L.jd.dateFromJdn(L.jd.jdnFromDate(2026,1,1)+367).reduce((o,v,i)=>(o["ymd"[i]]=v,o),{}))`, "RangeTooLong");
await expectErr(`L.findGoodDays("cuoi_hoi", {y:1899,m:12,d:31}, {y:1900,m:1,d:5})`, "OutOfRange");
await expectErr(`L.findGoodDays("khong_co", {y:2026,m:1,d:1}, {y:2026,m:1,d:31})`, "UnknownActivity");
await expectErr(`L.findGoodDays("cuoi_hoi", {y:2026,m:1,d:1}, {y:2026,m:1,d:31}, 1976.5)`, "InvalidArgument");
await expectErr(`L.findGoodDaysWith("xyz", "cuoi_hoi", {y:2026,m:1,d:1}, {y:2026,m:1,d:31})`, "InvalidArgument");
// C-2 birth lunar year
await expectVal(`[L.lunarYearOfBirth(1976,1,15), L.lunarYearOfBirth(1976,1,30), L.lunarYearOfBirth(1976,1,31), L.lunarYearOfBirth(1900,1,1)]`, [1975, 1975, 1976, 1899]);
await expectErr(`L.lunarYearOfBirth(1899, 12, 31)`, "OutOfRange");
await expectErr(`L.lunarYearOfBirth(1990, 2, 30)`, "InvalidDate");
await expectErr(`L.lunarYearOfBirth(NaN, 1, 1)`, "InvalidDate");
// Messages identical to Rust Display
await expectVal(`(() => { try { L.solarToLunar(2026,2,30) } catch (e) { return e.message } })()`, "ngày dương không hợp lệ: 2026-2-30");
await expectVal(`(() => { try { L.dayInfo(1899,1,1) } catch (e) { return e.message } })()`, "ngoài phạm vi hỗ trợ (năm 1900 đến năm 3000)");
ok(typeof LvhvError === "function", "LvhvError exported");

console.log(`tsport bad_input: ${pass} passed, ${fail} failed`);
process.exitCode = fail ? 1 : 0;
