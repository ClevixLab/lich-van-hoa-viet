// Thành phần dùng chung của mô-đun Người thân (docs/v0.2/08 §2).
import { clear, h, svg } from "../dom";
import { MAX_YEAR, MIN_YEAR } from "../engine";
import { M, cal, daysBetween, dm, dmy, itemKind, lunarOf, occLabel, ownerOf, displayName, relText, today, weekdayShort, type JsonObj, type Snap, type Ymd } from "./core";

// ---------------- biểu tượng loại (nét SVG viết tay) ----------------
const S = 'viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"';
export const PI = {
  gio: `<svg ${S}><path d="M8 20V9M12 20V7M16 20V9"/><path d="M8 6.5c-.8-1 .6-1.8 0-3M12 4.5c-.8-1 .6-1.8 0-3M16 6.5c-.8-1 .6-1.8 0-3"/><path d="M5 20h14"/></svg>`,
  tt: `<svg ${S}><path d="M4 15h16l-2 4H6z"/><circle cx="9" cy="12.5" r="2.2"/><circle cx="14.5" cy="12" r="2.6"/></svg>`,
  sn: `<svg ${S}><rect x="4.5" y="12" width="15" height="8" rx="1.5"/><path d="M4.5 15.5c2 1.3 3.5 1.3 5 0s3.5-1.3 5 0 3.5 1.3 5 0M12 12V8.5"/><path d="M12 6.8c-1-.9-.4-2.2 0-3 .4.8 1 2.1 0 3z"/></svg>`,
  ev: `<svg ${S}><path d="M12 3.5l2.5 5.2 5.6.7-4.1 3.9 1 5.6-5-2.7-5 2.7 1-5.6-4.1-3.9 5.6-.7z"/></svg>`,
  people: `<svg ${S}><circle cx="9" cy="8" r="3.2"/><path d="M3.5 19.5c.6-3.3 2.8-5 5.5-5s4.9 1.7 5.5 5"/><circle cx="16.5" cy="9" r="2.5"/><path d="M15.5 14.3c2.6-.2 4.5 1.4 5 4.7"/></svg>`,
};
export const KIND_WORD: Record<string, string> = { gio: "Ngày giỗ", tt: "Tiên thường", sn: "Sinh nhật", ev: "Sự kiện" };

// ---------------- thông báo ngắn ----------------
let toastHost: HTMLElement | null = null;
let toastTimer = 0;
let lastToast: { msg: string; at: number } | null = null;
/** Vùng thông báo của trang vừa dựng. Thông báo vừa phát ngay trước khi chuyển trang (ví dụ "Đã lưu hồ sơ.") được hiện lại ở trang mới. */
export function setToastHost(el: HTMLElement): void {
  toastHost = el;
  if (lastToast && Date.now() - lastToast.at < 3000) { const m = lastToast.msg; toast(m); lastToast = null; }
}
export function toast(msg: string): void {
  lastToast = { msg, at: Date.now() };
  if (!toastHost) return;
  clear(toastHost);
  toastHost.append(h("p", { class: "toast" }, msg));
  window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => { if (toastHost && !toastHost.contains(document.activeElement)) clear(toastHost); }, 6000);
}

