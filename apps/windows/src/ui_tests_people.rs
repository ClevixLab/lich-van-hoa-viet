//! Kiểm thử giao diện mô-đun "Người thân" (đặc tả 08 §9, luồng L1–L13, US-M-07, B4): bấm chuột giả lập vào
//! nút thật như người dùng; ô chữ được điền bằng thuộc tính đã nối hai chiều với ô nhập (như khi gõ) rồi gọi
//! sự kiện "đã sửa". Kết quả so với dữ liệu trong kho (lvhv-store) và hàm của lõi (lvhv-memorial), không tự
//! viết lại thuật toán. Kho dữ liệu nằm trong thư mục tạm; đồng hồ giả: 07/11/2026 09:00 (UTC+7).

use std::path::PathBuf;
use std::rc::Rc;

use i_slint_backend_testing::ElementHandle;
use serde_json::Value;
use slint::{ComponentHandle, Model};

use crate::platform::{self, Dialog};
use crate::ui::*;
use crate::ui_tests::{by_label, click, count_label, scroll_into_view};

const SECRET: &str = "BI-MAT-BAO-TU-7731";
/// Lý do trong thông tin báo tử của dữ liệu mẫu (shots::seed_store).
const SEED_SECRET: &str = "Tuổi cao, bệnh tim";

fn v(s: &str) -> Value {
    serde_json::from_str(s).unwrap()
}

fn last_by_label(ui: &AppWindow, label: &str) -> ElementHandle {
    let all: Vec<_> = ElementHandle::find_by_accessible_label(ui, label).collect();
    assert!(!all.is_empty(), "không tìm thấy phần tử có nhãn {label:?}");
    all.into_iter().last().unwrap()
}

/// Cuộn tới rồi bấm.
fn press(ui: &AppWindow, label: &str) {
    scroll_into_view(ui, label);
    click(ui, &by_label(ui, label));
}

/// Về một trang của thanh phân đoạn: bấm "‹ Về …" cho tới khi thanh phân đoạn hiện ra, rồi bấm mục đó.
fn seg(ui: &AppWindow, name: &str) {
    for _ in 0..6 {
        if !matches!(page(ui).as_str(), "sap-toi" | "theo-ky" | "ho-so" | "sao-luu") {
            let back = People::get(ui).get_back_label().to_string();
            press(ui, &back);
        }
    }
    press(ui, name);
}

fn page(ui: &AppWindow) -> String {
    People::get(ui).get_page().to_string()
}

