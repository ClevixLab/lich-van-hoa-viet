// Kiểm thử lớp lưu trữ + sao lưu của PWA (Node, IndexedDB giả lập bằng fake-indexeddb, WebCrypto của Node).
// Dựng src/store/ bằng esbuild (cùng #engine-impl → tsport như bản dựng PWA), rồi chạy:
//   (c)  vector chuẩn: Argon2id RFC 9106 §5.3, XChaCha20-Poly1305 draft-irtf-cfrg-xchacha A.3
//   §2.5 tệp mẫu example-v1.lvhv do Rust sinh: mở được; tệp JS xuất với salt/nonce cố định GIỐNG TỪNG BYTE
//   §2.5 các tệp hỏng: cùng mã lỗi với Rust
//   (a)  bộ R6: nạp → stateHash = băm của SQLite (Rust) → xuất → nhập Thay toàn bộ vào kho trống → giống hệt
//   (a2) 20 mật khẩu sai bị từ chối, NFD được chấp nhận, kho không đổi
//   (b)  lật bit 65 536 biến thể (4 096 byte đầu + 4 096 byte cuối) — 100 % bị từ chối, kho không đổi
//   (d)  tệp do Rust xuất mở trong JS (example + bộ R6); tệp JS xuất ghi ra pwa/test/store/out/ để Rust mở
//   (e)  31 ca gộp: kết quả JSON chuẩn có sha256 giống `merge_docs` của Rust
//   V1–V8: 63 ca validation-cases.json cùng (mã, đường dẫn) với Rust
// Chạy: node test/store/run.mjs [--quick]   (--quick bỏ phần lật bit)
import "fake-indexeddb/auto";
import { build } from "esbuild";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const REPO = resolve(ROOT, "..");
const TV = join(REPO, "docs/v0.2/schema/test-vectors");
const OUT = join(ROOT, "test/store/out");
mkdirSync(OUT, { recursive: true });
const QUICK = process.argv.includes("--quick");