// ---------------- hộp xác nhận ----------------
export interface DialogOpts {
  title: string;
  body: (Node | string)[];
  ok: string;
  cancel?: string;
  danger?: boolean;
  focusCancel?: boolean;
  /** Trả false (hoặc chuỗi lỗi) để giữ hộp mở. */
  onOk?: () => Promise<boolean | string | void> | boolean | string | void;
  cls?: string;
}
/** Hộp thoại gốc của trình duyệt; trả true nếu người dùng đồng ý. Tiêu điểm trở về phần tử đã mở hộp. */
let dialogSeq = 0;
export function confirmDialog(o: DialogOpts): Promise<boolean> {
  const back = document.activeElement as HTMLElement | null;
  const err = h("p", { class: "ferr", role: "alert", hidden: true });
  const okBtn = h("button", { type: "button", class: `dbtn ${o.danger ? "danger" : "main"}` }, o.ok);
  const cancelBtn = h("button", { type: "button", class: "dbtn" }, o.cancel ?? "Hủy");
  const tid = `pd-title-${++dialogSeq}`;
  const dlg = h("dialog", { class: `mdialog pdialog ${o.cls ?? ""}`, "aria-labelledby": tid, "aria-busy": "false" },
    h("h2", { id: tid, class: "pd-title" }, o.title),
    h("div", { class: "pd-body" }, ...o.body),
    err,
    h("div", { class: "pd-acts" }, cancelBtn, okBtn));
  document.body.append(dlg);
  return new Promise((resolve) => {
    let result = false, busy = false;
    const finish = () => { dlg.close(); };
    // MA3-15-03: trong lúc việc đang chạy (mã hóa, khôi phục…) phím Esc không đóng được hộp — nếu không, dữ liệu vẫn bị
    // thay mà trang không báo gì.
    dlg.addEventListener("cancel", (e) => { if (busy) e.preventDefault(); });
    dlg.addEventListener("close", () => { dlg.remove(); back?.focus?.(); resolve(result); });
    cancelBtn.addEventListener("click", finish);
    okBtn.addEventListener("click", async () => {
      err.hidden = true;
      okBtn.setAttribute("disabled", "");
      cancelBtn.setAttribute("disabled", "");
      busy = true; dlg.setAttribute("aria-busy", "true");
      let r: boolean | string | void = true;
      try { r = o.onOk ? await o.onOk() : true; } catch (e) { r = (e as Error).message || "Có lỗi xảy ra"; }
      busy = false; dlg.setAttribute("aria-busy", "false");
      okBtn.removeAttribute("disabled");
      cancelBtn.removeAttribute("disabled");
      if (typeof r === "string") { err.textContent = r; err.hidden = false; return; }
      if (r === false) return;
      result = true;
      finish();
    });
    dlg.showModal();
    (o.focusCancel ? cancelBtn : okBtn).focus();
  });
}

// ---------------- nhóm nút chọn (aria-pressed) ----------------
export function pressGroup<T extends string>(label: string, opts: [T, string][], value: T, onPick: (v: T) => void, cls = "pseg"): HTMLElement {
  const g = h("div", { class: cls, role: "group", "aria-label": label });
  for (const [v, t] of opts) {
    g.append(h("button", { type: "button", "data-v": v, "aria-pressed": String(v === value), onclick: () => {
      g.querySelectorAll("button").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.v === v)));
      onPick(v);
    } }, t));
  }
  return g;
}

// ---------------- mục sự kiện (08 §2) ----------------
/** Dòng chính của một lần xuất hiện: nhãn · tên người ("Giỗ đầu · Ông Nguyễn Văn A", "Tốt nghiệp · Lê Văn C"). */
export function itemTitle(sn: Snap, o: M.Occurrence): string {
  const owner = ownerOf(sn, o);
  return owner ? `${occLabel(o)} · ${displayName(owner)}` : occLabel(o);
}

/** Mục sự kiện. `open = null` → mục chỉ để xem (không phải nút, 08 §3.6 "Lần tới"). */
export function occItem(sn: Snap, o: M.Occurrence, open: ((personId: string) => void) | null, t: Ymd = today()): HTMLElement {
  const k = itemKind(o);
  const owner = ownerOf(sn, o);
  const days = daysBetween(t, o.date);
  const l1 = itemTitle(sn, o);
  let l3: string | null = null;
  if (o.class === "gio" && owner) {
    const t2 = (owner.death as JsonObj | null)?.tuoi as JsonObj | undefined;
    if (t2 && t2.value !== null) l3 = `${t2.label === "huong_duong" ? "Hưởng dương" : "Hưởng thọ"} ${t2.value} tuổi`;
  }
  const lunar = lunarOf(o.date);
  const cls = `pitem k-${k}${days === 0 ? " is-today" : ""}${days < 0 ? " is-past" : ""}${open ? "" : " static"}`;
  const attrs = { class: cls, "data-kind": k, "data-date": M.iso(o.date), "data-class": o.class };
  const btn = open
    ? h("button", { ...attrs, type: "button", onclick: () => { if (owner) open(owner.id as string); }, disabled: owner ? null : true })
    : h("div", attrs);
  // TK-07: dòng dưới ghi "thg 7" (tháng), không "Th 7" — tránh đọc thành Thứ Bảy.
  btn.append(
    h("span", { class: "pi-date", "aria-hidden": "true" },
      h("small", null, weekdayShort(o.date)), h("b", null, String(o.date.d)), h("small", null, `thg ${o.date.m}`)),
    h("span", { class: "pi-main" },
      h("span", { class: "pi-l1" }, l1),
      h("span", { class: "pi-l2" }, `${dmy(o.date)}${lunar ? ` · ${lunar}` : ""}`),
      l3 ? h("span", { class: "pi-l3" }, l3) : null),
    h("span", { class: "pi-side" },
      svg(PI[k], "ico pi-ico"), h("span", { class: "sr-only" }, KIND_WORD[k] + ". "),
      h("span", { class: "pi-rel" }, relText(days))));
  return btn;
}

