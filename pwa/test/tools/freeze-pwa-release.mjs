// Đóng băng bộ dữ liệu mẫu của bản web vừa phát hành (08-QUY-TRINH §10; 04 Bổ sung B5 / R14; released/README.md).
//
// Dữ liệu được tạo QUA GIAO DIỆN của chính bản dựng phát hành (pwa/dist, `npm run build` trên đúng commit phát hành):
// thêm người đã mất (ảnh chân dung, ảnh mâm lễ nếu bản có mục này, lời tưởng niệm), người đã mất nhập theo âm lịch,
// người đang sống (sinh nhật âm, hai sự kiện), một người đang sống được "Cập nhật: đã mất" kèm thông tin báo tử và hai
// tệp đính kèm (ảnh + PDF), nhắc sao lưu 60 ngày; rồi tạo tệp sao lưu bằng nút "Tạo tệp sao lưu…".
// Đầu ra (không bao giờ sửa sau khi commit):
//   docs/v0.2/schema/test-vectors/released/<VERSION>/pwa/indexeddb.json   kho IndexedDB nguyên vẹn (idb-dump.mjs)
//   …/pwa/backup.lvhv                                                    tệp sao lưu (mật khẩu trong manifest.json)
//   …/pwa/store-snapshot.json                                            ảnh chụp kho dạng tài liệu (để đọc)
//   …/pwa/manifest.json                                                  sha256 từng tệp, cách tạo, mật khẩu
//   docs/v0.2/schema/test-vectors/released/expected/<VERSION>-pwa.json   stateHash kỳ vọng (ngoài bộ mẫu)
// Chạy: cd pwa && npm run build && node test/tools/freeze-pwa-release.mjs [--force]
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createServer } from "node:http";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright-core";
import { buildSync } from "esbuild";
import { DUMP_IN_PAGE, restoreInPage } from "./idb-dump.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const REPO = join(ROOT, "..");
const DIST = join(ROOT, "dist");
const VERSION = readFileSync(join(REPO, "VERSION"), "utf8").trim();
const REL = join(REPO, "docs", "v0.2", "schema", "test-vectors", "released");
const OUT = join(REL, VERSION, "pwa");
const EXPECTED = join(REL, "expected", `${VERSION}-pwa.json`);
const PASSWORD = "mật khẩu thử nghiệm";
const FIXED = new Date("2026-11-07T09:00:00+07:00");
if (existsSync(join(OUT, "manifest.json")) && !process.argv.includes("--force")) { console.error(`Đã có bộ mẫu ${OUT} — bộ mẫu đóng băng không được tạo lại.`); process.exit(2); }
const dv = JSON.parse(readFileSync(join(DIST, "version.json"), "utf8"));
if (dv.version !== VERSION) { console.error(`dist/ là bản ${dv.version}, VERSION là ${VERSION} — hãy npm run build`); process.exit(2); }
const commit = execFileSync("git", ["-C", REPO, "rev-parse", "--short", "HEAD"], { encoding: "utf8" }).trim();

