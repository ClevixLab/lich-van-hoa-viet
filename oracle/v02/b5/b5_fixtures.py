# -*- coding: utf-8 -*-
"""Sinh bo tep thu doi khang B5/R14 + YC-2026-07 va ket qua ky vong, TRUOC khi doi chieu.

Cach dung:  python3 -I oracle/v02/b5/b5_fixtures.py [--big DIR]
  - ghi oracle/v02/b5/fixtures/ (tai lieu JSON, bang muc nhi phan, byte, tep .lvhv nho)
  - ghi oracle/v02/out/b5_expected.json (+ out/b5_expected/<ten>.json cho ca duoc chap nhan)
  - --big DIR: sinh them 4 ca lon (PDF/anh dung bien va vuot bien) vao DIR (khong commit, ghi sha256)
Tat dinh: anh JPEG tao bang Pillow tu mau co dinh; salt/nonce co dinh (chi dung cho kiem thu, 03 §2.5).
Nguon du lieu goc: docs/v0.2/schema/test-vectors/example-v1.json (v1 hop le, truong dan xuat da dung).
"""
import copy, hashlib, io, json, os, sys, unicodedata

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import b5_ref as R
import lvhvfile as L

HERE = os.path.dirname(os.path.abspath(__file__))
FX = os.path.join(HERE, "fixtures")
OUT = os.path.join(HERE, "..", "out")
TV = os.path.join(R.ROOT, "docs", "v0.2", "schema", "test-vectors")
PW = "mật khẩu thử nghiệm"
SALT = bytes(range(0x40, 0x50))
NONCE = bytes(range(0x60, 0x78))


def U(n):
    return "b5000000-0000-4000-8000-%012x" % n


def jpeg(seed, pad_to=None):
    from PIL import Image
    w, h = 64 + seed % 7, 48 + seed % 5
    im = Image.new("RGB", (w, h))
    px = im.load()
    for x in range(w):
        for y in range(h):
            px[x, y] = ((x * 4 + seed * 37) % 256, (y * 5 + seed * 11) % 256, (seed * 53) % 256)
    bio = io.BytesIO()
    im.save(bio, "JPEG", quality=85)
    b = bio.getvalue()
    assert b"Exif" not in b[:200]
    if pad_to:
        assert len(b) <= pad_to
        b = b + b"\x00" * (pad_to - len(b))   # du lieu thua sau EOI: giai ma van duoc, chi de dat kich thuoc
    return b


def pdf(n_total=None, magic=b"%PDF-"):
    body = magic + b"1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj<</Type/Catalog>>endobj\ntrailer<</Root 1 0 R>>\n%%EOF\n"
    if n_total:
        body = body[:-6] + b"%" + b"0" * (n_total - len(body) - 1) + b"%%EOF\n"
        assert len(body) == n_total
    return body


def sha(b):
    return hashlib.sha256(b).hexdigest()


def photo_rec(pid, b, w=None, h=None, created="2026-11-06T03:15:00Z"):
    # Sua lan 3 (truoc khi chay lan 3): ban d14f452 ghi co dinh 64x48 trong khi anh that co kich thuoc khac
    # -> loi cua tep thu (01 §3.8 width/height la kich thuoc that), lo ra khi doc 04 Bo sung B6.
    if w is None or h is None:
        from PIL import Image
        w, h = Image.open(io.BytesIO(b)).size
    return {"id": pid, "mime": "image/jpeg", "width": w, "height": h, "bytes": len(b), "sha256": sha(b),
            "stripped": True, "created_at": created}


def att_rec(aid, b, mime, name=None, created="2026-11-07T04:00:00Z"):
    return {"id": aid, "mime": mime, "bytes": len(b), "sha256": sha(b), "stripped": mime != "application/pdf",
            "file_name": name, "created_at": created}


