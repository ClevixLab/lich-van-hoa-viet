// Điểm vào ứng dụng (iOS và macOS dùng chung).
import SwiftUI

@main
struct LichVietApp: App {
    var body: some Scene {
        #if os(macOS)
        WindowGroup("Lịch và Văn hóa Việt") {
            RootView()
        }
        .defaultSize(width: 560, height: 900)
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
