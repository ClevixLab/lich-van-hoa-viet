# -*- coding: utf-8 -*-
"""Dung kho SQLite phien ban 1 (bo cuc cua kho dong bang 0.2-dev-before-attachments, doc nhu DU LIEU: luoc do
bang, hang meta) tu mot tep thu JSON cua kiem dinh, de thu chuyen doi tai cho bang hop den.

Cach dung: python3 -I b5_mkstore.py <kho-mau.sqlite> <tep-thu.json> <thu-muc-byte> <manifest-blobs-json> <thu-muc-ra>
Ghi <ra>/lvhv.sqlite (user_version 1) va <ra>/photos/<id>.img.
"""
import json, os, shutil, sqlite3, sys, unicodedata

tmpl, fx, bytes_dir, blobs_json, out = sys.argv[1:6]
doc = json.load(open(fx, encoding="utf-8"))
blobs = json.loads(blobs_json)
os.makedirs(os.path.join(out, "photos"), exist_ok=True)
db = os.path.join(out, "lvhv.sqlite")
shutil.copy(tmpl, db)
c = sqlite3.connect(db)
c.execute("pragma foreign_keys=off")
for t in ("event_person", "event", "event_type", "person", "photo", "settings"):
    c.execute("delete from %s" % t)
J = lambda o: json.dumps(o, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
c.execute("insert into settings values (1, ?)", (J(doc["settings"]),))
for p in doc["photos"]:
    c.execute("insert into photo values (?,?,?)", (p["id"], J(p), p["sha256"]))
    shutil.copy(os.path.join(bytes_dir, blobs[p["id"]]["sha256"] + ".bin"), os.path.join(out, "photos", p["id"] + ".img"))
for p in doc["persons"]:
    c.execute("insert into person values (?,?,?,?,?,?)", (p["id"], p["kind"], p.get("photo_id"), p["rev"], p["updated_at"], J(p)))
for t in doc["event_types"]:
    c.execute("insert into event_type values (?,?,?,?,?)",
              (t["id"], unicodedata.normalize("NFC", t["ten"]).lower(), t["rev"], t["updated_at"], J(t)))
for e in doc["events"]:
    c.execute("insert into event values (?,?,?,?,?)", (e["id"], e["type_id"], e["rev"], e["updated_at"], J(e)))
    for i, pid in enumerate(e["person_ids"]):
        c.execute("insert into event_person values (?,?,?)", (e["id"], pid, i))
c.commit()
c.execute("pragma user_version=1")
c.close()
print("ok", out)
