# -*- coding: utf-8 -*-
"""Lan 5 (main 7056353): don sau 7 ngay voi --now (Rust + TS qua --state) va tep xuat khong mang `legacy`
(kiem bang bo giai ma doc lap b5_r6v2 va bang inspect-export).

Cach dung: python3 -I oracle/v02/b5/b5_round5.py <thu-muc-ban-dung> <thu-muc-nhap> <nhan>
"""
import json, os, shutil, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import b5_ref as R
import b5_r6v2 as D
import b5_statehash as S
from b5_tool import Tool

HERE = os.path.dirname(os.path.abspath(__file__))
build, scratch, tag = sys.argv[1:4]
PW = "mật khẩu thử nghiệm"
DAY = 86400
T0 = 1793000000
TV = os.path.join(build, "docs/v0.2/schema/test-vectors")
FROZEN = os.path.join(TV, "released/0.2-dev-before-attachments")
MAN = json.load(open(os.path.join(HERE, "fixtures/manifest.json"), encoding="utf-8"))
TOOLS = {"rust": Tool(build, "rust"), "ts": Tool(build, "ts")}
os.makedirs(scratch, exist_ok=True)
RES = {"tag": tag, "T0": T0, "cases": {}}


def legacy_paths(o, p=""):
    out = []
    if isinstance(o, dict):
        for k, v in o.items():
            if k == "legacy":
                out.append(p + "/legacy")
            out += legacy_paths(v, p + "/" + k)
    elif isinstance(o, list):
        for i, v in enumerate(o):
            out += legacy_paths(v, p + "/" + str(i))
    return out


def strip_legacy(o):
    if isinstance(o, dict):
        return {k: strip_legacy(v) for k, v in o.items() if k != "legacy"}
    if isinstance(o, list):
        return [strip_legacy(v) for v in o]
    return o


def n_legacy(doc):
    return sum(1 for p in doc.get("persons", []) if "legacy" in p)