def notice(cert_ids=None, **kw):
    n = {"reported_by": "Chị Lê Thị C", "place": "Bệnh viện đa khoa tỉnh", "reason": "Tuổi cao",
         "notice_date": {"calendar": "solar", "precision": "day", "year": 2025, "month": 8, "day": 8, "leap": False},
         "certificate_photo_ids": list(cert_ids or []), "note": "Gia đình đã báo họ hàng.\nAn táng ở quê."}
    n.update(kw)
    return n


def base_v1():
    with open(os.path.join(TV, "example-v1.json"), encoding="utf-8") as f:
        d = json.load(f)
    with open(os.path.join(TV, "photo-0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e09.jpg"), "rb") as f:
        p09 = f.read()
    return d, {d["photos"][0]["id"]: p09}


FIXTURES = []   # (ten, doc, blobs, ky_vong, lop, mo_ta, lon)


def add(name, doc, blobs, expect, cls, desc, big=False):
    FIXTURES.append((name, doc, blobs, expect, cls, desc, big))


def build():
    C = [jpeg(i) for i in range(1, 12)]
    # ---------------- M: tai lieu cu (v1) co anh giay chung tu
    d, bl = base_v1()
    p1, p2, p3 = d["persons"]
    p1["death"]["notice"] = notice([U(0xC1), U(0xC2)])
    p2["death"]["notice"] = notice([U(0xC3)], reported_by=None, place=None, reason=None, notice_date=None)
    for i, k in ((0, 0xC1), (1, 0xC2), (2, 0xC3)):
        d["photos"].append(photo_rec(U(k), C[i]))
        bl[U(k)] = C[i]
    add("M01-cu-giay-chung-tu", d, bl, "accept", ["migrate"], "v1: 2 ho so da mat co thong tin bao tu, 3 anh giay chung tu, gioi_tinh vang")

    d, bl = base_v1()
    d["persons"][0]["death"]["notice"] = notice([U(0xD1 + i) for i in range(4)])
    for i in range(4):
        d["photos"].append(photo_rec(U(0xD1 + i), C[3 + i])); bl[U(0xD1 + i)] = C[3 + i]
    add("M02-cu-4-anh", d, bl, "accept", ["migrate"], "v1: toi da 4 anh giay chung tu -> 4 tep dinh kem")

    d, bl = base_v1()
    pid = d["persons"][0]["photo_id"]
    d["persons"][0]["death"]["notice"] = notice([pid])
    add("M03-cu-anh-chung", d, bl, "accept", ["migrate"], "v1: anh giay chung tu trung anh chan dung (A-M3): anh chan dung phai con")

    d, bl = base_v1()
    add("M04-cu-khong-bao-tu", d, bl, "accept", ["migrate"], "v1 khoa R6 (example-v1): chi tang phien ban + gioi_tinh null; quan_he chu thuong giu nguyen")

    d, bl = base_v1()
    d["persons"][0]["death"]["notice"] = notice([], reported_by=None, place=None, reason=None, notice_date=None, note=None)
    add("M05-cu-bao-tu-rong", d, bl, "accept", ["migrate"], "v1: notice moi truong null, certificate_photo_ids rong")

    d, bl = copy.deepcopy(FIXTURES[0][1]), dict(FIXTURES[0][2])
    d["persons"][0]["ten_goi_cu"] = "Cố Nguyễn"
    d["persons"][0]["death"]["notice"]["so_giay"] = "123/2025/TLKT"
    d["photos"][1]["nguon"] = "máy quét"
    d["ghi_chu_ban_cu"] = {"x": [1, 2]}
    add("M06-cu-khoa-la-legacy", d, bl, "accept_or_chua_ro", ["migrate", "legacy"],
        "v1 + khoa la o goc, Person, DeathNotice, Photo: phai giu trong legacy (A-M7), khong bo lang le")

    d, bl = base_v1()
    d["persons"][0]["danh_xung"] = "ông"
    d["persons"][0]["quan_he"] = "ông nội (bên nội)"
    d["persons"][1]["danh_xung"] = "bà cô tổ họ"
    d["persons"][1]["quan_he"] = "BÀ cố"
    d["persons"][2]["danh_xung"] = "cụ"          # nguoi song + cach goi chi danh cho nguoi mat: khong kiem theo danh sach
    d["persons"][2]["quan_he"] = "đứa em út"
    d["persons"][2]["ho_ten"] = "lê thị c"        # ho_ten khong bao gio viet hoa tu dong
    add("M07-cu-chu-thuong-nfc", d, bl, "accept", ["migrate", "names"],
        "v1: danh_xung/quan_he/ho_ten chu thuong, chu hoa le (NFC): nhap/chuyen doi giu nguyen tung byte")

    d, bl = base_v1()
    d["persons"][0]["danh_xung"] = unicodedata.normalize("NFD", "ông")
    d["persons"][0]["quan_he"] = unicodedata.normalize("NFD", "ừ, ông nội")
    d["persons"][2]["danh_xung"] = unicodedata.normalize("NFD", "Chị")
    assert d["persons"][0]["danh_xung"] != "ông"
    add("M08-cu-nfd", d, bl, "chua_ro", ["migrate", "names"], "v1: chuoi NFD trong tep nhap (A-N2): ghi nhan giu nguyen hay NFC")

    d, bl = base_v1()
    d["persons"][0]["death"]["notice"] = notice([U(0xEE)])
    add("M09-cu-anh-chung-tu-thieu", d, bl, "reject", ["V3"], "v1: certificate_photo_ids tro toi anh khong co")

    # Them o lan 3 (truoc khi chay lan 3): mot anh giay chung tu dung cho hai ho so (dem tham chieu)
    d, bl = base_v1()
    d["persons"][0]["death"]["notice"] = notice([U(0xC5)])
    d["persons"][1]["death"]["notice"] = notice([U(0xC5), U(0xC6)], reported_by=None)
    for k, i in ((0xC5, 4), (0xC6, 5)):
        d["photos"].append(photo_rec(U(k), C[i])); bl[U(k)] = C[i]
    add("M10-cu-anh-dung-chung", d, bl, "accept_or_chua_ro", ["migrate", "shared"],
        "v1: anh giay chung tu C5 dung cho hai ho so: phai con mot tep, ca hai ho so tro toi (A-U1)")

    # ---------------- N: phien ban
    m01 = FIXTURES[0]
    v2, b2 = R.migrate_1_to_2(m01[1], m01[2])
    d = copy.deepcopy(v2); d["schema_version"] = 3
    add("N01-schema-3", d, dict(b2), "reject", ["NEWER"], "tai lieu dang v2 nhung schema_version 3")
    d, bl = base_v1(); d["schema_version"] = 99
    add("N02-schema-99", d, bl, "reject", ["NEWER"], "tai lieu dang v1, schema_version 99")
    d = copy.deepcopy(v2); d["schema_version"] = 1
    add("N03-v1-dang-moi", d, dict(b2), "reject", ["V1"], "schema_version 1 nhung dung truong moi (attachments, gioi_tinh)")
    d = copy.deepcopy(m01[1]); d["schema_version"] = 2
    add("N04-v2-dang-cu", d, dict(m01[2]), "reject", ["V1"], "schema_version 2 nhung con certificate_photo_ids")
    d, bl = base_v1(); d["schema_version"] = 0
    add("N05-schema-0", d, bl, "reject", ["V1"], "schema_version 0")

    # ---------------- G: v2 hop le
    PDF = pdf()
    g = copy.deepcopy(v2); gb = dict(b2)
    g["persons"][0]["gioi_tinh"] = "nam"; g["persons"][1]["gioi_tinh"] = "nu"; g["persons"][2]["gioi_tinh"] = None
    g["persons"][0]["death"]["notice"]["attachments"].append({"id": U(0xA1), "kind": "tin_bao_tu", "file_id": U(0xF1)})
    g["persons"][1]["death"]["notice"]["attachments"].append({"id": U(0xA2), "kind": "khac", "file_id": U(0xF2)})
    g["attachments"].append(att_rec(U(0xF1), PDF, "application/pdf", "giay-bao-tu.pdf"))
    g["attachments"].append(att_rec(U(0xF2), C[7], "image/jpeg", "Ảnh tin báo tử.jpg"))
    gb[U(0xF1)] = PDF; gb[U(0xF2)] = C[7]
    g["attachments"].sort(key=lambda a: a["id"])
    G01 = (g, gb)
    add("G01-v2-hop-le", g, gb, "accept", ["v2"], "v2: anh + PDF, ba loai kind, gioi_tinh nam/nu/null")

    g = copy.deepcopy(G01[0]); gb = dict(G01[1])
    lst = g["persons"][0]["death"]["notice"]["attachments"]
    for i in range(6 - len(lst)):
        lst.append({"id": U(0xA10 + i), "kind": "khac", "file_id": U(0xF10 + i)})
        g["attachments"].append(att_rec(U(0xF10 + i), C[8 + i % 3] + bytes([i]) if False else C[8 + i % 3], "image/jpeg"))
        gb[U(0xF10 + i)] = C[8 + i % 3]
    assert len(lst) == 6
    g["attachments"].sort(key=lambda a: a["id"])
    add("G02-v2-dung-6-tep", g, gb, "accept", ["v2"], "v2: dung 6 tep dinh kem (bien tren)")

    g = copy.deepcopy(G01[0]); gb = dict(G01[1]); del g["persons"][2]["gioi_tinh"]
    add("G03-v2-gioi-tinh-vang", g, gb, "accept", ["v2"], "v2: khoa gioi_tinh vang mat (= null)")

    def with_att(blob, mime, rec_patch=None, ref_patch=None, blob_override=None, name="x"):
        g = copy.deepcopy(G01[0]); gb = dict(G01[1])
        ref = {"id": U(0xAB), "kind": "giay_chung_tu", "file_id": U(0xFB)}
        ref.update(ref_patch or {})
        g["persons"][1]["death"]["notice"]["attachments"].append(ref)
        r = att_rec(U(0xFB), blob, mime); r.update(rec_patch or {})
        g["attachments"].append(r); g["attachments"].sort(key=lambda a: a["id"])
        gb[U(0xFB)] = blob if blob_override is None else blob_override
        return g, gb

    # lon (sinh luc chay, khong commit)
    add("G04-pdf-dung-10MB", *with_att(pdf(R.PDF_MAX), "application/pdf"), "accept", ["v2"], "PDF dung 10 485 760 byte", big=True)
    add("G05-anh-dung-2MB", *with_att(jpeg(20, R.IMG_MAX), "image/jpeg"), "accept", ["v2"], "anh JPEG dung 2 097 152 byte", big=True)
    add("X02-pdf-qua-10MB", *with_att(pdf(R.PDF_MAX + 1), "application/pdf"), "reject", ["SIZE", "V1"], "PDF 10 485 761 byte", big=True)
    add("X03-anh-qua-2MB", *with_att(jpeg(21, R.IMG_MAX + 1), "image/jpeg"), "reject", ["SIZE"], "anh JPEG 2 097 153 byte", big=True)

    # ---------------- X: v2 khong hop le (phai bi tu choi)
    add("X01-pdf-sai-magic", *with_att(pdf(magic=b"%PDX-"), "application/pdf"), "reject", ["MAGIC"], "PDF bat dau '%PDX-'")
    add("X01b-pdf-co-bom-truoc", *with_att(b"\xef\xbb\xbf" + pdf(), "application/pdf"), "reject", ["MAGIC"], "PDF co BOM truoc '%PDF-'")
    add("X01c-pdf-chu-thuong", *with_att(pdf(magic=b"%pdf-"), "application/pdf"), "reject", ["MAGIC"], "PDF bat dau '%pdf-'")
    g, gb = copy.deepcopy(G01[0]), dict(G01[1])
    lst = g["persons"][0]["death"]["notice"]["attachments"]
    lst.append({"id": lst[0]["id"], "kind": "khac", "file_id": U(0xF2)})
    add("X04-trung-id-dinh-kem", g, gb, "reject", ["DUP"], "hai phan tu attachments cung id")
    g, gb = copy.deepcopy(G01[0]), dict(G01[1])
    g["persons"][1]["death"]["notice"]["attachments"].append({"id": g["persons"][0]["death"]["notice"]["attachments"][0]["id"], "kind": "khac", "file_id": U(0xF2)})
    add("X04b-trung-id-dinh-kem-hai-ho-so", g, gb, "reject", ["DUP"], "id phan tu dinh kem trung giua hai ho so")
    g, gb = copy.deepcopy(G01[0]), dict(G01[1])
    dup = copy.deepcopy([a for a in g["attachments"] if a["id"] == U(0xF1)][0]); g["attachments"].append(dup)
    add("X05-trung-id-ban-ghi-tep", g, gb, "reject", ["DUP", "BLOB", "V1"], "hai ban ghi Attachment cung id")
    g, gb = copy.deepcopy(G01[0]), dict(G01[1])
    g["persons"][0]["death"]["notice"]["attachments"].append({"id": U(0xAC), "kind": "khac", "file_id": U(0xFC)})
    add("X06-tep-duoc-tro-nhung-thieu", g, gb, "reject", ["V3"], "file_id khong co ban ghi Attachment")
    g, gb = copy.deepcopy(G01[0]), dict(G01[1]); del gb[U(0xF1)]
    add("X07-thieu-byte-tep", g, gb, "reject", ["BLOB"], "ban ghi Attachment co nhung khong co byte trong vung chua")
    g, gb = copy.deepcopy(G01[0]), dict(G01[1])
    lst = g["persons"][0]["death"]["notice"]["attachments"]
    for i in range(7 - len(lst)):
        lst.append({"id": U(0xA20 + i), "kind": "khac", "file_id": U(0xF20 + i)})
        g["attachments"].append(att_rec(U(0xF20 + i), C[8 + i % 3], "image/jpeg")); gb[U(0xF20 + i)] = C[8 + i % 3]
    assert len(lst) == 7
    g["attachments"].sort(key=lambda a: a["id"])
    add("X08-7-tep", g, gb, "reject", ["COUNT", "V1"], "7 tep dinh kem trong mot ho so")
    add("X09-kind-sai", *with_att(C[9], "image/jpeg", ref_patch={"kind": "giay_bao_tu"}), "reject", ["V1"], "kind 'giay_bao_tu' ngoai enum")
    add("X10-mime-png", *with_att(C[9], "image/png"), "reject", ["V1", "MAGIC"], "mime image/png ngoai enum")
    add("X11-sha-sai", *with_att(C[9], "image/jpeg", rec_patch={"sha256": "0" * 64}), "reject", ["SHA"], "sha256 khong khop byte")
    add("X12-bytes-sai", *with_att(C[9], "image/jpeg", rec_patch={"bytes": len(C[9]) + 1}), "reject", ["BYTES", "SHA"], "bytes khac do dai that")
    add("X13-pdf-stripped-true", *with_att(PDF, "application/pdf", rec_patch={"stripped": True}), "reject", ["V1"], "PDF voi stripped = true")
    add("X14-anh-stripped-false", *with_att(C[9], "image/jpeg", rec_patch={"stripped": False}), "reject", ["V1"], "anh voi stripped = false")
    for nm, val in (("X15-gioi-tinh-Nam", "Nam"), ("X16-gioi-tinh-khac", "khac"), ("X17-gioi-tinh-rong", ""), ("X17b-gioi-tinh-nu-dau", "nữ")):
        g, gb = copy.deepcopy(G01[0]), dict(G01[1]); g["persons"][0]["gioi_tinh"] = val
        add(nm, g, gb, "reject", ["V1"], "gioi_tinh = %r" % val)
    add("X18-ten-tep-121", *with_att(C[9], "image/jpeg", rec_patch={"file_name": "đ" * 121}), "reject", ["V1"], "file_name 121 diem ma")
    g, gb = copy.deepcopy(G01[0]), dict(G01[1]); gb[U(0xFE)] = C[10]
    add("X19-byte-thua", g, gb, "reject", ["BLOB"], "muc nhi phan khong co ban ghi nao")
    add("X20-mime-pdf-byte-jpeg", *with_att(C[9], "application/pdf", rec_patch={"stripped": False}), "reject", ["MAGIC"], "mime PDF nhung byte la JPEG")
    add("X21-mime-jpeg-byte-pdf", *with_att(PDF, "image/jpeg"), "reject", ["MAGIC"], "mime JPEG nhung byte la PDF")
    g, gb = copy.deepcopy(G01[0]), dict(G01[1])
    g["persons"][0]["death"]["notice"]["certificate_photo_ids"] = []
    add("X22-v2-con-khoa-cu", g, gb, "reject", ["V1"], "v2 con khoa certificate_photo_ids")
    add("X23-ten-tep-dieu-khien", *with_att(C[9], "image/jpeg", rec_patch={"file_name": "a\u0007b.jpg"}), "reject", ["V1"], "file_name co ky tu dieu khien")

    # ---------------- U: dac ta chua ro (chi ghi nhan)
    g, gb = copy.deepcopy(G01[0]), dict(G01[1])
    g["persons"][1]["death"]["notice"]["attachments"].append({"id": U(0xAD), "kind": "khac", "file_id": U(0xF1)})
    add("U01-hai-ho-so-chung-tep", g, gb, "chua_ro", ["A-U1"], "hai ho so tro cung mot Attachment")
    g, gb = copy.deepcopy(G01[0]), dict(G01[1])
    g["attachments"].append(att_rec(U(0xFD), C[10], "image/jpeg")); gb[U(0xFD)] = C[10]
    g["attachments"].sort(key=lambda a: a["id"])
    add("U02-tep-mo-coi", g, gb, "chua_ro", ["A-U1"], "Attachment khong ai tro toi")


