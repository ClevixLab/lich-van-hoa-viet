//! Kiểm thử giao diện bằng backend kiểm thử của Slint: dựng cửa sổ thật (không vẽ ra màn hình),
//! bấm chuột giả lập vào đúng vị trí của từng nút như người dùng, rồi kiểm tra trạng thái đổi.
//!
//! Lỗi bản 0.1.2: SwipeGestureHandler (vuốt đổi ngày) nằm đè lên tờ lịch nên nuốt mọi cú bấm — mục
//! "Chi tiết", "Xem thêm", dải tháng… không phản hồi. `sheet_clicks_reach_buttons` bấm thật vào các nút
//! này (qua đường xử lý chuột của Slint, gồm cả bước giữ cú nhấn 100 ms của vùng vuốt/cuộn) để lỗi không
//! quay lại. Mọi kiểm thử chạy trong một hàm vì backend kiểm thử chỉ khởi tạo được một lần mỗi luồng.

use std::time::Duration;

use i_slint_backend_testing::{ElementHandle, ElementRoot};
use slint::platform::{PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, Model};

use crate::ui::*;

/// Bấm chuột trái vào giữa phần tử (nhấn — chờ 50 ms thời gian giả — nhả), như người dùng.
pub(crate) fn click(ui: &AppWindow, el: &ElementHandle) {
    let p = el.absolute_position();
    let s = el.size();
    assert!(
        s.width > 0. && s.height > 0.,
        "phần tử có kích thước 0: {:?}",
        el.accessible_label()
    );
    let pos = LogicalPosition::new(p.x + s.width / 2., p.y + s.height / 2.);
    let w = ui.window();
    w.dispatch_event(WindowEvent::PointerMoved { position: pos });
    w.dispatch_event(WindowEvent::PointerPressed {
        position: pos,
        button: PointerEventButton::Left,
    });
    i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(50));
    w.dispatch_event(WindowEvent::PointerReleased {
        position: pos,
        button: PointerEventButton::Left,
    });
    w.dispatch_event(WindowEvent::PointerExited);
    i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(20));
}

pub(crate) fn by_label(ui: &AppWindow, label: &str) -> ElementHandle {
    let all: Vec<_> = ElementHandle::find_by_accessible_label(ui, label).collect();
    assert!(!all.is_empty(), "không tìm thấy phần tử có nhãn {label:?}");
    all.into_iter().next().unwrap()
}

fn by_label_prefix(ui: &AppWindow, prefix: &str) -> ElementHandle {
    let p = prefix.to_string();
    ui.root_element()
        .query_descendants()
        .match_predicate(move |e| e.accessible_label().is_some_and(|l| l.starts_with(&p)))
        .find_first()
        .unwrap_or_else(|| panic!("không tìm thấy phần tử có nhãn bắt đầu bằng {prefix:?}"))
}

pub(crate) fn count_label(ui: &AppWindow, label: &str) -> usize {
    ElementHandle::find_by_accessible_label(ui, label).count()
}

/// Số phần tử có nhãn trợ năng bắt đầu bằng `prefix`.
pub(crate) fn ui_find_prefix(ui: &AppWindow, prefix: &str) -> usize {
    use i_slint_backend_testing::ElementRoot;
    let prefix = prefix.to_string();
    ui.root_element()
        .query_descendants()
        .match_predicate(move |e| e.accessible_label().is_some_and(|l| l.starts_with(prefix.as_str())))
        .find_all()
        .len()
}

/// Cuộn tờ lịch/trang đang mở sao cho phần tử nằm trong khung nhìn (như người dùng cuộn chuột).
pub(crate) fn scroll_into_view(ui: &AppWindow, label: &str) {
    for n in 0..80 {
        let Some(el) = ElementHandle::find_by_accessible_label(ui, label).next() else {
            // Phần tử nằm ngoài khung nhìn (bị cắt): cuộn xuống 40 lần đầu, sau đó cuộn lên.
            let pos = LogicalPosition::new(300., 400.);
            let dy = if n < 40 { -120. } else { 120. };
            ui.window().dispatch_event(WindowEvent::PointerScrolled {
                position: pos,
                delta_x: 0.,
                delta_y: dy,
            });
            i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(20));
            continue;
        };
        let y = el.absolute_position().y;
        let h = ui
            .window()
            .size()
            .to_logical(ui.window().scale_factor())
            .height;
        if y >= 0. && y + el.size().height <= h {
            return;
        }
        let pos = LogicalPosition::new(300., 400.);
        let dy = if y + el.size().height > h {
            -120.
        } else {
            120.
        };
        ui.window().dispatch_event(WindowEvent::PointerScrolled {
            position: pos,
            delta_x: 0.,
            delta_y: dy,
        });
        i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(20));
    }
    panic!("không cuộn được tới {label:?}");
}

