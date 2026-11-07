//! R6(d) kiểm chéo Rust ↔ PWA qua tệp thật (không salt/nonce cố định), trên bộ R6v2 (04 Bổ sung B7):
//!
//!     cargo run --release -p lvhv-store --example cross_platform -- write   # Rust xuất bộ R6 → pwa/test/store/out/r6-from-rust.lvhv
//!     node pwa/test/store/run.mjs                                            # JS mở tệp Rust; JS xuất r6-from-pwa.lvhv
//!     cargo run --release -p lvhv-store --example cross_platform -- check   # Rust mở tệp của PWA → so state_hash
//!
//! `scripts/check-store.sh` chạy cả ba bước (CI gọi script này). Bước `write` còn chạy kịch bản dọn ảnh không
//! còn dùng (`photo_gc`) trên bộ R6 và ghi kết quả ra `photo-gc-rust.json`; PWA chạy cùng kịch bản và so.
use lvhv_backup::PhotoBlob;
use lvhv_store::{canonical_json, open_backup, AppInfo, ImportMode, Store};
use serde_json::Value;
use std::path::PathBuf;

#[path = "gen_test_vectors.rs"]
#[allow(dead_code)]
mod gen;

fn out() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../pwa/test/store/out")
}

fn seeded(dir: &PathBuf) -> Store {
    let _ = std::fs::remove_dir_all(dir);
    let mut s = Store::open(dir).unwrap();
    // 04 Bổ sung B7: bộ R6v2 (ảnh thật) — bộ R6 cũ bị kiểm tra chặt B6 từ chối khi nhập.
    let (ds, mut photos) = gen::dataset_r6v2();
    photos.sort_by(|a, b| a.0.cmp(&b.0));
    let blobs = photos
        .iter()
        .map(|(id, b)| PhotoBlob {
            id: lvhv_backup::uuid_from_str(id).unwrap(),
            bytes: b.clone(),
        })
        .collect();
    let v = lvhv_store::verify_contents(&lvhv_backup::Contents {
        json: canonical_json(&ds),
        photos: blobs,
    })
    .unwrap();
    s.apply_import(&v, ImportMode::Replace).unwrap();
    s
}

/// Kịch bản dọn ảnh, giống hệt `photoGc` trong pwa/test/store/run.mjs. Hai hồ sơ có ảnh đầu tiên (theo id) là
/// A và B. (1) A đổi sang ảnh của B → ảnh cũ của A bị xóa. (2) B bỏ ảnh → ảnh vẫn còn A dùng nên giữ.
/// (3) `delete_photo` ảnh đang dùng → lỗi "input"; ảnh đã xóa → "not_found".
fn photo_gc(s: &mut Store) -> Value {
    let mut ps: Vec<Value> = s
        .persons()
        .unwrap()
        .into_iter()
        .filter(|p| p["photo_id"].is_string())
        .collect();
    ps.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    let (mut a, mut b) = (ps[0].clone(), ps[1].clone());
    let (pa, pb) = (a["photo_id"].clone(), b["photo_id"].clone());
    let bump = |p: &mut Value| {
        p["rev"] = serde_json::json!(p["rev"].as_i64().unwrap() + 1);
        p["updated_at"] = serde_json::json!("2026-11-07T00:00:00Z");
    };
    a["photo_id"] = pb.clone();
    bump(&mut a);
    s.put_person(&a).unwrap();
    let after1 = s.photos().unwrap().len();
    b["photo_id"] = Value::Null;
    bump(&mut b);
    s.put_person(&b).unwrap();
    let after2 = s.photos().unwrap().len();
    let code = |r: lvhv_store::Result<()>| match r {
        Ok(()) => "ok",
        Err(lvhv_store::StoreError::Input(_)) => "input",
        Err(lvhv_store::StoreError::NotFound) => "not_found",
        Err(_) => "other",
    };
    let e_used = code(s.delete_photo(pb.as_str().unwrap()));
    let e_gone = code(s.delete_photo(pa.as_str().unwrap()));
    serde_json::json!({
        "photos_after": [after1, after2],
        "delete_used": e_used,
        "delete_gone": e_gone,
        "state_sha256": s.state_hash().unwrap(),
    })
}

