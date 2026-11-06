#!/usr/bin/env python3
"""Sinh bảng Sóc + 24 tiết khí của phương pháp `Jpl` từ JPL DE441 (docs/03 §17).

Cách dùng (chạy trên máy chủ tích hợp liên tục, xem .github/workflows/jpl-table.yml):

    python3 tools/jpl/gen_table.py --part1 de441_part-1.bsp --part2 de441_part-2.bsp --repo .

Đầu ra (ghi đè):
    core/data/jpl_events.bin       bảng nhị phân nhúng vào lõi Rust (include_bytes!)
    core/data/jpl_events.csv.gz    tệp kiểm toán người đọc được (kind,index,n_cs,jd_tt,tt_iso)
    core/data/jpl_events.meta.txt  phiên bản phần mềm, tổng kiểm SHA-256 của DE441 và của bảng
    tsport/src/jpl_data.ts         cùng các byte của .bin, dạng base64, cho TypeScript

Định nghĩa (docs/03 §17.3): λ là kinh độ hoàng đạo biểu kiến địa tâm (thời gian ánh sáng, lệch hấp dẫn,
quang sai năm, tuế sai IAU 2006 + chương động IAU 2000A, hoàng đạo và điểm xuân phân thật của ngày).
Sóc: λ☾ − λ☉ = 0. Tiết khí i: λ☉ = 15°·i. Mọi sự kiện có TT trong [JD 2414928.5, JD 2817548.5).

Định dạng `jpl_events.bin` (mọi số nguyên là varint LEB128 không dấu của zigzag(x) = (x << 1) ^ (x >> 63)):
    8 byte  "LVHVJPL1"
    varint  zigzag(k0)      chỉ số Hồ Ngọc Đức của Sóc đầu tiên (Sóc thứ j có k = k0 + j)
    varint  zigzag(ns)      số Sóc
    varint  zigzag(i0)      chỉ số (0..23) của tiết khí đầu tiên (tiết khí thứ j có chỉ số (i0 + j) mod 24)
    varint  zigzag(nt)      số tiết khí
    ns varint  dãy Sóc:     e_j = v_j với j < 3; e_j = v_j − (3v_{j−1} − 3v_{j−2} + v_{j−3}) với j ≥ 3
    nt varint  dãy tiết khí: r_j = v_j với j < 48; r_j = v_j − (2v_{j−24} − v_{j−48}) với j ≥ 48;
                            e_j = r_j với j ≤ 48; e_j = r_j − r_{j−1} với j > 48
    v_j = n (đơn vị 0,01 s TT kể từ J2000.0), n = floor(100·S + 0.5), S = (JD_TT − 2451545)·86400.
Không còn byte nào sau dãy tiết khí.
"""

import argparse
import base64
import datetime as dtm
import gzip
import hashlib
import io
import os
import sys

import numpy as np
import skyfield
import jplephem
from skyfield.api import load, load_file
from skyfield.framelib import ecliptic_frame

JD_START = 2414928.5  # 1899-10-01 00:00 TT
JD_END = 2817548.5    # 3002-02-01 00:00 TT (không gồm)
JD_SPLIT_TDB = 2440403.5  # 1969-07-01 00:00 TDB: trước → part-1, từ đó → part-2
J2000 = 2451545.0
HND_K_OFFSET = 1237
TOL_DAYS = 1e-9
CHUNK = 4000


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for b in iter(lambda: f.read(1 << 22), b''):
            h.update(b)
    return h.hexdigest()


class Ephem:
    def __init__(self, part1, part2):
        self.ts = load.timescale(builtin=True)
        self.k1 = load_file(part1)
        self.k2 = load_file(part2)
        for k, name in ((self.k1, 'part-1'), (self.k2, 'part-2')):
            print(f'[gen] {name}:', file=sys.stderr)
            print(str(k), file=sys.stderr)

    def _lon(self, kernel, body, jd_tt):
        out = np.empty(len(jd_tt))
        earth = kernel['earth']
        target = kernel[body]
        for a in range(0, len(jd_tt), CHUNK):
            t = self.ts.tt_jd(jd_tt[a:a + CHUNK])
            _, lon, _ = earth.at(t).observe(target).apparent().frame_latlon(ecliptic_frame)
            out[a:a + CHUNK] = lon.degrees
        return out

    def lon(self, body, jd_tt):
        jd_tt = np.asarray(jd_tt, dtype=float)
        tdb = self.ts.tt_jd(jd_tt).tdb
        out = np.empty(len(jd_tt))
        m1 = tdb < JD_SPLIT_TDB
        if m1.any():
            out[m1] = self._lon(self.k1, body, jd_tt[m1])
        if (~m1).any():
            out[~m1] = self._lon(self.k2, body, jd_tt[~m1])
        return out


def wrap(x):
    """Quy về [−180, 180)."""
    return (x + 180.0) % 360.0 - 180.0


