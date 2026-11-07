#!/usr/bin/env python3
"""Chạy kiểm định R15 (c)(d)(e)(g) trên hai bản lvhv-store-tool, cùng đầu vào. Hộp đen: chỉ gọi công cụ.

Dùng: python3 -I run_r15.py <repo> <thư-mục-làm-việc> <rust-tool>
  <repo>: bản checkout nhánh cần kiểm (có pwa/node_modules); <rust-tool>: target/debug/lvhv-store-tool
Ghi <thư-mục-làm-việc>/results.json (mọi ca, hai bản) và in tóm tắt.
"""
import copy, hashlib, json, os, shutil, struct, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lvhvpack as L

REPO, W, RS = sys.argv[1], sys.argv[2], os.path.abspath(sys.argv[3])
TV = os.path.join(REPO, "docs/v0.2/schema/test-vectors")
IMG = os.path.join(W, "img")
PW = "mật khẩu thử nghiệm"
os.makedirs(W, exist_ok=True)
MAN = json.load(open(os.path.join(IMG, "manifest.json")))
R = {}          # kết quả thô
LOG = open(os.path.join(W, "calls.log"), "w")


def tool(side, *args):
    cmd = [RS, *args] if side == "rs" else ["node", "pwa/test/tools/lvhv-store-tool.mjs", *args]
    p = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)
    LOG.write(json.dumps({"side": side, "args": args, "rc": p.returncode, "stderr": p.stderr[-500:]},
                         ensure_ascii=False) + "\n")
    try:
        j = json.loads(p.stdout)
    except Exception:
        j = {"_unparsed": p.stdout[-500:], "_stderr": p.stderr[-500:]}
    j["_rc"] = p.returncode
    return j


class Store:
    """Rust: thư mục kho; TS: tệp --state."""
    def __init__(self, side, name):
        self.side = side
        self.path = os.path.join(W, "stores", f"{name}-{side}" + ("" if side == "rs" else ".json"))
        os.makedirs(os.path.dirname(self.path), exist_ok=True)
        if os.path.isdir(self.path): shutil.rmtree(self.path)
        if os.path.isfile(self.path): os.remove(self.path)

    def a(self, cmd, *rest):
        if self.side == "rs":
            return (cmd, self.path, *rest)
        return (cmd, "-", *rest, "--state", self.path)

    def add_photo(self, fn, pid=None):
        m = MAN[fn]
        extra = ("--id", pid) if pid else ()
        return tool(self.side, *self.a("add-photo", m["mime"], os.path.join(IMG, fn), str(m["width"]),
                                      str(m["height"]), *extra))

    def save(self, person):
        f = os.path.join(W, "tmp-person.json")
        json.dump(person, open(f, "w"), ensure_ascii=False)
        return tool(self.side, *self.a("save-person", f))

    def open(self):
        return tool(self.side, *self.a("open"))

    def export(self, out):
        return tool(self.side, *self.a("export", PW, out))

    def imp(self, f, mode):
        if self.side == "rs":
            return tool("rs", "import", f, PW, mode, "--into", self.path)
        return tool("ts", "import", f, PW, mode, "--state", self.path)

    def copy_to(self, name):
        s = Store(self.side, name)
        (shutil.copytree if self.side == "rs" else shutil.copy)(self.path, s.path)
        return s


def sha(b): return hashlib.sha256(b).hexdigest()
def fsha(fn): return sha(open(fn, "rb").read())


# ---------- tự kiểm siêu dữ liệu (độc lập) ----------
def jpeg_markers(b):
    assert b[:2] == b"\xff\xd8"
    i, ms = 2, []
    while i < len(b):
        if b[i] != 0xFF: return ms + ["?bad"]
        m = b[i + 1]
        if m == 0xD9: ms.append("EOI"); break
        if m in (0x01,) or 0xD0 <= m <= 0xD7: i += 2; continue
        (ln,) = struct.unpack(">H", b[i + 2:i + 4]); ms.append("%02X" % m)
        i += 2 + ln
        if m == 0xDA:   # bỏ qua dữ liệu quét tới marker kế tiếp
            while i < len(b) - 1 and not (b[i] == 0xFF and b[i + 1] not in (0x00,) and not 0xD0 <= b[i + 1] <= 0xD7):
                i += 1
    return ms


