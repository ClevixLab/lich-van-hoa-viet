#!/usr/bin/env python3
"""Ve lai dong oracle theo dinh dang 'death' cua loi ({calendar,precision,year,month,day,leap}) de thu so tung byte."""
import gzip, json, sys
for l in gzip.open(sys.argv[1], "rt", encoding="utf-8"):
    o = json.loads(l); d = o["death"]
    if d["cal"] == "solar":
        dd = {"calendar": "solar", "precision": "day", "year": d["y"], "month": d["m"], "day": d["d"], "leap": False}
    else:
        dd = {"calendar": "lunar", "precision": "day" if d["y"] is not None else "month_day", "year": d["y"],
              "month": d["m"], "day": d["d"], "leap": d["leap"]}
    o["death"] = dd
    sys.stdout.write(json.dumps(o, ensure_ascii=False, separators=(",", ":")) + "\n")
