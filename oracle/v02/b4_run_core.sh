#!/bin/bash
# Chay dump cua loi (hop den, --method jpl) cho cac tieu chi B4. Cach dung:
#   bash b4_run_core.sh <lvhv-memorial-dump> <r3-dataset.json> <r13-linked.json> <nhan commit>
set -e
B="$1"; DS="$2"; LK="$3"; C="$4"
H="$(cd "$(dirname "$0")" && pwd)"; O="$H/out"
"$B" --method jpl upcoming "$DS" 2026-01-01 2036-12-31 > "$O/b4_core_r11_$C.csv"
"$B" --method jpl upcoming "$LK" 2026-01-01 2036-12-31 > "$O/b4_core_r11_linked_$C.csv"
"$B" --method jpl r12 "$DS" --steps > "$O/b4_core_r12_$C.csv"
"$B" --method jpl r13 "$DS" > "$O/b4_core_r13_r3_$C.jsonl"
"$B" --method jpl r13 "$LK" > "$O/b4_core_r13_linked_$C.jsonl"
"$B" --method jpl reminders "$DS" --global-off > "$O/b4_core_r3_global_off_$C.csv"
"$B" --method jpl reminders "$DS" --rule-off > "$O/b4_core_r3_rule_off_$C.csv"
"$B" --method jpl ics "$DS" 2026-11-06 20261106T031500Z > "$O/b4_core_r4_r3_$C.ics"
"$B" --method jpl ics "$LK" 2026-11-06 20261106T031500Z > "$O/b4_core_r4_linked_$C.ics"
: > "$O/b4_core_r3_reminders_$C.sha"
for f in "$DS" "$LK"; do
  h=$("$B" --method jpl reminders "$f" | sha256sum | cut -d' ' -f1)
  n=$("$B" --method jpl reminders "$f" | wc -l)
  echo "$h  uncompressed:reminders $(basename "$f" .json) ($n dong gom tieu de)" >> "$O/b4_core_r3_reminders_$C.sha"
done
echo XONG
