#!/usr/bin/env python3
"""So R1 dang luong (it bo nho): chuan hoa moi dong ve "khoa<TAB>gia tri" (JSON chuan), sap bang `sort`
ngoai, roi tron. Cung chuan hoa nhu compare_v02.norm_r1. Ghi out/cmp_<ten>.json va out/cmp_<ten>_mismatch.jsonl.
Cach dung: compare_stream.py <ten> <tep loi> <tep oracle>"""
import gzip, hashlib, json, os, subprocess, sys, tempfile
HERE = os.path.dirname(os.path.abspath(__file__)); OUT = os.path.join(HERE, "out")
sys.path.insert(0, HERE)
from compare_v02 import norm_r1, opn  # noqa: E402

TMP = os.environ.get("CMP_TMP", tempfile.gettempdir())


def normalize(src, dst):
    h = hashlib.sha256(); n = 0; rawsorted = None
    with opn(src) as f, open(dst, "w", encoding="utf-8") as g:
        for line in f:
            line = line.rstrip("\r\n")
            if not line.strip():
                continue
            n += 1
            o = json.loads(line)
            t = norm_r1(o)
            k = json.dumps([list(t[0]), t[1], o.get("case"), o.get("gio_het_nam")])
            v = json.dumps([list(t[2]), list(t[3]), t[4], t[5]], ensure_ascii=False)
            g.write(k + "\t" + v + "\n")
    with open(src, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    subprocess.run(["sort", "-S", "500M", "-T", TMP, "-o", dst, dst], check=True, env={**os.environ, "LC_ALL": "C"})
    return n, h.hexdigest()


def rows(p):
    with open(p, encoding="utf-8") as f:
        for l in f:
            k, v = l.rstrip("\n").split("\t")
            yield k, v


def main(name, core, orc):
    pc = os.path.join(TMP, "cmp_%s_core.tsv" % name); po = os.path.join(TMP, "cmp_%s_or.tsv" % name)
    nc, hc = normalize(core, pc); no, _ = normalize(orc, po)
    mism = []; dup_c = dup_o = 0
    A, B = rows(po), rows(pc)
    a, b = next(A, None), next(B, None)
    pa = pb = None
    while a or b:
        if a and pa and a[0] == pa: dup_o += 1
        if b and pb and b[0] == pb: dup_c += 1
        if b is None or (a and a[0] < b[0]):
            mism.append({"key": json.loads(a[0]), "oracle": json.loads(a[1]), "core": None}); pa = a[0]; a = next(A, None)
        elif a is None or b[0] < a[0]:
            mism.append({"key": json.loads(b[0]), "oracle": None, "core": json.loads(b[1])}); pb = b[0]; b = next(B, None)
        else:
            if a[1] != b[1]:
                mism.append({"key": json.loads(a[0]), "oracle": json.loads(a[1]), "core": json.loads(b[1])})
            pa, pb = a[0], b[0]; a, b = next(A, None), next(B, None)
    for m in mism:
        o, c = m["oracle"], m["core"]
        m["fields"] = (["chi_co_o_" + ("oracle" if c is None else "loi")] if o is None or c is None else
                       [f for i, f in enumerate(["lunar", "solar", "n", "label"]) if o[i] != c[i]])
    res = {"name": name, "core": core, "oracle": orc, "n_core": nc, "n_oracle": no, "dup_core": dup_c,
           "dup_oracle": dup_o, "mismatch": len(mism), "sha256_core": hc}
    with open(os.path.join(OUT, "cmp_%s_mismatch.jsonl" % name), "w", encoding="utf-8") as f:
        for m in mism:
            f.write(json.dumps(m, ensure_ascii=False) + "\n")
    json.dump(res, open(os.path.join(OUT, "cmp_%s.json" % name), "w"), ensure_ascii=False, indent=1)
    os.remove(pc); os.remove(po)
    print(json.dumps(res, ensure_ascii=False))


if __name__ == "__main__":
    main(*sys.argv[1:4])
