// Vỏ ứng dụng: điều hướng, cử chỉ, phím tắt, tùy chọn, service worker.
import { IS_MOCK, MAX_YEAR, MIN_YEAR, dayInfo } from "./engine";
import { $, clear, h, svg } from "./dom";
import { addDays, isoDate, parseIso, sameDay, todayLocal } from "./format";
import { I } from "./icons";
import { applyPrefs, getPrefs, setPref, type TextSize, type Theme } from "./prefs";
import { renderAbout, renderRuleSources } from "./ui/about";
import { renderConvert } from "./ui/convert";
import { renderGoodDays } from "./ui/gooddays";
import { renderMonth } from "./ui/month";
import { renderDayNav, renderSheet } from "./ui/sheet";
import { APP_VERSION } from "./gen/content";
import { applyUpdate, attachRegistration, autoCheck, onUpdateResult } from "./update";
import type { SolarDate } from "./types";

type Route = "ngay" | "chon-ngay" | "doi-ngay" | "gioi-thieu" | "nguon-quy-tac";
/** Trang con không có tab riêng → tab được đánh dấu. */
const PARENT: Partial<Record<Route, Route>> = { "nguon-quy-tac": "gioi-thieu" };

const state = {
  route: "ngay" as Route,
  date: todayLocal(),
  today: todayLocal(),
  monthView: { y: 0, m: 0 },
  detailsOpen: false,
  gdState: null as ReturnType<ReturnType<typeof renderGoodDays>> | null,
  gdGetter: null as null | (() => ReturnType<ReturnType<typeof renderGoodDays>>),
};

declare global {
  interface Window { __lvhv?: { renderMs: number[]; mock: boolean } }
}
const metrics = { renderMs: [] as number[], mock: IS_MOCK };
window.__lvhv = metrics;

function clamp(d: SolarDate): SolarDate {
  if (d.y < MIN_YEAR) return { y: MIN_YEAR, m: 1, d: 1 };
  if (d.y > MAX_YEAR) return { y: MAX_YEAR, m: 12, d: 31 };
  return d;
}

// ---------- Khung ----------
const app = $("#app");
clear(app);

const TABS: Array<[Route, string, string]> = [
  ["ngay", "Tờ lịch", I.sheet],
  ["chon-ngay", "Chọn ngày tốt", I.star],
  ["doi-ngay", "Đổi ngày", I.swap],
  ["gioi-thieu", "Giới thiệu", I.info],
];

const tabbar = h("nav", { class: "tabbar", "aria-label": "Mục chính" });
TABS.forEach(([r, label, ico]) => tabbar.append(h("a", { href: `#/${r}`, class: "tab", "data-r": r }, svg(ico), h("span", null, label))));

const sizeGroup = h("div", { class: "sizes", role: "group", "aria-label": "Cỡ chữ" });
([["1", "A", "Cỡ chữ thường"], ["2", "A+", "Cỡ chữ lớn"], ["3", "A++", "Cỡ chữ rất lớn"]] as Array<[TextSize, string, string]>).forEach(([v, t, l]) => {
  sizeGroup.append(h("button", { type: "button", "data-v": v, "aria-label": l, "aria-pressed": "false", onclick: () => { setPref("size", v); syncPrefsUi(); } }, t));
});
const themeBtn = h("button", { type: "button", class: "iconbtn sm themebtn", onclick: () => cycleTheme() });

const topbar = h("header", { class: "topbar" },
  h("a", { class: "brand", href: "#/ngay", "aria-label": "Lịch và Văn hóa Việt, về tờ lịch hôm nay" },
    h("img", { src: "icons/icon.svg", alt: "", width: "28", height: "28" }),
    h("span", { class: "brand-name" }, h("span", { class: "bn-long" }, "Lịch và Văn hóa Việt"), h("span", { class: "bn-short", "aria-hidden": "true" }, "Lịch Việt"))),
  h("div", { class: "tools" }, sizeGroup, themeBtn));

