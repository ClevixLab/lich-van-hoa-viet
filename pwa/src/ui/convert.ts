// Đổi ngày dương ↔ âm. Mỗi khung có nút "Đổi sang …" (luôn tính lại) và cũng tự cập nhật khi gõ.
// Mọi nhánh lỗi (ngoài phạm vi, ngày không tồn tại, tháng nhuận không có, lỗi lõi) đều hiện một
// thông báo tiếng Việt thay cho kết quả — không bao giờ để trống.
import { MAX_YEAR, MIN_YEAR, dayInfo, isCoreError, lunarToSolar } from "../engine";
import { clear, h } from "../dom";
import { LUNAR_MONTH_NAME, WEEKDAY, daysInMonth, dmy, todayLocal, weekdayOf } from "../format";
import type { SolarDate } from "../types";

/** Thông báo phạm vi dùng chung (smoke test kiểm chuỗi này). */
export const RANGE_MSG = `Ứng dụng hỗ trợ từ 01/01/${MIN_YEAR} đến 31/12/${MAX_YEAR}.`;
/** Ghi chú múi giờ lịch sử (bổ sung A1, docs/03 §16). */
export const TZ_NOTE =
  "Trước năm 1968, âm lịch tính theo múi giờ UTC+8 như lịch thời đó ở miền Bắc; miền Nam dùng UTC+8 đến năm 1975, nên một số tháng 1968–1975 có thể khác lịch miền Nam thời ấy.";

/** Đọc một ô số nguyên; `null` nếu trống hoặc không phải số nguyên. */
function intOf(el: HTMLInputElement): number | null {
  const v = el.value.trim();
  if (!/^\d{1,4}$/.test(v)) return null;
  return Number(v);
}

/** Kiểm ngày dương ba ô; trả ngày hoặc thông báo lỗi. */
export function checkSolar(d: number | null, m: number | null, y: number | null): SolarDate | string {
  if (d === null || m === null || y === null) return "Hãy nhập đủ ngày, tháng và năm (bằng số).";
  if (y < MIN_YEAR || y > MAX_YEAR) return RANGE_MSG;
  if (m < 1 || m > 12) return "Ngày không hợp lệ: tháng phải từ 1 đến 12.";
  const n = daysInMonth(y, m);
  if (d < 1 || d > n) return `Ngày không hợp lệ: tháng ${m} năm ${y} chỉ có ${n} ngày.`;
  return { y, m, d };
}

function numField(id: string, label: string, value: number, cls: string): [HTMLElement, HTMLInputElement] {
  const input = h("input", { type: "text", id, inputmode: "numeric", pattern: "[0-9]*", autocomplete: "off", maxlength: cls === "yy" ? "4" : "2", value: String(value), class: `num ${cls}` }) as HTMLInputElement;
  return [h("label", { class: "field", for: id }, h("span", null, label), input), input];
}

function warn(out: HTMLElement, msg: string): void {
  out.append(h("p", { class: "warn", role: "alert" }, msg));
}

