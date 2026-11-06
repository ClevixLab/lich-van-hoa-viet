//! # lvhv-memorial — ngày giỗ, tuổi, lời nhắc, tệp .ics (Lịch và Văn hóa Việt v0.2)
//!
//! Cài đặt `docs/v0.2/02-THUAT-TOAN-GIO-NHAC.md` trên kiểu dữ liệu `docs/v0.2/01-MO-HINH-DU-LIEU.md`,
//! có các quyết định G1 (`docs/v0.2/06-BIEN-BAN-G1.md`): O-1 tiên thường tắt mặc định, O-2 giỗ hết
//! năm 2 (chọn 3), O-3 ngưỡng 60, O-4 tháng nhuận → tháng thường, O-5 sự kiện cả ngày, O-9 thông
//! báo trên màn hình khóa không hiện tên mặc định.
//!
//! Chỉ phụ thuộc `lvhv-core`; không `unsafe`; không I/O (trừ công cụ `lvhv-memorial-dump`).
//! Bản TypeScript tương ứng: `tsport/src/memorial*.ts` (giống từng byte, `scripts/check-parity.sh`).
#![forbid(unsafe_code)]

pub mod cal;
pub mod dump;
pub mod gio;
pub mod ics;
pub mod json;
pub mod model;
pub mod occ;
pub mod remind;

pub use cal::Cal;
pub use gio::{
    anniversaries_in, anniversary, can_chi_year, derive_birth, label_of, save_death,
    save_death_with, solar_occurrence, tuoi_am, tuoi_duong, tuoi_mat, Anniversary, GioLabel,
    InputError,
};
pub use model::*;
pub use occ::{notification_text, occurrences, upcoming, OccClass, OccKey, Occurrence, RemType};
pub use remind::{reminders_for, reschedule, Reminder, Zone};
