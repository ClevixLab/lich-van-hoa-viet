// Phiên bản, nhắc cập nhật và nút cập nhật — KHÔNG dùng mạng.
//
// Ứng dụng không biết bản mới nhất là bản nào; nó chỉ biết ngày dựng (LVHVBuildDate trong Info.plist,
// gen-version.sh ghi lúc dựng). Bản dựng hơn 60 ngày thì hiện thẻ nhắc; "Để sau" ẩn 30 ngày (một
// khóa UserDefaults). Nút cập nhật giao địa chỉ cho hệ điều hành mở (App Store hoặc Safari); macOS chạy
// trong App Sandbox không có quyền mạng, iOS không gọi mạng.
import SwiftUI
#if os(iOS)
import UIKit
#else
import AppKit
#endif

enum AppUpdate {
    static let remindAfterDays = 60
    static let snoozeDays = 30
    static let downloadPage = "https://clevixlab.github.io/lich-van-hoa-viet/"

    static let button = "Kiểm tra cập nhật / Cập nhật"
    static let explain = "Ứng dụng không tự kết nối mạng để bảo vệ quyền riêng tư. Bấm nút để mở cửa hàng ứng dụng hoặc trang tải về và cập nhật lên phiên bản mới nhất."
    static let reminder = "Phiên bản này đã phát hành hơn 2 tháng. Hãy kiểm tra cập nhật để có dữ liệu và tính năng mới nhất."

    private static func info(_ key: String) -> String {
        (Bundle.main.object(forInfoDictionaryKey: key) as? String)?.trimmingCharacters(in: .whitespaces) ?? ""
    }

    static var version: String { info("CFBundleShortVersionString") }
    static var build: String { info("CFBundleVersion") }
    /// Mã số App Store (build setting APP_STORE_ID). Trống cho tới khi có bản ghi App Store thật.
    static var appStoreID: String { info("LVHVAppStoreID") }

    /// Dòng bản quyền (dòng 1 chân trang Giới thiệu); ký hiệu © được vẽ nhỏ và nhẹ hơn chữ.
    static let copyrightSign = "©"
    static let copyrightHolder = "\u{00A0}2026 Clevix Lab"
    static var copyright: String { copyrightSign + copyrightHolder }
    /// Dòng 2 chân trang Giới thiệu: số phiên bản, chữ nhỏ và nhạt.
    static var versionLine: String { "Phiên bản \(version)" }
    static var runningVersion: String { "Phiên bản đang dùng: \(version) (bản dựng \(build))" }

    /// itms-apps://apps.apple.com/app/id<ID> khi có APP_STORE_ID (chỉ chữ số), nếu không: trang tải về.
    static func updateURL(storeID: String) -> URL {
        if !storeID.isEmpty, storeID.allSatisfy(\.isNumber), let u = URL(string: "itms-apps://apps.apple.com/app/id\(storeID)") {
            return u
        }
        return URL(string: downloadPage)!
    }

    static func openStore() {
        let url = updateURL(storeID: appStoreID)
        #if os(iOS)
        UIApplication.shared.open(url) { ok in
            if !ok, url.absoluteString != downloadPage { UIApplication.shared.open(URL(string: downloadPage)!) }
        }
        #else
        if !NSWorkspace.shared.open(url), url.absoluteString != downloadPage {
            NSWorkspace.shared.open(URL(string: downloadPage)!)
        }
        #endif
    }

    // ---- Nhắc cập nhật ----

    private static var utc: Calendar {
        var c = Calendar(identifier: .gregorian)
        c.timeZone = TimeZone(identifier: "UTC")!
        return c
    }

    /// "YYYY-MM-DD" → ngày (UTC). Sai định dạng → nil (khi đó không nhắc).
    static func parseDate(_ s: String) -> Date? {
        let p = s.split(separator: "-").compactMap { Int($0) }
        guard s.count == 10, p.count == 3, (1...12).contains(p[1]), (1...31).contains(p[2]) else { return nil }
        return utc.date(from: DateComponents(year: p[0], month: p[1], day: p[2]))
    }

    static var buildDate: Date? { parseDate(info("LVHVBuildDate")) }

    static func shouldRemind(buildDate: Date?, now: Date, snoozedUntil: Date?) -> Bool {
        guard let b = buildDate else { return false }
        let days = utc.dateComponents([.day], from: b, to: now).day ?? 0
        if days <= remindAfterDays { return false }
        if let u = snoozedUntil, now < u { return false }
        return true
    }

    static func snoozeUntil(from now: Date) -> Date {
        utc.date(byAdding: .day, value: snoozeDays, to: now) ?? now
    }
}

/// Thẻ nhắc cập nhật nhẹ nhàng, không chặn thao tác, ở đầu màn hình Tờ lịch.
struct UpdateReminderCard: View {
    var onCheck: () -> Void
    var onLater: () -> Void
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(AppUpdate.reminder).sans(14).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
            HStack {
                Spacer()
                Button("Để sau", action: onLater).buttonStyle(.plain).foregroundStyle(C.ink2)
                    .padding(.horizontal, 10)
                Button(action: onCheck) {
                    Text("Kiểm tra cập nhật").sans(14, .semibold).foregroundStyle(.white)
                        .padding(.horizontal, 14).padding(.vertical, 8)
                        .background(C.redDeep, in: Capsule())
                }
                .buttonStyle(.plain)
            }
        }
        .padding(12)
        .frame(maxWidth: 560, alignment: .leading)
        .background(C.goldTint, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .padding(.horizontal, 16)
        .padding(.vertical, 4)
    }
}
