#!/usr/bin/env python3
"""R11 doc lap (02 §7.2): moi "hom nay" FROM..TO, danh sach lan xuat hien co ngay trong [hom nay, hom nay + 30].
Dinh dang nhu 02 Phu luc A.3: today,rank,days,date,class,entity_id,key,n,label.
Cach dung: b2_r11.py DATASET FROM TO > tep.csv"""
import bisect, datetime as dt, os, sys
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import b2_occ as O  # noqa: E402


def main():
    ds, f, t = sys.argv[1], dt.date.fromisoformat(sys.argv[2]), dt.date.fromisoformat(sys.argv[3])
    g = O.Gen(O.load(ds))
    occ = g.all_occ(f, t + dt.timedelta(days=30))
    dates = [o.date for o in occ]
    w = sys.stdout.write
    w("today,rank,days,date,class,entity_id,key,n,label\n")
    d = f
    while d <= t:
        i = bisect.bisect_left(dates, d)
        j = bisect.bisect_right(dates, d + dt.timedelta(days=30))
        for r, o in enumerate(occ[i:j]):
            w("%s,%d,%d,%s,%s,%s,%s,%s,%s\n" % (d.isoformat(), r, (o.date - d).days, o.date.isoformat(), o.cls,
                                              o.entity, o.key, "" if o.n is None else o.n, o.label or ""))
        d += dt.timedelta(days=1)


if __name__ == "__main__":
    main()
