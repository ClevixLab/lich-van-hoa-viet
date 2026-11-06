#!/usr/bin/env python3
"""Phan tich do nhay quy uoc mui gio 1900-1999 (KHONG phai quy tac cua oracle).
Tinh Tet theo cac bien the mui gio de xac dinh nam nao phan biet duoc cac quy uoc:
  A  = quy tac oracle (UTC+8 truoc 1968-01-01 00:00 UTC+7, UTC+7 sau)
  U7 = UTC+7 suot
  U8 = UTC+8 suot
  BJ = gio dia phuong Bac Kinh (116°24'E, +7:45:40) truoc 1929-01-01, roi nhu A
       (lich Trung Quoc truoc 1929 dung kinh tuyen Bac Kinh)
  HUE= gio dia phuong Hue (107°35'E, ~+7:10:20) 1900-1945 (gia thuyet lich Hiep Ky), roi nhu A
Ghi ra oracle/out/tet_variants_1900_1999.csv
"""
import csv
import os
import sys

import ephem

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import oracle as O  # noqa
import oracle_1900 as M  # noqa

SW = float(M.SWITCHES["1968-01-01"])
BJ_END = float(ephem.Date("1928/12/31 16:00:00"))
HUE_END = float(ephem.Date("1945/12/31 16:00:00"))
H = 1 / 24.0
tzs = {
    "A": lambda d: 8 * H if d < SW else 7 * H,
    "U7": lambda d: 7 * H,
    "U8": lambda d: 8 * H,
    "BJ": lambda d: (7 + 45 / 60 + 40 / 3600) * H if d < BJ_END else (8 * H if d < SW else 7 * H),
    "HUE": lambda d: (7 + 10 / 60 + 20 / 3600) * H if d < HUE_END else (8 * H if d < SW else 7 * H),
}
nm = M.compute_new_moons()
terms = M.compute_terms()
res = {}
for name, tz in tzs.items():
    nd = [M.local_jdn(x, tz) for x in nm]
    td = [(M.local_jdn(t, tz), k) for t, k in terms]
    months, _ = O.build_months(nd, td)
    res[name] = {ly: O.jdn_to_date(s).isoformat() for s, ly, m, lp in months if m == 1 and not lp and 1900 <= ly <= 1999}
with open(os.path.join(HERE, "out", "tet_variants_1900_1999.csv"), "w", newline="") as f:
    w = csv.writer(f, lineterminator="\n")
    w.writerow(["lunar_year"] + list(tzs) + ["all_equal"])
    for y in range(1900, 2000):
        vals = [res[n][y] for n in tzs]
        w.writerow([y] + vals + [int(len(set(vals)) == 1)])
for y in range(1900, 2000):
    vals = {n: res[n][y] for n in tzs}
    if len(set(vals.values())) > 1:
        print(y, vals)