def bisect(f, lo, hi):
    flo = f(lo)
    fhi = f(hi)
    if not (np.all(flo < 0) and np.all(fhi > 0)):
        bad = np.nonzero(~((flo < 0) & (fhi > 0)))[0]
        raise SystemExit(f'kẹp nghiệm thất bại ở {len(bad)} sự kiện, ví dụ lo={lo[bad[:3]]}')
    it = 0
    while True:
        w = hi - lo
        if np.max(w) <= TOL_DAYS:
            break
        mid = lo + w / 2.0
        if np.all((mid == lo) | (mid == hi)):
            break
        fm = f(mid)
        neg = fm < 0
        lo = np.where(neg, mid, lo)
        hi = np.where(neg, hi, mid)
        it += 1
        if it > 80:
            raise SystemExit('chia đôi không hội tụ')
    print(f'[gen] chia đôi: {it} vòng, độ rộng lớn nhất {np.max(hi - lo):.3e} ngày', file=sys.stderr)
    return lo + (hi - lo) / 2.0


def new_moons(eph):
    kmin = int(np.floor((JD_START - 2451550.09766) / 29.530588861)) - 1
    kmax = int(np.ceil((JD_END - 2451550.09766) / 29.530588861)) + 1
    k = np.arange(kmin, kmax + 1)
    T = k / 1236.85
    mean = 2451550.09766 + 29.530588861 * k + 0.00015437 * T ** 2
    f = lambda t: wrap(eph.lon('moon', t) - eph.lon('sun', t))
    t = bisect(f, mean - 1.5, mean + 1.5)
    keep = (t >= JD_START) & (t < JD_END)
    return k[keep] + HND_K_OFFSET, t[keep]


def solar_terms(eph):
    # Xuân phân 2000 ≈ JD 2451623.82 TT; năm chí tuyến ≈ 365.24219 ngày.
    n0 = int(np.floor((JD_START - 2451623.82) / 365.24219 * 24)) - 2
    n1 = int(np.ceil((JD_END - 2451623.82) / 365.24219 * 24)) + 2
    n = np.arange(n0, n1 + 1)
    idx = n % 24
    mean = 2451623.82 + n * (365.24219 / 24.0)
    target = 15.0 * idx
    f = lambda t: wrap(eph.lon('sun', t) - target_cur[0])
    target_cur = [target]
    t = bisect(f, mean - 5.0, mean + 5.0)
    keep = (t >= JD_START) & (t < JD_END)
    return idx[keep], t[keep]


def to_cs(jd):
    return [int(np.floor(100.0 * ((x - J2000) * 86400.0) + 0.5)) for x in jd]


def zz(x):
    return (x << 1) ^ (x >> 63)


def varint(u):
    out = bytearray()
    while True:
        b = u & 0x7F
        u >>= 7
        if u:
            out.append(b | 0x80)
        else:
            out.append(b)
            return bytes(out)


def encode(k0, soc, i0, tk):
    out = bytearray(b'LVHVJPL1')
    for x in (k0, len(soc), i0, len(tk)):
        out += varint(zz(x))
    for j, v in enumerate(soc):
        e = v if j < 3 else v - (3 * soc[j - 1] - 3 * soc[j - 2] + soc[j - 3])
        out += varint(zz(e))
    r = [v if j < 48 else v - (2 * tk[j - 24] - tk[j - 48]) for j, v in enumerate(tk)]
    for j in range(len(tk)):
        e = r[j] if j <= 48 else r[j] - r[j - 1]
        out += varint(zz(e))
    return bytes(out)


def decode(b):
    assert b[:8] == b'LVHVJPL1'
    pos = 8

    def rd():
        nonlocal pos
        u = 0
        s = 0
        while True:
            c = b[pos]
            pos += 1
            u |= (c & 0x7F) << s
            s += 7
            if c < 0x80:
                return (u >> 1) ^ -(u & 1)
    k0, ns, i0, nt = rd(), rd(), rd(), rd()
    soc = []
    for j in range(ns):
        e = rd()
        soc.append(e if j < 3 else e + 3 * soc[j - 1] - 3 * soc[j - 2] + soc[j - 3])
    tk = []
    r_prev = 0
    for j in range(nt):
        e = rd()
        r = e if j <= 48 else e + r_prev
        r_prev = r
        tk.append(r if j < 48 else r + 2 * tk[j - 24] - tk[j - 48])
    assert pos == len(b)
    return k0, soc, i0, tk


def tt_iso(cs):
    # cs: 0,01 s TT kể từ J2000.0 = 2000-01-01T12:00:00 TT (lịch Gregory đón trước).
    base = dtm.datetime(2000, 1, 1, 12, 0, 0)
    q, r = divmod(cs, 100)
    d = base + dtm.timedelta(seconds=q)
    return f'{d:%Y-%m-%dT%H:%M:%S}.{r:02d}'


