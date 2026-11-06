#!/usr/bin/env python3
"""Kiem tra nhat quan hai doan DE441 (truoc/sau 1969) trong vung thoi gian chung cua chung.

So kinh do bieu kien Mat Troi va (Mat Trang - Mat Troi) tinh rieng tung doan tai cac thoi diem trong vung
chung; quy sai khac ra giay theo toc do bien thien (Soc ~12.19 do/ngay, Mat Troi ~0.9856 do/ngay).
Dung: overlap_check.py de441.bsp [part-2.bsp]
"""
import copy
import os
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import instants as I  # noqa: E402

base = I.Ephem(sys.argv[1:])
nseg = min(len(v[0]) for v in base.segs.values())
print("so doan moi cap than:", {k: len(v[0]) for k, v in base.segs.items()})
for p, (lst, sp) in base.segs.items():
    for s in lst:
        print("  %s doan %s .. %s" % (p, I.jd_to_iso(s.start_jd)[:10], I.jd_to_iso(s.end_jd)[:10]))
if nseg < 2:
    print("chi co mot doan; khong co vung chung")
    sys.exit(0)
e = []
for j in (0, 1):
    x = copy.copy(base)
    x.segs = {k: ([v[0][j]], np.array([])) for k, v in base.segs.items()}
    e.append(x)
lo = max(v[0][1].start_jd for v in base.segs.values()) + 1
hi = min(v[0][0].end_jd for v in base.segs.values()) - 1
print("vung chung TDB: %s .. %s" % (I.jd_to_iso(lo), I.jd_to_iso(hi)))
t = np.linspace(lo, hi, 25)
r1 = I.Reducer(e[0]).apparent_lon(t)
r2 = I.Reducer(e[1]).apparent_lon(t)
dsun = I.wrap180(r1["sun"] - r2["sun"]) * 3600.0
delong = I.wrap180((r1["moon"] - r1["sun"]) - (r2["moon"] - r2["sun"])) * 3600.0
print("max |dLon Mat Troi| = %.3e arcsec  ~ %.3e s" % (np.abs(dsun).max(), np.abs(dsun).max() / (0.9856 * 3600 / 86400)))
print("max |dElong|        = %.3e arcsec  ~ %.3e s" % (np.abs(delong).max(), np.abs(delong).max() / (12.19 * 3600 / 86400)))
