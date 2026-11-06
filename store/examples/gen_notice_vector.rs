//! Vector **bổ sung** cho thông tin báo tử (01 §3.2a, YC-2026-05) — không đụng vector đã khóa của 03 §2.5
//! (`docs/v0.2/schema/test-vectors/*.json`, `manifest.json` giữ nguyên). Ghi vào
//! `docs/v0.2/schema/test-vectors/supplement-yc05/`:
//! - `notice-v1.json`     — `example-v1.json` + `death.notice` đủ trường cho người thứ nhất (đã mất) + một ảnh giấy
//!   báo tử (`Photo` thứ hai, cùng byte ảnh mẫu), JSON chuẩn.
//! - `notice-v1.lvhv`     — tệp sao lưu của `notice-v1.json` + hai ảnh, salt/nonce cố định như 03 §2.5, mật khẩu
//!   "mật khẩu thử nghiệm".
//! - `manifest.json`      — sha256 từng tệp, `state_sha256` (băm kho sau khi nhập, như R6).
//!
//!     cargo run --release -p lvhv-store --example gen_notice_vector
//!
//! `store/tests/notice.rs` sinh lại trong bộ nhớ và so từng byte; `pwa/test/store/run.mjs` kiểm bản JS.
use lvhv_store::{canonical_json, sha256_hex, validate, AppInfo, ImportMode, Store};
use serde_json::{json, Value};
use std::path::PathBuf;

pub const PASSWORD: &str = "mật khẩu thử nghiệm";
pub const SALT: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
pub const NONCE: [u8; 24] = [
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
];
/// Id ảnh giấy báo tử (UUID v4 cố định).
pub const CERT_ID: &str = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e0c";

pub fn tv() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/v0.2/schema/test-vectors")
}
pub fn out_dir() -> PathBuf {
    tv().join("supplement-yc05")
}

/// (tài liệu, ảnh theo thứ tự `photos`).
pub fn notice_doc() -> (Value, Vec<(String, Vec<u8>)>) {
    let photo = std::fs::read(tv().join("photo-0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e09.jpg")).unwrap();
    let text = std::fs::read_to_string(tv().join("example-v1.json")).unwrap();
    let mut doc = validate::parse_json(&text).unwrap();
    let mut cert = doc["photos"][0].clone();
    cert["id"] = json!(CERT_ID);
    doc["photos"].as_array_mut().unwrap().push(cert);
    // Dùng từ xưng hô thường ngày trên bia (chủ dự án, 07/11/2026), không dùng "Hiển khảo"/"Hiển tỷ" của tệp mẫu.
    doc["persons"][0]["memorial"]["bia_tieu_de"] = json!("Ông nội");
    doc["persons"][1]["memorial"]["bia_tieu_de"] = json!("Bà nội");
    doc["persons"][0]["death"]["notice"] = json!({
        "reported_by": "Chị Lê Thị C",
        "place": "Bệnh viện đa khoa tỉnh",
        "reason": "Tuổi cao, bệnh nặng",
        "notice_date": {"calendar": "solar", "precision": "day", "year": 2025, "month": 8, "day": 8, "leap": false},
        "certificate_photo_ids": [CERT_ID],
        "note": "Gia đình đã báo họ hàng.\nAn táng ở quê."
    });
    validate::document(&doc).unwrap();
    let mut photos: Vec<(String, Vec<u8>)> = doc["photos"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p["id"].as_str().unwrap().to_owned(), photo.clone()))
        .collect();
    photos.sort_by(|a, b| a.0.cmp(&b.0));
    (doc, photos)
}

/// Tệp sao lưu tất định của `notice_doc`.
pub fn notice_file(doc: &Value, photos: &[(String, Vec<u8>)]) -> Vec<u8> {
    let blobs: Vec<lvhv_backup::PhotoBlob> = photos
        .iter()
        .map(|(id, b)| lvhv_backup::PhotoBlob {
            id: lvhv_backup::uuid_from_str(id).unwrap(),
            bytes: b.clone(),
        })
        .collect();
    lvhv_backup::export_with_fixed_rng(PASSWORD, &canonical_json(doc), &blobs, SALT, NONCE).unwrap()
}

/// Băm kho sau khi nhập "Thay toàn bộ" tệp vector vào kho trống.
pub fn state_after_import(file: &[u8]) -> String {
    let dir = std::env::temp_dir().join(format!("lvhv-notice-vec-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut s = Store::open(&dir).unwrap();
    let v = lvhv_store::open_backup(file, PASSWORD).unwrap();
    s.apply_import(&v, ImportMode::Replace).unwrap();
    let h = s.state_hash().unwrap();
    let _ = AppInfo {
        version: String::new(),
        platform: String::new(),
    };
    drop(s);
    let _ = std::fs::remove_dir_all(&dir);
    h
}

pub fn manifest(json_text: &str, file: &[u8], state: &str) -> String {
    let m = json!({
        "description": "Vector bổ sung YC-2026-05 (thông tin báo tử, 01 §3.2a). Không thay vector đã khóa của 03 §2.5.",
        "password": PASSWORD,
        "salt_hex": SALT.iter().map(|b| format!("{b:02x}")).collect::<String>(),
        "nonce_hex": NONCE.iter().map(|b| format!("{b:02x}")).collect::<String>(),
        "json": {"file": "notice-v1.json", "sha256": sha256_hex(json_text.as_bytes())},
        "file": {"file": "notice-v1.lvhv", "sha256": sha256_hex(file)},
        "cert_photo_id": CERT_ID,
        "state_sha256": state,
    });
    let mut s = serde_json::to_string_pretty(&m).unwrap();
    s.push('\n');
    s
}

#[allow(dead_code)]
fn main() {
    let (doc, photos) = notice_doc();
    let text = canonical_json(&doc);
    let file = notice_file(&doc, &photos);
    let state = state_after_import(&file);
    std::fs::create_dir_all(out_dir()).unwrap();
    std::fs::write(out_dir().join("notice-v1.json"), &text).unwrap();
    std::fs::write(out_dir().join("notice-v1.lvhv"), &file).unwrap();
    std::fs::write(
        out_dir().join("manifest.json"),
        manifest(&text, &file, &state),
    )
    .unwrap();
    println!("ok: supplement-yc05 ({} byte, state {state})", file.len());
}
