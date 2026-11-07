# -*- coding: utf-8 -*-
"""Doc/ghi tep sao luu .lvhv doc lap theo docs/v0.2/03 §2 (ben kiem dinh, viet tu dac ta).

Argon2id (RFC 9106, v0x13) va XChaCha20-Poly1305 (HChaCha20 + ChaCha20-Poly1305 IETF) qua thu vien
`cryptography`; HChaCha20 tu cai theo draft-irtf-cfrg-xchacha §2.2. Khong doc ma cua ben xay dung.

Ban ro (03 §2.3): "LVHVPT01" ‖ u32 json_len ‖ json ‖ u32 n ‖ n × (16 byte UUID ‖ u32 len ‖ byte).
Gia dinh A-C1 (dang ky truoc): byte cua tep dinh kem (01 §3.8a) di trong cung danh sach muc nhi phan
voi anh, khop theo id cua ban ghi Attachment. Neu dac ta cua ben xay dung khac, chi sua ham pack/unpack
qua tham so `layout` va ghi lai sau doi chieu.
"""
import json, struct, unicodedata, uuid

from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305
from cryptography.hazmat.primitives.kdf.argon2 import Argon2id

MAGIC = b"LVHVBAK\x00"
PT_MAGIC = b"LVHVPT01"


def _rotl(v, c):
    return ((v << c) & 0xFFFFFFFF) | (v >> (32 - c))


def _qr(s, a, b, c, d):
    s[a] = (s[a] + s[b]) & 0xFFFFFFFF; s[d] = _rotl(s[d] ^ s[a], 16)
    s[c] = (s[c] + s[d]) & 0xFFFFFFFF; s[b] = _rotl(s[b] ^ s[c], 12)
    s[a] = (s[a] + s[b]) & 0xFFFFFFFF; s[d] = _rotl(s[d] ^ s[a], 8)
    s[c] = (s[c] + s[d]) & 0xFFFFFFFF; s[b] = _rotl(s[b] ^ s[c], 7)


def hchacha20(key, nonce16):
    s = [0x61707865, 0x3320646E, 0x79622D32, 0x6B206574]
    s += list(struct.unpack("<8I", key)) + list(struct.unpack("<4I", nonce16))
    for _ in range(10):
        _qr(s, 0, 4, 8, 12); _qr(s, 1, 5, 9, 13); _qr(s, 2, 6, 10, 14); _qr(s, 3, 7, 11, 15)
        _qr(s, 0, 5, 10, 15); _qr(s, 1, 6, 11, 12); _qr(s, 2, 7, 8, 13); _qr(s, 3, 4, 9, 14)
    return struct.pack("<8I", *(s[0:4] + s[12:16]))


def xchacha_encrypt(key, nonce24, pt, aad):
    sub = hchacha20(key, nonce24[:16])
    return ChaCha20Poly1305(sub).encrypt(b"\x00" * 4 + nonce24[16:], pt, aad)


def xchacha_decrypt(key, nonce24, ct, aad):
    sub = hchacha20(key, nonce24[:16])
    return ChaCha20Poly1305(sub).decrypt(b"\x00" * 4 + nonce24[16:], ct, aad)


def kdf(password, salt, m, t, p):
    pw = unicodedata.normalize("NFC", password).encode("utf-8")
    return Argon2id(salt=salt, length=32, iterations=t, lanes=p, memory_cost=m).derive(pw)


def canonical_json(doc):
    # 03 B1.4 muc 9: khoa sap theo diem ma, khong khoang trang. (sap mang theo id do ben goi lam)
    return json.dumps(doc, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode("utf-8")


def pack(json_bytes, blobs):
    """blobs: danh sach (uuid_str, bytes) theo thu tu da sap."""
    out = [PT_MAGIC, struct.pack("<I", len(json_bytes)), json_bytes, struct.pack("<I", len(blobs))]
    for i, b in blobs:
        out += [uuid.UUID(i).bytes, struct.pack("<I", len(b)), b]
    return b"".join(out)


def unpack(pt):
    if pt[:8] != PT_MAGIC:
        raise ValueError("malformed_plaintext")
    (jl,) = struct.unpack_from("<I", pt, 8)
    j = pt[12:12 + jl]
    o = 12 + jl
    (n,) = struct.unpack_from("<I", pt, o); o += 4
    blobs = []
    for _ in range(n):
        u = str(uuid.UUID(bytes=pt[o:o + 16])); o += 16
        (ln,) = struct.unpack_from("<I", pt, o); o += 4
        blobs.append((u, pt[o:o + ln])); o += ln
    if o != len(pt):
        raise ValueError("malformed_plaintext")
    return j, blobs


def encrypt_file(pt, password, salt, nonce, m=65536, t=3, p=4, file_version=1):
    hdr = MAGIC + struct.pack("<HBBIII", file_version, 1, 1, m, t, p) + salt + nonce + struct.pack("<Q", len(pt))
    assert len(hdr) == 72
    return hdr + xchacha_encrypt(kdf(password, salt, m, t, p), nonce, pt, hdr)


def decrypt_file(data, password):
    hdr = data[:72]
    if len(data) < 88 or hdr[:8] != MAGIC:
        raise ValueError("not_a_backup")
    fv, kid, aid, m, t, p = struct.unpack_from("<HBBIII", hdr, 8)
    if fv > 1:
        raise ValueError("newer_version")
    salt, nonce = hdr[24:40], hdr[40:64]
    return xchacha_decrypt(kdf(password, salt, m, t, p), nonce, data[72:], hdr)


if __name__ == "__main__":
    import sys, hashlib
    # Tu kiem: giai ma vector cua kho (03 §2.5) va dong goi lai phai ra dung tung byte.
    path, pw = sys.argv[1], sys.argv[2]
    data = open(path, "rb").read()
    pt = decrypt_file(data, pw)
    j, blobs = unpack(pt)
    doc = json.loads(j)
    print("schema_version", doc.get("schema_version"), "persons", len(doc["persons"]), "blobs", len(blobs))
    re = encrypt_file(pack(j, blobs), pw, data[24:40], data[40:64])
    print("re-encrypt identical:", re == data, hashlib.sha256(data).hexdigest()[:16])
    print("canonical json identical:", canonical_json(doc) == j)
