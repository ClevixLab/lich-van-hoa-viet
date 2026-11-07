# -*- coding: utf-8 -*-
"""Tham chieu doc lap cho R14 (04 Bo sung B5) va YC-2026-07 (01 §3.1, §3.2a, §3.8a, §7).

Viet tu dac ta tren origin/main 6ee647b, KHONG doc ma nguon memorial/ store/ backup/ core/ tsport/ ffi/
pwa/ apps/. Moi cho dac ta chua noi ro duoc dang ky truoc la gia dinh A-* (xem GIA_DINH ben duoi); khi
doi chieu, sai khac o mot gia dinh duoc ghi la "dac ta chua ro", khong tinh la loi cua ben nao.

Ham chinh
  migrate_1_to_2(doc, blobs) -> (doc2, blobs2)   chuyen doi thuan, khong I/O
  validate(doc, blobs)       -> [(ma, con_tro)]    rong = hop le
  cap_first(s)               -> chuoi              quy tac viet hoa chu cai dau (luu, khong ap khi nhap)
  GOI_Y_DA_MAT, GOI_Y_DANG_SONG                    danh sach goi y danh_xung dung nhu 01 §3.1
"""
import copy, hashlib, json, os, re, unicodedata

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
V1_SCHEMA = os.path.join(ROOT, "docs", "v0.2", "schema", "backup-v1.schema.json")

