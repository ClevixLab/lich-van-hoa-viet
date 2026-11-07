// Kiểm tra chặt ảnh và tệp đính kèm (04 Bổ sung B6, BM2-01/BM2-08) — đọc phần đầu tệp, không giải mã. Giống hệt
// store/src/media.rs (cùng mã lỗi; kiểm chéo bằng docs/v0.2/schema/test-vectors/media/manifest.json).
export const MAX_EDGE = 2048;
export const MAX_PDF = 10 * 1024 * 1024;
export const PDF_EOF_WINDOW = 1024;

export type MediaCode = "bad_mime" | "mime_mismatch" | "bad_header" | "metadata" | "dimensions_mismatch" | "too_large" | "pdf_no_eof";

export class MediaError extends Error {
  constructor(public code: MediaCode) { super(code); }
}

const bad = (c: MediaCode): never => { throw new MediaError(c); };
const at = (b: Uint8Array, i: number): number => { if (i >= b.length) bad("bad_header"); return b[i]; };
const be16 = (b: Uint8Array, i: number) => (at(b, i) << 8) | at(b, i + 1);
const le16 = (b: Uint8Array, i: number) => at(b, i) | (at(b, i + 1) << 8);
const le24 = (b: Uint8Array, i: number) => le16(b, i) | (at(b, i + 2) << 16);
const le32 = (b: Uint8Array, i: number) => (le24(b, i) | (at(b, i + 3) << 24)) >>> 0;
const tag = (b: Uint8Array, i: number) => String.fromCharCode(b[i], b[i + 1], b[i + 2], b[i + 3]);

export interface Dims { width: number; height: number }

export function jpegDims(b: Uint8Array): Dims {
  if (b.length < 4 || b[0] !== 0xff || b[1] !== 0xd8 || b[2] !== 0xff) bad("mime_mismatch");
  let i = 2;
  let dims: Dims | null = null;
  for (;;) {
    if (i + 4 > b.length || b[i] !== 0xff) bad("bad_header");
    const m = b[i + 1];
    if (m === 0xff) { i += 1; continue; }
    if (m === 0xd8 || (m >= 0xd0 && m <= 0xd7) || m === 0x01) { i += 2; continue; }
    if (m === 0xd9) bad("bad_header");
    const len = be16(b, i + 2);
    if (len < 2 || i + 2 + len > b.length) bad("bad_header");
    if (m === 0xe1 || m === 0xed) bad("metadata");
    if (m >= 0xc0 && m <= 0xcf && m !== 0xc4 && m !== 0xc8 && m !== 0xcc) {
      if (len < 7) bad("bad_header");
      dims = { width: be16(b, i + 7), height: be16(b, i + 5) };
    }
    if (m === 0xda) return dims ?? bad("bad_header");
    i += 2 + len;
  }
}

export function webpDims(b: Uint8Array): Dims {
  if (b.length < 12 || tag(b, 0) !== "RIFF" || tag(b, 8) !== "WEBP") bad("mime_mismatch");
  const riff = le32(b, 4);
  if (riff + 8 > b.length || riff < 4) bad("bad_header");
  const end = riff + 8;
  let i = 12;
  let dims: Dims | null = null;
  while (i + 8 <= end) {
    const t = tag(b, i);
    const size = le32(b, i + 4);
    const data = i + 8;
    if (data + size > end) bad("bad_header");
    if (t === "EXIF" || t === "XMP ") bad("metadata");
    if (t === "VP8X") {
      if (size < 10) bad("bad_header");
      const flags = b[data];
      if (flags & 0x08 || flags & 0x04) bad("metadata");
      dims = { width: le24(b, data + 4) + 1, height: le24(b, data + 7) + 1 };
    } else if (t === "VP8L" && !dims) {
      if (size < 5 || b[data] !== 0x2f) bad("bad_header");
      const v = le32(b, data + 1);
      dims = { width: (v & 0x3fff) + 1, height: ((v >>> 14) & 0x3fff) + 1 };
    } else if (t === "VP8 " && !dims) {
      if (size < 10 || b[data + 3] !== 0x9d || b[data + 4] !== 0x01 || b[data + 5] !== 0x2a) bad("bad_header");
      dims = { width: le16(b, data + 6) & 0x3fff, height: le16(b, data + 8) & 0x3fff };
    }
    i = data + size + (size & 1);
  }
  if (!dims || dims.width <= 0 || dims.height <= 0) bad("bad_header");
  return dims!;
}

function imageDims(mime: string, b: Uint8Array): Dims {
  if (mime === "image/jpeg") return jpegDims(b);
  if (mime === "image/webp") return webpDims(b);
  return bad("bad_mime");
}

/** Ảnh chân dung: kích thước bằng bản ghi và ≤ 2048. Ném MediaError. */
export function checkPhoto(mime: string, bytes: Uint8Array, width: number, height: number): void {
  const d = imageDims(mime, bytes);
  if (d.width > MAX_EDGE || d.height > MAX_EDGE) bad("too_large");
  if (d.width !== width || d.height !== height) bad("dimensions_mismatch");
}

/** Tệp đính kèm: ảnh cạnh dài ≤ 2048; PDF `%PDF-` ở đầu, `%%EOF` trong 1024 byte cuối, ≤ 10 MiB. Ném MediaError. */
export function checkAttachment(mime: string, bytes: Uint8Array): void {
  if (mime === "application/pdf") {
    if (bytes.length > MAX_PDF) bad("too_large");
    if (!(bytes.length >= 5 && String.fromCharCode(...bytes.subarray(0, 5)) === "%PDF-")) bad("mime_mismatch");
    const tail = bytes.subarray(Math.max(0, bytes.length - PDF_EOF_WINDOW));
    let found = false;
    for (let i = 0; i + 5 <= tail.length && !found; i++) found = tail[i] === 0x25 && tail[i + 1] === 0x25 && tail[i + 2] === 0x45 && tail[i + 3] === 0x4f && tail[i + 4] === 0x46;
    if (!found) bad("pdf_no_eof");
    return;
  }
  const d = imageDims(mime, bytes);
  if (Math.max(d.width, d.height) > MAX_EDGE) bad("too_large");
}

/** Mã lỗi hoặc "ok" — cho kiểm chéo với Rust. */
export function mediaVerdict(kind: "photo" | "attachment", mime: string, bytes: Uint8Array, width?: number | null, height?: number | null): string {
  try {
    if (kind === "photo") checkPhoto(mime, bytes, width ?? 0, height ?? 0); else checkAttachment(mime, bytes);
    return "ok";
  } catch (e) {
    if (e instanceof MediaError) return e.code;
    throw e;
  }
}

/** Câu cho người dùng khi THÊM ảnh/tệp (như Rust `media_input`). */
export function mediaMessage(code: MediaCode, what: string): string {
  if (code === "metadata") return `${what}: ảnh chưa được xử lý bỏ siêu dữ liệu`;
  if (code === "too_large") return `${what}: tệp quá lớn`;
  if (code === "pdf_no_eof") return `${what}: tệp PDF không hoàn chỉnh`;
  return `${what}: tệp không đúng định dạng đã khai (${code})`;
}
