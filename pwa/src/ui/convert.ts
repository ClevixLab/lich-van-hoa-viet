// Đổi ngày dương ↔ âm.
import { MAX_YEAR, MIN_YEAR, dayInfo, lunarToSolar, solarToLunar } from "../engine";
import { clear, h } from "../dom";
import { LUNAR_MONTH_NAME, WEEKDAY, dmy, isoDate, parseIso, todayLocal, weekdayOf } from "../format";
import type { SolarDate } from "../types";

export function renderConvert(root: HTMLElement, openDay: (d: SolarDate) => void, current: SolarDate): void {
  const outA = h("div", { class: "conv-out", "aria-live": "polite" });
  const outB = h("div", { class: "conv-out", "aria-live": "polite" });

  // Dương → âm
  const sIn = h("input", { type: "date", id: "cv-solar", value: isoDate(current), min: `${MIN_YEAR}-01-01`, max: `${MAX_YEAR}-12-31` });
  const doA = () => {
    clear(outA);
    const s = parseIso(sIn.value);
    if (!s || s.y < MIN_YEAR || s.y > MAX_YEAR) {
      outA.append(h("p", { class: "warn" }, `Hãy nhập một ngày dương lịch từ năm ${MIN_YEAR} đến năm ${MAX_YEAR}.`));
      return;
    }
    const l = solarToLunar(s.y, s.m, s.d);
    const info = dayInfo(s.y, s.m, s.d);
    outA.append(
      h("p", { class: "conv-big" }, `${l.d} tháng ${LUNAR_MONTH_NAME[l.m]}${l.leap ? " (nhuận)" : ""}`),
      h("p", { class: "conv-sub" }, `Năm ${info.can_chi.year} · ngày ${info.can_chi.day} · tháng ${info.can_chi.month}`),
      h("button", { type: "button", class: "linkbtn", onclick: () => openDay(s) }, `Mở tờ lịch ${WEEKDAY[weekdayOf(s)]}, ${dmy(s)}`),
    );
  };
  sIn.addEventListener("change", doA);
  sIn.addEventListener("input", doA);

  // Âm → dương
  const l0 = solarToLunar(current.y, current.m, current.d);
  const lyIn = h("input", { type: "number", id: "cv-ly", inputmode: "numeric", min: String(MIN_YEAR), max: String(MAX_YEAR), value: String(l0.y) });
  const lmIn = h("select", { id: "cv-lm" });
  for (let i = 1; i <= 12; i++) lmIn.append(h("option", { value: String(i), selected: i === l0.m || null }, `Tháng ${i} (${LUNAR_MONTH_NAME[i]})`));
  const ldIn = h("select", { id: "cv-ld" });
  for (let i = 1; i <= 30; i++) ldIn.append(h("option", { value: String(i), selected: i === l0.d || null }, i <= 10 ? `Mùng ${i}` : `Ngày ${i}`));
  const leapIn = h("input", { type: "checkbox", id: "cv-leap", checked: l0.leap || null });
  const doB = () => {
    clear(outB);
    const ly = Math.round(Number(lyIn.value));
    if (!Number.isFinite(ly) || ly < MIN_YEAR || ly > MAX_YEAR) {
      outB.append(h("p", { class: "warn" }, `Hãy nhập năm âm lịch từ ${MIN_YEAR} đến ${MAX_YEAR}.`));
      return;
    }
    const lm = Number(lmIn.value), ld = Number(ldIn.value), leap = leapIn.checked;
    const s = lunarToSolar(ly, lm, ld, leap);
    if (!s) {
      const plain = leap ? lunarToSolar(ly, lm, ld, false) : null;
      outB.append(h("p", { class: "warn" }, leap
        ? `Năm ${ly} không có tháng ${lm} nhuận${plain ? "" : ", hoặc tháng này không có ngày " + ld}. Hãy bỏ chọn "Tháng nhuận" hoặc chọn tháng khác.`
        : `Tháng ${lm} âm lịch năm ${ly} không có ngày ${ld} (tháng thiếu chỉ có 29 ngày).`));
      return;
    }
    outB.append(
      h("p", { class: "conv-big" }, `${WEEKDAY[weekdayOf(s)]}, ${dmy(s)}`),
      h("button", { type: "button", class: "linkbtn", onclick: () => openDay(s) }, "Mở tờ lịch ngày này"),
    );
  };
  [lyIn, lmIn, ldIn, leapIn].forEach((x) => x.addEventListener("change", doB));
  lyIn.addEventListener("input", doB);

  const today = todayLocal();
  clear(root);
  root.append(
    h("header", { class: "pagehead" }, h("h1", null, "Đổi ngày"), h("p", null, `Đổi qua lại giữa dương lịch và âm lịch, từ năm ${MIN_YEAR} đến năm ${MAX_YEAR}.`)),
    h("section", { class: "panel conv", "aria-labelledby": "h-cv1" },
      h("h2", { id: "h-cv1" }, "Dương lịch sang âm lịch"),
      h("label", { class: "field", for: "cv-solar" }, h("span", null, "Ngày dương lịch"), sIn),
      outA),
    h("section", { class: "panel conv", "aria-labelledby": "h-cv2" },
      h("h2", { id: "h-cv2" }, "Âm lịch sang dương lịch"),
      h("div", { class: "fields3" },
        h("label", { class: "field", for: "cv-ld" }, h("span", null, "Ngày"), ldIn),
        h("label", { class: "field", for: "cv-lm" }, h("span", null, "Tháng"), lmIn),
        h("label", { class: "field", for: "cv-ly" }, h("span", null, "Năm"), lyIn)),
      h("label", { class: "check", for: "cv-leap" }, leapIn, h("span", null, "Tháng nhuận")),
      outB),
    h("p", { class: "fine" }, `Hôm nay theo máy của bạn: ${dmy(today)}.`),
  );
  doA();
  doB();
}