const main = h("main", { id: "main", class: "main", tabindex: "-1" });
const footer = h("footer", { class: "foot" },
  h("span", { class: "foot-brand" },
    h("img", { src: "icons/clevix-logo-72.png", srcset: "icons/clevix-logo-72.png 1x, icons/clevix-logo-144.png 2x, icons/clevix-logo-216.png 3x", alt: "Logo Clevix", width: "67", height: "44", class: "foot-logo" }),
    h("span", { class: "foot-ver" },
      h("span", { class: "foot-copy" }, h("span", { class: "foot-c", "aria-label": "Bản quyền" }, "©"), "\u00a02026 Clevix Lab"),
      h("span", { class: "foot-v" }, `Phiên bản ${APP_VERSION}`))),
  IS_MOCK ? h("span", { class: "mock", title: "Dữ liệu lịch đang là bản giả lập để thử giao diện" }, "MOCK — dữ liệu giả lập") : null);

const dialog = h("dialog", { class: "mdialog", "aria-label": "Chọn ngày trên bảng tháng" });

/** Thanh báo cập nhật DUY NHẤT (ở đầu trang, không che nội dung, không chặn thao tác). */
const updateBar = h("div", { class: "update-bar", role: "status", "aria-live": "polite", hidden: true });

app.append(updateBar, topbar, tabbar, main, footer, dialog);

function cycleTheme(): void {
  const order: Theme[] = ["auto", "light", "dark"];
  setPref("theme", order[(order.indexOf(getPrefs().theme) + 1) % 3]);
  syncPrefsUi();
}

function syncPrefsUi(): void {
  const p = getPrefs();
  sizeGroup.querySelectorAll("button").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.v === p.size)));
  clear(themeBtn);
  const [ico, label] = p.theme === "auto" ? [I.auto, "Giao diện: theo máy"] : p.theme === "light" ? [I.sun, "Giao diện: sáng"] : [I.moon, "Giao diện: tối"];
  themeBtn.append(svg(ico));
  themeBtn.setAttribute("aria-label", `${label}. Bấm để đổi`);
  themeBtn.title = label;
}

// ---------- Định tuyến ----------
function parseHash(): void {
  const m = /^#\/([a-z-]+)(?:\/(\d{4}-\d{2}-\d{2}))?/.exec(location.hash);
  const r = (m?.[1] ?? "ngay") as Route;
  state.route = TABS.some((t) => t[0] === r) || r in PARENT ? r : "ngay";
  if (state.route === "ngay") {
    const d = m?.[2] ? parseIso(m[2]) : null;
    state.date = d ? clamp(d) : state.date;
  }
}

function setDate(d: SolarDate, push = false): void {
  state.date = clamp(d);
  const url = `#/ngay/${isoDate(state.date)}`;
  if (push) history.pushState(null, "", url); else history.replaceState(null, "", url);
  if (state.route !== "ngay") { state.route = "ngay"; render(); return; }
  renderDay();
}

const handlers = {
  prev: () => setDate(addDays(state.date, -1)),
  next: () => setDate(addDays(state.date, 1)),
  today: () => setDate(todayLocal()),
  openMonth: () => openMonthDialog(),
};

let sheetHost: HTMLElement | null = null;
let sideHost: HTMLElement | null = null;
let navHost: HTMLElement | null = null;

function render(): void {
  if (state.gdGetter) { state.gdState = state.gdGetter(); state.gdGetter = null; }
  tabbar.querySelectorAll<HTMLAnchorElement>(".tab").forEach((a) => {
    if (a.dataset.r === (PARENT[state.route] ?? state.route)) a.setAttribute("aria-current", "page"); else a.removeAttribute("aria-current");
  });
  document.body.dataset.route = state.route;
  clear(main);
  sheetHost = sideHost = navHost = null;
  const openDay = (d: SolarDate) => { setDate(d, true); window.scrollTo(0, 0); main.focus({ preventScroll: true }); };
  if (state.route === "ngay") {
    sheetHost = h("div", { class: "sheet-host" });
    navHost = h("div", { class: "nav-host" });
    sideHost = h("aside", { class: "side", "aria-label": "Bảng tháng" });
    main.append(h("div", { class: "daylayout" }, h("div", { class: "daycol" }, navHost, sheetHost), sideHost));
    renderDay();
    document.title = "Lịch và Văn hóa Việt";
  } else {
    const page = h("div", { class: "page" });
    main.append(page);
    if (state.route === "chon-ngay") state.gdGetter = renderGoodDays(page, openDay, state.gdState);
    else if (state.route === "doi-ngay") renderConvert(page, openDay, state.date);
    else if (state.route === "nguon-quy-tac") renderRuleSources(page);
    else renderAbout(page);
    const title = state.route === "nguon-quy-tac" ? "Nguồn quy tắc" : TABS.find((t) => t[0] === state.route)![1];
    document.title = `${title} — Lịch và Văn hóa Việt`;
    window.scrollTo(0, 0);
  }
}

