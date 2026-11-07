// XCUITest cho mô-đun "Người thân": luồng chính (L1, L5, US-M-07) và ảnh chụp mọi màn hình mới ở sáng, tối và chữ
// rất lớn. Ảnh được ghi vào thư mục SCREENSHOT_DIR (CI truyền qua TEST_RUNNER_SCREENSHOT_DIR) và đính kèm kết quả.
import XCTest

final class PeopleUITests: XCTestCase {
    override func setUp() { continueAfterFailure = false }

    private func launch(screen: String, seed: String = "demo", dark: Bool = false, big: Bool = false,
                        extra: [String] = []) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchArguments = ["-lvhvReset", "1", "-lvhvSeed", seed, "-lvhvToday", "2026-11-07", "-lvhvNoNotif", "1",
                               "-lvhvScreen", screen, "-theme", dark ? "2" : "1", "-size", "1"] + Self.extras(screen) + extra
        if big {
            app.launchArguments += ["-UIPreferredContentSizeCategoryName", "UICTContentSizeCategoryAccessibilityL"]
        }
        app.launch()
        return app
    }

    /// Tham số thêm cho từng màn hình kiểm thử (chỉ bản Debug).
    static func extras(_ screen: String) -> [String] {
        switch screen {
        case "upcoming-denied": return ["-lvhvFakeDenied", "1"]
        case "backup-nag": return ["-lvhvNagDue", "1"]
        case "form-am-nhuan": return ["-lvhvPrefill", "am-nhuan"]
        case "form-duong": return ["-lvhvPrefill", "duong"]
        case "form-loi": return ["-lvhvPrefill", "loi"]
        case "form-living-am": return ["-lvhvPrefill", "living-am"]
        default: return []
        }
    }

    private var prefix: String { ProcessInfo.processInfo.environment["SHOT_PREFIX"] ?? "ios" }

    private func snap(_ name: String) {
        Thread.sleep(forTimeInterval: 1.2)
        let shot = XCUIScreen.main.screenshot()
        let a = XCTAttachment(screenshot: shot)
        a.name = "\(prefix)-\(name)"
        a.lifetime = .keepAlways
        add(a)
        if let dir = ProcessInfo.processInfo.environment["SCREENSHOT_DIR"], !dir.isEmpty {
            try? FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)
            try? shot.pngRepresentation.write(to: URL(fileURLWithPath: dir).appendingPathComponent("\(prefix)-\(name).png"))
        }
    }

    // MARK: Ảnh chụp

    /// Màn hình mở thẳng bằng -lvhvScreen; `scroll` = chụp thêm ảnh sau khi cuộn.
    static let screens: [(String, Int)] = [
        ("upcoming", 1), ("period", 1), ("profiles", 1), ("backup", 1), ("person-deceased", 2), ("person-living", 1),
        ("form-deceased", 4), ("form-living", 2), ("event", 1), ("event-deceased", 1), ("mark", 2), ("memorial", 2),
        ("memorial-a", 2), ("settings", 2),
        ("form-am-nhuan", 2), ("form-duong", 3), ("form-loi", 2), ("form-living-am", 3),
        ("upcoming-denied", 1), ("store-error", 1), ("backup-nag", 1), ("form-edit", 4),
    ]

    private func shoot(variant: String, dark: Bool, big: Bool, screens: [(String, Int)]) {
        for (s, n) in screens {
            let app = launch(screen: s, dark: dark, big: big)
            XCTAssertTrue(app.wait(for: .runningForeground, timeout: 10))
            snap("\(s)-\(variant)")
            if n > 1 {
                for i in 2...n {
                    app.swipeUp(velocity: .slow)
                    snap("\(s)-\(variant)-\(i)")
                }
            }
            app.terminate()
        }
    }

    func testScreenshotsLight() { shoot(variant: "light", dark: false, big: false, screens: Self.screens) }
    func testScreenshotsDark() { shoot(variant: "dark", dark: true, big: false, screens: Self.screens) }
    func testScreenshotsLargeType() { shoot(variant: "large", dark: false, big: true, screens: Self.screens) }

    /// Cuộn tới khi phần tử bấm được rồi bấm.
    private func tapScrolling(_ app: XCUIApplication, _ el: XCUIElement) {
        var n = 0
        while !(el.exists && el.isHittable) && n < 10 { app.swipeUp(velocity: .slow); n += 1 }
        el.tap()
    }

    /// Trạng thái trống, hộp thoại và trang có khối mở sẵn — ở sáng, tối và chữ rất lớn.
    func testScreenshotsEmptyAndDialogs() {
        for (v, dark, big) in [("light", false, false), ("dark", true, false), ("large", false, true)] {
            var app = launch(screen: "upcoming", seed: "none", dark: dark, big: big)
            snap("empty-\(v)")
            app.terminate()
            app = launch(screen: "backup", dark: dark, big: big)
            tapScrolling(app, app.buttons["backup.ics"])
            snap("ics-confirm-\(v)")
            app.terminate()
            app = launch(screen: "backup", dark: dark, big: big)
            tapScrolling(app, app.buttons["backup.export"])
            snap("backup-export-\(v)")
            app.terminate()
            app = launch(screen: "person-deceased", dark: dark, big: big)
            tapScrolling(app, app.buttons["person.delete"].firstMatch)
            snap("delete-confirm-\(v)")
            app.terminate()
            app = launch(screen: "person-notice", dark: dark, big: big, extra: ["-lvhvExpand", "1"])
            app.swipeUp(velocity: .slow)
            snap("person-notice-\(v)")
            app.terminate()
            app = launch(screen: "period", dark: dark, big: big, extra: ["-lvhvPeriodKind", "quy"])
            snap("period-quy-\(v)")
            app.terminate()
            app = launch(screen: "restore", dark: dark, big: big)
            _ = app.otherElements["restore.sheet"].firstMatch.waitForExistence(timeout: 40)
            snap("restore-preview-\(v)")
            app.terminate()
            app = launch(screen: "restore-replace", dark: dark, big: big)
            _ = app.alerts.firstMatch.waitForExistence(timeout: 40)
            snap("restore-replace-confirm-\(v)")
            app.terminate()
            // Ngày mất điền sẵn (-lvhvPrefill mark) để không phụ thuộc bàn phím mô phỏng.
            app = launch(screen: "mark", dark: dark, big: big, extra: ["-lvhvPrefill", "mark"])
            tapScrolling(app, app.buttons["mark.save"].firstMatch)
            _ = app.alerts.firstMatch.waitForExistence(timeout: 5)
            snap("mark-confirm-\(v)")
            app.terminate()
        }
    }

    // MARK: Luồng

    /// L1: thêm người đã mất theo ngày dương → thấy "15/6 nhuận" → lưu → trang hồ sơ.
    func testAddDeceasedFlow() {
        let app = launch(screen: "upcoming", seed: "none")
        app.buttons["people.addDeceased"].firstMatch.tap()
        let name = app.textFields["form.hoTen"]
        XCTAssertTrue(name.waitForExistence(timeout: 5))
        name.tap(); name.typeText("Nguyễn Văn Thử")
        for (id, v) in [("death.day", "8"), ("death.month", "8"), ("death.year", "2025")] {
            let f = app.textFields[id]
            f.tap(); f.typeText(v)
        }
        let result = app.staticTexts["death.result"]
        XCTAssertTrue(result.waitForExistence(timeout: 5))
        XCTAssertTrue(result.label.contains("15/6 nhuận"), result.label)
        app.buttons["form.saveTop"].firstMatch.tap()
        let title = app.staticTexts["person.name"]
        XCTAssertTrue(title.waitForExistence(timeout: 5))
        XCTAssertEqual(title.label, "Nguyễn Văn Thử")
        snap("flow-added-light")
    }

    /// L5: Quý → "Quý IV/2026"; bộ lọc.
    func testPeriodFlow() {
        let app = launch(screen: "period")
        app.buttons["period.kind.quy"].tap()
        let t = app.staticTexts["period.title"]
        XCTAssertTrue(t.waitForExistence(timeout: 5))
        XCTAssertTrue(t.label.hasPrefix("Quý IV/2026"), t.label)
        app.buttons["Kỳ sau"].tap()
        XCTAssertTrue(app.staticTexts["period.title"].label.hasPrefix("Quý I/2027"))
        app.buttons["period.filter.da_mat"].tap()
        XCTAssertTrue(app.staticTexts["period.count"].waitForExistence(timeout: 5))
    }

    /// US-M-07: người đang sống → Cập nhật: đã mất → hồ sơ có nút Tưởng niệm.
    func testMarkDeceasedFlow() {
        let app = launch(screen: "mark")
        for (id, v) in [("death.day", "1"), ("death.month", "11"), ("death.year", "2026")] {
            let f = app.textFields[id]
            XCTAssertTrue(f.waitForExistence(timeout: 5))
            f.tap(); f.typeText(v)
        }
        let reason = app.textFields["notice.reason"]
        if reason.exists { reason.tap(); reason.typeText("Bệnh") }
        app.buttons["mark.save"].firstMatch.tap()
        let ok = app.alerts.buttons["Lưu thay đổi"]
        XCTAssertTrue(ok.waitForExistence(timeout: 5))
        ok.tap()
        XCTAssertTrue(app.buttons["person.memorial"].waitForExistence(timeout: 5))
        XCTAssertTrue(app.otherElements["person.notice"].exists || app.buttons["person.notice"].exists
                      || app.staticTexts["Thông tin báo tử"].exists)
    }
}
