// Kiểm thử khói (e2e) cho PWA: phục vụ dist/ trên localhost (có gzip), mở Chromium headless,
// chụp ảnh 360x780 và 1280x800, kiểm tra lỗi console, service worker, chạy offline lần hai,
// đo heap JS và dung lượng truyền (gzip). Kết quả: test/metrics.json, ảnh: test/shots/.
// Mục 1b mô phỏng cập nhật: phục vụ bản A (VERSION), rồi đổi sang bản B dựng vào thư mục tạm với
// LVHV_VERSION=<A + 1 bản vá> và kiểm tra thanh nhắc, nút "Cập nhật ngay", chân trang sau khi tải lại.
// Chạy: npm run build && node test/smoke.mjs
import { execFileSync } from "node:child_process";
import { createServer } from "node:http";
import { readFileSync, existsSync, statSync, mkdirSync, mkdtempSync, rmSync, writeFileSync, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { gzipSync } from "node:zlib";
import { dirname, extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright-core";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const DIST = join(ROOT, "dist");
const SHOTS = join(ROOT, "test", "shots");
mkdirSync(SHOTS, { recursive: true });
if (!existsSync(join(DIST, "index.html"))) { console.error("Chưa có dist/. Chạy npm run build trước."); process.exit(2); }

const TYPES = { ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".css": "text/css; charset=utf-8", ".svg": "image/svg+xml", ".png": "image/png", ".woff2": "font/woff2", ".webmanifest": "application/manifest+json", ".txt": "text/plain; charset=utf-8", ".json": "application/json" };
const COMPRESS = new Set([".html", ".js", ".css", ".svg", ".webmanifest", ".txt", ".json"]);

let bytesSent = 0;
let online = true;
let serveDir = DIST; // mục 1b đổi sang bản B để mô phỏng phát hành bản mới
const served = [];
const server = createServer((req, res) => {
  const url = new URL(req.url, "http://x");
  let p = normalize(decodeURIComponent(url.pathname)).replace(/^([/\\])+/, "");
  if (!p || p.endsWith("/")) p += "index.html";
  const f = join(serveDir, p);
  if (!online) { req.socket.destroy(); return; }
  if (!f.startsWith(serveDir) || !existsSync(f) || statSync(f).isDirectory()) { res.writeHead(404); res.end("not found"); return; }
  let body = readFileSync(f);
  const ext = extname(f);
  const headers = { "content-type": TYPES[ext] ?? "application/octet-stream", "cache-control": "no-cache" };
  if (COMPRESS.has(ext) && /gzip/.test(req.headers["accept-encoding"] ?? "")) { body = gzipSync(body, { level: 9 }); headers["content-encoding"] = "gzip"; }
  headers["content-length"] = body.length;
  bytesSent += body.length;
  served.push({ path: url.pathname, bytes: body.length });
  res.writeHead(200, headers);
  res.end(body);
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const ORIGIN = `http://127.0.0.1:${server.address().port}`;
const BASE = `${ORIGIN}/`;
const DAY = "#/ngay/2026-11-05";
// Tính sẵn bằng lõi thật (tsport) để chọn ngày "khó" cho bố cục.
const tsport = await import(join(ROOT, "..", "tsport", "dist", "lvhv.mjs"));
const CADAO = JSON.parse(readFileSync(join(ROOT, "..", "content", "cadao", "cadao.json"), "utf8"));
const iso = (y, m, d) => `${y}-${String(m).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
const LONG_CADAO_DAY = (() => {
  const maxLines = Math.max(...CADAO.map((c) => c.lines.length));
  const pos = CADAO.find((c) => c.lines.length === maxLines).order;
  const base = tsport.jd.jdnFromDate(2026, 1, 1);
  const off = ((pos - (base - 2451545)) % CADAO.length + CADAO.length) % CADAO.length;
  const [y, m, d] = tsport.jd.dateFromJdn(base + off);
  return iso(y, m, d);
})();
const MANY_STARS_DAY = (() => {
  let best = null, n = -1;
  for (let j = tsport.jd.jdnFromDate(2026, 1, 1); j <= tsport.jd.jdnFromDate(2026, 12, 31); j++) {
    const [y, m, d] = tsport.jd.dateFromJdn(j);
    const g = tsport.dayInfo(y, m, d).ngoc_hap;
    const k = g.sao_tot.length + g.sao_xau.length + g.nen.length + g.ky.length;
    if (k > n) { n = k; best = iso(y, m, d); }
  }
  return best;
})();

const failures = [];
const check = (ok, msg) => { if (!ok) failures.push(msg); console.log(`${ok ? "  ok " : "FAIL "} ${msg}`); };

const browser = await chromium.launch();
const metrics = { date: new Date().toISOString(), machine: `${process.platform} ${process.arch}, Node ${process.version}, Chromium ${browser.version()}`, viewports: {} };

async function newCtx(vp, opts = {}) {
  const ctx = await browser.newContext({ viewport: vp, deviceScaleFactor: opts.dpr ?? 1, colorScheme: opts.dark ? "dark" : "light", hasTouch: !!opts.touch, isMobile: !!opts.touch, locale: "vi-VN", reducedMotion: "reduce" });
  const page = await ctx.newPage();
  const errors = [];
  const external = [];
  page.on("console", (m) => { if (m.type() === "error" || m.type() === "warning") errors.push(`${m.type()}: ${m.text()}`); });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("request", (r) => { const u = r.url(); if (!u.startsWith(ORIGIN) && !u.startsWith("data:")) external.push(u); });
  return { ctx, page, errors, external };
}

// ---------- 1. Điện thoại 360x780: lần đầu, đo, SW, offline ----------
{
  bytesSent = 0; served.length = 0;
  const { ctx, page, errors, external } = await newCtx({ width: 360, height: 780 }, { touch: true, dpr: 2 });
  const cdp = await ctx.newCDPSession(page);
  await cdp.send("Network.enable");
  let cdpBytes = 0;
  cdp.on("Network.loadingFinished", (e) => { cdpBytes += e.encodedDataLength; });
  await page.goto(BASE + DAY, { waitUntil: "load" });
  await page.waitForSelector(".sheet");
  await page.evaluate(() => document.fonts.ready);
  const swOk = await page.evaluate(async () => { const r = await navigator.serviceWorker.ready; return !!r.active; });
  await page.waitForTimeout(400);
  const firstLoadGz = bytesSent;
  const firstLoadFiles = served.map((s) => s.path);
  check(swOk, "service worker đăng ký và kích hoạt");

  // Nội dung mẫu 2026-11-05
  const text = await page.innerText(".sheet");
  for (const s of ["Thứ Năm", "Tháng 11", "2026", "Bính Ngọ", "Quý Mùi", "Mậu Tuất", "Sương Giáng", "Chu Tước", "Hắc đạo", "Thu", "Dần", "Mão", "Tỵ", "Thân", "Tuất", "Hợi"]) check(text.includes(s), `tờ 2026-11-05 có "${s}"`);
  check(await page.locator(".lnum").innerText() === "27", "ngày âm 27");
  check((await page.locator(".bignum").innerText()) === "5", "số ngày lớn = 5");
  check(await page.locator(".foot .mock").count() === 0, "không có nhãn MOCK (lõi thật tsport)");
  check(await page.evaluate(() => window.__lvhv.mock === false), "window.__lvhv.mock = false");

  // Ngọc Hạp 2026-11-05 — đối chiếu trang lịch công khai (content/ngochap/rules.md §5)
  const tags = async (sel) => (await page.locator(sel).allTextContents()).map((t) => t.trim());
  const good = await tags(".details .tags.good li");
  const bad = await tags(".details .tags.bad li");
  const WANT_GOOD = ["Đại Hồng Sa", "Sát Cống", "Thiên Ân", "U Vi Tinh"];
  const WANT_BAD = ["Tam Nương", "Băng Tiêu Ngọa Hãm", "Chu Tước Hắc Đạo", "Địa Phá", "Hà Khôi (Cẩu Giảo)", "Hoang Vu", "Ngũ Hư", "Nguyệt Hình", "Tứ Thời Cô Quả"];
  const same = (a, b) => a.length === b.length && [...a].sort().every((x, i) => x === [...b].sort()[i]);
  check(same(good, WANT_GOOD), `sao tốt 2026-11-05 = ${WANT_GOOD.join(", ")} (có: ${good.join(", ")})`);
  check(same(bad, WANT_BAD), `sao xấu 2026-11-05 = ${WANT_BAD.length} sao fixture (có: ${bad.join(", ")})`);
  const xung = (await page.locator(".details .xung").textContent()).split(",").map((x) => x.trim());
  check(same(xung, ["Ất Sửu", "Đinh Hợi", "Tân Sửu", "Đinh Tỵ"]), `tuổi xung 2026-11-05 = Ất Sửu, Đinh Hợi, Tân Sửu, Đinh Tỵ (có: ${xung.join(", ")})`);
  check(text.includes("Kỵ") && (await tags(".dodont .ky li")).includes("Cưới hỏi"), "2026-11-05 kỵ cưới hỏi (Tam Nương)");
  check(await page.locator('.details a[href="#/nguon-quy-tac"]').count() === 1, "tờ lịch có liên kết Nguồn quy tắc");
  check((await page.locator(".details .srcnote").textContent()).includes("Theo Ngọc Hạp Thông Thư; tham khảo"), "ghi chú nguồn Ngọc Hạp");
  check(await page.locator(".cadao").getAttribute("data-id") === "cd0077", `ca dao 2026-11-05 = cd0077 (pos 411) (có: ${await page.locator(".cadao").getAttribute("data-id")})`);

  await page.screenshot({ path: join(SHOTS, "phone-360-sheet.png") });
  await page.screenshot({ path: join(SHOTS, "phone-360-sheet-full.png"), fullPage: true });

  // Heap
  await cdp.send("HeapProfiler.collectGarbage");
  const heap = await cdp.send("Runtime.getHeapUsage");
  // Render: chuyển 20 ngày bằng phím
  await page.evaluate(() => { window.__lvhv.renderMs.length = 0; });
  for (let i = 0; i < 20; i++) await page.keyboard.press("ArrowRight");
  const renderMs = await page.evaluate(() => window.__lvhv.renderMs.slice());
  check(renderMs.length === 20, "phím mũi tên đổi ngày (20 lần)");
  const maxRender = Math.max(...renderMs);
  check(maxRender < 16, `render tờ lịch < 16 ms (max ${maxRender.toFixed(2)} ms, bao gồm lõi tsport)`);
  const dateAfter = await page.locator(".sheet").getAttribute("data-date");
  check(dateAfter === "2026-11-25", `sau 20 lần → 2026-11-25 (${dateAfter})`);

  // Vuốt
  await page.goto(BASE + DAY);
  await page.waitForSelector(".sheet");
  await page.evaluate(() => {
    const el = document.querySelector("#main");
    const mk = (type, x) => { const t = new Touch({ identifier: 1, target: el, clientX: x, clientY: 300 }); el.dispatchEvent(new TouchEvent(type, { touches: type === "touchend" ? [] : [t], changedTouches: [t], bubbles: true })); };
    mk("touchstart", 300); mk("touchend", 80);
  });
  check(await page.locator(".sheet").getAttribute("data-date") === "2026-11-6", "vuốt trái → ngày sau");
  check(await page.locator(".cadao").getAttribute("data-id") === "cd0287", `ca dao 2026-11-06 = cd0287 (select.md: pos 412) (có: ${await page.locator(".cadao").getAttribute("data-id")})`);
  check((await page.locator(".cadao blockquote").innerText()).includes("Một lượt cỏ thêm giỏ thóc"), "ca dao 2026-11-06 hiển thị đúng câu");

  // Chi tiết + Xem thêm (dùng 2026-11-06: 11 việc nên làm → có nút Xem thêm)
  await page.goto(BASE + "#/ngay/2026-11-06");
  await page.waitForSelector(".sheet");
  await page.click(".details summary");
  const more = page.locator(".dodont .linkbtn").first();
  if (await more.count()) await more.click();
  await page.screenshot({ path: join(SHOTS, "phone-360-details-full.png"), fullPage: true });

  // Bảng tháng
  await page.click(".band-btn");
  await page.waitForSelector("dialog[open] .mgrid");
  await page.screenshot({ path: join(SHOTS, "phone-360-month.png") });
  await page.selectOption("#dlg-m", "1");
  await page.fill("#dlg-y", "2027");
  await page.dispatchEvent("#dlg-y", "change");
  await page.click('dialog[open] .mcell[data-d="15"]');
  check(await page.locator(".sheet").getAttribute("data-date") === "2027-1-15", "bảng tháng nhảy tới 15/1/2027");
  // Bảng tháng: năm 1899 → thông báo; 1968 → tháng Giêng có mùng 1 Tết ngày 29.
  await page.click(".band-btn");
  await page.waitForSelector("dialog[open] .mgrid");
  await page.fill("#dlg-y", "1899");
  await page.dispatchEvent("#dlg-y", "change");
  const mmsg = await page.locator("#dlg-ymsg").innerText().catch(() => "");
  check(/hỗ trợ từ năm 1900 đến năm 3000/.test(mmsg), `bảng tháng: năm 1899 → thông báo (${mmsg})`);
  await page.selectOption("#dlg-m", "1");
  await page.fill("#dlg-y", "1968");
  await page.dispatchEvent("#dlg-y", "change");
  const c29 = await page.locator('dialog[open] .mcell[data-d="29"] .ld').innerText();
  check(c29 === "1/1", `bảng tháng 1/1968: ngày 29 là mùng 1/1 (${c29})`);
  await page.keyboard.press("Escape");

  // Chọn ngày tốt
  await page.goto(BASE + "#/chon-ngay");
  await page.click('.actbtn[data-id="cuoi_hoi"]');
  await page.click('.seg button[data-k="3m"]');
  // C-2: ngày sinh dương lịch → năm âm (15/01/1976 là trước Tết 1976 → Ất Mão, không phải Bính Thìn).
  await page.fill("#gd-bd", "15");
  await page.fill("#gd-bm", "1");
  await page.fill("#gd-by", "1976");
  await page.dispatchEvent("#gd-by", "change");
  const ccTxt = await page.locator("#gd-birth-cc").innerText();
  check(/Tuổi Ất Mão/.test(ccTxt) && /1975/.test(ccTxt), `ngày sinh 15/01/1976 → tuổi Ất Mão, năm âm 1975 (${ccTxt})`);
  const sumTxt = await page.locator(".rsum").innerText();
  check(/tuổi Ất Mão/.test(sumTxt), `tóm tắt dùng tuổi theo năm âm (${sumTxt})`);
  // Khoảng tự chọn ngoài phạm vi → thông báo, không để trống.
  await page.click('.seg button[data-k="custom"]');
  await page.fill("#gd-from", "1899-12-01");
  await page.dispatchEvent("#gd-from", "change");
  const gdMsg = await page.locator(".results .warn").innerText().catch(() => "");
  check(/hỗ trợ từ 01\/01\/1900 đến 31\/12\/3000/.test(gdMsg), `chọn ngày tốt: khoảng bắt đầu 1899 → thông báo (${gdMsg})`);
  await page.fill("#gd-from", "1968-01-01");
  await page.fill("#gd-to", "1968-03-31");
  await page.dispatchEvent("#gd-to", "change");
  const n1968 = await page.locator(".gditem").count();
  check(n1968 > 0, `chọn ngày tốt 01/01–31/03/1968 có kết quả (${n1968})`);
  await page.click('.seg button[data-k="3m"]');
  // Lựa chọn "Tôi chỉ biết năm âm lịch / Can Chi".
  await page.check("#gd-lunar-only");
  await page.fill("#gd-bly", "1976");
  await page.dispatchEvent("#gd-bly", "change");
  const ccTxt2 = await page.locator("#gd-birth-cc").innerText();
  check(/Tuổi Bính Thìn/.test(ccTxt2), `năm âm 1976 → Bính Thìn (${ccTxt2})`);
  await page.uncheck("#gd-lunar-only");
  await page.dispatchEvent("#gd-by", "change");
  const scoreLbl = await page.locator(".gd-score small").first().innerText().catch(() => "");
  check(/điểm việc/.test(scoreLbl), `điểm trong danh sách ghi rõ là điểm cho việc (${scoreLbl})`);
  const nRes = await page.locator(".gditem").count();
  check(nRes > 0, `chọn ngày tốt có kết quả (${nRes})`);
  const nActs = await page.locator(".actbtn").count();
  check(nActs === 16, `danh sách việc thật từ lõi (16 việc) (${nActs})`);
  const firstReasons = await page.locator(".gd-why").first().innerText().catch(() => "");
  check(!/Phạm sao kỵ chính/.test(firstReasons), "kết quả không chứa ngày phạm sao kỵ chính");
  await page.screenshot({ path: join(SHOTS, "phone-360-gooddays.png"), fullPage: true });
  await page.locator(".gditem").first().click();
  check(await page.locator(".sheet").count() === 1, "chạm kết quả → mở tờ lịch");

  // Đổi ngày
  await page.goto(BASE + "#/doi-ngay");
  await page.waitForSelector("#cv-go-a");
  const outA = () => page.locator(".conv-out").first().innerText();
  const outB = () => page.locator(".conv-out").nth(1).innerText();
  // Gõ ba ô Ngày / Tháng / Năm rồi bấm nút (nút luôn tính lại).
  const solar = async (d, m, y) => {
    await page.fill("#cv-sd", String(d)); await page.fill("#cv-sm", String(m)); await page.fill("#cv-sy", String(y));
    await page.click("#cv-go-a");
    return outA();
  };
  const lunar = async (d, m, y, leap = false) => {
    await page.selectOption("#cv-ld", String(d)); await page.selectOption("#cv-lm", String(m)); await page.fill("#cv-ly", String(y));
    if ((await page.isChecked("#cv-leap")) !== leap) await page.click("#cv-leap");
    await page.click("#cv-go-b");
    return outB();
  };
  const RANGE = "Ứng dụng hỗ trợ từ 01/01/1900 đến 31/12/3000";
  let cv = await solar(5, 11, 2026);
  check(cv.includes("Ngày 27 tháng 9 năm Bính Ngọ"), `đổi dương → âm 05/11/2026 → "Ngày 27 tháng 9 năm Bính Ngọ" (${cv.split("\n")[0]})`);
  cv = await solar(8, 11, 1977);
  check(cv.includes("Ngày 27 tháng 9 năm Đinh Tỵ") && cv.includes("Ngày Kỷ Tỵ · tháng Canh Tuất"), `gõ 8/11/1977 + "Đổi sang âm lịch" → "Ngày 27 tháng 9 năm Đinh Tỵ", ngày Kỷ Tỵ, tháng Canh Tuất (${cv.split("\n").slice(0, 2).join(" | ")})`);
  cv = await solar(29, 1, 1968);
  check(cv.includes("Ngày 1 tháng 1 năm Mậu Thân"), `29/01/1968 → mùng 1 tháng 1 Mậu Thân (Tết miền Bắc) (${cv.split("\n")[0]})`);
  cv = await solar(28, 1, 1968);
  check(cv.includes("Ngày 29 tháng 12 năm Đinh Mùi"), `28/01/1968 → 29 tháng Chạp Đinh Mùi (${cv.split("\n")[0]})`);
  // Tết Giáp Ngọ 1954 = 03/02/1954 (Đài Thiên văn Hồng Kông, lịch 1954; trước 1968 lịch Việt tính theo UTC+8).
  cv = await solar(3, 2, 1954);
  check(cv.includes("Ngày 1 tháng 1 năm Giáp Ngọ"), `03/02/1954 → mùng 1 Tết Giáp Ngọ (${cv.split("\n")[0]})`);
  // Tết Bính Thìn 1916 = 03/02/1916 (HKO lịch 1916; trước 1929 tính theo giờ Bắc Kinh).
  cv = await solar(3, 2, 1916);
  check(cv.includes("Ngày 1 tháng 1 năm Bính Thìn"), `03/02/1916 → mùng 1 Tết Bính Thìn (${cv.split("\n")[0]})`);
  cv = await solar(1, 1, 1900);
  check(cv.includes("Ngày 1 tháng 12 năm Kỷ Hợi"), `01/01/1900 (đầu phạm vi) → 1 tháng Chạp Kỷ Hợi (${cv.split("\n")[0]})`);
  cv = await solar(31, 12, 1899);
  check(cv.includes(RANGE), `31/12/1899 → thông báo phạm vi (${cv})`);
  cv = await solar(1, 1, 3001);
  check(cv.includes(RANGE), `01/01/3001 → thông báo phạm vi (${cv})`);
  cv = await solar(31, 2, 2026);
  check(/Ngày không hợp lệ/.test(cv), `31/02/2026 → báo ngày không hợp lệ (${cv})`);
  cv = await solar("", 2, 2026);
  check(/Hãy nhập đủ/.test(cv), `ô ngày trống → nhắc nhập đủ (${cv})`);
  // Gõ (không bấm nút) cũng cập nhật.
  await page.fill("#cv-sd", "5"); await page.fill("#cv-sm", "11"); await page.fill("#cv-sy", "2026");
  check((await outA()).includes("Ngày 27 tháng 9 năm Bính Ngọ"), "gõ tay cũng tự cập nhật kết quả");
  let cv2 = await lunar(1, 1, 2026);
  check(cv2.includes("ngày 17/02/2026") && /^Thứ /.test(cv2), `đổi âm → dương 1/1/2026 → "Thứ …, ngày 17/02/2026" (${cv2.split("\n")[0]})`);
  cv2 = await lunar(27, 9, 1977);
  check(cv2.includes("Thứ Ba, ngày 08/11/1977"), `27/9/1977 âm → "Thứ Ba, ngày 08/11/1977" (${cv2.split("\n")[0]})`);
  cv2 = await lunar(1, 1, 1968);
  check(cv2.includes("ngày 29/01/1968"), `1/1 Mậu Thân → 29/01/1968 (${cv2.split("\n")[0]})`);
  cv2 = await lunar(1, 5, 2026, true);
  check(/không có tháng 5 nhuận/.test(cv2), `tháng 5 nhuận 2026 không tồn tại → thông báo (${cv2})`);
  const short = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12].find((m) => tsport.lunarToSolar(2026, m, 30, false) === null);
  cv2 = await lunar(30, short, 2026);
  check(/không có ngày 30/.test(cv2), `ngày 30 tháng ${short} (tháng thiếu) 2026 → thông báo (${cv2})`);
  cv2 = await lunar(1, 1, 1899);
  check(cv2.includes(RANGE), `tháng Giêng 1899 âm (trước 1900) → thông báo phạm vi (${cv2})`);
  cv2 = await lunar(1, 1, 3001);
  check(cv2.includes(RANGE), `năm âm 3001 → thông báo phạm vi (${cv2})`);
  const note = await page.locator(".tz-note").innerText();
  check(note.includes("Trước năm 1929, âm lịch tính theo giờ địa phương Bắc Kinh"), "Đổi ngày có ghi chú múi giờ (Bắc Kinh trước 1929, UTC+8 1929–1967, UTC+7 từ 1968)");
  await solar(8, 11, 1977); await lunar(27, 9, 1977);
  await page.screenshot({ path: join(SHOTS, "phone-360-convert.png"), fullPage: true });

  // Bảng tháng: năm ngoài phạm vi → thông báo; 1968 xem được.
  await page.goto(BASE + "#/ngay/1968-01-29");
  await page.waitForSelector(".sheet");
  const sh1968 = await page.locator(".sheet").innerText();
  check(/Mậu Thân/.test(sh1968), "tờ lịch 29/01/1968 hiện năm Mậu Thân");

  await page.goto(BASE + "#/gioi-thieu");
  await page.waitForSelector("#nguon-ca-dao");
  check((await page.innerText("main")).includes("Trước năm 1929, âm lịch tính theo giờ địa phương Bắc Kinh"), "Giới thiệu có ghi chú múi giờ trước 1968");
  const about = await page.innerText("#nguon-ca-dao");
  for (const s of ["CC BY-SA 4.0", "Wikiquote tiếng Việt", "Huỳnh Tịnh Của", "phạm vi công cộng", "Clevix Lab"]) check(about.includes(s), `Giới thiệu ghi công ca dao có "${s}"`);
  check(await page.locator('img[src*="wikimedia" i], img[alt*="wikimedia" i]').count() === 0, "không dùng logo Wikimedia");
  await page.screenshot({ path: join(SHOTS, "phone-360-about.png"), fullPage: true });
  // Liên kết nằm dưới thanh tab cố định khi cuộn tới; kích hoạt như bàn phím/trình đọc màn hình.
  await page.locator('#main a[href="#/nguon-quy-tac"]').evaluate((a) => a.click());
  await page.waitForSelector(".srclist");
  const nSrc = await page.locator(".srcitem").count();
  check(nSrc >= 27, `trang Nguồn quy tắc liệt kê nguồn từ content/ngochap/SOURCES.md (${nSrc})`);
  check((await page.innerText("main")).includes("N01") && (await page.innerText("main")).includes("Tân Việt & Thiều Phong"), "Nguồn quy tắc có N01 (Tân Việt & Thiều Phong)");
  await page.screenshot({ path: join(SHOTS, "phone-360-rules.png"), fullPage: true });
  const ovRules = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  check(ovRules <= 0, `Nguồn quy tắc không tràn ngang ở 360 px (${ovRules}px)`);

  // Offline: tắt máy chủ, tải lại
  online = false;
  await ctx.setOffline(true);
  await page.goto(BASE + DAY, { waitUntil: "load" }).catch((e) => errors.push("offline goto: " + e.message));
  const offOk = await page.locator(".sheet .bignum").count().catch(() => 0);
  check(offOk === 1, "tải lại khi không có mạng vẫn hiện tờ lịch (P5)");
  await page.screenshot({ path: join(SHOTS, "phone-360-offline.png") });
  await ctx.setOffline(false);
  online = true;

  // Lưu trữ: chỉ khóa tùy chọn giao diện
  const ls = await page.evaluate(() => Object.keys(localStorage));
  check(ls.every((k) => k === "lvhv.ui"), `localStorage chỉ có tùy chọn giao diện (${JSON.stringify(ls)})`);
  // S4: ngày sinh đã nhập (15/01/1976) không nằm trong bất kỳ kho lưu trữ nào.
  const stores = await page.evaluate(async () => {
    const ls = JSON.stringify(Object.fromEntries(Object.entries(localStorage)));
    const ss = JSON.stringify(Object.fromEntries(Object.entries(sessionStorage)));
    const idb = indexedDB.databases ? (await indexedDB.databases()).map((d) => d.name) : [];
    return { ls, ss, cookie: document.cookie, idb };
  });
  check(!/1976|1975|Ất Mão/.test(stores.ls + stores.ss + stores.cookie) && stores.ss === "{}" && stores.cookie === "" && stores.idb.length === 0,
    `ngày sinh không được lưu (${JSON.stringify(stores)})`);

  const realErrors = errors.filter((e) => !/offline goto|net::ERR_INTERNET_DISCONNECTED|Failed to load resource/.test(e));
  check(realErrors.length === 0, `không có lỗi console (${realErrors.join(" | ") || "0"})`);
  check(external.length === 0, `không gọi mạng ngoài (${external.length})`);

  metrics.viewports["360x780"] = {
    first_load_transfer_gzip_bytes: firstLoadGz,
    first_load_transfer_gzip_kb: +(firstLoadGz / 1024).toFixed(1),
    first_load_cdp_encoded_bytes: cdpBytes,
    first_load_files_page_and_sw_precache: firstLoadFiles.length,
    first_load_paths: firstLoadFiles,
    js_heap_used_bytes: heap.usedSize,
    js_heap_used_mb: +(heap.usedSize / 1048576).toFixed(2),
    js_heap_total_mb: +(heap.totalSize / 1048576).toFixed(2),
    render_ms_max: +maxRender.toFixed(2),
    render_ms_median: +renderMs.slice().sort((a, b) => a - b)[Math.floor(renderMs.length / 2)].toFixed(2),
    service_worker_active: swOk,
    offline_second_visit: offOk === 1,
    console_errors: realErrors,
    external_requests: external,
  };
  await ctx.close();
}

// ---------- 1c. Lưu trữ và sao lưu v0.2 (đợt 2): tải lười, CSP không đổi, IndexedDB, Argon2id trong Chromium ----------
const LAZY = JSON.parse(readFileSync(join(DIST, "lazy-manifest.json"), "utf8"));
{
  // P6: lần tải đầu không có tệp lưu trữ/mật mã (danh sách tệp phục vụ ở mục 1 + precache của SW).
  const sw = readFileSync(join(DIST, "sw.js"), "utf8");
  const pre = JSON.parse(/const PRECACHE = (\[[^\]]*\]);/.exec(sw)[1]);
  check(LAZY.lazy_js.length >= 2 && LAZY.lazy_js.every((f) => !pre.includes("./" + f)), `P6: ${LAZY.lazy_js.length} tệp tải lười không nằm trong precache`);
  check(LAZY.lazy_js.every((f) => !metrics.viewports["360x780"].first_load_paths.includes("/" + f)), "P6: lần tải đầu không yêu cầu tệp lưu trữ/mật mã");

  bytesSent = 0; served.length = 0;
  const { ctx, page, errors, external } = await newCtx({ width: 360, height: 780 }, { touch: true });
  const csp = [];
  page.on("console", (m) => { if (/Content.Security.Policy|CSP/i.test(m.text())) csp.push(m.text()); });
  await page.goto(BASE + DAY, { waitUntil: "load" });
  await page.waitForSelector(".sheet");
  await page.evaluate(() => navigator.serviceWorker.ready);
  const before = served.length;
  const PHOTO = [...readFileSync(join(ROOT, "..", "docs/v0.2/schema/test-vectors/photo-0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e09.jpg"))];
  const EX = readFileSync(join(ROOT, "..", "docs/v0.2/schema/test-vectors/example-v1.json"), "utf8");
  const RUST_FILE = [...readFileSync(join(ROOT, "..", "docs/v0.2/schema/test-vectors/example-v1.lvhv"))];
  const r = await page.evaluate(async ({ PHOTO, EX, RUST_FILE }) => {
    const st = window.__lvhv.storage;
    const due0 = await st.backupReminderDue();
    const M = await st.loadStore();
    const s = await M.openStore();
    const ex = JSON.parse(EX);
    const photo = new Uint8Array(PHOTO);
    const p = M.buildPerson(ex.persons[0]);
    await s.putPerson(p, { id: p.photo_id, mime: "image/jpeg", width: 768, height: 1024, bytes: photo });
    const persist = await s.persistResult();
    const h1 = await s.stateHash();
    let t = performance.now();
    const file = await s.exportBackup("mật khẩu thử nghiệm", { version: "0.2.0", platform: "pwa" });
    const exportMs = performance.now() - t;
    const s2 = await M.openStore(); // cùng kho; dùng để nhập lại "Thay toàn bộ" sau khi xóa
    await s2.deletePerson(p.id);
    t = performance.now();
    const v = await M.openBackup(file, "mật khẩu thử nghiệm");
    const importMs = performance.now() - t;
    await s2.applyImport(v, "replace");
    const h2 = await s2.stateHash();
    let wrong = "";
    try { await M.openBackup(file, "mật khẩu sai"); } catch (e) { wrong = e.message; }
    const fromRust = await M.openBackup(new Uint8Array(RUST_FILE), "mật khẩu thử nghiệm");
    const dbs = (await indexedDB.databases()).map((d) => d.name);
    return { due0, persist, same: h1 === h2, exportMs, importMs, wrong, rustPersons: fromRust.preview.persons, dbs, ls: Object.keys(localStorage) };
  }, { PHOTO, EX, RUST_FILE });
  const lazyServed = served.slice(before).map((x) => x.path);
  check(r.due0 === false, "backupReminderDue() khi chưa có dữ liệu = false (không tạo IndexedDB)");
  check(r.same, "Chromium: putPerson + ảnh → xuất → nhập Thay toàn bộ → giống hệt (stateHash)");
  check(r.wrong === "Sai mật khẩu hoặc tệp bị hỏng", `Chromium: sai mật khẩu → "${r.wrong}"`);
  check(r.rustPersons === 3, "Chromium: mở được tệp mẫu do Rust tạo (example-v1.lvhv)");
  check(typeof r.persist === "boolean", `navigator.storage.persist() đã gọi khi lưu hồ sơ đầu tiên (kết quả ${r.persist})`);
  check(LAZY.lazy_js.every((f) => lazyServed.includes("/" + f)), `tệp tải lười được nạp khi dùng (${lazyServed.join(", ")})`);
  check(csp.length === 0, `0 vi phạm CSP khi chạy Argon2id/XChaCha20 (${csp.join(" | ") || "0"})`);
  check(r.dbs.includes("lvhv") && r.ls.every((k) => k === "lvhv.ui"), "dữ liệu chỉ ở IndexedDB 'lvhv'; localStorage chỉ lvhv.ui (S4')");
  check(errors.length === 0, `không lỗi console (lưu trữ) ${errors.join(" | ")}`);
  check(external.length === 0, "không gọi mạng ngoài (lưu trữ)");
  // Ngoại tuyến: mô-đun tải lười đã được SW cache khi nạp lần đầu → dùng được khi mất mạng.
  online = false;
  await ctx.setOffline(true);
  await page.goto(BASE + DAY, { waitUntil: "load" });
  const off = await page.evaluate(async () => { const M = await window.__lvhv.storage.loadStore(); const s = await M.openStore(); return (await s.persons()).length; }).catch((e) => "ERR " + e.message);
  check(off === 1, `ngoại tuyến: mô-đun lưu trữ nạp từ cache, đọc được kho (${off})`);
  await ctx.setOffline(false);
  online = true;
  const lazyGz = LAZY.lazy_js.reduce((n, f) => n + gzipSync(readFileSync(join(DIST, f)), { level: 9 }).length, 0);
  metrics.storage_v02 = {
    lazy_js: LAZY.lazy_js, lazy_js_gzip_bytes: lazyGz, lazy_js_gzip_kb: +(lazyGz / 1024).toFixed(1),
    argon2id_export_ms_chromium: Math.round(r.exportMs), argon2id_import_ms_chromium: Math.round(r.importMs),
    persist_granted_headless: r.persist, roundtrip_identical: r.same, csp_violations: csp,
  };
  await ctx.close();
}

// ---------- 1b. Phiên bản và cập nhật (bản A → bản B), ngoại tuyến ----------
const VERSION_A = readFileSync(join(ROOT, "..", "VERSION"), "utf8").trim();
const VERSION_B = VERSION_A.replace(/(\d+)$/, (x) => String(+x + 1));
const TMP_B = mkdtempSync(join(tmpdir(), "lvhv-update-"));
const DIST_B = join(TMP_B, "dist");
{
  check(JSON.parse(readFileSync(join(DIST, "version.json"), "utf8")).version === VERSION_A, `dist/version.json = VERSION (${VERSION_A})`);
  const vjA = JSON.parse(readFileSync(join(DIST, "version.json"), "utf8"));
  check(Array.isArray(vjA.notes) && vjA.notes.length > 0 && /^\d{4}-\d{2}-\d{2}$/.test(vjA.date) && vjA.build.length >= 7, `version.json có build, date, notes (${vjA.build}, ${vjA.date}, ${vjA.notes.length} ghi chú)`);
  check(!readFileSync(join(DIST, "sw.js"), "utf8").includes('"./version.json"]') && !/PRECACHE = \[[^\]]*version\.json/.test(readFileSync(join(DIST, "sw.js"), "utf8")), "version.json không nằm trong precache");
  execFileSync(process.execPath, [join(ROOT, "build.mjs")], { cwd: ROOT, env: { ...process.env, LVHV_VERSION: VERSION_B, LVHV_OUT_DIR: DIST_B }, stdio: "ignore" });
  check(JSON.parse(readFileSync(join(DIST_B, "version.json"), "utf8")).version === VERSION_B, `bản B dựng với LVHV_VERSION=${VERSION_B}`);

  serveDir = DIST;
  const { ctx, page, errors, external } = await newCtx({ width: 360, height: 780 }, { touch: true, dpr: 2 });
  await page.goto(BASE + DAY, { waitUntil: "load" });
  await page.waitForSelector(".sheet");
  await page.evaluate(async () => { await navigator.serviceWorker.ready; });
  await page.waitForFunction(() => !!navigator.serviceWorker.controller);
  await page.waitForTimeout(300);
  check(((t) => t.includes(`© 2026 Clevix Lab`) && t.includes(`Phiên bản ${VERSION_A}`))((await page.innerText(".foot")).replace(/\u00a0/g, " ")), `chân trang: "© 2026 Clevix Lab / Phiên bản ${VERSION_A}"`);
  check(await page.locator(".update-bar").isHidden(), "bản A, máy chủ cũng A → không có thanh cập nhật");

  await page.click('.sizes button[data-v="3"]'); // A++
  await page.goto(BASE + "#/gioi-thieu");
  await page.waitForSelector("#check-update");
  const cur = await page.innerText(".cur-ver");
  check(cur.startsWith(`Phiên bản đang dùng: ${VERSION_A} (bản dựng ${vjA.build}, ngày ${vjA.date})`), `Giới thiệu: "${cur}"`);
  check((await page.innerText("main")).includes("mô hình thiên văn chính xác (Meeus)") && (await page.innerText("main")).includes("Hồ Ngọc Đức"), "Giới thiệu mô tả phương pháp Meeus, giữ HND để đối chiếu");
  await page.click("#check-update");
  await page.waitForFunction(() => /phiên bản mới nhất|ngoại tuyến|Có phiên bản/.test(document.querySelector(".upd-out")?.textContent ?? ""));
  check((await page.innerText(".upd-out")) === "Bạn đang dùng phiên bản mới nhất.", `Kiểm tra cập nhật (cùng bản) → "Bạn đang dùng phiên bản mới nhất." (${await page.innerText(".upd-out")})`);

  // Ngoại tuyến
  online = false;
  await ctx.setOffline(true);
  await page.click("#check-update");
  await page.waitForFunction(() => /ngoại tuyến|mới nhất|Có phiên bản/.test(document.querySelector(".upd-out")?.textContent ?? ""));
  check((await page.innerText(".upd-out")) === "Không kiểm tra được (đang ngoại tuyến).", `Kiểm tra cập nhật khi mất mạng → thông báo ngoại tuyến (${await page.innerText(".upd-out")})`);
  await ctx.setOffline(false);
  online = true;

  // Phát hành bản B
  serveDir = DIST_B;
  await page.click("#check-update");
  await page.waitForSelector(".update-bar:not([hidden])", { timeout: 15000 });
  const bar = await page.innerText(".update-bar");
  check(bar.includes(`Đã có phiên bản mới ${VERSION_B}.`) && bar.includes("Cập nhật ngay") && bar.includes("Để sau"), `thanh cập nhật: "${bar.replace(/\n/g, " ")}"`);
  check((await page.innerText(".upd-out")).includes(`Có phiên bản mới ${VERSION_B}`) && await page.locator("#apply-update").count() === 1, "Giới thiệu: có phiên bản mới + nút Cập nhật ngay");
  check(await page.locator(".update-bar").getAttribute("role") === "status", "thanh cập nhật role=status");
  await page.locator(".update-bar .ubtn").first().focus();
  check(await page.evaluate(() => document.activeElement?.textContent === "Cập nhật ngay"), "nút Cập nhật ngay nhận focus bàn phím");
  const ovBar = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  check(ovBar <= 0, `A++ 360 px: thanh cập nhật không tràn ngang (${ovBar}px)`);
  await page.screenshot({ path: join(SHOTS, "phone-360-xl-update.png") });

  // Tự nhắc khi mở lại ứng dụng (bản B đang chờ, trang vẫn chạy bản A)
  await page.goto(BASE + DAY);
  await page.reload({ waitUntil: "load" });
  await page.waitForSelector(".update-bar:not([hidden])", { timeout: 15000 });
  check((await page.innerText(".update-bar")).includes(VERSION_B), "mở lại ứng dụng → tự nhắc có phiên bản mới (không cần bấm)");
  check((await page.innerText(".foot")).includes(`Phiên bản ${VERSION_A}`), `trước khi đồng ý vẫn chạy bản ${VERSION_A} (RS-08)`);
  await page.click(".update-bar .ubtn:not(.primary)");
  check(await page.locator(".update-bar").isHidden(), "Để sau → ẩn thanh");

  // Cập nhật ngay → kích hoạt SW mới, tải lại một lần
  await page.goto(BASE + "#/gioi-thieu");
  await page.waitForSelector("#check-update");
  await page.click("#check-update");
  await page.waitForSelector(".update-bar:not([hidden])", { timeout: 15000 });
  await page.click(".update-bar .ubtn.primary");
  await page.waitForFunction((v) => document.querySelector(".foot")?.textContent?.includes(`Phiên bản ${v}`), VERSION_B, { timeout: 30000 });
  check(((t) => t.includes(`© 2026 Clevix Lab`) && t.includes(`Phiên bản ${VERSION_B}`))((await page.innerText(".foot")).replace(/\u00a0/g, " ")), `sau Cập nhật ngay → chân trang "Phiên bản ${VERSION_B}"`);
  await page.waitForTimeout(500);
  check(await page.locator(".update-bar").isHidden(), "sau cập nhật không còn thanh nhắc");
  if (await page.locator("#check-update").count() === 0) { await page.goto(BASE + "#/gioi-thieu"); await page.waitForSelector("#check-update"); }
  await page.click("#check-update");
  await page.waitForFunction(() => /phiên bản mới nhất|ngoại tuyến|Có phiên bản/.test(document.querySelector(".upd-out")?.textContent ?? ""));
  check((await page.innerText(".upd-out")) === "Bạn đang dùng phiên bản mới nhất.", "bản B: Kiểm tra cập nhật → mới nhất");
  const st = await page.evaluate(() => ({ ls: Object.keys(localStorage), ss: Object.keys(sessionStorage), cookie: document.cookie }));
  check(st.ls.every((k) => k === "lvhv.ui") && st.ss.length === 0 && st.cookie === "", `cập nhật không ghi dữ liệu xuống máy (S4) (${JSON.stringify(st)})`);
  const realErrors = errors.filter((e) => !/net::ERR_INTERNET_DISCONNECTED|net::ERR_FAILED|Failed to load resource|FetchEvent|network error/.test(e));
  check(realErrors.length === 0, `cập nhật: không có lỗi console (${realErrors.join(" | ") || "0"})`);
  check(external.length === 0, `cập nhật: không gọi mạng ngoài (${external.length})`);
  metrics.update_simulation = { from: VERSION_A, to: VERSION_B, banner: bar.replace(/\n/g, " "), pass: !failures.length };
  await ctx.close();
  serveDir = DIST;
}
rmSync(TMP_B, { recursive: true, force: true });

// ---------- 2. Điện thoại, chữ A++ và giao diện tối ----------
{
  const { ctx, page, errors } = await newCtx({ width: 360, height: 780 }, { touch: true, dpr: 2, dark: true });
  await page.goto(BASE + DAY);
  await page.waitForSelector(".sheet");
  await page.screenshot({ path: join(SHOTS, "phone-360-dark.png"), fullPage: true });
  await page.click('.sizes button[data-v="3"]');
  await page.screenshot({ path: join(SHOTS, "phone-360-dark-xl.png"), fullPage: true });
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  check(overflow <= 0, `A++ ở 360 px không tràn ngang (${overflow}px)`);
  // Ngày có ca dao dài nhất (6 dòng) và ngày nhiều sao nhất trong 2026, ở cỡ chữ A++
  for (const [d, label, slug] of [[LONG_CADAO_DAY, "ca dao 6 dòng", "long-cadao"], [MANY_STARS_DAY, "nhiều sao", "many-stars"]]) {
    await page.goto(BASE + "#/ngay/" + d);
    await page.waitForSelector(".sheet");
    await page.click(".details summary");
    const ov = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    check(ov <= 0, `A++ ${label} (${d}) không tràn ngang (${ov}px)`);
    await page.screenshot({ path: join(SHOTS, `phone-360-xl-${slug}.png`), fullPage: true });
  }
  // Đổi ngày ở A++ 360 px: ba ô số, nút lớn, kết quả 8/11/1977.
  await page.goto(BASE + "#/doi-ngay");
  await page.waitForSelector("#cv-go-a");
  await page.fill("#cv-sd", "8"); await page.fill("#cv-sm", "11"); await page.fill("#cv-sy", "1977");
  await page.click("#cv-go-a");
  const xlCv = await page.locator(".conv-out").first().innerText();
  check(xlCv.includes("Ngày 27 tháng 9 năm Đinh Tỵ"), `A++ Đổi ngày 8/11/1977 → Ngày 27 tháng 9 năm Đinh Tỵ`);
  const ovCv = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  check(ovCv <= 0, `A++ Đổi ngày không tràn ngang ở 360 px (${ovCv}px)`);
  await page.screenshot({ path: join(SHOTS, "phone-360-xl-convert.png"), fullPage: true });
  await page.click('.sizes button[data-v="1"]');
  check(errors.length === 0, `không lỗi console (tối) ${errors.join(" | ")}`);
  await ctx.close();
}

// ---------- 3. Máy tính 1280x800 ----------
{
  bytesSent = 0;
  const { ctx, page, errors, external } = await newCtx({ width: 1280, height: 800 });
  const cdp = await ctx.newCDPSession(page);
  await page.goto(BASE + DAY, { waitUntil: "load" });
  await page.waitForSelector(".side .mgrid");
  await page.evaluate(() => document.fonts.ready);
  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.waitForTimeout(300);
  const gz = bytesSent;
  await page.screenshot({ path: join(SHOTS, "desktop-1280-sheet.png") });
  await page.screenshot({ path: join(SHOTS, "desktop-1280-sheet-full.png"), fullPage: true });
  await cdp.send("HeapProfiler.collectGarbage");
  const heap = await cdp.send("Runtime.getHeapUsage");
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  check(overflow <= 0, "máy tính không tràn ngang");
  await page.goto(BASE + "#/chon-ngay");
  await page.click('.actbtn[data-id="khai_truong"]');
  await page.screenshot({ path: join(SHOTS, "desktop-1280-gooddays.png") });
  await page.goto(BASE + "#/nguon-quy-tac");
  await page.waitForSelector(".srclist");
  await page.screenshot({ path: join(SHOTS, "desktop-1280-rules.png") });
  await page.emulateMedia({ colorScheme: "dark" });
  await page.goto(BASE + DAY);
  await page.waitForSelector(".side .mgrid");
  await page.screenshot({ path: join(SHOTS, "desktop-1280-dark.png") });
  check(errors.length === 0, `không lỗi console (máy tính) ${errors.join(" | ")}`);
  check(external.length === 0, "không gọi mạng ngoài (máy tính)");
  metrics.viewports["1280x800"] = {
    first_load_transfer_gzip_bytes: gz,
    first_load_transfer_gzip_kb: +(gz / 1024).toFixed(1),
    js_heap_used_bytes: heap.usedSize,
    js_heap_used_mb: +(heap.usedSize / 1048576).toFixed(2),
    console_errors: errors,
    external_requests: external,
  };
  await ctx.close();
}

// ---------- Tổng hợp ----------
const files = (function list(d) { return readdirSync(d).flatMap((n) => { const p = join(d, n); return statSync(p).isDirectory() ? list(p) : [p]; }); })(DIST);
let distRaw = 0, distGz = 0;
for (const f of files) { const b = readFileSync(f); distRaw += b.length; distGz += COMPRESS.has(extname(f)) ? gzipSync(b, { level: 9 }).length : b.length; }
const html = readFileSync(join(DIST, "index.html"), "utf8");
const csp = /http-equiv="Content-Security-Policy" content="([^"]+)"/.exec(html)?.[1] ?? "";
const inlineScript = /<script(?![^>]*\bsrc=)[^>]*>/.test(html);
check(csp.includes("default-src 'self'") && !csp.includes("unsafe"), "CSP chặt (default-src 'self', không unsafe-*)");
// S2b (docs/v0.2/04): CSP giống từng byte bản 0.1.1 (O-7: mật mã JS, không ngoại lệ 'wasm-unsafe-eval').
const CSP_011 = "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; manifest-src 'self'; worker-src 'self'; object-src 'none'; base-uri 'self'; form-action 'none'";
check(csp === CSP_011, "S2b: CSP giống từng byte bản 0.1.1");
check(!/WebAssembly|\.wasm/.test(files.filter((f) => f.endsWith(".js")).map((f) => readFileSync(f, "utf8")).join("\n")), "không có WebAssembly trong bản dựng");
check(!inlineScript, "không có script nội tuyến");
const allJs = files.filter((f) => f.endsWith(".js")).map((f) => readFileSync(f, "utf8")).join("\n");
// URL chỉ được phép xuất hiện dưới dạng DỮ LIỆU hiển thị (nguồn ca dao trong cadao.json, nguồn quy tắc trong
// content/ngochap/SOURCES.md, giấy phép CC trong SOURCES.md §C). Mọi URL khác trong JS → lỗi.
// Việc không gọi mạng ngoài được kiểm tra lúc chạy (external_requests = 0).
const ALLOWED_URLS = new Set([
  ...CADAO.flatMap((c) => c.sources.map((x) => x.url)),
  ...(readFileSync(join(ROOT, "..", "content", "ngochap", "SOURCES.md"), "utf8").match(/https?:\/\/[^\s|)]+/g) ?? []),
  "https://creativecommons.org/licenses/by-sa/4.0/",
]);
let stripped = allJs.replace(/http:\/\/x/g, "");
let dataUrls = 0;
for (const u of [...ALLOWED_URLS].sort((a, b) => b.length - a.length)) { const parts = stripped.split(u); dataUrls += parts.length - 1; stripped = parts.join(""); }
const strayUrls = [...new Set(stripped.match(/https?:\/\/[^\s"'`\\]+/g) ?? [])].filter((u) => !u.startsWith("http://www.w3.org/"));
check(strayUrls.length === 0, `mã JS không chứa URL ngoài danh mục nguồn (${dataUrls} URL dữ liệu; lạ: ${strayUrls.slice(0, 5).join(" ") || "0"})`);

