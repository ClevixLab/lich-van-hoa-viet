// Tệp sao lưu mã hóa — bản JavaScript của backup/src/lib.rs (docs/v0.2/03 §2). TẢI LƯỜI: chỉ được nạp
// bằng `import()` khi người dùng xuất/nhập sao lưu (tiêu chí P6) — không mô-đun nào của lần tải đầu được
// import tĩnh tệp này.
//
// Không tự viết mật mã (S8): Argon2id = @noble/hashes/argon2 (CHƯA có kiểm toán độc lập — O-8, xem 03 §4.4),
// XChaCha20-Poly1305 = @noble/ciphers/chacha (Cure53 2024 cho bản 0.6.0/1.0.0). Số ngẫu nhiên:
// crypto.getRandomValues. Thứ tự kiểm tra và mã lỗi giống hệt bản Rust.
import { xchacha20poly1305 } from "@noble/ciphers/chacha.js";
import { argon2idAsync } from "@noble/hashes/argon2.js";

export const MAGIC = new Uint8Array([0x4c, 0x56, 0x48, 0x56, 0x42, 0x41, 0x4b, 0x00]); // "LVHVBAK\0"
export const PLAINTEXT_MAGIC = new Uint8Array([0x4c, 0x56, 0x48, 0x56, 0x50, 0x54, 0x30, 0x31]); // "LVHVPT01"
export const FILE_VERSION = 1;
export const HEADER_LEN = 72;
export const TAG_LEN = 16;
export const SALT_LEN = 16;
export const NONCE_LEN = 24;
export const DEFAULT_M_COST_KIB = 65536;
export const DEFAULT_T_COST = 3;
export const DEFAULT_P_COST = 4;
export const MAX_PLAINTEXT = 128 * 1024 * 1024;
export const MAX_JSON = 64 * 1024 * 1024;
export const MAX_PHOTO = 1024 * 1024;
export const MAX_PHOTOS = 10_000;
export const MIN_PASSWORD_CODEPOINTS = 8;

export type BackupErrorCode =
  | "not_a_backup" | "newer_version" | "unsupported" | "bad_kdf_params" | "bad_length"
  | "wrong_password_or_corrupt" | "malformed_plaintext" | "password_too_short" | "too_large" | "rng" | "kdf";

const MESSAGES: Record<BackupErrorCode, string> = {
  not_a_backup: "Đây không phải tệp sao lưu của Lịch và Văn hóa Việt",
  newer_version: "Tệp được tạo bởi phiên bản mới hơn — hãy cập nhật ứng dụng",
  unsupported: "Tệp sao lưu không hợp lệ hoặc không được hỗ trợ",
  bad_kdf_params: "Tệp sao lưu không hợp lệ hoặc không được hỗ trợ",
  bad_length: "Tệp sao lưu không hợp lệ hoặc không được hỗ trợ",
  wrong_password_or_corrupt: "Sai mật khẩu hoặc tệp bị hỏng",
  malformed_plaintext: "Sai mật khẩu hoặc tệp bị hỏng",
  password_too_short: "Mật khẩu cần ít nhất 8 ký tự",
  too_large: "Quá nhiều ảnh để sao lưu trong một tệp",
  rng: "Lỗi hệ thống khi mã hóa",
  kdf: "Lỗi hệ thống khi mã hóa",
};

export class BackupError extends Error {
  constructor(public code: BackupErrorCode) { super(MESSAGES[code]); }
}

export interface Header {
  fileVersion: number; mCostKib: number; tCost: number; pCost: number;
  salt: Uint8Array; nonce: Uint8Array; plaintextLen: number;
}

export interface PhotoBlob { id: Uint8Array; bytes: Uint8Array }
export interface Contents { json: string; photos: PhotoBlob[] }

const dv = (b: Uint8Array) => new DataView(b.buffer, b.byteOffset, b.byteLength);
const eq = (a: Uint8Array, b: Uint8Array) => a.length === b.length && a.every((x, i) => x === b[i]);

