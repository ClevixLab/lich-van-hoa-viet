// Kiểm thử lớp nối Swift ↔ lõi của mô-đun "Người thân": JSON, bản nháp biểu mẫu, lập lịch thông báo (bản giả của
// UNUserNotificationCenter), ảnh S9, S5, US-M-07 / R-PRIV-2. Mọi tính toán vẫn do lõi làm.
import CoreGraphics
import ImageIO
import UniformTypeIdentifiers
import XCTest
@testable import LichViet

final class FakeCenter: NotificationCenterLike {
    var auth: NotifAuth = .authorized
    var grant = true
    var requests = 0
    var store: [String: PendingNotif] = [:]
    var adds: [PlannedNotif] = []

    func authorization() async -> NotifAuth { auth }
    func requestAuthorization() async -> Bool { requests += 1; auth = grant ? .authorized : .denied; return grant }
    func pending() async -> [PendingNotif] { Array(store.values) }
    func remove(ids: [String]) { for i in ids { store.removeValue(forKey: i) } }
    func add(_ p: PlannedNotif) async throws {
        adds.append(p)
        store[p.id] = PendingNotif(id: p.id, atUnix: p.at_unix_s, title: p.title, body: p.body)
    }
}

final class PeopleGlueTests: XCTestCase {
    let today = CivilDate(y: 2026, m: 11, d: 7)
    let vn = DeviceZone(offsetMin: 420, transitions: [])
    /// 2026-11-07 06:00 giờ Việt Nam.
    var now: Date { Date(timeIntervalSince1970: 1_794_006_000) }

