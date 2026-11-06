#!/usr/bin/env python3
"""Ben kiem dinh doc lap - thoi diem Soc va 24 tiet khi tu JPL DE441 (thang TT).

Phong sach: KHONG doc core/, tsport/, ffi/, tools/jpl/ hay bang cua loi.
Phan mem: jplephem (chi doc he so Chebyshev cua cac doan SPK) + pyerfa (ERFA) cho moi phep
quy doi. Khong dung skyfield.

Dinh nghia (docs/02 Bo sung A4):
  * Soc: kinh do hoang dao bieu kien dia tam cua Mat Trang tru cua Mat Troi = 0.
  * Tiet khi k (k = 0..23): kinh do hoang dao bieu kien dia tam cua Mat Troi = 15 k do.
  * Hoang dao va diem xuan phan THAT cua ngay (true ecliptic/equinox of date).
Quy doi (tu cai, tung buoc):
  1. TT -> TDB: TDB = TT + eraDtdb (tam Trai Dat, u = v = 0).
  2. Vi tri, van toc khoi tam he Mat Troi (SSB) tu DE441: Trai Dat = (0->3) + (3->399);
     Mat Trang = (0->3) + (3->301); Mat Troi = (0->10). Don vi km, km/ngay.
  3. Thoi gian anh sang: P = X(t - tau) - E(t), lap tau = |P| / c den hoi tu.
  4. Lech tia sang do hap dan Mat Troi (chi cho Mat Trang; eraLd). Mat Troi tu lech chinh no = 0.
  5. Quang sai (tuong doi tinh, eraAb) theo van toc khoi tam cua Trai Dat.
  6. Ma tran GCRS -> xich dao that cua ngay: eraPn06 (tue sai IAU 2006, khung lech, chuong dong
     IAU 2000A (eraNut06a) hoac 2000B (eraNut00b) - tuy chon --nutation).
  7. Quay quanh truc x mot goc = do nghieng hoang dao trung binh IAU 2006 (eraObl06) + Delta eps.
     Kinh do = atan2(y, x).
Tim nghiem: Newton voi dao ham sai phan trung tam, vector hoa cho moi su kien, dung khi buoc
< 1e-9 ngay; phan du cuoi cung quy ra giay (kiem <= 0.01 s) duoc ghi trong tep ket qua.

Dung: python3 instants.py --bsp1 de441_part-1.bsp --bsp2 de441_part-2.bsp --out instants_tt.csv.gz
      (co the chi dua --bsp1 voi mot nhan bat ky de thu nghiem, vi du DE421 1900-2050.)
"""
import argparse
import gzip
import hashlib
import math
import os
import sys

import erfa
import jplephem
import numpy as np
from jplephem.spk import SPK

C_KM_DAY = erfa.CMPS / 1000.0 * 86400.0          # toc do anh sang, km/ngay
AU_KM = erfa.DAU / 1000.0
DJ0 = 2400000.5                                   # tach JD hai phan cho ERFA
TT_START = 2414928.5   # 1899-10-01 00:00 TT
TT_END = 2817548.5     # 3002-02-01 00:00 TT (docs/03 §17.3 mo rong tu 3001-03-31 de du thang 11 am nam 3001)
TERM_RATE = 360.0 / 365.2422
SYN = 29.530588861


def jd_of(y, m, d):
    return float(erfa.cal2jd(y, m, d)[0] + erfa.cal2jd(y, m, d)[1])