def check(k, tsoc, idx, tterm):
    ds = np.diff(tsoc)
    assert np.all(np.diff(k) == 1), 'k không liên tiếp'
    assert ds.min() > 29.2 and ds.max() < 29.9, (ds.min(), ds.max())
    dt = np.diff(tterm)
    assert np.all((np.diff(idx) % 24) == 1), 'tiết khí không liên tiếp'
    assert dt.min() > 14.6 and dt.max() < 15.8, (dt.min(), dt.max())
    # Mốc công bố (UT, làm tròn phút) + ΔT(2000) ≈ 64 s: Sóc 2000-01-06 18:14 UT, Xuân phân 2000-03-20 07:35 UT.
    i = int(np.argmin(np.abs(tsoc - 2451550.26)))
    ut = tsoc[i] - 63.8 / 86400.0
    assert abs(ut - (2451549.5 + (18 * 60 + 14) / 1440.0)) < 2.0 / 1440.0, ('Sóc 2000-01-06', ut)
    assert k[i] == 1237, k[i]
    j = int(np.argmin(np.abs(tterm - 2451623.82)))
    assert idx[j] == 0
    ut = tterm[j] - 63.8 / 86400.0
    assert abs(ut - (2451623.5 + (7 * 60 + 35) / 1440.0)) < 2.0 / 1440.0, ('Xuân phân 2000', ut)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--part1', required=True)
    ap.add_argument('--part2', required=True)
    ap.add_argument('--repo', default='.')
    ap.add_argument('--test-jd-end', type=float, default=None,
                    help='CHỈ để thử bộ sinh với lịch thiên văn ngắn (ví dụ DE421); không dùng cho bảng thật')
    a = ap.parse_args()
    if a.test_jd_end is not None:
        global JD_END
        JD_END = a.test_jd_end
    eph = Ephem(a.part1, a.part2)
    k, tsoc = new_moons(eph)
    idx, tterm = solar_terms(eph)
    check(k, tsoc, idx, tterm)
    soc = to_cs(tsoc)
    tk = to_cs(tterm)
    b = encode(int(k[0]), soc, int(idx[0]), tk)
    assert decode(b) == (int(k[0]), soc, int(idx[0]), tk)
    data = os.path.join(a.repo, 'core', 'data')
    os.makedirs(data, exist_ok=True)
    with open(os.path.join(data, 'jpl_events.bin'), 'wb') as f:
        f.write(b)
    rows = [('soc', int(kk), n, x) for kk, n, x in zip(k, soc, tsoc)]
    rows += [('tk', int(ii), n, x) for ii, n, x in zip(idx, tk, tterm)]
    rows.sort(key=lambda r: (r[2], r[0]))
    txt = io.StringIO()
    txt.write('kind,index,n_cs,jd_tt,tt_iso\n')
    for kind, i, n, x in rows:
        txt.write(f'{kind},{i},{n},{2451545.0 + n / 8640000.0:.8f},{tt_iso(n)}\n')
    raw = txt.getvalue().encode()
    with open(os.path.join(data, 'jpl_events.csv.gz'), 'wb') as f:
        with gzip.GzipFile(filename='jpl_events.csv', mode='wb', fileobj=f, mtime=0, compresslevel=9) as g:
            g.write(raw)
    sha = hashlib.sha256(b).hexdigest()
    with open(os.path.join(data, 'jpl_events.meta.txt'), 'w') as f:
        f.write(f'skyfield {skyfield.__version__}, jplephem {jplephem.__version__}, numpy {np.__version__}, '
                f'python {sys.version.split()[0]}\n')
        f.write(f'de441_part-1.bsp sha256 {sha256_file(a.part1)}\n')
        f.write(f'de441_part-2.bsp sha256 {sha256_file(a.part2)}\n')
        f.write(f'jpl_events.bin sha256 {sha} bytes {len(b)}\n')
        f.write(f'soc {len(soc)} k {int(k[0])}..{int(k[-1])}; tiet_khi {len(tk)} i0 {int(idx[0])}\n')
        f.write(f'range TT [JD {JD_START}, JD {JD_END})\n')
    ts_path = os.path.join(a.repo, 'tsport', 'src', 'jpl_data.ts')
    with open(ts_path, 'w') as f:
        f.write('// SINH TỰ ĐỘNG bởi tools/jpl/gen_table.py — không sửa tay. Cùng các byte với\n')
        f.write(f'// core/data/jpl_events.bin (sha256 {sha}, {len(b)} byte). docs/03 §17.5.\n')
        f.write(f'export const JPL_EVENTS_SHA256 = "{sha}";\n')
        f.write(f'export const JPL_EVENTS_B64 =\n  "{base64.b64encode(b).decode()}";\n')
    print(f'[gen] Sóc {len(soc)}, tiết khí {len(tk)}, bin {len(b)} byte, sha256 {sha}', file=sys.stderr)


if __name__ == '__main__':
    main()
