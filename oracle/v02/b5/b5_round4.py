# -*- coding: utf-8 -*-
"""Lan 4 (main 8835cd7): don `legacy`/.bak sau 7 ngay (gia lap thoi gian bang cach sua meta.migrated_at trong kho —
khoa nay duoc ghi o 01 §7.1), R6 tren R6v2 qua cong cu hop den (Rust + TS), mau giai ma doc lap, R6 cu bi tu choi.

Cach dung: python3 -I oracle/v02/b5/b5_round4.py <thu-muc-ban-dung> <thu-muc-nhap> <nhan>
"""
import copy, datetime as DT, hashlib, json, os, random, shutil, sqlite3, struct, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import b5_ref as R
import b5_r6v2 as D
import b5_statehash as S
import lvhvfile as L
from b5_tool import Tool

HERE = os.path.dirname(os.path.abspath(__file__))
build, scratch, tag = sys.argv[1:4]
PW = "mật khẩu thử nghiệm"
TV = os.path.join(build, "docs/v0.2/schema/test-vectors")
R6V2 = os.path.join(TV, "r6v2/dataset-r6v2-fixed.lvhv")
WANT_STATE = "9f4f7bb9f47d1a497decde0d1a53ec40f0aebddf9a00c20796a60599e1fdf36e"   # 03 §B1.6
TOOLS = {"rust": Tool(build, "rust"), "ts": Tool(build, "ts")}
RES = {"tag": tag}
os.makedirs(scratch, exist_ok=True)


def meta(db):
    c = sqlite3.connect(db)
    m = dict(c.execute("select key, value from meta").fetchall())
    c.close()
    return m


# ------------------------------------------------ 1. don sau 7 ngay (Rust)
prune = {}
for days in (6, 8):
    d = os.path.join(scratch, "prune-%d" % days)
    shutil.rmtree(d, ignore_errors=True)
    shutil.copytree(os.path.join(TV, "released/0.2-dev-before-attachments/store"), d)
    r1 = TOOLS["rust"].run("open", d)
    doc1 = r1["store"]["document"]
    m1 = meta(os.path.join(d, "lvhv.sqlite"))
    row = {"after_migration": {"state": r1["migration_status"]["state"], "meta": m1,
                               "persons_with_legacy": sum(1 for p in doc1["persons"] if "legacy" in p),
                               "bak": os.path.exists(os.path.join(d, "lvhv.sqlite.pre-v2.bak"))}}
    ma = m1.get("migrated_at")
    if ma:
        t = DT.datetime.strptime(ma, "%Y-%m-%dT%H:%M:%SZ") - DT.timedelta(days=days)
        c = sqlite3.connect(os.path.join(d, "lvhv.sqlite"))
        c.execute("update meta set value=? where key='migrated_at'", (t.strftime("%Y-%m-%dT%H:%M:%SZ"),))
        c.commit(); c.close()
        r2 = TOOLS["rust"].run("open", d)
        doc2 = r2["store"]["document"]
        m2 = meta(os.path.join(d, "lvhv.sqlite"))
        revs1 = {p["id"]: (p["rev"], p["updated_at"]) for p in doc1["persons"]}
        revs2 = {p["id"]: (p["rev"], p["updated_at"]) for p in doc2["persons"]}
        strip = lambda doc: [{k: v for k, v in p.items() if k != "legacy"} for p in doc["persons"]]
        row["after_reopen"] = {"state": r2["migration_status"]["state"], "meta": m2,
                               "persons_with_legacy": sum(1 for p in doc2["persons"] if "legacy" in p),
                               "bak": os.path.exists(os.path.join(d, "lvhv.sqlite.pre-v2.bak")),
                               "rev_updated_unchanged": revs1 == revs2,
                               "other_fields_unchanged": strip(doc1) == strip(doc2),
                               "attachments_unchanged": doc1.get("attachments") == doc2.get("attachments")}
        if days == 8:
            ok = (row["after_reopen"]["persons_with_legacy"] == 0 and not row["after_reopen"]["bak"]
                  and "legacy_pruned_at" in m2 and row["after_reopen"]["rev_updated_unchanged"]
                  and row["after_reopen"]["other_fields_unchanged"] and row["after_reopen"]["attachments_unchanged"])
        else:
            ok = (row["after_reopen"]["persons_with_legacy"] == row["after_migration"]["persons_with_legacy"] > 0
                  and row["after_reopen"]["bak"] and "legacy_pruned_at" not in m2)
        row["verdict"] = "KHOP" if ok else "SAI_LECH"
    else:
        row["verdict"] = "KHONG_CO_migrated_at"
    prune["%d_ngay" % days] = row
