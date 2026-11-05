//! Bảng quy tắc sao Ngọc Hạp (dữ liệu truyền thống). Mỗi sao mang mã quy tắc
//! (G.. = sao tốt, B.. = sao xấu, S.. = ngày đặc biệt, T.. = 12 thần) khớp với
//! `content/ngochap/rules.md`, nơi ghi nguồn và mâu thuẫn giữa các nguồn.
//!
//! Chỉ số tháng trong bảng: phần tử 0 = tháng Giêng (tháng 1 âm lịch) … 11 = tháng Chạp.
//! Bảng theo mùa: 0 = Xuân (tháng 1–3), 1 = Hạ (4–6), 2 = Thu (7–9), 3 = Đông (10–12).

// Thiên can
pub(crate) const GIAP: u8 = 0;
pub(crate) const AT: u8 = 1;
pub(crate) const BINH: u8 = 2;
pub(crate) const DINH: u8 = 3;
pub(crate) const MAU: u8 = 4;
pub(crate) const KY: u8 = 5;
pub(crate) const CANH: u8 = 6;
pub(crate) const TAN: u8 = 7;
pub(crate) const NHAM: u8 = 8;
pub(crate) const QUY: u8 = 9;

// Địa chi (tiền tố Z để không trùng tên với can)
pub(crate) const ZTY: u8 = 0;
pub(crate) const ZSUU: u8 = 1;
pub(crate) const ZDAN: u8 = 2;
pub(crate) const ZMAO: u8 = 3;
pub(crate) const ZTHIN: u8 = 4;
pub(crate) const ZTI: u8 = 5;
pub(crate) const ZNGO: u8 = 6;
pub(crate) const ZMUI: u8 = 7;
pub(crate) const ZTHAN: u8 = 8;
pub(crate) const ZDAU: u8 = 9;
pub(crate) const ZTUAT: u8 = 10;
pub(crate) const ZHOI: u8 = 11;

pub(crate) const CAN_NAME: [&str; 10] = [
    "Giáp", "Ất", "Bính", "Đinh", "Mậu", "Kỷ", "Canh", "Tân", "Nhâm", "Quý",
];
pub(crate) const CHI_NAME: [&str; 12] = [
    "Tý", "Sửu", "Dần", "Mão", "Thìn", "Tỵ", "Ngọ", "Mùi", "Thân", "Dậu", "Tuất", "Hợi",
];

/// Mã việc (chỉ số trong `ACTIVITIES`).
pub(crate) type Act = u8;
pub(crate) const CUOI_HOI: Act = 0;
pub(crate) const XAY_DUNG: Act = 1;
pub(crate) const KHAI_TRUONG: Act = 2;
pub(crate) const AN_TANG: Act = 3;
pub(crate) const TE_TU: Act = 4;
pub(crate) const DONG_THO: Act = 5;
pub(crate) const XUAT_HANH: Act = 6;
pub(crate) const GIAO_DICH: Act = 7;
pub(crate) const CAU_TAI: Act = 8;
pub(crate) const TO_TUNG: Act = 9;
pub(crate) const LAM_PHUC: Act = 10;
pub(crate) const NHAP_TRACH: Act = 11;
pub(crate) const KHAI_NGHIEP: Act = 12;
pub(crate) const CAU_TU: Act = 13;
pub(crate) const NHAP_HOC: Act = 14;
pub(crate) const CAU_SUC_KHOE: Act = 15;
pub(crate) const N_ACT: usize = 16;

/// Việc "cá nhân": tuổi xung với ngày thì không cho phép (allowed = false).
/// Tế tự, làm phúc, an táng không tính là việc cá nhân của người xem (xem SCORING.md).
pub(crate) const PERSONAL: [Act; 13] = [
    CUOI_HOI,
    XAY_DUNG,
    KHAI_TRUONG,
    DONG_THO,
    XUAT_HANH,
    GIAO_DICH,
    CAU_TAI,
    TO_TUNG,
    NHAP_TRACH,
    KHAI_NGHIEP,
    CAU_TU,
    NHAP_HOC,
    CAU_SUC_KHOE,
];

