// L7 verification: tsport vs Rust core dumps.
//  1. dayInfoJson, every day 1900-01-01..3000-12-31 (L7 + L12), for BOTH methods:
//       hnd     ← core/out/dayinfo_1900_3000.jsonl.gz
//       precise ← core/out/precise/dayinfo_1900_3000.jsonl.gz
//       jpl     ← core/out/jpl/dayinfo_1900_3000.jsonl.gz (docs/03 §17)
//  0. tsport/src/jpl_data.ts holds exactly the bytes of core/data/jpl_events.bin (SHA-256)
//  2. tet() 1900..3000 vs tet_1900_1999.csv + tet.csv (2000..3000), each method dir
//  3. findGoodDays JSONL vs core/out/gooddays_sample.jsonl (+ precise/ if present):
//       every activity, 2026-01-01..2026-12-31, birth year none and 1976 (lvhv-dump gooddays all …)
//  4. dayInfo micro-benchmark.
// Usage: node tsport/test/compare.mjs [--method hnd|precise|jpl] [--skip-dayinfo]
import { createReadStream, existsSync, readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { createGunzip } from "node:zlib";
import { createInterface } from "node:readline";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO = join(HERE, "..", "..");
const lib = await import(join(HERE, "..", "dist", "lvhv.mjs"));
const { dayInfo, dayInfoJson, tetWith, jd, activities, findGoodDaysWith, goodDayJson, DEFAULT_METHOD } = lib;

const argv = process.argv.slice(2);
const only = argv.includes("--method") ? [argv[argv.indexOf("--method") + 1]] : ["hnd", "precise", "jpl"];
const skipDayinfo = argv.includes("--skip-dayinfo");
// LVHV_GOLDEN_DIR=<dir>: read freshly generated dumps (<dir>/… and <dir>/precise/…) instead of the
// committed core/out goldens (scripts/check-parity.sh uses this; RS-12).
const GOLDEN = process.env.LVHV_GOLDEN_DIR || join(REPO, "core", "out");
const OUT = { hnd: GOLDEN, precise: join(GOLDEN, "precise"), jpl: join(GOLDEN, "jpl") };

function firstDiff(a, b) {
  let i = 0;
  while (i < a.length && i < b.length && a[i] === b[i]) i++;
  const s = Math.max(0, i - 40);
  return `  at char ${i}\n  rust: …${a.slice(s, i + 80)}\n  ts  : …${b.slice(s, i + 80)}`;
}

const summary = {};
let failed = false;

// ---- 0. Jpl table bytes: TS base64 ≡ core/data/jpl_events.bin ----
{
  const bin = readFileSync(join(REPO, "core", "data", "jpl_events.bin"));
  const tsSrc = readFileSync(join(REPO, "tsport", "src", "jpl_data.ts"), "utf8");
  const b64 = /JPL_EVENTS_B64 =\s*"([^"]*)"/.exec(tsSrc)[1];
  const same = Buffer.from(b64, "base64").equals(bin);
  const sha = createHash("sha256").update(bin).digest("hex");
  console.log(`jpl_data.ts ≡ jpl_events.bin: ${same ? "giống hệt" : "KHÁC"} (sha256 ${sha}, ${bin.length} byte)`);
  if (!same || !tsSrc.includes(sha)) failed = true;
}
console.log(`DEFAULT_METHOD (tsport) = ${DEFAULT_METHOD}; goldens: ${GOLDEN}`);

