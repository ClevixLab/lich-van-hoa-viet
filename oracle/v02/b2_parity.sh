#!/bin/bash
# Rust ≡ TypeScript tu phia ben kiem dinh: chay hai cong cu dump nhu hop den voi cung lenh, so sha256 dau ra.
# Cach dung: bash b2_parity.sh <lvhv-memorial-dump> "<node .../memorial_dump.mjs>" <r3-dataset.json> <tep ket qua>
# Moi lenh chay hai lan: co `--method jpl` va khong co co (mac dinh cua tung cong cu).
R="$1"; T="$2"; DS="$3"; OUTF="$4"
cmds=(
  "gio-dates solar 1900-01-01 3000-12-31 10"
  "gio-dates unknown 1900 3000"
  "gio-dates lunar 1900 2999 10"
  "--gio-het 3 gio-dates solar 1900-01-01 3000-12-31 10"
  "reminders configs"
  "reminders $DS"
  "upcoming $DS 2026-01-01 2036-12-31"
  "r12 $DS --steps"
)
: > "$OUTF"
for c in "${cmds[@]}"; do
  for m in "--method jpl " ""; do
    # shellcheck disable=SC2086
    a=$($R $m$c | sha256sum | cut -d' ' -f1)
    # shellcheck disable=SC2086
    b=$($T $m$c | sha256sum | cut -d' ' -f1)
    if [[ "$a" == "$b" ]]; then v=GIONG; else v=KHAC; fi
    echo "$v	$m$c	rust=$a	ts=$b" | sed "s#$DS#DATASET#" >> "$OUTF"
  done
done
echo XONG >> "$OUTF"
