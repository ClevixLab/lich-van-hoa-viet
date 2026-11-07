// Kiểm thử khói mô-đun "Người thân" (docs/v0.2/08 §9) — gọi từ test/smoke.mjs (mục 1d).
// Luồng thật qua giao diện: L1 thêm người đã mất theo ngày dương (kiểm ngày âm hiện ra), L2 theo ngày âm tháng nhuận
// (R-UX-1), L3 người đang sống + L4 sự kiện, L5 danh sách theo kỳ (R13: bốn loại kỳ, ‹ › Hôm nay, ba bộ lọc — đối chiếu
// với tập mong đợi tính độc lập bằng tsport/dist/memorial.mjs), L6 Sắp tới, L7 thẻ nhắc, L8 .ics, L9/L10 sao lưu
// xuất → nhập, L11 tưởng niệm, L12 sửa, L13 xóa. Ảnh chụp mọi màn hình mới: 360×780 và 1280×800, sáng và tối.
import { readFileSync } from "node:fs";
import { join } from "node:path";

const TODAY = "2026-07-25"; // Thứ Bảy; giỗ đầu của ca G-A (28/07/2026) còn 3 ngày → có thẻ nhắc (mốc mặc định [0, 3])
const FIXED = new Date(`${TODAY}T09:00:00+07:00`);

