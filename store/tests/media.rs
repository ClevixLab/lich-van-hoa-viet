//! 04 Bổ sung B6 (BM2-01, BM2-08; S9, S12): kiểm tra chặt ảnh và tệp đính kèm. Bộ tệp và mã kỳ vọng ở
//! `docs/v0.2/schema/test-vectors/media/` — bản PWA (`pwa/test/store/run.mjs`) kiểm cùng manifest, cùng mã.
use lvhv_store::media::{check_attachment, check_photo, MediaError, MAX_PDF};
use lvhv_store::{canonical_json, sha256_hex, verify_contents, ImportMode, Store};
use serde_json::{json, Value};
use std::path::PathBuf;

fn tv() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/v0.2/schema/test-vectors")
}

fn verdict(r: Result<(), MediaError>) -> &'static str {
    match r {
        Ok(()) => "ok",
        Err(e) => e.code(),
    }
}

#[test]
fn media_manifest_cases() {
    let d = tv().join("media");
    let m: Value =
        serde_json::from_slice(&std::fs::read(d.join("manifest.json")).unwrap()).unwrap();
    for (f, h) in m["files"].as_object().unwrap() {
        assert_eq!(
            sha256_hex(&std::fs::read(d.join(f)).unwrap()),
            h.as_str().unwrap(),
            "{f}"
        );
    }
    let cases = m["cases"].as_array().unwrap();
    assert!(cases.len() >= 20);
    for c in cases {
        let b = std::fs::read(d.join(c["file"].as_str().unwrap())).unwrap();
        let mime = c["mime"].as_str().unwrap();
        let got = if c["kind"] == "photo" {
            verdict(check_photo(
                mime,
                &b,
                c["width"].as_u64().unwrap() as u32,
                c["height"].as_u64().unwrap() as u32,
            ))
        } else {
            verdict(check_attachment(mime, &b))
        };
        assert_eq!(got, c["expect"].as_str().unwrap(), "{c}");
    }
    // PDF quá 10 MiB (không lưu thành tệp mẫu).
    let mut big = b"%PDF-1.4\n".to_vec();
    big.resize(MAX_PDF + 1, b' ');
    big.extend_from_slice(b"%%EOF\n");
    assert_eq!(
        verdict(check_attachment("application/pdf", &big)),
        "too_large"
    );
}

/// Tài liệu của vector bổ sung YC-05/07 + khối: (doc, [(id, byte)]) theo thứ tự tệp.
fn supplement() -> (Value, Vec<(String, Vec<u8>)>) {
    let d = tv().join("supplement-yc05");
    let doc: Value =
        serde_json::from_slice(&std::fs::read(d.join("notice-v1.json")).unwrap()).unwrap();
    let photo = std::fs::read(tv().join("photo-0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e09.jpg")).unwrap();
    let pdf = std::fs::read(d.join("giay-chung-tu.pdf")).unwrap();
    let mut blobs: Vec<(String, Vec<u8>)> = doc["photos"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p["id"].as_str().unwrap().to_owned(), photo.clone()))
        .collect();
    for a in doc["attachments"].as_array().unwrap() {
        let b = if a["mime"] == "application/pdf" {
            pdf.clone()
        } else {
            photo.clone()
        };
        blobs.push((a["id"].as_str().unwrap().to_owned(), b));
    }
    (doc, blobs)
}

fn contents(doc: &Value, blobs: &[(String, Vec<u8>)]) -> lvhv_backup::Contents {
    lvhv_backup::Contents {
        json: canonical_json(doc),
        photos: blobs
            .iter()
            .map(|(id, b)| lvhv_backup::PhotoBlob {
                id: lvhv_backup::uuid_from_str(id).unwrap(),
                bytes: b.clone(),
            })
            .collect(),
    }
}

/// Thay byte của tệp đính kèm `i` (cập nhật `bytes`/`sha256` của bản ghi cho khớp — như kẻ gửi làm).
fn with_attachment(i: usize, mime: &str, bytes: &[u8]) -> (Value, Vec<(String, Vec<u8>)>) {
    let (mut doc, mut blobs) = supplement();
    let id = doc["attachments"][i]["id"].as_str().unwrap().to_owned();
    doc["attachments"][i]["mime"] = json!(mime);
    doc["attachments"][i]["bytes"] = json!(bytes.len());
    doc["attachments"][i]["sha256"] = json!(sha256_hex(bytes));
    doc["attachments"][i]["stripped"] = json!(mime != "application/pdf");
    for b in blobs.iter_mut() {
        if b.0 == id {
            b.1 = bytes.to_vec();
        }
    }
    (doc, blobs)
}

