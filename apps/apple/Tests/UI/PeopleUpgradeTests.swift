// R14 (04 Bổ sung B5) — chạy trong job `upgrade` của CI sau khi bản cũ đã tạo dữ liệu trên máy mô phỏng.
// `xcodebuild test` cài bản mới ĐÈ lên bản cũ (cùng bundle id, giữ thư mục dữ liệu). Kiểm thử này KHÔNG xóa kho.
// Bỏ qua khi biến môi trường LVHV_UPGRADE không đặt (job kiểm thử thường).
import XCTest

final class PeopleUpgradeTests: XCTestCase {
    override func setUp() { continueAfterFailure = false }

    private func value(_ report: String, _ key: String) -> String? {
        for line in report.split(separator: "\n") {
            for part in line.split(separator: " ") {
                let kv = part.split(separator: "=", maxSplits: 1)
                if kv.count == 2, kv[0] == key { return String(kv[1]) }
            }
        }
        return nil
    }

    private func shot(_ name: String) {
        let s = XCUIScreen.main.screenshot()
        let a = XCTAttachment(screenshot: s)
        a.name = name
        a.lifetime = .keepAlways
        add(a)
        if let dir = ProcessInfo.processInfo.environment["SCREENSHOT_DIR"], !dir.isEmpty {
            try? FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)
            try? s.pngRepresentation.write(to: URL(fileURLWithPath: dir).appendingPathComponent("\(name).png"))
        }
    }

    func testDataSurvivesUpgrade() throws {
        let env = ProcessInfo.processInfo.environment
        guard let expect = env["LVHV_UPGRADE"], !expect.isEmpty else { throw XCTSkip("Chỉ chạy trong job upgrade") }
        let app = XCUIApplication()
        app.launchArguments = ["-lvhvScreen", "diag", "-lvhvToday", "2026-11-07"]
        app.launch()
        let report = app.staticTexts["diag.report"]
        XCTAssertTrue(report.waitForExistence(timeout: 20))
        Thread.sleep(forTimeInterval: 2)
        app.buttons["diag.refresh"].tap()
        Thread.sleep(forTimeInterval: 2)
        let r0 = report.label
        print("R14 trước khi cấp quyền:\n\(r0)")
        XCTAssertEqual(value(r0, "migration"), "migrated", "kho 0.2-dev (lược đồ 1) phải được chuyển sang 2: \(r0)")
        XCTAssertEqual(value(r0, "persons"), "5", r0)
        XCTAssertEqual(value(r0, "events"), "1", r0)
        XCTAssertEqual(value(r0, "notices"), "1", r0)
        XCTAssertEqual(value(r0, "photos"), value(r0, "photos_ok"), r0)
        XCTAssertEqual(value(r0, "photos"), "1", "ảnh chân dung: \(r0)")
        // 01 §7.1: ảnh giấy báo tử của bản 0.2-dev → tệp đính kèm loại giấy chứng tử, đọc lại được; khóa `legacy` giữ bản gốc.
        XCTAssertEqual(value(r0, "attachments"), "1", r0)
        XCTAssertEqual(value(r0, "attachments_ok"), "1", r0)
        XCTAssertEqual(value(r0, "legacy"), "1", r0)
        XCTAssertEqual(value(r0, "same_as_before"), "1", "dữ liệu sau nâng cấp phải giống hệt bản chép trước khi cài đè: \(r0)")
        // Lịch nhắc được đặt lại sau nâng cấp: cho phép thông báo (hộp của hệ thống) → lập lịch lại.
        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        app.buttons["diag.allow"].tap()
        for label in ["Allow", "Cho phép"] {
            let b = springboard.buttons[label]
            if b.waitForExistence(timeout: 5) { b.tap(); break }
        }
        Thread.sleep(forTimeInterval: 3)
        app.buttons["diag.refresh"].tap()
        Thread.sleep(forTimeInterval: 2)
        let r1 = report.label
        print("R14 sau khi cấp quyền:\n\(r1)")
        XCTAssertGreaterThan(Int(value(r1, "pending") ?? "0") ?? 0, 0, r1)
        shot("upgrade-diag")
        app.terminate()
        // Mở lại như người dùng: lập lịch lại khi mở ứng dụng giữ nguyên số thông báo chờ.
        app.launch()
        XCTAssertTrue(report.waitForExistence(timeout: 20))
        Thread.sleep(forTimeInterval: 3)
        app.buttons["diag.refresh"].tap()
        Thread.sleep(forTimeInterval: 2)
        XCTAssertEqual(value(report.label, "pending"), value(r1, "pending"), report.label)
    }

    /// R14 ca lỗi: ảnh giấy báo tử của kho cũ bị hỏng → lõi không chuyển, mở chỉ đọc, dữ liệu cũ còn nguyên.
    func testFailedMigrationIsReadOnly() throws {
        guard ProcessInfo.processInfo.environment["LVHV_UPGRADE_FAIL"] == "1" else { throw XCTSkip("Chỉ chạy trong job upgrade") }
        let app = XCUIApplication()
        app.launchArguments = ["-lvhvScreen", "diag", "-lvhvToday", "2026-11-07"]
        app.launch()
        let report = app.staticTexts["diag.report"]
        XCTAssertTrue(report.waitForExistence(timeout: 20))
        Thread.sleep(forTimeInterval: 2)
        app.buttons["diag.refresh"].tap()
        Thread.sleep(forTimeInterval: 1)
        let r = report.label
        print("R14 ca lỗi:\n\(r)")
        XCTAssertEqual(value(r, "migration"), "read_only", r)
        XCTAssertEqual(value(r, "persons"), "5", "dữ liệu cũ vẫn xem được: \(r)")
        // DV23-02: câu chuẩn của 08 §12l (TKN3-X-03); không có mã kỹ thuật trong lời chính.
        let text = app.staticTexts["people.readOnly.text"].firstMatch
        XCTAssertTrue(text.waitForExistence(timeout: 5))
        XCTAssertTrue(text.label.hasPrefix("Chưa chuyển được dữ liệu sang bản mới."), text.label)
        XCTAssertFalse(text.label.lowercased().contains("mime"), text.label)
        XCTAssertNil(text.label.range(of: "[0-9a-f]{8}-[0-9a-f]{4}-", options: .regularExpression), "không hiện mã UUID")
        XCTAssertTrue(app.buttons["people.readOnly.backup"].exists, "nút Đi tới Sao lưu")
        XCTAssertFalse(app.staticTexts.containing(NSPredicate(format: "label CONTAINS 'mime_'")).firstMatch.exists,
                       "câu của lõi chỉ nằm trong mục Chi tiết kỹ thuật đang đóng")
        shot("upgrade-readonly-diag")
        app.terminate()
        app.launchArguments = ["-lvhvScreen", "profiles", "-lvhvToday", "2026-11-07"]
        app.launch()
        Thread.sleep(forTimeInterval: 3)
        shot("upgrade-readonly-profiles")
        // TKN3-P-01: ẩn mọi việc ghi.
        XCTAssertFalse(app.buttons["profiles.add.deceased"].exists, "chỉ đọc: không có nút thêm hồ sơ")
        XCTAssertFalse(app.buttons["profiles.add.living"].exists)
        let first = app.buttons.matching(NSPredicate(format: "label CONTAINS 'Nguyễn Văn A'")).firstMatch
        XCTAssertTrue(first.waitForExistence(timeout: 10), "chỉ đọc: vẫn mở được hồ sơ để kiểm không có nút ghi")
        do {
            first.tap()
            Thread.sleep(forTimeInterval: 2)
            for id in ["person.edit", "person.delete", "person.addEvent", "person.markDeceased", "person.event.edit",
                       "person.event.delete"] {
                XCTAssertFalse(app.buttons[id].exists, "chỉ đọc: không có \(id)")
            }
            shot("upgrade-readonly-person")
        }
        app.terminate()
        app.launchArguments = ["-lvhvScreen", "backup", "-lvhvToday", "2026-11-07"]
        app.launch()
        Thread.sleep(forTimeInterval: 3)
        XCTAssertFalse(app.buttons["backup.pick"].exists, "chỉ đọc: không có khối Khôi phục")
        XCTAssertTrue(app.buttons["backup.export"].exists, "vẫn tạo được tệp sao lưu")
        shot("upgrade-readonly-backup")
    }
}
