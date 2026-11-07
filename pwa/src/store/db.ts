// Lớp lưu trữ IndexedDB của PWA (docs/v0.2/01 §4.1) — cùng mô hình dữ liệu và cùng quy tắc kiểm tra với
// lvhv-store (SQLite). Chỉ API hàm; giao diện ở đợt 3.
//
// Cơ sở dữ liệu `lvhv`, phiên bản IndexedDB = IDB_VERSION (2; schema_version của tài liệu vẫn là 1). Kho:
//   persons      keyPath id                 — bản ghi Person nguyên dạng JSON
//   events       keyPath id, by_person      — Event; chỉ mục multiEntry trên person_ids
//   event_types  keyPath id, by_name_key    — { id, name_key, doc } (chỉ mục duy nhất, V7)
//   photos       keyPath id                 — { id, meta, blob }
//   attachments  keyPath id                 — { id, meta, blob } — tệp đính kèm báo tử (01 §3.8a, YC-2026-07)
//   settings     khóa cố định "app"         — AppSettings
//   meta         khóa ngoài                 — schema_version, created_with_app_version, persist, backup_pending_since
// Không dùng localStorage/sessionStorage/Cache Storage cho dữ liệu cá nhân (01 §4.1, S4').
import { canonicalJson, type Json, type JsonObj } from "./json";
import { mergeDocs } from "./merge";
import { STORE_VERSION, hasCertificatePhotoIds, migrate1to2, type MigrationStatus } from "./migrate";
import * as V from "./validate";

export const DB_NAME = "lvhv";
export const SCHEMA_VERSION = 1;
/** Phiên bản IndexedDB (khác `SCHEMA_VERSION` của tài liệu sao lưu, vẫn 1 — vector đã khóa):
 *  1 = bản dựng 0.2 trước YC-2026-07 (`certificate_photo_ids`); 2 = bản dựng trung gian đã có kho `attachments` nhưng
 *  chưa chuyển dữ liệu; 3 = đã chuyển (04 B5). Tương ứng Rust STORE_VERSION 2 (`migrate.ts`). */
export const IDB_VERSION = 3;
export { STORE_VERSION };
export type { MigrationStatus };
const STORES = ["persons", "events", "event_types", "photos", "attachments", "settings", "meta"] as const;
type StoreName = (typeof STORES)[number];

/**
 * 01 §3.2a: `death.notice` vắng ≡ `null`. Bản Rust (chuẩn) đọc `null` thành `None` và KHÔNG ghi khóa khi `None`
 * (`skip_serializing_if`), nên ở đây cũng bỏ khóa `"notice": null` trước khi lưu — để `stateHash` và tệp xuất
 * giống Rust từng byte kể cả khi tài liệu nhập vào có ghi `null`.
 */
export function normNotice(p: JsonObj): JsonObj {
  const d = p.death as JsonObj | null;
  if (!d || typeof d !== "object" || !Object.prototype.hasOwnProperty.call(d, "notice") || d.notice !== null) return p;
  const { notice: _n, ...rest } = d;
  void _n;
  return { ...p, death: rest };
}

export class StoreError extends Error {
  constructor(public code: "invalid" | "input" | "newer_schema" | "not_found" | "backup" | "read_only", msg: string, public cause?: unknown) { super(msg); }
}

export interface AppInfo { version: string; platform: "pwa" }
export type ImportMode = "replace" | "merge";
export interface ImportPreview { persons: number; events: number; photos: number; exported_at: string; platform: string }
export interface VerifiedBackup { doc: JsonObj; photos: Map<string, Uint8Array>; attachments: Map<string, Uint8Array>; preview: ImportPreview }
export interface NewPhoto { id: string; mime: "image/jpeg" | "image/webp"; width: number; height: number; bytes: Uint8Array }
/** Tệp đính kèm mới (01 §3.8a): ảnh đã mã hóa lại (≤ 2 MiB, cạnh dài ≤ 2048 px) hoặc PDF nguyên bản (≤ 10 MiB). */
export interface NewAttachment { id: string; mime: "image/jpeg" | "image/webp" | "application/pdf"; file_name: string | null; bytes: Uint8Array }

/** Bản ghi `Attachment` cho byte mới — như Rust `add_attachment` (PDF phải bắt đầu bằng `%PDF-`). */
async function attachmentMeta(a: NewAttachment): Promise<JsonObj> {
  const pdf = a.mime === "application/pdf";
  if (pdf && !(a.bytes.length >= 5 && String.fromCharCode(...a.bytes.subarray(0, 5)) === "%PDF-")) {
    throw new StoreError("input", "Tệp không phải PDF hợp lệ (thiếu phần đầu %PDF-)");
  }
  const name = a.file_name === null ? null : a.file_name.normalize("NFC").trim() || null;
  const meta: JsonObj = { bytes: a.bytes.length, created_at: nowRfc3339(), file_name: name, id: a.id, mime: a.mime, sha256: await sha256Hex(a.bytes), stripped: !pdf };
  try { V.attachment(meta, "/attachment"); } catch (e) { throw vErr(e); }
  return meta;
}

export const DEFAULT_SETTINGS: JsonObj = {
  backup_nag_days: 30, default_offsets_days: [0, 3], default_time_local: "07:00", last_backup_at: null, reminders_enabled: true, upcoming_days: 30,
};
export const defaultEventTypes = (): JsonObj[] => V.BUILTIN_EVENT_TYPES.map(([b, id, ten, rec]) => ({
  builtin: b, created_at: "1970-01-01T00:00:00Z", default_recurring: rec, id, rev: 1, ten, updated_at: "1970-01-01T00:00:00Z",
}));

