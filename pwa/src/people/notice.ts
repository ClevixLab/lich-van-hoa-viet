// Thông tin báo tử (01 §3.2a, YC-2026-05, US-M-07, R-PRIV-2) và biểu mẫu "Cập nhật: đã mất" (08 §3.11).
//
// Riêng tư: `death.notice` chỉ được đọc ở tệp này (biểu mẫu, khối trên trang hồ sơ). `snap()` bỏ `notice` trước khi
// đưa dữ liệu cho lõi, nên thẻ nhắc, Sắp tới, Theo kỳ, tệp .ics, tên tệp và màn hình tưởng niệm không thể dùng tới nó.
import { clear, h } from "../dom";
import { buildPerson, normalizePersonNames, validate, type NewAttachment } from "../store/index";
import { M, cal as cal0, displayName, dmy, nowTs, snap, store, today, type JsonObj } from "./core";
import { deathPreview, errSummary, textField, type Issue, type Nav } from "./forms";
import { ATTACHMENT_LIMITS, drawPhoto, processPhoto } from "./photo";
import { download } from "./backup";
import { backLink, confirmDialog, dateInput, ruleSummary, toast, type DateValueJ } from "./ui";

/** Tối đa 6 tệp đính kèm (01 §3.2a, YC-2026-07). */
export const MAX_ATTACHMENTS = 6;
const MAX_PDF = 10 * 1024 * 1024;
export const ATTACHMENT_KIND_TEXT: Record<string, string> = { tin_bao_tu: "Ảnh tin báo tử", giay_chung_tu: "Giấy chứng tử (ảnh hoặc bản quét PDF)", khac: "Khác" };
const sizeText = (n: number) => (n >= 1048576 ? `${(n / 1048576).toFixed(1).replace(".", ",")} MB` : `${Math.max(1, Math.round(n / 1024))} KB`);

/** Một tệp đính kèm trong biểu mẫu: `id` của phần tử danh sách, `file_id` của bản ghi Attachment. */
interface AttItem { id: string; file_id: string; kind: string; mime: string; name: string | null; fresh: NewAttachment | null }

export interface NoticeSection {
  el: HTMLElement;
  /** Đọc và kiểm; thêm lỗi vào `issues`. `notice` = null khi mọi ô trống và không có tệp. Tệp gỡ khỏi danh sách
   *  được lớp lưu trữ tự xóa khi lưu hồ sơ. */
  collect(show: boolean, issues: Issue[]): { notice: JsonObj | null; newAttachments: NewAttachment[] };
}

/** Ô xem một tệp đính kèm: ảnh vẽ lên canvas; PDF hiện nhãn (không mở trong trang). */
function attThumb(it: { file_id: string; mime: string; kind: string; name: string | null; fresh: NewAttachment | null }, i: number): HTMLElement {
  const label = `${ATTACHMENT_KIND_TEXT[it.kind] ?? "Tệp"} ${i + 1}`;
  if (it.mime === "application/pdf") {
    return h("div", { class: "cert-thumb cert-pdf", role: "img", "aria-label": `${label} (PDF)` }, h("span", null, "PDF"), it.name ? h("span", { class: "fine" }, it.name) : null);
  }
  const c = h("canvas", { class: "cert-thumb", width: "120", height: "160", "aria-label": label }) as HTMLCanvasElement;
  if (it.fresh) void drawPhoto(c, it.fresh.bytes, it.mime, true);
  else void store().then((s) => s.attachmentBytes(it.file_id)).then((b) => drawPhoto(c, b, it.mime, true)).catch(() => {});
  return c;
}

