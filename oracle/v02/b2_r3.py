#!/usr/bin/env python3
"""R3 doc lap (04 R3, 02 §7.1, Phu luc A.2).

  b2_r3.py configs            -> cfg,presets,custom,offsets,tien_thuong,time_local,class (140 544 dong)
  b2_r3.py DATASET [FROM TO]  -> cfg,id,at_local,type,occ_date: voi moi cau hinh, moi lan xuat hien cua lop do
      co ngay trong [FROM, TO] (mac dinh 2026-01-01..2036-12-31), quy tac cua ban ghi voi offsets_days /
      tien_thuong / time_local thay bang cau hinh (id quy tac giu nguyen; ban ghi khong co quy tac -> khong nhac;
      enabled giu theo ban ghi), reminders_enabled = true, now = 2026-01-01T00:00. Sap theo (cfg, at, id).
Thu tu long cau hinh: mask nut co san 0..63 (bit i <-> [0,1,3,5,10,15][i]) -> moc tu dat (trong, 1..60) ->
tien_thuong (0,1) -> time_local (00:00, 07:00, 23:59) -> lop (6 lop)."""
import datetime as dt, os, sys
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import b2_occ as O  # noqa: E402

PRESET = [0, 1, 3, 5, 10, 15]
TIMES = ["00:00", "07:00", "23:59"]
CLASSES = ["gio", "sinh_nhat", "sinh_nhat_am", "su_kien_duong_lap", "su_kien_am_lap", "su_kien_khong_lap"]


def configs():
    i = 0
    for mask in range(64):
        presets = [PRESET[b] for b in range(6) if mask >> b & 1]
        for custom in [None] + list(range(1, 61)):
            offs = sorted(set(presets) | ({custom} if custom is not None else set()))
            for tt in (0, 1):
                for tl in TIMES:
                    for cls in CLASSES:
                        yield i, presets, custom, offs, tt, tl, cls
                        i += 1


def main():
    w = sys.stdout.write
    if sys.argv[1] == "configs":
        w("cfg,presets,custom,offsets,tien_thuong,time_local,class\n")
        for i, pr, cu, offs, tt, tl, cls in configs():
            w("%d,%s,%s,%s,%d,%s,%s\n" % (i, " ".join(map(str, pr)), "" if cu is None else cu,
                                         " ".join(map(str, offs)), tt, tl, cls))
        return
    ds = sys.argv[1]
    f = dt.date.fromisoformat(sys.argv[2]) if len(sys.argv) > 2 else dt.date(2026, 1, 1)
    t = dt.date.fromisoformat(sys.argv[3]) if len(sys.argv) > 3 else dt.date(2036, 12, 31)
    g = O.Gen(O.load(ds))
    now = dt.datetime(2026, 1, 1, 0, 0)
    by = {c: [] for c in CLASSES}
    for o in g.all_occ(f, t):
        if o.cls in by and o.rule is not None:
            by[o.cls].append(o)
    w("cfg,id,at_local,type,occ_date\n")
    for i, pr, cu, offs, tt, tl, cls in configs():
        rows = []
        for o in by[cls]:
            for at, nid, typ, _ in O.reminders_for(o.rule, o, now, True, offs, bool(tt) and cls == "gio", tl):
                rows.append((at, nid, typ, o.date))
        rows.sort(key=lambda r: (r[0], r[1]))
        for at, nid, typ, d in rows:
            w("%d,%s,%s,%s,%s\n" % (i, nid, O.fmt_dt(at), typ, d.isoformat()))


if __name__ == "__main__":
    main()
