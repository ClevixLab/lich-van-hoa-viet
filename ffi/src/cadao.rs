//! Ca dao/tục ngữ trong ngày — nhúng `content/cadao/cadao.json`, chọn theo
//! `content/cadao/select.md`:
//!
//! ```text
//! pos   = (JDN − 2451545) mod N   (Euclid, luôn ≥ 0)
//! entry = phần tử có entry.order == pos
//! ```

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Tệp dữ liệu nhúng lúc biên dịch (CC BY-SA 4.0, xem SOURCES.md mục C).
const CADAO_JSON: &str = include_str!("../../content/cadao/cadao.json");
/// Danh mục nguồn (hiển thị ở màn hình Giới thiệu).
pub const SOURCES_MD: &str = include_str!("../../content/cadao/SOURCES.md");

/// JDN của 2000-01-01 — mốc cố định của select.md.
const EPOCH_JDN: i64 = 2451545;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub title: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
    #[serde(default)]
    pub lines: Vec<String>,
    #[serde(default)]
    pub theme: String,
    pub order: usize,
    #[serde(default)]
    pub sources: Vec<Source>,
}

/// Các câu, sắp theo `order` (vị trí i giữ câu có order == i).
fn by_order() -> &'static [Entry] {
    static DATA: OnceLock<Vec<Entry>> = OnceLock::new();
    DATA.get_or_init(|| {
        // Dữ liệu được kiểm tra bởi content/cadao/validate.py và bởi test `cadao_order_is_permutation`;
        // nếu tệp hỏng thì đây là lỗi lúc dựng, không phải lỗi người dùng.
        let mut v: Vec<Entry> =
            serde_json::from_str(CADAO_JSON).expect("content/cadao/cadao.json không hợp lệ");
        v.sort_by_key(|e| e.order);
        v
    })
}

/// Số câu (N).
pub fn count() -> usize {
    by_order().len()
}

/// Câu của ngày có số ngày Julian `jdn`.
pub fn entry_for_jdn(jdn: i64) -> Option<&'static Entry> {
    let all = by_order();
    if all.is_empty() {
        return None;
    }
    let pos = (jdn - EPOCH_JDN).rem_euclid(all.len() as i64) as usize;
    // Sau khi sắp, all[pos].order == pos nếu order là hoán vị; vẫn tra lại cho chắc.
    match all.get(pos) {
        Some(e) if e.order == pos => Some(e),
        _ => all.iter().find(|e| e.order == pos),
    }
}

/// JSON của câu trong ngày (chuỗi "null" nếu không có dữ liệu).
pub fn for_jdn(jdn: i64) -> String {
    match entry_for_jdn(jdn) {
        Some(e) => serde_json::to_string(e).unwrap_or_else(|_| "null".to_string()),
        None => "null".to_string(),
    }
}

/// Lời ghi công bắt buộc — SOURCES.md mục C.1 (giữ nguyên câu chữ; test kiểm tra SOURCES.md vẫn
/// chứa đoạn này, nếu nguồn đổi thì phải cập nhật ở đây).
pub const ATTRIBUTION: &str = "Bộ ca dao, tục ngữ, thành ngữ là văn học dân gian Việt Nam (khuyết danh, phạm vi công cộng). Câu chữ được đối chiếu với Wikiquote tiếng Việt và Wikiquote tiếng Anh (nội dung theo giấy phép CC BY-SA 4.0, https://creativecommons.org/licenses/by-sa/4.0/), và với *Tục ngữ, cổ ngữ, gia ngôn* của Huỳnh Tịnh Của (1896–1897, phạm vi công cộng; bản số hóa trên Wikisource). Danh sách nguồn chi tiết: content/cadao/SOURCES.md. Biên soạn: Clevix Lab.";

/// Lời ghi công dạng văn bản thường (bỏ dấu * của Markdown).
pub fn attribution() -> String {
    ATTRIBUTION.replace('*', "")
}
