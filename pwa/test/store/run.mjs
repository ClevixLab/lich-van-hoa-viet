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
const nowRfc3339T = () => new Date().toISOString().slice(0, 19) + "Z";
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

// ---------- YC-2026-05 / YC-2026-07: vector bổ sung thông tin báo tử, tệp đính kèm, giới tính (không thay vector đã khóa) ----------
{
  const SD = join(TV, "supplement-yc05");
  const sm = JSON.parse(readFileSync(join(SD, "manifest.json"), "utf8"));
  const nFile = new Uint8Array(readFileSync(join(SD, "notice-v1.lvhv")));
  const nText = readFileSync(join(SD, "notice-v1.json"), "utf8");
  const v = await S.openBackup(nFile, sm.password);
  ok(S.canonicalJson(v.doc) === nText, "YC-2026-05: notice-v1.lvhv (Rust) mở trong JS, JSON giống từng byte");
  const pdf = new Uint8Array(readFileSync(join(SD, "giay-chung-tu.pdf")));
  ok(sha(pdf) === sm.pdf.sha256, "YC-2026-07: giay-chung-tu.pdf đúng sha256 của manifest");
  // Khối: ảnh (theo photos) rồi tệp đính kèm (theo attachments) — 03 §2.3.
  const attBytes = { [sm.attachment_image_id]: exPhoto, [sm.attachment_pdf_id]: pdf };
  const blobs = [...v.doc.photos.map((ph) => ({ id: bc.uuidToBytes(ph.id), bytes: exPhoto })),
    ...v.doc.attachments.map((a) => ({ id: bc.uuidToBytes(a.id), bytes: attBytes[a.id] }))];
  const f = await bc.exportWithFixedRng(sm.password, nText, blobs, unhex(sm.salt_hex), unhex(sm.nonce_hex));
  ok(sha(f) === sm.file.sha256, "YC-2026-05: JS xuất notice-v1 với salt/nonce cố định = tệp của Rust (giống từng byte)");
  const t = await freshStore();
  await t.applyImport(v, "replace");
  ok((await t.stateHash()) === sm.state_sha256, "YC-2026-05: nhập vào IndexedDB → stateHash = state_hash(SQLite) của Rust");
  const pA = await t.person("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01");
  ok(pA.gioi_tinh === "nam" && pA.death.notice.attachments.map((a) => a.file_id).join() === `${sm.attachment_image_id},${sm.attachment_pdf_id}`,
    "YC-2026-05/07: báo tử, tệp đính kèm và giới tính được giữ");
  ok(sha(await t.attachmentBytes(sm.attachment_pdf_id)) === sm.pdf.sha256, "YC-2026-07: PDF đính kèm lưu nguyên byte");
  // Gỡ PDF khỏi danh sách → lớp lưu trữ xóa tệp (như Rust put_person).
  const pA2 = structuredClone(pA);
  pA2.death.notice.attachments.pop(); pA2.rev += 1;
  await t.putPerson(pA2);
  ok((await code(() => t.attachmentBytes(sm.attachment_pdf_id))) === "not_found" && (await t.attachments()).length === 1, "YC-2026-07: gỡ tệp khỏi danh sách → tệp bị xóa");
  ok((await code(() => t.deleteAttachment(sm.attachment_image_id))) === "input", "YC-2026-07: deleteAttachment tệp đang dùng → input");
  ok((await code(() => t.addAttachment({ id: "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e1f", mime: "application/pdf", file_name: null, bytes: new TextEncoder().encode("<html>") }))) === "input", "YC-2026-07: PDF thiếu %PDF- → từ chối");
  ok((await code(() => t.addAttachment({ id: "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e1f", mime: "image/jpeg", file_name: null, bytes: new Uint8Array(2 * 1024 * 1024 + 1) }))) === "invalid", "YC-2026-07: ảnh đính kèm > 2 MB → từ chối");
  const loose = await t.addAttachment({ id: "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e1f", mime: "application/pdf", file_name: "  Bản quét.pdf ", bytes: pdf });
  ok(loose.stripped === false && loose.file_name === "Bản quét.pdf", "YC-2026-07: PDF stripped = false, tên tệp được cắt khoảng trắng");
  await t.deleteAttachment(loose.id);
  ok((await code(() => t.deleteAttachment(loose.id))) === "not_found", "YC-2026-07: deleteAttachment tệp chưa gắn → xóa được");
  await t.deletePerson("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01");
  ok((await t.attachments()).length === 0 && !("attachments" in (await t.snapshot({ version: "0.2.0", platform: "pwa" }, "2026-01-01T00:00:00Z"))),
    "YC-2026-07: xóa hồ sơ → xóa tệp đính kèm; tài liệu không còn khóa attachments");
  const bad = structuredClone(v.doc);
  bad.attachments = bad.attachments.filter((x) => x.id !== sm.attachment_image_id);
  let got = "";
  try { S.validate.document(bad); } catch (e) { got = `${e.code} ${e.path}`; }
  ok(got === "V3 /persons/0/death/notice/attachments/0/file_id", "YC-2026-07: tệp đính kèm không tồn tại → V3 (cùng đường dẫn với Rust)");
  for (const [k, val] of [["stripped", true], ["mime", "application/zip"], ["bytes", 10 * 1024 * 1024 + 1], ["file_name", ""]]) {
    const b2 = structuredClone(v.doc); b2.attachments[1][k] = val;
    let threw = false; try { S.validate.document(b2); } catch { threw = true; }
    ok(threw, `YC-2026-07: Attachment.${k} sai → từ chối`);
  }
  // PR #30: "Gộp" tệp có tệp đính kèm vào kho mới → giữ tệp đính kèm, tài liệu hợp lệ; state hash = Rust.
  const tm = await freshStore();
  await tm.applyImport(v, "merge");
  ok((await tm.attachments()).length === 2 && sha(await tm.attachmentBytes(sm.attachment_pdf_id)) === sm.pdf.sha256, "PR #30: Gộp giữ tệp đính kèm của bản nhập");
  const hm = await tm.stateHash();
  const rm = JSON.parse(readFileSync(join(OUT, "notice-merge-rust.json"), "utf8"));
  ok(hm === rm.state_sha256, "PR #30: Gộp tệp có tệp đính kèm — stateHash = Rust (notice-merge-rust.json)");
  await tm.applyImport(v, "merge");
  ok((await tm.stateHash()) === hm, "PR #30: Gộp lần hai cùng tệp → không đổi");
  // Tệp sao lưu sửa: khối PDF thiếu (chỉ còn ảnh) → từ chối.
  let rej = "";
  try { await S.verifyContents(nText, blobs.slice(0, -1).map((b) => ({ id: bc.bytesToUuid(b.id), bytes: b.bytes }))); } catch (e) { rej = e.message; }
  ok(rej === "Sai mật khẩu hoặc tệp bị hỏng", "YC-2026-07: thiếu khối tệp đính kèm → thông báo chung");
  // "notice": null ≡ vắng khóa (Rust: serde default + skip_serializing_if) → PWA bỏ khóa khi lưu, stateHash như Rust.
  const withNull = structuredClone(v.doc);
  const pB = withNull.persons.find((x) => x.death !== null && !("notice" in x.death));
  pB.death.notice = null;
  S.validate.document(withNull);
  const t2 = await freshStore();
  await t2.applyImport({ ...v, doc: withNull }, "replace");
  ok((await t2.stateHash()) === sm.state_sha256, "YC-2026-05: tài liệu có \"notice\": null → lưu không có khóa, stateHash = Rust");
  const stored = await t2.person(pB.id);
  ok(!("notice" in stored.death), "YC-2026-05: putPerson/applyImport bỏ \"notice\": null (như Rust skip_serializing_if)");
  await t2.putPerson(structuredClone({ ...stored, death: { ...stored.death, notice: null } }));
  ok(!("notice" in (await t2.person(pB.id)).death) && (await t2.stateHash()) === sm.state_sha256, "YC-2026-05: putPerson với \"notice\": null → không ghi khóa");
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
// 04 Bổ sung B7: R6 (a)–(d) và lật bit chạy trên bộ R6v2 (ảnh JPEG/WebP thật, đã khóa). Bộ R6 cũ nay bị từ chối khi
// nhập (ảnh giả) — kiểm ở cuối khối này.
const R6V2 = join(TV, "r6v2");
const R6V2M = JSON.parse(readFileSync(join(R6V2, "manifest.json"), "utf8"));
let r6v2Frozen = true;
for (const [f, h] of Object.entries(R6V2M.files)) if (sha(readFileSync(join(R6V2, f))) !== h) r6v2Frozen = false;
ok(r6v2Frozen && Object.keys(R6V2M.files).length === 23, "B7: bộ R6v2 đúng sha256 đã khóa (23 tệp)");
const dsText = readFileSync(join(R6V2, "dataset-r6v2.json"), "utf8");
const ds = JSON.parse(dsText);
const fileOf = new Map(R6V2M.photos.map((p) => [p.id, p.file]));
const dsPhotos = ds.photos.map((p) => ({ id: p.id, bytes: new Uint8Array(readFileSync(join(R6V2, fileOf.get(p.id)))) }));
ok(dsPhotos.every((p, i) => sha(p.bytes) === ds.photos[i].sha256 && S.mediaVerdict("photo", ds.photos[i].mime, p.bytes, ds.photos[i].width, ds.photos[i].height) === "ok"),
  "bộ R6v2: 20 ảnh thật khớp sha256 và qua kiểm tra chặt B6");
{
  // Bộ R6 cũ (đã khóa, ảnh giả) → bị từ chối khi nhập, thông báo chung (04 B7 mục 2).
  const oldText = readFileSync(join(TV, "dataset-r6.json"), "utf8");
  const old = JSON.parse(oldText);
  const bySha = new Map(Array.from({ length: 20 }, (_, i) => r6PhotoBytes(i)).map((b) => [sha(b), b]));
  const oldPhotos = old.photos.map((p) => ({ id: p.id, bytes: bySha.get(p.sha256) }));
  ok(oldPhotos.every((p) => p.bytes), "bộ R6 cũ: 20 ảnh tất định khớp sha256 (vector đã khóa không đổi)");
  const fx = await bc.exportWithFixedRng(PW, oldText, oldPhotos.map((p) => ({ id: bc.uuidToBytes(p.id), bytes: p.bytes })), SALT, NONCE);
  ok(sha(fx) === manifest.dataset_r6.fixed_rng_file_sha256, "bộ R6 cũ: tệp salt/nonce cố định JS = sha256 đã khóa");
  let m1 = "", m2 = "";
  try { await S.verifyContents(oldText, oldPhotos); } catch (e) { m1 = e.message; }
  try { await S.openBackup(fx, PW); } catch (e) { m2 = e.message; }
  ok(m1 === "Sai mật khẩu hoặc tệp bị hỏng" && m2 === "Sai mật khẩu hoặc tệp bị hỏng", "B7: bộ R6 cũ (ảnh giả) bị TỪ CHỐI khi nhập — thông báo chung");
}

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
  ok(S.canonicalJson(snap) === dsText, "bộ R6v2 nạp vào IndexedDB, ảnh chụp JSON = dataset-r6v2.json (giống từng byte)");
  const h1 = await s.stateHash();
  ok(h1 === R6V2M.store_state_sha256, "stateHash(IndexedDB) = state_hash(SQLite) của Rust (R6v2)");
  const fixed = await bc.exportWithFixedRng(PW, dsText, dsPhotos.map((p) => ({ id: bc.uuidToBytes(p.id), bytes: p.bytes })), SALT, NONCE);
  ok(sha(fixed) === R6V2M.fixed_rng_file_sha256 && sha(readFileSync(join(R6V2, "dataset-r6v2-fixed.lvhv"))) === R6V2M.fixed_rng_file_sha256,
    "tệp R6v2 salt/nonce cố định: JS = Rust = tệp đã khóa (giống từng byte)");
  const lastBefore = (await s.settings()).last_backup_at;
  const t0 = performance.now();
  r6File = await s.exportBackup(PW, { version: "0.2.0", platform: "pwa" });
  const exportMs = performance.now() - t0;
  ok((await s.settings()).last_backup_at === lastBefore && (await s.stateHash()) === h1, "MA3N-23-01: tạo byte sao lưu không đổi last_backup_at (chưa ghi nhận là đã sao lưu), stateHash không đổi");
  const savedAt = nowRfc3339T();
  await s.markBackupSaved(savedAt);
  ok((await s.backupPendingSince()) === null && (await s.settings()).last_backup_at === savedAt, "sau markBackupSaved: last_backup_at = lúc lưu, hết trạng thái chờ sao lưu");
  let twice = false;
  try { await s.markBackupSaved(nowRfc3339T()); } catch { twice = true; }
  ok(twice, "markBackupSaved hai lần cho một lần tạo tệp → lỗi");
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
  results.r6_d = { rust_to_pwa: (await u.stateHash()) === h1, pwa_file_for_rust: "pwa/test/store/out/r6-from-pwa.lvhv", fixed_rng_bytes_equal: sha(fixed) === R6V2M.fixed_rng_file_sha256, dataset: "r6v2" };

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
  // MA3N-24-08: deleteOrphanEvents xóa sự kiện chỉ gắn người đó, giữ sự kiện vốn không gắn ai và sự kiện còn người khác.
  {
    const q = S.buildPerson({ ...ex.persons[1], id: "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e71" });
    const r = S.buildPerson({ ...ex.persons[1], id: "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e72", ho_ten: "Lê Văn D" });
    await s.putPerson(q); await s.putPerson(r);
    const ev = (id, rid, pids) => ({ ...ex.events[0], id, reminder: { ...ex.events[0].reminder, id: rid }, person_ids: pids });
    await s.putEvent(ev("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e81", "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e91", [q.id]));
    await s.putEvent(ev("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e82", "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e92", [q.id, r.id]));
    const removed = await s.deletePerson(q.id, { deleteOrphanEvents: true });
    const left = (await s.events()).map((e) => e.id).sort();
    ok(JSON.stringify(removed) === JSON.stringify(["0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e81"]), "deleteOrphanEvents → trả id sự kiện chỉ của người đó");
    ok(JSON.stringify(left) === JSON.stringify([ex.events[0].id, "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e82"].sort()), "giữ sự kiện không gắn ai và sự kiện còn người khác");
    await s.deletePerson(r.id);
  }
  let bad = p; bad = { ...bad, death: { ...bad.death, lunar: { ...bad.death.lunar, day: 11 } } };
  ok((await code(() => s.putPerson(bad))) === "invalid", "V5: trường dẫn xuất sai → từ chối");
  results.store_api = "ok";
}

