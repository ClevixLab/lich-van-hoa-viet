//! Chế độ chụp màn hình (`LichViet.exe --chup-man-hinh <thư mục>`): mở lần lượt các màn hình, chụp nội dung
//! cửa sổ bằng `Window::take_snapshot()` của Slint (vẽ lại bằng chính renderer của ứng dụng — Skia), ghi PNG
//! rồi thoát. Tên tệp có hậu tố tỉ lệ màn hình (`-1x`, `-2x`; đặt `SLINT_SCALE_FACTOR=2` để chụp 2×) và giao
//! diện (`-sang`, `-toi`). Mô-đun Người thân dùng một kho tạm với dữ liệu mẫu (ci.rs đặt thư mục dữ liệu tạm),
//! không chạm dữ liệu thật. Khi chạy thường (không có cờ) mô-đun này không làm gì.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use serde_json::{json, Value};
use slint::{ComponentHandle, Model, Timer, TimerMode};

use crate::people::{self, local_today, PState};
use crate::toasts;
use crate::ui::*;
use crate::{State, D};

type Step = (String, Box<dyn Fn(&AppWindow)>);

fn lunar_of(d: D) -> (i32, i32, i32, bool) {
    let l = lvhv_ffi::solar_to_lunar(d.y, d.m, d.d).expect("ngày trong miền");
    (l.y, l.m, l.d, l.leap)
}

fn rule(offs: &[i64], tt: bool) -> Value {
    json!({"id": lvhv_ffi::new_uuid().unwrap(), "enabled": true, "offsets_days": offs, "time_local": "07:00", "tien_thuong": tt})
}

/// Ảnh chân dung mẫu (vẽ bằng mã: nền ấm, hình người đơn giản) — không dùng ảnh thật của ai.
fn demo_portrait() -> Vec<u8> {
    let (w, h) = (480u32, 600u32);
    let img = image::RgbImage::from_fn(w, h, |x, y| {
        let (fx, fy) = (x as f32 / w as f32, y as f32 / h as f32);
        let head = ((fx - 0.5).powi(2) / 0.03 + (fy - 0.36).powi(2) / 0.035) < 1.0;
        let body = fy > 0.62 && ((fx - 0.5).abs() < 0.42 - (fy - 0.62) * -0.3);
        if head {
            image::Rgb([214, 178, 150])
        } else if body {
            image::Rgb([58, 52, 60])
        } else {
            let t = (fy * 60.0) as u8;
            image::Rgb([150 - t / 2, 132 - t / 2, 120 - t / 3])
        }
    });
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 90)
        .encode_image(&img)
        .unwrap();
    out
}

/// Giấy báo tử mẫu (chỉ là khung chữ giả, không có thông tin thật).
fn demo_certificate() -> Vec<u8> {
    let (w, h) = (420u32, 560u32);
    let img = image::RgbImage::from_fn(w, h, |x, y| {
        let line = (y % 34) < 3 && y > 80 && y < 500 && x > 40 && x < 380;
        let head = y > 30 && y < 60 && x > 110 && x < 310;
        if line || head {
            image::Rgb([120, 110, 100])
        } else {
            image::Rgb([246, 240, 226])
        }
    });
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 90)
        .encode_image(&img)
        .unwrap();
    out
}

/// Gieo dữ liệu mẫu vào kho tạm. Trả id của (người đã mất có ảnh + báo tử, người đã mất có giỗ hôm nay, người
/// đang sống).
pub fn seed_demo(ps: &mut PState) -> (String, String, String) {
    seed_store(&ps.store.clone().expect("kho đã mở"))
}

/// Gieo dữ liệu mẫu vào một kho (dùng cả cho `--kiem-thong-bao`).
/// Một PDF nhỏ hợp lệ (bản quét mẫu) cho ảnh chụp và kiểm thử.
pub fn demo_pdf() -> Vec<u8> {
    b"%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[]/Count 0>>endobj\ntrailer<</Root 1 0 R>>\n%%EOF\n".to_vec()
}

