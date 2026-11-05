// Chọn ngày tốt.
import {
  BIRTH_MIN_YEAR, MAX_SPAN_DAYS, MAX_YEAR, MIN_YEAR, activities, dayInfo, findGoodDays, isCoreError, lunarYearCanChi,
  lunarYearOfBirth, solarToLunar,
} from "../engine";
import { clear, h } from "../dom";
import { WEEKDAY, addDays, daysInMonth, dmy, isoDate, jdn, parseIso, signed, todayLocal, weekdayOf } from "../format";
import type { SolarDate } from "../types";

type RangeKey = "week" | "month" | "3m" | "custom";

/**
 * Tuổi (C-2): Can Chi tuổi theo NĂM ÂM LỊCH của ngày sinh. Người dùng nhập ngày sinh dương lịch
 * (ngày/tháng/năm) → lõi đổi ra năm âm (`lunarYearOfBirth`); hoặc chọn "Tôi chỉ biết năm âm lịch /
 * Can Chi" rồi nhập thẳng năm âm. Chỉ giữ trong bộ nhớ khi đang mở trang, không ghi xuống máy (S4).
 */
interface State {
  act: string | null;
  range: RangeKey;
  from: string;
  to: string;
  /** "date": ngày sinh dương lịch; "lunar": năm âm lịch (biết Can Chi). */
  birthMode: "date" | "lunar";
  bd: string;
  bm: string;
  by: string;
  /** năm âm lịch, khi birthMode === "lunar" */
  bly: string;
}

const MAX_SPAN = MAX_SPAN_DAYS;

/** Năm âm lịch tuổi người xem, hoặc thông báo lỗi, hoặc null nếu chưa nhập gì. */
function birthLunarYear(st: State, today: SolarDate): { ly: number; note: string } | { err: string } | null {
  if (st.birthMode === "lunar") {
    if (!st.bly.trim()) return null;
    const ly = Number(st.bly);
    if (!Number.isInteger(ly) || ly < BIRTH_MIN_YEAR - 1 || ly > today.y) return { err: `Năm âm lịch cần từ ${BIRTH_MIN_YEAR - 1} đến ${today.y}.` };
    return { ly, note: `năm âm lịch ${ly}` };
  }
  if (!st.bd.trim() && !st.bm.trim() && !st.by.trim()) return null;
  const d = Number(st.bd), m = Number(st.bm), y = Number(st.by);
  if (!st.bd.trim() || !st.bm.trim() || !st.by.trim()) return { err: "Hãy nhập đủ ngày, tháng và năm sinh (dương lịch)." };
  if (!Number.isInteger(y) || y < BIRTH_MIN_YEAR || y > today.y) return { err: `Năm sinh cần từ ${BIRTH_MIN_YEAR} đến ${today.y}.` };
  if (jdn(y, m, d) > jdn(today.y, today.m, today.d)) return { err: "Ngày sinh không thể sau hôm nay." };
  try {
    const ly = lunarYearOfBirth(y, m, d);
    const pre = ly !== y ? " — sinh trước Tết nên thuộc năm âm trước" : "";
    return { ly, note: `năm âm lịch ${ly}${pre}` };
  } catch (e) {
    if (isCoreError(e) && e.code === "InvalidDate") return { err: `Ngày ${d}/${m}/${y} không tồn tại.` };
    if (isCoreError(e)) return { err: `Năm sinh cần từ ${BIRTH_MIN_YEAR} đến ${today.y}.` };
    throw e;
  }
}

function rangeFor(k: RangeKey, s: State): [SolarDate, SolarDate] | string {
  const t = todayLocal();
  if (k === "week") {
    const wd = weekdayOf(t); // 0 = CN
    return [t, addDays(t, wd === 0 ? 0 : 7 - wd)];
  }
  if (k === "month") return [t, { y: t.y, m: t.m, d: daysInMonth(t.y, t.m) }];
  if (k === "3m") return [t, addDays(t, 89)];
  const a = parseIso(s.from), b = parseIso(s.to);
  if (!a || !b) return "Hãy chọn đủ ngày bắt đầu và ngày kết thúc.";
  if (a.y < MIN_YEAR || b.y > MAX_YEAR) return `Lịch hỗ trợ từ năm ${MIN_YEAR} đến năm ${MAX_YEAR}.`;
  const span = jdn(b.y, b.m, b.d) - jdn(a.y, a.m, a.d);
  if (span < 0) return "Ngày kết thúc cần sau ngày bắt đầu.";
  if (span > MAX_SPAN) return "Khoảng tìm tối đa là một năm. Hãy thu hẹp lại.";
  return [a, b];
}

