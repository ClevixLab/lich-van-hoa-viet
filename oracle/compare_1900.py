#!/usr/bin/env python3
"""So sanh 1900-1999 (Bo sung A1): chi doc FILE DU LIEU cua loi (core/out), khong doc ma nguon.
Ket qua in ra stdout dang JSON + ghi oracle/out/mismatches_1900_1999_<method>.csv,
oracle/out/nm_diff_1900_1999_<method>.csv. Bao cao viet tay dua tren ket qua nay.
"""
import csv
import datetime as dt
import gzip
import json
import os
import re
import statistics
import sys
from collections import defaultdict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OR = os.path.join(ROOT, "oracle", "out")
CO = os.path.join(ROOT, "core", "out")
REF = os.path.join(ROOT, "oracle", "reference", "tet_published_1900_1999.csv")
DT_RE = re.compile(r"(\d{4})-(\d{2})-(\d{2})[ T](\d{2}):(\d{2})(?::(\d{2}(?:\.\d+)?))?(?:([+-])(\d{2}):?(\d{2})(?::?(\d{2}))?)?")
SWITCH_UT = dt.datetime(1967, 12, 31, 17, 0)
NEAR = 30.0
FIELDS = ["ly", "lm", "ld", "leap"]


def parse(s):
    """-> (naive local datetime, utc offset hours or None)"""
    m = DT_RE.search(s)
    if not m:
        return None, None
    y, mo, d, h, mi = (int(m.group(i)) for i in range(1, 6))
    t = dt.datetime(y, mo, d, h, mi) + dt.timedelta(seconds=float(m.group(6) or 0))
    off = None
    if m.group(7):
        off = (1 if m.group(7) == "+" else -1) * (int(m.group(8)) + int(m.group(9)) / 60 + int(m.group(10) or 0) / 3600)
    return t, off


def rd(p):
    with open(p, encoding="utf-8-sig", newline="") as f:
        return list(csv.DictReader(f))


def leap(v):
    return "1" if str(v).strip().lower() in ("1", "true", "yes") else "0"


def mins_mid(t):
    m = t.hour * 60 + t.minute + t.second / 60
    return min(m, 1440 - m)


def oracle_nm():
    out = []
    for r in rd(os.path.join(OR, "newmoons_1900_1999.csv")):
        t, _ = parse(r["datetime_local"])
        _, off = parse("2000-01-01 00:00:00" + r["utc_offset"])
        out.append((t - dt.timedelta(hours=off), t))  # (UT, local)
    return out


def core_nm(path):
    rows = rd(path)
    col = [c for c in rows[0] if parse(rows[0][c])[0] is not None][0]
    out = []
    for r in rows:
        t, off = parse(r[col])
        if off is None:
            raise SystemExit("core newmoons khong co offset: " + r[col])
        out.append((t - dt.timedelta(hours=off), t, off))
    return out


def continuity(col, a, b):
    days = [r for d, r in sorted(col.items()) if a <= d <= b]
    expect = (dt.date.fromisoformat(b) - dt.date.fromisoformat(a)).days + 1
    err = []
    for p, q in zip(days, days[1:]):
        pl, ql = int(p["ld"]), int(q["ld"])
        same = (p["ly"], p["lm"], leap(p["leap"])) == (q["ly"], q["lm"], leap(q["leap"]))
        if (ql == pl + 1 and same) or (ql == 1 and pl in (29, 30) and not same):
            continue
        err.append((p["date"], q["date"], pl, ql))
    keys = [(r["ly"], r["lm"], leap(r["leap"]), r["ld"]) for r in days]
    return {"days": len(days), "expected": expect, "n_seq_errors": len(err), "seq_errors": err[:10],
            "dup_lunar": len(keys) - len(set(keys))}


