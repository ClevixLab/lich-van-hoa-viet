// Điểm vào ứng dụng (iOS và macOS dùng chung).
import SwiftUI
import UserNotifications

@main
struct LichVietApp: App {
    init() {
        #if DEBUG
        DemoSeed.resetIfRequested()
        #endif
        // Chạm vào thông báo nhắc → mở "Người thân" và lập lịch lại (02 §7.4).
        UNUserNotificationCenter.current().delegate = NotificationDelegate.shared
        AttachmentTemp.clear() // tệp tạm xem tệp đính kèm còn sót từ lần trước (§5.3)
    }

    var body: some Scene {
        #if os(macOS)
        WindowGroup("Lịch và Văn hóa Việt") {
            RootView()
        }
        .defaultSize(width: 760, height: 900)
        .commands {
            CommandGroup(replacing: .newItem) {}
        }
        #else
        WindowGroup {
            RootView()
        }
        #endif
    }
}
