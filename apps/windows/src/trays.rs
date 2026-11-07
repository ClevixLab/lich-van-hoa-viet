//! Gói ảnh mâm lễ Clevix (YC-2026-10, 08 §16.3): nhúng lúc dựng từ `content/memorial/trays/manifest.json` — chỉ mục
//! `trang_thai = "duyet"`, có tệp, đúng `sha256` (build.rs kiểm; sai thì dừng dựng). Không tải gì qua mạng. Gói hiện rỗng.

/// Một ảnh đã duyệt trong gói.
pub struct PackItem {
    /// `id` trong manifest (để đọc mã sinh ra và chẩn đoán; giao diện chưa dùng).
    #[allow(dead_code)]
    pub id: &'static str,
    pub tray_style: &'static str,
    pub bytes: &'static [u8],
    /// "Ảnh: <tác giả> · <giấy phép>" khi giấy phép cần ghi công; rỗng nếu không.
    pub credit: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/trays.rs"));

#[cfg(test)]
thread_local! {
    /// Kiểm thử thay gói (08 §16.4 ca 5).
    pub static TEST_PACK: std::cell::RefCell<Option<Vec<(String, Vec<u8>, String)>>> = const { std::cell::RefCell::new(None) };
}

/// Ảnh Clevix đầu tiên cùng kiểu mâm (đã giải mã có trần) và dòng ghi công.
pub fn pick(style: &str) -> Option<(slint::Image, String)> {
    #[cfg(test)]
    if let Some(v) = TEST_PACK.with(|t| t.borrow().clone()) {
        return v
            .into_iter()
            .find(|(s, _, _)| s == style)
            .and_then(|(_, b, c)| crate::photo::to_slint(&b).map(|i| (i, c)));
    }
    PACK.iter()
        .find(|i| i.tray_style == style)
        .and_then(|i| crate::photo::to_slint(i.bytes).map(|img| (img, i.credit.to_string())))
}

/// Số ảnh trong gói đã nhúng (kiểm thử, CI).
#[allow(dead_code)]
pub fn pack_len() -> usize {
    PACK.len()
}
