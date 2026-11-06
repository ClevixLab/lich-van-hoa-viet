use lvhv_store::validate;

#[test]
fn example_backup_validates_except_placeholder_sha() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../docs/v0.2/schema/example-backup-v1.json"
    ))
    .unwrap();
    let v = validate::parse_json(&text).unwrap();
    validate::document(&v).unwrap();
}
