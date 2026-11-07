// Cửa vào mô-đun "Người thân" (docs/v0.2/08). NẠP LƯỜI từ main.ts bằng import() — không có trong lần tải đầu (P3, P6).
import { clear, h } from "../dom";
import { renderBackup, openIcsDialog } from "./backup";
import { snap, store } from "./core";
import { renderEventForm, renderPersonForm, type Nav } from "./forms";
import { renderConvertForm } from "./notice";
import { disposeMemorial, renderMemorial } from "./memorial";
import { setToastHost } from "./ui";
import { reminderCards, renderList, renderPeriod, renderPerson, renderUpcoming } from "./views";
import { getPrefs, setPref } from "../prefs";

export type { Nav };

const TABS: [string, string][] = [["", "Sắp tới"], ["theo-ky", "Theo kỳ"], ["ho-so", "Hồ sơ"], ["sao-luu", "Sao lưu"]];
const TITLES: Record<string, string> = { "": "Người thân", "theo-ky": "Theo kỳ", "ho-so": "Hồ sơ", "sao-luu": "Sao lưu" };

/** Ghi cờ "đã dùng Người thân" (MA3-15-08) khi kho có hồ sơ. */
function markUsed(n: number): void { if (n > 0 && !getPrefs().people) setPref("people", true); }

/** Dựng trang con `sub` (phần sau `#/nguoi-than/`) vào `root`. */
export async function renderPeople(root: HTMLElement, sub: string, nav: Nav): Promise<void> {
  disposeMemorial();
  const parts = sub.split("/").filter(Boolean);
  const top = parts[0] ?? "";
  const toastHost = h("div", { class: "toast-host", role: "status", "aria-live": "polite" });
  setToastHost(toastHost);
  root.classList.add("people");
  const body = h("div", { class: "people-body" });
  // 04 B5 mục 3: chuyển đổi lỗi → lõi mở kho cũ ở chế độ chỉ đọc (`migrationStatus().state === "read_only"`).
  const ms = await store().then((x) => x.migrationStatus()).catch(() => null);
  if (ms?.state === "read_only") {
    root.append(h("div", { class: "notice warnbox", role: "alert", id: "ro-banner" },
      h("p", null, h("b", null, "Chưa chuyển được dữ liệu sang bản mới. "), "Dữ liệu cũ vẫn còn nguyên trên máy và đang mở ở chế độ chỉ đọc: xem được, nhưng chưa thêm hay sửa được."),
      h("p", null, "Hãy tạo ngay một tệp sao lưu để giữ dữ liệu: vào Sao lưu → “Tạo tệp sao lưu…”, rồi cất tệp ở nơi an toàn. Sau đó thử tải lại trang; nếu vẫn thấy thông báo này, hãy báo cho Clevix Lab kèm tệp sao lưu."),
      h("a", { class: "btnlink main", href: "#/nguoi-than/sao-luu" }, "Đi tới Sao lưu")));
  }
  try {
    if (top === "" || top in TITLES) {
      const h1 = h("h1", { tabindex: "-1" }, "Người thân");
      const subnav = h("nav", { class: "subnav", "aria-label": "Mục trong Người thân" },
        ...TABS.map(([k, t]) => h("a", { href: k ? `#/nguoi-than/${k}` : "#/nguoi-than", "aria-current": k === top ? "page" : null, "data-k": k || "sap-toi" }, t)));
      root.append(h("div", { class: "pagehead" }, h1,
        h("p", null, "Ngày giỗ, sinh nhật và sự kiện của gia đình. Dữ liệu chỉ nằm trên máy này.")), subnav, toastHost, body);
      document.title = `${TITLES[top]} — Lịch và Văn hóa Việt`;
      if (top === "") await renderUpcoming(body, nav, () => void openIcsDialog());
      else if (top === "theo-ky") await renderPeriod(body, nav);
      else if (top === "ho-so") await renderList(body, nav);
      else await renderBackup(body, () => nav.go("#/nguoi-than/sao-luu", true));
      return;
    }
    root.append(toastHost, body);
    if (top === "them") await renderPersonForm(body, parts[1] === "dang-song" ? "living" : "deceased", null, nav);
    else if (top === "sua" && parts[1]) await renderPersonForm(body, null, parts[1], nav);
    else if (top === "bao-tu" && parts[1]) await renderConvertForm(body, parts[1], nav);
    else if (top === "nguoi" && parts[1]) { await renderPerson(body, parts[1], nav); markUsed(body.querySelector("#person-name") ? 1 : 0); }
    else if (top === "su-kien" && parts[1]) await renderEventForm(body, parts[1], parts[2] && parts[2] !== "moi" ? parts[2] : null, nav);
    else if (top === "tuong-niem" && parts[1]) { root.classList.add("is-memorial"); await renderMemorial(body, parts[1]); }
    else nav.go("#/nguoi-than", true);
  } catch (e) {
    console.error(e);
    clear(body);
    body.append(h("p", { class: "warn", role: "alert" }, `Không mở được dữ liệu trên máy: ${(e as Error).message}. Lịch vẫn dùng được bình thường.`));
  }
}

/** Thẻ nhắc khi mở ứng dụng (02 §7.5). Chỉ gọi khi đã có cơ sở dữ liệu `lvhv`. */
export async function startupCards(host: HTMLElement, nav: Nav): Promise<boolean> {
  const sn = await snap();
  markUsed(sn.persons.length);
  clear(host);
  const c = reminderCards(sn, nav, () => { host.hidden = true; }, true);
  if (c) host.append(c);
  return !!c;
}

export { disposeMemorial };
