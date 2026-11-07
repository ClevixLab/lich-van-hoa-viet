//! Tệp đính kèm thông tin báo tử (01 §3.8a, YC-2026-07) — phần của nền tảng Windows, theo yêu cầu đăng ký trước §5.3
//! của `docs/reviews/RA-SOAT-BAO-MAT-V02.md` và 04 Bổ sung B6:
//!
//! 1. Ảnh đi qua đúng bộ xử lý ảnh (giải mã có trần, mã hóa lại JPEG, bỏ siêu dữ liệu, cạnh dài ≤ 2048, ≤ 2 MB).
//! 2. PDF: kiểm kích thước (≤ 10 MiB, từ siêu dữ liệu tệp) và `%PDF-` **trước khi** đọc hết; có `%%EOF` gần cuối.
//!    Lưu nguyên byte ("Tệp PDF được lưu nguyên bản"); ứng dụng không bao giờ giải mã hay hiển thị PDF.
//! 3. Mở PDF bằng ứng dụng ngoài: chép vào thư mục con ngẫu nhiên trong
//!    `%LOCALAPPDATA%\ClevixLab\LichVanHoaViet\tmp\` (không phải `%TEMP%` dùng chung), tên chung `TepDinhKem.pdf`
//!    (không mang tên người hay tên gốc), xóa toàn bộ `tmp\` khi đóng ứng dụng và khi mở lần sau.
//!
//! Việc lưu vào kho (`add_attachment`, `attachment_bytes`) và giao diện chờ lõi (PR #28) vào `main`.
#![allow(dead_code)] // Giao diện tệp đính kèm nối vào khi lõi có `add_attachment` (PR #28).

use std::io::Read;
use std::path::{Path, PathBuf};

use crate::photo;

/// PDF tối đa 10 MiB (01 §3.8a).
pub const MAX_PDF: u64 = 10 * 1024 * 1024;
/// Tên chung của bản sao tạm.
pub const TEMP_NAME: &str = "TepDinhKem.pdf";

/// Tệp đã kiểm, sẵn sàng `add_attachment(id, mime, file_name, bytes)`.
#[derive(Debug)]
pub struct Prepared {
    pub mime: &'static str,
    pub bytes: Vec<u8>,
    /// Tên gốc chỉ để hiển thị (≤ 120 ký tự), không dùng làm tên tệp trên máy.
    pub file_name: Option<String>,
    /// `true` với PDF: giao diện ghi "Tệp PDF được lưu nguyên bản".
    pub kept_as_is: bool,
}

fn display_name(path: &Path) -> Option<String> {
    path.file_name()
        .map(|n| n.to_string_lossy().chars().take(120).collect::<String>())
        .filter(|n| !n.trim().is_empty())
}

/// Kiểm một PDF đã nằm trong bộ nhớ (cũng dùng cho byte từ kho trước khi mở).
pub fn check_pdf(b: &[u8]) -> Result<(), String> {
    if b.len() as u64 > MAX_PDF {
        return Err("Tệp PDF lớn hơn 10 MB — hãy quét lại với độ phân giải thấp hơn".into());
    }
    if !b.starts_with(b"%PDF-") {
        return Err("Tệp này không phải PDF".into());
    }
    let tail = &b[b.len().saturating_sub(1024)..];
    if !tail.windows(5).any(|w| w == b"%%EOF") {
        return Err("Tệp PDF bị cắt cụt hoặc hỏng".into());
    }
    Ok(())
}

/// Đọc và kiểm một tệp người dùng chọn. PDF nhận ra theo 5 byte đầu, không theo phần mở rộng.
pub fn prepare(path: &Path) -> Result<Prepared, String> {
    let meta = std::fs::metadata(path).map_err(|_| "Không đọc được tệp này".to_string())?;
    let mut f = std::fs::File::open(path).map_err(|_| "Không đọc được tệp này".to_string())?;
    let mut head = [0u8; 5];
    let n = f.read(&mut head).map_err(|_| "Không đọc được tệp này".to_string())?;
    if n == 5 && &head == b"%PDF-" {
        if meta.len() > MAX_PDF {
            return Err("Tệp PDF lớn hơn 10 MB — hãy quét lại với độ phân giải thấp hơn".into());
        }
        let mut b = head.to_vec();
        f.take(MAX_PDF + 1 - 5).read_to_end(&mut b).map_err(|_| "Không đọc được tệp này".to_string())?;
        check_pdf(&b)?;
        return Ok(Prepared { mime: "application/pdf", bytes: b, file_name: display_name(path), kept_as_is: true });
    }
    if meta.len() > photo::MAX_INPUT as u64 {
        return Err("Ảnh lớn hơn 25 MB — hãy chọn ảnh nhỏ hơn".into());
    }
    drop(f);
    let raw = std::fs::read(path).map_err(|_| "Không đọc được tệp này".to_string())?;
    let e = photo::reencode_with(&raw, photo::DOC_SIDE, photo::DOC_STORED)?;
    Ok(Prepared { mime: "image/jpeg", bytes: e.bytes, file_name: display_name(path), kept_as_is: false })
}

