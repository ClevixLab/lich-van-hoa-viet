#!/usr/bin/env python3
"""Oracle v0.2 — cai dat doc lap (phong sach) cac quy tac ngay gio, tuoi am, huong tho/huong duong,
Can Chi nam sinh va quy tac 29/02 cua "Lich va Van hoa Viet".

Nguoi viet: ben khong tham gia xay dung dac ta. Khong doc ma nguon core/, ngochap/, tsport/, ffi/, apps/.
Nguon quy tac: docs/v0.2/01, 02, 04, 06 (quyet dinh G1: O-2 gio het nam 2, O-3 nguong 60, O-4 nhuan -> thuong).

Doi lich (S2L / L2S / LYOB / do dai thang) KHONG tinh lai thien van o day: doc bang am lich theo tung ngay
(CSV `date,ly,lm,ld,leap,...`). Hai nguon bang:
  * "oracle": bang cua oracle v0.1 (PyEphem; oracle/oracle.py 2000-3000, oracle/oracle_1900.py 1900-1999,
    quy tac mui gio lich su A1/A2) -> lop R1-dau-cuoi.
  * "core":   tep am lich da cong bo cua loi, phuong phap Precise (core/out/precise/*.csv; chi la DU LIEU,
    khong phai ma nguon) -> lop R1-quy-tac.
Mien: ngay duong 1900-01-01 -> 3000-12-31 (moi ngay co trong bang). Ngay am ngoai bang -> None.

Can Chi nam: cai dat doc lap bang moc 1984 = Giap Ty (khong dung cong thuc (Y+6) mod 10 cua hop dong;
hai cach duoc so voi nhau trong R10(b)).
Nam nhuan Gregory: theo datetime (thu ngay 29/02) — doc lap voi cong thuc chia het.
"""
import csv
import datetime as dt
import gzip
import os

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))

TABLES = {
    "oracle": [os.path.join(ROOT, "oracle", "out", "lunar_1900_1999.csv"),
               os.path.join(ROOT, "oracle", "out", "lunar_2000_3000.csv")],
    "core": [os.path.join(ROOT, "core", "out", "precise", "lunar_1900_1999.csv"),
             os.path.join(ROOT, "core", "out", "precise", "lunar_2000_3000.csv")],
    # Bo sung B2 (2026-11-07): lich tu lap tu JPL DE441 cua ben kiem dinh (oracle/de441, L15 khop loi
    # 402 132/402 132 ngay). Dung cho R1-Jpl va R3/R11/R12/R13 (phuong phap Jpl).
    "de441": [os.path.join(ROOT, "oracle", "de441", "out", "days_1900_3000.csv.gz")],
}

FIRST = dt.date(1900, 1, 1)
LAST = dt.date(3000, 12, 31)
ORD0 = FIRST.toordinal()

CAN = ["Giáp", "Ất", "Bính", "Đinh", "Mậu", "Kỷ", "Canh", "Tân", "Nhâm", "Quý"]
CHI = ["Tý", "Sửu", "Dần", "Mão", "Thìn", "Tỵ", "Ngọ", "Mùi", "Thân", "Dậu", "Tuất", "Hợi"]

GIO_HET_NAM_MAC_DINH = 2      # O-2
NGUONG_MAC_DINH = 60          # O-3


