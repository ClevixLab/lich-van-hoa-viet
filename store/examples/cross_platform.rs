//! R6(d) kiểm chéo Rust ↔ PWA qua tệp thật (không salt/nonce cố định):
//!
//!     cargo run --release -p lvhv-store --example cross_platform -- write   # Rust xuất bộ R6 → pwa/test/store/out/r6-from-rust.lvhv
//!     node pwa/test/store/run.mjs                                            # JS mở tệp Rust; JS xuất r6-from-pwa.lvhv
//!     cargo run --release -p lvhv-store --example cross_platform -- check   # Rust mở tệp của PWA → so state_hash
//!
//! `scripts/check-store.sh` chạy cả ba bước (CI gọi script này).
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
    let (ds, mut photos) = gen::dataset_r6();
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

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    let tmp = std::env::temp_dir().join(format!("lvhv-cross-{}", std::process::id()));
    let manifest: Value = serde_json::from_slice(
        &std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../docs/v0.2/schema/test-vectors/manifest.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let want = manifest["dataset_r6"]["store_state_sha256"]
        .as_str()
        .unwrap()
        .to_owned();
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
