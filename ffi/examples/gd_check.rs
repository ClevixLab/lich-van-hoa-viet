//! Đối chiếu: ngày trong find_good_days có bị ghi "Kỵ" cho chính việc đó trên tờ lịch không.
//! Chạy: cargo run -p lvhv-ffi --release --example gd_check
fn main() {
    let acts: Vec<serde_json::Value> = serde_json::from_str(&lvhv_ffi::activities_json()).unwrap();
    let (mut total, mut clash, mut neg) = (0, 0, 0);
    let mut shown = 0;
    for a in &acts {
        let id = a["id"].as_str().unwrap();
        let name = a["name"].as_str().unwrap();
        let mut from = lvhv_ffi::SolarDay {
            y: 2026,
            m: 1,
            d: 1,
        };
        for _ in 0..4 {
            let to = lvhv_ffi::add_days(from, 365).unwrap();
            let v: Vec<serde_json::Value> = serde_json::from_str(
                &lvhv_ffi::find_good_days_json(id.into(), from, to, None).unwrap(),
            )
            .unwrap();
            for g in &v {
                total += 1;
                let s = &g["solar"];
                let (y, m, d) = (
                    s["y"].as_i64().unwrap() as i32,
                    s["m"].as_i64().unwrap() as i32,
                    s["d"].as_i64().unwrap() as i32,
                );
                let info: serde_json::Value =
                    serde_json::from_str(&lvhv_ffi::day_info_json(y, m, d).unwrap()).unwrap();
                let ky = info["ngoc_hap"]["ky"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|x| x == name);
                if g["score"].as_i64().unwrap() < 0 {
                    neg += 1;
                }
                if ky {
                    clash += 1;
                    if shown < 5 {
                        shown += 1;
                        println!(
                            "{y}-{m:02}-{d:02} {id} score={} reasons={}",
                            g["score"], g["reasons"]
                        );
                    }
                }
            }
            from = lvhv_ffi::add_days(to, 1).unwrap();
        }
    }
    println!("tổng ngày trả về: {total}; trong đó tờ lịch ghi Kỵ cho chính việc đó: {clash}; điểm âm: {neg}");
}