def has_metadata(b, mime):
    if mime == "image/jpeg":
        ms = jpeg_markers(b)
        return any(m in ("E1", "ED") for m in ms) or b"Exif\x00\x00" in b or b"http://ns.adobe.com/xap" in b \
            or b"x:xmpmeta" in b
    if mime == "image/webp":
        o, chunks = 12, []
        while o + 8 <= len(b):
            cid = b[o:o + 4]; (ln,) = struct.unpack("<I", b[o + 4:o + 8]); chunks.append(cid); o += 8 + ln + (ln & 1)
        return b"EXIF" in chunks or b"XMP " in chunks
    return None


def store_view(st):
    """Rút gọn trạng thái kho: tài liệu, sha ảnh, mồ côi."""
    s = st.get("store") or {}
    doc = s.get("document") or {}
    pb = s.get("photo_bytes") or {}
    used = set()
    for p in doc.get("persons", []):
        if p.get("photo_id"): used.add(p["photo_id"])
        mem = p.get("memorial") or {}
        if mem.get("tray_photo_id"): used.add(mem["tray_photo_id"])
        for a in ((p.get("death") or {}).get("notice") or {}).get("attachments", []) or []:
            pass
    recs = {ph["id"]: ph for ph in doc.get("photos", [])}
    orphan_records = sorted(set(recs) - used)
    bad_bytes = sorted(k for k, v in pb.items() if "error" in v)
    files = st.get("files_after")
    orphan_files = None
    if files is not None:
        img = {os.path.basename(k).split(".")[0] for k in files if k.startswith("photos/")}
        orphan_files = sorted(img ^ set(recs))
    trays = {p["id"]: (p.get("memorial") or {}).get("tray_photo_id", "ABSENT") if p.get("memorial") is not None
             else "NO_MEMORIAL" for p in doc.get("persons", [])}
    return {"trays": trays, "photo_sha": {k: v.get("sha256") for k, v in pb.items()},
            "orphan_records": orphan_records, "orphan_files": orphan_files, "bad_bytes": bad_bytes,
            "has_tray_key": "tray_photo_id" in json.dumps(doc)}


def verdict(j):
    import re
    err = j.get("error") or {}
    m = re.search(r"(V\d|PHOTO|ATTACHMENT|JSON) (/[^\s:]*)", json.dumps(err, ensure_ascii=False))
    return {"ok": j.get("ok"), "rc": j.get("_rc"), "stage": j.get("stage"), "outcome": j.get("outcome"),
            "v": (m.group(1) + " " + m.group(2)) if m else None,
            "display": err.get("display") or err.get("import_message")}


EX = json.load(open(os.path.join(TV, "example-v1.json")))
D1, D2, LV = (EX["persons"][0]["id"], EX["persons"][1]["id"], EX["persons"][2]["id"])
TRAY = "0b8e4f52-3c1a-4d2e-9f60-0000000000b1"
TRAY2 = "0b8e4f52-3c1a-4d2e-9f60-0000000000b2"
PORT = "0b8e4f52-3c1a-4d2e-9f60-0000000000b3"
MISSING = "0b8e4f52-3c1a-4d2e-9f60-0000000000ff"


def person(i, **mem):
    p = copy.deepcopy(EX["persons"][i])
    for k, v in mem.items():
        if k == "photo_id": p["photo_id"] = v
        else: p["memorial"][k] = v
    return p


def p_d2(tray=None, rev=None, photo=None):
    p = person(1)
    if tray is not None: p["memorial"]["tray_photo_id"] = tray
    if photo is not None: p["photo_id"] = photo
    return p


SIDES = ("rs", "ts")

# ================= (g1) kịch bản cố định =================
for s in SIDES:
    wd = os.path.join(W, "scen-rs")
    j = tool(s, "scenario", "tray_photo", wd)
    R.setdefault("g1", {})[s] = j
exp = json.load(open(os.path.join(TV, "tray-photo/expected-scenario.json")))
R["g1"]["rs_eq_ts"] = L.canon({k: v for k, v in R["g1"]["rs"].items() if k != "_rc"}) == \
    L.canon({k: v for k, v in R["g1"]["ts"].items() if k != "_rc"})
R["g1"]["rs_eq_expected"] = L.canon({k: v for k, v in R["g1"]["rs"].items() if k != "_rc"}) == L.canon(exp)

