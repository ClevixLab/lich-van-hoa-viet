// Trang: Sắp tới (08 §3.1), Theo kỳ (§3.2), Hồ sơ (§3.3), Hồ sơ một người (§3.6), thẻ nhắc (§5.5).
import { clear, h, svg } from "../dom";
import {
  M, addDays, cal, daysBetween, displayName, dm, dmy, lunarText, relText, snap, store, today, tuoiText,
  type JsonObj, type Snap, type Ymd,
} from "./core";
import type { Nav } from "./forms";
import { eventsInPeriod, periodBounds, periodNext, periodPrev, type PeriodFilter, type PeriodKind } from "./period";
import { drawPhoto } from "./photo";
import { PI, backLink, confirmDialog, occItem, pressGroup, ruleSummary, toast } from "./ui";

const openPerson = (nav: Nav) => (id: string) => nav.go(`#/nguoi-than/nguoi/${id}`);

export function welcome(): HTMLElement {
  return h("section", { class: "panel welcome" },
    h("h2", null, "Chưa có ai trong sổ"),
    h("p", null, "Thêm ông bà, cha mẹ đã khuất để được nhắc ngày giỗ theo âm lịch; thêm người đang sống để nhớ sinh nhật và ngày kỷ niệm."),
    addButtons());
}
export function addButtons(): HTMLElement {
  return h("div", { class: "addrow" },
    h("a", { class: "btnlink main", href: "#/nguoi-than/them/da-mat" }, "+ Thêm người đã mất"),
    h("a", { class: "btnlink", href: "#/nguoi-than/them/dang-song" }, "+ Thêm người đang sống"));
}

// ---------------- giới hạn web, nhắc sao lưu, cảnh báo lưu trữ ----------------
export function webLimitPanel(openIcs: () => void, open = true): HTMLElement {
  return h("details", { class: "panel notice-web", open },
    h("summary", null, "Lời nhắc trên ứng dụng web"),
    h("p", null, "Ứng dụng web không gửi được thông báo khi đang đóng. Lời nhắc chỉ hiện ở đầu trang khi bạn mở ứng dụng vào ngày cần nhắc. Để điện thoại tự nhắc cả khi không mở ứng dụng, hãy xuất tệp lịch (.ics) và mở bằng ứng dụng Lịch của máy."),
    h("button", { type: "button", class: "btn2", id: "web-ics", onclick: openIcs }, "Xuất tệp lịch (.ics)…"));
}

export async function storageNotices(sn: Snap): Promise<HTMLElement[]> {
  const s = await store();
  const out: HTMLElement[] = [];
  if (sn.persons.length && await s.backupReminderDue()) {
    const n = sn.settings.backup_nag_days as number;
    out.push(h("div", { class: "notice warnbox", role: "status", id: "nag-backup" },
      h("p", null, `Dữ liệu người thân đã thay đổi hơn ${n} ngày mà chưa sao lưu. Nếu mất máy hoặc trình duyệt xóa dữ liệu, sẽ không lấy lại được.`),
      h("a", { class: "btnlink main", href: "#/nguoi-than/sao-luu" }, "Sao lưu ngay")));
  }
  // TK-18: cảnh báo lưu trữ có "Sao lưu ngay" và "Đã hiểu"; "Đã hiểu" ẩn 30 ngày (ghi trong kho meta của IndexedDB,
  // khóa `ui_persist_notice_hidden_at` — không phải localStorage, S4').
  const hiddenAt = (await s.meta("ui_persist_notice_hidden_at")) as string | undefined;
  const hiddenFresh = !!hiddenAt && Date.now() - Date.parse(hiddenAt) < 30 * 86_400_000;
  if (sn.persons.length && (await s.persistResult()) === false && !hiddenFresh) {
    const box = h("div", { class: "notice", id: "persist-warn" },
      h("p", null, "Trình duyệt có thể xóa dữ liệu khi máy thiếu chỗ — hãy sao lưu. Trên iPhone, thêm ứng dụng vào Màn hình chính để giữ dữ liệu lâu hơn."),
      h("div", { class: "notice-acts" },
        h("a", { class: "btnlink main sm", href: "#/nguoi-than/sao-luu" }, "Sao lưu ngay"),
        h("button", { type: "button", class: "btn2 sm", id: "persist-ok", onclick: async () => {
          box.remove();
          await new Promise<void>((ok) => {
            const tx = s.db.transaction("meta", "readwrite");
            tx.objectStore("meta").put(new Date().toISOString().slice(0, 19) + "Z", "ui_persist_notice_hidden_at");
            tx.oncomplete = () => ok(); tx.onerror = () => ok();
          });
        } }, "Đã hiểu")));
    out.push(box);
  }
  return out;
}