pub fn seed_store(store: &lvhv_ffi::LvhvStore) -> (String, String, String) {
    let today = local_today();
    // 1) Ông nội — mất 08/08/2025 (15/6 nhuận Ất Tỵ), có ảnh.
    let ph = crate::photo::reencode(&demo_portrait()).unwrap();
    let photo_id = lvhv_ffi::new_uuid().unwrap();
    store
        .add_photo(photo_id.clone(), "image/jpeg".into(), ph.width, ph.height, ph.bytes)
        .unwrap();
    let a = json!({
        "kind": "deceased", "danh_xung": "Ông", "ho_ten": "Nguyễn Văn An", "ten_hieu": "Phúc Thiện",
        "quan_he": "ông nội", "ghi_chu": "",
        "birth": {"input": {"calendar":"solar","precision":"day","year":1940,"month":5,"day":1,"leap":false}, "noi_sinh": "Hà Nội"},
        "death": {"input": {"calendar":"solar","precision":"day","year":2025,"month":8,"day":8,"leap":false}},
        "noi_an_tang": "Nghĩa trang quê nhà, Thanh Oai", "photo_id": photo_id,
        "memorial": {"tray_style":"truyen_thong","incense":true,"bia_tieu_de":"","loi_tuong_niem":"Con cháu luôn nhớ ông.","gio_het_nam":2,"huong_duong_duoi":60},
        "reminders": {"gio": rule(&[0, 3], false), "sinh_nhat": null, "sinh_nhat_am": null},
        "sinh_nhat_am": false, "feb29_rule": "feb28",
    });
    let a = serde_json::from_str::<Value>(&store.save_person_json(a.to_string()).unwrap()).unwrap();
    // 2) Bà ngoại — giỗ đúng hôm nay (nhập theo âm lịch, ba năm trước).
    let (ly, lm, ld, leap) = lunar_of(today);
    let b = json!({
        "kind": "deceased", "danh_xung": "Bà", "ho_ten": "Trần Thị Bình", "ten_hieu": "", "quan_he": "bà ngoại",
        "ghi_chu": "", "birth": {"input": {"calendar":"lunar","precision":"year","year":1938,"month":null,"day":null,"leap":false}, "noi_sinh": ""},
        "death": {"input": {"calendar":"lunar","precision":"day","year":ly - 3,"month":lm,"day":ld.min(29),"leap":false}},
        "noi_an_tang": "", "photo_id": null,
        "memorial": {"tray_style":"truyen_thong","incense":true,"bia_tieu_de":"","loi_tuong_niem":"","gio_het_nam":2,"huong_duong_duoi":60},
        "reminders": {"gio": rule(&[0, 3], true), "sinh_nhat": null, "sinh_nhat_am": null},
        "sinh_nhat_am": false, "feb29_rule": "feb28",
    });
    let _ = leap;
    let b = serde_json::from_str::<Value>(&store.save_person_json(b.to_string()).unwrap()).unwrap();
    // 3) Cụ — không rõ năm mất, giỗ sau 12 ngày.
    let d12 = today.add(12).unwrap();
    let (_, m12, d12l, _) = lunar_of(d12);
    let c = json!({
        "kind": "deceased", "danh_xung": "Cụ", "ho_ten": "Lê Thị Cúc", "ten_hieu": "Diệu Hạnh", "quan_he": "cụ bà bên nội",
        "ghi_chu": "Không rõ năm mất.", "birth": null,
        "death": {"input": {"calendar":"lunar","precision":"month_day","year":null,"month":m12,"day":d12l,"leap":false}},
        "noi_an_tang": "", "photo_id": null,
        "memorial": {"tray_style":"truyen_thong","incense":true,"bia_tieu_de":"","loi_tuong_niem":"","gio_het_nam":2,"huong_duong_duoi":60},
        "reminders": {"gio": rule(&[0, 3, 10], false), "sinh_nhat": null, "sinh_nhat_am": null},
        "sinh_nhat_am": false, "feb29_rule": "feb28",
    });
    store.save_person_json(c.to_string()).unwrap();
    // 4) Người đang sống, sinh nhật sau 5 ngày; sự kiện tốt nghiệp sau 20 ngày.
    let bday = today.add(5).unwrap();
    let l = json!({
        "kind": "living", "danh_xung": "Anh", "ho_ten": "Lê Văn Cường", "ten_hieu": "", "quan_he": "con trai",
        "ghi_chu": "", "birth": {"input": {"calendar":"solar","precision":"day","year":1990,"month":bday.m,"day":bday.d,"leap":false}, "noi_sinh": "Hải Phòng"},
        "death": null, "noi_an_tang": "", "photo_id": null, "memorial": null,
        "reminders": {"gio": null, "sinh_nhat": rule(&[0, 3], false), "sinh_nhat_am": rule(&[0, 3], false)},
        "sinh_nhat_am": true, "feb29_rule": "feb28",
    });
    let l = serde_json::from_str::<Value>(&store.save_person_json(l.to_string()).unwrap()).unwrap();
    let grad = today.add(20).unwrap();
    let e = json!({
        "type_id": "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0001", "tieu_de": "",
        "date": {"calendar":"solar","precision":"day","year":grad.y,"month":grad.m,"day":grad.d,"leap":false},
        "recurring": false, "person_ids": [l["id"]], "ghi_chu": "", "reminder": rule(&[0, 1], false),
    });
    store.save_event_json(e.to_string()).unwrap();
    // 5) Người đang sống được cập nhật thành đã mất, kèm thông tin báo tử và ảnh giấy báo tử.
    let m = json!({
        "kind": "living", "danh_xung": "Bác", "ho_ten": "Đỗ Văn Minh", "ten_hieu": "", "quan_he": "bác ruột",
        "ghi_chu": "", "birth": {"input": {"calendar":"solar","precision":"day","year":1951,"month":3,"day":2,"leap":false}, "noi_sinh": "Nam Định"},
        "death": null, "noi_an_tang": "", "photo_id": null, "memorial": null,
        "reminders": {"gio": null, "sinh_nhat": rule(&[0, 3], false), "sinh_nhat_am": null},
        "sinh_nhat_am": false, "feb29_rule": "feb28",
    });
    let m = serde_json::from_str::<Value>(&store.save_person_json(m.to_string()).unwrap()).unwrap();
    // Tệp đính kèm (01 §3.8a): ảnh giấy chứng tử (nén lại, bỏ siêu dữ liệu) và bản quét PDF (lưu nguyên bản).
    let cert = crate::photo::reencode_with(&demo_certificate(), crate::photo::DOC_SIDE, crate::photo::DOC_STORED).unwrap();
    let cert_id = lvhv_ffi::new_uuid().unwrap();
    store
        .add_attachment(cert_id.clone(), "image/jpeg".into(), Some("anh-tin-bao-tu.jpg".into()), cert.bytes)
        .unwrap();
    let pdf_id = lvhv_ffi::new_uuid().unwrap();
    store
        .add_attachment(pdf_id.clone(), "application/pdf".into(), Some("Ban quet giay chung tu.pdf".into()), demo_pdf())
        .unwrap();
    let notice = json!({
        "reported_by": "Chị Hoa (con gái)", "place": "Bệnh viện tỉnh Nam Định", "reason": "Tuổi cao, bệnh tim",
        "notice_date": {"calendar":"solar","precision":"day","year":2024,"month":7,"day":18,"leap":false},
        "attachments": [
            {"id": lvhv_ffi::new_uuid().unwrap(), "kind": "tin_bao_tu", "file_id": cert_id},
            {"id": lvhv_ffi::new_uuid().unwrap(), "kind": "giay_chung_tu", "file_id": pdf_id},
        ],
        "note": "Gia đình đã báo tin cho họ hàng.",
    });
    store
        .mark_deceased_json(
            m["id"].as_str().unwrap().into(),
            json!({"calendar":"solar","precision":"day","year":2024,"month":7,"day":17,"leap":false}).to_string(),
            Some(notice.to_string()),
            today.solar(),
        )
        .unwrap();
    (
        a["id"].as_str().unwrap().to_string(),
        b["id"].as_str().unwrap().to_string(),
        l["id"].as_str().unwrap().to_string(),
    )
}

