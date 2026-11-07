//! # lvhv-backup — tệp sao lưu mã hóa của Lịch và Văn hóa Việt
//!
//! Cài đặt đúng `docs/v0.2/03-SAO-LUU.md` §2 (định dạng tệp phiên bản 1). Không tự viết thuật toán mật mã:
//! chỉ ghép Argon2id (crate `argon2`, RFC 9106) và XChaCha20-Poly1305 (crate `chacha20poly1305`) theo cách
//! dùng tài liệu của hai crate khuyến nghị; khóa và bản rõ được xóa bằng `zeroize`.
//!
//! ```text
//! tệp = phần đầu (72 byte, AAD) ‖ XChaCha20-Poly1305(khóa = Argon2id(NFC(mật khẩu), salt, m, t, p), nonce, bản rõ)
//! bản rõ = "LVHVPT01" ‖ u32 json_len ‖ json ‖ u32 photo_count ‖ photo_count × (16 byte UUID ‖ u32 len ‖ byte ảnh)
//! ```
//! Mọi số nguyên là little-endian.
//!
//! Lớp này chỉ lo **vùng chứa** (container). Kiểm tra nội dung JSON (lược đồ, V2–V8, `sha256` từng ảnh —
//! 01 §6.3) do `lvhv-store` làm sau khi giải mã.
#![forbid(unsafe_code)]

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
use unicode_normalization::UnicodeNormalization;
use zeroize::{Zeroize, Zeroizing};

/// `magic`: ASCII `LVHVBAK` + `0x00`.
pub const MAGIC: [u8; 8] = *b"LVHVBAK\0";
/// Phiên bản tệp mà bản cài đặt này viết và đọc.
pub const FILE_VERSION: u16 = 1;
/// `kdf_id` = 1: Argon2id, phiên bản thuật toán 0x13.
pub const KDF_ARGON2ID: u8 = 1;
/// `aead_id` = 1: XChaCha20-Poly1305.
pub const AEAD_XCHACHA20POLY1305: u8 = 1;
/// Độ dài phần đầu.
pub const HEADER_LEN: usize = 72;
/// Độ dài thẻ xác thực Poly1305.
pub const TAG_LEN: usize = 16;
/// Độ dài salt Argon2id.
pub const SALT_LEN: usize = 16;
/// Độ dài nonce XChaCha20.
pub const NONCE_LEN: usize = 24;
/// Độ dài khóa.
pub const KEY_LEN: usize = 32;

/// Tham số mặc định khi xuất (RFC 9106 §4, phương án thứ hai): 64 MiB, t = 3, p = 4.
pub const DEFAULT_M_COST_KIB: u32 = 65_536;
/// Số vòng mặc định.
pub const DEFAULT_T_COST: u32 = 3;
/// Số làn mặc định.
pub const DEFAULT_P_COST: u32 = 4;

/// Khoảng tham số chấp nhận khi nhập (03 §2.2).
pub const M_COST_RANGE: core::ops::RangeInclusive<u32> = 65_536..=1_048_576;
/// Khoảng `t_cost` chấp nhận khi nhập.
pub const T_COST_RANGE: core::ops::RangeInclusive<u32> = 1..=10;
/// Khoảng `p_cost` chấp nhận khi nhập.
pub const P_COST_RANGE: core::ops::RangeInclusive<u32> = 1..=8;

/// Tổng bản rõ tối đa: 128 MiB.
pub const MAX_PLAINTEXT: u64 = 128 * 1024 * 1024;
/// JSON tối đa: 64 MiB.
pub const MAX_JSON: usize = 64 * 1024 * 1024;
/// Mỗi ảnh tối đa: 1 MiB (01 §3.8). Kiểm ở lớp lưu trữ (bản ghi `Photo.bytes`).
pub const MAX_PHOTO: usize = 1024 * 1024;
/// Mỗi khối trong vùng chứa tối đa: 10 MiB — tệp đính kèm PDF (01 §3.8a, YC-2026-07) là khối lớn nhất. Giới hạn
/// riêng của từng loại (ảnh 1 MiB, ảnh đính kèm 2 MiB, PDF 10 MiB) kiểm ở lớp lưu trữ theo bản ghi JSON.
pub const MAX_BLOB: usize = 10 * 1024 * 1024;
/// Số ảnh tối đa (lược đồ: `photos.maxItems`).
pub const MAX_PHOTOS: usize = 10_000;
/// Số khối tối đa trong vùng chứa: ảnh + tệp đính kèm (mỗi loại ≤ 10 000).
pub const MAX_BLOBS: usize = 20_000;
/// Độ dài mật khẩu tối thiểu khi xuất (điểm mã sau NFC).
pub const MIN_PASSWORD_CODEPOINTS: usize = 8;
/// Dấu mở đầu bản rõ.
pub const PLAINTEXT_MAGIC: [u8; 8] = *b"LVHVPT01";

