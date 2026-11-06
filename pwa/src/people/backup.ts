// Sao lưu và lịch điện thoại (08 §3.8): xuất .ics (02 §8, S10), xuất/nhập sao lưu mã hóa (03, S10), nhắc sao lưu.
// Mật mã nằm trong mô-đun tải lười của lớp lưu trữ (store/backup-crypto, chỉ nạp khi xuất/nhập — P6).
import { clear, h } from "../dom";
import { APP_VERSION } from "../gen/content";
import { M, cal, dmy, nowTs, snap, store, today, type JsonObj } from "./core";
import { confirmDialog, toast } from "./ui";
import { openBackup, type VerifiedBackup } from "../store/index";

const iso = (d: M.Ymd) => M.iso(d);

/** Tải một tệp qua thẻ <a download> (không gửi mạng). blob: chỉ dùng cho điều hướng tải xuống, không cho ảnh. */
export function download(name: string, bytes: Uint8Array | string, type: string): void {
  const blob = new Blob([bytes as BlobPart], { type });
  const url = URL.createObjectURL(blob);
  const a = h("a", { href: url, download: name, hidden: true });
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 10_000);
}

// ---------------- .ics ----------------
export async function openIcsDialog(): Promise<void> {
  const sn = await snap();
  const t = today();
  const opt: M.IcsOptions = { today: t, dtstamp: nowTs().replace(/[-:]/g, ""), only_ids: null };
  const occ = M.icsOccurrences(cal(), sn.ds, opt);
  if (!occ.length) {
    await confirmDialog({ title: "Xuất tệp lịch", body: [h("p", null, "Chưa có hồ sơ hay sự kiện nào bật nhắc, nên chưa có gì để xuất.")], ok: "Đã hiểu", cancel: "Đóng" });
    return;
  }
  const people = new Set<string>();
  for (const [o] of occ) {
    if (o.class.startsWith("su_kien")) { const e = sn.events.find((x) => x.id === o.entity_id); for (const pid of (e?.person_ids as string[]) ?? []) people.add(pid); }
    else people.add(o.entity_id);
  }
  const end = M.ymdFromJdn(M.ymdJdn(M.tenYearsEnd(t)) - 1);
  const preview = occ.slice(0, 5).map(([o]) => h("li", null, `${dmy(o.date)} — ${o.headline}`));
  const ok = await confirmDialog({
    title: "Xuất tệp lịch", ok: "Tải tệp .ics", cls: "ics-dialog",
    body: [
      h("p", { class: "pd-privacy" }, `Tệp này chứa tên và ngày giỗ của ${people.size} người. Bất kỳ ai có tệp đều đọc được.`),
      h("p", null, `Gồm ${occ.length} ngày trong 10 năm tới (từ ${dmy(t)} đến ${dmy(end)}), mỗi ngày có lời nhắc như đã đặt.`),
      h("p", { class: "flabel" }, "Xem trước:"),
      h("ul", { class: "dots pd-preview" }, ...preview, occ.length > 5 ? h("li", null, `… và ${occ.length - 5} ngày khác`) : null),
      h("p", { class: "fine" }, "Mở tệp bằng ứng dụng Lịch của điện thoại hoặc máy tính để nhập. Một số ứng dụng lịch dùng lời nhắc mặc định của chúng thay cho lời nhắc trong tệp."),
    ],
    onOk: () => { download(M.fileName(t), M.generateIcs(cal(), sn.ds, opt), "text/calendar;charset=utf-8"); },
  });
  if (ok) toast("Đã tải tệp lịch. Mở tệp bằng ứng dụng Lịch để nhập.");
}