class Calendar:
    """Bang am lich theo ngay -> S2L, L2S, LYOB, len."""

    def __init__(self, source):
        self.source = source
        self.days = []                    # chi so = ordinal - ORD0 -> (ly, lm, ld, leap)
        self.month_start = {}             # (ly, lm, leap) -> ordinal cua ngay 1
        self.month_len = {}               # (ly, lm, leap) -> so ngay thay trong bang
        self.month_complete = {}          # True neu ca thang nam trong bang
        expect = ORD0
        for path in TABLES[source]:
            opener = gzip.open if path.endswith(".gz") else open
            with opener(path, "rt", encoding="utf-8", newline="") as f:
                r = csv.reader(f)
                next(r)
                for row in r:
                    o = dt.date.fromisoformat(row[0]).toordinal()
                    assert o == expect, (path, row[0])
                    expect += 1
                    rec = (int(row[1]), int(row[2]), int(row[3]), row[4] == "1")
                    self.days.append(rec)
        assert expect - 1 == LAST.toordinal()
        n = len(self.days)
        for i, (ly, lm, ld, leap) in enumerate(self.days):
            k = (ly, lm, leap)
            if ld == 1:
                assert k not in self.month_start, k
                self.month_start[k] = ORD0 + i
            self.month_len[k] = ld
        # thang dau (12/1899) bat dau dung 1900-01-01? thang cuoi chua tron.
        first_k = self.days[0][:2] + (self.days[0][3],)
        last_k = self.days[-1][:2] + (self.days[-1][3],)
        for k in self.month_len:
            self.month_complete[k] = True
        self.month_complete[last_k] = False
        if self.days[0][2] != 1:
            self.month_complete[first_k] = False
            self.month_start[first_k] = ORD0 - (self.days[0][2] - 1)
        for k, L in self.month_len.items():
            if self.month_complete[k]:
                assert L in (29, 30), (k, L)
        self.n = n

    # ------------------------------------------------------------ doi lich
    def s2l(self, y, m, d):
        o = dt.date(y, m, d).toordinal()
        i = o - ORD0
        if not (0 <= i < self.n):
            return None
        return self.days[i]

    def lyob(self, y, m, d):
        r = self.s2l(y, m, d)
        return None if r is None else r[0]

    def length(self, ly, lm, leap):
        """29/30 neu thang nam tron trong bang; None neu thang khong co hoac bi cat o bien mien."""
        k = (ly, lm, leap)
        if k not in self.month_len or not self.month_complete[k]:
            return None
        return self.month_len[k]

    def exists_month(self, ly, lm, leap):
        return (ly, lm, leap) in self.month_start

    def l2s(self, ly, lm, ld, leap):
        k = (ly, lm, leap)
        st = self.month_start.get(k)
        if st is None or not (1 <= ld <= 30):
            return None
        L = self.length(ly, lm, leap)
        if L is not None and ld > L:
            return None                  # ngay khong ton tai
        o = st + ld - 1
        i = o - ORD0
        if not (0 <= i < self.n):
            return None                  # ngoai mien duong
        rec = self.days[i]
        if rec != (ly, lm, ld, leap):
            return None                  # (thang bi cat o bien: ngay vuot do dai thuc)
        return dt.date.fromordinal(o)

    def lunar_dates(self, y0, y1):
        """Moi ngay am ton tai (ly, lm, ld, leap) co ly trong [y0, y1], theo thu tu thoi gian."""
        for rec in self.days:
            if y0 <= rec[0] <= y1:
                yield rec


# ---------------------------------------------------------------- 02 §1 save_death
def save_death(cal, inp):
    """inp: DateValue dict {calendar, precision, year, month, day, leap}. Tra ve DeathInfo (phan dan xuat)
    hoac raise ValueError (tu choi)."""
    if inp["calendar"] == "solar":
        if inp["precision"] != "day":
            raise ValueError("ngay mat duong phai du ngay")
        try:
            dt.date(inp["year"], inp["month"], inp["day"])
        except ValueError:
            raise ValueError("ngay duong khong ton tai")
        L = cal.s2l(inp["year"], inp["month"], inp["day"])
        if L is None:
            raise ValueError("ngoai mien 1900-3000")
        return {"input": inp, "lunar": {"year": L[0], "month": L[1], "day": L[2], "leap": L[3]},
                "solar": {"y": inp["year"], "m": inp["month"], "d": inp["day"]},
                "lunar_source": "converted", "method": "precise"}
    if inp["precision"] == "year":
        raise ValueError("ngay mat khong duoc chi co nam")
    if inp["precision"] == "month_day":
        if inp["year"] is not None or not (1 <= inp["day"] <= 30) or not (1 <= inp["month"] <= 12):
            raise ValueError("month_day sai")
        return {"input": inp, "lunar": {"year": None, "month": inp["month"], "day": inp["day"],
                                        "leap": inp["leap"]},
                "solar": None, "lunar_source": "entered", "method": "precise"}
    y = inp["year"]
    S = None
    if 1899 <= y <= 3000:
        S = cal.l2s(y, inp["month"], inp["day"], inp["leap"])
        if S is None:
            # trong mien doi lich ma khong ton tai -> tu choi (01 §2.1 V4); rieng ngay am co ngay duong
            # ngoai 1900-3000 (bien mien) cung khong doi duoc.
            raise ValueError("ngay am khong ton tai / ngoai mien")
    return {"input": inp, "lunar": {"year": y, "month": inp["month"], "day": inp["day"], "leap": inp["leap"]},
            "solar": None if S is None else {"y": S.year, "m": S.month, "d": S.day},
            "lunar_source": "entered", "method": "precise"}


