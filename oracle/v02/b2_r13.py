#!/usr/bin/env python3
"""R13 doc lap — danh sach su kien theo tuan / thang / quy / nam (04 Bo sung B2 R13; 02 §7.6 va Phu luc A.6
o nhanh v0.2-w3-core, commit 206fceb). Bo sinh lan xuat hien: b2_occ (lich DE441 = phuong phap Jpl).

  b2_r13.py DATASET [FROM TO]            -> JSONL theo A.6 (mot dong cho moi (ky, bo loc))
  b2_r13.py --check DATASET FILE [FROM TO] -> kiem (c) nhat quan va (d) ngay dau/cuoi tren FILE (JSON ra stdout)
"""
import datetime as dt, json, os, sys
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import b2_occ as O  # noqa: E402

D1 = dt.timedelta(days=1)
KINDS = ["tuan", "thang", "quy", "nam"]
FILTERS = ["tat_ca", "da_mat", "dang_song"]


def month_end(y, m):
    return (dt.date(y + (m == 12), m % 12 + 1, 1)) - D1


def period_bounds(kind, a):
    if kind == "tuan":
        f = a - dt.timedelta(days=a.weekday())          # weekday(): 0 = Thu Hai
        return f, f + dt.timedelta(days=6)
    if kind == "thang":
        return dt.date(a.year, a.month, 1), month_end(a.year, a.month)
    if kind == "quy":
        q = (a.month - 1) // 3
        return dt.date(a.year, 3 * q + 1, 1), month_end(a.year, 3 * q + 3)
    return dt.date(a.year, 1, 1), dt.date(a.year, 12, 31)


def periods(kind, F, T):
    out = []
    f, t = period_bounds(kind, F)
    while f <= T:
        out.append((max(f, F), f, t))
        f, t = period_bounds(kind, t + D1)
    return out


def jstr(s):
    o = ['"']
    for ch in s:
        c = ord(ch)
        if ch == '"':
            o.append('\\"')
        elif ch == "\\":
            o.append("\\\\")
        elif ch == "\n":
            o.append("\\n")
        elif ch == "\r":
            o.append("\\r")
        elif ch == "\t":
            o.append("\\t")
        elif c < 0x20:
            o.append("\\u%04x" % c)
        else:
            o.append(ch)
    o.append('"')
    return "".join(o)


def jval(v):
    if v is None:
        return "null"
    if v is True:
        return "true"
    if v is False:
        return "false"
    if isinstance(v, int):
        return str(v)
    if isinstance(v, str):
        return jstr(v)
    if isinstance(v, list):
        return "[" + ",".join(jval(x) for x in v) + "]"
    if isinstance(v, dict):
        return "{" + ",".join(jstr(k) + ":" + jval(x) for k, x in v.items()) + "}"
    raise TypeError(v)


class R13:
    def __init__(self, data):
        self.g = O.Gen(data)
        self.persons = {p["id"]: p for p in data["persons"]}

    def deceased(self, o):
        if O.B4:                                   # 04 B4 muc 3: da_mat = gio + tien thuong
            return o.cls in ("gio", "tien_thuong")
        if o.cls in ("su_kien_duong_lap", "su_kien_am_lap", "su_kien_khong_lap"):
            ids = o.person.get("person_ids") or []
            return bool(ids) and all(self.persons.get(i) is not None and self.persons[i].get("death") is not None
                                     for i in ids)
        return o.person.get("death") is not None

    def item(self, o):
        if o.cls.startswith("su_kien"):
            pids = list(o.person.get("person_ids") or [])
            name = (o.person.get("tieu_de") or "").strip()
            if not name:
                ty = self.g.types.get(o.person["type_id"])
                name = ty["ten"] if ty else "Sự kiện"
        else:
            p = o.person
            pids = [p["id"]]
            dx = (p.get("danh_xung") or "").strip()
            name = (dx + " " + p["ho_ten"]) if dx else p["ho_ten"]
        if o.cls == "gio":
            label = o.label
        elif o.cls == "tien_thuong":
            label = O.G.label_of(o.n, (o.person.get("memorial") or {}).get("gio_het_nam", 2))
        else:
            label = None
        y, m, d, leap = o.lunar
        return {"date": o.date.isoformat(), "lunar": {"y": y, "m": m, "d": d, "leap": leap}, "class": o.cls,
                "entity_id": o.entity, "person_ids": pids, "name": name, "key": o.key, "n": o.n, "label": label}

    def L(self, f, t):
        return self.g.all_occ(f, t)

    def lines(self, F, T):
        allocc = self.L(period_bounds("tuan", F)[0], period_bounds("tuan", T)[1])
        for kind in KINDS:
            for anchor, f, t in periods(kind, F, T):
                sel = [o for o in allocc if f <= o.date <= t]
                for flt in FILTERS:
                    if flt == "tat_ca":
                        items = sel
                    elif flt == "da_mat":
                        items = [o for o in sel if self.deceased(o)]
                    else:
                        items = [o for o in sel if not self.deceased(o)]
                    yield jval({"kind": kind, "anchor": anchor.isoformat(), "from": f.isoformat(),
                                "to": t.isoformat(), "filter": flt, "items": [self.item(o) for o in items]})