class Ephem:
    """Doc cac doan SPK (mot tep de441.bsp co hai doan cho moi cap than, hoac hai tep part-1/part-2).

    Moi cap (tam, dich) co the co nhieu doan noi tiep, chong lan nhau mot khoang ngan (DE441: quanh
    1969). Moc tach giua doan i va i+1: 1969-01-01 neu nam trong vung chung (cach bien >= 2 ngay), neu
    khong thi dau doan i+1 + 2 ngay. Truoc moc tach dung doan i, tu moc tach dung doan i+1.
    """

    PAIRS = [(0, 3), (3, 399), (3, 301), (0, 10)]

    def __init__(self, paths):
        self.paths = list(paths)
        self.files = [SPK.open(p) for p in self.paths]
        self.segs = {}
        for pair in self.PAIRS:
            lst = []
            for k in self.files:
                lst += [s for s in k.segments if (s.center, s.target) == pair]
            lst.sort(key=lambda s: s.start_jd)
            assert lst, pair
            splits = []
            for s1, s2 in zip(lst, lst[1:]):
                assert s2.start_jd < s1.end_jd, "hai doan khong noi tiep"
                sp = jd_of(1969, 1, 1)
                if not (s2.start_jd + 2 <= sp <= s1.end_jd - 2):
                    sp = s2.start_jd + 2.0
                splits.append(sp)
            self.segs[pair] = (lst, np.array(splits))
        self.lo = max(v[0][0].start_jd for v in self.segs.values())
        self.hi = min(v[0][-1].end_jd for v in self.segs.values())
        self.splits = sorted(set(float(x) for v in self.segs.values() for x in v[1]))

    def posvel(self, c, t, tdb1, tdb2):
        """Vector vi tri (km) va van toc (km/ngay), shape (3, n)."""
        tdb1 = np.atleast_1d(tdb1).astype(float)
        tdb2 = np.atleast_1d(tdb2).astype(float)
        n = tdb1.size
        pos = np.empty((3, n))
        vel = np.empty((3, n))
        lst, splits = self.segs[(c, t)]
        idx = np.searchsorted(splits, tdb1 + tdb2, side="right")
        for i in np.unique(idx):
            mask = idx == i
            p, v = lst[i].compute_and_differentiate(tdb1[mask], tdb2[mask])
            pos[:, mask] = p
            vel[:, mask] = v
        return pos, vel


def bary(eph, body, tdb1, tdb2):
    emb_p, emb_v = eph.posvel(0, 3, tdb1, tdb2)
    if body == "earth":
        p, v = eph.posvel(3, 399, tdb1, tdb2)
        return emb_p + p, emb_v + v
    if body == "moon":
        p, v = eph.posvel(3, 301, tdb1, tdb2)
        return emb_p + p, emb_v + v
    if body == "sun":
        return eph.posvel(0, 10, tdb1, tdb2)
    raise ValueError(body)


def unit(v):
    return v / np.linalg.norm(v, axis=0)


class Reducer:
    def __init__(self, eph, nutation="2000A"):
        self.eph = eph
        self.nutation = nutation

    def tdb(self, tt):
        """tt: JD TT (float array). Tra ve TDB hai phan (DJ0, phan con lai)."""
        d2 = tt - DJ0
        dt = erfa.dtdb(DJ0, d2, d2 % 1.0, 0.0, 0.0, 0.0)
        return np.full_like(tt, DJ0), d2 + dt / 86400.0

    def apparent_lon(self, tt, bodies=("sun", "moon")):
        """Kinh do hoang dao bieu kien dia tam (do), hoang dao/xuan phan that cua ngay."""
        tt = np.atleast_1d(np.asarray(tt, float))
        t1, t2 = self.tdb(tt)
        E, Ev = bary(self.eph, "earth", t1, t2)
        S0, _ = bary(self.eph, "sun", t1, t2)
        # quang sai: van toc khoi tam Trai Dat theo don vi c; khoang cach Mat Troi - Trai Dat (au)
        v = (Ev / C_KM_DAY).T
        vv = np.sum(v * v, axis=1)
        bm1 = np.sqrt(1.0 - vv)
        es = E - S0
        em = np.linalg.norm(es, axis=0) / AU_KM
        e_hat = unit(es)
        d2 = tt - DJ0
        if self.nutation == "2000A":
            dpsi, deps = erfa.nut06a(DJ0, d2)
        elif self.nutation == "2000B":
            dpsi, deps = erfa.nut00b(DJ0, d2)
            # nut00b la IAU 2000B (khong co hieu chinh J2 cua 2006); giu dung nhu ten
        else:
            raise ValueError(self.nutation)
        _epsa, _rb, _rp, _rbp, _rn, rbpn = erfa.pn06(DJ0, d2, dpsi, deps)
        eps = erfa.obl06(DJ0, d2) + deps
        ce, se = np.cos(eps), np.sin(eps)
        out = {}
        for body in bodies:
            tau = np.zeros_like(tt)
            for _ in range(5):
                X, _ = bary(self.eph, body, t1, t2 - tau)
                P = X - E
                tau_new = np.linalg.norm(P, axis=0) / C_KM_DAY
                done = np.max(np.abs(tau_new - tau)) < 1e-12
                tau = tau_new
                if done:
                    break
            X, _ = bary(self.eph, body, t1, t2 - tau)
            P = X - E
            p = unit(P).T
            if body == "moon":
                Sx, _ = bary(self.eph, "sun", t1, t2 - tau)
                q = unit(X - Sx).T
                p = erfa.ld(1.0, p, q, e_hat.T, em, 1e-9)
            ppr = erfa.ab(p, v, em, bm1)
            # GCRS -> xich dao that cua ngay
            pt = np.einsum("nij,nj->ni", rbpn, ppr)
            x = pt[:, 0]
            y = pt[:, 1] * ce + pt[:, 2] * se
            out[body] = np.degrees(np.arctan2(y, x)) % 360.0
        return out