/// Lỗi của lớp sao lưu. Thông báo cho người dùng: [`BackupError::user_message`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupError {
    /// `magic` sai hoặc tệp quá ngắn.
    NotABackup,
    /// `file_version` lớn hơn phiên bản hiểu được.
    NewerVersion,
    /// `file_version` = 0, `kdf_id` hoặc `aead_id` không hỗ trợ.
    Unsupported,
    /// Tham số KDF ngoài khoảng cho phép (03 §2.2).
    BadKdfParams,
    /// `plaintext_len` không khớp kích thước tệp hoặc vượt 128 MiB.
    BadLength,
    /// AEAD không xác thực: sai mật khẩu hoặc tệp bị sửa (không phân biệt — 03 §2.4).
    WrongPasswordOrCorrupt,
    /// Bản rõ đã xác thực nhưng sai cấu trúc §2.3.
    MalformedPlaintext,
    /// Khi xuất: mật khẩu ngắn hơn 8 điểm mã sau NFC.
    PasswordTooShort,
    /// Khi xuất: vượt giới hạn kích thước (03 §2.3).
    TooLarge,
    /// Bộ sinh ngẫu nhiên của hệ điều hành lỗi.
    Rng,
    /// Lỗi nội bộ của thư viện KDF (không xảy ra với tham số đã kiểm tra).
    Kdf,
}

impl BackupError {
    /// Thông báo tiếng Việt cho người dùng (03 §2.2, §2.4).
    pub fn user_message(self) -> &'static str {
        match self {
            BackupError::NotABackup => "Đây không phải tệp sao lưu của Lịch và Văn hóa Việt",
            BackupError::NewerVersion => {
                "Tệp được tạo bởi phiên bản mới hơn — hãy cập nhật ứng dụng"
            }
            BackupError::Unsupported | BackupError::BadKdfParams | BackupError::BadLength => {
                "Tệp sao lưu không hợp lệ hoặc không được hỗ trợ"
            }
            BackupError::WrongPasswordOrCorrupt | BackupError::MalformedPlaintext => {
                "Sai mật khẩu hoặc tệp bị hỏng"
            }
            BackupError::PasswordTooShort => "Mật khẩu cần ít nhất 8 ký tự",
            BackupError::TooLarge => "Quá nhiều ảnh để sao lưu trong một tệp",
            BackupError::Rng | BackupError::Kdf => "Lỗi hệ thống khi mã hóa",
        }
    }

    /// Mã ổn định (dùng chung với bản JavaScript và trong tệp vector kiểm thử).
    pub fn code(self) -> &'static str {
        match self {
            BackupError::NotABackup => "not_a_backup",
            BackupError::NewerVersion => "newer_version",
            BackupError::Unsupported => "unsupported",
            BackupError::BadKdfParams => "bad_kdf_params",
            BackupError::BadLength => "bad_length",
            BackupError::WrongPasswordOrCorrupt => "wrong_password_or_corrupt",
            BackupError::MalformedPlaintext => "malformed_plaintext",
            BackupError::PasswordTooShort => "password_too_short",
            BackupError::TooLarge => "too_large",
            BackupError::Rng => "rng",
            BackupError::Kdf => "kdf",
        }
    }
}

impl core::fmt::Display for BackupError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.user_message())
    }
}

impl std::error::Error for BackupError {}