/** Khối "Thông tin báo tử (không bắt buộc)". `open`: mở sẵn (biểu mẫu Cập nhật: đã mất) hay thu gọn (người đã mất mới). */
export function noticeSection(old: JsonObj | null, open: boolean, onDirty: () => void): NoticeSection {
  const n = old ?? {};
  const fBy = textField("nf-by", "Người báo tin", (n.reported_by as string) ?? "", 200);
  const fPlace = textField("nf-place", "Mất tại", (n.place as string) ?? "", 200, { hint: "ví dụ: tại nhà, bệnh viện…" });
  const fReason = textField("nf-reason", "Lý do", (n.reason as string) ?? "", 500);
  const fNote = textField("nf-note", "Ghi chú", (n.note as string) ?? "", 2000, { multiline: true });
  const date = dateInput("nf-date", "Ngày nhận tin", "notice", false);
  date.set((n.notice_date as unknown as DateValueJ | null) ?? null);
  date.onChange(onDirty);

  // Tệp đính kèm (YC-2026-07): ảnh tin báo tử, ảnh/bản quét giấy chứng tử (ảnh hoặc PDF), tệp khác.
  const metaOf = new Map<string, JsonObj>();
  const items: AttItem[] = ((n.attachments as JsonObj[] | undefined) ?? []).map((a) => ({ id: a.id as string, file_id: a.file_id as string, kind: a.kind as string, mime: "image/jpeg", name: null, fresh: null }));
  const list = h("div", { class: "cert-list", id: "nf-certs" });
  const err = h("p", { class: "ferr", role: "alert", hidden: true });
  const kindSel = h("select", { id: "nf-kind", class: "tin sm" },
    ...Object.entries(ATTACHMENT_KIND_TEXT).map(([k, t]) => h("option", { value: k, selected: k === "giay_chung_tu" }, t))) as HTMLSelectElement;
  const file = h("input", { type: "file", id: "nf-file", accept: "image/*,application/pdf", class: "sr-only", tabindex: "-1", "aria-hidden": "true" }) as HTMLInputElement;
  const add = h("button", { type: "button", class: "btn2", id: "nf-add", onclick: () => file.click() }, "Thêm tệp");
  const draw = () => {
    clear(list);
    items.forEach((it, i) => {
      list.append(h("div", { class: "cert-item" }, attThumb(it, i),
        h("p", { class: "fine" }, ATTACHMENT_KIND_TEXT[it.kind] ?? "Tệp"),
        h("button", { type: "button", class: "btn2 sm", "aria-label": `Bỏ tệp ${i + 1}`, onclick: () => { items.splice(i, 1); onDirty(); draw(); } }, "Bỏ tệp")));
    });
    add.hidden = items.length >= MAX_ATTACHMENTS;
  };
  void store().then((s) => s.attachments()).then((all) => {
    for (const m of all) metaOf.set(m.id as string, m);
    for (const it of items) { const m = metaOf.get(it.file_id); if (m) { it.mime = m.mime as string; it.name = (m.file_name as string | null) ?? null; } }
    draw();
  }).catch(() => {});
  file.addEventListener("change", async () => {
    const f = file.files?.[0];
    file.value = "";
    if (!f) return;
    err.hidden = true;
    try {
      const fileId = crypto.randomUUID();
      let fresh: NewAttachment;
      if (f.type === "application/pdf" || /\.pdf$/i.test(f.name)) {
        const go = await confirmDialog({
          title: "Đính kèm tệp PDF?", ok: "Đính kèm PDF", cancel: "Không đính kèm", cls: "pdf-warn-dialog",
          body: [
            h("p", null, "PDF được lưu nguyên bản, có thể còn tên máy quét, phần mềm, ngày quét."),
            h("p", { class: "fine" }, "Nếu không muốn, hãy chụp ảnh tờ giấy thay vì đính kèm PDF — ảnh sẽ được bỏ các thông tin này."),
          ],
        });
        if (!go) return;
        if (f.size > MAX_PDF) throw new Error("Tệp PDF lớn hơn 10 MB — hãy chọn tệp nhỏ hơn");
        const bytes = new Uint8Array(await f.arrayBuffer());
        if (String.fromCharCode(...bytes.subarray(0, 5)) !== "%PDF-") throw new Error("Tệp này không phải PDF hợp lệ");
        fresh = { id: fileId, mime: "application/pdf", file_name: f.name || null, bytes };
      } else {
        const p = await processPhoto(f, ATTACHMENT_LIMITS);
        fresh = { id: fileId, mime: p.mime, file_name: f.name || null, bytes: p.bytes };
      }
      items.push({ id: crypto.randomUUID(), file_id: fileId, kind: kindSel.value, mime: fresh.mime, name: fresh.file_name, fresh });
      onDirty(); draw();
    } catch (e) { err.textContent = (e as Error).message; err.hidden = false; }
  });
  draw();

  const el = h("details", { class: "pfs more notice-sec", id: "nf", open: open || !!old },
    h("summary", null, "Thông tin báo tử (không bắt buộc)"),
    h("p", { class: "fine" }, "Chỉ hiện trên trang hồ sơ này. Không đưa vào lời nhắc, tệp lịch (.ics) hay trang tưởng niệm; có trong tệp sao lưu (đã mã hóa)."),
    fBy.el, fPlace.el, date.el, fReason.el,
    h("div", { class: "field" }, h("p", { class: "flabel" }, `Tệp đính kèm — tin báo tử, giấy chứng tử (tối đa ${MAX_ATTACHMENTS})`), list,
      h("div", { class: "photo-acts" }, h("label", { class: "field inline", for: "nf-kind" }, h("span", null, "Loại"), kindSel), file, add, err),
      h("p", { class: "fine" }, "Ảnh được nén lại và bỏ thông tin vị trí, máy chụp. Tệp PDF được lưu nguyên bản.")),
    fNote.el);

  return {
    el,
    collect(show, issues) {
      const tf = (f: ReturnType<typeof textField>) => { const r = f.read(); if (show) f.showErr(r.err); if (r.err) issues.push({ msg: r.err, focus: () => f.input.focus() }); return r.v; };
      const by = tf(fBy), place = tf(fPlace), reason = tf(fReason), note = tf(fNote);
      const dr = date.read();
      let nd: DateValueJ | null = null;
      if (dr.error) { if (show) date.setResult(dr.error, true); issues.push({ msg: `Ngày nhận tin: ${dr.error}`, focus: () => date.focus() }); }
      else if (dr.value) {
        const v = dr.value;
        if (M.cmpYmd(M.ymd(v.year!, v.month!, v.day!), today()) > 0) {
          if (show) date.setResult("Ngày nhận tin không được sau hôm nay", true);
          issues.push({ msg: "Ngày nhận tin không được sau hôm nay", focus: () => date.focus() });
        } else { date.setResult(null); nd = v; }
      }
      const orNull = (s: string) => (s === "" ? null : s);
      const attachments: JsonObj[] = items.map((i) => ({ id: i.id, kind: i.kind, file_id: i.file_id }));
      const notice: JsonObj = { reported_by: orNull(by), place: orNull(place), reason: orNull(reason), notice_date: nd as unknown as JsonObj | null, attachments, note: orNull(note) };
      const empty = attachments.length === 0 && [notice.reported_by, notice.place, notice.reason, notice.notice_date, notice.note].every((x) => x === null);
      return { notice: empty ? null : notice, newAttachments: items.filter((i) => i.fresh).map((i) => i.fresh!) };
    },
  };
}

