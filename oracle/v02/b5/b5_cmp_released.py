# -*- coding: utf-8 -*-
"""Chay chuyen doi THAM CHIEU (b5_ref.migrate_1_to_2, commit d14f452) tren bo mau dong bang cua ben xay dung
(test-vectors/released/0.2-dev-before-attachments, chi doc nhu du lieu) va so voi tep ky vong cua ho
(released/expected/0.2-dev-before-attachments.json). So khong phu thuoc id (A-M1) va tach rieng `legacy`.

Cach dung: python3 -I oracle/v02/b5/b5_cmp_released.py <thu muc ban dung> <nhan>
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import b5_ref as R
import lvhvfile as L
import b5_statehash as S

b, tag = sys.argv[1], sys.argv[2]
rel = os.path.join(b, "docs/v0.2/schema/test-vectors/released")
man = json.load(open(os.path.join(rel, "0.2-dev-before-attachments/manifest.json"), encoding="utf-8"))
j, bl = L.unpack(L.decrypt_file(open(os.path.join(rel, "0.2-dev-before-attachments/backup.lvhv"), "rb").read(), man["password"]))
old, blobs = json.loads(j), dict(bl)
snap = json.load(open(os.path.join(rel, "0.2-dev-before-attachments/store-snapshot.json"), encoding="utf-8"))
exp_b = json.load(open(os.path.join(rel, "expected/0.2-dev-before-attachments.json"), encoding="utf-8"))
out = {"tag": tag, "old_schema_version": old.get("schema_version"),
       "old_has_certificate_photo_ids": "certificate_photo_ids" in j.decode("utf-8"),
       "expected_file_keys": sorted(exp_b) if isinstance(exp_b, dict) else type(exp_b).__name__}
bdoc = exp_b.get("state", exp_b.get("doc", exp_b)) if isinstance(exp_b, dict) else exp_b
mine, mblobs = R.migrate_1_to_2(old, blobs)
na, la = R.norm_for_compare(mine)
nb, lb = R.norm_for_compare(bdoc)
for d in (na, nb):
    d.pop("schema_version", None)
out["diff_ref_vs_builder_expected"] = [(p, str(x)[:160], str(y)[:160]) for p, x, y in R.diff(na, nb)]
out["builder_legacy"] = {k: str(v)[:400] for k, v in lb.items()}
out["ref_legacy"] = {k: str(v)[:400] for k, v in la.items()}
out["builder_ids_ok_bad"] = R.ids_ok(bdoc)
# moi gia tri cu co con trong ky vong cua ben xay dung? (0 truong mat)
def leaves(o, p=""):
    if isinstance(o, dict):
        for k, v in o.items():
            yield from leaves(v, p + "/" + k)
    elif isinstance(o, list):
        for i, v in enumerate(o):
            yield from leaves(v, p + "/" + str(i))
    else:
        yield p, o
bl_str = json.dumps(bdoc, ensure_ascii=False, sort_keys=True)
missing = [(p, v) for p, v in leaves({k: v for k, v in old.items() if k not in ("app", "exported_at")})
           if isinstance(v, str) and len(v) > 3 and json.dumps(v, ensure_ascii=False) not in bl_str]
out["old_string_values_absent_in_builder_expected"] = missing
out["builder_expected_state_sha256"] = exp_b.get("state_sha256") if isinstance(exp_b, dict) else None
try:
    out["independent_state_sha256_of_builder_doc"] = S.state_hash(bdoc, {**blobs, **mblobs})
except KeyError as e:
    out["independent_state_sha256_of_builder_doc"] = "thieu byte %s (id thay the)" % e
json.dump(out, open(os.path.join(os.path.dirname(__file__), "../out/b5_cmp_released_%s.json" % tag), "w", encoding="utf-8"),
          ensure_ascii=False, indent=1)
print(json.dumps(out, ensure_ascii=False, indent=1)[:6000])