// ---------------- ô nhập ngày (08 §2, R-UX-1) ----------------
export type DateMode = "death" | "birth" | "event";
export interface DateValueJ { calendar: "solar" | "lunar"; precision: "day" | "month_day" | "year"; year: number | null; month: number | null; day: number | null; leap: boolean }
export interface DateRead { empty: boolean; value: DateValueJ | null; error: string | null }

const intOf = (el: HTMLInputElement): number | null => (/^\d{1,4}$/.test(el.value.trim()) ? Number(el.value.trim()) : null);

/** Lỗi tồn tại của ngày âm (câu 01 §6.1), dùng bảng tháng của lõi. */
export function lunarMissing(y: number, m: number, d: number, leap: boolean): string | null {
  const c = cal();
  if (y < 1899 || y > 3000) return null; // ngoài miền đổi lịch: lưu được, không kiểm
  const len = c.monthLen(y, m, leap);
  if (len === null) return leap ? `Năm ${y} không có tháng ${m} nhuận` : `Không tìm thấy tháng ${m} âm lịch năm ${y}`;
  if (d > len) return `Tháng ${m}${leap ? " nhuận" : ""} âm lịch năm ${y} chỉ có ${len} ngày`;
  return null;
}

export interface DateInput {
  el: HTMLElement;
  read(): DateRead;
  set(v: DateValueJ | null): void;
  onChange(f: () => void): void;
  setResult(text: string | null, isError?: boolean): void;
  focus(): void;
  setRecurring(on: boolean): void;
  /** Đã gõ xong (đủ các ô đang hiện, năm đủ 4 chữ số) — chỉ khi đó mới báo lỗi lúc đang gõ. */
  complete(): boolean;
}