/** 03 §2.2 — trước khi chạy Argon2. Cùng thứ tự với `parse_header` của Rust. */
export function parseHeader(file: Uint8Array): Header {
  if (file.length < HEADER_LEN + TAG_LEN || !eq(file.subarray(0, 8), MAGIC)) throw new BackupError("not_a_backup");
  const v = dv(file);
  const fileVersion = v.getUint16(8, true);
  if (fileVersion > FILE_VERSION) throw new BackupError("newer_version");
  if (fileVersion === 0 || file[10] !== 1 || file[11] !== 1) throw new BackupError("unsupported");
  const mCostKib = v.getUint32(12, true), tCost = v.getUint32(16, true), pCost = v.getUint32(20, true);
  if (mCostKib < 65_536 || mCostKib > 1_048_576 || tCost < 1 || tCost > 10 || pCost < 1 || pCost > 8) throw new BackupError("bad_kdf_params");
  const len = v.getBigUint64(64, true);
  if (len !== BigInt(file.length - HEADER_LEN - TAG_LEN) || len > BigInt(MAX_PLAINTEXT)) throw new BackupError("bad_length");
  return { fileVersion, mCostKib, tCost, pCost, salt: file.slice(24, 40), nonce: file.slice(40, 64), plaintextLen: Number(len) };
}

export function headerBytes(h: Header): Uint8Array {
  const b = new Uint8Array(HEADER_LEN);
  const v = dv(b);
  b.set(MAGIC, 0);
  v.setUint16(8, h.fileVersion, true);
  b[10] = 1; b[11] = 1;
  v.setUint32(12, h.mCostKib, true); v.setUint32(16, h.tCost, true); v.setUint32(20, h.pCost, true);
  b.set(h.salt, 24); b.set(h.nonce, 40);
  v.setBigUint64(64, BigInt(h.plaintextLen), true);
  return b;
}

/** Mật khẩu → UTF-8 của dạng NFC. */
export const normalizePassword = (pw: string) => new TextEncoder().encode(pw.normalize("NFC"));

export async function deriveKeyBytes(pw: Uint8Array, salt: Uint8Array, m: number, t: number, p: number, onProgress?: (x: number) => void): Promise<Uint8Array> {
  try {
    // maxmem: m KiB khối + chỗ dư (mặc định của noble là 1 GiB, đúng bằng trần m của 03 §2.2).
    return await argon2idAsync(pw, salt, { m, t, p, dkLen: 32, version: 0x13, maxmem: m * 1024 + (1 << 20), asyncTick: 10, onProgress });
  } catch { throw new BackupError("kdf"); }
}

export function uuidToBytes(s: string): Uint8Array | null {
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(s)) return null;
  const h = s.replace(/-/g, "");
  return Uint8Array.from({ length: 16 }, (_, i) => parseInt(h.slice(2 * i, 2 * i + 2), 16));
}
export function bytesToUuid(b: Uint8Array): string {
  const h = [...b].map((x) => x.toString(16).padStart(2, "0")).join("");
  return `${h.slice(0, 8)}-${h.slice(8, 12)}-${h.slice(12, 16)}-${h.slice(16, 20)}-${h.slice(20)}`;
}

/** 03 §2.3. */
export function pack(json: string, photos: PhotoBlob[]): Uint8Array {
  const j = new TextEncoder().encode(json);
  if (j.length > MAX_JSON || photos.length > MAX_PHOTOS || photos.some((p) => p.bytes.length > MAX_PHOTO)) throw new BackupError("too_large");
  const total = 8 + 4 + j.length + 4 + photos.reduce((s, p) => s + 20 + p.bytes.length, 0);
  if (total > MAX_PLAINTEXT) throw new BackupError("too_large");
  const out = new Uint8Array(total);
  const v = dv(out);
  out.set(PLAINTEXT_MAGIC, 0);
  v.setUint32(8, j.length, true);
  out.set(j, 12);
  let i = 12 + j.length;
  v.setUint32(i, photos.length, true); i += 4;
  for (const p of photos) {
    out.set(p.id, i); v.setUint32(i + 16, p.bytes.length, true); i += 20;
    out.set(p.bytes, i); i += p.bytes.length;
  }
  j.fill(0);
  return out;
}

