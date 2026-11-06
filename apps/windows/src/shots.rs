//! Chế độ chụp màn hình (`LichViet.exe --chup-man-hinh <thư mục>`): mở lần lượt các màn hình chính,
//! chụp nội dung cửa sổ bằng `Window::take_snapshot()` của Slint (vẽ lại bằng chính renderer của ứng
//! dụng), ghi PNG rồi thoát. CI Windows dùng để đính kèm ảnh giao diện; trên Linux dùng với Xvfb.
//! Không đọc/ghi gì khác; khi chạy thường (không có cờ) mô-đun này không làm gì.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use slint::{ComponentHandle, Timer, TimerMode};

use crate::ui::*;
use crate::State;

type Step = (&'static str, fn(&AppWindow));

fn steps() -> Vec<Step> {
    vec![
        ("1-to-lich", |_| {}),
        ("2-chi-tiet", |ui| {
            ui.global::<Data>().set_details_open(true);
        }),
        ("2b-chi-tiet-cuon", |ui| {
            ui.global::<Data>().set_scroll_frac(1.0);
        }),
        ("3-bang-thang", |ui| {
            let g = ui.global::<Data>();
            g.set_scroll_frac(0.0);
            g.invoke_open_month();
            ui.set_month_open(true);
        }),
        ("4-chon-ngay-tot", |ui| {
            ui.set_month_open(false);
            let g = ui.global::<Data>();
            ui.set_tab(1);
            g.set_act(2);
            g.set_range(2);
            g.invoke_recompute_good();
        }),
        ("4b-chon-ngay-tot-ket-qua", |ui| {
            ui.global::<Data>().set_scroll_frac(0.62);
        }),
        ("5-doi-ngay", |ui| {
            let g = ui.global::<Data>();
            g.set_scroll_frac(0.0);
            ui.set_tab(2);
            g.set_s_text("02/09/1945".into());
            g.invoke_convert_solar();
            g.set_l_day(15);
            g.set_l_month(8);
            g.set_l_year("2027".into());
            g.invoke_convert_lunar();
        }),
        ("6-gioi-thieu", |ui| {
            ui.set_tab(3);
        }),
        ("6b-gioi-thieu-chan-trang", |ui| {
            ui.global::<Data>().set_scroll_frac(1.0);
        }),
        ("7-toi", |ui| {
            let g = ui.global::<Data>();
            g.set_scroll_frac(0.0);
            Theme::get(ui).set_dark(true);
            ui.set_tab(0);
            g.set_details_open(false);
        }),
        ("8-cho-lon", |ui| {
            Theme::get(ui).set_dark(false);
            Theme::get(ui).set_scale(1.32);
            ui.set_size(3);
        }),
    ]
}

/// Bắt đầu chuỗi chụp sau khi cửa sổ đã hiện. Mỗi bước: đổi trạng thái → chờ vẽ lại → chụp.
pub fn start(ui: &AppWindow, _st: Rc<RefCell<State>>, dir: PathBuf) {
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("không tạo được thư mục {}: {e}", dir.display());
        std::process::exit(2);
    }
    let steps = Rc::new(steps());
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
        let (name, apply) = steps[i];
        let snap = *phase.borrow();
        if !snap {
            apply(&ui);
            ui.window().request_redraw();
            *phase.borrow_mut() = true;
        } else {
            match ui.window().take_snapshot() {
                Ok(buf) => {
                    let p = dir.join(format!("{name}.png"));
                    match save_png(&p, buf.width(), buf.height(), buf.as_bytes()) {
                        Ok(()) => println!("đã chụp {}", p.display()),
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