/** Thời điểm hiện tại, RFC 3339 UTC độ chính xác giây. */
export const nowRfc3339 = (d = new Date()) => d.toISOString().slice(0, 19) + "Z";

export async function sha256Hex(b: Uint8Array): Promise<string> {
  const h = new Uint8Array(await crypto.subtle.digest("SHA-256", b as Uint8Array<ArrayBuffer>));
  return [...h].map((x) => x.toString(16).padStart(2, "0")).join("");
}

/** Lời nhắc sao lưu đến hạn (01 §3.9) — cùng quy tắc `backup_due` của Rust. */
export function backupDue(nagDays: number, pendingSince: string | null, now: string): boolean {
  if (nagDays <= 0 || !pendingSince || !V.isTimestamp(pendingSince) || !V.isTimestamp(now)) return false;
  return Date.parse(now) - Date.parse(pendingSince) >= nagDays * 86_400_000;
}

// ---------------- tiện ích IndexedDB ----------------

function req<T>(r: IDBRequest<T>): Promise<T> {
  return new Promise((ok, ko) => { r.onsuccess = () => ok(r.result); r.onerror = () => ko(r.error); });
}
function done(tx: IDBTransaction): Promise<void> {
  return new Promise((ok, ko) => { tx.oncomplete = () => ok(); tx.onerror = () => ko(tx.error); tx.onabort = () => ko(tx.error ?? new DOMException("abort", "AbortError")); });
}

function upgrade(db: IDBDatabase, tx: IDBTransaction, from: number): void {
  if (from < 1) {
    db.createObjectStore("persons", { keyPath: "id" });
    const ev = db.createObjectStore("events", { keyPath: "id" });
    ev.createIndex("by_person", "person_ids", { multiEntry: true });
    const et = db.createObjectStore("event_types", { keyPath: "id" });
    et.createIndex("by_name_key", "name_key", { unique: true });
    db.createObjectStore("photos", { keyPath: "id" });
    db.createObjectStore("settings");
    db.createObjectStore("meta");
    tx.objectStore("settings").put(DEFAULT_SETTINGS, "app");
    tx.objectStore("meta").put(SCHEMA_VERSION, "schema_version");
    tx.objectStore("meta").put("0.2.0", "created_with_app_version");
    for (const t of defaultEventTypes()) tx.objectStore("event_types").put({ id: t.id, name_key: V.nameKey(t.ten as string), doc: t });
  }
  if (from < 2 && !db.objectStoreNames.contains("attachments")) db.createObjectStore("attachments", { keyPath: "id" });
  // Phiên bản sau: thêm nhánh `if (from < N)` với migrate_{N-1}_to_N (01 §7).
  if (from >= 1 && from < 3) migrateData1to2(tx);
}

/**
 * Bước dữ liệu 1 → 2 (YC-2026-07) trong giao dịch `versionchange`: đọc hồ sơ, ảnh, tệp đính kèm bằng yêu cầu
 * IndexedDB (không `await` — giao dịch nâng cấp tự kết thúc khi hết yêu cầu), áp `migrate1to2` (hàm thuần đồng bộ,
 * như Rust), ghi lại, rồi kiểm 01 §6.2. Lỗi → `abort()`: IndexedDB quay lui toàn bộ nâng cấp (giữ nguyên phiên bản và
 * dữ liệu cũ — chính là "bản sao dữ liệu cũ" của 01 §7 trên trình duyệt); openStore mở lại ở chế độ chỉ đọc.
 */
function migrateData1to2(tx: IDBTransaction): void {
  const all = <T>(name: StoreName, then: (rows: T[]) => void) => {
    const r = tx.objectStore(name).getAll();
    r.onsuccess = () => then(r.result as T[]);
  };
  all<JsonObj>("persons", (persons) => all<{ id: string; meta: JsonObj; blob: Blob }>("photos", (photos) =>
    all<{ id: string; meta: JsonObj; blob: Blob }>("attachments", (atts) => {
      try {
        const m = migrate1to2({ persons, photos: photos.map((x) => x.meta), attachments: atts.map((x) => x.meta) });
        if (!m.changed) return;
        const blob = new Map(photos.map((x) => [x.id, x.blob] as const));
        const metas = new Map(((m.doc.attachments as JsonObj[] | undefined) ?? []).map((a) => [a.id as string, a] as const));
        for (const [aid, pid] of m.blob_sources) {
          const b = blob.get(pid);
          if (!b) throw new Error(`thiếu byte ảnh ${pid}`);
          tx.objectStore("attachments").put({ id: aid, meta: metas.get(aid)!, blob: b });
        }
        for (const pid of m.moved_photos) tx.objectStore("photos").delete(pid);
        const photoIds = new Set(((m.doc.photos as JsonObj[] | undefined) ?? []).map((x) => x.id as string));
        for (const p of m.doc.persons as JsonObj[]) {
          V.person(p, "/person");
          for (const pid of V.personPhotoRefs(p)) if (!photoIds.has(pid)) throw new Error(`V3 ảnh ${pid}`);
          for (const aid of V.personAttachmentRefs(p)) if (!metas.has(aid)) throw new Error(`V3 tệp đính kèm ${aid}`);
          tx.objectStore("persons").put(normNotice(p));
        }
        for (const a of metas.values()) V.attachment(a, "/attachment");
        tx.objectStore("meta").put(STORE_VERSION, "store_version");
      } catch (e) {
        (tx as unknown as { __lvhvError?: string }).__lvhvError = (e as Error).message;
        tx.abort();
      }
    })));
}

