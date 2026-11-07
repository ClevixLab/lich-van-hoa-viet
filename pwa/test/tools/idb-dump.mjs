// Chép nguyên kho IndexedDB `lvhv` ra JSON và dựng lại từ JSON (bộ mẫu đóng băng của bản web, 04 Bổ sung B5 / R14).
// Chạy TRONG trang (page.evaluate). Blob → { "$blob": base64, "type": mime }; khóa ngoài dòng (meta, settings) giữ nguyên.
// Định dạng tệp: { name, version, stores: [{ name, keyPath, autoIncrement, indexes: [...], records: [{ key, value }] }] }.

/** Mã chạy trong trang: trả tài liệu dump của kho `lvhv`. */
export const DUMP_IN_PAGE = `(async () => {
  const db = await new Promise((ok, ko) => { const r = indexedDB.open("lvhv"); r.onsuccess = () => ok(r.result); r.onerror = () => ko(r.error); });
  const b64 = async (blob) => { const b = new Uint8Array(await blob.arrayBuffer()); let s = ""; for (let i = 0; i < b.length; i += 0x8000) s += String.fromCharCode(...b.subarray(i, i + 0x8000)); return btoa(s); };
  const enc = async (v) => {
    if (v instanceof Blob) return { "$blob": await b64(v), type: v.type };
    if (Array.isArray(v)) { const o = []; for (const x of v) o.push(await enc(x)); return o; }
    if (v && typeof v === "object") { const o = {}; for (const k of Object.keys(v).sort()) o[k] = await enc(v[k]); return o; }
    return v;
  };
  const out = { name: db.name, version: db.version, stores: [] };
  for (const name of [...db.objectStoreNames].sort()) {
    const tx = db.transaction(name); const st = tx.objectStore(name);
    const keys = await new Promise((ok, ko) => { const r = st.getAllKeys(); r.onsuccess = () => ok(r.result); r.onerror = () => ko(r.error); });
    const vals = await new Promise((ok, ko) => { const r = st.getAll(); r.onsuccess = () => ok(r.result); r.onerror = () => ko(r.error); });
    const indexes = [...st.indexNames].sort().map((n) => { const ix = st.index(n); return { name: n, keyPath: ix.keyPath, unique: ix.unique, multiEntry: ix.multiEntry }; });
    const records = []; for (let i = 0; i < keys.length; i++) records.push({ key: keys[i], value: await enc(vals[i]) });
    out.stores.push({ name, keyPath: st.keyPath, autoIncrement: st.autoIncrement, indexes, records });
  }
  db.close();
  return out;
})()`;

/** Mã chạy trong trang: dựng kho `lvhv` từ dump (kho chưa có). Trả số bản ghi đã ghi. */
export const restoreInPage = (dump) => `(async () => {
  const dump = ${JSON.stringify(dump)};
  const dec = (v) => {
    if (v && typeof v === "object" && !Array.isArray(v) && "$blob" in v) { const s = atob(v["$blob"]); const b = new Uint8Array(s.length); for (let i = 0; i < s.length; i++) b[i] = s.charCodeAt(i); return new Blob([b], { type: v.type }); }
    if (Array.isArray(v)) return v.map(dec);
    if (v && typeof v === "object") { const o = {}; for (const k of Object.keys(v)) o[k] = dec(v[k]); return o; }
    return v;
  };
  const db = await new Promise((ok, ko) => {
    const r = indexedDB.open(dump.name, dump.version);
    r.onupgradeneeded = () => {
      for (const st of dump.stores) {
        const os = r.result.createObjectStore(st.name, st.keyPath === null ? { autoIncrement: st.autoIncrement } : { keyPath: st.keyPath, autoIncrement: st.autoIncrement });
        for (const ix of st.indexes) os.createIndex(ix.name, ix.keyPath, { unique: ix.unique, multiEntry: ix.multiEntry });
      }
    };
    r.onsuccess = () => ok(r.result); r.onerror = () => ko(r.error); r.onblocked = () => ko(new Error("blocked"));
  });
  let n = 0;
  for (const st of dump.stores) {
    if (!st.records.length) continue;
    await new Promise((ok, ko) => {
      const tx = db.transaction(st.name, "readwrite"); const os = tx.objectStore(st.name);
      for (const rec of st.records) { if (st.keyPath === null) os.put(dec(rec.value), rec.key); else os.put(dec(rec.value)); n++; }
      tx.oncomplete = ok; tx.onerror = () => ko(tx.error);
    });
  }
  db.close();
  return n;
})()`;
