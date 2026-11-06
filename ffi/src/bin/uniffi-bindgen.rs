//! Trình sinh mã UniFFI (Kotlin/Swift) — chỉ dùng khi dựng, không đóng gói vào ứng dụng.
//! Chạy: cargo run -p lvhv-ffi --features bindgen-cli --bin uniffi-bindgen -- generate ...
fn main() {
    uniffi::uniffi_bindgen_main()
}