/**
 * Mở (hoặc tạo) kho, chuyển đổi phiên bản nếu cần (01 §7, 04 B5). Kho có phiên bản lớn hơn bản này hiểu →
 * StoreError("newer_schema"). Chuyển đổi lỗi → mở lại kho cũ (đã quay lui) ở chế độ chỉ đọc; `migrationStatus()`
 * cho giao diện biết để hiện chế độ chỉ đọc và hướng dẫn xuất sao lưu.
 */
export function openStore(factory: IDBFactory = indexedDB): Promise<Store> {
  return new Promise((ok, ko) => {
    let r: IDBOpenDBRequest;
    let from = IDB_VERSION;
    let upgradeTx: IDBTransaction | null = null;
    try { r = factory.open(DB_NAME, IDB_VERSION); } catch (e) { ko(e); return; }
    r.onupgradeneeded = (e) => { from = e.oldVersion; upgradeTx = r.transaction!; upgrade(r.result, r.transaction!, e.oldVersion); };
    r.onsuccess = () => ok(new Store(r.result, { state: from > 0 && from < IDB_VERSION ? "migrated" : "ok", from_version: from, to_version: IDB_VERSION, error: null }));
    r.onerror = (ev) => {
      if (r.error?.name === "VersionError") { ko(new StoreError("newer_schema", "Dữ liệu được tạo bởi phiên bản mới hơn — hãy cập nhật ứng dụng")); return; }
      const msg = (upgradeTx as unknown as { __lvhvError?: string } | null)?.__lvhvError;
      if (msg === undefined || from === 0) { ko(r.error); return; }
      ev.preventDefault?.();
      // Nâng cấp đã quay lui: mở kho cũ (không chỉ định phiên bản → không nâng cấp) ở chế độ chỉ đọc.
      const r2 = factory.open(DB_NAME);
      r2.onsuccess = () => ok(new Store(r2.result, { state: "read_only", from_version: from, to_version: from, error: msg }));
      r2.onerror = () => ko(r2.error);
    };
    r.onblocked = () => ko(new StoreError("invalid", "Kho đang được mở ở thẻ khác"));
  });
}

const vErr = (e: unknown) => (e instanceof V.ValidationError ? new StoreError("invalid", e.message, e) : e);

export class Store {
  constructor(readonly db: IDBDatabase, private readonly status: MigrationStatus = { state: "ok", from_version: IDB_VERSION, to_version: IDB_VERSION, error: null }) {}

  close(): void { this.db.close(); }

  /** Trạng thái chuyển đổi khi mở kho (04 B5) — như Rust `migration_status`. */
  migrationStatus(): MigrationStatus { return { ...this.status }; }

  /** Chế độ chỉ đọc sau chuyển đổi lỗi: mọi thao tác ghi báo StoreError("read_only"). */
  private writable(): void {
    if (this.status.state === "read_only") throw new StoreError("read_only", "Dữ liệu đang ở chế độ chỉ đọc vì nâng cấp chưa xong — dữ liệu cũ vẫn còn nguyên; hãy xuất sao lưu");
  }

  private tx(names: StoreName[] | StoreName, mode: IDBTransactionMode = "readonly") { return this.db.transaction(names, mode); }

  async settings(): Promise<JsonObj> { return (await req(this.tx("settings").objectStore("settings").get("app"))) as JsonObj; }
  async meta(key: string): Promise<Json | undefined> { return (await req(this.tx("meta").objectStore("meta").get(key))) as Json | undefined; }
  async persons(): Promise<JsonObj[]> { return (await req(this.tx("persons").objectStore("persons").getAll())) as JsonObj[]; }
  async events(): Promise<JsonObj[]> { return (await req(this.tx("events").objectStore("events").getAll())) as JsonObj[]; }
  async eventTypes(): Promise<JsonObj[]> { return ((await req(this.tx("event_types").objectStore("event_types").getAll())) as { doc: JsonObj }[]).map((x) => x.doc); }
  async photos(): Promise<JsonObj[]> { return ((await req(this.tx("photos").objectStore("photos").getAll())) as { meta: JsonObj }[]).map((x) => x.meta); }
  async attachments(): Promise<JsonObj[]> {
    if (!this.db.objectStoreNames.contains("attachments")) return []; // kho phiên bản 1 mở chỉ đọc
    return ((await req(this.tx("attachments").objectStore("attachments").getAll())) as { meta: JsonObj }[]).map((x) => x.meta); }

  /** Byte một tệp đính kèm (kiểm `sha256` khi đọc). */
  async attachmentBytes(id: string): Promise<Uint8Array> {
    if (!this.db.objectStoreNames.contains("attachments")) throw new StoreError("not_found", "không tìm thấy");
    const r = (await req(this.tx("attachments").objectStore("attachments").get(id))) as { meta: JsonObj; blob: Blob } | undefined;
    if (!r) throw new StoreError("not_found", "không tìm thấy");
    const b = new Uint8Array(await r.blob.arrayBuffer());
    if ((await sha256Hex(b)) !== r.meta.sha256) throw new StoreError("invalid", "ATTACHMENT: sha256 không khớp");
    return b;
  }

