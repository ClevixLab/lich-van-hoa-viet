//! Công cụ xuất dữ liệu đối chiếu của lvhv-core.
#![forbid(unsafe_code)]

use lvhv_core::astro::{tz_of_instant, TZ_SWITCH_JDN};
use lvhv_core::jd::{date_from_jdn, fmt_date, jd_to_datetime, jdn_from_date};
use lvhv_core::lunar::new_moons_between;
use lvhv_core::{
    activities, day_ctx_jdn_with, day_info_with, find_good_days_with, solar_terms_of_year_with,
    tet_with, DateError, Method, SolarDate, DEFAULT_METHOD, MAX_SPAN_DAYS, MAX_YEAR, MIN_YEAR,
    TERM_NAMES,
};
use std::io::{self, BufWriter, Write};
use std::process::ExitCode;
use std::time::Instant;

const USAGE: &str = "Cách dùng: lvhv-dump [--method hnd|precise|jpl] LỆNH ...  (mặc định: precise)
  lvhv-dump lunar-csv FROM TO          (ngày dạng YYYY-MM-DD)
  lvhv-dump dayinfo-jsonl FROM TO
  lvhv-dump tet FROM_YEAR TO_YEAR
  lvhv-dump terms FROM_YEAR TO_YEAR
  lvhv-dump newmoons FROM_YEAR TO_YEAR
  lvhv-dump gooddays ACT|all FROM TO [BIRTH_LUNAR_YEAR|none]
        (JSONL; \"all\" = mọi việc; khoảng dài hơn 366 ngày được chia khúc 366 ngày,
         mỗi khúc sắp riêng; BIRTH_LUNAR_YEAR là năm ÂM lịch của ngày sinh)
  lvhv-dump bench
`terms`: hnd và precise cùng mô hình Mặt Trời (docs/03 §6); jpl theo bảng DE441 (docs/03 §17).
`lunar-csv` (mọi phương pháp) là kết xuất hộp đen: ngày dương, năm/tháng/ngày âm, nhuận, Can Chi
năm/tháng/ngày, tiết khí đang hiệu lực (docs/03 §17.9).
Ngày/năm hợp lệ: 1900-01-01 … 3000-12-31.
Giờ trong `terms`/`newmoons` là giờ lịch: +07:45:40 (Bắc Kinh) trước 1929, +08:00 1929–1967,
+07:00 từ 1968-01-01 (docs/03 §16);
khi khoảng năm bắt đầu trước 1968, tên cột là `*_local` thay cho `*_utc7`.";

fn parse_date(s: &str) -> Result<i64, String> {
    let p: Vec<&str> = s.split('-').collect();
    if p.len() != 3 {
        return Err(format!("ngày không hợp lệ: {s}"));
    }
    let y: i32 = p[0].parse().map_err(|_| format!("năm không hợp lệ: {s}"))?;
    let m: u32 = p[1]
        .parse()
        .map_err(|_| format!("tháng không hợp lệ: {s}"))?;
    let d: u32 = p[2]
        .parse()
        .map_err(|_| format!("ngày không hợp lệ: {s}"))?;
    // Kiểm hợp lệ + miền bằng chính lõi (một nguồn sự thật).
    lvhv_core::solar_to_lunar(y, m, d).map_err(|e| format!("{s}: {e}"))?;
    Ok(jdn_from_date(y, m, d))
}

fn parse_year(s: &str) -> Result<i32, String> {
    let y: i32 = s.parse().map_err(|_| format!("năm không hợp lệ: {s}"))?;
    if !(MIN_YEAR..=MAX_YEAR).contains(&y) {
        return Err(format!("{s}: {}", DateError::OUT_OF_RANGE));
    }
    Ok(y)
}

/// Thời điểm theo giờ lịch (múi giờ theo `astro::tz_of_instant`), kèm độ lệch đã dùng.
fn fmt_dt(jd_ut: f64) -> String {
    let tz = tz_of_instant(jd_ut);
    match jd_to_datetime(jd_ut, tz) {
        Some((y, m, d, hh, mi, ss)) => {
            // Độ lệch đúng đến giây: +07:45:40 (giờ Bắc Kinh), +08:00, +07:00.
            let off = (tz * 3600.0).round() as i64;
            let (oh, om, os) = (off / 3600, off % 3600 / 60, off % 60);
            let off = if os == 0 {
                format!("+{oh:02}:{om:02}")
            } else {
                format!("+{oh:02}:{om:02}:{os:02}")
            };
            format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mi:02}:{ss:02}{off}")
        }
        None => "NaN".to_string(),
    }
}

/// Lỗi lõi → lỗi IO (để dùng `?` trong các hàm ghi).
fn core_err(e: DateError) -> io::Error {
    io::Error::other(e.to_string())
}

