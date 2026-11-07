//! Chế độ chụp màn hình (`LichViet.exe --chup-man-hinh <thư mục>`): mở lần lượt các màn hình, chụp nội dung
//! cửa sổ bằng `Window::take_snapshot()` của Slint (vẽ lại bằng chính renderer của ứng dụng — Skia), ghi PNG
//! rồi thoát. Tên tệp có hậu tố tỉ lệ màn hình (`-1x`, `-2x`; đặt `SLINT_SCALE_FACTOR=2` để chụp 2×) và giao
//! diện (`-sang`, `-toi`). Mô-đun Người thân dùng một kho tạm với dữ liệu mẫu (main.rs đặt `LVHV_DATA_DIR`),
//! không chạm dữ liệu thật. Khi chạy thường (không có cờ) mô-đun này không làm gì.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use serde_json::{json, Value};
use slint::{ComponentHandle, Timer, TimerMode};

use crate::people::{self, local_today, PState};
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
    let cert = crate::photo::reencode(&demo_certificate()).unwrap();
    let cert_id = lvhv_ffi::new_uuid().unwrap();
    store
        .add_photo(cert_id.clone(), "image/jpeg".into(), cert.width, cert.height, cert.bytes)
        .unwrap();
    let notice = json!({
        "reported_by": "Chị Hoa (con gái)", "place": "Bệnh viện tỉnh Nam Định", "reason": "Tuổi cao, bệnh tim",
        "notice_date": {"calendar":"solar","precision":"day","year":2024,"month":7,"day":18,"leap":false},
        "certificate_photo_ids": [cert_id], "note": "Gia đình đã báo tin cho họ hàng.",
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
    screens.push(("nt-12-them-da-mat-loi", Rc::new(|ui: &AppWindow| {
        let p = People::get(ui);
        p.set_scroll_frac(0.0);
        p.set_f_ho_ten("".into());
        p.set_dd_m("5".into());
        p.invoke_save_form();
    })));
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
        p.invoke_form_changed();
    })));
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
    {
        let ids = ids.clone();
        screens.push(("nt-22-tuong-niem-gio-hom-nay", Rc::new(move |ui: &AppWindow| {
            let p = People::get(ui);
            p.invoke_go("sap-toi".into());
            p.invoke_open_memorial(ids.borrow().1.as_str().into());
            p.set_m_smoke(false);
        })));
    }
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
