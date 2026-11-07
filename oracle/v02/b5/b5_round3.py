# -*- coding: utf-8 -*-
"""Lan 3: doi chieu bang cong cu hop den lvhv-store-tool (docs/v0.2/10), ban Rust va ban TS.

Cach dung: python3 -I oracle/v02/b5/b5_round3.py <thu-muc-ban-dung> <thu-muc-tep-da-chuyen-dang> <thu-muc-nhap> <nhan>
Ghi oracle/v02/out/b5_r3_<nhan>.json. Ky vong lay tu b5_ref (commit d14f452; M10 them o b9d8b07).

So sanh "0 truong khac": tai lieu kho in ra (store.document) va ky vong cua tham chieu, ca hai qua
b5_ref.norm_for_compare (bo schema_version, vang == null cho gioi_tinh/notice/legacy, tham chieu tep dinh kem
so theo (kind, ban ghi tep tru id), legacy tach rieng). legacy: moi cap (khoa, gia tri) ky vong phai co mat;
phan legacy them cua ben xay dung chi duoc la certificate_photo_ids / certificate_photos (01 §7.1 tren PR #28),
va gia tri phai dung voi tep cu.
"""
import copy, hashlib, json, os, shutil, sqlite3, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import b5_ref as R
import lvhvfile as L
from b5_tool import Tool

HERE = os.path.dirname(os.path.abspath(__file__))
build, adapted, scratch, tag = sys.argv[1:5]
PW = "mật khẩu thử nghiệm"
MAN = json.load(open(os.path.join(HERE, "fixtures/manifest.json"), encoding="utf-8"))
EXP = {e["name"]: e for e in json.load(open(os.path.join(HERE, "../out/b5_expected.json"), encoding="utf-8"))["fixtures"]}
FROZEN = os.path.join(build, "docs/v0.2/schema/test-vectors/released/0.2-dev-before-attachments")
TOOLS = {"rust": Tool(build, "rust"), "ts": Tool(build, "ts")}
sha = lambda b: hashlib.sha256(b).hexdigest()
RES = {"tag": tag, "import": {}, "inplace": {}, "names": {}, "suggestions": {}, "display": {}, "media": {}, "frozen": {}}


def load_fixture(name, use_adapted):
    p = os.path.join(adapted if use_adapted else os.path.join(HERE, "fixtures"), name + ".lvhv")
    if not os.path.exists(p):
        return None, None, None
    j, bl = L.unpack(L.decrypt_file(open(p, "rb").read(), PW))
    return p, json.loads(j), dict(bl)


def expected_doc(name, doc, blobs):
    if doc.get("schema_version") == 1 and name.startswith("M"):
        return R.migrate_1_to_2(doc, blobs)
    return doc, blobs


ALLOWED_EXTRA_LEGACY = {"certificate_photo_ids", "certificate_photos"}


def compare_doc(old, exp, eblobs, got, got_bytes):
    """Tra (ban_ghi_mat, khac_truong, legacy_van_de, byte_van_de)."""
    ne, le = R.norm_for_compare(exp)
    ng, lg = R.norm_for_compare(got)
    for d in (ne, ng):
        d.pop("schema_version", None)
        if d.get("attachments") == []:
            del d["attachments"]
    lost = []
    for k in ("persons", "events", "event_types", "photos"):
        ids_e = {x.get("id") for x in exp.get(k, [])}
        ids_g = {x.get("id") for x in got.get(k, [])}
        lost += ["%s/%s" % (k, i) for i in sorted(ids_e - ids_g)]
    se = sorted(a["sha256"] for a in exp.get("attachments", []))
    sg = sorted(a["sha256"] for a in got.get("attachments", []))
    if se != sg:
        lost.append("attachments sha256 %s -> %s" % (len(se), len(sg)))
    diffs = [(p, str(a)[:120], str(b)[:120]) for p, a, b in R.diff(ne, ng)]
    # legacy
    leg = []
    exp_pairs = set(R.legacy_values(exp))
    got_pairs = set(R.legacy_values(got))
    for kv in sorted(exp_pairs - got_pairs):
        leg.append("thieu legacy %s" % kv[0])
    old_photos = {p["id"]: p for p in old.get("photos", [])}
    for path, lv in lg.items():
        for k, v in lv.items():
            if (k, json.dumps(v, sort_keys=True, ensure_ascii=False)) in exp_pairs:
                continue
            if k not in ALLOWED_EXTRA_LEGACY:
                leg.append("legacy la %s%s" % (path, k))
            elif k == "certificate_photos":
                for ph in v:
                    if old_photos.get(ph.get("id")) != ph:
                        leg.append("certificate_photos khac ban cu %s" % ph.get("id"))
    # byte
    bad = []
    for kind in ("photo_bytes", "attachment_bytes"):
        for i, m in (got_bytes.get(kind) or {}).items():
            if "error" in m:
                bad.append("%s %s %s" % (kind, i, m["error"]))
    want = sorted(sha(eblobs[a["id"]]) for a in exp.get("attachments", []) if a["id"] in eblobs)
    have = sorted(m.get("sha256") for m in (got_bytes.get("attachment_bytes") or {}).values())
    if want != have:
        bad.append("byte tep dinh kem khac: %d vs %d" % (len(want), len(have)))
    return lost, diffs, leg, bad


