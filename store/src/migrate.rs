//! Chuyển đổi phiên bản dữ liệu (01 §7, 04 Bổ sung B5 / R14) — hàm **thuần** trên tài liệu dạng sao lưu (01 §5),
//! không I/O. Bản TypeScript `pwa/src/store/migrate.ts` giống hệt (kiểm bằng bộ mẫu đóng băng
//! `docs/v0.2/schema/test-vectors/released/`).
//!
//! Hai số phiên bản:
//! - `schema_version` của **tài liệu sao lưu** (lược đồ backup-v1) — vẫn là 1. Khóa mới của v0.2 (`gioi_tinh`,
//!   `attachments`, `legacy`) là khóa tùy chọn, nên tài liệu cũ hợp lệ và vector đã khóa của 03 §2.5 không đổi.
//! - [`STORE_VERSION`] của **kho trên máy** (SQLite `user_version`, phiên bản IndexedDB): 1 = bản dựng 0.2 trước
//!   YC-2026-07 (`death.notice.certificate_photo_ids`, chưa có bảng tệp đính kèm), 2 = có `attachments`.
//!
//! Quy tắc B5: không phép chuyển nào xóa dữ liệu. Trường cũ được chuyển sang trường mới; bản gốc được giữ trong khóa
//! `legacy` của hồ sơ (đối tượng tự do, không hiển thị, có trong sao lưu).
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};

/// Phiên bản kho trên máy mà bản này hiểu.
pub const STORE_VERSION: i64 = 2;

/// Kết quả một bước chuyển đổi.
#[derive(Debug, Clone, PartialEq)]
pub struct Migrated {
    /// Tài liệu sau khi chuyển.
    pub doc: Value,
    /// Byte cho từng `Attachment` mới: (id tệp đính kèm, id ảnh nguồn) — người gọi chép byte ảnh sang.
    pub blob_sources: Vec<(String, String)>,
    /// Id ảnh đã **chuyển hẳn** sang tệp đính kèm (bản ghi `Photo` không còn; byte giữ ở tệp đính kèm cùng id).
    pub moved_photos: Vec<String>,
    /// Có thay đổi gì không.
    pub changed: bool,
}

/// Tài liệu còn khóa cũ `death.notice.certificate_photo_ids` (bản dựng 0.2 trước YC-2026-07)?
pub fn has_certificate_photo_ids(doc: &Value) -> bool {
    doc["persons"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|p| p["death"]["notice"].get("certificate_photo_ids").is_some())
}

/// Id thay thế (UUID v4 hợp lệ, tất định) cho tệp đính kèm khi ảnh nguồn còn được dùng làm ảnh chân dung: đảo
/// 12 chữ số hex cuối (`x` → `f − x`); trùng id đã có thì tăng chữ số cuối (mod 16) cho tới khi không trùng.
pub fn alt_id(id: &str, taken: &HashSet<String>) -> String {
    let flip = |c: char| {
        let v = c.to_digit(16).unwrap_or(0);
        char::from_digit(15 - v, 16).unwrap_or('0')
    };
    let (head, node) = id.split_at(id.len().saturating_sub(12));
    let mut cand: Vec<char> = head.chars().chain(node.chars().map(flip)).collect();
    loop {
        let s: String = cand.iter().collect();
        if !taken.contains(&s) {
            return s;
        }
        let last = cand.len() - 1;
        let v = cand[last].to_digit(16).unwrap_or(0);
        cand[last] = char::from_digit((v + 1) % 16, 16).unwrap_or('0');
    }
}