export function renderConvert(root: HTMLElement, openDay: (d: SolarDate) => void, current: SolarDate): void {
  const outA = h("div", { class: "conv-out", "aria-live": "polite" });
  const outB = h("div", { class: "conv-out", "aria-live": "polite" });

  // ---- Dương → âm: ba ô số (không dùng <input type=date>: khó tới năm xa trên điện thoại, và
  // min/max khiến trình duyệt lặng lẽ bỏ giá trị ngoài phạm vi). ----
  const [dL, dIn] = numField("cv-sd", "Ngày", current.d, "dd");
  const [mL, mIn] = numField("cv-sm", "Tháng", current.m, "mm");
  const [yL, yIn] = numField("cv-sy", "Năm", current.y, "yy");
  const doA = () => {
    clear(outA);
    const s = checkSolar(intOf(dIn), intOf(mIn), intOf(yIn));
    if (typeof s === "string") return warn(outA, s);
    let info;
    try {
      info = dayInfo(s.y, s.m, s.d);
    } catch (e) {
      if (!isCoreError(e)) console.error(e);
      return warn(outA, isCoreError(e) && e.code === "OutOfRange" ? RANGE_MSG : "Không tính được ngày này. Hãy kiểm tra lại ngày đã nhập.");
    }
    const l = info.lunar;
    outA.append(
      h("p", { class: "conv-big" }, `Ngày ${l.d} tháng ${l.m}${l.leap ? " (nhuận)" : ""} năm ${info.can_chi.year}`),
      h("p", { class: "conv-sub" }, `Ngày ${info.can_chi.day} · tháng ${info.can_chi.month} · tháng ${LUNAR_MONTH_NAME[l.m]} âm lịch năm ${l.y}`),
      h("button", { type: "button", class: "linkbtn", onclick: () => openDay(s) }, `Mở tờ lịch ${WEEKDAY[weekdayOf(s)]}, ${dmy(s)}`),
    );
  };
  [dIn, mIn, yIn].forEach((x) => x.addEventListener("input", doA));
  const btnA = h("button", { type: "button", class: "primary", id: "cv-go-a", onclick: doA }, "Đổi sang âm lịch");

  // ---- Âm → dương ----
  let l0 = { y: current.y, m: 1, d: 1, leap: false };
  try { l0 = dayInfo(current.y, current.m, current.d).lunar; } catch { /* giữ mặc định */ }
  const [lyL, lyIn] = numField("cv-ly", "Năm", l0.y, "yy");
  const lmIn = h("select", { id: "cv-lm" });
  for (let i = 1; i <= 12; i++) lmIn.append(h("option", { value: String(i), selected: i === l0.m || null }, `Tháng ${i} (${LUNAR_MONTH_NAME[i]})`));
  const ldIn = h("select", { id: "cv-ld" });
  for (let i = 1; i <= 30; i++) ldIn.append(h("option", { value: String(i), selected: i === l0.d || null }, i <= 10 ? `Mùng ${i}` : `Ngày ${i}`));
  const leapIn = h("input", { type: "checkbox", id: "cv-leap", checked: l0.leap || null });
  const doB = () => {
    clear(outB);
    const ly = intOf(lyIn);
    if (ly === null) return warn(outB, "Hãy nhập năm âm lịch (bằng số).");
    // Năm âm MIN_YEAR − 1 vẫn có tháng Chạp nằm trong phạm vi (01/01/1900 = mùng 1 tháng Chạp 1899).
    if (ly < MIN_YEAR - 1 || ly > MAX_YEAR) return warn(outB, RANGE_MSG);
    const lm = Number(lmIn.value), ld = Number(ldIn.value), leap = leapIn.checked;
    let s: SolarDate | null;
    try {
      s = lunarToSolar(ly, lm, ld, leap);
    } catch (e) {
      if (!isCoreError(e)) console.error(e);
      return warn(outB, "Không tính được ngày này. Hãy kiểm tra lại ngày đã nhập.");
    }
    if (!s) {
      const plainMonth = lunarToSolar(ly, lm, 1, false);
      const leapMonth = lunarToSolar(ly, lm, 1, true);
      if (!plainMonth && !leapMonth) return warn(outB, RANGE_MSG);
      if (leap && !leapMonth) return warn(outB, `Năm ${ly} không có tháng ${lm} nhuận. Hãy bỏ chọn "Tháng nhuận" hoặc chọn tháng khác.`);
      return warn(outB, `Tháng ${lm}${leap ? " nhuận" : ""} âm lịch năm ${ly} không có ngày ${ld} (tháng thiếu chỉ có 29 ngày).`);
    }
    const target = s;
    outB.append(
      h("p", { class: "conv-big" }, `${WEEKDAY[weekdayOf(target)]}, ngày ${dmy(target)}`),
      h("button", { type: "button", class: "linkbtn", onclick: () => openDay(target) }, "Mở tờ lịch ngày này"),
    );
  };
  [lmIn, ldIn, leapIn].forEach((x) => x.addEventListener("change", doB));
  lyIn.addEventListener("input", doB);
  const btnB = h("button", { type: "button", class: "primary", id: "cv-go-b", onclick: doB }, "Đổi sang dương lịch");

  const today = todayLocal();
  clear(root);
  root.append(
    h("header", { class: "pagehead" }, h("h1", null, "Đổi ngày"), h("p", null, `Đổi qua lại giữa dương lịch và âm lịch, từ ngày 01/01/${MIN_YEAR} đến ngày 31/12/${MAX_YEAR}.`)),
    h("section", { class: "panel conv", "aria-labelledby": "h-cv1" },
      h("h2", { id: "h-cv1" }, "Dương lịch sang âm lịch"),
      h("div", { class: "fields-dmy" }, dL, mL, yL),
      btnA,
      outA),
    h("section", { class: "panel conv", "aria-labelledby": "h-cv2" },
      h("h2", { id: "h-cv2" }, "Âm lịch sang dương lịch"),
      h("div", { class: "fields3" },
        h("label", { class: "field", for: "cv-ld" }, h("span", null, "Ngày"), ldIn),
        h("label", { class: "field", for: "cv-lm" }, h("span", null, "Tháng"), lmIn),
        lyL),
      h("label", { class: "check", for: "cv-leap" }, leapIn, h("span", null, "Tháng nhuận")),
      btnB,
      outB),
    h("p", { class: "fine tz-note" }, TZ_NOTE),
    h("p", { class: "fine" }, `Hôm nay theo máy của bạn: ${dmy(today)}.`),
  );
  doA();
  doB();
}