  /** Thêm một tệp đính kèm chưa gắn hồ sơ (như Rust `add_attachment`). Trả bản ghi. */
  async addAttachment(a: NewAttachment): Promise<JsonObj> {
    this.writable();
    const meta = await attachmentMeta(a);
    const tx = this.tx(["attachments", "meta"], "readwrite");
    const fin = done(tx);
    tx.objectStore("attachments").put({ id: a.id, meta, blob: new Blob([a.bytes as Uint8Array<ArrayBuffer>], { type: a.mime }) });
    Store.markChanged(tx, nowRfc3339());
    await fin;
    return meta;
  }

  /** Xóa tệp đính kèm không hồ sơ nào dùng. Còn dùng → "input"; không có → "not_found". */
  async deleteAttachment(id: string): Promise<void> {
    this.writable();
    const has = await req(this.tx("attachments").objectStore("attachments").getKey(id));
    if (has === undefined) throw new StoreError("not_found", "không tìm thấy");
    if ((await this.persons()).some((x) => V.personAttachmentRefs(x).includes(id))) {
      throw new StoreError("input", "Tệp đang được dùng trong một hồ sơ nên chưa xóa được");
    }
    const tx = this.tx(["attachments", "meta"], "readwrite");
    const fin = done(tx);
    tx.objectStore("attachments").delete(id);
    Store.markChanged(tx, nowRfc3339());
    await fin;
  }
  async person(id: string): Promise<JsonObj> {
    const p = await req(this.tx("persons").objectStore("persons").get(id));
    if (!p) throw new StoreError("not_found", "không tìm thấy");
    return p as JsonObj;
  }
  async eventsOf(personId: string): Promise<JsonObj[]> {
    return (await req(this.tx("events").objectStore("events").index("by_person").getAll(personId))) as JsonObj[];
  }

  /** Byte ảnh (kiểm `sha256` khi đọc). Giao diện vẽ qua createImageBitmap → canvas (CSP không có blob:, R-CSP-1). */
  async photoBytes(id: string): Promise<Uint8Array> {
    const r = (await req(this.tx("photos").objectStore("photos").get(id))) as { meta: JsonObj; blob: Blob } | undefined;
    if (!r) throw new StoreError("not_found", "không tìm thấy");
    const b = new Uint8Array(await r.blob.arrayBuffer());
    if ((await sha256Hex(b)) !== r.meta.sha256) throw new StoreError("invalid", "PHOTO: sha256 không khớp");
    return b;
  }

  /** `true` nếu đã có thay đổi chưa sao lưu và đã quá `backup_nag_days` (hook nhắc sao lưu, 01 §3.9). */
  async backupReminderDue(now = nowRfc3339()): Promise<boolean> {
    const s = await this.settings();
    return backupDue(s.backup_nag_days as number, ((await this.meta("backup_pending_since")) as string | undefined) ?? null, now);
  }
  async backupPendingSince(): Promise<string | null> { return ((await this.meta("backup_pending_since")) as string | undefined) ?? null; }

  /** Kết quả `navigator.storage.persist()` đã ghi (null nếu chưa gọi). */
  async persistResult(): Promise<boolean | null> { return ((await this.meta("persist")) as boolean | undefined) ?? null; }

  private static markChanged(tx: IDBTransaction, now: string): void {
    const m = tx.objectStore("meta");
    const r = m.get("backup_pending_since");
    r.onsuccess = () => { if (r.result === undefined) m.put(now, "backup_pending_since"); };
  }

