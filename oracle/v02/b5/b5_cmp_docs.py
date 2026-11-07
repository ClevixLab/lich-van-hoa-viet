# -*- coding: utf-8 -*-
"""Doi chieu muc du lieu/tai lieu (SAU khi commit d14f452): khong chay ma cua ben xay dung, chi doc
(a) luoc do JSON va vector du lieu cua ho (tai lieu/du lieu duoc phep doc), (b) tep do ho sinh ra.

Cach dung: python3 -I oracle/v02/b5/b5_cmp_docs.py <thu muc ban dung> <nhan commit>
Ghi oracle/v02/out/b5_cmp_docs_<nhan>.json.

Bo chuyen doi (adapter) ghi ro: ben xay dung giu schema_version 1 cho luoc do moi (01 §3.8a tren PR), nen
khi so quy tac truong moi, tai lieu v2 cua tham chieu duoc ha schema_version 2 -> 1 va khoa goc
'attachments' rong bi bo (ho chi ghi khi khac rong). Day la thay doi SAU doi chieu, khong sua ky vong.
"""
import copy, json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import b5_ref as R
import lvhvfile as L
import jsonschema

HERE = os.path.dirname(os.path.abspath(__file__))
build, tag = sys.argv[1], sys.argv[2]
sch = json.load(open(os.path.join(build, "docs/v0.2/schema/backup-v1.schema.json"), encoding="utf-8"))
V = jsonschema.Draft202012Validator(sch)
man = json.load(open(os.path.join(HERE, "fixtures/manifest.json"), encoding="utf-8"))
exp = {e["name"]: e for e in json.load(open(os.path.join(HERE, "../out/b5_expected.json"), encoding="utf-8"))["fixtures"]}
SCHEMA_CLASSES = {"V1", "COUNT"}       # lop loi ma JSON Schema dien dat duoc

out = {"tag": tag, "adapter": "schema_version 2->1 cho tai lieu v2; bo 'attachments' goc khi rong", "fixtures": [], "notes": []}

# 1. vector bo sung cua ben xay dung, giai ma bang bo doc doc lap
p = os.path.join(build, "docs/v0.2/schema/test-vectors/supplement-yc05/notice-v1.lvhv")
mf = json.load(open(os.path.join(build, "docs/v0.2/schema/test-vectors/supplement-yc05/manifest.json"), encoding="utf-8"))
j, bl = L.unpack(L.decrypt_file(open(p, "rb").read(), mf["password"]))
d = json.loads(j)
order = [b[0] for b in bl]
exp_order = [x["id"] for x in d.get("photos", [])] + [x["id"] for x in d.get("attachments", [])]
d2 = copy.deepcopy(d); d2["schema_version"] = 2; d2.setdefault("attachments", [])
out["builder_vector"] = {
    "file": "supplement-yc05/notice-v1.lvhv", "decrypt_independent": True,
    "schema_version": d.get("schema_version"),
    "has_certificate_photo_ids": "certificate_photo_ids" in j.decode("utf-8"),
    "block_order_photos_then_attachments": order == exp_order,
    "oracle_v2_validator_errors": R.validate(d2, dict(bl)),
    "builder_schema_errors": [e.message[:120] for e in V.iter_errors(d)],
}

# 2. bo tep thu cua kiem dinh qua luoc do cua ben xay dung
for f in man["fixtures"]:
    if f["big"]:
        continue
    doc = json.load(open(os.path.join(HERE, "fixtures", f["name"] + ".json"), encoding="utf-8"))
    raw_errs = [e.message[:100] for e in V.iter_errors(doc)]
    ad = copy.deepcopy(doc)
    if ad.get("schema_version") == 2:
        ad["schema_version"] = 1
        if ad.get("attachments") == []:
            del ad["attachments"]
    ad_errs = [e.message[:100] for e in V.iter_errors(ad)]
    e = exp[f["name"]]
    row = {"name": f["name"], "expect": e["expect"], "class": e["class"],
           "builder_schema_raw_ok": not raw_errs, "builder_schema_adapted_ok": not ad_errs,
           "adapted_errors": ad_errs[:3]}
    # so: chi voi ca ma loi ky vong thuoc lop luoc do dien dat duoc, hoac ca hop le v2
    if f["name"][0] in "GX":
        if e["expect"] == "accept":
            row["agree"] = not ad_errs
        elif set(e["class"]) <= SCHEMA_CLASSES or e["class"] == ["V1"]:
            row["agree"] = bool(ad_errs)
        else:
            row["agree"] = None     # loi can ma (sha, byte, tham chieu, magic): luoc do khong the hien
    elif f["name"][0] == "M":
        row["note"] = "tai lieu cu: luoc do moi cua ben xay dung %s" % ("CHAP NHAN" if not raw_errs else "TU CHOI")
    out["fixtures"].append(row)

# 3. tep sinh lai bang ban dung: sha256 so voi ky vong cua chinh no (manifest supplement)
out["builder_vector"]["manifest_state_sha256"] = mf.get("state_sha256")
path = os.path.join(HERE, "../out/b5_cmp_docs_%s.json" % tag)
json.dump(out, open(path, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print(json.dumps(out["builder_vector"], ensure_ascii=False, indent=1))
for r in out["fixtures"]:
    print(r["name"].ljust(36), r["expect"].ljust(18), "raw_ok=%s ad_ok=%s agree=%s %s" % (
        r["builder_schema_raw_ok"], r["builder_schema_adapted_ok"], r.get("agree"), r.get("note", "")))
