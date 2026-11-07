# -*- coding: utf-8 -*-
"""Goi cong cu hop den lvhv-store-tool (Rust hoac TS) va tra JSON. Chi dung giao dien da ghi o docs/v0.2/10."""
import json, os, subprocess


class Tool:
    def __init__(self, build, kind):
        self.build, self.kind = build, kind

    def run(self, *args, stdin=None):
        if self.kind == "rust":
            cmd = [os.path.join(self.build, "target/release/lvhv-store-tool"), *args]
            cwd = self.build
        else:
            cmd = ["node", "test/tools/lvhv-store-tool.mjs", *args]
            cwd = os.path.join(self.build, "pwa")
        p = subprocess.run(cmd, cwd=cwd, input=stdin, capture_output=True, text=True, timeout=900)
        line = p.stdout.strip().splitlines()[-1] if p.stdout.strip() else ""
        try:
            out = json.loads(line)
        except Exception:
            out = {"ok": False, "parse_error": True, "stdout": p.stdout[-400:], "stderr": p.stderr[-400:]}
        out["_exit"] = p.returncode
        return out


if __name__ == "__main__":
    import sys
    t = Tool(sys.argv[1], sys.argv[2])
    r = t.run(*sys.argv[3:])
    def shape(o, d=0):
        if isinstance(o, dict) and d < 2:
            return {k: shape(v, d + 1) for k, v in o.items()}
        return type(o).__name__ if isinstance(o, (dict, list)) else o
    print(json.dumps(shape(r), ensure_ascii=False, indent=1)[:4000])
