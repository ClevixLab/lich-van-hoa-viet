#!/usr/bin/env python3
"""R12 doc lap — mo phong xoay vong 64 thong bao cho cua iOS (02 §7.4 da sua theo Bo sung B2 muc 1:
sentinel.at := keep[CAP-2].at, cung phut voi cai thu 63, xep sau no).

Viet tu 02 §7.1, §7.4, Phu luc A.5 va 04 R12 (khong doc ma loi).
  * Ban ghi thu i (i = 0..size-1) = ban ghi (i mod 40) cua bo R3 (27 ho so roi 13 su kien, theo thu tu trong tep);
    moi id trong ban ghi (ho so/su kien va id quy tac nhac) doi 8 ky tu dau thanh i dang hex 8 chu so.
  * Kich ban p1, p0.1, p0; 2026-01-01..2028-12-31; mo ung dung luc 09:00; ngay dau luon mo; p0.1: mulberry32 hat
    20261106, mot so moi ngay (ke ca ngay dau), mo khi so < 0,1.
  * Moi lan mo: (1) giao moi thong bao dang cho co at <= luc mo; (2) reschedule(now): desired = moi loi nhac
    (§7.1, quy tac that cua ban ghi) co now < at <= now + 400 ngay, sap theo (at, id); > 64 thi giu 63 som nhat +
    sentinel tai at cua cai thu 63; (3) kiem bat bien. Cuoi mo phong giao not den 2028-12-31T23:59.
Bat bien (kiem truc tiep tren tap cho sau moi buoc):
  I1  so yeu cau cho <= 64
  I2  neu len(desired) > 64: 63 cai som nhat cua desired deu dang cho va sentinel o dung at cua cai thu 63;
      neu len(desired) <= 64: moi cai trong desired dang cho, khong co sentinel
  I3  moi loi nhac trong desired bi bo (khong dang cho) co at >= sentinel.at (va co sentinel)
Ra (CSV): size,scenario,opens,max_pending,delivered,sentinel_delivered,violations,missed,missed_without_sentinel
  delivered            so loi nhac (khong ke sentinel) duoc giao trong mo phong
  missed               so loi nhac co at trong (2026-01-01T09:00, 2028-12-31T23:59] khong duoc giao
  missed_without_sentinel  trong so do, nhung cai khong co sentinel nao duoc giao trong (lan mo truoc no, at]
  violations           tong so vi pham I1 + I2 + I3 (dem theo tung loi nhac vi pham)
Cach dung: b2_r12.py DATASET [--steps] > tep.csv
"""
import bisect, copy, datetime as dt, os, sys
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import b2_occ as O  # noqa: E402

CAP = 64
START, END = dt.date(2026, 1, 1), dt.date(2028, 12, 31)
END_T = dt.datetime(2028, 12, 31, 23, 59)
SENT = "lvhv:sentinel"
SKIP_FIRST_DRAW = "--rut-tu-ngay-2" in sys.argv


def mulberry32_js(seed):
    """Ban dung theo mulberry32 JavaScript chuan (Math.imul, >>> 0) / 2^32."""
    s = [seed & 0xFFFFFFFF]

    def imul(x, y):
        return (x * y) & 0xFFFFFFFF

    def nxt():
        s[0] = (s[0] + 0x6D2B79F5) & 0xFFFFFFFF
        t = s[0]
        t = imul(t ^ (t >> 15), t | 1)
        t = (t ^ ((t + imul(t ^ (t >> 7), t | 61)) & 0xFFFFFFFF)) & 0xFFFFFFFF
        return ((t ^ (t >> 14)) & 0xFFFFFFFF) / 4294967296.0
    return nxt


def reid(obj, i):
    """Doi 8 ky tu dau cua moi chuoi UUID (khoa 'id') trong ban ghi."""
    o = copy.deepcopy(obj)
    pre = "%08x" % i

    def fix(x):
        if isinstance(x, dict):
            for k, v in x.items():
                if k == "id" and isinstance(v, str) and len(v) == 36:
                    x[k] = pre + v[8:]
                else:
                    fix(v)
        elif isinstance(x, list):
            for v in x:
                fix(v)
    fix(o)
    return o


def records(data, size):
    base = [("p", p) for p in data["persons"]] + [("e", e) for e in data["events"]]
    assert len(base) == 40, len(base)
    return [(k, reid(r, i)) for i, (k, r) in enumerate(base[j % 40] for j in range(size))]


