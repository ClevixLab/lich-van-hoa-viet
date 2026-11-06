// Tùy chọn giao diện (cỡ chữ, giao diện sáng/tối). Đây là dữ liệu duy nhất được lưu
// trên máy — không phải dữ liệu cá nhân (tiêu chí S4). Mọi truy cập đều bọc try/catch.

export type TextSize = "1" | "2" | "3";
export type Theme = "auto" | "light" | "dark";

const KEY = "lvhv.ui";

/**
 * `people`: máy này đã từng lưu hồ sơ ở mục Người thân (chỉ một cờ đúng/sai, không phải dữ liệu cá nhân). Dùng để
 * biết có cần nạp mô-đun kiểm thẻ nhắc khi mở ứng dụng trên trình duyệt không có `indexedDB.databases()` (MA3-15-08).
 */
interface Prefs { size: TextSize; theme: Theme; people?: true }

function read(): Prefs {
  const def: Prefs = { size: "1", theme: "auto" };
  try {
    const raw = window.localStorage.getItem(KEY);
    if (!raw) return def;
    const p = JSON.parse(raw) as Partial<Prefs>;
    return {
      size: p.size === "2" || p.size === "3" ? p.size : "1",
      theme: p.theme === "light" || p.theme === "dark" ? p.theme : "auto",
      ...(p.people === true ? { people: true as const } : {}),
    };
  } catch {
    return def;
  }
}

let prefs = read();

function write(): void {
  try {
    window.localStorage.setItem(KEY, JSON.stringify(prefs));
  } catch {
    /* bộ nhớ trình duyệt bị chặn: vẫn chạy bình thường, chỉ không nhớ tùy chọn */
  }
}

export function getPrefs(): Readonly<Prefs> {
  return prefs;
}

export function setPref<K extends keyof Prefs>(k: K, v: Prefs[K]): void {
  prefs = { ...prefs, [k]: v };
  write();
  applyPrefs();
}

export function applyPrefs(): void {
  const root = document.documentElement;
  root.dataset.size = prefs.size;
  if (prefs.theme === "auto") delete root.dataset.theme;
  else root.dataset.theme = prefs.theme;
}
