#!/usr/bin/env python3
"""So hai lop cua chinh oracle (dau_cuoi vs quy_tac): sai lech nay chi do bang am lich khac nhau.
Dung de du bao sai lech R1-dau-cuoi neu quy tac cua loi dung. Ghi out/layerdiff_<part>.jsonl."""
import gzip, json, os, sys
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "out")
def rd(p):
    op = gzip.open if p.endswith(".gz") else open
    with op(p, "rt", encoding="utf-8") as f:
        return [l.rstrip("\n") for l in f if l.strip()]
def key(o):
    return json.dumps(o["death"], sort_keys=True) + "|" + str(o["Y"]) + "|" + str(o.get("case", ""))
res = {}
for part in "abcd":
    ext = ".jsonl" if part == "d" else ".jsonl.gz"
    A = {key(json.loads(l)): l for l in rd(os.path.join(OUT, "r1_dau_cuoi_%s%s" % (part, ext)))}
    B = {key(json.loads(l)): l for l in rd(os.path.join(OUT, "r1_quy_tac_%s%s" % (part, ext)))}
    diff = [(k, A.get(k), B.get(k)) for k in sorted(set(A) | set(B)) if A.get(k) != B.get(k)]
    with open(os.path.join(OUT, "layerdiff_%s.jsonl" % part), "w", encoding="utf-8") as f:
        for k, a, b in diff:
            f.write(json.dumps({"key": k, "dau_cuoi": a and json.loads(a), "quy_tac": b and json.loads(b)}, ensure_ascii=False) + "\n")
    res[part] = {"n_dau_cuoi": len(A), "n_quy_tac": len(B), "diff": len(diff)}
print(json.dumps(res))
