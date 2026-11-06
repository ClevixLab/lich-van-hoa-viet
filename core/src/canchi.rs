//! Can Chi, Nạp âm và các quy tắc truyền thống (Trực, Nhị thập bát tú, 12 thần, giờ Hoàng đạo).
//!
//! Miền của các hàm chỉ số (RS-04/C-5): can 0..9, chi 0..11, tháng âm 1..12, chi giờ 0..11.
//! Mọi hàm đều **toàn phần** trên `u32` — đầu vào được rút gọn (`%`) trước khi cộng/trừ nên
//! không bao giờ tràn số (kể cả với `overflow-checks = true`); với đầu vào trong miền, kết quả
//! giống hệt công thức trong đặc tả. Đầu vào ngoài miền cho kết quả không có ý nghĩa lịch, nên
//! lõi chỉ gọi với chỉ số đã tính từ JDN/ngày âm hợp lệ. `can_chi_name` thì kiểm miền thật sự.

/// 10 Thiên can.
pub const CAN: [&str; 10] = [
    "Giáp", "Ất", "Bính", "Đinh", "Mậu", "Kỷ", "Canh", "Tân", "Nhâm", "Quý",
];

/// 12 Địa chi.
pub const CHI: [&str; 12] = [
    "Tý", "Sửu", "Dần", "Mão", "Thìn", "Tỵ", "Ngọ", "Mùi", "Thân", "Dậu", "Tuất", "Hợi",
];

/// 12 Trực, 0 = Kiến.
pub const TRUC: [&str; 12] = [
    "Kiến", "Trừ", "Mãn", "Bình", "Định", "Chấp", "Phá", "Nguy", "Thành", "Thu", "Khai", "Bế",
];

/// Nhị thập bát tú, 0 = Giác.
pub const TU: [&str; 28] = [
    "Giác", "Cang", "Đê", "Phòng", "Tâm", "Vĩ", "Cơ", "Đẩu", "Ngưu", "Nữ", "Hư", "Nguy", "Thất",
    "Bích", "Khuê", "Lâu", "Vị", "Mão", "Tất", "Chủy", "Sâm", "Tỉnh", "Quỷ", "Liễu", "Tinh",
    "Trương", "Dực", "Chẩn",
];

/// 12 thần (Hoàng đạo / Hắc đạo), 0 = Thanh Long.
/// Thần thứ 6 (chỉ số 5) gọi là "Kim Đường" (nơi khác: Bảo Quang, Thiên Đức) — một tên duy nhất
/// trong toàn ứng dụng, khớp crate Ngọc Hạp ("Kim Đường Hoàng Đạo"); xem đặc tả §8–§9.
pub const THAN: [&str; 12] = [
    "Thanh Long",
    "Minh Đường",
    "Thiên Hình",
    "Chu Tước",
    "Kim Quỹ",
    "Kim Đường",
    "Bạch Hổ",
    "Ngọc Đường",
    "Thiên Lao",
    "Huyền Vũ",
    "Tư Mệnh",
    "Câu Trần",
];

/// Thần ở vị trí i có phải Hoàng đạo.
pub const THAN_HOANG_DAO: [bool; 12] = [
    true, true, false, false, true, true, false, true, false, false, true, false,
];

/// 30 tên Nạp âm, theo từng cặp trong vòng 60 Giáp Tý.
pub const NAP_AM: [&str; 30] = [
    "Hải Trung Kim",
    "Lô Trung Hỏa",
    "Đại Lâm Mộc",
    "Lộ Bàng Thổ",
    "Kiếm Phong Kim",
    "Sơn Đầu Hỏa",
    "Giản Hạ Thủy",
    "Thành Đầu Thổ",
    "Bạch Lạp Kim",
    "Dương Liễu Mộc",
    "Tuyền Trung Thủy",
    "Ốc Thượng Thổ",
    "Tích Lịch Hỏa",
    "Tùng Bách Mộc",
    "Trường Lưu Thủy",
    "Sa Trung Kim",
    "Sơn Hạ Hỏa",
    "Bình Địa Mộc",
    "Bích Thượng Thổ",
    "Kim Bạch Kim",
    "Phú Đăng Hỏa",
    "Thiên Hà Thủy",
    "Đại Trạch Thổ",
    "Thoa Xuyến Kim",
    "Tang Đố Mộc",
    "Đại Khê Thủy",
    "Sa Trung Thổ",
    "Thiên Thượng Hỏa",
    "Thạch Lựu Mộc",
    "Đại Hải Thủy",
];