APP_SCHEMA = 2          # A-V1: lan doi luoc do nay tang schema_version 1 -> 2 (01 §7: doi ten truong => tang)
MB = 1024 * 1024        # A-L1: "MB" hieu nhu §3.8 (1 MB = 1 048 576)
IMG_MAX = 2 * MB
PDF_MAX = 10 * MB
ATT_MAX = 6
KINDS = ("tin_bao_tu", "giay_chung_tu", "khac")
MIMES = ("image/jpeg", "image/webp", "application/pdf")
UUID_RE = re.compile(r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")

GIA_DINH = {
    "A-V1": "Doi certificate_photo_ids -> attachments tang schema_version 1 -> 2; tai lieu > 2 bi tu choi (newer_version).",
    "A-L1": "2 MB = 2 097 152 byte, 10 MB = 10 485 760 byte (cung don vi voi 'Photo ... 1 MB = 1 048 576', 01 §3.8). Dung bien: bang gioi han thi hop le.",
    "A-R1": "Ban ghi Attachment nam o mang goc 'attachments' cua tai lieu (canh 'photos'); byte di trong vung chua nhi phan cung danh sach voi anh (A-C1).",
    "A-M1": "Moi id trong certificate_photo_ids -> mot phan tu attachments {kind: giay_chung_tu}, giu thu tu. id cua phan tu tham chieu va file_id KHONG duoc dac ta quy dinh: so sanh khong phu thuoc gia tri id, chi doi hoi UUID v4 hop le, khong trung, va tep duoc tro toi co dung byte/sha256/mime cua anh cu.",
    "A-M2": "Ban ghi Photo cua anh giay chung tu duoc chuyen thanh Attachment {mime, bytes, sha256, created_at giu nguyen; stripped = true; file_name = null}. width/height cua Photo khong co cho trong Attachment: duoc phep bo vi suy ra duoc tu byte anh giu nguyen (khong tinh mat du lieu); giu trong legacy cung hop le.",
    "A-M3": "Neu anh giay chung tu dong thoi la anh chan dung (photo_id) cua mot ho so thi Photo van con trong photos (khong duoc mat anh chan dung) va van co Attachment tuong ung.",
    "A-M4": "Anh giay chung tu chi dung cho giay chung tu: Photo bi go khoi photos sau khi chuyen (con trong photos cung khong mat du lieu: chi ghi nhan, khong tinh sai lech).",
    "A-M5": "gioi_tinh vang mat -> null. Vang khoa va null la tuong duong khi so (01 §3.1 'Thieu khoa = null').",
    "A-M6": "Chuyen doi KHONG viet hoa danh_xung/quan_he va khong doi chuoi nao (B5 muc 1: khong doi nghia; 'ap khi luu, khong ap khi nhap').",
    "A-M7": "Khoa la (khong co trong luoc do v1) cua mot ban ghi duoc giu nguyen gia tri trong mot doi tuong 'legacy' (B5 muc 2). Vi tri chinh xac khong duoc dac ta: dat 'legacy' trong chinh ban ghi do la ky vong cua tham chieu; khi so, chap nhan bat ky khoa 'legacy' nao chua cung ten khoa va cung gia tri. Tu choi ca tai lieu (khong ghi gi) -> CHUA RO (dac ta: B5 muc 2 vs 01 §6.3); bo lang le -> SAI LECH.",
    "A-N1": "Viet hoa: cat khoang trang hai dau, NFC, roi doi diem ma DAU TIEN sang chu hoa (str.upper), phan con lai giu nguyen; NFC lai. Diem ma dau khong phai chu cai co hoa thuong -> giu nguyen (ca '(ben noi)' -> khong doi: diem ma dau la '('). Cach hieu khac ('chu cai dau tien tim thay') duoc danh dau ca CHUA RO.",
    "A-N2": "Khi nhap sao luu: danh_xung/quan_he giu dung byte trong tep (khong viet hoa). Chuoi NFD trong tep nhap: giu nguyen hay chuan hoa NFC deu chua duoc dac ta cho duong nhap -> CHUA RO, chi ghi nhan.",
    "A-C1": "Byte tep dinh kem di trong cung danh sach muc nhi phan cua ban ro (03 §2.3) voi anh, khop theo id ban ghi Attachment; gioi han moi muc 1 MiB cua anh duoc noi thanh 10 MiB cho PDF.",
    "A-U1": "Hai ho so tro cung mot file_id, hoac Attachment khong ai tro toi: dac ta chi noi 'thuoc ve ho so nay' -> CHUA RO, chi ghi nhan.",
}

# 01 §3.1, dung thu tu trong tai lieu
GOI_Y_DA_MAT = ["Ông", "Bà", "Cụ", "Kỵ", "Bác", "Cô", "Chú", "Dì", "Cậu", "Mợ", "Thím", "Anh", "Chị", "Em",
                "Cháu", "Con", "Bạn", "Bố", "Mẹ", "Ông Tổ họ", "Bà Cô Tổ họ", "Bé đỏ"]
_KHONG_CHO_NGUOI_SONG = {"Cụ", "Kỵ", "Ông Tổ họ", "Bà Cô Tổ họ", "Bé đỏ"}
GOI_Y_DANG_SONG = [x for x in GOI_Y_DA_MAT if x not in _KHONG_CHO_NGUOI_SONG]


def nfc(s):
    return unicodedata.normalize("NFC", s)


def cap_first(s):
    """A-N1. Tra ve chuoi sau khi luu."""
    s = nfc(s).strip()
    if not s:
        return s
    return nfc(s[0].upper() + s[1:])


def cap_first_alt(s):
    """Cach hieu thay the (chi de danh dau ca chua ro): viet hoa chu cai dau tien tim thay."""
    s = nfc(s).strip()
    for i, ch in enumerate(s):
        if ch.isalpha():
            return nfc(s[:i] + ch.upper() + s[i + 1:])
    return s


# ---------------------------------------------------------------- luoc do
def load_v1_schema():
    with open(V1_SCHEMA, encoding="utf-8") as f:
        return json.load(f)


def v2_schema():
    """Luoc do v2 suy ra tu v1 + 01 §3.1/§3.2a/§3.8a (A-R1)."""
    s = load_v1_schema()
    s["$id"] = "oracle:backup-v2"
    s["properties"]["schema_version"] = {"const": 2}
    s["properties"]["attachments"] = {"type": "array", "maxItems": 60000, "items": {"$ref": "#/$defs/Attachment"}}
    d = s["$defs"]
    d["Person"]["properties"]["gioi_tinh"] = {"enum": ["nam", "nu", None]}
    d["Person"]["properties"]["legacy"] = {"type": "object"}
    n = d["DeathNotice"]
    del n["properties"]["certificate_photo_ids"]
    n["required"] = [k for k in n["required"] if k != "certificate_photo_ids"] + ["attachments"]
    n["properties"]["attachments"] = {"type": "array", "maxItems": ATT_MAX, "items": {"$ref": "#/$defs/AttachmentRef"}}
    n["properties"]["legacy"] = {"type": "object"}
    d["AttachmentRef"] = {"type": "object", "additionalProperties": False, "required": ["id", "kind", "file_id"],
                          "properties": {"id": {"$ref": "#/$defs/uuid"}, "kind": {"enum": list(KINDS)},
                                         "file_id": {"$ref": "#/$defs/uuid"}}}
    d["Attachment"] = {"type": "object", "additionalProperties": False,
                       "required": ["id", "mime", "bytes", "sha256", "stripped", "file_name", "created_at"],
                       "properties": {"id": {"$ref": "#/$defs/uuid"}, "mime": {"enum": list(MIMES)},
                                      "bytes": {"type": "integer", "minimum": 1, "maximum": PDF_MAX},
                                      "sha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"},
                                      "stripped": {"type": "boolean"},
                                      "file_name": {"oneOf": [{"allOf": [{"$ref": "#/$defs/text"}, {"maxLength": 120}]}, {"type": "null"}]},
                                      "created_at": {"$ref": "#/$defs/timestamp"},
                                      "legacy": {"type": "object"}}}
    s["properties"]["legacy"] = {"type": "object"}
    for k in ("Photo", "Event", "EventType", "DeathInfo", "BirthInfo", "MemorialSettings"):
        if k in d:
            d[k].setdefault("properties", {})["legacy"] = {"type": "object"}
    return s