// Dọn ảnh không còn dùng — cùng kịch bản `photo_gc` của store/examples/cross_platform.rs, so với kết quả Rust.
{
  const s = await seeded();
  const ps = (await s.persons()).filter((p) => typeof p.photo_id === "string").sort((x, y) => (x.id < y.id ? -1 : 1));
  const [a, b] = [structuredClone(ps[0]), structuredClone(ps[1])];
  const [pa, pb] = [a.photo_id, b.photo_id];
  const bump = (p) => { p.rev += 1; p.updated_at = "2026-11-07T00:00:00Z"; };
  a.photo_id = pb; bump(a); await s.putPerson(a);
  const after1 = (await s.photos()).length;
  b.photo_id = null; bump(b); await s.putPerson(b);
  const after2 = (await s.photos()).length;
  const got = {
    delete_used: await code(() => s.deletePhoto(pb)),
    delete_gone: await code(() => s.deletePhoto(pa)),
    photos_after: [after1, after2],
    state_sha256: await s.stateHash(),
  };
  const rust = JSON.parse(readFileSync(join(OUT, "photo-gc-rust.json"), "utf8"));
  ok(JSON.stringify(got.photos_after) === "[19,19]" && got.delete_used === "input" && got.delete_gone === "not_found",
    `dọn ảnh: đổi ảnh xóa ảnh cũ, ảnh còn hồ sơ khác dùng thì giữ; deletePhoto đang dùng → input, đã xóa → not_found (${JSON.stringify(got)})`);
  ok(JSON.stringify(got, Object.keys(got).sort()) === JSON.stringify(rust, Object.keys(rust).sort()), "dọn ảnh: kết quả và stateHash giống hệt Rust (photo-gc-rust.json)");
  results.photo_gc = got;
}

