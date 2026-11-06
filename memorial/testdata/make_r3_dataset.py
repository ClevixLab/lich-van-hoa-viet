#!/usr/bin/env python3
"""Sinh phần nhập của bộ dữ liệu cố định R3/R11/R12 (docs/v0.2/04 R3: 40 hồ sơ/sự kiện).

Ghi JSON lược đồ v1 với trường dẫn xuất để trống; `lvhv-memorial-dump derive` điền chúng:
    python3 memorial/testdata/make_r3_dataset.py > /tmp/in.json
    target/release/lvhv-memorial-dump derive /tmp/in.json > memorial/testdata/r3-dataset.json
"""
import json

TS = "2026-11-06T03:15:00Z"
n_id = 0


def uid(kind):
    global n_id
    n_id += 1
    return f"{kind}0000000-0000-4000-8000-{n_id:012d}"


def dv(cal, y, m, d, leap=False, prec=None):
    if prec is None:
        prec = "day" if y is not None and m is not None else ("month_day" if m is not None else "year")
    return {"calendar": cal, "precision": prec, "year": y, "month": m, "day": d, "leap": leap}


def rule(offs=(0, 3), t="07:00", tt=False, enabled=True):
    return {"id": uid("c"), "enabled": enabled, "offsets_days": list(offs), "time_local": t, "tien_thuong": tt}


def deceased(name, death, birth=None, dx="Ông", gio_het=2, r="default", tray="truyen_thong"):
    return {
        "id": uid("a"), "kind": "deceased", "danh_xung": dx, "ho_ten": name, "ten_hieu": "", "quan_he": "",
        "birth": None if birth is None else {"input": birth, "noi_sinh": "", "lunar_year": None, "lunar_year_approx": False,
                                              "can_chi": None, "lunar": None, "solar": None},
        "death": {"input": death, "lunar": {"year": None, "month": 1, "day": 1, "leap": False}, "solar": None,
                  "lunar_source": "entered", "method": "precise",
                  "tuoi": {"value": None, "label": "huong_tho", "manual": False, "approx": False}},
        "noi_an_tang": "", "photo_id": None,
        "memorial": {"tray_style": tray, "incense": tray != "cong_giao", "bia_tieu_de": "", "loi_tuong_niem": "",
                     "gio_het_nam": gio_het, "huong_duong_duoi": 60},
        "reminders": {"gio": rule() if r == "default" else r, "sinh_nhat": None, "sinh_nhat_am": None},
        "sinh_nhat_am": False, "feb29_rule": "feb28", "ghi_chu": "", "created_at": TS, "updated_at": TS, "rev": 1,
    }


def living(name, birth, dx="", am=False, f29="feb28", r="default", ra="default"):
    return {
        "id": uid("b"), "kind": "living", "danh_xung": dx, "ho_ten": name, "ten_hieu": "", "quan_he": "",
        "birth": {"input": birth, "noi_sinh": "", "lunar_year": None, "lunar_year_approx": False, "can_chi": None,
                  "lunar": None, "solar": None},
        "death": None, "noi_an_tang": "", "photo_id": None, "memorial": None,
        "reminders": {"gio": None, "sinh_nhat": rule() if r == "default" else r,
                      "sinh_nhat_am": (rule((0,)) if ra == "default" else ra) if am else None},
        "sinh_nhat_am": am, "feb29_rule": f29, "ghi_chu": "", "created_at": TS, "updated_at": TS, "rev": 1,
    }


T_TN, T_CUOI, T_DH, T_DUHOC = (f"6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c000{i}" for i in range(1, 5))
T_USER = "e0000000-0000-4000-8000-000000000001"


def event(title, date, recurring, type_id, r="default", rev=1):
    return {"id": uid("d"), "type_id": type_id, "tieu_de": title, "date": date, "recurring": recurring,
            "person_ids": [], "ghi_chu": "", "reminder": rule() if r == "default" else r,
            "created_at": TS, "updated_at": TS, "rev": rev}


