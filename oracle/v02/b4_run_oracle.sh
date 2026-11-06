#!/bin/bash
# Sinh dau ra oracle theo Bo sung B4 (LVHV_B4=1). Cach dung: bash b4_run_oracle.sh <r3-dataset.json> <r13-linked.json>
# Ghi vao oracle/v02/out/b4_*; R3 day du chi ghi sha256 + so dong (tep 38 trieu dong khong luu).
set -e
export LVHV_B4=1
H="$(cd "$(dirname "$0")" && pwd)"; O="$H/out"
DS="$1"; LK="$2"
python3 "$H/b2_r11.py" "$DS" 2026-01-01 2036-12-31 > "$O/b4_r11_upcoming.csv"
python3 "$H/b2_r11.py" "$LK" 2026-01-01 2036-12-31 > "$O/b4_r11_upcoming_linked.csv"
python3 "$H/b3_r12.py" "$DS" --steps > "$O/b4_r12.csv"
python3 "$H/b2_r13.py" "$DS" > "$O/b4_r13_r3.jsonl"
python3 "$H/b2_r13.py" "$LK" > "$O/b4_r13_linked.jsonl"
python3 "$H/b2_r13.py" --check "$DS" "$O/b4_r13_r3.jsonl" > "$O/b4_r13_r3_check_oracle.json"
python3 "$H/b2_r13.py" --check "$LK" "$O/b4_r13_linked.jsonl" > "$O/b4_r13_linked_check_oracle.json"
python3 "$H/b4_r4.py" expect "$DS" 2026-11-06 > "$O/b4_r4_expect_r3.tsv"
python3 "$H/b4_r4.py" expect "$LK" 2026-11-06 > "$O/b4_r4_expect_linked.tsv"
python3 "$H/b2_r3.py" "$DS" --global-off > "$O/b4_r3_global_off.csv"
python3 "$H/b2_r3.py" "$DS" --rule-off > "$O/b4_r3_rule_off.csv"
for f in "$DS" "$LK"; do
  n=$(basename "$f" .json)
  python3 "$H/b2_r3.py" "$f" | tee >(wc -l > "$O/.b4_r3_$n.wc") | sha256sum | cut -d' ' -f1 > "$O/.b4_r3_$n.sha"
  sleep 1
  echo "$(cat "$O/.b4_r3_$n.sha")  uncompressed:reminders $n ($(cat "$O/.b4_r3_$n.wc") dong gom tieu de)" >> "$O/b4_r3_reminders.sha.tmp"
  rm -f "$O/.b4_r3_$n.sha" "$O/.b4_r3_$n.wc"
done
mv "$O/b4_r3_reminders.sha.tmp" "$O/b4_r3_reminders.sha"
echo XONG
