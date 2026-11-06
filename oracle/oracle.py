#!/usr/bin/env python3
"""Oracle doc lap (phong sach) cho lich am Viet Nam — "Lich va Van hoa Viet".

Phuong phap (KHAC voi loi Ho Ngoc Duc / Meeus):
  * Soc (trang moi): ephem.next_new_moon (PyEphem 4.2.1, libastro).
  * Kinh do bieu kien Mat Troi: ephem.Sun -> toa do xich dao bieu kien dia tam
    cua ngay (g_ra, g_dec) -> ephem.Ecliptic(..., epoch=ngay) (hoang dao cua ngay).
    Tiet khi tim bang chia doi (bisection) cua chinh oracle, do chinh xac ~0,05 s.
  * Delta T: PyEphem dung ham delta_t noi bo (ephem.delta_t); vi du
    2000: 63,8 s; 2100: 225 s; 2500: 1460 s; 3000: 4436 s. Xem docs/reports.
  * Mui gio co dinh UTC+7.

Quy uoc lich: theo docs/01-HOP-DONG-KIEN-TRUC.md muc 2-3, cai dat doc lap.
Ket qua tinh Soc/tiet khi duoc luu cache trong oracle/out/ (chay lai se dung cache).
"""
import csv
import datetime as dt
import json
import math
import os
import sys

import ephem

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "out")
os.makedirs(OUT, exist_ok=True)

TZ = 7.0 / 24.0               # UTC+7, ngay
DJD = 2415020.0               # ephem.Date 0 = JD 2415020.0 (1899-12-31 12:00 UT)

CAN = ["Giáp", "Ất", "Bính", "Đinh", "Mậu", "Kỷ", "Canh", "Tân", "Nhâm", "Quý"]
CHI = ["Tý", "Sửu", "Dần", "Mão", "Thìn", "Tỵ", "Ngọ", "Mùi", "Thân", "Dậu", "Tuất", "Hợi"]
# Ten tiet khi theo kinh do: chi so k -> kinh do 15*k
TERMS = ["Xuân Phân", "Thanh Minh", "Cốc Vũ", "Lập Hạ", "Tiểu Mãn", "Mang Chủng",
         "Hạ Chí", "Tiểu Thử", "Đại Thử", "Lập Thu", "Xử Thử", "Bạch Lộ",
         "Thu Phân", "Hàn Lộ", "Sương Giáng", "Lập Đông", "Tiểu Tuyết", "Đại Tuyết",
         "Đông Chí", "Tiểu Hàn", "Đại Hàn", "Lập Xuân", "Vũ Thủy", "Kinh Trập"]

FIRST_DAY = dt.date(2000, 1, 1)
LAST_DAY = dt.date(3000, 12, 31)
# Khoang tinh thien van (rong hon de bao du thang 11 truoc 2000 va sau 3000)
CALC_START = ephem.Date("1998/9/1")
CALC_END = ephem.Date("3002/3/1")


# ---------------------------------------------------------------- tien ich thoi gian
def ed_to_jd(d):
    return float(d) + DJD


def local_jdn(d):
    """So ngay Julian (JDN) cua ngay dan su UTC+7 chua thoi diem ephem.Date d."""
    return int(math.floor(ed_to_jd(d) + TZ + 0.5))


def jdn_to_date(jdn):
    return dt.date.fromordinal(jdn - 1721425)


def date_to_jdn(d):
    return d.toordinal() + 1721425


