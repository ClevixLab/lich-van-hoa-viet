// Sao lưu và lịch điện thoại (08 §3.8): xuất .ics (02 §8, S10), xuất/nhập sao lưu mã hóa (03, S10), nhắc sao lưu.
// Mật mã nằm trong mô-đun tải lười của lớp lưu trữ (store/backup-crypto, chỉ nạp khi xuất/nhập — P6).
import { clear, h } from "../dom";
import { APP_VERSION } from "../gen/content";
import { M, cal, dmy, nowTs, snap, store, today, type JsonObj } from "./core";
import { confirmDialog, itemTitle, toast } from "./ui";
import { openBackup, type VerifiedBackup } from "../store/index";
import { testGate } from "../testhooks";

const iso = (d: M.Ymd) => M.iso(d);

/**
 * MA3-15-02 (S12): cỡ lớn nhất của một tệp sao lưu hợp lệ = phần đầu 72 byte + bản rõ ≤ 128 MiB + thẻ 16 byte
 * (03 §2.1–§2.3; cùng giá trị HEADER_LEN, MAX_PLAINTEXT, TAG_LEN của store/backup-crypto — không import tĩnh tệp đó
 * vì nó kéo theo mã mật mã, P6). Tệp lớn hơn bị từ chối TRƯỚC khi đọc vào bộ nhớ.
 */
export const MAX_BACKUP_FILE = 72 + 128 * 1024 * 1024 + 16;