/// 1 → 2 (YC-2026-07): `death.notice.certificate_photo_ids` → `death.notice.attachments` loại `giay_chung_tu`.
///
/// Với mỗi id ảnh trong danh sách (theo thứ tự hồ sơ rồi thứ tự danh sách):
/// - ảnh có bản ghi `Photo`: tạo `Attachment` {mime, bytes, sha256 của ảnh; `stripped = true`; `file_name = null`;
///   `created_at` của ảnh}. Id tệp = id ảnh, và bản ghi `Photo` được **chuyển** sang (bỏ khỏi `photos`), trừ khi ảnh
///   còn là ảnh chân dung của một hồ sơ — khi đó giữ `Photo`, tệp đính kèm lấy id [`alt_id`]. Một ảnh dùng ở nhiều hồ
///   sơ → một tệp đính kèm dùng chung. Phần tử danh sách: `{id: <id ảnh>, kind: "giay_chung_tu", file_id}`.
/// - id không có ảnh: không chuyển được, chỉ giữ trong `legacy`.
///
/// Hồ sơ có chuyển đổi nhận `legacy.certificate_photo_ids` (danh sách gốc) và `legacy.certificate_photos` (bản ghi
/// `Photo` gốc của các ảnh đã chuyển) — không mất trường nào (`width`, `height` chỉ có ở đây).
pub fn migrate_1_to_2(doc: &Value) -> Migrated {
    let mut doc = doc.clone();
    let photos: BTreeMap<String, Value> = doc["photos"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| p["id"].as_str().map(|id| (id.to_owned(), p.clone())))
        .collect();
    let portrait: HashSet<String> = doc["persons"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| p["photo_id"].as_str().map(str::to_owned))
        .collect();
    let mut taken: HashSet<String> = photos.keys().cloned().collect();
    let mut atts: Vec<Value> = doc["attachments"].as_array().cloned().unwrap_or_default();
    taken.extend(
        atts.iter()
            .filter_map(|a| a["id"].as_str().map(str::to_owned)),
    );
    let mut memo: BTreeMap<String, String> = BTreeMap::new();
    let mut out = Migrated {
        doc: Value::Null,
        blob_sources: Vec::new(),
        moved_photos: Vec::new(),
        changed: false,
    };
    if let Some(persons) = doc["persons"].as_array_mut() {
        for p in persons.iter_mut() {
            // `get_mut`, không dùng `p["death"]["notice"]`: chỉ mục ghi trên `null` sẽ tạo đối tượng mới.
            let Some(n) = p
                .get_mut("death")
                .and_then(|d| d.get_mut("notice"))
                .and_then(Value::as_object_mut)
            else {
                continue;
            };
            let Some(old) = n.remove("certificate_photo_ids") else {
                continue;
            };
            out.changed = true;
            let mut items: Vec<Value> = n
                .get("attachments")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let mut originals: Vec<Value> = Vec::new();
            for pid in old
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                let Some(ph) = photos.get(pid) else {
                    continue; // không có ảnh → chỉ còn trong legacy
                };
                let aid = match memo.get(pid) {
                    Some(a) => a.clone(),
                    None => {
                        let aid = if portrait.contains(pid) {
                            alt_id(pid, &taken)
                        } else {
                            pid.to_owned()
                        };
                        taken.insert(aid.clone());
                        atts.push(json!({
                            "id": aid, "mime": ph["mime"], "bytes": ph["bytes"], "sha256": ph["sha256"],
                            "stripped": true, "file_name": null, "created_at": ph["created_at"],
                        }));
                        out.blob_sources.push((aid.clone(), pid.to_owned()));
                        if aid == pid {
                            out.moved_photos.push(pid.to_owned());
                        }
                        memo.insert(pid.to_owned(), aid.clone());
                        aid
                    }
                };
                originals.push(ph.clone());
                items.push(json!({"id": pid, "kind": "giay_chung_tu", "file_id": aid}));
            }
            n.insert("attachments".into(), Value::Array(items));
            let legacy = p
                .as_object_mut()
                .map(|o| o.entry("legacy").or_insert_with(|| json!({})));
            if let Some(Value::Object(l)) = legacy {
                l.insert("certificate_photo_ids".into(), old);
                l.insert("certificate_photos".into(), Value::Array(originals));
            }
        }
    }
    if !out.changed {
        out.doc = doc;
        return out;
    }
    let moved: HashSet<&String> = out.moved_photos.iter().collect();
    if let Some(a) = doc["photos"].as_array_mut() {
        a.retain(|p| {
            !p["id"]
                .as_str()
                .is_some_and(|id| moved.contains(&id.to_owned()))
        });
    }
    atts.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    if let Some(o) = doc.as_object_mut() {
        if atts.is_empty() {
            o.remove("attachments");
        } else {
            o.insert("attachments".into(), Value::Array(atts));
        }
    }
    out.doc = doc;
    out
}

