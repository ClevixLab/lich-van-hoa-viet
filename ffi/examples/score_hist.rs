//! Phân bố điểm chung Ngọc Hạp của ngày (2000–2100) — để chọn ngưỡng nhãn "Đánh giá".
//! Chạy: cargo run -p lvhv-ffi --release --example score_hist
fn main() {
    let mut h = std::collections::BTreeMap::new();
    let mut day = lvhv_ffi::SolarDay {
        y: 2000,
        m: 1,
        d: 1,
    };
    let mut n = 0u32;
    loop {
        let j = lvhv_ffi::day_info_json(day.y, day.m, day.d).unwrap();
        let k = j.rfind("\"score\":").unwrap();
        let s: i32 = j[k + 8..j.len() - 2].parse().unwrap();
        *h.entry(s).or_insert(0u32) += 1;
        n += 1;
        if day.y == 2100 && day.m == 12 && day.d == 31 {
            break;
        }
        day = lvhv_ffi::add_days(day, 1).unwrap();
    }
    let mut acc = 0;
    for (s, c) in h {
        acc += c;
        println!("{s:>4} {c:>6} {:>6.2}%", 100.0 * acc as f64 / n as f64);
    }
}