/// Mốc Nhị thập bát tú: ngày JDN 2461350 (2026-11-05, Thứ Năm) là sao Tỉnh (chỉ số 21).
pub const TU_ANCHOR_JDN: i64 = 2461350;
/// Chỉ số tú của ngày mốc.
pub const TU_ANCHOR_INDEX: i64 = 21;

/// Can ngày (0..9) từ JDN.
pub fn day_can(jdn: i64) -> u32 {
    (jdn + 9).rem_euclid(10) as u32
}

/// Chi ngày (0..11) từ JDN.
pub fn day_chi(jdn: i64) -> u32 {
    (jdn + 1).rem_euclid(12) as u32
}

/// Can năm âm lịch.
pub fn year_can(ly: i32) -> u32 {
    (ly as i64 + 6).rem_euclid(10) as u32
}

/// Chi năm âm lịch.
pub fn year_chi(ly: i32) -> u32 {
    (ly as i64 + 8).rem_euclid(12) as u32
}

/// Can tháng âm `lm` (1..12) của năm có can `ycan`. Tháng nhuận dùng chung số tháng.
pub fn month_can(ycan: u32, lm: u32) -> u32 {
    let can_thang1 = (ycan % 5) * 2 + 2; // Giáp/Kỷ → Bính, Ất/Canh → Mậu, …
                                         // = (can_thang1 + lm − 1) mod 10, viết không trừ để không tràn khi lm = 0.
    (can_thang1 + lm % 10 + 9) % 10
}

/// Chi tháng âm `lm`: tháng 1 = Dần.
pub fn month_chi(lm: u32) -> u32 {
    (lm % 12 + 1) % 12
}

/// Can của giờ Tý đầu ngày theo can ngày.
pub fn hour_ty_can(dcan: u32) -> u32 {
    (dcan % 5) * 2
}

/// Ghép tên Can Chi. `None` nếu `can ≥ 10`, `chi ≥ 12` hoặc cặp lệch chẵn lẻ (ví dụ "Ất Tý"
/// không tồn tại trong vòng 60 Giáp Tý) — trước đây panic chỉ số mảng (C-5).
pub fn can_chi_name(can: u32, chi: u32) -> Option<String> {
    if can >= 10 || chi >= 12 || can % 2 != chi % 2 {
        return None;
    }
    Some(format!("{} {}", CAN[can as usize], CHI[chi as usize]))
}

/// Chỉ số trong vòng 60 Giáp Tý (0 = Giáp Tý) của cặp (can, chi) cùng tính chẵn lẻ.
pub fn sexagenary_index(can: u32, chi: u32) -> u32 {
    // n ≡ can (mod 10), n ≡ chi (mod 12)
    // nghịch đảo CRT cho mod 10 và mod 12 (yêu cầu can ≡ chi mod 2)
    (6 * (can % 10) + 55 * (chi % 12)) % 60
}

/// Chỉ số Nạp âm (0..29) của cặp can chi.
pub fn nap_am_index(can: u32, chi: u32) -> u32 {
    sexagenary_index(can, chi) / 2
}

/// Trực (0 = Kiến) từ chi ngày và chi tháng tiết.
pub fn truc_index(dchi: u32, tiet_month_chi: u32) -> u32 {
    (dchi % 12 + 12 - tiet_month_chi % 12) % 12
}