def check(ds, path, F, T):
    """(c) nhat quan, (d) ngay dau/cuoi — tren tep cua oracle (hoac tep bat ky cung dinh dang)."""
    r = R13(O.load(ds))
    rows = [json.loads(l) for l in open(path, encoding="utf-8")]
    idx = {(o["kind"], o["from"], o["filter"]): o for o in rows}
    res = {"rows": len(rows), "periods": {k: len(periods(k, F, T)) for k in KINDS}}
    K = lambda it: json.dumps(it, ensure_ascii=False, sort_keys=True)
    viol = []
    # khong trung muc trong mot dong
    for o in rows:
        ks = [K(i) for i in o["items"]]
        if len(ks) != len(set(ks)):
            viol.append(["trung_muc", o["kind"], o["from"], o["filter"]])
    # da_mat U dang_song = tat_ca, roi nhau, giu thu tu
    for (kind, f, flt), o in idx.items():
        if flt != "tat_ca":
            continue
        a = [K(i) for i in o["items"]]
        dm = [K(i) for i in idx[(kind, f, "da_mat")]["items"]]
        ds_ = [K(i) for i in idx[(kind, f, "dang_song")]["items"]]
        if set(dm) & set(ds_):
            viol.append(["giao_nhau", kind, f])
        if sorted(dm + ds_) != sorted(a):
            viol.append(["hop_khac_tat_ca", kind, f])
        if [x for x in a if x in set(dm)] != dm or [x for x in a if x in set(ds_)] != ds_:
            viol.append(["thu_tu_loc", kind, f])
    # hop cac ky con (cat theo bien) = ky cha, theo tung bo loc, giu thu tu
    def items_in(kind, flt, lo, hi):
        out = []
        for anchor, f, t in periods(kind, F, T):
            if t < lo or f > hi:
                continue
            for it in idx[(kind, f.isoformat(), flt)]["items"]:
                if lo.isoformat() <= it["date"] <= hi.isoformat():
                    out.append(K(it))
        return out
    pairs = [("tuan", "thang"), ("thang", "quy"), ("quy", "nam")]
    cnt_c = 0
    for child, parent in pairs:
        for anchor, f, t in periods(parent, F, T):
            for flt in FILTERS:
                want = [K(i) for i in idx[(parent, f.isoformat(), flt)]["items"]]
                got = items_in(child, flt, f, t)
                cnt_c += 1
                if got != want:
                    viol.append(["hop_" + child + "_khac_" + parent, f.isoformat(), flt, len(got), len(want)])
    res["c_comparisons"] = cnt_c
    res["c_violations"] = viol
    # (d): moi ky, muc ngay `from` = L(from, from) va muc ngay `to` = L(to, to) (loc tat_ca)
    dv = []
    nonempty_first = nonempty_last = 0
    for kind in KINDS:
        for anchor, f, t in periods(kind, F, T):
            items = idx[(kind, f.isoformat(), "tat_ca")]["items"]
            for day, tag in ((f, "dau"), (t, "cuoi")):
                want = [K(r.item(o)) for o in r.L(day, day)]
                got = [K(i) for i in items if i["date"] == day.isoformat()]
                if got != want:
                    dv.append([kind, f.isoformat(), tag, len(got), len(want)])
                if want:
                    if tag == "dau":
                        nonempty_first += 1
                    else:
                        nonempty_last += 1
            # ky tu ngay: moi ngay trong ky (khong chi hai dau)
            per_day = []
            d = f
            while d <= t:
                per_day.extend(K(r.item(o)) for o in r.L(d, d))
                d += D1
            if sorted(per_day) != sorted(K(i) for i in items):
                dv.append([kind, f.isoformat(), "tung_ngay", len(per_day), len(items)])
    res["d_violations"] = dv
    res["d_periods_with_items_on_first_day"] = nonempty_first
    res["d_periods_with_items_on_last_day"] = nonempty_last
    return res


def main():
    a = sys.argv[1:]
    if a[0] == "--check":
        F = dt.date.fromisoformat(a[3]) if len(a) > 3 else dt.date(2026, 1, 1)
        T = dt.date.fromisoformat(a[4]) if len(a) > 4 else dt.date(2036, 12, 31)
        print(json.dumps(check(a[1], a[2], F, T), ensure_ascii=False, indent=1))
        return
    F = dt.date.fromisoformat(a[1]) if len(a) > 1 else dt.date(2026, 1, 1)
    T = dt.date.fromisoformat(a[2]) if len(a) > 2 else dt.date(2036, 12, 31)
    r = R13(O.load(a[0]))
    w = sys.stdout.write
    for l in r.lines(F, T):
        w(l + "\n")


if __name__ == "__main__":
    main()
