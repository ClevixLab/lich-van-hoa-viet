#!/usr/bin/env python3
"""Giai ma bang TK20 (am lich 1900-1999) do Ho Ngoc Duc cong bo trong ma nguon amlich.

Nguon: ref/tk20_ichuot_jquery-amlich_jquery.amlich.js (ban phan phoi lai jquery-amlich, (c) 2004 Ho Ngoc Duc),
tai ve boi workflow (ghi commit trong ref/FETCH-LOG.txt). Ma hoa moi nam (theo ham decodeLunarYear cua tac gia):
  offset Tet = k >> 17 (so ngay tu 1/1 duong); bit 16: do dai thang nhuan (0 -> 29, 1 -> 30);
  bits 4..15: do dai 12 thang thuong (bit cao nhat = thang 1; 0 -> 29, 1 -> 30); bits 0..3: thang nhuan (0 = khong).
"""
import datetime as dt
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "ref", "tk20_ichuot_jquery-amlich_jquery.amlich.js")


def load_tk20(path=SRC):
    s = open(path, encoding="utf-8").read()
    m = re.search(r"TK20\s*=\s*new\s+Array\(([^)]*)\)", s)
    vals = [int(x, 16) for x in re.findall(r"0x[0-9a-fA-F]+", m.group(1))]
    assert len(vals) == 100, len(vals)
    return vals


def decode_year(yy, k):
    """[(start_date, lunar_year, month, leap)]"""
    lengths = (29, 30)
    off = k >> 17
    leap_m = k & 0xF
    leap_len = lengths[(k >> 16) & 1]
    reg = [0] * 12
    j = k >> 4
    for i in range(12):
        reg[11 - i] = lengths[j & 1]
        j >>= 1
    cur = dt.date(yy, 1, 1) + dt.timedelta(days=off)
    out = []
    for mm in range(1, 13):
        out.append((cur, yy, mm, False))
        cur += dt.timedelta(days=reg[mm - 1])
        if mm == leap_m:
            out.append((cur, yy, mm, True))
            cur += dt.timedelta(days=leap_len)
    return out, cur


def months_1900_1999(path=SRC):
    vals = load_tk20(path)
    res = []
    prev_end = None
    for i, k in enumerate(vals):
        ms, end = decode_year(1900 + i, k)
        if prev_end is not None:
            assert ms[0][0] == prev_end, (1900 + i, ms[0][0], prev_end)  # tinh lien tuc giua cac nam
        prev_end = end
        res += ms
    return res


if __name__ == "__main__":
    ms = months_1900_1999()
    print(len(ms), ms[0], ms[-1])
