// Sinh biểu tượng SVG + PNG (192, 512, maskable 512) vào public/icons/.
// Dùng Chromium của playwright-core để rasterize (không cần thư viện ảnh).
import { writeFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright-core";
import { iconSvg } from "./icon-src.mjs";

const OUT = join(dirname(fileURLToPath(import.meta.url)), "..", "public", "icons");
mkdirSync(OUT, { recursive: true });
writeFileSync(join(OUT, "icon.svg"), iconSvg());
writeFileSync(join(OUT, "icon-maskable.svg"), iconSvg({ maskable: true }));

const browser = await chromium.launch();
const page = await browser.newPage();
for (const [name, size, maskable] of [["icon-192.png", 192, false], ["icon-512.png", 512, false], ["icon-maskable-512.png", 512, true]]) {
  await page.setViewportSize({ width: size, height: size });
  const svg = iconSvg({ maskable }).replace("<svg ", `<svg width="${size}" height="${size}" `);
  await page.setContent(`<html><body style="margin:0;background:transparent">${svg}</body></html>`);
  await page.screenshot({ path: join(OUT, name), omitBackground: true, clip: { x: 0, y: 0, width: size, height: size } });
  console.log("icon", name);
}
await browser.close();
