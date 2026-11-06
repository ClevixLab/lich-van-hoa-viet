//! Công cụ xuất dữ liệu kiểm định của lvhv-memorial (docs/v0.2/02 Phụ lục A).
#![forbid(unsafe_code)]

use lvhv_core::{Method, DEFAULT_METHOD};
use lvhv_memorial::dump::*;
use lvhv_memorial::model::*;
use lvhv_memorial::occ::OccClass;
use lvhv_memorial::remind::Zone;
use lvhv_memorial::{ics, Cal};
use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

const USAGE: &str = "Cách dùng: lvhv-memorial-dump [--method hnd|precise] LỆNH ...  (mặc định: precise)
  gio-dates solar FROM TO [N]          mọi ngày mất dương FROM…TO (YYYY-MM-DD), N năm giỗ (mặc định 10)
  gio-dates lunar FROM_Y TO_Y [N]      mọi ngày mất âm có năm (mọi tháng nhuận, mọi ngày 30), N năm giỗ
  gio-dates unknown FROM_Y TO_Y        mọi (tháng, ngày 1–30, nhuận) không rõ năm × Y = FROM_Y…TO_Y
  gio-dates list FILE [N]              mỗi dòng của FILE một DateValue JSON (ngày mất), N năm giỗ
      (JSONL, docs/v0.2/04 R1; tùy chọn --gio-het 2|3 đứng trước gio-dates, mặc định 2)
  reminders configs                    r3-configs.csv: 140 544 cấu hình R3
  reminders DATASET.json [FROM TO]     R3: mọi lời nhắc của mọi cấu hình (CSV), cửa sổ mặc định 2026-01-01…2036-12-31
  upcoming DATASET.json FROM TO        R11: danh sách Sắp tới 30 ngày cho mỗi hôm nay FROM…TO (CSV)
  r10 a|b|c|d                          R10: (a) sinh 1900-01-01…2026-12-31, (b) Can Chi 1899–3000,
                                       (c) 10 000 cặp hạt 20261106, (d) 29/02 1900–3000
  r12 DATASET.json [--steps]           R12: mô phỏng 10/50/200 hồ sơ × 3 kịch bản (CSV)
  ics DATASET.json TODAY DTSTAMP       tệp .ics (DTSTAMP dạng YYYYMMDDTHHMMSSZ)
  derive DATASET.json                  tính lại trường dẫn xuất (JSON thụt lề)";

fn parse_ymd(s: &str) -> Result<Ymd, String> {
    Ymd::parse(s).ok_or_else(|| format!("ngày không hợp lệ: {s}"))
}
fn parse_i32(s: &str) -> Result<i32, String> {
    s.parse().map_err(|_| format!("số không hợp lệ: {s}"))
}
fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}
fn load(path: &str) -> Result<Dataset, String> {
    parse_dataset(&read(path)?).map_err(|e| format!("{path}: {e}"))
}

