// Ảnh chân dung (01 §3.8, S9, S2b / R-CSP-1).
// - Thêm ảnh: giải mã bằng createImageBitmap (trình duyệt xoay theo EXIF), thu cạnh dài ≤ 1024 px, VẼ LẠI vào canvas
//   và mã hóa lại JPEG → byte lưu không còn EXIF/GPS/XMP/IPTC (canvas không chép siêu dữ liệu).
// - Hiển thị: vẽ lên <canvas> từ byte trong IndexedDB. Không dùng blob: URL cho <img> (CSP img-src 'self').
export const MAX_INPUT = 25 * 1024 * 1024;
export const MAX_STORED = 1024 * 1024;
export const MAX_EDGE = 1024;

export interface Processed { bytes: Uint8Array; width: number; height: number; mime: "image/jpeg" }

export class PhotoError extends Error {}

/** MA3-15-07: giới hạn kích thước điểm ảnh trước khi giải mã (ảnh nén nhỏ vẫn có thể giải mã ra hàng GB). */
export const MAX_SIDE = 12_000;
export const MAX_PIXELS = 50_000_000;

/**
 * Đọc chiều rộng × cao từ phần đầu tệp (PNG IHDR, GIF, JPEG SOFn, WebP VP8/VP8L/VP8X) mà không giải mã ảnh.
 * Trả null nếu không nhận ra định dạng (khi đó trình duyệt tự giải mã như cũ).
 */
export function sniffSize(b: Uint8Array): { w: number; h: number } | null {
  const u16be = (i: number) => (b[i] << 8) | b[i + 1];
  const u32be = (i: number) => ((b[i] << 24) >>> 0) + (b[i + 1] << 16) + (b[i + 2] << 8) + b[i + 3];
  const u16le = (i: number) => b[i] | (b[i + 1] << 8);
  const u24le = (i: number) => b[i] | (b[i + 1] << 8) | (b[i + 2] << 16);
  if (b.length >= 24 && b[0] === 0x89 && b[1] === 0x50 && b[2] === 0x4e && b[3] === 0x47) return { w: u32be(16), h: u32be(20) };
  if (b.length >= 10 && b[0] === 0x47 && b[1] === 0x49 && b[2] === 0x46) return { w: u16le(6), h: u16le(8) };
  if (b.length >= 30 && b[0] === 0x52 && b[1] === 0x49 && b[2] === 0x46 && b[3] === 0x46 && b[8] === 0x57 && b[9] === 0x45 && b[10] === 0x42 && b[11] === 0x50) {
    const fourcc = String.fromCharCode(b[12], b[13], b[14], b[15]);
    if (fourcc === "VP8X") return { w: u24le(24) + 1, h: u24le(27) + 1 };
    if (fourcc === "VP8L") { const v = b[21] | (b[22] << 8) | (b[23] << 16) | (b[24] << 24); return { w: (v & 0x3fff) + 1, h: ((v >> 14) & 0x3fff) + 1 }; }
    if (fourcc === "VP8 ") return { w: u16le(26) & 0x3fff, h: u16le(28) & 0x3fff };
    return null;
  }
  if (b.length >= 4 && b[0] === 0xff && b[1] === 0xd8) {
    let i = 2;
    while (i + 9 < b.length && b[i] === 0xff) {
      const m = b[i + 1];
      if (m >= 0xc0 && m <= 0xcf && m !== 0xc4 && m !== 0xc8 && m !== 0xcc) return { w: u16be(i + 7), h: u16be(i + 5) };
      if (m === 0xd8 || (m >= 0xd0 && m <= 0xd7)) { i += 2; continue; }
      i += 2 + u16be(i + 2);
    }
  }
  return null;
}

function toBlob(c: HTMLCanvasElement, q: number): Promise<Blob | null> {
  return new Promise((ok) => c.toBlob(ok, "image/jpeg", q));
}

/** Giới hạn cho ảnh tệp đính kèm báo tử (01 §3.8a): cạnh dài ≤ 2048 px để giấy tờ còn đọc được, ≤ 2 MB. */
export const ATTACHMENT_LIMITS = { maxEdge: 2048, maxStored: 2 * 1024 * 1024 } as const;

