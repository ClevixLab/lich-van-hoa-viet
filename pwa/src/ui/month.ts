// Bảng tháng: ngày dương lớn, ngày âm nhỏ; mùng 1 và rằm được làm nổi.
import { MAX_YEAR, MIN_YEAR, solarToLunar } from "../engine";
import { h, svg } from "../dom";
import { WEEKDAY, WEEKDAY_SHORT, daysInMonth, sameDay, todayLocal, weekdayOf } from "../format";
import { I } from "../icons";
import type { SolarDate } from "../types";

export interface MonthOpts {
  /** tháng đang hiển thị */
  view: { y: number; m: number };
  selected: SolarDate;
  onPick(d: SolarDate): void;
  onView(v: { y: number; m: number }): void;
  idPrefix: string;
  compact?: boolean;
}

export function shiftMonth(v: { y: number; m: number }, n: number): { y: number; m: number } {
  const t = v.y * 12 + (v.m - 1) + n;
  const y = Math.floor(t / 12);
  if (y < MIN_YEAR) return { y: MIN_YEAR, m: 1 };
  if (y > MAX_YEAR) return { y: MAX_YEAR, m: 12 };
  return { y, m: (t % 12) + 1 };
}

export function renderMonth(o: MonthOpts): HTMLElement {
  const { y, m } = o.view;
  const today = todayLocal();
  const first = weekdayOf({ y, m, d: 1 });
  // Lưới bắt đầu từ Thứ Hai như lịch treo tường Việt Nam.
  const lead = (first + 6) % 7;
  const n = daysInMonth(y, m);

  const monthSel = h("select", { id: `${o.idPrefix}-m`, "aria-label": "Tháng" });
  for (let i = 1; i <= 12; i++) monthSel.append(h("option", { value: String(i), selected: i === m || null }, `Tháng ${i}`));
  const yearIn = h("input", { id: `${o.idPrefix}-y`, type: "number", inputmode: "numeric", min: String(MIN_YEAR), max: String(MAX_YEAR), value: String(y), "aria-label": "Năm" });
  const yMsg = h("p", { class: "warn mwarn", role: "alert", hidden: true, id: `${o.idPrefix}-ymsg` });
  const goView = () => {
    const raw = yearIn.value.trim();
    const yy = Number(raw);
    if (raw === "" || !Number.isInteger(yy) || yy < MIN_YEAR || yy > MAX_YEAR) {
      yearIn.setAttribute("aria-invalid", "true");
      yearIn.setAttribute("aria-describedby", yMsg.id);
      yMsg.textContent = `Ứng dụng hỗ trợ từ năm ${MIN_YEAR} đến năm ${MAX_YEAR}.`;
      yMsg.hidden = false;
      return;
    }
    yearIn.removeAttribute("aria-invalid");
    yMsg.hidden = true;
    o.onView({ y: yy, m: Number(monthSel.value) });
  };
  monthSel.addEventListener("change", goView);
  yearIn.addEventListener("change", goView);

  const head = h("div", { class: "mhead" },
    h("button", { class: "iconbtn sm", type: "button", "aria-label": "Tháng trước", onclick: () => o.onView(shiftMonth(o.view, -1)), disabled: (y === MIN_YEAR && m === 1) || null }, svg(I.prev)),
    h("div", { class: "mpick" }, monthSel, yearIn),
    h("button", { class: "iconbtn sm", type: "button", "aria-label": "Tháng sau", onclick: () => o.onView(shiftMonth(o.view, 1)), disabled: (y === MAX_YEAR && m === 12) || null }, svg(I.next)),
  );

  const grid = h("div", { class: "mgrid", role: "grid", "aria-label": `Tháng ${m} năm ${y}` });
  const hdr = h("div", { class: "mrow mwd", role: "row" });
  [1, 2, 3, 4, 5, 6, 0].forEach((w) => hdr.append(h("span", { role: "columnheader", class: w === 0 ? "sun" : null, "aria-label": WEEKDAY[w] }, WEEKDAY_SHORT[w])));
  grid.append(hdr);

  let row = h("div", { class: "mrow", role: "row" });
  for (let i = 0; i < lead; i++) row.append(h("span", { class: "mcell empty", role: "gridcell" }));
  for (let d = 1; d <= n; d++) {
    const date = { y, m, d };
    const w = (lead + d - 1) % 7;
    const l = solarToLunar(y, m, d);
    const key = l.d === 1 || l.d === 15;
    const cls = ["mcell", w === 6 ? "sun" : "", key ? "key" : "", sameDay(date, today) ? "today" : "", sameDay(date, o.selected) ? "sel" : ""].filter(Boolean).join(" ");
    const lunarTxt = l.d === 1 ? `${l.d}/${l.m}${l.leap ? "n" : ""}` : String(l.d);
    const label = `${WEEKDAY[(w + 1) % 7]} ${d} tháng ${m}, âm lịch ${l.d} tháng ${l.m}${l.leap ? " nhuận" : ""}${sameDay(date, today) ? ", hôm nay" : ""}`;
    const cell = h("button", { class: cls, type: "button", role: "gridcell", "aria-label": label, "aria-current": sameDay(date, o.selected) ? "date" : null, "data-d": String(d), onclick: () => o.onPick(date) },
      h("span", { class: "sd" }, String(d)),
      h("span", { class: "ld" }, lunarTxt));
    row.append(cell);
    if (w === 6) { grid.append(row); row = h("div", { class: "mrow", role: "row" }); }
  }
  if (row.childNodes.length) {
    while (row.childNodes.length < 7) row.append(h("span", { class: "mcell empty", role: "gridcell" }));
    grid.append(row);
  }

  // Điều hướng bằng phím mũi tên trong lưới.
  grid.addEventListener("keydown", (e) => {
    const t = e.target as HTMLElement;
    if (!t.dataset.d) return;
    const step = { ArrowLeft: -1, ArrowRight: 1, ArrowUp: -7, ArrowDown: 7 }[e.key];
    if (!step) return;
    e.preventDefault();
    e.stopPropagation();
    const nd = Number(t.dataset.d) + step;
    const nx = grid.querySelector<HTMLElement>(`[data-d="${nd}"]`);
    if (nx) nx.focus();
  });

  return h("div", { class: `month${o.compact ? " compact" : ""}` }, head, yMsg, grid,
    h("p", { class: "mlegend" }, h("span", { class: "dot key" }), "Mùng 1 và ngày rằm âm lịch"));
}