// ---------- dựng mô-đun ----------
const bundle = join(OUT, "store.bundle.mjs");
await build({
  stdin: {
    contents: `export * from "./src/store/index.ts"; export * as bc from "./src/store/backup-crypto.ts";`,
    resolveDir: ROOT, loader: "ts",
  },
  bundle: true, format: "esm", platform: "node", target: "es2022", outfile: bundle, logLevel: "warning",
  plugins: [{ name: "engine", setup(b) { b.onResolve({ filter: /^#engine-impl$/ }, () => ({ path: join(REPO, "tsport/src/index.ts") })); } }],
});
const S = await import(pathToFileURL(bundle).href);
const { bc } = S;
const { argon2id } = await import("@noble/hashes/argon2.js");
const { xchacha20poly1305 } = await import("@noble/ciphers/chacha.js");

let fails = 0;
const ok = (c, msg) => { console.log(`${c ? "  ok " : "FAIL "} ${msg}`); if (!c) fails++; };
const hex = (b) => Buffer.from(b).toString("hex");
const unhex = (s) => Uint8Array.from(Buffer.from(s.replace(/\s+/g, ""), "hex"));
const sha = (b) => createHash("sha256").update(b).digest("hex");
const rd = (p) => new Uint8Array(readFileSync(join(TV, p)));
const json = (p) => JSON.parse(readFileSync(join(TV, p), "utf8"));
const manifest = json("manifest.json");
const PW = manifest.password;
const SALT = unhex(manifest.salt_hex), NONCE = unhex(manifest.nonce_hex);
const results = {};

async function code(fn) { try { await fn(); return "OK"; } catch (e) { return e.code ?? e.cause?.code ?? e.message; } }
let dbn = 0;
async function freshStore() {
  // fake-indexeddb: mỗi kho một IDBFactory riêng (như một "máy" riêng).
  const { IDBFactory } = await import("fake-indexeddb");
  const f = new IDBFactory();
  dbn++;
  return S.openStore(f);
}

// ---------- (c) vector chuẩn ----------
{
  const r = argon2id(new Uint8Array(32).fill(1), new Uint8Array(16).fill(2), { t: 3, m: 32, p: 4, dkLen: 32, key: new Uint8Array(8).fill(3), personalization: new Uint8Array(12).fill(4) });
  const a = hex(r) === "0d640df58d78766c08c037a34a8b53c9d01ef0452d75b65eb52520e96b01e659";
  ok(a, "Argon2id RFC 9106 §5.3");
  const key = Uint8Array.from({ length: 32 }, (_, i) => 0x80 + i);
  const nonce = Uint8Array.from({ length: 24 }, (_, i) => 0x40 + i);
  const aad = unhex("50515253c0c1c2c3c4c5c6c7");
  const pt = new TextEncoder().encode("Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.");
  const want = "bd6d179d3e83d43b9576579493c0e939572a1700252bfaccbed2902c21396cbb731c7f1b0b4aa6440bf3a82f4eda7e39ae64c6708c54c216cb96b72e1213b4522f8c9ba40db5d945b11b69b982c1bb9e3f3fac2bc369488f76b2383565d3fff921f9664c97637da9768812f615c68b13b52e" + "c0875924c1c7987947deafd8780acf49";
  const ct = xchacha20poly1305(key, nonce, aad).encrypt(pt);
  const back = xchacha20poly1305(key, nonce, aad).decrypt(unhex(want));
  const bad = unhex(want); bad[bad.length - 1] ^= 1;
  const rej = await code(() => xchacha20poly1305(key, nonce, aad).decrypt(bad)) !== "OK";
  const x = hex(ct) === want && Buffer.from(back).equals(Buffer.from(pt)) && rej;
  ok(x, "XChaCha20-Poly1305 draft-irtf-cfrg-xchacha A.3 (mã hóa, giải mã, từ chối thẻ sai)");
  const k = await bc.deriveKeyBytes(new TextEncoder().encode("password"), new Uint8Array(16), 65536, 3, 4);
  const k2 = hex(k) === "00b1eed9bee6dc0641a507717db76b6520ec876ece6cd10925e43875b543575e";
  ok(k2, "Argon2id tham số ứng dụng (64 MiB, t=3, p=4) = khóa tham chiếu của Rust");
  const k3 = hex(await bc.deriveKeyBytes(bc.normalizePassword(PW), SALT, 65536, 3, 4)) === manifest.key_hex;
  ok(k3, "khóa của vector §2.5 = manifest.key_hex (Rust)");
  results.c_standard_vectors = { argon2id_rfc9106: a, xchacha_draft_a3: x, app_params_key_matches_rust: k2 && k3 };
}

// ---------- §2.5 tệp mẫu ----------
const exFile = rd("example-v1.lvhv");
const exJsonText = readFileSync(join(TV, "example-v1.json"), "utf8");
const exPhoto = rd(manifest.example.photo);
{
  const v = await S.openBackup(exFile, PW);
  ok(S.canonicalJson(v.doc) === exJsonText, "example-v1.lvhv (Rust) mở trong JS, JSON giống từng byte");
  ok(v.preview.persons === 3 && v.preview.events === 1 && v.preview.photos === 1, "xem trước: 3 người, 1 sự kiện, 1 ảnh");
  const f = await bc.exportWithFixedRng(PW, exJsonText, [{ id: bc.uuidToBytes(v.doc.photos[0].id), bytes: exPhoto }], SALT, NONCE);
  ok(sha(f) === manifest.example.sha256, "JS xuất example với salt/nonce cố định = example-v1.lvhv của Rust (giống từng byte)");
  results.example_vector = { opens: true, js_fixed_rng_equals_rust: sha(f) === manifest.example.sha256 };

  const key = unhex(manifest.key_hex);
  const hdr = bc.parseHeader(exFile);
  let allOk = true;
  for (const c of manifest.corrupt) {
    const f = rd(c.file);
    let got;
    try {
      const h = bc.parseHeader(f);
      if (hex(h.salt) === hex(hdr.salt) && h.mCostKib === 65536 && h.tCost === 3 && h.pCost === 4) got = await code(() => bc.openWithKey(f, h, key));
      else got = await code(() => bc.importBackup(f, PW));
    } catch (e) { got = e.code; }
    if (got !== c.expect) { allOk = false; console.log(`     ${c.file}: ${got} ≠ ${c.expect}`); }
  }
  ok(allOk, `${manifest.corrupt.length} tệp hỏng (03 §2.5): cùng mã lỗi với Rust`);
  results.corrupt_vectors = { n: manifest.corrupt.length, all_rejected_same_code: allOk };
}

// ---------- YC-2026-05: vector bổ sung thông tin báo tử (01 §3.2a; không thay vector đã khóa) ----------
{
  const SD = join(TV, "supplement-yc05");
  const sm = JSON.parse(readFileSync(join(SD, "manifest.json"), "utf8"));
  const nFile = new Uint8Array(readFileSync(join(SD, "notice-v1.lvhv")));
  const nText = readFileSync(join(SD, "notice-v1.json"), "utf8");
  const v = await S.openBackup(nFile, sm.password);
  ok(S.canonicalJson(v.doc) === nText, "YC-2026-05: notice-v1.lvhv (Rust) mở trong JS, JSON giống từng byte");
  const blobs = (v.doc.photos).map((ph) => ({ id: bc.uuidToBytes(ph.id), bytes: exPhoto }));
  const f = await bc.exportWithFixedRng(sm.password, nText, blobs, unhex(sm.salt_hex), unhex(sm.nonce_hex));
  ok(sha(f) === sm.file.sha256, "YC-2026-05: JS xuất notice-v1 với salt/nonce cố định = tệp của Rust (giống từng byte)");
  const t = await freshStore();
  await t.applyImport(v, "replace");
  ok((await t.stateHash()) === sm.state_sha256, "YC-2026-05: nhập vào IndexedDB → stateHash = state_hash(SQLite) của Rust");
  const pA = await t.person("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01");
  ok(pA.death.notice.certificate_photo_ids[0] === sm.cert_photo_id, "YC-2026-05: báo tử và ảnh giấy báo tử được giữ");
  await t.deletePerson("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01");
  const left = (await t.snapshot({ version: "0.2.0", platform: "pwa" }, "2026-01-01T00:00:00Z")).photos.map((x) => x.id);
  ok(!left.includes(sm.cert_photo_id), "YC-2026-05: xóa hồ sơ → xóa ảnh giấy báo tử");
  const bad = structuredClone(v.doc);
  bad.photos = bad.photos.filter((x) => x.id !== sm.cert_photo_id);
  let got = "";
  try { S.validate.document(bad); } catch (e) { got = `${e.code} ${e.path}`; }
  ok(got === "V3 /persons/0/death/notice/certificate_photo_ids/0", "YC-2026-05: ảnh giấy báo tử không tồn tại → V3 (cùng đường dẫn với Rust)");
  results.notice_supplement = { opens: true, js_fixed_rng_equals_rust: sha(f) === sm.file.sha256 };
}

// ---------- V1–V8 ----------
{
  const cases = json("validation-cases.json");
  const base = JSON.parse(exJsonText);
  let bad = 0;
  for (const c of cases) {
    const text = c.raw ?? S.canonicalJson(applyPatch(base, c.pointer, c.value));
    let got = ["OK", ""];
    try { S.validate.document(S.validate.parseJson(text)); } catch (e) { got = [e.code, e.path]; }
    if (got[0] !== c.code || got[1] !== c.path) { bad++; console.log(`     ${c.name}: JS ${got.join(" ")} ≠ Rust ${c.code} ${c.path}`); }
  }
  ok(bad === 0, `${cases.length} ca kiểm tra V1–V8: JS = Rust (mã + đường dẫn)`);
  results.validation_cases = { n: cases.length, mismatches: bad };
}

function applyPatch(base, ptr, val) {
  const d = structuredClone(base);
  if (!ptr) return d;
  const parts = ptr.split("/").slice(1);
  const key = parts.pop();
  let t = d;
  for (const p of parts) t = t[Array.isArray(t) ? +p : p];
  if (val === "__delete__") delete t[key];
  else if (Array.isArray(t)) t[+key] = val;
  else t[key] = val;
  return d;
}

// ---------- R6 dữ liệu ----------
function r6PhotoBytes(i) {
  const len = 1000 + i * 997;
  const v = new Uint8Array(len);
  for (let k = 0; k < len; k++) v[k] = (k * 31 + i * 17 + (k >> 8)) & 0xff;
  v.set([0xff, 0xd8, 0xff, 0xe0], 0);
  v.set([0xff, 0xd9], len - 2);
  return v;
}
const dsText = readFileSync(join(TV, "dataset-r6.json"), "utf8");
const ds = JSON.parse(dsText);
// Ảnh: id trong dataset sắp theo id; byte theo chỉ số i của nhãn "r6-photo-{i}" → khớp bằng sha256.
const bySha = new Map(Array.from({ length: 20 }, (_, i) => r6PhotoBytes(i)).map((b) => [sha(b), b]));
const dsPhotos = ds.photos.map((p) => ({ id: p.id, bytes: bySha.get(p.sha256) }));
ok(dsPhotos.every((p) => p.bytes), "bộ R6: 20 ảnh tất định khớp sha256");

async function seeded() {
  const s = await freshStore();
  const v = await S.verifyContents(dsText, dsPhotos);
  await s.applyImport(v, "replace");
  return s;
}

// (a) khứ hồi + (d) chéo
let r6File;
{
  const s = await seeded();
  const snap = await s.snapshot({ version: "0.2.0", platform: "pwa" }, ds.exported_at);
  snap.app.platform = "windows";
  ok(S.canonicalJson(snap) === dsText, "bộ R6 nạp vào IndexedDB, ảnh chụp JSON = dataset-r6.json (giống từng byte)");
  const h1 = await s.stateHash();
  ok(h1 === manifest.dataset_r6.store_state_sha256, "stateHash(IndexedDB) = state_hash(SQLite) của Rust");
  const fixed = await bc.exportWithFixedRng(PW, dsText, dsPhotos.map((p) => ({ id: bc.uuidToBytes(p.id), bytes: p.bytes })), SALT, NONCE);
  ok(sha(fixed) === manifest.dataset_r6.fixed_rng_file_sha256, "tệp R6 salt/nonce cố định: JS = Rust (giống từng byte)");
  const t0 = performance.now();
  r6File = await s.exportBackup(PW, { version: "0.2.0", platform: "pwa" });
  const exportMs = performance.now() - t0;
  ok((await s.backupPendingSince()) === null && typeof (await s.settings()).last_backup_at === "string", "sau khi xuất: last_backup_at được đặt, hết trạng thái chờ sao lưu");
  writeFileSync(join(OUT, "r6-from-pwa.lvhv"), r6File);
  const t = await freshStore();
  const t1 = performance.now();
  const v = await S.openBackup(r6File, PW);
  const importMs = performance.now() - t1;
  await t.applyImport(v, "replace");
  const h2 = await t.stateHash();
  ok(h2 === h1, "R6(a) PWA: xuất → nhập Thay toàn bộ trên kho trống → giống hệt (stateHash)");
  // Tệp Rust → JS
  const rustFile = new Uint8Array(readFileSync(join(ROOT, "test/store/out/r6-from-rust.lvhv"), { flag: "r" }).buffer.slice(0));
  const u = await freshStore();
  await u.applyImport(await S.openBackup(rustFile, PW), "replace");
  ok((await u.stateHash()) === h1, "R6(d) tệp xuất từ Rust (SQLite) nhập vào PWA → giống hệt");
  results.r6_a = { pwa_roundtrip_identical: h2 === h1, state_hash: h1, export_ms_node: Math.round(exportMs), import_ms_node: Math.round(importMs), file_bytes: r6File.length };
  results.r6_d = { rust_to_pwa: (await u.stateHash()) === h1, pwa_file_for_rust: "pwa/test/store/out/r6-from-pwa.lvhv", fixed_rng_bytes_equal: sha(fixed) === manifest.dataset_r6.fixed_rng_file_sha256 };

  // (a2) sai mật khẩu
  const before = await s.stateHash();
  const nfd = PW.normalize("NFD");
  const wrong = ["", " ", `${PW} `, ` ${PW}`, PW.replace("m", "M"), PW.replace("ậ", "â"), PW.replace("ẩ", "ẫ"), PW.replace("ệ", "e"),
    PW.replace("thử", "thu"), PW.slice(0, -1), `${PW}m`, PW.toUpperCase(), "mat khau thu nghiem", `${PW}\n`, "mật  khẩu thử nghiệm",
    `${PW}​`, `${PW}\0`, "password", "mật khẩu thử nghiệ", `${PW}.`];
  let rejected = 0;
  for (const w of wrong) {
    try { await S.openBackup(exFile, w); } catch (e) { if (e.code === "backup" && e.message === "Sai mật khẩu hoặc tệp bị hỏng") rejected++; }
  }
  ok(wrong.length === 20 && rejected === 20, `R6(a2) 20 mật khẩu sai bị từ chối với thông báo chung (${rejected}/20)`);
  const nfdOk = nfd !== PW && (await code(() => S.openBackup(exFile, nfd))) === "OK";
  ok(nfdOk, "R6(a2) mật khẩu đúng dạng NFD được chấp nhận");
  ok((await s.stateHash()) === before, "R6(a2) kho không đổi");
  results.r6_a2 = { wrong: 20, rejected, nfd_accepted: nfdOk, store_unchanged: (await s.stateHash()) === before };

  // (b) lật bit
  if (!QUICK) {
    const file = r6File;
    const h = bc.parseHeader(file);
    const key = await bc.deriveKeyBytes(bc.normalizePassword(PW), h.salt, h.mCostKib, h.tCost, h.pCost);
    ok((await code(() => bc.openWithKey(file, h, key))) === "OK", "tệp gốc mở được bằng khóa đã dẫn xuất");
    let n = 0, accepted = 0, slow = 0;
    const t2 = performance.now();
    const positions = [...Array(4096).keys(), ...Array.from({ length: 4096 }, (_, i) => file.length - 4096 + i)];
    for (const i of positions) {
      for (let bit = 0; bit < 8; bit++) {
        const f = file.slice();
        f[i] ^= 1 << bit;
        n++;
        let hh;
        try { hh = bc.parseHeader(f); } catch { continue; }
        if (hex(hh.salt) === hex(h.salt) && hh.mCostKib === h.mCostKib && hh.tCost === h.tCost && hh.pCost === h.pCost) {
          if ((await code(() => bc.openWithKey(f, hh, key))) === "OK") accepted++;
        } else {
          // salt/m/t/p đổi → Argon2id thật với tham số mới (đúng đường nhập).
          slow++;
          if ((await code(() => bc.importBackup(f, PW))) === "OK") accepted++;
        }
      }
    }
    ok(n === 65536 && accepted === 0, `R6(b) lật bit: ${n} biến thể, ${accepted} được chấp nhận (${slow} qua Argon2id thật), ${Math.round((performance.now() - t2) / 1000)} s`);
    ok((await s.stateHash()) === before, "R6(b) kho không đổi");
    results.r6_b = { variants: n, accepted, argon2_real: slow, store_unchanged: (await s.stateHash()) === before };
  }
}

// (e) gộp
{
  const cases = json("merge-cases.json");
  let bad = 0;
  for (const c of cases) {
    const m = S.mergeDocs(c.local, c.incoming);
    try { S.validate.document(m); } catch (e) { bad++; console.log(`     ${c.name}: ${e.message}`); continue; }
    if (sha(S.canonicalJson(m)) !== c.merged_sha256) { bad++; console.log(`     ${c.name}: sha khác Rust`); }
  }
  // Một ca đi qua IndexedDB thật (applyImport "merge").
  const c = cases.find((x) => x.name.startsWith("duplicate photo"));
  const s = await freshStore();
  await s.applyImport(await S.verifyContents(S.canonicalJson(c.local), [{ id: c.local.photos[0].id, bytes: exPhoto }]), "replace");
  await s.applyImport(await S.verifyContents(S.canonicalJson(c.incoming), c.incoming.photos.map((p) => ({ id: p.id, bytes: exPhoto }))), "merge");
  const snap = await s.snapshot({ version: "0.0.0", platform: "pwa" }, "1970-01-01T00:00:00Z");
  const want = S.mergeDocs(c.local, c.incoming);
  const viaDb = ["persons", "events", "event_types", "photos"].every((k) => S.canonicalJson(snap[k]) === S.canonicalJson(want[k]));
  ok(bad === 0 && viaDb, `R6(e) ${cases.length} ca gộp: sha256 kết quả JS = Rust; qua IndexedDB = hàm thuần`);
  results.r6_e = { n: cases.length, mismatches: bad, indexeddb_path_matches: viaDb };
}

// Lớp lưu trữ: put/delete, V3, V7, backup hook, persist
{
  const s = await freshStore();
  const ex = JSON.parse(exJsonText);
  const p = S.buildPerson(ex.persons[1]);
  await s.putPerson(p);
  ok(JSON.stringify(await s.person(p.id)) === JSON.stringify(p), "putPerson → person() trả đúng bản ghi");
  ok((await s.persistResult()) === false, "lưu hồ sơ đầu tiên gọi requestPersistentStorage (Node không có navigator.storage → false, đã ghi)");
  const since = await s.backupPendingSince();
  ok(typeof since === "string" && !(await s.backupReminderDue()), "có thay đổi → backup_pending_since; chưa quá 30 ngày → không nhắc");
  const later = new Date(Date.parse(since) + 30 * 86400e3).toISOString().slice(0, 19) + "Z";
  ok(await s.backupReminderDue(later), "quá 30 ngày → backupReminderDue = true");
  ok(S.backupDue(0, since, later) === false, "backup_nag_days = 0 → tắt nhắc");
  ok((await code(() => s.putPerson({ ...p, photo_id: "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5eff" }))) === "invalid", "V3: photo_id không tồn tại → từ chối");
  ok((await code(() => s.putEventType({ id: "a1b2c3d4-0000-4000-8000-000000000001", builtin: null, ten: "TỐT NGHIỆP", default_recurring: false, created_at: ex.exported_at, updated_at: ex.exported_at, rev: 1 }))) === "invalid", "V7: tên loại trùng (khác hoa/thường) → từ chối");
  ok((await code(() => s.putEvent({ ...ex.events[0], person_ids: [p.id] }))) === "OK", "putEvent hợp lệ");
  ok((await s.eventsOf(p.id)).length === 1, "chỉ mục by_person");
  await s.deletePerson(p.id);
  ok((await s.events())[0].person_ids.length === 0, "xóa người → gỡ khỏi sự kiện (01 §3.4)");
  let bad = p; bad = { ...bad, death: { ...bad.death, lunar: { ...bad.death.lunar, day: 11 } } };
  ok((await code(() => s.putPerson(bad))) === "invalid", "V5: trường dẫn xuất sai → từ chối");
  results.store_api = "ok";
}

writeFileSync(join(OUT, "results.json"), JSON.stringify(results, null, 2) + "\n");
console.log(`\n${fails ? `KHÔNG ĐẠT: ${fails} lỗi` : "ĐẠT"} — ${join("pwa/test/store/out/results.json")}`);
process.exit(fails ? 1 : 0);