/// Phần đầu đã đọc và đã qua kiểm tra §2.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// `file_version`.
    pub file_version: u16,
    /// `m_cost_kib`.
    pub m_cost_kib: u32,
    /// `t_cost`.
    pub t_cost: u32,
    /// `p_cost`.
    pub p_cost: u32,
    /// Salt Argon2id.
    pub salt: [u8; SALT_LEN],
    /// Nonce XChaCha20.
    pub nonce: [u8; NONCE_LEN],
    /// Độ dài bản rõ.
    pub plaintext_len: u64,
}

impl Header {
    /// 72 byte phần đầu (cũng là AAD).
    pub fn to_bytes(&self) -> [u8; HEADER_LEN] {
        let mut h = [0u8; HEADER_LEN];
        h[0..8].copy_from_slice(&MAGIC);
        h[8..10].copy_from_slice(&self.file_version.to_le_bytes());
        h[10] = KDF_ARGON2ID;
        h[11] = AEAD_XCHACHA20POLY1305;
        h[12..16].copy_from_slice(&self.m_cost_kib.to_le_bytes());
        h[16..20].copy_from_slice(&self.t_cost.to_le_bytes());
        h[20..24].copy_from_slice(&self.p_cost.to_le_bytes());
        h[24..40].copy_from_slice(&self.salt);
        h[40..64].copy_from_slice(&self.nonce);
        h[64..72].copy_from_slice(&self.plaintext_len.to_le_bytes());
        h
    }
}

fn u16_at(b: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([b[i], b[i + 1]])
}
fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
}
fn u64_at(b: &[u8], i: usize) -> u64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(&b[i..i + 8]);
    u64::from_le_bytes(a)
}

/// Kiểm tra §2.2 — **trước khi** chạy Argon2 (không cấp bộ nhớ lớn, không tốn CPU).
///
/// Thứ tự kiểm tra cố định (bản JavaScript theo đúng thứ tự này để trả cùng mã lỗi):
/// độ dài tối thiểu và `magic` → `file_version` → `kdf_id`/`aead_id` → tham số KDF → `plaintext_len`.
pub fn parse_header(file: &[u8]) -> Result<Header, BackupError> {
    check_header(file, file.len() as u64)
}

/// Như [`parse_header`] nhưng chỉ cần phần đầu tệp (`head`, ít nhất 72 byte) và độ dài tệp `file_len` — để ứng
/// dụng kiểm §2.2 **trước khi** đọc phần thân (S12: không cấp bộ nhớ theo kích thước tệp lạ). Cùng thứ tự kiểm
/// và cùng mã lỗi với [`parse_header`].
pub fn check_header(head: &[u8], file_len: u64) -> Result<Header, BackupError> {
    let file = head;
    if file_len < (HEADER_LEN + TAG_LEN) as u64 || file.len() < HEADER_LEN || file[0..8] != MAGIC {
        return Err(BackupError::NotABackup);
    }
    let file_version = u16_at(file, 8);
    if file_version > FILE_VERSION {
        return Err(BackupError::NewerVersion);
    }
    if file_version == 0 || file[10] != KDF_ARGON2ID || file[11] != AEAD_XCHACHA20POLY1305 {
        return Err(BackupError::Unsupported);
    }
    let (m, t, p) = (u32_at(file, 12), u32_at(file, 16), u32_at(file, 20));
    if !M_COST_RANGE.contains(&m) || !T_COST_RANGE.contains(&t) || !P_COST_RANGE.contains(&p) {
        return Err(BackupError::BadKdfParams);
    }
    let plaintext_len = u64_at(file, 64);
    let expected = file_len - (HEADER_LEN + TAG_LEN) as u64;
    if plaintext_len != expected || plaintext_len > MAX_PLAINTEXT {
        return Err(BackupError::BadLength);
    }
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&file[24..40]);
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&file[40..64]);
    Ok(Header {
        file_version,
        m_cost_kib: m,
        t_cost: t,
        p_cost: p,
        salt,
        nonce,
        plaintext_len,
    })
}

