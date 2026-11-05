// Biên dịch giao diện Slint và (chỉ khi dựng cho Windows) nhúng biểu tượng + thông tin phiên bản vào .exe.
fn main() {
    slint_build::compile_with_config(
        "ui/app.slint",
        slint_build::CompilerConfiguration::new().with_style("fluent".into()),
    )
    .expect("biên dịch ui/app.slint");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
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