def check_export(tk, src_args, now, outf, store_doc):
    t = TOOLS[tk]
    r = t.run("export", *src_args[:1], PW, outf, "--now", str(now), *src_args[1:])
    row = {"ok": r.get("ok"), "error": r.get("error")}
    if not r.get("ok") or not os.path.exists(outf):
        return row
    data = open(outf, "rb").read()
    pt, _ = D.decrypt2(data, PW)                      # bo giai ma doc lap (argon2-cffi + PyNaCl)
    j, blocks = D.unpack_strict(pt)
    doc = json.loads(j)
    row["indep_legacy_paths"] = legacy_paths(doc)
    row["indep_json_canonical"] = json.dumps(doc, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode() == j
    recs = {x["id"]: x for x in doc.get("photos", []) + doc.get("attachments", [])}
    import hashlib
    row["indep_blocks_ok"] = all(u in recs and recs[u]["sha256"] == hashlib.sha256(b).hexdigest() and recs[u]["bytes"] == len(b)
                                 for u, b in blocks) and len(blocks) == len(recs)
    a = R.norm_for_compare(strip_legacy(store_doc))[0]; b = R.norm_for_compare(doc)[0]
    for x in (a, b):
        x.pop("schema_version", None)
        x.pop("settings", None)          # xuat co the ghi last_export_at o meta, khong o settings; so settings rieng
    row["indep_doc_equals_store_minus_legacy"] = R.diff(a, b) == []
    row["indep_diffs"] = [(p, str(x)[:80], str(y)[:80]) for p, x, y in R.diff(a, b)[:6]]
    row["settings_equal"] = strip_legacy(store_doc).get("settings") == doc.get("settings")
    for tk2, t2 in TOOLS.items():
        ie = t2.run("inspect-export", outf, PW)
        row["inspect_%s_legacy_paths" % tk2] = ie.get("legacy_paths")
        row["inspect_%s_same_doc" % tk2] = ie.get("document") == doc
        im = t2.run("import", outf, PW, "--replace")
        row["import_%s_ok" % tk2] = im.get("ok")
        row["import_%s_state_equals_indep" % tk2] = (im.get("store") or {}).get("state_sha256") == S.state_hash(doc, dict(blocks))
    return row


def run_case(name, rust_src, ts_src):
    out = {}
    for tk in ("rust", "ts"):
        t = TOOLS[tk]
        steps = {}
        if tk == "rust":
            d = os.path.join(scratch, "%s-rust" % name); shutil.rmtree(d, ignore_errors=True); shutil.copytree(rust_src, d)
            first = [d]; again = [d]; exp_src = [d]
            bak = os.path.join(d, "lvhv.sqlite.pre-v2.bak")
        else:
            st = os.path.join(scratch, "%s-ts.json" % name)
            if os.path.exists(st):
                os.remove(st)
            first = [ts_src, "--state", st]; again = ["-", "--state", st]; exp_src = ["-", "--state", st]
            bak = None
        def openat(args, now, label):
            r = t.run("open", *args, "--now", str(now))
            doc = (r.get("store") or {}).get("document") or {}
            steps[label] = {"ok": r.get("ok"), "state": (r.get("migration_status") or {}).get("state"),
                            "persons_with_legacy": n_legacy(doc), "bak": os.path.exists(bak) if bak else None,
                            "now_unix": r.get("now_unix"), "source": r.get("source")}
            return doc
        d0 = openat(first, T0, "T0")
        exp1 = check_export(tk, exp_src, T0 + DAY, os.path.join(scratch, "%s-%s-T1.lvhv" % (name, tk)),
                            openat(again, T0 + DAY, "T0+1d"))
        steps["export_T0+1d"] = exp1
        openat(again, T0 + 7 * DAY - 60, "T0+7d-60s")
        d8 = openat(again, T0 + 7 * DAY + 60, "T0+7d+60s")
        steps["prune_only_legacy"] = strip_legacy(d0.get("persons")) == d8.get("persons") and d0.get("attachments") == d8.get("attachments")
        openat(again, T0 + 8 * DAY, "T0+8d")
        steps["export_T0+8d"] = check_export(tk, exp_src, T0 + 8 * DAY, os.path.join(scratch, "%s-%s-T8.lvhv" % (name, tk)), d8)
        s = steps
        prune_ok = (s["T0"]["state"] == "migrated" and s["T0"]["persons_with_legacy"] > 0
                    and s["T0+1d"]["persons_with_legacy"] == s["T0"]["persons_with_legacy"]
                    and s["T0+7d-60s"]["persons_with_legacy"] == s["T0"]["persons_with_legacy"]
                    and s["T0+7d+60s"]["persons_with_legacy"] == 0 and s["T0+8d"]["persons_with_legacy"] == 0
                    and s["prune_only_legacy"])
        if tk == "rust":
            prune_ok = prune_ok and s["T0"]["bak"] and s["T0+1d"]["bak"] and s["T0+7d-60s"]["bak"] and not s["T0+7d+60s"]["bak"]
        exp_ok = all(e.get("ok") and e.get("indep_legacy_paths") == [] and e.get("inspect_rust_legacy_paths") == []
                     and e.get("inspect_ts_legacy_paths") == [] and e.get("indep_doc_equals_store_minus_legacy")
                     and e.get("indep_blocks_ok") and e.get("indep_json_canonical")
                     and e.get("import_rust_state_equals_indep") and e.get("import_ts_state_equals_indep")
                     for e in (s["export_T0+1d"], s["export_T0+8d"]))
        s["verdict_prune"] = "KHOP" if prune_ok else "SAI_LECH"
        s["verdict_export_no_legacy"] = "KHOP" if exp_ok else "SAI_LECH"
        out[tk] = s
    return out


cases = [("FROZEN", os.path.join(FROZEN, "store"), FROZEN)]
for f in MAN["fixtures"]:
    if f["name"] in ("M01-cu-giay-chung-tu", "M02-cu-4-anh", "M03-cu-anh-chung", "M10-cu-anh-dung-chung"):
        doc = json.load(open(os.path.join(HERE, "fixtures", f["name"] + ".json"), encoding="utf-8"))
        rs = os.path.join(scratch, "mk", f["name"]); shutil.rmtree(rs, ignore_errors=True)
        subprocess.run([sys.executable, "-I", os.path.join(HERE, "b5_mkstore.py"), os.path.join(FROZEN, "store/lvhv.sqlite"),
                        os.path.join(HERE, "fixtures", f["name"] + ".json"), os.path.join(HERE, "fixtures/bytes"),
                        json.dumps(f["blobs"]), rs], check=True, capture_output=True)
        ts = os.path.join(scratch, "mkts", f["name"]); shutil.rmtree(ts, ignore_errors=True)
        os.makedirs(os.path.join(ts, "store/photos"))
        json.dump(doc, open(os.path.join(ts, "store-snapshot.json"), "w", encoding="utf-8"), ensure_ascii=False)
        for ph in doc["photos"]:
            shutil.copy(os.path.join(HERE, "fixtures/bytes", f["blobs"][ph["id"]]["sha256"] + ".bin"), os.path.join(ts, "store/photos", ph["id"] + ".img"))
        cases.append((f["name"], rs, ts))
for name, rs, ts in cases:
    RES["cases"][name] = run_case(name, rs, ts)
    for tk, s in RES["cases"][name].items():
        print(name.ljust(26), tk.ljust(4), "prune=%s export=%s" % (s["verdict_prune"], s["verdict_export_no_legacy"]),
              {k: (v["persons_with_legacy"], v["bak"]) for k, v in s.items() if k.startswith("T0")})
json.dump(RES, open(os.path.join(HERE, "../out/b5_r5_%s.json" % tag), "w", encoding="utf-8"), ensure_ascii=False, indent=1)
