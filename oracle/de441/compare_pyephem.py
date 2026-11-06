#!/usr/bin/env python3
"""Thong tin (khong phai tieu chi): so thoi diem DE441 cua ben kiem dinh voi thoi diem oracle PyEphem cu
(oracle/out/{newmoons,terms}_{1900_1999,2000_3000}.csv, cot jd_ut).

  * Thang TT: PyEphem TT = jd_ut + ephem.delta_t / 86400 (Delta T noi bo cua PyEphem 4.2.1) -> do sai khac
    thuan thien van (lich thien van + quy doi).
  * Thang UT: so truc tiep (gom ca khac biet Delta T).
  * Liet ke moi su kien ma hai ben dat khac NGAY dia phuong (cung quy tac mui gio cua oracle cu).
Dung: compare_pyephem.py <outdir cua ben kiem dinh (events.csv.gz)>
"""
import bisect
import csv
import gzip
import math
import os
import sys

import ephem

HERE = os.path.dirname(os.path.abspath(__file__))
OLD = os.path.join(HERE, "..", "out")
sys.path.insert(0, HERE)
import calendar_de441 as C  # noqa: E402


def load_mine(outdir):
    ev = {"newmoon": [], "term": []}
    with gzip.open(os.path.join(outdir, "events.csv.gz"), "rt", encoding="utf-8") as f:
        for r in csv.DictReader(f):
            tt = float(r["jd_tt"])
            ut = tt - float(r["delta_t_s"]) / 86400.0
            ev[r["kind"]].append((tt, ut, int(r["lon_deg"])))
    return ev


def load_old():
    out = []
    for kind, fn in (("newmoon", "newmoons_1900_1999.csv"), ("newmoon", "newmoons_2000_3000.csv"),
                     ("term", "terms_1900_1999.csv"), ("term", "terms_2000_3000.csv")):
        for r in csv.DictReader(open(os.path.join(OLD, fn), encoding="utf-8")):
            ut = float(r["jd_ut"])
            dt_s = ephem.delta_t(ephem.Date(ut - 2415020.0))
            out.append((kind, ut, ut + dt_s / 86400.0, int(r.get("longitude", 0)), fn))
    return out


def main(outdir):
    mine = load_mine(outdir)
    keys = {k: [x[0] for x in v] for k, v in mine.items()}
    old = load_old()
    tz_old_2000 = lambda u: 7 / 24.0  # noqa: E731  oracle 2000-3000: UTC+7 co dinh
    tz_1900 = C.make_tz(C.SWITCHES_1968["1968-01-01T00+07"], C.SWITCHES_1929["UTC8"])  # quy tac oracle cu Lan 2
    stats = {}
    diffs = []
    for kind, ut, tt, lon, fn in old:
        arr = keys[kind]
        i = bisect.bisect_left(arr, tt)
        j = min((i - 1, i), key=lambda q: abs(arr[q] - tt) if 0 <= q < len(arr) else 1e9)
        mtt, mut, mlon = mine[kind][j]
        assert abs(mtt - tt) < 0.1 and (kind == "newmoon" or mlon == lon), (kind, tt, mtt, lon, mlon)
        cent = int(C.jdn_to_date(int(tt + 0.5)).year // 100) * 100
        key = (kind, cent)
        s = stats.setdefault(key, [0, 0.0, 0.0, 0.0])
        dtt = (mtt - tt) * 86400.0
        dut = (mut - ut) * 86400.0
        s[0] += 1
        s[1] = max(s[1], abs(dtt))
        s[2] += dtt
        s[3] = max(s[3], abs(dut))
        tz = tz_1900 if "1900" in fn else tz_old_2000
        d_old = math.floor(ut + tz(ut) + 0.5)
        d_new = math.floor(mut + tz(mut) + 0.5)
        if d_old != d_new:
            so = ((ut + tz(ut) + 0.5) % 1) * 86400
            sn = ((mut + tz(mut) + 0.5) % 1) * 86400
            diffs.append((kind, lon, C.fmt(ut + tz(ut)), C.fmt(mut + tz(mut)), "%.1f" % dut, "%.1f" % dtt,
                          "%.0f" % min(so, 86400 - so), "%.0f" % min(sn, 86400 - sn)))
    with open(os.path.join(outdir, "pyephem_vs_de441_stats.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["kind", "century", "n", "max_abs_dTT_s", "mean_dTT_s", "max_abs_dUT_s"])
        for (kind, cent), (n, mx, sm, mxu) in sorted(stats.items()):
            w.writerow([kind, cent, n, "%.2f" % mx, "%.2f" % (sm / n), "%.1f" % mxu])
            print(kind, cent, n, "max|dTT|=%.2f s mean=%.2f s max|dUT|=%.1f s" % (mx, sm / n, mxu))
    with open(os.path.join(outdir, "pyephem_vs_de441_dayflips.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["kind", "lon_deg", "local_pyephem", "local_de441", "dUT_s(de441-pyephem)", "dTT_s",
                    "pyephem_secs_to_midnight", "de441_secs_to_midnight"])
        w.writerows(diffs)
    print("su kien doi ngay:", len(diffs))


if __name__ == "__main__":
    main(sys.argv[1])