def verdict_reject(e, err):
    s = json.dumps(err, ensure_ascii=False)
    newer = "NewerSchema" in s or "newer_schema" in s
    return ("NEWER" in e["class"]) == newer


# ---------------------------------------------------------------- 1. nhap sao luu
for f in MAN["fixtures"]:
    name = f["name"]
    e = EXP[name]
    use_ad = name[0] in "GXU"
    p, doc, blobs = load_fixture(name, use_ad)
    if p is None and f["big"]:
        p, doc, blobs = load_fixture(name, True)
    if p is None:
        continue
    for tk, t in TOOLS.items():
        r = t.run("import", p, PW, "--replace")
        row = {"ok": r.get("ok"), "exit": r["_exit"]}
        if not r.get("ok"):
            row["error"] = {k: r.get("error", {}).get(k) for k in ("kind", "code", "path", "msg", "version", "backup_error")} if isinstance(r.get("error"), dict) else r.get("error")
            row["stage"] = r.get("stage")
        if e["expect"] in ("accept", "accept_or_chua_ro", "chua_ro"):
            if r.get("ok"):
                try:
                    ed, eb = expected_doc(name, doc, blobs)
                    lost, diffs, leg, bad = compare_doc(doc, ed, eb, r["store"]["document"], r["store"])
                except KeyError as ex:   # tham chieu tu khong chuyen doi duoc (vd id treo)
                    lost, diffs, leg, bad = ["tham-chieu-loi %s" % ex], [], [], []
                row.update(lost=lost, diffs=diffs[:12], n_diffs=len(diffs), legacy=leg, bytes=bad)
                clean = not (lost or diffs or leg or bad)
                row["verdict"] = ("KHOP" if clean else "SAI_LECH_TRUONG") if e["expect"] == "accept" else ("GHI_NHAN" + ("" if clean else "_KHAC"))
            else:
                row["verdict"] = "SAI_LECH_TU_CHOI" if e["expect"] == "accept" else "GHI_NHAN_TU_CHOI"
        else:
            row["verdict"] = "SAI_LECH_CHAP_NHAN" if r.get("ok") else ("KHOP" if verdict_reject(e, r.get("error")) else "SAI_LOP")
        RES["import"].setdefault(name, {})[tk] = row


# ---------------------------------------------------------------- 2. chuyen doi tai cho
def db_rows(path):
    c = sqlite3.connect("file:%s?mode=ro" % path, uri=True)
    out = {"user_version": c.execute("pragma user_version").fetchone()[0]}
    for (t,) in c.execute("select name from sqlite_master where type='table' order by name"):
        out[t] = sorted(map(list, c.execute("select * from %s" % t).fetchall()), key=str)
    c.close()
    return out


def ts_layout(doc, blobs, d):
    os.makedirs(os.path.join(d, "store/photos"), exist_ok=True)
    json.dump(doc, open(os.path.join(d, "store-snapshot.json"), "w", encoding="utf-8"), ensure_ascii=False)
    for ph in doc["photos"]:
        open(os.path.join(d, "store/photos", ph["id"] + ".img"), "wb").write(blobs[ph["id"]])


