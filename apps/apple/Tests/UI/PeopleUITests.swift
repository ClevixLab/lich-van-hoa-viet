// XCUITest cho mô-đun "Người thân": luồng chính (L1, L5, US-M-07) và ảnh chụp mọi màn hình mới ở sáng, tối và chữ
// rất lớn. Ảnh được ghi vào thư mục SCREENSHOT_DIR (CI truyền qua TEST_RUNNER_SCREENSHOT_DIR) và đính kèm kết quả.
import XCTest

final class PeopleUITests: XCTestCase {
    override func setUp() { continueAfterFailure = false }

    private func launch(screen: String, seed: String = "demo", dark: Bool = false, big: Bool = false,
                        extra: [String] = []) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchArguments = ["-lvhvReset", "1", "-lvhvSeed", seed, "-lvhvToday", "2026-11-07", "-lvhvNoNotif", "1",
                               "-lvhvScreen", screen, "-theme", dark ? "2" : "1", "-size", "1"] + extra
        if big {
            app.launchArguments += ["-UIPreferredContentSizeCategoryName", "UICTContentSizeCategoryAccessibilityL"]
        }
        app.launch()
        return app
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
        ("form-deceased", 4), ("form-living", 2), ("event", 1), ("event-deceased", 1), ("mark", 2), ("memorial", 1),
        ("memorial-a", 1), ("settings", 1),
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

    func testScreenshotsEmptyAndDialogs() {
        var app = launch(screen: "upcoming", seed: "none")
        snap("empty-light")
        app.terminate()
        for dark in [false, true] {
            let v = dark ? "dark" : "light"
            app = launch(screen: "backup", dark: dark)
            app.buttons["backup.ics"].tap()
            snap("ics-confirm-\(v)")
            app.terminate()
            app = launch(screen: "backup", dark: dark)
            app.buttons["backup.export"].tap()
            snap("backup-export-\(v)")
            app.terminate()
            app = launch(screen: "person-deceased", dark: dark)
            app.buttons["person.delete"].firstMatch.tap()
            snap("delete-confirm-\(v)")
            app.terminate()
        }
        for dark in [false, true] {
            app = launch(screen: "person-notice", dark: dark, extra: ["-lvhvExpand", "1"])
            app.swipeUp(velocity: .slow)
            snap("person-notice-\(dark ? "dark" : "light")")
            app.terminate()
        }
        app = launch(screen: "period", extra: ["-lvhvPeriodKind", "quy"])
        snap("period-quy-light")
        app.terminate()
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
        snap("mark-confirm-light")
        ok.tap()
        XCTAssertTrue(app.buttons["person.memorial"].waitForExistence(timeout: 5))
        XCTAssertTrue(app.otherElements["person.notice"].exists || app.buttons["person.notice"].exists
                      || app.staticTexts["Thông tin báo tử"].exists)
    }
}