fn theme(ui: &AppWindow, dark: bool) {
    Theme::get(ui).set_dark(dark);
}

fn steps(ui: &AppWindow, st: Rc<RefCell<State>>) -> Vec<Step> {
    let mut v: Vec<Step> = Vec::new();
    macro_rules! step {
        ($name:expr, $f:expr) => {
            v.push(($name.to_string(), Box::new($f)));
        };
    }
    // --- Các trang v0.1 (giao diện sáng; "7-toi" là tối) ---
    step!("01-to-lich", |_: &AppWindow| {});
    step!("02-chi-tiet", |ui: &AppWindow| ui.global::<Data>().set_details_open(true));
    step!("02b-chi-tiet-cuon", |ui: &AppWindow| ui.global::<Data>().set_scroll_frac(1.0));
    step!("03-bang-thang", |ui: &AppWindow| {
        let g = ui.global::<Data>();
        g.set_scroll_frac(0.0);
        g.invoke_open_month();
        ui.set_month_open(true);
    });
    step!("04-chon-ngay-tot", |ui: &AppWindow| {
        ui.set_month_open(false);
        let g = ui.global::<Data>();
        ui.set_tab(1);
        g.set_act(2);
        g.set_range(2);
        g.invoke_recompute_good();
    });
    step!("04b-chon-ngay-tot-ket-qua", |ui: &AppWindow| ui.global::<Data>().set_scroll_frac(0.62));
    step!("05-doi-ngay", |ui: &AppWindow| {
        let g = ui.global::<Data>();
        g.set_scroll_frac(0.0);
        ui.set_tab(2);
        g.set_s_text("02/09/1945".into());
        g.invoke_convert_solar();
        g.set_l_day(15);
        g.set_l_month(8);
        g.set_l_year("2027".into());
        g.invoke_convert_lunar();
    });
    step!("06-gioi-thieu", |ui: &AppWindow| ui.set_tab(3));
    step!("06b-gioi-thieu-chan-trang", |ui: &AppWindow| ui.global::<Data>().set_scroll_frac(1.0));
    step!("07-toi", |ui: &AppWindow| {
        let g = ui.global::<Data>();
        g.set_scroll_frac(0.0);
        theme(ui, true);
        ui.set_tab(0);
        g.set_details_open(false);
    });
    step!("08-chu-lon", |ui: &AppWindow| {
        theme(ui, false);
        Theme::get(ui).set_scale(1.32);
        ui.set_size(3);
    });

    // --- Mô-đun Người thân: mỗi màn hình chụp sáng và tối ---
    let ps = st.borrow().people.clone();
    let ids: Rc<RefCell<(String, String, String)>> = Rc::default();
    type Act = Rc<dyn Fn(&AppWindow)>;
    let mut screens: Vec<(&str, Act)> = Vec::new();
    {
        let ps = ps.clone();
        screens.push(("nt-01-trong", Rc::new(move |ui: &AppWindow| {
            Theme::get(ui).set_scale(1.0);
            ui.set_size(1);
            ui.set_tab(4);
            ui.global::<Data>().invoke_open_people();
            let _ = &ps;
        })));
    }
    {
        let ps = ps.clone();
        let ids = ids.clone();
        screens.push(("nt-02-sap-toi", Rc::new(move |ui: &AppWindow| {
            let mut p = ps.borrow_mut();
            if ids.borrow().0.is_empty() {
                *ids.borrow_mut() = seed_demo(&mut p);
            }
            people::go(ui, &mut p, "sap-toi");
            people::reschedule(Some(ui), &mut p);
        })));
    }
    for (name, r, enabled) in [
        ("nt-02b-thong-bao-dat-duoc", toasts::SyncResult::Scheduled { pending: 12, added: 12, removed: 0, confirmed: true }, true),
        ("nt-02c-thong-bao-ban-zip", toasts::SyncResult::Scheduled { pending: 12, added: 12, removed: 0, confirmed: false }, true),
        ("nt-02d-thong-bao-tat-trong-windows", toasts::SyncResult::Disabled { reason: "Thông báo của ứng dụng đang tắt trong Cài đặt Windows (Hệ thống › Thông báo).".into(), cleared: Ok(()) }, true),
        ("nt-02e-loi-nhac-dang-tat", toasts::SyncResult::Scheduled { pending: 0, added: 0, removed: 0, confirmed: true }, false),
    ] {
        let ps = ps.clone();
        screens.push((name, Rc::new(move |ui: &AppWindow| {
            people::go(ui, &mut ps.borrow_mut(), "sap-toi");
            let (ok, text) = people::notif_status(&r, enabled);
            let p = People::get(ui);
            p.set_notif_ok(ok);
            p.set_notif_status(text.into());
            p.set_rem_enabled(enabled);
            p.set_scroll_frac(0.0);
        })));
    }
    screens.push(("nt-02f-to-lich-the-nhac", Rc::new(|ui: &AppWindow| ui.set_tab(0))));
    {
        let ps = ps.clone();
        screens.push(("nt-02g-mac-dinh-nhac-mo", Rc::new(move |ui: &AppWindow| {
            ui.set_tab(4);
            people::go(ui, &mut ps.borrow_mut(), "sap-toi");
            let p = People::get(ui);
            p.set_defaults_open(true);
            p.set_scroll_frac(1.0);
        })));
    }
    for (name, kind, filter) in [("nt-03-theo-ky-thang", 1, 0), ("nt-04-theo-ky-quy", 2, 0), ("nt-05-theo-ky-nam-da-mat", 3, 1), ("nt-06-theo-ky-tuan", 0, 2)] {
        let ps = ps.clone();
        screens.push((name, Rc::new(move |ui: &AppWindow| {
            people::go(ui, &mut ps.borrow_mut(), "theo-ky");
            let p = People::get(ui);
            p.invoke_period_kind(kind);
            p.invoke_period_filter(filter);
        })));
    }
    {
        let ps = ps.clone();
        screens.push(("nt-07-ho-so", Rc::new(move |ui: &AppWindow| people::go(ui, &mut ps.borrow_mut(), "ho-so"))));
    }
    {
        let ids = ids.clone();
        screens.push(("nt-08-nguoi-da-mat", Rc::new(move |ui: &AppWindow| {
            People::get(ui).invoke_open_person(ids.borrow().0.as_str().into());
        })));
    }
    {
        let ps = ps.clone();
        screens.push(("nt-09-nguoi-bao-tu", Rc::new(move |ui: &AppWindow| {
            let id = {
                let p = ps.borrow();
                let s = p.store.clone().unwrap();
                let v: Value = serde_json::from_str(&s.persons_json().unwrap()).unwrap();
                v.as_array().unwrap().iter().find(|x| x["ho_ten"] == "Đỗ Văn Minh").unwrap()["id"].as_str().unwrap().to_string()
            };
            let p = People::get(ui);
            p.invoke_open_person(id.into());
            p.set_v_notice_open(true);
            p.set_scroll_frac(0.55);
        })));
    }
    // BM3-02: hỏi trước khi mở PDF bằng ứng dụng khác ("Hủy" là nút mặc định).
    screens.push(("nt-09c-hoi-mo-tep", Rc::new(|ui: &AppWindow| People::get(ui).invoke_open_v_attachment(1))));
    screens.push(("_nt-09c-dong", Rc::new(|ui: &AppWindow| People::get(ui).invoke_dlg_dismiss())));
    // Biểu mẫu sửa người đã mất: khối thông tin báo tử với hai tệp đính kèm (ảnh, PDF) — YC-2026-07.
    screens.push(("_nt-09b-chuan-bi", Rc::new(|ui: &AppWindow| {
        let p = People::get(ui);
        p.invoke_edit_person();
        p.set_n_open(true);
    })));
    screens.push(("nt-09b-sua-tep-dinh-kem", Rc::new(|ui: &AppWindow| People::get(ui).set_scroll_frac(0.86))));
    screens.push(("_nt-09c-ve", Rc::new(|ui: &AppWindow| People::get(ui).invoke_cancel_form())));
    {
        let ids = ids.clone();
        screens.push(("nt-10-nguoi-dang-song", Rc::new(move |ui: &AppWindow| {
            People::get(ui).invoke_open_person(ids.borrow().2.as_str().into());
        })));
    }
    screens.push(("nt-11-them-da-mat-am-nhuan", Rc::new(|ui: &AppWindow| {
        let p = People::get(ui);
        p.invoke_go("ho-so".into());
        p.invoke_add_dead();
        p.set_f_danh_xung("Bà".into());
        p.set_f_ho_ten("Phạm Thị Hồng".into());
        p.set_f_quan_he("bà nội".into());
        p.set_dd_cal(1);
        p.set_dd_d("15".into());
        p.set_dd_m("6".into());
        p.set_dd_y("2025".into());
        p.set_dd_leap(true);
        p.invoke_form_changed();
    })));
    screens.push(("nt-11b-them-da-mat-ket-qua", Rc::new(|ui: &AppWindow| People::get(ui).set_scroll_frac(0.3))));
    screens.push(("nt-11c-tu-nhap-tuoi", Rc::new(|ui: &AppWindow| {
        let p = People::get(ui);
        p.set_bd_d("1".into());
        p.set_bd_m("5".into());
        p.set_bd_y("1940".into());
        p.set_f_tuoi_manual(true);
        p.set_f_tuoi("85".into());
        p.invoke_form_changed();
        p.set_scroll_frac(0.42);
    })));
    screens.push(("nt-12-them-da-mat-loi", Rc::new(|ui: &AppWindow| {
        let p = People::get(ui);
        p.set_f_tuoi_manual(false);
        p.set_scroll_frac(0.0);
        p.set_f_ho_ten("".into());
        p.set_dd_m("5".into());
        p.invoke_save_form();
    })));
    screens.push(("nt-12b-them-da-mat-loi-ngay", Rc::new(|ui: &AppWindow| People::get(ui).set_scroll_frac(0.22))));
    screens.push(("_nt-13-chuan-bi", Rc::new(|ui: &AppWindow| {
        let p = People::get(ui);
        p.set_dlg("".into());
        p.invoke_go("ho-so".into());
        p.invoke_add_living();
        p.set_f_ho_ten("Lê Văn Dũng".into());
        p.set_f_quan_he("cháu".into());
        p.set_bd_d("20".into());
        p.set_bd_m("1".into());
        p.set_bd_y("1990".into());
        p.set_f_am(true);
        p.set_f_danh_xung("Cháu".into());
        p.invoke_form_changed();
    })));
    // YC-2026-07: gợi ý cách gọi của người đang sống (không có Cụ, Kỵ, Ông Tổ họ, Bà Cô Tổ họ, Bé đỏ).
    screens.push(("nt-13-them-dang-song-goi-y", Rc::new(|ui: &AppWindow| People::get(ui).set_scroll_frac(0.0))));
    screens.push(("nt-13b-them-dang-song-nhac", Rc::new(|ui: &AppWindow| People::get(ui).set_scroll_frac(0.75))));
    {
        let ids = ids.clone();
        screens.push(("nt-14-cap-nhat-da-mat", Rc::new(move |ui: &AppWindow| {
            let p = People::get(ui);
            p.invoke_go("ho-so".into());
            p.invoke_open_person(ids.borrow().2.as_str().into());
            p.invoke_mark_deceased();
            p.set_md_cal(1);
            p.set_md_d("12".into());
            p.set_md_m("6".into());
            p.set_md_y("2024".into());
            p.set_n_by("Anh Tuấn (cháu)".into());
            p.set_n_place("Tại nhà riêng".into());
            p.invoke_form_changed();
        })));
    }
    screens.push(("nt-15-cap-nhat-xac-nhan", Rc::new(|ui: &AppWindow| People::get(ui).invoke_save_form())));
    {
        let ids = ids.clone();
        screens.push(("nt-16-su-kien", Rc::new(move |ui: &AppWindow| {
            let p = People::get(ui);
            p.invoke_dlg_dismiss();
            p.invoke_go("ho-so".into());
            p.invoke_open_person(ids.borrow().2.as_str().into());
            p.invoke_add_event();
            p.set_ev_type(1);
            p.set_ev_title("Ngày cưới anh Cường".into());
            p.set_ed_cal(1);
            p.set_ed_d("10".into());
            p.set_ed_m("3".into());
            p.set_ed_y("2018".into());
            p.set_ev_recurring(true);
            p.invoke_form_changed();
        })));
    }
    {
        let ids = ids.clone();
        screens.push(("nt-17-su-kien-nguoi-da-mat", Rc::new(move |ui: &AppWindow| {
            let p = People::get(ui);
            p.invoke_go("ho-so".into());
            p.invoke_open_person(ids.borrow().0.as_str().into());
            p.invoke_add_event();
        })));
    }
    screens.push(("nt-18-sao-luu", Rc::new(|ui: &AppWindow| People::get(ui).invoke_go("sao-luu".into()))));
    screens.push(("nt-19-hop-xuat-ics", Rc::new(|ui: &AppWindow| People::get(ui).invoke_export_ics())));
    screens.push(("nt-20-hop-tao-sao-luu", Rc::new(|ui: &AppWindow| {
        let p = People::get(ui);
        p.invoke_dlg_dismiss();
        p.invoke_create_backup();
        p.set_pw1("ngan".into());
        p.invoke_dlg_confirm();
    })));
    {
        let ids = ids.clone();
        screens.push(("nt-21-tuong-niem", Rc::new(move |ui: &AppWindow| {
            let p = People::get(ui);
            p.invoke_dlg_dismiss();
            p.invoke_go("ho-so".into());
            p.invoke_open_memorial(ids.borrow().0.as_str().into());
            // Ảnh chụp tĩnh: khói đứng yên để ảnh sáng và tối giống nhau.
            p.set_m_smoke(false);
        })));
    }
    screens.push(("nt-21b-tuong-niem-khoi-dang-chay", Rc::new(|ui: &AppWindow| People::get(ui).set_m_smoke(true))));
    {
        let ps = ps.clone();
        screens.push(("nt-20b-hop-khoi-phuc", Rc::new(move |ui: &AppWindow| {
            let p = People::get(ui);
            p.invoke_go("sao-luu".into());
            let mut g = ps.borrow_mut();
            let store = g.store.clone().unwrap();
            let bytes = store.export_backup("mật khẩu mẫu 2026".into(), crate::update::VERSION.into(), "windows".into()).unwrap();
            let b = lvhv_ffi::open_backup(bytes, "mật khẩu mẫu 2026".into()).unwrap();
            people::show_restore_preview(ui, &mut g, b);
        })));
    }
    screens.push(("nt-20c-hop-thay-toan-bo", Rc::new(|ui: &AppWindow| {
        let p = People::get(ui);
        p.set_restore_mode(1);
        p.invoke_dlg_confirm();
    })));
    {
        let ids = ids.clone();
        screens.push(("nt-22-tuong-niem-gio-hom-nay", Rc::new(move |ui: &AppWindow| {
            let p = People::get(ui);
            p.invoke_go("sap-toi".into());
            p.invoke_dlg_dismiss();
            p.invoke_open_memorial(ids.borrow().1.as_str().into());
            p.set_m_smoke(false);
        })));
    }
    // YC-2026-10 (08 §16.4 ca 4): bàn thờ Công giáo — nến · hoa · nến, không hương (chỉ đổi cách vẽ, không đổi dữ liệu).
    screens.push(("nt-22b-tuong-niem-cong-giao", Rc::new(|ui: &AppWindow| {
        let p = People::get(ui);
        p.set_m_incense(false);
        p.set_m_catholic(true);
    })));
    let big: [&str; 13] = [
        "nt-02-sap-toi", "nt-03-theo-ky-thang", "nt-05-theo-ky-nam-da-mat", "nt-07-ho-so", "nt-08-nguoi-da-mat",
        "nt-11b-them-da-mat-ket-qua", "nt-12-them-da-mat-loi", "nt-13b-them-dang-song-nhac", "nt-15-cap-nhat-xac-nhan",
        "nt-19-hop-xuat-ics", "nt-20-hop-tao-sao-luu", "nt-21-tuong-niem", "nt-02f-to-lich-the-nhac",
    ];
    let again: Vec<(&str, Act)> = screens.iter().filter(|(n, _)| *n != "nt-01-trong").cloned().collect();
    for (name, act) in screens {
        // Bước có tên bắt đầu bằng "_": chỉ chuẩn bị, không chụp (để trang kịp dàn trước khi cuộn).
        if name.starts_with('_') {
            v.push((name.to_string(), Box::new(move |ui: &AppWindow| act(ui))));
            continue;
        }
        let a1 = act.clone();
        v.push((format!("{name}-sang"), Box::new(move |ui: &AppWindow| {
            theme(ui, false);
            a1(ui);
        })));
        v.push((format!("{name}-toi"), Box::new(move |ui: &AppWindow| theme(ui, true))));
    }
    v.push(("_a3-co-chu-rat-lon".to_string(), Box::new(|ui: &AppWindow| {
        theme(ui, false);
        Theme::get(ui).set_scale(1.32);
        ui.set_size(3);
        ui.set_tab(4);
    })));
    for (name, act) in again {
        if big.contains(&name) {
            v.push((format!("a3-{name}-sang"), Box::new(move |ui: &AppWindow| act(ui))));
        } else {
            v.push((format!("_a3-{name}"), Box::new(move |ui: &AppWindow| act(ui))));
        }
    }
    // R14: khung báo chỉ xem khi chuyển đổi lược đồ lỗi (câu lấy từ people::store_open_banner). Bước cuối vì nó bỏ kho.
    {
        let ps = st.borrow().people.clone();
        v.push(("nt-23-chuyen-doi-loi-sang".to_string(), Box::new(move |ui: &AppWindow| {
            theme(ui, false);
            Theme::get(ui).set_scale(1.0);
            ui.set_size(1);
            ui.set_tab(4);
            let mut g = ps.borrow_mut();
            let e = lvhv_ffi::LvhvDataError::Storage("Lỗi lưu trữ: table meta already exists".into());
            g.store = None;
            g.store_failed = Some(people::store_open_banner(&e));
            people::go(ui, &mut g, "sap-toi");
            let p = People::get(ui);
            p.set_has_any(false);
            p.set_store_detail_open(true);
            p.set_scroll_frac(0.0);
        })));
        v.push(("nt-23-chuyen-doi-loi-toi".to_string(), Box::new(|ui: &AppWindow| theme(ui, true))));
        // Chế độ chỉ đọc (01 §7.1): chuyển đổi lỗi thật trên một kho mẫu (phiên bản lược đồ ghi sai) → lõi quay lui.
        let ps = st.borrow().people.clone();
        v.push(("nt-24-che-do-chi-doc-sang".to_string(), Box::new(move |ui: &AppWindow| {
            theme(ui, false);
            let dir = std::env::temp_dir().join(format!("lvhv-shots-chi-doc-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            {
                let s = lvhv_ffi::LvhvStore::open(dir.to_string_lossy().into_owned()).unwrap();
                seed_store(&s);
            }
            if let Ok(c) = rusqlite::Connection::open(dir.join("lvhv.sqlite")) {
                let _ = c.pragma_update(None, "user_version", 0);
            }
            let mut g = ps.borrow_mut();
            g.dir = dir;
            g.store = None;
            g.store_failed = None;
            people::go(ui, &mut g, "ho-so");
            People::get(ui).set_scroll_frac(0.0);
        })));
        v.push(("nt-24-che-do-chi-doc-toi".to_string(), Box::new(|ui: &AppWindow| theme(ui, true))));
        v.push(("nt-24b-che-do-chi-doc-ho-so-sang".to_string(), Box::new(|ui: &AppWindow| {
            theme(ui, false);
            let p = People::get(ui);
            let id = p
                .get_dead()
                .iter()
                .find(|r| r.name.contains("Minh"))
                .or_else(|| p.get_dead().row_data(0))
                .map(|r| r.id.to_string())
                .unwrap_or_default();
            p.invoke_open_person(id.into());
            p.set_v_notice_open(true);
        })));
        v.push(("nt-24b-che-do-chi-doc-ho-so-toi".to_string(), Box::new(|ui: &AppWindow| theme(ui, true))));
    }
    let _ = ui;
    v
}

/// Bắt đầu chuỗi chụp sau khi cửa sổ đã hiện. Mỗi bước: đổi trạng thái → chờ vẽ lại → chụp.
pub fn start(ui: &AppWindow, st: Rc<RefCell<State>>, dir: PathBuf) {
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("không tạo được thư mục {}: {e}", dir.display());
        std::process::exit(2);
    }
    ui.window().set_size(slint::LogicalSize::new(1180., 860.));
    let steps = Rc::new(steps(ui, st));
    let idx = Rc::new(RefCell::new(0usize));
    let phase = Rc::new(RefCell::new(false)); // false: áp dụng bước; true: chụp
    let timer = Rc::new(Timer::default());
    let w = ui.as_weak();
    let t2 = timer.clone();
    timer.start(TimerMode::Repeated, Duration::from_millis(700), move || {
        let Some(ui) = w.upgrade() else { return };
        let i = *idx.borrow();
        if i >= steps.len() {
            t2.stop();
            let _ = slint::quit_event_loop();
            return;
        }
        let (name, apply) = &steps[i];
        let snap = *phase.borrow();
        if snap && name.starts_with('_') {
            *phase.borrow_mut() = false;
            *idx.borrow_mut() = i + 1;
            return;
        }
        if !snap {
            // Kích thước cố định 1180 × 860 điểm logic ở mọi tỉ lệ: đặt lại theo điểm ảnh vật lý mỗi bước (lúc mở,
            // tỉ lệ 2× có thể chưa áp nên kích thước logic đặt sớm bị co về kích thước tối thiểu).
            let sf = ui.window().scale_factor();
            let want = slint::PhysicalSize::new((1180. * sf).round() as u32, (860. * sf).round() as u32);
            if ui.window().size() != want {
                ui.window().set_size(want);
            }
            apply(&ui);
            ui.window().request_redraw();
            *phase.borrow_mut() = true;
        } else {
            let sf = ui.window().scale_factor();
            let tag = if sf > 1.5 { "2x" } else { "1x" };
            match ui.window().take_snapshot() {
                Ok(buf) => {
                    let p = dir.join(format!("{name}-{tag}.png"));
                    match save_png(&p, buf.width(), buf.height(), buf.as_bytes()) {
                        Ok(()) => println!("đã chụp {} ({}×{})", p.display(), buf.width(), buf.height()),
                        Err(e) => {
                            eprintln!("không ghi được {}: {e}", p.display());
                            std::process::exit(3);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("không chụp được màn hình: {e}");
                    std::process::exit(4);
                }
            }
            *phase.borrow_mut() = false;
            *idx.borrow_mut() = i + 1;
        }
    });
    // Giữ bộ hẹn giờ sống suốt vòng lặp sự kiện.
    std::mem::forget(timer);
}

/// `--do-bo-nho`: gieo dữ liệu mẫu, mở Người thân rồi trang tưởng niệm và đứng yên để đo Working Set.
pub fn memory_mode(ui: &AppWindow, st: Rc<RefCell<State>>) {
    let ps = st.borrow().people.clone();
    let w = ui.as_weak();
    let t = Box::leak(Box::new(Timer::default()));
    t.start(TimerMode::SingleShot, Duration::from_millis(600), move || {
        let Some(ui) = w.upgrade() else { return };
        ui.set_tab(4);
        ui.global::<Data>().invoke_open_people();
        let (a, _, _) = seed_demo(&mut ps.borrow_mut());
        let p = People::get(&ui);
        p.invoke_go("theo-ky".into());
        p.invoke_period_kind(3);
        p.invoke_go("sap-toi".into());
        p.invoke_open_person(a.as_str().into());
        p.invoke_open_memorial(a.as_str().into());
        println!("đã mở Người thân và trang tưởng niệm (dữ liệu mẫu)");
    });
}

fn save_png(path: &Path, w: u32, h: u32, rgba: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let f = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut enc = png::Encoder::new(f, w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut wr = enc.write_header()?;
    // Ảnh chụp là nền đặc; ép kênh alpha = 255 để PNG không bị trong suốt ở trình xem ảnh.
    let mut data = rgba.to_vec();
    for px in data.chunks_exact_mut(4) {
        px[3] = 255;
    }
    wr.write_image_data(&data)?;
    Ok(())
}
