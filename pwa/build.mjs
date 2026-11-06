// Build PWA → dist/ (đường dẫn tương đối, chạy trên mọi máy chủ tĩnh, kể cả GitHub Pages).
// Không phụ thuộc runtime. Chỉ dùng esbuild (devDependency).
//
// Mặc định: BẮT BUỘC có tsport/src/index.ts, content/cadao/cadao.json, content/cadao/SOURCES.md và
// content/ngochap/SOURCES.md — thiếu tệp nào thì build THẤT BẠI (mã thoát 1), không lặng lẽ dùng dữ
// liệu giả (C-10). Chỉ khi chạy `node build.mjs --mock` mới được dùng lõi giả lập/câu mẫu cho phần
// còn thiếu; giao diện khi đó có nhãn MOCK và bản build KHÔNG được phát hành.
import { build } from "esbuild";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync, copyFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = dirname(fileURLToPath(import.meta.url));
const REPO = resolve(ROOT, "..");
const DIST = join(ROOT, "dist");
const GEN = join(ROOT, "src", "gen");
const VERSION = "0.1";

const TSPORT = join(REPO, "tsport", "src", "index.ts");
const CADAO_JSON = join(REPO, "content", "cadao", "cadao.json");
const CADAO_SOURCES = join(REPO, "content", "cadao", "SOURCES.md");

const log = (...a) => console.log("[build]", ...a);
const ALLOW_MOCK = process.argv.slice(2).includes("--mock");
for (const a of process.argv.slice(2)) if (a !== "--mock") { console.error(`[build] LỖI: đối số không hợp lệ: ${a} (chỉ có --mock)`); process.exit(2); }

// ---------- 1. Nội dung sinh ra lúc build ----------
const NGOCHAP_SOURCES = join(REPO, "content", "ngochap", "SOURCES.md");
const RUST_LUNAR = join(REPO, "core", "src", "lunar.rs");
const TS_LUNAR = join(REPO, "tsport", "src", "lunar.ts");

const MOCK_CADAO = [
  ["Uống nước nhớ nguồn"],
  ["Bầu ơi thương lấy bí cùng", "Tuy rằng khác giống nhưng chung một giàn"],
  ["Công cha như núi Thái Sơn", "Nghĩa mẹ như nước trong nguồn chảy ra"],
].map((lines, i) => ({
  id: `mock${i}`, type: lines.length > 1 ? "ca dao" : "tục ngữ", text: lines.join(" / "), lines, theme: "", meaning: "",
  sources: [{ title: "MOCK — câu mẫu, chờ nội dung từ content/cadao", url: "", license: "" }], order: i,
}));

function fail(msg) { console.error("[build] LỖI:", msg); process.exit(1); }

/** Kiểm tra cadao.json theo lược đồ content/cadao/README.md, giữ nguyên tên trường, sắp theo `order`. */
function loadCadao(raw) {
  if (!Array.isArray(raw)) fail("cadao.json phải là một mảng");
  const keys = ["id", "type", "text", "lines", "theme", "meaning", "sources", "order"];
  const out = raw.map((e, i) => {
    for (const k of keys) if (!(k in e)) fail(`cadao.json mục ${i} thiếu trường "${k}"`);
    if (!Array.isArray(e.lines) || !e.lines.length || e.lines.some((l) => typeof l !== "string")) fail(`cadao.json ${e.id}: "lines" không hợp lệ`);
    if (e.text !== e.lines.join(" / ")) fail(`cadao.json ${e.id}: "text" khác các "lines" nối bằng " / "`);
    if (!Array.isArray(e.sources) || !e.sources.length) fail(`cadao.json ${e.id}: thiếu nguồn (C1)`);
    return Object.fromEntries(keys.map((k) => [k, e[k]]));
  });
  out.sort((a, b) => a.order - b.order);
  out.forEach((e, i) => { if (e.order !== i) fail(`cadao.json: "order" không phải hoán vị 0..N−1 (vị trí ${i} có order ${e.order})`); });
  return out;
}