# ---------------------------------------------------------------- 02 §3 label_of
def label_of(n, gio_het_nam=GIO_HET_NAM_MAC_DINH):
    if n is None:
        return "gio"
    if n == 1:
        return "gio_dau"
    if n == gio_het_nam:
        return "gio_het"
    return "gio"


# ---------------------------------------------------------------- 02 §2 anniversary
def anniversary(cal, death_lunar, Y, gio_het_nam=GIO_HET_NAM_MAC_DINH):
    """death_lunar: {year|None, month, day, leap}. Tra ve dict hoac None.
    Viet doc lap tu quy tac 02 §2.1:
      1. ngay am cua ngay mat, lap moi nam am;
      2. mat thang nhuan -> gio thang thuong cung so (va nam co ca thuong lan nhuan: thang thuong);
      3. ngay 30 ma thang do (thuong) trong nam Y chi 29 ngay -> 29;
      4. khong ro nam -> van tinh, n = None, nhan "gio"."""
    m, d = death_lunar["month"], death_lunar["day"]
    yd = death_lunar["year"]
    if yd is not None and Y <= yd:
        return None
    if not cal.exists_month(Y, m, False):
        return None                       # ngoai mien (thang m thuong cua nam Y khong co trong bang)
    dd = d
    if d == 30:
        L = cal.length(Y, m, False)
        if L == 29:
            dd = 29
        elif L is None:
            # thang bi cat o bien mien: ngay 29/30 co the vuot ra ngoai 3000-12-31
            pass
    S = cal.l2s(Y, m, dd, False)
    if S is None:
        return None
    n = None if yd is None else Y - yd
    return {"lunar": (Y, m, dd), "solar": S, "n": n, "label": label_of(n, gio_het_nam)}


def anniversaries_in(cal, death_lunar, d_from, d_to, gio_het_nam=GIO_HET_NAM_MAC_DINH):
    """02 §2.3 — viet doc lap: quet moi nam am co the cho ngay duong trong khoang."""
    y0 = cal.lyob(d_from.year, d_from.month, d_from.day) if FIRST <= d_from <= LAST else d_from.year - 1
    y1 = cal.lyob(d_to.year, d_to.month, d_to.day) if FIRST <= d_to <= LAST else d_to.year
    out = []
    for Y in range(y0 - 1, y1 + 2):
        a = anniversary(cal, death_lunar, Y, gio_het_nam)
        if a is not None and d_from <= a["solar"] <= d_to:
            out.append(a)
    return sorted(out, key=lambda a: a["solar"])


# ---------------------------------------------------------------- 02 §4 tuoi, Can Chi
def can_chi_year(Y):
    """Doc lap: 1984 la nam Giap Ty."""
    k = Y - 1984
    return CAN[k % 10] + " " + CHI[k % 12]


def can_chi_contract(Y):
    """Cong thuc cua hop dong §3 (de so voi can_chi_year trong R10(b))."""
    return CAN[(Y + 6) % 10] + " " + CHI[(Y + 8) % 12]


