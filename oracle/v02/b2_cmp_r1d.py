#!/usr/bin/env python3
"""R1(d) Jpl: chay `gio-dates list` cua loi (hop den, --method jpl) tren dau vao cac ca co dinh, gio_het 2 va 3,
roi so tung dong (d) cua oracle (out/r1_jpl_d.jsonl) theo khoa (death, Y).
Cach dung: b2_cmp_r1d.py <lvhv-memorial-dump>  -> out/b2_cmp_r1_jpl_d.json"""
import json, os, subprocess, sys, tempfile
HERE = os.path.dirname(os.path.abspath(__file__)); OUT = os.path.join(HERE, "out")
sys.path.insert(0, HERE)
from gen import FIXED  # noqa: E402
B = sys.argv[1]
J = lambda o: json.dumps(o, ensure_ascii=False, separators=(",", ":"))
core = {}
for het in (2, 3):
    inputs = [inp for code, inp, Ys, h in FIXED if h == het]
    with tempfile.NamedTemporaryFile("w", suffix=".jsonl", delete=False) as f:
        for inp in inputs:
            f.write(J(inp) + "\n")
        p = f.name
    r = subprocess.run([B, "--method", "jpl", "--gio-het", str(het), "gio-dates", "list", p, "12"],
                       capture_output=True, text=True, check=True)
    os.unlink(p)
    for l in r.stdout.splitlines():
        o = json.loads(l)
        core[(het, J(o["death"]), o["Y"])] = o
res = {"rows": 0, "match": 0, "mismatch": [], "absent_in_core": []}
for l in open(os.path.join(OUT, "r1_jpl_d.jsonl"), encoding="utf-8"):
    o = json.loads(l); res["rows"] += 1
    het = o.pop("gio_het_nam", 2); case = o.pop("case")
    c = core.get((het, J(o["death"]), o["Y"]))
    if c is None:
        res["absent_in_core"].append([case, het, o["Y"]])
    elif J(c) == J(o):
        res["match"] += 1
    else:
        res["mismatch"].append({"case": case, "oracle": o, "core": c})
json.dump(res, open(os.path.join(OUT, "b2_cmp_r1_jpl_d.json"), "w"), ensure_ascii=False, indent=1)
print(json.dumps(res, ensure_ascii=False))