/** Khối "Thông tin báo tử" trên trang hồ sơ (thu gọn). Trả null khi không có. */
export function noticeBlock(p: JsonObj): HTMLElement | null {
  const n = ((p.death as JsonObj | null)?.notice as JsonObj | null | undefined) ?? null;
  if (!n) return null;
  const row = (k: string, v: string | null) => (v ? [h("div", null, h("dt", null, k), h("dd", null, v))] : []);
  const nd = n.notice_date as JsonObj | null;
  const list = h("div", { class: "cert-list", id: "notice-atts" });
  const atts = (n.attachments as JsonObj[] | undefined) ?? [];
  if (atts.length) {
    void store().then((s) => s.attachments()).then((all) => {
      const byId = new Map(all.map((m) => [m.id as string, m]));
      atts.forEach((a, i) => {
        const m = byId.get(a.file_id as string);
        const pdf = m?.mime === "application/pdf";
        const dl = h("button", { type: "button", class: "btn2 sm att-dl", "data-i": String(i), onclick: async () => {
          // S10 / BM2-05: mọi đường đưa dữ liệu ra ngoài ứng dụng đều qua hộp xác nhận.
          const ok = await confirmDialog({
            title: "Tải tệp xuống máy?", ok: "Tải xuống", cancel: "Không tải", cls: "att-dl-dialog", focusCancel: true,
            body: [
              h("p", null, "Tệp có thể chứa thông tin riêng tư."),
              h("p", { class: "fine" }, "Tệp sẽ nằm trong thư mục Tải về của máy, ngoài ứng dụng: không có mật khẩu, ứng dụng khác có thể đọc được, và không tự xóa khi bạn xóa hồ sơ."),
              ...(pdf ? [h("p", { class: "fine" }, "Tệp PDF còn thông tin của máy quét (nếu có).")] : []),
            ],
          });
          if (!ok) return;
          const s = await store();
          // Tên tệp tải về là tên chung, không dùng tên gốc (01 §3.8a); tải bằng liên kết tải xuống — CSP giữ nguyên.
          download(`TepDinhKem-${i + 1}.${pdf ? "pdf" : "jpg"}`, await s.attachmentBytes(a.file_id as string), (m?.mime as string) ?? "application/octet-stream");
        } }, pdf ? "Tải tệp PDF…" : "Tải ảnh…");
        list.append(h("div", { class: "cert-item" }, attThumb({ file_id: a.file_id as string, kind: a.kind as string, mime: (m?.mime as string) ?? "image/jpeg", name: (m?.file_name as string | null) ?? null, fresh: null }, i),
          h("p", { class: "att-name" }, ATTACHMENT_KIND_TEXT[a.kind as string] ?? "Tệp"),
          h("p", { class: "fine" }, `${(m?.file_name as string | null) ?? (pdf ? "Tệp PDF" : "Ảnh")} · ${sizeText((m?.bytes as number) ?? 0)}`),
          pdf ? h("p", { class: "fine att-pdfnote" }, "Tệp PDF được lưu nguyên bản.") : null, dl));
      });
    }).catch(() => {});
  }
  return h("details", { class: "panel notice-box", id: "notice-box" },
    h("summary", null, h("h2", null, "Thông tin báo tử")),
    h("dl", { class: "facts pfacts" },
      ...row("Người báo tin", n.reported_by as string | null),
      ...row("Mất tại", n.place as string | null),
      ...row("Ngày nhận tin", nd ? dmy(M.ymd(nd.year as number, nd.month as number, nd.day as number)) : null),
      ...row("Lý do", n.reason as string | null),
      ...row("Ghi chú", n.note as string | null)),
    atts.length ? list : null);
}