/// S12 / BM2-01: tệp sao lưu (mật khẩu của kẻ gửi, AEAD qua) mang tệp đính kèm độc → từ chối toàn bộ, thông báo
/// chung, kho không đổi.
#[test]
fn malicious_attachments_rejected_whole_backup() {
    let md = tv().join("media");
    let (doc, blobs) = supplement();
    assert!(
        verify_contents(&contents(&doc, &blobs)).is_ok(),
        "đối chứng hợp lệ"
    );
    let pdf = std::fs::read(md.join("pdf-ok.pdf")).unwrap();
    let cases: Vec<(usize, &str, Vec<u8>)> = vec![
        // X21: khai image/jpeg nhưng byte là PDF.
        (0, "image/jpeg", pdf.clone()),
        (
            0,
            "image/jpeg",
            std::fs::read(md.join("png-20000x20000-bomb.png")).unwrap(),
        ),
        (
            0,
            "image/jpeg",
            std::fs::read(md.join("html-as-image.html")).unwrap(),
        ),
        (
            0,
            "image/jpeg",
            std::fs::read(md.join("jpeg-exif-gps.jpg")).unwrap(),
        ),
        (
            0,
            "image/webp",
            std::fs::read(md.join("webp-exif.webp")).unwrap(),
        ),
        (
            1,
            "application/pdf",
            std::fs::read(md.join("pdf-fake-no-eof.pdf")).unwrap(),
        ),
        (
            1,
            "application/pdf",
            std::fs::read(md.join("html-as-image.html")).unwrap(),
        ),
    ];
    for (i, mime, b) in cases {
        let (d, bl) = with_attachment(i, mime, &b);
        let e = verify_contents(&contents(&d, &bl)).unwrap_err();
        assert_eq!(
            e.import_message(),
            "Sai mật khẩu hoặc tệp bị hỏng",
            "{mime}"
        );
    }
    // Kho không đổi: nhập thử vào kho có dữ liệu → lỗi trước khi ghi.
    let dir = std::env::temp_dir().join(format!("lvhv-media-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut s = Store::open(&dir).unwrap();
    let good = verify_contents(&contents(&doc, &blobs)).unwrap();
    s.apply_import(&good, ImportMode::Replace).unwrap();
    let h = s.state_hash().unwrap();
    let (d, bl) = with_attachment(0, "image/jpeg", &pdf);
    assert!(verify_contents(&contents(&d, &bl)).is_err());
    assert_eq!(s.state_hash().unwrap(), h);
    // add_photo / add_attachment (BM2-08): ảnh còn EXIF, sai nhãn, kích thước khai sai → Input.
    let exif = std::fs::read(md.join("jpeg-exif-gps.jpg")).unwrap();
    let ok = std::fs::read(md.join("jpeg-ok.jpg")).unwrap();
    let id = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e7a";
    let msg = |r: lvhv_store::Result<Value>| match r {
        Err(lvhv_store::StoreError::Input(m)) => m,
        other => panic!("{other:?}"),
    };
    assert!(msg(s.add_photo(id, "image/jpeg", 768, 1024, &exif)).contains("bỏ siêu dữ liệu"));
    assert!(msg(s.add_photo(id, "image/jpeg", 64, 64, &ok)).contains("dimensions_mismatch"));
    assert!(msg(s.add_attachment(id, "image/jpeg", None, &pdf)).contains("mime_mismatch"));
    assert!(s.add_photo(id, "image/jpeg", 768, 1024, &ok).is_ok());
    let _ = std::fs::remove_dir_all(dir);
}

/// X22, M09 (báo cáo kiểm định B5): tài liệu có cả `attachments` lẫn `certificate_photo_ids`, hoặc
/// `certificate_photo_ids` trỏ tới ảnh không tồn tại → dị dạng, bị từ chối khi nhập.
#[test]
fn malformed_legacy_documents_rejected() {
    let d = tv().join("released/0.2-dev-before-attachments");
    let doc: Value =
        serde_json::from_slice(&std::fs::read(d.join("store-snapshot.json")).unwrap()).unwrap();
    let photo =
        std::fs::read(d.join("store/photos/0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e09.img")).unwrap();
    let blobs: Vec<(String, Vec<u8>)> = doc["photos"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p["id"].as_str().unwrap().to_owned(), photo.clone()))
        .collect();
    assert!(
        verify_contents(&contents(&doc, &blobs)).is_ok(),
        "đối chứng: tài liệu cũ hợp lệ được chuyển"
    );
    let mut x22 = doc.clone();
    x22["persons"][0]["death"]["notice"]["attachments"] = json!([]);
    let e = verify_contents(&contents(&x22, &blobs)).unwrap_err();
    assert!(
        matches!(e, lvhv_store::StoreError::Invalid(ref v) if v.code == "V1"),
        "{e:?}"
    );
    let mut m09 = doc.clone();
    m09["persons"][0]["death"]["notice"]["certificate_photo_ids"] =
        json!(["0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5eff"]);
    let e = verify_contents(&contents(&m09, &blobs)).unwrap_err();
    assert!(
        matches!(e, lvhv_store::StoreError::Invalid(ref v) if v.code == "V3" && v.path == "/persons/0/death/notice/certificate_photo_ids/0"),
        "{e:?}"
    );
}
