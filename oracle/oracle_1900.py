#!/usr/bin/env python3
"""Mo rong oracle doc lap cho 1900-01-01 -> 1999-12-31 (Bo sung A1).

Phuong phap thien van GIONG oracle.py (PyEphem: ephem.next_new_moon, kinh do bieu kien
Mat Troi tim bang chia doi). Khac biet duy nhat: mui gio lich phu thuoc thoi diem.

QUY TAC MUI GIO CUA ORACLE (do ben kiem dinh tu dat tu bang chung lich su, truoc khi
doc docs/03; xem docs/reports/BAO-CAO-KIEM-DINH-1900-1999.md muc 1):
  * Moi su kien thien van (Soc, tiet khi) co thoi diem UT t duoc quy ra ngay dan su theo
    mui gio z(t): UTC+8 neu t < SWITCH_UT, UTC+7 neu t >= SWITCH_UT.
  * SWITCH (mac dinh) = 1968-01-01 00:00 UTC+7 (= 1967-12-31 17:00 UT): hieu luc cua
    Quyet dinh 121-CP ngay 08/8/1967 ("tu 0 gio ngay 1.1.1968", theo Tran Tien Binh,
    Ban Lich Nha nuoc, dan qua danh sach thu tz@iana.org).
  * Bien the de do anh huong (--switch): 1967-08-08 00:00 UTC+8 (ngay ky quyet dinh).
  * Ngay dan su la ngay Gregory; ngay moi bat dau 00:00 theo mui gio ap dung.
Quy tac thang 11 / thang nhuan / Can Chi: nhu oracle.py (hop dong muc 2-3).
"""
import argparse
import bisect
import csv
import json
import math
import os
import sys

import ephem

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import oracle as O  # noqa: E402  (ham thien van, Can Chi, build_months)

OUT = os.path.join(HERE, "out")
CALC_START = ephem.Date("1898/9/1")
CALC_END = ephem.Date("2001/3/1")
FIRST = O.dt.date(1900, 1, 1)
LAST = O.dt.date(1999, 12, 31)

SWITCHES = {
    # ten -> thoi diem UT (ephem.Date)
    "1968-01-01": ephem.Date("1967/12/31 17:00:00"),
    "1967-08-08": ephem.Date("1967/8/7 16:00:00"),
    "never": ephem.Date("2100/1/1"),  # UTC+8 toan bo (doi chung, khong phai quy tac)
}


def compute_new_moons():
    path = os.path.join(OUT, "cache_newmoons_1898_2001.json")
    if os.path.exists(path):
        return json.load(open(path))
    res = []
    d = ephem.next_new_moon(CALC_START)
    while d < CALC_END:
        res.append(float(d))
        d = ephem.next_new_moon(ephem.Date(d + 1.0))
    json.dump(res, open(path + ".tmp", "w"))
    os.replace(path + ".tmp", path)
    return res


def compute_terms():
    path = os.path.join(OUT, "cache_terms_1898_2001.json")
    if os.path.exists(path):
        return [tuple(x) for x in json.load(open(path))]
    res = []
    t = float(CALC_START)
    lon = O.sun_lon_deg(t)
    k = int(lon // 15.0 + 1) % 24
    guess = t + ((15.0 * k - lon) % 360.0) / 0.98565
    while True:
        tt = O.find_term(15.0 * k, guess)
        if tt >= float(CALC_END):
            break
        res.append((tt, k))
        k = (k + 1) % 24
        guess = tt + 15.2
    json.dump(res, open(path + ".tmp", "w"))
    os.replace(path + ".tmp", path)
    return res


def make_tz(switch_ut):
    def tz(d):
        return 8.0 / 24.0 if d < float(switch_ut) else 7.0 / 24.0
    return tz


def local_jdn(d, tz):
    return int(math.floor(O.ed_to_jd(d) + tz(d) + 0.5))


def fmt(d, off_days):
    jd = O.ed_to_jd(d) + off_days
    jdn = int(math.floor(jd + 0.5))
    secs = int(round((jd + 0.5 - jdn) * 86400.0))
    if secs >= 86400:
        jdn += 1
        secs -= 86400
    return "%s %02d:%02d:%02d" % (O.jdn_to_date(jdn).isoformat(), secs // 3600, (secs // 60) % 60, secs % 60)


def secs_from_midnight(d, off_days):
    jd = O.ed_to_jd(d) + off_days + 0.5
    return (jd - math.floor(jd)) * 86400.0


def build(switch_name):
    tz = make_tz(SWITCHES[switch_name])
    nm = compute_new_moons()
    terms = compute_terms()
    nm_days = [local_jdn(x, tz) for x in nm]
    assert all(b > a for a, b in zip(nm_days, nm_days[1:]))
    term_days = [(local_jdn(t, tz), k) for t, k in terms]
    months, _ = O.build_months(nm_days, term_days)
    months.sort()
    return tz, nm, terms, term_days, months


def write_all(switch_name, suffix):
    tz, nm, terms, term_days, months = build(switch_name)
    lo, hi = FIRST.isoformat(), LAST.isoformat()
    with open(os.path.join(OUT, "newmoons_1900_1999%s.csv" % suffix), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["datetime_local", "utc_offset", "datetime_utc7", "jd_ut", "secs_from_midnight_local"])
        for x in nm:
            off = tz(x)
            s = fmt(x, off)
            if lo <= s[:10] <= hi:
                w.writerow([s, "+%02d:00" % round(off * 24), fmt(x, 7 / 24.0), "%.6f" % O.ed_to_jd(x),
                            "%.1f" % secs_from_midnight(x, off)])
    with open(os.path.join(OUT, "terms_1900_1999%s.csv" % suffix), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["year", "term_name", "datetime_local", "utc_offset", "longitude", "jd_ut"])
        for t, k in terms:
            off = tz(t)
            s = fmt(t, off)
            if lo <= s[:10] <= hi:
                w.writerow([s[:4], O.TERMS[k], s, "+%02d:00" % round(off * 24), 15 * k, "%.6f" % O.ed_to_jd(t)])
    with open(os.path.join(OUT, "tet_1900_1999%s.csv" % suffix), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["lunar_year", "tet_date"])
        for start, ly, m, leap in months:
            if m == 1 and not leap and 1900 <= ly <= 1999:
                w.writerow([ly, O.jdn_to_date(start).isoformat()])
    starts = [m[0] for m in months]
    tdays = [j for j, k in term_days]
    with open(os.path.join(OUT, "lunar_1900_1999%s.csv" % suffix), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["date", "ly", "lm", "ld", "leap", "cc_year", "cc_month", "cc_day", "tiet_khi"])
        for jdn in range(O.date_to_jdn(FIRST), O.date_to_jdn(LAST) + 1):
            i = bisect.bisect_right(starts, jdn) - 1
            start, ly, lm, leap = months[i]
            assert starts[i + 1] > jdn
            ld = jdn - start + 1
            assert 1 <= ld <= 30
            p = bisect.bisect_right(tdays, jdn) - 1
            w.writerow([O.jdn_to_date(jdn).isoformat(), ly, lm, ld, 1 if leap else 0,
                        O.cc_year(ly), O.cc_month(ly, lm), O.cc_day(jdn), O.TERMS[term_days[p][1]]])


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--switch", default="1968-01-01", choices=sorted(SWITCHES))
    ap.add_argument("--suffix", default="")
    a = ap.parse_args()
    write_all(a.switch, a.suffix)
    print("done", a.switch, file=sys.stderr)