  /**
   * Ghi một Person (đã đủ trường dẫn xuất — derive.buildPerson) và, nếu có, ảnh mới của người đó, trong
   * **một** giao dịch (01 §4.1). Lần lưu hồ sơ đầu tiên gọi requestPersistentStorage().
   */
  async putPerson(p: JsonObj, newPhoto?: NewPhoto, newAttachments: NewAttachment[] = []): Promise<void> {
    this.writable();
    try { V.person(p, "/person"); } catch (e) { throw vErr(e); }
    // `newAttachments`: tệp đính kèm báo tử mới (01 §3.8a) — ghi cùng giao dịch với hồ sơ.
    const attRecs: { id: string; meta: JsonObj; blob: Blob }[] = [];
    for (const a of newAttachments) attRecs.push({ id: a.id, meta: await attachmentMeta(a), blob: new Blob([a.bytes as Uint8Array<ArrayBuffer>], { type: a.mime }) });
    const recs: { id: string; meta: JsonObj; blob: Blob }[] = [];
    for (const np of newPhoto ? [newPhoto] : []) {
      const meta: JsonObj = { bytes: np.bytes.length, created_at: nowRfc3339(), height: np.height, id: np.id, mime: np.mime, sha256: await sha256Hex(np.bytes), stripped: true, width: np.width };
      try { V.photo(meta, "/photo"); } catch (e) { throw vErr(e); }
      recs.push({ id: np.id, meta, blob: new Blob([np.bytes as Uint8Array<ArrayBuffer>], { type: np.mime }) });
    }
    const all = await this.persons();
    const first = all.length === 0;
    // Ảnh bản cũ dùng mà bản mới bỏ, không hồ sơ nào khác dùng → xóa cùng giao dịch (như Rust put_person).
    const old = all.find((x) => x.id === p.id);
    const keep = new Set([...V.personPhotoRefs(p), ...all.filter((x) => x.id !== p.id).flatMap((x) => V.personPhotoRefs(x))]);
    const gone = [...new Set(old ? V.personPhotoRefs(old) : [])].filter((x) => !keep.has(x));
    const keepAtt = new Set([...V.personAttachmentRefs(p), ...all.filter((x) => x.id !== p.id).flatMap((x) => V.personAttachmentRefs(x))]);
    const goneAtt = [...new Set(old ? V.personAttachmentRefs(old) : [])].filter((x) => !keepAtt.has(x));
    const tx = this.tx(["persons", "photos", "attachments", "meta"], "readwrite");
    const fin = done(tx);
    let missing = "V3 /person/photo_id: ảnh không tồn tại";
    for (const r of recs) tx.objectStore("photos").put(r);
    for (const r of attRecs) tx.objectStore("attachments").put(r);
    const fresh = new Set(recs.map((r) => r.id));
    for (const pid of V.personPhotoRefs(p)) {
      if (fresh.has(pid)) continue;
      const r = tx.objectStore("photos").getKey(pid);
      r.onsuccess = () => { if (r.result === undefined) { tx.abort(); } };
    }
    const freshAtt = new Set(attRecs.map((r) => r.id));
    V.personAttachmentRefs(p).forEach((aid, j) => {
      if (freshAtt.has(aid)) return;
      const r = tx.objectStore("attachments").getKey(aid);
      r.onsuccess = () => { if (r.result === undefined) { missing = `V3 /person/death/notice/attachments/${j}/file_id: tệp đính kèm không tồn tại`; tx.abort(); } };
    });
    tx.objectStore("persons").put(normNotice(p));
    for (const pid of gone) tx.objectStore("photos").delete(pid);
    for (const aid of goneAtt) tx.objectStore("attachments").delete(aid);
    Store.markChanged(tx, nowRfc3339());
    try { await fin; } catch (e) { throw new StoreError("invalid", missing, e); }
    if (first && (await this.persistResult()) === null) await requestPersistentStorage(this);
  }

  /** Ghi một Event (V1, V3, V4, V8). */
  async putEvent(e: JsonObj): Promise<void> {
    this.writable();
    try { V.event(e, "/event"); } catch (x) { throw vErr(x); }
    const tx = this.tx(["events", "event_types", "persons", "meta"], "readwrite");
    const fin = done(tx);
    let bad = false;
    const check = (store: "event_types" | "persons", key: string) => {
      const r = tx.objectStore(store).getKey(key);
      r.onsuccess = () => { if (r.result === undefined && !bad) { bad = true; tx.abort(); } };
    };
    check("event_types", e.type_id as string);
    for (const pid of e.person_ids as string[]) check("persons", pid);
    tx.objectStore("events").put(e);
    Store.markChanged(tx, nowRfc3339());
    try { await fin; } catch (x) { throw new StoreError("invalid", "V3 /event: tham chiếu không tồn tại", x); }
  }

  /** Ghi một EventType (V1, V7 — chỉ mục duy nhất by_name_key). */
  async putEventType(t: JsonObj): Promise<void> {
    this.writable();
    try { V.eventType(t, "/event_type"); } catch (x) { throw vErr(x); }
    const tx = this.tx(["event_types", "meta"], "readwrite");
    const fin = done(tx);
    tx.objectStore("event_types").put({ id: t.id, name_key: V.nameKey(t.ten as string), doc: t });
    Store.markChanged(tx, nowRfc3339());
    try { await fin; } catch (x) { throw new StoreError("invalid", "V7 /event_type/ten: tên trùng", x); }
  }

  async putSettings(s: JsonObj): Promise<void> {
    this.writable();
    try { V.settings(s, "/settings"); } catch (x) { throw vErr(x); }
    const tx = this.tx("settings", "readwrite");
    tx.objectStore("settings").put(s, "app");
    await done(tx);
  }

  /**
   * Xóa một người: gỡ id khỏi mọi sự kiện (01 §3.4), xóa ảnh chân dung và ảnh giấy báo tử (01 §3.2a) nếu không hồ
   * sơ nào khác còn dùng — một giao dịch.
   */
  async deletePerson(id: string, opts: { deleteOrphanEvents?: boolean } = {}): Promise<string[]> {
    this.writable();
    // opts.deleteOrphanEvents: như Rust `delete_person_with` — trong cùng giao dịch xóa sự kiện mà người này là người
    // duy nhất được gắn; trả id các sự kiện đã xóa.
    const removed: string[] = [];
    const now = nowRfc3339();
    const p = await this.person(id);
    const used = new Set((await this.persons()).filter((x) => x.id !== id).flatMap((x) => V.personPhotoRefs(x)));
    const gone = V.personPhotoRefs(p).filter((x) => !used.has(x));
    const usedAtt = new Set((await this.persons()).filter((x) => x.id !== id).flatMap((x) => V.personAttachmentRefs(x)));
    const goneAtt = [...new Set(V.personAttachmentRefs(p))].filter((x) => !usedAtt.has(x));
    const evs = await this.eventsOf(id);
    const tx = this.tx(["persons", "events", "photos", "attachments", "meta"], "readwrite");
    const fin = done(tx);
    for (const e of evs) {
      const left = (e.person_ids as string[]).filter((x) => x !== id);
      if (opts.deleteOrphanEvents && left.length === 0) { tx.objectStore("events").delete(e.id as string); removed.push(e.id as string); continue; }
      tx.objectStore("events").put({ ...e, person_ids: left, rev: (e.rev as number) + 1, updated_at: now });
    }
    tx.objectStore("persons").delete(id);
    for (const pid of gone) tx.objectStore("photos").delete(pid);
    for (const aid of goneAtt) tx.objectStore("attachments").delete(aid);
    Store.markChanged(tx, now);
    await fin;
    return removed.sort();
  }

