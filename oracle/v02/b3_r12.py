#!/usr/bin/env python3
"""R12 doc lap theo Bo sung B3 (04 B3 muc 1; 02 §7.4 va Phu luc A.5 tai 181c039). Ban B2 (b2_r12.py) giu nguyen.

Khac B2:
  * reschedule: hi = now + 400 ngay; desired = loi nhac co now < at <= hi, sap (at, id); beyond = co loi nhac at > hi.
    Neu len(desired) > 64 hoac beyond: k = min(len, 63), giu k som nhat + sentinel tai at cua cai thu k (k = 0 -> hi).
  * Tap loi nhac: lan xuat hien co ngay trong [2026-01-01, 2028-12-31 + 900 ngay] va moi su kien khong lap toi
    3000-12-31.
  * Giao: moi ngay mo phong luc 09:00 (truoc khi mo); I1 kiem moi ngay sau khi giao; I2-I4 sau moi lan mo.
  * Bat bien (dem rieng i1..i4, violations = tong):
      I1 so cho <= 64
      I2 neu len(desired) >= 64: 63 som nhat dang cho; sentinel tai at63 neu con loi nhac khong dat (len > 64 hoac
         beyond), khong co sentinel neu khong con
      I3 moi loi nhac trong cua so khong dang cho co at >= sentinel.at (khong co sentinel -> vi pham)
      I4 neu co loi nhac tuong lai (trong hay ngoai cua so) khong dang cho: co sentinel voi at <= loi nhac som nhat
         trong so do
  * Cot: delivered GOM sentinel; sentinel_delivered = phan sentinel; missed = loi nhac co at trong
    (09:00 ngay dau, 09:00 ngay cuoi] khong duoc giao; missed_without_sentinel = trong so do, khong co sentinel nao
    duoc giao voi at <= loi nhac do.
  * p0.1: ngay dau khong rut so; tu ngay 2 rut mot so u32 (mulberry32, hat 20261106), mo khi u32 * 10 < 2^32.
Cach dung: b3_r12.py DATASET [--steps] > tep.csv
"""
import bisect, datetime as dt, os, sys
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import b2_occ as O  # noqa: E402
from b2_r12 import records  # noqa: E402

CAP = 64
START, END = dt.date(2026, 1, 1), dt.date(2028, 12, 31)
SENT = "lvhv:sentinel"
OCC_END = END + dt.timedelta(days=900)


def mulberry32_u32(seed):
    s = [seed & 0xFFFFFFFF]

    def imul(x, y):
        return (x * y) & 0xFFFFFFFF

    def nxt():
        s[0] = (s[0] + 0x6D2B79F5) & 0xFFFFFFFF
        t = s[0]
        t = imul(t ^ (t >> 15), t | 1)
        t = (t ^ ((t + imul(t ^ (t >> 7), t | 61)) & 0xFFFFFFFF)) & 0xFFFFFFFF
        return (t ^ (t >> 14)) & 0xFFFFFFFF
    return nxt


def all_reminders(g, recs, now0):
    out = {}
    s = g.settings.get("reminders_enabled", True)
    far = dt.date(3000, 12, 31)
    for k, r in recs:
        if k == "p":
            occs = g.person_occ(r, START, OCC_END)
        else:
            occs = g.event_occ(r, START, far if not r.get("recurring") else OCC_END)
        for o in occs:
            if o.cls == "tien_thuong":
                continue
            for t, nid, typ, _ in O.reminders_for(o.rule, o, now0, s):
                out[nid] = t
    return sorted((t, nid) for nid, t in out.items())