// YC-2026-10 (04 Bổ sung B8, R15 (c)–(f)): ảnh mâm lễ `memorial.tray_photo_id` — như Rust store/tests/tray.rs.
{
  const T1 = "0b8e4f52-3c1a-4d2e-9f60-0000000000c1", T2 = "0b8e4f52-3c1a-4d2e-9f60-0000000000c2";
  const D1 = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01", D2 = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02", L3 = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e03";
  const PW = manifest.password;
  const exStore = async () => { const s = await freshStore(); await s.applyImport(await S.openBackup(new Uint8Array(exFile), PW), "replace"); return s; };
  const photoFile = (n) => new Uint8Array(readFileSync(join(TV, "r6v2", `photo-${String(n).padStart(2, "0")}.jpg`)));
  const add = (s, id, n) => s.addPhoto({ id, mime: "image/jpeg", width: 300 + n, height: 400 + n, bytes: photoFile(n) });
  const bump = (p) => { p.rev += 1; p.updated_at = "2026-11-07T00:00:00Z"; };
  const withTray = async (s, id, tray) => { const p = structuredClone(await s.person(id)); p.memorial.tray_photo_id = tray; bump(p); return s.putPerson(S.normalizePersonNames(p)); };
  const v3 = async (fn) => { try { await fn(); return "OK"; } catch (e) { return / (V\d \/\S+?):/.exec(" " + e.message)?.[1] ?? e.message; } };
  const orphans = async (s) => {
    const ids = (await s.photos()).map((p) => p.id);
    const used = new Set((await s.persons()).flatMap((p) => S.validate.personPhotoRefs(p)));
    return ids.filter((i) => !used.has(i)).length;
  };
  // (e) R14: tệp của bản trước mở được, không đổi; tệp xuất lại không có khóa mới.
  const s = await exStore();
  const ex = JSON.parse(exJsonText);
  const snap = await s.snapshot({ version: "0.2.0", platform: "pwa" }, "x");
  ok(["persons", "photos", "events", "event_types"].every((k) => S.canonicalJson(snap[k]) === S.canonicalJson(ex[k])) && !S.canonicalJson(snap).includes("tray_photo_id"),
    "R15 (e): example-v1.lvhv (không có tray_photo_id) mở được, kho = tệp, không có khóa mới");
  ok(!("tray_photo_id" in S.normalizePersonNames({ memorial: { tray_photo_id: null } }).memorial), "R15 (e): tray_photo_id null → bỏ khóa khi lưu (như Rust)");
  // V3.
  await add(s, T1, 0);
  await withTray(s, D1, T1);
  const portrait = (await s.person(D1)).photo_id;
  const l = structuredClone(await s.person(L3)); l.photo_id = T1; bump(l);
  const v3s = [await v3(() => withTray(s, D2, "0b8e4f52-3c1a-4d2e-9f60-0000000000ee")), await v3(() => withTray(s, D2, T1)), await v3(() => withTray(s, D2, portrait)),
    await v3(() => withTray(s, D1, portrait)), await v3(() => s.putPerson(l))];
  ok((await s.person(D1)).memorial.tray_photo_id === T1 && JSON.stringify(v3s) === JSON.stringify(["V3 /person/memorial/tray_photo_id", "V3 /person/memorial/tray_photo_id",
    "V3 /person/memorial/tray_photo_id", "V3 /person/memorial/tray_photo_id", "V3 /person/photo_id"]), `R15 V3: ảnh không có / dùng chung → từ chối (như Rust) ${JSON.stringify(v3s)}`);
  const doc = structuredClone(ex); doc.persons[1].memorial.tray_photo_id = T1;
  const dv = await v3(() => S.validate.document(doc));
  doc.persons[1].memorial.tray_photo_id = portrait;
  const ds = await v3(() => S.validate.document(doc));
  ok(dv === "V3 /persons/1/memorial/tray_photo_id" && ds === "V3 /persons/0/photo_id", `R15 (d): tài liệu nhập có tray_photo_id treo / dùng chung → V3 (như Rust) ${dv}; ${ds}`);
  // (d) xuất / nhập.
  const sha1 = sha(await s.photoBytes(T1));
  const f = await s.exportBackup(PW, { version: "0.2.0", platform: "pwa" });
  let rt = true;
  for (const [mode, seed] of [["replace", false], ["merge", false], ["merge", true]]) {
    const u = seed ? await exStore() : await freshStore();
    await u.applyImport(await S.openBackup(f, PW), mode);
    rt &&= (await u.person(D1)).memorial.tray_photo_id === T1 && sha(await u.photoBytes(T1)) === sha1 && (await orphans(u)) === 0;
    u.close();
  }
  ok(rt, "R15 (d): xuất rồi nhập (Thay toàn bộ, Gộp vào kho trống, Gộp vào kho có bản cũ) giữ ảnh mâm lễ (sha256) và liên kết");
  // (f) đổi, bỏ, đổi về đang sống, xóa hồ sơ → ảnh cũ bị xóa; 0 ảnh mồ côi.
  const gone = async (id) => (await code(() => s.photoBytes(id))) !== "OK";
  await add(s, T2, 2); await withTray(s, D1, T2);
  const f1 = (await gone(T1)) && (await orphans(s)) === 0;
  await withTray(s, D1, null);
  const f2 = (await gone(T2)) && !("tray_photo_id" in (await s.person(D1)).memorial) && (await orphans(s)) === 0;
  await add(s, T1, 0); await withTray(s, D1, T1);
  const p = structuredClone(await s.person(D1));
  Object.assign(p, { kind: "living", death: null, memorial: null, noi_an_tang: "" }); p.reminders.gio = null; bump(p);
  await s.putPerson(p);
  const f3 = (await gone(T1)) && (await orphans(s)) === 0;
  await add(s, T2, 2); await withTray(s, D2, T2); await s.deletePerson(D2);
  const f4 = (await gone(T2)) && (await orphans(s)) === 0;
  ok(f1 && f2 && f3 && f4, `R15 (f): đổi / bỏ / đổi về đang sống / xóa hồ sơ → ảnh mâm lễ cũ bị xóa cùng giao dịch, 0 ảnh mồ côi (${[f1, f2, f3, f4].join(",")})`);
  s.close();
  // 01 §6.3: Gộp giữ ảnh mâm lễ của bản nhập mà hồ sơ sau gộp còn dùng.
  const inc = structuredClone(ex);
  inc.photos.push({ id: T1, mime: "image/jpeg", width: 300, height: 400, bytes: 10, sha256: "a".repeat(64), stripped: true, created_at: "2026-11-07T00:00:00Z" });
  inc.persons[1].memorial.tray_photo_id = T1; inc.persons[1].rev = 9;
  const m1 = S.mergeDocs(ex, inc);
  const loc2 = structuredClone(ex); loc2.persons[1].rev = 99;
  const m2 = S.mergeDocs(loc2, inc);
  ok(m1.photos.some((x) => x.id === T1) && m1.persons[1].memorial.tray_photo_id === T1 && !m2.photos.some((x) => x.id === T1),
    "Gộp giữ ảnh mâm lễ của bản nhập khi hồ sơ nhập thắng; bỏ khi hồ sơ trên máy thắng (như Rust merge_docs)");
  // Duyệt mã MA10-02..06, báo cáo kiểm định R15 F2 — như Rust store/tests/tray.rs.
  {
    const st = await exStore();
    const big = new Uint8Array(readFileSync(join(TV, "tray-photo", "tray-1601x1200.jpg")));
    await st.addPhoto({ id: T1, mime: "image/jpeg", width: 1601, height: 1200, bytes: big });
    const e1601 = await (async () => { try { await withTray(st, D2, T1); return null; } catch (e) { return e.message; } })();
    const l3 = structuredClone(await st.person(L3)); l3.photo_id = T1; bump(l3);
    const portraitOk = await code(() => st.putPerson(l3));
    const doc2 = structuredClone(ex);
    doc2.photos.push({ id: T2, mime: "image/jpeg", width: 1200, height: 1601, bytes: 10, sha256: "a".repeat(64), stripped: true, created_at: "2026-11-07T00:00:00Z" });
    doc2.persons[1].memorial.tray_photo_id = T2;
    let dErr = null; try { S.validate.document(doc2); } catch (e) { dErr = `${e.code} ${e.path}`; }
    doc2.photos[1].height = 1600;
    const d1600 = await code(async () => S.validate.document(doc2));
    ok(e1601 === "V1 /person/memorial/tray_photo_id: ảnh mâm lễ có cạnh dài quá 1600 px" && portraitOk === "OK" && dErr === "V1 /persons/1/memorial/tray_photo_id" && d1600 === "OK",
      `MA10-02: ảnh mâm lễ > 1600 px → V1 khi lưu và khi nhập; ảnh chân dung 1601 px vẫn được (${[e1601, portraitOk, dErr, d1600].join(" | ")})`);
    // MA10-04, MA10-05.
    const dd = structuredClone(await st.person(D2)); dd.photo_id = "0b8e4f52-3c1a-4d2e-9f60-0000000000ee"; dd.memorial.tray_photo_id = dd.photo_id; bump(dd);
    const eDouble = await (async () => { try { await st.putPerson(dd); return null; } catch (e) { return e.message; } })();
    const portrait = (await st.person(D1)).photo_id;
    const eOwn = await (async () => { try { await withTray(st, D1, portrait); return null; } catch (e) { return e.message; } })();
    ok(eDouble === "V3 /person/photo_id: ảnh không tồn tại" && eOwn === "V3 /person/memorial/tray_photo_id: ảnh mâm lễ không được trùng ảnh chân dung",
      `MA10-04/05: lỗi kép báo ảnh chân dung treo trước (như Rust); trùng ảnh chân dung của chính mình có câu riêng (${eDouble} | ${eOwn})`);
    // MA10-06, F2: null ghi thẳng / null trong tệp nhập → không ghi khóa.
    const pn = structuredClone(await st.person(D2)); pn.memorial.tray_photo_id = null; bump(pn);
    await st.putPerson(pn);
    const direct = !("tray_photo_id" in (await st.person(D2)).memorial);
    const nd = structuredClone(ex); nd.persons[1].memorial.tray_photo_id = null; nd.persons[1].rev = 60;
    const vb = await S.verifyContents(S.canonicalJson(nd), [{ id: ex.photos[0].id, bytes: new Uint8Array(readFileSync(join(TV, ex.photos[0].id.replace(/^/, "photo-") + ".jpg"))) }]);
    let viaImport = true;
    for (const mode of ["replace", "merge"]) {
      await st.applyImport(vb, mode);
      viaImport &&= !S.canonicalJson(await st.snapshot({ version: "0.2.0", platform: "pwa" }, "x")).includes("tray_photo_id");
    }
    ok(direct && viaImport, `MA10-06 / F2: tray_photo_id null (putPerson trực tiếp, tệp nhập Thay toàn bộ và Gộp) → không ghi khóa (${direct},${viaImport})`);
    st.close();
    // MA10-03: Gộp bỏ ảnh trên máy không còn hồ sơ nào dùng.
    const l2 = structuredClone(ex);
    l2.photos.push({ id: T2, mime: "image/jpeg", width: 300, height: 400, bytes: 10, sha256: "b".repeat(64), stripped: true, created_at: "2026-11-07T00:00:00Z" });
    l2.persons[1].memorial.tray_photo_id = T2;
    const inc2 = structuredClone(ex); inc2.persons[1].rev = 9;
    ok(!S.mergeDocs(l2, inc2).photos.some((x) => x.id === T2), "MA10-03: Gộp — ảnh mâm lễ trên máy mà hồ sơ thắng (bản nhập) không dùng bị bỏ (như Rust merge_docs)");
  }
  results.tray_photo = "ok";
}