export function renderGoodDays(root: HTMLElement, openDay: (d: SolarDate) => void, saved: State | null): () => State {
  const acts = activities();
  const t = todayLocal();
  const st: State = saved ?? { act: null, range: "month", from: isoDate(t), to: isoDate(addDays(t, 30)), birthMode: "date", bd: "", bm: "", by: "", bly: "" };

  const results = h("section", { class: "results", "aria-live": "polite", "aria-labelledby": "h-kq" });
  const actWrap = h("div", { class: "acts", role: "radiogroup", "aria-labelledby": "h-act" });
  acts.forEach((a) => {
    const b = h("button", { type: "button", class: "actbtn", role: "radio", "aria-checked": String(st.act === a.id), "data-id": a.id }, a.name);
    b.addEventListener("click", () => {
      st.act = a.id;
      actWrap.querySelectorAll("button").forEach((x) => x.setAttribute("aria-checked", String(x.dataset.id === a.id)));
      run();
    });
    actWrap.append(b);
  });

  const ranges: Array<[RangeKey, string]> = [["week", "Tuần này"], ["month", "Tháng này"], ["3m", "3 tháng tới"], ["custom", "Tự chọn"]];
  const rangeWrap = h("div", { class: "seg", role: "radiogroup", "aria-labelledby": "h-range" });
  const fromIn = h("input", { type: "date", id: "gd-from", value: st.from, min: `${MIN_YEAR}-01-01`, max: `${MAX_YEAR}-12-31` });
  const toIn = h("input", { type: "date", id: "gd-to", value: st.to, min: `${MIN_YEAR}-01-01`, max: `${MAX_YEAR}-12-31` });
  const custom = h("div", { class: "custom", hidden: st.range !== "custom" || null },
    h("label", { for: "gd-from" }, h("span", null, "Từ ngày"), fromIn),
    h("label", { for: "gd-to" }, h("span", null, "Đến ngày"), toIn));
  ranges.forEach(([k, label]) => {
    const b = h("button", { type: "button", role: "radio", "aria-checked": String(st.range === k), "data-k": k }, label);
    b.addEventListener("click", () => {
      st.range = k;
      rangeWrap.querySelectorAll("button").forEach((x) => x.setAttribute("aria-checked", String(x.dataset.k === k)));
      custom.hidden = k !== "custom";
      run();
    });
    rangeWrap.append(b);
  });
  fromIn.addEventListener("change", () => { st.from = fromIn.value; run(); });
  toIn.addEventListener("change", () => { st.to = toIn.value; run(); });

  // ---- Tuổi: ngày sinh dương lịch, hoặc năm âm lịch ----
  const num = (id: string, label: string, ph: string, val: string, min: number, max: number) =>
    h("input", { type: "number", id, inputmode: "numeric", min: String(min), max: String(max), placeholder: ph, value: val, "aria-label": label, "aria-describedby": "gd-birth-cc" });
  const bdIn = num("gd-bd", "Ngày sinh", "Ngày", st.bd, 1, 31);
  const bmIn = num("gd-bm", "Tháng sinh", "Tháng", st.bm, 1, 12);
  const byIn = num("gd-by", "Năm sinh", "Năm", st.by, BIRTH_MIN_YEAR, t.y);
  const blyIn = num("gd-bly", "Năm âm lịch", "Ví dụ 1975", st.bly, BIRTH_MIN_YEAR - 1, t.y);
  const lunarOnly = h("input", { type: "checkbox", id: "gd-lunar-only", checked: st.birthMode === "lunar" || null });
  const dateRow = h("span", { class: "birth-row birth-date", hidden: st.birthMode === "lunar" || null }, bdIn, h("span", { "aria-hidden": "true" }, "/"), bmIn, h("span", { "aria-hidden": "true" }, "/"), byIn);
  const lunarRow = h("span", { class: "birth-row", hidden: st.birthMode !== "lunar" || null }, blyIn);
  const birthCc = h("p", { id: "gd-birth-cc", class: "cc", "aria-live": "polite" });
  const showCc = () => {
    const b = birthLunarYear(st, t);
    birthCc.classList.toggle("warn", !!b && "err" in b);
    birthCc.textContent = !b ? "" : "err" in b ? b.err : `Tuổi ${lunarYearCanChi(b.ly)} (${b.note})`;
  };
  const bind = (el: HTMLInputElement, key: "bd" | "bm" | "by" | "bly") => {
    el.addEventListener("input", () => { st[key] = el.value; showCc(); });
    el.addEventListener("change", run);
  };
  bind(bdIn, "bd"); bind(bmIn, "bm"); bind(byIn, "by"); bind(blyIn, "bly");
  lunarOnly.addEventListener("change", () => {
    st.birthMode = lunarOnly.checked ? "lunar" : "date";
    dateRow.hidden = lunarOnly.checked;
    lunarRow.hidden = !lunarOnly.checked;
    showCc();
    run();
  });
  showCc();

  let lastKey = "";
  function run(): void {
    // Bỏ qua khi điều kiện không đổi (ví dụ sự kiện change phát lại lúc ô năm sinh mất tiêu điểm),
    // để không dựng lại danh sách ngay lúc người dùng đang chạm vào một kết quả.
    const key = JSON.stringify(st);
    if (key === lastKey) return;
    lastKey = key;
    clear(results);
    results.append(h("h2", { id: "h-kq" }, "Kết quả"));
    if (!st.act) {
      results.append(h("p", { class: "empty-hint" }, "Chọn một việc ở trên để xem những ngày phù hợp."));
      return;
    }
    const r = rangeFor(st.range, st);
    if (typeof r === "string") {
      results.append(h("p", { class: "warn", role: "alert" }, r));
      return;
    }
    const b = birthLunarYear(st, t);
    if (b && "err" in b) {
      results.append(h("p", { class: "warn", role: "alert" }, b.err));
      return;
    }
    const birth = b ? b.ly : undefined;
    let list;
    try {
      list = findGoodDays(st.act, r[0], r[1], birth);
    } catch (e) {
      if (!isCoreError(e)) throw e;
      results.append(h("p", { class: "warn", role: "alert" }, "Không tính được khoảng ngày này. Hãy chọn khoảng khác."));
      return;
    }
    const actName = acts.find((a) => a.id === st.act)?.name ?? "";
    results.append(h("p", { class: "rsum" }, `${actName}, từ ${dmy(r[0])} đến ${dmy(r[1])}${birth !== undefined ? `, tuổi ${lunarYearCanChi(birth)}` : ""}: ${list.length ? `${list.length} ngày phù hợp` : "chưa có ngày phù hợp"}.`));
    if (list.length) results.append(h("p", { class: "fine" }, `Chỉ liệt kê ngày không phạm sao kỵ chính, không ghi "Kỵ ${actName.toLowerCase()}" trên tờ lịch và có điểm cho việc này từ 0 trở lên. "Điểm cho việc" là quy ước của ứng dụng (cộng sao tốt, trừ sao xấu cho riêng việc này, cộng/trừ theo trực và tuổi), khác điểm chung của ngày trên tờ lịch; chỉ để so sánh các ngày với nhau. Theo Ngọc Hạp Thông Thư; tham khảo.`));
    if (!list.length) {
      results.append(h("p", { class: "empty-hint" }, "Hãy thử mở rộng khoảng thời gian, chẳng hạn 3 tháng tới."));
      return;
    }
    const ol = h("ol", { class: "gdlist" });
    const PAGE = 15;
    let shown = 0;
    const moreBtn = h("button", { type: "button", class: "morebtn" });
    const addItems = () => {
      list.slice(shown, shown + PAGE).forEach((g) => {
        const s = g.solar;
        const l = g.lunar ?? solarToLunar(s.y, s.m, s.d);
        const info = dayInfo(s.y, s.m, s.d);
        const firstHour = g.first_gio_hoang_dao ?? info.gio_hoang_dao[0];
        const wd = g.weekday ?? weekdayOf(s);
        // Lý do của lõi có dạng "+2 Sát Cống", "−1 Hoang Vu", "+1 Trực Khai", "−3 Tuổi … xung …".
        const plus = g.reasons.filter((r) => r.startsWith("+")).map((r) => r.replace(/^\+\d+\s+/, ""));
        const minus = g.reasons.filter((r) => r.startsWith("−")).map((r) => r.replace(/^−\d+\s+/, ""));
        const why = h("details", { class: "gd-why" },
          h("summary", null, `Cách tính điểm cho việc ${actName.toLowerCase()}`),
          h("ul", { class: "plain" }, ...g.reasons.map((r) => h("li", null, r))));
        ol.append(h("li", { class: "gdrow" },
          h("button", { type: "button", class: "gditem", onclick: () => openDay(s), "aria-label": `${WEEKDAY[wd]} ${dmy(s)}, âm lịch ${l.d}/${l.m}${l.leap ? " nhuận" : ""}, điểm cho việc ${actName.toLowerCase()} ${signed(g.score)}. Mở tờ lịch ngày này` },
            h("span", { class: `gd-date${wd === 0 ? " sun" : ""}` }, h("b", null, String(s.d)), h("small", null, `/${s.m}`)),
            h("span", { class: "gd-main" },
              h("span", { class: "gd-l1" }, `${WEEKDAY[wd]}, ${dmy(s)}`),
              h("span", { class: "gd-l2" }, `Âm lịch ${l.d}/${l.m}${l.leap ? " nhuận" : ""}, ngày ${info.can_chi.day}`),
              firstHour ? h("span", { class: "gd-l2" }, `Giờ tốt đầu tiên: ${firstHour.chi} (${firstHour.from}–${firstHour.to} giờ)`) : null,
              plus.length ? h("span", { class: "gd-l3" }, `Tốt: ${plus.join(", ")}`) : null,
              minus.length ? h("span", { class: "gd-l4" }, `Lưu ý: ${minus.join(", ")}`) : null),
            h("span", { class: "gd-score", "aria-hidden": "true", title: `Điểm cho việc ${actName.toLowerCase()}` }, h("b", null, signed(g.score)), h("small", null, "điểm việc"))),
          why));
      });
      shown = Math.min(list.length, shown + PAGE);
      const rest = list.length - shown;
      moreBtn.hidden = rest <= 0;
      moreBtn.textContent = `Xem thêm ${Math.min(rest, PAGE)} ngày`;
    };
    moreBtn.addEventListener("click", () => {
      const firstNew = shown;
      addItems();
      ol.querySelectorAll<HTMLElement>(".gditem")[firstNew]?.focus();
    });
    addItems();
    results.append(ol, moreBtn);
  }

  clear(root);
  root.append(
    h("header", { class: "pagehead" }, h("h1", null, "Chọn ngày tốt"), h("p", null, "Chọn việc định làm và khoảng thời gian. Ngày được xếp từ điểm cao nhất; ngày phạm sao kỵ chính của việc đã bị loại.")),
    h("section", { class: "panel" },
      h("h2", { id: "h-act" }, "Việc định làm"), actWrap,
      h("h2", { id: "h-range" }, "Trong khoảng"), rangeWrap, custom,
      h("fieldset", { class: "birth" },
        h("legend", null, "Ngày sinh dương lịch (không bắt buộc)"),
        dateRow, lunarRow,
        h("label", { class: "check", for: "gd-lunar-only" }, lunarOnly, h("span", null, "Tôi chỉ biết năm âm lịch / Can Chi")),
        birthCc),
      h("p", { class: "fine" }, "Tuổi tính theo năm âm lịch: người sinh trước Tết thuộc năm âm trước (ví dụ sinh 15/01/1976 là tuổi Ất Mão, không phải Bính Thìn). Ngày sinh chỉ dùng để tránh ngày xung tuổi, không được lưu lại.")),
    results);
  run();
  // Trạng thái chỉ giữ trong bộ nhớ khi đang mở ứng dụng, không ghi xuống máy.
  return () => ({ ...st });
}
