#!/usr/bin/env python3
"""Kiem tra noi bo oracle v02 bang cach thu hai (khong dung giolib.l2s/length):
dung chi muc truc tiep (ly,lm,ld,leap)->ngay tu bang CSV, do dai thang = max ld.
Kiem lai toan bo r1_*_{a,b,c}. Cung kiem: moi sai lech giua hai lop cham 3 doan da biet."""
import csv, datetime as dt, gzip, json, os, sys
HERE = os.path.dirname(os.path.abspath(__file__)); OUT = os.path.join(HERE, "out")
sys.path.insert(0, HERE); import giolib as G
SEG = {"oracle": [], "core": []}
def build(src):
    idx, mx, s2l = {}, {}, {}
    for p in G.TABLES[src]:
        for r in csv.DictReader(open(p, encoding="utf-8")):
            t = (int(r["ly"]), int(r["lm"]), int(r["ld"]), r["leap"] == "1")
            idx[t] = r["date"]; s2l[r["date"]] = t
            k = t[:2] + (t[3],); mx[k] = max(mx.get(k, 0), t[2])
    return idx, mx, s2l
def expect(idx, mx, lun, Y):
    m, d, yd = lun
    if yd is not None and Y <= yd: return None
    if (Y, m, False) not in mx: return None
    L = mx[(Y, m, False)]
    dd = 29 if (d == 30 and L == 29) else d
    s = idx.get((Y, m, dd, False))
    return None if s is None else (dd, s)
bad = 0; tot = 0
for src, tag in (("oracle", "dau_cuoi"), ("core", "quy_tac")):
    idx, mx, s2l = build(src)
    for part in "abc":
        for l in gzip.open(os.path.join(OUT, "r1_%s_%s.jsonl.gz" % (tag, part)), "rt", encoding="utf-8"):
            o = json.loads(l); tot += 1
            de = o["death"]
            if de["cal"] == "solar":
                t = s2l["%04d-%02d-%02d" % (de["y"], de["m"], de["d"])]; lun = (t[1], t[2], t[0])
            else:
                lun = (de["m"], de["d"], de["y"])
            e = expect(idx, mx, lun, o["Y"])
            got = (o["lunar"]["d"], "%04d-%02d-%02d" % (o["solar"]["y"], o["solar"]["m"], o["solar"]["d"]))
            n = None if lun[2] is None else o["Y"] - lun[2]
            lab = "gio" if n is None else ("gio_dau" if n == 1 else ("gio_het" if n == 2 else "gio"))
            if e != got or o["n"] != n or o["label"] != lab or o["lunar"]["m"] != lun[0]:
                bad += 1
                if bad < 5: print("BAD", src, part, l.strip(), e)
print("selfcheck lines", tot, "bad", bad)
