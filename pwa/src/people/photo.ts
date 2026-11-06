// Ảnh chân dung (01 §3.8, S9, S2b / R-CSP-1).
// - Thêm ảnh: giải mã bằng createImageBitmap (trình duyệt xoay theo EXIF), thu cạnh dài ≤ 1024 px, VẼ LẠI vào canvas
//   và mã hóa lại JPEG → byte lưu không còn EXIF/GPS/XMP/IPTC (canvas không chép siêu dữ liệu).
// - Hiển thị: vẽ lên <canvas> từ byte trong IndexedDB. Không dùng blob: URL cho <img> (CSP img-src 'self').
export const MAX_INPUT = 25 * 1024 * 1024;
export const MAX_STORED = 1024 * 1024;
export const MAX_EDGE = 1024;

export interface Processed { bytes: Uint8Array; width: number; height: number; mime: "image/jpeg" }

export class PhotoError extends Error {}

function toBlob(c: HTMLCanvasElement, q: number): Promise<Blob | null> {
  return new Promise((ok) => c.toBlob(ok, "image/jpeg", q));
}

export async function processPhoto(file: Blob): Promise<Processed> {
  if (file.size > MAX_INPUT) throw new PhotoError("Ảnh lớn hơn 25 MB — hãy chọn ảnh nhỏ hơn");
  let bmp: ImageBitmap;
  try {
    bmp = await createImageBitmap(file, { imageOrientation: "from-image" } as ImageBitmapOptions);
  } catch {
    throw new PhotoError("Không đọc được ảnh này (định dạng không hỗ trợ)");
  }
  const scale = Math.min(1, MAX_EDGE / Math.max(bmp.width, bmp.height));
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
    if (b.size <= MAX_STORED) return { bytes: new Uint8Array(await b.arrayBuffer()), width: w, height: hh, mime: "image/jpeg" };
  }
  throw new PhotoError("Ảnh quá chi tiết, chưa nén được dưới 1 MB");
}

/** Vẽ ảnh (cắt vừa khung, giữ tỷ lệ) lên canvas. */
export async function drawPhoto(canvas: HTMLCanvasElement, bytes: Uint8Array, mime = "image/jpeg"): Promise<void> {
  const bmp = await createImageBitmap(new Blob([bytes as Uint8Array<ArrayBuffer>], { type: mime }));
  const dpr = Math.min(2, window.devicePixelRatio || 1);
  const cw = canvas.clientWidth || Number(canvas.getAttribute("width")) || 160;
  const ch = canvas.clientHeight || Number(canvas.getAttribute("height")) || 200;
  canvas.width = Math.round(cw * dpr); canvas.height = Math.round(ch * dpr);
  const ctx = canvas.getContext("2d")!;
  const s = Math.max(canvas.width / bmp.width, canvas.height / bmp.height);
  const dw = bmp.width * s, dh = bmp.height * s;
  ctx.drawImage(bmp, (canvas.width - dw) / 2, (canvas.height - dh) / 3, dw, dh);
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
