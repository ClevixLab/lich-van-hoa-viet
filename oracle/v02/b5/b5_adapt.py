# -*- coding: utf-8 -*-
"""Bo chuyen doi SAU doi chieu (khong sua ky vong): dua tep thu v2 cua kiem dinh ve dang ben xay dung
dung o PR #28 (schema_version giu 1; khoa goc 'attachments' chi khi khac rong; khoi anh roi khoi tep dinh kem
theo thu tu mang, 03 §2.3 tren PR). Tep thu v1 (dang cu) giu nguyen.

Cach dung: python3 -I oracle/v02/b5/b5_adapt.py <thu muc ra> [thu muc tep lon]
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lvhvfile as L
import b5_fixtures as F

HERE = os.path.dirname(os.path.abspath(__file__))
out = sys.argv[1]; big = sys.argv[2] if len(sys.argv) > 2 else None
os.makedirs(out, exist_ok=True)
man = json.load(open(os.path.join(HERE, "fixtures/manifest.json"), encoding="utf-8"))
for f in man["fixtures"]:
    if f["big"] and not big:
        continue
    src = big if f["big"] else os.path.join(HERE, "fixtures")
    doc = json.load(open(os.path.join(src, f["name"] + ".json"), encoding="utf-8"))
    blobs = {}
    for i, m in f["blobs"].items():
        p = os.path.join(big, f["name"] + ".bytes", i + ".bin") if f["big"] else os.path.join(HERE, "fixtures/bytes", m["sha256"] + ".bin")
        blobs[i] = open(p, "rb").read()
    if doc.get("schema_version") == 2:
        doc["schema_version"] = 1
        if doc.get("attachments") == []:
            del doc["attachments"]
    order = [p["id"] for p in doc.get("photos", [])] + [a["id"] for a in doc.get("attachments", [])]
    seen, bl = set(), []
    for i in order + sorted(blobs):
        if i in blobs and i not in seen:
            bl.append((i, blobs[i])); seen.add(i)
    pt = L.pack(L.canonical_json(doc), bl)
    open(os.path.join(out, f["name"] + ".lvhv"), "wb").write(L.encrypt_file(pt, F.PW, F.SALT, F.NONCE))
print("adapted:", len(os.listdir(out)))
