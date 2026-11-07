#!/usr/bin/env python3
"""Bộ đóng gói / giải mã tệp sao lưu độc lập, viết chỉ từ 03 §2.1–§2.3 (không đọc mã cài đặt).

Phần đầu 72 byte LE: magic "LVHVBAK\\0", file_version u16=1, kdf_id=1, aead_id=1, m,t,p u32, salt 16, nonce 24,
plaintext_len u64. AAD = 72 byte đầu. Khóa = Argon2id(NFC(mật khẩu) UTF-8, salt, m, t, p, 32), v0x13.
AEAD = XChaCha20-Poly1305 (thẻ 16 byte nối sau bản mã).
Bản rõ = "LVHVPT01" ‖ u32 json_len ‖ json ‖ u32 n ‖ n × (UUID 16 byte ‖ u32 len ‖ byte).
"""
import hashlib, json, os, struct, unicodedata, uuid
from argon2.low_level import hash_secret_raw, Type
from nacl.bindings import (crypto_aead_xchacha20poly1305_ietf_encrypt as _enc,
                           crypto_aead_xchacha20poly1305_ietf_decrypt as _dec)

M, T, P = 65536, 3, 4


def _key(pw, salt, m=M, t=T, p=P):
    return hash_secret_raw(unicodedata.normalize("NFC", pw).encode(), salt, time_cost=t, memory_cost=m,
                           parallelism=p, hash_len=32, type=Type.ID, version=19)


def canon(doc):
    return json.dumps(doc, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def pack_plain(doc, blobs):
    js = canon(doc).encode()
    out = b"LVHVPT01" + struct.pack("<I", len(js)) + js + struct.pack("<I", len(blobs))
    for bid, data in blobs:
        out += uuid.UUID(bid).bytes + struct.pack("<I", len(data)) + data
    return out


def unpack_plain(pt):
    assert pt[:8] == b"LVHVPT01", pt[:8]
    (jl,) = struct.unpack_from("<I", pt, 8)
    js = pt[12:12 + jl]
    o = 12 + jl
    (n,) = struct.unpack_from("<I", pt, o); o += 4
    blobs = []
    for _ in range(n):
        bid = str(uuid.UUID(bytes=pt[o:o + 16])); o += 16
        (ln,) = struct.unpack_from("<I", pt, o); o += 4
        blobs.append((bid, pt[o:o + ln])); o += ln
    assert o == len(pt)
    return js, blobs


def encrypt(pt, pw, salt=None, nonce=None):
    salt = salt or os.urandom(16)
    nonce = nonce or os.urandom(24)
    hdr = (b"LVHVBAK\x00" + struct.pack("<HBBIII", 1, 1, 1, M, T, P) + salt + nonce
           + struct.pack("<Q", len(pt)))
    assert len(hdr) == 72
    return hdr + _enc(pt, hdr, nonce, _key(pw, salt))


def decrypt(data, pw):
    hdr = data[:72]
    assert hdr[:8] == b"LVHVBAK\x00"
    fv, kdf, aead, m, t, p = struct.unpack_from("<HBBIII", hdr, 8)
    salt, nonce = hdr[24:40], hdr[40:64]
    (ptl,) = struct.unpack_from("<Q", hdr, 64)
    assert ptl == len(data) - 72 - 16
    return _dec(data[72:], hdr, nonce, _key(pw, salt, m, t, p))


def photo_record(pid, data, mime, w, h, created="2026-11-07T00:00:00Z"):
    return {"id": pid, "mime": mime, "width": w, "height": h, "bytes": len(data),
            "sha256": hashlib.sha256(data).hexdigest(), "stripped": True, "created_at": created}
