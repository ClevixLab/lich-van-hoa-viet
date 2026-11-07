# -*- coding: utf-8 -*-
"""R6 (c)/(d) doc lap — bo giai ma Python thu hai, viet tu 03 §2 bang hai thu vien da kiem chung:
  Argon2id: argon2-cffi (argon2.low_level.hash_secret_raw, Type.ID, phien ban 0x13)
  XChaCha20-Poly1305: PyNaCl (libsodium) nacl.bindings.crypto_aead_xchacha20poly1305_ietf_*
Doi chieu cheo voi bo giai ma thu nhat (lvhvfile.py: `cryptography` + HChaCha20 tu cai).

Cach dung:
  python3 -I b5_r6v2.py vectors                      # R6 (c): RFC 9106 §5.3 + draft-irtf-cfrg-xchacha A.3.1
  python3 -I b5_r6v2.py decrypt <tep.lvhv> <mat-khau> [manifest-r6v2.json thu-muc-anh]   # R6 (d)
  python3 -I b5_r6v2.py flips <tep.lvhv> <mat-khau> <ra.json>   # R6 (b) phia doc lap: lat tung bit 4096 B dau + 4096 B cuoi
"""
import hashlib, json, os, struct, sys, unicodedata, uuid

import argon2.low_level as A
from nacl import bindings as NB

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lvhvfile as L1

MAGIC = b"LVHVBAK\x00"
PT_MAGIC = b"LVHVPT01"


class Reject(Exception):
    pass


def header_check(data):
    """03 §2.2, dung thu tu B1.4 muc 1."""
    if len(data) < 88 or data[:8] != MAGIC:
        raise Reject("not_a_backup")
    fv, kid, aid, m, t, p = struct.unpack_from("<HBBIII", data, 8)
    if fv > 1:
        raise Reject("newer_version")
    if fv == 0:
        raise Reject("unsupported")
    if kid != 1 or aid != 1:
        raise Reject("unsupported")
    if not (65536 <= m <= 1048576 and 1 <= t <= 10 and 1 <= p <= 8):
        raise Reject("bad_kdf_params")
    (ptl,) = struct.unpack_from("<Q", data, 64)
    if ptl != len(data) - 72 - 16 or ptl > 128 * 1024 * 1024:
        raise Reject("bad_length")
    return m, t, p, data[24:40], data[40:64]


def kdf2(password, salt, m, t, p):
    pw = unicodedata.normalize("NFC", password).encode("utf-8")
    return A.hash_secret_raw(pw, salt, time_cost=t, memory_cost=m, parallelism=p, hash_len=32, type=A.Type.ID, version=19)


def decrypt2(data, password, key_cache=None):
    m, t, p, salt, nonce = header_check(data)
    ck = (bytes(data[8:64]),)
    if key_cache is not None and ck in key_cache:
        key = key_cache[ck]
    else:
        key = kdf2(password, salt, m, t, p)
        if key_cache is not None:
            key_cache[ck] = key
    try:
        return NB.crypto_aead_xchacha20poly1305_ietf_decrypt(bytes(data[72:]), bytes(data[:72]), nonce, key), key
    except Exception:
        raise Reject("wrong_password_or_corrupt")


def unpack_strict(pt):
    """03 §2.3 + B1.4 muc 9: bo cuc va gioi han."""
    if pt[:8] != PT_MAGIC:
        raise Reject("malformed_plaintext")
    (jl,) = struct.unpack_from("<I", pt, 8)
    if jl > 64 * 1024 * 1024 or 12 + jl + 4 > len(pt):
        raise Reject("malformed_plaintext")
    j = pt[12:12 + jl]
    o = 12 + jl
    (n,) = struct.unpack_from("<I", pt, o); o += 4
    blocks = []
    for _ in range(n):
        if o + 20 > len(pt):
            raise Reject("malformed_plaintext")
        u = str(uuid.UUID(bytes=pt[o:o + 16])); o += 16
        (ln,) = struct.unpack_from("<I", pt, o); o += 4
        if o + ln > len(pt):
            raise Reject("malformed_plaintext")
        blocks.append((u, pt[o:o + ln])); o += ln
    if o != len(pt):
        raise Reject("malformed_plaintext")
    return j, blocks


def vectors():
    from cryptography.hazmat.primitives.kdf.argon2 import Argon2id
    out = {}
    # RFC 9106 §5.3 (Argon2id, v=0x13, m=32 KiB, t=3, p=4, secret 8x03, ad 12x04, tag 32)
    tag = Argon2id(salt=b"\x02" * 16, length=32, iterations=3, lanes=4, memory_cost=32,
                   ad=b"\x04" * 12, secret=b"\x03" * 8).derive(b"\x01" * 32).hex()
    out["argon2id_rfc9106_5_3"] = tag == "0d640df58d78766c08c037a34a8b53c9d01ef0452d75b65eb52520e96b01e659"
    # argon2-cffi khong nhan secret/ad: kiem rang no cho cung khoa voi `cryptography` tren tham so cua ung dung
    k1 = Argon2id(salt=b"\x07" * 16, length=32, iterations=3, lanes=4, memory_cost=65536).derive("mật khẩu".encode())
    k2 = A.hash_secret_raw("mật khẩu".encode(), b"\x07" * 16, time_cost=3, memory_cost=65536, parallelism=4,
                           hash_len=32, type=A.Type.ID, version=19)
    out["argon2_two_libs_same_key_64MiB_t3_p4"] = k1 == k2
    # draft-irtf-cfrg-xchacha-03 A.3.1
    pt = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it."
    aad = bytes.fromhex("50515253c0c1c2c3c4c5c6c7")
    key = bytes(range(0x80, 0xa0))
    nonce = bytes(range(0x40, 0x58))
    want = ("bd6d179d3e83d43b9576579493c0e939572a1700252bfaccbed2902c21396cbb731c7f1b0b4aa6440bf3a82f4eda7e39ae64c6708c54c216cb96b72e1213b4522f8c9ba40db5d945b11b69b982c1bb9e3f3fac2bc369488f76b2383565d3fff921f9664c97637da9768812f615c68b13b52e"
            + "c0875924c1c7987947deafd8780acf49")
    ct_nacl = NB.crypto_aead_xchacha20poly1305_ietf_encrypt(pt, aad, nonce, key).hex()
    ct_mine = L1.xchacha_encrypt(key, nonce, pt, aad).hex()
    out["xchacha_A31_pynacl"] = ct_nacl == want
    out["xchacha_A31_hchacha_tu_cai"] = ct_mine == want
    return out