export async function runPeople({ browser, BASE, ORIGIN, ROOT, SHOTS, check, metrics }) {
  const M = await import(join(ROOT, "..", "tsport", "dist", "memorial.mjs"));
  const cal = M.Cal.get("jpl");
  const ctx = await browser.newContext({ viewport: { width: 360, height: 780 }, deviceScaleFactor: 2, hasTouch: true, isMobile: true, locale: "vi-VN", timezoneId: "Asia/Ho_Chi_Minh", reducedMotion: "reduce", colorScheme: "light" });
  const page = await ctx.newPage();
  await page.clock.setFixedTime(FIXED);
  // Đường tải về (<a download>): tắt hộp "Lưu tệp" của hệ điều hành (showSaveFilePicker); đường đó kiểm ở runSavePicker().
  await page.addInitScript(() => { Object.defineProperty(window, "showSaveFilePicker", { value: undefined, configurable: true }); });
  const errors = [], external = [], csp = [];
  page.on("console", (m) => { if (m.type() === "error" || m.type() === "warning") errors.push(`${m.type()}: ${m.text()}`); if (/Content.Security.Policy/i.test(m.text())) csp.push(m.text()); });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("request", (r) => { const u = r.url(); if (!u.startsWith(ORIGIN) && !u.startsWith("data:") && !u.startsWith("blob:")) external.push(u); });
  const shots = [];
  const DPR = 2;
  // TK-02: ảnh chụp toàn trang phải rộng đúng DPR × chiều rộng khung nhìn đã đặt (đọc chiều rộng từ phần đầu IHDR của PNG).
  const shotWidthBad = [];
  const shot = async (name, full = true) => {
    const f = `v02-${name}.png`;
    const buf = await page.screenshot({ path: join(SHOTS, f), fullPage: full });
    shots.push(f);
    const w = buf.readUInt32BE(16), want = DPR * page.viewportSize().width;
    if (w !== want) shotWidthBad.push(`${f}: ${w} px (cần ${want})`);
  };
  const go = async (hash, sel) => { await page.goto(BASE + hash); await page.waitForSelector(sel ?? ".people-body > *", { timeout: 15000 }); };
  const text = (sel) => page.locator(sel).first().innerText();
  // TK-02: ở chế độ điện thoại, khi trang tràn thì window.innerWidth "nở" theo nội dung, nên phép trừ cũ luôn bằng 0.
  // Phép kiểm mới so mọi số đo với CHIỀU RỘNG CỐ ĐỊNH của khung nhìn đã đặt, và kiểm cạnh phải của thanh mục đáy.
  const layout = () => page.evaluate((W) => {
    const de = document.documentElement;
    const tab = document.querySelector(".tabbar")?.getBoundingClientRect();
    const wide = [...document.querySelectorAll("body *")].filter((e) => { const r = e.getBoundingClientRect(); return r.width > 0 && r.right > W + 0.5; })
      .slice(0, 4).map((e) => `${e.tagName.toLowerCase()}${e.id ? "#" + e.id : ""}${e.className && typeof e.className === "string" ? "." + e.className.split(" ")[0] : ""}→${Math.round(e.getBoundingClientRect().right)}`);
    return { W, scrollW: de.scrollWidth, clientW: de.clientWidth, vvW: Math.round(window.visualViewport ? window.visualViewport.width : 0), innerW: window.innerWidth, tabRight: tab ? Math.round(tab.right) : 0, wide };
  }, page.viewportSize().width);
  const layoutOk = (m) => m.scrollW <= m.W && m.clientW <= m.W && m.vvW <= m.W && m.innerW <= m.W && m.tabRight <= m.W;
  const layoutTxt = (m) => `scrollWidth ${m.scrollW}, clientWidth ${m.clientW}, visualViewport ${m.vvW}, innerWidth ${m.innerW}, thanh mục phải ${m.tabRight} / khung ${m.W}${m.wide.length ? `; tràn: ${m.wide.join(", ")}` : ""}`;
  // TK-05: vùng chạm ≥ 44 × 44 px cho mọi phần tử bấm được trong vùng nội dung (thanh mục, chân trang nằm ngoài <main>).
  const smallTargets = (scopes = ["main", ".remind-host", "dialog[open]"]) => page.evaluate((scopes) => {
    const parts = ["a[href]", "button", "summary", "label.check", "select", "input:not([type=checkbox]):not([type=radio]):not([type=file])"];
    const sel = scopes.flatMap((sc) => parts.map((q) => `${sc} ${q}`)).join(", ");
    const vis = [...document.querySelectorAll(sel)].filter((e) => { const r = e.getBoundingClientRect(); return r.width > 0 && r.height > 0; });
    window.__targets = (window.__targets ?? 0) + vis.length;
    return vis.filter((e) => { const r = e.getBoundingClientRect(); return r.height < 43.5 || r.width < 43.5; })
      .slice(0, 6).map((e) => `${(e.textContent || e.id || e.tagName).trim().slice(0, 24)} ${Math.round(e.getBoundingClientRect().width)}×${Math.round(e.getBoundingClientRect().height)}`);
  }, scopes);
  let targetsMeasured = 0;
  const measureTargets = async (name) => { const bad = await smallTargets(scopesOf(name)); targetsMeasured += await page.evaluate(() => { const n = window.__targets ?? 0; window.__targets = 0; return n; }); return bad; };
  const scopesOf = (name) => (name === "sheet-reminder" ? [".remind-host"] : undefined); // Tờ lịch (v0.1) ngoài phạm vi mô-đun
  // Biểu thức chuỗi chạy qua CDP (không dùng eval trong trang — CSP chặn eval, đúng như mong muốn).
  const store = (fn) => page.evaluate(`(async () => { const S = await window.__lvhv.storage.loadStore(); const s = await S.openStore(); try { return await (${fn})(s); } finally { s.close(); } })()`);

  // Mọi bước trong một khối: nếu dừng giữa chừng thì chụp màn hình và in nội dung trang để chẩn đoán trên CI.
  try {
    // ---------- trống ----------
    await go("#/nguoi-than", ".welcome");
    check((await text(".welcome")).includes("Chưa có ai trong sổ"), "Người thân: trạng thái trống có lời chào + nút thêm");
    check(await page.locator('.tab[data-r="nguoi-than"][aria-current="page"]').count() === 1, "thanh mục có \"Người thân\" và được đánh dấu");
    await shot("empty-360");

    // ---------- L1: người đã mất theo ngày dương ----------
    await page.click('a[href="#/nguoi-than/them/da-mat"]');
    await page.waitForSelector("#pf-ten");
    await page.click(".form-acts .primary");
    const sum0 = await text(".errsum");
    check(sum0.includes("Họ tên không được để trống") && sum0.includes("Hãy nhập ngày mất"), `L1: lưu khi trống → "Họ tên không được để trống", "Hãy nhập ngày mất" (${sum0.replace(/\n/g, " | ")})`);
    const dxDead = await page.$$eval("#pf-dx-list option", (os) => os.map((o) => o.value));
    check(dxDead.includes("Bé đỏ") && dxDead.includes("Ông Tổ họ") && dxDead.includes("Kỵ") && dxDead.every((x) => x[0] === x[0].toLocaleUpperCase("vi")), `YC-2026-07: gợi ý cách gọi (lõi) cho người đã mất viết hoa chữ đầu, có Cụ/Kỵ/Ông Tổ họ/Bà Cô Tổ họ/Bé đỏ (${dxDead.length} mục)`);
    await page.check("#pf-gt-nam");
    await page.fill("#pf-dx", "ông");
    await page.fill("#pf-ten", "Nguyễn Văn A");
    const sum1 = await page.locator(".errsum").innerText();
    check(await page.locator("#pf-ten-err").isHidden() && !sum1.includes("Họ tên") && sum1.includes("Hãy nhập ngày mất"), `TK-03: gõ tên → lỗi dưới ô mất, khung tóm tắt bỏ dòng "Họ tên" (${sum1.replace(/\n/g, " | ")})`);
    await page.fill("#pf-ten", ""); await page.locator("#pf-ten").blur();
    check(!(await page.locator("#pf-ten-err").isHidden()), "TK-03: rời ô Họ tên khi trống → báo lỗi ngay");
    await page.fill("#pf-ten", "Nguyễn Văn A");
    await page.fill("#pf-qh", "ông nội");
    await page.fill("#pf-death-d", "31"); await page.fill("#pf-death-m", "2"); await page.fill("#pf-death-y", "2025");
    check((await text("#pf-death-err")).includes("Tháng 2 năm 2025 chỉ có 28 ngày"), "L1: 31/02/2025 → \"Tháng 2 năm 2025 chỉ có 28 ngày\"");
    await page.fill("#pf-death-d", "8"); await page.fill("#pf-death-m", "8"); await page.fill("#pf-death-y", "2027");
    check((await text("#pf-death-err")).includes("Ngày mất không được sau hôm nay"), "L1: ngày mất ở tương lai → báo lỗi");
    await page.fill("#pf-death-y", "2025");
    const res1 = await text("#pf-death-res");
    check(res1 === "Âm lịch: 15/6 nhuận năm Ất Tỵ (2025)", `L1: 08/08/2025 → hiện ngay "15/6 nhuận … Ất Tỵ (2025)" (${res1})`);
    await page.fill("#pf-birth-d", "1"); await page.fill("#pf-birth-m", "5"); await page.fill("#pf-birth-y", "1940");
    check((await text("#pf-tuoi")) === "Hưởng thọ 86 tuổi", `L1: sinh 01/05/1940 → "Hưởng thọ 86 tuổi" (${await text("#pf-tuoi")})`);
    await page.fill("#pf-hieu", "Phúc Thiện");
    await page.fill("#pf-antang", "Nghĩa trang Văn Điển, Hà Nội");
    // Ảnh có đoạn EXIF (APP1) chèn thêm → byte lưu phải không còn APP1 (S9).
    const jpg = readFileSync(join(ROOT, "..", "docs/v0.2/schema/test-vectors/photo-0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e09.jpg"));
    const exif = Buffer.concat([Buffer.from("Exif\0\0", "binary"), Buffer.from("GPS 21.0285N 105.8542E — thông tin vị trí giả", "utf8")]);
    const app1 = Buffer.concat([Buffer.from([0xff, 0xe1, (exif.length + 2) >> 8, (exif.length + 2) & 255]), exif]);
    const withExif = Buffer.concat([jpg.subarray(0, 2), app1, jpg.subarray(2)]);
    await page.setInputFiles("#pf-file", { name: "khong-phai-anh.jpg", mimeType: "image/jpeg", buffer: Buffer.from("đây không phải ảnh", "utf8") });
    await page.waitForSelector(".photo-acts .ferr:not([hidden])");
    check((await text(".photo-acts .ferr")) === "Định dạng ảnh chưa hỗ trợ — hãy chọn ảnh JPEG, PNG, WebP hoặc GIF", "ảnh hỏng / định dạng không nhận ra → không giải mã, báo \"Định dạng ảnh chưa hỗ trợ…\" (BM2-13)");
    const bigPng = Buffer.alloc(64);
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]).copy(bigPng); bigPng.writeUInt32BE(13, 8); bigPng.write("IHDR", 12); bigPng.writeUInt32BE(20000, 16); bigPng.writeUInt32BE(20000, 20);
    await page.setInputFiles("#pf-file", { name: "rat-lon.png", mimeType: "image/png", buffer: bigPng });
    await page.waitForFunction(() => /Ảnh quá lớn/.test(document.querySelector(".photo-acts .ferr")?.textContent ?? ""));
    check((await text(".photo-acts .ferr")) === "Ảnh quá lớn (20000 × 20000 điểm ảnh) — hãy chọn ảnh nhỏ hơn", "MA3-15-07: ảnh 20 000 × 20 000 điểm ảnh bị từ chối trước khi giải mã");
    await page.setInputFiles("#pf-file", { name: "chan-dung.jpg", mimeType: "image/jpeg", buffer: withExif });
    await page.waitForSelector(".pf-photo[data-drawn]");
    await page.click("#pf-mem > summary");
    check(await page.locator("#pf-bia, #pf-bia-hint").count() === 0, "Yêu cầu 07/11: không còn ô \"dòng đầu bia\" riêng trên biểu mẫu");
    const qhOpts = await page.$$eval("#pf-qh-list option", (os) => os.map((o) => o.value));
    check(["Bố", "Mẹ", "Ông nội", "Bà ngoại", "Cụ ông", "Kỵ bà", "Chú", "Thím"].every((x) => qhOpts.includes(x)) && !qhOpts.some((x) => /Hiển|khảo|tỷ|Tằng/.test(x)),
      `Ô Quan hệ gợi ý từ thường ngày (${qhOpts.length} mục), không có từ Hán-Việt cổ`);
    await page.fill("#pf-loi", "Con cháu muôn đời ghi nhớ công ơn ông.");
    await shot("form-deceased-360");
    await page.click(".form-acts .primary");
    await page.waitForSelector("#person-name");
    const idA = /nguoi\/([0-9a-f-]+)/.exec(page.url())[1];
    const pA = await text(".people-body");
    check(pA.includes("15/6 nhuận năm Ất Tỵ (2025) — đổi từ ngày dương") && pA.includes("Giỗ đầu"), "L1: hồ sơ ghi ngày âm đã đổi và lần tới là Giỗ đầu");
    const recA = await store(`async (s) => { const p = await s.person("${idA}"); const ph = new Uint8Array(await s.photoBytes(p.photo_id)); let i = 2, app1 = false; while (i + 4 < ph.length && ph[i] === 0xff && ph[i+1] !== 0xda) { if (ph[i+1] === 0xe1 || ph[i+1] === 0xed) app1 = true; i += 2 + ((ph[i+2] << 8) | ph[i+3]); } return { lunar: p.death.lunar, src: p.death.lunar_source, rev: p.rev, persist: await s.persistResult(), app1, mime: (await s.photos())[0].mime }; }`);
    check(JSON.stringify(recA.lunar) === JSON.stringify({ day: 15, leap: true, month: 6, year: 2025 }) || (recA.lunar.year === 2025 && recA.lunar.month === 6 && recA.lunar.day === 15 && recA.lunar.leap === true), `L1: lưu death.lunar = 15/6 nhuận 2025 (${JSON.stringify(recA.lunar)})`);
    check(recA.src === "converted" && recA.rev === 1, "L1: lunar_source = converted, rev = 1");
    const gA = await store(`async (s) => { const p = await s.person("${idA}"); return [p.gioi_tinh, p.danh_xung, p.quan_he]; }`);
    const pfacts = await text(".people-body");
    check(JSON.stringify(gA) === JSON.stringify(["nam", "Ông", "Ông nội"]) && /Giới tính\s*Nam/.test(pfacts),
      `YC-2026-07: lưu gioi_tinh = "nam"; gõ "ông"/"ông nội" → lưu "Ông"/"Ông nội"; hồ sơ hiện "Giới tính: Nam" (${JSON.stringify(gA)})`);
    check(recA.app1 === false && recA.mime === "image/jpeg", "S9: ảnh lưu đã mã hóa lại, không còn đoạn APP1/APP13 (EXIF/GPS)");
    check(typeof recA.persist === "boolean", `R-PWA-1: navigator.storage.persist() đã gọi khi lưu hồ sơ đầu tiên (${recA.persist})`);
    await page.waitForSelector(".pp-photo[data-drawn]");
    check(!(await page.content()).includes("blob:"), "ảnh hiển thị bằng canvas, không có blob: trong trang (S2b)");
    await shot("person-deceased-360");

    // ---------- L2: người đã mất theo ngày âm, tháng nhuận ----------
    await go("#/nguoi-than/them/da-mat", "#pf-ten");
    await page.fill("#pf-dx", "Bà");
    await page.fill("#pf-ten", "Trần Thị B");
    await page.fill("#pf-qh", "bà nội");
    await page.click('#pf-death-cal button[data-v="lunar"]');
    await page.fill("#pf-death-d", "15"); await page.fill("#pf-death-m", "5"); await page.fill("#pf-death-y", "2026");
    check(await page.locator('label[for="pf-death-leap"]').isHidden() && (await text("#pf-death .fine")).includes("Năm 2026 không có tháng nhuận"), "L2 (R-UX-1): năm 2026 không nhuận → ẩn hộp \"Tháng nhuận\", có chú thích");
    await page.fill("#pf-death-y", "2025");
    const leapLbl = await text('label[for="pf-death-leap"]');
    check(leapLbl.includes("năm 2025 nhuận tháng 6"), `L2: năm 2025 → hộp nhuận ghi "năm 2025 nhuận tháng 6" (${leapLbl})`);
    await page.check("#pf-death-leap");
    check((await text("#pf-death-err")).includes("Năm 2025 không có tháng 5 nhuận"), "L2: 15/5 nhuận 2025 → \"Năm 2025 không có tháng 5 nhuận\"");
    await page.fill("#pf-death-m", "6");
    const res2 = await text("#pf-death-res");
    check(res2 === "Dương lịch: Thứ Sáu, 08/08/2025", `L2: 15/6 nhuận 2025 → "Dương lịch: Thứ Sáu, 08/08/2025" (${res2})`);
    await page.click(".form-acts .primary");
    await page.waitForSelector("#person-name");
    const idB = /nguoi\/([0-9a-f-]+)/.exec(page.url())[1];
    check((await text(".people-body")).includes("nhập theo âm lịch"), "L2: hồ sơ ghi \"nhập theo âm lịch\"");
    const recB = await store(`async (s) => (await s.person("${idB}")).death`);
    check(recB.lunar_source === "entered" && recB.solar.y === 2025 && recB.solar.m === 8 && recB.solar.d === 8, "L2: lưu lunar_source = entered, solar = 2025-08-08");

    // Một người đã mất không rõ năm (giỗ 10/3 âm) để danh sách năm có thêm mục.
    await go("#/nguoi-than/them/da-mat", "#pf-ten");
    await page.fill("#pf-dx", "Cụ"); await page.fill("#pf-ten", "Lê Văn D");
    await page.click('#pf-death-cal button[data-v="lunar"]');
    await page.check("#pf-death-unk");
    await page.fill("#pf-death-d", "10"); await page.fill("#pf-death-m", "3");
    check((await text("#pf-death-res")).includes("Không rõ năm: vẫn nhắc giỗ hằng năm vào 10/3 âm lịch"), "ngày mất âm không rõ năm → giải thích");
    await page.click(".form-acts .primary");
    await page.waitForSelector("#person-name");

    // ---------- L3 + L4: người đang sống, sự kiện ----------
    await go("#/nguoi-than/them/dang-song", "#pf-ten");
    const dxLive = await page.$$eval("#pf-dx-list option", (os) => os.map((o) => o.value));
    check(!["Cụ", "Kỵ", "Ông Tổ họ", "Bà Cô Tổ họ", "Bé đỏ"].some((x) => dxLive.includes(x)) && dxLive.includes("Anh") && await page.isChecked("#pf-gt-none"),
      `YC-2026-07: gợi ý cách gọi người đang sống không có Cụ, Kỵ, Ông Tổ họ, Bà Cô Tổ họ, Bé đỏ; giới tính mặc định "Không ghi" (${dxLive.join(", ")})`);
    // #38 (core3): mọi trường chữ tự do cắt khoảng trắng hai đầu khi lưu; quan hệ viết hoa chữ đầu (sau khi cắt).
    // page.fill không rời ô → không qua bước viết hoa khi rời ô (forms.ts): kiểm đúng việc chuẩn hóa lúc lưu của lõi.
    await page.fill("#pf-ten", "  Lê Văn C \u00a0"); await page.fill("#pf-qh", "   con trai  ");
    await page.fill("#pf-noisinh", "  Huế  "); await page.fill("#pf-note", "\n  học ở Huế  \n");
    await page.fill("#pf-birth-d", "20"); await page.fill("#pf-birth-m", "1"); await page.fill("#pf-birth-y", "1990");
    const res3 = await text("#pf-birth-res");
    check(res3.includes("Âm lịch: 24/12 năm Kỷ Tỵ (1989)") && res3.includes("Hôm nay 36 tuổi (tuổi âm 38)"), `L3: sinh 20/01/1990 → Kỷ Tỵ (1989), 36 tuổi (tuổi âm 38) (${res3})`);
    await page.check("#pf-snam");
    // TK-29: mặc định "giống sinh nhật dương lịch" → khối riêng ẩn.
    check(await page.isChecked("#pf-snam-same") && await page.locator("#pf-rsna").isHidden(), "TK-29: bật sinh nhật âm → mặc định nhắc giống sinh nhật dương, khối riêng ẩn");
    // TK-28: giờ nhắc là hai ô số 24 giờ; 25 giờ → lỗi; 18:5 → lưu "18:05".
    check(await page.locator('#pf-rsn input[type="time"]').count() === 0, "TK-28: không dùng ô type=time (SA/CH theo trình duyệt)");
    await page.fill("#pf-rsn-time", "25");
    await page.click(".form-acts .primary");
    await page.waitForSelector("#pf-rsn .ferr:not([hidden])");
    check((await text("#pf-rsn .ferr")) === "Giờ nhắc phải từ 0 đến 23 giờ, phút từ 0 đến 59" && (await page.getAttribute("#pf-rsn-time", "aria-invalid")) === "true", "TK-28: giờ 25 → lỗi dưới khối nhắc, ô giờ được đánh dấu");
    await page.fill("#pf-rsn-time", "18"); await page.fill("#pf-rsn-min", "5");
    await shot("form-living-360");
    await page.click(".form-acts .primary");
    await page.waitForSelector("#person-name");
    const idC = /nguoi\/([0-9a-f-]+)/.exec(page.url())[1];
    const trimC = await store(`async (s) => { const p = await s.person("${idC}"); return { ho_ten: p.ho_ten, quan_he: p.quan_he, ghi_chu: p.ghi_chu, noi_sinh: p.birth.noi_sinh }; }`);
    const viewC = { name: await text("#person-name"), rel: await page.locator(".phead-rel").first().innerText().catch(() => ""), body: await text(".people-body") };
    check(trimC.ho_ten === "Lê Văn C" && trimC.quan_he === "Con trai" && trimC.ghi_chu === "học ở Huế" && trimC.noi_sinh === "Huế"
      && viewC.name === "Lê Văn C" && viewC.rel === "Con trai" && viewC.body.includes("Nơi sinh\nHuế") && viewC.body.includes("Ghi chú\nhọc ở Huế"),
      `#38: lưu cắt khoảng trắng mọi trường chữ, quan hệ viết hoa; hồ sơ hiện đúng giá trị đã cắt (${JSON.stringify(trimC)})`);
    await shot("person-living-trimmed-360");
    const remC = await store(`async (s) => (await s.person("${idC}")).reminders`);
    check(remC.sinh_nhat.time_local === "18:05" && remC.sinh_nhat_am.time_local === "18:05" && JSON.stringify(remC.sinh_nhat_am.offsets_days) === JSON.stringify(remC.sinh_nhat.offsets_days) && remC.sinh_nhat_am.id !== remC.sinh_nhat.id,
      `TK-28/29: lưu giờ "18:05"; sinh nhật âm chép cài đặt sinh nhật dương, id riêng (${JSON.stringify(remC.sinh_nhat_am)})`);
    check((await text("#next-box")).includes("Sinh nhật âm lịch") || (await text("#next-box")).includes("Sinh nhật thứ 37"), "L3: hồ sơ có \"Lần tới\" sinh nhật");
    await page.click("#add-event");
    await page.waitForSelector("#ef-type");
    check(!(await page.isChecked("#ef-rec")), "L4: loại Tốt nghiệp mặc định không lặp");
    await page.fill("#ef-date-d", "15"); await page.fill("#ef-date-m", "8"); await page.fill("#ef-date-y", "2026");
    await shot("form-event-360");
    await page.click(".form-acts .primary");
    await page.waitForSelector("#event-box .evrow");
    const evTxt = await text("#event-box");
    check(evTxt.includes("Tốt nghiệp · 15/08/2026 · một lần"), `L4: sự kiện Tốt nghiệp 15/08/2026 một lần (${evTxt.replace(/\n/g, " | ")})`);
    await page.click("#add-event");
    await page.waitForSelector("#ef-type");
    await page.selectOption("#ef-type", "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0002");
    check(await page.isChecked("#ef-rec"), "L4: Ngày cưới mặc định lặp hằng năm");
    await page.click('#ef-date-cal button[data-v="lunar"]');
    await page.fill("#ef-date-d", "10"); await page.fill("#ef-date-m", "10"); await page.fill("#ef-date-y", "2020");
    await page.click(".form-acts .primary");
    await page.waitForSelector("#event-box .evrow >> nth=1");
    await shot("person-living-360");

    // ---------- Bổ sung B4: với người đã mất chỉ nhắc ngày giỗ ----------
    // (1) Sự kiện của người đã mất: biểu mẫu không có khối nhắc, có câu giải thích; vẫn lưu và hiện trên hồ sơ.
    await go(`#/nguoi-than/su-kien/${idA}/moi`, "#ef-type");
    check(await page.locator("#ef-rule").count() === 0 && (await text("#ef-b4")).startsWith("Không nhắc sự kiện của người đã mất"), "B4: sự kiện của người đã mất — không có khối nhắc, có câu giải thích");
    await page.selectOption("#ef-type", "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0002");
    await page.fill("#ef-date-d", "20"); await page.fill("#ef-date-m", "7"); await page.fill("#ef-date-y", "1965");
    await shot("form-event-deceased-360");
    await page.click(".form-acts .primary");
    await page.waitForSelector("#event-box .evrow");
    check((await text("#event-box")).includes("Ngày cưới · 20/07/1965 · hằng năm") && (await text("#b4-note")).startsWith("Không nhắc sinh nhật và sự kiện của người đã mất"),
      "B4: hồ sơ người đã mất vẫn hiện sự kiện, có câu \"Không nhắc sinh nhật và sự kiện của người đã mất\"");
    await shot("person-deceased-b4-360");
    // (2) Sự kiện gắn [người đã mất, người đang sống] (dữ liệu từ sao lưu nền tảng khác): không sinh lần xuất hiện.
    await store(`async (s) => { const e = (await s.events()).find((x) => x.person_ids[0] === "${idC}" && !x.recurring); await s.putEvent({ ...e, id: crypto.randomUUID(), tieu_de: "Họp mặt chung", person_ids: ["${idA}", "${idC}"], rev: 1 }); }`);

    // ---------- L5: Theo kỳ (R13) ----------
    const ds = M.parseDataset(await store(`async (s) => JSON.stringify({ settings: await s.settings(), persons: await s.persons(), events: await s.events(), event_types: await s.eventTypes() })`));
    const ymd = (s) => ({ y: +s.slice(0, 4), m: +s.slice(5, 7), d: +s.slice(8, 10) });
    const isoOf = (d) => `${d.y}-${String(d.m).padStart(2, "0")}-${String(d.d).padStart(2, "0")}`;
    // MA3-15-01: mong đợi = `eventsInPeriod` của lõi (tsport, cổng của memorial/src/period.rs) — kiểm thử không tự
    // viết lại quy tắc kỳ hay B4. Giao diện phải hiện đúng từng mục, đúng thứ tự.
    const mixed = ds.events.find((e) => e.tieu_de === "Họp mặt chung");
    const expected = (kind, anchor, filter) => M.eventsInPeriod(cal, ds, kind, anchor, filter).items.map((it) => `${it.class}@${isoOf(it.date)}`);
    // B4 mục 2: sự kiện gắn [đã mất, đang sống] không có ở BẤT KỲ bộ lọc nào (kỳ nào cũng vậy).
    const mixedHits = ["tuan", "thang", "quy", "nam"].flatMap((k) => ["tat_ca", "da_mat", "dang_song"].flatMap((f) => M.eventsInPeriod(cal, ds, k, ymd("2026-08-15"), f).items.filter((it) => it.entity_id === mixed.id)));
    const mixedRaw = M.occurrences(cal, { ...ds, events: [mixed], persons: ds.persons.map((p) => ({ ...p, death: null, kind: "living" })) }, ymd("2026-08-01"), ymd("2026-08-31")).length;
    check(mixed && mixedRaw === 1 && mixedHits.length === 0, `B4 (L5): sự kiện "Họp mặt chung" gắn [Nguyễn Văn A (đã mất), Lê Văn C] không có ở bộ lọc nào (0 mục / 12 kỳ × bộ lọc; nếu không có B4 sẽ có ${mixedRaw})`);
    const shown = () => page.$$eval("#period-list .pitem", (els) => els.map((e) => `${e.dataset.class}@${e.dataset.date}`));
    await go("#/nguoi-than/theo-ky", "#period-title");
    const t0 = ymd(TODAY);
    const titles = {};
    let periodOk = true;
    for (const [kind, label] of [["tuan", "Tuần"], ["thang", "Tháng"], ["quy", "Quý"], ["nam", "Năm"]]) {
      await page.click(`.pseg.four button[data-v="${kind}"]`);
      titles[kind] = await text("#period-title");
      for (const filter of ["tat_ca", "da_mat", "dang_song"]) {
        await page.click(`.pseg.three button[data-v="${filter}"]`);
        const got = await shown(), exp = expected(kind, t0, filter);
        const ok = JSON.stringify(got) === JSON.stringify(exp);
        if (!ok) periodOk = false;
        check(ok, `L5 ${label} · ${filter}: ${got.length} mục khớp tập mong đợi (${exp.length})${ok ? "" : ` — có ${got.join(",")} / cần ${exp.join(",")}`}`);
      }
      await page.click('.pseg.three button[data-v="tat_ca"]');
      if (kind === "thang") await shot("period-month-360");
    }
    check(titles.tuan === "Tuần 20/07 – 26/07/2026", `L5: tiêu đề tuần (Thứ Hai → Chủ nhật) "${titles.tuan}"`);
    check(titles.thang === "Tháng 7/2026 · 01/07 – 31/07", `L5: tiêu đề tháng "${titles.thang}"`);
    check(titles.quy === "Quý III/2026 · 01/07 – 30/09", `L5: tiêu đề quý "${titles.quy}"`);
    check(titles.nam === "Năm 2026 · 01/01 – 31/12", `L5: tiêu đề năm "${titles.nam}"`);
    // ‹ › Hôm nay (đang ở Năm)
    await page.click('.pseg.four button[data-v="quy"]');
    await page.click("#period-next");
    check((await text("#period-title")) === "Quý IV/2026 · 01/10 – 31/12", "L5: › → Quý IV/2026");
    check(JSON.stringify(await shown()) === JSON.stringify(expected("quy", ymd("2026-10-01"), "tat_ca")), "L5: Quý IV/2026 đúng tập mục");
    await page.click("#period-next");
    check((await text("#period-title")) === "Quý I/2027 · 01/01 – 31/03", "L5: › → Quý I/2027 (qua năm)");
    check(JSON.stringify(await shown()) === JSON.stringify(expected("quy", ymd("2027-01-01"), "tat_ca")), "L5: Quý I/2027 đúng tập mục (có sinh nhật dương và âm)");
    await page.click("#period-prev"); await page.click("#period-prev"); await page.click("#period-prev");
    check((await text("#period-title")) === "Quý II/2026 · 01/04 – 30/06", "L5: ‹ ×3 → Quý II/2026");
    await page.click('.pseg.four button[data-v="tuan"]');
    check((await text("#period-title")) === "Tuần 30/03 – 05/04/2026", `L5: đổi sang Tuần giữ ngày neo (${await text("#period-title")})`);
    await page.click("#period-today");
    check((await text("#period-title")) === "Tuần 20/07 – 26/07/2026" && await page.locator("#period-today").isHidden() && await page.locator("#period-current").isVisible(), "L5: Hôm nay → tuần chứa hôm nay; thay nút bằng \"Đang xem kỳ có hôm nay\" (TK-23)");
    await page.click("#period-next");
    const wk = await shown();
    check(wk.filter((x) => x.startsWith("gio@2026-07-28")).length === 2, `L5: tuần 27/07 – 02/08 có 2 giỗ đầu ngày 28/07 (${wk.join(",")})`);
    await page.click('.pseg.three button[data-v="dang_song"]');
    check((await text("#period-list")).includes("Không có ngày giỗ, sinh nhật hay sự kiện nào trong kỳ này. (đang lọc: Người đang sống)"), "L5: trạng thái trống có ghi bộ lọc");
    await page.click('.pseg.three button[data-v="tat_ca"]');
    await page.click('.pseg.four button[data-v="nam"]');
    // R-UX-2: đúng một tiêu đề cho mỗi tháng có mục, theo thứ tự, không lặp, không thiếu.
    const wantMonths = [...new Set(expected("nam", t0, "tat_ca").map((x) => x.split("@")[1].slice(0, 7)))].map((ym) => `Tháng ${+ym.slice(5, 7)}/${ym.slice(0, 4)}`);
    const gotMonths = await page.$$eval("#period-list .pmonth", (els) => els.map((e) => e.textContent));
    check(JSON.stringify(gotMonths) === JSON.stringify(wantMonths), `R-UX-2: danh sách năm có đúng các nhóm tháng (${gotMonths.join(", ")})`);
    const yearTitles = await page.$$eval("#period-list .pitem .pi-l1", (els) => els.map((e) => e.textContent));
    check(!yearTitles.some((x) => /Họp mặt chung|Ngày cưới · .*Nguyễn Văn A/.test(x)), `B4: năm 2026 không có sự kiện gắn người đã mất (${yearTitles.length} mục)`);
    await page.click('.pseg.three button[data-v="da_mat"]');
    check((await text("#period-filter-note")).startsWith("Chỉ có ngày giỗ") && (await page.$$eval("#period-list .pitem", (els) => els.every((e) => e.dataset.class === "gio" || e.dataset.class === "tien_thuong"))),
      "B4: bộ lọc \"Người đã mất\" chỉ có giỗ/tiên thường, có câu giải thích");
    await shot("period-deceased-filter-360");
    await page.click('.pseg.three button[data-v="tat_ca"]');
    // R13 (c) trên GIAO DIỆN: "Người đã mất" ∪ "Người đang sống" = "Tất cả", không trùng mục (đo trên danh sách hiển thị).
    const uiAll = await shown();
    await page.click('.pseg.three button[data-v="da_mat"]'); const uiDead = await shown();
    await page.click('.pseg.three button[data-v="dang_song"]'); const uiLive = await shown();
    await page.click('.pseg.three button[data-v="tat_ca"]');
    const dup = uiDead.filter((x) => uiLive.includes(x));
    check(uiAll.length > 0 && JSON.stringify([...uiDead, ...uiLive].sort()) === JSON.stringify(uiAll.slice().sort()) && dup.length === 0,
      `R13 (c) trên giao diện, năm 2026: ${uiDead.length} đã mất + ${uiLive.length} đang sống = ${uiAll.length} tất cả, ${dup.length} trùng`);
    // Chạm mục đầu tiên → mở đúng hồ sơ của người ghi trên mục.
    const firstTitle = await page.locator("#period-list .pitem .pi-l1").first().innerText();
    const firstName = firstTitle.split(" · ").pop();
    await page.locator("#period-list .pitem").first().click();
    await page.waitForSelector("#person-name");
    const opened = await text("#person-name");
    check(opened === firstName, `L5: chạm mục "${firstTitle}" → mở đúng hồ sơ "${opened}"`);
    metrics.people_period = { titles, all_match: periodOk };

    // ---------- L6: Sắp tới, L7: thẻ nhắc ----------
    await go("#/nguoi-than", "#upcoming-list");
    const up = await page.$$eval("#upcoming-list .pitem", (els) => els.map((e) => `${e.dataset.class}@${e.dataset.date}`));
    const upExp = M.upcoming(cal, ds, t0, 30).map((u) => `${u.occ.class}@${isoOf(u.occ.date)}`);
    check(JSON.stringify(up) === JSON.stringify(upExp), `L6: "Sắp tới" 30 ngày khớp upcoming() của lõi (${up.length} mục)`);
    check((await text("#upcoming-list")).includes("Còn 3 ngày"), "L6: mục ghi \"Còn 3 ngày\"");
    check((await text(".remind")).includes("Còn 3 ngày: Giỗ đầu Ông Nguyễn Văn A"), "L7: thẻ nhắc trong trang Sắp tới");
    check((await text(".notice-web")).includes("Ứng dụng web không gửi được thông báo khi đang đóng"), "Sắp tới nói rõ giới hạn của ứng dụng web");
    await shot("upcoming-360");
    await page.goto(BASE + "#/ngay/" + TODAY);
    await page.reload(); // mở lại ứng dụng (cùng tài liệu → tải lại để chạy bước khởi động)
    await page.waitForSelector(".remind-host .rcard", { timeout: 15000 });
    const card = await text(".remind-host .rcard");
    check(card.includes("Còn 3 ngày: Giỗ đầu Ông Nguyễn Văn A") && card.includes("Thứ Ba, 28/07/2026 — 15/6 âm lịch"), `L7: mở ứng dụng ngày D−3 → thẻ nhắc trên Tờ lịch (02 §7.1) (${card.replace(/\n/g, " | ")})`);
    await shot("reminder-card-360", false);
    const more = await text(".remind-host .rc-more");
    check(await page.locator(".remind-host .rcard").count() === 1 && more === "Còn 1 lời nhắc khác hôm nay — Xem", `TK-15: Tờ lịch chỉ một thẻ, lời nhắc còn lại gộp một dòng (${more})`);
    await page.locator('.remind-host .rcard button:has-text("Đã biết")').first().click();
    check(await page.locator(".remind-host .rcard").count() === 0, "L7: \"Đã biết\" ẩn thẻ");

    // ---------- L12: sửa ----------
    await go(`#/nguoi-than/sua/${idA}`, "#pf-ten");
    check((await page.inputValue("#pf-ten")) === "Nguyễn Văn A" && (await text("#pf-death-res")).includes("15/6 nhuận"), "L12: biểu mẫu sửa điền sẵn, có kết quả đổi lịch");
    await page.fill("#pf-qh", "ông nội (bên nội)");
    await page.click(".form-acts .primary");
    await page.waitForSelector("#person-name");
    const rev2 = await store(`async (s) => { const p = await s.person("${idA}"); return [p.rev, p.quan_he, p.photo_id !== null]; }`);
    check(rev2[0] === 2 && rev2[1] === "Ông nội (bên nội)" && rev2[2], `L12: sửa → rev = 2, giữ ảnh; gõ "ông nội (bên nội)" → lưu "Ông nội (bên nội)" (YC-2026-07 viết hoa chữ đầu) (${JSON.stringify(rev2)})`);

    // ---------- L11: tưởng niệm ----------
    await page.click("#open-memorial");
    await page.waitForSelector(".bia-name");
    const bia = (await page.locator(".bia").textContent()).replace(/\s+/g, " ");
    const biaTop = await text(".bia-top");
    check(biaTop === "Ông nội" && bia.includes("Nguyễn Văn A") && !bia.includes("Ông Nguyễn") && !/Hiển|khảo|Tổ tỷ/.test(bia) && bia.includes("Canh Thìn 1940 — Ất Tỵ 2025") && bia.includes("Hưởng thọ 86 tuổi"), `L11: bia ghi quan hệ "${biaTop}" (từ ô Quan hệ "ông nội (bên nội)"), tên không lặp cách gọi, năm Can Chi, hưởng thọ (${bia})`);
    check(await page.locator(".incense .stick").count() === 3, "L11: ba nén hương");
    check(await page.locator("audio, video").count() === 0, "R8 (e): không có audio/video");
    check(!(await page.title()).includes("Nguyễn"), "O-9: tiêu đề tab tưởng niệm không có tên người");
    check(await page.locator(".mem-scene.lit").count() === 0 && await page.locator("#mem-toggle").isHidden(), "R8 (d): giảm chuyển động → khói tĩnh, không có nút");
    await page.waitForSelector(".mem-photo canvas[data-drawn]");
    await shot("memorial-360");

    // ---------- L8: .ics ----------
    await go("#/nguoi-than/sao-luu", "#ics-open");
    await shot("backup-360");
    await page.click("#ics-open");
    await page.waitForSelector(".ics-dialog[open]");
    const icsTxt = await text(".ics-dialog");
    check(icsTxt.includes("Không gồm thông tin báo tử."), "TKN-X-04: hộp .ics nói rõ \"Không gồm thông tin báo tử.\"");
    check(icsTxt.includes("Tệp này chứa tên, ngày giỗ, sinh nhật và sự kiện của 4 người. Bất kỳ ai có tệp đều đọc được.") && icsTxt.includes("Xem trước") && icsTxt.includes("Tốt nghiệp · Lê Văn C"), `L8 (S10, TK-11): câu riêng tư đúng nội dung tệp, xem trước có tên người (${icsTxt.split("\n")[1]})`);
    await shot("ics-dialog-360", false);
    const [dl] = await Promise.all([page.waitForEvent("download"), page.click('.ics-dialog .dbtn.main')]);
    const ics = readFileSync(await dl.path(), "utf8");
    check(dl.suggestedFilename() === `LichVanHoaViet-ngay-gio-${TODAY}.ics` && ics.startsWith("BEGIN:VCALENDAR\r\n") && ics.includes("SUMMARY:Giỗ đầu Ông Nguyễn Văn A") && !ics.includes("RRULE"), `L8: tải ${dl.suggestedFilename()} (RFC 5545, không RRULE)`);
    check(!ics.includes("Họp mặt chung") && !/SUMMARY:Ngày cưới[^\r]*Nguyễn Văn A/.test(ics), "B4: tệp .ics không có sự kiện gắn người đã mất");

    // ---------- L13: xóa (Hủy rồi đồng ý) ----------
    await go(`#/nguoi-than/nguoi/${idB}`, "#del-person");
    await page.click("#del-person");
    await page.waitForSelector(".pdialog[open]");
    check((await text(".pdialog")).includes("Xóa hồ sơ của Bà Trần Thị B?") && await page.evaluate(() => document.activeElement?.textContent === "Giữ lại"), "L13: hộp xác nhận xóa, tiêu điểm ở \"Giữ lại\"");
    await page.click('.pdialog .dbtn:has-text("Giữ lại")');
    check((await store(`async (s) => (await s.persons()).length`)) === 4, "L13: Giữ lại → hồ sơ còn");
    await page.click("#del-person");
    await page.click(".pdialog .dbtn.danger");
    await page.waitForSelector("#list-deceased");
    check((await store(`async (s) => (await s.persons()).some((p) => p.id === "${idB}")`)) === false && (await text(".toast")).includes("Đã xóa hồ sơ."), "L13: Xóa hồ sơ → không còn, có thông báo");
    await shot("list-360");

    // ---------- L9/L10: sao lưu xuất → nhập ----------
    await go("#/nguoi-than/sao-luu", "#bk-export");
    check((await text("#bk-in")).includes("Chọn tệp sao lưu…") && !(await text("#bk-in")).includes("Choose File"), "TK-06: khôi phục có nút \"Chọn tệp sao lưu…\" tiếng Việt");
    const h1 = await store(`async (s) => s.stateHash()`);
    await page.click("#bk-export");
    await page.fill("#bk-pw1", "ngắn");
    await page.fill("#bk-pw2", "ngắn");
    await page.click(".bk-dialog .dbtn.main");
    check((await text("#bk-pw1-err")) === "Mật khẩu cần ít nhất 8 ký tự" && (await page.getAttribute("#bk-pw1", "aria-invalid")) === "true", "L9 (TK-14): mật khẩu ngắn → lỗi ngay dưới ô, ô được đánh dấu");
    await page.fill("#bk-pw1", "nhà có cây bàng");
    await page.fill("#bk-pw2", "nhà có cây bàng khác");
    await page.click(".bk-dialog .dbtn.main");
    check((await text("#bk-pw2-err")) === "Hai lần nhập mật khẩu chưa giống nhau" && await page.locator("#bk-pw1-err").isHidden(), "L9 (TK-14): hai lần nhập khác nhau → lỗi dưới ô nhập lại");
    await page.fill("#bk-pw2", "nhà có cây bàng");
    await shot("backup-dialog-360", false);
    const dlP = page.waitForEvent("download", { timeout: 120000 });
    // MA3-15-03 (Esc trong lúc mã hóa) kiểm tất định ở runEscGate() bên dưới, trên bản dựng kiểm thử có cổng giữ bước mã hóa.
    await page.click(".bk-dialog .dbtn.main");
    const bk = await dlP;
    const bkBytes = readFileSync(await bk.path());
    check(bk.suggestedFilename() === `LichVanHoaViet-saoluu-${TODAY}.lvhv` && bkBytes.subarray(0, 7).toString() === "LVHVBAK", `L9: tải ${bk.suggestedFilename()} (${bkBytes.length} byte)`);
    // Trang Sao lưu dựng lại sau khi xuất xong — chờ đúng dòng mới, không đọc bản cũ.
    const lastOk = await page.waitForSelector('#bk-last:has-text("Lần sao lưu gần nhất: 25/07/2026")', { timeout: 30000 }).then(() => true, () => false);
    check(lastOk, `L9: ghi lần sao lưu gần nhất (${await text("#bk-last")})`);
    const srcBackup = readFileSync(join(ROOT, "src/people/backup.ts"), "utf8"), srcCrypto = readFileSync(join(ROOT, "src/store/backup-crypto.ts"), "utf8");
    const num = (re, src) => Number(eval(re.exec(src)[1].replace(/_/g, "")));
    const limitOk = num(/MAX_BACKUP_FILE = ([0-9 +*]+);/, srcBackup) === num(/HEADER_LEN = (\d+);/, srcCrypto) + num(/MAX_PLAINTEXT = ([0-9 *]+);/, srcCrypto) + num(/TAG_LEN = (\d+);/, srcCrypto);
    const huge = await page.evaluate(() => {
      // Tệp "thưa": ghép 130 lần cùng một Blob 1 MiB (trình duyệt không chép byte) → 130 MiB > giới hạn.
      const mb = new Blob([new Uint8Array(1024 * 1024)]);
      const f = new File(Array(130).fill(mb), "qua-lon.lvhv", { type: "application/octet-stream" });
      window.__ab = 0;
      for (const P of [Blob.prototype, File.prototype]) { const o = P.arrayBuffer; P.arrayBuffer = function (...a) { window.__ab++; return o.apply(this, a); }; }
      const dt = new DataTransfer(); dt.items.add(f);
      const inp = document.querySelector("#bk-file"); inp.files = dt.files; inp.dispatchEvent(new Event("change"));
      return f.size;
    });
    await page.fill("#bk-ipw", "nhà có cây bàng");
    await page.click("#bk-open");
    await page.waitForSelector("#bk-imp-err:not([hidden])", { timeout: 5000 });
    const hugeReads = await page.evaluate(() => window.__ab);
    check(limitOk && (await text("#bk-imp-err")) === "Sai mật khẩu hoặc tệp bị hỏng" && hugeReads === 0,
      `MA3-15-02 (S12): tệp ${(huge / 1048576).toFixed(0)} MiB > giới hạn → thông báo chung, 0 lần đọc tệp (${hugeReads}); giới hạn = 72 + MAX_PLAINTEXT + 16 (${limitOk})`);
    await page.fill("#bk-ipw", "");
    await store(`async (s) => { await s.deletePerson("${idC}"); }`);
    await page.setInputFiles("#bk-file", { name: bk.suggestedFilename(), mimeType: "application/octet-stream", buffer: bkBytes });
    check((await text("#bk-file-name")) === `Đã chọn: ${bk.suggestedFilename()}`, "TK-06: hiện tên tệp đã chọn");
    await page.fill("#bk-ipw", "sai mật khẩu");
    await page.click("#bk-open");
    await page.waitForSelector("#bk-imp-err:not([hidden])", { timeout: 120000 });
    check((await text("#bk-imp-err")) === "Sai mật khẩu hoặc tệp bị hỏng", "L10: sai mật khẩu → \"Sai mật khẩu hoặc tệp bị hỏng\"");
    await page.fill("#bk-ipw", "nhà có cây bàng");
    await page.click("#bk-open");
    await page.waitForSelector(".imp-dialog[open]", { timeout: 120000 });
    check((await text(".imp-dialog")).includes("Tệp xuất ngày 25/07/2026 từ ứng dụng web: 3 người, 4 sự kiện, 1 ảnh."), `L10: xem trước tệp (${(await text(".imp-dialog")).split("\n")[1]})`);
    await shot("import-dialog-360", false);
    await page.check("#bk-mode-replace");
    await page.click(".imp-dialog .dbtn.main");
    await page.waitForSelector('.pdialog[open] .dbtn.danger');
    await page.click('.pdialog[open] .dbtn.danger'); // MA3-15-03 (Esc trong lúc khôi phục): kiểm tất định ở runEscGate()
    await page.waitForSelector('.toast:has-text("Đã khôi phục: 3 người, 4 sự kiện.")', { timeout: 60000 });
    const h2 = await store(`async (s) => s.stateHash()`);
    check(h1 === h2, "L9/L10: xuất → xóa một hồ sơ → nhập \"Thay toàn bộ\" → dữ liệu giống hệt (stateHash)");
    // TK-16: sau "Thay toàn bộ", trang nói thật: dữ liệu khôi phục từ tệp ngày …; trên máy này chưa sao lưu.
    await page.waitForSelector("#bk-restored");
    const lastAfter = await text("#bk-last"), restoredTxt = await text("#bk-restored");
    check(restoredTxt === `Dữ liệu được khôi phục từ bản sao lưu ngày ${TODAY.slice(8, 10)}/${TODAY.slice(5, 7)}/${TODAY.slice(0, 4)}.` && /^(Chưa sao lưu lần nào trên máy này|Lần sao lưu gần nhất: )/.test(lastAfter) && h1 === h2,
      `TK-16: sau khôi phục "${lastAfter}" + "${restoredTxt}"; stateHash không đổi`);
    await shot("backup-restored-360");
    check((await text(".toast")).startsWith("Đã khôi phục: 3 người") && await page.locator(".pdialog[open]").count() === 0, "L10: khôi phục xong — có thông báo \"Đã khôi phục\", không còn hộp mở");

    // ---------- Ảnh chụp mọi màn hình mới: 360/1280 × sáng/tối; bố cục (TK-02) và vùng chạm (TK-05) ở A và A++ ----------
    const screens = [
      ["upcoming", "#/nguoi-than", "#upcoming-list"],
      ["period", "#/nguoi-than/theo-ky", "#period-list"],
      ["list", "#/nguoi-than/ho-so", "#list-deceased"],
      ["backup", "#/nguoi-than/sao-luu", "#bk-export"],
      ["person", `#/nguoi-than/nguoi/${idA}`, "#person-name"],
      ["form-deceased", `#/nguoi-than/sua/${idA}`, "#pf-ten"],
      ["form-deceased-new", "#/nguoi-than/them/da-mat", "#pf-ten"],
      ["form-living", "#/nguoi-than/them/dang-song", "#pf-ten"],
      ["form-event", `#/nguoi-than/su-kien/${idC}/moi`, "#ef-type"],
      ["memorial", `#/nguoi-than/tuong-niem/${idA}`, ".bia-name"],
      ["sheet-reminder", `#/ngay/${TODAY}`, ".remind-host .rcard"],
    ];
    const layoutBad = [], targetBad = [];
    const open = async (hash, sel) => {
      await page.goto(BASE + hash);
      await page.reload(); // mở như lần đầu (thẻ nhắc chạy lại ở bước khởi động)
      await page.waitForSelector(sel, { timeout: 15000 });
      if (/form-deceased|person|memorial/.test(hash) || /sua|nguoi\/|tuong-niem/.test(hash)) await page.waitForSelector("canvas[data-drawn]", { timeout: 3000 }).catch(() => {});
    };
    for (const [w, hgt] of [[360, 780], [1280, 800]]) {
      await page.setViewportSize({ width: w, height: hgt });
      for (const scheme of ["light", "dark"]) {
        await page.emulateMedia({ colorScheme: scheme });
        for (const [name, hash, sel] of screens) {
          await open(hash, sel);
          await shot(`${name}-${w}-${scheme}`, name !== "sheet-reminder");
          const m = await layout();
          if (!layoutOk(m)) layoutBad.push(`${name} ${w}px ${scheme} A: ${layoutTxt(m)}`);
          if (scheme === "light") for (const t of await measureTargets(name)) targetBad.push(`${name} ${w}px A: ${t}`);
        }
      }
    }
    // A++ (360 px và 1280 px, sáng)
    await page.emulateMedia({ colorScheme: "light" });
    for (const [w, hgt] of [[360, 780], [1280, 800]]) {
      await page.setViewportSize({ width: w, height: hgt });
      await page.goto(BASE + "#/nguoi-than"); await page.reload(); await page.waitForSelector(".sizes");
      await page.click('.sizes button[data-v="3"]');
      for (const [name, hash, sel] of screens) {
        await open(hash, sel);
        const m = await layout();
        if (!layoutOk(m)) layoutBad.push(`${name} ${w}px A++: ${layoutTxt(m)}`);
        if (w === 360) for (const t of await measureTargets(name)) targetBad.push(`${name} 360px A++: ${t}`);
        if (w === 360 && (name === "period" || name === "form-deceased" || name === "upcoming" || name === "person")) await shot(`${name}-360-xl`);
      }
      await page.click('.sizes button[data-v="1"]');
    }
    check(layoutBad.length === 0, `TK-02: mọi màn hình mới (${screens.length} × 360/1280 × sáng/tối ở A, và A++) không tràn ngang, thanh mục đáy trong khung nhìn${layoutBad.length ? " — " + layoutBad.slice(0, 6).join(" | ") : ""}`);
    check(shotWidthBad.length === 0, `TK-02: mọi ảnh chụp rộng đúng ${DPR} × khung nhìn (${shots.length} ảnh)${shotWidthBad.length ? " — " + shotWidthBad.slice(0, 6).join(" | ") : ""}`);
    check(targetBad.length === 0 && targetsMeasured > 200, `TK-05: mọi vùng chạm ≥ 44 × 44 px (A ở 360/1280 px, A++ ở 360 px; đã đo ${targetsMeasured} phần tử)${targetBad.length ? " — " + targetBad.slice(0, 8).join(" | ") : ""}`);
    metrics.people_layout = { layout_failures: layoutBad, screenshot_width_failures: shotWidthBad, small_targets: targetBad };

    await page.setViewportSize({ width: 360, height: 780 });
    await page.emulateMedia({ colorScheme: "light", reducedMotion: "no-preference" });
    // R8 (phần kiểm được trong kiểm thử khói): khói chạy bằng CSS khi hiển thị, nút Dừng khói / Thắp lại, không rAF.
    await page.addInitScript(() => { window.__raf = 0; const r = window.requestAnimationFrame.bind(window); window.requestAnimationFrame = (f) => { window.__raf++; return r(f); }; });
    await page.goto(BASE + `#/nguoi-than/tuong-niem/${idA}`);
    await page.reload();
    await page.waitForSelector(".mem-scene.lit");
    const anim = await page.evaluate(() => getComputedStyle(document.querySelector(".smoke")).animationName);
    await page.click("#mem-toggle");
    const stopped = await page.evaluate(() => !document.querySelector(".mem-scene").classList.contains("lit") && document.querySelector("#mem-toggle").textContent === "Thắp lại");
    const raf = await page.evaluate(() => window.__raf);
    check(anim === "smokeRise" && stopped && raf === 0, `R8: khói CSS chạy khi mở (${anim}), "Dừng khói" dừng ngay, 0 lần requestAnimationFrame (${raf})`);
    await page.emulateMedia({ reducedMotion: "reduce" });

    // ---------- TK-35 (phần còn lại) ----------
    // (a) Tên rất dài trên bia: chữ nhỏ lại (lớp .long), không tràn khung.
    const idD = await store(`async (s) => (await s.persons()).find((p) => p.ho_ten === "Lê Văn D").id`);
    await go(`#/nguoi-than/sua/${idD}`, "#pf-ten");
    await page.fill("#pf-ten", "Nguyễn Thị Hoàng Bích Ngọc Phương Thảo Diễm Kiều");
    await page.click(".form-acts .primary");
    await page.waitForSelector("#person-name");
    await go(`#/nguoi-than/tuong-niem/${idD}`, ".bia-name");
    const longM = await layout();
    check(await page.locator(".bia-name.long").count() === 1 && layoutOk(longM) && (await page.locator(".bia-top").count()) === 0,
      `TK-35: tên dài trên bia → chữ nhỏ lại, không tràn (${layoutTxt(longM)}); quan hệ trống → không có dòng trên tên`);
    await shot("memorial-longname-360");
    // (b) Nhắc sao lưu đến hạn: thay đổi chưa sao lưu từ 70 ngày trước → khung "Sao lưu ngay".
    const pend = await store(`async (s) => s.backupPendingSince()`);
    const setPend = (v) => store(`async (s) => new Promise((ok) => { const tx = s.db.transaction("meta", "readwrite"); tx.objectStore("meta").put(${JSON.stringify(v)}, "backup_pending_since"); tx.oncomplete = () => ok(true); })`);
    await setPend("2026-05-16T02:00:00Z");
    await go("#/nguoi-than", "#upcoming-list");
    const nag = await page.locator("#nag-backup").count() ? await text("#nag-backup") : "";
    const nagDays = await store(`async (s) => (await s.settings()).backup_nag_days`);
    check(nag.includes(`đã thay đổi hơn ${nagDays} ngày mà chưa sao lưu`) && await page.locator('#nag-backup a[href="#/nguoi-than/sao-luu"]').count() === 1, `TK-35: thay đổi chưa sao lưu 70 ngày → nhắc sao lưu có "Sao lưu ngay" (${nag.replace(/\n/g, " ")})`);
    await shot("backup-nag-360");
    if (pend) await setPend(pend);
    // (c) Hơn 3 lời nhắc hôm nay: 3 thẻ + "… và n lời nhắc khác"; trên Tờ lịch: 1 thẻ + "Còn n lời nhắc khác hôm nay — Xem".
    for (const nm of ["Phạm Văn E", "Phạm Thị G", "Phạm Văn H", "Phạm Thị K"]) {
      await go("#/nguoi-than/them/dang-song", "#pf-ten");
      await page.fill("#pf-ten", nm);
      await page.fill("#pf-birth-d", "25"); await page.fill("#pf-birth-m", "7"); await page.fill("#pf-birth-y", "2000");
      await page.click(".form-acts .primary");
      await page.waitForSelector("#person-name");
    }
    await go("#/nguoi-than", "#upcoming-list");
    const nCards = await page.locator(".people-body .remind .rcard").count();
    const moreUp = await page.locator(".people-body .remind .rc-more").count() ? await text(".people-body .remind .rc-more") : "";
    const nMore = Number((/và (\d+) lời nhắc khác/.exec(moreUp) ?? [])[1] ?? 0);
    check(nCards === 3 && nMore >= 1, `TK-35: ${3 + nMore} lời nhắc hôm nay → 3 thẻ + "${moreUp}"`);
    await shot("reminders-many-360");
    await page.goto(BASE + "#/ngay");
    await page.reload(); // thẻ nhắc trên Tờ lịch được dựng một lần khi mở ứng dụng
    await page.waitForSelector(".remind-host .rcard", { timeout: 15000 });
    const moreSheet = await text(".remind-host .rc-more");
    check(await page.locator(".remind-host .rcard").count() === 1 && moreSheet === `Còn ${nMore + 2} lời nhắc khác hôm nay — Xem`, `TK-35: Tờ lịch → 1 thẻ + "${moreSheet}"`);
    // TK-31 (b): ở 1280 px, thẻ nhắc thẳng cột trái với tờ lịch.
    await page.setViewportSize({ width: 1280, height: 800 });
    const al = await page.evaluate(() => { const c = document.querySelector(".remind-host .rcard").getBoundingClientRect(), s = document.querySelector(".daylayout > *").getBoundingClientRect(); return [Math.round(c.left), Math.round(s.left), Math.round(c.width), Math.round(s.width)]; });
    check(Math.abs(al[0] - al[1]) <= 1 && Math.abs(al[2] - al[3]) <= 1, `TK-31 (b): 1280 px — thẻ nhắc trái ${al[0]} / tờ lịch ${al[1]}, rộng ${al[2]} / ${al[3]}`);
    await shot("sheet-reminders-many-1280", false);
    await page.setViewportSize({ width: 360, height: 780 });
    for (const nm of ["Phạm Văn E", "Phạm Thị G", "Phạm Văn H", "Phạm Thị K"]) await store(`async (s) => { const p = (await s.persons()).find((x) => x.ho_ten === "${nm}"); await s.deletePerson(p.id); }`);
    // (d) Lỗi kho: kho trên máy có phiên bản mới hơn ứng dụng → câu báo, lịch vẫn dùng được.
    const ctx3 = await browser.newContext({ viewport: { width: 360, height: 780 }, locale: "vi-VN", timezoneId: "Asia/Ho_Chi_Minh" });
    const p3 = await ctx3.newPage();
    await p3.goto(BASE + "#/ngay");
    await p3.waitForSelector(".tabbar");
    await p3.evaluate(() => new Promise((ok) => { const r = indexedDB.open("lvhv", 99); r.onupgradeneeded = () => r.result.createObjectStore("x"); r.onsuccess = () => { r.result.close(); ok(); }; }));
    await p3.goto(BASE + "#/nguoi-than");
    await p3.reload();
    await p3.waitForSelector(".people-body .warn", { timeout: 15000 });
    const se = await p3.locator(".people-body .warn").innerText();
    check(se.startsWith("Không mở được dữ liệu trên máy") && se.includes("Lịch vẫn dùng được bình thường"), `TK-35: kho phiên bản mới hơn → "${se}"`);
    await p3.screenshot({ path: join(SHOTS, "v02-store-error-360.png"), fullPage: true }); shots.push("v02-store-error-360.png");
    await p3.click('.tab[data-r="ngay"]');
    await p3.waitForSelector(".daylayout");
    check(await p3.locator(".daylayout").count() === 1, "TK-35: sau lỗi kho, Tờ lịch vẫn mở được");
    await ctx3.close();

    // ---------- YC-2026-05 / US-M-07: "Cập nhật: đã mất" + thông tin báo tử (01 §3.2a, R-PRIV-2) ----------
    await page.emulateMedia({ colorScheme: "light", reducedMotion: "reduce" });
    await go("#/nguoi-than/them/dang-song", "#pf-ten");
    await page.fill("#pf-ten", "Đỗ Văn M"); await page.fill("#pf-qh", "Bác");
    await page.fill("#pf-birth-d", "1"); await page.fill("#pf-birth-m", "8"); await page.fill("#pf-birth-y", "1950");
    await page.click(".form-acts .primary");
    await page.waitForSelector("#person-name");
    const idM = /nguoi\/([0-9a-f-]+)/.exec(page.url())[1];
    await page.click("#add-event");
    await page.waitForSelector("#ef-type");
    await page.selectOption("#ef-type", "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0002");
    await page.fill("#ef-date-d", "30"); await page.fill("#ef-date-m", "7"); await page.fill("#ef-date-y", "1975");
    await page.click(".form-acts .primary");
    await page.waitForSelector("#event-box .evrow");
    const mItems = async (sel) => page.$$eval(`${sel} .pitem`, (els) => els.filter((e) => e.querySelector(".pi-l1")?.textContent.includes("Đỗ Văn M")).map((e) => e.dataset.class));
    await go("#/nguoi-than", "#upcoming-list");
    const beforeUp = await mItems("#upcoming-list");
    check(beforeUp.includes("sinh_nhat") && beforeUp.some((c) => c.startsWith("su_kien")), `US-M-07: trước khi cập nhật, "Sắp tới" có sinh nhật và sự kiện của Đỗ Văn M (${beforeUp.join(", ")})`);
    const createdM = await store(`async (s) => (await s.person("${idM}")).created_at`);
    await go(`#/nguoi-than/nguoi/${idM}`, "#convert-person");
    check((await text("#convert-person")) === "Cập nhật: đã mất", "US-M-07: hồ sơ người đang sống có nút \"Cập nhật: đã mất\"");
    await page.click("#convert-person");
    await page.waitForSelector("#cf-death");
    check(await page.locator("#nf[open]").count() === 1, "US-M-07: khối \"Thông tin báo tử (không bắt buộc)\" mở sẵn trong biểu mẫu cập nhật");
    await page.click('#cf-death-cal button[data-v="lunar"]');
    await page.fill("#cf-death-d", "12"); await page.fill("#cf-death-m", "6"); await page.fill("#cf-death-y", "2024");
    const cvRes = await text("#cf-death-res");
    check(cvRes.startsWith("Dương lịch:"), `US-M-07: ngày mất âm 12/6/2024 → kết quả đổi lịch (${cvRes})`);
    const SEN = { by: "SENTINEL-BAOTIN-7731", place: "SENTINEL-NOIMAT-7731", reason: "SENTINEL-LYDO-7731", note: "SENTINEL-GHICHU-7731" };
    await page.fill("#nf-by", SEN.by); await page.fill("#nf-place", SEN.place); await page.fill("#nf-reason", SEN.reason); await page.fill("#nf-note", SEN.note);
    await page.fill("#nf-date-d", "18"); await page.fill("#nf-date-m", "7"); await page.fill("#nf-date-y", "2024");
    // Tên tệp gốc cũng là chuỗi canh gác: chỉ được nằm trong bản ghi Attachment (R-PRIV-2, YC-2026-07).
    await page.selectOption("#nf-kind", "tin_bao_tu");
    await page.setInputFiles("#nf-file", { name: "SENTINEL-TENTEP-7731.jpg", mimeType: "image/jpeg", buffer: withExif });
    await page.waitForSelector(".cert-item canvas[data-drawn]");
    // YC-2026-07 + BM2-12: PDF — hộp báo "lưu nguyên bản" trước khi nhận; "Không đính kèm" → không thêm.
    const pdfBytes = Buffer.from("%PDF-1.4\n1 0 obj<</Type/Catalog>>endobj\n% SENTINEL-NOIDUNG-PDF\ntrailer<<>>\n%%EOF\n");
    await page.selectOption("#nf-kind", "giay_chung_tu");
    await page.setInputFiles("#nf-file", { name: "SENTINEL-TENPDF-7731.pdf", mimeType: "application/pdf", buffer: pdfBytes });
    await page.waitForSelector(".pdf-warn-dialog[open]");
    const pdfWarn = await text(".pdf-warn-dialog");
    await page.click(".pdf-warn-dialog .dbtn:not(.main)");
    check(pdfWarn.includes("PDF được lưu nguyên bản, có thể còn tên máy quét") && await page.locator("#nf-certs .cert-pdf").count() === 0, "BM2-12: chọn PDF → hộp báo lưu nguyên bản; \"Không đính kèm\" → không thêm");
    await page.setInputFiles("#nf-file", { name: "SENTINEL-TENPDF-7731.pdf", mimeType: "application/pdf", buffer: pdfBytes });
    await page.waitForSelector(".pdf-warn-dialog[open]");
    await page.click(".pdf-warn-dialog .dbtn.main");
    await page.waitForSelector("#nf-certs .cert-pdf");
    await page.setInputFiles("#nf-file", { name: "khong-phai.pdf", mimeType: "application/pdf", buffer: Buffer.from("hello") });
    await page.waitForSelector(".pdf-warn-dialog[open]");
    await page.click(".pdf-warn-dialog .dbtn.main");
    await page.waitForSelector("#nf .ferr:not([hidden])");
    check(await page.locator("#nf-certs .cert-item").count() === 2, "YC-2026-07: tệp .pdf không phải PDF → báo lỗi, không thêm; còn 2 tệp (ảnh tin báo tử + PDF giấy chứng tử)");
    const cvLay = await layout();
    check(layoutOk(cvLay), `US-M-07: biểu mẫu cập nhật không tràn ở 360 px (${layoutTxt(cvLay)})`);
    await shot("convert-form-360");
    await page.click(".form-acts .primary");
    await page.waitForSelector(".cv-dialog[open]");
    const cvTxt = (await text(".cv-dialog")).replace(/\s+/g, " ");
    check(cvTxt.includes("Sẽ không còn nhắc sinh nhật và sự kiện của Đỗ Văn M.") && cvTxt.includes("Bắt đầu nhắc ngày giỗ hằng năm vào 12/6 âm lịch") && cvTxt.includes("Họ tên, ảnh, ngày sinh và sự kiện vẫn được giữ"),
      `US-M-07: hộp xác nhận liệt kê thay đổi (${cvTxt.slice(0, 220)})`);
    await shot("convert-confirm-360", false);
    await page.click(".cv-dialog .dbtn.main");
    await page.waitForSelector("#person-name");
    const recM = await store(`async (s) => { const p = await s.person("${idM}"); const n = p.death.notice; const ph = new Uint8Array(await s.attachmentBytes(n.attachments[0].file_id)); const am = (await s.attachments()).find((a) => a.id === n.attachments[0].file_id); const pdf = (await s.attachments()).find((a) => a.id === n.attachments[1].file_id); const pdfB = await s.attachmentBytes(n.attachments[1].file_id); let i = 2, app1 = false; while (i + 4 < ph.length && ph[i] === 0xff && ph[i+1] !== 0xda) { if (ph[i+1] === 0xe1 || ph[i+1] === 0xed) app1 = true; i += 2 + ((ph[i+2] << 8) | ph[i+3]); } return { am, pdf, pdfB64: btoa(String.fromCharCode(...pdfB)), id: p.id, kind: p.kind, created: p.created_at, rev: p.rev, birth: p.birth && p.birth.input.year, sn: p.reminders.sinh_nhat, gio: !!p.reminders.gio, n, app1, evs: (await s.events()).filter((e) => e.person_ids.includes(p.id)).length }; }`);
    check(recM.id === idM && page.url().endsWith(`/nguoi/${idM}`) && recM.kind === "deceased" && recM.created === createdM && recM.rev === 2 && recM.birth === 1950 && recM.sn === null && recM.gio && recM.evs === 1,
      `US-M-07: CÙNG hồ sơ (id giữ nguyên, rev 2) chuyển sang đã mất; giữ ngày sinh và sự kiện; nhắc giỗ bật, không còn nhắc sinh nhật (${JSON.stringify({ kind: recM.kind, rev: recM.rev, evs: recM.evs })})`);
    check(recM.n.reported_by === SEN.by && recM.n.place === SEN.place && recM.n.reason === SEN.reason && recM.n.note === SEN.note && recM.n.notice_date.day === 18 && recM.n.attachments.length === 2 && recM.n.attachments[0].kind === "tin_bao_tu" && recM.n.attachments[1].kind === "giay_chung_tu"
      && recM.am.file_name === "SENTINEL-TENTEP-7731.jpg" && recM.am.stripped === true,
      "01 §3.2a, §3.8a: lưu death.notice đủ trường (tệp đính kèm loại giấy chứng tử, tên gốc trong bản ghi Attachment)");
    check(recM.app1 === false, "S9: ảnh đính kèm đã mã hóa lại, không còn EXIF/GPS (APP1/APP13)");
    check(Buffer.from(recM.pdfB64, "base64").equals(pdfBytes) && recM.pdf.stripped === false && recM.pdf.mime === "application/pdf", "01 §3.8a: PDF lưu nguyên byte (stripped = false)");
    // Tài liệu sao lưu có notice qua được kiểm tra; trỏ tới ảnh không có → V3; notice có chuỗi rỗng → V1.
    const vdoc = await page.evaluate(`(async () => { const S = await window.__lvhv.storage.loadStore(); const s = await S.openStore(); try {
      const doc = await s.snapshot({ version: "0.1.4", platform: "pwa" }, "2026-07-25T02:00:00Z");
      const r = []; const tryV = (d) => { try { S.validate.document(JSON.parse(JSON.stringify(d))); return "OK"; } catch (e) { return (e.code || "?") + " " + (e.path || ""); } };
      r.push(tryV(doc));
      const m = doc.persons.find((p) => p.id === "${idM}");
      const bad1 = JSON.parse(JSON.stringify(doc)); bad1.persons.find((p) => p.id === "${idM}").death.notice.attachments[0].file_id = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e6f"; r.push(tryV(bad1));
      const bad2 = JSON.parse(JSON.stringify(doc)); bad2.persons.find((p) => p.id === "${idM}").death.notice.place = ""; r.push(tryV(bad2));
      r.push(String((doc.attachments || []).some((a) => a.id === m.death.notice.attachments[0].file_id)));
      return r; } finally { s.close(); } })()`);
    check(vdoc[0] === "OK" && vdoc[1].startsWith("V3") && vdoc[2].startsWith("V1") && vdoc[3] === "true", `01 §3.2a: tài liệu sao lưu có notice hợp lệ, tệp đính kèm có trong attachments; tệp không tồn tại → V3; chuỗi rỗng → V1 (${vdoc.join(" | ")})`);
    const nb = page.locator("#notice-box");
    check(await nb.count() === 1 && await page.locator("#notice-box[open]").count() === 0, "Trang hồ sơ: khối \"Thông tin báo tử\" thu gọn");
    await page.click("#notice-box > summary");
    await page.waitForSelector("#notice-box .cert-thumb[data-drawn]");
    const nbTxt = await text("#notice-box");
    check(nbTxt.includes(SEN.reason) && nbTxt.includes("18/07/2024") && nbTxt.includes("Tin báo tử") && nbTxt.includes("Giấy chứng tử") && nbTxt.includes("Tệp PDF được lưu nguyên bản"),
      "Trang hồ sơ: mở khối thì thấy lý do, ngày nhận tin, hai tệp đính kèm theo loại");
    // S10 / BM2-05: tải tệp đính kèm phải qua hộp xác nhận; "Không tải" → không có tệp nào được tải.
    let dlCount = 0; const onDl = () => { dlCount++; }; page.on("download", onDl);
    await page.click('#notice-box .att-dl[data-i="1"]');
    await page.waitForSelector(".att-dl-dialog[open]");
    const dlTxt = await text(".att-dl-dialog");
    await page.click(".att-dl-dialog .dbtn:not(.main)");
    await page.waitForTimeout(500);
    page.off("download", onDl);
    check(dlTxt.includes("Tải tệp xuống máy?") && dlTxt.includes("Tệp có thể chứa thông tin riêng tư.") && dlTxt.includes("Tệp PDF còn thông tin của máy quét") && dlCount === 0,
      "S10 / BM2-05: tải tệp đính kèm → hộp \"Tải tệp xuống máy?\"; \"Không tải\" → 0 tệp tải xuống");
    await page.click('#notice-box .att-dl[data-i="1"]');
    await page.waitForSelector(".att-dl-dialog[open]");
    await shot("attachment-download-dialog-360", false);
    const [pdfDl] = await Promise.all([page.waitForEvent("download"), page.click(".att-dl-dialog .dbtn.main")]);
    check(readFileSync(await pdfDl.path()).equals(pdfBytes) && pdfDl.suggestedFilename() === "TepDinhKem-2.pdf", `YC-2026-07: tải PDF từ trang hồ sơ → đúng từng byte, tên tệp chung "${pdfDl.suggestedFilename()}" (không lộ tên gốc)`);
    await shot("person-notice-360");
    // 04 B6 mục 3 / BM2-10: byte độc đã có trong kho từ trước (ghi thẳng IndexedDB) — "JPEG" khai 20000 × 20000 → không
    // giải mã khi hiển thị, khung trống, trang vẫn chạy.
    const bomb = Buffer.from([0xff, 0xd8, 0xff, 0xc0, 0x00, 0x11, 0x08, 0x4e, 0x20, 0x4e, 0x20, 0x03, 0x01, 0x22, 0x00, 0x02, 0x11, 0x01, 0x03, 0x11, 0x01, 0xff, 0xd9]);
    const bombSaved = await page.evaluate(async ([b64, fid]) => {
      const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
      const db = await new Promise((ok) => { const r = indexedDB.open("lvhv"); r.onsuccess = () => ok(r.result); });
      const rec = await new Promise((ok) => { const r = db.transaction("attachments").objectStore("attachments").get(fid); r.onsuccess = () => ok(r.result); });
      const orig = new Uint8Array(await rec.blob.arrayBuffer()); let bin = ""; for (const x of orig) bin += String.fromCharCode(x);
      const hash = [...new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))].map((x) => x.toString(16).padStart(2, "0")).join("");
      await new Promise((ok) => { const tx = db.transaction("attachments", "readwrite"); tx.objectStore("attachments").put({ ...rec, meta: { ...rec.meta, bytes: bytes.length, sha256: hash }, blob: new Blob([bytes], { type: "image/jpeg" }) }); tx.oncomplete = ok; });
      db.close();
      return { rec: { ...rec, blob: null }, b64: btoa(bin) };
    }, [bomb.toString("base64"), recM.n.attachments[0].file_id]);
    await page.reload();
    await page.waitForSelector("#notice-box");
    await page.click("#notice-box > summary");
    await page.waitForSelector('#notice-box .cert-thumb[data-blocked="20000x20000"]', { timeout: 10000 });
    check(await page.locator('#notice-box .cert-thumb[data-blocked="20000x20000"]').count() === 1 && (await text("#person-name")) !== "", "04 B6 mục 3 / BM2-10: ảnh khai 20000 × 20000 trong kho → không giải mã khi hiển thị, khung trống, trang vẫn chạy");
    await page.evaluate(async (sv) => {
      const bytes = Uint8Array.from(atob(sv.b64), (c) => c.charCodeAt(0));
      const db = await new Promise((ok) => { const r = indexedDB.open("lvhv"); r.onsuccess = () => ok(r.result); });
      await new Promise((ok) => { const tx = db.transaction("attachments", "readwrite"); tx.objectStore("attachments").put({ ...sv.rec, blob: new Blob([bytes], { type: sv.rec.meta.mime }) }); tx.oncomplete = ok; });
      db.close();
    }, bombSaved);
    // B4 sau khi chuyển: sinh nhật và sự kiện của M không còn trong Sắp tới / Theo kỳ; giỗ hôm nay có thẻ nhắc.
    await page.goto(BASE + "#/nguoi-than"); await page.reload();
    await page.waitForSelector("#upcoming-list");
    const afterUp = await mItems("#upcoming-list");
    check(!afterUp.includes("sinh_nhat") && !afterUp.some((c) => c.startsWith("su_kien")) && afterUp.includes("gio"), `US-M-07 + B4: sau khi cập nhật, "Sắp tới" chỉ còn giỗ của Đỗ Văn M (${afterUp.join(", ")})`);
    const cards = await page.locator(".people-body .remind").innerText();
    check(cards.includes("Đỗ Văn M") && !cards.includes("SENTINEL"), "R-PRIV-2: thẻ nhắc giỗ hôm nay của Đỗ Văn M không có chữ nào của thông tin báo tử");
    await go("#/nguoi-than/theo-ky", "#period-title");
    await page.click('.pseg.four button[data-v="nam"]');
    const yearM = await mItems("#period-list");
    check(!yearM.includes("sinh_nhat") && !yearM.some((c) => c.startsWith("su_kien")) && yearM.includes("gio"), `US-M-07 + B4: Theo kỳ năm 2026 chỉ có giỗ của Đỗ Văn M (${yearM.join(", ")})`);
    await page.click('.pseg.four button[data-v="thang"]');
    await go(`#/nguoi-than/tuong-niem/${idM}`, ".bia-name");
    const memHtml = await page.evaluate(() => document.documentElement.outerHTML + document.title + location.href);
    check(!memHtml.includes("SENTINEL") && (await text(".bia-top")) === "Bác", "R-PRIV-2: màn hình tưởng niệm (DOM, tiêu đề, URL) không có thông tin báo tử");
    await go("#/nguoi-than/sao-luu", "#ics-open");
    await page.click("#ics-open");
    await page.waitForSelector(".ics-dialog[open]");
    const icsDlg = await text(".ics-dialog");
    const [dl2] = await Promise.all([page.waitForEvent("download"), page.click('.ics-dialog .dbtn.main')]);
    const ics2 = readFileSync(await dl2.path(), "utf8");
    check(ics2.includes("Đỗ Văn M") && !ics2.includes("SENTINEL") && !icsDlg.includes("SENTINEL") && !dl2.suggestedFilename().includes("SENTINEL"), "R-PRIV-2: tệp .ics (nội dung, xem trước, tên tệp) không có thông tin báo tử");
    await page.click("#bk-export");
    await page.waitForSelector(".bk-dialog[open]");
    check((await text("#bk-notice")).startsWith("Tệp gồm cả thông tin báo tử"), "R-PRIV-2: hộp tạo tệp sao lưu nói rõ tệp gồm cả thông tin báo tử");
    check((await text("#bk-pdf")).startsWith("Tệp gồm 1 tệp PDF đính kèm, lưu nguyên bản"), "BM2-12 / R-PDF-1: hộp tạo tệp sao lưu ghi số tệp PDF lưu nguyên bản");
    await shot("backup-notice-dialog-360", false);
    await page.click(".bk-dialog .dbtn:not(.main)");
    // Người đã mất thêm mới: khối báo tử thu gọn.
    await go("#/nguoi-than/them/da-mat", "#pf-ten");
    check(await page.locator("#nf").count() === 1 && await page.locator("#nf[open]").count() === 0, "YC-2026-05: biểu mẫu người đã mất mới có khối \"Thông tin báo tử\", thu gọn");
    // Ảnh chụp màn hình mới ở 1280 px và giao diện tối.
    for (const [w, hgt, theme] of [[1280, 800, "light"], [360, 780, "dark"], [1280, 800, "dark"]]) {
      await page.setViewportSize({ width: w, height: hgt });
      await page.emulateMedia({ colorScheme: theme });
      await go(`#/nguoi-than/bao-tu/${idC}`, "#cf-death");
      await page.click("#nf > summary").catch(() => {});
      await shot(`convert-form-${w}-${theme}`);
      await go(`#/nguoi-than/nguoi/${idM}`, "#notice-box");
      await page.click("#notice-box > summary");
      await shot(`person-notice-${w}-${theme}`);
    }
    await page.setViewportSize({ width: 360, height: 780 });
    await page.emulateMedia({ colorScheme: "light" });

    const realErrors = errors.filter((e) => !/Failed to load resource/.test(e));
    check(realErrors.length === 0, `Người thân: không có lỗi console (${realErrors.join(" | ") || "0"})`);
    check(external.length === 0, `Người thân: không gọi mạng ngoài (${external.length})`);
    check(csp.length === 0, `Người thân: 0 vi phạm CSP (${csp.join(" | ") || "0"})`);
    const ls = await page.evaluate(() => Object.keys(localStorage));
    check(ls.every((k) => k === "lvhv.ui"), `S4': localStorage chỉ có lvhv.ui (${JSON.stringify(ls)})`);
    const cdp = await ctx.newCDPSession(page);
    await page.goto(BASE + "#/nguoi-than");
    await page.reload();
    await page.waitForSelector("#upcoming-list");
    await cdp.send("HeapProfiler.collectGarbage");
    const heap = await cdp.send("Runtime.getHeapUsage");
    metrics.people = { screenshots: shots, heap_upcoming_mb: +(heap.usedSize / 1048576).toFixed(2), period: "tsport eventsInPeriod (memorial_period.ts)" };
  } catch (e) {
    await page.screenshot({ path: join(SHOTS, "v02-FAIL.png"), fullPage: true }).catch(() => {});
    const diag = await page.evaluate(() => ({ url: location.hash, dialogs: [...document.querySelectorAll("dialog[open]")].map((d) => d.innerText.slice(0, 300)), main: document.querySelector("main")?.innerText.slice(0, 600) })).catch(() => ({}));
    console.log("Người thân — chẩn đoán:", JSON.stringify(diag), "| lỗi console:", errors.join(" | "));
    await ctx.close();
    throw e;
  }
  await ctx.close();
  return shots;
}

