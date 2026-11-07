// Biểu mẫu: thêm/sửa người đã mất (08 §3.4), người đang sống (08 §3.5), sự kiện (08 §3.7).
import { append, clear, h } from "../dom";
import { buildPerson, capitalizeFirst, danhXungSuggestions, normalizePersonNames, quanHeSuggestions, validate } from "../store/index";
import { M, cal, clean, cpLen, displayName, lunarText, nowTs, snap, store, today, tuoiText, uuid, type JsonObj, type Ymd } from "./core";
import { drawPhoto, processPhoto, type Processed } from "./photo";
import { noticeSection } from "./notice";
import { backLink, confirmDialog, dateInput, pressGroup, ruleInput, solarLong, toast, type DateInput, type DateValueJ } from "./ui";

export interface Nav { go(hash: string, replace?: boolean): void }


export interface TextF { el: HTMLElement; input: HTMLInputElement | HTMLTextAreaElement; read(): { v: string; err: string | null }; showErr(m: string | null): void }

export function textField(id: string, label: string, value: string, max: number, o: { required?: boolean; multiline?: boolean; list?: string; hint?: string; what?: string } = {}): TextF {
  const input = o.multiline
    ? h("textarea", { id, rows: "3", class: "tin" })
    : h("input", { type: "text", id, class: "tin", autocomplete: "off", list: o.list ?? null, placeholder: o.hint ?? null });
  (input as HTMLInputElement).value = value;
  const err = h("p", { class: "ferr", id: `${id}-err`, role: "alert", hidden: true });
  const el = h("div", { class: "field" }, h("label", { for: id }, label + (o.required ? " *" : "")), input, err);
  const showErr = (m: string | null) => {
    err.hidden = !m; err.textContent = m ?? "";
    if (m) { input.setAttribute("aria-invalid", "true"); input.setAttribute("aria-describedby", err.id); } else { input.removeAttribute("aria-invalid"); input.removeAttribute("aria-describedby"); }
  };
  const read = () => {
    const v = clean(input.value, !!o.multiline);
    const what = o.what ?? label;
    if (o.required && v === "") return { v, err: `${what} không được để trống` };
    if (cpLen(v) > max) return { v, err: `${what} tối đa ${max} ký tự` };
    return { v, err: null };
  };
  // TK-03 (08 §3.4): báo lỗi khi rời ô; khi đang có lỗi thì kiểm lại theo từng lần gõ để lỗi tự mất khi đã sửa.
  input.addEventListener("blur", () => showErr(read().err));
  input.addEventListener("input", () => { if (!err.hidden) showErr(read().err); });
  return { el, input, showErr, read };
}

export interface Issue { msg: string; focus: () => void }
/** Khung tóm tắt lỗi. `show(list, false)`: cập nhật khi người dùng sửa (không chuyển tiêu điểm); rỗng thì ẩn. */
export function errSummary(): { el: HTMLElement; show(list: Issue[], focus?: boolean): void; visible(): boolean } {
  const el = h("div", { class: "errsum", role: "alert", tabindex: "-1", hidden: true });
  return {
    el,
    visible: () => !el.hidden,
    show(list, focus = true) {
      clear(el);
      el.hidden = list.length === 0;
      if (!list.length) return;
      el.append(h("p", null, `Còn ${list.length} chỗ cần sửa:`), h("ul", null, ...list.map((i) => h("li", null, h("button", { type: "button", class: "linkbtn", onclick: i.focus }, i.msg)))));
      if (focus) el.focus();
    },
  };
}

const after = (a: Ymd, b: Ymd) => M.cmpYmd(a, b) > 0;

/** Kết quả trực tiếp của ngày mất (02 §1): giá trị sẽ lưu. Trả [chữ, lỗi]. */
export function deathPreview(v: DateValueJ): { text: string | null; err: string | null; lunarYear: number | null } {
  let r;
  try { r = M.convertDeath(cal(), v as M.DateValue); } catch (e) { return { text: null, err: (e as Error).message, lunarYear: null }; }
  const [lunar, solar] = r;
  if (solar && after(solar, today())) return { text: null, err: "Ngày mất không được sau hôm nay", lunarYear: null };
  if (v.calendar === "solar") return { text: `Âm lịch: ${lunarText(lunar, true, false)}`, err: null, lunarYear: lunar.year };
  if (v.precision === "month_day") return { text: `Không rõ năm: vẫn nhắc giỗ hằng năm vào ${lunar.day}/${lunar.month} âm lịch, không ghi giỗ đầu, giỗ hết.`, err: null, lunarYear: null };
  if (!solar) return { text: "Năm này ngoài khoảng đổi lịch — không có ngày dương tương ứng", err: null, lunarYear: lunar.year };
  return { text: `Dương lịch: ${solarLong(solar)}`, err: null, lunarYear: lunar.year };
}