tmpl = os.path.join(FROZEN, "store/lvhv.sqlite")
cases = [("FROZEN", None)] + [(f["name"], f) for f in MAN["fixtures"] if f["name"][0] == "M"]
for name, f in cases:
    if name == "FROZEN":
        snap = json.load(open(os.path.join(FROZEN, "store-snapshot.json"), encoding="utf-8"))
        old = snap
        oblobs = {fn[:-4]: open(os.path.join(FROZEN, "store/photos", fn), "rb").read() for fn in os.listdir(os.path.join(FROZEN, "store/photos"))}
        src_rust = os.path.join(FROZEN, "store")
        src_ts = FROZEN
    else:
        old = json.load(open(os.path.join(HERE, "fixtures", name + ".json"), encoding="utf-8"))
        oblobs = {i: open(os.path.join(HERE, "fixtures/bytes", m["sha256"] + ".bin"), "rb").read() for i, m in f["blobs"].items()}
        src_rust = os.path.join(scratch, "mk", name)
        shutil.rmtree(src_rust, ignore_errors=True)
        subprocess.run([sys.executable, "-I", os.path.join(HERE, "b5_mkstore.py"), tmpl, os.path.join(HERE, "fixtures", name + ".json"),
                        os.path.join(HERE, "fixtures/bytes"), json.dumps(f["blobs"]), src_rust], check=True, capture_output=True)
        src_ts = os.path.join(scratch, "mkts", name)
        shutil.rmtree(src_ts, ignore_errors=True)
        ts_layout(old, oblobs, src_ts)
    row = {}
    for force in (False, True):
        tagf = "loi" if force else "thuong"
        # Rust
        d = os.path.join(scratch, "inplace", name + "-" + tagf)
        shutil.rmtree(d, ignore_errors=True)
        shutil.copytree(src_rust, d)
        orig_db = open(os.path.join(d, "lvhv.sqlite"), "rb").read()
        orig_rows = db_rows(os.path.join(d, "lvhv.sqlite"))
        args = ["open", d] + (["--force-migration-failure"] if force else [])
        r = TOOLS["rust"].run(*args)
        rr = {"ok": r.get("ok"), "migration_status": r.get("migration_status")}
        bak = os.path.join(d, "lvhv.sqlite.pre-v2.bak")
        rr["bak_exists"] = os.path.exists(bak)
        if rr["bak_exists"]:
            bb = open(bak, "rb").read()
            rr["bak_byte_identical"] = bb == orig_db
            try:
                rr["bak_rows_identical"] = db_rows(bak) == orig_rows
            except Exception as ex:
                rr["bak_rows_identical"] = "loi %s" % ex
        rr["photos_unchanged"] = all(open(os.path.join(d, "photos", fn), "rb").read() == open(os.path.join(src_rust, "photos", fn), "rb").read()
                                     for fn in os.listdir(os.path.join(src_rust, "photos")))
        rr["files_after"] = sorted((r.get("files_after") or {}).keys())
        if force:
            now = db_rows(os.path.join(d, "lvhv.sqlite"))
            rr["user_version_after"] = now["user_version"]
            rr["db_rows_unchanged"] = now == orig_rows
            rr["read_only_probe"] = r.get("read_only_probe")
            rr["orphan_attachment_files"] = [x for x in rr["files_after"] if x.startswith("attachments/")]
            st = (r.get("migration_status") or {}).get("state")
            ro = r.get("read_only_probe") or {}
            rr["verdict"] = "KHOP" if (st == "read_only" and rr["user_version_after"] == 1 and rr["db_rows_unchanged"] and rr["bak_exists"]) else "SAI_LECH"
        elif r.get("ok"):
            try:
                ed, eb = R.migrate_1_to_2(old, oblobs)
                lost, diffs, leg, bad = compare_doc(old, ed, eb, r["store"]["document"], r["store"])
            except KeyError as ex:
                lost, diffs, leg, bad = ["tham-chieu-loi %s" % ex], [], [], []
            rr.update(lost=lost, diffs=diffs[:12], n_diffs=len(diffs), legacy=leg, bytes=bad, user_version=r["store"].get("user_version"),
                      state_sha256=r["store"].get("state_sha256"))
            rr["verdict"] = "KHOP" if not (lost or diffs or leg or bad) else "SAI_LECH_TRUONG"
        else:
            rr["error"] = r.get("error")
            rr["verdict"] = "TU_CHOI"
        row["rust-" + tagf] = rr
        # TS
        r = TOOLS["ts"].run(*(["open", src_ts] + (["--force-migration-failure"] if force else [])))
        tr = {"ok": r.get("ok"), "migration_status": r.get("migration_status")}
        if force:
            st = (r.get("migration_status") or {}).get("state")
            tr["read_only_probe"] = r.get("read_only_probe")
            docg = (r.get("store") or {}).get("document")
            tr["doc_equals_old"] = None if docg is None else (R.norm_for_compare(docg)[0] == R.norm_for_compare({k: v for k, v in old.items()})[0])
            tr["verdict"] = "KHOP" if st == "read_only" else "SAI_LECH"
        elif r.get("ok"):
            try:
                ed, eb = R.migrate_1_to_2(old, oblobs)
                lost, diffs, leg, bad = compare_doc(old, ed, eb, r["store"]["document"], r["store"])
            except KeyError as ex:
                lost, diffs, leg, bad = ["tham-chieu-loi %s" % ex], [], [], []
            tr.update(lost=lost, diffs=diffs[:12], n_diffs=len(diffs), legacy=leg, bytes=bad, state_sha256=r["store"].get("state_sha256"))
            tr["verdict"] = "KHOP" if not (lost or diffs or leg or bad) else "SAI_LECH_TRUONG"
        else:
            tr["error"] = r.get("error")
            tr["verdict"] = "TU_CHOI"
        row["ts-" + tagf] = tr
    RES["inplace"][name] = row