/// Tập việc chịu ảnh hưởng của một sao.
#[derive(Clone, Copy)]
pub(crate) enum Acts {
    None,
    /// "Mọi việc".
    All,
    /// "Mọi việc, nhất là …" — các việc nêu đích danh được cộng thêm.
    AllEsp(&'static [Act]),
    Some(&'static [Act]),
}

impl Acts {
    pub(crate) fn contains(self, a: Act) -> bool {
        match self {
            Acts::None => false,
            Acts::All | Acts::AllEsp(_) => true,
            Acts::Some(list) => list.contains(&a),
        }
    }
    /// Việc được nêu đích danh (không phải "mọi việc").
    pub(crate) fn names(self, a: Act) -> bool {
        match self {
            Acts::Some(list) | Acts::AllEsp(list) => list.contains(&a),
            _ => false,
        }
    }
}

/// Điều kiện ngày trong bảng tháng có cả can lẫn chi (ví dụ Thiên Đức).
#[derive(Clone, Copy)]
pub(crate) enum Cond {
    Can(u8),
    Chi(u8),
}

/// Cách định vị sao.
#[derive(Clone, Copy)]
pub(crate) enum Rule {
    /// Mỗi tháng âm một chi ngày.
    MonthChi([u8; 12]),
    /// Mỗi tháng âm hai chi ngày.
    MonthChi2([[u8; 2]; 12]),
    /// Mỗi tháng âm một can ngày.
    MonthCan([u8; 12]),
    /// Mỗi tháng âm một can hoặc một chi.
    MonthCond([Cond; 12]),
    /// Mỗi tháng âm một can chi ngày.
    MonthCanChi([(u8, u8); 12]),
    /// Như `MonthChi` nhưng có tháng nguồn để trống (tháng đó không có sao).
    MonthChiOpt([Option<u8>; 12]),
    /// Mỗi mùa (theo tháng âm) một danh sách chi ngày.
    SeasonChi([&'static [u8]; 4]),
    /// Mỗi mùa một danh sách can ngày.
    SeasonCan([&'static [u8]; 4]),
    /// Mỗi mùa một can chi ngày.
    SeasonCanChi([(u8, u8); 4]),
    /// Tập can chi cố định, không phụ thuộc tháng.
    CanChiSet(&'static [(u8, u8)]),
    /// Tập can chi theo nhóm tháng Mạnh (1,4,7,10), Trọng (2,5,8,11), Quý (3,6,9,12).
    CanChiByGroup([&'static [(u8, u8)]; 3]),
    /// Ngày âm lịch cố định mọi tháng.
    LunarDays(&'static [u8]),
    /// Cặp (tháng âm, ngày âm) cố định trong năm.
    MonthDays(&'static [(u8, u8)]),
    /// Thập ác đại bại: theo can năm, tháng âm và can chi ngày.
    ThapAc,
    /// Một trong 12 thần Hoàng đạo / Hắc đạo (chỉ số, 0 = Thanh Long).
    Than(u8),
}

/// Một sao trong hệ Ngọc Hạp.
#[derive(Clone, Copy)]
pub(crate) struct Star {
    /// Mã quy tắc, khớp rules.md.
    pub id: &'static str,
    /// Tên hiển thị tiếng Việt.
    pub name: &'static str,
    pub good: bool,
    /// Trọng số (dương), xem SCORING.md.
    pub weight: i32,
    pub rule: Rule,
    /// Việc sao tốt trợ giúp (sao tốt) — bỏ qua với sao xấu.
    pub tot: Acts,
    /// Việc sao kỵ (sao xấu, hoặc lời "kỵ" kèm theo sao tốt).
    pub ky: Acts,
    /// Việc mà sao này là "kỵ chính" (đại kỵ): có sao thì không cho phép.
    pub major: Acts,
}

const fn x3(a: [u8; 4]) -> [u8; 12] {
    [
        a[0], a[1], a[2], a[3], a[0], a[1], a[2], a[3], a[0], a[1], a[2], a[3],
    ]
}
const fn x2(a: [u8; 6]) -> [u8; 12] {
    [
        a[0], a[1], a[2], a[3], a[4], a[5], a[0], a[1], a[2], a[3], a[4], a[5],
    ]
}
const fn x4(a: [u8; 3]) -> [u8; 12] {
    [
        a[0], a[1], a[2], a[0], a[1], a[2], a[0], a[1], a[2], a[0], a[1], a[2],
    ]
}

const ALL: Acts = Acts::All;
const NONE: Acts = Acts::None;
const fn s(list: &'static [Act]) -> Acts {
    Acts::Some(list)
}
const fn esp(list: &'static [Act]) -> Acts {
    Acts::AllEsp(list)
}

const fn good(id: &'static str, name: &'static str, weight: i32, rule: Rule, tot: Acts) -> Star {
    Star {
        id,
        name,
        good: true,
        weight,
        rule,
        tot,
        ky: NONE,
        major: NONE,
    }
}
const fn bad(
    id: &'static str,
    name: &'static str,
    weight: i32,
    rule: Rule,
    ky: Acts,
    major: Acts,
) -> Star {
    Star {
        id,
        name,
        good: false,
        weight,
        rule,
        tot: NONE,
        ky,
        major,
    }
}

// ---- Danh sách việc dùng lại ----
const A_TAI: &[Act] = &[CAU_TAI, KHAI_TRUONG];
const A_CUOI: &[Act] = &[CUOI_HOI];
const A_XAY: &[Act] = &[XAY_DUNG];
const A_TANG: &[Act] = &[AN_TANG];
const A_XH: &[Act] = &[XUAT_HANH];
const A_DT: &[Act] = &[DONG_THO];
const A_TE: &[Act] = &[TE_TU];

/// Bảng sao tốt (nguồn chính: N01 – Ngọc hạp thông thư triều Nguyễn, trích Tân Việt & Thiều Phong 1997).
pub(crate) const GOOD_STARS: &[Star] = &[
    good(
        "G01",
        "Thiên Đức",
        2,
        Rule::MonthCond([
            Cond::Can(DINH),
            Cond::Chi(ZTHAN),
            Cond::Can(NHAM),
            Cond::Can(TAN),
            Cond::Chi(ZHOI),
            Cond::Can(GIAP),
            Cond::Can(QUY),
            Cond::Chi(ZDAN),
            Cond::Can(BINH),
            Cond::Can(AT),
            Cond::Chi(ZTI),
            Cond::Can(CANH),
        ]),
        ALL,
    ),
    good(
        "G02",
        "Thiên Đức Hợp",
        1,
        Rule::MonthCond([
            Cond::Can(NHAM),
            Cond::Chi(ZTI),
            Cond::Can(DINH),
            Cond::Can(BINH),
            Cond::Chi(ZDAN),
            Cond::Can(KY),
            Cond::Can(MAU),
            Cond::Chi(ZHOI),
            Cond::Can(TAN),
            Cond::Can(CANH),
            Cond::Chi(ZTHAN),
            Cond::Can(AT),
        ]),
        ALL,
    ),
    good(
        "G03",
        "Nguyệt Đức",
        2,
        Rule::MonthCan(x3([BINH, GIAP, NHAM, CANH])),
        ALL,
    ),
    Star {
        id: "G04",
        name: "Nguyệt Đức Hợp",
        good: true,
        weight: 1,
        rule: Rule::MonthCan(x3([TAN, KY, DINH, AT])),
        tot: ALL,
        ky: s(&[TO_TUNG]),
        major: NONE,
    },
    good(
        "G05",
        "Thiên Hỷ",
        1,
        Rule::MonthChi([
            ZTUAT, ZHOI, ZTY, ZSUU, ZDAN, ZMAO, ZTHIN, ZTI, ZNGO, ZMUI, ZTHAN, ZDAU,
        ]),
        esp(A_CUOI),
    ),
    good(
        "G06",
        "Thiên Phú",
        1,
        Rule::MonthChi([
            ZTHIN, ZTI, ZNGO, ZMUI, ZTHAN, ZDAU, ZTUAT, ZHOI, ZTY, ZSUU, ZDAN, ZMAO,
        ]),
        esp(&[XAY_DUNG, KHAI_TRUONG, AN_TANG, CAU_TAI]),
    ),
    good(
        "G07",
        "Thiên Quý",
        1,
        Rule::SeasonCan([&[GIAP, AT], &[BINH, DINH], &[CANH, TAN], &[NHAM, QUY]]),
        ALL,
    ),
    Star {
        id: "G08",
        name: "Thiên Xá",
        good: true,
        weight: 1,
        rule: Rule::SeasonCanChi([(MAU, ZDAN), (GIAP, ZNGO), (MAU, ZTHAN), (GIAP, ZTY)]),
        tot: s(&[TE_TU, TO_TUNG]),
        ky: s(&[DONG_THO]),
        major: NONE,
    },
    good(
        "G09",
        "Sinh Khí",
        1,
        Rule::MonthChi([
            ZTY, ZSUU, ZDAN, ZMAO, ZTHIN, ZTI, ZNGO, ZMUI, ZTHAN, ZDAU, ZTUAT, ZHOI,
        ]),
        esp(&[XAY_DUNG, DONG_THO]),
    ),
    good(
        "G10",
        "Thiên Thành",
        1,
        Rule::MonthChi(x2([ZMUI, ZDAU, ZHOI, ZSUU, ZMAO, ZTI])),
        ALL,
    ),
    good(
        "G11",
        "Thiên Quan",
        1,
        Rule::MonthChi(x2([ZTUAT, ZTY, ZDAN, ZTHIN, ZNGO, ZTHAN])),
        ALL,
    ),
    good(
        "G12",
        "Thiên Mã",
        1,
        Rule::MonthChi(x2([ZNGO, ZTHAN, ZTUAT, ZTY, ZDAN, ZTHIN])),
        s(&[XUAT_HANH, GIAO_DICH, CAU_TAI]),
    ),
    good(
        "G13",
        "Thiên Tài",
        1,
        Rule::MonthChi(x2([ZTHIN, ZNGO, ZTHAN, ZTUAT, ZTY, ZDAN])),
        s(A_TAI),
    ),
    good(
        "G14",
        "Địa Tài",
        1,
        Rule::MonthChi(x2([ZTI, ZMUI, ZDAU, ZHOI, ZSUU, ZMAO])),
        s(A_TAI),
    ),
    good(
        "G15",
        "Nguyệt Tài",
        1,
        Rule::MonthChi(x2([ZNGO, ZTI, ZTI, ZMUI, ZDAU, ZHOI])),
        s(&[CAU_TAI, KHAI_TRUONG, XUAT_HANH, GIAO_DICH, NHAP_TRACH]),
    ),
    good(
        "G16",
        "Nguyệt Ân",
        1,
        Rule::MonthCan([
            BINH, DINH, CANH, KY, MAU, TAN, NHAM, QUY, CANH, AT, GIAP, TAN,
        ]),
        ALL,
    ),
    good(
        "G17",
        "Nguyệt Không",
        1,
        Rule::MonthCan(x3([NHAM, CANH, BINH, GIAP])),
        s(&[XAY_DUNG]),
    ),
    good(
        "G18",
        "Minh Tinh",
        1,
        Rule::MonthChi(x2([ZTHAN, ZTUAT, ZTY, ZDAN, ZTHIN, ZNGO])),
        s(&[TO_TUNG, AN_TANG]),
    ),
    good(
        "G19",
        "Thánh Tâm",
        1,
        Rule::MonthChi([
            ZHOI, ZTI, ZTY, ZNGO, ZSUU, ZMUI, ZDAN, ZTHAN, ZMAO, ZDAU, ZTHIN, ZTUAT,
        ]),
        esp(&[TE_TU, LAM_PHUC]),
    ),
    good(
        "G20",
        "Ngũ Phú",
        1,
        Rule::MonthChi(x3([ZHOI, ZDAN, ZTI, ZTHAN])),
        ALL,
    ),
    good(
        "G21",
        "Lộc Khố",
        1,
        Rule::MonthChi([
            ZTHIN, ZTI, ZNGO, ZMUI, ZTHAN, ZDAU, ZTUAT, ZHOI, ZTY, ZSUU, ZDAN, ZMAO,
        ]),
        s(&[CAU_TAI, GIAO_DICH, KHAI_TRUONG]),
    ),
    good(
        "G22",
        "Phúc Sinh",
        1,
        Rule::MonthChi([
            ZDAU, ZMAO, ZTUAT, ZTHIN, ZHOI, ZTI, ZTY, ZNGO, ZSUU, ZMUI, ZDAN, ZTHAN,
        ]),
        ALL,
    ),
    good(
        "G23",
        "Cát Khánh",
        1,
        Rule::MonthChi([
            ZDAU, ZDAN, ZHOI, ZTHIN, ZSUU, ZNGO, ZMAO, ZTHAN, ZTI, ZTUAT, ZMUI, ZTY,
        ]),
        ALL,
    ),
    good(
        "G24",
        "Âm Đức",
        1,
        Rule::MonthChi(x2([ZDAU, ZMUI, ZTI, ZMAO, ZSUU, ZHOI])),
        ALL,
    ),
    good(
        "G25",
        "U Vi Tinh",
        1,
        Rule::MonthChi([
            ZHOI, ZTHIN, ZSUU, ZNGO, ZMAO, ZTHAN, ZTI, ZTUAT, ZMUI, ZTY, ZDAU, ZDAN,
        ]),
        ALL,
    ),
    good(
        "G26",
        "Mãn Đức Tinh",
        1,
        Rule::MonthChi([
            ZDAN, ZMUI, ZTHIN, ZDAU, ZNGO, ZHOI, ZTHAN, ZSUU, ZTUAT, ZMAO, ZTY, ZTI,
        ]),
        ALL,
    ),
    good(
        "G27",
        "Kính Tâm",
        1,
        Rule::MonthChi([
            ZMUI, ZSUU, ZTHAN, ZDAN, ZDAU, ZMAO, ZTUAT, ZTHIN, ZHOI, ZTI, ZTY, ZNGO,
        ]),
        s(A_TANG),
    ),
    good(
        "G28",
        "Tuế Hợp",
        1,
        Rule::MonthChi([
            ZSUU, ZTY, ZHOI, ZTUAT, ZDAU, ZTHAN, ZMUI, ZNGO, ZTI, ZTHIN, ZMAO, ZDAN,
        ]),
        ALL,
    ),
    good(
        "G29",
        "Nguyệt Giải",
        1,
        Rule::MonthChi([
            ZTHAN, ZTHAN, ZDAU, ZDAU, ZTUAT, ZTUAT, ZHOI, ZHOI, ZNGO, ZNGO, ZMUI, ZMUI,
        ]),
        ALL,
    ),
    // G30 Hoạt Điệu: bị bỏ khi trùng Thụ Tử (xem lib.rs).
    good(
        "G30",
        "Hoạt Điệu",
        1,
        Rule::MonthChi([
            ZTI, ZTUAT, ZMUI, ZTY, ZDAU, ZDAN, ZHOI, ZTHIN, ZSUU, ZNGO, ZMAO, ZTHAN,
        ]),
        ALL,
    ),
    good(
        "G31",
        "Giải Thần",
        1,
        Rule::MonthChi([
            ZTHAN, ZTHAN, ZTUAT, ZTUAT, ZTY, ZTY, ZDAN, ZDAN, ZTHIN, ZTHIN, ZNGO, ZNGO,
        ]),
        s(&[TE_TU, TO_TUNG]),
    ),
    good(
        "G32",
        "Phổ Hộ",
        1,
        Rule::MonthChi([
            ZTHAN, ZDAN, ZDAU, ZMAO, ZTUAT, ZTHIN, ZHOI, ZTI, ZTY, ZNGO, ZSUU, ZMUI,
        ]),
        esp(&[LAM_PHUC, CUOI_HOI, XUAT_HANH]),
    ),
    good(
        "G33",
        "Ích Hậu",
        1,
        Rule::MonthChi([
            ZTY, ZNGO, ZSUU, ZMUI, ZDAN, ZTHAN, ZMAO, ZDAU, ZTHIN, ZTUAT, ZTI, ZHOI,
        ]),
        esp(A_CUOI),
    ),
    good(
        "G34",
        "Tục Thế",
        1,
        Rule::MonthChi([
            ZSUU, ZMUI, ZDAN, ZTHAN, ZMAO, ZDAU, ZTHIN, ZTUAT, ZTI, ZHOI, ZNGO, ZTY,
        ]),
        esp(A_CUOI),
    ),
    good(
        "G35",
        "Yếu Yên",
        1,
        Rule::MonthChi([
            ZDAN, ZTHAN, ZMAO, ZDAU, ZTHIN, ZTUAT, ZTI, ZHOI, ZNGO, ZTY, ZMUI, ZSUU,
        ]),
        s(A_CUOI),
    ),
    good(
        "G36",
        "Dịch Mã",
        1,
        Rule::MonthChi(x3([ZTHAN, ZTI, ZDAN, ZHOI])),
        s(&[XUAT_HANH, CAU_SUC_KHOE]),
    ),
    good(
        "G37",
        "Tam Hợp",
        1,
        Rule::MonthChi2([
            [ZNGO, ZTUAT],
            [ZMUI, ZHOI],
            [ZTHAN, ZTY],
            [ZDAU, ZSUU],
            [ZTUAT, ZDAN],
            [ZHOI, ZMAO],
            [ZTY, ZTHIN],
            [ZSUU, ZTI],
            [ZDAN, ZNGO],
            [ZMAO, ZMUI],
            [ZTHIN, ZTHAN],
            [ZTI, ZDAU],
        ]),
        ALL,
    ),
    good(
        "G38",
        "Lục Hợp",
        1,
        Rule::MonthChi([
            ZHOI, ZTUAT, ZDAU, ZTHAN, ZMUI, ZNGO, ZTI, ZTHIN, ZMAO, ZDAN, ZSUU, ZTY,
        ]),
        ALL,
    ),
    good(
        "G39",
        "Mẫu Thương",
        1,
        Rule::SeasonChi([&[ZHOI, ZTY], &[ZDAN, ZMAO], &[ZTHIN, ZSUU], &[ZTHAN, ZDAU]]),
        s(A_TAI),
    ),
    good(
        "G40",
        "Phúc Hậu",
        1,
        Rule::SeasonChi([&[ZDAN], &[ZTI], &[ZTHAN], &[ZHOI]]),
        s(A_TAI),
    ),
    good(
        "G41",
        "Đại Hồng Sa",
        1,
        Rule::SeasonChi([&[ZTY, ZSUU], &[ZTHIN, ZTI], &[ZNGO, ZMUI], &[ZTHAN, ZTUAT]]),
        ALL,
    ),
    good(
        "G42",
        "Dân Nhật",
        1,
        Rule::SeasonChi([&[ZNGO], &[ZDAU], &[ZTY], &[ZMAO]]),
        s(&[DONG_THO, XAY_DUNG]),
    ),
    good(
        "G43",
        "Hoàng Ân",
        1,
        Rule::MonthChi([
            ZTUAT, ZSUU, ZDAN, ZTI, ZDAU, ZMAO, ZTY, ZNGO, ZHOI, ZTHIN, ZTHAN, ZMUI,
        ]),
        s(&[XAY_DUNG, KHAI_TRUONG]),
    ),
    good(
        "G44",
        "Thiên Ân",
        1,
        Rule::CanChiSet(&[
            (GIAP, ZTY),
            (AT, ZSUU),
            (BINH, ZDAN),
            (DINH, ZMAO),
            (MAU, ZTHIN),
            (KY, ZMAO),
            (CANH, ZTHIN),
            (TAN, ZTI),
            (NHAM, ZNGO),
            (QUY, ZMUI),
            (KY, ZDAU),
            (CANH, ZTUAT),
            (TAN, ZHOI),
            (NHAM, ZTY),
            (QUY, ZSUU),
        ]),
        ALL,
    ),
    good(
        "G45",
        "Sát Cống",
        2,
        Rule::CanChiByGroup([
            &[
                (DINH, ZMAO),
                (BINH, ZTY),
                (AT, ZDAU),
                (GIAP, ZNGO),
                (QUY, ZMAO),
                (NHAM, ZTY),
                (TAN, ZDAU),
            ],
            &[
                (BINH, ZDAN),
                (AT, ZHOI),
                (GIAP, ZTHAN),
                (QUY, ZTI),
                (NHAM, ZDAN),
                (TAN, ZHOI),
                (CANH, ZTHAN),
            ],
            &[
                (AT, ZSUU),
                (GIAP, ZTUAT),
                (QUY, ZMUI),
                (NHAM, ZTHIN),
                (TAN, ZSUU),
                (CANH, ZTUAT),
                (KY, ZMUI),
            ],
        ]),
        ALL,
    ),
    good(
        "G46",
        "Trực Tinh",
        2,
        Rule::CanChiByGroup([
            &[
                (MAU, ZTHIN),
                (DINH, ZSUU),
                (BINH, ZTUAT),
                (AT, ZMUI),
                (GIAP, ZTHIN),
                (QUY, ZSUU),
                (NHAM, ZTUAT),
            ],
            &[
                (DINH, ZMAO),
                (BINH, ZTY),
                (AT, ZDAU),
                (GIAP, ZNGO),
                (QUY, ZMAO),
                (NHAM, ZTY),
                (TAN, ZDAU),
            ],
            &[
                (BINH, ZDAN),
                (AT, ZHOI),
                (GIAP, ZTHAN),
                (QUY, ZTI),
                (NHAM, ZDAN),
                (TAN, ZHOI),
                (CANH, ZTHAN),
            ],
        ]),
        ALL,
    ),
    good(
        "G47",
        "Nhân Chuyên",
        2,
        Rule::CanChiByGroup([
            &[
                (TAN, ZMUI),
                (CANH, ZTHIN),
                (KY, ZSUU),
                (MAU, ZTUAT),
                (DINH, ZMUI),
                (BINH, ZTHIN),
            ],
            &[
                (CANH, ZNGO),
                (KY, ZMAO),
                (MAU, ZTY),
                (DINH, ZDAU),
                (BINH, ZNGO),
                (AT, ZMAO),
            ],
            &[
                (KY, ZTI),
                (MAU, ZDAN),
                (DINH, ZHOI),
                (BINH, ZTHAN),
                (AT, ZTI),
                (GIAP, ZDAN),
                (QUY, ZHOI),
            ],
        ]),
        ALL,
    ),
    good(
        "G48",
        "Thiên Thụy",
        1,
        Rule::CanChiSet(&[
            (MAU, ZDAN),
            (KY, ZMAO),
            (CANH, ZDAN),
            (TAN, ZTI),
            (NHAM, ZTY),
        ]),
        ALL,
    ),
    good(
        "G49",
        "Quan Nhật",
        1,
        Rule::MonthChiOpt([
            None,
            Some(ZMAO),
            None,
            None,
            Some(ZNGO),
            None,
            None,
            Some(ZDAU),
            None,
            None,
            Some(ZTY),
            None,
        ]),
        s(&[KHAI_NGHIEP]),
    ),
];

/// Bảng sao xấu (nguồn chính: N02 – "Bảng kê các sao xấu", cùng dòng sách Tân Việt & Thiều Phong 1997).
pub(crate) const BAD_STARS: &[Star] = &[
    bad(
        "B01",
        "Thiên Cương",
        3,
        Rule::MonthChi([
            ZTI, ZTY, ZMUI, ZDAN, ZDAU, ZTHIN, ZHOI, ZNGO, ZSUU, ZTHAN, ZMAO, ZTUAT,
        ]),
        ALL,
        ALL,
    ),
    bad(
        "B02",
        "Thiên Lại",
        1,
        Rule::MonthChi(x3([ZDAU, ZNGO, ZMAO, ZTY])),
        ALL,
        NONE,
    ),
    bad(
        "B03",
        "Thiên Ngục",
        1,
        Rule::MonthChi(x3([ZTY, ZMAO, ZNGO, ZDAU])),
        ALL,
        NONE,
    ),
    bad(
        "B04",
        "Thiên Hỏa",
        1,
        Rule::MonthChi(x3([ZTY, ZMAO, ZNGO, ZDAU])),
        s(A_XAY),
        s(A_XAY),
    ),
    bad(
        "B05",
        "Tiểu Hồng Sa",
        1,
        Rule::MonthChi(x4([ZTI, ZDAU, ZSUU])),
        ALL,
        NONE,
    ),
    bad(
        "B06",
        "Đại Hao",
        3,
        Rule::MonthChi([
            ZNGO, ZMUI, ZTHAN, ZDAU, ZTUAT, ZHOI, ZTY, ZSUU, ZDAN, ZMAO, ZTHIN, ZTI,
        ]),
        ALL,
        ALL,
    ),
    bad(
        "B07",
        "Tiểu Hao",
        1,
        Rule::MonthChi([
            ZTI, ZNGO, ZMUI, ZTHAN, ZDAU, ZTUAT, ZHOI, ZTY, ZSUU, ZDAN, ZMAO, ZTHIN,
        ]),
        s(&[CAU_TAI, GIAO_DICH, KHAI_TRUONG]),
        NONE,
    ),
    bad(
        "B08",
        "Nguyệt Phá",
        1,
        Rule::MonthChi([
            ZTHAN, ZTUAT, ZTUAT, ZHOI, ZSUU, ZSUU, ZDAN, ZTHIN, ZTHIN, ZTI, ZMUI, ZMUI,
        ]),
        s(&[XAY_DUNG, KHAI_TRUONG, CAU_TAI]),
        s(A_TAI),
    ),
    bad(
        "B09",
        "Kiếp Sát",
        3,
        Rule::MonthChi(x3([ZHOI, ZTHAN, ZTI, ZDAN])),
        s(&[XUAT_HANH, CUOI_HOI, AN_TANG, XAY_DUNG]),
        s(&[XUAT_HANH, CUOI_HOI, AN_TANG, XAY_DUNG]),
    ),
    bad(
        "B10",
        "Địa Phá",
        1,
        Rule::MonthChi([
            ZHOI, ZTY, ZSUU, ZDAN, ZMAO, ZTHIN, ZTI, ZNGO, ZMUI, ZTHAN, ZDAU, ZTUAT,
        ]),
        s(&[XAY_DUNG, DONG_THO]),
        s(A_DT),
    ),
    bad(
        "B11",
        "Thổ Phủ",
        1,
        Rule::MonthChi([
            ZDAN, ZMAO, ZTHIN, ZTI, ZNGO, ZMUI, ZTHAN, ZDAU, ZTUAT, ZHOI, ZTY, ZSUU,
        ]),
        s(&[XAY_DUNG, DONG_THO]),
        s(A_DT),
    ),
    bad(
        "B12",
        "Thổ Ôn",
        1,
        Rule::MonthChi([
            ZTHIN, ZTI, ZNGO, ZMUI, ZTHAN, ZDAU, ZTUAT, ZHOI, ZTY, ZSUU, ZDAN, ZMAO,
        ]),
        s(&[XAY_DUNG, DONG_THO, TE_TU]),
        s(A_DT),
    ),
    bad(
        "B13",
        "Thiên Ôn",
        1,
        Rule::MonthChi([
            ZMUI, ZTUAT, ZTHIN, ZDAN, ZNGO, ZTY, ZDAU, ZTHAN, ZTI, ZHOI, ZTY, ZMAO,
        ]),
        s(&[XAY_DUNG, KHAI_TRUONG, CAU_TAI]),
        s(&[XAY_DUNG, KHAI_TRUONG, CAU_TAI]),
    ),
    bad(
        "B14",
        "Thụ Tử",
        3,
        Rule::MonthChi([
            ZTUAT, ZTHIN, ZHOI, ZTI, ZTY, ZNGO, ZSUU, ZMUI, ZDAN, ZTHAN, ZMAO, ZDAU,
        ]),
        ALL,
        ALL,
    ),
    bad(
        "B15",
        "Hoang Vu",
        1,
        Rule::SeasonChi([
            &[ZTI, ZDAU, ZSUU],
            &[ZTHAN, ZTY, ZTHIN],
            &[ZHOI, ZMAO, ZMUI],
            &[ZDAN, ZNGO, ZTUAT],
        ]),
        ALL,
        NONE,
    ),
    bad(
        "B16",
        "Thiên Tặc",
        1,
        Rule::MonthChi([
            ZTHIN, ZDAU, ZDAN, ZMUI, ZTY, ZTI, ZTUAT, ZMAO, ZTHAN, ZSUU, ZNGO, ZHOI,
        ]),
        s(&[XAY_DUNG, DONG_THO, NHAP_TRACH, KHAI_TRUONG, CAU_TAI]),
        s(&[XAY_DUNG, KHAI_TRUONG, CAU_TAI]),
    ),
    bad(
        "B17",
        "Địa Tặc",
        1,
        Rule::MonthChi([
            ZSUU, ZTY, ZHOI, ZTUAT, ZDAU, ZTHAN, ZMUI, ZNGO, ZTI, ZTHIN, ZMAO, ZDAN,
        ]),
        s(&[XAY_DUNG, AN_TANG, DONG_THO, XUAT_HANH]),
        s(A_DT),
    ),
    bad(
        "B18",
        "Hỏa Tai",
        1,
        Rule::MonthChi([
            ZSUU, ZMUI, ZDAN, ZTHAN, ZMAO, ZDAU, ZTHIN, ZTUAT, ZTI, ZHOI, ZNGO, ZTY,
        ]),
        s(A_XAY),
        NONE,
    ),
    bad(
        "B19",
        "Nguyệt Hỏa (Độc Hỏa)",
        1,
        Rule::MonthChi([
            ZTI, ZTHIN, ZMAO, ZDAN, ZSUU, ZTY, ZHOI, ZTUAT, ZDAU, ZTHAN, ZMUI, ZNGO,
        ]),
        s(A_XAY),
        NONE,
    ),
    bad(
        "B20",
        "Nguyệt Yếm Đại Họa",
        1,
        Rule::MonthChi([
            ZTUAT, ZDAU, ZTHAN, ZMUI, ZNGO, ZTI, ZTHIN, ZMAO, ZDAN, ZSUU, ZTY, ZHOI,
        ]),
        s(&[XUAT_HANH, CUOI_HOI]),
        NONE,
    ),
    bad(
        "B21",
        "Nguyệt Hư",
        1,
        Rule::MonthChi(x3([ZSUU, ZTUAT, ZMUI, ZTHIN])),
        s(&[CUOI_HOI, KHAI_TRUONG]),
        s(A_CUOI),
    ),
    bad(
        "B22",
        "Hoàng Sa",
        1,
        Rule::MonthChi(x4([ZNGO, ZDAN, ZTY])),
        s(A_XH),
        NONE,
    ),
    bad(
        "B23",
        "Lục Bất Thành",
        1,
        Rule::MonthChi([
            ZDAN, ZNGO, ZTUAT, ZTI, ZDAU, ZSUU, ZTHAN, ZTY, ZTHIN, ZHOI, ZMAO, ZMUI,
        ]),
        s(&[XAY_DUNG, KHAI_TRUONG, CAU_TAI]),
        s(A_TAI),
    ),
    bad(
        "B24",
        "Nhân Cách",
        1,
        Rule::MonthChi(x2([ZDAU, ZMUI, ZTI, ZMAO, ZSUU, ZHOI])),
        s(&[CUOI_HOI, XAY_DUNG]),
        s(A_CUOI),
    ),
    bad(
        "B25",
        "Thần Cách",
        1,
        Rule::MonthChi(x2([ZTI, ZMAO, ZSUU, ZHOI, ZDAU, ZMUI])),
        s(A_TE),
        s(A_TE),
    ),
    bad(
        "B26",
        "Phi Ma Sát",
        1,
        Rule::MonthChi(x3([ZTY, ZDAU, ZNGO, ZMAO])),
        s(&[CUOI_HOI, NHAP_TRACH]),
        s(&[NHAP_TRACH]),
    ),
    bad(
        "B27",
        "Ngũ Quỷ",
        1,
        Rule::MonthChi([
            ZNGO, ZDAN, ZTHIN, ZDAU, ZMAO, ZTHAN, ZSUU, ZTI, ZTY, ZHOI, ZMUI, ZTUAT,
        ]),
        s(A_XH),
        s(A_XH),
    ),
    bad(
        "B28",
        "Băng Tiêu Ngọa Hãm",
        1,
        Rule::MonthChi([
            ZTI, ZTY, ZSUU, ZDAN, ZMAO, ZTUAT, ZHOI, ZNGO, ZMUI, ZTHAN, ZDAU, ZTHIN,
        ]),
        ALL,
        NONE,
    ),
    bad(
        "B29",
        "Hà Khôi (Cẩu Giảo)",
        1,
        Rule::MonthChi([
            ZHOI, ZNGO, ZSUU, ZTHAN, ZMAO, ZTUAT, ZTI, ZTY, ZMUI, ZDAN, ZDAU, ZTHIN,
        ]),
        ALL,
        NONE,
    ),
    bad(
        "B30",
        "Vãng Vong (Thổ Kỵ)",
        1,
        Rule::MonthChi([
            ZDAN, ZTI, ZTHAN, ZHOI, ZMAO, ZNGO, ZDAU, ZTY, ZTHIN, ZMUI, ZTUAT, ZSUU,
        ]),
        s(&[XUAT_HANH, CUOI_HOI, CAU_TAI, DONG_THO]),
        s(A_DT),
    ),
    bad(
        "B31",
        "Cửu Không",
        1,
        Rule::MonthChi([
            ZTHIN, ZSUU, ZTUAT, ZMUI, ZMAO, ZTY, ZDAU, ZNGO, ZDAN, ZHOI, ZTHAN, ZTI,
        ]),
        s(&[XUAT_HANH, CAU_TAI, KHAI_TRUONG]),
        NONE,
    ),
    bad(
        "B32",
        "Trùng Tang",
        3,
        Rule::MonthCan([GIAP, AT, KY, BINH, DINH, KY, CANH, TAN, KY, NHAM, QUY, KY]),
        s(&[CUOI_HOI, AN_TANG, XAY_DUNG]),
        s(&[CUOI_HOI, AN_TANG, XAY_DUNG]),
    ),
    bad(
        "B33",
        "Trùng Phục",
        1,
        Rule::MonthCan([CANH, TAN, KY, NHAM, QUY, MAU, GIAP, AT, KY, BINH, QUY, KY]),
        s(&[CUOI_HOI, AN_TANG]),
        s(A_TANG),
    ),
    bad(
        "B34",
        "Lôi Công",
        1,
        Rule::MonthChi(x3([ZDAN, ZHOI, ZTI, ZTHAN])),
        s(A_XAY),
        s(A_XAY),
    ),
    bad(
        "B35",
        "Cô Thần",
        1,
        Rule::MonthChi([
            ZTUAT, ZHOI, ZTY, ZSUU, ZDAN, ZMAO, ZTHIN, ZTI, ZNGO, ZMUI, ZTHAN, ZDAU,
        ]),
        s(&[CUOI_HOI, NHAP_TRACH]),
        s(&[CUOI_HOI, NHAP_TRACH]),
    ),
    bad(
        "B36",
        "Quả Tú",
        1,
        Rule::MonthChi([
            ZTHIN, ZTI, ZNGO, ZMUI, ZTHAN, ZDAU, ZTUAT, ZHOI, ZTY, ZSUU, ZDAN, ZMAO,
        ]),
        s(&[CUOI_HOI, NHAP_TRACH]),
        s(&[CUOI_HOI, NHAP_TRACH]),
    ),
    bad(
        "B37",
        "Sát Chủ",
        3,
        Rule::MonthChi([
            ZTI, ZTY, ZMUI, ZMAO, ZTHAN, ZTUAT, ZSUU, ZHOI, ZNGO, ZDAU, ZDAN, ZTHIN,
        ]),
        ALL,
        ALL,
    ),
    bad(
        "B38",
        "Nguyệt Hình",
        1,
        Rule::MonthChi([
            ZTI, ZTY, ZTHIN, ZTHAN, ZNGO, ZSUU, ZDAN, ZDAU, ZMUI, ZHOI, ZMAO, ZTUAT,
        ]),
        ALL,
        NONE,
    ),
    bad(
        "B39",
        "Tội Chỉ",
        1,
        Rule::MonthChi([
            ZNGO, ZTY, ZMUI, ZSUU, ZTHAN, ZDAN, ZDAU, ZMAO, ZTUAT, ZTHIN, ZHOI, ZTI,
        ]),
        s(&[TE_TU, TO_TUNG]),
        NONE,
    ),
    bad(
        "B40",
        "Nguyệt Kiến Chuyển Sát",
        1,
        Rule::SeasonChi([&[ZMAO], &[ZNGO], &[ZDAU], &[ZTY]]),
        s(A_DT),
        s(A_DT),
    ),
    bad(
        "B41",
        "Thiên Địa Chính Chuyển",
        1,
        Rule::SeasonCanChi([(QUY, ZMAO), (BINH, ZNGO), (DINH, ZDAU), (CANH, ZTY)]),
        s(A_DT),
        s(A_DT),
    ),
    bad(
        "B42",
        "Thiên Địa Chuyển Sát",
        1,
        Rule::SeasonCanChi([(AT, ZMAO), (BINH, ZNGO), (TAN, ZDAU), (NHAM, ZTY)]),
        s(A_DT),
        s(A_DT),
    ),
    bad(
        "B43",
        "Lỗ Ban Sát",
        1,
        Rule::SeasonChi([&[ZTY], &[ZMAO], &[ZNGO], &[ZDAU]]),
        s(A_XAY),
        s(A_XAY),
    ),
    bad(
        "B44",
        "Phủ Đầu Sát",
        1,
        Rule::SeasonChi([&[ZTHIN], &[ZMUI], &[ZDAU], &[ZTY]]),
        s(A_XAY),
        NONE,
    ),
    bad(
        "B45",
        "Tam Tang",
        1,
        Rule::SeasonChi([&[ZTHIN], &[ZMUI], &[ZTUAT], &[ZSUU]]),
        s(&[XAY_DUNG, CUOI_HOI, AN_TANG]),
        s(A_TANG),
    ),
    bad(
        "B46",
        "Ngũ Hư",
        1,
        Rule::MonthChi([
            ZTI, ZDAU, ZSUU, ZTHAN, ZTY, ZTHIN, ZHOI, ZMAO, ZMUI, ZDAN, ZNGO, ZTUAT,
        ]),
        s(&[XAY_DUNG, CUOI_HOI, AN_TANG]),
        NONE,
    ),
    bad(
        "B47",
        "Tứ Thời Đại Mộ",
        1,
        Rule::SeasonCanChi([(AT, ZMUI), (BINH, ZTUAT), (TAN, ZSUU), (NHAM, ZTHIN)]),
        s(&[AN_TANG, XAY_DUNG]),
        s(&[AN_TANG, XAY_DUNG]),
    ),
    bad(
        "B48",
        "Thổ Cấm",
        1,
        Rule::SeasonChi([&[ZHOI], &[ZDAN], &[ZTI], &[ZTHAN]]),
        s(&[XAY_DUNG, AN_TANG, DONG_THO]),
        s(A_DT),
    ),
    bad(
        "B49",
        "Ly Sàng",
        1,
        Rule::SeasonChi([&[ZDAU], &[ZDAN, ZNGO], &[ZTUAT], &[ZTI]]),
        s(&[CUOI_HOI, NHAP_TRACH]),
        s(&[CUOI_HOI, NHAP_TRACH]),
    ),
    bad(
        "B50",
        "Tứ Thời Cô Quả",
        1,
        Rule::SeasonChi([&[ZSUU], &[ZTHIN], &[ZMUI], &[ZTUAT]]),
        s(&[CUOI_HOI, NHAP_TRACH]),
        s(&[CUOI_HOI, NHAP_TRACH]),
    ),
    bad(
        "B51",
        "Không Phòng",
        1,
        Rule::MonthChi([
            ZTHIN, ZTI, ZTY, ZTUAT, ZHOI, ZMUI, ZDAN, ZMAO, ZNGO, ZTHAN, ZDAU, ZSUU,
        ]),
        s(A_CUOI),
        s(A_CUOI),
    ),
    bad(
        "B52",
        "Âm Thác",
        1,
        Rule::MonthCanChi([
            (CANH, ZTUAT),
            (TAN, ZDAU),
            (CANH, ZTHAN),
            (DINH, ZMUI),
            (BINH, ZNGO),
            (DINH, ZTI),
            (GIAP, ZTHIN),
            (AT, ZMAO),
            (GIAP, ZDAN),
            (QUY, ZSUU),
            (NHAM, ZTY),
            (QUY, ZHOI),
        ]),
        s(&[XUAT_HANH, CUOI_HOI, AN_TANG]),
        NONE,
    ),
    bad(
        "B53",
        "Dương Thác",
        1,
        Rule::MonthCanChi([
            (GIAP, ZDAN),
            (AT, ZMAO),
            (GIAP, ZTHIN),
            (DINH, ZTI),
            (BINH, ZNGO),
            (DINH, ZMUI),
            (CANH, ZTHAN),
            (TAN, ZDAU),
            (CANH, ZTUAT),
            (QUY, ZHOI),
            (NHAM, ZTY),
            (QUY, ZSUU),
        ]),
        s(&[XUAT_HANH, CUOI_HOI, AN_TANG]),
        NONE,
    ),
    bad(
        "B54",
        "Quỷ Khốc",
        1,
        Rule::MonthChi([ZTUAT; 12]),
        s(&[TE_TU, AN_TANG, XAY_DUNG]),
        s(&[AN_TANG, XAY_DUNG]),
    ),
    bad(
        "B55",
        "Địa Hỏa",
        1,
        Rule::MonthChi([
            ZTUAT, ZDAU, ZTHAN, ZMUI, ZNGO, ZTI, ZTHIN, ZMAO, ZDAN, ZSUU, ZTY, ZHOI,
        ]),
        ALL,
        NONE,
    ),
    bad(
        "B56",
        "Thập Ác Đại Bại",
        3,
        Rule::ThapAc,
        s(&[CAU_TAI, GIAO_DICH, XAY_DUNG, KHAI_TRUONG]),
        s(&[CAU_TAI, GIAO_DICH, XAY_DUNG, KHAI_TRUONG]),
    ),
];

/// Ngày đặc biệt (ghi vào danh sách sao xấu).
pub(crate) const SPECIAL_DAYS: &[Star] = &[
    bad(
        "S01",
        "Tam Nương",
        2,
        Rule::LunarDays(&[3, 7, 13, 18, 22, 27]),
        s(&[CUOI_HOI, KHAI_TRUONG, GIAO_DICH, XAY_DUNG, XUAT_HANH]),
        s(&[CUOI_HOI, KHAI_TRUONG, GIAO_DICH]),
    ),
    bad(
        "S02",
        "Nguyệt Kỵ",
        2,
        Rule::LunarDays(&[5, 14, 23]),
        ALL,
        s(&[XUAT_HANH, GIAO_DICH, CAU_TAI, KHAI_TRUONG]),
    ),
    bad(
        "S03",
        "Dương Công Kỵ Nhật",
        3,
        Rule::MonthDays(&[
            (1, 13),
            (2, 11),
            (3, 9),
            (4, 7),
            (5, 5),
            (6, 3),
            (7, 1),
            (7, 29),
            (8, 27),
            (9, 25),
            (10, 23),
            (11, 21),
            (12, 19),
        ]),
        ALL,
        ALL,
    ),
    // Thọ Tử (bản can chi, ngày đặc biệt dân gian). Ngày này luôn đồng thời là Thụ Tử (B14).
    bad(
        "S04",
        "Thọ Tử",
        1,
        Rule::MonthCanChi([
            (BINH, ZTUAT),
            (NHAM, ZTHIN),
            (TAN, ZHOI),
            (DINH, ZTI),
            (MAU, ZTY),
            (BINH, ZNGO),
            (AT, ZSUU),
            (QUY, ZMUI),
            (GIAP, ZDAN),
            (MAU, ZTHAN),
            (TAN, ZMAO),
            (TAN, ZDAU),
        ]),
        ALL,
        ALL,
    ),
];

/// Thập ác đại bại: (nhóm can năm, tháng âm, can ngày, chi ngày).
/// Nhóm can năm: 0 = Giáp/Kỷ, 1 = Ất/Canh, 2 = Bính/Tân, 3 = Đinh/Nhâm, 4 = Mậu/Quý.
pub(crate) const THAP_AC: &[(u8, u8, u8, u8)] = &[
    (0, 3, MAU, ZTUAT),
    (0, 7, QUY, ZHOI),
    (0, 10, BINH, ZTHAN),
    (0, 11, DINH, ZHOI),
    (1, 4, NHAM, ZTHAN),
    (1, 9, AT, ZTI),
    (2, 3, TAN, ZTI),
    (2, 9, CANH, ZTHIN),
    (2, 10, GIAP, ZTHIN),
    (4, 6, KY, ZSUU),
];

/// 12 thần Hoàng đạo / Hắc đạo (T01..T12), cùng thứ tự với lõi (0 = Thanh Long).
/// Chỉ số 5: "Kim Đường" (lõi dùng cùng tên, `core::canchi::THAN[5]`), để khỏi trùng sao Thiên Đức (G01).
pub(crate) const THAN: [Star; 12] = [
    good("T01", "Thanh Long Hoàng Đạo", 2, Rule::Than(0), ALL),
    good("T02", "Minh Đường Hoàng Đạo", 2, Rule::Than(1), ALL),
    bad("T03", "Thiên Hình Hắc Đạo", 2, Rule::Than(2), ALL, NONE),
    bad(
        "T04",
        "Chu Tước Hắc Đạo",
        2,
        Rule::Than(3),
        esp(&[NHAP_TRACH, KHAI_TRUONG, CAU_TAI]),
        s(&[NHAP_TRACH, KHAI_TRUONG, CAU_TAI]),
    ),
    good("T05", "Kim Quỹ Hoàng Đạo", 2, Rule::Than(4), ALL),
    good("T06", "Kim Đường Hoàng Đạo", 2, Rule::Than(5), ALL),
    bad(
        "T07",
        "Bạch Hổ Hắc Đạo",
        2,
        Rule::Than(6),
        esp(A_TANG),
        s(A_TANG),
    ),
    good("T08", "Ngọc Đường Hoàng Đạo", 2, Rule::Than(7), ALL),
    bad("T09", "Thiên Lao Hắc Đạo", 2, Rule::Than(8), ALL, NONE),
    bad(
        "T10",
        "Huyền Vũ Hắc Đạo",
        2,
        Rule::Than(9),
        esp(A_TANG),
        NONE,
    ),
    good("T11", "Tư Mệnh Hoàng Đạo", 2, Rule::Than(10), ALL),
    bad(
        "T12",
        "Câu Trần Hắc Đạo",
        2,
        Rule::Than(11),
        esp(A_TANG),
        NONE,
    ),
];

/// Trực tốt / xấu (0 = Kiến … 11 = Bế): +1 tốt, 0 bình, −1 xấu.
pub(crate) const TRUC_SCORE: [i32; 12] = [1, 0, 1, 1, 1, 0, -1, -1, 1, 0, 1, -1];

/// Bảng tuổi xung khắc với ngày, chỉ mục = vị trí trong vòng 60 Giáp Tý (0 = Giáp Tý).
/// Nguồn: xem content/ngochap/rules.md, mục X01.
pub(crate) const XUNG_TUOI: [&[(u8, u8)]; 60] = [
    &[(MAU, ZNGO), (NHAM, ZNGO), (CANH, ZDAN), (CANH, ZTHAN)], //  1 Giáp Tý
    &[(KY, ZMUI), (QUY, ZMUI), (TAN, ZMAO), (TAN, ZDAU)],      //  2 Ất Sửu
    &[(GIAP, ZTHAN), (NHAM, ZTHAN), (NHAM, ZTUAT), (NHAM, ZTHIN)], //  3 Bính Dần
    &[(AT, ZDAU), (QUY, ZDAU), (QUY, ZTI), (QUY, ZHOI)],       //  4 Đinh Mão
    &[(CANH, ZTUAT), (BINH, ZTUAT)],                           //  5 Mậu Thìn
    &[(TAN, ZHOI), (DINH, ZHOI)],                              //  6 Kỷ Tỵ
    &[(NHAM, ZTY), (BINH, ZTY), (GIAP, ZTHAN), (GIAP, ZDAN)],  //  7 Canh Ngọ
    &[(QUY, ZSUU), (DINH, ZSUU), (AT, ZDAU), (AT, ZMAO)],      //  8 Tân Mùi
    &[(BINH, ZDAN), (CANH, ZDAN), (BINH, ZTHAN)],              //  9 Nhâm Thân
    &[(DINH, ZMAO), (TAN, ZMAO), (DINH, ZDAU)],                // 10 Quý Dậu
    &[(NHAM, ZTHIN), (CANH, ZTHIN), (CANH, ZTUAT)],            // 11 Giáp Tuất
    &[(QUY, ZTI), (TAN, ZTI), (TAN, ZHOI)],                    // 12 Ất Hợi
    &[(CANH, ZNGO), (MAU, ZNGO)],                              // 13 Bính Tý
    &[(TAN, ZMUI), (KY, ZMUI)],                                // 14 Đinh Sửu
    &[(CANH, ZTHAN), (GIAP, ZTHAN)],                           // 15 Mậu Dần
    &[(TAN, ZDAU), (AT, ZDAU)],                                // 16 Kỷ Mão
    &[(GIAP, ZTUAT), (MAU, ZTUAT), (GIAP, ZTHIN)],             // 17 Canh Thìn
    &[(AT, ZHOI), (KY, ZHOI), (AT, ZTI)],                      // 18 Tân Tỵ
    &[(GIAP, ZTY), (CANH, ZTY), (BINH, ZTUAT), (BINH, ZTHIN)], // 19 Nhâm Ngọ
    &[(AT, ZSUU), (TAN, ZSUU), (DINH, ZHOI), (DINH, ZTI)],     // 20 Quý Mùi
    &[(MAU, ZDAN), (BINH, ZDAN), (CANH, ZNGO), (CANH, ZTY)],   // 21 Giáp Thân
    &[(KY, ZMAO), (DINH, ZMAO), (TAN, ZMUI), (TAN, ZSUU)],     // 22 Ất Dậu
    &[(MAU, ZTHIN), (NHAM, ZTHIN), (NHAM, ZNGO), (NHAM, ZTY)], // 23 Bính Tuất
    &[(KY, ZTI), (QUY, ZTI), (QUY, ZMUI), (QUY, ZSUU)],        // 24 Đinh Hợi
    &[(BINH, ZNGO), (GIAP, ZNGO)],                             // 25 Mậu Tý
    &[(DINH, ZMUI), (AT, ZMUI)],                               // 26 Kỷ Sửu
    &[(NHAM, ZTHAN), (MAU, ZTHAN), (GIAP, ZTY), (GIAP, ZNGO)], // 27 Canh Dần
    &[(QUY, ZDAU), (KY, ZDAU), (AT, ZSUU), (AT, ZMUI)],        // 28 Tân Mão
    &[(BINH, ZTUAT), (GIAP, ZTUAT), (BINH, ZDAN)],             // 29 Nhâm Thìn
    &[(DINH, ZHOI), (AT, ZHOI), (DINH, ZMAO)],                 // 30 Quý Tỵ
    &[(MAU, ZTY), (NHAM, ZTY), (CANH, ZDAN), (NHAM, ZDAN)],    // 31 Giáp Ngọ
    &[(KY, ZSUU), (QUY, ZSUU), (TAN, ZMAO), (TAN, ZDAU)],      // 32 Ất Mùi
    &[(GIAP, ZDAN), (NHAM, ZTHAN), (NHAM, ZTUAT), (NHAM, ZTHIN)], // 33 Bính Thân
    &[(AT, ZMAO), (QUY, ZMAO), (QUY, ZTI), (QUY, ZHOI)],       // 34 Đinh Dậu
    &[(CANH, ZTHIN), (BINH, ZTHIN)],                           // 35 Mậu Tuất
    &[(TAN, ZTI), (DINH, ZTI)],                                // 36 Kỷ Hợi
    &[(NHAM, ZNGO), (BINH, ZNGO), (GIAP, ZTHAN), (GIAP, ZDAN)], // 37 Canh Tý
    &[(QUY, ZMUI), (DINH, ZMUI), (AT, ZDAU), (AT, ZMAO)],      // 38 Tân Sửu
    &[(CANH, ZTHAN), (BINH, ZTHAN), (BINH, ZDAN)],             // 39 Nhâm Dần
    &[(TAN, ZDAU), (DINH, ZDAU), (DINH, ZMAO)],                // 40 Quý Mão
    &[(NHAM, ZTUAT), (CANH, ZTUAT), (CANH, ZTHIN)],            // 41 Giáp Thìn
    &[(QUY, ZHOI), (TAN, ZHOI), (TAN, ZTI)],                   // 42 Ất Tỵ
    &[(MAU, ZTY), (CANH, ZTY)],                                // 43 Bính Ngọ
    &[(KY, ZSUU), (TAN, ZSUU)],                                // 44 Đinh Mùi
    &[(CANH, ZDAN), (GIAP, ZDAN)],                             // 45 Mậu Thân
    &[(TAN, ZMAO), (AT, ZMAO)],                                // 46 Kỷ Dậu
    &[(GIAP, ZTHIN), (MAU, ZTHIN), (GIAP, ZTUAT)],             // 47 Canh Tuất
    &[(AT, ZTI), (KY, ZTI), (AT, ZHOI)],                       // 48 Tân Hợi
    &[(GIAP, ZNGO), (CANH, ZNGO), (BINH, ZTUAT), (BINH, ZTHIN)], // 49 Nhâm Tý
    &[(AT, ZMUI), (TAN, ZMUI), (DINH, ZHOI), (DINH, ZTI)],     // 50 Quý Sửu
    &[(MAU, ZTHAN), (BINH, ZTHAN), (CANH, ZNGO), (CANH, ZTY)], // 51 Giáp Dần
    &[(KY, ZDAU), (DINH, ZDAU), (TAN, ZMUI), (TAN, ZSUU)],     // 52 Ất Mão
    &[(MAU, ZTUAT), (NHAM, ZTUAT), (NHAM, ZNGO), (NHAM, ZTY)], // 53 Bính Thìn
    &[(KY, ZHOI), (QUY, ZHOI), (QUY, ZSUU), (QUY, ZMUI)],      // 54 Đinh Tỵ
    &[(BINH, ZTY), (GIAP, ZTY)],                               // 55 Mậu Ngọ
    &[(DINH, ZSUU), (AT, ZSUU)],                               // 56 Kỷ Mùi
    &[(NHAM, ZDAN), (MAU, ZDAN), (GIAP, ZTY), (GIAP, ZNGO)],   // 57 Canh Thân
    &[(QUY, ZMAO), (KY, ZMAO), (AT, ZSUU), (AT, ZMUI)],        // 58 Tân Dậu
    &[(BINH, ZTHIN), (GIAP, ZTHIN), (BINH, ZTHAN), (BINH, ZDAN)], // 59 Nhâm Tuất
    &[(DINH, ZTI), (AT, ZTI), (DINH, ZMAO), (DINH, ZDAU)],     // 60 Quý Hợi
];