// ---------------- thẻ nhắc hôm nay (02 §7.5, 08 §5.5) ----------------
export interface Due { occ: M.Occurrence; t: M.RemType; time: number }
export function dueToday(sn: Snap, t: Ymd = today()): Due[] {
  const ds = sn.ds;
  if (!ds.reminders_enabled) return [];
  const out: Due[] = [];
  for (const occ of M.occurrencesWithTt(cal(), ds, t, addDays(t, 61))) {
    const rule = M.ruleOf(ds, occ);
    if (!rule || !rule.enabled) continue;
    if (occ.class === "tien_thuong") { if (daysBetween(t, occ.date) === 0) out.push({ occ, t: "tt", time: rule.time_local }); continue; }
    for (const k of rule.offsets_days) if (daysBetween(t, occ.date) === k) out.push({ occ, t: k, time: rule.time_local });
  }
  out.sort((a, b) => a.time - b.time || M.cmpOcc(a.occ, b.occ));
  return out;
}

const dismissed = new Set<string>();
/** Thẻ nhắc hôm nay. `compact` (Tờ lịch, TK-15): một thẻ, các lời nhắc còn lại gộp thành một dòng dẫn tới Sắp tới. */
export function reminderCards(sn: Snap, nav: Nav, onEmpty?: () => void, compact = false): HTMLElement | null {
  const due = dueToday(sn).filter((d) => !dismissed.has(`${d.occ.entity_id}:${d.occ.class}:${String(d.t)}`));
  if (!due.length) return null;
  const box = h("section", { class: `remind${compact ? " compact" : ""}`, "aria-label": "Lời nhắc hôm nay" });
  const shown = due.slice(0, compact ? 1 : 3);
  for (const d of shown) {
    const key = `${d.occ.entity_id}:${d.occ.class}:${String(d.t)}`;
    const txt = M.reminderTextNamed(cal(), d.occ, d.t);
    const owner = d.occ.class === "gio" || d.occ.class === "tien_thuong" || d.occ.class.startsWith("sinh") ? sn.byId.get(d.occ.entity_id) : sn.byId.get(((sn.events.find((e) => e.id === d.occ.entity_id)?.person_ids as string[]) ?? [])[0] ?? "");
    const card = h("div", { class: "rcard", "data-k": String(d.t) },
      svg(PI[d.occ.class === "gio" ? "gio" : d.occ.class === "tien_thuong" ? "tt" : d.occ.class.startsWith("sinh") ? "sn" : "ev"], "ico rc-ico"),
      h("div", { class: "rc-main" }, h("p", { class: "rc-t" }, txt.title), h("p", { class: "rc-b" }, txt.body),
        h("div", { class: "rc-acts" },
          d.occ.class === "gio" && d.t === 0 && owner ? h("a", { class: "btnlink main", href: `#/nguoi-than/tuong-niem/${owner.id as string}` }, "Mở trang tưởng niệm") : null,
          owner ? h("button", { type: "button", class: "btn2", onclick: () => nav.go(`#/nguoi-than/nguoi/${owner.id as string}`) }, "Xem") : null,
          h("button", { type: "button", class: "btn2", onclick: () => { dismissed.add(key); card.remove(); if (!box.querySelector(".rcard")) { box.remove(); onEmpty?.(); } } }, "Đã biết"))));
    box.append(card);
  }
  const rest = due.length - shown.length;
  if (rest > 0) box.append(h("a", { class: "textlink rc-more", href: "#/nguoi-than" }, compact ? `Còn ${rest} lời nhắc khác hôm nay — Xem` : `… và ${rest} lời nhắc khác`));
  return box;
}

