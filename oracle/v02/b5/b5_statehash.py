# -*- coding: utf-8 -*-
"""Bam trang thai kho doc lap theo 03 Bo sung B1.4 muc 10: SHA-256( JSON chuan toan kho, bo `app` va
`exported_at` ‖ sha256 hex cua byte tung anh theo thu tu id ). Bien the cho tep dinh kem (khong duoc
dac ta): them sha256 hex cua tung tep dinh kem theo thu tu id, sau anh. Bien the duoc chon bang cach
khop vector cong khai cua ben xay dung (supplement-yc05/manifest.json state_sha256) — ghi trong bao cao.
"""
import hashlib, json, sys


def canon(doc):
    d = {k: v for k, v in doc.items() if k not in ("app", "exported_at")}
    for k in ("persons", "events", "event_types", "photos", "attachments"):
        if isinstance(d.get(k), list):
            d[k] = sorted(d[k], key=lambda x: x["id"])
    return json.dumps(d, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode("utf-8")


def state_hash(doc, blobs, variant="photos+attachments"):
    h = hashlib.sha256(canon(doc))
    ids = sorted(p["id"] for p in doc.get("photos", []))
    if variant == "photos+attachments":
        ids += sorted(a["id"] for a in doc.get("attachments", []))
    elif variant == "all-sorted":
        ids = sorted(ids + [a["id"] for a in doc.get("attachments", [])])
    for i in ids:
        h.update(hashlib.sha256(blobs[i]).hexdigest().encode())
    return h.hexdigest()


if __name__ == "__main__":
    sys.path.insert(0, __file__.rsplit("/", 1)[0])
    import lvhvfile as L
    j, bl = L.unpack(L.decrypt_file(open(sys.argv[1], "rb").read(), sys.argv[2]))
    doc, b = json.loads(j), dict(bl)
    for v in ("photos-only", "photos+attachments", "all-sorted"):
        try:
            print(v, state_hash(doc, b, v))
        except KeyError as e:
            print(v, "thieu", e)
