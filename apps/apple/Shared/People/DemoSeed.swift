// CHỈ bản Debug: dữ liệu mẫu và lối vào thẳng từng màn hình cho kiểm thử giao diện / ảnh chụp trên CI.
// Tham số khởi chạy: -lvhvReset 1 (xóa kho trước khi mở), -lvhvSeed demo, -lvhvToday 2026-11-07,
// -lvhvScreen <tên màn hình>. Không có trong bản Release.
#if DEBUG
import CoreGraphics
import Foundation
import ImageIO
import SwiftUI
import UniformTypeIdentifiers
import UserNotifications

@MainActor
enum DemoSeed {
    static var ids: [String: String] = [:]
    /// Màn hình kiểm tra sau nâng cấp (R14, chỉ bản Debug).
    static var isDiag: Bool { UserDefaults.standard.string(forKey: "lvhvScreen") == "diag" }

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

    /// "store-error": tạo kho rồi ghi `user_version` = 99 vào phần đầu tệp SQLite (byte 60–63) để lõi báo
    /// "phiên bản mới hơn" thật khi mở.
    static func prepareStoreErrorIfRequested(_ dir: URL) {
        guard UserDefaults.standard.string(forKey: "lvhvScreen") == "store-error" else { return }
        do { _ = try LvhvStore.open(dir: dir.path) } catch { return }
        let db = dir.appendingPathComponent("lvhv.sqlite")
        guard let h = try? FileHandle(forUpdating: db) else { return }
        try? h.seek(toOffset: 60)
        h.write(Data([0, 0, 0, 99]))
        try? h.close()
    }

    /// Điền sẵn biểu mẫu để chụp ô ngày đang nhập, kết quả đổi lịch và lỗi.
    static func prefill(_ d: inout PersonDraft, _ kind: String) {
        switch kind {
        case "am-nhuan":
            d.danhXung = "Bà"; d.hoTen = "Trần Thị B"; d.quanHe = "Bà nội"
            d.death.cal = .lunar; d.death.day = "15"; d.death.month = "6"; d.death.year = "2025"; d.death.leap = true
        case "duong":
            d.danhXung = "Ông"; d.hoTen = "Nguyễn Văn A"; d.quanHe = "Ông nội"
            d.death.day = "8"; d.death.month = "8"; d.death.year = "2025"
            d.birth.day = "1"; d.birth.month = "5"; d.birth.year = "1940"
        case "loi":
            d.death.day = "30"; d.death.month = "2"; d.death.year = "2026"
            d.birth.day = "1"; d.birth.month = "5"; d.birth.year = "1940"
            d.gio.hour = "25"
        case "living-am":
            d.hoTen = "Lê Văn C"; d.quanHe = "Con trai"
            d.birth.day = "20"; d.birth.month = "1"; d.birth.year = "1990"
            d.sinhNhatAmOn = true; d.amSame = false
        default: break
        }
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
        a.gioiTinh = "nam"
        // Tệp đính kèm mẫu: một ảnh "giấy chứng tử" (giả, vẽ trong mã) và một PDF nhỏ hợp lệ.
        if let img = portrait(0.8) {
            a.notice.attachments.append(.make(NewAttachment(data: img.jpeg, mime: "image/jpeg", fileName: nil), kind: .giayChungTu))
        }
        let pdf = Data("%PDF-1.4\n1 0 obj << /Type /Catalog >> endobj\ntrailer << /Root 1 0 R >>\n%%EOF\n".utf8)
        a.notice.attachments.append(.make(NewAttachment(data: pdf, mime: "application/pdf", fileName: "giay-bao-tu.pdf"),
                                          kind: .tinBaoTu))
        _ = try? PeopleModel.uploadAttachments(&a.notice)
        ids["A"] = (try? m.savePerson(a.build(photoId: pA, today: today)))?["id"].str

        // B — bà ngoại, giỗ đúng hôm nay (ngày âm của hôm nay, năm 2019).
        var b = PersonDraft(kind: "deceased", settings: m.settings)
        b.danhXung = "Bà"; b.hoTen = "Trần Thị B"; b.quanHe = "Bà ngoại"; b.gioiTinh = "nu"
        if let l = Lich.lunar(today) {
            b.death.cal = .lunar; b.death.day = "\(l.d)"; b.death.month = "\(l.m)"; b.death.year = "2019"
        }
        b.birth.yearOnly = true; b.birth.year = "1935"
        ids["B"] = (try? m.savePerson(b.build(photoId: nil, today: today)))?["id"].str

        // E — cụ ông, không rõ năm mất.
        var e = PersonDraft(kind: "deceased", settings: m.settings)
        e.danhXung = "Cụ"; e.hoTen = "Đỗ Văn E"; e.quanHe = "Cụ ông"
        e.death.cal = .lunar; e.death.unknownYear = true; e.death.day = "10"; e.death.month = "3"
        ids["E"] = (try? m.savePerson(e.build(photoId: nil, today: today)))?["id"].str

        // C — con trai, có sinh nhật âm và sự kiện tốt nghiệp.
        var c = PersonDraft(kind: "living", settings: m.settings)
        c.hoTen = "Lê Văn C"; c.quanHe = "Con trai"
        c.birth.day = "20"; c.birth.month = "1"; c.birth.year = "1990"
        c.sinhNhatAmOn = true
        let cId = (try? m.savePerson(c.build(photoId: nil, today: today)))?["id"].str
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
        ids["D"] = (try? m.savePerson(d.build(photoId: nil, today: today)))?["id"].str
        m.toast = nil
    }