fn lunar_csv(out: &mut impl Write, mt: Method, from: i64, to: i64) -> io::Result<()> {
    use lvhv_core::canchi::can_chi_name;
    writeln!(out, "date,ly,lm,ld,leap,cc_year,cc_month,cc_day,tiet_khi")?;
    let name = |a, b| can_chi_name(a, b).ok_or_else(|| core_err(DateError::Computation));
    for j in from..=to {
        let c = day_ctx_jdn_with(mt, j).map_err(core_err)?;
        let (y, m, d) = date_from_jdn(j);
        writeln!(
            out,
            "{},{},{},{},{},{},{},{},{}",
            fmt_date(y, m, d),
            c.lunar_year,
            c.lunar_month,
            c.lunar_day,
            u8::from(c.leap),
            name(c.year_can, c.year_chi)?,
            name(c.month_can, c.month_chi)?,
            name(c.day_can, c.day_chi)?,
            TERM_NAMES[c.tiet_khi as usize]
        )?;
    }
    Ok(())
}

fn dayinfo_jsonl(out: &mut impl Write, mt: Method, from: i64, to: i64) -> io::Result<()> {
    for j in from..=to {
        let (y, m, d) = date_from_jdn(j);
        writeln!(
            out,
            "{}",
            day_info_with(mt, y, m, d).map_err(core_err)?.to_json()
        )?;
    }
    Ok(())
}

fn solar_of(j: i64) -> SolarDate {
    let (year, month, day) = date_from_jdn(j);
    SolarDate { year, month, day }
}

/// Mỗi dòng: `{"activity":ID,"birth_year":N|null,` + phần thân của `GoodDay::to_json()`.
/// (Khóa JSON `birth_year` giữ nguyên tên để tệp vàng ổn định; giá trị là năm ÂM lịch.)
/// Khoảng dài hơn `MAX_SPAN_DAYS` được chia thành các khúc liên tiếp, mỗi khúc một lời gọi lõi.
fn gooddays_jsonl(
    out: &mut impl Write,
    mt: Method,
    act: &str,
    from: i64,
    to: i64,
    birth: Option<i32>,
) -> io::Result<()> {
    let ids: Vec<&str> = if act == "all" {
        activities().iter().map(|a| a.id).collect()
    } else {
        vec![act]
    };
    let by = birth.map_or("null".to_string(), |y| y.to_string());
    for id in ids {
        let mut a = from;
        while a <= to {
            let b = (a + MAX_SPAN_DAYS).min(to);
            let list =
                find_good_days_with(mt, id, solar_of(a), solar_of(b), birth).map_err(core_err)?;
            for g in list {
                let j = g.to_json();
                writeln!(
                    out,
                    "{{\"activity\":\"{id}\",\"birth_year\":{by},{}",
                    &j[1..]
                )?;
            }
            a = b + 1;
        }
    }
    Ok(())
}

fn tet_csv(out: &mut impl Write, mt: Method, a: i32, b: i32) -> io::Result<()> {
    writeln!(out, "lunar_year,tet_date")?;
    for y in a..=b {
        let t = tet_with(mt, y).map_err(core_err)?;
        writeln!(out, "{},{}", y, fmt_date(t.year, t.month, t.day))?;
    }
    Ok(())
}

fn terms_csv(out: &mut impl Write, mt: Method, a: i32, b: i32) -> io::Result<()> {
    if jdn_from_date(a, 1, 1) < TZ_SWITCH_JDN {
        writeln!(out, "year,term_name,date_local,datetime_local")?;
    } else {
        writeln!(out, "year,term_name,date_utc7,datetime_utc7")?;
    }
    for y in a..=b {
        for t in solar_terms_of_year_with(mt, y).map_err(core_err)? {
            let (yy, mm, dd) = date_from_jdn(t.local_jdn());
            writeln!(
                out,
                "{},{},{},{}",
                y,
                t.name(),
                fmt_date(yy, mm, dd),
                fmt_dt(t.jd_ut)
            )?;
        }
    }
    Ok(())
}

fn newmoons_csv(out: &mut impl Write, mt: Method, a: i32, b: i32) -> io::Result<()> {
    let start = jdn_from_date(a, 1, 1);
    if start < TZ_SWITCH_JDN {
        writeln!(out, "k,datetime_local")?;
    } else {
        writeln!(out, "k,datetime_utc7")?;
    }
    let end = jdn_from_date(b, 12, 31);
    for nm in new_moons_between(mt, start, end).map_err(core_err)? {
        writeln!(out, "{},{}", nm.k, fmt_dt(nm.jd_ut))?;
    }
    Ok(())
}

fn stats(name: &str, mut ns: Vec<u128>) {
    ns.sort_unstable();
    let n = ns.len();
    let mean = ns.iter().sum::<u128>() as f64 / n as f64;
    let p50 = ns[n / 2];
    let p99 = ns[(n * 99) / 100];
    let max = ns[n - 1];
    println!(
        "{name}: n={n}, trung bình={:.2} µs, p50={:.2} µs, p99={:.2} µs, max={:.2} µs",
        mean / 1000.0,
        p50 as f64 / 1000.0,
        p99 as f64 / 1000.0,
        max as f64 / 1000.0
    );
}

