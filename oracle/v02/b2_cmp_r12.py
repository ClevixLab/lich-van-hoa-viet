#!/usr/bin/env python3
"""So R12: bang tong hop theo o (size, scenario) va khoi --steps (future, pending, co sentinel, dropped, violations).
Cach dung: b2_cmp_r12.py <tep loi> <tep oracle>  -> JSON ra stdout"""
import json, sys


def parse(p):
    L = open(p).read().splitlines()
    i = L.index("size,scenario,day,future,pending,sentinel,dropped,violations")
    summ = {tuple(l.split(",")[:2]): l.split(",")[2:] for l in L[1:i]}
    steps = {}
    for l in L[i + 1:]:
        f = l.split(",")
        steps[(f[0], f[1], f[2])] = (f[3], f[4], "0" if f[5] in ("", "0") else "1", f[6], f[7] or "0")
    return summ, steps


cs, ct = parse(sys.argv[1])
os_, ot = parse(sys.argv[2])
cols = "opens,max_pending,delivered,sentinel_delivered,violations,missed,missed_without_sentinel".split(",")
cells = {}
for k in cs:
    a, b = cs[k], os_.get(k)
    diff = {c: [a[j], b[j]] for j, c in enumerate(cols) if b is None or a[j] != b[j]}
    sk = [s for s in ct if s[:2] == k]
    common = [s for s in sk if s in ot]
    sd = [[s[2], ct[s], ot[s]] for s in common if ct[s] != ot[s]]
    cells["%s/%s" % k] = {"core": dict(zip(cols, a)), "oracle": dict(zip(cols, b)), "khac_tong_hop": diff,
                         "steps_core": len(sk), "steps_oracle": len([s for s in ot if s[:2] == k]),
                         "steps_chung": len(common), "steps_khac": len(sd), "vi_du_steps_khac": sd[:5],
                         "vi_pham_steps_core": sum(int(ct[s][4]) for s in sk)}
print(json.dumps(cells, ensure_ascii=False, indent=1))