/// Nhị thập bát tú (0 = Giác) theo JDN.
pub fn tu_index(jdn: i64) -> u32 {
    (jdn - TU_ANCHOR_JDN + TU_ANCHOR_INDEX).rem_euclid(28) as u32
}

/// Chi ngày khởi Thanh Long theo số tháng âm (1..12).
pub fn thanh_long_day_chi(lm: u32) -> u32 {
    // 1,7 → Tý; 2,8 → Dần; 3,9 → Thìn; 4,10 → Ngọ; 5,11 → Thân; 6,12 → Tuất
    // = ((lm − 1) mod 6)·2, viết không trừ: `lm - 1` tràn với lm = 0 (RS-04, canchi.rs:151 cũ).
    ((lm % 6 + 5) % 6) * 2
}

/// Thần của ngày (0 = Thanh Long) theo số tháng âm và chi ngày.
pub fn than_index(lm: u32, dchi: u32) -> u32 {
    (dchi % 12 + 12 - thanh_long_day_chi(lm)) % 12
}

/// Chi giờ khởi Thanh Long theo chi ngày.
pub fn thanh_long_hour_chi(dchi: u32) -> u32 {
    // Tý/Ngọ → Thân; Sửu/Mùi → Tuất; Dần/Thân → Tý; Mão/Dậu → Dần; Thìn/Tuất → Thìn; Tỵ/Hợi → Ngọ
    (8 + 2 * (dchi % 6)) % 12
}

/// Thần của giờ có chi `hchi` trong ngày có chi `dchi`.
pub fn hour_than_index(dchi: u32, hchi: u32) -> u32 {
    (hchi % 12 + 12 - thanh_long_hour_chi(dchi)) % 12
}

/// Các chi giờ Hoàng đạo (6 giờ, theo thứ tự Tý → Hợi).
pub fn gio_hoang_dao(dchi: u32) -> [u32; 6] {
    let mut out = [0u32; 6];
    let mut n = 0;
    for h in 0..12 {
        if THAN_HOANG_DAO[hour_than_index(dchi, h) as usize] {
            out[n] = h;
            n += 1;
        }
    }
    debug_assert_eq!(n, 6);
    out
}

