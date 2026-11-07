# -*- coding: utf-8 -*-
"""Doi chieu hop den: lop luu tru + sao luu cua ben xay dung mo tung tep thu (qua vi du `cross_platform
check`, duong vao duy nhat nhan tep tuy y), so voi ky vong da commit truoc (d14f452).

Dau ra cua hop den: (a) loi (ma lop + duong dan) neu tu choi; (b) neu nhan: bam trang thai kho sau khi
nhap (gia tri `left` cua phep so trong chuong trinh). Bam trang thai tinh doc lap bang b5_statehash.py
(03 B1.4 muc 10; bien the cho tep dinh kem khop vector cong khai cua ben xay dung). Bam bang nhau <=>
JSON chuan toan kho + byte moi anh/tep giong het => 0 truong khac, 0 ban ghi mat.

Cach dung: python3 -I oracle/v02/b5/b5_compare.py <thu muc ban dung> <thu muc tep da chuyen dang> <nhan>
"""
import copy, json, os, re, shutil, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import b5_ref as R
import lvhvfile as L
import b5_statehash as S

HERE = os.path.dirname(os.path.abspath(__file__))
build, fxdir, tag = sys.argv[1], sys.argv[2], sys.argv[3]
PW = "mật khẩu thử nghiệm"
exp = {e["name"]: e for e in json.load(open(os.path.join(HERE, "../out/b5_expected.json"), encoding="utf-8"))["fixtures"]}
target = os.path.join(build, "pwa/test/store/out/r6-from-pwa.lvhv")
os.makedirs(os.path.dirname(target), exist_ok=True)

CLASS_MAP = {  # lop cua kiem dinh -> mau ma/loi cua ben xay dung duoc coi la cung lop
    "NEWER": r"NewerSchema", "V1": r'code: "V1"', "V3": r'code: "V3"',
    "DUP": r"trùng", "COUNT": r"quá 6|maxItems", "SIZE": r"ngoài khoảng|MalformedPlaintext|quá lớn",
    "MAGIC": r"PDF không hợp lệ|ATTACHMENT|ảnh không hợp lệ", "SHA": r"sha256", "BYTES": r"sha256/bytes|bytes",
    "BLOB": r"thiếu tệp|thừa|PHOTO|ATTACHMENT",
}


def probe(path):
    shutil.copy(path, target)
    p = subprocess.run(["./target/release/examples/cross_platform", "check"], cwd=build,
                       capture_output=True, text=True, timeout=600)
    txt = p.stdout + p.stderr
    m = re.search(r'left: "([0-9a-f]{64})"', txt)
    if m:
        return {"accepted": True, "state": m.group(1)}
    if p.returncode == 0:
        return {"accepted": True, "state": "R6"}
    m = re.search(r"Err` value: (.*)", txt)
    return {"accepted": False, "error": (m.group(1) if m else txt.strip().splitlines()[-1])[:240]}


def expected_states(doc, blobs):
    """Bam ky vong + hai bien the bieu dien tuong duong (gioi_tinh null <-> vang)."""
    out = {}
    out["nguyen-van"] = S.state_hash(doc, blobs)
    d = copy.deepcopy(doc)
    for p in d.get("persons", []):
        if p.get("gioi_tinh", 0) is None:
            del p["gioi_tinh"]
    out["bo-gioi_tinh-null"] = S.state_hash(d, blobs)
    d = copy.deepcopy(doc)
    for p in d.get("persons", []):
        p.setdefault("gioi_tinh", None)
    out["them-gioi_tinh-null"] = S.state_hash(d, blobs)
    return out


ALT = {"0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e09": "0b8e4f52-3c1a-4d2e-9f60-e5d4c3b2a1f6"}  # id thay the, doc tu
# released/expected cua ben xay dung (anh chan dung 5e09 cung la anh giay chung tu); chi dung cho ung vien bieu dien.


