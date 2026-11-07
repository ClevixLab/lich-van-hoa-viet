// Phiên bản và cập nhật: một nơi duy nhất quyết định "có bản mới không" và "cập nhật thế nào".
//
// - Bản đang chạy: APP_VERSION / BUILD_ID / BUILD_DATE (sinh lúc build từ tệp VERSION ở gốc kho).
// - Bản trên máy chủ: version.json (service worker luôn lấy qua mạng, cache: "no-store").
// - Cập nhật: service worker mới cài xong thì ở trạng thái chờ (RS-08); người dùng bấm "Cập nhật ngay" →
//   gửi SKIP_WAITING → bản mới kích hoạt → trang tải lại MỘT lần khi controllerchange.
// - Tự nhắc: lúc mở ứng dụng và khi trang hiện lại (visibilitychange), tối đa 6 giờ một lần. Mốc thời gian chỉ
//   giữ trong bộ nhớ của trang (S4: không ghi gì xuống máy).
import { APP_VERSION } from "./gen/content";

export interface RemoteVersion { version: string; build: string; date: string; notes: string[] }
export type CheckResult =
  | { status: "latest" }
  | { status: "new"; version: string | null; notes: string[] }
  | { status: "offline" };

const SIX_HOURS = 6 * 3600 * 1000;
const hasSW = typeof navigator !== "undefined" && "serviceWorker" in navigator && location.protocol !== "file:";

let reg: ServiceWorkerRegistration | null = null;
let lastAutoCheck = 0;
let userAskedUpdate = false;
let controlled = hasSW && !!navigator.serviceWorker.controller;
/**
 * Nguồn của một lần kiểm tra: "user" (bấm "Kiểm tra cập nhật"), "auto" (lúc mở / trang hiện lại, tối đa 6 giờ một
 * lần), "install" (service worker mới vừa cài xong — lần báo thứ hai của cùng một phát hành). `seq` tăng dần theo
 * thứ tự BẮT ĐẦU kiểm tra, để biết một kết quả có phải của lần kiểm tra bắt đầu trước khi người dùng bấm "Để sau".
 */
export type CheckSource = "user" | "auto" | "install";
export interface CheckInfo { source: CheckSource; seq: number }
let seq = 0;
/** Số lần kiểm tra đã bắt đầu tới giờ. */
export function checksStarted(): number { return seq; }
const listeners = new Set<(r: CheckResult, info: CheckInfo) => void>();

/** So sánh X.Y.Z; > 0 nếu a mới hơn b. Chuỗi sai dạng coi như 0.0.0. */
export function cmpVersion(a: string, b: string): number {
  const p = (s: string) => (/^(\d+)\.(\d+)\.(\d+)$/.exec(s.trim()) ?? [0, 0, 0, 0]).slice(1).map(Number);
  const x = p(a), y = p(b);
  for (let i = 0; i < 3; i++) if (x[i] !== y[i]) return x[i] - y[i];
  return 0;
}

/** Đăng ký nghe kết quả kiểm tra (banner, trang Giới thiệu). Trả hàm hủy. */
export function onUpdateResult(fn: (r: CheckResult, info: CheckInfo) => void): () => void {
  listeners.add(fn);
  return () => listeners.delete(fn);
}
function emit(r: CheckResult, info: CheckInfo): void { listeners.forEach((fn) => fn(r, info)); }

async function fetchRemote(): Promise<RemoteVersion> {
  const res = await fetch("version.json", { cache: "no-store" });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  const j = await res.json() as Partial<RemoteVersion>;
  if (typeof j.version !== "string") throw new Error("version.json không hợp lệ");
  return { version: j.version, build: String(j.build ?? ""), date: String(j.date ?? ""), notes: Array.isArray(j.notes) ? j.notes.map(String) : [] };
}

/** Kiểm tra cập nhật: hỏi trình duyệt tải lại sw.js (registration.update()) và đọc version.json. */
export async function checkForUpdate(source: CheckSource = "user"): Promise<CheckResult> {
  const info: CheckInfo = { source, seq: ++seq };
  let result: CheckResult;
  try {
    const [remote] = await Promise.all([fetchRemote(), reg ? reg.update().catch(() => undefined) : Promise.resolve()]);
    const newer = cmpVersion(remote.version, APP_VERSION) > 0;
    const pending = !!(reg && (reg.waiting || reg.installing));
    result = newer ? { status: "new", version: remote.version, notes: remote.notes }
      : pending ? { status: "new", version: null, notes: [] }
      : { status: "latest" };
  } catch {
    result = { status: "offline" };
  }
  emit(result, info);
  return result;
}

/** Kiểm tra im lặng, tối đa 6 giờ một lần (mốc chỉ trong bộ nhớ). */
export function autoCheck(): void {
  const now = Date.now();
  if (now - lastAutoCheck < SIX_HOURS) return;
  lastAutoCheck = now;
  void checkForUpdate("auto");
}

/** Chờ service worker đang cài xong (trạng thái installed) — tối đa `ms`. */
function waitInstalled(w: ServiceWorker, ms: number): Promise<boolean> {
  return new Promise((resolve) => {
    if (w.state === "installed" || w.state === "activated") { resolve(true); return; }
    const t = window.setTimeout(() => resolve(false), ms);
    w.addEventListener("statechange", () => {
      if (w.state === "installed" || w.state === "activated") { window.clearTimeout(t); resolve(true); }
      else if (w.state === "redundant") { window.clearTimeout(t); resolve(false); }
    });
  });
}

/** "Cập nhật ngay": kích hoạt bản đang chờ rồi tải lại một lần (controllerchange). */
export async function applyUpdate(): Promise<void> {
  userAskedUpdate = true;
  if (!reg) { location.reload(); return; }
  if (!reg.waiting && !reg.installing) await reg.update().catch(() => undefined);
  if (!reg.waiting && reg.installing) await waitInstalled(reg.installing, 30000);
  const w = reg.waiting;
  if (w) w.postMessage({ type: "SKIP_WAITING" });
  else location.reload(); // không có bản chờ (ví dụ trình duyệt không hỗ trợ SW): tải lại là đủ
}

/** Gắn với service worker đã đăng ký: bản mới cài xong giữa chừng cũng báo qua banner. */
export function attachRegistration(r: ServiceWorkerRegistration): void {
  reg = r;
  const announce = () => { void checkForUpdate("install"); };
  if (r.waiting && navigator.serviceWorker.controller) announce();
  r.addEventListener("updatefound", () => {
    const w = r.installing;
    w?.addEventListener("statechange", () => {
      if (w.state === "installed" && navigator.serviceWorker.controller) announce();
    });
  });
}

let reloaded = false;
if (hasSW) {
  navigator.serviceWorker.addEventListener("controllerchange", () => {
    // Lần cài đầu, clients.claim() cũng phát controllerchange — chỉ tải lại khi thay một bản cũ đang điều
    // khiển trang, hoặc khi người dùng vừa bấm "Cập nhật ngay".
    const shouldReload = controlled || userAskedUpdate;
    controlled = true;
    if (reloaded || !shouldReload) return;
    reloaded = true;
    location.reload();
  });
}