/** Bỏ cú pháp Markdown nội dòng (liên kết → chữ, *, _, `), không sinh HTML. */
const mdClean = (s) => s.replace(/\[([^\]]+)\]\(([^)]+)\)/g, "$1").replace(/[*_`]+/g, "").trim();

/** Nội dung ghi công bắt buộc: khối trích dẫn ở SOURCES.md §C mục 1. */
function cadaoAttribution(md) {
  const i = md.indexOf("## C.");
  if (i < 0) fail("content/cadao/SOURCES.md thiếu mục C");
  const end = md.indexOf("\n## ", i + 5);
  const sec = md.slice(i, end < 0 ? undefined : end);
  const q = sec.split(/\r?\n/).filter((l) => /^\s*>/.test(l)).map((l) => l.replace(/^\s*>\s?/, "")).join(" ").trim();
  if (!q) fail("content/cadao/SOURCES.md §C không có đoạn ghi công (khối >)");
  return mdClean(q);
}

/** Bảng nguồn quy tắc Ngọc Hạp (content/ngochap/SOURCES.md): lời dẫn + bảng N.. + bảng F.. + hạn chế. */
function ngochapSources(md) {
  const rows = [];
  const intro = [];
  const limits = [];
  let sec = "intro";
  for (const line of md.split(/\r?\n/)) {
    const t = line.trim();
    if (/^##\s/.test(t)) { sec = /Hạn chế/i.test(t) ? "limits" : /đối chiếu/i.test(t) ? "check" : "other"; continue; }
    if (/^#\s/.test(t)) continue;
    const m = /^\|\s*([NF]\d{2})\s*\|(.*)\|\s*$/.exec(t);
    if (m) {
      const c = m[2].replace(/\\\|/g, "\u0000").split("|").map((x) => mdClean(x.replace(/\u0000/g, "|")));
      rows.push({ code: m[1], title: c[0] ?? "", url: c[1] ?? "", lineage: m[1].startsWith("F") ? "" : (c[2] ?? ""), usedFor: m[1].startsWith("F") ? (c[2] ?? "") : (c[3] ?? "") });
      continue;
    }
    if (sec === "intro" && t && !t.startsWith("|")) intro.push(mdClean(t));
    if (sec === "limits" && /^-\s/.test(t)) limits.push(mdClean(t.slice(2)));
    else if (sec === "limits" && t && limits.length) limits[limits.length - 1] += " " + mdClean(t);
  }
  if (!rows.some((r) => r.code === "N01")) fail("content/ngochap/SOURCES.md: không đọc được bảng nguồn");
  return { intro: intro.join(" "), rows, limits };
}

/** DEFAULT_METHOD sao chép từ lõi Rust; tsport phải cùng giá trị. LVHV_METHOD=hnd|precise để chọn khác khi build. */
function defaultMethod() {
  const m = /pub const DEFAULT_METHOD: Method = Method::(\w+);/.exec(readFileSync(RUST_LUNAR, "utf8"));
  if (!m) fail("không tìm thấy DEFAULT_METHOD trong core/src/lunar.rs");
  const rust = { Hnd: "hnd", Precise: "precise" }[m[1]];
  if (!rust) fail(`DEFAULT_METHOD lạ: ${m[1]}`);
  if (existsSync(TS_LUNAR)) {
    const t = /export const DEFAULT_METHOD: Method = "(\w+)";/.exec(readFileSync(TS_LUNAR, "utf8"));
    if (!t || t[1] !== rust) fail(`DEFAULT_METHOD của tsport (${t?.[1]}) khác lõi Rust (${rust})`);
  }
  const env = process.env.LVHV_METHOD;
  if (env && env !== "hnd" && env !== "precise") fail(`LVHV_METHOD không hợp lệ: ${env}`);
  return { method: env || rust, rust };
}

/** Thiếu tệp bắt buộc: lỗi, trừ khi có `--mock` (khi đó cảnh báo rồi dùng dữ liệu giả). */
function requireOrMock(path, what) {
  if (existsSync(path)) return true;
  if (!ALLOW_MOCK) fail(`thiếu ${what} (${relative(REPO, path)}). Không dựng bản giả lập khi chưa có cờ --mock.`);
  log(`CẢNH BÁO (--mock): thiếu ${what} — dùng dữ liệu giả, bản build KHÔNG được phát hành`);
  return false;
}

let cadao = MOCK_CADAO, cadaoMock = true;
if (requireOrMock(CADAO_JSON, "dữ liệu ca dao")) { cadao = loadCadao(JSON.parse(readFileSync(CADAO_JSON, "utf8"))); cadaoMock = false; }
const sourcesMock = !requireOrMock(CADAO_SOURCES, "danh mục nguồn ca dao");
const attribution = sourcesMock ? "Danh mục nguồn ca dao, tục ngữ sẽ được bổ sung khi bộ nội dung hoàn tất." : cadaoAttribution(readFileSync(CADAO_SOURCES, "utf8"));
const rulesSrc = requireOrMock(NGOCHAP_SOURCES, "danh mục nguồn Ngọc Hạp") ? ngochapSources(readFileSync(NGOCHAP_SOURCES, "utf8")) : { intro: "", rows: [], limits: [] };
const engineSource = requireOrMock(TSPORT, "lõi tsport") ? "tsport" : "mock";
const { method, rust: rustMethod } = defaultMethod();

mkdirSync(GEN, { recursive: true });
writeFileSync(join(GEN, "content.ts"),
  `// TỆP SINH TỰ ĐỘNG bởi build.mjs — không sửa tay.\n` +
  `import type { Cadao, Method, RuleSource } from "../types";\n` +
  `export const ENGINE_SOURCE: "tsport" | "mock" = ${JSON.stringify(engineSource)};\n` +
  `/** Mirror of Rust DEFAULT_METHOD (core/src/lunar.rs: ${rustMethod}). */\n` +
  `export const DEFAULT_METHOD: Method = ${JSON.stringify(method)};\n` +
  `export const CADAO_IS_MOCK = ${cadaoMock};\n` +
  `export const SOURCES_IS_MOCK = ${sourcesMock};\n` +
  `export const APP_VERSION = ${JSON.stringify(VERSION)};\n` +
  `/** content/cadao/cadao.json, sắp theo "order" (CADAO[i].order === i). */\n` +
  `export const CADAO: Cadao[] = ${JSON.stringify(cadao)};\n` +
  `/** content/cadao/SOURCES.md §C — nội dung ghi công bắt buộc. */\n` +
  `export const CADAO_ATTRIBUTION = ${JSON.stringify(attribution)};\n` +
  `export const RULE_SOURCES_INTRO = ${JSON.stringify(rulesSrc.intro)};\n` +
  `export const RULE_SOURCES: RuleSource[] = ${JSON.stringify(rulesSrc.rows)};\n` +
  `export const RULE_SOURCES_LIMITS: string[] = ${JSON.stringify(rulesSrc.limits)};\n`);
log(`lõi: ${engineSource}; phương pháp: ${method}${method !== rustMethod ? ` (LVHV_METHOD, lõi Rust: ${rustMethod})` : ""}; ca dao: ${cadao.length} câu${cadaoMock ? " (mock)" : ""}; nguồn Ngọc Hạp: ${rulesSrc.rows.length} dòng`);

// ---------- 2. Bundle ----------
rmSync(DIST, { recursive: true, force: true });
mkdirSync(DIST, { recursive: true });

const engineAlias = {
  name: "engine-impl",
  setup(b) {
    b.onResolve({ filter: /^#engine-impl$/ }, () => ({ path: engineSource === "tsport" ? TSPORT : join(ROOT, "src", "engine.mock.ts") }));
  },
};

const res = await build({
  entryPoints: { app: join(ROOT, "src", "main.ts"), style: join(ROOT, "src", "style.css") },
  bundle: true,
  minify: true,
  format: "esm",
  target: "es2020",
  outdir: DIST,
  entryNames: "[name]-[hash]",
  assetNames: "fonts/[name]-[hash]",
  loader: { ".woff2": "file" },
  external: [],
  plugins: [engineAlias],
  metafile: true,
  legalComments: "none",
  charset: "utf8",
  logLevel: "warning",
});

const outputs = Object.keys(res.metafile.outputs).map((p) => relative(DIST, resolve(ROOT, p)).split("\\").join("/"));
const jsFile = outputs.find((p) => /^app-.*\.js$/.test(p));
const cssFile = outputs.find((p) => /^style-.*\.css$/.test(p));
const fontFiles = outputs.filter((p) => p.endsWith(".woff2"));

// ---------- 3. Tệp tĩnh ----------
function copyDir(src, dst) {
  mkdirSync(dst, { recursive: true });
  for (const name of readdirSync(src)) {
    const s = join(src, name), d = join(dst, name);
    if (statSync(s).isDirectory()) copyDir(s, d); else copyFileSync(s, d);
  }
}
copyDir(join(ROOT, "public", "icons"), join(DIST, "icons"));
copyFileSync(join(ROOT, "public", "fonts", "OFL.txt"), join(DIST, "fonts", "OFL.txt"));
copyFileSync(join(ROOT, "public", "manifest.webmanifest"), join(DIST, "manifest.webmanifest"));

const CSP = [
  "default-src 'self'",
  "script-src 'self'",
  "style-src 'self'",
  "img-src 'self'", // RS-09: không nơi nào dùng data: (biểu tượng SVG dựng bằng DOMParser)
  "font-src 'self'",
  "connect-src 'self'",
  "manifest-src 'self'",
  "worker-src 'self'",
  "object-src 'none'",
  "base-uri 'self'",
  "form-action 'none'",
].join("; ");

const preloadFont = fontFiles.find((f) => /lichviet-serif-[A-Z0-9]+\.woff2$/i.test(f) && !f.includes("italic"));
if (!preloadFont) fail("không tìm thấy phông lichviet-serif để preload");
const html = readFileSync(join(ROOT, "src", "index.html"), "utf8")
  .replace("%CSP%", CSP)
  .replace("%CSS%", cssFile)
  .replace("%JS%", jsFile)
  .replace("%FONT_PRELOAD%", preloadFont ? `<link rel="preload" href="${preloadFont}" as="font" type="font/woff2" crossorigin>` : "");
writeFileSync(join(DIST, "index.html"), html);

// GitHub Pages: tắt Jekyll để giữ nguyên tên tệp.
writeFileSync(join(DIST, ".nojekyll"), "");

// ---------- 4. Service worker ----------
function listFiles(dir, base = dir) {
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    return statSync(p).isDirectory() ? listFiles(p, base) : [relative(base, p).split("\\").join("/")];
  });
}
const precache = ["./", ...listFiles(DIST).filter((f) => f !== ".nojekyll").map((f) => "./" + f)].sort();
const hash = createHash("sha256");
for (const f of precache) if (f !== "./") hash.update(f).update(readFileSync(join(DIST, f.slice(2))));
const cacheName = `lvhv-v${VERSION}-${hash.digest("hex").slice(0, 10)}`;
const sw = readFileSync(join(ROOT, "src", "sw.js"), "utf8")
  .replace("%CACHE_NAME%", cacheName)
  .replace("%PRECACHE%", JSON.stringify(precache));
writeFileSync(join(DIST, "sw.js"), sw);

// ---------- 5. Tóm tắt ----------
let total = 0;
for (const f of listFiles(DIST)) total += statSync(join(DIST, f)).size;
log(`xong → dist/ (${listFiles(DIST).length} tệp, ${(total / 1024).toFixed(1)} KB chưa nén), cache ${cacheName}`);