def builder_repr(doc, blobs, legacy_on_empty):
    """Ung vien bieu dien theo 01 §7.1 tren PR (doc SAU doi chieu): ket qua tham chieu + Person.legacy
    {certificate_photo_ids, certificate_photos}; anh chung voi anh chan dung -> tep id thay the; sv giu 1."""
    mdoc, mbl = R.migrate_1_to_2(doc, blobs)
    photos = {p["id"]: p for p in doc.get("photos", [])}
    portrait = {p.get("photo_id") for p in doc.get("persons", [])}
    for p0, p in zip(doc.get("persons", []), mdoc.get("persons", [])):
        n0 = ((p0.get("death") or {}).get("notice")) or None
        if isinstance(n0, dict) and "certificate_photo_ids" in n0:
            ids = n0["certificate_photo_ids"]
            if ids or legacy_on_empty:
                p.setdefault("legacy", {}).update({"certificate_photo_ids": list(ids),
                                                   "certificate_photos": [photos[i] for i in ids if i in photos]})
            for r in p["death"]["notice"]["attachments"]:
                if r["file_id"] in portrait and r["file_id"] in ALT:
                    r["file_id"] = ALT[r["file_id"]]
    for a in mdoc.get("attachments", []):
        if a["id"] in portrait and a["id"] in ALT:
            mbl[ALT[a["id"]]] = mbl[a["id"]]; a["id"] = ALT[a["id"]]
    mdoc["attachments"] = sorted(mdoc.get("attachments", []), key=lambda a: a["id"])
    mdoc["schema_version"] = 1
    if not mdoc["attachments"]:
        del mdoc["attachments"]
    for p in mdoc.get("persons", []):
        if p.get("gioi_tinh", 0) is None:
            del p["gioi_tinh"]
    return mdoc, mbl


rows = []
for fn in sorted(os.listdir(fxdir)):
    if not fn.endswith(".lvhv"):
        continue
    name = fn[:-5]
    e = exp[name]
    j, bl = L.unpack(L.decrypt_file(open(os.path.join(fxdir, fn), "rb").read(), PW))
    doc, blobs = json.loads(j), dict(bl)
    act = probe(os.path.join(fxdir, fn))
    row = {"name": name, "expect": e["expect"], "class": e["class"], "actual": act}
    if e["expect"] in ("accept", "accept_or_chua_ro", "chua_ro"):
        if name.startswith("M") and doc.get("schema_version") == 1 and "migrate" in e["class"]:
            mdoc, mbl = R.migrate_1_to_2(doc, blobs)
            mdoc["schema_version"] = 1                       # bieu dien cua ben xay dung (khong tang phien ban)
            if mdoc.get("attachments") == []:
                del mdoc["attachments"]
            cand = {"chuyen-doi:" + k: v for k, v in expected_states(mdoc, mbl).items()}
            cand.update({"nguyen-tep:" + k: v for k, v in expected_states(doc, blobs).items()})
            for le in (False, True):
                try:
                    bd, bb = builder_repr(doc, blobs, le)
                    cand["bieu-dien-PR(legacy%s)" % ("+rong" if le else "")] = S.state_hash(bd, bb)
                except Exception as ex:  # tep loi (vd M09) khong co ung vien
                    cand["bieu-dien-PR-loi"] = str(ex)[:80]
        else:
            cand = expected_states(doc, blobs)
        row["expected_states"] = cand
        if act["accepted"]:
            hit = [k for k, v in cand.items() if v == act["state"]]
            row["state_match"] = hit
            if e["expect"] == "accept":
                row["verdict"] = "KHOP" if hit else "SAI_LECH_TRUONG"
            else:
                row["verdict"] = "GHI_NHAN"
        else:
            row["verdict"] = "SAI_LECH_TU_CHOI" if e["expect"] == "accept" else "GHI_NHAN"
    else:  # reject
        if act["accepted"]:
            row["verdict"] = "SAI_LECH_CHAP_NHAN"
        else:
            newer_ok = ("NEWER" in e["class"]) == ("NewerSchema" in act["error"])
            fine = any(re.search(CLASS_MAP[c], act["error"]) for c in e["class"] if c in CLASS_MAP)
            row["class_newer_ok"] = newer_ok
            row["class_fine_ok"] = bool(fine)
            row["verdict"] = "KHOP" if newer_ok else "SAI_LOP"
    rows.append(row)
    print(name.ljust(36), e["expect"].ljust(18), row["verdict"].ljust(20),
          (act.get("error") or ",".join(row.get("state_match", [])) or act.get("state", ""))[:110])
if os.path.exists(target):
    os.remove(target)
summ = {}
for r in rows:
    summ[r["verdict"]] = summ.get(r["verdict"], 0) + 1
res = {"tag": tag, "fixtures_dir": os.path.basename(fxdir.rstrip("/")), "summary": summ, "rows": rows}
json.dump(res, open(os.path.join(HERE, "../out/b5_cmp_%s.json" % tag), "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print(summ)