/// Dựng cửa sổ, kho tạm, đồng hồ giả, hộp chọn tệp giả. Mỗi kiểm thử chạy trên luồng riêng (backend kiểm thử khởi
/// tạo một lần mỗi luồng; đồng hồ giả và hộp chọn tệp giả là biến cục bộ của luồng).
#[allow(clippy::type_complexity)]
fn fixture(tag: &str) -> (AppWindow, std::rc::Rc<std::cell::RefCell<crate::State>>, crate::people::Ps, PathBuf, Rc<std::cell::RefCell<PathBuf>>, Rc<std::cell::RefCell<PathBuf>>, crate::D) {
    i_slint_backend_testing::init_no_event_loop();
    let tmp = std::env::temp_dir().join(format!("lvhv-win-uitest-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    // 07/11/2026 09:00 giờ Việt Nam.
    let day0 = crate::D { y: 2026, m: 11, d: 7 };
    platform::set_fake_now(Some((day0.jdn() - 2_440_588) * 86_400 + 2 * 3600));
    platform::set_fake_offset(Some(420));
    // Kiểm thử không đặt thông báo thật của Windows (máy CI hay máy người chạy kiểm thử).
    platform::set_demo_mode();
    let ui = AppWindow::new().unwrap();
    let st = crate::setup(&ui);
    let ps = st.borrow().people.clone();
    ps.borrow_mut().dir = tmp.clone();
    ps.borrow_mut().store = None;
    ui.window().set_size(slint::LogicalSize::new(1280., 3000.));
    ui.show().unwrap();

    // Ảnh mẫu có EXIF + APP13 (vị trí giả) để kiểm S9.
    let img = tmp.join("chan-dung.jpg");
    std::fs::write(&img, crate::photo::tests::jpeg_with_exif(320, 200)).unwrap();
    let save_to: Rc<std::cell::RefCell<PathBuf>> = Rc::new(std::cell::RefCell::new(tmp.join("x")));
    let open_from: Rc<std::cell::RefCell<PathBuf>> = Rc::new(std::cell::RefCell::new(img.clone()));
    {
        let (s, o, i) = (save_to.clone(), open_from.clone(), img.clone());
        platform::set_dialog_override(Some(Box::new(move |k| match k {
            Dialog::OpenImage => Some(i.clone()),
            Dialog::OpenBackup | Dialog::OpenAttachment => Some(o.borrow().clone()),
            Dialog::Save(_) => Some(s.borrow().clone()),
        })));
    }

    (ui, st, ps, tmp, save_to, open_from, day0)
}

/// L2 (ngày âm, tháng nhuận, lỗi), nhắc, ảnh (S9), L11 tưởng niệm, L12 sửa.
#[test]
fn nguoi_da_mat_tuong_niem_sua() {
    let (ui, _st, ps, tmp, save_to, open_from, day0) = fixture("a");
    let p = People::get(&ui);
    let _ = (&save_to, &open_from, day0);
    // --- Mở mô-đun: trống → khung chào ---
    click(&ui, &by_label(&ui, "Người thân"));
    assert_eq!(ui.get_tab(), 4);
    assert_eq!(page(&ui), "sap-toi");
    assert!(!p.get_has_any());
    assert!(count_label(&ui, "+ Thêm người đã mất") >= 1, "khung chào có nút thêm");

    // --- L2: thêm người đã mất theo ngày âm, tháng nhuận ---
    press(&ui, "+ Thêm người đã mất");
    assert_eq!(page(&ui), "form");
    assert!(p.get_f_dead());
    press(&ui, "Lưu");
    assert_eq!(page(&ui), "form", "thiếu họ tên và ngày mất thì không lưu");
    assert!(p.get_f_summary().starts_with("Còn "), "khung tóm tắt lỗi");
    assert_eq!(p.get_e_ho_ten(), "Họ tên không được để trống");
    assert_eq!(p.get_dd_error(), "Hãy nhập ngày mất");
    // YC-2026-07: gợi ý cách gọi cho người đã mất có Cụ, Kỵ, Ông Tổ họ, Bà Cô Tổ họ, Bé đỏ; bấm nút gợi ý điền ô.
    for w in lvhv_ffi::danh_xung_suggestions(true, None) {
        assert_eq!(count_label(&ui, &format!("Cách gọi: {w}")), 1, "gợi ý {w}");
    }
    for w in lvhv_ffi::quan_he_suggestions(true) {
        assert_eq!(count_label(&ui, &format!("Quan hệ: {w}")), 1, "gợi ý {w}");
    }
    // Giới tính (YC-2026-07): chọn Nữ → gợi ý hợp giới tính lên trước.
    press(&ui, "Giới tính: Nữ");
    assert_eq!(p.get_f_gender(), 1);
    let first = p.get_dx_rows().row_data(0).unwrap().items.row_data(0).unwrap();
    assert_eq!(first, lvhv_ffi::danh_xung_suggestions(true, Some("nu".into()))[0]);
    press(&ui, "Cách gọi: Bà");
    assert_eq!(p.get_f_danh_xung(), "Bà");
    p.set_f_ho_ten("Trần Thị Bình".into());
    p.set_f_quan_he("bà nội (bên nội)".into());
    p.invoke_form_changed();
    assert_eq!(p.get_e_ho_ten(), "", "lỗi tự mất khi đã sửa");
    press(&ui, "Ngày mất: Nhập theo âm lịch");
    assert_eq!(p.get_dd_cal(), 1);
    p.set_dd_d("15".into());
    p.set_dd_m("5".into());
    p.set_dd_y("2025".into());
    p.set_dd_leap(true);
    p.invoke_form_changed();
    assert_eq!(p.get_dd_error(), "Năm 2025 không có tháng 5 nhuận");
    assert_eq!(p.get_dd_leap_label(), "Tháng nhuận (năm 2025 nhuận tháng 6)");
    p.set_dd_y("2026".into());
    p.invoke_form_changed();
    assert!(!p.get_dd_leap_visible(), "năm không có tháng nhuận → ẩn hộp nhuận");
    assert_eq!(p.get_dd_note(), "Năm 2026 không có tháng nhuận");
    p.set_dd_y("2025".into());
    p.set_dd_m("6".into());
    p.invoke_form_changed();
    assert_eq!(p.get_dd_error(), "");
    assert_eq!(p.get_dd_result(), "Dương lịch: Thứ Sáu, 08/08/2025");
    // Nhắc: thêm mốc 10 ngày, bật tiên thường.
    press(&ui, "Nhắc trước: 10 ngày");
    press(&ui, "Nhắc thêm ngày tiên thường (lễ cúng hôm trước ngày giỗ)");
    assert!(p.get_r_tt());
    p.set_r_hh("25".into());
    p.invoke_form_changed();
    assert_eq!(p.get_r_error(), "Giờ nhắc phải từ 0 đến 23 giờ, phút từ 0 đến 59");
    p.set_r_hh("18".into());
    p.set_r_mm("5".into());
    p.invoke_form_changed();
    assert_eq!(p.get_r_error(), "");
    // Ảnh chân dung (S9).
    press(&ui, "Chọn ảnh");
    assert!(p.get_f_has_photo());
    press(&ui, "Lưu");
    assert_eq!(page(&ui), "nguoi", "lưu xong về trang hồ sơ");
    assert_eq!(p.get_toast(), "Đã lưu hồ sơ.");
    let store = ps.borrow().store.clone().unwrap();
    let persons = v(&store.persons_json().unwrap());
    let b = persons.as_array().unwrap()[0].clone();
    let bid = b["id"].as_str().unwrap().to_string();
    assert_eq!(b["death"]["lunar"], serde_json::json!({"year": 2025, "month": 6, "day": 15, "leap": true}));
    assert_eq!(b["death"]["lunar_source"], "entered");
    assert_eq!(b["danh_xung"], "Bà");
    assert_eq!(b["quan_he"], "Bà nội (bên nội)", "YC-2026-07: viết hoa chữ đầu khi lưu");
    assert_eq!(b["gioi_tinh"], "nu");
    assert!(p.get_v_info().iter().any(|kv| kv.k == "Giới tính" && kv.v == "Nữ"), "trang hồ sơ ghi giới tính");
    assert_eq!(b["reminders"]["gio"]["offsets_days"], serde_json::json!([0, 3, 10]));
    assert_eq!(b["reminders"]["gio"]["time_local"], "18:05");
    assert_eq!(b["reminders"]["gio"]["tien_thuong"], true);
    let bytes = store.photo_bytes(b["photo_id"].as_str().unwrap().into()).unwrap();
    assert!(!crate::photo::has_metadata_segments(&bytes), "ảnh lưu không còn APP1/APP13");
    assert!(!bytes.windows(10).any(|w| w == b"GPS-BI-MAT"));
    assert!(p.get_v_has_photo());

    // --- L11: màn hình tưởng niệm ---
    press(&ui, "Tưởng niệm");
    assert_eq!(page(&ui), "tuong-niem");
    assert_eq!(p.get_m_rel(), "Bà nội", "dòng trên tên lấy từ ô Quan hệ, bỏ phần trong ngoặc");
    assert_eq!(p.get_m_name(), "Trần Thị Bình", "không lặp cách gọi");
    assert!(p.get_m_incense());
    assert!(p.get_m_dateline().contains("15/6 âm lịch"), "giỗ hằng năm vào tháng thường (O-4)");
    assert!(p.get_m_smoke());
    press(&ui, "Dừng khói");
    assert!(!p.get_m_smoke());
    press(&ui, "Thắp lại");
    assert!(p.get_m_smoke());
    press(&ui, "‹ Về hồ sơ");
    assert_eq!(page(&ui), "nguoi");

    // --- L12: sửa → rev tăng ---
    press(&ui, "Sửa hồ sơ");
    assert_eq!(page(&ui), "form");
    assert_eq!(p.get_f_ho_ten(), "Trần Thị Bình");
    p.set_f_ghi_chu("Giỗ ở nhà bác cả.".into());
    p.invoke_form_changed();
    press(&ui, "Lưu");
    assert_eq!(page(&ui), "nguoi");
    assert_eq!(v(&store.person_json(bid.clone()).unwrap())["rev"], 2);

    platform::set_fake_now(None);
    platform::set_fake_offset(None);
    platform::set_dialog_override(None);
    let _ = std::fs::remove_dir_all(&tmp);
}

/// L3 người đang sống + sinh nhật âm, L4 sự kiện, US-M-07 cập nhật đã mất + thông tin báo tử, B4.
#[test]
fn nguoi_dang_song_su_kien_bao_tu() {
    let (ui, _st, ps, tmp, save_to, open_from, day0) = fixture("b");
    let p = People::get(&ui);
    let _ = (&save_to, &open_from, day0);
    click(&ui, &by_label(&ui, "Người thân"));
    let store = ps.borrow().store.clone().unwrap();
    // --- L3: người đang sống ---
    seg(&ui, "Hồ sơ");
    assert_eq!(page(&ui), "ho-so");
    press(&ui, "Thêm người đang sống");
    assert!(!p.get_f_dead());
    // Người đang sống: không có năm cách gọi chỉ dành cho người đã mất (YC-2026-07).
    for w in ["Cụ", "Kỵ", "Ông Tổ họ", "Bà Cô Tổ họ", "Bé đỏ"] {
        assert_eq!(count_label(&ui, &format!("Cách gọi: {w}")), 0, "không gợi ý {w} cho người sống");
    }
    assert_eq!(count_label(&ui, "Cách gọi: Anh"), 1);
    p.set_f_danh_xung("  anh".into());
    p.set_f_ho_ten("Lê Văn Cường".into());
    p.set_f_quan_he("con trai".into());
    p.set_bd_d("20".into());
    p.set_bd_m("1".into());
    p.set_bd_y("1990".into());
    p.invoke_form_changed();
    assert_eq!(p.get_bd_result(), "Âm lịch: 24/12 năm Kỷ Tỵ (1989) · Hôm nay 36 tuổi (tuổi âm 38)");
    press(&ui, "Nhắc cả sinh nhật âm lịch hằng năm");
    assert!(p.get_f_am());
    press(&ui, "Lưu");
    assert_eq!(page(&ui), "nguoi");
    let cid = p.get_v_id().to_string();
    let c = v(&store.person_json(cid.clone()).unwrap());
    assert_eq!(c["sinh_nhat_am"], true);
    assert_eq!(c["danh_xung"], "Anh", "viết hoa chữ đầu, bỏ khoảng trắng");
    assert_eq!(c["quan_he"], "Con trai");
    assert_eq!(p.get_v_rel(), "Con trai");
    assert_eq!(c["reminders"]["sinh_nhat_am"]["offsets_days"], c["reminders"]["sinh_nhat"]["offsets_days"]);
    assert_ne!(c["reminders"]["sinh_nhat_am"]["id"], c["reminders"]["sinh_nhat"]["id"], "TK-29: id riêng");

    // --- L4: sự kiện ---
    press(&ui, "+ Thêm sự kiện");
    assert_eq!(page(&ui), "su-kien");
    assert!(!p.get_ev_dead());
    p.set_ed_d("17".into());
    p.set_ed_m("11".into());
    p.set_ed_y("2026".into());
    p.invoke_form_changed();
    press(&ui, "Lưu");
    assert_eq!(page(&ui), "nguoi");
    assert_eq!(p.get_v_events().row_count(), 1);
    let involves = |o: &Value| {
        o["entity_id"] == cid.as_str()
            || o["person_ids"]
                .as_array()
                .is_some_and(|a| a.iter().any(|x| x == cid.as_str()))
    };
    let up_before = v(&store.upcoming_json(day0.solar(), 366).unwrap());
    assert!(
        up_before.as_array().unwrap().iter().any(|o| involves(o) && o["class"] != "gio"),
        "trước khi cập nhật: có sinh nhật / sự kiện"
    );

    // --- US-M-07: người đang sống → đã mất, kèm thông tin báo tử ---
    press(&ui, "Cập nhật: đã mất");
    assert_eq!(page(&ui), "bao-tu");
    assert!(p.get_n_open(), "khối báo tử mở sẵn ở biểu mẫu này");
    p.set_md_d("1".into());
    p.set_md_m("11".into());
    p.set_md_y("2026".into());
    p.set_n_by(SECRET.into());
    p.set_n_reason(format!("{SECRET} lý do").into());
    p.invoke_form_changed();
    // YC-2026-07 / 01 §3.8a: tệp đính kèm — một ảnh (nén lại, bỏ EXIF) và một PDF (lưu nguyên bản).
    *open_from.borrow_mut() = tmp.join("chan-dung.jpg");
    press(&ui, "Thêm tệp đính kèm");
    assert_eq!(p.get_n_atts().row_count(), 1);
    assert!(p.get_n_atts().row_data(0).unwrap().has_thumb);
    let pdf_path = tmp.join("Giấy chứng tử của Cường.pdf");
    std::fs::write(&pdf_path, crate::shots::demo_pdf()).unwrap();
    *open_from.borrow_mut() = pdf_path;
    press(&ui, "Thêm tệp đính kèm");
    assert_eq!(p.get_n_atts().row_count(), 2);
    assert!(p.get_n_atts().row_data(1).unwrap().pdf);
    press(&ui, "Loại tệp 2: Tệp khác");
    assert_eq!(p.get_n_atts().row_data(1).unwrap().kind, 2);
    // Mở PDF: hỏi trước (BM3-02, S10) — "Hủy" là mặc định và không mở gì; "Mở tệp" mới chép bản tạm tên chung trong
    // thư mục riêng của ứng dụng (§5.3), không mang tên người.
    press(&ui, "Mở tệp đính kèm 2");
    assert_eq!(p.get_dlg(), "mo-tep");
    assert!(p.get_dlg_default_cancel(), "nút mặc định là Hủy");
    assert!(p.get_dlg_lines().iter().any(|l| l.contains("ứng dụng khác") ));
    assert!(p.get_dlg_lines().iter().any(|l| l.contains("riêng tư")));
    click(&ui, &last_by_label(&ui, "Hủy"));
    assert_eq!(p.get_dlg(), "");
    assert!(ps.borrow().last_opened.is_none(), "Hủy thì không mở");
    assert!(!tmp.join("tmp").exists());
    press(&ui, "Mở tệp đính kèm 2");
    click(&ui, &last_by_label(&ui, "Mở tệp"));
    let opened = ps.borrow().last_opened.clone().unwrap();
    assert_eq!(opened.file_name().unwrap(), "TepDinhKem.pdf");
    assert!(opened.starts_with(tmp.join("tmp")));
    // Tệp không phải PDF/ảnh bị từ chối, không thêm dòng.
    let fake = tmp.join("gia.pdf");
    std::fs::write(&fake, b"<html>khong phai pdf</html>").unwrap();
    *open_from.borrow_mut() = fake;
    press(&ui, "Thêm tệp đính kèm");
    assert_eq!(p.get_n_atts().row_count(), 2);
    assert!(!p.get_e_notice().is_empty());
    press(&ui, "Lưu");
    assert_eq!(p.get_dlg(), "xac-nhan", "hộp xác nhận trước khi đổi");
    assert!(p.get_dlg_lines().iter().any(|l| l.contains("Sẽ không còn nhắc sinh nhật và sự kiện")));
    click(&ui, &last_by_label(&ui, "Lưu thay đổi"));
    assert_eq!(p.get_dlg(), "");
    assert_eq!(page(&ui), "nguoi");
    let c2 = v(&store.person_json(cid.clone()).unwrap());
    assert_eq!(c2["id"], cid.as_str(), "cùng hồ sơ");
    assert_eq!(c2["kind"], "deceased");
    assert_eq!(c2["rev"], c["rev"].as_i64().unwrap() + 1);
    assert_eq!(c2["birth"], c["birth"], "giữ ngày sinh");
    assert_eq!(c2["death"]["notice"]["reported_by"], SECRET);
    let atts = c2["death"]["notice"]["attachments"].as_array().unwrap().clone();
    assert_eq!(atts.len(), 2);
    assert_eq!(atts[0]["kind"], "giay_chung_tu");
    assert_eq!(atts[1]["kind"], "khac");
    let img = store.attachment_bytes(atts[0]["file_id"].as_str().unwrap().into()).unwrap();
    assert!(!crate::photo::has_metadata_segments(&img), "ảnh đính kèm không còn EXIF/APP13");
    assert!(!img.windows(10).any(|w| w == b"GPS-BI-MAT"));
    let pdf = store.attachment_bytes(atts[1]["file_id"].as_str().unwrap().into()).unwrap();
    assert_eq!(pdf, crate::shots::demo_pdf(), "PDF lưu nguyên bản");
    // Tệp đính kèm không vào lời nhắc, .ics (R-PRIV-2).
    let ds = store.dataset_json().unwrap();
    let ics = lvhv_ffi::ics_export(ds.clone(), day0.solar(), platform::now_unix(), None).unwrap();
    assert!(!ics.content.contains("Giấy chứng tử của Cường") && !ics.content.contains("TepDinhKem"));
    let rem = lvhv_ffi::reminders_json(ds, platform::now_unix(), crate::platform::device_zone(), 400, true).unwrap();
    assert!(!rem.contains("Giấy chứng tử của Cường"));
    assert!(p.get_v_dead());
    assert_eq!(p.get_v_events().row_count(), 1, "sự kiện vẫn giữ trên hồ sơ");
    // B4: sinh nhật và sự kiện của người này không còn trong Sắp tới / Theo kỳ.
    let up = v(&store.upcoming_json(day0.solar(), 366).unwrap());
    for o in up.as_array().unwrap().iter().filter(|o| involves(o)) {
        assert!(o["class"] == "gio" || o["class"] == "tien_thuong", "B4: {o}");
    }
    assert!(
        p.get_upcoming().iter().all(|it| !(it.pid == cid.as_str() && it.kind >= 2)),
        "Sắp tới không còn sinh nhật / sự kiện của người đã mất"
    );
    // Thông tin báo tử chỉ ở trang hồ sơ.
    press(&ui, "Mở thông tin báo tử");
    assert!(p.get_v_notice().iter().any(|kv| kv.v == SECRET));
    assert_eq!(p.get_v_atts().row_count(), 2);
    assert_eq!(p.get_v_atts().row_data(1).unwrap().name, "Giấy chứng tử của Cường.pdf");
    ps.borrow_mut().last_opened = None;
    press(&ui, "Mở tệp đính kèm 2");
    assert_eq!(p.get_dlg(), "mo-tep", "trang hồ sơ cũng hỏi trước");
    p.invoke_dlg_confirm();
    assert!(ps.borrow().last_opened.as_ref().unwrap().ends_with("TepDinhKem.pdf"));

    // --- B4: biểu mẫu sự kiện của người đã mất không có khối nhắc ---
    press(&ui, "+ Thêm sự kiện");
    assert!(p.get_ev_dead());
    assert_eq!(count_label(&ui, "Nhắc trước: Đúng ngày"), 0);
    press(&ui, "Hủy");
    assert_eq!(page(&ui), "nguoi");

    platform::set_fake_now(None);
    platform::set_fake_offset(None);
    platform::set_dialog_override(None);
    let _ = std::fs::remove_dir_all(&tmp);
}

/// L5 theo kỳ (so với lõi), O-9, L8 .ics, L9/L10 sao lưu → xóa → khôi phục, L7 thẻ nhắc hôm nay (cả trên Tờ lịch).
#[test]
fn theo_ky_ics_sao_luu_the_nhac() {
    let (ui, _st, ps, tmp, save_to, open_from, day0) = fixture("c");
    let p = People::get(&ui);
    let _ = (&save_to, &open_from, day0);
    click(&ui, &by_label(&ui, "Người thân"));
    let store = ps.borrow().store.clone().unwrap();
    let (_a_id, b_id, _c_id) = crate::shots::seed_store(&store);
    crate::people::refresh(&ui, &mut ps.borrow_mut());
    // --- L5: Theo kỳ, bốn loại kỳ × ba bộ lọc, so với lõi ---
    seg(&ui, "Theo kỳ");
    assert_eq!(page(&ui), "theo-ky");
    assert_eq!(p.get_ptitle(), "Tháng 11/2026 · 01/11 – 30/11");
    assert!(p.get_pcurrent());
    for (k, code) in [(0, "tuan"), (1, "thang"), (2, "quy"), (3, "nam")] {
        press(&ui, &format!("Loại kỳ: {}", ["Tuần", "Tháng", "Quý", "Năm"][k]));
        for (f, fcode) in [(0, "tat_ca"), (1, "da_mat"), (2, "dang_song")] {
            press(&ui, &format!("Bộ lọc: {}", ["Tất cả", "Người đã mất", "Người đang sống"][f]));
            let core = v(&store
                .events_in_period_json(code.into(), day0.solar(), fcode.into())
                .unwrap());
            assert_eq!(
                p.get_pitems().row_count(),
                core["items"].as_array().unwrap().len(),
                "kỳ {code}, lọc {fcode}"
            );
        }
    }
    press(&ui, "Bộ lọc: Tất cả");
    assert_eq!(p.get_ptitle(), "Năm 2026 · 01/01 – 31/12");
    press(&ui, "Loại kỳ: Tuần");
    assert_eq!(p.get_ptitle(), "Tuần 02/11 – 08/11/2026", "tuần Thứ Hai → Chủ nhật");
    press(&ui, "Loại kỳ: Quý");
    assert_eq!(p.get_ptitle(), "Quý IV/2026 · 01/10 – 31/12");
    press(&ui, "Kỳ sau");
    assert_eq!(p.get_ptitle(), "Quý I/2027 · 01/01 – 31/03");
    assert!(!p.get_pcurrent());
    press(&ui, "Kỳ trước");
    press(&ui, "Kỳ trước");
    assert_eq!(p.get_ptitle(), "Quý III/2026 · 01/07 – 30/09");
    press(&ui, "Hôm nay");
    assert_eq!(p.get_ptitle(), "Quý IV/2026 · 01/10 – 31/12");
    // Chạm một mục → hồ sơ người đó.
    if let Some(first) = p.get_pitems().row_data(0) {
        let label = format!("{}, {}, {}. Mở hồ sơ", first.line1, first.line2, first.right);
        press(&ui, &label);
        assert_eq!(page(&ui), "nguoi");
        assert_eq!(p.get_v_id(), first.pid);
    }

    // --- Cài đặt: hiện tên trong thông báo (O-9), lưu theo máy ---
    seg(&ui, "Sắp tới");
    assert!(!store.lock_screen_names().unwrap());
    press(&ui, "Hiện tên người trong thông báo");
    assert!(store.lock_screen_names().unwrap());

    // --- L8: .ics có xác nhận; không chứa thông tin báo tử ---
    seg(&ui, "Sao lưu");
    assert_eq!(page(&ui), "sao-luu");
    *save_to.borrow_mut() = tmp.join("lich.ics");
    press(&ui, "Xuất tệp lịch (.ics)…");
    assert_eq!(p.get_dlg(), "ics");
    assert!(p.get_dlg_lines().iter().any(|l| l.contains("Bất kỳ ai có tệp đều đọc được")));
    click(&ui, &last_by_label(&ui, "Lưu tệp .ics"));
    let ics = std::fs::read_to_string(tmp.join("lich.ics")).unwrap();
    assert!(ics.starts_with("BEGIN:VCALENDAR"));
    assert!(!ics.contains(SEED_SECRET), "R-PRIV-2: không có thông tin báo tử trong .ics");

    // --- L9/L10: sao lưu → xóa → khôi phục "Thay toàn bộ" ---
    let before = store.persons_json().unwrap();
    *save_to.borrow_mut() = tmp.join("saoluu.lvhv");
    press(&ui, "Tạo tệp sao lưu…");
    assert_eq!(p.get_dlg(), "tao-sao-luu");
    assert!(p.get_dlg_lines().iter().any(|l| l.contains("thông tin báo tử")), "hộp sao lưu nói rõ có báo tử");
    p.set_pw1("ngan".into());
    click(&ui, &last_by_label(&ui, "Tạo và lưu tệp"));
    assert_eq!(p.get_e_pw1(), "Mật khẩu cần ít nhất 8 ký tự");
    p.set_pw1("nhà ở Hà Đông có cây bàng".into());
    p.set_pw2("nhà ở Hà Đông có cây bàng".into());
    click(&ui, &last_by_label(&ui, "Tạo và lưu tệp"));
    assert_eq!(p.get_dlg(), "");
    assert!(tmp.join("saoluu.lvhv").exists());
    assert!(p.get_last_backup().starts_with("Lần sao lưu gần nhất: 07/11/2026"), "{}", p.get_last_backup());
    // Xóa một hồ sơ qua giao diện (L13): hủy thì còn, đồng ý thì mất.
    seg(&ui, "Hồ sơ");
    let n0 = p.get_dead().row_count();
    let first = p.get_dead().row_data(0).unwrap();
    press(&ui, &format!("{}{}. {}", first.name, if first.rel.is_empty() { String::new() } else { format!(", {}", first.rel) }, first.summary));
    press(&ui, "Xóa hồ sơ");
    assert_eq!(p.get_dlg(), "xac-nhan");
    click(&ui, &last_by_label(&ui, "Giữ lại"));
    assert_eq!(p.get_dlg(), "");
    click(&ui, &by_label(&ui, "Xóa hồ sơ"));
    click(&ui, &last_by_label(&ui, "Xóa hồ sơ"));
    assert_eq!(page(&ui), "ho-so");
    assert_eq!(p.get_dead().row_count(), n0 - 1);
    // Khôi phục.
    seg(&ui, "Sao lưu");
    *open_from.borrow_mut() = tmp.join("saoluu.lvhv");
    press(&ui, "Chọn tệp sao lưu…");
    assert_eq!(p.get_restore_file(), "Đã chọn: saoluu.lvhv");
    p.set_restore_pw("sai mật khẩu".into());
    press(&ui, "Mở tệp");
    assert_eq!(p.get_restore_error(), "Sai mật khẩu hoặc tệp bị hỏng");
    p.set_restore_pw("nhà ở Hà Đông có cây bàng".into());
    press(&ui, "Mở tệp");
    assert_eq!(p.get_dlg(), "khoi-phuc");
    click(&ui, &last_by_label(&ui, "Cách nhập: Thay toàn bộ"));
    click(&ui, &last_by_label(&ui, "Khôi phục"));
    assert_eq!(p.get_dlg_title(), "Thay toàn bộ dữ liệu?");
    click(&ui, &last_by_label(&ui, "Thay toàn bộ"));
    assert_eq!(p.get_dlg(), "");
    assert_eq!(store.persons_json().unwrap(), before, "khôi phục giống hệt trước khi xóa");
    assert!(p.get_toast().starts_with("Đã khôi phục: 5 người"));
    assert!(p.get_last_backup().contains("Dữ liệu được khôi phục từ bản sao lưu ngày 07/11/2026"));

    // --- Thẻ nhắc hôm nay (L7): giỗ bà Bình (dữ liệu mẫu) đúng hôm nay; trên Sắp tới và một thẻ trên Tờ lịch ---
    seg(&ui, "Sắp tới");
    assert!(p.get_cards().iter().any(|c| c.title.starts_with("Hôm nay: ") && c.title.contains("Trần Thị Bình") && c.memorial), "thẻ nhắc trong ứng dụng có tên");
    assert!(p.get_sheet_has_card());
    click(&ui, &by_label(&ui, "Tờ lịch"));
    assert_eq!(count_label(&ui, "Đã biết lời nhắc trên Tờ lịch"), 1, "Tờ lịch chỉ một thẻ");
    if p.get_sheet_card().memorial {
        click(&ui, &by_label(&ui, "Mở trang tưởng niệm"));
        assert_eq!(ui.get_tab(), 4);
        assert_eq!(page(&ui), "tuong-niem");
        assert_eq!(p.get_m_name(), "Trần Thị Bình");
        let _ = &b_id;
    }
    platform::set_fake_now(None);
    platform::set_fake_offset(None);
    platform::set_dialog_override(None);
    let _ = std::fs::remove_dir_all(&tmp);
}

/// R14 (04 Bổ sung B5 mục 3, 01 §7.1): chuyển đổi lỗi thật trên kho của bộ mẫu đóng băng → chế độ chỉ đọc: xem được,
/// không có nút sửa, xuất sao lưu được (không ghi mốc "đã sao lưu"); kho của bản mới hơn → chỉ còn khung báo.
#[test]
fn chuyen_doi_loi_che_do_chi_doc() {
    let (ui, _st, ps, tmp, save_to, _o, _d) = fixture("r14");
    let p = People::get(&ui);
    let fix = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/v0.2/schema/test-vectors/released/0.2-dev-before-attachments/store");
    crate::upgrade::copy_dir(&fix, &tmp).unwrap();
    let c = rusqlite::Connection::open(tmp.join("lvhv.sqlite")).unwrap();
    c.execute("CREATE TRIGGER kiem_dinh_loi_chuyen_doi BEFORE INSERT ON meta BEGIN SELECT RAISE(ABORT, 'kiểm định: lỗi chuyển đổi'); END", []).unwrap();
    let n: i64 = c.query_row("SELECT count(*) FROM person", [], |r| r.get(0)).unwrap();
    drop(c);
    click(&ui, &by_label(&ui, "Người thân"));
    assert!(ps.borrow().store.is_some(), "kho mở được (chỉ đọc)");
    assert!(p.get_read_only());
    assert!(!p.get_store_failed());
    assert!(p.get_store_error().contains("chỉ đọc"), "{}", p.get_store_error());
    // TKN3-W-01: chi tiết kỹ thuật không nằm trong lời chính, thu gọn sẵn; bấm mới hiện.
    assert!(!p.get_store_error().contains("SQLite") && !p.get_store_title().contains("SQLite"));
    assert!(p.get_store_detail().contains("SQLite"), "{}", p.get_store_detail());
    assert!(!p.get_store_detail_open());
    press(&ui, "Chi tiết kỹ thuật");
    assert!(p.get_store_detail_open());
    seg(&ui, "Hồ sơ");
    assert!(p.get_has_any(), "dữ liệu cũ xem được");
    assert_eq!(count_label(&ui, "Thêm người đã mất"), 0, "không có nút thêm");
    let first = p.get_dead().row_data(0).unwrap().id.to_string();
    p.invoke_open_person(first.into());
    assert_eq!(count_label(&ui, "Sửa hồ sơ"), 0, "không có nút sửa");
    assert_eq!(count_label(&ui, "Xóa hồ sơ"), 0);
    // Xuất sao lưu từ chế độ chỉ đọc (nút trên khung báo).
    seg(&ui, "Sắp tới");
    *save_to.borrow_mut() = tmp.join("chi-doc.lvhv");
    click(&ui, &last_by_label(&ui, "Tạo tệp sao lưu…"));
    assert_eq!(p.get_dlg(), "tao-sao-luu");
    p.set_pw1("mật khẩu chỉ đọc 2026".into());
    p.set_pw2("mật khẩu chỉ đọc 2026".into());
    p.invoke_dlg_confirm();
    assert!(p.get_toast().starts_with("Đã tạo tệp sao lưu"), "{}", p.get_toast());
    let b = lvhv_ffi::open_backup(std::fs::read(tmp.join("chi-doc.lvhv")).unwrap(), "mật khẩu chỉ đọc 2026".into()).unwrap();
    assert_eq!(b.preview().persons as i64, n, "tệp có đủ hồ sơ");
    let c = rusqlite::Connection::open(tmp.join("lvhv.sqlite")).unwrap();
    let m: i64 = c.query_row("SELECT count(*) FROM person", [], |r| r.get(0)).unwrap();
    let v: i64 = c.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
    assert_eq!((m, v), (n, 1), "không mất hồ sơ, kho cũ giữ phiên bản 1");
    drop(c);

    // Kho của bản mới hơn: không mở, chỉ còn khung báo và nút mở thư mục dữ liệu.
    let newer = tmp.join("moi-hon");
    {
        let s = lvhv_ffi::LvhvStore::open(newer.to_string_lossy().into_owned()).unwrap();
        crate::shots::seed_store(&s);
    }
    rusqlite::Connection::open(newer.join("lvhv.sqlite")).unwrap().pragma_update(None, "user_version", 99).unwrap();
    {
        let mut g = ps.borrow_mut();
        g.dir = newer.clone();
        g.store = None;
    }
    p.invoke_go("sap-toi".into());
    assert!(p.get_store_failed());
    assert!(p.get_store_title().contains("bản mới hơn"), "{}", p.get_store_title());
    press(&ui, "Mở thư mục dữ liệu");
    seg(&ui, "Hồ sơ");
    assert!(p.get_store_failed(), "không thử lại khi chuyển trang");
    platform::set_fake_now(None);
    platform::set_fake_offset(None);
    platform::set_dialog_override(None);
    let _ = std::fs::remove_dir_all(&tmp);
}
