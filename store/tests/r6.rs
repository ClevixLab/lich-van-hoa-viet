//! R6 (docs/v0.2/04) phía Rust/SQLite: (a) khứ hồi, (a2) sai mật khẩu, (b) chống sửa + lật bit,
//! (d) tệp vector dùng chung (tệp xuất từ PWA nằm trong pwa/test/store/ và được kiểm ở `cross_platform`),
//! (e) 31 ca gộp; thêm V1–V8 (`validation-cases.json`), S12 và tính tất định của vector.
use lvhv_backup::{BackupError, PhotoBlob};
use lvhv_store::{
    canonical_json, open_backup, sha256_hex, validate, AppInfo, ImportMode, Store, StoreError,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[path = "../examples/gen_test_vectors.rs"]
#[allow(dead_code)]
mod gen;

fn tv() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/v0.2/schema/test-vectors")
}
fn read(p: impl AsRef<Path>) -> Vec<u8> {
    std::fs::read(tv().join(p)).unwrap()
}
fn tmpdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("lvhv-store-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}
fn app() -> AppInfo {
    AppInfo {
        version: "0.2.0".into(),
        platform: "windows".into(),
    }
}

/// Đưa tài liệu + byte ảnh vào một kho trống bằng đường nhập "Thay toàn bộ".
fn seed(store: &mut Store, doc: &Value, photos: &[(String, Vec<u8>)]) {
    let blobs: Vec<PhotoBlob> = photos
        .iter()
        .map(|(id, b)| PhotoBlob {
            id: lvhv_backup::uuid_from_str(id).unwrap(),
            bytes: b.clone(),
        })
        .collect();
    let c = lvhv_backup::Contents {
        json: canonical_json(doc),
        photos: blobs,
    };
    let v = lvhv_store::verify_contents(&c).unwrap();
    store.apply_import(&v, ImportMode::Replace).unwrap();
}

/// Ảnh của bộ R6v2 (04 Bổ sung B7 — R6 (a)–(d) và lật bit chạy trên bộ này), sắp theo id.
fn r6_photos() -> Vec<(String, Vec<u8>)> {
    let (doc, photos) = gen::dataset_r6v2();
    let ids: Vec<String> = doc["photos"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap().to_owned())
        .collect();
    let mut out = photos;
    out.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(out.iter().map(|x| x.0.clone()).collect::<Vec<_>>(), ids);
    out
}

#[test]
fn vectors_are_deterministic_and_committed() {
    let photo = read(gen::EXAMPLE_PHOTO);
    let ex = gen::example_doc(&photo);
    assert_eq!(canonical_json(&ex).into_bytes(), read("example-v1.json"));
    let blob = PhotoBlob {
        id: lvhv_backup::uuid_from_str(ex["photos"][0]["id"].as_str().unwrap()).unwrap(),
        bytes: photo,
    };
    let f = lvhv_backup::export_with_fixed_rng(
        gen::PASSWORD,
        &canonical_json(&ex),
        &[blob],
        gen::SALT,
        gen::NONCE,
    )
    .unwrap();
    assert_eq!(
        f,
        read("example-v1.lvhv"),
        "example-v1.lvhv khác bản sinh lại"
    );
    let (ds, _) = gen::dataset_r6();
    assert_eq!(canonical_json(&ds).into_bytes(), read("dataset-r6.json"));
    let m: Value = serde_json::from_slice(&read("manifest.json")).unwrap();
    assert_eq!(m["example"]["sha256"], json!(sha256_hex(&f)));
    let key = lvhv_backup::derive_key(gen::PASSWORD, &gen::SALT, 65536, 3, 4).unwrap();
    assert_eq!(
        m["key_hex"].as_str().unwrap(),
        key.iter().map(|x| format!("{x:02x}")).collect::<String>()
    );
}

/// 04 Bổ sung B7: bộ R6v2 đã khóa — sinh lại đúng từng byte, sha256 mọi tệp khớp manifest, tệp salt/nonce cố định
/// mở được và cho đúng băm trạng thái; mọi ảnh qua kiểm tra chặt B6.
#[test]
fn r6v2_vectors_locked() {
    let d = gen::r6v2_dir();
    let m: Value =
        serde_json::from_slice(&std::fs::read(d.join("manifest.json")).unwrap()).unwrap();
    for (f, h) in m["files"].as_object().unwrap() {
        assert_eq!(
            sha256_hex(&std::fs::read(d.join(f)).unwrap()),
            h.as_str().unwrap(),
            "{f} khác sha256 đã khóa"
        );
    }
    assert_eq!(m["files"].as_object().unwrap().len(), 23);
    let (ds, _) = gen::dataset_r6v2();
    assert_eq!(
        canonical_json(&ds).into_bytes(),
        std::fs::read(d.join("dataset-r6v2.json")).unwrap()
    );
    let file = std::fs::read(d.join("dataset-r6v2-fixed.lvhv")).unwrap();
    let v = open_backup(&file, gen::PASSWORD).unwrap();
    assert_eq!(
        (v.preview().persons, v.preview().events, v.preview().photos),
        (60, 30, 20)
    );
    let dir = tmpdir("r6v2");
    let mut s = Store::open(&dir).unwrap();
    s.apply_import(&v, ImportMode::Replace).unwrap();
    assert_eq!(
        s.state_hash().unwrap(),
        m["store_state_sha256"].as_str().unwrap()
    );
    let _ = std::fs::remove_dir_all(dir);
}

/// 04 Bổ sung B7: bộ R6 cũ (ảnh giả: `FF D8 FF E0 …`, 10 ảnh khai `image/webp`) nay bị **từ chối** khi nhập (B6),
/// với thông báo chung; vector cũ vẫn sinh lại đúng từng byte (vectors_are_deterministic_and_committed).
#[test]
fn old_r6_rejected_on_import() {
    let (ds, mut photos) = gen::dataset_r6();
    photos.sort_by(|a, b| a.0.cmp(&b.0));
    let blobs: Vec<PhotoBlob> = photos
        .iter()
        .map(|(id, b)| PhotoBlob {
            id: lvhv_backup::uuid_from_str(id).unwrap(),
            bytes: b.clone(),
        })
        .collect();
    let e = lvhv_store::verify_contents(&lvhv_backup::Contents {
        json: canonical_json(&ds),
        photos: blobs.clone(),
    })
    .unwrap_err();
    assert_eq!(e.import_message(), "Sai mật khẩu hoặc tệp bị hỏng");
    // Tệp salt/nonce cố định của R6 cũ (đúng sha256 đã khóa) cũng bị từ chối qua đường nhập thật.
    let f = lvhv_backup::export_with_fixed_rng(
        gen::PASSWORD,
        &canonical_json(&ds),
        &blobs,
        gen::SALT,
        gen::NONCE,
    )
    .unwrap();
    let m: Value = serde_json::from_slice(&read("manifest.json")).unwrap();
    assert_eq!(
        m["dataset_r6"]["fixed_rng_file_sha256"],
        json!(sha256_hex(&f))
    );
    assert!(open_backup(&f, gen::PASSWORD).is_err());
}

#[test]
fn example_vector_opens() {
    let v = open_backup(&read("example-v1.lvhv"), gen::PASSWORD).unwrap();
    assert_eq!(
        canonical_json(v.document()).into_bytes(),
        read("example-v1.json")
    );
    let p = v.preview();
    assert_eq!(
        (p.persons, p.events, p.photos, p.exported_at.as_str()),
        (3, 1, 1, "2026-11-06T03:15:00Z")
    );
}

#[test]
fn corrupt_vectors_rejected_with_expected_code() {
    let m: Value = serde_json::from_slice(&read("manifest.json")).unwrap();
    let file = read("example-v1.lvhv");
    let hdr = lvhv_backup::parse_header(&file).unwrap();
    let key = lvhv_backup::derive_key(gen::PASSWORD, &hdr.salt, 65536, 3, 4).unwrap();
    for c in m["corrupt"].as_array().unwrap() {
        let f = read(c["file"].as_str().unwrap());
        assert_eq!(
            gen::apply_corruption(&file, &c["change"]),
            f,
            "{}",
            c["file"]
        );
        let want = c["expect"].as_str().unwrap();
        // Lỗi phần đầu: không cần chạy Argon2. Lỗi AEAD: dùng lại khóa nếu (salt, m, t, p) không đổi, nếu không
        // chạy Argon2 thật với tham số trong tệp.
        let got = match lvhv_backup::parse_header(&f) {
            Err(e) => e,
            Ok(h) if h.salt == hdr.salt && (h.m_cost_kib, h.t_cost, h.p_cost) == (65536, 3, 4) => {
                lvhv_backup::open_with_key(&f, &h, &key).unwrap_err()
            }
            Ok(_) => lvhv_backup::import(&f, gen::PASSWORD).unwrap_err(),
        };
        assert_eq!(got.code(), want, "{}", c["file"]);
        assert_eq!(got.code(), want);
    }
}

#[test]
fn wrong_passwords_rejected_nfd_accepted_store_unchanged() {
    let file = read("example-v1.lvhv");
    let dir = tmpdir("a2");
    let mut s = Store::open(&dir).unwrap();
    let (ds, _) = gen::dataset_r6v2();
    seed(&mut s, &ds, &r6_photos());
    let before = s.state_hash().unwrap();
    let pw = gen::PASSWORD;
    let nfd: String = unicode_normalization::UnicodeNormalization::nfd(pw).collect();
    assert_ne!(nfd, pw);
    let wrong: Vec<String> = vec![
        String::new(),
        " ".into(),
        format!("{pw} "),
        format!(" {pw}"),
        pw.replacen('m', "M", 1),
        pw.replacen('ậ', "â", 1),
        pw.replacen('ẩ', "ẫ", 1),
        pw.replacen('ệ', "e", 1),
        pw.replacen("thử", "thu", 1),
        pw[..pw.len() - 1].into(),
        format!("{pw}m"),
        pw.to_uppercase(),
        "mat khau thu nghiem".into(),
        "mật khẩu thử nghiệm\n".into(),
        "mật  khẩu thử nghiệm".into(),
        "mật khẩu thử nghiệm\u{200b}".into(),
        "mật khẩu thử nghiệm\0".into(),
        "password".into(),
        "mật khẩu thử nghiệ".into(),
        "mật khẩu thử nghiệm.".into(),
    ];
    assert_eq!(wrong.len(), 20);
    for w in &wrong {
        let e = open_backup(&file, w).unwrap_err();
        assert!(
            matches!(e, StoreError::Backup(BackupError::WrongPasswordOrCorrupt)),
            "{w:?}: {e}"
        );
        assert_eq!(e.import_message(), "Sai mật khẩu hoặc tệp bị hỏng");
    }
    assert!(
        open_backup(&file, &nfd).is_ok(),
        "bản NFD của mật khẩu đúng phải được chấp nhận"
    );
    assert_eq!(s.state_hash().unwrap(), before);
}

/// R6 (b): lật từng bit của 4 096 byte đầu và 4 096 byte cuối (65 536 biến thể) → 100 % bị từ chối.
/// Khóa Argon2id được dẫn xuất một lần cho (salt, m, t, p) gốc; biến thể đổi salt/m/t/p được kiểm bằng
/// đường nhập thật (`import`, có Argon2id) cho các byte 12–39 (224 biến thể, chạy song song).
#[test]
fn bit_flip_65536_all_rejected() {
    let dir = tmpdir("b");
    let mut s = Store::open(&dir).unwrap();
    let (ds, photos) = (gen::dataset_r6v2().0, r6_photos());
    seed(&mut s, &ds, &photos);
    let pw = gen::PASSWORD;
    let file = s
        .export_backup_fixed(pw, &app(), "2026-11-06T03:15:00Z", gen::SALT, gen::NONCE)
        .unwrap();
    assert!(
        file.len() > 8192,
        "tệp R6 phải dài hơn 8 KiB để hai vùng không chồng nhau ({})",
        file.len()
    );
    let before = s.state_hash().unwrap();
    let hdr = lvhv_backup::parse_header(&file).unwrap();
    let key = lvhv_backup::derive_key(pw, &hdr.salt, 65536, 3, 4).unwrap();
    assert!(lvhv_backup::open_with_key(&file, &hdr, &key).is_ok());
    let positions: Vec<usize> = (0..4096).chain(file.len() - 4096..file.len()).collect();
    let mut slow = Vec::new();
    let mut n = 0usize;
    for &i in &positions {
        for bit in 0..8 {
            let mut f = file.clone();
            f[i] ^= 1 << bit;
            n += 1;
            match lvhv_backup::parse_header(&f) {
                Err(_) => {}
                Ok(h)
                    if h.salt == hdr.salt
                        && (h.m_cost_kib, h.t_cost, h.p_cost) == (65536, 3, 4) =>
                {
                    assert!(
                        lvhv_backup::open_with_key(&f, &h, &key).is_err(),
                        "byte {i} bit {bit} được chấp nhận"
                    );
                }
                Ok(_) => slow.push(f),
            }
        }
    }
    assert_eq!(n, 65_536);
    // Biến thể đổi salt/m/t/p: Argon2id thật (m có thể tới 1 GiB khi lật bit cao → giới hạn bằng §2.2).
    let threads = std::thread::available_parallelism()
        .map_or(2, |x| x.get())
        .min(4);
    let chunks: Vec<Vec<Vec<u8>>> = slow
        .chunks(slow.len().div_ceil(threads))
        .map(<[_]>::to_vec)
        .collect();
    let n_slow = slow.len();
    std::thread::scope(|sc| {
        for ch in chunks {
            sc.spawn(move || {
                for f in ch {
                    assert!(lvhv_backup::import(&f, pw).is_err());
                }
            });
        }
    });
    eprintln!("lật bit: 65 536 biến thể bị từ chối ({n_slow} qua Argon2id thật)");
    assert_eq!(s.state_hash().unwrap(), before);
}

/// R6 (a): bộ dữ liệu R6 (60 người, 30 sự kiện, 20 ảnh) → xuất → nhập "Thay toàn bộ" trên máy trống → giống hệt.
#[test]
fn roundtrip_replace_identical() {
    let a = tmpdir("a-src");
    let mut s = Store::open(&a).unwrap();
    let (ds, photos) = (gen::dataset_r6v2().0, r6_photos());
    seed(&mut s, &ds, &photos);
    let snap1 = s.snapshot(&app(), "2026-11-06T03:15:00Z").unwrap();
    assert_eq!(canonical_json(&snap1), canonical_json(&ds));
    let h1 = s.state_hash().unwrap();
    let file = s.export_backup("mật khẩu thử nghiệm", &app()).unwrap();
    // Đóng và mở lại kho (R2: đọc lại sau khi đóng/mở).
    drop(s);
    let mut s = Store::open(&a).unwrap();
    // MA3N-23-01: tạo byte chưa phải là đã sao lưu; chỉ mark_backup_saved (sau khi lưu tệp xong) mới ghi nhận.
    // (Bộ R6 có sẵn last_backup_at = giá trị của tệp; xuất không được đổi nó, cũng không đổi state_hash.)
    assert_eq!(
        s.settings().unwrap()["last_backup_at"],
        snap1["settings"]["last_backup_at"]
    );
    assert_eq!(s.state_hash().unwrap(), h1);
    s.mark_backup_saved("2026-11-07T01:00:00Z").unwrap();
    assert_eq!(
        s.settings().unwrap()["last_backup_at"],
        "2026-11-07T01:00:00Z"
    );
    assert_eq!(s.backup_pending_since().unwrap(), None);
    // Ghi nhận hai lần cho một lần tạo tệp → lỗi.
    assert!(s.mark_backup_saved("2026-11-07T01:00:00Z").is_err());
    let b = tmpdir("a-dst");
    let mut t = Store::open(&b).unwrap();
    let v = open_backup(&file, "mật khẩu thử nghiệm").unwrap();
    assert_eq!(v.preview().persons, 60);
    assert_eq!(v.preview().photos, 20);
    t.apply_import(&v, ImportMode::Replace).unwrap();
    drop(t);
    let t = Store::open(&b).unwrap();
    // Dữ liệu giống hệt (settings.last_backup_at trong tệp = giá trị trước khi xuất).
    assert_eq!(t.state_hash().unwrap(), h1);
    for (id, bytes) in &photos {
        assert_eq!(&t.photo_bytes(id).unwrap(), bytes);
    }
}

fn apply_merge(local: &Value, incoming: &Value) -> Value {
    let photo = read(gen::EXAMPLE_PHOTO);
    let pid = local["photos"][0]["id"].as_str().unwrap().to_owned();
    let mut photos = vec![(pid, photo.clone())];
    let dir = tmpdir("e");
    let mut s = Store::open(&dir).unwrap();
    seed(&mut s, local, &photos);
    for ph in incoming["photos"].as_array().unwrap() {
        let id = ph["id"].as_str().unwrap().to_owned();
        if !photos.iter().any(|x| x.0 == id) {
            photos.push((id, photo.clone()));
        }
    }
    let blobs: Vec<PhotoBlob> = photos
        .iter()
        .filter(|(id, _)| {
            incoming["photos"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["id"] == json!(id))
        })
        .map(|(id, b)| PhotoBlob {
            id: lvhv_backup::uuid_from_str(id).unwrap(),
            bytes: b.clone(),
        })
        .collect();
    let v = lvhv_store::verify_contents(&lvhv_backup::Contents {
        json: canonical_json(incoming),
        photos: blobs,
    })
    .unwrap();
    s.apply_import(&v, ImportMode::Merge).unwrap();
    let out = s
        .snapshot(
            &AppInfo {
                version: "0.0.0".into(),
                platform: "pwa".into(),
            },
            "1970-01-01T00:00:00Z",
        )
        .unwrap();
    validate::document(&out).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    out
}

fn find<'a>(doc: &'a Value, coll: &str, id: &Value) -> Option<&'a Value> {
    doc[coll]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| &x["id"] == id)
}

