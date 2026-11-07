// Kiểm thử nâng cấp tại chỗ — docs/v0.2/04 Bổ sung B5 / R14 (bản web).
//
// Dựng bản CŨ từ git, phục vụ bản cũ, ghi dữ liệu bằng chính mã của bản cũ, rồi thay bằng bản mới qua đường cập nhật
// thật của service worker (thanh "Đã có bản cập nhật…" → "Để sau" → "Cập nhật ngay"), và so dữ liệu sau nâng cấp với
// dữ liệu kỳ vọng do hàm chuyển đổi của lõi (`migrate1to2`, ≡ Rust `migrate_1_to_2`) sinh ra.
//   (a) 0.1.4 (d9650c7): cỡ chữ, giao diện tối; "Để sau" giữ bản cũ chạy tiếp.
//   (b) 0.2-dev (c4d3128, kho IndexedDB v1 có `certificate_photo_ids`): hồ sơ, sự kiện, ảnh, thông tin báo tử có ảnh
//       giấy báo tử, cài đặt → chuyển sang `attachments` (giay_chung_tu), không mất gì.
//   (c) như (b) nhưng chuyển đổi lỗi (móc kiểm thử) → dữ liệu cũ còn nguyên, chế độ chỉ đọc + hướng dẫn xuất sao lưu.
// Chạy: npm run build && node test/upgrade.mjs   (cần git có lịch sử: CI checkout fetch-depth: 0)
import { execFileSync } from "node:child_process";
import { createServer } from "node:http";
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, symlinkSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { DUMP_IN_PAGE, restoreInPage } from "./tools/idb-dump.mjs";
import { dirname, extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright-core";
import { buildSync } from "esbuild";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const REPO = join(ROOT, "..");
const DIST = join(ROOT, "dist");
const OUT = join(ROOT, "test", "out-upgrade");
const SHOTS = join(ROOT, "test", "shots");
mkdirSync(OUT, { recursive: true });
mkdirSync(SHOTS, { recursive: true });
const OLD = { v014: "d9650c7", v02dev: "c4d3128" };
// Bản MỚI = tệp VERSION của cây mã đang kiểm (bản dựng dist/). Ca (a) phải là nâng cấp THẬT từ 0.1.4 lên số này — cùng
// số phiên bản thì kiểm thử không chứng minh được gì cho bản phát hành (08-QUY-TRINH §10).
const NEW_VERSION = readFileSync(join(REPO, "VERSION"), "utf8").trim();
const footVersion = (page) => page.evaluate(() => (/Phiên bản (\d+\.\d+\.\d+)/.exec((document.querySelector(".foot")?.textContent ?? "").replace(/\u00a0/g, " ")) ?? [])[1] ?? null);

const failures = [];
const check = (ok, msg) => { if (!ok) failures.push(msg); console.log(`${ok ? "  ok " : "FAIL "} ${msg}`); };

/**
 * Bản MỚI dựng kiểm thử (LVHV_TEST_HOOKS=1) — chỉ dùng cho ca (c) chuyển đổi lỗi: móc `__lvhvFailMigration` chỉ có ở
 * bản dựng này. Ca (a), (b) dùng bản dựng chính dist/ (không có móc nào). Cùng mã nguồn, cùng VERSION.
 */
function buildNewTest() {
  const dist = join(OUT, "dist-new-testhooks");
  rmSync(dist, { recursive: true, force: true });
  execFileSync("node", ["build.mjs"], { cwd: ROOT, env: { ...process.env, LVHV_TEST_HOOKS: "1", LVHV_OUT_DIR: dist }, stdio: "inherit" });
  return dist;
}

/** Dựng bản PWA của một commit cũ vào OUT/dist-<commit> (dùng node_modules hiện tại — cùng phiên bản phụ thuộc). */
function buildOld(commit) {
  const src = join(OUT, `src-${commit}`), dist = join(OUT, `dist-${commit}`);
  if (existsSync(join(dist, "index.html"))) return dist;
  rmSync(src, { recursive: true, force: true });
  mkdirSync(src, { recursive: true });
  const paths = ["pwa", "tsport", "content", "core/src", "VERSION", "CHANGELOG.md"];
  const tar = execFileSync("git", ["-C", REPO, "archive", commit, ...paths], { maxBuffer: 1 << 30 });
  execFileSync("tar", ["-x", "-C", src], { input: tar });
  symlinkSync(join(ROOT, "node_modules"), join(src, "pwa", "node_modules"), "dir");
  execFileSync("node", ["build.mjs"], { cwd: join(src, "pwa"), env: { ...process.env, LVHV_OUT_DIR: dist }, stdio: "inherit" });
  return dist;
}

const TYPES = { ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".css": "text/css; charset=utf-8", ".svg": "image/svg+xml", ".png": "image/png", ".woff2": "font/woff2", ".webmanifest": "application/manifest+json", ".txt": "text/plain; charset=utf-8", ".json": "application/json" };
let serveDir = DIST;
const server = createServer((req, res) => {
  const url = new URL(req.url, "http://x");
  let p = normalize(decodeURIComponent(url.pathname)).replace(/^([/\\])+/, "");
  if (!p || p.endsWith("/")) p += "index.html";
  const f = join(serveDir, p);
  if (!f.startsWith(serveDir) || !existsSync(f) || statSync(f).isDirectory()) { res.writeHead(404); res.end(); return; }
  res.writeHead(200, { "content-type": TYPES[extname(f)] ?? "application/octet-stream", "cache-control": "no-cache" });
  res.end(readFileSync(f));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const BASE = `http://127.0.0.1:${server.address().port}/`;
const browser = await chromium.launch();

async function newPage() {
  const ctx = await browser.newContext({ viewport: { width: 360, height: 780 }, deviceScaleFactor: 2, locale: "vi-VN", timezoneId: "Asia/Ho_Chi_Minh" });
  const page = await ctx.newPage();
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  return { ctx, page, errors };
}
async function controlled(page) {
  await page.evaluate(async () => { await navigator.serviceWorker.ready; });
  if (!(await page.evaluate(() => !!navigator.serviceWorker.controller))) await page.reload({ waitUntil: "load" });
  await page.waitForFunction(() => !!navigator.serviceWorker.controller, null, { timeout: 15000 });
}
/**
 * Mã lưu trữ hiện tại (src/store/index.ts) đóng gói IIFE để kiểm thử nạp vào trang bằng page.evaluate (qua giao thức gỡ
 * lỗi — không phải script trên trang, CSP không đổi). Dùng ở ca (b): bản mới là bản dựng CHÍNH, không có `window.__lvhv`
 * (chỉ bản dựng kiểm thử có), nên kiểm thử tự mang mã đọc kho — cùng mã nguồn với ứng dụng, cùng IndexedDB của trang.
 */
const STORE_LIB = buildSync({ entryPoints: [join(ROOT, "src", "store", "index.ts")], bundle: true, format: "iife", globalName: "__testStoreLib",
  write: false, target: "es2020", platform: "browser", logLevel: "silent" }).outputFiles[0].text;
const withStoreLib = (page) => page.evaluate(`${STORE_LIB}\n;window.__testStoreLib = __testStoreLib; true`);

/** Chờ service worker của bản mới cài xong và ở trạng thái chờ (trang vẫn do bản cũ điều khiển). */
async function newWorkerWaiting(page) {
  await page.evaluate(async () => {
    const r = await navigator.serviceWorker.getRegistration();
    const installed = (w) => new Promise((ok) => {
      if (w.state === "installed" || w.state === "activated") { ok(); return; }
      w.addEventListener("statechange", () => { if (w.state === "installed" || w.state === "activated") ok(); });
    });
    if (r.waiting) return;
    if (!r.installing) await r.update();
    if (!r.waiting && r.installing) await installed(r.installing);
  });
}
/**
 * Mở lại ứng dụng khi bản mới đang chờ, rồi chờ CẢ HAI lần báo của bản cũ tới thanh: lúc mở, update.ts của bản cũ
 * kiểm tra hai lần (autoCheck, và attachRegistration vì đã có bản chờ), mỗi lần đọc version.json một lần.
 * Vì sao phải chờ: bản 0.1.4 và 0.2-dev có lỗi "Để sau" bị lần báo đến sau bỏ qua — thanh tự hiện lại (đây là
 * nguyên nhân kiểm thử này chập chờn 1/20). Lỗi đã sửa trong app (main.ts, update.ts; kiểm tất định ở smoke.mjs),
 * nhưng mã của bản đã phát hành không sửa được, nên ở đây người dùng giả lập bấm "Để sau" khi các lần báo đã xong.
 */
async function reopenWithPending(page) {
  await page.reload({ waitUntil: "load" });
  await page.waitForSelector(".update-bar:not([hidden])", { timeout: 30000 });
  // Chỉ đếm lần đọc của CHÍNH trang vừa mở (Resource Timing của tài liệu này), không lẫn yêu cầu của trang trước.
  const vjDone = () => performance.getEntriesByType("resource").filter((e) => e.name.endsWith("/version.json") && e.responseEnd > 0).length;
  await page.waitForFunction(`(${vjDone})() >= 2`, null, { timeout: 30000 })
    .catch(async () => { throw new Error(`bản cũ chỉ đọc version.json ${await page.evaluate(`(${vjDone})()`)}/2 lần sau khi mở lại`); });
  await page.waitForLoadState("networkidle"); // phản hồi đã tới trang và kết quả đã báo tới thanh
  return page.evaluate(`(${vjDone})()`);
}
/** Phát hành bản mới (thay thư mục phục vụ), đi đúng đường người dùng: thanh cập nhật → "Để sau" → "Cập nhật ngay". */
async function releaseNew(page, hash, newDir = DIST) {
  serveDir = newDir;
  await page.goto(BASE + hash);
  await newWorkerWaiting(page);
  const reports1 = await reopenWithPending(page);
  await page.click(".update-bar .ubtn:not(.primary)"); // Để sau
  const laterHidden = await page.locator(".update-bar").isHidden();
  const stillOld = await page.evaluate(() => navigator.serviceWorker.controller?.scriptURL ?? "");
  const hasWaiting = await page.evaluate(async () => !!(await navigator.serviceWorker.getRegistration())?.waiting);
  const reports2 = await reopenWithPending(page);
  const barText = (await page.innerText(".update-bar")).replace(/\n/g, " ");
  const versionBefore = await footVersion(page);
  await Promise.all([
    page.waitForEvent("framenavigated", { timeout: 30000 }),
    page.click(".update-bar .ubtn.primary"),
  ]);
  await page.waitForLoadState("load");
  await page.waitForFunction(() => !!document.querySelector('.tab[data-r="nguoi-than"]') && !!navigator.serviceWorker.controller, null, { timeout: 30000 });
  await page.waitForFunction(() => /Phiên bản/.test(document.querySelector(".foot")?.textContent ?? ""), null, { timeout: 30000 }).catch(() => {});
  return { laterHidden, stillOld, hasWaiting, reports1, reports2, barText, versionBefore, versionAfter: await footVersion(page) };
}

// Ghi dữ liệu bằng mã của BẢN CŨ (window.__lvhv.storage của bản 0.2-dev): 3 hồ sơ, ảnh chân dung, thông tin báo tử có
// 2 ảnh giấy báo tử, 2 sự kiện, cài đặt nhắc sao lưu 60 ngày. Trả ảnh chụp kho cũ (tài liệu + sha256 byte ảnh + stateHash).
const SEED = `(async () => {
  const S = await window.__lvhv.storage.loadStore(); const s = await S.openStore();
  const jpg = async (r, g, b) => { const c = new OffscreenCanvas(64, 80); const x = c.getContext("2d"); x.fillStyle = "rgb(" + r + "," + g + "," + b + ")"; x.fillRect(0, 0, 64, 80); x.fillStyle = "#fff"; x.fillRect(8, 8, 20, 30);
    const bl = await c.convertToBlob({ type: "image/jpeg", quality: 0.9 }); return { id: crypto.randomUUID(), mime: "image/jpeg", width: 64, height: 80, bytes: new Uint8Array(await bl.arrayBuffer()) }; };
  const now = "2026-07-20T01:00:00Z";
  const rule = (o) => ({ id: crypto.randomUUID(), enabled: true, offsets_days: [0, 3], time_local: "07:00", tien_thuong: false, ...o });
  const mem = { tray_style: "truyen_thong", incense: true, bia_tieu_de: "", loi_tuong_niem: "Con cháu ghi nhớ.", gio_het_nam: 2, huong_duong_duoi: 60 };
  const portrait = await jpg(120, 80, 40), c1 = await jpg(200, 200, 190), c2 = await jpg(180, 190, 200);
  const A = S.buildPerson({ id: crypto.randomUUID(), kind: "deceased", danh_xung: "Ông", ho_ten: "Nguyễn Văn Nâng", ten_hieu: "", quan_he: "ông nội",
    birth: { input: { calendar: "solar", precision: "day", year: 1940, month: 5, day: 1, leap: false }, noi_sinh: "Hà Nội" },
    death: { input: { calendar: "solar", precision: "day", year: 2025, month: 8, day: 8, leap: false },
      notice: { reported_by: "Chị Lê Thị C", place: "Bệnh viện tỉnh", reason: "Tuổi cao", notice_date: { calendar: "solar", precision: "day", year: 2025, month: 8, day: 8, leap: false }, certificate_photo_ids: [c1.id, c2.id], note: "Đã báo họ hàng." } },
    noi_an_tang: "Nghĩa trang quê", photo_id: portrait.id, memorial: mem, reminders: { gio: rule({ tien_thuong: true }), sinh_nhat: null, sinh_nhat_am: null },
    sinh_nhat_am: false, feb29_rule: "feb28", ghi_chu: "", created_at: now, updated_at: now, rev: 1 });
  await s.putPerson(A, portrait, [c1, c2]);
  const B = S.buildPerson({ id: crypto.randomUUID(), kind: "deceased", danh_xung: "Bà", ho_ten: "Trần Thị Cũ", ten_hieu: "", quan_he: "bà nội", birth: null,
    death: { input: { calendar: "lunar", precision: "month_day", year: null, month: 3, day: 10, leap: false } }, noi_an_tang: "", photo_id: null, memorial: mem,
    reminders: { gio: rule({}), sinh_nhat: null, sinh_nhat_am: null }, sinh_nhat_am: false, feb29_rule: "feb28", ghi_chu: "Không rõ năm mất", created_at: now, updated_at: now, rev: 1 });
  await s.putPerson(B);
  const Cc = S.buildPerson({ id: crypto.randomUUID(), kind: "living", danh_xung: "anh", ho_ten: "Lê Văn Sống", ten_hieu: "", quan_he: "con trai",
    birth: { input: { calendar: "solar", precision: "day", year: 1990, month: 1, day: 20, leap: false }, noi_sinh: "" }, death: null, noi_an_tang: "", photo_id: null, memorial: null,
    reminders: { gio: null, sinh_nhat: rule({}), sinh_nhat_am: rule({}) }, sinh_nhat_am: true, feb29_rule: "feb28", ghi_chu: "", created_at: now, updated_at: now, rev: 1 });
  await s.putPerson(Cc);
  const ev = (o) => ({ id: crypto.randomUUID(), tieu_de: "", recurring: false, ghi_chu: "", reminder: rule({}), created_at: now, updated_at: now, rev: 1, ...o });
  await s.putEvent(ev({ type_id: "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0001", date: { calendar: "solar", precision: "day", year: 2026, month: 8, day: 15, leap: false }, person_ids: [Cc.id] }));
  await s.putEvent(ev({ type_id: "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0002", recurring: true, date: { calendar: "lunar", precision: "day", year: 2020, month: 10, day: 10, leap: false }, person_ids: [Cc.id, A.id] }));
  await s.putSettings({ ...(await s.settings()), backup_nag_days: 60 });
  const doc = await s.snapshot({ version: "0.1.4", platform: "pwa" }, "2026-07-20T01:00:00Z");
  const sha = {}; for (const ph of doc.photos) sha[ph.id] = ph.sha256;
  const hash = await s.stateHash(); s.close();
  return { doc, sha, hash, A: A.id, certs: [c1.id, c2.id] };
})()`;

try {
  const d014 = buildOld(OLD.v014);
  const d02 = buildOld(OLD.v02dev);

  // ---------- (a) 0.1.4 → bản mới: cài đặt giao diện giữ nguyên, "Để sau" ----------
  {
    serveDir = d014;
    const { ctx, page, errors } = await newPage();
    await page.goto(BASE + "#/ngay/2026-07-25", { waitUntil: "load" });
    await page.waitForSelector(".sheet");
    await controlled(page);
    check(await page.locator('.tab[data-r="nguoi-than"]').count() === 0, "(a) đang chạy bản 0.1.4 (chưa có mục Người thân)");
    await page.click('.sizes button[data-v="3"]');
    for (let i = 0; i < 3 && (await page.evaluate(() => JSON.parse(localStorage.getItem("lvhv.ui") ?? "{}").theme)) !== "dark"; i++) await page.click(".themebtn");
    const before = await page.evaluate(() => localStorage.getItem("lvhv.ui"));
    const r = await releaseNew(page, "#/ngay/2026-07-25");
    check(r.laterHidden && /sw\.js/.test(r.stillOld) && r.hasWaiting, `(a) "Để sau" → ẩn thanh, bản cũ chạy tiếp; mở lại → nhắc lại (${JSON.stringify(r)})`);
    check(NEW_VERSION !== "0.1.4" && r.versionBefore === "0.1.4" && r.versionAfter === NEW_VERSION && r.barText.includes(`Đã có phiên bản mới ${NEW_VERSION}.`),
      `R14 (a): nâng cấp THẬT 0.1.4 → ${NEW_VERSION} qua service worker — thanh "${r.barText.split("Cập nhật")[0].trim()}", chân trang ${r.versionBefore} → ${r.versionAfter}`);
    const after = await page.evaluate(() => ({ ls: localStorage.getItem("lvhv.ui"), size: document.documentElement.dataset.size, theme: document.documentElement.dataset.theme }));
    check(JSON.parse(before).size === "3" && JSON.parse(before).theme === "dark" && JSON.parse(after.ls).size === "3" && JSON.parse(after.ls).theme === "dark" && after.size === "3",
      `R14 (a) 0.1.4 → mới qua "Cập nhật ngay": cỡ chữ A++ và giao diện tối giữ nguyên (${before} → ${after.ls})`);
    check(errors.length === 0, `(a) không có lỗi trang (${errors.join(" | ") || "0"})`);
    await page.screenshot({ path: join(SHOTS, "v02-upgrade-014-360.png") });
    await ctx.close();
  }

  // ---------- (b) 0.2-dev (IndexedDB v1, certificate_photo_ids) → bản mới ----------
  {
    serveDir = d02;
    const { ctx, page, errors } = await newPage();
    await page.goto(BASE + "#/nguoi-than", { waitUntil: "load" });
    await page.waitForSelector(".people-body");
    await controlled(page);
    await page.click('.sizes button[data-v="2"]');
    const old = await page.evaluate(SEED);
    const ui = await page.evaluate(() => localStorage.getItem("lvhv.ui"));
    const dbv = await page.evaluate(async () => (await indexedDB.databases()).find((d) => d.name === "lvhv")?.version);
    check(dbv === 1 && old.doc.persons.length === 3 && old.doc.photos.length === 3 && old.doc.events.length === 2, `(b) bản 0.2-dev ghi dữ liệu: kho v${dbv}, 3 hồ sơ, 3 ảnh (1 chân dung + 2 giấy báo tử), 2 sự kiện`);
    // Bản dựng chính KHÔNG nhận móc kiểm thử: trang đặt cờ làm chuyển đổi lỗi mà dist/ vẫn chuyển đổi bình thường.
    await page.addInitScript(() => { globalThis.__lvhvFailMigration = true; });
    await releaseNew(page, "#/nguoi-than");
    await page.goto(BASE + `#/nguoi-than/nguoi/${old.A}`);
    await page.waitForSelector("#notice-box", { timeout: 30000 });
    await withStoreLib(page);
    const got = await page.evaluate(async (o) => {
      const S = window.__testStoreLib; const s = await S.openStore();
      const doc = await s.snapshot({ version: "0.1.4", platform: "pwa" }, "2026-07-20T01:00:00Z");
      const hex = async (b) => [...new Uint8Array(await crypto.subtle.digest("SHA-256", b))].map((x) => x.toString(16).padStart(2, "0")).join("");
      const att = {}; for (const a of doc.attachments ?? []) att[a.id] = { sha: a.sha256, real: await hex(await s.attachmentBytes(a.id)) };
      // Kỳ vọng: hàm chuyển đổi của lõi trên đúng tài liệu bản cũ đã ghi (tất định — không có id ngẫu nhiên).
      const exp = S.migrate1to2(o.doc).doc;
      const v = (await indexedDB.databases()).find((d) => d.name === "lvhv")?.version;
      const ms = s.migrationStatus();
      let valid = "OK"; try { S.validate.document(doc); } catch (e) { valid = e.code + " " + e.path; }
      s.close();
      return { doc, att, exp, v, ms, valid, idbv: 3 };
    }, old);
    const norm = (p) => JSON.stringify(p);
    const byId = (a) => Object.fromEntries(a.map((x) => [x.id, x]));
    const gotP = byId(got.doc.persons), expP = byId(got.exp.persons);
    let lost = 0, diff = 0;
    for (const p of old.doc.persons) {
      if (!gotP[p.id]) { lost++; continue; }
      const want = expP[p.id] ?? p;
      if (norm(gotP[p.id]) !== norm(want)) diff++;
    }
    check(lost === 0 && diff === 0, `R14 (b): 3/3 hồ sơ còn, khớp từng byte dữ liệu kỳ vọng của migrate1to2 của lõi (mất ${lost}, khác ${diff})`);
    check(JSON.stringify(got.doc.attachments) === JSON.stringify(got.exp.attachments) && JSON.stringify(got.doc.photos) === JSON.stringify(got.exp.photos),
      "R14: kho attachments và photos sau nâng cấp = kỳ vọng của lõi");
    const legacyA = gotP[old.A].legacy;
    check(legacyA && JSON.stringify(legacyA.certificate_photo_ids) === JSON.stringify(old.certs) && legacyA.certificate_photos.length === 2,
      "04 B5 mục 2: bản gốc certificate_photo_ids và siêu dữ liệu ảnh giữ trong khóa `legacy` của hồ sơ (không xóa)");
    const nA = gotP[old.A].death.notice;
    check(!("certificate_photo_ids" in nA) && nA.attachments.length === 2 && nA.attachments.every((a, i) => a.kind === "giay_chung_tu" && a.file_id === old.certs[i]),
      "R14 (c): certificate_photo_ids → attachments (2 tệp, loại giấy chứng tử), đúng thứ tự");
    check(old.certs.every((id) => got.att[id] && got.att[id].sha === old.sha[id] && got.att[id].real === old.sha[id]),
      "R14: byte ảnh giấy báo tử chuyển sang tệp đính kèm còn nguyên (sha256 trước = sau)");
    const portraitId = old.doc.persons.find((p) => p.id === old.A).photo_id;
    check(got.doc.photos.length === 1 && got.doc.photos[0].id === portraitId && got.doc.photos[0].sha256 === old.sha[portraitId], "R14: ảnh chân dung còn nguyên");
    check(JSON.stringify(got.doc.events) === JSON.stringify(old.doc.events) && JSON.stringify(got.doc.settings) === JSON.stringify(old.doc.settings) && got.doc.event_types.length === old.doc.event_types.length,
      "R14: sự kiện, loại sự kiện, cài đặt (nhắc sao lưu 60 ngày) giữ nguyên từng byte");
    check(got.v === got.idbv && got.ms.state !== "read_only" && got.valid === "OK", `R14: kho nâng lên IndexedDB v${got.v}, không ở chế độ chỉ đọc, toàn bộ dữ liệu qua kiểm tra (${got.valid})`);
    check(got.v === got.idbv && got.ms.state !== "read_only" && await page.locator("#ro-banner:not([hidden])").count() === 0,
      "(b) bản dựng chính bỏ qua cờ kiểm thử __lvhvFailMigration (trang đã đặt): chuyển đổi bình thường, không có khung chỉ đọc");
    const uiAfter = JSON.parse(await page.evaluate(() => localStorage.getItem("lvhv.ui")));
    check(Object.entries(JSON.parse(ui)).every(([k, x]) => uiAfter[k] === x) && uiAfter.size === "2", `R14: tùy chọn giao diện (A+) giữ nguyên — chỉ có thể thêm khóa, không đổi khóa cũ (${ui} → ${JSON.stringify(uiAfter)})`);
    await page.click("#notice-box > summary");
    const nb = await page.innerText("#notice-box");
    check((nb.match(/Giấy chứng tử/g) ?? []).length === 2 && nb.includes("Chị Lê Thị C"), "R14: trang hồ sơ hiện 2 tệp đính kèm \"Giấy chứng tử\" và thông tin báo tử cũ");
    await page.goto(BASE + "#/nguoi-than");
    await page.waitForSelector(".people-body > *", { timeout: 30000 });
    await withStoreLib(page);
    const remind = await page.evaluate(async () => { const S = window.__testStoreLib; const s = await S.openStore(); const p = await s.persons(); s.close(); return p.filter((x) => x.reminders.gio || x.reminders.sinh_nhat).length; });
    check(remind === 3, "R14: lịch nhắc của cả 3 hồ sơ còn nguyên (quy tắc nhắc giữ trong hồ sơ)");
    // Đối chứng cho TKN3-P-01: ở chế độ thường cùng bộ chọn tìm thấy nút ghi (phép đếm 0 ở ca (c) không rỗng nghĩa).
    await page.goto(BASE + "#/nguoi-than/ho-so"); await page.waitForSelector("#list-deceased");
    const wList = await page.locator('a[href*="/nguoi-than/them/"]').count();
    await page.goto(BASE + `#/nguoi-than/nguoi/${old.A}`); await page.waitForSelector("#person-name");
    const wPerson = await page.locator("#edit-person, #del-person, #add-event").count();
    check(wList === 2 && wPerson === 3, `(b) chế độ thường: có nút ghi (danh sách ${wList} "+ Thêm", trang hồ sơ ${wPerson}: Sửa, Xóa hồ sơ, + Thêm sự kiện)`);
    // #38 (core3, 01 §7, BM2-07): 7 ngày sau chuyển đổi, mở kho thì `legacy` được dọn khỏi mọi hồ sơ (không đổi rev,
    // updated_at); trước 7 ngày thì còn. Đồng hồ của lõi (`OpenOptions.now`) đặt sau mốc `migrated_at` 6 và 8 ngày.
    await withStoreLib(page);
    const pr = await page.evaluate(async () => {
      const S = window.__testStoreLib;
      const iso = (ms) => new Date(ms).toISOString().replace(/\.\d+Z$/, "Z");
      let s = await S.openStore(); const at = await s.meta("migrated_at"); const before = await s.persons(); s.close();
      s = await S.openStore(indexedDB, { now: iso(Date.parse(at) + 6 * 86400000) }); const d6 = (await s.persons()).filter((p) => "legacy" in p).length; s.close();
      s = await S.openStore(indexedDB, { now: iso(Date.parse(at) + 8 * 86400000) });
      const after = await s.persons(); const pruned = await s.meta("legacy_pruned_at"); const left = await s.meta("migrated_at"); s.close();
      const strip = (p) => { const { legacy: _l, ...r } = p; return JSON.stringify(r); };
      return { at, pruned, left: left ?? null, had: before.filter((p) => "legacy" in p).length, d6, d8: after.filter((p) => "legacy" in p).length,
        same: before.every((p) => { const q = after.find((x) => x.id === p.id); return q && strip(q) === strip(p); }) };
    });
    check(typeof pr.at === "string" && pr.had >= 1 && pr.d6 === pr.had && pr.d8 === 0 && pr.same && typeof pr.pruned === "string" && pr.left === null,
      `#38: legacy giữ 7 ngày sau chuyển đổi (ngày 6: còn ${pr.d6}/${pr.had}), ngày 8 mở kho → dọn hết, phần còn lại của hồ sơ giữ từng byte (${JSON.stringify(pr)})`);
    await page.goto(BASE + `#/nguoi-than/nguoi/${old.A}`); await page.reload();
    await page.waitForSelector("#notice-box", { timeout: 30000 });
    await page.click("#notice-box > summary");
    check(((await page.innerText("#notice-box")).match(/Giấy chứng tử/g) ?? []).length === 2, "#38: sau khi dọn legacy, hồ sơ vẫn hiện 2 tệp đính kèm \"Giấy chứng tử\"");
    check(errors.length === 0, `(b) không có lỗi trang (${errors.join(" | ") || "0"})`);
    await page.screenshot({ path: join(SHOTS, "v02-upgrade-02dev-360.png"), fullPage: true });
    await ctx.close();
  }

  // ---------- (c) chuyển đổi lỗi: dữ liệu cũ còn nguyên, chế độ chỉ đọc ----------
  {
    serveDir = d02;
    const { ctx, page } = await newPage();
    await page.goto(BASE + "#/nguoi-than", { waitUntil: "load" });
    await page.waitForSelector(".people-body");
    await controlled(page);
    const old = await page.evaluate(SEED);
    await page.addInitScript(() => { globalThis.__lvhvFailMigration = true; });
    // Đường tải về (<a download>): tắt hộp "Lưu tệp" của hệ điều hành (showSaveFilePicker) — có kiểm riêng ở smoke.
    await page.addInitScript(() => { Object.defineProperty(window, "showSaveFilePicker", { value: undefined, configurable: true }); });
    await releaseNew(page, "#/ngay/2026-07-25", buildNewTest()); // bản mới dựng kiểm thử: chỉ bản này có móc
    await page.goto(BASE + "#/nguoi-than/ho-so");
    await page.waitForSelector("#ro-banner", { timeout: 30000 });
    const st = await page.evaluate(async () => {
      // Mở kho ĐANG CÓ (không nâng phiên bản, không chạy lại chuyển đổi) để đo phiên bản và stateHash. Việc thử ghi
      // làm trên kho của chính ứng dụng (MA3C-30-02, bên dưới).
      const S = await window.__lvhv.storage.loadStore();
      const db = await new Promise((ok, ko) => { const r = indexedDB.open("lvhv"); r.onsuccess = () => ok(r.result); r.onerror = () => ko(r.error); });
      const s = new S.Store(db, { state: "read_only", from_version: db.version, to_version: db.version, error: null });
      const r = { hash: await s.stateHash(), v: db.version, persons: (await s.persons()).length };
      db.close(); return r;
    });
    const banner = await page.innerText("#ro-banner");
    check(st.hash === old.hash && st.v === 1 && st.persons === 3, `R14 ca lỗi: chuyển đổi lỗi → kho v1 còn nguyên (stateHash trước = sau, ${st.persons} hồ sơ)`);
    // MA3C-30-02: ghi bị chặn trên ĐÚNG kho mà mô-đun Người thân đang dùng (`store()` của ứng dụng, lộ ra chỉ ở bản dựng
    // kiểm thử) — không phải một Store do kiểm thử tự dựng.
    const app = await page.evaluate(async (pid) => {
      const s = await window.__lvhvTest.peopleStore();
      const code = async (f) => { try { await f(); return "ok"; } catch (e) { return e.code ?? String(e); } };
      const p = (await s.persons()).find((x) => x.id === pid);
      const r = { state: s.migrationStatus().state, v: s.db.version,
        settings: await code(async () => s.putSettings(await s.settings())),
        person: await code(() => s.putPerson({ ...p, ghi_chu: "thử ghi" })),
        del: await code(() => s.deletePerson(pid)),
        saved: await code(() => s.markBackupSaved("2026-07-25T00:00:00Z")),
        ui: await s.putUiMeta("ui_test_flag", "x") };
      r.hash = await s.stateHash(); r.uiFlag = (await s.meta("ui_test_flag")) ?? null;
      return r;
    }, old.A);
    check(app.state === "read_only" && app.v === 1 && app.settings === "read_only" && app.person === "read_only" && app.del === "read_only" && app.saved === "read_only" && app.hash === old.hash
      && banner.includes("chế độ chỉ đọc") && banner.includes("Tạo tệp sao lưu"),
      `R14 ca lỗi (MA3C-30-02): kho của ứng dụng ở chế độ chỉ đọc — lưu cài đặt, lưu hồ sơ, xóa hồ sơ, ghi nhận sao lưu đều bị lõi từ chối; stateHash không đổi; có khung hướng dẫn (${JSON.stringify(app)})`);
    check(app.ui === false && app.uiFlag === null, "MA3C-30-04: chế độ chỉ đọc → cờ giao diện (putUiMeta) không được ghi vào kho cũ");
    check((await page.innerText(".people-body")).includes("Nguyễn Văn Nâng"), "R14 ca lỗi: vẫn xem được hồ sơ cũ");
    // TKN3-P-01: chế độ chỉ đọc ẩn MỌI việc ghi (như Android, Windows); vẫn xem, tạo tệp sao lưu, xuất .ics được.
    const WRITE = 'a[href*="/nguoi-than/them/"], a[href*="/nguoi-than/sua/"], a[href*="/nguoi-than/bao-tu/"], a[href*="/nguoi-than/su-kien/"], #edit-person, #convert-person, #del-person, #add-event, .ev-acts, #bk-in, #bk-open, #bk-file-btn, #bk-nag, #persist-ok';
    const roPages = [];
    const living = old.doc.persons.find((p) => p.kind === "living"), dead = old.doc.persons.find((p) => p.kind === "deceased");
    for (const [hash, sel] of [["#/nguoi-than", ".people-body > *"], ["#/nguoi-than/ho-so", "#list-deceased"], ["#/nguoi-than/theo-ky", ".people-body > *"],
      [`#/nguoi-than/nguoi/${dead.id}`, "#person-name"], [`#/nguoi-than/nguoi/${living.id}`, "#person-name"], ["#/nguoi-than/sao-luu", "#bk-export"]]) {
      await page.goto(BASE + hash); await page.waitForSelector(sel, { timeout: 30000 });
      roPages.push({ hash, write: await page.locator(WRITE).count(), banner: await page.locator("#ro-banner").count() });
    }
    const keep = { exp: await page.locator("#bk-export:not([disabled])").count(), ics: await page.locator("#ics-open").count() };
    check(roPages.every((x) => x.write === 0 && x.banner === 1) && keep.exp === 1 && keep.ics === 1,
      `TKN3-P-01: chỉ đọc → 0 nút ghi ở Sắp tới, Hồ sơ, Theo kỳ, trang người đã mất, người đang sống, Sao lưu; còn "Tạo tệp sao lưu…" và "Xuất tệp lịch" (${JSON.stringify(roPages.map((x) => x.write))}, ${JSON.stringify(keep)})`);
    await page.goto(BASE + "#/nguoi-than/them/da-mat");
    await page.waitForSelector("#ro-noform", { timeout: 30000 });
    const noForm = await page.locator("#pf-ten").count() === 0 && (await page.innerText("#ro-noform")).includes("chế độ chỉ đọc");
    await page.goto(BASE + `#/nguoi-than/sua/${dead.id}`);
    await page.waitForSelector("#ro-noform", { timeout: 30000 });
    check(noForm && await page.locator("#pf-ten").count() === 0, "TKN3-P-01: mở thẳng địa chỉ Thêm / Sửa ở chế độ chỉ đọc → không dựng biểu mẫu, có câu giải thích");
    // Vẫn tạo được tệp sao lưu, không lỗi, và kho cũ không đổi (không ghi "đã sao lưu").
    await page.goto(BASE + "#/nguoi-than/sao-luu"); await page.waitForSelector("#bk-export");
    await page.click("#bk-export");
    await page.fill("#bk-pw1", "nhà có cây bàng"); await page.fill("#bk-pw2", "nhà có cây bàng");
    const roDl = page.waitForEvent("download", { timeout: 60000 });
    await page.click(".bk-dialog .dbtn.main");
    const roFile = await roDl.then((d) => d.path()).then((f) => readFileSync(f)).catch(() => null);
    await page.waitForSelector(".bk-dialog", { state: "detached", timeout: 30000 }).catch(() => {});
    const roToast = await page.locator(".toast").first().innerText().catch(() => "");
    const roErr = await page.locator(".bk-dialog .ferr:not([hidden])").count();
    const roHash = await page.evaluate(async () => {
      const S = await window.__lvhv.storage.loadStore();
      const db = await new Promise((ok, ko) => { const r = indexedDB.open("lvhv"); r.onsuccess = () => ok(r.result); r.onerror = () => ko(r.error); });
      const s = new S.Store(db, { state: "read_only", from_version: db.version, to_version: db.version, error: null });
      const r = { hash: await s.stateHash(), v: db.version }; db.close(); return r;
    });
    check(!!roFile && roFile.subarray(0, 7).toString() === "LVHVBAK" && await page.locator(".bk-dialog").count() === 0 && roErr === 0 && roToast.startsWith("Đã tạo tệp sao lưu") && roHash.hash === old.hash && roHash.v === 1,
      `MA3C-30-01 / TKN3-P-01: chỉ đọc vẫn tạo được tệp sao lưu (${roFile ? roFile.length : 0} byte), hộp đóng, báo "${roToast.split(".")[0]}", không lỗi; kho cũ không đổi (stateHash, v${roHash.v})`);
    // MA3C-30-01: tệp xuất từ chế độ chỉ đọc khôi phục được vào kho TRỐNG của bản mới (bản dựng chính, ngữ cảnh mới).
    {
      serveDir = DIST;
      const fresh = await newPage();
      await fresh.page.goto(BASE + "#/nguoi-than/sao-luu");
      await fresh.page.waitForSelector("#bk-file", { state: "attached", timeout: 30000 });
      await fresh.page.setInputFiles("#bk-file", { name: "tu-che-do-chi-doc.lvhv", mimeType: "application/octet-stream", buffer: roFile ?? Buffer.alloc(0) });
      await fresh.page.fill("#bk-ipw", "nhà có cây bàng");
      await fresh.page.click("#bk-open");
      await fresh.page.waitForSelector(".imp-dialog[open]", { timeout: 60000 });
      await fresh.page.click(".imp-dialog .dbtn.main");
      const restored = await fresh.page.waitForSelector('.toast:has-text("Đã khôi phục: 3 người")', { timeout: 60000 }).then(() => true, () => false);
      await fresh.page.goto(BASE + "#/nguoi-than/ho-so"); await fresh.page.waitForSelector("#list-deceased");
      const rows = await fresh.page.locator(".prow").count();
      await fresh.page.goto(BASE + `#/nguoi-than/nguoi/${old.A}`); await fresh.page.waitForSelector("#notice-box");
      await fresh.page.click("#notice-box > summary");
      const certs = ((await fresh.page.innerText("#notice-box")).match(/Giấy chứng tử/g) ?? []).length;
      const fv = await fresh.page.evaluate(async () => (await indexedDB.databases()).find((d) => d.name === "lvhv")?.version);
      check(restored && rows === 3 && certs === 2 && fv === 3 && fresh.errors.length === 0,
        `MA3C-30-01: tệp sao lưu tạo ở chế độ chỉ đọc khôi phục vào kho trống của bản mới — 3 hồ sơ, 2 ảnh giấy báo tử thành tệp đính kèm "Giấy chứng tử", kho v${fv} (${fresh.errors.join(" | ") || "0 lỗi trang"})`);
      await fresh.ctx.close();
    }
    await page.goto(BASE + "#/nguoi-than/ho-so"); await page.waitForSelector("#list-deceased");
    await page.screenshot({ path: join(SHOTS, "v02-upgrade-readonly-360.png"), fullPage: true });
    await ctx.close();
  }
  // ---------- (d) bộ mẫu đóng băng của các bản web đã phát hành (released/<phiên bản>/pwa, 08-QUY-TRINH §10) ----------
  // Mỗi bản phát hành web thêm bộ mẫu tạo bằng giao diện của chính nó (pwa/test/tools/freeze-pwa-release.mjs). Mọi bản
  // sau phải: (1) bộ mẫu còn nguyên (sha256); (2) mở được kho đó — dựng lại từ indexeddb.json rồi để bản đang kiểm mở
  // (tự chuyển lược đồ nếu cần); (3) khôi phục được tệp sao lưu của bản đó vào kho trống; (4) khi đã có thẻ git
  // v<phiên bản>: nâng cấp THẬT từ bản dựng của thẻ (dữ liệu nằm trong kho của bản cũ) qua service worker.
  const TV_REL = join(REPO, "docs", "v0.2", "schema", "test-vectors", "released");
  for (const rel of readdirSync(TV_REL).filter((d) => existsSync(join(TV_REL, d, "pwa", "manifest.json"))).sort()) {
    const dir = join(TV_REL, rel, "pwa");
    const man = JSON.parse(readFileSync(join(dir, "manifest.json"), "utf8"));
    const exp = JSON.parse(readFileSync(join(TV_REL, "expected", `${rel}-pwa.json`), "utf8"));
    const bad = Object.entries(man.files).filter(([f, h]) => createHash("sha256").update(readFileSync(join(dir, f))).digest("hex") !== h).map(([f]) => f);
    check(bad.length === 0, `(d) ${rel}: bộ mẫu đóng băng còn nguyên (sha256 ${Object.keys(man.files).length} tệp${bad.length ? `; KHÁC: ${bad.join(", ")}` : ""})`);
    const dump = JSON.parse(readFileSync(join(dir, "indexeddb.json"), "utf8"));
    const backupBytes = readFileSync(join(dir, "backup.lvhv"));
    const measureNow = async (page) => {
      await withStoreLib(page);
      return page.evaluate(`(async () => { const S = window.__testStoreLib; const s = await S.openStore(); try {
        const doc = await s.snapshot({ version: ${JSON.stringify(NEW_VERSION)}, platform: "pwa" }, "2026-11-07T02:00:00Z");
        let valid = "OK"; try { S.validate.document(doc); } catch (e) { valid = (e.code ?? "?") + " " + (e.path ?? ""); }
        return { hash: await s.stateHash(), v: s.db.version, state: s.migrationStatus().state, persons: doc.persons.length, events: doc.events.length, valid }; } finally { s.close(); } })()`);
    };
    // Cùng phiên bản kho → phải giống từng byte (stateHash); kho đã nâng phiên bản → đủ hồ sơ, sự kiện, hợp lệ.
    const sameData = (m, hash) => m.persons === exp.persons && m.events === exp.events && m.valid === "OK" && m.state !== "read_only" && (m.v !== exp.store_version || m.hash === hash);
    // (2) kho của bản đó, mở bằng bản đang kiểm
    {
      serveDir = DIST;
      const { ctx, page, errors } = await newPage();
      await page.goto(BASE + "__dung_lai_kho__"); // trang 404 cùng nguồn gốc: dựng kho trước khi ứng dụng mở
      await page.evaluate(restoreInPage(dump));
      await page.goto(BASE + "#/nguoi-than/ho-so"); await page.waitForSelector("#list-deceased", { timeout: 30000 });
      const rows = await page.locator(".prow").count();
      await page.goto(BASE + `#/nguoi-than/nguoi/${exp.person_ids.D}`); await page.waitForSelector("#notice-box", { timeout: 30000 });
      await page.click("#notice-box > summary");
      const certs = ((await page.innerText("#notice-box")).match(/Giấy chứng tử|Tin báo tử/g) ?? []).length;
      await page.goto(BASE + `#/nguoi-than/tuong-niem/${exp.person_ids.A}`); await page.waitForSelector(".bia-name", { timeout: 30000 });
      const m = await measureNow(page);
      check(sameData(m, exp.state_hash) && rows === exp.persons && certs >= 2 && errors.length === 0,
        `(d) ${rel}: kho IndexedDB của bản ${rel} mở bằng bản đang kiểm (${NEW_VERSION}) — ${m.persons} hồ sơ, ${m.events} sự kiện, kho v${m.v}, ${m.v === exp.store_version ? (m.hash === exp.state_hash ? "stateHash giống từng byte" : "stateHash KHÁC") : "đã chuyển lược đồ"}; trang hồ sơ, tệp đính kèm, tưởng niệm mở được (${errors.join(" | ") || "0 lỗi trang"}; ${JSON.stringify({ ...m, hash: undefined, rows, certs })})`);
      await ctx.close();
    }
    // (3) tệp sao lưu của bản đó → kho trống
    {
      serveDir = DIST;
      const { ctx, page, errors } = await newPage();
      await page.goto(BASE + "#/nguoi-than/sao-luu"); await page.waitForSelector("#bk-file", { state: "attached", timeout: 30000 });
      await page.setInputFiles("#bk-file", { name: "backup.lvhv", mimeType: "application/octet-stream", buffer: backupBytes });
      await page.fill("#bk-ipw", man.password); await page.click("#bk-open");
      await page.waitForSelector(".imp-dialog[open]", { timeout: 60000 }); await page.click(".imp-dialog .dbtn.main");
      const ok = await page.waitForSelector('.toast:has-text("Đã khôi phục")', { timeout: 60000 }).then(() => true, () => false);
      const m = await measureNow(page);
      check(ok && sameData(m, exp.backup_import_state_hash) && errors.length === 0,
        `(d) ${rel}: tệp sao lưu của bản ${rel} khôi phục vào kho trống bằng bản đang kiểm — ${m.persons} hồ sơ, ${m.v === exp.store_version ? (m.hash === exp.backup_import_state_hash ? "stateHash giống từng byte" : "stateHash KHÁC") : "kho mới hơn"} (${JSON.stringify({ ...m, hash: undefined, ok })})`);
      await ctx.close();
    }
    // (4) nâng cấp thật từ bản dựng của thẻ v<rel>
    let tagged = false;
    try { execFileSync("git", ["-C", REPO, "rev-parse", "-q", "--verify", `refs/tags/v${rel}`], { stdio: "ignore" }); tagged = true; } catch { /* chưa phát hành thẻ */ }
    if (!tagged || rel === NEW_VERSION) {
      console.log(`  --   (d) ${rel}: nâng cấp thật từ thẻ v${rel} — ${tagged ? "bản đang kiểm chính là bản này" : "chưa có thẻ"}, bỏ qua (chạy ở bản phát hành sau)`);
    } else {
      const dOld = buildOld(`v${rel}`);
      serveDir = dOld;
      const { ctx, page, errors } = await newPage();
      await page.goto(BASE + "__dung_lai_kho__"); await page.evaluate(restoreInPage(dump));
      await page.goto(BASE + "#/nguoi-than", { waitUntil: "load" }); await page.waitForSelector(".people-body", { timeout: 30000 });
      await controlled(page);
      await releaseNew(page, "#/nguoi-than/ho-so");
      await page.goto(BASE + "#/nguoi-than/ho-so"); await page.waitForSelector("#list-deceased", { timeout: 30000 });
      const m = await measureNow(page);
      check(sameData(m, exp.state_hash) && errors.length === 0, `(d) ${rel} → ${NEW_VERSION}: nâng cấp thật qua service worker, dữ liệu của bản ${rel} còn đủ (${m.persons} hồ sơ, kho v${m.v})`);
      await ctx.close();
    }
  }
  void DUMP_IN_PAGE;
} catch (e) {
  failures.push(String(e?.stack ?? e));
  console.error(e);
}
await browser.close();
server.close();
writeFileSync(join(OUT, "result.json"), JSON.stringify({ failures }, null, 2));
console.log(failures.length ? `\nKHÔNG ĐẠT — ${failures.length} lỗi` : "\nĐẠT — R14 (bản web)");
process.exit(failures.length ? 1 : 0);