const wide = window.matchMedia("(min-width: 960px)");

function renderDay(): void {
  if (!sheetHost || !navHost || !sideHost) return;
  const t0 = performance.now();
  const keepScroll = window.scrollY;
  const detailsEl = sheetHost.querySelector("details");
  if (detailsEl) state.detailsOpen = detailsEl.open;
  let el: HTMLElement;
  try {
    el = renderSheet(state.date, handlers, state.detailsOpen).el;
  } catch (e) {
    // Ranh giới lỗi (DUYET-MA §6): lõi ném lỗi → báo cho người dùng thay vì để màn hình trống.
    console.error(e);
    el = h("section", { class: "sheet sheet-error", role: "alert" },
      h("p", { class: "warn" }, `Không tính được ngày ${state.date.d}/${state.date.m}/${state.date.y}. Ứng dụng hỗ trợ từ 01/01/${MIN_YEAR} đến 31/12/${MAX_YEAR}.`));
  }
  clear(sheetHost);
  sheetHost.append(el);
  clear(navHost);
  navHost.append(renderDayNav(handlers, sameDay(state.date, todayLocal())));
  if (wide.matches) renderSide();
  else clear(sideHost);
  window.scrollTo(0, keepScroll);
  metrics.renderMs.push(+(performance.now() - t0).toFixed(2));
  if (metrics.renderMs.length > 50) metrics.renderMs.shift();
}

function renderSide(): void {
  if (!sideHost) return;
  if (state.monthView.y !== state.date.y || state.monthView.m !== state.date.m) state.monthView = { y: state.date.y, m: state.date.m };
  const draw = () => {
    clear(sideHost!);
    sideHost!.append(renderMonth({
      view: state.monthView, selected: state.date, idPrefix: "side",
      onPick: (d) => setDate(d, true),
      onView: (v) => { state.monthView = v; draw(); },
    }));
  };
  draw();
}
wide.addEventListener("change", () => { if (state.route === "ngay") renderDay(); });

function openMonthDialog(): void {
  if (wide.matches && sideHost) {
    const first = sideHost.querySelector<HTMLElement>(".mcell.sel") ?? sideHost.querySelector<HTMLElement>("select");
    first?.focus();
    return;
  }
  let view = { y: state.date.y, m: state.date.m };
  const draw = () => {
    clear(dialog);
    dialog.append(
      h("div", { class: "dhead" },
        h("h2", null, "Chọn ngày"),
        h("button", { type: "button", class: "iconbtn sm", "aria-label": "Đóng", onclick: () => dialog.close() }, svg(I.close))),
      renderMonth({
        view, selected: state.date, idPrefix: "dlg",
        onPick: (d) => { dialog.close(); setDate(d, true); },
        onView: (v) => { view = v; draw(); (dialog.querySelector(".mhead select") as HTMLElement | null)?.focus(); },
      }),
      h("div", { class: "dfoot" },
        h("button", { type: "button", class: "todaybtn", onclick: () => { dialog.close(); setDate(todayLocal(), true); } }, "Về hôm nay")));
  };
  draw();
  dialog.showModal();
  dialog.querySelector<HTMLElement>(".mcell.sel")?.focus();
}
dialog.addEventListener("click", (e) => { if (e.target === dialog) dialog.close(); });

// ---------- Cử chỉ và phím ----------
let touch: { x: number; y: number; t: number } | null = null;
main.addEventListener("touchstart", (e) => {
  if (state.route !== "ngay" || e.touches.length !== 1) { touch = null; return; }
  const t = e.touches[0];
  touch = { x: t.clientX, y: t.clientY, t: performance.now() };
}, { passive: true });
main.addEventListener("touchend", (e) => {
  if (!touch) return;
  const t = e.changedTouches[0];
  const dx = t.clientX - touch.x, dy = t.clientY - touch.y, dt = performance.now() - touch.t;
  touch = null;
  if (Math.abs(dx) > 60 && Math.abs(dx) > Math.abs(dy) * 1.6 && dt < 800) {
    const dir = dx < 0 ? "next" : "prev";
    animateTurn(dir);
    handlers[dir]();
  }
}, { passive: true });

