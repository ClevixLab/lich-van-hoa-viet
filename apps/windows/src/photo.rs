//! Ảnh chân dung và ảnh giấy báo tử (01 §3.8, S9): giải mã, xoay theo EXIF, thu cạnh dài ≤ 1024 px (không
//! phóng to), **mã hóa lại JPEG** — byte lưu chỉ gồm điểm ảnh, không còn EXIF/GPS/XMP/IPTC. Không đọc mạng.

use std::io::Cursor;

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, GenericImageView, ImageFormat};

/// Ảnh đầu vào tối đa 25 MB (01 §3.8).
pub const MAX_INPUT: usize = 25 * 1024 * 1024;
/// Ảnh lưu tối đa 1 MB.
pub const MAX_STORED: usize = 1024 * 1024;
const MAX_SIDE: u32 = 1024;
/// Tệp đính kèm dạng ảnh (01 §3.8a): cạnh dài ≤ 2048 px để giấy tờ còn đọc được, ≤ 2 MB.
pub const DOC_SIDE: u32 = 2048;
pub const DOC_STORED: usize = 2 * 1024 * 1024;
/// Trần bộ nhớ cấp cho một lần giải mã (BM2-10, T-D3): 50 Mpx × 4 byte + đệm.
const MAX_ALLOC: u64 = 256 * 1024 * 1024;
/// Hiển thị (BM2-10, B6 mục 3): ảnh trong kho do ứng dụng tạo có cạnh ≤ 2048; tệp cũ hay độc vượt trần này thì hiện
/// khung trống chứ không giải mã.
pub const DISPLAY_MAX_SIDE: u32 = 4096;
const DISPLAY_MAX_ALLOC: u64 = 80 * 1024 * 1024;
/// Ảnh hiển thị được thu về cạnh dài ≤ 1024 (khung lớn nhất trên màn hình là ảnh tưởng niệm).
const DISPLAY_SIDE: u32 = 1024;

fn limits(max_side: u32, max_alloc: u64) -> image::io::Limits {
    let mut l = image::io::Limits::default();
    l.max_image_width = Some(max_side);
    l.max_image_height = Some(max_side);
    l.max_alloc = Some(max_alloc);
    l
}

