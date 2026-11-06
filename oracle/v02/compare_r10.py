#!/usr/bin/env python3
"""So R10 cua loi (/home/claude/v02-data/core/r10*.csv) voi oracle v02.
(a) lunar_year + can_chi theo LYOB bang oracle (r10_a.jsonl);
(b) can_chi 1899..3000 (r10_b.jsonl);
(c) oracle tinh lai tren DAU VAO cua loi (birth, death, nguong) bang giolib (bang am oracle), so moi truong;
(d) 29/02: oracle solar_occurrence((1896,2,29), Y, rule) cho moi dong (nam goc nhuan truoc 1900).
Ghi out/cmp_r10.json va out/cmp_r10_mismatch.jsonl."""
import csv, datetime as dt, json, os, sys
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import giolib as G
C = "/home/claude/v02-data/core"; OUT = os.path.join(HERE, "out")
cal = G.Calendar("oracle"); calc = G.Calendar("core")
res = {}; mism = []
# (a)
orc = {}
for l in open(os.path.join(OUT, "r10_a.jsonl"), encoding="utf-8"):
    o = json.loads(l); b = o["birth"]; orc["%04d-%02d-%02d" % (b["y"], b["m"], b["d"])] = (o["lunar_year"], o["can_chi"])
n = 0; seen = set()
for r in csv.DictReader(open(os.path.join(C, "r10a.csv"), encoding="utf-8")):
    n += 1; seen.add(r["birth"])
    v = (int(r["lunar_year"]), r["can_chi"])
    if orc.get(r["birth"]) != v: mism.append({"part": "a", "key": r["birth"], "oracle": orc.get(r["birth"]), "core": v})
for k in set(orc) - seen: mism.append({"part": "a", "key": k, "oracle": orc[k], "core": None})
res["a"] = {"n_core": n, "n_oracle": len(orc)}
# (b)
orc = {json.loads(l)["Y"]: json.loads(l)["can_chi"] for l in open(os.path.join(OUT, "r10_b.jsonl"), encoding="utf-8")}
n = 0; seen = set()
for r in csv.DictReader(open(os.path.join(C, "r10b.csv"), encoding="utf-8")):
    n += 1; Y = int(r["lunar_year"]); seen.add(Y)
    if orc.get(Y) != r["can_chi"]: mism.append({"part": "b", "key": Y, "oracle": orc.get(Y), "core": r["can_chi"]})
for k in set(orc) - seen: mism.append({"part": "b", "key": k, "oracle": orc[k], "core": None})
res["b"] = {"n_core": n, "n_oracle": len(orc)}
# (c)
n = 0; kinds = {}; errs = 0; diff_tables = 0
def nz(x): return None if x in ("", None) else x
for r in csv.DictReader(open(os.path.join(C, "r10c.csv"), encoding="utf-8")):
    n += 1
    bi, di, ng = json.loads(r["birth"]), json.loads(r["death"]), int(r["nguong"])
    kk = (bi["calendar"] + "/" + bi["precision"], di["calendar"] + "/" + di["precision"], ng)
    kinds[kk] = kinds.get(kk, 0) + 1
    def run(cl):
        try:
            b = G.birth_derive(cl, bi); d = G.save_death(cl, di)
            t = G.tuoi_mat(b["lunar_year"], b["lunar_year_approx"], d["lunar"]["year"], ng)
            return (b["lunar_year"], int(b["lunar_year_approx"]), d["lunar"]["year"], t["value"], t["label"], False)
        except ValueError as e:
            return ("ERR", str(e))
    ov = run(cal)
    if ov != run(calc): diff_tables += 1
    if r["error"]:
        errs += 1
        cv = ("ERR", r["error"])
        ok = ov[0] == "ERR"
    else:
        cv = (int(r["birth_lunar_year"]) if r["birth_lunar_year"] else None, int(r["approx"]),
              int(r["death_lunar_year"]) if r["death_lunar_year"] else None,
              int(r["value"]) if r["value"] else None, r["label"], False)
        ok = ov == cv
    if not ok: mism.append({"part": "c", "key": r["i"], "birth": bi, "death": di, "nguong": ng, "oracle": ov, "core": cv})
res["c"] = {"n_core": n, "core_error_rows": errs, "oracle_vs_core_table_differ": diff_tables,
            "kinds": {"|".join(map(str, k)): v for k, v in sorted(kinds.items())}}
# (d)
n = 0; rules = set(); years = set()
for r in csv.DictReader(open(os.path.join(C, "r10d.csv"), encoding="utf-8")):
    n += 1; Y = int(r["year"]); rules.add(r["rule"]); years.add(Y)
    s = G.solar_occurrence((1896, 2, 29), Y, r["rule"]).isoformat()
    if s != r["date"]: mism.append({"part": "d", "key": [Y, r["rule"]], "oracle": s, "core": r["date"]})
res["d"] = {"n_core": n, "rules": sorted(rules), "Y_min": min(years), "Y_max": max(years),
            "complete": years == set(range(1900, 3001)) and rules == {"feb28", "mar1"} and n == 2202}
for p in "abcd": res[p]["mismatch"] = sum(1 for m in mism if m["part"] == p)
json.dump(res, open(os.path.join(OUT, "cmp_r10.json"), "w"), ensure_ascii=False, indent=1)
with open(os.path.join(OUT, "cmp_r10_mismatch.jsonl"), "w", encoding="utf-8") as f:
    for m in mism: f.write(json.dumps(m, ensure_ascii=False, default=str) + "\n")
print(json.dumps(res, ensure_ascii=False, indent=1))
for m in mism[:10]: print(m)