/**
 * MA3-15-03, tất định: nhấn Esc trong lúc mã hóa sao lưu → Esc bị chặn, hộp vẫn mở, tệp vẫn được tải.
 * Chạy trên bản dựng KIỂM THỬ (LVHV_TEST_HOOKS=1, smoke.mjs dựng vào thư mục tạm): cổng "backup-encrypt"
 * (src/testhooks.ts) giữ bước mã hóa cho tới khi kiểm thử đã nhấn Esc và kiểm xong hộp còn mở, rồi mới cho chạy tiếp.
 * Không phụ thuộc mã hóa nhanh hay chậm. Bản dựng chính không có cổng (smoke.mjs kiểm).
 */
export async function runEscGate({ browser, BASE, check }) {
  const ctx = await browser.newContext({ viewport: { width: 360, height: 780 }, deviceScaleFactor: 2, hasTouch: true, isMobile: true, locale: "vi-VN", timezoneId: "Asia/Ho_Chi_Minh", acceptDownloads: true });
  const page = await ctx.newPage();
  await page.clock.setFixedTime(FIXED);
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  // Đường tải về (<a download>): tắt hộp "Lưu tệp" của hệ điều hành (showSaveFilePicker); đường đó kiểm ở runSavePicker().
  await page.addInitScript(() => { Object.defineProperty(window, "showSaveFilePicker", { value: undefined, configurable: true }); });
  await page.addInitScript(() => {
    window.__gate = { reached: 0, release: null, restoreReached: 0, restoreRelease: null };
    window.__lvhvGates = {
      "backup-encrypt": () => new Promise((ok) => { window.__gate.reached++; window.__gate.release = ok; }),
      "backup-restore": () => new Promise((ok) => { window.__gate.restoreReached++; window.__gate.restoreRelease = ok; }),
    };
    window.__esc = [];
    window.addEventListener("cancel", (e) => { const busy = e.target.getAttribute?.("aria-busy"); setTimeout(() => window.__esc.push({ busy, prevented: e.defaultPrevented })); }, true);
  });
  try {
    await page.goto(BASE + "#/nguoi-than/them/dang-song");
    await page.waitForSelector("#pf-ten", { timeout: 15000 });
    await page.fill("#pf-ten", "Trần Thị Esc");
    await page.fill("#pf-birth-d", "25"); await page.fill("#pf-birth-m", "7"); await page.fill("#pf-birth-y", "2000");
    await page.click(".form-acts .primary");
    await page.waitForSelector("#person-name", { timeout: 15000 });
    await page.goto(BASE + "#/nguoi-than/sao-luu");
    await page.waitForSelector("#bk-export:not([disabled])", { timeout: 15000 });
    await page.click("#bk-export");
    await page.fill("#bk-pw1", "nhà có cây bàng");
    await page.fill("#bk-pw2", "nhà có cây bàng");
    const dlP = page.waitForEvent("download", { timeout: 60000 });
    await page.click(".bk-dialog .dbtn.main");
    // Bước mã hóa đã tới cổng và đang bị giữ: hộp chắc chắn đang bận.
    await page.waitForFunction(() => window.__gate.reached === 1, null, { timeout: 15000 });
    const busy = await page.getAttribute(".bk-dialog", "aria-busy");
    await page.keyboard.press("Escape");
    await page.waitForFunction(() => window.__esc.length >= 1, null, { timeout: 5000 }).catch(() => {});
    const esc = await page.evaluate(() => window.__esc);
    const open = await page.locator(".bk-dialog[open]").count();
    const prog = await page.locator("#bk-prog:not([hidden])").count();
    check(busy === "true" && esc.length === 1 && esc[0].busy === "true" && esc[0].prevented === true && open === 1 && prog === 1,
      `MA3-15-03: nhấn Esc trong lúc mã hóa (bước mã hóa đang giữ) — Esc bị chặn, hộp vẫn mở và đang chạy (busy ${busy}, Esc ${JSON.stringify(esc)}, hộp mở ${open})`);
    await page.evaluate(() => window.__gate.release()); // cho bước mã hóa chạy tiếp
    const bk = await dlP.catch(() => null);
    await page.waitForSelector(".bk-dialog", { state: "detached", timeout: 30000 }).catch(() => {});
    const toast = await page.locator(".toast").first().innerText().catch(() => "");
    check(!!bk && bk.suggestedFilename() === `LichVanHoaViet-saoluu-${TODAY}.lvhv` && await page.locator(".bk-dialog").count() === 0 && toast.startsWith("Đã tạo tệp sao lưu"),
      `MA3-15-03: sau khi mã hóa xong — tệp vẫn được tải (${bk ? bk.suggestedFilename() : "không có"}), hộp đóng, có thông báo`);

    // Esc trong lúc khôi phục: mở chính tệp vừa tải, "Thay toàn bộ", giữ bước khôi phục ở cổng "backup-restore".
    await page.goto(BASE + "#/nguoi-than/sao-luu");
    await page.waitForSelector("#bk-file", { state: "attached", timeout: 15000 });
    await page.setInputFiles("#bk-file", { name: bk.suggestedFilename(), mimeType: "application/octet-stream", buffer: readFileSync(await bk.path()) });
    await page.fill("#bk-ipw", "nhà có cây bàng");
    await page.click("#bk-open");
    await page.waitForSelector(".imp-dialog[open]", { timeout: 60000 });
    await page.check("#bk-mode-replace");
    await page.click(".imp-dialog .dbtn.main");
    await page.waitForSelector(".pdialog[open] .dbtn.danger");
    await page.click(".pdialog[open] .dbtn.danger");
    await page.waitForFunction(() => window.__gate.restoreReached === 1, null, { timeout: 15000 });
    const rBusy = await page.getAttribute(".imp-dialog", "aria-busy");
    await page.evaluate(() => { window.__esc = []; });
    await page.keyboard.press("Escape");
    await page.waitForFunction(() => window.__esc.length >= 1, null, { timeout: 5000 }).catch(() => {});
    const rEsc = await page.evaluate(() => window.__esc);
    const rOpen = await page.locator(".imp-dialog[open]").count();
    check(rBusy === "true" && rEsc.length === 1 && rEsc[0].busy === "true" && rEsc[0].prevented === true && rOpen === 1,
      `MA3-15-03: nhấn Esc trong lúc khôi phục (bước khôi phục đang giữ) — Esc bị chặn, hộp vẫn mở và đang chạy (busy ${rBusy}, Esc ${JSON.stringify(rEsc)}, hộp mở ${rOpen})`);
    await page.evaluate(() => window.__gate.restoreRelease());
    const restored = await page.waitForSelector('.toast:has-text("Đã khôi phục: 1 người")', { timeout: 30000 }).then(() => true, () => false);
    check(restored && await page.locator(".pdialog[open]").count() === 0, "MA3-15-03: sau khi khôi phục xong — có thông báo \"Đã khôi phục\", không còn hộp mở");
    check(errors.length === 0, `MA3-15-03: không có lỗi trang (${errors.join(" | ") || "0"})`);
  } finally {
    await ctx.close();
  }
}

