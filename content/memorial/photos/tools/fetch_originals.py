"""Step 1: download the chosen REAL photos into originals/ and pin their sha256 in ledger.json.

First download of an entry: the sha256 field is empty -> it is filled in (recorded).
Later runs: the file is downloaded again only if missing, and its sha256 must match the ledger, otherwise the
run fails. Commons entries are re-checked against the API: licence must still be CC0 / PD / CC BY / CC BY-SA.
Large originals are fetched as the provider's own scaled rendition (long side <= 2400 px) to keep the repo
small; the URL actually downloaded is recorded in `download_url`.
"""
import pathlib
import sys

from common import ROOT, commons_fileinfo, http_get, licence_ok, load, save, sha256

ORIG = ROOT / "originals"
ORIG.mkdir(exist_ok=True)
MAX_SIDE = 2400


def download_url(e):
    if e["source"] == "commons":
        info = commons_fileinfo(e["commons_title"], thumb_width=MAX_SIDE)
        if not licence_ok(info["licence"]) or info["ai_category"]:
            raise SystemExit(f"Licence no longer acceptable for {e['id']}: {info['licence']}")
        e["licence"], e["licence_url"] = info["licence"], info["licence_url"]
        e["photographer"] = e.get("photographer") or info["photographer"]
        big = max(info["width"] or 0, info["height"] or 0)
        return info["thumb_url"] if big > MAX_SIDE and info.get("thumb_url") else info["original_url"]
    if e["source"] == "unsplash":
        return e["original_url"].split("?")[0] + f"?w={MAX_SIDE}&q=90&fm=jpg"
    return e["original_url"]


def main():
    ledger = load("ledger.json")
    bad = []
    for e in ledger["photos"]:
        if not e.get("chosen"):
            continue
        dst = ORIG / e["file"]
        if not dst.exists():
            url = download_url(e)
            e["download_url"] = url
            dst.write_bytes(http_get(url))
        h = sha256(dst)
        if not e.get("sha256"):
            e["sha256"] = h
            print("recorded", e["file"], h)
        elif e["sha256"] != h:
            bad.append((e["file"], e["sha256"], h))
        e["stored_at"] = str(dst.relative_to(ROOT.parents[2]))
    save("ledger.json", ledger)
    if bad:
        for b in bad:
            print("SHA256 MISMATCH", *b, file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