/// Thư mục tạm riêng của ứng dụng: `<thư mục dữ liệu>\tmp`.
pub fn tmp_root(data_dir: &Path) -> PathBuf {
    data_dir.join("tmp")
}

/// Xóa mọi bản sao tạm (gọi khi mở và khi đóng ứng dụng). Không đụng tới gì ngoài `tmp\`.
pub fn cleanup(data_dir: &Path) {
    let _ = std::fs::remove_dir_all(tmp_root(data_dir));
}

/// Ghi bản sao tạm `tmp\<ngẫu nhiên>\TepDinhKem.pdf` và trả đường dẫn. Kiểm lại byte trước khi ghi.
pub fn temp_copy(data_dir: &Path, bytes: &[u8]) -> Result<PathBuf, String> {
    check_pdf(bytes)?;
    let sub = lvhv_ffi::new_uuid().map_err(|e| e.to_string())?.replace('-', "");
    let dir = tmp_root(data_dir).join(sub);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let p = dir.join(TEMP_NAME);
    std::fs::write(&p, bytes).map_err(|e| e.to_string())?;
    Ok(p)
}

/// Mở bản sao tạm bằng ứng dụng xem PDF mặc định của Windows (không qua trình duyệt, không có mạng).
pub fn open_pdf(data_dir: &Path, bytes: &[u8]) -> Result<PathBuf, String> {
    let p = temp_copy(data_dir, bytes)?;
    if !cfg!(test) && !crate::platform::is_demo() {
        #[cfg(windows)]
        std::process::Command::new("explorer.exe").arg(&p).spawn().map_err(|e| e.to_string())?;
        #[cfg(not(windows))]
        crate::update::open_with_os(&p.to_string_lossy());
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("lvhv-att-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn pdf() -> Vec<u8> {
        b"%PDF-1.4\n1 0 obj<<>>endobj\ntrailer<<>>\n%%EOF\n".to_vec()
    }

    #[test]
    fn pdf_checks() {
        let d = tmp("pdf");
        let ok = d.join("Giấy chứng tử của bà Bình.pdf");
        std::fs::write(&ok, pdf()).unwrap();
        let p = prepare(&ok).unwrap();
        assert_eq!(p.mime, "application/pdf");
        assert_eq!(p.bytes, pdf(), "PDF lưu nguyên byte");
        assert!(p.kept_as_is);
        // Giả PDF: HTML đổi đuôi; PDF cụt; PDF quá 10 MiB (kiểm từ siêu dữ liệu, không đọc hết).
        let html = d.join("a.pdf");
        std::fs::write(&html, b"<html>...</html>").unwrap();
        assert!(prepare(&html).is_err());
        let cut = d.join("b.pdf");
        std::fs::write(&cut, b"%PDF-1.4\n1 0 obj").unwrap();
        assert!(prepare(&cut).unwrap_err().contains("cắt cụt"));
        let big = d.join("c.pdf");
        let f = std::fs::File::create(&big).unwrap();
        std::io::Write::write_all(&mut &f, b"%PDF-").unwrap();
        f.set_len(MAX_PDF + 1).unwrap();
        assert!(prepare(&big).unwrap_err().contains("10 MB"));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn image_goes_through_photo_pipeline() {
        let d = tmp("img");
        let f = d.join("anh.jpg");
        std::fs::write(&f, crate::photo::tests::jpeg_with_exif(3000, 1000)).unwrap();
        let p = prepare(&f).unwrap();
        assert_eq!(p.mime, "image/jpeg");
        assert!(!p.kept_as_is);
        assert!(!crate::photo::has_metadata_segments(&p.bytes));
        assert!(p.bytes.len() <= crate::photo::DOC_STORED);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn temp_copy_neutral_name_and_cleanup() {
        let d = tmp("open");
        let a = open_pdf(&d, &pdf()).unwrap();
        let b = open_pdf(&d, &pdf()).unwrap();
        assert_eq!(a.file_name().unwrap(), TEMP_NAME);
        assert_ne!(a.parent(), b.parent(), "mỗi lần mở một thư mục con riêng");
        assert!(a.starts_with(d.join("tmp")), "trong thư mục dữ liệu của ứng dụng, không phải %TEMP% chung");
        assert!(open_pdf(&d, b"<html>").is_err(), "không ghi tệp không phải PDF");
        std::fs::write(d.join("lvhv.sqlite"), b"x").unwrap();
        cleanup(&d);
        assert!(!d.join("tmp").exists());
        assert!(d.join("lvhv.sqlite").exists(), "dọn tệp tạm không đụng dữ liệu");
        let _ = std::fs::remove_dir_all(&d);
    }
}