export function dateInput(idp: string, legend: string, mode: DateMode, required: boolean, legendHidden = false): DateInput {
  const listeners: (() => void)[] = [];
  const fire = () => { sync(); listeners.forEach((f) => f()); };
  const calSel = { v: "solar" as "solar" | "lunar" };
  const calGroup = pressGroup<"solar" | "lunar">(mode === "death" ? "Nhập ngày mất theo" : "Lịch", [["solar", mode === "death" ? "Nhập theo dương lịch" : "Dương lịch"], ["lunar", mode === "death" ? "Nhập theo âm lịch" : "Âm lịch"]], "solar", (v) => { calSel.v = v; fire(); }, "pseg two");
  calGroup.id = `${idp}-cal`;
  const num = (id: string, label: string, max: number) => {
    const input = h("input", { type: "text", id, inputmode: "numeric", pattern: "[0-9]*", autocomplete: "off", maxlength: String(max), class: "num" }) as HTMLInputElement;
    input.addEventListener("input", fire);
    return { input, wrap: h("label", { class: "field", for: id }, h("span", null, label), input) };
  };
  const D = num(`${idp}-d`, "Ngày", 2), Mo = num(`${idp}-m`, "Tháng", 2), Y = num(`${idp}-y`, "Năm", 4);
  const leap = h("input", { type: "checkbox", id: `${idp}-leap` }) as HTMLInputElement;
  const leapLbl = h("span", null, "Tháng nhuận");
  const leapWrap = h("label", { class: "check", for: `${idp}-leap` }, leap, leapLbl);
  const leapNote = h("p", { class: "fine", hidden: true });
  leap.addEventListener("change", fire);
  const unk = h("input", { type: "checkbox", id: `${idp}-unk` }) as HTMLInputElement;
  const unkWrap = h("label", { class: "check", for: `${idp}-unk` }, unk, h("span", null, mode === "death" ? "Không rõ năm mất" : "Không rõ năm"));
  unk.addEventListener("change", fire);
  const prec = { v: "day" as "day" | "year" };
  const precGroup = mode === "birth" ? pressGroup<"day" | "year">("Mức biết", [["day", "Biết đủ ngày"], ["year", "Chỉ biết năm"]], "day", (v) => { prec.v = v; fire(); }, "pseg two") : null;
  const err = h("p", { class: "ferr", id: `${idp}-err`, role: "alert", hidden: true });
  const result = h("p", { class: "dres", id: `${idp}-res`, "aria-live": "polite" });
  const dmyRow = h("div", { class: "fields-dmy" }, D.wrap, Mo.wrap, Y.wrap);
  let recurring = true;
  const el = h("fieldset", { class: "pfs date-in", id: idp },
    h("legend", { class: legendHidden ? "sr-only" : null }, legend + (required ? " *" : "")),
    calGroup, precGroup, dmyRow, h("div", { class: "date-opts" }, leapWrap, unkWrap), leapNote, err, result);

  // TK-03: rời cả nhóm ba ô ngày → coi như đã nhập xong, báo lỗi kể cả khi còn thiếu ô.
  let touched = false;
  dmyRow.addEventListener("focusout", (e) => {
    if (dmyRow.contains((e as FocusEvent).relatedTarget as Node | null)) return;
    if ([D, Mo, Y].some((f) => f.input.value.trim() !== "")) { touched = true; fire(); }
  });

  function sync(): void {
    const lunar = calSel.v === "lunar";
    const yearOnly = prec.v === "year";
    const unknown = lunar && unk.checked && mode !== "birth" && (mode === "death" || recurring);
    unkWrap.hidden = !lunar || mode === "birth" || (mode === "event" && !recurring);
    D.wrap.hidden = Mo.wrap.hidden = yearOnly;
    Y.wrap.hidden = unknown;
    // R-UX-1: hộp "Tháng nhuận" chỉ hiện khi năm âm có tháng nhuận (hoặc chưa biết năm).
    const y = intOf(Y.input);
    leapNote.hidden = true;
    if (!lunar || yearOnly) { leapWrap.hidden = true; return; }
    if (unknown || y === null || y < 1899 || y > 3000) { leapWrap.hidden = false; leapLbl.textContent = "Tháng nhuận"; return; }
    const lm = cal().leapMonth(y);
    if (lm === null) {
      leapWrap.hidden = true;
      leapNote.hidden = false;
      leapNote.textContent = `Năm ${y} không có tháng nhuận`;
    } else {
      leapWrap.hidden = false;
      leapLbl.textContent = `Tháng nhuận (năm ${y} nhuận tháng ${lm})`;
    }
  }

  function read(): DateRead {
    const lunar = calSel.v === "lunar";
    const yearOnly = mode === "birth" && prec.v === "year";
    const unknown = lunar && !unkWrap.hidden && unk.checked;
    const d = intOf(D.input), m = intOf(Mo.input), y = intOf(Y.input);
    const raw = [D.input.value, Mo.input.value, Y.input.value].map((s) => s.trim());
    const empty = yearOnly ? raw[2] === "" : unknown ? raw[0] === "" && raw[1] === "" : raw.every((s) => s === "");
    const fail = (e: string): DateRead => ({ empty, value: null, error: e });
    if (empty) return { empty, value: null, error: required ? (mode === "death" ? "Hãy nhập ngày mất" : "Hãy nhập ngày") : null };
    const leapOn = lunar && !leapWrap.hidden && leap.checked;
    if (yearOnly) {
      if (y === null) return fail("Năm phải là số");
      if (y < (lunar ? 1000 : 1000) || y > 3000) return fail("Năm phải từ 1000 đến 3000");
      return { empty, value: { calendar: calSel.v, precision: "year", year: y, month: null, day: null, leap: false }, error: null };
    }
    if (d === null || m === null || (!unknown && y === null)) return fail(unknown ? "Hãy nhập đủ ngày và tháng âm" : "Hãy nhập đủ ngày, tháng, năm");
    if (m < 1 || m > 12) return fail("Tháng phải từ 1 đến 12");
    if (!lunar) {
      if (y! < MIN_YEAR || y! > MAX_YEAR) return fail(`Năm phải từ ${MIN_YEAR} đến ${MAX_YEAR}`);
      const n = new Date(Date.UTC(y!, m, 0)).getUTCDate();
      if (d < 1 || d > n) return fail(`Tháng ${m} năm ${y} chỉ có ${n} ngày`);
      return { empty, value: { calendar: "solar", precision: "day", year: y, month: m, day: d, leap: false }, error: null };
    }
    if (d < 1 || d > 30) return fail("Ngày âm lịch phải từ 1 đến 30");
    if (unknown) return { empty, value: { calendar: "lunar", precision: "month_day", year: null, month: m, day: d, leap: leapOn }, error: null };
    if (y! < 1000 || y! > 3000) return fail("Năm phải từ 1000 đến 3000");
    const miss = lunarMissing(y!, m, d, leapOn);
    if (miss) return fail(miss);
    return { empty, value: { calendar: "lunar", precision: "day", year: y, month: m, day: d, leap: leapOn }, error: null };
  }

  function set(v: DateValueJ | null): void {
    if (!v) return;
    calSel.v = v.calendar;
    calGroup.querySelectorAll("button").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.v === v.calendar)));
    if (precGroup) {
      prec.v = v.precision === "year" ? "year" : "day";
      precGroup.querySelectorAll("button").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.v === prec.v)));
    }
    D.input.value = v.day === null ? "" : String(v.day);
    Mo.input.value = v.month === null ? "" : String(v.month);
    Y.input.value = v.year === null ? "" : String(v.year);
    unk.checked = v.precision === "month_day";
    sync();
    leap.checked = v.leap;
  }

  sync();
  return {
    el, read, set,
    onChange: (f) => { listeners.push(f); },
    setResult(text, isError = false) {
      err.hidden = !isError || !text;
      if (isError && text) { err.textContent = text; result.textContent = ""; }
      else result.textContent = text ?? "";
      for (const i of [D.input, Mo.input, Y.input]) {
        if (isError && text) { i.setAttribute("aria-invalid", "true"); i.setAttribute("aria-describedby", err.id); }
        else { i.removeAttribute("aria-invalid"); i.removeAttribute("aria-describedby"); }
      }
    },
    focus: () => { (calGroup.querySelector("button") as HTMLElement).focus(); },
    setRecurring(on) { recurring = on; sync(); },
    complete() {
      if (touched) return true;
      return [D, Mo, Y].every((f) => f.wrap.hidden || (f === Y ? f.input.value.trim().length === 4 : f.input.value.trim() !== ""));
    },
  };
}