def wrap180(x):
    return (x + 180.0) % 360.0 - 180.0


def solve(func, t0, tol=1e-9, maxit=30):
    """Newton vector hoa; func(t) -> sai lech goc (do) da quy ve (-180,180]."""
    t = np.array(t0, float)
    h = 1e-4  # ngay (8.64 s) cho sai phan trung tam
    for _ in range(maxit):
        f = func(t)
        rate = (func(t + h) - func(t - h)) / (2 * h)
        step = f / rate
        t = t - step
        if np.max(np.abs(step)) < tol:
            break
    f = func(t)
    rate = (func(t + h) - func(t - h)) / (2 * h)
    resid_s = np.abs(f / rate) * 86400.0
    return t, resid_s


def new_moons(red, lo, hi):
    # uoc luong ban dau: Soc trung binh (Meeus ch.49, k = 0 la 2000-01-06)
    k0 = math.floor((lo - 2451550.09766) / SYN) - 2
    k1 = math.ceil((hi - 2451550.09766) / SYN) + 2
    ks = np.arange(k0, k1 + 1)
    guess = 2451550.09766 + SYN * ks
    guess = guess[(guess > lo - 20) & (guess < hi + 20)]

    def f(t):
        r = red.apparent_lon(t)
        return wrap180(r["moon"] - r["sun"])

    t, res = solve(f, guess)
    m = (t >= lo) & (t < hi)
    return t[m], res[m]


def solar_terms(red, lo, hi):
    # k = so tiet khi tinh tu xuan phan 2000 (kinh do 0); kinh do = 15 (k mod 24)
    ref = 2451623.8  # ~ xuan phan 2000-03-20 TT
    n0 = math.floor((lo - ref) / (365.2422 / 24)) - 3
    n1 = math.ceil((hi - ref) / (365.2422 / 24)) + 3
    ns = np.arange(n0, n1 + 1)
    guess = ref + ns * 365.2422 / 24
    target = (15.0 * ns) % 360.0
    keep = (guess > lo - 10) & (guess < hi + 10)
    guess, target = guess[keep], target[keep]

    def f(t):
        return wrap180(red.apparent_lon(t, bodies=("sun",))["sun"] - target)

    t, res = solve(f, guess)
    m = (t >= lo) & (t < hi)
    return t[m], (target[m]).astype(int), res[m]


