#!/usr/bin/env python3
"""Delta T = TT - UT (giay), ben kiem dinh tu cai tu nguon cong bo.

Nguon: Morrison, Stephenson, Hohenkerk & Zawilski (2021), "Addendum 2020 to 'Measurement of the Earth's rotation:
720 BC to AD 2015'", Proc. R. Soc. A 477: 20200776 - Table S15 (v. 2020): he so da thuc bac ba tung khoang
K_i..K_{i+1}, -720.0 -> 2019.0. Tep: ref/Table-S15.2020.txt (ban sao nguyen van cua HMNAO, xem ref/SOURCES.txt).
  t = (Y - K_i)/(K_{i+1} - K_i);  DT = a0 + a1 t + a2 t^2 + a3 t^3.
Ngoai bang: parabol dai han cua cung tac gia DT = -320.0 + 32.5 u^2, u = (Y - 1825)/100.
Quy tac noi va dinh nghia nam thap phan Y: docs/03 §17.4 (xem delta_t_year, year_of_jd).
"""
import math
import os

HERE = os.path.dirname(os.path.abspath(__file__))
TABLE = os.path.join(HERE, "ref", "Table-S15.2020.txt")


def load_table(path=TABLE):
    rows = []
    for ln in open(path, encoding="utf-8"):
        p = ln.split()
        if len(p) == 7 and p[0].isdigit():
            i, k0, k1, a0, a1, a2, a3 = p
            rows.append((float(k0), float(k1), float(a0), float(a1), float(a2), float(a3)))
    # kiem: cac khoang lien tiep, lien tuc gia tri
    for r, s in zip(rows, rows[1:]):
        assert r[1] == s[0]
        end = r[2] + r[3] + r[4] + r[5]
        assert abs(end - s[2]) < 0.0015, (r, s)  # bang lam tron 3 chu so: nhay <= 0.001 s tai nut
    return rows


ROWS = load_table()
K_FIRST, K_LAST = ROWS[0][0], ROWS[-1][1]


def spline(y):
    # hang i voi K_i <= Y < K_{i+1} (docs/03 §17.4); rieng Y = K_cuoi dung hang cuoi tai t = 1
    for n, (k0, k1, a0, a1, a2, a3) in enumerate(ROWS):
        if k0 <= y < k1 or (n == len(ROWS) - 1 and y == k1):
            t = (y - k0) / (k1 - k0)
            return a0 + t * (a1 + t * (a2 + t * a3))
    raise ValueError(y)


def spline_deriv(y):
    """dDT/dY (giay/nam)."""
    for k0, k1, a0, a1, a2, a3 in ROWS:
        if k0 <= y <= k1:
            t = (y - k0) / (k1 - k0)
            return (a1 + t * (2 * a2 + 3 * t * a3)) / (k1 - k0)
    raise ValueError(y)


def parabola(y):
    u = (y - 1825.0) / 100.0
    return -320.0 + 32.5 * u * u


def parabola_deriv(y):
    return 2 * 32.5 * (y - 1825.0) / 100.0 / 100.0


# ----- quy tac noi theo docs/03 §17.4 (doc sau khi da dung pipeline DE441; tu cai lai tu van ban) -----
# (a) Y < 2019.0: bang S15.2020 (hang K_i <= Y < K_{i+1});
# (b) 2019.0 <= Y < 2800.0: Hermite bac ba noi gia tri + dao ham cua bang tai 2019.0 voi gia tri + dao ham
#     cua parabol tai 2800.0;  (c) Y >= 2800.0: parabol -320 + 32.5 u^2.
Y_JOIN_END = 2800.0


def hermite(y):
    y0, y1 = K_LAST, Y_JOIN_END
    h = y1 - y0
    s = (y - y0) / h
    p0, m0 = spline(y0), spline_deriv(y0)
    p1, m1 = parabola(y1), parabola_deriv(y1)
    h00 = 2 * s ** 3 - 3 * s ** 2 + 1
    h10 = s ** 3 - 2 * s ** 2 + s
    h01 = -2 * s ** 3 + 3 * s ** 2
    h11 = s ** 3 - s ** 2
    return h00 * p0 + h10 * h * m0 + h01 * p1 + h11 * h * m1


def year_of_jd(jd_tt):
    """Nam thap phan theo docs/03 §17.4: Y = 2000 + (JD_TT - 2451544.5)/365.2425."""
    return 2000.0 + (jd_tt - 2451544.5) / 365.2425


def delta_t_year(y):
    if y < K_FIRST:
        raise ValueError(y)
    if y < K_LAST:
        return spline(y)
    if y < Y_JOIN_END:
        return hermite(y)
    return parabola(y)


def delta_t_tt(jd_tt):
    return delta_t_year(year_of_jd(jd_tt))