#[test]
fn all_clickables_work() {
    i_slint_backend_testing::init_no_event_loop();
    let ui = AppWindow::new().unwrap();
    let st = crate::setup(&ui);
    // Cửa sổ cao để toàn bộ tờ lịch (kể cả phần chi tiết đã mở) nằm trong khung nhìn: ElementHandle chỉ
    // thấy phần tử không bị cắt bởi vùng cuộn. scroll_into_view vẫn cuộn khi cần (trang dài hơn).
    ui.window().set_size(slint::LogicalSize::new(1280., 2600.));
    ui.show().unwrap();
    let g = ui.global::<Data>();

    // Đặt một ngày cố định có nhiều việc "Nên làm" (> 4, có nút Xem thêm) và có sao.
    {
        let mut s = st.borrow_mut();
        let d = (2026..=2027)
            .flat_map(|y| (1..=12).flat_map(move |m| (1..=28).map(move |d| crate::D { y, m, d })))
            .find(|d| {
                let j: crate::JDay =
                    serde_json::from_str(&lvhv_ffi::day_info_json(d.y, d.m, d.d).unwrap()).unwrap();
                j.ngoc_hap.nen.len() > 4 && !j.ngoc_hap.sao_tot.is_empty()
            })
            .expect("có ngày nhiều việc nên làm");
        crate::set_date(&ui, &mut s, d);
    }
    let sheet = g.get_sheet();
    assert!(sheet.nen.row_count() > 4);

    // --- Tờ lịch: mục "Chi tiết" mở ra và thu lại khi bấm (lỗi chính của 0.1.2) ---
    assert!(!g.get_details_open());
    assert_eq!(
        count_label(&ui, &format!("Nhị thập bát tú: {}", sheet.tu)),
        0
    );
    scroll_into_view(&ui, "Mở chi tiết");
    click(&ui, &by_label(&ui, "Mở chi tiết"));
    assert!(
        g.get_details_open(),
        "bấm \"Chi tiết\" phải mở phần chi tiết"
    );
    let tu = format!("Nhị thập bát tú: {}", sheet.tu);

    assert_eq!(
        count_label(&ui, &tu),
        1,
        "phần chi tiết hiện Nhị thập bát tú"
    );
    assert_eq!(count_label(&ui, &format!("Nạp âm: {}", sheet.nap_am)), 1);
    assert_eq!(count_label(&ui, &format!("Tuổi xung: {}", sheet.xung)), 1);
    let first_star = sheet
        .sao_tot
        .row_data(0)
        .unwrap()
        .items
        .row_data(0)
        .unwrap();
    assert!(
        ui.root_element()
            .query_descendants()
            .match_predicate(
                move |e| e.accessible_value().is_some_and(|v| v == first_star)
                    || e.accessible_label().is_some_and(|v| v == first_star)
            )
            .find_first()
            .is_some(),
        "sao tốt hiện trong chi tiết"
    );
    scroll_into_view(&ui, "Thu gọn chi tiết");
    click(&ui, &by_label(&ui, "Thu gọn chi tiết"));
    assert!(!g.get_details_open(), "bấm lần nữa thì thu gọn");
    assert_eq!(count_label(&ui, &tu), 0);

    // --- "Xem thêm" việc nên làm ---
    let nen_last = sheet.nen.row_data(sheet.nen.row_count() - 1).unwrap();
    assert_eq!(
        count_label(&ui, nen_last.as_str()),
        0,
        "việc thứ 5 trở đi ẩn khi chưa bấm Xem thêm"
    );
    scroll_into_view(&ui, "Xem thêm việc nên làm");
    click(&ui, &by_label(&ui, "Xem thêm việc nên làm"));
    assert_eq!(
        count_label(&ui, nen_last.as_str()),
        1,
        "bấm Xem thêm thì hiện đủ việc"
    );
    assert_eq!(count_label(&ui, "Thu gọn việc nên làm"), 1);

    // --- Ngày sau / Ngày trước / Hôm nay ---
    let day0 = st.borrow().date;
    scroll_into_view(&ui, "Ngày sau");
    click(&ui, &by_label(&ui, "Ngày sau"));
    assert_eq!(st.borrow().date, day0.add(1).unwrap(), "nút Ngày sau");
    click(&ui, &by_label(&ui, "Ngày trước"));
    assert_eq!(st.borrow().date, day0, "nút Ngày trước");
    click(&ui, &by_label(&ui, "Hôm nay"));
    assert_eq!(st.borrow().date, st.borrow().today, "nút Hôm nay");

    // --- Bảng tháng cạnh tờ lịch: bấm ngày 9 → tờ lịch ngày 9 ---
    let d9 = crate::D {
        d: 9,
        ..st.borrow().date
    };
    click(
        &ui,
        &by_label_prefix(
            &ui,
            &format!("{} 9 tháng {},", crate::WEEKDAY[d9.weekday()], d9.m),
        ),
    );
    assert_eq!(st.borrow().date.d, 9, "bấm ô ngày 9 trên bảng tháng");
    click(&ui, &by_label(&ui, "Tháng sau"));
    let (vy, vm) = st.borrow().view;
    let d = st.borrow().date;
    assert_eq!(
        vy * 12 + vm,
        d.y * 12 + d.m + 1,
        "nút Tháng sau đổi bảng tháng"
    );

    // --- Dải tháng đầu tờ → hộp thoại Chọn ngày → chọn ngày 20 ---
    scroll_into_view(
        &ui,
        &format!("Tháng {} năm {}. Mở bảng tháng để chọn ngày khác", d.m, d.y),
    );
    click(
        &ui,
        &by_label(
            &ui,
            &format!("Tháng {} năm {}. Mở bảng tháng để chọn ngày khác", d.m, d.y),
        ),
    );
    assert!(ui.get_month_open(), "bấm dải tháng mở hộp thoại Chọn ngày");
    let want = format!(" 20 tháng {}", d.m);
    let dialog_cells: Vec<_> = ui
        .root_element()
        .query_descendants()
        .match_predicate(move |e| {
            e.accessible_label().is_some_and(|l| {
                l.split(',')
                    .next()
                    .is_some_and(|h| h.ends_with(want.as_str()))
            })
        })
        .find_all();
    assert_eq!(
        dialog_cells.len(),
        2,
        "ô ngày 20 có ở bảng cạnh và trong hộp thoại"
    );
    click(&ui, dialog_cells.last().unwrap());
    assert!(!ui.get_month_open(), "chọn ngày thì đóng hộp thoại");
    assert_eq!(st.borrow().date.d, 20);
    click(
        &ui,
        &by_label(
            &ui,
            &format!(
                "Tháng {} năm {}. Mở bảng tháng để chọn ngày khác",
                st.borrow().date.m,
                st.borrow().date.y
            ),
        ),
    );
    assert!(ui.get_month_open());
    click(&ui, &by_label(&ui, "Đóng"));
    assert!(!ui.get_month_open(), "nút Đóng");

    // --- Thanh mục: Chọn ngày tốt → chọn việc → kết quả → mở tờ lịch ---
    click(&ui, &by_label(&ui, "Chọn ngày tốt"));
    assert_eq!(ui.get_tab(), 1);
    let act0 = g.get_activities().row_data(0).unwrap();
    click(&ui, &by_label(&ui, act0.as_str()));
    assert_eq!(g.get_act(), 0, "bấm chọn việc");
    click(&ui, &by_label(&ui, "3 tháng tới"));
    assert_eq!(g.get_range(), 2);
    assert!(
        g.get_good().row_count() > 0,
        "có ngày tốt trong 3 tháng tới"
    );
    let first = g.get_good().row_data(0).unwrap();
    let row_label = format!(
        "{}, {}, điểm {}. Mở tờ lịch ngày này",
        first.l1, first.l2, first.score
    );
    scroll_into_view(&ui, &row_label);
    click(&ui, &by_label(&ui, &row_label));
    assert_eq!(ui.get_tab(), 0, "bấm kết quả mở tờ lịch");
    assert_eq!(
        (st.borrow().date.y, st.borrow().date.m, st.borrow().date.d),
        (first.y, first.m, first.d)
    );

    // --- Đổi ngày: dương → âm và mở tờ lịch ---
    click(&ui, &by_label(&ui, "Đổi ngày"));
    assert_eq!(ui.get_tab(), 2);
    g.set_s_text("02/09/2045".into());
    g.invoke_convert_solar();
    assert!(g.get_s_ok());
    click(&ui, &by_label(&ui, g.get_s_link().as_str()));
    assert_eq!(ui.get_tab(), 0, "nút Mở tờ lịch (dương → âm)");
    assert_eq!(
        st.borrow().date,
        crate::D {
            y: 2045,
            m: 9,
            d: 2
        }
    );
    click(&ui, &by_label(&ui, "Đổi ngày"));
    assert!(g.get_l_ok());
    click(&ui, &by_label(&ui, "Mở tờ lịch ngày này"));
    assert_eq!(ui.get_tab(), 0, "nút Mở tờ lịch (âm → dương)");

    // --- Giới thiệu: nút cập nhật gọi callback; Nguồn quy tắc và quay lại ---
    click(&ui, &by_label(&ui, "Giới thiệu"));
    assert_eq!(ui.get_tab(), 3);
    // Nội dung như bản web 0.2.1: ba tiêu đề nhỏ, 8 mục "Ứng dụng giúp bạn", 6 bước, mẹo về thông báo của Windows.
    for h in ["Ứng dụng giúp bạn", "Bắt đầu trong một phút", "Mẹo để không lỡ ngày nào"] {
        assert_eq!(count_label(&ui, h), 1, "{h}");
    }
    for t in ["Tờ lịch mỗi ngày", "Chọn ngày tốt", "Đổi ngày", "Người thân", "Nhắc trước", "Xem cả nhà có việc gì", "Trang tưởng niệm ngày giỗ", "Thông tin báo tử"] {
        assert!(count_label(&ui, t) >= 1, "mục {t}");
    }
    for start in ["Mở mục Người thân, chọn Hồ sơ", "Nhập họ tên, cách gọi", "Chọn nhắc trước mấy ngày", "Mở Sắp tới để xem", "Đúng ngày giỗ, bấm", "Vào Sao lưu để cất"] {
        assert_eq!(ui_find_prefix(&ui, start), 1, "bước: {start}");
    }
    assert_eq!(ui_find_prefix(&ui, "Mỗi lần bạn mở ứng dụng, các lời nhắc sắp tới (tối đa 64)"), 1);
    let called = std::rc::Rc::new(std::cell::Cell::new(false));
    let c2 = called.clone();
    g.on_check_update(move || c2.set(true));
    scroll_into_view(&ui, "Kiểm tra cập nhật / Cập nhật");
    click(&ui, &by_label(&ui, "Kiểm tra cập nhật / Cập nhật"));
    assert!(called.get(), "nút Kiểm tra cập nhật / Cập nhật");
    assert_eq!(
        count_label(&ui, "Logo Clevix"),
        2,
        "logo Clevix ở dòng Clevix Lab và ở chân trang"
    );
    assert_eq!(
        count_label(
            &ui,
            &format!(
                "Bản quyền 2026 Clevix Lab, Phiên bản {}",
                crate::update::VERSION
            )
        ),
        1,
        "chân trang bản quyền"
    );
    let running = format!("Phiên bản đang dùng: {}", crate::update::VERSION);
    assert_eq!(
        count_label(&ui, &running),
        1,
        "chỉ hiện số phiên bản, không có bản dựng"
    );
    scroll_into_view(&ui, "Nguồn quy tắc xem ngày");
    click(&ui, &by_label(&ui, "Nguồn quy tắc xem ngày"));
    assert!(g.get_about_rules());
    click(&ui, &by_label(&ui, "Về trang Giới thiệu"));
    assert!(!g.get_about_rules());

    // --- Cỡ chữ và giao diện tối ---
    click(&ui, &by_label(&ui, "Cỡ chữ rất lớn"));
    assert_eq!(ui.get_size(), 3);
    assert!((Theme::get(&ui).get_scale() - 1.32).abs() < 1e-6);
    click(&ui, &by_label_prefix(&ui, "Giao diện theo máy"));
    assert!(Theme::get(&ui).get_dark(), "bấm nút giao diện → tối");

    // --- Phím tắt ← → ---
    click(&ui, &by_label(&ui, "Tờ lịch"));
    let d0 = st.borrow().date;
    ui.window().dispatch_event(WindowEvent::KeyPressed {
        text: slint::platform::Key::RightArrow.into(),
    });
    ui.window().dispatch_event(WindowEvent::KeyReleased {
        text: slint::platform::Key::RightArrow.into(),
    });
    assert_eq!(st.borrow().date, d0.add(1).unwrap(), "phím → sang ngày sau");
}