  /** Xóa một ảnh không hồ sơ nào dùng (như Rust `delete_photo`). Còn được dùng → "input"; không có → "not_found". */
  async deletePhoto(id: string): Promise<void> {
    this.writable();
    const has = await req(this.tx("photos").objectStore("photos").getKey(id));
    if (has === undefined) throw new StoreError("not_found", "không tìm thấy");
    if ((await this.persons()).some((x) => V.personPhotoRefs(x).includes(id))) {
      throw new StoreError("input", "Ảnh đang được dùng trong một hồ sơ nên chưa xóa được");
    }
    const tx = this.tx(["photos", "meta"], "readwrite");
    const fin = done(tx);
    tx.objectStore("photos").delete(id);
    Store.markChanged(tx, nowRfc3339());
    await fin;
  }

  async deleteEvent(id: string): Promise<void> {
    this.writable();
    const tx = this.tx(["events", "meta"], "readwrite");
    const fin = done(tx);
    tx.objectStore("events").delete(id);
    Store.markChanged(tx, nowRfc3339());
    await fin;
  }

  // ---------------- ảnh chụp / sao lưu ----------------

  /** Toàn bộ kho dưới dạng tài liệu sao lưu (01 §5), chưa có byte ảnh. Cùng thứ tự với bản Rust. */
  async snapshot(app: AppInfo, exportedAt: string): Promise<JsonObj> {
    const byId = (a: JsonObj[]) => a.slice().sort((x, y) => ((x.id as string) < (y.id as string) ? -1 : 1));
    const atts = byId(await this.attachments());
    return {
      app: { name: "Lịch và Văn hóa Việt", platform: app.platform, version: app.version },
      // `attachments` chỉ có khi khác rỗng (như Rust) — tài liệu không có tệp đính kèm giữ nguyên từng byte.
      ...(atts.length ? { attachments: atts } : {}),
      event_types: byId(await this.eventTypes()),
      events: byId(await this.events()),
      exported_at: exportedAt,
      format: "lvhv-backup",
      persons: byId(await this.persons()),
      photos: byId(await this.photos()),
      schema_version: SCHEMA_VERSION,
      settings: await this.settings(),
    };
  }

  /** Băm SHA-256 toàn bộ kho — cùng định nghĩa `state_hash` của Rust (R6 (a), (a2), (b)). */
  async stateHash(): Promise<string> {
    const doc = await this.snapshot({ version: "0.0.0", platform: "pwa" }, "1970-01-01T00:00:00Z");
    delete doc.app; delete doc.exported_at;
    let s = canonicalJson(doc);
    for (const ph of doc.photos as JsonObj[]) s += await sha256Hex(await this.photoBytes(ph.id as string));
    for (const a of (doc.attachments as JsonObj[] | undefined) ?? []) s += await sha256Hex(await this.attachmentBytes(a.id as string));
    return sha256Hex(new TextEncoder().encode(s));
  }

  private async photoBlobs(doc: JsonObj, uuidToBytes: (s: string) => Uint8Array | null) {
    const out = [];
    for (const ph of doc.photos as JsonObj[]) out.push({ id: uuidToBytes(ph.id as string)!, bytes: await this.photoBytes(ph.id as string) });
    // Tệp đính kèm đi sau ảnh, cùng danh sách khối (03 §2.3).
    for (const a of (doc.attachments as JsonObj[] | undefined) ?? []) out.push({ id: uuidToBytes(a.id as string)!, bytes: await this.attachmentBytes(a.id as string) });
    return out;
  }

  /**
   * Xuất tệp sao lưu (03 §2.4). Nạp lười mô-đun mật mã. Sau khi xuất: `settings.last_backup_at = now` và xóa
   * trạng thái "chờ sao lưu". Trả byte tệp — giao diện (đợt 3) ghi qua tải về, không ghi tệp tạm bản rõ.
   */
  async exportBackup(password: string, app: AppInfo, onProgress?: (x: number) => void): Promise<Uint8Array> {
    const bc = await import("./backup-crypto");
    const now = nowRfc3339();
    const doc = await this.snapshot(app, now);
    const photos = await this.photoBlobs(doc, bc.uuidToBytes);
    const file = await bc.exportBackup(password, canonicalJson(doc), photos, onProgress);
    for (const p of photos) p.bytes.fill(0);
    // MA3N-23-01: chỉ ghi nhớ thời điểm tạo; "đã sao lưu" do markBackupSaved ghi sau khi tệp được lưu.
    // Chế độ chỉ đọc (04 B5): vẫn xuất được, không ghi gì.
    if (this.status.state !== "read_only") {
      const tx = this.tx("meta", "readwrite");
      tx.objectStore("meta").put(now, "last_export_at");
      await done(tx);
    }
    return file;
  }