// ---------------- Sắp tới ----------------
export async function renderUpcoming(root: HTMLElement, nav: Nav, openIcs: () => void): Promise<void> {
  const sn = await snap();
  const t = today();
  const notices = await storageNotices(sn);
  if (!sn.persons.length) {
    root.append(...notices, welcome(), webLimitPanel(openIcs));
    return;
  }
  const items = M.upcoming(cal(), sn.ds, t, 30);
  // TK-19: mục giỗ của hôm nay có lối vào trang tưởng niệm (kể cả khi hồ sơ tắt nhắc).
  const list = h("div", { class: "plist", id: "upcoming-list" }, ...items.map((u) => {
    const it = occItem(sn, u.occ, openPerson(nav), t);
    if (u.occ.class !== "gio" || u.days !== 0) return it;
    return h("div", { class: "pitem-wrap" }, it,
      h("a", { class: "btnlink main sm mem-link", href: `#/nguoi-than/tuong-niem/${u.occ.entity_id}` }, "Mở trang tưởng niệm"));
  }));
  const cards = reminderCards(sn, nav);
  root.append(
    h("div", { class: "pcols" },
      h("div", { class: "pcol-main" },
        cards, ...notices,
        h("section", { class: "panel" },
          h("h2", null, "30 ngày tới"),
          items.length ? list : h("p", { class: "empty-hint" }, "Trong 30 ngày tới không có ngày giỗ, sinh nhật hay sự kiện nào. ",
            h("a", { class: "textlink", href: "#/nguoi-than/theo-ky" }, "Xem theo tháng, quý, năm")))),
      h("div", { class: "pcol-side" }, webLimitPanel(openIcs), h("section", { class: "panel" }, h("h2", null, "Thêm vào sổ"), addButtons()))));
}

// ---------------- Theo kỳ (R13) ----------------
const KIND_OPTS: [PeriodKind, string][] = [["tuan", "Tuần"], ["thang", "Tháng"], ["quy", "Quý"], ["nam", "Năm"]];
const FILTER_OPTS: [PeriodFilter, string][] = [["tat_ca", "Tất cả"], ["da_mat", "Người đã mất"], ["dang_song", "Người đang sống"]];
const ROMAN = ["I", "II", "III", "IV"];
const pstate = { kind: "thang" as PeriodKind, anchor: null as Ymd | null, filter: "tat_ca" as PeriodFilter };

export function periodTitle(kind: PeriodKind, from: Ymd, to: Ymd): string {
  if (kind === "tuan") return from.y === to.y ? `Tuần ${dm(from)} – ${dmy(to)}` : `Tuần ${dmy(from)} – ${dmy(to)}`;
  if (kind === "thang") return `Tháng ${from.m}/${from.y} · ${dm(from)} – ${dm(to)}`;
  if (kind === "quy") return `Quý ${ROMAN[Math.floor((from.m - 1) / 3)]}/${from.y} · ${dm(from)} – ${dm(to)}`;
  return `Năm ${from.y} · ${dm(from)} – ${dm(to)}`;
}
const THIS: Record<PeriodKind, string> = { tuan: "tuần này", thang: "tháng này", quy: "quý này", nam: "năm này" };

