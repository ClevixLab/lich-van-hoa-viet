// lvhv-store-tool (bản TypeScript/PWA) — công cụ hộp đen cho bên kiểm định độc lập (docs/v0.2/10-CONG-CU-KIEM-DINH.md).
//
// Cùng lệnh và cùng dạng kết quả với bản Rust (ffi/src/bin/lvhv-store-tool.rs), chạy trên lớp lưu trữ thật của PWA
// (pwa/src/store, dựng bằng esbuild như bản dựng PWA) với IndexedDB giả lập (fake-indexeddb). Chỉ gọi API công khai
// của src/store/index.ts; không sửa mã nguồn PWA.
//
//   node test/tools/lvhv-store-tool.mjs open <thư-mục-bộ-mẫu> [--force-migration-failure] [--now <unix_s>]
//   node test/tools/lvhv-store-tool.mjs import <tệp-sao-lưu> <mật-khẩu> [--merge|--replace] [--into <thư-mục-bộ-mẫu>]
//   node test/tools/lvhv-store-tool.mjs save-person <thư-mục-bộ-mẫu|-> <person.json|->
//   node test/tools/lvhv-store-tool.mjs suggestions <deceased|living> [nam|nu]
//   node test/tools/lvhv-store-tool.mjs display-capitalize <chuỗi>
//   node test/tools/lvhv-store-tool.mjs check-media <photo|attachment> <mime> <tệp> [width height]
//   node test/tools/lvhv-store-tool.mjs export <thư-mục-bộ-mẫu|-> <mật-khẩu> <tệp-ra> [--now <unix_s>]
//   node test/tools/lvhv-store-tool.mjs inspect-export <tệp-sao-lưu> <mật-khẩu>
//   node test/tools/lvhv-store-tool.mjs add-photo <thư-mục-bộ-mẫu|-> <mime> <tệp> <width> <height> [--id <uuid>]
//   node test/tools/lvhv-store-tool.mjs delete-person <thư-mục-bộ-mẫu|-> <id>
//   node test/tools/lvhv-store-tool.mjs scenario tray_photo [<thư-mục-làm-việc>]
//   Tùy chọn chung cho open/import/save-person/export/add-photo/delete-person: --state <tệp> (giữ kho giữa các lần
//   chạy, xem dưới). `scenario tray_photo` (04 Bổ sung B8, R15 (g)) in đúng cùng JSON với bản Rust; thư mục làm việc
//   không dùng (kho IndexedDB trong tiến trình), nhận để cùng cú pháp.
//
// Khác bản Rust (do trình duyệt không có "thư mục kho"):
// - Kho IndexedDB chỉ sống trong tiến trình. <thư-mục-bộ-mẫu> là một thư mục theo bố cục bộ mẫu đóng băng
//   (test-vectors/released/<bản>/: `store-snapshot.json` + `store/photos/<id>.img`). Công cụ dựng kho IndexedDB phiên
//   bản 1 (như bản dựng 0.2 trước YC-2026-07) từ đó, rồi mở bằng `openStore` của bản này (chuyển đổi trong
//   onupgradeneeded). Thư mục không có `store-snapshot.json` (hoặc `-`) → kho trống mới. Không ghi gì vào thư mục.
// - --state <tệp>: nếu tệp đã có, kho IndexedDB được dựng lại từ đó (thay cho <thư-mục-bộ-mẫu>); sau lệnh, toàn bộ
//   IndexedDB (phiên bản, kho đối tượng, chỉ mục, mọi bản ghi; Blob ở dạng base64) được ghi lại vào tệp dạng JSON chuẩn.
//   Nhờ vậy `open` chạy hai lần trên cùng một kho được, như thư mục kho của bản Rust.
// - --force-migration-failure, --now: `openStore(factory, { forceMigrationFailure, now })` (như Rust `OpenOptions`).
//
// stdout: một dòng JSON chuẩn (canonicalJson). Mã thoát: 0 thành công; 1 bị từ chối/lỗi; 2 sai cú pháp.
import "fake-indexeddb/auto";
import { build } from "esbuild";
import { createHash, randomUUID } from "node:crypto";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const REPO = resolve(ROOT, "..");
const USAGE = `dùng:
  node test/tools/lvhv-store-tool.mjs open <thư-mục-bộ-mẫu> [--force-migration-failure] [--now <unix_s>]
  node test/tools/lvhv-store-tool.mjs import <tệp-sao-lưu> <mật-khẩu> [--merge|--replace] [--into <thư-mục-bộ-mẫu>]
  node test/tools/lvhv-store-tool.mjs save-person <thư-mục-bộ-mẫu|-> <person.json|->
  node test/tools/lvhv-store-tool.mjs suggestions <deceased|living> [nam|nu]
  node test/tools/lvhv-store-tool.mjs display-capitalize <chuỗi>
  node test/tools/lvhv-store-tool.mjs check-media <photo|attachment> <mime> <tệp> [width height]
  node test/tools/lvhv-store-tool.mjs export <thư-mục-bộ-mẫu|-> <mật-khẩu> <tệp-ra> [--now <unix_s>]
  node test/tools/lvhv-store-tool.mjs inspect-export <tệp-sao-lưu> <mật-khẩu>
  node test/tools/lvhv-store-tool.mjs add-photo <thư-mục-bộ-mẫu|-> <mime> <tệp> <width> <height> [--id <uuid>]
  node test/tools/lvhv-store-tool.mjs delete-person <thư-mục-bộ-mẫu|-> <id>
  node test/tools/lvhv-store-tool.mjs scenario tray_photo [<thư-mục-làm-việc>]
  tùy chọn chung (open, import, save-person, export, add-photo, delete-person): --state <tệp>`;

