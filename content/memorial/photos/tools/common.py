"""Shared helpers for the memorial tray photo pipeline (runs on the CI runner only)."""
import hashlib
import json
import pathlib
import re
import time
import urllib.parse
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[1]  # content/memorial/photos
UA = "LichVanHoaViet-memorial-assets/1.0 (+https://github.com/ClevixLab/lich-van-hoa-viet)"
COMMONS_API = "https://commons.wikimedia.org/w/api.php"

# Licences accepted for the composite (YC-2026-09, coordinator 07/11/2026).
ALLOWED_LICENCE = re.compile(
    r"^(cc0|cc0 1\.0|public domain|pd|pdm|cc[ -]by[ -](sa[ -])?\d\.\d(.*)?|cc[ -]by[ -](sa[ -])?\d\.\d)$",
    re.I,
)


def http_get(url, binary=True, retries=4):
    last = None
    for i in range(retries):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": UA})
            with urllib.request.urlopen(req, timeout=60) as r:
                data = r.read()
            return data if binary else data.decode("utf-8")
        except Exception as e:  # noqa: BLE001
            last = e
            time.sleep(2 * (i + 1))
    raise RuntimeError(f"GET failed {url}: {last}")


def commons_api(params):
    params = dict(params, format="json", formatversion="2")
    return json.loads(http_get(COMMONS_API + "?" + urllib.parse.urlencode(params), binary=False))


def strip_html(s):
    s = re.sub(r"<[^>]+>", "", s or "")
    return re.sub(r"\s+", " ", s).strip()


def commons_fileinfo(title, thumb_width=None):
    """Return licence/author metadata for one File: page straight from the Commons API."""
    p = {
        "action": "query",
        "titles": title,
        "prop": "imageinfo|categories",
        "iiprop": "url|size|mime|sha1|extmetadata",
        "cllimit": "max",
    }
    if thumb_width:
        p["iiurlwidth"] = str(thumb_width)
    page = commons_api(p)["query"]["pages"][0]
    ii = page["imageinfo"][0]
    md = ii.get("extmetadata", {})
    cats = [c["title"] for c in page.get("categories", [])]
    return {
        "title": page["title"],
        "page_url": ii.get("descriptionurl"),
        "original_url": ii.get("url"),
        "thumb_url": ii.get("thumburl"),
        "width": ii.get("width"),
        "height": ii.get("height"),
        "mime": ii.get("mime"),
        "commons_sha1": ii.get("sha1"),
        "photographer": strip_html(md.get("Artist", {}).get("value", "")),
        "credit": strip_html(md.get("Credit", {}).get("value", "")),
        "licence": strip_html(md.get("LicenseShortName", {}).get("value", "")),
        "licence_url": strip_html(md.get("LicenseUrl", {}).get("value", "")),
        "date": strip_html(md.get("DateTimeOriginal", {}).get("value", "")),
        "description": strip_html(md.get("ImageDescription", {}).get("value", ""))[:300],
        "ai_category": any("AI-generated" in c or "AI generated" in c for c in cats),
    }


def licence_ok(name):
    return bool(name) and bool(ALLOWED_LICENCE.match(name.strip()))


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def load(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def save(name, obj):
    (ROOT / name).write_text(json.dumps(obj, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
