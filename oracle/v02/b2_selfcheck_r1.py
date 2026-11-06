#!/usr/bin/env python3
"""Tu kiem r1_jpl_{a,b,c} bang cach thu hai (nhu selfcheck.py, khong dung giolib.l2s/length): chi muc truc tiep
(ly,lm,ld,leap)->ngay tu bang DE441, do dai thang = max ld."""
import csv, gzip, json, os, sys
HERE = os.path.dirname(os.path.abspath(__file__)); OUT = os.path.join(HERE, "out")
sys.path.insert(0, HERE); import giolib as G
idx, mx, s2l = {}, {}, {}
for p in G.TABLES["de441"]:
    for r in csv.DictReader(gzip.open(p, "rt", encoding="utf-8")):
        t = (int(r["ly"]), int(r["lm"]), int(r["ld"]), r["leap"] == "1")
        idx[t] = r["date"]; s2l[r["date"]] = t
        k = t[:2] + (t[3],); mx[k] = max(mx.get(k, 0), t[2])
def expect(lun, Y):
    m, d, yd = lun
    if yd is not None and Y <= yd: return None
    if (Y, m, False) not in mx: return None
    L = mx[(Y, m, False)]
    dd = 29 if (d == 30 and L == 29) else d
    s = idx.get((Y, m, dd, False))
    return None if s is None else (dd, s)
bad = tot = 0
for part in "abc":
    for l in gzip.open(os.path.join(OUT, "r1_jpl_%s.jsonl.gz" % part), "rt", encoding="utf-8"):
        o = json.loads(l); tot += 1; de = o["death"]
        if de["calendar"] == "solar":
            t = s2l["%04d-%02d-%02d" % (de["year"], de["month"], de["day"])]; lun = (t[1], t[2], t[0])
        else:
            lun = (de["month"], de["day"], de["year"])
        e = expect(lun, o["Y"])
        got = (o["lunar"]["d"], "%04d-%02d-%02d" % (o["solar"]["y"], o["solar"]["m"], o["solar"]["d"]))
        n = None if lun[2] is None else o["Y"] - lun[2]
        lab = "gio" if n is None else ("gio_dau" if n == 1 else ("gio_het" if n == 2 else "gio"))
        if e != got or o["n"] != n or o["label"] != lab or o["lunar"]["m"] != lun[0]:
            bad += 1
            if bad < 5: print("BAD", part, l.strip(), e)
print("b2 selfcheck r1_jpl lines", tot, "bad", bad)