fn bench(mt: Method) {
    println!("phương pháp: {mt:?}");
    // day_info: mọi ngày của năm 2026 (kèm to_json)
    let mut v = Vec::new();
    let mut sink = 0usize;
    for j in jdn_from_date(2026, 1, 1)..jdn_from_date(2027, 1, 1) {
        let (y, m, d) = date_from_jdn(j);
        let t = Instant::now();
        let info = day_info_with(mt, y, m, d).expect("ngày trong miền");
        let s = info.to_json();
        v.push(t.elapsed().as_nanos());
        sink += s.len();
    }
    stats("day_info + to_json (2026, 365 ngày)", v);

    // find_good_days: 90 ngày, mọi việc, có và không có năm sinh (P2 < 50 ms)
    let mut v = Vec::new();
    let a = jdn_from_date(2026, 11, 1);
    for act in activities() {
        for birth in [None, Some(1976)] {
            let t = Instant::now();
            let r = find_good_days_with(mt, act.id, solar_of(a), solar_of(a + 89), birth)
                .expect("khoảng hợp lệ");
            v.push(t.elapsed().as_nanos());
            sink += r.len();
        }
    }
    stats("find_good_days (90 ngày, 16 việc × 2)", v);

    let mut v = Vec::new();
    let t_all = Instant::now();
    for j in jdn_from_date(2000, 1, 1)..=jdn_from_date(3000, 12, 31) {
        let (y, m, d) = date_from_jdn(j);
        let t = Instant::now();
        let l = lvhv_core::solar_to_lunar_with(mt, y, m, d).expect("ngày trong miền");
        v.push(t.elapsed().as_nanos());
        sink += l.day as usize;
    }
    let total = t_all.elapsed();
    stats("solar_to_lunar (2000–3000)", v);
    println!(
        "solar_to_lunar tổng thời gian: {:.2} s",
        total.as_secs_f64()
    );
    println!("(sink={sink})");
}

fn run(args: &[String]) -> Result<(), String> {
    let (mt, args) = match args {
        [f, v, rest @ ..] if f == "--method" => {
            let mt = match v.as_str() {
                "hnd" => Method::Hnd,
                "precise" => Method::Precise,
                "jpl" => Method::Jpl,
                _ => return Err(format!("phương pháp không hợp lệ: {v}\n{USAGE}")),
            };
            (mt, rest)
        }
        _ => (DEFAULT_METHOD, args),
    };
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    let io_err = |e: io::Error| e.to_string();
    match args {
        [c, a, b] if c == "lunar-csv" => {
            lunar_csv(&mut out, mt, parse_date(a)?, parse_date(b)?).map_err(io_err)
        }
        [c, a, b] if c == "dayinfo-jsonl" => {
            dayinfo_jsonl(&mut out, mt, parse_date(a)?, parse_date(b)?).map_err(io_err)
        }
        [c, a, b] if c == "tet" => {
            tet_csv(&mut out, mt, parse_year(a)?, parse_year(b)?).map_err(io_err)
        }
        [c, a, b] if c == "terms" => {
            terms_csv(&mut out, mt, parse_year(a)?, parse_year(b)?).map_err(io_err)
        }
        [c, a, b] if c == "newmoons" => {
            newmoons_csv(&mut out, mt, parse_year(a)?, parse_year(b)?).map_err(io_err)
        }
        [c, act, a, b, rest @ ..] if c == "gooddays" && rest.len() <= 1 => {
            if act != "all" && !activities().iter().any(|x| x.id == act.as_str()) {
                let ids: Vec<&str> = activities().iter().map(|x| x.id).collect();
                return Err(format!("việc không hợp lệ: {act} (có: {})", ids.join(", ")));
            }
            let birth = match rest.first().map(String::as_str) {
                None | Some("none") => None,
                // Năm âm của ngày sinh: số nguyên bất kỳ (chỉ dùng để tính Can Chi), không giới
                // hạn 2000–3000.
                Some(y) => Some(
                    y.parse()
                        .map_err(|_| format!("năm sinh không hợp lệ: {y}"))?,
                ),
            };
            gooddays_jsonl(&mut out, mt, act, parse_date(a)?, parse_date(b)?, birth).map_err(io_err)
        }
        [c] if c == "jpl-days-ts" => {
            let s = lvhv_core::jpl::days_ts_source().ok_or("bảng Jpl nhúng hỏng")?;
            out.write_all(s.as_bytes()).map_err(io_err)
        }
        [c] if c == "bench" => {
            drop(out);
            bench(mt);
            Ok(())
        }
        _ => Err(USAGE.to_string()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}
