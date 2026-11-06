#!/usr/bin/env python3
"""So sanh voi ben xay dung (chay SAU KHI ket qua cua ben kiem dinh da commit).

L14: thoi diem TT cua moi Soc / tiet khi: tep kiem toan cua ben xay dung (kind,index,n_cs,jd_tt,tt_iso; soc/tk)
     so voi out/instants_tt.csv.gz. Nguong |d| <= 2 s.
L15: lich tung ngay: dump hop den `lvhv-dump --method jpl lunar-csv 1900-01-01 3000-12-31`
     (date,ly,lm,ld,leap,cc_year,cc_month,cc_day,tiet_khi) so voi out/days_1900_3000.csv.gz.
     Ngay lech duoc nhom thanh doan; doan do su kien ma mot ben dat cach nua dem < 2 s gay ra -> "khong phan dinh
     duoc" (khong tinh). Thoi diem cua ben xay dung duoc quy ra ngay dia phuong bang CUNG Delta T va quy tac mui gio
     cua ben kiem dinh (deltat.py, calendar_de441.py).
Dung: compare_builder.py <outdir> <builder_events.csv.gz> <builder_lunar.csv.gz> <reportdir>
"""
import bisect
import csv
import gzip
import io
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import calendar_de441 as C  # noqa: E402
import deltat  # noqa: E402

FIELDS = ["ly", "lm", "ld", "leap", "cc_year", "cc_month", "cc_day", "tiet_khi"]


def read_csv_gz(path):
    op = gzip.open if path.endswith(".gz") else open
    with op(path, "rt", encoding="utf-8") as f:
        txt = "".join(ln for ln in f if not ln.startswith("#"))
    return list(csv.DictReader(io.StringIO(txt)))


def local_info(jd_tt, tz):
    u = jd_tt - deltat.delta_t_tt(jd_tt) / 86400.0
    off = tz(u)
    x = u + off + 0.5
    j = math.floor(x)
    s = (x - j) * 86400.0
    return int(j), s, min(s, 86400.0 - s), C.fmt(u + off)


