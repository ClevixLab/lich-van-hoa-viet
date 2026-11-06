//! Sinh vector kiểm thử dùng chung Rust ↔ JavaScript (docs/v0.2/03 §2.5) vào
//! `docs/v0.2/schema/test-vectors/`. Kết quả tất định (salt/nonce cố định, UUID sinh từ SHA-256 của bộ đếm):
//! chạy lại cho đúng từng byte; `store/tests/vectors.rs` kiểm điều đó.
//!
//!     cargo run --release -p lvhv-store --example gen_test_vectors
//!
//! Tệp sinh ra:
//! - `example-v1.json`        — example-backup-v1.json với `photos[0].bytes`/`sha256` của ảnh mẫu (tệp mẫu
//!   trong lược đồ để 0…0), JSON chuẩn.
//! - `example-v1.lvhv`        — tệp sao lưu của example-v1.json + ảnh mẫu, mật khẩu "mật khẩu thử nghiệm".
//! - `corrupt/*.lvhv`         — các tệp hỏng của 03 §2.5 (manifest ghi mã lỗi phải trả).
//! - `dataset-r6.json`        — bộ dữ liệu khứ hồi R6(a) (60 người, 30 sự kiện, 20 ảnh tất định).
//! - `validation-cases.json`  — các ca kiểm tra V1–V8 (hai bản phải trả cùng mã và đường dẫn).
//! - `merge-cases.json`       — 31 ca gộp R6(e) với kết quả mong đợi ghi tường minh.
//! - `manifest.json`          — tham số, sha256 từng tệp, mã lỗi mong đợi.
use lvhv_store::{build_person, canonical_json, sha256_hex, validate};
use serde_json::{json, Value};
use std::path::PathBuf;

pub const PASSWORD: &str = "mật khẩu thử nghiệm";
pub const SALT: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
pub const NONCE: [u8; 24] = [
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
];

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/v0.2/schema/test-vectors")
}

/// UUID v4 tất định từ một nhãn.
fn uid(label: &str) -> String {
    let h = sha256_hex(label.as_bytes());
    let mut b: Vec<u8> = (0..16)
        .map(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16).unwrap())
        .collect();
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    lvhv_backup::uuid_to_string(&b.try_into().unwrap())
}

/// Byte "ảnh" tất định của bộ R6 (lớp lưu trữ coi ảnh là byte mờ; giải mã/mã hóa lại ảnh là việc của giao
/// diện — 01 §3.8). Công thức đơn giản để bản JavaScript sinh lại đúng: FF D8 FF E0 ‖ thân ‖ FF D9.
pub fn r6_photo_bytes(i: usize) -> Vec<u8> {
    let len = 1000 + i * 997;
    let mut v: Vec<u8> = (0..len)
        .map(|k| ((k * 31 + i * 17 + (k >> 8)) & 0xff) as u8)
        .collect();
    v[..4].copy_from_slice(&[0xff, 0xd8, 0xff, 0xe0]);
    let n = v.len();
    v[n - 2..].copy_from_slice(&[0xff, 0xd9]);
    v
}

fn rule(label: &str, offsets: &[u32], tt: bool) -> Value {
    json!({"id": uid(label), "enabled": true, "offsets_days": offsets, "time_local": "07:00", "tien_thuong": tt})
}

fn dv(cal: &str, pr: &str, y: Option<i32>, m: Option<u32>, d: Option<u32>, leap: bool) -> Value {
    json!({"calendar": cal, "precision": pr, "year": y, "month": m, "day": d, "leap": leap})
}

const TS: &str = "2026-11-06T03:15:00Z";

fn deceased(
    i: usize,
    death: Value,
    birth: Option<Value>,
    photo: Option<String>,
    style: &str,
) -> Value {
    let base = json!({
        "id": uid(&format!("r6-person-{i}")), "kind": "deceased", "danh_xung": (["Ông", "Bà", "Cụ", "Bác"][i % 4]),
        "ho_ten": format!("Nguyễn Văn Thử {i}"), "ten_hieu": if i % 5 == 0 { "Pháp danh Diệu Âm" } else { "" },
        "quan_he": "họ hàng", "birth": birth.map(|b| json!({"input": b, "noi_sinh": "Hà Nội"})),
        "death": {"input": death, "tuoi": {"manual": false}}, "noi_an_tang": "Nghĩa trang Văn Điển",
        "photo_id": photo,
        "memorial": {"tray_style": style, "incense": style != "cong_giao", "bia_tieu_de": "Hiển khảo",
                     "loi_tuong_niem": "Dòng 1\nDòng 2 — nhớ ông", "gio_het_nam": 2 + (i % 2), "huong_duong_duoi": 60},
        "reminders": {"gio": rule(&format!("r6-gio-{i}"), &[0, 3], i % 3 == 0), "sinh_nhat": null, "sinh_nhat_am": null},
        "sinh_nhat_am": false, "feb29_rule": "feb28", "ghi_chu": format!("Ghi chú {i}\nthêm dòng"),
        "created_at": TS, "updated_at": TS, "rev": 1 + (i % 4),
    });
    build_person(&base).unwrap()
}

