// Biên dịch giao diện Slint; kiểm phiên bản với tệp VERSION; ghi ngày/số bản dựng và mã Microsoft
// Store vào biến môi trường lúc biên dịch; (chỉ khi dựng cho Windows) nhúng biểu tượng + thông tin
// phiên bản vào .exe.
use std::fs;

fn main() {
    // Kiểm thử giao diện (src/ui_tests.rs) tìm phần tử theo nhãn trợ năng qua ElementHandle, cần thông tin
    // gỡ lỗi trong mã Slint sinh ra. Chỉ bật cho bản dựng không tối ưu (debug/test); bản release giữ gọn.
    if std::env::var("PROFILE").as_deref() == Ok("debug") {
        std::env::set_var("SLINT_EMIT_DEBUG_INFO", "1");
    }
    slint_build::compile_with_config(
        "ui/app.slint",
        slint_build::CompilerConfiguration::new().with_style("fluent".into()),
    )
    .expect("biên dịch ui/app.slint");

    // Nguồn duy nhất của số phiên bản: tệp VERSION ở gốc kho. Cargo.toml không đọc được tệp ngoài nên
    // vẫn ghi số; nếu hai nơi khác nhau thì dừng dựng (CI đỏ).
    let version = fs::read_to_string("../../VERSION")
        .expect("không đọc được ../../VERSION")
        .trim()
        .to_string();
    let parts: Vec<&str> = version.split('.').collect();
    assert!(
        parts.len() == 3
            && parts
                .iter()
                .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit())),
        "VERSION phải có dạng X.Y.Z (số), đang là {version:?}"
    );
    let cargo = std::env::var("CARGO_PKG_VERSION").unwrap();
    assert!(
        cargo == version,
        "Phiên bản trong apps/windows/Cargo.toml ({cargo}) khác tệp VERSION ({version}). Sửa Cargo.toml cho khớp."
    );
    println!("cargo:rerun-if-changed=../../VERSION");

    // Ngày dựng (YYYY-MM-DD, UTC) — dùng cho thẻ nhắc cập nhật sau 60 ngày. CI đặt LVHV_BUILD_DATE.
    let date = std::env::var("LVHV_BUILD_DATE")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(today_utc);
    assert!(
        date.len() == 10 && date.as_bytes()[4] == b'-' && date.as_bytes()[7] == b'-',
        "LVHV_BUILD_DATE phải có dạng YYYY-MM-DD, đang là {date:?}"
    );
    let build = std::env::var("LVHV_BUILD_NUMBER")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "0".into());
    assert!(
        build.bytes().all(|b| b.is_ascii_digit()),
        "LVHV_BUILD_NUMBER phải là số"
    );
    println!("cargo:rustc-env=LVHV_BUILD_DATE={date}");
    println!("cargo:rustc-env=LVHV_BUILD_NUMBER={build}");
    println!("cargo:rerun-if-env-changed=LVHV_BUILD_DATE");
    println!("cargo:rerun-if-env-changed=LVHV_BUILD_NUMBER");

    // Mã Microsoft Store: [package.metadata.lvhv] ms-store-id trong Cargo.toml (mặc định trống).
    let toml = fs::read_to_string("Cargo.toml").expect("đọc Cargo.toml");
    let store_id = toml
        .lines()
        .find_map(|l| {
            let l = l.trim();
            l.strip_prefix("ms-store-id")
                .and_then(|r| r.trim_start().strip_prefix('='))
                .map(|r| r.trim().trim_matches('"').to_string())
        })
        .unwrap_or_default();
    assert!(
        store_id.bytes().all(|b| b.is_ascii_alphanumeric()),
        "ms-store-id chỉ gồm chữ và số, đang là {store_id:?}"
    );
    println!("cargo:rustc-env=LVHV_MS_STORE_ID={store_id}");
    println!("cargo:rerun-if-changed=Cargo.toml");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        // FileVersion/ProductVersion lấy từ CARGO_PKG_VERSION (đã kiểm bằng VERSION ở trên).
        res.set_icon("assets/icon.ico")
            .set("ProductName", "Lịch và Văn hóa Việt")
            .set("FileDescription", "Lịch và Văn hóa Việt")
            .set("CompanyName", "Clevix LLC (Clevix Lab)")
            .set("LegalCopyright", "© 2026 Clevix LLC")
            .set("OriginalFilename", "LichViet.exe")
            .set_language(0x042A); // vi-VN
        if let Err(e) = res.compile() {
            println!("cargo:warning=không nhúng được tài nguyên Windows: {e}");
        }
    }
    println!("cargo:rerun-if-changed=assets/icon.ico");
}

/// Ngày hôm nay (UTC) dạng YYYY-MM-DD, không cần thư viện ngoài.
fn today_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let (y, m, d) = civil_from_days(secs.div_euclid(86400));
    format!("{y:04}-{m:02}-{d:02}")
}

/// Thuật toán của Howard Hinnant: số ngày kể từ 1970-01-01 → (năm, tháng, ngày).
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + if m <= 2 { 1 } else { 0 }, m, d)
}