// YC-2026-07: gợi ý cách gọi / quan hệ và viết hoa chữ đầu — giống hệt Rust (names-rust.json do cross_platform write ghi).
{
  const rust = JSON.parse(readFileSync(join(OUT, "names-rust.json"), "utf8"));
  const dx = {};
  for (const [k, dead] of [["deceased", true], ["living", false]]) for (const g of [null, "nam", "nu"]) dx[`${k}:${g ?? "null"}`] = S.danhXungSuggestions(dead, g);
  const mine = { capitalize: rust.capitalize.map(([x]) => [x, S.capitalizeFirst(x)]), danh_xung: dx, quan_he: { deceased: S.quanHeSuggestions(true), living: S.quanHeSuggestions(false) } };
  ok(S.canonicalJson(mine) === S.canonicalJson(rust), "YC-2026-07: gợi ý cách gọi, quan hệ và viết hoa chữ đầu giống hệt Rust");
  const np = S.normalizePersonNames({ danh_xung: "ông", quan_he: "bà ngoại", gioi_tinh: null, ho_ten: "x" });
  ok(np.danh_xung === "Ông" && np.quan_he === "Bà ngoại" && !("gioi_tinh" in np), "YC-2026-07: normalizePersonNames (viết hoa, gioi_tinh null → bỏ khóa)");
  const nt = S.normalizePersonNames({ danh_xung: "  chú  ", quan_he: "   ", ho_ten: " Lê Văn C ", ten_hieu: "\t", birth: { noi_sinh: " Hà Nội " }, memorial: null, gioi_tinh: null });
  ok(S.canonicalJson(nt) === S.canonicalJson({ danh_xung: "Chú", quan_he: "", ho_ten: "Lê Văn C", ten_hieu: "", birth: { noi_sinh: "Hà Nội" }, memorial: null })
    && S.displayCapitalize("  chú  ") === "Chú" && S.displayCapitalize("   ") === "",
    "Kiểm định vòng 3 (01 §1): cắt khoảng trắng rồi viết hoa; chỉ còn khoảng trắng → \"\" (như Rust normalize_person_names)");
  results.names = { same_as_rust: S.canonicalJson(mine) === S.canonicalJson(rust) };
}