export function unpack(pt: Uint8Array): Contents {
  const bad = () => new BackupError("malformed_plaintext");
  if (pt.length < 16 || !eq(pt.subarray(0, 8), PLAINTEXT_MAGIC)) throw bad();
  const v = dv(pt);
  const jl = v.getUint32(8, true);
  if (jl > MAX_JSON || 12 + jl + 4 > pt.length) throw bad();
  let json: string;
  try { json = new TextDecoder("utf-8", { fatal: true }).decode(pt.subarray(12, 12 + jl)); } catch { throw bad(); }
  let i = 12 + jl;
  const count = v.getUint32(i, true); i += 4;
  if (count > MAX_PHOTOS) throw bad();
  const photos: PhotoBlob[] = [];
  for (let n = 0; n < count; n++) {
    if (i + 20 > pt.length) throw bad();
    const id = pt.slice(i, i + 16);
    const len = v.getUint32(i + 16, true); i += 20;
    if (len > MAX_PHOTO || i + len > pt.length) throw bad();
    photos.push({ id, bytes: pt.slice(i, i + len) }); i += len;
  }
  if (i !== pt.length) throw bad();
  return { json, photos };
}

function seal(key: Uint8Array, h: Header, pt: Uint8Array): Uint8Array {
  const hdr = headerBytes(h);
  const ct = xchacha20poly1305(key, h.nonce, hdr).encrypt(pt);
  const out = new Uint8Array(HEADER_LEN + ct.length);
  out.set(hdr, 0); out.set(ct, HEADER_LEN);
  return out;
}

/** Salt/nonce cố định — chỉ để tạo/kiểm vector (03 §2.5). */
export async function exportWithFixedRng(password: string, json: string, photos: PhotoBlob[], salt: Uint8Array, nonce: Uint8Array, onProgress?: (x: number) => void): Promise<Uint8Array> {
  const pw = normalizePassword(password);
  try {
    if ([...password.normalize("NFC")].length < MIN_PASSWORD_CODEPOINTS) throw new BackupError("password_too_short");
    const pt = pack(json, photos);
    const h: Header = { fileVersion: FILE_VERSION, mCostKib: DEFAULT_M_COST_KIB, tCost: DEFAULT_T_COST, pCost: DEFAULT_P_COST, salt, nonce, plaintextLen: pt.length };
    const key = await deriveKeyBytes(pw, salt, h.mCostKib, h.tCost, h.pCost, onProgress);
    try { return seal(key, h, pt); } finally { key.fill(0); pt.fill(0); }
  } finally { pw.fill(0); }
}

/** Xuất: salt và nonce mới từ crypto.getRandomValues (03 §1). */
export function exportBackup(password: string, json: string, photos: PhotoBlob[], onProgress?: (x: number) => void): Promise<Uint8Array> {
  let salt: Uint8Array, nonce: Uint8Array;
  try { salt = crypto.getRandomValues(new Uint8Array(SALT_LEN)); nonce = crypto.getRandomValues(new Uint8Array(NONCE_LEN)); } catch { throw new BackupError("rng"); }
  return exportWithFixedRng(password, json, photos, salt, nonce, onProgress);
}

/** Giải mã với khóa đã có (kiểm thử lật bit; kết quả giống `importBackup`). */
export function openWithKey(file: Uint8Array, h: Header, key: Uint8Array): Contents {
  let pt: Uint8Array;
  try { pt = xchacha20poly1305(key, h.nonce, file.subarray(0, HEADER_LEN)).decrypt(file.subarray(HEADER_LEN)); }
  catch { throw new BackupError("wrong_password_or_corrupt"); }
  try { return unpack(pt); } finally { pt.fill(0); }
}

/** Nhập: §2.2 → Argon2id (tham số trong tệp) → AEAD → mở gói. */
export async function importBackup(file: Uint8Array, password: string, onProgress?: (x: number) => void): Promise<Contents> {
  const h = parseHeader(file);
  const pw = normalizePassword(password);
  let key: Uint8Array | null = null;
  try {
    key = await deriveKeyBytes(pw, h.salt, h.mCostKib, h.tCost, h.pCost, onProgress);
    return openWithKey(file, h, key);
  } finally { pw.fill(0); key?.fill(0); }
}
