//! Các chế độ chỉ dùng trong CI, gọi qua `lvhv-ci-check.exe` (không có trong `LichViet.exe` phát hành, không đóng gói —
//! MA3C-25-01; CI khẳng định `LichViet.exe` không chứa các chuỗi cờ này):
//!
//! - `--chup-man-hinh <thư mục>`: chụp mọi màn hình (kho tạm, dữ liệu mẫu) rồi thoát.
//! - `--do-bo-nho`: mở Người thân + tưởng niệm với dữ liệu mẫu (kho tạm) rồi đứng yên để đo Working Set.
//! - `--kiem-thong-bao`: tự kiểm thông báo đặt lịch của Windows (kho tạm; gỡ thông báo thử khi xong).
//! - `--tao-du-lieu-mau <ảnh chụp.json>`: R14 — tạo dữ liệu mẫu trong thư mục dữ liệu (từ chối nếu đã có kho).
//! - `--kiem-nang-cap <ảnh chụp.json>`: R14 (a) — so dữ liệu sau khi cài đè với ảnh chụp, đặt lại thông báo.
//! - `--kiem-mau-dong-bang <expected.json>`: R14 (c) — kho của bộ mẫu đóng băng đã chuyển đúng tệp kỳ vọng.
//! - `--kiem-chi-doc <kho bộ mẫu>`: R14 ca lỗi — trên bản chép trong thư mục tạm.
//!
//! Biến môi trường (chỉ ở đây, bản phát hành không đọc): `LVHV_DATA_DIR` thay thư mục dữ liệu.

use crate::{platform, shots, toasts, upgrade, UiMode};

/// Mọi cờ CI — CI tìm từng chuỗi này trong `LichViet.exe` phát hành và báo lỗi nếu thấy.
pub const FLAGS: [&str; 7] = [
    "--chup-man-hinh",
    "--do-bo-nho",
    "--kiem-thong-bao",
    "--tao-du-lieu-mau",
    "--kiem-nang-cap",
    "--kiem-mau-dong-bang",
    "--kiem-chi-doc",
];

fn done(r: Result<String, String>, fail: i32) -> i32 {
    match r {
        Ok(msg) => {
            println!("{msg}");
            0
        }
        Err(msg) => {
            eprintln!("{msg}");
            fail
        }
    }
}

pub fn main() -> i32 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args.first().cloned().unwrap_or_default();
    let arg1 = std::path::PathBuf::from(args.get(1).cloned().unwrap_or_default());
    if let Some(d) = std::env::var_os("LVHV_DATA_DIR").filter(|d| !d.is_empty()) {
        platform::set_data_dir_override(Some(d.into()));
    }
    match mode.as_str() {
        "--kiem-thong-bao" => {
            platform::init_app_identity();
            let dir = std::env::temp_dir().join(format!("lvhv-kiem-thong-bao-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            let r = lvhv_ffi::LvhvStore::open(dir.to_string_lossy().into_owned())
                .map_err(|e| e.to_string())
                .and_then(|store| {
                    shots::seed_store(&store);
                    toasts::self_test(toasts::system().as_ref(), &store, platform::now_unix(), platform::device_zone())
                });
            let _ = std::fs::remove_dir_all(&dir);
            done(r, 5)
        }
        "--tao-du-lieu-mau" | "--kiem-nang-cap" | "--kiem-mau-dong-bang" | "--kiem-chi-doc" => {
            platform::init_app_identity();
            let dir = platform::data_dir();
            let r = match mode.as_str() {
                "--tao-du-lieu-mau" => upgrade::create(&dir, &arg1),
                "--kiem-nang-cap" => upgrade::check(&dir, &arg1, toasts::system().as_ref(), platform::now_unix()),
                "--kiem-mau-dong-bang" => {
                    upgrade::check_frozen(&dir, &arg1, toasts::system().as_ref(), platform::now_unix())
                }
                _ => upgrade::check_read_only(
                    &arg1,
                    &std::env::temp_dir().join(format!("lvhv-kiem-chi-doc-{}", std::process::id())),
                ),
            };
            done(r, 6)
        }
        "--chup-man-hinh" | "--do-bo-nho" => {
            // Kho tạm, không bao giờ đặt thông báo thật.
            let dir = std::env::temp_dir().join(format!("lvhv-demo-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            platform::set_data_dir_override(Some(dir));
            platform::set_demo_mode();
            let m = if mode == "--chup-man-hinh" { UiMode::Shots(arg1) } else { UiMode::Memory };
            match crate::run_ui(m, vec![]) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("{e}");
                    7
                }
            }
        }
        _ => {
            eprintln!("lvhv-ci-check: dùng một trong {}", FLAGS.join(", "));
            2
        }
    }
}
