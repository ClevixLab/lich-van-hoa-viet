// R2 phía TypeScript: cùng tập ngày với store/examples/r2_full.rs, `saveDeath`/`buildPerson` của
// pwa/src/store/derive.ts → băm JSON chuẩn của `death` (một dòng mỗi ngày). Hai băm (a), (b) phải bằng
// a_sha256, b_sha256 do Rust in ra (Rust ≡ TypeScript). Đối số: đường dẫn tệp JSON kết quả của r2_full.
// Với --store: mọi bản ghi còn đi qua lớp lưu trữ IndexedDB (src/store/db.ts, applyImport theo lô 10 000 —
// kiểm tra V1–V8 rồi một giao dịch) → đóng → mở lại → đọc lại, so từng bản ghi. IndexedDB ở đây là
// fake-indexeddb (Node), KHÔNG phải trình duyệt thật — xem docs/v0.2/03 Bổ sung B1.
import "fake-indexeddb/auto";
import { IDBFactory } from "fake-indexeddb";
import { build } from "esbuild";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const REPO = resolve(ROOT, "..");
const out = join(ROOT, "test/store/out/derive.bundle.mjs");
await build({
  stdin: { contents: `export * from "./src/store/derive.ts"; export { canonicalJson } from "./src/store/json.ts"; export * as core from "#engine-impl"; export * as db from "./src/store/db.ts";`, resolveDir: ROOT, loader: "ts" },
  bundle: true, format: "esm", platform: "node", outfile: out, logLevel: "warning",
  plugins: [{ name: "e", setup(b) { b.onResolve({ filter: /^#engine-impl$/ }, () => ({ path: join(REPO, "tsport/src/index.ts") })); } }],
});
const D = await import(pathToFileURL(out).href);
const TS = "2026-11-06T03:15:00Z";
const person = (input) => D.buildPerson({ birth: null, death: { input, tuoi: { manual: false } }, memorial: { huong_duong_duoi: 60 } });
const STORE = process.argv.includes("--store");
let batch = [], seq = 0, storeErrors = 0, storeChecked = 0;
const uid = (n) => { const h = n.toString(16).padStart(32, "0"); return `${h.slice(0, 8)}-${h.slice(8, 12)}-4${h.slice(13, 16)}-8${h.slice(17, 20)}-${h.slice(20)}`; };
async function flush() {
  if (!batch.length) return;
  const f = new IDBFactory();
  let s = await D.db.openStore(f);
  const doc = await s.snapshot({ version: "0.2.0", platform: "pwa" }, TS);
  doc.persons = batch;
  await s.applyImport(await D.db.verifyContents(D.canonicalJson(doc), []), "replace");
  s.close();
  s = await D.db.openStore(f);
  const got = new Map((await s.persons()).map((p) => [p.id, p]));
  for (const p of batch) { storeChecked++; if (D.canonicalJson(got.get(p.id) ?? null) !== D.canonicalJson(p)) storeErrors++; }
  s.close();
  batch = [];
}
async function stored(p) {
  if (!STORE) return;
  batch.push({ created_at: TS, danh_xung: "", feb29_rule: "feb28", ghi_chu: "", ho_ten: "R2", id: uid(seq++), kind: "deceased", noi_an_tang: "", photo_id: null, quan_he: "",
    reminders: { gio: null, sinh_nhat: null, sinh_nhat_am: null }, rev: 1, sinh_nhat_am: false, ten_hieu: "", updated_at: TS, birth: null, death: p.death,
    memorial: { bia_tieu_de: "", gio_het_nam: 2, huong_duong_duoi: 60, incense: true, loi_tuong_niem: "", tray_style: "truyen_thong" } });
  if (batch.length >= 10_000) await flush();
}
const H = () => createHash("sha256");
const ha = H(), hb = H();
let na = 0, nb = 0;
for (let y = 1900; y <= 3000; y++) for (let m = 1; m <= 12; m++) for (let d = 1; d <= D.daysInMonth(y, m); d++) {
  const p = person({ calendar: "solar", day: d, leap: false, month: m, precision: "day", year: y });
  ha.update(D.canonicalJson(p.death) + "\n"); na++; await stored(p);
}
for (let y = 1900; y <= 2999; y++) for (let m = 1; m <= 12; m++) for (const leap of [false, true]) for (let d = 1; d <= 30; d++) {
  if (D.core.lunarToSolar(y, m, d, leap) === null) continue;
  const p = person({ calendar: "lunar", day: d, leap, month: m, precision: "day", year: y });
  hb.update(D.canonicalJson(p.death) + "\n"); nb++; await stored(p);
}
await flush();
const res = { store_checked: storeChecked, store_errors: storeErrors, a_solar_days: na, a_sha256: ha.digest("hex"), b_lunar_days: nb, b_sha256: hb.digest("hex") };
console.log(JSON.stringify(res));
const f = process.argv.slice(2).find((a) => !a.startsWith("--"));
if (f) {
  const r = JSON.parse(readFileSync(f, "utf8"));
  const same = storeErrors === 0 && r.a_sha256 === res.a_sha256 && r.b_sha256 === res.b_sha256 && r.a_solar_days === na && r.b_lunar_days === nb;
  console.log(same ? "ĐẠT: Rust ≡ TypeScript" : "KHÔNG ĐẠT: Rust ≠ TypeScript");
  process.exit(same ? 0 : 1);
}
