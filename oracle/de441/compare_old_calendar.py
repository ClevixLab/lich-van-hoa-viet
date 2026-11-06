#!/usr/bin/env python3
"""Thong tin: lich DE441 cua ben kiem dinh so voi lich oracle PyEphem cu (oracle/out/lunar_1900_1999.csv - quy tac A2;
oracle/out/lunar_2000_3000.csv - UTC+7). Nhom ngay lech thanh doan.
Dung: compare_old_calendar.py <outdir> > out/old_oracle_vs_de441_days.txt
"""
import csv
import datetime as dt
import gzip
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
F = ["ly", "lm", "ld", "leap", "cc_year", "cc_month", "cc_day", "tiet_khi"]


def main(outdir):
    mine = {r["date"]: r for r in csv.DictReader(gzip.open(os.path.join(outdir, "days_1900_3000.csv.gz"), "rt",
                                                            encoding="utf-8"))}
    for fn in ("lunar_1900_1999.csv", "lunar_2000_3000.csv"):
        n = 0
        segs = []
        for r in csv.DictReader(open(os.path.join(HERE, "..", "out", fn), encoding="utf-8")):
            a = mine[r["date"]]
            bad = [f for f in F if a[f] != r[f]]
            if not bad:
                continue
            n += 1
            d = dt.date.fromisoformat(r["date"])
            if segs and segs[-1][1] == d - dt.timedelta(1):
                segs[-1][1] = d
                segs[-1][2] |= set(bad)
            else:
                segs.append([d, d, set(bad)])
        print("%s: %d ngay lech, %d doan" % (fn, n, len(segs)))
        for s in segs:
            print("  %s .. %s (%d ngay) %s" % (s[0], s[1], (s[1] - s[0]).days + 1, "|".join(sorted(s[2]))))


if __name__ == "__main__":
    main(sys.argv[1])
