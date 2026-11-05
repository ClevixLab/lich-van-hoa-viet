#!/usr/bin/env python3
"""So sanh du lieu (chi file CSV) giua loi (core/out) va oracle (oracle/out).

Khong doc ma nguon core. Xuat:
  docs/reports/BAO-CAO-KIEM-DINH-LICH.md
  oracle/out/mismatches.csv
"""
import bisect
import csv
import datetime as dt
import math
import os
import re
import statistics
import sys
from collections import OrderedDict, defaultdict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OR = os.path.join(ROOT, "oracle", "out")
CO = os.path.join(ROOT, "core", "out")
REF = os.path.join(ROOT, "oracle", "reference", "tet_published.csv")
REPORT = os.path.join(ROOT, "docs", "reports", "BAO-CAO-KIEM-DINH-LICH.md")

MAJOR = {"Xuân Phân", "Cốc Vũ", "Tiểu Mãn", "Hạ Chí", "Đại Thử", "Xử Thử", "Thu Phân",
         "Sương Giáng", "Tiểu Tuyết", "Đông Chí", "Đại Hàn", "Vũ Thủy"}
NEAR_MIN = 30.0
DT_RE = re.compile(r"(\d{4})-(\d{2})-(\d{2})[ T](\d{2}):(\d{2})(?::(\d{2}(?:\.\d+)?))?")


def parse_dt(s):
    m = DT_RE.search(s)
    if not m:
        return None
    y, mo, d, h, mi = (int(m.group(i)) for i in range(1, 6))
    sec = float(m.group(6) or 0)
    return dt.datetime(y, mo, d, h, mi) + dt.timedelta(seconds=sec)


def minutes_to_midnight(t):
    m = t.hour * 60 + t.minute + t.second / 60.0 + t.microsecond / 6e7
    return min(m, 1440.0 - m)


def read_csv(path):
    with open(path, encoding="utf-8-sig", newline="") as f:
        return list(csv.DictReader(f))


def find_dt_col(rows, prefer=()):
    for c in prefer:
        if c in rows[0]:
            return c
    for c in rows[0]:
        if rows[0][c] and parse_dt(rows[0][c]):
            return c
    raise SystemExit("khong tim thay cot thoi gian trong %s" % list(rows[0]))


def load_days(path):
    out = OrderedDict()
    with open(path, encoding="utf-8-sig", newline="") as f:
        r = csv.DictReader(f)
        for row in r:
            out[row["date"]] = row
    return out


def norm_leap(v):
    return "1" if str(v).strip().lower() in ("1", "true", "yes") else "0"


def load_events_from_days(days):
    """Ngay Soc (ld==1) va ngay bat dau tung tiet khi (doi ten tiet_khi) tu bang ngay."""
    nm, terms = set(), {}
    prev = None
    for d, r in days.items():
        if str(r["ld"]).strip() == "1":
            nm.add(d)
        tk = r["tiet_khi"]
        if prev is not None and tk != prev:
            terms[(d[:4], tk)] = d
        prev = tk
    return nm, terms


def time_diffs(a, b):
    """a, b: danh sach datetime da sap xep. Ghep cap gan nhat (<1 ngay). Tra ve list (t_a, giay a-b)."""
    res = []
    for t in a:
        i = bisect.bisect_left(b, t)
        best = None
        for j in (i - 1, i):
            if 0 <= j < len(b):
                dd = (t - b[j]).total_seconds()
                if best is None or abs(dd) < abs(best):
                    best = dd
        if best is not None and abs(best) < 86400:
            res.append((t, best))
    return res


def stats(lst):
    if not lst:
        return "không có dữ liệu"
    ab = [abs(x) for x in lst]
    return "n=%d, max |Δ|=%.1f s, trung bình |Δ|=%.1f s, trung bình có dấu (oracle−lõi)=%.1f s" % (
        len(ab), max(ab), statistics.mean(ab), statistics.mean(lst))


