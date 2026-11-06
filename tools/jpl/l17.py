#!/usr/bin/env python3
"""L17 (docs/02 Bổ sung A4): chạy lại L9 (Tết 1900–1999 so nguồn Việt Nam, cách chấm A3) với `Jpl`.

Cách dùng: python3 tools/jpl/l17.py --dump target/release/lvhv-dump
In bảng Markdown ra stdout. Phần TK20 (1237 tháng) do test `core/tests/range1900.rs::tk20_month_starts_1900_1999`
kiểm (Jpl: 0 tháng lệch).
"""
import argparse
import csv
import os
import subprocess
from collections import defaultdict

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..')
# A3 / biên bản G1–G4: năm nguồn Việt Nam mâu thuẫn → chấm theo nguồn tuân thủ quy ước lịch sử (xemlicham; 1925:
# trang ngày của xemlicham).
A3_CHOICE = {
    1903: 'https://www.xemlicham.com/am-lich/nam/1903',
    1925: 'https://www.xemlicham.com/am-lich/nam/1925/thang/1/ngay/24',
    1935: 'https://www.xemlicham.com/am-lich/nam/1935',
    1965: 'https://www.xemlicham.com/am-lich/nam/1965',
}


def tet(dump, method):
    out = subprocess.run([dump, '--method', method, 'tet', '1900', '1999'], check=True, capture_output=True,
                         text=True).stdout.splitlines()[1:]
    return {int(l.split(',')[0]): l.split(',')[1] for l in out}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--dump', required=True)
    a = ap.parse_args()
    tj = tet(a.dump, 'jpl')
    tp = tet(a.dump, 'precise')
    src = defaultdict(list)
    with open(os.path.join(ROOT, 'oracle', 'reference', 'tet_published_1900_1999.csv'), encoding='utf-8') as f:
        for r in csv.DictReader(f):
            src[int(r['lunar_year'])].append(r)
    per_source = defaultdict(lambda: [0, 0, []])
    agree_years = conflict_years = 0
    bad = []
    for y in range(1900, 2000):
        vn = [r for r in src[y] if r['source_id'].startswith('VN_')]
        for r in src[y]:
            per_source[r['source_id']][1] += 1
            per_source[r['source_id']][0] += r['tet_date'] == tj[y]
            if r['tet_date'] != tj[y]:
                per_source[r['source_id']][2].append(f"{y} ({r['tet_date']})")
        dates = {r['tet_date'] for r in vn}
        if not vn:
            bad.append((y, 'không có nguồn VN'))
        elif len(dates) == 1:
            agree_years += 1
            if tj[y] not in dates:
                bad.append((y, f'Jpl {tj[y]} ≠ nguồn {dates}'))
        else:
            conflict_years += 1
            chosen = [r for r in vn if r['source'] == A3_CHOICE.get(y)]
            if not chosen or chosen[0]['tet_date'] != tj[y]:
                bad.append((y, f'năm mâu thuẫn, Jpl {tj[y]}, nguồn theo A3 {chosen[0]["tet_date"] if chosen else "?"}'))
    same = sum(tj[y] == tp[y] for y in tj)
    print(f'- Tết Jpl = Tết Precise: {same}/100 năm.')
    print(f'- Năm nguồn VN thống nhất: {agree_years}; năm mâu thuẫn: {conflict_years} (chấm theo A3).')
    print('- Khớp theo từng nguồn (số dòng khớp / số dòng):')
    for k in sorted(per_source):
        miss = per_source[k][2]
        print(f'  - {k}: {per_source[k][0]}/{per_source[k][1]}' + (f' — lệch: {", ".join(miss)}' if miss else ''))
    print(f'- Sai lệch theo cách chấm A3: {len(bad)} {bad}')
    print(f'- L9 với Jpl: {"ĐẠT (theo A3)" if not bad else "KHÔNG ĐẠT"}')


if __name__ == '__main__':
    main()