def jd_to_iso(jd):
    # lam tron den mili-giay truoc khi tach ngay (tranh "60.000")
    d2 = jd - DJ0
    day = math.floor(d2)
    ms = round((d2 - day) * 86400000.0)
    if ms >= 86400000:
        day += 1
        ms -= 86400000
    y, mo, d, _ = erfa.jd2cal(DJ0, float(day))
    s = ms / 1000.0
    hh = int(s // 3600)
    mm = int((s - hh * 3600) // 60)
    ss = s - hh * 3600 - mm * 60
    return "%04d-%02d-%02dT%02d:%02d:%06.3f" % (y, mo, d, hh, mm, ss)


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(1 << 22), b""):
            h.update(b)
    return h.hexdigest()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bsp1", required=True, help="de441.bsp (hoac part-1)")
    ap.add_argument("--bsp2")
    ap.add_argument("--out", required=True)
    ap.add_argument("--nutation", default="2000A", choices=["2000A", "2000B"])
    ap.add_argument("--start", type=float, default=TT_START)
    ap.add_argument("--end", type=float, default=TT_END)
    ap.add_argument("--no-hash", action="store_true")
    a = ap.parse_args()
    paths = [a.bsp1] + ([a.bsp2] if a.bsp2 else [])
    eph = Ephem(paths)
    red = Reducer(eph, a.nutation)
    lo = max(a.start, eph.lo + 40)
    hi = min(a.end, eph.hi - 40)
    chunks_nm, chunks_t = [], []
    step = 36525.0  # tung the ky de gioi han bo nho
    s = lo
    while s < hi:
        e = min(s + step, hi)
        chunks_nm.append(new_moons(red, s, e))
        chunks_t.append(solar_terms(red, s, e))
        print("chunk %s .. %s" % (jd_to_iso(s)[:10], jd_to_iso(e)[:10]), file=sys.stderr)
        s = e
    nm_t = np.concatenate([c[0] for c in chunks_nm])
    nm_r = np.concatenate([c[1] for c in chunks_nm])
    tm_t = np.concatenate([c[0] for c in chunks_t])
    tm_l = np.concatenate([c[1] for c in chunks_t])
    tm_r = np.concatenate([c[2] for c in chunks_t])
    # kiem tra: don dieu, khoang cach hop ly, phan du <= 0.01 s
    assert np.all(np.diff(nm_t) > 29.0) and np.all(np.diff(nm_t) < 30.0), "khoang Soc bat thuong"
    o = np.argsort(tm_t)
    tm_t, tm_l, tm_r = tm_t[o], tm_l[o], tm_r[o]
    assert np.all(np.diff(tm_t) > 14.0) and np.all(np.diff(tm_t) < 16.5), "khoang tiet khi bat thuong"
    assert np.all((np.diff(tm_l) % 360) == 15), "thieu/trung tiet khi"
    assert nm_r.max() <= 0.01 and tm_r.max() <= 0.01, (nm_r.max(), tm_r.max())
    with gzip.open(a.out, "wt", encoding="utf-8", newline="") as f:
        f.write("# ben kiem dinh doc lap - DE441 -> TT; jplephem %s, pyerfa %s (ERFA %s), numpy %s; nutation IAU %s\n"
                % (jplephem.__version__, erfa.__version__, erfa.version.erfa_version, np.__version__, a.nutation))
        for p in paths:
            f.write("# kernel %s size=%d sha256=%s\n" % (os.path.basename(p), os.path.getsize(p),
                                                        "-" if a.no_hash else sha256(p)))
        for sp in eph.splits:
            f.write("# segment_split_jd_tdb=%.1f (%s)\n" % (sp, jd_to_iso(sp)))
        f.write("# max_residual_s newmoon=%.6f term=%.6f\n" % (nm_r.max(), tm_r.max()))
        f.write("kind,lon_deg,jd_tt,tt_iso,resid_s\n")
        ev = [(t, "newmoon", 0, r) for t, r in zip(nm_t, nm_r)] + \
             [(t, "term", int(l), r) for t, l, r in zip(tm_t, tm_l, tm_r)]
        ev.sort()
        for t, kind, lon, r in ev:
            f.write("%s,%d,%.9f,%s,%.6f\n" % (kind, lon, t, jd_to_iso(t), r))
    print("newmoons=%d terms=%d max_resid nm=%.2e s term=%.2e s" % (len(nm_t), len(tm_t), nm_r.max(), tm_r.max()),
          file=sys.stderr)


if __name__ == "__main__":
    main()