/// Ảnh đã mã hóa lại, sẵn sàng `LvhvStore::add_photo(id, "image/jpeg", width, height, bytes)`.
#[derive(Clone, Debug)]
pub struct Encoded {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Hướng EXIF (thẻ 0x0112) của tệp JPEG; 1 khi không có.
pub fn exif_orientation(b: &[u8]) -> u16 {
    if b.len() < 4 || b[0] != 0xFF || b[1] != 0xD8 {
        return 1;
    }
    let mut i = 2usize;
    while i + 4 <= b.len() {
        if b[i] != 0xFF {
            return 1;
        }
        let marker = b[i + 1];
        if marker == 0xDA || marker == 0xD9 {
            return 1;
        }
        let len = u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
        let seg = i + 4;
        let end = (i + 2 + len).min(b.len());
        if marker == 0xE1 && end >= seg + 6 && &b[seg..seg + 6] == b"Exif\0\0" {
            return tiff_orientation(&b[seg + 6..end]).unwrap_or(1);
        }
        i += 2 + len;
    }
    1
}

fn tiff_orientation(t: &[u8]) -> Option<u16> {
    let le = match t.get(0..2)? {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let r16 = |o: usize| -> Option<u16> {
        let s = t.get(o..o + 2)?;
        Some(if le {
            u16::from_le_bytes([s[0], s[1]])
        } else {
            u16::from_be_bytes([s[0], s[1]])
        })
    };
    let r32 = |o: usize| -> Option<u32> {
        let s = t.get(o..o + 4)?;
        Some(if le {
            u32::from_le_bytes([s[0], s[1], s[2], s[3]])
        } else {
            u32::from_be_bytes([s[0], s[1], s[2], s[3]])
        })
    };
    let ifd = r32(4)? as usize;
    let n = r16(ifd)? as usize;
    for k in 0..n.min(200) {
        let e = ifd + 2 + k * 12;
        if r16(e)? == 0x0112 {
            let v = r16(e + 8)?;
            return (1..=8).contains(&v).then_some(v);
        }
    }
    None
}

fn orient(img: DynamicImage, o: u16) -> DynamicImage {
    match o {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    }
}

/// Giải mã + mã hóa lại một ảnh người dùng chọn. Lỗi là câu hiển thị được (§3.4).
pub fn reencode(input: &[u8]) -> Result<Encoded, String> {
    reencode_with(input, MAX_SIDE, MAX_STORED)
}

/// Như [`reencode`] với cạnh dài và dung lượng lưu riêng (tệp đính kèm dạng ảnh: 2048 px, 2 MB).
pub fn reencode_with(input: &[u8], max_side: u32, max_stored: usize) -> Result<Encoded, String> {
    if input.len() > MAX_INPUT {
        return Err("Ảnh lớn hơn 25 MB — hãy chọn ảnh nhỏ hơn".into());
    }
    let unreadable = || "Không đọc được ảnh này (định dạng không hỗ trợ)".to_string();
    let reader = image::io::Reader::new(Cursor::new(input))
        .with_guessed_format()
        .map_err(|_| unreadable())?;
    let fmt = reader.format().ok_or_else(unreadable)?;
    if !matches!(
        fmt,
        ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::Gif | ImageFormat::WebP | ImageFormat::Bmp
    ) {
        return Err(unreadable());
    }
    let (w, h) = reader.into_dimensions().map_err(|_| unreadable())?;
    if w > 12_000 || h > 12_000 || (w as u64) * (h as u64) > 50_000_000 {
        return Err(format!(
            "Ảnh quá lớn ({w} × {h} điểm ảnh) — hãy chọn ảnh nhỏ hơn"
        ));
    }
    // Giải mã có trần (BM2-10): kích thước đã kiểm từ phần đầu tệp ở trên; trần cấp phát chặn thêm định dạng nén.
    let mut reader = image::io::Reader::with_format(Cursor::new(input), fmt);
    reader.limits(limits(12_000, MAX_ALLOC));
    let img = reader.decode().map_err(|_| unreadable())?;
    let img = if fmt == ImageFormat::Jpeg {
        orient(img, exif_orientation(input))
    } else {
        img
    };
    let (w, h) = img.dimensions();
    let img = if w.max(h) > max_side {
        let s = max_side as f64 / w.max(h) as f64;
        let nw = ((w as f64 * s).round() as u32).max(1);
        let nh = ((h as f64 * s).round() as u32).max(1);
        img.resize_exact(nw, nh, image::imageops::FilterType::Triangle)
    } else {
        img
    };
    // Nền trắng cho ảnh trong suốt; bỏ kênh alpha.
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut rgb = image::RgbImage::new(w, h);
    for (x, y, p) in rgba.enumerate_pixels() {
        let a = p[3] as u32;
        let mix = |c: u8| ((c as u32 * a + 255 * (255 - a)) / 255) as u8;
        rgb.put_pixel(x, y, image::Rgb([mix(p[0]), mix(p[1]), mix(p[2])]));
    }
    for q in [86u8, 76, 66, 56] {
        let mut out = Vec::new();
        JpegEncoder::new_with_quality(&mut out, q)
            .encode_image(&rgb)
            .map_err(|_| unreadable())?;
        if out.len() <= max_stored {
            return Ok(Encoded {
                bytes: out,
                width: w,
                height: h,
            });
        }
    }
    Err(format!("Ảnh quá chi tiết, chưa nén được dưới {} MB", max_stored / (1024 * 1024)))
}

/// Ảnh để hiển thị trong Slint (giải mã byte JPEG đã lưu). BM2-10: đọc kích thước từ phần đầu tệp trước; vượt
/// [`DISPLAY_MAX_SIDE`] thì trả `None` (khung trống), không giải mã. Giải mã có trần cấp phát, rồi thu về cạnh dài
/// ≤ 1024 trước khi đưa cho Slint.
pub fn to_slint(bytes: &[u8]) -> Option<slint::Image> {
    let img = decode_bounded(bytes)?.to_rgba8();
    let (w, h) = img.dimensions();
    let buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(img.as_raw(), w, h);
    Some(slint::Image::from_rgba8(buf))
}

/// Giải mã ảnh để hiển thị với giới hạn kích thước điểm ảnh và bộ nhớ (BM2-10).
pub fn decode_bounded(bytes: &[u8]) -> Option<DynamicImage> {
    let probe = image::io::Reader::new(Cursor::new(bytes)).with_guessed_format().ok()?;
    let fmt = probe.format()?;
    if !matches!(fmt, ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP) {
        return None;
    }
    let (w, h) = probe.into_dimensions().ok()?;
    if w == 0 || h == 0 || w > DISPLAY_MAX_SIDE || h > DISPLAY_MAX_SIDE {
        return None;
    }
    let mut reader = image::io::Reader::with_format(Cursor::new(bytes), fmt);
    reader.limits(limits(DISPLAY_MAX_SIDE, DISPLAY_MAX_ALLOC));
    let img = reader.decode().ok()?;
    Some(if img.width().max(img.height()) > DISPLAY_SIDE {
        img.thumbnail(DISPLAY_SIDE, DISPLAY_SIDE)
    } else {
        img
    })
}

/// Byte có đoạn APP1 (EXIF/XMP) hoặc APP13 (IPTC) không — dùng trong kiểm thử S9.
#[cfg_attr(not(test), allow(dead_code))]
pub fn has_metadata_segments(b: &[u8]) -> bool {
    let mut i = 2usize;
    while i + 4 <= b.len() && b[i] == 0xFF {
        let marker = b[i + 1];
        if marker == 0xDA {
            return false;
        }
        if marker == 0xE1 || marker == 0xED {
            return true;
        }
        let len = u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
        i += 2 + len;
    }
    false
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// JPEG thử có đoạn EXIF (hướng 6) và APP13 với chuỗi "GPS-BI-MAT".
    pub fn jpeg_with_exif(w: u32, h: u32) -> Vec<u8> {
        let img = image::RgbImage::from_fn(w, h, |x, y| image::Rgb([(x * 7) as u8, (y * 5) as u8, 90]));
        let mut plain = Vec::new();
        JpegEncoder::new_with_quality(&mut plain, 90)
            .encode_image(&img)
            .unwrap();
        // TIFF (II), IFD0 có 1 mục: 0x0112 SHORT 1 = 6.
        let mut tiff = vec![b'I', b'I', 42, 0, 8, 0, 0, 0, 1, 0];
        tiff.extend_from_slice(&[0x12, 0x01, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0]);
        tiff.extend_from_slice(&[0, 0, 0, 0]);
        let mut app1 = b"Exif\0\0".to_vec();
        app1.extend_from_slice(&tiff);
        app1.extend_from_slice(b"GPS-BI-MAT");
        let mut app13 = b"Photoshop 3.0\0GPS-BI-MAT".to_vec();
        let mut out = vec![0xFF, 0xD8];
        for (m, d) in [(0xE1u8, &mut app1), (0xED, &mut app13)] {
            out.extend_from_slice(&[0xFF, m]);
            out.extend_from_slice(&((d.len() + 2) as u16).to_be_bytes());
            out.extend_from_slice(d);
        }
        out.extend_from_slice(&plain[2..]);
        out
    }

    #[test]
    fn strips_metadata_and_rotates() {
        let src = jpeg_with_exif(300, 120);
        assert_eq!(exif_orientation(&src), 6);
        assert!(has_metadata_segments(&src));
        let e = reencode(&src).unwrap();
        assert_eq!((e.width, e.height), (120, 300), "xoay 90° theo EXIF");
        assert!(!has_metadata_segments(&e.bytes), "không còn APP1/APP13");
        assert!(!e.bytes.windows(10).any(|w| w == b"GPS-BI-MAT"));
    }

    #[test]
    fn downscales_and_errors() {
        let img = image::RgbImage::from_fn(2400, 1200, |x, _| image::Rgb([x as u8, 0, 0]));
        let mut png = Vec::new();
        DynamicImage::ImageRgb8(img)
            .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
            .unwrap();
        let e = reencode(&png).unwrap();
        assert_eq!((e.width, e.height), (1024, 512));
        assert!(e.bytes.len() <= MAX_STORED);
        assert_eq!(
            reencode(b"khong phai anh").err().unwrap(),
            "Không đọc được ảnh này (định dạng không hỗ trợ)"
        );
    }

    fn png(w: u32, h: u32) -> Vec<u8> {
        let mut out = Vec::new();
        DynamicImage::ImageLuma8(image::GrayImage::new(w, h))
            .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
            .unwrap();
        out
    }

    /// BM2-10 / B6: hiển thị có trần; ảnh vượt trần, ảnh giả (HTML), byte hỏng → khung trống, không giải mã.
    #[test]
    fn display_decode_is_bounded() {
        let big = png(DISPLAY_MAX_SIDE + 1, 2);
        assert!(big.len() < 64 * 1024, "tệp nhỏ nhưng khổ lớn");
        assert!(decode_bounded(&big).is_none());
        assert!(decode_bounded(b"<html><script>alert(1)</script></html>").is_none());
        let mut cut = png(800, 600);
        cut.truncate(cut.len() / 2);
        assert!(decode_bounded(&cut).is_none());
        let ok = decode_bounded(&png(3000, 1500)).unwrap();
        assert_eq!((ok.width(), ok.height()), (1024, 512), "thu nhỏ trước khi đưa cho Slint");
        let e = reencode(&png(800, 600)).unwrap();
        assert_eq!(decode_bounded(&e.bytes).unwrap().dimensions(), (800, 600));
    }

    /// Tệp đính kèm dạng ảnh (01 §3.8a): cạnh dài ≤ 2048, ≤ 2 MB; ảnh khổng lồ bị từ chối từ phần đầu tệp.
    #[test]
    fn document_reencode_and_huge_input() {
        let e = reencode_with(&png(3000, 1500), DOC_SIDE, DOC_STORED).unwrap();
        assert_eq!((e.width, e.height), (2048, 1024));
        assert!(e.bytes.len() <= DOC_STORED);
        let huge = png(12_001, 1);
        assert!(reencode(&huge).unwrap_err().contains("Ảnh quá lớn"));
    }
}
