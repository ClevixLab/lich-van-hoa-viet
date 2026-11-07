"""Chọn UDID một máy mô phỏng iOS có sẵn: python3 pick_sim.py <tiền tố tên> [tên ưu tiên…]."""
import json
import subprocess
import sys

want, prefs = sys.argv[1], sys.argv[2:]
out = subprocess.run(["xcrun", "simctl", "list", "devices", "available", "-j"], capture_output=True, text=True).stdout
devs = [x for k, v in json.loads(out)["devices"].items() if "iOS" in k for x in v if x["name"].startswith(want)]
for p in prefs:
    for x in devs:
        if x["name"] == p:
            print(x["udid"])
            sys.exit(0)
print(devs[-1]["udid"] if devs else "")