def fmt_local(d):
    """ephem.Date (UT) -> chuoi 'YYYY-MM-DD HH:MM:SS' theo UTC+7 (lam tron giay)."""
    jd = ed_to_jd(d) + TZ
    jdn = int(math.floor(jd + 0.5))
    secs = int(round((jd + 0.5 - jdn) * 86400.0))
    if secs >= 86400:
        jdn += 1
        secs -= 86400
    day = jdn_to_date(jdn)
    return "%s %02d:%02d:%02d" % (day.isoformat(), secs // 3600, (secs // 60) % 60, secs % 60)


def secs_from_local_midnight(d):
    jd = ed_to_jd(d) + TZ + 0.5
    return (jd - math.floor(jd)) * 86400.0


# ---------------------------------------------------------------- Soc
def compute_new_moons():
    path = os.path.join(OUT, "cache_newmoons.json")
    if os.path.exists(path):
        with open(path) as f:
            return json.load(f)
    res = []
    d = ephem.next_new_moon(CALC_START)
    while d < CALC_END:
        res.append(float(d))
        d = ephem.next_new_moon(ephem.Date(d + 1.0))
    with open(path + ".tmp", "w") as f:
        json.dump(res, f)
    os.replace(path + ".tmp", path)
    return res


# ---------------------------------------------------------------- Kinh do Mat Troi / tiet khi
_sun = ephem.Sun()


def sun_lon_deg(t):
    """Kinh do hoang dao bieu kien dia tam cua Mat Troi (do), xich dao/hoang dao cua ngay."""
    t = ephem.Date(t)
    _sun.compute(t, epoch=t)
    eq = ephem.Equatorial(_sun.g_ra, _sun.g_dec, epoch=t)
    return math.degrees(ephem.Ecliptic(eq, epoch=t).lon) % 360.0


def _signed(lon, target):
    return (lon - target + 180.0) % 360.0 - 180.0


def find_term(target, guess):
    """Tim t sao cho kinh do = target, bat dau tu uoc luong guess (ephem.Date float)."""
    # Hieu chinh uoc luong bang toc do trung binh
    t = guess
    for _ in range(3):
        t -= _signed(sun_lon_deg(t), target) / 0.98565
    lo, hi = t - 1.0, t + 1.0
    flo, fhi = _signed(sun_lon_deg(lo), target), _signed(sun_lon_deg(hi), target)
    if not (flo < 0 < fhi):
        lo, hi = t - 4.0, t + 4.0
        flo, fhi = _signed(sun_lon_deg(lo), target), _signed(sun_lon_deg(hi), target)
        assert flo < 0 < fhi, (target, guess)
    while hi - lo > 5e-7:          # ~0,04 s
        mid = 0.5 * (lo + hi)
        if _signed(sun_lon_deg(mid), target) < 0:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


def compute_terms():
    """Danh sach (ephem_date_float, k) voi kinh do 15k, theo thu tu thoi gian."""
    path = os.path.join(OUT, "cache_terms.json")
    if os.path.exists(path):
        with open(path) as f:
            return [tuple(x) for x in json.load(f)]
    res = []
    t = float(CALC_START)
    lon = sun_lon_deg(t)
    k = int(lon // 15.0 + 1) % 24
    guess = t + ((15.0 * k - lon) % 360.0) / 0.98565
    while True:
        tt = find_term(15.0 * k, guess)
        if tt >= float(CALC_END):
            break
        res.append((tt, k))
        k = (k + 1) % 24
        guess = tt + 15.2
    with open(path + ".tmp", "w") as f:
        json.dump(res, f)
    os.replace(path + ".tmp", path)
    return res


# ---------------------------------------------------------------- Can Chi (doc lap)
YEAR_CAN_TO_MONTH1_CAN = {  # theo hop dong muc 3
    "Giáp": "Bính", "Kỷ": "Bính", "Ất": "Mậu", "Canh": "Mậu",
    "Bính": "Canh", "Tân": "Canh", "Đinh": "Nhâm", "Nhâm": "Nhâm",
    "Mậu": "Giáp", "Quý": "Giáp",
}


def cc_year(y):
    return CAN[(y + 6) % 10] + " " + CHI[(y + 8) % 12]


def cc_month(y, m):
    ycan = CAN[(y + 6) % 10]
    c1 = CAN.index(YEAR_CAN_TO_MONTH1_CAN[ycan])
    return CAN[(c1 + m - 1) % 10] + " " + CHI[(m + 1) % 12]


def cc_day(jdn):
    return CAN[(jdn + 9) % 10] + " " + CHI[(jdn + 1) % 12]


# ---------------------------------------------------------------- Dung thang am
def build_months(nm_days, terms):
    """nm_days: danh sach JDN (UTC+7) cac ngay Soc, tang dan.
    terms: danh sach (jdn_local, k).
    Tra ve danh sach (start_jdn, lunar_year, month, leap)."""
    zq_days = sorted(j for j, k in terms if k % 2 == 0)        # trung khi: boi so 30 do
    ws_days = [(j) for j, k in terms if k == 18]                # Dong chi (270 do)
    import bisect

    def month_index_containing(day):
        return bisect.bisect_right(nm_days, day) - 1

    def has_zq(i):
        a, b = nm_days[i], nm_days[i + 1]
        p = bisect.bisect_left(zq_days, a)
        return p < len(zq_days) and zq_days[p] < b

    # Thang 11 cho tung nam duong co Dong chi
    m11 = {}
    for j in ws_days:
        y = jdn_to_date(j).year
        i = month_index_containing(j)
        if i >= 0 and i + 1 < len(nm_days):
            m11[y] = i
    months = []
    years = sorted(m11)
    for a, b in zip(years, years[1:]):
        assert b == a + 1
        i0, i1 = m11[a], m11[b]
        n = i1 - i0
        assert n in (12, 13), (a, n)
        leap_idx = None
        if n == 13:
            for i in range(1, 13):
                if not has_zq(i0 + i):
                    leap_idx = i
                    break
            assert leap_idx is not None
        cur, ly = 11, a
        months.append((nm_days[i0], a, 11, False))
        for i in range(1, n):
            if i == leap_idx:
                months.append((nm_days[i0 + i], ly, cur, True))
                continue
            cur = cur % 12 + 1
            if cur == 1:
                ly = b
            months.append((nm_days[i0 + i], ly, cur, False))
        assert cur == 10, (a, cur)
    # loai trung lap (thang 11 cua nam a duoc them o vong sau) -> moi vong chi them i0..i1-1
    return months, nm_days


def main():
    nm = compute_new_moons()
    print("new moons:", len(nm), file=sys.stderr)
    terms = compute_terms()
    print("terms:", len(terms), file=sys.stderr)

    nm_days = [local_jdn(x) for x in nm]
    assert all(b > a for a, b in zip(nm_days, nm_days[1:]))
    term_days = [(local_jdn(t), k) for t, k in terms]
    months, _ = build_months(nm_days, term_days)
    months.sort()

    first_jdn, last_jdn = date_to_jdn(FIRST_DAY), date_to_jdn(LAST_DAY)

    # ---- Soc
    with open(os.path.join(OUT, "newmoons_2000_3000.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["datetime_utc7", "jd_ut", "secs_from_midnight_utc7"])
        for x in nm:
            s = fmt_local(x)
            if "2000-01-01" <= s[:10] <= "3000-12-31":
                w.writerow([s, "%.6f" % ed_to_jd(x), "%.1f" % secs_from_local_midnight(x)])
    # ---- tiet khi
    with open(os.path.join(OUT, "terms_2000_3000.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["year", "term_name", "datetime_utc7", "longitude", "jd_ut"])
        for t, k in terms:
            s = fmt_local(t)
            if "2000-01-01" <= s[:10] <= "3000-12-31":
                w.writerow([s[:4].lstrip("0"), TERMS[k], s, 15 * k, "%.6f" % ed_to_jd(t)])
    # ---- Tet
    with open(os.path.join(OUT, "tet.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["lunar_year", "tet_date"])
        for start, ly, m, leap in months:
            if m == 1 and not leap and 2000 <= ly <= 3000:
                w.writerow([ly, jdn_to_date(start).isoformat()])

    # ---- tung ngay
    import bisect
    starts = [m[0] for m in months]
    tdays = [j for j, k in term_days]
    with open(os.path.join(OUT, "lunar_2000_3000.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["date", "ly", "lm", "ld", "leap", "cc_year", "cc_month", "cc_day", "tiet_khi"])
        for jdn in range(first_jdn, last_jdn + 1):
            i = bisect.bisect_right(starts, jdn) - 1
            start, ly, lm, leap = months[i]
            assert starts[i + 1] > jdn
            ld = jdn - start + 1
            assert 1 <= ld <= 30
            p = bisect.bisect_right(tdays, jdn) - 1
            tk = TERMS[term_days[p][1]]
            w.writerow([jdn_to_date(jdn).isoformat(), ly, lm, ld, 1 if leap else 0,
                        cc_year(ly), cc_month(ly, lm), cc_day(jdn), tk])
    print("done", file=sys.stderr)


if __name__ == "__main__":
    main()