def all_reminders(g, recs, now0):
    """Moi loi nhac (at > now0) cua moi ban ghi, voi occ trong [2026-01-01, 2031-12-31]."""
    d0, d1 = dt.date(2025, 12, 1), dt.date(2031, 12, 31)
    out = {}
    s = g.settings.get("reminders_enabled", True)
    for k, r in recs:
        occs = g.person_occ(r, d0, d1) if k == "p" else g.event_occ(r, d0, d1)
        for o in occs:
            if o.cls == "tien_thuong":
                continue
            for t, nid, typ, _ in O.reminders_for(o.rule, o, now0, s):
                out[nid] = t          # id trung (neu co) -> mot yeu cau
    lst = sorted((t, nid) for nid, t in out.items())
    return lst


def simulate(rem, scenario, steps=None, size=None):
    ats = [t for t, _ in rem]
    rng = mulberry32_js(20261106)
    pending = {}            # id -> at
    delivered = sent_deliv = viol = opens = maxp = 0
    delivered_ids = set()
    sentinel_fire_times = []
    open_times = []
    day = START
    while day <= END:
        # Bien the SAU DOI CHIEU (co --rut-tu-ngay-2): khong rut so o ngay dau (ngay dau luon mo).
        u = rng() if (day != START or not SKIP_FIRST_DRAW) else 1.0
        is_open = day == START or (scenario == "p1") or (scenario == "p0.1" and u < 0.1)
        if is_open:
            now = dt.datetime(day.year, day.month, day.day, 9, 0)
            # (1) giao
            for nid, t in list(pending.items()):
                if t <= now:
                    if nid == SENT:
                        sent_deliv += 1
                        sentinel_fire_times.append(t)
                    else:
                        delivered += 1
                        delivered_ids.add(nid)
                    del pending[nid]
            # (2) reschedule
            hi = now + dt.timedelta(days=400)
            i = bisect.bisect_right(ats, now)
            j = bisect.bisect_right(ats, hi)
            desired = rem[i:j]
            if len(desired) > CAP:
                keep = desired[:CAP - 1]
                sat = keep[CAP - 2][0]
                keep = keep + [(sat, SENT)]
            else:
                keep = desired
                sat = None
            pending = {nid: t for t, nid in keep}
            opens += 1
            open_times.append(now)
            maxp = max(maxp, len(pending))
            # (3) bat bien
            v = 0
            if len(pending) > CAP:
                v += 1
            dropped = 0
            if len(desired) > CAP:
                for t, nid in desired[:CAP - 1]:
                    if pending.get(nid) != t:
                        v += 1
                if pending.get(SENT) != desired[CAP - 2][0]:
                    v += 1
                for t, nid in desired[CAP - 1:]:
                    if nid not in pending:
                        dropped += 1
                        if t < pending.get(SENT, dt.datetime.max):
                            v += 1
            else:
                for t, nid in desired:
                    if pending.get(nid) != t:
                        v += 1
                if SENT in pending:
                    v += 1
            viol += v
            if steps is not None:
                steps.append((size, scenario, day.isoformat(), len(desired), len(pending),
                              "" if sat is None else O.fmt_dt(sat), dropped, v))
        day += dt.timedelta(days=1)
    for nid, t in list(pending.items()):
        if t <= END_T:
            if nid == SENT:
                sent_deliv += 1
                sentinel_fire_times.append(t)
            else:
                delivered += 1
                delivered_ids.add(nid)
    first = open_times[0]
    missed = mws = 0
    for t, nid in rem:
        if first < t <= END_T and nid not in delivered_ids:
            missed += 1
            k = bisect.bisect_left(open_times, t) - 1          # lan mo cuoi truoc t
            lo = open_times[k]
            if not any(lo < s <= t for s in sentinel_fire_times):
                mws += 1
    return dict(opens=opens, max_pending=maxp, delivered=delivered, sentinel_delivered=sent_deliv,
                violations=viol, missed=missed, missed_without_sentinel=mws)


def main():
    ds = sys.argv[1]
    want_steps = "--steps" in sys.argv
    data = O.load(ds)
    g = O.Gen(data)
    w = sys.stdout.write
    w("size,scenario,opens,max_pending,delivered,sentinel_delivered,violations,missed,missed_without_sentinel\n")
    steps = [] if want_steps else None
    for size in (10, 50, 200):
        rem = all_reminders(g, records(data, size), dt.datetime(2026, 1, 1, 0, 0))
        for sc in ("p1", "p0.1", "p0"):
            r = simulate(rem, sc, steps, size)
            w("%d,%s,%d,%d,%d,%d,%d,%d,%d\n" % (size, sc, r["opens"], r["max_pending"], r["delivered"],
                                              r["sentinel_delivered"], r["violations"], r["missed"],
                                              r["missed_without_sentinel"]))
    if want_steps:
        w("size,scenario,day,future,pending,sentinel,dropped,violations\n")
        for s in steps:
            w("%d,%s,%s,%d,%d,%s,%d,%d\n" % s)


if __name__ == "__main__":
    main()