def blob_list(doc, blobs):
    order = [p["id"] for p in doc.get("photos", [])] + [a["id"] for a in doc.get("attachments", [])]
    seen, out = set(), []
    for i in sorted(set(order)):
        if i in blobs and i not in seen:
            out.append((i, blobs[i])); seen.add(i)
    for i in sorted(blobs):
        if i not in seen:
            out.append((i, blobs[i]))
    return out


def main():
    big_dir = None
    if "--big" in sys.argv:
        big_dir = sys.argv[sys.argv.index("--big") + 1]
        os.makedirs(big_dir, exist_ok=True)
    build()
    os.makedirs(os.path.join(FX, "bytes"), exist_ok=True)
    os.makedirs(os.path.join(OUT, "b5_expected"), exist_ok=True)
    man, exp = [], []
    for name, doc, blobs, expect, cls, desc, big in FIXTURES:
        errs = R.validate(doc, blobs)
        codes = sorted({c for c, _ in errs})
        # tu kiem: ky vong viet tay phai khop bo kiem tra cua chinh tham chieu
        if expect == "accept":
            assert not errs, (name, errs)
        elif expect == "reject":
            assert errs and set(codes) & set(cls), (name, codes, cls)
        jb = L.canonical_json(doc)
        bl = blob_list(doc, blobs)
        entry = {"name": name, "desc": desc, "schema_version": doc.get("schema_version"), "expect": expect,
                 "class": cls, "oracle_codes": codes, "json_sha256": sha(jb),
                 "blobs": {i: {"sha256": sha(b), "len": len(b)} for i, b in bl}, "big": big}
        tgt = big_dir if big else FX
        if tgt:
            with open(os.path.join(tgt, name + ".json"), "wb") as f:
                f.write(jb)
            if not big:
                for i, b in bl:
                    p = os.path.join(FX, "bytes", sha(b) + ".bin")
                    if not os.path.exists(p):
                        open(p, "wb").write(b)
            else:
                d = os.path.join(big_dir, name + ".bytes"); os.makedirs(d, exist_ok=True)
                for i, b in bl:
                    open(os.path.join(d, i + ".bin"), "wb").write(b)
            lv = L.encrypt_file(L.pack(jb, bl), PW, SALT, NONCE)
            with open(os.path.join(tgt, name + ".lvhv"), "wb") as f:
                f.write(lv)
            entry["lvhv_sha256"] = sha(lv)
        man.append(entry)
        e = {"name": name, "expect": expect, "class": cls}
        if expect in ("accept", "accept_or_chua_ro"):
            exp_doc = R.migrate_1_to_2(doc, blobs)[0] if doc["schema_version"] == 1 else doc
            if "shared" not in cls:  # M10: bo kiem tra tham chieu coi id trung giua hai ho so la DUP (mau thuan tu ghi, muc 5 bao cao)
                assert not R.validate(exp_doc, R.migrate_1_to_2(doc, blobs)[1] if doc["schema_version"] == 1 else blobs), name
            ej = L.canonical_json(exp_doc)
            if not big:
                open(os.path.join(OUT, "b5_expected", name + ".json"), "wb").write(ej)
            e["expected_json_sha256"] = sha(ej)
            nd, leg = R.norm_for_compare(exp_doc)
            e["expected_norm_sha256"] = sha(L.canonical_json(nd))
            e["expected_legacy"] = sorted(R.legacy_values(exp_doc))
            e["counts"] = {k: len(exp_doc.get(k, [])) for k in ("persons", "events", "event_types", "photos", "attachments")}
        exp.append(e)
    names = []
    cases = [("ông", None), ("bà", None), ("cụ", None), ("kỵ", None), ("ông tổ họ", None), ("bà cô tổ họ", None),
             ("bé đỏ", None), ("đứa em", None), ("ừ", None), ("ỵ", None), ("ơ kìa", None), ("Ông", None), ("ÔNG", None),
             ("bÀ", None), ("  chú  ", None), ("", None), ("   ", None), ("2 bà", None), ("ông nội (bên nội)", None),
             ("(bên nội) ông", "A-N1"), ("́a", "A-N1"), ("«bà»", "A-N1"),
             (unicodedata.normalize("NFD", "ông"), None), (unicodedata.normalize("NFD", "ừ"), None),
             (unicodedata.normalize("NFD", "ấu"), None), ("ống", None), ("đ", None), ("ừ", None)]
    for s, amb in cases:
        names.append({"input": s, "input_nfc": R.nfc(s), "expected": R.cap_first(s),
                      "alt_first_letter": R.cap_first_alt(s), "ambiguous": amb or (R.cap_first(s) != R.cap_first_alt(s) and "A-N1") or None})
    result = {
        "spec_commit": "6ee647b (origin/main)", "app_schema_version": R.APP_SCHEMA, "assumptions": R.GIA_DINH,
        "password": PW, "salt_hex": SALT.hex(), "nonce_hex": NONCE.hex(),
        "fixtures": exp,
        "names_on_save": names, "names_on_import": "giu nguyen tung byte (A-N2); NFD: chua ro",
        "goi_y": {"deceased": R.GOI_Y_DA_MAT, "living": R.GOI_Y_DANG_SONG},
        "limits": {"attachments_max": R.ATT_MAX, "image_max_bytes": R.IMG_MAX, "pdf_max_bytes": R.PDF_MAX,
                   "kinds": list(R.KINDS), "mimes": list(R.MIMES), "pdf_magic": "%PDF-", "file_name_max": 120},
    }
    with open(os.path.join(OUT, "b5_expected.json"), "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, indent=1, sort_keys=True)
    with open(os.path.join(FX, "manifest.json"), "w", encoding="utf-8") as f:
        json.dump({"password": PW, "salt_hex": SALT.hex(), "nonce_hex": NONCE.hex(), "fixtures": man},
                  f, ensure_ascii=False, indent=1, sort_keys=True)
    print("fixtures:", len(man), "accept:", sum(e["expect"] == "accept" for e in exp),
          "reject:", sum(e["expect"] == "reject" for e in exp), "other:", sum(e["expect"] not in ("accept", "reject") for e in exp))


if __name__ == "__main__":
    main()