    /// Gắn ảnh mâm lễ thử cho một người (chỉ khi chưa có).
    static func seedTray(_ m: PeopleModel, _ pid: String) {
        guard let p = m.person(pid), p["memorial"]["tray_photo_id"].str == nil,
              let raw = exifJPEG(), let t = try? PhotoProcessor.process(raw, maxSide: TrayPhoto.maxSide, maxBytes: TrayPhoto.maxBytes),
              let photo = try? m.addPhoto(t) else { return }
        // Cùng đường lưu với biểu mẫu: bản nháp → JSON → save_person_json.
        var d = PersonDraft(person: p, settings: m.settings)
        d.trayPhotoId = photo
        do { _ = try m.savePerson(d.build(photoId: p["photo_id"].str, today: m.today)) }
        catch { print("seedTray: \(error)"); m.discardPhoto(photo) }
        m.toast = nil
    }

    /// Ảnh mâm thử 2400 × 1800 có EXIF GPS và tên máy (để thấy ảnh lưu đã bỏ siêu dữ liệu và đã thu nhỏ ≤ 1600 px).
    static func exifJPEG() -> Data? {
        let w = 2400, h = 1800
        guard let ctx = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: 0,
                                  space: CGColorSpace(name: CGColorSpace.sRGB)!,
                                  bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue) else { return nil }
        ctx.setFillColor(CGColor(red: 0.42, green: 0.27, blue: 0.17, alpha: 1))
        ctx.fill(CGRect(x: 0, y: 0, width: w, height: h))
        ctx.setFillColor(CGColor(red: 0.8, green: 0.64, blue: 0.32, alpha: 1))
        ctx.fillEllipse(in: CGRect(x: 300, y: 300, width: 1800, height: 900))
        let fruits: [(CGFloat, CGFloat, CGFloat, CGFloat)] = [(0.95, 0.6, 0.15, 700), (0.45, 0.65, 0.25, 1000),
                                                              (0.85, 0.25, 0.2, 1300), (0.98, 0.82, 0.3, 1600)]
        for (r, g, b, x) in fruits {
            ctx.setFillColor(CGColor(red: r, green: g, blue: b, alpha: 1))
            ctx.fillEllipse(in: CGRect(x: x - 150, y: 700, width: 300, height: 300))
        }
        guard let img = ctx.makeImage() else { return nil }
        let out = NSMutableData()
        guard let dst = CGImageDestinationCreateWithData(out, UTType.jpeg.identifier as CFString, 1, nil) else { return nil }
        let props: [CFString: Any] = [
            kCGImagePropertyGPSDictionary: [kCGImagePropertyGPSLatitude: 21.03, kCGImagePropertyGPSLatitudeRef: "N",
                                            kCGImagePropertyGPSLongitude: 105.85, kCGImagePropertyGPSLongitudeRef: "E"],
            kCGImagePropertyTIFFDictionary: [kCGImagePropertyTIFFModel: "MayChupThu"],
        ]
        CGImageDestinationAddImage(dst, img, props as CFDictionary)
        guard CGImageDestinationFinalize(dst) else { return nil }
        return out as Data
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
        case "upcoming", "upcoming-denied", "backup-nag", "store-error": return (.upcoming, [])
        case "restore", "restore-replace": return (.backup, [])
        case "form-am-nhuan", "form-duong", "form-loi": return (.profiles, [.add("deceased")])
        case "form-living-am": return (.profiles, [.add("living")])
        case "period": return (.period, [])
        case "profiles": return (.profiles, [])
        case "backup": return (.backup, [])
        case "person-deceased": return (.profiles, [.person(id("A", m))])
        case "person-notice": return (.profiles, [.person(id("A", m))])
        case "person-living": return (.profiles, [.person(id("C", m))])
        case "form-deceased": return (.profiles, [.add("deceased")])
        case "form-living": return (.profiles, [.add("living")])
        case "form-edit": return (.profiles, [.person(id("A", m)), .edit(id("A", m))])
        case "event": return (.profiles, [.person(id("C", m)), .event(id("C", m), nil)])
        case "event-deceased": return (.profiles, [.person(id("A", m)), .event(id("A", m), nil)])
        case "mark": return (.profiles, [.person(id("D", m)), .markDeceased(id("D", m))])
        case "memorial": return (.profiles, [.person(id("B", m)), .memorial(id("B", m))])
        case "memorial-a": return (.profiles, [.person(id("A", m)), .memorial(id("A", m))])
        case "memorial-cg":
            // Công giáo (08 §16.1): đổi `tray_style` của E rồi mở màn hình tưởng niệm.
            let e = id("E", m)
            if let p = m.person(e), p["memorial"]["tray_style"].str != "cong_giao" {
                var j = PersonDraft(person: p, settings: m.settings).build(photoId: p["photo_id"].str, today: m.today)
                var mem = j["memorial"]
                mem["tray_style"] = "cong_giao"
                mem["incense"] = false
                j["memorial"] = mem
                do { _ = try m.savePerson(j) } catch { print("memorial-cg: \(error)") }
                m.toast = nil
            }
            return (.profiles, [.person(e), .memorial(e)])
        case "memorial-tray":
            // 08 §16.4 ca 2: ảnh mâm lễ có EXIF (GPS, tên máy) đi qua đúng đường xử lý của biểu mẫu rồi gắn cho A.
            seedTray(m, id("A", m))
            return (.profiles, [.person(id("A", m)), .memorial(id("A", m))])
        case "form-tray":
            seedTray(m, id("A", m))
            return (.profiles, [.person(id("A", m)), .editTray(id("A", m))])
        case "form-tray-empty": return (.profiles, [.person(id("B", m)), .editTray(id("B", m))])
        case "settings": return (.upcoming, [.settings])
        default: return (.upcoming, [])
        }
    }
}