# ================= (c) =================
for s in SIDES:
    c = R.setdefault("c", {}).setdefault(s, {})
    st = Store(s, "c")
    for fn in ("exif_gps.jpg", "xmp.jpg", "app13.jpg", "exif.webp", "big.jpg", "fake.jpg"):
        c["c1_" + fn] = verdict(st.add_photo(fn))
    for fn, pid in (("clean.jpg", TRAY), ("clean.webp", TRAY2), ("clean2.jpg", PORT)):
        j = st.add_photo(fn, pid)
        c["c2_" + fn] = verdict(j)
    o = st.open()
    pb = o["store"]["photo_bytes"]
    c["c2_bytes"] = {pid: {"sha_eq_input": pb.get(pid, {}).get("sha256") == MAN[fn]["sha256"],
                           "metadata": None} for fn, pid in (("clean.jpg", TRAY), ("clean.webp", TRAY2), ("clean2.jpg", PORT))}
    # byte đọc lại: Rust có tệp photos/<id>.img; TS: lấy qua export + giải mã độc lập
    ex = os.path.join(W, f"c-export-{s}.lvhv")
    st.save(person(0, tray_photo_id=TRAY, photo_id=None))  # gắn để ảnh vào tệp xuất
    st.save(p_d2(tray=TRAY2, photo=PORT))
    st.export(ex)
    _, blobs = L.unpack_plain(L.decrypt(open(ex, "rb").read(), PW))
    bl = dict(blobs)
    for fn, pid in (("clean.jpg", TRAY), ("clean.webp", TRAY2), ("clean2.jpg", PORT)):
        c["c2_bytes"][pid]["metadata"] = has_metadata(bl[pid], MAN[fn]["mime"])
        c["c2_bytes"][pid]["export_sha_eq_input"] = sha(bl[pid]) == MAN[fn]["sha256"]
    # c4 — liên kết đúng người, trên kho riêng
    k = Store(s, "c4")
    k.add_photo("clean.jpg", TRAY); k.add_photo("clean2.jpg", PORT); k.add_photo("clean3.jpg", TRAY2)
    c["c4_A_tray"] = verdict(k.save(person(0, tray_photo_id=TRAY)))            # 5e01 có photo_id 5e09 không có trong kho!
    c["c4_A_tray_noportrait"] = verdict(k.save(person(0, tray_photo_id=TRAY, photo_id=None)))
    c["c4_B_share_tray"] = verdict(k.save(p_d2(tray=TRAY)))
    c["c4_B_portrait_is_A_tray"] = verdict(k.save(p_d2(photo=TRAY)))
    c["c4_B_dangling"] = verdict(k.save(p_d2(tray=MISSING)))
    c["c4_B_own_portrait_as_tray"] = verdict(k.save(p_d2(tray=PORT, photo=PORT)))
    c["c4_B_ok"] = verdict(k.save(p_d2(tray=TRAY2, photo=PORT)))
    c["c4_A_tray_is_B_portrait"] = verdict(k.save(person(0, tray_photo_id=PORT, photo_id=None)))
    lv = copy.deepcopy(EX["persons"][2])
    c["c4_living_portrait_is_B_tray"] = verdict(k.save({**lv, "photo_id": TRAY2}))
    c["c4_view"] = store_view(k.open())
    # c5 — thăm dò 2000 px
    k5 = Store(s, "c5")
    c["c5_add_wide2000"] = verdict(k5.add_photo("wide2000.jpg", TRAY))
    c["c5_set_tray_wide2000"] = verdict(k5.save(p_d2(tray=TRAY)))

