// Chuyển đổi phiên bản dữ liệu (01 §7, 04 Bổ sung B5 / R14) — hàm thuần, ĐỒNG BỘ (chạy được trong
// `onupgradeneeded` của IndexedDB), giống hệt store/src/migrate.rs. Kiểm bằng bộ mẫu đóng băng
// docs/v0.2/schema/test-vectors/released/ (cùng tệp kỳ vọng với bản Rust).
import type { Json, JsonObj } from "./json";
import { ValidationError } from "./validate";

/** Phiên bản kho (như Rust STORE_VERSION): 1 = bản dựng 0.2 trước YC-2026-07, 2 = có tệp đính kèm. */
export const STORE_VERSION = 2;

export interface Migrated { doc: JsonObj; blob_sources: [string, string][]; moved_photos: string[]; changed: boolean }

export interface MigrationStatus {
  /** "ok" | "migrated" | "read_only" — như Rust MigrationStatus.state. */
  state: "ok" | "migrated" | "read_only";
  from_version: number;
  to_version: number;
  /** Câu lỗi khi chỉ đọc. */
  error: string | null;
}

const isObj = (v: Json | undefined): v is JsonObj => typeof v === "object" && v !== null && !Array.isArray(v);

export function hasCertificatePhotoIds(doc: JsonObj): boolean {
  return ((doc.persons as JsonObj[] | undefined) ?? []).some((p) => {
    const d = p.death;
    return isObj(d) && isObj(d.notice) && Object.prototype.hasOwnProperty.call(d.notice, "certificate_photo_ids");
  });
}

/** Như Rust `alt_id`: đảo 12 chữ số hex cuối; trùng thì tăng chữ số cuối (mod 16). */
export function altId(id: string, taken: Set<string>): string {
  const cut = Math.max(0, id.length - 12);
  const cand = [...id.slice(0, cut), ...[...id.slice(cut)].map((c) => (15 - (parseInt(c, 16) || 0)).toString(16))];
  for (;;) {
    const s = cand.join("");
    if (!taken.has(s)) return s;
    const last = cand.length - 1;
    cand[last] = (((parseInt(cand[last], 16) || 0) + 1) % 16).toString(16);
  }
}

/** 1 → 2 (YC-2026-07): `death.notice.certificate_photo_ids` → `death.notice.attachments` loại `giay_chung_tu`.
 *  Quy tắc như Rust `migrate_1_to_2` (xem chú thích ở đó). Tài liệu dị dạng → ném ValidationError (cùng mã, đường dẫn):
 *  X22 có cả `attachments` lẫn `certificate_photo_ids` (V1); M09 id không có ảnh (V3); không phải mảng chuỗi (V1). */
export function migrate1to2(input: JsonObj): Migrated {
  const doc = structuredClone(input);
  const photos = new Map(((doc.photos as JsonObj[] | undefined) ?? []).map((p) => [p.id as string, p] as const));
  const portrait = new Set(((doc.persons as JsonObj[] | undefined) ?? []).flatMap((p) => (typeof p.photo_id === "string" ? [p.photo_id] : [])));
  const taken = new Set(photos.keys());
  const atts: JsonObj[] = ((doc.attachments as JsonObj[] | undefined) ?? []).slice();
  for (const a of atts) taken.add(a.id as string);
  const memo = new Map<string, string>();
  const out: Migrated = { doc, blob_sources: [], moved_photos: [], changed: false };
  ((doc.persons as JsonObj[] | undefined) ?? []).forEach((p, pi) => {
    const d = p.death;
    if (!isObj(d) || !isObj(d.notice) || !Object.prototype.hasOwnProperty.call(d.notice, "certificate_photo_ids")) return;
    const n = d.notice;
    const old = n.certificate_photo_ids as Json;
    delete n.certificate_photo_ids;
    const np = `/persons/${pi}/death/notice`;
    if (Object.prototype.hasOwnProperty.call(n, "attachments")) throw new ValidationError("V1", np, "có cả attachments lẫn certificate_photo_ids (khóa cũ) — tài liệu dị dạng");
    if (!Array.isArray(old) || !old.every((x) => typeof x === "string")) throw new ValidationError("V1", `${np}/certificate_photo_ids`, "phải là mảng UUID");
    out.changed = true;
    const items: JsonObj[] = Array.isArray(n.attachments) ? (n.attachments as JsonObj[]).slice() : [];
    const originals: JsonObj[] = [];
    (old as string[]).forEach((pid, j) => {
      const ph = photos.get(pid);
      if (!ph) throw new ValidationError("V3", `${np}/certificate_photo_ids/${j}`, "ảnh không tồn tại");
      let aid = memo.get(pid);
      if (aid === undefined) {
        aid = portrait.has(pid) ? altId(pid, taken) : pid;
        taken.add(aid);
        atts.push({ id: aid, mime: ph.mime, bytes: ph.bytes, sha256: ph.sha256, stripped: true, file_name: null, created_at: ph.created_at });
        out.blob_sources.push([aid, pid]);
        if (aid === pid) out.moved_photos.push(pid);
        memo.set(pid, aid);
      }
      originals.push(structuredClone(ph));
      items.push({ id: pid, kind: "giay_chung_tu", file_id: aid });
    });
    n.attachments = items;
    const legacy = isObj(p.legacy) ? p.legacy : (p.legacy = {});
    legacy.certificate_photo_ids = old;
    legacy.certificate_photos = originals;
  });
  if (!out.changed) return out;
  const moved = new Set(out.moved_photos);
  if (Array.isArray(doc.photos)) doc.photos = (doc.photos as JsonObj[]).filter((p) => !moved.has(p.id as string));
  atts.sort((a, b) => ((a.id as string) < (b.id as string) ? -1 : (a.id as string) > (b.id as string) ? 1 : 0));
  if (atts.length) doc.attachments = atts; else delete doc.attachments;
  return out;
}
