#!/usr/bin/env python3
"""Dựng ảnh thử cho kiểm định R15 (c)(d) — chỉ từ đặc tả (01 §3.7/§3.8, 03 §2.3, 04 B6/B8).

Dùng: python3 -I make_inputs.py <thư-mục-ra>
Sinh tất định (không ngẫu nhiên ngoài seed cố định). In manifest.json: tên, mime, rộng, cao, bytes, sha256.
"""
import hashlib, io, json, random, struct, sys, os
from PIL import Image

out = sys.argv[1]
os.makedirs(out, exist_ok=True)
man = {}


def grad(w, h, seed=0):
    im = Image.new("RGB", (w, h))
    px = im.load()
    for y in range(h):
        for x in range(w):
            px[x, y] = ((x * 7 + seed) % 256, (y * 5 + seed * 3) % 256, ((x + y) * 3) % 256)
    return im


def put(name, data, mime, w, h):
    with open(os.path.join(out, name), "wb") as f:
        f.write(data)
    man[name] = {"mime": mime, "width": w, "height": h, "bytes": len(data),
                 "sha256": hashlib.sha256(data).hexdigest()}


def jpeg(im, **kw):
    b = io.BytesIO()
    im.save(b, "JPEG", quality=kw.pop("quality", 85), **kw)
    return b.getvalue()


def webp(im, **kw):
    b = io.BytesIO()
    im.save(b, "WEBP", quality=80, **kw)
    return b.getvalue()


def exif_with_gps():
    ex = Image.Exif()
    ex[0x010F] = "OracleCam"          # Make
    ex[0x0110] = "R15"                # Model
    gps = ex.get_ifd(0x8825)
    gps[1] = "N"; gps[2] = (21.0, 1.0, 42.0)   # Hà Nội (giả)
    gps[3] = "E"; gps[4] = (105.0, 50.0, 0.0)
    return ex.tobytes()


XMP = (b'<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?><x:xmpmeta xmlns:x="adobe:ns:meta/">'
       b'<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"><rdf:Description '
       b'xmlns:exif="http://ns.adobe.com/exif/1.0/" exif:GPSLatitude="21,1.7N"/></rdf:RDF></x:xmpmeta>'
       b'<?xpacket end="w"?>')

im = grad(64, 48, 1)
put("clean.jpg", jpeg(im), "image/jpeg", 64, 48)
put("clean2.jpg", jpeg(grad(80, 60, 2)), "image/jpeg", 80, 60)
put("clean3.jpg", jpeg(grad(72, 40, 3)), "image/jpeg", 72, 40)
put("clean.webp", webp(grad(50, 70, 4)), "image/webp", 50, 70)
put("exif_gps.jpg", jpeg(im, exif=exif_with_gps()), "image/jpeg", 64, 48)
put("xmp.jpg", jpeg(im, xmp=XMP), "image/jpeg", 64, 48)
# APP13 (IPTC trong khối Photoshop) chèn tay ngay sau SOI
base = jpeg(im)
iptc = b"\x1c\x02\x05" + struct.pack(">H", 9) + b"OracleR15"
res = b"8BIM" + struct.pack(">H", 0x0404) + b"\x00\x00" + struct.pack(">I", len(iptc)) + iptc
if len(iptc) % 2:
    res += b"\x00"
seg = b"Photoshop 3.0\x00" + res
put("app13.jpg", base[:2] + b"\xff\xed" + struct.pack(">H", len(seg) + 2) + seg + base[2:], "image/jpeg", 64, 48)
put("exif.webp", webp(grad(50, 70, 4), exif=exif_with_gps()), "image/webp", 50, 70)
# ảnh sạch > 1 MiB, cạnh ≤ 2048
rnd = random.Random(15)
noise = Image.frombytes("RGB", (1400, 1400), bytes(rnd.getrandbits(8) for _ in range(1400 * 1400 * 3)))
put("big.jpg", jpeg(noise, quality=95), "image/jpeg", 1400, 1400)
# ảnh sạch cạnh dài 2000 px (thăm dò c5)
put("wide2000.jpg", jpeg(grad(2000, 120, 5)), "image/jpeg", 2000, 120)
# "ảnh" giả
put("fake.jpg", b"<!doctype html><html><body><script>alert(1)</script></body></html>\n", "image/jpeg", 64, 48)

assert man["big.jpg"]["bytes"] > 1048576, man["big.jpg"]
with open(os.path.join(out, "manifest.json"), "w") as f:
    json.dump(man, f, indent=1, sort_keys=True)
print(json.dumps(man, indent=1, sort_keys=True))
