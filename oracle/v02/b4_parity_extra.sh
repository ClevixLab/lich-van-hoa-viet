#!/bin/bash
# Rust ≡ TS cho cac lenh B4 chua co trong b2_parity.sh: ics (hai bo), upcoming bo r13-linked.
# Cach dung: bash b4_parity_extra.sh <lvhv-memorial-dump> "<node .../memorial_dump.mjs>" <r3-dataset> <r13-linked> <tep ra>
R="$1"; T="$2"; DS="$3"; LK="$4"; OUTF="$5"
cmds=("ics $DS 2026-11-06 20261106T031500Z" "ics $LK 2026-11-06 20261106T031500Z" "upcoming $LK 2026-01-01 2036-12-31")
: > "$OUTF"
for c in "${cmds[@]}"; do
  # shellcheck disable=SC2086
  a=$($R --method jpl $c | sha256sum | cut -d' ' -f1)
  # shellcheck disable=SC2086
  b=$($T --method jpl $c | sha256sum | cut -d' ' -f1)
  if [[ "$a" == "$b" ]]; then v=GIONG; else v=KHAC; fi
  echo "$v	--method jpl $(echo "$c" | sed 's#/[^ ]*/##g')	rust=$a	ts=$b" >> "$OUTF"
done