# ================= (d) =================
for s in SIDES:
    d = R.setdefault("d", {}).setdefault(s, {})
    a = Store(s, "dA")
    a.save(p_d2())                                              # rev 1, chưa có ảnh
    old = a.copy_to("dOld")                                     # kho "bản cũ"
    d["d1_add"] = verdict(a.add_photo("clean.jpg", TRAY))
    d["d1_save"] = verdict(a.save(p_d2(tray=TRAY)))             # rev 2
    d["d1_save_rev"] = None
    va = a.open(); d["d1_view_src"] = store_view(va)
    d["d1_save_rev"] = [p["rev"] for p in va["store"]["document"]["persons"] if p["id"] == D2]
    ex = os.path.join(W, f"d-export-{s}.lvhv")
    d["d1_export"] = verdict(a.export(ex))
    ins = tool(s, "inspect-export", ex, PW)
    pers = [p for p in ins["document"]["persons"] if p["id"] == D2][0]
    d["d1_inspect"] = {"tray_key": pers["memorial"].get("tray_photo_id"),
                       "blob_sha": {b["id"]: b["sha256"] for b in ins["blobs"]},
                       "blob_sha_eq_input": {b["id"]: b["sha256"] for b in ins["blobs"]}.get(TRAY) == MAN["clean.jpg"]["sha256"]}
    # độc lập: tự giải mã tệp xuất
    _, bl = L.unpack_plain(L.decrypt(open(ex, "rb").read(), PW))
    d["d1_indep_blob_sha_eq"] = sha(dict(bl)[TRAY]) == MAN["clean.jpg"]["sha256"]
    for nm, mode, base in (("d2_replace_empty", "--replace", None), ("d3_merge_empty", "--merge", None),
                           ("d4_merge_over_old", "--merge", old)):
        tgt = base.copy_to(nm) if base else Store(s, nm)
        if not base: tgt.open()
        j = tgt.imp(ex, mode)
        v = store_view(tgt.open())
        d[nm] = {"verdict": verdict(j), "tray": v["trays"].get(D2), "sha_eq": v["photo_sha"].get(TRAY) == MAN["clean.jpg"]["sha256"],
                 "orphan_records": v["orphan_records"], "orphan_files": v["orphan_files"], "bad_bytes": v["bad_bytes"]}
    # d4b (bổ sung sau đăng ký, không vào ngưỡng): kho đích có bản MỚI hơn, không ảnh → ảnh nhập không được thêm
    nb = old.copy_to("d4b")
    nb.save(p_d2()); nb.save(p_d2())                             # rev 3 > rev 2 của tệp
    j = nb.imp(ex, "--merge"); v = store_view(nb.open())
    d["d4b_merge_local_newer"] = {"verdict": verdict(j), "tray": v["trays"].get(D2), "photo_ids": sorted(v["photo_sha"]),
                                  "orphan_records": v["orphan_records"], "orphan_files": v["orphan_files"]}

# ---------- tệp tự mã hóa (d5–d9, e5) ----------
_, ex_blobs = L.unpack_plain(L.decrypt(open(os.path.join(TV, "example-v1.lvhv"), "rb").read(), PW))
EXB = dict(ex_blobs)


def craft(name, mut):
    doc = copy.deepcopy(EX)
    blobs = dict(EXB)
    mut(doc, blobs)
    doc["photos"] = sorted(doc["photos"], key=lambda r: r["id"])
    order = [r["id"] for r in doc["photos"]]
    pt = L.pack_plain(doc, [(i, blobs[i]) for i in order if i in blobs])
    f = os.path.join(W, "crafted", name + ".lvhv")
    os.makedirs(os.path.dirname(f), exist_ok=True)
    open(f, "wb").write(L.encrypt(pt, PW))
    return f


def img(fn): return open(os.path.join(IMG, fn), "rb").read()


def add_tray(doc, blobs, fn, pid=TRAY, who=1, mime=None):
    m = MAN[fn]
    data = img(fn)
    doc["photos"].append(L.photo_record(pid, data, mime or m["mime"], m["width"], m["height"]))
    blobs[pid] = data
    doc["persons"][who]["memorial"]["tray_photo_id"] = pid


CR = {
    "d9_control": lambda d, b: add_tray(d, b, "clean.jpg"),
    "d5_dangling": lambda d, b: d["persons"][1]["memorial"].__setitem__("tray_photo_id", MISSING),
    "d6_tray_exif_gps": lambda d, b: add_tray(d, b, "exif_gps.jpg"),
    "d6b_tray_xmp": lambda d, b: add_tray(d, b, "xmp.jpg"),
    "d6c_tray_exif_webp": lambda d, b: add_tray(d, b, "exif.webp"),
    "d7_tray_fake_html": lambda d, b: add_tray(d, b, "fake.jpg"),
    "d8_shared_tray": lambda d, b: (add_tray(d, b, "clean.jpg"),
                                    d["persons"][0]["memorial"].__setitem__("tray_photo_id", TRAY)),
    "d8b_tray_is_other_portrait": lambda d, b: d["persons"][1]["memorial"].__setitem__("tray_photo_id", d["persons"][0]["photo_id"]),
    "e5_explicit_null": lambda d, b: d["persons"][1]["memorial"].__setitem__("tray_photo_id", None),
}
files = {k: craft(k, f) for k, f in CR.items()}
for s in SIDES:
    d = R["d"][s]
    for k, f in files.items():
        for mode in ("--replace", "--merge"):
            tgt = Store(s, f"cr-{k}{mode}")
            tgt.open()
            before = store_view(tgt.open())
            j = tgt.imp(f, mode)
            after = tgt.open()
            v = store_view(after)
            d[f"{k}{mode}"] = {"verdict": verdict(j), "tray": v["trays"].get(D2), "tray_d1": v["trays"].get(D1),
                               "sha_eq": v["photo_sha"].get(TRAY) == MAN["clean.jpg"]["sha256"] if k == "d9_control" else None,
                               "store_unchanged_if_rejected": (before == v) if not j.get("ok") else None,
                               "orphan_records": v["orphan_records"], "orphan_files": v["orphan_files"]}

