#!/bin/sh
# Tong bam cua dau ra oracle B2: sha256 tep, va sha256 cua noi dung da giai nen + sap (LC_ALL=C) cho tep .jsonl.gz
# Cach dung: sh b2_sums.sh <tep ra> <tep>...
set -e
export LC_ALL=C
OUTF="$1"; shift
: > "$OUTF.tmp"
for f in "$@"; do
  sha256sum "$f" >> "$OUTF.tmp"
  case "$f" in
    *.gz) h=$(zcat "$f" | sort -S 1500M -T "${TMPDIR:-/tmp}" | sha256sum | cut -d' ' -f1)
          echo "$h  sorted-uncompressed:$f" >> "$OUTF.tmp" ;;
  esac
done
mv "$OUTF.tmp" "$OUTF"
