#!/usr/bin/env python3
"""Kiểm đồ thị phụ thuộc mà cargo-deny quét (job CI "deps") phủ **mọi** gói trong Cargo.lock của workspace gốc.

cargo-deny chỉ quét các gói có trong đồ thị của `cargo metadata` với cấu hình `[graph]` của deny.toml. Trước đây
`all-features = false` bỏ sót các gói chỉ đến qua tính năng không mặc định (ví dụ `lvhv-ffi/bindgen-cli` → uniffi_bindgen
→ askama → basic-toml, RUSTSEC-2026-0374), nên cảnh báo không hiện. Phép kiểm này bảo đảm không gói nào trong Cargo.lock
nằm ngoài đồ thị (mọi tính năng, mọi nền tảng), tức là mọi cảnh báo RustSec của Cargo.lock đều được cargo-deny xét.

Dùng: python3 scripts/check-lock-coverage.py   (cần cargo; chạy ở gốc kho mã)
"""
import json
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# Cùng tùy chọn đồ thị với cargo-deny (deny.toml [graph]).
g = tomllib.loads((ROOT / "deny.toml").read_text(encoding="utf-8")).get("graph", {})
# `cargo metadata` tỉa theo tính năng ít hơn cargo-deny, nên phép so dưới đây là điều kiện cần. Điều kiện đủ: cargo-deny
# quét với mọi tính năng (gồm bindgen-cli, test-hooks).
if g.get("all-features") is not True or g.get("no-default-features"):
    print("::error::deny.toml [graph] phải có all-features = true (và không no-default-features) để cargo-deny xét "
          "cả phụ thuộc lúc dựng chỉ đến qua tính năng không mặc định")
    sys.exit(1)
args = ["--all-features"] if g.get("all-features") else []
if g.get("no-default-features"):
    args.append("--no-default-features")
if g.get("features"):
    args += ["--features", ",".join(g["features"])]
meta = json.loads(subprocess.run(
    ["cargo", "metadata", "--locked", "--format-version", "1", *args],
    cwd=ROOT, check=True, capture_output=True, text=True).stdout)
pkg = {p["id"]: (p["name"], p["version"]) for p in meta["packages"]}
graph = {pkg[n["id"]] for n in meta["resolve"]["nodes"]}
lock = {(p["name"], p["version"]) for p in tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))["package"]}
missing = sorted(lock - graph)
if missing:
    for n, v in missing:
        print(f"::error::{n} {v} có trong Cargo.lock nhưng nằm ngoài đồ thị cargo-deny quét")
    sys.exit(1)
print(f"đồ thị cargo-deny phủ {len(lock)}/{len(lock)} gói của Cargo.lock")