export async function renderPeriod(root: HTMLElement, nav: Nav): Promise<void> {
  const sn = await snap();
  if (!sn.persons.length) { root.append(welcome()); return; }
  const t = today();
  pstate.anchor ??= t;
  const count = h("p", { class: "pcount", id: "period-count" });
  const list = h("div", { class: "plist", id: "period-list" });
  const title = h("h2", { class: "ptitle", id: "period-title", tabindex: "-1" });
  const todayBtn = h("button", { type: "button", class: "todaybtn sm", id: "period-today", onclick: () => { pstate.anchor = t; draw(); title.focus(); } }, "Hôm nay");
  const curNote = h("p", { class: "pcur", id: "period-current" }, "Đang xem kỳ có hôm nay");
  const draw = () => {
    const r = eventsInPeriod(cal(), sn.ds, pstate.kind, pstate.anchor!, pstate.filter);
    title.textContent = periodTitle(pstate.kind, r.from, r.to);
    count.textContent = `${r.items.length} mục`;
    const b = periodBounds(pstate.kind, t);
    // TK-23: không để nút "Hôm nay" bị tắt (trông như hỏng) — khi đang xem kỳ hiện tại thì thay bằng một dòng chữ.
    const isCur = M.cmpYmd(b.from, r.from) === 0;
    todayBtn.hidden = isCur; curNote.hidden = !isCur;
    clear(list);
    if (!r.items.length) {
      const isThis = M.cmpYmd(b.from, r.from) === 0;
      list.append(h("p", { class: "empty-hint" }, `Không có ngày giỗ, sinh nhật hay sự kiện nào trong ${isThis ? THIS[pstate.kind] : "kỳ này"}.${pstate.filter !== "tat_ca" ? ` (đang lọc: ${FILTER_OPTS.find((f) => f[0] === pstate.filter)![1]})` : ""}`));
      return;
    }
    const group = pstate.kind === "quy" || pstate.kind === "nam";
    let lastM = -1;
    for (const o of r.items) {
      if (group && o.date.m !== lastM) { lastM = o.date.m; list.append(h("h3", { class: "pmonth" }, `Tháng ${o.date.m}/${o.date.y}`)); }
      list.append(occItem(sn, o, openPerson(nav), t));
    }
  };
  root.append(h("section", { class: "panel period" },
    h("div", { class: "pctl" },
      pressGroup("Loại kỳ", KIND_OPTS, pstate.kind, (v) => { pstate.kind = v; draw(); }, "pseg four"),
      pressGroup("Lọc", FILTER_OPTS, pstate.filter, (v) => { pstate.filter = v; draw(); }, "pseg three")),
    h("div", { class: "pnav" },
      h("button", { type: "button", class: "iconbtn", id: "period-prev", "aria-label": "Kỳ trước", onclick: () => { pstate.anchor = periodPrev(pstate.kind, pstate.anchor!); draw(); } }, "‹"),
      h("div", { class: "pnav-mid", "aria-live": "polite" }, title, count),
      h("button", { type: "button", class: "iconbtn", id: "period-next", "aria-label": "Kỳ sau", onclick: () => { pstate.anchor = periodNext(pstate.kind, pstate.anchor!); draw(); } }, "›")),
    h("div", { class: "pnav-today" }, todayBtn, curNote),
    list));
  draw();
}

