#!/usr/bin/env python3
"""Sinh bộ dữ liệu bổ sung R13 (docs/v0.2/04 Bổ sung B3 mục 3; 02 Phụ lục A.6): sự kiện gắn với người đã mất
và người đang sống.

Lấy nguyên hồ sơ, loại sự kiện và cài đặt của bộ R3 (`r3-dataset.json`, đã có trường dẫn xuất; sự kiện không
có trường dẫn xuất nên không cần `derive`), gắn `person_ids` cho 13 sự kiện của bộ R3 theo các ca dưới đây
và thêm 4 sự kiện. Mọi `person_ids` trỏ tới hồ sơ có thật và không trùng (01 §3.4, V3).

    python3 memorial/testdata/make_r13_linked.py > memorial/testdata/r13-linked.json

Ca (sau Bổ sung B4, 02 §6.1 và §7.6 — bộ này nay kiểm phần loại trừ):
  - một người đang sống / nhiều người đang sống / không gắn ai          → có lần xuất hiện, bộ lọc dang_song
  - một người đã mất / hai người đã mất / hỗn hợp đã mất + đang sống     → KHÔNG có lần xuất hiện nào
  - sự kiện âm lặp, không rõ năm, gắn người đã mất, cùng ngày với giỗ   → không có (chỉ còn giỗ, ở da_mat)
  - sự kiện dương lặp 29/02 gắn hai người đã mất, quy tắc nhắc tắt      → không có
  - sự kiện một lần đã qua gắn người đã mất                             → không có
  - sự kiện không tiêu đề (hiện tên loại) gắn người đã mất, không có quy tắc nhắc → không có
  (Trước B4: sự kiện mà mọi người đã mất thuộc da_mat; hỗn hợp thuộc dang_song.)
"""
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
with open(os.path.join(HERE, "r3-dataset.json"), encoding="utf-8") as f:
    doc = json.load(f)

pid = {p["ho_ten"]: p["id"] for p in doc["persons"]}
ev = {(e["tieu_de"] or "<cuoi-29-02>"): e for e in doc["events"]}

links = {
    "Ngày cưới anh chị C": ["Lê Thị C"],                                  # sống
    "<cuoi-29-02>": ["Nguyễn Văn A", "Phan Văn Hai Chín"],                  # hỗn hợp
    "Kỷ niệm thành lập họ": [],                                             # không gắn ai
    "Giỗ tổ dòng họ": ["Cao Văn Biên Đầu", "Mai Văn Đầu Năm"],              # hai người đã mất
    "Họp họ đầu xuân": ["Trần Thị B"],                                      # đã mất, không rõ năm mất
    "Tất niên dòng họ": ["Vũ Thị Chạp Ba Mươi"],                            # đã mất
    "Lễ nhuận": ["Quách Văn Âm Nhuận"],                                     # sống
    "Tốt nghiệp đại học": ["Đinh Thị Ba Mươi Dương"],                       # đã mất, một lần
    "Đính hôn đầu năm": ["Kiều Văn Cuối Năm", "Lương Thị Âm Ba Mươi"],      # hai người sống
    "Lễ âm một lần": ["Hồ Thị Không Nhắc"],                                 # đã mất, một lần âm
    "Du học Nhật": ["Mạc Thị Mới Sinh"],                                    # sống
    "Mốc cuối cửa sổ": ["Tạ Văn Biên Cuối", "Âu Văn Ba Mốt"],               # hỗn hợp
    "Sự kiện đã qua": ["Nguyễn Văn A"],                                     # đã mất, đã qua
}
assert set(links) == set(ev), set(ev) ^ set(links)
for k, names in links.items():
    ev[k]["person_ids"] = [pid[n] for n in names]

TS = "2026-11-06T03:15:00Z"
T_TN, T_CUOI = "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0001", "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0002"
T_USER = "e0000000-0000-4000-8000-000000000001"


def dv(cal, y, m, d, leap=False):
    prec = "day" if y is not None else "month_day"
    return {"calendar": cal, "precision": prec, "year": y, "month": m, "day": d, "leap": leap}


def rule(i, enabled=True):
    return {"id": f"f1000000-0000-4000-8000-{i:012d}", "enabled": enabled, "offsets_days": [0, 3],
            "time_local": "07:00", "tien_thuong": False}


def event(i, title, date, recurring, type_id, names, r):
    return {"id": f"f0000000-0000-4000-8000-{i:012d}", "type_id": type_id, "tieu_de": title, "date": date,
            "recurring": recurring, "person_ids": [pid[n] for n in names], "ghi_chu": "", "reminder": r,
            "created_at": TS, "updated_at": TS, "rev": 1}


doc["events"] += [
    # Cùng ngày (âm 10/3) với giỗ Trần Thị B: giỗ đứng trước sự kiện (02 §7.2).
    event(1, "Lễ tạ mộ", dv("lunar", None, 3, 10), True, T_USER, ["Trần Thị B"], rule(1)),
    # Ngày cưới 29/02 của hai người đã mất; quy tắc nhắc tắt — vẫn có trong danh sách.
    event(2, "Ngày cưới ông bà", dv("solar", 1964, 2, 29), True, T_CUOI, ["Nguyễn Văn A", "Trần Thị B"],
          rule(2, enabled=False)),
    # Không tiêu đề → tên loại "Tốt nghiệp"; không có quy tắc nhắc.
    event(3, "", dv("solar", 2027, 6, 7), False, T_TN, ["Đặng Thị Nhuận"], None),
    # Sống + đã mất, dương lặp, đúng ngày đầu năm.
    event(4, "Mừng thọ", dv("solar", 2021, 1, 1), True, T_USER, ["Âu Văn Ba Mốt", "Lê Văn GD"], rule(4)),
]
print(json.dumps(doc, ensure_ascii=False, indent=2))
