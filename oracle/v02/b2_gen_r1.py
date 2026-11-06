#!/usr/bin/env python3
"""Bo sung B2 muc 3 — R1 chay lai voi phuong phap Jpl.

Cung quy tac giolib (save_death, anniversary, label_of) nhu lan R1 truoc; doi lich bang lich DE441 cua ben kiem
dinh (giolib.Calendar("de441") = oracle/de441/out/days_1900_3000.csv.gz). Dong JSONL viet thang theo dinh dang
cong khai cua 02 Phu luc A.1 (truong `death` la DateValue, thu tu khoa calendar, precision, year, month, day, leap)
de so tung byte sau khi sap.

  out/r1_jpl_a.jsonl.gz  (a) moi ngay mat duong 1900-01-01..3000-12-31, Y = nam am mat + 1..+10
  out/r1_jpl_b.jsonl.gz  (b) thang 1..12 x ngay 1..30 x leap {false,true}, khong ro nam, Y = 1900..3000
  out/r1_jpl_c.jsonl.gz  (c) moi ngay am ton tai nam am 1900..2999, 10 nam gio
  out/r1_jpl_d.jsonl     (d) ca co dinh (gen.FIXED), kem truong "case"
  out/b2_r1_jpl_summary.json
"""
import datetime as dt
import gzip
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import giolib as G  # noqa: E402
from gen import FIXED, EXPECT  # noqa: E402

OUT = os.path.join(HERE, "out")


def J(o):
    return json.dumps(o, ensure_ascii=False, separators=(",", ":"))


def dv(cal_, prec, y, m, d, leap):
    return {"calendar": cal_, "precision": prec, "year": y, "month": m, "day": d, "leap": leap}


def line(death, a):
    Y, m, d = a["lunar"]
    S = a["solar"]
    return J({"death": death, "Y": Y, "lunar": {"y": Y, "m": m, "d": d},
              "solar": {"y": S.year, "m": S.month, "d": S.day}, "n": a["n"], "label": a["label"]})


def main():
    cal = G.Calendar("de441")
    summ = {}
    # (a)
    c = 0
    with gzip.open(os.path.join(OUT, "r1_jpl_a.jsonl.gz"), "wt", encoding="utf-8", compresslevel=3) as f:
        for o in range(G.FIRST.toordinal(), G.LAST.toordinal() + 1):
            day = dt.date.fromordinal(o)
            inp = dv("solar", "day", day.year, day.month, day.day, False)
            death = G.save_death(cal, inp)
            yd = death["lunar"]["year"]
            for Y in range(yd + 1, yd + 11):
                a = G.anniversary(cal, death["lunar"], Y)
                if a is not None:
                    f.write(line(inp, a) + "\n")
                    c += 1
    summ["a"] = c
    # (b)
    c = 0
    with gzip.open(os.path.join(OUT, "r1_jpl_b.jsonl.gz"), "wt", encoding="utf-8", compresslevel=3) as f:
        for m in range(1, 13):
            for d in range(1, 31):
                for leap in (False, True):
                    inp = dv("lunar", "month_day", None, m, d, leap)
                    death = G.save_death(cal, inp)
                    for Y in range(1900, 3001):
                        a = G.anniversary(cal, death["lunar"], Y)
                        if a is not None:
                            f.write(line(inp, a) + "\n")
                            c += 1
    summ["b"] = c
    # (c)
    c = nd = 0
    with gzip.open(os.path.join(OUT, "r1_jpl_c.jsonl.gz"), "wt", encoding="utf-8", compresslevel=3) as f:
        for (ly, lm, ld, leap) in cal.lunar_dates(1900, 2999):
            inp = dv("lunar", "day", ly, lm, ld, leap)
            death = G.save_death(cal, inp)
            nd += 1
            for Y in range(ly + 1, ly + 11):
                a = G.anniversary(cal, death["lunar"], Y)
                if a is not None:
                    f.write(line(inp, a) + "\n")
                    c += 1
    summ["c"] = c
    summ["c_deaths"] = nd
    # (d)
    lines, got = [], {}
    for code, inp, Ys, het in FIXED:
        death = G.save_death(cal, inp)
        for Y in Ys:
            a = G.anniversary(cal, death["lunar"], Y, het)
            got[(code, Y)] = a
            if a is None:
                continue
            o = {"case": code, **json.loads(line(inp, a))}
            if het != 2:
                o["gio_het_nam"] = het
            lines.append(J(o))
    with open(os.path.join(OUT, "r1_jpl_d.jsonl"), "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    bad = []
    for code, Y, lun, sol, n, lab in EXPECT:
        a = got.get((code, Y))
        if a is None:
            bad.append([code, Y, None])
            continue
        if lun is not None and (a["lunar"] != lun or (a["solar"].year, a["solar"].month, a["solar"].day) != sol):
            bad.append([code, Y, list(a["lunar"]), a["solar"].isoformat()])
        if a["n"] != n or a["label"] != lab:
            bad.append([code, Y, a["n"], a["label"]])
    summ["d"] = len(lines)
    summ["d_self_check_vs_02_examples_bad"] = bad
    with open(os.path.join(OUT, "b2_r1_jpl_summary.json"), "w", encoding="utf-8") as f:
        json.dump(summ, f, ensure_ascii=False, indent=1)
    print(summ)


if __name__ == "__main__":
    main()
