import json, os, sys, unicodedata, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sources import SOURCES
TYPES = {"cd": "ca dao", "tn": "tục ngữ", "th": "thành ngữ"}
def norm(s):
    s = unicodedata.normalize('NFD', s.lower()).replace('đ', 'd')
    s = ''.join(c for c in s if unicodedata.category(c) != 'Mn')
    return re.sub(r'\s+', ' ', re.sub(r'[^\w\s]', ' ', s)).strip()
def xorshift32(x):
    x ^= (x << 13) & 0xFFFFFFFF; x ^= x >> 17; x ^= (x << 5) & 0xFFFFFFFF
    return x & 0xFFFFFFFF
entries, seen = [], set()
for ln in open(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'data.txt'), encoding='utf-8'):
    ln = ln.rstrip('\n')
    if not ln.strip() or ln.startswith('#'):
        continue
    t, theme, srcs, text = ln.split('|', 3)
    text = unicodedata.normalize('NFC', text.strip())
    lines = [p.strip() for p in text.split(' / ')]
    k = norm(' '.join(lines))
    if k in seen:
        print('DUP skipped:', text); continue
    seen.add(k)
    src = []
    for key in srcs.split(','):
        ti, url, lic = SOURCES[key.strip()]
        src.append({"title": ti, "url": url, "license": lic})
    joined = ' / '.join(lines)
    entries.append({"type": TYPES[t], "text": joined if len(lines) > 1 else lines[0],
                    "lines": lines, "theme": theme, "meaning": "", "sources": src})
for i, e in enumerate(entries, 1):
    e["id"] = f"cd{i:04d}"
# order: Fisher–Yates with xorshift32, seed 0x4C564E31 ("LVN1")
N = len(entries); perm = list(range(N)); x = 0x4C564E31
for i in range(N - 1, 0, -1):
    x = xorshift32(x); j = x % (i + 1); perm[i], perm[j] = perm[j], perm[i]
for pos, idx in enumerate(perm):
    entries[idx]["order"] = pos
keys = ["id", "type", "text", "lines", "theme", "meaning", "sources", "order"]
out = [{k: e[k] for k in keys} for e in entries]
json.dump(out, open(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'cadao.json'), 'w', encoding='utf-8'), ensure_ascii=False, indent=1)
print('entries', N)
