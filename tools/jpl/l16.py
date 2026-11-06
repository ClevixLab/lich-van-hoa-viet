#!/usr/bin/env python3
"""L16 (docs/02 Bổ sung A4): liệt kê MỌI ngày mà `Jpl` khác `Precise`, 1900-01-01 → 3000-12-31.

Cách dùng:  python3 tools/jpl/l16.py --dump target/release/lvhv-dump \
                --baseline core/out/baseline-2000-3000/precise.jsonl.gz --out docs/reports/BAO-CAO-JPL-L16.md

So các trường: ngày âm (năm, tháng, ngày, nhuận), Can Chi năm/tháng, tiết khí đang hiệu lực (tức ngày bắt đầu
tiết khí). Precise lấy từ lõi hiện hành và được đối chiếu với tệp chuẩn L13 (2000–3000) trước khi dùng.
Mỗi đoạn ngày liên tiếp khác nhau được gán cho các sự kiện (Sóc k, tiết khí) mà hai phương pháp đặt khác ngày
lịch; đoạn không gán được → "nguyên nhân khác".
"""
import argparse
import datetime as dt
import gzip
import json
import subprocess
import sys
from collections import defaultdict


def run(dump, *args):
    r = subprocess.run([dump, *args], check=True, capture_output=True, text=True)
    return r.stdout.splitlines()


def lunar_rows(dump, method):
    rows = {}
    for line in run(dump, '--method', method, 'lunar-csv', '1900-01-01', '3000-12-31')[1:]:
        f = line.split(',')
        rows[f[0]] = f[1:]
    return rows


def parse_local(s):
    """'YYYY-MM-DDTHH:MM:SS+HH:MM[:SS]' → (datetime giờ lịch, độ lệch giây)."""
    base, off = s[:19], s[19:]
    sign = 1 if off[0] == '+' else -1
    p = [int(x) for x in off[1:].split(':')]
    secs = p[0] * 3600 + p[1] * 60 + (p[2] if len(p) > 2 else 0)
    return dt.datetime.fromisoformat(base), sign * secs


def events(dump, method):
    ev = {}
    for line in run(dump, '--method', method, 'newmoons', '1900', '3000')[1:]:
        k, s = line.split(',')
        ev[('Sóc', int(k))] = s
    seen = defaultdict(int)
    for line in run(dump, '--method', method, 'terms', '1900', '3000')[1:]:
        y, name, _, s = line.split(',')
        ev[('Tiết khí', f'{name} {y}')] = s
        seen[(name, y)] += 1
    return ev