def main(method):
    sub = "" if method == "hnd" else "precise"
    cl = os.path.join(CO, sub, "lunar_1900_1999.csv")
    ct = os.path.join(CO, sub, "tet_1900_1999.csv")
    cn = os.path.join(CO, sub, "newmoons_1900_1999.csv")
    res = {"method": method}
    orl = {r["date"]: r for r in rd(os.path.join(OR, "lunar_1900_1999.csv"))}
    col = {r["date"]: r for r in rd(cl)}
    res["core_days"] = len(col)
    res["missing_in_core"] = sorted(set(orl) - set(col))[:20]
    res["n_missing"] = len(set(orl) - set(col))
    mism = []
    for d in sorted(orl):
        if d not in col:
            continue
        o, c = orl[d], col[d]
        ov = (o["ly"], o["lm"], o["ld"], leap(o["leap"]))
        cv = (c["ly"].strip(), c["lm"].strip(), c["ld"].strip(), leap(c["leap"]))
        cc = [k for k in ("cc_year", "cc_month", "cc_day", "tiet_khi") if k in c and c[k] != o[k]]
        if ov != cv or cc:
            mism.append((d, ov, cv, cc))
    res["n_mismatch_days"] = len([m for m in mism if m[1] != m[2]])
    res["n_mismatch_other_fields"] = len([m for m in mism if m[1] == m[2] and m[3]])
    # group runs
    groups = []
    for m in mism:
        if m[1] == m[2]:
            continue
        d = dt.date.fromisoformat(m[0])
        if groups and (d - groups[-1][-1][0]).days == 1:
            groups[-1].append((d, m))
        else:
            groups.append([(d, m)])
    # events
    onm = oracle_nm()
    cnm = core_nm(cn) if os.path.exists(cn) else []
    res["core_nm_count"] = len(cnm)
    # match new moons
    diffs = []
    import bisect
    cut = [x[0] for x in cnm]
    pairs = []
    for ut, loc in onm:
        i = bisect.bisect_left(cut, ut)
        best = None
        for j in (i - 1, i):
            if 0 <= j < len(cnm):
                dd = (ut - cnm[j][0]).total_seconds()
                if best is None or abs(dd) < abs(best[0]):
                    best = (dd, j)
        if best and abs(best[0]) < 86400 * 3:
            pairs.append((ut, loc, cnm[best[1]], best[0]))
            diffs.append(best[0])
    res["nm_matched"] = len(pairs)
    if diffs:
        res["nm_diff"] = {"n": len(diffs), "max_abs_s": round(max(abs(x) for x in diffs), 1),
                          "mean_abs_s": round(statistics.mean(abs(x) for x in diffs), 1),
                          "mean_signed_s": round(statistics.mean(diffs), 1)}
        top = sorted(pairs, key=lambda p: -abs(p[3]))[:5]
        res["nm_top"] = [(p[1].isoformat(sep=" "), round(p[3], 1)) for p in top]
    # offsets used by core around switch
    offs = defaultdict(list)
    for ut, loc, off in cnm:
        offs[off].append(loc)
    res["core_offsets"] = {"%.6f" % k: [min(v).isoformat(sep=" "), max(v).isoformat(sep=" "), len(v)] for k, v in offs.items()}
    # discordant new-moon days
    disc = []
    for ut, loc, (cut_, cloc, coff), d in pairs:
        if loc.date() != cloc.date():
            disc.append((loc, cloc, coff, round(d, 1), round(min(mins_mid(loc), mins_mid(cloc)), 2)))
    res["nm_discordant_days"] = [(a.isoformat(sep=" "), b.isoformat(sep=" "), off, d, mm) for a, b, off, d, mm in disc]
    # classify groups
    gout = []
    for g in groups:
        a, b = g[0][0], g[-1][0]
        rel = [x for x in disc if (a - dt.timedelta(days=2)) <= x[0].date() <= (b + dt.timedelta(days=2))
               or (a - dt.timedelta(days=2)) <= x[1].date() <= (b + dt.timedelta(days=2))]
        near = bool(rel) and all(x[4] < NEAR for x in rel)
        gout.append({"from": a.isoformat(), "to": b.isoformat(), "days": len(g),
                     "oracle_first": g[0][1][1], "core_first": g[0][1][2],
                     "class": "sat_nua_dem" if near else "nguyen_nhan_khac",
                     "events": [(x[0].isoformat(sep=" "), x[1].isoformat(sep=" "), x[2], x[4]) for x in rel]})
    res["groups"] = gout
    with open(os.path.join(OR, "mismatches_1900_1999_%s.csv" % method), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["date", "oracle_ly", "oracle_lm", "oracle_ld", "oracle_leap", "core_ly", "core_lm", "core_ld", "core_leap", "other_fields_differ"])
        for d, ov, cv, cc in mism:
            w.writerow([d, *ov, *cv, "|".join(cc)])
    with open(os.path.join(OR, "nm_diff_1900_1999_%s.csv" % method), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["oracle_local", "core_local", "core_offset", "oracle_minus_core_s"])
        for ut, loc, (cut_, cloc, coff), d in pairs:
            w.writerow([loc.isoformat(sep=" "), cloc.isoformat(sep=" "), coff, round(d, 1)])
    # Tet
    ctet = {r["lunar_year"]: r["tet_date"] for r in rd(ct)}
    otet = {r["lunar_year"]: r["tet_date"] for r in rd(os.path.join(OR, "tet_1900_1999.csv"))}
    res["tet_core_vs_oracle"] = sorted((y, otet[y], ctet.get(y)) for y in otet if ctet.get(y) != otet[y])
    ref = defaultdict(dict)
    for r in rd(REF):
        ref[r["lunar_year"]].setdefault(r["source_id"], set()).add(r["tet_date"])
    l9 = {}
    for y in range(1900, 2000):
        ys = str(y)
        vn = {s: v for s, v in ref.get(ys, {}).items() if s.startswith("VN_")}
        l9[ys] = {"core": ctet.get(ys), "vn": {s: sorted(v) for s, v in vn.items()},
                  "agree": {s: (ctet.get(ys) in v) for s, v in vn.items()}}
    res["l9"] = l9
    # L11
    days = [r for d, r in sorted(col.items()) if "1967-01-01" <= d <= "1969-12-31"]
    expect = (dt.date(1969, 12, 31) - dt.date(1967, 1, 1)).days + 1
    datesok = len(days) == expect and len({r["date"] for r in days}) == expect
    seq_err = []
    for p, q in zip(days, days[1:]):
        pl, ql = int(p["ld"]), int(q["ld"])
        if ql == pl + 1 and (p["ly"], p["lm"], leap(p["leap"])) == (q["ly"], q["lm"], leap(q["leap"])):
            continue
        if ql == 1 and pl in (29, 30) and (p["ly"], p["lm"], leap(p["leap"])) != (q["ly"], q["lm"], leap(q["leap"])):
            continue
        seq_err.append((p["date"], q["date"], pl, ql))
    lunar_keys = [(r["ly"], r["lm"], leap(r["leap"]), r["ld"]) for r in days]
    dup = len(lunar_keys) - len(set(lunar_keys))
    res["l11_1929"] = continuity(col, "1928-01-01", "1930-12-31")
    res["l11"] = {"days": len(days), "expected": expect, "dates_ok": datesok, "seq_errors": seq_err[:10],
                  "n_seq_errors": len(seq_err), "dup_lunar": dup, "tet1968": ctet.get("1968")}
    return res


if __name__ == "__main__":
    out = {m: main(m) for m in sys.argv[1:] or ["hnd", "precise"]}
    json.dump(out, sys.stdout, ensure_ascii=False, indent=1, default=str)
