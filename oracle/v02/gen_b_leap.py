#!/usr/bin/env python3
"""Bo sung (b): ngay mat am khong ro nam co co nhuan = true (loi xuat them 360 ca nay ngoai 12x30 cua 04).
Theo 02 §2.1 quy tac 2, co nhuan bi bo qua -> ket qua nhu leap=false. Ghi r1_<lop>_b_leap.jsonl.gz."""
import gzip, os, sys
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import giolib as G, gen
for src, tag in (("oracle", "dau_cuoi"), ("core", "quy_tac")):
    cal = G.Calendar(src); c = 0
    p = os.path.join(HERE, "out", "r1_%s_b_leap.jsonl.gz" % tag)
    with gzip.open(p, "wt", encoding="utf-8", compresslevel=3) as f:
        for m in range(1, 13):
            for d in range(1, 31):
                death = G.save_death(cal, {"calendar": "lunar", "precision": "month_day", "year": None,
                                           "month": m, "day": d, "leap": True})
                key = {"cal": "lunar", "y": None, "m": m, "d": d, "leap": True}
                for Y in range(1900, 3001):
                    a = G.anniversary(cal, death["lunar"], Y)
                    if a is not None:
                        f.write(gen.rec(key, a) + "\n"); c += 1
    print(tag, c)
