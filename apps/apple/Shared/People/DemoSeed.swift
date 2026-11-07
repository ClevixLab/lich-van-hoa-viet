// CHỈ bản Debug: dữ liệu mẫu và lối vào thẳng từng màn hình cho kiểm thử giao diện / ảnh chụp trên CI.
// Tham số khởi chạy: -lvhvReset 1 (xóa kho trước khi mở), -lvhvSeed demo, -lvhvToday 2026-11-07,
// -lvhvScreen <tên màn hình>. Không có trong bản Release.
#if DEBUG
import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

@MainActor
enum DemoSeed {
    static var ids: [String: String] = [:]

    static func resetIfRequested() {
        let d = UserDefaults.standard
        guard d.bool(forKey: "lvhvReset") else { return }
        try? FileManager.default.removeItem(at: PeoplePaths.storeDir())
    }

    /// Ảnh chân dung giả (không phải ảnh người thật) để thấy khung ảnh.
    static func portrait(_ hue: CGFloat) -> ProcessedPhoto? {
        let w = 300, h = 380
        guard let ctx = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: 0,
                                  space: CGColorSpace(name: CGColorSpace.sRGB)!,
                                  bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue) else { return nil }
        ctx.setFillColor(CGColor(red: 0.55 + hue * 0.2, green: 0.5, blue: 0.45, alpha: 1))
        ctx.fill(CGRect(x: 0, y: 0, width: w, height: h))
        ctx.setFillColor(CGColor(red: 0.25, green: 0.2, blue: 0.18, alpha: 1))
        ctx.fillEllipse(in: CGRect(x: 95, y: 170, width: 110, height: 130))
        ctx.fillEllipse(in: CGRect(x: 40, y: -60, width: 220, height: 220))
        guard let img = ctx.makeImage(), let jpg = PhotoProcessor.encode(img, quality: 0.8) else { return nil }
        return try? PhotoProcessor.process(jpg)
    }

    static func seedIfNeeded(_ m: PeopleModel) {
        let kind = UserDefaults.standard.string(forKey: "lvhvSeed") ?? ""
        guard kind == "demo", m.persons.isEmpty else { return }
        let today = m.today
        // A — ông nội, ngày mất dương, có ảnh, có thông tin báo tử.
        var a = PersonDraft(kind: "deceased", settings: m.settings)
        a.danhXung = "Ông"; a.hoTen = "Nguyễn Văn A"; a.quanHe = "ông nội (bên nội)"; a.tenHieu = "Phúc Thiện"
        a.death.day = "8"; a.death.month = "8"; a.death.year = "2025"
        a.birth.day = "1"; a.birth.month = "5"; a.birth.year = "1940"; a.noiSinh = "Hà Nội"
        a.noiAnTang = "Nghĩa trang quê nhà"
        a.loiTuongNiem = "Con cháu luôn nhớ ông."
        a.notice.reportedBy = "Chú Ba"; a.notice.place = "Bệnh viện tỉnh"; a.notice.reason = "Tuổi cao, bệnh nặng"
        a.notice.date.day = "8"; a.notice.date.month = "8"; a.notice.date.year = "2025"
        a.notice.note = "Gia đình đã báo cho họ hàng."
        let pA = portrait(0).flatMap { try? m.addPhoto($0) }
        let cert = portrait(0.8).flatMap { try? m.addPhoto($0) }
        ids["A"] = (try? m.savePerson(a.build(photoId: pA, noticePhotoIds: cert.map { [$0] } ?? [], today: today)))?["id"].str

        // B — bà ngoại, giỗ đúng hôm nay (ngày âm của hôm nay, năm 2019).
        var b = PersonDraft(kind: "deceased", settings: m.settings)
        b.danhXung = "Bà"; b.hoTen = "Trần Thị B"; b.quanHe = "Bà ngoại"
        if let l = Lich.lunar(today) {
            b.death.cal = .lunar; b.death.day = "\(l.d)"; b.death.month = "\(l.m)"; b.death.year = "2019"
        }
        b.birth.yearOnly = true; b.birth.year = "1935"
        ids["B"] = (try? m.savePerson(b.build(photoId: nil, noticePhotoIds: [], today: today)))?["id"].str

        // E — cụ ông, không rõ năm mất.
        var e = PersonDraft(kind: "deceased", settings: m.settings)
        e.danhXung = "Cụ"; e.hoTen = "Đỗ Văn E"; e.quanHe = "Cụ ông"
        e.death.cal = .lunar; e.death.unknownYear = true; e.death.day = "10"; e.death.month = "3"
        ids["E"] = (try? m.savePerson(e.build(photoId: nil, noticePhotoIds: [], today: today)))?["id"].str

        // C — con trai, có sinh nhật âm và sự kiện tốt nghiệp.
        var c = PersonDraft(kind: "living", settings: m.settings)
        c.hoTen = "Lê Văn C"; c.quanHe = "Con trai"
        c.birth.day = "20"; c.birth.month = "1"; c.birth.year = "1990"
        c.sinhNhatAmOn = true
        let cId = (try? m.savePerson(c.build(photoId: nil, noticePhotoIds: [], today: today)))?["id"].str
        ids["C"] = cId
        if let cId {
            var ev = EventDraft(personId: cId, types: m.eventTypes, settings: m.settings)
            ev.typeId = "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0001"
            ev.date.day = "15"; ev.date.month = "6"; ev.date.year = "2027"
            try? m.saveEvent(ev.build(typeId: ev.typeId, personDeceased: false))
        }

        // D — chị, sinh nhật trong 30 ngày tới.
        var d = PersonDraft(kind: "living", settings: m.settings)
        let bd = today.adding(5)
        d.danhXung = "Chị"; d.hoTen = "Phạm Thị D"; d.quanHe = "Chị"
        d.birth.day = "\(bd.d)"; d.birth.month = "\(bd.m)"; d.birth.year = "1985"
        ids["D"] = (try? m.savePerson(d.build(photoId: nil, noticePhotoIds: [], today: today)))?["id"].str
        m.toast = nil
    }

    /// Tìm id theo tên khi kho đã có sẵn dữ liệu mẫu từ lần chạy trước.
    private static func id(_ key: String, _ m: PeopleModel) -> String {
        if let v = ids[key] { return v }
        let names = ["A": "Nguyễn Văn A", "B": "Trần Thị B", "C": "Lê Văn C", "D": "Phạm Thị D", "E": "Đỗ Văn E"]
        return m.persons.first { $0["ho_ten"].str == names[key] }?["id"].str ?? ""
    }

    static func deepLink(_ m: PeopleModel) -> (PSection, [PRoute])? {
        seedIfNeeded(m)
        guard let s = UserDefaults.standard.string(forKey: "lvhvScreen") else { return nil }
        switch s {
        case "upcoming": return (.upcoming, [])
        case "period": return (.period, [])
        case "profiles": return (.profiles, [])
        case "backup": return (.backup, [])
        case "person-deceased": return (.profiles, [.person(id("A", m))])
        case "person-living": return (.profiles, [.person(id("C", m))])
        case "form-deceased": return (.profiles, [.add("deceased")])
        case "form-living": return (.profiles, [.add("living")])
        case "form-edit": return (.profiles, [.person(id("A", m)), .edit(id("A", m))])
        case "event": return (.profiles, [.person(id("C", m)), .event(id("C", m), nil)])
        case "event-deceased": return (.profiles, [.person(id("A", m)), .event(id("A", m), nil)])
        case "mark": return (.profiles, [.person(id("D", m)), .markDeceased(id("D", m))])
        case "memorial": return (.profiles, [.person(id("B", m)), .memorial(id("B", m))])
        case "memorial-a": return (.profiles, [.person(id("A", m)), .memorial(id("A", m))])
        case "settings": return (.upcoming, [.settings])
        default: return (.upcoming, [])
        }
    }
}
#endif