class Usage extends Error {}
const sha = (b) => createHash("sha256").update(b).digest("hex");

// ---------- dựng mô-đun lưu trữ của PWA ----------
const tmp = mkdtempSync(join(tmpdir(), "lvhv-store-tool-"));
let S;
try {
  const bundle = join(tmp, "store.bundle.mjs");
  await build({
    stdin: { contents: `export * from "./src/store/index.ts"; export * as bc from "./src/store/backup-crypto.ts";`, resolveDir: ROOT, loader: "ts" },
    bundle: true, format: "esm", platform: "node", target: "es2022", outfile: bundle, logLevel: "warning",
    plugins: [{ name: "engine", setup(b) { b.onResolve({ filter: /^#engine-impl$/ }, () => ({ path: join(REPO, "tsport/src/index.ts") })); } }],
  });
  S = await import(pathToFileURL(bundle).href);
} finally {
  rmSync(tmp, { recursive: true, force: true });
}
const { IDBFactory } = await import("fake-indexeddb");

const errJson = (e) => {
  const o = { kind: e?.constructor?.name ?? "Error", code: e?.code ?? null, display: String(e?.message ?? e) };
  const c = e?.cause;
  if (c) {
    o.cause = { kind: c.constructor?.name ?? "Error", code: c.code ?? null, display: String(c.message ?? c) };
    if (c.path !== undefined) o.cause.path = c.path;
  }
  if (e?.path !== undefined) o.path = e.path;
  return o;
};

// ---------- kho ----------
/** Dựng kho IndexedDB phiên bản 1 từ thư mục bộ mẫu (cùng cách pwa/test/store/run.mjs). */
async function v1Factory(dir) {
  const f = new IDBFactory();
  const doc = JSON.parse(readFileSync(join(dir, "store-snapshot.json"), "utf8"));
  await new Promise((ok, ko) => {
    const r = f.open("lvhv", 1);
    r.onupgradeneeded = () => {
      const db = r.result, tx = r.transaction;
      db.createObjectStore("persons", { keyPath: "id" });
      db.createObjectStore("events", { keyPath: "id" }).createIndex("by_person", "person_ids", { multiEntry: true });
      db.createObjectStore("event_types", { keyPath: "id" }).createIndex("by_name_key", "name_key", { unique: true });
      db.createObjectStore("photos", { keyPath: "id" });
      db.createObjectStore("settings");
      db.createObjectStore("meta");
      tx.objectStore("settings").put(doc.settings, "app");
      tx.objectStore("meta").put(1, "schema_version");
      for (const t of doc.event_types) tx.objectStore("event_types").put({ id: t.id, name_key: S.validate.nameKey(t.ten), doc: t });
      for (const p of doc.persons) tx.objectStore("persons").put(p);
      for (const e of doc.events) tx.objectStore("events").put(e);
      for (const ph of doc.photos) tx.objectStore("photos").put({ id: ph.id, meta: ph, blob: new Blob([readFileSync(join(dir, "store", "photos", `${ph.id}.img`))], { type: ph.mime }) });
    };
    r.onsuccess = () => { r.result.close(); ok(); };
    r.onerror = () => ko(r.error);
  });
  return f;
}

// ---------- --state: lưu / nạp toàn bộ IndexedDB ----------
let STATE = null; // đường dẫn tệp --state (null = không giữ)

const reqP = (r) => new Promise((ok, ko) => { r.onsuccess = () => ok(r.result); r.onerror = () => ko(r.error); });
async function encodeVal(v) {
  if (v instanceof Blob) return { $blob: { type: v.type, base64: Buffer.from(await v.arrayBuffer()).toString("base64") } };
  if (Array.isArray(v)) return Promise.all(v.map(encodeVal));
  if (v && typeof v === "object") { const o = {}; for (const [k, x] of Object.entries(v)) o[k] = await encodeVal(x); return o; }
  return v;
}
function decodeVal(v) {
  if (Array.isArray(v)) return v.map(decodeVal);
  if (v && typeof v === "object") {
    if (v.$blob && Object.keys(v).length === 1) return new Blob([Buffer.from(v.$blob.base64, "base64")], { type: v.$blob.type });
    const o = {}; for (const [k, x] of Object.entries(v)) o[k] = decodeVal(x); return o;
  }
  return v;
}
/** Toàn bộ một IDBDatabase dạng JSON (phiên bản, kho đối tượng, chỉ mục, bản ghi theo khóa). */
async function dumpDb(db) {
  const stores = {};
  for (const name of Array.from(db.objectStoreNames).sort()) {
    const tx = db.transaction(name, "readonly"), os = tx.objectStore(name);
    const keys = await reqP(os.getAllKeys()), vals = await reqP(os.getAll());
    const indexes = Array.from(os.indexNames).sort().map((n) => { const ix = os.index(n); return { name: n, keyPath: ix.keyPath, unique: ix.unique, multiEntry: ix.multiEntry }; });
    const records = [];
    for (let i = 0; i < keys.length; i++) records.push({ key: keys[i], value: await encodeVal(vals[i]) });
    stores[name] = { keyPath: os.keyPath, autoIncrement: os.autoIncrement, indexes, records };
  }
  return { format: "lvhv-idb-state", name: db.name, version: db.version, stores };
}
async function factoryFromState(file) {
  const d = JSON.parse(readFileSync(file, "utf8"));
  if (d.format !== "lvhv-idb-state") throw new Error(`${file}: không phải tệp --state`);
  const f = new IDBFactory();
  await new Promise((ok, ko) => {
    const r = f.open(d.name, d.version);
    r.onupgradeneeded = () => {
      const db = r.result, tx = r.transaction;
      for (const [name, s] of Object.entries(d.stores)) {
        const os = db.createObjectStore(name, { keyPath: s.keyPath, autoIncrement: s.autoIncrement });
        for (const ix of s.indexes) os.createIndex(ix.name, ix.keyPath, { unique: ix.unique, multiEntry: ix.multiEntry });
        for (const rec of s.records) {
          if (s.keyPath === null) os.put(decodeVal(rec.value), rec.key); else os.put(decodeVal(rec.value));
        }
      }
      void tx;
    };
    r.onsuccess = () => { r.result.close(); ok(); };
    r.onerror = () => ko(r.error);
  });
  return f;
}
/** Kết thúc lệnh trên kho: ghi --state (nếu có) rồi đóng. */
async function finish(st) {
  if (STATE) writeFileSync(STATE, S.canonicalJson(await dumpDb(st.db)) + "\n");
  st.close();
}

/** Mở kho: từ --state (nếu tệp đã có), từ bộ mẫu (phiên bản 1 → chuyển đổi) hoặc kho trống. */
async function openFrom(dir, { force = false, now = null } = {}) {
  let f, source;
  if (STATE && existsSync(STATE)) { f = await factoryFromState(STATE); source = "state"; }
  else if (dir && dir !== "-" && existsSync(join(dir, "store-snapshot.json"))) { f = await v1Factory(dir); source = "fixture_v1"; }
  else { f = new IDBFactory(); source = "fresh"; }
  const opts = { forceMigrationFailure: force };
  if (now !== null) opts.now = S.nowRfc3339(new Date(now * 1000));
  const st = await S.openStore(f, opts);
  return { st, source };
}

async function storeJson(st) {
  const doc = await st.snapshot({ version: "0.0.0", platform: "pwa" }, "1970-01-01T00:00:00Z");
  delete doc.app; delete doc.exported_at;
  const photo_bytes = {}, attachment_bytes = {};
  for (const p of await st.photos()) {
    try { const b = await st.photoBytes(p.id); photo_bytes[p.id] = { bytes: b.length, sha256: sha(b) }; } catch (e) { photo_bytes[p.id] = { error: errJson(e) }; }
  }
  for (const a of await st.attachments()) {
    try { const b = await st.attachmentBytes(a.id); attachment_bytes[a.id] = { bytes: b.length, sha256: sha(b) }; } catch (e) { attachment_bytes[a.id] = { error: errJson(e) }; }
  }
  let state_sha256 = null;
  try { state_sha256 = await st.stateHash(); } catch { /* như Rust: null khi không băm được */ }
  return { document: doc, photo_bytes, attachment_bytes, state_sha256, idb_version: st.db.version, backup_pending_since: await st.backupPendingSince() };
}

const result = async (fn) => { try { await fn(); return { ok: true }; } catch (e) { return { ok: false, error: errJson(e) }; } };

// ---------- lệnh ----------
async function cmdOpen(a) {
  let dir = null, force = false, now = null;
  for (let i = 0; i < a.length; i++) {
    const x = a[i];
    if (x === "--force-migration-failure") force = true;
    else if (x === "--now") { now = Number(a[++i]); if (!Number.isSafeInteger(now)) throw new Usage("--now cần <unix_s> (số nguyên giây)"); }
    else if (x.startsWith("--")) throw new Usage(`tùy chọn lạ: ${x}`);
    else if (dir === null) dir = x;
    else throw new Usage(`đối số thừa: ${x}`);
  }
  if (dir === null) throw new Usage("thiếu <thư-mục-bộ-mẫu>");
  let o;
  try { o = await openFrom(dir, { force, now }); } catch (e) { return [{ command: "open", ok: false, stage: "open", error: errJson(e) }, false]; }
  const { st, source } = o;
  const ms = st.migrationStatus();
  const out = { command: "open", ok: true, source, force_migration_failure: force, now_unix: now, migration_status: ms, store: await storeJson(st) };
  if (ms.state === "read_only") {
    const probe = {};
    probe.put_settings = await result(async () => st.putSettings(await st.settings()));
    const p = (await st.persons())[0];
    if (p) {
      probe.put_person = await result(() => st.putPerson(p));
      probe.delete_person = await result(() => st.deletePerson(p.id));
    }
    try { const b = await st.exportBackup("mật khẩu thử nghiệm", { version: "0.0.0", platform: "pwa" }); probe.export_backup = { ok: true, bytes: b.length }; }
    catch (e) { probe.export_backup = { ok: false, error: errJson(e) }; }
    out.read_only_probe = probe;
  }
  await finish(st);
  return [out, true];
}

async function cmdImport(a) {
  const pos = []; let mode = null, into = null;
  for (let i = 0; i < a.length; i++) {
    const x = a[i];
    if (x === "--merge" || x === "--replace") { if (mode) throw new Usage("chỉ một --merge/--replace"); mode = x.slice(2); }
    else if (x === "--into") { into = a[++i]; if (into === undefined) throw new Usage("--into cần <thư-mục-bộ-mẫu>"); }
    else if (x.startsWith("--")) throw new Usage(`tùy chọn lạ: ${x}`);
    else pos.push(x);
  }
  if (pos.length !== 2) throw new Usage("cần <tệp-sao-lưu> <mật-khẩu>");
  mode ??= "merge"; // mặc định Gộp, như ứng dụng (BK-S1)
  const [file, password] = pos;
  let bytes;
  try { bytes = new Uint8Array(readFileSync(file)); } catch (e) { return [{ command: "import", ok: false, stage: "read_file", error: errJson(e) }, false]; }
  const base = { command: "import", mode, file_sha256: sha(bytes), into };
  let vb;
  try { vb = await S.openBackup(bytes, password); } catch (e) { return [{ ...base, ok: false, stage: "open_backup", error: errJson(e) }, false]; }
  let o;
  try { o = await openFrom(into); } catch (e) { return [{ ...base, ok: false, stage: "open_store", error: errJson(e) }, false]; }
  const { st, source } = o;
  const before = { migration_status: st.migrationStatus(), state_sha256: await st.stateHash().catch(() => null), source };
  try { await st.applyImport(vb, mode); } catch (e) { await finish(st); return [{ ...base, ok: false, stage: "apply_import", before, error: errJson(e) }, false]; }
  const out = { ...base, ok: true, preview: vb.preview, verified_document: vb.doc, before, store: await storeJson(st) };
  await finish(st);
  return [out, true];
}

async function cmdSavePerson(a) {
  if (a.length !== 2) throw new Usage("cần <thư-mục-bộ-mẫu|-> <person.json|->");
  const [dir, file] = a;
  let input;
  try { input = JSON.parse(readFileSync(file === "-" ? 0 : file, "utf8")); } catch (e) { return [{ command: "save-person", ok: false, stage: "read_file", error: errJson(e) }, false]; }
  let o;
  try { o = await openFrom(dir); } catch (e) { return [{ command: "save-person", ok: false, stage: "open", error: errJson(e) }, false]; }
  const { st, source } = o;
  const ms = st.migrationStatus();
  try {
    // Như màn hình hồ sơ (src/people/forms.ts): gắn id / created_at / updated_at / rev, rồi
    // buildPerson(normalizePersonNames(base)) → validate.person → putPerson.
    const now = S.nowRfc3339();
    const id = typeof input.id === "string" && input.id ? input.id : randomUUID();
    const old = await st.person(id).catch(() => null);
    const base = { ...input, id, created_at: old?.created_at ?? now, updated_at: now, rev: old ? old.rev + 1 : 1 };
    const p = S.buildPerson(S.normalizePersonNames(base));
    S.validate.person(p, "/person");
    await st.putPerson(p);
    const saved = await st.person(id);
    await finish(st);
    return [{ command: "save-person", ok: true, source, migration_status: ms, saved }, true];
  } catch (e) {
    await finish(st);
    return [{ command: "save-person", ok: false, stage: "save_person", source, migration_status: ms, error: errJson(e) }, false];
  }
}

function cmdSuggestions(a) {
  if (a.length < 1 || a.length > 2) throw new Usage("cần <deceased|living> [nam|nu]");
  if (a[0] !== "deceased" && a[0] !== "living") throw new Usage(`phải là deceased hoặc living: ${a[0]}`);
  const g = a[1] ?? null;
  if (g !== null && g !== "nam" && g !== "nu") throw new Usage(`giới tính phải là nam hoặc nu: ${g}`);
  const deceased = a[0] === "deceased";
  return [{ command: "suggestions", ok: true, deceased, gioi_tinh: g, danh_xung: S.danhXungSuggestions(deceased, g), quan_he: S.quanHeSuggestions(deceased) }, true];
}

function cmdDisplay(a) {
  if (a.length !== 1) throw new Usage("cần <chuỗi>");
  return [{ command: "display-capitalize", ok: true, input: a[0], output: S.displayCapitalize(a[0]) }, true];
}

function cmdCheckMedia(a) {
  if (a.length !== 3 && a.length !== 5) throw new Usage("cần <photo|attachment> <mime> <tệp> [width height]");
  const [kind, mime, file] = a;
  if (kind !== "photo" && kind !== "attachment") throw new Usage(`phải là photo hoặc attachment: ${kind}`);
  const dims = a.length === 5 ? a.slice(3).map(Number) : [null, null];
  if (dims.some((x) => x !== null && !(Number.isInteger(x) && x >= 0))) throw new Usage("width/height phải là số nguyên không âm");
  let bytes;
  try { bytes = new Uint8Array(readFileSync(file)); } catch (e) { return [{ command: "check-media", ok: false, stage: "read_file", error: errJson(e) }, false]; }
  return [{ command: "check-media", ok: true, kind, mime, bytes: bytes.length, sha256: sha(bytes), verdict: S.mediaVerdict(kind, mime, bytes, dims[0], dims[1]) }, true];
}

function legacyPaths(v, path = "", out = []) {
  if (Array.isArray(v)) v.forEach((x, i) => legacyPaths(x, `${path}/${i}`, out));
  else if (v && typeof v === "object") for (const k of Object.keys(v).sort()) { if (k === "legacy") out.push(`${path}/${k}`); legacyPaths(v[k], `${path}/${k}`, out); }
  return out;
}

async function cmdExport(a) {
  const pos = []; let now = null;
  for (let i = 0; i < a.length; i++) {
    if (a[i] === "--now") { now = Number(a[++i]); if (!Number.isSafeInteger(now)) throw new Usage("--now cần <unix_s> (số nguyên giây)"); }
    else if (a[i].startsWith("--")) throw new Usage(`tùy chọn lạ: ${a[i]}`);
    else pos.push(a[i]);
  }
  if (pos.length !== 3) throw new Usage("cần <thư-mục-bộ-mẫu|-> <mật-khẩu> <tệp-ra>");
  const [dir, password, outFile] = pos;
  let o;
  try { o = await openFrom(dir, { now }); } catch (e) { return [{ command: "export", ok: false, stage: "open", error: errJson(e) }, false]; }
  const { st, source } = o;
  const ms = st.migrationStatus();
  let bytes;
  try { bytes = await st.exportBackup(password, { version: "0.2.0", platform: "pwa" }); }
  catch (e) { await finish(st); return [{ command: "export", ok: false, stage: "export_backup", source, migration_status: ms, error: errJson(e) }, false]; }
  await finish(st);
  writeFileSync(outFile, bytes);
  return [{ command: "export", ok: true, source, now_unix: now, migration_status: ms, out_file: outFile, bytes: bytes.length, sha256: sha(bytes) }, true];
}

async function cmdInspectExport(a) {
  if (a.length !== 2) throw new Usage("cần <tệp-sao-lưu> <mật-khẩu>");
  const [file, password] = a;
  let bytes;
  try { bytes = new Uint8Array(readFileSync(file)); } catch (e) { return [{ command: "inspect-export", ok: false, stage: "read_file", error: errJson(e) }, false]; }
  let c;
  try { c = await S.bc.importBackup(bytes, password); } catch (e) { return [{ command: "inspect-export", ok: false, stage: "decrypt", error: errJson(e) }, false]; }
  let doc;
  try { doc = JSON.parse(c.json); } catch (e) { return [{ command: "inspect-export", ok: false, stage: "parse_json", error: errJson(e) }, false]; }
  return [{
    command: "inspect-export", ok: true, file_sha256: sha(bytes), json_sha256: sha(Buffer.from(c.json, "utf8")),
    json_is_canonical: S.canonicalJson(doc) === c.json, document: doc,
    blobs: c.photos.map((b) => ({ id: S.bc.bytesToUuid(b.id), bytes: b.bytes.length, sha256: sha(b.bytes) })),
    legacy_paths: legacyPaths(doc),
  }, true];
}

// ---------- add-photo, delete-person (YC-2026-10) ----------
async function cmdAddPhoto(a) {
  const pos = []; let id = null;
  for (let i = 0; i < a.length; i++) {
    if (a[i] === "--id") { id = a[++i]; if (id === undefined) throw new Usage("--id cần <uuid>"); }
    else if (a[i].startsWith("--")) throw new Usage(`tùy chọn lạ: ${a[i]}`);
    else pos.push(a[i]);
  }
  if (pos.length !== 5) throw new Usage("cần <thư-mục-bộ-mẫu|-> <mime> <tệp> <width> <height>");
  const [dir, mime, file, w, h] = pos;
  const [width, height] = [Number(w), Number(h)];
  if (![width, height].every((x) => Number.isInteger(x) && x >= 0)) throw new Usage("width/height phải là số nguyên không âm");
  let bytes;
  try { bytes = new Uint8Array(readFileSync(file)); } catch (e) { return [{ command: "add-photo", ok: false, stage: "read_file", error: errJson(e) }, false]; }
  let o;
  try { o = await openFrom(dir); } catch (e) { return [{ command: "add-photo", ok: false, stage: "open", error: errJson(e) }, false]; }
  const { st } = o;
  try {
    const photo = await st.addPhoto({ id: id ?? randomUUID(), mime, width, height, bytes });
    await finish(st);
    return [{ command: "add-photo", ok: true, photo }, true];
  } catch (e) {
    await finish(st);
    return [{ command: "add-photo", ok: false, stage: "add_photo", error: errJson(e), outcome: outcomeOf(e) }, false];
  }
}

async function cmdDeletePerson(a) {
  if (a.length !== 2) throw new Usage("cần <thư-mục-bộ-mẫu|-> <id>");
  const [dir, id] = a;
  let o;
  try { o = await openFrom(dir); } catch (e) { return [{ command: "delete-person", ok: false, stage: "open", error: errJson(e) }, false]; }
  const { st } = o;
  try { await st.deletePerson(id); await finish(st); return [{ command: "delete-person", ok: true, id }, true]; }
  catch (e) { await finish(st); return [{ command: "delete-person", ok: false, stage: "delete_person", error: errJson(e), outcome: outcomeOf(e) }, false]; }
}

// ---------- scenario tray_photo (04 Bổ sung B8, R15 (c)–(g)) — cùng bước, cùng JSON với bản Rust ----------
const vCode = (msg) => { const m = /(?:^| )(V\d \/\S*?)(?=: |$)/.exec(String(msg)); return m ? m[1] : null; };
/** Như Rust `ffi_outcome` / `store_outcome`: "ok" | "invalid <mã> <đường dẫn>" | "input" | "not_found" | "backup" | "storage". */
function outcomeOf(e) {
  if (e instanceof S.validate.ValidationError) return `invalid ${e.code} ${e.path}`;
  const c = e?.cause;
  if (e?.code === "invalid") {
    if (c instanceof S.validate.ValidationError) return `invalid ${c.code} ${c.path}`;
    const v = vCode(e.message);
    return v ? `invalid ${v}` : "input";
  }
  if (e?.code === "input" || e?.code === "read_only") return "input";
  if (e?.code === "not_found") return "not_found";
  if (e?.code === "backup" || e?.code === "newer_schema") return "backup";
  return "storage";
}
const outcome = async (fn) => { try { await fn(); return "ok"; } catch (e) { return outcomeOf(e); } };

const TP = "mật khẩu thử nghiệm";
const T1 = "0b8e4f52-3c1a-4d2e-9f60-0000000000a1", T2 = "0b8e4f52-3c1a-4d2e-9f60-0000000000a2";
const T3 = "0b8e4f52-3c1a-4d2e-9f60-0000000000a3", T4 = "0b8e4f52-3c1a-4d2e-9f60-0000000000a4";
const TX = "0b8e4f52-3c1a-4d2e-9f60-0000000000af";
const T0 = "0b8e4f52-3c1a-4d2e-9f60-0000000000a0", T5 = "0b8e4f52-3c1a-4d2e-9f60-0000000000a5", T6 = "0b8e4f52-3c1a-4d2e-9f60-0000000000a6";
const D1 = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01", D2 = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02", L3 = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e03";
const PORTRAIT = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e09";
const TV = join(REPO, "docs/v0.2/schema/test-vectors");

class Tray {
  constructor(st) { this.st = st; }
  static async fresh() { return new Tray(await S.openStore(new IDBFactory())); }
  person(id) { return this.st.person(id); }
  /** Như cmdSavePerson (đường lưu của màn hình hồ sơ, như Rust save_person_json). */
  async save(input) {
    return outcome(async () => {
      const now = S.nowRfc3339();
      const old = await this.st.person(input.id).catch(() => null);
      const base = { ...structuredClone(input), created_at: old?.created_at ?? now, updated_at: now, rev: old ? old.rev + 1 : 1 };
      const p = S.buildPerson(S.normalizePersonNames(base));
      S.validate.person(p, "/person");
      await this.st.putPerson(p);
    });
  }
  async withTray(id, tray) { const p = structuredClone(await this.person(id)); p.memorial.tray_photo_id = tray; return this.save(p); }
  add(id, file, w, h, mime = "image/jpeg") { return outcome(() => this.st.addPhoto({ id, mime, width: w, height: h, bytes: new Uint8Array(readFileSync(join(TV, file))) })); }
  async photos() {
    const ids = (await this.st.photos()).map((p) => p.id).sort();
    const used = new Set((await this.st.persons()).flatMap((p) => S.validate.personPhotoRefs(p)));
    let missing = 0;
    for (const id of ids) await this.st.photoBytes(id).catch(() => { missing += 1; });
    return { ids, orphan_files: missing, orphan_records: ids.filter((i) => !used.has(i)).length };
  }
  async trayOf(id) {
    const p = await this.person(id);
    const m = p.memorial;
    if (m === null || typeof m !== "object") return "no_memorial";
    return Object.prototype.hasOwnProperty.call(m, "tray_photo_id") ? m.tray_photo_id : "absent";
  }
  async photoSha(id) { try { return sha(await this.st.photoBytes(id)); } catch { return null; } }
}
/** Kết quả `openBackup` như người dùng thấy: lỗi nào cũng là thông báo chung (03 §2.4) → "backup" (như Rust FFI
 *  `open_backup` → `LvhvDataError::Backup`); chi tiết xem ở `verify`. */
const openOutcome = (e) => (e?.code === "backup" || e?.message === "Sai mật khẩu hoặc tệp bị hỏng" ? "backup" : outcomeOf(e));
async function trayImport(t, file, mode) {
  let v;
  try { v = await S.openBackup(new Uint8Array(file), TP); } catch (e) { return openOutcome(e); }
  // Như Rust FFI `apply_import` (lỗi → `LvhvDataError::Backup`, thông báo chung) và màn hình khôi phục của PWA ("Chưa khôi
  // phục được — dữ liệu trên máy giữ nguyên."): người dùng chỉ thấy thông báo chung.
  try { await t.st.applyImport(v, mode); return "ok"; } catch (e) { const o = outcomeOf(e); return o === "storage" ? o : "backup"; }
}
async function fixedFile(doc, blobs) {
  return S.bc.exportWithFixedRng(TP, S.canonicalJson(doc), blobs.map(([id, bytes]) => ({ id: S.bc.uuidToBytes(id), bytes })), new Uint8Array(16).fill(0x11), new Uint8Array(24).fill(0x22));
}
async function contentsOf(t) {
  const d = await t.st.snapshot({ version: "0.2.0", platform: "pwa" }, "2026-11-07T00:00:00Z");
  d.app = { name: "Lịch và Văn hóa Việt", version: "0.2.0", platform: "android" };
  d.exported_at = "2026-11-07T00:00:00Z";
  const blobs = [];
  for (const p of d.photos) blobs.push([p.id, await t.st.photoBytes(p.id)]);
  return [d, blobs];
}
async function crafted(doc, blobs) {
  const json = S.canonicalJson(doc);
  const verify = await outcome(() => S.verifyContents(json, blobs.map(([id, bytes]) => ({ id, bytes }))));
  const file = await S.bc.exportWithFixedRng(TP, json, blobs.map(([id, bytes]) => ({ id: S.bc.uuidToBytes(id), bytes })), new Uint8Array(16).fill(0x11), new Uint8Array(24).fill(0x22));
  let open = "ok";
  try { await S.openBackup(file, TP); } catch (e) { open = openOutcome(e); }
  return { open_backup: open, verify };
}

async function cmdScenario(a) {
  if (a.length < 1 || a.length > 2) throw new Usage("cần tray_photo [<thư-mục-làm-việc>]");
  if (a[0] !== "tray_photo") throw new Usage(`kịch bản lạ: ${a[0]}`);
  const out = {};
  const example = readFileSync(join(TV, "example-v1.lvhv"));
  const exampleDoc = JSON.parse(readFileSync(join(TV, "example-v1.json"), "utf8"));
  const cj = S.canonicalJson;

  // (e) R14.
  const t = await Tray.fresh();
  const r = await trayImport(t, example, "replace");
  const got = await t.st.snapshot({ version: "0.2.0", platform: "pwa" }, "2026-11-07T00:00:00Z");
  delete got.app; delete got.exported_at; delete got.settings;
  const unchanged = cj(got.persons) === cj(exampleDoc.persons) && cj(got.photos) === cj(exampleDoc.photos) && cj(got.events) === cj(exampleDoc.events);
  out.e_old_backup = { import: r, unchanged, has_tray_key: cj(got).includes("tray_photo_id"), tray_d1: await t.trayOf(D1) };

  // (c) phía lõi.
  const exif = await t.add(TX, "media/jpeg-exif-gps.jpg", 768, 1024);
  const big = new Uint8Array(1_048_577); big.set(readFileSync(join(TV, "media/jpeg-ok.jpg")));
  const overBig = await outcome(() => t.st.addPhoto({ id: TX, mime: "image/jpeg", width: 768, height: 1024, bytes: big }));
  const add1 = await t.add(T1, "r6v2/photo-00.jpg", 300, 400);
  out.c_add_photo = { exif, over_1mib: overBig, ok: add1 };

  // V3.
  const set = await t.withTray(D1, T1);
  const l3 = structuredClone(await t.person(L3)); l3.photo_id = T1;
  const v3 = {
    dangling: await t.withTray(D2, "0b8e4f52-3c1a-4d2e-9f60-0000000000ee"),
    shared_tray: await t.withTray(D2, T1),
    tray_is_other_portrait: await t.withTray(D2, PORTRAIT),
    tray_is_own_portrait: await t.withTray(D1, PORTRAIT),
    portrait_is_other_tray: await t.save(l3),
  };
  out.set = { save: set, tray_d1: await t.trayOf(D1), photos: await t.photos(), v3 };
  // MA10-02, MA10-04.
  const add5 = await t.add(T5, "tray-photo/tray-1601x1200.jpg", 1601, 1200);
  const save5 = await t.withTray(D2, T5);
  const del5 = await outcome(() => t.st.deletePhoto(T5));
  const dd = structuredClone(await t.person(D2));
  dd.photo_id = "0b8e4f52-3c1a-4d2e-9f60-0000000000ee"; dd.memorial.tray_photo_id = "0b8e4f52-3c1a-4d2e-9f60-0000000000ee";
  out.g_rules = { tray_1601: { add_photo: add5, save: save5, delete_photo: del5 }, double_dangling: await t.save(dd), photos: await t.photos() };

  // (d)
  const resave = await t.withTray(D1, T1); // rev tăng để bản này thắng khi gộp (MA10-03)
  const file = await t.st.exportBackup(TP, { version: "0.2.0", platform: "pwa" });
  const sha1 = await t.photoSha(T1);
  const d = {};
  for (const [k, mode, seed] of [["replace", "replace", false], ["merge_empty", "merge", false], ["merge_over_old", "merge", true]]) {
    const u = await Tray.fresh();
    if (seed) await trayImport(u, example, "replace");
    const ri = await trayImport(u, file, mode);
    d[k] = { import: ri, tray_d1: await u.trayOf(D1), sha_equal: sha1 !== null && (await u.photoSha(T1)) === sha1, photos: await u.photos() };
    u.st.close();
  }
  const [doc, blobs] = await contentsOf(t);
  const dangling = structuredClone(doc); dangling.photos = dangling.photos.filter((p) => p.id !== T1);
  const exifB = new Uint8Array(readFileSync(join(TV, "media/jpeg-exif-gps.jpg")));
  const exifDoc = structuredClone(doc);
  for (const ph of exifDoc.photos) if (ph.id === T1) { ph.bytes = exifB.length; ph.sha256 = sha(exifB); ph.width = 768; ph.height = 1024; }
  const shared = structuredClone(doc); shared.persons.find((p) => p.id === L3).photo_id = T1;
  const bigB = new Uint8Array(readFileSync(join(TV, "tray-photo/tray-1601x1200.jpg")));
  const bigDoc = structuredClone(doc);
  for (const ph of bigDoc.photos) if (ph.id === T1) { ph.bytes = bigB.length; ph.sha256 = sha(bigB); ph.width = 1601; ph.height = 1200; }
  d.crafted = {
    dangling_tray: await crafted(dangling, blobs.filter(([i]) => i !== T1)),
    tray_with_exif: await crafted(exifDoc, blobs.map(([i, b]) => [i, i === T1 ? exifB : b])),
    shared_tray: await crafted(shared, blobs),
    control: await crafted(doc, blobs),
    tray_too_large: await crafted(bigDoc, blobs.map(([i, b]) => [i, i === T1 ? bigB : b])),
  };
  // F2.
  const nullDoc = structuredClone(doc); nullDoc.persons.find((p) => p.id === D2).memorial.tray_photo_id = null;
  {
    const u = await Tray.fresh();
    const ri = await trayImport(u, await fixedFile(nullDoc, blobs), "replace");
    const again = await u.st.exportBackup(TP, { version: "0.2.0", platform: "pwa" });
    const c = await S.bc.importBackup(again, TP);
    const snap = await u.st.snapshot({ version: "0.2.0", platform: "pwa" }, "2026-11-07T00:00:00Z");
    d.explicit_null = { import: ri, tray_d2: await u.trayOf(D2), store_has_null: JSON.stringify(snap).includes('"tray_photo_id":null'), reexport_has_null: c.json.includes('"tray_photo_id":null') };
    u.st.close();
  }
  // MA10-03.
  {
    const u = await Tray.fresh();
    await trayImport(u, example, "replace");
    const a0 = await u.add(T0, "r6v2/photo-08.jpg", 308, 408);
    const s0 = await u.withTray(D1, T0);
    const ri = await trayImport(u, file, "merge");
    const snap = await u.st.snapshot({ version: "0.2.0", platform: "pwa" }, "2026-11-07T00:00:00Z");
    d.merge_replaces_local_tray = { setup: [a0, s0, resave], import: ri, tray_d1: await u.trayOf(D1), photos: await u.photos(), old_tray_in_store: JSON.stringify(snap).includes(T0) };
    u.st.close();
  }
  // MA10-10: gộp làm hai hồ sơ cùng trỏ một ảnh mâm lễ.
  {
    const u = await Tray.fresh();
    await trayImport(u, example, "replace");
    const a1 = await u.add(T1, "r6v2/photo-00.jpg", 300, 400);
    const s1 = await u.withTray(D2, T1);
    const before = S.canonicalJson(await u.st.persons());
    const ri = await trayImport(u, file, "merge");
    d.merge_shared_tray = { setup: [a1, s1], import: ri, unchanged: S.canonicalJson(await u.st.persons()) === before, photos: await u.photos() };
    u.st.close();
  }
  out.d_round_trip = d;

  // (f)
  const f = {};
  await t.add(T2, "r6v2/photo-02.jpg", 302, 402);
  f.replace = { save: await t.withTray(D1, T2), photos: await t.photos() };
  f.remove = { save: await t.withTray(D1, null), tray_d1: await t.trayOf(D1), photos: await t.photos() };
  await t.add(T3, "r6v2/photo-04.jpg", 304, 404);
  const s3 = await t.withTray(D1, T3);
  const p = structuredClone(await t.person(D1));
  p.kind = "living"; p.death = null; p.memorial = null; p.reminders.gio = null; p.noi_an_tang = "";
  f.to_living = { set: s3, save: await t.save(p), tray_d1: await t.trayOf(D1), photos: await t.photos() };
  const a6 = await t.add(T6, "r6v2/photo-01.webp", 301, 401, "image/webp");
  const s6 = await t.withTray(D2, T6);
  f.webp = { add_photo: a6, save: s6, tray_d2: await t.trayOf(D2), photos: await t.photos() };
  await t.add(T4, "r6v2/photo-06.jpg", 306, 406);
  const s4 = await t.withTray(D2, T4);
  f.delete_person = { set: s4, delete: await outcome(() => t.st.deletePerson(D2)), photos: await t.photos() };
  out.f_cleanup = f;
  t.st.close();
  return [{ command: "scenario", scenario: "tray_photo", ok: true, result: out }, true];
}

const argv = process.argv.slice(2);
const si = argv.indexOf("--state");
if (si >= 0) {
  STATE = argv[si + 1] ?? null;
  argv.splice(si, 2);
}
const [cmd, ...rest] = argv;
const table = { open: cmdOpen, import: cmdImport, "save-person": cmdSavePerson, suggestions: cmdSuggestions, "display-capitalize": cmdDisplay,
  "check-media": cmdCheckMedia, export: cmdExport, "inspect-export": cmdInspectExport, "add-photo": cmdAddPhoto, "delete-person": cmdDeletePerson,
  scenario: cmdScenario };
try {
  if (si >= 0 && !STATE) throw new Usage("--state cần <tệp>");
  if (si >= 0 && !["open", "import", "save-person", "export", "add-photo", "delete-person"].includes(cmd)) throw new Usage(`--state không dùng với ${cmd}`);
  if (!table[cmd]) throw new Usage("");
  const [out, ok] = await table[cmd](rest);
  process.stdout.write(S.canonicalJson(out) + "\n");
  process.exitCode = ok ? 0 : 1;
} catch (e) {
  if (!(e instanceof Usage)) throw e;
  if (e.message) process.stderr.write(`lỗi: ${e.message}\n`);
  process.stderr.write(USAGE + "\n");
  process.exitCode = 2;
}