    private func tempStore() throws -> (LvhvStore, URL) {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("lvhv-test-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return (try LvhvStore.open(dir: dir.path), dir)
    }

    private func deceasedDraft(_ name: String, settings: JV) -> PersonDraft {
        var a = PersonDraft(kind: "deceased", settings: settings)
        a.danhXung = "Ông"; a.hoTen = name; a.quanHe = "ông nội"
        a.death.day = "8"; a.death.month = "8"; a.death.year = "2025"
        a.birth.day = "1"; a.birth.month = "5"; a.birth.year = "1940"
        return a
    }

    // MARK: JSON, định dạng

    func testJVRoundTripKeepsUnknownFieldsAndIntegers() {
        let j = JV.parse(#"{"a":1,"b":[true,null,"x"],"c":{"bia_tieu_de":"Hiển khảo"},"d":2.5}"#)!
        XCTAssertEqual(j["a"].int, 1)
        XCTAssertEqual(j["c"]["bia_tieu_de"].str, "Hiển khảo")
        XCTAssertEqual(JV.parse(j.text), j)
        XCTAssertTrue(j.text.contains("\"a\":1"))
    }

    func testPeriodTitles() {
        XCTAssertEqual(PFmt.periodTitle(kind: "tuan", from: CivilDate(y: 2026, m: 11, d: 2), to: CivilDate(y: 2026, m: 11, d: 8)),
                       "Tuần 02/11 – 08/11/2026")
        XCTAssertEqual(PFmt.periodTitle(kind: "tuan", from: CivilDate(y: 2026, m: 12, d: 28), to: CivilDate(y: 2027, m: 1, d: 3)),
                       "Tuần 28/12/2026 – 03/01/2027")
        XCTAssertEqual(PFmt.periodTitle(kind: "quy", from: CivilDate(y: 2026, m: 10, d: 1), to: CivilDate(y: 2026, m: 12, d: 31)),
                       "Quý IV/2026 · 01/10 – 31/12")
        XCTAssertEqual(PFmt.steleRelation("ông nội (bên nội)"), "Ông nội")
        XCTAssertEqual(PFmt.distance(-2), "Đã qua 2 ngày")
    }

    func testWeekStartsMondayViaCore() throws {
        let r = try periodBounds(kind: "tuan", anchor: today.sd)
        XCTAssertEqual(CivilDate(r.from), CivilDate(y: 2026, m: 11, d: 2))
        XCTAssertEqual(CivilDate(r.to), CivilDate(y: 2026, m: 11, d: 8))
    }

    // MARK: Biểu mẫu (08 §3.4)

    func testDateMessagesComeFromCore() {
        var d = DateDraft()
        XCTAssertEqual(DateCheck.death(d, today: today).error, "Hãy nhập ngày mất")
        d.day = "30"; d.month = "2"
        XCTAssertEqual(DateCheck.death(d, today: today).error, "Hãy nhập đủ ngày, tháng, năm")
        d.year = "2026"
        XCTAssertEqual(DateCheck.death(d, today: today).error, "Tháng 2 năm 2026 chỉ có 28 ngày")
        d.day = "1"; d.month = "12"
        XCTAssertEqual(DateCheck.death(d, today: today).error, "Ngày mất không được sau hôm nay")
        var l = DateDraft()
        l.cal = .lunar; l.day = "15"; l.month = "5"; l.year = "2025"; l.leap = true
        XCTAssertEqual(DateCheck.death(l, today: today).error, "Năm 2025 không có tháng 5 nhuận")
        XCTAssertEqual(DateCheck.leapMonth(ofLunarYear: 2025), 6)
        XCTAssertNil(DateCheck.leapMonth(ofLunarYear: 2026))
    }

    func testResultLines() {
        var d = DateDraft()
        d.day = "8"; d.month = "8"; d.year = "2025"
        XCTAssertEqual(DateCheck.resultLine(DateCheck.death(d, today: today), purpose: .death, today: today),
                       "Âm lịch: 15/6 nhuận năm Ất Tỵ (2025)")
        var l = DateDraft()
        l.cal = .lunar; l.day = "15"; l.month = "6"; l.year = "2025"; l.leap = true
        XCTAssertEqual(DateCheck.resultLine(DateCheck.death(l, today: today), purpose: .death, today: today),
                       "Dương lịch: Thứ Sáu, 08/08/2025")
        var b = DateDraft()
        b.day = "20"; b.month = "1"; b.year = "1990"
        let line = DateCheck.resultLine(DateCheck.birth(b, today: today), purpose: .birth, living: true, today: today) ?? ""
        XCTAssertTrue(line.contains("Kỷ Tỵ"), line)
        XCTAssertTrue(line.contains("36 tuổi"), line)
    }

    func testFormValidation() {
        var p = PersonDraft(kind: "deceased", settings: .null)
        var e = p.validate(today: today)
        XCTAssertEqual(e["ho_ten"], "Họ tên không được để trống")
        XCTAssertEqual(e["death"], "Hãy nhập ngày mất")
        p.hoTen = "  Nguyễn Văn A "
        p.death.day = "8"; p.death.month = "8"; p.death.year = "2025"
        p.gio.hour = "25"
        e = p.validate(today: today)
        XCTAssertNil(e["ho_ten"])
        XCTAssertEqual(e["gio.time"], "Giờ nhắc phải từ 0 đến 23 giờ, phút từ 0 đến 59")
        p.gio.hour = "18"; p.gio.minute = "5"
        XCTAssertTrue(p.validate(today: today).isEmpty)
        let j = p.build(photoId: nil, noticePhotoIds: [], today: today)
        XCTAssertEqual(j["ho_ten"].str, "Nguyễn Văn A")
        XCTAssertEqual(j["reminders"]["gio"]["time_local"].str, "18:05")
        XCTAssertNil(j["death"].object?["notice"], "không ghi khóa notice khi không có thông tin báo tử")
    }

    func testLunarBirthdaySameAsSolarCopiesSettingsWithOwnId() {
        var c = PersonDraft(kind: "living", settings: .null)
        c.hoTen = "Lê Văn C"; c.birth.day = "20"; c.birth.month = "1"; c.birth.year = "1990"
        c.sinhNhatAmOn = true
        c.sinhNhat.offsets = [0, 1]; c.sinhNhat.hour = "08"
        let j = c.build(photoId: nil, noticePhotoIds: [], today: today)
        XCTAssertEqual(j["reminders"]["sinh_nhat_am"]["offsets_days"], [0, 1])
        XCTAssertEqual(j["reminders"]["sinh_nhat_am"]["time_local"].str, "08:00")
        XCTAssertNotEqual(j["reminders"]["sinh_nhat_am"]["id"], j["reminders"]["sinh_nhat"]["id"])
        XCTAssertEqual(j["sinh_nhat_am"].bool, true)
    }

    // MARK: Kho, US-M-07, R-PRIV-2, S5

    func testSaveDeceasedAndMarkDeceasedKeepsIdAndHidesNotice() throws {
        let (st, dir) = try tempStore()
        let settings = JV.parse(try st.settingsJson())!
        let saved = JV.parse(try st.savePersonJson(json: deceasedDraft("Nguyễn Văn A", settings: settings)
            .build(photoId: nil, noticePhotoIds: [], today: today).text))!
        XCTAssertEqual(saved["death"]["lunar"]["leap"].bool, true)
        XCTAssertEqual(saved["death"]["lunar_source"].str, "converted")

        var c = PersonDraft(kind: "living", settings: settings)
        c.hoTen = "Lê Văn C"; c.birth.day = "20"; c.birth.month = "1"; c.birth.year = "1990"
        let living = JV.parse(try st.savePersonJson(json: c.build(photoId: nil, noticePhotoIds: [], today: today).text))!
        let id = living["id"].string
        var n = NoticeDraft()
        n.reportedBy = "Chú Ba"; n.reason = "SENTINEL-LY-DO-XYZ"
        var death = DateDraft(); death.day = "1"; death.month = "11"; death.year = "2026"
        let dv = DateCheck.death(death, today: today).value
        let marked = JV.parse(try st.markDeceasedJson(personId: id, deathDateValueJson: dv.text,
                                                      noticeJson: n.json(photoIds: [], today: today)?.text, today: today.sd))!
        XCTAssertEqual(marked["id"].str, id)
        XCTAssertEqual(marked["kind"].str, "deceased")
        XCTAssertEqual(marked["rev"].int, 2)
        XCTAssertEqual(marked["birth"]["input"]["year"].int, 1990)
        XCTAssertEqual(marked["death"]["notice"]["reason"].str, "SENTINEL-LY-DO-XYZ")

        let ics = try icsExport(datasetJson: try st.datasetJson(), today: today.sd, nowUnixS: Int64(now.timeIntervalSince1970), onlyIds: nil)
        XCTAssertFalse(ics.content.contains("SENTINEL"))
        XCTAssertFalse(ics.fileName.contains("Lê"))
        let up = try st.upcomingJson(today: today.sd, days: 400)
        XCTAssertFalse(up.contains("SENTINEL"))
        let items = Core.decode([OccItem].self, up) ?? []
        XCTAssertFalse(items.contains { $0.entity_id == id && $0.class.hasPrefix("sinh_nhat") }, "B4: không còn sinh nhật")
        XCTAssertTrue(items.contains { $0.entity_id == id && $0.class == "gio" })
        let rem = try remindersJson(datasetJson: try st.datasetJson(), nowUnixS: Int64(now.timeIntervalSince1970), zone: vn,
                                    horizonDays: 400, showNames: true)
        XCTAssertFalse(rem.contains("SENTINEL"))

        let marks = PeoplePaths.excludeFromBackup(dir)
        XCTAssertTrue(marks.contains(dir))
        for u in [dir, dir.appendingPathComponent("lvhv.sqlite")] {
            XCTAssertEqual(try u.resourceValues(forKeys: [.isExcludedFromBackupKey]).isExcludedFromBackup, true, u.path)
        }
    }

    // MARK: Lập lịch thông báo (02 §7.4) với bản giả

    @MainActor
    func testRescheduleHidesNamesByDefaultAndIsIdempotent() async throws {
        let (st, _) = try tempStore()
        let settings = JV.parse(try st.settingsJson())!
        _ = try st.savePersonJson(json: deceasedDraft("Nguyễn Văn A", settings: settings)
            .build(photoId: nil, noticePhotoIds: [], today: today).text)
        let fake = FakeCenter()
        let s = ReminderScheduler(center: fake)
        let o1 = await s.reschedule(store: st, reason: .save, now: now, zone: vn)
        XCTAssertGreaterThan(o1.added, 0)
        XCTAssertTrue(fake.store.keys.allSatisfy { $0.hasPrefix("lvhv:") })
        XCTAssertTrue(fake.store.values.allSatisfy { $0.title == "Sắp đến ngày giỗ" && $0.body.isEmpty }, "O-9: không có tên")
        XCTAssertNotNil(ReminderScheduler.components(fake.adds[0].fire_local)?.hour)

        let o2 = await s.reschedule(store: st, reason: .appOpen, now: now, zone: vn)
        XCTAssertEqual(o2.added, 0)
        XCTAssertEqual(o2.replaced, 0)
        XCTAssertEqual(o2.kept, fake.store.count)

        try st.setLockScreenNames(show: true)
        let o3 = await s.reschedule(store: st, reason: .settings, now: now, zone: vn)
        XCTAssertGreaterThan(o3.replaced, 0)
        XCTAssertTrue(fake.store.values.allSatisfy { $0.title.contains("Nguyễn Văn A") })
    }

    @MainActor
    func testRescheduleCapsAt64WithSentinel() async throws {
        let (st, _) = try tempStore()
        let settings = JV.parse(try st.settingsJson())!
        for i in 0..<70 {
            var a = deceasedDraft("Người \(i)", settings: settings)
            a.death.day = "\(i % 28 + 1)"; a.death.month = "\(i % 12 + 1)"; a.death.year = "2020"
            _ = try st.savePersonJson(json: a.build(photoId: nil, noticePhotoIds: [], today: today).text)
        }
        let fake = FakeCenter()
        let s = ReminderScheduler(center: fake)
        await s.reschedule(store: st, reason: .save, now: now, zone: vn)
        XCTAssertLessThanOrEqual(fake.store.count, 64)
        XCTAssertNotNil(fake.store["lvhv:sentinel"])
        XCTAssertEqual(fake.store["lvhv:sentinel"]?.title, sentinelInfo().text)
    }

    @MainActor
    func testPermissionAskedOnlyAfterSave() async throws {
        let (st, _) = try tempStore()
        let settings = JV.parse(try st.settingsJson())!
        _ = try st.savePersonJson(json: deceasedDraft("A", settings: settings).build(photoId: nil, noticePhotoIds: [], today: today).text)
        let fake = FakeCenter()
        fake.auth = .notDetermined
        let s = ReminderScheduler(center: fake)
        let o = await s.reschedule(store: st, reason: .appOpen, now: now, zone: vn)
        XCTAssertEqual(fake.requests, 0)
        XCTAssertEqual(o.added, 0)
        fake.grant = false
        await s.reschedule(store: st, reason: .save, now: now, zone: vn)
        XCTAssertEqual(fake.requests, 1)
        XCTAssertTrue(fake.store.isEmpty, "bị từ chối → không đặt gì")
    }

    func testDeviceZone() {
        let z = DeviceZones.current(at: now, tz: TimeZone(identifier: "Asia/Ho_Chi_Minh")!)
        XCTAssertEqual(z.offsetMin, 420)
        XCTAssertTrue(z.transitions.isEmpty)
        let b = DeviceZones.current(at: now, tz: TimeZone(identifier: "Europe/Berlin")!)
        XCTAssertEqual(b.offsetMin, 60)
        XCTAssertGreaterThanOrEqual(b.transitions.count, 2)
        XCTAssertEqual(zip(b.transitions, b.transitions.dropFirst()).allSatisfy { $0.atUnixS < $1.atUnixS }, true)
    }

    func testPendingJSONMatchesCoreContent() {
        let p = [PendingNotif(id: "lvhv:sentinel", atUnix: 100, title: "T", body: ""),
                 PendingNotif(id: "lvhv:x:2026:0", atUnix: 200, title: "A", body: "B")]
        let j = JV.parse(ReminderScheduler.pendingJSON(p))!
        XCTAssertEqual(j[0]["content"].str, "T")
        XCTAssertEqual(j[1]["content"].str, "A\nB")
        let c = ReminderScheduler.components("2026-07-25T07:05")!
        XCTAssertEqual([c.year, c.month, c.day, c.hour, c.minute], [2026, 7, 25, 7, 5])
        XCTAssertNil(c.timeZone)
    }

    // MARK: Ảnh (S9)

    private func jpegWithMetadata(width: Int, height: Int) -> Data {
        let ctx = CGContext(data: nil, width: width, height: height, bitsPerComponent: 8, bytesPerRow: 0,
                            space: CGColorSpace(name: CGColorSpace.sRGB)!, bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue)!
        ctx.setFillColor(CGColor(red: 0.2, green: 0.5, blue: 0.8, alpha: 1))
        ctx.fill(CGRect(x: 0, y: 0, width: width, height: height))
        let img = ctx.makeImage()!
        let out = NSMutableData()
        let dest = CGImageDestinationCreateWithData(out, UTType.jpeg.identifier as CFString, 1, nil)!
        let props: [CFString: Any] = [
            kCGImagePropertyOrientation: 6,
            kCGImagePropertyGPSDictionary: [kCGImagePropertyGPSLatitude: 21.03, kCGImagePropertyGPSLatitudeRef: "N",
                                            kCGImagePropertyGPSLongitude: 105.85, kCGImagePropertyGPSLongitudeRef: "E"],
            kCGImagePropertyTIFFDictionary: [kCGImagePropertyTIFFMake: "MayChupThu", kCGImagePropertyTIFFModel: "X1"],
            kCGImagePropertyIPTCDictionary: [kCGImagePropertyIPTCCaption: "chu thich"],
        ]
        CGImageDestinationAddImage(dest, img, props as CFDictionary)
        CGImageDestinationFinalize(dest)
        return out as Data
    }

    func testPhotoIsReencodedRotatedAndStripped() throws {
        let input = jpegWithMetadata(width: 2000, height: 1000)
        XCTAssertTrue(JPEGSanitizer.markers(input).contains(0xE1), "ảnh đầu vào phải có EXIF để phép thử có nghĩa")
        let p = try PhotoProcessor.process(input)
        let mk = JPEGSanitizer.markers(p.jpeg)
        XCTAssertFalse(mk.contains(0xE1), "không còn APP1 (EXIF/XMP)")
        XCTAssertFalse(mk.contains(0xED), "không còn APP13 (IPTC)")
        XCTAssertEqual(p.width, 512)  // đã xoay 90° theo EXIF: 1000×2000 → cạnh dài 1024
        XCTAssertEqual(p.height, 1024)
        XCTAssertLessThanOrEqual(p.jpeg.count, 1_048_576)
        XCTAssertFalse(String(decoding: p.jpeg, as: UTF8.self).contains("MayChupThu"))
        XCTAssertThrowsError(try PhotoProcessor.process(Data("khong phai anh".utf8))) { e in
            XCTAssertEqual(e as? PhotoError, .unreadable)
        }
    }

    func testIcsPreviewLines() {
        let ics = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nDTSTART;VALUE=DATE:20260728\r\nSUMMARY:Giỗ đầu Ông\r\n  Nguyễn Văn A\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
        XCTAssertEqual(IcsPreview.lines(ics), ["28/07/2026 — Giỗ đầu Ông Nguyễn Văn A"])
    }
}
