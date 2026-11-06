#!/usr/bin/env python3
"""Phan loai sai lech: 'sat_nua_dem' neu ngay mat, ngay gio (duong, ca hai ben) hoac thang am lien quan
cham 90 ngay lech am lich da biet giua oracle v0.1 va loi Precise (2242-07-28.., 2962-10-19.., 2988-06-05..);
nguoc lai 'khac'."""
import csv, json, os, sys
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); import giolib as G
def load(src):
    d = {}
    for p in G.TABLES[src]:
        for r in csv.DictReader(open(p, encoding="utf-8")):
            d[r["date"]] = (int(r["ly"]), int(r["lm"]), int(r["ld"]), r["leap"] == "1")
    return d
O, C = load("oracle"), load("core")
DIFFDAYS = sorted(k for k in O if O[k] != C[k])
MONTHS = set()
for k in DIFFDAYS:
    for t in (O[k], C[k]): MONTHS.add((t[0], t[1], t[3]))
# thang ke ben (do dai thang lien ke co the doi khi ranh gioi Soc dich 1 ngay)
SEGMENTS = sorted({k[:7] for k in DIFFDAYS})
def ds(s): return None if s is None else "%04d-%02d-%02d" % (s["y"], s["m"], s["d"])
def classify(o1, o2, death):
    days = set()
    for o in (o1, o2):
        if o: days.add(ds(o["solar"]))
    if death.get("cal") == "solar": days.add("%04d-%02d-%02d" % (death["y"], death["m"], death["d"]))
    if days & set(DIFFDAYS): return "sat_nua_dem"
    for o in (o1, o2):
        if o and ((o["Y"], o["lunar"]["m"], False) in MONTHS): return "sat_nua_dem"
    if death.get("cal") == "lunar" and death.get("y") is not None and (death["y"], death["m"], death["leap"]) in MONTHS:
        return "sat_nua_dem"
    return "khac"
if __name__ == "__main__":
    print(len(DIFFDAYS), SEGMENTS, sorted(MONTHS))
    import collections
    for part in "abcd":
        cnt = collections.Counter(); yrs = collections.Counter()
        for l in open(os.path.join(HERE, "out", "layerdiff_%s.jsonl" % part), encoding="utf-8"):
            x = json.loads(l); a, b = x["dau_cuoi"], x["quy_tac"]
            death = (a or b)["death"]; cnt[classify(a, b, death)] += 1
            yrs[(a or b)["Y"]] += 1
        print(part, dict(cnt), sorted(yrs)[:3], sorted(yrs)[-3:])