/// TKN-W-01: ở mọi cỡ chữ (A, A+, A++ và 1,16) và bề rộng 1180 px / 900 px / tối thiểu 560 px, nút cỡ chữ, nút sáng/tối và
/// mọi mục của thanh mục nằm trọn trong cửa sổ.
#[test]
fn top_bar_fits_at_every_text_size() {
    i_slint_backend_testing::init_no_event_loop();
    let ui = AppWindow::new().unwrap();
    let _st = crate::setup(&ui);
    ui.show().unwrap();
    for width in [1180., 900., 560.] {
        ui.window().set_size(slint::LogicalSize::new(width, 900.));
        for scale in [1.0f32, 1.15, 1.16, 1.32] {
            Theme::get(&ui).set_scale(scale);
            for label in ["Cỡ chữ rất lớn", "Tờ lịch", "Người thân", "Giới thiệu"] {
                let e = by_label(&ui, label);
                let right = e.absolute_position().x + e.size().width;
                assert!(
                    e.absolute_position().x >= 0. && right <= width + 0.5,
                    "{label} tràn ở {width} px, scale {scale}: cạnh phải {right}"
                );
            }
            let t = by_label_prefix(&ui, "Giao diện theo máy");
            let right = t.absolute_position().x + t.size().width;
            assert!(right <= width + 0.5, "nút sáng/tối tràn ở {width} px, scale {scale}: {right}");
        }
    }
}
