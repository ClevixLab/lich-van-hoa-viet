#!/usr/bin/env python3
"""Vòng 2 (báo cáo §5): V2-1..V2-5. Hộp đen; dùng lại phần hàm của run_r15.py (không chạy phần ca của nó).

Dùng: python3 -I run_r15b.py <repo> <thư-mục-làm-việc> <rust-tool>
(<thư-mục-làm-việc>/img phải có ảnh của make_inputs.py; script tự thêm ảnh 1600/1601 px.)
Ghi <thư-mục-làm-việc>/results-v2.json.
"""
import io, os, sys, json, copy, hashlib
HERE = os.path.dirname(os.path.abspath(__file__))
src = open(os.path.join(HERE, "run_r15.py"), encoding="utf-8").read()
exec(compile(src.split("SIDES = (\"rs\", \"ts\")")[0], "run_r15.py(helpers)", "exec"))
SIDES = ("rs", "ts")
from PIL import Image
_, _exb = L.unpack_plain(L.decrypt(open(os.path.join(TV, "example-v1.lvhv"), "rb").read(), PW))
EXB = dict(_exb)


def img(fn): return open(os.path.join(IMG, fn), "rb").read()


# ---- ảnh 1600/1601 px (tất định) ----
def _jpg(w, h, seed):
    im = Image.new("RGB", (w, h))
    px = im.load()
    for y in range(h):
        for x in range(w):
            px[x, y] = ((x * 3 + seed) % 256, (y * 11) % 256, (x ^ y) % 256)
    b = io.BytesIO(); im.save(b, "JPEG", quality=80); return b.getvalue()

for fn, w, h, sd in (("wide1601.jpg", 1601, 100, 7), ("tall1601.jpg", 100, 1601, 8), ("w1600.jpg", 1600, 100, 9)):
    data = _jpg(w, h, sd)
    open(os.path.join(IMG, fn), "wb").write(data)
    MAN[fn] = {"mime": "image/jpeg", "width": w, "height": h, "bytes": len(data), "sha256": sha(data)}
PDF = b"%PDF-1.4\n1 0 obj<<>>endobj\ntrailer<<>>\n%%EOF\n"
json.dump({k: MAN[k] for k in ("wide1601.jpg", "tall1601.jpg", "w1600.jpg")} | {"f0.pdf": {"bytes": len(PDF), "sha256": sha(PDF)}},
          open(os.path.join(W, "v2-images.json"), "w"), indent=1, sort_keys=True)

R2 = {}
W1601, T1601, W1600 = ("0b8e4f52-3c1a-4d2e-9f60-0000000000c1", "0b8e4f52-3c1a-4d2e-9f60-0000000000c2",
                       "0b8e4f52-3c1a-4d2e-9f60-0000000000c3")
P0 = EX["persons"][0]["photo_id"]
F0, NA0 = "0b8e4f52-3c1a-4d2e-9f60-0000000000f0", "0b8e4f52-3c1a-4d2e-9f60-0000000000f1"
T0, T1, Q0 = TRAY, TRAY2, PORT


def view2(st):
    v = store_view(st)
    doc = (st.get("store") or {}).get("document") or {}
    used_att = {a["file_id"] for p in doc.get("persons", [])
                for a in (((p.get("death") or {}).get("notice") or {}).get("attachments") or [])}
    recs_att = {a["id"] for a in doc.get("attachments", [])}
    v["att_ids"] = sorted(recs_att)
    v["orphan_att_records"] = sorted(recs_att - used_att)
    v["att_sha"] = {k: x.get("sha256") for k, x in ((st.get("store") or {}).get("attachment_bytes") or {}).items()}
    files = st.get("files_after")
    if files is not None:
        af = {os.path.basename(k).split(".")[0] for k in files if k.startswith("attachments/")}
        v["orphan_att_files"] = sorted(af ^ recs_att)
    v["null_tray_key"] = any("tray_photo_id" in (p.get("memorial") or {}) and p["memorial"]["tray_photo_id"] is None
                             for p in doc.get("persons", []))
    return v


def pd(i, rev=None, **kw):
    p = person(i) if i != 2 else copy.deepcopy(EX["persons"][2])
    for k, v in kw.items():
        if k == "photo_id": p["photo_id"] = v
        else: p["memorial"][k] = v
    return p

