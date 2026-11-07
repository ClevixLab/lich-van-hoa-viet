//! Kiểm tra chặt ảnh và tệp đính kèm (04 Bổ sung B6, rà soát bảo mật BM2-01/BM2-08) — đọc phần đầu tệp, **không**
//! giải mã. Bản TypeScript `pwa/src/store/media.ts` giống hệt (cùng mã lỗi; kiểm chéo bằng bộ tệp
//! `test-vectors/media/`). Dùng ở mọi đường ghi: nhập sao lưu, `add_photo`, `add_attachment`, chuyển đổi lược đồ.
//!
//! - Chữ ký khớp `mime`: JPEG `FF D8 FF`; WebP `RIFF····WEBP`; PDF `%PDF-`.
//! - JPEG: duyệt đoạn tới SOS; đoạn APP1 (EXIF/XMP) hay APP13 (IPTC) → từ chối; kích thước lấy từ SOFn.
//! - WebP: duyệt khối RIFF; khối `EXIF` hay `XMP ` (hoặc cờ tương ứng trong VP8X) → từ chối; kích thước từ
//!   VP8X / VP8L / `VP8 `.
//! - Kích thước: ảnh (`Photo`) phải **bằng** `width`/`height` của bản ghi và ≤ 2048; ảnh đính kèm cạnh dài ≤ 2048.
//! - PDF: `%PDF-` ở đầu, `%%EOF` trong 1024 byte cuối, ≤ 10 MiB.

/// Cạnh dài tối đa (01 §3.8 lược đồ, §3.8a).
pub const MAX_EDGE: u32 = 2048;
/// PDF tối đa 10 MiB (01 §3.8a).
pub const MAX_PDF: usize = 10 * 1024 * 1024;
/// `%%EOF` phải nằm trong ngần này byte cuối.
pub const PDF_EOF_WINDOW: usize = 1024;

/// Lỗi kiểm tra — mã ổn định, giống bản TS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaError {
    /// `mime` không được phép.
    BadMime,
    /// Chữ ký tệp không khớp `mime`.
    MimeMismatch,
    /// Phần đầu tệp dị dạng / không đọc được kích thước.
    BadHeader,
    /// Còn siêu dữ liệu (JPEG APP1/APP13, WebP EXIF/XMP).
    Metadata,
    /// Kích thước điểm ảnh khác bản ghi.
    DimensionsMismatch,
    /// Vượt giới hạn kích thước (điểm ảnh hoặc byte).
    TooLarge,
    /// PDF thiếu `%%EOF` gần cuối.
    PdfNoEof,
}

impl MediaError {
    /// Mã ổn định (cùng bản TS).
    pub fn code(self) -> &'static str {
        match self {
            MediaError::BadMime => "bad_mime",
            MediaError::MimeMismatch => "mime_mismatch",
            MediaError::BadHeader => "bad_header",
            MediaError::Metadata => "metadata",
            MediaError::DimensionsMismatch => "dimensions_mismatch",
            MediaError::TooLarge => "too_large",
            MediaError::PdfNoEof => "pdf_no_eof",
        }
    }
}

/// Kích thước điểm ảnh đọc từ phần đầu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dims {
    pub width: u32,
    pub height: u32,
}

fn be16(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from(*b.get(i)?) << 8 | u32::from(*b.get(i + 1)?))
}
fn le16(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from(*b.get(i)?) | u32::from(*b.get(i + 1)?) << 8)
}
fn le24(b: &[u8], i: usize) -> Option<u32> {
    Some(le16(b, i)? | u32::from(*b.get(i + 2)?) << 16)
}
fn le32(b: &[u8], i: usize) -> Option<u32> {
    Some(le24(b, i)? | u32::from(*b.get(i + 3)?) << 24)
}

/// JPEG: chữ ký, không APP1/APP13 trước SOS, kích thước từ SOFn.
pub fn jpeg_dims(b: &[u8]) -> Result<Dims, MediaError> {
    if b.len() < 4 || b[0] != 0xFF || b[1] != 0xD8 || b[2] != 0xFF {
        return Err(MediaError::MimeMismatch);
    }
    let mut i = 2;
    let mut dims = None;
    loop {
        if i + 4 > b.len() || b[i] != 0xFF {
            return Err(MediaError::BadHeader);
        }
        let m = b[i + 1];
        if m == 0xFF {
            i += 1; // byte đệm
            continue;
        }
        if m == 0xD8 || (0xD0..=0xD7).contains(&m) || m == 0x01 {
            i += 2;
            continue;
        }
        if m == 0xD9 {
            return Err(MediaError::BadHeader); // EOI trước SOS
        }
        let len = be16(b, i + 2).ok_or(MediaError::BadHeader)? as usize;
        if len < 2 || i + 2 + len > b.len() {
            return Err(MediaError::BadHeader);
        }
        if m == 0xE1 || m == 0xED {
            return Err(MediaError::Metadata);
        }
        if (0xC0..=0xCF).contains(&m) && m != 0xC4 && m != 0xC8 && m != 0xCC {
            if len < 7 {
                return Err(MediaError::BadHeader);
            }
            let h = be16(b, i + 5).ok_or(MediaError::BadHeader)?;
            let w = be16(b, i + 7).ok_or(MediaError::BadHeader)?;
            dims = Some(Dims {
                width: w,
                height: h,
            });
        }
        if m == 0xDA {
            return dims.ok_or(MediaError::BadHeader);
        }
        i += 2 + len;
    }
}

