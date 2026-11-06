#!/usr/bin/env python3
"""So R12 (B3): bang tong hop tung o (moi cot) va khoi --steps (future, pending, co sentinel, dropped, violations).
Loi: steps `size,scenario,day,future,pending,sentinel,dropped,violations`; oracle co them cot `beyond` va `sentinel`
la thoi diem. Cach dung: b3_cmp_r12.py <tep loi> <tep oracle> -> JSON"""
import json, sys


def parse(p):
    L = open(p).read().splitlines()
    i = next(k for k in range(1, len(L)) if L[k].startswith("size,scenario,day"))
    hdr = L[0].split(",")
    summ = {tuple(l.split(",")[:2]): dict(zip(hdr[2:], l.split(",")[2:])) for l in L[1:i]}
    sh = L[i].split(",")
    steps = {}
    for l in L[i + 1:]:
        o = dict(zip(sh, l.split(",")))
        steps[(o["size"], o["scenario"], o["day"])] = (o["future"], o["pending"],
                                                     "0" if o["sentinel"] in ("", "0") else "1",
                                                     o["dropped"], o["violations"] or "0")
    return summ, steps


cs, ct = parse(sys.argv[1])
os_, ot = parse(sys.argv[2])
res = {}
for k in cs:
    a, b = cs[k], os_.get(k, {})
    sk = [s for s in ct if s[:2] == k]
    sd = [[s[2], ct[s], ot.get(s)] for s in sk if ct[s] != ot.get(s)]
    res["%s/%s" % k] = {"core": a, "khac_tong_hop": {c: [a[c], b.get(c)] for c in a if a[c] != b.get(c)},
                       "steps_core": len(sk), "steps_oracle": len([s for s in ot if s[:2] == k]),
                       "steps_khac": len(sd), "vi_du_steps_khac": sd[:5]}
print(json.dumps(res, ensure_ascii=False, indent=1))