# ================= V2-1 =================
for s in SIDES:
    c = R2.setdefault("v2_1", {}).setdefault(s, {})
    k = Store(s, "v21")
    for fn, pid in (("clean.jpg", TRAY), ("wide1601.jpg", W1601), ("tall1601.jpg", T1601), ("w1600.jpg", W1600)):
        c["add_" + fn] = verdict(k.add_photo(fn, pid))
    c["base_D2_tray_clean"] = verdict(k.save(p_d2(tray=TRAY)))
    c["D2_tray_1601x100"] = verdict(k.save(p_d2(tray=W1601)))
    c["D2_tray_100x1601"] = verdict(k.save(p_d2(tray=T1601)))
    v = view2(k.open())
    c["after_rejects"] = {"D2_tray": v["trays"].get(D2), "TRAY_present": TRAY in v["photo_sha"]}
    c["D2_tray_1600x100"] = verdict(k.save(p_d2(tray=W1600)))
    c["D1_portrait_1601x100"] = verdict(k.save(person(0, photo_id=W1601)))
    v = view2(k.open())
    c["final"] = {"trays": v["trays"], "photos": sorted(v["photo_sha"]), "orphan_records": v["orphan_records"],
                  "orphan_files": v["orphan_files"]}

# ================= crafted V2-2, V2-3, V2-4 =================
def craft2(name, doc, blobs):
    doc = copy.deepcopy(doc)
    doc["photos"] = sorted(doc["photos"], key=lambda r: r["id"])
    if doc.get("attachments"):
        doc["attachments"] = sorted(doc["attachments"], key=lambda r: r["id"])
    elif "attachments" in doc:
        del doc["attachments"]
    order = [r["id"] for r in doc["photos"]] + [a["id"] for a in doc.get("attachments", [])]
    pt = L.pack_plain(doc, [(i, blobs[i]) for i in order])
    f = os.path.join(W, "crafted2", name + ".lvhv")
    os.makedirs(os.path.dirname(f), exist_ok=True)
    open(f, "wb").write(L.encrypt(pt, PW))
    return f


def base():
    return copy.deepcopy(EX), dict(EXB)


def with_tray(fn, pid, who=1, as_portrait=False):
    d, b = base()
    m = MAN[fn]; data = img(fn)
    d["photos"].append(L.photo_record(pid, data, m["mime"], m["width"], m["height"]))
    b[pid] = data
    if as_portrait: d["persons"][who]["photo_id"] = pid
    else: d["persons"][who]["memorial"]["tray_photo_id"] = pid
    return d, b

F2 = {
    "v2_2_tray_1601": craft2("v2_2_tray_1601", *with_tray("wide1601.jpg", W1601)),
    "v2_2_tray_1600_control": craft2("v2_2_tray_1600_control", *with_tray("w1600.jpg", W1600)),
    "v2_2_portrait_1601": craft2("v2_2_portrait_1601", *with_tray("wide1601.jpg", W1601, as_portrait=True)),
}
# V2-3: tệp A (trên máy) và B (gộp vào)
dA, bA = base()
dA["persons"][0]["death"]["notice"] = {"reported_by": None, "place": None, "reason": None, "notice_date": None,
                                       "note": None, "attachments": [{"id": NA0, "kind": "khac", "file_id": F0}]}
dA["attachments"] = [{"id": F0, "mime": "application/pdf", "bytes": len(PDF), "sha256": sha(PDF), "stripped": False,
                      "file_name": None, "created_at": "2026-11-07T00:00:00Z"}]
bA[F0] = PDF
for fn, pid, who, port in (("clean.jpg", T0, 1, False), ("clean2.jpg", Q0, 2, True)):
    m = MAN[fn]; data = img(fn)
    dA["photos"].append(L.photo_record(pid, data, m["mime"], m["width"], m["height"])); bA[pid] = data
    if port: dA["persons"][who]["photo_id"] = pid
    else: dA["persons"][who]["memorial"]["tray_photo_id"] = pid


def bump(doc, rev):
    for i in (0, 1):
        doc["persons"][i]["rev"] = rev
        doc["persons"][i]["updated_at"] = "2026-11-07T0%d:00:00Z" % min(rev, 9)

dA_new = copy.deepcopy(dA); bump(dA_new, 9)           # bản trên máy MỚI hơn B (ca ngược)
dB = copy.deepcopy(dA); bB = {}
bump(dB, 7)
dB["persons"][0]["photo_id"] = None
dB["persons"][0]["death"]["notice"]["attachments"] = []
dB["persons"][1]["memorial"]["tray_photo_id"] = T1
dB["persons"] = dB["persons"][:2]
dB["events"] = []
dB["attachments"] = []
m = MAN["clean3.jpg"]; data = img("clean3.jpg")
dB["photos"] = [L.photo_record(T1, data, m["mime"], m["width"], m["height"])]
bB[T1] = data
for p in dA["persons"][:2]: p["rev"], p["updated_at"] = 1, "2026-11-06T03:15:00Z"
F2["v2_3_A"] = craft2("v2_3_A", dA, bA)
F2["v2_3_A_newer"] = craft2("v2_3_A_newer", dA_new, bA)
F2["v2_3_B"] = craft2("v2_3_B", dB, bB)
dn, bn = base(); dn["persons"][1]["memorial"]["tray_photo_id"] = None
F2["v2_4_null"] = craft2("v2_4_null", dn, bn)

