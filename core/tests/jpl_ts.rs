//! tsport/src/jpl_days.ts phải đúng bằng bảng mức ngày do lõi sinh từ bảng Jpl nhúng (docs/03 §17.5).
//! Sinh lại: `cargo run --release -p lvhv-core --bin lvhv-dump -- jpl-days-ts > tsport/src/jpl_days.ts`.

#[test]
fn tsport_jpl_days_matches_core() {
    let want = lvhv_core::jpl::days_ts_source().expect("bảng Jpl");
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../tsport/src/jpl_days.ts");
    let got = std::fs::read_to_string(path).expect("đọc tsport/src/jpl_days.ts");
    assert!(
        got == want,
        "tsport/src/jpl_days.ts cũ — sinh lại bằng `lvhv-dump jpl-days-ts`"
    );
}