/// Gợi ý cách gọi / quan hệ và quy tắc viết hoa (YC-2026-07) — PWA (`names.ts`) so với tệp này.
fn names_doc() -> Value {
    use lvhv_store::names::*;
    let mut dx = serde_json::Map::new();
    for (k, dead) in [("deceased", true), ("living", false)] {
        for g in [None, Some("nam"), Some("nu")] {
            dx.insert(
                format!("{k}:{}", g.unwrap_or("null")),
                serde_json::json!(danh_xung_suggestions(dead, g)),
            );
        }
    }
    let samples = [
        "ông nội",
        "o\u{0302}\u{0300}ng",
        "đ",
        "Bà Cô Tổ họ",
        "bé đỏ",
        "",
        "1 ông",
        "ê",
        "ơ ba",
        "ưu",
        "ỷ",
        "  chú  ",
        "   ",
        "\u{3000}bà\u{feff}",
    ];
    serde_json::json!({
        "danh_xung": dx,
        "quan_he": {"deceased": quan_he_suggestions(true), "living": quan_he_suggestions(false)},
        "capitalize": samples.iter().map(|s| serde_json::json!([s, capitalize_first(s)])).collect::<Vec<_>>(),
    })
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    let tmp = std::env::temp_dir().join(format!("lvhv-cross-{}", std::process::id()));
    let manifest: Value = serde_json::from_slice(
        &std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../docs/v0.2/schema/test-vectors/r6v2/manifest.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let want = manifest["store_state_sha256"].as_str().unwrap().to_owned();
    match mode.as_str() {
        "write" => {
            std::fs::create_dir_all(out()).unwrap();
            let mut s = seeded(&tmp.join("src"));
            assert_eq!(s.state_hash().unwrap(), want);
            let f = s
                .export_backup(
                    gen::PASSWORD,
                    &AppInfo {
                        version: "0.2.0".into(),
                        platform: "windows".into(),
                    },
                )
                .unwrap();
            std::fs::write(out().join("r6-from-rust.lvhv"), &f).unwrap();
            println!("ok: r6-from-rust.lvhv ({} byte)", f.len());
            let mut g = seeded(&tmp.join("gc"));
            let r = photo_gc(&mut g);
            assert_eq!(r["photos_after"], serde_json::json!([19, 19]), "{r}");
            assert_eq!(
                (r["delete_used"].as_str(), r["delete_gone"].as_str()),
                (Some("input"), Some("not_found"))
            );
            std::fs::write(out().join("photo-gc-rust.json"), canonical_json(&r)).unwrap();
            std::fs::write(out().join("names-rust.json"), canonical_json(&names_doc())).unwrap();
            // PR #30: Gộp tệp có tệp đính kèm (vector bổ sung YC-05/07) vào kho mới — PWA so state hash.
            let nf = std::fs::read(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../docs/v0.2/schema/test-vectors/supplement-yc05/notice-v1.lvhv"),
            )
            .unwrap();
            let nv = open_backup(&nf, gen::PASSWORD).unwrap();
            let d = tmp.join("merge");
            let _ = std::fs::remove_dir_all(&d);
            let mut m = Store::open(&d).unwrap();
            m.apply_import(&nv, ImportMode::Merge).unwrap();
            assert_eq!(m.attachments().unwrap().len(), 2);
            std::fs::write(
                out().join("notice-merge-rust.json"),
                canonical_json(&serde_json::json!({"state_sha256": m.state_hash().unwrap()})),
            )
            .unwrap();
            println!("ok: photo-gc-rust.json ({})", r["state_sha256"]);
        }
        "check" => {
            let f = std::fs::read(out().join("r6-from-pwa.lvhv"))
                .expect("chưa có r6-from-pwa.lvhv — chạy node pwa/test/store/run.mjs");
            let v = open_backup(&f, gen::PASSWORD).unwrap();
            assert_eq!(v.preview().platform, "pwa");
            let dir = tmp.join("dst");
            let _ = std::fs::remove_dir_all(&dir);
            let mut s = Store::open(&dir).unwrap();
            s.apply_import(&v, ImportMode::Replace).unwrap();
            drop(s);
            let s = Store::open(&dir).unwrap();
            let got = s.state_hash().unwrap();
            assert_eq!(got, want, "tệp PWA nhập vào SQLite không giống hệt");
            println!("ok: r6-from-pwa.lvhv nhập vào SQLite → state_hash = {got}");
        }
        "dump" => {
            let s = seeded(&tmp.join("dump"));
            let mut doc = s
                .snapshot(
                    &AppInfo {
                        version: "0.0.0".into(),
                        platform: "pwa".into(),
                    },
                    "1970-01-01T00:00:00Z",
                )
                .unwrap();
            doc.as_object_mut().unwrap().remove("app");
            doc.as_object_mut().unwrap().remove("exported_at");
            std::fs::write(out().join("rust-state.json"), canonical_json(&doc)).unwrap();
        }
        _ => {
            eprintln!("dùng: cross_platform write|check");
            std::process::exit(2);
        }
    }
    let _ = std::fs::remove_dir_all(&tmp);
}
