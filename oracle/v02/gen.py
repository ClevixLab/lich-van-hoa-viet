#!/usr/bin/env python3
"""Sinh dau ra oracle v0.2 cho R1 va R10 (docs/v0.2/04) vao oracle/v02/out/.

R1 (JSONL nen gzip, moi dong: {"death":…, "Y":…, "lunar":{y,m,d}, "solar":{y,m,d}, "n":…, "label":…},
    JSON gon, khoa theo thu tu tren, ensure_ascii=False, sap theo thu tu sinh):
  r1_<lop>_a.jsonl.gz  (a) moi ngay mat duong 1900-01-01..3000-12-31, Y = nam am mat + 1..+10
                           (bo nam co ngay duong ngoai mien)
  r1_<lop>_b.jsonl.gz  (b) ngay mat am khong ro nam 12 thang x 30 ngay, Y = 1900..3000
  r1_<lop>_c.jsonl.gz  (c) moi ngay am ton tai, nam am 1900..2999 (gom nhuan, ngay 30), 10 nam gio
  r1_<lop>_d.jsonl     (d) ca co dinh G-A..G-E va ca bien 02 §9
  <lop> = "dau_cuoi" (bang oracle v0.1 PyEphem) | "quy_tac" (bang am lich da cong bo cua loi, Precise)
  "death": (a) {"cal":"solar","y","m","d"}; (b),(c) {"cal":"lunar","y"|null,"m","d","leap"}

R10:
  r10_a.jsonl    {"birth":{y,m,d},"lunar_year":…}  moi ngay sinh duong 1900-01-01..2026-12-31 (bang oracle)
  r10_b.jsonl    {"Y":…, "can_chi":…}               1899..3000
  r10_c.jsonl    10 000 cap (sinh, mat) ngau nhien, hat 20261106, nguong {59,60,61}
  r10_d.jsonl    {"orig":{y:…,m:2,d:29}, "Y":…, "rule":…, "date":{y,m,d}}  1900..3000 x {feb28, mar1}
"""
import datetime as dt
import gzip
import json
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import giolib as G  # noqa: E402

OUT = os.path.join(HERE, "out")
os.makedirs(OUT, exist_ok=True)


def J(o):
    return json.dumps(o, ensure_ascii=False, separators=(",", ":"))


def rec(death_key, a):
    Y, m, d = a["lunar"]
    S = a["solar"]
    return J({"death": death_key, "Y": Y, "lunar": {"y": Y, "m": m, "d": d},
              "solar": {"y": S.year, "m": S.month, "d": S.day}, "n": a["n"], "label": a["label"]})


def gen_r1(layer, cal):
    tag = {"oracle": "dau_cuoi", "core": "quy_tac"}[layer]
    counts = {}
    # (a)
    p = os.path.join(OUT, "r1_%s_a.jsonl.gz" % tag)
    c = 0
    with gzip.open(p + ".tmp", "wt", encoding="utf-8", compresslevel=3) as f:
        o = G.FIRST.toordinal()
        while o <= G.LAST.toordinal():
            day = dt.date.fromordinal(o)
            death = G.save_death(cal, {"calendar": "solar", "precision": "day", "year": day.year,
                                       "month": day.month, "day": day.day, "leap": False})
            key = {"cal": "solar", "y": day.year, "m": day.month, "d": day.day}
            yd = death["lunar"]["year"]
            for Y in range(yd + 1, yd + 11):
                a = G.anniversary(cal, death["lunar"], Y)
                if a is not None:
                    f.write(rec(key, a) + "\n")
                    c += 1
            o += 1
    os.replace(p + ".tmp", p)
    counts["a"] = c
    # (b)
    p = os.path.join(OUT, "r1_%s_b.jsonl.gz" % tag)
    c = 0
    with gzip.open(p + ".tmp", "wt", encoding="utf-8", compresslevel=3) as f:
        for m in range(1, 13):
            for d in range(1, 31):
                death = G.save_death(cal, {"calendar": "lunar", "precision": "month_day", "year": None,
                                           "month": m, "day": d, "leap": False})
                key = {"cal": "lunar", "y": None, "m": m, "d": d, "leap": False}
                for Y in range(1900, 3001):
                    a = G.anniversary(cal, death["lunar"], Y)
                    if a is not None:
                        f.write(rec(key, a) + "\n")
                        c += 1
    os.replace(p + ".tmp", p)
    counts["b"] = c
    # (c)
    p = os.path.join(OUT, "r1_%s_c.jsonl.gz" % tag)
    c = 0
    nd = 0
    with gzip.open(p + ".tmp", "wt", encoding="utf-8", compresslevel=3) as f:
        for (ly, lm, ld, leap) in cal.lunar_dates(1900, 2999):
            death = G.save_death(cal, {"calendar": "lunar", "precision": "day", "year": ly,
                                       "month": lm, "day": ld, "leap": leap})
            nd += 1
            key = {"cal": "lunar", "y": ly, "m": lm, "d": ld, "leap": leap}
            for Y in range(ly + 1, ly + 11):
                a = G.anniversary(cal, death["lunar"], Y)
                if a is not None:
                    f.write(rec(key, a) + "\n")
                    c += 1
    os.replace(p + ".tmp", p)
    counts["c"] = c
    counts["c_deaths"] = nd
    return tag, counts