const TYPES = { ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".css": "text/css; charset=utf-8", ".svg": "image/svg+xml", ".png": "image/png", ".jpg": "image/jpeg", ".webp": "image/webp", ".woff2": "font/woff2", ".webmanifest": "application/manifest+json", ".txt": "text/plain; charset=utf-8", ".json": "application/json" };
const server = createServer((req, res) => {
  let p = normalize(decodeURIComponent(new URL(req.url, "http://x").pathname)).replace(/^([/\\])+/, "");
  if (!p || p.endsWith("/")) p += "index.html";
  const f = join(DIST, p);
  if (!f.startsWith(DIST) || !existsSync(f) || statSync(f).isDirectory()) { res.writeHead(404); res.end(); return; }
  res.writeHead(200, { "content-type": TYPES[extname(f)] ?? "application/octet-stream", "cache-control": "no-cache" });
  res.end(readFileSync(f));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const BASE = `http://127.0.0.1:${server.address().port}/`;
// Mã lưu trữ của CHÍNH commit này (src/store), nạp vào trang để đo stateHash / chụp tài liệu (bản dựng chính không có
// window.__lvhv). Chỉ đọc kho; mọi dữ liệu do giao diện ghi.
const STORE_LIB = buildSync({ entryPoints: [join(ROOT, "src", "store", "index.ts")], bundle: true, format: "iife", globalName: "__testStoreLib", write: false, target: "es2020", platform: "browser", logLevel: "silent" }).outputFiles[0].text;
const withLib = (page) => page.evaluate(`${STORE_LIB}\n;window.__testStoreLib = __testStoreLib; true`);
const measure = async (page) => { await withLib(page); return page.evaluate(`(async () => { const S = window.__testStoreLib; const s = await S.openStore(); try {
  return { hash: await s.stateHash(), snapshot: await s.snapshot({ version: ${JSON.stringify(VERSION)}, platform: "pwa" }, "2026-11-07T02:00:00Z"), status: s.migrationStatus(), version: s.db.version }; } finally { s.close(); } })()`); };

const browser = await chromium.launch();
const newCtx = async () => {
  const ctx = await browser.newContext({ viewport: { width: 360, height: 780 }, locale: "vi-VN", timezoneId: "Asia/Ho_Chi_Minh", reducedMotion: "reduce", acceptDownloads: true });
  await ctx.addInitScript(() => { Object.defineProperty(window, "showSaveFilePicker", { value: undefined, configurable: true }); });
  const page = await ctx.newPage();
  await page.clock.setFixedTime(FIXED);
  const errors = []; page.on("pageerror", (e) => errors.push(e.message));
  return { ctx, page, errors };
};
const save = async (page) => { await page.click(".form-acts .primary"); await page.waitForSelector("#person-name", { timeout: 30000 }); return /nguoi\/([0-9a-f-]+)/.exec(page.url())[1]; };
const JPG = readFileSync(join(REPO, "docs/v0.2/schema/test-vectors/photo-0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e09.jpg"));
const PDF = Buffer.from("%PDF-1.4\n1 0 obj<</Type/Catalog>>endobj\n% Ban quet giay chung tu (du lieu mau)\ntrailer<<>>\n%%EOF\n");

const { ctx, page, errors } = await newCtx();
try {
  // A — người đã mất, ngày mất dương (rơi vào tháng 6 nhuận âm lịch), đủ trường, ảnh chân dung, ảnh mâm lễ (nếu có).
  await page.goto(BASE + "#/nguoi-than/them/da-mat"); await page.waitForSelector("#pf-ten");
  await page.fill("#pf-dx", "ông"); await page.check("#pf-gt-nam");
  await page.fill("#pf-ten", "Nguyễn Văn Tổ"); await page.fill("#pf-qh", "ông nội (bên nội)");
  await page.fill("#pf-hieu", "Phúc Thiện"); await page.fill("#pf-antang", "Nghĩa trang Văn Điển, Hà Nội");
  await page.fill("#pf-death-d", "8"); await page.fill("#pf-death-m", "8"); await page.fill("#pf-death-y", "2025");
  await page.fill("#pf-birth-d", "1"); await page.fill("#pf-birth-m", "5"); await page.fill("#pf-birth-y", "1940");
  await page.setInputFiles("#pf-file", { name: "chan-dung.jpg", mimeType: "image/jpeg", buffer: JPG });
  await page.waitForSelector(".pf-photo[data-drawn]");
  await page.click("#pf-mem > summary");
  await page.fill("#pf-loi", "Con cháu muôn đời ghi nhớ công ơn ông.");
  const hasTray = await page.locator("#pf-tray-file").count() === 1;
  if (hasTray) { await page.setInputFiles("#pf-tray-file", { name: "mam-le.jpg", mimeType: "image/jpeg", buffer: JPG }); await page.waitForSelector("#pf-tray-img[data-drawn]"); }
  const idA = await save(page);
  // B — người đã mất, ngày mất nhập theo âm lịch, không ảnh.
  await page.goto(BASE + "#/nguoi-than/them/da-mat"); await page.waitForSelector("#pf-ten");
  await page.fill("#pf-dx", "bà"); await page.check("#pf-gt-nu");
  await page.fill("#pf-ten", "Trần Thị Bích"); await page.fill("#pf-qh", "bà nội");
  await page.click('#pf-death-cal button[data-v="lunar"]');
  await page.fill("#pf-death-d", "10"); await page.fill("#pf-death-m", "3"); await page.fill("#pf-death-y", "2010");
  await save(page);
  // C — người đang sống, sinh nhật âm, hai sự kiện (một lần; lặp hằng năm theo âm lịch).
  await page.goto(BASE + "#/nguoi-than/them/dang-song"); await page.waitForSelector("#pf-ten");
  await page.fill("#pf-dx", "anh"); await page.check("#pf-gt-nam");
  await page.fill("#pf-ten", "Lê Văn Cường"); await page.fill("#pf-qh", "con trai"); await page.fill("#pf-noisinh", "Huế");
  await page.fill("#pf-birth-d", "20"); await page.fill("#pf-birth-m", "1"); await page.fill("#pf-birth-y", "1990");
  await page.check("#pf-snam");
  await save(page);
  await page.click("#add-event"); await page.waitForSelector("#ef-type");
  await page.fill("#ef-date-d", "15"); await page.fill("#ef-date-m", "8"); await page.fill("#ef-date-y", "2026");
  await page.click(".form-acts .primary"); await page.waitForSelector("#event-box .evrow");
  await page.click("#add-event"); await page.waitForSelector("#ef-type");
  await page.selectOption("#ef-type", "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0002");
  await page.click('#ef-date-cal button[data-v="lunar"]');
  await page.fill("#ef-date-d", "10"); await page.fill("#ef-date-m", "10"); await page.fill("#ef-date-y", "2020");
  await page.click(".form-acts .primary"); await page.waitForSelector("#event-box .evrow >> nth=1");
  // D — người đang sống → "Cập nhật: đã mất", thông tin báo tử, tệp đính kèm ảnh + PDF.
  await page.goto(BASE + "#/nguoi-than/them/dang-song"); await page.waitForSelector("#pf-ten");
  await page.fill("#pf-ten", "Đỗ Văn Mạnh"); await page.fill("#pf-qh", "bác");
  await page.fill("#pf-birth-d", "2"); await page.fill("#pf-birth-m", "9"); await page.fill("#pf-birth-y", "1950");
  const idD = await save(page);
  await page.click("#convert-person"); await page.waitForSelector("#cf-death");
  await page.click('#cf-death-cal button[data-v="lunar"]');
  await page.fill("#cf-death-d", "12"); await page.fill("#cf-death-m", "6"); await page.fill("#cf-death-y", "2024");
  await page.fill("#nf-by", "Anh Đỗ Văn Hải"); await page.fill("#nf-place", "Bệnh viện Bạch Mai"); await page.fill("#nf-reason", "Tuổi cao");
  await page.fill("#nf-date-d", "18"); await page.fill("#nf-date-m", "7"); await page.fill("#nf-date-y", "2024");
  await page.selectOption("#nf-kind", "tin_bao_tu");
  await page.setInputFiles("#nf-file", { name: "tin-bao-tu.jpg", mimeType: "image/jpeg", buffer: JPG });
  await page.waitForSelector(".cert-item canvas[data-drawn]");
  await page.selectOption("#nf-kind", "giay_chung_tu");
  await page.setInputFiles("#nf-file", { name: "giay-chung-tu.pdf", mimeType: "application/pdf", buffer: PDF });
  await page.waitForSelector(".pdf-warn-dialog[open]"); await page.click(".pdf-warn-dialog .dbtn.main");
  await page.waitForSelector("#nf-certs .cert-pdf");
  await page.click(".form-acts .primary"); await page.waitForSelector(".cv-dialog[open]"); await page.click(".cv-dialog .dbtn.main");
  await page.waitForSelector("#person-name");
  // Cài đặt: nhắc sao lưu sau 60 ngày.
  await page.goto(BASE + "#/nguoi-than/sao-luu"); await page.waitForSelector("#bk-nag");
  await page.selectOption("#bk-nag", "60"); await page.waitForSelector('.toast:has-text("Đã lưu cài đặt")');
  // Tệp sao lưu bằng nút của giao diện.
  await page.click("#bk-export");
  await page.fill("#bk-pw1", PASSWORD); await page.fill("#bk-pw2", PASSWORD);
  const dl = page.waitForEvent("download", { timeout: 120000 });
  await page.click(".bk-dialog .dbtn.main");
  const backup = readFileSync(await (await dl).path());
  await page.waitForSelector(".bk-dialog", { state: "detached" });
  // Chụp kho.
  const dump = await page.evaluate(DUMP_IN_PAGE);
  const m = await measure(page);
  if (errors.length) throw new Error(`lỗi trang: ${errors.join(" | ")}`);

  // Tự kiểm: (1) dựng lại kho từ dump ở ngữ cảnh mới → stateHash bằng; (2) khôi phục tệp sao lưu vào kho trống bằng giao
  // diện → ghi stateHash sau khôi phục (kỳ vọng cho các bản sau).
  const v1 = await newCtx();
  await v1.page.goto(BASE + "__khong_co__"); await v1.page.evaluate(restoreInPage(dump));
  await v1.page.goto(BASE + "#/nguoi-than/ho-so"); await v1.page.waitForSelector("#list-deceased");
  const m1 = await measure(v1.page);
  await v1.ctx.close();
  if (m1.hash !== m.hash) throw new Error(`dựng lại từ dump khác stateHash (${m1.hash} ≠ ${m.hash})`);
  const v2 = await newCtx();
  await v2.page.goto(BASE + "#/nguoi-than/sao-luu"); await v2.page.waitForSelector("#bk-file", { state: "attached" });
  await v2.page.setInputFiles("#bk-file", { name: "backup.lvhv", mimeType: "application/octet-stream", buffer: backup });
  await v2.page.fill("#bk-ipw", PASSWORD); await v2.page.click("#bk-open");
  await v2.page.waitForSelector(".imp-dialog[open]", { timeout: 60000 }); await v2.page.click(".imp-dialog .dbtn.main");
  await v2.page.waitForSelector('.toast:has-text("Đã khôi phục")', { timeout: 60000 });
  const m2 = await measure(v2.page);
  await v2.ctx.close();

  mkdirSync(OUT, { recursive: true }); mkdirSync(dirname(EXPECTED), { recursive: true });
  const files = {
    "indexeddb.json": JSON.stringify(dump, null, 1) + "\n",
    "backup.lvhv": backup,
    "store-snapshot.json": JSON.stringify(m.snapshot, null, 1) + "\n",
    "README.md": `# Bộ mẫu đóng băng — bản web ${VERSION}

Tạo ngày 2026-11-07 bằng \`pwa/test/tools/freeze-pwa-release.mjs\` trên commit \`${commit}\` (VERSION ${VERSION}),
**qua giao diện của chính bản dựng phát hành** (\`npm run build\` → \`pwa/dist\`, Chromium, đồng hồ cố định
2026-11-07 09:00 giờ Việt Nam). Không bao giờ sửa, đổi tên hay xóa tệp nào ở đây (released/README.md).

| Tệp | Nội dung |
|---|---|
| \`indexeddb.json\` | Kho IndexedDB \`lvhv\` (phiên bản ${dump.version}) chép nguyên: mọi kho con, chỉ mục, bản ghi; byte ảnh / tệp là base64 (\`pwa/test/tools/idb-dump.mjs\`) |
| \`backup.lvhv\` | Tệp sao lưu tạo bằng nút "Tạo tệp sao lưu…"; mật khẩu trong \`manifest.json\` |
| \`store-snapshot.json\` | Ảnh chụp kho dạng tài liệu (để đọc, so sánh) |
| \`manifest.json\` | sha256 từng tệp, cách tạo |

Dữ liệu: A người đã mất nhập ngày dương (tháng 6 nhuận âm lịch), cách gọi, giới tính, tên hiệu, nơi an táng, ảnh
chân dung${hasTray ? ", ảnh mâm lễ" : ""}, lời tưởng niệm; B người đã mất nhập theo âm lịch; C người đang sống, sinh nhật âm,
hai sự kiện (Tốt nghiệp một lần; Ngày cưới âm lịch lặp hằng năm); D người đang sống chuyển "Cập nhật: đã mất", có thông
tin báo tử và hai tệp đính kèm (ảnh tin báo tử, PDF giấy chứng tử); nhắc sao lưu 60 ngày.

Kỳ vọng (ngoài bộ mẫu): \`released/expected/${VERSION}-pwa.json\` — stateHash của kho, stateHash sau khi khôi phục tệp
sao lưu vào kho trống. Kiểm thử: \`pwa/test/upgrade.mjs\` ca (d) — đóng băng (sha256), dựng lại kho từ
\`indexeddb.json\` rồi mở bằng bản đang kiểm, khôi phục \`backup.lvhv\`; khi đã có thẻ \`v${VERSION}\` thì còn nâng cấp
thật từ bản dựng của thẻ đó qua service worker.
`,
  };
  for (const [n, b] of Object.entries(files)) writeFileSync(join(OUT, n), b);
  const sha = (b) => createHash("sha256").update(b).digest("hex");
  const manifest = {
    fixture: `${VERSION}/pwa`,
    created_by: `PWA ${VERSION} (VERSION ${VERSION}, build ${dv.build}) — tạo qua giao diện của chính bản dựng phát hành, commit ${commit}; pwa/test/tools/freeze-pwa-release.mjs`,
    created_at: "2026-11-07",
    idb_version: dump.version,
    password: PASSWORD,
    persons: m.snapshot.persons.length,
    has_tray_photo: hasTray,
    description: "Kho IndexedDB thật của bản web: A người đã mất (ảnh chân dung" + (hasTray ? ", ảnh mâm lễ" : "") + ", lời tưởng niệm), B người đã mất nhập âm lịch, C người đang sống (sinh nhật âm, 2 sự kiện), D người đang sống → đã mất (thông tin báo tử, tệp ảnh + PDF); nhắc sao lưu 60 ngày; tệp sao lưu tạo bằng nút của giao diện. Không bao giờ sửa.",
    files: Object.fromEntries(Object.entries(files).map(([n, b]) => [n, sha(Buffer.from(b))])),
  };
  writeFileSync(join(OUT, "manifest.json"), JSON.stringify(manifest, null, 2) + "\n");
  writeFileSync(EXPECTED, JSON.stringify({ fixture: `${VERSION}/pwa`, state_hash: m.hash, store_version: m.version, backup_import_state_hash: m2.hash,
    persons: m.snapshot.persons.length, events: m.snapshot.events.length, person_ids: { A: idA, D: idD } }, null, 2) + "\n");
  console.log(`ĐÃ ĐÓNG BĂNG ${OUT} — ${m.snapshot.persons.length} hồ sơ, ${m.snapshot.events.length} sự kiện, stateHash ${m.hash}; khôi phục từ tệp sao lưu → ${m2.hash}`);
} finally {
  await ctx.close(); await browser.close(); server.close();
}
