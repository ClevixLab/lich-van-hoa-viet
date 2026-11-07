//! `LichViet.exe` — bản phát hành của Lịch và Văn hóa Việt cho Windows. Không có chế độ kiểm thử (MA3C-25-01); các
//! chế độ của CI nằm trong `lvhv-ci-check.exe`, không đóng gói.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![deny(unsafe_code)]

fn main() -> Result<(), slint::PlatformError> {
    lvhv_windows::run_app()
}