/** Tóm tắt ngày dương: "Thứ Sáu, 08/08/2025". */
export function solarLong(d: Ymd): string {
  return `${["Chủ nhật", "Thứ Hai", "Thứ Ba", "Thứ Tư", "Thứ Năm", "Thứ Sáu", "Thứ Bảy"][(M.ymdJdn(d) + 1) % 7]}, ${dmy(d)}`;
}
export { dm, dmy };

// ---------------- quy tắc nhắc (01 §3.6) ----------------
export interface RuleInput { el: HTMLElement; read(): { rule: JsonObj | null; error: string | null; warn: string | null } }
const PRESETS: [number, string][] = [[0, "Đúng ngày"], [1, "1 ngày"], [3, "3 ngày"], [5, "5 ngày"], [10, "10 ngày"], [15, "15 ngày"]];

export function ruleInput(idp: string, title: string, rule: JsonObj | null, withTt: boolean, defaults: JsonObj): RuleInput {
  const r = rule ?? { id: crypto.randomUUID(), enabled: true, offsets_days: defaults.default_offsets_days ?? [0, 3], time_local: defaults.default_time_local ?? "07:00", tien_thuong: false };
  const offs = new Set<number>(r.offsets_days as number[]);
  const en = h("input", { type: "checkbox", id: `${idp}-on`, checked: r.enabled === true }) as HTMLInputElement;
  const chips = h("div", { class: "chips", role: "group", "aria-label": "Nhắc trước" });
  for (const [k, t] of PRESETS) {
    chips.append(h("button", { type: "button", class: "chip", "data-k": String(k), "aria-pressed": String(offs.has(k)), onclick: (e: Event) => {
      const b = e.currentTarget as HTMLElement;
      const on = b.getAttribute("aria-pressed") !== "true";
      b.setAttribute("aria-pressed", String(on));
    } }, t));
  }
  const customVals = [...offs].filter((k) => !PRESETS.some(([p]) => p === k));
  const custom = h("input", { type: "text", inputmode: "numeric", id: `${idp}-custom`, class: "num sm", maxlength: "2", value: customVals.length ? String(customVals[0]) : "" }) as HTMLInputElement;
  const time = h("input", { type: "time", id: `${idp}-time`, value: r.time_local as string, class: "time" }) as HTMLInputElement;
  const tt = h("input", { type: "checkbox", id: `${idp}-tt`, checked: r.tien_thuong === true }) as HTMLInputElement;
  const err = h("p", { class: "ferr", role: "alert", hidden: true });
  const body = h("div", { class: "rule-body" },
    h("p", { class: "flabel" }, "Nhắc trước"), chips,
    h("label", { class: "field inline", for: `${idp}-custom` }, h("span", null, "Tự đặt (1–60 ngày)"), custom),
    h("label", { class: "field inline", for: `${idp}-time` }, h("span", null, "Giờ nhắc"), time),
    withTt ? h("label", { class: "check", for: `${idp}-tt` }, tt, h("span", null, "Nhắc thêm ngày tiên thường (chiều hôm trước ngày giỗ)")) : null,
    err);
  const syncEn = () => body.classList.toggle("off", !en.checked);
  en.addEventListener("change", syncEn);
  syncEn();
  const el = h("fieldset", { class: "pfs rule", id: idp },
    h("legend", null, title),
    h("label", { class: "check", for: `${idp}-on` }, en, h("span", null, "Bật nhắc")),
    body);
  return {
    el,
    read() {
      const set = new Set<number>();
      chips.querySelectorAll<HTMLElement>(".chip").forEach((b) => { if (b.getAttribute("aria-pressed") === "true") set.add(Number(b.dataset.k)); });
      const cv = custom.value.trim();
      err.hidden = true;
      custom.removeAttribute("aria-invalid");
      if (cv !== "") {
        const n = /^\d{1,2}$/.test(cv) ? Number(cv) : NaN;
        if (!(n >= 1 && n <= 60)) { err.textContent = "Số ngày tự đặt phải từ 1 đến 60"; err.hidden = false; custom.setAttribute("aria-invalid", "true"); return { rule: null, error: err.textContent, warn: null }; }
        set.add(n);
      }
      const list = [...set].sort((a, b) => a - b);
      if (list.length > 8) { err.textContent = "Tối đa 8 mốc nhắc"; err.hidden = false; return { rule: null, error: err.textContent, warn: null }; }
      const tl = /^([01]\d|2[0-3]):[0-5]\d$/.test(time.value) ? time.value : "07:00";
      return {
        rule: { id: r.id as string, enabled: en.checked, offsets_days: list, time_local: tl, tien_thuong: withTt && tt.checked },
        error: null,
        warn: en.checked && list.length === 0 ? "Chưa chọn mốc nhắc nào — sẽ không có lời nhắc" : null,
      };
    },
  };
}

export function ruleSummary(r: JsonObj | null | undefined): string {
  if (!r || r.enabled !== true) return "Không nhắc";
  const o = (r.offsets_days as number[]).map((k) => (k === 0 ? "đúng ngày" : `${k} ngày trước`));
  const s = o.length ? `${o.join(", ")}, lúc ${r.time_local as string}` : "chưa chọn mốc nhắc";
  return s.charAt(0).toUpperCase() + s.slice(1) + (r.tien_thuong ? " · có nhắc tiên thường" : "");
}

/** Liên kết quay lại (TK-21): chữ theo đích — "‹ Về danh sách hồ sơ" hoặc "‹ Về hồ sơ". */
export function backLink(label = "‹ Về danh sách hồ sơ", href = "#/nguoi-than/ho-so"): HTMLElement {
  return h("a", { class: "textlink pback", href }, label);
}
