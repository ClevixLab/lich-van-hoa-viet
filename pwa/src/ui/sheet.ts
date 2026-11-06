// Tờ lịch — màn hình chính.
import { cadaoFor, dayInfo } from "../engine";
import { h, svg } from "../dom";
import { LUNAR_MONTH_NAME, WEEKDAY, capFirst, hourRange, rating, sameDay, signed, todayLocal } from "../format";
import { I } from "../icons";
import type { DayInfo, SolarDate } from "../types";
import { NGOCHAP_NOTE, RULES_HREF } from "./about";

export interface SheetHandlers {
  prev(): void;
  next(): void;
  today(): void;
  openMonth(): void;
}

const MAX_LIST = 4;

function chipList(items: string[], cls: string, label: string): HTMLElement {
  if (!items.length) return h("p", { class: "muted" }, "Không có");
  const ul = h("ul", { class: `tags ${cls}`, "aria-label": label });
  items.forEach((t) => ul.append(h("li", null, t)));
  return ul;
}

function expandableList(items: string[], id: string, label: string): HTMLElement {
  const wrap = h("div", { class: "xlist" });
  if (!items.length) {
    wrap.append(h("p", { class: "muted" }, "Không có ghi nhận"));
    return wrap;
  }
  const ul = h("ul", { class: "plain", id, "aria-label": label });
  items.forEach((t, i) => ul.append(h("li", { class: i >= MAX_LIST ? "extra" : null, hidden: i >= MAX_LIST }, t)));
  wrap.append(ul);
  if (items.length > MAX_LIST) {
    const more = items.length - MAX_LIST;
    const btn = h("button", { class: "linkbtn", type: "button", "aria-expanded": "false", "aria-controls": id }, `Xem thêm ${more} việc`);
    btn.addEventListener("click", () => {
      const open = btn.getAttribute("aria-expanded") === "true";
      ul.querySelectorAll<HTMLElement>("li.extra").forEach((li) => (li.hidden = open));
      btn.setAttribute("aria-expanded", String(!open));
      btn.textContent = open ? `Xem thêm ${more} việc` : "Thu gọn";
    });
    wrap.append(btn);
  }
  return wrap;
}

export function lunarLine(info: DayInfo): string {
  return `${info.lunar.d} tháng ${info.lunar.m}${info.lunar.leap ? " (nhuận)" : ""} năm ${info.can_chi.year}`;
}

