// Gộp tài liệu sao lưu (docs/v0.2/01 §6.3) — bản TypeScript của `merge_docs` (store/src/lib.rs). Hàm thuần;
// kết quả còn phải qua validate.document. Mảng kết quả sắp theo `id` (như BTreeMap của bản Rust).
import type { Json, JsonObj } from "./json";
import { nameKey, personAttachmentRefs, personPhotoRefs } from "./validate";

const rows = (v: Json): JsonObj[] => (Array.isArray(v) ? (v as JsonObj[]) : []);
const byId = (v: Json) => new Map(rows(v).map((x) => [x.id as string, x] as const));
const sortedValues = (m: Map<string, JsonObj>) => [...m.keys()].sort().map((k) => m.get(k)!);

/** Bản nhập thắng: `rev` lớn hơn; bằng thì `updated_at` muộn hơn; vẫn bằng thì giữ bản trên máy. */
export function incomingWins(local: JsonObj, incoming: JsonObj): boolean {
  const lr = local.rev as number, ir = incoming.rev as number;
  if (ir !== lr) return ir > lr;
  return (incoming.updated_at as string) > (local.updated_at as string);
}

export function mergeDocs(local: JsonObj, incoming: JsonObj): JsonObj {
  const types = byId(local.event_types);
  const localKey = new Map([...types.values()].map((t) => [nameKey(t.ten as string), t.id as string] as const));
  const remap = new Map<string, string>();
  for (const id of [...byId(incoming.event_types).keys()].sort()) {
    const t = byId(incoming.event_types).get(id)!;
    const l = types.get(id);
    if (l) { if (incomingWins(l, t)) types.set(id, t); continue; }
    const lid = localKey.get(nameKey(t.ten as string));
    if (lid !== undefined) remap.set(id, lid); else types.set(id, t);
  }
  const mergeList = (k: string, fix: (x: JsonObj) => JsonObj): JsonObj[] => {
    const m = byId(local[k]);
    const inc = byId(incoming[k]);
    for (const id of [...inc.keys()].sort()) {
      const x = fix(inc.get(id)!);
      const l = m.get(id);
      if (!l || incomingWins(l, x)) m.set(id, x);
    }
    return sortedValues(m);
  };
  const persons = mergeList("persons", (x) => x);
  const events = mergeList("events", (e) => {
    const n = remap.get(e.type_id as string);
    return n === undefined ? e : { ...e, type_id: n };
  });
  const photos = byId(local.photos);
  const localPhotoIds = new Set(photos.keys());
  for (const [id, x] of byId(incoming.photos)) if (!photos.has(id)) photos.set(id, x);
  // Ảnh chân dung và ảnh mâm lễ (YC-2026-10) mà hồ sơ sau gộp còn dùng — như Rust merge_docs.
  const used = new Set(persons.flatMap((p) => personPhotoRefs(p)));
  for (const id of [...photos.keys()]) if (!used.has(id) && !localPhotoIds.has(id)) photos.delete(id);
  // Tệp đính kèm (YC-2026-07): như ảnh; khóa `attachments` chỉ có khi khác rỗng (như Rust merge_docs).
  const atts = byId(local.attachments);
  const localAttIds = new Set(atts.keys());
  for (const [id, x] of byId(incoming.attachments)) if (!atts.has(id)) atts.set(id, x);
  const usedAtt = new Set(persons.flatMap((p) => personAttachmentRefs(p)));
  for (const id of [...atts.keys()]) if (!usedAtt.has(id) && !localAttIds.has(id)) atts.delete(id);
  const { attachments: _a, ...rest } = local;
  void _a;
  return { ...rest, ...(atts.size ? { attachments: sortedValues(atts) } : {}), event_types: sortedValues(types), events, persons, photos: sortedValues(photos) };
}