/** Ngày theo giờ của máy từ một thời điểm RFC 3339 (…Z) — MA3-15-06. */
function localDmy(ts: string): string {
  const d = new Date(ts);
  return dmy({ y: d.getFullYear(), m: d.getMonth() + 1, d: d.getDate() });
}

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
  // Tóm tắt của lõi (MA3N-24-08; cùng bản với Rust ics::summary): số mục, số người, loại dữ liệu, xem trước.
  const sum = M.icsSummary(cal(), sn.ds, opt);
  if (!sum.events) {
    await confirmDialog({ title: "Xuất tệp lịch", body: [h("p", null, "Chưa có hồ sơ hay sự kiện nào bật nhắc, nên chưa có gì để xuất.")], ok: "Đã hiểu", cancel: "Đóng" });
    return;
  }
  // Xem trước: PWA ghi thêm người gắn với sự kiện (itemTitle), nên lấy 5 mục đầu từ cùng danh sách của lõi.
  const preview = M.icsOccurrences(cal(), sn.ds, opt).slice(0, sum.preview.length).map(([o]) => h("li", null, `${dmy(o.date)} — ${itemTitle(sn, o)}`));
  const ok = await confirmDialog({
    title: "Xuất tệp lịch", ok: "Tải tệp .ics", cls: "ics-dialog",
    body: [
      h("p", { class: "pd-privacy" }, sum.privacy_text),
      // TKN-X-04 (duyệt thiết kế native): mọi nền tảng nói rõ .ics không gồm thông tin báo tử (R-PRIV-2).
      h("p", { class: "fine", id: "ics-no-notice" }, "Không gồm thông tin báo tử."),
      h("p", null, `Gồm ${sum.events} ngày trong 10 năm tới (từ ${dmy(sum.from)} đến ${dmy(sum.to)}), mỗi ngày có lời nhắc như đã đặt.`),
      h("p", { class: "flabel" }, "Xem trước:"),
      h("ul", { class: "dots pd-preview" }, ...preview, sum.events > sum.preview.length ? h("li", null, `… và ${sum.events - sum.preview.length} ngày khác`) : null),
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
  const lastTxt = last ? `Lần sao lưu gần nhất: ${localDmy(last)}` : "Chưa sao lưu lần nào trên máy này";
  // TK-16 (08 §12h): sau "Thay toàn bộ", cài đặt (kể cả last_backup_at) lấy theo tệp — lõi không đổi được vì vectơ R6
  // đã khóa. Giao diện nói thật nguồn dữ liệu: ngày tạo của tệp đã khôi phục, ghi ở khóa giao diện `ui_restored_from`
  // trong kho meta (không thuộc tài liệu sao lưu, không vào stateHash, không đổi logic nhắc sao lưu).
  const restoredFrom = (await s.meta("ui_restored_from")) as string | undefined;

  // A. .ics
  const icsBtn = h("button", { type: "button", class: "primary", id: "ics-open", onclick: () => void openIcsDialog() }, "Xuất tệp lịch (.ics)…");
  // B. xuất sao lưu
  const expBtn = h("button", { type: "button", class: "primary", id: "bk-export", disabled: sn.persons.length === 0 && sn.events.length === 0, onclick: () => void exportFlow() }, "Tạo tệp sao lưu…");
  async function exportFlow(): Promise<void> {
    // BM2-12 / R-PDF-1: hộp xuất ghi số tệp PDF lưu nguyên bản.
    const nPdf = (await s.attachments()).filter((a) => a.mime === "application/pdf").length;
    const pw1 = h("input", { type: "password", id: "bk-pw1", autocomplete: "new-password", autocapitalize: "off", spellcheck: "false", class: "tin" }) as HTMLInputElement;
    const pw2 = h("input", { type: "password", id: "bk-pw2", autocomplete: "new-password", autocapitalize: "off", spellcheck: "false", class: "tin" }) as HTMLInputElement;
    const e1 = h("p", { class: "ferr", id: "bk-pw1-err", role: "alert", hidden: true });
    const e2 = h("p", { class: "ferr", id: "bk-pw2-err", role: "alert", hidden: true });
    const mark = (inp: HTMLInputElement, el: HTMLElement, msg: string | null) => {
      el.hidden = !msg; el.textContent = msg ?? "";
      if (msg) { inp.setAttribute("aria-invalid", "true"); inp.setAttribute("aria-describedby", el.id); } else { inp.removeAttribute("aria-invalid"); inp.removeAttribute("aria-describedby"); }
    };
    pw1.addEventListener("input", () => { if (!e1.hidden && [...pw1.value.normalize("NFC")].length >= 8) mark(pw1, e1, null); });
    pw2.addEventListener("input", () => { if (!e2.hidden && pw1.value.normalize("NFC") === pw2.value.normalize("NFC")) mark(pw2, e2, null); });
    const prog = h("progress", { max: "1", value: "0", hidden: true, id: "bk-prog" }) as HTMLProgressElement;
    const progTxt = h("p", { class: "fine", hidden: true }, "Đang mã hóa… (có thể mất vài giây)");
    const ok = await confirmDialog({
      title: "Tạo tệp sao lưu", ok: "Tạo và tải tệp", cls: "bk-dialog",
      body: [
        h("p", null, `Tệp sẽ chứa ${sn.persons.length} người, ${sn.events.length} sự kiện, ${photos} ảnh — đã mã hóa bằng mật khẩu.`),
        ...(sn.persons.some((p) => (p.death as JsonObj | null)?.notice) ? [h("p", { class: "fine", id: "bk-notice" }, "Tệp gồm cả thông tin báo tử (người báo tin, nơi mất, lý do, ghi chú, tệp đính kèm) — chỉ ai biết mật khẩu mới đọc được.")] : []),
        ...(nPdf ? [h("p", { class: "fine", id: "bk-pdf" }, `Tệp gồm ${nPdf} tệp PDF đính kèm, lưu nguyên bản — có thể còn thông tin của máy quét (tên máy, phần mềm, ngày quét).`)] : []),
        h("div", { class: "field" }, h("label", { for: "bk-pw1" }, "Mật khẩu (ít nhất 8 ký tự)"), pw1, e1),
        h("div", { class: "field" }, h("label", { for: "bk-pw2" }, "Nhập lại mật khẩu"), pw2, e2),
        h("p", { class: "notice pd-note" }, "Lưu ý: quên mật khẩu là mất dữ liệu trong tệp — không ai lấy lại được, kể cả Clevix Lab."),
        h("p", { class: "fine" }, "Gợi ý: dùng một câu dễ nhớ, ví dụ “nhà ở Hà Đông có cây bàng”."),
        prog, progTxt,
      ],
      onOk: async () => {
        const a = pw1.value, b = pw2.value;
        mark(pw1, e1, null); mark(pw2, e2, null);
        // Quy tắc mật khẩu của lõi (MA3N-24-08), cùng bản với Rust lvhv_backup::password_error.
        const e1m = M.backupPasswordError(a, null);
        if (e1m) { mark(pw1, e1, e1m); pw1.focus(); return false; }
        const e2m = M.backupPasswordError(a, b);
        if (e2m) { mark(pw2, e2, e2m); pw2.focus(); return false; }
        prog.hidden = false; progTxt.hidden = false;
        if (__LVHV_TEST_HOOKS__) await testGate("backup-encrypt"); // chỉ bản dựng kiểm thử: giữ bước mã hóa tới khi kiểm thử cho chạy (MA3-15-03)
        const bytes = await s.exportBackup(a, { version: APP_VERSION, platform: "pwa" }, (x) => { prog.value = x; });
        pw1.value = ""; pw2.value = ""; // MA3-15-08: không giữ mật khẩu trong ô sau khi xuất
        download(`LichVanHoaViet-saoluu-${iso(today())}.lvhv`, bytes, "application/octet-stream");
        // MA3N-23-01: chỉ ghi nhận "đã sao lưu" sau khi đã giao tệp cho trình duyệt tải về.
        await s.markBackupSaved(nowTs());
        return true;
      },
    });
    if (ok) { toast("Đã tạo tệp sao lưu. Hãy cất tệp ở nơi an toàn (ổ đám mây, USB) — tệp không tự gửi đi đâu."); rerender(); }
  }

  // C. nhập
  const fileIn = h("input", { type: "file", id: "bk-file", accept: ".lvhv,application/octet-stream", class: "sr-only", tabindex: "-1", "aria-hidden": "true" }) as HTMLInputElement;
  const fileName = h("p", { class: "fine", id: "bk-file-name", "aria-live": "polite" }, "Chưa chọn tệp.");
  fileIn.addEventListener("change", () => { const f = fileIn.files?.[0]; fileName.textContent = f ? `Đã chọn: ${f.name}` : "Chưa chọn tệp."; });
  const pwIn = h("input", { type: "password", id: "bk-ipw", autocomplete: "current-password", autocapitalize: "off", spellcheck: "false", class: "tin" }) as HTMLInputElement;
  const impErr = h("p", { class: "ferr", role: "alert", hidden: true, id: "bk-imp-err" });
  const impProg = h("progress", { max: "1", value: "0", hidden: true }) as HTMLProgressElement;
  const impBtn = h("button", { type: "button", class: "btn2", id: "bk-open", onclick: () => void importFlow() }, "Mở tệp");
  async function importFlow(): Promise<void> {
    impErr.hidden = true;
    const f = fileIn.files?.[0];
    if (!f) { impErr.textContent = "Hãy chọn tệp sao lưu"; impErr.hidden = false; return; }
    if (!pwIn.value) { impErr.textContent = "Hãy nhập mật khẩu của tệp"; impErr.hidden = false; return; }
    // S12: tệp lớn hơn mọi tệp sao lưu hợp lệ → thông báo chung (03 §2.4), không đọc tệp vào bộ nhớ.
    if (f.size > MAX_BACKUP_FILE) { impErr.textContent = "Sai mật khẩu hoặc tệp bị hỏng"; impErr.hidden = false; return; }
    impBtn.disabled = true; impProg.hidden = false;
    let v: VerifiedBackup;
    try { v = await openBackup(new Uint8Array(await f.arrayBuffer()), pwIn.value, (x) => { impProg.value = x; }); }
    catch (e) { impErr.textContent = (e as Error).message; impErr.hidden = false; return; }
    finally { impBtn.disabled = false; impProg.hidden = true; }
    pwIn.value = "";
    const pv = v.preview;
    const exp = localDmy(pv.exported_at);
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
        if (mode === "replace") {
          await new Promise<void>((done) => {
            const tx = s.db.transaction("meta", "readwrite");
            tx.objectStore("meta").put(pv.exported_at, "ui_restored_from");
            tx.oncomplete = () => done(); tx.onerror = () => done();
          });
        }
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
        restoredFrom ? h("p", { id: "bk-restored", class: "fine" }, `Dữ liệu được khôi phục từ bản sao lưu ngày ${localDmy(restoredFrom)}.`) : null,
        h("p", { class: "fine" }, "Tệp sao lưu được mã hóa bằng mật khẩu của bạn và chỉ nằm ở nơi bạn cất. Ứng dụng không gửi dữ liệu đi đâu."),
        expBtn)),
    h("div", { class: "pcol-side" },
      h("section", { class: "panel", id: "bk-in" }, h("h2", null, "Khôi phục từ tệp sao lưu"),
        h("div", { class: "field" }, h("span", { class: "flabel" }, "Tệp sao lưu (.lvhv)"), fileIn, h("button", { type: "button", class: "btn2 filebtn", id: "bk-file-btn", onclick: () => fileIn.click() }, "Chọn tệp sao lưu…"), fileName),
        h("div", { class: "field" }, h("label", { for: "bk-ipw" }, "Mật khẩu của tệp"), pwIn),
        impErr, impProg, impBtn),
      h("section", { class: "panel" }, h("h2", null, "Nhắc sao lưu"),
        h("div", { class: "field" }, h("label", { for: "bk-nag" }, "Nhắc khi có thay đổi chưa sao lưu sau"), nag)))));
}