fn living(i: usize, birth: Value, photo: Option<String>) -> Option<Value> {
    let base = json!({
        "id": uid(&format!("r6-person-{i}")), "kind": "living", "danh_xung": "Chị", "ho_ten": format!("Trần Thị Sống {i}"),
        "ten_hieu": "", "quan_he": "chị", "birth": {"input": birth, "noi_sinh": ""}, "death": null, "noi_an_tang": "",
        "photo_id": photo, "memorial": null,
        "reminders": {"gio": null, "sinh_nhat": rule(&format!("r6-sn-{i}"), &[0, 1, 15], false),
                      "sinh_nhat_am": if i % 2 == 0 { rule(&format!("r6-snam-{i}"), &[0], false) } else { Value::Null }},
        "sinh_nhat_am": i % 2 == 0, "feb29_rule": if i % 3 == 0 { "mar1" } else { "feb28" }, "ghi_chu": "",
        "created_at": TS, "updated_at": TS, "rev": 1,
    });
    build_person(&base).ok()
}

/// Bộ dữ liệu R6(a): mọi kiểu nhập ngày (dương, âm, âm không năm, tháng nhuận, 30 âm, 29/02), Unicode,
/// nhiều dòng, 20 ảnh, loại sự kiện tự tạo.
pub fn dataset_r6() -> (Value, Vec<(String, Vec<u8>)>) {
    let mut photos = Vec::new();
    let mut photo_meta = Vec::new();
    for i in 0..20 {
        let id = uid(&format!("r6-photo-{i}"));
        let b = r6_photo_bytes(i);
        photo_meta.push(json!({"id": id, "mime": if i % 2 == 0 { "image/jpeg" } else { "image/webp" },
            "width": 300 + i, "height": 400 + i, "bytes": b.len(), "sha256": sha256_hex(&b), "stripped": true, "created_at": TS}));
        photos.push((id, b));
    }
    let deaths = [
        dv("solar", "day", Some(2025), Some(8), Some(8), false), // → 15/6 nhuận 2025
        dv("solar", "day", Some(1900), Some(1), Some(31), false), // đầu miền
        dv("solar", "day", Some(2000), Some(2), Some(29), false), // 29/02
        dv("solar", "day", Some(3000), Some(12), Some(31), false), // cuối miền
        dv("lunar", "day", Some(2023), Some(2), Some(29), true), // tháng 2 nhuận 2023 (thiếu)
        dv("lunar", "day", Some(2020), Some(4), Some(1), true),  // tháng 4 nhuận 2020
        dv("lunar", "month_day", None, Some(3), Some(10), false),
        dv("lunar", "month_day", None, Some(12), Some(30), false),
        dv("lunar", "day", Some(1500), Some(5), Some(5), false), // ngoài miền đổi lịch: solar = null
        dv("lunar", "day", Some(1968), Some(1), Some(1), false), // Tết Mậu Thân
    ];
    let births = [
        Some(dv("solar", "day", Some(1940), Some(5), Some(1), false)),
        None,
        Some(dv("solar", "year", Some(1930), None, None, false)),
        Some(dv("lunar", "year", Some(2900), None, None, false)),
        Some(dv("lunar", "day", Some(1950), Some(7), Some(15), false)),
        Some(dv("solar", "day", Some(1960), Some(2), Some(29), false)),
        None,
        Some(dv("solar", "day", Some(1800), Some(3), Some(3), false)), // ngoài miền
        None,
        Some(dv("solar", "day", Some(1910), Some(1), Some(20), false)),
    ];
    let styles = ["truyen_thong", "phat_giao", "cong_giao"];
    let mut persons = Vec::new();
    for i in 0..30 {
        let photo = (i < 12).then(|| photo_meta[i]["id"].as_str().unwrap().to_owned());
        persons.push(deceased(
            i,
            deaths[i % 10].clone(),
            births[i % 10].clone(),
            photo,
            styles[i % 3],
        ));
    }
    let lb = [
        dv("solar", "day", Some(1990), Some(1), Some(20), false),
        dv("solar", "day", Some(2000), Some(2), Some(29), false),
        dv("lunar", "day", Some(1995), Some(8), Some(30), true), // 30 tháng 8 nhuận 1995 (đủ)
        dv("solar", "year", Some(1985), None, None, false),
        dv("lunar", "day", Some(2033), Some(11), Some(15), true), // tháng 11 nhuận 2033
        dv("lunar", "year", Some(1979), None, None, false),
    ];
    for i in 30..60 {
        let photo = (i < 38).then(|| photo_meta[i - 18]["id"].as_str().unwrap().to_owned());
        let b = lb[i % 6].clone();
        // Ngày âm phải tồn tại; nếu không, generator dừng (bộ dữ liệu phải đúng như mô tả).
        persons.push(
            living(i, b, photo)
                .unwrap_or_else(|| panic!("ngày sinh âm của người {i} không tồn tại")),
        );
    }
    persons.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    let mut types: Vec<Value> = lvhv_store::default_event_types();
    types.push(json!({"id": uid("r6-type-0"), "builtin": null, "ten": "Lễ mừng thọ", "default_recurring": true,
                      "created_at": TS, "updated_at": TS, "rev": 2}));
    types.push(json!({"id": uid("r6-type-1"), "builtin": null, "ten": "Khai trương cửa hàng", "default_recurring": false,
                      "created_at": TS, "updated_at": TS, "rev": 1}));
    types.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    let mut events = Vec::new();
    for i in 0..30 {
        let date = if i % 3 == 0 {
            dv(
                "lunar",
                "month_day",
                None,
                Some(1 + (i % 12) as u32),
                Some(1 + (i % 30) as u32),
                false,
            )
        } else {
            dv(
                "solar",
                "day",
                Some(2000 + i as i32),
                Some(1 + (i % 12) as u32),
                Some(1 + (i % 28) as u32),
                false,
            )
        };
        let pids: Vec<String> = (0..(i % 4))
            .map(|k| {
                persons[(i * 7 + k * 13) % 60]["id"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect();
        let mut pids = pids;
        pids.dedup();
        events.push(json!({"id": uid(&format!("r6-event-{i}")), "type_id": types[i % types.len()]["id"], "tieu_de": format!("Sự kiện {i}"),
            "date": date, "recurring": i % 3 == 0 || i % 2 == 0, "person_ids": pids, "ghi_chu": "",
            "reminder": if i % 5 == 0 { Value::Null } else { rule(&format!("r6-ev-rule-{i}"), &[1, 3, 5, 10, 15, 60], false) },
            "created_at": TS, "updated_at": TS, "rev": 1}));
    }
    events.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    photo_meta.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    let doc = json!({
        "format": "lvhv-backup", "schema_version": 1,
        "app": {"name": "Lịch và Văn hóa Việt", "version": "0.2.0", "platform": "windows"},
        "exported_at": TS,
        "settings": {"reminders_enabled": true, "default_offsets_days": [0, 3, 10], "default_time_local": "06:30",
                     "upcoming_days": 30, "last_backup_at": "2026-10-01T00:00:00Z", "backup_nag_days": 60},
        "event_types": types, "persons": persons, "events": events, "photos": photo_meta,
    });
    validate::document(&doc).unwrap();
    (doc, photos)
}

/// Tài liệu mẫu của lược đồ, với ảnh mẫu thật.
pub fn example_doc(photo: &[u8]) -> Value {
    let text = std::fs::read_to_string(dir().join("../example-backup-v1.json")).unwrap();
    let mut doc = validate::parse_json(&text).unwrap();
    doc["photos"][0]["bytes"] = json!(photo.len());
    doc["photos"][0]["sha256"] = json!(sha256_hex(photo));
    validate::document(&doc).unwrap();
    doc
}

pub const EXAMPLE_PHOTO: &str = "photo-0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e09.jpg";

/// (tên, vị trí byte, giá trị XOR hoặc ghi đè, mã lỗi mong đợi). `None` ở vị trí = cắt cụt.
pub fn corrupt_specs() -> Vec<(&'static str, Value, &'static str)> {
    vec![
        ("magic", json!({"xor": [0, 1]}), "not_a_backup"),
        ("file_version-0", json!({"xor": [8, 1]}), "unsupported"),
        (
            "file_version-2",
            json!({"set_u16": [8, 2]}),
            "newer_version",
        ),
        ("kdf_id", json!({"xor": [10, 1]}), "unsupported"),
        ("aead_id", json!({"xor": [11, 1]}), "unsupported"),
        (
            "m_cost_kib",
            json!({"xor": [12, 1]}),
            "wrong_password_or_corrupt",
        ),
        (
            "m_cost_kib-4GiB",
            json!({"set_u32": [12, 4_194_304]}),
            "bad_kdf_params",
        ),
        (
            "m_cost_kib-low",
            json!({"set_u32": [12, 65_535]}),
            "bad_kdf_params",
        ),
        (
            "t_cost",
            json!({"xor": [16, 1]}),
            "wrong_password_or_corrupt",
        ),
        ("t_cost-0", json!({"set_u32": [16, 0]}), "bad_kdf_params"),
        ("t_cost-11", json!({"set_u32": [16, 11]}), "bad_kdf_params"),
        (
            "p_cost",
            json!({"xor": [20, 1]}),
            "wrong_password_or_corrupt",
        ),
        ("p_cost-9", json!({"set_u32": [20, 9]}), "bad_kdf_params"),
        ("salt", json!({"xor": [24, 1]}), "wrong_password_or_corrupt"),
        (
            "nonce",
            json!({"xor": [40, 1]}),
            "wrong_password_or_corrupt",
        ),
        ("plaintext_len", json!({"xor": [64, 1]}), "bad_length"),
        (
            "ciphertext",
            json!({"xor": [72, 1]}),
            "wrong_password_or_corrupt",
        ),
        (
            "tag",
            json!({"xor": [-1, 128]}),
            "wrong_password_or_corrupt",
        ),
        ("truncated-1", json!({"truncate": 1}), "bad_length"),
        ("truncated-to-87", json!({"keep": 87}), "not_a_backup"),
        ("appended-1", json!({"append": [0]}), "bad_length"),
    ]
}

pub fn apply_corruption(file: &[u8], spec: &Value) -> Vec<u8> {
    let mut f = file.to_vec();
    if let Some(a) = spec.get("xor") {
        let i = a[0].as_i64().unwrap();
        let i = if i < 0 {
            (f.len() as i64 + i) as usize
        } else {
            i as usize
        };
        f[i] ^= a[1].as_u64().unwrap() as u8;
    } else if let Some(a) = spec.get("set_u16") {
        let i = a[0].as_u64().unwrap() as usize;
        f[i..i + 2].copy_from_slice(&(a[1].as_u64().unwrap() as u16).to_le_bytes());
    } else if let Some(a) = spec.get("set_u32") {
        let i = a[0].as_u64().unwrap() as usize;
        f[i..i + 4].copy_from_slice(&(a[1].as_u64().unwrap() as u32).to_le_bytes());
    } else if let Some(n) = spec.get("truncate") {
        f.truncate(f.len() - n.as_u64().unwrap() as usize);
    } else if let Some(n) = spec.get("keep") {
        f.truncate(n.as_u64().unwrap() as usize);
    } else if let Some(a) = spec.get("append") {
        f.extend(
            a.as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_u64().unwrap() as u8),
        );
    }
    f
}

/// Các ca kiểm tra dữ liệu: (tên, đường dẫn JSON Pointer cần đổi, giá trị mới hoặc "__delete__", mã, đường dẫn lỗi).
pub fn validation_cases(base: &Value) -> Vec<Value> {
    let p0 = "/persons/0"; // Nguyễn Văn A (deceased, solar death 2025-08-08)
    let p2 = "/persons/2"; // Lê Thị C (living)
    let raw: Vec<(&str, String, Value)> = vec![
        ("ok", String::new(), Value::Null),
        ("unknown root key", "/extra".into(), json!(1)),
        ("missing key", format!("{p0}/ghi_chu"), json!("__delete__")),
        (
            "unknown nested key",
            format!("{p0}/memorial/x"),
            json!(true),
        ),
        (
            "bad uuid upper",
            format!("{p0}/id"),
            json!("0B8E4F52-3C1A-4D2E-9F60-1A2B3C4D5E01"),
        ),
        (
            "bad uuid v1",
            format!("{p0}/id"),
            json!("0b8e4f52-3c1a-1d2e-9f60-1a2b3c4d5e01"),
        ),
        ("ho_ten empty", format!("{p0}/ho_ten"), json!("")),
        ("ho_ten spaces", format!("{p0}/ho_ten"), json!(" \u{3000} ")),
        (
            "ho_ten 121 cp",
            format!("{p0}/ho_ten"),
            json!("ệ".repeat(121)),
        ),
        (
            "ho_ten 120 cp ok",
            format!("{p0}/ho_ten"),
            json!("ệ".repeat(120)),
        ),
        ("control char", format!("{p0}/quan_he"), json!("a\u{7}b")),
        ("newline in text", format!("{p0}/quan_he"), json!("a\nb")),
        (
            "newline in multiline ok",
            format!("{p0}/ghi_chu"),
            json!("a\nb"),
        ),
        (
            "C1 control in multiline",
            format!("{p0}/ghi_chu"),
            json!("a\u{85}b"),
        ),
        (
            "timestamp no Z",
            format!("{p0}/created_at"),
            json!("2026-11-06T03:15:00+07:00"),
        ),
        ("rev 0", format!("{p0}/rev"), json!(0)),
        ("rev float 2.0 ok", format!("{p0}/rev"), json!(2.0)),
        ("rev 2.5", format!("{p0}/rev"), json!(2.5)),
        ("rev string", format!("{p0}/rev"), json!("2")),
        (
            "rev > 2^53",
            format!("{p0}/rev"),
            json!(9_007_199_254_740_993_i64),
        ),
        (
            "V2 deceased without death",
            format!("{p0}/death"),
            Value::Null,
        ),
        (
            "V2 living with memorial",
            format!("{p2}/memorial"),
            base["persons"][0]["memorial"].clone(),
        ),
        (
            "V3 missing photo",
            format!("{p0}/photo_id"),
            json!("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5eff"),
        ),
        (
            "V3 missing type",
            "/events/0/type_id".into(),
            json!("6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0009"),
        ),
        (
            "V3 missing person",
            "/events/0/person_ids/0".into(),
            json!("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5eff"),
        ),
        (
            "V4 Feb 30",
            format!("{p0}/death/input"),
            json!({"calendar": "solar", "precision": "day", "year": 2025, "month": 2, "day": 30, "leap": false}),
        ),
        (
            "V4 Apr 31",
            format!("{p2}/birth/input"),
            json!({"calendar": "solar", "precision": "day", "year": 1990, "month": 4, "day": 31, "leap": false}),
        ),
        (
            "V4 lunar 30 of short month",
            format!("{p0}/death/input"),
            json!({"calendar": "lunar", "precision": "day", "year": 2025, "month": 6, "day": 30, "leap": true}),
        ),
        (
            "V4 lunar leap month missing",
            "/persons/1/death/input".into(),
            json!({"calendar": "lunar", "precision": "day", "year": 2026, "month": 6, "day": 1, "leap": true}),
        ),
        ("V5 lunar wrong", format!("{p0}/death/lunar/day"), json!(16)),
        (
            "V5 lunar_source wrong",
            format!("{p0}/death/lunar_source"),
            json!("entered"),
        ),
        ("V5 tuoi wrong", format!("{p0}/death/tuoi/value"), json!(85)),
        (
            "V5 tuoi manual ok",
            format!("{p0}/death/tuoi"),
            json!({"value": 85, "label": "huong_tho", "manual": true, "approx": false}),
        ),
        (
            "V5 can_chi wrong",
            format!("{p2}/birth/can_chi"),
            json!("Canh Ngọ"),
        ),
        (
            "V5 hnd method keeps stored lunar",
            format!("{p0}/death/method"),
            json!("hnd"),
        ),
        (
            "V6 cong_giao incense",
            format!("{p0}/memorial/tray_style"),
            json!("cong_giao"),
        ),
        (
            "V7 duplicate type name",
            "/event_types/1/ten".into(),
            json!("TỐT NGHIỆP"),
        ),
        (
            "V7 NFD duplicate",
            "/event_types/1/ten".into(),
            json!("To\u{302}\u{301}t nghie\u{323}\u{302}p"),
        ),
        (
            "V8 offsets unsorted",
            format!("{p0}/reminders/gio/offsets_days"),
            json!([3, 0]),
        ),
        (
            "offsets duplicate",
            format!("{p0}/reminders/gio/offsets_days"),
            json!([3, 3]),
        ),
        (
            "offsets 61",
            format!("{p0}/reminders/gio/offsets_days"),
            json!([61]),
        ),
        (
            "offsets 9 items",
            format!("{p0}/reminders/gio/offsets_days"),
            json!([0, 1, 2, 3, 4, 5, 6, 7, 8]),
        ),
        (
            "time 24:00",
            format!("{p0}/reminders/gio/time_local"),
            json!("24:00"),
        ),
        (
            "deceased with sinh_nhat",
            format!("{p0}/reminders/sinh_nhat"),
            base["persons"][0]["reminders"]["gio"].clone(),
        ),
        (
            "living noi_an_tang",
            format!("{p2}/noi_an_tang"),
            json!("x"),
        ),
        (
            "solar leap true",
            format!("{p0}/death/input/leap"),
            json!(true),
        ),
        (
            "solar month_day",
            format!("{p0}/death/input"),
            json!({"calendar": "solar", "precision": "month_day", "year": null, "month": 1, "day": 1, "leap": false}),
        ),
        (
            "death precision year",
            format!("{p0}/death/input"),
            json!({"calendar": "solar", "precision": "year", "year": 2000, "month": null, "day": null, "leap": false}),
        ),
        (
            "event lunar month_day recurring ok",
            "/events/0/date".into(),
            json!({"calendar": "lunar", "precision": "month_day", "year": null, "month": 1, "day": 1, "leap": false}),
        ),
        ("event month_day not recurring", "/events/0".into(), {
            let mut e = base["events"][0].clone();
            e["date"] = json!({"calendar": "lunar", "precision": "month_day", "year": null, "month": 1, "day": 1, "leap": false});
            e["recurring"] = json!(false);
            e
        }),
        (
            "builtin wrong id",
            "/event_types/0/id".into(),
            json!("6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0005"),
        ),
        (
            "photo stripped false",
            "/photos/0/stripped".into(),
            json!(false),
        ),
        (
            "photo sha upper",
            "/photos/0/sha256".into(),
            json!("A".repeat(64)),
        ),
        (
            "upcoming_days 31",
            "/settings/upcoming_days".into(),
            json!(31),
        ),
        (
            "backup_nag_days 45",
            "/settings/backup_nag_days".into(),
            json!(45),
        ),
        ("schema_version 2", "/schema_version".into(), json!(2)),
        ("format wrong", "/format".into(), json!("other")),
        ("app version bad", "/app/version".into(), json!("0.2")),
        (
            "duplicate person id",
            "/persons/1/id".into(),
            json!("0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01"),
        ),
    ];
    let mut out = Vec::new();
    for (name, ptr, val) in raw {
        let d = apply_patch(base, &ptr, &val);
        let text = canonical_json(&d);
        let (code, path) = check(&text);
        out.push(json!({"name": name, "pointer": ptr, "value": val, "code": code, "path": path}));
    }
    // JSON lồng sâu (S12): 33 và 10 000 cấp — từ chối trước khi phân tích; JSON hỏng.
    for (name, text) in [
        ("depth 33", format!("{}{}", "[".repeat(33), "]".repeat(33))),
        (
            "depth 10000",
            format!("{}{}", "[".repeat(10_000), "]".repeat(10_000)),
        ),
        (
            "depth 32 array (not a doc)",
            format!("{}{}", "[".repeat(32), "]".repeat(32)),
        ),
        ("invalid json", "{\"format\":".to_owned()),
    ] {
        let (code, path) = check(&text);
        out.push(json!({"name": name, "raw": text, "code": code, "path": path}));
    }
    out
}

fn check(text: &str) -> (String, String) {
    match validate::parse_json(text).and_then(|v| validate::document(&v)) {
        Ok(()) => ("OK".to_owned(), String::new()),
        Err(e) => (e.code.to_owned(), e.path),
    }
}

/// Áp một thay đổi (JSON Pointer, giá trị mới hoặc "__delete__") — bản JS: pwa/test/store/run.mjs `applyPatch`.
pub fn apply_patch(base: &Value, ptr: &str, val: &Value) -> Value {
    let mut d = base.clone();
    if !ptr.is_empty() {
        let (parent, key) = ptr.rsplit_once('/').unwrap();
        let target = d.pointer_mut(parent).unwrap();
        if val == &json!("__delete__") {
            target.as_object_mut().unwrap().remove(key);
        } else if let Some(a) = target.as_array_mut() {
            a[key.parse::<usize>().unwrap()] = val.clone();
        } else {
            target[key] = val.clone();
        }
    }
    d
}

fn et(id: &str, ten: &str, rev: i64, upd: &str) -> Value {
    json!({"id": id, "builtin": null, "ten": ten, "default_recurring": false, "created_at": TS, "updated_at": upd, "rev": rev})
}

/// 31 ca gộp (R6(e)) với kết quả mong đợi viết tường minh theo 01 §6.3 (không tính bằng `merge_docs`).
pub fn merge_cases(example: &Value) -> Vec<Value> {
    let mut cases = Vec::new();
    let t0 = "2026-01-01T00:00:00Z";
    let t1 = "2026-06-01T00:00:00Z";
    let rel = [("lt", -1i64), ("eq", 0), ("gt", 1)];
    // 27 ca: {Person, Event, EventType} × rev (<, =, >) × updated_at (<, =, >).
    for kind in ["persons", "events", "event_types"] {
        for (rn, rd) in rel {
            for (un, ud) in rel {
                let local = example.clone();
                let mut incoming = example.clone();
                let idx = if kind == "event_types" { 3 } else { 0 };
                let mut item = incoming[kind][idx].clone();
                let lrev = local[kind][idx]["rev"].as_i64().unwrap() + 1;
                let mut l2 = local.clone();
                l2[kind][idx]["rev"] = json!(lrev);
                l2[kind][idx]["updated_at"] = json!(t0);
                item["rev"] = json!(lrev + rd);
                item["updated_at"] = json!(match ud {
                    -1 => "2025-12-31T23:59:59Z",
                    0 => t0,
                    _ => t1,
                });
                if kind == "persons" {
                    item["ghi_chu"] = json!("bản nhập");
                } else if kind == "events" {
                    item["tieu_de"] = json!("bản nhập");
                } else {
                    item["ten"] = json!("Du học (nhập)");
                }
                incoming[kind][idx] = item;
                // 01 §6.3: rev lớn hơn thắng; bằng → updated_at muộn hơn thắng; vẫn bằng → giữ bản trên máy.
                let winner = if rd > 0 || (rd == 0 && ud > 0) {
                    "incoming"
                } else {
                    "local"
                };
                let id = l2[kind][idx]["id"].clone();
                cases.push(json!({"name": format!("{kind} rev {rn} updated_at {un}"), "local": l2, "incoming": incoming,
                    "expect": {"winner": [{"collection": kind, "id": id, "from": winner}]}}));
            }
        }
    }
    // 28: loại tự tạo trùng tên (khác id, khác hoa/thường) → gộp về id trên máy, đổi type_id của sự kiện nhập.
    {
        let lid = uid("merge-type-local");
        let iid = uid("merge-type-incoming");
        let mut local = example.clone();
        local["event_types"]
            .as_array_mut()
            .unwrap()
            .push(et(&lid, "Lễ mừng thọ", 1, TS));
        let mut incoming = example.clone();
        incoming["event_types"]
            .as_array_mut()
            .unwrap()
            .push(et(&iid, "LỄ MỪNG THỌ", 5, TS));
        let eid = uid("merge-event-new");
        let mut ev = example["events"][0].clone();
        ev["id"] = json!(eid);
        ev["type_id"] = json!(iid);
        incoming["events"].as_array_mut().unwrap().push(ev);
        cases.push(json!({"name": "event type same name different id → local id, remap", "local": local, "incoming": incoming,
            "expect": {"type_ids_absent": [iid], "type_ids_present": [lid], "event_type_id": [{"event": eid, "type_id": lid}]}}));
    }
    // 29: trùng tên dạng NFD.
    {
        let lid = uid("merge-type-local-nfc");
        let iid = uid("merge-type-incoming-nfd");
        let mut local = example.clone();
        local["event_types"]
            .as_array_mut()
            .unwrap()
            .push(et(&lid, "Giỗ tổ họ Nguyễn", 1, TS));
        let mut incoming = example.clone();
        incoming["event_types"].as_array_mut().unwrap().push(et(
            &iid,
            "Gio\u{302}\u{303} to\u{302}\u{309} ho\u{323} Nguye\u{302}\u{303}n",
            1,
            TS,
        ));
        cases.push(json!({"name": "event type NFD name equals NFC name", "local": local, "incoming": incoming,
            "expect": {"type_ids_absent": [iid], "type_ids_present": [lid]}}));
    }
    // 30: ảnh trùng (cùng id, cùng sha) → một bản ghi; người mới có ảnh mới → thêm ảnh.
    {
        let local = example.clone();
        let mut incoming = example.clone();
        let pid = uid("merge-photo-new");
        let mut ph = example["photos"][0].clone();
        ph["id"] = json!(pid);
        incoming["photos"].as_array_mut().unwrap().push(ph);
        let mut p = example["persons"][1].clone();
        let nid = uid("merge-person-new");
        p["id"] = json!(nid);
        p["photo_id"] = json!(pid);
        p["reminders"]["gio"]["id"] = json!(uid("merge-rule-new"));
        incoming["persons"].as_array_mut().unwrap().push(p);
        cases.push(json!({"name": "duplicate photo kept once; new person brings new photo", "local": local, "incoming": incoming,
            "expect": {"photo_count": 2, "person_count": 4, "photo_ids_present": [example["photos"][0]["id"], pid]}}));
    }
    // 31: ảnh nhập không được ai tham chiếu (người nhập thua theo rev) → không thêm ảnh.
    {
        let local = example.clone();
        let mut incoming = example.clone();
        let pid = uid("merge-photo-orphan");
        let mut ph = example["photos"][0].clone();
        ph["id"] = json!(pid);
        incoming["photos"].as_array_mut().unwrap().push(ph);
        incoming["persons"][1]["photo_id"] = json!(pid);
        incoming["persons"][1]["rev"] = json!(1); // local rev 2 → local thắng
        cases.push(json!({"name": "incoming photo of losing person is dropped", "local": local, "incoming": incoming,
            "expect": {"photo_count": 1, "photo_ids_absent": [pid],
                       "winner": [{"collection": "persons", "id": example["persons"][1]["id"], "from": "local"}]}}));
    }
    cases
}

fn main() {
    let d = dir();
    std::fs::create_dir_all(d.join("corrupt")).unwrap();
    let photo = std::fs::read(d.join(EXAMPLE_PHOTO)).unwrap();
    let ex = example_doc(&photo);
    let ex_json = canonical_json(&ex);
    std::fs::write(d.join("example-v1.json"), &ex_json).unwrap();
    let blob = lvhv_backup::PhotoBlob {
        id: lvhv_backup::uuid_from_str(ex["photos"][0]["id"].as_str().unwrap()).unwrap(),
        bytes: photo.clone(),
    };
    let file =
        lvhv_backup::export_with_fixed_rng(PASSWORD, &ex_json, &[blob], SALT, NONCE).unwrap();
    std::fs::write(d.join("example-v1.lvhv"), &file).unwrap();
    let mut corrupt = Vec::new();
    for (name, spec, code) in corrupt_specs() {
        let f = apply_corruption(&file, &spec);
        std::fs::write(d.join(format!("corrupt/{name}.lvhv")), &f).unwrap();
        corrupt.push(json!({"file": format!("corrupt/{name}.lvhv"), "change": spec, "expect": code, "sha256": sha256_hex(&f)}));
    }
    let (ds, mut ds_photos) = dataset_r6();
    std::fs::write(d.join("dataset-r6.json"), canonical_json(&ds)).unwrap();
    // Băm trạng thái kho sau khi nạp bộ R6 vào SQLite, và tệp sao lưu salt/nonce cố định của bộ R6: bản PWA
    // phải cho đúng hai giá trị này (R6 (a), (d) — cùng byte).
    ds_photos.sort_by(|a, b| a.0.cmp(&b.0));
    let blobs: Vec<lvhv_backup::PhotoBlob> = ds_photos
        .iter()
        .map(|(id, b)| lvhv_backup::PhotoBlob {
            id: lvhv_backup::uuid_from_str(id).unwrap(),
            bytes: b.clone(),
        })
        .collect();
    let tmp = std::env::temp_dir().join(format!("lvhv-gen-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let mut st = lvhv_store::Store::open(&tmp).unwrap();
    let v = lvhv_store::verify_contents(&lvhv_backup::Contents {
        json: canonical_json(&ds),
        photos: blobs.clone(),
    })
    .unwrap();
    st.apply_import(&v, lvhv_store::ImportMode::Replace)
        .unwrap();
    let ds_state = st.state_hash().unwrap();
    let ds_file =
        lvhv_backup::export_with_fixed_rng(PASSWORD, &canonical_json(&ds), &blobs, SALT, NONCE)
            .unwrap();
    drop(st);
    let _ = std::fs::remove_dir_all(&tmp);
    let vc = validation_cases(&ex);
    std::fs::write(
        d.join("validation-cases.json"),
        serde_json::to_string(&vc).unwrap() + "\n",
    )
    .unwrap();
    let mut mc = merge_cases(&ex);
    for c in &mut mc {
        let m = lvhv_store::merge_docs(&c["local"], &c["incoming"]);
        validate::document(&m).unwrap();
        c["merged_sha256"] = json!(sha256_hex(canonical_json(&m).as_bytes()));
    }
    std::fs::write(
        d.join("merge-cases.json"),
        serde_json::to_string(&mc).unwrap() + "\n",
    )
    .unwrap();
    let manifest = json!({
        "format": "lvhv-backup test vectors, file_version 1 (docs/v0.2/03 §2.5)",
        "generator": "cargo run --release -p lvhv-store --example gen_test_vectors",
        "password": PASSWORD,
        "password_utf8_nfc_hex": PASSWORD.nfc_hex(),
        "salt_hex": hexs(&SALT), "nonce_hex": hexs(&NONCE),
        "kdf": {"algorithm": "argon2id", "version": 19, "m_cost_kib": 65536, "t_cost": 3, "p_cost": 4, "out_len": 32},
        "key_hex": hexs(lvhv_backup::derive_key(PASSWORD, &SALT, 65536, 3, 4).unwrap().as_ref()),
        "example": {"file": "example-v1.lvhv", "sha256": sha256_hex(&file), "json": "example-v1.json",
                    "json_sha256": sha256_hex(ex_json.as_bytes()), "photo": EXAMPLE_PHOTO, "photo_sha256": sha256_hex(&photo)},
        "corrupt": corrupt,
        "dataset_r6": {"file": "dataset-r6.json", "photo_bytes": "r6_photo_bytes(i), i = 0..19 (store/examples/gen_test_vectors.rs)",
                       "store_state_sha256": ds_state, "fixed_rng_file_sha256": sha256_hex(&ds_file), "fixed_rng_file_len": ds_file.len()},
        "validation_cases": vc.len(), "merge_cases": mc.len(),
    });
    std::fs::write(
        d.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap() + "\n",
    )
    .unwrap();
    println!(
        "ok: {} corrupt, {} validation cases, {} merge cases",
        corrupt.len(),
        vc.len(),
        mc.len()
    );
}

fn hexs(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

trait NfcHex {
    fn nfc_hex(&self) -> String;
}
impl NfcHex for str {
    fn nfc_hex(&self) -> String {
        hexs(&lvhv_backup::normalize_password(self))
    }
}