/// Lỗi của mật khẩu khi **tạo** sao lưu (03 §2.4), để mọi giao diện dùng một quy tắc: ít nhất
/// [`MIN_PASSWORD_CODEPOINTS`] điểm mã sau NFC; khi có ô nhập lại, hai ô phải bằng nhau sau NFC. `None` = hợp lệ.
pub fn password_error(password: &str, confirm: Option<&str>) -> Option<&'static str> {
    let n: String = password.nfc().collect();
    if n.chars().count() < MIN_PASSWORD_CODEPOINTS {
        return Some("Mật khẩu cần ít nhất 8 ký tự");
    }
    if let Some(c) = confirm {
        if c.nfc().collect::<String>() != n {
            return Some("Hai lần nhập mật khẩu chưa giống nhau");
        }
    }
    None
}

/// Mật khẩu → byte UTF-8 của dạng NFC (03 §2.4). Kết quả tự xóa khi giải phóng.
pub fn normalize_password(password: &str) -> Zeroizing<Vec<u8>> {
    let s: Zeroizing<String> = Zeroizing::new(password.nfc().collect());
    Zeroizing::new(s.as_bytes().to_vec())
}

/// Argon2id(NFC(mật khẩu), salt, m, t, p) → khóa 32 byte.
pub fn derive_key(
    password: &str,
    salt: &[u8; SALT_LEN],
    m: u32,
    t: u32,
    p: u32,
) -> Result<Zeroizing<[u8; KEY_LEN]>, BackupError> {
    let pw = normalize_password(password);
    derive_key_bytes(&pw, salt, m, t, p)
}

/// Như [`derive_key`] nhưng nhận sẵn byte mật khẩu (đã chuẩn hóa).
pub fn derive_key_bytes(
    pw: &[u8],
    salt: &[u8],
    m: u32,
    t: u32,
    p: u32,
) -> Result<Zeroizing<[u8; KEY_LEN]>, BackupError> {
    let params = Params::new(m, t, p, Some(KEY_LEN)).map_err(|_| BackupError::BadKdfParams)?;
    let a2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    a2.hash_password_into(pw, salt, key.as_mut())
        .map_err(|_| BackupError::Kdf)?;
    Ok(key)
}

/// Một khối trong vùng chứa: UUID nhị phân 16 byte + byte — ảnh (`Photo`) hoặc tệp đính kèm (`Attachment`,
/// YC-2026-07); lớp lưu trữ ghép khối với bản ghi theo id (ảnh trước, tệp đính kèm sau).
#[derive(Clone, PartialEq, Eq)]
pub struct PhotoBlob {
    /// UUID của bản ghi `Photo` (16 byte, thứ tự như chuỗi hex).
    pub id: [u8; 16],
    /// Byte ảnh đã lưu.
    pub bytes: Vec<u8>,
}

impl core::fmt::Debug for PhotoBlob {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "PhotoBlob({}, {} byte)",
            uuid_to_string(&self.id),
            self.bytes.len()
        )
    }
}

impl Drop for PhotoBlob {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

/// Nội dung đã giải mã: JSON (UTF-8) + ảnh. Tự xóa khi giải phóng.
#[derive(Clone, PartialEq, Eq)]
pub struct Contents {
    /// JSON theo lược đồ backup-v1 (01 §5).
    pub json: String,
    /// Khối (ảnh rồi tệp đính kèm) theo thứ tự trong tệp.
    pub photos: Vec<PhotoBlob>,
}

impl core::fmt::Debug for Contents {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Contents(json {} byte, {} ảnh)",
            self.json.len(),
            self.photos.len()
        )
    }
}

impl Drop for Contents {
    fn drop(&mut self) {
        self.json.zeroize();
    }
}

/// Chuỗi UUID dạng `xxxxxxxx-xxxx-…` → 16 byte. `None` nếu sai dạng (chữ hoa không được nhận).
pub fn uuid_from_str(s: &str) -> Option<[u8; 16]> {
    let b = s.as_bytes();
    if b.len() != 36 || b[8] != b'-' || b[13] != b'-' || b[18] != b'-' || b[23] != b'-' {
        return None;
    }
    let mut out = [0u8; 16];
    let mut n = 0;
    let mut hi: Option<u8> = None;
    for (i, &c) in b.iter().enumerate() {
        if matches!(i, 8 | 13 | 18 | 23) {
            continue;
        }
        let v = match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            _ => return None,
        };
        match hi.take() {
            None => hi = Some(v),
            Some(h) => {
                out[n] = (h << 4) | v;
                n += 1;
            }
        }
    }
    Some(out)
}