/// Trạng thái chuyển đổi khi mở kho (04 B5) — để giao diện hiện "chỉ đọc" / lỗi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationStatus {
    /// `"ok"` (không cần chuyển), `"migrated"` (đã chuyển xong), `"read_only"` (chuyển lỗi: dữ liệu cũ giữ nguyên,
    /// chỉ đọc — hãy xuất sao lưu).
    pub state: &'static str,
    /// Phiên bản kho trước khi mở.
    pub from_version: i64,
    /// Phiên bản kho sau khi mở (bằng `from_version` khi chỉ đọc).
    pub to_version: i64,
    /// Đường dẫn bản sao dữ liệu cũ (nếu đã tạo).
    pub backup_path: Option<String>,
    /// Lỗi chuyển đổi (khi chỉ đọc).
    pub error: Option<String>,
}

/// Bản ghi `legacy` hợp lệ: đối tượng JSON (nội dung tự do).
pub fn legacy_ok(v: &Value) -> bool {
    v.is_object()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alt_id_is_v4_and_avoids_taken() {
        let id = "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e0c";
        let mut taken = HashSet::new();
        let a = alt_id(id, &taken);
        assert_eq!(a, "0b8e4f52-3c1a-4d2e-9f60-e5d4c3b2a1f3");
        taken.insert(a.clone());
        assert_eq!(alt_id(id, &taken), "0b8e4f52-3c1a-4d2e-9f60-e5d4c3b2a1f4");
        assert!(crate::validate::is_uuid_v4(&a));
    }

    #[test]
    fn shared_portrait_missing_and_noop() {
        let doc = json!({
            "photos": [
                {"id": "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01", "mime": "image/jpeg", "width": 10, "height": 20, "bytes": 5,
                 "sha256": "a".repeat(64), "stripped": true, "created_at": "2026-01-01T00:00:00Z"},
                {"id": "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02", "mime": "image/webp", "width": 10, "height": 20, "bytes": 6,
                 "sha256": "b".repeat(64), "stripped": true, "created_at": "2026-01-02T00:00:00Z"}
            ],
            "persons": [
                {"id": "p1", "photo_id": "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01", "death": {"notice": {"certificate_photo_ids": [
                    "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e01", "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02",
                    "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5eff"]}}},
                {"id": "p2", "photo_id": null, "death": {"notice": {"certificate_photo_ids": ["0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02"]}}},
                {"id": "p3", "photo_id": null, "death": null}
            ]
        });
        let m = migrate_1_to_2(&doc);
        assert!(m.changed);
        let d = &m.doc;
        // Ảnh 01 là ảnh chân dung → giữ Photo, tệp đính kèm id thay thế; ảnh 02 chuyển hẳn, dùng chung cho p1 và p2.
        assert_eq!(d["photos"].as_array().unwrap().len(), 1);
        assert_eq!(d["attachments"].as_array().unwrap().len(), 2);
        let n1 = &d["persons"][0]["death"]["notice"];
        assert!(n1.get("certificate_photo_ids").is_none());
        assert_eq!(n1["attachments"].as_array().unwrap().len(), 2);
        assert_eq!(
            n1["attachments"][1]["file_id"],
            "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02"
        );
        assert_eq!(
            d["persons"][1]["death"]["notice"]["attachments"][0]["file_id"],
            "0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02"
        );
        // Id thiếu ảnh và bản ghi gốc nằm trong legacy.
        assert_eq!(
            d["persons"][0]["legacy"]["certificate_photo_ids"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
        assert_eq!(
            d["persons"][0]["legacy"]["certificate_photos"][0]["width"],
            10
        );
        assert_eq!(
            m.moved_photos,
            vec!["0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02".to_string()]
        );
        assert_eq!(m.blob_sources.len(), 2);
        // Chạy lại: không còn gì để chuyển.
        let again = migrate_1_to_2(&m.doc);
        assert!(!again.changed);
        assert_eq!(again.doc, m.doc);
    }
}