def midnight_dist(s):
    d, _ = parse_local(s)
    sec = d.hour * 3600 + d.minute * 60 + d.second
    return min(sec, 86400 - sec)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--dump', required=True)
    ap.add_argument('--baseline', required=True)
    ap.add_argument('--out', required=True)
    a = ap.parse_args()

    pre = lunar_rows(a.dump, 'precise')
    jpl = lunar_rows(a.dump, 'jpl')
    assert len(pre) == len(jpl) == 402132, (len(pre), len(jpl))

    # Precise hiện hành ≡ tệp chuẩn L13 ở các trường đang so (2000–3000).
    nb = 0
    with gzip.open(a.baseline, 'rt', encoding='utf-8') as f:
        for line in f:
            o = json.loads(line)
            s = o['solar']
            key = f"{s['y']:04d}-{s['m']:02d}-{s['d']:02d}"
            l = o['lunar']
            want = [str(l['y']), str(l['m']), str(l['d']), '1' if l['leap'] else '0',
                    o['can_chi']['year'], o['can_chi']['month'], o['can_chi']['day'], o['tiet_khi']]
            assert pre[key] == want, (key, pre[key], want)
            nb += 1
    print(f'[l16] Precise hiện hành trùng tệp chuẩn L13 ở {nb} ngày (2000–3000)', file=sys.stderr)

    days = sorted(pre)
    diff = [d for d in days if pre[d] != jpl[d]]
    # Nhóm thành đoạn liên tiếp.
    segs = []
    prev = None
    for d in diff:
        dd = dt.date.fromisoformat(d)
        if prev is not None and (dd - prev).days == 1:
            segs[-1][1] = dd
        else:
            segs.append([dd, dd])
        prev = dd

    ep = events(a.dump, 'precise')
    ej = events(a.dump, 'jpl')
    disc = []  # sự kiện mà hai phương pháp đặt khác ngày lịch
    for key in ep:
        if key not in ej:
            continue
        dp, _ = parse_local(ep[key])
        dj, _ = parse_local(ej[key])
        if dp.date() != dj.date():
            disc.append((key, ep[key], ej[key], min(dp.date(), dj.date()), max(dp.date(), dj.date())))
    missing = [k for k in ep if k not in ej] + [k for k in ej if k not in ep]

    rows = []
    other = 0
    used = set()
    for s0, s1 in segs:
        # Sự kiện giải thích: ngày (theo một trong hai phương pháp) nằm trong đoạn [s0, s1 + 1].
        cause = [e for e in disc if not (e[4] < s0 or e[3] > s1 + dt.timedelta(days=1))]
        fields = set()
        cur = s0
        while cur <= s1:
            k = cur.isoformat()
            names = ['năm âm', 'tháng âm', 'ngày âm', 'nhuận', 'Can Chi năm', 'Can Chi tháng', 'Can Chi ngày',
                     'tiết khí']
            for i, n in enumerate(names):
                if pre[k][i] != jpl[k][i]:
                    fields.add(n)
            cur += dt.timedelta(days=1)
        if cause:
            kind = 'sát nửa đêm / hai phương pháp đặt sự kiện khác ngày'
            for e in cause:
                used.add(e[0])
        else:
            kind = 'nguyên nhân khác'
            other += 1
        rows.append((s0, s1, sorted(fields), cause, kind))

    unused = [e for e in disc if e[0] not in used]
    ndays = len(diff)
    with open(a.out, 'w', encoding='utf-8') as f:
        w = f.write
        w('# Báo cáo L16 — mọi ngày `Jpl` khác `Precise`, 1900–3000 (docs/02 Bổ sung A4)\n\n')
        w('Sinh tự động bởi `tools/jpl/l16.py` (không sửa tay). Precise = lõi hiện hành, đã kiểm trùng tệp chuẩn L13 '
          f'(`core/out/baseline-2000-3000/precise.jsonl.gz`) ở {nb} ngày 2000–3000 (năm/tháng/ngày âm, nhuận, Can Chi '
          'năm/tháng/ngày, tiết khí); 1900–1999 lấy từ lõi hiện hành (đã qua L10/L12 với Precise).\n\n')
        w('Trường so: năm/tháng/ngày âm, nhuận, Can Chi năm/tháng/ngày, tên tiết khí đang hiệu lực (tức ngày bắt đầu '
          'tiết khí). Giờ là giờ lịch (§16.1: +07:45:40 trước 1929, +08:00 1929–1967, +07:00 từ 1968). "Cách nửa đêm" '
          '= khoảng cách tới 00:00 gần nhất theo giờ lịch.\n\n')
        w('## Tóm tắt\n\n')
        w(f'- Số ngày khác: **{ndays}**; số đoạn: **{len(segs)}**.\n')
        w(f'- Đoạn "nguyên nhân khác": **{other}**.\n')
        w(f'- Sự kiện hai phương pháp đặt khác ngày lịch: {len(disc)} '
          f'(Sóc {sum(1 for e in disc if e[0][0] == "Sóc")}, tiết khí {sum(1 for e in disc if e[0][0] != "Sóc")}); '
          f'trong đó {len(unused)} sự kiện không làm khác ngày nào (Trung khí/Sóc khác ngày nhưng không đổi tháng nhuận, '
          'tháng 11, hay ngày đầu tháng).\n')
        if missing:
            w(f'- Sự kiện chỉ có ở một phương pháp (ghép theo chỉ số): {len(missing)} — {missing[:10]}\n')
        w(f'- Kết luận L16: **{"ĐẠT" if other == 0 else "KHÔNG ĐẠT"}** (ngưỡng: mọi đoạn do một sự kiện mà hai '
          'phương pháp đặt khác ngày; đoạn "nguyên nhân khác" = KHÔNG ĐẠT).\n\n')
        cents = defaultdict(lambda: [0, 0])
        for s0, s1, *_ in rows:
            c = s0.year // 100 * 100
            cents[c][0] += 1
            cents[c][1] += (s1 - s0).days + 1
        w('| Thế kỷ | Số đoạn | Số ngày |\n|---|---|---|\n')
        for c in sorted(cents):
            w(f'| {c}–{c + 99} | {cents[c][0]} | {cents[c][1]} |\n')
        w('\n## Danh sách đoạn\n\n')
        w('| # | Từ | Đến | Số ngày | Trường khác | Sự kiện gây khác | Precise (giờ lịch) | Jpl (giờ lịch) | Cách nửa đêm Precise / Jpl | Phân loại |\n')
        w('|---|---|---|---|---|---|---|---|---|---|\n')
        for i, (s0, s1, fields, cause, kind) in enumerate(rows, 1):
            if cause:
                ev = '<br>'.join(f'{e[0][0]} {e[0][1]}' for e in cause)
                tp = '<br>'.join(e[1] for e in cause)
                tj = '<br>'.join(e[2] for e in cause)
                md = '<br>'.join(f'{midnight_dist(e[1])} s / {midnight_dist(e[2])} s' for e in cause)
            else:
                ev = tp = tj = md = '—'
            w(f'| {i} | {s0} | {s1} | {(s1 - s0).days + 1} | {", ".join(fields)} | {ev} | {tp} | {tj} | {md} | {kind} |\n')
        w('\n## Sự kiện khác ngày nhưng không làm khác ngày nào\n\n')
        w('| Sự kiện | Precise | Jpl | Cách nửa đêm Precise / Jpl |\n|---|---|---|---|\n')
        for e in unused:
            w(f'| {e[0][0]} {e[0][1]} | {e[1]} | {e[2]} | {midnight_dist(e[1])} s / {midnight_dist(e[2])} s |\n')
    print(f'[l16] ngày khác {ndays}, đoạn {len(segs)}, nguyên nhân khác {other}, sự kiện khác ngày {len(disc)}',
          file=sys.stderr)


if __name__ == '__main__':
    main()
