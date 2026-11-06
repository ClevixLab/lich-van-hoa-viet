#!/usr/bin/env python3
"""Ben kiem dinh doc lap - lap lich am 1900-01-01 -> 3000-12-31 tu thoi diem DE441 (TT) cua chinh minh.

Dau vao: out/instants_tt.csv.gz (instants.py). Delta T: deltat.py (tu cai tu nguon cong bo).
Quy tac mui gio (docs/02 A1, A2; docs/03 §16/§17):
  * thoi diem UT t < 1928-12-31 16:00 UT (= 1929-01-01 00:00 UTC+8): gio dia phuong Bac Kinh UT+7:45:40;
  * tiep theo den truoc moc 1968 (mac dinh 1967-12-31 16:00 UT = 1968-01-01 00:00 UTC+8): UTC+8;
  * sau do: UTC+7.
Quy tac lich (docs/01 §2): ngay am bat dau tu ngay chua Soc; thang 11 chua Dong chi; nam 13 thang -> thang
nhuan la thang dau tien sau thang 11 khong chua Trung khi (kinh do boi so 30 do); thang nhuan mang so thang
truoc no. Can Chi (docs/01 §3). Tiet khi cua mot ngay: tiet khi gan nhat co ngay bat dau (theo mui gio ap
dung) <= ngay do.

Viet moi hoan toan (khong dung PyEphem); quy tac giong oracle.py da kiem dinh truoc day.
"""
import argparse
import bisect
import csv
import datetime as dt
import gzip
import io
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import deltat  # noqa: E402

CAN = ["Giáp", "Ất", "Bính", "Đinh", "Mậu", "Kỷ", "Canh", "Tân", "Nhâm", "Quý"]
CHI = ["Tý", "Sửu", "Dần", "Mão", "Thìn", "Tỵ", "Ngọ", "Mùi", "Thân", "Dậu", "Tuất", "Hợi"]
TERMS = ["Xuân Phân", "Thanh Minh", "Cốc Vũ", "Lập Hạ", "Tiểu Mãn", "Mang Chủng",
         "Hạ Chí", "Tiểu Thử", "Đại Thử", "Lập Thu", "Xử Thử", "Bạch Lộ",
         "Thu Phân", "Hàn Lộ", "Sương Giáng", "Lập Đông", "Tiểu Tuyết", "Đại Tuyết",
         "Đông Chí", "Tiểu Hàn", "Đại Hàn", "Lập Xuân", "Vũ Thủy", "Kinh Trập"]
MONTH1_CAN = {"Giáp": "Bính", "Kỷ": "Bính", "Ất": "Mậu", "Canh": "Mậu", "Bính": "Canh",
              "Tân": "Canh", "Đinh": "Nhâm", "Nhâm": "Nhâm", "Mậu": "Giáp", "Quý": "Giáp"}

FIRST = dt.date(1900, 1, 1)
LAST = dt.date(3000, 12, 31)


def jd_of_date(d):
    return d.toordinal() + 1721424.5          # JD luc 00:00 UT cua ngay Gregory d


def jdn_to_date(jdn):
    return dt.date.fromordinal(jdn - 1721425)


def date_to_jdn(d):
    return d.toordinal() + 1721425


BJ_LMT = (7 + 45 / 60 + 40 / 3600) / 24.0
SW_1929_UT = jd_of_date(dt.date(1928, 12, 31)) + 16 / 24.0
SWITCHES_1968 = {
    "1968-01-01T00+08": jd_of_date(dt.date(1967, 12, 31)) + 16 / 24.0,
    "1968-01-01T00+07": jd_of_date(dt.date(1967, 12, 31)) + 17 / 24.0,
}


def make_tz(sw1968):
    def tz(jd_ut):
        if jd_ut < SW_1929_UT:
            return BJ_LMT
        if jd_ut < sw1968:
            return 8 / 24.0
        return 7 / 24.0
    return tz


