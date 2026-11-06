#!/bin/sh
# Chay cong cu dump cua loi (hop den) cho R1 voi --method jpl, ghi tep gzip vao thu muc $2.
# Cach dung: sh b2_run_core_r1.sh <lvhv-memorial-dump> <thu muc ra>
set -e
B="$1"; D="$2"; mkdir -p "$D"
"$B" --method jpl gio-dates solar 1900-01-01 3000-12-31 10 | gzip -3 > "$D/core_r1_jpl_a.jsonl.gz"
"$B" --method jpl gio-dates unknown 1900 3000 | gzip -3 > "$D/core_r1_jpl_b.jsonl.gz"
"$B" --method jpl gio-dates lunar 1900 2999 10 | gzip -3 > "$D/core_r1_jpl_c.jsonl.gz"