def main(outdir, b_events, b_lunar, repdir):
    os.makedirs(repdir, exist_ok=True)
    tz = C.make_tz(C.SWITCHES_1968["1968-01-01T00+08"])
    # ---------------- L14
    mine = {"newmoon": [], "term": []}
    for r in read_csv_gz(os.path.join(outdir, "instants_tt.csv.gz")):
        mine[r["kind"]].append((float(r["jd_tt"]), int(r["lon_deg"])))
    theirs = []
    for r in read_csv_gz(b_events):
        kind = {"soc": "newmoon", "tk": "term"}[r["kind"]]
        lon = 15 * int(r["index"]) % 360 if kind == "term" else 0
        # n_cs (so nguyen, 0.01 s tu J2000.0 TT, docs/03 §17.5) chinh xac hon cot jd_tt (8 chu so thap phan)
        t = 2451545.0 + int(r["n_cs"]) / 8640000.0 if r.get("n_cs") else float(r["jd_tt"])
        theirs.append((kind, lon, t, r["index"], r.get("n_cs", "")))
    l14 = []
    used = {"newmoon": set(), "term": set()}
    unmatched = []
    for kind, lon, t, idx, ncs in theirs:
        arr = [x[0] for x in mine[kind]]
        i = bisect.bisect_left(arr, t)
        cands = [q for q in (i - 1, i) if 0 <= q < len(arr)]
        q = min(cands, key=lambda q: abs(arr[q] - t))
        if abs(arr[q] - t) > 1.0 or (kind == "term" and mine[kind][q][1] != lon):
            unmatched.append((kind, idx, t))
            continue
        used[kind].add(q)
        # kiem them: n_cs dung la jd_tt lam tron 0.01 s (thong tin)
        d = (arr[q] - t) * 86400.0
        l14.append((kind, idx, lon, t, arr[q], d))
    missing = [(k, mine[k][q]) for k in mine for q in range(len(mine[k])) if q not in used[k]]
    with gzip.open(os.path.join(repdir, "l14_diff.csv.gz"), "wt", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["kind", "index", "lon_deg", "builder_jd_tt", "verifier_jd_tt", "diff_s_verifier_minus_builder"])
        for k, idx, lon, t, m, d in l14:
            w.writerow([k, idx, lon, "%.9f" % t, "%.9f" % m, "%.4f" % d])
    absd = [abs(x[5]) for x in l14]
    mx = max(l14, key=lambda x: abs(x[5]))
    over = [x for x in l14 if abs(x[5]) > 2.0]
    print("L14: so su kien so khop: %d (soc %d, tk %d); ben XD khong co cap: %d; ben KD khong co cap: %d" % (
        len(l14), sum(1 for x in l14 if x[0] == "newmoon"), sum(1 for x in l14 if x[0] == "term"),
        len(unmatched), len(missing)))
    for k in ("newmoon", "term"):
        ds = [x[5] for x in l14 if x[0] == k]
        if ds:
            print("  %s: max|d| = %.4f s, TB|d| = %.4f s, TB co dau = %+.4f s" % (
                k, max(abs(d) for d in ds), sum(abs(d) for d in ds) / len(ds), sum(ds) / len(ds)))
    print("  max |d| = %.4f s (%s %s %s); vuot 2 s: %d" % (abs(mx[5]), mx[0], mx[1], C.fmt(mx[3]), len(over)))
    for x in over:
        print("  VUOT:", x)
    for u in unmatched[:20]:
        print("  XD khong cap:", u)
    if missing:
        print("  KD khong cap (dau/cuoi):", missing[:3], "...", missing[-3:])
    # phan bo theo the ky
    buckets = {}
    for k, idx, lon, t, m, d in l14:
        c = C.jdn_to_date(int(t + 0.5)).year // 100 * 100
        b = buckets.setdefault((k, c), [0, 0.0])
        b[0] += 1
        b[1] = max(b[1], abs(d))
    with open(os.path.join(repdir, "l14_by_century.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["kind", "century", "n", "max_abs_diff_s"])
        for (k, c), (n, m) in sorted(buckets.items()):
            w.writerow([k, c, n, "%.4f" % m])

    # ---------------- su kien dat khac ngay
    ev_diff = []
    for k, idx, lon, t, m, d in l14:
        jb, sb, nb, fb = local_info(t, tz)
        jm, sm, nm, fm = local_info(m, tz)
        if jb != jm:
            ev_diff.append((k, idx, lon, fb, fm, nb, nm, min(jb, jm), max(jb, jm)))
    with open(os.path.join(repdir, "events_day_differ.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["kind", "index", "lon_deg", "builder_local", "verifier_local", "builder_secs_to_midnight",
                    "verifier_secs_to_midnight"])
        for e in ev_diff:
            w.writerow(list(e[:5]) + ["%.3f" % e[5], "%.3f" % e[6]])
    print("Su kien hai ben dat khac ngay dia phuong:", len(ev_diff))
    for e in ev_diff:
        print("  ", e[:7])

    # ---------------- L15
    if b_lunar == "-":
        return
    mine_days = {r["date"]: r for r in read_csv_gz(os.path.join(outdir, "days_1900_3000.csv.gz"))}
    their_days = {}
    for r in read_csv_gz(b_lunar):
        their_days[r["date"]] = r
    missing_days = sorted(set(mine_days) - set(their_days))
    extra_days = sorted(set(their_days) - set(mine_days))
    mism = []
    for dte in sorted(mine_days):
        if dte not in their_days:
            continue
        a, b = mine_days[dte], their_days[dte]
        bad = [f for f in FIELDS if str(a[f]).strip() != str(b[f]).strip()]
        if bad:
            mism.append((dte, bad, a, b))
    # nhom doan
    segs = []
    for dte, bad, a, b in mism:
        j = C.date_to_jdn(C.dt.date.fromisoformat(dte))
        if segs and segs[-1][1] == j - 1:
            segs[-1][1] = j
            segs[-1][2].append((dte, bad, a, b))
        else:
            segs.append([j, j, [(dte, bad, a, b)]])
    rows = []
    n_unres = n_fail = 0
    for j0, j1, items in segs:
        # su kien khac ngay co anh huong: nam trong [j0 - 31, j1 + 1]
        cause = [e for e in ev_diff if j0 - 31 <= e[7] <= j1 + 1]
        if cause and all(min(e[5], e[6]) < 2.0 for e in cause):
            cls = "không phân định được (sự kiện cách nửa đêm < 2 s)"
            n_unres += j1 - j0 + 1
        elif cause:
            cls = "SAI LỆCH (sự kiện khác ngày, cách nửa đêm ≥ 2 s)"
            n_fail += j1 - j0 + 1
        else:
            cls = "SAI LỆCH (không có sự kiện khác ngày)"
            n_fail += j1 - j0 + 1
        fields = sorted(set(f for it in items for f in it[1]))
        rows.append((C.jdn_to_date(j0).isoformat(), C.jdn_to_date(j1).isoformat(), j1 - j0 + 1, "|".join(fields), cls,
                     "; ".join("%s %s idx=%s XD %s / KD %s (%.3f s / %.3f s)" % (e[0], e[2], e[1], e[3], e[4], e[5], e[6])
                               for e in cause)))
    with open(os.path.join(repdir, "l15_mismatch_segments.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["from", "to", "days", "fields", "classification", "events"])
        w.writerows(rows)
    with gzip.open(os.path.join(repdir, "l15_mismatch_days.csv.gz"), "wt", encoding="utf-8", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["date", "fields"] + ["kd_" + x for x in FIELDS] + ["xd_" + x for x in FIELDS])
        for dte, bad, a, b in mism:
            w.writerow([dte, "|".join(bad)] + [a[x] for x in FIELDS] + [b[x] for x in FIELDS])
    print("L15: ngay so: %d; thieu o ben XD: %d; thua o ben XD: %d; ngay lech: %d trong %d doan; "
          "khong phan dinh duoc: %d ngay; sai lech tinh: %d ngay" % (
              len(mine_days) - len(missing_days), len(missing_days), len(extra_days), len(mism), len(segs),
              n_unres, n_fail))
    for r in rows:
        print("  ", r)


if __name__ == "__main__":
    main(*sys.argv[1:5])
