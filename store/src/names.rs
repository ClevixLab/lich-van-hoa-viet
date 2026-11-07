//! Cách gọi, quan hệ, giới tính (01 §3.1, YC-2026-07) — dùng chung cho bốn giao diện qua FFI; bản TypeScript
//! `pwa/src/store/names.ts` giống hệt (kiểm chéo trong `scripts/check-store.sh`).
use serde_json::Value;
use unicode_normalization::UnicodeNormalization;

/// Gợi ý cách gọi (đứng trước tên) cho **người đã mất** (01 §3.1).
pub const DANH_XUNG_DECEASED: [&str; 22] = [
    "Ông",
    "Bà",
    "Cụ",
    "Kỵ",
    "Bác",
    "Cô",
    "Chú",
    "Dì",
    "Cậu",
    "Mợ",
    "Thím",
    "Anh",
    "Chị",
    "Em",
    "Cháu",
    "Con",
    "Bạn",
    "Bố",
    "Mẹ",
    "Ông Tổ họ",
    "Bà Cô Tổ họ",
    "Bé đỏ",
];

/// Cách gọi chỉ dành cho người đã mất — không gợi ý cho người đang sống (YC-2026-07).
pub const DANH_XUNG_DECEASED_ONLY: [&str; 5] = ["Cụ", "Kỵ", "Ông Tổ họ", "Bà Cô Tổ họ", "Bé đỏ"];

/// Cách gọi thường dùng cho nam / cho nữ — chỉ để xếp gợi ý theo `gioi_tinh` (các từ còn lại dùng cho cả hai).
const DANH_XUNG_NAM: [&str; 6] = ["Ông", "Chú", "Cậu", "Anh", "Bố", "Ông Tổ họ"];
const DANH_XUNG_NU: [&str; 8] = ["Bà", "Cô", "Dì", "Mợ", "Thím", "Chị", "Mẹ", "Bà Cô Tổ họ"];

/// Gợi ý quan hệ cho người đã mất (đã viết hoa chữ đầu).
pub const QUAN_HE_DECEASED: [&str; 27] = [
    "Bố",
    "Mẹ",
    "Ông nội",
    "Bà nội",
    "Ông ngoại",
    "Bà ngoại",
    "Cụ ông",
    "Cụ bà",
    "Kỵ ông",
    "Kỵ bà",
    "Ông cố",
    "Bà cố",
    "Ông sơ",
    "Bà sơ",
    "Bác",
    "Chú",
    "Thím",
    "Cô",
    "Dì",
    "Cậu",
    "Mợ",
    "Anh",
    "Chị",
    "Em",
    "Chồng",
    "Vợ",
    "Con",
];

/// Gợi ý quan hệ cho người đang sống (đã viết hoa chữ đầu).
pub const QUAN_HE_LIVING: [&str; 21] = [
    "Bố",
    "Mẹ",
    "Ông nội",
    "Bà nội",
    "Ông ngoại",
    "Bà ngoại",
    "Bác",
    "Chú",
    "Thím",
    "Cô",
    "Dì",
    "Cậu",
    "Mợ",
    "Anh",
    "Chị",
    "Em",
    "Chồng",
    "Vợ",
    "Con trai",
    "Con gái",
    "Cháu",
];

/// Danh sách gợi ý cách gọi. `gioi_tinh` = `"nam"`: từ cho nam lên trước, rồi từ chung, rồi từ cho nữ; `"nu"`
/// ngược lại; khác (`None`) → thứ tự gốc. Người đang sống không có Cụ, Kỵ, Ông Tổ họ, Bà Cô Tổ họ, Bé đỏ.
pub fn danh_xung_suggestions(deceased: bool, gioi_tinh: Option<&str>) -> Vec<&'static str> {
    let base: Vec<&'static str> = DANH_XUNG_DECEASED
        .iter()
        .copied()
        .filter(|x| deceased || !DANH_XUNG_DECEASED_ONLY.contains(x))
        .collect();
    let (first, last): (&[&str], &[&str]) = match gioi_tinh {
        Some("nam") => (&DANH_XUNG_NAM, &DANH_XUNG_NU),
        Some("nu") => (&DANH_XUNG_NU, &DANH_XUNG_NAM),
        _ => return base,
    };
    let rank = |x: &&str| {
        if first.contains(x) {
            0
        } else if last.contains(x) {
            2
        } else {
            1
        }
    };
    let mut v = base;
    v.sort_by_key(rank); // sắp ổn định: giữ thứ tự gốc trong mỗi nhóm
    v
}

