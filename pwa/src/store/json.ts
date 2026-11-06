// Kiểu JSON dùng chung của lớp lưu trữ PWA, và JSON chuẩn (khóa sắp theo thứ tự byte UTF-8 như
// serde_json/BTreeMap của bản Rust, không khoảng trắng) — hai bản sinh cùng từng byte.
export type Json = null | boolean | number | string | Json[] | JsonObj;
export interface JsonObj { [k: string]: Json }

/** So hai khóa theo thứ tự byte UTF-8 (= thứ tự điểm mã), không theo UTF-16 như `Array.sort()` mặc định. */
export function cmpCodepoint(a: string, b: string): number {
  const x = [...a], y = [...b];
  const n = Math.min(x.length, y.length);
  for (let i = 0; i < n; i++) {
    const d = x[i].codePointAt(0)! - y[i].codePointAt(0)!;
    if (d) return d;
  }
  return x.length - y.length;
}

/** JSON chuẩn: giống `serde_json::to_string(&Value)` của lvhv-store (khóa sắp, số nguyên, không khoảng trắng). */
export function canonicalJson(v: Json): string {
  if (v === null || typeof v !== "object") return JSON.stringify(v);
  if (Array.isArray(v)) return `[${v.map(canonicalJson).join(",")}]`;
  return `{${Object.keys(v).sort(cmpCodepoint).map((k) => `${JSON.stringify(k)}:${canonicalJson(v[k])}`).join(",")}}`;
}