for s in SIDES:
    d = R2.setdefault("v2_2", {}).setdefault(s, {})
    for k in ("v2_2_tray_1601", "v2_2_tray_1600_control", "v2_2_portrait_1601"):
        for mode in ("--replace", "--merge"):
            t = Store(s, f"{k}{mode}"); before = view2(t.open())
            j = t.imp(F2[k], mode); v = view2(t.open())
            d[k + mode] = {"verdict": verdict(j), "unchanged_if_rejected": (before == v) if not j.get("ok") else None,
                           "trays": v["trays"], "photos": sorted(v["photo_sha"])}
    d3 = R2.setdefault("v2_3", {}).setdefault(s, {})
    for nm, srcf in (("forward", "v2_3_A"), ("reverse_local_newer", "v2_3_A_newer")):
        t = Store(s, "v23-" + nm); t.open()
        ja = t.imp(F2[srcf], "--replace"); va = view2(t.open())
        jb = t.imp(F2["v2_3_B"], "--merge"); vb = view2(t.open())
        d3[nm] = {"import_A": verdict(ja), "A_photos": sorted(va["photo_sha"]), "A_atts": va["att_ids"],
                  "merge_B": verdict(jb), "photos": sorted(vb["photo_sha"]), "atts": vb["att_ids"],
                  "trays": vb["trays"],
                  "P0": P0 in vb["photo_sha"], "F0": F0 in vb["att_ids"], "T0": T0 in vb["photo_sha"],
                  "T1": T1 in vb["photo_sha"], "Q0": Q0 in vb["photo_sha"],
                  "T1_sha_eq": vb["photo_sha"].get(T1) == MAN["clean3.jpg"]["sha256"],
                  "orphan_records": vb["orphan_records"], "orphan_att_records": vb["orphan_att_records"],
                  "orphan_files": vb["orphan_files"], "orphan_att_files": vb.get("orphan_att_files"),
                  "bad_bytes": vb["bad_bytes"]}
    d4 = R2.setdefault("v2_4", {}).setdefault(s, {})
    for mode in ("--replace", "--merge"):
        t = Store(s, "v24" + mode); t.open()
        j = t.imp(F2["v2_4_null"], mode); v = view2(t.open())
        out = os.path.join(W, f"v24-{s}{mode}.lvhv"); t.export(out)
        js, _ = L.unpack_plain(L.decrypt(open(out, "rb").read(), PW))
        d4[mode] = {"verdict": verdict(j), "store_null_key": v["null_tray_key"], "store_has_tray_key": v["has_tray_key"],
                    "reexport_has_tray_string": b"tray_photo_id" in js}
    t = Store(s, "v24-save"); t.open()
    j = t.save(p_d2(tray=None) | {"memorial": {**p_d2()["memorial"], "tray_photo_id": None}})
    saved = (j.get("saved") or {}).get("memorial") or {}
    d4["save_null"] = {"verdict": verdict(j), "saved_has_key": "tray_photo_id" in saved}

# ================= V2-5 =================
for s in SIDES:
    wd = os.path.join(W, "scen-keep-" + s)
    shutil.rmtree(wd, ignore_errors=True); os.makedirs(os.path.join(wd, "keep"))
    open(os.path.join(wd, "keep", "file.txt"), "w").write("giữ nguyên\n")
    j = tool(s, "scenario", "tray_photo", wd)
    listing = sorted(os.path.relpath(os.path.join(dp, f), wd) for dp, _, fs in os.walk(wd) for f in fs)
    dirs = sorted(os.path.relpath(os.path.join(dp, x), wd) for dp, ds, _ in os.walk(wd) for x in ds)
    R2.setdefault("v2_5", {})[s] = {"scenario": {k: v for k, v in j.items() if k != "_rc"}, "rc": j["_rc"],
                                    "files_left": listing, "dirs_left": dirs,
                                    "keep_intact": open(os.path.join(wd, "keep", "file.txt")).read() == "giữ nguyên\n"}
exp = json.load(open(os.path.join(TV, "tray-photo/expected-scenario.json")))
R2["v2_5"]["rs_eq_ts"] = L.canon(R2["v2_5"]["rs"]["scenario"]) == L.canon(R2["v2_5"]["ts"]["scenario"])
R2["v2_5"]["eq_expected"] = L.canon(R2["v2_5"]["rs"]["scenario"]) == L.canon(exp)
json.dump(R2, open(os.path.join(W, "results-v2.json"), "w"), indent=1, ensure_ascii=False, sort_keys=True)
print("xong:", os.path.join(W, "results-v2.json"))