# ---------------------------------------------------------------- 3. viet hoa khi luu / khi hien thi
exp_all = json.load(open(os.path.join(HERE, "../out/b5_expected.json"), encoding="utf-8"))
base = json.load(open(os.path.join(build, "docs/v0.2/schema/test-vectors/example-v1.json"), encoding="utf-8"))
living = copy.deepcopy([p for p in base["persons"] if p["kind"] == "living"][0])
for i, c in enumerate(exp_all["names_on_save"]):
    s = c["input"]
    if len(R.nfc(s).strip()) > 40:
        continue
    for tk, t in TOOLS.items():
        p = copy.deepcopy(living); p["id"] = ""; p["danh_xung"] = s; p["quan_he"] = s
        for k in ("created_at", "updated_at"):
            p.pop(k, None)
        d = os.path.join(scratch, "save", "%s-%d" % (tk, i))
        shutil.rmtree(d, ignore_errors=True); os.makedirs(d)
        r = t.run("save-person", d if tk == "rust" else "-", "-", stdin=json.dumps(p, ensure_ascii=False))
        saved = r.get("saved") or {}
        disp = t.run("display-capitalize", s)
        dv = disp.get("result", disp.get("output", disp.get("value")))
        RES["names"].setdefault(repr(s), {})[tk] = {
            "ok": r.get("ok"), "danh_xung": saved.get("danh_xung"), "quan_he": saved.get("quan_he"),
            "ho_ten_unchanged": saved.get("ho_ten") == living["ho_ten"],
            "expected": c["expected"], "alt": c["alt_first_letter"], "ambiguous": c["ambiguous"],
            "save_match": saved.get("danh_xung") == c["expected"] and saved.get("quan_he") == c["expected"],
            "display": dv, "display_raw": {k: v for k, v in disp.items() if k not in ("_exit",)},
            "display_match": dv == c["expected"], "error": r.get("error")}

# ---------------------------------------------------------------- 4. goi y
for tk, t in TOOLS.items():
    for who, lst in (("deceased", exp_all["goi_y"]["deceased"]), ("living", exp_all["goi_y"]["living"])):
        for g in (None, "nam", "nu"):
            r = t.run("suggestions", who, *([g] if g else []))
            got = r.get("danh_xung") or []
            RES["suggestions"]["%s/%s/%s" % (tk, who, g)] = {
                "same_set": sorted(got) == sorted(lst), "same_order": got == lst, "n": len(got),
                "missing": [x for x in lst if x not in got], "extra": [x for x in got if x not in lst], "got": got}

# ---------------------------------------------------------------- 5. check-media
pdf = open(os.path.join(HERE, "fixtures/bytes", [m for m in MAN["fixtures"] if m["name"] == "G01-v2-hop-le"][0]["blobs"]["b5000000-0000-4000-8000-0000000000f1"]["sha256"] + ".bin"), "rb").read()
mp = os.path.join(scratch, "media"); os.makedirs(mp, exist_ok=True)
open(os.path.join(mp, "pdf.pdf"), "wb").write(pdf)
for tk, t in TOOLS.items():
    RES["media"][tk] = {"X21 attachment image/jpeg <- PDF": t.run("check-media", "attachment", "image/jpeg", os.path.join(mp, "pdf.pdf")).get("verdict"),
                        "PDF attachment application/pdf": t.run("check-media", "attachment", "application/pdf", os.path.join(mp, "pdf.pdf")).get("verdict")}

json.dump(RES, open(os.path.join(HERE, "../out/b5_r3_%s.json" % tag), "w", encoding="utf-8"), ensure_ascii=False, indent=1)
# tom tat
for name, v in RES["import"].items():
    print("IMPORT", name.ljust(34), " ".join("%s=%s" % (k, x["verdict"]) for k, x in v.items()),
          "|", (v["rust"].get("error") or {}).get("msg") if isinstance(v["rust"].get("error"), dict) else "")
for name, v in RES["inplace"].items():
    print("INPLACE", name.ljust(30), " ".join("%s=%s" % (k, x["verdict"]) for k, x in v.items()))
for s, v in RES["names"].items():
    print("NAME", s.ljust(26), " ".join("%s:save=%s disp=%s" % (k, x["save_match"], x["display_match"]) for k, x in v.items()))
for k, v in RES["suggestions"].items():
    print("SUG", k, v["same_set"], v["same_order"], v["n"])
print("MEDIA", RES["media"])