// ---------------- trang Sao lưu ----------------
export async function renderBackup(root: HTMLElement, rerender: () => void): Promise<void> {
  const sn = await snap();
  const s = await store();
  const last = sn.settings.last_backup_at as string | null;
  const photos = (await s.photos()).length;
  const lastTxt = last ? `Lần sao lưu gần nhất: ${dmy({ y: +last.slice(0, 4), m: +last.slice(5, 7), d: +last.slice(8, 10) })}` : "Chưa sao lưu lần nào";

  // A. .ics
  const icsBtn = h("button", { type: "button", class: "primary", id: "ics-open", onclick: () => void openIcsDialog() }, "Xuất tệp lịch (.ics)…");
  // B. xuất sao lưu
  const expBtn = h("button", { type: "button", class: "primary", id: "bk-export", disabled: sn.persons.length === 0 && sn.events.length === 0, onclick: () => void exportFlow() }, "Tạo tệp sao lưu…");
  async function exportFlow(): Promise<void> {
    const pw1 = h("input", { type: "password", id: "bk-pw1", autocomplete: "new-password", autocapitalize: "off", spellcheck: "false", class: "tin" }) as HTMLInputElement;
    const pw2 = h("input", { type: "password", id: "bk-pw2", autocomplete: "new-password", autocapitalize: "off", spellcheck: "false", class: "tin" }) as HTMLInputElement;
    const prog = h("progress", { max: "1", value: "0", hidden: true, id: "bk-prog" }) as HTMLProgressElement;
    const progTxt = h("p", { class: "fine", hidden: true }, "Đang mã hóa… (có thể mất vài giây)");
    const ok = await confirmDialog({
      title: "Tạo tệp sao lưu", ok: "Tạo và tải tệp", cls: "bk-dialog",
      body: [
        h("p", null, `Tệp sẽ chứa ${sn.persons.length} người, ${sn.events.length} sự kiện, ${photos} ảnh — đã mã hóa bằng mật khẩu.`),
        h("div", { class: "field" }, h("label", { for: "bk-pw1" }, "Mật khẩu (ít nhất 8 ký tự)"), pw1),
        h("div", { class: "field" }, h("label", { for: "bk-pw2" }, "Nhập lại mật khẩu"), pw2),
        h("p", { class: "warn" }, "Quên mật khẩu là mất dữ liệu trong tệp — không ai lấy lại được, kể cả Clevix Lab."),
        h("p", { class: "fine" }, "Gợi ý: dùng một câu dễ nhớ, ví dụ “nhà ở Hà Đông có cây bàng”."),
        prog, progTxt,
      ],
      onOk: async () => {
        const a = pw1.value, b = pw2.value;
        if ([...a.normalize("NFC")].length < 8) return "Mật khẩu cần ít nhất 8 ký tự";
        if (a.normalize("NFC") !== b.normalize("NFC")) return "Hai lần nhập mật khẩu chưa giống nhau";
        prog.hidden = false; progTxt.hidden = false;
        const bytes = await s.exportBackup(a, { version: APP_VERSION, platform: "pwa" }, (x) => { prog.value = x; });
        download(`LichVanHoaViet-saoluu-${iso(today())}.lvhv`, bytes, "application/octet-stream");
        return true;
      },
    });
    if (ok) { toast("Đã tạo tệp sao lưu. Hãy cất tệp ở nơi an toàn (ổ đám mây, USB) — tệp không tự gửi đi đâu."); rerender(); }
  }

  // C. nhập
  const fileIn = h("input", { type: "file", id: "bk-file", accept: ".lvhv,application/octet-stream" }) as HTMLInputElement;
  const pwIn = h("input", { type: "password", id: "bk-ipw", autocomplete: "current-password", autocapitalize: "off", spellcheck: "false", class: "tin" }) as HTMLInputElement;
  const impErr = h("p", { class: "ferr", role: "alert", hidden: true, id: "bk-imp-err" });
  const impProg = h("progress", { max: "1", value: "0", hidden: true }) as HTMLProgressElement;
  const impBtn = h("button", { type: "button", class: "primary", id: "bk-open", onclick: () => void importFlow() }, "Mở tệp");
  async function importFlow(): Promise<void> {
    impErr.hidden = true;
    const f = fileIn.files?.[0];
    if (!f) { impErr.textContent = "Hãy chọn tệp sao lưu"; impErr.hidden = false; return; }
    if (!pwIn.value) { impErr.textContent = "Hãy nhập mật khẩu của tệp"; impErr.hidden = false; return; }
    impBtn.disabled = true; impProg.hidden = false;
    let v: VerifiedBackup;
    try { v = await openBackup(new Uint8Array(await f.arrayBuffer()), pwIn.value, (x) => { impProg.value = x; }); }
    catch (e) { impErr.textContent = (e as Error).message; impErr.hidden = false; return; }
    finally { impBtn.disabled = false; impProg.hidden = true; }
    pwIn.value = "";
    const pv = v.preview;
    const exp = dmy({ y: +pv.exported_at.slice(0, 4), m: +pv.exported_at.slice(5, 7), d: +pv.exported_at.slice(8, 10) });
    const plat = ({ pwa: "ứng dụng web", android: "Android", ios: "iPhone/iPad", macos: "máy Mac", windows: "Windows" } as Record<string, string>)[pv.platform] ?? pv.platform;
    let mode: "merge" | "replace" = "merge";
    const radio = (val: "merge" | "replace", label: string, sub: string) => {
      const r = h("input", { type: "radio", name: "bk-mode", id: `bk-mode-${val}`, value: val, checked: val === "merge" }) as HTMLInputElement;
      r.addEventListener("change", () => { if (r.checked) mode = val; });
      return h("label", { class: "check radio", for: `bk-mode-${val}` }, r, h("span", null, h("b", null, label), h("br"), h("small", null, sub)));
    };
    const ok = await confirmDialog({
      title: "Khôi phục dữ liệu", ok: "Khôi phục", cls: "imp-dialog",
      body: [
        h("p", null, `Tệp xuất ngày ${exp} từ ${plat}: ${pv.persons} người, ${pv.events} sự kiện, ${pv.photos} ảnh.`),
        h("fieldset", { class: "pfs" }, h("legend", null, "Cách nhập"),
          radio("merge", "Gộp vào dữ liệu đang có (khuyên dùng)", "Giữ bản mới hơn của từng hồ sơ"),
          radio("replace", "Thay toàn bộ", "Xóa dữ liệu đang có trên máy rồi dùng dữ liệu trong tệp")),
      ],
      onOk: async () => {
        if (mode === "replace") {
          const n = (await s.persons()).length;
          const sure = await confirmDialog({ title: "Thay toàn bộ dữ liệu?", danger: true, focusCancel: true, ok: "Thay toàn bộ",
            body: [`Dữ liệu hiện có trên máy (${n} người) sẽ bị thay bằng dữ liệu trong tệp xuất ngày ${exp}. Không hoàn tác được.`] });
          if (!sure) return false;
        }
        try { await s.applyImport(v, mode); } catch { return "Chưa khôi phục được — dữ liệu trên máy giữ nguyên."; }
        return true;
      },
    });
    if (ok) { toast(`Đã khôi phục: ${pv.persons} người, ${pv.events} sự kiện.`); rerender(); }
  }

  // D. nhắc sao lưu
  const nag = h("select", { id: "bk-nag" }, ...[[30, "30 ngày"], [60, "60 ngày"], [90, "90 ngày"], [0, "Không nhắc"]].map(([v, t]) => h("option", { value: String(v) }, String(t)))) as HTMLSelectElement;
  nag.value = String(sn.settings.backup_nag_days);
  nag.addEventListener("change", async () => { await s.putSettings({ ...(sn.settings as JsonObj), backup_nag_days: Number(nag.value) }); sn.settings.backup_nag_days = Number(nag.value); toast("Đã lưu cài đặt."); });

  clear(root);
  root.append(h("div", { class: "pcols" },
    h("div", { class: "pcol-main" },
      h("section", { class: "panel", id: "bk-ics" }, h("h2", null, "Để điện thoại tự nhắc: xuất lịch (.ics)"),
        h("p", null, "Ứng dụng web không gửi được thông báo khi đang đóng. Lời nhắc chỉ hiện ở đầu trang khi bạn mở ứng dụng vào ngày cần nhắc. Để điện thoại tự nhắc cả khi không mở ứng dụng, hãy xuất tệp lịch (.ics) và mở bằng ứng dụng Lịch của máy."),
        icsBtn),
      h("section", { class: "panel", id: "bk-out" }, h("h2", null, "Sao lưu dữ liệu"),
        h("p", { id: "bk-last" }, lastTxt),
        h("p", { class: "fine" }, "Tệp sao lưu được mã hóa bằng mật khẩu của bạn và chỉ nằm ở nơi bạn cất. Ứng dụng không gửi dữ liệu đi đâu."),
        expBtn)),
    h("div", { class: "pcol-side" },
      h("section", { class: "panel", id: "bk-in" }, h("h2", null, "Khôi phục từ tệp sao lưu"),
        h("div", { class: "field" }, h("label", { for: "bk-file" }, "Tệp sao lưu (.lvhv)"), fileIn),
        h("div", { class: "field" }, h("label", { for: "bk-ipw" }, "Mật khẩu của tệp"), pwIn),
        impErr, impProg, impBtn),
      h("section", { class: "panel" }, h("h2", null, "Nhắc sao lưu"),
        h("div", { class: "field" }, h("label", { for: "bk-nag" }, "Nhắc khi có thay đổi chưa sao lưu sau"), nag)))));
}