/// Giờ đồng hồ (from, to) của chi giờ: Tý 23–1, Sửu 1–3, … (`hchi` rút gọn mod 12).
pub fn hour_range(hchi: u32) -> (u32, u32) {
    let h = hchi % 12;
    ((h * 2 + 23) % 24, h * 2 + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sexagenary() {
        for n in 0..60u32 {
            assert_eq!(sexagenary_index(n % 10, n % 12), n);
        }
        assert_eq!(NAP_AM[nap_am_index(9, 7) as usize], "Dương Liễu Mộc"); // Quý Mùi
        assert_eq!(NAP_AM[nap_am_index(0, 0) as usize], "Hải Trung Kim");
        assert_eq!(NAP_AM[nap_am_index(9, 11) as usize], "Đại Hải Thủy"); // Quý Hợi
    }

    #[test]
    fn month_can_table() {
        // Giáp → Bính Dần, Ất → Mậu Dần, Bính → Canh Dần, Đinh → Nhâm Dần, Mậu → Giáp Dần
        assert_eq!(month_can(0, 1), 2);
        assert_eq!(month_can(1, 1), 4);
        assert_eq!(month_can(2, 1), 6);
        assert_eq!(month_can(3, 1), 8);
        assert_eq!(month_can(4, 1), 0);
        assert_eq!(month_can(5, 1), 2);
        assert_eq!(month_can(2, 9), 4); // Bính Ngọ tháng 9 → Mậu Tuất
    }

    #[test]
    fn hour_ty() {
        assert_eq!(hour_ty_can(0), 0);
        assert_eq!(hour_ty_can(1), 2);
        assert_eq!(hour_ty_can(2), 4);
        assert_eq!(hour_ty_can(3), 6);
        assert_eq!(hour_ty_can(4), 8);
        assert_eq!(hour_ty_can(9), 8);
    }

    #[test]
    fn gio_hd() {
        // Ngày Mùi → Dần, Mão, Tỵ, Thân, Tuất, Hợi
        assert_eq!(gio_hoang_dao(7), [2, 3, 5, 8, 10, 11]);
        // Ngày Dậu → Tý, Dần, Mão, Ngọ, Mùi, Dậu
        assert_eq!(gio_hoang_dao(9), [0, 2, 3, 6, 7, 9]);
        // Ngày Tý → Tý, Sửu, Mão, Ngọ, Thân, Dậu (bảng truyền thống)
        assert_eq!(gio_hoang_dao(0), [0, 1, 3, 6, 8, 9]);
        // Bảng truyền thống đủ 6 cặp ngày
        assert_eq!(gio_hoang_dao(1), [2, 3, 5, 8, 10, 11]); // Sửu
        assert_eq!(gio_hoang_dao(2), [0, 1, 4, 5, 7, 10]); // Dần: Tý Sửu Thìn Tỵ Mùi Tuất
        assert_eq!(gio_hoang_dao(4), [2, 4, 5, 8, 9, 11]); // Thìn: Dần Thìn Tỵ Thân Dậu Hợi
        assert_eq!(gio_hoang_dao(5), [1, 4, 6, 7, 10, 11]); // Tỵ: Sửu Thìn Ngọ Mùi Tuất Hợi
        for c in 0..6 {
            assert_eq!(gio_hoang_dao(c), gio_hoang_dao(c + 6));
        }
        assert_eq!(hour_range(0), (23, 1));
        assert_eq!(hour_range(1), (1, 3));
        assert_eq!(hour_range(11), (21, 23));
    }

    #[test]
    fn tu_weekday_consistency() {
        // Quy tắc truyền thống: Giác, Đẩu, Khuê, Tỉnh (Mộc) rơi vào Thứ Năm;
        // tổng quát: sao i rơi vào thứ (4 + i) mod 7.
        for j in 2451545i64..2451545 + 400 {
            let i = tu_index(j) as i64;
            assert_eq!((4 + i) % 7, crate::jd::weekday(j) as i64);
        }
    }

    #[test]
    fn total_on_u32_no_overflow() {
        // RS-04: các giá trị từng tràn/wrap.
        for x in [0u32, 1, 11, 12, 13, 100, u32::MAX - 1, u32::MAX] {
            let _ = (
                month_can(x, x),
                month_chi(x),
                hour_ty_can(x),
                sexagenary_index(x, x),
            );
            let _ = (truc_index(x, x), thanh_long_day_chi(x), than_index(x, x));
            let _ = (
                thanh_long_hour_chi(x),
                hour_than_index(x, x),
                gio_hoang_dao(x),
            );
            let (a, b) = hour_range(x);
            assert!(a < 24 && b < 24);
        }
        // Trong miền: giống công thức (lm − 1) mod 6 · 2 và (c + lm − 1) mod 10.
        for lm in 1..=12u32 {
            assert_eq!(thanh_long_day_chi(lm), ((lm - 1) % 6) * 2);
            for yc in 0..10u32 {
                assert_eq!(month_can(yc, lm), ((yc % 5) * 2 + 2 + lm - 1) % 10);
            }
        }
        assert_eq!(hour_range(100), hour_range(4));
    }

    #[test]
    fn can_chi_name_checks_domain() {
        assert_eq!(can_chi_name(2, 6).as_deref(), Some("Bính Ngọ"));
        assert_eq!(can_chi_name(10, 12), None);
        assert_eq!(can_chi_name(1, 0), None); // "Ất Tý" không tồn tại
        assert_eq!(can_chi_name(u32::MAX, 0), None);
    }

    #[test]
    fn truc_and_than() {
        assert_eq!(TRUC[truc_index(7, 10) as usize], "Thu");
        assert_eq!(THAN[than_index(9, 7) as usize], "Chu Tước");
    }
}