  /**
   * Ghi nhận tệp vừa tạo bằng exportBackup đã được lưu (03 §2.4, MA3N-23-01) — giống `Store::mark_backup_saved` của
   * Rust: `last_backup_at = savedAt`; xóa "chờ sao lưu" nếu thay đổi chờ không muộn hơn lúc tạo tệp. Chưa tạo tệp → lỗi.
   */
  async markBackupSaved(savedAt: string): Promise<void> {
    this.writable();
    if (!V.isTimestamp(savedAt)) throw new StoreError("input", "Thời điểm không hợp lệ (RFC 3339 UTC).");
    const exported = (await this.meta("last_export_at")) as string | undefined;
    if (exported === undefined) throw new StoreError("input", "Chưa tạo tệp sao lưu nào để ghi nhận.");
    const pending = await this.backupPendingSince();
    const s = { ...(await this.settings()), last_backup_at: savedAt };
    const tx = this.tx(["settings", "meta"], "readwrite");
    tx.objectStore("settings").put(s, "app");
    if (pending !== null && pending <= exported) tx.objectStore("meta").delete("backup_pending_since");
    tx.objectStore("meta").delete("last_export_at");
    await done(tx);
  }

  /** Như exportBackup với salt/nonce/thời điểm cố định — chỉ cho vector kiểm thử. */
  async exportBackupFixed(password: string, app: AppInfo, exportedAt: string, salt: Uint8Array, nonce: Uint8Array): Promise<Uint8Array> {
    const bc = await import("./backup-crypto");
    const doc = await this.snapshot(app, exportedAt);
    return bc.exportWithFixedRng(password, canonicalJson(doc), await this.photoBlobs(doc, bc.uuidToBytes), salt, nonce);
  }

  /** Ghi tài liệu đã kiểm tra — Thay toàn bộ hoặc Gộp — trong **một** giao dịch. Lỗi → không ghi gì. */
  async applyImport(b: VerifiedBackup, mode: ImportMode): Promise<void> {
    this.writable();
    let target = b.doc;
    if (mode === "merge") {
      target = mergeDocs(await this.snapshot({ version: "0.0.0", platform: "pwa" }, "1970-01-01T00:00:00Z"), b.doc);
      try { V.document(target); } catch (e) { throw vErr(e); }
    }
    // Byte ảnh: ưu tiên bản đã có trên máy nếu cùng sha256 (gộp), nếu không lấy từ tệp nhập.
    const existing = new Map((await this.photos()).map((m) => [m.id as string, m.sha256 as string]));
    const recs: { id: string; meta: JsonObj; blob: Blob }[] = [];
    const keepBlob = new Set<string>();
    for (const m of target.photos as JsonObj[]) {
      const id = m.id as string;
      const bytes = b.photos.get(id);
      if (existing.get(id) === m.sha256 && (mode === "merge" || !bytes)) keepBlob.add(id);
      else if (bytes) recs.push({ id, meta: m, blob: new Blob([bytes as Uint8Array<ArrayBuffer>], { type: m.mime as string }) });
      else throw new StoreError("invalid", `PHOTO ${id}: thiếu byte ảnh`);
    }
    const keptPhotos: { id: string; meta: JsonObj; blob: Blob }[] = [];
    if (keepBlob.size) {
      const all = (await req(this.tx("photos").objectStore("photos").getAll())) as { id: string; meta: JsonObj; blob: Blob }[];
      for (const r of all) if (keepBlob.has(r.id)) keptPhotos.push(r);
    }
    // Tệp đính kèm: cùng quy tắc với ảnh.
    const existingAtt = new Map((await this.attachments()).map((m) => [m.id as string, m.sha256 as string]));
    const attRecs: { id: string; meta: JsonObj; blob: Blob }[] = [];
    const keepAtt = new Set<string>();
    for (const m of (target.attachments as JsonObj[] | undefined) ?? []) {
      const id = m.id as string;
      const bytes = b.attachments.get(id);
      if (existingAtt.get(id) === m.sha256 && (mode === "merge" || !bytes)) keepAtt.add(id);
      else if (bytes) attRecs.push({ id, meta: m, blob: new Blob([bytes as Uint8Array<ArrayBuffer>], { type: m.mime as string }) });
      else throw new StoreError("invalid", `ATTACHMENT ${id}: thiếu byte tệp đính kèm`);
    }
    if (keepAtt.size) {
      const all = (await req(this.tx("attachments").objectStore("attachments").getAll())) as { id: string; meta: JsonObj; blob: Blob }[];
      for (const r of all) if (keepAtt.has(r.id)) attRecs.push(r);
    }
    const tx = this.tx(["persons", "events", "event_types", "photos", "attachments", "settings", "meta"], "readwrite");
    const fin = done(tx);
    for (const n of ["persons", "events", "event_types", "photos", "attachments"] as const) tx.objectStore(n).clear();
    for (const r of [...keptPhotos, ...recs]) tx.objectStore("photos").put(r);
    for (const r of attRecs) tx.objectStore("attachments").put(r);
    for (const t of target.event_types as JsonObj[]) tx.objectStore("event_types").put({ id: t.id, name_key: V.nameKey(t.ten as string), doc: t });
    for (const p of target.persons as JsonObj[]) tx.objectStore("persons").put(normNotice(p));
    for (const e of target.events as JsonObj[]) tx.objectStore("events").put(e);
    if (mode === "replace") tx.objectStore("settings").put(target.settings, "app");
    Store.markChanged(tx, nowRfc3339());
    await fin;
  }
}

