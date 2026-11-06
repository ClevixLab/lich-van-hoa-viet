// Port of memorial/src/json.rs (writer side) — compact and pretty JSON with the same string escaping
// as the Rust core (`writeJsonStr`). Parsing uses JSON.parse (key order is preserved for non-integer
// keys, which is all the v1 schema has); integers are checked by the model parser.

import { writeJsonStr } from "./json.ts";

export type JValue = null | boolean | number | string | JValue[] | { [k: string]: JValue };

export function jstr(s: string): string {
  const o: string[] = [];
  writeJsonStr(o, s);
  return o.join("");
}

export function writeCompact(v: JValue): string {
  if (v === null) return "null";
  if (typeof v === "boolean") return v ? "true" : "false";
  if (typeof v === "number") return String(v);
  if (typeof v === "string") return jstr(v);
  if (Array.isArray(v)) return "[" + v.map(writeCompact).join(",") + "]";
  return "{" + Object.keys(v).map((k) => jstr(k) + ":" + writeCompact(v[k])).join(",") + "}";
}

export function writePretty(v: JValue, indent = 0): string {
  const pad = (n: number) => "  ".repeat(n);
  if (Array.isArray(v) && v.length > 0) {
    return "[\n" + v.map((x) => pad(indent + 1) + writePretty(x, indent + 1)).join(",\n") + "\n" + pad(indent) + "]";
  }
  if (v !== null && typeof v === "object" && !Array.isArray(v) && Object.keys(v).length > 0) {
    const ks = Object.keys(v);
    return "{\n" + ks.map((k) => pad(indent + 1) + jstr(k) + ": " + writePretty(v[k], indent + 1)).join(",\n") + "\n" + pad(indent) + "}";
  }
  return writeCompact(v);
}
