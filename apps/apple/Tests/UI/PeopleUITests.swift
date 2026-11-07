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
        ("upcoming-denied", 1), ("store-error", 1), ("backup-nag", 1), ("form-edit", 4), ("memorial-cg", 2),
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

    /// BM3-03 (S10): ảnh đính kèm xem ngay trong ứng dụng (không có Chia sẻ); PDF phải qua hộp hỏi lại với "Hủy" mặc định —
    /// bấm Hủy thì không mở gì, bấm "Mở tệp" mới mở trình xem của hệ thống. Chụp ở sáng, tối, chữ rất lớn.
    func testAttachmentViewing() {
        for (v, dark, big) in [("light", false, false), ("dark", true, false), ("large", false, true)] {
            let app = launch(screen: "person-notice", dark: dark, big: big, extra: ["-lvhvExpand", "1"])
            let img = app.buttons["person.att.view.0"].firstMatch
            XCTAssertTrue(img.waitForExistence(timeout: 10))
            tapScrolling(app, img)
            XCTAssertTrue(app.descendants(matching: .any)["att.image"].firstMatch.waitForExistence(timeout: 5),
                          "ảnh đính kèm phải mở ngay trong ứng dụng")
            XCTAssertFalse(app.buttons["Share"].exists || app.buttons["Chia sẻ"].exists, "trang xem ảnh không có nút Chia sẻ")
            snap("att-image-\(v)")
            app.buttons["att.image.close"].firstMatch.tap()

            let pdf = app.buttons["person.att.view.1"].firstMatch
            tapScrolling(app, pdf)
            let alert = app.alerts.firstMatch
            XCTAssertTrue(alert.waitForExistence(timeout: 5), "PDF phải hỏi lại trước khi mở Quick Look")
            XCTAssertTrue(alert.staticTexts.containing(NSPredicate(format: "label CONTAINS %@", "Chia sẻ")).firstMatch.exists)
            snap("att-pdf-confirm-\(v)")
            alert.buttons["Hủy"].tap()
            var w = 0
            while alert.exists && w < 25 { Thread.sleep(forTimeInterval: 0.2); w += 1 }
            XCTAssertFalse(alert.exists)
            Thread.sleep(forTimeInterval: 1.5)
            XCTAssertTrue(pdf.isHittable, "bấm Hủy thì không mở trình xem")

            if v == "light" {
                tapScrolling(app, pdf)
                XCTAssertTrue(alert.waitForExistence(timeout: 5))
                alert.buttons["Mở tệp"].tap()
                Thread.sleep(forTimeInterval: 2.5)
                XCTAssertFalse(pdf.isHittable, "bấm Mở tệp thì trình xem của hệ thống phủ lên")
                snap("att-pdf-open-\(v)")
            }
            app.terminate()
        }
    }

    /// DV23-01, DV23-03: tấm khôi phục có hai hàng chọn to theo chữ; khi đang khôi phục, nút Hủy bị khóa.
    func testRestoreSheetLocksCancelWhileRunning() {
        let app = launch(screen: "restore", extra: ["-lvhvSlowRestore", "1"])
        // Tấm mở sau khi bản Debug tạo và giải mã tệp sao lưu mẫu (Argon2id) — chờ chính hàng chọn.
        let merge = app.descendants(matching: .any)["restore.merge"].firstMatch
        let replace = app.descendants(matching: .any)["restore.replace"].firstMatch
        XCTAssertTrue(merge.waitForExistence(timeout: 60), "tấm khôi phục có hàng Gộp")
        XCTAssertTrue(replace.exists)
        XCTAssertTrue(merge.isSelected, "mặc định là Gộp")
        XCTAssertFalse(replace.isSelected)
        let cancel = app.buttons["restore.cancel"].firstMatch
        XCTAssertTrue(cancel.isEnabled)
        app.navigationBars.buttons["Khôi phục"].firstMatch.tap()
        Thread.sleep(forTimeInterval: 0.8)
        XCTAssertFalse(cancel.isEnabled, "đang khôi phục thì không đóng được tấm")
        snap("restore-running-light")
        app.terminate()
    }

    // MARK: Luồng

    /// L1: thêm người đã mất theo ngày dương → thấy "15/6 nhuận" → lưu → trang hồ sơ.
    func testAddDeceasedFlow() {
        let app = launch(screen: "upcoming", seed: "none")
        let add = app.buttons["people.addDeceased"].firstMatch
        XCTAssertTrue(add.waitForExistence(timeout: 15))
        Thread.sleep(forTimeInterval: 1)
        add.tap()
        let name = app.textFields["form.hoTen"]
        // Lần chạy đầu trên máy mô phỏng mới có thể nuốt mất cú chạm khi trang còn đang dựng: thử chạm lại một lần.
        if !name.waitForExistence(timeout: 8), add.exists, add.isHittable { add.tap() }
        XCTAssertTrue(name.waitForExistence(timeout: 15))
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

    // MARK: YC-2026-10 — ảnh mâm lễ (08 §16.4)

    /// YC-2026-10 (08 §16.4 ca 1 và 4): chưa có ảnh mâm lễ, gói ảnh Clevix rỗng → hàng bàn thờ chỉ có hai nến và bát hương
    /// ba nén, không có mâm; Công giáo → nến, hoa, nến, không hương. Chụp sáng và tối (iPhone, và iPad trên CI).
    func testMemorialAltar() {
        for (v, dark) in [("light", false), ("dark", true)] {
            for (screen, catholic) in [("memorial", false), ("memorial-cg", true)] {
                let app = launch(screen: screen, dark: dark)
                let altar = app.descendants(matching: .any)["memorial.altar"].firstMatch
                XCTAssertTrue(altar.waitForExistence(timeout: 15))
                XCTAssertEqual(app.descendants(matching: .any).matching(identifier: "memorial.candle").count, 2)
                XCTAssertEqual(app.descendants(matching: .any)["memorial.incense"].exists, !catholic)
                XCTAssertEqual(app.descendants(matching: .any)["memorial.flowers"].exists, catholic)
                XCTAssertFalse(app.descendants(matching: .any)["memorial.tray"].exists, "không có mâm khi chưa có ảnh")
                XCTAssertFalse(altar.label.contains("mâm"), "nhãn trợ năng không nhắc mâm")
                if !catholic { XCTAssertTrue(app.buttons["memorial.smoke"].exists) } else { XCTAssertFalse(app.buttons["memorial.smoke"].exists) }
                snap("altar-\(screen == "memorial" ? "huong" : "cong-giao")-\(v)")
                // V2-01: cuộn để thấy cả hàng bàn thờ (và nút thêm ảnh mâm lễ) trong một ảnh.
                altar.swipeUp(velocity: .slow)
                snap("altar-\(screen == "memorial" ? "huong" : "cong-giao")-\(v)-2")
                app.terminate()
            }
        }
    }

    /// YC-2026-10 (08 §16.4 ca 2, 3, 5) — ảnh mâm lễ: ảnh gia đình hiện dưới bàn thờ (không còn nút thêm); bỏ ảnh trong
    /// biểu mẫu rồi lưu → trang hồ sơ "Chưa có"; gói thử có một ảnh đã duyệt → ảnh Clevix hiện khi chưa có ảnh gia đình,
    /// ảnh gia đình thắng khi có; nút "Thêm ảnh mâm lễ của gia đình" mở biểu mẫu ở đúng mục.
    func testTrayPhoto() {
        let any = { (app: XCUIApplication, id: String) in app.descendants(matching: .any)[id].firstMatch }
        for (v, dark) in [("light", false), ("dark", true)] {
            // Ca 2: ảnh gia đình.
            var app = launch(screen: "memorial-tray", dark: dark)
            XCTAssertTrue(any(app, "memorial.tray").waitForExistence(timeout: 15))
            XCTAssertEqual(any(app, "memorial.tray").label, "Ảnh mâm lễ của gia đình")
            XCTAssertFalse(any(app, "memorial.addTray").exists, "có ảnh gia đình thì không có nút thêm")
            snap("altar-tray-\(v)")
            app.swipeUp(velocity: .slow)
            snap("altar-tray-\(v)-2")
            app.terminate()

            // Biểu mẫu: chưa có ảnh / có ảnh.
            app = launch(screen: "form-tray-empty", dark: dark)
            XCTAssertTrue(any(app, "form.tray.none").waitForExistence(timeout: 15))
            snap("form-tray-empty-\(v)")
            app.terminate()
            app = launch(screen: "form-tray", dark: dark)
            XCTAssertTrue(any(app, "form.tray.thumb").waitForExistence(timeout: 15))
            XCTAssertEqual(any(app, "form.tray.thumb").label, "Ảnh mâm lễ đã chọn")
            XCTAssertEqual(app.buttons["form.tray.remove"].firstMatch.label, "Bỏ ảnh mâm lễ")
            snap("form-tray-\(v)")
            app.terminate()
            if !dark {
                app = launch(screen: "form-tray", big: true)
                XCTAssertTrue(any(app, "form.tray.thumb").waitForExistence(timeout: 15))
                snap("form-tray-large")
                app.swipeUp(velocity: .slow)
                snap("form-tray-large-2")
                app.terminate()
            }

            // Ca 5: gói thử — ảnh Clevix khi chưa có ảnh gia đình (có dòng ghi công) …
            app = launch(screen: "memorial", dark: dark, extra: ["-lvhvTrayPack", "test"])
            XCTAssertTrue(any(app, "memorial.tray").waitForExistence(timeout: 15))
            XCTAssertEqual(any(app, "memorial.tray").label, "Ảnh mâm lễ")
            XCTAssertTrue(any(app, "memorial.trayCredit").exists)
            XCTAssertTrue(any(app, "memorial.addTray").exists)
            snap("altar-clevix-\(v)")
            // V2-02: cuộn để thấy ảnh Clevix dưới bàn thờ cùng dòng ghi công.
            app.swipeUp(velocity: .slow)
            snap("altar-clevix-\(v)-2")
            app.terminate()
            // … ảnh gia đình thắng khi có.
            app = launch(screen: "memorial-tray", dark: dark, extra: ["-lvhvTrayPack", "test"])
            XCTAssertTrue(any(app, "memorial.tray").waitForExistence(timeout: 15))
            XCTAssertEqual(any(app, "memorial.tray").label, "Ảnh mâm lễ của gia đình")
            XCTAssertFalse(any(app, "memorial.trayCredit").exists)
            app.terminate()
        }

        // Ca 3: bỏ ảnh rồi lưu → trang hồ sơ "Ảnh mâm lễ: Chưa có", màn hình tưởng niệm không còn mâm.
        var app = launch(screen: "form-tray")
        let remove = app.buttons["form.tray.remove"].firstMatch
        XCTAssertTrue(remove.waitForExistence(timeout: 15))
        tapScrolling(app, remove)
        XCTAssertTrue(any(app, "form.tray.none").waitForExistence(timeout: 5))
        app.buttons["form.saveTop"].firstMatch.tap()
        XCTAssertTrue(app.staticTexts["Chưa có"].firstMatch.waitForExistence(timeout: 10), "trang hồ sơ: Ảnh mâm lễ — Chưa có")
        app.terminate()

        // Nút trên màn hình tưởng niệm mở biểu mẫu ở mục ảnh mâm lễ.
        app = launch(screen: "memorial")
        let add = app.buttons["memorial.addTray"].firstMatch
        XCTAssertTrue(add.waitForExistence(timeout: 15))
        tapScrolling(app, add)
        XCTAssertTrue(any(app, "form.tray.none").waitForExistence(timeout: 10))
        Thread.sleep(forTimeInterval: 1)
        XCTAssertTrue(app.buttons["form.tray.pick"].firstMatch.isHittable, "biểu mẫu cuộn tới mục ảnh mâm lễ")
        app.terminate()
    }
}