persons = [
    # G-A … G-E (02 §2.4)
    deceased("Nguyễn Văn A", dv("solar", 2025, 8, 8), birth=dv("solar", 1940, 5, 1)),
    deceased("Phạm Văn GB", dv("solar", 2025, 9, 21)),
    deceased("Trần Thị B", dv("lunar", None, 3, 10), dx="Cụ", r=rule((0, 3, 10), tt=True), tray="phat_giao"),
    deceased("Lê Văn GD", dv("solar", 2024, 2, 9), birth=dv("lunar", 1950, None, None, prec="year")),
    deceased("Hoàng Thị GE", dv("lunar", 2027, 5, 15), dx="Bà"),
    # Ca biên 02 §9
    deceased("Đỗ Văn Tết", dv("lunar", 2000, 1, 1), r=rule((0, 15, 60), tt=True)),
    deceased("Vũ Thị Chạp Ba Mươi", dv("lunar", 1997, 12, 30), dx="Bà", r=rule((0, 1, 3))),
    deceased("Bùi Văn Chạp", dv("lunar", 2010, 12, 29), r=rule((0, 1), t="23:59", tt=True)),
    deceased("Đặng Thị Nhuận", dv("lunar", 2020, 4, 10, leap=True), dx="Bà", birth=dv("solar", 1980, 7, 7)),
    deceased("Ngô Văn Ba Mươi", dv("lunar", None, 11, 30), dx="Cụ ông", r=rule((5,), t="00:00")),
    deceased("Dương Thị Nhuận Không Năm", dv("lunar", None, 7, 15, leap=True), dx="Cụ bà"),
    deceased("Lý Văn Giỗ Hết Ba", dv("solar", 2024, 6, 15), gio_het=3, birth=dv("solar", 1966, 1, 1, prec="day")),
    deceased("Mai Văn Đầu Năm", dv("solar", 1995, 1, 5), r=rule((15,))),
    deceased("Cao Văn Biên Đầu", dv("solar", 1900, 1, 1), dx="Cụ"),
    deceased("Tạ Văn Biên Cuối", dv("solar", 3000, 12, 31)),
    deceased("Hồ Thị Không Nhắc", dv("solar", 2010, 10, 10), dx="Bà", r=None, tray="cong_giao"),
    deceased("Châu Văn Sáu Mươi", dv("solar", 2019, 12, 25), r=rule((0, 60), t="00:00")),
    deceased("Đinh Thị Ba Mươi Dương", dv("solar", 2025, 2, 27), dx="Bà"),  # 30/1 âm năm 2025
    # Người đang sống
    living("Lê Thị C", dv("solar", 1990, 1, 20), dx="Chị", am=True),
    living("Phan Văn Hai Chín", dv("solar", 2000, 2, 29), dx="Anh"),
    living("Trịnh Thị Mồng Một", dv("solar", 1996, 2, 29), dx="Em", f29="mar1", r=rule((0, 1, 15), t="23:59")),
    living("Kiều Văn Cuối Năm", dv("solar", 1985, 12, 31), am=True, ra=rule((0, 60))),
    living("Lương Thị Âm Ba Mươi", dv("lunar", 1997, 12, 30), am=True, r=rule((3,))),
    living("Quách Văn Âm Nhuận", dv("lunar", 2020, 4, 10, leap=True), dx="Cháu", am=True),
    living("Tôn Thất Chỉ Năm", dv("solar", 1970, None, None, prec="year"), dx="Bác"),
    living("Mạc Thị Mới Sinh", dv("solar", 2026, 1, 15), dx="Bé", am=True),
    living("Âu Văn Ba Mốt", dv("solar", 1950, 3, 1), dx="Ông", r=rule((0, 1, 3, 5, 10, 15))),
]
event_types = [
    {"id": T_TN, "builtin": "tot_nghiep", "ten": "Tốt nghiệp", "default_recurring": False},
    {"id": T_CUOI, "builtin": "ngay_cuoi", "ten": "Ngày cưới", "default_recurring": True},
    {"id": T_DH, "builtin": "dinh_hon", "ten": "Đính hôn", "default_recurring": False},
    {"id": T_DUHOC, "builtin": "du_hoc", "ten": "Du học", "default_recurring": False},
    {"id": T_USER, "builtin": None, "ten": "Lên lão", "default_recurring": True},
]
for t in event_types:
    t.update({"created_at": TS, "updated_at": TS, "rev": 1})
events = [
    event("Ngày cưới anh chị C", dv("solar", 2015, 12, 12), True, T_CUOI, r=rule((0, 1))),
    event("", dv("solar", 2012, 2, 29), True, T_CUOI, rev=3),
    event("Kỷ niệm thành lập họ", dv("solar", 2020, 12, 31), True, T_USER, r=rule((0, 60), tt=False)),
    event("Giỗ tổ dòng họ", dv("lunar", 2010, 8, 15), True, T_USER),
    event("Họp họ đầu xuân", dv("lunar", None, 1, 1), True, T_USER, r=rule((0, 15))),
    event("Tất niên dòng họ", dv("lunar", None, 12, 30), True, T_USER),
    event("Lễ nhuận", dv("lunar", 2023, 2, 10, leap=True), True, T_USER),
    event("Tốt nghiệp đại học", dv("solar", 2026, 3, 15), False, T_TN),
    event("Đính hôn đầu năm", dv("solar", 2026, 1, 1), False, T_DH, r=rule((0,), t="00:00")),
    event("Lễ âm một lần", dv("lunar", 2026, 6, 15), False, T_USER),
    event("Du học Nhật", dv("solar", 2030, 7, 1), False, T_DUHOC, r=rule((0, 3, 60))),
    event("Mốc cuối cửa sổ", dv("solar", 2036, 12, 31), False, T_USER, r=rule((0, 1), t="23:59")),
    event("Sự kiện đã qua", dv("lunar", 2025, 6, 15, leap=True), False, T_USER),
]
doc = {
    "format": "lvhv-backup", "schema_version": 1,
    "app": {"name": "Lịch và Văn hóa Việt", "version": "0.2.0", "platform": "pwa"},
    "exported_at": TS,
    "settings": {"reminders_enabled": True, "default_offsets_days": [0, 3], "default_time_local": "07:00",
                 "upcoming_days": 30, "last_backup_at": None, "backup_nag_days": 30},
    "event_types": event_types, "persons": persons, "events": events, "photos": [],
}
assert len(persons) + len(events) == 40, len(persons) + len(events)
print(json.dumps(doc, ensure_ascii=False, indent=2))