for (const method of only) {
  const res = (summary[method] = {});
  // ---- 1. day_info JSONL ----
  const gz = join(OUT[method], "dayinfo_1900_3000.jsonl.gz");
  if (!skipDayinfo) {
    if (!existsSync(gz)) { console.log(`[${method}] MISSING ${gz}`); failed = true; }
    else {
      const rl = createInterface({ input: createReadStream(gz).pipe(createGunzip()), crlfDelay: Infinity });
      const firstJdn = jd.jdnFromDate(1900, 1, 1);
      const lastJdn = jd.jdnFromDate(3000, 12, 31);
      let j = firstJdn, n = 0, bad = 0;
      const shown = [];
      const t0 = process.hrtime.bigint();
      for await (const line of rl) {
        if (line === "") continue;
        const [y, m, d] = jd.dateFromJdn(j);
        const mine = dayInfoJson(y, m, d, method);
        if (mine !== line) {
          bad++;
          if (shown.length < 10) shown.push(`#${bad} ${jd.fmtDate(y, m, d)}\n${firstDiff(line, mine)}`);
        }
        n++;
        j++;
      }
      const secs = Number(process.hrtime.bigint() - t0) / 1e9;
      const expected = lastJdn - firstJdn + 1;
      console.log(`[${method}] dayinfo: lines=${n} expected=${expected} mismatches=${bad} (${secs.toFixed(1)} s)`);
      for (const s of shown) console.log(s);
      res.dayinfo = { lines: n, expected, mismatches: bad };
      if (bad || n !== expected) failed = true;
    }
  }

  // ---- 2. tet ----
  const tetCsvs = [join(OUT[method], "tet_1900_1999.csv"), join(OUT[method], "tet.csv")];
  const missing = tetCsvs.filter((f) => !existsSync(f));
  if (missing.length) { console.log(`[${method}] tet: MISSING ${missing.join(", ")}`); failed = true; }
  else {
    const rows = tetCsvs.flatMap((f) => readFileSync(f, "utf8").trim().split("\n").slice(1));
    let tbad = 0;
    for (const r of rows) {
      const [ly, date] = r.trim().split(",");
      const t = tetWith(method, Number(ly));
      const mine = jd.fmtDate(t.y, t.m, t.d);
      if (mine !== date) { tbad++; if (tbad <= 10) console.log(`[${method}] tet ${ly}: rust ${date} ts ${mine}`); }
    }
    console.log(`[${method}] tet: rows=${rows.length} mismatches=${tbad}`);
    res.tet = { rows: rows.length, mismatches: tbad };
    if (tbad) failed = true;
  }

  // ---- 3. gooddays sample ----
  const gd = join(OUT[method], "gooddays_sample.jsonl");
  if (!existsSync(gd)) { console.log(`[${method}] gooddays: MISSING sample file (${gd})`); failed = true; }
  else {
    const want = readFileSync(gd, "utf8").split("\n").filter(Boolean);
    const mine = [];
    const from = { y: 2026, m: 1, d: 1 }, to = { y: 2026, m: 12, d: 31 };
    for (const birth of [null, 1976]) {
      for (const a of activities()) {
        for (const g of findGoodDaysWith(method, a.id, from, to, birth)) {
          mine.push(`{"activity":"${a.id}","birth_year":${birth === null ? "null" : birth},${goodDayJson(g).slice(1)}`);
        }
      }
    }
    let bad = 0;
    const n = Math.max(want.length, mine.length);
    for (let i = 0; i < n; i++) {
      if (want[i] !== mine[i]) {
        bad++;
        if (bad <= 10) console.log(`[${method}] gooddays line ${i + 1}\n${firstDiff(want[i] ?? "", mine[i] ?? "")}`);
      }
    }
    console.log(`[${method}] gooddays: rust=${want.length} ts=${mine.length} mismatches=${bad}`);
    res.gooddays = { rust_lines: want.length, ts_lines: mine.length, mismatches: bad };
    if (bad) failed = true;
  }
}

// ---- 4. bench: dayInfo (+ JSON) for every day of 2026, default method ----
const days = [];
for (let j = jd.jdnFromDate(2026, 1, 1); j < jd.jdnFromDate(2027, 1, 1); j++) days.push(jd.dateFromJdn(j));
let sink = 0;
for (let w = 0; w < 3; w++) for (const [y, m, d] of days) sink += dayInfo(y, m, d).lunar.d; // warm-up
const R = 20;
const tb = process.hrtime.bigint();
for (let r = 0; r < R; r++) for (const [y, m, d] of days) sink += dayInfo(y, m, d).lunar.d;
const usInfo = Number(process.hrtime.bigint() - tb) / 1e3 / (R * days.length);
const tj = process.hrtime.bigint();
for (let r = 0; r < R; r++) for (const [y, m, d] of days) sink += dayInfoJson(y, m, d).length;
const usJson = Number(process.hrtime.bigint() - tj) / 1e3 / (R * days.length);
const tg = process.hrtime.bigint();
for (const a of activities()) sink += findGoodDaysWith(DEFAULT_METHOD, a.id, { y: 2026, m: 11, d: 1 }, { y: 2027, m: 1, d: 29 }, 1976).length;
const msGd = Number(process.hrtime.bigint() - tg) / 1e6 / activities().length;
console.log(`bench (Node ${process.version}, 2026, 365 days x ${R}): dayInfo mean ${usInfo.toFixed(2)} µs; dayInfoJson mean ${usJson.toFixed(2)} µs; findGoodDays 90 days mean ${msGd.toFixed(2)} ms (sink ${sink})`);
console.log(`SUMMARY ${JSON.stringify(summary)}`);

process.exitCode = failed ? 1 : 0;