const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
function animateTurn(dir: "next" | "prev"): void {
  if (reduceMotion.matches || !sheetHost) return;
  sheetHost.classList.remove("turn-next", "turn-prev");
  void sheetHost.offsetWidth;
  sheetHost.classList.add(dir === "next" ? "turn-next" : "turn-prev");
}

document.addEventListener("keydown", (e) => {
  if (state.route !== "ngay" || dialog.open || e.altKey || e.ctrlKey || e.metaKey) return;
  const tag = (e.target as HTMLElement).tagName;
  if (tag === "INPUT" || tag === "SELECT" || tag === "TEXTAREA") return;
  if ((e.target as HTMLElement).closest?.(".mgrid")) return;
  if (e.key === "ArrowLeft") { e.preventDefault(); animateTurn("prev"); handlers.prev(); }
  else if (e.key === "ArrowRight") { e.preventDefault(); animateTurn("next"); handlers.next(); }
  else if (e.key === "Home" || e.key === "t" || e.key === "T") { e.preventDefault(); handlers.today(); }
});

window.addEventListener("hashchange", () => { parseHash(); render(); });
window.addEventListener("popstate", () => { parseHash(); render(); });

// ---------- Qua nửa đêm ----------
// Không dùng đồng hồ chạy liên tục: chỉ một bộ hẹn giờ duy nhất tới 00:00 khi trang đang
// hiển thị; khi trang ẩn thì hủy, khi hiện lại thì kiểm tra ngày qua visibilitychange.
let midnightTimer = 0;
function armMidnight(): void {
  window.clearTimeout(midnightTimer);
  if (document.visibilityState !== "visible") return;
  const now = new Date();
  const next = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1, 0, 0, 5);
  midnightTimer = window.setTimeout(checkRollover, Math.min(next.getTime() - now.getTime(), 2147483000));
}
function checkRollover(): void {
  const t = todayLocal();
  if (!sameDay(t, state.today)) {
    const wasOnToday = sameDay(state.date, state.today);
    state.today = t;
    if (state.route === "ngay") { if (wasOnToday) setDate(t); else renderDay(); }
  }
  armMidnight();
}
document.addEventListener("visibilitychange", checkRollover);

// ---------- Khởi động ----------
applyPrefs();
syncPrefsUi();
parseHash();
if (!location.hash) history.replaceState(null, "", `#/ngay/${isoDate(state.date)}`);
try { dayInfo(state.date.y, state.date.m, state.date.d); } catch (e) { console.error(e); }
try {
  render();
} catch (e) {
  console.error(e);
  clear(main);
  main.append(h("p", { class: "warn", role: "alert" }, "Có lỗi khi hiển thị trang này. Hãy tải lại trang."));
}
armMidnight();

// ---------- Phiên bản và cập nhật (update.ts) ----------
// "Để sau" chỉ ẩn thanh; lần kiểm tra kế tiếp (tự động, tối đa 6 giờ một lần) sẽ nhắc lại.
onUpdateResult((r) => {
  if (r.status !== "new") return;
  clear(updateBar);
  const now = h("button", { type: "button", class: "ubtn primary", onclick: () => {
    now.setAttribute("disabled", ""); now.textContent = "Đang cập nhật…"; void applyUpdate();
  } }, "Cập nhật ngay");
  const later = h("button", { type: "button", class: "ubtn", onclick: () => { updateBar.hidden = true; } }, "Để sau");
  updateBar.append(
    h("span", { class: "update-msg" }, r.version ? `Đã có phiên bản mới ${r.version}.` : "Đã có bản cập nhật mới của ứng dụng."),
    h("span", { class: "update-acts" }, now, later));
  updateBar.hidden = false;
});

if ("serviceWorker" in navigator && location.protocol !== "file:") {
  window.addEventListener("load", () => {
    navigator.serviceWorker.register("sw.js", { scope: "./" })
      .then((reg) => { attachRegistration(reg); autoCheck(); })
      .catch((e) => console.warn("Không đăng ký được chế độ ngoại tuyến:", e));
  });
  document.addEventListener("visibilitychange", () => { if (document.visibilityState === "visible") autoCheck(); });
}