#[test]
fn merge_cases_31() {
    let cases: Vec<Value> = serde_json::from_slice(&read("merge-cases.json")).unwrap();
    assert_eq!(cases.len(), 31);
    for c in &cases {
        let name = c["name"].as_str().unwrap();
        let out = apply_merge(&c["local"], &c["incoming"]);
        // Hàm thuần cho cùng kết quả với đường qua SQLite.
        let pure = lvhv_store::merge_docs(&c["local"], &c["incoming"]);
        assert_eq!(
            sha256_hex(canonical_json(&pure).as_bytes()),
            c["merged_sha256"].as_str().unwrap(),
            "{name}"
        );
        for k in ["persons", "events", "event_types", "photos"] {
            assert_eq!(
                canonical_json(&out[k]),
                canonical_json(&pure[k]),
                "{name}: {k}"
            );
        }
        let e = &c["expect"];
        for w in e["winner"].as_array().into_iter().flatten() {
            let coll = w["collection"].as_str().unwrap();
            let from = if w["from"] == "incoming" {
                &c["incoming"]
            } else {
                &c["local"]
            };
            assert_eq!(
                find(&out, coll, &w["id"]),
                find(from, coll, &w["id"]),
                "{name}"
            );
        }
        for id in e["type_ids_absent"].as_array().into_iter().flatten() {
            assert!(find(&out, "event_types", id).is_none(), "{name}");
        }
        for id in e["type_ids_present"].as_array().into_iter().flatten() {
            assert!(find(&out, "event_types", id).is_some(), "{name}");
        }
        for x in e["event_type_id"].as_array().into_iter().flatten() {
            assert_eq!(
                find(&out, "events", &x["event"]).unwrap()["type_id"],
                x["type_id"],
                "{name}"
            );
        }
        if let Some(n) = e["photo_count"].as_u64() {
            assert_eq!(out["photos"].as_array().unwrap().len() as u64, n, "{name}");
        }
        if let Some(n) = e["person_count"].as_u64() {
            assert_eq!(out["persons"].as_array().unwrap().len() as u64, n, "{name}");
        }
        for id in e["photo_ids_present"].as_array().into_iter().flatten() {
            assert!(find(&out, "photos", id).is_some(), "{name}");
        }
        for id in e["photo_ids_absent"].as_array().into_iter().flatten() {
            assert!(find(&out, "photos", id).is_none(), "{name}");
        }
    }
}