function birthPreview(v: DateValueJ, living: boolean): { text: string | null; err: string | null; b: M.BirthInfo | null } {
  let b: M.BirthInfo;
  try { b = M.deriveBirth(cal(), v as M.DateValue); } catch (e) { return { text: null, err: (e as Error).message, b: null }; }
  const t = today();
  if (b.solar && after(b.solar, t)) return { text: null, err: "Ngày sinh không được sau hôm nay", b: null };
  if (v.precision === "year" && v.calendar === "solar" && v.year !== null && v.year > t.y) return { text: null, err: "Ngày sinh không được sau hôm nay", b: null };
  if (b.lunar_year === null) return { text: null, err: null, b };
  let s = b.lunar && v.precision === "day" ? `Âm lịch: ${lunarText(b.lunar, true, false)}` : b.lunar_year_approx
    ? `Khoảng năm âm ${b.lunar_year} — ${b.can_chi} (nếu sinh trước Tết thì là năm trước)`
    : `Năm âm ${b.lunar_year} — ${b.can_chi}`;
  if (v.calendar === "lunar" && b.solar) s += ` · Dương lịch: ${solarLong(b.solar)}`;
  if (living) {
    const ages: string[] = [];
    if (b.solar) { const a = M.tuoiDuong(b.solar, t, "feb28"); if (a !== null) ages.push(`${a} tuổi`); }
    const am = M.tuoiAm(cal(), b.lunar_year, t);
    if (am !== null) ages.push(ages.length ? `tuổi âm ${am}` : `${am} tuổi âm`);
    if (ages.length) s += ` · Hôm nay ${ages[0]}${ages[1] ? ` (${ages[1]})` : ""}`;
  }
  return { text: s, err: null, b };
}