# ---------------------------------------------------------------- (d) ca co dinh
FIXED = [
    # (ma, input ngay mat, [Y...], gio_het_nam)
    ("G-A", {"calendar": "solar", "precision": "day", "year": 2025, "month": 8, "day": 8, "leap": False},
     [2026, 2027, 2028], 2),
    ("G-B", {"calendar": "solar", "precision": "day", "year": 2025, "month": 9, "day": 21, "leap": False},
     [2026, 2027], 2),
    ("G-C", {"calendar": "lunar", "precision": "month_day", "year": None, "month": 3, "day": 10, "leap": False},
     [2026, 2027], 2),
    ("G-D", {"calendar": "solar", "precision": "day", "year": 2024, "month": 2, "day": 9, "leap": False},
     [2024, 2025], 2),
    ("G-E", {"calendar": "lunar", "precision": "day", "year": 2027, "month": 5, "day": 15, "leap": False},
     [2028], 2),
    ("G-A/het3", {"calendar": "solar", "precision": "day", "year": 2025, "month": 8, "day": 8, "leap": False},
     [2026, 2027, 2028], 3),
    ("B-29/12", {"calendar": "lunar", "precision": "day", "year": 2025, "month": 12, "day": 29, "leap": False},
     list(range(2026, 2036)), 2),
    ("B-30/12", {"calendar": "lunar", "precision": "day", "year": 2020, "month": 12, "day": 30, "leap": False},
     list(range(2021, 2031)), 2),
    ("B-1/1", {"calendar": "lunar", "precision": "day", "year": 2025, "month": 1, "day": 1, "leap": False},
     list(range(2026, 2036)), 2),
    ("B-1900-01-01", {"calendar": "solar", "precision": "day", "year": 1900, "month": 1, "day": 1, "leap": False},
     list(range(1900, 1911)), 2),
    ("B-3000-12-31", {"calendar": "solar", "precision": "day", "year": 3000, "month": 12, "day": 31, "leap": False},
     list(range(3000, 3011)), 2),
    ("B-3000-01-01", {"calendar": "solar", "precision": "day", "year": 3000, "month": 1, "day": 1, "leap": False},
     list(range(2999, 3003)), 2),
]

# Ket qua mong doi trong 02 §2.4 (de tu kiem oracle; khong phai doi chieu voi loi)
EXPECT = [
    ("G-A", 2026, (2026, 6, 15), (2026, 7, 28), 1, "gio_dau"),
    ("G-A", 2027, (2027, 6, 15), (2027, 7, 18), 2, "gio_het"),
    ("G-B", 2026, (2026, 7, 29), (2026, 9, 10), 1, "gio_dau"),
    ("G-B", 2027, (2027, 7, 30), (2027, 8, 31), 2, "gio_het"),
    ("G-C", 2026, (2026, 3, 10), (2026, 4, 26), None, "gio"),
    ("G-C", 2027, (2027, 3, 10), (2027, 4, 16), None, "gio"),
    ("G-D", 2024, (2024, 12, 29), (2025, 1, 28), 1, "gio_dau"),
    ("G-D", 2025, (2025, 12, 29), (2026, 2, 16), 2, "gio_het"),
    ("G-E", 2028, (2028, 5, 15), (2028, 6, 7), 1, "gio_dau"),
    ("G-A/het3", 2027, (2027, 6, 15), (2027, 7, 18), 2, "gio"),
    ("G-A/het3", 2028, None, None, 3, "gio_het"),
]