#[test]
fn validation_cases_match() {
    let cases: Vec<Value> = serde_json::from_slice(&read("validation-cases.json")).unwrap();
    let base: Value = serde_json::from_slice(&read("example-v1.json")).unwrap();
    for c in &cases {
        let text = match c["raw"].as_str() {
            Some(t) => t.to_owned(),
            None => canonical_json(&gen::apply_patch(
                &base,
                c["pointer"].as_str().unwrap(),
                &c["value"],
            )),
        };
        let r = validate::parse_json(&text).and_then(|v| validate::document(&v));
        let (code, path) = match r {
            Ok(()) => ("OK".to_owned(), String::new()),
            Err(e) => (e.code.to_owned(), e.path),
        };
        assert_eq!(
            (code.as_str(), path.as_str()),
            (c["code"].as_str().unwrap(), c["path"].as_str().unwrap()),
            "{}",
            c["name"]
        );
    }
}

/// S12: tệp `m_cost_kib` = 4 GiB bị từ chối trước Argon2 (không cấp bộ nhớ); JSON lồng 10 000 cấp; 10 000 ảnh rỗng.
#[test]
fn s12_malicious_files() {
    let e = lvhv_backup::parse_header(&read("corrupt/m_cost_kib-4GiB.lvhv")).unwrap_err();
    assert_eq!(e, BackupError::BadKdfParams);
    // 10 000 ảnh rỗng trong vùng chứa hợp lệ (mật khẩu đúng) → bị từ chối ở kiểm tra ảnh, không treo.
    let photos: Vec<PhotoBlob> = (0..10_000u32)
        .map(|i| {
            let mut id = [0u8; 16];
            id[..4].copy_from_slice(&i.to_le_bytes());
            PhotoBlob {
                id,
                bytes: Vec::new(),
            }
        })
        .collect();
    let ex = String::from_utf8(read("example-v1.json")).unwrap();
    let f = lvhv_backup::export_with_fixed_rng(gen::PASSWORD, &ex, &photos, gen::SALT, gen::NONCE)
        .unwrap();
    assert!(matches!(
        open_backup(&f, gen::PASSWORD),
        Err(StoreError::Invalid(_))
    ));
    let deep = format!("{}{}", "[".repeat(10_000), "]".repeat(10_000));
    let f = lvhv_backup::export_with_fixed_rng(gen::PASSWORD, &deep, &[], gen::SALT, gen::NONCE)
        .unwrap();
    match open_backup(&f, gen::PASSWORD) {
        Err(StoreError::Invalid(e)) => assert_eq!(e.code, "JSON"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn newer_schema_rejected() {
    let dir = tmpdir("newer");
    {
        let s = Store::open(&dir).unwrap();
        drop(s);
        let c = rusqlite::Connection::open(dir.join(lvhv_store::DB_FILE)).unwrap();
        c.pragma_update(None, "user_version", lvhv_store::migrate::STORE_VERSION + 1)
            .unwrap();
    }
    assert!(matches!(
        Store::open(&dir),
        Err(StoreError::NewerSchema(v)) if v == lvhv_store::migrate::STORE_VERSION + 1
    ));
}

#[test]
fn backup_reminder_hook() {
    assert!(!lvhv_store::backup_due(30, None, "2026-12-01T00:00:00Z"));
    assert!(!lvhv_store::backup_due(
        0,
        Some("2026-01-01T00:00:00Z"),
        "2026-12-01T00:00:00Z"
    ));
    assert!(!lvhv_store::backup_due(
        30,
        Some("2026-11-01T00:00:01Z"),
        "2026-12-01T00:00:00Z"
    ));
    assert!(lvhv_store::backup_due(
        30,
        Some("2026-11-01T00:00:00Z"),
        "2026-12-01T00:00:00Z"
    ));
    assert_eq!(
        lvhv_store::rfc3339_from_unix(
            lvhv_store::unix_from_rfc3339("2024-02-29T23:59:59Z").unwrap()
        ),
        "2024-02-29T23:59:59Z"
    );
    let dir = tmpdir("nag");
    let mut s = Store::open(&dir).unwrap();
    assert_eq!(s.backup_pending_since().unwrap(), None);
    let ex: Value = serde_json::from_slice(&read("example-v1.json")).unwrap();
    s.put_event_type(&json!({"id": "a1b2c3d4-0000-4000-8000-000000000001", "builtin": null, "ten": "Lễ thử", "default_recurring": false,
        "created_at": "2026-11-06T03:15:00Z", "updated_at": "2026-11-06T03:15:00Z", "rev": 1})).unwrap();
    let since = s.backup_pending_since().unwrap().unwrap();
    let now = lvhv_store::now_rfc3339();
    assert!(!s.backup_reminder_due(&now).unwrap());
    let later =
        lvhv_store::rfc3339_from_unix(lvhv_store::unix_from_rfc3339(&since).unwrap() + 30 * 86_400);
    assert!(s.backup_reminder_due(&later).unwrap());
    let _ = s.export_backup("mật khẩu thử nghiệm", &app()).unwrap();
    // Chưa lưu tệp (hủy hộp "Lưu") → vẫn nhắc sao lưu (MA3N-23-01).
    assert!(s.backup_reminder_due(&later).unwrap());
    assert!(s.settings().unwrap()["last_backup_at"].is_null());
    s.mark_backup_saved(&lvhv_store::now_rfc3339()).unwrap();
    assert!(!s.backup_reminder_due(&later).unwrap());
    // Thay đổi sau khi tạo tệp nhưng trước khi ghi nhận vẫn là "chờ sao lưu".
    let _ = s.export_backup("mật khẩu thử nghiệm", &app()).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    s.put_event_type(&json!({"id": "a1b2c3d4-0000-4000-8000-000000000002", "builtin": null, "ten": "Lễ thử hai", "default_recurring": false,
        "created_at": "2026-11-06T03:15:00Z", "updated_at": "2026-11-06T03:15:00Z", "rev": 1})).unwrap();
    s.mark_backup_saved(&lvhv_store::now_rfc3339()).unwrap();
    assert!(s.backup_pending_since().unwrap().is_some());
    assert!(s.mark_backup_saved("không phải thời điểm").is_err());
    let _ = ex;
}