/** Giải mã + kiểm tra toàn bộ (01 §6.3) — không ghi gì. Nạp lười mô-đun mật mã. */
export async function openBackup(file: Uint8Array, password: string, onProgress?: (x: number) => void): Promise<VerifiedBackup> {
  const bc = await import("./backup-crypto");
  let c;
  try { c = await bc.importBackup(file, password, onProgress); } catch (e) { throw new StoreError("backup", (e as Error).message, e); }
  return verifyContents(c.json, c.photos.map((p) => ({ id: bc.bytesToUuid(p.id), bytes: p.bytes })));
}

/** Phần kiểm tra nội dung của openBackup (sau khi giải mã). Cùng thứ tự với `verify_contents` của Rust. */
export async function verifyContents(json: string, blobsIn: { id: string; bytes: Uint8Array }[]): Promise<VerifiedBackup> {
  let photos = blobsIn;
  const generic = (e: unknown) => new StoreError("invalid", "Sai mật khẩu hoặc tệp bị hỏng", e);
  let doc: JsonObj;
  try {
    doc = V.parseJson(json) as JsonObj;
    const sv = V.int((doc as JsonObj)?.schema_version ?? null);
    if (sv !== null && sv > SCHEMA_VERSION) throw new StoreError("newer_schema", "Tệp được tạo bởi phiên bản mới hơn — hãy cập nhật ứng dụng");
    // 04 B5 mục 5: tệp của bản dựng 0.2 trước YC-2026-07 → cùng hàm chuyển đổi thuần như Rust; xếp lại khối.
    if (hasCertificatePhotoIds(doc)) {
      const m = migrate1to2(doc);
      const src = new Map(photos.map((b) => [b.id, b.bytes] as const));
      const from = new Map(m.blob_sources);
      const list: { id: string; bytes: Uint8Array }[] = [];
      for (const [k, att] of [["photos", false], ["attachments", true]] as const) {
        for (const r of (m.doc[k] as JsonObj[] | undefined) ?? []) {
          const id = r.id as string;
          const b = src.get(att ? from.get(id) ?? id : id);
          if (b) list.push({ id, bytes: b });
        }
      }
      photos = list;
      doc = m.doc;
    }
    V.document(doc);
  } catch (e) { throw e instanceof StoreError ? e : generic(e); }
  const meta = new Map((doc.photos as JsonObj[]).map((p) => [p.id as string, p] as const));
  const attMeta = new Map(((doc.attachments as JsonObj[] | undefined) ?? []).map((p) => [p.id as string, p] as const));
  const out = new Map<string, Uint8Array>(), atts = new Map<string, Uint8Array>();
  const perr = (m: string) => generic(new V.ValidationError("PHOTO", "/photos", m));
  const aerr = (m: string) => generic(new V.ValidationError("ATTACHMENT", "/attachments", m));
  // Khối: ảnh trước, tệp đính kèm sau (03 §2.3) — cùng thứ tự kiểm với Rust.
  for (const b of photos) {
    const m = meta.get(b.id);
    if (m) {
      if (m.sha256 !== (await sha256Hex(b.bytes)) || m.bytes !== b.bytes.length) throw perr("sha256/bytes không khớp");
      if (atts.size) throw perr("ảnh phải đứng trước tệp đính kèm");
      if (out.has(b.id)) throw perr("ảnh trùng");
      out.set(b.id, b.bytes);
      continue;
    }
    const a = attMeta.get(b.id);
    if (!a) throw perr("ảnh thừa");
    if (a.sha256 !== (await sha256Hex(b.bytes)) || a.bytes !== b.bytes.length) throw aerr("sha256/bytes không khớp");
    if (a.mime === "application/pdf" && String.fromCharCode(...b.bytes.subarray(0, 5)) !== "%PDF-") throw aerr("PDF không hợp lệ");
    if (atts.has(b.id)) throw aerr("tệp đính kèm trùng");
    atts.set(b.id, b.bytes);
  }
  if (out.size !== meta.size) throw perr("thiếu ảnh");
  if (atts.size !== attMeta.size) throw aerr("thiếu tệp đính kèm");
  const app = doc.app as JsonObj;
  return { doc, photos: out, attachments: atts, preview: { persons: (doc.persons as Json[]).length, events: (doc.events as Json[]).length, photos: meta.size, exported_at: doc.exported_at as string, platform: app.platform as string } };
}

/**
 * Xin trình duyệt lưu dữ liệu bền vững (`navigator.storage.persist()`, rủi ro R-PWA-1) và ghi kết quả vào
 * `meta.persist`. Trả `true` nếu được cấp; `false` nếu bị từ chối hoặc trình duyệt không hỗ trợ — khi đó giao
 * diện phải nói "trình duyệt có thể xóa dữ liệu khi máy thiếu chỗ — hãy sao lưu" (01 §4.1).
 */
export async function requestPersistentStorage(store?: Store): Promise<boolean> {
  let granted = false;
  try {
    const sm = (globalThis.navigator as Navigator | undefined)?.storage;
    if (sm?.persist) granted = (await sm.persisted?.()) || (await sm.persist());
  } catch { granted = false; }
  if (store) {
    const tx = store.db.transaction("meta", "readwrite");
    tx.objectStore("meta").put(granted, "persist");
    await done(tx);
  }
  return granted;
}