/** Quy tắc nhắc giỗ mặc định khi chuyển sang người đã mất (theo cài đặt chung; tiên thường tắt — O-1). */
function defaultGioRule(settings: JsonObj): JsonObj {
  return { id: crypto.randomUUID(), enabled: true, offsets_days: (settings.default_offsets_days as number[]) ?? [0, 3], time_local: (settings.default_time_local as string) ?? "07:00", tien_thuong: false };
}

// ======================================================================================================
/** Biểu mẫu "Cập nhật: đã mất" — chuyển CÙNG hồ sơ đang sống sang người đã mất (01 §3.2a, US-M-07). */
export async function renderConvertForm(root: HTMLElement, id: string, nav: Nav): Promise<void> {
  const sn = await snap();
  const s = await store();
  const old = sn.byId.get(id);
  if (!old) { root.append(backLink(), h("p", { class: "warn" }, "Không tìm thấy hồ sơ này (có thể đã bị xóa).")); return; }
  if (old.kind !== "living") { nav.go(`#/nguoi-than/nguoi/${id}`, true); return; }
  const name = displayName(old);
  document.title = "Cập nhật: đã mất — Lịch và Văn hóa Việt";
  const h1 = h("h1", { tabindex: "-1" }, "Cập nhật: đã mất");
  const sum = errSummary();
  let dirty = false;
  const markDirty = () => { dirty = true; };

  const death = dateInput("cf-death", "Ngày mất", "death", true);
  let lunarOf: { y: number | null; text: string } | null = null;
  const refresh = (): string | null => {
    const dr = death.read();
    lunarOf = null;
    if (dr.error) { death.setResult(death.complete() ? dr.error : null, death.complete()); return dr.error; }
    if (!dr.value) { death.setResult(null); return null; }
    const p = deathPreview(dr.value);
    if (p.err) { death.setResult(p.err, true); return p.err; }
    death.setResult(p.text);
    const conv = M.convertDeath(cal0(), dr.value as M.DateValue)[0];
    lunarOf = { y: conv.year, text: `${conv.day}/${conv.month}${conv.leap ? " nhuận" : ""} âm lịch` };
    return null;
  };
  death.onChange(() => { dirty = true; refresh(); });
  const notice = noticeSection(null, true, markDirty);

  async function save(): Promise<void> {
    const issues: Issue[] = [];
    const dErr = refresh();
    const dr = death.read();
    if (dErr || !dr.value) issues.push({ msg: `Ngày mất: ${dErr ?? "Hãy nhập ngày mất"}`, focus: () => death.focus() });
    if (dErr) death.setResult(dErr, true);
    const nc = notice.collect(true, issues);
    sum.show(issues);
    if (issues.length) return;
    const now = nowTs();
    const base: JsonObj = {
      ...structuredClone(old!),
      kind: "deceased",
      death: { input: dr.value as unknown as JsonObj, ...(nc.notice ? { notice: nc.notice } : {}) },
      noi_an_tang: "",
      memorial: { tray_style: "truyen_thong", incense: true, bia_tieu_de: "", loi_tuong_niem: "", gio_het_nam: 2, huong_duong_duoi: 60 },
      reminders: { gio: defaultGioRule(sn.settings), sinh_nhat: null, sinh_nhat_am: null },
      sinh_nhat_am: false,
      updated_at: now, rev: (old!.rev as number) + 1,
    };
    let p: JsonObj;
    try { p = buildPerson(normalizePersonNames(base)); validate.person(p, "/person"); }
    catch (e) { sum.show([{ msg: `Chưa lưu được: ${(e as Error).message}`, focus: () => death.focus() }]); return; }
    const evN = sn.events.filter((e) => (e.person_ids as string[]).includes(id)).length;
    const ok = await confirmDialog({
      title: `Cập nhật ${name} là người đã mất?`, ok: "Lưu thay đổi", cancel: "Quay lại", cls: "cv-dialog",
      body: [
        h("ul", { class: "cv-list" },
          h("li", null, `Sẽ không còn nhắc sinh nhật${evN ? " và sự kiện" : ""} của ${name}.`),
          h("li", null, `Bắt đầu nhắc ngày giỗ hằng năm vào ${lunarOf ? lunarOf.text : "ngày mất theo âm lịch"} (${ruleSummary((p.reminders as JsonObj).gio as JsonObj).toLowerCase()}).`),
          h("li", null, "Họ tên, ảnh, ngày sinh và sự kiện vẫn được giữ trong hồ sơ."),
          nc.notice ? h("li", null, "Thông tin báo tử chỉ hiện trên trang hồ sơ.") : null),
      ],
      onOk: async () => {
        try { await s.putPerson(p, undefined, nc.newAttachments); } catch (e) { return `Chưa lưu được: ${(e as Error).message}`; }
        return true;
      },
    });
    if (!ok) return;
    dirty = false;
    toast("Đã cập nhật hồ sơ.");
    nav.go(`#/nguoi-than/nguoi/${id}`, true);
  }

  const form = h("form", { class: "pform", novalidate: true, onsubmit: (e: Event) => { e.preventDefault(); void save(); } },
    sum.el,
    h("p", { class: "cv-intro" }, `Ghi lại ngày mất của ${name}. Hồ sơ, ảnh và ngày sinh được giữ nguyên; ứng dụng sẽ nhắc ngày giỗ hằng năm.`),
    death.el,
    notice.el,
    h("div", { class: "form-acts" },
      h("button", { type: "button", class: "btn2", onclick: async () => {
        if (!dirty || await confirmDialog({ title: "Bỏ các thay đổi chưa lưu?", body: ["Những gì vừa nhập sẽ không được lưu."], ok: "Bỏ thay đổi", cancel: "Tiếp tục sửa", focusCancel: true })) {
          dirty = false; nav.go(`#/nguoi-than/nguoi/${id}`, true);
        }
      } }, "Hủy"),
      h("button", { type: "submit", class: "primary" }, "Lưu")));
  form.addEventListener("input", markDirty);
  root.append(backLink("‹ Về hồ sơ", `#/nguoi-than/nguoi/${id}`), h("div", { class: "pagehead" }, h1, h("p", null, name)), form);
  h1.focus();
}