def gen_r1_d(layer, cal):
    tag = {"oracle": "dau_cuoi", "core": "quy_tac"}[layer]
    lines, got, notes = [], {}, []
    for code, inp, Ys, het in FIXED:
        death = G.save_death(cal, inp)
        if inp["calendar"] == "solar":
            key = {"cal": "solar", "y": inp["year"], "m": inp["month"], "d": inp["day"]}
        else:
            key = {"cal": "lunar", "y": inp["year"], "m": inp["month"], "d": inp["day"], "leap": inp["leap"]}
        notes.append((code, death["lunar"], death["solar"]))
        for Y in Ys:
            a = G.anniversary(cal, death["lunar"], Y, het)
            if a is None:
                got[(code, Y)] = None
                continue
            got[(code, Y)] = a
            o = json.loads(rec(key, a))
            o = {"case": code, **o}
            if het != 2:
                o["gio_het_nam"] = het
            lines.append(J(o))
    with open(os.path.join(OUT, "r1_%s_d.jsonl" % tag), "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    # tu kiem
    bad = []
    for code, Y, lun, sol, n, lab in EXPECT:
        a = got.get((code, Y))
        if a is None:
            bad.append((code, Y, "None"))
            continue
        if lun is not None and (a["lunar"] != lun or (a["solar"].year, a["solar"].month, a["solar"].day) != sol):
            bad.append((code, Y, a["lunar"], a["solar"].isoformat()))
        if a["n"] != n or a["label"] != lab:
            bad.append((code, Y, a["n"], a["label"]))
    # G-E: khong phai thang nhuan 2028-07-07
    return tag, bad, notes, got


# ---------------------------------------------------------------- R10
def gen_r10(cal):
    res = {}
    # (a)
    c = 0
    with open(os.path.join(OUT, "r10_a.jsonl"), "w", encoding="utf-8") as f:
        o, end = dt.date(1900, 1, 1).toordinal(), dt.date(2026, 12, 31).toordinal()
        while o <= end:
            d = dt.date.fromordinal(o)
            b = G.birth_derive(cal, {"calendar": "solar", "precision": "day", "year": d.year, "month": d.month,
                                     "day": d.day, "leap": False})
            f.write(J({"birth": {"y": d.year, "m": d.month, "d": d.day}, "lunar_year": b["lunar_year"],
                       "can_chi": b["can_chi"]}) + "\n")
            c += 1
            o += 1
    res["a"] = c
    # (b)
    bad_b = []
    with open(os.path.join(OUT, "r10_b.jsonl"), "w", encoding="utf-8") as f:
        for Y in range(1899, 3001):
            cc = G.can_chi_year(Y)
            if cc != G.can_chi_contract(Y):
                bad_b.append(Y)
            f.write(J({"Y": Y, "can_chi": cc}) + "\n")
    res["b"] = 3000 - 1899 + 1
    res["b_internal_mismatch"] = bad_b
    # (c)
    rng = random.Random(20261106)
    kinds_b = [("solar", "day"), ("solar", "year"), ("lunar", "day"), ("lunar", "year")]
    kinds_d = [("solar", "day"), ("lunar", "day"), ("lunar", "month_day")]
    lunar_all = list(cal.lunar_dates(1900, 2999))
    by_year = {}
    for r in lunar_all:
        by_year.setdefault(r[0], []).append(r)
    c = 0
    with open(os.path.join(OUT, "r10_c.jsonl"), "w", encoding="utf-8") as f:
        i = 0
        while c < 10000:
            kb = kinds_b[i % 4]
            kd = kinds_d[(i // 4) % 3]
            ng = [59, 60, 61][(i // 12) % 3]
            i += 1
            # chon nam sinh 1900..2026, nam mat >= nam sinh, <= 2026 (khong o tuong lai)
            by = rng.randint(1900, 2026)
            if kb == ("solar", "day"):
                d0 = dt.date(by, 1, 1).toordinal()
                d1 = dt.date(by, 12, 31).toordinal()
                bd = dt.date.fromordinal(rng.randint(d0, d1))
                binp = {"calendar": "solar", "precision": "day", "year": bd.year, "month": bd.month,
                        "day": bd.day, "leap": False}
            elif kb == ("solar", "year"):
                binp = {"calendar": "solar", "precision": "year", "year": by, "month": None, "day": None,
                        "leap": False}
            elif kb == ("lunar", "day"):
                r = rng.choice(by_year[by])
                binp = {"calendar": "lunar", "precision": "day", "year": r[0], "month": r[1], "day": r[2],
                        "leap": r[3]}
            else:
                binp = {"calendar": "lunar", "precision": "year", "year": by, "month": None, "day": None,
                        "leap": False}
            bder = G.birth_derive(cal, binp)
            # tuoi muc tieu quanh nguong: 50 % trong [ng-2, ng+2], con lai 1..110
            if rng.random() < 0.5:
                age = rng.randint(ng - 2, ng + 2)
            else:
                age = rng.randint(1, 110)
            dy = bder["lunar_year"] + age - 1
            if dy > 2026 or dy < 1900:
                continue
            if kd == ("solar", "day"):
                # mot ngay duong ma nam am = dy
                r = rng.choice(by_year[dy]) if dy in by_year else None
                if r is None:
                    continue
                S = cal.l2s(*r)
                dinp = {"calendar": "solar", "precision": "day", "year": S.year, "month": S.month, "day": S.day,
                        "leap": False}
            elif kd == ("lunar", "day"):
                r = rng.choice(by_year[dy])
                dinp = {"calendar": "lunar", "precision": "day", "year": r[0], "month": r[1], "day": r[2],
                        "leap": r[3]}
            else:
                dinp = {"calendar": "lunar", "precision": "month_day", "year": None, "month": rng.randint(1, 12),
                        "day": rng.randint(1, 30), "leap": False}
            death = G.save_death(cal, dinp)
            try:
                t = G.tuoi_mat(bder["lunar_year"], bder["lunar_year_approx"], death["lunar"]["year"], ng)
            except ValueError:
                continue
            f.write(J({"birth_input": binp, "death_input": dinp, "nguong": ng,
                       "birth": {"lunar_year": bder["lunar_year"], "lunar_year_approx": bder["lunar_year_approx"],
                                 "can_chi": bder["can_chi"]},
                       "death_lunar_year": death["lunar"]["year"], "tuoi": t}) + "\n")
            c += 1
    res["c"] = c
    # (d)
    c = 0
    with open(os.path.join(OUT, "r10_d.jsonl"), "w", encoding="utf-8") as f:
        for Y0 in (1904, 2000):
            for rule in ("feb28", "mar1"):
                for Y in range(1900, 3001):
                    s = G.solar_occurrence((Y0, 2, 29), Y, rule)
                    f.write(J({"orig": {"y": Y0, "m": 2, "d": 29}, "Y": Y, "rule": rule,
                               "date": None if s is None else {"y": s.year, "m": s.month, "d": s.day}}) + "\n")
                    c += 1
    res["d"] = c
    # vi du
    b = G.birth_derive(cal, {"calendar": "solar", "precision": "day", "year": 1990, "month": 1, "day": 20, "leap": False})
    b2 = G.birth_derive(cal, {"calendar": "solar", "precision": "day", "year": 1940, "month": 5, "day": 1, "leap": False})
    dd = G.save_death(cal, {"calendar": "solar", "precision": "day", "year": 2025, "month": 8, "day": 8, "leap": False})
    t = G.tuoi_mat(b2["lunar_year"], False, dd["lunar"]["year"])
    ts = G.tuoi_song(cal, {}, b, dt.date(2026, 11, 6))
    res["examples"] = {"1990-01-20": [b["lunar"], b["can_chi"]], "1940/2025": t, "song_2026-11-06": ts,
                       "29/02/2000->2100": G.solar_occurrence((2000, 2, 29), 2100).isoformat()}
    return res


def main():
    which = sys.argv[1:] or ["oracle", "core", "r10"]
    summary = {}
    for w in which:
        if w in ("oracle", "core"):
            cal = G.Calendar(w)
            tag, bad, notes, _ = gen_r1_d(w, cal)
            summary["r1_d_" + tag] = {"self_check_vs_02_examples": bad,
                                      "save_death": [(c, l, s) for c, l, s in notes]}
            print(w, "d self-check:", bad, file=sys.stderr)
            tag, counts = gen_r1(w, cal)
            summary["r1_" + tag] = counts
            print(w, counts, file=sys.stderr)
        elif w == "r10":
            cal = G.Calendar("oracle")
            summary["r10"] = gen_r10(cal)
            print(summary["r10"], file=sys.stderr)
    p = os.path.join(OUT, "summary_%s.json" % "_".join(which))
    with open(p, "w", encoding="utf-8") as f:
        json.dump(summary, f, ensure_ascii=False, indent=1, default=str)


if __name__ == "__main__":
    main()