// ---------- 04 B5 / R14: bộ mẫu đóng băng released/0.2-dev-before-attachments ----------
// Dựng kho IndexedDB phiên bản 1 (như bản dựng 0.2 trước YC-2026-07) từ ảnh chụp kho + byte ảnh của bộ mẫu, mở bằng
// bản này → chuyển trong onupgradeneeded → so với cùng tệp kỳ vọng mà bản Rust phải ra.
{
  const RD = join(TV, "released", "0.2-dev-before-attachments");
  const rm = JSON.parse(readFileSync(join(RD, "manifest.json"), "utf8"));
  let frozen = true;
  for (const [f, h] of Object.entries(rm.files)) if (sha(readFileSync(join(RD, f))) !== h) frozen = false;
  ok(frozen, "B5: bộ mẫu đóng băng 0.2-dev-before-attachments không bị sửa (sha256 = manifest)");
  const snapText = readFileSync(join(RD, "store-snapshot.json"), "utf8");
  const old = JSON.parse(snapText);
  const exp = JSON.parse(readFileSync(join(TV, "released", "expected", "0.2-dev-before-attachments.json"), "utf8"));
  const { IDBFactory } = await import("fake-indexeddb");
  const v1Store = async (mutate) => {
    const f = new IDBFactory();
    const doc = structuredClone(old);
    if (mutate) mutate(doc);
    await new Promise((okk, ko) => {
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
        for (const ph of doc.photos) tx.objectStore("photos").put({ id: ph.id, meta: ph, blob: new Blob([readFileSync(join(RD, "store", "photos", `${ph.id}.img`))], { type: ph.mime }) });
      };
      r.onsuccess = () => { r.result.close(); okk(); };
      r.onerror = () => ko(r.error);
    });
    return f;
  };
  const stateDoc = async (st) => { const d = await st.snapshot({ version: "0.0.0", platform: "pwa" }, "1970-01-01T00:00:00Z"); delete d.app; delete d.exported_at; return d; };
  // Chuyển kho.
  const f1 = await v1Store();
  const st = await S.openStore(f1);
  const ms = st.migrationStatus();
  ok(ms.state === "migrated" && ms.from_version === 1 && ms.to_version === 3, `B5: kho phiên bản 1 → chuyển đổi (${JSON.stringify(ms)})`);
  const got = await stateDoc(st);
  ok(S.canonicalJson(got) === S.canonicalJson(exp.state), "B5/R14: kho IndexedDB sau chuyển đổi = tệp kỳ vọng (giống bản Rust, 0 trường khác)");
  ok((await st.stateHash()) === exp.state_sha256, "B5/R14: stateHash sau chuyển đổi = state_sha256 của Rust");
  let bytesOk = true;
  for (const ph of old.photos) {
    const orig = sha(readFileSync(join(RD, "store", "photos", `${ph.id}.img`)));
    const b = await st.photoBytes(ph.id).catch(() => st.attachmentBytes(ph.id));
    if (sha(b) !== orig) bytesOk = false;
  }
  for (const a of exp.state.attachments) if (sha(await st.attachmentBytes(a.id)) !== a.sha256) bytesOk = false;
  ok(bytesOk, "B5/R14: mọi byte ảnh cũ còn nguyên (ảnh chân dung, tệp đính kèm chuyển từ ảnh giấy báo tử)");
  st.close();
  const st2 = await S.openStore(f1);
  ok(st2.migrationStatus().state === "ok" && (await st2.stateHash()) === exp.state_sha256, "B5: mở lại → không chuyển nữa, không đổi");
  st2.close();
  // Tệp sao lưu của bản dựng cũ.
  const ov = await S.openBackup(new Uint8Array(readFileSync(join(RD, "backup.lvhv"))), rm.password);
  const ti = await freshStore();
  await ti.applyImport(ov, "replace");
  ok((await ti.stateHash()) === exp.state_sha256, "B5 mục 5: tệp sao lưu của bản dựng cũ nhập được → cùng trạng thái với Rust");
  // Ca lỗi: dữ liệu hỏng → quay lui, chỉ đọc, dữ liệu cũ còn nguyên, vẫn xuất được.
  const fbad = await v1Store((d) => { d.persons.find((p) => p.id === "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02").death.notice.place = "  Quê nhà"; });
  const sb = await S.openStore(fbad);
  const bs = sb.migrationStatus();
  ok(bs.state === "read_only" && bs.to_version === 1 && typeof bs.error === "string", `B5: chuyển đổi lỗi → chỉ đọc (${JSON.stringify(bs)})`);
  ok(sb.db.version === 1 && (await sb.attachments()).length === 0 && (await sb.person("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02")).death.notice.certificate_photo_ids.length === 2,
    "B5: giao dịch nâng cấp quay lui — kho phiên bản 1 và dữ liệu cũ còn nguyên");
  ok((await code(() => sb.putPerson(old.persons[0]))) === "read_only" && (await code(() => sb.deletePerson(old.persons[0].id))) === "read_only", "B5: chỉ đọc → mọi thao tác ghi báo read_only");
  const xf = await sb.exportBackup("mật khẩu thử nghiệm", { version: "0.2.0", platform: "pwa" });
  ok(xf.length > 1000, "B5: chỉ đọc vẫn xuất được tệp sao lưu");
  sb.close();
  // Hàm thuần: cùng kết quả với Rust trên ca biên (ảnh chân dung dùng chung, id treo → V3, hai khóa → V1, chạy lại
  // không đổi) — như store/src/migrate.rs shared_portrait_missing_and_noop.
  const edge = {
    photos: [
      { id: "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01", mime: "image/jpeg", width: 10, height: 20, bytes: 5, sha256: "a".repeat(64), stripped: true, created_at: "2026-01-01T00:00:00Z" },
      { id: "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02", mime: "image/webp", width: 10, height: 20, bytes: 6, sha256: "b".repeat(64), stripped: true, created_at: "2026-01-02T00:00:00Z" }],
    persons: [
      { id: "p1", photo_id: "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01", death: { notice: { certificate_photo_ids: ["0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01", "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02", "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5eff"] } } },
      { id: "p2", photo_id: null, death: { notice: { certificate_photo_ids: ["0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02"] } } },
      { id: "p3", photo_id: null, death: null }],
  };
  const err = (d) => { try { S.migrate1to2(d); return "OK"; } catch (e) { return `${e.code} ${e.path}`; } };
  ok(err(edge) === "V3 /persons/0/death/notice/certificate_photo_ids/2", "M09: certificate_photo_ids trỏ tới ảnh không có → V3 (như Rust)");
  edge.persons[0].death.notice.certificate_photo_ids.pop();
  const both = structuredClone(edge); both.persons[1].death.notice.attachments = [];
  ok(err(both) === "V1 /persons/1/death/notice", "X22: có cả attachments lẫn certificate_photo_ids → V1 (như Rust)");
  const pure = S.migrate1to2(edge);
  ok(pure.doc.photos.length === 1 && pure.doc.attachments.length === 2 && pure.doc.persons[0].legacy.certificate_photo_ids.length === 2
    && pure.doc.persons[0].legacy.certificate_photos[0].width === 10 && pure.doc.persons[2].death === null
    && S.altId("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e0c", new Set()) === "0b8e4f52-3c1a-4d2e-9f60-e5d4c3b2a1f3"
    && !S.migrate1to2(pure.doc).changed, "B5: migrate1to2 cùng ca biên với Rust (store/src/migrate.rs)");
  // BM2-07 (01 §7): 7 ngày sau chuyển đổi thành công bỏ `legacy`; tệp xuất trong 7 ngày không mang `legacy` — như Rust
  // legacy_and_backup_pruned_after_7_days.
  {
    const fp = await v1Store();
    const iso = (ms) => new Date(ms).toISOString().slice(0, 19) + "Z";
    const t0 = Date.parse("2026-11-10T00:00:00Z"), day = 86_400_000;
    const nLegacy = async (st) => (await st.persons()).filter((p) => "legacy" in p).length;
    const a0 = await S.openStore(fp, { now: iso(t0) });
    const c0 = a0.migrationStatus().state === "migrated" && (await nLegacy(a0)) === 2;
    const xf = await a0.exportBackup("mật khẩu thử nghiệm", { version: "0.2.0", platform: "pwa" });
    const xv = await S.openBackup(xf, "mật khẩu thử nghiệm");
    const noLegacyInFile = xv.doc.persons.every((p) => !("legacy" in p));
    a0.close();
    const a1 = await S.openStore(fp, { now: iso(t0 + 7 * day - 3_600_000) });
    const c1 = (await nLegacy(a1)) === 2;
    a1.close();
    const a2 = await S.openStore(fp, { now: iso(t0 + 7 * day) });
    const pruned = (await stateDoc(a2));
    const expPersons = exp.state.persons.map((p) => { const { legacy: _l, ...r } = p; void _l; return r; });
    const c2 = (await nLegacy(a2)) === 0 && S.canonicalJson(pruned.persons) === S.canonicalJson(expPersons)
      && S.canonicalJson(pruned.attachments) === S.canonicalJson(exp.state.attachments);
    a2.close();
    ok(c0 && noLegacyInFile && c1 && c2, `BM2-07: legacy giữ 7 ngày rồi bị bỏ (như Rust); tệp xuất không mang legacy (${[c0, noLegacyInFile, c1, c2].join(",")})`);
  }
  // Rà soát bảo mật vòng 2, BM3-01 (như Rust moved_certificate_photos_removed_after_7_days_and_on_delete): bản web
  // chuyển hẳn byte ảnh giấy báo tử sang kho `attachments` và xóa ảnh gốc trong CÙNG giao dịch nâng cấp (quay lui do
  // IndexedDB lo) — không còn ảnh nào không có hồ sơ dùng; xóa mọi hồ sơ → kho ảnh và tệp đính kèm rỗng.
  {
    const fm = await v1Store();
    const sm = await S.openStore(fm);
    const ids = new Set((await sm.photos()).map((p) => p.id));
    const used = new Set((await sm.persons()).flatMap((p) => S.validate.personPhotoRefs(p)));
    const movedGone = old.photos.filter((ph) => !used.has(ph.id)).every((ph) => !ids.has(ph.id));
    const noOrphan = [...ids].every((id) => used.has(id));
    for (const p of await sm.persons()) await sm.deletePerson(p.id);
    const empty = (await sm.photos()).length === 0 && (await sm.attachments()).length === 0;
    sm.close();
    ok(movedGone && noOrphan && empty, `BM3-01: sau chuyển đổi không còn ảnh gốc mồ côi; xóa mọi hồ sơ → 0 ảnh, 0 tệp đính kèm (${[movedGone, noOrphan, empty].join(",")})`);
  }
  // putUiMeta (PR #30, MA3C-30-04) chỉ ghi khóa giao diện; khóa của lõi trong `meta` không bị giao diện ghi đè.
  {
    const su = await freshStore();
    const okUi = (await su.putUiMeta("ui_restored_from", "2026-11-07T00:00:00Z")) === true && (await su.putUiMeta("persist", true)) === true;
    const denied = await Promise.all(["migrated_at", "migration_moved_photos", "store_version", "schema_version", "ui_", "UI_X"].map((k) => code(() => su.putUiMeta(k, "x"))));
    su.close();
    ok(okUi && denied.every((c) => c === "input"), `putUiMeta: khóa ui_*/persist ghi được; khóa của lõi → input (${denied.join(",")})`);
  }
  // Kiểm định: buộc lỗi chuyển đổi → chỉ đọc, dữ liệu cũ nguyên.
  const ff = await v1Store();
  const sf = await S.openStore(ff, { forceMigrationFailure: true });
  ok(sf.migrationStatus().state === "read_only" && sf.db.version === 1, "B5: forceMigrationFailure → quay lui, chỉ đọc (như Rust OpenOptions)");
  sf.close();
  // 04 B6 trong chuyển đổi: ảnh giấy báo tử của kho cũ còn EXIF → không chuyển, chỉ đọc, dữ liệu cũ nguyên.
  const exifJpg = readFileSync(join(TV, "media", "jpeg-exif-gps.jpg"));
  const fx = new (await import("fake-indexeddb")).IDBFactory();
  {
    const doc = structuredClone(old);
    await new Promise((okk, ko) => {
      const r = fx.open("lvhv", 1);
      r.onupgradeneeded = () => {
        const db = r.result, tx = r.transaction;
        for (const n of ["photos", "persons", "events"]) db.createObjectStore(n, { keyPath: "id" });
        db.createObjectStore("event_types", { keyPath: "id" }).createIndex("by_name_key", "name_key", { unique: true });
        db.createObjectStore("settings"); db.createObjectStore("meta");
        tx.objectStore("settings").put(doc.settings, "app");
        for (const t of doc.event_types) tx.objectStore("event_types").put({ id: t.id, name_key: S.validate.nameKey(t.ten), doc: t });
        for (const p of doc.persons) tx.objectStore("persons").put(p);
        for (const ph of doc.photos) tx.objectStore("photos").put({ id: ph.id, meta: ph, blob: new Blob([ph.id.endsWith("5e0c") ? exifJpg : readFileSync(join(RD, "store", "photos", `${ph.id}.img`))]) });
      };
      r.onsuccess = () => { r.result.close(); okk(); };
      r.onerror = () => ko(r.error);
    });
  }
  const sx = await S.openStore(fx);
  ok(sx.migrationStatus().state === "read_only" && /metadata/.test(sx.migrationStatus().error ?? ""), `B6: ảnh giấy báo tử còn EXIF → không chuyển, chỉ đọc (${sx.migrationStatus().error})`);
  sx.close();
  results.released = { frozen, migrated_equals_rust: S.canonicalJson(got) === S.canonicalJson(exp.state) };
}

