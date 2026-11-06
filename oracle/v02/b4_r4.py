#!/usr/bin/env python3
"""R4(b) ngu nghia (04 R4; 02 §8.1, §8.2 UID): tap (UID, DTSTART) ky vong cua tep .ics.

  b4_r4.py expect DATASET TODAY            -> moi dong "UID<TAB>YYYYMMDD", sap
  b4_r4.py parse  TEP.ics                  -> cung dinh dang, doc tu tep .ics (bo gap dong RFC 5545 §3.1)

Ky vong: moi lan xuat hien (bo sinh b2_occ, ke ca Bo sung B4 khi LVHV_B4=1) co ngay trong [TODAY, TODAY + 10 nam)
cua muc co quy tac nhac bat (reminders_enabled va rule.enabled); UID = <id thuc the>-<loai>-<khoa nam>@lich.clevix.vn,
loai: gio, tt (tien thuong, ngay D - 1, chi khi rule.tien_thuong), sn, sna, ev; khoa nam = occ.key (`once` cho su
kien mot lan)."""
import datetime as dt, os, sys
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import b2_occ as O  # noqa: E402

TYPE = {"gio": "gio", "tien_thuong": "tt", "sinh_nhat": "sn", "sinh_nhat_am": "sna",
        "su_kien_duong_lap": "ev", "su_kien_am_lap": "ev", "su_kien_khong_lap": "ev"}


def expect(ds, today):
    g = O.Gen(O.load(ds))
    end = dt.date(today.year + 10, today.month, today.day) - dt.timedelta(days=1)
    if not g.settings.get("reminders_enabled", True):
        return []
    out = []
    for o in g.all_occ(today, end):
        if o.rule is None or not o.rule.get("enabled", True):
            continue
        out.append("%s-%s-%s@lich.clevix.vn\t%s" % (o.entity, TYPE[o.cls], o.key, o.date.strftime("%Y%m%d")))
    return sorted(out)


def parse(path):
    raw = open(path, encoding="utf-8", newline="").read().split("\r\n")
    lines = []
    for l in raw:
        if l.startswith(" ") and lines:
            lines[-1] += l[1:]
        else:
            lines.append(l)
    out, uid, start, inev = [], None, None, False
    for l in lines:
        if l == "BEGIN:VEVENT":
            inev, uid, start = True, None, None
        elif l == "END:VEVENT":
            out.append("%s\t%s" % (uid, start))
            inev = False
        elif inev and l.startswith("UID:"):
            uid = l[4:]
        elif inev and l.startswith("DTSTART"):
            start = l.split(":", 1)[1]
    return sorted(out)


def main():
    if sys.argv[1] == "expect":
        rows = expect(sys.argv[2], dt.date.fromisoformat(sys.argv[3]))
    else:
        rows = parse(sys.argv[2])
    for r in rows:
        print(r)


if __name__ == "__main__":
    main()
