#!/usr/bin/env python3
"""Kiem tra nhat quan DE441 part-1 va part-2 trong vung thoi gian chung cua hai tep.

So kinh do bieu kien Mat Troi va (Mat Trang - Mat Troi) tinh rieng tung tep tai cac thoi diem trong vung
chung; quy sai khac ra giay theo toc do bien thien (Soc ~12.19 do/ngay, Mat Troi ~0.9856 do/ngay).
"""
import os
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import instants as I  # noqa: E402

p1, p2 = sys.argv[1], sys.argv[2]
e1 = I.Ephem([p1])
e2 = I.Ephem([p2])
lo = max(e1.kernels[0][0], e2.kernels[0][0]) + 1
hi = min(e1.kernels[0][1], e2.kernels[0][1]) - 1
print("vung chung TDB: %s .. %s" % (I.jd_to_iso(lo), I.jd_to_iso(hi)))
t = np.linspace(lo, hi, 25)
r1 = I.Reducer(e1).apparent_lon(t)
r2 = I.Reducer(e2).apparent_lon(t)
dsun = I.wrap180(r1["sun"] - r2["sun"]) * 3600.0
delong = I.wrap180((r1["moon"] - r1["sun"]) - (r2["moon"] - r2["sun"])) * 3600.0
print("max |dLon Mat Troi| = %.3e arcsec  ~ %.3e s" % (np.abs(dsun).max(), np.abs(dsun).max() / (0.9856 * 3600 / 86400)))
print("max |dElong|        = %.3e arcsec  ~ %.3e s" % (np.abs(delong).max(), np.abs(delong).max() / (12.19 * 3600 / 86400)))
