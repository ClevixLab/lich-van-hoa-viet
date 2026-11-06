#!/usr/bin/env python3
"""So du lieu v0.2 cua loi (/home/claude/v02-data/core/, chi DU LIEU) voi oracle/v02/out/.

Cach dung: python3 compare_v02.py <tep_loi_R1_a> ... (tu dong do tim theo ten neu khong truyen).
Chuan hoa moi dong JSON ve bo (death, Y, lunar y/m/d, solar y/m/d, n, label) vi ten khoa cua loi co the
khac (calendar/cal, year/y ...); ghi ro trong bao cao neu so tung byte khong ap dung duoc.
Ket qua: out/cmp_<ten>.json (tom tat) va out/cmp_<ten>_mismatch.jsonl (moi sai lech).
"""
import gzip
import hashlib
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "out")
sys.path.insert(0, HERE)


def opn(p):
    return gzip.open(p, "rt", encoding="utf-8") if p.endswith(".gz") else open(p, encoding="utf-8")


def g(o, *names, default=KeyError):
    for n in names:
        if isinstance(o, dict) and n in o:
            return o[n]
    if default is KeyError:
        raise KeyError(names)
    return default


def norm_date(x):
    if x is None:
        return None
    if isinstance(x, str):
        y, m, d = x.split("-")
        return (int(y), int(m), int(d))
    if isinstance(x, (list, tuple)):
        return tuple(int(v) for v in x[:3])
    return (int(g(x, "y", "year")), int(g(x, "m", "month")), int(g(x, "d", "day")))


def norm_death(x):
    if isinstance(x, dict) and "input" in x:
        x = x["input"]
    cal = g(x, "cal", "calendar", default=None)
    if cal is None:
        cal = "lunar" if "leap" in x else "solar"
    y = g(x, "y", "year", default=None)
    m = int(g(x, "m", "month"))
    d = int(g(x, "d", "day"))
    if cal == "solar":
        return ("solar", int(y), m, d)
    return ("lunar", None if y is None else int(y), m, d, bool(g(x, "leap", default=False)))


def norm_r1(o):
    lun = g(o, "lunar")
    return (norm_death(g(o, "death")), int(g(o, "Y")),
            (int(g(lun, "y", "year")), int(g(lun, "m", "month")), int(g(lun, "d", "day"))),
            norm_date(g(o, "solar")), g(o, "n", default=None), g(o, "label"))


def load_r1(p):
    rows, raw = {}, []
    dup = 0
    with opn(p) as f:
        for line in f:
            line = line.rstrip("\r\n")
            if not line.strip():
                continue
            raw.append(line)
            o = json.loads(line)
            t = norm_r1(o)
            k = (t[0], t[1], o.get("case"), o.get("gio_het_nam"))
            if k in rows:
                dup += 1
            rows[k] = t
    return rows, raw, dup


def cmp_r1(name, core_path, oracle_path):
    C, craw, cdup = load_r1(core_path)
    O, oraw, odup = load_r1(oracle_path)
    mism = []
    for k in sorted(set(C) | set(O), key=lambda k: json.dumps(k, default=str)):
        a, b = O.get(k), C.get(k)
        if a != b:
            fields = []
            if a is None or b is None:
                fields = ["chi_co_o_" + ("loi" if a is None else "oracle")]
            else:
                for i, f in enumerate(["death", "Y", "lunar", "solar", "n", "label"]):
                    if a[i] != b[i]:
                        fields.append(f)
            mism.append({"key": k, "oracle": a, "core": b, "fields": fields})
    bytes_equal = sorted(craw) == sorted(oraw)
    res = {"name": name, "core": core_path, "oracle": oracle_path, "n_core": len(C), "n_oracle": len(O),
           "dup_core": cdup, "dup_oracle": odup, "mismatch": len(mism), "bytes_equal_after_sort": bytes_equal,
           "sha256_core": hashlib.sha256(open(core_path, "rb").read()).hexdigest()}
    with open(os.path.join(OUT, "cmp_%s_mismatch.jsonl" % name), "w", encoding="utf-8") as f:
        for m in mism:
            f.write(json.dumps(m, ensure_ascii=False, default=str) + "\n")
    with open(os.path.join(OUT, "cmp_%s.json" % name), "w", encoding="utf-8") as f:
        json.dump(res, f, ensure_ascii=False, indent=1)
    return res, mism


if __name__ == "__main__":
    name, core_path, oracle_path = sys.argv[1], sys.argv[2], sys.argv[3]
    res, mism = cmp_r1(name, core_path, oracle_path)
    print(json.dumps(res, ensure_ascii=False))
    for m in mism[:10]:
        print(json.dumps(m, ensure_ascii=False, default=str))