/// 16 byte → chuỗi UUID chữ thường có gạch.
pub fn uuid_to_string(id: &[u8; 16]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(36);
    for (i, &x) in id.iter().enumerate() {
        if matches!(i, 4 | 6 | 8 | 10) {
            s.push('-');
        }
        s.push(HEX[(x >> 4) as usize] as char);
        s.push(HEX[(x & 15) as usize] as char);
    }
    s
}

/// Đóng gói bản rõ (§2.3). Kiểm tra giới hạn kích thước.
pub fn pack(json: &str, photos: &[PhotoBlob]) -> Result<Zeroizing<Vec<u8>>, BackupError> {
    if json.len() > MAX_JSON
        || photos.len() > MAX_BLOBS
        || photos.iter().any(|p| p.bytes.len() > MAX_BLOB)
    {
        return Err(BackupError::TooLarge);
    }
    let total: u64 = 8
        + 4
        + json.len() as u64
        + 4
        + photos
            .iter()
            .map(|p| 20 + p.bytes.len() as u64)
            .sum::<u64>();
    if total > MAX_PLAINTEXT {
        return Err(BackupError::TooLarge);
    }
    let mut pt = Zeroizing::new(Vec::with_capacity(total as usize));
    pt.extend_from_slice(&PLAINTEXT_MAGIC);
    pt.extend_from_slice(&(json.len() as u32).to_le_bytes());
    pt.extend_from_slice(json.as_bytes());
    pt.extend_from_slice(&(photos.len() as u32).to_le_bytes());
    for p in photos {
        pt.extend_from_slice(&p.id);
        pt.extend_from_slice(&(p.bytes.len() as u32).to_le_bytes());
        pt.extend_from_slice(&p.bytes);
    }
    Ok(pt)
}

/// Mở gói bản rõ (§2.3). Mọi sai lệch cấu trúc → [`BackupError::MalformedPlaintext`].
pub fn unpack(pt: &[u8]) -> Result<Contents, BackupError> {
    let bad = BackupError::MalformedPlaintext;
    if pt.len() < 16 || pt[0..8] != PLAINTEXT_MAGIC {
        return Err(bad);
    }
    let json_len = u32_at(pt, 8) as usize;
    if json_len > MAX_JSON || 12 + json_len + 4 > pt.len() {
        return Err(bad);
    }
    let json = core::str::from_utf8(&pt[12..12 + json_len])
        .map_err(|_| bad)?
        .to_owned();
    let mut i = 12 + json_len;
    let count = u32_at(pt, i) as usize;
    i += 4;
    if count > MAX_BLOBS {
        return Err(bad);
    }
    let mut photos = Vec::with_capacity(count);
    for _ in 0..count {
        if i + 20 > pt.len() {
            return Err(bad);
        }
        let mut id = [0u8; 16];
        id.copy_from_slice(&pt[i..i + 16]);
        let len = u32_at(pt, i + 16) as usize;
        i += 20;
        if len > MAX_BLOB || i + len > pt.len() {
            return Err(bad);
        }
        photos.push(PhotoBlob {
            id,
            bytes: pt[i..i + len].to_vec(),
        });
        i += len;
    }
    if i != pt.len() {
        return Err(bad);
    }
    Ok(Contents { json, photos })
}

/// Mã hóa bản rõ với khóa và phần đầu đã cho.
fn seal(key: &[u8; KEY_LEN], header: &Header, pt: &[u8]) -> Result<Vec<u8>, BackupError> {
    let hdr = header.to_bytes();
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| BackupError::Kdf)?;
    let ct = cipher
        .encrypt((&header.nonce).into(), Payload { msg: pt, aad: &hdr })
        .map_err(|_| BackupError::TooLarge)?;
    let mut out = Vec::with_capacity(HEADER_LEN + ct.len());
    out.extend_from_slice(&hdr);
    out.extend_from_slice(&ct);
    Ok(out)
}