def decrypt_report(path, password, manifest=None, photo_dir=None):
    data = open(path, "rb").read()
    rep = {"file": os.path.basename(path), "len": len(data), "sha256": hashlib.sha256(data).hexdigest()}
    pt2, key2 = decrypt2(data, password)
    pt1 = L1.decrypt_file(data, password)
    rep["two_decryptors_same_plaintext"] = pt1 == pt2
    rep["plaintext_len_matches_header"] = len(pt2) == struct.unpack_from("<Q", data, 64)[0]
    j, blocks = unpack_strict(pt2)
    doc = json.loads(j)
    canon = json.dumps(doc, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode("utf-8")
    rep["json_canonical"] = canon == j
    rep["schema_version"] = doc.get("schema_version")
    rep["counts"] = {k: len(doc.get(k, [])) for k in ("persons", "events", "event_types", "photos", "attachments")}
    order = [p["id"] for p in doc.get("photos", [])] + [a["id"] for a in doc.get("attachments", [])]
    rep["block_order_matches_records"] = [b[0] for b in blocks] == order
    recs = {r["id"]: r for r in doc.get("photos", []) + doc.get("attachments", [])}
    bad = []
    for u, b in blocks:
        r = recs.get(u)
        if r is None or r["bytes"] != len(b) or r["sha256"] != hashlib.sha256(b).hexdigest():
            bad.append(u)
    rep["blocks_sha256_and_len_match_records"] = not bad
    rep["bad_blocks"] = bad
    # anh: phan dau va kich thuoc doc lap (Pillow), khong siêu du lieu
    from PIL import Image
    import io
    img = []
    for u, b in blocks:
        r = recs[u]
        if r.get("mime", "").startswith("image/"):
            im = Image.open(io.BytesIO(b))
            exif = b"Exif\x00\x00" in b[:4096] or b"EXIF" in b[:64] or b"XMP " in b[:4096]
            img.append({"id": u[-4:], "fmt": im.format, "size": list(im.size), "decl": [r.get("width"), r.get("height")],
                        "mime": r["mime"], "meta": exif})
    rep["images_ok"] = all(((x["fmt"] == "JPEG") == (x["mime"] == "image/jpeg")) and (x["size"] == x["decl"] or x["decl"] == [None, None]) and not x["meta"]
                           for x in img) if img else None
    rep["n_images"] = len(img)
    if manifest:
        m = json.load(open(manifest, encoding="utf-8"))
        files = m["files"]
        want = sorted(v for k, v in files.items() if k.startswith("photo-"))
        rep["photo_sha256_match_manifest"] = sorted(hashlib.sha256(b).hexdigest() for _, b in blocks) == want
        if photo_dir:
            rep["photo_files_match_manifest"] = all(hashlib.sha256(open(os.path.join(photo_dir, k), "rb").read()).hexdigest() == v
                                                    for k, v in files.items())
            ds = json.load(open(os.path.join(photo_dir, m["dataset"]), encoding="utf-8"))
            strip = lambda d: {k: v for k, v in d.items() if k not in ("app", "exported_at")}
            rep["json_equals_dataset_file"] = strip(ds) == strip(doc)
    import b5_statehash as S
    rep["independent_state_sha256"] = S.state_hash(doc, dict(blocks))
    return rep, doc, blocks


def flips(path, password, outp):
    data = bytearray(open(path, "rb").read())
    n = len(data)
    idx = sorted(set(list(range(min(4096, n))) + list(range(max(0, n - 4096), n))))
    cache = {}
    decrypt2(bytes(data), password, cache)     # khoa cua tep goc
    accepted, by = [], {}
    for i in idx:
        for bit in range(8):
            v = bytearray(data); v[i] ^= 1 << bit
            try:
                pt, _ = decrypt2(bytes(v), password, cache)
                unpack_strict(pt)
                accepted.append((i, bit))
                code = "ACCEPTED"
            except Reject as e:
                code = str(e)
            by[code] = by.get(code, 0) + 1
    res = {"file": os.path.basename(path), "variants": len(idx) * 8, "accepted": accepted, "by_code": by}
    json.dump(res, open(outp, "w"), indent=1)
    print(json.dumps(res)[:600])


if __name__ == "__main__":
    if sys.argv[1] == "vectors":
        print(json.dumps(vectors(), indent=1))
    elif sys.argv[1] == "decrypt":
        rep, _, _ = decrypt_report(sys.argv[2], sys.argv[3], *(sys.argv[4:6]))
        print(json.dumps(rep, ensure_ascii=False, indent=1))
    elif sys.argv[1] == "flips":
        flips(sys.argv[2], sys.argv[3], sys.argv[4])
