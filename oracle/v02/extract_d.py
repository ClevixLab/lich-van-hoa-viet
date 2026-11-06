#!/usr/bin/env python3
"""Trich cac ca (d) (G-A..G-E, ca bien) tu tep (a)/(b)/(c) cua loi, so voi r1_<lop>_d.jsonl cua oracle.
Ca G-A/het3 (gio_het_nam=3) khong co trong du lieu loi -> ghi la thieu."""
import gzip, json, os, sys
HERE = os.path.dirname(os.path.abspath(__file__)); C = "/home/claude/v02-data/core"
def dk_core(d):
    return (d["calendar"], d["year"], d["month"], d["day"]) + ((d["leap"],) if d["calendar"] == "lunar" else ())
def dk_or(d):
    return (d["cal"], d["y"], d["m"], d["d"]) + ((d["leap"],) if d["cal"] == "lunar" else ())
want = {}
for l in open(os.path.join(HERE, "out", "r1_quy_tac_d.jsonl"), encoding="utf-8"):
    o = json.loads(l)
    if "gio_het_nam" in o: continue
    want[(dk_or(o["death"]), o["Y"])] = (o["case"], o["lunar"], o["solar"], o["n"], o["label"])
need_deaths = {k[0] for k in want}
got = {}
for f in ("r1a_solar_1900_3000", "r1b_unknown_1900_3000", "r1c_lunar_1900_2999"):
    for l in gzip.open(os.path.join(C, f + ".jsonl.gz"), "rt", encoding="utf-8"):
        o = json.loads(l); k = dk_core(o["death"])
        if k in need_deaths:
            got[(k, o["Y"])] = (o["lunar"], o["solar"], o["n"], o["label"])
ok = bad = miss = 0
for k, v in sorted(want.items(), key=lambda kv: (kv[1][0], kv[0][1])):
    g = got.get(k)
    if g is None: miss += 1; print("THIEU", v[0], k)
    elif g != v[1:]: bad += 1; print("KHAC", v[0], k, v, g)
    else: ok += 1
# them: so dong loi cho cac ngay mat bien (khong co giỗ ngoai mien)
extra = sorted((k for k in got if k not in want), key=str)
print(json.dumps({"khop": ok, "khac": bad, "thieu": miss, "dong_loi_ngoai_bo_oracle_cho_cung_ngay_mat": len(extra)}))
for k in extra[:20]: print("EXTRA", k, got[k])
