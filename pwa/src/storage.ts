// Cửa vào lớp lưu trữ cho giao diện (đợt 3). Không import tĩnh src/store/: mô-đun lưu trữ (IndexedDB,
// kiểm tra dữ liệu) và mô-đun mật mã chỉ được nạp khi gọi các hàm dưới đây (tiêu chí P6) — lần tải đầu
// không mở IndexedDB và không tải mã mật mã.
type StoreModule = typeof import("./store/index");

let mod: Promise<StoreModule> | null = null;

/** Nạp lười mô-đun lưu trữ (một lần). */
export function loadStore(): Promise<StoreModule> {
  return (mod ??= import("./store/index"));
}

/** Mở kho IndexedDB `lvhv` (tạo mới nếu chưa có). */
export async function openStore() {
  return (await loadStore()).openStore();
}

/**
 * Hook nhắc sao lưu (01 §3.9): `true` khi có thay đổi chưa sao lưu quá `backup_nag_days` ngày.
 * Không tạo cơ sở dữ liệu nếu người dùng chưa từng lưu gì (trả `false`).
 */
export async function backupReminderDue(now?: string): Promise<boolean> {
  try {
    if (indexedDB.databases && !(await indexedDB.databases()).some((d) => d.name === "lvhv")) return false;
  } catch { /* trình duyệt không có databases(): mở kho như bình thường */ }
  const s = await openStore();
  try { return await s.backupReminderDue(now); } finally { s.close(); }
}

/** Xin lưu trữ bền vững (`navigator.storage.persist()`, R-PWA-1). Kho tự gọi khi lưu hồ sơ đầu tiên. */
export async function requestPersistentStorage(): Promise<boolean> {
  const m = await loadStore();
  const s = await m.openStore();
  try { return await m.requestPersistentStorage(s); } finally { s.close(); }
}
