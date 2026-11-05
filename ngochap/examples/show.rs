//! In kết quả xem ngày cho ba ngày đối chiếu (dùng khi soát tay).
use lvhv_ngochap::*;

fn main() {
    // (tháng âm, ngày âm, can, chi, trực, thần, nhãn)
    let days = [
        (9u8, 27u8, 9u8, 7u8, 9u8, 3u8, "2026-11-05 Quý Mùi"),
        (9, 30, 2, 10, 11, 6, "2026-11-08 Bính Tuất"),
        (10, 2, 4, 0, 1, 6, "2026-11-10 Mậu Tý"),
    ];
    for (lm, ld, can, chi, truc, than, label) in days {
        let c = DayCtx {
            lunar_month: lm,
            lunar_day: ld,
            leap: false,
            day_can: can,
            day_chi: chi,
            month_chi_tiet: 0,
            year_can: 2,
            year_chi: 6,
            truc,
            tu: 0,
            than,
            solar_month: 11,
            solar_day: 1,
        };
        let e = evaluate(&c);
        println!("== {label}\n{e:#?}");
        for a in activities() {
            let s = suitability(&c, a.id, Some(1985));
            println!(
                "  {:<13} allowed={:<5} score={:>3}  {}",
                a.id,
                s.allowed,
                s.score,
                s.reasons.join("; ")
            );
        }
    }
}