# ================= (e) =================
EXF = os.path.join(TV, "example-v1.lvhv")
ex_js, _ = L.unpack_plain(L.decrypt(open(EXF, "rb").read(), PW))
ex_doc = json.loads(ex_js)
ex_cmp = L.canon({k: v for k, v in ex_doc.items() if k not in ("app", "exported_at")})
for s in SIDES:
    e = R.setdefault("e", {}).setdefault(s, {})
    for mode in ("--replace", "--merge"):
        t = Store(s, "e1" + mode); t.open()
        j = t.imp(EXF, mode)
        o = t.open()
        doc = o["store"]["document"]
        e["e1" + mode] = {"verdict": verdict(j), "doc_eq_file": L.canon(doc) == ex_cmp,
                          "diff_keys": sorted(k for k in set(doc) | set(ex_doc) - {"app", "exported_at"}
                                              if L.canon(doc.get(k)) != L.canon(ex_doc.get(k))),
                          "has_tray_key": "tray_photo_id" in json.dumps(doc),
                          "photo_sha_eq": {k: v["sha256"] == EXB_sha for k, v in o["store"]["photo_bytes"].items()
                                           for EXB_sha in [sha(EXB[k])] if k in EXB}}
        if mode == "--replace":
            out = os.path.join(W, f"e3-{s}.lvhv")
            e["e3_export"] = verdict(t.export(out))
            ins = tool(s, "inspect-export", out, PW)
            js2, bl2 = L.unpack_plain(L.decrypt(open(out, "rb").read(), PW))
            d2 = json.loads(js2)
            e["e3"] = {"no_tray_string": b"tray_photo_id" not in js2,
                       "doc_bytes_eq": L.canon({k: v for k, v in d2.items() if k not in ("app", "exported_at")}).encode()
                                       == ex_cmp.encode(),
                       "json_canonical": L.canon(d2).encode() == js2,
                       "blobs_eq": [(i, sha(x)) for i, x in bl2] == [(i, sha(x)) for i, x in ex_blobs],
                       "tool_inspect_ok": ins.get("ok"), "tool_legacy_paths": ins.get("legacy_paths")}
    # e2 — bộ mẫu đóng băng
    fx = os.path.join(TV, "released/0.2-dev-before-attachments")
    expd = json.load(open(os.path.join(TV, "released/expected/0.2-dev-before-attachments.json")))
    if s == "rs":
        dst = os.path.join(W, "stores", "e2-rs"); shutil.rmtree(dst, ignore_errors=True)
        shutil.copytree(os.path.join(fx, "store"), dst)
        o = tool("rs", "open", dst)
    else:
        o = tool("ts", "open", fx)
    e["e2"] = {"ok": o.get("ok"), "migration": (o.get("migration_status") or {}).get("state"),
               "state_sha_eq_expected": (o.get("store") or {}).get("state_sha256") == expd["state_sha256"],
               "has_tray_key": "tray_photo_id" in json.dumps((o.get("store") or {}).get("document"))}
# e4 — vector đã khóa không đổi
man = json.load(open(os.path.join(TV, "manifest.json")))
R["e"]["e4"] = {"file_sha": fsha(EXF), "manifest_sha": man["example"]["sha256"]}

json.dump(R, open(os.path.join(W, "results.json"), "w"), indent=1, ensure_ascii=False, sort_keys=True)
print("xong:", os.path.join(W, "results.json"))
