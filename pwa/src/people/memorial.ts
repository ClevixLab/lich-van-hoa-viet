// Màn hình tưởng niệm (docs/v0.2/08 §3.9; 05 §1; tiêu chí R8). Hình vẽ mới cho dự án, viết thẳng trong mã (SVG/CSS),
// không tệp ảnh ngoài, không âm thanh, không quảng cáo.
// Khói (R8): chỉ CSS @keyframes, KHÔNG requestAnimationFrame. Chạy khi trang hiển thị, tự dừng sau 2 phút (tính thời
// gian hiển thị), dừng ngay khi tab ẩn hoặc rời trang; prefers-reduced-motion → khói tĩnh từ đầu.
import { h, svg } from "../dom";
import { M, addDays, biaHeading, cal, daysBetween, displayName, lunarText, snap, store, today, tuoiText, type JsonObj } from "./core";
import { drawPhoto } from "./photo";
import { backLink } from "./ui";

export const SMOKE_MS = 120_000;

const BOWL = `<svg viewBox="0 0 120 74"><defs>
<linearGradient id="mbz" x1="0" x2="0" y1="0" y2="1"><stop offset="0" stop-color="#c99a4a"/><stop offset=".55" stop-color="#8a5a22"/><stop offset="1" stop-color="#5b3814"/></linearGradient>
<linearGradient id="mbr" x1="0" x2="1"><stop offset="0" stop-color="#7a4d1c"/><stop offset=".5" stop-color="#e2bb6a"/><stop offset="1" stop-color="#7a4d1c"/></linearGradient></defs>
<path d="M40 64h40l4 8H36z" fill="#5b3814"/>
<path d="M8 18h104l-9 36q-3 10-13 10H30q-10 0-13-10z" fill="url(#mbz)"/>
<ellipse cx="60" cy="18" rx="53" ry="7" fill="url(#mbr)"/>
<ellipse cx="60" cy="18" rx="46" ry="4.5" fill="#d9cdb3"/>
<path d="M30 34q30 8 60 0" stroke="#e2bb6a" stroke-width="1.6" fill="none" opacity=".7"/>
<circle cx="60" cy="44" r="7" fill="none" stroke="#e2bb6a" stroke-width="1.5" opacity=".75"/>
<path d="M56 44h8M60 40v8" stroke="#e2bb6a" stroke-width="1.3" opacity=".75"/></svg>`;

const CANDLE = `<svg viewBox="0 0 30 110"><defs><linearGradient id="mcs" x1="0" x2="1"><stop offset="0" stop-color="#7a4d1c"/><stop offset=".5" stop-color="#e2bb6a"/><stop offset="1" stop-color="#7a4d1c"/></linearGradient>
<radialGradient id="mfl" cx=".5" cy=".6"><stop offset="0" stop-color="#fff6c8"/><stop offset=".55" stop-color="#ffc24a"/><stop offset="1" stop-color="#ff7a1a" stop-opacity="0"/></radialGradient></defs>
<ellipse cx="15" cy="14" rx="9" ry="13" fill="url(#mfl)" opacity=".85"/><path d="M15 6q-4 8 0 14q4-6 0-14z" fill="#fff3b0"/>
<rect x="9" y="22" width="12" height="46" rx="2" fill="#c0392b"/><rect x="9" y="22" width="4" height="46" fill="#e2564a"/>
<path d="M4 68h22l-4 6H8z" fill="url(#mcs)"/><rect x="12" y="74" width="6" height="22" fill="url(#mcs)"/><path d="M5 96h20l3 10H2z" fill="url(#mcs)"/></svg>`;

