#!/usr/bin/env python3
"""L17 (phan ben kiem dinh tu cham bang lich cua chinh minh): Tet 1900-1999 so voi nguon Viet Nam (cach cham
Bo sung A3) va doi chieu bang TK20 cua Ho Ngoc Duc (1237 thang).

Cach cham A3: nam cac nguon VN thong nhat -> phai khop moi nguon VN; nam cac nguon VN mau thuan (1903, 1935, 1965;
1925 khi xemlicham tu mau thuan) -> cham theo nguon tuan thu quy uoc lich su da chon (gio Bac Kinh truoc 1929 /
UTC+8 den 1967) - tuc VN_XEMLICHAM (1925: trang ngay cua xemlicham, = chogia). Liet ke nguon con lai lech bao nhieu ngay.
Dung: l17.py <outdir chua tet_1900_3000.csv, months_1900_3000.csv, events.csv.gz>
"""
import collections
import csv
import datetime as dt
import gzip
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import tk20  # noqa: E402

REF = os.path.join(HERE, "..", "reference", "tet_published_1900_1999.csv")
CONFLICT_RULE = {1903: "VN_XEMLICHAM", 1935: "VN_XEMLICHAM", 1965: "VN_XEMLICHAM"}
CONFLICT_1925_URL_DAYPAGE = "https://www.xemlicham.com/am-lich/nam/1925/thang/1/ngay/24"


def main(outdir):
    tet = {int(r["lunar_year"]): r["tet_date"] for r in csv.DictReader(open(os.path.join(outdir, "tet_1900_3000.csv")))}
    pub = collections.defaultdict(list)
    for r in csv.DictReader(open(REF, encoding="utf-8")):
        pub[int(r["lunar_year"])].append(r)
    lines = []
    fails = []
    for y in range(1900, 2000):
        vn = [r for r in pub[y] if r["source_id"].startswith("VN_")]
        dates = sorted(set(r["tet_date"] for r in vn))
        mine = tet[y]
        if not vn:
            lines.append((y, mine, "-", "CHƯA ĐỦ BẰNG CHỨNG (không có nguồn VN)"))
            continue
        if len(dates) == 1:
            ok = mine == dates[0]
            verdict = "khớp mọi nguồn VN" if ok else "LỆCH"
            ref = dates[0]
        else:
            if y == 1925:
                ref = [r for r in vn if r["source"] == CONFLICT_1925_URL_DAYPAGE][0]["tet_date"]
                rule = "xemlicham trang ngày (= chogia)"
            else:
                ref = [r for r in vn if r["source_id"] == CONFLICT_RULE[y]][0]["tet_date"]
                rule = CONFLICT_RULE[y]
            ok = mine == ref
            others = "; ".join("%s %s (%+d ngày so với nguồn chấm)" % (
                r["source_id"] + ("[trang năm]" if r["source"].endswith("/%d" % y) else ""), r["tet_date"],
                (dt.date.fromisoformat(r["tet_date"]) - dt.date.fromisoformat(ref)).days) for r in vn)
            verdict = ("khớp nguồn chấm theo A3 (%s); các nguồn: %s" % (rule, others)) if ok else \
                ("LỆCH nguồn chấm theo A3 (%s); các nguồn: %s" % (rule, others))
        if not ok:
            fails.append(y)
        lines.append((y, mine, ref, verdict))
    with open(os.path.join(outdir, "l17_tet_1900_1999.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["year", "tet_ben_kiem_dinh", "tet_nguon_cham", "ket_qua"])
        w.writerows(lines)
    print("L17-Tết: %d năm, lệch: %s" % (len(lines), fails or "0"))
    for y, mine, ref, v in lines:
        if "khớp mọi" not in v:
            print("  %d %s ref=%s %s" % (y, mine, ref, v))

    # ---- TK20
    mine_m = {}
    for r in csv.DictReader(open(os.path.join(outdir, "months_1900_3000.csv"))):
        mine_m[(int(r["lunar_year"]), int(r["month"]), r["leap"] == "1")] = r["start_date"]
    tkm = tk20.months_1900_1999()
    bad = []
    for start, ly, m, leap in tkm:
        got = mine_m.get((ly, m, leap))
        if got != start.isoformat():
            bad.append((ly, m, leap, start.isoformat(), got))
    with open(os.path.join(outdir, "l17_tk20.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["lunar_year", "month", "leap", "tk20_start", "ben_kiem_dinh_start"])
        w.writerows(bad)
    print("L17-TK20: %d/%d tháng khớp" % (len(tkm) - len(bad), len(tkm)))
    for b in bad:
        print("  lệch", b)


if __name__ == "__main__":
    main(sys.argv[1])
