//! Móc kiểm thử (chỉ có khi bật tính năng `test-hooks`; không bao giờ có trong bản dựng ứng dụng).
//!
//! Dùng cho công cụ kiểm định hộp đen `lvhv-store-tool open --force-migration-failure` (docs/v0.2/10): ép bước
//! chuyển đổi kho SQLite (phiên bản ≥ 1 → [`crate::migrate::STORE_VERSION`]) lỗi **sau** khi mọi bước chuyển đổi
//! đã chạy trong giao dịch, ngay trước `COMMIT`. Như vậy quan sát được đúng đường lỗi của 04 B5 mục 3: giao dịch
//! quay lui, bản sao `.pre-vN.bak` còn nguyên, kho mở ở chế độ chỉ đọc. Kho mới tạo (phiên bản 0) không bị ảnh hưởng.
use std::sync::atomic::{AtomicBool, Ordering};

static FORCE_MIGRATION_FAILURE: AtomicBool = AtomicBool::new(false);

/// Bật/tắt lỗi chuyển đổi ép buộc cho các lần [`crate::Store::open`] sau đó (toàn tiến trình).
pub fn set_force_migration_failure(on: bool) {
    FORCE_MIGRATION_FAILURE.store(on, Ordering::SeqCst);
}

pub(crate) fn force_migration_failure() -> bool {
    FORCE_MIGRATION_FAILURE.load(Ordering::SeqCst)
}