def simulate(rem, scenario, steps=None, size=None):
    ats = [t for t, _ in rem]
    nxt = mulberry32_u32(20261106)
    pending = {}
    delivered = sent_deliv = opens = maxp = 0
    iv = [0, 0, 0, 0]
    delivered_ids = set()
    sent_times = []
    day = START
    first = dt.datetime(2026, 1, 1, 9, 0)
    last = dt.datetime(END.year, END.month, END.day, 9, 0)
    while day <= END:
        now = dt.datetime(day.year, day.month, day.day, 9, 0)
        for nid, t in list(pending.items()):
            if t <= now:
                delivered += 1
                if nid == SENT:
                    sent_deliv += 1
                    sent_times.append(t)
                else:
                    delivered_ids.add(nid)
                del pending[nid]
        if len(pending) > CAP:
            iv[0] += 1
        if day == START or scenario == "p1":
            is_open = True
        elif scenario == "p0.1":
            is_open = nxt() * 10 < 2 ** 32
        else:
            is_open = False
        if is_open:
            hi = now + dt.timedelta(days=400)
            i = bisect.bisect_right(ats, now)
            j = bisect.bisect_right(ats, hi)
            desired = rem[i:j]
            beyond = j < len(rem)
            if len(desired) > CAP or beyond:
                k = min(len(desired), CAP - 1)
                keep = desired[:k]
                if k > 0:
                    sat = keep[k - 1][0]
                elif O.B4:   # 02 ghi chu MA3-14-06 (doc tai 8faa3cd): 09:00 cua ngay hom nay + 399
                    d399 = day + dt.timedelta(days=399)
                    sat = dt.datetime(d399.year, d399.month, d399.day, 9, 0)
                else:
                    sat = hi
                keep = keep + [(sat, SENT)]
            else:
                keep = desired
                sat = None
            pending = {nid: t for t, nid in keep}
            opens += 1
            maxp = max(maxp, len(pending))
            v = [0, 0, 0, 0]
            if len(pending) > CAP:
                v[0] += 1
            unsched = len(desired) > CAP or beyond
            if len(desired) >= CAP:
                for t, nid in desired[:CAP - 1]:
                    if pending.get(nid) != t:
                        v[1] += 1
                if unsched and pending.get(SENT) != desired[CAP - 2][0]:
                    v[1] += 1
                if not unsched and SENT in pending:
                    v[1] += 1
            sp = pending.get(SENT)
            dropped = 0
            for t, nid in desired:
                if nid not in pending:
                    dropped += 1
                    if sp is None or t < sp:
                        v[2] += 1
            # I4: loi nhac tuong lai som nhat khong dang cho (trong hoac ngoai cua so)
            earliest = None
            for t, nid in rem[i:]:
                if nid not in pending:
                    earliest = t
                    break
            if earliest is not None and (sp is None or sp > earliest):
                v[3] += 1
            for x in range(4):
                iv[x] += v[x]
            if steps is not None:
                steps.append((size, scenario, day.isoformat(), len(desired), int(beyond), len(pending),
                              "" if sat is None else O.fmt_dt(sat), dropped, sum(v)))
        day += dt.timedelta(days=1)
    missed = mws = 0
    for t, nid in rem:
        if first < t <= last and nid not in delivered_ids:
            missed += 1
            if not any(s <= t for s in sent_times):
                mws += 1
    return dict(opens=opens, max_pending=maxp, delivered=delivered, sentinel_delivered=sent_deliv,
                violations=sum(iv), missed=missed, missed_without_sentinel=mws, i1=iv[0], i2=iv[1], i3=iv[2], i4=iv[3])


COLS = ["opens", "max_pending", "delivered", "sentinel_delivered", "violations", "missed", "missed_without_sentinel",
        "i1", "i2", "i3", "i4"]


def main():
    data = O.load(sys.argv[1])
    want_steps = "--steps" in sys.argv
    g = O.Gen(data)
    w = sys.stdout.write
    w("size,scenario," + ",".join(COLS) + "\n")
    steps = [] if want_steps else None
    for size in (10, 50, 200):
        rem = all_reminders(g, records(data, size), dt.datetime(2026, 1, 1, 0, 0))
        for sc in ("p1", "p0.1", "p0"):
            r = simulate(rem, sc, steps, size)
            w("%d,%s," % (size, sc) + ",".join(str(r[c]) for c in COLS) + "\n")
    if want_steps:
        w("size,scenario,day,future,beyond,pending,sentinel,dropped,violations\n")
        for s in steps:
            w("%d,%s,%s,%d,%d,%d,%s,%d,%d\n" % s)


if __name__ == "__main__":
    main()
