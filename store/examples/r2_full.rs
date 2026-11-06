//! R2 (docs/v0.2/04) phía Rust/SQLite, toàn dải:
//!   (a) mọi ngày dương 1900-01-01 … 3000-12-31 (402 132 ngày) → `save_death` của lớp lưu trữ → ghi vào SQLite
//!       (đường ghi thật: kiểm tra V1–V8 rồi giao dịch) → đóng, mở lại kho → đọc lại: giống hệt từng trường;
//!       `death.lunar = S2L(input)`, `L2S(death.lunar) = input`, `lunar_source = "converted"`, `method = "precise"`.
//!   (b) mọi ngày âm tồn tại trong năm âm 1900–2999 (gồm tháng nhuận, ngày 30), nhập theo âm:
//!       `death.solar = L2S(input)`, `S2L(death.solar) = input`, `lunar_source = "entered"`; ghi/đọc lại như (a).
//!   (c) ngày dương không tồn tại (30/02, 31/04 …) và ngày âm không tồn tại (tháng nhuận không có, 30 tháng thiếu)
//!       bị từ chối với thông báo 01 §6.1, kho không đổi.
//! In băm SHA-256 của chuỗi JSON chuẩn các `death` (một dòng mỗi ngày) để so với bản TypeScript
//! (`pwa/test/store/r2-derive.mjs`, Rust ≡ TypeScript).
//!
//!     cargo run --release -p lvhv-store --example r2_full
use lvhv_core::{lunar_to_solar, solar_to_lunar};
use lvhv_store::{build_person, canonical_json, AppInfo, ImportMode, Store};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const TS: &str = "2026-11-06T03:15:00Z";
const BATCH: usize = 10_000; // lược đồ: persons.maxItems = 10 000

fn uid(n: u64) -> String {
    let h = format!("{n:032x}");
    format!(
        "{}-{}-4{}-8{}-{}",
        &h[0..8],
        &h[8..12],
        &h[13..16],
        &h[17..20],
        &h[20..32]
    )
}

fn person(n: u64, input: Value) -> Result<Value, String> {
    let base = json!({
        "id": uid(n), "kind": "deceased", "danh_xung": "", "ho_ten": "R2", "ten_hieu": "", "quan_he": "", "birth": null,
        "death": {"input": input, "tuoi": {"manual": false}}, "noi_an_tang": "", "photo_id": null,
        "memorial": {"tray_style": "truyen_thong", "incense": true, "bia_tieu_de": "", "loi_tuong_niem": "", "gio_het_nam": 2, "huong_duong_duoi": 60},
        "reminders": {"gio": null, "sinh_nhat": null, "sinh_nhat_am": null}, "sinh_nhat_am": false, "feb29_rule": "feb28",
        "ghi_chu": "", "created_at": TS, "updated_at": TS, "rev": 1,
    });
    build_person(&base).map_err(|e| e.to_string())
}

fn days_in_month(y: i32, m: u32) -> u32 {
    lvhv_store::validate::days_in_month(i64::from(y), i64::from(m)) as u32
}

struct Run {
    dir: std::path::PathBuf,
    batch: Vec<Value>,
    hasher: Sha256,
    n: u64,
    errors: u64,
}

impl Run {
    fn flush(&mut self) {
        if self.batch.is_empty() {
            return;
        }
        let _ = std::fs::remove_dir_all(&self.dir);
        let mut doc = {
            let s = Store::open(&self.dir).unwrap();
            s.snapshot(
                &AppInfo {
                    version: "0.2.0".into(),
                    platform: "windows".into(),
                },
                TS,
            )
            .unwrap()
        };
        doc["persons"] = Value::Array(std::mem::take(&mut self.batch));
        let want: Vec<Value> = doc["persons"].as_array().unwrap().clone();
        {
            let mut s = Store::open(&self.dir).unwrap();
            let v = lvhv_store::verify_contents(&lvhv_backup::Contents {
                json: canonical_json(&doc),
                photos: vec![],
            })
            .unwrap();
            s.apply_import(&v, ImportMode::Replace).unwrap();
        }
        let s = Store::open(&self.dir).unwrap(); // đóng rồi mở lại
        let got = s.persons().unwrap();
        let mut want = want;
        want.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        if got != want {
            self.errors += 1;
            eprintln!("LỖI: đọc lại khác ghi (lô kết thúc ở {})", self.n);
        }
    }