const vp = metrics.viewports["360x780"];
metrics.criteria = {
  P3_first_load_gzip_under_500KB: { value_kb: vp.first_load_transfer_gzip_kb, pass: vp.first_load_transfer_gzip_bytes < 500 * 1024 },
  P4_js_heap_under_30MB: { value_mb: Math.max(vp.js_heap_used_mb, metrics.viewports["1280x800"].js_heap_used_mb), pass: Math.max(vp.js_heap_used_bytes, metrics.viewports["1280x800"].js_heap_used_bytes) < 30 * 1048576 },
  P5_offline_after_first_load: { pass: vp.offline_second_visit },
  S3_no_external_network_and_strict_csp: { csp, pass: vp.external_requests.length === 0 && metrics.viewports["1280x800"].external_requests.length === 0 && csp.includes("default-src 'self'") && !inlineScript },
  S2b_csp_unchanged_from_0_1_1: { pass: csp === CSP_011 },
  P6_crypto_lazy_loaded: { lazy_js_gzip_kb: metrics.storage_v02?.lazy_js_gzip_kb, pass: !failures.some((f) => f.startsWith("P6")) },
  S4_no_personal_data_stored: { note: "localStorage chỉ có khóa lvhv.ui (cỡ chữ, sáng/tối); năm sinh không lưu", pass: !failures.some((f) => f.startsWith("localStorage")) },
};
metrics.dist = { files: files.length, raw_bytes: distRaw, gzip_bytes_all_files: distGz };
const gen = readFileSync(join(ROOT, "src", "gen", "content.ts"), "utf8");
metrics.engine = /ENGINE_SOURCE[^=]*= "mock"/.test(gen) ? "mock" : "tsport";
metrics.method = /DEFAULT_METHOD: Method = "(\w+)"/.exec(gen)?.[1] ?? "?";
metrics.layout_probe_days = { long_cadao: LONG_CADAO_DAY, many_stars: MANY_STARS_DAY };
metrics.failures = failures;
metrics.pass = failures.length === 0;
writeFileSync(join(ROOT, "test", "metrics.json"), JSON.stringify(metrics, null, 2) + "\n");

await browser.close();
server.close();
console.log(`\n${failures.length ? `KHÔNG ĐẠT: ${failures.length} lỗi` : "ĐẠT"} — metrics: test/metrics.json, ảnh: test/shots/`);
process.exit(failures.length ? 1 : 0);
