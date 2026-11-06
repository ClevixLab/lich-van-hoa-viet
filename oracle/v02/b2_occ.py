#!/usr/bin/env python3
"""Bo sinh lan xuat hien doc lap (Bo sung B2) — dung chung cho R3, R11, R12, R13.

Viet tu 01 §3, 02 §2, §3, §5, §6, §7.1, §7.2 va 04 (khong doc ma loi). Doi lich: giolib.Calendar("de441")
(phuong phap Jpl). Bo du lieu: tep JSON sao luu (01 §5), vi du bo R3 `memorial/testdata/r3-dataset.json`
(chi doc nhu DU LIEU). Moi truong dan xuat (death.lunar, birth.solar, birth.lunar) duoc TINH LAI tu `input`
bang lich DE441, khong lay tu tep.

Lan xuat hien (Occ):
  date     ngay duong (datetime.date)
  cls      gio | tien_thuong | sinh_nhat | sinh_nhat_am | su_kien_duong_lap | su_kien_am_lap | su_kien_khong_lap
  entity   id ho so (Person) hoac su kien (Event)
  key      "Y" (nam am: gio, tien_thuong, sinh_nhat_am, su_kien_am_lap; nam duong: sinh_nhat, su_kien_duong_lap)
           hoac "once"
  n        so lan (gio: Y - nam am mat; sinh nhat: Y - nam goc; su kien lap: Y - nam goc) hoac None
  label    gio | gio_dau | gio_het (gio, va tien_thuong: nhan cua gio hom sau), "" (con lai)
           [SUA SAU DOI CHIEU R11, 2026-11-07: truoc do tien_thuong ghi label "tien_thuong"; A.3 khong dinh nghia,
            02 §7.6 ghi ro "nhan cua gio hom sau" -> theo §7.6]
  lunar    (y, m, d, leap) cua ngay duong `date` theo lich DE441
  name     ten de sap (ho_ten cua Person; Event: tieu_de, rong thi ten loai)
  rule     ReminderRule (dict) hoac None
  status   "deceased" | "living" (Person); Event: None
Lua chon cua ben kiem dinh o cho dac ta chua noi (ghi trong bao cao B2):
  * Nguoi da mat chi co gio (+ tien thuong neu rule.gio.tien_thuong); nguoi song co sinh_nhat khi biet ngay duong
    du (nhap duong `day`, hoac nhap am `day` da doi), sinh_nhat_am khi `sinh_nhat_am = true` va biet ngay am du.
  * Sinh nhat duong: Y > nam sinh (02 §6). Sinh nhat am: Y > nam am sinh (anniversary §2.2).
  * Su kien duong lap: Y >= nam goc (02 §6 dung nguyen `if Y < Y0: None`); 29/02 -> 28/02 (B2 muc 2).
    Su kien am lap: anniversary §2.2 (Y > nam goc; month_day: moi Y, n = None).
  * Su kien khong lap: mot lan, ngay da nhap (am: L2S kem co nhuan).
  * Tien thuong: lan xuat hien rieng ngay D - 1 khi rule gio co tien_thuong = true (ke ca rule.enabled = false).
"""
import datetime as dt
import json
import os
import sys
import unicodedata

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import giolib as G  # noqa: E402

RANK = {"gio": 0, "tien_thuong": 1, "sinh_nhat": 2, "sinh_nhat_am": 2,
        "su_kien_duong_lap": 3, "su_kien_am_lap": 3, "su_kien_khong_lap": 3}


class Occ:
    __slots__ = ("date", "cls", "entity", "key", "n", "label", "lunar", "name", "rule", "status", "person")

    def __init__(self, **kw):
        for k in self.__slots__:
            setattr(self, k, kw.get(k))

    def sort_key(self):
        # 02 §7.2: ngay, gio -> tien thuong -> sinh nhat -> su kien, ho_ten (diem ma sau NFC);
        # phan xu them (02 Phu luc A.3): lop, id thuc the, khoa nam.
        return (self.date, RANK[self.cls], unicodedata.normalize("NFC", self.name or ""), self.cls,
                self.entity, self.key)


def load(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)