export async function processPhoto(file: Blob, lim: { maxEdge: number; maxStored: number } = { maxEdge: MAX_EDGE, maxStored: MAX_STORED }): Promise<Processed> {
  if (file.size > MAX_INPUT) throw new PhotoError("Ảnh lớn hơn 25 MB — hãy chọn ảnh nhỏ hơn");
  // Chỉ đọc 256 KB đầu để tìm kích thước điểm ảnh (đủ cho JPEG có EXIF/ảnh nhỏ kèm theo).
  const head = new Uint8Array(await file.slice(0, 256 * 1024).arrayBuffer());
  const dim = sniffSize(head);
  if (dim && (dim.w > MAX_SIDE || dim.h > MAX_SIDE || dim.w * dim.h > MAX_PIXELS)) {
    throw new PhotoError(`Ảnh quá lớn (${dim.w} × ${dim.h} điểm ảnh) — hãy chọn ảnh nhỏ hơn`);
  }
  let bmp: ImageBitmap;
  try {
    bmp = await createImageBitmap(file, { imageOrientation: "from-image" } as ImageBitmapOptions);
  } catch {
    throw new PhotoError("Không đọc được ảnh này (định dạng không hỗ trợ)");
  }
  const scale = Math.min(1, lim.maxEdge / Math.max(bmp.width, bmp.height));
  const w = Math.max(1, Math.round(bmp.width * scale)), hh = Math.max(1, Math.round(bmp.height * scale));
  const c = document.createElement("canvas");
  c.width = w; c.height = hh;
  const ctx = c.getContext("2d")!;
  ctx.fillStyle = "#fff";
  ctx.fillRect(0, 0, w, hh);
  ctx.drawImage(bmp, 0, 0, w, hh);
  bmp.close();
  for (const q of [0.86, 0.75, 0.62, 0.5]) {
    const b = await toBlob(c, q);
    if (!b) break;
    if (b.size <= lim.maxStored) return { bytes: new Uint8Array(await b.arrayBuffer()), width: w, height: hh, mime: "image/jpeg" };
  }
  throw new PhotoError(`Ảnh quá chi tiết, chưa nén được dưới ${Math.round(lim.maxStored / (1024 * 1024))} MB`);
}

/** Vẽ ảnh (cắt vừa khung, giữ tỷ lệ) lên canvas. */
export async function drawPhoto(canvas: HTMLCanvasElement, bytes: Uint8Array, mime = "image/jpeg", contain = false): Promise<void> {
  const bmp = await createImageBitmap(new Blob([bytes as Uint8Array<ArrayBuffer>], { type: mime }));
  const dpr = Math.min(2, window.devicePixelRatio || 1);
  const cw = canvas.clientWidth || Number(canvas.getAttribute("width")) || 160;
  const ch = canvas.clientHeight || Number(canvas.getAttribute("height")) || 200;
  canvas.width = Math.round(cw * dpr); canvas.height = Math.round(ch * dpr);
  const ctx = canvas.getContext("2d")!;
  // `contain`: giấy tờ (giấy báo tử) — thấy cả tờ, không cắt.
  const s = (contain ? Math.min : Math.max)(canvas.width / bmp.width, canvas.height / bmp.height);
  const dw = bmp.width * s, dh = bmp.height * s;
  if (contain) ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.drawImage(bmp, (canvas.width - dw) / 2, contain ? (canvas.height - dh) / 2 : (canvas.height - dh) / 3, dw, dh);
  bmp.close();
  canvas.dataset.drawn = "1";
}

/** Byte JPEG có đoạn APP1 (EXIF/XMP) hoặc APP13 (IPTC) không — dùng cho kiểm thử S9. */
export function hasMetadataSegments(b: Uint8Array): boolean {
  let i = 2;
  while (i + 4 < b.length && b[i] === 0xff) {
    const marker = b[i + 1];
    if (marker === 0xda) break;
    if (marker === 0xe1 || marker === 0xed) return true;
    i += 2 + ((b[i + 2] << 8) | b[i + 3]);
  }
  return false;
}
