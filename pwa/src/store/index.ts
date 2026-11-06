// API lưu trữ của PWA (đợt 2) — chỉ hàm, giao diện ở đợt 3. Mô-đun này được nạp LƯỜI từ main.ts
// (`import("./store/index")`), và tự nạp lười mô-đun mật mã (backup-crypto) khi xuất/nhập sao lưu (P6).
export { openStore, openBackup, verifyContents, requestPersistentStorage, backupDue, nowRfc3339, sha256Hex,
  Store, StoreError, DB_NAME, SCHEMA_VERSION, DEFAULT_SETTINGS, defaultEventTypes } from "./db";
export type { AppInfo, ImportMode, ImportPreview, VerifiedBackup, NewPhoto } from "./db";
export { buildPerson, saveDeath, deriveBirth, tuoiMat, canChiYear, InputError } from "./derive";
export { mergeDocs } from "./merge";
export { canonicalJson } from "./json";
export type { Json, JsonObj } from "./json";
export * as validate from "./validate";