// ---------------- Hồ sơ (danh sách) ----------------
function personSummary(p: JsonObj, t: Ymd): string {
  if (p.kind === "deceased") {
    const d = p.death as JsonObj;
    const l = d.lunar as JsonObj;
    const base = `Giỗ ${l.day as number}/${l.month as number} âm lịch`;
    // TK-27: không rõ năm mất vẫn có "lần tới" (giỗ lặp hằng năm).
    const nx = M.anniversariesIn(cal(), l.year as number | null, l.month as number, l.day as number, 2, t, addDays(t, 400))[0];
    const tail = l.year === null ? " (không rõ năm mất)" : "";
    return nx ? `${base} · lần tới ${dmy(nx.solar)} (${relText(daysBetween(t, nx.solar)).toLowerCase()})${tail}` : `${base}${tail}`;
  }
  const b = p.birth as JsonObj | null;
  if (!b) return "Chưa có ngày sinh";
  const s = b.solar as JsonObj | null;
  if (s && (b.input as JsonObj).precision === "day") {
    const a = M.tuoiDuong({ y: s.y as number, m: s.m as number, d: s.d as number }, t, p.feb29_rule as M.Feb29Rule);
    return `Sinh ${dmy({ y: s.y as number, m: s.m as number, d: s.d as number })}${a !== null ? ` · ${a} tuổi` : ""}`;
  }
  return `Sinh năm ${(b.input as JsonObj).year as number}`;
}

export async function renderList(root: HTMLElement, nav: Nav): Promise<void> {
  const sn = await snap();
  const t = today();
  // TK-30: xếp theo tên (chữ cuối của họ tên) rồi đến cả họ tên — nhiều nhà cùng họ.
  const given = (p: JsonObj) => ((p.ho_ten as string).trim().split(/\s+/).pop() ?? "");
  const sorted = sn.persons.slice().sort((a, b) => given(a).localeCompare(given(b), "vi") || (a.ho_ten as string).localeCompare(b.ho_ten as string, "vi"));
  const box = (kind: "deceased" | "living", label: string, add: string) => {
    const ps = sorted.filter((p) => p.kind === kind);
    return h("section", { class: "panel plbox", id: `list-${kind}` },
      h("div", { class: "plhead" }, h("h2", null, `${label} (${ps.length})`), h("a", { class: "btnlink sm", href: add }, "+ Thêm")),
      ps.length ? h("ul", { class: "plrows" }, ...ps.map((p) => h("li", null, h("button", { type: "button", class: "prow", onclick: () => openPerson(nav)(p.id as string) },
        h("span", { class: "avatar", "aria-hidden": "true" }, [...(p.ho_ten as string).trim().split(/\s+/).pop() ?? "?"][0]?.toUpperCase() ?? "?"),
        h("span", { class: "pr-main" }, h("span", { class: "pr-name" }, displayName(p)), p.quan_he ? h("span", { class: "pr-sub" }, p.quan_he as string) : null,
          h("span", { class: "pr-sub" }, personSummary(p, t)))))))
        : h("p", { class: "empty-hint" }, "Chưa có ai."));
  };
  root.append(...await storageNotices(sn), h("div", { class: "pgrid2" },
    box("deceased", "Người đã mất", "#/nguoi-than/them/da-mat"),
    box("living", "Người đang sống", "#/nguoi-than/them/dang-song")));
}

// ---------------- Hồ sơ một người ----------------
function row(k: string, v: string | null | undefined): HTMLElement | null {
  return v ? h("div", null, h("dt", null, k), h("dd", null, v)) : null;
}
function dvText(dv: JsonObj): string {
  if (dv.precision === "year") return `năm ${dv.year as number}${dv.calendar === "lunar" ? " (âm lịch)" : ""}`;
  if (dv.calendar === "solar") return dmy({ y: dv.year as number, m: dv.month as number, d: dv.day as number });
  return lunarText({ year: dv.year as number | null, month: dv.month as number, day: dv.day as number, leap: dv.leap === true });
}