// ======================================================================================================
export async function renderPersonForm(root: HTMLElement, kindIn: "deceased" | "living" | null, id: string | null, nav: Nav): Promise<void> {
  const sn = await snap();
  const s = await store();
  const old = id ? sn.byId.get(id) ?? null : null;
  if (id && !old) { root.append(backLink(), h("p", { class: "warn" }, "Không tìm thấy hồ sơ này (có thể đã bị xóa).")); return; }
  const kind = (old?.kind as "deceased" | "living" | undefined) ?? kindIn ?? "deceased";
  const dead = kind === "deceased";
  const mem = (old?.memorial as JsonObj | null) ?? { tray_style: "truyen_thong", incense: true, bia_tieu_de: "", loi_tuong_niem: "", gio_het_nam: 2, huong_duong_duoi: 60 };
  const title = old ? "Sửa hồ sơ" : dead ? "Thêm người đã mất" : "Thêm người đang sống";
  document.title = `${title} — Lịch và Văn hóa Việt`;
  const h1 = h("h1", { tabindex: "-1" }, title);
  const sum = errSummary();
  let dirty = false;

  // YC-2026-07: gợi ý cách gọi của lõi (theo người sống/mất, xếp theo giới tính); chữ đầu viết hoa khi lưu.
  let gt: string | null = (old?.gioi_tinh as string | null | undefined) ?? null;
  const dl = h("datalist", { id: "pf-dx-list" });
  const fillDx = () => { clear(dl); for (const x of danhXungSuggestions(dead, gt)) dl.append(h("option", { value: x })); };
  fillDx();
  const fDx = textField("pf-dx", "Cách gọi (đứng trước tên)", (old?.danh_xung as string) ?? "", 40, { list: "pf-dx-list", hint: dead ? "ví dụ: Ông, Bà, Cụ, Bác" : "ví dụ: Anh, Chị, Cô, Chú", what: "Cách gọi" });
  const gtGroup = h("fieldset", { class: "field radios", id: "pf-gt" }, h("legend", null, "Giới tính"),
    ...([["nam", "Nam"], ["nu", "Nữ"], ["", "Không ghi"]] as const).map(([v, t]) => {
      const r = h("input", { type: "radio", name: "pf-gt", id: `pf-gt-${v || "none"}`, value: v, checked: (gt ?? "") === v }) as HTMLInputElement;
      r.addEventListener("change", () => { if (r.checked) { gt = v || null; fillDx(); } });
      return h("label", { class: "check", for: `pf-gt-${v || "none"}` }, r, h("span", null, t));
    }));
  const fTen = textField("pf-ten", "Họ và tên", (old?.ho_ten as string) ?? "", 120, { required: true, what: "Họ tên" });
  const fHieu = textField("pf-hieu", "Tên hiệu / pháp danh", (old?.ten_hieu as string) ?? "", 120, { what: "Tên hiệu" });
  const fQh = textField("pf-qh", "Quan hệ với bạn", (old?.quan_he as string) ?? "", 120, { list: "pf-qh-list", hint: dead ? "ví dụ: Ông nội, Bà ngoại, Bố" : "ví dụ: Con trai, Em gái", what: "Quan hệ" });
  // YC-2026-07: viết hoa chữ cái đầu ngay khi rời ô (hàm của lõi; lưu cũng chuẩn hóa như vậy).
  for (const f of [fDx, fQh]) f.input.addEventListener("blur", () => { const v = capitalizeFirst(f.input.value); if (v !== f.input.value) f.input.value = v; });
  const qhList = h("datalist", { id: "pf-qh-list" }, ...quanHeSuggestions(dead).map((x) => h("option", { value: x })));
  const fNote = textField("pf-note", "Ghi chú", (old?.ghi_chu as string) ?? "", 2000, { multiline: true });

  // ---- ngày mất ----
  let death: DateInput | null = null;
  const tuoiLine = h("p", { class: "dres tuoi", id: "pf-tuoi", "aria-live": "polite" });
  const oldTuoi = (old?.death as JsonObj | null)?.tuoi as JsonObj | undefined;
  let manual = oldTuoi?.manual === true ? (oldTuoi.value as number | null) : null;
  const ageIn = h("input", { type: "text", inputmode: "numeric", id: "pf-age", class: "num sm", maxlength: "3", value: manual !== null ? String(manual) : "" }) as HTMLInputElement;
  const ageErr = h("p", { class: "ferr", role: "alert", hidden: true });
  const ageBox = h("div", { class: "agebox", hidden: manual === null },
    h("label", { class: "field inline", for: "pf-age" }, h("span", null, "Tuổi (tự ghi)"), ageIn),
    h("button", { type: "button", class: "linkbtn", onclick: () => { manual = null; ageIn.value = ""; ageBox.hidden = true; editAge.hidden = false; refresh(); } }, "Tính lại"), ageErr);
  const editAge = h("button", { type: "button", class: "linkbtn", hidden: manual !== null, onclick: () => { ageBox.hidden = false; editAge.hidden = true; ageIn.focus(); } }, "Sửa tuổi");
  ageIn.addEventListener("input", () => { manual = /^\d{1,3}$/.test(ageIn.value.trim()) ? Number(ageIn.value.trim()) : null; refresh(); });

  // ---- ngày sinh ----
  const birth = dateInput("pf-birth", "Ngày sinh", "birth", false, true);
  const fNoiSinh = textField("pf-noisinh", "Nơi sinh", ((old?.birth as JsonObj | null)?.noi_sinh as string) ?? "", 300);
  const fAnTang = textField("pf-antang", "Nơi an táng", (old?.noi_an_tang as string) ?? "", 300);

  // ---- người đang sống: sinh nhật âm, 29/02 ----
  const snAm = h("input", { type: "checkbox", id: "pf-snam", checked: old?.sinh_nhat_am === true }) as HTMLInputElement;
  const snAmNote = h("span", { class: "fine inline" });
  const snAmWrap = h("label", { class: "check", for: "pf-snam" }, snAm, h("span", null, "Nhắc cả sinh nhật âm lịch hằng năm"), snAmNote);
  let feb = (old?.feb29_rule as string) ?? "feb28";
  const febGroup = pressGroup("Sinh 29/02", [["feb28", "Năm không nhuận: mừng 28/02"], ["mar1", "Năm không nhuận: mừng 01/03"]], feb, (v) => { feb = v; dirty = true; }, "pseg two");
  const febWrap = h("div", { class: "field", hidden: true }, h("p", { class: "flabel" }, "Sinh ngày 29/02"), febGroup);

  // ---- ảnh ----
  const photo = { pending: null as Processed | null, removed: false, oldId: (old?.photo_id as string | null) ?? null };
  const canvas = h("canvas", { class: "pf-photo", width: "160", height: "200", "aria-label": "Ảnh chân dung" }) as HTMLCanvasElement;
  const photoErr = h("p", { class: "ferr", role: "alert", hidden: true });
  const file = h("input", { type: "file", id: "pf-file", accept: "image/*", class: "sr-only", tabindex: "-1", "aria-hidden": "true" }) as HTMLInputElement;
  // Nút thật (bàn phím bấm được) mở hộp chọn tệp; ô chọn tệp gốc ẩn hẳn (TK-01, TK-06).
  const pickLbl = h("button", { type: "button", class: "btn2", id: "pf-pick", onclick: () => file.click() }, photo.oldId ? "Đổi ảnh" : "Chọn ảnh");
  const rmBtn = h("button", { type: "button", class: "btn2", hidden: !photo.oldId, onclick: () => {
    photo.pending = null; photo.removed = true; dirty = true;
    canvas.getContext("2d")!.clearRect(0, 0, canvas.width, canvas.height); delete canvas.dataset.drawn;
    rmBtn.hidden = true; pickLbl.textContent = "Chọn ảnh";
  } }, "Bỏ ảnh");
  file.addEventListener("change", async () => {
    const f = file.files?.[0];
    file.value = "";
    if (!f) return;
    photoErr.hidden = true;
    try {
      photo.pending = await processPhoto(f);
      photo.removed = false; dirty = true;
      await drawPhoto(canvas, photo.pending.bytes);
      rmBtn.hidden = false; pickLbl.textContent = "Đổi ảnh";
    } catch (e) { photoErr.textContent = (e as Error).message; photoErr.hidden = false; }
  });
  if (photo.oldId) s.photoBytes(photo.oldId).then((b) => drawPhoto(canvas, b)).catch(() => {});

  // ---- nhắc ----
  const reminders = (old?.reminders as JsonObj | undefined) ?? {};
  const rGio = dead ? ruleInput("pf-rgio", "Nhắc ngày giỗ", (reminders.gio as JsonObj | null) ?? null, true, sn.settings) : null;
  const rSn = !dead ? ruleInput("pf-rsn", "Nhắc sinh nhật", (reminders.sinh_nhat as JsonObj | null) ?? null, false, sn.settings) : null;
  const rSnAm = !dead ? ruleInput("pf-rsna", "Nhắc sinh nhật âm lịch", (reminders.sinh_nhat_am as JsonObj | null) ?? null, false, sn.settings) : null;
  // TK-29 (08 §12e, chờ chủ dự án xác nhận): mặc định nhắc sinh nhật âm lịch GIỐNG sinh nhật dương lịch — khi lưu,
  // chép cài đặt của quy tắc `sinh_nhat` sang `sinh_nhat_am` (giữ `id` riêng của nó). Không đổi mô hình dữ liệu 01.
  const sameRule = (a: JsonObj | null | undefined, b: JsonObj | null | undefined) => !a || !b
    || (a.enabled === b.enabled && a.time_local === b.time_local && JSON.stringify(a.offsets_days) === JSON.stringify(b.offsets_days));
  const snAmSame = h("input", { type: "checkbox", id: "pf-snam-same", checked: sameRule(reminders.sinh_nhat_am as JsonObj | null, reminders.sinh_nhat as JsonObj | null) }) as HTMLInputElement;
  const snAmSameWrap = h("label", { class: "check sub", for: "pf-snam-same" }, snAmSame, h("span", null, "Nhắc sinh nhật âm lịch giống nhắc sinh nhật dương lịch"));
  const snAmId = ((reminders.sinh_nhat_am as JsonObj | null)?.id as string) ?? crypto.randomUUID();

  // ---- thông tin báo tử (01 §3.2a) — chỉ người đã mất; thu gọn khi thêm mới ----
  const oldNotice = ((old?.death as JsonObj | null)?.notice as JsonObj | null | undefined) ?? null;
  const notice = dead ? noticeSection(oldNotice, false, () => { dirty = true; }) : null;

  // ---- tưởng niệm ----
  let gioHet = String(mem.gio_het_nam ?? 2);
  const fLoi = textField("pf-loi", "Lời tưởng niệm", (mem.loi_tuong_niem as string) ?? "", 2000, { multiline: true });
  const nguongIn = h("input", { type: "text", inputmode: "numeric", id: "pf-nguong", class: "num sm", maxlength: "3", value: String(mem.huong_duong_duoi ?? 60) }) as HTMLInputElement;
  const nguongErr = h("p", { class: "ferr", role: "alert", hidden: true });
  nguongIn.addEventListener("input", () => refresh());
  const nguong = () => { const n = /^\d{1,3}$/.test(nguongIn.value.trim()) ? Number(nguongIn.value.trim()) : NaN; return n >= 1 && n <= 150 ? n : null; };

  // ---- cập nhật trực tiếp ----
  function refresh(): { deathErr: string | null; birthErr: string | null; ageErr: string | null } {
    let deathErr: string | null = null, birthErr: string | null = null, aErr: string | null = null;
    const br = birth.read();
    let bInfo: M.BirthInfo | null = null;
    if (br.error) { birthErr = br.error; birth.setResult(birth.complete() || !br.error.startsWith("Hãy") ? br.error : null, birth.complete()); }
    else if (br.value) {
      const p = birthPreview(br.value, !dead);
      if (p.err) { birthErr = p.err; birth.setResult(p.err, true); } else { birth.setResult(p.text); bInfo = p.b; }
    } else birth.setResult(null);
    if (!dead) {
      const full = !!br.value && br.value.precision === "day" && !birthErr;
      snAm.disabled = !full;
      snAmNote.textContent = full ? "" : " (cần biết đủ ngày sinh)";
      if (!full) snAm.checked = false;
      snAmSameWrap.hidden = !snAm.checked;
      rSnAm!.el.hidden = !snAm.checked || snAmSame.checked;
      febWrap.hidden = !(br.value && br.value.calendar === "solar" && br.value.month === 2 && br.value.day === 29);
    }
    if (death) {
      const dr = death.read();
      if (dr.error) { deathErr = dr.error; death.setResult(death.complete() ? dr.error : null, death.complete()); }
      else if (dr.value) {
        const p = deathPreview(dr.value);
        if (p.err) { deathErr = p.err; death.setResult(p.err, true); }
        else {
          death.setResult(p.text);
          const ng = nguong();
          nguongErr.hidden = ng !== null; nguongErr.textContent = ng === null ? "Ngưỡng phải từ 1 đến 150" : "";
          if (manual !== null || !ageBox.hidden) {
            if (manual === null || manual < 1 || manual > 150) { aErr = "Tuổi phải từ 1 đến 150"; }
            tuoiLine.textContent = manual !== null ? `${manual < (ng ?? 60) ? "Hưởng dương" : "Hưởng thọ"} ${manual} tuổi (tự ghi)` : "";
          } else {
            try {
              const t = M.tuoiMat(bInfo, p.lunarYear, ng ?? 60);
              tuoiLine.textContent = tuoiText(t as unknown as JsonObj) ?? (bInfo ? "Chưa đủ năm để tính tuổi" : "Chưa đủ năm sinh để tính tuổi");
            } catch (e) {
              const code = (e as M.InputError).code;
              deathErr = code === "AgeTooLarge" ? "Tuổi tính ra lớn hơn 150 — hãy kiểm tra năm sinh và năm mất" : (e as Error).message;
              death.setResult(deathErr, true);
              tuoiLine.textContent = "";
            }
          }
        }
      } else { death.setResult(null); tuoiLine.textContent = ""; }
      // "Sửa tuổi" chỉ có nghĩa khi đã có ngày mất hợp lệ.
      if (ageBox.hidden) editAge.hidden = !dr.value || !!deathErr;
      ageErr.hidden = !aErr; ageErr.textContent = aErr ?? "";
    }
    return { deathErr, birthErr, ageErr: aErr };
  }

  if (dead) {
    death = dateInput("pf-death", "Ngày mất", "death", true);
    death.set(((old?.death as JsonObj | null)?.input as unknown as DateValueJ) ?? null);
    death.onChange(() => { dirty = true; refresh(); });
    death.el.append(h("div", { class: "tuoi-row" }, tuoiLine, editAge, ageBox));
  }
  birth.set(((old?.birth as JsonObj | null)?.input as unknown as DateValueJ) ?? null);
  birth.onChange(() => { dirty = true; refresh(); });
  snAm.addEventListener("change", () => refresh());
  snAmSame.addEventListener("change", () => { dirty = true; refresh(); });

  // ---- kiểm toàn biểu mẫu (dùng khi Lưu, và khi đang có khung tóm tắt lỗi thì sau mỗi lần sửa — TK-03) ----
  function collect(show: boolean) {
    const issues: Issue[] = [];
    const tf = (f: TextF) => { const r = f.read(); if (show) f.showErr(r.err); if (r.err) issues.push({ msg: r.err, focus: () => f.input.focus() }); return r.v; };
    const danh_xung = tf(fDx), ho_ten = tf(fTen), ten_hieu = dead ? tf(fHieu) : "", quan_he = tf(fQh), ghi_chu = tf(fNote);
    const noi_sinh = tf(fNoiSinh), noi_an_tang = dead ? tf(fAnTang) : "";
    // bia_tieu_de (01) không còn trên giao diện: giữ nguyên giá trị đang lưu để sao lưu/khôi phục không mất dữ liệu.
    const bia = dead ? ((mem.bia_tieu_de as string) ?? "") : "", loi = dead ? tf(fLoi) : "";
    const st = refresh();
    const br = birth.read();
    if (st.birthErr) { if (show) birth.setResult(st.birthErr, true); issues.push({ msg: `Ngày sinh: ${st.birthErr}`, focus: () => birth.focus() }); }
    if (!br.value && noi_sinh !== "") issues.push({ msg: "Hãy nhập ngày sinh (ít nhất năm) để ghi nơi sinh", focus: () => birth.focus() });
    let dv: DateValueJ | null = null;
    if (death) {
      const dr = death.read();
      if (st.deathErr) { if (show) death.setResult(st.deathErr, true); issues.push({ msg: `Ngày mất: ${st.deathErr}`, focus: () => death!.focus() }); }
      dv = dr.value;
      if (st.ageErr) issues.push({ msg: st.ageErr, focus: () => ageIn.focus() });
      if (nguong() === null) issues.push({ msg: "Ngưỡng hưởng dương phải từ 1 đến 150", focus: () => nguongIn.focus() });
    }
    const nc = notice ? notice.collect(show, issues) : null;
    const rules = [rGio, rSn, snAm.checked && !snAmSame.checked ? rSnAm : null].filter(Boolean).map((r) => r!.read());
    rules.forEach((r) => { if (r.error) issues.push({ msg: r.error, focus: () => {} }); });
    return { issues, danh_xung, ho_ten, ten_hieu, quan_he, ghi_chu, noi_sinh, noi_an_tang, bia, loi, br, dv, rules, nc };
  }

  // ---- lưu ----
  async function save(): Promise<void> {
    const c = collect(true);
    sum.show(c.issues);
    if (c.issues.length) return;
    const { danh_xung, ho_ten, ten_hieu, quan_he, ghi_chu, noi_sinh, noi_an_tang, bia, loi, br, dv, rules, nc } = c;

    const now = nowTs();
    const pid = old ? (old.id as string) : uuid();
    let photoId: string | null = photo.removed ? null : photo.oldId;
    let newPhoto;
    if (photo.pending) { photoId = uuid(); newPhoto = { id: photoId, ...photo.pending }; }
    const base: JsonObj = {
      id: pid, kind, danh_xung, gioi_tinh: gt, ho_ten, ten_hieu, quan_he,
      birth: br.value ? { input: br.value as unknown as JsonObj, noi_sinh } : null,
      death: dead ? { input: dv as unknown as JsonObj, ...(nc?.notice ? { notice: nc.notice } : {}), ...(manual !== null ? { tuoi: { value: manual, label: manual < (nguong() ?? 60) ? "huong_duong" : "huong_tho", manual: true, approx: false } } : {}) } : null,
      noi_an_tang, photo_id: photoId,
      memorial: dead ? { tray_style: mem.tray_style ?? "truyen_thong", incense: mem.tray_style === "cong_giao" ? false : (mem.incense ?? true), bia_tieu_de: bia, loi_tuong_niem: loi, gio_het_nam: Number(gioHet), huong_duong_duoi: nguong() ?? 60 } : null,
      reminders: dead ? { gio: rules[0].rule, sinh_nhat: null, sinh_nhat_am: null } : { gio: null, sinh_nhat: rules[0].rule, sinh_nhat_am: !snAm.checked ? ((reminders.sinh_nhat_am as JsonObj | null) ?? null)
        : snAmSame.checked ? { ...(rules[0].rule as JsonObj), id: snAmId, tien_thuong: false } : rules[1].rule },
      sinh_nhat_am: !dead && snAm.checked,
      feb29_rule: dead ? ((old?.feb29_rule as string) ?? "feb28") : feb,
      ghi_chu,
      created_at: (old?.created_at as string) ?? now, updated_at: now, rev: old ? (old.rev as number) + 1 : 1,
    };
    if (base.death && manual !== null && br.value) {
      try { ((base.death as JsonObj).tuoi as JsonObj).approx = M.deriveBirth(cal(), br.value as M.DateValue).lunar_year_approx; } catch { /* lỗi ngày sinh đã báo ở trên */ }
    }
    let p: JsonObj;
    try { p = buildPerson(normalizePersonNames(base)); validate.person(p, "/person"); }
    catch (e) { sum.show([{ msg: `Chưa lưu được: ${(e as Error).message}`, focus: () => fTen.input.focus() }]); return; }
    try {
      // Ảnh / tệp đính kèm bản cũ dùng mà bản mới bỏ: lớp lưu trữ tự xóa trong cùng giao dịch.
      await s.putPerson(p, newPhoto, nc?.newAttachments ?? []);
    } catch (e) { sum.show([{ msg: `Chưa lưu được: ${(e as Error).message}. Dữ liệu đang nhập vẫn còn trên màn hình.`, focus: () => fTen.input.focus() }]); return; }
    dirty = false;
    toast("Đã lưu hồ sơ.");
    nav.go(`#/nguoi-than/nguoi/${pid}`, true);
  }

  const form = h("form", { class: "pform", novalidate: true, onsubmit: (e: Event) => { e.preventDefault(); void save(); } });
  form.addEventListener("input", () => { dirty = true; if (sum.visible()) sum.show(collect(false).issues, false); });
  form.addEventListener("change", () => { if (sum.visible()) sum.show(collect(false).issues, false); });
  append(form, [
    sum.el,
    h("fieldset", { class: "pfs" }, h("legend", null, dead ? "Người đã mất" : "Người thân"),
      fDx.el, dl, gtGroup, fTen.el, dead ? fHieu.el : null, fQh.el, qhList, dead ? h("p", { class: "fine qh-note" }, "Trang tưởng niệm ghi quan hệ này phía trên tên, ví dụ “Ông nội”, “Mẹ”. Để trống thì chỉ ghi tên.") : null, fNote.el),
    death ? death.el : null,
    h("fieldset", { class: "pfs" }, h("legend", null, "Ngày sinh và nơi sinh"), birth.el, fNoiSinh.el, !dead ? febWrap : null, !dead ? snAmWrap : null, !dead ? snAmSameWrap : null),
    dead ? h("fieldset", { class: "pfs" }, h("legend", null, "Nơi an táng và ảnh"), fAnTang.el,
      h("div", { class: "photo-row" }, canvas, h("div", { class: "photo-acts" }, file, pickLbl, rmBtn, photoErr,
        h("p", { class: "fine" }, "Ảnh chỉ lưu trong ứng dụng trên máy này, đã bỏ thông tin vị trí và máy chụp.")))) : null,
    notice?.el ?? null,
    rGio?.el ?? null, rSn?.el ?? null, rSnAm?.el ?? null,
    dead ? h("details", { class: "pfs more", id: "pf-mem", open: !!old && (mem.loi_tuong_niem as string) !== "" },
      h("summary", null, "Tưởng niệm (giỗ hết, lời tưởng niệm)"),
      h("div", { class: "field" }, h("p", { class: "flabel" }, "Giỗ hết (giỗ lần cuối của thời gian để tang) vào"),
        pressGroup("Giỗ hết vào", [["2", "Năm thứ 2 sau năm mất"], ["3", "Năm thứ 3 sau năm mất"]], gioHet, (v) => { gioHet = v; dirty = true; }, "pseg two")),
      fLoi.el,
      h("label", { class: "field inline", for: "pf-nguong" }, h("span", null, "Mất trước tuổi này thì bia ghi “hưởng dương” (mất sớm) thay cho “hưởng thọ”"), nguongIn), nguongErr) : null,
    h("div", { class: "form-acts" },
      h("button", { type: "button", class: "btn2", onclick: async () => {
        if (!dirty || await confirmDialog({ title: "Bỏ các thay đổi chưa lưu?", body: ["Những gì vừa nhập sẽ không được lưu."], ok: "Bỏ thay đổi", cancel: "Tiếp tục sửa", focusCancel: true })) {
          dirty = false; nav.go(old ? `#/nguoi-than/nguoi/${old.id as string}` : "#/nguoi-than/ho-so", true);
        }
      } }, "Hủy"),
      h("button", { type: "submit", class: "primary" }, "Lưu")),
  ]);
  root.append(old ? backLink("‹ Về hồ sơ", `#/nguoi-than/nguoi/${old.id as string}`) : backLink(), h("div", { class: "pagehead" }, h1, old ? h("p", null, displayName(old)) : null), form);
  refresh();
  h1.focus();
}

