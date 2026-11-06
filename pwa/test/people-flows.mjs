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
  const errors = [], external = [], csp = [];
  page.on("console", (m) => { if (m.type() === "error" || m.type() === "warning") errors.push(`${m.type()}: ${m.text()}`); if (/Content.Security.Policy/i.test(m.text())) csp.push(m.text()); });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("request", (r) => { const u = r.url(); if (!u.startsWith(ORIGIN) && !u.startsWith("data:") && !u.startsWith("blob:")) external.push(u); });
  const shots = [];
  const shot = async (name, full = true) => { const f = `v02-${name}.png`; await page.screenshot({ path: join(SHOTS, f), fullPage: full }); shots.push(f); };
  const go = async (hash, sel) => { await page.goto(BASE + hash); await page.waitForSelector(sel ?? ".people-body > *", { timeout: 15000 }); };
  const text = (sel) => page.locator(sel).first().innerText();
  const overflow = () => page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  // Biểu thức chuỗi chạy qua CDP (không dùng eval trong trang — CSP chặn eval, đúng như mong muốn).
  const store = (fn) => page.evaluate(`(async () => { const S = await window.__lvhv.storage.loadStore(); const s = await S.openStore(); try { return await (${fn})(s); } finally { s.close(); } })()`);

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
  await page.fill("#pf-dx", "Ông");
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
  await page.setInputFiles("#pf-file", { name: "chan-dung.jpg", mimeType: "image/jpeg", buffer: withExif });
  await page.waitForSelector(".pf-photo[data-drawn]");
  await page.click(".pform details.more > summary");
  await page.fill("#pf-bia", "Hiển khảo");
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
  await page.fill("#pf-ten", "Lê Văn C"); await page.fill("#pf-qh", "con trai");
  await page.fill("#pf-birth-d", "20"); await page.fill("#pf-birth-m", "1"); await page.fill("#pf-birth-y", "1990");
  const res3 = await text("#pf-birth-res");
  check(res3.includes("Âm lịch: 24/12 năm Kỷ Tỵ (1989)") && res3.includes("Hôm nay 36 tuổi (tuổi âm 38)"), `L3: sinh 20/01/1990 → Kỷ Tỵ (1989), 36 tuổi (tuổi âm 38) (${res3})`);
  await page.check("#pf-snam");
  await shot("form-living-360");
  await page.click(".form-acts .primary");
  await page.waitForSelector("#person-name");
  const idC = /nguoi\/([0-9a-f-]+)/.exec(page.url())[1];
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

  // ---------- L5: Theo kỳ (R13) ----------
  const ds = M.parseDataset(await store(`async (s) => JSON.stringify({ settings: await s.settings(), persons: await s.persons(), events: await s.events(), event_types: await s.eventTypes() })`));
  const ymd = (s) => ({ y: +s.slice(0, 4), m: +s.slice(5, 7), d: +s.slice(8, 10) });
  const isoOf = (d) => `${d.y}-${String(d.m).padStart(2, "0")}-${String(d.d).padStart(2, "0")}`;
  // Kỳ tính độc lập bằng Date (UTC), không dùng bộ chuyển của PWA.
  const bounds = (kind, a) => {
    const t = Date.UTC(a.y, a.m - 1, a.d), D = (ms) => { const x = new Date(ms); return { y: x.getUTCFullYear(), m: x.getUTCMonth() + 1, d: x.getUTCDate() }; };
    if (kind === "tuan") { const wd = (new Date(t).getUTCDay() + 6) % 7; return [D(t - wd * 864e5), D(t + (6 - wd) * 864e5)]; }
    if (kind === "thang") return [D(Date.UTC(a.y, a.m - 1, 1)), D(Date.UTC(a.y, a.m, 0))];
    if (kind === "quy") { const q = Math.floor((a.m - 1) / 3) * 3; return [D(Date.UTC(a.y, q, 1)), D(Date.UTC(a.y, q + 3, 0))]; }
    return [D(Date.UTC(a.y, 0, 1)), D(Date.UTC(a.y, 11, 31))];
  };
  const kindOf = new Map(ds.persons.map((p) => [p.id, p.kind]));
  const expected = (kind, anchor, filter) => {
    const [from, to] = bounds(kind, anchor);
    const want = filter === "da_mat" ? "deceased" : "living";
    const d2 = filter === "tat_ca" ? ds : { ...ds, persons: ds.persons.filter((p) => p.kind === want), events: ds.events.filter((e) => (kindOf.get(e.person_ids[0]) ?? "living") === want) };
    return M.occurrencesWithTt(cal, d2, from, to).map((o) => `${o.class}@${isoOf(o.date)}`);
  };
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
  check((await text("#period-title")) === "Tuần 20/07 – 26/07/2026" && await page.locator("#period-today").isDisabled(), "L5: Hôm nay → tuần chứa hôm nay, nút Hôm nay tắt");
  await page.click("#period-next");
  const wk = await shown();
  check(wk.filter((x) => x.startsWith("gio@2026-07-28")).length === 2, `L5: tuần 27/07 – 02/08 có 2 giỗ đầu ngày 28/07 (${wk.join(",")})`);
  await page.click('.pseg.three button[data-v="dang_song"]');
  check((await text("#period-list")).includes("Không có ngày giỗ, sinh nhật hay sự kiện nào trong kỳ này. (đang lọc: Người đang sống)"), "L5: trạng thái trống có ghi bộ lọc");
  await page.click('.pseg.three button[data-v="tat_ca"]');
  await page.click('.pseg.four button[data-v="nam"]');
  check(await page.locator("#period-list .pmonth").count() >= 3, "R-UX-2: danh sách năm chia nhóm theo tháng");
  // Tự kiểm tính nhất quán R13 (c) trên tập mong đợi: hợp tuần = tháng, hợp 3 tháng = quý, đã mất ∪ đang sống = tất cả
  const all = expected("nam", t0, "tat_ca");
  const union = [...expected("nam", t0, "da_mat"), ...expected("nam", t0, "dang_song")].sort();
  check(JSON.stringify(union) === JSON.stringify(all.slice().sort()), "R13 (c): đã mất ∪ đang sống = tất cả (năm 2026), không trùng");
  await page.locator("#period-list .pitem").first().click();
  await page.waitForSelector("#person-name");
  check(true, "L5: chạm một mục → mở hồ sơ");
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
  const nCards = await page.locator(".remind-host .rcard").count();
  await page.locator('.remind-host .rcard button:has-text("Đã biết")').first().click();
  check(await page.locator(".remind-host .rcard").count() === nCards - 1, `L7: "Đã biết" ẩn một thẻ (${nCards} → ${nCards - 1})`);

  // ---------- L12: sửa ----------
  await go(`#/nguoi-than/sua/${idA}`, "#pf-ten");
  check((await page.inputValue("#pf-ten")) === "Nguyễn Văn A" && (await text("#pf-death-res")).includes("15/6 nhuận"), "L12: biểu mẫu sửa điền sẵn, có kết quả đổi lịch");
  await page.fill("#pf-qh", "ông nội (bên nội)");
  await page.click(".form-acts .primary");
  await page.waitForSelector("#person-name");
  const rev2 = await store(`async (s) => { const p = await s.person("${idA}"); return [p.rev, p.quan_he, p.photo_id !== null]; }`);
  check(rev2[0] === 2 && rev2[1] === "ông nội (bên nội)" && rev2[2], `L12: sửa → rev = 2, giữ ảnh (${JSON.stringify(rev2)})`);

  // ---------- L11: tưởng niệm ----------
  await page.click("#open-memorial");
  await page.waitForSelector(".bia-name");
  const bia = (await page.locator(".bia").textContent()).replace(/\s+/g, " ");
  check(bia.includes("Hiển khảo") && bia.includes("Ông Nguyễn Văn A") && bia.includes("Canh Thìn 1940 — Ất Tỵ 2025") && bia.includes("Hưởng thọ 86 tuổi"), `L11: bia có dòng đầu, tên, năm Can Chi, hưởng thọ (${bia.replace(/\n/g, " | ")})`);
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
  check(/Tệp này chứa tên và ngày giỗ của 4 người\. Bất kỳ ai có tệp đều đọc được\./.test(icsTxt) && icsTxt.includes("Xem trước"), `L8 (S10): hộp xác nhận ghi số người + xem trước (${icsTxt.split("\n")[1]})`);
  await shot("ics-dialog-360", false);
  const [dl] = await Promise.all([page.waitForEvent("download"), page.click('.ics-dialog .dbtn.main')]);
  const ics = readFileSync(await dl.path(), "utf8");
  check(dl.suggestedFilename() === `LichVanHoaViet-ngay-gio-${TODAY}.ics` && ics.startsWith("BEGIN:VCALENDAR\r\n") && ics.includes("SUMMARY:Giỗ đầu Ông Nguyễn Văn A") && !ics.includes("RRULE"), `L8: tải ${dl.suggestedFilename()} (RFC 5545, không RRULE)`);

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
  const h1 = await store(`async (s) => s.stateHash()`);
  await page.click("#bk-export");
  await page.fill("#bk-pw1", "ngắn");
  await page.fill("#bk-pw2", "ngắn");
  await page.click(".bk-dialog .dbtn.main");
  check((await text(".bk-dialog .ferr")).includes("Mật khẩu cần ít nhất 8 ký tự"), "L9: mật khẩu ngắn → báo lỗi");
  await page.fill("#bk-pw1", "nhà có cây bàng");
  await page.fill("#bk-pw2", "nhà có cây bàng khác");
  await page.click(".bk-dialog .dbtn.main");
  check((await text(".bk-dialog .ferr")).includes("Hai lần nhập mật khẩu chưa giống nhau"), "L9: hai lần nhập khác nhau → báo lỗi");
  await page.fill("#bk-pw2", "nhà có cây bàng");
  await shot("backup-dialog-360", false);
  const [bk] = await Promise.all([page.waitForEvent("download", { timeout: 120000 }), page.click(".bk-dialog .dbtn.main")]);
  const bkBytes = readFileSync(await bk.path());
  check(bk.suggestedFilename() === `LichVanHoaViet-saoluu-${TODAY}.lvhv` && bkBytes.subarray(0, 7).toString() === "LVHVBAK", `L9: tải ${bk.suggestedFilename()} (${bkBytes.length} byte)`);
  await page.waitForSelector("#bk-last");
  check((await text("#bk-last")).includes(`Lần sao lưu gần nhất: 25/07/2026`), "L9: ghi lần sao lưu gần nhất");
  await store(`async (s) => { await s.deletePerson("${idC}"); }`);
  await page.setInputFiles("#bk-file", { name: bk.suggestedFilename(), mimeType: "application/octet-stream", buffer: bkBytes });
  await page.fill("#bk-ipw", "sai mật khẩu");
  await page.click("#bk-open");
  await page.waitForSelector("#bk-imp-err:not([hidden])", { timeout: 120000 });
  check((await text("#bk-imp-err")) === "Sai mật khẩu hoặc tệp bị hỏng", "L10: sai mật khẩu → \"Sai mật khẩu hoặc tệp bị hỏng\"");
  await page.fill("#bk-ipw", "nhà có cây bàng");
  await page.click("#bk-open");
  await page.waitForSelector(".imp-dialog[open]", { timeout: 120000 });
  check((await text(".imp-dialog")).includes("Tệp xuất ngày 25/07/2026 từ ứng dụng web: 3 người, 2 sự kiện, 1 ảnh."), `L10: xem trước tệp (${(await text(".imp-dialog")).split("\n")[1]})`);
  await shot("import-dialog-360", false);
  await page.check("#bk-mode-replace");
  await page.click(".imp-dialog .dbtn.main");
  await page.waitForSelector('.pdialog[open] .dbtn.danger');
  await page.click('.pdialog[open] .dbtn.danger');
  await page.waitForSelector('.toast:has-text("Đã khôi phục: 3 người, 2 sự kiện.")', { timeout: 60000 });
  const h2 = await store(`async (s) => s.stateHash()`);
  check(h1 === h2, "L9/L10: xuất → xóa một hồ sơ → nhập \"Thay toàn bộ\" → dữ liệu giống hệt (stateHash)");

  // ---------- Ảnh chụp mọi màn hình mới: 360/1280 × sáng/tối; A++ không tràn ----------
  const screens = [
    ["upcoming", "#/nguoi-than", "#upcoming-list"],
    ["period", "#/nguoi-than/theo-ky", "#period-list"],
    ["list", "#/nguoi-than/ho-so", "#list-deceased"],
    ["backup", "#/nguoi-than/sao-luu", "#bk-export"],
    ["person", `#/nguoi-than/nguoi/${idA}`, "#person-name"],
    ["form-deceased", `#/nguoi-than/sua/${idA}`, "#pf-ten"],
    ["form-living", "#/nguoi-than/them/dang-song", "#pf-ten"],
    ["form-event", `#/nguoi-than/su-kien/${idC}/moi`, "#ef-type"],
    ["memorial", `#/nguoi-than/tuong-niem/${idA}`, ".bia-name"],
    ["sheet-reminder", `#/ngay/${TODAY}`, ".remind-host .rcard"],
  ];
  for (const [w, hgt] of [[360, 780], [1280, 800]]) {
    await page.setViewportSize({ width: w, height: hgt });
    for (const scheme of ["light", "dark"]) {
      await page.emulateMedia({ colorScheme: scheme });
      for (const [name, hash, sel] of screens) {
        await page.goto(BASE + hash);
        await page.reload(); // mở như lần đầu (thẻ nhắc chạy lại ở bước khởi động)
        await page.waitForSelector(sel, { timeout: 15000 });
        if (name === "memorial" || name === "person" || name === "form-deceased") await page.waitForSelector("canvas[data-drawn]", { timeout: 5000 }).catch(() => {});
        await shot(`${name}-${w}-${scheme}`, name !== "sheet-reminder");
        const ov = await overflow();
        if (ov > 0) check(false, `${name} ${w}px ${scheme}: tràn ngang ${ov}px`);
      }
    }
  }
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
  await page.click('.sizes button[data-v="3"]');
  let worst = 0;
  for (const [name, hash, sel] of screens) {
    await page.goto(BASE + hash);
    await page.reload();
    await page.waitForSelector(sel, { timeout: 15000 });
    const ov = await overflow();
    worst = Math.max(worst, ov);
    if (ov > 0) check(false, `A++ 360 px ${name}: tràn ngang ${ov}px`);
    if (name === "period" || name === "form-deceased" || name === "upcoming") await shot(`${name}-360-xl`);
  }
  check(worst <= 0, `A++ 360 px: mọi màn hình mới không tràn ngang (${worst}px)`);
  await page.click('.sizes button[data-v="1"]');

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
  metrics.people = { screenshots: shots, heap_upcoming_mb: +(heap.usedSize / 1048576).toFixed(2), adapter: "pwa/src/people/period.ts (bộ chuyển tạm, chờ tsport eventsInPeriod)" };
  await ctx.close();
  return shots;
}