    fn push(&mut self, p: Value) {
        self.hasher.update(canonical_json(&p["death"]).as_bytes());
        self.hasher.update(b"\n");
        self.batch.push(p);
        self.n += 1;
        if self.batch.len() >= BATCH {
            self.flush();
        }
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn main() {
    let t0 = std::time::Instant::now();
    let dir = std::env::temp_dir().join(format!("lvhv-r2-{}", std::process::id()));
    let mut run = Run {
        dir: dir.clone(),
        batch: Vec::new(),
        hasher: Sha256::new(),
        n: 0,
        errors: 0,
    };
    // (a)
    let mut na = 0u64;
    for y in 1900..=3000 {
        for m in 1..=12 {
            for d in 1..=days_in_month(y, m) {
                let input = json!({"calendar": "solar", "precision": "day", "year": y, "month": m, "day": d, "leap": false});
                let p = person(run.n, input).unwrap();
                let de = &p["death"];
                let l = solar_to_lunar(y, m, d).unwrap();
                let ok = de["lunar"]
                    == json!({"year": l.year, "month": l.month, "day": l.day, "leap": l.leap})
                    && lunar_to_solar(l.year, l.month, l.day, l.leap)
                        .map(|s| (s.year, s.month, s.day))
                        == Some((y, m, d))
                    && de["solar"] == json!({"y": y, "m": m, "d": d})
                    && de["lunar_source"] == "converted"
                    && de["method"] == "precise";
                if !ok {
                    run.errors += 1;
                    eprintln!("LỖI (a) {y}-{m}-{d}");
                }
                run.push(p);
                na += 1;
            }
        }
    }
    run.flush();
    let ha = hex(&std::mem::take(&mut run.hasher).finalize());
    // (b)
    let mut nb = 0u64;
    for y in 1900..=2999 {
        for m in 1..=12u32 {
            for leap in [false, true] {
                for d in 1..=30u32 {
                    let Some(s) = lunar_to_solar(y, m, d, leap) else {
                        continue;
                    };
                    let input = json!({"calendar": "lunar", "precision": "day", "year": y, "month": m, "day": d, "leap": leap});
                    let p = person(run.n, input).unwrap();
                    let de = &p["death"];
                    let back = solar_to_lunar(s.year, s.month, s.day).unwrap();
                    let ok = de["solar"] == json!({"y": s.year, "m": s.month, "d": s.day})
                        && (back.year, back.month, back.day, back.leap) == (y, m, d, leap)
                        && de["lunar"] == json!({"year": y, "month": m, "day": d, "leap": leap})
                        && de["lunar_source"] == "entered";
                    if !ok {
                        run.errors += 1;
                        eprintln!("LỖI (b) {y}-{m}{}-{d}", if leap { "n" } else { "" });
                    }
                    run.push(p);
                    nb += 1;
                }
            }
        }
    }
    run.flush();
    let hb = hex(&std::mem::take(&mut run.hasher).finalize());
    // (c)
    let mut nc = 0u64;
    let mut rejected = 0u64;
    let mut msgs = std::collections::BTreeMap::<String, u64>::new();
    let probe_dir = dir.join("probe");
    let mut probe = Store::open(&probe_dir).unwrap();
    let before = probe.state_hash().unwrap();
    let mut check = |input: Value, want_prefix: &str| {
        nc += 1;
        match person(1, input.clone()) {
            Err(m) if m.starts_with(want_prefix) => {
                rejected += 1;
                let kind = if want_prefix.starts_with("Năm ") {
                    "Năm … không có tháng … nhuận"
                } else {
                    "Tháng … chỉ có … ngày"
                };
                *msgs.entry(kind.to_owned()).or_default() += 1;
            }
            other => eprintln!("LỖI (c) {input}: {other:?}"),
        }
        // Đường ghi của kho cũng từ chối bản ghi tự dựng với trường dẫn xuất bất kỳ.
        let mut p = person(1, json!({"calendar": "solar", "precision": "day", "year": 2000, "month": 1, "day": 1, "leap": false})).unwrap();
        p["death"]["input"] = input;
        if probe.put_person(&p).is_ok() {
            eprintln!("LỖI (c) kho nhận bản ghi ngày không tồn tại");
        }
    };
    for y in 1900..=3000 {
        for m in 1..=12 {
            for d in days_in_month(y, m) + 1..=31 {
                check(
                    json!({"calendar": "solar", "precision": "day", "year": y, "month": m, "day": d, "leap": false}),
                    "Tháng ",
                );
            }
        }
    }
    for y in 1900..=2999 {
        for m in 1..=12u32 {
            if lunar_to_solar(y, m, 1, true).is_none() {
                check(
                    json!({"calendar": "lunar", "precision": "day", "year": y, "month": m, "day": 1, "leap": true}),
                    &format!("Năm {y} không có tháng {m} nhuận"),
                );
            }
            for leap in [false, true] {
                if lunar_to_solar(y, m, 1, leap).is_some()
                    && lunar_to_solar(y, m, 30, leap).is_none()
                {
                    check(
                        json!({"calendar": "lunar", "precision": "day", "year": y, "month": m, "day": 30, "leap": leap}),
                        "Tháng ",
                    );
                }
            }
        }
    }
    let unchanged = probe.state_hash().unwrap() == before;
    drop(probe);
    let _ = std::fs::remove_dir_all(&dir);
    let res = json!({
        "a_solar_days": na, "a_sha256": ha, "b_lunar_days": nb, "b_sha256": hb,
        "c_invalid": nc, "c_rejected": rejected, "c_messages": msgs, "c_store_unchanged": unchanged,
        "errors": run.errors, "seconds": t0.elapsed().as_secs(),
    });
    println!("{}", serde_json::to_string_pretty(&res).unwrap());
    if run.errors > 0 || rejected != nc || !unchanged || na != 402_132 {
        std::process::exit(1);
    }
}