def offstr(off):
    s = int(round(off * 86400))
    return "+%02d:%02d" % (s // 3600, (s // 60) % 60) + (":%02d" % (s % 60) if s % 60 else "")


def fmt(jd, ndig=3):
    """JD (bat ky thang) -> 'YYYY-MM-DDTHH:MM:SS.sss' (lam tron mili-giay)."""
    jdn = math.floor(jd + 0.5)
    ms = round((jd + 0.5 - jdn) * 86400000.0)
    if ms >= 86400000:
        jdn += 1
        ms -= 86400000
    s = ms / 1000.0
    hh = int(s // 3600)
    mm = int((s - hh * 3600) // 60)
    return "%sT%02d:%02d:%06.3f" % (jdn_to_date(int(jdn)).isoformat(), hh, mm, s - hh * 3600 - mm * 60)


def load_instants(path):
    nm, terms = [], []
    with gzip.open(path, "rt", encoding="utf-8") as f:
        rows = [ln for ln in f if not ln.startswith("#")]
    for r in csv.DictReader(io.StringIO("".join(rows))):
        t = float(r["jd_tt"])
        if r["kind"] == "newmoon":
            nm.append(t)
        else:
            terms.append((t, int(r["lon_deg"]) // 15))
    return nm, terms


def cc_year(y):
    return CAN[(y + 6) % 10] + " " + CHI[(y + 8) % 12]


def cc_month(y, m):
    c1 = CAN.index(MONTH1_CAN[CAN[(y + 6) % 10]])
    return CAN[(c1 + m - 1) % 10] + " " + CHI[(m + 1) % 12]


def cc_day(jdn):
    return CAN[(jdn + 9) % 10] + " " + CHI[(jdn + 1) % 12]


def build_months(nm_days, term_days):
    """nm_days: JDN dia phuong cac ngay Soc (tang dan). term_days: [(jdn, idx)] (idx: kinh do/15).
    Tra ve [(start_jdn, lunar_year, month, leap)]."""
    zq = sorted(j for j, k in term_days if k % 2 == 0)
    ws = [j for j, k in term_days if k == 18]

    def idx_containing(day):
        return bisect.bisect_right(nm_days, day) - 1

    def has_zq(i):
        a, b = nm_days[i], nm_days[i + 1]
        p = bisect.bisect_left(zq, a)
        return p < len(zq) and zq[p] < b

    m11 = {}
    for j in ws:
        i = idx_containing(j)
        if 0 <= i and i + 1 < len(nm_days):
            m11[jdn_to_date(j).year] = i
    years = sorted(m11)
    months = []
    for a, b in zip(years, years[1:]):
        assert b == a + 1
        i0, i1 = m11[a], m11[b]
        n = i1 - i0
        assert n in (12, 13), (a, n)
        leap = None
        if n == 13:
            leap = next(i for i in range(1, 13) if not has_zq(i0 + i))
        cur, ly = 11, a
        months.append((nm_days[i0], a, 11, False))
        for i in range(1, n):
            if i == leap:
                months.append((nm_days[i0 + i], ly, cur, True))
                continue
            cur = cur % 12 + 1
            if cur == 1:
                ly = b
            months.append((nm_days[i0 + i], ly, cur, False))
        assert cur == 10
    return months


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--instants", required=True)
    ap.add_argument("--outdir", required=True)
    ap.add_argument("--switch1968", default="1968-01-01T00+08", choices=sorted(SWITCHES_1968))
    ap.add_argument("--suffix", default="")
    a = ap.parse_args()
    tz = make_tz(SWITCHES_1968[a.switch1968])
    nm_tt, terms_tt = load_instants(a.instants)

    def ut_of(t):
        return t - deltat.delta_t_tt(t) / 86400.0

    def local(t):
        u = ut_of(t)
        off = tz(u)
        x = u + off + 0.5
        j = math.floor(x)
        return u, off, int(j), (x - j) * 86400.0

    nm_info = [local(t) for t in nm_tt]
    tm_info = [local(t) for t, k in terms_tt]
    nm_days = [x[2] for x in nm_info]
    assert all(b > a for a, b in zip(nm_days, nm_days[1:]))
    term_days = [(x[2], k) for x, (t, k) in zip(tm_info, terms_tt)]
    assert all(b[0] > a[0] for a, b in zip(term_days, term_days[1:]))
    months = build_months(nm_days, term_days)
    months.sort()
    os.makedirs(a.outdir, exist_ok=True)
    sfx = a.suffix

    # ---- su kien (UT, gio dia phuong, khoang cach toi nua dem gan nhat)
    with gzip.open(os.path.join(a.outdir, "events%s.csv.gz" % sfx), "wt", encoding="utf-8", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["kind", "lon_deg", "jd_tt", "delta_t_s", "ut", "utc_offset", "local", "local_date",
                    "secs_from_local_midnight", "secs_to_nearest_midnight"])
        ev = [(t, "newmoon", 0, x) for t, x in zip(nm_tt, nm_info)] + \
             [(t, "term", 15 * k, x) for (t, k), x in zip(terms_tt, tm_info)]
        ev.sort()
        for t, kind, lon, (u, off, j, s) in ev:
            w.writerow([kind, lon, "%.9f" % t, "%.3f" % deltat.delta_t_tt(t), fmt(u), offstr(off),
                        fmt(u + off), jdn_to_date(j).isoformat(), "%.3f" % s, "%.3f" % min(s, 86400 - s)])

    # ---- Tet
    with open(os.path.join(a.outdir, "tet_1900_3000%s.csv" % sfx), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["lunar_year", "tet_date"])
        for start, ly, m, leap in months:
            if m == 1 and not leap and 1900 <= ly <= 3000:
                w.writerow([ly, jdn_to_date(start).isoformat()])
    # ---- bang thang (ngay dau moi thang am)
    with open(os.path.join(a.outdir, "months_1900_3000%s.csv" % sfx), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["start_date", "lunar_year", "month", "leap"])
        for start, ly, m, leap in months:
            if date_to_jdn(FIRST) - 31 <= start <= date_to_jdn(LAST):
                w.writerow([jdn_to_date(start).isoformat(), ly, m, 1 if leap else 0])

    # ---- tung ngay
    starts = [m[0] for m in months]
    tdays = [j for j, k in term_days]
    with gzip.open(os.path.join(a.outdir, "days_1900_3000%s.csv.gz" % sfx), "wt", encoding="utf-8",
                   newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["date", "ly", "lm", "ld", "leap", "cc_year", "cc_month", "cc_day", "tiet_khi"])
        for jdn in range(date_to_jdn(FIRST), date_to_jdn(LAST) + 1):
            i = bisect.bisect_right(starts, jdn) - 1
            start, ly, lm, leap = months[i]
            assert starts[i + 1] > jdn
            ld = jdn - start + 1
            assert 1 <= ld <= 30
            p = bisect.bisect_right(tdays, jdn) - 1
            assert p >= 0
            w.writerow([jdn_to_date(jdn).isoformat(), ly, lm, ld, 1 if leap else 0,
                        cc_year(ly), cc_month(ly, lm), cc_day(jdn), TERMS[term_days[p][1]]])
    print("ok months=%d" % len(months), file=sys.stderr)


if __name__ == "__main__":
    main()