const VASE = `<svg viewBox="0 0 70 110"><path d="M35 70V40M35 52q-12-6-18-18M35 48q12-8 16-22M35 60q-16 0-22-10" stroke="#3d6b2a" stroke-width="2" fill="none"/>
<g fill="#f5d36b" stroke="#b8860b" stroke-width=".8"><circle cx="17" cy="32" r="7"/><circle cx="51" cy="25" r="7"/><circle cx="35" cy="38" r="7"/><circle cx="13" cy="49" r="6"/></g>
<g fill="#b8860b"><circle cx="17" cy="32" r="2.2"/><circle cx="51" cy="25" r="2.2"/><circle cx="35" cy="38" r="2.2"/><circle cx="13" cy="49" r="2"/></g>
<path d="M24 70h22q4 14-2 28q-2 6-9 6t-9-6q-6-14-2-28z" fill="#2f5d8a"/><path d="M26 72h18" stroke="#c9d9ea" stroke-width="2"/><path d="M28 86q7 4 14 0" stroke="#c9d9ea" stroke-width="1.5" fill="none"/></svg>`;

let cleanupFn: (() => void) | null = null;
export function disposeMemorial(): void { cleanupFn?.(); cleanupFn = null; }

export async function renderMemorial(root: HTMLElement, id: string): Promise<void> {
  disposeMemorial();
  const sn = await snap();
  const s = await store();
  const p = sn.byId.get(id);
  if (!p || p.kind !== "deceased") { root.append(backLink(), h("p", { class: "warn" }, "Không tìm thấy hồ sơ này (có thể đã bị xóa).")); return; }
  // O-9 (tinh thần riêng tư): tiêu đề tab không có tên người.
  document.title = "Tưởng niệm — Lịch và Văn hóa Việt";
  const d = p.death as JsonObj;
  const b = p.birth as JsonObj | null;
  const mem = p.memorial as JsonObj;
  const lunar = d.lunar as JsonObj;
  const t = today();
  const catholic = mem.tray_style === "cong_giao";
  const heading = biaHeading(p);
  const biaName = heading ? ((p.ho_ten as string) ?? "").normalize("NFC").trim() : displayName(p); // #38: đã cắt khoảng trắng
  const incense = !catholic && mem.incense !== false;

  // Dòng ngày
  const ann = M.anniversariesIn(cal(), lunar.year as number | null, lunar.month as number, lunar.day as number, mem.gio_het_nam as number, t, addDays(t, 400))[0];
  let dayLine: string;
  if (ann && daysBetween(t, ann.solar) === 0) dayLine = `${M.labelText(ann.label, ann.n)} · hôm nay ${ann.day}/${ann.month} âm lịch`;
  else if (ann) dayLine = `Ngày giỗ sắp tới: ${ann.day}/${ann.month} âm lịch, ${String(ann.solar.d).padStart(2, "0")}/${String(ann.solar.m).padStart(2, "0")}/${ann.solar.y}`;
  else dayLine = `Ngày giỗ: ${lunarText({ year: null, month: lunar.month as number, day: lunar.day as number, leap: false })}`;

  // Năm trên bia
  const by = b ? (b.lunar_year as number | null) : null;
  const dy = lunar.year as number | null;
  const yr = (y: number) => h("span", { class: "nw" }, `${M.canChiYear(y)} ${y}`);
  const years: (Node | string)[] = by !== null && dy !== null ? [yr(by), " — ", yr(dy)] : dy !== null ? ["Mất năm ", yr(dy)] : [`Giỗ ngày ${lunar.day as number}/${lunar.month as number} âm lịch`];
  const age = tuoiText(d.tuoi as JsonObj);

  const frame = h("div", { class: "mem-photo" });
  if (typeof p.photo_id === "string") {
    const c = h("canvas", { width: "132", height: "168", "aria-label": `Ảnh ${displayName(p)}`, role: "img" }) as HTMLCanvasElement;
    frame.append(c);
    s.photoBytes(p.photo_id).then((bytes) => drawPhoto(c, bytes)).catch(() => { c.remove(); frame.append(h("span", { class: "mem-init" }, initial(p))); });
  } else frame.append(h("span", { class: "mem-init", "aria-hidden": "true" }, initial(p)));

  const sticks = incense ? h("div", { class: "incense", "aria-hidden": "true" },
    ...[0, 1, 2].map((i) => h("span", { class: `stick s${i}` }, h("i", { class: "smoke" }), h("i", { class: "smoke b" }), h("b", { class: "tip" })))) : null;

  const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  const toggle = h("button", { type: "button", class: "btn2 mem-toggle", id: "mem-toggle", hidden: !incense || reduce });
  const scene = h("div", { class: "mem-scene lit", id: "mem-scene" },
    h("p", { class: "mem-day" }, dayLine),
    h("div", { class: "mem-tools" }, toggle),
    frame,
    h("div", { class: "bia", role: "group", "aria-label": "Bia tưởng niệm" },
      heading ? h("p", { class: "bia-top" }, heading) : null,
      // Dòng trên tên = Quan hệ bằng từ thường ngày ("Ông nội", "Mẹ") — khi có thì không lặp cách gọi trước tên.
      h("h1", { class: `bia-name${biaName.length > 28 ? " long" : ""}`, tabindex: "-1" }, biaName),
      p.ten_hieu ? h("p", { class: "bia-hieu" }, p.ten_hieu as string) : null,
      h("p", { class: "bia-years" }, ...years),
      age ? h("p", { class: "bia-age" }, age) : null),
    h("div", { class: "altar", "aria-hidden": "true" },
      // YC-2026-10 (08 §16.1): không vẽ mâm quả, không lọ hoa — nến · bát hương ba nén · nến. Ảnh mâm lễ (của gia đình,
      // hay ảnh Clevix đã duyệt) nằm dưới hàng bàn thờ. Công giáo: nến · hoa · nến, không hương.
      h("div", { class: `altar-row${catholic ? " catholic" : ""}` },
        svg(CANDLE, "candle"),
        catholic ? svg(VASE, "vase") : h("div", { class: "bowl-wrap" }, sticks, svg(BOWL, "bowl")),
        svg(CANDLE, "candle")),
      h("div", { class: "altar-table" })),
    ((mem.loi_tuong_niem as string) ?? "").trim() ? h("blockquote", { class: "mem-loi" }, (mem.loi_tuong_niem as string).normalize("NFC").trim()) : null);

  let remaining = SMOKE_MS, startedAt = 0, timer = 0;
  const stopAll = () => { window.clearTimeout(timer); timer = 0; scene.classList.remove("lit"); toggle.textContent = "Thắp lại"; };
  const run = () => {
    if (reduce || !incense || remaining <= 0) { stopAll(); return; }
    if (document.visibilityState !== "visible") { scene.classList.add("paused"); return; }
    scene.classList.add("lit"); scene.classList.remove("paused");
    toggle.textContent = "Dừng khói";
    startedAt = performance.now();
    window.clearTimeout(timer);
    timer = window.setTimeout(() => { remaining = 0; stopAll(); }, remaining);
  };
  const onVis = () => {
    if (!scene.classList.contains("lit")) return;
    if (document.visibilityState !== "visible") {
      window.clearTimeout(timer); timer = 0;
      remaining -= performance.now() - startedAt;
      scene.classList.add("paused");
    } else run();
  };
  toggle.addEventListener("click", () => {
    if (scene.classList.contains("lit")) { remaining = 0; stopAll(); }
    else { remaining = SMOKE_MS; run(); }
  });
  document.addEventListener("visibilitychange", onVis);
  cleanupFn = () => { window.clearTimeout(timer); document.removeEventListener("visibilitychange", onVis); scene.classList.remove("lit"); };
  if (reduce || !incense) scene.classList.remove("lit");

  root.append(h("div", { class: "mem-page" },
    backLink("‹ Về hồ sơ", `#/nguoi-than/nguoi/${id}`), scene));
  run();
  (scene.querySelector(".bia-name") as HTMLElement).focus({ preventScroll: true });
}

function initial(p: JsonObj): string {
  const last = (p.ho_ten as string).trim().split(/\s+/).pop() ?? "";
  return [...last][0]?.toUpperCase() ?? "";
}