export function renderSheet(date: SolarDate, on: SheetHandlers, detailsOpen: boolean): { el: HTMLElement; info: DayInfo } {
  const info = dayInfo(date.y, date.m, date.d);
  const wd = info.solar.weekday;
  const isToday = sameDay(date, todayLocal());
  const r = rating(info.ngoc_hap.score);
  const cd = cadaoFor(date.y, date.m, date.d);
  const lm = info.lunar;
  const monthName = LUNAR_MONTH_NAME[lm.m] ?? String(lm.m);

  const head = h("header", { class: "sheet-band" },
    h("button", { class: "band-btn", type: "button", onclick: () => on.openMonth(), "aria-label": `Tháng ${date.m} năm ${date.y}. Mở bảng tháng để chọn ngày khác` },
      h("span", { class: "band-month" }, `Tháng ${date.m}`),
      h("span", { class: "band-year" }, String(date.y)),
      svg(I.grid, "ico band-ico")),
  );

  const solar = h("section", { class: "solar", "aria-label": "Ngày dương lịch" },
    h("p", { class: `weekday${wd === 0 ? " sun" : ""}` }, WEEKDAY[wd]),
    h("p", { class: `bignum${wd === 0 ? " sun" : ""}`, "aria-hidden": "true" }, String(date.d)),
    h("h1", { class: "sr-only" }, `${WEEKDAY[wd]}, ngày ${date.d} tháng ${date.m} năm ${date.y}. Âm lịch ${lunarLine(info)}`),
    isToday ? h("p", { class: "today-mark" }, "Hôm nay") : null,
  );

  const lunar = h("section", { class: "lunar", "aria-label": "Âm lịch" },
    h("div", { class: "lunar-day" },
      h("p", { class: "cap" }, "Âm lịch"),
      h("p", { class: `lnum${lm.d === 1 || lm.d === 15 ? " key" : ""}` }, String(lm.d)),
      h("p", { class: "lmonth" }, `tháng ${monthName}${lm.leap ? " (nhuận)" : ""}`),
      h("p", { class: "lmeta" }, lm.month_len === 30 ? "tháng đủ" : "tháng thiếu"),
    ),
    h("dl", { class: "canchi" },
      h("div", null, h("dt", null, "Năm"), h("dd", null, info.can_chi.year)),
      h("div", null, h("dt", null, "Tháng"), h("dd", null, info.can_chi.month)),
      h("div", null, h("dt", null, "Ngày"), h("dd", null, info.can_chi.day)),
      h("div", null, h("dt", null, "Tiết"), h("dd", null, info.tiet_khi)),
    ),
  );

  const labels = h("ul", { class: "labels", "aria-label": "Đặc điểm ngày" },
    h("li", { class: info.than.hoang_dao ? "lb lb-hd" : "lb lb-hk" },
      h("span", { class: "lb-k" }, info.than.hoang_dao ? "Hoàng đạo" : "Hắc đạo"),
      h("span", { class: "lb-v" }, info.than.name)),
    h("li", { class: "lb" }, h("span", { class: "lb-k" }, "Trực"), h("span", { class: "lb-v" }, info.truc)),
    h("li", { class: `lb ${r.cls}`, title: `Điểm chung của ngày: ${signed(info.ngoc_hap.score)}` }, h("span", { class: "lb-k" }, "Đánh giá"), h("span", { class: "lb-v" }, r.label)),
  );

  const hours = h("section", { class: "block", "aria-labelledby": "h-gio" },
    h("h2", { id: "h-gio" }, "Giờ hoàng đạo"),
    (() => {
      const ul = h("ul", { class: "hours" });
      info.gio_hoang_dao.forEach((g) => ul.append(h("li", null, h("span", { class: "hr-chi" }, g.chi), h("span", { class: "hr-t" }, hourRange(g)))));
      return ul;
    })(),
  );

  const doDont = h("section", { class: "block dodont" },
    h("div", { class: "col nen" }, h("h2", { id: "h-nen" }, "Nên làm"), expandableList(info.ngoc_hap.nen, "l-nen", "Việc nên làm")),
    h("div", { class: "col ky" }, h("h2", { id: "h-ky" }, "Kỵ"), expandableList(info.ngoc_hap.ky, "l-ky", "Việc nên tránh")),
  );

  const proverb = cd
    ? h("figure", { class: "cadao", "aria-label": `${capFirst(cd.type)} trong ngày`, "data-id": cd.id },
        h("blockquote", null, ...cd.lines.map((line) => h("span", { class: "cd-line" }, line))),
        h("figcaption", null, capFirst(cd.type)))
    : null;

  const details = h("details", { class: "block details", open: detailsOpen || null },
    h("summary", null, h("span", null, "Chi tiết"), svg(I.down, "ico chev")),
    h("div", { class: "det-body" },
      h("div", { class: "det-row" }, h("h3", null, "Sao tốt"), chipList(info.ngoc_hap.sao_tot, "good", "Sao tốt")),
      h("div", { class: "det-row" }, h("h3", null, "Sao xấu"), chipList(info.ngoc_hap.sao_xau, "bad", "Sao xấu")),
      h("dl", { class: "facts" },
        h("div", null, h("dt", null, "Nhị thập bát tú"), h("dd", null, `Sao ${info.tu}`)),
        h("div", null, h("dt", null, "Nạp âm"), h("dd", null, info.nap_am)),
        h("div", null, h("dt", null, "Giờ Tý đầu ngày"), h("dd", null, info.can_chi.hour_ty)),
        h("div", null, h("dt", null, "Tuổi xung"), h("dd", { class: "xung" }, info.ngoc_hap.xung_tuoi.join(", ") || "Không có")),
        h("div", null, h("dt", null, "Điểm chung"), h("dd", null, signed(info.ngoc_hap.score))),
      ),
      h("p", { class: "srcnote" }, `${NGOCHAP_NOTE}. `, h("a", { href: RULES_HREF, class: "textlink" }, "Nguồn quy tắc")),
    ),
  );

  const el = h("article", { class: `sheet${wd === 0 ? " is-sun" : ""}`, "data-date": `${date.y}-${date.m}-${date.d}` },
    h("div", { class: "rings", "aria-hidden": "true" }, h("i"), h("i")),
    head,
    h("div", { class: "sheet-body" }, solar, lunar, labels, hours, doDont, proverb, details),
  );

  return { el, info };
}

export function renderDayNav(on: SheetHandlers, isToday: boolean): HTMLElement {
  return h("nav", { class: "daynav", "aria-label": "Chuyển ngày" },
    h("button", { class: "iconbtn", type: "button", onclick: () => on.prev(), "aria-label": "Ngày trước", "aria-keyshortcuts": "ArrowLeft" }, svg(I.prev)),
    h("button", { class: "todaybtn", type: "button", onclick: () => on.today(), disabled: isToday || null }, "Hôm nay"),
    h("button", { class: "iconbtn", type: "button", onclick: () => on.next(), "aria-label": "Ngày sau", "aria-keyshortcuts": "ArrowRight" }, svg(I.next)),
  );
}