_VAL = {}


def _validator(ver):
    import jsonschema
    if ver not in _VAL:
        sch = load_v1_schema() if ver == 1 else v2_schema()
        _VAL[ver] = jsonschema.Draft202012Validator(sch)
    return _VAL[ver]


def schema_errors(doc, ver):
    return [("V1", "/" + "/".join(str(x) for x in e.absolute_path)) for e in _validator(ver).iter_errors(doc)]


# ---------------------------------------------------------------- kiem tra
def _magic_ok(mime, b):
    if mime == "application/pdf":
        return b[:5] == b"%PDF-"
    if mime == "image/jpeg":
        return b[:3] == b"\xff\xd8\xff"
    if mime == "image/webp":
        return b[:4] == b"RIFF" and b[8:12] == b"WEBP"
    return False


def validate(doc, blobs):
    """Tra ve danh sach loi [(ma, con_tro)]. Ma: NEWER, V1, V3, DUP, COUNT, SIZE, MAGIC, SHA, BYTES, BLOB.
    Chi cac quy tac moi cua YC-2026-07 va phan luoc do; V2, V4-V8 cu do bo R6 da khoa kiem."""
    errs = []
    sv = doc.get("schema_version") if isinstance(doc, dict) else None
    if isinstance(sv, int) and not isinstance(sv, bool) and sv > APP_SCHEMA:
        return [("NEWER", "/schema_version")]
    if sv not in (1, 2):
        return [("V1", "/schema_version")]
    errs += schema_errors(doc, sv)
    photos = {p.get("id"): p for p in doc.get("photos", []) if isinstance(p, dict)}
    atts = {}
    for i, a in enumerate(doc.get("attachments", []) if sv == 2 else []):
        if not isinstance(a, dict):
            continue
        if a.get("id") in atts:
            errs.append(("DUP", "/attachments/%d/id" % i))
        atts[a.get("id")] = a
    # tham chieu
    seen_ref = set()
    for pi, p in enumerate(doc.get("persons", [])):
        if not isinstance(p, dict):
            continue
        n = ((p.get("death") or {}).get("notice")) if isinstance(p.get("death"), dict) else None
        if not isinstance(n, dict):
            continue
        if sv == 1:
            for j, x in enumerate(n.get("certificate_photo_ids", []) or []):
                if x not in photos:
                    errs.append(("V3", "/persons/%d/death/notice/certificate_photo_ids/%d" % (pi, j)))
            continue
        lst = n.get("attachments")
        if not isinstance(lst, list):
            continue
        if len(lst) > ATT_MAX:
            errs.append(("COUNT", "/persons/%d/death/notice/attachments" % pi))
        for j, r in enumerate(lst):
            if not isinstance(r, dict):
                continue
            ptr = "/persons/%d/death/notice/attachments/%d" % (pi, j)
            if r.get("id") in seen_ref:
                errs.append(("DUP", ptr + "/id"))
            seen_ref.add(r.get("id"))
            if r.get("file_id") not in atts:
                errs.append(("V3", ptr + "/file_id"))
    # tep dinh kem: gioi han, stripped, magic, sha256, do dai
    for i, a in enumerate(doc.get("attachments", []) if sv == 2 else []):
        if not isinstance(a, dict):
            continue
        ptr = "/attachments/%d" % i
        mime, nb = a.get("mime"), a.get("bytes")
        if mime == "application/pdf":
            if a.get("stripped") is not False:
                errs.append(("V1", ptr + "/stripped"))
            if isinstance(nb, int) and nb > PDF_MAX:
                errs.append(("SIZE", ptr + "/bytes"))
        elif mime in ("image/jpeg", "image/webp"):
            if a.get("stripped") is not True:
                errs.append(("V1", ptr + "/stripped"))
            if isinstance(nb, int) and nb > IMG_MAX:
                errs.append(("SIZE", ptr + "/bytes"))
        b = blobs.get(a.get("id"))
        if b is None:
            errs.append(("BLOB", ptr))
            continue
        if len(b) > (PDF_MAX if mime == "application/pdf" else IMG_MAX):
            errs.append(("SIZE", ptr))
        if len(b) != nb:
            errs.append(("BYTES", ptr + "/bytes"))
        if hashlib.sha256(b).hexdigest() != a.get("sha256"):
            errs.append(("SHA", ptr + "/sha256"))
        if not _magic_ok(mime, b):
            errs.append(("MAGIC", ptr))
    # khong co muc nhi phan thua (03 §2.3: thua hoac thieu -> tu choi)
    known = set(photos) | set(atts)
    for k in blobs:
        if k not in known:
            errs.append(("BLOB", "blob:" + k))
    return errs


