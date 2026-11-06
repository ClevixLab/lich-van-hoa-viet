// Dựng lại khi dữ liệu nhúng (include_str!) thay đổi.
fn main() {
    println!("cargo:rerun-if-changed=../content/cadao/cadao.json");
    println!("cargo:rerun-if-changed=../content/cadao/SOURCES.md");
}
