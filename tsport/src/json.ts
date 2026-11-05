// Port of Rust `dayinfo::write_json_str`: UTF-8 emitted raw; only `"`, `\` and control
// characters (< 0x20) are escaped; others as \u00xx with lowercase hex.

export function writeJsonStr(out: string[], s: string): void {
  let r = '"';
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if (c === 0x22) r += '\\"';
    else if (c === 0x5c) r += "\\\\";
    else if (c === 0x0a) r += "\\n";
    else if (c === 0x0d) r += "\\r";
    else if (c === 0x09) r += "\\t";
    else if (c < 0x20) r += "\\u" + c.toString(16).padStart(4, "0");
    else r += s[i];
  }
  out.push(r + '"');
}