// ---------- 04 B6 / BM2-01: kiểm tra chặt ảnh và tệp đính kèm — cùng manifest, cùng mã với Rust (store/tests/media.rs) ----------
{
  const MD = join(TV, "media");
  const mm = JSON.parse(readFileSync(join(MD, "manifest.json"), "utf8"));
  let filesOk = true;
  for (const [f, h] of Object.entries(mm.files)) if (sha(readFileSync(join(MD, f))) !== h) filesOk = false;
  ok(filesOk, "B6: tệp mẫu media đúng sha256 của manifest");
  const wrong = [];
  for (const c of mm.cases) {
    const got = S.mediaVerdict(c.kind, c.mime, new Uint8Array(readFileSync(join(MD, c.file))), c.width, c.height);
    if (got !== c.expect) wrong.push(`${c.file}/${c.mime}/${c.kind}: ${got} ≠ ${c.expect}`);
  }
  ok(wrong.length === 0, `B6: ${mm.cases.length} ca media cho đúng mã kỳ vọng (= Rust) ${wrong.join("; ")}`);
  // Tệp sao lưu (của kẻ gửi) mang tệp đính kèm độc → từ chối toàn bộ, thông báo chung (S12).
  const SD = join(TV, "supplement-yc05");
  const sdoc = JSON.parse(readFileSync(join(SD, "notice-v1.json"), "utf8"));
  const pdf0 = new Uint8Array(readFileSync(join(SD, "giay-chung-tu.pdf")));
  const blobsOf = (doc, over) => [...doc.photos.map((p) => ({ id: p.id, bytes: exPhoto })),
    ...doc.attachments.map((a) => ({ id: a.id, bytes: over?.[a.id] ?? (a.mime === "application/pdf" ? pdf0 : exPhoto) }))];
  ok((await code(() => S.verifyContents(S.canonicalJson(sdoc), blobsOf(sdoc)))) === "OK", "B6: đối chứng — vector bổ sung hợp lệ");
  const evil = [
    [0, "image/jpeg", "pdf-ok.pdf"], [0, "image/jpeg", "png-20000x20000-bomb.png"], [0, "image/jpeg", "html-as-image.html"],
    [0, "image/jpeg", "jpeg-exif-gps.jpg"], [0, "image/webp", "webp-exif.webp"], [1, "application/pdf", "pdf-fake-no-eof.pdf"],
    [1, "application/pdf", "html-as-image.html"]];
  const bad = [];
  for (const [i, mime, f] of evil) {
    const d = structuredClone(sdoc);
    const b = new Uint8Array(readFileSync(join(MD, f)));
    Object.assign(d.attachments[i], { mime, bytes: b.length, sha256: sha(b), stripped: mime !== "application/pdf" });
    let msg = "";
    try { await S.verifyContents(S.canonicalJson(d), blobsOf(d, { [d.attachments[i].id]: b })); } catch (e) { msg = e.message; }
    if (msg !== "Sai mật khẩu hoặc tệp bị hỏng") bad.push(`${f}:${msg}`);
  }
  ok(bad.length === 0, `B6/S12: ${evil.length} tệp sao lưu có tệp đính kèm độc (X21 PDF khai image/jpeg, bom PNG, HTML, EXIF, WebP EXIF, PDF giả) → từ chối, thông báo chung ${bad.join("; ")}`);
  // Thêm ảnh / tệp (BM2-08).
  const sa = await freshStore();
  ok((await code(() => sa.addAttachment({ id: "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e7a", mime: "image/jpeg", file_name: null, bytes: new Uint8Array(readFileSync(join(MD, "jpeg-exif-gps.jpg"))) }))) === "input",
    "B6/BM2-08: addAttachment ảnh còn EXIF → input");
  ok((await code(() => sa.addAttachment({ id: "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e7a", mime: "application/pdf", file_name: null, bytes: new Uint8Array(readFileSync(join(MD, "pdf-fake-no-eof.pdf"))) }))) === "input",
    "B6: addAttachment PDF không có %%EOF → input");
  // X22 / M09 khi nhập tệp sao lưu của bản dựng cũ.
  const RD2 = join(TV, "released", "0.2-dev-before-attachments");
  const odoc = JSON.parse(readFileSync(join(RD2, "store-snapshot.json"), "utf8"));
  const ob = odoc.photos.map((p) => ({ id: p.id, bytes: new Uint8Array(readFileSync(join(RD2, "store", "photos", `${p.id}.img`))) }));
  ok((await code(() => S.verifyContents(S.canonicalJson(odoc), ob))) === "OK", "B5: tài liệu cũ hợp lệ được chuyển khi nhập");
  const x22 = structuredClone(odoc); x22.persons[0].death.notice.attachments = [];
  const m09 = structuredClone(odoc); m09.persons[0].death.notice.certificate_photo_ids = ["0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5eff"];
  ok((await code(() => S.verifyContents(S.canonicalJson(x22), ob))) === "invalid" && (await code(() => S.verifyContents(S.canonicalJson(m09), ob))) === "invalid",
    "X22/M09: nhập tài liệu có cả hai khóa, hoặc id ảnh treo → từ chối");
  results.media = { cases: mm.cases.length, mismatches: wrong.length };
}

writeFileSync(join(OUT, "results.json"), JSON.stringify(results, null, 2) + "\n");
console.log(`\n${fails ? `KHÔNG ĐẠT: ${fails} lỗi` : "ĐẠT"} — ${join("pwa/test/store/out/results.json")}`);
process.exit(fails ? 1 : 0);