/// Xuất với salt và nonce **cố định** — chỉ dùng để tạo vector kiểm thử (03 §2.5). Ứng dụng luôn gọi
/// [`export`]. Dùng tham số KDF mặc định; không chép tham số của tệp cũ (BK-T2).
pub fn export_with_fixed_rng(
    password: &str,
    json: &str,
    photos: &[PhotoBlob],
    salt: [u8; SALT_LEN],
    nonce: [u8; NONCE_LEN],
) -> Result<Vec<u8>, BackupError> {
    let pw = normalize_password(password);
    let pw_str = core::str::from_utf8(&pw).map_err(|_| BackupError::Kdf)?;
    if pw_str.chars().count() < MIN_PASSWORD_CODEPOINTS {
        return Err(BackupError::PasswordTooShort);
    }
    let pt = pack(json, photos)?;
    let header = Header {
        file_version: FILE_VERSION,
        m_cost_kib: DEFAULT_M_COST_KIB,
        t_cost: DEFAULT_T_COST,
        p_cost: DEFAULT_P_COST,
        salt,
        nonce,
        plaintext_len: pt.len() as u64,
    };
    let key = derive_key_bytes(&pw, &salt, header.m_cost_kib, header.t_cost, header.p_cost)?;
    seal(&key, &header, &pt)
}

/// Xuất tệp sao lưu: salt và nonce mới từ bộ sinh ngẫu nhiên của hệ điều hành (03 §1, §2.4).
pub fn export(password: &str, json: &str, photos: &[PhotoBlob]) -> Result<Vec<u8>, BackupError> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    getrandom::fill(&mut salt).map_err(|_| BackupError::Rng)?;
    getrandom::fill(&mut nonce).map_err(|_| BackupError::Rng)?;
    export_with_fixed_rng(password, json, photos, salt, nonce)
}

/// Giải mã với khóa đã dẫn xuất (phần đầu phải là kết quả [`parse_header`] của chính `file`).
/// Tách riêng để kiểm thử lật bit dùng lại khóa của cùng (salt, m, t, p) — kết quả giống hệt [`import`].
pub fn open_with_key(
    file: &[u8],
    header: &Header,
    key: &[u8; KEY_LEN],
) -> Result<Contents, BackupError> {
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| BackupError::Kdf)?;
    let pt = Zeroizing::new(
        cipher
            .decrypt(
                (&header.nonce).into(),
                Payload {
                    msg: &file[HEADER_LEN..],
                    aad: &file[..HEADER_LEN],
                },
            )
            .map_err(|_| BackupError::WrongPasswordOrCorrupt)?,
    );
    unpack(&pt)
}

/// Nhập tệp sao lưu: kiểm tra §2.2 → Argon2id theo tham số của tệp → giải mã → mở gói.
pub fn import(file: &[u8], password: &str) -> Result<Contents, BackupError> {
    let header = parse_header(file)?;
    let key = derive_key(
        password,
        &header.salt,
        header.m_cost_kib,
        header.t_cost,
        header.p_cost,
    )?;
    open_with_key(file, &header, &key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_roundtrip() {
        let s = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e09";
        let b = uuid_from_str(s).unwrap();
        assert_eq!(uuid_to_string(&b), s);
        assert!(uuid_from_str("0B8E4F52-3c1a-4d2e-9f60-1a2b3c4d5e09").is_none());
        assert!(uuid_from_str("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e0").is_none());
    }

    #[test]
    fn pack_unpack() {
        let ph = vec![PhotoBlob {
            id: [7; 16],
            bytes: vec![1, 2, 3],
        }];
        let pt = pack("{}", &ph).unwrap();
        let c = unpack(&pt).unwrap();
        assert_eq!(c.json, "{}");
        assert_eq!(c.photos, ph);
        let mut extra = pt.to_vec();
        extra.push(0);
        assert_eq!(unpack(&extra), Err(BackupError::MalformedPlaintext));
        assert_eq!(
            unpack(&pt[..pt.len() - 1]),
            Err(BackupError::MalformedPlaintext)
        );
    }
}