// ======================================================================================================
export async function renderEventForm(root: HTMLElement, personId: string, eventId: string | null, nav: Nav): Promise<void> {
  const sn = await snap();
  const s = await store();
  const person = sn.byId.get(personId);
  const old = eventId ? sn.events.find((e) => e.id === eventId) ?? null : null;
  if (!person || (eventId && !old)) { root.append(backLink(), h("p", { class: "warn" }, "Không tìm thấy hồ sơ hoặc sự kiện này.")); return; }
  const title = old ? "Sửa sự kiện" : "Thêm sự kiện";
  document.title = `${title} — Lịch và Văn hóa Việt`;
  const h1 = h("h1", { tabindex: "-1" }, title);
  const sum = errSummary();
  // Loại có sẵn theo thứ tự 01 §3.5 (id cố định tăng dần), rồi loại tự tạo theo tên.
  const types = sn.types.slice().sort((a, b) => (a.builtin ? 0 : 1) - (b.builtin ? 0 : 1) || (a.builtin ? ((a.id as string) < (b.id as string) ? -1 : 1) : (a.ten as string).localeCompare(b.ten as string, "vi")));
  const sel = h("select", { id: "ef-type" }, ...types.map((t) => h("option", { value: t.id as string }, t.ten as string)), h("option", { value: "__new" }, "Loại khác…")) as HTMLSelectElement;
  sel.value = (old?.type_id as string) ?? types[0].id as string;
  const fNewType = textField("ef-newtype", "Tên loại mới", "", 60, { what: "Tên loại sự kiện" });
  const fTitle = textField("ef-title", "Tiêu đề", (old?.tieu_de as string) ?? "", 120, { hint: "để trống sẽ dùng tên loại" });
  const fNote = textField("ef-note", "Ghi chú", (old?.ghi_chu as string) ?? "", 2000, { multiline: true });
  const date = dateInput("ef-date", "Ngày", "event", true);
  const rec = h("input", { type: "checkbox", id: "ef-rec", checked: old ? old.recurring === true : types[0].default_recurring === true }) as HTMLInputElement;
  const rule = ruleInput("ef-rule", "Nhắc sự kiện này", (old?.reminder as JsonObj | null) ?? null, false, sn.settings);
  if (old && old.reminder === null) (rule.el.querySelector("#ef-rule-on") as HTMLInputElement).checked = false;
  // Bổ sung B4: sự kiện gắn với người đã mất không được nhắc — không hiện khối nhắc; quy tắc đang lưu (nếu có) giữ nguyên.
  const deadEv = ((old?.person_ids as string[]) ?? [personId]).some((id) => sn.dead.has(id));
  const b4Note = h("p", { class: "notice", id: "ef-b4" }, "Không nhắc sự kiện của người đã mất — với người đã mất, ứng dụng chỉ nhắc ngày giỗ. Sự kiện vẫn được lưu và hiện trên hồ sơ.");
  const syncType = () => {
    fNewType.el.hidden = sel.value !== "__new";
    if (!old) { const t = types.find((x) => x.id === sel.value); if (t) rec.checked = t.default_recurring === true; }
    date.setRecurring(rec.checked);
  };
  sel.addEventListener("change", syncType);
  rec.addEventListener("change", () => date.setRecurring(rec.checked));
  date.set((old?.date as unknown as DateValueJ) ?? null);
  date.onChange(() => {
    const r = date.read();
    if (r.error) date.setResult(date.complete() ? r.error : null, date.complete());
    else if (r.value && r.value.precision === "day") {
      if (r.value.calendar === "lunar") { const so = cal().l2s(r.value.year!, r.value.month!, r.value.day!, r.value.leap); date.setResult(so ? `Dương lịch: ${solarLong(so)}` : null); }
      else { try { date.setResult(`Âm lịch: ${lunarText(M.convertDeath(cal(), r.value as M.DateValue)[0], true, false)}`); } catch { date.setResult(null); } }
    } else if (r.value) date.setResult(`Hằng năm vào ${r.value.day}/${r.value.month} âm lịch`);
    else date.setResult(null);
  });
  syncType();

  let dirty = false;
  function collect(show: boolean) {
    const issues: Issue[] = [];
    const tf = (f: TextF) => { const r = f.read(); if (show) f.showErr(r.err); if (r.err) issues.push({ msg: r.err, focus: () => f.input.focus() }); return r.v; };
    const tieu_de = tf(fTitle), ghi_chu = tf(fNote);
    let newName = "";
    if (sel.value === "__new") { newName = tf(fNewType); if (!newName && !issues.length) { if (show) fNewType.showErr("Hãy đặt tên cho loại sự kiện"); issues.push({ msg: "Hãy đặt tên cho loại sự kiện", focus: () => fNewType.input.focus() }); } }
    const dr = date.read();
    if (dr.error) { if (show) date.setResult(dr.error, true); issues.push({ msg: `Ngày: ${dr.error}`, focus: () => date.focus() }); }
    if (dr.value?.precision === "month_day" && !rec.checked) issues.push({ msg: "Sự kiện không rõ năm phải lặp hằng năm", focus: () => rec.focus() });
    const rr = deadEv ? { rule: (old?.reminder as JsonObj | null) ?? null, error: null, warn: null } : rule.read();
    if (rr.error) issues.push({ msg: rr.error, focus: () => {} });
    return { issues, tieu_de, ghi_chu, newName, dr, rr };
  }

  async function save(): Promise<void> {
    const c = collect(true);
    sum.show(c.issues);
    if (c.issues.length) return;
    const { tieu_de, ghi_chu, newName, dr, rr } = c;
    const now = nowTs();
    let typeId = sel.value;
    try {
      if (typeId === "__new") {
        const key = newName.normalize("NFC").toLowerCase();
        const same = sn.types.find((t) => (t.ten as string).normalize("NFC").toLowerCase() === key);
        if (same) typeId = same.id as string;
        else { typeId = uuid(); await s.putEventType({ id: typeId, builtin: null, ten: newName, default_recurring: rec.checked, created_at: now, updated_at: now, rev: 1 }); }
      }
      const ev: JsonObj = {
        id: (old?.id as string) ?? uuid(), type_id: typeId, tieu_de, date: dr.value as unknown as JsonObj, recurring: rec.checked,
        person_ids: (old?.person_ids as string[]) ?? [personId], ghi_chu, reminder: deadEv ? rr.rule : rr.rule && rr.rule.enabled ? rr.rule : null,
        created_at: (old?.created_at as string) ?? now, updated_at: now, rev: old ? (old.rev as number) + 1 : 1,
      };
      await s.putEvent(ev);
    } catch (e) { sum.show([{ msg: `Chưa lưu được: ${(e as Error).message}`, focus: () => sel.focus() }]); return; }
    dirty = false;
    toast("Đã lưu sự kiện.");
    nav.go(`#/nguoi-than/nguoi/${personId}`, true);
  }

  // TK-17: hộp "Lặp lại hằng năm" nằm trong khung "Ngày", ngay dưới ba ô ngày (nó điều khiển ô "Không rõ năm").
  const recLabel = h("label", { class: "check", for: "ef-rec" }, rec, h("span", null, "Lặp lại hằng năm"));
  date.el.insertBefore(recLabel, date.el.querySelector(".date-opts"));
  const form = h("form", { class: "pform", novalidate: true, onsubmit: (e: Event) => { e.preventDefault(); void save(); } },
    sum.el,
    h("fieldset", { class: "pfs" }, h("legend", null, "Sự kiện"),
      h("div", { class: "field" }, h("label", { for: "ef-type" }, "Loại *"), sel), fNewType.el, fTitle.el, fNote.el),
    date.el,
    deadEv ? b4Note : rule.el,
    h("div", { class: "form-acts" },
      // TK-09: cùng hộp xác nhận như biểu mẫu người khi đã sửa.
      h("button", { type: "button", class: "btn2", onclick: async () => {
        if (!dirty || await confirmDialog({ title: "Bỏ các thay đổi chưa lưu?", body: ["Những gì vừa nhập sẽ không được lưu."], ok: "Bỏ thay đổi", cancel: "Tiếp tục sửa", focusCancel: true })) {
          dirty = false; nav.go(`#/nguoi-than/nguoi/${personId}`, true);
        }
      } }, "Hủy"),
      h("button", { type: "submit", class: "primary" }, "Lưu")));
  form.addEventListener("input", () => { dirty = true; if (sum.visible()) sum.show(collect(false).issues, false); });
  form.addEventListener("change", () => { dirty = true; if (sum.visible()) sum.show(collect(false).issues, false); });
  root.append(backLink("‹ Về hồ sơ", `#/nguoi-than/nguoi/${personId}`), h("div", { class: "pagehead" }, h1, h("p", null, displayName(person))), form);
  h1.focus();
}
