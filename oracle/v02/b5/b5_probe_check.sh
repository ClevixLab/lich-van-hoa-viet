#!/bin/bash
# Do hop den duong mo tep sao luu cua loi: dat tung tep thu vao cho ma vi du `cross_platform check` doc
# (duong dan lay tu thong bao loi cua chinh chuong trinh), chay, ghi dong ket qua/loi dau tien.
# Cach dung: bash b5_probe_check.sh <thu muc ban dung> <thu muc tep thu> [thu muc tep lon]
B="$1"; FX="$2"; BIG="$3"
T="$B/pwa/test/store/out/r6-from-pwa.lvhv"
mkdir -p "$(dirname "$T")"
for f in "$FX"/*.lvhv ${BIG:+"$BIG"/*.lvhv}; do
  cp "$f" "$T"
  r=$(cd "$B" && ./target/release/examples/cross_platform check 2>&1 | grep -v '^ \|^stack\|^note' | head -3 | tr '\n' ' ' | cut -c1-300)
  echo "$(basename "$f" .lvhv)	$r"
done
rm -f "$T"