class Gen:
    def __init__(self, data, cal=None):
        self.cal = cal or G.Calendar("de441")
        self.data = data
        self.types = {t["id"]: t for t in data.get("event_types", [])}
        self.settings = data.get("settings", {})

    # ------------------------------------------------------------------ dan xuat
    def _lunar_of(self, d):
        r = self.cal.s2l(d.year, d.month, d.day)
        return tuple(r)

    def death_lunar(self, p):
        return G.save_death(self.cal, p["death"]["input"])["lunar"]

    def birth(self, p):
        b = p.get("birth")
        if not b:
            return None
        return G.birth_derive(self.cal, b["input"])

    # ------------------------------------------------------------------ lan xuat hien
    def person_occ(self, p, d0, d1):
        """Moi lan xuat hien cua ho so p co ngay duong trong [d0, d1]."""
        out = []
        pid = p["id"]
        name = p["ho_ten"]
        if p["kind"] == "deceased":
            dl = self.death_lunar(p)
            het = (p.get("memorial") or {}).get("gio_het_nam", 2)
            rule = p["reminders"].get("gio")
            tt = bool(rule and rule.get("tien_thuong"))
            # mo rong 1 ngay de bat tien thuong (D-1) cua gio ngay d1+1
            for a in G.anniversaries_in(self.cal, dl, d0, d1 + dt.timedelta(days=1), het):
                key = str(a["lunar"][0])
                if d0 <= a["solar"] <= d1:
                    out.append(Occ(date=a["solar"], cls="gio", entity=pid, key=key, n=a["n"], label=a["label"],
                                   name=name, rule=rule, status="deceased", person=p))
                t = a["solar"] - dt.timedelta(days=1)
                if tt and d0 <= t <= d1:
                    out.append(Occ(date=t, cls="tien_thuong", entity=pid, key=key, n=a["n"], label=a["label"],
                                   name=name, rule=rule, status="deceased", person=p))
            return self._fill(out)
        bd = self.birth(p)
        if bd is None:
            return out
        if bd["solar"] is not None:
            s = bd["solar"]
            for Y in range(max(d0.year, s["y"] + 1), d1.year + 1):
                o = G.solar_occurrence((s["y"], s["m"], s["d"]), Y, p.get("feb29_rule", "feb28"))
                if o is not None and d0 <= o <= d1:
                    out.append(Occ(date=o, cls="sinh_nhat", entity=pid, key=str(Y), n=Y - s["y"], label="",
                                   name=name, rule=p["reminders"].get("sinh_nhat"), status="living", person=p))
        if p.get("sinh_nhat_am") and bd["lunar"] is not None:
            bl = {"year": bd["lunar"]["year"], "month": bd["lunar"]["month"], "day": bd["lunar"]["day"],
                  "leap": bd["lunar"]["leap"]}
            for a in G.anniversaries_in(self.cal, bl, d0, d1):
                out.append(Occ(date=a["solar"], cls="sinh_nhat_am", entity=pid, key=str(a["lunar"][0]), n=a["n"],
                               label="", name=name, rule=p["reminders"].get("sinh_nhat_am"), status="living",
                               person=p))
        return self._fill(out)

    def event_name(self, e):
        t = (e.get("tieu_de") or "").strip()
        if t:
            return t
        ty = self.types.get(e["type_id"])
        return ty["ten"] if ty else ""

    def event_occ(self, e, d0, d1):
        out = []
        date = e["date"]
        rule = e.get("reminder")
        name = self.event_name(e)
        eid = e["id"]
        if not e.get("recurring"):
            if date["calendar"] == "solar":
                D = dt.date(date["year"], date["month"], date["day"])
            else:
                D = self.cal.l2s(date["year"], date["month"], date["day"], date["leap"])
            if D is not None and d0 <= D <= d1:
                out.append(Occ(date=D, cls="su_kien_khong_lap", entity=eid, key="once", n=None, label="",
                               name=name, rule=rule, person=e))
            return self._fill(out)
        if date["calendar"] == "solar":
            Y0 = date["year"]
            for Y in range(max(d0.year, Y0), d1.year + 1):
                o = G.solar_occurrence((Y0, date["month"], date["day"]), Y, "feb28")
                if o is not None and d0 <= o <= d1:
                    out.append(Occ(date=o, cls="su_kien_duong_lap", entity=eid, key=str(Y), n=Y - Y0, label="",
                                   name=name, rule=rule, person=e))
            return self._fill(out)
        dl = {"year": date["year"], "month": date["month"], "day": date["day"], "leap": date["leap"]}
        for a in G.anniversaries_in(self.cal, dl, d0, d1):
            out.append(Occ(date=a["solar"], cls="su_kien_am_lap", entity=eid, key=str(a["lunar"][0]), n=a["n"],
                           label="", name=name, rule=rule, person=e))
        return self._fill(out)

    def _fill(self, out):
        for o in out:
            o.lunar = self._lunar_of(o.date)
        return out

    def all_occ(self, d0, d1, persons=None, events=None):
        persons = self.data["persons"] if persons is None else persons
        events = self.data["events"] if events is None else events
        out = []
        for p in persons:
            out.extend(self.person_occ(p, d0, d1))
        for e in events:
            out.extend(self.event_occ(e, d0, d1))
        out.sort(key=Occ.sort_key)
        return out


# ---------------------------------------------------------------------- 02 §7.1
def hhmm(s):
    h, m = s.split(":")
    return int(h), int(m)


def reminders_for(rule, occ, now, reminders_enabled=True, offsets=None, tien_thuong=None, time_local=None):
    """Tra ve [(at: datetime naive gio thiet bi, id, type, occ)]. occ.cls khong duoc la tien_thuong."""
    if not reminders_enabled or rule is None or not rule.get("enabled", True):
        return []
    offs = rule["offsets_days"] if offsets is None else offsets
    tt = rule.get("tien_thuong", False) if tien_thuong is None else tien_thuong
    tl = rule["time_local"] if time_local is None else time_local
    h, m = hhmm(tl)
    out = []
    for k in offs:
        day = occ.date - dt.timedelta(days=k)
        t = dt.datetime(day.year, day.month, day.day, h, m)
        if t > now:
            out.append((t, "lvhv:%s:%s:%d" % (rule["id"], occ.key, k), "k=%d" % k, occ))
    if tt and occ.cls == "gio":
        day = occ.date - dt.timedelta(days=1)
        t = dt.datetime(day.year, day.month, day.day, h, m)
        if t > now:
            out.append((t, "lvhv:%s:%s:tt" % (rule["id"], occ.key), "tt", occ))
    return out


def fmt_dt(t):
    return "%04d-%02d-%02dT%02d:%02d" % (t.year, t.month, t.day, t.hour, t.minute)