struct Out<W: Write> {
    w: W,
    buf: String,
}
impl<W: Write> Out<W> {
    fn flush_if(&mut self) -> Result<(), String> {
        if self.buf.len() > 1 << 20 {
            self.flush()?;
        }
        Ok(())
    }
    fn flush(&mut self) -> Result<(), String> {
        self.w
            .write_all(self.buf.as_bytes())
            .map_err(|e| e.to_string())?;
        self.buf.clear();
        Ok(())
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let mut args: Vec<String> = args.to_vec();
    let mut method = DEFAULT_METHOD;
    let mut gio_het = DEFAULT_GIO_HET_NAM;
    loop {
        match args.first().map(String::as_str) {
            Some("--method") if args.len() > 1 => {
                method = match args[1].as_str() {
                    "hnd" => Method::Hnd,
                    "precise" => Method::Precise,
                    v => return Err(format!("phương pháp không hợp lệ: {v}")),
                };
                args.drain(..2);
            }
            Some("--gio-het") if args.len() > 1 => {
                gio_het = match args[1].as_str() {
                    "2" => 2,
                    "3" => 3,
                    v => return Err(format!("--gio-het phải là 2 hoặc 3: {v}")),
                };
                args.drain(..2);
            }
            _ => break,
        }
    }
    let cal = Cal::get(method);
    let stdout = io::stdout();
    let mut o = Out {
        w: BufWriter::new(stdout.lock()),
        buf: String::new(),
    };
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    match a.as_slice() {
        ["gio-dates", "solar", f, t, rest @ ..] if rest.len() <= 1 => {
            let n = rest.first().map_or(Ok(10), |x| parse_i32(x))?;
            let (f, t) = (parse_ymd(f)?, parse_ymd(t)?);
            for j in f.jdn()..=t.jdn() {
                let d = Ymd::from_jdn(j);
                gio_lines_for(
                    cal,
                    &mut o.buf,
                    &DateValue::solar(d.y, d.m, d.d),
                    n,
                    (0, -1),
                    gio_het,
                );
                o.flush_if()?;
            }
        }
        ["gio-dates", "lunar", f, t, rest @ ..] if rest.len() <= 1 => {
            let n = rest.first().map_or(Ok(10), |x| parse_i32(x))?;
            for ly in parse_i32(f)?..=parse_i32(t)? {
                for (m, d, leap) in lunar_days_of_year(cal, ly) {
                    gio_lines_for(
                        cal,
                        &mut o.buf,
                        &DateValue::lunar(Some(ly), m, d, leap),
                        n,
                        (0, -1),
                        gio_het,
                    );
                }
                o.flush_if()?;
            }
        }
        ["gio-dates", "unknown", f, t] => {
            let r = (parse_i32(f)?, parse_i32(t)?);
            for m in 1..=12 {
                for d in 1..=30 {
                    for leap in [false, true] {
                        gio_lines_for(
                            cal,
                            &mut o.buf,
                            &DateValue::lunar(None, m, d, leap),
                            0,
                            r,
                            gio_het,
                        );
                        o.flush_if()?;
                    }
                }
            }
        }
        ["gio-dates", "list", file, rest @ ..] if rest.len() <= 1 => {
            let n = rest.first().map_or(Ok(10), |x| parse_i32(x))?;
            for (i, line) in read(file)?.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                let v = lvhv_memorial::json::parse(line)
                    .map_err(|e| format!("dòng {}: {}", i + 1, e.msg))?;
                let dv = parse_date_value(&v).map_err(|e| format!("dòng {}: {e}", i + 1))?;
                gio_lines_for(cal, &mut o.buf, &dv, n, (1900, 3000), gio_het);
                o.flush_if()?;
            }
        }
        ["reminders", "configs"] => {
            o.buf.push_str(R3_CONFIGS_HEADER);
            for c in r3_configs() {
                r3_config_line(&mut o.buf, &c);
            }
        }
        ["reminders", file, rest @ ..] if rest.is_empty() || rest.len() == 2 => {
            let ds = load(file)?;
            let (f, t) = match rest {
                [f, t] => (parse_ymd(f)?, parse_ymd(t)?),
                _ => (Ymd::new(2026, 1, 1), Ymd::new(2036, 12, 31)),
            };
            let tz = Zone::vietnam();
            let now = r3_now(&tz);
            let by_class: Vec<(OccClass, Vec<_>)> = OccClass::R3
                .iter()
                .map(|&c| (c, class_occurrences(cal, &ds, c, f, t)))
                .collect();
            o.buf.push_str(R3_REMINDERS_HEADER);
            for c in r3_configs() {
                let occs = &by_class
                    .iter()
                    .find(|(k, _)| *k == c.class)
                    .expect("lớp R3")
                    .1;
                for r in r3_reminders(&ds, occs, &c, now, &tz) {
                    reminder_line(&mut o.buf, c.index, &r);
                }
                o.flush_if()?;
            }
        }
        ["upcoming", file, f, t] => {
            let ds = load(file)?;
            o.buf.push_str(R11_HEADER);
            for j in parse_ymd(f)?.jdn()..=parse_ymd(t)?.jdn() {
                upcoming_lines(cal, &ds, Ymd::from_jdn(j), &mut o.buf);
                o.flush_if()?;
            }
        }
        ["r10", "a"] => {
            o.buf.push_str(R10A_HEADER);
            for j in Ymd::new(1900, 1, 1).jdn()..=Ymd::new(2026, 12, 31).jdn() {
                r10a_line(cal, &mut o.buf, Ymd::from_jdn(j));
                o.flush_if()?;
            }
        }
        ["r10", "b"] => {
            o.buf.push_str(R10B_HEADER);
            for y in 1899..=3000 {
                r10b_line(&mut o.buf, y);
            }
        }
        ["r10", "c"] => {
            o.buf.push_str(R10C_HEADER);
            r10c_lines(cal, &mut o.buf, 10_000, 20_261_106);
        }
        ["r10", "d"] => {
            o.buf.push_str(R10D_HEADER);
            for y in 1900..=3000 {
                for r in [Feb29Rule::Feb28, Feb29Rule::Mar1] {
                    r10d_line(&mut o.buf, y, r);
                }
            }
        }
        ["r12", file, rest @ ..] if rest.is_empty() || rest == ["--steps"] => {
            let ds = load(file)?;
            let mut steps = String::new();
            o.buf.push_str(R12_HEADER);
            let want_steps = !rest.is_empty();
            r12_lines(
                cal,
                &ds,
                &[10, 50, 200],
                &mut o.buf,
                want_steps.then_some(&mut steps),
            );
            if want_steps {
                o.buf.push_str(R12_STEPS_HEADER);
                o.buf.push_str(&steps);
            }
        }
        ["ics", file, today, stamp] => {
            let ds = load(file)?;
            let opt = ics::IcsOptions {
                today: parse_ymd(today)?,
                dtstamp: stamp.to_string(),
                only_ids: None,
            };
            o.buf.push_str(&ics::generate(cal, &ds, &opt));
        }
        ["derive", file] => {
            let v = lvhv_memorial::json::parse(&read(file)?)
                .map_err(|e| format!("{file}: {}", e.msg))?;
            let d = derive_document(cal, &v)?;
            d.write_pretty(&mut o.buf, 0);
            o.buf.push('\n');
        }
        _ => return Err(USAGE.to_string()),
    }
    o.flush()?;
    o.w.flush().map_err(|e| e.to_string())
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
