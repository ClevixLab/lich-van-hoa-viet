// TypeScript twin of `lvhv-memorial-dump` (memorial/src/bin/lvhv-memorial-dump.rs): same commands, same
// bytes. scripts/check-parity.sh runs both and compares with cmp.
// Usage: node tsport/test/memorial_dump.mjs [--method hnd|precise] [--gio-het 2|3] COMMAND ...
import { readFileSync, writeSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const M = await import(join(HERE, "..", "dist", "memorial.mjs"));
const { Cal, dump: D, parseYmd, ymdJdn, ymdFromJdn, ymd, dvSolar, dvLunar, parseDataset, parseDateValue, Zone, generateIcs, writePretty, DEFAULT_GIO_HET_NAM, R3_CLASSES } = M;

let args = process.argv.slice(2);
let method = "precise";
let gioHet = DEFAULT_GIO_HET_NAM;
for (;;) {
  if (args[0] === "--method" && args.length > 1) { method = args[1]; args = args.slice(2); }
  else if (args[0] === "--gio-het" && args.length > 1) { gioHet = Number(args[1]); args = args.slice(2); }
  else break;
}
if (method !== "precise" && method !== "hnd") die(`phương pháp không hợp lệ: ${method}`);
if (gioHet !== 2 && gioHet !== 3) die("--gio-het phải là 2 hoặc 3");
const cal = Cal.get(method);

const buf = [];
let size = 0;
function flushIf() { if (size > 1 << 20) flush(); }
function flush() {
  const s = buf.join("");
  buf.length = 0; size = 0;
  const b = Buffer.from(s, "utf8");
  let off = 0;
  while (off < b.length) {
    try { off += writeSync(1, b, off); } catch (e) { if (e.code !== "EAGAIN") throw e; }
  }
}
const out = { push: (...xs) => { for (const x of xs) { buf.push(x); size += x.length; } } };
function die(msg) { process.stderr.write(msg + "\n"); process.exit(2); }
function pYmd(s) { const d = parseYmd(s); if (d === null) die(`ngày không hợp lệ: ${s}`); return d; }
function pInt(s) { const n = Number(s); if (!Number.isInteger(n)) die(`số không hợp lệ: ${s}`); return n; }
const load = (f) => parseDataset(readFileSync(f, "utf8"));

const [c0, c1, c2, c3, c4] = args;
if (c0 === "gio-dates" && c1 === "solar" && args.length >= 4) {
  const n = c4 === undefined ? 10 : pInt(c4);
  for (let j = ymdJdn(pYmd(c2)); j <= ymdJdn(pYmd(c3)); j++) {
    const d = ymdFromJdn(j);
    D.gioLinesFor(cal, out, dvSolar(d.y, d.m, d.d), n, [0, -1], gioHet);
    flushIf();
  }
} else if (c0 === "gio-dates" && c1 === "lunar" && args.length >= 4) {
  const n = c4 === undefined ? 10 : pInt(c4);
  for (let ly = pInt(c2); ly <= pInt(c3); ly++) {
    for (const [m, d, leap] of D.lunarDaysOfYear(cal, ly)) D.gioLinesFor(cal, out, dvLunar(ly, m, d, leap), n, [0, -1], gioHet);
    flushIf();
  }
} else if (c0 === "gio-dates" && c1 === "unknown" && args.length === 4) {
  const r = [pInt(c2), pInt(c3)];
  for (let m = 1; m <= 12; m++) for (let d = 1; d <= 30; d++) for (const leap of [false, true]) {
    D.gioLinesFor(cal, out, dvLunar(null, m, d, leap), 0, r, gioHet);
    flushIf();
  }
} else if (c0 === "gio-dates" && c1 === "list" && args.length >= 3) {
  const n = c3 === undefined ? 10 : pInt(c3);
  for (const line of readFileSync(c2, "utf8").split("\n")) {
    if (line.trim() === "") continue;
    D.gioLinesFor(cal, out, parseDateValue(JSON.parse(line)), n, [1900, 3000], gioHet);
    flushIf();
  }
} else if (c0 === "reminders" && c1 === "configs" && args.length === 2) {
  out.push(D.R3_CONFIGS_HEADER);
  for (const c of D.r3Configs()) D.r3ConfigLine(out, c);
} else if (c0 === "reminders" && (args.length === 2 || args.length === 4)) {
  const ds = load(c1);
  const f = c2 === undefined ? ymd(2026, 1, 1) : pYmd(c2);
  const t = c3 === undefined ? ymd(2036, 12, 31) : pYmd(c3);
  const tz = Zone.vietnam();
  const now = D.r3Now(tz);
  const byClass = new Map(R3_CLASSES.map((c) => [c, D.classOccurrences(cal, ds, c, f, t)]));
  out.push(D.R3_REMINDERS_HEADER);
  for (const c of D.r3Configs()) {
    for (const r of D.r3Reminders(ds, byClass.get(c.class), c, now, tz)) D.reminderLine(out, c.index, r);
    flushIf();
  }
} else if (c0 === "upcoming" && args.length === 4) {
  const ds = load(c1);
  out.push(D.R11_HEADER);
  for (let j = ymdJdn(pYmd(c2)); j <= ymdJdn(pYmd(c3)); j++) { D.upcomingLines(cal, ds, ymdFromJdn(j), out); flushIf(); }
} else if (c0 === "r10" && c1 === "a") {
  out.push(D.R10A_HEADER);
  for (let j = ymdJdn(ymd(1900, 1, 1)); j <= ymdJdn(ymd(2026, 12, 31)); j++) { D.r10aLine(cal, out, ymdFromJdn(j)); flushIf(); }
} else if (c0 === "r10" && c1 === "b") {
  out.push(D.R10B_HEADER);
  for (let y = 1899; y <= 3000; y++) D.r10bLine(out, y);
} else if (c0 === "r10" && c1 === "c") {
  out.push(D.R10C_HEADER);
  D.r10cLines(cal, out, 10000, 20261106);
} else if (c0 === "r10" && c1 === "d") {
  out.push(D.R10D_HEADER);
  for (let y = 1900; y <= 3000; y++) for (const r of ["feb28", "mar1"]) D.r10dLine(out, y, r);
} else if (c0 === "r12" && (args.length === 2 || (args.length === 3 && c2 === "--steps"))) {
  const ds = load(c1);
  const steps = c2 === "--steps" ? [] : null;
  out.push(D.R12_HEADER);
  D.r12Lines(cal, ds, [10, 50, 200], out, steps);
  if (steps !== null) { out.push(D.R12_STEPS_HEADER); out.push(steps.join("")); }
} else if (c0 === "ics" && args.length === 4) {
  out.push(generateIcs(cal, load(c1), { today: pYmd(c2), dtstamp: c3, only_ids: null }));
} else if (c0 === "derive" && args.length === 2) {
  out.push(writePretty(D.deriveDocument(cal, JSON.parse(readFileSync(c1, "utf8")))) + "\n");
} else {
  die("Cách dùng: xem lvhv-memorial-dump");
}
flush();