# ca loi: khong ghi migrated_at, khong bao gio don
d = os.path.join(scratch, "prune-fail")
shutil.rmtree(d, ignore_errors=True)
shutil.copytree(os.path.join(TV, "released/0.2-dev-before-attachments/store"), d)
rf = TOOLS["rust"].run("open", d, "--force-migration-failure")
prune["loi"] = {"state": rf["migration_status"]["state"], "meta": meta(os.path.join(d, "lvhv.sqlite")),
                "bak": os.path.exists(os.path.join(d, "lvhv.sqlite.pre-v2.bak"))}
prune["loi"]["verdict"] = "KHOP" if ("migrated_at" not in prune["loi"]["meta"] and prune["loi"]["bak"]) else "SAI_LECH"
RES["prune_rust"] = prune

# ------------------------------------------------ 2. R6v2 (a): nhap tep co dinh, so voi tai lieu giai ma doc lap
rep, doc, blocks = D.decrypt_report(R6V2, PW, os.path.join(TV, "r6v2/manifest.json"), os.path.join(TV, "r6v2"))
RES["r6v2_independent_decrypt"] = rep
r6a = {}
for tk, t in TOOLS.items():
    for mode in ("--replace", "--merge"):
        r = t.run("import", R6V2, PW, mode)
        row = {"ok": r.get("ok")}
        if r.get("ok"):
            st = r["store"]
            ne, _ = R.norm_for_compare(doc); ng, _ = R.norm_for_compare(st["document"])
            for x in (ne, ng):
                x.pop("schema_version", None)
                x.pop("settings", None) if False else None
            dif = R.diff(ne, ng)
            pb = st.get("photo_bytes") or {}
            row.update(state_sha256=st.get("state_sha256"), state_ok=st.get("state_sha256") == WANT_STATE,
                       indep_state_of_tool_doc=S.state_hash(st["document"], dict(blocks)),
                       n_diffs=len(dif), diffs=[(p, str(a)[:80], str(b)[:80]) for p, a, b in dif[:8]],
                       photo_bytes_ok=sorted(m.get("sha256") for m in pb.values()) == sorted(hashlib.sha256(b).hexdigest() for _, b in blocks))
        else:
            row["error"] = r.get("error")
        r6a["%s%s" % (tk, mode)] = row
RES["r6a_import"] = r6a

# ------------------------------------------------ 3. R6 (a2): 20 mat khau
import unicodedata
wrong = ["", "mật khẩu thử nghiệ", "mật khẩu thử nghiệmm", "Mật khẩu thử nghiệm", "mật khẩu thử nghiệm ",
         " mật khẩu thử nghiệm", "mat khau thu nghiem", "mật khẩu  thử nghiệm", "mật khẩu thử nghiêm",
         "mật khẩu thử nghiệm", "MẬT KHẨU THỬ NGHIỆM", "mật khẩu thử nghiệm\n", "12345678", "mật khẩu thử",
         "mật khẩu thử nghiệm!", "mật khẩu thử nghiệm​", "mậtkhẩuthửnghiệm", "nghiệm thử khẩu mật",
         "mật khẩu thử nghiệm\t"]
right_nfd = unicodedata.normalize("NFD", PW)
assert right_nfd != PW
a2 = {}
for tk, t in TOOLS.items():
    acc = []
    for w in wrong:
        r = t.run("import", R6V2, w, "--replace")
        if r.get("ok"):
            acc.append(repr(w))
    rn = t.run("import", R6V2, right_nfd, "--replace")
    a2[tk] = {"wrong_tested": len(wrong), "wrong_accepted": acc, "nfd_of_right_accepted": bool(rn.get("ok")),
              "nfd_state_ok": (rn.get("store") or {}).get("state_sha256") == WANT_STATE}
RES["r6a2"] = a2

# ------------------------------------------------ 4. R6 (b) mau qua cong cu
data = open(R6V2, "rb").read()
n = len(data)
rnd = random.Random(20261107)
pos = sorted(set([rnd.randrange(0, 4096) for _ in range(40)] + [rnd.randrange(n - 4096, n) for _ in range(40)]
                 + list(range(0, 72, 3))))