# ---------------------------------------------------------------- chuyen doi
def _v1_props(schema, defname):
    return set(schema["$defs"][defname]["properties"])


def _move_unknown(obj, allowed):
    unk = {k: obj[k] for k in list(obj) if k not in allowed}
    for k in unk:
        del obj[k]
    if unk:
        obj.setdefault("legacy", {}).update(unk)


def migrate_1_to_2(doc, blobs):
    """Ham thuan. doc: tai lieu v1 (co the co khoa la), blobs: {uuid: bytes}. Tra (doc2, blobs2)."""
    if doc.get("schema_version") != 1:
        raise ValueError("chi nhan schema_version 1")
    s1 = load_v1_schema()
    d = copy.deepcopy(doc)
    b2 = dict(blobs)
    root_allowed = set(s1["properties"])
    _move_unknown(d, root_allowed)
    d["schema_version"] = 2
    portrait = {p.get("photo_id") for p in d.get("persons", []) if p.get("photo_id")}
    photos = {p["id"]: p for p in d.get("photos", [])}
    atts, moved = [], set()
    for p in d.get("persons", []):
        _move_unknown(p, _v1_props(s1, "Person"))
        p.setdefault("gioi_tinh", None)                                   # A-M5
        death = p.get("death")
        if isinstance(death, dict):
            _move_unknown(death, _v1_props(s1, "DeathInfo"))
            n = death.get("notice")
            if isinstance(n, dict):
                _move_unknown(n, _v1_props(s1, "DeathNotice"))
                ids = n.pop("certificate_photo_ids", []) or []
                n["attachments"] = []
                for pid in ids:                                           # A-M1
                    ph = photos[pid]
                    n["attachments"].append({"id": pid, "kind": "giay_chung_tu", "file_id": pid})
                    if pid not in moved:
                        atts.append({"id": pid, "mime": ph["mime"], "bytes": ph["bytes"], "sha256": ph["sha256"],
                                     "stripped": True, "file_name": None, "created_at": ph["created_at"]})  # A-M2
                        moved.add(pid)
        for k in ("birth", "memorial"):
            if isinstance(p.get(k), dict):
                _move_unknown(p[k], _v1_props(s1, "BirthInfo" if k == "birth" else "MemorialSettings"))
    for ph in d.get("photos", []):
        _move_unknown(ph, _v1_props(s1, "Photo"))
    for e in d.get("events", []):
        _move_unknown(e, _v1_props(s1, "Event"))
    for t in d.get("event_types", []):
        _move_unknown(t, _v1_props(s1, "EventType"))
    # A-M3 / A-M4
    d["photos"] = [ph for ph in d.get("photos", []) if ph["id"] not in moved or ph["id"] in portrait]
    d["attachments"] = sorted(atts, key=lambda a: a["id"])
    return d, b2