/// Gợi ý quan hệ theo người đã mất / đang sống.
pub fn quan_he_suggestions(deceased: bool) -> Vec<&'static str> {
    if deceased {
        QUAN_HE_DECEASED.to_vec()
    } else {
        QUAN_HE_LIVING.to_vec()
    }
}

/// Viết hoa chữ cái đầu (YC-2026-07): chuẩn hóa NFC, chữ đầu tiên của chuỗi thành chữ hoa, phần còn lại giữ
/// nguyên như người dùng gõ. "ông nội" → "Ông nội"; "bà Cô Tổ họ" → "Bà Cô Tổ họ"; "" → "".
pub fn capitalize_first(s: &str) -> String {
    let n: String = s.nfc().collect();
    let mut it = n.chars();
    match it.next() {
        None => String::new(),
        Some(c) => {
            let mut out: String = c.to_uppercase().collect();
            out.push_str(it.as_str());
            out.nfc().collect()
        }
    }
}

/// Chuẩn hóa một `Person` (JSON) **khi lưu từ giao diện** (01 §3.1): viết hoa chữ đầu `danh_xung` và `quan_he`;
/// `gioi_tinh: null` → bỏ khóa (thiếu ≡ `null`, tài liệu không có giới tính giữ nguyên từng byte). Không áp cho
/// tệp sao lưu khi nhập (giữ nguyên dữ liệu người dùng).
pub fn normalize_person_names(p: &mut Value) {
    for k in ["danh_xung", "quan_he"] {
        if let Some(s) = p[k].as_str() {
            p[k] = Value::String(capitalize_first(s));
        }
    }
    if let Some(o) = p.as_object_mut() {
        if o.get("gioi_tinh").is_some_and(Value::is_null) {
            o.remove("gioi_tinh");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capitalize() {
        assert_eq!(capitalize_first("ông nội"), "Ông nội");
        assert_eq!(capitalize_first("o\u{0302}\u{0300}ng"), "Ồng");
        assert_eq!(capitalize_first("đ"), "Đ");
        assert_eq!(capitalize_first("Bà Cô Tổ họ"), "Bà Cô Tổ họ");
        assert_eq!(capitalize_first("bé đỏ"), "Bé đỏ");
        assert_eq!(capitalize_first(""), "");
        assert_eq!(capitalize_first("1 ông"), "1 ông");
    }

    #[test]
    fn suggestions() {
        let living = danh_xung_suggestions(false, None);
        assert_eq!(living.len(), 17);
        for x in DANH_XUNG_DECEASED_ONLY {
            assert!(!living.contains(&x));
        }
        assert_eq!(danh_xung_suggestions(true, None).len(), 22);
        let nam = danh_xung_suggestions(true, Some("nam"));
        assert_eq!(&nam[..3], &["Ông", "Chú", "Cậu"]);
        assert_eq!(nam.last(), Some(&"Bà Cô Tổ họ"));
        let nu = danh_xung_suggestions(false, Some("nu"));
        assert_eq!(&nu[..2], &["Bà", "Cô"]);
        assert_eq!(nu.len(), 17);
        assert!(QUAN_HE_DECEASED
            .iter()
            .chain(QUAN_HE_LIVING.iter())
            .all(|x| capitalize_first(x) == *x));
    }
}
