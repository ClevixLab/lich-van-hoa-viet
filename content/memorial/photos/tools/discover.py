"""Step 0: find candidate REAL photos and build one contact sheet per item for a human to look at.

Commons: searched through the API; only files whose licence (from the file page metadata) is CC0, public
domain, CC BY or CC BY-SA are kept; files in an "AI-generated" category are dropped.
Direct candidates (Unsplash / Pixabay) were checked by hand on their photo pages; only a small thumbnail is
downloaded here so they can be compared on the same sheet.
Output: discover/<item>.jpg (contact sheet) and discover/candidates.json.
"""
import io
import json

from PIL import Image, ImageDraw

from common import ROOT, commons_api, commons_fileinfo, http_get, licence_ok, load

THUMB = 300
OUT = ROOT / "discover"
OUT.mkdir(exist_ok=True)


def commons_search(query, limit):
    r = commons_api({
        "action": "query", "list": "search", "srsearch": query, "srnamespace": "6", "srlimit": str(limit),
    })
    return [h["title"] for h in r.get("query", {}).get("search", [])]


def sheet(item, cands):
    cols = 4
    rows = max(1, (len(cands) + cols - 1) // cols)
    W, H = cols * (THUMB + 10) + 10, rows * (THUMB + 40) + 10
    img = Image.new("RGB", (W, H), (40, 40, 40))
    d = ImageDraw.Draw(img)
    for i, c in enumerate(cands):
        x, y = 10 + (i % cols) * (THUMB + 10), 10 + (i // cols) * (THUMB + 40)
        try:
            t = Image.open(io.BytesIO(http_get(c["thumb_url"]))).convert("RGB")
            t.thumbnail((THUMB, THUMB))
            img.paste(t, (x, y))
        except Exception as e:  # noqa: BLE001
            d.text((x, y + 10), f"lỗi tải: {e}"[:40], fill=(255, 80, 80))
        d.text((x, y + THUMB + 4), f"#{i} {c['source']} {c.get('licence','')}"[:48], fill=(255, 255, 255))
        d.text((x, y + THUMB + 20), c["key"][:48], fill=(200, 200, 200))
    img.save(OUT / f"{item}.jpg", quality=80)


def main():
    spec = load("discover.json")
    result = {}
    for item, cfg in spec["items"].items():
        cands, seen = [], set()
        for d in cfg.get("direct", []):
            cands.append(dict(d, key=d["page_url"].rstrip("/").split("/")[-1]))
        for q in cfg.get("commons_queries", []):
            for title in commons_search(q, cfg.get("per_query", 6)):
                if title in seen:
                    continue
                seen.add(title)
                try:
                    info = commons_fileinfo(title, thumb_width=THUMB)
                except Exception as e:  # noqa: BLE001
                    print("skip", title, e)
                    continue
                if info["mime"] not in ("image/jpeg", "image/png"):
                    continue
                if info["ai_category"] or not licence_ok(info["licence"]):
                    print("reject licence/AI", title, info["licence"])
                    continue
                cands.append(dict(info, source="commons", key=title.replace("File:", "")))
        cands = cands[: cfg.get("max", 16)]
        result[item] = cands
        sheet(item, cands)
        print(item, len(cands))
    (OUT / "candidates.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
