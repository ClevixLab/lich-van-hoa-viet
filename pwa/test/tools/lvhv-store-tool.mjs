// lvhv-store-tool (bản TypeScript/PWA) — công cụ hộp đen cho bên kiểm định độc lập (docs/v0.2/10-CONG-CU-KIEM-DINH.md).
//
// Cùng lệnh và cùng dạng kết quả với bản Rust (ffi/src/bin/lvhv-store-tool.rs), chạy trên lớp lưu trữ thật của PWA
// (pwa/src/store, dựng bằng esbuild như bản dựng PWA) với IndexedDB giả lập (fake-indexeddb). Chỉ gọi API công khai
// của src/store/index.ts; không sửa mã nguồn PWA.
//
//   node test/tools/lvhv-store-tool.mjs open <thư-mục-bộ-mẫu> [--force-migration-failure]
//   node test/tools/lvhv-store-tool.mjs import <tệp-sao-lưu> <mật-khẩu> [--merge|--replace] [--into <thư-mục-bộ-mẫu>]
//   node test/tools/lvhv-store-tool.mjs save-person <thư-mục-bộ-mẫu|-> <person.json|->
//   node test/tools/lvhv-store-tool.mjs suggestions <deceased|living> [nam|nu]
//   node test/tools/lvhv-store-tool.mjs display-capitalize <chuỗi>
//   node test/tools/lvhv-store-tool.mjs check-media <photo|attachment> <mime> <tệp> [width height]
//
// Khác bản Rust (do trình duyệt không có "thư mục kho"):
// - Kho IndexedDB chỉ sống trong tiến trình. <thư-mục-bộ-mẫu> là một thư mục theo bố cục bộ mẫu đóng băng
//   (test-vectors/released/<bản>/: `store-snapshot.json` + `store/photos/<id>.img`). Công cụ dựng kho IndexedDB phiên
//   bản 1 (như bản dựng 0.2 trước YC-2026-07) từ đó, rồi mở bằng `openStore` của bản này (chuyển đổi trong
//   onupgradeneeded). Thư mục không có `store-snapshot.json` (hoặc `-`) → kho trống mới. Không ghi gì vào thư mục.
// - --force-migration-failure: `openStore(factory, { forceMigrationFailure: true })` (như Rust `OpenOptions`).
//
// stdout: một dòng JSON chuẩn (canonicalJson). Mã thoát: 0 thành công; 1 bị từ chối/lỗi; 2 sai cú pháp.
import "fake-indexeddb/auto";
import { build } from "esbuild";
import { createHash, randomUUID } from "node:crypto";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const REPO = resolve(ROOT, "..");
const USAGE = `dùng:
  node test/tools/lvhv-store-tool.mjs open <thư-mục-bộ-mẫu> [--force-migration-failure]
  node test/tools/lvhv-store-tool.mjs import <tệp-sao-lưu> <mật-khẩu> [--merge|--replace] [--into <thư-mục-bộ-mẫu>]
  node test/tools/lvhv-store-tool.mjs save-person <thư-mục-bộ-mẫu|-> <person.json|->
  node test/tools/lvhv-store-tool.mjs suggestions <deceased|living> [nam|nu]
  node test/tools/lvhv-store-tool.mjs display-capitalize <chuỗi>
  node test/tools/lvhv-store-tool.mjs check-media <photo|attachment> <mime> <tệp> [width height]`;

class Usage extends Error {}
const sha = (b) => createHash("sha256").update(b).digest("hex");

// ---------- dựng mô-đun lưu trữ của PWA ----------
const tmp = mkdtempSync(join(tmpdir(), "lvhv-store-tool-"));
let S;
try {
  const bundle = join(tmp, "store.bundle.mjs");
  await build({
    stdin: { contents: `export * from "./src/store/index.ts";`, resolveDir: ROOT, loader: "ts" },
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

/** Mở kho: từ bộ mẫu (phiên bản 1 → chuyển đổi) hoặc kho trống. */
async function openFrom(dir, force = false) {
  const fixture = dir && dir !== "-" && existsSync(join(dir, "store-snapshot.json"));
  const f = fixture ? await v1Factory(dir) : new IDBFactory();
  const st = await S.openStore(f, { forceMigrationFailure: force });
  return { st, source: fixture ? "fixture_v1" : "fresh" };
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
  let dir = null, force = false;
  for (const x of a) {
    if (x === "--force-migration-failure") force = true;
    else if (x.startsWith("--")) throw new Usage(`tùy chọn lạ: ${x}`);
    else if (dir === null) dir = x;
    else throw new Usage(`đối số thừa: ${x}`);
  }
  if (dir === null) throw new Usage("thiếu <thư-mục-bộ-mẫu>");
  let o;
  try { o = await openFrom(dir, force); } catch (e) { return [{ command: "open", ok: false, stage: "open", error: errJson(e) }, false]; }
  const { st, source } = o;
  const ms = st.migrationStatus();
  const out = { command: "open", ok: true, source, force_migration_failure: force, migration_status: ms, store: await storeJson(st) };
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
  st.close();
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
  try { await st.applyImport(vb, mode); } catch (e) { st.close(); return [{ ...base, ok: false, stage: "apply_import", before, error: errJson(e) }, false]; }
  const out = { ...base, ok: true, preview: vb.preview, verified_document: vb.doc, before, store: await storeJson(st) };
  st.close();
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
    st.close();
    return [{ command: "save-person", ok: true, source, migration_status: ms, saved }, true];
  } catch (e) {
    st.close();
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

const [cmd, ...rest] = process.argv.slice(2);
const table = { open: cmdOpen, import: cmdImport, "save-person": cmdSavePerson, suggestions: cmdSuggestions, "display-capitalize": cmdDisplay, "check-media": cmdCheckMedia };
try {
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