/// WebP: chữ ký RIFF/WEBP, không khối EXIF/XMP, kích thước từ VP8X / VP8L / `VP8 `.
pub fn webp_dims(b: &[u8]) -> Result<Dims, MediaError> {
    if b.len() < 12 || &b[0..4] != b"RIFF" || &b[8..12] != b"WEBP" {
        return Err(MediaError::MimeMismatch);
    }
    let riff = le32(b, 4).ok_or(MediaError::BadHeader)? as usize;
    if riff + 8 > b.len() || riff < 4 {
        return Err(MediaError::BadHeader);
    }
    let end = riff + 8;
    let mut i = 12;
    let mut dims = None;
    while i + 8 <= end {
        let tag = &b[i..i + 4];
        let size = le32(b, i + 4).ok_or(MediaError::BadHeader)? as usize;
        let data = i + 8;
        if data + size > end {
            return Err(MediaError::BadHeader);
        }
        match tag {
            b"EXIF" | b"XMP " => return Err(MediaError::Metadata),
            b"VP8X" => {
                if size < 10 {
                    return Err(MediaError::BadHeader);
                }
                let flags = b[data];
                if flags & 0x08 != 0 || flags & 0x04 != 0 {
                    return Err(MediaError::Metadata);
                }
                dims = Some(Dims {
                    width: le24(b, data + 4).ok_or(MediaError::BadHeader)? + 1,
                    height: le24(b, data + 7).ok_or(MediaError::BadHeader)? + 1,
                });
            }
            b"VP8L" if dims.is_none() => {
                if size < 5 || b[data] != 0x2F {
                    return Err(MediaError::BadHeader);
                }
                let v = le32(b, data + 1).ok_or(MediaError::BadHeader)?;
                dims = Some(Dims {
                    width: (v & 0x3FFF) + 1,
                    height: ((v >> 14) & 0x3FFF) + 1,
                });
            }
            b"VP8 " if dims.is_none() => {
                if size < 10 || b[data + 3..data + 6] != [0x9D, 0x01, 0x2A] {
                    return Err(MediaError::BadHeader);
                }
                dims = Some(Dims {
                    width: le16(b, data + 6).ok_or(MediaError::BadHeader)? & 0x3FFF,
                    height: le16(b, data + 8).ok_or(MediaError::BadHeader)? & 0x3FFF,
                });
            }
            _ => {}
        }
        i = data + size + (size & 1);
    }
    match dims {
        Some(d) if d.width > 0 && d.height > 0 => Ok(d),
        _ => Err(MediaError::BadHeader),
    }
}

fn image_dims(mime: &str, b: &[u8]) -> Result<Dims, MediaError> {
    match mime {
        "image/jpeg" => jpeg_dims(b),
        "image/webp" => webp_dims(b),
        _ => Err(MediaError::BadMime),
    }
}

/// Ảnh chân dung (`Photo`): kích thước phải bằng bản ghi và ≤ 2048.
pub fn check_photo(mime: &str, bytes: &[u8], width: u32, height: u32) -> Result<(), MediaError> {
    let d = image_dims(mime, bytes)?;
    if d.width > MAX_EDGE || d.height > MAX_EDGE {
        return Err(MediaError::TooLarge);
    }
    if d.width != width || d.height != height {
        return Err(MediaError::DimensionsMismatch);
    }
    Ok(())
}

/// Tệp đính kèm (`Attachment`): ảnh cạnh dài ≤ 2048; PDF như mô tả của mô-đun.
pub fn check_attachment(mime: &str, bytes: &[u8]) -> Result<(), MediaError> {
    if mime == "application/pdf" {
        if bytes.len() > MAX_PDF {
            return Err(MediaError::TooLarge);
        }
        if !bytes.starts_with(b"%PDF-") {
            return Err(MediaError::MimeMismatch);
        }
        let tail = &bytes[bytes.len().saturating_sub(PDF_EOF_WINDOW)..];
        if !tail.windows(5).any(|w| w == b"%%EOF") {
            return Err(MediaError::PdfNoEof);
        }
        return Ok(());
    }
    let d = image_dims(mime, bytes)?;
    if d.width.max(d.height) > MAX_EDGE {
        return Err(MediaError::TooLarge);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(
            check_attachment("image/jpeg", b"%PDF-1.4\n%%EOF\n"),
            Err(MediaError::MimeMismatch)
        );
        assert_eq!(
            check_attachment("application/pdf", b"%PDF-1.4\n<html>"),
            Err(MediaError::PdfNoEof)
        );
        assert_eq!(
            check_attachment("application/pdf", b"%PDF-1.4\n%%EOF\n"),
            Ok(())
        );
        assert_eq!(
            check_attachment("image/png", b"x"),
            Err(MediaError::BadMime)
        );
    }
}
