#!/usr/bin/env python3
"""Phan loai sai lech R1-dau-cuoi (out/cmp_dau_cuoi_*_mismatch.jsonl) va kiem tra chung trung voi
du bao layerdiff (gia tri cua loi == gia tri lop quy_tac cua oracle). Ghi out/r1_dau_cuoi_mismatch_classified.csv"""
import collections, csv, json, os, sys
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import classify as K
def d3(t): return {"y": t[0], "m": t[1], "d": t[2]}
w = csv.writer(open(os.path.join(HERE, "out", "r1_dau_cuoi_mismatch_classified.csv"), "w", newline="", encoding="utf-8"))
w.writerow(["phan", "ngay_mat", "Y", "truong_khac", "oracle_lunar", "oracle_solar", "loi_lunar", "loi_solar", "n", "label", "phan_loai"])
summary = {}
for part in "abc":
    cnt = collections.Counter(); fc = collections.Counter(); yrs = collections.Counter()
    for l in open(os.path.join(HERE, "out", "cmp_dau_cuoi_%s_mismatch.jsonl" % part), encoding="utf-8"):
        m = json.loads(l); k = m["key"]; dk = k[0]; Y = k[1]
        death = ({"cal": "solar", "y": dk[1], "m": dk[2], "d": dk[3]} if dk[0] == "solar" else
                 {"cal": "lunar", "y": dk[1], "m": dk[2], "d": dk[3], "leap": dk[4]})
        def mk(v): return None if v is None else {"Y": Y, "lunar": d3(v[0]), "solar": d3(v[1])}
        o, c = mk(m["oracle"]), mk(m["core"])
        cl = K.classify(o, c, death)
        cnt[cl] += 1; fc[tuple(m["fields"])] += 1; yrs[Y // 1] += 1
        def s(v, i): return "" if v is None else "%04d-%02d-%02d" % tuple(v[i]) if i == 1 else "%d/%d/%d" % (v[i][2], v[i][1], v[i][0])
        w.writerow([part, json.dumps(dk, ensure_ascii=False), Y, "+".join(m["fields"]), s(m["oracle"], 0), s(m["oracle"], 1),
                    s(m["core"], 0), s(m["core"], 1), (m["oracle"] or m["core"])[2], (m["oracle"] or m["core"])[3], cl])
    summary[part] = {"phan_loai": dict(cnt), "truong": {"+".join(k): v for k, v in fc.items()},
                     "Y_min": min(yrs) if yrs else None, "Y_max": max(yrs) if yrs else None}
print(json.dumps(summary, ensure_ascii=False, indent=1))