/// R14 — kiểm tra dữ liệu sau khi cài đè bản mới lên bản cũ (chỉ bản Debug, dùng trong CI `apple` job `upgrade`).
/// So kho hiện tại với bản chép kho cũ (`LichVanHoaViet-before`, CI chép trước khi cài đè), đếm hồ sơ/sự kiện/ảnh,
/// đọc lại từng ảnh (kiểm sha256 ở lõi), đếm thông báo đang chờ sau khi lập lịch lại.
struct UpgradeDiagView: View {
    @ObservedObject private var m = PeopleModel.shared
    @State private var report = "đang kiểm…"

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            PHeading("Kiểm tra sau nâng cấp (R14)")
            Text(report).font(.system(.body, design: .monospaced)).foregroundStyle(C.ink)
                .accessibilityIdentifier("diag.report")
            Button("Cho phép thông báo và lập lịch lại") {
                Task {
                    await m.scheduler.requestPermission(store: m.store)
                    report = await compute()
                }
            }
            .buttonStyle(SecondaryButtonStyle())
            .accessibilityIdentifier("diag.allow")
            Button("Kiểm lại") { Task { report = await compute() } }
                .buttonStyle(SecondaryButtonStyle())
                .accessibilityIdentifier("diag.refresh")
        }
        .task { report = await compute() }
    }

    private func normalized(_ ds: String) -> JV? {
        guard var j = JV.parse(ds) else { return nil }
        j = j.removing("exported_at").removing("app")
        return j
    }

    func compute() async -> String {
        guard let st = m.store else { return "migration=fail error=\(m.openError ?? "?")" }
        var lines: [String] = ["migration=ok"]
        let ds = (try? st.datasetJson()) ?? ""
        let cur = normalized(ds)
        lines.append("persons=\(m.persons.count)")
        lines.append("events=\(m.events.count)")
        let ms = st.migrationStatus()
        lines[0] = "migration=\(ms.state) from=\(ms.fromVersion) to=\(ms.toVersion) backup=\(ms.backupPath == nil ? 0 : 1)"
        let photoIds = m.persons.compactMap { $0["photo_id"].str }
        let readable = photoIds.filter { (try? st.photoBytes(id: $0)) != nil }.count
        lines.append("photos=\(photoIds.count) photos_ok=\(readable)")
        let attIds = m.persons.flatMap { NoticeDraft($0["death"]["notice"]).attachments.compactMap(\.fileId) }
        let attOk = attIds.filter { (try? st.attachmentBytes(id: $0)) != nil }.count
        lines.append("attachments=\(attIds.count) attachments_ok=\(attOk)")
        lines.append("legacy=\(m.persons.filter { $0["legacy"].isObject }.count)")
        lines.append("notices=\(m.persons.filter { $0["death"]["notice"].isObject }.count)")
        let beforeDir = PeoplePaths.storeDir().deletingLastPathComponent().appendingPathComponent("LichVanHoaViet-before")
        if FileManager.default.fileExists(atPath: beforeDir.path),
           let old = try? LvhvStore.open(dir: beforeDir.path), let ods = try? old.datasetJson() {
            lines.append("same_as_before=\(normalized(ods) == cur ? 1 : 0)")
        } else {
            lines.append("same_as_before=na")
        }
        let s = await UNUserNotificationCenter.current().notificationSettings()
        lines.append("auth=\(s.authorizationStatus.rawValue)")
        let pending = await UNUserNotificationCenter.current().pendingNotificationRequests()
        lines.append("pending=\(pending.filter { $0.identifier.hasPrefix("lvhv:") }.count)")
        let ui = UserDefaults.standard
        lines.append("theme=\(ui.integer(forKey: "theme")) size=\(ui.integer(forKey: "size"))")
        return lines.joined(separator: "\n")
    }
}
#endif