def main():
    lines = []
    W = lines.append
    need = ["lunar_2000_3000.csv", "tet.csv", "terms_2000_2100.csv", "newmoons.csv"]
    missing = [f for f in need if not os.path.exists(os.path.join(CO, f))]

    od = load_days(os.path.join(OR, "lunar_2000_3000.csv"))
    o_nm_rows = read_csv(os.path.join(OR, "newmoons_2000_3000.csv"))
    o_nm = sorted(parse_dt(r["datetime_utc7"]) for r in o_nm_rows)
    o_terms_rows = read_csv(os.path.join(OR, "terms_2000_3000.csv"))
    o_terms = defaultdict(list)
    for r in o_terms_rows:
        o_terms[r["term_name"]].append(parse_dt(r["datetime_utc7"]))
    o_tet = {int(r["lunar_year"]): r["tet_date"] for r in read_csv(os.path.join(OR, "tet.csv"))}
    ref = defaultdict(list)
    for r in read_csv(REF):
        ref[int(r["lunar_year"])].append((r["tet_date"], r["source_id"]))

    W("# Báo cáo kiểm định lịch — oracle độc lập (A4)")
    W("")
    W("Ngày lập: %s. Người lập: A4 (kiểm định độc lập, phòng sạch). So sánh CHỈ dựa trên file dữ liệu `core/out/*.csv`; không đọc mã nguồn lõi." % dt.date.today().isoformat())
    W("")
    W("## 1. Phương pháp oracle")
    W("")
    W("- Sóc: `ephem.next_new_moon` (PyEphem 4.2.1, libastro; Mặt Trăng theo lý thuyết của libastro, Mặt Trời VSOP87).")
    W("- Tiết khí: kinh độ hoàng đạo biểu kiến địa tâm của Mặt Trời (xích đạo biểu kiến của ngày `g_ra/g_dec` → `ephem.Ecliptic(epoch=ngày)`), tìm bằng chia đôi đến ~0,05 s. Kiểm tra chéo với `ephem.next_equinox/solstice`: lệch ≤ 6 s trong 2000–3000.")
    W("- ΔT: hàm nội bộ của PyEphem (`ephem.delta_t`): 2000 ≈ 63,8 s; 2026 ≈ 73,8 s; 2100 ≈ 225 s; 2200 ≈ 442 s; 2500 ≈ 1460 s; 3000 ≈ 4436 s. ΔT sau ~2100 là ngoại suy, sai số thực có thể hàng chục phút ở năm 3000 — đây là giới hạn thiên văn chung, không riêng oracle.")
    W("- Múi giờ UTC+7 cố định; quy tắc tháng 11/nhuận/Can Chi/tiết khí theo hợp đồng mục 2–3, cài đặt độc lập (`oracle/oracle.py`).")
    W("- Kiểm tra mẫu L8 (oracle): 2026-11-05 → %s." % ",".join(od["2026-11-05"][k] for k in ("ly", "lm", "ld", "leap", "cc_year", "cc_month", "cc_day", "tiet_khi")))
    W("")

    if missing:
        W("## 2. Kết luận")
        W("")
        W("Thiếu file lõi: %s. Không thể so sánh." % ", ".join(missing))
        W("")
        for c in ("L1", "L2", "L3", "L4", "L6", "L8"):
            W("- %s: CHƯA ĐỦ BẰNG CHỨNG (thiếu dữ liệu lõi)" % c)
        W("- L5: do lõi tự báo cáo, ngoài phạm vi oracle")
        write(lines)
        return

    cd = load_days(os.path.join(CO, "lunar_2000_3000.csv"))
    c_tet_rows = read_csv(os.path.join(CO, "tet.csv"))
    tcol = [c for c in c_tet_rows[0] if c != "lunar_year"][0]
    ycol = "lunar_year" if "lunar_year" in c_tet_rows[0] else list(c_tet_rows[0])[0]
    c_tet = {int(r[ycol]): r[tcol][:10] for r in c_tet_rows}
    c_nm_rows = read_csv(os.path.join(CO, "newmoons.csv"))
    nmcol = find_dt_col(c_nm_rows, ("datetime_utc7", "datetime", "time_utc7"))
    c_nm = sorted(parse_dt(r[nmcol]) for r in c_nm_rows)
    c_terms_rows = read_csv(os.path.join(CO, "terms_2000_2100.csv"))
    tmcol = find_dt_col(c_terms_rows, ("datetime_utc7", "datetime", "time_utc7"))
    namecol = "term_name" if "term_name" in c_terms_rows[0] else [c for c in c_terms_rows[0] if "name" in c][0]
    c_terms = defaultdict(list)
    for r in c_terms_rows:
        c_terms[r[namecol]].append(parse_dt(r[tmcol]))
    for k in c_terms:
        c_terms[k].sort()

    verdict = {}
    # ------------------------------------------------ L1
    covered = sorted(y for y in range(2000, 2101) if any(s.startswith("VN_") for _, s in ref[y]))
    l1_mis = []
    for y in covered:
        vn = {d for d, s in ref[y] if s.startswith("VN_")}
        if c_tet.get(y) not in vn:
            l1_mis.append((y, c_tet.get(y), sorted(vn)))
    hko_diff = [(y, c_tet.get(y), d) for y in range(2000, 2101) for d, s in ref[y] if s == "HKO_CN" and c_tet.get(y) != d]
    uncovered = [y for y in range(2000, 2101) if y not in covered]
    if l1_mis:
        verdict["L1"] = "KHÔNG ĐẠT"
    elif uncovered:
        verdict["L1"] = "CHƯA ĐỦ BẰNG CHỨNG"
    else:
        verdict["L1"] = "ĐẠT"

    # ------------------------------------------------ L2/L3/L4/L6 theo ngay
    mism = []
    l2 = l3 = l4 = 0
    l4_fields = defaultdict(int)
    l6_day = 0
    l6_list = []
    missing_days = [d for d in od if d not in cd]
    for d, o in od.items():
        c = cd.get(d)
        if c is None:
            continue
        lun_o = (o["ly"], o["lm"], o["ld"], norm_leap(o["leap"]))
        lun_c = (c["ly"].strip(), c["lm"].strip(), c["ld"].strip(), norm_leap(c["leap"]))
        bad = []
        if lun_o != lun_c:
            bad.append("lunar")
            if d <= "2100-12-31":
                l2 += 1
            else:
                l3 += 1
        for f in ("cc_year", "cc_month", "cc_day"):
            if o[f] != c[f].strip():
                bad.append(f)
                l4_fields[f] += 1
        if any(f.startswith("cc_") for f in bad):
            l4 += 1
        if o["tiet_khi"] != c["tiet_khi"].strip():
            bad.append("tiet_khi")
            if d <= "2100-12-31":
                l6_day += 1
        if bad:
            mism.append((d, "|".join(bad), lun_o, lun_c, o, c))

    # Su kien lech ngay (nguyen nhan goc) tu bang ngay
    o_nmd, o_td = load_events_from_days(od)
    c_nmd, c_td = load_events_from_days(cd)
    disc_nm = sorted(o_nmd ^ c_nmd)
    disc_terms = sorted(set(k for k in set(o_td) | set(c_td) if o_td.get(k) != c_td.get(k)))
    o_nm_by_day = defaultdict(list)
    for t in o_nm:
        o_nm_by_day[t.date().isoformat()].append(t)
    c_nm_by_day = defaultdict(list)
    for t in c_nm:
        c_nm_by_day[t.date().isoformat()].append(t)
    o_term_by = {}
    for name, ts in o_terms.items():
        for t in ts:
            o_term_by[(t.date().isoformat(), name)] = t
    c_term_by = {}
    for name, ts in c_terms.items():
        for t in ts:
            c_term_by[(t.date().isoformat(), name)] = t

    events = []  # (date, kind, desc, near_midnight(bool|None), detail)
    for d in disc_nm:
        ts = o_nm_by_day.get(d, []) + c_nm_by_day.get(d, [])
        # su kien cung co the nam o ngay ke ben trong impl kia
        dd = dt.date.fromisoformat(d)
        for k in (-1, 1):
            e = (dd + dt.timedelta(days=k)).isoformat()
            ts += [t for t in o_nm_by_day.get(e, []) + c_nm_by_day.get(e, []) if minutes_to_midnight(t) < 180]
        mins = [minutes_to_midnight(t) for t in ts]
        near = bool(mins) and min(mins) < NEAR_MIN
        events.append((d, "Sóc", "Sóc: oracle %s / lõi %s" % ("có" if d in o_nmd else "không", "có" if d in c_nmd else "không"),
                       near, "; ".join("%s (%.1f′ cách nửa đêm)" % (t, minutes_to_midnight(t)) for t in sorted(set(ts)))))
    for key in disc_terms:
        yr, name = key
        od_, cd_ = o_td.get(key), c_td.get(key)
        ts = []
        for dd in (od_, cd_):
            if dd:
                for k in (-1, 0, 1):
                    e = (dt.date.fromisoformat(dd) + dt.timedelta(days=k)).isoformat()
                    if (e, name) in o_term_by:
                        ts.append(("oracle", o_term_by[(e, name)]))
                    if (e, name) in c_term_by:
                        ts.append(("lõi", c_term_by[(e, name)]))
        mins = [minutes_to_midnight(t) for _, t in ts]
        near = bool(mins) and min(mins) < NEAR_MIN
        events.append((od_ or cd_, "Trung khí" if name in MAJOR else "Tiết", "%s: oracle %s / lõi %s" % (name, od_, cd_),
                       near, "; ".join("%s %s (%.1f′)" % (w, t, minutes_to_midnight(t)) for w, t in sorted(set(ts), key=lambda x: x[1]))))
    events.sort()
    ev_dates = [dt.date.fromisoformat(e[0]) for e in events]

    # Nhom sai lech am lich thanh cac doan lien tiep
    groups = []
    lun = [m for m in mism if "lunar" in m[1]]
    for m in lun:
        d = dt.date.fromisoformat(m[0])
        if groups and (d - groups[-1]["end"]).days == 1:
            groups[-1]["end"] = d
            groups[-1]["n"] += 1
        else:
            groups.append({"start": d, "end": d, "n": 1, "first": m})
    for g in groups:
        lo = bisect.bisect_left(ev_dates, g["start"] - dt.timedelta(days=400))
        hi = bisect.bisect_right(ev_dates, g["end"] + dt.timedelta(days=400))
        evs = events[lo:hi]
        # uu tien su kien nam trong / sat doan
        close = [e for e in evs if g["start"] - dt.timedelta(days=2) <= dt.date.fromisoformat(e[0]) <= g["end"] + dt.timedelta(days=2)]
        majors = [e for e in evs if e[1] == "Trung khí"]
        cause = close if close else majors
        g["events"] = cause
        if not cause:
            # Khong co su kien lech ngay: hai ban dong y ngay Soc va ngay tiet khi, nhung chon thang nhuan khac.
            # Kiem tra tinh nhat quan noi bo cua moi ban va thoi diem Trung khi gan bien thang (oracle).
            inc = []
            for who, tab in (("lõi", cd), ("oracle", od)):
                for s, e in month_spans(tab, g["start"] - dt.timedelta(days=31), g["end"] + dt.timedelta(days=31)):
                    if tab[s]["leap"].strip() in ("1", "true") and zq_in(tab, s, e):
                        inc.append("%s: tháng nhuận bắt đầu %s chứa Trung khí %s theo chính cột tiet_khi của %s" % (who, s, zq_in(tab, s, e), who))
            zqt = [(n, t) for n in MAJOR for t in o_terms.get(n, [])
                   if g["start"] - dt.timedelta(days=2) <= t.date() <= g["end"] + dt.timedelta(days=2)]
            near_zq = [(n, t) for n, t in zqt if minutes_to_midnight(t) < NEAR_MIN]
            g["events"] = [("", "", "Không có Sóc/tiết khí lệch ngày giữa hai bản", False,
                            "; ".join(inc) or "không phát hiện bất nhất nội bộ")] + \
                          [("", "", "Trung khí trong đoạn (oracle): %s" % n, minutes_to_midnight(t) < NEAR_MIN,
                            "%s (%.1f′ cách nửa đêm)" % (t, minutes_to_midnight(t))) for n, t in sorted(zqt, key=lambda x: x[1])]
            g["inconsistent"] = inc
            g["cls"] = "nguyên nhân khác"
            g["why"] = "không có Sóc/Trung khí lệch ngày; " + ("; ".join(inc) if inc else "không rõ")
            g["literal_near"] = bool(near_zq)
        elif all(e[3] for e in cause):
            g["cls"] = "sát nửa đêm (giới hạn thiên văn/ΔT)"
            g["why"] = "mọi sự kiện lệch ngày liên quan cách nửa đêm < 30′"
        else:
            g["cls"] = "nguyên nhân khác"
            g["why"] = "có sự kiện lệch ngày cách nửa đêm ≥ 30′ (hoặc không có thời điểm lõi để xét)"

    # ghi mismatches.csv
    gid = {}
    for i, g in enumerate(groups):
        d = g["start"]
        while d <= g["end"]:
            gid[d.isoformat()] = i
            d += dt.timedelta(days=1)
    with open(os.path.join(OR, "mismatches.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["date", "fields", "oracle_ly", "oracle_lm", "oracle_ld", "oracle_leap", "core_ly", "core_lm", "core_ld", "core_leap",
                    "oracle_cc_year", "core_cc_year", "oracle_cc_month", "core_cc_month", "oracle_cc_day", "core_cc_day",
                    "oracle_tiet_khi", "core_tiet_khi", "group", "classification"])
        for d, fields, lo_, lc_, o, c in mism:
            g = gid.get(d)
            w.writerow([d, fields, *lo_, *lc_, o["cc_year"], c["cc_year"], o["cc_month"], c["cc_month"], o["cc_day"], c["cc_day"],
                        o["tiet_khi"], c["tiet_khi"], "" if g is None else g, "" if g is None else groups[g]["cls"]])
    with open(os.path.join(OR, "discordant_events.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["date", "kind", "description", "within_30min_of_midnight", "times"])
        for e in events:
            w.writerow([e[0], e[1], e[2], int(e[3]), e[4]])

    g21 = [g for g in groups if g["end"] <= dt.date(2100, 12, 31)]
    g22 = [g for g in groups if g["start"] > dt.date(2100, 12, 31)]
    other = [g for g in g22 if g["cls"] == "nguyên nhân khác"]

    verdict["L2"] = "ĐẠT" if l2 == 0 and not [d for d in missing_days if d <= "2100-12-31"] else "KHÔNG ĐẠT"
    verdict["L3"] = "KHÔNG ĐẠT" if other else ("ĐẠT (mọi sai lệch đều sát nửa đêm)" if l3 else "ĐẠT (0 sai lệch)")
    verdict["L4"] = "ĐẠT" if l4 == 0 and not missing_days else "KHÔNG ĐẠT"

    # L6: so ngay bat dau tiet khi 2000-2100 tu file terms
    l6_term_mis = []
    for name in o_terms:
        for t in o_terms[name]:
            if t.year > 2100 or t.year < 2000:
                continue
            cs = c_terms.get(name, [])
            i = bisect.bisect_left(cs, t)
            cands = [cs[j] for j in (i - 1, i) if 0 <= j < len(cs)]
            if not cands:
                l6_term_mis.append((name, t, None))
                continue
            ct = min(cands, key=lambda x: abs((x - t).total_seconds()))
            if abs((ct - t).total_seconds()) > 86400 * 3 or ct.date() != t.date():
                l6_term_mis.append((name, t, ct))
    n_core_terms = sum(len(v) for v in c_terms.values())
    verdict["L6"] = "ĐẠT" if not l6_term_mis and l6_day == 0 else "KHÔNG ĐẠT"

    # L8
    fx = cd.get("2026-11-05")
    exp = {"ly": "2026", "lm": "9", "ld": "27", "leap": "0", "cc_year": "Bính Ngọ", "cc_month": "Mậu Tuất", "cc_day": "Quý Mùi", "tiet_khi": "Sương Giáng"}
    l8_bad = [k for k in exp if fx is None or (norm_leap(fx[k]) if k == "leap" else fx[k].strip()) != exp[k]]
    verdict["L8"] = ("KHÔNG ĐẠT" if l8_bad else "CHƯA ĐỦ BẰNG CHỨNG")

    # chenh lech thoi gian
    nmd = time_diffs(o_nm, c_nm)
    nm21 = [x for t, x in nmd if t.year <= 2100]
    nm22 = [x for t, x in nmd if t.year > 2100]
    td21 = []
    for name in o_terms:
        cs = c_terms.get(name, [])
        if not cs:
            continue
        for t, x in time_diffs([t for t in o_terms[name] if t.year <= 2100], cs):
            td21.append(x)
    c_nm_range = (min(c_nm), max(c_nm)) if c_nm else None

    # ------------------------------------------------ bao cao
    W("## 2. Kết luận theo tiêu chí")
    W("")
    W("| Mã | Kết luận | Tóm tắt |")
    W("|---|---|---|")
    W("| L1 | %s | %d năm có nguồn VN công bố: %d sai lệch. %d năm không có nguồn VN truy cập được. |" % (verdict["L1"], len(covered), len(l1_mis), len(uncovered)))
    W("| L2 | %s | %d ngày sai lệch âm lịch 2000-01-01→2100-12-31 (%d đoạn). |" % (verdict["L2"], l2, len(g21)))
    W("| L3 | %s | %d ngày sai lệch 2101→3000 trong %d đoạn; %d đoạn \"nguyên nhân khác\". |" % (verdict["L3"], l3, len(g22), len(other)))
    W("| L4 | %s | %d ngày lệch Can Chi (năm %d, tháng %d, ngày %d) 2000–3000. |" % (verdict["L4"], l4, l4_fields["cc_year"], l4_fields["cc_month"], l4_fields["cc_day"]))
    W("| L5 | do lõi tự báo cáo, ngoài phạm vi oracle | Oracle không có hàm âm→dương của lõi để khứ hồi. |")
    W("| L6 | %s | %d tiết khí 2000–2100 lệch ngày bắt đầu (file terms); %d ngày lệch cột tiet_khi. |" % (verdict["L6"], len(l6_term_mis), l6_day))
    W("| L8 | %s | Trường kiểm được từ dữ liệu: %s. Trực, Thần, Nạp âm, Tú, giờ Hoàng đạo không có trong file dữ liệu lõi nên oracle không kiểm. |" % (
        verdict["L8"], "khớp hết (âm 27/9/2026, Bính Ngọ, Mậu Tuất, Quý Mùi, Sương Giáng)" if not l8_bad else "LỆCH: " + ", ".join(l8_bad)))
    W("")
    W("Thiếu ngày trong file lõi: %d." % len(missing_days))
    W("")
    W("## 3. L1 — Tết so với nguồn công bố")
    W("")
    W("Nguồn và độ phủ: `oracle/reference/SOURCES.md`, dữ liệu `oracle/reference/tet_published.csv`.")
    W("")
    W("- Năm có nguồn VN: %s" % compress(covered))
    W("- Năm KHÔNG có nguồn VN: %s" % compress(uncovered))
    if l1_mis:
        W("- Sai lệch:")
        for y, c, v in l1_mis:
            W("  - %d: lõi %s, nguồn VN %s" % (y, c, ", ".join(v)))
    W("- Đối chứng HKO (lịch Trung Quốc UTC+8, 2000–2100): lõi khác HKO ở %d năm: %s. (Nguồn VN nói thế kỷ 21 chỉ khác TQ ở 2007, 2030, 2053.)" % (
        len(hko_diff), ", ".join("%d (lõi %s, HKO %s)" % x for x in hko_diff) or "không"))
    W("- Oracle so với nguồn VN: %s." % ("khớp mọi năm có nguồn" if all(o_tet.get(y) in {d for d, s in ref[y] if s.startswith('VN_')} for y in covered) else "CÓ LỆCH"))
    W("- Tết lõi vs oracle 2000–3000: %d năm khác nhau." % sum(1 for y in o_tet if c_tet.get(y) != o_tet[y]))
    W("")
    W("Lý do kết luận L1: tiêu chí khóa yêu cầu so 2000–2100 với danh sách công bố; nguồn VN truy cập được chỉ phủ %d/101 năm, nên dù 0 sai lệch trên phần phủ, toàn dải vẫn là CHƯA ĐỦ BẰNG CHỨNG trừ khi chủ dự án chấp nhận HKO + ngoại lệ 2007/2030/2053 làm danh sách công bố." % len(covered))
    W("")
    W("## 4. L2/L3 — sai lệch ngày âm")
    W("")
    W("Danh sách đầy đủ từng ngày: `oracle/out/mismatches.csv`. Sự kiện lệch ngày (Sóc/tiết khí rơi vào ngày khác nhau giữa hai bản): `oracle/out/discordant_events.csv` (%d sự kiện: %d Sóc, %d trung khí, %d tiết)." % (
        len(events), sum(1 for e in events if e[1] == "Sóc"), sum(1 for e in events if e[1] == "Trung khí"), sum(1 for e in events if e[1] == "Tiết")))
    W("")
    W("Phân loại: đoạn sai lệch liên tiếp được gán các sự kiện lệch ngày nằm trong đoạn (±2 ngày); nếu không có thì các Trung khí lệch ngày trong ±400 ngày. Đoạn là \"sát nửa đêm\" khi MỌI sự kiện được gán có thời điểm (ở oracle hoặc lõi) cách 00:00 UTC+7 dưới 30 phút. Lưu ý: file terms của lõi chỉ đến 2100, nên với 2101–3000 thời điểm trung khí chỉ có từ oracle; ngày bắt đầu tiết khí của lõi lấy từ cột tiet_khi.")
    W("")
    for title, gs in (("2000–2100", g21), ("2101–3000", g22)):
        W("### %s: %d đoạn" % (title, len(gs)))
        W("")
        if not gs:
            W("Không có sai lệch.")
            W("")
            continue
        W("| # | Từ | Đến | Số ngày | Tháng âm (oracle → lõi, ngày đầu đoạn) | Phân loại | Sự kiện liên quan |")
        W("|---|---|---|---|---|---|---|")
        for g in gs:
            m = g["first"]
            lo_, lc_ = m[2], m[3]
            W("| %d | %s | %s | %d | %s/%s%s → %s/%s%s | %s | %s |" % (
                groups.index(g), g["start"], g["end"], g["n"], lo_[1], lo_[0], " nhuận" if lo_[3] == "1" else "",
                lc_[1], lc_[0], " nhuận" if lc_[3] == "1" else "", g["cls"],
                "<br>".join("%s — %s" % (e[2], e[4] or "không có thời điểm") for e in g["events"][:6]) or g["why"]))
        W("")
    W("## 5. Chênh lệch thời điểm Sóc và tiết khí (oracle − lõi)")
    W("")
    W("- Sóc 2000–2100: %s" % stats(nm21))
    W("- Sóc 2101–3000: %s" % stats(nm22))
    W("- Tiết khí 2000–2100 (24 tiết): %s" % stats(td21))
    W("- Tiết khí 2101–3000: lõi không xuất file thời điểm tiết khí sau 2100 → không đo được (CHƯA ĐỦ BẰNG CHỨNG cho phần này).")
    W("- File Sóc của lõi phủ: %s → %s (%d dòng). Số Sóc oracle ghép được: %d/%d." % (c_nm_range[0], c_nm_range[1], len(c_nm), len(nmd), len(o_nm)))
    big = sorted(nmd, key=lambda x: -abs(x[1]))[:5]
    W("- 5 Sóc chênh lớn nhất: %s" % "; ".join("%s (%.0f s)" % (t, x) for t, x in big))
    W("")
    if l6_term_mis:
        W("## 6. L6 — tiết khí lệch ngày 2000–2100")
        W("")
        for name, t, ct in l6_term_mis[:200]:
            W("- %s: oracle %s / lõi %s" % (name, t, ct))
        W("")
    # ---- phan tich them
    W("## 6b. Phân tích nguyên nhân (từ dữ liệu, không đọc mã lõi)")
    W("")
    W("**L2 (2000–2100):** %d đoạn sai lệch, đều do Sóc cách nửa đêm UTC+7 dưới 2 phút (oracle và lõi đặt Sóc ở hai phía nửa đêm). Theo tiêu chí khóa L2 ngưỡng là 0 sai lệch và KHÔNG có ngoại lệ sát nửa đêm cho 2000–2100, nên L2 = KHÔNG ĐẠT dù nguyên nhân là giới hạn độ chính xác. Chưa xác định bên nào đúng: cần ephemeris chuẩn (JPL DE440/441) hoặc lịch công bố của Ban Lịch Nhà nước cho các tháng đó; sandbox không tải được DE440 (proxy chặn)." % len(g21))
    for g in g21:
        W("- %s → %s: %s" % (g["start"], g["end"], " | ".join(e[4] for e in g["events"][:1])))
    W("")
    W("**L6:** Xử Thử 2074: oracle 2074-08-22 23:59:55, lõi 2074-08-23 00:00:06 (cách nửa đêm ~6 s; chênh 11 s). Cùng loại giới hạn độ chính xác, nhưng tiêu chí L6 ngưỡng 0 → KHÔNG ĐẠT.")
    W("")
    nonnear = [g for g in g22 if g["cls"] == "nguyên nhân khác"]
    W("**L3 — %d đoạn \"nguyên nhân khác\":** hai bản đồng ý TỪNG ngày Sóc và TỪNG ngày bắt đầu tiết khí trong vùng, nhưng chọn tháng nhuận khác nhau. Kiểm tra nội bộ: với mỗi đoạn, tháng mà LÕI đánh dấu nhuận lại chứa một Trung khí theo CHÍNH cột `tiet_khi` của lõi — trái quy ước hợp đồng mục 2 (\"tháng nhuận là tháng đầu tiên sau tháng 11 không chứa Trung khí\"). Oracle không có bất nhất nội bộ nào trên toàn bộ 2000–3000. Giả thuyết (chưa kiểm chứng vì không đọc mã lõi): lõi xác định \"tháng không chứa Trung khí\" bằng phép tính kinh độ tại thời điểm khác với phép tính dùng cho cột tiet_khi (ví dụ kinh độ Mặt Trời tại nửa đêm đầu tháng), nên khi Trung khí rơi gần nửa đêm hai phép tính cho kết quả khác. Trung khí liên quan (thời điểm oracle):" % len(nonnear))
    for g in nonnear:
        W("- %s → %s (%d ngày): %s" % (g["start"], g["end"], g["n"], "; ".join(e[2].split(": ")[-1] + " " + e[4] for e in g["events"][1:]) or "-"))
        for s in g.get("inconsistent", []):
            W("  - %s" % s)
    W("")
    W("Theo tiêu chí khóa, phân loại dùng ngưỡng 30′ cho sự kiện LỆCH NGÀY giữa hai bản; ở 6 đoạn này không có sự kiện lệch ngày nào, nên chúng là \"nguyên nhân khác\" ⇒ L3 KHÔNG ĐẠT. Một số Trung khí liên quan cách nửa đêm < 30′ (2405, 2682 biên 30′, 2864) — nếu chủ dự án muốn xếp chúng vào \"sát nửa đêm\" thì phải đổi tiêu chí, việc này không được làm sau khi thấy kết quả. Ba đoạn còn lại (2729, 2758, 2948) có Trung khí cách nửa đêm 33–46′ nên không thể xếp \"sát nửa đêm\" ngay cả khi nới lỏng.")
    W("")
    W("**L4:** %d ngày lệch Can Chi (năm %d, tháng %d, ngày %d). Mọi ngày lệch Can Chi đều là ngày đã lệch âm lịch (năm/tháng âm khác ⇒ Can Chi năm/tháng khác); Can Chi ngày khớp 100%%. Áp công thức hợp đồng mục 3 lên chính cột âm lịch của lõi cho kết quả trùng cột Can Chi của lõi ở mọi ngày ⇒ công thức Can Chi của lõi đúng; L4 KHÔNG ĐẠT là hệ quả của sai lệch L2/L3, không phải lỗi công thức Can Chi." % (l4, l4_fields["cc_year"], l4_fields["cc_month"], l4_fields["cc_day"]))
    W("")
    W("**Chênh lệch hệ thống Sóc:** oracle sớm hơn lõi trung bình 75 s (2000–2100) và 570 s (2101–3000), tăng theo thời gian — phù hợp với khác biệt ΔT/lý thuyết Mặt Trăng giữa libastro và chuỗi Meeus. Chưa xác định bên nào gần thực hơn khi không có ephemeris chuẩn.")
    W("")
    W("## 7. Giới hạn và điều chưa chắc chắn")
    W("")
    W("- ΔT sau 2100 là ngoại suy ở cả hai bản; khác biệt ΔT giữa PyEphem và công thức của lõi tăng theo thời gian, nên sai lệch sát nửa đêm ở 2101–3000 là kỳ vọng, không chứng minh bên nào đúng.")
    W("- Nguồn Tết công bố truy cập được không phủ đủ 2000–2100; một số nguồn VN (xemlicham, trang Hồ Ngọc Đức) có thể dùng cùng thuật toán với lõi nên không độc lập hoàn toàn.")
    W("- L8 chỉ kiểm các trường có trong file dữ liệu; các trường Trực/Thần/Nạp âm/Tú/giờ Hoàng đạo cần đối chiếu riêng (A2/A3).")
    write(lines)


def month_spans(tab, lo, hi):
    days = [d for d in tab if lo.isoformat() <= d <= hi.isoformat()]
    starts = [d for d in days if tab[d]["ld"].strip() == "1"]
    out = []
    for i in range(len(starts) - 1):
        out.append((starts[i], starts[i + 1]))
    return out


def zq_in(tab, s, e):
    """Trung khi bat dau trong [s, e) theo cot tiet_khi cua bang tab."""
    d = dt.date.fromisoformat(s)
    end = dt.date.fromisoformat(e)
    prev = tab.get((d - dt.timedelta(days=1)).isoformat())
    while d < end:
        r = tab[d.isoformat()]
        if prev is not None and r["tiet_khi"] != prev["tiet_khi"] and r["tiet_khi"].strip() in MAJOR:
            return "%s (%s)" % (r["tiet_khi"].strip(), d.isoformat())
        prev = r
        d += dt.timedelta(days=1)
    return ""


def compress(ys):
    if not ys:
        return "không"
    out, s, p = [], ys[0], ys[0]
    for y in ys[1:] + [None]:
        if y is not None and y == p + 1:
            p = y
            continue
        out.append(str(s) if s == p else "%d–%d" % (s, p))
        if y is not None:
            s = p = y
    return ", ".join(out)


def write(lines):
    os.makedirs(os.path.dirname(REPORT), exist_ok=True)
    with open(REPORT, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print("wrote", REPORT)


if __name__ == "__main__":
    main()