def birth_derive(cal, inp):
    """inp: DateValue ngay sinh hoac None -> {lunar_year, lunar_year_approx, can_chi, lunar, solar}."""
    if inp is None:
        return {"lunar_year": None, "lunar_year_approx": False, "can_chi": None, "lunar": None, "solar": None}
    cal_, prec = inp["calendar"], inp["precision"]
    if prec == "month_day":
        raise ValueError("ngay sinh khong duoc month_day")
    if cal_ == "solar" and prec == "day":
        dt.date(inp["year"], inp["month"], inp["day"])     # raise neu khong ton tai
        L = cal.s2l(inp["year"], inp["month"], inp["day"])
        if L is None:                                      # ngoai mien doi lich
            return {"lunar_year": None, "lunar_year_approx": False, "can_chi": None, "lunar": None,
                    "solar": {"y": inp["year"], "m": inp["month"], "d": inp["day"]}}
        ly = L[0]
        return {"lunar_year": ly, "lunar_year_approx": False, "can_chi": can_chi_year(ly),
                "lunar": {"year": L[0], "month": L[1], "day": L[2], "leap": L[3]},
                "solar": {"y": inp["year"], "m": inp["month"], "d": inp["day"]}}
    if cal_ == "solar" and prec == "year":
        y = inp["year"]
        return {"lunar_year": y, "lunar_year_approx": True, "can_chi": can_chi_year(y), "lunar": None,
                "solar": None}
    if cal_ == "lunar" and prec == "day":
        y = inp["year"]
        S = cal.l2s(y, inp["month"], inp["day"], inp["leap"]) if 1899 <= y <= 3000 else None
        if 1899 <= y <= 3000 and S is None:
            raise ValueError("ngay am khong ton tai")
        return {"lunar_year": y, "lunar_year_approx": False, "can_chi": can_chi_year(y),
                "lunar": {"year": y, "month": inp["month"], "day": inp["day"], "leap": inp["leap"]},
                "solar": None if S is None else {"y": S.year, "m": S.month, "d": S.day}}
    if cal_ == "lunar" and prec == "year":
        y = inp["year"]
        return {"lunar_year": y, "lunar_year_approx": False, "can_chi": can_chi_year(y), "lunar": None,
                "solar": None}
    raise ValueError("kieu nhap khong ho tro")


def tuoi_mat(birth_lunar_year, birth_approx, death_lunar_year, nguong=NGUONG_MAC_DINH):
    """Tuoi mu (tinh ca nam sinh). value < 1 -> ValueError (01 §3.2 / 02 §4.2)."""
    if birth_lunar_year is None or death_lunar_year is None:
        value = None
    else:
        value = death_lunar_year - birth_lunar_year + 1
        if value < 1:
            raise ValueError("Ngày mất trước ngày sinh")
    label = "huong_duong" if (value is not None and value < nguong) else "huong_tho"
    return {"value": value, "label": label, "manual": False, "approx": bool(birth_approx)}


def tuoi_song(cal, birth_inp, birth_der, today):
    """02 §4.3: tuoi duong (so sinh nhat duong da qua) va tuoi am (nam am hom nay - lunar_year + 1)."""
    ly_today = cal.lyob(today.year, today.month, today.day)
    am = None if birth_der["lunar_year"] is None else ly_today - birth_der["lunar_year"] + 1
    duong = None
    if birth_inp is not None and birth_der["solar"] is not None:
        s = birth_der["solar"]
        bd = solar_occurrence((s["y"], s["m"], s["d"]), today.year, "feb28")
        duong = today.year - s["y"] - (0 if bd is not None and bd <= today else 1)
    return {"tuoi_duong": duong, "tuoi_am": am}


# ---------------------------------------------------------------- 02 §6 29/02
def is_leap_greg(Y):
    try:
        dt.date(Y, 2, 29)
        return True
    except ValueError:
        return False


def solar_occurrence(orig, Y, feb29_rule="feb28"):
    Y0, M, D = orig
    if Y < Y0:
        return None
    if (M, D) == (2, 29) and not is_leap_greg(Y):
        return dt.date(Y, 2, 28) if feb29_rule == "feb28" else dt.date(Y, 3, 1)
    return dt.date(Y, M, D)