export async function renderPerson(root: HTMLElement, id: string, nav: Nav): Promise<void> {
  const sn = await snap();
  const s = await store();
  const p = sn.byId.get(id);
  if (!p) { root.append(backLink(), h("p", { class: "warn" }, "Không tìm thấy hồ sơ này (có thể đã bị xóa). "), h("a", { class: "textlink", href: "#/nguoi-than/ho-so" }, "Về danh sách hồ sơ")); return; }
  const dead = p.kind === "deceased";
  const t = today();
  document.title = "Hồ sơ — Lịch và Văn hóa Việt";
  const name = displayName(p);
  const h1 = h("h1", { tabindex: "-1", id: "person-name" }, name);
  const canvas = dead ? h("canvas", { class: "pp-photo", width: "120", height: "150", "aria-hidden": "true" }) as HTMLCanvasElement : null;
  if (canvas && typeof p.photo_id === "string") s.photoBytes(p.photo_id).then((b) => drawPhoto(canvas, b)).catch(() => {});

  // lần tới
  const own = sn.ds.persons.filter((x) => x.id === id);
  const evIds = new Set(sn.events.filter((e) => (e.person_ids as string[]).includes(id)).map((e) => e.id as string));
  // TK-34: mọi lần xuất hiện trong 12 tháng tới (giỗ, tiên thường, sinh nhật, sự kiện), tối đa 5 mục, theo thứ tự 02 §7.2.
  const nextList = M.occurrencesWithTt(cal(), { ...sn.ds, persons: own, events: sn.ds.events.filter((e) => evIds.has(e.id)) }, t, addDays(t, 365)).slice(0, 5);

  const d = p.death as JsonObj | null;
  const b = p.birth as JsonObj | null;
  const mem = p.memorial as JsonObj | null;
  const facts = h("dl", { class: "facts pfacts" },
    d ? row("Ngày mất (đã nhập)", dvText(d.input as JsonObj)) : null,
    d ? row("Âm lịch", `${lunarText(d.lunar as never, true, false)} — ${d.lunar_source === "converted" ? "đổi từ ngày dương" : "nhập theo âm lịch"}`) : null,
    d && d.solar && (d.input as JsonObj).calendar === "lunar" ? row("Dương lịch", dmy(d.solar as never)) : null,
    d ? row("Tuổi", tuoiText(d.tuoi as JsonObj) ?? "Chưa đủ năm sinh để tính") : null,
    b ? row("Ngày sinh", dvText(b.input as JsonObj)) : row("Ngày sinh", "Chưa ghi"),
    b && b.lunar_year !== null ? row("Năm âm sinh", `${b.lunar_year as number} — ${b.can_chi as string}${b.lunar_year_approx ? " (khoảng)" : ""}`) : null,
    !dead && b && b.solar && (b.input as JsonObj).precision === "day" ? row("Tuổi", (() => {
      const a = M.tuoiDuong(b.solar as never, t, p.feb29_rule as M.Feb29Rule); const am = M.tuoiAm(cal(), b.lunar_year as number, t);
      return a !== null ? `${a}${am !== null ? ` (tuổi âm ${am})` : ""}` : null;
    })()) : null,
    b ? row("Nơi sinh", b.noi_sinh as string) : null,
    row("Tên hiệu / pháp danh", p.ten_hieu as string),
    dead ? row("Nơi an táng", p.noi_an_tang as string) : null,
    dead && mem ? row("Giỗ hết", `năm thứ ${mem.gio_het_nam as number} sau năm mất`) : null,
    row("Ghi chú", p.ghi_chu as string));

  const events = sn.events.filter((e) => (e.person_ids as string[]).includes(id))
    .sort((a, b2) => String((a.date as JsonObj).year).localeCompare(String((b2.date as JsonObj).year)));
  const typeName = (e: JsonObj) => (e.tieu_de as string) || ((sn.types.find((x) => x.id === e.type_id)?.ten as string) ?? "Sự kiện");
  const evList = events.length ? h("ul", { class: "evrows" }, ...events.map((e) => h("li", { class: "evrow" },
    h("span", { class: "ev-main" }, h("b", null, typeName(e)), h("span", null, ` · ${dvText(e.date as JsonObj)} · ${e.recurring ? "hằng năm" : "một lần"}`)),
    h("span", { class: "ev-acts" },
      h("a", { class: "btn2 sm", href: `#/nguoi-than/su-kien/${id}/${e.id as string}` }, "Sửa"),
      h("button", { type: "button", class: "btn2 sm danger", onclick: async () => {
        if (await confirmDialog({ title: "Xóa sự kiện?", body: [`Xóa sự kiện “${typeName(e)}”? Không hoàn tác được.`], ok: "Xóa sự kiện", cancel: "Giữ lại", danger: true, focusCancel: true, onOk: async () => { await s.deleteEvent(e.id as string); } })) {
          toast("Đã xóa sự kiện."); nav.go(`#/nguoi-than/nguoi/${id}`, true);
        }
      } }, "Xóa")))))
    : h("p", { class: "empty-hint" }, "Chưa có sự kiện nào.");

  const rem = p.reminders as JsonObj;
  const remBox = h("dl", { class: "facts pfacts" },
    dead ? row("Nhắc giỗ", ruleSummary(rem.gio as JsonObj | null)) : row("Nhắc sinh nhật", ruleSummary(rem.sinh_nhat as JsonObj | null)),
    !dead && p.sinh_nhat_am ? row("Nhắc sinh nhật âm lịch", ruleSummary(rem.sinh_nhat_am as JsonObj | null)) : null);

  const del = h("button", { type: "button", class: "btn2 danger", id: "del-person", onclick: async () => {
    const ok = await confirmDialog({
      title: "Xóa hồ sơ?", danger: true, focusCancel: true, ok: "Xóa hồ sơ", cancel: "Giữ lại",
      body: [`Xóa hồ sơ của ${name}? ${dead ? "Ngày giỗ, lời nhắc và ảnh" : "Sinh nhật, lời nhắc"} của người này sẽ bị xóa khỏi máy. Sự kiện gắn với người này vẫn còn nếu có người khác cùng tham gia. Không hoàn tác được.`],
      onOk: async () => {
        await s.deletePerson(id);
        for (const e of await s.events()) if ((e.person_ids as string[]).length === 0 && evIds.has(e.id as string)) await s.deleteEvent(e.id as string);
      },
    });
    if (ok) { toast("Đã xóa hồ sơ."); nav.go("#/nguoi-than/ho-so", true); }
  } }, "Xóa hồ sơ");

  root.append(
    backLink(),
    h("div", { class: "phead" }, canvas,
      h("div", { class: "phead-t" }, h1, p.quan_he ? h("p", { class: "phead-rel" }, p.quan_he as string) : null,
        h("div", { class: "phead-acts" },
          dead ? h("a", { class: "btnlink main", href: `#/nguoi-than/tuong-niem/${id}`, id: "open-memorial" }, "Tưởng niệm") : null,
          h("a", { class: "btnlink", href: `#/nguoi-than/sua/${id}`, id: "edit-person" }, "Sửa")))),
    h("div", { class: "pcols" },
      // TK-08: "Lần tới" ngay dưới đầu trang (trước "Thông tin"); mục chỉ để xem, không phải nút.
      h("div", { class: "pcol-main" },
        h("section", { class: "panel", id: "next-box" }, h("h2", null, "Lần tới"),
          nextList.length ? h("div", { class: "plist" }, ...nextList.map((o) => occItem(sn, o, null, t))) : h("p", { class: "empty-hint" }, "Chưa có ngày nào trong năm tới.")),
        h("section", { class: "panel" }, h("h2", null, "Thông tin"), facts)),
      h("div", { class: "pcol-side" },
        h("section", { class: "panel", id: "event-box" },
          h("div", { class: "plhead" }, h("h2", null, "Sự kiện"), h("a", { class: "btnlink sm", href: `#/nguoi-than/su-kien/${id}/moi`, id: "add-event" }, "+ Thêm sự kiện")), evList),
        h("section", { class: "panel" }, h("h2", null, "Nhắc"), remBox),
        h("div", { class: "delrow" }, del))));
  h1.focus();
}