# ---------------------------------------------------------------- so sanh
NULL_EQ_KEYS = {"gioi_tinh", "notice", "legacy"}


def norm_for_compare(doc):
    """Chuan hoa de so 'khong phu thuoc gia dinh': bo app/exported_at (03 B1.4 muc 10), vang khoa == null
    cho NULL_EQ_KEYS, mang goc sap theo id, legacy tach rieng; attachments ref -> (kind, sha256 cua tep)."""
    d = copy.deepcopy(doc)
    for k in ("app", "exported_at"):
        d.pop(k, None)
    legacy = {}

    def strip_null(o, path):
        if isinstance(o, dict):
            if "legacy" in o:
                legacy[path] = o.pop("legacy")
            for k in list(o):
                if k in NULL_EQ_KEYS and o[k] is None:
                    del o[k]
                else:
                    strip_null(o[k], path + "/" + k)
        elif isinstance(o, list):
            for i, x in enumerate(o):
                strip_null(x, path + "/" + str(x.get("id") if isinstance(x, dict) and "id" in x else i))
    strip_null(d, "")
    att = {a["id"]: a for a in d.get("attachments", [])}
    for p in d.get("persons", []):
        n = (p.get("death") or {}).get("notice")
        if isinstance(n, dict) and isinstance(n.get("attachments"), list):
            n["attachments"] = [{"kind": r.get("kind"),
                                 "file": {k: v for k, v in att.get(r.get("file_id"), {"missing": r.get("file_id")}).items() if k != "id"}}
                                for r in n["attachments"]]
    d["attachments"] = sorted(({k: v for k, v in a.items() if k != "id"} for a in d.get("attachments", [])),
                              key=lambda a: (a.get("sha256", ""), a.get("created_at", "")))
    for k in ("persons", "events", "event_types", "photos"):
        if isinstance(d.get(k), list):
            d[k] = sorted(d[k], key=lambda x: x.get("id", ""))
    return d, legacy


def diff(a, b, path=""):
    out = []
    if isinstance(a, dict) and isinstance(b, dict):
        for k in sorted(set(a) | set(b)):
            if k not in a:
                out.append((path + "/" + k, "<vang>", b[k]))
            elif k not in b:
                out.append((path + "/" + k, a[k], "<vang>"))
            else:
                out += diff(a[k], b[k], path + "/" + k)
    elif isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b):
            out.append((path, "len=%d" % len(a), "len=%d" % len(b)))
        for i, (x, y) in enumerate(zip(a, b)):
            out += diff(x, y, path + "/" + str(x.get("id", i) if isinstance(x, dict) else i))
    elif a != b or type(a) != type(b):
        out.append((path, a, b))
    return out


def legacy_values(doc):
    """Moi cap (khoa, gia tri) nam duoi bat ky khoa 'legacy' nao (A-M7)."""
    found = []

    def walk(o):
        if isinstance(o, dict):
            for k, v in o.items():
                if k == "legacy" and isinstance(v, dict):
                    found.extend((kk, json.dumps(vv, sort_keys=True, ensure_ascii=False)) for kk, vv in v.items())
                walk(v)
        elif isinstance(o, list):
            for x in o:
                walk(x)
    walk(doc)
    return found


def ids_ok(doc):
    """A-M1: id tham chieu va id Attachment la UUID v4, khong trung."""
    bad, seen = [], set()
    for a in doc.get("attachments", []):
        if not UUID_RE.match(a.get("id", "")) or a["id"] in seen:
            bad.append(a.get("id"))
        seen.add(a.get("id"))
    seen = set()
    for p in doc.get("persons", []):
        n = (p.get("death") or {}).get("notice")
        for r in (n or {}).get("attachments", []) if isinstance(n, dict) else []:
            if not UUID_RE.match(r.get("id", "")) or r["id"] in seen:
                bad.append(r.get("id"))
            seen.add(r.get("id"))
    return bad
