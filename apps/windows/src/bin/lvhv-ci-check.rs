//! `lvhv-ci-check.exe` — chỉ dùng trong CI (`.github/workflows/windows.yml`), **không** đóng gói vào `.msi`/`.zip`
//! (MA3C-25-01). Xem `src/ci.rs` cho danh sách chế độ.
#![deny(unsafe_code)]

fn main() {
    std::process::exit(lvhv_windows::ci::main());
}