bsamp = {}
fdir = os.path.join(scratch, "flips"); os.makedirs(fdir, exist_ok=True)
for tk, t in TOOLS.items():
    acc, cnt = [], 0
    for i in pos:
        bit = rnd.randrange(8)
        v = bytearray(data); v[i] ^= 1 << bit
        p = os.path.join(fdir, "f.lvhv"); open(p, "wb").write(v)
        r = t.run("import", p, PW, "--replace")
        cnt += 1
        if r.get("ok"):
            acc.append((i, bit))
    bsamp[tk] = {"variants": cnt, "accepted": acc}
RES["r6b_tool_sample"] = bsamp

# ------------------------------------------------ 5. R6 (d): 10 tep mau do ban xay dung tao
samples = [os.path.join(TV, "example-v1.lvhv"), os.path.join(TV, "supplement-yc05/notice-v1.lvhv"),
           os.path.join(TV, "released/0.2-dev-before-attachments/backup.lvhv"), R6V2]
out = os.path.join(build, "pwa/test/store/out")
for k in range(6):
    p = subprocess.run(["./target/release/examples/cross_platform", "write"], cwd=build, capture_output=True, text=True, timeout=600)
    src = os.path.join(out, "r6-from-rust.lvhv")
    if os.path.exists(src):
        dst = os.path.join(scratch, "rust-export-%d.lvhv" % k); shutil.copy(src, dst); samples.append(dst)
dd = []
for s in samples:
    try:
        rp, dc, bl = D.decrypt_report(s, PW)
        rp["ok"] = all(rp[k] for k in ("two_decryptors_same_plaintext", "plaintext_len_matches_header", "json_canonical",
                                       "block_order_matches_records", "blocks_sha256_and_len_match_records")) and rp["images_ok"] is not False
        r = TOOLS["rust"].run("import", s, PW, "--replace")
        rp["tool_state_equals_independent"] = (r.get("store") or {}).get("state_sha256") == rp["independent_state_sha256"]
        rp["tool_ok"] = r.get("ok")
        rp["tool_error"] = None if r.get("ok") else r.get("error")
    except Exception as ex:
        rp = {"file": os.path.basename(s), "ok": False, "error": repr(ex)}
    dd.append(rp)
RES["r6d_samples"] = dd
salts = {open(s, "rb").read()[24:40].hex() for s in samples}
RES["r6d_distinct_salts"] = len(salts)

# ------------------------------------------------ 6. R6 cu bi tu choi
# Tep R6 cu co dinh khong nam trong kho (chi co sha256 trong manifest.json). Dung lai bang bo ma hoa doc lap:
# dataset-r6.json + 20 "anh" tong hop dung dang 03 §B1.6 mo ta (FF D8 FF E0 ... FF D9) voi do dai va sha256 ghi lai
# vao ban ghi (de loi chi con co the den tu kiem tra anh B6).
old = json.load(open(os.path.join(TV, "dataset-r6.json"), encoding="utf-8"))
bl = []
for i, ph in enumerate(old["photos"]):
    ln = ph["bytes"]
    b = b"\xff\xd8\xff\xe0" + bytes([(i * 7 + k) % 251 for k in range(max(0, ln - 6))]) + b"\xff\xd9"
    ph["sha256"] = hashlib.sha256(b).hexdigest(); ph["bytes"] = len(b)
    bl.append((ph["id"], b))
old["photos"].sort(key=lambda p: p["id"])
bl.sort()
oldp = os.path.join(scratch, "old-r6-reconstructed.lvhv")
open(oldp, "wb").write(L.encrypt_file(L.pack(L.canonical_json(old), bl), PW, bytes(16), bytes(24)))
oldres = {}
for tk, t in TOOLS.items():
    r = t.run("import", oldp, PW, "--replace")
    oldres[tk] = {"ok": r.get("ok"), "error": r.get("error"), "stage": r.get("stage")}
RES["old_r6_rejected"] = oldres

json.dump(RES, open(os.path.join(HERE, "../out/b5_r4_%s.json" % tag), "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print(json.dumps({k: RES[k] for k in ("prune_rust", "r6a_import", "r6a2", "r6b_tool_sample", "r6d_distinct_salts", "old_r6_rejected")},
                 ensure_ascii=False, indent=1)[:9000])
for x in dd:
    print("R6d", x.get("file"), x.get("ok"), x.get("tool_ok"), x.get("tool_state_equals_independent"), x.get("counts"), x.get("error"))