/**
 * MA3C-30-03: hộp "Lưu tệp" của hệ điều hành (showSaveFilePicker, Chromium trên máy tính). Chỉ ghi nhận "đã sao lưu"
 * sau khi ghi xong; hủy hộp hay ghi lỗi → hộp sao lưu báo rõ, không ghi nhận. Chạy trên bản dựng CHÍNH (dist/); hộp của
 * hệ điều hành được thay bằng bản giả trong trang (kiểm thử không bấm được hộp thật).
 */
export async function runSavePicker({ browser, BASE, check }) {
  const ctx = await browser.newContext({ viewport: { width: 360, height: 780 }, deviceScaleFactor: 2, hasTouch: true, isMobile: true, locale: "vi-VN", timezoneId: "Asia/Ho_Chi_Minh" });
  const page = await ctx.newPage();
  await page.clock.setFixedTime(FIXED);
  const errors = [], downloads = [];
  page.on("pageerror", (e) => errors.push(e.message));
  page.on("download", (d) => downloads.push(d.suggestedFilename()));
  await page.addInitScript(() => {
    window.__pick = { mode: "ok", calls: [], saved: [] };
    window.showSaveFilePicker = async (o) => {
      window.__pick.calls.push(o.suggestedName);
      if (window.__pick.mode === "cancel") throw new DOMException("Người dùng đóng hộp", "AbortError");
      const parts = [];
      return { createWritable: async () => ({
        write: async (d) => { parts.push(new Uint8Array(await new Blob([d]).arrayBuffer())); },
        close: async () => {
          if (window.__pick.mode === "fail") throw new DOMException("Ổ đầy", "QuotaExceededError");
          const n = parts.reduce((a, b) => a + b.length, 0), all = new Uint8Array(n); let i = 0;
          for (const b of parts) { all.set(b, i); i += b.length; }
          window.__pick.saved.push({ head: String.fromCharCode(...all.slice(0, 7)), bytes: n });
        },
      }) };
    };
  });
  try {
    await page.goto(BASE + "#/nguoi-than/them/dang-song");
    await page.waitForSelector("#pf-ten", { timeout: 15000 });
    await page.fill("#pf-ten", "Phạm Văn Lưu");
    await page.fill("#pf-birth-d", "2"); await page.fill("#pf-birth-m", "3"); await page.fill("#pf-birth-y", "1985");
    await page.click(".form-acts .primary");
    await page.waitForSelector("#person-name", { timeout: 15000 });
    const openBackup = async () => {
      await page.goto(BASE + "#/nguoi-than/sao-luu"); await page.waitForSelector("#bk-export:not([disabled])", { timeout: 15000 });
      await page.click("#bk-export");
      await page.fill("#bk-pw1", "nhà có cây bàng"); await page.fill("#bk-pw2", "nhà có cây bàng");
    };
    const lastTxt = async () => (await page.innerText("#bk-last")).trim();
    const last0 = await (async () => { await page.goto(BASE + "#/nguoi-than/sao-luu"); await page.waitForSelector("#bk-last"); return lastTxt(); })();

    // 1. Hủy hộp chọn nơi lưu → không mã hóa, không ghi nhận, hộp sao lưu vẫn mở và nói rõ.
    await openBackup();
    await page.evaluate(() => { window.__pick.mode = "cancel"; });
    await page.click(".bk-dialog .dbtn.main");
    await page.waitForSelector(".bk-dialog .ferr:not([hidden])", { timeout: 15000 });
    const cancelMsg = await page.innerText(".bk-dialog .ferr:not([hidden])");
    const cancelOpen = await page.locator(".bk-dialog[open]").count();
    await page.click(".bk-dialog .dbtn:not(.main)");
    await page.waitForSelector(".bk-dialog", { state: "detached" });
    const lastAfterCancel = await (async () => { await page.goto(BASE + "#/nguoi-than/sao-luu"); await page.waitForSelector("#bk-last"); return lastTxt(); })();
    check(cancelOpen === 1 && cancelMsg.startsWith("Chưa lưu tệp") && lastAfterCancel === last0,
      `MA3C-30-03: đóng hộp chọn nơi lưu → hộp sao lưu vẫn mở, báo "${cancelMsg.split("—")[0].trim()}", không ghi nhận đã sao lưu ("${lastAfterCancel}")`);

    // 2. Ghi lỗi (ổ đầy) → báo lỗi, không ghi nhận.
    await openBackup();
    await page.evaluate(() => { window.__pick.mode = "fail"; });
    await page.click(".bk-dialog .dbtn.main");
    await page.waitForSelector(".bk-dialog .ferr:not([hidden])", { timeout: 60000 });
    const failMsg = await page.innerText(".bk-dialog .ferr:not([hidden])");
    await page.click(".bk-dialog .dbtn:not(.main)");
    await page.waitForSelector(".bk-dialog", { state: "detached" });
    const lastAfterFail = await (async () => { await page.goto(BASE + "#/nguoi-than/sao-luu"); await page.waitForSelector("#bk-last"); return lastTxt(); })();
    check(failMsg.startsWith("Chưa lưu được tệp") && lastAfterFail === last0, `MA3C-30-03: ghi tệp lỗi → báo "${failMsg.split("(")[0].trim()}", không ghi nhận đã sao lưu`);

    // 3. Lưu xong → có tệp đúng định dạng, thông báo thành công, "Lần sao lưu gần nhất" cập nhật; không có tải về.
    await openBackup();
    await page.evaluate(() => { window.__pick.mode = "ok"; });
    await page.click(".bk-dialog .dbtn.main");
    await page.waitForSelector(".bk-dialog", { state: "detached", timeout: 60000 });
    const toast = await page.locator(".toast").first().innerText().catch(() => "");
    await page.waitForSelector('#bk-last:has-text("Lần sao lưu gần nhất")', { timeout: 15000 }).catch(() => {});
    const pick = await page.evaluate(() => window.__pick);
    const lastOk = await lastTxt();
    check(pick.saved.length === 1 && pick.saved[0].head === "LVHVBAK" && pick.calls.every((n) => n === `LichVanHoaViet-saoluu-${TODAY}.lvhv`) && toast.startsWith("Đã tạo tệp sao lưu")
      && lastOk === `Lần sao lưu gần nhất: ${TODAY.slice(8, 10)}/${TODAY.slice(5, 7)}/${TODAY.slice(0, 4)}` && downloads.length === 0,
      `MA3C-30-03: lưu qua hộp "Lưu tệp" → tệp ${pick.saved[0]?.bytes ?? 0} byte, ghi nhận sau khi ghi xong ("${lastOk}"), không dùng tải về (${downloads.length})`);
    check(errors.length === 0, `MA3C-30-03: không có lỗi trang (${errors.join(" | ") || "0"})`);
  } finally {
    await ctx.close();
  }
}
